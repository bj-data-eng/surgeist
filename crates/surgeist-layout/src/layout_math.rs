use crate::geometry::{FlowAxes, LogicalEdgesOf, LogicalSizeOf, PhysicalAxis, PhysicalProgression};
use crate::scroll::ScrollOriginProgression;
use crate::{
    AlignItems, AspectRatioOf, BoxSizing, Edges, LayoutScalar, LengthOf, LengthResolutionOf,
    LengthResolutionStatus, Size,
};

// With two definite insets the logical end inset is weaker. Retaining the
// start and reducing the end establishes the nonnegative used interval.
pub(crate) fn floor_absolute_definite_insets<S: LayoutScalar>(
    inset: Edges<Option<S>>,
    containing_size: Size<S>,
    flow_axes: FlowAxes,
) -> Edges<Option<S>> {
    let inset = flow_axes.logical_edges(inset);
    let size = flow_axes.logical_size(containing_size);
    let floor_end = |extent: S, start: Option<S>, end: Option<S>| {
        start
            .zip(end)
            .map_or(end, |(start, end)| Some(end.min(extent - start)))
    };
    flow_axes.physical_edges(LogicalEdgesOf::new(
        inset.inline_start,
        floor_end(size.inline, inset.inline_start, inset.inline_end),
        inset.block_start,
        floor_end(size.block, inset.block_start, inset.block_end),
    ))
}

fn absolute_inset_axis_extent<S: LayoutScalar>(
    containing: S,
    start: Option<S>,
    end: Option<S>,
) -> Option<S> {
    (start.is_some() || end.is_some())
        .then(|| (containing - start.unwrap_or(S::ZERO) - end.unwrap_or(S::ZERO)).max(S::ZERO))
}

pub(crate) fn absolute_inset_extents<S: LayoutScalar>(
    inset: Edges<Option<S>>,
    containing: Size<S>,
    flow: FlowAxes,
) -> Size<Option<S>> {
    let inset = flow.logical_edges(inset);
    let containing = flow.logical_size(containing);
    flow.physical_size(LogicalSizeOf::new(
        absolute_inset_axis_extent(containing.inline, inset.inline_start, inset.inline_end),
        absolute_inset_axis_extent(containing.block, inset.block_start, inset.block_end),
    ))
}

pub(crate) fn absolute_inset_child_axes<S: LayoutScalar>(
    available: Size<crate::AxisSizingRequestOf<S>>,
    inset_extents: Size<Option<S>>,
    margin: Edges<Option<S>>,
    measured_leaf: bool,
) -> Size<Result<crate::AxisSizingRequestOf<S>, LengthResolutionStatus<S>>> {
    // The leaf receiver debits its margins while resolving content space.
    let margin_debit = if measured_leaf {
        Size::ZERO
    } else {
        margin.map(|margin| margin.unwrap_or(S::ZERO)).sum_axes()
    };
    let extents = inset_extents.zip_map(margin_debit, |extent, debit| {
        extent
            .map(|extent| remaining_available_extent(extent, debit))
            .transpose()
    });
    available.zip_map(extents, |request, extent| {
        with_available_extent(request, extent?)
    })
}

/// Authored sizing declarations of an absolutely positioned box, resolved in
/// the box selected by `box-sizing` and before any ratio work.
pub(crate) struct PositionedSizeDeclarations<S: LayoutScalar> {
    pub(crate) preferred: Size<Option<S>>,
    pub(crate) min: Size<Option<S>>,
    pub(crate) max: Size<Option<S>>,
    pub(crate) aspect_ratio: Option<AspectRatioOf<S>>,
    pub(crate) box_sizing: BoxSizing,
    pub(crate) padding_border: Size<S>,
}

/// Preferred, minimum and maximum border-box sizes of an absolutely
/// positioned box, shared by every parent formatting context.
///
/// The ratio works with the box selected by `box-sizing` (Sizing 4 §4.1).
/// A definite preferred size transfers through the ratio to an automatic
/// axis. A definite minimum transfers to an indefinite minimum, capped by the
/// declared preferred and maximum sizes of that axis. A definite maximum
/// transfers to an indefinite maximum, floored by the declared preferred and
/// minimum sizes of that axis and by the transferred minimum (Sizing 4 §4.4).
/// The border box never drops below padding plus border (Sizing 3 §3.1), and
/// clamping applies the maximum before the minimum so the minimum wins
/// (Sizing 3 §3.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PositionedUsedSize<S: LayoutScalar> {
    pub(crate) preferred: Size<Option<S>>,
    min: Size<S>,
    max: Size<Option<S>>,
    box_adjustment: Size<S>,
}

impl<S: LayoutScalar> PositionedUsedSize<S> {
    pub(crate) fn resolve(declarations: PositionedSizeDeclarations<S>) -> Self {
        let PositionedSizeDeclarations {
            preferred,
            min,
            max,
            aspect_ratio,
            box_sizing,
            padding_border,
        } = declarations;
        let box_adjustment = if box_sizing == BoxSizing::ContentBox {
            padding_border
        } else {
            Size::ZERO
        };
        let (min, max) = match aspect_ratio {
            Some(ratio) => Self::transfer_limits(preferred, min, max, ratio.get()),
            None => (min, max),
        };
        Self {
            preferred: preferred
                .apply_aspect_ratio(aspect_ratio)
                .add_optional(box_adjustment),
            min: min
                .add_optional(box_adjustment)
                .unwrap_or(padding_border)
                .max_optional(padding_border.map(Some)),
            max: max.add_optional(box_adjustment),
            box_adjustment,
        }
    }

