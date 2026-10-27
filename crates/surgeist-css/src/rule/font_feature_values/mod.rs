//! `@font-feature-values` rules, blocks and values.

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
