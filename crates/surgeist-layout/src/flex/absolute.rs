use super::alignment::{alignment_fallback, alignment_offset};
use super::input::{with_flex_item_projection, with_flex_item_scroll_projections};
use super::scroll::{
    FlexChildContribution, FlexChildContributionsResult, retain_flex_scroll_geometry,
    retained_flex_child_scroll_geometry,
};
use super::{Constants, resolve_auto_optional, resolve_length_or_zero};
use crate::error::{SizingAlgorithm, layout_child_geometry_error};
use crate::geometry::{LogicalSizeOf, PhysicalAxis};
use crate::layout_math::{
    AbsoluteInsetStartOf, OptionalSizeExt, PositionedSizeDeclarations, PositionedUsedSize,
    absolute_inset_alignment_offsets, absolute_inset_child_axes, absolute_inset_extents,
    absolute_stretch_fit_sizes, floor_absolute_definite_insets, resolve_absolute_margins,
};
use crate::scroll::CanonicalScrollBoxOf;
use crate::sizing::resolve::{
    EdgesResultExt, SizeResultExt, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_optional,
};
use crate::{
    AlignItems, AxisSizingRequestOf, Compute, ComputeInputOf, ContainingLayoutContext, Edges,
    FlexItemCollapse, LayoutResultOf, LayoutScalar, NodeOutputOf, ParentFormattingContext, Point,
    Position, RequestedAxis, RunMode, Size, SizingMode, Traverse,
};

