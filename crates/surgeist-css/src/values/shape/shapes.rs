//! Checked authored basic shapes and clipping composition.

use crate::numeric::{CssLength, CssLengthPercentage, CssNonNegative, CssNumeric};
use crate::values::common::CssAutoOr;
use crate::values::image::CssRadialExtent;
use crate::values::position::CssPosition;

/// The authored radius branch of a `circle()` value.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssCircleRadius {
    /// The radius was omitted; no default is inserted.
    Default,
    /// A radial extent keyword.
    Extent(CssRadialExtent),
    /// A non-negative `<length-percentage>` radius.
    LengthPercentage(CssNumeric<CssLengthPercentage, CssNonNegative>),
}

impl PartialEq for CssCircleRadius {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Default, Self::Default) => true,
            (Self::Extent(left), Self::Extent(right)) => left == right,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => left == right,
            _ => false,
        }
    }
}

/// A checked ordered pair of independently authored ellipse radius components.
#[derive(Clone, Debug, PartialEq)]
pub struct CssEllipseRadii {
    horizontal: CssEllipseRadius,
    vertical: CssEllipseRadius,
}

impl CssEllipseRadii {
    /// Composes two independently checked nonnegative radii in horizontal/vertical order.
    pub const fn new(horizontal: CssEllipseRadius, vertical: CssEllipseRadius) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
    /// Returns the horizontal radius component.
    pub const fn horizontal(&self) -> &CssEllipseRadius {
        &self.horizontal
    }
    /// Returns the vertical radius component.
    pub const fn vertical(&self) -> &CssEllipseRadius {
        &self.vertical
    }
}

/// One independently authored nonnegative radius or radial extent of an ellipse.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssEllipseRadius {
    /// A radial extent keyword.
    Extent(CssRadialExtent),
    /// A non-negative `<length-percentage>` radius.
    LengthPercentage(CssNumeric<CssLengthPercentage, CssNonNegative>),
}

impl PartialEq for CssEllipseRadius {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Extent(left), Self::Extent(right)) => left == right,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => left == right,
            _ => false,
        }
    }
}

/// An authored `circle()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCircleShape {
    radius: CssCircleRadius,
    position: Option<CssPosition>,
}

impl CssCircleShape {
    /// Creates a circle from its radius branch and optional center position.
    #[must_use]
    pub const fn new(radius: CssCircleRadius, position: Option<CssPosition>) -> Self {
        Self { radius, position }
    }

    /// Returns the authored radius branch.
    #[must_use]
    pub const fn radius(&self) -> &CssCircleRadius {
        &self.radius
    }

    /// Returns the authored center position, or `None` when `at <position>` was omitted.
    #[must_use]
    pub const fn position(&self) -> Option<&CssPosition> {
        self.position.as_ref()
    }
}

/// An authored `ellipse()` with an omitted or complete ordered radius pair.
/// Omission does not insert an effective extent or position default.
#[derive(Clone, Debug, PartialEq)]
pub struct CssEllipseShape {
    radii: Option<CssEllipseRadii>,
    position: Option<CssPosition>,
}

impl CssEllipseShape {
    /// Creates an ellipse from an optional radius pair and an optional center position.
    #[must_use]
    pub const fn new(radii: Option<CssEllipseRadii>, position: Option<CssPosition>) -> Self {
        Self { radii, position }
    }

    #[must_use]
    /// Borrows the authored pair, preserving omission separately from explicit extents.
    pub const fn radii(&self) -> Option<&CssEllipseRadii> {
        self.radii.as_ref()
    }

    /// Returns the authored center position, or `None` when `at <position>` was omitted.
    #[must_use]
    pub const fn position(&self) -> Option<&CssPosition> {
        self.position.as_ref()
    }
}

/// One-to-four authored inset `<length-percentage>` offsets.
#[derive(Clone, Debug)]
pub struct CssInsetShapeOffsets {
    values: Vec<CssNumeric<CssLengthPercentage>>,
}

impl PartialEq for CssInsetShapeOffsets {
    fn eq(&self, other: &Self) -> bool {
        self.values.len() == other.values.len()
            && self
                .values
                .iter()
                .zip(&other.values)
                .all(|(left, right)| left == right)
    }
}

