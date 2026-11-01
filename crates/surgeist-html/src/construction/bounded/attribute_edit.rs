//! Complete, exclusive edits of the actual ordered attribute backing.

use super::*;
use crate::Attribute;
use crate::values::bounded::{
    ComparisonStep, Phase as NamePhase, advance_comparison, comparison_layout,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlConstructionAttributeEditMode {
    NamedMerge { candidate_count: usize },
    Slots,
}

/// Actual public-payload vector dimensions; no allocation or release authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlConstructionAttributeCapacity {
    pub initialized_len: usize,
    pub capacity: usize,
}
impl HtmlConstructionAttributeCapacity {
    pub fn element_size(self) -> usize {
        size_of::<Attribute>()
    }
    pub fn element_alignment(self) -> usize {
        align_of::<Attribute>()
    }
}

/// Fixed bounds admitted before the action, excluding allocator internals.
/// Comparison actions conservatively admit retained flags whose writes depend
/// on the comparison outcome, before reading that outcome.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlConstructionAttributeWork {
    pub record_visits: usize,
    pub provider_calls: usize,
    pub inspected_bytes: usize,
    pub copied_bytes: usize,
    pub record_writes: usize,
    /// Largest indivisible physical transfer or individual fixed field write.
    /// This is a field-wise atomic maximum, not the sum of copied bytes.
    pub atomic_layout_bytes: usize,
    pub auxiliary_entries: usize,
}

#[derive(Debug)]
pub enum HtmlConstructionAttributeDescription<'a> {
    Inspect(HtmlConstructionAttributeWork),
    UniqueBacking(HtmlConstructionAttributeWork),
    PrepareAllocationReport(HtmlConstructionAttributeWork),
    Compare {
        prior_index: Option<usize>,
        work: HtmlConstructionAttributeWork,
    },
    ReservePlan {
        allocation: &'a HtmlConstructionAllocationSnapshot,
        minimum_capacity: usize,
        work: HtmlConstructionAttributeWork,
    },
    InitializeNameWitness(HtmlConstructionAttributeWork),
    ReserveAttributes {
        before: HtmlConstructionAttributeCapacity,
        minimum_capacity: usize,
        work: HtmlConstructionAttributeWork,
    },
    Commit {
        work: HtmlConstructionAttributeWork,
        usage: HtmlConstructionUsage,
    },
}
impl HtmlConstructionAttributeDescription<'_> {
    fn transitions<T>(mut self) -> Self {
        let work = match &mut self {
            Self::Inspect(v)
            | Self::UniqueBacking(v)
            | Self::PrepareAllocationReport(v)
            | Self::InitializeNameWitness(v) => v,
            Self::Compare { work, .. }
            | Self::ReservePlan { work, .. }
            | Self::ReserveAttributes { work, .. }
            | Self::Commit { work, .. } => work,
        };
        // Perform and accept each write phase:T and next:Option<T>. Do not
        // assume these two compiled fields have equal widths. A failed action
        // stores its typed failure instead; admit the larger concrete path.
        work.record_writes += 4;
        work.copied_bytes += (2 * size_of::<T>() + 2 * size_of::<Option<T>>())
            .max(size_of::<Option<HtmlConstructionError>>() + size_of::<T>());
        work.atomic_layout_bytes = work
            .atomic_layout_bytes
            .max(size_of::<T>())
            .max(size_of::<Option<T>>())
            .max(size_of::<Option<HtmlConstructionError>>());
        self
    }
    pub fn work(&self) -> HtmlConstructionAttributeWork {
        match self {
            Self::Inspect(v)
            | Self::UniqueBacking(v)
            | Self::PrepareAllocationReport(v)
            | Self::InitializeNameWitness(v) => *v,
            Self::Compare { work, .. }
            | Self::ReservePlan { work, .. }
            | Self::ReserveAttributes { work, .. }
            | Self::Commit { work, .. } => *work,
        }
    }
    pub fn prior_index(&self) -> Option<usize> {
        match self {
            Self::Compare { prior_index, .. } => *prior_index,
            _ => None,
        }
    }
}

#[derive(Debug)]
struct NameDecision {
    name: Weak<Vec<u16>>,
    keep: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GroupPhase {
    Target,
    Report,
    Ready,
    Observing,
    Failed,
}

/// Owns the arena and its only merge plan; no name admission escapes this owner.
pub struct HtmlConstructionAttributeEdit {
    storage: HtmlConstructionStorage,
    element: HtmlBuilderNode,
    mode: HtmlConstructionAttributeEditMode,
    target: usize,
    target_checked: bool,
    phase: GroupPhase,
    next: Option<GroupPhase>,
    failure: Option<HtmlConstructionError>,
    busy: bool,
    used: bool,
    capacity: HtmlConstructionAttributeCapacity,
    decisions: Vec<NameDecision>,
    plan: Option<HtmlConstructionAllocationSnapshot>,
    selected: usize,
    appended: usize,
    next_selected: usize,
}
pub struct HtmlConstructionAttributeRejection {
    storage: HtmlConstructionStorage,
    element: HtmlBuilderNode,
    mode: HtmlConstructionAttributeEditMode,
    reason: HtmlConstructionError,
}
impl HtmlConstructionAttributeRejection {
    pub fn into_parts(
        self,
    ) -> (
        HtmlConstructionStorage,
        HtmlBuilderNode,
        HtmlConstructionAttributeEditMode,
        HtmlConstructionError,
    ) {
        (self.storage, self.element, self.mode, self.reason)
    }
}
pub struct HtmlConstructionAttributeCompletion {
    storage: HtmlConstructionStorage,
    retirement: HtmlConstructionAttributePlanRetirement,
}
impl HtmlConstructionAttributeCompletion {
    pub fn into_parts(
        self,
    ) -> (
        HtmlConstructionStorage,
        HtmlConstructionAttributePlanRetirement,
    ) {
        (self.storage, self.retirement)
    }
}
pub struct HtmlConstructionAttributeAction<'a> {
    edit: &'a mut HtmlConstructionAttributeEdit,
}
pub struct HtmlConstructionAttributeObservation<'a> {
    edit: &'a mut HtmlConstructionAttributeEdit,
}
impl HtmlConstructionAttributeEdit {
    #[expect(
        clippy::result_large_err,
        reason = "refusal returns the actual allocation-free storage and operands"
    )]
    pub fn begin(
        storage: HtmlConstructionStorage,
        element: HtmlBuilderNode,
        mode: HtmlConstructionAttributeEditMode,
    ) -> Result<Self, HtmlConstructionAttributeRejection> {
        Ok(Self {
            storage,
            element,
            mode,
            target: 0,
            target_checked: false,
            phase: GroupPhase::Target,
            next: None,
            failure: None,
            busy: false,
            used: false,
            capacity: HtmlConstructionAttributeCapacity::default(),
            decisions: Vec::new(),
            plan: None,
            selected: 0,
            appended: 0,
            next_selected: 0,
        })
    }
    pub fn storage_view(&self) -> &HtmlConstructionStorage {
        &self.storage
    }
    pub fn shared_attributes(&self) -> Result<&Arc<Vec<Attribute>>, HtmlConstructionError> {
        if !self.target_checked {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        match &self.storage.arena.records[self.target].kind {
            HtmlNodeKind::Element(v) => Ok(v.attributes.shared_attributes()),
            _ => Err(HtmlConstructionError::WrongNodeKind),
        }
    }
    pub fn capacity(&self) -> HtmlConstructionAttributeCapacity {
        self.capacity
    }
    fn attributes(&self) -> &[Attribute] {
        self.shared_attributes()
            .expect("checked element")
            .as_slice()
    }
    fn backing_mut(&mut self) -> &mut Arc<Vec<Attribute>> {
        match &mut self.storage.arena.records[self.target].kind {
            HtmlNodeKind::Element(v) => v.attributes.construction_backing_mut(),
            _ => unreachable!("checked element"),
        }
    }
    fn check_ready(&self) -> Result<(), HtmlConstructionError> {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        if self.phase != GroupPhase::Ready || self.busy {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        Ok(())
    }
    fn planning_complete(&self) -> bool {
        match self.mode {
            HtmlConstructionAttributeEditMode::NamedMerge { candidate_count } => {
                self.decisions.len() == candidate_count
            }
            HtmlConstructionAttributeEditMode::Slots => true,
        }
    }
    fn poison(&mut self) {
        self.busy = true;
    }
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionAttributeAction<'_>>, HtmlConstructionError> {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        if self.busy || self.phase == GroupPhase::Observing {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        Ok((self.phase != GroupPhase::Ready)
            .then_some(HtmlConstructionAttributeAction { edit: self }))
    }
    pub fn candidate<'e, 'n>(
        &'e mut self,
        index: usize,
        name: &'n DocumentString,
    ) -> Result<HtmlConstructionAttributeCandidate<'e, 'n>, HtmlConstructionError> {
        self.check_ready()?;
        let HtmlConstructionAttributeEditMode::NamedMerge { candidate_count } = self.mode else {
            return Err(HtmlConstructionError::AttributeOrderMismatch);
        };
        if index != self.decisions.len() || index >= candidate_count {
            return Err(HtmlConstructionError::AttributeOrderMismatch);
        }
        Ok(HtmlConstructionAttributeCandidate {
            edit: self,
            index,
            name,
            prior_name: None,
            phase: CandidatePhase::Target(0),
            next: None,
            found: false,
            complete: false,
            failure: None,
        })
    }
    pub fn decision(&self, index: usize) -> Result<bool, HtmlConstructionError> {
        self.check_ready()?;
        self.decisions
            .get(index)
            .map(|v| v.keep)
            .ok_or(HtmlConstructionError::AttributeOrderMismatch)
    }
    pub fn selected_count(&self) -> Result<usize, HtmlConstructionError> {
        self.check_ready()?;
        if !self.planning_complete() {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        Ok(self.selected)
    }
    fn slot<'a>(
        &'a mut self,
        input: SlotInput<'a>,
    ) -> Result<HtmlConstructionAttributeSlot<'a>, HtmlConstructionError> {
        self.check_ready()?;
        if !self.planning_complete() || self.used {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        if matches!(
            self.mode,
            HtmlConstructionAttributeEditMode::NamedMerge { .. }
        ) && !matches!(input, SlotInput::Append(_) | SlotInput::Reserve(_))
        {
            return Err(HtmlConstructionError::AttributeOrderMismatch);
        }
        let usage = self.storage.usage();
        Ok(HtmlConstructionAttributeSlot {
            edit: self,
            input,
            phase: SlotPhase::Input,
            next: None,
            failure: None,
            finished: false,
            usage,
            minimum_capacity: 0,
            plan_index: 0,
        })
    }
    pub fn reserve(
        &mut self,
        additional: usize,
    ) -> Result<HtmlConstructionAttributeSlot<'_>, HtmlConstructionError> {
        self.slot(SlotInput::Reserve(additional))
    }
    pub fn append<'a>(
        &'a mut self,
        incoming: &'a mut Vec<Attribute>,
    ) -> Result<HtmlConstructionAttributeSlot<'a>, HtmlConstructionError> {
        self.slot(SlotInput::Append(incoming))
    }
    pub fn replace<'a>(
        &'a mut self,
        index: usize,
        incoming: &'a mut Option<Attribute>,
    ) -> Result<HtmlConstructionAttributeSlot<'a>, HtmlConstructionError> {
        self.slot(SlotInput::Replace { index, incoming })
    }
    pub fn swap_adjacent(
        &mut self,
        index: usize,
    ) -> Result<HtmlConstructionAttributeSlot<'_>, HtmlConstructionError> {
        self.slot(SlotInput::Swap(index))
    }
    pub fn pop(&mut self) -> Result<HtmlConstructionAttributeSlot<'_>, HtmlConstructionError> {
        self.slot(SlotInput::Pop)
    }
    pub fn try_finish(
        owner: &mut Option<Self>,
    ) -> Result<HtmlConstructionAttributeCompletion, HtmlConstructionError> {
        let edit = owner
            .as_ref()
            .ok_or(HtmlConstructionError::OperationConsumed)?;
        edit.check_ready()?;
        if !edit.planning_complete() || edit.appended != edit.selected {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        Ok(owner.take().expect("checked group").abort())
    }
    pub fn abort(self) -> HtmlConstructionAttributeCompletion {
        HtmlConstructionAttributeCompletion {
            storage: self.storage,
            retirement: HtmlConstructionAttributePlanRetirement {
                decisions: self.decisions,
                snapshot: self.plan,
            },
        }
    }
}
impl<'a> HtmlConstructionAttributeAction<'a> {
    pub fn description(&self) -> HtmlConstructionAttributeDescription<'_> {
        match self.edit.phase {
            GroupPhase::Target => {
                HtmlConstructionAttributeDescription::Inspect(HtmlConstructionAttributeWork {
                    record_visits: 5,
                    provider_calls: 2,
                    record_writes: 3,
                    copied_bytes: size_of::<usize>()
                        + size_of::<HtmlConstructionAttributeCapacity>()
                        + size_of::<bool>(),
                    atomic_layout_bytes: size_of::<HtmlConstructionAttributeCapacity>(),
                    ..Default::default()
                })
            }
            GroupPhase::Report => HtmlConstructionAttributeDescription::PrepareAllocationReport(
                HtmlConstructionAttributeWork {
                    record_visits: 3,
                    provider_calls: 1,
                    copied_bytes: size_of::<HtmlConstructionAllocationSnapshot>()
                        + size_of::<u64>(),
                    atomic_layout_bytes: size_of::<HtmlConstructionAllocationSnapshot>(),
                    record_writes: 2,
                    ..Default::default()
                },
            ),
            _ => unreachable!("only pending group actions"),
        }
        .transitions::<GroupPhase>()
    }
    pub fn perform(
        self,
    ) -> Result<HtmlConstructionAttributeObservation<'a>, HtmlConstructionError> {
        let edit = self.edit;
        let result = (|| {
            Ok(match edit.phase {
                GroupPhase::Target => {
                    edit.target = edit.storage.position(&edit.element)?;
                    let HtmlNodeKind::Element(v) = &edit.storage.arena.records[edit.target].kind
                    else {
                        return Err(HtmlConstructionError::WrongNodeKind);
                    };
                    edit.capacity = HtmlConstructionAttributeCapacity {
                        initialized_len: v.attributes.len(),
                        capacity: v.attributes.shared_attributes().capacity(),
                    };
                    edit.target_checked = true;
                    if matches!(
                        edit.mode,
                        HtmlConstructionAttributeEditMode::NamedMerge { .. }
                    ) {
                        GroupPhase::Report
                    } else {
                        GroupPhase::Ready
                    }
                }
                GroupPhase::Report => {
                    let generation = edit
                        .storage
                        .arena
                        .temporary_generation
                        .checked_add(1)
                        .ok_or(HtmlConstructionError::RepresentationExhausted)?;
                    let report = HtmlConstructionAllocationSnapshot::vector(
                        &edit.storage.arena.identity,
                        AllocationRole::AttributePlan(edit.target),
                        generation,
                        &edit.decisions,
                    )?;
                    edit.storage.arena.temporary_generation = generation;
                    edit.plan = Some(report);
                    GroupPhase::Ready
                }
                _ => unreachable!("only pending group actions"),
            })
        })();
        match result {
            Ok(next) => {
                edit.next = Some(next);
                edit.phase = GroupPhase::Observing;
                Ok(HtmlConstructionAttributeObservation { edit })
            }
            Err(e) => {
                edit.failure = Some(e.clone());
                edit.phase = GroupPhase::Failed;
                Err(e)
            }
        }
    }
}
impl HtmlConstructionAttributeObservation<'_> {
    pub fn allocation(&self) -> Option<&HtmlConstructionAllocationSnapshot> {
        self.edit.plan.as_ref()
    }
    pub fn accept(self) {
        self.edit.phase = self.edit.next.take().expect("pending group observation");
    }
}

