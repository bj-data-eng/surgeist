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
    /// `normal`: no blending; the blending function selects the source color.
    Normal,
    /// `darken`: selects the darker of the backdrop and source colors.
    Darken,
    /// `multiply`: multiplies the source color by the backdrop color; the result is at least as
    /// dark as either.
    Multiply,
    /// `color-burn`: darkens the backdrop to reflect the source color.
    ColorBurn,
    /// `lighten`: selects the lighter of the backdrop and source colors.
    Lighten,
    /// `screen`: multiplies the complements of the backdrop and source colors, then complements
    /// the result; the result is at least as light as either.
    Screen,
    /// `color-dodge`: brightens the backdrop to reflect the source color.
    ColorDodge,
    /// `overlay`: multiplies or screens depending on the backdrop color, preserving its highlights and shadows.
    Overlay,
    /// `soft-light`: darkens or lightens depending on the source color, like a diffused spotlight.
    SoftLight,
    /// `hard-light`: multiplies or screens depending on the source color, like a harsh spotlight.
    HardLight,
    /// `difference`: subtracts the darker of the colors from the lighter.
    Difference,
    /// `exclusion`: like `difference` with lower contrast.
    Exclusion,
    /// `hue`: takes the hue of the source with the saturation and luminosity of the backdrop.
    Hue,
    /// `saturation`: takes the saturation of the source with the hue and luminosity of the backdrop.
    Saturation,
    /// `color`: takes the hue and saturation of the source with the luminosity of the backdrop.
    Color,
    /// `luminosity`: takes the luminosity of the source with the hue and saturation of the backdrop.
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
    /// Returns the list, or `None` when `modes` is empty.
    #[must_use]
    pub fn try_new(modes: Vec<CssBlendMode>) -> Option<Self> {
        (!modes.is_empty()).then_some(Self { modes })
    }

    /// Returns the authored blend modes in comma order, including duplicates.
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
    /// A single `<visual-box>` that supplies both `background-origin` and `background-clip`.
    One(CssBackgroundBox),
    /// Two `<visual-box>` values, the first for `background-origin` and the second for `background-clip`.
    OriginAndClip {
        /// The authored `background-origin` box.
        origin: CssBackgroundBox,
        /// The authored `background-clip` box.
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

/// An authored `<bg-size>` for one layer.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundSize {
    /// `cover`: scales the image to the smallest size that covers the positioning area.
    Cover,
    /// `contain`: scales the image to the largest size that fits inside the positioning area.
    Contain,
    /// One or two explicit sizes, each a non-negative `<length-percentage>` or `auto`.
    Explicit {
        /// The authored width.
        width: CssAutoOr<CssNumeric<CssLengthPercentage, CssNonNegative>>,
        /// The authored height, or `None` when only one value was given.
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
    /// Returns the list, or `None` when `sizes` is empty.
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

    /// Returns the authored sizes in comma order.
    #[must_use]
    pub fn sizes(&self) -> &[CssBackgroundSize] {
        &self.sizes
    }
}

/// One axis of an authored `<repeat-style>`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundRepeatStyle {
    /// `repeat`: the image is tiled along the axis until the area is covered.
    Repeat,
    /// `space`: the image is tiled without clipping, with the leftover space distributed between tiles.
    Space,
    /// `round`: the image is tiled and rescaled so a whole number of tiles fits.
    Round,
    /// `no-repeat`: the image is not tiled along the axis.
    NoRepeat,
}

/// An authored `<repeat-style>` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundRepeat {
    /// `repeat-x`: shorthand for `repeat no-repeat`.
    RepeatX,
    /// `repeat-y`: shorthand for `no-repeat repeat`.
    RepeatY,
    /// Two axis styles, horizontal then vertical, as authored.
    Axes {
        /// The horizontal repeat style.
        x: CssBackgroundRepeatStyle,
        /// The vertical repeat style.
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
    /// Returns the list, or `None` when `repeats` is empty.
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

    /// Returns the authored repeat styles in comma order.
    #[must_use]
    pub fn repeats(&self) -> &[CssBackgroundRepeat] {
        &self.repeats
    }
}

/// An authored `<visual-box>` value used by `background-origin` and `background-clip`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundBox {
    /// `border-box`: the border box.
    BorderBox,
    /// `padding-box`: the padding box.
    PaddingBox,
    /// `content-box`: the content box.
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

/// An authored `<attachment>` value for `background-attachment`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBackgroundAttachment {
    /// `scroll`: the background is fixed relative to the element itself and does not scroll with its contents.
    Scroll,
    /// `fixed`: the background is fixed relative to the viewport.
    Fixed,
    /// `local`: the background is fixed relative to the element's contents and scrolls with them.
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
    /// Returns the list, or `None` when `attachments` is empty.
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

    /// Returns the authored attachments in comma order.
    #[must_use]
    pub fn attachments(&self) -> &[CssBackgroundAttachment] {
        &self.attachments
    }
}
use crate::color::CssColor;
