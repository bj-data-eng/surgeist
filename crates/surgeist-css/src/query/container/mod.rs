//! Container queries and container properties.
//!
//! The authored container-query grammar: the `@container` prelude with its optional name and
//! size, `style()` and `scroll-state()` queries, and the container properties (`container-name`,
//! `container-type` and the `container` shorthand). These types retain authored syntax;
//! no container is selected or queried.

pub(crate) mod construction;
pub(crate) mod container_features;
pub(crate) mod container_properties;
pub(crate) mod container_scroll;
pub(crate) mod container_style;
pub(crate) mod syntax;

#[doc(inline)]
pub use container_features::{
    CssContainerLength, CssContainerLengthRef, CssContainerOrientation, CssContainerOrientationRef,
    CssContainerPendingValue, CssContainerRatio, CssContainerRatioRef, CssContainerSizeFeatureKind,
    CssContainerValueDomain,
};
#[doc(inline)]
pub use container_properties::{
    CssContainer, CssContainerNameList, CssContainerNames, CssContainerType,
};
#[doc(inline)]
pub use container_scroll::{
    CssContainerScrollDirectionKeyword, CssContainerScrollDirectionValue,
    CssContainerScrollDirectionValueRef, CssContainerScrollFeature, CssContainerScrollFeatureKind,
    CssContainerSnappedKeyword, CssContainerSnappedValue, CssContainerSnappedValueRef,
    CssContainerStuckKeyword, CssContainerStuckValue, CssContainerStuckValueRef,
    CssScrollStateLeaf,
};
#[doc(inline)]
pub use container_style::{
    CssContainerStyleFeature, CssContainerStyleFeatureName, CssContainerStyleRange,
    CssContainerStyleRangeOperand, CssContainerStyleRangeOperandRef, CssContainerStyleRangeRef,
    CssContainerStyleValue, CssStyleQueryLeaf,
};
#[doc(inline)]
pub use syntax::{
    CssContainerConstructionError, CssContainerFeatureQuery, CssContainerGeneralEnclosed,
    CssContainerLeaf, CssContainerName, CssContainerPrelude, CssContainerQueryEntry,
};
