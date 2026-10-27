use crate::numeric::{CssLengthPercentage, CssNonNegative, CssNumeric};
use crate::values::common::CssAutoOr;
use crate::values::image::CssImageValue;

/// One authored Compositing and Blending Level 1 `<blend-mode>` keyword.
///
/// The value identifies authored syntax only; it does not perform painting or
/// compositing calculations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssBlendMode {
    Normal,
    Darken,
    Multiply,
    ColorBurn,
    Lighten,
    Screen,
    ColorDodge,
    Overlay,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl CssBlendMode {
    /// Converts one decoded CSS keyword to its typed blend-mode identity.
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Some(match keyword.to_ascii_lowercase().as_str() {
            "normal" => Self::Normal,
            "darken" => Self::Darken,
            "multiply" => Self::Multiply,
            "color-burn" => Self::ColorBurn,
            "lighten" => Self::Lighten,
            "screen" => Self::Screen,
            "color-dodge" => Self::ColorDodge,
            "overlay" => Self::Overlay,
            "soft-light" => Self::SoftLight,
            "hard-light" => Self::HardLight,
            "difference" => Self::Difference,
            "exclusion" => Self::Exclusion,
            "hue" => Self::Hue,
            "saturation" => Self::Saturation,
            "color" => Self::Color,
            "luminosity" => Self::Luminosity,
            _ => return None,
        })
    }

    /// Returns the canonical CSS keyword spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Darken => "darken",
            Self::Multiply => "multiply",
            Self::ColorBurn => "color-burn",
            Self::Lighten => "lighten",
            Self::Screen => "screen",
            Self::ColorDodge => "color-dodge",
            Self::Overlay => "overlay",
            Self::SoftLight => "soft-light",
            Self::HardLight => "hard-light",
            Self::Difference => "difference",
            Self::Exclusion => "exclusion",
            Self::Hue => "hue",
            Self::Saturation => "saturation",
            Self::Color => "color",
            Self::Luminosity => "luminosity",
        }
    }
}

/// A nonempty ordered authored comma-separated blend-mode list.
///
/// # Specified serialization
///
/// Emits every represented blend mode in its authored comma-list order, including duplicates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBlendModeList {
    modes: Vec<CssBlendMode>,
}

impl CssBlendModeList {
    #[must_use]
    pub fn try_new(modes: Vec<CssBlendMode>) -> Option<Self> {
        (!modes.is_empty()).then_some(Self { modes })
    }

    #[must_use]
    pub fn modes(&self) -> &[CssBlendMode] {
        &self.modes
    }
}

use crate::values::background::CssBackgroundPosition;

/// The exact authored background box component count for one shorthand layer.
///
/// One box supplies both origin and clip. Two boxes preserve their authored
/// origin-then-clip order without resolving either box against layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundLayerBoxes {
    One(CssBackgroundBox),
    OriginAndClip {
        origin: CssBackgroundBox,
        clip: CssBackgroundBox,
    },
}

impl CssBackgroundLayerBoxes {
    /// Returns the shorthand layer's background-origin component.
    #[must_use]
    pub const fn origin(self) -> CssBackgroundBox {
        match self {
            Self::One(value) | Self::OriginAndClip { origin: value, .. } => value,
        }
    }

    /// Returns the shorthand layer's background-clip component.
    #[must_use]
    pub const fn clip(self) -> CssBackgroundBox {
        match self {
            Self::One(value) | Self::OriginAndClip { clip: value, .. } => value,
        }
    }
}

/// One checked authored `background` shorthand layer.
///
/// Checked construction retains omissions and requires a position with a size.
/// The enclosing background additionally restricts colors to its final layer.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackgroundLayer {
    image: Option<CssImageValue>,
    position: Option<CssBackgroundPosition>,
    size: Option<CssBackgroundSize>,
    repeat: Option<CssBackgroundRepeat>,
    attachment: Option<CssBackgroundAttachment>,
    boxes: Option<CssBackgroundLayerBoxes>,
    color: Option<CssColor>,
}

