//! Raw literal family preludes through the native feature-value grammar.
use super::recovery::RecoveryState;
use super::{fragments, syntax_bridge};
use crate::component::values::CssComponentValueErrorKind;
use crate::component::{CssComponentValueError, CssComponentValueLimits, CssValueOrigin};
use crate::diagnostic::CssRecoveryAction;
use crate::parser::recovery::SharedNamespaces;
use crate::rule::font_feature_values::CssFontFeatureValuesFamilyList;
use crate::source::CssParsedOrigin;
use crate::syntax_consumption::DocumentExt;
use crate::{CssParseReport, CssParserContext};

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
            .snapshot()
            .expect("original source document")
            .clone();
        let state =
            RecoveryState::at_depth_with_snapshot(0, SharedNamespaces::default(), snapshot.clone())
                .with_parser_context(context);
        let mut origins = Vec::new();
        let result = {
            let mut root = document.root();
            super::font_feature_values::parse_families_observed(
                source,
                &mut root,
                &state,
                |range, location| {
                    let origin = CssParsedOrigin::from_range(&snapshot, range)
                        .expect("native member source range");
                    origins.try_reserve(1).map_err(|_| {
                        crate::diagnostic::error::invalid_component_value(
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
            )
        };
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
