use crate::error::{
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant, LayoutOperation,
    LayoutResultOf, SizingAlgorithm, sizing_resolution_error,
};
use crate::geometry::{FlowAxes, PhysicalAxis, PhysicalSide};
use crate::layout_math::{MaxBeforeMinScalarClampExt, OptionalSizeExt};
use crate::scroll::{
    CanonicalRetainedScrollSourceOf, CanonicalScrollRangeSeedPolicy,
    CanonicalScrollSourceBuilderOf, ScrollOriginAxes, ScrollOriginProgression,
    SettledAutoScrollbarState,
};
use crate::sizing::resolve::{
    EdgesResultExt, ResolvedPreferredSize, SizingResolutionError, resolve_auto_or_zero_fallible,
    resolve_length_or_zero_fallible, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_sizing,
};
use crate::{
    AvailableSpaceOf, AxisSizingRequestOf, BoxSizing, CacheAccess, Compute, ComputeInputOf,
    ComputeOutputOf, Edges, LayoutInputViewOf, LayoutScalar, NodeInputOf, NodeOutputOf, Point,
    Size, SizingOperation, SizingRequestOf, Traverse,
};

pub(crate) fn compute_hidden<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    source_index: crate::SourceIndex,
    containing_layout_context: crate::ContainingLayoutContext,
    containing_auto_scrollbar_pass: SettledAutoScrollbarState,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    ComputeOutputOf<<Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    M,
>
where
    Tree: Compute<'input, M>
        + CacheAccess<M, Node = <Tree as Traverse>::Node, Scalar = <Tree as Traverse>::Scalar>,
{
    tree.cache_clear(node)?;
    tree.set_unrounded_inline_fragment_state(node, None);
    tree.set_unrounded(crate::LayoutOutputEntryOf::hidden_unrounded(
        node,
        source_index,
    ));

    for index in 0..tree.child_count(node) {
        let child = tree.child(node, index);
        match tree.layout_input(child) {
            LayoutInputViewOf::Box(_) => {
                tree.set_unrounded(crate::LayoutOutputEntryOf::hidden_unrounded(
                    child,
                    crate::SourceIndex::new(index),
                ));
                let descendant_context = crate::ContainingLayoutContext::new(
                    containing_layout_context.flow_axes(),
                    crate::ParentFormattingContext::NoParent,
                );
                tree.compute_child(
                    child,
                    ComputeInputOf::hidden_in_containing_pass(
                        descendant_context,
                        containing_auto_scrollbar_pass,
                    ),
                )?;
            }
            LayoutInputViewOf::InlineText(_)
            | LayoutInputViewOf::LineBreak(_)
            | LayoutInputViewOf::InlineBoundary(_)
            | LayoutInputViewOf::TextControl(_) => {
                tree.cache_clear(child)?;
                tree.set_unrounded_inline_fragment_state(child, None);
                tree.set_unrounded(crate::LayoutOutputEntryOf::hidden_unrounded(
                    child,
                    crate::SourceIndex::new(index),
                ));
            }
        }
    }

    Ok(ComputeOutputOf::HIDDEN)
}

pub(crate) fn compute_root<'input, Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
    request: crate::LayoutRootRequestOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let style = tree.node_input(root);
    let RootComputeInput {
        input,
        align_to_parent,
    } = root_compute_input(tree, root, style, request)?;
    let output = tree.compute_child(root, input)?;
    finish_root(tree, root, style, input, output, align_to_parent)
}

pub(crate) fn compute_root_scoped<'input, Tree, M>(
    tree: &mut Tree,
    root: Tree::Node,
    request: crate::LayoutRootRequestOf<Tree::Scalar>,
) -> super::block_scope::ComputeResult<Tree::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let style = tree.node_input(root);
    let RootComputeInput {
        input,
        align_to_parent,
    } = root_compute_input(tree, root, style, request)?;
    let request = if tree.block_scope_child(root) {
        crate::block::scope::ChildRequest::Scope(crate::block::scope::root(tree, root, input)?)
    } else {
        crate::block::scope::ChildRequest::Ordinary(crate::block::scope::Ordinary {
            node: root,
            input,
            entry: None,
        })
    };
    let output = tree.compute_block_child(request)?;
    finish_root(tree, root, style, input, output, align_to_parent).map_err(Into::into)
}