    // Caps and floors come from the declared preferred sizes, not from a size
    // derived through the ratio. The padding floor is applied later, in `clamp`.
    fn transfer_limits(
        preferred: Size<Option<S>>,
        min: Size<Option<S>>,
        max: Size<Option<S>>,
        ratio: S,
    ) -> (Size<Option<S>>, Size<Option<S>>) {
        let cap = |value: S, first: Option<S>, second: Option<S>| {
            let value = first.map_or(value, |first| value.min(first));
            second.map_or(value, |second| value.min(second))
        };
        let floor = |value: S, floors: [Option<S>; 3]| {
            floors
                .into_iter()
                .flatten()
                .fold(value, |value, floor| value.max(floor))
        };
        let transferred_min = Size::new(
            min.width.or_else(|| {
                min.height
                    .map(|height| cap(height * ratio, preferred.width, max.width))
            }),
            min.height.or_else(|| {
                min.width
                    .map(|width| cap(width / ratio, preferred.height, max.height))
            }),
        );
        let transferred_max = Size::new(
            max.width.or_else(|| {
                max.height.map(|height| {
                    floor(
                        height * ratio,
                        [preferred.width, min.width, transferred_min.width],
                    )
                })
            }),
            max.height.or_else(|| {
                max.width.map(|width| {
                    floor(
                        width / ratio,
                        [preferred.height, min.height, transferred_min.height],
                    )
                })
            }),
        );
        (transferred_min, transferred_max)
    }

    pub(crate) fn clamp(self, size: Size<S>) -> Size<S> {
        size.clamp_max_before_min_optional(self.min.map(Some), self.max)
    }

    pub(crate) fn clamp_optional(self, size: Size<Option<S>>) -> Size<Option<S>> {
        size.clamp_max_before_min_optional(self.min.map(Some), self.max)
    }

    /// Fills the missing axis of a border-box size through `ratio` in the box
    /// selected by `box-sizing`.
    pub(crate) fn transfer_ratio(
        self,
        size: Size<Option<S>>,
        ratio: Option<AspectRatioOf<S>>,
    ) -> Size<Option<S>> {
        let Some(ratio) = ratio else {
            return size;
        };
        let ratio = ratio.get();
        let adjustment = self.box_adjustment;
        match (size.width, size.height) {
            (Some(width), None) => Size::new(
                Some(width),
                Some((width - adjustment.width) / ratio + adjustment.height),
            ),
            (None, Some(height)) => Size::new(
                Some((height - adjustment.height) * ratio + adjustment.width),
                Some(height),
            ),
            _ => size,
        }
    }
}

pub(crate) fn absolute_stretch_fit_sizes<S: LayoutScalar>(
    inset: Edges<Option<S>>,
    margin: Edges<Option<S>>,
    extent: Size<Option<S>>,
    alignment: Size<Option<AlignItems>>,
    child: &crate::NodeInputOf<S>,
    padding_border_size: Size<S>,
) -> (Size<Option<S>>, Option<AspectRatioOf<S>>) {
    let stretch = |insets: (Option<S>, Option<S>),
                   margins: (Option<S>, Option<S>),
                   extent: Option<S>,
                   alignment| {
        if insets.0.is_none() || insets.1.is_none() {
            return None;
        }
        let stretches = match alignment {
            None => !child.item_is_replaced && !child.item_is_table,
            Some(AlignItems::Stretch) => margins.0.is_some() && margins.1.is_some(),
            Some(_) => false,
        };
        extent.filter(|_| stretches).map(|extent| {
            (extent - margins.0.unwrap_or(S::ZERO) - margins.1.unwrap_or(S::ZERO)).max(S::ZERO)
        })
    };
    let stretch_sizes = Size::new(
        stretch(
            (inset.left, inset.right),
            (margin.left, margin.right),
            extent.width,
            alignment.width,
        ),
        stretch(
            (inset.top, inset.bottom),
            (margin.top, margin.bottom),
            extent.height,
            alignment.height,
        ),
    );
    let Some(ratio) = child.aspect_ratio else {
        return (stretch_sizes, None);
    };
    if !child.size.width.is_auto() || !child.size.height.is_auto() {
        return (stretch_sizes, Some(ratio));
    }

    let flow = FlowAxes::new(child.writing_mode, child.direction);
    let inset = flow.logical_edges(inset);
    let inline_has_auto = inset.inline_start.is_none() || inset.inline_end.is_none();
    let block_has_auto = inset.block_start.is_none() || inset.block_end.is_none();
    let dependent_axis = if inline_has_auto && !block_has_auto {
        flow.inline_axis()
    } else {
        flow.block_axis()
    };
    let determining_border = match dependent_axis {
        PhysicalAxis::Horizontal => Size::new(
            None,
            stretch_sizes
                .height
                .map(|height| height.max(padding_border_size.height)),
        ),
        PhysicalAxis::Vertical => Size::new(
            stretch_sizes
                .width
                .map(|width| width.max(padding_border_size.width)),
            None,
        ),
    };
    if determining_border == Size::NONE {
        return (stretch_sizes, Some(ratio));
    }

    // Stretch fit floors the determining border box at its padding and border.
    // The ratio then transfers within the box selected by box-sizing.
    let sizing_box_inset = if child.box_sizing == crate::BoxSizing::ContentBox {
        padding_border_size
    } else {
        Size::ZERO
    };
    let transferred = determining_border
        .zip_map(sizing_box_inset, |extent, inset| {
            extent.map(|extent| extent - inset)
        })
        .apply_aspect_ratio(Some(ratio))
        .add_optional(sizing_box_inset);
    (determining_border.or(transferred), None)
}

