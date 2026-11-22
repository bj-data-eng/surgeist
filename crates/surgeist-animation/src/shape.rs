//! Computed basic-shape interpolation over caller-selected reference boxes.
//!
//! Callers normalize centers, expand inset offsets and corner radii, supply
//! polygon fill rules and lower omitted polygon rounding to a computed zero
//! length. Percentages and contextual numeric sources stay symbolic. Reference
//! box selection, authored CSS, final range/overlap adjustment and geometric
//! execution remain outside Animation. Native `shape()`/`path()` commands retain
//! their forms and implicit controls. Shape addition and accumulation are not provided.
//!
//! ```
//! use surgeist_animation::computed::{ComputedDimension, ComputedNumeric, ComputedPosition, DimensionKind};
//! use surgeist_animation::interpolation::InterpolationProgress;
//! use surgeist_animation::shape::{BasicShape, Circle, LengthPercentage, Radius, ReferenceBox, ShapeValue};
//! let radius = |value| LengthPercentage::new(ComputedNumeric::dimension(
//!     ComputedDimension::new(DimensionKind::Length, value).unwrap(),
//! )).unwrap();
//! let center = ComputedPosition::new(
//!     ComputedNumeric::percentage(0.5).unwrap(),
//!     ComputedNumeric::percentage(0.5).unwrap(),
//! ).unwrap();
//! let from = ShapeValue::new(ReferenceBox::BorderBox,
//!     BasicShape::Circle(Circle::new(Radius::Numeric(radius(10.0)), center.clone())));
//! let to = ShapeValue::new(ReferenceBox::BorderBox,
//!     BasicShape::Circle(Circle::new(Radius::Numeric(radius(30.0)), center)));
//! let sampled = from.interpolate(&to, InterpolationProgress::new(0.5).unwrap()).unwrap();
//! let BasicShape::Circle(circle) = sampled.shape() else { panic!("expected circle") };
//! assert_eq!(circle.radius(), &Radius::Numeric(radius(20.0)));
//! ```

mod commands;
pub use commands::{
    ArcCommand, ArcSize, ArcSweep, AxisCommand, CommandComponent, CommandPath, CommandShape,
    ControlAnchor, ControlPoint, CurveControls, Endpoint, ShapeCommand,
};

use core::fmt;
use std::sync::Arc;

use crate::computed::{
    ComputedError, ComputedNumeric, ComputedPosition, DimensionKind, NumericType,
};
use crate::interpolation::InterpolationProgress;

/// A caller-selected semantic reference box, independent of its used geometry.
///
/// The consuming property's owner selects the applicable box and SVG mapping;
/// Animation compares only the already supplied semantic identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceBox {
    ContentBox,
    PaddingBox,
    BorderBox,
    MarginBox,
    FillBox,
    StrokeBox,
    ViewBox,
}

/// A computed shape's effective fill rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FillRule {
    Nonzero,
    Evenodd,
}

/// A shape constructor or interpolation failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeErrorKind {
    InvalidNumericType,
    EmptyPolygon,
    EmptyCommands,
    InvalidPathControlAnchor,
    Numeric,
    Capacity,
}

/// The shape component whose construction or sampling failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeComponent {
    Radius,
    HorizontalRadius,
    VerticalRadius,
    Center,
    /// Top, right, bottom, left order.
    InsetOffset(usize),
    /// Top-left, top-right, bottom-right, bottom-left order.
    CornerHorizontalRadius(usize),
    /// Top-left, top-right, bottom-right, bottom-left order.
    CornerVerticalRadius(usize),
    PolygonVertex(usize),
    PolygonRounding,
    InitialPoint,
    Command {
        index: usize,
        component: CommandComponent,
    },
}

/// A checked shape error with its numeric cause or rejected computed type.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeError {
    kind: ShapeErrorKind,
    component: Option<ShapeComponent>,
    actual_type: Option<NumericType>,
    numeric: Option<ComputedError>,
}

impl ShapeError {
    fn new(kind: ShapeErrorKind) -> Self {
        Self {
            kind,
            component: None,
            actual_type: None,
            numeric: None,
        }
    }
    fn invalid_type(actual_type: NumericType) -> Self {
        Self {
            actual_type: Some(actual_type),
            ..Self::new(ShapeErrorKind::InvalidNumericType)
        }
    }
    fn numeric(error: ComputedError) -> Self {
        Self {
            numeric: Some(error),
            ..Self::new(ShapeErrorKind::Numeric)
        }
    }
    fn at(mut self, component: ShapeComponent) -> Self {
        self.component = Some(component);
        self
    }
    pub const fn kind(&self) -> ShapeErrorKind {
        self.kind
    }
    pub const fn component(&self) -> Option<ShapeComponent> {
        self.component
    }
    pub const fn actual_type(&self) -> Option<NumericType> {
        self.actual_type
    }
    pub const fn numeric_error(&self) -> Option<&ComputedError> {
        self.numeric.as_ref()
    }
}