pub(crate) fn finish_root<'input, Tree, M>(
    tree: &mut Tree,
    root: Tree::Node,
    style: &'input NodeInputOf<Tree::Scalar>,
    input: ComputeInputOf<Tree::Scalar>,
    output: ComputeOutputOf<Tree::Scalar>,
    align_to_parent: bool,
) -> LayoutResultOf<Tree::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let axes = input.containing_flow_axes();
    let root_edges = resolve_root_edges(tree, root, style, axes, input.parent())?;
    let scroll_geometry = root_scroll_geometry(root, style, output, &root_edges)?;
    let location = if align_to_parent {
        root_start_location(
            axes,
            output.size,
            input
                .sizing_request()
                .axes()
                .map(AxisSizingRequestOf::space),
        )
    } else {
        Point::ZERO
    };
    tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
        root,
        crate::output::NormalGeometryOf {
            location,
            size: output.size,
            content: if style.display == crate::Display::None {
                crate::output::NormalContentOf::Hidden
            } else {
                scroll_geometry.map_or(crate::output::NormalContentOf::Metadata, |geometry| {
                    crate::output::NormalContentOf::Box(geometry.content_box())
                })
            },
        },
        NodeOutputOf {
            source_index: crate::SourceIndex::ZERO,
            location,
            size: output.size,
            named_baselines: output.named_baselines,
            content_size: output.content_size,
            padding: root_edges.padding,
            border: root_edges.border,
            margin: root_edges.margin,
            ..NodeOutputOf::new()
        }
        .with_scroll_geometry(scroll_geometry),
    ));
    Ok(())
}

pub(super) fn position_fragment_root<'input, Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
    axes: FlowAxes,
    frame_size: Size<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let entry = tree.staged_unrounded(root)?;
    let mut output = entry.output();
    let location = axes.physical_point(
        crate::geometry::LogicalPointOf::new(Tree::Scalar::ZERO, Tree::Scalar::ZERO),
        axes.logical_size(output.size),
        frame_size,
    );
    let crate::output::GeometryStateOf::OrdinaryUnrounded {
        mut measured_normal,
    } = entry.geometry_state()
    else {
        return Err(LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(root),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::InvalidOutputGeometryPhase,
            ),
        ));
    };
    output.location = location;
    measured_normal.location = location;
    tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
        root,
        measured_normal,
        output,
    ));
    Ok(())
}

pub(super) struct RootComputeInput<S: LayoutScalar> {
    pub(super) input: ComputeInputOf<S>,
    align_to_parent: bool,
}