#[derive(Clone, Copy, Debug)]
enum CandidatePhase {
    Target(usize),
    TargetName { index: usize, name: NamePhase },
    Prior(usize),
    PriorName { index: usize, name: NamePhase },
    Reserve,
    Witness,
    Observing,
    Done,
}
pub struct HtmlConstructionAttributeCandidate<'e, 'n> {
    edit: &'e mut HtmlConstructionAttributeEdit,
    index: usize,
    name: &'n DocumentString,
    prior_name: Option<&'n DocumentString>,
    phase: CandidatePhase,
    next: Option<CandidatePhase>,
    found: bool,
    complete: bool,
    failure: Option<HtmlConstructionError>,
}
pub struct HtmlConstructionAttributeCandidateAction<'a, 'e, 'n> {
    candidate: &'a mut HtmlConstructionAttributeCandidate<'e, 'n>,
}
pub struct HtmlConstructionAttributeCandidateObservation<'a, 'e, 'n> {
    candidate: &'a mut HtmlConstructionAttributeCandidate<'e, 'n>,
    allocation: bool,
}
impl Drop for HtmlConstructionAttributeCandidate<'_, '_> {
    fn drop(&mut self) {
        if !self.complete {
            self.edit.poison();
        }
    }
}
impl<'e, 'n> HtmlConstructionAttributeCandidate<'e, 'n> {
    /// The actual retained report, including after a provider failure.
    pub fn allocation(&self) -> &HtmlConstructionAllocationSnapshot {
        self.edit.plan.as_ref().expect("candidate has issued plan")
    }
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionAttributeCandidateAction<'_, 'e, 'n>>, HtmlConstructionError>
    {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        if matches!(self.phase, CandidatePhase::Observing) {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        Ok((!matches!(self.phase, CandidatePhase::Done))
            .then_some(HtmlConstructionAttributeCandidateAction { candidate: self }))
    }
    pub fn try_finish(owner: &mut Option<Self>) -> Result<bool, HtmlConstructionError> {
        let candidate = owner
            .as_ref()
            .ok_or(HtmlConstructionError::OperationConsumed)?;
        if let Some(e) = &candidate.failure {
            return Err(e.clone());
        }
        if !matches!(candidate.phase, CandidatePhase::Done) {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        let mut candidate = owner.take().expect("ready candidate");
        candidate.complete = true;
        candidate.edit.busy = false;
        Ok(!candidate.found)
    }
}
fn comparison_work(phase: NamePhase) -> HtmlConstructionAttributeWork {
    let layout = comparison_layout(phase);
    HtmlConstructionAttributeWork {
        record_visits: layout.record_visits() as usize,
        inspected_bytes: layout.inspected_bytes() as usize,
        ..Default::default()
    }
}
impl<'a, 'e, 'n> HtmlConstructionAttributeCandidateAction<'a, 'e, 'n> {
    pub fn description(&self) -> HtmlConstructionAttributeDescription<'_> {
        let c = &self.candidate;
        match c.phase {
            CandidatePhase::TargetName { name, .. } => {
                let mut work = comparison_work(name);
                work.record_writes = 1;
                work.copied_bytes = size_of::<bool>();
                HtmlConstructionAttributeDescription::Compare {
                    prior_index: None,
                    work,
                }
            }
            CandidatePhase::PriorName { name, .. } => {
                let mut work = comparison_work(name);
                work.record_writes = 2;
                work.copied_bytes = size_of::<bool>() + size_of::<Option<&DocumentString>>();
                HtmlConstructionAttributeDescription::Compare {
                    prior_index: None,
                    work,
                }
            }
            CandidatePhase::Prior(index) => HtmlConstructionAttributeDescription::Compare {
                prior_index: (index < c.index).then_some(index),
                work: HtmlConstructionAttributeWork {
                    record_visits: 3,
                    provider_calls: if index < c.index { 2 } else { 0 },
                    record_writes: usize::from(index < c.index),
                    copied_bytes: usize::from(index < c.index)
                        * size_of::<Option<&DocumentString>>(),
                    ..Default::default()
                },
            },
            CandidatePhase::Reserve => HtmlConstructionAttributeDescription::ReservePlan {
                allocation: c.edit.plan.as_ref().expect("issued plan"),
                minimum_capacity: c.index + 1,
                work: HtmlConstructionAttributeWork {
                    provider_calls: 1,
                    record_visits: 2,
                    record_writes: 4,
                    copied_bytes: 2 * size_of::<usize>() + 2 * size_of::<u64>(),
                    ..Default::default()
                },
            },
            CandidatePhase::Witness => HtmlConstructionAttributeDescription::InitializeNameWitness(
                HtmlConstructionAttributeWork {
                    provider_calls: 1,
                    copied_bytes: size_of::<NameDecision>()
                        + 3 * size_of::<usize>()
                        + 2 * size_of::<bool>(),
                    // Vec slot/length, report length, selected count, then
                    // the guarded finish's complete/busy flags.
                    record_writes: 6,
                    atomic_layout_bytes: size_of::<NameDecision>(),
                    auxiliary_entries: 1,
                    ..Default::default()
                },
            ),
            CandidatePhase::Target(_) => {
                HtmlConstructionAttributeDescription::Inspect(HtmlConstructionAttributeWork {
                    record_visits: 1,
                    record_writes: usize::from(!c.edit.busy),
                    copied_bytes: usize::from(!c.edit.busy) * size_of::<bool>(),
                    ..Default::default()
                })
            }
            _ => unreachable!("pending candidate action"),
        }
        .transitions::<CandidatePhase>()
    }
    pub fn perform(
        self,
        prior: Option<&'n DocumentString>,
    ) -> Result<HtmlConstructionAttributeCandidateObservation<'a, 'e, 'n>, HtmlConstructionError>
    {
        let c = self.candidate;
        // Owner state, rather than Drop alone, prevents a forgotten cursor or
        // observation from making an unaccepted admission usable.
        if !c.edit.busy {
            c.edit.busy = true;
        }
        let mut allocation = false;
        let result = (|| {
            Ok(match c.phase {
                CandidatePhase::Target(index) => {
                    if index == c.edit.attributes().len() {
                        CandidatePhase::Prior(0)
                    } else if c.edit.attributes()[index]
                        .name()
                        .namespace()
                        .uri()
                        .is_none()
                    {
                        CandidatePhase::TargetName {
                            index,
                            name: NamePhase::LocalLength,
                        }
                    } else {
                        CandidatePhase::Target(index + 1)
                    }
                }
                CandidatePhase::TargetName { index, name } => {
                    let old = c.edit.attributes()[index].name().local_name();
                    match advance_comparison(name, None, old, None, c.name) {
                        ComparisonStep::Continue(name) => {
                            CandidatePhase::TargetName { index, name }
                        }
                        ComparisonStep::Different => CandidatePhase::Target(index + 1),
                        ComparisonStep::Equal => {
                            c.found = true;
                            CandidatePhase::Prior(0)
                        }
                    }
                }
                CandidatePhase::Prior(index) => {
                    if index == c.index {
                        if c.edit.decisions.len() == c.edit.decisions.capacity() {
                            CandidatePhase::Reserve
                        } else {
                            CandidatePhase::Witness
                        }
                    } else {
                        let prior = prior.ok_or(HtmlConstructionError::AttributeNameMismatch)?;
                        if c.edit.decisions[index].name.as_ptr()
                            != Arc::as_ptr(prior.shared_code_units())
                        {
                            return Err(HtmlConstructionError::AttributeNameMismatch);
                        }
                        c.prior_name = Some(prior);
                        CandidatePhase::PriorName {
                            index,
                            name: NamePhase::LocalLength,
                        }
                    }
                }
                CandidatePhase::PriorName { index, name } => {
                    match advance_comparison(
                        name,
                        None,
                        c.prior_name.expect("qualified prior"),
                        None,
                        c.name,
                    ) {
                        ComparisonStep::Continue(name) => CandidatePhase::PriorName { index, name },
                        ComparisonStep::Different => {
                            c.prior_name = None;
                            CandidatePhase::Prior(index + 1)
                        }
                        ComparisonStep::Equal => {
                            c.found = true;
                            c.prior_name = None;
                            if c.edit.decisions.len() == c.edit.decisions.capacity() {
                                CandidatePhase::Reserve
                            } else {
                                CandidatePhase::Witness
                            }
                        }
                    }
                }
                CandidatePhase::Reserve => {
                    let new_generation = c
                        .edit
                        .storage
                        .arena
                        .temporary_generation
                        .checked_add(1)
                        .ok_or(HtmlConstructionError::RepresentationExhausted)?;
                    let bytes = (c.index + 1)
                        .checked_mul(size_of::<NameDecision>())
                        .filter(|v| *v <= isize::MAX as usize)
                        .ok_or(HtmlConstructionError::RepresentationExhausted)?;
                    let _ = bytes;
                    let result = c.edit.decisions.try_reserve_exact(1);
                    let snapshot = c.edit.plan.as_mut().expect("plan report");
                    snapshot.capacity = c.edit.decisions.capacity();
                    snapshot.payload_bytes = snapshot.capacity * size_of::<NameDecision>();
                    if result.is_ok() {
                        snapshot.id.generation = new_generation;
                        c.edit.storage.arena.temporary_generation = new_generation;
                    }
                    allocation = true;
                    result.map_err(|_| HtmlConstructionError::AllocationFailed)?;
                    CandidatePhase::Witness
                }
                CandidatePhase::Witness => {
                    c.edit.decisions.push(NameDecision {
                        name: Arc::downgrade(c.name.shared_code_units()),
                        keep: !c.found,
                    });
                    c.edit.plan.as_mut().expect("plan report").initialized_len =
                        c.edit.decisions.len();
                    c.edit.selected += usize::from(!c.found);
                    CandidatePhase::Done
                }
                _ => unreachable!("pending candidate action"),
            })
        })();
        match result {
            Ok(next) => {
                c.next = Some(next);
                c.phase = CandidatePhase::Observing;
                Ok(HtmlConstructionAttributeCandidateObservation {
                    candidate: c,
                    allocation,
                })
            }
            Err(e) => {
                c.failure = Some(e.clone());
                Err(e)
            }
        }
    }
}
impl HtmlConstructionAttributeCandidateObservation<'_, '_, '_> {
    pub fn allocation(&self) -> Option<&HtmlConstructionAllocationSnapshot> {
        self.allocation.then(|| {
            self.candidate
                .edit
                .plan
                .as_ref()
                .expect("actual plan allocation")
        })
    }
    pub fn accept(self) {
        self.candidate.phase = self.candidate.next.take().expect("candidate observation");
    }
}

