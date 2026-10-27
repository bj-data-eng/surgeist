use crate::diagnostic::CssError;
use crate::diagnostic::error::basic;
use crate::parser::recovery::{RecoveryState, StyleContextCaptures};
use crate::parser::{fragments, keyframes};
use crate::rule::keyframes::{CssParsedKeyframeRule, CssParsedKeyframeSelectors};
use crate::source::CssParsedOrigin;
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
            let selectors = keyframes::parse_keyframe_selector_list(
                &mut input,
                &crate::numeric::expression::NumericInputContext::parsed(state.source_snapshot()),
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

pub(crate) fn parse_keyframe_rule_with_context(
    source: &str,
    context: crate::CssParserContext,
) -> crate::CssParseReport<Option<CssParsedKeyframeRule>> {
    fragments::bounded(source, || {
        let state = RecoveryState::at_depth(source, 0, StyleContextCaptures::default())
            .with_parser_context(context);
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
                crate::diagnostic::error::invalid_component_value(
                    input.current_source_location(),
                    error,
                )
            })?;
            let selected =
                crate::syntax_consumption::consume_one_rule(&mut document.cursor(document.root))
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
