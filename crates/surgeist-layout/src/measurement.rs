use crate::error::{
    InvalidMeasurementOutputOf, LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf,
    LayoutInternalInvariant, LayoutInvalidInputOf, LayoutOperation, LayoutResultOf,
    SizingAlgorithm, sizing_resolution_error, sizing_resolution_error_at_site,
};
use crate::geometry::{FlowAxes, PhysicalAxis};
use crate::layout_math::{
    MaxBeforeMinScalarClampExt, MaxBeforeMinSizeClampExt, OptionalMinimumSizeFloorExt,
    OptionalSizeExt,
};
use crate::scroll::{
    CanonicalScrollGeometryErrorOf, MeasuredLeafContentBoxInsetSourceOf,
    MeasuredLeafScrollGeometrySourceOf, canonical_measured_leaf_scroll_geometry,
    measured_leaf_content_box_inset,
};
use crate::sizing::resolve::{
    ResolvedPreferredSize, SizingResolutionError, resolution_optional_fallible,
    resolution_or_zero_fallible, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_sizing,
};
use crate::{
    AspectRatioOf, AvailableSpaceOf, AxisSizingRequestOf, BoxSizing, CollapsibleMarginOf,
    ComputeInputOf, ComputeOutputOf, ContentSizingRequestOf, DefaultScalar, Edges, LayoutScalar,
    LayoutTree, LengthResolutionOf, LengthResolutionStatus, NodeInputOf, NonNegativeFiniteOf,
    PhysicalBlockMarginCollapseOf, RunMode, Size, SizingMode, SizingOperation,
};

#[cfg(test)]
use crate::SizingRequestOf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeafMeasureInputOf<S: LayoutScalar = DefaultScalar> {
    known_content_size: Size<Option<S>>,
    content_sizing_request: ContentSizingRequestOf<S>,
    flow_axes: FlowAxes,
    containing_layout_context: crate::ContainingLayoutContext,
}

pub type LeafMeasureInput = LeafMeasureInputOf<DefaultScalar>;

impl<S: LayoutScalar> LeafMeasureInputOf<S> {
    #[must_use]
    pub const fn known_content_size(&self) -> Size<Option<S>> {
        self.known_content_size
    }

    /// Effective content-space requests after owned property/edge lowering.
    /// The fallback extents are this leaf's final content-space projection, not a
    /// recursive ICB/scrollport frame to forward to another generated child.
    #[must_use]
    pub const fn content_sizing_request(&self) -> ContentSizingRequestOf<S> {
        self.content_sizing_request
    }

    /// Actual normalized used axes of the measured node, supplied by Layout.
    #[must_use]
    pub const fn flow_axes(&self) -> FlowAxes {
        self.flow_axes
    }

