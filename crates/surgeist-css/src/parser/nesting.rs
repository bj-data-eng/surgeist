use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, match_ignore_ascii_case,
};

use super::queries::parse_media_query_list;
use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::selectors::{SelectorRecovery, parse_nested_style_selector_list};
use super::supports::{parse_supports_condition, with_supports_prelude_context};
use super::{
    CssContainerPrelude, CssScopePrelude, Recovered, StrictDeclarationParser,
    block_item_diagnostic, consume_failed_rule_block, is_declaration_recovery_unit,
    parse_container_prelude, parse_layer_prelude, parse_scope_prelude, parse_scoped_rule_list,
    structural_recovery_action, structural_recovery_production, structural_rule_diagnostic,
    with_container_prelude_context,
};
use crate::error::{
    CssFeatureId, Error, invalid_at_rule_block, invalid_at_rule_placement, invalid_syntax,
    is_nesting_limit_error, with_at_rule_prelude_context, with_media_query_context,
};
use crate::syntax::*;

pub(super) static IMPLEMENTED_SELECTORS: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.selector.nesting")];

pub(super) fn parse_style_rule_block<'i, 't>(
    source: &'i str,
    selectors: CssStyleSelectorList,
    position: crate::CssSourcePosition,
    input: &mut Parser<'i, 't>,
    recovery: RecoveryState,
) -> std::result::Result<Recovered<Vec<CssRule>>, ParseError<'i, Error>> {
    recovery.record_style_context(input.position().byte_index());
    let recovered = parse_style_contents(source, input, recovery)?;
    Ok(Recovered {
        syntax: vec![CssRule::Style(CssStyleRule::new(
            selectors,
            recovered.syntax.declarations,
            recovered.syntax.rules,
            position,
        ))],
        diagnostics: recovered.diagnostics,
    })
}

pub(super) fn parse_style_contents<'i, 't>(
    source: &'i str,
    input: &mut Parser<'i, 't>,
    recovery: RecoveryState,
) -> std::result::Result<Recovered<StyleContents>, ParseError<'i, Error>> {
    recovery.record_style_context(input.position().byte_index());
    let mut body_parser = NestedStyleRuleParser {
        source,
        diagnostics: Vec::new(),
        recovery,
        qualified_resource_error: None,
    };
    let mut declarations = Vec::new();
    let mut rules = Vec::new();
    let mut declaration_buffer = Vec::new();
    let mut previous_end = input.position().byte_index();

    let mut items = RuleBodyParser::new(input, &mut body_parser);
    loop {
        let progress = RecoveryProgress::record(items.input);
        // cssparser's identifier-led declaration fallback discards qualified-rule
        // errors. Preserve only typed resource failures for this one item so they
        // cannot become an ordinary declaration error or leak into the next item.
        items.parser.qualified_resource_error = None;
        let item = items.next();
        let qualified_resource_error = items.parser.qualified_resource_error.take();
        let Some(item) = item else {
            break;
        };
        let (failed_at_block, failed_block_error) = item
            .as_ref()
            .err()
            .map(|(_, failed_unit)| {
                consume_failed_rule_block(
                    source,
                    items.input,
                    true,
                    &items.parser.recovery,
                    structural_recovery_production(failed_unit),
                )
            })
            .unwrap_or((false, None));
        let progress_outcome = progress.finish(items.input, item.is_ok());
        let unit_end = items.input.position().byte_index();
        match item {
            Ok(StyleBlockItem::Declaration(declaration)) => {
                declaration_buffer.push(*declaration);
            }
            Ok(StyleBlockItem::NestedRules(nested_rules)) => {
                flush_declarations(&mut declaration_buffer, &mut declarations, &mut rules);
                rules.extend(nested_rules);
            }
            Err((error, failed_unit))
                if is_declaration_recovery_unit(failed_unit)
                    && !failed_at_block
                    && qualified_resource_error.is_none() =>
            {
                if let Some(diagnostic) = block_item_diagnostic(
                    source,
                    error,
                    failed_unit,
                    unit_end,
                    crate::CssRecoveryAction::DropDeclaration,
                ) {
                    items.parser.diagnostics.push(diagnostic);
                }
            }
            Err((error, failed_unit)) => {
                let error = qualified_resource_error
                    .or(failed_block_error)
                    .unwrap_or(error);
                if let Some(diagnostic) = structural_rule_diagnostic(
                    source,
                    error,
                    failed_unit,
                    previous_end,
                    unit_end,
                    structural_recovery_action(failed_unit),
                ) {
                    items.parser.diagnostics.push(diagnostic);
                }
            }
        }
        previous_end = unit_end;
        if progress_outcome == RecoveryLoopOutcome::Terminated {
            break;
        }
    }
    flush_declarations(&mut declaration_buffer, &mut declarations, &mut rules);
    Ok(Recovered {
        syntax: StyleContents {
            declarations: CssDeclarationList::new(declarations),
            rules,
        },
        diagnostics: body_parser.diagnostics,
    })
}

