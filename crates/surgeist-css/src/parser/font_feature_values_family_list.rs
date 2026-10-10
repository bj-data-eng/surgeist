//! Raw literal family preludes through the native feature-value grammar.
use super::{StyleContextCaptures, fragments, recovery::RecoveryState, syntax_bridge};
use crate::*;
use cssparser::{Parser, ParserInput};

/// Parses one complete nonempty nongeneric literal-family prelude, without
/// constructing an enclosing rule. Retention is independent of report cleanliness.
#[must_use]
pub fn parse_font_feature_values_family_list(
    source: &str,
) -> CssParseReport<Option<CssFontFeatureValuesFamilyList>> {
    parse_font_feature_values_family_list_with_limits(source, CssComponentValueLimits::default())
}
/// Charges the complete original input once, including trivia and later members.
/// Syntax rejection and resource exhaustion both return `None`, distinguished
/// by typed diagnostics. Native valid EOF recovery can retain a list with notes.
#[must_use]
pub fn parse_font_feature_values_family_list_with_limits(
    source: &str,
    limits: CssComponentValueLimits,
) -> CssParseReport<Option<CssFontFeatureValuesFamilyList>> {
    parse_with_context(source, limits, CssParserContext::default())
}
pub(crate) fn parse_with_context(
    source: &str,
    limits: CssComponentValueLimits,
    context: CssParserContext,
) -> CssParseReport<Option<CssFontFeatureValuesFamilyList>> {
    fragments::bounded(source, || {
        let document = match crate::syntax_consumption::source_document_with_limits(source, limits)
        {
            Ok(document) => document,
            Err(error) => {
                return CssParseReport::new(None, vec![syntax_bridge::arena_error(source, error)]);
            }
        };
        let snapshot = document
            .source
            .as_ref()
            .expect("original source document")
            .original
            .clone();
        let state = RecoveryState::at_depth_with_snapshot(
            source,
            0,
            StyleContextCaptures::default(),
            snapshot.clone(),
        )
        .with_parser_context(context);
        let working_source = crate::tokenization::prepare(source);
        let mut parser_input = ParserInput::new(&working_source);
        let mut input = Parser::new(&mut parser_input);
        let mut origins = Vec::new();
        let result = (|| {
            let families = super::font_feature_values::parse_families_observed(
                source,
                &mut input,
                &state,
                |range, location| {
                    let origin = CssParsedOrigin::from_range(&snapshot, range)
                        .expect("native member source range");
                    origins.try_reserve(1).map_err(|_| {
                        crate::error::invalid_component_value(
                            location,
                            CssComponentValueError::new(
                                CssComponentValueErrorKind::CapacityOverflow,
                                CssValueOrigin::Parsed(origin.clone()),
                            ),
                        )
                    })?;
                    origins.push(origin);
                    Ok(())
                },
            )?;
            input.expect_exhausted().map_err(crate::error::basic)?;
            Ok(families)
        })();
        match result {
            Ok(families) => {
                state.retain_component_closures(state.component_openings_in(0..source.len()));
                let origin = CssParsedOrigin::from_range(&snapshot, 0..source.len())
                    .expect("complete original prelude");
                CssParseReport::new(
                    Some(CssFontFeatureValuesFamilyList::from_parsed(
                        families, origin, origins,
                    )),
                    state.take_implicit_closure_diagnostics(source),
                )
            }
            Err(error) => CssParseReport::new(
                None,
                vec![fragments::reject(
                    source,
                    error,
                    CssRecoveryAction::RejectInput,
                )],
            ),
        }
    })
}
