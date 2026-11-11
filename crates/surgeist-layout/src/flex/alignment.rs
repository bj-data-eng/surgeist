use super::distribution_number::DistributionNumber;
use super::flexible_lengths::{clamp_cross_size, flex_main_size, flex_used_space};
use super::items::{FinalFlexItem, ResolvedFlexItem};
use super::lines::FlexLine;
use super::{
    AlignContent, AlignItems, BaselinesOf, ComputeOutputOf, Constants, Direction, FlexAxes,
    LayoutScalar, Point, Size,
};
use crate::WritingMode;
use crate::geometry::{FlowAxes, LogicalAxis, PhysicalAxis, PhysicalSide};
use crate::layout_math::OptionalSizeExt;
use crate::output::PhysicalBaseline;
use crate::{Edges, LayoutResultOf, LengthResolutionStatus};
use core::cmp::Ordering;
use core::ops::{Add, Div};

#[derive(Clone, Copy, Debug)]
pub(super) struct FlexItemBaseline<S: LayoutScalar> {
    flow_axes: FlowAxes,
    measured: BaselinesOf<S>,
}

impl<S: LayoutScalar> FlexItemBaseline<S> {
    pub(super) fn from_output(output: ComputeOutputOf<S>, flow_axes: FlowAxes) -> Self {
        Self {
            flow_axes,
            measured: output.baselines(),
        }
    }

    pub(super) fn refresh(&mut self, output: ComputeOutputOf<S>) {
        self.measured = output.baselines();
    }

    fn cross_flow_axes(self, axes: FlexAxes) -> FlowAxes {
        if self.flow_axes.block_axis() == axes.cross_physical_axis() {
            return self.flow_axes;
        }
        if axes.main_logical_axis() == LogicalAxis::Inline {
            return axes.flow_axes();
        }
        // In a column, the container's own writing mode is also incompatible
        // with the cross axis. Orient the synthesized port along that axis.
        let direction = axes.flow_direction();
        let writing_mode = match (axes.cross_physical_axis(), direction) {
            (PhysicalAxis::Vertical, _) => WritingMode::HorizontalTb,
            (PhysicalAxis::Horizontal, Direction::Ltr) => WritingMode::VerticalLr,
            (PhysicalAxis::Horizontal, Direction::Rtl) => WritingMode::VerticalRl,
        };
        FlowAxes::new(writing_mode, direction)
    }

    fn physical(self, size: Size<S>, port: BaselinePort, axes: FlexAxes) -> PhysicalBaseline<S> {
        let flow_axes = self.cross_flow_axes(axes);
        match port {
            BaselinePort::First => self
                .measured
                .first_or_synthesize_block_baseline(flow_axes, size),
            BaselinePort::Last => self
                .measured
                .last_or_synthesize_block_baseline(flow_axes, size),
        }
    }

    fn sharing_group(
        self,
        preference: Option<BaselinePort>,
        axes: FlexAxes,
    ) -> Option<(BaselinePort, PhysicalSide)> {
        preference.map(|port| (port, port.fallback_side(self.cross_flow_axes(axes))))
    }

    fn translated_start_item(
        self,
        size: Size<S>,
        location: Point<S>,
        axes: FlexAxes,
        sharing_port: Option<BaselinePort>,
    ) -> Point<Option<S>> {
        let baseline = if let Some(port) = sharing_port {
            self.physical(size, port, axes)
        } else {
            self.measured
                .first_or_synthesize_block_baseline(axes.flow_axes(), size)
        };
        baseline.translated(location)
    }

    fn translated_end_item(
        self,
        size: Size<S>,
        location: Point<S>,
        axes: FlexAxes,
        sharing_port: Option<BaselinePort>,
    ) -> Point<Option<S>> {
        if let Some(port) = sharing_port {
            return self.physical(size, port, axes).translated(location);
        }
        self.measured
            .last_block_baseline(axes.flow_axes())
            .unwrap_or_else(|| {
                self.measured
                    .first_or_synthesize_block_baseline(axes.flow_axes(), size)
            })
            .translated(location)
    }
}