    /// The exact containing boundary supplied to the compute operation.
    ///
    /// Opaque providers own their actual content-formatting policy. These facts
    /// permit a supported profile to select a required orthogonal inline
    /// fallback, without guessing the frame or capping every indefinite Normal
    /// request. They do not declare a general orthogonal-layout algorithm.
    #[must_use]
    pub const fn containing_layout_context(&self) -> crate::ContainingLayoutContext {
        self.containing_layout_context
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LeafMeasureErrorOf<S: LayoutScalar, M> {
    Provider(M),
    InvalidOutput(InvalidMeasurementOutputOf<S>),
}

pub type LeafMeasureError<M> = LeafMeasureErrorOf<DefaultScalar, M>;

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeafBoxEdges<S: LayoutScalar> {
    padding: Edges<S>,
    border: Edges<S>,
    padding_size: Size<S>,
    padding_border_size: Size<S>,
}

impl<S: LayoutScalar> LeafBoxEdges<S> {
    fn new(padding: Edges<S>, border: Edges<S>) -> Result<Self, LengthResolutionStatus<S>> {
        let validate = |value: S| {
            NonNegativeFiniteOf::new(value)
                .map(NonNegativeFiniteOf::get)
                .map_err(|_| LengthResolutionStatus::InvalidNumeric { value })
        };
        let padding = transpose_leaf_edges(padding.map(validate))?;
        let border = transpose_leaf_edges(border.map(validate))?;
        let padding_border = transpose_leaf_edges((padding + border).map(validate))?;
        let validate_size =
            |size: Size<S>| Ok(Size::new(validate(size.width)?, validate(size.height)?));
        let padding_size = validate_size(padding.sum_axes())?;
        // Validate each resolved box component as well as their combined inset.
        validate_size(border.sum_axes())?;
        let padding_border_size = validate_size(padding_border.sum_axes())?;
        Ok(Self {
            padding,
            border,
            padding_size,
            padding_border_size,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeafResolvedValues<S: LayoutScalar> {
    margin: Edges<S>,
    box_edges: LeafBoxEdges<S>,
    node_size: Size<Option<S>>,
    node_min_size: Size<Option<S>>,
    node_max_size: Size<Option<S>>,
    preferred_intrinsic_operation: Size<Option<SizingOperation>>,
    flex_basis_intrinsic_operation: Size<Option<SizingOperation>>,
    aspect_ratio: Option<AspectRatioOf<S>>,
}

fn resolve_leaf_values<S>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolve_auto: impl Fn(crate::LengthAutoOf<S>, Option<S>) -> Result<S, LengthResolutionStatus<S>>,
    resolve_length: impl Fn(crate::LengthOf<S>, Option<S>) -> Result<S, LengthResolutionStatus<S>>,
) -> Result<LeafResolvedValues<S>, SizingResolutionError<S>>
where
    S: LayoutScalar,
{
    let margin = transpose_leaf_edges(
        input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.margin, input.parent(), resolve_auto),
    )?;
    let padding = transpose_leaf_edges(
        input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.padding, input.parent(), &resolve_length),
    )?;
    let border = transpose_leaf_edges(
        input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(style.border, input.parent(), resolve_length),
    )?;
    let box_edges = LeafBoxEdges::new(padding, border)?;
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        box_edges.padding_border_size
    } else {
        Size::ZERO
    };

    let (node_size, node_min_size, node_max_size, preferred_intrinsic_operation, aspect_ratio) =
        match input.sizing_mode() {
            SizingMode::ContentSize | SizingMode::PositionedColumnFlexInline => {
                (input.known(), Size::NONE, Size::NONE, Size::NONE, None)
            }
            SizingMode::InherentSize => {
                let missing_basis_is_indefinite = input.run_mode() == RunMode::ComputeSize;
                let (style_size, preferred_intrinsic_operation) = if input.known().width.is_some()
                    && input.known().height.is_some()
                {
                    // Both used border dimensions are authoritative. Their own
                    // preferred size and intrinsic operation are no longer a
                    // demand; min/max and box-edge resolution remain independent.
                    (Size::NONE, Size::NONE)
                } else {
                    let preferred = Size::new(
                        resolve_preferred_sizing(
                            &style.size.width,
                            SizingAlgorithm::Leaf,
                            PhysicalAxis::Horizontal,
                            input.parent().width,
                            missing_basis_is_indefinite,
                        )?,
                        resolve_preferred_sizing(
                            &style.size.height,
                            SizingAlgorithm::Leaf,
                            PhysicalAxis::Vertical,
                            input.parent().height,
                            missing_basis_is_indefinite,
                        )?,
                    );
                    let style_size = preferred.map(|resolved| match resolved {
                        ResolvedPreferredSize::Definite(value) => Some(value),
                        ResolvedPreferredSize::Auto
                        | ResolvedPreferredSize::MinContent
                        | ResolvedPreferredSize::MaxContent => None,
                    });
                    // With an allocated width, an automatic height transfers from
                    // that allocation during measurement.
                    let height_is_intrinsic = matches!(
                        preferred.height,
                        ResolvedPreferredSize::MinContent | ResolvedPreferredSize::MaxContent
                    );
                    let style_size = if input.known().width.is_some() || height_is_intrinsic {
                        style_size
                    } else {
                        style_size.apply_aspect_ratio(style.aspect_ratio)
                    };
                    let style_size = validate_resolved_leaf_size(
                        style_size.add_optional(box_sizing_adjustment),
                    )?;
                    let preferred_intrinsic_operation = preferred.map(|resolved| match resolved {
                        ResolvedPreferredSize::MinContent => Some(SizingOperation::MinContent),
                        ResolvedPreferredSize::MaxContent => Some(SizingOperation::MaxContent),
                        ResolvedPreferredSize::Auto | ResolvedPreferredSize::Definite(_) => None,
                    });
                    (style_size, preferred_intrinsic_operation)
                };
                let style_min_size = validate_resolved_leaf_size(
                    Size::new(
                        resolve_minimum_optional(
                            &style.min_size.width,
                            SizingAlgorithm::Leaf,
                            PhysicalAxis::Horizontal,
                            input.parent().width,
                            missing_basis_is_indefinite,
                        )?,
                        resolve_minimum_optional(
                            &style.min_size.height,
                            SizingAlgorithm::Leaf,
                            PhysicalAxis::Vertical,
                            input.parent().height,
                            missing_basis_is_indefinite,
                        )?,
                    )
                    .apply_aspect_ratio(style.aspect_ratio)
                    .add_optional(box_sizing_adjustment),
                )?;
                let style_max_size = validate_resolved_leaf_size(
                    Size::new(
                        resolve_maximum_optional(
                            &style.max_size.width,
                            SizingAlgorithm::Leaf,
                            PhysicalAxis::Horizontal,
                            input.parent().width,
                            missing_basis_is_indefinite,
                        )?,
                        resolve_maximum_optional(
                            &style.max_size.height,
                            SizingAlgorithm::Leaf,
                            PhysicalAxis::Vertical,
                            input.parent().height,
                            missing_basis_is_indefinite,
                        )?,
                    )
                    .add_optional(box_sizing_adjustment),
                )?;

                (
                    input.known().or(style_size),
                    style_min_size,
                    style_max_size,
                    preferred_intrinsic_operation,
                    style.aspect_ratio,
                )
            }
        };
    let flex_basis_intrinsic =
        if input.parent_formatting_context() == crate::ParentFormattingContext::Flex {
            if style.flex_basis.is_min_content() {
                Some(SizingOperation::MinContent)
            } else if style.flex_basis.is_max_content() {
                Some(SizingOperation::MaxContent)
            } else {
                None
            }
        } else {
            None
        };
    let flex_basis_intrinsic_operation = input
        .sizing_request()
        .axes()
        .map(|request| flex_basis_intrinsic.filter(|intrinsic| *intrinsic == request.operation()));

    Ok(LeafResolvedValues {
        margin,
        box_edges,
        node_size,
        node_min_size,
        node_max_size,
        preferred_intrinsic_operation,
        flex_basis_intrinsic_operation,
        aspect_ratio,
    })
}

fn resolve_leaf_values_for_input<S>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
) -> Result<LeafResolvedValues<S>, SizingResolutionError<S>>
where
    S: LayoutScalar,
{
    resolve_leaf_values(
        input,
        style,
        |length, basis| resolve_leaf_auto(input, length, basis),
        |length, basis| resolve_leaf_length(input, length, basis),
    )
}

fn transpose_leaf_edges<S, E>(edges: Edges<Result<S, E>>) -> Result<Edges<S>, E> {
    Ok(Edges::new(
        edges.top?,
        edges.right?,
        edges.bottom?,
        edges.left?,
    ))
}

fn validate_resolved_leaf_size<S: LayoutScalar>(
    size: Size<Option<S>>,
) -> Result<Size<Option<S>>, LengthResolutionStatus<S>> {
    for value in [size.width, size.height].into_iter().flatten() {
        if !value.is_finite() {
            return Err(LengthResolutionStatus::InvalidNumeric { value });
        }
    }
    Ok(size)
}

fn validate_leaf_size<S: LayoutScalar>(size: Size<S>) -> Result<Size<S>, S> {
    for value in [size.width, size.height] {
        if !value.is_finite() || value < S::ZERO {
            return Err(value);
        }
    }
    Ok(size)
}

pub fn compute_leaf<S, M>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    mut measure: impl FnMut(LeafMeasureInputOf<S>) -> Result<Size<S>, M>,
) -> LayoutResultOf<(), ComputeOutputOf<S>, S, M>
where
    S: LayoutScalar,
{
    let site = LayoutErrorSiteOf::Standalone;
    let resolved = resolve_leaf_values_for_input(input, style)
        .map_err(|error| sizing_resolution_error_at_site(site, error))?;

    compute_leaf_with_resolved_values(site, input, style, resolved, None, |measure_input| {
        measure(measure_input).map_err(|error| {
            LayoutErrorOf::new(
                site,
                LayoutOperation::LeafMeasurement,
                LayoutErrorKindOf::Measurement(error),
            )
        })
    })
}

pub(crate) fn compute_tree_leaf<Tree>(
    tree: &Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    style: &NodeInputOf<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    compute_tree_leaf_with_intrinsic_icb(tree, node, input, style, None)
}

pub(crate) fn compute_tree_leaf_with_intrinsic_icb<Tree>(
    tree: &Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    style: &NodeInputOf<Tree::Scalar>,
    intrinsic_icb: Option<Size<NonNegativeFiniteOf<Tree::Scalar>>>,
) -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    compute_leaf_with_sampler(node, input, style, intrinsic_icb, |request| {
        sample_tree_leaf(tree, node, request)
    })
}

