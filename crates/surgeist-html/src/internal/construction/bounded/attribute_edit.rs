//! Complete, exclusive edits of the actual ordered attribute backing.

use super::operations::attribute_units;
use super::*;
use crate::internal::values::Attribute;
use crate::internal::values::bounded::{
    ComparisonStep, Phase as NamePhase, advance_comparison, comparison_layout,
};
use crate::internal::values::name_index::{
    Binding, CLEAR_CHUNK, HtmlAttributeNameIndex, HtmlNameIndexProbe, HtmlNameIndexStep,
    HtmlNameIndexVacancy, Occupant, entry, occupant, tag,
};

/// Which protocol an attribute edit follows. It is fixed when the edit begins.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlConstructionAttributeEditMode {
    /// Plans a merge of named candidate attributes into the element. Each candidate is kept
    /// only if no existing attribute and no earlier candidate has the same name; the kept
    /// candidates are then appended in order.
    NamedMerge {
        /// The number of candidates that will be planned, in order, before any append.
        candidate_count: usize,
    },
    /// Positional edits (`reserve`, `append`, `replace`, `swap_adjacent`, `pop`). Each stored
    /// attribute must keep an expanded name distinct from every other attribute, or the step is
    /// refused with a duplicate-attribute error.
    Slots,
}

/// Actual public-payload vector dimensions; no allocation or release authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlConstructionAttributeCapacity {
    /// The number of attributes currently stored.
    pub initialized_len: usize,
    /// The reserved attribute capacity; at least `initialized_len`.
    pub capacity: usize,
}
impl HtmlConstructionAttributeCapacity {
    /// The size in bytes of one stored attribute.
    pub fn element_size(self) -> usize {
        size_of::<Attribute>()
    }
    /// The alignment in bytes of one stored attribute.
    pub fn element_alignment(self) -> usize {
        align_of::<Attribute>()
    }
}

/// Fixed bounds admitted before the action, excluding allocator internals.
/// Comparison actions conservatively admit retained flags whose writes depend
/// on the comparison outcome, before reading that outcome.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlConstructionAttributeWork {
    /// The retained records the action reads or updates.
    pub record_visits: usize,
    /// The provider requests the action makes.
    pub provider_calls: usize,
    /// The bytes of name text the action compares without copying.
    pub inspected_bytes: usize,
    /// The bytes the action copies, including bookkeeping values it moves.
    pub copied_bytes: usize,
    /// The record fields the action writes.
    pub record_writes: usize,
    /// Largest indivisible physical transfer or individual fixed field write.
    /// This is a field-wise atomic maximum, not the sum of copied bytes.
    pub atomic_layout_bytes: usize,
    /// The name-witness entries the action creates or destroys. Only planning and plan cleanup use
    /// it.
    pub auxiliary_entries: usize,
}

/// The fixed next step of an attribute edit, with the work the consumer admits before running it.
#[derive(Debug)]
pub enum HtmlConstructionAttributeDescription<'a> {
    /// Reads retained records to qualify the target element or an edit operand.
    Inspect(HtmlConstructionAttributeWork),
    /// Qualifies that the attribute backing is uniquely owned before the step modifies it.
    UniqueBacking(HtmlConstructionAttributeWork),
    /// Prepares the allocation snapshot of the merge plan or the next reservation. It allocates
    /// nothing.
    PrepareAllocationReport(HtmlConstructionAttributeWork),
    /// Compares one stretch of a candidate's name with an existing attribute name or an earlier
    /// candidate's name.
    Compare {
        /// In candidate planning, the index of the earlier candidate whose admitted name the
        /// consumer must pass to `perform`. `None` when no earlier name is needed.
        prior_index: Option<usize>,
        /// The work the step costs.
        work: HtmlConstructionAttributeWork,
    },
    /// Reserves capacity in the merge plan so that one more decision fits.
    ReservePlan {
        /// The plan's allocation, as reported before the step.
        allocation: &'a HtmlConstructionAllocationSnapshot,
        /// The least plan capacity that suffices.
        minimum_capacity: usize,
        /// The work the step costs.
        work: HtmlConstructionAttributeWork,
    },
    /// Records the candidate's decision and a weak witness of its name in the merge plan.
    InitializeNameWitness(HtmlConstructionAttributeWork),
    /// Reserves capacity in the attribute backing.
    ReserveAttributes {
        /// The backing's dimensions before the reservation.
        before: HtmlConstructionAttributeCapacity,
        /// The least capacity that suffices.
        minimum_capacity: usize,
        /// The work the reservation costs.
        work: HtmlConstructionAttributeWork,
    },
    /// Describes the prepaid writes of the positional edit and the new usage. Performing this
    /// action applies nothing; `HtmlConstructionReadyAttributeSlot::commit` applies the writes.
    Commit {
        /// The work the commit costs.
        work: HtmlConstructionAttributeWork,
        /// The usage the edit leaves in place.
        usage: HtmlConstructionUsage,
    },
}
impl HtmlConstructionAttributeDescription<'_> {
    fn work_mut(&mut self) -> &mut HtmlConstructionAttributeWork {
        match self {
            Self::Inspect(v)
            | Self::UniqueBacking(v)
            | Self::PrepareAllocationReport(v)
            | Self::InitializeNameWitness(v) => v,
            Self::Compare { work, .. }
            | Self::ReservePlan { work, .. }
            | Self::ReserveAttributes { work, .. }
            | Self::Commit { work, .. } => work,
        }
    }
    /// A setup action stores the stage it reaches, holds the edit with `observing`, and accept
    /// releases it again; a failed action stores its typed failure instead. Admit the larger
    /// concrete path.
    fn stage_transitions(mut self) -> Self {
        let work = self.work_mut();
        work.record_writes += 3;
        work.copied_bytes += (size_of::<EditStage>() + 2 * size_of::<bool>())
            .max(size_of::<Option<HtmlConstructionError>>());
        work.atomic_layout_bytes = work
            .atomic_layout_bytes
            .max(size_of::<EditStage>())
            .max(size_of::<Option<HtmlConstructionError>>());
        self
    }
    fn transitions<T>(mut self) -> Self {
        let work = self.work_mut();
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
    /// The work the consumer admits for this step, whichever step it is.
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
    /// For a name comparison against an earlier candidate, that candidate's index; `None` for
    /// every other step.
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
/// What the edit has established, which is exactly the data its next step needs. The setup
/// actions advance it when they run; `observing` then holds the edit until the observation is
/// accepted, so a dropped observation leaves the established data in place.
#[derive(Debug)]
enum EditStage {
    /// The target element is not qualified yet.
    Target,
    /// The target is a qualified element; a merge plan report is pending.
    Report { target: usize },
    /// Positional edits of the qualified element may begin.
    Slots { target: usize },
    /// Planning and appending of a named merge may begin; `plan` is the issued plan report.
    Merge {
        target: usize,
        plan: HtmlConstructionAllocationSnapshot,
    },
}
impl EditStage {
    /// The qualified element's ordinal, once the first action has qualified it.
    fn target(&self) -> Option<usize> {
        match self {
            Self::Target => None,
            Self::Report { target } | Self::Slots { target } | Self::Merge { target, .. } => {
                Some(*target)
            }
        }
    }
    fn is_ready(&self) -> bool {
        matches!(self, Self::Slots { .. } | Self::Merge { .. })
    }
    fn plan(&self) -> Option<&HtmlConstructionAllocationSnapshot> {
        match self {
            Self::Merge { plan, .. } => Some(plan),
            _ => None,
        }
    }
}
/// The attributes of the qualified element `target`. The first action fails unless the target is
/// an element and the arena is exclusively owned by the edit afterwards, so the not-ready error is
/// the only one a target that is not a qualified element can produce.
fn element_attributes(
    storage: &HtmlConstructionStorage,
    target: usize,
) -> Result<&[Attribute], HtmlConstructionError> {
    match storage.arena.records.get(target).map(|record| &record.kind) {
        Some(HtmlNodeKind::Element(v)) => Ok(v.attributes.shared_attributes().as_slice()),
        _ => Err(HtmlConstructionErrorKind::OperationNotReady.into()),
    }
}

/// Owns the arena and its only merge plan; no name admission escapes this owner.
#[derive(Debug)]
pub struct HtmlConstructionAttributeEdit {
    storage: HtmlConstructionStorage,
    element: HtmlBuilderNode,
    mode: HtmlConstructionAttributeEditMode,
    stage: EditStage,
    observing: bool,
    failure: Option<HtmlConstructionError>,
    busy: bool,
    used: bool,
    capacity: HtmlConstructionAttributeCapacity,
    decisions: Vec<NameDecision>,
    selected: usize,
    appended: usize,
    next_selected: usize,
    index: Option<HtmlAttributeNameIndex>,
}
/// The result of a finished or abandoned edit.
#[derive(Debug)]
pub struct HtmlConstructionAttributeCompletion {
    storage: HtmlConstructionStorage,
    retirement: HtmlConstructionAttributePlanRetirement,
    index: Option<HtmlAttributeNameIndex>,
}
impl HtmlConstructionAttributeCompletion {
    /// Takes back the name index that the edit held, whatever its contents, so that the consumer
    /// can keep it for the next edit or release its storage. `None` when the edit held none.
    pub fn take_index(&mut self) -> Option<HtmlAttributeNameIndex> {
        self.index.take()
    }

    /// Returns the storage and the retirement owner for the merge plan. Cleanup the plan with
    /// `HtmlConstructionAttributePlanRetirement::next_action`.
    pub fn into_parts(
        self,
    ) -> (
        HtmlConstructionStorage,
        HtmlConstructionAttributePlanRetirement,
    ) {
        (self.storage, self.retirement)
    }
}
/// One pending action of an edit's own setup. It borrows the edit exclusively; `perform` consumes
/// it.
#[derive(Debug)]
pub struct HtmlConstructionAttributeAction<'a> {
    edit: &'a mut HtmlConstructionAttributeEdit,
    pending: SetupAction,
}
/// The setup action that the edit is about to issue.
#[derive(Clone, Copy, Debug)]
enum SetupAction {
    /// Qualify the target element.
    Target,
    /// Prepare the merge plan's report.
    Report,
}
/// The observation after a setup action. It borrows the edit exclusively; `accept` resumes it.
#[derive(Debug)]
pub struct HtmlConstructionAttributeObservation<'a> {
    edit: &'a mut HtmlConstructionAttributeEdit,
}
impl HtmlConstructionAttributeEdit {
    /// Starts an attribute edit of `element` in `mode`. Allocation-free and infallible: the first
    /// action qualifies that the element exists and is an element, and `abort` hands the storage
    /// back.
    pub fn begin(
        storage: HtmlConstructionStorage,
        element: HtmlBuilderNode,
        mode: HtmlConstructionAttributeEditMode,
    ) -> Self {
        Self {
            storage,
            element,
            mode,
            stage: EditStage::Target,
            observing: false,
            failure: None,
            busy: false,
            used: false,
            capacity: HtmlConstructionAttributeCapacity::default(),
            decisions: Vec::new(),
            selected: 0,
            appended: 0,
            next_selected: 0,
            index: None,
        }
    }

