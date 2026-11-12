use super::super::relative::{self, InsetReference};
use super::super::scroll::retained_child_scroll_geometry;
use super::prepare::InlineRunControl;
use super::*;
use crate::inline::InlineSourcePositionOf;

type TextFragmentGroup<Node, S> = (
    Node,
    Vec<InlineFragmentOutputOf<S>>,
    Option<Point<S>>,
    Option<Point<S>>,
    Option<Point<S>>,
    Option<Point<S>>,
);

struct CapturedRunPlacement<'a, Node, S: LayoutScalar> {
    container: Node,
    run: &'a [Node],
    source_index_start: usize,
    input: ComputeInputOf<S>,
    available_inline_extent: AxisSizingRequestOf<S>,
    set_layout: bool,
    binding: &'a crate::text_layout::TextParagraphBinding<Node>,
    output: crate::text_layout::run::Output<Node, S>,
    box_frontier: &'a mut Option<crate::text_layout::boxes::BoxFrontier>,
}

fn place_captured_run<'input, Tree, S, M>(
    tree: &mut Tree,
    placement: CapturedRunPlacement<'_, Tree::Node, S>,
    static_positions: &mut Vec<(Tree::Node, StaticPositionOf<S>)>,
    contributions: &mut BlockScrollContributions<S>,
) -> LayoutResultOf<Tree::Node, (), S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let CapturedRunPlacement {
        container,
        run,
        source_index_start,
        input,
        available_inline_extent,
        set_layout,
        binding,
        output,
        box_frontier,
    } = placement;
    if let Some(state) = tree.text_computation() {
        state.record_boxes(crate::text_layout::boxes::BoxRunUse {
            container,
            prepared: output.paragraph.prepared.clone(),
            nodes: run.to_vec(),
            input,
            available: crate::layout_math::text_axis_request(available_inline_extent),
            trials: output.box_uses.clone(),
        });
    }
    if let Some(Some(assignment)) = output.paragraph.boxes.last() {
        *box_frontier = Some(assignment.outgoing.clone());
    }
    if set_layout {
        // Only the actual line allocation supplies terminal padding edges.
        // Placed metrics and source fragments remain independent overflow.
        for bounds in &output.line_bounds {
            contributions
                .include_line(bounds.allocation)
                .map_err(|error| layout_own_geometry_error(container, input.run_mode(), error))?;
            contributions.geometry.include_direct_line(bounds.metrics);
        }
        for rect in output
            .fragments
            .iter()
            .filter_map(|source| source.fragment.rect())
        {
            contributions.geometry.include_direct_line(rect);
        }
    }
    let mut groups: Vec<TextFragmentGroup<Tree::Node, S>> = Vec::new();
    let mut scope_fragments = Vec::new();
    for crate::text_layout::run::SourceFragment {
        owner,
        normal_rect,
        fragment,
    } in output.fragments
    {
        let crate::InlineFragmentOwner::Node(node) = owner else {
            scope_fragments.push(crate::InlineFragmentOutputEntryOf::new(owner, fragment));
            continue;
        };
        let index = groups
            .iter()
            .position(|(subject, _, _, _, _, _)| *subject == node);
        let index = index.unwrap_or_else(|| {
            groups.push((node, Vec::new(), None, None, None, None));
            groups.len() - 1
        });
        let (_, fragments, minimum, maximum, normal_minimum, normal_maximum) = &mut groups[index];
        if let Some(rect) = normal_rect {
            let location = rect.origin();
            let end = Point::new(
                location.x + rect.size().width,
                location.y + rect.size().height,
            );
            *normal_minimum = Some(normal_minimum.map_or(location, |p| {
                Point::new(p.x.min(location.x), p.y.min(location.y))
            }));
            *normal_maximum =
                Some(normal_maximum.map_or(end, |p| Point::new(p.x.max(end.x), p.y.max(end.y))));
        }
        if let Some(rect) = fragment.rect() {
            let location = rect.origin();
            let end = Point::new(
                location.x + rect.size().width,
                location.y + rect.size().height,
            );
            *minimum = Some(minimum.map_or(location, |point| {
                Point::new(point.x.min(location.x), point.y.min(location.y))
            }));
            *maximum = Some(maximum.map_or(end, |point| {
                Point::new(point.x.max(end.x), point.y.max(end.y))
            }));
        }
        fragments.push(fragment);
    }
    for (node, actual) in output.static_positions {
        if let Some((_, position)) = static_positions
            .iter_mut()
            .find(|(subject, _)| *subject == node)
        {
            *position = actual;
        } else {
            static_positions.push((node, actual));
        }
    }
    if set_layout {
        for (child, normal_location, location) in output.nonformatted_controls {
            let source_index = source_index_start
                + run
                    .iter()
                    .position(|node| *node == child)
                    .expect("nonformatted descendant is an actual bound control");
            let (border, padding) = match tree.layout_input(child) {
                crate::LayoutInputViewOf::InlineBoundary(boundary) => {
                    crate::inline::InlineEndpointOf::from_input(boundary).physical_components()
                }
                _ => (Edges::ZERO, Edges::ZERO),
            };
            tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                child,
                crate::output::NormalGeometryOf::metadata(normal_location, crate::Size::ZERO),
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location,
                    border,
                    padding,
                    ..NodeOutputOf::new()
                },
            ));
        }
    }
    if set_layout {
        for row in binding.origins.iter().filter(|row| run.contains(&row.node)) {
            if !groups.iter().any(|(node, _, _, _, _, _)| *node == row.node) {
                groups.push((row.node, Vec::new(), None, None, None, None));
            }
        }
        for (node, fragments, minimum, maximum, normal_minimum, normal_maximum) in groups {
            let (normal_location, normal_size) = match (normal_minimum, normal_maximum) {
                (Some(start), Some(end)) => (start, Size::new(end.x - start.x, end.y - start.y)),
                _ => (Point::ZERO, Size::ZERO),
            };
            let (location, size) = match (minimum, maximum) {
                (Some(minimum), Some(maximum)) => (
                    minimum,
                    Size::new(maximum.x - minimum.x, maximum.y - minimum.y),
                ),
                _ => (Point::ZERO, Size::ZERO),
            };
            let source_index = source_index_start
                + run
                    .iter()
                    .position(|child| *child == node)
                    .expect("capture admitted this source node");
            tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                node,
                crate::output::NormalGeometryOf::metadata(normal_location, normal_size),
                NodeOutputOf {
                    source_index: crate::SourceIndex::new(source_index),
                    location,
                    size,
                    content_size: size,
                    ..NodeOutputOf::new()
                },
            ));
            tree.set_unrounded_inline_fragment_state(node, Some(fragments));
        }
        let state = tree
            .text_computation()
            .expect("capture branch retains actual Text state");
        state.record_scope_fragments(
            output.paragraph.container,
            output.paragraph.prepared.clone(),
            output.paragraph.nodes.clone(),
            scope_fragments,
        );
        state.record(output.paragraph);
    }
    Ok(())
}