impl CssBackgroundLayer {
    /// Constructs one nonempty authored layer without filling omitted components.
    /// A size requires a position; final-layer color validity belongs to [`CssBackground`].
    pub fn try_new(
        image: Option<CssImageValue>,
        position: Option<CssBackgroundPosition>,
        size: Option<CssBackgroundSize>,
        repeat: Option<CssBackgroundRepeat>,
        attachment: Option<CssBackgroundAttachment>,
        boxes: Option<CssBackgroundLayerBoxes>,
        color: Option<CssColor>,
    ) -> Result<Self, CssBackgroundConstructionError> {
        if size.is_some() && position.is_none() {
            return Err(CssBackgroundConstructionError::from(
                BackgroundConstructionKind::SizeWithoutPosition,
            ));
        }
        if image.is_none()
            && position.is_none()
            && size.is_none()
            && repeat.is_none()
            && attachment.is_none()
            && boxes.is_none()
            && color.is_none()
        {
            return Err(CssBackgroundConstructionError::from(
                BackgroundConstructionKind::EmptyLayer,
            ));
        }
        Ok(Self {
            image,
            position,
            size,
            repeat,
            attachment,
            boxes,
            color,
        })
    }

    /// Returns this layer's authored image, if present.
    #[must_use]
    pub const fn image(&self) -> Option<&CssImageValue> {
        self.image.as_ref()
    }

    /// Returns this layer's authored position, if present.
    #[must_use]
    pub const fn position(&self) -> Option<&CssBackgroundPosition> {
        self.position.as_ref()
    }

    /// Returns the size coupled to this layer's position, if authored.
    #[must_use]
    pub const fn size(&self) -> Option<&CssBackgroundSize> {
        self.size.as_ref()
    }

    /// Returns this layer's authored repeat style, if present.
    #[must_use]
    pub const fn repeat(&self) -> Option<CssBackgroundRepeat> {
        self.repeat
    }

    /// Returns this layer's authored attachment, if present.
    #[must_use]
    pub const fn attachment(&self) -> Option<CssBackgroundAttachment> {
        self.attachment
    }

    /// Returns the exact one- or two-box shorthand component, if present.
    #[must_use]
    pub const fn boxes(&self) -> Option<CssBackgroundLayerBoxes> {
        self.boxes
    }

    /// Returns the authored color on the final layer, if present.
    #[must_use]
    pub const fn color(&self) -> Option<&CssColor> {
        self.color.as_ref()
    }
}

/// Why checked authored background construction cannot represent a valid shorthand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssBackgroundConstructionError {
    kind: BackgroundConstructionKind,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BackgroundConstructionKind {
    EmptyLayer,
    SizeWithoutPosition,
    EmptyLayers,
    NonFinalColor { layer_index: usize },
}
impl From<BackgroundConstructionKind> for CssBackgroundConstructionError {
    fn from(kind: BackgroundConstructionKind) -> Self {
        Self { kind }
    }
}
impl CssBackgroundConstructionError {
    crate::diagnostic::error::kind_predicates! { kind;
        /// Returns whether a background layer has no component.
        is_empty_layer => BackgroundConstructionKind::EmptyLayer,
        /// Returns whether a background size was given without a position.
        is_size_without_position => BackgroundConstructionKind::SizeWithoutPosition,
        /// Returns whether the layer list is empty.
        is_empty_layers => BackgroundConstructionKind::EmptyLayers,
    }
    /// Returns the index of a nonfinal layer that carries a background color.
    #[must_use]
    pub const fn non_final_color_layer(&self) -> Option<usize> {
        match self.kind {
            BackgroundConstructionKind::NonFinalColor { layer_index } => Some(layer_index),
            _ => None,
        }
    }
}
impl std::fmt::Display for CssBackgroundConstructionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            BackgroundConstructionKind::EmptyLayer => {
                formatter.write_str("background layer is empty")
            }
            BackgroundConstructionKind::SizeWithoutPosition => {
                formatter.write_str("background size requires a position")
            }
            BackgroundConstructionKind::EmptyLayers => {
                formatter.write_str("background layer list is empty")
            }
            BackgroundConstructionKind::NonFinalColor { layer_index } => write!(
                formatter,
                "background color in nonfinal layer {layer_index}"
            ),
        }
    }
}

impl std::error::Error for CssBackgroundConstructionError {}

/// A nonempty authored comma list of `background` shorthand layers.
///
/// # Specified serialization
///
/// Serializes canonical specified layers without resolving contextual values. Simple literal
/// initials may be omitted; authored omissions remain omitted.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackground {
    layers: Vec<CssBackgroundLayer>,
}

