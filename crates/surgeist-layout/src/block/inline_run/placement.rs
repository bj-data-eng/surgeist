use super::*;

type TextFragmentGroup<Node, S> = (
    Node,
    Vec<InlineFragmentOutputOf<S>>,
    Option<Point<S>>,
    Option<Point<S>>,
    Option<Point<S>>,
    Option<Point<S>>,
);

pub(super) struct CapturedRunPlacement<'a, Node, S: LayoutScalar> {
    pub(super) container: Node,
    pub(super) run: &'a [Node],
    pub(super) source_index_start: usize,
    pub(super) input: ComputeInputOf<S>,
    pub(super) available_inline_extent: AxisSizingRequestOf<S>,
    pub(super) set_layout: bool,
    pub(super) binding: &'a crate::text_layout::TextParagraphBinding<Node>,
    pub(super) output: crate::text_layout::run::Output<Node, S>,
    pub(super) box_frontier: &'a mut Option<crate::text_layout::boxes::BoxFrontier>,
}

pub(super) fn place_captured_run<'input, Tree, S, M>(
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
