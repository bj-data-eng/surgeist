use crate::{
    BoxFacts, BoxId, ChildOrder, PaintError, PaintOptions, PaintPhase, PaintReport, Participation,
    ResourceKind, StackingPlan, WorkKind,
};
use crate::{
    accounting::{
        Meter, OperationKind, PhaseMeter, allocate, apply, charge, finish, release, setup_error,
    },
    model::ContextRecord,
};
use std::{
    cmp::Ordering,
    mem::{size_of, take},
};
use surgeist_ledger::{Change, Limits, Scope, Tariffs};

/// Complete result and optional accepted-prefix report. A semantic/admission
/// failure contains no partial plan; accepted work remains observable.
#[derive(Debug)]
pub struct PaintOutcome<'a> {
    pub result: Result<StackingPlan<'a>, PaintError>,
    pub report: Option<PaintReport>,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct NodeScratch {
    count: usize,
    pub(crate) start: usize,
    pub(crate) end: usize,
    cursor: usize,
    pub(crate) rank: usize,
    color: u8,
}
#[derive(Default)]
pub(crate) struct State {
    pub(crate) nodes: Vec<NodeScratch>,
    pub(crate) edges: Vec<usize>,
    pub(crate) merge: Vec<usize>,
    pub(crate) stack: Vec<usize>,
    pub(crate) membership: Vec<usize>,
    pub(crate) contexts: Vec<ContextRecord>,
    pub(crate) children: Vec<usize>,
}
impl State {
    pub(crate) fn scratch_cleanup(
        &mut self,
        meter: &mut PhaseMeter<'_, '_>,
        root: Scope,
    ) -> Result<(), PaintError> {
        release(
            meter,
            root,
            take(&mut self.nodes),
            ResourceKind::ScratchBytes,
        )?;
        release(
            meter,
            root,
            take(&mut self.edges),
            ResourceKind::ScratchBytes,
        )?;
        release(
            meter,
            root,
            take(&mut self.merge),
            ResourceKind::ScratchBytes,
        )?;
        release(
            meter,
            root,
            take(&mut self.stack),
            ResourceKind::ScratchBytes,
        )?;
        Ok(())
    }
    pub(crate) fn result_cleanup(
        &mut self,
        meter: &mut PhaseMeter<'_, '_>,
        root: Scope,
    ) -> Result<(), PaintError> {
        release(
            meter,
            root,
            take(&mut self.membership),
            ResourceKind::RetainedResultBytes,
        )?;
        release(
            meter,
            root,
            take(&mut self.contexts),
            ResourceKind::RetainedResultBytes,
        )?;
        release(
            meter,
            root,
            take(&mut self.children),
            ResourceKind::RetainedResultBytes,
        )?;
        Ok(())
    }
}

/// Plans actual stacking-context membership/order from resolved rendering boxes.
/// Parent references may point forward. Input slice order is never a tie breaker.
/// The result deliberately does not emit the complete Appendix E paint sequence.
pub fn plan_stacking(
    boxes: &[BoxFacts],
    root_box: BoxId,
    options: PaintOptions,
) -> PaintOutcome<'_> {
    let mut meter = match options.operation(OperationKind::Stacking) {
        Ok(meter) => meter,
        Err(error) => {
            return PaintOutcome {
                result: Err(error),
                report: None,
            };
        }
    };
    let root = meter.root();
    let scopes = setup_scopes(&mut meter);
    let scopes = match scopes {
        Ok(scopes) => scopes,
        Err(error) => {
            let (result, report) = finish(meter, Err(error));
            return PaintOutcome { result, report };
        }
    };
    let mut state = State::default();
    let result = run(&mut meter, root, &scopes, &mut state, boxes, root_box);
    // Cleanup also runs after every semantic/admission refusal. It never undoes work.
    let cleanup = meter
        .with_phase(scopes[6], |phase| {
            state.scratch_cleanup(phase, root)?;
            if result.is_err() {
                state.result_cleanup(phase, root)?;
            }
            Ok(())
        })
        .map_err(setup_error)
        .and_then(|done| done);
    let result = result.and_then(|value| cleanup.map(|()| value));
    let (result, report) = finish(meter, result);
    PaintOutcome { result, report }
}

