use super::block_units::{
    DeclarationUnit, GrammarError, NativeError, drive_units, dropped_unit, grammar_error,
    location_at, offset_of, refused_declaration,
};
use super::declaration_unit::{DeclarationText, parse_declaration_unit};
use super::property_input::NativeValueRange;
use super::syntax_types::{Attached, Stream, SyntaxError};
use cssparser::{AtRuleParser, ParseError, Parser, ParserState, QualifiedRuleParser, Token};
use surgeist_css_parser::grammar::{AtRule, BlockGrammar, QualifiedRule, RuleGrammar, UnitKind};
use surgeist_css_parser::match_ignore_ascii_case;

use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::values::{
    checked_percentage_value, parse_custom_ident_from_str_at_source, parse_numeric_function,
};
use super::{
    DeclarationMode, consume_failed_rule_block, is_declaration_recovery_unit,
    structural_recovery_production, structural_rule_diagnostic,
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

/// The declarations of a keyframe's block, from the native callback that was
/// handed the block: the contents of the block are the scope.
pub(super) fn parse_declarations<'i>(
    source: &'i str,
    input: &mut Parser<'i, '_>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Result<CssKeyframeDeclarationList, ParseError<'i, CssError>> {
    let numeric =
        crate::numeric::expression::NumericInputContext::parsed(recovery.source_snapshot());
    let (declarations, mut recovered) = super::property_input::adapt_native(
        input,
        &numeric,
        NativeValueRange::Scope,
        |contents, _| Ok(drive_declarations(contents, source, &recovery)),
    )?;
    diagnostics.append(&mut recovered);
    Ok(declarations)
}

/// Consumes the declarations of a keyframe's block, with the recovery of
/// Syntax 3 (2026 CRD) §7.1: qualified rules and at-rules are invalid in a
/// keyframe's `<declaration-list>` and are dropped one by one; the keyframe
/// block itself survives.
pub(super) fn drive_declarations<'i>(
    contents: &mut Stream<'i>,
    source: &str,
    recovery: &RecoveryState,
) -> (
    CssKeyframeDeclarationList,
    Vec<crate::diagnostic::CssRecoveryDiagnostic>,
) {
    let snapshot = recovery.source_snapshot();
    let mut grammar = KeyframeDeclarationParser {
        source,
        recovery: recovery.clone(),
    };
    let mut declarations = Vec::new();
    let mut diagnostics = Vec::new();
    let mut legacy = KeyframeDeclarationParser {
        source,
        recovery: recovery.clone(),
    };
    drive_units(contents, &mut grammar, |unit| {
        let failed = match unit {
            Ok(declaration) => {
                declarations.push(declaration);
                return None;
            }
            Err(failed) => failed,
        };
        let mut dropped = dropped_unit(snapshot, source, &failed);
        if let Some((start, end)) = refused_declaration(&failed) {
            // The native block parser read the value of a declaration that
            // step 8 refuses, to its `;`, and reported its grammar's verdict.
            let text = DeclarationText::read(start, end);
            let name = match start.token() {
                Some(surgeist_css_parser::Token::Ident(name)) => name,
                _ => "",
            };
            match legacy.admit(name, &text) {
                Ok(declaration) => {
                    declarations.push(declaration);
                    return Some(end);
                }
                Err(error) => dropped.error = error,
            }
        }
        // cssparser slices an at-rule whose prelude it rejected before the `;`
        // or `{` that ended it, and every at-rule is rejected in a keyframe's
        // block.
        let text = if dropped.kind == UnitKind::AtRule {
            dropped.text_to_prelude_end()
        } else {
            dropped.text()
        };
        // A failed rule's block is checked for the resources a grammar would
        // have been charged for entering it.
        let error = dropped
            .block_contents
            .and_then(|contents| {
                recovery.check_failed_rule_block_at(source, contents, "baseline.rule.keyframes")
            })
            .unwrap_or(dropped.error);
        let action = if is_declaration_recovery_unit(text) {
            crate::diagnostic::CssRecoveryAction::DropDeclaration
        } else if text.trim_start().starts_with('@') {
            crate::diagnostic::CssRecoveryAction::DropAtRule
        } else {
            crate::diagnostic::CssRecoveryAction::DropQualifiedRule
        };
        if let Some(diagnostic) =
            structural_rule_diagnostic(source, error, text, dropped.start, dropped.end, action)
        {
            diagnostics.push(diagnostic);
        }
        None
    });
    (CssKeyframeDeclarationList::new(declarations), diagnostics)
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
                    let start = input.mark();
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

/// The grammar of the declarations of a keyframe's block: declarations only,
/// admitted as the declarations of every other block are, with the keyframe
/// restrictions of the declaration mode. An at-rule or a qualified rule is not
/// valid (Syntax 3 (2026 CRD) §5.5.5 reads a unit that is not a declaration
/// as a qualified rule with a `;` stop token, and §7.1 admits no rule here).
struct KeyframeDeclarationParser<'s> {
    source: &'s str,
    recovery: RecoveryState,
}

impl<'i> RuleGrammar<'i, Attached<'i>> for KeyframeDeclarationParser<'_> {
    type Item = CssKeyframeDeclaration;
    type Error = NativeError<'i>;

    fn at_rule(&mut self, rule: AtRule<'i, Attached<'i>>) -> Result<Self::Item, GrammarError<'i>> {
        let snapshot = self.recovery.source_snapshot();
        let location = location_at(snapshot, offset_of(rule.prelude().start_site()));
        Err(grammar_error(
            rule.start(),
            invalid_at_rule_placement(location, rule.name(), "a keyframe declaration list"),
        ))
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i, Attached<'i>>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        Err(SyntaxError::invalid_qualified_rule_error(rule.start()))
    }
}

impl<'i> BlockGrammar<'i, Attached<'i>> for KeyframeDeclarationParser<'_> {
    fn declaration(
        &mut self,
        declaration: DeclarationUnit<'i>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        let at = declaration.start();
        self.admit(declaration.name(), &DeclarationText::of(&declaration))
            .map_err(|error| grammar_error(at, error))
    }
}

impl KeyframeDeclarationParser<'_> {
    /// Admits the declaration named `name` that lies at `text`.
    fn admit<'i>(
        &mut self,
        name: &str,
        text: &DeclarationText,
    ) -> Result<CssKeyframeDeclaration, NativeError<'i>> {
        let implicit_closures = self.recovery.check_component_values_from(
            self.source,
            text.value_start,
            "css.declaration",
        )?;
        let parsed = parse_declaration_unit(
            DeclarationMode::Keyframe,
            name,
            text,
            self.recovery.source_snapshot(),
            self.recovery.parser_context(),
        )?;
        self.recovery.retain_component_closures(implicit_closures);
        self.recovery.retain_navigation_diagnostic(&parsed.body);
        Ok(parsed.into_keyframe_declaration())
    }
}
