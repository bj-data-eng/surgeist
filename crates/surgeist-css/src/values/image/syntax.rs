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
    None,
    Url(CssUrl),
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
    FromImage,
    None,
    Angle(CssNumeric<CssAngle>),
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
    Auto,
    Smooth,
    HighQuality,
    CrispEdges,
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
    Fill,
    Contain,
    Cover,
    None,
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
    #[must_use]
    pub fn try_new(images: Vec<CssImageValue>) -> Option<Self> {
        (!images.is_empty()).then_some(Self { images })
    }

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
    Linear(CssLinearGradient),
    Radial(CssRadialGradient),
    RepeatingLinear(CssLinearGradient),
    RepeatingRadial(CssRadialGradient),
    Conic(CssConicGradient),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHorizontalGradientSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssVerticalGradientSide {
    Top,
    Bottom,
}

/// A checked authored linear-gradient side or corner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssSideOrCorner {
    horizontal: Option<CssHorizontalGradientSide>,
    vertical: Option<CssVerticalGradientSide>,
}

impl CssSideOrCorner {
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

    #[must_use]
    pub const fn horizontal(self) -> Option<CssHorizontalGradientSide> {
        self.horizontal
    }

    #[must_use]
    pub const fn vertical(self) -> Option<CssVerticalGradientSide> {
        self.vertical
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssLinearGradientDirection {
    Angle(CssAngleOrZero),
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
    #[must_use]
    pub const fn color(&self) -> &CssColor {
        &self.color
    }

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
    Stop(Box<CssGradientColorStop>),
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
    #[must_use]
    pub const fn direction(&self) -> Option<&CssLinearGradientDirection> {
        self.direction.as_ref()
    }

    #[must_use]
    pub const fn stops(&self) -> &CssColorStopList {
        &self.stops
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRadialShape {
    Circle,
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
    pub const fn horizontal(&self) -> &CssNumeric<CssLengthPercentage, CssNonNegative> {
        &self.horizontal
    }
    pub const fn vertical(&self) -> &CssNumeric<CssLengthPercentage, CssNonNegative> {
        &self.vertical
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssRadialSize {
    Extent(CssRadialExtent),
    Circle(CssNumeric<CssLength, CssNonNegative>),
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
    #[must_use]
    pub const fn shape(&self) -> Option<CssRadialShape> {
        self.shape
    }

    #[must_use]
    pub const fn size(&self) -> Option<&CssRadialSize> {
        self.size.as_ref()
    }

    #[must_use]
    pub const fn position(&self) -> Option<&CssPosition> {
        self.position.as_ref()
    }

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
    Angle(CssNumeric<CssAngle>),
    Percentage(crate::numeric::CssNumeric<crate::numeric::CssPercentage>),
    Zero(crate::numeric::CssZeroLiteral),
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
    Stop(Box<CssAngularColorStop>),
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
    ClosestSide,
    FarthestSide,
    ClosestCorner,
    FarthestCorner,
}
use crate::color::{CssColor, CssColorInterpolation};