fn absolute_inset_axis_alignment<S: LayoutScalar>(
    inset: (Option<S>, Option<S>),
    margin: (Option<S>, Option<S>),
    item: S,
    containing: S,
    alignment: Option<AlignItems>,
    scroll_progression: Option<ScrollOriginProgression>,
) -> S {
    let ((Some(start), Some(end)), (Some(margin_start), Some(margin_end)), Some(alignment)) =
        (inset, margin, alignment)
    else {
        return S::ZERO;
    };
    let empty_interval_slack = absolute_alignment_sum([-item, -margin_start, -margin_end]);
    let modified_interval_slack =
        absolute_alignment_sum([containing, -item, -start, -end, -margin_start, -margin_end]);
    // Floor the modified interval in slack coordinates. Materializing a
    // reduced end inset can round a crossed-inset point into an interval.
    let free = modified_interval_slack.max(empty_interval_slack);
    let offset = match alignment.safe_fallback(free) {
        AlignItems::Center => free / S::from_usize(2),
        AlignItems::End | AlignItems::FlexEnd | AlignItems::LastBaseline => free,
        _ => S::ZERO,
    };
    if free >= S::ZERO
        || !matches!(
            alignment,
            AlignItems::Start
                | AlignItems::End
                | AlignItems::FlexStart
                | AlignItems::FlexEnd
                | AlignItems::Center
        )
    {
        return offset;
    }

    let limit_start = (-start).min(S::ZERO);
    // Covering the modified interval bounds the offset to [free, 0].
    // Intersect with the reachable limit in the same coordinate frame.
    let mut earliest = free;
    let mut latest = S::ZERO;
    if scroll_progression != Some(ScrollOriginProgression::FlowStartward) {
        earliest = earliest.max(limit_start);
    }
    if scroll_progression != Some(ScrollOriginProgression::FlowEndward) {
        // The farther end of the original and modified intervals controls
        // reachability. Compute both slacks without rebuilding the margin box.
        let original_interval_slack =
            absolute_alignment_sum([containing, -item, -start, -margin_start, -margin_end]);
        latest = latest.min(free.max(original_interval_slack));
    }
    if earliest > latest {
        limit_start
    } else {
        offset.max(earliest).min(latest)
    }
}

pub(crate) fn absolute_inset_alignment_offsets<S: LayoutScalar>(
    inset: Edges<Option<S>>,
    margin: Edges<Option<S>>,
    item: Size<S>,
    containing: Size<S>,
    flow: FlowAxes,
    alignment: LogicalSizeOf<Option<AlignItems>>,
    scroll_progressions: LogicalSizeOf<Option<ScrollOriginProgression>>,
) -> Edges<S> {
    let inset = flow.logical_edges(inset);
    let margin = flow.logical_edges(margin);
    let item = flow.logical_size(item);
    let containing = flow.logical_size(containing);
    flow.physical_edges(LogicalEdgesOf::new(
        absolute_inset_axis_alignment(
            (inset.inline_start, inset.inline_end),
            (margin.inline_start, margin.inline_end),
            item.inline,
            containing.inline,
            alignment.inline,
            scroll_progressions.inline,
        ),
        S::ZERO,
        absolute_inset_axis_alignment(
            (inset.block_start, inset.block_end),
            (margin.block_start, margin.block_end),
            item.block,
            containing.block,
            alignment.block,
            scroll_progressions.block,
        ),
        S::ZERO,
    ))
}

pub(crate) struct AbsoluteInsetStartOf<S> {
    inset: S,
    margin: S,
    adjustment: S,
}

impl<S: LayoutScalar> AbsoluteInsetStartOf<S> {
    pub(crate) const fn new(inset: S, margin: S, adjustment: S) -> Self {
        Self {
            inset,
            margin,
            adjustment,
        }
    }

    pub(crate) fn project(
        self,
        area_start: S,
        size: S,
        extent: S,
        progression: PhysicalProgression,
    ) -> S {
        if progression.is_decreasing() {
            absolute_alignment_sum([
                extent,
                -size,
                -area_start,
                -self.inset,
                -self.margin,
                -self.adjustment,
            ])
        } else {
            absolute_alignment_sum([area_start, self.inset, self.margin, self.adjustment])
        }
    }
}

fn absolute_alignment_sum<S: LayoutScalar, const N: usize>(mut terms: [S; N]) -> S {
    // Retain low-order terms when a larger term can cancel later in the sum.
    let mut sum = S::ZERO;
    let mut remainder = S::ZERO;
    for index in 0..N {
        let mut term = terms[index];
        let mut next = sum + term;
        if !next.is_finite() {
            // Admit a later opposing term before retrying the term that overflowed.
            let rest = index + 1;
            if let Some(opposing) = terms[rest..]
                .iter()
                .position(|&other| other != S::ZERO && (sum < S::ZERO) != (other < S::ZERO))
            {
                terms.swap(index, rest + opposing);
                term = terms[index];
            } else {
                term = term + remainder;
                remainder = S::ZERO;
            }
            next = sum + term;
            if !next.is_finite() {
                return next;
            }
        }
        remainder = remainder
            + if sum.abs() >= term.abs() {
                (sum - next) + term
            } else {
                (term - next) + sum
            };
        sum = next;
    }
    sum + remainder
}

pub(crate) fn resolve_absolute_margins<S: LayoutScalar>(
    margin: Edges<Option<S>>,
    inset: Edges<Option<S>>,
    size: Size<S>,
    containing_size: Size<S>,
    flow_axes: FlowAxes,
) -> Edges<S> {
    let logical_margin = flow_axes.logical_edges(margin);
    let logical_inset = flow_axes.logical_edges(inset);
    let logical_size = flow_axes.logical_size(size);
    let logical_containing_size = flow_axes.logical_size(containing_size);
    let (inline_start, inline_end) = resolve_absolute_axis_margins(
        logical_margin.inline_start,
        logical_margin.inline_end,
        logical_inset.inline_start,
        logical_inset.inline_end,
        logical_size.inline,
        logical_containing_size.inline,
        true,
    );
    let (block_start, block_end) = resolve_absolute_axis_margins(
        logical_margin.block_start,
        logical_margin.block_end,
        logical_inset.block_start,
        logical_inset.block_end,
        logical_size.block,
        logical_containing_size.block,
        false,
    );

    flow_axes.physical_edges(LogicalEdgesOf::new(
        inline_start,
        inline_end,
        block_start,
        block_end,
    ))
}

