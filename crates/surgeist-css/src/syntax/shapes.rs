//! Checked authored basic shapes and clipping composition.

use super::{
    CssBoxEdgeKeyword, CssPosition, CssRadialExtent, CssSpecifiedLength,
    CssSpecifiedLengthPercentage, CssSpecifiedNonNegativeLengthPercentage, CssUrl,
    optional_numeric_eq,
};

/// The authored radius branch of a `circle()` value.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssCircleRadius {
    Default,
    Extent(CssRadialExtent),
    LengthPercentage(CssSpecifiedNonNegativeLengthPercentage),
}

impl PartialEq for CssCircleRadius {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Default, Self::Default) => true,
            (Self::Extent(left), Self::Extent(right)) => left == right,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => {
                left.structural_eq(right)
            }
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
    pub const fn horizontal(&self) -> &CssEllipseRadius {
        &self.horizontal
    }
    pub const fn vertical(&self) -> &CssEllipseRadius {
        &self.vertical
    }
}

/// One independently authored nonnegative radius or radial extent of an ellipse.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssEllipseRadius {
    Extent(CssRadialExtent),
    LengthPercentage(CssSpecifiedNonNegativeLengthPercentage),
}

impl PartialEq for CssEllipseRadius {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Extent(left), Self::Extent(right)) => left == right,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => {
                left.structural_eq(right)
            }
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
    #[must_use]
    pub const fn new(radius: CssCircleRadius, position: Option<CssPosition>) -> Self {
        Self { radius, position }
    }

    #[must_use]
    pub const fn radius(&self) -> &CssCircleRadius {
        &self.radius
    }

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
    #[must_use]
    pub const fn new(radii: Option<CssEllipseRadii>, position: Option<CssPosition>) -> Self {
        Self { radii, position }
    }

    #[must_use]
    /// Borrows the authored pair, preserving omission separately from explicit extents.
    pub const fn radii(&self) -> Option<&CssEllipseRadii> {
        self.radii.as_ref()
    }

    #[must_use]
    pub const fn position(&self) -> Option<&CssPosition> {
        self.position.as_ref()
    }
}

/// One-to-four authored inset `<length-percentage>` offsets.
#[derive(Clone, Debug)]
pub struct CssInsetShapeOffsets {
    values: Vec<CssSpecifiedLengthPercentage>,
}

impl PartialEq for CssInsetShapeOffsets {
    fn eq(&self, other: &Self) -> bool {
        self.values.len() == other.values.len()
            && self
                .values
                .iter()
                .zip(&other.values)
                .all(|(left, right)| left.structural_eq(right))
    }
}

impl CssInsetShapeOffsets {
    #[must_use]
    pub fn try_new(values: Vec<CssSpecifiedLengthPercentage>) -> Option<Self> {
        (1..=4).contains(&values.len()).then_some(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[CssSpecifiedLengthPercentage] {
        &self.values
    }
}

/// An authored `inset()` value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssInsetShape {
    offsets: CssInsetShapeOffsets,
    round: Option<crate::CssBorderRadiusShorthand>,
}

impl CssInsetShape {
    #[must_use]
    pub const fn new(
        offsets: CssInsetShapeOffsets,
        round: Option<crate::CssBorderRadiusShorthand>,
    ) -> Self {
        Self { offsets, round }
    }

    #[must_use]
    pub const fn offsets(&self) -> &CssInsetShapeOffsets {
        &self.offsets
    }

    #[must_use]
    pub const fn round(&self) -> Option<&crate::CssBorderRadiusShorthand> {
        self.round.as_ref()
    }
}

/// The optional authored fill rule of `polygon()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPolygonFillRule {
    Nonzero,
    Evenodd,
}

/// One checked authored point in a `polygon()` value.
#[derive(Clone, Debug)]
pub struct CssPolygonPoint {
    x: CssSpecifiedLengthPercentage,
    y: CssSpecifiedLengthPercentage,
}
numeric_fields_eq!(CssPolygonPoint, [x, y], [], []);

impl CssPolygonPoint {
    /// Composes a checked signed point without resolving its percentage bases.
    pub const fn new(x: CssSpecifiedLengthPercentage, y: CssSpecifiedLengthPercentage) -> Self {
        Self { x, y }
    }
    pub const fn x(&self) -> &CssSpecifiedLengthPercentage {
        &self.x
    }
    pub const fn y(&self) -> &CssSpecifiedLengthPercentage {
        &self.y
    }
}

/// A non-empty authored polygon point list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPolygonPointList {
    points: Vec<CssPolygonPoint>,
}

impl CssPolygonPointList {
    #[must_use]
    pub fn try_new(points: Vec<CssPolygonPoint>) -> Option<Self> {
        (!points.is_empty()).then_some(Self { points })
    }

    #[must_use]
    pub fn points(&self) -> &[CssPolygonPoint] {
        &self.points
    }
}

/// An authored `polygon()` value.
#[derive(Clone, Debug)]
pub struct CssPolygonShape {
    fill_rule: Option<CssPolygonFillRule>,
    round: Option<CssSpecifiedLength>,
    points: CssPolygonPointList,
}
numeric_fields_eq!(CssPolygonShape, [], [round], [fill_rule, points]);