pub(crate) fn sample_tree_leaf<Tree: LayoutTree>(
    tree: &Tree,
    node: Tree::Node,
    request: LeafMeasureInputOf<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, Size<Tree::Scalar>, Tree::Scalar, Tree::MeasureError> {
    let site = LayoutErrorSiteOf::Node(node);
    match tree.measure_leaf(node, request) {
        Some(Ok(output)) => Ok(output),
        Some(Err(error)) => Err(LayoutErrorOf::new(
            site,
            LayoutOperation::LeafMeasurement,
            LayoutErrorKindOf::Measurement(error),
        )),
        None => Err(LayoutErrorOf::new(
            site,
            LayoutOperation::LeafMeasurement,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::MissingLeafMeasurementProvider,
            ),
        )),
    }
}

/// Private producer injection for operation-local native observations. All
/// property lowering, used-size/scrollbar passes and output validation remain
/// in this measurement owner; a sampler supplies only the actual raw result.
pub(crate) fn compute_leaf_with_sampler<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    intrinsic_icb: Option<Size<NonNegativeFiniteOf<S>>>,
    sample: impl FnMut(LeafMeasureInputOf<S>) -> LayoutResultOf<Node, Size<S>, S, M>,
) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M> {
    let resolved = resolve_leaf_values_for_input(input, style)
        .map_err(|error| sizing_resolution_error(node, error))?;
    compute_leaf_with_resolved_values(
        LayoutErrorSiteOf::Node(node),
        input,
        style,
        resolved,
        intrinsic_icb,
        sample,
    )
}

fn compute_leaf_with_resolved_values<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    mut resolved: LeafResolvedValues<S>,
    intrinsic_icb: Option<Size<NonNegativeFiniteOf<S>>>,
    measure: impl FnMut(LeafMeasureInputOf<S>) -> LayoutResultOf<Node, Size<S>, S, M>,
) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let containment = crate::engine::containment::size_containment(style);
    if containment != crate::SizeContainment::None {
        let sizing_input = crate::engine::containment::empty_sizing_input(input);
        let empty =
            compute_leaf_passes(site, sizing_input, style, resolved, intrinsic_icb, |_| {
                Ok(Size::ZERO)
            })?;
        let needs_block_margin_preview = input.parent_formatting_context()
            == crate::ParentFormattingContext::BlockFlow
            && style.display == crate::Display::Block
            && !style.item_is_replaced
            && input.sizing_mode() == SizingMode::InherentSize
            && input
                .containing_flow_axes()
                .logical_size(input.sizing_request().axes())
                .inline
                .operation()
                == SizingOperation::Normal;
        if containment == crate::SizeContainment::BothAxes
            && !input.run_mode().is_perform_layout()
            && !needs_block_margin_preview
        {
            return Ok(empty);
        }
        // Lower the accepted allocation into the provider's actual content
        // space after resolving the original request and authored bounds.
        resolved.node_size = crate::engine::containment::contained_outer_size(
            containment,
            crate::FlowAxes::new(style.writing_mode, style.direction),
            empty.size,
        )
        .or(resolved.node_size);
    }
    compute_leaf_passes(site, input, style, resolved, intrinsic_icb, measure)
}

