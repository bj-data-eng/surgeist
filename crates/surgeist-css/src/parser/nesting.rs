use cssparser::{
    AtRuleParser, BasicParseErrorKind, CowRcStr, DeclarationParser, ParseError, Parser,
    ParserState, QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, ToCss, Token,
    match_ignore_ascii_case,
};

use super::queries::parse_media_query_list;
use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::selectors::{
    SelectorRecovery, consume_selector_whitespace, parse_complex_selector_part,
    parse_compound_selector_model, parse_rule_selector,
};
use super::supports::{parse_supports_condition, with_supports_prelude_context};
use super::{
    CssContainerPrelude, CssScopePrelude, Recovered, StrictDeclarationParser,
    block_item_diagnostic, consume_failed_rule_block, is_declaration_recovery_unit,
    parse_container_prelude, parse_layer_prelude, parse_scope_prelude, parse_scoped_rule_list,
    structural_recovery_action, structural_recovery_production, structural_rule_diagnostic,
};
use crate::error::{
    CssFeatureId, Error, invalid_at_rule_block, invalid_at_rule_placement, invalid_selector,
    invalid_syntax, selector_basic, with_at_rule_prelude_context, with_media_query_context,
};
use crate::syntax::*;

pub(super) static IMPLEMENTED_SELECTORS: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.selector.nesting")];

