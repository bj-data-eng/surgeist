//! Checks the Native-only outputs before projecting original Flex elements.

use super::{Error, TestNode, TestTree, flex_content_text::LoweredOccurrence, layout};

#[derive(Clone, Debug, Default)]
pub(super) struct ValidatedProjection {
    synthetic_nodes: Vec<usize>,
    collapsed_sources: Vec<usize>,
}

impl ValidatedProjection {
    pub(super) fn is_synthetic_node(&self, node: usize) -> bool {
        self.synthetic_nodes.contains(&node)
    }

    pub(super) fn is_collapsed_source(&self, node: usize) -> bool {
        self.collapsed_sources.contains(&node)
    }
}

fn invalid(detail: impl std::fmt::Display) -> Error {
    Error::new(format!("Flex content Text projection: {detail}"))
}

#[derive(Clone, Copy)]
struct ExpectedOutput {
    source_index: layout::SourceIndex,
    collapsed: bool,
    is_box: bool,
    kind: super::observation::Kind,
}

pub(super) fn validate_batch_targets(
    tree: &TestTree,
    batch: &layout::CompletedLayoutBatch<usize>,
) -> Result<(), Error> {
    let count = tree.nodes.len();
    let foreign_node = batch
        .invalidated_nodes()
        .iter()
        .copied()
        .chain(batch.cache_clear_entries().iter().map(|entry| entry.node()))
        .chain(batch.cache_store_entries().iter().map(|entry| entry.node()))
        .chain(batch.node_layouts().map(|(node, _)| node))
        .any(|node| node >= count);
    let foreign_fragment = batch.unrounded_inline_fragments().iter()
        .chain(batch.final_inline_fragments())
        .any(|entry| !matches!(entry.owner(), layout::InlineFragmentOwner::Node(node) if *node < count));
    if foreign_node || foreign_fragment {
        return Err(invalid("batch targets a foreign node or fragment owner"));
    }
    Ok(())
}

pub(super) fn validate(
    tree: &TestTree,
    batch: &layout::CompletedLayoutBatch<usize>,
) -> Result<ValidatedProjection, Error> {
    let mut projection = ValidatedProjection::default();
    if tree.flex_text.is_empty() {
        return Err(invalid("missing lowered source occurrences"));
    }
    let root = tree
        .nodes
        .first()
        .ok_or_else(|| invalid("missing original Flex root"))?;
    if root.synthetic
        || root.parent.is_some()
        || root.layout_input.as_box().map(|input| input.display) != Some(layout::Display::Flex)
        || root.children.len() != tree.flex_text.len()
        || tree.paragraphs.len() != tree.flex_text.len()
        || batch.native_inline_box_families().next().is_some()
        || batch.native_inline_box_pieces().next().is_some()
    {
        return Err(invalid(
            "unsupported root, paragraph or native inline-box census",
        ));
    }

    let mut expected = vec![None; tree.nodes.len()];
    expected[0] = Some(ExpectedOutput {
        source_index: layout::SourceIndex::ZERO,
        collapsed: false,
        is_box: true,
        kind: super::observation::Kind::Box,
    });
    let mut visible_text = Vec::new();
    let mut shared_baseline = None;
    for (index, occurrence) in tree.flex_text.iter().enumerate() {
        let (source, wrapper, text) = (occurrence.source, occurrence.wrapper, occurrence.text);
        let (collapsed, origin) = validate_occurrence(tree, index, occurrence)?;
        for (id, source_index, is_box) in
            [(source, index, true), (wrapper, 0, true), (text, 0, false)]
        {
            if expected[id]
                .replace(ExpectedOutput {
                    source_index: layout::SourceIndex::new(source_index),
                    collapsed,
                    is_box,
                    kind: if collapsed {
                        super::observation::Kind::NonBox(layout::NonBoxKind::Hidden)
                    } else if is_box {
                        super::observation::Kind::Box
                    } else {
                        super::observation::Kind::NonBox(layout::NonBoxKind::InlineText)
                    },
                })
                .is_some()
            {
                return Err(invalid(format!(
                    "source 0.{index} shares a Native participant"
                )));
            }
        }
        projection.synthetic_nodes.extend([wrapper, text]);
        if collapsed {
            projection.collapsed_sources.push(source);
            continue;
        }

        visible_text.push(text);
        let anchor = validate_visible_text(tree, batch, index, occurrence, origin)?;
        if !anchor.is_finite()
            || shared_baseline
                .replace(anchor)
                .is_some_and(|previous| previous != anchor)
        {
            return Err(invalid(format!(
                "visible source 0.{index} lost the actual shared content baseline"
            )));
        }
    }

    validate_outputs(&tree.nodes, &expected, batch, false)?;
    validate_outputs(&tree.nodes, &expected, batch, true)?;
    if batch.placed_text().len() != visible_text.len()
        || batch.unrounded_inline_fragments() != batch.final_inline_fragments()
        || batch.unrounded_inline_fragments().iter().any(|entry| {
            !matches!(entry.owner(), layout::InlineFragmentOwner::Node(node) if visible_text.contains(node))
        })
    {
        return Err(invalid("unexpected placed Text or fragment outside a visible Text leaf"));
    }
    Ok(projection)
}