    /// Lends the edit a name index. Appends to an element, and the planning of a merge, then
    /// find a duplicate name by probing the few names that share a hash, so an edit that adds
    /// `n` names costs work linear in `n` and their length instead of quadratic. An index with
    /// too little capacity for the names (see [`HtmlAttributeNameIndex::slots_required`]) is
    /// ignored by the step that needs more, which then compares every name.
    ///
    /// An index kept from an earlier edit of the same element stays usable while nothing but
    /// appends through an index changed that element. Any other change to it, or an edit of
    /// another element, makes the next step that needs the index rebuild it first. The index is
    /// returned by [`HtmlConstructionAttributeCompletion::take_index`].
    pub fn adopt_index(&mut self, mut index: HtmlAttributeNameIndex) {
        if matches!(
            self.mode,
            HtmlConstructionAttributeEditMode::NamedMerge { .. }
        ) {
            index.bind(Binding::Unbound);
        }
        self.index = Some(index);
    }
    /// The construction storage as it stands, for read-only inspection.
    pub fn storage_view(&self) -> &HtmlConstructionStorage {
        &self.storage
    }
    /// The element's shared attribute backing. Returns the not-ready error until the first action
    /// has qualified the target. That action fails for a target that is not an element, so the
    /// target is never recorded as qualified and this method returns not-ready, not
    /// wrong-node-kind, in that case.
    pub fn shared_attributes(&self) -> Result<&Arc<Vec<Attribute>>, HtmlConstructionError> {
        let target = self
            .stage
            .target()
            .ok_or(HtmlConstructionErrorKind::OperationNotReady)?;
        match self
            .storage
            .arena
            .records
            .get(target)
            .map(|record| &record.kind)
        {
            Some(HtmlNodeKind::Element(v)) => Ok(v.attributes.shared_attributes()),
            _ => Err(HtmlConstructionErrorKind::OperationNotReady.into()),
        }
    }
    /// The attribute backing's dimensions as last observed.
    pub fn capacity(&self) -> HtmlConstructionAttributeCapacity {
        self.capacity
    }
    fn attributes(&self) -> Result<&[Attribute], HtmlConstructionError> {
        Ok(self.shared_attributes()?.as_slice())
    }
    fn backing_mut(&mut self) -> Result<&mut Arc<Vec<Attribute>>, HtmlConstructionError> {
        let target = self
            .stage
            .target()
            .ok_or(HtmlConstructionErrorKind::OperationNotReady)?;
        match self
            .storage
            .arena
            .records
            .get_mut(target)
            .map(|record| &mut record.kind)
        {
            Some(HtmlNodeKind::Element(v)) => Ok(v.attributes.construction_backing_mut()),
            _ => Err(HtmlConstructionErrorKind::OperationNotReady.into()),
        }
    }
    /// The element's attribute vector for a ready slot's commit. The slot's `Unique` action
    /// proved, under this edit's exclusive ownership of the storage, that the target is an
    /// element whose backing has no other strong or weak owner, and nothing runs between that
    /// proof and the commit that could change it.
    fn ready_backing(&mut self) -> &mut Vec<Attribute> {
        self.backing_mut()
            .ok()
            .and_then(Arc::get_mut)
            .expect("ready slot proved an exclusive element backing")
    }
    fn check_ready(&self) -> Result<(), HtmlConstructionError> {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        if !self.stage.is_ready() || self.observing || self.busy {
            return Err(HtmlConstructionErrorKind::OperationNotReady.into());
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
    /// The next setup action, or `Ok(None)` when planning and positional edits may begin. It
    /// returns the latched failure, or the not-ready error while an observation, candidate or slot
    /// is outstanding.
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionAttributeAction<'_>>, HtmlConstructionError> {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        if self.busy || self.observing {
            return Err(HtmlConstructionErrorKind::OperationNotReady.into());
        }
        let pending = match self.stage {
            EditStage::Target => SetupAction::Target,
            EditStage::Report { .. } => SetupAction::Report,
            EditStage::Slots { .. } | EditStage::Merge { .. } => return Ok(None),
        };
        Ok(Some(HtmlConstructionAttributeAction {
            edit: self,
            pending,
        }))
    }
    /// Starts planning candidate `index` whose name is `name`. Candidates are planned in order
    /// from 0, and `index` must equal the number planned so far. Only in `NamedMerge` mode;
    /// `Slots` mode returns the attribute-order-mismatch error.
    pub fn candidate<'e, 'n>(
        &'e mut self,
        index: usize,
        name: &'n DocumentString,
    ) -> Result<HtmlConstructionAttributeCandidate<'e, 'n>, HtmlConstructionError> {
        self.check_ready()?;
        let HtmlConstructionAttributeEditMode::NamedMerge { candidate_count } = self.mode else {
            return Err(HtmlConstructionErrorKind::AttributeOrderMismatch.into());
        };
        if index != self.decisions.len() || index >= candidate_count {
            return Err(HtmlConstructionErrorKind::AttributeOrderMismatch.into());
        }
        let target = match &self.stage {
            EditStage::Merge { target, .. } => *target,
            _ => return Err(HtmlConstructionErrorKind::OperationNotReady.into()),
        };
        let existing = element_attributes(&self.storage, target)?.len();
        let mut phase = CandidatePhase::Target(0);
        if let Some(held) = self.index.as_mut()
            && let Some(geometry) = existing
                .checked_add(candidate_count)
                .and_then(|entries| held.geometry_for(entries))
        {
            if held.binding() == (Binding::Merge { target }) {
                phase = CandidatePhase::IndexHashLength {
                    subject: Subject::Incoming,
                };
            } else {
                held.begin_clear(geometry);
                phase = CandidatePhase::IndexClear;
            }
        }
        let EditStage::Merge { plan, .. } = &mut self.stage else {
            return Err(HtmlConstructionErrorKind::OperationNotReady.into());
        };
        Ok(HtmlConstructionAttributeCandidate {
            storage: &mut self.storage,
            busy: &mut self.busy,
            decisions: &mut self.decisions,
            selected: &mut self.selected,
            plan,
            target,
            index,
            name,
            phase,
            found: false,
            complete: false,
            failure: None,
            table: &mut self.index,
        })
    }
    /// Whether the planned candidate `index` is kept: `true` if no existing attribute and no
    /// earlier candidate has the same name.
    pub fn decision(&self, index: usize) -> Result<bool, HtmlConstructionError> {
        self.check_ready()?;
        self.decisions
            .get(index)
            .map(|v| v.keep)
            .ok_or(HtmlConstructionErrorKind::AttributeOrderMismatch.error())
    }
    /// The number of kept candidates. Available once every candidate is planned.
    pub fn selected_count(&self) -> Result<usize, HtmlConstructionError> {
        self.check_ready()?;
        if !self.planning_complete() {
            return Err(HtmlConstructionErrorKind::OperationNotReady.into());
        }
        Ok(self.selected)
    }
    fn slot<'a>(
        &'a mut self,
        input: SlotInput<'a>,
    ) -> Result<HtmlConstructionAttributeSlot<'a>, HtmlConstructionError> {
        self.check_ready()?;
        if !self.planning_complete() || self.used {
            return Err(HtmlConstructionErrorKind::OperationNotReady.into());
        }
        if matches!(
            self.mode,
            HtmlConstructionAttributeEditMode::NamedMerge { .. }
        ) && !matches!(input, SlotInput::Append(_) | SlotInput::Reserve(_))
        {
            return Err(HtmlConstructionErrorKind::AttributeOrderMismatch.into());
        }
        let usage = self.storage.usage();
        Ok(HtmlConstructionAttributeSlot {
            edit: self,
            input,
            phase: SlotPhase::Input,
            failure: None,
            finished: false,
            usage,
            minimum_capacity: 0,
            plan_index: 0,
            insert_slot: None,
        })
    }
    /// Starts a step that ensures the backing can hold `additional` more attributes.
    pub fn reserve(
        &mut self,
        additional: usize,
    ) -> Result<HtmlConstructionAttributeSlot<'_>, HtmlConstructionError> {
        self.slot(SlotInput::Reserve(additional))
    }
    /// Starts a step that appends the last element of `incoming`, which is removed from it on
    /// commit. The backing needs a free slot, so `reserve` first. In `Slots` mode the name must be
    /// unique; in `NamedMerge` mode it must be the next kept candidate.
    pub fn append<'a>(
        &'a mut self,
        incoming: &'a mut Vec<Attribute>,
    ) -> Result<HtmlConstructionAttributeSlot<'a>, HtmlConstructionError> {
        self.slot(SlotInput::Append(incoming))
    }
    /// Starts a step that replaces the attribute at `index` with the attribute in `incoming`,
    /// which is taken on commit. The replacement's name must remain unique. Only in `Slots` mode.
    pub fn replace<'a>(
        &'a mut self,
        index: usize,
        incoming: &'a mut Option<Attribute>,
    ) -> Result<HtmlConstructionAttributeSlot<'a>, HtmlConstructionError> {
        self.slot(SlotInput::Replace { index, incoming })
    }
    /// Starts a step that swaps the attributes at `index` and `index + 1`. Only in `Slots` mode.
    pub fn swap_adjacent(
        &mut self,
        index: usize,
    ) -> Result<HtmlConstructionAttributeSlot<'_>, HtmlConstructionError> {
        self.slot(SlotInput::Swap(index))
    }
    /// Starts a step that removes the last attribute, which commit returns. Only in `Slots` mode.
    pub fn pop(&mut self) -> Result<HtmlConstructionAttributeSlot<'_>, HtmlConstructionError> {
        self.slot(SlotInput::Pop)
    }
    /// Consumes a finished edit. Requires all planning done and every kept candidate appended.
    /// Returns the latched failure or the not-ready error; either way the edit is handed back
    /// unchanged.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn try_finish(
        self,
    ) -> Result<HtmlConstructionAttributeCompletion, HtmlConstructionRefusal<Self>> {
        if let Err(error) = self.check_ready() {
            return Err(HtmlConstructionRefusal::new(self, error));
        }
        if !self.planning_complete() || self.appended != self.selected {
            return Err(HtmlConstructionRefusal::new(
                self,
                HtmlConstructionErrorKind::OperationNotReady,
            ));
        }
        Ok(self.abort())
    }
    /// Abandons the edit and returns the storage and the retirement owner for the merge plan.
    pub fn abort(self) -> HtmlConstructionAttributeCompletion {
        HtmlConstructionAttributeCompletion {
            index: self.index,
            storage: self.storage,
            retirement: HtmlConstructionAttributePlanRetirement {
                decisions: self.decisions,
                snapshot: match self.stage {
                    EditStage::Merge { plan, .. } => Some(plan),
                    _ => None,
                },
            },
        }
    }
}
impl<'a> HtmlConstructionAttributeAction<'a> {
    /// Describes the pending setup action's work for the consumer to admit.
    pub fn description(&self) -> HtmlConstructionAttributeDescription<'_> {
        match self.pending {
            SetupAction::Target => {
                // The capacity is the action's own write; the stage it reaches is charged with
                // the transition.
                HtmlConstructionAttributeDescription::Inspect(HtmlConstructionAttributeWork {
                    record_visits: 5,
                    provider_calls: 2,
                    record_writes: 1,
                    copied_bytes: size_of::<HtmlConstructionAttributeCapacity>(),
                    atomic_layout_bytes: size_of::<HtmlConstructionAttributeCapacity>(),
                    ..Default::default()
                })
            }
            SetupAction::Report => {
                // The generation counter is the action's own write; the plan report it issues is
                // stored with the stage and charged with the transition.
                HtmlConstructionAttributeDescription::PrepareAllocationReport(
                    HtmlConstructionAttributeWork {
                        record_visits: 3,
                        provider_calls: 1,
                        copied_bytes: size_of::<u64>(),
                        atomic_layout_bytes: size_of::<HtmlConstructionAllocationSnapshot>(),
                        record_writes: 1,
                        ..Default::default()
                    },
                )
            }
        }
        .stage_transitions()
    }
    /// Performs the setup action. The observation must be accepted before the next one.
    pub fn perform(
        self,
    ) -> Result<HtmlConstructionAttributeObservation<'a>, HtmlConstructionError> {
        let edit = self.edit;
        let result = (|| -> Result<_, HtmlConstructionError> {
            Ok(match &edit.stage {
                EditStage::Target => {
                    let target = edit.storage.position(&edit.element)?;
                    let HtmlNodeKind::Element(v) = &edit.storage.arena.records[target].kind else {
                        return Err(HtmlConstructionErrorKind::WrongNodeKind.into());
                    };
                    edit.capacity = HtmlConstructionAttributeCapacity {
                        initialized_len: v.attributes.len(),
                        capacity: v.attributes.shared_attributes().capacity(),
                    };
                    if matches!(
                        edit.mode,
                        HtmlConstructionAttributeEditMode::NamedMerge { .. }
                    ) {
                        EditStage::Report { target }
                    } else {
                        EditStage::Slots { target }
                    }
                }
                EditStage::Report { target } => {
                    let target = *target;
                    let generation = edit
                        .storage
                        .arena
                        .temporary_generation
                        .checked_add(1)
                        .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?;
                    let plan = HtmlConstructionAllocationSnapshot::vector(
                        &edit.storage.arena.identity,
                        AllocationRole::AttributePlan(target),
                        generation,
                        &edit.decisions,
                    )?;
                    edit.storage.arena.temporary_generation = generation;
                    EditStage::Merge { target, plan }
                }
                // A ready edit issues no setup action.
                EditStage::Slots { .. } | EditStage::Merge { .. } => {
                    return Err(HtmlConstructionErrorKind::OperationNotReady.into());
                }
            })
        })();
        match result {
            Ok(next) => {
                edit.stage = next;
                edit.observing = true;
                Ok(HtmlConstructionAttributeObservation { edit })
            }
            Err(e) => {
                edit.failure = Some(e.clone());
                Err(e)
            }
        }
    }
}
impl HtmlConstructionAttributeObservation<'_> {
    /// The merge plan's allocation snapshot, once the plan exists.
    pub fn allocation(&self) -> Option<&HtmlConstructionAllocationSnapshot> {
        self.edit.stage.plan()
    }
    /// Accepts the observation, which resumes the edit.
    pub fn accept(self) {
        self.edit.observing = false;
    }
}