pub(super) fn parse_style_rule_block<'i, 't>(
    source: &'i str,
    parent_selectors: Vec<CssSelector>,
    position: crate::CssSourcePosition,
    input: &mut Parser<'i, 't>,
    recovery: RecoveryState,
) -> std::result::Result<Recovered<Vec<CssRule>>, ParseError<'i, Error>> {
    let content_start = input.position().byte_index();
    let suppress_preflight_placeholder =
        recovery.record_style_context(content_start, &parent_selectors, position);
    let mut body_parser = NestedStyleRuleParser {
        source,
        parent_selectors,
        diagnostics: Vec::new(),
        recovery,
    };
    let parent_selectors = body_parser.parent_selectors.clone();
    let mut rules = Vec::new();
    let mut declaration_buffer = Vec::new();
    let mut previous_end = input.position().byte_index();

    let mut items = RuleBodyParser::new(input, &mut body_parser);
    loop {
        let progress = RecoveryProgress::record(items.input);
        let Some(item) = items.next() else {
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
        let retained = item.is_ok();
        let progress_outcome = progress.finish(items.input, retained);
        let unit_end = items.input.position().byte_index();
        match item {
            Ok(StyleBlockItem::Declaration(declaration)) => {
                declaration_buffer.push(*declaration);
            }
            Ok(StyleBlockItem::NestedRules(nested_rules)) => {
                flush_declarations(
                    &parent_selectors,
                    position,
                    &mut declaration_buffer,
                    &mut rules,
                );
                rules.extend(nested_rules);
            }
            Err((error, failed_unit))
                if is_declaration_recovery_unit(failed_unit) && !failed_at_block =>
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
                let error = failed_block_error.unwrap_or(error);
                flush_declarations(
                    &parent_selectors,
                    position,
                    &mut declaration_buffer,
                    &mut rules,
                );
                let action = structural_recovery_action(failed_unit);
                if let Some(diagnostic) = structural_rule_diagnostic(
                    source,
                    error,
                    failed_unit,
                    previous_end,
                    unit_end,
                    action,
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

    flush_declarations(
        &parent_selectors,
        position,
        &mut declaration_buffer,
        &mut rules,
    );
    if rules.is_empty() && !suppress_preflight_placeholder {
        for selector in &parent_selectors {
            rules.push(CssRule::Style(CssStyleRule::new(
                selector.clone(),
                CssDeclarationList::new(Vec::new()),
                position,
            )));
        }
    }

    Ok(Recovered {
        syntax: rules,
        diagnostics: body_parser.diagnostics,
    })
}

fn flush_declarations(
    parent_selectors: &[CssSelector],
    position: crate::CssSourcePosition,
    declaration_buffer: &mut Vec<CssDeclaration>,
    rules: &mut Vec<CssRule>,
) {
    if declaration_buffer.is_empty() {
        return;
    }

    for selector in parent_selectors {
        rules.push(CssRule::Style(CssStyleRule::new(
            selector.clone(),
            CssDeclarationList::new(declaration_buffer.clone()),
            position,
        )));
    }
    declaration_buffer.clear();
}

struct NestedStyleRuleParser<'s> {
    source: &'s str,
    parent_selectors: Vec<CssSelector>,
    diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
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
                let prelude = parse_container_prelude(input).map_err(|error| {
                    with_at_rule_prelude_context(
                        error,
                        "container",
                        "baseline.rule.container",
                        "a supported @container prelude",
                    )
                })?;
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
                let recovered = parse_style_rule_block(
                    self.source,
                    self.parent_selectors.clone(),
                    position,
                    input,
                    self.recovery.clone(),
                )?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                CssRule::Media(CssMediaRule::new(query, rules, position))
            }
            NestedStyleAtRulePrelude::Supports(condition) => {
                let recovered = parse_style_rule_block(
                    self.source,
                    self.parent_selectors.clone(),
                    position,
                    input,
                    self.recovery.clone(),
                )?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                CssRule::Supports(CssSupportsRule::new(condition, rules, position))
            }
            NestedStyleAtRulePrelude::Container(prelude) => {
                let recovered = parse_style_rule_block(
                    self.source,
                    self.parent_selectors.clone(),
                    position,
                    input,
                    self.recovery.clone(),
                )?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
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
                let recovered = parse_style_rule_block(
                    self.source,
                    self.parent_selectors.clone(),
                    position,
                    input,
                    self.recovery.clone(),
                )?;
                self.diagnostics.extend(recovered.diagnostics);
                let rules = recovered.syntax;
                CssRule::LayerBlock(CssLayerBlockRule::new(
                    names.into_iter().next(),
                    rules,
                    position,
                ))
            }
            NestedStyleAtRulePrelude::Scope(prelude) => {
                let recovered = parse_scoped_rule_list(self.source, input, self.recovery.clone())?;
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
    type Prelude = Vec<NestedSelector>;
    type QualifiedRule = StyleBlockItem;
    type Error = Error;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        let mut recovery =
            SelectorRecovery::new(self.source, &mut self.diagnostics, self.recovery.clone());
        parse_nested_selector_list(input, &mut recovery)
    }

    fn parse_block<'t>(
        &mut self,
        nested_selectors: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let mut depth =
            self.recovery
                .enter_rule_block(self.source, input, "baseline.rule.style")?;
        let mut flattened_selectors = Vec::new();
        for parent_selector in &self.parent_selectors {
            for nested_selector in &nested_selectors {
                flattened_selectors.push(nested_selector.flatten(parent_selector.clone(), input)?);
            }
        }

        let recovered = parse_style_rule_block(
            self.source,
            flattened_selectors,
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

#[derive(Clone, Debug)]
enum NestedSelector {
    Descendant(CssSelector),
    Relative(Vec<CssComplexSelectorPart>),
    Parent,
    Append(CssCompoundSelector),
}

impl NestedSelector {
    fn flatten<'i, 't>(
        &self,
        parent: CssSelector,
        input: &Parser<'i, 't>,
    ) -> std::result::Result<CssSelector, ParseError<'i, Error>> {
        match self {
            Self::Descendant(child) => CssSelector::combine_descendant(parent, child.clone())
                .ok_or_else(|| invalid_selector(input, "invalid nested descendant selector")),
            Self::Relative(parts) => {
                let mut parts = parts.iter();
                let Some(first) = parts.next() else {
                    return Ok(parent);
                };
                let mut combined = CssSelector::combine_with_combinator(
                    parent,
                    first.combinator(),
                    first.selector().clone(),
                )
                .ok_or_else(|| invalid_selector(input, "invalid nested relative selector"))?;
                for part in parts {
                    combined = CssSelector::combine_with_combinator(
                        combined,
                        part.combinator(),
                        part.selector().clone(),
                    )
                    .ok_or_else(|| invalid_selector(input, "invalid nested relative selector"))?;
                }
                Ok(combined)
            }
            Self::Parent => Ok(parent),
            Self::Append(suffix) => CssSelector::append_to_subject(parent, suffix.clone())
                .ok_or_else(|| invalid_selector(input, "invalid nested selector suffix")),
        }
    }
}

fn parse_nested_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<Vec<NestedSelector>, ParseError<'i, Error>> {
    recovery.check_depth(input)?;
    let mut selectors = Vec::new();
    loop {
        selectors.push(parse_nested_selector(input, recovery)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
    }
    input.expect_exhausted().map_err(selector_basic)?;
    Ok(selectors)
}

fn parse_nested_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<NestedSelector, ParseError<'i, Error>> {
    consume_selector_whitespace(input)?;
    let state = input.state();
    match input.next_including_whitespace() {
        Ok(Token::Delim('&')) => parse_ampersand_nested_selector(input, recovery),
        Ok(Token::Delim('>')) => {
            parse_relative_selector(input, CssSelectorCombinator::Child, recovery)
        }
        Ok(Token::Delim('+')) => {
            parse_relative_selector(input, CssSelectorCombinator::NextSibling, recovery)
        }
        Ok(Token::Delim('~')) => {
            parse_relative_selector(input, CssSelectorCombinator::SubsequentSibling, recovery)
        }
        Ok(Token::Delim('|')) => Err(invalid_selector(
            input,
            "unsupported selector combinator `||`",
        )),
        Ok(_) => {
            input.reset(&state);
            parse_rule_selector(input, recovery).map(NestedSelector::Descendant)
        }
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            input.reset(&state);
            Err(invalid_selector(input, "nested selector is empty"))
        }
        Err(error) => Err(selector_basic(error)),
    }
}

