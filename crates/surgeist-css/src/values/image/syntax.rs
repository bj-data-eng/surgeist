use crate::numeric::CssAngleOrZero;
use crate::numeric::{CssAngle, CssLength, CssLengthPercentage, CssNonNegative, CssNumeric};
use crate::values::common::CssUrl;
use crate::values::image::images;

/// An authored image value accepted by the shared Image consumers.
///
/// # Specified serialization
///
/// Serializes an authored image value, including the property-level `none` branch.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssImageValue {
    /// `none`: no image.
    None,
    /// A `url()` or `src()` image reference.
    Url(CssUrl),
    /// A gradient function.
    Gradient(CssGradient),
    /// Two authored image/none branches, retaining their order and scheme dependency.
    LightDark(Box<CssLightDarkImage>),
    /// A distinct authored String or checked Image input and nonempty filter list.
    Filter(Box<CssFilterImage>),
}

/// The authored Images 3 `image-orientation` syntax.
///
/// # Specified serialization
///
/// Emits canonical specified syntax, with an authored angle before `flip`. Omitted angles stay
/// omitted; explicit keywords and zero angles remain explicit. This does not perform computed-value
/// rotation normalization.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssImageOrientation {
    /// `from-image`: computes to the angle that the image's own metadata, such as EXIF, specifies
    /// is necessary to orient it correctly, or to `none` when it specifies no orientation.
    FromImage,
    /// `none`: no additional rotation is applied; the image is oriented as encoded.
    None,
    /// An `<angle>` rotation; positive angles rotate clockwise.
    Angle(CssNumeric<CssAngle>),
    /// The `flip` keyword, which flips the image horizontally after rotation, with the `<angle>`
    /// that was authored with it; `None` when no angle was authored.
    Flip(Option<CssNumeric<CssAngle>>),
}

impl PartialEq for CssImageOrientation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::FromImage, Self::FromImage) => true,
            (Self::None, Self::None) => true,
            (Self::Angle(left), Self::Angle(right)) => left == right,
            (Self::Flip(left), Self::Flip(right)) => left == right,
            _ => false,
        }
    }
}

/// The authored Images 3 `image-rendering` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssImageRendering {
    /// `auto`: the user agent picks the scaling algorithm.
    Auto,
    /// `smooth`: the image is scaled with an algorithm that maximizes its appearance, such as bilinear interpolation.
    Smooth,
    /// `high-quality`: like `smooth`, with a preference for higher-quality scaling when resources are constrained.
    HighQuality,
    /// `crisp-edges`: the image is scaled so that contrast and edges are preserved and no smoothing or blurring is applied.
    CrispEdges,
    /// `pixelated`: the image is scaled in a way that preserves its pixelation as much as
    /// possible, but allows minor smoothing when the target size is not a clean multiple of the
    /// original.
    Pixelated,
    /// Deprecated CSS-standard syntax accepted by user agents; authors must not use it.
    /// Its authored identity stays distinct from the downstream crisp-edges behavior.
    OptimizeSpeed,
    /// Deprecated CSS-standard syntax accepted by user agents; authors must not use it.
    /// Its authored identity stays distinct from the downstream smooth behavior.
    OptimizeQuality,
}

/// The authored Images 3 `object-fit` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssObjectFit {
    /// `fill`: the replaced content is sized to fill the element's content box.
    Fill,
    /// `contain`: the replaced content is sized to maintain its natural aspect ratio while
    /// fitting within the content box.
    Contain,
    /// `cover`: the replaced content is sized to maintain its natural aspect ratio while filling
    /// the entire content box.
    Cover,
    /// `none`: the replaced content is not resized to fit inside the content box.
    None,
    /// `scale-down`: sized as if `none` or `contain` were specified, whichever gives the smaller
    /// concrete object size.
    ScaleDown,
}

/// A nonempty authored comma-separated image list.
///
/// # Specified serialization
///
/// Serializes every image in its authored order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssImageValueList {
    images: Vec<CssImageValue>,
}

impl CssImageValueList {
    /// Returns the list, or `None` when `images` is empty.
    #[must_use]
    pub fn try_new(images: Vec<CssImageValue>) -> Option<Self> {
        (!images.is_empty()).then_some(Self { images })
    }

    /// Returns the authored images in comma order.
    #[must_use]
    pub fn images(&self) -> &[CssImageValue] {
        &self.images
    }
}

/// One of the selected authored Images 3/4 gradient functions.
///
/// # Specified serialization
///
/// Serializes the checked gradient with its authored function identity.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssGradient {
    /// A `linear-gradient()`.
    Linear(CssLinearGradient),
    /// A `radial-gradient()`.
    Radial(CssRadialGradient),
    /// A `repeating-linear-gradient()`.
    RepeatingLinear(CssLinearGradient),
    /// A `repeating-radial-gradient()`.
    RepeatingRadial(CssRadialGradient),
    /// A `conic-gradient()`.
    Conic(CssConicGradient),
}

