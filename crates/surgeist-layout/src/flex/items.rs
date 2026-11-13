use std::ops::ControlFlow;

use super::alignment::{BaselinePort, FlexItemBaseline};
use super::distribution_number::DistributionNumber;
use super::flexible_lengths::clamp_main_size_axes;
use super::input::{FlexItemProjection, flex_item_order_permutation, with_flex_item_projection};
use super::lines::FlexLineCollectionRound;
use super::scroll::{retain_flex_scroll_geometry, retained_flex_child_scroll_geometry};
use super::{Constants, resolve_auto_optional, resolve_auto_or_zero, resolve_length_or_zero};
use crate::error::{SizingAlgorithm, layout_child_geometry_error, sizing_resolution_error};
use crate::geometry::PhysicalAxis;
use crate::inline_wrapping::{EdgeEffect, WrappingMode};
use crate::layout_math::{MaxBeforeMinScalarClampExt, OptionalSizeExt};
use crate::output::MeasuredInsetBasisOf;
use crate::sizing::resolve::{
    EdgesResultExt, ResolvedFlexBasis, SizeResultExt, resolve_child_preferred_size,
    resolve_flex_basis, resolve_maximum_optional, resolve_minimum_optional, select_relative_insets,
};
use crate::{
    AlignItems, BoxSizing, Compute, ComputeInputOf, ComputeOutputOf, ComputedOverflow,
    ContainingLayoutContext, Edges, FlexItemCollapse, LayoutResultOf, LayoutScalar, LengthAutoOf,
    NodeOutputOf, ParentFormattingContext, Point, Position, RequestedAxis, RunMode, Size,
    SizingMode, Traverse,
};

/// Intrinsic basis provenance; final normal allocation is a separate phase.
#[derive(Clone, Copy, Debug)]
pub(super) enum IntrinsicFlexBasis {
    MinContent,
    MaxContent,
}

impl IntrinsicFlexBasis {
    fn operation(self) -> crate::SizingOperation {
        match self {
            Self::MinContent => crate::SizingOperation::MinContent,
            Self::MaxContent => crate::SizingOperation::MaxContent,
        }
    }
}

/// The raw border-box and content-box flex base sizes, excluding margins.
#[derive(Clone, Copy, Debug)]
pub(super) struct FlexBaseSize<S: LayoutScalar> {
    inner: S,
    outer: S,
}

impl<S: LayoutScalar> FlexBaseSize<S> {
    fn from_outer(outer: S, padding_border: S) -> Self {
        Self {
            inner: outer - padding_border,
            outer,
        }
    }

    pub(super) fn inner(self) -> S {
        self.inner
    }

    pub(super) fn outer(self) -> S {
        self.outer
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CollectedFlexItem<Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) source_index: usize,
    pub(super) collapse: FlexItemCollapse,
    pub(super) wrap_before: EdgeEffect,
    pub(super) wrap_after: EdgeEffect,
    pub(super) size: Size<Option<S>>,
    pub(super) authored_cross_size: Option<S>,
    pub(super) indefinite_percentage_axis: Option<PhysicalAxis>,
    pub(super) ratio_cross_has_percentage_basis: bool,
    pub(super) initial_output: ComputeOutputOf<S>,
    pub(super) initial_output_is_preferred_basis: bool,
    pub(super) flex_base: FlexBaseSize<S>,
    pub(super) hypothetical_main_size: S,
    pub(super) max_content_main_size: S,
    pub(super) hypothetical_size: Size<S>,
    pub(super) cross_size_is_auto: bool,
    pub(super) automatic_min_main_size: Option<S>,
    pub(super) min_size: Size<Option<S>>,
    pub(super) max_size: Size<Option<S>>,
    pub(super) min_cross_size: Option<S>,
    pub(super) max_cross_size: Option<S>,
    pub(super) margin: Edges<S>,
    pub(super) margin_is_auto: Edges<bool>,
    pub(super) padding: Edges<S>,
    pub(super) border: Edges<S>,
    pub(super) align_self: AlignItems,
    pub(super) initial_baseline: FlexItemBaseline<S>,
    pub(super) flex_grow_factor: S,
    pub(super) flex_shrink_factor: S,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ResolvedFlexItem<Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) source_index: usize,
    pub(super) authored_cross_size: Option<S>,
    pub(super) indefinite_percentage_axis: Option<PhysicalAxis>,
    pub(super) ratio_cross_has_percentage_basis: bool,
    pub(super) initial_output: ComputeOutputOf<S>,
    pub(super) initial_output_is_preferred_basis: bool,
    pub(super) flex_base: FlexBaseSize<S>,
    pub(super) hypothetical_main_size: S,
    pub(super) max_content_main_size: S,
    pub(super) target_size: Size<S>,
    pub(super) cross_size_is_auto: bool,
    pub(super) automatic_min_main_size: Option<S>,
    pub(super) min_size: Size<Option<S>>,
    pub(super) max_size: Size<Option<S>>,
    pub(super) min_cross_size: Option<S>,
    pub(super) max_cross_size: Option<S>,
    pub(super) margin: Edges<S>,
    pub(super) margin_is_auto: Edges<bool>,
    pub(super) padding: Edges<S>,
    pub(super) border: Edges<S>,
    pub(super) align_self: AlignItems,
    pub(super) baseline: FlexItemBaseline<S>,
    pub(super) flex_grow_factor: S,
    pub(super) flex_shrink_factor: S,
    pub(super) offset_main: DistributionNumber<S>,
    pub(super) offset_cross: S,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FinalFlexItem<Node, S: LayoutScalar> {
    pub(super) _node: core::marker::PhantomData<Node>,
    pub(super) source_index: crate::SourceIndex,
    pub(super) output: ComputeOutputOf<S>,
    pub(super) margin: Edges<S>,
    pub(super) baseline_preference: Option<BaselinePort>,
    pub(super) baseline: FlexItemBaseline<S>,
    pub(super) location: Point<S>,
    pub(super) normal_location: Point<S>,
}

impl<Node, S: LayoutScalar> CollectedFlexItem<Node, S> {
    pub(super) fn is_collapsed(&self) -> bool {
        self.collapse == FlexItemCollapse::Collapsed
    }
}

