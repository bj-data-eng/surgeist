//! Ordered native displacement coordinates and closed, accounted GPU facts.
use super::{
    FilterOutputFootprint, FrameContext, GraphLoweringSpatialDescriptor,
    bounds::{
        FrameSpatialPlan, LogicalBounds, NonEmptyFrameSpatialPlan, PositiveDeviceExtent,
        SignedDeviceOrigin, TexelCenterMapping,
    },
    convolution::intersection,
    filter_grid::{self as grid, FilterGridMapping, FilterGridOwner, PeriodCapture},
};
use crate::{
    Color, DisplacementChannel, DisplacementNumericPhase, DisplacementPreparationError,
    FilterColorSpace, Point, PreparedDisplacement, Rect, Transform,
    accounting::{Account, RenderWork, SharedValue},
    displacement::NormalizedDisplacementForeground,
    error::Result,
    filter_source::FilterSourceDescriptor,
    image::RawImageUploadDescriptor,
};
use surgeist_color::{Color as IntrinsicColor, ColorSpace, Finite, PredefinedColorSpace};

const OWNER: FilterGridOwner = FilterGridOwner::Displacement;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DisplacementStage {
    RealizeSource,
    PassThrough,
    Displace,
    Transparent,
}
impl DisplacementStage {
    pub(super) fn read_count(self, kind: DisplacementSourceKind) -> usize {
        match self {
            Self::Transparent => 0,
            Self::Displace => 1,
            _ => usize::from(!matches!(kind, DisplacementSourceKind::Constant)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DisplacementSourceKind {
    Image,
    Constant,
    Periodic,
}

/// The storage-buffer representation has exactly the WGSL eight-byte stride.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(crate) struct DisplacementAddressEntry {
    q0: i32,
    outside: u32,
}
impl DisplacementAddressEntry {
    pub(crate) const fn q0(self) -> i32 {
        self.q0
    }
    pub(crate) const fn outside(self) -> bool {
        self.outside != 0
    }
}

/// One native nearest-cell decision; no shader float addressing is required.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub(crate) struct DisplacementMapRecord {
    pixel: u32,
    coverage: f32,
    outside: u32,
    padding: u32,
}
impl DisplacementMapRecord {
    pub(crate) const fn pixel(self) -> u32 {
        self.pixel
    }
    pub(crate) const fn coverage(self) -> f32 {
        self.coverage
    }
    pub(crate) const fn outside(self) -> bool {
        self.outside != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DisplacementMapping {
    grid: FilterGridMapping,
    kind: DisplacementSourceKind,
}
impl DisplacementMapping {
    pub(crate) fn source_spatial(self) -> Option<GraphLoweringSpatialDescriptor> {
        self.grid.source_spatial()
    }
    pub(crate) fn target_spatial(self) -> GraphLoweringSpatialDescriptor {
        self.grid.target_spatial()
    }
    pub(crate) fn source_extent(self) -> [u32; 2] {
        self.grid.source_extent()
    }
    pub(crate) const fn target_extent(self) -> [u32; 2] {
        self.grid.target_extent()
    }
    pub(crate) const fn source_base(self) -> [i32; 2] {
        self.grid.source_base()
    }
    pub(crate) const fn source_clip(self) -> [f64; 4] {
        self.grid.source_clip()
    }
    pub(crate) const fn output_clip(self) -> [f64; 4] {
        self.grid.output_clip()
    }
    pub(crate) const fn periodic_base(self) -> [f64; 2] {
        self.grid.periodic_base()
    }
    pub(crate) const fn periodic_step(self) -> [f64; 2] {
        self.grid.periodic_step()
    }
    pub(crate) const fn color(self) -> Color {
        self.grid.color()
    }
    pub(crate) const fn source_kind(self) -> DisplacementSourceKind {
        self.kind
    }
    pub(super) fn target_plan(self) -> NonEmptyFrameSpatialPlan {
        self.grid.target_plan()
    }
    pub(super) fn output_footprint(self) -> FilterOutputFootprint {
        self.grid.output_footprint()
    }
}

/// A period capture retains its actual suspended command context. It is never
/// reconstructed from the planner's ambient/root context after suspension.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ScopedPeriodContext {
    pub(super) capture: PeriodCapture,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RouteKind {
    Transparent,
    PassThrough,
    Active,
}

#[derive(Debug, PartialEq)]
struct DisplacementRouteData {
    foreground: NormalizedDisplacementForeground,
    result: NonEmptyFrameSpatialPlan,
    domain: Rect,
    window: Option<NonEmptyFrameSpatialPlan>,
    period: Option<ScopedPeriodContext>,
    kind: RouteKind,
    addresses: Option<SharedValue<Vec<DisplacementAddressEntry>>>,
    records: Option<SharedValue<Vec<DisplacementMapRecord>>>,
    zero_map: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct DisplacementRoute(SharedValue<DisplacementRouteData>);
impl DisplacementRoute {
    pub(super) fn try_new(
        context: FrameContext,
        mut foreground: NormalizedDisplacementForeground,
        transform: Transform,
        source_available: bool,
        force_empty: bool,
        account: &Account,
    ) -> Result<Self> {
        let result = grid::require_spatial(
            context,
            grid::result_domain(foreground.source),
            transform,
            OWNER,
        )?;
        let domain = grid::finite_domain(foreground.source).unwrap_or(foreground.source.filter());
        let passthrough = foreground.operation.is_passthrough() || foreground.map.is_passthrough();
        let empty = force_empty
            || foreground.map.is_empty()
            || grid::empty(domain)
            || (!source_available
                && !matches!(foreground.source, FilterSourceDescriptor::Constant { .. }))
            || matches!(foreground.source, FilterSourceDescriptor::Constant { color, .. } if color.a() == 0.0);
        if empty {
            foreground.map = crate::displacement::NormalizedDisplacementMap::Empty;
        }
        let mut kind = if empty {
            RouteKind::Transparent
        } else if passthrough {
            RouteKind::PassThrough
        } else {
            RouteKind::Active
        };
        let mut window = None;
        let mut addresses = None;
        let mut records = None;
        let zero_map = foreground.map.native_parts().is_none_or(|(upload, m)| {
            let size = upload.physical_size();
            size.width() == 0 || size.height() == 0 || grid::empty(m)
        });
        if !empty {
            if kind == RouteKind::Active {
                // D supplies coordinate bounds, not an allocation. Qualify the
                // physical extent only after the reached window is known.
                let FrameSpatialPlan::NonEmpty(original) = context
                    .plan_local_bounds(
                        LogicalBounds::try_from_rect(domain, "displacement source domain")?,
                        transform,
                    )
                    .map_err(|error| {
                        if error.code() == crate::ErrorCode::InvalidInput {
                            numeric(
                                DisplacementNumericPhase::GridMapping,
                                domain.width().max(domain.height()),
                            )
                        } else {
                            error
                        }
                    })?
                else {
                    return Err(numeric(DisplacementNumericPhase::GridMapping, 0.0));
                };
                let built = address_tables(foreground.operation, original, result, account)?;
                window = built.as_ref().map(|(spatial, _)| *spatial);
                addresses = built.map(|(_, entries)| entries);
                if window.is_none() {
                    kind = RouteKind::Transparent;
                }
                if kind == RouteKind::Active && !zero_map {
                    let (upload, map) = foreground
                        .map
                        .native_parts()
                        .ok_or_else(|| numeric(DisplacementNumericPhase::MapMapping, 0.0))?;
                    records = Some(map_records(
                        upload.physical_size().width(),
                        upload.physical_size().height(),
                        map,
                        foreground.source.filter(),
                        result,
                        account,
                    )?);
                }
            } else {
                // D and O may occupy disjoint fractions of the same lattice
                // cell. Narrow by integer cell intersection, never D intersect O.
                if matches!(foreground.source, FilterSourceDescriptor::Finite { .. }) {
                    let FrameSpatialPlan::NonEmpty(original) = context.plan_local_bounds(
                        LogicalBounds::try_from_rect(domain, "displacement bypass source")?,
                        transform,
                    )?
                    else {
                        return Err(numeric(DisplacementNumericPhase::GridMapping, 0.0));
                    };
                    let base = grid::base_between(original, result, OWNER)?;
                    let mut lower = [0; 2];
                    let mut upper = [0; 2];
                    for axis in 0..2 {
                        account.work(RenderWork::CoordinateVisits, 1)?;
                        let n = i64::from(grid::extent(original)[axis]);
                        lower[axis] = base[axis].clamp(0, n) as u32;
                        upper[axis] =
                            (base[axis] + i64::from(grid::extent(result)[axis])).clamp(0, n) as u32;
                    }
                    if (0..2).any(|axis| lower[axis] >= upper[axis]) {
                        kind = RouteKind::Transparent;
                    } else {
                        window = Some(window_spatial(original, lower, upper, account)?);
                    }
                } else {
                    window = Some(result);
                }
            }
        }
        let period = if kind != RouteKind::Transparent {
            match foreground.source {
                FilterSourceDescriptor::Periodic { period, .. } => Some(ScopedPeriodContext {
                    capture: PeriodCapture::try_new_for(
                        context, period, transform, account, OWNER,
                    )?,
                }),
                _ => None,
            }
        } else {
            None
        };
        SharedValue::new(
            DisplacementRouteData {
                foreground,
                result,
                domain,
                window,
                period,
                kind,
                addresses,
                records,
                zero_map,
            },
            &account.storage(),
        )
        .map(Self)
    }
    pub(super) fn capture_plan(&self) -> Option<NonEmptyFrameSpatialPlan> {
        self.0.period.map(|p| p.capture.spatial).or(self.0.window)
    }
    pub(super) fn period_context(&self) -> Option<ScopedPeriodContext> {
        self.0.period
    }
    pub(super) fn needs_source(&self) -> bool {
        self.0.kind != RouteKind::Transparent
            && !matches!(
                self.0.foreground.source,
                FilterSourceDescriptor::Constant { .. }
            )
    }
    pub(super) fn len(&self) -> usize {
        if self.0.kind == RouteKind::Active {
            2
        } else {
            1
        }
    }
    fn stage_at(&self, index: usize) -> Option<DisplacementStage> {
        match (self.0.kind, index) {
            (RouteKind::Transparent, 0) => Some(DisplacementStage::Transparent),
            (RouteKind::PassThrough, 0) => Some(DisplacementStage::PassThrough),
            (RouteKind::Active, 0) => Some(DisplacementStage::RealizeSource),
            (RouteKind::Active, 1) => Some(DisplacementStage::Displace),
            _ => None,
        }
    }
    fn mapping(
        &self,
        index: usize,
        raw: Option<NonEmptyFrameSpatialPlan>,
        account: &Account,
    ) -> Result<DisplacementMapping> {
        account.work(RenderWork::FilterOperationVisits, 1)?;
        let data = &*self.0;
        let stage = self
            .stage_at(index)
            .ok_or_else(|| numeric(DisplacementNumericPhase::TableLayout, index as f64))?;
        let kind = match data.foreground.source {
            FilterSourceDescriptor::Finite { .. } => DisplacementSourceKind::Image,
            FilterSourceDescriptor::Constant { .. } => DisplacementSourceKind::Constant,
            FilterSourceDescriptor::Periodic { .. } => DisplacementSourceKind::Periodic,
        };
        if stage == DisplacementStage::Transparent {
            return Ok(DisplacementMapping {
                grid: FilterGridMapping::new(None, data.result, OWNER).with_output_clip(
                    grid::result_domain(data.foreground.source),
                    data.result,
                    account,
                )?,
                kind: DisplacementSourceKind::Constant,
            });
        }
        let window = data
            .window
            .ok_or_else(|| numeric(DisplacementNumericPhase::GridMapping, 0.0))?;
        if stage == DisplacementStage::Displace {
            return Ok(DisplacementMapping {
                grid: FilterGridMapping::new(Some(window), data.result, OWNER).with_output_clip(
                    grid::result_domain(data.foreground.source),
                    data.result,
                    account,
                )?,
                kind: DisplacementSourceKind::Image,
            });
        }
        let target = if stage == DisplacementStage::PassThrough {
            data.result
        } else {
            window
        };
        let mut mapping = FilterGridMapping::new(raw, target, OWNER);
        if stage == DisplacementStage::PassThrough {
            mapping = mapping.with_output_clip(
                grid::result_domain(data.foreground.source),
                target,
                account,
            )?;
        }
        match data.foreground.source {
            FilterSourceDescriptor::Constant { color, .. } => {
                mapping =
                    mapping
                        .with_color(color)
                        .with_source_clip(data.domain, target, account)?;
            }
            FilterSourceDescriptor::Periodic { .. } => {
                let period = data
                    .period
                    .ok_or_else(|| numeric(DisplacementNumericPhase::GridMapping, 0.0))?
                    .capture;
                if raw != Some(period.spatial) {
                    return Err(numeric(DisplacementNumericPhase::GridMapping, 0.0));
                }
                mapping = mapping
                    .with_periodic_resolve(period, target, account)?
                    .with_source_clip(data.domain, target, account)?;
            }
            FilterSourceDescriptor::Finite { .. } => {
                let raw = raw.ok_or_else(|| numeric(DisplacementNumericPhase::GridMapping, 0.0))?;
                let raw_bounds = raw.logical_bounds.rect();
                let needed = window.logical_bounds.rect();
                if raw_bounds.x() > needed.x()
                    || raw_bounds.y() > needed.y()
                    || raw_bounds.max().x() < needed.max().x()
                    || raw_bounds.max().y() < needed.max().y()
                {
                    return Err(numeric(DisplacementNumericPhase::GridMapping, 0.0));
                }
                mapping = mapping
                    .with_base(grid::base_between(raw, target, OWNER)?, [0; 2], account)?
                    .with_source_clip(data.domain, raw, account)?;
            }
        }
        Ok(DisplacementMapping {
            grid: mapping,
            kind,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DisplacementPassFacts(SharedValue<DisplacementPassData>);
#[derive(Debug, PartialEq)]
struct DisplacementPassData {
    route: DisplacementRoute,
    raw: Option<NonEmptyFrameSpatialPlan>,
    index: usize,
    mapping: DisplacementMapping,
    stage: DisplacementStage,
}
impl DisplacementPassFacts {
    pub(super) fn try_new(
        route: DisplacementRoute,
        index: usize,
        raw: Option<NonEmptyFrameSpatialPlan>,
        account: &Account,
    ) -> Result<Self> {
        let stage = route
            .stage_at(index)
            .ok_or_else(|| numeric(DisplacementNumericPhase::TableLayout, index as f64))?;
        let mapping = route.mapping(index, raw, account)?;
        SharedValue::new(
            DisplacementPassData {
                route,
                raw,
                index,
                mapping,
                stage,
            },
            &account.storage(),
        )
        .map(Self)
    }
    pub(crate) fn stage(&self) -> DisplacementStage {
        self.0.stage
    }
    pub(crate) fn mapping(&self) -> DisplacementMapping {
        self.0.mapping
    }
    pub(crate) fn operation(&self) -> PreparedDisplacement {
        self.0.route.0.foreground.operation
    }
    pub(crate) fn map_upload(&self) -> Option<&RawImageUploadDescriptor> {
        (self.stage() == DisplacementStage::Displace && !self.map_is_zero())
            .then(|| {
                self.0
                    .route
                    .0
                    .foreground
                    .map
                    .native_parts()
                    .map(|(upload, _)| upload)
            })
            .flatten()
    }
    pub(crate) fn address_entries(&self) -> Option<&SharedValue<Vec<DisplacementAddressEntry>>> {
        (self.stage() == DisplacementStage::Displace)
            .then_some(self.0.route.0.addresses.as_ref())
            .flatten()
    }
    pub(crate) fn map_records(&self) -> Option<&SharedValue<Vec<DisplacementMapRecord>>> {
        (self.stage() == DisplacementStage::Displace)
            .then_some(self.0.route.0.records.as_ref())
            .flatten()
    }
    pub(crate) fn map_is_zero(&self) -> bool {
        self.0.route.0.zero_map
    }
    pub(crate) fn route_index(&self) -> usize {
        self.0.index
    }
    pub(crate) fn route_len(&self) -> usize {
        self.0.route.len()
    }
    pub(crate) fn same_route_in(&self, other: &Self, account: &Account) -> Result<bool> {
        account.work(RenderWork::ResourceProbes, 1)?;
        Ok(self.0.route.0.same_allocation(&other.0.route.0))
    }
    pub(crate) fn proves_exact(&self, account: &Account) -> Result<bool> {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        Ok(self.0.route.stage_at(self.0.index) == Some(self.stage())
            && self.0.route.mapping(self.0.index, self.0.raw, account)? == self.mapping())
    }
}

fn numeric(phase: DisplacementNumericPhase, value: f64) -> crate::Error {
    DisplacementPreparationError::Numeric { phase, value }.into()
}
fn finite(value: f64, phase: DisplacementNumericPhase) -> Result<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(numeric(phase, value))
    }
}

#[derive(Clone, Copy)]
struct CoordinateEntry {
    q0: f64,
    outside: bool,
}

/// Coordinate qualification and table construction are distinct metered phases.
/// Outside offsets never cross a float-to-integer conversion boundary.
fn address_tables(
    operation: PreparedDisplacement,
    source: NonEmptyFrameSpatialPlan,
    output: NonEmptyFrameSpatialPlan,
    account: &Account,
) -> Result<
    Option<(
        NonEmptyFrameSpatialPlan,
        SharedValue<Vec<DisplacementAddressEntry>>,
    )>,
> {
    let base = grid::base_between(source, output, OWNER)?;
    let n = grid::extent(source);
    let m = grid::extent(output);
    let scale = finite(
        operation.scale() * source.raster_scale.get(),
        DisplacementNumericPhase::TableLayout,
    )?;
    let coefficient = finite(scale / 255.0, DisplacementNumericPhase::TableLayout)?;
    let half = finite(0.5 * scale, DisplacementNumericPhase::TableLayout)?;
    let bias = finite(0.5 - half, DisplacementNumericPhase::TableLayout)?;
    let linear = operation.color_space() == FilterColorSpace::LinearSrgb
        && operation
            .channels()
            .into_iter()
            .any(|c| c != DisplacementChannel::A);
    let mut lut = account.temporary(if linear { 256 } else { 0 })?;
    if linear {
        for byte in 0_u16..256 {
            account.work(RenderWork::DisplacementTableVisits, 1)?;
            let value = f64::from(byte) / 255.0;
            let value = Finite::new(value)
                .ok_or_else(|| numeric(DisplacementNumericPhase::TableLayout, value))?;
            let color = IntrinsicColor::new(
                ColorSpace::Predefined(PredefinedColorSpace::Srgb),
                [Some(value); 3],
                None,
            )
            .converted(ColorSpace::Predefined(PredefinedColorSpace::SrgbLinear));
            let channel = color.channels()[0]
                .ok_or_else(|| numeric(DisplacementNumericPhase::TableLayout, f64::from(byte)))?;
            let quantized = finite(
                (255.0 * channel + 0.5).floor(),
                DisplacementNumericPhase::TableLayout,
            )?
            .clamp(0.0, 255.0) as u8;
            lut.append_slot()?.write(quantized);
        }
    }
    let mut coordinates = account.temporary(512)?;
    let mut lower = [u32::MAX; 2];
    let mut upper = [0; 2];
    for axis in 0..2 {
        let low_offset = -base[axis] - i64::from(m[axis]) + 1;
        let high_offset = i64::from(n[axis]) - 1 - base[axis];
        for byte in 0..256 {
            account.work(RenderWork::CoordinateVisits, 1)?;
            let selected = if linear && operation.channels()[axis] != DisplacementChannel::A {
                f64::from(lut[byte])
            } else {
                byte as f64
            };
            let product = finite(
                coefficient * selected,
                DisplacementNumericPhase::TableLayout,
            )?;
            let value = finite(product + bias, DisplacementNumericPhase::TableLayout)?.trunc();
            let outside = value < low_offset as f64 || value > high_offset as f64;
            let q0 = if outside {
                0.0
            } else {
                value + base[axis] as f64
            };
            if !outside {
                let start = q0 as i64;
                let end = start + i64::from(m[axis]);
                lower[axis] = lower[axis].min(start.max(0) as u32);
                upper[axis] = upper[axis].max(end.min(i64::from(n[axis])) as u32);
            }
            coordinates
                .append_slot()?
                .write(CoordinateEntry { q0, outside });
        }
    }
    if (0..2).any(|axis| lower[axis] >= upper[axis]) {
        return Ok(None);
    }
    let window = window_spatial(source, lower, upper, account)?;
    let mut entries = account.retained(512)?;
    for axis in 0..2 {
        for entry in &coordinates[axis * 256..axis * 256 + 256] {
            account.work(RenderWork::DisplacementTableVisits, 1)?;
            let q0 = if entry.outside {
                0
            } else {
                let value = entry.q0 - f64::from(lower[axis]);
                if value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
                    return Err(numeric(DisplacementNumericPhase::GridIndex, value));
                }
                let q0 = value as i32;
                i32::try_from(i64::from(q0) + i64::from(m[axis]) - 1)
                    .map_err(|_| numeric(DisplacementNumericPhase::GridIndex, value))?;
                q0
            };
            entries.append_slot()?.write(DisplacementAddressEntry {
                q0,
                outside: u32::from(entry.outside),
            });
        }
    }
    Ok(Some((
        window,
        SharedValue::from_accounted(entries, &account.storage())?,
    )))
}

fn window_spatial(
    source: NonEmptyFrameSpatialPlan,
    lower: [u32; 2],
    upper: [u32; 2],
    account: &Account,
) -> Result<NonEmptyFrameSpatialPlan> {
    let mut origin = [0; 2];
    for axis in 0..2 {
        account.work(RenderWork::CoordinateVisits, 1)?;
        if upper[axis] - lower[axis] > 8_388_608 {
            return Err(numeric(
                DisplacementNumericPhase::GridIndex,
                f64::from(upper[axis] - lower[axis]),
            ));
        }
        let base = if axis == 0 {
            source.device_origin.x
        } else {
            source.device_origin.y
        };
        origin[axis] = i32::try_from(i64::from(base) + i64::from(lower[axis]))
            .map_err(|_| numeric(DisplacementNumericPhase::GridIndex, f64::from(base)))?;
    }
    let rho = source.raster_scale.get();
    let point = Point::new(f64::from(origin[0]) / rho, f64::from(origin[1]) / rho);
    let rect = Rect::new(
        point.x(),
        point.y(),
        f64::from(upper[0] - lower[0]) / rho,
        f64::from(upper[1] - lower[1]) / rho,
    );
    let LogicalBounds::NonEmpty(bounds) =
        LogicalBounds::try_from_rect(rect, "displacement source window")?
    else {
        return Err(numeric(DisplacementNumericPhase::GridMapping, 0.0));
    };
    Ok(NonEmptyFrameSpatialPlan {
        logical_bounds: bounds,
        device_origin: SignedDeviceOrigin::new(origin[0], origin[1]),
        device_extent: PositiveDeviceExtent {
            width: upper[0] - lower[0],
            height: upper[1] - lower[1],
        },
        raster_scale: source.raster_scale,
        texel_center_mapping: TexelCenterMapping {
            origin: point,
            raster_scale: source.raster_scale,
        },
    })
}

fn map_records(
    width: u32,
    height: u32,
    map: Rect,
    filter: Rect,
    output: NonEmptyFrameSpatialPlan,
    account: &Account,
) -> Result<SharedValue<Vec<DisplacementMapRecord>>> {
    let size = grid::extent(output);
    let count = usize::try_from(u64::from(size[0]) + u64::from(size[1]))
        .map_err(|_| numeric(DisplacementNumericPhase::MapMapping, f64::from(size[0])))?;
    let clipped = intersection(map, filter);
    let mut records = account.retained(count)?;
    let mut minimum = [None::<f32>; 2];
    for axis in 0..2 {
        account.work(RenderWork::CoordinateVisits, 1)?;
        let origin = if axis == 0 {
            output.device_origin.x
        } else {
            output.device_origin.y
        };
        let (map_start, length, clipped_start, clipped_length, pixels) = if axis == 0 {
            (map.x(), map.width(), clipped.x(), clipped.width(), width)
        } else {
            (map.y(), map.height(), clipped.y(), clipped.height(), height)
        };
        let rho = output.raster_scale.get();
        // A native map anchor is compared on the common device lattice first.
        // Per-cell geometry then uses the integer rebased device delta.
        let map_device = finite(map_start * rho, DisplacementNumericPhase::MapMapping)?;
        let relative_map = finite(
            map_device - f64::from(origin),
            DisplacementNumericPhase::MapMapping,
        )?;
        let clip_start = finite(
            clipped_start * rho - f64::from(origin),
            DisplacementNumericPhase::MapMapping,
        )?;
        let clip_end = finite(
            (clipped_start + clipped_length) * rho - f64::from(origin),
            DisplacementNumericPhase::MapMapping,
        )?;
        for cell in 0..size[axis] {
            account.work(RenderWork::DisplacementMapRecordVisits, 1)?;
            let cell = f64::from(cell);
            let fraction = ((cell + 1.0).min(clip_end) - cell.max(clip_start)).clamp(0.0, 1.0);
            if fraction == 0.0 || pixels == 0 || grid::empty(clipped) {
                records.append_slot()?.write(DisplacementMapRecord {
                    pixel: 0,
                    coverage: 0.0,
                    outside: 1,
                    padding: 0,
                });
                continue;
            }
            let coverage = fraction as f32;
            if !coverage.is_normal() || coverage <= 0.0 || coverage > 1.0 {
                return Err(numeric(DisplacementNumericPhase::MapMapping, fraction));
            }
            minimum[axis] = Some(minimum[axis].map_or(coverage, |n| n.min(coverage)));
            let delta = finite(
                (cell + 0.5 - relative_map) / rho,
                DisplacementNumericPhase::MapMapping,
            )?
            .clamp(0.0, length);
            let normalized = finite(delta / length, DisplacementNumericPhase::MapMapping)?;
            let pixel = finite(
                normalized * f64::from(pixels),
                DisplacementNumericPhase::MapMapping,
            )?
            .floor()
            .clamp(0.0, f64::from(pixels - 1)) as u32;
            records.append_slot()?.write(DisplacementMapRecord {
                pixel,
                coverage,
                outside: 0,
                padding: 0,
            });
        }
    }
    if let [Some(x), Some(y)] = minimum {
        account.work(RenderWork::CoordinateVisits, 1)?;
        let product = x * y;
        if !product.is_normal() || product <= 0.0 {
            return Err(numeric(
                DisplacementNumericPhase::MapMapping,
                f64::from(product),
            ));
        }
    }
    SharedValue::from_accounted(records, &account.storage())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DisplacementInput, RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind,
        RenderLimits, RenderPhase, prepare_displacement,
    };

    fn operation(
        scale: f64,
        channels: [DisplacementChannel; 2],
        space: FilterColorSpace,
    ) -> PreparedDisplacement {
        prepare_displacement(
            DisplacementInput::new()
                .with_scale(scale)
                .with_channels(channels[0], channels[1]),
            space,
            &mut RenderAccountingContext::new(RenderAccountingOptions::new()),
        )
        .unwrap()
    }
    fn spatial(rect: Rect, density: f64) -> NonEmptyFrameSpatialPlan {
        grid::require_spatial(
            FrameContext::try_for_spatial_test(density).unwrap(),
            rect,
            Transform::identity(),
            OWNER,
        )
        .unwrap()
    }
    fn table(scale: f64) -> SharedValue<Vec<DisplacementAddressEntry>> {
        let image = spatial(Rect::new(0.0, 0.0, 100.0, 100.0), 1.0);
        address_tables(
            operation(scale, [DisplacementChannel::A; 2], FilterColorSpace::Srgb),
            image,
            image,
            &Account::for_test(),
        )
        .unwrap()
        .unwrap()
        .1
    }

    #[test]
    fn signed_scale_four_uses_asymmetric_literal_offsets() {
        let positive = table(4.0);
        assert_eq!((positive[0].q0(), positive[255].q0()), (-1, 2));
        let negative = table(-4.0);
        assert_eq!((negative[0].q0(), negative[255].q0()), (2, -1));
        assert_eq!((positive[256].q0(), positive[511].q0()), (-1, 2));
    }

    #[test]
    fn ordered_binary64_retains_threshold_witnesses() {
        assert_eq!(table(-153.0)[85].q0(), 26);
        assert_eq!(table(-155.0)[102].q0(), 16);
        assert_eq!(table(0.75)[0].q0(), 0);
        assert_eq!(table(0.75)[255].q0(), 0);
    }

    #[test]
    fn source_window_includes_reachable_texels_outside_output() {
        let source = spatial(Rect::new(0.0, 0.0, 10.0, 1.0), 1.0);
        let output = spatial(Rect::new(4.0, 0.0, 1.0, 1.0), 1.0);
        let (window, entries) = address_tables(
            operation(4.0, [DisplacementChannel::A; 2], FilterColorSpace::Srgb),
            source,
            output,
            &Account::for_test(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(window.device_origin.x, 3);
        assert_eq!(window.device_extent.width, 4);
        assert_eq!((entries[0].q0(), entries[255].q0()), (0, 3));
        assert!(entries[256].outside());
        assert!(entries[511].outside());
    }

    #[test]
    fn huge_representable_offsets_are_outside_without_integer_casts() {
        let image = spatial(Rect::new(0.0, 0.0, 2.0, 2.0), 1.0);
        assert!(
            address_tables(
                operation(1e100, [DisplacementChannel::A; 2], FilterColorSpace::Srgb),
                image,
                image,
                &Account::for_test()
            )
            .unwrap()
            .is_none()
        );
        let image = spatial(Rect::new(0.0, 0.0, 2.0, 2.0), 2.0);
        let error = address_tables(
            operation(
                f64::MAX,
                [DisplacementChannel::A; 2],
                FilterColorSpace::Srgb,
            ),
            image,
            image,
            &Account::for_test(),
        )
        .unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
    }

    #[test]
    fn linear_rgb_table_uses_color_owner_conversion_without_changing_alpha() {
        let image = spatial(Rect::new(0.0, 0.0, 100.0, 100.0), 1.0);
        let entries = address_tables(
            operation(
                8.0,
                [DisplacementChannel::R, DisplacementChannel::A],
                FilterColorSpace::LinearSrgb,
            ),
            image,
            image,
            &Account::for_test(),
        )
        .unwrap()
        .unwrap()
        .1;
        assert_eq!(entries[128].q0(), -1); // Color owner converts byte128 to byte55.
        assert_eq!(entries[256 + 128].q0(), 0);
    }

    #[test]
    fn map_cell_ties_choose_higher_cell_and_partial_centers_clamp() {
        let output = spatial(Rect::new(0.0, 0.0, 1.0, 1.0), 1.0);
        let filter = Rect::new(-2.0, -2.0, 4.0, 4.0);
        let ties = map_records(
            2,
            1,
            Rect::new(-0.5, 0.0, 2.0, 1.0),
            filter,
            output,
            &Account::for_test(),
        )
        .unwrap();
        assert_eq!((ties[0].pixel(), ties[0].coverage()), (1, 1.0));
        let partial = map_records(
            2,
            1,
            Rect::new(0.75, 0.0, 1.0, 1.0),
            filter,
            output,
            &Account::for_test(),
        )
        .unwrap();
        assert_eq!((partial[0].pixel(), partial[0].coverage()), (0, 0.25));
        let exterior = map_records(
            2,
            1,
            Rect::new(1.0, 0.0, 1.0, 1.0),
            filter,
            output,
            &Account::for_test(),
        )
        .unwrap();
        assert!(exterior[0].outside());
    }

    #[test]
    fn map_coverage_uses_full_output_cells_before_output_coverage() {
        let output = spatial(Rect::new(0.0, 0.0, 0.5, 1.0), 1.0);
        let records = map_records(
            1,
            1,
            Rect::new(0.5, 0.0, 0.5, 1.0),
            Rect::new(0.0, 0.0, 1.0, 1.0),
            output,
            &Account::for_test(),
        )
        .unwrap();
        assert_eq!(records[0].coverage(), 0.5);
        assert!(!records[0].outside());
    }

    #[test]
    fn binary64_map_boundaries_use_the_qualified_host_records() {
        let output = spatial(Rect::new(0.0, 0.0, 1.0, 1.0), 1.0);
        let filter = Rect::new(-2.0, -2.0, 4.0, 4.0);
        for (start, expected) in [
            (-0.4999999999999999, 0),
            (-0.5, 1),
            (-0.5000000000000001, 1),
        ] {
            let records = map_records(
                2,
                1,
                Rect::new(start, 0.0, 2.0, 1.0),
                filter,
                output,
                &Account::for_test(),
            )
            .unwrap();
            assert_eq!(records[0].pixel(), expected);
        }
    }

    #[test]
    fn native_map_records_rebase_large_logical_origins_on_device_cells() {
        let output = spatial(Rect::new(1e12, 0.0, 1e6, 1e6), 1e-6);
        let records = map_records(
            2,
            1,
            Rect::new(1e12 - 5e5, 0.0, 2e6, 1e6),
            Rect::new(1e12 - 1e6, 0.0, 4e6, 1e6),
            output,
            &Account::for_test(),
        )
        .unwrap();
        assert_eq!(records[0].pixel(), 1);
        assert_eq!(records[0].coverage(), 1.0);
    }

    #[test]
    fn positive_axis_fractions_with_unrepresentable_product_refuse() {
        let output = spatial(Rect::new(0.0, 0.0, 1.0, 1.0), 1.0);
        assert!(
            map_records(
                1,
                1,
                Rect::new(0.0, 0.0, 1e-25, 1e-25),
                Rect::new(0.0, 0.0, 1.0, 1.0),
                output,
                &Account::for_test()
            )
            .is_err()
        );
    }

    #[test]
    fn final_address_construction_admits_exact_512_and_refuses_one_under() {
        let image = spatial(Rect::new(0.0, 0.0, 10.0, 10.0), 1.0);
        let effect = operation(4.0, [DisplacementChannel::A; 2], FilterColorSpace::Srgb);
        for limit in [511, 512] {
            let mut context = RenderAccountingContext::new(
                RenderAccountingOptions::new().with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::DisplacementTableVisits, limit)
                        .unwrap(),
                ),
            );
            let result = context.run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
                address_tables(effect, image, image, account).map(|_| ())
            });
            assert_eq!(result.is_ok(), limit == 512);
            let report = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(
                report
                    .phase_usage(RenderPhase::Plan)
                    .work(RenderWork::DisplacementTableVisits),
                limit
            );
        }
    }

    #[test]
    fn semantic_bypass_omits_unreached_overflow_and_all_map_payloads() {
        let account = Account::for_test();
        let domain = Rect::new(0.0, 0.0, 4.0, 4.0);
        let foreground = NormalizedDisplacementForeground {
            operation: operation(
                f64::MAX,
                [DisplacementChannel::A; 2],
                FilterColorSpace::Srgb,
            ),
            source: FilterSourceDescriptor::Constant {
                color: Color::BLACK,
                filter: domain,
                result: domain,
            },
            map: crate::displacement::NormalizedDisplacementMap::Bypass,
        };
        let route = DisplacementRoute::try_new(
            FrameContext::try_for_spatial_test(2.0).unwrap(),
            foreground,
            Transform::identity(),
            false,
            false,
            &account,
        )
        .unwrap();
        let facts = DisplacementPassFacts::try_new(route, 0, None, &account).unwrap();
        assert_eq!(facts.stage(), DisplacementStage::PassThrough);
        assert!(facts.map_upload().is_none());
        assert!(facts.address_entries().is_none());
        assert!(facts.map_records().is_none());
    }

    #[test]
    fn zero_native_map_keeps_active_tables_without_records_or_upload() {
        let account = Account::for_test();
        let domain = Rect::new(0.0, 0.0, 4.0, 4.0);
        let foreground = NormalizedDisplacementForeground {
            operation: operation(4.0, [DisplacementChannel::A; 2], FilterColorSpace::Srgb),
            source: FilterSourceDescriptor::Constant {
                color: Color::BLACK,
                filter: domain,
                result: domain,
            },
            map: crate::displacement::NormalizedDisplacementMap::ZeroNative,
        };
        let route = DisplacementRoute::try_new(
            FrameContext::try_for_spatial_test(1.0).unwrap(),
            foreground,
            Transform::identity(),
            false,
            false,
            &account,
        )
        .unwrap();
        let facts = DisplacementPassFacts::try_new(route, 1, None, &account).unwrap();
        assert_eq!(facts.stage(), DisplacementStage::Displace);
        assert_eq!(facts.address_entries().unwrap().len(), 512);
        assert!(facts.map_is_zero());
        assert!(facts.map_records().is_none());
        assert!(facts.map_upload().is_none());
    }

    #[test]
    fn empty_source_omits_unreached_scaled_coefficient_overflow() {
        let account = Account::for_test();
        let domain = Rect::new(0.0, 0.0, 4.0, 4.0);
        let foreground = NormalizedDisplacementForeground {
            operation: operation(
                f64::MAX,
                [DisplacementChannel::A; 2],
                FilterColorSpace::Srgb,
            ),
            source: FilterSourceDescriptor::Constant {
                color: Color::TRANSPARENT,
                filter: domain,
                result: domain,
            },
            map: crate::displacement::NormalizedDisplacementMap::Empty,
        };
        let route = DisplacementRoute::try_new(
            FrameContext::try_for_spatial_test(2.0).unwrap(),
            foreground,
            Transform::identity(),
            false,
            false,
            &account,
        )
        .unwrap();
        let facts = DisplacementPassFacts::try_new(route, 0, None, &account).unwrap();
        assert_eq!(facts.stage(), DisplacementStage::Transparent);
        assert!(facts.address_entries().is_none());
    }

    #[test]
    fn period_capture_uses_explicit_suspended_context_density() {
        let account = Account::for_test();
        let domain = Rect::new(0.0, 0.0, 4.0, 4.0);
        let foreground = NormalizedDisplacementForeground {
            operation: operation(0.0, [DisplacementChannel::A; 2], FilterColorSpace::Srgb),
            source: FilterSourceDescriptor::Periodic {
                period: Rect::new(0.0, 0.0, 2.0, 3.0),
                filter: domain,
                result: domain,
            },
            map: crate::displacement::NormalizedDisplacementMap::Bypass,
        };
        let route = DisplacementRoute::try_new(
            FrameContext::try_for_spatial_test(2.0).unwrap(),
            foreground,
            Transform::identity(),
            true,
            false,
            &account,
        )
        .unwrap();
        let period = route.period_context().unwrap().capture;
        assert_eq!(grid::extent(period.spatial), [4, 6]);
        assert_eq!(period.density, [2.0, 2.0]);
        assert_eq!(period.context.surface_scale, 1.0);
    }

    #[test]
    fn bypass_preserves_disjoint_source_and_output_fractions_of_one_cell() {
        let account = Account::for_test();
        let source = Rect::new(0.0, 0.0, 0.5, 1.0);
        let filter = Rect::new(0.0, 0.0, 1.0, 1.0);
        let output = Rect::new(0.5, 0.0, 0.5, 1.0);
        let foreground = NormalizedDisplacementForeground {
            operation: operation(0.0, [DisplacementChannel::A; 2], FilterColorSpace::Srgb),
            source: FilterSourceDescriptor::Finite {
                domains: crate::FilterDomains::try_new(source, filter, output).unwrap(),
                edge: crate::FilterEdgeMode::None,
            },
            map: crate::displacement::NormalizedDisplacementMap::Bypass,
        };
        let route = DisplacementRoute::try_new(
            FrameContext::try_for_spatial_test(1.0).unwrap(),
            foreground,
            Transform::identity(),
            true,
            false,
            &account,
        )
        .unwrap();
        let capture = route.capture_plan().unwrap();
        let facts = DisplacementPassFacts::try_new(route, 0, Some(capture), &account).unwrap();
        assert_eq!(facts.stage(), DisplacementStage::PassThrough);
        assert_eq!(facts.mapping().source_clip(), [0.0, 0.0, 0.5, 1.0]);
        assert_eq!(facts.mapping().output_clip(), [0.5, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn large_declared_domain_does_not_become_an_unneeded_physical_allocation() {
        let account = Account::for_test();
        let foreground = NormalizedDisplacementForeground {
            operation: operation(4.0, [DisplacementChannel::A; 2], FilterColorSpace::Srgb),
            source: FilterSourceDescriptor::Constant {
                color: Color::BLACK,
                filter: Rect::new(0.0, 0.0, 20_000_000.0, 1.0),
                result: Rect::new(10_000_000.0, 0.0, 1.0, 1.0),
            },
            map: crate::displacement::NormalizedDisplacementMap::ZeroNative,
        };
        let route = DisplacementRoute::try_new(
            FrameContext::try_for_spatial_test(1.0).unwrap(),
            foreground,
            Transform::identity(),
            false,
            false,
            &account,
        )
        .unwrap();
        let capture = route.capture_plan().unwrap();
        assert_eq!(
            (capture.device_origin.x, capture.device_extent.width),
            (9_999_999, 4)
        );
    }
}
