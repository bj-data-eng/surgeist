/// A nonempty scope boundary list. Its members may be explicitly relative; no
/// member contains a pseudo-element.
#[derive(Clone, Debug, PartialEq)]
pub struct CssScopeSelectorList {
    selectors: Vec<CssStyleSelector>,
}

impl CssScopeSelectorList {
    #[must_use]
    pub fn try_new(selectors: Vec<CssStyleSelector>) -> Option<Self> {
        if selectors.is_empty()
            || selectors
                .iter()
                .any(|member| member.selector().has_pseudo_elements())
        {
            None
        } else {
            Some(Self::new(selectors))
        }
    }

    #[must_use]
    pub(crate) fn new(selectors: Vec<CssStyleSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        debug_assert!(
            !selectors
                .iter()
                .any(|member| member.selector().has_pseudo_elements())
        );
        Self { selectors }
    }

    /// Returns the boundary members; a leading combinator is retained without
    /// fabricating an explicit anchor. Root relatives require an enclosing style
    /// or scope; limit relatives refer to the scope introduced by their rule.
    #[must_use]
    pub fn selectors(&self) -> &[CssStyleSelector] {
        &self.selectors
    }
}

/// A nonempty selector list for an authored style rule.
///
/// Ordinary stylesheet contexts accept only non-relative selectors. Nested style contexts and
/// scope bodies also accept relative selectors; their relationship to the parent or scope stays
/// symbolic until matching. Explicit `&` anchors mean nesting in a style context and scope
/// anchors in a scope body; the list itself records which, in its selectors.
#[derive(Clone, Debug, PartialEq)]
pub struct CssStyleSelectorList {
    selectors: Vec<CssStyleSelector>,
}

impl CssStyleSelectorList {
    /// Returns `None` for an empty list.
    #[must_use]
    pub fn try_new(selectors: Vec<CssStyleSelector>) -> Option<Self> {
        if selectors.is_empty() {
            None
        } else {
            Some(Self::new(selectors))
        }
    }

    pub(crate) fn new(selectors: Vec<CssStyleSelector>) -> Self {
        debug_assert!(!selectors.is_empty());
        Self { selectors }
    }

    pub(crate) fn absolute(selectors: Vec<CssSelector>) -> Self {
        Self::new(
            selectors
                .into_iter()
                .map(CssStyleSelector::Selector)
                .collect(),
        )
    }

    #[must_use]
    pub fn selectors(&self) -> &[CssStyleSelector] {
        &self.selectors
    }
}

/// An authored selector, or an explicitly relative selector in a nested rule.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssStyleSelector {
    Selector(CssSelector),
    Relative(CssRelativeSelector),
}

impl CssStyleSelector {
    /// Returns the selector itself; [`Self::Relative`] also retains its leading combinator.
    #[must_use]
    pub const fn selector(&self) -> &CssSelector {
        match self {
            Self::Selector(selector) => selector,
            Self::Relative(relative) => relative.selector(),
        }
    }
}
use crate::selector::{CssRelativeSelector, CssSelector};
