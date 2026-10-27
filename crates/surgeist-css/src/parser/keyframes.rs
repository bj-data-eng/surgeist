use super::property_input::NativeValueRange;
use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, Token,
};
use surgeist_css_parser::match_ignore_ascii_case;

use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::values::{
    checked_percentage_value, parse_custom_ident_from_str_at_source, parse_numeric_function,
};
use super::{
    DeclarationMode, consume_failed_rule_block, is_declaration_recovery_unit,
    parse_declaration_core, structural_recovery_production, structural_rule_diagnostic,
};
use crate::diagnostic::error::{basic, invalid_at_rule_placement, invalid_syntax, unexpected_at};
use crate::diagnostic::{CssError, CssFeatureId};
use crate::numeric::expression::CalculationRoot;
use crate::numeric::{CssCalculation, CssPercentage};
use crate::rule::keyframes::{
    CssKeyframeBlock, CssKeyframeDeclaration, CssKeyframeDeclarationList, CssKeyframePercent,
    CssKeyframeSelector, CssKeyframeSelectorList, CssKeyframesIdent, CssKeyframesName,
    CssKeyframesRule, CssKeyframesString,
};

pub(super) static IMPLEMENTED_RULES: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.rule.keyframes")];

pub(crate) fn parse_keyframes_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssKeyframesName, ParseError<'i, CssError>> {
    let location = input.current_source_location();
    if let Ok(name) = input.try_parse(Parser::expect_ident_cloned) {
        return parse_custom_ident_from_str_at_source(name.as_ref(), location)
            .and_then(|name| {
                CssKeyframesIdent::try_new(name).ok_or_else(|| unexpected_at(location))
            })
            .map(CssKeyframesName::Ident);
    }

    let value = input.expect_string_cloned().map_err(basic)?;
    Ok(CssKeyframesName::String(CssKeyframesString::new(
        value.to_string(),
    )))
}

pub(crate) fn parse_keyframes_rule<'i, 't>(
    source: &'i str,
    name: CssKeyframesName,
    input: &mut Parser<'i, 't>,
    start: &ParserState,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> std::result::Result<CssKeyframesRule, ParseError<'i, CssError>> {
    let blocks = parse_body(source, input, diagnostics, recovery)?;
    Ok(CssKeyframesRule::new(
        name,
        blocks,
        crate::source::CssSourcePosition::from_cssparser(start.position(), start.source_location()),
    ))
}

// Syntax 3 (2026 CRD) §5.5.2 consumes the @keyframes block with consume a
// block's contents; the <qualified-rule-list> grammar then admits only
// keyframe rules, so every other selected unit is one dropped keyframe block.
pub(super) fn parse_body<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<Vec<CssKeyframeBlock>, ParseError<'i, CssError>> {
    let mut parser = KeyframeBlockParser {
        source,
        diagnostics: Vec::new(),
        recovery: recovery.clone(),
    };
    let mut blocks = Vec::new();
    let (document, selected) = super::syntax_bridge::rules(
        source,
        input,
        &recovery,
        crate::syntax_consumption::RuleListSyntax::BlockContents,
    )
    .map_err(|error| {
        crate::diagnostic::error::invalid_component_value(input.current_source_location(), error)
    })?;
    for selected in &selected {
        let progress = RecoveryProgress::record(input);
        let (unit_start, item) = super::syntax_bridge::parse_selected(
            source,
            input,
            &mut parser,
            &recovery,
            &document,
            selected,
        );
        let failed_block_error = item.as_ref().err().and_then(|(_, failed_unit)| {
            consume_failed_rule_block(
                source,
                input,
                unit_start,
                &parser.recovery,
                structural_recovery_production(failed_unit),
            )
            .1
        });
        let retained = item.is_ok();
        let progress_outcome = progress.finish(input, retained);
        let unit_end = input.position().byte_index();
        match item {
            Ok(block) => blocks.push(block),
            Err((error, failed_unit)) => {
                let error = failed_block_error.unwrap_or(*error);
                if let Some(diagnostic) = structural_rule_diagnostic(
                    source,
                    error,
                    failed_unit,
                    unit_start,
                    unit_end,
                    crate::diagnostic::CssRecoveryAction::DropKeyframeBlock,
                ) {
                    parser.diagnostics.push(diagnostic);
                }
            }
        }
        if progress_outcome == RecoveryLoopOutcome::Terminated {
            break;
        }
    }
    super::syntax_bridge::discard_block_contents_tail(input);
    diagnostics.append(&mut parser.diagnostics);

    Ok(blocks)
}