pub(crate) fn resolve_absolute_axis_margins<S: LayoutScalar>(
    start_margin: Option<S>,
    end_margin: Option<S>,
    start_inset: Option<S>,
    end_inset: Option<S>,
    target_size: S,
    containing_size: S,
    anchor_negative_two_auto_at_start: bool,
) -> (S, S) {
    let fixed_start = start_margin.unwrap_or(S::ZERO);
    let fixed_end = end_margin.unwrap_or(S::ZERO);
    let (Some(start_inset), Some(end_inset)) = (start_inset, end_inset) else {
        return (fixed_start, fixed_end);
    };
    let remaining =
        containing_size - start_inset - end_inset - target_size - fixed_start - fixed_end;

    match (start_margin.is_none(), end_margin.is_none()) {
        (false, false) => (fixed_start, fixed_end),
        (true, false) => (remaining, fixed_end),
        (false, true) => (fixed_start, remaining),
        (true, true) if anchor_negative_two_auto_at_start && remaining < S::ZERO => {
            (S::ZERO, remaining)
        }
        (true, true) => {
            let half = remaining / S::from_usize(2);
            (half, half)
        }
    }
}

/// Checks a physical space produced by an owning sizing algorithm.
pub(crate) fn checked_available_space<S: LayoutScalar>(
    value: S,
) -> Result<crate::AvailableSpaceOf<S>, LengthResolutionStatus<S>> {
    crate::NonNegativeFiniteOf::new(value)
        .map(crate::AvailableSpaceOf::Definite)
        .map_err(|_| LengthResolutionStatus::InvalidNumeric { value })
}

/// Preserves the recipient of failures while lowering physical axis requests.
pub(crate) fn transpose_axis_requests_at_node<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    requests: Size<Result<crate::AxisSizingRequestOf<S>, LengthResolutionStatus<S>>>,
) -> crate::LayoutResultOf<Node, Size<crate::AxisSizingRequestOf<S>>, S, M> {
    Ok(Size::new(
        requests
            .width
            .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?,
        requests
            .height
            .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?,
    ))
}

/// Retains a request unless its producer supplies a replacement extent.
pub(crate) fn with_available_extent<S: LayoutScalar>(
    request: crate::AxisSizingRequestOf<S>,
    extent: Option<S>,
) -> Result<crate::AxisSizingRequestOf<S>, LengthResolutionStatus<S>> {
    extent
        .map(|value| checked_available_space(value).map(|space| request.with_space(space)))
        .transpose()
        .map(|value| value.unwrap_or(request))
}

/// Performs a signed edge debit before flooring remaining physical space.
pub(crate) fn remaining_available_extent<S: LayoutScalar>(
    extent: S,
    inset: S,
) -> Result<S, LengthResolutionStatus<S>> {
    for value in [extent, inset] {
        if !value.is_finite() {
            return Err(LengthResolutionStatus::InvalidNumeric { value });
        }
    }
    let remaining = extent - inset;
    if !remaining.is_finite() {
        return Err(LengthResolutionStatus::InvalidNumeric { value: remaining });
    }
    Ok(remaining.max(S::ZERO))
}

/// Lowers signed edge arithmetic without changing the sizing operation.
pub(crate) fn inset_axis_request<S: LayoutScalar>(
    request: crate::AxisSizingRequestOf<S>,
    inset: S,
) -> Result<crate::AxisSizingRequestOf<S>, LengthResolutionStatus<S>> {
    if !inset.is_finite() {
        return Err(LengthResolutionStatus::InvalidNumeric { value: inset });
    }
    let Some(space) = request.space().definite_size() else {
        return Ok(request);
    };
    let remaining = remaining_available_extent(space.get(), inset)?;
    Ok(request.with_space(checked_available_space(remaining)?))
}

/// A bound changes available space, never the requested intrinsic operation.
pub(crate) fn clamp_axis_request_space<S: LayoutScalar>(
    request: crate::AxisSizingRequestOf<S>,
    minimum: Option<S>,
    maximum: Option<S>,
) -> Result<crate::AxisSizingRequestOf<S>, LengthResolutionStatus<S>> {
    for value in [minimum, maximum].into_iter().flatten() {
        if !value.is_finite() {
            return Err(LengthResolutionStatus::InvalidNumeric { value });
        }
    }
    let bound = match request.operation() {
        crate::SizingOperation::MinContent => minimum,
        crate::SizingOperation::Normal | crate::SizingOperation::MaxContent => maximum,
    };
    let value = request
        .space()
        .definite_size()
        .map(|space| space.get().clamp_max_before_min_optional(minimum, maximum))
        .or(bound);
    Ok(request.with_space(match value {
        Some(value) => checked_available_space(value.max(S::ZERO))?,
        None => crate::AvailableSpaceOf::Indefinite,
    }))
}

/// Actual finite normal layout, excluding bounded intrinsic queries.
pub(crate) fn normal_available_size<S: LayoutScalar>(
    request: crate::AxisSizingRequestOf<S>,
) -> Option<S> {
    if request.operation() == crate::SizingOperation::Normal {
        request
            .space()
            .definite_size()
            .map(crate::NonNegativeFiniteOf::get)
    } else {
        None
    }
}

