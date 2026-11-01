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
    members: Option<(usize, usize)>,
    visual: Vec<usize>,
    prepared: bool,
}
#[derive(Clone, Copy, Debug)]
enum Witness {
    Neighbor {
        rank: usize,
        affinity: InlineGapAffinity,
    },
    Start,
    End,
}
#[derive(Clone, Copy, Debug)]
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
            if row.parent_edge_axes.is_none() || !flows.contains(&row.flow) {
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
                        members: None,
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
                members: None,
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
                if matches!(
                    slots[rank].role,
                    Role::Operand(_) | Role::Structural | Role::Prepared
                ) {
                    actual.members = Some(
                        actual
                            .members
                            .map_or((rank, rank), |(first, _)| (first, rank)),
                    );
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
        for input in gaps {
            let first_owner = resolve_box(&input.boxes[0])?;
            if rows[first_owner].flow != flow {
                continue;
            }
            let mut owners = buffer(input.boxes.len())?;
            for reference in &input.boxes {
                let owner = resolve_box(reference)?;
                if rows[owner].flow != flow || owners.contains(&owner) {
                    return Err(WrappingScopeError::ConflictingGap);
                }
                owners.push(owner);
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
                let row = data.box_row(owner)?;
                let range = &data.bodies[body].range;
                if row.range.start >= range.end || range.start >= row.range.end {
                    return Err(WrappingScopeError::InvalidGapAnchor);
                }
                if data
                    .groups
                    .iter()
                    .any(|group| group.body == body && group.owners.contains(&owner))
                {
                    return Err(WrappingScopeError::ConflictingGap);
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
                first = first.min(data.box_row(owner)?.range.start);
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
                let range = &data.box_row(owner)?.range;
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
            // admitted ancestor and its descendant in the same group.
            for (index, &owner) in group.owners.iter().enumerate() {
                let parent = &data.box_row(owner)?.range;
                for (child_index, &child) in group.owners.iter().enumerate() {
                    let descendant = &data.box_row(child)?.range;
                    if owner == child
                        || !(parent.start < descendant.start && descendant.end < parent.end)
                    {
                        continue;
                    }
                    for &other in &group.owners[index.min(child_index)..index.max(child_index)] {
                        let actual = &data.box_row(other)?.range;
                        if actual.end <= parent.start || parent.end <= actual.start {
                            return Err(WrappingScopeError::ConflictingGap);
                        }
                    }
                }
            }
        }
        profiles.push(NativeOrderingProfile {
            flow,
            data: Arc::new(data),
        });
    }
    Ok(profiles)
}
impl NativeOrderingData {
    fn box_row(&self, owner: usize) -> Result<&BoxRow> {
        let index = self
            .boxes
            .binary_search_by_key(&owner, |row| row.owner)
            .map_err(|_| WrappingScopeError::InvalidGapAnchor)?;
        Ok(&self.boxes[index])
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
            slot.role != Role::Structural
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
        ranks: &[usize],
        payload: &[usize],
        gap: usize,
    ) -> Result<Option<bool>> {
        let contains = |owner: usize, child: usize| -> Result<bool> {
            let a = &self.box_row(owner)?.range;
            let b = &self.box_row(child)?.range;
            Ok(a.start <= b.start && b.end <= a.end)
        };
        let mut result = None;
        for group in self.groups.iter().filter(|row| row.body == body) {
            let actual = match self.project_anchor(group.anchor, body, ranks, payload) {
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
        ranks: &[usize],
        payload: &[usize],
    ) -> Result<usize> {
        match anchor {
            Anchor::Explicit(witness) => self.project_witness(witness, body, ranks, payload),
            Anchor::Derived {
                preceding,
                following,
            } => {
                let before = self.project_witness(preceding, body, ranks, payload);
                let after = self.project_witness(following, body, ranks, payload);
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
        ranks: &[usize],
        payload: &[usize],
    ) -> Result<usize> {
        match witness {
            Witness::Neighbor { rank, affinity } => payload
                .iter()
                .position(|&unit| ranks[unit] == rank)
                .map(|position| position + usize::from(affinity == InlineGapAffinity::After))
                .ok_or(WrappingScopeError::GapOutsideSelection),
            Witness::Start | Witness::End => {
                let actual = &self.bodies[body];
                let selected = match witness {
                    Witness::Start => ranks.first(),
                    _ => ranks.last(),
                };
                let expected = actual.members.map(|(first, last)| {
                    if matches!(witness, Witness::Start) {
                        first
                    } else {
                        last
                    }
                });
                if selected.copied() != expected {
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
        ranks: &[usize],
        payload: &[usize],
    ) -> Result<usize> {
        if let Some(group) = self
            .groups
            .iter()
            .find(|group| group.body == body && group.owners.contains(&owner))
        {
            return self.project_anchor(group.anchor, body, ranks, payload);
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
        self.project_anchor(self.derive_gap(range, body)?, body, ranks, payload)
    }
}

/// The actual fitting run's native source ordinals, sharing its formatter owner.
#[derive(Debug)]
pub(crate) struct NativeOrderingProjection {
    pub(crate) profile: Arc<NativeOrderingData>,
    pub(crate) ranks: Vec<usize>,
}
impl PartialEq for NativeOrderingProjection {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.profile, &other.profile) && self.ranks == other.ranks
    }
}
