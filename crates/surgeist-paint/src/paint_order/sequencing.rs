use super::{model::*, validation::PaintIndex};
use crate::{
    AccountingFailure, BoxId, PaintError, Participation, Position, ResourceKind, WorkKind,
    accounting::{PhaseMeter, allocate, charge, release},
    algorithm,
};
use std::{mem::take, ops::Range};
use surgeist_ledger::Scope;

fn invariant() -> PaintError {
    PaintError::Accounting(AccountingFailure::Invariant)
}

#[cfg(test)]
mod pending_tests {
    use super::*;
    use crate::{
        BoxFacts, ChildOrder, ContextEffect, PaintChange, PaintLimits, PaintOptions, PaintPhase,
        ProfileMode, ZIndex, accounting::OperationKind, paint_order::validation,
    };
    use surgeist_layout::SourceIndex;
    use surgeist_ledger::{Limits, Tariffs};

    #[test]
    fn refused_first_task_keeps_pending_scope_unstarted_until_cleanup_and_retry() {
        let boxes = [BoxFacts::new(
            None,
            SourceIndex::new(0),
            ChildOrder::Source,
            Participation::Ordinary,
            Position::Static,
            ZIndex::Auto,
            ContextEffect::None,
        )];
        let paint = [BoxPaint::new(BoxPaintKind::Block, FloatParticipation::None)];
        let request = PaintRequest::new(&boxes, &paint, &[], &[], &[], &[], BoxId::new(0));
        // Prepare visits rank, scope, children and the sole member (4).
        // The initial Scope push is the fifth visit; consuming it is refused.
        let options = PaintOptions::new(
            PaintLimits::unlimited().with_work_limit(WorkKind::PaintScopeVisit, 5),
            ProfileMode::Summary { timing: false },
        );
        let mut meter = options.operation(OperationKind::PaintOrder).unwrap();
        let root = meter.root();
        let scopes = algorithm::setup_scopes(&mut meter).unwrap();
        let validate = meter
            .child(
                root,
                PaintPhase::PaintValidation,
                Limits::unlimited(),
                Tariffs::identity(),
            )
            .unwrap();
        let sequence = meter
            .child(
                root,
                PaintPhase::PaintSequence,
                Limits::unlimited(),
                Tariffs::identity(),
            )
            .unwrap();
        let mut core = algorithm::State::default();
        algorithm::prepare(&mut meter, root, &scopes, &mut core, &boxes, request.root()).unwrap();
        let mut index = meter
            .with_phase(validate, |p| {
                validation::prepare(request, &core, p, root, validate)
            })
            .unwrap()
            .unwrap();
        let mut state = SequenceState::default();
        let error = meter
            .with_phase(sequence, |p| {
                run(request, &core, &index, &mut state, p, root, sequence)
            })
            .unwrap()
            .unwrap_err();
        let PaintError::Budget(refusal) = error else {
            panic!("expected task admission refusal");
        };
        assert_eq!(refusal.origin, PaintPhase::PaintSequence);
        assert_eq!(refusal.current, 5);
        assert_eq!(
            refusal.change,
            PaintChange::Work {
                kind: WorkKind::PaintScopeVisit,
                amount: 1
            }
        );
        assert!(
            matches!(state.tasks.as_slice(), [Task::Scope(Invocation { box_id, limit: None })] if *box_id == request.root())
        );
        assert!(state.tasks.capacity() >= 1);
        assert!(state.events.is_empty());
        assert!(state.events.capacity() >= 2);
        assert_eq!(state.context_state, [0]);
        meter
            .with_phase(scopes[6], |p| {
                core.scratch_cleanup(p, root).unwrap();
                index.cleanup(p, root).unwrap();
                state.scratch_cleanup(p, root).unwrap();
                core.result_cleanup(p, root).unwrap();
                state.result_cleanup(p, root).unwrap();
            })
            .unwrap();
        let report = meter.finish().unwrap().unwrap();
        assert_eq!(report.work(WorkKind::PaintScopeVisit), 5);
        for &resource in ResourceKind::ALL {
            assert!(report.peak(resource) > 0);
            assert_eq!(report.live(resource), 0);
        }
        let retry = crate::plan_paint_order(request, options.with_limits(PaintLimits::unlimited()));
        let plan = retry.result.unwrap();
        let events: Vec<_> = plan.events().collect();
        assert!(
            matches!(events.as_slice(), [PaintOrderEvent::BeginContext(begin), PaintOrderEvent::EndContext(end)] if begin.id() == request.root() && end.id() == request.root())
        );
    }
}

#[cfg(test)]
mod growth_tests {
    use super::*;
    use crate::{PaintLimits, PaintOptions, PaintReport, ProfileMode, accounting::OperationKind};
    use std::mem::size_of;