impl<Node, S: LayoutScalar> From<CollectedFlexItem<Node, S>> for ResolvedFlexItem<Node, S> {
    fn from(item: CollectedFlexItem<Node, S>) -> Self {
        Self {
            node: item.node,
            source_index: item.source_index,
            authored_cross_size: item.authored_cross_size,
            indefinite_percentage_axis: item.indefinite_percentage_axis,
            ratio_cross_has_percentage_basis: item.ratio_cross_has_percentage_basis,
            initial_output: item.initial_output,
            initial_output_is_preferred_basis: item.initial_output_is_preferred_basis,
            flex_base: item.flex_base,
            hypothetical_main_size: item.hypothetical_main_size,
            max_content_main_size: item.max_content_main_size,
            target_size: item.hypothetical_size,
            cross_size_is_auto: item.cross_size_is_auto,
            automatic_min_main_size: item.automatic_min_main_size,
            min_size: item.min_size,
            max_size: item.max_size,
            min_cross_size: item.min_cross_size,
            max_cross_size: item.max_cross_size,
            margin: item.margin,
            margin_is_auto: item.margin_is_auto,
            padding: item.padding,
            border: item.border,
            align_self: item.align_self,
            baseline: item.initial_baseline,
            flex_grow_factor: item.flex_grow_factor,
            flex_shrink_factor: item.flex_shrink_factor,
            offset_main: DistributionNumber::ZERO,
            offset_cross: S::ZERO,
        }
    }
}

fn is_eligible_item<S: LayoutScalar>(style: &FlexItemProjection<'_, S>) -> bool {
    style.common.position != Position::Absolute && style.display != crate::Display::None
}

pub(super) fn has_eligible_items<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
) -> bool
where
    Tree: Compute<'input, M>,
{
    // Index the actual child list without retaining an iterator borrow across
    // the projection callback or allocating a second item collection.
    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        if with_flex_item_projection::<Tree, M, _>(tree, child, |_, style| is_eligible_item(&style))
        {
            return true;
        }
    }
    false
}

pub(super) struct ItemCollection<'input, Node, S: LayoutScalar> {
    source: CollectionSource<Node>,
    items: Vec<CollectedFlexItem<Node, S>>,
    order_entries: Vec<(crate::ItemOrder, crate::SourceIndex)>,
    pending: Option<ItemBuilder<'input, Node, S>>,
    input: Option<ComputeInputOf<S>>,
    run_mode: RunMode,
}

enum CollectionSource<Node> {
    Children(std::iter::Enumerate<std::vec::IntoIter<Node>>),
    Recollect {
        next_index: usize,
        round: FlexLineCollectionRound,
    },
}

impl<'input, Node: Copy, S: LayoutScalar> ItemCollection<'input, Node, S> {
    pub(super) fn new<Tree, M>(tree: &Tree, node: Node, run_mode: RunMode) -> Self
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let children = tree.children(node).collect::<Vec<_>>();
        Self {
            items: Vec::with_capacity(children.len()),
            order_entries: Vec::with_capacity(children.len()),
            source: CollectionSource::Children(children.into_iter().enumerate()),
            pending: None,
            input: None,
            run_mode,
        }
    }

    pub(super) fn recollect(
        count: usize,
        run_mode: RunMode,
        round: FlexLineCollectionRound,
    ) -> Self {
        Self {
            source: CollectionSource::Recollect {
                next_index: 0,
                round,
            },
            items: Vec::with_capacity(count),
            order_entries: Vec::new(),
            pending: None,
            input: None,
            run_mode,
        }
    }

    pub(super) fn next<Tree, M>(
        &mut self,
        tree: &mut Tree,
        constants: &Constants<S>,
    ) -> LayoutResultOf<Node, Option<FlexItemRequest<Node, S>>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        self.next_item(tree, constants, &[])
    }

    pub(super) fn next_recollected<Tree, M>(
        &mut self,
        tree: &mut Tree,
        constants: &Constants<S>,
        original: &[CollectedFlexItem<Node, S>],
    ) -> LayoutResultOf<Node, Option<FlexItemRequest<Node, S>>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        self.next_item(tree, constants, original)
    }

    fn next_item<Tree, M>(
        &mut self,
        tree: &mut Tree,
        constants: &Constants<S>,
        original: &[CollectedFlexItem<Node, S>],
    ) -> LayoutResultOf<Node, Option<FlexItemRequest<Node, S>>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        if let Some(input) = self.input.take() {
            let node = self
                .pending
                .as_ref()
                .expect("active Flex item")
                .prepared
                .node;
            return Ok(Some(FlexItemRequest { node, input }));
        }
        debug_assert!(self.pending.is_none());
        loop {
            let (node, source_index, check_eligibility) = match &mut self.source {
                CollectionSource::Children(children) => {
                    let Some((source_index, node)) = children.next() else {
                        return Ok(None);
                    };
                    (node, source_index, true)
                }
                CollectionSource::Recollect { next_index, round } => {
                    let Some(item) = original.get(*next_index) else {
                        return Ok(None);
                    };
                    *next_index += 1;
                    if *round == FlexLineCollectionRound::Collapsed && item.is_collapsed() {
                        self.items.push(*item);
                        continue;
                    }
                    (item.node, item.source_index, false)
                }
            };
            let item = with_flex_item_projection::<Tree, M, _>(tree, node, |tree, style| {
                if check_eligibility && !is_eligible_item(&style) {
                    return Ok(None);
                }
                ItemBuilder::begin(tree, node, source_index, style, constants, self.run_mode)
                    .map(Some)
            })?;
            if let Some((builder, input)) = item {
                self.pending = Some(builder);
                return Ok(Some(FlexItemRequest { node, input }));
            }
        }
    }

    pub(super) fn accept<Tree, M>(
        &mut self,
        tree: &Tree,
        constants: &Constants<S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let builder = self.pending.take().expect("active Flex item");
        let item_order = builder.prepared.style.item_order;
        match builder.accept(tree, constants, self.run_mode, output)? {
            ControlFlow::Break(item) => {
                if matches!(self.source, CollectionSource::Children(_)) {
                    self.order_entries
                        .push((item_order, crate::SourceIndex::new(item.source_index)));
                }
                self.items.push(item);
            }
            ControlFlow::Continue((builder, input)) => {
                self.pending = Some(builder);
                self.input = Some(input);
            }
        }
        Ok(())
    }

    pub(super) fn finish(self) -> Vec<CollectedFlexItem<Node, S>> {
        if matches!(self.source, CollectionSource::Recollect { .. }) {
            return self.items;
        }
        let permutation = flex_item_order_permutation(&self.order_entries);
        let mut items_by_source = self
            .items
            .into_iter()
            .map(|item| (crate::SourceIndex::new(item.source_index), item))
            .collect::<std::collections::BTreeMap<_, _>>();
        permutation
            .into_iter()
            .map(|source_index| {
                items_by_source
                    .remove(&source_index)
                    .expect("the flex order permutation contains every collected source index")
            })
            .collect()
    }
}