impl fmt::Display for ShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ShapeErrorKind::InvalidNumericType => {
                write!(f, "invalid shape numeric type {:?}", self.actual_type)?
            }
            ShapeErrorKind::EmptyPolygon => {
                f.write_str("computed polygon must contain at least one vertex")?
            }
            ShapeErrorKind::EmptyCommands => {
                f.write_str("computed shape must contain a command after its initial point")?
            }
            ShapeErrorKind::InvalidPathControlAnchor => {
                f.write_str("path control anchor does not match its absolute or relative command")?
            }
            ShapeErrorKind::Numeric => f.write_str("shape numeric interpolation failed")?,
            ShapeErrorKind::Capacity => {
                f.write_str("shape result storage could not be reserved")?
            }
        }
        if let Some(component) = self.component {
            write!(f, " at {component:?}")?;
        }
        if let Some(error) = &self.numeric {
            write!(f, ": {error}")?;
        }
        Ok(())
    }
}
impl std::error::Error for ShapeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.numeric
            .as_ref()
            .map(|error| error as &dyn std::error::Error)
    }
}

/// A validated computed length, percentage or length-percentage expression.
///
/// Negative sampled values are retained for final caller-owned range handling.
#[derive(Debug, Clone, PartialEq)]
pub struct LengthPercentage {
    numeric: ComputedNumeric,
}

impl LengthPercentage {
    pub fn new(numeric: ComputedNumeric) -> Result<Self, ShapeError> {
        match numeric.numeric_type() {
            NumericType::Dimension(DimensionKind::Length)
            | NumericType::DimensionPercentage(DimensionKind::Length)
            | NumericType::Percentage => Ok(Self { numeric }),
            actual => Err(ShapeError::invalid_type(actual)),
        }
    }
    pub const fn numeric(&self) -> &ComputedNumeric {
        &self.numeric
    }
    fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ShapeError> {
        Self::new(
            self.numeric
                .interpolate(&other.numeric, progress)
                .map_err(ShapeError::numeric)?,
        )
    }
}

/// A computed radial-size branch, retaining keywords without inventing geometry.
#[derive(Debug, Clone, PartialEq)]
pub enum Radius {
    Numeric(LengthPercentage),
    ClosestSide,
    FarthestSide,
    ClosestCorner,
    FarthestCorner,
}

/// A circle with an explicit radius and normalized horizontal/vertical center.
#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    radius: Radius,
    center: ComputedPosition,
}

impl Circle {
    pub const fn new(radius: Radius, center: ComputedPosition) -> Self {
        Self { radius, center }
    }
    pub const fn radius(&self) -> &Radius {
        &self.radius
    }
    pub const fn center(&self) -> &ComputedPosition {
        &self.center
    }
}

/// An ellipse retaining independent horizontal and vertical radius branches.
#[derive(Debug, Clone, PartialEq)]
pub struct Ellipse {
    horizontal_radius: Radius,
    vertical_radius: Radius,
    center: ComputedPosition,
}

impl Ellipse {
    pub const fn new(
        horizontal_radius: Radius,
        vertical_radius: Radius,
        center: ComputedPosition,
    ) -> Self {
        Self {
            horizontal_radius,
            vertical_radius,
            center,
        }
    }
    pub const fn horizontal_radius(&self) -> &Radius {
        &self.horizontal_radius
    }
    pub const fn vertical_radius(&self) -> &Radius {
        &self.vertical_radius
    }
    pub const fn center(&self) -> &ComputedPosition {
        &self.center
    }
}

/// Eight expanded corner components in top-left, top-right, bottom-right, bottom-left order.
#[derive(Debug, Clone, PartialEq)]
pub struct CornerRadii {
    horizontal: [LengthPercentage; 4],
    vertical: [LengthPercentage; 4],
}

impl CornerRadii {
    pub const fn new(horizontal: [LengthPercentage; 4], vertical: [LengthPercentage; 4]) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
    pub const fn horizontal(&self) -> &[LengthPercentage; 4] {
        &self.horizontal
    }
    pub const fn vertical(&self) -> &[LengthPercentage; 4] {
        &self.vertical
    }
}