pub(super) fn layout_absolute_children<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    constants: &Constants<Tree::Scalar>,
    final_scroll_box: CanonicalScrollBoxOf<Tree::Scalar>,
) -> FlexChildContributionsResult<Tree, M>
where
    Tree: Compute<'input, M>,
{
    let settled_constants = constants.with_final_scroll_box(final_scroll_box);
    let constants = &settled_constants;
    let children = tree.children(node).collect::<Vec<_>>();
    let mut contributions = Vec::new();
    let scroll_progressions = constants
        .axes
        .scroll_origin_axes()
        .scrollable_progressions(constants.flow_axes, final_scroll_box.used_overflow());
    let absolute_containing_size = final_scroll_box.scrollport().size();
    let inset_relative_size = absolute_containing_size.map(Some);
    let available = crate::layout_math::transpose_axis_requests_at_node(
        node,
        absolute_containing_size.map(|value| {
            crate::layout_math::checked_available_space(value)
                .map(|space| AxisSizingRequestOf::new(space, crate::SizingOperation::Normal))
        }),
    )?;

    for (source_index, child) in children.into_iter().enumerate() {
        let contribution = with_flex_item_projection::<Tree, M, _>(tree, child, |tree, style| {
            if style.common.position != Position::Absolute || style.display == crate::Display::None
            {
                return Ok(None);
            }
            let padding = constants
                .flow_axes
                .zip_physical_edges_with_inline_extent(
                    *style.common.padding,
                    inset_relative_size,
                    resolve_length_or_zero,
                )
                .transpose_with_node(tree, child)?;
            let border = constants
                .flow_axes
                .zip_physical_edges_with_inline_extent(
                    *style.common.border,
                    inset_relative_size,
                    resolve_length_or_zero,
                )
                .transpose_with_node(tree, child)?;
            let margin = constants
                .flow_axes
                .zip_physical_edges_with_inline_extent(
                    *style.common.margin,
                    inset_relative_size,
                    resolve_auto_optional,
                )
                .transpose_with_node(tree, child)?;
            let padding_border_size = (padding + border).sum_axes();
            let used_size = PositionedUsedSize::resolve(PositionedSizeDeclarations {
                preferred: Size::new(
                    resolve_preferred_optional(
                        &style.common.size.width,
                        SizingAlgorithm::Positioned,
                        PhysicalAxis::Horizontal,
                        inset_relative_size.width,
                        true,
                    ),
                    resolve_preferred_optional(
                        &style.common.size.height,
                        SizingAlgorithm::Positioned,
                        PhysicalAxis::Vertical,
                        inset_relative_size.height,
                        true,
                    ),
                )
                .transpose_with_node(tree, child)?,
                min: Size::new(
                    resolve_minimum_optional(
                        &style.common.min_size.width,
                        SizingAlgorithm::Positioned,
                        PhysicalAxis::Horizontal,
                        inset_relative_size.width,
                        true,
                    ),
                    resolve_minimum_optional(
                        &style.common.min_size.height,
                        SizingAlgorithm::Positioned,
                        PhysicalAxis::Vertical,
                        inset_relative_size.height,
                        true,
                    ),
                )
                .transpose_with_node(tree, child)?,
                max: Size::new(
                    resolve_maximum_optional(
                        &style.common.max_size.width,
                        SizingAlgorithm::Positioned,
                        PhysicalAxis::Horizontal,
                        inset_relative_size.width,
                        true,
                    ),
                    resolve_maximum_optional(
                        &style.common.max_size.height,
                        SizingAlgorithm::Positioned,
                        PhysicalAxis::Vertical,
                        inset_relative_size.height,
                        true,
                    ),
                )
                .transpose_with_node(tree, child)?,
                aspect_ratio: *style.common.aspect_ratio,
                box_sizing: style.common.box_sizing,
                padding_border: padding_border_size,
            });
            let mut known_size = used_size.preferred;

            let resolved_inset = style
                .common
                .inset
                .zip_size(inset_relative_size, |length, basis| {
                    resolve_auto_optional(length, basis)
                })
                .transpose_with_node(tree, child)?;

            let inset = floor_absolute_definite_insets(
                resolved_inset,
                absolute_containing_size,
                constants.flow_axes,
            );

            let child_input = tree.node_input(child);
            let alignment = LogicalSizeOf::new(child_input.justify_self, child_input.align_self);
            let inset_extents =
                absolute_inset_extents(inset, absolute_containing_size, constants.flow_axes);
            let (stretch, remaining_ratio) = absolute_stretch_fit_sizes(
                inset,
                margin,
                inset_extents,
                constants.flow_axes.physical_size(alignment),
                child_input,
                padding_border_size,
            );
            if known_size.width.is_none()
                && let Some(stretch_width) = stretch.width
            {
                known_size.width = Some(stretch_width);
                known_size =
                    used_size.clamp_optional(used_size.transfer_ratio(known_size, remaining_ratio));
            }
            if known_size.height.is_none()
                && let Some(stretch_height) = stretch.height
            {
                known_size.height = Some(stretch_height);
                known_size =
                    used_size.clamp_optional(used_size.transfer_ratio(known_size, remaining_ratio));
            }
            known_size = used_size.clamp_optional(known_size);
            let available = crate::layout_math::transpose_axis_requests_at_node(
                child,
                absolute_inset_child_axes(
                    available,
                    inset_extents,
                    margin,
                    tree.has_leaf_measurement(child),
                ),
            )?;

            let output = tree.compute_child(
                child,
                ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    known_size,
                    inset_relative_size,
                    ContainingLayoutContext::new(
                        constants.flow_axes,
                        ParentFormattingContext::Flex,
                    ),
                    constants
                        .sizing_request
                        .with_axes(available)
                        .with_orthogonal_fallback(
                            constants.sizing_request.orthogonal_fallback().for_child(
                                constants.positioned_fallback_limits,
                                crate::ScrollportFallbackChangeOf::Preserve,
                            ),
                        ),
                )
                .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
            )?;
            let final_size = used_size.clamp(known_size.unwrap_or(output.size));
            let inset_alignment = absolute_inset_alignment_offsets(
                resolved_inset,
                margin,
                final_size,
                absolute_containing_size,
                constants.flow_axes,
                alignment,
                scroll_progressions,
            );
            let margin = resolve_absolute_margins(
                margin,
                inset,
                final_size,
                absolute_containing_size,
                constants.flow_axes,
            );
            let location = absolute_location(
                final_size,
                margin,
                inset,
                inset_alignment,
                style.align_self.unwrap_or(constants.align_items),
                constants,
            );
            let scroll_geometry = with_flex_item_scroll_projections::<Tree, M, _>(
                tree,
                child,
                |scroll_box, scroll_target| {
                    retained_flex_child_scroll_geometry(
                        scroll_box,
                        scroll_target,
                        final_size,
                        output.content_size,
                        padding,
                        border,
                        output.scroll_geometry,
                    )
                },
            )
            .map_err(|error| layout_child_geometry_error(node, child, error))?;
            let output = retain_flex_scroll_geometry(output, scroll_geometry);

            tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                child,
                crate::output::NormalGeometryOf::box_geometry(
                    location,
                    final_size,
                    scroll_geometry.content_box(),
                ),
                NodeOutputOf::<Tree::Scalar> {
                    source_index: crate::SourceIndex::new(source_index),
                    location,
                    size: final_size,
                    named_baselines: output.named_baselines,
                    content_size: output.content_size,
                    border,
                    padding,
                    margin,
                    ..NodeOutputOf::new()
                }
                .with_scroll_geometry(Some(scroll_geometry)),
            ));
            Ok(Some(FlexChildContribution {
                source_index: crate::SourceIndex::new(source_index),
                location,
                margin,
                geometry: scroll_geometry,
                in_flow: false,
            }))
        })?;
        if let Some(contribution) = contribution {
            contributions.push(contribution);
        }
    }
    Ok(contributions)
}

