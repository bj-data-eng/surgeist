use super::{
    AlignContent, AlignItems, AspectRatioOf, AxisSizingRequestOf, BaselinesOf, BoxSizing, Compute,
    ComputeInputOf, ComputeOutputOf, ContainingLayoutContext, Direction, Edges, FlexDirection,
    FlexWrap, LayoutResultOf, LayoutScalar, LengthAutoOf, LengthOf, LengthResolutionStatus,
    ParentFormattingContext, Point, RequestedAxis, RunMode, Size, SizingMode, Traverse,
};
use crate::engine::containment::{
    compute_empty_box, contained_outer_size, empty_sizing_input, size_containment,
};
use crate::error::{SizingAlgorithm, layout_own_geometry_error};
use crate::geometry::{FlowAxes, LogicalAxis, PhysicalAxis, PhysicalProgression, PhysicalSide};
use crate::layout_math::{
    MaxBeforeMinOptionalSizeClampExt, MaxBeforeMinScalarClampExt, MaxBeforeMinSizeClampExt,
    OptionalMinimumSizeFloorExt, OptionalSizeExt, OptionalSizeMaxExt, UncheckedOptionalSizeSubExt,
    resolution_optional, resolution_or_zero, resolve_containing_padding_border,
    transfer_aspect_ratio_limits,
};
use crate::scroll::{
    CanonicalScrollBoxOf, CanonicalScrollBoxSourceOf, ScrollOriginAxes, ScrollOriginProgression,
    canonical_scroll_box_from_source,
};
use crate::sizing::resolve::{
    EdgesResultExt, SizeResultExt, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_optional,
};

mod absolute;
mod alignment;
mod distribution_number;
mod flexible_lengths;
mod input;
mod intrinsic;
mod items;
mod lines;
mod scroll;

use absolute::{layout_absolute_children, layout_hidden_children};
use alignment::{
    align_items_on_cross_axis, align_lines_on_cross_axis, alignment_fallback_from_sign,
    first_final_vertical_baseline, last_final_vertical_baseline, line_cross_size, line_free_space,
    main_alignment_offset, resolve_main_axis_auto_margins, resolved_vertical_baselines,
    stretch_lines_on_cross_axis,
};
use distribution_number::DistributionNumber;
use flexible_lengths::resolve_flexible_lengths;
use input::{FlexContainerProjection, with_flex_item_projection};
use intrinsic::{
    intrinsic_content_main_size, resolved_cross_layout_constants, resolved_layout_constants,
};
use items::{
    CollectedFlexItem, ResolvedFlexItem, collect_items, compute_sized_item, final_layout,
    has_eligible_items, recollect_items,
};
use lines::{
    CollapsedFlexStruts, FlexLine, FlexLineCollectionRound, collect_flex_lines,
    flex_line_collection_size,
};
use scroll::{
    flex_container_scroll_box, flex_container_scroll_geometry, flex_scroll_contribution_error,
    flex_scroll_contributions, retain_flex_scroll_geometry,
};

/// When a row owns its preferred inline size, retain any intrinsic cross
/// preference handled by the same formatter.
pub(crate) fn preferred_flex_axes<S: LayoutScalar>(
    display: crate::Display,
    direction: FlexDirection,
    flow_axes: FlowAxes,
    preferred: &Size<crate::PreferredSizeOf<S>>,
) -> Size<bool> {
    let preferred = flow_axes.logical_size(Size::new(&preferred.width, &preferred.height));
    let owns_inline =
        display == crate::Display::Flex && direction.is_row() && preferred.inline.is_max_content();
    flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
        owns_inline,
        owns_inline && preferred.block.is_max_content(),
    ))
}

pub(crate) fn child_preferred_axes<'input, Tree, M>(tree: &Tree, child: Tree::Node) -> Size<bool>
where
    Tree: Compute<'input, M>,
{
    if tree.has_leaf_measurement(child) {
        return Size::splat(false);
    }
    let input = tree.node_input(child);
    preferred_flex_axes(
        input.display,
        input.flex_direction,
        FlowAxes::new(input.writing_mode, input.direction),
        &input.size,
    )
}

fn preferred_intrinsic_request<S: LayoutScalar>() -> AxisSizingRequestOf<S> {
    AxisSizingRequestOf::new(
        crate::AvailableSpaceOf::Indefinite,
        crate::SizingOperation::MaxContent,
    )
}

/// A ContentSize probe omits the child's authored sizes. Retain the operations
/// that its intrinsic preferences supply while preserving imposed dimensions
/// and the specific main-axis contribution requested by the parent.
fn child_preferred_probe_axes<S: LayoutScalar>(
    axes: FlexAxes,
    requests: Size<AxisSizingRequestOf<S>>,
    known: Size<Option<S>>,
    preferred: Size<bool>,
) -> Size<AxisSizingRequestOf<S>> {
    let mut main = axes.main_size(requests);
    let mut cross = axes.cross_size(requests);
    if axes.main_size(preferred) && axes.main_size(known).is_none() {
        main = main.with_space(crate::AvailableSpaceOf::Indefinite);
    }
    if axes.cross_size(preferred) && axes.cross_size(known).is_none() {
        cross = preferred_intrinsic_request();
    }
    axes.size_from_main_cross(main, cross)
}

pub(crate) fn compute_flex<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let style = tree.node_input(node);
    let containment = size_containment(style);
    let contained_size = if containment != crate::SizeContainment::None {
        let empty = compute_empty_box(node, style, |probe| {
            compute_flex_inner(probe, node, empty_sizing_input(input), Size::NONE)
        })?;
        if containment == crate::SizeContainment::BothAxes
            && !input.run_mode().is_perform_layout()
            && !input.requires_content_baseline()
        {
            return Ok(empty);
        }
        contained_outer_size(
            containment,
            crate::FlowAxes::new(style.writing_mode, style.direction),
            empty.size,
        )
    } else {
        Size::NONE
    };
    let mut pass_input = input;
    loop {
        let output =
            compute_flex_inner::<Tree, Tree::Scalar, M>(tree, node, pass_input, contained_size)?;
        if !input.run_mode().is_perform_layout() {
            return Ok(output);
        }
        let Some(geometry) = output.scroll_geometry else {
            return Ok(output);
        };
        let next_state = pass_input.settled_auto_scrollbars().transition(geometry);
        if next_state == pass_input.settled_auto_scrollbars()
            || !crate::scroll::settled_auto_scrollbars_change_available_geometry(
                geometry, next_state,
            )
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?
        {
            return Ok(output);
        }
        pass_input = input.with_settled_auto_scrollbars(next_state);
    }
}