pub(super) struct FlexItemRequest<Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) input: ComputeInputOf<S>,
}

#[derive(Clone, Copy)]
struct ItemPreparation<'input, Node, S: LayoutScalar> {
    node: Node,
    source_index: usize,
    style: FlexItemProjection<'input, S>,
    padding: Edges<S>,
    border: Edges<S>,
    margin: Edges<S>,
    margin_is_auto: Edges<bool>,
    box_sizing_adjustment: Size<S>,
    child_owned_axes: Size<bool>,
    authored_cross_size: Option<S>,
    authored_size: Size<Option<S>>,
    intrinsic_flex_basis: Option<IntrinsicFlexBasis>,
    padding_border_main: S,
    resolved_flex_basis: Option<S>,
    measured_basis: Option<S>,
    raw_min_size: Size<Option<S>>,
    raw_max_size: Size<Option<S>>,
    min_size: Size<Option<S>>,
    max_size: Size<Option<S>>,
    align_self: AlignItems,
    cross_size_is_auto: bool,
    border_box_offer_axis: Option<PhysicalAxis>,
    use_content_sizing_for_base: bool,
    preferred_basis_probe: bool,
    base_cross: Option<S>,
    child_known: Size<Option<S>>,
    child_known_for_base: Size<Option<S>>,
    content_basis_operation: crate::SizingOperation,
    child_available: Size<crate::AxisSizingRequestOf<S>>,
    flex_basis_uses_content: bool,
}

enum ItemRequest<S: LayoutScalar> {
    Cross,
    Initial,
    Minimum {
        initial: ComputeOutputOf<S>,
        minimum: AutomaticMinimum<S>,
    },
    Basis {
        initial: ComputeOutputOf<S>,
        automatic_minimum: Option<S>,
    },
}

struct ItemBuilder<'input, Node, S: LayoutScalar> {
    prepared: ItemPreparation<'input, Node, S>,
    pending: ItemRequest<S>,
}