pub(super) fn root_compute_input<'input, Tree, M>(
    tree: &mut Tree,
    root: Tree::Node,
    style: &'input NodeInputOf<Tree::Scalar>,
    request: crate::LayoutRootRequestOf<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, RootComputeInput<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let sizing_request = request.sizing_request();
    let (axes, role, parent, mut known, fill_inline, align_to_parent) = match request.context() {
        crate::LayoutRootContextOf::Standalone {
            containing_flow_axes,
            percentage_parent,
        } => (
            containing_flow_axes,
            crate::ParentFormattingContext::NoParent,
            percentage_parent.map(|value| value.map(crate::NonNegativeFiniteOf::get)),
            Size::NONE,
            true,
            true,
        ),
        crate::LayoutRootContextOf::FlexItem(context) => (
            context.parent_flow_axes(),
            crate::ParentFormattingContext::Flex,
            context
                .percentage_parent()
                .map(|value| value.map(crate::NonNegativeFiniteOf::get)),
            context
                .allocated_size()
                .map(|value| value.map(crate::NonNegativeFiniteOf::get)),
            false,
            false,
        ),
        crate::LayoutRootContextOf::BlockFlow(context) => {
            admit_root_box(
                root,
                style,
                crate::Position::Relative,
                RootEdgeProfile::Zero,
            )?;
            (
                context.parent_flow_axes(),
                crate::ParentFormattingContext::BlockFlow,
                Size::new(Some(context.parent_inline_extent().get()), None),
                Size::NONE,
                true,
                true,
            )
        }
        crate::LayoutRootContextOf::PositionedAutoInsets(context) => {
            admit_positioned_root(root, style, None)?;
            (
                context.containing_flow_axes(),
                crate::ParentFormattingContext::BlockFlow,
                context.containing_size().map(|value| Some(value.get())),
                Size::NONE,
                false,
                true,
            )
        }
        crate::LayoutRootContextOf::RootLocalPositioned(context) => {
            admit_positioned_root(root, style, Some(context))?;
            (
                context.containing_flow_axes(),
                crate::ParentFormattingContext::BlockFlow,
                context.containing_size().map(|value| Some(value.get())),
                Size::NONE,
                false,
                false,
            )
        }
    };
    let containing = crate::ContainingLayoutContext::new(axes, role);
    if fill_inline {
        known = known.or(root_known_inline::<Tree, M>(
            tree,
            root,
            style,
            axes,
            sizing_request.axes(),
            parent,
        )?);
    }
    let inline_axis = FlowAxes::new(style.writing_mode, style.direction).inline_axis();
    if matches!(
        request.context(),
        crate::LayoutRootContextOf::PositionedAutoInsets(_)
            | crate::LayoutRootContextOf::RootLocalPositioned(_)
    ) && root_physical_axis_value(
        Size::new(&style.size.width, &style.size.height),
        inline_axis,
    )
    .is_auto()
    {
        known = root_known_on_axis(
            inline_axis,
            positioned_auto_inline_size(
                tree,
                root,
                style,
                containing,
                sizing_request,
                parent,
                inline_axis,
            )?,
        );
    }
    Ok(RootComputeInput {
        input: ComputeInputOf::root_layout(known, parent, containing, sizing_request),
        align_to_parent,
    })
}

fn root_context_error<Node, S: LayoutScalar, M>(
    node: Node,
    reason: crate::RootContextMismatch,
) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RootLayout,
        LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::RootContext(reason)),
    )
}

// Only the root-local profile can request padding/border admission, and only
// through actual definite preferred-inline resolution against its full CB.
enum RootEdgeProfile<S: LayoutScalar> {
    Zero,
    DefiniteInline {
        percentage_parent: crate::NonNegativeFiniteOf<S>,
    },
}

fn admit_root_box<Node, S: LayoutScalar, M>(
    node: Node,
    style: &NodeInputOf<S>,
    position: crate::Position,
    edges: RootEdgeProfile<S>,
) -> LayoutResultOf<Node, (), S, M>
where
    Node: Copy,
{
    let mismatch = if style.position != position {
        Some(crate::RootContextMismatch::Position)
    } else if style.display == crate::Display::None || style.display.is_inline_level() {
        Some(crate::RootContextMismatch::Display)
    } else if style.float != crate::Float::None {
        Some(crate::RootContextMismatch::Float)
    } else if style.clear != crate::Clear::None {
        Some(crate::RootContextMismatch::Clearance)
    } else if style.margin != Edges::all(crate::LengthAutoOf::ZERO) {
        Some(crate::RootContextMismatch::BoxEdges)
    } else {
        None
    };
    if let Some(reason) = mismatch {
        return Err(root_context_error(node, reason));
    }
    if style.padding != Edges::all(crate::LengthOf::ZERO)
        || style.border != Edges::all(crate::LengthOf::ZERO)
    {
        let allowed = match edges {
            RootEdgeProfile::Zero => false,
            RootEdgeProfile::DefiniteInline { percentage_parent } => matches!(
                resolve_preferred_sizing(
                    &style.size.width,
                    SizingAlgorithm::Block,
                    PhysicalAxis::Horizontal,
                    Some(percentage_parent.get()),
                    false,
                )
                .map_err(|error| sizing_resolution_error(node, error))?,
                ResolvedPreferredSize::Definite(_)
            ),
        };
        if !allowed {
            return Err(root_context_error(
                node,
                crate::RootContextMismatch::BoxEdges,
            ));
        }
    }
    if style.inset != Edges::all(crate::LengthAutoOf::AUTO) {
        return Err(root_context_error(node, crate::RootContextMismatch::Insets));
    }
    Ok(())
}