pub(super) fn align_lines_on_cross_axis<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    lines: &mut [FlexLine<S>],
    constants: &Constants<S>,
) {
    let Some(container_cross_size) = constants.axes.cross_size(constants.node_inner_size) else {
        return;
    };
    let line_count = lines.len();
    let cross_gap = constants.axes.cross_size(constants.gap);
    let used_cross_size = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + cross_gap * S::from_usize(line_count.saturating_sub(1));
    let free_space = container_cross_size - used_cross_size;
    let align_content = alignment_fallback(free_space, line_count, constants.align_content);
    let mut cross_cursor = alignment_offset(
        free_space,
        line_count,
        cross_gap,
        align_content,
        constants.axes.cross_is_reversed(),
        true,
    );
    for (index, line) in lines.iter_mut().enumerate() {
        if index > 0 {
            cross_cursor = cross_cursor
                + alignment_offset(
                    free_space,
                    line_count,
                    cross_gap,
                    align_content,
                    constants.axes.cross_is_reversed(),
                    false,
                );
        }
        let delta = cross_cursor - line.offset_cross;
        line.offset_cross = cross_cursor;
        for item in &mut items[line.start..line.end] {
            item.offset_cross = item.offset_cross + delta;
        }
        cross_cursor = cross_cursor + line.cross_size;
    }
}

pub(super) fn stretch_lines_on_cross_axis<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    lines: &mut [FlexLine<S>],
    constants: &Constants<S>,
) {
    if constants.align_content != AlignContent::Stretch {
        return;
    }

    let Some(container_cross_size) = constants.axes.cross_size(constants.node_inner_size) else {
        return;
    };
    let cross_gap = constants.axes.cross_size(constants.gap);
    let used_cross_size = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + cross_gap * S::from_usize(lines.len().saturating_sub(1));
    if used_cross_size >= container_cross_size || lines.is_empty() {
        return;
    }

    let addition = (container_cross_size - used_cross_size) / S::from_usize(lines.len());
    for line in lines {
        line.cross_size = line.cross_size + addition;
        align_items_on_cross_axis(
            &mut items[line.start..line.end],
            line.cross_size,
            line.offset_cross,
            constants,
        );
    }
}

pub(super) fn resolve_main_axis_auto_margins<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> Result<(), S> {
    for item in &mut *items {
        if item.margin_main_start_is_auto(constants) {
            item.set_margin_main_start(constants, S::ZERO);
        }
        if item.margin_main_end_is_auto(constants) {
            item.set_margin_main_end(constants, S::ZERO);
        }
    }

    let free_space = line_free_space(items, constants)?;
    if free_space.sign() != Ordering::Greater {
        return Ok(());
    }

    let auto_margin_count = items
        .iter()
        .map(|item| {
            usize::from(item.margin_main_start_is_auto(constants))
                + usize::from(item.margin_main_end_is_auto(constants))
        })
        .sum::<usize>();
    if auto_margin_count == 0 {
        return Ok(());
    }

    let margin = (free_space / DistributionNumber::finite(S::from_usize(auto_margin_count))?)
        .checked_narrow()?;
    for item in items {
        if item.margin_main_start_is_auto(constants) {
            item.set_margin_main_start(constants, margin);
        }
        if item.margin_main_end_is_auto(constants) {
            item.set_margin_main_end(constants, margin);
        }
    }
    Ok(())
}