impl<'input, Node: Copy, S: LayoutScalar> ItemBuilder<'input, Node, S> {
    fn begin<Tree, M>(
        tree: &Tree,
        node: Node,
        source_index: usize,
        style: FlexItemProjection<'input, S>,
        constants: &Constants<S>,
        run_mode: RunMode,
    ) -> LayoutResultOf<Node, (Self, ComputeInputOf<S>), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let padding = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                *style.common.padding,
                constants.percentage_parent_size,
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, node)?;
        let border = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                *style.common.border,
                constants.percentage_parent_size,
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, node)?;
        let margin = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                *style.common.margin,
                constants.percentage_parent_size,
                resolve_auto_or_zero,
            )
            .transpose_with_node(tree, node)?;
        let margin_is_auto = style.common.margin.map(LengthAutoOf::is_auto);
        let padding_border = padding + border;
        let box_sizing_adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
            padding_border.sum_axes()
        } else {
            Size::ZERO
        };
        let child_owned_axes = super::child_preferred_axes::<Tree, M>(tree, node);
        let resolved_preferred_size = resolve_child_preferred_size(
            style.common.size,
            constants.percentage_parent_size,
            SizingAlgorithm::Flex,
            child_owned_axes,
        )
        .transpose_with_node(tree, node)?;
        let authored_cross_size = constants
            .axes
            .cross_size(resolved_preferred_size.add_optional(box_sizing_adjustment));
        let authored_size = resolved_preferred_size
            .apply_aspect_ratio(
                style
                    .common
                    .aspect_ratio
                    .filter(|_| !child_owned_axes.width && !child_owned_axes.height),
            )
            .add_optional(box_sizing_adjustment);
        let flex_basis_resolution = resolve_flex_basis(
            style.flex_basis,
            constants.axes.main_physical_axis(),
            constants.axes.main_size(constants.percentage_parent_size),
        )
        .map_err(|error| sizing_resolution_error(node, error))?;
        let flex_basis_uses_content = match flex_basis_resolution {
            ResolvedFlexBasis::Auto => constants.axes.main_size(authored_size).is_none(),
            ResolvedFlexBasis::Content => true,
            ResolvedFlexBasis::MinContent
            | ResolvedFlexBasis::MaxContent
            | ResolvedFlexBasis::Definite(_) => false,
        };
        let basis_is_preferred_max_content =
            matches!(flex_basis_resolution, ResolvedFlexBasis::Auto)
                && constants.axes.main_size(child_owned_axes);
        let intrinsic_flex_basis = match flex_basis_resolution {
            ResolvedFlexBasis::MinContent => Some(IntrinsicFlexBasis::MinContent),
            ResolvedFlexBasis::MaxContent => Some(IntrinsicFlexBasis::MaxContent),
            ResolvedFlexBasis::Auto if basis_is_preferred_max_content => {
                Some(IntrinsicFlexBasis::MaxContent)
            }
            ResolvedFlexBasis::Auto
            | ResolvedFlexBasis::Content
            | ResolvedFlexBasis::Definite(_) => None,
        };
        let padding_border_main = constants.axes.main_size(padding_border.sum_axes());
        let resolved_flex_basis = match flex_basis_resolution {
            ResolvedFlexBasis::Auto => constants.axes.main_size(authored_size),
            ResolvedFlexBasis::Content
            | ResolvedFlexBasis::MinContent
            | ResolvedFlexBasis::MaxContent => None,
            ResolvedFlexBasis::Definite(flex_basis) => {
                Some(flex_basis + constants.axes.main_size(box_sizing_adjustment))
            }
        };
        // The child's border box respects its insets even if its raw basis does not.
        let measured_basis = resolved_flex_basis.map(|basis| match flex_basis_resolution {
            ResolvedFlexBasis::Definite(_) => basis.max(padding_border_main),
            _ => basis,
        });
        let size = constants.axes.with_main_size(authored_size, measured_basis);
        let raw_min_size = Size::new(
            resolve_minimum_optional(
                &style.common.min_size.width,
                SizingAlgorithm::Flex,
                PhysicalAxis::Horizontal,
                constants.percentage_parent_size.width,
                true,
            ),
            resolve_minimum_optional(
                &style.common.min_size.height,
                SizingAlgorithm::Flex,
                PhysicalAxis::Vertical,
                constants.percentage_parent_size.height,
                true,
            ),
        )
        .transpose_with_node(tree, node)?;
        let raw_max_size = Size::new(
            resolve_maximum_optional(
                &style.common.max_size.width,
                SizingAlgorithm::Flex,
                PhysicalAxis::Horizontal,
                constants.percentage_parent_size.width,
                true,
            ),
            resolve_maximum_optional(
                &style.common.max_size.height,
                SizingAlgorithm::Flex,
                PhysicalAxis::Vertical,
                constants.percentage_parent_size.height,
                true,
            ),
        )
        .transpose_with_node(tree, node)?;
        let (resolved_min_size, resolved_max_size) = match *style.common.aspect_ratio {
            Some(ratio) if child_owned_axes.width || child_owned_axes.height => {
                crate::layout_math::transfer_aspect_ratio_limits(
                    resolved_preferred_size,
                    raw_min_size,
                    raw_max_size,
                    ratio.get(),
                )
            }
            ratio => (
                raw_min_size.apply_aspect_ratio(ratio),
                raw_max_size.apply_aspect_ratio(ratio),
            ),
        };
        let min_size = resolved_min_size.add_optional(box_sizing_adjustment);
        let max_size = resolved_max_size.add_optional(box_sizing_adjustment);
        let align_self = style.align_self.unwrap_or(constants.align_items);
        let cross_size_is_auto = constants
            .axes
            .cross_size(Size::new(
                &style.common.size.width,
                &style.common.size.height,
            ))
            .is_auto();
        let percentage_parent_size = constants.percentage_parent_size;
        let inherited_requests = constants.sizing_request.axes();
        let available_inner_size = constants
            .axes
            .with_cross_size(constants.node_inner_size, constants.cross_content_extent());
        let extents = available_inner_size
            .zip_map(constants.max_inner_size, |definite, maximum| {
                (definite, maximum)
            });
        let available = crate::layout_math::transpose_axis_requests_at_node(
            node,
            inherited_requests.zip_map(extents, |request, (definite, maximum)| {
                if let (None, Some(maximum)) = (definite, maximum) {
                    crate::layout_math::clamp_axis_request_space(request, None, Some(maximum))
                } else {
                    crate::layout_math::with_available_extent(request, definite)
                }
            }),
        )?;
        let border_box_offer_axis = (cross_size_is_auto && tree.has_leaf_measurement(node))
            .then_some(constants.axes.cross_physical_axis());
        let mut cross_request = constants.axes.cross_size(available);
        if cross_size_is_auto {
            cross_request = crate::layout_math::inset_axis_request(
                cross_request,
                constants.axes.cross_edge_sum(margin),
            )
            .map_err(|error| sizing_resolution_error(node, error.into()))?;
        }
        let cross_available = crate::layout_math::clamp_axis_request_space(
            cross_request,
            constants.axes.cross_size(min_size),
            constants.axes.cross_size(max_size),
        )
        .map_err(|error| sizing_resolution_error(node, error.into()))?;
        let available = constants.axes.with_cross_size(available, cross_available);
        let use_content_sizing_for_base = intrinsic_flex_basis.is_some()
            || flex_basis_uses_content && style.display == crate::Display::Block;
        let clamp_cross = |cross: S| {
            cross
                .clamp_max_before_min_optional(
                    constants.axes.cross_size(min_size),
                    constants.axes.cross_size(max_size),
                )
                .max(constants.axes.cross_size(padding_border.sum_axes()))
        };
        let stretched_cross = if !constants.wraps
            && align_self == AlignItems::Stretch
            && cross_size_is_auto
            && !constants.axes.cross_start_edge(margin_is_auto)
            && !constants.axes.cross_end_edge(margin_is_auto)
        {
            constants
                .axes
                .cross_size(percentage_parent_size)
                .map(|cross| cross - constants.axes.cross_edge_sum(margin))
        } else {
            None
        };
        let child_owns_preference = child_owned_axes.width || child_owned_axes.height;
        let preferred_basis_probe = use_content_sizing_for_base && child_owns_preference;
        let base_cross = constants
            .axes
            .cross_size(size)
            .or(stretched_cross)
            .map(clamp_cross);
        let child_known = if child_owns_preference {
            constants.axes.with_cross_size(size, base_cross)
        } else if use_content_sizing_for_base {
            constants.axes.with_cross_size(
                size,
                stretched_cross
                    .map(clamp_cross)
                    .or(constants.axes.cross_size(size)),
            )
        } else {
            size
        };
        let child_known_for_base = constants.axes.size_from_main_cross(None, base_cross);
        let main_request = constants.axes.main_size(available);
        let content_basis_operation = if constants
            .axes
            .main_size(constants.node_inner_size)
            .is_none()
            && constants.available_main.operation() == crate::SizingOperation::MinContent
        {
            crate::SizingOperation::MinContent
        } else {
            crate::SizingOperation::MaxContent
        };
        let operation = intrinsic_flex_basis
            .map(IntrinsicFlexBasis::operation)
            .or(use_content_sizing_for_base.then_some(content_basis_operation));
        let child_available = constants.axes.with_main_size(
            available,
            operation.map_or(main_request, |operation| {
                main_request.with_operation(operation)
            }),
        );
        let child_context =
            ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::Flex);
        let child_sizing = constants.sizing_request.with_axes(child_available);
        let prepared = ItemPreparation {
            node,
            source_index,
            style,
            padding,
            border,
            margin,
            margin_is_auto,
            box_sizing_adjustment,
            child_owned_axes,
            authored_cross_size,
            authored_size,
            intrinsic_flex_basis,
            padding_border_main,
            resolved_flex_basis,
            measured_basis,
            raw_min_size,
            raw_max_size,
            min_size,
            max_size,
            align_self,
            cross_size_is_auto,
            border_box_offer_axis,
            use_content_sizing_for_base,
            preferred_basis_probe,
            base_cross,
            child_known,
            child_known_for_base,
            content_basis_operation,
            child_available,
            flex_basis_uses_content,
        };
        let (pending, input) = if use_content_sizing_for_base
            && constants.axes.main_size(child_owned_axes)
            && constants.axes.cross_size(child_owned_axes)
            && style.common.aspect_ratio.is_some()
            && constants.axes.cross_size(child_known).is_none()
        {
            (
                ItemRequest::Cross,
                ComputeInputOf::for_child(
                    RunMode::ComputeSize,
                    SizingMode::InherentSize,
                    constants.axes.cross_requested_axis(),
                    child_known,
                    percentage_parent_size,
                    child_context,
                    child_sizing,
                )
                .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
            )
        } else {
            (
                ItemRequest::Initial,
                prepared.initial_request(constants, run_mode),
            )
        };
        Ok((Self { prepared, pending }, input))
    }

    fn accept<Tree, M>(
        mut self,
        tree: &Tree,
        constants: &Constants<S>,
        run_mode: RunMode,
        output: ComputeOutputOf<S>,
    ) -> ItemStep<'input, Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let node = self.prepared.node;
        let (initial, automatic_minimum) = match self.pending {
            ItemRequest::Cross => {
                self.prepared.child_known = constants.axes.with_cross_size(
                    self.prepared.child_known,
                    Some(constants.axes.cross_size(output.size)),
                );
                self.pending = ItemRequest::Initial;
                let input = self.prepared.initial_request(constants, run_mode);
                return Ok(ControlFlow::Continue((self, input)));
            }
            ItemRequest::Initial => {
                if let Some((minimum, input)) = AutomaticMinimum::prepare(
                    tree,
                    node,
                    &self.prepared.style,
                    constants,
                    FlexItemSizes {
                        preferred: self.prepared.authored_size,
                        min: self.prepared.min_size,
                        max: self.prepared.max_size,
                    },
                    self.prepared.child_known_for_base,
                )? {
                    self.pending = ItemRequest::Minimum {
                        initial: output,
                        minimum,
                    };
                    return Ok(ControlFlow::Continue((self, input)));
                }
                (output, None)
            }
            ItemRequest::Minimum { initial, minimum } => {
                (initial, Some(minimum.resolve(output, constants)))
            }
            ItemRequest::Basis {
                initial,
                automatic_minimum,
            } => {
                return self
                    .prepared
                    .finish(
                        tree,
                        constants,
                        initial,
                        automatic_minimum,
                        constants.axes.main_size(output.size),
                    )
                    .map(ControlFlow::Break);
            }
        };
        match self.prepared.basis_request(constants, initial)? {
            ControlFlow::Break(basis) => self
                .prepared
                .finish(tree, constants, initial, automatic_minimum, basis)
                .map(ControlFlow::Break),
            ControlFlow::Continue(input) => {
                self.pending = ItemRequest::Basis {
                    initial,
                    automatic_minimum,
                };
                Ok(ControlFlow::Continue((self, input)))
            }
        }
    }
}

