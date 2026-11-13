use super::*;
use crate::LayoutErrorOf;

pub(super) struct LineResolution<Node, S: LayoutScalar> {
    items: Vec<ResolvedFlexItem<Node, S>>,
    lines: Vec<FlexLine<S>>,
    line_index: usize,
    cross_cursor: S,
    active: Option<LinePosition<S>>,
}

struct LinePosition<S: LayoutScalar> {
    item_index: usize,
    free_space: DistributionNumber<S>,
    justify_content: AlignContent,
    main_gap: DistributionNumber<S>,
    main_cursor: DistributionNumber<S>,
    cross_size: S,
}

pub(super) struct CrossSizeChild<Node, S: LayoutScalar> {
    pub(super) request: FlexItemRequest<Node, S>,
    receipt: CrossSizeReceipt<S>,
}

enum CrossSizeReceipt<S: LayoutScalar> {
    AuthoredBaseline,
    RatioMinimum {
        transferred: S,
        baseline: Option<RatioBaselineRequest<S>>,
    },
    RatioBaseline {
        cross: S,
    },
    Measured,
}

struct RatioBaselineRequest<S: LayoutScalar> {
    available_main: AxisSizingRequestOf<S>,
    available_cross: AxisSizingRequestOf<S>,
    child_flow: FlowAxes,
}

impl<Node: Copy, S: LayoutScalar> LineResolution<Node, S> {
    pub(super) fn new(items: &[CollectedFlexItem<Node, S>], lines: Vec<FlexLine<S>>) -> Self {
        Self {
            items: items.iter().copied().map(ResolvedFlexItem::from).collect(),
            lines,
            line_index: 0,
            cross_cursor: S::ZERO,
            active: None,
        }
    }