/// The sealed f32/f64 Layout scalars widen losslessly to the Text adapter lane.
pub(crate) fn text_axis_request<S: LayoutScalar>(
    request: crate::AxisSizingRequestOf<S>,
) -> crate::AxisSizingRequestOf<f64> {
    let space = match request.space() {
        crate::AvailableSpaceOf::Definite(value) => crate::AvailableSpaceOf::Definite(
            crate::NonNegativeFiniteOf::new(value.get().to_f64())
                .expect("a checked Layout scalar widens to finite nonnegative f64"),
        ),
        crate::AvailableSpaceOf::Indefinite => crate::AvailableSpaceOf::Indefinite,
    };
    crate::AxisSizingRequestOf::new(space, request.operation())
}

/// The actual physical query purpose, not its output mask or finiteness.
pub(crate) fn fallback_preferred_is_available<S: LayoutScalar>(
    input: crate::ComputeInputOf<S>,
    axis: crate::PhysicalAxis,
) -> bool {
    if input.sizing_mode() == crate::SizingMode::InherentSize {
        return true;
    }
    let axes = input.sizing_request().axes();
    let (current, opposite) = match axis {
        crate::PhysicalAxis::Horizontal => (axes.width, axes.height),
        crate::PhysicalAxis::Vertical => (axes.height, axes.width),
    };
    current.operation() == crate::SizingOperation::Normal
        && opposite.operation() != crate::SizingOperation::Normal
}