fn parse_ampersand_nested_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<NestedSelector, ParseError<'i, Error>> {
    let had_whitespace = consume_selector_whitespace(input)?;
    let state = input.state();
    match input.next_including_whitespace() {
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            input.reset(&state);
            Ok(NestedSelector::Parent)
        }
        Err(error) => Err(selector_basic(error)),
        Ok(Token::Comma) => {
            input.reset(&state);
            Ok(NestedSelector::Parent)
        }
        Ok(Token::Delim('&')) => Err(invalid_selector(
            input,
            "nesting selector `&` is only supported once at the start",
        )),
        Ok(Token::Delim('>')) => {
            parse_relative_selector(input, CssSelectorCombinator::Child, recovery)
        }
        Ok(Token::Delim('+')) => {
            parse_relative_selector(input, CssSelectorCombinator::NextSibling, recovery)
        }
        Ok(Token::Delim('~')) => {
            parse_relative_selector(input, CssSelectorCombinator::SubsequentSibling, recovery)
        }
        Ok(Token::Delim('|')) => Err(invalid_selector(
            input,
            "unsupported selector combinator `||`",
        )),
        Ok(_) if had_whitespace => {
            input.reset(&state);
            parse_relative_selector(input, CssSelectorCombinator::Descendant, recovery)
        }
        Ok(_) => {
            input.reset(&state);
            let suffix = parse_compound_selector_model(input, recovery)?;
            ensure_nested_selector_boundary(input)?;
            Ok(NestedSelector::Append(suffix))
        }
    }
}

fn parse_relative_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    first_combinator: CssSelectorCombinator,
    recovery: &mut SelectorRecovery<'_>,
) -> std::result::Result<NestedSelector, ParseError<'i, Error>> {
    let mut parts = vec![parse_complex_selector_part(
        input,
        first_combinator,
        recovery,
    )?];
    loop {
        let had_whitespace = consume_selector_whitespace(input)?;
        let state = input.state();
        match input.next_including_whitespace() {
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
                input.reset(&state);
                break;
            }
            Err(error) => return Err(selector_basic(error)),
            Ok(Token::Comma) => {
                input.reset(&state);
                break;
            }
            Ok(Token::Delim('>')) => parts.push(parse_complex_selector_part(
                input,
                CssSelectorCombinator::Child,
                recovery,
            )?),
            Ok(Token::Delim('+')) => parts.push(parse_complex_selector_part(
                input,
                CssSelectorCombinator::NextSibling,
                recovery,
            )?),
            Ok(Token::Delim('~')) => parts.push(parse_complex_selector_part(
                input,
                CssSelectorCombinator::SubsequentSibling,
                recovery,
            )?),
            Ok(Token::Delim('|')) => {
                return Err(invalid_selector(
                    input,
                    "unsupported selector combinator `||`",
                ));
            }
            Ok(Token::Delim('&')) => {
                return Err(invalid_selector(
                    input,
                    "nesting selector `&` is only supported once at the start",
                ));
            }
            Ok(_) if had_whitespace => {
                input.reset(&state);
                let selector = parse_compound_selector_model(input, recovery)?;
                parts.push(CssComplexSelectorPart::new(
                    CssSelectorCombinator::Descendant,
                    selector,
                ));
            }
            Ok(token) => {
                let message = format!("unexpected selector token `{}`", token.to_css_string());
                input.reset(&state);
                return Err(invalid_selector(input, message));
            }
        }
    }
    Ok(NestedSelector::Relative(parts))
}

fn ensure_nested_selector_boundary<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    consume_selector_whitespace(input)?;
    let state = input.state();
    match input.next_including_whitespace() {
        Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => {
            input.reset(&state);
            Ok(())
        }
        Err(error) => Err(selector_basic(error)),
        Ok(Token::Comma) => {
            input.reset(&state);
            Ok(())
        }
        Ok(Token::Delim('&')) => Err(invalid_selector(
            input,
            "nesting selector `&` is only supported once at the start",
        )),
        Ok(token) => {
            let message = format!("unexpected selector token `{}`", token.to_css_string());
            input.reset(&state);
            Err(invalid_selector(input, message))
        }
    }
}