fn compute_flex_inner<'input, Tree, S, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<S>,
    contained_size: Size<Option<S>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let (constants, own_ratio_cross) =
        self::input::with_flex_container_projection::<Tree, M, _>(tree, node, |style| {
            self::input::with_flex_scroll_projections::<Tree, M, _>(tree, node, |scroll_box, _| {
                let constants = Constants::new::<Tree, M>(
                    tree,
                    node,
                    &style,
                    scroll_box,
                    input,
                    contained_size,
                )?;
                let own_ratio_cross = OwnRatioCross::for_preferred_inline(&constants, &style);
                Ok((constants, own_ratio_cross))
            })
        })?;
    if input.run_mode() == RunMode::ComputeSize
        && !input.physical_baseline_demand()
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
    {
        let output = ComputeOutputOf::<S>::from_outer_size(Size::new(width, height));
        // A requested named fact needs the actual eligible item set even when
        // known dimensions make child layout unnecessary.
        return Ok(
            if input.named_baseline_demand() != crate::output::NamedBaselineDemand::None
                && !has_eligible_items::<Tree, M>(tree, node)
            {
                output.with_named_baselines(crate::NamedBaselinesOf::absent())
            } else {
                output
            },
        );
    }

    let requested_inline_operation = constants
        .flow_axes
        .logical_size(input.sizing_request().axes())
        .inline
        .operation();
    let first_run_mode = if constants.preferred_inline_phase == PreferredInlinePhase::Discover {
        // Preferred-size discovery has no accepted percentage basis yet.
        RunMode::ComputeSize
    } else {
        input.run_mode()
    };
    let mut collected_items = collect_items(tree, node, &constants, first_run_mode)?;
    let (mut layout_constants, mut resolved_items, mut lines) =
        resolve_flex_collection(tree, node, first_run_mode, &constants, &collected_items)?;
    let mut sizes = container_sizes(input, &layout_constants, &resolved_items, &lines);
    let scroll_box_for = |tree: &Tree, constants: &Constants<S>, size: Size<S>| {
        DistributionNumber::finite(constants.axes.main_size(size)).map_err(|value| {
            crate::error::sizing_resolution_error(
                node,
                LengthResolutionStatus::InvalidNumeric { value }.into(),
            )
        })?;
        self::input::with_flex_scroll_projections::<Tree, M, _>(tree, node, |projection, _| {
            flex_container_scroll_box::<_, S, M>(
                node,
                input.run_mode(),
                projection,
                constants,
                size,
            )
        })
    };
    // Normal size queries need the same accepted inline frame as publication.
    // Constructing this internal box does not stage output geometry.
    let mut final_scroll_box = scroll_box_for(tree, &layout_constants, sizes.output)?;
    if (!collected_items.is_empty() || own_ratio_cross.is_some())
        && let Some(mut contents_constants) = constants
            .complete_inline_contents(
                final_scroll_box,
                requested_inline_operation,
                own_ratio_cross.is_some(),
            )
            .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?
    {
        if let Some(ratio) = own_ratio_cross {
            let cross = settle_own_ratio_cross(tree, node, &contents_constants, ratio)?;
            let size = contents_constants
                .axes
                .with_cross_size(final_scroll_box.border_box().size(), cross);
            let settled = scroll_box_for(tree, &contents_constants, size)?;
            // Use the settled cross geometry for lines and placement while
            // retaining only the entry-qualified percentage bases.
            contents_constants = contents_constants.with_final_scroll_box(settled);
            contents_constants.max_inner_size = contents_constants
                .max_outer_size
                .sub_optional_unchecked(contents_constants.content_box_inset.sum_axes())
                .max_optional(Size::ZERO.map(Some));
        }
        // Replay with the accepted extent and the caller's contents operation.
        collected_items = collect_items(tree, node, &contents_constants, input.run_mode())?;
        (layout_constants, resolved_items, lines) = resolve_flex_collection(
            tree,
            node,
            input.run_mode(),
            &contents_constants,
            &collected_items,
        )?;
        sizes = container_sizes(input, &layout_constants, &resolved_items, &lines);
        final_scroll_box = scroll_box_for(tree, &layout_constants, sizes.output)?;
    }
    let (absolute_contributions, final_items) = if input.run_mode().is_perform_layout() {
        let inset_basis = crate::output::MeasuredInsetBasisOf::from_used_content_size(
            final_scroll_box.content_box().size(),
        );
        let final_items = final_layout(
            tree,
            node,
            &collected_items,
            &resolved_items,
            &layout_constants,
            inset_basis,
        )?;
        let absolute_contributions =
            layout_absolute_children(tree, node, &layout_constants, final_scroll_box)?;
        layout_hidden_children(
            tree,
            node,
            layout_constants.axes.flow_axes(),
            layout_constants.settled_auto_scrollbars,
        )?;
        (absolute_contributions, Some(final_items))
    } else {
        (Vec::new(), None)
    };

    let mut final_geometry_and_content_size = None;
    if input.run_mode().is_perform_layout() {
        let final_items = final_items
            .as_deref()
            .expect("performed flex layout retains its final in-flow items");
        let scroll_box = final_scroll_box;
        let range_error = |error| {
            flex_scroll_contribution_error(
                node,
                input.run_mode(),
                layout_constants.axes.main_physical_axis(),
                error,
            )
        };
        let mut contributions = flex_scroll_contributions(
            final_items,
            &absolute_contributions,
            &lines,
            &layout_constants,
            scroll_box,
        )
        .map_err(range_error)?;
        contributions.exclude_reserved_gutter_from_range();
        let scroll_geometry = self::input::with_flex_scroll_projections::<Tree, M, _>(
            tree,
            node,
            |scroll_box_projection, scroll_target_projection| {
                flex_container_scroll_geometry::<_, S, M>(
                    node,
                    input.run_mode(),
                    scroll_box_projection,
                    scroll_target_projection,
                    &layout_constants,
                    scroll_box,
                    contributions,
                )
            },
        )?;
        let content_size = contributions
            .content_size_from_anchor(scroll_geometry.content_box().origin())
            .map_err(range_error)?;
        final_geometry_and_content_size = Some((scroll_geometry, content_size));
    }
    let baselines = if let Some(items) = final_items.as_deref() {
        BaselinesOf {
            first: first_final_vertical_baseline(items, &lines, &layout_constants)
                .unwrap_or(Point::NONE),
            last: last_final_vertical_baseline(items, &lines, &layout_constants)
                .unwrap_or(Point::NONE),
        }
    } else {
        resolved_vertical_baselines(&resolved_items, &lines, &layout_constants, |item| {
            if input.physical_baseline_demand() {
                with_flex_item_projection::<Tree, M, _>(tree, item.node, |tree, style| {
                    compute_sized_item(tree, item, &style, &layout_constants, RunMode::ComputeSize)
                        .map(Some)
                })
            } else {
                Ok(None)
            }
        })?
    };
    let content_size = if input.run_mode().is_perform_layout() {
        final_geometry_and_content_size
            .as_ref()
            .expect("perform-layout flex output requires accumulated content extent")
            .1
    } else {
        DistributionNumber::finite(layout_constants.axes.main_size(sizes.intrinsic_content))
            .map_err(|value| {
                crate::error::sizing_resolution_error(
                    node,
                    LengthResolutionStatus::InvalidNumeric { value }.into(),
                )
            })?;
        sizes.intrinsic_content
    };
    let output = ComputeOutputOf::from_sizes_and_baselines(sizes.output, content_size, baselines);
    // Inspect the original eligible collection, not the resolved lines: a
    // nonempty collection of collapsed items is not proof of named absence.
    let output = if collected_items.is_empty() {
        output.with_named_baselines(crate::NamedBaselinesOf::absent())
    } else {
        output
    };
    Ok(
        final_geometry_and_content_size.map_or(output, |(scroll_geometry, _)| {
            retain_flex_scroll_geometry(output, scroll_geometry)
        }),
    )
}

fn settle_own_ratio_cross<'input, Tree, S, M>(
    tree: &mut Tree,
    node: Tree::Node,
    constants: &Constants<S>,
    ratio: OwnRatioCross<S>,
) -> LayoutResultOf<Tree::Node, S, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let axes = constants.axes;
    let main = axes
        .main_size(constants.node_outer_size)
        .expect("the ratio settles after the preferred inline size");
    let transferred = axes
        .cross_size_from_main_in_sizing_box(main, ratio.value, ratio.sizing_box_insets)
        .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?;

    let cross = if ratio.needs_content_minimum {
        // Measure the content floor at the accepted inline size with no
        // provisional cross size. Apply the authored maximum after measuring.
        let mut minimum = *constants;
        minimum.max_outer_size = axes.with_cross_size(minimum.max_outer_size, None);
        minimum.max_inner_size = axes.with_cross_size(minimum.max_inner_size, None);
        let cross_request = AxisSizingRequestOf::new(
            crate::AvailableSpaceOf::Indefinite,
            crate::SizingOperation::MinContent,
        );
        minimum.sizing_request = minimum
            .sizing_request
            .with_axes(axes.with_cross_size(minimum.sizing_request.axes(), cross_request));
        let items = collect_items(tree, node, &minimum, RunMode::ComputeSize)?;
        let (measured, _, _) =
            resolve_flex_collection(tree, node, RunMode::ComputeSize, &minimum, &items)?;
        transferred.max(
            axes.cross_size(measured.node_outer_size)
                .expect("Flex cross measurement resolves the measured outer size"),
        )
    } else {
        transferred
    };

    Ok(cross
        .clamp_max_before_min_optional(
            axes.cross_size(constants.min_outer_size),
            axes.cross_size(constants.max_outer_size),
        )
        .max(axes.cross_size(constants.padding_border_size)))
}

type ResolvedFlexRound<Tree> = (
    Constants<<Tree as Traverse>::Scalar>,
    Vec<ResolvedFlexItem<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>>,
    Vec<FlexLine<<Tree as Traverse>::Scalar>>,
);

fn resolve_flex_collection<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    run_mode: RunMode,
    constants: &Constants<Tree::Scalar>,
    collected_items: &[CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>],
) -> LayoutResultOf<<Tree as Traverse>::Node, ResolvedFlexRound<Tree>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut first_constants = *constants;
    let first_round = FlexLineCollectionRound::Normal;
    let first_items = collect_with_column_cross_offer(
        tree,
        &mut first_constants,
        collected_items,
        run_mode,
        first_round,
    )?;
    let first_layout = resolve_flex_round(tree, node, &first_constants, &first_items, None)?;
    if !collected_items.iter().any(CollectedFlexItem::is_collapsed) {
        return Ok(first_layout);
    }

    drop(first_items);
    let (_, _, first_lines) = first_layout;
    let struts = CollapsedFlexStruts::capture(collected_items, &first_lines);
    let mut second_constants = *constants;
    let second_round = FlexLineCollectionRound::Collapsed;
    let second_items = collect_with_column_cross_offer(
        tree,
        &mut second_constants,
        collected_items,
        run_mode,
        second_round,
    )?;
    resolve_flex_round(tree, node, &second_constants, &second_items, Some(&struts))
}