/// Qualified outer-box source roles for one physical axis.
struct FixedFallbackAxisSources<S: LayoutScalar> {
    definite_outer: crate::sizing::resolve::FixedFallbackSource<S>,
    minimum_outer: crate::sizing::resolve::FixedFallbackSource<S>,
    maximum_outer: crate::sizing::resolve::FixedFallbackSource<S>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FixedFallbackSources<S: LayoutScalar> {
    pub(crate) definite_outer: Size<crate::sizing::resolve::FixedFallbackSource<S>>,
    pub(crate) minimum_outer: Size<crate::sizing::resolve::FixedFallbackSource<S>>,
    pub(crate) maximum_outer: Size<crate::sizing::resolve::FixedFallbackSource<S>>,
}

/// Reads original owner properties only for the fixed-source roles needed by
/// its local cap and actual scrollport. Query/body substitutions never enter.
pub(crate) fn qualify_fixed_fallback_sources<S: LayoutScalar>(
    input: crate::ComputeInputOf<S>,
    algorithm: crate::SizingAlgorithm,
    common: &crate::node_projection::CommonBoxProjection<'_, S>,
    definite_outer: Size<Option<S>>,
    box_sizing_adjustment: Size<S>,
    is_scrollport: bool,
) -> Result<FixedFallbackSources<S>, crate::sizing::resolve::SizingResolutionError<S>> {
    use crate::sizing::resolve::{
        FixedFallbackSource, SizingResolutionError, fixed_fallback_maximum, fixed_fallback_minimum,
        fixed_fallback_preferred,
    };
    let qualify = |axis: crate::PhysicalAxis,
                   preferred: &crate::PreferredSizeOf<S>,
                   minimum: &crate::MinSizeOf<S>,
                   maximum: &crate::MaxSizeOf<S>,
                   known: Option<S>,
                   adjustment: S|
     -> Result<FixedFallbackAxisSources<S>, SizingResolutionError<S>> {
        let needed = fallback_preferred_is_available(input, axis) || known.is_some();
        let retain = |result| match result {
            Err(SizingResolutionError::Unsupported(_)) if !needed => {
                Ok(FixedFallbackSource::Unqualified)
            }
            result => result,
        };
        let outer =
            |source| outer_fallback_source(source, adjustment).map_err(SizingResolutionError::from);
        let maximum = outer(retain(fixed_fallback_maximum(
            maximum,
            algorithm,
            axis,
            match axis {
                crate::PhysicalAxis::Horizontal => input.parent().width,
                crate::PhysicalAxis::Vertical => input.parent().height,
            },
        ))?)?;
        let definite = if let Some(known) = known {
            FixedFallbackSource::Fixed(known)
        } else if !is_scrollport {
            FixedFallbackSource::Unqualified
        } else if needed {
            outer(fixed_fallback_preferred(
                preferred,
                algorithm,
                axis,
                match axis {
                    crate::PhysicalAxis::Horizontal => input.parent().width,
                    crate::PhysicalAxis::Vertical => input.parent().height,
                },
            )?)?
        } else if preferred.is_auto() || preferred.is_min_content() || preferred.is_max_content() {
            // These original keywords prove no fixed body without evaluating
            // a query-suppressed preferred expression.
            FixedFallbackSource::NoFixedBound
        } else {
            FixedFallbackSource::Unqualified
        };
        let minimum = if matches!(maximum, FixedFallbackSource::Fixed(_))
            || is_scrollport && matches!(definite, FixedFallbackSource::Fixed(_))
        {
            outer(retain(fixed_fallback_minimum(
                minimum,
                algorithm,
                axis,
                match axis {
                    crate::PhysicalAxis::Horizontal => input.parent().width,
                    crate::PhysicalAxis::Vertical => input.parent().height,
                },
            ))?)?
        } else {
            FixedFallbackSource::Unqualified
        };
        Ok(FixedFallbackAxisSources {
            definite_outer: definite,
            minimum_outer: minimum,
            maximum_outer: maximum,
        })
    };
    let width = qualify(
        crate::PhysicalAxis::Horizontal,
        &common.size.width,
        &common.min_size.width,
        &common.max_size.width,
        definite_outer.width,
        box_sizing_adjustment.width,
    )?;
    let height = qualify(
        crate::PhysicalAxis::Vertical,
        &common.size.height,
        &common.min_size.height,
        &common.max_size.height,
        definite_outer.height,
        box_sizing_adjustment.height,
    )?;
    Ok(FixedFallbackSources {
        definite_outer: Size::new(width.definite_outer, height.definite_outer),
        minimum_outer: Size::new(width.minimum_outer, height.minimum_outer),
        maximum_outer: Size::new(width.maximum_outer, height.maximum_outer),
    })
}

/// Converts a qualified property value to its outer-box source exactly once.
pub(crate) fn outer_fallback_source<S: LayoutScalar>(
    source: crate::sizing::resolve::FixedFallbackSource<S>,
    box_sizing_adjustment: S,
) -> Result<crate::sizing::resolve::FixedFallbackSource<S>, LengthResolutionStatus<S>> {
    use crate::sizing::resolve::FixedFallbackSource;
    match source {
        FixedFallbackSource::Fixed(value) => {
            for value in [value, box_sizing_adjustment] {
                if !value.is_finite() {
                    return Err(LengthResolutionStatus::InvalidNumeric { value });
                }
            }
            let value = value + box_sizing_adjustment;
            if !value.is_finite() {
                return Err(LengthResolutionStatus::InvalidNumeric { value });
            }
            Ok(FixedFallbackSource::Fixed(value))
        }
        source => Ok(source),
    }
}

fn inner_fallback_source<S: LayoutScalar>(
    source: crate::sizing::resolve::FixedFallbackSource<S>,
    debit: S,
) -> Result<crate::sizing::resolve::FixedFallbackSource<S>, LengthResolutionStatus<S>> {
    use crate::sizing::resolve::FixedFallbackSource;
    match source {
        FixedFallbackSource::Fixed(value) => {
            remaining_available_extent(value, debit).map(FixedFallbackSource::Fixed)
        }
        source => Ok(source),
    }
}

fn fixed_fallback_value<S: LayoutScalar>(
    source: crate::sizing::resolve::FixedFallbackSource<S>,
) -> Option<crate::NonNegativeFiniteOf<S>> {
    match source {
        crate::sizing::resolve::FixedFallbackSource::Fixed(value) => Some(
            crate::NonNegativeFiniteOf::new(value)
                .expect("the owning checked debit produced a finite nonnegative inner bound"),
        ),
        _ => None,
    }
}

/// Qualifies local physical containing bounds from original owner evidence.
/// A known absent fixed maximum makes the minimum irrelevant to this cap.
pub(crate) fn containing_orthogonal_fallback_limits<S: LayoutScalar>(
    minimum_outer: Size<crate::sizing::resolve::FixedFallbackSource<S>>,
    maximum_outer: Size<crate::sizing::resolve::FixedFallbackSource<S>>,
    debit: Size<S>,
) -> Result<Size<crate::OrthogonalFallbackLimitOf<S>>, LengthResolutionStatus<S>> {
    use crate::sizing::resolve::FixedFallbackSource;
    let axis = |minimum,
                maximum,
                debit|
     -> Result<crate::OrthogonalFallbackLimitOf<S>, LengthResolutionStatus<S>> {
        match maximum {
            FixedFallbackSource::NoFixedBound => Ok(crate::OrthogonalFallbackLimitOf::Unbounded),
            FixedFallbackSource::Unqualified => Ok(crate::OrthogonalFallbackLimitOf::Unknown),
            FixedFallbackSource::Fixed(_) => {
                if matches!(minimum, FixedFallbackSource::Unqualified) {
                    return Ok(crate::OrthogonalFallbackLimitOf::Unknown);
                }
                Ok(crate::OrthogonalFallbackLimitOf::from_containing_bounds(
                    fixed_fallback_value(inner_fallback_source(minimum, debit)?),
                    fixed_fallback_value(inner_fallback_source(maximum, debit)?),
                ))
            }
        }
    };
    Ok(Size::new(
        axis(minimum_outer.width, maximum_outer.width, debit.width)?,
        axis(minimum_outer.height, maximum_outer.height, debit.height)?,
    ))
}

/// Qualifies this containing owner once for normal AND positioned descendants.
/// Fixed sources precede query suppression/measurement. Requested insets come
/// from the real current scrollbar settlement, not provisional rectangle sizes.
pub(crate) fn child_orthogonal_fallback<S: LayoutScalar>(
    inherited: crate::OrthogonalFallbackOf<S>,
    definite_outer: Size<crate::sizing::resolve::FixedFallbackSource<S>>,
    minimum_outer: Size<crate::sizing::resolve::FixedFallbackSource<S>>,
    maximum_outer: Size<crate::sizing::resolve::FixedFallbackSource<S>>,
    source: crate::scroll::MeasuredLeafContentBoxInsetSourceOf<S>,
) -> Result<crate::OrthogonalFallbackOf<S>, LengthResolutionStatus<S>> {
    use crate::sizing::resolve::FixedFallbackSource;
    let content_debit = crate::scroll::measured_leaf_content_box_inset(source).sum_axes();
    let local = containing_orthogonal_fallback_limits(minimum_outer, maximum_outer, content_debit)?;
    let change = if crate::scroll::UsedOverflow::from_computed(
        source.computed_overflow,
        source.item_is_replaced,
    )
    .is_scroll_container()
    {
        let debit = crate::scroll::measured_leaf_content_box_inset(
            crate::scroll::MeasuredLeafContentBoxInsetSourceOf {
                padding: Edges::ZERO,
                ..source
            },
        )
        .sum_axes();
        let axis =
            |definite,
             minimum,
             maximum,
             debit|
             -> Result<crate::OrthogonalFallbackLimitOf<S>, LengthResolutionStatus<S>> {
                if matches!(
                    (definite, maximum),
                    (
                        FixedFallbackSource::NoFixedBound,
                        FixedFallbackSource::NoFixedBound
                    )
                ) {
                    return Ok(crate::OrthogonalFallbackLimitOf::Unbounded);
                }
                if matches!(definite, FixedFallbackSource::Unqualified)
                    || matches!(maximum, FixedFallbackSource::Unqualified)
                    || matches!(minimum, FixedFallbackSource::Unqualified)
                {
                    return Ok(crate::OrthogonalFallbackLimitOf::Unknown);
                }
                Ok(crate::OrthogonalFallbackLimitOf::from_scrollport(
                    fixed_fallback_value(inner_fallback_source(definite, debit)?),
                    fixed_fallback_value(inner_fallback_source(minimum, debit)?),
                    fixed_fallback_value(inner_fallback_source(maximum, debit)?),
                ))
            };
        crate::ScrollportFallbackChangeOf::Enter(Size::new(
            axis(
                definite_outer.width,
                minimum_outer.width,
                maximum_outer.width,
                debit.width,
            )?,
            axis(
                definite_outer.height,
                minimum_outer.height,
                maximum_outer.height,
                debit.height,
            )?,
        ))
    } else {
        crate::ScrollportFallbackChangeOf::Preserve
    };
    Ok(inherited.for_child(local, change))
}

pub(crate) fn resolution_or_zero<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution
            .value
            .expect("resolved length resolution must carry a value")),
        LengthResolutionStatus::InvalidNumeric { .. } => Err(resolution.status()),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::NonNumeric => Ok(S::ZERO),
    }
}