fn validate_occurrence<'a>(
    tree: &'a TestTree,
    index: usize,
    occurrence: &LoweredOccurrence,
) -> Result<(bool, &'a surgeist_text::PreparedOrigin), Error> {
    let (source, wrapper, text) = (occurrence.source, occurrence.wrapper, occurrence.text);
    let root = &tree.nodes[0];
    let [source_node, wrapper_node, text_node] =
        [source, wrapper, text].map(|id| tree.nodes.get(id));
    let (Some(source_node), Some(wrapper_node), Some(text_node)) =
        (source_node, wrapper_node, text_node)
    else {
        return Err(invalid(format!(
            "source 0.{index} has a missing participant"
        )));
    };
    let Some(source_input) = source_node.layout_input.as_box() else {
        return Err(invalid(format!("source 0.{index} is missing its Flex box")));
    };
    let layout::LayoutInput::InlineText(participation) = &text_node.layout_input else {
        return Err(invalid(format!(
            "source 0.{index} is missing its prepared Text leaf"
        )));
    };
    let origin = participation.origin();
    if root.children[index] != source
        || source_node.synthetic
        || source_node.parent != Some(0)
        || source_node.children != [wrapper]
        || source_input.display != layout::Display::Flex
        || source_input.position == layout::Position::Absolute
        || source_node.text.is_some()
        || !wrapper_node.synthetic
        || wrapper_node.parent != Some(source)
        || wrapper_node.children != [text]
        || wrapper_node.text.is_some()
        || wrapper_node.layout_input.as_box().is_none_or(|input| {
            input.display != layout::Display::Block
                || input.position == layout::Position::Absolute
                || input.writing_mode != source_input.writing_mode
                || input.direction != source_input.direction
        })
        || !text_node.synthetic
        || text_node.parent != Some(wrapper)
        || !text_node.children.is_empty()
        || text_node.text.is_some()
    {
        return Err(invalid(format!(
            "source 0.{index} has unexpected Flex/Text topology"
        )));
    }

    let mut bindings = tree
        .paragraphs
        .iter()
        .filter(|binding| binding.container == wrapper);
    let binding = bindings.next().ok_or_else(|| {
        invalid(format!(
            "source 0.{index} is missing its authenticated paragraph"
        ))
    })?;
    if bindings.next().is_some()
        || binding.nodes != [text]
        || binding.prepared != origin.prepared_text()
        || !matches!(binding.origins.as_slice(), [row] if row.node == text && &row.origin == origin)
        || !binding.atomics.is_empty()
        || !binding.controls.is_empty()
        || binding.inline_box_fragments != layout::InlineBoxFragmentationDemand::NotRequested
    {
        return Err(invalid(format!(
            "source 0.{index} lost its prepared paragraph custody"
        )));
    }

    let collapsed = source_input.flex_item_collapse == layout::FlexItemCollapse::Collapsed;
    Ok((collapsed, origin))
}

fn validate_visible_text(
    tree: &TestTree,
    batch: &layout::CompletedLayoutBatch<usize>,
    index: usize,
    occurrence: &LoweredOccurrence,
    origin: &surgeist_text::PreparedOrigin,
) -> Result<f64, Error> {
    let (source, wrapper, text) = (occurrence.source, occurrence.wrapper, occurrence.text);
    let mut placements = batch
        .placed_text()
        .iter()
        .filter(|row| row.container() == wrapper);
    let placement = placements
        .next()
        .ok_or_else(|| invalid(format!("visible source 0.{index} is missing placed Text")))?;
    if placements.next().is_some()
        || placement.nodes() != [text]
        || placement.inline_box_fragments().is_some()
    {
        return Err(invalid(format!(
            "visible source 0.{index} has unexpected placed Text custody"
        )));
    }
    let placed = placement.layout();
    let lines = placed.lines();
    let [line] = lines.as_slice() else {
        return Err(invalid(format!(
            "visible source 0.{index} requires one placed line"
        )));
    };
    if placed.nonformatted_lines().next().is_some()
        || line.range() != origin.view().aggregate_range()
    {
        return Err(invalid(format!(
            "visible source 0.{index} lost complete label placement"
        )));
    }
    let observed = placed
        .source_range_fragments(origin, origin.view().local_range())
        .map_err(|error| {
            invalid(format!(
                "visible source 0.{index} lost placed origin: {error:?}"
            ))
        })?;
    let mut units = observed.units();
    let first = units.next().ok_or_else(|| {
        invalid(format!(
            "visible source 0.{index} has no placed source units"
        ))
    })?;
    let placed_line = first.finalized_line();
    if first.line_index() != 0
        || units.any(|unit| unit.line_index() != 0 || unit.finalized_line() != placed_line)
    {
        return Err(invalid(format!(
            "visible source 0.{index} has ambiguous placed source line"
        )));
    }

    // Placed Text uses the canonical final visual frame. Fragment anchors
    // instead belong to their paragraph container's local frame.
    let wrapper_y = [0, source, wrapper]
        .into_iter()
        .map(|node| f64::from(tree.nodes[node].final_layout.location.y))
        .sum::<f64>();
    let fragments = validate_fragments(
        index,
        &tree.nodes[text],
        &placed_line,
        origin,
        line.baseline() - wrapper_y,
    )?;
    validate_text_output(index, &tree.nodes[text], &fragments)?;
    Ok(line.baseline())
}