fn fixed_grid_axis<S: LayoutScalar>(components: &[crate::TrackComponentOf<S>]) -> bool {
    let mut has_track = false;
    let fixed = components.iter().all(|component| match component {
        crate::TrackComponentOf::Track(track) => {
            has_track = true;
            track
                .min
                .definite(None)
                .zip(track.max.definite(None))
                .is_some_and(|(minimum, maximum)| minimum == maximum && minimum.is_finite())
        }
        crate::TrackComponentOf::LineNames(_) => true,
        crate::TrackComponentOf::Repeat(repetition) => {
            has_track = true;
            matches!(repetition.repeat(), crate::TrackRepeat::Count(_))
                && fixed_grid_axis(repetition.components())
        }
        crate::TrackComponentOf::Subgrid(_) => false,
    });
    fixed && has_track
}

fn admit_positioned_root<Node, S: LayoutScalar, M>(
    root: Node,
    style: &NodeInputOf<S>,
    local_context: Option<crate::RootLocalPositionedContextOf<S>>,
) -> LayoutResultOf<Node, (), S, M>
where
    Node: Copy,
{
    let horizontal = style.writing_mode == crate::WritingMode::HorizontalTb;
    let local_containing_size = local_context.map(|context| context.containing_size());
    let has_block_offset = local_context
        .and_then(|context| context.hypothetical_block_offset())
        .is_some();
    let orthogonal_auto_inline = has_block_offset
        && matches!(
            style.writing_mode,
            crate::WritingMode::VerticalLr | crate::WritingMode::VerticalRl
        )
        && style.display == crate::Display::Block
        && style.block_generation == crate::BlockGeneration::Principal
        && !style.item_is_table
        && style.size.height.is_auto();
    let auto_fixed_grid = has_block_offset
        && style.display == crate::Display::Grid
        && style.size.width.is_auto()
        && style.size.height.is_auto()
        && fixed_grid_axis(&style.grid_template_columns)
        && fixed_grid_axis(&style.grid_template_rows);
    let edges = match local_containing_size {
        Some(size) if horizontal => RootEdgeProfile::DefiniteInline {
            percentage_parent: size.width,
        },
        _ => RootEdgeProfile::Zero,
    };
    admit_root_box(root, style, crate::Position::Absolute, edges)?;
    let writing_mode_error = || root_context_error(root, crate::RootContextMismatch::WritingMode);
    if (!horizontal || has_block_offset) && !auto_fixed_grid {
        let basis = local_containing_size.ok_or_else(writing_mode_error)?;
        // Resolve only for admission. The normal box algorithm retains the
        // original preferred dimensions and owns their min/max constraints.
        for (axis, preferred, basis) in [
            (PhysicalAxis::Horizontal, &style.size.width, basis.width),
            (PhysicalAxis::Vertical, &style.size.height, basis.height),
        ] {
            if orthogonal_auto_inline && axis == PhysicalAxis::Vertical {
                continue;
            }
            let resolved = resolve_preferred_sizing(
                preferred,
                SizingAlgorithm::Block,
                axis,
                Some(basis.get()),
                false,
            )
            .map_err(|error| sizing_resolution_error(root, error))?;
            if !matches!(resolved, ResolvedPreferredSize::Definite(value) if value.is_finite()) {
                return Err(writing_mode_error());
            }
        }
    }
    if style.item_is_replaced {
        return Err(root_context_error(
            root,
            crate::RootContextMismatch::Replaced,
        ));
    }
    if style.aspect_ratio.is_some() {
        return Err(root_context_error(
            root,
            crate::RootContextMismatch::AspectRatio,
        ));
    }
    if style.align_self.is_some() || style.justify_self.is_some() {
        return Err(root_context_error(
            root,
            crate::RootContextMismatch::Alignment,
        ));
    }
    Ok(())
}

