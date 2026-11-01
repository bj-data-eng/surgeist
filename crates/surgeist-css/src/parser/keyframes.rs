use super::block_units::{
    DeclarationUnit, GrammarError, NativeError, drive_units, dropped_unit, grammar_error,
    location_at, offset_of, refused_declaration,
};
use super::declaration_unit::{DeclarationText, parse_declaration_unit};
use super::descriptor_stream::{Value, fail, from_component, from_here, parse_scope};
use super::syntax_types::{Attached, Stream, SyntaxError};
use cssparser::{AtRuleParser, ParseError, Parser, ParserState, QualifiedRuleParser};
use surgeist_css_parser::Token;
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
use crate::diagnostic::error::{invalid_at_rule_placement, invalid_syntax, unexpected_at};
use crate::diagnostic::{CssError, CssFeatureId};
use crate::numeric::expression::{CalculationRoot, NumericInputContext};
use crate::numeric::{CssCalculation, CssPercentage};
use crate::rule::keyframes::{
    CssKeyframeBlock, CssKeyframeDeclaration, CssKeyframeDeclarationList, CssKeyframePercent,
    CssKeyframeSelector, CssKeyframeSelectorList, CssKeyframesIdent, CssKeyframesName,
    CssKeyframesRule, CssKeyframesString,
};

pub(super) static IMPLEMENTED_RULES: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.rule.keyframes")];

/// The name of a `@keyframes` rule that the native parser `input` holds next.
pub(crate) fn parse_keyframes_name<'i, 't>(
    input: &mut Parser<'i, 't>,
    snapshot: &crate::source::CssSourceSnapshot,
) -> std::result::Result<CssKeyframesName, ParseError<'i, CssError>> {
    super::descriptor_stream::parse_scope(input, snapshot, |stream, numeric| {
        keyframes_name(stream, numeric)
    })
}

fn keyframes_name<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssKeyframesName> {
    let at = input.site();
    let location = super::descriptor_stream::location(numeric, at);
    if let Ok(name) = input.attempt(Stream::consume_ident) {
        return parse_custom_ident_from_str_at_source(name, location)
            .and_then(|name| {
                CssKeyframesIdent::try_new(name).ok_or_else(|| unexpected_at(location))
            })
            .map(CssKeyframesName::Ident)
            .map_err(|error| SyntaxError::custom(at, error));
    }

    let value = from_here(input, Stream::consume_string)?;
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
    let (declarations, mut recovered) =
        super::property_input::adapt_native(input, recovery.source_snapshot(), |contents| {
            Ok(drive_declarations(contents, source, &recovery))
        })?;
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
    let mut readmit = KeyframeDeclarationParser {
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
            // A declaration that step 8 refuses is read to its `;` all the
            // same, and the grammar's verdict on it is reported (until the
            // declaration paths follow step 8).
            let text = DeclarationText::read(start, end);
            let name = match start.token() {
                Some(surgeist_css_parser::Token::Ident(name)) => name,
                _ => "",
            };
            match readmit.admit(name, &text) {
                Ok(declaration) => {
                    declarations.push(declaration);
                    return Some(end);
                }
                Err(error) => dropped.error = error,
            }
        }
        // An at-rule is dropped for its prelude, so its text ends before the
        // `;` or `{` that ended it; every at-rule is invalid in a keyframe's
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
        parse_keyframe_selector_list(input, self.recovery.source_snapshot())
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

/// The selector list of a keyframe block that the native parser `input`
/// holds, to the end of its scope.
pub(super) fn parse_keyframe_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    snapshot: &crate::source::CssSourceSnapshot,
) -> std::result::Result<CssKeyframeSelectorList, ParseError<'i, CssError>> {
    parse_scope(input, snapshot, read_keyframe_selector_list)
}

fn read_keyframe_selector_list<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssKeyframeSelectorList> {
    let selectors = super::combinators::parse_native_comma_list(
        input,
        |input| parse_keyframe_selector(input, numeric),
        |_, at| fail(numeric, at, unexpected_at),
    )?;
    CssKeyframeSelectorList::try_new(selectors)
        .ok_or_else(|| fail(numeric, input.site(), unexpected_at))
}

fn parse_keyframe_selector<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssKeyframeSelector> {
    input.discard_whitespace();
    let location = input.site();
    let start = input.mark();
    match input.consume_after_whitespace()? {
        Token::Ident(ident) => match_ignore_ascii_case! { ident,
            "from" => Ok(CssKeyframeSelector::From),
            "to" => Ok(CssKeyframeSelector::To),
            _ => Err(fail(numeric, location, invalid_syntax)),
        },
        Token::Percentage(_) => {
            let text = input.text_since(location).unwrap_or_default();
            let value = checked_percentage_value(
                super::descriptor_stream::location(numeric, location),
                text,
            )
            .map_err(|error| SyntaxError::custom(location, error))?;
            CssKeyframePercent::try_new(value)
                .map(CssKeyframeSelector::Percent)
                .ok_or_else(|| fail(numeric, location, invalid_syntax))
        }
        Token::Number(_) => Err(fail(numeric, location, invalid_syntax)),
        Token::Function(name) if crate::numeric::expression::is_math_function(name) => {
            input.restore(start);
            let expression = from_component(numeric, input, |input| {
                let start = input.mark();
                parse_numeric_function(input, &start, numeric, CalculationRoot::Percentage)
            })?;
            CssKeyframePercent::from_parser_calculation(
                CssCalculation::<CssPercentage>::from_expression(expression),
            )
            .map(CssKeyframeSelector::Percent)
            .ok_or_else(|| fail(numeric, location, invalid_syntax))
        }
        token => Err(location.unexpected(token)),
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
