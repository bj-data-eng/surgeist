//! One side-aware permutation of actual selected, indivisible payload packets.
use super::*;
use crate::engine::layout::data::{ShapedData, ShapedItem};
use crate::protocol::prepared::{PresentationUnitKind as K, SelectedRelationEvent};
use crate::protocol::selection::SelectedData;
use crate::transform::relation::{ComponentOwner, RelationEvent};

pub(crate) struct ExecutionPlan {
    pub(crate) order: Vec<usize>,
    pub(crate) boxed_units: Vec<bool>,
}
struct Node<'a> {
    source: &'a InlineBoxSource,
    start: usize,
    end: usize,
    parent: Option<usize>,
    members: Option<(usize, usize)>,
    gap: Gap,
}
#[derive(Clone, Copy)]
enum Gap {
    Absent,
    At(usize),
    Ambiguous,
}
#[derive(Clone, Copy)]
enum SourceIncidence {
    Event(usize),
    Cut(usize),
}
impl SourceIncidence {
    fn position(self) -> usize {
        match self {
            Self::Event(at) | Self::Cut(at) => at,
        }
    }
    fn before_end(self, end: usize) -> bool {
        match self {
            Self::Event(at) => at < end,
            Self::Cut(at) => at <= end,
        }
    }
}
#[derive(Clone, Copy)]
struct Edge {
    item: usize,
    node: usize,
    side: LineEdgeSide,
    rank: usize,
    gap: usize,
}
fn contains(a: &Node<'_>, b: &Node<'_>) -> bool {
    a.start < b.start && b.end < a.end
}
fn find_source(nodes: &[Node<'_>], source: &InlineBoxSource, ledger: &mut Ledger) -> Result<usize> {
    account::work(ledger, nodes.len())?;
    nodes
        .iter()
        .position(|row| row.source == source)
        .ok_or_else(|| invalid(InlineBoxEdgeError::InvalidOwner))
}
fn unit_for_item(
    prepared: &crate::engine::PreparedData,
    presentation: &crate::protocol::prepared::SelectedPresentation,
    shaped: &ShapedData,
    item: ShapedItem,
    atom_units: &[usize],
    ledger: &mut Ledger,
) -> Result<usize> {
    if let ShapedItem::TextCluster(index) = item {
        return Ok(atom_units[shaped.clusters[index].atoms.start]);
    }
    if let ShapedItem::Anchor { unit } = item {
        return Ok(unit);
    }
    for (unit, row) in presentation.units.iter().enumerate() {
        account::work(ledger, 1)?;
        let matches = match item {
            ShapedItem::InlineBox { occurrence, .. } => row.kind == K::Atomic { occurrence },
            ShapedItem::Cell(cell) => matches!(row.kind, K::Cell { cell: original }
                if prepared.shaped.cells[original].binding == shaped.cells[cell].binding),
            _ => false,
        };
        if matches {
            return Ok(unit);
        }
    }
    Err(invalid(InlineBoxEdgeError::UnrepresentedIncidence))
}

pub(crate) fn build(
    selected: &SelectedData,
    shaped: &ShapedData,
    canonical: &[usize],
    ledger: &mut Ledger,
) -> Result<Option<ExecutionPlan>> {
    if contexts(&selected.request.controls, &[]).next().is_none() {
        return Ok(None);
    }
    account::work(ledger, selected.request.controls.len())?;
    let presentation = &selected.presentation;
    let mut nodes = Vec::new();
    for (_, context) in contexts(
        &selected.request.controls,
        &selected.request.boundary_context,
    ) {
        for source in [&context.owner, &context.parent] {
            account::work(ledger, nodes.len())?;
            if !nodes.iter().any(|node: &Node<'_>| node.source == source) {
                let (start, end) = source.interval();
                account::push_fallible(
                    ledger,
                    &mut nodes,
                    Node {
                        source,
                        start: start.index,
                        end: end.index,
                        parent: None,
                        members: None,
                        gap: Gap::Absent,
                    },
                )?;
            }
        }
    }
    comparison_work(ledger, nodes.len())?;
    nodes.sort_unstable_by_key(|row| row.start);
    for child in 0..nodes.len() {
        let mut parent = None;
        for candidate in 0..child {
            account::work(ledger, 1)?;
            if contains(&nodes[candidate], &nodes[child]) {
                parent = Some(candidate);
            }
        }
        nodes[child].parent = parent;
    }
    let mut boxed_units = account::filled_fallible(ledger, presentation.units.len(), false)?;
    let mut edges = Vec::new();
    for (control, context) in contexts(&selected.request.controls, &[]) {
        let node = find_source(&nodes, &context.owner, ledger)?;
        let occurrence = selected.prepared.tables.controls[control.index].occurrence;
        account::work(ledger, presentation.units.len())?;
        let unit = presentation
            .units
            .iter()
            .position(|row| row.kind == K::Control { occurrence })
            .ok_or_else(|| invalid(InlineBoxEdgeError::UnrepresentedIncidence))?;
        boxed_units[unit] = true;
        let actual = control
            .captured_control()
            .ok_or_else(|| invalid(InlineBoxEdgeError::MissingCapture))?;
        let (start, end) = context
            .owner
            .endpoints()
            .ok_or_else(|| invalid(InlineBoxEdgeError::InvalidOwner))?;
        let side = if actual == start {
            context.start_side
        } else if actual == end {
            match context.start_side {
                LineEdgeSide::Start => LineEdgeSide::End,
                LineEdgeSide::End => LineEdgeSide::Start,
            }
        } else {
            return Err(invalid(InlineBoxEdgeError::InvalidOwner));
        };
        account::work(ledger, shaped.items.len())?;
        let item = shaped
            .items
            .iter()
            .position(|item| matches!(item, ShapedItem::Anchor { unit: actual } if *actual == unit))
            .ok_or_else(|| invalid(InlineBoxEdgeError::UnrepresentedIncidence))?;
        account::push_fallible(
            ledger,
            &mut edges,
            Edge {
                item,
                node,
                side,
                rank: 0,
                gap: 0,
            },
        )?;
    }
    let mut edge_for_item = account::filled_fallible(ledger, shaped.items.len(), None)?;
    for (index, edge) in edges.iter().enumerate() {
        account::work(ledger, 1)?;
        edge_for_item[edge.item] = Some(index);
    }
    let groups = crate::protocol::boundary::groups_accounted(selected, shaped, ledger)?;
    let mut atom_groups = account::filled_fallible(ledger, presentation.projection.atoms.len(), 0)?;
    for (index, group) in groups.iter().enumerate() {
        account::work(ledger, group.len())?;
        atom_groups[group.clone()].fill(index);
    }
    let mut atom_units = account::filled_fallible(ledger, presentation.projection.atoms.len(), 0)?;
    for (unit, row) in presentation.units.iter().enumerate() {
        account::work(ledger, row.atoms.len())?;
        atom_units[row.atoms.clone()].fill(unit);
    }
    let count = groups
        .len()
        .checked_add(shaped.items.len())
        .ok_or_else(|| {
            crate::Error::new(
                crate::ErrorCode::LayoutFailed,
                "inline edge packet size overflow",
            )
        })?;
    let mut packets: Vec<Option<(usize, usize)>> = account::filled_fallible(ledger, count, None)?;
    let mut item_packets = account::filled_fallible(ledger, shaped.items.len(), 0)?;
    let mut item_units = account::filled_fallible(ledger, shaped.items.len(), 0)?;
    let mut payload = Vec::new();
    for (rank, &item) in canonical.iter().enumerate() {
        account::work(ledger, 1)?;
        if let Some(edge) = edge_for_item[item] {
            edges[edge].rank = rank;
            edges[edge].gap = payload.len();
            let node = &mut nodes[edges[edge].node];
            node.gap = match node.gap {
                Gap::Absent => Gap::At(payload.len()),
                Gap::At(previous) if previous == payload.len() => Gap::At(previous),
                Gap::At(_) | Gap::Ambiguous => Gap::Ambiguous,
            };
            continue;
        }
        let unit = unit_for_item(
            &selected.prepared,
            presentation,
            shaped,
            shaped.items[item],
            &atom_units,
            ledger,
        )?;
        item_units[item] = unit;
        let packet = match shaped.items[item] {
            ShapedItem::TextCluster(index) => atom_groups[shaped.clusters[index].atoms.start],
            ShapedItem::Cell(_) if !presentation.units[unit].atoms.is_empty() => {
                atom_groups[presentation.units[unit].atoms.start]
            }
            ShapedItem::Anchor { .. } => {
                let cut = presentation.units[unit].atoms.start;
                let mut low = 0;
                let mut high = groups.len();
                while low < high {
                    account::work(ledger, 1)?;
                    let middle = low + (high - low) / 2;
                    if groups[middle].end <= cut {
                        low = middle + 1;
                    } else {
                        high = middle;
                    }
                }
                if groups
                    .get(low)
                    .is_some_and(|range| range.start < cut && cut < range.end)
                {
                    low
                } else {
                    groups.len() + item
                }
            }
            _ => groups.len() + item,
        };
        item_packets[item] = packet;
        let position = payload.len();
        packets[packet] =
            Some(packets[packet].map_or((position, position + 1), |(a, _)| (a, position + 1)));
        account::push_fallible(ledger, &mut payload, item)?;
    }
    let mut unit_bounds: Vec<Option<(usize, usize)>> =
        account::filled_fallible(ledger, presentation.units.len(), None)?;
    for &item in &payload {
        account::work(ledger, 1)?;
        let (first, after) = packets[item_packets[item]]
            .ok_or_else(|| invalid(InlineBoxEdgeError::UnrepresentedIncidence))?;
        let row = &mut unit_bounds[item_units[item]];
        *row = Some(row.map_or((first, after), |(a, b)| (a.min(first), b.max(after))));
    }
    for (unit, bounds) in unit_bounds.iter().enumerate() {
        account::work(ledger, 1)?;
        let Some((first, after)) = *bounds else {
            continue;
        };
        visit_unit_events(
            &selected.prepared,
            presentation,
            unit,
            ledger,
            |incidence, ledger| {
                let event = incidence.position();
                let mut low = 0;
                let mut high = nodes.len();
                while low < high {
                    account::work(ledger, 1)?;
                    let middle = low + (high - low) / 2;
                    if nodes[middle].start < event {
                        low = middle + 1;
                    } else {
                        high = middle;
                    }
                }
                let mut node = low.checked_sub(1);
                while let Some(index) = node {
                    account::work(ledger, 1)?;
                    if incidence.before_end(nodes[index].end) {
                        let row = &mut nodes[index].members;
                        *row =
                            Some(row.map_or((first, after), |(a, b)| (a.min(first), b.max(after))));
                    }
                    node = nodes[index].parent;
                }
                Ok(())
            },
        )?;
    }
    for edge in &mut edges {
        account::work(ledger, 1)?;
        edge.gap = if let Some((first, after)) = nodes[edge.node].members {
            match edge.side {
                LineEdgeSide::Start => first,
                LineEdgeSide::End => after,
            }
        } else {
            let Gap::At(gap) = nodes[edge.node].gap else {
                return Err(invalid(InlineBoxEdgeError::UnrepresentedIncidence));
            };
            // The genuine original endpoint gap can be inside one retained
            // provider/EGC packet. Close that incidence over the actual packet;
            // do not split it or fabricate a body packet for the empty owner.
            let mut insertion = gap;
            for packet in packets.iter().flatten() {
                account::work(ledger, 1)?;
                if packet.0 < gap && gap < packet.1 {
                    insertion = match edge.side {
                        LineEdgeSide::Start => packet.0,
                        LineEdgeSide::End => packet.1,
                    };
                    break;
                }
            }
            insertion
        };
    }
    comparison_work(ledger, edges.len())?;
    edges.sort_unstable_by_key(|edge| (edge.gap, edge.rank));
    let mut emitted = account::filled_fallible(ledger, edges.len(), false)?;
    let mut order = Vec::new();
    let mut edge_cursor = 0;
    for gap in 0..=payload.len() {
        let first_edge = edge_cursor;
        while edge_cursor < edges.len() {
            account::work(ledger, 1)?;
            if edges[edge_cursor].gap != gap {
                break;
            }
            edge_cursor += 1;
        }
        loop {
            let mut selected_edge = None;
            for candidate in first_edge..edge_cursor {
                let edge = &edges[candidate];
                account::work(ledger, 1)?;
                if emitted[candidate] || edge.gap != gap {
                    continue;
                }
                let mut blocked = false;
                for other in first_edge..edge_cursor {
                    let prior = &edges[other];
                    account::work(ledger, 1)?;
                    if emitted[other] || other == candidate || prior.gap != gap {
                        continue;
                    }
                    let related = prior.node == edge.node
                        || contains(&nodes[prior.node], &nodes[edge.node])
                        || contains(&nodes[edge.node], &nodes[prior.node]);
                    let precedes = if related {
                        match (prior.side, edge.side) {
                            (LineEdgeSide::Start, LineEdgeSide::End) => true,
                            (LineEdgeSide::End, LineEdgeSide::Start) => false,
                            (LineEdgeSide::Start, LineEdgeSide::Start) => {
                                contains(&nodes[prior.node], &nodes[edge.node])
                            }
                            (LineEdgeSide::End, LineEdgeSide::End) => {
                                contains(&nodes[edge.node], &nodes[prior.node])
                            }
                        }
                    } else {
                        prior.rank < edge.rank
                    };
                    blocked |= precedes;
                }
                if !blocked && selected_edge.is_none_or(|i: usize| edges[i].rank > edge.rank) {
                    selected_edge = Some(candidate);
                }
            }
            let Some(index) = selected_edge else {
                break;
            };
            emitted[index] = true;
            account::push_fallible(ledger, &mut order, edges[index].item)?;
        }
        for index in first_edge..edge_cursor {
            let edge = &edges[index];
            account::work(ledger, 1)?;
            if edge.gap == gap && !emitted[index] {
                return Err(invalid(InlineBoxEdgeError::UnrepresentedIncidence));
            }
        }
        if let Some(&item) = payload.get(gap) {
            account::push_fallible(ledger, &mut order, item)?;
        }
    }
    account::drop_vec(ledger, edge_for_item)?;
    account::drop_vec(ledger, item_packets)?;
    // Every temporary backing has one actual admission and release. The returned
    // order/membership buffers remain alive through all consumers of execution.
    account::drop_vec(ledger, nodes)?;
    account::drop_vec(ledger, edges)?;
    account::drop_vec(ledger, groups)?;
    account::drop_vec(ledger, atom_groups)?;
    account::drop_vec(ledger, atom_units)?;
    account::drop_vec(ledger, packets)?;
    account::drop_vec(ledger, item_units)?;
    account::drop_vec(ledger, payload)?;
    account::drop_vec(ledger, unit_bounds)?;
    account::drop_vec(ledger, emitted)?;
    Ok(Some(ExecutionPlan { order, boxed_units }))
}

fn visit_unit_events(
    prepared: &crate::engine::PreparedData,
    presentation: &crate::protocol::prepared::SelectedPresentation,
    unit: usize,
    ledger: &mut Ledger,
    mut emit: impl FnMut(SourceIncidence, &mut Ledger) -> Result<()>,
) -> Result<()> {
    let analyzed = &prepared.analyzed;
    let paragraph = &analyzed.paragraph;
    let topology = paragraph
        .scope_topology
        .as_ref()
        .ok_or_else(|| invalid(InlineBoxEdgeError::MissingCapture))?;
    let transform = &analyzed.transform;
    // Components are source-owned; no glyph hull or presentation interval grants
    // ownership. Generated operations retain their actual original site.
    for relation in &presentation.selected_relation.events
        [presentation.selected_relation.unit_events[unit].clone()]
    {
        account::work(ledger, 1)?;
        let SelectedRelationEvent::Base { event } = *relation else {
            continue;
        };
        let event = &transform.events[event];
        match *event {
            RelationEvent::Text { atom } => {
                let component = transform.component_for_atom(atom);
                match component.owner {
                    ComponentOwner::Original(row) => {
                        visit_original_row(prepared, row, ledger, &mut emit)?
                    }
                    ComponentOwner::Wbr(control) => emit(
                        SourceIncidence::Event(topology.control_orders[control]),
                        ledger,
                    )?,
                    ComponentOwner::Virtual(index) => emit(
                        SourceIncidence::Cut(transform.virtuals[index].capture_cut),
                        ledger,
                    )?,
                }
            }
            RelationEvent::Removal { row, .. } => {
                visit_original_row(prepared, row, ledger, &mut emit)?
            }
            RelationEvent::GeneratedSuppression { component, .. } => {
                match transform.components[component].owner {
                    ComponentOwner::Original(row) => {
                        visit_original_row(prepared, row, ledger, &mut emit)?
                    }
                    ComponentOwner::Wbr(control) => emit(
                        SourceIncidence::Event(topology.control_orders[control]),
                        ledger,
                    )?,
                    ComponentOwner::Virtual(index) => emit(
                        SourceIncidence::Cut(transform.virtuals[index].capture_cut),
                        ledger,
                    )?,
                }
            }
            RelationEvent::PhaseIDeleted { source, .. } => {
                paragraph.visit_origin_indices(source, |visit| {
                    match visit {
                        crate::protocol::paragraph::OriginVisit::Inspect => {
                            account::work(ledger, 1)?
                        }
                        crate::protocol::paragraph::OriginVisit::Intersection {
                            origin, ..
                        } => emit(
                            SourceIncidence::Event(
                                paragraph.captured_origin_event(origin).ok_or_else(|| {
                                    invalid(InlineBoxEdgeError::UnrepresentedIncidence)
                                })?,
                            ),
                            ledger,
                        )?,
                    }
                    Ok(())
                })?
            }
            RelationEvent::Anchor { order } => {
                use crate::protocol::paragraph::SemanticAnchorItem as A;
                match paragraph.anchor_order[order].item {
                    A::Control(control) => emit(
                        SourceIncidence::Event(topology.control_orders[control]),
                        ledger,
                    )?,
                    A::Atomic(atomic) => emit(
                        SourceIncidence::Event(topology.capture.atomic_orders[atomic]),
                        ledger,
                    )?,
                    A::EmptyOrigin(origin) => emit(
                        SourceIncidence::Event(
                            paragraph.captured_origin_event(origin).ok_or_else(|| {
                                invalid(InlineBoxEdgeError::UnrepresentedIncidence)
                            })?,
                        ),
                        ledger,
                    )?,
                    A::ComputedLineBoundary(boundary) => {
                        for &cause in &paragraph.computed_boundaries[boundary].causes {
                            account::work(ledger, 1)?;
                            emit(
                                SourceIncidence::Cut(topology.capture.computed[cause].capture_cut),
                                ledger,
                            )?;
                        }
                    }
                }
            }
        }
    }
    if let K::Replacement { alternative, .. } = presentation.units[unit].kind {
        use crate::protocol::prepared::AlternativeDefinition as D;
        let event = match prepared.tables.alternatives[alternative].definition {
            D::Source { origin, .. } | D::Automatic { origin, .. } => {
                paragraph.captured_origin_event(origin)
            }
            D::Control { control, .. } => Some(topology.control_orders[control]),
        }
        .ok_or_else(|| invalid(InlineBoxEdgeError::UnrepresentedIncidence))?;
        emit(SourceIncidence::Event(event), ledger)?;
    }
    Ok(())
}

fn visit_original_row(
    prepared: &crate::engine::PreparedData,
    row: usize,
    ledger: &mut Ledger,
    emit: &mut impl FnMut(SourceIncidence, &mut Ledger) -> Result<()>,
) -> Result<()> {
    let transform = &prepared.analyzed.transform;
    for piece in &transform.pieces[transform.rows[row].pieces.clone()] {
        account::work(ledger, 1)?;
        emit(
            SourceIncidence::Event(
                piece
                    .origin
                    .captured_event()
                    .ok_or_else(|| invalid(InlineBoxEdgeError::UnrepresentedIncidence))?,
            ),
            ledger,
        )?;
    }
    Ok(())
}