fn flush_declarations(
    buffer: &mut Vec<CssDeclaration>,
    leading: &mut Vec<CssDeclaration>,
    rules: &mut Vec<CssRule>,
) {
    if buffer.is_empty() {
        return;
    }
    if rules.is_empty() {
        leading.append(buffer);
    } else {
        rules.push(CssRule::NestedDeclarations(CssNestedDeclarationsRule::new(
            CssDeclarationList::new(std::mem::take(buffer)),
        )));
    }
}

struct NestedStyleRuleParser<'s> {
    source: &'s str,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
    qualified_resource_error: Option<ParseError<'s, Error>>,
}

enum StyleBlockItem {
    Declaration(Box<CssDeclaration>),
    NestedRules(Vec<CssRule>),
}

enum NestedStyleAtRulePrelude {
    Media(CssMediaQueryList),
    Supports(CssSupportsCondition),
    Container(CssContainerPrelude),
    Layer(Vec<CssLayerName>),
    Scope(CssScopePrelude),
}

impl NestedStyleAtRulePrelude {
    fn production(&self) -> &'static str {
        match self {
            Self::Media(_) => "baseline.rule.media",
            Self::Supports(_) => "baseline.rule.supports",
            Self::Container(_) => "baseline.rule.container",
            Self::Layer(_) => "baseline.rule.layer-block",
            Self::Scope(_) => "baseline.rule.scope",
        }
    }
}

impl<'i> AtRuleParser<'i> for NestedStyleRuleParser<'i> {
    type Prelude = NestedStyleAtRulePrelude;
    type AtRule = StyleBlockItem;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        match_ignore_ascii_case! { &name,
            "media" => {
                let query = parse_media_query_list(
                    self.source,
                    input,
                    &mut self.diagnostics,
                    &self.recovery,
                )?;
                if !input.is_exhausted() {
                    return Err(with_media_query_context(
                        invalid_syntax(
                            input.current_source_location(),
                            "unexpected token after media query list",
                        ),
                        None,
                    ));
                }
                Ok(NestedStyleAtRulePrelude::Media(query))
            },
            "supports" => {
                let condition = parse_supports_condition(
                    self.source,
                    input,
                    &mut self.diagnostics,
                    &self.recovery,
                ).map_err(with_supports_prelude_context)?;
                Ok(NestedStyleAtRulePrelude::Supports(condition))
            },
            "container" => {
                let prelude = parse_container_prelude(self.source, input, &self.recovery)
                    .map_err(with_container_prelude_context)?;
                if !input.is_exhausted() {
                    return Err(with_at_rule_prelude_context(
                        invalid_syntax(
                            input.current_source_location(),
                            "unexpected token after container condition",
                        ),
                        "container",
                        "baseline.rule.container",
                        "the end of the @container prelude",
                    ));
                }
                Ok(NestedStyleAtRulePrelude::Container(prelude))
            },
            "layer" => Ok(NestedStyleAtRulePrelude::Layer(
                parse_layer_prelude(input).map_err(|error| {
                    with_at_rule_prelude_context(
                        error,
                        "layer",
                        "baseline.rule.layer-block",
                        "a supported @layer prelude",
                    )
                })?,
            )),
            "scope" => Ok(NestedStyleAtRulePrelude::Scope(
                parse_scope_prelude(
                    self.source,
                    input,
                    &mut self.diagnostics,
                    &self.recovery,
                ).map_err(|error| {
                    with_at_rule_prelude_context(
                        error,
                        "scope",
                        "baseline.rule.scope",
                        "a supported @scope prelude",
                    )
                })?,
            )),
            "import" => Err(invalid_at_rule_placement(
                input.current_source_location(),
                "import",
                "the stylesheet top level",
            )),
            "custom-media" => Err(invalid_at_rule_placement(input.current_source_location(), "custom-media", "a rule list without a style-rule ancestor")),
            "font-feature-values" => Err(invalid_at_rule_placement(
                input.current_source_location(),
                "font-feature-values",
                "a rule list without a style-rule ancestor",
            )),
            "font-face" => Err(invalid_at_rule_placement(
                input.current_source_location(),
                "font-face",
                "a stylesheet or conditional group rule list",
            )),
            "keyframes" => Err(invalid_at_rule_placement(
                input.current_source_location(),
                "keyframes",
                "a stylesheet or conditional group rule list",
            )),
            "namespace" => Err(invalid_at_rule_placement(
                input.current_source_location(),
                "namespace",
                "the stylesheet top level",
            )),
            "counter-style" => Err(super::top_level_only_at_rule_placement(
                input.current_source_location(),
                "counter-style",
            )),
            "page" => Err(super::top_level_only_at_rule_placement(
                input.current_source_location(),
                "page",
            )),
            _ => Err(input.new_error(cssparser::BasicParseErrorKind::AtRuleInvalid(name))),
        }
    }

    fn rule_without_block(
        &mut self,
        _prelude: Self::Prelude,
        _start: &ParserState,
    ) -> std::result::Result<Self::AtRule, ()> {
        Err(())
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::AtRule, ParseError<'i, Self::Error>> {
        let mut depth = self
            .recovery
            .enter_rule_block(self.source, input, prelude.production())?;
        let position = crate::source::CssSourcePosition::from_cssparser(
            start.position(),
            start.source_location(),
        );
        let rule = match prelude {
            NestedStyleAtRulePrelude::Media(query) => {
                let recovered = parse_style_contents(self.source, input, self.recovery.clone())?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax.into_nested_rules();
                CssRule::Media(CssMediaRule::new(query, rules, position))
            }
            NestedStyleAtRulePrelude::Supports(condition) => {
                let recovered = parse_style_contents(self.source, input, self.recovery.clone())?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax.into_nested_rules();
                CssRule::Supports(CssSupportsRule::new(condition, rules, position))
            }
            NestedStyleAtRulePrelude::Container(prelude) => {
                let recovered = parse_style_contents(self.source, input, self.recovery.clone())?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax.into_nested_rules();
                CssRule::Container(CssContainerRule::new(
                    prelude.name,
                    prelude.condition,
                    rules,
                    position,
                ))
            }
            NestedStyleAtRulePrelude::Layer(names) => {
                if names.len() > 1 {
                    return Err(invalid_at_rule_block(
                        input,
                        "layer",
                        "baseline.rule.layer-block",
                        "at most one layer name before a block",
                    ));
                }
                let recovered = parse_style_contents(self.source, input, self.recovery.clone())?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax.into_nested_rules();
                CssRule::LayerBlock(CssLayerBlockRule::new(
                    names.into_iter().next(),
                    rules,
                    position,
                ))
            }
            NestedStyleAtRulePrelude::Scope(prelude) => {
                let recovered =
                    parse_scoped_rule_list(self.source, input, self.recovery.clone(), true)?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                CssRule::Scope(CssScopeRule::new(
                    prelude.root,
                    prelude.limit,
                    rules,
                    position,
                ))
            }
        };
        depth.retain();
        Ok(StyleBlockItem::NestedRules(vec![rule]))
    }
}