type ItemStep<'input, Node, S, M> = LayoutResultOf<
    Node,
    ControlFlow<CollectedFlexItem<Node, S>, (ItemBuilder<'input, Node, S>, ComputeInputOf<S>)>,
    S,
    M,
>;

impl<Node: Copy, S: LayoutScalar> ItemPreparation<'_, Node, S> {
    fn initial_request(&self, constants: &Constants<S>, run_mode: RunMode) -> ComputeInputOf<S> {
        let Self {
            style,
            child_available,
            child_known,
            child_owned_axes,
            preferred_basis_probe,
            use_content_sizing_for_base,
            align_self,
            margin_is_auto,
            border_box_offer_axis,
            ..
        } = *self;
        let percentage_parent_size = constants.percentage_parent_size;
        let child_context =
            ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::Flex);
        let child_sizing = constants.sizing_request.with_axes(child_available);
        let child_sizing = if preferred_basis_probe {
            child_sizing.with_axes(super::child_preferred_probe_axes(
                constants.axes,
                child_available,
                child_known,
                child_owned_axes,
            ))
        } else {
            child_sizing
        };
        ComputeInputOf::for_child(
            if preferred_basis_probe {
                RunMode::ComputeSize
            } else {
                run_mode
            },
            if use_content_sizing_for_base {
                SizingMode::ContentSize
            } else {
                SizingMode::InherentSize
            },
            RequestedAxis::Both,
            child_known,
            percentage_parent_size,
            child_context,
            child_sizing,
        )
        .with_physical_baseline_demand(
            constants.physical_baseline_demand
                || BaselinePort::for_alignment(align_self).is_some()
                    && style.common.flow_axes.block_axis() == constants.axes.cross_physical_axis()
                    && !constants.axes.cross_start_edge(margin_is_auto)
                    && !constants.axes.cross_end_edge(margin_is_auto),
        )
        .with_border_box_offer_axis(border_box_offer_axis)
        .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars)
    }

    fn basis_request<M>(
        &self,
        constants: &Constants<S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, ControlFlow<S, ComputeInputOf<S>>, S, M> {
        let Self {
            node,
            style,
            resolved_flex_basis,
            intrinsic_flex_basis,
            base_cross,
            box_sizing_adjustment,
            child_available,
            content_basis_operation,
            child_known_for_base,
            child_owned_axes,
            border_box_offer_axis,
            ..
        } = *self;
        if let Some(flex_basis) = resolved_flex_basis {
            Ok(ControlFlow::Break(flex_basis))
        } else if intrinsic_flex_basis.is_some() {
            Ok(ControlFlow::Break(constants.axes.main_size(output.size)))
        } else if let Some((ratio, cross)) = style.common.aspect_ratio.zip(base_cross) {
            constants
                .axes
                .main_size_from_cross_in_sizing_box(cross, ratio, box_sizing_adjustment)
                .map(ControlFlow::Break)
                .map_err(|error| sizing_resolution_error(node, error.into()))
        } else {
            let percentage_parent_size = constants.percentage_parent_size;
            let basis_available = constants.axes.with_main_size(
                child_available,
                constants
                    .axes
                    .main_size(child_available)
                    .with_operation(content_basis_operation),
            );
            let basis_available = super::child_preferred_probe_axes(
                constants.axes,
                basis_available,
                child_known_for_base,
                child_owned_axes,
            );
            Ok(ControlFlow::Continue(
                ComputeInputOf::for_child(
                    RunMode::ComputeSize,
                    SizingMode::ContentSize,
                    constants.axes.main_requested_axis(),
                    child_known_for_base,
                    constants.axes.with_main_size(percentage_parent_size, None),
                    ContainingLayoutContext::new(
                        constants.flow_axes,
                        ParentFormattingContext::Flex,
                    ),
                    constants.sizing_request.with_axes(basis_available),
                )
                .with_border_box_offer_axis(border_box_offer_axis)
                .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
            ))
        }
    }

    fn finish<'tree, Tree, M>(
        &self,
        tree: &Tree,
        constants: &Constants<S>,
        output: ComputeOutputOf<S>,
        automatic_min_main_size: Option<S>,
        flex_basis: S,
    ) -> LayoutResultOf<Node, CollectedFlexItem<Node, S>, S, M>
    where
        Tree: Compute<'tree, M, Node = Node, Scalar = S>,
    {
        let Self {
            node,
            source_index,
            style,
            padding,
            border,
            margin,
            margin_is_auto,
            authored_cross_size,
            authored_size,
            padding_border_main,
            resolved_flex_basis,
            measured_basis,
            raw_min_size,
            raw_max_size,
            min_size,
            max_size,
            align_self,
            cross_size_is_auto,
            preferred_basis_probe,
            base_cross,
            child_owned_axes,
            flex_basis_uses_content,
            ..
        } = *self;
        let basis_is_preferred_max_content =
            style.flex_basis.is_auto() && constants.axes.main_size(child_owned_axes);
        let padding_border = padding + border;
        let percentage_parent_size = constants.percentage_parent_size;
        let hypothetical_main_size = clamp_main_size_axes(
            flex_basis,
            automatic_min_main_size,
            constants.axes.main_size(min_size),
            constants.axes.main_size(max_size),
            padding_border_main,
        );
        let authored_main_size = constants.axes.main_size(authored_size);
        let flex_basis_uses_padding_floor = measured_basis.is_some_and(|basis| {
            basis <= padding_border_main
                && style.flex_grow.get() == S::ZERO
                && (tree.child_count(node) > 0
                    || constants.axes.main_size(output.content_size) <= basis)
        });
        let intrinsic_main_size = if flex_basis_uses_padding_floor || basis_is_preferred_max_content
        {
            flex_basis
        } else if style.flex_basis.is_auto() && authored_main_size.is_some() {
            authored_main_size.unwrap_or(S::ZERO)
        } else if flex_basis_uses_content || self.intrinsic_flex_basis.is_some() {
            constants.axes.main_size(output.content_size)
        } else {
            constants
                .axes
                .main_size(output.content_size)
                .max(authored_main_size.unwrap_or(S::ZERO))
        };
        let max_content_main_size = intrinsic_main_size
            .clamp_max_before_min_optional(
                constants.axes.main_size(min_size),
                constants.axes.main_size(max_size),
            )
            .max(padding_border_main);
        let mut target_size = constants
            .axes
            .with_main_size(output.size, hypothetical_main_size);
        if align_self != AlignItems::Stretch
            && cross_size_is_auto
            && let Some(ratio) = *style.common.aspect_ratio
        {
            let transferred_cross = match constants.axes.main_physical_axis() {
                PhysicalAxis::Horizontal => hypothetical_main_size / ratio.get(),
                PhysicalAxis::Vertical => hypothetical_main_size * ratio.get(),
            };
            target_size = constants
                .axes
                .with_cross_size(target_size, transferred_cross);
        }
        target_size = constants.axes.with_cross_size(
            target_size,
            constants
                .axes
                .cross_size(target_size)
                .clamp_max_before_min_optional(
                    constants.axes.cross_size(raw_min_size),
                    constants.axes.cross_size(raw_max_size),
                )
                .max(constants.axes.cross_size(padding_border.sum_axes())),
        );
        let child_flow_axes = style.common.flow_axes;
        let main_axis = constants.axes.main_physical_axis();
        let ratio_supplies_basis = !style.common.item_is_replaced
            && style.common.aspect_ratio.is_some()
            && base_cross.is_some();
        let indefinite_percentage_axis = (constants.axes.main_logical_axis()
            == crate::geometry::LogicalAxis::Block
            && child_flow_axes.block_axis() == main_axis
            && constants.axes.main_size(percentage_parent_size).is_none()
            && resolved_flex_basis.is_none()
            && !ratio_supplies_basis)
            .then_some(main_axis);
        let ratio_cross_has_percentage_basis =
            if cross_size_is_auto && style.common.aspect_ratio.is_some() {
                // Flex definiteness comes from the container's independent main size
                // or a definite basis, not a percentage basis installed by replay.
                !constants.axes.main_size(child_owned_axes)
                    || constants
                        .axes
                        .main_size(constants.entry_percentage_basis)
                        .is_some()
                    || matches!(
                        resolve_flex_basis(style.flex_basis, main_axis, None)
                            .map_err(|error| sizing_resolution_error(node, error))?,
                        ResolvedFlexBasis::Definite(_)
                    )
            } else {
                false
            };
        let baseline = FlexItemBaseline::from_output(output, child_flow_axes);

        Ok(CollectedFlexItem {
            node,
            source_index,
            collapse: style.collapse,
            wrap_before: style.wrap_edges.before.effect(WrappingMode::Flex {
                multiline: constants.wraps,
            }),
            wrap_after: style.wrap_edges.after.effect(WrappingMode::Flex {
                multiline: constants.wraps,
            }),
            size: authored_size,
            authored_cross_size,
            indefinite_percentage_axis,
            ratio_cross_has_percentage_basis,
            initial_output: output,
            initial_output_is_preferred_basis: preferred_basis_probe,
            flex_base: FlexBaseSize::from_outer(flex_basis, padding_border_main),
            hypothetical_main_size,
            max_content_main_size,
            hypothetical_size: target_size,
            cross_size_is_auto,
            automatic_min_main_size,
            min_size,
            max_size,
            min_cross_size: constants.axes.cross_size(raw_min_size),
            max_cross_size: constants.axes.cross_size(raw_max_size),
            margin,
            margin_is_auto,
            padding,
            border,
            align_self,
            initial_baseline: baseline,
            flex_grow_factor: style.flex_grow.get(),
            flex_shrink_factor: style.flex_shrink.get(),
        })
    }
}