pub(super) fn parse_declarations<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssKeyframeDeclarationList, ParseError<'i, CssError>> {
    let mut declarations = Vec::new();
    let mut declaration_parser = KeyframeDeclarationParser { source, recovery };
    let mut items = RuleBodyParser::new(input, &mut declaration_parser);
    loop {
        let progress = RecoveryProgress::record(items.input);
        let unit_start = super::rule_list::skip_block_contents_trivia(items.input);
        let Some(item) = items.next() else {
            break;
        };
        // A failed nested rule leaves its block pending; consume it as part
        // of the dropped unit so the recovery span covers the whole unit.
        let failed_block_error = item.as_ref().err().and_then(|_| {
            consume_failed_rule_block(
                source,
                items.input,
                unit_start,
                &items.parser.recovery,
                "baseline.rule.keyframes",
            )
            .1
        });
        let retained = item.is_ok();
        let progress_outcome = progress.finish(items.input, retained);
        let unit_end = items.input.position().byte_index();
        match item {
            Ok(declaration) => declarations.push(declaration),
            // Syntax 3 (2026 CRD) §7.1: qualified rules and at-rules are
            // invalid in a keyframe's `<declaration-list>` and are dropped one
            // by one; the keyframe block itself survives.
            Err((error, failed_unit)) => {
                let action = if is_declaration_recovery_unit(failed_unit) {
                    crate::diagnostic::CssRecoveryAction::DropDeclaration
                } else if failed_unit.trim_start().starts_with('@') {
                    crate::diagnostic::CssRecoveryAction::DropAtRule
                } else {
                    crate::diagnostic::CssRecoveryAction::DropQualifiedRule
                };
                if let Some(diagnostic) = structural_rule_diagnostic(
                    source,
                    failed_block_error.unwrap_or(error),
                    failed_unit,
                    unit_start,
                    unit_end,
                    action,
                ) {
                    diagnostics.push(diagnostic);
                }
            }
        }
        if progress_outcome == RecoveryLoopOutcome::Terminated {
            break;
        }
    }
    Ok(CssKeyframeDeclarationList::new(declarations))
}

pub(super) fn parse_detached_rule<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    recovery: RecoveryState,
) -> Result<
    (
        CssKeyframeBlock,
        Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    ),
    ParseError<'i, CssError>,
> {
    let mut parser = KeyframeBlockParser {
        source,
        diagnostics: Vec::new(),
        recovery: recovery.clone(),
    };
    let block = super::syntax_bridge::one(source, input, &mut parser, &recovery)?;
    Ok((block, parser.diagnostics))
}

struct KeyframeBlockParser<'s> {
    source: &'s str,
    diagnostics: Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
}

impl<'i> AtRuleParser<'i> for KeyframeBlockParser<'i> {
    type Prelude = ();
    type AtRule = CssKeyframeBlock;
    type Error = CssError;
}

impl<'i> QualifiedRuleParser<'i> for KeyframeBlockParser<'i> {
    type Prelude = CssKeyframeSelectorList;
    type QualifiedRule = CssKeyframeBlock;
    type Error = CssError;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        parse_keyframe_selector_list(
            input,
            &crate::numeric::expression::NumericInputContext::parsed(
                self.recovery.source_snapshot(),
            ),
        )
    }

    fn parse_block<'t>(
        &mut self,
        selectors: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let mut depth =
            self.recovery
                .enter_rule_block(self.source, input, "baseline.keyframes.block")?;
        let declarations = parse_declarations(
            self.source,
            input,
            &mut self.diagnostics,
            self.recovery.clone(),
        )?;

        let result = CssKeyframeBlock::new(
            selectors,
            declarations,
            crate::source::CssSourcePosition::from_cssparser(
                start.position(),
                start.source_location(),
            ),
        );
        depth.retain();
        Ok(result)
    }
}