impl CssPolygonShape {
    #[must_use]
    pub const fn new(
        fill_rule: Option<CssPolygonFillRule>,
        round: Option<CssSpecifiedLength>,
        points: CssPolygonPointList,
    ) -> Self {
        Self {
            fill_rule,
            round,
            points,
        }
    }

    #[must_use]
    pub const fn fill_rule(&self) -> Option<CssPolygonFillRule> {
        self.fill_rule
    }

    #[must_use]
    pub const fn round(&self) -> Option<&CssSpecifiedLength> {
        self.round.as_ref()
    }

    #[must_use]
    pub const fn points(&self) -> &CssPolygonPointList {
        &self.points
    }
}

/// One independently authored `rect()` edge, without inferred geometry.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssRectShapeEdge {
    Auto,
    LengthPercentage(CssSpecifiedLengthPercentage),
}

impl PartialEq for CssRectShapeEdge {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Auto, Self::Auto) => true,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }
}

/// Four independently authored rectangle edges and optional authored rounding.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRectShape {
    top: CssRectShapeEdge,
    right: CssRectShapeEdge,
    bottom: CssRectShapeEdge,
    left: CssRectShapeEdge,
    round: Option<crate::CssBorderRadiusShorthand>,
}

impl CssRectShape {
    /// Composes checked edges in top/right/bottom/left order without resolving them.
    #[must_use]
    pub const fn new(
        top: CssRectShapeEdge,
        right: CssRectShapeEdge,
        bottom: CssRectShapeEdge,
        left: CssRectShapeEdge,
        round: Option<crate::CssBorderRadiusShorthand>,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
            round,
        }
    }
    #[must_use]
    pub const fn top(&self) -> &CssRectShapeEdge {
        &self.top
    }
    #[must_use]
    pub const fn right(&self) -> &CssRectShapeEdge {
        &self.right
    }
    #[must_use]
    pub const fn bottom(&self) -> &CssRectShapeEdge {
        &self.bottom
    }
    #[must_use]
    pub const fn left(&self) -> &CssRectShapeEdge {
        &self.left
    }
    #[must_use]
    pub const fn round(&self) -> Option<&crate::CssBorderRadiusShorthand> {
        self.round.as_ref()
    }
}

/// Authored signed coordinates, checked nonnegative dimensions, and optional rounding.
#[derive(Clone, Debug)]
pub struct CssXywhShape {
    x: CssSpecifiedLengthPercentage,
    y: CssSpecifiedLengthPercentage,
    width: CssSpecifiedNonNegativeLengthPercentage,
    height: CssSpecifiedNonNegativeLengthPercentage,
    round: Option<crate::CssBorderRadiusShorthand>,
}
numeric_fields_eq!(CssXywhShape, [x, y, width, height], [], [round]);

impl CssXywhShape {
    /// Composes checked inputs without resolving coordinates or clamping dimensions.
    #[must_use]
    pub const fn new(
        x: CssSpecifiedLengthPercentage,
        y: CssSpecifiedLengthPercentage,
        width: CssSpecifiedNonNegativeLengthPercentage,
        height: CssSpecifiedNonNegativeLengthPercentage,
        round: Option<crate::CssBorderRadiusShorthand>,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            round,
        }
    }
    #[must_use]
    pub const fn x(&self) -> &CssSpecifiedLengthPercentage {
        &self.x
    }
    #[must_use]
    pub const fn y(&self) -> &CssSpecifiedLengthPercentage {
        &self.y
    }
    #[must_use]
    pub const fn width(&self) -> &CssSpecifiedNonNegativeLengthPercentage {
        &self.width
    }
    #[must_use]
    pub const fn height(&self) -> &CssSpecifiedNonNegativeLengthPercentage {
        &self.height
    }
    #[must_use]
    pub const fn round(&self) -> Option<&crate::CssBorderRadiusShorthand> {
        self.round.as_ref()
    }
}

/// A selected authored basic-shape function.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBasicShape {
    Inset(Box<CssInsetShape>),
    Circle(CssCircleShape),
    Ellipse(CssEllipseShape),
    Polygon(CssPolygonShape),
    Rect(CssRectShape),
    Xywh(CssXywhShape),
}

/// The supported authored subset of `clip-path`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssClipPath {
    None,
    Url(CssUrl),
    GeometryBox(CssBoxEdgeKeyword),
    BasicShape(CssClipPathShape),
}

/// An authored shape with an optional explicit clipping reference box.
/// Box omission stays distinct from explicit border-box and requires downstream context.
#[derive(Clone, Debug, PartialEq)]
pub struct CssClipPathShape {
    shape: CssBasicShape,
    reference_box: Option<CssBoxEdgeKeyword>,
}

impl CssClipPathShape {
    /// Composes checked inputs without inferring a default reference box.
    #[must_use]
    pub const fn new(shape: CssBasicShape, reference_box: Option<CssBoxEdgeKeyword>) -> Self {
        Self {
            shape,
            reference_box,
        }
    }
    #[must_use]
    pub const fn shape(&self) -> &CssBasicShape {
        &self.shape
    }
    #[must_use]
    pub const fn reference_box(&self) -> Option<CssBoxEdgeKeyword> {
        self.reference_box
    }
}
