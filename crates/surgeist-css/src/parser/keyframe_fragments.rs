use crate::diagnostic::CssError;
use crate::diagnostic::error::basic;
use crate::parser::recovery::{RecoveryState, StyleContextCaptures};
use crate::parser::{fragments, keyframes};
use crate::rule::keyframes::{CssParsedKeyframeRule, CssParsedKeyframeSelectors};
use crate::source::CssParsedOrigin;
use crate::syntax_consumption::DocumentExt;
use cssparser::{ParseError, Parser, ParserInput};

#[must_use]
pub(crate) fn parse_keyframe_selector_list(
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
            let selectors =
                keyframes::parse_keyframe_selector_list(&mut input, state.source_snapshot())?;
            input.expect_exhausted().map_err(basic)?;
            state.retain_component_closures(openings);
            let origin = CssParsedOrigin::from_range(state.source_snapshot(), 0..source.len())
                .expect("complete original selector input");
            Ok(CssParsedKeyframeSelectors { selectors, origin })
        })();
        finish(source, result, Vec::new(), &state)
    })
}

pub(crate) fn parse_keyframe_rule_with_context(
    source: &str,
    context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssParsedKeyframeRule>> {
    fragments::bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default())
            .with_parser_context(context);
        let snapshot = state.source_snapshot().clone();
        let document = match state.syntax_document(source) {
            Ok(document) => document,
            Err(error) => {
                return finish::<CssParsedKeyframeRule>(
                    source,
                    Err(crate::diagnostic::error::invalid_component_value(
                        crate::parser::block_units::location_at(&snapshot, 0),
                        error,
                    )),
                    Vec::new(),
                    &state,
                );
            }
        };
        let mut grammar = keyframes::detached_grammar(source, state.clone());
        let result = document
            .root()
            .parse_rule(&mut grammar)
            .map_err(|failed| {
                if failed.rule_end().is_none() && failed.error().is_end_of_stream() {
                    // Nothing but trivia: the failure is where the source begins.
                    crate::diagnostic::error::unexpected_syntax_end(
                        crate::parser::block_units::location_at(&snapshot, 0),
                    )
                } else {
                    crate::parser::block_units::native_error(&snapshot, failed.error())
                }
            })
            .and_then(|block| {
                let start = block.position().byte_offset().value();
                let unexpected = || {
                    crate::diagnostic::error::unexpected_syntax_end(
                        crate::parser::block_units::location_at(&snapshot, start),
                    )
                };
                // The generic single-rule driver consumes trailing trivia. The
                // source arena supplies the exact admitted rule's end boundary.
                let selected = crate::syntax_consumption::consume_one_rule(
                    &mut document.cursor(document.root_list()),
                )
                .map_err(|_| unexpected())?;
                let crate::syntax_consumption::GenericRule::Qualified(rule) = selected else {
                    unreachable!("keyframe grammar admits only qualified rules")
                };
                let end = document
                    .boundary(rule.range.list, rule.range.end)
                    .offset
                    .ok_or_else(unexpected)?;
                let origin = CssParsedOrigin::from_range(state.source_snapshot(), start..end)
                    .ok_or_else(unexpected)?;
                Ok(CssParsedKeyframeRule { block, origin })
            });
        let diagnostics = std::mem::take(&mut grammar.diagnostics);
        finish(source, result, diagnostics, &state)
    })
}

fn finish<T>(
    source: &str,
    result: Result<T, ParseError<'_, CssError>>,
    mut diagnostics: Vec<crate::diagnostic::CssRecoveryDiagnostic>,
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
                crate::diagnostic::CssRecoveryAction::RejectInput,
            )],
        ),
    }
}
