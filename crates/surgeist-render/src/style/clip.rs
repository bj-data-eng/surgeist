use super::super::{
    Capabilities, CoordinateSpaceTag, Error, FillRule, FilledPath, Path, Point, PrimitiveFamily,
    PrimitiveOperation, Radii, Rect, Result, Shape, Size, Transform, UnresolvedResource,
    UnresolvedResourceKind, UnsupportedPrimitive,
    shape::ShapeKind,
    validation::{validate_finite_f64, validate_path, validate_shape},
};
use super::image::StyleResourceRef;
use crate::accounting::{
    Account, BackedValue, Hold, RenderAccountingContext, RenderAttemptKind, RenderPhase,
};
use kurbo::Shape as KurboShape;

#[derive(Clone, Debug, PartialEq)]
/// An authored clip source with an optional tagged coordinate space.
///
/// Concrete shapes and paths can be normalized locally. A resource reference
/// remains symbolic and produces an unresolved-resource diagnostic until the
/// surrounding context replaces it with concrete geometry.
pub struct ClipInput {
    kind: ClipInputKind,
    coordinate_space: Option<CoordinateSpaceTag>,
}

#[derive(Clone, Debug, PartialEq)]
/// The authored source represented by a [`ClipInput`].
pub enum ClipInputKind {
    /// Validated shape geometry.
    Shape(Shape),
    /// A validated path paired with its fill rule.
    Path(FilledPath),
    /// A symbolic clip-resource reference.
    Reference(StyleResourceRef),
}

impl ClipInput {
    pub(crate) fn validate_in(&self, account: &Account) -> Result<()> {
        match &self.kind {
            ClipInputKind::Shape(shape) => crate::validation::validate_shape_in(shape, account)?,
            ClipInputKind::Path(path) => crate::validation::validate_path_in(path.path(), account)?,
            ClipInputKind::Reference(_) => {}
        }
        if let Some(coordinate_space) = self.coordinate_space {
            validate_clip_transform_values(coordinate_space.transform())?;
        }
        Ok(())
    }
    /// Creates a clip from validated logical shape geometry.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] when the shape is invalid.
    pub fn try_shape(shape: Shape) -> Result<Self> {
        validate_shape(&shape)?;
        Ok(Self::from_validated_shape(shape))
    }

    /// Stores the intrinsic invariants established by public Shape construction.
    pub(crate) fn from_validated_shape(shape: Shape) -> Self {
        Self {
            kind: ClipInputKind::Shape(shape),
            coordinate_space: None,
        }
    }

    /// Creates a clip from a filled logical path.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] when path validation fails.
    pub fn try_filled_path(path: FilledPath) -> Result<Self> {
        validate_path_clip(path.path())?;
        Ok(Self {
            kind: ClipInputKind::Path(path),
            coordinate_space: None,
        })
    }

    /// Creates a symbolic clip-resource input.
    #[must_use]
    pub const fn reference(reference: StyleResourceRef) -> Self {
        Self {
            kind: ClipInputKind::Reference(reference),
            coordinate_space: None,
        }
    }

    /// Associates the input with a tagged coordinate space.
    #[must_use]
    pub fn with_coordinate_space(mut self, coordinate_space: CoordinateSpaceTag) -> Self {
        self.coordinate_space = Some(coordinate_space);
        self
    }

    /// Returns the authored clip source.
    #[must_use]
    pub const fn kind(&self) -> &ClipInputKind {
        &self.kind
    }

    /// Returns the shape source, if this input contains one.
    #[must_use]
    pub const fn shape(&self) -> Option<&Shape> {
        match &self.kind {
            ClipInputKind::Shape(shape) => Some(shape),
            ClipInputKind::Path(_) => None,
            ClipInputKind::Reference(_) => None,
        }
    }

    /// Returns the filled-path source, if this input contains one.
    #[must_use]
    pub const fn filled_path(&self) -> Option<&FilledPath> {
        match &self.kind {
            ClipInputKind::Shape(_) => None,
            ClipInputKind::Path(path) => Some(path),
            ClipInputKind::Reference(_) => None,
        }
    }

    /// Returns the symbolic resource reference, if this input contains one.
    #[must_use]
    pub const fn reference_ref(&self) -> Option<&StyleResourceRef> {
        match &self.kind {
            ClipInputKind::Shape(_) => None,
            ClipInputKind::Path(_) => None,
            ClipInputKind::Reference(reference) => Some(reference),
        }
    }