pub(super) fn align_items_on_cross_axis<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    line_cross_size: S,
    line_cross_offset: S,
    constants: &Constants<S>,
) {
    let axes = constants.axes;
    let (start_group, end_group) = line_baseline_extents(items.iter(), axes);
    for item in items {
        if let Some((anchor, baseline_offset)) = item.cross_baseline(axes) {
            let line_baseline = if anchor == axes.cross_start_side {
                start_group
                    .expect("sharing item contributes to its start group")
                    .before
            } else {
                let group = end_group.expect("sharing item contributes to its end group");
                // Safe fallback keeps an overflowing group at flex cross-start.
                (line_cross_size - group.after).max(group.before)
            };
            item.offset_cross = line_cross_offset + line_baseline - baseline_offset;
            continue;
        }
        resolve_cross_axis_auto_margins(item, line_cross_size, constants);
        let outer_cross_size = constants.axes.cross_size(item.target_size)
            + constants.axes.cross_edge_sum(item.margin);
        let free_space = line_cross_size - outer_cross_size;
        let has_auto_cross_margin = item.has_auto_cross_margin(axes);
        if item.stretches_cross(axes) {
            let stretched_cross_size = S::max(
                S::ZERO,
                line_cross_size - constants.axes.cross_edge_sum(item.margin),
            );
            let stretched_cross_size = clamp_cross_size(item, stretched_cross_size);
            item.target_size = constants
                .axes
                .with_cross_size(item.target_size, stretched_cross_size);
        }
        let alignment_offset = if has_auto_cross_margin {
            S::ZERO
        } else {
            match item.align_self.safe_fallback(free_space) {
                AlignItems::Start => {
                    if constants.axes.cross_is_reversed() {
                        free_space
                    } else {
                        S::ZERO
                    }
                }
                AlignItems::End => {
                    if constants.axes.cross_is_reversed() {
                        S::ZERO
                    } else {
                        free_space
                    }
                }
                AlignItems::FlexStart | AlignItems::Stretch => S::ZERO,
                AlignItems::Center => free_space / S::from_f64(2.0),
                AlignItems::FlexEnd => free_space,
                AlignItems::Baseline | AlignItems::LastBaseline => {
                    unreachable!("eligible baseline alignment uses its sharing group")
                }
                AlignItems::Safe(_) | AlignItems::Unsafe(_) => {
                    unreachable!("safe_fallback returns unqualified item alignment")
                }
            }
        };
        item.offset_cross =
            line_cross_offset + axes.cross_start_edge(item.margin) + alignment_offset;
    }
}

#[derive(Clone, Copy)]
struct BaselineExtents<S> {
    before: S,
    after: S,
}

fn line_baseline_extents<'a, Node: 'a, S: LayoutScalar>(
    items: impl Iterator<Item = &'a ResolvedFlexItem<Node, S>>,
    axes: FlexAxes,
) -> (Option<BaselineExtents<S>>, Option<BaselineExtents<S>>) {
    let mut start = None;
    let mut end = None;
    for item in items {
        let Some((anchor, baseline_offset)) = item.cross_baseline(axes) else {
            continue;
        };
        let before = axes.cross_start_edge(item.margin) + baseline_offset;
        let after =
            axes.cross_end_edge(item.margin) + axes.cross_size(item.target_size) - baseline_offset;
        let group = if anchor == axes.cross_start_side {
            &mut start
        } else {
            &mut end
        };
        *group = Some(group.map_or(
            BaselineExtents { before, after },
            |current: BaselineExtents<S>| BaselineExtents {
                before: current.before.max(before),
                after: current.after.max(after),
            },
        ));
    }
    (start, end)
}

pub(super) fn line_cross_size<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    let axes = constants.axes;
    // Flex line sizing uses the baseline extents of axis-compatible items.
    // Other items contribute their outer size even if they align by synthesis.
    let eligible = items
        .iter()
        .filter(|item| item.uses_native_cross_baseline(axes));
    let (start, end) = line_baseline_extents(eligible, axes);
    let baseline_extent = [start, end]
        .into_iter()
        .flatten()
        .map(|extent| extent.before + extent.after)
        .fold(S::ZERO, S::max);
    items
        .iter()
        .filter(|item| !item.uses_native_cross_baseline(axes))
        .map(|item| axes.cross_size(item.target_size) + axes.cross_edge_sum(item.margin))
        .fold(baseline_extent, S::max)
}

#[derive(Clone, Copy, Debug)]
pub(super) enum BaselinePort {
    First,
    Last,
}

impl BaselinePort {
    pub(super) fn for_alignment(alignment: AlignItems) -> Option<Self> {
        match alignment {
            AlignItems::Baseline => Some(Self::First),
            AlignItems::LastBaseline => Some(Self::Last),
            _ => None,
        }
    }

    fn fallback_side(self, flow_axes: FlowAxes) -> PhysicalSide {
        match self {
            Self::First => flow_axes.block_start(),
            Self::Last => flow_axes.block_end(),
        }
    }
}

