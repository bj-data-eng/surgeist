use crate::{
    PaintError, PaintOptions, PaintPhase, PaintReport, ResourceKind, WorkKind,
    accounting::{Meter, OperationKind, PhaseMeter, apply, bytes, charge, finish, setup_error},
};
use std::cmp::Ordering;
use surgeist_layout::{Edges, LengthPercentageOf, Point, Size};
use surgeist_ledger::{Change, Limits, Scope, Tariffs};

mod mask;
mod tiling;
pub use mask::{MaskBorderOutcome, MaskBorderPlan, resolve_mask_border_in_area};
pub use tiling::{
    BorderImageAxisPattern, BorderImageAxisPlacement, BorderImageRepeat, BorderImageTileRegion,
    BorderImageTiling, BorderImageTilingOutcome, resolve_border_image_tiling,
};

/// The concrete image coordinates and natural CSS-pixel dimensions of a loaded source.
///
/// The coordinate size is positive. It is the decoded pixel extent for a raster
/// image or the supplied concrete coordinate extent for a vector image. Natural
/// dimensions are independently optional and can differ from the coordinate size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderImageSource {
    pub coordinate_size: Size<f64>,
    pub natural_size: Size<Option<f64>>,
}

/// An inward source slice. A fraction of 1 denotes the entire source axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderImageSlice {
    SourceUnits(f64),
    Fraction(f64),
}

/// The resolved form of a border-image-width component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderImageWidth {
    /// Uses the slice's natural CSS-pixel extent or the corresponding border width.
    Auto,
    /// Resolves finite length and percentage coefficients against the paint area
    /// and clamps a negative used width to zero.
    LengthPercentage(LengthPercentageOf<f64>),
    BorderWidthMultiple(f64),
}

/// An outward CSS-pixel length or a multiple of the corresponding computed border width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderImageOutset {
    Length(f64),
    BorderWidthMultiple(f64),
}

/// Input for outset geometry, in CSS pixels in one physical coordinate space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderImageAreaRequest {
    pub border_origin: Point<f64>,
    pub border_size: Size<f64>,
    pub border_widths: Edges<f64>,
    pub outsets: Edges<BorderImageOutset>,
}

/// Input for slicing a concrete source into an already admitted paint area.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderImagePartitionRequest {
    pub source: BorderImageSource,
    pub slices: Edges<BorderImageSlice>,
    pub widths: Edges<BorderImageWidth>,
    pub fill_center: bool,
}

/// Input for an already-sized border image. Box facts use CSS pixels in one
/// caller-selected physical coordinate space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderImageRequest {
    pub border_origin: Point<f64>,
    pub border_size: Size<f64>,
    pub border_widths: Edges<f64>,
    pub source: BorderImageSource,
    pub slices: Edges<BorderImageSlice>,
    pub widths: Edges<BorderImageWidth>,
    pub outsets: Edges<BorderImageOutset>,
    pub fill_center: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BorderImageGeometryError {
    InvalidBox,
    InvalidBorderWidth,
    InvalidSource,
    InvalidSlice,
    InvalidWidth,
    InvalidOutset,
    UnrepresentableGeometry,
}

/// An admitted rectangle in source units or CSS pixels, as indicated by its accessor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderImageRect {
    min: Point<f64>,
    max: Point<f64>,
}
impl BorderImageRect {
    pub const fn min(self) -> Point<f64> {
        self.min
    }
    pub const fn max(self) -> Point<f64> {
        self.max
    }
}

/// A resolved paint area and the border widths that produced it.
///
/// ```compile_fail
/// use surgeist_layout::Size;
/// use surgeist_paint::BorderImageArea;
///
/// fn replace_sizing_extent(area: &mut BorderImageArea) {
///     area.extent = Size::new(10.0, 20.0);
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderImageArea {
    rect: BorderImageRect,
    extent: Size<f64>,
    border_widths: Edges<f64>,
}
impl BorderImageArea {
    /// Returns the signed paint rectangle in CSS pixels.
    pub const fn rect(self) -> BorderImageRect {
        self.rect
    }

    /// Returns the CSS-pixel extent used for default source sizing and width percentages.
    /// This extent is independent of rounding in the placed rectangle's coordinates.
    pub const fn default_size(self) -> Size<f64> {
        self.extent
    }
}

/// One complete paint area or a typed refusal, with the operation's Ledger report.
#[derive(Debug)]
pub struct BorderImageAreaOutcome {
    pub result: Result<BorderImageArea, PaintError>,
    pub report: Option<PaintReport>,
}

/// One paired source and destination region. An empty region on one side
/// preserves the independent facts on the other side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderImageRegion {
    source: Option<BorderImageRect>,
    destination: Option<BorderImageRect>,
}
impl BorderImageRegion {
    const EMPTY: Self = Self {
        source: None,
        destination: None,
    };

    /// Returns the nonempty source region in source units, when present.
    pub const fn source(self) -> Option<BorderImageRect> {
        self.source
    }

    /// Returns the nonempty destination partition in CSS pixels, when present.
    pub const fn destination(self) -> Option<BorderImageRect> {
        self.destination
    }
}

/// The complete geometry for later repetition and concrete draw lowering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderImageGeometry {
    area: BorderImageRect,
    used_widths: Edges<f64>,
    regions: [[BorderImageRegion; 3]; 3],
}
impl BorderImageGeometry {
    /// Returns the CSS-pixel paint area, including any outset.
    pub const fn area(&self) -> BorderImageRect {
        self.area
    }

    /// Returns the physical width offsets after common-factor reduction.
    pub const fn used_widths(&self) -> Edges<f64> {
        self.used_widths
    }

    /// Returns top, middle, and bottom rows, each in left-to-right physical order.
    pub const fn regions(&self) -> &[[BorderImageRegion; 3]; 3] {
        &self.regions
    }
}

/// One complete geometry or a typed refusal, with the operation's Ledger report.
#[derive(Debug)]
pub struct BorderImageOutcome {
    pub result: Result<BorderImageGeometry, PaintError>,
    pub report: Option<PaintReport>,
}

/// Resolves the outset-adjusted paint area without concrete image coordinates.
pub fn resolve_border_image_area(
    request: BorderImageAreaRequest,
    options: PaintOptions,
) -> BorderImageAreaOutcome {
    let (result, report) = run(
        options,
        OperationKind::BorderImageArea,
        PaintPhase::BorderImageArea,
        Some(|work: &mut GeometryWork<'_, '_, '_>| {
            validate_box(work, request)?;
            validate_border_widths(work, request.border_widths)?;
            validate_outsets(work, request.outsets)
        }),
        |work| resolve_area(work, request),
    );
    BorderImageAreaOutcome { result, report }
}

