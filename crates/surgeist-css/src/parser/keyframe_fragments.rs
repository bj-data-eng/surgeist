//! Name-free complete keyframe consumer boundaries over original source.

use super::*;

/// A complete raw selector list with its unmodified source observation.
///
/// This parser-produced carrier preserves surrounding whitespace and comments
/// in its origin. Its selector list retains authored order and duplicates.
#[derive(Clone, Debug, PartialEq)]
pub struct CssParsedKeyframeSelectors {
    selectors: CssKeyframeSelectorList,
    origin: CssParsedOrigin,
}

impl CssParsedKeyframeSelectors {
    /// Borrows selectors before endpoint normalization or numeric formatting.
    #[must_use]
    pub const fn selectors(&self) -> &CssKeyframeSelectorList {
        &self.selectors
    }

    /// Borrows the complete original input, including trivia.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
}

/// Exactly one admitted detached keyframe rule and its original source range.
///
/// No enclosing `@keyframes` name or brace is created. The range includes the
/// actual selector prelude and declaration braces, excluding surrounding trivia.
/// Declaration recovery belongs to the parse report.
#[derive(Clone, Debug, PartialEq)]
pub struct CssParsedKeyframeRule {
    block: CssKeyframeBlock,
    origin: CssParsedOrigin,
}

impl CssParsedKeyframeRule {
    /// Borrows the parser-only authored block and genuine declaration positions.
    #[must_use]
    pub const fn block(&self) -> &CssKeyframeBlock {
        &self.block
    }

    /// Borrows the source range of the complete detached rule.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
}

/// Parses a complete raw keyframe selector list without a synthetic rule.
///
/// The owning authored grammar admits `from`, `to`, bounded literal percentages
/// and supported symbolic percentage math. Empty, invalid, or trailing input
/// returns `None` with `RejectInput`; resource failures preserve their typed
/// diagnostic action. Order, duplicates, trivia and original coordinates survive.
/// Lexical EOF recovery remains visible and fails clean-report validation.
#[must_use]
pub fn parse_keyframe_selector_list(
    source: &str,
) -> crate::CssParseReport<Option<CssParsedKeyframeSelectors>> {
    fragments::bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let result = (|| {
            let openings =
                state.check_component_values(source, &input, "baseline.keyframes.selector")?;
            let selectors = keyframes::parse_keyframe_selector_list(
                &mut input,
                &crate::numeric::NumericInputContext::parsed(state.source_snapshot()),
            )?;
            input.expect_exhausted().map_err(basic)?;
            state.retain_component_closures(openings);
            let origin = CssParsedOrigin::from_range(state.source_snapshot(), 0..source.len())
                .expect("complete original selector input");
            Ok(CssParsedKeyframeSelectors { selectors, origin })
        })();
        finish(source, result, Vec::new(), &state)
    })
}

/// Parses exactly one complete detached keyframe rule on unmodified input.
///
/// A selector prelude and real declaration braces are required. Multiple rules,
/// outer at-rules, trailing nontrivia and structural children reject the whole
/// input. Invalid declarations use normal keyframe recovery and retain admitted
/// siblings with ordered original diagnostics. Empty blocks are admitted.
/// Genuine braces count toward the fixed nesting ceiling; input and component
/// resources share the existing parser owner. Implicit EOF closure is retained
/// with diagnostics. This supplies no live rule identity or mutation policy.
#[must_use]
pub fn parse_keyframe_rule(source: &str) -> crate::CssParseReport<Option<CssParsedKeyframeRule>> {
    crate::CssParserContext::default().parse_keyframe_rule(source)
}

impl crate::CssParserContext {
    /// Parses one detached keyframe with this context's existing declaration admission.
    #[must_use]
    pub fn parse_keyframe_rule(
        self,
        source: &str,
    ) -> crate::CssParseReport<Option<CssParsedKeyframeRule>> {
        fragments::bounded(source, || {
            let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default())
                .with_parser_context(self);
            let working_source = crate::tokenization::prepare(source);
            let mut parser_input = ParserInput::new(&working_source);
            let mut input = Parser::new(&mut parser_input);
            let mut diagnostics = Vec::new();
            let result = (|| {
                let (block, recovered) =
                    keyframes::parse_detached_rule(source, &mut input, state.clone())?;
                diagnostics = recovered;
                let start = block.position().byte_offset().value();
                // The generic single-rule driver consumes trailing trivia. The
                // source arena supplies the exact admitted rule's end boundary.
                let document = state.syntax_document(source).map_err(|error| {
                    crate::error::invalid_component_value(input.current_source_location(), error)
                })?;
                let selected = crate::syntax_consumption::consume_one_rule(
                    &mut document.cursor(document.root),
                )
                .expect("admitted exactly one rule");
                let crate::syntax_consumption::GenericRule::Qualified(rule) = selected else {
                    unreachable!("keyframe grammar admits only qualified rules")
                };
                let end = document
                    .boundary(rule.range.list, rule.range.end)
                    .source
                    .expect("original rule boundary")
                    .offset;
                let origin = CssParsedOrigin::from_range(state.source_snapshot(), start..end)
                    .expect("admitted original keyframe rule range");
                Ok(CssParsedKeyframeRule { block, origin })
            })();
            finish(source, result, diagnostics, &state)
        })
    }
}

fn finish<T>(
    source: &str,
    result: Result<T, ParseError<'_, Error>>,
    mut diagnostics: Vec<crate::CssRecoveryDiagnostic>,
    state: &RecoveryState,
) -> crate::CssParseReport<Option<T>> {
    match result {
        Ok(value) => {
            diagnostics.extend(state.take_implicit_closure_diagnostics(source));
            crate::CssParseReport::new(Some(value), diagnostics)
        }
        Err(error) => crate::CssParseReport::new(
            None,
            vec![fragments::reject(
                source,
                error,
                crate::CssRecoveryAction::RejectInput,
            )],
        ),
    }
}