fn baseline_export_item<'a, Item, S: LayoutScalar>(
    items: &'a [Item],
    lines: &[FlexLine<S>],
    axes: FlexAxes,
    port: BaselinePort,
    sharing_group: impl Fn(&Item) -> Option<(BaselinePort, PhysicalSide)>,
) -> Option<(&'a Item, Option<BaselinePort>)> {
    let first = matches!(port, BaselinePort::First);
    // Line and item slices remain in collection order, before axis reversal.
    let from_first_line = first != axes.cross_is_reversed();
    let mut lines = lines.iter();
    let line = if from_first_line {
        lines.next()?
    } else {
        lines.next_back()?
    };
    let line_items = &items[line.start..line.end];
    let from_start = first != axes.main_is_reversed();
    if axes.main_logical_axis() == LogicalAxis::Inline {
        let preferred_edge = port.fallback_side(axes.flow_axes());
        let mut fallback_group = None;
        let mut consider = |item: &'a Item| {
            let (item_port, edge) = sharing_group(item)?;
            fallback_group.get_or_insert((item, item_port));
            (edge == preferred_edge).then_some((item, item_port))
        };
        let sharing = if from_start {
            line_items.iter().find_map(&mut consider)
        } else {
            line_items.iter().rev().find_map(&mut consider)
        };
        if let Some((item, port)) = sharing.or(fallback_group) {
            return Some((item, Some(port)));
        }
    }
    // An edge line can retain a strut without containing a rendered item.
    let fallback_items = if line_items.is_empty() {
        let line = if from_first_line {
            lines.find(|line| line.start < line.end)?
        } else {
            lines.rfind(|line| line.start < line.end)?
        };
        &items[line.start..line.end]
    } else {
        line_items
    };
    let fallback = if from_start {
        fallback_items.first()
    } else {
        fallback_items.last()
    };
    fallback.map(|item| (item, None))
}

pub(super) fn resolved_vertical_baselines<Node: Copy, S: LayoutScalar, M>(
    items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
    mut measure: impl FnMut(
        &ResolvedFlexItem<Node, S>,
    ) -> LayoutResultOf<Node, Option<ComputeOutputOf<S>>, S, M>,
) -> LayoutResultOf<Node, BaselinesOf<S>, S, M> {
    let axes = constants.axes;
    let mut last_measurement = None;
    let mut baseline_for_port = |port| {
        let Some((item, sharing_port)) = baseline_export_item(items, lines, axes, port, |item| {
            item.baseline
                .sharing_group(item.baseline_preference(axes), axes)
        }) else {
            return Ok(Point::NONE);
        };
        let measured = match last_measurement {
            Some((index, output)) if index == item.source_index => output,
            _ => {
                let output = measure(item)?;
                last_measurement = Some((item.source_index, output));
                output
            }
        };
        let mut baseline = item.baseline;
        let size = if let Some(output) = measured {
            baseline.refresh(output);
            output.size
        } else {
            item.target_size
        };
        let location = axes.point_from_main_cross(
            item.normal_main_location(constants, size)
                .map_err(|value| {
                    crate::error::sizing_resolution_error(
                        item.node,
                        LengthResolutionStatus::InvalidNumeric { value }.into(),
                    )
                })?,
            item.normal_cross_location(constants, size),
        );
        Ok(match port {
            BaselinePort::First => {
                baseline.translated_start_item(size, location, axes, sharing_port)
            }
            BaselinePort::Last => baseline.translated_end_item(size, location, axes, sharing_port),
        })
    };
    Ok(BaselinesOf {
        first: baseline_for_port(BaselinePort::First)?,
        last: baseline_for_port(BaselinePort::Last)?,
    })
}