fn compute_leaf_passes<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolved: LeafResolvedValues<S>,
    intrinsic_icb: Option<Size<NonNegativeFiniteOf<S>>>,
    mut measure: impl FnMut(LeafMeasureInputOf<S>) -> LayoutResultOf<Node, Size<S>, S, M>,
) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let LeafResolvedValues {
        box_edges:
            LeafBoxEdges {
                padding,
                border,
                padding_border_size,
                ..
            },
        node_size,
        node_min_size,
        node_max_size,
        ..
    } = resolved;
    let arithmetic_error =
        |value| invalid_numeric_error_at_site(site, LayoutOperation::LeafMeasurement, value);
    let leaf_flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let block_start = leaf_flow_axes.block_start();
    let block_end = leaf_flow_axes.block_end();
    let node_block_size = match leaf_flow_axes.block_axis() {
        PhysicalAxis::Horizontal => node_size.width,
        PhysicalAxis::Vertical => node_size.height,
    };
    let node_min_block_size = match leaf_flow_axes.block_axis() {
        PhysicalAxis::Horizontal => node_min_size.width,
        PhysicalAxis::Vertical => node_min_size.height,
    };

    let prevents_margin_collapse = input.parent_formatting_context()
        != crate::ParentFormattingContext::BlockFlow
        || style.display != crate::Display::Block
        || crate::block::flow_root::node_establishes_flow_root(
            style,
            input.containing_flow_axes().writing_mode(),
        )
        || padding.at_physical_side(block_start) > S::ZERO
        || padding.at_physical_side(block_end) > S::ZERO
        || border.at_physical_side(block_start) > S::ZERO
        || border.at_physical_side(block_end) > S::ZERO
        || matches!(node_block_size, Some(size) if size > S::ZERO)
        || matches!(node_min_block_size, Some(size) if size > S::ZERO);

    if input.run_mode() == RunMode::ComputeSize
        && prevents_margin_collapse
        && let Size {
            width: Some(width),
            height: Some(height),
        } = node_size
    {
        let size = Size::new(width, height)
            .clamp_max_before_min_optional(node_min_size, node_max_size)
            .max_optional(padding_border_size.map(Some));
        return Ok(ComputeOutputOf::from_outer_size(
            validate_leaf_size(size).map_err(arithmetic_error)?,
        ));
    }

    let mut pass_input = input;
    let mut reusable_measurement = None;
    loop {
        let pass = leaf_pass_input(site, pass_input, style, &resolved, intrinsic_icb)?;
        let measured = match reusable_measurement.take() {
            Some((measurement_input, measured)) if measurement_input == pass.measurement_input => {
                measured
            }
            _ => validate_measurement_output(measure(pass.measurement_input)?)
                .map_err(|error| leaf_measurement_error_at_site(site, error))?,
        };
        let unclamped = pass_input
            .known()
            .or(resolved.node_size)
            .unwrap_or(measured + pass.content_box_inset_size);
        // Check before clamping: a finite maximum must not conceal overflow.
        for value in [unclamped.width, unclamped.height] {
            if !value.is_finite() {
                return Err(arithmetic_error(value));
            }
        }
        let height_is_definite =
            pass_input.known().height.is_some() || resolved.node_size.height.is_some();
        let aspect_height =
            if height_is_definite || resolved.preferred_intrinsic_operation.height.is_some() {
                unclamped.height
            } else if let Some(ratio) = resolved.aspect_ratio {
                let insets = if style.box_sizing == BoxSizing::ContentBox {
                    padding_border_size
                } else {
                    Size::ZERO
                };
                let width = (unclamped.width - insets.width).max(S::ZERO);
                let ratio_height = width / ratio.get() + insets.height;
                if !ratio_height.is_finite() {
                    return Err(arithmetic_error(ratio_height));
                }
                let has_automatic_minimum = style.min_size.height.is_auto()
                    && !style.item_is_replaced
                    && !style.overflow.y().is_scrollable();
                if pass_input.known().width.is_none() || has_automatic_minimum {
                    unclamped.height.max(ratio_height)
                } else {
                    ratio_height
                }
            } else {
                unclamped.height
            };
        let aspect_size = Size::new(unclamped.width, aspect_height)
            .clamp_max_before_min_optional(resolved.node_min_size, resolved.node_max_size)
            .max_optional(padding_border_size.map(Some));

        let aspect_size = validate_leaf_size(aspect_size).map_err(arithmetic_error)?;
        let content_size = validate_leaf_size(measured + resolved.box_edges.padding_size)
            .map_err(arithmetic_error)?;
        let mut output = ComputeOutputOf::from_sizes(aspect_size, content_size);
        let can_collapse_through = !prevents_margin_collapse
            && leaf_flow_axes.logical_size(aspect_size).block == S::ZERO
            && leaf_flow_axes.logical_size(measured).block == S::ZERO;
        output.block_margin_collapse = PhysicalBlockMarginCollapseOf::from_block_flow(
            leaf_flow_axes,
            CollapsibleMarginOf::ZERO,
            CollapsibleMarginOf::ZERO,
            can_collapse_through,
        );
        let geometry =
            canonical_measured_leaf_scroll_geometry(MeasuredLeafScrollGeometrySourceOf {
                box_projection: crate::scroll::ScrollBoxProjection::from_node(style),
                target_projection: crate::scroll::ScrollTargetProjection::from_node(style),
                border_box_size: aspect_size,
                border: resolved.box_edges.border,
                padding: resolved.box_edges.padding,
                settled_auto_scrollbars: pass_input.settled_auto_scrollbars(),
                measured_content_size: measured,
            })
            .map_err(|error| leaf_scroll_error_at_site(site, pass_input.run_mode(), error))?;
        let next_state = pass_input.settled_auto_scrollbars().transition(geometry);
        if next_state == pass_input.settled_auto_scrollbars()
            || style.scrollbar_width.get() == S::ZERO
        {
            if input.run_mode().is_perform_layout() {
                output.content_size = geometry.canonical_content_size().map_err(|error| {
                    leaf_scroll_error_at_site(site, pass_input.run_mode(), error)
                })?;
                output.scroll_geometry = Some(geometry);
            }
            validate_leaf_size(output.content_size).map_err(arithmetic_error)?;
            return Ok(output);
        }

        reusable_measurement = Some((pass.measurement_input, measured));
        pass_input = pass_input.with_settled_auto_scrollbars(next_state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeafPassInputOf<S: LayoutScalar> {
    content_box_inset_size: Size<S>,
    measurement_input: LeafMeasureInputOf<S>,
}

fn leaf_pass_input<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    resolved: &LeafResolvedValues<S>,
    intrinsic_icb: Option<Size<NonNegativeFiniteOf<S>>>,
) -> LayoutResultOf<Node, LeafPassInputOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let content_box_inset = measured_leaf_content_box_inset(MeasuredLeafContentBoxInsetSourceOf {
        flow_axes: FlowAxes::new(style.writing_mode, style.direction),
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars: input.settled_auto_scrollbars(),
        padding: resolved.box_edges.padding,
        border: resolved.box_edges.border,
    });
    let content_box_inset_size =
        validate_leaf_size(content_box_inset.sum_axes()).map_err(|value| {
            invalid_numeric_error_at_site(site, LayoutOperation::LeafMeasurement, value)
        })?;
    let axes = input.sizing_request().axes();
    let facts = |axis: PhysicalAxis| LeafAxisSpace {
        known: physical_axis(input.known(), axis),
        node_size: physical_axis(resolved.node_size, axis),
        minimum: physical_axis(resolved.node_min_size, axis),
        maximum: physical_axis(resolved.node_max_size, axis),
        margin: match axis {
            PhysicalAxis::Horizontal => resolved.margin.horizontal_sum(),
            PhysicalAxis::Vertical => resolved.margin.vertical_sum(),
        },
        inset: physical_axis(content_box_inset_size, axis),
        intrinsic: physical_axis(resolved.flex_basis_intrinsic_operation, axis)
            .or(physical_axis(resolved.preferred_intrinsic_operation, axis)),
    };
    let lower = |axis| {
        let mut axis_facts = facts(axis);
        if input.border_box_offer_axis() == Some(axis) {
            axis_facts.margin = S::ZERO;
        }
        leaf_content_axis(physical_axis(axes, axis), axis_facts).map_err(|value| {
            invalid_numeric_error_at_site(site, LayoutOperation::LeafMeasurement, value)
        })
    };
    let content_axes = Size::new(
        lower(PhysicalAxis::Horizontal)?,
        lower(PhysicalAxis::Vertical)?,
    );
    let fallback = input.sizing_request().orthogonal_fallback();
    // Intrinsic purpose is a property of this effective query, not RunMode.
    // A provider can need the opposite ICB axis to resolve that component.
    let fallback = if (content_axes.width.operation() != SizingOperation::Normal
        || content_axes.height.operation() != SizingOperation::Normal)
        && let Some(icb) = intrinsic_icb
    {
        crate::OrthogonalFallbackOf::new(
            icb.map(Some),
            fallback.nearest_scrollport(),
            fallback.containing_limits(),
        )
    } else {
        fallback
    };
    let lower_fallback = |axis| {
        let facts = facts(axis);
        // An unresolved/unrepresentable fallback is carried as a precise Result.
        // A provider that does not use that axis must not fail because of it.
        content_fallback(fallback, axis, facts.margin, facts.inset)
    };
    let content_sizing_request = ContentSizingRequestOf::new(
        content_axes,
        Size::new(
            lower_fallback(PhysicalAxis::Horizontal),
            lower_fallback(PhysicalAxis::Vertical),
        ),
    );

    let known_content_size = match input.run_mode() {
        RunMode::ComputeSize => input.known(),
        RunMode::PerformRootLayout | RunMode::PerformLayout => Size::NONE,
        RunMode::PerformHiddenLayout => {
            unreachable!("hidden layout uses ComputeOutput::HIDDEN")
        }
    }
    .zip_map(content_box_inset_size, |value, inset| {
        value.map(|value| finite_floor_at_zero(value - inset))
    });
    let known_content_size =
        transpose_leaf_known_content_size(known_content_size).map_err(|value| {
            invalid_numeric_error_at_site(site, LayoutOperation::LeafMeasurement, value)
        })?;
    let measurement_input = LeafMeasureInputOf {
        known_content_size,
        content_sizing_request,
        flow_axes: FlowAxes::new(style.writing_mode, style.direction),
        containing_layout_context: input.containing_layout_context(),
    };
    Ok(LeafPassInputOf {
        content_box_inset_size,
        measurement_input,
    })
}