enum SlotInput<'a> {
    Reserve(usize),
    Append(&'a mut Vec<Attribute>),
    Replace {
        index: usize,
        incoming: &'a mut Option<Attribute>,
    },
    Swap(usize),
    Pop,
}
impl SlotInput<'_> {
    fn attribute(&self) -> Option<&Attribute> {
        match self {
            Self::Append(v) => v.last(),
            Self::Replace { incoming, .. } => incoming.as_ref(),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug)]
enum SlotPhase {
    Input,
    NextName,
    NamedIdentity,
    SameName,
    Compare { index: usize, name: NamePhase },
    Reserve,
    Layout,
    Unique,
    Observing,
    Ready,
}
pub struct HtmlConstructionAttributeSlot<'a> {
    edit: &'a mut HtmlConstructionAttributeEdit,
    input: SlotInput<'a>,
    phase: SlotPhase,
    next: Option<SlotPhase>,
    failure: Option<HtmlConstructionError>,
    finished: bool,
    usage: HtmlConstructionUsage,
    minimum_capacity: usize,
    plan_index: usize,
}
pub struct HtmlConstructionAttributeSlotAction<'a, 'e> {
    slot: &'a mut HtmlConstructionAttributeSlot<'e>,
}
pub struct HtmlConstructionAttributeSlotObservation<'a, 'e> {
    slot: &'a mut HtmlConstructionAttributeSlot<'e>,
    capacity: bool,
}
pub struct HtmlConstructionReadyAttributeSlot<'a> {
    slot: HtmlConstructionAttributeSlot<'a>,
}
impl Drop for HtmlConstructionAttributeSlot<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.edit.poison();
        }
    }
}
impl<'e> HtmlConstructionAttributeSlot<'e> {
    pub fn shared_attributes(&self) -> Result<&Arc<Vec<Attribute>>, HtmlConstructionError> {
        if self.failure.is_none()
            && (matches!(self.phase, SlotPhase::Ready)
                || matches!(self.next, Some(SlotPhase::Ready)))
        {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        self.edit.shared_attributes()
    }
    /// Actual provider result remains available even when reservation failed.
    pub fn capacity(&self) -> HtmlConstructionAttributeCapacity {
        self.edit.capacity
    }
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionAttributeSlotAction<'_, 'e>>, HtmlConstructionError> {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        if matches!(self.phase, SlotPhase::Observing) {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        Ok((!matches!(self.phase, SlotPhase::Ready))
            .then_some(HtmlConstructionAttributeSlotAction { slot: self }))
    }
    pub fn try_ready(
        owner: &mut Option<Self>,
    ) -> Result<HtmlConstructionReadyAttributeSlot<'e>, HtmlConstructionError> {
        let slot = owner
            .as_ref()
            .ok_or(HtmlConstructionError::OperationConsumed)?;
        if let Some(e) = &slot.failure {
            return Err(e.clone());
        }
        if !matches!(slot.phase, SlotPhase::Ready) {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        Ok(HtmlConstructionReadyAttributeSlot {
            slot: owner.take().expect("ready slot"),
        })
    }
}
fn plus(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_add(b)
        .ok_or(HtmlConstructionError::RepresentationExhausted)
}
fn minus(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_sub(b)
        .ok_or(HtmlConstructionError::ReleaseUnderflow)
}
fn identical_name(left: &QualifiedName, right: &QualifiedName) -> bool {
    let namespace = match (left.namespace().uri(), right.namespace().uri()) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a.shared_code_units(), b.shared_code_units()),
        _ => false,
    };
    namespace
        && Arc::ptr_eq(
            left.local_name().shared_code_units(),
            right.local_name().shared_code_units(),
        )
}
impl<'a, 'e> HtmlConstructionAttributeSlotAction<'a, 'e> {
    pub fn shared_attributes(&self) -> Result<&Arc<Vec<Attribute>>, HtmlConstructionError> {
        self.slot.shared_attributes()
    }
    pub fn description(&self) -> HtmlConstructionAttributeDescription<'_> {
        let s = &self.slot;
        match s.phase {
            SlotPhase::Compare { name, .. } => HtmlConstructionAttributeDescription::Compare {
                prior_index: None,
                work: comparison_work(name),
            },
            SlotPhase::NamedIdentity => {
                HtmlConstructionAttributeDescription::Inspect(HtmlConstructionAttributeWork {
                    record_visits: 3,
                    provider_calls: 2,
                    ..Default::default()
                })
            }
            SlotPhase::SameName => {
                HtmlConstructionAttributeDescription::Inspect(HtmlConstructionAttributeWork {
                    record_visits: 4,
                    provider_calls: 2,
                    ..Default::default()
                })
            }
            SlotPhase::Unique => {
                HtmlConstructionAttributeDescription::UniqueBacking(HtmlConstructionAttributeWork {
                    record_visits: 2,
                    provider_calls: 2,
                    ..Default::default()
                })
            }
            SlotPhase::Reserve => HtmlConstructionAttributeDescription::ReserveAttributes {
                before: s.edit.capacity,
                minimum_capacity: s.minimum_capacity,
                work: HtmlConstructionAttributeWork {
                    provider_calls: 3,
                    record_visits: 2,
                    record_writes: 1,
                    copied_bytes: size_of::<usize>(),
                    ..Default::default()
                },
            },
            SlotPhase::Input => {
                let counters = match s.input {
                    SlotInput::Reserve(_) | SlotInput::Replace { .. } => 1,
                    SlotInput::Append(_) => {
                        2 + usize::from(matches!(
                            s.edit.mode,
                            HtmlConstructionAttributeEditMode::NamedMerge { .. }
                        ))
                    }
                    SlotInput::Pop => 2,
                    SlotInput::Swap(_) => 0,
                };
                let busy = usize::from(!s.edit.busy);
                HtmlConstructionAttributeDescription::Inspect(HtmlConstructionAttributeWork {
                    record_visits: 8,
                    record_writes: counters + busy,
                    copied_bytes: counters * size_of::<usize>() + busy * size_of::<bool>(),
                    ..Default::default()
                })
            }
            SlotPhase::NextName => {
                HtmlConstructionAttributeDescription::Inspect(HtmlConstructionAttributeWork {
                    record_visits: 8,
                    // A discarded decision advances the retained plan ordinal.
                    record_writes: 1,
                    copied_bytes: size_of::<usize>(),
                    ..Default::default()
                })
            }
            SlotPhase::Layout => {
                let (copies, writes) = match s.input {
                    SlotInput::Append(_) => (2, 3),
                    SlotInput::Replace { .. } => (3, 3),
                    SlotInput::Swap(_) => (3, 2),
                    SlotInput::Pop => (1, 2),
                    SlotInput::Reserve(_) => (0, 0),
                };
                // Count only the fields actually written by commit, alongside
                // the original physical pop/push/replace/swap producers.
                let (counter_writes, counter_bytes) = match s.input {
                    SlotInput::Append(_) => {
                        if matches!(
                            s.edit.mode,
                            HtmlConstructionAttributeEditMode::NamedMerge { .. }
                        ) {
                            (3, 3 * size_of::<usize>()) // cached length, appended, next selected
                        } else {
                            (2, size_of::<usize>() + size_of::<bool>()) // cached length, used
                        }
                    }
                    SlotInput::Replace { .. } | SlotInput::Swap(_) => (1, size_of::<bool>()),
                    SlotInput::Pop => (2, size_of::<usize>() + size_of::<bool>()),
                    SlotInput::Reserve(_) => (0, 0),
                };
                let fixed_bytes =
                    counter_bytes + size_of::<HtmlConstructionUsage>() + 2 * size_of::<bool>();
                HtmlConstructionAttributeDescription::Commit {
                    usage: s.usage,
                    work: HtmlConstructionAttributeWork {
                        provider_calls: usize::from(!matches!(s.input, SlotInput::Reserve(_))),
                        copied_bytes: copies * size_of::<Attribute>() + fixed_bytes,
                        atomic_layout_bytes: (copies * size_of::<Attribute>())
                            .max(size_of::<HtmlConstructionUsage>())
                            .max(size_of::<usize>())
                            .max(size_of::<bool>()),
                        record_writes: writes + counter_writes + 3,
                        ..Default::default()
                    },
                }
            }
            SlotPhase::Observing | SlotPhase::Ready => unreachable!("pending slot action"),
        }
        .transitions::<SlotPhase>()
    }
    pub fn perform(
        self,
    ) -> Result<HtmlConstructionAttributeSlotObservation<'a, 'e>, HtmlConstructionError> {
        let s = self.slot;
        if !s.edit.busy {
            s.edit.busy = true;
        }
        let mut capacity = false;
        let result = (|| {
            Ok(match s.phase {
                SlotPhase::Input => match &s.input {
                    SlotInput::Reserve(additional) => {
                        s.minimum_capacity = plus(s.edit.capacity.initialized_len, *additional)?;
                        if s.minimum_capacity > s.edit.capacity.capacity {
                            SlotPhase::Reserve
                        } else {
                            SlotPhase::Layout
                        }
                    }
                    SlotInput::Append(v) => {
                        let attribute = v.last().ok_or(HtmlConstructionError::CapacityMismatch)?;
                        if s.edit.capacity.initialized_len == s.edit.capacity.capacity {
                            return Err(HtmlConstructionError::CapacityMismatch);
                        }
                        s.usage.attributes = plus(s.usage.attributes, 1)?;
                        s.usage.utf16_units =
                            plus(s.usage.utf16_units, attribute_units(attribute)?)?;
                        if matches!(
                            s.edit.mode,
                            HtmlConstructionAttributeEditMode::NamedMerge { .. }
                        ) {
                            s.plan_index = s.edit.next_selected;
                            SlotPhase::NextName
                        } else {
                            SlotPhase::Compare {
                                index: 0,
                                name: NamePhase::Pair,
                            }
                        }
                    }
                    SlotInput::Replace { index, incoming } => {
                        let old = s
                            .edit
                            .attributes()
                            .get(*index)
                            .ok_or(HtmlConstructionError::CapacityMismatch)?;
                        let new = incoming
                            .as_ref()
                            .ok_or(HtmlConstructionError::CapacityMismatch)?;
                        s.usage.utf16_units = plus(
                            minus(s.usage.utf16_units, attribute_units(old)?)?,
                            attribute_units(new)?,
                        )?;
                        SlotPhase::SameName
                    }
                    SlotInput::Swap(index) => {
                        if plus(*index, 1)? >= s.edit.capacity.initialized_len {
                            return Err(HtmlConstructionError::CapacityMismatch);
                        }
                        SlotPhase::Layout
                    }
                    SlotInput::Pop => {
                        let old = s
                            .edit
                            .attributes()
                            .last()
                            .ok_or(HtmlConstructionError::CapacityMismatch)?;
                        s.usage.attributes = minus(s.usage.attributes, 1)?;
                        s.usage.utf16_units = minus(s.usage.utf16_units, attribute_units(old)?)?;
                        SlotPhase::Layout
                    }
                },
                SlotPhase::NextName => {
                    if s.edit.appended == s.edit.selected {
                        return Err(HtmlConstructionError::AttributeOrderMismatch);
                    }
                    let decision = s
                        .edit
                        .decisions
                        .get(s.plan_index)
                        .ok_or(HtmlConstructionError::AttributeOrderMismatch)?;
                    if decision.keep {
                        SlotPhase::NamedIdentity
                    } else {
                        s.plan_index += 1;
                        SlotPhase::NextName
                    }
                }
                SlotPhase::NamedIdentity => {
                    let attribute = s.input.attribute().expect("checked incoming slot");
                    if attribute.name().namespace().uri().is_some()
                        || s.edit.decisions[s.plan_index].name.as_ptr()
                            != Arc::as_ptr(attribute.name().local_name().shared_code_units())
                    {
                        return Err(HtmlConstructionError::AttributeNameMismatch);
                    }
                    SlotPhase::Layout
                }
                SlotPhase::SameName => {
                    let SlotInput::Replace { index, .. } = s.input else {
                        unreachable!("replacement name check")
                    };
                    if identical_name(
                        s.edit.attributes()[index].name(),
                        s.input.attribute().expect("checked incoming").name(),
                    ) {
                        SlotPhase::Layout
                    } else {
                        SlotPhase::Compare {
                            index: 0,
                            name: NamePhase::Pair,
                        }
                    }
                }
                SlotPhase::Compare { index, name } => {
                    if index == s.edit.capacity.initialized_len {
                        SlotPhase::Layout
                    } else if matches!(s.input, SlotInput::Replace { index: replaced, .. } if index == replaced)
                    {
                        SlotPhase::Compare {
                            index: index + 1,
                            name: NamePhase::Pair,
                        }
                    } else {
                        let old = s.edit.attributes()[index].name();
                        let new = s.input.attribute().expect("checked incoming").name();
                        match advance_comparison(
                            name,
                            old.namespace().uri(),
                            old.local_name(),
                            new.namespace().uri(),
                            new.local_name(),
                        ) {
                            ComparisonStep::Continue(name) => SlotPhase::Compare { index, name },
                            ComparisonStep::Different => SlotPhase::Compare {
                                index: index + 1,
                                name: NamePhase::Pair,
                            },
                            ComparisonStep::Equal => {
                                return Err(HtmlConstructionError::DuplicateAttribute {
                                    first: index,
                                    duplicate: match s.input {
                                        SlotInput::Replace { index, .. } => index,
                                        _ => s.edit.capacity.initialized_len,
                                    },
                                });
                            }
                        }
                    }
                }
                SlotPhase::Reserve => {
                    let required_bytes = s
                        .minimum_capacity
                        .checked_mul(size_of::<Attribute>())
                        .filter(|v| *v <= isize::MAX as usize)
                        .ok_or(HtmlConstructionError::RepresentationExhausted)?;
                    let _ = required_bytes;
                    let additional = s.minimum_capacity - s.edit.capacity.initialized_len;
                    let backing = Arc::get_mut(s.edit.backing_mut())
                        .ok_or(HtmlConstructionError::SharedAllocationAliased)?;
                    let result = backing.try_reserve_exact(additional);
                    s.edit.capacity.capacity = backing.capacity();
                    capacity = true;
                    result.map_err(|_| HtmlConstructionError::AllocationFailed)?;
                    SlotPhase::Layout
                }
                SlotPhase::Layout => {
                    if matches!(s.input, SlotInput::Reserve(_)) {
                        SlotPhase::Ready
                    } else {
                        SlotPhase::Unique
                    }
                }
                SlotPhase::Unique => {
                    let backing = s.edit.shared_attributes()?;
                    if Arc::strong_count(backing) != 1 || Arc::weak_count(backing) != 0 {
                        return Err(HtmlConstructionError::SharedAllocationAliased);
                    }
                    SlotPhase::Ready
                }
                _ => unreachable!("pending slot action"),
            })
        })();
        match result {
            Ok(next) => {
                s.next = Some(next);
                s.phase = SlotPhase::Observing;
                Ok(HtmlConstructionAttributeSlotObservation { slot: s, capacity })
            }
            Err(e) => {
                s.failure = Some(e.clone());
                Err(e)
            }
        }
    }
}
impl HtmlConstructionAttributeSlotObservation<'_, '_> {
    pub fn shared_attributes(&self) -> Result<&Arc<Vec<Attribute>>, HtmlConstructionError> {
        self.slot.shared_attributes()
    }
    pub fn capacity(&self) -> Option<HtmlConstructionAttributeCapacity> {
        self.capacity.then_some(self.slot.edit.capacity)
    }
    pub fn accept(self) {
        self.slot.phase = self.slot.next.take().expect("slot observation");
    }
}
impl HtmlConstructionReadyAttributeSlot<'_> {
    pub fn commit(mut self) -> Option<Attribute> {
        let s = &mut self.slot;
        let removed = match &mut s.input {
            SlotInput::Reserve(_) => None,
            SlotInput::Append(incoming) => {
                Arc::get_mut(s.edit.backing_mut())
                    .expect("exclusive ready backing")
                    .push(incoming.pop().expect("ready actual last slot"));
                s.edit.capacity.initialized_len += 1;
                if matches!(
                    s.edit.mode,
                    HtmlConstructionAttributeEditMode::NamedMerge { .. }
                ) {
                    s.edit.appended += 1;
                    s.edit.next_selected = s.plan_index + 1;
                } else {
                    s.edit.used = true;
                }
                None
            }
            SlotInput::Replace { index, incoming } => {
                s.edit.used = true;
                Some(std::mem::replace(
                    &mut Arc::get_mut(s.edit.backing_mut()).expect("exclusive ready backing")
                        [*index],
                    incoming.take().expect("ready actual replacement"),
                ))
            }
            SlotInput::Swap(index) => {
                Arc::get_mut(s.edit.backing_mut())
                    .expect("exclusive ready backing")
                    .swap(*index, *index + 1);
                s.edit.used = true;
                None
            }
            SlotInput::Pop => {
                let value = Arc::get_mut(s.edit.backing_mut())
                    .expect("exclusive ready backing")
                    .pop()
                    .expect("ready last slot");
                s.edit.capacity.initialized_len -= 1;
                s.edit.used = true;
                Some(value)
            }
        };
        s.edit.storage.arena.usage = s.usage;
        s.edit.busy = false;
        s.finished = true;
        removed
    }
}