fn validate_text_output(
    index: usize,
    text_node: &TestNode,
    fragments: &[layout::ScrollRect],
) -> Result<(), Error> {
    let text_output = text_node.unrounded;
    let start = fragments
        .iter()
        .map(|rect| rect.origin())
        .reduce(|a, b| layout::Point::new(a.x.min(b.x), a.y.min(b.y)))
        .expect("validated line has actual fragment bounds");
    let end = fragments
        .iter()
        .map(|rect| {
            let origin = rect.origin();
            let size = rect.size();
            layout::Point::new(origin.x + size.width, origin.y + size.height)
        })
        .reduce(|a, b| layout::Point::new(a.x.max(b.x), a.y.max(b.y)))
        .expect("validated line has actual fragment bounds");
    let size = layout::Size::new(end.x - start.x, end.y - start.y);
    if text_output.kind != super::observation::Kind::NonBox(layout::NonBoxKind::InlineText)
        || !text_output.matches_numeric_fields(
            &(layout::NodeOutput {
                location: start,
                size,
                content_size: size,
                ..layout::NodeOutput::new()
            }),
        )
        || text_node.final_layout != text_output
    {
        return Err(invalid(format!(
            "visible source 0.{index} Text packet disagrees with its fragment bounds"
        )));
    }

    Ok(())
}

fn validate_fragments(
    index: usize,
    text_node: &TestNode,
    line: &surgeist_text::FinalizedLine,
    origin: &surgeist_text::PreparedOrigin,
    baseline: f64,
) -> Result<Vec<layout::ScrollRect>, Error> {
    let actual = text_node
        .unrounded_inline_fragments
        .as_deref()
        .unwrap_or_default();
    let mut issued = line.fragments().collect::<Vec<_>>();
    if actual.is_empty()
        || actual.len() != issued.len()
        || text_node.final_inline_fragments.as_deref() != Some(actual)
    {
        return Err(invalid(format!(
            "visible source 0.{index} has incomplete fragment readback"
        )));
    }
    actual.iter().map(|fragment| {
        let handle = fragment.finalized_fragment();
        let position = issued.iter().position(|candidate| candidate == handle);
        if fragment.line_index() != 0
            || !matches!(handle.view().owner(), surgeist_text::FinalizedFragmentOwner::Origin(owner) if &owner == origin)
            || f64::from(fragment.baseline().y) != baseline
            || position.is_none()
        {
            return Err(invalid(format!("visible source 0.{index} lost actual fragment custody")));
        }
        issued.swap_remove(position.expect("verified fragment belongs to the placed line"));
        fragment.rect().ok_or_else(|| invalid(format!("visible source 0.{index} is missing actual fragment bounds")))
    }).collect()
}

fn validate_outputs(
    nodes: &[TestNode],
    expected: &[Option<ExpectedOutput>],
    batch: &layout::CompletedLayoutBatch<usize>,
    final_phase: bool,
) -> Result<(), Error> {
    let phase = if final_phase { "final" } else { "unrounded" };
    let mut seen = vec![false; nodes.len()];
    for (id, view) in batch.node_layouts() {
        if matches!(view, layout::NodeLayoutRefOf::FragmentedBox(_)) {
            return Err(invalid(format!(
                "unexpected {phase} box fragmentation for node {id}"
            )));
        }
        let Some(Some(role)) = expected.get(id) else {
            return Err(invalid(format!("unexpected {phase} packet for node {id}")));
        };
        let node = &nodes[id];
        let output = super::observation::observe((batch, final_phase), id, role.kind);
        let applied = if final_phase {
            node.final_layout_present.then_some(node.final_layout)
        } else {
            node.unrounded_present.then_some(node.unrounded)
        };
        if std::mem::replace(&mut seen[id], true)
            || Some(output) != applied
            || output.source_index != role.source_index
            || output.scroll_geometry.is_some() != (role.is_box && !role.collapsed)
            || role.collapsed
                && !output.matches_numeric_fields(&layout::NodeOutput::with_source_index(
                    role.source_index,
                ))
        {
            return Err(invalid(format!(
                "invalid or duplicate {phase} packet for node {id}"
            )));
        }
    }
    if seen.iter().any(|present| !present) {
        return Err(invalid(format!("incomplete {phase} node output census")));
    }
    Ok(())
}