/// Resolves slices and widths with the area and border widths already admitted by Paint.
pub fn resolve_border_image_in_area(
    area: BorderImageArea,
    request: BorderImagePartitionRequest,
    options: PaintOptions,
) -> BorderImageOutcome {
    let (result, report) = run(
        options,
        OperationKind::BorderImage,
        PaintPhase::BorderImageGeometry,
        Some(|work: &mut GeometryWork<'_, '_, '_>| validate_partition(work, request)),
        |work| resolve_geometry(work, request, |_| Ok(area)),
    );
    BorderImageOutcome { result, report }
}

/// Resolves slices, widths and outsets in a single budgeted operation for a concrete source.
pub fn resolve_border_image(
    request: BorderImageRequest,
    options: PaintOptions,
) -> BorderImageOutcome {
    let area = BorderImageAreaRequest {
        border_origin: request.border_origin,
        border_size: request.border_size,
        border_widths: request.border_widths,
        outsets: request.outsets,
    };
    let partition = BorderImagePartitionRequest {
        source: request.source,
        slices: request.slices,
        widths: request.widths,
        fill_center: request.fill_center,
    };
    let (result, report) = run(
        options,
        OperationKind::BorderImage,
        PaintPhase::BorderImageGeometry,
        Some(|work: &mut GeometryWork<'_, '_, '_>| {
            validate_box(work, area)?;
            validate_source(work, partition.source)?;
            validate_border_widths(work, area.border_widths)?;
            validate_slices(work, partition.slices)?;
            validate_outsets(work, area.outsets)?;
            validate_widths(work, partition.widths)
        }),
        |work| resolve_geometry(work, partition, |work| resolve_area(work, area)),
    );
    BorderImageOutcome { result, report }
}