struct FlexItemSizes<S: LayoutScalar> {
    preferred: Size<Option<S>>,
    min: Size<Option<S>>,
    max: Size<Option<S>>,
}

struct AutomaticMinimum<S: LayoutScalar> {
    authored_size: Size<Option<S>>,
    resolved_max_size: Size<Option<S>>,
    transferred: Option<S>,
    replaced: bool,
    padding_border_main: S,
}

type AutomaticMinimumRequest<S> = Option<(AutomaticMinimum<S>, ComputeInputOf<S>)>;

impl<S: LayoutScalar> AutomaticMinimum<S> {
    fn prepare<'input, Tree, M>(
        tree: &Tree,
        node: Tree::Node,
        style: &FlexItemProjection<'_, S>,
        constants: &Constants<S>,
        sizing: FlexItemSizes<S>,
        child_known: Size<Option<S>>,
    ) -> LayoutResultOf<Tree::Node, AutomaticMinimumRequest<S>, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        if !constants
            .axes
            .main_size(Size::new(
                &style.common.min_size.width,
                &style.common.min_size.height,
            ))
            .is_auto()
            || flex_automatic_minimum_is_zero(style.common.overflow)
        {
            return Ok(None);
        }
        let FlexItemSizes {
            preferred: authored_size,
            min: min_size,
            max: resolved_max_size,
        } = sizing;
        let padding = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                *style.common.padding,
                constants.node_inner_size,
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, node)?;
        let border = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                *style.common.border,
                constants.node_inner_size,
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, node)?;
        let padding_border = padding + border;

        let cross = crate::layout_math::with_available_extent(
            crate::AxisSizingRequestOf::new(
                crate::AvailableSpaceOf::Indefinite,
                crate::SizingOperation::Normal,
            ),
            constants.cross_content_extent(),
        )
        .map_err(|error| sizing_resolution_error(node, error.into()))?;
        let cross = crate::layout_math::clamp_axis_request_space(
            cross,
            constants.axes.cross_size(min_size),
            constants.axes.cross_size(resolved_max_size),
        )
        .map_err(|error| sizing_resolution_error(node, error.into()))?;
        let available = constants.axes.size_from_main_cross(
            constants
                .axes
                .main_size(constants.sizing_request.axes())
                .with_operation(crate::SizingOperation::MinContent),
            cross,
        );
        let available = super::child_preferred_probe_axes(
            constants.axes,
            available,
            child_known,
            super::child_preferred_axes::<Tree, M>(tree, node),
        );
        let transferred = style
            .common
            .aspect_ratio
            .zip(constants.axes.cross_size(child_known))
            .map(|(ratio, cross)| constants.axes.main_size_from_cross_aspect(cross, ratio));
        Ok(Some((
            Self {
                authored_size,
                resolved_max_size,
                transferred,
                replaced: style.common.item_is_replaced,
                padding_border_main: constants.axes.main_size(padding_border.sum_axes()),
            },
            ComputeInputOf::for_child(
                RunMode::ComputeSize,
                SizingMode::ContentSize,
                constants.axes.main_requested_axis(),
                child_known,
                constants
                    .axes
                    .with_main_size(constants.percentage_parent_size, None),
                ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::Flex),
                constants.sizing_request.with_axes(available),
            )
            .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
        )))
    }

    fn resolve(self, output: ComputeOutputOf<S>, constants: &Constants<S>) -> S {
        let Self {
            authored_size,
            resolved_max_size,
            transferred,
            replaced,
            ..
        } = self;
        let mut min_content = constants
            .axes
            .main_size(output.size)
            .clamp_max_before_min_optional(None, constants.axes.main_size(authored_size))
            .clamp_max_before_min_optional(None, constants.axes.main_size(resolved_max_size));
        if let Some(transferred) = transferred {
            let transferred = transferred
                .clamp_max_before_min_optional(None, constants.axes.main_size(authored_size))
                .clamp_max_before_min_optional(None, constants.axes.main_size(resolved_max_size));
            min_content = if replaced {
                min_content.min(transferred)
            } else {
                min_content.max(transferred)
            };
        }
        min_content.max(self.padding_border_main)
    }
}

