use std::ops::ControlFlow;

use super::items::{CollectedFlexItem, FlexItemRequest, ResolvedFlexItem};
use super::lines::FlexLine;
use super::{Constants, resolve_length_or_zero};
use crate::geometry::LogicalAxis;
use crate::layout_math::{MaxBeforeMinScalarClampExt, OptionalSizeExt};
use crate::sizing::resolve::SizeResultExt;
use crate::{
    AlignItems, AxisSizingRequestOf, Compute, ComputeInputOf, ComputeOutputOf,
    ContainingLayoutContext, LayoutResultOf, LayoutScalar, ParentFormattingContext, RunMode, Size,
    SizingMode, SizingOperation,
};

pub(super) fn intrinsic_content_main_size<Node, S: LayoutScalar>(
    input: ComputeInputOf<S>,
    constants: &Constants<S>,
    items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
) -> S {
    if constants
        .axes
        .main_size(constants.node_outer_size)
        .is_none()
        && constants.axes.main_logical_axis() == LogicalAxis::Inline
        && {
            let request = constants.axes.main_size(input.sizing_request().axes());
            request.operation() == crate::SizingOperation::MaxContent
                || request.operation() == crate::SizingOperation::Normal
                    && request.space() == crate::AvailableSpaceOf::Indefinite
        }
    {
        return lines
            .iter()
            .map(|line| max_content_line_main_size(&items[line.start..line.end], constants))
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
            .unwrap_or(S::ZERO);
    }

    lines
        .iter()
        .map(|line| line.main_size.narrow())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
        .unwrap_or(S::ZERO)
}

fn max_content_line_main_size<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    let gap = constants.axes.main_size(constants.gap);
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 { S::ZERO } else { gap };
            gap + item.max_content_main_size + constants.axes.main_edge_sum(item.margin)
        })
        .fold(S::ZERO, |sum, value| sum + value)
}

pub(super) fn begin_main_size<Node: Copy, S: LayoutScalar>(
    constants: &Constants<S>,
    items: &[CollectedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
) -> ControlFlow<S, IntrinsicMainSize<S>> {
    if let Some(size) = constants.axes.main_size(constants.node_outer_size) {
        return ControlFlow::Break(size);
    }
    {
        let main_request = constants.available_main;
        let content_main = match (main_request.operation(), main_request.space()) {
            (crate::SizingOperation::Normal, crate::AvailableSpaceOf::Definite(available_main)) => {
                let available_main = available_main.get();
                let longest = lines
                    .iter()
                    .map(|line| flex_basis_line_main_size(&items[line.start..line.end], constants))
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
                    .unwrap_or(S::ZERO);
                if lines.len() > 1 {
                    longest.max(available_main)
                } else {
                    longest
                }
            }
            (operation, _) => {
                let intrinsic = match operation {
                    SizingOperation::Normal | SizingOperation::MaxContent => {
                        SizingOperation::MaxContent
                    }
                    SizingOperation::MinContent => SizingOperation::MinContent,
                };
                return ControlFlow::Continue(IntrinsicMainSize::new(constants, lines, intrinsic));
            }
        };
        ControlFlow::Break(
            content_main
                + constants
                    .axes
                    .main_size(constants.content_box_inset.sum_axes()),
        )
    }
}

pub(super) fn finish_main_size<'input, Tree, M>(
    tree: &Tree,
    node: Tree::Node,
    mut constants: Constants<Tree::Scalar>,
    outer_main_size: Tree::Scalar,
) -> LayoutResultOf<Tree::Node, Constants<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let outer_main_size =
        if constants.preferred_inline_phase == super::PreferredInlinePhase::Accepted {
            outer_main_size
        } else {
            outer_main_size.clamp_max_before_min_optional(
                constants.axes.main_size(constants.min_outer_size),
                constants.axes.main_size(constants.max_outer_size),
            )
        };
    let outer_main_size = outer_main_size.max(
        constants
            .axes
            .main_size(constants.non_gutter_box_inset.sum_axes()),
    );
    let inner_main_size = (outer_main_size
        - constants
            .axes
            .main_size(constants.content_box_inset.sum_axes()))
    .max(Tree::Scalar::ZERO);

    constants.node_outer_size = constants
        .axes
        .with_main_size(constants.node_outer_size, Some(outer_main_size));
    constants.node_inner_size = constants
        .axes
        .with_main_size(constants.node_inner_size, Some(inner_main_size));
    constants.available_main = constants.available_main.with_space(
        crate::layout_math::checked_available_space(inner_main_size)
            .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?,
    );
    constants.max_inner_size = constants.max_inner_size.or(constants.node_inner_size);
    constants.gap = constants
        .gap_input
        .zip_map(constants.entry_percentage_basis, |length, basis| {
            resolve_length_or_zero(length, Some(basis.unwrap_or(Tree::Scalar::ZERO)))
        })
        .transpose_with_node(tree, node)?;
    Ok(constants)
}