pub(crate) fn resolution_optional<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution.value),
        LengthResolutionStatus::InvalidNumeric { .. } => Err(resolution.status()),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::NonNumeric => Ok(None),
    }
}

pub(crate) fn resolve_containing_padding_border<S: LayoutScalar, E>(
    containing_flow_axes: crate::geometry::FlowAxes,
    parent: Size<Option<S>>,
    padding: Edges<LengthOf<S>>,
    border: Edges<LengthOf<S>>,
    resolve_length: impl Fn(LengthOf<S>, Option<S>) -> Result<S, LengthResolutionStatus<S>>,
    mut transpose: impl FnMut(Edges<Result<S, LengthResolutionStatus<S>>>) -> Result<Edges<S>, E>,
) -> Result<(Edges<S>, Edges<S>), E> {
    let padding = transpose(containing_flow_axes.zip_physical_edges_with_inline_extent(
        padding,
        parent,
        &resolve_length,
    ))?;
    let border = transpose(containing_flow_axes.zip_physical_edges_with_inline_extent(
        border,
        parent,
        resolve_length,
    ))?;
    Ok((padding, border))
}

pub(crate) trait OptionalSizeExt {
    type Scalar: LayoutScalar;

    fn or(self, other: Self) -> Self;
    fn unwrap_or(self, fallback: Size<Self::Scalar>) -> Size<Self::Scalar>;
    fn add_optional(self, amount: Size<Self::Scalar>) -> Self;
    fn apply_aspect_ratio(self, aspect_ratio: Option<AspectRatioOf<Self::Scalar>>) -> Self;
}

impl<S: LayoutScalar> OptionalSizeExt for Size<Option<S>> {
    type Scalar = S;

    fn or(self, other: Self) -> Self {
        Size::new(self.width.or(other.width), self.height.or(other.height))
    }

    fn unwrap_or(self, fallback: Size<S>) -> Size<S> {
        Size::new(
            self.width.unwrap_or(fallback.width),
            self.height.unwrap_or(fallback.height),
        )
    }

    fn add_optional(self, amount: Size<S>) -> Self {
        Size::new(
            self.width.map(|width| width + amount.width),
            self.height.map(|height| height + amount.height),
        )
    }

    fn apply_aspect_ratio(self, aspect_ratio: Option<AspectRatioOf<S>>) -> Self {
        let Some(ratio) = aspect_ratio else {
            return self;
        };
        let ratio = ratio.get();
        match (self.width, self.height) {
            (Some(width), None) => Size::new(Some(width), Some(width / ratio)),
            (None, Some(height)) => Size::new(Some(height * ratio), Some(height)),
            _ => self,
        }
    }
}

mod sealed {
    use crate::{LayoutScalar, Size};

    pub(crate) trait OptionalSizeMax {}

    impl<S: LayoutScalar> OptionalSizeMax for Size<Option<S>> {}

    pub(crate) trait OptionalMinimumSizeFloor {}

    impl<S: LayoutScalar> OptionalMinimumSizeFloor for Size<S> {}
}

pub(crate) trait OptionalSizeMaxExt: sealed::OptionalSizeMax {
    fn max_optional(self, min: Self) -> Self;
}

impl<S: LayoutScalar> OptionalSizeMaxExt for Size<Option<S>> {
    fn max_optional(self, min: Self) -> Self {
        Size::new(
            self.width
                .zip(min.width)
                .map(|(value, min)| value.max(min))
                .or(self.width),
            self.height
                .zip(min.height)
                .map(|(value, min)| value.max(min))
                .or(self.height),
        )
    }
}

pub(crate) trait OptionalMinimumSizeFloorExt: sealed::OptionalMinimumSizeFloor {
    type Scalar: LayoutScalar;

    fn max_optional(self, min: Size<Option<Self::Scalar>>) -> Self;
}

impl<S: LayoutScalar> OptionalMinimumSizeFloorExt for Size<S> {
    type Scalar = S;

    fn max_optional(self, min: Size<Option<S>>) -> Self {
        Size::new(
            min.width.map_or(self.width, |min| self.width.max(min)),
            min.height.map_or(self.height, |min| self.height.max(min)),
        )
    }
}

pub(crate) trait UncheckedOptionalSizeSubExt {
    type Scalar: LayoutScalar;

    fn sub_optional_unchecked(self, amount: Size<Self::Scalar>) -> Self;
}

impl<S: LayoutScalar> UncheckedOptionalSizeSubExt for Size<Option<S>> {
    type Scalar = S;