impl CssInsetShapeOffsets {
    /// Returns the offsets, or `None` unless `values` holds one to four offsets.
    #[must_use]
    pub fn try_new(values: Vec<CssNumeric<CssLengthPercentage>>) -> Option<Self> {
        (1..=4).contains(&values.len()).then_some(Self { values })
    }

    /// Returns the authored offsets, one to four, in source order.
    #[must_use]
    pub fn values(&self) -> &[CssNumeric<CssLengthPercentage>] {
        &self.values
    }
}

/// An authored `inset()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssInsetShape {
    offsets: CssInsetShapeOffsets,
    round: Option<crate::values::border::CssBorderRadiusShorthand>,
}

impl CssInsetShape {
    /// Creates an inset from its offsets and optional `round` radii.
    #[must_use]
    pub const fn new(
        offsets: CssInsetShapeOffsets,
        round: Option<crate::values::border::CssBorderRadiusShorthand>,
    ) -> Self {
        Self { offsets, round }
    }

    /// Returns the authored offsets.
    #[must_use]
    pub const fn offsets(&self) -> &CssInsetShapeOffsets {
        &self.offsets
    }

    /// Returns the authored `round` corner radii, or `None` when omitted.
    #[must_use]
    pub const fn round(&self) -> Option<&crate::values::border::CssBorderRadiusShorthand> {
        self.round.as_ref()
    }
}

/// The optional authored fill rule shared by basic shapes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFillRule {
    /// `nonzero`: the `nonzero` filling rule of the SVG `fill-rule` property, used to determine
    /// the interior.
    Nonzero,
    /// `evenodd`: the `evenodd` filling rule of the SVG `fill-rule` property, used to determine
    /// the interior.
    Evenodd,
}

/// One checked authored point in a `polygon()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPolygonPoint {
    x: CssNumeric<CssLengthPercentage>,
    y: CssNumeric<CssLengthPercentage>,
}

impl CssPolygonPoint {
    /// Composes a checked signed point without resolving its percentage bases.
    pub const fn new(
        x: CssNumeric<CssLengthPercentage>,
        y: CssNumeric<CssLengthPercentage>,
    ) -> Self {
        Self { x, y }
    }
    /// Returns the horizontal coordinate.
    pub const fn x(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.x
    }
    /// Returns the vertical coordinate.
    pub const fn y(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.y
    }
}

/// A non-empty authored polygon point list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPolygonPointList {
    points: Vec<CssPolygonPoint>,
}

impl CssPolygonPointList {
    /// Returns the list, or `None` when `points` is empty.
    #[must_use]
    pub fn try_new(points: Vec<CssPolygonPoint>) -> Option<Self> {
        (!points.is_empty()).then_some(Self { points })
    }

    /// Returns the points in authored order.
    #[must_use]
    pub fn points(&self) -> &[CssPolygonPoint] {
        &self.points
    }
}

/// An authored `polygon()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPolygonShape {
    fill_rule: Option<CssFillRule>,
    round: Option<CssNumeric<CssLength>>,
    points: CssPolygonPointList,
}

impl CssPolygonShape {
    /// Creates a polygon from an optional fill rule, an optional corner-rounding length and its points.
    #[must_use]
    pub const fn new(
        fill_rule: Option<CssFillRule>,
        round: Option<CssNumeric<CssLength>>,
        points: CssPolygonPointList,
    ) -> Self {
        Self {
            fill_rule,
            round,
            points,
        }
    }

    /// Returns the authored fill rule, or `None` when omitted.
    #[must_use]
    pub const fn fill_rule(&self) -> Option<CssFillRule> {
        self.fill_rule
    }

    /// Returns the authored corner-rounding length, or `None` when omitted.
    #[must_use]
    pub const fn round(&self) -> Option<&CssNumeric<CssLength>> {
        self.round.as_ref()
    }

    /// Returns the authored points.
    #[must_use]
    pub const fn points(&self) -> &CssPolygonPointList {
        &self.points
    }
}

/// Four independently authored rectangle edges and optional authored rounding.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRectShape {
    top: CssAutoOr<CssNumeric<CssLengthPercentage>>,
    right: CssAutoOr<CssNumeric<CssLengthPercentage>>,
    bottom: CssAutoOr<CssNumeric<CssLengthPercentage>>,
    left: CssAutoOr<CssNumeric<CssLengthPercentage>>,
    round: Option<crate::values::border::CssBorderRadiusShorthand>,
}

