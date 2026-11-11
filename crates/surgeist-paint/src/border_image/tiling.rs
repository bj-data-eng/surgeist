use super::{
    BorderImageGeometry, BorderImageRect, BorderImageRegion, ExactProduct, GeometryWork,
    compare_product_sums, geometry_error, run,
};
use crate::{
    PaintError, PaintOptions, PaintPhase, PaintReport, WorkKind,
    accounting::{OperationKind, charge},
};
use std::{cmp::Ordering, num::NonZeroU64};
use surgeist_layout::Size;

/// The resolved repeat choice for one physical axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BorderImageRepeat {
    Stretch,
    Repeat,
    Round,
    Space,
}

/// A tile pattern in CSS pixels, relative to its region's clip minimum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderImageAxisPattern {
    extent: f64,
    anchor: f64,
    gap: f64,
    count: Option<NonZeroU64>,
}

impl BorderImageAxisPattern {
    /// The nominal extent of one tile before clipping.
    ///
    /// A finite pattern with zero gap evenly subdivides the clip. Use the
    /// clip endpoints for the first and final boundaries of that pattern.
    pub const fn extent(self) -> f64 {
        self.extent
    }

    /// The origin of tile zero, relative to the clip minimum.
    ///
    /// For a periodic repeat this is a clip-local representative of the
    /// centered pattern. Tiles extend in both directions from this anchor.
    pub const fn anchor(self) -> f64 {
        self.anchor
    }

    /// The distance between successive tile edges. Space also uses this gap
    /// between the clip boundaries and the nearest tile on each side.
    pub const fn gap(self) -> f64 {
        self.gap
    }

    /// The number of whole tiles in a finite pattern starting at the anchor.
    /// A periodic repeat returns `None` and is limited by the region's clip.
    pub const fn count(self) -> Option<NonZeroU64> {
        self.count
    }

    fn stretch(extent: f64) -> Self {
        Self {
            extent,
            anchor: 0.0,
            gap: 0.0,
            count: Some(NonZeroU64::MIN),
        }
    }
}

/// One drawable source region and its clipped destination pattern.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderImageTileRegion {
    source: BorderImageRect,
    clip: BorderImageRect,
    axes: Size<BorderImageAxisPattern>,
}
impl BorderImageTileRegion {
    /// The slice to sample, in source coordinate units.
    pub const fn source(self) -> BorderImageRect {
        self.source
    }

    /// The destination clip in CSS pixels.
    pub const fn clip(self) -> BorderImageRect {
        self.clip
    }

    /// Horizontal and vertical patterns, each local to the clip.
    pub const fn axes(self) -> Size<BorderImageAxisPattern> {
        self.axes
    }
}

/// A fixed-size tile plan in physical row order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderImageTiling {
    regions: [[Option<BorderImageTileRegion>; 3]; 3],
}
impl BorderImageTiling {
    /// Top, middle, and bottom rows, with absent entries for regions that emit no tiles.
    pub const fn regions(&self) -> &[[Option<BorderImageTileRegion>; 3]; 3] {
        &self.regions
    }
}

/// One complete plan or a typed refusal, with its Ledger report.
#[derive(Debug)]
pub struct BorderImageTilingOutcome {
    pub result: Result<BorderImageTiling, PaintError>,
    pub report: Option<PaintReport>,
}