fn positioned_auto_inline_size<'input, Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    containing: crate::ContainingLayoutContext,
    sizing_request: SizingRequestOf<Tree::Scalar>,
    parent: Size<Option<Tree::Scalar>>,
    inline_axis: PhysicalAxis,
) -> LayoutResultOf<Tree::Node, Tree::Scalar, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let available = root_physical_axis_value(sizing_request.axes(), inline_axis)
        .space()
        .definite_size()
        .ok_or_else(|| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(root),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::RequiredBasis),
            )
        })?
        .get();
    let opposite_axis = inline_axis.other();
    let opposite_parent = root_physical_axis_value(parent, opposite_axis);
    let opposite = match resolve_preferred_sizing(
        root_physical_axis_value(
            Size::new(&style.size.width, &style.size.height),
            opposite_axis,
        ),
        SizingAlgorithm::Block,
        opposite_axis,
        opposite_parent,
        false,
    )
    .map_err(|error| sizing_resolution_error(root, error))?
    {
        ResolvedPreferredSize::Definite(value) => {
            let minimum = resolve_minimum_optional(
                root_physical_axis_value(
                    Size::new(&style.min_size.width, &style.min_size.height),
                    opposite_axis,
                ),
                SizingAlgorithm::Block,
                opposite_axis,
                opposite_parent,
                false,
            )
            .map_err(|error| sizing_resolution_error(root, error))?;
            let maximum = resolve_maximum_optional(
                root_physical_axis_value(
                    Size::new(&style.max_size.width, &style.max_size.height),
                    opposite_axis,
                ),
                SizingAlgorithm::Block,
                opposite_axis,
                opposite_parent,
                false,
            )
            .map_err(|error| sizing_resolution_error(root, error))?;
            Some(value.clamp_max_before_min_optional(minimum, maximum))
        }
        ResolvedPreferredSize::Auto
        | ResolvedPreferredSize::MinContent
        | ResolvedPreferredSize::MaxContent => None,
    };
    let opposite = opposite
        .map(|value| {
            crate::NonNegativeFiniteOf::new(value)
                .map(crate::NonNegativeFiniteOf::get)
                .map_err(|_| {
                    sizing_resolution_error(
                        root,
                        SizingResolutionError::Status(
                            crate::LengthResolutionStatus::InvalidNumeric { value },
                        ),
                    )
                })
        })
        .transpose()?;
    let known_opposite =
        opposite.map_or(Size::NONE, |value| root_known_on_axis(opposite_axis, value));
    let query = |tree: &mut Tree, operation| {
        let mut axes = sizing_request.axes();
        let requested_axis = match inline_axis {
            PhysicalAxis::Horizontal => {
                axes.width = axes.width.with_operation(operation);
                crate::RequestedAxis::Horizontal
            }
            PhysicalAxis::Vertical => {
                axes.height = axes.height.with_operation(operation);
                crate::RequestedAxis::Vertical
            }
        };
        tree.compute_child(
            root,
            ComputeInputOf::for_child(
                crate::RunMode::ComputeSize,
                crate::SizingMode::ContentSize,
                requested_axis,
                known_opposite,
                parent,
                containing,
                sizing_request.with_axes(axes),
            ),
        )
    };
    let minimum_content =
        root_physical_axis_value(query(tree, SizingOperation::MinContent)?.size, inline_axis);
    let maximum_content =
        root_physical_axis_value(query(tree, SizingOperation::MaxContent)?.size, inline_axis);
    for value in [minimum_content, maximum_content] {
        if !value.is_finite() || value < Tree::Scalar::ZERO {
            return Err(crate::error::sizing_resolution_error(
                root,
                SizingResolutionError::Status(crate::LengthResolutionStatus::InvalidNumeric {
                    value,
                }),
            ));
        }
    }
    let minimum = resolve_minimum_optional(
        root_physical_axis_value(
            Size::new(&style.min_size.width, &style.min_size.height),
            inline_axis,
        ),
        SizingAlgorithm::Block,
        inline_axis,
        root_physical_axis_value(parent, inline_axis),
        false,
    )
    .map_err(|error| sizing_resolution_error(root, error))?;
    let maximum = resolve_maximum_optional(
        root_physical_axis_value(
            Size::new(&style.max_size.width, &style.max_size.height),
            inline_axis,
        ),
        SizingAlgorithm::Block,
        inline_axis,
        root_physical_axis_value(parent, inline_axis),
        false,
    )
    .map_err(|error| sizing_resolution_error(root, error))?;
    Ok(available
        .max(minimum_content)
        .min(maximum_content)
        .clamp_max_before_min_optional(minimum, maximum))
}

