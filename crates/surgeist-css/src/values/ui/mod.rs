//! Cursor, caret, appearance, interactivity, scrollbar and will-change values.
//!
//! User-interface values: `cursor` (keywords, images and URL sets), caret and accent colors,
//! `appearance`, `user-select`, `pointer-events`, `resize`, scrollbar properties, spatial and
//! sequential navigation, interactivity and `will-change`. They retain authored syntax; no
//! hit testing or interaction is performed.

pub(crate) mod cursor_serialization;
pub(crate) mod cursor_values;
pub(crate) mod scrollbar;
pub(crate) mod syntax;
pub(crate) mod ui_serialization;
pub(crate) mod values;
pub(crate) mod will_change;

#[doc(inline)]
pub use cursor_values::{
    CssCursor, CssCursorImage, CssCursorImageSource, CssCursorImages, CssCursorUrlSet,
    CssCursorUrlSetDescriptor, CssCursorUrlSetOption, CssCursorUrlSetReference,
};
#[doc(inline)]
pub use scrollbar::CssScrollbarColor;
#[doc(inline)]
pub use syntax::{
    CssCaretColor, CssCursorKeyword, CssPointerEvents, CssResize, CssScrollbarWidth, CssUserSelect,
};
#[doc(inline)]
pub use values::{
    CssAccentColor, CssAppearance, CssCaret, CssCaretAnimation, CssCaretShape, CssInteractivity,
    CssInterestDelayValue, CssLegacyNavigationTargetName, CssNavigation, CssNavigationId,
    CssNavigationReference, CssNavigationTarget, CssNavigationTargetName,
};
#[doc(inline)]
pub use will_change::{
    CssWillChange, CssWillChangeFeature, CssWillChangeFeatures, CssWillChangePropertyName,
};