/// Plans tiling for an admitted partition. Width and height select the horizontal
/// and vertical repeat modes. Each pattern is clipped to its destination region.
pub fn resolve_border_image_tiling(
    geometry: &BorderImageGeometry,
    repeat: Size<BorderImageRepeat>,
    options: PaintOptions,
) -> BorderImageTilingOutcome {
    let (result, report) = run(
        options,
        OperationKind::BorderImageTiling,
        PaintPhase::BorderImageTiling,
        None::<fn(&mut GeometryWork<'_, '_, '_>) -> Result<(), PaintError>>,
        |work| resolve(work, geometry, repeat),
    );
    BorderImageTilingOutcome { result, report }
}

fn resolve(
    work: &mut GeometryWork<'_, '_, '_>,
    geometry: &BorderImageGeometry,
    repeat: Size<BorderImageRepeat>,
) -> Result<BorderImageTiling, PaintError> {
    use Direction::{Horizontal, Vertical};
    let [
        [top_left, top, top_right],
        [left, middle, right],
        [bottom_left, bottom, bottom_right],
    ] = *geometry.regions();
    let top_left = corner(work, top_left)?;
    let (top, top_basis) = edge(work, top, Horizontal, repeat.width)?;
    let top_right = corner(work, top_right)?;
    let (left, left_basis) = edge(work, left, Vertical, repeat.height)?;
    let (right, right_basis) = edge(work, right, Vertical, repeat.height)?;
    let bottom_left = corner(work, bottom_left)?;
    let (bottom, bottom_basis) = edge(work, bottom, Horizontal, repeat.width)?;
    let bottom_right = corner(work, bottom_right)?;
    let middle = center(
        work,
        middle,
        repeat,
        Size::new(top_basis.or(bottom_basis), left_basis.or(right_basis)),
    )?;
    Ok(BorderImageTiling {
        regions: [
            [top_left, top, top_right],
            [left, middle, right],
            [bottom_left, bottom, bottom_right],
        ],
    })
}

fn corner(
    work: &mut GeometryWork<'_, '_, '_>,
    region: BorderImageRegion,
) -> Result<Option<BorderImageTileRegion>, PaintError> {
    let Some((source, clip)) = pair(work, region)? else {
        return Ok(None);
    };
    let extent = size(work, clip)?;
    emit(
        work,
        source,
        clip,
        extent.map(BorderImageAxisPattern::stretch),
    )
}

#[derive(Clone, Copy)]
enum Direction {
    Horizontal,
    Vertical,
}
impl Direction {
    fn split<T>(self, size: Size<T>) -> (T, T) {
        match self {
            Self::Horizontal => (size.width, size.height),
            Self::Vertical => (size.height, size.width),
        }
    }

    fn axes<T>(self, along: T, across: T) -> Size<T> {
        match self {
            Self::Horizontal => Size::new(along, across),
            Self::Vertical => Size::new(across, along),
        }
    }
}

fn edge(
    work: &mut GeometryWork<'_, '_, '_>,
    region: BorderImageRegion,
    direction: Direction,
    repeat: BorderImageRepeat,
) -> Result<(Option<BorderImageTileRegion>, Option<f64>), PaintError> {
    let Some((source, clip)) = pair(work, region)? else {
        return Ok((None, None));
    };
    let (along, across) = direction.split(size(work, clip)?);
    let basis = if repeat == BorderImageRepeat::Stretch {
        None
    } else {
        let (span, thickness) = direction.split(size(work, source)?);
        Some(proportional(work, span, across, thickness)?)
    };
    let Some(pattern) = axis_pattern(work, repeat, along, basis.unwrap_or(along))? else {
        // Space can discard this edge while its proportional scale is still
        // the preferred basis for the middle on the same axis.
        return Ok((None, basis));
    };
    let axes = direction.axes(pattern, BorderImageAxisPattern::stretch(across));
    Ok((emit(work, source, clip, axes)?, basis))
}

fn center(
    work: &mut GeometryWork<'_, '_, '_>,
    region: BorderImageRegion,
    repeat: Size<BorderImageRepeat>,
    basis: Size<Option<f64>>,
) -> Result<Option<BorderImageTileRegion>, PaintError> {
    let Some((source, clip)) = pair(work, region)? else {
        return Ok(None);
    };
    let extent = size(work, clip)?;
    let Some(x) = center_axis(
        work,
        repeat.width,
        extent.width,
        basis.width,
        source.min.x,
        source.max.x,
    )?
    else {
        return Ok(None);
    };
    let Some(y) = center_axis(
        work,
        repeat.height,
        extent.height,
        basis.height,
        source.min.y,
        source.max.y,
    )?
    else {
        return Ok(None);
    };
    emit(work, source, clip, Size::new(x, y))
}

fn center_axis(
    work: &mut GeometryWork<'_, '_, '_>,
    repeat: BorderImageRepeat,
    extent: f64,
    basis: Option<f64>,
    source_min: f64,
    source_max: f64,
) -> Result<Option<BorderImageAxisPattern>, PaintError> {
    if repeat == BorderImageRepeat::Stretch {
        return Ok(Some(BorderImageAxisPattern::stretch(extent)));
    }
    let tile = match basis {
        Some(basis) => basis,
        None => work.coordinate(|| source_max - source_min)?,
    };
    axis_pattern(work, repeat, extent, tile)
}

fn pair(
    work: &mut GeometryWork<'_, '_, '_>,
    region: BorderImageRegion,
) -> Result<Option<(BorderImageRect, BorderImageRect)>, PaintError> {
    work.coordinate(|| region.source().zip(region.destination()))
}

fn size(
    work: &mut GeometryWork<'_, '_, '_>,
    rect: BorderImageRect,
) -> Result<Size<f64>, PaintError> {
    Ok(Size::new(
        work.coordinate(|| rect.max.x - rect.min.x)?,
        work.coordinate(|| rect.max.y - rect.min.y)?,
    ))
}

fn emit(
    work: &mut GeometryWork<'_, '_, '_>,
    source: BorderImageRect,
    clip: BorderImageRect,
    axes: Size<BorderImageAxisPattern>,
) -> Result<Option<BorderImageTileRegion>, PaintError> {
    charge(work.phase, work.scope, WorkKind::OutputRecord, 1)?;
    Ok(Some(BorderImageTileRegion { source, clip, axes }))
}

fn proportional(
    work: &mut GeometryWork<'_, '_, '_>,
    span: f64,
    width: f64,
    thickness: f64,
) -> Result<f64, PaintError> {
    // Prefer the cross-axis scale. Change association if that intermediate
    // would overflow or lose precision in the subnormal range.
    let scale = work.coordinate(|| width / thickness)?;
    let value = if work.coordinate(|| scale.is_normal())? {
        work.coordinate(|| span * scale)?
    } else {
        let ratio = work.coordinate(|| span / thickness)?;
        if work.coordinate(|| ratio.is_normal())? {
            work.coordinate(|| width * ratio)?
        } else {
            let numerator = work.coordinate(|| span * width)?;
            work.coordinate(|| numerator / thickness)?
        }
    };
    positive(work, value)
}

fn positive(work: &mut GeometryWork<'_, '_, '_>, value: f64) -> Result<f64, PaintError> {
    work.finite(value)?;
    work.coordinate(|| value > 0.0)?
        .then_some(value)
        .ok_or_else(geometry_error)
}

fn axis_pattern(
    work: &mut GeometryWork<'_, '_, '_>,
    repeat: BorderImageRepeat,
    extent: f64,
    tile: f64,
) -> Result<Option<BorderImageAxisPattern>, PaintError> {
    match repeat {
        BorderImageRepeat::Stretch => Ok(Some(BorderImageAxisPattern::stretch(extent))),
        BorderImageRepeat::Repeat => {
            // Remove whole periods before centering. Subtracting a small
            // tile from the original clip extent could erase half its phase.
            let double_tile = work.coordinate(|| tile * 2.0)?;
            let reduced_extent = if work.coordinate(|| double_tile.is_finite())? {
                work.coordinate(|| extent % double_tile)?
            } else {
                // The finite clip is already shorter than two tiles.
                extent
            };
            let delta = work.coordinate(|| reduced_extent - tile)?;
            let anchor = work.coordinate(|| delta * 0.5)?;
            let restored = work.coordinate(|| anchor * 2.0)?;
            if work.coordinate(|| restored != delta)? {
                return Err(geometry_error());
            }
            Ok(Some(BorderImageAxisPattern {
                extent: tile,
                anchor,
                gap: 0.0,
                count: None,
            }))
        }
        BorderImageRepeat::Round => {
            let (whole, remainder) = whole_tiles(work, extent, tile)?;
            let count = round_count(work, extent, tile, whole, remainder)?;
            let length = work.coordinate(|| extent / count)?;
            Ok(Some(BorderImageAxisPattern {
                extent: positive(work, length)?,
                anchor: 0.0,
                gap: 0.0,
                count: Some(tile_count(work, count)?),
            }))
        }
        BorderImageRepeat::Space => {
            let (whole, remainder) = whole_tiles(work, extent, tile)?;
            if work.coordinate(|| whole == 0.0)? {
                return Ok(None);
            }
            let gaps = work.coordinate(|| whole + 1.0)?;
            let gap = work.coordinate(|| remainder / gaps)?;
            if work.coordinate(|| gap == 0.0)? && work.coordinate(|| remainder > 0.0)? {
                return Err(geometry_error());
            }
            if work.coordinate(|| gap > 0.0)? {
                // With subnormal spacing, the gap's rounding quantum can
                // change its size substantially. Require equal outer gaps in
                // that range. For normal spacing, prove the last tile fits
                // with a trailing gap despite nominal rounding.
                let exact_gap = work.coordinate(|| gap.is_subnormal())?;
                let used_gaps = if exact_gap { gaps } else { whole };
                let tiles = ExactProduct::new(work, whole, tile)?;
                let gaps = ExactProduct::new(work, used_gaps, gap)?;
                let clip = ExactProduct::new(work, extent, 1.0)?;
                let expected = if exact_gap {
                    Ordering::Equal
                } else {
                    Ordering::Less
                };
                if compare_product_sums(work, [tiles, gaps], [clip, ExactProduct::ZERO])?
                    != expected
                {
                    return Err(geometry_error());
                }
            }
            Ok(Some(BorderImageAxisPattern {
                extent: tile,
                anchor: gap,
                gap,
                count: Some(tile_count(work, whole)?),
            }))
        }
    }
}

// Integer counts below this limit have individually distinguishable adjacent
// indices and an exactly representable count + 1 for Space.
const TILE_COUNT_LIMIT: f64 = 9_007_199_254_740_992.0;

fn whole_tiles(
    work: &mut GeometryWork<'_, '_, '_>,
    extent: f64,
    tile: f64,
) -> Result<(f64, f64), PaintError> {
    let ratio = work.coordinate(|| extent / tile)?;
    if work.coordinate(|| ratio > TILE_COUNT_LIMIT)? {
        return Err(geometry_error());
    }
    let mut whole = work.coordinate(|| ratio.floor())?;
    let mut remainder = work.coordinate(|| (-whole).mul_add(tile, extent))?;
    // Division can round a fraction just below an integer up to that integer.
    // A fused product retains the sign of the exact remainder.
    if work.coordinate(|| remainder.is_sign_negative())? {
        whole = work.coordinate(|| whole - 1.0)?;
        remainder = work.coordinate(|| (-whole).mul_add(tile, extent))?;
    }
    if work.coordinate(|| whole >= TILE_COUNT_LIMIT)? {
        return Err(geometry_error());
    }
    Ok((whole, remainder))
}

fn round_count(
    work: &mut GeometryWork<'_, '_, '_>,
    extent: f64,
    tile: f64,
    whole: f64,
    remainder: f64,
) -> Result<f64, PaintError> {
    let half = work.coordinate(|| tile * 0.5)?;
    let order = work
        .coordinate(|| remainder.partial_cmp(&half))?
        .ok_or_else(geometry_error)?;
    let order = if order == Ordering::Equal {
        // Determine the exact side of the half-tile boundary when rounded
        // remainders compare equal. Compare 2*extent with 2*whole*tile + tile.
        let doubled = work.coordinate(|| whole * 2.0)?;
        let extent = ExactProduct::new(work, extent, 2.0)?;
        let covered = ExactProduct::new(work, tile, doubled)?;
        let tile = ExactProduct::new(work, tile, 1.0)?;
        compare_product_sums(work, [extent, ExactProduct::ZERO], [covered, tile])?
    } else {
        order
    };
    let nearest = if order == Ordering::Less {
        whole
    } else {
        work.coordinate(|| whole + 1.0)?
    };
    work.coordinate(|| nearest.max(1.0))
}

fn tile_count(work: &mut GeometryWork<'_, '_, '_>, count: f64) -> Result<NonZeroU64, PaintError> {
    if work.coordinate(|| count >= TILE_COUNT_LIMIT)? {
        return Err(geometry_error());
    }
    work.coordinate(|| NonZeroU64::new(count as u64))?
        .ok_or_else(geometry_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BorderImageOutset, BorderImageRequest, BorderImageSlice, BorderImageSource,
        BorderImageWidth, PaintChange, PaintLimits, ProfileMode, ResourceKind,
        resolve_border_image,
    };
    use std::mem::size_of;
    use surgeist_layout::{Edges, Point};

    fn options(limits: PaintLimits) -> PaintOptions {
        PaintOptions::new(limits, ProfileMode::Summary { timing: false })
    }

    fn request() -> BorderImageRequest {
        BorderImageRequest {
            border_origin: Point::new(-8.0, 4.0),
            border_size: Size::new(34.0, 26.0),
            border_widths: Edges::new(4.0, 3.0, 2.0, 3.0),
            source: BorderImageSource {
                coordinate_size: Size::new(12.0, 10.0),
                natural_size: Size::new(Some(6.0), Some(5.0)),
            },
            slices: Edges::new(2.0, 3.0, 1.0, 3.0).map(BorderImageSlice::SourceUnits),
            widths: Edges::all(BorderImageWidth::BorderWidthMultiple(1.0)),
            outsets: Edges::all(BorderImageOutset::Length(0.0)),
            fill_center: true,
        }
    }

    fn geometry(request: BorderImageRequest) -> BorderImageGeometry {
        resolve_border_image(request, options(PaintLimits::unlimited()))
            .result
            .unwrap()
    }

    fn tiling(request: BorderImageRequest, repeat: Size<BorderImageRepeat>) -> BorderImageTiling {
        resolve_border_image_tiling(
            &geometry(request),
            repeat,
            options(PaintLimits::unlimited()),
        )
        .result
        .unwrap()
    }

    #[test]
    fn tiling_scales_edges_centers_repeat_and_distributes_space() {
        use BorderImageRepeat::{Repeat, Space};
        let plan = tiling(request(), Size::new(Repeat, Space));
        let [[corner, top, _], [left, middle, _], _] = *plan.regions();
        let corner = corner.unwrap();
        assert_eq!(corner.source().min(), Point::new(0.0, 0.0));
        assert_eq!(corner.source().max(), Point::new(3.0, 2.0));
        assert_eq!(corner.clip().min(), Point::new(-8.0, 4.0));
        assert_eq!(corner.axes().width, BorderImageAxisPattern::stretch(3.0));
        assert_eq!(corner.axes().height, BorderImageAxisPattern::stretch(4.0));

        let top = top.unwrap();
        assert_eq!(top.source().min(), Point::new(3.0, 0.0));
        assert_eq!(top.source().max(), Point::new(9.0, 2.0));
        assert_eq!(top.clip().min(), Point::new(-5.0, 4.0));
        assert_eq!(top.clip().max(), Point::new(23.0, 8.0));
        assert_eq!(top.axes().width.extent(), 12.0);
        assert_eq!(top.axes().width.anchor(), -4.0);
        assert_eq!(top.axes().width.count(), None);
        assert_eq!(top.axes().height, BorderImageAxisPattern::stretch(4.0));

        let left = left.unwrap();
        assert_eq!(left.axes().width, BorderImageAxisPattern::stretch(3.0));
        assert_eq!(left.axes().height.extent(), 7.0);
        assert_eq!(left.axes().height.count().unwrap().get(), 2);
        assert_eq!(left.axes().height.gap(), 2.0);
        assert_eq!(left.axes().height.anchor(), 2.0);

        let middle = middle.unwrap();
        assert_eq!(middle.source().min(), Point::new(3.0, 2.0));
        assert_eq!(middle.source().max(), Point::new(9.0, 9.0));
        assert_eq!(middle.axes().width, top.axes().width);
        assert_eq!(middle.axes().height, left.axes().height);
    }

    #[test]
    fn tiling_middle_uses_opposite_edge_then_unscaled_source_and_honors_fill() {
        use BorderImageRepeat::Repeat;
        let mut input = request();
        input.border_widths = Edges::new(0.0, 6.0, 3.0, 0.0);
        let repeat = Size::new(Repeat, Repeat);
        let plan = tiling(input, repeat);
        assert!(plan.regions()[0][1].is_none());
        assert!(plan.regions()[1][0].is_none());
        let axes = plan.regions()[1][1].unwrap().axes();
        assert_eq!(axes.width.extent(), 18.0);
        assert_eq!(axes.height.extent(), 14.0);

        input.border_widths = Edges::all(0.0);
        let plan = tiling(input, repeat);
        let axes = plan.regions()[1][1].unwrap().axes();
        assert_eq!(axes.width.extent(), 6.0);
        assert_eq!(axes.height.extent(), 7.0);

        input.fill_center = false;
        assert!(
            tiling(input, repeat)
                .regions()
                .iter()
                .flatten()
                .all(Option::is_none)
        );
    }

    #[test]
    fn tiling_round_fits_whole_tiles_and_space_can_discard_a_region() {
        use BorderImageRepeat::{Round, Space, Stretch};
        let mut input = request();
        input.border_size.width = 36.0;
        let plan = tiling(input, Size::new(Round, Stretch));
        let top = plan.regions()[0][1].unwrap().axes().width;
        assert_eq!(top.extent(), 10.0);
        assert_eq!(top.count().unwrap().get(), 3);
        assert_eq!(top.anchor(), 0.0);
        assert_eq!(top.gap(), 0.0);
        assert_eq!(plan.regions()[1][1].unwrap().axes().width, top);

        input.border_size.width = 11.0;
        input.border_widths.bottom = 0.5;
        let plan = tiling(input, Size::new(Space, Stretch));
        assert!(plan.regions()[0][1].is_none());
        // The top scale still determines the middle even if Space drops the top tiles.
        assert!(plan.regions()[1][1].is_none());
        let bottom = plan.regions()[2][1].unwrap().axes().width;
        assert_eq!(bottom.extent(), 3.0);
        assert_eq!(bottom.count().unwrap().get(), 1);
        assert_eq!(bottom.gap(), 1.0);
        assert_eq!(bottom.anchor(), 1.0);

        let repeat = tiling(input, Size::new(BorderImageRepeat::Repeat, Stretch));
        assert_eq!(repeat.regions()[0][1].unwrap().axes().width.anchor(), -3.5);
    }

    fn axis(
        repeat: BorderImageRepeat,
        extent: f64,
        tile: f64,
    ) -> Result<Option<BorderImageAxisPattern>, PaintError> {
        run(
            options(PaintLimits::unlimited()),
            OperationKind::BorderImageTiling,
            PaintPhase::BorderImageTiling,
            None::<fn(&mut GeometryWork<'_, '_, '_>) -> Result<(), PaintError>>,
            |work| axis_pattern(work, repeat, extent, tile),
        )
        .0
    }

    #[test]
    fn tiling_counts_use_the_true_whole_and_half_tile_boundaries() {
        use BorderImageRepeat::{Round, Space};
        // The binary64 value 0.1 exceeds one tenth, so only four tiles fit.
        let space = axis(Space, 0.5, 0.1).unwrap().unwrap();
        assert_eq!(space.count().unwrap().get(), 4);
        assert!((space.gap() - 0.02).abs() < 1e-17);

        // 7/5 rounds to one tile. Both the remainder and the rounded half
        // tile equal two subnormal units, but the exact half exceeds it.
        let extent = f64::from_bits(7);
        let tile = f64::from_bits(5);
        let round = axis(Round, extent, tile).unwrap().unwrap();
        assert_eq!(round.count().unwrap().get(), 1);
        assert_eq!(round.extent(), extent);
        assert_eq!(axis(Round, 1.0, f64::MIN_POSITIVE), Err(geometry_error()));
    }

    #[test]
    fn tiling_scales_along_the_admitted_clip_thickness() {
        let mut input = request();
        input.border_origin.y = 9_007_199_254_740_992.0;
        input.border_widths.top = 3.0;
        let repeat = Size::new(BorderImageRepeat::Repeat, BorderImageRepeat::Stretch);
        let plan = tiling(input, repeat);
        let top = plan.regions()[0][1].unwrap().axes();
        assert_eq!(top.height.extent(), 4.0);
        assert_eq!(top.width.extent(), 12.0);
        assert_eq!(plan.regions()[1][1].unwrap().axes().width.extent(), 12.0);
    }

    fn center_only(destination_width: f64, source_width: f64) -> BorderImageRequest {
        BorderImageRequest {
            border_origin: Point::new(0.0, 0.0),
            border_size: Size::new(destination_width, 1.0),
            border_widths: Edges::all(0.0),
            source: BorderImageSource {
                coordinate_size: Size::new(source_width, 1.0),
                natural_size: Size::new(None, None),
            },
            slices: Edges::all(BorderImageSlice::SourceUnits(0.0)),
            ..request()
        }
    }

    #[test]
    fn tiling_refuses_space_whose_rounded_gap_clips_the_last_tile() {
        let input = center_only(f64::from_bits(23), f64::from_bits(5));
        let repeat = Size::new(BorderImageRepeat::Space, BorderImageRepeat::Stretch);
        let outcome = resolve_border_image_tiling(
            &geometry(input),
            repeat,
            options(PaintLimits::unlimited()),
        );
        assert_eq!(outcome.result, Err(geometry_error()));
    }

    #[test]
    fn tiling_keeps_centered_repeat_phase_at_large_tile_counts() {
        let input = center_only(18_014_398_509_481_984.0, 1.0);
        let repeat = Size::new(BorderImageRepeat::Repeat, BorderImageRepeat::Stretch);
        let plan = tiling(input, repeat);
        let axis = plan.regions()[1][1].unwrap().axes().width;
        assert_eq!(axis.extent(), 1.0);
        assert_eq!(axis.anchor().rem_euclid(1.0), 0.5);
    }

    #[test]
    fn tiling_refuses_an_unrepresentable_subnormal_repeat_phase() {
        let input = center_only(f64::from_bits(6), f64::from_bits(3));
        let repeat = Size::new(BorderImageRepeat::Repeat, BorderImageRepeat::Stretch);
        let outcome = resolve_border_image_tiling(
            &geometry(input),
            repeat,
            options(PaintLimits::unlimited()),
        );
        // The ideal anchor is negative 1.5 minimum-subnormal units.
        assert_eq!(outcome.result, Err(geometry_error()));

        let exact = tiling(center_only(f64::from_bits(6), f64::from_bits(2)), repeat);
        let axis = exact.regions()[1][1].unwrap().axes().width;
        assert_eq!(axis.anchor(), 0.0);
    }

    #[test]
    fn tiling_refuses_subnormal_space_with_unequal_outer_gaps() {
        let input = center_only(f64::from_bits(205), f64::from_bits(100));
        let repeat = Size::new(BorderImageRepeat::Space, BorderImageRepeat::Stretch);
        let outcome = resolve_border_image_tiling(
            &geometry(input),
            repeat,
            options(PaintLimits::unlimited()),
        );
        // Rounding the ideal 5/3-unit gap to two leaves only one unit at the end.
        assert_eq!(outcome.result, Err(geometry_error()));

        let exact = tiling(
            center_only(f64::from_bits(206), f64::from_bits(100)),
            repeat,
        );
        let axis = exact.regions()[1][1].unwrap().axes().width;
        assert_eq!(axis.count().unwrap().get(), 2);
        assert_eq!(axis.gap(), f64::from_bits(2));
    }

    #[test]
    fn tiling_reports_refusal_prefix_and_releases_a_provisional_plan() {
        use BorderImageRepeat::{Repeat, Space};
        let geometry = geometry(request());
        let repeat = Size::new(Repeat, Space);
        for (kind, cap, expected_phase) in [
            (
                WorkKind::BorderImageCoordinate,
                1,
                PaintPhase::BorderImageTiling,
            ),
            (WorkKind::OutputRecord, 9, PaintPhase::Publication),
        ] {
            let limited = options(PaintLimits::unlimited().with_work_limit(kind, cap));
            let outcome = resolve_border_image_tiling(&geometry, repeat, limited);
            let Err(PaintError::Budget(refusal)) = outcome.result else {
                panic!("expected a budget refusal");
            };
            assert_eq!(refusal.origin, expected_phase);
            assert_eq!(refusal.current, cap);
            assert_eq!(refusal.change, PaintChange::Work { kind, amount: 1 });
            let report = outcome.report.unwrap();
            assert_eq!(report.work(kind), cap);
            assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
            assert_eq!(report.live(ResourceKind::RetainedResultBytes), 0);
            if kind == WorkKind::OutputRecord {
                assert_eq!(
                    report.peak(ResourceKind::RetainedResultBytes),
                    size_of::<BorderImageTiling>() as u64
                );
            }
        }
        let retried =
            resolve_border_image_tiling(&geometry, repeat, options(PaintLimits::unlimited()));
        assert!(
            retried
                .result
                .unwrap()
                .regions()
                .iter()
                .flatten()
                .all(Option::is_some)
        );
        let report = retried.report.unwrap();
        assert_eq!(report.work(WorkKind::OutputRecord), 10);
        assert_eq!(
            report.live(ResourceKind::RetainedResultBytes),
            size_of::<BorderImageTiling>() as u64
        );
        assert!(
            !report
                .phases()
                .any(|phase| phase.phase() == PaintPhase::PaintValidation)
        );
    }
}
