//! Background and blend-mode values.

pub(crate) mod background_layer_serialization;
pub(crate) mod background_serialization;
pub(crate) mod position;
pub(crate) mod syntax;

#[doc(inline)]
pub use position::{CssBackgroundPosition, CssBackgroundPositionList};
#[doc(inline)]
pub use syntax::{
    CssBackground, CssBackgroundAttachment, CssBackgroundAttachmentList, CssBackgroundBox,
    CssBackgroundBoxList, CssBackgroundConstructionError, CssBackgroundLayer,
    CssBackgroundLayerBoxes, CssBackgroundRepeat, CssBackgroundRepeatList,
    CssBackgroundRepeatStyle, CssBackgroundSize, CssBackgroundSizeList, CssBlendMode,
    CssBlendModeList,
};