fn leaf_scroll_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    run_mode: RunMode,
    _error: CanonicalScrollGeometryErrorOf<S>,
) -> LayoutErrorOf<Node, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let (operation, invariant) = match (site, run_mode) {
        (LayoutErrorSiteOf::Standalone, _) => (
            LayoutOperation::LeafMeasurement,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        ),
        (_, RunMode::PerformRootLayout) => (
            LayoutOperation::RootLayout,
            LayoutInternalInvariant::InvalidRootScrollGeometry,
        ),
        _ => (
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        ),
    };
    LayoutErrorOf::new(
        site,
        operation,
        LayoutErrorKindOf::InternalInvariant(invariant),
    )
}

pub(crate) fn validate_native_sample<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    measured: Size<S>,
) -> LayoutResultOf<Node, Size<S>, S, M> {
    validate_measurement_output(measured)
        .map_err(|error| leaf_measurement_error_at_site(LayoutErrorSiteOf::Node(node), error))
}

fn validate_measurement_output<S, M>(measured: Size<S>) -> Result<Size<S>, LeafMeasureErrorOf<S, M>>
where
    S: LayoutScalar,
{
    let width = NonNegativeFiniteOf::new(measured.width).map_err(|error| {
        LeafMeasureErrorOf::InvalidOutput(InvalidMeasurementOutputOf::new(
            PhysicalAxis::Horizontal,
            error,
        ))
    })?;
    let height = NonNegativeFiniteOf::new(measured.height).map_err(|error| {
        LeafMeasureErrorOf::InvalidOutput(InvalidMeasurementOutputOf::new(
            PhysicalAxis::Vertical,
            error,
        ))
    })?;

    Ok(Size::new(width.get(), height.get()))
}

fn finite_floor_at_zero<S>(value: S) -> Result<S, S>
where
    S: LayoutScalar,
{
    if value.is_finite() {
        Ok(value.max(S::ZERO))
    } else {
        Err(value)
    }
}

fn transpose_leaf_known_content_size<S>(
    size: Size<Option<Result<S, S>>>,
) -> Result<Size<Option<S>>, S>
where
    S: LayoutScalar,
{
    Ok(Size::new(size.width.transpose()?, size.height.transpose()?))
}

fn invalid_numeric_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    operation: LayoutOperation,
    value: S,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    LayoutErrorOf::new(
        site,
        operation,
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value }),
    )
}

fn leaf_measurement_error_at_site<Node, S, M>(
    site: LayoutErrorSiteOf<Node>,
    error: LeafMeasureErrorOf<S, M>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    let kind = match error {
        LeafMeasureErrorOf::Provider(error) => LayoutErrorKindOf::Measurement(error),
        LeafMeasureErrorOf::InvalidOutput(error) => {
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::MeasurementOutput(error))
        }
    };
    LayoutErrorOf::new(site, LayoutOperation::LeafMeasurement, kind)
}

fn resolve_leaf_auto<S>(
    input: ComputeInputOf<S>,
    length: crate::LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    Ok(resolve_leaf_optional(input, length.resolve_with_status(basis))?.unwrap_or(S::ZERO))
}

fn resolve_leaf_length<S>(
    input: ComputeInputOf<S>,
    length: crate::LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    let resolution = length.resolve_with_status(basis);
    if input.run_mode() == RunMode::ComputeSize
        && matches!(resolution.status(), LengthResolutionStatus::MissingBasis)
    {
        return Ok(S::ZERO);
    }

    resolution_or_zero_fallible(resolution)
}

fn resolve_leaf_optional<S>(
    input: ComputeInputOf<S>,
    resolution: LengthResolutionOf<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>>
where
    S: LayoutScalar,
{
    if input.run_mode() == RunMode::ComputeSize
        && matches!(resolution.status(), LengthResolutionStatus::MissingBasis)
    {
        return Ok(None);
    }

    resolution_optional_fallible(resolution)
}

fn content_fallback<S: LayoutScalar>(
    fallback: crate::OrthogonalFallbackOf<S>,
    axis: PhysicalAxis,
    margin: S,
    inset: S,
) -> Result<NonNegativeFiniteOf<S>, crate::OrthogonalFallbackErrorOf<S>> {
    fallback.resolve(axis).and_then(|value| {
        let raw = value.get() - margin - inset;
        finite_floor_at_zero(raw)
            .and_then(|value| NonNegativeFiniteOf::new(value).map_err(|_| value))
            .map_err(|value| crate::OrthogonalFallbackErrorOf::InvalidContentSpace { axis, value })
    })
}

#[derive(Clone, Copy)]
struct LeafAxisSpace<S: LayoutScalar> {
    known: Option<S>,
    node_size: Option<S>,
    minimum: Option<S>,
    maximum: Option<S>,
    margin: S,
    inset: S,
    intrinsic: Option<SizingOperation>,
}

fn physical_axis<T>(size: Size<T>, axis: PhysicalAxis) -> T {
    match axis {
        PhysicalAxis::Horizontal => size.width,
        PhysicalAxis::Vertical => size.height,
    }
}

fn leaf_content_axis<S: LayoutScalar>(
    request: AxisSizingRequestOf<S>,
    facts: LeafAxisSpace<S>,
) -> Result<AxisSizingRequestOf<S>, S> {
    let raw_space = facts.node_size.or(facts.known).or_else(|| {
        request
            .space()
            .definite_size()
            .map(|value| value.get() - facts.margin)
    });
    let space = match raw_space {
        Some(value) => {
            // Preserve invalid arithmetic before clamping: scalar min/max must not hide NaN.
            if !value.is_finite() {
                return Err(value);
            }
            let value =
                value.clamp_max_before_min_optional(facts.minimum, facts.maximum) - facts.inset;
            let value = finite_floor_at_zero(value)?;
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(value).map_err(|_| value)?)
        }
        None => AvailableSpaceOf::Indefinite,
    };
    // A resolved authored/imposed body size is a used-size authority. A finite
    // external space (including a clamped bound) is not. ContentSize suppresses
    // authored size in resolve_leaf_values, so only imposed known sizes
    // take this branch for that mode.
    let operation = if facts.known.is_some() || facts.node_size.is_some() {
        SizingOperation::Normal
    } else {
        facts.intrinsic.unwrap_or(request.operation())
    };
    Ok(AxisSizingRequestOf::new(space, operation))
}

