//! Checked, layer-local convolution lattice mapping. Kernel images use their own
//! integer allocation coordinates; the two-axis mapping remains separate from the
//! isotropic capture descriptors and the full outer affine composition.
use super::{
    FrameContext,
    bounds::{FrameSpatialPlan, LogicalBounds, NonEmptyFrameSpatialPlan, PositiveDeviceExtent},
};
use crate::{
    PreparedConvolution, Rect, Transform,
    accounting::{Account, RenderWork},
    error::{Error, Result},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConvolutionStage {
    RebaseSource,
    PassThrough,
    Prepare,
    Accumulate {
        first_tap: u32,
        tap_count: u32,
        first: bool,
    },
    Finalize,
    Resolve,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum MappingUse {
    Convolution,
    PassThrough,
    Rebase,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ConvolutionMapping {
    source_domain: Rect,
    result_domain: Rect,
    unit_length: [f64; 2],
    kernel_origin: [f64; 2],
    source_grid_origin: [i32; 2],
    source_grid_extent: [u32; 2],
    result_grid_origin: [i32; 2],
    result_grid_extent: [u32; 2],
    capture_origin: [f64; 2],
    capture_scale: f64,
    result_capture_origin: [f64; 2],
    result_capture_scale: f64,
}

impl ConvolutionMapping {
    pub(crate) const fn source_domain(self) -> Rect {
        self.source_domain
    }
    pub(crate) const fn result_domain(self) -> Rect {
        self.result_domain
    }
    pub(crate) const fn unit_length(self) -> [f64; 2] {
        self.unit_length
    }
    pub(crate) const fn kernel_origin(self) -> [f64; 2] {
        self.kernel_origin
    }
    pub(crate) const fn source_grid_origin(self) -> [i32; 2] {
        self.source_grid_origin
    }
    pub(crate) const fn source_grid_extent(self) -> [u32; 2] {
        self.source_grid_extent
    }
    pub(crate) const fn result_grid_origin(self) -> [i32; 2] {
        self.result_grid_origin
    }
    pub(crate) const fn result_grid_extent(self) -> [u32; 2] {
        self.result_grid_extent
    }
    pub(crate) const fn capture_origin(self) -> [f64; 2] {
        self.capture_origin
    }
    pub(crate) const fn capture_scale(self) -> f64 {
        self.capture_scale
    }
    pub(crate) const fn result_capture_origin(self) -> [f64; 2] {
        self.result_capture_origin
    }
    pub(crate) const fn result_capture_scale(self) -> f64 {
        self.result_capture_scale
    }

    pub(super) fn try_new(
        operation: &PreparedConvolution,
        source: NonEmptyFrameSpatialPlan,
        result: NonEmptyFrameSpatialPlan,
        account: &Account,
    ) -> Result<Self> {
        let domains = operation.domains();
        Self::try_from_parts(
            intersection(domains.source(), domains.filter()),
            intersection(domains.result(), domains.filter()),
            operation.unit_length(),
            source,
            result,
            if operation.is_passthrough() {
                MappingUse::PassThrough
            } else {
                MappingUse::Convolution
            },
            account,
        )
    }

    pub(super) fn try_rebase(
        operation: &PreparedConvolution,
        source: NonEmptyFrameSpatialPlan,
        result: NonEmptyFrameSpatialPlan,
        account: &Account,
    ) -> Result<Self> {
        let domains = operation.domains();
        let domain = intersection(domains.source(), domains.filter());
        Self::try_from_parts(
            domain,
            domain,
            None,
            source,
            result,
            MappingUse::Rebase,
            account,
        )
    }

    fn try_from_parts(
        source_domain: Rect,
        result_domain: Rect,
        explicit_units: Option<[f64; 2]>,
        source: NonEmptyFrameSpatialPlan,
        result: NonEmptyFrameSpatialPlan,
        usage: MappingUse,
        account: &Account,
    ) -> Result<Self> {
        // Either zero axis makes the entire source image empty. Its other
        // coordinates and axis range are unused, so do not qualify them as a
        // physical lattice or send their magnitudes to the shader.
        let source_empty = source_domain.width() == 0.0 || source_domain.height() == 0.0;
        let source_domain = if source_empty {
            Rect::new(0.0, 0.0, 0.0, 0.0)
        } else {
            source_domain
        };
        let origin = source.texel_center_mapping.origin;
        let kernel_origin = [origin.x(), origin.y()];
        let scale = source.raster_scale.get();
        let unit_length = explicit_units.unwrap_or([1.0 / scale; 2]);
        let mut source_grid_origin = [0; 2];
        let mut source_grid_extent = [0; 2];
        let mut result_grid_origin = [0; 2];
        let mut result_grid_extent = [0; 2];
        for axis in 0..2 {
            if usage != MappingUse::Convolution {
                continue;
            }
            account.work(RenderWork::CoordinateVisits, 1)?;
            let h = unit_length[axis];
            if !h.is_finite() || h <= 0.0 || !(h as f32).is_normal() {
                return Err(mapping_error("kernel unit length", h));
            }
            let (smin, slen, pmin, plen) = if axis == 0 {
                (
                    source_domain.x(),
                    source_domain.width(),
                    result_domain.x(),
                    result_domain.width(),
                )
            } else {
                (
                    source_domain.y(),
                    source_domain.height(),
                    result_domain.y(),
                    result_domain.height(),
                )
            };
            let source_range = if source_empty {
                (0, 0)
            } else {
                grid_range(smin, slen, kernel_origin[axis], h, false)?
            };
            let result_range = grid_range(pmin, plen, kernel_origin[axis], h, true)?;
            source_grid_origin[axis] = source_range.0;
            source_grid_extent[axis] = source_range.1;
            result_grid_origin[axis] = result_range.0;
            result_grid_extent[axis] = result_range.1;
        }
        // These coordinates are consumed by f32 shader arithmetic. Reject a
        // collapsed/overflowing representation before allocating GPU resources.
        for value in [
            source_domain.x(),
            source_domain.y(),
            source_domain.max().x(),
            source_domain.max().y(),
            result_domain.x(),
            result_domain.y(),
            result_domain.max().x(),
            result_domain.max().y(),
            kernel_origin[0],
            kernel_origin[1],
            scale,
            result.raster_scale.get(),
        ] {
            account.work(RenderWork::CoordinateVisits, 1)?;
            if !(value as f32).is_finite() {
                return Err(mapping_error("coordinate", value));
            }
        }
        for spatial in [source, result] {
            let scale = spatial.raster_scale.get() as f32;
            if !scale.is_normal() || scale <= 0.0 {
                return Err(mapping_error("raster scale", f64::from(scale)));
            }
            for (origin, extent) in [
                (
                    spatial.texel_center_mapping.origin.x(),
                    spatial.device_extent.width,
                ),
                (
                    spatial.texel_center_mapping.origin.y(),
                    spatial.device_extent.height,
                ),
            ] {
                account.work(RenderWork::CoordinateVisits, 1)?;
                if extent > 8_388_608 {
                    return Err(mapping_error("kernel grid index", f64::from(extent)));
                }
                let origin = origin as f32;
                let first = origin + 0.5 / scale;
                let last = origin + (extent as f32 - 0.5) / scale;
                if !first.is_finite()
                    || !last.is_finite()
                    || (extent > 1
                        && (first == origin + 1.5 / scale
                            || last == origin + (extent as f32 - 1.5) / scale))
                {
                    return Err(mapping_error(
                        "capture coordinate precision",
                        f64::from(origin),
                    ));
                }
            }
        }
        if usage == MappingUse::Convolution {
            for axis in 0..2 {
                account.work(RenderWork::CoordinateVisits, 1)?;
                let origin = kernel_origin[axis] as f32;
                let step = unit_length[axis] as f32;
                for (start, extent) in [
                    (source_grid_origin[axis], source_grid_extent[axis]),
                    (result_grid_origin[axis], result_grid_extent[axis]),
                ] {
                    if extent == 0 {
                        continue;
                    }
                    let lo = start as f32;
                    let hi = lo + extent as f32 - 1.0;
                    let first = origin + (lo + 0.5) * step;
                    let last = origin + (hi + 0.5) * step;
                    if !first.is_finite()
                        || !last.is_finite()
                        || first == origin + (lo + 1.5) * step
                        || last == origin + (hi - 0.5) * step
                    {
                        return Err(mapping_error(
                            "kernel coordinate precision",
                            f64::from(origin),
                        ));
                    }
                }
            }
        }
        let result_origin = result.texel_center_mapping.origin;
        // Qualify the composed expressions reached by the shader, before its
        // finite-raster clamp. Finite kernel centers alone do not prove that
        // subtracting an origin, scaling, or narrowing their neighbors is safe.
        for axis in 0..2 {
            account.work(RenderWork::CoordinateVisits, 1)?;
            let capture_origin = kernel_origin[axis] as f32;
            let capture_scale = scale as f32;
            let result_scale = result.raster_scale.get() as f32;
            let result_axis_origin = if axis == 0 {
                result_origin.x()
            } else {
                result_origin.y()
            } as f32;
            let result_extent = if axis == 0 {
                result.device_extent.width
            } else {
                result.device_extent.height
            };
            let output_points = capture_centers(result_axis_origin, result_scale, result_extent);
            let source_extent = if axis == 0 {
                source.device_extent.width
            } else {
                source.device_extent.height
            };
            let capture_points = capture_centers(capture_origin, capture_scale, source_extent);
            let source_edges = if axis == 0 {
                [source_domain.x(), source_domain.max().x()]
            } else {
                [source_domain.y(), source_domain.max().y()]
            }
            .map(|v| v as f32);
            let result_edges = if axis == 0 {
                [result_domain.x(), result_domain.max().x()]
            } else {
                [result_domain.y(), result_domain.max().y()]
            }
            .map(|v| v as f32);
            if usage != MappingUse::Rebase {
                qualify_coverage(output_points, result_scale, result_edges)?;
                if !source_empty {
                    let mut bounds = [0_i32; 2];
                    for endpoint in 0..2 {
                        let delta = finite(source_edges[endpoint] - capture_origin)?;
                        let coordinate = finite(delta * capture_scale)?;
                        bounds[endpoint] = qualify_index(coordinate, endpoint == 1, false)?;
                    }
                    if bounds[1].checked_sub(bounds[0]).is_none()
                        || bounds[1].checked_sub(1).is_none()
                    {
                        return Err(mapping_error("kernel grid index", f64::from(bounds[1])));
                    }
                    qualify_coverage(capture_points, capture_scale, source_edges)?;
                }
            }
            match usage {
                MappingUse::Convolution => {
                    if !source_empty {
                        let start = source_grid_origin[axis] as f32;
                        let end = start + source_grid_extent[axis] as f32 - 1.0;
                        for index in [start, end] {
                            let offset = finite((index + 0.5) * unit_length[axis] as f32)?;
                            let point = finite(capture_origin + offset)?;
                            qualify_capture_sample(point, capture_origin, capture_scale, true)?;
                        }
                    }
                    for point in output_points {
                        let delta = finite(point - capture_origin)?;
                        let coordinate = finite(finite(delta / unit_length[axis] as f32)? - 0.5)?;
                        qualify_index(coordinate, false, true)?;
                    }
                }
                MappingUse::PassThrough | MappingUse::Rebase => {
                    for point in output_points {
                        qualify_capture_sample(
                            point,
                            capture_origin,
                            capture_scale,
                            usage == MappingUse::PassThrough,
                        )?;
                    }
                }
            }
        }
        Ok(Self {
            source_domain,
            result_domain,
            unit_length,
            kernel_origin,
            source_grid_origin,
            source_grid_extent,
            result_grid_origin,
            result_grid_extent,
            capture_origin: kernel_origin,
            capture_scale: scale,
            result_capture_origin: [result_origin.x(), result_origin.y()],
            result_capture_scale: result.raster_scale.get(),
        })
    }

    pub(super) fn validate_tap_offsets(
        self,
        operation: &PreparedConvolution,
        account: &Account,
    ) -> Result<()> {
        if operation.is_passthrough() {
            return Ok(());
        }
        for axis in 0..2 {
            account.work(RenderWork::CoordinateVisits, 1)?;
            let order = i64::from(operation.order()[axis]);
            let target = i64::from(operation.target()[axis]);
            let first = i64::from(self.result_grid_origin[axis]) - target;
            let last = first + i64::from(self.result_grid_extent[axis]) - 1 + order - 1;
            let source = i64::from(self.source_grid_origin[axis]);
            if order > i64::from(i32::MAX)
                || first < i64::from(i32::MIN)
                || last > i64::from(i32::MAX)
                || first - source < i64::from(i32::MIN)
                || last - source > i64::from(i32::MAX)
            {
                return Err(mapping_error("kernel grid index", last as f64));
            }
        }
        Ok(())
    }

    /// Integer allocation space only. Its unit descriptor deliberately does not
    /// claim to describe local logical geometry: the two-axis mapping above does.
    pub(super) fn allocation_spatial(
        self,
        context: FrameContext,
        source: bool,
    ) -> Result<NonEmptyFrameSpatialPlan> {
        let extent = if source {
            self.source_grid_extent
        } else {
            self.result_grid_extent
        };
        let rect = Rect::new(
            0.0,
            0.0,
            f64::from(extent[0].max(1)),
            f64::from(extent[1].max(1)),
        );
        let mut unit_context = context;
        unit_context.surface_scale = 1.0;
        require_spatial(unit_context, rect, Transform::identity())
    }
}

/// A transparent binding for an empty source has no semantic pixel coverage.
/// Allocate exactly one pixel rather than outward-rounding a reciprocal scale;
/// that rounding could fabricate a second dummy pixel. Anchor it to the result
/// lattice so translating the result cannot create unused source index work.
pub(super) fn empty_source_spatial(
    result: NonEmptyFrameSpatialPlan,
) -> Result<NonEmptyFrameSpatialPlan> {
    let origin = result.texel_center_mapping.origin;
    let step = 1.0 / result.raster_scale.get();
    let logical_bounds = match LogicalBounds::try_from_rect(
        Rect::new(origin.x(), origin.y(), step, step),
        "empty convolution source binding",
    )? {
        LogicalBounds::NonEmpty(bounds) => bounds,
        LogicalBounds::Empty(_) => return Err(mapping_error("empty source binding", step)),
    };
    Ok(NonEmptyFrameSpatialPlan {
        logical_bounds,
        device_origin: result.device_origin,
        device_extent: PositiveDeviceExtent {
            width: 1,
            height: 1,
        },
        raster_scale: result.raster_scale,
        texel_center_mapping: result.texel_center_mapping,
    })
}

pub(super) fn require_spatial(
    context: FrameContext,
    rect: Rect,
    transform: Transform,
) -> Result<NonEmptyFrameSpatialPlan> {
    let planned = context
        .plan_local_bounds(
            LogicalBounds::try_from_rect(rect, "convolution domain")?,
            transform,
        )
        .map_err(|error| {
            if error.code() == crate::ErrorCode::InvalidInput {
                mapping_error(
                    "capture spatial mapping",
                    rect.x()
                        .abs()
                        .max(rect.y().abs())
                        .max(rect.width())
                        .max(rect.height()),
                )
            } else {
                error
            }
        })?;
    match planned {
        FrameSpatialPlan::NonEmpty(spatial) => Ok(spatial),
        FrameSpatialPlan::Empty(_) => Err(Error::invalid_value(
            "convolution domain",
            "empty",
            "requires nonempty raster coverage",
        )),
    }
}

pub(super) fn source_domain_is_empty(operation: &PreparedConvolution) -> bool {
    let domains = operation.domains();
    let source = intersection(domains.source(), domains.filter());
    source.width() == 0.0 || source.height() == 0.0
}

pub(super) fn intersection(a: Rect, b: Rect) -> Rect {
    let x = a.x().max(b.x());
    let y = a.y().max(b.y());
    Rect::new(
        x,
        y,
        (a.max().x().min(b.max().x()) - x).max(0.0),
        (a.max().y().min(b.max().y()) - y).max(0.0),
    )
}

fn grid_range(min: f64, length: f64, origin: f64, step: f64, halo: bool) -> Result<(i32, u32)> {
    if length == 0.0 {
        return Ok((0, 0));
    }
    let lo = (min - origin) / step;
    let hi = (min + length - origin) / step;
    // Source footprints intersect [min,max). Result adds the neighbor required
    // by linear reconstruction at either extreme, independently of result clip.
    let start = if halo { (lo - 0.5).floor() } else { lo.floor() };
    let end = if halo {
        (hi - 0.5).ceil() + 1.0
    } else {
        hi.ceil()
    };
    const EXACT_INDEX_LIMIT: f64 = 8_388_608.0;
    if !start.is_finite()
        || !end.is_finite()
        || start < -EXACT_INDEX_LIMIT
        || end > EXACT_INDEX_LIMIT
        || end <= start
    {
        return Err(mapping_error("kernel grid index", start));
    }
    Ok((start as i32, (end - start) as u32))
}

fn capture_centers(origin: f32, scale: f32, extent: u32) -> [f32; 2] {
    [origin + 0.5 / scale, origin + (extent as f32 - 0.5) / scale]
}

fn finite(value: f32) -> Result<f32> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(mapping_error("composed coordinate", f64::from(value)))
    }
}

fn qualify_index(coordinate: f32, ceil: bool, neighbor: bool) -> Result<i32> {
    let rounded = if ceil {
        coordinate.ceil()
    } else {
        coordinate.floor()
    };
    // Compare in f64: i32::MAX rounds upward to 2^31 when converted to f32.
    let value = f64::from(rounded);
    let maximum = i32::MAX - i32::from(neighbor);
    if !value.is_finite() || value < f64::from(i32::MIN) || value > f64::from(maximum) {
        return Err(mapping_error("kernel grid index", value));
    }
    Ok(rounded as i32)
}

fn qualify_capture_sample(point: f32, origin: f32, scale: f32, bilinear: bool) -> Result<()> {
    let delta = finite(point - origin)?;
    let scaled = finite(delta * scale)?;
    let coordinate = if bilinear {
        finite(scaled - 0.5)?
    } else {
        scaled
    };
    qualify_index(coordinate, false, bilinear)?;
    Ok(())
}

fn qualify_coverage(points: [f32; 2], scale: f32, domain: [f32; 2]) -> Result<()> {
    let step = finite(1.0 / scale)?;
    if !step.is_normal() || step <= 0.0 {
        return Err(mapping_error("coverage step", f64::from(step)));
    }
    for point in points {
        let low = finite(point - step * 0.5)?;
        let high = finite(point + step * 0.5)?;
        finite(high.min(domain[1]) - low.max(domain[0]))?;
    }
    // Between the endpoints each clipped difference is bounded above by the
    // finite footprint width and below by its smaller endpoint value.
    Ok(())
}

fn mapping_error(name: &'static str, value: f64) -> Error {
    let phase = if name == "kernel grid index" {
        crate::ConvolutionNumericPhase::GridIndex
    } else {
        crate::ConvolutionNumericPhase::GridMapping
    };
    crate::ConvolutionPreparationError::Numeric { phase, value }.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Size, renderer::Antialiasing};

    fn context(scale: f64) -> FrameContext {
        FrameContext::try_new(
            Size::new(64.0, 64.0),
            scale,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap()
    }

    #[test]
    fn anisotropic_grid_uses_capture_origin_and_independent_axis_steps() {
        let account = Account::for_test();
        let context = context(2.0);
        let source_domain = Rect::new(1.25, -2.5, 6.0, 8.0);
        let result_domain = Rect::new(2.0, -1.0, 2.0, 1.0);
        let source = require_spatial(context, source_domain, Transform::identity()).unwrap();
        let result = require_spatial(context, result_domain, Transform::identity()).unwrap();
        let mapping = ConvolutionMapping::try_from_parts(
            source_domain,
            result_domain,
            Some([2.0, 4.0]),
            source,
            result,
            MappingUse::Convolution,
            &account,
        )
        .unwrap();
        // Outward capture at density 2 begins at (1,-2.5), and the explicit
        // source footprints are [1+2i,3+2i) × [-2.5+4j,1.5+4j).
        assert_eq!(mapping.kernel_origin(), [1.0, -2.5]);
        assert_eq!(mapping.source_grid_origin(), [0, 0]);
        assert_eq!(mapping.source_grid_extent(), [4, 2]);
        assert_eq!(mapping.result_grid_origin(), [0, -1]);
        assert_eq!(mapping.result_grid_extent(), [2, 3]);
        // Kernel index (1,0) is local (4,-.5), capture texel boundary (6,4).
        let local = [
            mapping.kernel_origin()[0] + 1.5 * mapping.unit_length()[0],
            mapping.kernel_origin()[1] + 0.5 * mapping.unit_length()[1],
        ];
        assert_eq!(local, [4.0, -0.5]);
        assert_eq!(
            [
                (local[0] - mapping.capture_origin()[0]) * mapping.capture_scale(),
                (local[1] - mapping.capture_origin()[1]) * mapping.capture_scale()
            ],
            [6.0, 4.0]
        );
    }

    #[test]
    fn full_affine_only_changes_capture_density_not_explicit_axis_spacing() {
        let account = Account::for_test();
        // Nonuniform rotated/sheared mapping: (x,y) -> (2x-y+7, x+3y-4).
        let affine = Transform::try_new([2.0, 1.0, -1.0, 3.0, 7.0, -4.0]).unwrap();
        let bounds = Rect::new(0.0, 0.0, 4.0, 4.0);
        let spatial = require_spatial(context(1.5), bounds, affine).unwrap();
        let mapping = ConvolutionMapping::try_from_parts(
            bounds,
            bounds,
            Some([2.0, 0.5]),
            spatial,
            spatial,
            MappingUse::Convolution,
            &account,
        )
        .unwrap();
        assert_eq!(mapping.unit_length(), [2.0, 0.5]);
        let local = [
            mapping.kernel_origin()[0] + 1.5 * 2.0,
            mapping.kernel_origin()[1] + 2.5 * 0.5,
        ];
        assert_eq!(local, [3.0, 1.25]);
        // A remains an outer composition map; the independent equation gives
        // surface point (11.75,2.75), physical point (17.625,4.125).
        let surface = [
            2.0 * local[0] - local[1] + 7.0,
            local[0] + 3.0 * local[1] - 4.0,
        ];
        assert_eq!(surface, [11.75, 2.75]);
        assert_eq!([surface[0] * 1.5, surface[1] * 1.5], [17.625, 4.125]);
        assert!(mapping.capture_scale() > 1.5);
    }

    #[test]
    fn source_footprints_include_fractional_edges_and_result_has_reconstruction_halo() {
        assert_eq!(grid_range(0.25, 1.5, 0.0, 1.0, false).unwrap(), (0, 2));
        assert_eq!(grid_range(-1.75, 1.5, 0.0, 1.0, false).unwrap(), (-2, 2));
        assert_eq!(grid_range(0.0, 2.0, 0.0, 2.0, false).unwrap(), (0, 1));
        assert_eq!(grid_range(0.0, 2.0, 0.0, 2.0, true).unwrap(), (-1, 3));
        assert_eq!(grid_range(50.0, 0.0, 0.0, 1.0, false).unwrap(), (0, 0));
    }

    #[test]
    fn unrepresentable_grid_indices_and_collapsed_coordinates_are_typed_refusals() {
        let error = grid_range(0.0, 10.0, 0.0, 1.0e-8, false).unwrap_err();
        assert!(matches!(
            error.convolution_diagnostic(),
            Some(crate::ConvolutionPreparationError::Numeric {
                phase: crate::ConvolutionNumericPhase::GridIndex,
                ..
            })
        ));
        let account = Account::for_test();
        let bounds = Rect::new(1_000_000_000.0, 0.0, 4.0, 4.0);
        let spatial = require_spatial(context(1.0), bounds, Transform::identity()).unwrap();
        let error = ConvolutionMapping::try_from_parts(
            bounds,
            bounds,
            None,
            spatial,
            spatial,
            MappingUse::Convolution,
            &account,
        )
        .unwrap_err();
        assert!(matches!(
            error.convolution_diagnostic(),
            Some(crate::ConvolutionPreparationError::Numeric {
                phase: crate::ConvolutionNumericPhase::GridMapping,
                ..
            })
        ));
    }

    #[test]
    fn composed_capture_mapping_qualifies_float_and_signed_neighbor_boundaries() {
        let account = Account::for_test();
        let bounds = Rect::new(0.0, 0.0, 2.0, 1.0);
        let spatial = require_spatial(context(16.0), bounds, Transform::identity()).unwrap();
        // For a single input cell, its x center is h/2 and the capture
        // coordinate is 8h-.5. The predecessor of 2^28 is 268435440;
        // multiplication by8 gives the last f32 integer below 2^31.
        let valid = ConvolutionMapping::try_from_parts(
            bounds,
            bounds,
            Some([268_435_440.0, 1.0]),
            spatial,
            spatial,
            MappingUse::Convolution,
            &account,
        )
        .unwrap();
        assert_eq!(valid.source_grid_extent(), [1, 1]);
        for (unit, phase) in [
            (268_435_456.0, crate::ConvolutionNumericPhase::GridIndex),
            (1.0e10, crate::ConvolutionNumericPhase::GridIndex),
            (1.0e38, crate::ConvolutionNumericPhase::GridMapping),
            (2.0e38, crate::ConvolutionNumericPhase::GridMapping),
        ] {
            let error = ConvolutionMapping::try_from_parts(
                bounds,
                bounds,
                Some([unit, 1.0]),
                spatial,
                spatial,
                MappingUse::Convolution,
                &account,
            )
            .unwrap_err();
            assert!(
                matches!(error.convolution_diagnostic(),
                Some(crate::ConvolutionPreparationError::Numeric { phase: actual, .. }) if *actual == phase),
                "unit{unit}: {error:?}"
            );
        }
        // Integer conversion itself and its +1 bilinear neighbor use signed
        // bounds, including the negative boundary, independently of grid_size.
        assert_eq!(
            qualify_index(-2_147_483_648.0, false, true).unwrap(),
            i32::MIN
        );
        assert_eq!(
            qualify_index(2_147_483_520.0, false, true).unwrap(),
            2_147_483_520
        );
        assert!(qualify_index(2_147_483_648.0, false, true).is_err());
        assert!(
            qualify_index(
                f32::from_bits((-2_147_483_648.0_f32).to_bits() + 1),
                false,
                true
            )
            .is_err()
        );
    }

    #[test]
    fn empty_input_pass_through_and_rebase_qualify_only_reached_mapping() {
        let account = Account::for_test();
        let domain = Rect::new(1.0, 0.0, 2.0, 1.0);
        let context = context(16.0);
        let spatial = require_spatial(context, domain, Transform::identity()).unwrap();
        let dummy = empty_source_spatial(spatial).unwrap();
        let empty = ConvolutionMapping::try_from_parts(
            Rect::new(1.0, 0.0, 0.0, 1.0e12),
            domain,
            Some([2.0e38; 2]),
            dummy,
            spatial,
            MappingUse::Convolution,
            &account,
        )
        .unwrap();
        assert_eq!(empty.source_grid_extent(), [0; 2]);
        assert_eq!(dummy.device_extent.width, 1);
        assert_eq!(dummy.device_extent.height, 1);
        let mut accounting = crate::RenderAccountingContext::default();
        let operation = |kernel: &[f64], accounting: &mut crate::RenderAccountingContext| {
            crate::prepare_convolution(
                crate::ConvolutionInput::new(kernel)
                    .with_order(1.0, None)
                    .with_unit_length(2.0e38, None),
                crate::ConvolutionDomains::try_new(domain, domain, domain).unwrap(),
                crate::FilterColorSpace::Srgb,
                accounting,
            )
            .unwrap()
        };
        let passthrough = operation(&[], &mut accounting);
        assert!(passthrough.is_passthrough());
        assert_eq!(
            ConvolutionMapping::try_new(&passthrough, spatial, spatial, &account)
                .unwrap()
                .unit_length(),
            [1.0 / 16.0; 2]
        );
        let union = require_spatial(
            context,
            Rect::new(0.0, 0.0, 6.0, 1.0),
            Transform::identity(),
        )
        .unwrap();
        let executable = operation(&[1.0], &mut accounting);
        let rebase = ConvolutionMapping::try_rebase(&executable, union, spatial, &account).unwrap();
        assert_eq!(rebase.result_capture_origin(), [1.0, 0.0]);
        assert_eq!(rebase.unit_length(), [1.0 / 16.0; 2]);
        // Pass-through constructs a bilinear coordinate even for empty D;
        // rebasing constructs a raw coordinate without a bilinear neighbor.
        assert!(
            ConvolutionMapping::try_from_parts(
                Rect::new(0.0, 0.0, 0.0, 0.0),
                domain,
                None,
                dummy,
                spatial,
                MappingUse::PassThrough,
                &account
            )
            .is_ok()
        );
        assert!(qualify_capture_sample(1.0e10, 0.0, 16.0, false).is_err());
        assert!(qualify_capture_sample(1.0e38, 0.0, 16.0, false).is_err());
        assert!(qualify_capture_sample(-2.0e38, 2.0e38, 1.0, true).is_err());
        assert!(qualify_coverage([-2.0e38, 0.0], 1.0, [2.0e38, 2.0e38]).is_err());
    }
    fn planned(
        operation: crate::PreparedConvolution,
        draw: bool,
    ) -> crate::accounting::OwnedBox<super::super::GpuRenderGraph> {
        let account = Account::for_test();
        let mut scene = crate::Scene::new();
        scene.layer(crate::Layer::new().with_convolution(operation), |scene| {
            if draw {
                scene.fill(Rect::new(0.0, 0.0, 8.0, 8.0), Color::BLACK);
            }
        });
        let commands = scene
            .normalize(crate::Capabilities::CURRENT, &account)
            .unwrap();
        let super::super::FramePlan::GpuGraph(graph) =
            super::super::FramePlan::try_from_commands(commands, context(1.0), &account).unwrap()
        else {
            panic!("convolution requires the concrete graph receiver");
        };
        graph
    }

    fn prepare(kernel: &[f64], source: Rect, result: Rect) -> crate::PreparedConvolution {
        let mut accounting =
            crate::RenderAccountingContext::new(crate::RenderAccountingOptions::new());
        crate::prepare_convolution(
            crate::ConvolutionInput::new(kernel)
                .with_order(kernel.len() as f64, Some(1.0))
                .with_divisor(1.0)
                .with_bias(0.25),
            crate::ConvolutionDomains::try_new(source, Rect::new(0.0, 0.0, 16.0, 16.0), result)
                .unwrap(),
            crate::FilterColorSpace::Srgb,
            &mut accounting,
        )
        .unwrap()
    }

    #[test]
    fn bias_retains_empty_children_and_empty_input_through_complete_graph_lowering() {
        let source = Rect::new(2.0, 2.0, 0.0, 0.0);
        let result = Rect::new(1.0, 1.0, 3.0, 3.0);
        let graph = planned(prepare(&[1.0], source, result), false);
        let first = graph.convolutions.first().unwrap();
        assert_eq!(first.mapping.source_grid_origin(), [0; 2]);
        assert_eq!(first.mapping.source_grid_extent(), [0; 2]);
        let source_id = graph.passes[first.pass.index.as_usize().unwrap()].reads[0];
        let descriptor = graph.resources[source_id.index.as_usize().unwrap()].descriptor;
        assert_eq!(descriptor.device_extent.width, 1);
        assert_eq!(descriptor.device_extent.height, 1);
        assert_eq!(
            descriptor.logical_bounds.rect(),
            Rect::new(1.0, 1.0, 1.0, 1.0)
        );
        let view = graph.lowering_view(&Account::for_test()).unwrap();
        let passes = view.passes(&Account::for_test()).unwrap();
        let mut resolved = false;
        for pass in &passes {
            if let super::super::GraphLoweringPassKind::Convolution(operation) =
                pass.kind(&Account::for_test()).unwrap()
            {
                assert_eq!(
                    operation.mapping().source_domain(),
                    Rect::new(0.0, 0.0, 0.0, 0.0)
                );
                assert_eq!(operation.mapping().result_domain(), result);
                if operation.stage() == ConvolutionStage::Resolve {
                    resolved = true;
                }
            }
        }
        assert!(resolved);
    }

    #[test]
    fn zero_source_axis_skips_unused_magnitudes_and_uses_one_transparent_pixel() {
        let filter = Rect::new(0.0, 0.0, 1.0e201, 1.0e201);
        let result = Rect::new(2.25, 3.25, 2.0, 1.0);
        for source in [
            Rect::new(1.0e200, 0.0, 0.0, 1.0e200),
            Rect::new(0.0, 1.0e200, 1.0e200, 0.0),
        ] {
            let mut accounting = crate::RenderAccountingContext::default();
            let operation = crate::prepare_convolution(
                crate::ConvolutionInput::new(&[1.0])
                    .with_order(1.0, None)
                    .with_bias(0.5),
                crate::ConvolutionDomains::try_new(source, filter, result).unwrap(),
                crate::FilterColorSpace::Srgb,
                &mut accounting,
            )
            .unwrap();
            assert_eq!(operation.domains().source(), source);
            let graph = planned(operation, true);
            assert!(graph.vello_spans.is_empty());
            let first = graph.convolutions.first().unwrap();
            assert_eq!(first.mapping.source_domain(), Rect::new(0.0, 0.0, 0.0, 0.0));
            assert_eq!(first.mapping.source_grid_origin(), [0; 2]);
            assert_eq!(first.mapping.source_grid_extent(), [0; 2]);
            assert_eq!(first.mapping.capture_origin(), [2.0, 3.0]);
            assert_eq!(first.mapping.kernel_origin(), [2.0, 3.0]);
            let source_id = graph.passes[first.pass.index.as_usize().unwrap()].reads[0];
            let descriptor = graph.resources[source_id.index.as_usize().unwrap()].descriptor;
            assert_eq!(descriptor.device_extent.width, 1);
            assert_eq!(descriptor.device_extent.height, 1);
            assert_eq!(
                descriptor.logical_bounds.rect(),
                Rect::new(2.0, 3.0, 1.0, 1.0)
            );
            graph.lowering_view(&Account::for_test()).unwrap();
        }
    }

    #[test]
    fn backdrop_rebase_restores_the_original_source_lattice_before_explicit_units() {
        let account = Account::for_test();
        let union = Rect::new(0.0, 0.0, 6.0, 1.0);
        let domain = Rect::new(1.0, 0.0, 2.0, 1.0);
        let mut accounting = crate::RenderAccountingContext::default();
        let operation = crate::prepare_convolution(
            crate::ConvolutionInput::new(&[1.0])
                .with_order(1.0, None)
                .with_unit_length(2.0, Some(1.0)),
            crate::ConvolutionDomains::try_new(domain, union, domain).unwrap(),
            crate::FilterColorSpace::Srgb,
            &mut accounting,
        )
        .unwrap();
        let union_spatial = require_spatial(context(1.0), union, Transform::identity()).unwrap();
        let domain_spatial = require_spatial(context(1.0), domain, Transform::identity()).unwrap();
        let rebase =
            ConvolutionMapping::try_rebase(&operation, union_spatial, domain_spatial, &account)
                .unwrap();
        assert_eq!(rebase.source_domain(), domain);
        assert_eq!(rebase.result_domain(), domain);
        assert_eq!(rebase.capture_origin(), [0.0, 0.0]);
        assert_eq!(rebase.result_capture_origin(), [1.0, 0.0]);
        assert_eq!(rebase.unit_length(), [1.0, 1.0]);
        assert_eq!(rebase.source_grid_origin(), [0; 2]);
        assert_eq!(rebase.source_grid_extent(), [0; 2]);
        assert_eq!(rebase.result_grid_origin(), [0; 2]);
        assert_eq!(rebase.result_grid_extent(), [0; 2]);
        let prepared =
            ConvolutionMapping::try_new(&operation, domain_spatial, domain_spatial, &account)
                .unwrap();
        // The required D lattice starts at x=1. Explicit dx=2 therefore
        // creates one input cell centered at x=2, independent of the union.
        assert_eq!(prepared.kernel_origin(), [1.0, 0.0]);
        assert_eq!(prepared.unit_length(), [2.0, 1.0]);
        assert_eq!(prepared.source_grid_extent(), [1, 1]);
    }

    #[test]
    fn smaller_primitive_result_does_not_shrink_the_neighborhood_source_domain() {
        let source = Rect::new(0.0, 0.0, 8.0, 8.0);
        let result = Rect::new(3.0, 3.0, 1.0, 1.0);
        let graph = planned(prepare(&[1.0, 0.0, -1.0], source, result), true);
        let mapping = graph.convolutions.first().unwrap().mapping;
        assert_eq!(mapping.source_domain(), source);
        assert_eq!(mapping.source_grid_extent(), [8, 8]);
        assert_eq!(mapping.result_domain(), result);
        graph.lowering_view(&Account::for_test()).unwrap();
    }

    #[test]
    fn skipped_or_reordered_tap_batch_is_rejected_before_lowering() {
        let bounds = Rect::new(0.0, 0.0, 4.0, 4.0);
        let mut graph = planned(prepare(&[1.0; 33], bounds, bounds), true);
        // First 32 taps must be followed by tap 32. Repeating tap 31 would
        // silently corrupt signed arithmetic even though all edges remain valid.
        let stage = &mut graph.convolutions[2].stage;
        *stage = ConvolutionStage::Accumulate {
            first_tap: 31,
            tap_count: 1,
            first: false,
        };
        assert!(graph.lowering_view(&Account::for_test()).is_err());
    }
}