pub(super) struct InlineSourcePlacement<'a, 'input, Node, S: LayoutScalar> {
    pub(super) container: Node,
    pub(super) run: &'a [Node],
    pub(super) source_index_start: usize,
    pub(super) input: ComputeInputOf<S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) available_inline_extent: AxisSizingRequestOf<S>,
    pub(super) containing_size: Size<S>,
    pub(super) node_inner_size: Size<Option<S>>,
    pub(super) cursor_block: S,
    pub(super) set_layout: bool,
    pub(super) text_binding: Option<&'input crate::text_layout::TextParagraphBinding<Node>>,
    pub(super) text_output: Option<crate::text_layout::run::Output<Node, S>>,
    pub(super) atomic_children: Vec<RetainedAtomicChild<'input, Node, S>>,
    pub(super) control_children: Vec<InlineRunControl<Node, S>>,
    pub(super) report: &'a crate::inline::MixedInlineRunReportOf<S>,
    pub(super) box_frontier: &'a mut Option<crate::text_layout::boxes::BoxFrontier>,
}

pub(super) fn place_inline_sources<'input, Tree, S, M>(
    tree: &mut Tree,
    placement: InlineSourcePlacement<'_, 'input, Tree::Node, S>,
    static_positions: &mut Vec<(Tree::Node, StaticPositionOf<S>)>,
    contributions: &mut BlockScrollContributions<S>,
) -> LayoutResultOf<Tree::Node, (), S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let InlineSourcePlacement {
        container,
        run,
        source_index_start,
        input,
        constants,
        available_inline_extent,
        containing_size,
        node_inner_size,
        cursor_block,
        set_layout,
        text_binding,
        text_output,
        atomic_children,
        control_children,
        report,
        box_frontier,
    } = placement;
    let logical_content_box_inset = constants.logical_content_box_inset();
    let project_point = |inline: S, block: S, size: LogicalSizeOf<S>| {
        constants.flow_axes.physical_point(
            LogicalPointOf::new(
                logical_content_box_inset.inline_start + inline,
                cursor_block + block,
            ),
            size,
            containing_size,
        )
    };
    let project_source =
        |position: InlineSourcePositionOf<S>, size: LogicalSizeOf<S>| match position {
            InlineSourcePositionOf::Logical(point) => {
                project_point(point.inline, point.block, size)
            }
            InlineSourcePositionOf::Physical { visual, .. } => visual,
        };

    let project_normal_source =
        |position: InlineSourcePositionOf<S>, size: LogicalSizeOf<S>| match position {
            InlineSourcePositionOf::Logical(point) => {
                project_point(point.inline, point.block, size)
            }
            InlineSourcePositionOf::Physical { normal, .. } => normal,
        };

    // Numeric geometry has no grid conversion. Validate every newly consumed
    // numeric displacement before publishing output, using retained input loans.
    // Captured Text owns its separate checked, once-applied grid conversion.
    let atomic_sources = report
        .atomics
        .iter()
        .map(|source| (source.item.source_index, *source))
        .collect::<BTreeMap<_, _>>();
    let control_sources = report
        .controls
        .iter()
        .map(|source| (source.source_index, *source))
        .collect::<BTreeMap<_, _>>();
    let mut shifted_atomic_locations = BTreeMap::new();
    let mut shifted_control_locations = BTreeMap::new();
    if text_binding.is_none() {
        for child in &atomic_children {
            let source = atomic_sources[&child.source_index];
            if !matches!(source.item.alignment, VerticalAlignOf::Shift(offset) if offset.get() != S::ZERO)
            {
                continue;
            }
            let InlineParticipantKindOf::Box(projection) = child.projection.kind() else {
                unreachable!("retained atomic projection is a box");
            };
            let inset = atomic_inset_offset::<Tree, S, M>(
                tree,
                child.node,
                &projection,
                constants.flow_axes,
                node_inner_size,
                text_binding.is_some(),
            )?;
            let logical_size = constants.flow_axes.logical_size(source.item.size);
            let InlineSourcePositionOf::Logical(position) = source.position else {
                unreachable!("the numeric fitter emits logical positions");
            };
            let location = checked_shift_projection(
                source.item.alignment,
                constants.flow_axes,
                position.block,
                |block| {
                    let point = project_point(position.inline, block, logical_size);
                    Point::new(point.x + inset.x, point.y + inset.y)
                },
            )
            .map_err(|error| error.layout::<Tree::Node, S, M>(child.node))?;
            shifted_atomic_locations.insert(child.source_index, location);
        }
        for (child, source_index, alignment) in &control_children {
            let Some(alignment) = alignment else {
                continue;
            };
            if !matches!(alignment, VerticalAlignOf::Shift(offset) if offset.get() != S::ZERO) {
                continue;
            }
            let Some(source) = control_sources.get(source_index) else {
                continue;
            };
            let size = LogicalSizeOf::new(source.inline_extent, S::ZERO);
            let InlineSourcePositionOf::Logical(position) = source.position else {
                unreachable!("the numeric fitter emits logical positions");
            };
            let location = checked_shift_projection(
                *alignment,
                constants.flow_axes,
                position.block,
                |block| project_point(position.inline, block, size),
            )
            .map_err(|error| error.layout::<Tree::Node, S, M>(*child))?;
            shifted_control_locations.insert(*source_index, location);
        }
    }

    if set_layout && text_output.is_none() {
        for band in &report.line_bands {
            let size = LogicalSizeOf::new(
                band.inline_end - band.inline_start,
                band.block_end - band.block_start,
            );
            let rect = crate::ScrollRectOf::try_new(
                project_point(band.inline_start, band.block_start, size),
                constants.flow_axes.physical_size(size),
            )
            .map_err(|error| layout_own_geometry_error(container, input.run_mode(), error))?;
            contributions
                .include_line(rect)
                .map_err(|error| layout_own_geometry_error(container, input.run_mode(), error))?;
        }
    }
    if let Some(output) = text_output {
        place_captured_run(
            tree,
            CapturedRunPlacement {
                container,
                run,
                source_index_start,
                input,
                available_inline_extent,
                set_layout,
                binding: text_binding.expect("a captured result retains its binding"),
                output,
                box_frontier,
            },
            static_positions,
            contributions,
        )?;
    }

    for RetainedAtomicChild {
        node: child,
        source_index,
        projection: child_style,
        output,
        ..
    } in atomic_children
    {
        let source = atomic_sources[&source_index];
        let logical_size = constants.flow_axes.logical_size(source.item.size);
        let InlineParticipantKindOf::Box(child_projection) = child_style.kind() else {
            unreachable!("atomic child retains its settled box projection");
        };
        let normal_location = project_normal_source(source.position, logical_size);
        let location = if let Some(location) = shifted_atomic_locations.get(&source_index) {
            *location
        } else {
            let point = project_source(source.position, logical_size);
            let inset = atomic_inset_offset::<Tree, S, M>(
                tree,
                child,
                &child_projection,
                constants.flow_axes,
                node_inner_size,
                text_binding.is_some(),
            )?;
            Point::new(point.x + inset.x, point.y + inset.y)
        };
        if set_layout {
            let scroll_geometry = with_block_scroll_projections::<Tree, M, _>(
                tree,
                child,
                |box_projection, target_projection| {
                    retained_child_scroll_geometry(
                        box_projection,
                        target_projection,
                        source.item.size,
                        source.item.content_size,
                        source.item.padding,
                        source.item.border,
                        output.scroll_geometry,
                    )
                },
            )
            .map_err(|error| layout_child_geometry_error(container, child, error))?;
            if text_binding.is_some()
                || !relative::needs_basis(*child_projection.common.inset, constants.flow_axes)
            {
                contributions
                    .include_in_flow_geometry(location, source.item.margin, scroll_geometry)
                    .map_err(|error| layout_child_geometry_error(container, child, error))?;
            }
            let named_baselines = match source.canonical_geometry {
                Some(canonical) => crate::text_layout::run::pending_visual_named_baselines(
                    output.named_baselines,
                    canonical.visual_readback_offset,
                )
                .map_err(|error| text_run_error(container, error))?,
                None => output.named_baselines,
            };
            let visual = NodeOutputOf::<S> {
                source_index: crate::SourceIndex::new(source_index),
                location,
                size: source.item.size,
                named_baselines,
                content_size: source.item.content_size,
                border: source.item.border,
                padding: source.item.padding,
                margin: source.item.margin,
                ..NodeOutputOf::new()
            }
            .with_scroll_geometry(Some(scroll_geometry));
            let entry = match source.canonical_geometry {
                Some(canonical) => crate::LayoutOutputEntryOf::canonical_atomic(
                    child,
                    canonical.measured_normal,
                    canonical.normal,
                    canonical.normal_origin,
                    canonical.visual_origin,
                    visual,
                ),
                None => crate::LayoutOutputEntryOf::ordinary_unrounded(
                    child,
                    crate::output::NormalGeometryOf::box_geometry(
                        normal_location,
                        source.item.size,
                        scroll_geometry.content_box(),
                    ),
                    visual,
                ),
            };
            tree.set_unrounded(entry);
        }
    }

    if set_layout {
        let nonformatted_controls = report
            .nonformatted_controls
            .iter()
            .map(|source| (source.control.source_index(), source))
            .collect::<BTreeMap<_, _>>();
        for (child, source_index, _) in control_children {
            let (position, inline_extent, endpoint, margin_flow, formatted) =
                if let Some(source) = control_sources.get(&source_index) {
                    (
                        source.position,
                        source.inline_extent,
                        source.endpoint,
                        source.margin_flow,
                        true,
                    )
                } else if let Some(source) = nonformatted_controls.get(&source_index) {
                    (
                        InlineSourcePositionOf::Logical(LogicalPointOf::new(
                            source.inline_start,
                            source.block_start,
                        )),
                        source.control.edge().border_padding_advance(),
                        Some(crate::inline::InlineEndpointOf::from_control(
                            source.control,
                        )),
                        None,
                        false,
                    )
                } else {
                    continue;
                };
            let logical_size = LogicalSizeOf::new(inline_extent, S::ZERO);
            let size = constants.flow_axes.physical_size(logical_size);
            let (border, padding) = endpoint.map_or((Edges::ZERO, Edges::ZERO), |endpoint| {
                endpoint.physical_components()
            });
            let normal_location = project_normal_source(position, logical_size);
            let location = shifted_control_locations
                .get(&source_index)
                .copied()
                .unwrap_or_else(|| project_source(position, logical_size));
            if let Some((before_flow, after_flow)) = margin_flow {
                let InlineSourcePositionOf::Logical(point) = position else {
                    unreachable!("numeric margin flow has the same logical phase as its strip");
                };
                let endpoint = endpoint.expect("numeric margin belongs to an endpoint");
                let flow_point = |inline| {
                    project_source(
                        InlineSourcePositionOf::Logical(LogicalPointOf::new(inline, point.block)),
                        LogicalSizeOf::new(S::ZERO, S::ZERO),
                    )
                };
                let before_flow = flow_point(before_flow);
                let after_flow = flow_point(after_flow);
                let positive =
                    FlowAxes::new(constants.flow_axes.writing_mode(), crate::Direction::Ltr)
                        .inline_start();
                let side = match endpoint.kind {
                    crate::InlineBoundaryKind::Start => endpoint.axes.inline_start(),
                    crate::InlineBoundaryKind::End => endpoint.axes.inline_end(),
                };
                let (before, after) = match (positive, side == positive) {
                    (crate::PhysicalSide::Left, true) => (before_flow.x, location.x),
                    (crate::PhysicalSide::Left, false) => (location.x + size.width, after_flow.x),
                    (crate::PhysicalSide::Top, true) => (before_flow.y, location.y),
                    (crate::PhysicalSide::Top, false) => (location.y + size.height, after_flow.y),
                    (crate::PhysicalSide::Bottom, true) => {
                        (before_flow.y, location.y + size.height)
                    }
                    (crate::PhysicalSide::Bottom, false) => (location.y, after_flow.y),
                    _ => unreachable!("the forward native inline basis is left, top or bottom"),
                };
                if !before.is_finite() || !after.is_finite() {
                    return Err(LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(child),
                        LayoutOperation::ChildLayout,
                        LayoutErrorKindOf::TextNumeric {
                            value: if !after.is_finite() {
                                after.to_f64()
                            } else {
                                before.to_f64()
                            },
                            reason: crate::LayoutTextNumericError::NonFinite,
                        },
                    ));
                }
                if before == after {
                    return Err(LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(child),
                        LayoutOperation::ChildLayout,
                        LayoutErrorKindOf::TextNumeric {
                            value: endpoint.edge.margin().to_f64(),
                            reason: crate::LayoutTextNumericError::NonZeroLost,
                        },
                    ));
                }
            }
            if formatted {
                contributions
                    .include_inline_strip(
                        location,
                        size,
                        endpoint.map_or(
                            Edges::ZERO,
                            crate::inline::InlineEndpointOf::physical_margin,
                        ),
                    )
                    .map_err(|error| layout_child_geometry_error(container, child, error))?;
            }
            tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                child,
                crate::output::NormalGeometryOf::metadata(normal_location, size),
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location,
                    size,
                    border,
                    padding,
                    margin: endpoint.map_or(
                        Edges::ZERO,
                        crate::inline::InlineEndpointOf::physical_margin,
                    ),
                    ..NodeOutputOf::new()
                },
            ));
        }
    }

    Ok(())
}

