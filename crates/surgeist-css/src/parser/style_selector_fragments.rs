//! Raw selector admission in the actual style-rule grammar context.
use crate::parser::recovery::{RecoveryState, StyleContextCaptures};
use crate::parser::{fragments, selectors, syntax_bridge};
use crate::selector::{CssNamespaceContext, CssStyleSelectorList};
use crate::selector::{CssParsedStyleSelectors, CssStyleSelectorContext};
use crate::source::CssParsedOrigin;

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
                    vec![syntax_bridge::arena_error(source, error)],
                );
            }
        };
        let snapshot = document
            .source
            .as_ref()
            .expect("original selector input")
            .original
            .clone();
        let state = RecoveryState::at_depth_with_snapshot(
            source,
            0,
            StyleContextCaptures::with_namespaces(namespaces),
            snapshot.clone(),
        );
        let (selectors, diagnostics) =
            fragments::selector_fragment_with_state(source, state, false, |input, recovery| {
                match context {
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
                }
            })
            .into_parts();
        let syntax = selectors.map(|selectors| CssParsedStyleSelectors {
            selectors,
            context,
            origin: CssParsedOrigin::from_range(&snapshot, 0..source.len())
                .expect("complete original selector input"),
        });
        crate::CssParseReport::new(syntax, diagnostics)
    })
}
