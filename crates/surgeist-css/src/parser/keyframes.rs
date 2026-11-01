use super::block_units::{
    DeclarationUnit, GrammarError, NativeError, drive_units, dropped_unit, grammar_error,
    location_at, offset_of, refused_declaration,
};
use super::declaration_unit::{DeclarationText, parse_declaration_unit};
use super::descriptor_stream::{Value, fail, from_component, from_here, parse_scope, read_scope};
use super::rule_list::{
    RuleBlock, RuleListGrammar, RuleListKind, RuleListSink, UnitState, recover_rule_list,
};
use super::syntax_types::{Attached, Site, Stream, SyntaxError};
use cssparser::{ParseError, Parser};
use surgeist_css_parser::Offset;
use surgeist_css_parser::Token;
use surgeist_css_parser::grammar::{
    AtRule, BlockGrammar, QualifiedRule, RuleEnd, RuleGrammar, UnitKind,
};
use surgeist_css_parser::match_ignore_ascii_case;

use super::recovery::RecoveryState;
use super::values::{
    checked_percentage_value, parse_custom_ident_from_str_at_source, parse_numeric_function,
};
use super::{DeclarationMode, is_declaration_recovery_unit, structural_rule_diagnostic};
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

/// The name of a `@keyframes` rule that the stream `input` holds next.
pub(crate) fn parse_keyframes_name<'i>(
    input: &mut Stream<'i>,
    snapshot: &crate::source::CssSourceSnapshot,
) -> std::result::Result<CssKeyframesName, ParseError<'i, CssError>> {
    read_scope(input, snapshot, keyframes_name)
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

pub(crate) fn parse_keyframes_rule<'i>(
    source: &str,
    name: CssKeyframesName,
    contents: &mut Stream<'i>,
    position: crate::source::CssSourcePosition,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> CssKeyframesRule {
    let blocks = parse_body(source, contents, diagnostics, recovery);
    CssKeyframesRule::new(name, blocks, position)
}

/// The keyframe blocks of the block `contents` holds. Syntax 3 (2026 CRD)
/// §5.5.2 consumes the @keyframes block with consume a block's contents; the
/// <qualified-rule-list> grammar then admits only keyframe rules, so every
/// other unit is one dropped keyframe block.
pub(super) fn parse_body<'i>(
    source: &str,
    contents: &mut Stream<'i>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> Vec<CssKeyframeBlock> {
    let mut grammar = KeyframeBlockParser::new(source, recovery);
    let mut sink = BlockSink(Vec::new());
    recover_rule_list(
        source,
        contents.clone(),
        RuleListKind::Rules,
        &mut grammar,
        &mut sink,
    );
    super::descriptor_stream::consume_remaining(contents);
    diagnostics.append(&mut grammar.diagnostics);
    sink.0
}

/// The keyframe blocks of a list of them.
struct BlockSink(Vec<CssKeyframeBlock>);

impl RuleListSink<CssKeyframeBlock> for BlockSink {
    fn retain(
        &mut self,
        item: CssKeyframeBlock,
        _: &str,
        _: usize,
        _: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    ) {
        self.0.push(item);
    }
}

/// The declarations of a keyframe's block `contents`.
pub(super) fn parse_declarations<'i>(
    source: &str,
    contents: &mut Stream<'i>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
) -> CssKeyframeDeclarationList {
    let (declarations, mut recovered) = drive_declarations(contents, source, &recovery);
    diagnostics.append(&mut recovered);
    declarations
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

/// The grammar of the one keyframe block a detached rule is.
pub(super) fn detached_grammar(source: &str, recovery: RecoveryState) -> KeyframeBlockParser<'_> {
    KeyframeBlockParser::new(source, recovery)
}

/// The grammar of a list of keyframe blocks: qualified rules whose preludes are
/// keyframe selectors and whose blocks hold declarations.
pub(super) struct KeyframeBlockParser<'s> {
    source: &'s str,
    pub(super) diagnostics: Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: RecoveryState,
    unit: UnitState,
}

impl<'s> KeyframeBlockParser<'s> {
    fn new(source: &'s str, recovery: RecoveryState) -> Self {
        Self {
            source,
            diagnostics: Vec::new(),
            recovery,
            unit: UnitState::new(),
        }
    }

    /// The block of a keyframe whose selectors were read.
    fn block<'i>(
        &mut self,
        selectors: CssKeyframeSelectorList,
        start: Site<'i>,
        opening: usize,
        block: RuleBlock<'i>,
    ) -> Result<CssKeyframeBlock, NativeError<'i>> {
        let mut depth = self.recovery.enter_rule_block(
            self.source,
            Offset::new(opening),
            "baseline.keyframes.block",
        )?;
        let RuleBlock::Open { mut contents, .. } = block else {
            return Err(crate::diagnostic::error::nesting_limit(
                self.source,
                opening,
                super::recovery::STRUCTURAL_NESTING_LIMIT,
                "baseline.keyframes.block",
            ));
        };
        let declarations = parse_declarations(
            self.source,
            &mut contents,
            &mut self.diagnostics,
            self.recovery.clone(),
        );
        let result = CssKeyframeBlock::new(
            selectors,
            declarations,
            self.recovery.source_position(offset_of(start)),
        );
        depth.retain();
        Ok(result)
    }
}