fn atomic_inset_offset<'input, Tree, S, M>(
    tree: &Tree,
    child: Tree::Node,
    projection: &BlockChildProjection<'_, S>,
    axes: FlowAxes,
    inner_size: Size<Option<S>>,
    captured: bool,
) -> LayoutResultOf<Tree::Node, Point<S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    if captured || relative::needs_basis(*projection.common.inset, axes) {
        // Captured offsets are sealed by the real converter. Native percentage
        // offsets are deferred until this parent's performed normal size exists.
        return Ok(Point::ZERO);
    }
    relative::resolve(
        tree,
        child,
        *projection.common.inset,
        InsetReference::Measurement(inner_size),
        axes,
    )
}

/// Compare the actual post-match displacement inside the same retained frame.
/// Reconstructing the matched point does not refit or renormalize the line.
pub(super) fn checked_shift_projection<S: LayoutScalar>(
    alignment: VerticalAlignOf<S>,
    axes: FlowAxes,
    block: S,
    project: impl Fn(S) -> Point<S>,
) -> Result<Point<S>, crate::group_alignment::NumericError> {
    let actual = project(block);
    if let VerticalAlignOf::Shift(offset) = alignment
        && offset.get() != S::ZERO
    {
        let inverse = if axes.line_over() == axes.block_start() {
            offset.get()
        } else {
            -offset.get()
        };
        let matched = project(crate::baseline_offset::displaced(block, inverse)?);
        let before = axes.block_axis_coordinate(matched);
        let after = axes.block_axis_coordinate(actual);
        let delta = match axes.line_over() {
            PhysicalSide::Top | PhysicalSide::Left => -offset.get(),
            PhysicalSide::Right | PhysicalSide::Bottom => offset.get(),
        };
        crate::baseline_offset::displaced(before, delta)?;
        if !after.is_finite() {
            return Err(crate::group_alignment::NumericError::new(
                after.to_f64(),
                crate::LayoutTextNumericError::NonFinite,
            ));
        }
        if after == before {
            return Err(crate::group_alignment::NumericError::new(
                offset.get().to_f64(),
                crate::LayoutTextNumericError::NonZeroLost,
            ));
        }
    }
    Ok(actual)
}
