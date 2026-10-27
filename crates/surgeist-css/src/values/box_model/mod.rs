//! Display, sizing, spacing, overflow, float, containment and break values.

pub(crate) mod box_spacing;
pub(crate) mod break_controls;
pub(crate) mod calc_size;
pub(crate) mod contain_intrinsic_size;
pub(crate) mod display;
pub(crate) mod float_clear;
pub(crate) mod inset;
pub(crate) mod misc_property_serialization;
pub(crate) mod overflow;
pub(crate) mod overflow_controls;
pub(crate) mod sizing;
pub(crate) mod sizing_controls;
pub(crate) mod syntax;

#[doc(inline)]
pub use box_spacing::{CssMarginValue, CssPaddingValue};
#[doc(inline)]
pub use calc_size::{CssBoxCalcSize, CssCalcSize, CssCalcSizeBasisRef, CssIntrinsicSizeKeyword};
#[doc(inline)]
pub use contain_intrinsic_size::{CssContainIntrinsicSizeFallback, CssContainIntrinsicSizeValue};
#[doc(inline)]
pub use display::{
    CssDisplayBox, CssDisplayInside, CssDisplayInternal, CssDisplayLegacy,
    CssDisplayListItemInside, CssDisplayOutside, CssDisplayValue,
};
#[doc(inline)]
pub use inset::CssInsetValue;
#[doc(inline)]
pub use overflow::CssOverflowValue;
#[doc(inline)]
pub use overflow_controls::{
    CssOverflowAnchor, CssOverflowClipMargin, CssScrollBehavior, CssScrollbarGutter,
};
#[doc(inline)]
pub use sizing::{CssBoxSize, CssMaxSizeValue, CssSizeValue};
#[doc(inline)]
pub use sizing_controls::{CssFrameSizing, CssMinIntrinsicSizing};
#[doc(inline)]
pub use syntax::{
    CssAspectRatioValue, CssBoxDecorationBreak, CssBoxSizing, CssBreakBetween, CssBreakInside,
    CssClear, CssContain, CssContainComponent, CssContainComponentList, CssContentVisibility,
    CssFloat, CssFlowTolerance, CssFlowToleranceRef, CssLayoutPosition, CssOverflow, CssVisibility,
    CssZIndexValue,
};
