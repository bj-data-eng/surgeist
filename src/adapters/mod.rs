//! Cross-crate Surgeist integration adapters.
//!
//! Backend-local adapters remain in their owning crates. This module only
//! composes public Surgeist crate contracts.

mod error;
mod retained_style;
mod style_layout;

#[cfg(test)]
mod retained_style_tests;

#[cfg(test)]
mod style_layout_tests;

#[cfg(test)]
mod tests;

pub use error::{AdapterBoundary, AdapterError, AdapterErrorKind, AdapterResult};
pub use retained_style::{
    RetainedStyleTree, clear_style_cache_for_retained_changes, style_change_from_retained_flags,
};
pub use style_layout::{
    LayoutLoweringOutput, LayoutLoweringSession, lower_style_to_layout,
    lower_style_to_layout_with_store,
};