type FlexRoundItems<'a, Tree> =
    std::borrow::Cow<'a, [CollectedFlexItem<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>]>;

fn collect_with_column_cross_offer<'input, 'a, Tree, M>(
    tree: &mut Tree,
    constants: &mut Constants<Tree::Scalar>,
    collected_items: &'a [CollectedFlexItem<Tree::Node, Tree::Scalar>],
    run_mode: RunMode,
    round: FlexLineCollectionRound,
) -> LayoutResultOf<Tree::Node, FlexRoundItems<'a, Tree>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    if constants.is_wrapping_column()
        && constants
            .axes
            .cross_size(constants.node_outer_size)
            .is_none()
        && constants
            .axes
            .cross_size(constants.sizing_request.axes())
            .operation()
            == crate::SizingOperation::MaxContent
        && let Some(offer) = collected_items
            .iter()
            .filter(|item| round == FlexLineCollectionRound::Normal || !item.is_collapsed())
            .map(|item| {
                constants.axes.cross_size(item.hypothetical_size)
                    + constants.axes.cross_edge_sum(item.margin)
            })
            .reduce(Tree::Scalar::max)
    {
        // The common cross offer can change flex bases as well as line widths.
        constants.cross_content_offer = Some(offer.max(Tree::Scalar::ZERO));
        return recollect_items(tree, collected_items, constants, run_mode, round)
            .map(std::borrow::Cow::Owned);
    }
    Ok(std::borrow::Cow::Borrowed(collected_items))
}

