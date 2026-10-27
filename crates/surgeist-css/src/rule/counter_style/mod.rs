//! `@counter-style` rules, descriptors and definitions.

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