impl CssBackground {
    /// Constructs a nonempty authored list with a color only on its final layer.
    pub fn try_new(
        layers: Vec<CssBackgroundLayer>,
    ) -> Result<Self, CssBackgroundConstructionError> {
        if layers.is_empty() {
            return Err(CssBackgroundConstructionError::from(
                BackgroundConstructionKind::EmptyLayers,
            ));
        }
        for (layer_index, layer) in layers[..layers.len() - 1].iter().enumerate() {
            if layer.color().is_some() {
                return Err(CssBackgroundConstructionError::from(
                    BackgroundConstructionKind::NonFinalColor { layer_index },
                ));
            }
        }
        Ok(Self { layers })
    }

    /// Returns the authored layers in comma order.
    #[must_use]
    pub fn layers(&self) -> &[CssBackgroundLayer] {
        &self.layers
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundSize {
    Cover,
    Contain,
    Explicit {
        width: CssAutoOr<CssNumeric<CssLengthPercentage, CssNonNegative>>,
        height: Option<CssAutoOr<CssNumeric<CssLengthPercentage, CssNonNegative>>>,
    },
}

/// # Specified serialization
///
/// Serializes the nonempty authored layers in comma order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackgroundSizeList {
    sizes: Vec<CssBackgroundSize>,
}

impl CssBackgroundSizeList {
    #[must_use]
    pub fn try_new(sizes: Vec<CssBackgroundSize>) -> Option<Self> {
        if sizes.is_empty() {
            None
        } else {
            Some(Self::new(sizes))
        }
    }

    #[must_use]
    pub(crate) fn new(sizes: Vec<CssBackgroundSize>) -> Self {
        Self { sizes }
    }

    #[must_use]
    pub fn sizes(&self) -> &[CssBackgroundSize] {
        &self.sizes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundRepeatStyle {
    Repeat,
    Space,
    Round,
    NoRepeat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundRepeat {
    RepeatX,
    RepeatY,
    Axes {
        x: CssBackgroundRepeatStyle,
        y: CssBackgroundRepeatStyle,
    },
}

/// # Specified serialization
///
/// Serializes the nonempty authored layers in comma order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBackgroundRepeatList {
    repeats: Vec<CssBackgroundRepeat>,
}

impl CssBackgroundRepeatList {
    #[must_use]
    pub fn try_new(repeats: Vec<CssBackgroundRepeat>) -> Option<Self> {
        if repeats.is_empty() {
            None
        } else {
            Some(Self::new(repeats))
        }
    }

    #[must_use]
    pub(crate) fn new(repeats: Vec<CssBackgroundRepeat>) -> Self {
        Self { repeats }
    }

    #[must_use]
    pub fn repeats(&self) -> &[CssBackgroundRepeat] {
        &self.repeats
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundBox {
    BorderBox,
    PaddingBox,
    ContentBox,
}

/// A nonempty authored comma list of background box longhand values.
///
/// # Specified serialization
///
/// Serializes the nonempty authored layers in comma order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBackgroundBoxList {
    boxes: Vec<CssBackgroundBox>,
}

impl CssBackgroundBoxList {
    /// Constructs a nonempty background box list.
    #[must_use]
    pub fn try_new(boxes: Vec<CssBackgroundBox>) -> Option<Self> {
        (!boxes.is_empty()).then_some(Self { boxes })
    }

    /// Returns the authored boxes in comma order.
    #[must_use]
    pub fn boxes(&self) -> &[CssBackgroundBox] {
        &self.boxes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundAttachment {
    Scroll,
    Fixed,
    Local,
}

/// # Specified serialization
///
/// Serializes the nonempty authored layers in comma order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBackgroundAttachmentList {
    attachments: Vec<CssBackgroundAttachment>,
}

impl CssBackgroundAttachmentList {
    #[must_use]
    pub fn try_new(attachments: Vec<CssBackgroundAttachment>) -> Option<Self> {
        if attachments.is_empty() {
            None
        } else {
            Some(Self::new(attachments))
        }
    }

    #[must_use]
    pub(crate) fn new(attachments: Vec<CssBackgroundAttachment>) -> Self {
        Self { attachments }
    }

    #[must_use]
    pub fn attachments(&self) -> &[CssBackgroundAttachment] {
        &self.attachments
    }
}
use crate::color::CssColor;
