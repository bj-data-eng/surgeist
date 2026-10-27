//! Intrinsically checked authored Masking 1 domains, before used-box or image processing.

use crate::values::border::{
    CssBorderImageOutset, CssBorderImageRepeat, CssBorderImageSlice, CssBorderImageWidth,
};
use crate::values::image::CssImageValue;
use crate::values::transform::CssCoordBox;

/// A mask-clip value. NoClip cannot be used as a mask-origin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMaskClip {
    /// A `<coord-box>` clip area.
    Box(CssCoordBox),
    /// `no-clip`: the layer is not clipped to any area.
    NoClip,
}

/// Retained shorthand box arity and binding; omission is stored by the layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMaskLayerBoxes {
    /// One authored box sets both origin and clip.
    Box(CssCoordBox),
    /// Two authored slots, in origin/clip order.
    Pair {
        /// The authored `mask-origin` box.
        origin: CssCoordBox,
        /// The authored `mask-clip` value, which may be `no-clip`.
        clip: CssMaskClip,
    },
    /// Clip only; origin remains omitted and expands to its initial.
    NoClip,
}
impl CssMaskLayerBoxes {
    /// Returns the authored origin box, or `None` for [`CssMaskLayerBoxes::NoClip`], where the origin was omitted.
    #[must_use]
    pub const fn origin(self) -> Option<CssCoordBox> {
        match self {
            Self::Box(v) | Self::Pair { origin: v, .. } => Some(v),
            Self::NoClip => None,
        }
    }
    /// Returns the clip: the single or second authored box, or `no-clip`.
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
    /// `alpha`: the mask image's alpha channel is used as the mask.
    Alpha,
    /// `luminance`: the mask image's luminance values are used as the mask.
    Luminance,
    /// `match-source`: uses alpha masking for an image reference and the referenced `mask` element's `mask-type` for a mask reference.
    MatchSource,
}
/// The operation between ordered mask layers, before compositing execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMaskComposite {
    /// `add`: the source is placed over the destination.
    Add,
    /// `subtract`: the source is placed where it falls outside the destination.
    Subtract,
    /// `intersect`: the parts of the source that overlap the destination replace the destination.
    Intersect,
    /// `exclude`: the non-overlapping regions of source and destination are combined.
    Exclude,
}
/// SVG mask interpretation, also the scalar domain of mask-border-mode.
/// Property wrappers retain their distinct initials and applicability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMaskType {
    /// `luminance`: the mask's luminance values are used.
    Luminance,
    /// `alpha`: the mask's alpha values are used.
    Alpha,
}
/// SVG clipping winding, independent of CSS basic-shape fill-rule values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssClipRule {
    /// `nonzero`: the `nonzero` algorithm, whose definition Masking 1 takes from the SVG
    /// `fill-rule` property.
    Nonzero,
    /// `evenodd`: the `evenodd` algorithm, whose definition Masking 1 takes from the SVG
    /// `fill-rule` property.
    Evenodd,
}

macro_rules! mask_list {
    ($list:ident, $item:ty, $accessor:ident, $accessor_doc:literal) => {
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
            #[doc = $accessor_doc]
            #[must_use]
            pub fn $accessor(&self) -> &[$item] {
                &self.values
            }
        }
    };
}
mask_list!(
    CssMaskBoxList,
    CssCoordBox,
    boxes,
    "Returns the authored `<coord-box>` values in comma order."
);
mask_list!(
    CssMaskClipList,
    CssMaskClip,
    clips,
    "Returns the authored `mask-clip` values in comma order."
);
mask_list!(
    CssMaskModeList,
    CssMaskMode,
    modes,
    "Returns the authored `mask-mode` values in comma order."
);
mask_list!(
    CssMaskCompositeList,
    CssMaskComposite,
    operators,
    "Returns the authored `mask-composite` operators in comma order."
);

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
    /// Returns the authored `mask-border-source`, or `None` when omitted.
    #[must_use]
    pub const fn source(&self) -> Option<&CssImageValue> {
        self.source.as_ref()
    }
    /// Returns the authored `mask-border-slice`, or `None` when omitted.
    #[must_use]
    pub const fn slice(&self) -> Option<&CssBorderImageSlice> {
        self.slice.as_ref()
    }
    /// Returns the authored `mask-border-width`, or `None` when omitted.
    #[must_use]
    pub const fn width(&self) -> Option<&CssBorderImageWidth> {
        self.width.as_ref()
    }
    /// Returns the authored `mask-border-outset`, or `None` when omitted.
    #[must_use]
    pub const fn outset(&self) -> Option<&CssBorderImageOutset> {
        self.outset.as_ref()
    }
    /// Returns the authored `mask-border-repeat`, or `None` when omitted.
    #[must_use]
    pub const fn repeat(&self) -> Option<CssBorderImageRepeat> {
        self.repeat
    }
    /// Returns the authored `mask-border-mode`, or `None` when omitted.
    #[must_use]
    pub const fn mode(&self) -> Option<CssMaskType> {
        self.mode
    }
}