    fn exercise(work_limit: u64, resource_limit: u64) -> (bool, u64, u64, PaintReport) {
        let limits = PaintLimits::unlimited()
            .with_work_limit(WorkKind::OrderMove, work_limit)
            .with_resource_limit(ResourceKind::ScratchBytes, resource_limit);
        let mut meter = PaintOptions::new(limits, ProfileMode::Summary { timing: false })
            .operation(OperationKind::PaintOrder)
            .unwrap();
        let root = meter.root();
        let (accepted, old_bytes, new_bytes) = meter
            .with_phase(root, |p| {
                let mut values = allocate::<u64>(p, root, 3, ResourceKind::ScratchBytes).unwrap();
                values.extend_from_slice(&[17, 23, 41]);
                let old_capacity = values.capacity();
                let old_bytes = (old_capacity * size_of::<u64>()) as u64;
                let result = capacity(&mut values, old_capacity + 1, p, root, root);
                assert_eq!(values, [17, 23, 41]);
                let accepted = result.is_ok();
                if !accepted {
                    assert!(matches!(result, Err(PaintError::Budget(_))));
                    assert_eq!(values.capacity(), old_capacity);
                }
                let new_bytes = (values.capacity() * size_of::<u64>()) as u64;
                release(p, root, values, ResourceKind::ScratchBytes).unwrap();
                (accepted, old_bytes, new_bytes)
            })
            .unwrap();
        let report = meter.finish().unwrap().unwrap();
        assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
        assert_eq!(report.live(ResourceKind::RetainedResultBytes), 0);
        (accepted, old_bytes, new_bytes, report)
    }

    #[test]
    fn growth_admits_both_real_backings_until_copy_and_releases_old_capacity() {
        let (accepted, old, new, report) = exercise(u64::MAX, u64::MAX);
        assert!(accepted);
        assert!(new > old);
        assert_eq!(report.work(WorkKind::OrderMove), 3);
        assert_eq!(report.peak(ResourceKind::ScratchBytes), old + new);
        let (accepted, same_old, same_new, exact) = exercise(3, old + new);
        assert!(accepted);
        assert_eq!((same_old, same_new), (old, new));
        assert_eq!(exact.peak(ResourceKind::ScratchBytes), old + new);
    }

    #[test]
    fn copy_refusal_preserves_old_values_and_releases_new_backing() {
        let (_, old, new, _) = exercise(u64::MAX, u64::MAX);
        let (accepted, retained, unchanged, report) = exercise(2, u64::MAX);
        assert!(!accepted);
        assert_eq!((retained, unchanged), (old, old));
        assert_eq!(report.work(WorkKind::OrderMove), 0);
        assert_eq!(report.peak(ResourceKind::ScratchBytes), old + new);
    }

