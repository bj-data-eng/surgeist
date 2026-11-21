//! Reached windows on the filter-anchored lighting canvas and the local raster.
use super::{
    FrameContext,
    bounds::{
        LogicalBounds, NonEmptyFrameSpatialPlan, PositiveDeviceExtent, RasterScale,
        SignedDeviceOrigin, TexelCenterMapping,
    },
    convolution::intersection,
    filter_grid::{self, FilterGridOwner},
};
use crate::{
    ErrorCode,
    accounting::{Account, RenderWork},
    error::Result,
    filter_domains::FilterDomains,
    geometry::{Point, Rect, Transform},
    lighting::{LightingNumericPhase as Phase, numeric_error},
};

const OWNER: FilterGridOwner = FilterGridOwner::Lighting;
// Beyond this boundary f64 cannot distinguish every consecutive integer.
const EXACT_INTEGER_LIMIT: f64 = 9_007_199_254_740_992.0;
// At this boundary f32 no longer distinguishes any subtexel fraction.
const SAMPLE_LIMIT: f32 = 8_388_608.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LightingGrid {
    pub(super) spacing: [f64; 2],
    pub(super) counts: [i64; 2],
    pub(super) output: NonEmptyFrameSpatialPlan,
    pub(super) output_clip: [f32; 4],
    pub(super) shade: LightingWindow,
    pub(super) bump: LightingWindow,
    pub(super) capture: Option<LightingCapture>,
    pub(super) resolve_map: [f32; 4],
    pub(super) shade_source_offset: [i32; 2],
    axes: [RasterAxis; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LightingWindow {
    pub(super) index_origin: [i64; 2],
    pub(super) spatial: NonEmptyFrameSpatialPlan,
    pub(super) edge_flags: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LightingCapture {
    pub(super) spatial: NonEmptyFrameSpatialPlan,
    pub(super) clip: [f32; 4],
    pub(super) sample_map: [f32; 4],
}

impl LightingGrid {
    pub(super) fn try_new(
        context: FrameContext,
        domains: FilterDomains,
        raster_transform: Transform,
        spacing: Option<[f64; 2]>,
        source_available: bool,
        account: &Account,
    ) -> Result<Option<Self>> {
        let filter = domains.filter();
        if filter_grid::empty(filter) || filter_grid::empty(domains.result()) {
            return Ok(None);
        }
        let domain = intersection(filter, domains.result());
        let mins = minima(filter);
        let maxs = maxima(filter);
        let lengths = [filter.width(), filter.height()];
        // A disjoint O needs no raster arithmetic. A positive declared F
        // with collapsed endpoints still needs its numeric diagnostic.
        if filter_grid::empty(domain) && maxs[0] > mins[0] && maxs[1] > mins[1] {
            return Ok(None);
        }
        let Some(rho) = context
            .local_raster_scale(raster_transform)
            .map_err(|error| {
                if error.code() == ErrorCode::InvalidInput {
                    numeric_error(Phase::GridMapping, context.surface_scale)
                } else {
                    error
                }
            })?
        else {
            return Ok(None);
        };
        qualify_canvas_extent(domains, account)?;
        if filter_grid::empty(domain) {
            return Ok(None);
        }
        let output = output_spatial(domain, rho, account)?;
        let output_clip = raster_clip(domain, output, account)?;
        let rho = rho.get();
        let sizes = filter_grid::extent(output);
        let device = device_origin(output).map(i64::from);
        let implicit = spacing.is_none();
        let spacing = spacing.unwrap_or([1.0 / rho; 2]);
        let axes = [
            RasterAxis::try_new(mins[0], lengths[0], spacing[0], rho, implicit, account)?,
            RasterAxis::try_new(mins[1], lengths[1], spacing[1], rho, implicit, account)?,
        ];

        let x = axes[0].reach_shade(device[0], sizes[0], account)?;
        let y = axes[1].reach_shade(device[1], sizes[1], account)?;
        let counts = axes.map(|axis| axis.count);
        let shade_ranges = [x.range, y.range];
        let bump_ranges = [
            shade_ranges[0].with_sobel_neighbors(counts[0])?,
            shade_ranges[1].with_sobel_neighbors(counts[1])?,
        ];
        let shade = LightingWindow::try_new(context, shade_ranges, counts)?;
        let bump = LightingWindow::try_new(context, bump_ranges, counts)?;
        let source = intersection(filter, domains.source());
        let capture = if source_available && !filter_grid::empty(source) {
            prepare_capture(source, output, axes, bump_ranges, account)?
        } else {
            None
        };

        Ok(Some(Self {
            spacing,
            counts,
            output,
            output_clip,
            shade,
            bump,
            capture,
            resolve_map: [x.map[0], y.map[0], x.map[1], y.map[1]],
            // Growing a Sobel window adds at most one preceding cell per axis.
            shade_source_offset: [
                i32::from(shade_ranges[0].start > bump_ranges[0].start),
                i32::from(shade_ranges[1].start > bump_ranges[1].start),
            ],
            axes,
        }))
    }

    pub(super) fn relative_node(&self, axis: usize, index: i64, light: f64) -> Result<f64> {
        let raster_origin = device_origin(self.output)[axis];
        let origin = f64::from(raster_origin);
        let rho = self.output.raster_scale.get();
        let center = self.axes[axis].node_coordinate(i64::from(raster_origin), index, 0.0)?;
        let device_center = origin + center;
        let represented_offset = device_center - origin;
        let sum_residual =
            (origin - (device_center - represented_offset)) + (center - represented_offset);
        // If the full device center combines exactly, recognize its rounded
        // local position as coincidence before a raster round trip can
        // invent a separation smaller than any local caller could express.
        if sum_residual == 0.0 && device_center / rho == light {
            return Ok(0.0);
        }
        // Subtract the light before leaving raster units to preserve the
        // recovered center phase. Fall back if scaled light deltas overflow.
        let raster_delta = if light == origin / rho {
            0.0
        } else {
            (-light).mul_add(rho, origin)
        };
        let combined = center + raster_delta;
        let relative = if combined.is_finite() {
            combined / rho
        } else {
            (origin / rho - light) + center / rho
        };
        finite(relative, Phase::LightVector)
    }
}

pub(super) fn qualify_canvas_extent(domains: FilterDomains, account: &Account) -> Result<()> {
    let filter = domains.filter();
    let result = domains.result();
    if filter_grid::empty(filter) || filter_grid::empty(result) {
        return Ok(());
    }
    let low = minima(filter);
    let high = maxima(filter);
    let result_low = minima(result);
    let filter_size = [filter.width(), filter.height()];
    let result_size = [result.width(), result.height()];
    if (0..2).any(|axis| {
        let start = low[axis].max(result_low[axis]);
        start - low[axis] >= filter_size[axis] || start - result_low[axis] >= result_size[axis]
    }) {
        return Ok(());
    }
    for (max, min) in high.into_iter().zip(low) {
        account.work(RenderWork::CoordinateVisits, 1)?;
        positive(max - min, Phase::GridExtent)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RasterAxis {
    count: i64,
    raster_anchor: i64,
    raster_phase: f64,
    step: RasterStep,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RasterStep {
    rounded: f64,
    residual: f64,
}

impl RasterStep {
    fn mul_add(self, index: f64, base: f64) -> f64 {
        index.mul_add(self.residual, index.mul_add(self.rounded, base))
    }

    fn divide(self, value: f64) -> f64 {
        let approximate = value / self.rounded;
        approximate + self.mul_add(-approximate, value) / self.rounded
    }

    fn reciprocal(self) -> Result<Self> {
        let rounded = positive(1.0 / self.rounded, Phase::GridMapping)?;
        let residual = finite(
            self.mul_add(-rounded, 1.0) / self.rounded,
            Phase::GridMapping,
        )?;
        Ok(Self { rounded, residual })
    }
}

impl RasterAxis {
    fn try_new(
        origin: f64,
        length: f64,
        spacing: f64,
        rho: f64,
        implicit: bool,
        account: &Account,
    ) -> Result<Self> {
        account.work(RenderWork::CoordinateVisits, 1)?;
        positive(spacing, Phase::GridExtent)?;
        // Default kernel cells and raster cells have exactly the same length.
        // Compute their count in device units so rounding the reciprocal
        // before taking the ceiling cannot introduce an extra cell.
        let (count, rounded, residual) = if implicit {
            let [anchor, phase] = split_raster_edge(length, rho, Phase::GridExtent)?;
            (
                exact_integer((anchor + phase.ceil()).max(1.0), Phase::GridExtent)?,
                1.0,
                0.0,
            )
        } else {
            let rounded = positive(spacing * rho, Phase::GridMapping)?;
            (
                canvas_count(length, spacing)?,
                rounded,
                spacing.mul_add(rho, -rounded),
            )
        };
        let step = RasterStep { rounded, residual };

        // Split the F origin into an integer raster anchor and short residual
        // before computing either affine map. The existing lattice represents
        // an aligned edge as integer / rho in logical coordinates.
        let [anchor, raster_phase] = split_raster_edge(origin, rho, Phase::GridMapping)?;
        let raster_anchor = exact_index(anchor)?;
        Ok(Self {
            count,
            raster_anchor,
            raster_phase,
            step,
        })
    }

    fn reach_shade(
        self,
        output_origin: i64,
        output_count: u32,
        account: &Account,
    ) -> Result<ShadeReach> {
        account.work(RenderWork::CoordinateVisits, 1)?;
        let delta = exact_delta(output_origin, self.raster_anchor)?;
        let step = self.step.reciprocal()?;
        // The seed only chooses a nearby integer basis. Recompute the phase
        // relative to it so a distant F edge does not discard the half cell.
        let seed =
            exact_index((self.step.divide((delta - self.raster_phase) + 0.5) - 0.5).round())?;
        let base = self.shade_coordinate(output_origin, seed)?;
        let last_pixel = output_count - 1;
        let last_seed = add_index(
            seed,
            exact_index(step.mul_add(f64::from(last_pixel), base).round())?,
        )?;
        let last =
            self.shade_coordinate(add_index(output_origin, i64::from(last_pixel))?, last_seed)?;
        let first_floor = add_index(seed, exact_index(base.floor())?)?;
        let last_floor = add_index(last_seed, exact_index(last.floor())?)?;
        let last_cell = self.count - 1;
        let lo = first_floor.clamp(0, last_cell);
        let hi = add_index(last_floor, 1)?.clamp(0, last_cell);
        let range = IndexRange {
            start: lo,
            end: add_index(hi, 1)?,
        };
        let rebased = finite(base + exact_delta(seed, lo)?, Phase::GridMapping)?;
        let map = qualify_map(
            rebased,
            step,
            output_count,
            range.width()?,
            AddressMode::Clamp {
                at_min: lo == 0,
                at_max: hi == last_cell,
            },
            CanonicalMap::Resolve {
                axis: self,
                raster_origin: output_origin,
                kernel_origin: lo,
            },
            account,
        )?;
        Ok(ShadeReach { range, map })
    }

    fn bump_sample_base(self, origin: i64, kernel_index: i64) -> Result<f64> {
        self.node_coordinate(origin, kernel_index, -0.5)
    }

    fn shade_coordinate(self, raster_index: i64, kernel_index: i64) -> Result<f64> {
        // Measure from the actual center near the destination pixel. Adding
        // an inverse stride to a rounded affine base can miss an exact node.
        let center_offset = self.bump_sample_base(raster_index, kernel_index)?;
        finite(self.step.divide(-center_offset), Phase::GridMapping)
    }

    fn node_coordinate(self, origin: i64, kernel_index: i64, center_offset: f64) -> Result<f64> {
        let delta = exact_delta(origin, self.raster_anchor)?;
        let edge = self.step.mul_add(kernel_index as f64, -delta);
        let half = self.step.mul_add(0.5, center_offset);
        finite((edge + self.raster_phase) + half, Phase::GridMapping)
    }

    fn capture_floor(self, index: i64, account: &Account) -> Result<i64> {
        account.work(RenderWork::CoordinateVisits, 1)?;
        // Pick an origin close to the tap before flooring. An absolute f64
        // center index can already have lost its phase by this point.
        let edge = self.step.mul_add(index as f64, self.raster_anchor as f64);
        let seed = exact_index((edge + self.step.mul_add(0.5, -0.5)).round())?;
        add_index(
            seed,
            exact_index(self.bump_sample_base(seed, index)?.floor())?,
        )
    }

    fn first_capture_floor(
        self,
        mut range: IndexRange,
        threshold: f64,
        account: &Account,
    ) -> Result<i64> {
        // Capture tap indices increase monotonically with the kernel node.
        // A coarse kernel may leave untouched gaps inside the outer envelope.
        while range.start < range.end {
            let middle = range.start + (range.end - range.start) / 2;
            if (self.capture_floor(middle, account)? as f64) < threshold {
                range.start = middle + 1;
            } else {
                range.end = middle;
            }
        }
        Ok(range.start)
    }
}

struct ShadeReach {
    range: IndexRange,
    map: [f32; 2],
}

#[derive(Clone, Copy)]
struct IndexRange {
    start: i64,
    end: i64,
}

impl IndexRange {
    fn width(self) -> Result<u32> {
        let width = self
            .end
            .checked_sub(self.start)
            .ok_or_else(|| numeric_error(Phase::GridIndex, self.end as f64))?;
        u32::try_from(width).map_err(|_| numeric_error(Phase::GridIndex, width as f64))
    }

    fn with_sobel_neighbors(self, count: i64) -> Result<Self> {
        Ok(Self {
            start: add_index(self.start, -1)?.max(0),
            end: if self.end < count {
                add_index(self.end, 1)?
            } else {
                count
            },
        })
    }
}

impl LightingWindow {
    fn try_new(context: FrameContext, ranges: [IndexRange; 2], counts: [i64; 2]) -> Result<Self> {
        // Bits follow the WGSL x-min/x-max/y-min/y-max edge layout.
        let edge_flags = u32::from(ranges[0].start == 0)
            | (u32::from(ranges[0].end == counts[0]) << 1)
            | (u32::from(ranges[1].start == 0) << 2)
            | (u32::from(ranges[1].end == counts[1]) << 3);
        Ok(Self {
            index_origin: ranges.map(|range| range.start),
            spatial: filter_grid::allocation_spatial(
                context,
                [ranges[0].width()?, ranges[1].width()?],
                OWNER,
            )?,
            edge_flags,
        })
    }
}

fn prepare_capture(
    source: Rect,
    output: NonEmptyFrameSpatialPlan,
    axes: [RasterAxis; 2],
    bump: [IndexRange; 2],
    account: &Account,
) -> Result<Option<LightingCapture>> {
    let rho = output.raster_scale.get();
    let source_min = minima(source);
    let source_max = maxima(source);
    let mut origins = [0; 2];
    let mut size = [0; 2];
    for axis in 0..2 {
        account.work(RenderWork::CoordinateVisits, 1)?;
        let a = axes[axis];
        let first = a.capture_floor(bump[axis].start, account)?;
        let last = a.capture_floor(add_index(bump[axis].end, -1)?, account)?;
        // Both bilinear taps are reached, even when one has zero weight.
        let [domain_lo, domain_hi] = outward_edges(source_min[axis], source_max[axis], rho)?;
        if domain_lo >= add_index(last, 2)? as f64 || domain_hi <= first as f64 {
            return Ok(None);
        }
        let lo_node = if domain_lo <= add_index(first, 1)? as f64 {
            bump[axis].start
        } else {
            a.first_capture_floor(bump[axis], domain_lo - 1.0, account)?
        };
        let hi_node = if domain_hi > last as f64 {
            bump[axis].end
        } else {
            a.first_capture_floor(bump[axis], domain_hi, account)?
        };
        if lo_node >= hi_node {
            return Ok(None);
        }
        let lo_tap = a.capture_floor(lo_node, account)?;
        let hi_tap = a.capture_floor(add_index(hi_node, -1)?, account)?;
        let lo = exact_index((lo_tap as f64).max(domain_lo))?;
        let hi = exact_index((add_index(hi_tap, 2)? as f64).min(domain_hi))?;
        origins[axis] =
            i32::try_from(lo).map_err(|_| numeric_error(Phase::GridIndex, lo as f64))?;
        i32::try_from(hi).map_err(|_| numeric_error(Phase::GridIndex, hi as f64))?;
        size[axis] = IndexRange { start: lo, end: hi }.width()?;
    }
    let spatial = capture_spatial(output, origins, size)?;
    let clip = raster_clip(source, spatial, account)?;
    let mut sample_map = [0.0; 4];
    for axis in 0..2 {
        let a = axes[axis];
        let origin = i64::from(origins[axis]);
        let base = a.bump_sample_base(origin, bump[axis].start)?;
        let [base, step] = qualify_map(
            base,
            a.step,
            bump[axis].width()?,
            size[axis],
            AddressMode::Transparent,
            CanonicalMap::Resample {
                axis: a,
                raster_origin: origin,
                kernel_origin: bump[axis].start,
            },
            account,
        )?;
        sample_map[axis] = base;
        sample_map[axis + 2] = step;
    }
    Ok(Some(LightingCapture {
        spatial,
        clip,
        sample_map,
    }))
}

fn outward_edges(minimum: f64, maximum: f64, rho: f64) -> Result<[f64; 2]> {
    let [lo, lo_phase] = split_raster_edge(minimum, rho, Phase::GridMapping)?;
    let [hi, hi_phase] = split_raster_edge(maximum, rho, Phase::GridMapping)?;
    Ok([lo + lo_phase.floor(), hi + hi_phase.ceil()])
}

fn split_raster_edge(edge: f64, rho: f64, phase: Phase) -> Result<[f64; 2]> {
    let anchor = finite(edge * rho, phase)?.round();
    // The logical raster boundary uses anchor / rho. For nonaligned edges,
    // retain the part the absolute rounded multiplication would discard.
    if edge == anchor / rho {
        return Ok([anchor, 0.0]);
    }
    let residual = finite(edge.mul_add(rho, -anchor), phase)?;
    if residual == 0.0 {
        return Err(numeric_error(phase, edge));
    }
    Ok([anchor, residual])
}

fn output_spatial(
    domain: Rect,
    density: RasterScale,
    account: &Account,
) -> Result<NonEmptyFrameSpatialPlan> {
    let min = minima(domain);
    let max = maxima(domain);
    let mut origin = [0; 2];
    let mut size = [0; 2];
    for axis in 0..2 {
        account.work(RenderWork::CoordinateVisits, 1)?;
        let [lo, hi] = outward_edges(min[axis], max[axis], density.get())?;
        let lo = exact_index(lo)?;
        let hi = exact_index(hi)?;
        origin[axis] = i32::try_from(lo).map_err(|_| numeric_error(Phase::GridIndex, lo as f64))?;
        i32::try_from(hi).map_err(|_| numeric_error(Phase::GridIndex, hi as f64))?;
        size[axis] = IndexRange { start: lo, end: hi }.width()?;
    }
    raster_spatial(domain, density, origin, size)
}

fn capture_spatial(
    output: NonEmptyFrameSpatialPlan,
    origin: [i32; 2],
    size: [u32; 2],
) -> Result<NonEmptyFrameSpatialPlan> {
    let density = output.raster_scale;
    let rho = density.get();
    let [x, y] = origin.map(|index| f64::from(index) / rho);
    let [width, height] = size.map(|count| f64::from(count) / rho);
    raster_spatial(Rect::new(x, y, width, height), density, origin, size)
}

fn raster_spatial(
    bounds: Rect,
    density: RasterScale,
    origin: [i32; 2],
    size: [u32; 2],
) -> Result<NonEmptyFrameSpatialPlan> {
    for (index, count) in size.into_iter().enumerate() {
        if count == 0 || f64::from(count) > f64::from(SAMPLE_LIMIT) {
            return Err(numeric_error(Phase::GridIndex, f64::from(count)));
        }
        positive(
            maxima(bounds)[index] - minima(bounds)[index],
            Phase::GridMapping,
        )?;
    }
    let dimension = bounds.width().max(bounds.height());
    let logical = LogicalBounds::try_from_rect(bounds, "lighting raster window")
        .map_err(|_| numeric_error(Phase::GridMapping, dimension))?;
    let LogicalBounds::NonEmpty(logical_bounds) = logical else {
        return Err(numeric_error(Phase::GridMapping, dimension));
    };
    let [x, y] = origin.map(|index| f64::from(index) / density.get());
    Ok(NonEmptyFrameSpatialPlan {
        logical_bounds,
        device_origin: SignedDeviceOrigin::new(origin[0], origin[1]),
        device_extent: PositiveDeviceExtent {
            width: size[0],
            height: size[1],
        },
        raster_scale: density,
        texel_center_mapping: TexelCenterMapping {
            origin: Point::try_new(x, y)
                .map_err(|_| numeric_error(Phase::GridMapping, x.abs().max(y.abs())))?,
            raster_scale: density,
        },
    })
}

fn canvas_count(length: f64, spacing: f64) -> Result<i64> {
    if length <= spacing {
        return Ok(1);
    }
    let n = exact_integer((length / spacing).ceil(), Phase::GridExtent)?;
    // Correct a quotient rounded onto an integer before ceil. The fused
    // residual determines which side of the candidate boundary F actually ends.
    if (-(n as f64)).mul_add(spacing, length) > 0.0 {
        exact_integer((n + 1) as f64, Phase::GridExtent)
    } else if (-((n - 1) as f64)).mul_add(spacing, length) <= 0.0 {
        Ok(n - 1)
    } else {
        Ok(n)
    }
}

fn raster_clip(
    domain: Rect,
    spatial: NonEmptyFrameSpatialPlan,
    account: &Account,
) -> Result<[f32; 4]> {
    let [x, y] = minima(domain);
    let [right, bottom] = maxima(domain);
    let rho = spatial.raster_scale.get();
    let [ox, oy] = device_origin(spatial);
    let [w, h] = filter_grid::extent(spatial);
    let mut clip = [0.0; 4];
    for (index, (edge, anchor, size)) in [(x, ox, w), (y, oy, h), (right, ox, w), (bottom, oy, h)]
        .into_iter()
        .enumerate()
    {
        account.work(RenderWork::CoordinateVisits, 1)?;
        let [base, residual] = split_raster_edge(edge, rho, Phase::GridMapping)?;
        let base = base - f64::from(anchor);
        let extent = f64::from(size);
        let relative = (base + residual).clamp(0.0, extent);
        let value = uploaded(relative, Phase::GridMapping)?;
        let widened = f64::from(value);
        // Derive the expected covered cells before rounding the rebased sum.
        // A very thin sliver can disappear even in the f64 sum near an edge.
        if widened.floor() != (base + residual.floor()).clamp(0.0, extent)
            || widened.ceil() != (base + residual.ceil()).clamp(0.0, extent)
        {
            return Err(numeric_error(Phase::GridMapping, relative));
        }
        clip[index] = value;
    }
    for axis in 0..2 {
        if clip[axis + 2] <= clip[axis] {
            return Err(numeric_error(Phase::GridMapping, f64::from(clip[axis + 2])));
        }
    }
    Ok(clip)
}

#[derive(Clone, Copy)]
enum AddressMode {
    Clamp { at_min: bool, at_max: bool },
    Transparent,
}

#[derive(Clone, Copy)]
enum CanonicalMap {
    Resolve {
        axis: RasterAxis,
        raster_origin: i64,
        kernel_origin: i64,
    },
    Resample {
        axis: RasterAxis,
        raster_origin: i64,
        kernel_origin: i64,
    },
}

impl CanonicalMap {
    fn aligned(self) -> bool {
        let axis = match self {
            Self::Resolve { axis, .. } | Self::Resample { axis, .. } => axis,
        };
        axis.raster_phase == 0.0 && axis.step.rounded == 1.0 && axis.step.residual == 0.0
    }

    fn sample(self, pixel: u32, predicted: f64) -> Result<(i64, f64)> {
        let near = exact_index(predicted.round())?;
        let p = i64::from(pixel);
        let residual = match self {
            Self::Resolve {
                axis,
                raster_origin,
                kernel_origin,
            } => axis.shade_coordinate(
                add_index(raster_origin, p)?,
                add_index(kernel_origin, near)?,
            )?,
            Self::Resample {
                axis,
                raster_origin,
                kernel_origin,
            } => axis.bump_sample_base(
                add_index(raster_origin, near)?,
                add_index(kernel_origin, p)?,
            )?,
        };
        Ok((near, residual))
    }
}

fn qualify_map(
    base: f64,
    step: RasterStep,
    target_count: u32,
    source_count: u32,
    mode: AddressMode,
    canonical: CanonicalMap,
    account: &Account,
) -> Result<[f32; 2]> {
    let base32 = uploaded(base, Phase::GridMapping)?;
    let step_value = positive(step.rounded + step.residual, Phase::GridMapping)?;
    let step32 = uploaded(step_value, Phase::GridMapping)?;
    let map = [base32, step32];
    let last = target_count - 1;
    let mut previous =
        qualify_sample(0, (base, step), map, source_count, mode, canonical, account)?;
    if last == 0 {
        return Ok(map);
    }
    // This proof covers aligned maps such as the default one-cell stride.
    // Each integer product is exactly representable through 2^24. Once the
    // endpoints qualify, every sum lies within the exact sampling range and
    // increases by an integer at each step.
    let exact_integral_map = canonical.aligned()
        && base == f64::from(base32)
        && step.residual == 0.0
        && step_value == f64::from(step32)
        && base.fract() == 0.0
        && step_value.fract() == 0.0
        && f64::from(last) * step_value <= 2.0 * f64::from(SAMPLE_LIMIT);
    if exact_integral_map {
        qualify_sample(
            last,
            (base, step),
            map,
            source_count,
            mode,
            canonical,
            account,
        )?;
        return Ok(map);
    }
    // Monotone endpoints bound an affine map but cannot prove that rounding
    // preserves the floor or fraction at an interior crossing. Qualify each
    // reached coordinate on this axis, for both shader evaluation orders.
    for pixel in 1..target_count {
        let actual = qualify_sample(
            pixel,
            (base, step),
            map,
            source_count,
            mode,
            canonical,
            account,
        )?;
        for (coordinate, prior) in actual.into_iter().zip(previous) {
            if coordinate <= prior {
                return Err(numeric_error(Phase::Sampling, f64::from(coordinate)));
            }
        }
        previous = actual;
    }

    Ok(map)
}

fn qualify_sample(
    pixel: u32,
    (base, step): (f64, RasterStep),
    [base32, step32]: [f32; 2],
    source_count: u32,
    mode: AddressMode,
    canonical: CanonicalMap,
    account: &Account,
) -> Result<[f32; 2]> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    let predicted = finite(step.mul_add(f64::from(pixel), base), Phase::GridMapping)?;
    let (near, residual) = canonical.sample(pixel, predicted)?;
    let exact = near as f64 + residual;
    let exact_floor = add_index(near, exact_index(residual.floor())?)?;
    let exact_taps = [exact_floor, add_index(exact_floor, 1)?];
    let expected = named_taps(exact_taps, source_count, mode, exact)?;
    let pixel = pixel as f32;
    let coordinates = [pixel * step32 + base32, pixel.mul_add(step32, base32)];
    for actual in coordinates {
        if !actual.is_finite() || actual.abs() >= SAMPLE_LIMIT {
            return Err(numeric_error(Phase::Sampling, f64::from(actual)));
        }
        let actual_floor = i64::from(actual.floor() as i32);
        let actual_taps = [actual_floor, add_index(actual_floor, 1)?];
        if expected != named_taps(actual_taps, source_count, mode, exact)? {
            return Err(numeric_error(Phase::Sampling, exact));
        }
        let actual_fraction = actual - actual.floor();
        if expected[0] != expected[1]
            && ((residual == residual.floor()) != (actual_fraction == 0.0)
                || actual_fraction >= 1.0)
        {
            return Err(numeric_error(Phase::Sampling, exact));
        }
    }
    Ok(coordinates)
}

fn named_taps(
    taps: [i64; 2],
    source_count: u32,
    mode: AddressMode,
    coordinate: f64,
) -> Result<[i64; 2]> {
    let count = i64::from(source_count);
    let valid = 0..count;
    match mode {
        AddressMode::Clamp { at_min, at_max } => {
            let taps = taps.map(|i| {
                let i = if at_min { i.max(0) } else { i };
                if at_max { i.min(count - 1) } else { i }
            });
            if taps.iter().all(|i| valid.contains(i)) {
                Ok(taps)
            } else {
                Err(numeric_error(Phase::Sampling, coordinate))
            }
        }
        AddressMode::Transparent => Ok(taps.map(|i| if valid.contains(&i) { i } else { -1 })),
    }
}

fn minima(rect: Rect) -> [f64; 2] {
    [rect.x(), rect.y()]
}

fn maxima(rect: Rect) -> [f64; 2] {
    [rect.max().x(), rect.max().y()]
}

fn device_origin(spatial: NonEmptyFrameSpatialPlan) -> [i32; 2] {
    [spatial.device_origin.x, spatial.device_origin.y]
}

fn finite(value: f64, phase: Phase) -> Result<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(numeric_error(phase, value))
    }
}

fn positive(value: f64, phase: Phase) -> Result<f64> {
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(numeric_error(phase, value))
    }
}

fn uploaded(value: f64, phase: Phase) -> Result<f32> {
    let upload = value as f32;
    if upload.is_normal() || value == 0.0 {
        Ok(upload)
    } else {
        Err(numeric_error(phase, value))
    }
}

fn exact_index(value: f64) -> Result<i64> {
    exact_integer(value, Phase::GridIndex)
}

fn exact_integer(value: f64, phase: Phase) -> Result<i64> {
    if value.abs() < EXACT_INTEGER_LIMIT && value.fract() == 0.0 {
        Ok(value as i64)
    } else {
        Err(numeric_error(phase, value))
    }
}

fn add_index(lhs: i64, rhs: i64) -> Result<i64> {
    let sum = lhs
        .checked_add(rhs)
        .ok_or_else(|| numeric_error(Phase::GridIndex, lhs as f64))?;
    exact_index(sum as f64)
}

fn exact_delta(lhs: i64, rhs: i64) -> Result<f64> {
    let delta = lhs
        .checked_sub(rhs)
        .ok_or_else(|| numeric_error(Phase::GridIndex, lhs as f64))?;
    exact_index(delta as f64)?;
    Ok(delta as f64)
}

#[cfg(test)]
mod tests;
