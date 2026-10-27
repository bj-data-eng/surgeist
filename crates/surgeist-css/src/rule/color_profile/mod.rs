//! `@color-profile` rules and descriptors.

pub(crate) mod color_profile_serialization;
pub(crate) mod rule;

#[doc(inline)]
pub use rule::{
    CssColorProfileDescriptor, CssColorProfileDescriptorKind, CssColorProfileDescriptorValue,
    CssColorProfileDescriptorValueRef, CssColorProfileRenderingIntent, CssColorProfileRule,
    CssColorProfileRuleName, CssColorProfileValueError,
};