fn run<T>(
    options: PaintOptions,
    kind: OperationKind,
    geometry_phase: PaintPhase,
    validate: Option<impl FnOnce(&mut GeometryWork<'_, '_, '_>) -> Result<(), PaintError>>,
    resolve: impl FnOnce(&mut GeometryWork<'_, '_, '_>) -> Result<T, PaintError>,
) -> (Result<T, PaintError>, Option<PaintReport>) {
    run_stages(
        options,
        kind,
        [geometry_phase],
        validate,
        |meter, [scope]| in_geometry(meter, scope, resolve),
    )
}

fn in_geometry<T>(
    meter: &mut Meter,
    scope: Scope,
    resolve: impl FnOnce(&mut GeometryWork<'_, '_, '_>) -> Result<T, PaintError>,
) -> Result<T, PaintError> {
    meter
        .with_phase(scope, |phase| resolve(&mut GeometryWork { phase, scope }))
        .map_err(setup_error)?
}

fn run_stages<T, const N: usize>(
    options: PaintOptions,
    kind: OperationKind,
    geometry_phases: [PaintPhase; N],
    validate: Option<impl FnOnce(&mut GeometryWork<'_, '_, '_>) -> Result<(), PaintError>>,
    resolve: impl FnOnce(&mut Meter, [Scope; N]) -> Result<T, PaintError>,
) -> (Result<T, PaintError>, Option<PaintReport>) {
    let mut meter = match options.operation(kind) {
        Ok(meter) => meter,
        Err(error) => return (Err(error), None),
    };
    let result = (|| {
        let root = meter.root();
        let validation = validate
            .is_some()
            .then(|| {
                meter.child(
                    root,
                    PaintPhase::PaintValidation,
                    Limits::unlimited(),
                    Tariffs::identity(),
                )
            })
            .transpose()?;
        let mut scopes = [root; N];
        for (slot, phase) in scopes.iter_mut().zip(geometry_phases) {
            *slot = meter.child(root, phase, Limits::unlimited(), Tariffs::identity())?;
        }
        let publication = meter.child(
            root,
            PaintPhase::Publication,
            Limits::unlimited(),
            Tariffs::identity(),
        )?;
        let cleanup = meter.child(
            root,
            PaintPhase::Cleanup,
            Limits::unlimited(),
            Tariffs::identity(),
        )?;
        let mut retained = false;
        let result = (|| {
            if let (Some(validation), Some(validate)) = (validation, validate) {
                meter
                    .with_phase(validation, |phase| {
                        validate(&mut GeometryWork {
                            phase,
                            scope: validation,
                        })
                    })
                    .map_err(setup_error)??;
            }
            let result = resolve(&mut meter, scopes)?;
            meter
                .with_phase(publication, |phase| {
                    apply(
                        phase,
                        root,
                        Change::Acquire(ResourceKind::RetainedResultBytes, bytes::<T>(1)?),
                    )?;
                    retained = true;
                    charge(phase, publication, WorkKind::OutputRecord, 1)?;
                    Ok(result)
                })
                .map_err(setup_error)?
        })();
        let release = meter
            .with_phase(cleanup, |phase| {
                if result.is_err() && retained {
                    apply(
                        phase,
                        root,
                        Change::Release(ResourceKind::RetainedResultBytes, bytes::<T>(1)?),
                    )?;
                }
                Ok(())
            })
            .map_err(setup_error)
            .and_then(|done| done);
        result.and_then(|value| release.map(|()| value))
    })();
    finish(meter, result)
}

fn validate_partition(
    work: &mut GeometryWork<'_, '_, '_>,
    request: BorderImagePartitionRequest,
) -> Result<(), PaintError> {
    validate_source(work, request.source)?;
    validate_slices(work, request.slices)?;
    validate_widths(work, request.widths)
}

fn validate_box(
    work: &mut GeometryWork<'_, '_, '_>,
    request: BorderImageAreaRequest,
) -> Result<(), PaintError> {
    for value in [request.border_origin.x, request.border_origin.y] {
        work.check(value, f64::is_finite, BorderImageGeometryError::InvalidBox)?;
    }
    for value in [request.border_size.width, request.border_size.height] {
        work.check(value, nonnegative, BorderImageGeometryError::InvalidBox)?;
    }
    Ok(())
}

fn validate_source(
    work: &mut GeometryWork<'_, '_, '_>,
    source: BorderImageSource,
) -> Result<(), PaintError> {
    for value in [source.coordinate_size.width, source.coordinate_size.height] {
        work.check(
            value,
            |value| value.is_finite() && value > 0.0,
            BorderImageGeometryError::InvalidSource,
        )?;
    }
    for value in [source.natural_size.width, source.natural_size.height]
        .into_iter()
        .flatten()
    {
        work.check(value, nonnegative, BorderImageGeometryError::InvalidSource)?;
    }
    Ok(())
}

fn validate_border_widths(
    work: &mut GeometryWork<'_, '_, '_>,
    widths: Edges<f64>,
) -> Result<(), PaintError> {
    for value in physical_edges(widths) {
        work.check(
            value,
            nonnegative,
            BorderImageGeometryError::InvalidBorderWidth,
        )?;
    }
    Ok(())
}

fn validate_slices(
    work: &mut GeometryWork<'_, '_, '_>,
    slices: Edges<BorderImageSlice>,
) -> Result<(), PaintError> {
    for slice in physical_edges(slices) {
        let (BorderImageSlice::SourceUnits(value) | BorderImageSlice::Fraction(value)) = slice;
        work.check(value, nonnegative, BorderImageGeometryError::InvalidSlice)?;
    }
    Ok(())
}

fn validate_outsets(
    work: &mut GeometryWork<'_, '_, '_>,
    outsets: Edges<BorderImageOutset>,
) -> Result<(), PaintError> {
    for outset in physical_edges(outsets) {
        let (BorderImageOutset::Length(value) | BorderImageOutset::BorderWidthMultiple(value)) =
            outset;
        work.check(value, nonnegative, BorderImageGeometryError::InvalidOutset)?;
    }
    Ok(())
}

fn validate_widths(
    work: &mut GeometryWork<'_, '_, '_>,
    widths: Edges<BorderImageWidth>,
) -> Result<(), PaintError> {
    for width in physical_edges(widths) {
        if let BorderImageWidth::BorderWidthMultiple(value) = width {
            work.check(value, nonnegative, BorderImageGeometryError::InvalidWidth)?;
        }
    }
    Ok(())
}

fn physical_edges<T>(edges: Edges<T>) -> [T; 4] {
    [edges.top, edges.right, edges.bottom, edges.left]
}

fn nonnegative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn geometry_error() -> PaintError {
    PaintError::BorderImage(BorderImageGeometryError::UnrepresentableGeometry)
}

struct GeometryWork<'phase, 'borrow, 'operation> {
    phase: &'phase mut PhaseMeter<'borrow, 'operation>,
    scope: Scope,
}
impl GeometryWork<'_, '_, '_> {
    fn check(
        &mut self,
        value: f64,
        valid: fn(f64) -> bool,
        error: BorderImageGeometryError,
    ) -> Result<(), PaintError> {
        charge(self.phase, self.scope, WorkKind::BorderImageValidation, 1)?;
        if valid(value) {
            Ok(())
        } else {
            Err(PaintError::BorderImage(error))
        }
    }

    fn coordinate<T>(&mut self, action: impl FnOnce() -> T) -> Result<T, PaintError> {
        charge(self.phase, self.scope, WorkKind::BorderImageCoordinate, 1)?;
        Ok(action())
    }

    fn finite(&mut self, value: f64) -> Result<f64, PaintError> {
        self.coordinate(|| value.is_finite())?
            .then_some(value)
            .ok_or_else(geometry_error)
    }

    fn product(&mut self, left: f64, right: f64) -> Result<f64, PaintError> {
        let result = self.coordinate(|| left * right)?;
        self.finite(result)?;
        if self.coordinate(|| result == 0.0)?
            && self.coordinate(|| left > 0.0)?
            && self.coordinate(|| right > 0.0)?
        {
            return Err(geometry_error());
        }
        Ok(result)
    }
}

type Axis = [(f64, f64); 3];

#[derive(Clone, Copy)]
struct SourceSlice {
    distance: f64,
    fraction: Option<f64>,
}

#[derive(Clone, Copy)]
struct AxisWidths {
    extent: f64,
    first: f64,
    last: f64,
}

fn resolve_area(
    work: &mut GeometryWork<'_, '_, '_>,
    request: BorderImageAreaRequest,
) -> Result<BorderImageArea, PaintError> {
    let input = request.outsets;
    let border = request.border_widths;
    let outsets = Edges::new(
        outset(work, input.top, border.top)?,
        outset(work, input.right, border.right)?,
        outset(work, input.bottom, border.bottom)?,
        outset(work, input.left, border.left)?,
    );
    let (x0, x1, width) = area_axis(
        work,
        request.border_origin.x,
        request.border_size.width,
        outsets.left,
        outsets.right,
    )?;
    let (y0, y1, height) = area_axis(
        work,
        request.border_origin.y,
        request.border_size.height,
        outsets.top,
        outsets.bottom,
    )?;
    Ok(BorderImageArea {
        rect: BorderImageRect {
            min: Point::new(x0, y0),
            max: Point::new(x1, y1),
        },
        extent: Size::new(width, height),
        border_widths: border,
    })
}

fn resolve_geometry(
    work: &mut GeometryWork<'_, '_, '_>,
    request: BorderImagePartitionRequest,
    area: impl FnOnce(&mut GeometryWork<'_, '_, '_>) -> Result<BorderImageArea, PaintError>,
) -> Result<BorderImageGeometry, PaintError> {
    let source = request.source.coordinate_size;
    let natural = request.source.natural_size;
    let input = request.slices;
    let slices = Edges::new(
        slice(work, input.top, source.height)?,
        slice(work, input.right, source.width)?,
        slice(work, input.bottom, source.height)?,
        slice(work, input.left, source.width)?,
    );
    let area = area(work)?;
    let border = area.border_widths;
    let Size { width, height } = area.extent;
    let Point { x: x0, y: y0 } = area.rect.min();
    let Point { x: x1, y: y1 } = area.rect.max();
    let input = request.widths;
    let widths = Edges::new(
        used_width(
            work,
            input.top,
            border.top,
            height,
            slices.top,
            source.height,
            natural.height,
        )?,
        used_width(
            work,
            input.right,
            border.right,
            width,
            slices.right,
            source.width,
            natural.width,
        )?,
        used_width(
            work,
            input.bottom,
            border.bottom,
            height,
            slices.bottom,
            source.height,
            natural.height,
        )?,
        used_width(
            work,
            input.left,
            border.left,
            width,
            slices.left,
            source.width,
            natural.width,
        )?,
    );
    let x = AxisWidths {
        extent: width,
        first: widths.left,
        last: widths.right,
    };
    let y = AxisWidths {
        extent: height,
        first: widths.top,
        last: widths.bottom,
    };
    let x_fit = compare_opposing(work, x.extent, x.first, x.last)?;
    let y_fit = compare_opposing(work, y.extent, y.first, y.last)?;
    let comparison = match (x_fit, y_fit) {
        (Ordering::Greater, Ordering::Greater) => compare_fit_ratios(work, x, y)?,
        (Ordering::Greater, _) => Ordering::Less,
        (_, Ordering::Greater) => Ordering::Greater,
        _ => Ordering::Equal,
    };
    let selected = if comparison == Ordering::Greater {
        y
    } else {
        x
    };
    let collapse_x = work.coordinate(|| width == 0.0)?
        || (x_fit != Ordering::Less && comparison != Ordering::Greater);
    let collapse_y = work.coordinate(|| height == 0.0)?
        || (y_fit != Ordering::Less && comparison != Ordering::Less);
    let extent = Size::new(width, height);
    let collapse = Size::new(collapse_x, collapse_y);
    let mut factor = if x_fit == Ordering::Greater || y_fit == Ordering::Greater {
        fit_factor(work, selected)?
    } else {
        1.0
    };
    let mut used_widths = scale_widths(work, widths, factor, extent, collapse)?;
    if widths_overlap(work, used_widths, extent)? {
        // Multiplying the two sides rounds them independently. Use the next
        // smaller common factor if the published widths still overlap.
        factor = work.coordinate(|| factor.next_down())?;
        if work.coordinate(|| factor <= 0.0)? {
            return Err(geometry_error());
        }
        used_widths = scale_widths(work, widths, factor, extent, collapse)?;
        if widths_overlap(work, used_widths, extent)? {
            return Err(geometry_error());
        }
    }
    let source_x = source_axis(work, source.width, slices.left, slices.right)?;
    let source_y = source_axis(work, source.height, slices.top, slices.bottom)?;
    let dest_x = destination_axis(
        work,
        x0,
        x1,
        used_widths.left,
        used_widths.right,
        collapse_x,
    )?;
    let dest_y = destination_axis(
        work,
        y0,
        y1,
        used_widths.top,
        used_widths.bottom,
        collapse_y,
    )?;
    let mut regions = [[BorderImageRegion::EMPTY; 3]; 3];
    for (row_index, (row, (sy, dy))) in regions
        .iter_mut()
        .zip(source_y.into_iter().zip(dest_y))
        .enumerate()
    {
        for (column_index, (region, (sx, dx))) in row
            .iter_mut()
            .zip(source_x.into_iter().zip(dest_x))
            .enumerate()
        {
            let source = if row_index == 1 && column_index == 1 && !request.fill_center {
                None
            } else {
                region_rect(work, sx, sy)?
            };
            let destination = region_rect(work, dx, dy)?;
            charge(work.phase, work.scope, WorkKind::OutputRecord, 1)?;
            *region = BorderImageRegion {
                source,
                destination,
            };
        }
    }
    Ok(BorderImageGeometry {
        area: area.rect,
        used_widths,
        regions,
    })
}

fn slice(
    work: &mut GeometryWork<'_, '_, '_>,
    slice: BorderImageSlice,
    extent: f64,
) -> Result<SourceSlice, PaintError> {
    work.coordinate(|| ())?;
    match slice {
        BorderImageSlice::SourceUnits(value) => Ok(SourceSlice {
            distance: work.coordinate(|| value.min(extent))?,
            fraction: None,
        }),
        BorderImageSlice::Fraction(fraction) => {
            let fraction = work.coordinate(|| fraction.min(1.0))?;
            Ok(SourceSlice {
                distance: work.product(fraction, extent)?,
                fraction: Some(fraction),
            })
        }
    }
}

fn outset(
    work: &mut GeometryWork<'_, '_, '_>,
    outset: BorderImageOutset,
    border: f64,
) -> Result<f64, PaintError> {
    work.coordinate(|| ())?;
    match outset {
        BorderImageOutset::Length(value) => Ok(value),
        BorderImageOutset::BorderWidthMultiple(multiple) => work.product(multiple, border),
    }
}

fn area_axis(
    work: &mut GeometryWork<'_, '_, '_>,
    start: f64,
    size: f64,
    first: f64,
    last: f64,
) -> Result<(f64, f64, f64), PaintError> {
    let end = work.coordinate(|| start + size)?;
    let min = work.coordinate(|| start - first)?;
    let max = work.coordinate(|| end + last)?;
    let used_extent = work.coordinate(|| size + first)?;
    let used_extent = work.coordinate(|| used_extent + last)?;
    work.finite(used_extent)?;
    let represented_extent = work.coordinate(|| max - min)?;
    work.finite(represented_extent)?;
    if (work.coordinate(|| size > 0.0)? && work.coordinate(|| start == end)?)
        || (work.coordinate(|| first > 0.0)? && work.coordinate(|| min == start)?)
        || (work.coordinate(|| last > 0.0)? && work.coordinate(|| max == end)?)
    {
        return Err(geometry_error());
    }
    Ok((min, max, used_extent))
}

fn used_width(
    work: &mut GeometryWork<'_, '_, '_>,
    width: BorderImageWidth,
    border: f64,
    area: f64,
    slice: SourceSlice,
    source: f64,
    natural: Option<f64>,
) -> Result<f64, PaintError> {
    work.coordinate(|| ())?;
    match width {
        BorderImageWidth::Auto => {
            let Some(natural) = natural else {
                return Ok(border);
            };
            let fraction = match slice.fraction {
                Some(fraction) => fraction,
                None => work.coordinate(|| slice.distance / source)?,
            };
            if work.coordinate(|| fraction == 0.0)?
                && work.coordinate(|| slice.distance > 0.0)?
                && work.coordinate(|| natural > 0.0)?
            {
                return Err(geometry_error());
            }
            work.product(fraction, natural)
        }
        BorderImageWidth::LengthPercentage(value) => {
            let width =
                work.coordinate(|| value.percent_fraction().mul_add(area, value.absolute_px()))?;
            let width = work.finite(width)?;
            work.coordinate(|| width.max(0.0))
        }
        BorderImageWidth::BorderWidthMultiple(multiple) => work.product(multiple, border),
    }
}

fn compare_opposing(
    work: &mut GeometryWork<'_, '_, '_>,
    extent: f64,
    first: f64,
    last: f64,
) -> Result<Ordering, PaintError> {
    let (larger, smaller) = if work.coordinate(|| first >= last)? {
        (first, last)
    } else {
        (last, first)
    };
    // Near equality the larger offset is at least half the extent, so the
    // subtraction retains the difference without rounding the opposing sum.
    let remaining = work.coordinate(|| extent - larger)?;
    work.coordinate(|| smaller.partial_cmp(&remaining))?
        .ok_or_else(geometry_error)
}

fn fit_factor(work: &mut GeometryWork<'_, '_, '_>, axis: AxisWidths) -> Result<f64, PaintError> {
    let AxisWidths {
        extent,
        first,
        last,
    } = axis;
    let sum = work.coordinate(|| first + last)?;
    let (scaled_extent, first, last, sum) = if work.coordinate(|| sum.is_finite())? {
        (extent, first, last, sum)
    } else {
        // Halving both terms avoids overflow in a finite opposing pair.
        let half_extent = work.coordinate(|| extent * 0.5)?;
        let half_first = work.coordinate(|| first * 0.5)?;
        let half_last = work.coordinate(|| last * 0.5)?;
        let half_sum = work.coordinate(|| half_first + half_last)?;
        (half_extent, half_first, half_last, half_sum)
    };
    // Retain the error of the rounded opposing sum when computing the factor.
    let (larger, smaller) = if work.coordinate(|| first >= last)? {
        (first, last)
    } else {
        (last, first)
    };
    let rounded_smaller = work.coordinate(|| sum - larger)?;
    let sum_tail = work.coordinate(|| smaller - rounded_smaller)?;
    let quotient = work.coordinate(|| scaled_extent / sum)?;
    let residual = work.coordinate(|| (-quotient).mul_add(sum, scaled_extent))?;
    let residual = work.coordinate(|| (-quotient).mul_add(sum_tail, residual))?;
    let adjustment = work.coordinate(|| residual / sum)?;
    let factor = work.coordinate(|| quotient + adjustment)?;
    if work.coordinate(|| factor == 0.0)? && work.coordinate(|| extent > 0.0)? {
        return Err(geometry_error());
    }
    if work.coordinate(|| factor == 1.0)? {
        return work.coordinate(|| factor.next_down());
    }
    Ok(factor)
}

// A product of two finite, nonnegative binary64 numbers has at most 106
// significand bits. Keeping its binary exponent separate avoids overflow and
// preserves contributions too small to affect a rounded opposing sum.
struct ExactProduct {
    significand: u128,
    exponent: i32,
}

impl ExactProduct {
    const ZERO: Self = Self {
        significand: 0,
        exponent: 0,
    };

    fn new(work: &mut GeometryWork<'_, '_, '_>, left: f64, right: f64) -> Result<Self, PaintError> {
        let (left, left_exponent) = exact_parts(work, left)?;
        let (right, right_exponent) = exact_parts(work, right)?;
        Ok(Self {
            significand: work.coordinate(|| left * right)?,
            exponent: work.coordinate(|| left_exponent + right_exponent)?,
        })
    }

    fn integer_multiple(
        work: &mut GeometryWork<'_, '_, '_>,
        value: f64,
        count: u64,
    ) -> Result<Self, PaintError> {
        let (significand, exponent) = exact_parts(work, value)?;
        // A binary64 significand uses at most 53 bits and count at most 64.
        let count = work.coordinate(|| u128::from(count))?;
        let significand = work.coordinate(|| significand * count)?;
        Ok(Self {
            significand,
            exponent,
        })
    }

    fn word_at(
        &self,
        work: &mut GeometryWork<'_, '_, '_>,
        exponent: i32,
    ) -> Result<u128, PaintError> {
        let shift = work.coordinate(|| exponent - self.exponent)?;
        let distance = work.coordinate(|| shift.unsigned_abs())?;
        let aligned = if work.coordinate(|| shift < 0)? {
            work.coordinate(|| self.significand.checked_shl(distance))?
        } else {
            work.coordinate(|| self.significand.checked_shr(distance))?
        };
        work.coordinate(|| u128::from(aligned.unwrap_or_default() as u64))
    }
}

fn exact_parts(work: &mut GeometryWork<'_, '_, '_>, value: f64) -> Result<(u128, i32), PaintError> {
    let bits = work.coordinate(|| value.to_bits())?;
    let exponent = work.coordinate(|| ((bits >> 52) & 0x7ff) as i32)?;
    let fraction = work.coordinate(|| u128::from(bits & ((1_u64 << 52) - 1)))?;
    if work.coordinate(|| exponent == 0)? {
        return Ok((fraction, -1074));
    }
    let significand = work.coordinate(|| fraction | (1_u128 << 52))?;
    let exponent = work.coordinate(|| exponent - 1075)?;
    Ok((significand, exponent))
}

fn compare_fit_ratios(
    work: &mut GeometryWork<'_, '_, '_>,
    x: AxisWidths,
    y: AxisWidths,
) -> Result<Ordering, PaintError> {
    // Compare x.extent / (x.first + x.last) with the corresponding y
    // ratio by cross multiplication, distributing before the exact products.
    let x_first = ExactProduct::new(work, x.extent, y.first)?;
    let x_last = ExactProduct::new(work, x.extent, y.last)?;
    let y_first = ExactProduct::new(work, y.extent, x.first)?;
    let y_last = ExactProduct::new(work, y.extent, x.last)?;
    compare_product_sums(work, [x_first, x_last], [y_first, y_last])
}

fn compare_product_sums(
    work: &mut GeometryWork<'_, '_, '_>,
    [x_first, x_last]: [ExactProduct; 2],
    [y_first, y_last]: [ExactProduct; 2],
) -> Result<Ordering, PaintError> {
    let mut start = i32::MAX;
    let mut end = i32::MIN;
    for term in [&x_first, &x_last, &y_first, &y_last] {
        if work.coordinate(|| term.significand != 0)? {
            start = work.coordinate(|| start.min(term.exponent))?;
            // Include a possible carry above the product.
            let leading = work.coordinate(|| term.significand.leading_zeros())?;
            let bits_with_carry = work.coordinate(|| 129 - leading as i32)?;
            let top = work.coordinate(|| term.exponent + bits_with_carry)?;
            end = work.coordinate(|| end.max(top))?;
        }
    }

    // Walk low to high so each sum can carry into its next word. The highest
    // differing word determines the order. Binary64 bounds this to 66 words.
    let mut exponent = start;
    let mut x_carry = 0;
    let mut y_carry = 0;
    let mut order = Ordering::Equal;
    while work.coordinate(|| exponent < end)? {
        let first = x_first.word_at(work, exponent)?;
        let last = x_last.word_at(work, exponent)?;
        let sum = work.coordinate(|| first + last)?;
        let x_word = work.coordinate(|| sum + x_carry)?;
        x_carry = work.coordinate(|| x_word >> 64)?;
        let first = y_first.word_at(work, exponent)?;
        let last = y_last.word_at(work, exponent)?;
        let sum = work.coordinate(|| first + last)?;
        let y_word = work.coordinate(|| sum + y_carry)?;
        y_carry = work.coordinate(|| y_word >> 64)?;
        let word_order = work.coordinate(|| (x_word as u64).cmp(&(y_word as u64)))?;
        if word_order != Ordering::Equal {
            order = word_order;
        }
        exponent = work.coordinate(|| exponent + 64)?;
    }
    Ok(order)
}

fn scale_widths(
    work: &mut GeometryWork<'_, '_, '_>,
    widths: Edges<f64>,
    factor: f64,
    extent: Size<f64>,
    collapse: Size<bool>,
) -> Result<Edges<f64>, PaintError> {
    Ok(Edges::new(
        fitted_width(
            work,
            widths.top,
            widths.bottom,
            factor,
            extent.height,
            collapse.height,
        )?,
        fitted_width(
            work,
            widths.right,
            widths.left,
            factor,
            extent.width,
            collapse.width,
        )?,
        fitted_width(
            work,
            widths.bottom,
            widths.top,
            factor,
            extent.height,
            collapse.height,
        )?,
        fitted_width(
            work,
            widths.left,
            widths.right,
            factor,
            extent.width,
            collapse.width,
        )?,
    ))
}

fn widths_overlap(
    work: &mut GeometryWork<'_, '_, '_>,
    widths: Edges<f64>,
    extent: Size<f64>,
) -> Result<bool, PaintError> {
    Ok(
        compare_opposing(work, extent.width, widths.left, widths.right)? == Ordering::Greater
            || compare_opposing(work, extent.height, widths.top, widths.bottom)?
                == Ordering::Greater,
    )
}

fn fitted_width(
    work: &mut GeometryWork<'_, '_, '_>,
    width: f64,
    opposite: f64,
    factor: f64,
    extent: f64,
    fills: bool,
) -> Result<f64, PaintError> {
    if fills && work.coordinate(|| opposite == 0.0)? {
        return Ok(extent);
    }
    work.product(width, factor)
}

fn mixed_slices_fill(
    work: &mut GeometryWork<'_, '_, '_>,
    extent: f64,
    fraction: f64,
    units: f64,
) -> Result<bool, PaintError> {
    if work.coordinate(|| fraction == 1.0)? || work.coordinate(|| units == extent)? {
        return Ok(true);
    }
    let excess = if work.coordinate(|| fraction >= 0.5)? {
        // The complement is exact here. Retain the sign through the multiply
        // and comparison instead of rounding the absolute slice first.
        let remainder = work.coordinate(|| 1.0 - fraction)?;
        work.coordinate(|| (-extent).mul_add(remainder, units))?
    } else {
        let half = work.coordinate(|| extent * 0.5)?;
        if work.coordinate(|| units < half)? {
            return Ok(false);
        }
        let rest = work.coordinate(|| units - extent)?;
        work.coordinate(|| fraction.mul_add(extent, rest))?
    };
    work.coordinate(|| !excess.is_sign_negative())
}

fn source_axis(
    work: &mut GeometryWork<'_, '_, '_>,
    extent: f64,
    first: SourceSlice,
    last: SourceSlice,
) -> Result<Axis, PaintError> {
    let fills = match (first.fraction, last.fraction) {
        (Some(first), Some(last)) => compare_opposing(work, 1.0, first, last)? != Ordering::Less,
        (Some(fraction), None) => mixed_slices_fill(work, extent, fraction, last.distance)?,
        (None, Some(fraction)) => mixed_slices_fill(work, extent, fraction, first.distance)?,
        (None, None) => {
            compare_opposing(work, extent, first.distance, last.distance)? != Ordering::Less
        }
    };
    let cut = work.coordinate(|| extent - last.distance)?;
    if work.coordinate(|| last.distance > 0.0)? && work.coordinate(|| cut == extent)? {
        return Err(geometry_error());
    }
    let middle_end = if fills {
        first.distance
    } else {
        if work.coordinate(|| cut <= first.distance)? {
            return Err(geometry_error());
        }
        cut
    };
    Ok([
        (0.0, first.distance),
        (first.distance, middle_end),
        (cut, extent),
    ])
}

fn destination_axis(
    work: &mut GeometryWork<'_, '_, '_>,
    min: f64,
    max: f64,
    first: f64,
    last: f64,
    collapse_middle: bool,
) -> Result<Axis, PaintError> {
    let (first_cut, last_cut) = if collapse_middle && work.coordinate(|| last == 0.0)? {
        (max, max)
    } else {
        let first_cut = work.coordinate(|| min + first)?;
        let last_cut = if collapse_middle {
            first_cut
        } else {
            let cut = work.coordinate(|| max - last)?;
            if work.coordinate(|| cut <= first_cut)? {
                return Err(geometry_error());
            }
            cut
        };
        (first_cut, last_cut)
    };
    if (work.coordinate(|| first > 0.0)? && work.coordinate(|| min >= first_cut)?)
        || (work.coordinate(|| last > 0.0)? && work.coordinate(|| last_cut >= max)?)
    {
        return Err(geometry_error());
    }
    Ok([(min, first_cut), (first_cut, last_cut), (last_cut, max)])
}

fn region_rect(
    work: &mut GeometryWork<'_, '_, '_>,
    x: (f64, f64),
    y: (f64, f64),
) -> Result<Option<BorderImageRect>, PaintError> {
    if work.coordinate(|| x.0 < x.1)? && work.coordinate(|| y.0 < y.1)? {
        Ok(Some(BorderImageRect {
            min: Point::new(x.0, y.0),
            max: Point::new(x.1, y.1),
        }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PaintChange, PaintLimits, ProfileMode};

    fn options(limits: PaintLimits) -> PaintOptions {
        PaintOptions::new(limits, ProfileMode::Summary { timing: false })
    }

    fn request() -> BorderImageRequest {
        use BorderImageOutset::{BorderWidthMultiple as Multiple, Length};
        use BorderImageSlice::{Fraction, SourceUnits};

        BorderImageRequest {
            border_origin: Point::new(10.0, 20.0),
            border_size: Size::new(40.0, 20.0),
            border_widths: Edges::new(2.0, 3.0, 4.0, 5.0),
            source: BorderImageSource {
                coordinate_size: Size::new(100.0, 80.0),
                natural_size: Size::new(Some(50.0), None),
            },
            slices: Edges::new(
                Fraction(0.25),
                SourceUnits(10.0),
                SourceUnits(30.0),
                SourceUnits(20.0),
            ),
            widths: Edges::new(
                BorderImageWidth::Auto,
                BorderImageWidth::LengthPercentage(
                    LengthPercentageOf::from_percent_fraction(0.25).unwrap(),
                ),
                BorderImageWidth::BorderWidthMultiple(2.0),
                BorderImageWidth::Auto,
            ),
            outsets: Edges::new(Multiple(2.0), Length(4.0), Length(6.0), Multiple(2.0)),
            fill_center: true,
        }
    }

    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> BorderImageRect {
        BorderImageRect {
            min: Point::new(x0, y0),
            max: Point::new(x1, y1),
        }
    }

    fn geometry(request: BorderImageRequest) -> BorderImageGeometry {
        resolve_border_image(request, options(PaintLimits::unlimited()))
            .result
            .unwrap()
    }

    #[test]
    fn staged_area_supplies_the_width_basis_without_repeating_work() {
        let input = request();
        let area_outcome = resolve_border_image_area(
            BorderImageAreaRequest {
                border_origin: input.border_origin,
                border_size: input.border_size,
                border_widths: input.border_widths,
                outsets: input.outsets,
            },
            options(PaintLimits::unlimited()),
        );
        let area = area_outcome.result.unwrap();
        assert_eq!(area.rect(), rect(0.0, 16.0, 54.0, 46.0));
        assert_eq!(area.default_size(), Size::new(54.0, 30.0));

        let partition = BorderImagePartitionRequest {
            source: input.source,
            slices: input.slices,
            widths: input.widths,
            fill_center: input.fill_center,
        };
        let refused = resolve_border_image_in_area(
            area,
            partition,
            options(PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, 9)),
        );
        let Err(PaintError::Budget(refusal)) = refused.result else {
            panic!("expected partition publication refusal");
        };
        assert_eq!(refusal.origin, PaintPhase::Publication);
        assert_eq!(
            refused
                .report
                .unwrap()
                .live(ResourceKind::RetainedResultBytes),
            0
        );

        let staged =
            resolve_border_image_in_area(area, partition, options(PaintLimits::unlimited()));
        let combined = resolve_border_image(input, options(PaintLimits::unlimited()));
        assert_eq!(staged.result, combined.result);
        assert_eq!(
            staged.result.unwrap().used_widths(),
            Edges::new(2.0, 13.5, 8.0, 10.0)
        );
        let area_report = area_outcome.report.unwrap();
        let staged_report = staged.report.unwrap();
        let combined_report = combined.report.unwrap();
        assert_eq!(area_report.work(WorkKind::BorderImageValidation), 12);
        for kind in [
            WorkKind::BorderImageValidation,
            WorkKind::BorderImageCoordinate,
        ] {
            assert_eq!(
                area_report.work(kind) + staged_report.work(kind),
                combined_report.work(kind)
            );
        }
        assert_eq!(
            area_report
                .phases()
                .find(|phase| phase.phase() == PaintPhase::BorderImageArea)
                .unwrap()
                .work(WorkKind::BorderImageCoordinate),
            area_report.work(WorkKind::BorderImageCoordinate)
        );
        assert_eq!(area_report.work(WorkKind::OutputRecord), 1);
        assert_eq!(staged_report.work(WorkKind::OutputRecord), 10);
        assert_eq!(combined_report.work(WorkKind::OutputRecord), 10);
        assert_eq!(area_report.live(ResourceKind::ScratchBytes), 0);
        assert_eq!(
            area_report.live(ResourceKind::RetainedResultBytes),
            size_of::<BorderImageArea>() as u64
        );
    }

    #[test]
    fn area_refusal_reports_its_phase_and_releases_provisional_output() {
        let mut input = BorderImageAreaRequest {
            border_origin: Point::new(0.1, 0.0),
            border_size: Size::new(0.2, 10.0),
            border_widths: Edges::all(1.0),
            outsets: Edges::all(BorderImageOutset::Length(0.0)),
        };
        for (kind, cap, phase) in [
            (
                WorkKind::BorderImageValidation,
                0,
                PaintPhase::PaintValidation,
            ),
            (
                WorkKind::BorderImageCoordinate,
                1,
                PaintPhase::BorderImageArea,
            ),
            (WorkKind::OutputRecord, 0, PaintPhase::Publication),
        ] {
            let limited = options(PaintLimits::unlimited().with_work_limit(kind, cap));
            let outcome = resolve_border_image_area(input, limited);
            let Err(PaintError::Budget(refusal)) = outcome.result else {
                panic!(
                    "expected a budget refusal for {kind:?}: {:?}",
                    outcome.result
                );
            };
            assert_eq!(refusal.origin, phase);
            assert_eq!(refusal.current, cap);
            assert_eq!(refusal.change, PaintChange::Work { kind, amount: 1 });
            let report = outcome.report.unwrap();
            assert_eq!(report.work(kind), cap);
            assert_eq!(report.live(ResourceKind::RetainedResultBytes), 0);
            if kind == WorkKind::OutputRecord {
                assert_eq!(
                    report.peak(ResourceKind::RetainedResultBytes),
                    size_of::<BorderImageArea>() as u64
                );
            }
        }
        let retried = resolve_border_image_area(input, options(PaintLimits::unlimited()))
            .result
            .unwrap();
        assert_eq!(retried.default_size(), input.border_size);
        assert_ne!(retried.rect().max().x - retried.rect().min().x, 0.2);

        input.outsets.left = BorderImageOutset::Length(-1.0);
        assert_eq!(
            resolve_border_image_area(input, options(PaintLimits::unlimited())).result,
            Err(PaintError::BorderImage(
                BorderImageGeometryError::InvalidOutset
            ))
        );
        input.outsets.left = BorderImageOutset::Length(0.0);
        input.border_origin.x = f64::MAX;
        assert_eq!(
            resolve_border_image_area(input, options(PaintLimits::unlimited())).result,
            Err(geometry_error())
        );
    }

    fn partition_request(x: f64, extent: f64, left: f64, right: f64) -> BorderImageRequest {
        let px = |value| BorderImageWidth::LengthPercentage(LengthPercentageOf::px(value).unwrap());
        BorderImageRequest {
            border_origin: Point::new(x, 0.0),
            border_size: Size::new(extent, 20.0),
            widths: Edges::new(px(2.0), px(right), px(0.0), px(left)),
            outsets: Edges::all(BorderImageOutset::Length(0.0)),
            ..request()
        }
    }

    #[test]
    fn partition_keeps_zero_width_side_empty_when_opposite_side_is_reduced() {
        let result = geometry(partition_request(0.0, 1.0, 49.0, 0.0));
        assert!(result.regions()[0][2].source().is_some());
        assert_eq!(result.regions()[0][2].destination(), None);
        assert_eq!(result.used_widths().left, 1.0);
        assert_eq!(result.regions()[0][0].destination().unwrap().max().x, 1.0);
    }

    #[test]
    fn partition_keeps_exactly_filled_destination_middle_empty_at_a_fractional_origin() {
        let result = geometry(partition_request(0.1, 1.0, 0.5, 0.5));
        assert!(
            result
                .regions()
                .iter()
                .all(|row| row[1].destination().is_none())
        );
        assert_eq!(
            result.regions()[0][0].destination().unwrap().max().x,
            result.regions()[0][2].destination().unwrap().min().x,
        );
    }

    #[test]
    fn partition_refuses_a_positive_middle_erased_by_destination_placement() {
        let input = partition_request(9_007_199_254_740_992.0, 4.0, 1.75, 1.75);
        let result = resolve_border_image(input, options(PaintLimits::unlimited())).result;
        assert_eq!(
            result,
            Err(PaintError::BorderImage(
                BorderImageGeometryError::UnrepresentableGeometry
            ))
        );
    }

    #[test]
    fn partition_keeps_complementary_percentages_empty_in_source_units() {
        let mut input = request();
        input.source.coordinate_size.width = 0.9;
        input.slices.left = BorderImageSlice::Fraction(0.125);
        input.slices.right = BorderImageSlice::Fraction(0.875);
        let result = geometry(input);
        assert!(result.regions().iter().all(|row| row[1].source().is_none()));
        assert!(result.regions()[0][0].source().is_some());
        assert!(result.regions()[0][2].source().is_some());
    }

    #[test]
    fn partition_preserves_a_representable_gap_below_one_extent_ulp() {
        let gap = f64::EPSILON * 0.25;
        let result = geometry(partition_request(-0.5, 1.0, 0.5, 0.5 - gap));
        assert_eq!(
            result.regions()[0][1].destination(),
            Some(rect(0.0, 0.0, gap, 2.0))
        );
    }

    #[test]
    fn partition_reduces_opposing_widths_whose_sum_rounds_to_the_extent() {
        let result = geometry(partition_request(0.0, 1.0, 0.5, 0.5 + f64::EPSILON * 0.5));
        let widths = result.used_widths();
        assert!(widths.left < 0.5);
        assert!(widths.right <= 1.0 - widths.left);
    }

    #[test]
    fn partition_uses_the_tighter_axis_when_both_opposing_sums_round_to_one() {
        let mut input = partition_request(-0.5, 1.0, 0.5, 0.5 + f64::EPSILON * 0.5);
        input.border_origin.y = -0.5;
        input.border_size.height = 1.0;
        let px = |value| BorderImageWidth::LengthPercentage(LengthPercentageOf::px(value).unwrap());
        input.widths.top = px(0.375 + f64::EPSILON * 0.25);
        input.widths.bottom = px(0.625);
        let result = geometry(input);
        assert!(
            result
                .regions()
                .iter()
                .all(|row| row[1].destination().is_none())
        );
        assert!(result.regions()[1][0].destination().is_some());
    }

    #[test]
    fn partition_collapses_both_axes_at_equal_proportional_fit_ratios() {
        let mut input = partition_request(0.0, 1.0, 0.125, 1.2499999999999996);
        input.border_size.height = 1.5;
        let px = |value| BorderImageWidth::LengthPercentage(LengthPercentageOf::px(value).unwrap());
        // The y extent and both y widths are exactly 1.5 times their x equivalents.
        input.widths.top = px(0.1875);
        input.widths.bottom = px(1.8749999999999993);
        let result = geometry(input);
        for (row_index, row) in result.regions().iter().enumerate() {
            for (column_index, region) in row.iter().enumerate() {
                assert_eq!(
                    region.destination().is_some(),
                    row_index != 1 && column_index != 1
                );
            }
        }
    }

    #[test]
    fn partition_ratio_order_retains_tiny_products_and_carries() {
        let mut meter = options(PaintLimits::unlimited())
            .operation(OperationKind::BorderImage)
            .unwrap();
        let phase = meter
            .child(
                meter.root(),
                PaintPhase::BorderImageGeometry,
                Limits::unlimited(),
                Tariffs::identity(),
            )
            .unwrap();
        let half = AxisWidths {
            extent: 1.0,
            first: 2.0,
            last: 0.0,
        };
        let smallest = f64::from_bits(1);
        let just_below_half = AxisWidths {
            last: smallest,
            ..half
        };
        let large_half = AxisWidths {
            extent: f64::MAX,
            first: f64::MAX,
            last: f64::MAX,
        };
        let tiny = AxisWidths {
            extent: smallest,
            first: f64::MAX,
            last: smallest,
        };
        let tiny_without_last = AxisWidths { last: 0.0, ..tiny };
        for (x, y, expected) in [
            (half, large_half, Ordering::Equal),
            (just_below_half, large_half, Ordering::Less),
            (large_half, just_below_half, Ordering::Greater),
            (tiny, tiny_without_last, Ordering::Less),
        ] {
            let actual = meter
                .with_phase(phase, |phase_meter| {
                    compare_fit_ratios(
                        &mut GeometryWork {
                            phase: phase_meter,
                            scope: phase,
                        },
                        x,
                        y,
                    )
                })
                .unwrap()
                .unwrap();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn partition_keeps_meeting_fraction_and_source_units_empty_in_either_order() {
        let mut input = request();
        input.source.coordinate_size.width = 0.9;
        let fraction = BorderImageSlice::Fraction(0.75);
        let units = BorderImageSlice::SourceUnits(0.225);
        for (left, right) in [(fraction, units), (units, fraction)] {
            input.slices.left = left;
            input.slices.right = right;
            let result = geometry(input);
            assert!(result.regions().iter().all(|row| row[1].source().is_none()));
            assert!(result.regions()[0][0].source().is_some());
            assert!(result.regions()[0][2].source().is_some());
        }
    }

    #[test]
    fn computes_slices_and_outset_area_in_separate_units() {
        let result = geometry(request());
        assert_eq!(result.area(), rect(0.0, 16.0, 54.0, 46.0));
        assert_eq!(result.used_widths(), Edges::new(2.0, 13.5, 8.0, 10.0));
        let sources = [
            [
                rect(0.0, 0.0, 20.0, 20.0),
                rect(20.0, 0.0, 90.0, 20.0),
                rect(90.0, 0.0, 100.0, 20.0),
            ],
            [
                rect(0.0, 20.0, 20.0, 50.0),
                rect(20.0, 20.0, 90.0, 50.0),
                rect(90.0, 20.0, 100.0, 50.0),
            ],
            [
                rect(0.0, 50.0, 20.0, 80.0),
                rect(20.0, 50.0, 90.0, 80.0),
                rect(90.0, 50.0, 100.0, 80.0),
            ],
        ];
        for (row, expected) in result.regions().iter().zip(sources) {
            assert_eq!(row.map(BorderImageRegion::source), expected.map(Some));
        }
        assert_eq!(
            result.regions()[1][1].destination(),
            Some(rect(10.0, 18.0, 40.5, 38.0))
        );
        assert_eq!(
            result.regions()[2][2].destination(),
            Some(rect(40.5, 38.0, 54.0, 46.0))
        );
    }

    #[test]
    fn overlapping_slices_remove_middle_axes_and_keep_independent_corners() {
        let mut input = request();
        input.slices.left = BorderImageSlice::SourceUnits(65.0);
        input.slices.right = BorderImageSlice::Fraction(2.0);
        let horizontal = geometry(input);
        for row in horizontal.regions() {
            assert_eq!(row[1].source(), None);
            assert!(row[1].destination().is_some());
        }
        assert_eq!(
            horizontal.regions()[0][0].source(),
            Some(rect(0.0, 0.0, 65.0, 20.0))
        );
        assert_eq!(
            horizontal.regions()[0][2].source(),
            Some(rect(0.0, 0.0, 100.0, 20.0))
        );
        input.slices.bottom = BorderImageSlice::SourceUnits(60.0);
        let both = geometry(input);
        assert!(
            both.regions()[1]
                .iter()
                .all(|region| region.source().is_none())
        );
        assert!(both.regions()[2][2].source().is_some());

        let without_fill = geometry(BorderImageRequest {
            fill_center: false,
            ..request()
        });
        assert_eq!(without_fill.regions()[1][1].source(), None);
        assert!(without_fill.regions()[1][0].source().is_some());
    }

    #[test]
    fn one_factor_fits_all_destination_widths_and_empty_boxes_stay_empty() {
        let px = |value| BorderImageWidth::LengthPercentage(LengthPercentageOf::px(value).unwrap());
        let mut input = BorderImageRequest {
            outsets: Edges::all(BorderImageOutset::Length(0.0)),
            widths: Edges::new(px(12.0), px(50.0), px(8.0), px(30.0)),
            ..request()
        };
        let result = geometry(input);
        assert_eq!(result.used_widths(), Edges::new(6.0, 25.0, 4.0, 15.0));
        assert_eq!(
            result.regions()[0][0].destination(),
            Some(rect(10.0, 20.0, 25.0, 26.0))
        );
        assert_eq!(
            result.regions()[0][2].destination(),
            Some(rect(25.0, 20.0, 50.0, 26.0))
        );
        assert!(
            result
                .regions()
                .iter()
                .all(|row| row[1].destination().is_none())
        );
        input.widths.top = px(72.0);
        let vertical = geometry(input);
        assert_eq!(vertical.used_widths(), Edges::new(18.0, 12.5, 2.0, 7.5));
        assert!(
            vertical.regions()[1]
                .iter()
                .all(|region| region.destination().is_none())
        );
        input.border_size.width = 0.0;
        let empty = geometry(input);
        assert_eq!(empty.used_widths(), Edges::all(0.0));
        assert!(
            empty
                .regions()
                .iter()
                .flatten()
                .all(|region| region.destination().is_none())
        );
    }

    #[test]
    fn resolves_mixed_widths_and_avoids_overflow_in_the_opposing_sum() {
        let px = |value| BorderImageWidth::LengthPercentage(LengthPercentageOf::px(value).unwrap());
        let mut input = BorderImageRequest {
            border_origin: Point::new(0.0, 0.0),
            outsets: Edges::all(BorderImageOutset::Length(0.0)),
            widths: Edges::new(px(0.0), px(f64::MAX), px(0.0), px(f64::MAX)),
            ..request()
        };
        assert_eq!(
            geometry(input).used_widths(),
            Edges::new(0.0, 20.0, 0.0, 20.0)
        );
        input.widths.right = BorderImageWidth::LengthPercentage(
            LengthPercentageOf::from_coefficients(-5.0, 0.25).unwrap(),
        );
        input.widths.left = BorderImageWidth::LengthPercentage(
            LengthPercentageOf::from_coefficients(-15.0, 0.25).unwrap(),
        );
        assert_eq!(
            geometry(input).used_widths(),
            Edges::new(0.0, 5.0, 0.0, 0.0)
        );
    }

    #[test]
    fn reports_validation_and_coordinate_refusal_and_releases_provisional_result() {
        for (kind, cap) in [
            (WorkKind::BorderImageValidation, 0),
            (WorkKind::BorderImageCoordinate, 1),
            (WorkKind::OutputRecord, 9),
        ] {
            let limited = options(PaintLimits::unlimited().with_work_limit(kind, cap));
            let outcome = resolve_border_image(request(), limited);
            let Err(PaintError::Budget(refusal)) = outcome.result else {
                panic!(
                    "expected a budget refusal for {kind:?}: {:?}",
                    outcome.result
                );
            };
            assert_eq!(refusal.change, PaintChange::Work { kind, amount: 1 });
            assert_eq!(refusal.current, cap);
            let report = outcome.report.unwrap();
            assert_eq!(report.work(kind), cap);
            assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
            assert_eq!(report.live(ResourceKind::RetainedResultBytes), 0);
            if kind == WorkKind::OutputRecord {
                assert_eq!(refusal.origin, PaintPhase::Publication);
                assert_eq!(
                    report.peak(ResourceKind::RetainedResultBytes),
                    size_of::<BorderImageGeometry>() as u64
                );
            }
        }
        let outcome = resolve_border_image(request(), options(PaintLimits::unlimited()));
        assert!(outcome.result.is_ok());
        let report = outcome.report.unwrap();
        assert_eq!(report.work(WorkKind::OutputRecord), 10);
        assert_eq!(
            report.live(ResourceKind::RetainedResultBytes),
            size_of::<BorderImageGeometry>() as u64
        );
        assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
    }

    #[test]
    fn rejects_invalid_source_and_unrepresentable_signed_geometry() {
        let mut input = request();
        input.source.coordinate_size.width = 0.0;
        let bad = resolve_border_image(input, options(PaintLimits::unlimited()));
        assert_eq!(
            bad.result,
            Err(PaintError::BorderImage(
                BorderImageGeometryError::InvalidSource
            ))
        );
        input = request();
        input.border_origin.x = f64::MAX;
        let bad = resolve_border_image(input, options(PaintLimits::unlimited()));
        assert_eq!(
            bad.result,
            Err(PaintError::BorderImage(
                BorderImageGeometryError::UnrepresentableGeometry
            ))
        );
    }
}
