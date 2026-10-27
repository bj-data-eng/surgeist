//! `@font-feature-values` rules, blocks and values.
//!
//! # Font feature values
//!
//! `@font-feature-values` retains ordered families, all seven subsidiary blocks,
//! repeated definitions, and interleaved `font-display` occurrences through
//! [`CssFontFeatureValuesRule`]. Checked constructors share parser constraints;
//! [`CssFontFeatureValueIndex`] preserves exact nonnegative decimal digits without a
//! machine-integer limit. Parsed integer origins address their complete original
//! token, while Rust construction has no invented source coordinates.
//!
//! The pinned Fonts 4 draft (7 September 2026) conflicts between sections 6.9.1
//! and 6.9.2. The selected section 6.9.2 / frozen WebKit policy admits one or two
//! character-variant indexes and nonempty styleset lists without feature-specific
//! upper bounds. The normative contradiction remains unresolved. All index
//! positions retain arbitrarily large exact nonnegative authored integer tokens;
//! downstream font activation may ignore unsupported feature indexes.
//!
//! Ordinary stylesheet/group/scope rule lists retain these global named rules;
//! style-rule ancestry rejects them. Normalization retains one opaque payload and
//! its parent contexts, without emitting property contributions or applying font
//! mapping/cascade. Normalization limits do not count payload declarations or bound
//! payload allocation. Fonts rules have canonical effective specified writers;
//! generic group/sheet wrapper coverage remains with its separate owner.

pub(crate) mod descriptor_values;
pub(crate) mod rule;

#[doc(inline)]
pub use descriptor_values::{
    CssFontFeatureDisplayValue, CssFontFeatureDisplayValueRef, CssFontFeatureValue,
    CssFontFeatureValueRef,
};
#[doc(inline)]
pub use rule::{
    CssFontFeatureDisplayOccurrence, CssFontFeatureValueBlock, CssFontFeatureValueDefinition,
    CssFontFeatureValueIndex, CssFontFeatureValueKind, CssFontFeatureValueName,
    CssFontFeatureValuesError, CssFontFeatureValuesFamilyList, CssFontFeatureValuesItem,
    CssFontFeatureValuesRule,
};