/// The horizontal side keyword of a linear-gradient side or corner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHorizontalGradientSide {
    /// `left`: the left side.
    Left,
    /// `right`: the right side.
    Right,
}

/// The vertical side keyword of a linear-gradient side or corner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVerticalGradientSide {
    /// `top`: the top side.
    Top,
    /// `bottom`: the bottom side.
    Bottom,
}

/// A checked authored linear-gradient side or corner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssSideOrCorner {
    horizontal: Option<CssHorizontalGradientSide>,
    vertical: Option<CssVerticalGradientSide>,
}

impl CssSideOrCorner {
    /// Returns a side or corner, or `None` when both `horizontal` and `vertical` are omitted.
    /// One component names a side and two name a corner.
    #[must_use]
    pub const fn try_new(
        horizontal: Option<CssHorizontalGradientSide>,
        vertical: Option<CssVerticalGradientSide>,
    ) -> Option<Self> {
        if horizontal.is_none() && vertical.is_none() {
            None
        } else {
            Some(Self {
                horizontal,
                vertical,
            })
        }
    }

    /// Returns the horizontal side, or `None` when only a vertical side was authored.
    #[must_use]
    pub const fn horizontal(self) -> Option<CssHorizontalGradientSide> {
        self.horizontal
    }

    /// Returns the vertical side, or `None` when only a horizontal side was authored.
    #[must_use]
    pub const fn vertical(self) -> Option<CssVerticalGradientSide> {
        self.vertical
    }
}

/// The authored direction of a linear gradient.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssLinearGradientDirection {
    /// An `<angle>` or bare zero.
    Angle(CssAngleOrZero),
    /// A `to <side-or-corner>` direction.
    SideOrCorner(CssSideOrCorner),
}

impl PartialEq for CssLinearGradientDirection {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Angle(left), Self::Angle(right)) => left == right,
            (Self::SideOrCorner(left), Self::SideOrCorner(right)) => left == right,
            _ => false,
        }
    }
}

/// One authored color stop in a gradient color-stop list.
#[derive(Clone, Debug)]
pub struct CssGradientColorStop {
    pub(crate) color: CssColor,
    pub(crate) positions: Vec<CssNumeric<CssLengthPercentage>>,
}
impl PartialEq for CssGradientColorStop {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
            && self.positions.len() == other.positions.len()
            && self
                .positions
                .iter()
                .zip(&other.positions)
                .all(|(a, b)| a == b)
    }
}

impl CssGradientColorStop {
    /// Returns the authored stop color.
    #[must_use]
    pub const fn color(&self) -> &CssColor {
        &self.color
    }

    /// Returns the first authored position, or `None` when the stop has no position.
    #[must_use]
    pub fn position(&self) -> Option<&CssNumeric<CssLengthPercentage>> {
        self.positions.first()
    }

    /// Borrows zero, one, or two authored positions without expanding the stop.
    pub fn positions(&self) -> &[CssNumeric<CssLengthPercentage>] {
        &self.positions
    }
}

/// One authored item in a color-stop list, preserving stop and hint order.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssColorStopListItem {
    /// A color stop, boxed to keep the enum small.
    Stop(Box<CssGradientColorStop>),
    /// A `<length-percentage>` interpolation hint between two stops.
    Hint(CssNumeric<CssLengthPercentage>),
}

impl PartialEq for CssColorStopListItem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Stop(left), Self::Stop(right)) => left == right,
            (Self::Hint(left), Self::Hint(right)) => left == right,
            _ => false,
        }
    }
}

/// A checked authored color-stop list with one or more stops and interleaved hints.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColorStopList {
    items: Vec<CssColorStopListItem>,
}

impl CssColorStopList {
    /// Returns the list, or `None` unless it starts and ends with a stop and has no two adjacent hints.
    #[must_use]
    pub fn try_new(items: Vec<CssColorStopListItem>) -> Option<Self> {
        let ordered = matches!(items.first(), Some(CssColorStopListItem::Stop(_)))
            && matches!(items.last(), Some(CssColorStopListItem::Stop(_)))
            && items.windows(2).all(|pair| {
                !matches!(
                    pair,
                    [CssColorStopListItem::Hint(_), CssColorStopListItem::Hint(_)]
                )
            });
        ordered.then_some(Self { items })
    }

    /// Returns the authored stops and hints in order.
    #[must_use]
    pub fn items(&self) -> &[CssColorStopListItem] {
        &self.items
    }
}

/// An authored linear or repeating-linear gradient payload.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLinearGradient {
    pub(crate) direction: Option<CssLinearGradientDirection>,
    pub(crate) interpolation: Option<CssColorInterpolation>,
    pub(crate) stops: CssColorStopList,
}