impl CssRectShape {
    /// Composes checked edges in top/right/bottom/left order without resolving them.
    #[must_use]
    pub const fn new(
        top: CssAutoOr<CssNumeric<CssLengthPercentage>>,
        right: CssAutoOr<CssNumeric<CssLengthPercentage>>,
        bottom: CssAutoOr<CssNumeric<CssLengthPercentage>>,
        left: CssAutoOr<CssNumeric<CssLengthPercentage>>,
        round: Option<crate::values::border::CssBorderRadiusShorthand>,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
            round,
        }
    }
    /// Returns the authored top edge, a `<length-percentage>` or `auto`.
    #[must_use]
    pub const fn top(&self) -> &CssAutoOr<CssNumeric<CssLengthPercentage>> {
        &self.top
    }
    /// Returns the authored right edge, a `<length-percentage>` or `auto`.
    #[must_use]
    pub const fn right(&self) -> &CssAutoOr<CssNumeric<CssLengthPercentage>> {
        &self.right
    }
    /// Returns the authored bottom edge, a `<length-percentage>` or `auto`.
    #[must_use]
    pub const fn bottom(&self) -> &CssAutoOr<CssNumeric<CssLengthPercentage>> {
        &self.bottom
    }
    /// Returns the authored left edge, a `<length-percentage>` or `auto`.
    #[must_use]
    pub const fn left(&self) -> &CssAutoOr<CssNumeric<CssLengthPercentage>> {
        &self.left
    }
    /// Returns the authored `round` corner radii, or `None` when omitted.
    #[must_use]
    pub const fn round(&self) -> Option<&crate::values::border::CssBorderRadiusShorthand> {
        self.round.as_ref()
    }
}

/// Authored signed coordinates, checked nonnegative dimensions, and optional rounding.
#[derive(Clone, Debug, PartialEq)]
pub struct CssXywhShape {
    x: CssNumeric<CssLengthPercentage>,
    y: CssNumeric<CssLengthPercentage>,
    width: CssNumeric<CssLengthPercentage, CssNonNegative>,
    height: CssNumeric<CssLengthPercentage, CssNonNegative>,
    round: Option<crate::values::border::CssBorderRadiusShorthand>,
}

impl CssXywhShape {
    /// Composes checked inputs without resolving coordinates or clamping dimensions.
    #[must_use]
    pub const fn new(
        x: CssNumeric<CssLengthPercentage>,
        y: CssNumeric<CssLengthPercentage>,
        width: CssNumeric<CssLengthPercentage, CssNonNegative>,
        height: CssNumeric<CssLengthPercentage, CssNonNegative>,
        round: Option<crate::values::border::CssBorderRadiusShorthand>,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            round,
        }
    }
    /// Returns the authored horizontal coordinate of the top-left corner.
    #[must_use]
    pub const fn x(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.x
    }
    /// Returns the authored vertical coordinate of the top-left corner.
    #[must_use]
    pub const fn y(&self) -> &CssNumeric<CssLengthPercentage> {
        &self.y
    }
    /// Returns the authored non-negative width.
    #[must_use]
    pub const fn width(&self) -> &CssNumeric<CssLengthPercentage, CssNonNegative> {
        &self.width
    }
    /// Returns the authored non-negative height.
    #[must_use]
    pub const fn height(&self) -> &CssNumeric<CssLengthPercentage, CssNonNegative> {
        &self.height
    }
    /// Returns the authored `round` corner radii, or `None` when omitted.
    #[must_use]
    pub const fn round(&self) -> Option<&crate::values::border::CssBorderRadiusShorthand> {
        self.round.as_ref()
    }
}

/// An authored `path()` with checked decoded SVG data and optional fill.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssPathShape {
    fill_rule: Option<CssFillRule>,
    data: super::CssPathData,
}

impl CssPathShape {
    /// Composes checked path data without inserting a contextual fill default.
    #[must_use]
    pub const fn new(fill_rule: Option<CssFillRule>, data: super::CssPathData) -> Self {
        Self { fill_rule, data }
    }

    /// Returns the authored fill rule, preserving omission.
    #[must_use]
    pub const fn fill_rule(&self) -> Option<CssFillRule> {
        self.fill_rule
    }

    /// Returns the checked decoded SVG data.
    #[must_use]
    pub const fn data(&self) -> &super::CssPathData {
        &self.data
    }
}

