//! `@color-profile` rules and descriptors.
//!
//! A `@color-profile` rule defines a named or device color profile through the descriptors
//! `src`, `rendering-intent` and `components`. The rule retains every valid descriptor occurrence
//! in authored order; loading the profile resource belongs to downstream owners.

pub(crate) mod color_profile_serialization;
pub(crate) mod rule;

#[doc(inline)]
pub use rule::{
    CssColorProfileDescriptor, CssColorProfileDescriptorKind, CssColorProfileDescriptorValue,
    CssColorProfileDescriptorValueRef, CssColorProfileRenderingIntent, CssColorProfileRule,
    CssColorProfileRuleName, CssColorProfileValueError,
};