#[cfg(test)]
mod fri08_c07_t03_optional_math_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(scalar(4.0), scalar(12.0)).max_optional(Size::new(None, Some(scalar(15.0)))),
            Size::new(scalar(4.0), scalar(15.0))
        );
        assert_eq!(
            Size::new(scalar(4.0), scalar(12.0))
                .max_optional(Size::new(Some(scalar(9.0)), Some(scalar(3.0)),)),
            Size::new(scalar(9.0), scalar(12.0))
        );
    }

    #[test]
    fn fri08_c07_t03_optional_math_compute_minimum_floor_preserves_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri08_c07_t03_optional_math_compute_minimum_floor_preserves_f64() {
        characterize::<f64>();
    }
}

#[cfg(test)]
mod fri06_c13_t05_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;
        let optional = Size::new(None, Some(scalar(9.0)));

        assert_eq!(
            optional.or(Size::new(Some(scalar(4.0)), Some(scalar(3.0)))),
            Size::new(Some(scalar(4.0)), Some(scalar(9.0)))
        );
        assert_eq!(
            optional.unwrap_or(Size::new(scalar(6.0), scalar(7.0))),
            Size::new(scalar(6.0), scalar(9.0))
        );
        assert_eq!(
            optional.add_optional(Size::new(scalar(2.0), scalar(3.0))),
            Size::new(None, Some(scalar(12.0)))
        );

        let Some(ratio) = AspectRatioOf::new(scalar(2.0)) else {
            panic!("finite positive test aspect ratio must be accepted");
        };
        assert_eq!(
            Size::new(Some(scalar(12.0)), None).apply_aspect_ratio(Some(ratio)),
            Size::new(Some(scalar(12.0)), Some(scalar(6.0)))
        );
        assert_eq!(
            Size::new(None, Some(scalar(7.0))).apply_aspect_ratio(Some(ratio)),
            Size::new(Some(scalar(14.0)), Some(scalar(7.0)))
        );
        assert_eq!(
            Size::new(scalar(8.0), scalar(12.0)).clamp_max_before_min_optional(
                Size::new(Some(scalar(3.0)), None),
                Size::new(Some(scalar(10.0)), Some(scalar(11.0))),
            ),
            Size::new(scalar(8.0), scalar(11.0))
        );
        assert_eq!(
            scalar(5.0).clamp_max_before_min_optional(Some(scalar(10.0)), Some(scalar(3.0))),
            scalar(10.0)
        );
    }

    #[test]
    fn fri06_c13_t05_compute_optional_math_and_clamp_order_preserve_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri06_c13_t05_compute_optional_math_and_clamp_order_preserve_f64() {
        characterize::<f64>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_fri06_c13_t06_leaf_missing_basis_counterexample<S: LayoutScalar>() {
        let style = NodeInputOf::<S> {
            padding: Edges::all(crate::LengthOf::percent(S::from_f64(0.5))),
            ..NodeInputOf::default()
        };
        let context = crate::ContainingLayoutContext::new(
            FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        );
        // This case isolates missing percentage-edge authority, not an intrinsic
        // size calculation: both direct entrypoints use Normal with
        // indefinite space and no supplied ICB/scrollport fallback frame.
        let request = SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::Normal,
            )),
            crate::OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Unknown,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
            ),
        );
        let measured = Size::new(S::from_f64(12.0), S::from_f64(8.0));

        let content_size_input =
            ComputeInputOf::leaf_content_size(Size::NONE, Size::NONE, context, request)
                .unwrap_or_else(|_| panic!("normal indefinite content-size input is valid"));
        let content_size = compute_leaf(content_size_input, &style, |_input| Ok::<_, ()>(measured))
            .unwrap_or_else(|_| panic!("ComputeSize explicitly treats missing edge basis as zero"));
        assert_eq!(content_size.size, measured);

        let layout_input = ComputeInputOf::leaf_layout(Size::NONE, Size::NONE, context, request)
            .unwrap_or_else(|_| panic!("indefinite layout leaf input is valid"));
        let error = compute_leaf(layout_input, &style, |_input| -> Result<Size<S>, ()> {
            panic!("layout missing-basis failure must precede measurement")
        })
        .expect_err("layout keeps missing edge basis fallible");
        assert_eq!(error.site(), LayoutErrorSiteOf::Standalone);
        assert_eq!(error.operation(), LayoutOperation::ValueResolution);
        crate::test_support::assert_error_kind(
            error.kind(),
            &LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::RequiredBasis),
        );
    }

    #[test]
    fn fri06_c13_t06_leaf_missing_basis_compute_size_and_layout_differ_f32() {
        assert_fri06_c13_t06_leaf_missing_basis_counterexample::<f32>();
    }

    #[test]
    fn fri06_c13_t06_leaf_missing_basis_compute_size_and_layout_differ_f64() {
        assert_fri06_c13_t06_leaf_missing_basis_counterexample::<f64>();
    }
}

#[cfg(test)]
mod leaf_body_size_authority_tests {
    use super::*;