    /// Returns the optional coordinate-space tag.
    #[must_use]
    pub const fn coordinate_space(&self) -> Option<CoordinateSpaceTag> {
        self.coordinate_space
    }

    /// Checks whether the current capability contract accepts this clip source.
    ///
    /// Concrete geometry is checked as shape clipping. A symbolic reference
    /// returns an unresolved clip-resource diagnostic.
    pub fn ensure_supported(&self, capabilities: Capabilities) -> Result<()> {
        match &self.kind {
            ClipInputKind::Shape(_) | ClipInputKind::Path(_) => {
                capabilities.ensure_supported(UnsupportedPrimitive::new(
                    PrimitiveFamily::MasksAndClips,
                    PrimitiveOperation::ShapeClip,
                ))
            }
            ClipInputKind::Reference(reference) => Err(Error::unresolved_resource(
                UnresolvedResource::new(UnresolvedResourceKind::Clip, reference.identifier()),
            )),
        }
    }

    /// Converts a supported concrete input into normalized clip geometry.
    ///
    /// This validates the source and its finite transformed logical bounds.
    /// Unsupported geometry returns a capability diagnostic, while symbolic
    /// references return an unresolved-resource diagnostic.
    pub fn normalize(
        &self,
        capabilities: Capabilities,
        context: &mut RenderAccountingContext,
    ) -> Result<NormalizedClip> {
        context.run(RenderAttemptKind::Clip, RenderPhase::Clip, |account| {
            let guarded = self.normalize_in(capabilities, account)?;
            guarded.require_preparation()?;
            let (clip, backing, import) = guarded.into_parts();
            for hold in backing.into_iter().flatten() {
                hold.into_preparation()?.retain();
            }
            drop(import);
            Ok(clip)
        })
    }