pub(super) fn layout_hidden_children<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    containing_flow_axes: crate::geometry::FlowAxes,
    settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let children = tree.children(node).collect::<Vec<_>>();
    for (source_index, child) in children.into_iter().enumerate() {
        with_flex_item_projection::<Tree, M, _>(tree, child, |tree, style| {
            let is_collapsed_in_flow = style.common.position != Position::Absolute
                && style.collapse == FlexItemCollapse::Collapsed;
            if style.display != crate::Display::None && !is_collapsed_in_flow {
                return Ok(());
            }

            tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                child,
                crate::output::NormalGeometryOf::metadata(crate::Point::ZERO, crate::Size::ZERO),
                NodeOutputOf::with_source_index(crate::SourceIndex::new(source_index)),
            ));
            tree.compute_child(
                child,
                ComputeInputOf::hidden_in_containing_pass(
                    ContainingLayoutContext::new(
                        containing_flow_axes,
                        ParentFormattingContext::Flex,
                    ),
                    settled_auto_scrollbars,
                ),
            )?;
            Ok(())
        })?;
    }
    Ok(())
}

fn absolute_location<S: LayoutScalar>(
    size: Size<S>,
    margin: Edges<S>,
    inset: Edges<Option<S>>,
    inset_alignment: Edges<S>,
    align_self: AlignItems,
    constants: &Constants<S>,
) -> Point<S> {
    let container = constants
        .node_outer_size
        .unwrap_or(constants.node_inner_size.unwrap_or(Size::<S>::ZERO));
    let axes = constants.axes;
    let main_size = axes.main_size(size);
    let main_extent = axes.main_size(container);
    let main_from_start =
        |offset| axes.main_position_from_normal_start(container, offset, main_size);
    let main = if let Some(start) = axes.normal_main_start_edge(inset) {
        let area_start = axes.normal_main_start_edge(constants.scrollport_inset);
        let margin = axes.normal_main_start_edge(margin);
        let adjustment = axes.normal_main_start_edge(inset_alignment);
        if adjustment == S::ZERO {
            main_from_start(area_start + start + margin)
        } else {
            AbsoluteInsetStartOf::new(start, margin, adjustment).project(
                area_start,
                main_size,
                main_extent,
                axes.normal_axis_start_side(axes.main_logical_axis)
                    .progression(),
            )
        }
    } else if let Some(end) = axes.normal_main_end_edge(inset) {
        main_from_start(
            main_extent
                - axes.normal_main_end_edge(constants.scrollport_inset)
                - main_size
                - end
                - axes.normal_main_end_edge(margin),
        )
    } else {
        let main = absolute_main_alignment(size, margin, container, constants);
        axes.main_position_from_start(container, S::ZERO, main, main_size, S::ZERO)
    };
    let cross_size = axes.cross_size(size);
    let cross_extent = axes.cross_size(container);
    let cross_from_start =
        |offset| axes.cross_position_from_normal_start(container, offset, cross_size);
    let cross = if let Some(start) = axes.normal_cross_start_edge(inset) {
        let area_start = axes.normal_cross_start_edge(constants.scrollport_inset);
        let margin = axes.normal_cross_start_edge(margin);
        let adjustment = axes.normal_cross_start_edge(inset_alignment);
        if adjustment == S::ZERO {
            cross_from_start(area_start + start + margin)
        } else {
            AbsoluteInsetStartOf::new(start, margin, adjustment).project(
                area_start,
                cross_size,
                cross_extent,
                axes.normal_axis_start_side(axes.cross_logical_axis)
                    .progression(),
            )
        }
    } else if let Some(end) = axes.normal_cross_end_edge(inset) {
        cross_from_start(
            cross_extent
                - axes.normal_cross_end_edge(constants.scrollport_inset)
                - cross_size
                - end
                - axes.normal_cross_end_edge(margin),
        )
    } else {
        let cross = absolute_cross_alignment(size, margin, container, align_self, constants);
        axes.cross_position_from_start(container, S::ZERO, cross, cross_size, S::ZERO)
    };

    constants.axes.point_from_main_cross(main, cross)
}