    fn qualify<S: LayoutScalar>() {
        let px = S::from_f64;
        let thirty = NonNegativeFiniteOf::new(px(30.0)).unwrap();
        let request = SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(thirty),
                    SizingOperation::MinContent,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            crate::OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Unknown,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
            ),
        );
        let context = crate::ContainingLayoutContext::new(
            FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        );
        let authored = NodeInputOf::<S> {
            size: Size::new(
                crate::PreferredSizeOf::px(px(30.0)),
                crate::PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        };
        let automatic = NodeInputOf::<S>::default();
        let normal =
            AxisSizingRequestOf::new(AvailableSpaceOf::Definite(thirty), SizingOperation::Normal);
        let intrinsic = request.axes().width;
        // This opaque native leaf has an unbreakable 40x10 content rectangle.
        // Its content may overflow an authored 30px border box; it has no Text
        // source, invented Grid output, or dependency on the observed request.
        let content = Size::new(px(40.0), px(10.0));
        let measure = |input, style: &NodeInputOf<S>| {
            let mut observed = None;
            let output = compute_leaf(input, style, |packet| {
                observed = Some(packet);
                Ok::<_, ()>(packet.known_content_size().unwrap_or(content))
            })
            .unwrap();
            (
                output,
                observed.expect("unknown height requires actual leaf measurement"),
            )
        };

        let layout = ComputeInputOf::leaf_layout(Size::NONE, Size::NONE, context, request).unwrap();
        let (authored_output, authored_packet) = measure(layout, &authored);
        let (bounded_output, bounded_packet) = measure(layout, &automatic);
        assert_eq!(authored_packet.known_content_size().width, None);
        assert_eq!(bounded_packet.known_content_size().width, None);
        assert_eq!(
            authored_packet.content_sizing_request().axes().width,
            normal
        );
        assert_eq!(
            bounded_packet.content_sizing_request().axes().width,
            intrinsic
        );
        assert_ne!(authored_packet, bounded_packet);
        assert_eq!(authored_output.size.width, px(30.0));
        assert_eq!(bounded_output.size.width, px(40.0));
        assert_eq!(authored_output.content_size.width, px(40.0));
        assert_eq!(layout.sizing_request(), request);

        // ContentSize deliberately ignores authored preferred size; physical
        // availability stays 30 while the intrinsic operation remains MIN.
        let content_input =
            ComputeInputOf::leaf_content_size(Size::NONE, Size::NONE, context, request).unwrap();
        let (suppressed_output, suppressed_packet) = measure(content_input, &authored);
        let (_, automatic_content_packet) = measure(content_input, &automatic);
        assert_eq!(suppressed_packet, automatic_content_packet);
        assert_eq!(
            suppressed_packet.content_sizing_request().axes().width,
            intrinsic
        );
        assert_eq!(suppressed_output.size.width, px(40.0));

        // A real imposed known size survives ContentSize's authored suppression.
        let imposed = ComputeInputOf::leaf_content_size(
            Size::new(Some(px(30.0)), None),
            Size::NONE,
            context,
            request,
        )
        .unwrap();
        let (imposed_output, imposed_packet) = measure(imposed, &authored);
        assert_eq!(imposed_packet.known_content_size().width, Some(px(30.0)));
        assert_eq!(imposed_packet.content_sizing_request().axes().width, normal);
        assert_eq!(imposed_output.size.width, px(30.0));
        assert_eq!(imposed.sizing_request(), request);
    }

    #[test]
    fn authored_size_and_external_intrinsic_bound_remain_distinct_f32() {
        qualify::<f32>();
    }

    #[test]
    fn authored_size_and_external_intrinsic_bound_remain_distinct_f64() {
        qualify::<f64>();
    }
}

#[cfg(test)]
mod leaf_fallback_projection_tests {
    use super::*;

    fn qualify<S: LayoutScalar>(maximum: S) {
        let nn = |v| NonNegativeFiniteOf::new(S::from_f64(v)).unwrap();
        let unbounded = Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded);
        let fallback = crate::OrthogonalFallbackOf::new(
            Size::new(Some(nn(100.0)), None),
            crate::NearestScrollportOf::Absent,
            unbounded,
        );
        let request = SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(nn(20.0)),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            fallback,
        );
        let context = crate::ContainingLayoutContext::new(
            FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        );
        let input = ComputeInputOf::leaf_layout(Size::NONE, Size::NONE, context, request).unwrap();
        let mut observed = None;
        let output = compute_leaf(input, &NodeInputOf::<S>::default(), |packet| {
            observed = Some(packet);
            Ok::<_, ()>(Size::new(S::from_f64(10.0), S::from_f64(10.0)))
        })
        .unwrap();
        assert_eq!(output.size, Size::new(S::from_f64(10.0), S::from_f64(10.0)));
        let final_fallback = observed
            .unwrap()
            .content_sizing_request()
            .orthogonal_fallback();
        assert_eq!(final_fallback.width, Ok(nn(100.0)));
        assert_eq!(
            final_fallback.height,
            Err(
                crate::OrthogonalFallbackErrorOf::MissingInitialContainingBlock {
                    axis: PhysicalAxis::Vertical
                }
            )
        );
        assert_eq!(input.sizing_request().orthogonal_fallback(), fallback);
        assert_eq!(
            content_fallback(
                fallback,
                PhysicalAxis::Horizontal,
                S::from_f64(20.0),
                S::from_f64(90.0)
            ),
            Ok(NonNegativeFiniteOf::ZERO)
        );
        let huge = crate::OrthogonalFallbackOf::new(
            Size::new(Some(NonNegativeFiniteOf::new(maximum).unwrap()), None),
            crate::NearestScrollportOf::Absent,
            unbounded,
        );
        assert_eq!(
            content_fallback(huge, PhysicalAxis::Horizontal, -maximum, S::ZERO),
            Err(crate::OrthogonalFallbackErrorOf::InvalidContentSpace {
                axis: PhysicalAxis::Horizontal,
                value: S::INFINITY
            })
        );
    }

    #[test]
    fn projects_available_axis_without_rejecting_unused_missing_axis_f32() {
        qualify::<f32>(f32::MAX);
    }
    #[test]
    fn projects_available_axis_without_rejecting_unused_missing_axis_f64() {
        qualify::<f64>(f64::MAX);
    }
}

#[cfg(test)]
mod leaf_flow_context_tests {
    use super::*;

    // A native measured strip has FOUR breakable 10x10 tiles. This provider
    // owns only this declared profile: its ordinary indefinite request is
    // unbounded; its orthogonal Auto Normal request needs the supplied inline
    // fallback. This is not Text/Unicode or a general WM4 7.3.3 implementation.
    fn native_tiles<S: LayoutScalar>(
        packet: LeafMeasureInputOf<S>,
    ) -> Result<Size<S>, crate::OrthogonalFallbackErrorOf<S>> {
        let flow = packet.flow_axes();
        let axis = flow.inline_axis();
        let request = physical_axis(packet.content_sizing_request().axes(), axis);
        let known = physical_axis(packet.known_content_size(), axis);
        let extent = match request.operation() {
            SizingOperation::MinContent => Some(S::from_f64(10.0)),
            SizingOperation::MaxContent => None,
            SizingOperation::Normal => match request.space().definite_size() {
                Some(value) => Some(value.get()),
                None if axis != packet.containing_layout_context().flow_axes().inline_axis() => {
                    Some(
                        physical_axis(packet.content_sizing_request().orthogonal_fallback(), axis)?
                            .get(),
                    )
                }
                None => None,
            },
        };
        let columns = extent.map_or(4, |value| {
            (value / S::from_f64(10.0))
                .floor_to_usize_saturating()
                .clamp(1, 4)
        });
        let rows = 4_usize.div_ceil(columns);
        let inline = known.unwrap_or(S::from_usize(columns) * S::from_f64(10.0));
        Ok(flow.physical_size(crate::geometry::LogicalSizeOf::new(
            inline,
            S::from_usize(rows) * S::from_f64(10.0),
        )))
    }