    pub(crate) fn normalize_in(
        &self,
        capabilities: Capabilities,
        account: &Account,
    ) -> Result<BackedValue<NormalizedClip>> {
        let account = account.at(RenderPhase::Clip);
        account.phase(RenderPhase::Clip, || {
            self.ensure_supported(capabilities)?;
            let (geometry, hold) = match &self.kind {
                ClipInputKind::Shape(shape) => ClipGeometry::from_shape_in(shape, &account)?,
                ClipInputKind::Path(path) => {
                    crate::validation::validate_path_in(path.path(), &account)?;
                    let (copied, hold) = path.path().try_clone_in(&account)?;
                    (
                        ClipGeometry {
                            kind: ClipGeometryKind::Path(FilledPath::from_validated_parts(
                                copied,
                                path.fill_rule(),
                            )),
                        },
                        Some(hold),
                    )
                }
                ClipInputKind::Reference(reference) => {
                    return Err(Error::unresolved_resource(UnresolvedResource::new(
                        UnresolvedResourceKind::Clip,
                        reference.identifier(),
                    )));
                }
            };
            let normalized = BackedValue::new(
                NormalizedClip {
                    geometry,
                    coordinate_space: self.coordinate_space,
                },
                [hold, None],
            );
            validate_clip_transformed_bounds_in(
                &normalized.geometry,
                normalized.coordinate_space,
                &account,
            )?;
            Ok(normalized)
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Intrinsically validated clip geometry and its optional coordinate-space tag.
///
/// Construction guarantees valid geometry, finite transform coefficients, and
/// finite logical bounds after applying the tagged transform.
pub struct NormalizedClip {
    geometry: ClipGeometry,
    coordinate_space: Option<CoordinateSpaceTag>,
}

impl NormalizedClip {
    pub(crate) fn into_parts(self) -> (ClipGeometry, Option<CoordinateSpaceTag>) {
        (self.geometry, self.coordinate_space)
    }
    /// Validates normalized clip geometry in its optional coordinate space.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] for invalid geometry or for
    /// transform coefficients or transformed bounds that are not finite.
    pub fn try_new(
        geometry: ClipGeometry,
        coordinate_space: Option<CoordinateSpaceTag>,
    ) -> Result<Self> {
        validate_clip_geometry(&geometry)?;
        validate_clip_transformed_bounds(&geometry, coordinate_space)?;
        Ok(Self {
            geometry,
            coordinate_space,
        })
    }

    /// Returns the normalized clip geometry.
    #[must_use]
    pub const fn geometry(&self) -> &ClipGeometry {
        &self.geometry
    }

    /// Returns the optional coordinate-space tag used for the geometry.
    #[must_use]
    pub const fn coordinate_space(&self) -> Option<CoordinateSpaceTag> {
        self.coordinate_space
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Intrinsically validated clip geometry in logical rendering units.
pub struct ClipGeometry {
    kind: ClipGeometryKind,
}

#[derive(Clone, Debug, PartialEq)]
/// The closed geometry choice stored by [`ClipGeometry`].
pub enum ClipGeometryKind {
    /// An axis-aligned logical rectangle.
    Rect(Rect),
    /// A logical rectangle with corner radii.
    RoundedRect {
        /// The rectangle bounds.
        rect: Rect,
        /// The logical corner radii.
        radii: Radii,
    },
    /// A circle in logical coordinates.
    Circle {
        /// The logical center point.
        center: Point,
        /// The logical radius.
        radius: f64,
    },
    /// An ellipse in logical coordinates.
    Ellipse {
        /// The logical center point.
        center: Point,
        /// The horizontal and vertical logical radii.
        radii: Size,
    },
    /// A logical path paired with its fill rule.
    Path(FilledPath),
}

impl ClipGeometry {
    pub(crate) fn into_kind(self) -> ClipGeometryKind {
        self.kind
    }
    fn from_shape_in(shape: &Shape, account: &Account) -> Result<(Self, Option<Hold>)> {
        crate::validation::validate_shape_in(shape, account)?;
        let (kind, hold) = match shape.kind() {
            ShapeKind::Rect(rect) => (ClipGeometryKind::Rect(*rect), None),
            ShapeKind::RoundedRect { rect, radii } => (
                ClipGeometryKind::RoundedRect {
                    rect: *rect,
                    radii: *radii,
                },
                None,
            ),
            ShapeKind::Circle { center, radius } => (
                ClipGeometryKind::Circle {
                    center: *center,
                    radius: *radius,
                },
                None,
            ),
            ShapeKind::Ellipse { center, radii } => (
                ClipGeometryKind::Ellipse {
                    center: *center,
                    radii: *radii,
                },
                None,
            ),
            ShapeKind::Path(path) => {
                let (path, hold) = path.try_clone_in(account)?;
                (
                    ClipGeometryKind::Path(FilledPath::from_validated_parts(
                        path,
                        FillRule::NonZero,
                    )),
                    Some(hold),
                )
            }
        };
        Ok((Self { kind }, hold))
    }
    /// Validates and stores filled path geometry.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] when path validation fails.
    pub fn try_path(path: FilledPath) -> Result<Self> {
        validate_path_clip(path.path())?;
        Ok(Self {
            kind: ClipGeometryKind::Path(path),
        })
    }

    /// Returns the normalized geometry choice.
    #[must_use]
    pub const fn kind(&self) -> &ClipGeometryKind {
        &self.kind
    }
}

pub(crate) fn validate_clip_input(input: &ClipInput) -> Result<()> {
    match input.kind() {
        ClipInputKind::Shape(shape) => validate_shape(shape),
        ClipInputKind::Path(path) => validate_path_clip(path.path()),
        ClipInputKind::Reference(_) => Ok(()),
    }?;
    if let Some(coordinate_space) = input.coordinate_space() {
        validate_clip_transform_values(coordinate_space.transform())?;
    }
    Ok(())
}

fn validate_clip_geometry(geometry: &ClipGeometry) -> Result<()> {
    match geometry.kind() {
        ClipGeometryKind::Rect(rect) => validate_shape(&Shape::rect(*rect)),
        ClipGeometryKind::RoundedRect { rect, radii } => {
            validate_shape(&Shape::try_rounded_rect(*rect, *radii)?)
        }
        ClipGeometryKind::Circle { center, radius } => {
            validate_shape(&Shape::try_circle(*center, *radius)?)
        }
        ClipGeometryKind::Ellipse { center, radii } => {
            validate_shape(&Shape::try_ellipse(*center, *radii)?)
        }
        ClipGeometryKind::Path(path) => validate_path_clip(path.path()),
    }
}

fn validate_path_clip(path: &Path) -> Result<()> {
    validate_path(path)
}

fn validate_clip_transform_values(transform: Transform) -> Result<()> {
    for value in transform.as_array() {
        validate_finite_f64(value, "clip coordinate-space transform")?;
    }
    Ok(())
}

fn validate_clip_transformed_bounds(
    geometry: &ClipGeometry,
    coordinate_space: Option<CoordinateSpaceTag>,
) -> Result<()> {
    let bounds = clip_geometry_bounds(geometry)?;
    let transform = coordinate_space
        .map(CoordinateSpaceTag::transform)
        .unwrap_or_else(Transform::identity);
    validate_clip_transform_values(transform)?;
    transformed_clip_bounds(bounds, transform).map(|_| ())
}

fn clip_geometry_bounds(geometry: &ClipGeometry) -> Result<Rect> {
    match geometry.kind() {
        ClipGeometryKind::Rect(rect) | ClipGeometryKind::RoundedRect { rect, .. } => Ok(*rect),
        ClipGeometryKind::Circle { center, radius } => finite_clip_rect(
            center.x() - radius,
            center.y() - radius,
            radius * 2.0,
            radius * 2.0,
        ),
        ClipGeometryKind::Ellipse { center, radii } => finite_clip_rect(
            center.x() - radii.width(),
            center.y() - radii.height(),
            radii.width() * 2.0,
            radii.height() * 2.0,
        ),
        ClipGeometryKind::Path(path) => {
            let bounds = path.path().to_kurbo().bounding_box();
            finite_clip_rect(bounds.x0, bounds.y0, bounds.width(), bounds.height())
        }
    }
}

fn validate_clip_transformed_bounds_in(
    geometry: &ClipGeometry,
    coordinate_space: Option<CoordinateSpaceTag>,
    account: &Account,
) -> Result<()> {
    let bounds = match geometry.kind() {
        ClipGeometryKind::Path(path) => {
            let path = path.path().kurbo_shape_in(account)?;
            let bounds = path.bounding_box();
            finite_clip_rect(bounds.x0, bounds.y0, bounds.width(), bounds.height())?
        }
        _ => clip_geometry_bounds(geometry)?,
    };
    let transform = coordinate_space
        .map(CoordinateSpaceTag::transform)
        .unwrap_or_else(Transform::identity);
    validate_clip_transform_values(transform)?;
    transformed_clip_bounds(bounds, transform).map(|_| ())
}

fn transformed_clip_bounds(rect: Rect, transform: Transform) -> Result<Rect> {
    let [a, b, c, d, e, f] = transform.as_array();
    let max = rect.max();
    let corners = [
        (rect.x(), rect.y()),
        (max.x(), rect.y()),
        (max.x(), max.y()),
        (rect.x(), max.y()),
    ];
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for (x, y) in corners {
        let transformed_x = a * x + c * y + e;
        let transformed_y = b * x + d * y + f;
        if !transformed_x.is_finite() || !transformed_y.is_finite() {
            return Err(Error::invalid_value(
                "clip transformed bounds",
                "non-finite",
                "must remain finite after coordinate-space transform",
            ));
        }
        min_x = min_x.min(transformed_x);
        min_y = min_y.min(transformed_y);
        max_x = max_x.max(transformed_x);
        max_y = max_y.max(transformed_y);
    }
    finite_clip_rect(min_x, min_y, max_x - min_x, max_y - min_y)
}

fn finite_clip_rect(x: f64, y: f64, width: f64, height: f64) -> Result<Rect> {
    if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() {
        return Err(Error::invalid_value(
            "clip transformed bounds",
            format!("x {x}, y {y}, width {width}, height {height}"),
            "must remain finite after coordinate-space transform",
        ));
    }
    if width < 0.0 || height < 0.0 {
        return Err(Error::invalid_value(
            "clip transformed bounds",
            format!("width {width}, height {height}"),
            "must be non-negative",
        ));
    }
    Ok(Rect::new(x, y, width, height))
}

#[cfg(test)]
mod accounting_tests {
    use super::*;
    use crate::{RenderAccountingOptions, RenderLimits, RenderWork};

    #[test]
    fn allocating_path_normalization_preserves_geometry_and_measures_each_walk() {
        let mut path = Path::new();
        path.move_to(Point::new(1.0, 2.0))
            .line_to(Point::new(3.0, 4.0))
            .close();
        let input = ClipInput::try_filled_path(
            FilledPath::try_new(path.clone(), FillRule::EvenOdd).unwrap(),
        )
        .unwrap();
        let mut context = RenderAccountingContext::default();
        let clip = input
            .normalize(Capabilities::CURRENT, &mut context)
            .unwrap();
        let ClipGeometryKind::Path(normalized) = clip.geometry().kind() else {
            panic!("expected path clip")
        };
        assert_eq!(normalized.path(), &path);
        assert_eq!(normalized.fill_rule(), FillRule::EvenOdd);
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .usage()
                .work(RenderWork::PathElementVisits),
            9
        );
    }

    #[test]
    fn path_refusal_preserves_validation_prefix_and_transfers_no_output() {
        let mut path = Path::new();
        path.move_to(Point::new(1.0, 2.0))
            .line_to(Point::new(3.0, 4.0))
            .close();
        let input = ClipInput::try_shape(Shape::path(path)).unwrap();
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::PathElementVisits, 2)
            .unwrap();
        let mut context =
            RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
        assert!(
            input
                .normalize(Capabilities::CURRENT, &mut context)
                .is_err()
        );
        let usage = context.last_attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::PathElementVisits), 2);
        assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
    }

    mod normalization_custody_tests {
        use super::*;
        use crate::accounting::{FrameAttempt, FrameReportSlot, RendererCpu};
        use crate::{
            ErrorCode, RenderAccountingError, RenderAttemptStatus, RenderCpuScope, RenderProfile,
            RenderQuantity, RenderRefusalReason, RenderResource,
        };

        #[test]
        fn transformed_path_refusal_releases_frame_copy_and_preserves_standalone_transfer() {
            let mut path = Path::new();
            path.move_to(Point::new(1.0, 2.0))
                .line_to(Point::new(3.0, 4.0))
                .close();
            let input = ClipInput::try_filled_path(
                FilledPath::try_new(path.clone(), FillRule::EvenOdd).unwrap(),
            )
            .unwrap()
            .with_coordinate_space(
                CoordinateSpaceTag::surface(Transform::translation(5.0, 7.0).unwrap()).unwrap(),
            );
            let options = RenderAccountingOptions::new()
                .with_profile(RenderProfile::Summary { timing: false })
                .with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::PathElementVisits, 6)
                        .unwrap(),
                );
            let cpu = RendererCpu::new(options).unwrap();
            let opening = cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes);
            let attempt = FrameAttempt::begin(FrameReportSlot::default(), &cpu, options).unwrap();
            let error = input
                .normalize_in(Capabilities::CURRENT, attempt.account())
                .map(drop)
                .unwrap_err();
            let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
            else {
                panic!("expected the transformed-bounds path inspection to refuse, got {error:?}");
            };
            assert_eq!(error.code(), ErrorCode::Accounting);
            assert_eq!(refusal.phase(), RenderPhase::Attempt);
            assert_eq!(
                refusal.quantity(),
                RenderQuantity::Work(RenderWork::PathElementVisits)
            );
            assert_eq!(refusal.current(), 6);
            assert_eq!(refusal.amount(), 1);
            assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 6 });
            let (report, fault) =
                attempt.finish(RenderAttemptStatus::Refused, error.accounting_diagnostic());
            assert_eq!(fault, None);
            let usage = report.attempt().unwrap().usage();
            // Three validation visits and three copied elements precede the
            // first refused transformed-bounds materialization visit.
            assert_eq!(usage.work(RenderWork::PathElementVisits), 6);
            assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
            let cpu_usage = report.renderer_cpu().unwrap();
            assert_eq!(
                cpu_usage.usage().live(RenderResource::RetainedCpuBytes),
                opening
            );
            assert_eq!(
                cpu_usage
                    .scope_usage(RenderCpuScope::FrameBacking)
                    .live(RenderResource::RetainedCpuBytes),
                0
            );
            assert_eq!(
                cpu_usage
                    .scope_usage(RenderCpuScope::FrameBacking)
                    .live(RenderResource::ScratchCpuBytes),
                0
            );
            assert!(
                cpu_usage
                    .scope_usage(RenderCpuScope::FrameBacking)
                    .peak(RenderResource::RetainedCpuBytes)
                    > 0
            );
            assert!(
                cpu_usage
                    .scope_usage(RenderCpuScope::FrameBacking)
                    .peak(RenderResource::ScratchCpuBytes)
                    > 0
            );

            let before = cpu.snapshot().unwrap();
            let mut standalone = RenderAccountingContext::default();
            let normalized = input
                .normalize(Capabilities::CURRENT, &mut standalone)
                .unwrap();
            let ClipGeometryKind::Path(output) = normalized.geometry().kind() else {
                panic!("the standalone result must preserve the path");
            };
            assert_eq!(output.path(), &path);
            assert_eq!(output.fill_rule(), FillRule::EvenOdd);
            let clone = normalized.clone();
            drop(normalized);
            assert_eq!(clone.coordinate_space(), input.coordinate_space());
            assert!(
                standalone
                    .last_attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::PreparedTransferredBytes)
                    > 0
            );
            assert_eq!(
                standalone
                    .last_attempt()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes),
                0
            );
            drop(clone);
            assert_eq!(cpu.snapshot().unwrap(), before);
        }
    }
}