/// Whose name a hashing step is for.
#[derive(Clone, Copy, Debug)]
enum Subject {
    /// The stored attribute at this position, entered in the index.
    Existing(usize),
    /// The name being planned or appended, looked up in the index.
    Incoming,
}

#[derive(Clone, Copy, Debug)]
enum CandidatePhase<'n> {
    Target(usize),
    /// Overwrite the next slots of the index with the empty value.
    IndexClear,
    /// Read the lengths of the name of `subject`.
    IndexHashLength {
        subject: Subject,
    },
    /// Hash the name of `subject`, which has `units` code units.
    IndexHash {
        subject: Subject,
        units: usize,
    },
    /// Enter the existing attribute at `position` in the first empty slot of its probe.
    IndexInsert {
        position: usize,
        probe: HtmlNameIndexProbe,
        tag: u32,
    },
    /// Read the slot the candidate's probe is at.
    IndexProbe {
        probe: HtmlNameIndexProbe,
        tag: u32,
    },
    /// Comparing the candidate with the existing attribute at `position` found by `probe`.
    IndexTargetName {
        probe: HtmlNameIndexProbe,
        tag: u32,
        position: usize,
        name: NamePhase,
    },
    /// The probe found the earlier candidate `index`; its name is needed.
    IndexPrior {
        probe: HtmlNameIndexProbe,
        tag: u32,
        index: usize,
    },
    /// Comparing the candidate with the earlier candidate's name found by `probe`.
    IndexPriorName {
        probe: HtmlNameIndexProbe,
        tag: u32,
        index: usize,
        name: NamePhase,
        prior: &'n DocumentString,
    },
    TargetName {
        index: usize,
        name: NamePhase,
    },
    Prior(usize),
    /// Comparing the candidate with the qualified prior name it carries.
    PriorName {
        index: usize,
        name: NamePhase,
        prior: &'n DocumentString,
    },
    Reserve,
    Witness,
    Observing,
    Done,
}
/// The planning of one named candidate. It compares the candidate's name with the target's
/// existing attributes and then with earlier candidates. Dropping it before `try_finish` poisons
/// the edit.
#[derive(Debug)]
pub struct HtmlConstructionAttributeCandidate<'e, 'n> {
    storage: &'e mut HtmlConstructionStorage,
    busy: &'e mut bool,
    decisions: &'e mut Vec<NameDecision>,
    selected: &'e mut usize,
    plan: &'e mut HtmlConstructionAllocationSnapshot,
    target: usize,
    index: usize,
    name: &'n DocumentString,
    phase: CandidatePhase<'n>,
    found: bool,
    complete: bool,
    failure: Option<HtmlConstructionError>,
    table: &'e mut Option<HtmlAttributeNameIndex>,
}
/// One pending candidate action. It borrows the candidate exclusively; `perform` consumes it.
#[derive(Debug)]
pub struct HtmlConstructionAttributeCandidateAction<'a, 'e, 'n> {
    candidate: &'a mut HtmlConstructionAttributeCandidate<'e, 'n>,
}
/// The observation after a candidate action. It borrows the candidate exclusively; `accept`
/// resumes it.
#[derive(Debug)]
pub struct HtmlConstructionAttributeCandidateObservation<'a, 'e, 'n> {
    candidate: &'a mut HtmlConstructionAttributeCandidate<'e, 'n>,
    /// The phase this accepted observation resumes.
    next: CandidatePhase<'n>,
    allocation: bool,
}
impl Drop for HtmlConstructionAttributeCandidate<'_, '_> {
    fn drop(&mut self) {
        if !self.complete {
            *self.busy = true;
        }
    }
}
impl<'e, 'n> HtmlConstructionAttributeCandidate<'e, 'n> {
    /// The actual retained report, including after a provider failure.
    pub fn allocation(&self) -> &HtmlConstructionAllocationSnapshot {
        self.plan
    }
    /// The next candidate action, or `Ok(None)` when planning of this candidate is finished. It
    /// returns the latched failure, or the not-ready error while an observation awaits `accept`.
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionAttributeCandidateAction<'_, 'e, 'n>>, HtmlConstructionError>
    {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        if matches!(self.phase, CandidatePhase::Observing) {
            return Err(HtmlConstructionErrorKind::OperationNotReady.into());
        }
        Ok((!matches!(self.phase, CandidatePhase::Done))
            .then_some(HtmlConstructionAttributeCandidateAction { candidate: self }))
    }
    /// Consumes a finished candidate and returns whether it is kept (no matching name). Returns
    /// the latched failure, or the not-ready error before planning finished; either way the
    /// candidate is handed back unchanged.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn try_finish(mut self) -> Result<bool, HtmlConstructionRefusal<Self>> {
        if let Some(e) = &self.failure {
            let e = e.clone();
            return Err(HtmlConstructionRefusal::new(self, e));
        }
        if !matches!(self.phase, CandidatePhase::Done) {
            return Err(HtmlConstructionRefusal::new(
                self,
                HtmlConstructionErrorKind::OperationNotReady,
            ));
        }
        self.complete = true;
        *self.busy = false;
        Ok(!self.found)
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
fn table_error() -> HtmlConstructionError {
    HtmlConstructionErrorKind::CapacityMismatch.into()
}
/// The work of reading one slot of the index, which writes the empty slot it finds.
fn probe_work() -> HtmlConstructionAttributeWork {
    HtmlConstructionAttributeWork {
        record_visits: 1,
        record_writes: 1,
        copied_bytes: size_of::<usize>(),
        ..Default::default()
    }
}
/// The work of clearing the next slots of the index.
fn clear_work(unwritten: usize) -> HtmlConstructionAttributeWork {
    let slots = unwritten.min(CLEAR_CHUNK);
    HtmlConstructionAttributeWork {
        record_writes: slots,
        copied_bytes: slots * size_of::<usize>(),
        ..Default::default()
    }
}
/// The work of reading the lengths of a name to hash: the attribute and its two name headers.
fn hash_length_work() -> HtmlConstructionAttributeWork {
    HtmlConstructionAttributeWork {
        record_visits: 3,
        ..Default::default()
    }
}
fn hash_work(units: usize) -> HtmlConstructionAttributeWork {
    HtmlConstructionAttributeWork {
        inspected_bytes: 2 * units,
        ..Default::default()
    }
}
impl<'e, 'n> HtmlConstructionAttributeCandidate<'e, 'n> {
    /// The phase after the candidate's own name is settled: the plan is reserved or the
    /// decision is recorded.
    fn settled(&self) -> CandidatePhase<'n> {
        if self.decisions.len() == self.decisions.capacity() {
            CandidatePhase::Reserve
        } else {
            CandidatePhase::Witness
        }
    }
    /// The phase after the existing attribute at `position` is entered in the index, or skipped.
    fn after_existing(
        &mut self,
        position: usize,
    ) -> Result<CandidatePhase<'n>, HtmlConstructionError> {
        let len = element_attributes(self.storage, self.target)?.len();
        if position + 1 < len {
            return Ok(CandidatePhase::IndexHashLength {
                subject: Subject::Existing(position + 1),
            });
        }
        let table = self.table.as_mut().ok_or_else(table_error)?;
        table.bind(Binding::Merge {
            target: self.target,
        });
        Ok(CandidatePhase::IndexHashLength {
            subject: Subject::Incoming,
        })
    }
    /// The phase that starts entering the existing attributes, or the candidate's own lookup
    /// when the element has none.
    fn after_clear(&mut self) -> Result<CandidatePhase<'n>, HtmlConstructionError> {
        let len = element_attributes(self.storage, self.target)?.len();
        if len > 0 {
            return Ok(CandidatePhase::IndexHashLength {
                subject: Subject::Existing(0),
            });
        }
        let table = self.table.as_mut().ok_or_else(table_error)?;
        table.bind(Binding::Merge {
            target: self.target,
        });
        Ok(CandidatePhase::IndexHashLength {
            subject: Subject::Incoming,
        })
    }
}
impl<'a, 'e, 'n> HtmlConstructionAttributeCandidateAction<'a, 'e, 'n> {
    /// Describes the pending candidate action's work for the consumer to admit.
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
                allocation: c.plan,
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
                    record_writes: usize::from(!*c.busy),
                    copied_bytes: usize::from(!*c.busy) * size_of::<bool>(),
                    ..Default::default()
                })
            }
            CandidatePhase::IndexClear => HtmlConstructionAttributeDescription::Inspect(
                clear_work(c.table.as_ref().map_or(0, |table| table.unwritten())),
            ),
            CandidatePhase::IndexHashLength { .. } => {
                HtmlConstructionAttributeDescription::Inspect(hash_length_work())
            }
            CandidatePhase::IndexHash { units, .. } => {
                HtmlConstructionAttributeDescription::Inspect(hash_work(units))
            }
            CandidatePhase::IndexInsert { .. } | CandidatePhase::IndexProbe { .. } => {
                HtmlConstructionAttributeDescription::Inspect(probe_work())
            }
            CandidatePhase::IndexTargetName { name, .. } => {
                let mut work = comparison_work(name);
                work.record_writes = 1;
                work.copied_bytes = size_of::<bool>();
                HtmlConstructionAttributeDescription::Compare {
                    prior_index: None,
                    work,
                }
            }
            CandidatePhase::IndexPrior { index, .. } => {
                HtmlConstructionAttributeDescription::Compare {
                    prior_index: Some(index),
                    work: HtmlConstructionAttributeWork {
                        record_visits: 3,
                        provider_calls: 2,
                        record_writes: 1,
                        copied_bytes: size_of::<Option<&DocumentString>>(),
                        ..Default::default()
                    },
                }
            }
            CandidatePhase::IndexPriorName { name, .. } => {
                let mut work = comparison_work(name);
                work.record_writes = 2;
                work.copied_bytes = size_of::<bool>() + size_of::<Option<&DocumentString>>();
                HtmlConstructionAttributeDescription::Compare {
                    prior_index: None,
                    work,
                }
            }
            _ => unreachable!("pending candidate action"),
        }
        .transitions::<CandidatePhase>()
    }
    /// Performs the candidate action. Pass the admitted name of the earlier candidate when
    /// `description().prior_index()` is `Some`; the name must be the same shared allocation
    /// planned earlier, or the attribute-name-mismatch error is returned. A plan reservation
    /// reserves exactly the described minimum capacity.
    pub fn perform(
        self,
        prior: Option<&'n DocumentString>,
    ) -> Result<HtmlConstructionAttributeCandidateObservation<'a, 'e, 'n>, HtmlConstructionError>
    {
        self.perform_reserving(prior, None)
    }
    /// Performs a plan reservation with the consumer's admitted `capacity`, so that planning `n`
    /// candidates one at a time can request `O(log n)` reallocations by growing the plan
    /// geometrically, up to the edit's candidate count. The capacity cannot be below the described
    /// minimum. Any other action, or a smaller capacity, latches the capacity-mismatch error
    /// before any provider call. When the allocator refuses the larger capacity the reservation
    /// requests the minimum instead, and the observation reports the capacity actually reached.
    pub fn perform_with_capacity(
        self,
        prior: Option<&'n DocumentString>,
        capacity: usize,
    ) -> Result<HtmlConstructionAttributeCandidateObservation<'a, 'e, 'n>, HtmlConstructionError>
    {
        self.perform_reserving(prior, Some(capacity))
    }
    fn perform_reserving(
        self,
        prior: Option<&'n DocumentString>,
        reserved: Option<usize>,
    ) -> Result<HtmlConstructionAttributeCandidateObservation<'a, 'e, 'n>, HtmlConstructionError>
    {
        let c = self.candidate;
        // Owner state, rather than Drop alone, prevents a forgotten cursor or
        // observation from making an unaccepted admission usable.
        if !*c.busy {
            *c.busy = true;
        }
        let mut allocation = false;
        let result = (|| -> Result<_, HtmlConstructionError> {
            if reserved.is_some_and(|capacity| {
                !matches!(c.phase, CandidatePhase::Reserve) || capacity <= c.index
            }) {
                return Err(HtmlConstructionErrorKind::CapacityMismatch.into());
            }
            Ok(match c.phase {
                CandidatePhase::Target(index) => {
                    if index == element_attributes(c.storage, c.target)?.len() {
                        CandidatePhase::Prior(0)
                    } else if element_attributes(c.storage, c.target)?[index]
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
                    let old = element_attributes(c.storage, c.target)?[index]
                        .name()
                        .local_name();
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
                        if c.decisions.len() == c.decisions.capacity() {
                            CandidatePhase::Reserve
                        } else {
                            CandidatePhase::Witness
                        }
                    } else {
                        let prior =
                            prior.ok_or(HtmlConstructionErrorKind::AttributeNameMismatch)?;
                        if c.decisions[index].name.as_ptr()
                            != Arc::as_ptr(prior.shared_code_units())
                        {
                            return Err(HtmlConstructionErrorKind::AttributeNameMismatch.into());
                        }
                        CandidatePhase::PriorName {
                            index,
                            name: NamePhase::LocalLength,
                            prior,
                        }
                    }
                }
                CandidatePhase::PriorName { index, name, prior } => {
                    match advance_comparison(name, None, prior, None, c.name) {
                        ComparisonStep::Continue(name) => {
                            CandidatePhase::PriorName { index, name, prior }
                        }
                        ComparisonStep::Different => CandidatePhase::Prior(index + 1),
                        ComparisonStep::Equal => {
                            c.found = true;
                            if c.decisions.len() == c.decisions.capacity() {
                                CandidatePhase::Reserve
                            } else {
                                CandidatePhase::Witness
                            }
                        }
                    }
                }
                CandidatePhase::IndexClear => {
                    let table = c.table.as_mut().ok_or_else(table_error)?;
                    table.clear_chunk(CLEAR_CHUNK);
                    if table.unwritten() == 0 {
                        c.after_clear()?
                    } else {
                        CandidatePhase::IndexClear
                    }
                }
                CandidatePhase::IndexHashLength { subject } => match subject {
                    Subject::Existing(position) => {
                        let name = element_attributes(c.storage, c.target)?
                            .get(position)
                            .ok_or_else(table_error)?
                            .name();
                        if name.namespace().uri().is_some() {
                            // A candidate name has no namespace, so it never equals this one.
                            c.after_existing(position)?
                        } else {
                            CandidatePhase::IndexHash {
                                subject,
                                units: name.local_name().len(),
                            }
                        }
                    }
                    Subject::Incoming => CandidatePhase::IndexHash {
                        subject,
                        units: c.name.len(),
                    },
                },
                CandidatePhase::IndexHash { subject, .. } => {
                    let table = c.table.as_ref().ok_or_else(table_error)?;
                    let hash = match subject {
                        Subject::Existing(position) => table.hash_local(
                            element_attributes(c.storage, c.target)?
                                .get(position)
                                .ok_or_else(table_error)?
                                .name()
                                .local_name()
                                .code_units(),
                        ),
                        Subject::Incoming => table.hash_local(c.name.code_units()),
                    };
                    let probe = table.probe(hash).ok_or_else(table_error)?;
                    let tag = tag(hash);
                    match subject {
                        Subject::Existing(position) => CandidatePhase::IndexInsert {
                            position,
                            probe,
                            tag,
                        },
                        Subject::Incoming => CandidatePhase::IndexProbe { probe, tag },
                    }
                }
                CandidatePhase::IndexInsert {
                    position,
                    mut probe,
                    tag,
                } => {
                    let table = c.table.as_mut().ok_or_else(table_error)?;
                    match table.step(&probe).map_err(|_| table_error())? {
                        HtmlNameIndexStep::Vacant(vacancy) => {
                            let placed = entry(tag, Occupant::Attribute(position))
                                .is_some_and(|entry| table.insert(vacancy, entry).is_ok());
                            if !placed {
                                return Err(table_error());
                            }
                            c.after_existing(position)?
                        }
                        HtmlNameIndexStep::Occupied(_) => {
                            probe.next_slot().ok_or_else(table_error)?;
                            CandidatePhase::IndexInsert {
                                position,
                                probe,
                                tag,
                            }
                        }
                    }
                }
                CandidatePhase::IndexProbe { mut probe, tag } => {
                    let table = c.table.as_mut().ok_or_else(table_error)?;
                    match table.step(&probe).map_err(|_| table_error())? {
                        HtmlNameIndexStep::Vacant(vacancy) => {
                            // Nothing has the name: the candidate is kept and entered.
                            let placed = entry(tag, Occupant::Candidate(c.index))
                                .is_some_and(|entry| table.insert(vacancy, entry).is_ok());
                            if !placed {
                                return Err(table_error());
                            }
                            c.settled()
                        }
                        HtmlNameIndexStep::Occupied(held) => match occupant(held) {
                            (_, held_tag) if held_tag != tag => {
                                probe.next_slot().ok_or_else(table_error)?;
                                CandidatePhase::IndexProbe { probe, tag }
                            }
                            (Occupant::Candidate(index), _) if index < c.index => {
                                CandidatePhase::IndexPrior { probe, tag, index }
                            }
                            (Occupant::Attribute(position), _) => CandidatePhase::IndexTargetName {
                                probe,
                                tag,
                                position,
                                name: NamePhase::LocalLength,
                            },
                            _ => return Err(table_error()),
                        },
                    }
                }
                CandidatePhase::IndexTargetName {
                    mut probe,
                    tag,
                    position,
                    name,
                } => {
                    let old = element_attributes(c.storage, c.target)?
                        .get(position)
                        .ok_or_else(table_error)?
                        .name()
                        .local_name();
                    match advance_comparison(name, None, old, None, c.name) {
                        ComparisonStep::Continue(name) => CandidatePhase::IndexTargetName {
                            probe,
                            tag,
                            position,
                            name,
                        },
                        ComparisonStep::Different => {
                            probe.next_slot().ok_or_else(table_error)?;
                            CandidatePhase::IndexProbe { probe, tag }
                        }
                        ComparisonStep::Equal => {
                            c.found = true;
                            c.settled()
                        }
                    }
                }
                CandidatePhase::IndexPrior { probe, tag, index } => {
                    let prior = prior.ok_or(HtmlConstructionErrorKind::AttributeNameMismatch)?;
                    if c.decisions[index].name.as_ptr() != Arc::as_ptr(prior.shared_code_units()) {
                        return Err(HtmlConstructionErrorKind::AttributeNameMismatch.into());
                    }
                    CandidatePhase::IndexPriorName {
                        probe,
                        tag,
                        index,
                        name: NamePhase::LocalLength,
                        prior,
                    }
                }
                CandidatePhase::IndexPriorName {
                    mut probe,
                    tag,
                    index,
                    name,
                    prior,
                } => match advance_comparison(name, None, prior, None, c.name) {
                    ComparisonStep::Continue(name) => CandidatePhase::IndexPriorName {
                        probe,
                        tag,
                        index,
                        name,
                        prior,
                    },
                    ComparisonStep::Different => {
                        probe.next_slot().ok_or_else(table_error)?;
                        CandidatePhase::IndexProbe { probe, tag }
                    }
                    ComparisonStep::Equal => {
                        c.found = true;
                        c.settled()
                    }
                },
                CandidatePhase::Reserve => {
                    let new_generation = c
                        .storage
                        .arena
                        .temporary_generation
                        .checked_add(1)
                        .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?;
                    let minimum = c.index + 1;
                    let target = reserved.unwrap_or(minimum);
                    target
                        .checked_mul(size_of::<NameDecision>())
                        .filter(|v| *v <= isize::MAX as usize)
                        .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?;
                    let result = crate::internal::amortized::reserve_to(c.decisions, target);
                    let snapshot: &mut HtmlConstructionAllocationSnapshot = c.plan;
                    snapshot.capacity = c.decisions.capacity();
                    snapshot.payload_bytes = snapshot.capacity * size_of::<NameDecision>();
                    if result.is_ok() {
                        snapshot.id.generation = new_generation;
                        c.storage.arena.temporary_generation = new_generation;
                    }
                    allocation = true;
                    result.map_err(|_| HtmlConstructionErrorKind::AllocationFailed)?;
                    CandidatePhase::Witness
                }
                CandidatePhase::Witness => {
                    c.decisions.push(NameDecision {
                        name: Arc::downgrade(c.name.shared_code_units()),
                        keep: !c.found,
                    });
                    c.plan.initialized_len = c.decisions.len();
                    *c.selected += usize::from(!c.found);
                    CandidatePhase::Done
                }
                _ => unreachable!("pending candidate action"),
            })
        })();
        match result {
            Ok(next) => {
                c.phase = CandidatePhase::Observing;
                Ok(HtmlConstructionAttributeCandidateObservation {
                    candidate: c,
                    next,
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
    /// The plan's allocation snapshot when the performed action reserved plan storage; `None`
    /// otherwise.
    pub fn allocation(&self) -> Option<&HtmlConstructionAllocationSnapshot> {
        self.allocation.then_some(&*self.candidate.plan)
    }
    /// Accepts the observation, which resumes planning.
    pub fn accept(self) {
        self.candidate.phase = self.next;
    }
}

#[derive(Debug)]
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
    Compare {
        index: usize,
        name: NamePhase,
    },
    /// Overwrite the next slots of the index with the empty value.
    IndexClear,
    /// Read the lengths of the name of `subject`.
    IndexHashLength {
        subject: Subject,
    },
    /// Hash the name of `subject`, which has `units` code units.
    IndexHash {
        subject: Subject,
        units: usize,
    },
    /// Enter the stored attribute at `position` in the first empty slot of its probe.
    IndexInsert {
        position: usize,
        probe: HtmlNameIndexProbe,
        tag: u32,
    },
    /// Read the slot the appended name's probe is at.
    IndexProbe {
        probe: HtmlNameIndexProbe,
        tag: u32,
    },
    /// Comparing the appended name with the stored attribute at `position` found by `probe`.
    IndexCompare {
        probe: HtmlNameIndexProbe,
        tag: u32,
        position: usize,
        name: NamePhase,
    },
    Reserve,
    Layout,
    Unique,
    /// Awaiting acceptance; records whether acceptance resumes at `Ready`.
    Observing {
        resumes_ready: bool,
    },
    Ready,
}
/// One positional step on an edit: reserve, append, replace, swap or pop. Dropping it before it
/// commits poisons the edit.
#[derive(Debug)]
pub struct HtmlConstructionAttributeSlot<'a> {
    edit: &'a mut HtmlConstructionAttributeEdit,
    input: SlotInput<'a>,
    phase: SlotPhase,
    failure: Option<HtmlConstructionError>,
    finished: bool,
    usage: HtmlConstructionUsage,
    minimum_capacity: usize,
    plan_index: usize,
    /// The empty slot of the index that the appended name's probe reached, with the name's
    /// tag, which the commit fills.
    insert_slot: Option<(HtmlNameIndexVacancy, u32)>,
}
/// One pending slot action. It borrows the slot exclusively; `perform` consumes it.
#[derive(Debug)]
pub struct HtmlConstructionAttributeSlotAction<'a, 'e> {
    slot: &'a mut HtmlConstructionAttributeSlot<'e>,
}
/// The observation after a slot action. It borrows the slot exclusively; `accept` resumes it.
#[derive(Debug)]
pub struct HtmlConstructionAttributeSlotObservation<'a, 'e> {
    slot: &'a mut HtmlConstructionAttributeSlot<'e>,
    /// The phase this accepted observation resumes.
    next: SlotPhase,
    capacity: bool,
}
/// A slot whose every action was accepted. `commit` applies it.
#[derive(Debug)]
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
impl HtmlConstructionAttributeSlot<'_> {
    /// How the appended name is checked against the stored ones: through the index when it can
    /// hold them, rebuilding it first unless it already describes exactly these attributes, and
    /// against every stored name otherwise.
    fn membership_start(&mut self) -> Result<SlotPhase, HtmlConstructionError> {
        let brute = SlotPhase::Compare {
            index: 0,
            name: NamePhase::Pair,
        };
        let target = self
            .edit
            .stage
            .target()
            .ok_or(HtmlConstructionErrorKind::OperationNotReady)?;
        let len = self.edit.capacity.initialized_len;
        let backing = Arc::as_ptr(self.edit.shared_attributes()?) as usize;
        let Some(index) = self.edit.index.as_mut() else {
            return Ok(brute);
        };
        let Some(entries) = len.checked_add(1) else {
            return Ok(brute);
        };
        let described = Binding::Slots {
            target,
            backing,
            len,
        };
        let held = HtmlAttributeNameIndex::slots_required(entries);
        if index.binding() == described && held.is_some_and(|slots| slots <= index.size()) {
            return Ok(SlotPhase::IndexHashLength {
                subject: Subject::Incoming,
            });
        }
        match index.geometry_for(entries) {
            Some(geometry) => {
                index.begin_clear(geometry);
                Ok(SlotPhase::IndexClear)
            }
            None => Ok(brute),
        }
    }
    /// The phase after the stored attribute at `position` is entered in the index.
    fn after_existing(&mut self, position: usize) -> Result<SlotPhase, HtmlConstructionError> {
        if position + 1 < self.edit.capacity.initialized_len {
            return Ok(SlotPhase::IndexHashLength {
                subject: Subject::Existing(position + 1),
            });
        }
        self.bind_described()?;
        Ok(SlotPhase::IndexHashLength {
            subject: Subject::Incoming,
        })
    }
    /// The index no longer describes the element: the next step that needs it rebuilds it.
    fn invalidate_index(&mut self) {
        if let Some(index) = self.edit.index.as_mut() {
            index.bind(Binding::Unbound);
        }
    }
    /// Enters the attribute just appended at position `before` in the slot its probe reached,
    /// and renews what the index describes. An append that was not looked up through the index
    /// leaves the index describing the element as it was, so it is no longer valid.
    fn enter_appended(&mut self, before: usize) {
        let Some((vacancy, tag)) = self.insert_slot.take() else {
            self.invalidate_index();
            return;
        };
        let described = self.bind_described_after();
        if let Some(index) = self.edit.index.as_mut() {
            let placed = entry(tag, Occupant::Attribute(before))
                .is_some_and(|entry| index.insert(vacancy, entry).is_ok());
            match described {
                Some(binding) if placed => index.bind(binding),
                _ => index.bind(Binding::Unbound),
            }
        }
    }
    fn bind_described_after(&self) -> Option<Binding> {
        Some(Binding::Slots {
            target: self.edit.stage.target()?,
            backing: Arc::as_ptr(self.edit.shared_attributes().ok()?) as usize,
            len: self.edit.capacity.initialized_len,
        })
    }
    /// Records that the index describes the attributes the element has now.
    fn bind_described(&mut self) -> Result<(), HtmlConstructionError> {
        let target = self
            .edit
            .stage
            .target()
            .ok_or(HtmlConstructionErrorKind::OperationNotReady)?;
        let len = self.edit.capacity.initialized_len;
        let backing = Arc::as_ptr(self.edit.shared_attributes()?) as usize;
        self.edit
            .index
            .as_mut()
            .ok_or_else(table_error)?
            .bind(Binding::Slots {
                target,
                backing,
                len,
            });
        Ok(())
    }
}
impl<'e> HtmlConstructionAttributeSlot<'e> {
    /// The element's shared attribute backing. Returns the not-ready error while the slot is ready
    /// to commit.
    pub fn shared_attributes(&self) -> Result<&Arc<Vec<Attribute>>, HtmlConstructionError> {
        if self.failure.is_none()
            && matches!(
                self.phase,
                SlotPhase::Ready
                    | SlotPhase::Observing {
                        resumes_ready: true
                    }
            )
        {
            return Err(HtmlConstructionErrorKind::OperationNotReady.into());
        }
        self.edit.shared_attributes()
    }
    /// Actual provider result remains available even when reservation failed.
    pub fn capacity(&self) -> HtmlConstructionAttributeCapacity {
        self.edit.capacity
    }
    /// The next slot action, or `Ok(None)` once it is ready. It returns the latched failure, or
    /// the not-ready error while an observation awaits `accept`.
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionAttributeSlotAction<'_, 'e>>, HtmlConstructionError> {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        if matches!(self.phase, SlotPhase::Observing { .. }) {
            return Err(HtmlConstructionErrorKind::OperationNotReady.into());
        }
        Ok((!matches!(self.phase, SlotPhase::Ready))
            .then_some(HtmlConstructionAttributeSlotAction { slot: self }))
    }
    /// Consumes a ready slot. Returns the latched failure, or the not-ready error before the last
    /// action was accepted; either way the slot is handed back unchanged.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn try_ready(
        self,
    ) -> Result<HtmlConstructionReadyAttributeSlot<'e>, HtmlConstructionRefusal<Self>> {
        if let Some(e) = &self.failure {
            let e = e.clone();
            return Err(HtmlConstructionRefusal::new(self, e));
        }
        if !matches!(self.phase, SlotPhase::Ready) {
            return Err(HtmlConstructionRefusal::new(
                self,
                HtmlConstructionErrorKind::OperationNotReady,
            ));
        }
        Ok(HtmlConstructionReadyAttributeSlot { slot: self })
    }
}
fn plus(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_add(b)
        .ok_or(HtmlConstructionErrorKind::RepresentationExhausted.error())
}
fn minus(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_sub(b)
        .ok_or(HtmlConstructionErrorKind::ReleaseUnderflow.error())
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
    /// The element's shared attribute backing, as
    /// `HtmlConstructionAttributeSlot::shared_attributes`.
    pub fn shared_attributes(&self) -> Result<&Arc<Vec<Attribute>>, HtmlConstructionError> {
        self.slot.shared_attributes()
    }
    /// Describes the pending slot action's work for the consumer to admit.
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
                // An append through the index fills the slot its probe reached and renews the
                // binding that says which attributes the index describes.
                let (counter_writes, counter_bytes) = if s.insert_slot.is_some() {
                    (
                        counter_writes + 2,
                        counter_bytes + size_of::<usize>() + size_of::<Binding>(),
                    )
                } else {
                    (counter_writes, counter_bytes)
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
            SlotPhase::IndexClear => HtmlConstructionAttributeDescription::Inspect(clear_work(
                s.edit.index.as_ref().map_or(0, |table| table.unwritten()),
            )),
            SlotPhase::IndexHashLength { .. } => {
                HtmlConstructionAttributeDescription::Inspect(hash_length_work())
            }
            SlotPhase::IndexHash { units, .. } => {
                HtmlConstructionAttributeDescription::Inspect(hash_work(units))
            }
            SlotPhase::IndexInsert { .. } | SlotPhase::IndexProbe { .. } => {
                HtmlConstructionAttributeDescription::Inspect(probe_work())
            }
            SlotPhase::IndexCompare { name, .. } => HtmlConstructionAttributeDescription::Compare {
                prior_index: None,
                work: comparison_work(name),
            },
            SlotPhase::Observing { .. } | SlotPhase::Ready => unreachable!("pending slot action"),
        }
        .transitions::<SlotPhase>()
    }
    /// Performs the slot action. The observation must be accepted before the next action. The
    /// first failure is latched.
    pub fn perform(
        self,
    ) -> Result<HtmlConstructionAttributeSlotObservation<'a, 'e>, HtmlConstructionError> {
        let s = self.slot;
        if !s.edit.busy {
            s.edit.busy = true;
        }
        let mut capacity = false;
        let result = (|| -> Result<_, HtmlConstructionError> {
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
                        let attribute = v
                            .last()
                            .ok_or(HtmlConstructionErrorKind::CapacityMismatch)?;
                        if s.edit.capacity.initialized_len == s.edit.capacity.capacity {
                            return Err(HtmlConstructionErrorKind::CapacityMismatch.into());
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
                            s.membership_start()?
                        }
                    }
                    SlotInput::Replace { index, incoming } => {
                        let old = s
                            .edit
                            .attributes()?
                            .get(*index)
                            .ok_or(HtmlConstructionErrorKind::CapacityMismatch)?;
                        let new = incoming
                            .as_ref()
                            .ok_or(HtmlConstructionErrorKind::CapacityMismatch)?;
                        s.usage.utf16_units = plus(
                            minus(s.usage.utf16_units, attribute_units(old)?)?,
                            attribute_units(new)?,
                        )?;
                        SlotPhase::SameName
                    }
                    SlotInput::Swap(index) => {
                        if plus(*index, 1)? >= s.edit.capacity.initialized_len {
                            return Err(HtmlConstructionErrorKind::CapacityMismatch.into());
                        }
                        SlotPhase::Layout
                    }
                    SlotInput::Pop => {
                        let old = s
                            .edit
                            .attributes()?
                            .last()
                            .ok_or(HtmlConstructionErrorKind::CapacityMismatch)?;
                        s.usage.attributes = minus(s.usage.attributes, 1)?;
                        s.usage.utf16_units = minus(s.usage.utf16_units, attribute_units(old)?)?;
                        SlotPhase::Layout
                    }
                },
                SlotPhase::NextName => {
                    if s.edit.appended == s.edit.selected {
                        return Err(HtmlConstructionErrorKind::AttributeOrderMismatch.into());
                    }
                    let decision = s
                        .edit
                        .decisions
                        .get(s.plan_index)
                        .ok_or(HtmlConstructionErrorKind::AttributeOrderMismatch)?;
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
                        return Err(HtmlConstructionErrorKind::AttributeNameMismatch.into());
                    }
                    SlotPhase::Layout
                }
                SlotPhase::SameName => {
                    let SlotInput::Replace { index, .. } = s.input else {
                        unreachable!("replacement name check")
                    };
                    if identical_name(
                        s.edit.attributes()?[index].name(),
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
                        let old = s.edit.attributes()?[index].name();
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
                                return Err(HtmlConstructionErrorKind::DuplicateAttribute {
                                    first: index,
                                    duplicate: match s.input {
                                        SlotInput::Replace { index, .. } => index,
                                        _ => s.edit.capacity.initialized_len,
                                    },
                                }
                                .into());
                            }
                        }
                    }
                }
                SlotPhase::IndexClear => {
                    let table = s.edit.index.as_mut().ok_or_else(table_error)?;
                    table.clear_chunk(CLEAR_CHUNK);
                    if table.unwritten() != 0 {
                        SlotPhase::IndexClear
                    } else if s.edit.capacity.initialized_len == 0 {
                        s.bind_described()?;
                        SlotPhase::IndexHashLength {
                            subject: Subject::Incoming,
                        }
                    } else {
                        SlotPhase::IndexHashLength {
                            subject: Subject::Existing(0),
                        }
                    }
                }
                SlotPhase::IndexHashLength { subject } => {
                    let name = match subject {
                        Subject::Existing(position) => s
                            .edit
                            .attributes()?
                            .get(position)
                            .ok_or_else(table_error)?
                            .name(),
                        Subject::Incoming => s.input.attribute().ok_or_else(table_error)?.name(),
                    };
                    let units = name
                        .namespace()
                        .uri()
                        .map_or(0, DocumentString::len)
                        .saturating_add(name.local_name().len());
                    SlotPhase::IndexHash { subject, units }
                }
                SlotPhase::IndexHash { subject, .. } => {
                    let table = s.edit.index.as_ref().ok_or_else(table_error)?;
                    let name = match subject {
                        Subject::Existing(position) => s
                            .edit
                            .attributes()?
                            .get(position)
                            .ok_or_else(table_error)?
                            .name(),
                        Subject::Incoming => s.input.attribute().ok_or_else(table_error)?.name(),
                    };
                    let hash = table.hash(name);
                    let probe = table.probe(hash).ok_or_else(table_error)?;
                    let tag = tag(hash);
                    match subject {
                        Subject::Existing(position) => SlotPhase::IndexInsert {
                            position,
                            probe,
                            tag,
                        },
                        Subject::Incoming => SlotPhase::IndexProbe { probe, tag },
                    }
                }
                SlotPhase::IndexInsert {
                    position,
                    mut probe,
                    tag,
                } => {
                    let table = s.edit.index.as_mut().ok_or_else(table_error)?;
                    match table.step(&probe).map_err(|_| table_error())? {
                        HtmlNameIndexStep::Vacant(vacancy) => {
                            let placed = entry(tag, Occupant::Attribute(position))
                                .is_some_and(|entry| table.insert(vacancy, entry).is_ok());
                            if !placed {
                                return Err(table_error());
                            }
                            s.after_existing(position)?
                        }
                        HtmlNameIndexStep::Occupied(_) => {
                            probe.next_slot().ok_or_else(table_error)?;
                            SlotPhase::IndexInsert {
                                position,
                                probe,
                                tag,
                            }
                        }
                    }
                }
                SlotPhase::IndexProbe { mut probe, tag } => {
                    let table = s.edit.index.as_ref().ok_or_else(table_error)?;
                    match table.step(&probe).map_err(|_| table_error())? {
                        HtmlNameIndexStep::Vacant(vacancy) => {
                            s.insert_slot = Some((vacancy, tag));
                            SlotPhase::Layout
                        }
                        HtmlNameIndexStep::Occupied(held) => match occupant(held) {
                            (_, held_tag) if held_tag != tag => {
                                probe.next_slot().ok_or_else(table_error)?;
                                SlotPhase::IndexProbe { probe, tag }
                            }
                            (Occupant::Attribute(position), _)
                                if position < s.edit.capacity.initialized_len =>
                            {
                                SlotPhase::IndexCompare {
                                    probe,
                                    tag,
                                    position,
                                    name: NamePhase::Pair,
                                }
                            }
                            _ => return Err(table_error()),
                        },
                    }
                }
                SlotPhase::IndexCompare {
                    mut probe,
                    tag,
                    position,
                    name,
                } => {
                    let old = s
                        .edit
                        .attributes()?
                        .get(position)
                        .ok_or_else(table_error)?
                        .name();
                    let new = s.input.attribute().ok_or_else(table_error)?.name();
                    match advance_comparison(
                        name,
                        old.namespace().uri(),
                        old.local_name(),
                        new.namespace().uri(),
                        new.local_name(),
                    ) {
                        ComparisonStep::Continue(name) => SlotPhase::IndexCompare {
                            probe,
                            tag,
                            position,
                            name,
                        },
                        ComparisonStep::Different => {
                            probe.next_slot().ok_or_else(table_error)?;
                            SlotPhase::IndexProbe { probe, tag }
                        }
                        ComparisonStep::Equal => {
                            return Err(HtmlConstructionErrorKind::DuplicateAttribute {
                                first: position,
                                duplicate: s.edit.capacity.initialized_len,
                            }
                            .into());
                        }
                    }
                }
                SlotPhase::Reserve => {
                    let required_bytes = s
                        .minimum_capacity
                        .checked_mul(size_of::<Attribute>())
                        .filter(|v| *v <= isize::MAX as usize)
                        .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?;
                    let _ = required_bytes;
                    let additional = s.minimum_capacity - s.edit.capacity.initialized_len;
                    let backing = Arc::get_mut(s.edit.backing_mut()?)
                        .ok_or(HtmlConstructionErrorKind::SharedAllocationAliased)?;
                    let result = backing.try_reserve_exact(additional);
                    s.edit.capacity.capacity = backing.capacity();
                    capacity = true;
                    result.map_err(|_| HtmlConstructionErrorKind::AllocationFailed)?;
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
                        return Err(HtmlConstructionErrorKind::SharedAllocationAliased.into());
                    }
                    SlotPhase::Ready
                }
                _ => unreachable!("pending slot action"),
            })
        })();
        match result {
            Ok(next) => {
                s.phase = SlotPhase::Observing {
                    resumes_ready: matches!(next, SlotPhase::Ready),
                };
                Ok(HtmlConstructionAttributeSlotObservation {
                    slot: s,
                    next,
                    capacity,
                })
            }
            Err(e) => {
                s.failure = Some(e.clone());
                Err(e)
            }
        }
    }
}
impl HtmlConstructionAttributeSlotObservation<'_, '_> {
    /// The element's shared attribute backing, as
    /// `HtmlConstructionAttributeSlot::shared_attributes`.
    pub fn shared_attributes(&self) -> Result<&Arc<Vec<Attribute>>, HtmlConstructionError> {
        self.slot.shared_attributes()
    }
    /// The backing's dimensions when the performed action reserved capacity; `None` otherwise. A
    /// failed reservation returns an error and produces no observation, so only the slot's own
    /// `capacity` survives such a failure.
    pub fn capacity(&self) -> Option<HtmlConstructionAttributeCapacity> {
        self.capacity.then_some(self.slot.edit.capacity)
    }
    /// Accepts the observation, which resumes the slot.
    pub fn accept(self) {
        self.slot.phase = self.next;
    }
}
impl HtmlConstructionReadyAttributeSlot<'_> {
    /// Applies the prepaid step. Returns the attribute that `replace` or `pop` removed, and `None`
    /// for every other step.
    pub fn commit(mut self) -> Option<Attribute> {
        let s = &mut self.slot;
        let removed = match &mut s.input {
            SlotInput::Reserve(_) => None,
            SlotInput::Append(incoming) => {
                let before = s.edit.capacity.initialized_len;
                s.edit
                    .ready_backing()
                    .push(incoming.pop().expect("ready actual last slot"));
                s.edit.capacity.initialized_len += 1;
                if matches!(
                    s.edit.mode,
                    HtmlConstructionAttributeEditMode::NamedMerge { .. }
                ) {
                    s.edit.appended += 1;
                    s.edit.next_selected = s.plan_index + 1;
                    s.invalidate_index();
                } else {
                    s.edit.used = true;
                    s.enter_appended(before);
                }
                None
            }
            SlotInput::Replace { index, incoming } => {
                s.edit.used = true;
                if let Some(table) = s.edit.index.as_mut() {
                    table.bind(Binding::Unbound);
                }
                Some(std::mem::replace(
                    &mut s.edit.ready_backing()[*index],
                    incoming.take().expect("ready actual replacement"),
                ))
            }
            SlotInput::Swap(index) => {
                s.edit.ready_backing().swap(*index, *index + 1);
                s.edit.used = true;
                if let Some(table) = s.edit.index.as_mut() {
                    table.bind(Binding::Unbound);
                }
                None
            }
            SlotInput::Pop => {
                s.invalidate_index();
                let value = s.edit.ready_backing().pop().expect("ready last slot");
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
#[derive(Debug)]
pub struct HtmlConstructionAttributePlanRetirement {
    decisions: Vec<NameDecision>,
    snapshot: Option<HtmlConstructionAllocationSnapshot>,
}
/// What a cleanup action of the merge plan releases.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlConstructionAttributeRetirementDescription {
    /// One initialized decision cell with its weak name witness.
    NameWitness {
        /// The auxiliary entries the action destroys: 1.
        auxiliary_entries: usize,
        /// The provider requests the action makes: 1.
        provider_calls: usize,
    },
    /// The plan's allocation itself, once every decision cell is released.
    Allocation,
}
/// One pending cleanup action of the merge plan. It borrows the retirement exclusively; `perform`
/// consumes it.
#[derive(Debug)]
pub struct HtmlConstructionAttributeRetirementAction<'a> {
    retirement: &'a mut HtmlConstructionAttributePlanRetirement,
}
impl HtmlConstructionAttributePlanRetirement {
    /// The next cleanup action, or `None` once the plan is fully released.
    pub fn next_action(&mut self) -> Option<HtmlConstructionAttributeRetirementAction<'_>> {
        (!self.decisions.is_empty() || self.snapshot.is_some())
            .then_some(HtmlConstructionAttributeRetirementAction { retirement: self })
    }
}
impl HtmlConstructionAttributeRetirementAction<'_> {
    /// Describes what the pending action releases.
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
    /// The plan's allocation snapshot, if one was made.
    pub fn allocation(&self) -> Option<&HtmlConstructionAllocationSnapshot> {
        self.retirement.snapshot.as_ref()
    }
    /// Performs the cleanup. Returns the retirement evidence when the plan's allocation was
    /// released and `None` for a decision cell.
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
    use crate::internal::construction::bounded::operations::HtmlConstructionCommit;
    use crate::internal::values::name_index::HtmlNameIndexSlot;

    fn attribute(name: impl Into<DocumentString>, value: &str) -> Attribute {
        Attribute::new(QualifiedName::new(Namespace::none(), None, name), value)
    }
    fn make_storage(attributes: Vec<Attribute>) -> (HtmlConstructionStorage, HtmlBuilderNode) {
        let mut initialization = Some(HtmlConstructionStorage::begin());
        while let Some(action) = initialization.as_mut().unwrap().next_action().unwrap() {
            action.perform().accept();
        }
        let storage = slots::initialization(&mut initialization).unwrap().commit();
        let mut operation = Some(storage.prepare(HtmlConstructionCommand::Create {
            kind: HtmlNodeKind::Element(HtmlElement::new(
                QualifiedName::new(Namespace::none(), None, "root"),
                OrderedAttributes::new(attributes).unwrap(),
            )),
        }));
        while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        let HtmlConstructionCommit::Changed { storage, created } =
            slots::operation(&mut operation).unwrap().commit()
        else {
            panic!("create")
        };
        (storage, created.unwrap())
    }
    #[test]
    fn setup_actions_charge_the_stage_store_and_the_two_observation_flag_writes() {
        // Derived from the writes themselves: a setup action stores the stage it reaches and sets
        // the observing flag, accept clears the flag, and a failed action stores its typed failure
        // instead, so admission covers the larger of the two paths.
        let transition_bytes = (size_of::<EditStage>() + 2 * size_of::<bool>())
            .max(size_of::<Option<HtmlConstructionError>>());
        let transition_atomic =
            size_of::<EditStage>().max(size_of::<Option<HtmlConstructionError>>());
        let (storage, target) = make_storage(Vec::new());
        let mut edit = HtmlConstructionAttributeEdit::begin(
            storage,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 1 },
        );
        let qualify = edit.next_action().unwrap().unwrap();
        assert_eq!(
            qualify.description().work(),
            HtmlConstructionAttributeWork {
                record_visits: 5,
                provider_calls: 2,
                // The capacity is the action's own write.
                record_writes: 1 + 3,
                copied_bytes: size_of::<HtmlConstructionAttributeCapacity>() + transition_bytes,
                atomic_layout_bytes: size_of::<HtmlConstructionAttributeCapacity>()
                    .max(transition_atomic),
                ..Default::default()
            }
        );
        qualify.perform().unwrap().accept();
        let report = edit.next_action().unwrap().unwrap();
        assert_eq!(
            report.description().work(),
            HtmlConstructionAttributeWork {
                record_visits: 3,
                provider_calls: 1,
                // The generation counter is the action's own write; the report is stored with
                // the stage.
                record_writes: 1 + 3,
                copied_bytes: size_of::<u64>() + transition_bytes,
                atomic_layout_bytes: size_of::<HtmlConstructionAllocationSnapshot>()
                    .max(transition_atomic),
                ..Default::default()
            }
        );
    }
    fn make_edit(
        storage: HtmlConstructionStorage,
        target: HtmlBuilderNode,
        mode: HtmlConstructionAttributeEditMode,
    ) -> HtmlConstructionAttributeEdit {
        let mut edit = HtmlConstructionAttributeEdit::begin(storage, target, mode);
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
        slots::attribute_slot(&mut slot).unwrap().commit()
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
            slots::candidate(&mut candidate).unwrap();
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
        let (storage, retirement) = slots::edit(&mut owner).unwrap().into_parts();
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
        assert!(matches!(slots::attribute_slot(&mut owner), Err(error) if error == expected));
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
            HtmlConstructionErrorKind::DuplicateAttribute {
                first: 1,
                duplicate: 0,
            }
            .error(),
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
                HtmlConstructionErrorKind::CapacityMismatch.error(),
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
            HtmlConstructionErrorKind::CapacityMismatch.error(),
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
                HtmlConstructionErrorKind::CapacityMismatch.error(),
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
        assert_slot_refused(
            edit.pop().unwrap(),
            HtmlConstructionErrorKind::CapacityMismatch.error(),
        );
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
        let attrs = edit.attributes().unwrap();
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
            slots::edit(&mut edit),
            Err(ref error) if error.kind == HtmlConstructionErrorKind::OperationNotReady
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
            Err(ref error) if error.kind == HtmlConstructionErrorKind::OperationNotReady
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
            slots::candidate(&mut candidate),
            Err(ref error) if error.kind == HtmlConstructionErrorKind::OperationNotReady
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
        assert_eq!(
            failure,
            HtmlConstructionErrorKind::AttributeNameMismatch.error()
        );
        drop(pending);
        assert_eq!(incoming.len(), 1);
        assert!(edit.attributes().unwrap().is_empty());
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
            HtmlConstructionErrorKind::DuplicateAttribute {
                first: 0,
                duplicate: 1
            }
            .error()
        );
        drop(pending);
        assert_eq!(incoming[0].value().to_utf8().unwrap(), "new");
        assert_eq!(
            edit.attributes().unwrap()[0].value().to_utf8().unwrap(),
            "old"
        );
        let (storage, retirement) = edit.abort().into_parts();
        retire(retirement);
        let target = storage.node_at(0).unwrap().handle();
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let mut incoming = vec![Attribute::new(
            QualifiedName::new(Namespace::svg(), None, "x"),
            "namespaced",
        )];
        slot(edit.append(&mut incoming).unwrap());
        assert_eq!(edit.attributes().unwrap().len(), 2);
        assert!(
            edit.attributes().unwrap()[1]
                .name()
                .namespace()
                .uri()
                .is_some()
        );
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
            if matches!(observation.next, SlotPhase::Ready) {
                assert!(matches!(
                    observation.shared_attributes(),
                    Err(ref error) if error.kind == HtmlConstructionErrorKind::OperationNotReady
                ));
            }
            observation.accept();
        }
        assert!(matches!(
            pending.as_ref().unwrap().shared_attributes(),
            Err(ref error) if error.kind == HtmlConstructionErrorKind::OperationNotReady
        ));
        let ready = slots::attribute_slot(&mut pending).unwrap();
        drop(ready);
        drop(pending);
        assert!(incoming.is_some());
        assert_eq!(
            edit.attributes().unwrap()[0]
                .name()
                .local_name()
                .to_utf8()
                .unwrap(),
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
            slots::attribute_slot(&mut pending),
            Err(ref error) if error.kind == HtmlConstructionErrorKind::OperationNotReady
        ));
        drop(pending);
        assert_eq!(incoming.len(), 1);
        assert!(edit.attributes().unwrap().is_empty());
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
            edit.attributes().unwrap()[0]
                .name()
                .local_name()
                .to_utf8()
                .unwrap(),
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
        );
        assert!(matches!(
            edit.next_action().unwrap().unwrap().perform(),
            Err(ref error) if error.kind == HtmlConstructionErrorKind::ForeignBuilder
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
            assert_eq!(
                error,
                HtmlConstructionErrorKind::AttributeNameMismatch.error()
            );
            drop(pending);
            assert_eq!(incoming.len(), 1);
            assert!(edit.attributes().unwrap().is_empty());
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
            Err(ref error) if error.kind == HtmlConstructionErrorKind::AttributeOrderMismatch
        ));
        plan(&mut edit, &names);
        slot(edit.reserve(2).unwrap());
        let mut incoming: Vec<_> = names
            .into_iter()
            .rev()
            .map(|name| attribute(name, "value"))
            .collect();
        slot(edit.append(&mut incoming).unwrap());
        let mut replay = vec![edit.attributes().unwrap()[0].clone()];
        let mut pending = edit.append(&mut replay).unwrap();
        let error = loop {
            match pending.next_action().unwrap().unwrap().perform() {
                Ok(v) => v.accept(),
                Err(e) => break e,
            }
        };
        assert_eq!(
            error,
            HtmlConstructionErrorKind::AttributeNameMismatch.error()
        );
        drop(pending);
        assert_eq!(replay.len(), 1);
        assert_eq!(incoming.len(), 1);
        assert_eq!(edit.attributes().unwrap().len(), 1);
        assert_eq!(
            edit.attributes().unwrap()[0]
                .name()
                .local_name()
                .to_utf8()
                .unwrap(),
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
            Err(ref error) if error.kind == HtmlConstructionErrorKind::OperationNotReady
        ));
        assert_eq!(excess.len(), 1);
        assert_eq!(edit.attributes().unwrap().len(), 2);
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
            slots::candidate(&mut candidate).unwrap();
        }
        // Original exact comparisons: old a:a, old a:b, prior a:b, one u16 each.
        assert_eq!(inspected, 3 * 2 * size_of::<u16>());
        assert_eq!(witnesses, 2);
        assert!(
            names
                .iter()
                .all(|name| Arc::strong_count(name.shared_code_units()) == 1)
        );
        let snapshot = edit.stage.plan().unwrap();
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
        slots::attribute_slot(&mut pending).unwrap().commit();
        drop(pending);
        let mut owner = Some(edit);
        let (_, retirement) = slots::edit(&mut owner).unwrap().into_parts();
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
            slots::edit(&mut edit),
            Err(ref error) if error.kind == HtmlConstructionErrorKind::OperationNotReady
        ));
        assert!(matches!(
            edit.as_ref().unwrap().selected_count(),
            Err(ref error) if error.kind == HtmlConstructionErrorKind::OperationNotReady
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
                slots::candidate(&mut candidate).unwrap();
            }
            assert!(reserves >= 2, "exercise multiple actual plan allocations");
            let report = edit.stage.plan().unwrap().clone();
            if let Some(previous) = &previous {
                assert!(report.id().same_owner_role(previous));
                assert_ne!(report.id(), previous);
            }
            assert_eq!(
                Arc::as_ptr(edit.shared_attributes().unwrap()),
                original_backing
            );
            assert_eq!(edit.attributes().unwrap().len(), 1);
            assert_eq!(
                edit.attributes().unwrap()[0].value().to_utf8().unwrap(),
                "retained"
            );
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

    /// Planning n candidates one at a time requests a logarithmic number of plan reallocations in
    /// n when the consumer requests a doubling capacity, up to the candidate count. A reservation
    /// that changes the plan's capacity is exactly one reallocation request, and each is visible in
    /// the allocation the observation reports. The oracle is the doubling bound: from any positive
    /// capacity, `ceil(log2 n)` doublings reach `n`, so `floor(log2 n) + 2` changes allow the first
    /// allocation and one more for rounding.
    #[test]
    fn plan_reallocation_requests_for_n_candidates_grow_at_most_logarithmically() {
        for count in [16_usize, 128, 512] {
            let (storage, target) = make_storage(Vec::new());
            let names: Vec<_> = (0..count)
                .map(|i| DocumentString::from(format!("a{i}").as_str()))
                .collect();
            let mut edit = make_edit(
                storage,
                target,
                HtmlConstructionAttributeEditMode::NamedMerge {
                    candidate_count: count,
                },
            );
            let mut capacity = 0;
            let mut changes = 0;
            for (index, name) in names.iter().enumerate() {
                let mut candidate = Some(edit.candidate(index, name).unwrap());
                while let Some(action) = candidate.as_mut().unwrap().next_action().unwrap() {
                    let description = action.description();
                    let prior = description.prior_index().map(|prior| &names[prior]);
                    let doubled = match description {
                        HtmlConstructionAttributeDescription::ReservePlan {
                            allocation,
                            minimum_capacity,
                            ..
                        } => Some(minimum_capacity.max(2 * allocation.capacity()).min(count)),
                        _ => None,
                    };
                    let observation = match doubled {
                        Some(doubled) => action.perform_with_capacity(prior, doubled),
                        None => action.perform(prior),
                    }
                    .unwrap();
                    if let Some(allocation) = observation.allocation()
                        && allocation.capacity() != capacity
                    {
                        capacity = allocation.capacity();
                        changes += 1;
                    }
                    observation.accept();
                }
                slots::candidate(&mut candidate).unwrap();
            }
            assert_eq!(edit.decisions.len(), count);
            let allowed = count.ilog2() as usize + 2;
            assert!(
                changes <= allowed,
                "{count} candidates changed the plan capacity {changes} times, more than {allowed}"
            );
            let (_, retirement) = edit.abort().into_parts();
            assert_eq!(retire(retirement).0, count);
        }
    }

    /// The reservation's minimum is the least capacity that holds the candidate's decision; a
    /// smaller request, or any request on another action, is refused before the plan is touched.
    #[test]
    fn plan_capacity_below_the_minimum_or_for_another_action_is_refused_before_any_reserve() {
        let (storage, target) = make_storage(Vec::new());
        let names = ["a", "b"].map(DocumentString::from);
        let mut edit = make_edit(
            storage,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 2 },
        );
        plan(&mut edit, &names[..1]);
        let mut candidate = Some(edit.candidate(1, &names[1]).unwrap());
        // The candidate's first action does not reserve, so a capacity is refused.
        let action = candidate.as_mut().unwrap().next_action().unwrap().unwrap();
        assert!(!matches!(
            action.description(),
            HtmlConstructionAttributeDescription::ReservePlan { .. }
        ));
        assert_eq!(
            action.perform_with_capacity(None, 8).err().unwrap(),
            HtmlConstructionErrorKind::CapacityMismatch.error()
        );
        drop(candidate);
        let (_, retirement) = edit.abort().into_parts();
        assert_eq!(retire(retirement).0, 1);

        let (storage, target) = make_storage(Vec::new());
        let mut edit = make_edit(
            storage,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 2 },
        );
        plan(&mut edit, &names[..1]);
        let mut candidate = Some(edit.candidate(1, &names[1]).unwrap());
        // Step to the second candidate's reservation, which needs two slots.
        let refused = loop {
            let action = candidate.as_mut().unwrap().next_action().unwrap().unwrap();
            let description = action.description();
            let prior = description.prior_index().map(|prior| &names[prior]);
            if matches!(
                description,
                HtmlConstructionAttributeDescription::ReservePlan {
                    minimum_capacity: 2,
                    ..
                }
            ) {
                break action.perform_with_capacity(prior, 1).err().unwrap();
            }
            action.perform(prior).unwrap().accept();
        };
        assert_eq!(refused, HtmlConstructionErrorKind::CapacityMismatch.error());
        drop(candidate);
        // The refusal reserved nothing: the plan keeps its single slot.
        assert_eq!(edit.decisions.capacity(), 1);
        let (_, retirement) = edit.abort().into_parts();
        assert_eq!(retire(retirement).0, 1);
    }

    // ---- Name index ----

    /// A deterministic generator, so a failing case is replayable.
    struct Random(u64);
    impl Random {
        fn next(&mut self, bound: usize) -> usize {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 % bound as u64) as usize
        }
    }
    fn qualified(namespace: &str, prefix: Option<&str>, local: &str) -> Attribute {
        Attribute::new(
            QualifiedName::new(
                Namespace::from_uri(namespace),
                prefix.map(DocumentString::from),
                DocumentString::from(local),
            ),
            "v",
        )
    }
    fn new_index(entries: usize) -> HtmlAttributeNameIndex {
        let slots = HtmlAttributeNameIndex::slots_required(entries).unwrap();
        HtmlAttributeNameIndex::new(Vec::<HtmlNameIndexSlot>::with_capacity(slots))
    }
    /// The work a driver admits, in every category, plus one per action.
    #[derive(Default)]
    struct Admitted(u64);
    impl Admitted {
        fn add(&mut self, work: HtmlConstructionAttributeWork) {
            self.0 += (work.record_visits
                + work.provider_calls
                + work.inspected_bytes
                + work.copied_bytes
                + work.record_writes
                + work.auxiliary_entries) as u64
                + 1;
        }
    }
    /// One append of `incoming` in its own Slots edit that borrows `index` and gives it back, as a
    /// consumer does for each attribute of an element it builds one at a time. The error is the
    /// first refusal; the edit is then abandoned.
    fn append_in_new_edit(
        storage: HtmlConstructionStorage,
        index: HtmlAttributeNameIndex,
        incoming: Attribute,
        admitted: &mut Admitted,
    ) -> (
        HtmlConstructionStorage,
        HtmlAttributeNameIndex,
        Result<(), HtmlConstructionError>,
    ) {
        let target = storage.node_at(0).unwrap().handle();
        let mut edit = HtmlConstructionAttributeEdit::begin(
            storage,
            target,
            HtmlConstructionAttributeEditMode::Slots,
        );
        edit.adopt_index(index);
        while let Some(action) = edit.next_action().unwrap() {
            admitted.add(action.description().work());
            action.perform().unwrap().accept();
        }
        let mut incoming = vec![incoming];
        let result = (|| {
            let mut reserve = Some(edit.reserve(1)?);
            while let Some(action) = reserve.as_mut().unwrap().next_action()? {
                admitted.add(action.description().work());
                action.perform()?.accept();
            }
            slots::attribute_slot(&mut reserve)?.commit();
            drop(reserve);
            let mut append = Some(edit.append(&mut incoming)?);
            while let Some(action) = append.as_mut().unwrap().next_action()? {
                admitted.add(action.description().work());
                action.perform()?.accept();
            }
            slots::attribute_slot(&mut append)?.commit();
            Ok(())
        })();
        let mut completion = match result {
            Ok(()) => {
                let mut owner = Some(edit);
                slots::edit(&mut owner).unwrap()
            }
            Err(_) => edit.abort(),
        };
        let index = completion.take_index().expect("the edit held the index");
        let (storage, retirement) = completion.into_parts();
        retire(retirement);
        (storage, index, result)
    }

    #[test]
    fn indexed_appends_agree_with_a_set_of_names_through_swaps_pops_and_rebuilds() {
        let mut random = Random(0x2545_f491_4f6c_dd1d);
        const LOCALS: [&str; 6] = ["a", "A", "b", "ab", "", "x\u{0}"];
        const NAMESPACES: [&str; 3] = ["", "", "urn:n"];
        for round in 0..40 {
            let (mut storage, _) = make_storage(Vec::new());
            let mut index = new_index(400);
            let mut expected: Vec<(String, String)> = Vec::new();
            let mut admitted = Admitted::default();
            for step in 0..120 {
                let kind = random.next(10);
                if kind == 0 && expected.len() >= 2 {
                    // A change that is not an index-aware append: the index must notice.
                    let target = storage.node_at(0).unwrap().handle();
                    let mut edit = HtmlConstructionAttributeEdit::begin(
                        storage,
                        target,
                        HtmlConstructionAttributeEditMode::Slots,
                    );
                    edit.adopt_index(index);
                    while let Some(action) = edit.next_action().unwrap() {
                        action.perform().unwrap().accept();
                    }
                    if random.next(2) == 0 {
                        let at = random.next(expected.len() - 1);
                        slot(edit.swap_adjacent(at).unwrap());
                        expected.swap(at, at + 1);
                    } else {
                        slot(edit.pop().unwrap());
                        expected.pop();
                    }
                    let mut owner = Some(edit);
                    let mut completion = slots::edit(&mut owner).unwrap();
                    index = completion.take_index().unwrap();
                    storage = completion.into_parts().0;
                    continue;
                }
                let namespace = NAMESPACES[random.next(NAMESPACES.len())];
                let local = LOCALS[random.next(LOCALS.len())];
                let candidate = if random.next(3) == 0 {
                    format!("{local}{}", random.next(60))
                } else {
                    local.to_string()
                };
                let incoming = qualified(namespace, (step % 2 == 0).then_some("p"), &candidate);
                let key = (namespace.to_string(), candidate.clone());
                let owner = expected.iter().position(|held| *held == key);
                let (next, back, result) =
                    append_in_new_edit(storage, index, incoming, &mut admitted);
                storage = next;
                index = back;
                match (owner, result) {
                    (None, Ok(())) => expected.push(key),
                    (Some(first), Err(error)) => assert_eq!(
                        error,
                        HtmlConstructionErrorKind::DuplicateAttribute {
                            first,
                            duplicate: expected.len()
                        }
                        .error(),
                        "round {round} step {step}"
                    ),
                    (owner, result) => panic!("round {round} step {step}: {owner:?} {result:?}"),
                }
                let stored: Vec<(String, String)> = match storage.node_at(0).unwrap().kind() {
                    HtmlNodeKind::Element(element) => element
                        .attributes()
                        .as_slice()
                        .iter()
                        .map(|held| {
                            (
                                held.name()
                                    .namespace()
                                    .uri()
                                    .map_or(String::new(), |uri| uri.to_utf8().unwrap()),
                                held.name().local_name().to_utf8().unwrap(),
                            )
                        })
                        .collect(),
                    _ => unreachable!(),
                };
                assert_eq!(stored, expected, "round {round} step {step}");
            }
        }
    }

    #[test]
    fn an_index_without_room_is_ignored_and_the_append_compares_every_name() {
        let (storage, _) = make_storage(vec![attribute("a", "1"), attribute("b", "2")]);
        let mut admitted = Admitted::default();
        let (storage, index, result) = append_in_new_edit(
            storage,
            HtmlAttributeNameIndex::new(Vec::<HtmlNameIndexSlot>::new()),
            attribute("b", "3"),
            &mut admitted,
        );
        assert_eq!(
            result,
            Err(HtmlConstructionErrorKind::DuplicateAttribute {
                first: 1,
                duplicate: 2
            }
            .error())
        );
        let (_, _, result) = append_in_new_edit(storage, index, attribute("c", "3"), &mut admitted);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn indexed_appends_of_distinct_names_at_most_double_in_work_with_their_number() {
        let work = |count: usize| {
            let (mut storage, _) = make_storage(Vec::new());
            let mut index = new_index(count + 8);
            let mut admitted = Admitted::default();
            for i in 0..count {
                let (next, back, result) = append_in_new_edit(
                    storage,
                    index,
                    attribute(format!("attribute{i}"), "v"),
                    &mut admitted,
                );
                result.unwrap();
                storage = next;
                index = back;
            }
            admitted.0
        };
        let measured = [250, 500, 1000, 2000].map(work);
        for pair in measured.windows(2) {
            assert!(
                pair[1] * 10 <= pair[0] * 22,
                "doubling the appends took work from {} to {} ({measured:?})",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn indexed_named_merge_decides_as_the_pairwise_plan_on_random_names() {
        let mut random = Random(0x9e37_79b9_7f4a_7c15);
        for round in 0..300 {
            let installed: Vec<Attribute> = {
                let mut names: Vec<(String, String)> = Vec::new();
                (0..random.next(6))
                    .filter_map(|_| {
                        let namespace = ["", "urn:n"][random.next(2)];
                        let local = ["a", "b", "c", ""][random.next(4)];
                        let key = (namespace.to_string(), local.to_string());
                        (!names.contains(&key)).then(|| {
                            names.push(key);
                            qualified(namespace, None, local)
                        })
                    })
                    .collect()
            };
            let names: Vec<DocumentString> = (0..random.next(12))
                .map(|_| DocumentString::from(["a", "b", "c", "d", "A", "", "ab"][random.next(7)]))
                .collect();
            let decide = |index: Option<HtmlAttributeNameIndex>| {
                let (storage, target) = make_storage(installed.clone());
                let mut edit = make_edit(
                    storage,
                    target,
                    HtmlConstructionAttributeEditMode::NamedMerge {
                        candidate_count: names.len(),
                    },
                );
                if let Some(index) = index {
                    edit.adopt_index(index);
                }
                plan(&mut edit, &names);
                let decisions: Vec<bool> = (0..names.len())
                    .map(|i| edit.decision(i).unwrap())
                    .collect();
                retire(edit.abort().into_parts().1);
                decisions
            };
            let expected = decide(None);
            let got = decide(Some(new_index(installed.len() + names.len())));
            assert_eq!(
                got,
                expected,
                "round {round}: {names:?} against {} installed",
                installed.len()
            );
        }
    }

    #[test]
    fn indexed_named_merge_planning_at_most_doubles_in_work_with_the_candidate_count() {
        let work = |count: usize| {
            let (storage, target) = make_storage(vec![attribute("existing", "v")]);
            let names: Vec<DocumentString> = (0..count)
                .map(|i| DocumentString::from(format!("candidate{i}")))
                .collect();
            let mut edit = HtmlConstructionAttributeEdit::begin(
                storage,
                target,
                HtmlConstructionAttributeEditMode::NamedMerge {
                    candidate_count: count,
                },
            );
            edit.adopt_index(new_index(count + 1));
            let mut admitted = Admitted::default();
            while let Some(action) = edit.next_action().unwrap() {
                admitted.add(action.description().work());
                action.perform().unwrap().accept();
            }
            for (index, name) in names.iter().enumerate() {
                let mut candidate = Some(edit.candidate(index, name).unwrap());
                while let Some(action) = candidate.as_mut().unwrap().next_action().unwrap() {
                    let description = action.description();
                    admitted.add(description.work());
                    let prior = description.prior_index().map(|prior| &names[prior]);
                    action.perform(prior).unwrap().accept();
                }
                slots::candidate(&mut candidate).unwrap();
            }
            let mut completion = edit.abort();
            completion.take_index();
            retire(completion.into_parts().1);
            admitted.0
        };
        let measured = [250, 500, 1000, 2000].map(work);
        for pair in measured.windows(2) {
            assert!(
                pair[1] * 10 <= pair[0] * 22,
                "doubling the candidates took work from {} to {}",
                pair[0],
                pair[1]
            );
        }
    }
}