fn absolute_main_alignment<S: LayoutScalar>(
    size: Size<S>,
    margin: Edges<S>,
    container: Size<S>,
    constants: &Constants<S>,
) -> S {
    let axes = constants.axes;
    let content_start = axes.main_start_edge(constants.content_box_inset);
    let content_end = axes.main_end_edge(constants.content_box_inset);
    let free_space = axes.main_size(container)
        - content_start
        - content_end
        - axes.main_size(size)
        - axes.main_edge_sum(margin);
    let alignment = alignment_fallback(free_space, 1, constants.justify_content);
    content_start
        + axes.main_start_edge(margin)
        + alignment_offset(
            free_space,
            1,
            S::ZERO,
            alignment,
            axes.main_is_reversed(),
            true,
        )
}

fn absolute_cross_alignment<S: LayoutScalar>(
    size: Size<S>,
    margin: Edges<S>,
    container: Size<S>,
    align_self: AlignItems,
    constants: &Constants<S>,
) -> S {
    let content_start = constants.axes.cross_start_edge(constants.content_box_inset);
    let content_end = constants.axes.cross_end_edge(constants.content_box_inset);
    let free_space = constants.axes.cross_size(container)
        - content_start
        - content_end
        - constants.axes.cross_size(size);
    let start_edge = || content_start + constants.axes.cross_start_edge(margin);
    let end_edge = || {
        constants.axes.cross_size(container)
            - content_end
            - constants.axes.cross_size(size)
            - constants.axes.cross_end_edge(margin)
    };
    match align_self.safe_fallback(free_space) {
        AlignItems::Start => {
            if constants.axes.cross_is_reversed() {
                end_edge()
            } else {
                start_edge()
            }
        }
        AlignItems::End | AlignItems::LastBaseline => {
            if constants.axes.cross_is_reversed() {
                start_edge()
            } else {
                end_edge()
            }
        }
        AlignItems::FlexStart | AlignItems::Stretch | AlignItems::Baseline => start_edge(),
        AlignItems::FlexEnd => end_edge(),
        AlignItems::Center => {
            (constants.axes.cross_size(container) + content_start
                - content_end
                - constants.axes.cross_size(size)
                + constants.axes.cross_start_edge(margin)
                - constants.axes.cross_end_edge(margin))
                / S::from_f64(2.0)
        }
        AlignItems::Safe(_) | AlignItems::Unsafe(_) => {
            unreachable!("safe_fallback returns unqualified item alignment")
        }
    }
}
