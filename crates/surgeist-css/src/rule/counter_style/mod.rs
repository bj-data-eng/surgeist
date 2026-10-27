//! `@counter-style` rules, descriptors and definitions.
//!
//! # Counter styles and page rules
//!
//! [`CssRule::CounterStyle`](crate::rule::CssRule::CounterStyle) retains a checked, case-sensitive [`CssCounterStyleName`], every
//! valid descriptor occurrence in authored order, the effective last valid occurrence of each
//! descriptor, and the rule position. The typed descriptor values cover Counter Styles 3
//! `system`, `negative`, `prefix`, `suffix`, `range`, `pad`, `fallback`, `symbols`,
//! `additive-symbols`, and `speak-as`. Invalid individual descriptors recover locally.
//! A grammar-valid collection can still fail to define a counter style; the separate
//! [`CssCounterStyleDefinitionStatus`] records that distinction. Prospective descriptor
//! entries retain real named occurrences or honestly source-free programmatic values.

pub(crate) mod counter_definition;
pub(crate) mod counter_style_serialization;
pub(crate) mod descriptor_values;
pub(crate) mod syntax;

#[doc(inline)]
pub use counter_definition::{
    CssCounterStyleAlgorithm, CssCounterStyleDefinitionIssue, CssCounterStyleDefinitionStatus,
    CssCounterStyleDescriptorCollection, CssCounterStyleDescriptorEntry,
};
#[doc(inline)]
pub use descriptor_values::{
    CssCounterStyleDescriptorValue, CssCounterStyleDescriptorValueRef, CssCounterStyleValueError,
};
#[doc(inline)]
pub use syntax::{
    CssCounterAdditiveSymbols, CssCounterAdditiveTuple, CssCounterStyleDescriptorKind,
    CssCounterStyleDescriptorRef, CssCounterStyleDescriptors, CssCounterStyleFixedSystem,
    CssCounterStyleName, CssCounterStyleNegative, CssCounterStylePad, CssCounterStyleRange,
    CssCounterStyleRangeBound, CssCounterStyleRangeInterval, CssCounterStyleRanges,
    CssCounterStyleRule, CssCounterStyleSpeakAs, CssCounterStyleSystem, CssCounterSymbol,
    CssCounterSymbolIdent, CssCounterSymbols,
};