pub(crate) fn setup_scopes(meter: &mut Meter) -> Result<[Scope; 7], PaintError> {
    let mut scopes = [meter.root(); 7];
    for (slot, phase) in scopes.iter_mut().zip([
        PaintPhase::Validate,
        PaintPhase::Topology,
        PaintPhase::SiblingOrder,
        PaintPhase::Membership,
        PaintPhase::ContextOrder,
        PaintPhase::Publication,
        PaintPhase::Cleanup,
    ]) {
        *slot = meter.child(
            meter.root(),
            phase,
            Limits::unlimited(),
            Tariffs::identity(),
        )?;
    }
    Ok(scopes)
}

fn run<'a>(
    meter: &mut Meter,
    root: Scope,
    scopes: &[Scope; 7],
    state: &mut State,
    boxes: &'a [BoxFacts],
    root_box: BoxId,
) -> Result<StackingPlan<'a>, PaintError> {
    prepare(meter, root, scopes, state, boxes, root_box)?;
    meter
        .with_phase(scopes[5], |p| {
            // Construct the actual fixed owner before acquiring its custody. No
            // allocation can fail here; output backing was admitted when allocated.
            let plan = StackingPlan {
                facts: boxes,
                membership: take(&mut state.membership),
                contexts: take(&mut state.contexts),
                children: take(&mut state.children),
            };
            let admitted = apply(
                p,
                root,
                Change::Acquire(
                    ResourceKind::RetainedResultBytes,
                    u64::try_from(size_of::<StackingPlan<'_>>())
                        .map_err(|_| PaintError::CapacityOverflow)?,
                ),
            );
            match admitted {
                Ok(()) => Ok(plan),
                Err(error) => {
                    // Return backing to the operation cleanup owner before dropping
                    // this unpublished fixed value; no accepted fixed-owner charge.
                    state.membership = plan.membership;
                    state.contexts = plan.contexts;
                    state.children = plan.children;
                    Err(error)
                }
            }
        })
        .map_err(setup_error)?
}

/// The shared semantic core leaves its admitted state with the calling operation.
/// Both public planners own publication and cleanup in their one Ledger domain.
pub(crate) fn prepare(
    meter: &mut Meter,
    root: Scope,
    scopes: &[Scope; 7],
    state: &mut State,
    boxes: &[BoxFacts],
    root_box: BoxId,
) -> Result<(), PaintError> {
    meter
        .with_phase(scopes[0], |p| validate(p, scopes[0], boxes, root_box))
        .map_err(setup_error)??;
    meter
        .with_phase(scopes[1], |p| topology(p, root, scopes[1], state, boxes))
        .map_err(setup_error)??;
    meter
        .with_phase(scopes[2], |p| order_siblings(p, scopes[2], state, boxes))
        .map_err(setup_error)??;
    meter
        .with_phase(scopes[3], |p| {
            membership(p, root, scopes[3], state, boxes, root_box)
        })
        .map_err(setup_error)??;
    meter
        .with_phase(scopes[4], |p| order_contexts(p, root, scopes[4], state))
        .map_err(setup_error)??;
    Ok(())
}

fn validate(
    p: &mut PhaseMeter<'_, '_>,
    scope: Scope,
    boxes: &[BoxFacts],
    root: BoxId,
) -> Result<(), PaintError> {
    if root.get() >= boxes.len() {
        return Err(PaintError::MissingRoot);
    }
    let mut found_root = None;
    for (i, facts) in boxes.iter().enumerate() {
        charge(p, scope, WorkKind::BoxValidation, 1)?;
        let id = BoxId::new(i);
        match facts.parent {
            Some(parent) => {
                if parent.get() >= boxes.len() {
                    return Err(PaintError::MissingParent { child: id, parent });
                }
                if parent == id {
                    return Err(PaintError::SelfParent { box_id: id });
                }
                let valid = match (boxes[parent.get()].child_order, facts.participation) {
                    (_, Participation::Ordinary) if facts.position.out_of_flow() => true,
                    (ChildOrder::Source, Participation::Ordinary) => true,
                    (ChildOrder::Flex, Participation::FlexItem(_))
                    | (ChildOrder::Grid, Participation::GridItem(_)) => {
                        !facts.position.out_of_flow()
                    }
                    _ => false,
                };
                if !valid {
                    return Err(PaintError::IncompatibleParticipation { box_id: id });
                }
            }
            None => {
                if let Some(first) = found_root {
                    return Err(PaintError::MultipleRoots { first, other: id });
                }
                found_root = Some(id);
                if facts.participation != Participation::Ordinary {
                    return Err(PaintError::IncompatibleParticipation { box_id: id });
                }
            }
        }
    }
    if found_root != Some(root) {
        return Err(PaintError::MissingRoot);
    }
    Ok(())
}