struct RootEdges<S: LayoutScalar> {
    padding: Edges<S>,
    border: Edges<S>,
    margin: Edges<S>,
}

fn root_scroll_geometry<Node, S, M>(
    node: Node,
    style: &NodeInputOf<S>,
    output: ComputeOutputOf<S>,
    edges: &RootEdges<S>,
) -> LayoutResultOf<Node, Option<crate::ScrollGeometryOf<S>>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    if style.display == crate::Display::None {
        return Ok(None);
    }

    let settled_auto_scrollbars = SettledAutoScrollbarState::INITIAL;
    let source = match output.scroll_geometry {
        Some(ref geometry) => CanonicalRetainedScrollSourceOf::Existing(geometry),
        None => CanonicalRetainedScrollSourceOf::Reconstruct {
            content_size: output.content_size,
        },
    };
    CanonicalScrollSourceBuilderOf::for_node(
        crate::scroll::ScrollBoxProjection::from_node(style),
        crate::scroll::ScrollTargetProjection::from_node(style),
        output.size,
        edges.border,
        edges.padding,
        settled_auto_scrollbars,
        ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
    )
    .geometry_from_retained_source(
        source,
        CanonicalScrollRangeSeedPolicy::IncludeReservedGutter,
    )
    .map(Some)
    .map_err(|error| root_scroll_error(node, error))
}

fn root_scroll_error<Node, S, M, E>(node: Node, error: E) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let _ = error;
    let kind =
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidRootScrollGeometry);

    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RootLayout,
        kind,
    )
}

type RootKnownInlineResult<Node, S, M> = LayoutResultOf<Node, Size<Option<S>>, S, M>;

fn resolve_root_edges<'input, Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    containing_flow_axes: FlowAxes,
    parent: Size<Option<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, RootEdges<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.padding, parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.border, parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let margin = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.margin, parent, |length, basis| {
            resolve_auto_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;

    Ok(RootEdges {
        padding,
        border,
        margin,
    })
}