fn resolve_flex_round<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    constants: &Constants<Tree::Scalar>,
    collected_items: &[CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>],
    collapsed_struts: Option<&CollapsedFlexStruts<Tree::Node, Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ResolvedFlexRound<Tree>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let round = if collapsed_struts.is_some() {
        FlexLineCollectionRound::Collapsed
    } else {
        FlexLineCollectionRound::Normal
    };
    let collect = |constants| {
        collect_flex_lines(collected_items, constants, round)
            .map_err(|reason| crate::inline_wrapping::wrapping_error(node, reason))
    };
    let mut lines = collect(constants)?;
    let mut collapsed =
        collapsed_struts.map(|struts| struts.prepare_second_round(collected_items, &lines));
    let (sizing_items, sizing_lines) = collapsed
        .as_ref()
        .map_or((collected_items, lines.as_slice()), |(items, lines)| {
            (items.as_slice(), lines.as_slice())
        });
    let mut layout_constants =
        resolved_layout_constants(tree, node, constants, sizing_items, sizing_lines)?;
    if constants.wraps
        && !collected_items.is_empty()
        && flex_line_collection_size(constants) != flex_line_collection_size(&layout_constants)
    {
        // Intrinsic and auto sizing can use provisional lines. Placement and
        // collapsed struts use the accepted inner main size.
        lines = collect(&layout_constants)?;
        collapsed =
            collapsed_struts.map(|struts| struts.prepare_second_round(collected_items, &lines));
    }
    let (layout_items, layout_lines) = collapsed
        .as_mut()
        .map_or((collected_items, &mut lines), |(items, lines)| {
            (items.as_slice(), lines)
        });
    let mut resolved_items =
        resolve_lines(tree, node, layout_items, layout_lines, &layout_constants)?;
    let cross_layout_constants = resolved_cross_layout_constants(&layout_constants, layout_lines);
    if cross_layout_constants.node_inner_size != layout_constants.node_inner_size {
        layout_constants = cross_layout_constants;
        resolved_items = resolve_lines(tree, node, layout_items, layout_lines, &layout_constants)?;
    } else {
        layout_constants = cross_layout_constants;
    }
    Ok((
        layout_constants,
        resolved_items,
        collapsed.map_or(lines, |(_, lines)| lines),
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreferredInlinePhase {
    Ordinary,
    Discover,
    Accepted,
}

#[derive(Clone, Copy)]
struct OwnRatioCross<S: LayoutScalar> {
    value: AspectRatioOf<S>,
    sizing_box_insets: Size<S>,
    needs_content_minimum: bool,
}

impl<S: LayoutScalar> OwnRatioCross<S> {
    fn for_preferred_inline(
        constants: &Constants<S>,
        style: &FlexContainerProjection<'_, S>,
    ) -> Option<Self> {
        let axes = constants.axes;
        let value = (*style.common.aspect_ratio)?;
        let preferred = axes.cross_size(Size::new(
            &style.common.size.width,
            &style.common.size.height,
        ));
        if constants.preferred_inline_phase == PreferredInlinePhase::Ordinary
            || preferred.is_max_content()
            || axes.cross_size(constants.node_outer_size).is_some()
        {
            return None;
        }

        let cross_overflow = axes.cross_size(Size::new(
            style.common.overflow.x(),
            style.common.overflow.y(),
        ));
        let minimum = axes.cross_size(Size::new(
            &style.common.min_size.width,
            &style.common.min_size.height,
        ));
        let sizing_box_insets = if style.common.box_sizing == BoxSizing::ContentBox {
            constants.padding_border_size
        } else {
            Size::ZERO
        };
        Some(Self {
            value,
            sizing_box_insets,
            needs_content_minimum: minimum.is_auto()
                && !style.common.item_is_replaced
                && !cross_overflow.is_scrollable(),
        })
    }
}

#[derive(Clone, Copy)]
struct Constants<S: LayoutScalar> {
    flow_axes: crate::geometry::FlowAxes,
    axes: FlexAxes,
    node_outer_size: Size<Option<S>>,
    node_inner_size: Size<Option<S>>,
    preferred_inline_phase: PreferredInlinePhase,
    cross_content_offer: Option<S>,
    percentage_parent_size: Size<Option<S>>,
    physical_baseline_demand: bool,
    min_outer_size: Size<Option<S>>,
    max_outer_size: Size<Option<S>>,
    max_inner_size: Size<Option<S>>,
    border: Edges<S>,
    padding: Edges<S>,
    padding_border_size: Size<S>,
    scrollport_inset: Edges<S>,
    non_gutter_box_inset: Edges<S>,
    content_box_inset: Edges<S>,
    settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState,
    gap: Size<S>,
    gap_input: Size<LengthOf<S>>,
    // Entry-qualified own dimensions for this scrollbar pass. None denotes a
    // cyclic percentage gap, which uses numeric zero, not a missing basis.
    gap_percentage_basis: Size<Option<S>>,
    align_items: AlignItems,
    authored_align_content: Option<AlignContent>,
    align_content: AlignContent,
    authored_justify_content: Option<AlignContent>,
    justify_content: AlignContent,
    wraps: bool,
    sizing_request: crate::SizingRequestOf<S>,
    positioned_fallback_limits: Size<crate::OrthogonalFallbackLimitOf<S>>,
    available_main: AxisSizingRequestOf<S>,
}

impl<S: LayoutScalar> Constants<S> {
    fn new<'input, Tree, M>(
        tree: &Tree,
        node: <Tree as Traverse>::Node,
        style: &FlexContainerProjection<'_, S>,
        scroll_box_projection: crate::scroll::ScrollBoxProjection<'_, S>,
        input: ComputeInputOf<S>,
        contained_size: Size<Option<S>>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        let flow_axes = style.common.flow_axes;
        let axes = FlexAxes::new(flow_axes, style.flex_direction, style.flex_wrap);
        let (padding, border) = resolve_containing_padding_border(
            input.containing_flow_axes(),
            input.parent(),
            *style.common.padding,
            *style.common.border,
            resolve_length_or_zero,
            |edges| edges.transpose_with_node(tree, node),
        )?;
        let padding_border = (padding + border).sum_axes();
        let box_sizing_adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
            padding_border
        } else {
            Size::<S>::ZERO
        };

        let preferred = Size::new(&style.common.size.width, &style.common.size.height);
        let row_preferred_size = input.sizing_mode() == SizingMode::InherentSize
            && axes.main_logical_axis() == LogicalAxis::Inline;
        let max_content_main = row_preferred_size && axes.main_size(preferred).is_max_content();
        let max_content_cross = row_preferred_size && axes.cross_size(preferred).is_max_content();
        let (style_size, min_size, max_size) = match input.sizing_mode() {
            SizingMode::ContentSize => (Size::NONE, Size::NONE, Size::NONE),
            SizingMode::InherentSize => {
                let resolve_preferred = |value, axis, basis| {
                    if max_content_main && axis == axes.main_physical_axis()
                        || max_content_cross && axis == axes.cross_physical_axis()
                    {
                        Ok(None)
                    } else {
                        resolve_preferred_optional(value, SizingAlgorithm::Flex, axis, basis, true)
                    }
                };
                let style_size = Size::new(
                    resolve_preferred(
                        preferred.width,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                    ),
                    resolve_preferred(
                        preferred.height,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                    ),
                )
                .transpose_with_node(tree, node)?;
                let min_size = Size::new(
                    resolve_minimum_optional(
                        &style.common.min_size.width,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        true,
                    ),
                    resolve_minimum_optional(
                        &style.common.min_size.height,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        true,
                    ),
                )
                .transpose_with_node(tree, node)?;
                let max_size = Size::new(
                    resolve_maximum_optional(
                        &style.common.max_size.width,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        true,
                    ),
                    resolve_maximum_optional(
                        &style.common.max_size.height,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        true,
                    ),
                )
                .transpose_with_node(tree, node)?;
                let ratio = *style.common.aspect_ratio;
                let (style_size, min_size, max_size) = if max_content_main || max_content_cross {
                    let (min_size, max_size) = ratio.map_or((min_size, max_size), |ratio| {
                        transfer_aspect_ratio_limits(style_size, min_size, max_size, ratio.get())
                    });
                    (style_size, min_size, max_size)
                } else {
                    (
                        style_size.apply_aspect_ratio(ratio),
                        min_size.apply_aspect_ratio(ratio),
                        max_size.apply_aspect_ratio(ratio),
                    )
                };
                (
                    style_size.add_optional(box_sizing_adjustment),
                    min_size.add_optional(box_sizing_adjustment),
                    max_size.add_optional(box_sizing_adjustment),
                )
            }
        };
        let min_max_definite_size = min_size.zip_map(max_size, |min, max| match (min, max) {
            (Some(min), Some(max)) if max <= min => Some(min),
            _ => None,
        });
        let definite_outer_size = input
            .known()
            .or(min_max_definite_size
                .or(style_size.clamp_max_before_min_optional(min_size, max_size)))
            .max_optional(padding_border.map(Some));
        let node_outer_size = contained_size.or(definite_outer_size);
        let mut entry_axes = input.sizing_request().axes();
        // Preferred intrinsic sizes use Flex contributions independently of the
        // finite containing offer. An already imposed size retains its request.
        let intrinsic = preferred_intrinsic_request();
        let preferred_inline_phase = if !max_content_main {
            PreferredInlinePhase::Ordinary
        } else if axes.main_size(node_outer_size).is_some() {
            PreferredInlinePhase::Accepted
        } else {
            PreferredInlinePhase::Discover
        };
        if preferred_inline_phase == PreferredInlinePhase::Discover {
            entry_axes = axes.with_main_size(entry_axes, intrinsic);
        }
        if max_content_cross && axes.cross_size(node_outer_size).is_none() {
            entry_axes = axes.with_cross_size(entry_axes, intrinsic);
        }
        let margin_debit = if input.run_mode() == RunMode::PerformRootLayout
            && input.parent_formatting_context() == ParentFormattingContext::NoParent
        {
            let unresolved = node_outer_size.zip_map(entry_axes, |size, request| {
                size.is_none() && crate::layout_math::normal_available_size(request).is_some()
            });
            if unresolved.width || unresolved.height {
                input
                    .containing_flow_axes()
                    .zip_physical_edges_with_inline_extent(
                        *style.common.margin,
                        input.parent(),
                        resolve_auto_or_zero,
                    )
                    .transpose_with_node(tree, node)?
                    .sum_axes()
                    .zip_map(
                        unresolved,
                        |margin, unresolved| {
                            if unresolved { margin } else { S::ZERO }
                        },
                    )
            } else {
                Size::ZERO
            }
        } else {
            Size::ZERO
        };
        let border_box_axes = if margin_debit == Size::ZERO {
            entry_axes
        } else {
            crate::layout_math::transpose_axis_requests_at_node(
                node,
                entry_axes.zip_map(margin_debit, crate::layout_math::inset_axis_request),
            )?
        };
        let mut scroll_box_source = CanonicalScrollBoxSourceOf::from_projection(
            scroll_box_projection,
            Size::ZERO,
            border,
            padding,
            input.settled_auto_scrollbars(),
        );
        let unconstrained_scroll_box_size = padding_border
            + Size::splat(
                scroll_box_source.scrollbar_width.get() + scroll_box_source.scrollbar_width.get(),
            );
        let scroll_box_size = node_outer_size
            .or(border_box_axes.map(|request| {
                request
                    .space()
                    .definite_size()
                    .map(crate::NonNegativeFiniteOf::get)
            }))
            .or(max_size)
            .unwrap_or(unconstrained_scroll_box_size)
            .zip_map(padding_border, |size, minimum| size.max(minimum));
        scroll_box_source.border_box_size = scroll_box_size;
        let scroll_box = canonical_scroll_box_from_source(scroll_box_source)
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        let scrollport_inset = scroll_box.effective_border() + scroll_box.effective_gutter();
        let non_gutter_box_inset = scroll_box.effective_border() + scroll_box.effective_padding();
        let content_box_inset = scroll_box.content_box_inset();
        let content_box_inset_size = content_box_inset.sum_axes();
        let content_axes = crate::layout_math::transpose_axis_requests_at_node(
            node,
            entry_axes.zip_map(
                content_box_inset_size + margin_debit,
                crate::layout_math::inset_axis_request,
            ),
        )?;
        let node_inner_size = node_outer_size
            .sub_optional_unchecked(content_box_inset_size)
            .max_optional(Size::<S>::ZERO.map(Some));
        let percentage_parent_size = input.descendant_percentage_basis(
            node_inner_size.zip_map(definite_outer_size, |inner, definite| definite.and(inner)),
        );
        let max_inner_size = max_size
            .sub_optional_unchecked(content_box_inset_size)
            .max_optional(Size::<S>::ZERO.map(Some));
        let gap = style
            .gap
            .zip_map(percentage_parent_size, |length, basis| {
                resolve_length_or_zero(length, Some(basis.unwrap_or(S::ZERO)))
            })
            .transpose_with_node(tree, node)?;

        let fallback_source = crate::scroll::MeasuredLeafContentBoxInsetSourceOf {
            flow_axes,
            computed_overflow: scroll_box_source.computed_overflow,
            item_is_replaced: scroll_box_source.item_is_replaced,
            scrollbar_gutter: scroll_box_source.scrollbar_gutter,
            scrollbar_width: scroll_box_source.scrollbar_width,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
            padding,
            border,
        };
        let fixed_outer = if input.sizing_mode() == SizingMode::InherentSize {
            definite_outer_size
        } else {
            input.known()
        };
        let fallback_sources = crate::layout_math::qualify_fixed_fallback_sources(
            input,
            SizingAlgorithm::Flex,
            &style.common,
            fixed_outer,
            box_sizing_adjustment,
            crate::scroll::UsedOverflow::from_computed(
                style.common.overflow,
                style.common.item_is_replaced,
            )
            .is_scroll_container(),
        )
        .map_err(|error| crate::error::sizing_resolution_error(node, error))?;
        let fallback = crate::layout_math::child_orthogonal_fallback(
            input.sizing_request().orthogonal_fallback(),
            fallback_sources.definite_outer,
            fallback_sources.minimum_outer,
            fallback_sources.maximum_outer,
            fallback_source,
        )
        .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?;
        let positioned_fallback_limits = crate::layout_math::containing_orthogonal_fallback_limits(
            fallback_sources.minimum_outer,
            fallback_sources.maximum_outer,
            crate::scroll::measured_leaf_content_box_inset(
                crate::scroll::MeasuredLeafContentBoxInsetSourceOf {
                    padding: Edges::ZERO,
                    ..fallback_source
                },
            )
            .sum_axes(),
        )
        .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?;
        Ok(Self {
            flow_axes,
            axes,
            node_outer_size,
            node_inner_size,
            preferred_inline_phase,
            cross_content_offer: None,
            percentage_parent_size,
            physical_baseline_demand: input.physical_baseline_demand(),
            min_outer_size: min_size,
            max_outer_size: max_size,
            max_inner_size,
            border,
            padding,
            padding_border_size: padding_border,
            scrollport_inset,
            non_gutter_box_inset,
            content_box_inset,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
            gap,
            gap_input: *style.gap,
            gap_percentage_basis: percentage_parent_size,
            align_items: style.align_items.unwrap_or(AlignItems::Stretch),
            authored_align_content: style.align_content,
            align_content: style.align_content.unwrap_or(AlignContent::Stretch),
            authored_justify_content: style.justify_content,
            justify_content: style.justify_content.unwrap_or(AlignContent::FlexStart),
            wraps: matches!(
                style.flex_wrap,
                super::FlexWrap::Wrap | super::FlexWrap::WrapReverse
            ),
            sizing_request: input
                .sizing_request()
                .with_axes(content_axes)
                .with_orthogonal_fallback(fallback),
            positioned_fallback_limits,
            available_main: axes.main_size(content_axes),
        })
    }

    fn is_wrapping_column(&self) -> bool {
        self.wraps && self.axes.main_logical_axis() == LogicalAxis::Block
    }

    fn cross_content_extent(&self) -> Option<S> {
        self.cross_content_offer
            .or(self.axes.cross_size(self.node_inner_size))
    }

    fn complete_inline_contents(
        self,
        accepted: CanonicalScrollBoxOf<S>,
        requested_inline_operation: crate::SizingOperation,
        settle_cross: bool,
    ) -> Result<Option<Self>, LengthResolutionStatus<S>> {
        let axes = self.flow_axes;
        let mut percentage_parent = axes.logical_size(self.percentage_parent_size);
        let normal = requested_inline_operation == crate::SizingOperation::Normal;
        if !settle_cross
            && self.preferred_inline_phase != PreferredInlinePhase::Discover
            && (!normal || percentage_parent.inline.is_some())
        {
            return Ok(None);
        }
        // Use the accepted unrounded inline content extent. All block sizing,
        // availability and maximum authority still come from the entry context.
        let mut outer = axes.logical_size(self.node_outer_size);
        let mut inner = axes.logical_size(self.node_inner_size);
        let accepted_inline = axes.logical_size(accepted.content_box().size()).inline;
        outer.inline = Some(axes.logical_size(accepted.border_box().size()).inline);
        inner.inline = Some(accepted_inline);
        if normal {
            percentage_parent.inline = inner.inline;
        }
        let inline_edges = |original, used| {
            let mut original = axes.logical_edges(original);
            let used = axes.logical_edges(used);
            original.inline_start = used.inline_start;
            original.inline_end = used.inline_end;
            axes.physical_edges(original)
        };
        let mut contents = self;
        contents.node_outer_size = axes.physical_size(outer);
        contents.node_inner_size = axes.physical_size(inner);
        contents.percentage_parent_size = axes.physical_size(percentage_parent);
        let mut requests = axes.logical_size(self.sizing_request.axes());
        if self.preferred_inline_phase == PreferredInlinePhase::Discover {
            // A completed property measurement does not select the caller's
            // sizing operation or establish a formal percentage basis.
            requests.inline = AxisSizingRequestOf::new(
                crate::layout_math::checked_available_space(accepted_inline)?,
                requested_inline_operation,
            );
            contents.sizing_request = self.sizing_request.with_axes(axes.physical_size(requests));
            contents.available_main = self.axes.main_size(contents.sizing_request.axes());
            contents.preferred_inline_phase = PreferredInlinePhase::Accepted;
        }
        contents.scrollport_inset = inline_edges(
            self.scrollport_inset,
            accepted.effective_border() + accepted.effective_gutter(),
        );
        contents.content_box_inset =
            inline_edges(self.content_box_inset, accepted.content_box_inset());
        contents.max_inner_size = self
            .max_outer_size
            .sub_optional_unchecked(contents.content_box_inset.sum_axes())
            .max_optional(Size::ZERO.map(Some));
        Ok(Some(contents))
    }

    fn with_final_scroll_box(mut self, scroll_box: CanonicalScrollBoxOf<S>) -> Self {
        self.node_outer_size = scroll_box.border_box().size().map(Some);
        self.node_inner_size = scroll_box.content_box().size().map(Some);
        self.percentage_parent_size = self
            .percentage_parent_size
            .zip_map(scroll_box.content_box().size(), |original, used| {
                original.map(|_| used)
            });
        self.scrollport_inset = scroll_box.effective_border() + scroll_box.effective_gutter();
        self.non_gutter_box_inset = scroll_box.effective_border() + scroll_box.effective_padding();
        self.content_box_inset = scroll_box.content_box_inset();
        self
    }
}