fn topology(
    p: &mut PhaseMeter<'_, '_>,
    root: Scope,
    scope: Scope,
    s: &mut State,
    boxes: &[BoxFacts],
) -> Result<(), PaintError> {
    let n = boxes.len();
    s.nodes = allocate(p, root, n, ResourceKind::ScratchBytes)?;
    charge(p, scope, WorkKind::TopologyVisit, n)?;
    s.nodes.resize(n, NodeScratch::default());
    s.stack = allocate(p, root, n, ResourceKind::ScratchBytes)?;
    s.edges = allocate(p, root, n - 1, ResourceKind::ScratchBytes)?;
    charge(p, scope, WorkKind::TopologyVisit, n - 1)?;
    s.edges.resize(n - 1, 0);
    s.merge = allocate(p, root, n, ResourceKind::ScratchBytes)?;
    charge(p, scope, WorkKind::TopologyVisit, n)?;
    s.merge.resize(n, 0);
    // Parent walks inspect every component, including disconnected cycles.
    // A finite functional parent graph with exactly one root can be disconnected
    // only if another component contains a cycle; Cycle is the precise diagnostic.
    for start in 0..n {
        charge(p, scope, WorkKind::TopologyVisit, 1)?;
        if s.nodes[start].color != 0 {
            continue;
        }
        let mut cursor = Some(start);
        while let Some(i) = cursor {
            charge(p, scope, WorkKind::TopologyVisit, 1)?;
            match s.nodes[i].color {
                1 => {
                    return Err(PaintError::Cycle {
                        box_id: BoxId::new(i),
                    });
                }
                2 => break,
                _ => {
                    s.nodes[i].color = 1;
                    s.stack.push(i);
                    cursor = boxes[i].parent.map(BoxId::get);
                }
            }
        }
        while !s.stack.is_empty() {
            charge(p, scope, WorkKind::TopologyVisit, 1)?;
            let i = s.stack.pop().expect("nonempty traversal stack");
            s.nodes[i].color = 2;
        }
    }
    for facts in boxes {
        charge(p, scope, WorkKind::TopologyVisit, 1)?;
        if let Some(parent) = facts.parent {
            charge(p, scope, WorkKind::ParentEdge, 1)?;
            s.nodes[parent.get()].count += 1;
        }
    }
    let mut start = 0;
    for node in &mut s.nodes {
        charge(p, scope, WorkKind::TopologyVisit, 1)?;
        node.start = start;
        start += node.count;
        node.end = start;
        node.cursor = node.start;
    }
    for (i, facts) in boxes.iter().enumerate() {
        charge(p, scope, WorkKind::TopologyVisit, 1)?;
        if let Some(parent) = facts.parent {
            let node = &mut s.nodes[parent.get()];
            s.edges[node.cursor] = i;
            node.cursor += 1;
        }
    }
    Ok(())
}

/// Stable bottom-up merge sort with fallible admission before every comparator
/// and every index move. No comparator substitutes Equal after a refusal.
pub(crate) fn sort(
    p: &mut PhaseMeter<'_, '_>,
    scope: Scope,
    values: &mut [usize],
    scratch: &mut [usize],
    comparisons: WorkKind,
    compare: impl Fn(usize, usize) -> Ordering,
) -> Result<(), PaintError> {
    let n = values.len();
    let mut width = 1;
    while width < n {
        let mut begin = 0;
        while begin < n {
            let middle = begin.saturating_add(width).min(n);
            let end = middle.saturating_add(width).min(n);
            let (mut left, mut right) = (begin, middle);
            for target in &mut scratch[begin..end] {
                let take_left = if left == middle {
                    false
                } else if right == end {
                    true
                } else {
                    charge(p, scope, comparisons, 1)?;
                    compare(values[left], values[right]) != Ordering::Greater
                };
                charge(p, scope, WorkKind::OrderMove, 1)?;
                *target = if take_left {
                    let value = values[left];
                    left += 1;
                    value
                } else {
                    let value = values[right];
                    right += 1;
                    value
                };
            }
            begin = end;
        }
        for (target, &value) in values.iter_mut().zip(&scratch[..n]) {
            charge(p, scope, WorkKind::OrderMove, 1)?;
            *target = value;
        }
        width = width.saturating_mul(2);
    }
    Ok(())
}