pub(super) fn parse_keyframe_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssKeyframeSelectorList, ParseError<'i, CssError>> {
    let selectors = super::combinators::parse_native_comma_list(
        input,
        |input| parse_keyframe_selector(input, numeric),
        |_, at| unexpected_at(at),
    )?;
    CssKeyframeSelectorList::try_new(selectors)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_keyframe_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssKeyframeSelector, ParseError<'i, CssError>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let start = input.state();
    let token_start = input.position();
    match input.next().map_err(basic)? {
        Token::Ident(ident) => match_ignore_ascii_case! { ident,
            "from" => Ok(CssKeyframeSelector::From),
            "to" => Ok(CssKeyframeSelector::To),
            _ => Err(invalid_syntax(location)),
        },
        Token::Percentage { .. } => {
            let value = checked_percentage_value(location, input.slice_from(token_start))?;
            CssKeyframePercent::try_new(value)
                .map(CssKeyframeSelector::Percent)
                .ok_or_else(|| invalid_syntax(location))
        }
        Token::Number { .. } => Err(invalid_syntax(location)),
        Token::Function(name) if crate::numeric::expression::is_math_function(name) => {
            input.reset(&start);
            let expression = super::property_input::adapt_native(
                input,
                numeric,
                NativeValueRange::Component,
                |input, numeric| {
                    let start = input.state();
                    parse_numeric_function(input, &start, numeric, CalculationRoot::Percentage)
                },
            )?;
            CssKeyframePercent::from_parser_calculation(
                CssCalculation::<CssPercentage>::from_expression(expression),
            )
            .map(CssKeyframeSelector::Percent)
            .ok_or_else(|| invalid_syntax(location))
        }
        token => Err(location.new_unexpected_token_error::<CssError>(token.clone())),
    }
}

struct KeyframeDeclarationParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
}

impl<'i> AtRuleParser<'i> for KeyframeDeclarationParser<'i> {
    type Prelude = ();
    type AtRule = CssKeyframeDeclaration;
    type Error = CssError;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> std::result::Result<Self::Prelude, ParseError<'i, Self::Error>> {
        Err(invalid_at_rule_placement(
            input.current_source_location(),
            name.as_ref(),
            "a keyframe declaration list",
        ))
    }
}

impl<'i> QualifiedRuleParser<'i> for KeyframeDeclarationParser<'i> {
    type Prelude = ();
    type QualifiedRule = CssKeyframeDeclaration;
    type Error = CssError;
}

impl<'i> RuleBodyItemParser<'i, CssKeyframeDeclaration, CssError>
    for KeyframeDeclarationParser<'i>
{
    fn parse_declarations(&self) -> bool {
        true
    }

    // Syntax 3 (2026 CRD) §5.5.5 reconsumes a failed declaration, and any
    // other non-at-rule unit, as a nested qualified rule with a `;` stop token.
    fn parse_qualified(&self) -> bool {
        true
    }
}

impl<'i> DeclarationParser<'i> for KeyframeDeclarationParser<'i> {
    type Declaration = CssKeyframeDeclaration;
    type Error = CssError;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        declaration_start: &ParserState,
    ) -> std::result::Result<Self::Declaration, ParseError<'i, Self::Error>> {
        let implicit_closures =
            self.recovery
                .check_component_values(self.source, input, "css.declaration")?;
        let parsed = parse_declaration_core(
            DeclarationMode::Keyframe,
            name,
            input,
            declaration_start,
            self.recovery.source_snapshot(),
            self.recovery.parser_context(),
        )?;
        self.recovery.retain_component_closures(implicit_closures);
        self.recovery.retain_navigation_diagnostic(&parsed.body);
        Ok(parsed.into_keyframe_declaration())
    }
}
