//! Raw selector admission in the actual style-rule grammar context.
use super::*;

/// The actual selector grammar at an authored style rule's destination.
///
/// Scope context applies until an intervening style rule supplies ordinary
/// nesting context. An enclosing style ancestor through a scope remains an
/// explicit fact; it does not change scope anchors into nesting selectors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssStyleSelectorContext {
    /// No enclosing style or scope: leading relative combinators are invalid.
    Ordinary,
    /// An enclosing style rule supplies ordinary symbolic nesting context.
    Nested,
    /// A scope supplies symbolic scope anchors, with actual style ancestry.
    Scoped(crate::CssStyleAncestor),
}

/// Existing checked selector carriers, selected by their actual grammar.
#[derive(Clone, Debug, PartialEq)]
pub enum CssAdmittedStyleSelectors {
    /// Ordinary or nested style selectors; explicit anchors mean nesting.
    Ordinary(CssStyleSelectorList),
    /// Scope-body style selectors; explicit anchors mean scope anchors.
    Scoped(CssScopedStyleSelectorList),
}

/// A parser-produced complete raw selector input and its contextual payload.
///
/// This immutable authored carrier assigns no live identity and binds no parent
/// selectors. Its origin includes all original trivia. Token/component origins
/// within the payload share this input occurrence, never a fabricated rule.
#[derive(Clone, Debug, PartialEq)]
pub struct CssParsedStyleSelectors {
    selectors: CssAdmittedStyleSelectors,
    context: CssStyleSelectorContext,
    origin: CssParsedOrigin,
}

impl CssParsedStyleSelectors {
    /// Borrows the existing checked list without converting its anchor domain.
    #[must_use]
    pub const fn selectors(&self) -> &CssAdmittedStyleSelectors {
        &self.selectors
    }

    /// Returns the actual admission context, including supplied style ancestry.
    #[must_use]
    pub const fn context(&self) -> CssStyleSelectorContext {
        self.context
    }

    /// Borrows the complete original input occurrence, including trivia.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
}

/// Parses a complete raw style-selector list in its actual destination context.
///
/// Ordinary lists reject leading combinators. Nested and scoped lists retain
/// explicit relative relationships and implicit anchors symbolically, using the
/// same grammar as authored rules. Namespace bindings are supplied explicitly.
/// One invalid outer member or trailing nontrivia rejects the complete input as
/// `None`; forgiving inner recovery and EOF closures retain ordered diagnostics.
/// Retention, rather than report cleanliness, is the selectorText admission test.
/// Resource failures also return `None` but keep their typed diagnostic cause
/// and resource action, so a consumer can distinguish them from syntax no-ops.
///
/// ```
/// use surgeist_css::{CssAdmittedStyleSelectors, CssNamespaceContext,
///     CssSelectorCombinator, CssStyleSelector, CssStyleSelectorContext,
///     parse_style_selector_list};
/// let report = parse_style_selector_list(
///     "> .child", &CssNamespaceContext::default(), CssStyleSelectorContext::Nested);
/// let parsed = report.syntax().as_ref().unwrap();
/// let CssAdmittedStyleSelectors::Ordinary(list) = parsed.selectors() else {
///     panic!("ordinary nesting domain");
/// };
/// let CssStyleSelector::Relative(child) = &list.selectors()[0] else {
///     panic!("explicit relative relationship");
/// };
/// assert_eq!(child.combinator(), CssSelectorCombinator::Child);
/// assert_eq!(parsed.origin().source().as_str(), "> .child");
/// ```
#[must_use]
pub fn parse_style_selector_list(
    source: &str,
    namespaces: &CssNamespaceContext,
    context: CssStyleSelectorContext,
) -> crate::CssParseReport<Option<CssParsedStyleSelectors>> {
    parse_style_selector_list_with_limits(
        source,
        namespaces,
        context,
        crate::CssComponentValueLimits::default(),
    )
}

/// Parses with cumulative component limits over the complete original input.
///
/// Bytes include trivia and trailing input; components include every function,
/// descendant and trivia node. The owning syntax preflight charges this input
/// once, before the existing selector grammar probes. No member gets a fresh
/// allowance. Failure publishes no payload; an unchanged larger-budget retry is
/// independent and preserves the caller's input. Formatting has its own limits.
#[must_use]
pub fn parse_style_selector_list_with_limits(
    source: &str,
    namespaces: &CssNamespaceContext,
    context: CssStyleSelectorContext,
    limits: crate::CssComponentValueLimits,
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
                            .map(CssAdmittedStyleSelectors::Ordinary)
                    }
                    CssStyleSelectorContext::Nested => {
                        selectors::parse_nested_style_selector_list(input, recovery)
                            .map(CssStyleSelectorList::new)
                            .map(CssAdmittedStyleSelectors::Ordinary)
                    }
                    CssStyleSelectorContext::Scoped(_) => {
                        selectors::parse_scoped_style_selector_list(input, recovery)
                            .map(CssAdmittedStyleSelectors::Scoped)
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