/// Resolved flex main/cross roles for one container.
///
/// This is the sole flex owner for translating a container's logical flow into
/// physical axes, sides, and progressions. It is derived only from the
/// container's `FlowAxes`, `FlexDirection`, and `FlexWrap`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FlexAxes {
    flow_axes: FlowAxes,
    main_logical_axis: LogicalAxis,
    cross_logical_axis: LogicalAxis,
    main_physical_axis: PhysicalAxis,
    cross_physical_axis: PhysicalAxis,
    main_start_side: PhysicalSide,
    main_end_side: PhysicalSide,
    cross_start_side: PhysicalSide,
    cross_end_side: PhysicalSide,
    main_reversed: bool,
    cross_reversed: bool,
    main_progression: PhysicalProgression,
    cross_progression: PhysicalProgression,
}

impl FlexAxes {
    #[must_use]
    pub(crate) const fn new(
        flow_axes: FlowAxes,
        flex_direction: FlexDirection,
        flex_wrap: FlexWrap,
    ) -> Self {
        let main_reversed = flex_direction.is_reverse();
        let cross_reversed = matches!(flex_wrap, FlexWrap::WrapReverse);
        let (
            main_logical_axis,
            cross_logical_axis,
            main_physical_axis,
            cross_physical_axis,
            normal_main_start_side,
            normal_main_end_side,
            normal_cross_start_side,
            normal_cross_end_side,
        ) = if flex_direction.is_row() {
            (
                LogicalAxis::Inline,
                LogicalAxis::Block,
                flow_axes.inline_axis(),
                flow_axes.block_axis(),
                flow_axes.inline_start(),
                flow_axes.inline_end(),
                flow_axes.block_start(),
                flow_axes.block_end(),
            )
        } else {
            (
                LogicalAxis::Block,
                LogicalAxis::Inline,
                flow_axes.block_axis(),
                flow_axes.inline_axis(),
                flow_axes.block_start(),
                flow_axes.block_end(),
                flow_axes.inline_start(),
                flow_axes.inline_end(),
            )
        };
        let (main_start_side, main_end_side) = if main_reversed {
            (normal_main_end_side, normal_main_start_side)
        } else {
            (normal_main_start_side, normal_main_end_side)
        };
        let (cross_start_side, cross_end_side) = if cross_reversed {
            (normal_cross_end_side, normal_cross_start_side)
        } else {
            (normal_cross_start_side, normal_cross_end_side)
        };

        Self {
            flow_axes,
            main_logical_axis,
            cross_logical_axis,
            main_physical_axis,
            cross_physical_axis,
            main_start_side,
            main_end_side,
            cross_start_side,
            cross_end_side,
            main_reversed,
            cross_reversed,
            main_progression: Self::reverse_progression(
                flow_axes.physical_axis_progression(main_physical_axis),
                main_reversed,
            ),
            cross_progression: Self::reverse_progression(
                flow_axes.physical_axis_progression(cross_physical_axis),
                cross_reversed,
            ),
        }
    }

    #[must_use]
    pub(crate) const fn flow_axes(self) -> FlowAxes {
        self.flow_axes
    }

    #[must_use]
    pub(crate) const fn flow_direction(self) -> Direction {
        self.flow_axes.direction()
    }

    #[must_use]
    pub(crate) const fn scroll_origin_axes(self) -> ScrollOriginAxes {
        let main = if self.main_reversed {
            ScrollOriginProgression::FlowStartward
        } else {
            ScrollOriginProgression::FlowEndward
        };
        let cross = if self.cross_reversed {
            ScrollOriginProgression::FlowStartward
        } else {
            ScrollOriginProgression::FlowEndward
        };
        match self.main_logical_axis {
            LogicalAxis::Inline => ScrollOriginAxes::new(main, cross),
            LogicalAxis::Block => ScrollOriginAxes::new(cross, main),
        }
    }

