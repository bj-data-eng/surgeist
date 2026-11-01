//! Native formatter occurrences, before fitting-run and descriptor filtering.
//! This owner records actual graph facts; it does not manufacture Text input.
use super::{
    GeneratedBoxRef, InlineBoxRef, InlineDelimiterSide, InlineFormatterEdge, InlineGapAffinity,
    InlineGapAnchor, InlineGapGroupInput, ScopeRow, WrappingScopeError,
};
use crate::{
    Direction, LayoutInputViewOf, LayoutTree, Position, engine::validation::ValidationGraph,
};
use std::{ops::Range, sync::Arc};
use surgeist_text::{BidiLevel, SuppliedInlineParticipant, SuppliedLine};

fn proof_ancestor(
    parents: &[Option<usize>],
    root: usize,
    ancestor: usize,
    mut child: usize,
) -> Result<bool> {
    for _ in 0..=parents.len() {
        if child == ancestor {
            return Ok(true);
        }
        if child == root {
            return Ok(false);
        }
        child = parents
            .get(child)
            .copied()
            .flatten()
            .ok_or(WrappingScopeError::IncompleteFamily)?;
    }
    Err(WrappingScopeError::ChildOrder)
}
type Result<T> = std::result::Result<T, WrappingScopeError>;
fn buffer<T>(count: usize) -> Result<Vec<T>> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| WrappingScopeError::Allocation)?;
    Ok(result)
}
fn push<T>(rows: &mut Vec<T>, value: T) -> Result<()> {
    rows.try_reserve(1)
        .map_err(|_| WrappingScopeError::Allocation)?;
    rows.push(value);
    Ok(())
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Role {
    Operand(BidiLevel),
    Structural,
    Alias,
    Prepared,
    Metadata,
    Delimiter,
}
#[derive(Clone, Copy, Debug)]
struct Slot {
    role: Role,
    body: usize,
    visual: Option<usize>,
    previous: Option<usize>,
    next: Option<usize>,
}
#[derive(Debug)]
struct Body {
    range: Range<usize>,
    visual: Vec<usize>,
    prepared: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Witness {
    Neighbor {
        rank: usize,
        affinity: InlineGapAffinity,
    },
    Start,
    End,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Anchor {
    Explicit(Witness),
    Derived {
        preceding: Witness,
        following: Witness,
    },
    Parent(Option<usize>),
}
#[derive(Debug)]
struct Group {
    body: usize,
    anchor: Anchor,
    owners: Vec<usize>,
}
#[derive(Debug)]
struct BoxRow {
    owner: usize,
    range: Range<usize>,
}
/// One immutable native occurrence profile for an actual formatter.
#[derive(Debug)]
pub(crate) struct NativeOrderingData {
    slots: Vec<Slot>,
    bodies: Vec<Body>,
    boxes: Vec<BoxRow>,
    groups: Vec<Group>,
    direction: Direction,
    family: Option<super::family::NativeFamilyData>,
}
#[derive(Clone, Debug)]
pub(super) struct NativeOrderingProfile<Node> {
    pub(super) flow: Node,
    pub(super) data: Arc<NativeOrderingData>,
}

pub(super) fn prepare<Node: Copy + Eq, Tree: LayoutTree<Node = Node>>(
    tree: &Tree,
    graph: &ValidationGraph<Node>,
    rows: &[ScopeRow<Node>],
    gaps: &[InlineGapGroupInput<Node>],
    demands: &[super::NativeInlineBoxFamilyInput<Node>],
    proof: Option<&Arc<super::edge_plan::AdmittedGeneratedBoxProof<Node>>>,
    prepared_execution: &impl Fn(Node) -> bool,
) -> Result<Vec<NativeOrderingProfile<Node>>> {
    // Existence belongs to every supplied node reference, before interpreting
    // its role or looking for an admitted box/body in this formatter.
    let require_node = |node: Node| -> Result<()> {
        graph
            .index(node)
            .map(|_| ())
            .ok_or(WrappingScopeError::NodeNotReachable)
    };
    let require_box_nodes = |reference: &InlineBoxRef<Node>| -> Result<()> {
        if let InlineBoxRef::NativePair { start, end } = reference {
            require_node(*start)?;
            require_node(*end)?;
        }
        Ok(())
    };
    for group in gaps {
        for reference in &group.boxes {
            require_box_nodes(reference)?;
        }
        match &group.anchor {
            InlineGapAnchor::Neighbor { node, .. }
            | InlineGapAnchor::Formatter { node, .. }
            | InlineGapAnchor::Delimiter { node, .. } => require_node(*node)?,
            InlineGapAnchor::EmptyParent(GeneratedBoxRef::Node(node)) => require_node(*node)?,
            InlineGapAnchor::EmptyParent(GeneratedBoxRef::Inline(reference)) => {
                require_box_nodes(reference)?
            }
        }
    }
    let mut flows = Vec::new();
    for (index, demand) in demands.iter().enumerate() {
        require_node(demand.formatter)?;
        if demands[..index]
            .iter()
            .any(|prior| prior.formatter == demand.formatter)
        {
            return Err(WrappingScopeError::DuplicateFamily);
        }
        if prepared_execution(demand.formatter) {
            return Err(WrappingScopeError::UnsupportedIncidence);
        }
        push(&mut flows, demand.formatter)?;
    }
    for (index, &node) in graph.nodes.iter().enumerate() {
        if matches!(tree.layout_input(node), LayoutInputViewOf::InlineBoundary(input) if input.parent_edge_axes().is_some())
        {
            let parents = &graph.parents[index];
            if parents.len() != 1 {
                return Err(WrappingScopeError::AmbiguousNode);
            }
            let flow = graph.nodes[parents[0]];
            if !flows.contains(&flow) {
                flows
                    .try_reserve(1)
                    .map_err(|_| WrappingScopeError::Allocation)?;
                flows.push(flow);
            }
        }
    }
    let resolve_box = |reference: &InlineBoxRef<Node>| -> Result<usize> {
        rows.iter()
            .position(|row| match reference {
                InlineBoxRef::NativePair { start, end } => row.pair == Some((*start, *end)),
                InlineBoxRef::Scope(scope) => row.capture.as_ref() == Some(scope),
            })
            .ok_or(WrappingScopeError::InvalidGapAnchor)
    };
    for group in gaps {
        if group.boxes.is_empty() {
            return Err(WrappingScopeError::InvalidGapAnchor);
        }
        for reference in &group.boxes {
            let row = &rows[resolve_box(reference)?];
            let demanded = demands.iter().any(|d| d.formatter == row.flow);
            if (row.parent_edge_axes.is_none() && !demanded) || !flows.contains(&row.flow) {
                return Err(WrappingScopeError::InvalidGapAnchor);
            }
        }
    }
    let mut profiles = buffer(flows.len())?;
    for flow in flows {
        let index = graph
            .index(flow)
            .ok_or(WrappingScopeError::NodeNotReachable)?;
        let children = &graph.children[index];
        let admission = if demands.iter().any(|d| d.formatter == flow) {
            Some(
                proof
                    .ok_or(WrappingScopeError::MissingChildOrder)?
                    .family(tree, graph, rows, flow)?,
            )
        } else {
            None
        };
        let direction = tree.node_input(flow).direction;
        let base =
            BidiLevel::try_new(u8::from(direction == Direction::Rtl)).expect("paragraph level");
        let mut slots = buffer(children.len())?;
        let mut bodies = Vec::new();
        let mut body = 0;
        let mut start = 0;
        for (rank, &child) in children.iter().enumerate() {
            let role = match tree.layout_input(graph.nodes[child]) {
                LayoutInputViewOf::Box(input) => {
                    if input.display == crate::Display::None
                        || input.position == Position::Absolute
                        || !input.float.is_none()
                    {
                        Role::Metadata
                    } else if input.display.is_inline_level() {
                        // Prepared-backed atomics have their level in the Text adapter.
                        // They remain explicitly unavailable to native gap derivation.
                        input
                            .atomic_inline_participation
                            .map_or(Role::Prepared, |p| Role::Operand(p.bidi_level()))
                    } else {
                        Role::Delimiter
                    }
                }
                LayoutInputViewOf::InlineBoundary(input) => {
                    if input.parent_edge_axes().is_some() {
                        Role::Structural
                    } else if admission.as_ref().is_some_and(|a| {
                        rows.iter().enumerate().any(|(owner, row)| {
                            a.scope_keys[owner].is_some()
                                && row.pair.is_some_and(|(start, end)| {
                                    start == graph.nodes[child] || end == graph.nodes[child]
                                })
                        })
                    }) {
                        if input.edge().flow_advance()
                            != <Tree::Scalar as crate::LayoutScalar>::ZERO
                        {
                            return Err(WrappingScopeError::IncompatibleContribution);
                        }
                        Role::Alias
                    } else {
                        Role::Operand(base)
                    }
                }
                LayoutInputViewOf::LineBreak(input) => {
                    if input.display().is_none() {
                        Role::Metadata
                    } else {
                        Role::Delimiter
                    }
                }
                LayoutInputViewOf::InlineText(_) | LayoutInputViewOf::TextControl(_) => {
                    Role::Prepared
                }
            };
            slots.push(Slot {
                role,
                body,
                visual: None,
                previous: None,
                next: None,
            });
            if role == Role::Delimiter {
                push(
                    &mut bodies,
                    Body {
                        range: start..rank,
                        visual: Vec::new(),
                        prepared: false,
                    },
                )?;
                body += 1;
                start = rank + 1;
            }
        }
        push(
            &mut bodies,
            Body {
                range: start..children.len(),
                visual: Vec::new(),
                prepared: false,
            },
        )?;
        for actual in &mut bodies {
            let mut supplied = buffer(actual.range.len())?;
            let mut logical = buffer(actual.range.len())?;
            let mut previous = None;
            for rank in actual.range.clone() {
                slots[rank].previous = previous;
                if matches!(slots[rank].role, Role::Operand(_) | Role::Prepared) {
                    previous = Some(rank);
                }
                match slots[rank].role {
                    Role::Operand(level) => {
                        supplied.push(SuppliedInlineParticipant::Atomic { level });
                        logical.push(rank);
                    }
                    Role::Prepared => actual.prepared = true,
                    _ => {}
                }
            }
            let mut next = None;
            for rank in actual.range.clone().rev() {
                slots[rank].next = next;
                if matches!(slots[rank].role, Role::Operand(_) | Role::Prepared) {
                    next = Some(rank);
                }
            }
            let mut permutation = buffer(supplied.len())?;
            permutation.resize(supplied.len(), 0);
            SuppliedLine::new(&supplied)
                .reorder_into(&mut permutation)
                .map_err(|_| WrappingScopeError::InconsistentIncidence)?;
            actual.visual = buffer(permutation.len())?;
            for (position, local) in permutation.into_iter().enumerate() {
                let rank = logical[local];
                slots[rank].visual = Some(position);
                actual.visual.push(rank);
            }
        }
        let mut boxes = Vec::new();
        for (owner, row) in rows.iter().enumerate().filter(|(_, row)| row.flow == flow) {
            if let Some(span) = &row.native {
                boxes
                    .try_reserve(1)
                    .map_err(|_| WrappingScopeError::Allocation)?;
                boxes.push(BoxRow {
                    owner,
                    range: span.first..span.last + 1,
                });
            }
        }
        let mut data = NativeOrderingData {
            slots,
            bodies,
            boxes,
            groups: Vec::new(),
            direction,
            family: None,
        };
        let rank_of = |node: Node| -> Result<usize> {
            if graph.index(node).is_none() {
                return Err(WrappingScopeError::NodeNotReachable);
            }
            let mut matches = children
                .iter()
                .enumerate()
                .filter(|(_, child)| graph.nodes[**child] == node);
            let rank = matches
                .next()
                .map(|(rank, _)| rank)
                .ok_or(WrappingScopeError::InvalidGapAnchor)?;
            if matches.next().is_some() {
                return Err(WrappingScopeError::AmbiguousNode);
            }
            Ok(rank)
        };
        let mut admitted_execution = None;
        for input in gaps {
            let first_owner = resolve_box(&input.boxes[0])?;
            if rows[first_owner].flow != flow {
                continue;
            }
            let mut owners = buffer(input.boxes.len())?;
            for reference in &input.boxes {
                let owner = resolve_box(reference)?;
                if rows[owner].flow != flow {
                    return Err(WrappingScopeError::InvalidGapAnchor);
                }
                if owners.contains(&owner) {
                    return Err(WrappingScopeError::ConflictingGap);
                }
                owners.push(owner);
            }
            // Native gaps must not be admitted for actual scheduled Text runs,
            // even when their only native nodes are structural controls. Query
            // that retained authority only for this requested formatter.
            if *admitted_execution.get_or_insert_with(|| prepared_execution(flow))
                || data.bodies.iter().any(|body| body.prepared)
            {
                return Err(WrappingScopeError::UnsupportedIncidence);
            }
            let (body, anchor) = match &input.anchor {
                InlineGapAnchor::Neighbor { node, affinity } => {
                    let rank = rank_of(*node)?;
                    if !matches!(data.slots[rank].role, Role::Operand(_)) {
                        return Err(WrappingScopeError::InvalidGapAnchor);
                    }
                    (
                        data.slots[rank].body,
                        Anchor::Explicit(Witness::Neighbor {
                            rank,
                            affinity: *affinity,
                        }),
                    )
                }
                InlineGapAnchor::Formatter { node, edge } => {
                    if *node != flow {
                        return Err(WrappingScopeError::InvalidGapAnchor);
                    }
                    match edge {
                        InlineFormatterEdge::Start => (0, Anchor::Explicit(Witness::Start)),
                        InlineFormatterEdge::End => {
                            (data.bodies.len() - 1, Anchor::Explicit(Witness::End))
                        }
                    }
                }
                InlineGapAnchor::Delimiter { node, side } => {
                    let rank = rank_of(*node)?;
                    if data.slots[rank].role != Role::Delimiter {
                        return Err(WrappingScopeError::InvalidGapAnchor);
                    }
                    match side {
                        InlineDelimiterSide::Before => {
                            (data.slots[rank].body, Anchor::Explicit(Witness::End))
                        }
                        InlineDelimiterSide::After => {
                            (data.slots[rank].body + 1, Anchor::Explicit(Witness::Start))
                        }
                    }
                }
                InlineGapAnchor::EmptyParent(parent)
                    if admission.is_some() && data.unique_empty_raw_frontier() =>
                {
                    let admitted = admission
                        .as_ref()
                        .ok_or(WrappingScopeError::IncompleteFamily)?;
                    let proof = proof.ok_or(WrappingScopeError::MissingChildOrder)?;
                    let parent = match parent {
                        GeneratedBoxRef::Node(node) if *node == flow => admitted.root,
                        GeneratedBoxRef::Inline(reference @ InlineBoxRef::Scope(_)) => {
                            let owner = resolve_box(reference)?;
                            if rows[owner].flow != flow || !proof.captured_orders_validated {
                                return Err(WrappingScopeError::InvalidGapAnchor);
                            }
                            admitted.scope_keys[owner]
                                .ok_or(WrappingScopeError::InvalidGapAnchor)?
                        }
                        _ => return Err(WrappingScopeError::InvalidGapAnchor),
                    };
                    for &owner in &owners {
                        let child = admitted.scope_keys[owner]
                            .ok_or(WrappingScopeError::InvalidGapAnchor)?;
                        if !proof.captured_complete_normal(child)
                            || proof.data.parents[child] != Some(parent)
                        {
                            return Err(WrappingScopeError::InvalidGapAnchor);
                        }
                    }
                    // This is the same actual Start/End raw frontier. A parent
                    // identity authenticates nesting, not an invented span.
                    (
                        0,
                        Anchor::Derived {
                            preceding: Witness::Start,
                            following: Witness::End,
                        },
                    )
                }
                InlineGapAnchor::EmptyParent(parent) => {
                    let (parent_range, parent_owner) = match parent {
                        GeneratedBoxRef::Node(node) if *node == flow => (0..children.len(), None),
                        GeneratedBoxRef::Inline(reference) => {
                            let owner = resolve_box(reference)?;
                            if rows[owner].flow != flow {
                                return Err(WrappingScopeError::InvalidGapAnchor);
                            }
                            (data.box_row(owner)?.range.clone(), Some(owner))
                        }
                        _ => return Err(WrappingScopeError::InvalidGapAnchor),
                    };
                    for &owner in &owners {
                        let child = &data.box_row(owner)?.range;
                        if !(parent_range.start <= child.start && child.end <= parent_range.end)
                            || parent_owner.is_some() && child == &parent_range
                        {
                            return Err(WrappingScopeError::InvalidGapAnchor);
                        }
                        if data.boxes.iter().any(|middle| {
                            Some(middle.owner) != parent_owner
                                && middle.owner != owner
                                && parent_range.start <= middle.range.start
                                && middle.range.end <= parent_range.end
                                && middle.range.start < child.start
                                && child.end < middle.range.end
                        }) {
                            return Err(WrappingScopeError::InvalidGapAnchor);
                        }
                    }
                    let mut found = None;
                    for (body, actual) in data.bodies.iter().enumerate() {
                        let first = parent_range.start.max(actual.range.start);
                        let after = parent_range.end.min(actual.range.end);
                        if first >= after
                            || owners.iter().any(|&owner| {
                                let child = &data
                                    .boxes
                                    .iter()
                                    .find(|row| row.owner == owner)
                                    .expect("admitted box")
                                    .range;
                                child.start >= actual.range.end || actual.range.start >= child.end
                            })
                        {
                            continue;
                        }
                        let content = if parent_owner.is_some() {
                            parent_range.start + 1..parent_range.end - 1
                        } else {
                            parent_range.clone()
                        };
                        if data.slots[first..after]
                            .iter()
                            .enumerate()
                            .any(|(offset, slot)| {
                                content.contains(&(first + offset))
                                    && matches!(slot.role, Role::Operand(_) | Role::Prepared)
                            })
                        {
                            continue;
                        }
                        if found.is_some() {
                            return Err(WrappingScopeError::AmbiguousGap);
                        }
                        found = Some((body, Anchor::Parent(parent_owner)));
                    }
                    found.ok_or(WrappingScopeError::InvalidGapAnchor)?
                }
            };
            for &owner in &owners {
                if data
                    .groups
                    .iter()
                    .any(|group| group.body == body && group.owners.contains(&owner))
                {
                    return Err(WrappingScopeError::ConflictingGap);
                }
                let Some(row) = data.native_box_row(owner) else {
                    let admitted = admission
                        .as_ref()
                        .ok_or(WrappingScopeError::InvalidGapAnchor)?;
                    let key =
                        admitted.scope_keys[owner].ok_or(WrappingScopeError::IncompleteFamily)?;
                    let proof = proof.ok_or(WrappingScopeError::MissingChildOrder)?;
                    if !(proof.captured_empty_leaf(key)
                        || data.unique_empty_raw_frontier() && proof.captured_complete_normal(key))
                    {
                        return Err(WrappingScopeError::UnsupportedIncidence);
                    }
                    proof.validate_attachment(
                        key,
                        admitted.root,
                        data.raw_frontier(anchor, body)?,
                    )?;
                    continue;
                };
                let range = &data.bodies[body].range;
                if row.range.start >= range.end || range.start >= row.range.end {
                    return Err(WrappingScopeError::InvalidGapAnchor);
                }
            }
            data.groups
                .try_reserve(1)
                .map_err(|_| WrappingScopeError::Allocation)?;
            data.groups.push(Group {
                body,
                anchor,
                owners,
            });
        }
        // Strict native containment orders an actual parent's group before its
        // children, independently of input order. This is a temporary index into
        // this owner's groups, not another box tree or recursive traversal.
        let mut group_order = buffer(data.groups.len())?;
        for (index, group) in data.groups.iter().enumerate() {
            let mut first = usize::MAX;
            for &owner in &group.owners {
                first = first.min(if let Some(row) = data.native_box_row(owner) {
                    row.range.start
                } else {
                    data.raw_frontier(group.anchor, group.body)?
                });
            }
            group_order.push((first, index));
        }
        group_order.sort_unstable();
        for (_, index) in group_order {
            let Anchor::Parent(parent) = data.groups[index].anchor else {
                continue;
            };
            let body = data.groups[index].body;
            let inherited = parent.and_then(|owner| {
                data.groups
                    .iter()
                    .find(|row| row.body == body && row.owners.contains(&owner))
            });
            let anchor = if let Some(row) = inherited {
                if matches!(row.anchor, Anchor::Parent(_)) {
                    return Err(WrappingScopeError::CyclicGap);
                }
                row.anchor
            } else {
                let range = if let Some(owner) = parent {
                    let range = &data.box_row(owner)?.range;
                    range.start + 1..range.end - 1
                } else {
                    data.bodies[body].range.clone()
                };
                data.derive_gap(range, body)?
            };
            data.groups[index].anchor = anchor;
        }
        for group in &data.groups {
            let body = &data.bodies[group.body];
            let gap = data.anchor_gap(group.anchor, group.body)?;
            for &owner in &group.owners {
                let Some(row) = data.native_box_row(owner) else {
                    continue;
                };
                let range = &row.range;
                let mut extent = None;
                for rank in
                    (range.start + 1).max(body.range.start)..(range.end - 1).min(body.range.end)
                {
                    if data.slots[rank].role == Role::Prepared {
                        return Err(WrappingScopeError::UnsupportedIncidence);
                    }
                    if let Some(position) = data.slots[rank].visual {
                        extent = Some(extent.map_or(
                            (position, position),
                            |(first, last): (usize, usize)| {
                                (first.min(position), last.max(position))
                            },
                        ));
                    }
                }
                if let Some((first, last)) = extent
                    && gap != first
                    && gap != last + 1
                {
                    return Err(WrappingScopeError::ConflictingGap);
                }
            }
            // A physical tie cannot interpose an unrelated box between an
            // admitted ancestor and descendant. Captured attachments use the
            // original proof ancestry, not a fabricated native range.
            let contains = |owner: usize, child: usize| -> Result<bool> {
                if let Some(admitted) = &admission {
                    return proof_ancestor(
                        &proof
                            .ok_or(WrappingScopeError::MissingChildOrder)?
                            .data
                            .parents,
                        admitted.root,
                        admitted.scope_keys[owner].ok_or(WrappingScopeError::IncompleteFamily)?,
                        admitted.scope_keys[child].ok_or(WrappingScopeError::IncompleteFamily)?,
                    );
                }
                let parent = &data.box_row(owner)?.range;
                let descendant = &data.box_row(child)?.range;
                Ok(
                    owner == child
                        || parent.start < descendant.start && descendant.end < parent.end,
                )
            };
            for (index, &owner) in group.owners.iter().enumerate() {
                for (child_index, &child) in group.owners.iter().enumerate() {
                    if owner == child || !contains(owner, child)? {
                        continue;
                    }
                    for &other in &group.owners[index.min(child_index)..index.max(child_index)] {
                        if !contains(owner, other)? && !contains(other, owner)? {
                            return Err(WrappingScopeError::ConflictingGap);
                        }
                    }
                }
            }
        }
        if let Some(admission) = admission {
            if data.bodies.iter().any(|body| body.prepared) {
                return Err(WrappingScopeError::UnsupportedIncidence);
            }
            data.build_family(
                tree,
                graph,
                flow,
                proof.ok_or(WrappingScopeError::MissingChildOrder)?,
                admission,
            )?;
        }
        profiles.push(NativeOrderingProfile {
            flow,
            data: Arc::new(data),
        });
    }
    Ok(profiles)
}
impl NativeOrderingData {
    pub(crate) fn selected_body(&self, selection: &NativeOrderingSelection<'_>) -> Result<usize> {
        if let Some(&first) = selection.ranks.first() {
            return self.body(first);
        }
        let mut found = None;
        for (index, body) in self.bodies.iter().enumerate() {
            if body.range.start <= selection.raw_fences.start
                && selection.raw_fences.end <= body.range.end
            {
                if found.is_some() {
                    return Err(WrappingScopeError::AmbiguousGap);
                }
                found = Some(index);
            }
        }
        found.ok_or(WrappingScopeError::GapOutsideSelection)
    }
    pub(crate) fn body(&self, rank: usize) -> Result<usize> {
        self.slots
            .get(rank)
            .map(|s| s.body)
            .ok_or(WrappingScopeError::InconsistentIncidence)
    }
    pub(crate) fn has_family(&self) -> bool {
        self.family.is_some()
    }
    pub(crate) fn family(&self) -> Option<&super::family::NativeFamilyData> {
        self.family.as_ref()
    }
    pub(crate) fn structural(&self, rank: usize) -> bool {
        self.slots
            .get(rank)
            .is_some_and(|s| matches!(s.role, Role::Structural | Role::Alias))
    }
    pub(crate) fn source_start_left(&self) -> bool {
        self.direction == Direction::Ltr
    }
    pub(crate) fn alias(&self, rank: usize) -> bool {
        self.slots.get(rank).is_some_and(|s| s.role == Role::Alias)
    }
    pub(crate) fn endpoint(&self, rank: usize) -> Option<(usize, bool)> {
        self.boxes.iter().find_map(|b| {
            if b.range.start == rank {
                Some((b.owner, true))
            } else if b.range.end == rank + 1 {
                Some((b.owner, false))
            } else {
                None
            }
        })
    }
    pub(crate) fn family_key(&self, owner: usize) -> Option<usize> {
        self.family
            .as_ref()?
            .scope_keys
            .get(owner)
            .copied()
            .flatten()
    }

    fn unique_empty_raw_frontier(&self) -> bool {
        self.slots.is_empty() && self.bodies.len() == 1 && self.bodies[0].range == (0..0)
    }
    fn native_box_row(&self, owner: usize) -> Option<&BoxRow> {
        self.boxes
            .binary_search_by_key(&owner, |row| row.owner)
            .ok()
            .map(|index| &self.boxes[index])
    }
    fn box_row(&self, owner: usize) -> Result<&BoxRow> {
        self.native_box_row(owner)
            .ok_or(WrappingScopeError::InvalidGapAnchor)
    }
    fn raw_frontier(&self, anchor: Anchor, body: usize) -> Result<usize> {
        let frontier = |witness| -> Result<usize> {
            match witness {
                Witness::Start => Ok(self.bodies[body].range.start),
                Witness::End => Ok(self.bodies[body].range.end),
                Witness::Neighbor { rank, affinity } => {
                    let Role::Operand(level) = self.slots[rank].role else {
                        return Err(WrappingScopeError::InvalidGapAnchor);
                    };
                    let after = (affinity == InlineGapAffinity::After) != (level.get() % 2 == 1);
                    rank.checked_add(usize::from(after))
                        .ok_or(WrappingScopeError::InvalidGapAnchor)
                }
            }
        };
        match anchor {
            Anchor::Explicit(witness) => frontier(witness),
            Anchor::Derived {
                preceding,
                following,
            } => {
                let first = frontier(preceding)?;
                if first != frontier(following)? {
                    return Err(WrappingScopeError::ConflictingGap);
                }
                Ok(first)
            }
            Anchor::Parent(_) => Err(WrappingScopeError::UnsupportedIncidence),
        }
    }
    pub(crate) fn validate_slot(
        &self,
        rank: usize,
        structural: bool,
        level: BidiLevel,
    ) -> Result<usize> {
        let slot = self
            .slots
            .get(rank)
            .ok_or(WrappingScopeError::InconsistentIncidence)?;
        if if structural {
            !matches!(slot.role, Role::Structural | Role::Alias)
        } else {
            slot.role != Role::Operand(level)
        } {
            return Err(WrappingScopeError::InconsistentIncidence);
        }
        Ok(slot.body)
    }
    pub(crate) fn owns(&self, owner: usize, rank: usize) -> Result<bool> {
        let range = &self.box_row(owner)?.range;
        Ok(range.start < rank && rank + 1 < range.end)
    }
    pub(crate) fn tie_before(
        &self,
        a: usize,
        b: usize,
        body: usize,
        selection: &NativeOrderingSelection<'_>,
        payload: &[usize],
        gap: usize,
    ) -> Result<Option<bool>> {
        self.tie_before_using((a, b), body, selection, payload, gap, |owner, child| {
            if let Some(family) = &self.family {
                let a = family.scope_keys[owner].ok_or(WrappingScopeError::IncompleteFamily)?;
                let b = family.scope_keys[child].ok_or(WrappingScopeError::IncompleteFamily)?;
                return proof_ancestor(&family.proof.parents, family.root, a, b);
            }
            let a = &self.box_row(owner)?.range;
            let b = &self.box_row(child)?.range;
            Ok(a.start <= b.start && b.end <= a.end)
        })
    }

    fn tie_before_using(
        &self,
        (a, b): (usize, usize),
        body: usize,
        selection: &NativeOrderingSelection<'_>,
        payload: &[usize],
        gap: usize,
        contains: impl Fn(usize, usize) -> Result<bool>,
    ) -> Result<Option<bool>> {
        let mut result = None;
        for group in self.groups.iter().filter(|row| row.body == body) {
            let actual = match self.project_anchor(group.anchor, body, selection, payload) {
                Ok(actual) => actual,
                Err(WrappingScopeError::GapOutsideSelection) => continue,
                Err(error) => return Err(error),
            };
            if actual != gap {
                continue;
            }
            let mut aa = None;
            let mut bb = None;
            for (rank, &owner) in group.owners.iter().enumerate() {
                if contains(owner, a)? {
                    aa = Some(rank);
                }
                if contains(owner, b)? {
                    bb = Some(rank);
                }
            }
            if let (Some(aa), Some(bb)) = (aa, bb)
                && aa != bb
            {
                if contains(group.owners[aa], group.owners[bb])?
                    || contains(group.owners[bb], group.owners[aa])?
                {
                    continue;
                }
                let before = aa < bb;
                if result.is_some_and(|old| old != before) {
                    return Err(WrappingScopeError::ConflictingGap);
                }
                result = Some(before);
            }
        }
        Ok(result)
    }
    /// Prove a source-empty interval's unique physical gap from real adjacent
    /// operands, rather than treating a structural control as an embedding level.
    fn derive_gap(&self, range: Range<usize>, body: usize) -> Result<Anchor> {
        let actual = &self.bodies[body];
        if actual.prepared {
            return Err(WrappingScopeError::MissingGap);
        }
        let first = range.start.max(actual.range.start).min(actual.range.end);
        let after = range.end.min(actual.range.end).max(actual.range.start);
        let previous = if first == actual.range.start {
            None
        } else {
            let rank = first - 1;
            if matches!(self.slots[rank].role, Role::Operand(_)) {
                Some(rank)
            } else {
                self.slots[rank].previous
            }
        };
        let next = if after == actual.range.end {
            None
        } else {
            if matches!(self.slots[after].role, Role::Operand(_)) {
                Some(after)
            } else {
                self.slots[after].next
            }
        };
        let source_side = |rank: usize, after: bool| -> Result<(usize, Witness)> {
            let Role::Operand(level) = self.slots[rank].role else {
                return Err(WrappingScopeError::InconsistentIncidence);
            };
            let physical_after = after != (level.get() % 2 == 1);
            let position = self.slots[rank]
                .visual
                .ok_or(WrappingScopeError::InconsistentIncidence)?;
            Ok((
                position + usize::from(physical_after),
                Witness::Neighbor {
                    rank,
                    affinity: if physical_after {
                        InlineGapAffinity::After
                    } else {
                        InlineGapAffinity::Before
                    },
                },
            ))
        };
        let start = if self.direction == Direction::Ltr {
            0
        } else {
            actual.visual.len()
        };
        let end = actual.visual.len() - start;
        let (before, preceding) = previous
            .map(|rank| source_side(rank, true))
            .transpose()?
            .unwrap_or((start, Witness::Start));
        let (after, following) = next
            .map(|rank| source_side(rank, false))
            .transpose()?
            .unwrap_or((end, Witness::End));
        if before != after {
            return Err(WrappingScopeError::MissingGap);
        }
        Ok(Anchor::Derived {
            preceding,
            following,
        })
    }
    /// Evaluate the same primitive fact against the complete native body.
    fn witness_gap(&self, witness: Witness, body: usize) -> Result<usize> {
        let actual = &self.bodies[body];
        match witness {
            Witness::Neighbor { rank, affinity } => {
                let slot = self
                    .slots
                    .get(rank)
                    .ok_or(WrappingScopeError::InvalidGapAnchor)?;
                if slot.body != body {
                    return Err(WrappingScopeError::InvalidGapAnchor);
                }
                Ok(slot.visual.ok_or(WrappingScopeError::InvalidGapAnchor)?
                    + usize::from(affinity == InlineGapAffinity::After))
            }
            Witness::Start => Ok(if self.direction == Direction::Ltr {
                0
            } else {
                actual.visual.len()
            }),
            Witness::End => Ok(if self.direction == Direction::Ltr {
                actual.visual.len()
            } else {
                0
            }),
        }
    }
    fn anchor_gap(&self, anchor: Anchor, body: usize) -> Result<usize> {
        match anchor {
            Anchor::Explicit(witness) => self.witness_gap(witness, body),
            Anchor::Derived {
                preceding,
                following,
            } => {
                let before = self.witness_gap(preceding, body)?;
                let after = self.witness_gap(following, body)?;
                if before != after {
                    return Err(WrappingScopeError::ConflictingGap);
                }
                Ok(before)
            }
            Anchor::Parent(_) => Err(WrappingScopeError::CyclicGap),
        }
    }
    fn project_anchor(
        &self,
        anchor: Anchor,
        body: usize,
        selection: &NativeOrderingSelection<'_>,
        payload: &[usize],
    ) -> Result<usize> {
        match anchor {
            Anchor::Explicit(witness) => self.project_witness(witness, body, selection, payload),
            Anchor::Derived {
                preceding,
                following,
            } => {
                let before = self.project_witness(preceding, body, selection, payload);
                let after = self.project_witness(following, body, selection, payload);
                match (before, after) {
                    (Err(error), _) if error != WrappingScopeError::GapOutsideSelection => {
                        Err(error)
                    }
                    (_, Err(error)) if error != WrappingScopeError::GapOutsideSelection => {
                        Err(error)
                    }
                    (Ok(before), Ok(after)) => {
                        if before != after {
                            return Err(WrappingScopeError::ConflictingGap);
                        }
                        Ok(before)
                    }
                    (Ok(gap), _) | (_, Ok(gap)) => Ok(gap),
                    _ => Err(WrappingScopeError::GapOutsideSelection),
                }
            }
            Anchor::Parent(_) => Err(WrappingScopeError::CyclicGap),
        }
    }
    /// A public primitive anchor remains exact. Only a derived pair may use
    /// the other independently proven witness on a narrower selected cut.
    fn project_witness(
        &self,
        witness: Witness,
        body: usize,
        selection: &NativeOrderingSelection<'_>,
        payload: &[usize],
    ) -> Result<usize> {
        match witness {
            Witness::Neighbor { rank, affinity } => payload
                .iter()
                .position(|&unit| selection.ranks[unit] == rank)
                .map(|position| position + usize::from(affinity == InlineGapAffinity::After))
                .ok_or(WrappingScopeError::GapOutsideSelection),
            Witness::Start | Witness::End => {
                let actual = &self.bodies[body];
                let (selected, expected) = if matches!(witness, Witness::Start) {
                    (selection.raw_fences.start, actual.range.start)
                } else {
                    (selection.raw_fences.end, actual.range.end)
                };
                if selected != expected {
                    return Err(WrappingScopeError::GapOutsideSelection);
                }
                let left = matches!(witness, Witness::Start) == (self.direction == Direction::Ltr);
                Ok(if left { 0 } else { payload.len() })
            }
        }
    }
    pub(crate) fn gap(
        &self,
        owner: usize,
        body: usize,
        selection: &NativeOrderingSelection<'_>,
        payload: &[usize],
    ) -> Result<usize> {
        if let Some(group) = self
            .groups
            .iter()
            .find(|group| group.body == body && group.owners.contains(&owner))
        {
            return self.project_anchor(group.anchor, body, selection, payload);
        }
        let range = self.box_row(owner)?.range.clone();
        let actual = &self.bodies[body];
        // Actual owned content outside this candidate cannot be donated to an
        // endpoint-only piece on another selected cut.
        let first = (range.start + 1).max(actual.range.start);
        let after = (range.end - 1).min(actual.range.end);
        if first < after {
            let next = if matches!(self.slots[first].role, Role::Operand(_) | Role::Prepared) {
                Some(first)
            } else {
                self.slots[first].next
            };
            if next.is_some_and(|rank| rank < after) {
                return Err(WrappingScopeError::GapOutsideSelection);
            }
        }
        self.project_anchor(self.derive_gap(range, body)?, body, selection, payload)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum AliasPort {
    Neighbor {
        rank: usize,
        affinity: InlineGapAffinity,
    },
    Body {
        start: bool,
    },
}
#[derive(Debug, PartialEq)]
pub(crate) struct FamilyEmpty {
    row: usize,
    body: usize,
    owner: Option<usize>,
    // Original generated-owner preorder, used only for logical first/last.
    logical_owner_order: usize,
    anchor: Anchor,
    site: EmptySite,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum EmptySite {
    Endpoint(usize),
    BodyStart(usize),
    // No scope-native occurrence exists. The actual nominated/inferred
    // formatter frontier must qualify through project_anchor on a real run.
    Attached,
}
impl NativeOrderingData {
    fn full_gap(&self, owner: usize, body: usize) -> Result<(usize, Anchor)> {
        let anchor = if let Some(group) = self
            .groups
            .iter()
            .find(|g| g.body == body && g.owners.contains(&owner))
        {
            group.anchor
        } else {
            self.derive_gap(self.box_row(owner)?.range.clone(), body)?
        };
        Ok((self.anchor_gap(anchor, body)?, anchor))
    }
    fn build_family<Node: Copy + Eq, Tree: LayoutTree<Node = Node>>(
        &mut self,
        tree: &Tree,
        graph: &ValidationGraph<Node>,
        flow: Node,
        proof: &super::edge_plan::AdmittedGeneratedBoxProof<Node>,
        admission: super::family::Admission,
    ) -> Result<()> {
        use super::family::{NativeFamilyData, NativeFamilyRow};
        use surgeist_text::{
            InlineBoxComponentParticipation as P, SuppliedInlineBoxComponentRow,
            SuppliedInlineBoxComponents, SuppliedInlineBoxComponentsInput,
        };
        let fmt = graph
            .index(flow)
            .ok_or(WrappingScopeError::NodeNotReachable)?;
        let mut rows = buffer(self.slots.len())?;
        let mut input = buffer(self.slots.len())?;
        let mut memberships = Vec::new();
        let mut map = buffer(self.slots.len())?;
        map.resize(self.slots.len(), usize::MAX);
        let mut ordinal = 0;
        for (rank, slot) in self.slots.iter().enumerate() {
            let first = memberships.len();
            let content = matches!(slot.role, Role::Operand(_));
            if content {
                let node = graph.nodes[graph.children[fmt][rank]];
                let key = if let Some(key) = proof
                    .boxes
                    .iter()
                    .position(|b| b.box_ == GeneratedBoxRef::Node(node))
                {
                    proof.nearest(key, &admission)?
                } else {
                    if admission.scope_keys.iter().enumerate().any(|(owner, key)| {
                        key.is_some() && !self.boxes.iter().any(|b| b.owner == owner)
                    }) {
                        return Err(WrappingScopeError::UnsupportedIncidence);
                    }
                    let mut key = admission.root;
                    let mut width = usize::MAX;
                    for b in &self.boxes {
                        if b.range.start < rank && rank + 1 < b.range.end && b.range.len() < width {
                            key = admission.scope_keys[b.owner]
                                .ok_or(WrappingScopeError::IncompleteFamily)?;
                            width = b.range.len();
                        }
                    }
                    key
                };
                push(&mut memberships, key)?;
                map[rank] = rows.len();
                push(
                    &mut rows,
                    NativeFamilyRow::Operand {
                        rank,
                        ordinal,
                        body: slot.body,
                    },
                )?;
                ordinal += 1;
            } else {
                map[rank] = rows.len();
                push(&mut rows, NativeFamilyRow::Metadata)?;
            }
            push(
                &mut input,
                SuppliedInlineBoxComponentRow {
                    owners: first..memberships.len(),
                    participation: if content {
                        P::Participating
                    } else {
                        P::Metadata
                    },
                },
            )?;
            let _ = tree; // All native roles/levels were read once above, not from geometry.
        }
        let mut empties = Vec::new();
        let mut empty_gaps = Vec::new();
        for (body, actual) in self.bodies.iter().enumerate() {
            let before = empties.len();
            for b in &self.boxes {
                let first = (b.range.start + 1).max(actual.range.start);
                let after = (b.range.end - 1).min(actual.range.end);
                if b.range.start >= actual.range.end || actual.range.start >= b.range.end {
                    continue;
                }
                if (first..after).any(|r| matches!(self.slots[r].role, Role::Operand(_))) {
                    continue;
                }
                // A real descendant gap carries its ancestors. Do not invent
                // an extra empty row for the enclosing normal box itself.
                if self.boxes.iter().any(|c| {
                    c.owner != b.owner
                        && b.range.start < c.range.start
                        && c.range.end < b.range.end
                        && c.range.start < actual.range.end
                        && actual.range.start < c.range.end
                }) {
                    continue;
                }
                let (gap, anchor) = self.full_gap(b.owner, body)?;
                let key =
                    admission.scope_keys[b.owner].ok_or(WrappingScopeError::IncompleteFamily)?;
                let row = rows.len();
                let start = memberships.len();
                push(&mut memberships, key)?;
                push(&mut rows, NativeFamilyRow::Empty { owner: key, body })?;
                push(
                    &mut input,
                    SuppliedInlineBoxComponentRow {
                        owners: start..memberships.len(),
                        participation: P::Participating,
                    },
                )?;
                push(&mut empty_gaps, gap)?;
                push(
                    &mut empties,
                    FamilyEmpty {
                        row,
                        body,
                        owner: Some(b.owner),
                        logical_owner_order: admission
                            .active
                            .iter()
                            .position(|&owner| owner == key)
                            .ok_or(WrappingScopeError::IncompleteFamily)?,
                        anchor,
                        site: if actual.range.contains(&b.range.start) {
                            EmptySite::Endpoint(b.range.start)
                        } else {
                            EmptySite::BodyStart(actual.range.start)
                        },
                    },
                )?;
            }
            for (owner, key) in admission.scope_keys.iter().enumerate() {
                let Some(key) = *key else {
                    continue;
                };
                if self.native_box_row(owner).is_some() || !proof.captured_empty_leaf(key) {
                    continue;
                }
                let attachment = self
                    .groups
                    .iter()
                    .find(|group| group.body == body && group.owners.contains(&owner));
                let anchor = if let Some(group) = attachment {
                    group.anchor
                } else if self.unique_empty_raw_frontier() {
                    // Start and End are the same actual RAW frontier as well
                    // as the same ordering gap. Metadata-only is insufficient.
                    Anchor::Derived {
                        preceding: Witness::Start,
                        following: Witness::End,
                    }
                } else {
                    continue;
                };
                proof.validate_attachment(key, admission.root, self.raw_frontier(anchor, body)?)?;
                let row = rows.len();
                let start = memberships.len();
                push(&mut memberships, key)?;
                push(&mut rows, NativeFamilyRow::Empty { owner: key, body })?;
                push(
                    &mut input,
                    SuppliedInlineBoxComponentRow {
                        owners: start..memberships.len(),
                        participation: P::Participating,
                    },
                )?;
                push(&mut empty_gaps, self.anchor_gap(anchor, body)?)?;
                push(
                    &mut empties,
                    FamilyEmpty {
                        row,
                        body,
                        owner: Some(owner),
                        logical_owner_order: admission
                            .active
                            .iter()
                            .position(|&owner| owner == key)
                            .ok_or(WrappingScopeError::IncompleteFamily)?,
                        anchor,
                        site: EmptySite::Attached,
                    },
                )?;
            }
            if actual.visual.is_empty() && empties.len() == before {
                let row = rows.len();
                let start = memberships.len();
                push(&mut memberships, admission.root)?;
                push(
                    &mut rows,
                    NativeFamilyRow::Empty {
                        owner: admission.root,
                        body,
                    },
                )?;
                push(
                    &mut input,
                    SuppliedInlineBoxComponentRow {
                        owners: start..memberships.len(),
                        participation: P::Participating,
                    },
                )?;
                push(&mut empty_gaps, 0)?;
                push(
                    &mut empties,
                    FamilyEmpty {
                        row,
                        body,
                        owner: None,
                        logical_owner_order: 0,
                        anchor: Anchor::Derived {
                            preceding: Witness::Start,
                            following: Witness::End,
                        },
                        site: EmptySite::BodyStart(actual.range.start),
                    },
                )?;
            }
        }
        let mut order = buffer(rows.len())?;
        let mut bodies = buffer(self.bodies.len())?;
        let mut body_ports = buffer(self.bodies.len())?;
        let mut ports = buffer(self.slots.len())?;
        ports.resize(self.slots.len(), None);
        let mut all = buffer(self.slots.len())?;
        all.extend(0..self.slots.len());
        let mut group_bounds: Vec<Option<(usize, usize)>> = Vec::new();
        for (body, actual) in self.bodies.iter().enumerate() {
            let begin = order.len();
            let selection = NativeOrderingSelection {
                ranks: &all,
                raw_fences: actual.range.clone(),
            };
            for gap in 0..=actual.visual.len() {
                let mut tied = Vec::new();
                for (i, empty) in empties.iter().enumerate() {
                    if empty.body == body && empty_gaps[i] == gap {
                        push(&mut tied, i)?;
                    }
                }
                // The existing gap authority owns unrelated tie order; source
                // rank or metric magnitude cannot invent a physical order.
                for i in 1..tied.len() {
                    let mut j = i;
                    while j > 0 {
                        let a = &empties[tied[j - 1]];
                        let b = &empties[tied[j]];
                        let before = match (a.owner, b.owner) {
                            (Some(a), Some(b)) => self
                                .tie_before_using(
                                    (a, b),
                                    body,
                                    &selection,
                                    &actual.visual,
                                    gap,
                                    |a, b| {
                                        proof_ancestor(
                                            &proof.data.parents,
                                            admission.root,
                                            admission.scope_keys[a]
                                                .ok_or(WrappingScopeError::IncompleteFamily)?,
                                            admission.scope_keys[b]
                                                .ok_or(WrappingScopeError::IncompleteFamily)?,
                                        )
                                    },
                                )?
                                .ok_or(WrappingScopeError::AmbiguousGap)?,
                            _ => true,
                        };
                        if before {
                            break;
                        }
                        tied.swap(j - 1, j);
                        j -= 1;
                    }
                }
                if tied.len() > 1 {
                    // Insertion established each adjacent precedence. Certify
                    // every declared unrelated-owner constraint as well: an
                    // inverted nonadjacent constraint closes that chain into
                    // a cycle. Bounds describe actual descendant leaf rows,
                    // including constraints inherited from named ancestors.
                    for group in self
                        .groups
                        .iter()
                        .filter(|group| group.body == body && group.owners.len() > 1)
                    {
                        let actual_gap = match self.project_anchor(
                            group.anchor,
                            body,
                            &selection,
                            &actual.visual,
                        ) {
                            Ok(gap) => gap,
                            Err(WrappingScopeError::GapOutsideSelection) => continue,
                            Err(error) => return Err(error),
                        };
                        if actual_gap != gap {
                            continue;
                        }
                        // A pure ancestry chain supplies no unrelated-owner
                        // precedence and needs no bounds scratch or pair walk.
                        let mut deepest = admission.scope_keys[group.owners[0]]
                            .ok_or(WrappingScopeError::IncompleteFamily)?;
                        let mut constrained = false;
                        for &owner in &group.owners[1..] {
                            let key = admission.scope_keys[owner]
                                .ok_or(WrappingScopeError::IncompleteFamily)?;
                            if proof_ancestor(&proof.data.parents, admission.root, deepest, key)? {
                                deepest = key;
                            } else if !proof_ancestor(
                                &proof.data.parents,
                                admission.root,
                                key,
                                deepest,
                            )? {
                                constrained = true;
                                break;
                            }
                        }
                        if !constrained {
                            continue;
                        }
                        group_bounds.clear();
                        group_bounds
                            .try_reserve(group.owners.len())
                            .map_err(|_| WrappingScopeError::Allocation)?;
                        group_bounds.resize(group.owners.len(), None);
                        for (index, &owner) in group.owners.iter().enumerate() {
                            let key = admission.scope_keys[owner]
                                .ok_or(WrappingScopeError::IncompleteFamily)?;
                            for (position, &empty) in tied.iter().enumerate() {
                                let child = empties[empty]
                                    .owner
                                    .ok_or(WrappingScopeError::InconsistentIncidence)?;
                                let child = admission.scope_keys[child]
                                    .ok_or(WrappingScopeError::IncompleteFamily)?;
                                if proof_ancestor(&proof.data.parents, admission.root, key, child)?
                                {
                                    group_bounds[index] =
                                        Some(group_bounds[index].map_or(
                                            (position, position),
                                            |(first, last)| {
                                                (first.min(position), last.max(position))
                                            },
                                        ));
                                }
                            }
                        }
                        for (left, bounds) in group_bounds.iter().copied().enumerate() {
                            let Some((_, last)) = bounds else {
                                continue;
                            };
                            for (right, bounds) in
                                group_bounds.iter().copied().enumerate().skip(left + 1)
                            {
                                let Some((first, _)) = bounds else {
                                    continue;
                                };
                                if last < first {
                                    continue;
                                }
                                let a = admission.scope_keys[group.owners[left]]
                                    .ok_or(WrappingScopeError::IncompleteFamily)?;
                                let b = admission.scope_keys[group.owners[right]]
                                    .ok_or(WrappingScopeError::IncompleteFamily)?;
                                if !proof_ancestor(&proof.data.parents, admission.root, a, b)?
                                    && !proof_ancestor(&proof.data.parents, admission.root, b, a)?
                                {
                                    return Err(WrappingScopeError::CyclicGap);
                                }
                            }
                        }
                    }
                }
                for i in tied {
                    push(&mut order, empties[i].row)?;
                }
                if let Some(&rank) = actual.visual.get(gap) {
                    ports[rank] = Some((order.len(), order.len() + 1));
                    push(&mut order, map[rank])?;
                }
            }
            push(&mut body_ports, (begin, order.len()))?;
            for (rank, slot) in self.slots.iter().enumerate() {
                if slot.body == body && !matches!(slot.role, Role::Operand(_)) {
                    push(&mut order, map[rank])?;
                }
            }
            push(&mut bodies, begin..order.len())?;
        }
        let components = SuppliedInlineBoxComponents::try_new(SuppliedInlineBoxComponentsInput {
            root: admission.root,
            owners: &admission.active,
            parents: &proof.data.parents,
            rows: &input,
            memberships: &memberships,
            order: &order,
            bodies: &bodies,
        })
        .map_err(|error| {
            if error.code == surgeist_text::ErrorCode::InvalidTextProtocol {
                WrappingScopeError::InconsistentIncidence
            } else {
                WrappingScopeError::Allocation
            }
        })?;
        for &owner in &admission.active {
            if !components
                .components()
                .any(|component| component.owner() == owner)
            {
                return Err(WrappingScopeError::MissingGap);
            }
        }
        self.family = Some(NativeFamilyData {
            proof: Arc::clone(&proof.data),
            root: admission.root,
            scope_keys: admission.scope_keys,
            components,
            rows,
            row_for_rank: map,
            empties,
            ports,
            body_ports,
        });
        Ok(())
    }
    fn alias_anchor(&self, owner: usize, start: bool, body: usize) -> Result<Anchor> {
        let range = &self.box_row(owner)?.range;
        let actual = &self.bodies[body];
        let mut members = (range.start + 1..range.end - 1).filter(|&r| {
            self.slots[r].body == body && matches!(self.slots[r].role, Role::Operand(_))
        });
        let rank = if start {
            members.next()
        } else {
            members.next_back()
        };
        let Some(rank) = rank else {
            if self.family.is_some()
                && let Some(empty) = self.logical_empty(owner, start, body)?
            {
                return Ok(empty.anchor);
            }
            return self.full_gap(owner, body).map(|(_, a)| a);
        };
        let Role::Operand(level) = self.slots[rank].role else {
            return Err(WrappingScopeError::InconsistentIncidence);
        };
        let after = !start != (level.get() % 2 == 1);
        let primary = Witness::Neighbor {
            rank,
            affinity: if after {
                InlineGapAffinity::After
            } else {
                InlineGapAffinity::Before
            },
        };
        let alternative = if start {
            self.slots[range.start].previous.map(|r| (r, true))
        } else {
            self.slots[range.end - 1].next.map(|r| (r, false))
        };
        let other = if let Some((neighbor, source_after)) = alternative {
            if let Role::Operand(level) = self.slots[neighbor].role {
                let after = source_after != (level.get() % 2 == 1);
                Some(Witness::Neighbor {
                    rank: neighbor,
                    affinity: if after {
                        InlineGapAffinity::After
                    } else {
                        InlineGapAffinity::Before
                    },
                })
            } else {
                None
            }
        } else {
            Some(if start { Witness::Start } else { Witness::End })
        };
        if let Some(other) = other
            && self.same_alias_gap(primary, other, body)?
        {
            return Ok(Anchor::Derived {
                preceding: primary,
                following: other,
            });
        }
        let _ = actual;
        Ok(Anchor::Explicit(primary))
    }

    fn same_alias_gap(&self, a: Witness, b: Witness, body: usize) -> Result<bool> {
        let family = self
            .family
            .as_ref()
            .ok_or(WrappingScopeError::IncompleteFamily)?;
        let gap = |w: Witness| -> Result<usize> {
            match w {
                Witness::Neighbor { rank, affinity } => {
                    let ports = family.ports[rank].ok_or(WrappingScopeError::InvalidGapAnchor)?;
                    Ok(if affinity == InlineGapAffinity::Before {
                        ports.0
                    } else {
                        ports.1
                    })
                }
                Witness::Start => {
                    let p = family.body_ports[body];
                    Ok(if self.direction == Direction::Ltr {
                        p.0
                    } else {
                        p.1
                    })
                }
                Witness::End => {
                    let p = family.body_ports[body];
                    Ok(if self.direction == Direction::Ltr {
                        p.1
                    } else {
                        p.0
                    })
                }
            }
        };
        if gap(a)? != gap(b)? {
            return Ok(false);
        }
        let source = |w| match w {
            Witness::Neighbor { rank, .. } => rank,
            Witness::Start => self.bodies[body].range.start,
            Witness::End => self.bodies[body].range.end,
        };
        let (x, y) = (source(a), source(b));
        let first = x.min(y);
        let after = x.max(y);
        // An intervening physical operation is a distinct authenticated cut,
        // even if its current dimensions happen to be zero. Alias metadata is
        // transparent; real empty participating rows were counted above.
        Ok(!self.slots[first..after]
            .iter()
            .any(|slot| slot.role == Role::Structural))
    }
    fn selected_empty_gap(
        &self,
        empty: &FamilyEmpty,
        selection: &NativeOrderingSelection<'_>,
        payload: &[usize],
    ) -> Result<Option<usize>> {
        let site_present = match empty.site {
            EmptySite::Endpoint(rank) => selection.ranks.contains(&rank),
            EmptySite::BodyStart(rank) => selection.raw_fences.start == rank,
            EmptySite::Attached => true,
        };
        if !site_present {
            return Ok(None);
        }
        match self.project_anchor(empty.anchor, empty.body, selection, payload) {
            Ok(gap) => Ok(Some(gap)),
            Err(WrappingScopeError::GapOutsideSelection) => Ok(None),
            Err(error) => Err(error),
        }
    }
    fn empty_rows_for(&self, owner: usize, body: usize) -> Result<Vec<&FamilyEmpty>> {
        let family = self
            .family
            .as_ref()
            .ok_or(WrappingScopeError::IncompleteFamily)?;
        let key = family.scope_keys[owner].ok_or(WrappingScopeError::IncompleteFamily)?;
        let mut result = Vec::new();
        for empty in &family.empties {
            if empty.body == body
                && family.components.row_components(empty.row).any(|c| {
                    family
                        .components
                        .component(c)
                        .is_some_and(|c| c.owner() == key)
                })
            {
                push(&mut result, empty)?;
            }
        }
        Ok(result)
    }
    pub(crate) fn empty_extents(
        &self,
        owner: usize,
        body: usize,
        selection: &NativeOrderingSelection<'_>,
        payload: &[usize],
    ) -> Result<Option<(usize, usize)>> {
        if self.family.is_none() {
            return Ok(None);
        }
        let mut result = None;
        let owned = self.empty_rows_for(owner, body)?;
        let demanded = !owned.is_empty();
        for empty in owned {
            if let Some(gap) = self.selected_empty_gap(empty, selection, payload)? {
                result = Some(result.map_or((gap, gap), |(a, b): (usize, usize)| {
                    (a.min(gap), b.max(gap))
                }));
            }
        }
        if demanded && result.is_none() {
            return Err(WrappingScopeError::GapOutsideSelection);
        }
        Ok(result)
    }
    pub(crate) fn empty_component(
        &self,
        owner: usize,
        body: usize,
        selection: &NativeOrderingSelection<'_>,
        payload: &[usize],
        left: bool,
    ) -> Result<usize> {
        let family = self
            .family
            .as_ref()
            .ok_or(WrappingScopeError::IncompleteFamily)?;
        let key = family.scope_keys[owner].ok_or(WrappingScopeError::IncompleteFamily)?;
        let mut chosen = None;
        for empty in self.empty_rows_for(owner, body)? {
            if self
                .selected_empty_gap(empty, selection, payload)?
                .is_none()
            {
                continue;
            }
            for c in family.components.row_components(empty.row) {
                let component = family
                    .components
                    .component(c)
                    .ok_or(WrappingScopeError::InconsistentIncidence)?;
                if component.owner() == key
                    && chosen.is_none_or(|old| {
                        let prior = family
                            .components
                            .component(old)
                            .expect("retained component");
                        if left {
                            component.visual_range().start < prior.visual_range().start
                        } else {
                            component.visual_range().end > prior.visual_range().end
                        }
                    })
                {
                    chosen = Some(c);
                }
            }
        }
        chosen.ok_or(WrappingScopeError::GapOutsideSelection)
    }
    fn logical_empty(
        &self,
        owner: usize,
        start: bool,
        body: usize,
    ) -> Result<Option<&FamilyEmpty>> {
        let mut found = None;
        for empty in self.empty_rows_for(owner, body)? {
            if empty.owner.is_none() {
                continue;
            }
            if found.is_none_or(|old: &FamilyEmpty| {
                if start {
                    empty.logical_owner_order < old.logical_owner_order
                } else {
                    old.logical_owner_order < empty.logical_owner_order
                }
            }) {
                found = Some(empty);
            }
        }
        Ok(found)
    }
    pub(crate) fn component_for(
        &self,
        owner: usize,
        rank: Option<usize>,
        body: usize,
    ) -> Result<usize> {
        let family = self
            .family
            .as_ref()
            .ok_or(WrappingScopeError::IncompleteFamily)?;
        let key = family
            .scope_keys
            .get(owner)
            .copied()
            .flatten()
            .ok_or(WrappingScopeError::IncompleteFamily)?;
        if let Some(rank) = rank {
            return family
                .components
                .row_components(family.row_for_rank[rank])
                .find(|&c| {
                    family
                        .components
                        .component(c)
                        .is_some_and(|c| c.owner() == key && c.body() == body)
                })
                .ok_or(WrappingScopeError::InconsistentIncidence);
        }
        for empty in &family.empties {
            if empty.body == body
                && let Some(component) = family.components.row_components(empty.row).find(|&c| {
                    family
                        .components
                        .component(c)
                        .is_some_and(|c| c.owner() == key)
                })
            {
                return Ok(component);
            }
        }
        Err(WrappingScopeError::InconsistentIncidence)
    }
    pub(crate) fn alias_component(&self, owner: usize, start: bool, body: usize) -> Result<usize> {
        let r = &self.box_row(owner)?.range;
        let mut ranks = (r.start + 1..r.end - 1).filter(|&r| {
            self.slots[r].body == body && matches!(self.slots[r].role, Role::Operand(_))
        });
        let rank = if start {
            ranks.next()
        } else {
            ranks.next_back()
        };
        if rank.is_none()
            && let Some(empty) = self.logical_empty(owner, start, body)?
        {
            let family = self
                .family
                .as_ref()
                .ok_or(WrappingScopeError::IncompleteFamily)?;
            let key = family.scope_keys[owner].ok_or(WrappingScopeError::IncompleteFamily)?;
            return family
                .components
                .row_components(empty.row)
                .find(|&c| {
                    family
                        .components
                        .component(c)
                        .is_some_and(|c| c.owner() == key)
                })
                .ok_or(WrappingScopeError::InconsistentIncidence);
        }
        self.component_for(owner, rank, body)
    }
    pub(crate) fn alias_ports(
        &self,
        owner: usize,
        start: bool,
        body: usize,
        selection: &NativeOrderingSelection<'_>,
        payload: &[usize],
    ) -> Result<[Option<AliasPort>; 2]> {
        let anchor = self.alias_anchor(owner, start, body)?;
        self.project_anchor(anchor, body, selection, payload)?;
        let range = &self.box_row(owner)?.range;
        if !(range.start + 1..range.end - 1)
            .any(|r| self.slots[r].body == body && matches!(self.slots[r].role, Role::Operand(_)))
        {
            if let Some(empty) = self.logical_empty(owner, start, body)?
                && empty.owner != Some(owner)
                && self
                    .selected_empty_gap(empty, selection, payload)?
                    .is_none()
            {
                return Err(WrappingScopeError::GapOutsideSelection);
            }
            return Ok([None, None]);
        }
        let witnesses = match anchor {
            Anchor::Explicit(a) => [Some(a), None],
            Anchor::Derived {
                preceding,
                following,
            } => [Some(preceding), Some(following)],
            Anchor::Parent(_) => return Err(WrappingScopeError::CyclicGap),
        };
        let mut ports = [None, None];
        for (i, witness) in witnesses.into_iter().enumerate() {
            if let Some(witness) = witness {
                match self.project_witness(witness, body, selection, payload) {
                    Ok(_) => {
                        ports[i] = Some(match witness {
                            Witness::Neighbor { rank, affinity } => {
                                AliasPort::Neighbor { rank, affinity }
                            }
                            Witness::Start => AliasPort::Body { start: true },
                            Witness::End => AliasPort::Body { start: false },
                        });
                    }
                    Err(WrappingScopeError::GapOutsideSelection) => (),
                    Err(error) => return Err(error),
                }
            }
        }
        Ok(ports)
    }

    pub(crate) fn alias_gap(
        &self,
        owner: usize,
        start: bool,
        body: usize,
        selection: &NativeOrderingSelection<'_>,
        payload: &[usize],
    ) -> Result<usize> {
        self.project_anchor(
            self.alias_anchor(owner, start, body)?,
            body,
            selection,
            payload,
        )
    }
    pub(crate) fn selected_family_rows(
        &self,
        selection: &NativeOrderingSelection<'_>,
        payload: &[usize],
        body: usize,
    ) -> Result<Vec<usize>> {
        let mut result = Vec::new();
        let Some(family) = &self.family else {
            return Ok(result);
        };
        for &rank in selection.ranks {
            let row = family.row_for_rank[rank];
            if family.components.row_components(row).next().is_some() {
                push(&mut result, row)?;
            }
        }
        for empty in &family.empties {
            if empty.body == body
                && self
                    .selected_empty_gap(empty, selection, payload)?
                    .is_some()
            {
                push(&mut result, empty.row)?;
            }
        }
        Ok(result)
    }
}

/// The actual fitting run's native source ordinals, sharing its formatter owner.
#[derive(Debug)]
pub(crate) struct NativeOrderingProjection {
    pub(crate) profile: Arc<NativeOrderingData>,
    pub(crate) ranks: Vec<usize>,
    pub(crate) raw_range: Range<usize>,
}
impl PartialEq for NativeOrderingProjection {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.profile, &other.profile)
            && self.ranks == other.ranks
            && self.raw_range == other.raw_range
    }
}

/// The source slice and child ordinal issued by the real run transition.
/// Its source occurrence is checked against the same immutable graph before use.
#[derive(Clone, Copy)]
pub(crate) struct NativeRunSource<'a, Node> {
    pub(crate) nodes: &'a [Node],
    pub(crate) start: usize,
}
impl<Node: Copy + Eq> NativeRunSource<'_, Node> {
    pub(super) fn validate(
        self,
        graph: &ValidationGraph<Node>,
        owner: usize,
    ) -> Result<Range<usize>> {
        let after = self
            .start
            .checked_add(self.nodes.len())
            .ok_or(WrappingScopeError::InconsistentIncidence)?;
        let children = graph.children[owner]
            .get(self.start..after)
            .ok_or(WrappingScopeError::InconsistentIncidence)?;
        if children
            .iter()
            .zip(self.nodes)
            .any(|(&index, &node)| graph.nodes[index] != node)
        {
            return Err(WrappingScopeError::InconsistentIncidence);
        }
        Ok(self.start..after)
    }
}
/// Borrowed emitted incidence and raw source fences of one actual candidate.
/// Filtered metadata between emissions can be covered by both adjacent fences;
/// these are edge capabilities, not exclusive metadata ownership/readback.
pub(crate) struct NativeOrderingSelection<'a> {
    pub(crate) ranks: &'a [usize],
    pub(crate) raw_fences: Range<usize>,
}
impl NativeOrderingProjection {
    pub(crate) fn selection(&self, emitted: Range<usize>) -> Result<NativeOrderingSelection<'_>> {
        let ranks = self
            .ranks
            .get(emitted.clone())
            .ok_or(WrappingScopeError::InconsistentIncidence)?;
        let first = if emitted.start == 0 {
            self.raw_range.start
        } else {
            self.ranks[emitted.start - 1]
                .checked_add(1)
                .ok_or(WrappingScopeError::InconsistentIncidence)?
        };
        let after = if emitted.end == self.ranks.len() {
            self.raw_range.end
        } else {
            self.ranks[emitted.end]
        };
        if first < self.raw_range.start || self.raw_range.end < after || after < first {
            return Err(WrappingScopeError::InconsistentIncidence);
        }
        Ok(NativeOrderingSelection {
            ranks,
            raw_fences: first..after,
        })
    }
}