/// Cleanup owns the same initialized decision cells and their actual allocation.
pub struct HtmlConstructionAttributePlanRetirement {
    decisions: Vec<NameDecision>,
    snapshot: Option<HtmlConstructionAllocationSnapshot>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlConstructionAttributeRetirementDescription {
    NameWitness {
        auxiliary_entries: usize,
        provider_calls: usize,
    },
    Allocation,
}
pub struct HtmlConstructionAttributeRetirementAction<'a> {
    retirement: &'a mut HtmlConstructionAttributePlanRetirement,
}
impl HtmlConstructionAttributePlanRetirement {
    pub fn next_action(&mut self) -> Option<HtmlConstructionAttributeRetirementAction<'_>> {
        (!self.decisions.is_empty() || self.snapshot.is_some())
            .then_some(HtmlConstructionAttributeRetirementAction { retirement: self })
    }
}
impl HtmlConstructionAttributeRetirementAction<'_> {
    pub fn description(&self) -> HtmlConstructionAttributeRetirementDescription {
        if self.retirement.decisions.is_empty() {
            HtmlConstructionAttributeRetirementDescription::Allocation
        } else {
            HtmlConstructionAttributeRetirementDescription::NameWitness {
                auxiliary_entries: 1,
                provider_calls: 1,
            }
        }
    }
    pub fn allocation(&self) -> Option<&HtmlConstructionAllocationSnapshot> {
        self.retirement.snapshot.as_ref()
    }
    pub fn perform(self) -> Option<HtmlConstructionRetired> {
        if let Some(decision) = self.retirement.decisions.pop() {
            drop(decision);
            if let Some(snapshot) = self.retirement.snapshot.as_mut() {
                snapshot.initialized_len = self.retirement.decisions.len();
            }
            None
        } else {
            let snapshot = self.retirement.snapshot.take().expect("actual plan bank");
            drop(std::mem::take(&mut self.retirement.decisions));
            Some(HtmlConstructionRetired { snapshot })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attribute(name: impl Into<DocumentString>, value: &str) -> Attribute {
        Attribute::new(QualifiedName::new(Namespace::none(), None, name), value)
    }
    fn make_storage(attributes: Vec<Attribute>) -> (HtmlConstructionStorage, HtmlBuilderNode) {
        let mut initialization = Some(HtmlConstructionStorage::begin());
        while let Some(action) = initialization.as_mut().unwrap().next_action().unwrap() {
            action.perform().accept();
        }
        let storage = HtmlConstructionInitialization::try_ready(&mut initialization)
            .unwrap()
            .commit();
        let mut operation = Some(
            storage
                .prepare(HtmlConstructionCommand::Create {
                    kind: HtmlNodeKind::Element(HtmlElement::new(
                        QualifiedName::new(Namespace::none(), None, "root"),
                        OrderedAttributes::new(attributes).unwrap(),
                    )),
                })
                .unwrap_or_else(|_| panic!("owned create")),
        );
        while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        let HtmlConstructionCommit::Changed { storage, created } =
            HtmlConstructionOperation::try_ready(&mut operation)
                .unwrap()
                .commit()
        else {
            panic!("create")
        };
        (storage, created.unwrap())
    }
    fn make_edit(
        storage: HtmlConstructionStorage,
        target: HtmlBuilderNode,
        mode: HtmlConstructionAttributeEditMode,
    ) -> HtmlConstructionAttributeEdit {
        let mut edit = HtmlConstructionAttributeEdit::begin(storage, target, mode)
            .unwrap_or_else(|_| panic!("owned edit"));
        while let Some(action) = edit.next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        edit
    }
    fn slot(slot: HtmlConstructionAttributeSlot<'_>) -> Option<Attribute> {
        let mut slot = Some(slot);
        while let Some(action) = slot.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        HtmlConstructionAttributeSlot::try_ready(&mut slot)
            .unwrap()
            .commit()
    }
    fn plan(edit: &mut HtmlConstructionAttributeEdit, names: &[DocumentString]) -> Vec<usize> {
        let mut priors = Vec::new();
        for (index, name) in names.iter().enumerate() {
            let mut candidate = Some(edit.candidate(index, name).unwrap());
            while let Some(action) = candidate.as_mut().unwrap().next_action().unwrap() {
                let prior = action.description().prior_index();
                if let Some(prior) = prior {
                    priors.push(prior);
                }
                action
                    .perform(prior.map(|index| &names[index]))
                    .unwrap()
                    .accept();
            }
            HtmlConstructionAttributeCandidate::try_finish(&mut candidate).unwrap();
        }
        priors
    }
    fn retire(mut retirement: HtmlConstructionAttributePlanRetirement) -> (usize, usize) {
        let mut names = 0;
        let mut bytes = 0;
        while let Some(action) = retirement.next_action() {
            if matches!(
                action.description(),
                HtmlConstructionAttributeRetirementDescription::NameWitness { .. }
            ) {
                names += 1;
            }
            if let Some(receipt) = action.perform() {
                bytes = receipt.allocation().payload_bytes();
            }
        }
        (names, bytes)
    }
    fn finish(edit: HtmlConstructionAttributeEdit) -> HtmlConstructionStorage {
        let mut owner = Some(edit);
        let (storage, retirement) = HtmlConstructionAttributeEdit::try_finish(&mut owner)
            .unwrap()
            .into_parts();
        retire(retirement);
        assert!(owner.is_none());
        storage
    }

    struct UnchangedAttributes {
        backing: *const Vec<Attribute>,
        fields: Vec<(*const Vec<u16>, *const Vec<u16>)>,
        capacity: HtmlConstructionAttributeCapacity,
        usage: HtmlConstructionUsage,
        records: usize,
    }
    impl UnchangedAttributes {
        fn capture(edit: &HtmlConstructionAttributeEdit) -> Self {
            let attributes = edit.shared_attributes().unwrap();
            Self {
                backing: Arc::as_ptr(attributes),
                fields: attributes
                    .iter()
                    .map(|attribute| {
                        (
                            Arc::as_ptr(attribute.name().local_name().shared_code_units()),
                            Arc::as_ptr(attribute.value().shared_code_units()),
                        )
                    })
                    .collect(),
                capacity: edit.capacity(),
                usage: edit.storage_view().usage(),
                records: edit.storage_view().len(),
            }
        }
        fn assert_unchanged(
            &self,
            edit: &HtmlConstructionAttributeEdit,
            target: &HtmlBuilderNode,
            expected: &[(&str, &str)],
        ) {
            let attributes = edit.shared_attributes().unwrap();
            assert_eq!(Arc::as_ptr(attributes), self.backing);
            assert_eq!(edit.capacity(), self.capacity);
            assert_eq!(edit.storage_view().usage(), self.usage);
            assert_eq!(edit.storage_view().usage().attributes(), expected.len());
            let expected_units = 4 + expected
                .iter()
                .map(|(name, value)| name.encode_utf16().count() + value.encode_utf16().count())
                .sum::<usize>();
            assert_eq!(edit.storage_view().usage().utf16_units(), expected_units);
            assert_eq!(edit.storage_view().len(), self.records);
            let HtmlNodeKind::Element(element) = edit.storage_view().node(target).unwrap().kind()
            else {
                panic!("the original element remains addressable");
            };
            assert_eq!(element.name().local_name().to_utf8().unwrap(), "root");
            assert_eq!(element.attributes().as_slice(), attributes.as_slice());
            assert_eq!(attributes.len(), expected.len());
            for ((attribute, &(name, value)), &(name_backing, value_backing)) in
                attributes.iter().zip(expected).zip(&self.fields)
            {
                assert_eq!(attribute.name().local_name().to_utf8().unwrap(), name);
                assert_eq!(attribute.value().to_utf8().unwrap(), value);
                assert!(attribute.name().namespace().uri().is_none());
                assert!(attribute.name().prefix().is_none());
                assert_eq!(
                    Arc::as_ptr(attribute.name().local_name().shared_code_units()),
                    name_backing
                );
                assert_eq!(
                    Arc::as_ptr(attribute.value().shared_code_units()),
                    value_backing
                );
            }
        }
        fn assert_abort_and_retirement(
            &self,
            edit: HtmlConstructionAttributeEdit,
            target: HtmlBuilderNode,
            expected: &[(&str, &str)],
        ) {
            let (storage, retirement) = edit.abort().into_parts();
            assert_eq!(
                retire(retirement),
                (0, 0),
                "single-slot refusals own no merge plan"
            );
            let recovered = make_edit(
                storage,
                target.clone(),
                HtmlConstructionAttributeEditMode::Slots,
            );
            self.assert_unchanged(&recovered, &target, expected);
            let backing = Arc::downgrade(recovered.shared_attributes().unwrap());
            let storage = finish(recovered);
            assert_eq!(storage.usage(), self.usage);
            assert!(storage.node(&target).is_ok());
            drop(storage);
            assert!(
                backing.upgrade().is_none(),
                "cleanup releases the actual installed strong owner"
            );
        }
    }
    fn assert_slot_refused(
        slot: HtmlConstructionAttributeSlot<'_>,
        expected: HtmlConstructionError,
    ) {
        let mut owner = Some(slot);
        loop {
            let action = owner
                .as_mut()
                .unwrap()
                .next_action()
                .unwrap()
                .expect("the invalid edit cannot become ready");
            match action.perform() {
                Ok(observation) => observation.accept(),
                Err(error) => {
                    assert_eq!(error, expected);
                    break;
                }
            }
        }
        assert!(matches!(owner.as_mut().unwrap().next_action(), Err(error) if error == expected));
        assert!(
            matches!(HtmlConstructionAttributeSlot::try_ready(&mut owner), Err(error) if error == expected)
        );
        assert!(
            owner.is_some(),
            "typed refusal retains the same pending owner"
        );
    }

    #[test]
    fn replacement_colliding_with_another_slot_preserves_incoming_and_installed_prefix() {
        let (storage, target) = make_storage(vec![
            attribute("a", "one"),
            attribute("b", "two"),
            attribute("c", "three"),
        ]);
        let mut edit = make_edit(
            storage,
            target.clone(),
            HtmlConstructionAttributeEditMode::Slots,
        );
        let original = UnchangedAttributes::capture(&edit);
        let mut incoming = Some(attribute("b", "replacement"));
        let name = Arc::as_ptr(
            incoming
                .as_ref()
                .unwrap()
                .name()
                .local_name()
                .shared_code_units(),
        );
        let value = Arc::as_ptr(incoming.as_ref().unwrap().value().shared_code_units());
        assert_slot_refused(
            edit.replace(0, &mut incoming).unwrap(),
            HtmlConstructionError::DuplicateAttribute {
                first: 1,
                duplicate: 0,
            },
        );
        let incoming = incoming
            .as_ref()
            .expect("duplicate rejection cannot consume the replacement");
        assert_eq!(incoming.name().local_name().to_utf8().unwrap(), "b");
        assert_eq!(incoming.value().to_utf8().unwrap(), "replacement");
        assert!(incoming.name().namespace().uri().is_none());
        assert!(incoming.name().prefix().is_none());
        assert_eq!(
            Arc::as_ptr(incoming.name().local_name().shared_code_units()),
            name
        );
        assert_eq!(Arc::as_ptr(incoming.value().shared_code_units()), value);
        let expected = [("a", "one"), ("b", "two"), ("c", "three")];
        original.assert_unchanged(&edit, &target, &expected);
        original.assert_abort_and_retirement(edit, target, &expected);
    }

    #[test]
    fn invalid_replacement_index_preserves_exact_incoming_and_installed_attributes() {
        for index in [2, usize::MAX] {
            let (storage, target) =
                make_storage(vec![attribute("a", "one"), attribute("b", "two")]);
            let mut edit = make_edit(
                storage,
                target.clone(),
                HtmlConstructionAttributeEditMode::Slots,
            );
            let original = UnchangedAttributes::capture(&edit);
            let mut incoming = Some(attribute("new", "candidate"));
            let name = Arc::as_ptr(
                incoming
                    .as_ref()
                    .unwrap()
                    .name()
                    .local_name()
                    .shared_code_units(),
            );
            let value = Arc::as_ptr(incoming.as_ref().unwrap().value().shared_code_units());
            assert_slot_refused(
                edit.replace(index, &mut incoming).unwrap(),
                HtmlConstructionError::CapacityMismatch,
            );
            let incoming = incoming
                .as_ref()
                .expect("invalid index retains the actual incoming attribute");
            assert_eq!(incoming.name().local_name().to_utf8().unwrap(), "new");
            assert_eq!(incoming.value().to_utf8().unwrap(), "candidate");
            assert_eq!(
                Arc::as_ptr(incoming.name().local_name().shared_code_units()),
                name
            );
            assert_eq!(Arc::as_ptr(incoming.value().shared_code_units()), value);
            let expected = [("a", "one"), ("b", "two")];
            original.assert_unchanged(&edit, &target, &expected);
            original.assert_abort_and_retirement(edit, target, &expected);
        }
    }

    #[test]
    fn absent_replacement_input_preserves_existing_slot_and_usage() {
        let (storage, target) = make_storage(vec![attribute("a", "one"), attribute("b", "two")]);
        let mut edit = make_edit(
            storage,
            target.clone(),
            HtmlConstructionAttributeEditMode::Slots,
        );
        let original = UnchangedAttributes::capture(&edit);
        let mut incoming = None;
        assert_slot_refused(
            edit.replace(0, &mut incoming).unwrap(),
            HtmlConstructionError::CapacityMismatch,
        );
        assert!(incoming.is_none());
        let expected = [("a", "one"), ("b", "two")];
        original.assert_unchanged(&edit, &target, &expected);
        original.assert_abort_and_retirement(edit, target, &expected);
    }

    #[test]
    fn adjacent_swap_at_last_or_out_of_range_index_preserves_actual_order() {
        for index in [1, 2, 7] {
            let (storage, target) =
                make_storage(vec![attribute("a", "one"), attribute("b", "two")]);
            let mut edit = make_edit(
                storage,
                target.clone(),
                HtmlConstructionAttributeEditMode::Slots,
            );
            let original = UnchangedAttributes::capture(&edit);
            assert_slot_refused(
                edit.swap_adjacent(index).unwrap(),
                HtmlConstructionError::CapacityMismatch,
            );
            let expected = [("a", "one"), ("b", "two")];
            original.assert_unchanged(&edit, &target, &expected);
            original.assert_abort_and_retirement(edit, target, &expected);
        }
    }

    #[test]
    fn pop_on_empty_attributes_preserves_actual_backing_and_usage() {
        let (storage, target) = make_storage(Vec::new());
        let mut edit = make_edit(
            storage,
            target.clone(),
            HtmlConstructionAttributeEditMode::Slots,
        );
        let original = UnchangedAttributes::capture(&edit);
        assert_slot_refused(edit.pop().unwrap(), HtmlConstructionError::CapacityMismatch);
        original.assert_unchanged(&edit, &target, &[]);
        original.assert_abort_and_retirement(edit, target, &[]);
    }

    #[test]
    fn named_merge_keeps_first_source_value_and_checks_priors_after_installed_match() {
        let (storage, target) = make_storage(vec![attribute("a", "old")]);
        let original_backing = Arc::as_ptr(match storage.node(&target).unwrap().kind() {
            HtmlNodeKind::Element(v) => v.attributes().shared_attributes(),
            _ => unreachable!(),
        });
        let names = ["a", "b", "b", "a", "c"].map(DocumentString::from);
        let mut edit = make_edit(
            storage,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge {
                candidate_count: names.len(),
            },
        );
        let priors = plan(&mut edit, &names);
        // Candidate 3 matches installed a, but still compares prior lexical a.
        assert_eq!(priors, [0, 0, 1, 0, 0, 1, 2, 3]);
        let decisions: Vec<_> = (0..names.len())
            .map(|i| edit.decision(i).unwrap())
            .collect();
        assert_eq!(decisions, [false, true, false, false, true]);
        assert_eq!(edit.selected_count().unwrap(), 2);
        let mut incoming: Vec<_> = names
            .into_iter()
            .enumerate()
            .filter(|(i, _)| decisions[*i])
            .map(|(i, name)| attribute(name, if i == 1 { "first-b" } else { "c" }))
            .collect();
        incoming.reverse();
        slot(edit.reserve(2).unwrap());
        slot(edit.append(&mut incoming).unwrap());
        slot(edit.append(&mut incoming).unwrap());
        assert!(incoming.is_empty());
        assert_eq!(
            Arc::as_ptr(edit.shared_attributes().unwrap()),
            original_backing
        );
        assert_eq!(edit.storage_view().usage().attributes(), 3);
        let attrs = edit.attributes();
        assert_eq!(
            attrs
                .iter()
                .map(|v| v.name().local_name().to_utf8().unwrap())
                .collect::<Vec<_>>(),
            ["a", "b", "c"]
        );
        assert_eq!(attrs[1].value().to_utf8().unwrap(), "first-b");
        finish(edit);
    }

    #[test]
    fn missing_candidate_or_dropped_observation_cannot_finish_or_resume_group() {
        let (storage, target) = make_storage(Vec::new());
        let mut edit = Some(make_edit(
            storage,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 1 },
        ));
        assert!(matches!(
            HtmlConstructionAttributeEdit::try_finish(&mut edit),
            Err(HtmlConstructionError::OperationNotReady)
        ));
        let name = DocumentString::from("x");
        let mut candidate = edit.as_mut().unwrap().candidate(0, &name).unwrap();
        // No action admission: dropping the borrowed cursor closes the group.
        {
            let _unperformed = candidate.next_action().unwrap().unwrap();
        }
        drop(candidate);
        assert!(matches!(
            edit.as_mut().unwrap().candidate(0, &name),
            Err(HtmlConstructionError::OperationNotReady)
        ));
        let (_, retirement) = edit.take().unwrap().abort().into_parts();
        assert_eq!(retire(retirement).0, 0);

        let (storage, target) = make_storage(Vec::new());
        let mut edit = make_edit(
            storage,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 1 },
        );
        let mut candidate = Some(edit.candidate(0, &name).unwrap());
        loop {
            let action = candidate.as_mut().unwrap().next_action().unwrap().unwrap();
            let witness = matches!(
                action.description(),
                HtmlConstructionAttributeDescription::InitializeNameWitness(_)
            );
            let observation = action.perform(None).unwrap();
            if witness {
                break;
            }
            observation.accept();
        }
        assert!(matches!(
            HtmlConstructionAttributeCandidate::try_finish(&mut candidate),
            Err(HtmlConstructionError::OperationNotReady)
        ));
        drop(candidate);
        let (_, retirement) = edit.abort().into_parts();
        assert_eq!(retire(retirement).0, 1);
    }

    #[test]
    fn merge_rejects_equal_text_in_other_backing_and_keeps_incoming_slot() {
        let (storage, target) = make_storage(Vec::new());
        let names = [DocumentString::from("x")];
        let mut edit = make_edit(
            storage,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 1 },
        );
        plan(&mut edit, &names);
        slot(edit.reserve(1).unwrap());
        let mut incoming = vec![attribute("x", "new")];
        let mut pending = edit.append(&mut incoming).unwrap();
        let failure = loop {
            let action = pending.next_action().unwrap().unwrap();
            match action.perform() {
                Ok(observation) => observation.accept(),
                Err(error) => break error,
            }
        };
        assert_eq!(failure, HtmlConstructionError::AttributeNameMismatch);
        drop(pending);
        assert_eq!(incoming.len(), 1);
        assert!(edit.attributes().is_empty());
        let (_, retirement) = edit.abort().into_parts();
        assert_eq!(retire(retirement).0, 1);
    }

    #[test]
    fn checked_append_checks_actual_namespace_and_rejects_duplicate_without_effect() {
        let (storage, target) = make_storage(vec![attribute("x", "old")]);
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        slot(edit.reserve(1).unwrap());
        let storage = finish(edit);
        let target = storage.node_at(0).unwrap().handle();
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let mut incoming = vec![attribute("x", "new")];
        let mut pending = edit.append(&mut incoming).unwrap();
        let error = loop {
            match pending.next_action().unwrap().unwrap().perform() {
                Ok(v) => v.accept(),
                Err(e) => break e,
            }
        };
        assert_eq!(
            error,
            HtmlConstructionError::DuplicateAttribute {
                first: 0,
                duplicate: 1
            }
        );
        drop(pending);
        assert_eq!(incoming[0].value().to_utf8().unwrap(), "new");
        assert_eq!(edit.attributes()[0].value().to_utf8().unwrap(), "old");
        let (storage, retirement) = edit.abort().into_parts();
        retire(retirement);
        let target = storage.node_at(0).unwrap().handle();
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let mut incoming = vec![Attribute::new(
            QualifiedName::new(Namespace::svg(), None, "x"),
            "namespaced",
        )];
        slot(edit.append(&mut incoming).unwrap());
        assert_eq!(edit.attributes().len(), 2);
        assert!(edit.attributes()[1].name().namespace().uri().is_some());
        finish(edit);
    }

    #[test]
    fn ready_slot_hides_backing_and_dropped_ready_preserves_actual_source_and_target() {
        let (storage, target) = make_storage(vec![attribute("x", "old")]);
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let original_usage = edit.storage_view().usage();
        let mut incoming = Some(attribute("y", "new"));
        let mut pending = Some(edit.replace(0, &mut incoming).unwrap());
        while let Some(action) = pending.as_mut().unwrap().next_action().unwrap() {
            let observation = action.perform().unwrap();
            if matches!(observation.slot.next, Some(SlotPhase::Ready)) {
                assert!(matches!(
                    observation.shared_attributes(),
                    Err(HtmlConstructionError::OperationNotReady)
                ));
            }
            observation.accept();
        }
        assert!(matches!(
            pending.as_ref().unwrap().shared_attributes(),
            Err(HtmlConstructionError::OperationNotReady)
        ));
        let ready = HtmlConstructionAttributeSlot::try_ready(&mut pending).unwrap();
        drop(ready);
        drop(pending);
        assert!(incoming.is_some());
        assert_eq!(
            edit.attributes()[0].name().local_name().to_utf8().unwrap(),
            "x"
        );
        assert_eq!(edit.storage_view().usage(), original_usage);
        let (_, retirement) = edit.abort().into_parts();
        retire(retirement);
    }

    #[test]
    fn unadmitted_atomic_slot_write_keeps_input_and_prior_prefix() {
        let (storage, target) = make_storage(Vec::new());
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        slot(edit.reserve(1).unwrap());
        let storage = finish(edit);
        let target = storage.node_at(0).unwrap().handle();
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let mut incoming = vec![attribute("x", "new")];
        let mut pending = Some(edit.append(&mut incoming).unwrap());
        loop {
            let action = pending.as_mut().unwrap().next_action().unwrap().unwrap();
            if let HtmlConstructionAttributeDescription::Commit { work, .. } = action.description()
            {
                // One pop/push transfers exactly two public Attribute records.
                assert_eq!(work.atomic_layout_bytes, 2 * size_of::<Attribute>());
                // Three physical writes, cached length/used, usage/busy/finish,
                // and four admitted phase/successor writes.
                assert_eq!(work.record_writes, 12);
                let one_short = 2 * size_of::<Attribute>() - 1;
                assert!(work.atomic_layout_bytes > one_short);
                break;
            }
            action.perform().unwrap().accept();
        }
        assert!(matches!(
            HtmlConstructionAttributeSlot::try_ready(&mut pending),
            Err(HtmlConstructionError::OperationNotReady)
        ));
        drop(pending);
        assert_eq!(incoming.len(), 1);
        assert!(edit.attributes().is_empty());
        let (_, retirement) = edit.abort().into_parts();
        retire(retirement);
    }

    /// Attribute-edit usage counts local name, prefix, namespace URI and value.
    ///
    /// Expected totals are counted by hand from the UTF-16 unit rule for
    /// qualified names: the root element name `root` is 4 units, and each
    /// attribute adds its URI, prefix, local name and value lengths.
    #[test]
    fn qualified_attribute_edits_count_uri_prefix_local_name_and_value_units() {
        let qualified = |uri: &str, prefix: &str, local: &str, value: &str| {
            Attribute::new(
                QualifiedName::new(Namespace::from_uri(uri), Some(prefix.into()), local),
                value,
            )
        };
        // URI 5 + prefix 1 + local 1 + value 3 = 10.
        let (storage, target) = make_storage(vec![qualified("urn:a", "p", "k", "val")]);
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        slot(edit.reserve(1).unwrap());
        let storage = finish(edit);
        assert_eq!(storage.usage().utf16_units(), 4 + 10);
        let target = storage.node_at(0).unwrap().handle();
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        // URI 6 + prefix 2 + local 2 + value 1 = 11 replaces 10.
        let mut replacement = Some(qualified("urn:bb", "pq", "kk", "v"));
        assert!(slot(edit.replace(0, &mut replacement).unwrap()).is_some());
        assert_eq!(edit.storage_view().usage().utf16_units(), 4 + 11);
        let storage = finish(edit);
        let target = storage.node_at(0).unwrap().handle();
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        // URI 1 + prefix 1 + local 1 + empty value = 3 appended.
        let mut incoming = vec![qualified("u", "x", "n", "")];
        slot(edit.append(&mut incoming).unwrap());
        assert_eq!(edit.storage_view().usage().attributes(), 2);
        assert_eq!(edit.storage_view().usage().utf16_units(), 4 + 11 + 3);
        let storage = finish(edit);
        let target = storage.node_at(0).unwrap().handle();
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let popped = slot(edit.pop().unwrap()).unwrap();
        assert_eq!(popped.name().local_name().to_utf8().unwrap(), "n");
        assert_eq!(edit.storage_view().usage().attributes(), 1);
        assert_eq!(edit.storage_view().usage().utf16_units(), 4 + 11);
        finish(edit);
    }

    #[test]
    fn replacement_swap_and_pop_keep_uniqueness_and_update_real_usage() {
        let (storage, target) = make_storage(vec![attribute("a", "one"), attribute("b", "two")]);
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let mut replacement = Some(attribute("c", "three"));
        assert_eq!(
            slot(edit.replace(0, &mut replacement).unwrap())
                .unwrap()
                .name()
                .local_name()
                .to_utf8()
                .unwrap(),
            "a"
        );
        assert!(replacement.is_none());
        let storage = finish(edit);
        let target = storage.node_at(0).unwrap().handle();
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        slot(edit.swap_adjacent(0).unwrap());
        assert_eq!(
            edit.attributes()[0].name().local_name().to_utf8().unwrap(),
            "b"
        );
        let storage = finish(edit);
        let target = storage.node_at(0).unwrap().handle();
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        assert_eq!(
            slot(edit.pop().unwrap())
                .unwrap()
                .name()
                .local_name()
                .to_utf8()
                .unwrap(),
            "c"
        );
        assert_eq!(edit.storage_view().usage().attributes(), 1);
        assert_eq!(
            edit.storage_view().usage().utf16_units(),
            "root".len() + "b".len() + "two".len()
        );
        finish(edit);
    }

    #[test]
    fn foreign_target_refusal_preserves_same_storage_and_no_plan_allocation() {
        let (storage, _) = make_storage(Vec::new());
        let (_, foreign) = make_storage(Vec::new());
        let mut edit = HtmlConstructionAttributeEdit::begin(
            storage,
            foreign,
            HtmlConstructionAttributeEditMode::Slots,
        )
        .unwrap_or_else(|_| panic!("pending check"));
        assert!(matches!(
            edit.next_action().unwrap().unwrap().perform(),
            Err(HtmlConstructionError::ForeignBuilder)
        ));
        let (storage, retirement) = edit.abort().into_parts();
        assert_eq!(storage.len(), 1);
        assert_eq!(retire(retirement), (0, 0));
    }

    #[test]
    fn named_admission_rejects_namespace_change_and_expired_name_replacement() {
        for change_namespace in [true, false] {
            let (storage, target) = make_storage(Vec::new());
            let names = [DocumentString::from("x")];
            let mut edit = make_edit(
                storage,
                target,
                HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 1 },
            );
            plan(&mut edit, &names);
            slot(edit.reserve(1).unwrap());
            let name = if change_namespace {
                names.into_iter().next().unwrap()
            } else {
                drop(names);
                assert!(edit.decisions[0].name.upgrade().is_none());
                DocumentString::from("x")
            };
            let mut incoming = vec![Attribute::new(
                QualifiedName::new(
                    if change_namespace {
                        Namespace::svg()
                    } else {
                        Namespace::none()
                    },
                    None,
                    name,
                ),
                "new",
            )];
            let mut pending = edit.append(&mut incoming).unwrap();
            let error = loop {
                match pending.next_action().unwrap().unwrap().perform() {
                    Ok(v) => v.accept(),
                    Err(e) => break e,
                }
            };
            assert_eq!(error, HtmlConstructionError::AttributeNameMismatch);
            drop(pending);
            assert_eq!(incoming.len(), 1);
            assert!(edit.attributes().is_empty());
            let (_, retirement) = edit.abort().into_parts();
            assert_eq!(retire(retirement).0, 1);
        }
    }

    #[test]
    fn committed_prefix_rejects_replayed_name_and_retires_unconsumed_witnesses() {
        let (storage, target) = make_storage(Vec::new());
        let names = ["a", "b"].map(DocumentString::from);
        let mut edit = make_edit(
            storage,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 2 },
        );
        assert!(matches!(
            edit.candidate(1, &names[1]),
            Err(HtmlConstructionError::AttributeOrderMismatch)
        ));
        plan(&mut edit, &names);
        slot(edit.reserve(2).unwrap());
        let mut incoming: Vec<_> = names
            .into_iter()
            .rev()
            .map(|name| attribute(name, "value"))
            .collect();
        slot(edit.append(&mut incoming).unwrap());
        let mut replay = vec![edit.attributes()[0].clone()];
        let mut pending = edit.append(&mut replay).unwrap();
        let error = loop {
            match pending.next_action().unwrap().unwrap().perform() {
                Ok(v) => v.accept(),
                Err(e) => break e,
            }
        };
        assert_eq!(error, HtmlConstructionError::AttributeNameMismatch);
        drop(pending);
        assert_eq!(replay.len(), 1);
        assert_eq!(incoming.len(), 1);
        assert_eq!(edit.attributes().len(), 1);
        assert_eq!(
            edit.attributes()[0].name().local_name().to_utf8().unwrap(),
            "a"
        );
        let (storage, retirement) = edit.abort().into_parts();
        assert_eq!(storage.usage().attributes(), 1);
        assert_eq!(retire(retirement).0, 2);
    }

    #[test]
    fn single_slot_group_cannot_replay_and_initial_reserve_precedes_duplicate_check() {
        let (storage, target) = make_storage(vec![attribute("x", "old")]);
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        // Physical setup succeeds independently of any later name admission.
        slot(edit.reserve(1).unwrap());
        assert!(edit.capacity().capacity >= 2);
        let mut incoming = vec![attribute("y", "new")];
        slot(edit.append(&mut incoming).unwrap());
        let mut excess = vec![attribute("z", "excess")];
        assert!(matches!(
            edit.append(&mut excess),
            Err(HtmlConstructionError::OperationNotReady)
        ));
        assert_eq!(excess.len(), 1);
        assert_eq!(edit.attributes().len(), 2);
        finish(edit);
    }

    #[test]
    fn real_plan_layout_and_witness_work_replace_bool_cells_without_second_name_pass() {
        let (storage, target) = make_storage(vec![attribute("a", "old")]);
        let names = ["a", "b"].map(DocumentString::from);
        let mut edit = make_edit(
            storage,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 2 },
        );
        let mut inspected = 0;
        let mut witnesses = 0;
        for (index, name) in names.iter().enumerate() {
            let mut candidate = Some(edit.candidate(index, name).unwrap());
            while let Some(action) = candidate.as_mut().unwrap().next_action().unwrap() {
                let description = action.description();
                inspected += description.work().inspected_bytes;
                if let HtmlConstructionAttributeDescription::InitializeNameWitness(work) =
                    description
                {
                    assert_eq!(work.provider_calls, 1);
                    assert_eq!(work.auxiliary_entries, 1);
                    let fixed_field = size_of::<NameDecision>()
                        .max(size_of::<CandidatePhase>())
                        .max(size_of::<Option<CandidatePhase>>())
                        .max(size_of::<Option<HtmlConstructionError>>());
                    assert_eq!(work.atomic_layout_bytes, fixed_field);
                    // Slot/Vec length, snapshot length, selection, guarded
                    // finish flags, and four phase/successor writes.
                    assert_eq!(work.record_writes, 10);
                    witnesses += 1;
                }
                let prior = action.description().prior_index();
                action
                    .perform(prior.map(|index| &names[index]))
                    .unwrap()
                    .accept();
            }
            HtmlConstructionAttributeCandidate::try_finish(&mut candidate).unwrap();
        }
        // Original exact comparisons: old a:a, old a:b, prior a:b, one u16 each.
        assert_eq!(inspected, 3 * 2 * size_of::<u16>());
        assert_eq!(witnesses, 2);
        assert!(
            names
                .iter()
                .all(|name| Arc::strong_count(name.shared_code_units()) == 1)
        );
        let snapshot = edit.plan.as_ref().unwrap();
        assert_eq!(snapshot.initialized_len(), 2);
        assert_eq!(snapshot.element_size(), size_of::<NameDecision>());
        assert!(snapshot.element_size() > size_of::<bool>());
        let bytes = snapshot.payload_bytes();
        assert_eq!(bytes, snapshot.capacity() * snapshot.element_size());
        slot(edit.reserve(1).unwrap());
        let mut incoming = vec![attribute(names.into_iter().nth(1).unwrap(), "new")];
        let mut pending = Some(edit.append(&mut incoming).unwrap());
        while let Some(action) = pending.as_mut().unwrap().next_action().unwrap() {
            if let HtmlConstructionAttributeDescription::Commit { work, .. } = action.description()
            {
                assert_eq!(
                    work.record_writes, 13,
                    "named commit includes cached length and both plan progress fields"
                );
            }
            assert_eq!(
                action.description().work().inspected_bytes,
                0,
                "named append repeats no UTF-16 comparison"
            );
            action.perform().unwrap().accept();
        }
        HtmlConstructionAttributeSlot::try_ready(&mut pending)
            .unwrap()
            .commit();
        drop(pending);
        let mut owner = Some(edit);
        let (_, retirement) = HtmlConstructionAttributeEdit::try_finish(&mut owner)
            .unwrap()
            .into_parts();
        assert_eq!(retire(retirement), (2, bytes));
    }

    #[test]
    fn forgotten_cursor_cannot_bypass_unaccepted_witness_observation() {
        let (storage, target) = make_storage(Vec::new());
        let names = [DocumentString::from("x")];
        let mut edit = Some(make_edit(
            storage,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 1 },
        ));
        let mut candidate = edit.as_mut().unwrap().candidate(0, &names[0]).unwrap();
        loop {
            let action = candidate.next_action().unwrap().unwrap();
            let witness = matches!(
                action.description(),
                HtmlConstructionAttributeDescription::InitializeNameWitness(_)
            );
            let observation = action.perform(None).unwrap();
            if witness {
                break;
            }
            observation.accept();
        }
        std::mem::forget(candidate);
        assert!(matches!(
            HtmlConstructionAttributeEdit::try_finish(&mut edit),
            Err(HtmlConstructionError::OperationNotReady)
        ));
        assert!(matches!(
            edit.as_ref().unwrap().selected_count(),
            Err(HtmlConstructionError::OperationNotReady)
        ));
        let (_, retirement) = edit.take().unwrap().abort().into_parts();
        assert_eq!(retire(retirement).0, 1);
    }

    #[test]
    fn grown_plan_abort_and_recreation_issue_distinct_generations_on_same_target() {
        let (storage, target) = make_storage(vec![attribute("old", "retained")]);
        let mut storage = storage;
        let original_backing = Arc::as_ptr(match storage.node(&target).unwrap().kind() {
            HtmlNodeKind::Element(v) => v.attributes().shared_attributes(),
            _ => unreachable!(),
        });
        let mut previous = None;
        for _ in 0..2 {
            let names = ["a", "b", "c"].map(DocumentString::from);
            let mut edit = make_edit(
                storage,
                target.clone(),
                HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 3 },
            );
            let mut reserves = 0;
            for (index, name) in names.iter().enumerate() {
                let mut candidate = Some(edit.candidate(index, name).unwrap());
                while let Some(action) = candidate.as_mut().unwrap().next_action().unwrap() {
                    let description = action.description();
                    reserves += usize::from(matches!(
                        description,
                        HtmlConstructionAttributeDescription::ReservePlan { .. }
                    ));
                    let prior = description.prior_index();
                    action.perform(prior.map(|i| &names[i])).unwrap().accept();
                }
                HtmlConstructionAttributeCandidate::try_finish(&mut candidate).unwrap();
            }
            assert!(reserves >= 2, "exercise multiple actual plan allocations");
            let report = edit.plan.as_ref().unwrap().clone();
            if let Some(previous) = &previous {
                assert!(report.id().same_owner_role(previous));
                assert_ne!(report.id(), previous);
            }
            assert_eq!(
                Arc::as_ptr(edit.shared_attributes().unwrap()),
                original_backing
            );
            assert_eq!(edit.attributes().len(), 1);
            assert_eq!(edit.attributes()[0].value().to_utf8().unwrap(), "retained");
            let (recovered, mut retirement) = edit.abort().into_parts();
            let mut released = None;
            while let Some(action) = retirement.next_action() {
                if let Some(receipt) = action.perform() {
                    released = Some(receipt);
                }
            }
            let receipt = released.expect("the actual grown plan is retired");
            assert_eq!(receipt.allocation().id(), report.id());
            assert_eq!(receipt.allocation().capacity(), report.capacity());
            previous = Some(report.id().clone());
            storage = recovered;
        }
        assert_eq!(storage.usage().attributes(), 1);
    }
}
