//! View transition names and pseudo arguments.
//!
//! View Transitions 1 names: [`CssViewTransitionName`] (`none` or a case-sensitive
//! [`CssViewTransitionIdent`]) and the wildcard or identifier arguments of the view-transition
//! pseudo-elements ([`CssViewTransitionNameSelector`]).

pub(crate) mod transitions;

#[doc(inline)]
pub use transitions::{
    CssViewTransitionIdent, CssViewTransitionName, CssViewTransitionNameSelector,
};