    pub(super) fn advance<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
        container: Node,
        constants: &Constants<S>,
    ) -> LayoutResultOf<Node, Option<CrossSizeChild<Node, S>>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let invalid_main = |value| invalid_main(container, value);
        let cross_gap = constants.axes.cross_size(constants.gap);
        while let Some(line) = self.lines.get_mut(self.line_index) {
            if self.active.is_none() {
                let items = &mut self.items[line.start..line.end];
                resolve_flexible_lengths(items, constants).map_err(invalid_main)?;
                if items.is_empty() && line.contains_collapsed_slot {
                    line.max_item_cross_size = S::ZERO;
                    line.main_size = DistributionNumber::ZERO;
                    line.main_overflows = false;
                    line.cross_size = if constants.wraps {
                        line.strut_floor
                    } else {
                        constants
                            .axes
                            .cross_size(constants.node_inner_size)
                            .unwrap_or(line.strut_floor)
                    };
                    line.offset_cross = self.cross_cursor;
                    self.cross_cursor = self.cross_cursor + line.cross_size + cross_gap;
                    self.line_index += 1;
                    continue;
                }
                resolve_main_axis_auto_margins(items, constants).map_err(invalid_main)?;
                let free_space = line_free_space(items, constants).map_err(invalid_main)?;
                let sign = free_space.sign();
                line.main_overflows = sign == core::cmp::Ordering::Less;
                let justify_content = alignment_fallback_from_sign(
                    Some(sign),
                    items.len(),
                    constants.justify_content,
                );
                let main_gap = DistributionNumber::finite(constants.axes.main_size(constants.gap))
                    .map_err(invalid_main)?;
                self.active = Some(LinePosition {
                    item_index: line.start,
                    free_space,
                    justify_content,
                    main_gap,
                    main_cursor: main_alignment_offset(
                        free_space,
                        items.len(),
                        main_gap,
                        justify_content,
                        constants.axes.main_is_reversed(),
                        true,
                    ),
                    cross_size: S::ZERO,
                });
            }
            let active = self.active.as_mut().expect("active Flex line");
            if active.item_index == line.end {
                line.max_item_cross_size = active.cross_size;
                let items = &mut self.items[line.start..line.end];
                let cross_size = active
                    .cross_size
                    .max(line_cross_size(items, constants))
                    .max(line.strut_floor);
                line.main_size = active.main_cursor;
                line.cross_size = if constants.wraps {
                    cross_size
                } else {
                    constants
                        .axes
                        .cross_size(constants.node_inner_size)
                        .unwrap_or(cross_size)
                };
                line.offset_cross = self.cross_cursor;
                align_items_on_cross_axis(items, line.cross_size, self.cross_cursor, constants);
                self.cross_cursor = self.cross_cursor + line.cross_size + cross_gap;
                self.active = None;
                self.line_index += 1;
                continue;
            }

            if active.item_index > line.start {
                active.main_cursor = active.main_cursor
                    + main_alignment_offset(
                        active.free_space,
                        line.end - line.start,
                        active.main_gap,
                        active.justify_content,
                        constants.axes.main_is_reversed(),
                        false,
                    );
            }
            let item = &mut self.items[active.item_index];
            if let Some(child) = prepare_cross_size(tree, item, constants)? {
                return Ok(Some(child));
            }
            active.finish_item(container, item, self.cross_cursor, constants)?;
        }
        stretch_lines_on_cross_axis(&mut self.items, &mut self.lines, constants);
        align_lines_on_cross_axis(&mut self.items, &mut self.lines, constants);
        Ok(None)
    }

    pub(super) fn accept<M>(
        &mut self,
        container: Node,
        constants: &Constants<S>,
        child: CrossSizeChild<Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, Option<CrossSizeChild<Node, S>>, S, M> {
        let active = self
            .active
            .as_mut()
            .expect("cross measurement belongs to an active Flex line");
        let item = &mut self.items[active.item_index];
        let axes = constants.axes;
        match child.receipt {
            CrossSizeReceipt::AuthoredBaseline => item.baseline.refresh(output),
            CrossSizeReceipt::RatioBaseline { cross } => {
                item.baseline.refresh(output);
                item.target_size = axes.with_cross_size(item.target_size, cross);
            }
            CrossSizeReceipt::RatioMinimum {
                transferred,
                baseline,
            } => {
                let cross = clamp_cross(item, axes, transferred.max(axes.cross_size(output.size)));
                if let Some(baseline) = baseline {
                    return Ok(Some(baseline.prepare(item, constants, cross)));
                }
                item.target_size = axes.with_cross_size(item.target_size, cross);
            }
            CrossSizeReceipt::Measured => {
                item.baseline.refresh(output);
                let cross = clamp_cross(item, axes, axes.cross_size(output.size));
                item.target_size = axes.with_cross_size(item.target_size, cross);
            }
        }
        active.finish_item(container, item, self.cross_cursor, constants)?;
        Ok(None)
    }

    pub(super) fn lines(&self) -> &[FlexLine<S>] {
        &self.lines
    }

    pub(super) fn finish(self) -> (Vec<ResolvedFlexItem<Node, S>>, Vec<FlexLine<S>>) {
        (self.items, self.lines)
    }
}

impl<S: LayoutScalar> LinePosition<S> {
    fn finish_item<Node: Copy, M>(
        &mut self,
        container: Node,
        item: &mut ResolvedFlexItem<Node, S>,
        cross_cursor: S,
        constants: &Constants<S>,
    ) -> LayoutResultOf<Node, (), S, M> {
        let wide = |value| {
            DistributionNumber::finite(value).map_err(|value| invalid_main(container, value))
        };
        let axes = constants.axes;
        item.offset_main = self.main_cursor + wide(item.margin_main_start(constants))?;
        item.offset_cross = cross_cursor + axes.cross_start_edge(item.margin);

        let margin =
            wide(item.margin_main_start(constants))? + wide(axes.main_end_edge(item.margin))?;
        self.main_cursor = self.main_cursor + wide(axes.main_size(item.target_size))? + margin;
        self.cross_size = self
            .cross_size
            .max(axes.cross_size(item.target_size) + axes.cross_edge_sum(item.margin));
        self.item_index += 1;
        Ok(())
    }
}

fn invalid_main<Node: Copy, S: LayoutScalar, M>(node: Node, value: S) -> LayoutErrorOf<Node, S, M> {
    crate::error::sizing_resolution_error(
        node,
        LengthResolutionStatus::InvalidNumeric { value }.into(),
    )
}