/// A selected authored basic-shape function.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBasicShape {
    /// A `shape()` function.
    Shape(CssShapeFunction),
    /// An `inset()` function, boxed to keep the enum small.
    Inset(Box<CssInsetShape>),
    /// A `circle()` function.
    Circle(CssCircleShape),
    /// An `ellipse()` function.
    Ellipse(CssEllipseShape),
    /// A `polygon()` function.
    Polygon(CssPolygonShape),
    /// A `path()` function.
    Path(CssPathShape),
    /// A `rect()` function.
    Rect(CssRectShape),
    /// An `xywh()` function.
    Xywh(CssXywhShape),
}

/// A Shapes 1 reference box: the Box 4 visual boxes plus `margin-box`.
/// SVG geometry boxes are outside this authored grammar.
///
/// # Specified serialization
///
/// Serializes authored children without selecting geometry or loading images.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssShapeBox {
    /// `content-box`: the content box edge.
    ContentBox,
    /// `padding-box`: the padding box edge.
    PaddingBox,
    /// `border-box`: the border box edge.
    BorderBox,
    /// `margin-box`: the margin box edge.
    MarginBox,
}

impl CssShapeBox {
    /// Admits one decoded shape-box keyword without selecting used geometry.
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Some(match keyword.to_ascii_lowercase().as_str() {
            "content-box" => Self::ContentBox,
            "padding-box" => Self::PaddingBox,
            "border-box" => Self::BorderBox,
            "margin-box" => Self::MarginBox,
            _ => return None,
        })
    }

    /// Returns the canonical authored box keyword.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContentBox => "content-box",
            Self::PaddingBox => "padding-box",
            Self::BorderBox => "border-box",
            Self::MarginBox => "margin-box",
        }
    }
}

/// An authored `shape-outside` value before float geometry or resource loading.
/// The checked image branch excludes bare `none` but retains symbolic image/none
/// children of `light-dark()`.
///
/// # Specified serialization
///
/// Serializes authored children without selecting geometry or loading images.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssShapeOutside {
    /// `none`: the float area is unaffected.
    None,
    /// A bare `<shape-box>`, whose box, including curvature from `border-radius`, defines the
    /// shape.
    ShapeBox(CssShapeBox),
    /// A `<basic-shape>` with an optional reference box.
    BasicShape(CssShapeOutsideShape),
    /// An `<image>` whose alpha channel defines the shape.
    Image(crate::values::image::CssImage),
}

/// A basic shape with an optional explicit Shapes reference box.
/// Omission stays distinct from explicit `margin-box`; the default is downstream.
///
/// # Specified serialization
///
/// Serializes authored children without selecting geometry or loading images.
#[derive(Clone, Debug, PartialEq)]
pub struct CssShapeOutsideShape {
    shape: CssBasicShape,
    reference_box: Option<CssShapeBox>,
}

impl CssShapeOutsideShape {
    /// Composes checked children without inferring an omitted reference box.
    #[must_use]
    pub const fn new(shape: CssBasicShape, reference_box: Option<CssShapeBox>) -> Self {
        Self {
            shape,
            reference_box,
        }
    }

    /// Borrows the authored basic-shape function.
    #[must_use]
    pub const fn shape(&self) -> &CssBasicShape {
        &self.shape
    }

    /// Returns the explicit box, preserving authored omission.
    #[must_use]
    pub const fn reference_box(&self) -> Option<CssShapeBox> {
        self.reference_box
    }
}

/// An authored shape function with full initial position and nonempty commands.
#[derive(Clone, Debug, PartialEq)]
pub struct CssShapeFunction {
    fill_rule: Option<super::CssFillRule>,
    start: CssPosition,
    commands: super::CssShapeCommandList,
}
impl CssShapeFunction {
    /// Creates a shape function from an optional fill rule, a starting position and its commands.
    pub const fn new(
        fill_rule: Option<super::CssFillRule>,
        start: CssPosition,
        commands: super::CssShapeCommandList,
    ) -> Self {
        Self {
            fill_rule,
            start,
            commands,
        }
    }
    /// Returns the authored fill rule, or `None` when omitted.
    pub const fn fill_rule(&self) -> Option<super::CssFillRule> {
        self.fill_rule
    }
    /// Returns the authored starting position.
    pub const fn start(&self) -> &CssPosition {
        &self.start
    }
    /// Returns the commands, which are never empty.
    pub const fn commands(&self) -> &super::CssShapeCommandList {
        &self.commands
    }
}