    #[test]
    fn backing_preview_refusal_leaves_old_values_without_artificial_overlap() {
        let (_, old, _, _) = exercise(u64::MAX, u64::MAX);
        let (accepted, retained, unchanged, report) = exercise(u64::MAX, old);
        assert!(!accepted);
        assert_eq!((retained, unchanged), (old, old));
        assert_eq!(report.work(WorkKind::OrderMove), 0);
        assert_eq!(report.peak(ResourceKind::ScratchBytes), old);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScopeKind {
    Ordinary,
    Actual,
    Positioned,
    Float,
    AtomicInline,
}

#[derive(Clone, Copy)]
struct ScopeInfo {
    kind: ScopeKind,
    owner: BoxId,
    parent: Option<BoxId>,
}

#[derive(Clone, Copy)]
struct Invocation {
    box_id: BoxId,
    limit: Option<FragmentId>,
}

#[derive(Clone, Copy)]
enum Stage {
    Decoration,
    Negative,
    Blocks,
    Floats,
    Content,
    Zero,
    Positive,
    Outlines,
}

#[derive(Clone, Copy)]
enum Task {
    Scope(Invocation),
    Stage(Invocation, Stage),
    EndContext(BoxId),
    Inline(FragmentId, Invocation),
    Line(LineId, Invocation, Option<BoxId>),
    Promoted(FragmentId, Invocation),
    Replaced(FragmentId),
}

#[derive(Clone, Copy)]
struct FlowCandidate {
    key: (usize, usize, usize),
    entry: ContentEntry,
}

#[derive(Default)]
pub(super) struct SequenceState {
    pub(super) events: Vec<OrderEvent>,
    tasks: Vec<Task>,
    scopes: Vec<ScopeInfo>,
    order: Vec<usize>,
    members: Vec<usize>,
    children: Vec<usize>,
    member_ranges: Vec<Range<usize>>,
    child_ranges: Vec<Range<usize>>,
    merge: Vec<usize>,
    drawn: Vec<bool>,
    context_state: Vec<u8>,
    flow: Vec<FlowCandidate>,
    flow_order: Vec<usize>,
    flow_merge: Vec<usize>,
}

impl SequenceState {
    pub(super) fn scratch_cleanup(
        &mut self,
        p: &mut PhaseMeter<'_, '_>,
        root: Scope,
    ) -> Result<(), PaintError> {
        macro_rules! drop_scratch {
            ($($field:ident),* $(,)?) => {$(
                release(p, root, take(&mut self.$field), ResourceKind::ScratchBytes)?;
            )*};
        }
        drop_scratch!(
            tasks,
            scopes,
            order,
            members,
            children,
            member_ranges,
            child_ranges,
            merge,
            drawn,
            context_state,
            flow,
            flow_order,
            flow_merge
        );
        Ok(())
    }

    pub(super) fn result_cleanup(
        &mut self,
        p: &mut PhaseMeter<'_, '_>,
        root: Scope,
    ) -> Result<(), PaintError> {
        release(
            p,
            root,
            take(&mut self.events),
            ResourceKind::RetainedResultBytes,
        )?;
        Ok(())
    }
}

/// Keep replacement backing live with the old backing through the actual copy.
fn capacity<T: Copy>(
    values: &mut Vec<T>,
    wanted: usize,
    p: &mut PhaseMeter<'_, '_>,
    root: Scope,
    phase: Scope,
) -> Result<(), PaintError> {
    if wanted <= values.capacity() {
        return Ok(());
    }
    let doubled = values.capacity().checked_mul(2).unwrap_or(wanted);
    let mut next = allocate(p, root, wanted.max(doubled), ResourceKind::ScratchBytes)?;
    if let Err(error) = charge(p, phase, WorkKind::OrderMove, values.len()) {
        release(p, root, next, ResourceKind::ScratchBytes)?;
        return Err(error);
    }
    next.extend_from_slice(values);
    let old = std::mem::replace(values, next);
    release(p, root, old, ResourceKind::ScratchBytes)?;
    Ok(())
}

struct Sequencer<'a, 'p, 'm, 'operation> {
    request: PaintRequest<'a>,
    core: &'p algorithm::State,
    index: &'p PaintIndex,
    state: &'p mut SequenceState,
    p: &'p mut PhaseMeter<'m, 'operation>,
    root: Scope,
    phase: Scope,
}

pub(super) fn run(
    request: PaintRequest<'_>,
    core: &algorithm::State,
    index: &PaintIndex,
    state: &mut SequenceState,
    p: &mut PhaseMeter<'_, '_>,
    root: Scope,
    phase: Scope,
) -> Result<(), PaintError> {
    let mut runner = Sequencer {
        request,
        core,
        index,
        state,
        p,
        root,
        phase,
    };
    runner.prepare()?;
    runner.push(Task::Scope(Invocation {
        box_id: request.root(),
        limit: None,
    }))?;
    while let [remaining @ .., pending] = runner.state.tasks.as_mut_slice() {
        charge(runner.p, runner.phase, WorkKind::PaintScopeVisit, 1)?;
        let task = *pending;
        let keep = remaining.len();
        runner.state.tasks.truncate(keep);
        runner.execute(task)?;
    }
    for i in 0..runner.state.drawn.len() {
        runner.work(WorkKind::PaintValidation, 1)?;
        if !runner.state.drawn[i] {
            return Err(invariant());
        }
    }
    for i in 0..runner.state.context_state.len() {
        runner.work(WorkKind::PaintValidation, 1)?;
        if runner.state.context_state[i] != 2 {
            return Err(invariant());
        }
    }
    Ok(())
}

impl Sequencer<'_, '_, '_, '_> {
    fn work(&mut self, kind: WorkKind, amount: usize) -> Result<(), PaintError> {
        charge(self.p, self.phase, kind, amount)
    }

    fn push(&mut self, task: Task) -> Result<(), PaintError> {
        let wanted = self
            .state
            .tasks
            .len()
            .checked_add(1)
            .ok_or(PaintError::CapacityOverflow)?;
        capacity(&mut self.state.tasks, wanted, self.p, self.root, self.phase)?;
        self.work(WorkKind::PaintScopeVisit, 1)?;
        self.state.tasks.push(task);
        Ok(())
    }

    fn prepare(&mut self) -> Result<(), PaintError> {
        let n = self.request.boxes().len();
        let contexts = self.core.contexts.len();
        let events = contexts
            .checked_mul(2)
            .and_then(|count| count.checked_add(self.request.primitives().len()))
            .ok_or(PaintError::CapacityOverflow)?;
        self.state.events = allocate(self.p, self.root, events, ResourceKind::RetainedResultBytes)?;
        self.state.order = allocate(self.p, self.root, n, ResourceKind::ScratchBytes)?;
        self.work(WorkKind::OrderMove, n)?;
        self.state.order.resize(n, 0);
        self.state.scopes = allocate(self.p, self.root, n, ResourceKind::ScratchBytes)?;
        self.work(WorkKind::OrderMove, n)?;
        self.state.scopes.resize(
            n,
            ScopeInfo {
                kind: ScopeKind::Ordinary,
                owner: self.request.root(),
                parent: None,
            },
        );
        for i in 0..n {
            self.work(WorkKind::PaintScopeVisit, 1)?;
            self.work(WorkKind::OrderMove, 1)?;
            self.state.order[self.core.nodes[i].rank] = i;
        }
        for ordinal in 0..n {
            self.work(WorkKind::PaintScopeVisit, 1)?;
            let i = self.state.order[ordinal];
            let facts = self.request.boxes()[i];
            let context = self.core.contexts[self.core.membership[i]];
            let actual = context.box_id.get() == i;
            let paint = self.request.box_paint()[i];
            let kind = if actual {
                ScopeKind::Actual
            } else if facts.position() != Position::Static {
                ScopeKind::Positioned
            } else if paint.float() == FloatParticipation::Floating {
                ScopeKind::Float
            } else if paint.kind() == BoxPaintKind::InlineBlock
                || facts.participation() != Participation::Ordinary
            {
                ScopeKind::AtomicInline
            } else {
                ScopeKind::Ordinary
            };
            let inherited = facts
                .parent()
                .map(|parent| self.state.scopes[parent.get()].owner);
            let parent = match kind {
                ScopeKind::Actual => context.parent.map(|id| self.core.contexts[id].box_id),
                ScopeKind::Positioned => Some(context.box_id),
                _ => inherited,
            };
            self.work(WorkKind::OrderMove, 1)?;
            self.state.scopes[i] = ScopeInfo {
                kind,
                owner: if kind == ScopeKind::Ordinary {
                    inherited.ok_or_else(invariant)?
                } else {
                    BoxId::new(i)
                },
                parent,
            };
        }
        self.state.members = allocate(self.p, self.root, n, ResourceKind::ScratchBytes)?;
        self.work(WorkKind::OrderMove, n)?;
        self.state.members.extend_from_slice(&self.state.order);
        self.state.children = allocate(self.p, self.root, n, ResourceKind::ScratchBytes)?;
        for i in 0..n {
            self.work(WorkKind::PaintScopeVisit, 1)?;
            if self.state.scopes[i].kind != ScopeKind::Ordinary
                && self.state.scopes[i].parent.is_some()
            {
                self.work(WorkKind::OrderMove, 1)?;
                self.state.children.push(i);
            }
        }
        self.state.merge = allocate(self.p, self.root, n, ResourceKind::ScratchBytes)?;
        self.work(WorkKind::OrderMove, n)?;
        self.state.merge.resize(n, 0);
        let scopes = &self.state.scopes;
        let nodes = &self.core.nodes;
        algorithm::SortBuffers::new(&mut self.state.members, &mut self.state.merge)?.sort(
            self.p,
            self.phase,
            WorkKind::PaintOrderComparison,
            |a, b| {
                (scopes[a].owner.get(), nodes[a].rank).cmp(&(scopes[b].owner.get(), nodes[b].rank))
            },
        )?;
        algorithm::SortBuffers::new(&mut self.state.children, &mut self.state.merge)?.sort(
            self.p,
            self.phase,
            WorkKind::PaintOrderComparison,
            |a, b| {
                (scopes[a].parent.map(BoxId::get), nodes[a].rank)
                    .cmp(&(scopes[b].parent.map(BoxId::get), nodes[b].rank))
            },
        )?;
        self.state.member_ranges = allocate(self.p, self.root, n, ResourceKind::ScratchBytes)?;
        self.state.child_ranges = allocate(self.p, self.root, n, ResourceKind::ScratchBytes)?;
        self.work(WorkKind::OrderMove, n)?;
        self.state.member_ranges.resize(n, 0..0);
        self.work(WorkKind::OrderMove, n)?;
        self.state.child_ranges.resize(n, 0..0);
        let (mut member, mut child) = (0, 0);
        for i in 0..n {
            let start = member;
            while member < self.state.members.len() {
                self.work(WorkKind::PaintScopeVisit, 1)?;
                if self.state.scopes[self.state.members[member]].owner.get() != i {
                    break;
                }
                member += 1;
            }
            self.work(WorkKind::OrderMove, 1)?;
            self.state.member_ranges[i] = start..member;
            let start = child;
            while child < self.state.children.len() {
                self.work(WorkKind::PaintScopeVisit, 1)?;
                if self.state.scopes[self.state.children[child]]
                    .parent
                    .map(BoxId::get)
                    != Some(i)
                {
                    break;
                }
                child += 1;
            }
            self.work(WorkKind::OrderMove, 1)?;
            self.state.child_ranges[i] = start..child;
        }
        let primitives = self.request.primitives().len();
        self.state.drawn = allocate(self.p, self.root, primitives, ResourceKind::ScratchBytes)?;
        self.work(WorkKind::OrderMove, primitives)?;
        self.state.drawn.resize(primitives, false);
        self.state.context_state =
            allocate(self.p, self.root, contexts, ResourceKind::ScratchBytes)?;
        self.work(WorkKind::OrderMove, contexts)?;
        self.state.context_state.resize(contexts, 0);
        Ok(())
    }

    fn event(&mut self, event: OrderEvent) -> Result<(), PaintError> {
        if self.state.events.len() == self.state.events.capacity() {
            return Err(invariant());
        }
        self.work(WorkKind::OutputRecord, 1)?;
        self.state.events.push(event);
        Ok(())
    }

    fn context_event(&mut self, box_id: BoxId, end: bool) -> Result<(), PaintError> {
        let context = self.core.membership[box_id.get()];
        self.work(WorkKind::PaintValidation, 1)?;
        if self.state.context_state[context] != u8::from(end) {
            return Err(invariant());
        }
        self.event(if end {
            OrderEvent::EndContext(box_id)
        } else {
            OrderEvent::BeginContext(box_id)
        })?;
        self.state.context_state[context] = if end { 2 } else { 1 };
        Ok(())
    }

    fn draw(&mut self, id: PrimitiveId) -> Result<(), PaintError> {
        self.work(WorkKind::PaintValidation, 1)?;
        if self.state.drawn[id.get()] {
            return Err(invariant());
        }
        self.event(OrderEvent::Draw(id))?;
        self.state.drawn[id.get()] = true;
        Ok(())
    }

    fn group(
        &mut self,
        fragment: Option<FragmentId>,
        kind: PrimitiveKind,
    ) -> Result<(), PaintError> {
        let bucket = fragment.map_or(0, |id| id.get() + 1);
        for cursor in self.index.primitive_ranges[bucket].clone() {
            self.work(WorkKind::PrimitiveVisit, 1)?;
            let id = self.index.primitives[cursor];
            if self.request.primitives()[id].kind() == kind {
                self.draw(PrimitiveId::new(id))?;
            }
        }
        Ok(())
    }

    fn decoration(&mut self, fragment: FragmentId, root: bool) -> Result<(), PaintError> {
        if !root {
            self.group(
                Some(fragment),
                PrimitiveKind::Fragment(FragmentPaintKind::BackgroundColor),
            )?;
            self.group(
                Some(fragment),
                PrimitiveKind::Fragment(FragmentPaintKind::BackgroundImages),
            )?;
        }
        self.group(
            Some(fragment),
            PrimitiveKind::Fragment(FragmentPaintKind::Border),
        )
    }

    fn within(
        &mut self,
        fragment: FragmentId,
        limit: Option<FragmentId>,
    ) -> Result<bool, PaintError> {
        let Some(limit) = limit else {
            return Ok(true);
        };
        let mut current = Some(fragment);
        while let Some(id) = current {
            self.work(WorkKind::FragmentVisit, 1)?;
            if id == limit {
                return Ok(true);
            }
            current = self.request.fragments()[id.get()].parent();
        }
        Ok(false)
    }

    fn promotion_anchor(
        &mut self,
        fragment: FragmentId,
        scope: BoxId,
    ) -> Result<Option<FragmentId>, PaintError> {
        let mut current = Some(fragment);
        while let Some(id) = current {
            self.work(WorkKind::FragmentVisit, 1)?;
            let record = self.request.fragments()[id.get()];
            if record.block_in_inline_placement().is_some()
                && let FragmentOwner::Box(owner) = record.owner()
                && owner != scope
                && self.state.scopes[owner.get()].owner == scope
            {
                return Ok(Some(id));
            }
            current = record.parent();
        }
        Ok(None)
    }

    fn execute(&mut self, task: Task) -> Result<(), PaintError> {
        match task {
            Task::Scope(invocation) => {
                let actual = self.state.scopes[invocation.box_id.get()].kind == ScopeKind::Actual;
                if actual {
                    self.context_event(invocation.box_id, false)?;
                    self.push(Task::EndContext(invocation.box_id))?;
                }
                for stage in [
                    Stage::Outlines,
                    Stage::Positive,
                    Stage::Zero,
                    Stage::Content,
                    Stage::Floats,
                    Stage::Blocks,
                    Stage::Negative,
                    Stage::Decoration,
                ] {
                    self.push(Task::Stage(invocation, stage))?;
                }
                Ok(())
            }
            Task::EndContext(id) => self.context_event(id, true),
            Task::Stage(invocation, stage) => self.stage(invocation, stage),
            Task::Inline(fragment, invocation) => self.inline(fragment, invocation),
            Task::Line(line, invocation, inline_root) => self.line(line, invocation, inline_root),
            Task::Promoted(fragment, invocation) => self.foreground(invocation, Some(fragment)),
            Task::Replaced(fragment) => self.group(
                Some(fragment),
                PrimitiveKind::Fragment(FragmentPaintKind::ReplacedContent),
            ),
        }
    }

    fn stage(&mut self, invocation: Invocation, stage: Stage) -> Result<(), PaintError> {
        match stage {
            Stage::Decoration => self.own_decoration(invocation),
            Stage::Negative => self.context_band(invocation, -1),
            Stage::Blocks => self.blocks(invocation),
            Stage::Floats => self.floats(invocation),
            Stage::Content => self.content(invocation),
            Stage::Zero => self.context_band(invocation, 0),
            Stage::Positive => self.context_band(invocation, 1),
            Stage::Outlines => self.outlines(invocation),
        }
    }
}

impl Sequencer<'_, '_, '_, '_> {
    fn own_decoration(&mut self, invocation: Invocation) -> Result<(), PaintError> {
        let owner = invocation.box_id;
        let root = owner == self.request.root();
        if root {
            self.group(
                None,
                PrimitiveKind::Canvas(CanvasPaintKind::BackgroundColor),
            )?;
            self.group(
                None,
                PrimitiveKind::Canvas(CanvasPaintKind::BackgroundImages),
            )?;
        }
        let kind = self.request.box_paint()[owner.get()].kind();
        if matches!(kind, BoxPaintKind::Inline | BoxPaintKind::InlineReplaced) {
            return Ok(());
        }
        for cursor in self.index.fragment_ranges[owner.get()].clone() {
            self.work(WorkKind::FragmentVisit, 1)?;
            let fragment = FragmentId::new(self.index.fragments[cursor]);
            if self.within(fragment, invocation.limit)? {
                self.decoration(fragment, root)?;
            }
        }
        Ok(())
    }

    fn context_band(&mut self, invocation: Invocation, band: i8) -> Result<(), PaintError> {
        let owner = invocation.box_id;
        if self.state.scopes[owner.get()].kind != ScopeKind::Actual {
            return Ok(());
        }
        if band == 0 {
            for cursor in self.state.child_ranges[owner.get()].clone().rev() {
                self.work(WorkKind::PaintScopeVisit, 1)?;
                let i = self.state.children[cursor];
                let kind = self.state.scopes[i].kind;
                let zero_context = kind == ScopeKind::Actual
                    && self.core.contexts[self.core.membership[i]].level == 0;
                if zero_context || kind == ScopeKind::Positioned {
                    self.push(Task::Scope(Invocation {
                        box_id: BoxId::new(i),
                        limit: None,
                    }))?;
                }
            }
        } else {
            let context = self.core.contexts[self.core.membership[owner.get()]];
            for cursor in (context.start..context.end).rev() {
                self.work(WorkKind::PaintScopeVisit, 1)?;
                let child = self.core.contexts[self.core.children[cursor]];
                if (band < 0 && child.level < 0) || (band > 0 && child.level > 0) {
                    self.push(Task::Scope(Invocation {
                        box_id: child.box_id,
                        limit: None,
                    }))?;
                }
            }
        }
        Ok(())
    }

    fn blocks(&mut self, invocation: Invocation) -> Result<(), PaintError> {
        for cursor in self.state.member_ranges[invocation.box_id.get()].clone() {
            self.work(WorkKind::PaintScopeVisit, 1)?;
            let i = self.state.members[cursor];
            if i == invocation.box_id.get() || self.state.scopes[i].kind != ScopeKind::Ordinary {
                continue;
            }
            if !matches!(
                self.request.box_paint()[i].kind(),
                BoxPaintKind::Block | BoxPaintKind::BlockReplaced
            ) {
                continue;
            }
            for occurrence in self.index.fragment_ranges[i].clone() {
                self.work(WorkKind::FragmentVisit, 1)?;
                let fragment = FragmentId::new(self.index.fragments[occurrence]);
                if self.within(fragment, invocation.limit)? {
                    self.decoration(fragment, false)?;
                }
            }
        }
        Ok(())
    }

    fn floats(&mut self, invocation: Invocation) -> Result<(), PaintError> {
        for cursor in self.state.child_ranges[invocation.box_id.get()]
            .clone()
            .rev()
        {
            self.work(WorkKind::PaintScopeVisit, 1)?;
            let i = self.state.children[cursor];
            if self.state.scopes[i].kind != ScopeKind::Float {
                continue;
            }
            let mut present = false;
            for occurrence in self.index.fragment_ranges[i].clone() {
                self.work(WorkKind::FragmentVisit, 1)?;
                if self.within(
                    FragmentId::new(self.index.fragments[occurrence]),
                    invocation.limit,
                )? {
                    present = true;
                    break;
                }
            }
            if present {
                self.push(Task::Scope(Invocation {
                    box_id: BoxId::new(i),
                    limit: invocation.limit,
                }))?;
            }
        }
        Ok(())
    }

    fn content(&mut self, invocation: Invocation) -> Result<(), PaintError> {
        match self.request.box_paint()[invocation.box_id.get()].kind() {
            BoxPaintKind::Inline | BoxPaintKind::InlineReplaced => {
                self.inline_scope_content(invocation)
            }
            _ => self.foreground(invocation, None),
        }
    }

    fn foreground(
        &mut self,
        invocation: Invocation,
        anchor: Option<FragmentId>,
    ) -> Result<(), PaintError> {
        let members = self.state.member_ranges[invocation.box_id.get()].clone();
        let children = self.state.child_ranges[invocation.box_id.get()].clone();
        let (mut member, mut child) = (members.end, children.end);
        while member > members.start || child > children.start {
            let from_child = if member == members.start {
                true
            } else if child == children.start {
                false
            } else {
                self.work(WorkKind::PaintOrderComparison, 1)?;
                self.core.nodes[self.state.children[child - 1]].rank
                    > self.core.nodes[self.state.members[member - 1]].rank
            };
            self.work(WorkKind::PaintScopeVisit, 1)?;
            let i = if from_child {
                child -= 1;
                self.state.children[child]
            } else {
                member -= 1;
                self.state.members[member]
            };
            let item = from_child
                && self.state.scopes[i].kind == ScopeKind::AtomicInline
                && self.request.boxes()[i].participation() != Participation::Ordinary;
            let ordinary = !from_child
                && (i == invocation.box_id.get()
                    || self.state.scopes[i].kind == ScopeKind::Ordinary)
                && matches!(
                    self.request.box_paint()[i].kind(),
                    BoxPaintKind::Block | BoxPaintKind::BlockReplaced | BoxPaintKind::InlineBlock
                );
            if !item && !ordinary {
                continue;
            }
            for cursor in self.index.fragment_ranges[i].clone().rev() {
                self.work(WorkKind::FragmentVisit, 1)?;
                let fragment = FragmentId::new(self.index.fragments[cursor]);
                if !self.within(fragment, invocation.limit)?
                    || self.promotion_anchor(fragment, invocation.box_id)? != anchor
                {
                    continue;
                }
                if item {
                    self.push(Task::Scope(Invocation {
                        box_id: BoxId::new(i),
                        limit: Some(fragment),
                    }))?;
                } else if self.request.box_paint()[i].kind() == BoxPaintKind::BlockReplaced {
                    self.push(Task::Replaced(fragment))?;
                } else {
                    self.container_content(fragment, invocation, None)?;
                }
            }
        }
        Ok(())
    }

    fn container_content(
        &mut self,
        container: FragmentId,
        invocation: Invocation,
        inline_root: Option<BoxId>,
    ) -> Result<(), PaintError> {
        for cursor in self.index.content_ranges[container.get()].clone().rev() {
            self.work(WorkKind::InlineMemberVisit, 1)?;
            match self.index.content[cursor] {
                ContentEntry::Line(line) => self.push(Task::Line(line, invocation, inline_root))?,
                ContentEntry::PromotedBlock(fragment) => {
                    self.work(WorkKind::FragmentVisit, 1)?;
                    let FragmentOwner::Box(owner) =
                        self.request.fragments()[fragment.get()].owner()
                    else {
                        return Err(invariant());
                    };
                    if self.state.scopes[owner.get()].owner == invocation.box_id
                        && self.within(fragment, invocation.limit)?
                    {
                        self.push(Task::Promoted(fragment, invocation))?;
                    }
                }
            }
        }
        Ok(())
    }

    fn inline_scope_content(&mut self, invocation: Invocation) -> Result<(), PaintError> {
        self.state.flow.clear();
        for cursor in self.index.fragment_ranges[invocation.box_id.get()].clone() {
            self.work(WorkKind::FragmentVisit, 1)?;
            let fragment = FragmentId::new(self.index.fragments[cursor]);
            if !self.within(fragment, invocation.limit)? {
                continue;
            }
            let line = self.request.fragments()[fragment.get()]
                .line()
                .ok_or_else(invariant)?;
            self.work(WorkKind::LineVisit, 1)?;
            let record = self.request.lines()[line.get()];
            self.add_flow(
                record.container(),
                record.logical_order(),
                ContentEntry::Line(line),
            )?;
        }
        for cursor in self.state.member_ranges[invocation.box_id.get()].clone() {
            self.work(WorkKind::PaintScopeVisit, 1)?;
            let i = self.state.members[cursor];
            if self.state.scopes[i].kind != ScopeKind::Ordinary {
                continue;
            }
            for occurrence in self.index.fragment_ranges[i].clone() {
                self.work(WorkKind::FragmentVisit, 1)?;
                let fragment = FragmentId::new(self.index.fragments[occurrence]);
                let record = self.request.fragments()[fragment.get()];
                let Some(placement) = record.block_in_inline_placement() else {
                    continue;
                };
                let outside = match record.parent() {
                    Some(parent) => self.promotion_anchor(parent, invocation.box_id)?,
                    None => None,
                };
                if outside.is_none() && self.within(fragment, invocation.limit)? {
                    self.add_flow(
                        placement.container(),
                        placement.content_order(),
                        ContentEntry::PromotedBlock(fragment),
                    )?;
                }
            }
        }
        let n = self.state.flow.len();
        self.state.flow_order.clear();
        self.state.flow_merge.clear();
        capacity(&mut self.state.flow_order, n, self.p, self.root, self.phase)?;
        capacity(&mut self.state.flow_merge, n, self.p, self.root, self.phase)?;
        self.work(WorkKind::OrderMove, n)?;
        self.state.flow_order.extend(0..n);
        self.work(WorkKind::OrderMove, n)?;
        self.state.flow_merge.resize(n, 0);
        let flow = &self.state.flow;
        algorithm::SortBuffers::new(&mut self.state.flow_order, &mut self.state.flow_merge)?.sort(
            self.p,
            self.phase,
            WorkKind::PaintOrderComparison,
            |a, b| flow[a].key.cmp(&flow[b].key),
        )?;
        let mut previous = None;
        for cursor in (0..n).rev() {
            self.work(WorkKind::InlineMemberVisit, 1)?;
            let candidate = self.state.flow[self.state.flow_order[cursor]];
            if previous == Some(candidate.entry) {
                continue;
            }
            previous = Some(candidate.entry);
            match candidate.entry {
                ContentEntry::Line(line) => {
                    self.push(Task::Line(line, invocation, Some(invocation.box_id)))?
                }
                ContentEntry::PromotedBlock(fragment) => {
                    self.push(Task::Promoted(fragment, invocation))?
                }
            }
        }
        Ok(())
    }

    fn add_flow(
        &mut self,
        container: FragmentId,
        order: usize,
        entry: ContentEntry,
    ) -> Result<(), PaintError> {
        self.work(WorkKind::FragmentVisit, 1)?;
        let record = self.request.fragments()[container.get()];
        let FragmentOwner::Box(owner) = record.owner() else {
            return Err(invariant());
        };
        let key = (
            self.core.nodes[owner.get()].rank,
            record.logical_order(),
            order,
        );
        let wanted = self
            .state
            .flow
            .len()
            .checked_add(1)
            .ok_or(PaintError::CapacityOverflow)?;
        capacity(&mut self.state.flow, wanted, self.p, self.root, self.phase)?;
        self.work(WorkKind::OrderMove, 1)?;
        self.state.flow.push(FlowCandidate { key, entry });
        Ok(())
    }

    fn line(
        &mut self,
        line: LineId,
        invocation: Invocation,
        inline_root: Option<BoxId>,
    ) -> Result<(), PaintError> {
        self.work(WorkKind::LineVisit, 1)?;
        let container = self.request.lines()[line.get()].container();
        if let Some(owner) = inline_root {
            for cursor in self.index.fragment_ranges[owner.get()].clone().rev() {
                self.work(WorkKind::FragmentVisit, 1)?;
                let fragment = FragmentId::new(self.index.fragments[cursor]);
                if self.request.fragments()[fragment.get()].line() == Some(line)
                    && self.within(fragment, invocation.limit)?
                {
                    self.push(Task::Inline(fragment, invocation))?;
                }
            }
            Ok(())
        } else {
            self.inline_children(container, line, invocation)
        }
    }

    fn inline(&mut self, fragment: FragmentId, invocation: Invocation) -> Result<(), PaintError> {
        self.work(WorkKind::FragmentVisit, 1)?;
        let record = self.request.fragments()[fragment.get()];
        match record.owner() {
            FragmentOwner::Text(_) => {
                for kind in [
                    PrimitiveKind::Decoration(DecorationKind::Underline),
                    PrimitiveKind::Decoration(DecorationKind::Overline),
                    PrimitiveKind::Fragment(FragmentPaintKind::Text),
                    PrimitiveKind::Decoration(DecorationKind::LineThrough),
                ] {
                    self.group(Some(fragment), kind)?;
                }
                Ok(())
            }
            FragmentOwner::Box(owner) => {
                let info = self.state.scopes[owner.get()];
                if info.kind == ScopeKind::AtomicInline && owner != invocation.box_id {
                    return self.push(Task::Scope(Invocation {
                        box_id: owner,
                        limit: Some(fragment),
                    }));
                }
                self.decoration(fragment, false)?;
                match self.request.box_paint()[owner.get()].kind() {
                    BoxPaintKind::Inline => self.inline_children(
                        fragment,
                        record.line().ok_or_else(invariant)?,
                        invocation,
                    ),
                    BoxPaintKind::InlineReplaced => self.group(
                        Some(fragment),
                        PrimitiveKind::Fragment(FragmentPaintKind::ReplacedContent),
                    ),
                    _ => Err(invariant()),
                }
            }
        }
    }

    fn inline_children(
        &mut self,
        parent: FragmentId,
        line: LineId,
        invocation: Invocation,
    ) -> Result<(), PaintError> {
        self.work(WorkKind::FragmentVisit, 1)?;
        let FragmentOwner::Box(owner) = self.request.fragments()[parent.get()].owner() else {
            return Err(invariant());
        };
        for cursor in self.index.source_ranges[owner.get()].clone().rev() {
            self.work(WorkKind::InlineMemberVisit, 1)?;
            let source = self.index.sources[cursor];
            if source < self.request.boxes().len() {
                let info = self.state.scopes[source];
                let visible = info.kind == ScopeKind::Ordinary && info.owner == invocation.box_id
                    || info.kind == ScopeKind::AtomicInline
                        && info.parent == Some(invocation.box_id);
                if !visible {
                    continue;
                }
            }
            for occurrence in self.index.fragment_ranges[source].clone().rev() {
                self.work(WorkKind::FragmentVisit, 1)?;
                let fragment = FragmentId::new(self.index.fragments[occurrence]);
                let record = self.request.fragments()[fragment.get()];
                if record.parent() == Some(parent) && record.line() == Some(line) {
                    self.push(Task::Inline(fragment, invocation))?;
                }
            }
        }
        Ok(())
    }

    fn outlines(&mut self, invocation: Invocation) -> Result<(), PaintError> {
        for cursor in self.state.member_ranges[invocation.box_id.get()].clone() {
            self.work(WorkKind::PaintScopeVisit, 1)?;
            let owner = self.state.members[cursor];
            for occurrence in self.index.fragment_ranges[owner].clone() {
                self.work(WorkKind::FragmentVisit, 1)?;
                let fragment = FragmentId::new(self.index.fragments[occurrence]);
                if self.within(fragment, invocation.limit)? {
                    self.group(
                        Some(fragment),
                        PrimitiveKind::Fragment(FragmentPaintKind::Outline),
                    )?;
                }
            }
        }
        Ok(())
    }
}