pub(super) fn first_final_vertical_baseline<Node, S: LayoutScalar>(
    items: &[FinalFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<Point<Option<S>>> {
    let (item, sharing_port) =
        baseline_export_item(items, lines, constants.axes, BaselinePort::First, |item| {
            item.baseline
                .sharing_group(item.baseline_preference, constants.axes)
        })?;
    Some(item.baseline.translated_start_item(
        item.output.size,
        item.normal_location,
        constants.axes,
        sharing_port,
    ))
}

pub(super) fn last_final_vertical_baseline<Node, S: LayoutScalar>(
    items: &[FinalFlexItem<Node, S>],
    lines: &[FlexLine<S>],
    constants: &Constants<S>,
) -> Option<Point<Option<S>>> {
    let (item, sharing_port) =
        baseline_export_item(items, lines, constants.axes, BaselinePort::Last, |item| {
            item.baseline
                .sharing_group(item.baseline_preference, constants.axes)
        })?;
    Some(item.baseline.translated_end_item(
        item.output.size,
        item.normal_location,
        constants.axes,
        sharing_port,
    ))
}

fn resolve_cross_axis_auto_margins<Node, S: LayoutScalar>(
    item: &mut ResolvedFlexItem<Node, S>,
    line_cross_size: S,
    constants: &Constants<S>,
) {
    let auto_start = constants.axes.normal_cross_start_edge(item.margin_is_auto);
    let auto_end = constants.axes.normal_cross_end_edge(item.margin_is_auto);
    if !auto_start && !auto_end {
        return;
    }
    if auto_start {
        constants
            .axes
            .set_normal_cross_start_edge(&mut item.margin, S::ZERO);
    }
    if auto_end {
        constants
            .axes
            .set_normal_cross_end_edge(&mut item.margin, S::ZERO);
    }

    let free_space = line_cross_size
        - constants.axes.cross_size(item.target_size)
        - constants.axes.cross_edge_sum(item.margin);
    if free_space <= S::ZERO {
        let overflow_end = line_cross_size
            - constants.axes.cross_size(item.target_size)
            - constants.axes.normal_cross_start_edge(item.margin);
        constants
            .axes
            .set_normal_cross_end_edge(&mut item.margin, overflow_end);
        return;
    }

    if auto_start && auto_end {
        let margin = free_space / S::from_f64(2.0);
        constants
            .axes
            .set_normal_cross_start_edge(&mut item.margin, margin);
        constants
            .axes
            .set_normal_cross_end_edge(&mut item.margin, margin);
    } else if auto_start {
        constants
            .axes
            .set_normal_cross_start_edge(&mut item.margin, free_space);
    } else if auto_end {
        constants
            .axes
            .set_normal_cross_end_edge(&mut item.margin, free_space);
    }
}

pub(super) fn line_free_space<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> Result<DistributionNumber<S>, S> {
    let Some(container_main_size) = flex_main_size(constants) else {
        return Ok(DistributionNumber::ZERO);
    };
    Ok(DistributionNumber::finite(container_main_size)? - flex_used_space(items, constants, None)?)
}

pub(super) fn alignment_fallback<S: LayoutScalar>(
    free_space: S,
    item_count: usize,
    alignment_mode: AlignContent,
) -> AlignContent {
    alignment_fallback_from_sign(free_space.partial_cmp(&S::ZERO), item_count, alignment_mode)
}

pub(super) fn alignment_fallback_from_sign(
    sign: Option<Ordering>,
    item_count: usize,
    alignment_mode: AlignContent,
) -> AlignContent {
    let alignment_mode = match alignment_mode {
        AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter
            if sign == Some(Ordering::Less) =>
        {
            AlignContent::FlexStart
        }
        alignment => alignment.unsafe_position(),
    };
    if item_count > 1 && sign == Some(Ordering::Greater) {
        return alignment_mode;
    }

    match alignment_mode {
        AlignContent::Stretch
        | AlignContent::SpaceBetween
        | AlignContent::SpaceAround
        | AlignContent::SpaceEvenly
            if matches!(sign, Some(Ordering::Less | Ordering::Equal)) =>
        {
            AlignContent::FlexStart
        }
        AlignContent::Stretch | AlignContent::SpaceBetween => AlignContent::FlexStart,
        AlignContent::SpaceAround | AlignContent::SpaceEvenly => AlignContent::Center,
        mode => mode,
    }
}

pub(super) fn alignment_offset<S: LayoutScalar>(
    free_space: S,
    item_count: usize,
    gap: S,
    alignment_mode: AlignContent,
    layout_is_flex_reversed: bool,
    is_first: bool,
) -> S {
    let sign = free_space.partial_cmp(&S::ZERO);
    let free_space = if is_first {
        free_space
    } else {
        free_space.max(S::ZERO)
    };
    alignment_offset_for(
        (free_space, sign),
        item_count,
        gap,
        alignment_mode,
        layout_is_flex_reversed,
        is_first,
        core::convert::identity,
    )
}

pub(super) fn main_alignment_offset<S: LayoutScalar>(
    free_space: DistributionNumber<S>,
    item_count: usize,
    gap: DistributionNumber<S>,
    alignment_mode: AlignContent,
    layout_is_flex_reversed: bool,
    is_first: bool,
) -> DistributionNumber<S> {
    let sign = Some(free_space.sign());
    let free_space = if is_first {
        free_space
    } else {
        free_space.max(DistributionNumber::ZERO)
    };
    alignment_offset_for(
        (free_space, sign),
        item_count,
        gap,
        alignment_mode,
        layout_is_flex_reversed,
        is_first,
        |value| {
            DistributionNumber::finite(value).expect("alignment divisors fit either scalar lane")
        },
    )
}

fn alignment_offset_for<S: LayoutScalar, T: Copy + Add<Output = T> + Div<Output = T>>(
    (free_space, sign): (T, Option<Ordering>),
    item_count: usize,
    gap: T,
    alignment_mode: AlignContent,
    layout_is_flex_reversed: bool,
    is_first: bool,
    from_scalar: impl Fn(S) -> T,
) -> T {
    let zero = from_scalar(S::ZERO);
    let two = from_scalar(S::from_f64(2.0));
    if is_first {
        match alignment_mode {
            AlignContent::Start => {
                if layout_is_flex_reversed {
                    free_space
                } else {
                    zero
                }
            }
            AlignContent::FlexStart => zero,
            AlignContent::End => {
                if layout_is_flex_reversed {
                    zero
                } else {
                    free_space
                }
            }
            AlignContent::FlexEnd => free_space,
            AlignContent::Center => free_space / two,
            AlignContent::Stretch | AlignContent::SpaceBetween => zero,
            AlignContent::SpaceAround => {
                if matches!(sign, Some(Ordering::Equal | Ordering::Greater)) {
                    (free_space / from_scalar(S::from_usize(item_count))) / two
                } else {
                    free_space / two
                }
            }
            AlignContent::SpaceEvenly => {
                if matches!(sign, Some(Ordering::Equal | Ordering::Greater)) {
                    free_space / from_scalar(S::from_usize(item_count + 1))
                } else {
                    free_space / two
                }
            }
            AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
                unreachable!("safe_fallback returns unsafe content alignment")
            }
        }
    } else {
        gap + match alignment_mode {
            AlignContent::SpaceBetween => free_space / from_scalar(S::from_usize(item_count - 1)),
            AlignContent::SpaceAround => free_space / from_scalar(S::from_usize(item_count)),
            AlignContent::SpaceEvenly => free_space / from_scalar(S::from_usize(item_count + 1)),
            AlignContent::Start
            | AlignContent::FlexStart
            | AlignContent::End
            | AlignContent::FlexEnd
            | AlignContent::Center
            | AlignContent::Stretch => zero,
            AlignContent::SafeEnd | AlignContent::SafeFlexEnd | AlignContent::SafeCenter => {
                unreachable!("safe_fallback returns unsafe content alignment")
            }
        }
    }
}

impl<Node, S: LayoutScalar> ResolvedFlexItem<Node, S> {
    fn has_auto_cross_margin(&self, axes: FlexAxes) -> bool {
        axes.cross_start_edge(self.margin_is_auto) || axes.cross_end_edge(self.margin_is_auto)
    }

    pub(super) fn stretches_cross(&self, axes: FlexAxes) -> bool {
        self.align_self == AlignItems::Stretch
            && self.cross_size_is_auto
            && !self.has_auto_cross_margin(axes)
    }

    pub(super) fn baseline_preference(&self, axes: FlexAxes) -> Option<BaselinePort> {
        BaselinePort::for_alignment(self.align_self).filter(|_| !self.has_auto_cross_margin(axes))
    }

    pub(super) fn uses_native_cross_baseline(&self, axes: FlexAxes) -> bool {
        self.baseline_preference(axes).is_some()
            && self.baseline.flow_axes.block_axis() == axes.cross_physical_axis()
    }

    fn cross_baseline(&self, axes: FlexAxes) -> Option<(PhysicalSide, S)> {
        let (port, anchor) = self
            .baseline
            .sharing_group(self.baseline_preference(axes), axes)?;
        let coordinate = self
            .baseline
            .physical(self.target_size, port, axes)
            .coordinate();
        let offset = if axes.cross_progression.is_decreasing() {
            axes.cross_size(self.target_size) - coordinate
        } else {
            coordinate
        };
        Some((anchor, offset))
    }

    pub(super) fn margin_main_start(&self, constants: &Constants<S>) -> S {
        constants.axes.main_start_edge(self.margin)
    }

    fn margin_main_start_is_auto(&self, constants: &Constants<S>) -> bool {
        constants.axes.main_start_edge(self.margin_is_auto)
    }

    fn margin_main_end_is_auto(&self, constants: &Constants<S>) -> bool {
        constants.axes.main_end_edge(self.margin_is_auto)
    }

    fn set_margin_main_start(&mut self, constants: &Constants<S>, value: S) {
        constants.axes.set_main_start_edge(&mut self.margin, value);
    }

    fn set_margin_main_end(&mut self, constants: &Constants<S>, value: S) {
        constants.axes.set_main_end_edge(&mut self.margin, value);
    }

    pub(super) fn final_main_location(
        &self,
        constants: &Constants<S>,
        output_size: Size<S>,
        inset: Edges<Option<S>>,
    ) -> Result<S, S> {
        let normal_offset = constants
            .axes
            .normal_main_start_edge(inset)
            .or_else(|| constants.axes.normal_main_end_edge(inset).map(|end| -end))
            .unwrap_or(S::ZERO);
        self.main_location(
            constants,
            output_size,
            constants.axes.main_offset_from_normal_flow(normal_offset),
        )
    }

    pub(super) fn normal_main_location(
        &self,
        constants: &Constants<S>,
        output_size: Size<S>,
    ) -> Result<S, S> {
        self.main_location(constants, output_size, S::ZERO)
    }

    fn main_location(
        &self,
        constants: &Constants<S>,
        output_size: Size<S>,
        relative: S,
    ) -> Result<S, S> {
        let container = constants
            .node_outer_size
            .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
        let wide = DistributionNumber::finite;
        constants
            .axes
            .main_position_from_start(
                Size::splat(wide(constants.axes.main_size(container))?),
                wide(constants.axes.main_start_edge(constants.content_box_inset))?,
                self.offset_main,
                wide(constants.axes.main_size(output_size))?,
                wide(relative)?,
            )
            .checked_narrow()
    }

    pub(super) fn final_cross_location(
        &self,
        constants: &Constants<S>,
        output_size: Size<S>,
        inset: Edges<Option<S>>,
    ) -> Result<S, S> {
        let normal_offset = constants
            .axes
            .normal_cross_start_edge(inset)
            .or_else(|| constants.axes.normal_cross_end_edge(inset).map(|end| -end))
            .unwrap_or(S::ZERO);
        let container = constants
            .node_outer_size
            .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
        let wide = DistributionNumber::finite;
        constants
            .axes
            .cross_position_from_start(
                Size::splat(wide(constants.axes.cross_size(container))?),
                wide(constants.axes.cross_start_edge(constants.content_box_inset))?,
                wide(self.offset_cross)?,
                wide(constants.axes.cross_size(output_size))?,
                wide(constants.axes.cross_offset_from_normal_flow(normal_offset))?,
            )
            .checked_narrow()
    }

    pub(super) fn normal_cross_location(
        &self,
        constants: &Constants<S>,
        output_size: Size<S>,
    ) -> S {
        let container = constants
            .node_outer_size
            .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
        constants.axes.cross_position_from_start(
            container,
            constants.axes.cross_start_edge(constants.content_box_inset),
            self.offset_cross,
            constants.axes.cross_size(output_size),
            S::ZERO,
        )
    }
}

#[cfg(test)]
mod final_baseline_selection_tests {
    use super::*;
    use crate::{Edges, FlexDirection, FlexWrap, LengthOf, NodeInputOf};

    fn default_flow_axes<S: LayoutScalar>() -> FlowAxes {
        let style = NodeInputOf::<S>::default();
        FlowAxes::new(style.writing_mode, style.direction)
    }

    fn final_item<S: LayoutScalar>(
        align_self: AlignItems,
        location_y: f64,
        baseline_y: f64,
    ) -> FinalFlexItem<(), S> {
        let size = Size::new(S::from_f64(10.0), S::from_f64(10.0));
        let output = ComputeOutputOf::from_sizes_and_baselines(
            size,
            size,
            BaselinesOf::first(Point::new(None, Some(S::from_f64(baseline_y)))),
        );
        FinalFlexItem {
            _node: core::marker::PhantomData,
            source_index: crate::SourceIndex::ZERO,
            output,
            margin: Edges::ZERO,
            baseline_preference: BaselinePort::for_alignment(align_self),
            baseline: FlexItemBaseline::from_output(output, default_flow_axes::<S>()),
            location: Point::new(S::ZERO, S::from_f64(location_y)),
            normal_location: Point::new(S::ZERO, S::from_f64(location_y)),
        }
    }

    fn constants<S: LayoutScalar>(flex_direction: FlexDirection) -> Constants<S> {
        let flow_axes = default_flow_axes::<S>();
        Constants {
            flow_axes,
            axes: super::super::FlexAxes::new(flow_axes, flex_direction, FlexWrap::NoWrap),
            node_outer_size: Size::NONE,
            percentage_parent_size: Size::NONE,
            physical_baseline_demand: false,
            node_inner_size: Size::NONE,
            preferred_inline_phase: crate::flex::PreferredInlinePhase::Ordinary,
            cross_content_offer: None,
            min_outer_size: Size::NONE,
            max_outer_size: Size::NONE,
            max_inner_size: Size::NONE,
            border: Edges::ZERO,
            padding: Edges::ZERO,
            padding_border_size: Size::ZERO,
            scrollport_inset: Edges::ZERO,
            non_gutter_box_inset: Edges::ZERO,
            content_box_inset: Edges::ZERO,
            settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState::INITIAL,
            gap: Size::ZERO,
            gap_input: Size::splat(LengthOf::ZERO),
            gap_percentage_basis: Size::NONE,
            align_items: AlignItems::Stretch,
            authored_align_content: None,
            align_content: AlignContent::Stretch,
            authored_justify_content: None,
            justify_content: AlignContent::FlexStart,
            wraps: false,
            sizing_request: crate::SizingRequestOf::new(
                Size::splat(crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::Normal,
                )),
                crate::OrthogonalFallbackOf::new(
                    crate::Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            positioned_fallback_limits: Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            available_main: crate::AxisSizingRequestOf::new(
                crate::AvailableSpaceOf::Indefinite,
                crate::SizingOperation::Normal,
            ),
        }
    }

    fn assert_final_baselines_follow_main_logical_axis<S: LayoutScalar>() {
        let items = [
            final_item::<S>(AlignItems::Stretch, 10.0, 1.0),
            final_item::<S>(AlignItems::Baseline, 20.0, 2.0),
            final_item::<S>(AlignItems::Stretch, 30.0, 3.0),
        ];
        let lines = [FlexLine {
            start: 0,
            end: items.len(),
            strut_floor: S::ZERO,
            contains_collapsed_slot: false,
            main_size: DistributionNumber::ZERO,
            main_overflows: false,
            max_item_cross_size: S::ZERO,
            cross_size: S::ZERO,
            offset_cross: S::ZERO,
        }];
        for (flex_direction, first, last) in [
            (FlexDirection::Row, 22.0, 22.0),
            (FlexDirection::Column, 11.0, 33.0),
        ] {
            let constants = constants::<S>(flex_direction);
            assert_eq!(
                first_final_vertical_baseline(&items, &lines, &constants),
                Some(Point::new(None, Some(S::from_f64(first)))),
                "the first final baseline follows the resolved main logical axis"
            );
            assert_eq!(
                last_final_vertical_baseline(&items, &lines, &constants),
                Some(Point::new(None, Some(S::from_f64(last)))),
                "the last final baseline follows the resolved main logical axis"
            );
        }
    }

    #[test]
    fn logical_flex_placement_final_baselines_follow_main_logical_axis_for_f32() {
        assert_final_baselines_follow_main_logical_axis::<f32>();
    }

    #[test]
    fn logical_flex_placement_final_baselines_follow_main_logical_axis_for_f64() {
        assert_final_baselines_follow_main_logical_axis::<f64>();
    }
}