impl<'i> RuleListGrammar<'i> for KeyframeBlockParser<'_> {
    fn unit(&mut self) -> &mut UnitState {
        &mut self.unit
    }

    fn recovery(&self) -> &RecoveryState {
        &self.recovery
    }

    fn diagnostics(&mut self) -> &mut Vec<crate::diagnostic::CssRecoveryDiagnostic> {
        &mut self.diagnostics
    }

    // No at-rule is a keyframe: the name is the failure of its prelude.
    fn admit_at_rule(
        &mut self,
        name: &'i str,
        start: Site<'i>,
        _: RuleEnd,
        _: Site<'i>,
        prelude: &mut Stream<'i>,
        block: RuleBlock<'i>,
    ) -> Result<CssKeyframeBlock, NativeError<'i>> {
        self.unit.start_index = Some(start.index());
        let error = ParseError {
            kind: cssparser::ParseErrorKind::Basic(cssparser::BasicParseErrorKind::AtRuleInvalid(
                cssparser::CowRcStr::from(name),
            )),
            location: location_at(self.recovery.source_snapshot(), offset_of(prelude.site())),
        };
        if let Some(opening) = block_opening_of(&block) {
            self.unit.prelude_failed = true;
            self.unit.block_error =
                self.recovery
                    .check_failed_rule_block_at(self.source, opening + 1, "css.at-rule");
        }
        Err(error)
    }

    fn admit_qualified(
        &mut self,
        start: Site<'i>,
        prelude: &mut Stream<'i>,
        block: RuleBlock<'i>,
    ) -> Result<CssKeyframeBlock, NativeError<'i>> {
        self.unit.start_index = Some(start.index());
        let Some(opening) = block_opening_of(&block) else {
            return Err(crate::diagnostic::error::unexpected_syntax_end(
                location_at(self.recovery.source_snapshot(), offset_of(start)),
            ));
        };
        let selectors =
            match parse_keyframe_selector_list_stream(prelude, self.recovery.source_snapshot()) {
                Ok(selectors) => selectors,
                Err(error) => {
                    self.unit.prelude_failed = true;
                    self.unit.block_error = self.recovery.check_failed_rule_block_at(
                        self.source,
                        opening + 1,
                        "css.qualified-rule",
                    );
                    return Err(error);
                }
            };
        self.block(selectors, start, opening, block)
    }

    fn admit_declaration(
        &mut self,
        _: &str,
        _: &DeclarationText,
    ) -> Result<CssKeyframeBlock, NativeError<'i>> {
        unreachable!("a list of keyframe blocks is read as rules")
    }

    fn drop_action(&self, _: &str) -> crate::diagnostic::CssRecoveryAction {
        crate::diagnostic::CssRecoveryAction::DropKeyframeBlock
    }
}

/// Where the block of a rule opens, when it has one.
fn block_opening_of(block: &RuleBlock<'_>) -> Option<usize> {
    match block {
        RuleBlock::Absent => None,
        RuleBlock::Open { opening, .. } | RuleBlock::Denied { opening } => Some(*opening),
    }
}

impl<'i> RuleGrammar<'i, Attached<'i>> for KeyframeBlockParser<'_> {
    type Item = CssKeyframeBlock;
    type Error = NativeError<'i>;

    fn at_rule(&mut self, rule: AtRule<'i, Attached<'i>>) -> Result<Self::Item, GrammarError<'i>> {
        let at = rule.start();
        let block = super::rule_list::rule_block_of(rule.block(), rule.end());
        let mut prelude = rule.prelude();
        self.admit_at_rule(
            rule.name(),
            at,
            rule.rule_end(),
            rule.end(),
            &mut prelude,
            block,
        )
        .map_err(|error| grammar_error(at, error))
    }

    fn qualified_rule(
        &mut self,
        rule: QualifiedRule<'i, Attached<'i>>,
    ) -> Result<Self::Item, GrammarError<'i>> {
        let at = rule.start();
        let block = super::rule_list::rule_block_of(Some(rule.block()), rule.end());
        let mut prelude = rule.prelude();
        self.admit_qualified(at, &mut prelude, block)
            .map_err(|error| grammar_error(at, error))
    }
}

impl<'i> BlockGrammar<'i, Attached<'i>> for KeyframeBlockParser<'_> {
    fn declaration(&mut self, d: DeclarationUnit<'i>) -> Result<Self::Item, GrammarError<'i>> {
        // A declaration is no keyframe: the unit is read again as a rule.
        Err(SyntaxError::invalid_qualified_rule_error(d.start()))
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

/// The selector list of a keyframe block that the stream `input` holds.
pub(super) fn parse_keyframe_selector_list_stream<'i>(
    input: &mut Stream<'i>,
    snapshot: &crate::source::CssSourceSnapshot,
) -> std::result::Result<CssKeyframeSelectorList, ParseError<'i, CssError>> {
    read_scope(input, snapshot, read_keyframe_selector_list)
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