fn clamp_cross<Node, S: LayoutScalar>(
    item: &ResolvedFlexItem<Node, S>,
    axes: FlexAxes,
    cross: S,
) -> S {
    cross
        .clamp_max_before_min_optional(
            axes.cross_size(item.min_size),
            axes.cross_size(item.max_size),
        )
        .max(axes.cross_size((item.padding + item.border).sum_axes()))
}

impl<S: LayoutScalar> RatioBaselineRequest<S> {
    fn prepare<Node: Copy>(
        self,
        item: &ResolvedFlexItem<Node, S>,
        constants: &Constants<S>,
        cross: S,
    ) -> CrossSizeChild<Node, S> {
        let input = cross_size_request(
            item,
            constants,
            CrossSizeRequest {
                mode: SizingMode::InherentSize,
                available_main: self.available_main,
                available_cross: self.available_cross,
                known_cross: Some(cross),
                demand_baseline: true,
                indefinite_axis: item
                    .settled_indefinite_percentage_axis(constants.axes, self.child_flow),
            },
        );
        CrossSizeChild {
            request: FlexItemRequest {
                node: item.node,
                input,
            },
            receipt: CrossSizeReceipt::RatioBaseline { cross },
        }
    }
}

struct CrossSizeRequest<S: LayoutScalar> {
    mode: SizingMode,
    available_main: AxisSizingRequestOf<S>,
    available_cross: AxisSizingRequestOf<S>,
    known_cross: Option<S>,
    demand_baseline: bool,
    indefinite_axis: Option<PhysicalAxis>,
}

fn cross_size_request<Node, S: LayoutScalar>(
    item: &ResolvedFlexItem<Node, S>,
    constants: &Constants<S>,
    request: CrossSizeRequest<S>,
) -> ComputeInputOf<S> {
    let axes = constants.axes;
    ComputeInputOf::for_child(
        RunMode::ComputeSize,
        request.mode,
        axes.cross_requested_axis(),
        axes.size_from_main_cross(Some(axes.main_size(item.target_size)), request.known_cross),
        constants.percentage_parent_size,
        ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::Flex),
        constants
            .sizing_request
            .with_axes(axes.size_from_main_cross(request.available_main, request.available_cross)),
    )
    .with_physical_baseline_demand(request.demand_baseline)
    .with_indefinite_percentage_axis(request.indefinite_axis)
    .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars)
}

type CrossSizeStep<Node, S, M> = LayoutResultOf<Node, Option<CrossSizeChild<Node, S>>, S, M>;

