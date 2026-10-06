//! Intrinsically checked authored Masking 1 domains, before used-box or image processing.

use super::*;

/// A Masking positioning/clipping box. The selected property-specific domain
/// excludes margin-box; clip-path retains its independent geometry-box domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMaskBox {
    ContentBox,
    PaddingBox,
    BorderBox,
    FillBox,
    StrokeBox,
    ViewBox,
}

impl CssMaskBox {
    /// Narrows a general geometry box without resolving it against an element.
    #[must_use]
    pub const fn try_new(value: CssBoxEdgeKeyword) -> Option<Self> {
        Some(match value {
            CssBoxEdgeKeyword::ContentBox => Self::ContentBox,
            CssBoxEdgeKeyword::PaddingBox => Self::PaddingBox,
            CssBoxEdgeKeyword::BorderBox => Self::BorderBox,
            CssBoxEdgeKeyword::FillBox => Self::FillBox,
            CssBoxEdgeKeyword::StrokeBox => Self::StrokeBox,
            CssBoxEdgeKeyword::ViewBox => Self::ViewBox,
            CssBoxEdgeKeyword::MarginBox => return None,
        })
    }
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::ContentBox => "content-box",
            Self::PaddingBox => "padding-box",
            Self::BorderBox => "border-box",
            Self::FillBox => "fill-box",
            Self::StrokeBox => "stroke-box",
            Self::ViewBox => "view-box",
        }
    }
}

/// A mask-clip value. NoClip cannot be used as a mask-origin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMaskClip {
    Box(CssMaskBox),
    NoClip,
}

/// Retained shorthand box arity and binding; omission is stored by the layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMaskLayerBoxes {
    /// One authored box sets both origin and clip.
    Box(CssMaskBox),
    /// Two authored slots, in origin/clip order.
    Pair {
        origin: CssMaskBox,
        clip: CssMaskClip,
    },
    /// Clip only; origin remains omitted and expands to its initial.
    NoClip,
}
impl CssMaskLayerBoxes {
    #[must_use]
    pub const fn origin(self) -> Option<CssMaskBox> {
        match self {
            Self::Box(v) | Self::Pair { origin: v, .. } => Some(v),
            Self::NoClip => None,
        }
    }
    #[must_use]
    pub const fn clip(self) -> CssMaskClip {
        match self {
            Self::Box(v) => CssMaskClip::Box(v),
            Self::Pair { clip, .. } => clip,
            Self::NoClip => CssMaskClip::NoClip,
        }
    }
}

/// Image interpretation on the referencing element, independent of mask-type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMaskMode {
    Alpha,
    Luminance,
    MatchSource,
}
/// The operation between ordered mask layers, before compositing execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMaskComposite {
    Add,
    Subtract,
    Intersect,
    Exclude,
}
/// SVG mask interpretation, also the scalar domain of mask-border-mode.
/// Property wrappers retain their distinct initials and applicability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMaskType {
    Luminance,
    Alpha,
}
/// SVG clipping winding, independent of CSS basic-shape fill-rule values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssClipRule {
    Nonzero,
    Evenodd,
}

macro_rules! mask_list {
    ($list:ident, $item:ty, $accessor:ident) => {
        /// A nonempty comma-separated authored list, independent of image count.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $list {
            values: Vec<$item>,
        }
        impl $list {
            /// Rejects an empty list; preserves every authored entry in order.
            #[must_use]
            pub fn try_new(values: Vec<$item>) -> Option<Self> {
                (!values.is_empty()).then_some(Self { values })
            }
            #[must_use]
            pub fn $accessor(&self) -> &[$item] {
                &self.values
            }
        }
    };
}
mask_list!(CssMaskBoxList, CssMaskBox, boxes);
mask_list!(CssMaskClipList, CssMaskClip, clips);
mask_list!(CssMaskModeList, CssMaskMode, modes);
mask_list!(CssMaskCompositeList, CssMaskComposite, operators);

/// Authored mask-border components. Four-edge child domains reuse the checked
/// border-image owners; defaults and shorthand slash grammar belong to Masking.
#[derive(Clone, Debug, PartialEq)]
pub struct CssMaskBorder {
    source: Option<CssImageValue>,
    slice: Option<CssBorderImageSlice>,
    width: Option<CssBorderImageWidth>,
    outset: Option<CssBorderImageOutset>,
    repeat: Option<CssBorderImageRepeat>,
    mode: Option<CssMaskType>,
}
impl CssMaskBorder {
    /// Requires a nonempty shorthand and a slice before width/outset slash groups.
    /// Every numeric/image child is already checked by its semantic owner.
    #[must_use]
    pub fn try_new(
        source: Option<CssImageValue>,
        slice: Option<CssBorderImageSlice>,
        width: Option<CssBorderImageWidth>,
        outset: Option<CssBorderImageOutset>,
        repeat: Option<CssBorderImageRepeat>,
        mode: Option<CssMaskType>,
    ) -> Option<Self> {
        if source.is_none() && slice.is_none() && repeat.is_none() && mode.is_none()
            || slice.is_none() && (width.is_some() || outset.is_some())
        {
            return None;
        }
        Some(Self {
            source,
            slice,
            width,
            outset,
            repeat,
            mode,
        })
    }
    #[must_use]
    pub const fn source(&self) -> Option<&CssImageValue> {
        self.source.as_ref()
    }
    #[must_use]
    pub const fn slice(&self) -> Option<&CssBorderImageSlice> {
        self.slice.as_ref()
    }
    #[must_use]
    pub const fn width(&self) -> Option<&CssBorderImageWidth> {
        self.width.as_ref()
    }
    #[must_use]
    pub const fn outset(&self) -> Option<&CssBorderImageOutset> {
        self.outset.as_ref()
    }
    #[must_use]
    pub const fn repeat(&self) -> Option<CssBorderImageRepeat> {
        self.repeat
    }
    #[must_use]
    pub const fn mode(&self) -> Option<CssMaskType> {
        self.mode
    }
}