fn flex_basis_line_main_size<Node, S: LayoutScalar>(
    items: &[CollectedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> S {
    let gap = constants.axes.main_size(constants.gap);
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let gap = if index == 0 { S::ZERO } else { gap };
            let padding_border = constants
                .axes
                .main_size((item.padding + item.border).sum_axes());
            let main_size = constants
                .axes
                .main_size(item.min_size)
                .map_or(item.flex_base.outer(), |min| {
                    item.flex_base.outer().max(min)
                })
                .max(padding_border);
            gap + main_size + constants.axes.main_edge_sum(item.margin)
        })
        .fold(S::ZERO, |sum, value| sum + value)
}

pub(super) struct IntrinsicMainSize<S: LayoutScalar> {
    operation: SizingOperation,
    largest_item: bool,
    extent: S,
    index: usize,
    pending: Option<IntrinsicItemRequest<S>>,
}

struct IntrinsicItemRequest<S: LayoutScalar> {
    cross_available: AxisSizingRequestOf<S>,
    child_known: Size<Option<S>>,
    child_owned_axes: Size<bool>,
    measured: Option<S>,
}

impl<S: LayoutScalar> IntrinsicMainSize<S> {
    fn new(constants: &Constants<S>, lines: &[FlexLine<S>], operation: SizingOperation) -> Self {
        let largest_item = operation == SizingOperation::MinContent && constants.wraps;
        let extent = if largest_item {
            S::ZERO
        } else {
            let gap_count = lines
                .iter()
                .map(|line| (line.end - line.start).saturating_sub(1))
                .sum();
            constants.axes.main_size(constants.gap) * S::from_usize(gap_count)
        };
        Self {
            operation,
            largest_item,
            extent,
            index: 0,
            pending: None,
        }
    }