fn prepare_cross_size<'input, Tree, M>(
    tree: &mut Tree,
    item: &mut ResolvedFlexItem<Tree::Node, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
) -> CrossSizeStep<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let axes = constants.axes;
    let padding_border = (item.padding + item.border).sum_axes();
    let padding_border_cross = axes.cross_size(padding_border);
    let minimum = axes.cross_size(item.min_size);
    let maximum = axes.cross_size(item.max_size);
    if let Some(authored_cross) = item.authored_cross_size {
        item.target_size =
            axes.with_cross_size(item.target_size, clamp_cross(item, axes, authored_cross));
        if item.uses_native_cross_baseline(axes) && item.target_size != item.initial_output.size {
            let input = with_flex_item_projection::<Tree, M, _>(tree, item.node, |tree, style| {
                items::sized_item_request(tree, item, &style, constants, RunMode::ComputeSize)
            })?;
            return Ok(Some(CrossSizeChild {
                request: FlexItemRequest {
                    node: item.node,
                    input,
                },
                receipt: CrossSizeReceipt::AuthoredBaseline,
            }));
        }
        return Ok(None);
    }

    let used_main = axes.main_size(item.target_size);
    let style = tree.node_input(item.node);
    let ratio = style.aspect_ratio.filter(|_| item.cross_size_is_auto);
    let ratio_box_insets = if style.box_sizing == BoxSizing::ContentBox {
        padding_border
    } else {
        Size::ZERO
    };
    let scrolls_cross = axes.cross_size(Size::new(style.overflow.x(), style.overflow.y()));
    let needs_ratio_minimum = axes
        .cross_size(Size::new(&style.min_size.width, &style.min_size.height))
        .is_auto()
        && !style.item_is_replaced
        && !scrolls_cross.is_scrollable();
    let main_size_changed = (used_main - axes.main_size(item.initial_output.size)).abs()
        > Tree::Scalar::from_f64(0.001);
    if ratio.is_none()
        && !item.initial_output_is_preferred_basis
        && item.initial_output.content_size == item.initial_output.size
        && !main_size_changed
    {
        let cross = clamp_cross(item, axes, axes.cross_size(item.initial_output.size));
        item.target_size = axes.with_cross_size(item.target_size, cross);
        return Ok(None);
    }

    let cross_request = crate::layout_math::with_available_extent(
        axes.cross_size(constants.sizing_request.axes()),
        constants.cross_content_extent(),
    )
    .map_err(|error| crate::error::sizing_resolution_error(item.node, error.into()))?;
    let mut available_cross =
        crate::layout_math::clamp_axis_request_space(cross_request, minimum, maximum)
            .map_err(|error| crate::error::sizing_resolution_error(item.node, error.into()))?;
    if let Some(extent) = available_cross.space().definite_size() {
        available_cross = available_cross.with_space(
            crate::layout_math::checked_available_space(extent.get().max(padding_border_cross))
                .map_err(|error| crate::error::sizing_resolution_error(item.node, error.into()))?,
        );
    }
    let available_main = crate::AxisSizingRequestOf::new(
        crate::layout_math::checked_available_space(used_main)
            .map_err(|error| crate::error::sizing_resolution_error(item.node, error.into()))?,
        crate::SizingOperation::Normal,
    );
    let uses_native_baseline = item.uses_native_cross_baseline(axes);
    if let Some(ratio) = ratio {
        let transferred = axes
            .cross_size_from_main_in_sizing_box(used_main, ratio, ratio_box_insets)
            .map_err(|error| crate::error::sizing_resolution_error(item.node, error.into()))?;
        let baseline = uses_native_baseline.then(|| RatioBaselineRequest {
            available_main,
            available_cross,
            child_flow: FlowAxes::new(style.writing_mode, style.direction),
        });
        // Measure the automatic minimum without giving descendants a provisional
        // definite cross size.
        if needs_ratio_minimum {
            let min_request = crate::layout_math::with_available_extent(
                axes.cross_size(constants.sizing_request.axes())
                    .with_operation(crate::SizingOperation::MinContent),
                constants
                    .cross_content_offer
                    .or(axes.cross_size(constants.percentage_parent_size)),
            )
            .map_err(|error| crate::error::sizing_resolution_error(item.node, error.into()))?;
            let input = cross_size_request(
                item,
                constants,
                CrossSizeRequest {
                    mode: SizingMode::ContentSize,
                    available_main,
                    available_cross: min_request,
                    known_cross: None,
                    demand_baseline: false,
                    indefinite_axis: item.indefinite_percentage_axis,
                },
            );
            return Ok(Some(CrossSizeChild {
                request: FlexItemRequest {
                    node: item.node,
                    input,
                },
                receipt: CrossSizeReceipt::RatioMinimum {
                    transferred,
                    baseline,
                },
            }));
        }
        let cross = clamp_cross(item, axes, transferred);
        if let Some(baseline) = baseline {
            return Ok(Some(baseline.prepare(item, constants, cross)));
        }
        item.target_size = axes.with_cross_size(item.target_size, cross);
        return Ok(None);
    }

    let input = cross_size_request(
        item,
        constants,
        CrossSizeRequest {
            mode: SizingMode::InherentSize,
            available_main,
            available_cross,
            known_cross: None,
            demand_baseline: constants.physical_baseline_demand || uses_native_baseline,
            indefinite_axis: item.indefinite_percentage_axis,
        },
    );
    Ok(Some(CrossSizeChild {
        request: FlexItemRequest {
            node: item.node,
            input,
        },
        receipt: CrossSizeReceipt::Measured,
    }))
}