    #[must_use]
    pub(crate) const fn main_logical_axis(self) -> LogicalAxis {
        self.main_logical_axis
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn cross_logical_axis(self) -> LogicalAxis {
        self.cross_logical_axis
    }

    #[must_use]
    pub(crate) const fn main_physical_axis(self) -> PhysicalAxis {
        self.main_physical_axis
    }

    #[must_use]
    pub(crate) const fn cross_physical_axis(self) -> PhysicalAxis {
        self.cross_physical_axis
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn main_start_side(self) -> PhysicalSide {
        self.main_start_side
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn main_end_side(self) -> PhysicalSide {
        self.main_end_side
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn cross_start_side(self) -> PhysicalSide {
        self.cross_start_side
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn cross_end_side(self) -> PhysicalSide {
        self.cross_end_side
    }

    #[must_use]
    pub(crate) const fn main_is_reversed(self) -> bool {
        self.main_reversed
    }

    #[must_use]
    pub(crate) const fn cross_is_reversed(self) -> bool {
        self.cross_reversed
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn main_progression(self) -> PhysicalProgression {
        self.main_progression
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn cross_progression(self) -> PhysicalProgression {
        self.cross_progression
    }

    #[must_use]
    pub(crate) fn main_size<T>(self, size: Size<T>) -> T {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => size.width,
            PhysicalAxis::Vertical => size.height,
        }
    }

    #[must_use]
    pub(crate) fn cross_size<T>(self, size: Size<T>) -> T {
        match self.cross_physical_axis {
            PhysicalAxis::Horizontal => size.width,
            PhysicalAxis::Vertical => size.height,
        }
    }

    #[must_use]
    pub(crate) fn size_from_main_cross<T>(self, main: T, cross: T) -> Size<T> {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => Size::new(main, cross),
            PhysicalAxis::Vertical => Size::new(cross, main),
        }
    }

    #[must_use]
    pub(crate) fn with_main_size<T>(self, size: Size<T>, value: T) -> Size<T> {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => Size::new(value, size.height),
            PhysicalAxis::Vertical => Size::new(size.width, value),
        }
    }

    #[must_use]
    pub(crate) fn with_cross_size<T>(self, size: Size<T>, value: T) -> Size<T> {
        match self.cross_physical_axis {
            PhysicalAxis::Horizontal => Size::new(value, size.height),
            PhysicalAxis::Vertical => Size::new(size.width, value),
        }
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn main_point<T>(self, point: Point<T>) -> T {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => point.x,
            PhysicalAxis::Vertical => point.y,
        }
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn cross_point<T>(self, point: Point<T>) -> T {
        match self.cross_physical_axis {
            PhysicalAxis::Horizontal => point.x,
            PhysicalAxis::Vertical => point.y,
        }
    }

    #[must_use]
    pub(crate) fn point_from_main_cross<T>(self, main: T, cross: T) -> Point<T> {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => Point::new(main, cross),
            PhysicalAxis::Vertical => Point::new(cross, main),
        }
    }

    #[must_use]
    pub(crate) fn main_start_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.main_start_side)
    }

    #[must_use]
    pub(crate) fn main_end_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.main_end_side)
    }

    #[must_use]
    pub(crate) fn cross_start_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.cross_start_side)
    }

    #[must_use]
    pub(crate) fn cross_end_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.cross_end_side)
    }

    #[must_use]
    pub(crate) fn normal_main_start_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.normal_axis_start_side(self.main_logical_axis))
    }

    #[must_use]
    pub(crate) fn normal_main_end_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.normal_axis_end_side(self.main_logical_axis))
    }

    #[must_use]
    pub(crate) fn normal_cross_start_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.normal_axis_start_side(self.cross_logical_axis))
    }

    #[must_use]
    pub(crate) fn normal_cross_end_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.normal_axis_end_side(self.cross_logical_axis))
    }

    #[must_use]
    pub(crate) fn main_edge_sum<S: LayoutScalar>(self, edges: Edges<S>) -> S {
        self.main_start_edge(edges) + self.main_end_edge(edges)
    }

    #[must_use]
    pub(crate) fn cross_edge_sum<S: LayoutScalar>(self, edges: Edges<S>) -> S {
        self.cross_start_edge(edges) + self.cross_end_edge(edges)
    }

    pub(crate) fn set_main_start_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.main_start_side, value);
    }

    pub(crate) fn set_main_end_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.main_end_side, value);
    }

    #[cfg(test)]
    pub(crate) fn set_cross_start_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.cross_start_side, value);
    }

    #[cfg(test)]
    pub(crate) fn set_cross_end_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.cross_end_side, value);
    }

    pub(crate) fn set_normal_cross_start_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(
            edges,
            self.normal_axis_start_side(self.cross_logical_axis),
            value,
        );
    }

    pub(crate) fn set_normal_cross_end_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(
            edges,
            self.normal_axis_end_side(self.cross_logical_axis),
            value,
        );
    }

    #[must_use]
    pub(crate) const fn main_requested_axis(self) -> RequestedAxis {
        Self::requested_axis_for(self.main_physical_axis)
    }

    #[must_use]
    pub(crate) const fn cross_requested_axis(self) -> RequestedAxis {
        Self::requested_axis_for(self.cross_physical_axis)
    }

    #[must_use]
    pub(crate) fn main_size_from_cross_aspect<S: LayoutScalar>(
        self,
        cross: S,
        aspect_ratio: AspectRatioOf<S>,
    ) -> S {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => cross * aspect_ratio.get(),
            PhysicalAxis::Vertical => cross / aspect_ratio.get(),
        }
    }

    fn main_size_from_cross_in_sizing_box<S: LayoutScalar>(
        self,
        border_box_cross: S,
        aspect_ratio: AspectRatioOf<S>,
        sizing_box_insets: Size<S>,
    ) -> Result<S, crate::LengthResolutionStatus<S>> {
        let cross = (border_box_cross - self.cross_size(sizing_box_insets)).max(S::ZERO);
        let main = self.main_size_from_cross_aspect(cross, aspect_ratio)
            + self.main_size(sizing_box_insets);
        if main.is_finite() {
            Ok(main)
        } else {
            Err(crate::LengthResolutionStatus::InvalidNumeric { value: main })
        }
    }

    fn cross_size_from_main_in_sizing_box<S: LayoutScalar>(
        self,
        border_box_main: S,
        aspect_ratio: AspectRatioOf<S>,
        sizing_box_insets: Size<S>,
    ) -> Result<S, LengthResolutionStatus<S>> {
        let main = (border_box_main - self.main_size(sizing_box_insets)).max(S::ZERO);
        let cross = match self.main_physical_axis {
            PhysicalAxis::Horizontal => main / aspect_ratio.get(),
            PhysicalAxis::Vertical => main * aspect_ratio.get(),
        } + self.cross_size(sizing_box_insets);
        if cross.is_finite() {
            Ok(cross)
        } else {
            Err(LengthResolutionStatus::InvalidNumeric { value: cross })
        }
    }

    #[must_use]
    pub(crate) fn main_position_from_start<
        T: core::ops::Add<Output = T> + core::ops::Sub<Output = T>,
    >(
        self,
        container: Size<T>,
        start_inset: T,
        offset: T,
        item_size: T,
        relative_offset: T,
    ) -> T {
        let coordinate = start_inset + offset + relative_offset;
        match self.main_progression {
            PhysicalProgression::Increasing => coordinate,
            PhysicalProgression::Decreasing => self.main_size(container) - coordinate - item_size,
        }
    }

    #[must_use]
    pub(crate) fn cross_position_from_start<
        T: core::ops::Add<Output = T> + core::ops::Sub<Output = T>,
    >(
        self,
        container: Size<T>,
        start_inset: T,
        offset: T,
        item_size: T,
        relative_offset: T,
    ) -> T {
        let coordinate = start_inset + offset + relative_offset;
        match self.cross_progression {
            PhysicalProgression::Increasing => coordinate,
            PhysicalProgression::Decreasing => self.cross_size(container) - coordinate - item_size,
        }
    }

    #[must_use]
    pub(crate) fn main_offset_from_normal_flow<S: LayoutScalar>(self, offset: S) -> S {
        if self.main_reversed { -offset } else { offset }
    }

    #[must_use]
    pub(crate) fn cross_offset_from_normal_flow<S: LayoutScalar>(self, offset: S) -> S {
        if self.cross_reversed { -offset } else { offset }
    }

    #[must_use]
    pub(crate) fn main_position_from_normal_start<S: LayoutScalar>(
        self,
        container: Size<S>,
        offset: S,
        item_size: S,
    ) -> S {
        let flex_offset = if self.main_reversed {
            self.main_size(container) - offset - item_size
        } else {
            offset
        };
        self.main_position_from_start(container, S::ZERO, flex_offset, item_size, S::ZERO)
    }

    #[must_use]
    pub(crate) fn cross_position_from_normal_start<S: LayoutScalar>(
        self,
        container: Size<S>,
        offset: S,
        item_size: S,
    ) -> S {
        let flex_offset = if self.cross_reversed {
            self.cross_size(container) - offset - item_size
        } else {
            offset
        };
        self.cross_position_from_start(container, S::ZERO, flex_offset, item_size, S::ZERO)
    }

    #[must_use]
    const fn normal_axis_start_side(self, axis: LogicalAxis) -> PhysicalSide {
        match axis {
            LogicalAxis::Inline => self.flow_axes.inline_start(),
            LogicalAxis::Block => self.flow_axes.block_start(),
        }
    }

    #[must_use]
    const fn normal_axis_end_side(self, axis: LogicalAxis) -> PhysicalSide {
        self.normal_axis_start_side(axis).opposite()
    }

    const fn reverse_progression(
        progression: PhysicalProgression,
        reversed: bool,
    ) -> PhysicalProgression {
        if !reversed {
            return progression;
        }

        match progression {
            PhysicalProgression::Increasing => PhysicalProgression::Decreasing,
            PhysicalProgression::Decreasing => PhysicalProgression::Increasing,
        }
    }

    fn set_edge_at_side<T>(self, edges: &mut Edges<T>, side: PhysicalSide, value: T) {
        match side {
            PhysicalSide::Top => edges.top = value,
            PhysicalSide::Right => edges.right = value,
            PhysicalSide::Bottom => edges.bottom = value,
            PhysicalSide::Left => edges.left = value,
        }
    }

    const fn requested_axis_for(axis: PhysicalAxis) -> RequestedAxis {
        match axis {
            PhysicalAxis::Horizontal => RequestedAxis::Horizontal,
            PhysicalAxis::Vertical => RequestedAxis::Vertical,
        }
    }
}