fn root_known_inline<'input, Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    style: &NodeInputOf<Tree::Scalar>,
    containing_flow_axes: FlowAxes,
    fill_request: Size<AxisSizingRequestOf<Tree::Scalar>>,
    percentage_parent: Size<Option<Tree::Scalar>>,
) -> RootKnownInlineResult<<Tree as Traverse>::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let inline_axis = containing_flow_axes.inline_axis();
    let min_inline = root_physical_axis_value(
        Size::new(&style.min_size.width, &style.min_size.height),
        inline_axis,
    );
    if FlowAxes::new(style.writing_mode, style.direction).inline_axis() != inline_axis
        || style.display.is_inline_level()
        || style.item_is_replaced
        || !root_physical_axis_value(
            Size::new(&style.size.width, &style.size.height),
            inline_axis,
        )
        .is_auto()
        || !(min_inline.is_auto()
            || style.display == crate::Display::Flex
                && min_inline.is_calculation()
                && !tree.has_leaf_measurement(node))
    {
        return Ok(Size::NONE);
    }

    let request = root_physical_axis_value(fill_request, inline_axis);
    if request.operation() != SizingOperation::Normal {
        return Ok(Size::NONE);
    }
    let Some(available_inline) = request
        .space()
        .definite_size()
        .map(crate::NonNegativeFiniteOf::get)
    else {
        return Ok(Size::NONE);
    };
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.padding, percentage_parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(style.border, percentage_parent, |length, basis| {
            resolve_length_or_zero_fallible(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let padding_border_size = (padding + border).sum_axes();
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        padding_border_size
    } else {
        Size::ZERO
    };
    let max_size = Size::new(
        resolve_maximum_optional(
            &style.max_size.width,
            SizingAlgorithm::Block,
            PhysicalAxis::Horizontal,
            percentage_parent.width,
            false,
        ),
        resolve_maximum_optional(
            &style.max_size.height,
            SizingAlgorithm::Block,
            PhysicalAxis::Vertical,
            percentage_parent.height,
            false,
        ),
    );
    if matches!(max_size.width, Err(SizingResolutionError::Unsupported(_)))
        || matches!(max_size.height, Err(SizingResolutionError::Unsupported(_)))
    {
        // The root optimization cannot know whether a root with block display is
        // measured as a leaf. Defer contextual rejection to the actual consumer.
        return Ok(Size::NONE);
    }
    let max_size = Size::new(
        max_size
            .width
            .map_err(|error| sizing_resolution_error(node, error))?,
        max_size
            .height
            .map_err(|error| sizing_resolution_error(node, error))?,
    )
    .add_optional(box_sizing_adjustment);

    let minimum = resolve_minimum_optional(
        min_inline,
        SizingAlgorithm::Block,
        inline_axis,
        root_physical_axis_value(percentage_parent, inline_axis),
        true,
    )
    .map_err(|error| sizing_resolution_error(node, error))?
    .map(|minimum| minimum + root_physical_axis_value(box_sizing_adjustment, inline_axis));

    Ok(root_known_on_axis(
        inline_axis,
        available_inline.clamp_max_before_min_optional(
            minimum,
            root_physical_axis_value(max_size, inline_axis),
        ),
    ))
}

fn root_physical_axis_value<T>(size: Size<T>, axis: PhysicalAxis) -> T {
    match axis {
        PhysicalAxis::Horizontal => size.width,
        PhysicalAxis::Vertical => size.height,
    }
}

fn root_known_on_axis<S: LayoutScalar>(axis: PhysicalAxis, value: S) -> Size<Option<S>> {
    match axis {
        PhysicalAxis::Horizontal => Size::new(Some(value), None),
        PhysicalAxis::Vertical => Size::new(None, Some(value)),
    }
}

fn root_start_location<S: LayoutScalar>(
    containing_flow_axes: FlowAxes,
    root_size: Size<S>,
    available: Size<AvailableSpaceOf<S>>,
) -> Point<S> {
    Point::new(
        root_start_coordinate(
            containing_flow_axes.inline_start(),
            containing_flow_axes.block_start(),
            root_size,
            available,
            PhysicalAxis::Horizontal,
        ),
        root_start_coordinate(
            containing_flow_axes.inline_start(),
            containing_flow_axes.block_start(),
            root_size,
            available,
            PhysicalAxis::Vertical,
        ),
    )
}

fn root_start_coordinate<S: LayoutScalar>(
    inline_start: PhysicalSide,
    block_start: PhysicalSide,
    root_size: Size<S>,
    available: Size<AvailableSpaceOf<S>>,
    axis: PhysicalAxis,
) -> S {
    let start_side = if inline_start.axis() == axis {
        inline_start
    } else {
        block_start
    };
    match start_side {
        PhysicalSide::Top | PhysicalSide::Left => S::ZERO,
        PhysicalSide::Right | PhysicalSide::Bottom => root_physical_axis_value(available, axis)
            .definite_size()
            .map(crate::NonNegativeFiniteOf::get)
            .map_or(S::ZERO, |extent| {
                extent - root_physical_axis_value(root_size, axis)
            }),
    }
}