/// Four expanded top/right/bottom/left inset offsets and eight corner radii.
///
/// An offset array with any other arity cannot enter this computed model:
///
/// ```compile_fail,E0308
/// use surgeist_animation::shape::{CornerRadii, Inset, LengthPercentage};
/// fn wrong_arity(offsets: [LengthPercentage; 3], corners: CornerRadii) {
///     let _ = Inset::new(offsets, corners);
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Inset {
    data: Arc<InsetData>,
}

#[derive(Debug, Clone, PartialEq)]
struct InsetData {
    offsets: [LengthPercentage; 4],
    corners: CornerRadii,
}

impl Inset {
    pub fn new(offsets: [LengthPercentage; 4], corners: CornerRadii) -> Self {
        Self {
            data: Arc::new(InsetData { offsets, corners }),
        }
    }
    pub fn offsets(&self) -> &[LengthPercentage; 4] {
        &self.data.offsets
    }
    pub fn corners(&self) -> &CornerRadii {
        &self.data.corners
    }
}

/// A nonempty ordered polygon and its effective fill rule and computed rounding length.
///
/// One and two vertices are valid. The caller supplies a computed zero length
/// when authored rounding was omitted. Vertex/rounding geometry is not executed.
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    vertices: Arc<[ComputedPosition]>,
    fill_rule: FillRule,
    rounding: ComputedNumeric,
}

impl Polygon {
    pub fn new(
        vertices: Vec<ComputedPosition>,
        fill_rule: FillRule,
        rounding: ComputedNumeric,
    ) -> Result<Self, ShapeError> {
        if vertices.is_empty() {
            return Err(ShapeError::new(ShapeErrorKind::EmptyPolygon));
        }
        if rounding.numeric_type() != NumericType::Dimension(DimensionKind::Length) {
            return Err(ShapeError::invalid_type(rounding.numeric_type())
                .at(ShapeComponent::PolygonRounding));
        }
        Ok(Self {
            vertices: vertices.into(),
            fill_rule,
            rounding,
        })
    }
    pub fn vertices(&self) -> &[ComputedPosition] {
        &self.vertices
    }
    pub const fn fill_rule(&self) -> FillRule {
        self.fill_rule
    }
    pub const fn rounding(&self) -> &ComputedNumeric {
        &self.rounding
    }
}

/// The supported computed basic-shape functions.
#[derive(Debug, Clone, PartialEq)]
pub enum BasicShape {
    Circle(Circle),
    Ellipse(Ellipse),
    Inset(Inset),
    Polygon(Polygon),
    Shape(CommandShape),
    Path(CommandPath),
}

/// An immutable computed shape with an explicit semantic reference-box identity.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeValue {
    reference_box: ReferenceBox,
    shape: Arc<BasicShape>,
}

impl ShapeValue {
    pub fn new(reference_box: ReferenceBox, shape: BasicShape) -> Self {
        Self {
            reference_box,
            shape: Arc::new(shape),
        }
    }
    pub const fn reference_box(&self) -> ReferenceBox {
        self.reference_box
    }
    pub fn shape(&self) -> &BasicShape {
        &self.shape
    }