#[expect(
    clippy::type_complexity,
    reason = "the private flex resolver preserves node, scalar, and provider error types"
)]
fn resolve_lines<'input, Tree, M>(
    tree: &mut Tree,
    container_node: <Tree as Traverse>::Node,
    items: &[CollectedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>],
    lines: &mut [FlexLine<Tree::Scalar>],
    constants: &Constants<Tree::Scalar>,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    Vec<ResolvedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<'input, M>,
{
    let mut resolved_items = items
        .iter()
        .copied()
        .map(ResolvedFlexItem::from)
        .collect::<Vec<_>>();
    let cross_gap = constants.axes.cross_size(constants.gap);
    let mut cross_cursor = Tree::Scalar::ZERO;
    let single_line = !constants.wraps;

    for line in &mut *lines {
        let invalid_main = |value| {
            crate::error::sizing_resolution_error(
                container_node,
                LengthResolutionStatus::InvalidNumeric { value }.into(),
            )
        };
        let wide = |value| DistributionNumber::finite(value).map_err(invalid_main);
        resolve_flexible_lengths(&mut resolved_items[line.start..line.end], constants)
            .map_err(invalid_main)?;

        let item_count = line.end - line.start;
        if item_count == 0 && line.contains_collapsed_slot {
            line.max_item_cross_size = Tree::Scalar::ZERO;
            line.main_size = DistributionNumber::ZERO;
            line.main_overflows = false;
            line.cross_size = if single_line {
                constants
                    .axes
                    .cross_size(constants.node_inner_size)
                    .unwrap_or(line.strut_floor)
            } else {
                line.strut_floor
            };
            line.offset_cross = cross_cursor;
            cross_cursor = cross_cursor + line.cross_size + cross_gap;
            continue;
        }
        resolve_main_axis_auto_margins(&mut resolved_items[line.start..line.end], constants)
            .map_err(invalid_main)?;
        let free_space = line_free_space(&resolved_items[line.start..line.end], constants)
            .map_err(invalid_main)?;
        let sign = free_space.sign();
        line.main_overflows = sign == core::cmp::Ordering::Less;
        let justify_content =
            alignment_fallback_from_sign(Some(sign), item_count, constants.justify_content);
        let main_gap = wide(constants.axes.main_size(constants.gap))?;
        let mut main_cursor = main_alignment_offset(
            free_space,
            item_count,
            main_gap,
            justify_content,
            constants.axes.main_is_reversed(),
            true,
        );
        let mut cross_size = Tree::Scalar::ZERO;

        for (index, item_index) in (line.start..line.end).enumerate() {
            if index > 0 {
                main_cursor = main_cursor
                    + main_alignment_offset(
                        free_space,
                        item_count,
                        main_gap,
                        justify_content,
                        constants.axes.main_is_reversed(),
                        false,
                    );
            }

            let item = &mut resolved_items[item_index];
            determine_hypothetical_cross_size(tree, item, constants)?;
            item.offset_main = main_cursor + wide(item.margin_main_start(constants))?;
            item.offset_cross = cross_cursor + constants.axes.cross_start_edge(item.margin);

            let margin = wide(item.margin_main_start(constants))?
                + wide(constants.axes.main_end_edge(item.margin))?;
            main_cursor = main_cursor + wide(constants.axes.main_size(item.target_size))? + margin;
            cross_size = Tree::Scalar::max(
                cross_size,
                constants.axes.cross_size(item.target_size)
                    + constants.axes.cross_edge_sum(item.margin),
            );
        }
        line.max_item_cross_size = cross_size;
        cross_size = Tree::Scalar::max(
            cross_size,
            line_cross_size(&resolved_items[line.start..line.end], constants),
        );
        cross_size = Tree::Scalar::max(cross_size, line.strut_floor);

        line.main_size = main_cursor;
        line.cross_size = if single_line {
            constants
                .axes
                .cross_size(constants.node_inner_size)
                .unwrap_or(cross_size)
        } else {
            cross_size
        };
        line.offset_cross = cross_cursor;
        align_items_on_cross_axis(
            &mut resolved_items[line.start..line.end],
            line.cross_size,
            cross_cursor,
            constants,
        );
        cross_cursor = cross_cursor + line.cross_size + cross_gap;
    }

    stretch_lines_on_cross_axis(&mut resolved_items, lines, constants);
    align_lines_on_cross_axis(&mut resolved_items, lines, constants);
    Ok(resolved_items)
}

fn determine_hypothetical_cross_size<'input, Tree, M>(
    tree: &mut Tree,
    item: &mut ResolvedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let axes = constants.axes;
    let padding_border = (item.padding + item.border).sum_axes();
    let padding_border_cross = axes.cross_size(padding_border);
    let minimum = axes.cross_size(item.min_size);
    let maximum = axes.cross_size(item.max_size);
    let clamp_cross = |size: Tree::Scalar| {
        size.clamp_max_before_min_optional(minimum, maximum)
            .max(padding_border_cross)
    };
    if let Some(authored_cross) = item.authored_cross_size {
        item.target_size = axes.with_cross_size(item.target_size, clamp_cross(authored_cross));
        if item.uses_native_cross_baseline(axes) && item.target_size != item.initial_output.size {
            let output =
                with_flex_item_projection::<Tree, M, _>(tree, item.node, |tree, style| {
                    compute_sized_item(tree, item, &style, constants, RunMode::ComputeSize)
                })?;
            item.baseline.refresh(output);
        }
        return Ok(());
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
        let cross = clamp_cross(axes.cross_size(item.initial_output.size));
        item.target_size = axes.with_cross_size(item.target_size, cross);
        return Ok(());
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
    let demand_baseline =
        constants.physical_baseline_demand || item.uses_native_cross_baseline(axes);
    let measure_cross = |tree: &mut Tree, mode, known_cross, cross_request, demand| {
        tree.compute_child(
            item.node,
            ComputeInputOf::for_child(
                RunMode::ComputeSize,
                mode,
                axes.cross_requested_axis(),
                axes.size_from_main_cross(Some(used_main), known_cross),
                constants.percentage_parent_size,
                ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::Flex),
                constants
                    .sizing_request
                    .with_axes(axes.size_from_main_cross(available_main, cross_request)),
            )
            .with_physical_baseline_demand(demand)
            .with_indefinite_percentage_axis(item.indefinite_percentage_axis)
            .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
        )
    };
    let cross = if let Some(ratio) = ratio {
        let transferred = axes
            .cross_size_from_main_in_sizing_box(used_main, ratio, ratio_box_insets)
            .map_err(|error| crate::error::sizing_resolution_error(item.node, error.into()))?;
        // Measure the automatic minimum without giving descendants a provisional
        // definite cross size.
        let cross = if needs_ratio_minimum {
            let min_request = crate::layout_math::with_available_extent(
                axes.cross_size(constants.sizing_request.axes())
                    .with_operation(crate::SizingOperation::MinContent),
                constants
                    .cross_content_offer
                    .or(axes.cross_size(constants.percentage_parent_size)),
            )
            .map_err(|error| crate::error::sizing_resolution_error(item.node, error.into()))?;
            let measured = measure_cross(tree, SizingMode::ContentSize, None, min_request, false)?;
            transferred.max(axes.cross_size(measured.size))
        } else {
            transferred
        };
        let cross = clamp_cross(cross);
        if item.uses_native_cross_baseline(axes) {
            item.baseline.refresh(measure_cross(
                tree,
                SizingMode::InherentSize,
                Some(cross),
                available_cross,
                true,
            )?);
        }
        cross
    } else {
        let measured = measure_cross(
            tree,
            SizingMode::InherentSize,
            None,
            available_cross,
            demand_baseline,
        )?;
        item.baseline.refresh(measured);
        clamp_cross(axes.cross_size(measured.size))
    };
    item.target_size = axes.with_cross_size(item.target_size, cross);
    Ok(())
}

#[derive(Clone, Copy)]
struct FlexContainerSizes<S: LayoutScalar> {
    output: Size<S>,
    intrinsic_content: Size<S>,
}

