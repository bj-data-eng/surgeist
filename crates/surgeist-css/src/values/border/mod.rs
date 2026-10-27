//! Border, outline and line style and width values.

pub(crate) mod border_color;
pub(crate) mod border_image_serialization;
pub(crate) mod border_radius;
pub(crate) mod border_style;
pub(crate) mod border_width;
pub(crate) mod outline_serialization;
pub(crate) mod syntax;

#[doc(inline)]
pub use border_radius::CssBorderRadiusShorthand;
#[doc(inline)]
pub use border_width::{CssBorder, CssLineWidth};
#[doc(inline)]
pub use syntax::{
    CssBorderImage, CssBorderImageOutset, CssBorderImageOutsetComponent, CssBorderImageRepeat,
    CssBorderImageRepeatKeyword, CssBorderImageSlice, CssBorderImageSliceComponent,
    CssBorderImageWidth, CssBorderImageWidthComponent, CssLineStyle, CssOutline, CssOutlineColor,
    CssOutlineStyle,
};