    fn sub_optional_unchecked(self, amount: Size<S>) -> Self {
        Size::new(
            self.width.map(|width| width - amount.width),
            self.height.map(|height| height - amount.height),
        )
    }
}

pub(crate) trait MaxBeforeMinScalarClampExt {
    fn clamp_max_before_min_optional(self, min: Option<Self>, max: Option<Self>) -> Self
    where
        Self: Sized;
}

impl<S: LayoutScalar> MaxBeforeMinScalarClampExt for S {
    fn clamp_max_before_min_optional(self, min: Option<Self>, max: Option<Self>) -> Self {
        let value = max.map_or(self, |max| self.min(max));
        min.map_or(value, |min| value.max(min))
    }
}

pub(crate) trait MaxBeforeMinSizeClampExt {
    type Scalar: LayoutScalar;

    fn clamp_max_before_min_optional(
        self,
        min: Size<Option<Self::Scalar>>,
        max: Size<Option<Self::Scalar>>,
    ) -> Self;
}

impl<S: LayoutScalar> MaxBeforeMinSizeClampExt for Size<S> {
    type Scalar = S;

    fn clamp_max_before_min_optional(self, min: Size<Option<S>>, max: Size<Option<S>>) -> Self {
        Size::new(
            self.width
                .clamp_max_before_min_optional(min.width, max.width),
            self.height
                .clamp_max_before_min_optional(min.height, max.height),
        )
    }
}

pub(crate) trait MaxBeforeMinOptionalSizeClampExt {
    type Scalar: LayoutScalar;

    fn clamp_max_before_min_optional(self, min: Self, max: Self) -> Self;
}

impl<S: LayoutScalar> MaxBeforeMinOptionalSizeClampExt for Size<Option<S>> {
    type Scalar = S;

    fn clamp_max_before_min_optional(self, min: Self, max: Self) -> Self {
        Size::new(
            self.width
                .map(|value| value.clamp_max_before_min_optional(min.width, max.width)),
            self.height
                .map(|value| value.clamp_max_before_min_optional(min.height, max.height)),
        )
    }
}

#[cfg(test)]
type ResolutionOrZeroFn<S> =
    fn(crate::LengthResolutionOf<S>) -> Result<S, crate::LengthResolutionStatus<S>>;

#[cfg(test)]
type ResolutionOptionalFn<S> =
    fn(crate::LengthResolutionOf<S>) -> Result<Option<S>, crate::LengthResolutionStatus<S>>;

#[cfg(test)]
pub(crate) fn assert_fri06_c13_t06_resolution_policy<S: LayoutScalar>(
    resolution_or_zero: ResolutionOrZeroFn<S>,
    resolution_optional: ResolutionOptionalFn<S>,
) {
    use crate::{LengthResolutionOf, LengthResolutionStatus};

    let resolved = S::from_f64(7.5);
    let negative = S::from_f64(-3.25);
    let invalid = LengthResolutionStatus::InvalidNumeric { value: S::INFINITY };

    assert_eq!(
        resolution_or_zero(LengthResolutionOf::definite(resolved, true)),
        Ok(resolved)
    );
    assert_eq!(
        resolution_or_zero(LengthResolutionOf::definite(S::ZERO, false)),
        Ok(S::ZERO)
    );
    assert_eq!(
        resolution_or_zero(LengthResolutionOf::definite(negative, false)),
        Ok(negative)
    );
    assert_eq!(
        resolution_or_zero(LengthResolutionOf::unresolved(true)),
        Ok(S::ZERO)
    );
    assert_eq!(
        resolution_or_zero(LengthResolutionOf::non_numeric()),
        Ok(S::ZERO)
    );
    assert_eq!(
        resolution_or_zero(LengthResolutionOf::invalid_numeric(S::INFINITY, true)),
        Err(invalid)
    );

    assert_eq!(
        resolution_optional(LengthResolutionOf::definite(resolved, true)),
        Ok(Some(resolved))
    );
    assert_eq!(
        resolution_optional(LengthResolutionOf::definite(S::ZERO, false)),
        Ok(Some(S::ZERO))
    );
    assert_eq!(
        resolution_optional(LengthResolutionOf::definite(negative, false)),
        Ok(Some(negative))
    );
    assert_eq!(
        resolution_optional(LengthResolutionOf::unresolved(true)),
        Ok(None)
    );
    assert_eq!(
        resolution_optional(LengthResolutionOf::non_numeric()),
        Ok(None)
    );
    assert_eq!(
        resolution_optional(LengthResolutionOf::invalid_numeric(S::INFINITY, true)),
        Err(invalid)
    );
}

#[cfg(test)]
mod tests {
    use super::{OptionalMinimumSizeFloorExt, OptionalSizeMaxExt};
    use crate::{LayoutScalar, Size};

    fn assert_optional_size_operations<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(None, Some(scalar(12.0))).max_optional(Size::new(None, None)),
            Size::new(None, Some(scalar(12.0)))
        );
        assert_eq!(
            Size::new(Some(scalar(4.0)), Some(scalar(12.0)))
                .max_optional(Size::new(Some(scalar(9.0)), Some(scalar(3.0)))),
            Size::new(Some(scalar(9.0)), Some(scalar(12.0)))
        );
        assert_eq!(
            Size::new(scalar(4.0), scalar(12.0)).max_optional(Size::new(None, Some(scalar(15.0)))),
            Size::new(scalar(4.0), scalar(15.0))
        );
    }

    #[test]
    fn optional_size_operations_preserve_f32() {
        assert_optional_size_operations::<f32>();
    }

    #[test]
    fn optional_size_operations_preserve_f64() {
        assert_optional_size_operations::<f64>();
    }
}

#[cfg(test)]
#[path = "layout_math/fallback_tests.rs"]
mod fallback_tests;