    pub(super) fn next<'input, Node: Copy, Tree, M>(
        &mut self,
        tree: &Tree,
        constants: &Constants<S>,
        items: &[CollectedFlexItem<Node, S>],
    ) -> LayoutResultOf<Node, Option<FlexItemRequest<Node, S>>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Some(item) = items.get(self.index) else {
            return Ok(None);
        };
        if let Some(pending) = &self.pending {
            debug_assert!(pending.measured.is_some());
            return Ok(Some(FlexItemRequest {
                node: item.node,
                input: pending.input(constants, SizingOperation::MaxContent),
            }));
        }
        let pending = IntrinsicItemRequest::prepare(tree, constants, item)?;
        let input = pending.input(constants, self.operation);
        self.pending = Some(pending);
        Ok(Some(FlexItemRequest {
            node: item.node,
            input,
        }))
    }

    pub(super) fn accept<'input, Node: Copy, Tree, M>(
        &mut self,
        tree: &Tree,
        constants: &Constants<S>,
        items: &[CollectedFlexItem<Node, S>],
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let item = &items[self.index];
        let mut pending = self.pending.take().expect("active intrinsic Flex item");
        let axes = constants.axes;
        let size = axes.main_size(output.size);
        let measured = pending.measured.unwrap_or(size);
        let preferred = if pending.measured.is_some() {
            size
        } else {
            let style = tree.node_input(item.node);
            if axes.main_size(pending.child_owned_axes) {
                if self.operation != SizingOperation::MaxContent {
                    pending.measured = Some(measured);
                    self.pending = Some(pending);
                    return Ok(());
                }
                measured
            } else if let Some(preferred) = axes.main_size(item.size) {
                preferred
            } else if let Some((cross, ratio)) =
                axes.cross_size(pending.child_known).zip(style.aspect_ratio)
            {
                let padding_border = (item.padding + item.border).sum_axes();
                let ratio_insets = if style.box_sizing == crate::BoxSizing::ContentBox {
                    padding_border
                } else {
                    Size::ZERO
                };
                axes.main_size_from_cross_in_sizing_box(
                    cross.max(axes.cross_size(padding_border)),
                    ratio,
                    ratio_insets,
                )
                .map_err(|error| crate::error::sizing_resolution_error(item.node, error.into()))?
            } else {
                S::ZERO
            }
        };
        let contribution = intrinsic_item_contribution(constants, item, measured, preferred);
        self.extent = if self.largest_item {
            self.extent.max(contribution)
        } else {
            self.extent + contribution
        };
        self.index += 1;
        Ok(())
    }

    pub(super) fn outer_size(&self, constants: &Constants<S>) -> S {
        self.extent.max(S::ZERO)
            + constants
                .axes
                .main_size(constants.content_box_inset.sum_axes())
    }
}

impl<S: LayoutScalar> IntrinsicItemRequest<S> {
    fn prepare<'input, Node: Copy, Tree, M>(
        tree: &Tree,
        constants: &Constants<S>,
        item: &CollectedFlexItem<Node, S>,
    ) -> LayoutResultOf<Node, Self, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let axes = constants.axes;
        let cross_available = intrinsic_item_cross_available(constants, item)
            .map_err(|error| crate::error::sizing_resolution_error(item.node, error.into()))?;
        let child_known = intrinsic_item_known_size(constants, item);
        let cross = axes.cross_size(child_known).map(|cross| {
            cross.clamp_max_before_min_optional(
                axes.cross_size(item.min_size),
                axes.cross_size(item.max_size),
            )
        });
        let child_known = axes.with_cross_size(child_known, cross);
        // A flex allocation can supply the cross extent but cannot measure the
        // item's intrinsic main content. Apply its authored main bounds below.
        let child_owned_axes = super::child_preferred_axes::<Tree, M>(tree, item.node);
        Ok(Self {
            cross_available,
            child_known,
            child_owned_axes,
            measured: None,
        })
    }

    fn input(&self, constants: &Constants<S>, operation: SizingOperation) -> ComputeInputOf<S> {
        let Self {
            cross_available,
            child_known,
            child_owned_axes,
            ..
        } = *self;
        let axes = constants.axes;

        let child_available = super::child_preferred_probe_axes(
            axes,
            axes.size_from_main_cross(
                constants.available_main.with_operation(operation),
                cross_available,
            ),
            child_known,
            child_owned_axes,
        );
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::ContentSize,
            axes.main_requested_axis(),
            child_known,
            axes.with_main_size(constants.percentage_parent_size, None),
            ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::Flex),
            constants.sizing_request.with_axes(child_available),
        )
        .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars)
    }
}

fn intrinsic_item_contribution<Node, S: LayoutScalar>(
    constants: &Constants<S>,
    item: &CollectedFlexItem<Node, S>,
    measured: S,
    preferred: S,
) -> S {
    let axes = constants.axes;
    let padding_border = (item.padding + item.border).sum_axes();
    let flex_base = item.flex_base.outer();
    let basis_min = (item.flex_shrink_factor == S::ZERO).then_some(flex_base);
    let basis_max = (item.flex_grow_factor == S::ZERO).then_some(flex_base);
    let minimum = axes
        .main_size(item.min_size)
        .or(item.automatic_min_main_size);
    let contribution = measured
        .max(preferred)
        .clamp_max_before_min_optional(basis_min, basis_max)
        .clamp_max_before_min_optional(minimum, axes.main_size(item.max_size))
        .max(axes.main_size(padding_border));
    contribution + axes.main_edge_sum(item.margin)
}

