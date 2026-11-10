//! Raw selector admission in the actual style-rule grammar context.
use crate::parser::recovery::{RecoveryState, SharedNamespaces};
use crate::parser::{fragments, selectors, tree_build_diagnostics};
use crate::selector::{CssNamespaceContext, CssStyleSelectorList};
use crate::selector::{CssParsedStyleSelectors, CssStyleSelectorContext};
use crate::source::CssParsedOrigin;
use crate::syntax_consumption::DocumentExt;

#[must_use]
pub(crate) fn parse_style_selector_list_with_limits(
    source: &str,
    namespaces: &CssNamespaceContext,
    context: CssStyleSelectorContext,
    limits: crate::component::CssComponentValueLimits,
) -> crate::CssParseReport<Option<CssParsedStyleSelectors>> {
    fragments::bounded(source, || {
        let document = match crate::syntax_consumption::source_document_with_limits(source, limits)
        {
            Ok(document) => document,
            Err(error) => {
                return crate::CssParseReport::new(
                    None,
                    vec![tree_build_diagnostics::tree_build_failure(source, error)],
                );
            }
        };
        let snapshot = document
            .snapshot()
            .expect("original selector input")
            .clone();
        let state = RecoveryState::at_depth_with_snapshot(
            0,
            SharedNamespaces::with_namespaces(namespaces),
            snapshot.clone(),
        );
        let (selectors, diagnostics) = fragments::selector_fragment_with_state(
            source,
            &document,
            state,
            false,
            |input, recovery| match context {
                CssStyleSelectorContext::Ordinary => {
                    selectors::parse_rule_selector_list(input, recovery)
                        .map(CssStyleSelectorList::absolute)
                }
                CssStyleSelectorContext::Nested => {
                    selectors::parse_nested_style_selector_list(input, recovery)
                        .map(CssStyleSelectorList::new)
                }
                CssStyleSelectorContext::Scoped(_) => {
                    selectors::parse_scoped_style_selector_list(input, recovery)
                }
            },
        )
        .into_parts();
        let syntax = selectors.map(|selectors| CssParsedStyleSelectors {
            selectors,
            context,
            origin: CssParsedOrigin::from_range(&snapshot, 0..source.len())
                .expect("complete original selector input")
                .into(),
        });
        crate::CssParseReport::new(syntax, diagnostics)
    })
}
