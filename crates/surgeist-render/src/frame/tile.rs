//! Sparse capture and periodic reconstruction for a resolved filter producer.

use super::{
    FilterOutputFootprint, FrameContext, GraphLoweringSpatialDescriptor,
    bounds::{
        LogicalBounds, NonEmptyFrameSpatialPlan, PositiveDeviceExtent, RasterScale,
        SignedDeviceOrigin, TexelCenterMapping,
    },
    convolution::intersection,
    filter_grid::{self, FilterGridOwner},
};
use crate::{
    Error, Result,
    accounting::{Account, RenderWork},
    filter_domains::FilterDomains,
    geometry::{Point, Rect, Transform},
    number_text::Number,
};

const UNIFORM_SIZE: usize = 80;
// A binary32 fragment position distinguishes all centers up to this extent.
const MAX_RASTER_EXTENT: u32 = 1 << 23;
// Both bilinear axes together may displace a normalized output by one RGBA8 code.
const MAX_CHANNEL_ERROR: f64 = 1.0 / 255.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TilePlan {
    output: NonEmptyFrameSpatialPlan,
    output_clip: [f64; 4],
    capture: Option<TileCapture>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TileCapture {
    pub(super) context: FrameContext,
    pub(super) spatial: NonEmptyFrameSpatialPlan,
    pub(super) transform: Transform,
    period_window: [u32; 4],
    source_clip: [f32; 4],
    phase_step: [f32; 4],
}

pub(super) fn source_domain(domains: FilterDomains) -> Rect {
    intersection(domains.source(), domains.filter())
}

pub(super) fn output_domain(domains: FilterDomains) -> Rect {
    intersection(domains.result(), domains.filter())
}

pub(super) fn capture_basis(
    context: FrameContext,
    domains: FilterDomains,
    transform: Transform,
    account: &Account,
) -> Result<Option<(FrameContext, Transform)>> {
    let Some(density) = context.local_raster_scale(transform)? else {
        return Ok(None);
    };
    Ok(
        CaptureBasis::try_new(context, domains, density.get(), account)?
            .map(|basis| (basis.context, basis.transform)),
    )
}

impl TilePlan {
    pub(super) fn try_new(
        context: FrameContext,
        domains: FilterDomains,
        transform: Transform,
        source_reached: bool,
        account: &Account,
    ) -> Result<Option<Self>> {
        let output_domain = output_domain(domains);
        if filter_grid::empty(output_domain) {
            return Ok(None);
        }
        let Some(density) = context.local_raster_scale(transform)? else {
            return Ok(None);
        };
        account.work(RenderWork::FilterOperationVisits, 1)?;
        let (output, output_clip) = output_raster(output_domain, density, account)?;

        let capture = if source_reached {
            CaptureBasis::try_new(context, domains, density.get(), account)?
                .map(|basis| basis.resolve(output, account))
                .transpose()?
        } else {
            None
        };
        Ok(Some(Self {
            output,
            output_clip,
            capture,
        }))
    }

    pub(super) const fn spatial(self) -> NonEmptyFrameSpatialPlan {
        self.output
    }

    pub(super) const fn capture(self) -> Option<TileCapture> {
        self.capture
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TilePassFacts {
    target: GraphLoweringSpatialDescriptor,
    source: Option<GraphLoweringSpatialDescriptor>,
    output_clip: [f64; 4],
    uniform: [u8; UNIFORM_SIZE],
    samples_per_cell: u64,
}

impl TilePassFacts {
    pub(super) fn try_new(plan: TilePlan, account: &Account) -> Result<Self> {
        let mut uniform = [0; UNIFORM_SIZE];
        let (source, samples_per_cell) = if let Some(capture) = plan.capture {
            account.work(RenderWork::SerializedBytes, UNIFORM_SIZE)?;
            let [width, height] = filter_grid::extent(capture.spatial);
            for (vector, words) in uniform.as_chunks_mut::<16>().0.iter_mut().zip([
                plan.output_clip.map(|value| (value as f32).to_bits()),
                capture.phase_step.map(f32::to_bits),
                capture.period_window,
                [width, height, 0, 0],
                capture.source_clip.map(f32::to_bits),
            ]) {
                for (word, value) in vector.as_chunks_mut::<4>().0.iter_mut().zip(words) {
                    *word = value.to_le_bytes();
                }
            }
            let samples = if capture.source_clip == [0.0, 0.0, width as f32, height as f32] {
                4
            } else {
                16
            };
            (
                Some(filter_grid::spatial_descriptor(capture.spatial)),
                samples,
            )
        } else {
            (None, 0)
        };
        Ok(Self {
            target: filter_grid::spatial_descriptor(plan.output),
            source,
            output_clip: plan.output_clip,
            uniform,
            samples_per_cell,
        })
    }

    pub(crate) const fn target_plan(&self) -> GraphLoweringSpatialDescriptor {
        self.target
    }

    pub(crate) const fn source_plan(&self) -> Option<GraphLoweringSpatialDescriptor> {
        self.source
    }

    pub(crate) const fn output_clip(&self) -> [f64; 4] {
        self.output_clip
    }

    pub(crate) const fn output_footprint(&self) -> FilterOutputFootprint {
        FilterOutputFootprint::from_qualified_grid(self.target, self.output_clip)
    }

    pub(crate) const fn uniform_bytes(&self) -> &[u8; UNIFORM_SIZE] {
        &self.uniform
    }

    pub(crate) const fn samples_per_cell(&self) -> u64 {
        self.samples_per_cell
    }
}

struct CaptureBasis {
    context: FrameContext,
    spatial: NonEmptyFrameSpatialPlan,
    transform: Transform,
    axes: [CaptureAxis; 2],
}

impl CaptureBasis {
    fn try_new(
        context: FrameContext,
        domains: FilterDomains,
        rho: f64,
        account: &Account,
    ) -> Result<Option<Self>> {
        let domain = source_domain(domains);
        if filter_grid::empty(domain) {
            return Ok(None);
        }
        let tile = domains.source();
        let [x, y] = rect_axes(tile);
        let [dx, dy] = rect_axes(domain);
        let axes = [
            CaptureAxis::try_new(x, tile.width(), dx, rho, account)?,
            CaptureAxis::try_new(y, tile.height(), dy, rho, account)?,
        ];
        let [x, y] = axes;
        let size = [x.width, y.width];
        let spatial = filter_grid::allocation_spatial(context, size, FilterGridOwner::Tile)?;
        let mut scoped = context;
        scoped.surface_scale = 1.0;
        scoped.output_bounds = LogicalBounds::NonEmpty(spatial.logical_bounds);
        let transform = Transform::try_new_in(
            [
                x.storage_density,
                0.0,
                0.0,
                y.storage_density,
                x.storage_translate,
                y.storage_translate,
            ],
            account,
        )?;
        Ok(Some(Self {
            context: scoped,
            spatial,
            transform,
            axes,
        }))
    }

    fn resolve(self, output: NonEmptyFrameSpatialPlan, account: &Account) -> Result<TileCapture> {
        let [x, y] = self.axes;
        let rho = output.raster_scale.get();
        let output_extent = filter_grid::extent(output);
        let (phase_x, x_error) =
            x.resolve_phase(output.device_origin.x, output_extent[0], rho, account)?;
        let (phase_y, y_error) =
            y.resolve_phase(output.device_origin.y, output_extent[1], rho, account)?;
        let phase_error = x_error + y_error;
        if phase_error > MAX_CHANNEL_ERROR {
            return Err(numeric("sampling precision", phase_error));
        }
        Ok(TileCapture {
            context: self.context,
            spatial: self.spatial,
            transform: self.transform,
            period_window: [x.period, y.period, x.start, y.start],
            source_clip: [x.clip[0], y.clip[0], x.clip[1], y.clip[1]],
            phase_step: [phase_x[0], phase_y[0], phase_x[1], phase_y[1]],
        })
    }
}

#[derive(Clone, Copy)]
struct CaptureAxis {
    tile_min: f64,
    tile_length: f64,
    period: u32,
    density: f64,
    start: u32,
    width: u32,
    clip: [f32; 2],
    storage_density: f64,
    storage_translate: f64,
}

impl CaptureAxis {
    fn try_new(
        tile: [f64; 2],
        length: f64,
        source: [f64; 2],
        rho: f64,
        account: &Account,
    ) -> Result<Self> {
        account.work(RenderWork::CoordinateVisits, 1)?;
        let period = GridEdge::scaled(length, rho)?.ceil().max(1.0);
        if period > f64::from(i32::MAX) {
            return Err(numeric("period cells", period));
        }
        let density = period / length;
        let [lower, upper] = source;
        let [tile_min, tile_max] = tile;
        let project = |edge: f64| {
            if edge == tile_min {
                Ok(GridEdge::integer(0.0))
            } else if edge == tile_max {
                Ok(GridEdge::integer(period))
            } else {
                GridEdge::project(edge, tile_min, density, length, period)
            }
        };
        account.work(RenderWork::CoordinateVisits, 2)?;
        let projected = [project(lower)?, project(upper)?];
        let start = projected[0].floor().clamp(0.0, period);
        let stop = projected[1].ceil().clamp(0.0, period);
        let width = window_width(start, stop)?;
        let clip = [
            projected[0].relative(start, width)? as f32,
            projected[1].relative(start, width)? as f32,
        ];
        if clip[0] >= clip[1] {
            return Err(numeric("capture extent", f64::from(clip[1] - clip[0])));
        }

        // The conceptual window supplies the repeat phase and per-cell coverage.
        // Its fractional edge cells would capture ink from outside the hard filter
        // if used directly. Align the physical viewport with the reached source
        // domain while keeping the same storage extent.
        let full_window = [0.0, f64::from(width)];
        let (storage_density, storage_translate) = if clip.map(f64::from) == full_window {
            (density, (-tile_min).mul_add(density, -start))
        } else {
            let density = f64::from(width) / (upper - lower);
            (density, -lower * density)
        };
        let density_f32 = upload(storage_density, "capture density")?;
        if density_f32 <= 0.0 {
            return Err(numeric("capture density", storage_density));
        }
        let translate_f32 = upload(storage_translate, "capture translation")?;
        for (edge, expected) in source.into_iter().zip(full_window) {
            account.work(RenderWork::CoordinateVisits, 1)?;
            // The reached physical endpoints are always 0 and the storage extent.
            // Preserve them through the actual affine evaluation and upload.
            let coordinate = edge as f32;
            for actual in [
                edge.mul_add(storage_density, storage_translate),
                edge * storage_density + storage_translate,
                f64::from(coordinate.mul_add(density_f32, translate_f32)),
                f64::from(coordinate * density_f32 + translate_f32),
            ] {
                let error = (expected - actual).abs();
                if !error.is_finite() || error > MAX_CHANNEL_ERROR {
                    return Err(numeric("capture precision", error));
                }
            }
        }

        Ok(Self {
            tile_min,
            tile_length: length,
            period: period as u32,
            density,
            start: start as u32,
            width,
            clip,
            storage_density,
            storage_translate,
        })
    }

    fn resolve_phase(
        self,
        output_origin: i32,
        width: u32,
        rho: f64,
        account: &Account,
    ) -> Result<([f32; 2], f64)> {
        account.work(RenderWork::CoordinateVisits, 1)?;
        // Every wrapped neighbor is cell zero in a singleton period.
        if self.period == 1 {
            return Ok(([0.0; 2], 0.0));
        }

        let first_device_center = f64::from(output_origin) + 0.5;
        let center = first_device_center / rho;
        let residual = (-center).mul_add(rho, first_device_center) / rho;
        let length = self.tile_length;
        // Retain the residual from the device-to-local quotient and reduce
        // before multiplication by density. The centered representative limits
        // precision loss and shifts only by an integer number of full periods.
        let phase = ((center % length - self.tile_min % length) % length + residual) % length;
        let phase = if phase > length * 0.5 {
            phase - length
        } else if phase < -length * 0.5 {
            phase + length
        } else {
            phase
        };
        let base = phase.mul_add(self.density, -0.5);
        let step = self.density / rho;
        let base_f32 = upload(base, "sample phase")?;
        let step_f32 = upload(step, "sample step")?;
        let last = width - 1;
        let mut error = 0.0_f64;
        let mut max_gpu = 0.0_f32;
        for index in [0, last] {
            account.work(RenderWork::CoordinateVisits, 1)?;
            let exact = f64::from(index).mul_add(step, base);
            let pixel = index as f32;
            let separate = pixel * step_f32 + base_f32;
            let fused = pixel.mul_add(step_f32, base_f32);
            for coordinate in [exact, f64::from(separate), f64::from(fused)] {
                // The shader floors, converts to signed integer, then forms
                // the neighboring index before wrapping on the full period.
                if coordinate.floor() < f64::from(i32::MIN)
                    || coordinate.floor() + 1.0 > f64::from(i32::MAX)
                {
                    return Err(numeric("sample index", coordinate));
                }
                error = error.max((exact - coordinate).abs());
            }
            max_gpu = max_gpu.max(fused.abs()).max(separate.abs());
        }
        let exact_aligned = step == 1.0
            && base == f64::from(base_f32)
            && base.fract() == 0.0
            && max_gpu <= (1_u32 << 24) as f32;
        if width > 2 && !exact_aligned {
            // The affine endpoints bound the largest input, product and
            // result over the interior. Include one rounding for each step of
            // the separate multiply/add and both narrowed coefficients.
            let coefficient_error = (base - f64::from(base_f32)).abs()
                + f64::from(last) * (step - f64::from(step_f32)).abs();
            let rounding_error = half_ulp(last as f32 * step_f32) + half_ulp(max_gpu);
            error = error.max(coefficient_error + rounding_error);
        }
        Ok(([base_f32, step_f32], error))
    }
}

fn rect_axes(rect: Rect) -> [[f64; 2]; 2] {
    [[rect.x(), rect.max().x()], [rect.y(), rect.max().y()]]
}

fn output_raster(
    domain: Rect,
    raster_scale: RasterScale,
    account: &Account,
) -> Result<(NonEmptyFrameSpatialPlan, [f64; 4])> {
    let mut origin = [0; 2];
    let mut size = [0; 2];
    let mut clip = [0.0; 4];
    let rho = raster_scale.get();
    for (axis, [minimum, maximum]) in rect_axes(domain).into_iter().enumerate() {
        account.work(RenderWork::CoordinateVisits, 2)?;
        let lower = GridEdge::absolute(minimum, rho)?;
        let upper = GridEdge::absolute(maximum, rho)?;
        let lo = lower.floor();
        let hi = upper.ceil();
        let signed = f64::from(i32::MIN)..=f64::from(i32::MAX);
        if !signed.contains(&lo) || !signed.contains(&hi) {
            return Err(numeric("output index", lo.abs().max(hi.abs())));
        }
        origin[axis] = lo as i32;
        size[axis] = window_width(lo, hi)?;
        clip[axis] = lower.relative(lo, size[axis])?;
        clip[axis + 2] = upper.relative(lo, size[axis])?;
        if clip[axis] as f32 >= clip[axis + 2] as f32 {
            return Err(numeric("output clip", clip[axis + 2]));
        }
    }
    let LogicalBounds::NonEmpty(logical_bounds) =
        LogicalBounds::try_from_rect(domain, "tile output bounds")?
    else {
        return Err(numeric("output extent", 0.0));
    };
    let [x, y] = origin.map(|index| f64::from(index) / rho);
    Ok((
        NonEmptyFrameSpatialPlan {
            logical_bounds,
            device_origin: SignedDeviceOrigin::new(origin[0], origin[1]),
            device_extent: PositiveDeviceExtent {
                width: size[0],
                height: size[1],
            },
            raster_scale,
            texel_center_mapping: TexelCenterMapping {
                origin: Point::try_new(x, y)?,
                raster_scale,
            },
        },
        clip,
    ))
}

#[derive(Clone, Copy)]
struct GridEdge {
    integer: f64,
    residual: f64,
}

impl GridEdge {
    fn integer(integer: f64) -> Self {
        Self {
            integer,
            residual: 0.0,
        }
    }

    fn scaled(value: f64, density: f64) -> Result<Self> {
        let integer = (value * density).round();
        if !integer.is_finite() {
            return Err(numeric("grid coordinate", integer));
        }
        Ok(Self {
            integer,
            residual: value.mul_add(density, -integer),
        })
    }

    fn absolute(value: f64, density: f64) -> Result<Self> {
        let mut edge = Self::scaled(value, density)?;
        // The inverse lattice mapping canonically names this exact device edge.
        if value == edge.integer / density {
            edge.residual = 0.0;
        }
        Ok(edge)
    }

    fn project(edge: f64, origin: f64, density: f64, length: f64, period: f64) -> Result<Self> {
        let delta = edge - origin;
        let mut projected = Self::scaled(delta, density)?;
        // Recover the subtraction error and the correction to period/length,
        // so evaluating an interior edge near an integer preserves its side.
        let virtual_origin = edge - delta;
        let delta_error = (edge - (delta + virtual_origin)) + (virtual_origin - origin);
        let density_error = (-density).mul_add(length, period) / length;
        projected.residual = delta_error.mul_add(density, projected.residual);
        projected.residual = delta.mul_add(density_error, projected.residual);
        if projected.residual.is_finite() {
            Ok(projected)
        } else {
            Err(numeric("source grid edge", projected.residual))
        }
    }

    fn floor(self) -> f64 {
        self.integer + self.residual.floor()
    }

    fn ceil(self) -> f64 {
        self.integer + self.residual.ceil()
    }

    fn relative(self, start: f64, width: u32) -> Result<f64> {
        let extent = f64::from(width);
        let relative = ((self.integer - start) + self.residual).clamp(0.0, extent);
        let narrow = f64::from(upload(relative, "raster clip")?);
        // Preserve whether a fractional boundary reaches either edge cell.
        if narrow.floor() != (self.floor() - start).clamp(0.0, extent)
            || narrow.ceil() != (self.ceil() - start).clamp(0.0, extent)
            || (relative - narrow).abs() > MAX_CHANNEL_ERROR
        {
            return Err(numeric("raster clip", relative));
        }
        Ok(relative)
    }
}

fn window_width(lo: f64, hi: f64) -> Result<u32> {
    let width = hi - lo;
    if width >= 1.0 && width <= f64::from(MAX_RASTER_EXTENT) {
        Ok(width as u32)
    } else {
        Err(numeric("raster window", width))
    }
}

fn half_ulp(value: f32) -> f64 {
    f64::from(value.next_up() - value) * 0.5
}

fn upload(value: f64, field: &str) -> Result<f32> {
    let narrowed = value as f32;
    if narrowed.is_finite() && (value == 0.0 || narrowed.is_normal()) {
        Ok(narrowed)
    } else {
        Err(numeric(field, value))
    }
}

fn numeric(field: &str, value: f64) -> Error {
    Error::invalid_value(
        format!("tile {field}"),
        Number(value),
        "cannot represent the reached periodic sampling lattice",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planned(
        tile: Rect,
        filter: Rect,
        output: Rect,
        rho: f64,
        source_reached: bool,
    ) -> Result<Option<TilePlan>> {
        let context = FrameContext::try_for_spatial_test(rho)?;
        let domains = FilterDomains::try_new(tile, filter, output)?;
        TilePlan::try_new(
            context,
            domains,
            Transform::identity(),
            source_reached,
            &Account::for_test(),
        )
    }

    fn words(facts: &TilePassFacts, vector: usize) -> [u32; 4] {
        let start = 16 * vector;
        core::array::from_fn(|lane| {
            let offset = start + 4 * lane;
            u32::from_le_bytes(
                facts.uniform_bytes()[offset..offset + 4]
                    .try_into()
                    .unwrap(),
            )
        })
    }

    #[test]
    fn a_twenty_million_cell_period_captures_only_the_four_cells_inside_the_filter() {
        let tile = Rect::new(-20_000_000.0, 0.0, 20_000_004.0, 1.0);
        let filter = Rect::new(0.0, 0.0, 8.0, 1.0);
        let plan = planned(tile, filter, filter, 1.0, true).unwrap().unwrap();
        let capture = plan.capture().unwrap();
        assert_eq!(filter_grid::extent(capture.spatial), [4, 1]);
        assert_eq!(capture.transform, Transform::identity());
        assert_eq!(capture.context.surface_scale, 1.0);
        let facts = TilePassFacts::try_new(plan, &Account::for_test()).unwrap();
        assert_eq!(words(&facts, 2), [20_000_004, 1, 20_000_000, 0]);
        assert_eq!(words(&facts, 3), [4, 1, 0, 0]);
        assert_eq!(words(&facts, 1).map(f32::from_bits), [-4.0, 0.0, 1.0, 0.0]);
        assert_eq!(words(&facts, 4).map(f32::from_bits), [0.0, 0.0, 4.0, 1.0]);
        assert_eq!(facts.output_clip(), [0.0, 0.0, 8.0, 1.0]);
        assert_eq!(facts.samples_per_cell(), 4);
    }

    #[test]
    fn fractional_source_coverage_aligns_the_physical_capture_and_preserves_repeat_phase() {
        let tile = Rect::new(1.25, -1.5, 2.5, 1.25);
        let filter = Rect::new(0.0, -1.25, 10.0, 4.0);
        let output = Rect::new(4.0, -1.0, 3.0, 2.0);
        let plan = planned(tile, filter, output, 2.0, true).unwrap().unwrap();
        let capture = plan.capture().unwrap();
        // The conceptual Y cells use 2.4 texels per unit, while the physical
        // capture fits the reached 1-unit interval into three full texels.
        assert_eq!(
            capture.transform.as_array(),
            [2.0, 0.0, 0.0, 3.0, -2.5, 3.75]
        );
        let [_, _, _, y_scale, _, y_shift] = capture.transform.as_array();
        assert_eq!((-0.75_f64).mul_add(y_scale, y_shift), 1.5);

        // Nested source qualification uses the same capture basis even before
        // output phase and bounds are known.
        let early_basis = capture_basis(
            FrameContext::try_for_spatial_test(2.0).unwrap(),
            FilterDomains::try_new(tile, filter, output).unwrap(),
            Transform::identity(),
            &Account::for_test(),
        )
        .unwrap();
        assert_eq!(early_basis, Some((capture.context, capture.transform)));

        let facts = TilePassFacts::try_new(plan, &Account::for_test()).unwrap();
        assert_eq!(words(&facts, 2), [5, 3, 0, 0]);
        assert_eq!(words(&facts, 4).map(f32::from_bits), [0.0, 0.6, 5.0, 3.0]);
        assert_eq!(words(&facts, 1).map(f32::from_bits), [0.5, -1.7, 1.0, 1.2]);
        assert_eq!(facts.output_clip(), [0.0, 0.0, 6.0, 4.0]);
        assert_eq!(facts.samples_per_cell(), 16);
    }

    #[test]
    fn empty_or_unreached_producer_keeps_output_footprint_without_capture() {
        let filter = Rect::new(0.0, 0.0, 4.0, 3.0);
        let output = Rect::new(2.0, 1.0, 5.0, 1.0);
        let touching = Rect::new(-2.0, 0.0, 2.0, 1.0);
        for (tile, reached) in [(touching, true), (filter, false)] {
            let plan = planned(tile, filter, output, 1.0, reached)
                .unwrap()
                .unwrap();
            assert!(plan.capture().is_none());
            let account = Account::for_test();
            let facts = TilePassFacts::try_new(plan, &account).unwrap();
            assert_eq!(facts.source_plan(), None);
            assert_eq!(facts.samples_per_cell(), 0);
            assert_eq!(facts.output_clip(), [0.0, 0.0, 2.0, 1.0]);
            assert_eq!(facts.output_footprint().output_clip(), [0.0, 0.0, 2.0, 1.0]);
            assert_eq!(account.work_count(RenderWork::SerializedBytes).unwrap(), 0);
        }

        let touching_output = Rect::new(4.0, 0.0, 1.0, 1.0);
        assert!(
            planned(filter, filter, touching_output, 1.0, true)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn an_outward_edge_cannot_silently_lose_a_reached_cell_after_narrowing() {
        // Ordinary binary64 multiplication rounds this product to 9.
        let edge = 0.9_f64.next_down();
        assert_eq!(edge * 10.0, 9.0);
        let projected = GridEdge::absolute(edge, 10.0).unwrap();
        assert_eq!(projected.floor(), 8.0);
        let error = projected.relative(8.0, 1).unwrap_err();
        assert_eq!(
            error.invalid_value_diagnostic().unwrap().field(),
            "tile raster clip"
        );
    }
}