fn intrinsic_item_cross_available<Node, S: LayoutScalar>(
    constants: &Constants<S>,
    item: &CollectedFlexItem<Node, S>,
) -> Result<AxisSizingRequestOf<S>, crate::LengthResolutionStatus<S>> {
    let cross_margin_sum = constants.axes.cross_edge_sum(item.margin);
    let child_min_cross = constants
        .axes
        .cross_size(item.min_size)
        .map(|value| value + cross_margin_sum);
    let child_max_cross = constants
        .axes
        .cross_size(item.max_size)
        .map(|value| value + cross_margin_sum);
    let parent_cross = constants.cross_content_extent();
    let cross_available = constants.axes.cross_size(constants.sizing_request.axes());
    let cross_available = crate::layout_math::with_available_extent(cross_available, parent_cross)?;
    crate::layout_math::clamp_axis_request_space(cross_available, child_min_cross, child_max_cross)
}

fn intrinsic_item_known_size<Node, S: LayoutScalar>(
    constants: &Constants<S>,
    item: &CollectedFlexItem<Node, S>,
) -> Size<Option<S>> {
    let mut known = constants.axes.with_main_size(item.size, None);
    if !constants.wraps
        && item.align_self == AlignItems::Stretch
        && item.cross_size_is_auto
        && constants.axes.cross_size(known).is_none()
        && !constants.axes.cross_start_edge(item.margin_is_auto)
        && !constants.axes.cross_end_edge(item.margin_is_auto)
        && let Some(cross) = constants.axes.cross_size(constants.percentage_parent_size)
    {
        known = constants.axes.with_cross_size(
            known,
            Some((cross - constants.axes.cross_edge_sum(item.margin)).max(S::ZERO)),
        );
    }
    known
}

pub(super) fn resolved_cross_layout_constants<S: LayoutScalar>(
    constants: &Constants<S>,
    lines: &[FlexLine<S>],
) -> Constants<S> {
    if constants
        .axes
        .cross_size(constants.node_outer_size)
        .is_some()
    {
        return *constants;
    }

    let cross_operation = constants
        .axes
        .cross_size(constants.sizing_request.axes())
        .operation();
    let use_largest_item = match cross_operation {
        SizingOperation::MinContent => !constants.wraps || constants.is_wrapping_column(),
        SizingOperation::MaxContent => !constants.wraps,
        SizingOperation::Normal => false,
    };
    let content_cross = if use_largest_item {
        lines.iter().fold(S::ZERO, |largest, line| {
            largest.max(line.max_item_cross_size).max(line.strut_floor)
        })
    } else {
        let line_cross_gap =
            constants.axes.cross_size(constants.gap) * S::from_usize(lines.len().saturating_sub(1));
        lines
            .iter()
            .fold(S::ZERO, |sum, line| sum + line.cross_size)
            + line_cross_gap
    };
    let cross_inset = constants
        .axes
        .cross_size(constants.content_box_inset.sum_axes());
    let outer_cross_size = (content_cross + cross_inset)
        .clamp_max_before_min_optional(
            constants.axes.cross_size(constants.min_outer_size),
            constants.axes.cross_size(constants.max_outer_size),
        )
        .max(
            constants
                .axes
                .cross_size(constants.non_gutter_box_inset.sum_axes()),
        )
        .max(constants.axes.cross_size(constants.padding_border_size));
    let inner_cross_size = (outer_cross_size - cross_inset).max(S::ZERO);

    let mut constants = *constants;
    constants.node_outer_size = constants
        .axes
        .with_cross_size(constants.node_outer_size, Some(outer_cross_size));
    constants.node_inner_size = constants
        .axes
        .with_cross_size(constants.node_inner_size, Some(inner_cross_size));
    constants.max_inner_size = constants.max_inner_size.or(constants.node_inner_size);
    constants
}