impl<'i> QualifiedRuleParser<'i> for NestedStyleRuleParser<'i> {
    type Prelude = Vec<CssStyleSelector>;
    type QualifiedRule = StyleBlockItem;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        let mut recovery =
            SelectorRecovery::new(self.source, &mut self.diagnostics, self.recovery.clone());
        let result = parse_nested_style_selector_list(input, &mut recovery);
        if let Err(error) = &result
            && is_nesting_limit_error(error)
        {
            self.qualified_resource_error = Some(error.clone());
        }
        result
    }

    fn parse_block<'t>(
        &mut self,
        selectors: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let result = (|| {
            let mut depth =
                self.recovery
                    .enter_rule_block(self.source, input, "baseline.rule.style")?;
            let recovered = parse_style_rule_block(
                self.source,
                CssStyleSelectorList::new(selectors),
                crate::source::CssSourcePosition::from_cssparser(
                    start.position(),
                    start.source_location(),
                ),
                input,
                self.recovery.clone(),
            )?;
            self.diagnostics.extend(recovered.diagnostics);
            depth.retain();
            Ok(StyleBlockItem::NestedRules(recovered.syntax))
        })();
        if let Err(error) = &result
            && is_nesting_limit_error(error)
        {
            self.qualified_resource_error = Some(error.clone());
        }
        result
    }
}

impl<'i> RuleBodyItemParser<'i, StyleBlockItem, Error> for NestedStyleRuleParser<'i> {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

impl<'i> DeclarationParser<'i> for NestedStyleRuleParser<'i> {
    type Declaration = StyleBlockItem;
    type Error = Error;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        declaration_start: &ParserState,
    ) -> std::result::Result<Self::Declaration, ParseError<'i, Self::Error>> {
        let mut declaration_parser =
            StrictDeclarationParser::new(self.source, self.recovery.clone(), false);
        declaration_parser
            .parse_value(name, input, declaration_start)
            .map(Box::new)
            .map(StyleBlockItem::Declaration)
    }
}