pub(super) fn flex_automatic_minimum_is_zero(overflow: ComputedOverflow) -> bool {
    overflow.x().is_scrollable() || overflow.y().is_scrollable()
}

pub(super) struct FinalItems<Node, S: LayoutScalar> {
    container_node: Node,
    resolved: std::vec::IntoIter<ResolvedFlexItem<Node, S>>,
    items: Vec<FinalFlexItem<Node, S>>,
    inset_basis: MeasuredInsetBasisOf<S>,
}

pub(super) struct PendingFinalItem<'input, Node, S: LayoutScalar> {
    pub(super) request: FlexItemRequest<Node, S>,
    item: ResolvedFlexItem<Node, S>,
    style: FlexItemProjection<'input, S>,
    inset: Edges<Option<S>>,
}

type FinalItemStep<'input, Node, S, M> =
    LayoutResultOf<Node, Option<PendingFinalItem<'input, Node, S>>, S, M>;

impl<Node: Copy, S: LayoutScalar> FinalItems<Node, S> {
    pub(super) fn begin<'input, Tree, M>(
        tree: &mut Tree,
        container_node: Node,
        collected_items: &[CollectedFlexItem<Node, S>],
        resolved: Vec<ResolvedFlexItem<Node, S>>,
        inset_basis: MeasuredInsetBasisOf<S>,
    ) -> Self
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        for item in collected_items.iter().filter(|item| item.is_collapsed()) {
            tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                item.node,
                crate::output::NormalGeometryOf::metadata(crate::Point::ZERO, crate::Size::ZERO),
                NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(item.source_index)),
            ));
        }

        Self {
            container_node,
            items: Vec::with_capacity(resolved.len()),
            resolved: resolved.into_iter(),
            inset_basis,
        }
    }

    pub(super) fn next<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
        constants: &Constants<S>,
    ) -> FinalItemStep<'input, Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Some(item) = self.resolved.next() else {
            return Ok(None);
        };
        with_flex_item_projection::<Tree, M, _>(tree, item.node, |tree, style| {
            let inset = select_relative_insets(*style.common.inset, constants.flow_axes)
                .zip_size(self.inset_basis.size().map(Some), resolve_auto_optional)
                .transpose_with_node(tree, item.node)?;
            let input = sized_item_request(tree, &item, &style, constants, RunMode::PerformLayout)?;
            Ok(Some(PendingFinalItem {
                request: FlexItemRequest {
                    node: item.node,
                    input,
                },
                item,
                style,
                inset,
            }))
        })
    }

    pub(super) fn accept<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
        constants: &Constants<S>,
        pending: PendingFinalItem<'input, Node, S>,
        mut output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let PendingFinalItem {
            item, style, inset, ..
        } = pending;
        let item = &item;
        let container_node = self.container_node;
        let resolved_flex_basis = match resolve_flex_basis(
            style.flex_basis,
            constants.axes.main_physical_axis(),
            constants.axes.main_size(constants.percentage_parent_size),
        )
        .map_err(|error| sizing_resolution_error(item.node, error))?
        {
            ResolvedFlexBasis::Definite(value) => Some(value),
            ResolvedFlexBasis::Auto
            | ResolvedFlexBasis::Content
            | ResolvedFlexBasis::MinContent
            | ResolvedFlexBasis::MaxContent => None,
        };
        suppress_padding_floor_flex_basis_content_overflow(
            tree,
            item,
            &mut output,
            resolved_flex_basis,
            constants,
        );
        let baseline = FlexItemBaseline::from_output(output, style.common.flow_axes);

        let invalid_position = |value| {
            sizing_resolution_error(
                item.node,
                crate::LengthResolutionStatus::InvalidNumeric { value }.into(),
            )
        };
        let normal_location = constants.axes.point_from_main_cross(
            item.normal_main_location(constants, output.size)
                .map_err(invalid_position)?,
            item.normal_cross_location(constants, output.size),
        );
        let location = constants.axes.point_from_main_cross(
            item.final_main_location(constants, output.size, inset)
                .map_err(invalid_position)?,
            item.final_cross_location(constants, output.size, inset)
                .map_err(invalid_position)?,
        );
        let scroll_geometry = super::input::with_flex_item_scroll_projections::<Tree, M, _>(
            tree,
            item.node,
            |scroll_box, scroll_target| {
                retained_flex_child_scroll_geometry(
                    scroll_box,
                    scroll_target,
                    output.size,
                    output.content_size,
                    item.padding,
                    item.border,
                    output.scroll_geometry,
                )
            },
        )
        .map_err(|error| layout_child_geometry_error(container_node, item.node, error))?;
        output = retain_flex_scroll_geometry(output, scroll_geometry);
        tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
            item.node,
            crate::output::NormalGeometryOf::box_geometry(
                normal_location,
                output.size,
                scroll_geometry.content_box(),
            ),
            NodeOutputOf::<S> {
                source_index: crate::SourceIndex::new(item.source_index),
                location,
                size: output.size,
                named_baselines: output.named_baselines,
                content_size: output.content_size,
                border: item.border,
                padding: item.padding,
                margin: item.margin,
                ..NodeOutputOf::new()
            }
            .with_scroll_geometry(Some(scroll_geometry)),
        ));
        self.items.push(FinalFlexItem {
            _node: core::marker::PhantomData,
            source_index: crate::SourceIndex::new(item.source_index),
            output,
            margin: item.margin,
            baseline_preference: item.baseline_preference(constants.axes),
            baseline,
            location,
            normal_location,
        });
        Ok(())
    }

    pub(super) fn finish(self) -> Vec<FinalFlexItem<Node, S>> {
        self.items
    }
}