fn container_sizes<Node, S: LayoutScalar>(
    input: ComputeInputOf<S>,
    constants: &Constants<S>,
    resolved_items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
) -> FlexContainerSizes<S> {
    let line_cross_gap =
        constants.axes.cross_size(constants.gap) * S::from_usize(lines.len().saturating_sub(1));
    let content_main = intrinsic_content_main_size(input, constants, resolved_items, lines);
    let content_cross = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + line_cross_gap;
    let content_size = constants
        .axes
        .size_from_main_cross(content_main, content_cross);
    let outer_size = constants
        .node_outer_size
        .unwrap_or(content_size + constants.content_box_inset.sum_axes())
        .clamp_max_before_min_optional(constants.min_outer_size, constants.max_outer_size);
    let mut output_size = input
        .known()
        .or(constants.node_outer_size)
        .unwrap_or(outer_size)
        .max_optional(constants.padding_border_size.map(Some));
    if constants
        .axes
        .main_size(constants.node_outer_size)
        .is_none()
        && lines.len() > 1
        && let Some(available_main) = crate::layout_math::normal_available_size(
            constants.axes.main_size(input.sizing_request().axes()),
        )
    {
        output_size = constants.axes.with_main_size(
            output_size,
            constants.axes.main_size(output_size).max(available_main),
        );
    }

    FlexContainerSizes {
        output: output_size,
        intrinsic_content: content_size,
    }
}

fn resolve_length_or_zero<S: LayoutScalar>(
    length: LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    resolution_or_zero(length.resolve_with_status(basis))
}

fn resolve_auto_or_zero<S: LayoutScalar>(
    length: LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    resolution_or_zero(length.resolve_with_status(basis))
}

fn resolve_auto_optional<S: LayoutScalar>(
    length: LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    resolution_optional(length.resolve_with_status(basis))
}

#[cfg(test)]
mod fri08_c07_t03_optional_math_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(None, Some(scalar(12.0))).max_optional(Size::new(Some(scalar(9.0)), None)),
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
    fn fri08_c07_t03_optional_math_flex_componentwise_floors_preserve_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri08_c07_t03_optional_math_flex_componentwise_floors_preserve_f64() {
        characterize::<f64>();
    }
}

#[cfg(test)]
mod fri06_c13_t06_characterization_tests {
    use super::*;
    use crate::{
        LayoutErrorKindOf, LayoutErrorSiteOf, LayoutInvalidInputOf, LayoutOperation,
        LengthPercentageOf, NodeInputOf, ParentFormattingContext, RequestedAxis, WritingMode,
    };

    fn input<S: LayoutScalar>(
        containing_flow_axes: FlowAxes,
        parent: Size<Option<S>>,
    ) -> ComputeInputOf<S> {
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::ContentSize,
            RequestedAxis::Both,
            Size::NONE,
            parent,
            crate::ContainingLayoutContext::new(
                containing_flow_axes,
                ParentFormattingContext::NoParent,
            ),
            crate::SizingRequestOf::new(
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
        )
    }

    fn percentage_edges<S: LayoutScalar>() -> Edges<LengthOf<S>> {
        Edges::new(
            LengthOf::percent(S::from_f64(0.1)),
            LengthOf::percent(S::from_f64(0.2)),
            LengthOf::percent(S::from_f64(0.3)),
            LengthOf::percent(S::from_f64(0.4)),
        )
    }

    fn expected_percentage_edges<S: LayoutScalar>(basis: S) -> Edges<S> {
        Edges::new(
            S::from_f64(0.1) * basis,
            S::from_f64(0.2) * basis,
            S::from_f64(0.3) * basis,
            S::from_f64(0.4) * basis,
        )
    }

    fn characterize_constants<S: LayoutScalar>(largest: S) {
        crate::layout_math::assert_fri06_c13_t06_resolution_policy::<S>(
            resolution_or_zero,
            resolution_optional,
        );

        let border = Edges::new(
            LengthOf::px(S::from_f64(1.0)),
            LengthOf::px(S::from_f64(2.0)),
            LengthOf::px(S::from_f64(3.0)),
            LengthOf::px(S::from_f64(4.0)),
        );
        let expected_border = Edges::new(
            S::from_f64(1.0),
            S::from_f64(2.0),
            S::from_f64(3.0),
            S::from_f64(4.0),
        );
        let style = NodeInputOf {
            display: crate::Display::Flex,
            padding: percentage_edges(),
            border,
            ..NodeInputOf::default()
        };
        let mut owner =
            crate::test_support::layout_tree::OracleTreeOf::new().style(7, style.clone());
        let tree = owner.borrow_compute();

        for (flow, parent, expected_padding) in [
            (
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(200.0))),
                expected_percentage_edges(S::from_f64(100.0)),
            ),
            (
                FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(200.0))),
                expected_percentage_edges(S::from_f64(200.0)),
            ),
            (
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(None, Some(S::from_f64(200.0))),
                Edges::ZERO,
            ),
            (
                FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), None),
                Edges::ZERO,
            ),
        ] {
            let constants = super::input::with_flex_container_projection::<
                _,
                core::convert::Infallible,
                _,
            >(&tree, 7, |projection| {
                super::input::with_flex_scroll_projections::<_, core::convert::Infallible, _>(
                    &tree,
                    7,
                    |scroll_box, _| {
                        Constants::new::<_, core::convert::Infallible>(
                            &tree,
                            7,
                            &projection,
                            scroll_box,
                            input(flow, parent),
                            Size::NONE,
                        )
                    },
                )
            })
            .expect("flex constants edge characterization must resolve");
            assert_eq!(constants.padding, expected_padding);
            assert_eq!(constants.border, expected_border);
        }

        let positive_overflow = LengthOf::value(
            LengthPercentageOf::from_coefficients(largest, S::ONE)
                .expect("finite positive overflow coefficients"),
        );
        let negative_overflow = LengthOf::value(
            LengthPercentageOf::from_coefficients(-largest, -S::ONE)
                .expect("finite negative overflow coefficients"),
        );
        let failing_style = NodeInputOf {
            display: crate::Display::Flex,
            padding: Edges::new(
                LengthOf::ZERO,
                LengthOf::ZERO,
                LengthOf::ZERO,
                positive_overflow,
            ),
            border: Edges::new(
                negative_overflow,
                LengthOf::ZERO,
                LengthOf::ZERO,
                LengthOf::ZERO,
            ),
            ..NodeInputOf::default()
        };
        let mut failing_owner =
            crate::test_support::layout_tree::OracleTreeOf::new().style(7, failing_style.clone());
        let failing_tree = failing_owner.borrow_compute();
        let result = super::input::with_flex_container_projection::<_, core::convert::Infallible, _>(
            &failing_tree,
            7,
            |projection| {
                super::input::with_flex_scroll_projections::<_, core::convert::Infallible, _>(
                    &failing_tree,
                    7,
                    |scroll_box, _| {
                        Constants::new::<_, core::convert::Infallible>(
                            &failing_tree,
                            7,
                            &projection,
                            scroll_box,
                            input(
                                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                                Size::splat(Some(largest)),
                            ),
                            Size::NONE,
                        )
                    },
                )
            },
        );
        let error = match result {
            Ok(_) => panic!("padding failure must precede the distinct border failure"),
            Err(error) => error,
        };
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(7));
        assert_eq!(error.operation(), LayoutOperation::ValueResolution);
        crate::test_support::assert_error_kind(
            error.kind(),
            &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
                value: S::INFINITY,
            }),
        );
    }

    #[test]
    fn fri06_c13_t06_flex_resolution_edges_and_error_order_preserve_f32() {
        characterize_constants::<f32>(f32::MAX);
    }

    #[test]
    fn fri06_c13_t06_flex_resolution_edges_and_error_order_preserve_f64() {
        characterize_constants::<f64>(f64::MAX);
    }
}

#[cfg(test)]
mod fri06_c13_t05_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(Some(scalar(2.0)), Some(scalar(9.0)))
                .sub_optional_unchecked(Size::new(scalar(5.0), scalar(4.0))),
            Size::new(Some(scalar(-3.0)), Some(scalar(5.0)))
        );
        assert_eq!(
            Size::new(None, Some(scalar(9.0)))
                .sub_optional_unchecked(Size::new(scalar(5.0), scalar(4.0))),
            Size::new(None, Some(scalar(5.0)))
        );
        assert_eq!(
            Size::new(scalar(8.0), scalar(12.0)).clamp_max_before_min_optional(
                Size::new(Some(scalar(3.0)), None),
                Size::new(Some(scalar(10.0)), Some(scalar(11.0))),
            ),
            Size::new(scalar(8.0), scalar(11.0))
        );
        assert_eq!(
            Size::new(Some(scalar(5.0)), Some(scalar(5.0))).clamp_max_before_min_optional(
                Size::new(Some(scalar(10.0)), Some(scalar(10.0))),
                Size::new(Some(scalar(3.0)), Some(scalar(3.0))),
            ),
            Size::new(Some(scalar(10.0)), Some(scalar(10.0)))
        );
    }

    #[test]
    fn fri06_c13_t05_flex_unchecked_subtraction_and_clamp_order_preserve_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri06_c13_t05_flex_unchecked_subtraction_and_clamp_order_preserve_f64() {
        characterize::<f64>();
    }
}