impl CssLinearGradient {
    /// Borrows the explicitly authored interpolation method; omission stays omitted.
    pub const fn interpolation(&self) -> Option<&CssColorInterpolation> {
        self.interpolation.as_ref()
    }

    /// Selects an intrinsically checked interpolation method, preserving direction and stops.
    pub fn with_interpolation(mut self, interpolation: Option<CssColorInterpolation>) -> Self {
        self.interpolation = interpolation;
        self
    }
    /// Returns the authored direction, or `None` when it was omitted.
    #[must_use]
    pub const fn direction(&self) -> Option<&CssLinearGradientDirection> {
        self.direction.as_ref()
    }

    /// Returns the authored color-stop list.
    #[must_use]
    pub const fn stops(&self) -> &CssColorStopList {
        &self.stops
    }
}

/// The authored ending shape of a radial gradient.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRadialShape {
    /// `circle`: the ending shape is a circle.
    Circle,
    /// `ellipse`: the ending shape is an ellipse.
    Ellipse,
}

/// A checked pair of non-negative radial-gradient ellipse radii.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRadialEllipseSize {
    horizontal: CssNumeric<CssLengthPercentage, CssNonNegative>,
    vertical: CssNumeric<CssLengthPercentage, CssNonNegative>,
}

impl CssRadialEllipseSize {
    /// Composes two independently checked nonnegative radii in horizontal/vertical order.
    pub const fn new(
        horizontal: CssNumeric<CssLengthPercentage, CssNonNegative>,
        vertical: CssNumeric<CssLengthPercentage, CssNonNegative>,
    ) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
    /// Returns the horizontal radius.
    pub const fn horizontal(&self) -> &CssNumeric<CssLengthPercentage, CssNonNegative> {
        &self.horizontal
    }
    /// Returns the vertical radius.
    pub const fn vertical(&self) -> &CssNumeric<CssLengthPercentage, CssNonNegative> {
        &self.vertical
    }
}

/// The authored size of a radial gradient's ending shape.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssRadialSize {
    /// A radial extent keyword.
    Extent(CssRadialExtent),
    /// A single non-negative `<length>` radius, sizing a circle.
    Circle(CssNumeric<CssLength, CssNonNegative>),
    /// A pair of non-negative `<length-percentage>` radii, sizing an ellipse.
    Ellipse(CssRadialEllipseSize),
}

impl PartialEq for CssRadialSize {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Extent(left), Self::Extent(right)) => left == right,
            (Self::Circle(left), Self::Circle(right)) => left == right,
            (Self::Ellipse(left), Self::Ellipse(right)) => left == right,
            _ => false,
        }
    }
}

/// A current authored radial or repeating-radial gradient payload.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRadialGradient {
    pub(crate) shape: Option<CssRadialShape>,
    pub(crate) size: Option<CssRadialSize>,
    pub(crate) position: Option<CssPosition>,
    pub(crate) interpolation: Option<CssColorInterpolation>,
    pub(crate) stops: CssColorStopList,
}

impl CssRadialGradient {
    /// Borrows the explicitly authored interpolation method; omission stays omitted.
    pub const fn interpolation(&self) -> Option<&CssColorInterpolation> {
        self.interpolation.as_ref()
    }

    /// Selects an intrinsically checked interpolation method, preserving the radial prelude.
    pub fn with_interpolation(mut self, interpolation: Option<CssColorInterpolation>) -> Self {
        self.interpolation = interpolation;
        self
    }
    /// Returns the authored ending shape, or `None` when it was omitted.
    #[must_use]
    pub const fn shape(&self) -> Option<CssRadialShape> {
        self.shape
    }

    /// Returns the authored ending-shape size, or `None` when it was omitted.
    #[must_use]
    pub const fn size(&self) -> Option<&CssRadialSize> {
        self.size.as_ref()
    }

    /// Returns the authored center position, or `None` when it was omitted.
    #[must_use]
    pub const fn position(&self) -> Option<&CssPosition> {
        self.position.as_ref()
    }

    /// Returns the authored color-stop list.
    #[must_use]
    pub const fn stops(&self) -> &CssColorStopList {
        &self.stops
    }
}

/// An authored conic stop/hint position, with an unresolved angular percentage basis.
///
/// # Specified serialization
///
/// Serializes a checked angular position without resolving its percentage basis.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssAngularColorStopPosition {
    /// An `<angle>` position.
    Angle(CssNumeric<CssAngle>),
    /// A `<percentage>` position.
    Percentage(crate::numeric::CssNumeric<crate::numeric::CssPercentage>),
    /// A bare zero position.
    Zero(crate::numeric::CssZeroLiteral),
    /// A `calc()` expression resolving to an `<angle-percentage>`.
    Calculation(crate::numeric::CssCalculation<crate::numeric::CssAnglePercentage>),
}
impl PartialEq for CssAngularColorStopPosition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Angle(a), Self::Angle(b)) => a == b,
            (Self::Percentage(a), Self::Percentage(b)) => a == b,
            (Self::Zero(a), Self::Zero(b)) => a == b,
            (Self::Calculation(a), Self::Calculation(b)) => a == b,
            _ => false,
        }
    }
}