fn suppress_padding_floor_flex_basis_content_overflow<Node, S: LayoutScalar>(
    tree: &impl Traverse<Node = Node>,
    item: &ResolvedFlexItem<Node, S>,
    output: &mut ComputeOutputOf<S>,
    resolved_flex_basis: Option<S>,
    constants: &Constants<S>,
) where
    Node: Copy,
{
    let Some(resolved_flex_basis) = resolved_flex_basis else {
        return;
    };
    let padding_border = constants
        .axes
        .main_size((item.padding + item.border).sum_axes());
    let measured_basis = item.flex_base.outer().max(padding_border);
    if item.flex_grow_factor == S::ZERO
        && resolved_flex_basis <= padding_border
        && tree.child_count(item.node) == 0
        && constants.axes.main_size(output.size) <= measured_basis
        && constants.axes.main_size(output.content_size) <= measured_basis
        && constants.axes.main_size(item.target_size) <= padding_border
    {
        output.content_size = constants.axes.with_main_size(
            output.content_size,
            constants.axes.main_size(item.target_size),
        );
    }
}

impl<Node, S: LayoutScalar> ResolvedFlexItem<Node, S> {
    pub(super) fn settled_indefinite_percentage_axis(
        &self,
        axes: super::FlexAxes,
        child_flow: crate::FlowAxes,
    ) -> Option<PhysicalAxis> {
        let cross_axis = axes.cross_physical_axis();
        let cross_is_fixed_by_bounds = axes
            .cross_size(self.min_size)
            .zip(axes.cross_size(self.max_size))
            .is_some_and(|(min, max)| max <= min);
        let indefinite_cross = (child_flow.block_axis() == cross_axis
            && self.authored_cross_size.is_none()
            && !cross_is_fixed_by_bounds
            && !self.ratio_cross_has_percentage_basis
            && !self.stretches_cross(axes))
        .then_some(cross_axis);
        self.indefinite_percentage_axis.or(indefinite_cross)
    }
}

pub(super) fn sized_item_request<'input, Tree, M>(
    tree: &Tree,
    item: &ResolvedFlexItem<Tree::Node, Tree::Scalar>,
    style: &FlexItemProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    mode: RunMode,
) -> LayoutResultOf<Tree::Node, ComputeInputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let known = final_item_size::<Tree, M>(tree, item, style, constants)?;

    let axes = constants.axes;
    let available = crate::layout_math::transpose_axis_requests_at_node(
        item.node,
        Size::new(
            crate::layout_math::with_available_extent(
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::Normal,
                ),
                constants.node_inner_size.width,
            ),
            crate::layout_math::with_available_extent(
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::Normal,
                ),
                constants.node_inner_size.height,
            ),
        ),
    )?;
    let available = crate::layout_math::transpose_axis_requests_at_node(
        item.node,
        Size::new(
            crate::layout_math::with_available_extent(available.width, known.width),
            crate::layout_math::with_available_extent(available.height, known.height),
        ),
    )?;
    Ok(ComputeInputOf::for_child(
        mode,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        known,
        constants.percentage_parent_size,
        ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::Flex),
        constants.sizing_request.with_axes(available),
    )
    .with_physical_baseline_demand(mode == RunMode::ComputeSize)
    .with_indefinite_percentage_axis(
        item.settled_indefinite_percentage_axis(axes, style.common.flow_axes),
    )
    .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars))
}

#[expect(
    clippy::type_complexity,
    reason = "the private flex size helper preserves the session's generic error envelope"
)]
fn final_item_size<'input, Tree, M>(
    tree: &Tree,
    item: &ResolvedFlexItem<<Tree as Traverse>::Node, Tree::Scalar>,
    style: &FlexItemProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Size<Option<Tree::Scalar>>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let child_owned_axes = super::child_preferred_axes::<Tree, M>(tree, item.node);
    let mut known = Size::new(Some(item.target_size.width), Some(item.target_size.height));
    if child_owned_axes.width || child_owned_axes.height {
        // The accepted target already accounts for the authored cross bounds.
        return Ok(known);
    }
    let padding = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            *style.common.padding,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, item.node)?;
    let border = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            *style.common.border,
            constants.node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, item.node)?;
    let box_sizing_adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
        (padding + border).sum_axes()
    } else {
        Size::<Tree::Scalar>::ZERO
    };
    let authored = resolve_child_preferred_size(
        style.common.size,
        constants.percentage_parent_size,
        SizingAlgorithm::Flex,
        Size::splat(false),
    )
    .transpose_with_node(tree, item.node)?
    .apply_aspect_ratio(*style.common.aspect_ratio)
    .add_optional(box_sizing_adjustment);

    if constants.axes.main_requested_axis() == RequestedAxis::Horizontal {
        if style.common.size.height.depends_on_basis() {
            known.height = authored.height.or(known.height);
        }
    } else if style.common.size.width.depends_on_basis() {
        known.width = authored.width.or(known.width);
    }
    Ok(known)
}