fn order_siblings(
    p: &mut PhaseMeter<'_, '_>,
    scope: Scope,
    s: &mut State,
    boxes: &[BoxFacts],
) -> Result<(), PaintError> {
    for (i, node) in s.nodes.iter().enumerate() {
        charge(p, scope, WorkKind::TopologyVisit, 1)?;
        let children = &mut s.edges[node.start..node.end];
        sort(
            p,
            scope,
            children,
            &mut s.merge,
            WorkKind::SiblingComparison,
            |a, b| boxes[a].source_index.cmp(&boxes[b].source_index),
        )?;
        for pair in children.windows(2) {
            charge(p, scope, WorkKind::SiblingComparison, 1)?;
            if boxes[pair[0]].source_index == boxes[pair[1]].source_index {
                return Err(PaintError::DuplicateSourceIndex {
                    parent: BoxId::new(i),
                    source_index: boxes[pair[0]].source_index,
                });
            }
        }
        if boxes[i].child_order != ChildOrder::Source {
            sort(
                p,
                scope,
                children,
                &mut s.merge,
                WorkKind::SiblingComparison,
                |a, b| {
                    (boxes[a].effective_order(), boxes[a].source_index)
                        .cmp(&(boxes[b].effective_order(), boxes[b].source_index))
                },
            )?;
        }
    }
    Ok(())
}

fn membership(
    p: &mut PhaseMeter<'_, '_>,
    root: Scope,
    scope: Scope,
    s: &mut State,
    boxes: &[BoxFacts],
    root_box: BoxId,
) -> Result<(), PaintError> {
    let n = boxes.len();
    s.membership = allocate(p, root, n, ResourceKind::RetainedResultBytes)?;
    charge(p, scope, WorkKind::OutputRecord, n)?;
    s.membership.resize(n, 0);
    s.contexts = allocate(p, root, n, ResourceKind::RetainedResultBytes)?;
    s.children = allocate(p, root, n - 1, ResourceKind::RetainedResultBytes)?;
    charge(p, scope, WorkKind::TopologyVisit, 1)?;
    s.stack.push(root_box.get());
    let mut rank = 0;
    while !s.stack.is_empty() {
        charge(p, scope, WorkKind::TopologyVisit, 1)?;
        let i = s.stack.pop().expect("nonempty traversal stack");
        charge(p, scope, WorkKind::MembershipVisit, 1)?;
        s.nodes[i].rank = rank;
        rank += 1;
        let facts = boxes[i];
        let parent_context = facts.parent.map(|parent| s.membership[parent.get()]);
        let (creates, level) = facts.classification();
        let context = if let (false, Some(parent)) = (creates, parent_context) {
            parent
        } else {
            let index = s.contexts.len();
            charge(p, scope, WorkKind::OutputRecord, 1)?;
            s.contexts.push(ContextRecord {
                box_id: BoxId::new(i),
                parent: parent_context,
                level: if parent_context.is_none() { 0 } else { level },
                start: 0,
                end: 0,
            });
            index
        };
        charge(p, scope, WorkKind::OutputRecord, 1)?;
        s.membership[i] = context;
        let node = s.nodes[i];
        for &child in s.edges[node.start..node.end].iter().rev() {
            charge(p, scope, WorkKind::TopologyVisit, 1)?;
            s.stack.push(child);
        }
    }
    Ok(())
}

fn order_contexts(
    p: &mut PhaseMeter<'_, '_>,
    _root: Scope,
    scope: Scope,
    s: &mut State,
) -> Result<(), PaintError> {
    for i in 1..s.contexts.len() {
        charge(p, scope, WorkKind::OutputRecord, 1)?;
        s.children.push(i);
    }
    sort(
        p,
        scope,
        &mut s.children,
        &mut s.merge,
        WorkKind::ContextComparison,
        |a, b| {
            let a = s.contexts[a];
            let b = s.contexts[b];
            (a.parent, a.level, s.nodes[a.box_id.get()].rank).cmp(&(
                b.parent,
                b.level,
                s.nodes[b.box_id.get()].rank,
            ))
        },
    )?;
    // Contexts were created in preorder, hence parent IDs increase over groups.
    let mut cursor = 0;
    for i in 0..s.contexts.len() {
        charge(p, scope, WorkKind::TopologyVisit, 1)?;
        let start = cursor;
        while cursor < s.children.len() {
            charge(p, scope, WorkKind::TopologyVisit, 1)?;
            if s.contexts[s.children[cursor]].parent != Some(i) {
                break;
            }
            cursor += 1;
        }
        charge(p, scope, WorkKind::OutputRecord, 1)?;
        s.contexts[i].start = start;
        s.contexts[i].end = cursor;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