/// One authored angular color stop with zero, one, or two positions.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAngularColorStop {
    color: CssColor,
    positions: Vec<CssAngularColorStopPosition>,
}
impl CssAngularColorStop {
    /// Borrows the unresolved authored color.
    pub const fn color(&self) -> &CssColor {
        &self.color
    }
    /// Borrows zero, one, or two positions in their authored order.
    pub fn positions(&self) -> &[CssAngularColorStopPosition] {
        &self.positions
    }
    /// Checks authored cardinality without running stop fixup or interpolation.
    pub fn try_new(
        color: CssColor,
        positions: Vec<CssAngularColorStopPosition>,
    ) -> Result<Self, CssImageConstructionError> {
        if positions.len() > 2 {
            return Err(CssImageConstructionError::new(
                images::ImageConstructionKind::TooManyPositions,
            ));
        }
        Ok(Self { color, positions })
    }
}

/// One conic stop or intervening angular hint, in authored order.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAngularColorStopListItem {
    /// An angular color stop, boxed to keep the enum small.
    Stop(Box<CssAngularColorStop>),
    /// An angular interpolation hint position.
    Hint(CssAngularColorStopPosition),
}

/// A nonempty angular stop list with no endpoint or consecutive hints.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAngularColorStopList {
    items: Vec<CssAngularColorStopListItem>,
}
impl CssAngularColorStopList {
    /// Rejects an empty list and endpoint or adjacent interpolation hints.
    pub fn try_new(items: Vec<CssAngularColorStopListItem>) -> Option<Self> {
        let ordered = matches!(items.first(), Some(CssAngularColorStopListItem::Stop(_)))
            && matches!(items.last(), Some(CssAngularColorStopListItem::Stop(_)))
            && items.windows(2).all(|pair| {
                !matches!(
                    pair,
                    [
                        CssAngularColorStopListItem::Hint(_),
                        CssAngularColorStopListItem::Hint(_)
                    ]
                )
            });
        ordered.then_some(Self { items })
    }
    /// Borrows the authored stop/hint sequence.
    pub fn items(&self) -> &[CssAngularColorStopListItem] {
        &self.items
    }
}

/// Authored conic grammar; context-dependent geometry and color execution are deferred.
#[derive(Clone, Debug, PartialEq)]
pub struct CssConicGradient {
    from: Option<CssAngleOrZero>,
    position: Option<CssPosition>,
    interpolation: Option<CssColorInterpolation>,
    stops: CssAngularColorStopList,
}
impl CssConicGradient {
    /// Composes checked children, preserving optional authored prelude fields.
    /// `CssImage::try_new` checks the complete retained function-depth envelope.
    pub const fn new(
        from: Option<CssAngleOrZero>,
        position: Option<CssPosition>,
        interpolation: Option<CssColorInterpolation>,
        stops: CssAngularColorStopList,
    ) -> Self {
        Self {
            from,
            position,
            interpolation,
            stops,
        }
    }
    /// Borrows the explicit starting angle or exact bare zero, when authored.
    pub const fn from(&self) -> Option<&CssAngleOrZero> {
        self.from.as_ref()
    }
    /// Borrows the full symbolic position, when authored.
    pub const fn position(&self) -> Option<&CssPosition> {
        self.position.as_ref()
    }
    /// Borrows the explicit Color 4/5 interpolation method, preserving omission.
    pub const fn interpolation(&self) -> Option<&CssColorInterpolation> {
        self.interpolation.as_ref()
    }
    /// Borrows the nonempty authored angular stop list.
    pub const fn stops(&self) -> &CssAngularColorStopList {
        &self.stops
    }
}
use crate::values::effects::CssFilterImage;
use crate::values::image::{CssImageConstructionError, CssLightDarkImage};
use crate::values::position::CssPosition;

/// An authored radial extent keyword shared by `circle()` and `ellipse()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRadialExtent {
    /// `closest-side`: the ending shape meets the side of the box closest to its center.
    ClosestSide,
    /// `farthest-side`: the ending shape meets the side of the box farthest from its center.
    FarthestSide,
    /// `closest-corner`: the ending shape passes through the corner of the box closest to its center.
    ClosestCorner,
    /// `farthest-corner`: the ending shape passes through the corner of the box farthest from its center.
    FarthestCorner,
}
use crate::color::{CssColor, CssColorInterpolation};