    /// Samples compatible shapes, otherwise switching the complete value at 0.5.
    ///
    /// Compatible mixed Shape/Path lists return Shape even at exact endpoints;
    /// two Path lists retain Path. Command compatibility includes effective
    /// control anchors and preserves implicit smooth controls. Geometry and
    /// final range handling remain with the consuming owner.
    pub fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ShapeError> {
        // Compatibility is global: even a late keyword or metadata conflict
        // must win over an earlier component's otherwise overflowing arithmetic.
        if !self.compatible(other) {
            return Ok(self.discrete(other, progress));
        }
        if let Some(shape) = self.shape().interpolate_commands(other.shape(), progress)? {
            return Ok(Self::new(self.reference_box, shape));
        }
        if progress.value() == 0.0 {
            return Ok(self.clone());
        }
        if progress.value() == 1.0 {
            return Ok(other.clone());
        }
        let shape = match (self.shape(), other.shape()) {
            (
                BasicShape::Circle(Circle {
                    radius: Radius::Numeric(first),
                    center: first_center,
                }),
                BasicShape::Circle(Circle {
                    radius: Radius::Numeric(second),
                    center: second_center,
                }),
            ) => BasicShape::Circle(Circle::new(
                Radius::Numeric(
                    first
                        .interpolate(second, progress)
                        .map_err(|error| error.at(ShapeComponent::Radius))?,
                ),
                first_center
                    .interpolate(second_center, progress)
                    .map_err(|error| ShapeError::numeric(error).at(ShapeComponent::Center))?,
            )),
            (
                BasicShape::Ellipse(Ellipse {
                    horizontal_radius: Radius::Numeric(first_x),
                    vertical_radius: Radius::Numeric(first_y),
                    center: first_center,
                }),
                BasicShape::Ellipse(Ellipse {
                    horizontal_radius: Radius::Numeric(second_x),
                    vertical_radius: Radius::Numeric(second_y),
                    center: second_center,
                }),
            ) => BasicShape::Ellipse(Ellipse::new(
                Radius::Numeric(
                    first_x
                        .interpolate(second_x, progress)
                        .map_err(|error| error.at(ShapeComponent::HorizontalRadius))?,
                ),
                Radius::Numeric(
                    first_y
                        .interpolate(second_y, progress)
                        .map_err(|error| error.at(ShapeComponent::VerticalRadius))?,
                ),
                first_center
                    .interpolate(second_center, progress)
                    .map_err(|error| ShapeError::numeric(error).at(ShapeComponent::Center))?,
            )),
            (BasicShape::Inset(first), BasicShape::Inset(second)) => BasicShape::Inset(Inset::new(
                interpolate_four(
                    first.offsets(),
                    second.offsets(),
                    progress,
                    ShapeComponent::InsetOffset,
                )?,
                CornerRadii::new(
                    interpolate_four(
                        first.corners().horizontal(),
                        second.corners().horizontal(),
                        progress,
                        ShapeComponent::CornerHorizontalRadius,
                    )?,
                    interpolate_four(
                        first.corners().vertical(),
                        second.corners().vertical(),
                        progress,
                        ShapeComponent::CornerVerticalRadius,
                    )?,
                ),
            )),
            (BasicShape::Polygon(first), BasicShape::Polygon(second)) => {
                let mut vertices = Vec::new();
                vertices
                    .try_reserve_exact(first.vertices.len())
                    .map_err(|_| ShapeError::new(ShapeErrorKind::Capacity))?;
                for (index, (first, second)) in first
                    .vertices
                    .iter()
                    .zip(second.vertices.iter())
                    .enumerate()
                {
                    vertices.push(first.interpolate(second, progress).map_err(|error| {
                        ShapeError::numeric(error).at(ShapeComponent::PolygonVertex(index))
                    })?);
                }
                let rounding = first
                    .rounding
                    .interpolate(&second.rounding, progress)
                    .map_err(|error| {
                        ShapeError::numeric(error).at(ShapeComponent::PolygonRounding)
                    })?;
                BasicShape::Polygon(Polygon::new(vertices, first.fill_rule, rounding)?)
            }
            _ => return Ok(self.discrete(other, progress)),
        };
        Ok(Self::new(self.reference_box, shape))
    }

    fn compatible(&self, other: &Self) -> bool {
        self.reference_box == other.reference_box
            && match (self.shape(), other.shape()) {
                (BasicShape::Circle(first), BasicShape::Circle(second)) => matches!(
                    (&first.radius, &second.radius),
                    (Radius::Numeric(_), Radius::Numeric(_))
                ),
                (BasicShape::Ellipse(first), BasicShape::Ellipse(second)) => matches!(
                    (
                        &first.horizontal_radius,
                        &first.vertical_radius,
                        &second.horizontal_radius,
                        &second.vertical_radius
                    ),
                    (
                        Radius::Numeric(_),
                        Radius::Numeric(_),
                        Radius::Numeric(_),
                        Radius::Numeric(_)
                    )
                ),
                (BasicShape::Inset(_), BasicShape::Inset(_)) => true,
                (BasicShape::Polygon(first), BasicShape::Polygon(second)) => {
                    first.fill_rule == second.fill_rule
                        && first.vertices.len() == second.vertices.len()
                }
                _ => self.shape().commands_compatible(other.shape()),
            }
    }
    fn discrete(&self, other: &Self, progress: InterpolationProgress) -> Self {
        if progress.value() < 0.5 {
            self.clone()
        } else {
            other.clone()
        }
    }
}

fn interpolate_four(
    first: &[LengthPercentage; 4],
    second: &[LengthPercentage; 4],
    progress: InterpolationProgress,
    component: fn(usize) -> ShapeComponent,
) -> Result<[LengthPercentage; 4], ShapeError> {
    let mut result = first.clone();
    for (index, ((value, first), second)) in result
        .iter_mut()
        .zip(first.iter())
        .zip(second.iter())
        .enumerate()
    {
        *value = first
            .interpolate(second, progress)
            .map_err(|error| error.at(component(index)))?;
    }
    Ok(result)
}