    fn qualify<S: LayoutScalar>() {
        let nn = |value| NonNegativeFiniteOf::new(S::from_f64(value)).unwrap();
        let unbounded = Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded);
        let fallback = crate::OrthogonalFallbackOf::new(
            Size::splat(Some(nn(20.0))),
            crate::NearestScrollportOf::Absent,
            unbounded,
        );
        let missing_height = crate::OrthogonalFallbackOf::new(
            Size::new(Some(nn(20.0)), None),
            crate::NearestScrollportOf::Absent,
            unbounded,
        );
        let missing_width = crate::OrthogonalFallbackOf::new(
            Size::new(None, Some(nn(20.0))),
            crate::NearestScrollportOf::Absent,
            unbounded,
        );
        let containing = crate::ContainingLayoutContext::new(
            FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Rtl),
            crate::ParentFormattingContext::BlockFlow,
        );
        let request = |fallback| {
            SizingRequestOf::new(
                Size::splat(AxisSizingRequestOf::new(
                    AvailableSpaceOf::Indefinite,
                    SizingOperation::Normal,
                )),
                fallback,
            )
        };
        let parallel = NodeInputOf::<S> {
            display: crate::Display::Block,
            ..NodeInputOf::default()
        };
        let orthogonal = NodeInputOf::<S> {
            writing_mode: crate::WritingMode::VerticalRl,
            ..parallel.clone()
        };
        let run = |input, style: &NodeInputOf<S>| {
            let mut packet = None;
            let output = compute_leaf(input, style, |observed| {
                packet = Some(observed);
                native_tiles(observed)
            });
            (output, packet)
        };
        let ordinary_input =
            ComputeInputOf::leaf_layout(Size::NONE, Size::NONE, containing, request(fallback))
                .unwrap();
        let (ordinary, ordinary_packet) = run(ordinary_input, &parallel);
        assert_eq!(
            ordinary.unwrap().size,
            Size::new(S::from_f64(40.0), S::from_f64(10.0))
        );
        let ordinary_packet = ordinary_packet.unwrap();
        assert_eq!(
            ordinary_packet.flow_axes(),
            FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr)
        );
        assert_eq!(ordinary_packet.containing_layout_context(), containing);
        assert_eq!(
            ordinary_packet.content_sizing_request().axes(),
            ordinary_input.sizing_request().axes()
        );
        let (orthogonal_output, orthogonal_packet) = run(ordinary_input, &orthogonal);
        assert_eq!(
            orthogonal_output.unwrap().size,
            Size::new(S::from_f64(20.0), S::from_f64(20.0))
        );
        let orthogonal_packet = orthogonal_packet.unwrap();
        assert_eq!(
            orthogonal_packet.flow_axes(),
            FlowAxes::new(crate::WritingMode::VerticalRl, crate::Direction::Ltr)
        );
        assert_eq!(orthogonal_packet.containing_layout_context(), containing);
        assert_eq!(
            orthogonal_packet.content_sizing_request().axes(),
            ordinary_input.sizing_request().axes()
        );

        let partial_input =
            ComputeInputOf::leaf_layout(Size::NONE, Size::NONE, containing, request(missing_width))
                .unwrap();
        assert_eq!(
            run(partial_input, &orthogonal).0.unwrap().size,
            Size::new(S::from_f64(20.0), S::from_f64(20.0))
        );
        let missing_input = ComputeInputOf::leaf_layout(
            Size::NONE,
            Size::NONE,
            containing,
            request(missing_height),
        )
        .unwrap();
        assert_eq!(
            run(missing_input, &parallel).0.unwrap().size,
            Size::new(S::from_f64(40.0), S::from_f64(10.0))
        );
        let required = run(missing_input, &orthogonal).0.unwrap_err();
        assert_eq!(required.operation(), LayoutOperation::LeafMeasurement);
        assert!(matches!(
            required.kind(),
            LayoutErrorKindOf::Measurement(
                crate::OrthogonalFallbackErrorOf::MissingInitialContainingBlock {
                    axis: PhysicalAxis::Vertical
                }
            )
        ));
        for (operation, expected) in [
            (
                SizingOperation::MinContent,
                Size::new(S::from_f64(40.0), S::from_f64(10.0)),
            ),
            (
                SizingOperation::MaxContent,
                Size::new(S::from_f64(10.0), S::from_f64(40.0)),
            ),
        ] {
            let mut axes = request(missing_height).axes();
            axes.height = axes.height.with_operation(operation);
            let input = ComputeInputOf::leaf_layout(
                Size::NONE,
                Size::NONE,
                containing,
                request(missing_height).with_axes(axes),
            )
            .unwrap();
            assert_eq!(run(input, &orthogonal).0.unwrap().size, expected);
        }
        // A real authored inline height becomes Normal/Definite before the
        // provider; it does not require the missing external fallback.
        let authored = NodeInputOf::<S> {
            size: Size::new(
                crate::PreferredSizeOf::AUTO,
                crate::PreferredSizeOf::px(S::from_f64(30.0)),
            ),
            ..orthogonal
        };
        let mut axes = request(missing_height).axes();
        axes.height = axes.height.with_operation(SizingOperation::MinContent);
        let authored_request = request(missing_height).with_axes(axes);
        let input =
            ComputeInputOf::leaf_layout(Size::NONE, Size::NONE, containing, authored_request)
                .unwrap();
        let (sized, packet) = run(input, &authored);
        assert_eq!(
            sized.unwrap().size,
            Size::new(S::from_f64(20.0), S::from_f64(30.0))
        );
        assert_eq!(
            packet
                .unwrap()
                .content_sizing_request()
                .axes()
                .height
                .operation(),
            SizingOperation::Normal
        );
        // ContentSize suppression restores the MIN request, not a
        // fallback-derived known height or an unconditional ICB cap.
        let input =
            ComputeInputOf::leaf_content_size(Size::NONE, Size::NONE, containing, authored_request)
                .unwrap();
        assert_eq!(
            run(input, &authored).0.unwrap().size,
            Size::new(S::from_f64(40.0), S::from_f64(10.0))
        );
    }

    #[test]
    fn opaque_provider_uses_actual_flow_and_only_required_fallback_f32() {
        qualify::<f32>();
    }
    #[test]
    fn opaque_provider_uses_actual_flow_and_only_required_fallback_f64() {
        qualify::<f64>();
    }
}

#[cfg(test)]
#[path = "measurement/known_preferred_authority_tests.rs"]
mod known_preferred_authority_tests;
