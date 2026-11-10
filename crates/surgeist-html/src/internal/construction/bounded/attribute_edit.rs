//! Complete, exclusive edits of the actual ordered attribute backing.

use super::operations::attribute_units;
use super::*;
use crate::internal::accounting::{HtmlChargedError, HtmlResource, HtmlWork, wide};
use crate::internal::values::attribute_name_index::{
    Binding, CLEAR_CHUNK, HtmlAttributeNameIndex, Occupant, entry, occupant, tag,
};
use crate::internal::values::bounded::{
    ComparisonStep, Phase as NamePhase, advance_comparison, comparison_layout,
};
use crate::internal::values::name_index::{
    HtmlNameIndex, HtmlNameIndexProbe, HtmlNameIndexSlot, HtmlNameIndexSlots, HtmlNameIndexStep,
    HtmlNameIndexVacancy,
};
use crate::internal::values::{Attribute, ReserveOutcome, WeakTextBlock};
use surgeist_ledger::{Change, Charge};

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
///
/// The amounts are not readable. A consumer charges them through
/// [`HtmlConstructionAttributeDescription::charge`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlConstructionAttributeWork {
    /// The retained records the action reads or updates.
    pub(crate) record_visits: usize,
    /// The provider requests the action makes.
    pub(crate) provider_calls: usize,
    /// The bytes of name text the action compares without copying.
    pub(crate) inspected_bytes: usize,
    /// The bytes the action copies, including bookkeeping values it moves.
    pub(crate) copied_bytes: usize,
    /// The record fields the action writes.
    pub(crate) record_writes: usize,
    /// Largest indivisible physical transfer or individual fixed field write.
    /// This is a field-wise atomic maximum, not the sum of copied bytes.
    pub(crate) atomic_layout_bytes: usize,
    /// The name-witness entries the action creates or destroys. Only planning and plan cleanup use
    /// it.
    pub(crate) auxiliary_entries: usize,
}

impl HtmlConstructionAttributeWork {
    /// Charges the step's work: the record visits, provider calls, inspected bytes, copied bytes
    /// and record writes as one batch, then a nonzero atomic layout as the largest reached, then a
    /// nonzero count of auxiliary entries as one acquisition. `prepaid_provider_calls` are calls
    /// of the step that the caller has already charged, so they are left out of the batch.
    pub(crate) fn charge<C: Charge<HtmlWork, HtmlResource>>(
        self,
        prepaid_provider_calls: u64,
        charge: &mut C,
    ) -> Result<(), HtmlChargedError<C::Stop, HtmlConstructionChargeFault>> {
        let provider_calls = wide(self.provider_calls)
            .checked_sub(prepaid_provider_calls)
            .ok_or(HtmlChargedError::Faulted(
                HtmlConstructionChargeFault::PrepaidCallsExceedPlan,
            ))?;
        charge
            .charge(&[
                Change::Work(HtmlWork::RecordVisit, wide(self.record_visits)),
                Change::Work(HtmlWork::ProviderCall, provider_calls),
                Change::Work(HtmlWork::InspectByte, wide(self.inspected_bytes)),
                Change::Work(HtmlWork::CopyByte, wide(self.copied_bytes)),
                Change::Work(HtmlWork::RecordWrite, wide(self.record_writes)),
            ])
            .map_err(HtmlChargedError::Stopped)?;
        if self.atomic_layout_bytes != 0 {
            charge
                .record_largest(
                    HtmlResource::AtomicLayoutBytes,
                    wide(self.atomic_layout_bytes),
                )
                .map_err(HtmlChargedError::Stopped)?;
        }
        if self.auxiliary_entries != 0 {
            charge
                .charge(&[Change::Acquire(
                    HtmlResource::AuxiliaryEntries,
                    wide(self.auxiliary_entries),
                )])
                .map_err(HtmlChargedError::Stopped)?;
        }
        Ok(())
    }
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
    /// Charges the work of this step, whichever step it is, as
    /// [`HtmlConstructionAttributeWork`] describes. `prepaid_provider_calls` are provider calls of
    /// the step that the caller has already charged, such as the one allocator call that its own
    /// reservation policy admits.
    pub fn charge<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        prepaid_provider_calls: u64,
        charge: &mut C,
    ) -> Result<(), HtmlChargedError<C::Stop, HtmlConstructionChargeFault>> {
        self.work().charge(prepaid_provider_calls, charge)
    }
    /// The work of this step, whichever step it is.
    pub(crate) fn work(&self) -> HtmlConstructionAttributeWork {
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
    name: WeakTextBlock,
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
        _ => Err(HtmlConstructionCause::OperationNotReady.into()),
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

    /// As [`into_parts`](Self::into_parts), and the name index too if it was not taken, so that
    /// the consumer can always release the storage it lent.
    pub fn into_parts_with_index(
        self,
    ) -> (
        HtmlConstructionStorage,
        HtmlConstructionAttributePlanRetirement,
        Option<HtmlAttributeNameIndex>,
    ) {
        (self.storage, self.retirement, self.index)
    }

    /// Returns the storage and the retirement owner for the merge plan. A name index that was not
    /// taken with [`take_index`](Self::take_index) is dropped with its storage. Cleanup the plan
    /// with
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

    /// Starts an attribute edit as [`begin`](Self::begin) does, lent a name index. Appends to an
    /// element, and the planning of a merge, then find a duplicate name by probing the few names
    /// that share a hash, so an edit that adds `n` names costs work linear in `n` and their
    /// length instead of quadratic. An index with too little capacity for the names (see
    /// [`HtmlAttributeNameIndex::slots_required`]) is ignored by the step that needs more, which
    /// then compares every name.
    ///
    /// An index can only be lent when an edit begins, so that every name it describes was
    /// entered by a step of an edit that held it. An index kept from an earlier edit describes
    /// the same element while the revision of the storage's attributes is the one it recorded;
    /// any change made since, by an edit that held no index or by another element's edit, makes
    /// the next step that needs the index rebuild it first. The index is returned by
    /// [`HtmlConstructionAttributeCompletion::take_index`] and its variants.
    pub fn begin_indexed(
        storage: HtmlConstructionStorage,
        element: HtmlBuilderNode,
        mode: HtmlConstructionAttributeEditMode,
        mut index: HtmlAttributeNameIndex,
    ) -> Self {
        if matches!(mode, HtmlConstructionAttributeEditMode::NamedMerge { .. }) {
            index.bind(Binding::Unbound);
        }
        let mut edit = Self::begin(storage, element, mode);
        edit.index = Some(index);
        edit
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
            .ok_or(HtmlConstructionCause::OperationNotReady)?;
        match self
            .storage
            .arena
            .records
            .get(target)
            .map(|record| &record.kind)
        {
            Some(HtmlNodeKind::Element(v)) => Ok(v.attributes.shared_attributes()),
            _ => Err(HtmlConstructionCause::OperationNotReady.into()),
        }
    }
    /// The attribute backing's dimensions as last observed.
    pub fn capacity(&self) -> HtmlConstructionAttributeCapacity {
        self.capacity
    }
    fn attributes(&self) -> Result<&[Attribute], HtmlConstructionError> {
        Ok(self.shared_attributes()?.as_slice())
    }
    /// The element's backing, taken for a change: the storage's attribute revision advances.
    fn backing_mut(&mut self) -> Result<&mut Arc<Vec<Attribute>>, HtmlConstructionError> {
        let target = self
            .stage
            .target()
            .ok_or(HtmlConstructionCause::OperationNotReady)?;
        let arena = &mut self.storage.arena;
        match arena.records.get_mut(target).map(|record| &mut record.kind) {
            Some(HtmlNodeKind::Element(v)) => Ok(v
                .attributes
                .construction_backing_mut(&mut arena.attribute_revision)),
            _ => Err(HtmlConstructionCause::OperationNotReady.into()),
        }
    }
    /// Reserves capacity in the element's backing without changing any attribute.
    fn reserve_backing(
        &mut self,
        additional: usize,
    ) -> Result<Option<ReserveOutcome>, HtmlConstructionError> {
        let target = self
            .stage
            .target()
            .ok_or(HtmlConstructionCause::OperationNotReady)?;
        match self
            .storage
            .arena
            .records
            .get_mut(target)
            .map(|record| &mut record.kind)
        {
            Some(HtmlNodeKind::Element(v)) => {
                Ok(v.attributes.reserve_exact_in_construction(additional))
            }
            _ => Err(HtmlConstructionCause::OperationNotReady.into()),
        }
    }
    /// The element's attribute vector for a ready slot's commit. The slot's `Unique` action
    /// checked, under this edit's exclusive ownership of the storage, that the target is an
    /// element whose backing has no other strong or weak owner. The commit checks it again, and a
    /// backing held by another owner is refused before the storage's attribute revision advances.
    /// The borrow that advances it is taken last, once nothing else can refuse the step.
    fn exclusive_backing(&mut self) -> Result<&mut Vec<Attribute>, HtmlConstructionError> {
        let backing = self.shared_attributes()?;
        if Arc::strong_count(backing) != 1 || Arc::weak_count(backing) != 0 {
            return Err(HtmlConstructionCause::SharedAllocationAliased.into());
        }
        Arc::get_mut(self.backing_mut()?)
            .ok_or_else(|| HtmlConstructionCause::SharedAllocationAliased.into())
    }
    fn check_ready(&self) -> Result<(), HtmlConstructionError> {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        if !self.stage.is_ready() || self.observing || self.busy {
            return Err(HtmlConstructionCause::OperationNotReady.into());
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
            return Err(HtmlConstructionCause::OperationNotReady.into());
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
            return Err(HtmlConstructionCause::AttributeOrderMismatch.into());
        };
        if index != self.decisions.len() || index >= candidate_count {
            return Err(HtmlConstructionCause::AttributeOrderMismatch.into());
        }
        let target = match &self.stage {
            EditStage::Merge { target, .. } => *target,
            _ => return Err(HtmlConstructionCause::OperationNotReady.into()),
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
                phase = CandidatePhase::IndexBegin { geometry };
            }
        }
        let EditStage::Merge { plan, .. } = &mut self.stage else {
            return Err(HtmlConstructionCause::OperationNotReady.into());
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
            phase: CandidateState::Pending(phase),
            found: false,
            complete: false,
            failure: None,
            table: &mut self.index,
            probe: None,
            tag: 0,
        })
    }
    /// Whether the planned candidate `index` is kept: `true` if no existing attribute and no
    /// earlier candidate has the same name.
    pub fn decision(&self, index: usize) -> Result<bool, HtmlConstructionError> {
        self.check_ready()?;
        self.decisions
            .get(index)
            .map(|v| v.keep)
            .ok_or(HtmlConstructionCause::AttributeOrderMismatch.error())
    }
    /// The number of kept candidates. Available once every candidate is planned.
    pub fn selected_count(&self) -> Result<usize, HtmlConstructionError> {
        self.check_ready()?;
        if !self.planning_complete() {
            return Err(HtmlConstructionCause::OperationNotReady.into());
        }
        Ok(self.selected)
    }
    fn slot<'a>(
        &'a mut self,
        input: SlotInput<'a>,
    ) -> Result<HtmlConstructionAttributeSlot<'a>, HtmlConstructionError> {
        self.check_ready()?;
        if !self.planning_complete() || self.used {
            return Err(HtmlConstructionCause::OperationNotReady.into());
        }
        if matches!(
            self.mode,
            HtmlConstructionAttributeEditMode::NamedMerge { .. }
        ) && !matches!(input, SlotInput::Append(_) | SlotInput::Reserve(_))
        {
            return Err(HtmlConstructionCause::AttributeOrderMismatch.into());
        }
        let usage = self.storage.usage();
        Ok(HtmlConstructionAttributeSlot {
            edit: self,
            input,
            phase: SlotState::Pending(SlotPhase::Input),
            failure: None,
            finished: false,
            usage,
            minimum_capacity: 0,
            plan_index: 0,
            insert_slot: None,
            probe: None,
            tag: 0,
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
                HtmlConstructionCause::OperationNotReady,
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
                        return Err(HtmlConstructionCause::WrongNodeKind.into());
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
                        .ok_or(HtmlConstructionCause::RepresentationExhausted)?;
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
                    return Err(HtmlConstructionCause::OperationNotReady.into());
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
    /// Start rebuilding the index with this geometry.
    IndexBegin {
        geometry: HtmlNameIndexSlots,
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
    /// Enter the existing attribute at `position` in the first empty slot of the probe.
    IndexInsert {
        position: usize,
    },
    /// Move the candidate's probe to its next slot and read it.
    IndexProbe,
    /// Comparing the candidate with the existing attribute at `position` that the probe found.
    IndexTargetName {
        position: usize,
        name: NamePhase,
    },
    /// The probe found the earlier candidate `index`; its name is needed.
    IndexPrior {
        index: usize,
    },
    /// Comparing the candidate with the earlier candidate's name that the probe found.
    IndexPriorName {
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
}
/// Where the planning of one candidate stands: an action is pending, the observation of a
/// performed action awaits acceptance, or planning is finished. Only a pending action has a phase,
/// so an action cannot exist without one.
#[derive(Clone, Copy, Debug)]
enum CandidateState<'n> {
    Pending(CandidatePhase<'n>),
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
    phase: CandidateState<'n>,
    found: bool,
    complete: bool,
    failure: Option<HtmlConstructionError>,
    table: &'e mut Option<HtmlAttributeNameIndex>,
    /// The probe of the name being looked up or entered, and its tag.
    probe: Option<HtmlNameIndexProbe>,
    tag: u32,
}
/// One pending candidate action. It borrows the candidate exclusively; `perform` consumes it.
#[derive(Debug)]
pub struct HtmlConstructionAttributeCandidateAction<'a, 'e, 'n> {
    candidate: &'a mut HtmlConstructionAttributeCandidate<'e, 'n>,
    /// The phase this action performs.
    phase: CandidatePhase<'n>,
}
/// The observation after a candidate action. It borrows the candidate exclusively; `accept`
/// resumes it.
#[derive(Debug)]
pub struct HtmlConstructionAttributeCandidateObservation<'a, 'e, 'n> {
    candidate: &'a mut HtmlConstructionAttributeCandidate<'e, 'n>,
    /// The state this accepted observation resumes.
    next: CandidateState<'n>,
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
        match self.phase {
            CandidateState::Observing => Err(HtmlConstructionCause::OperationNotReady.into()),
            CandidateState::Done => Ok(None),
            CandidateState::Pending(phase) => Ok(Some(HtmlConstructionAttributeCandidateAction {
                candidate: self,
                phase,
            })),
        }
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
        if !matches!(self.phase, CandidateState::Done) {
            return Err(HtmlConstructionRefusal::new(
                self,
                HtmlConstructionCause::OperationNotReady,
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
/// The name index could not serve a step: its storage, probe or vacancy did not agree with the
/// table the step expected.
fn table_error() -> HtmlConstructionError {
    HtmlConstructionCause::IndexUnavailable.into()
}
/// The work of reading one slot of the index, which writes the empty slot it finds, counts the
/// entry and may renew what the index describes.
fn probe_work() -> HtmlConstructionAttributeWork {
    HtmlConstructionAttributeWork {
        record_visits: 1,
        record_writes: 3,
        copied_bytes: size_of::<HtmlNameIndexSlot>() + size_of::<usize>() + size_of::<Binding>(),
        ..Default::default()
    }
}
/// The work of starting a rebuild: a new index over the storage, an unbound table and an empty
/// storage.
fn begin_clear_work() -> HtmlConstructionAttributeWork {
    HtmlConstructionAttributeWork {
        record_writes: 3,
        copied_bytes: size_of::<HtmlNameIndex>() + size_of::<Binding>() + size_of::<usize>(),
        ..Default::default()
    }
}
/// The work of clearing the next slots of the index. The action that finishes the table also
/// binds it to its index, which reads each slot once.
fn clear_work(table: Option<&HtmlAttributeNameIndex>) -> HtmlConstructionAttributeWork {
    let slots = table
        .map_or(0, HtmlAttributeNameIndex::unwritten)
        .min(CLEAR_CHUNK);
    let scanned = table.map_or(0, |table| table.scanned_by_clearing(CLEAR_CHUNK));
    // The chunk that finishes the table also records the storage the index guards and what the
    // table describes.
    let finishing = usize::from(scanned != 0);
    HtmlConstructionAttributeWork {
        record_writes: slots + 2 * finishing,
        copied_bytes: slots * size_of::<HtmlNameIndexSlot>()
            + finishing * (size_of::<usize>() + size_of::<Binding>()),
        inspected_bytes: scanned * size_of::<HtmlNameIndexSlot>(),
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
        match self.phase {
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
            CandidatePhase::IndexBegin { .. } => {
                HtmlConstructionAttributeDescription::Inspect(begin_clear_work())
            }
            CandidatePhase::IndexClear => {
                HtmlConstructionAttributeDescription::Inspect(clear_work(c.table.as_ref()))
            }
            CandidatePhase::IndexHashLength { .. } => {
                HtmlConstructionAttributeDescription::Inspect(hash_length_work())
            }
            CandidatePhase::IndexHash { units, .. } => {
                HtmlConstructionAttributeDescription::Inspect(hash_work(units))
            }
            CandidatePhase::IndexInsert { .. } | CandidatePhase::IndexProbe => {
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
        }
        .transitions::<CandidateState>()
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
        let phase = self.phase;
        // Owner state, rather than Drop alone, prevents a forgotten cursor or
        // observation from making an unaccepted admission usable.
        if !*c.busy {
            *c.busy = true;
        }
        let mut allocation = false;
        let result = (|| -> Result<_, HtmlConstructionError> {
            if reserved.is_some_and(|capacity| {
                !matches!(phase, CandidatePhase::Reserve) || capacity <= c.index
            }) {
                return Err(HtmlConstructionCause::CapacityMismatch.into());
            }
            Ok(CandidateState::Pending(match phase {
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
                        let prior = prior.ok_or(HtmlConstructionCause::AttributeNameMismatch)?;
                        if c.decisions[index].name.address() != prior.block().address() {
                            return Err(HtmlConstructionCause::AttributeNameMismatch.into());
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
                CandidatePhase::IndexBegin { geometry } => {
                    c.table
                        .as_mut()
                        .ok_or_else(table_error)?
                        .begin_clear(geometry);
                    CandidatePhase::IndexClear
                }
                CandidatePhase::IndexClear => {
                    let table = c.table.as_mut().ok_or_else(table_error)?;
                    table.clear_chunk(CLEAR_CHUNK);
                    if table.unwritten() == 0 {
                        table.bind_storage().map_err(|_| table_error())?;
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
                    let table = c.table.as_mut().ok_or_else(table_error)?;
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
                    c.probe = Some(table.probe(hash).map_err(|_| table_error())?);
                    c.tag = tag(hash);
                    match subject {
                        Subject::Existing(position) => CandidatePhase::IndexInsert { position },
                        Subject::Incoming => CandidatePhase::IndexProbe,
                    }
                }
                CandidatePhase::IndexInsert { position } => {
                    let table = c.table.as_mut().ok_or_else(table_error)?;
                    let probe = c.probe.as_mut().ok_or_else(table_error)?;
                    match table.advance(probe).map_err(|_| table_error())? {
                        Some(HtmlNameIndexStep::Vacant(vacancy)) => {
                            let placed = entry(c.tag, Occupant::Attribute(position))
                                .is_some_and(|entry| table.insert(vacancy, entry).is_ok());
                            if !placed {
                                return Err(table_error());
                            }
                            c.probe = None;
                            c.after_existing(position)?
                        }
                        Some(HtmlNameIndexStep::Occupied(_)) => {
                            CandidatePhase::IndexInsert { position }
                        }
                        None => return Err(table_error()),
                    }
                }
                CandidatePhase::IndexProbe => {
                    let table = c.table.as_mut().ok_or_else(table_error)?;
                    let probe = c.probe.as_mut().ok_or_else(table_error)?;
                    match table.advance(probe).map_err(|_| table_error())? {
                        Some(HtmlNameIndexStep::Vacant(vacancy)) => {
                            // Nothing has the name: the candidate is kept and entered.
                            let placed = entry(c.tag, Occupant::Candidate(c.index))
                                .is_some_and(|entry| table.insert(vacancy, entry).is_ok());
                            if !placed {
                                return Err(table_error());
                            }
                            c.probe = None;
                            c.settled()
                        }
                        Some(HtmlNameIndexStep::Occupied(held)) => match occupant(held) {
                            (_, held_tag) if held_tag != c.tag => CandidatePhase::IndexProbe,
                            (Occupant::Candidate(index), _) if index < c.index => {
                                CandidatePhase::IndexPrior { index }
                            }
                            (Occupant::Attribute(position), _) => CandidatePhase::IndexTargetName {
                                position,
                                name: NamePhase::LocalLength,
                            },
                            _ => return Err(table_error()),
                        },
                        None => return Err(table_error()),
                    }
                }
                CandidatePhase::IndexTargetName { position, name } => {
                    let old = element_attributes(c.storage, c.target)?
                        .get(position)
                        .ok_or_else(table_error)?
                        .name()
                        .local_name();
                    match advance_comparison(name, None, old, None, c.name) {
                        ComparisonStep::Continue(name) => {
                            CandidatePhase::IndexTargetName { position, name }
                        }
                        ComparisonStep::Different => CandidatePhase::IndexProbe,
                        ComparisonStep::Equal => {
                            c.found = true;
                            c.probe = None;
                            c.settled()
                        }
                    }
                }
                CandidatePhase::IndexPrior { index } => {
                    let prior = prior.ok_or(HtmlConstructionCause::AttributeNameMismatch)?;
                    if c.decisions[index].name.address() != prior.block().address() {
                        return Err(HtmlConstructionCause::AttributeNameMismatch.into());
                    }
                    CandidatePhase::IndexPriorName {
                        index,
                        name: NamePhase::LocalLength,
                        prior,
                    }
                }
                CandidatePhase::IndexPriorName { index, name, prior } => {
                    match advance_comparison(name, None, prior, None, c.name) {
                        ComparisonStep::Continue(name) => {
                            CandidatePhase::IndexPriorName { index, name, prior }
                        }
                        ComparisonStep::Different => CandidatePhase::IndexProbe,
                        ComparisonStep::Equal => {
                            c.found = true;
                            c.probe = None;
                            c.settled()
                        }
                    }
                }
                CandidatePhase::Reserve => {
                    let new_generation = c
                        .storage
                        .arena
                        .temporary_generation
                        .checked_add(1)
                        .ok_or(HtmlConstructionCause::RepresentationExhausted)?;
                    let minimum = c.index + 1;
                    let target = reserved.unwrap_or(minimum);
                    target
                        .checked_mul(size_of::<NameDecision>())
                        .filter(|v| *v <= isize::MAX as usize)
                        .ok_or(HtmlConstructionCause::RepresentationExhausted)?;
                    let result = crate::internal::amortized::reserve_to(c.decisions, target);
                    let snapshot: &mut HtmlConstructionAllocationSnapshot = c.plan;
                    snapshot.capacity = c.decisions.capacity();
                    snapshot.payload_bytes = snapshot.capacity * size_of::<NameDecision>();
                    if result.is_ok() {
                        snapshot.id.generation = new_generation;
                        c.storage.arena.temporary_generation = new_generation;
                    }
                    allocation = true;
                    result.map_err(|_| HtmlConstructionCause::AllocationFailed)?;
                    CandidatePhase::Witness
                }
                CandidatePhase::Witness => {
                    c.decisions.push(NameDecision {
                        name: c.name.block().downgrade(),
                        keep: !c.found,
                    });
                    c.plan.initialized_len = c.decisions.len();
                    *c.selected += usize::from(!c.found);
                    return Ok(CandidateState::Done);
                }
            }))
        })();
        match result {
            Ok(next) => {
                c.phase = CandidateState::Observing;
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
    /// The attribute this step brings in. Its absence is the refusal that the step's first action
    /// gives for a missing attribute.
    fn incoming(&self) -> Result<&Attribute, HtmlConstructionError> {
        self.attribute()
            .ok_or_else(|| HtmlConstructionCause::CapacityMismatch.into())
    }
}
#[derive(Clone, Copy, Debug)]
enum SlotPhase {
    Input,
    NextName,
    NamedIdentity,
    /// Checking whether the replacement at `index` keeps the replaced attribute's name.
    SameName {
        index: usize,
    },
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
    /// Enter the stored attribute at `position` in the first empty slot of the probe.
    IndexInsert {
        position: usize,
    },
    /// Move the appended name's probe to its next slot and read it.
    IndexProbe,
    /// Comparing the appended name with the stored attribute at `position` that the probe found.
    IndexCompare {
        position: usize,
        name: NamePhase,
    },
    Reserve,
    Layout,
    Unique,
}
/// Where a positional step stands: an action is pending, the observation of a performed action
/// awaits acceptance, or every action was accepted. Only a pending action has a phase, so an action
/// cannot exist without one.
#[derive(Clone, Copy, Debug)]
enum SlotState {
    Pending(SlotPhase),
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
    phase: SlotState,
    failure: Option<HtmlConstructionError>,
    finished: bool,
    usage: HtmlConstructionUsage,
    minimum_capacity: usize,
    plan_index: usize,
    /// The empty slot of the index that the appended name's probe reached, with the name's
    /// tag, which the commit fills.
    insert_slot: Option<(HtmlNameIndexVacancy, u32)>,
    /// The probe of the name being looked up or entered, and its tag.
    probe: Option<HtmlNameIndexProbe>,
    tag: u32,
}
/// One pending slot action. It borrows the slot exclusively; `perform` consumes it.
#[derive(Debug)]
pub struct HtmlConstructionAttributeSlotAction<'a, 'e> {
    slot: &'a mut HtmlConstructionAttributeSlot<'e>,
    /// The phase this action performs.
    phase: SlotPhase,
}
/// The observation after a slot action. It borrows the slot exclusively; `accept` resumes it.
#[derive(Debug)]
pub struct HtmlConstructionAttributeSlotObservation<'a, 'e> {
    slot: &'a mut HtmlConstructionAttributeSlot<'e>,
    /// The state this accepted observation resumes.
    next: SlotState,
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
            .ok_or(HtmlConstructionCause::OperationNotReady)?;
        let len = self.edit.capacity.initialized_len;
        let (storage, revision) = (
            self.edit.storage.arena.serial,
            self.edit.storage.arena.attribute_revision,
        );
        let Some(index) = self.edit.index.as_mut() else {
            return Ok(brute);
        };
        let Some(entries) = len.checked_add(1) else {
            return Ok(brute);
        };
        let described = Binding::Slots {
            storage,
            target,
            revision,
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
        let described = self.described();
        if let Some(index) = self.edit.index.as_mut() {
            let placed = entry(tag, Occupant::Attribute(before))
                .is_some_and(|entry| index.insert(vacancy, entry).is_ok());
            match described {
                Some(binding) if placed => index.bind(binding),
                _ => index.bind(Binding::Unbound),
            }
        }
    }
    /// What the index would describe now: the element's attributes at the storage's current
    /// revision.
    fn described(&self) -> Option<Binding> {
        Some(Binding::Slots {
            storage: self.edit.storage.arena.serial,
            target: self.edit.stage.target()?,
            revision: self.edit.storage.arena.attribute_revision,
            len: self.edit.capacity.initialized_len,
        })
    }
    /// Records that the index describes the attributes the element has now.
    fn bind_described(&mut self) -> Result<(), HtmlConstructionError> {
        let described = self
            .described()
            .ok_or(HtmlConstructionCause::OperationNotReady)?;
        self.edit
            .index
            .as_mut()
            .ok_or_else(table_error)?
            .bind(described);
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
                SlotState::Ready
                    | SlotState::Observing {
                        resumes_ready: true
                    }
            )
        {
            return Err(HtmlConstructionCause::OperationNotReady.into());
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
        match self.phase {
            SlotState::Observing { .. } => Err(HtmlConstructionCause::OperationNotReady.into()),
            SlotState::Ready => Ok(None),
            SlotState::Pending(phase) => Ok(Some(HtmlConstructionAttributeSlotAction {
                slot: self,
                phase,
            })),
        }
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
        if !matches!(self.phase, SlotState::Ready) {
            return Err(HtmlConstructionRefusal::new(
                self,
                HtmlConstructionCause::OperationNotReady,
            ));
        }
        Ok(HtmlConstructionReadyAttributeSlot { slot: self })
    }
}
fn plus(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_add(b)
        .ok_or(HtmlConstructionCause::RepresentationExhausted.error())
}
fn minus(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_sub(b)
        .ok_or(HtmlConstructionCause::ReleaseUnderflow.error())
}
fn identical_name(left: &QualifiedName, right: &QualifiedName) -> bool {
    let namespace = match (left.namespace().uri(), right.namespace().uri()) {
        (None, None) => true,
        (Some(a), Some(b)) => a.shares_storage_with(b),
        _ => false,
    };
    namespace && left.local_name().shares_storage_with(right.local_name())
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
        match self.phase {
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
            SlotPhase::SameName { .. } => {
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
                // A checked append that looks its name up through the index may start rebuilding
                // it, which writes a new index, its binding and its storage length.
                let rebuild = usize::from(
                    s.edit.index.is_some()
                        && matches!(s.input, SlotInput::Append(_))
                        && matches!(s.edit.mode, HtmlConstructionAttributeEditMode::Slots),
                );
                HtmlConstructionAttributeDescription::Inspect(HtmlConstructionAttributeWork {
                    record_visits: 8,
                    record_writes: counters + busy + 3 * rebuild,
                    copied_bytes: counters * size_of::<usize>()
                        + busy * size_of::<bool>()
                        + rebuild
                            * (size_of::<HtmlNameIndex>()
                                + size_of::<Binding>()
                                + size_of::<usize>()),
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
                // Every change to the attributes advances the storage's revision. A held index
                // is bound again or unbound, and an append through it fills the slot its probe
                // reached and counts the entry.
                let changes = !matches!(s.input, SlotInput::Reserve(_));
                let held = changes && s.edit.index.is_some();
                let entered = s.insert_slot.is_some();
                let counter_writes = counter_writes
                    + usize::from(changes)
                    + usize::from(held)
                    + 2 * usize::from(entered);
                let counter_bytes = counter_bytes
                    + usize::from(changes) * size_of::<u64>()
                    + usize::from(held) * size_of::<Binding>()
                    + usize::from(entered) * (size_of::<HtmlNameIndexSlot>() + size_of::<usize>());
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
            SlotPhase::IndexClear => {
                HtmlConstructionAttributeDescription::Inspect(clear_work(s.edit.index.as_ref()))
            }
            SlotPhase::IndexHashLength { .. } => {
                HtmlConstructionAttributeDescription::Inspect(hash_length_work())
            }
            SlotPhase::IndexHash { units, .. } => {
                HtmlConstructionAttributeDescription::Inspect(hash_work(units))
            }
            SlotPhase::IndexInsert { .. } | SlotPhase::IndexProbe => {
                HtmlConstructionAttributeDescription::Inspect(probe_work())
            }
            SlotPhase::IndexCompare { name, .. } => HtmlConstructionAttributeDescription::Compare {
                prior_index: None,
                work: comparison_work(name),
            },
        }
        .transitions::<SlotState>()
    }
    /// Performs the slot action. The observation must be accepted before the next action. The
    /// first failure is latched.
    pub fn perform(
        self,
    ) -> Result<HtmlConstructionAttributeSlotObservation<'a, 'e>, HtmlConstructionError> {
        let s = self.slot;
        let phase = self.phase;
        if !s.edit.busy {
            s.edit.busy = true;
        }
        let mut capacity = false;
        let result = (|| -> Result<_, HtmlConstructionError> {
            Ok(SlotState::Pending(match phase {
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
                        let attribute = v.last().ok_or(HtmlConstructionCause::CapacityMismatch)?;
                        if s.edit.capacity.initialized_len == s.edit.capacity.capacity {
                            return Err(HtmlConstructionCause::CapacityMismatch.into());
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
                            .ok_or(HtmlConstructionCause::CapacityMismatch)?;
                        let new = incoming
                            .as_ref()
                            .ok_or(HtmlConstructionCause::CapacityMismatch)?;
                        s.usage.utf16_units = plus(
                            minus(s.usage.utf16_units, attribute_units(old)?)?,
                            attribute_units(new)?,
                        )?;
                        SlotPhase::SameName { index: *index }
                    }
                    SlotInput::Swap(index) => {
                        if plus(*index, 1)? >= s.edit.capacity.initialized_len {
                            return Err(HtmlConstructionCause::CapacityMismatch.into());
                        }
                        SlotPhase::Layout
                    }
                    SlotInput::Pop => {
                        let old = s
                            .edit
                            .attributes()?
                            .last()
                            .ok_or(HtmlConstructionCause::CapacityMismatch)?;
                        s.usage.attributes = minus(s.usage.attributes, 1)?;
                        s.usage.utf16_units = minus(s.usage.utf16_units, attribute_units(old)?)?;
                        SlotPhase::Layout
                    }
                },
                SlotPhase::NextName => {
                    if s.edit.appended == s.edit.selected {
                        return Err(HtmlConstructionCause::AttributeOrderMismatch.into());
                    }
                    let decision = s
                        .edit
                        .decisions
                        .get(s.plan_index)
                        .ok_or(HtmlConstructionCause::AttributeOrderMismatch)?;
                    if decision.keep {
                        SlotPhase::NamedIdentity
                    } else {
                        s.plan_index += 1;
                        SlotPhase::NextName
                    }
                }
                SlotPhase::NamedIdentity => {
                    let attribute = s.input.incoming()?;
                    let decision = s
                        .edit
                        .decisions
                        .get(s.plan_index)
                        .ok_or(HtmlConstructionCause::AttributeOrderMismatch)?;
                    if attribute.name().namespace().uri().is_some()
                        || decision.name.address()
                            != attribute.name().local_name().block().address()
                    {
                        return Err(HtmlConstructionCause::AttributeNameMismatch.into());
                    }
                    SlotPhase::Layout
                }
                SlotPhase::SameName { index } => {
                    let old = s
                        .edit
                        .attributes()?
                        .get(index)
                        .ok_or(HtmlConstructionCause::CapacityMismatch)?;
                    if identical_name(old.name(), s.input.incoming()?.name()) {
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
                        let old = s
                            .edit
                            .attributes()?
                            .get(index)
                            .ok_or(HtmlConstructionCause::CapacityMismatch)?
                            .name();
                        let new = s.input.incoming()?.name();
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
                                return Err(HtmlConstructionCause::DuplicateAttribute {
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
                    } else {
                        table.bind_storage().map_err(|_| table_error())?;
                        if s.edit.capacity.initialized_len == 0 {
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
                    let name = match subject {
                        Subject::Existing(position) => s
                            .edit
                            .attributes()?
                            .get(position)
                            .ok_or_else(table_error)?
                            .name(),
                        Subject::Incoming => s.input.attribute().ok_or_else(table_error)?.name(),
                    };
                    let hash = s.edit.index.as_ref().ok_or_else(table_error)?.hash(name);
                    let table = s.edit.index.as_mut().ok_or_else(table_error)?;
                    s.probe = Some(table.probe(hash).map_err(|_| table_error())?);
                    s.tag = tag(hash);
                    match subject {
                        Subject::Existing(position) => SlotPhase::IndexInsert { position },
                        Subject::Incoming => SlotPhase::IndexProbe,
                    }
                }
                SlotPhase::IndexInsert { position } => {
                    let table = s.edit.index.as_mut().ok_or_else(table_error)?;
                    let probe = s.probe.as_mut().ok_or_else(table_error)?;
                    match table.advance(probe).map_err(|_| table_error())? {
                        Some(HtmlNameIndexStep::Vacant(vacancy)) => {
                            let placed = entry(s.tag, Occupant::Attribute(position))
                                .is_some_and(|entry| table.insert(vacancy, entry).is_ok());
                            if !placed {
                                return Err(table_error());
                            }
                            s.probe = None;
                            s.after_existing(position)?
                        }
                        Some(HtmlNameIndexStep::Occupied(_)) => SlotPhase::IndexInsert { position },
                        None => return Err(table_error()),
                    }
                }
                SlotPhase::IndexProbe => {
                    let table = s.edit.index.as_mut().ok_or_else(table_error)?;
                    let probe = s.probe.as_mut().ok_or_else(table_error)?;
                    match table.advance(probe).map_err(|_| table_error())? {
                        Some(HtmlNameIndexStep::Vacant(vacancy)) => {
                            s.insert_slot = Some((vacancy, s.tag));
                            s.probe = None;
                            SlotPhase::Layout
                        }
                        Some(HtmlNameIndexStep::Occupied(held)) => match occupant(held) {
                            (_, held_tag) if held_tag != s.tag => SlotPhase::IndexProbe,
                            (Occupant::Attribute(position), _)
                                if position < s.edit.capacity.initialized_len =>
                            {
                                SlotPhase::IndexCompare {
                                    position,
                                    name: NamePhase::Pair,
                                }
                            }
                            _ => return Err(table_error()),
                        },
                        None => return Err(table_error()),
                    }
                }
                SlotPhase::IndexCompare { position, name } => {
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
                        ComparisonStep::Continue(name) => {
                            SlotPhase::IndexCompare { position, name }
                        }
                        ComparisonStep::Different => SlotPhase::IndexProbe,
                        ComparisonStep::Equal => {
                            return Err(HtmlConstructionCause::DuplicateAttribute {
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
                        .ok_or(HtmlConstructionCause::RepresentationExhausted)?;
                    let _ = required_bytes;
                    let additional = s.minimum_capacity - s.edit.capacity.initialized_len;
                    let (result, reached) = s
                        .edit
                        .reserve_backing(additional)?
                        .ok_or(HtmlConstructionCause::SharedAllocationAliased)?;
                    s.edit.capacity.capacity = reached;
                    capacity = true;
                    result.map_err(|_| HtmlConstructionCause::AllocationFailed)?;
                    SlotPhase::Layout
                }
                SlotPhase::Layout => {
                    if matches!(s.input, SlotInput::Reserve(_)) {
                        return Ok(SlotState::Ready);
                    }
                    SlotPhase::Unique
                }
                SlotPhase::Unique => {
                    let backing = s.edit.shared_attributes()?;
                    if Arc::strong_count(backing) != 1 || Arc::weak_count(backing) != 0 {
                        return Err(HtmlConstructionCause::SharedAllocationAliased.into());
                    }
                    return Ok(SlotState::Ready);
                }
            }))
        })();
        match result {
            Ok(next) => {
                s.phase = SlotState::Observing {
                    resumes_ready: matches!(next, SlotState::Ready),
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
    ///
    /// Nothing changes until every check has passed. A backing that another owner holds, an
    /// incoming attribute that is gone, or a position outside the backing refuses the commit with
    /// the ready slot handed back, and its attributes, counts, incoming attribute and the
    /// storage's attribute revision as they were. Dropping the refused slot poisons the edit.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn commit(mut self) -> Result<Option<Attribute>, HtmlConstructionRefusal<Self>> {
        match self.apply() {
            Ok(removed) => {
                let s = &mut self.slot;
                s.edit.storage.arena.usage = s.usage;
                s.edit.busy = false;
                s.finished = true;
                Ok(removed)
            }
            Err(error) => Err(HtmlConstructionRefusal::new(self, error)),
        }
    }
    fn apply(&mut self) -> Result<Option<Attribute>, HtmlConstructionError> {
        let s = &mut self.slot;
        // Everything that can refuse is checked against the attributes as they stand before the
        // backing is taken for a change, which advances the storage's attribute revision.
        let stored = s.edit.attributes()?.len();
        Ok(match &mut s.input {
            SlotInput::Reserve(_) => None,
            SlotInput::Append(incoming) => {
                let before = s.edit.capacity.initialized_len;
                let grown = plus(before, 1)?;
                if incoming.is_empty() {
                    return Err(HtmlConstructionCause::CapacityMismatch.into());
                }
                let backing = s.edit.exclusive_backing()?;
                let attribute = incoming
                    .pop()
                    .ok_or(HtmlConstructionCause::CapacityMismatch)?;
                backing.push(attribute);
                s.edit.capacity.initialized_len = grown;
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
                if *index >= stored || incoming.is_none() {
                    return Err(HtmlConstructionCause::CapacityMismatch.into());
                }
                let backing = s.edit.exclusive_backing()?;
                let target = backing
                    .get_mut(*index)
                    .ok_or(HtmlConstructionCause::CapacityMismatch)?;
                let attribute = incoming
                    .take()
                    .ok_or(HtmlConstructionCause::CapacityMismatch)?;
                let removed = std::mem::replace(target, attribute);
                s.edit.used = true;
                if let Some(table) = s.edit.index.as_mut() {
                    table.bind(Binding::Unbound);
                }
                Some(removed)
            }
            SlotInput::Swap(index) => {
                if plus(*index, 1)? >= stored {
                    return Err(HtmlConstructionCause::CapacityMismatch.into());
                }
                let backing = s.edit.exclusive_backing()?;
                let Some([first, second, ..]) = backing.get_mut(*index..) else {
                    return Err(HtmlConstructionCause::CapacityMismatch.into());
                };
                std::mem::swap(first, second);
                s.edit.used = true;
                if let Some(table) = s.edit.index.as_mut() {
                    table.bind(Binding::Unbound);
                }
                None
            }
            SlotInput::Pop => {
                let remaining = minus(s.edit.capacity.initialized_len, 1)?;
                if stored == 0 {
                    return Err(HtmlConstructionCause::CapacityMismatch.into());
                }
                let backing = s.edit.exclusive_backing()?;
                let value = backing
                    .pop()
                    .ok_or(HtmlConstructionCause::CapacityMismatch)?;
                s.invalidate_index();
                s.edit.capacity.initialized_len = remaining;
                s.edit.used = true;
                Some(value)
            }
        })
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
    step: RetirementStep,
}
/// What a cleanup action releases. The allocation step holds the snapshot it retires.
#[derive(Debug)]
enum RetirementStep {
    /// The last initialized decision cell.
    Cell,
    /// The plan's allocation, once every decision cell is released.
    Allocation(HtmlConstructionAllocationSnapshot),
}
impl HtmlConstructionAttributePlanRetirement {
    /// The next cleanup action, or `None` once the plan is fully released.
    pub fn next_action(&mut self) -> Option<HtmlConstructionAttributeRetirementAction<'_>> {
        let step = if !self.decisions.is_empty() {
            RetirementStep::Cell
        } else {
            RetirementStep::Allocation(self.snapshot.clone()?)
        };
        Some(HtmlConstructionAttributeRetirementAction {
            retirement: self,
            step,
        })
    }
}
impl HtmlConstructionAttributeRetirementAction<'_> {
    /// Describes what the pending action releases.
    pub fn description(&self) -> HtmlConstructionAttributeRetirementDescription {
        match self.step {
            RetirementStep::Allocation(_) => {
                HtmlConstructionAttributeRetirementDescription::Allocation
            }
            RetirementStep::Cell => HtmlConstructionAttributeRetirementDescription::NameWitness {
                auxiliary_entries: 1,
                provider_calls: 1,
            },
        }
    }
    /// The plan's allocation snapshot, if one was made.
    pub fn allocation(&self) -> Option<&HtmlConstructionAllocationSnapshot> {
        match &self.step {
            RetirementStep::Allocation(snapshot) => Some(snapshot),
            RetirementStep::Cell => self.retirement.snapshot.as_ref(),
        }
    }
    /// Performs the cleanup. Returns the retirement evidence when the plan's allocation was
    /// released and `None` for a decision cell.
    pub fn perform(self) -> Option<HtmlConstructionRetired> {
        match self.step {
            RetirementStep::Cell => {
                drop(self.retirement.decisions.pop());
                if let Some(snapshot) = self.retirement.snapshot.as_mut() {
                    snapshot.initialized_len = self.retirement.decisions.len();
                }
                None
            }
            RetirementStep::Allocation(snapshot) => {
                self.retirement.snapshot = None;
                drop(std::mem::take(&mut self.retirement.decisions));
                Some(HtmlConstructionRetired { snapshot })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::construction::bounded::operations::HtmlConstructionCommit;

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
            slots::operation(&mut operation).unwrap().commit().unwrap()
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
    fn make_edit_with(
        storage: HtmlConstructionStorage,
        target: HtmlBuilderNode,
        mode: HtmlConstructionAttributeEditMode,
        index: Option<HtmlAttributeNameIndex>,
    ) -> HtmlConstructionAttributeEdit {
        let mut edit = match index {
            Some(index) => {
                HtmlConstructionAttributeEdit::begin_indexed(storage, target, mode, index)
            }
            None => HtmlConstructionAttributeEdit::begin(storage, target, mode),
        };
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
        slots::attribute_slot(&mut slot).unwrap().commit().unwrap()
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
        fields: Vec<(usize, usize)>,
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
                            attribute.name().local_name().block().address(),
                            attribute.value().block().address(),
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
                    attribute.name().local_name().block().address(),
                    name_backing
                );
                assert_eq!(attribute.value().block().address(), value_backing);
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
            let recovered = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
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
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let original = UnchangedAttributes::capture(&edit);
        let mut incoming = Some(attribute("b", "replacement"));
        let name = incoming
            .as_ref()
            .unwrap()
            .name()
            .local_name()
            .block()
            .address();
        let value = incoming.as_ref().unwrap().value().block().address();
        assert_slot_refused(
            edit.replace(0, &mut incoming).unwrap(),
            HtmlConstructionCause::DuplicateAttribute {
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
        assert_eq!(incoming.name().local_name().block().address(), name);
        assert_eq!(incoming.value().block().address(), value);
        let expected = [("a", "one"), ("b", "two"), ("c", "three")];
        original.assert_unchanged(&edit, &target, &expected);
        original.assert_abort_and_retirement(edit, target, &expected);
    }

    #[test]
    fn invalid_replacement_index_preserves_exact_incoming_and_installed_attributes() {
        for index in [2, usize::MAX] {
            let (storage, target) =
                make_storage(vec![attribute("a", "one"), attribute("b", "two")]);
            let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
            let original = UnchangedAttributes::capture(&edit);
            let mut incoming = Some(attribute("new", "candidate"));
            let name = incoming
                .as_ref()
                .unwrap()
                .name()
                .local_name()
                .block()
                .address();
            let value = incoming.as_ref().unwrap().value().block().address();
            assert_slot_refused(
                edit.replace(index, &mut incoming).unwrap(),
                HtmlConstructionCause::CapacityMismatch.error(),
            );
            let incoming = incoming
                .as_ref()
                .expect("invalid index retains the actual incoming attribute");
            assert_eq!(incoming.name().local_name().to_utf8().unwrap(), "new");
            assert_eq!(incoming.value().to_utf8().unwrap(), "candidate");
            assert_eq!(incoming.name().local_name().block().address(), name);
            assert_eq!(incoming.value().block().address(), value);
            let expected = [("a", "one"), ("b", "two")];
            original.assert_unchanged(&edit, &target, &expected);
            original.assert_abort_and_retirement(edit, target, &expected);
        }
    }

    #[test]
    fn absent_replacement_input_preserves_existing_slot_and_usage() {
        let (storage, target) = make_storage(vec![attribute("a", "one"), attribute("b", "two")]);
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let original = UnchangedAttributes::capture(&edit);
        let mut incoming = None;
        assert_slot_refused(
            edit.replace(0, &mut incoming).unwrap(),
            HtmlConstructionCause::CapacityMismatch.error(),
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
            let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
            let original = UnchangedAttributes::capture(&edit);
            assert_slot_refused(
                edit.swap_adjacent(index).unwrap(),
                HtmlConstructionCause::CapacityMismatch.error(),
            );
            let expected = [("a", "one"), ("b", "two")];
            original.assert_unchanged(&edit, &target, &expected);
            original.assert_abort_and_retirement(edit, target, &expected);
        }
    }

    #[test]
    fn pop_on_empty_attributes_preserves_actual_backing_and_usage() {
        let (storage, target) = make_storage(Vec::new());
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let original = UnchangedAttributes::capture(&edit);
        assert_slot_refused(
            edit.pop().unwrap(),
            HtmlConstructionCause::CapacityMismatch.error(),
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
            Err(ref error) if error.cause == HtmlConstructionCause::OperationNotReady
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
            Err(ref error) if error.cause == HtmlConstructionCause::OperationNotReady
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
            Err(ref error) if error.cause == HtmlConstructionCause::OperationNotReady
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
            HtmlConstructionCause::AttributeNameMismatch.error()
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
            HtmlConstructionCause::DuplicateAttribute {
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

    /// The error of a ready slot's refused commit, and the slot handed back with it.
    fn refused(
        ready: HtmlConstructionReadyAttributeSlot<'_>,
    ) -> (
        HtmlConstructionReadyAttributeSlot<'_>,
        HtmlConstructionError,
    ) {
        match ready.commit() {
            Err(refusal) => refusal.into_parts(),
            Ok(removed) => panic!("the commit must be refused, but it removed {removed:?}"),
        }
    }

    /// The tests below tamper with a slot's private state, which no caller of the public API can
    /// reach, to observe what a step does when a state that its earlier actions established has
    /// been lost. Each must be refused with a typed error, and none may panic.
    #[test]
    fn commit_is_refused_while_another_owner_holds_the_backing_and_succeeds_once_it_is_alone() {
        let (storage, target) = make_storage(vec![attribute("x", "old")]);
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let mut incoming = Some(attribute("y", "new"));
        let mut pending = Some(edit.replace(0, &mut incoming).unwrap());
        while let Some(action) = pending.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        let ready = slots::attribute_slot(&mut pending).unwrap();
        let alias = Arc::clone(ready.slot.edit.shared_attributes().unwrap());
        let revision = ready.slot.edit.storage.arena.attribute_revision;
        let (ready, error) = refused(ready);
        assert!(
            matches!(
                error.cause(),
                HtmlConstructionCause::SharedAllocationAliased
            ),
            "{error}"
        );
        // The refusal changed nothing: the backing, the incoming attribute and the revision.
        assert_eq!(alias[0].value().to_utf8().unwrap(), "old");
        assert_eq!(ready.slot.edit.storage.arena.attribute_revision, revision);
        assert!(
            matches!(&ready.slot.input, SlotInput::Replace { incoming, .. } if incoming.is_some())
        );
        drop(alias);
        let removed = ready.commit().unwrap().unwrap();
        drop(pending);
        assert_eq!(removed.value().to_utf8().unwrap(), "old");
        assert_eq!(
            edit.attributes().unwrap()[0].value().to_utf8().unwrap(),
            "new"
        );
    }

    #[test]
    fn replacement_steps_refuse_a_missing_incoming_attribute() {
        for steps_before_tamper in [1, 3] {
            let (storage, target) =
                make_storage(vec![attribute("x", "old"), attribute("w", "old")]);
            let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
            let mut incoming = Some(attribute("y", "new"));
            let mut slot = edit.replace(0, &mut incoming).unwrap();
            // Input, the same-name check, which finds the names differ, then the comparisons with
            // each stored name: the replaced position is skipped and the next one is compared.
            for _ in 0..steps_before_tamper {
                slot.next_action()
                    .unwrap()
                    .unwrap()
                    .perform()
                    .unwrap()
                    .accept();
            }
            if let SlotInput::Replace { incoming, .. } = &mut slot.input {
                incoming.take();
            }
            let error = slot
                .next_action()
                .unwrap()
                .unwrap()
                .perform()
                .map(|_| ())
                .expect_err("a missing incoming attribute is refused");
            assert!(
                matches!(error.cause(), HtmlConstructionCause::CapacityMismatch),
                "step {steps_before_tamper}: {error}"
            );
        }
    }

    #[test]
    fn a_named_append_refuses_a_missing_incoming_attribute() {
        let (storage, target) = make_storage(Vec::new());
        let names = [DocumentString::from("a")];
        let mut edit = make_edit(
            storage,
            target,
            HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 1 },
        );
        plan(&mut edit, &names);
        slot(edit.reserve(1).unwrap());
        let mut incoming = vec![attribute("a", "v")];
        let mut pending = edit.append(&mut incoming).unwrap();
        // Input, then the search for the next kept name, which leaves the identity check pending.
        for _ in 0..2 {
            pending
                .next_action()
                .unwrap()
                .unwrap()
                .perform()
                .unwrap()
                .accept();
        }
        if let SlotInput::Append(attributes) = &mut pending.input {
            attributes.clear();
        }
        let error = pending
            .next_action()
            .unwrap()
            .unwrap()
            .perform()
            .map(|_| ())
            .expect_err("a missing incoming attribute is refused");
        assert!(
            matches!(error.cause(), HtmlConstructionCause::CapacityMismatch),
            "{error}"
        );
    }

    #[test]
    fn commit_refuses_a_replacement_that_is_gone_and_leaves_the_backing() {
        let (storage, target) = make_storage(vec![attribute("x", "old")]);
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let mut replacement = Some(attribute("y", "new"));
        let mut pending = Some(edit.replace(0, &mut replacement).unwrap());
        while let Some(action) = pending.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        let mut ready = slots::attribute_slot(&mut pending).unwrap();
        if let SlotInput::Replace { incoming, .. } = &mut ready.slot.input {
            incoming.take();
        }
        let (ready, error) = refused(ready);
        assert!(
            matches!(error.cause(), HtmlConstructionCause::CapacityMismatch),
            "{error}"
        );
        assert_eq!(ready.slot.edit.attributes().unwrap().len(), 1);
        assert_eq!(
            ready.slot.edit.attributes().unwrap()[0]
                .value()
                .to_utf8()
                .unwrap(),
            "old"
        );
    }

    #[test]
    fn commit_refuses_an_appended_attribute_that_is_gone_and_leaves_the_counts() {
        let (storage, target) = make_storage(Vec::new());
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        slot(edit.reserve(1).unwrap());
        let mut appended = vec![attribute("z", "new")];
        let mut pending = Some(edit.append(&mut appended).unwrap());
        while let Some(action) = pending.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        let mut ready = slots::attribute_slot(&mut pending).unwrap();
        if let SlotInput::Append(attributes) = &mut ready.slot.input {
            attributes.clear();
        }
        let (ready, error) = refused(ready);
        assert!(
            matches!(error.cause(), HtmlConstructionCause::CapacityMismatch),
            "{error}"
        );
        assert_eq!(ready.slot.edit.capacity.initialized_len, 0);
        assert!(ready.slot.edit.attributes().unwrap().is_empty());
    }

    #[test]
    fn commit_refuses_a_backing_that_lost_the_attributes_it_checked() {
        // Pop and swap meet an emptied backing; replace meets a position beyond it.
        for step in 0..3 {
            let (storage, target) =
                make_storage(vec![attribute("x", "old"), attribute("y", "old")]);
            let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
            let mut replacement = Some(attribute("z", "new"));
            let mut pending = Some(match step {
                0 => edit.pop().unwrap(),
                1 => edit.swap_adjacent(0).unwrap(),
                _ => edit.replace(1, &mut replacement).unwrap(),
            });
            while let Some(action) = pending.as_mut().unwrap().next_action().unwrap() {
                action.perform().unwrap().accept();
            }
            let ready = slots::attribute_slot(&mut pending).unwrap();
            Arc::get_mut(ready.slot.edit.backing_mut().unwrap())
                .unwrap()
                .clear();
            let revision = ready.slot.edit.storage.arena.attribute_revision;
            let (ready, error) = refused(ready);
            assert!(
                matches!(error.cause(), HtmlConstructionCause::CapacityMismatch),
                "step {step}: {error}"
            );
            // The refusal left the recorded length, so the lost attributes stay visible as such,
            // and it left the revision, which only a change to the attributes advances.
            assert_eq!(ready.slot.edit.capacity.initialized_len, 2, "step {step}");
            assert_eq!(
                ready.slot.edit.storage.arena.attribute_revision, revision,
                "step {step}"
            );
        }
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
            if matches!(observation.next, SlotState::Ready) {
                assert!(matches!(
                    observation.shared_attributes(),
                    Err(ref error) if error.cause == HtmlConstructionCause::OperationNotReady
                ));
            }
            observation.accept();
        }
        assert!(matches!(
            pending.as_ref().unwrap().shared_attributes(),
            Err(ref error) if error.cause == HtmlConstructionCause::OperationNotReady
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
                // four admitted phase/successor writes and the storage's attribute revision.
                assert_eq!(work.record_writes, 13);
                let one_short = 2 * size_of::<Attribute>() - 1;
                assert!(work.atomic_layout_bytes > one_short);
                break;
            }
            action.perform().unwrap().accept();
        }
        assert!(matches!(
            slots::attribute_slot(&mut pending),
            Err(ref error) if error.cause == HtmlConstructionCause::OperationNotReady
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
            Err(ref error) if error.cause == HtmlConstructionCause::ForeignBuilder
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
            assert_eq!(error, HtmlConstructionCause::AttributeNameMismatch.error());
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
            Err(ref error) if error.cause == HtmlConstructionCause::AttributeOrderMismatch
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
        assert_eq!(error, HtmlConstructionCause::AttributeNameMismatch.error());
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
            Err(ref error) if error.cause == HtmlConstructionCause::OperationNotReady
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
        assert!(names.iter().all(|name| name.block().holders() == 1));
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
                    work.record_writes, 14,
                    "named commit includes cached length, both plan progress fields and the \
                     storage's attribute revision"
                );
            }
            assert_eq!(
                action.description().work().inspected_bytes,
                0,
                "named append repeats no UTF-16 comparison"
            );
            action.perform().unwrap().accept();
        }
        slots::attribute_slot(&mut pending)
            .unwrap()
            .commit()
            .unwrap();
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
            Err(ref error) if error.cause == HtmlConstructionCause::OperationNotReady
        ));
        assert!(matches!(
            edit.as_ref().unwrap().selected_count(),
            Err(ref error) if error.cause == HtmlConstructionCause::OperationNotReady
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
                target,
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
            HtmlConstructionCause::CapacityMismatch.error()
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
        assert_eq!(refused, HtmlConstructionCause::CapacityMismatch.error());
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
    struct Admitted(u64, usize);
    impl Admitted {
        /// Counts a `Compare` description, which is where a name is read.
        fn note(&mut self, description: &HtmlConstructionAttributeDescription<'_>) {
            if matches!(
                description,
                HtmlConstructionAttributeDescription::Compare { .. }
            ) {
                self.1 += 1;
            }
        }
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
    fn append_at(
        storage: HtmlConstructionStorage,
        ordinal: usize,
        index: HtmlAttributeNameIndex,
        incoming: Attribute,
        admitted: &mut Admitted,
    ) -> (
        HtmlConstructionStorage,
        HtmlAttributeNameIndex,
        Result<(), HtmlConstructionError>,
    ) {
        let target = storage.node_at(ordinal).unwrap().handle();
        let mut edit = HtmlConstructionAttributeEdit::begin_indexed(
            storage,
            target,
            HtmlConstructionAttributeEditMode::Slots,
            index,
        );
        while let Some(action) = edit.next_action().unwrap() {
            let description = action.description();
            admitted.note(&description);
            admitted.add(description.work());
            action.perform().unwrap().accept();
        }
        let mut incoming = vec![incoming];
        let result = (|| {
            let mut reserve = Some(edit.reserve(1)?);
            while let Some(action) = reserve.as_mut().unwrap().next_action()? {
                let description = action.description();
                admitted.note(&description);
                admitted.add(description.work());
                action.perform()?.accept();
            }
            slots::attribute_slot(&mut reserve)?.commit().unwrap();
            drop(reserve);
            let mut append = Some(edit.append(&mut incoming)?);
            while let Some(action) = append.as_mut().unwrap().next_action()? {
                let description = action.description();
                admitted.note(&description);
                admitted.add(description.work());
                action.perform()?.accept();
            }
            slots::attribute_slot(&mut append)?.commit().unwrap();
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
        append_at(storage, 0, index, incoming, admitted)
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
                    let mut edit = make_edit_with(
                        storage,
                        target,
                        HtmlConstructionAttributeEditMode::Slots,
                        Some(index),
                    );
                    let mut aborted = false;
                    match random.next(3) {
                        0 => {
                            let at = random.next(expected.len() - 1);
                            slot(edit.swap_adjacent(at).unwrap());
                            expected.swap(at, at + 1);
                        }
                        1 => {
                            slot(edit.pop().unwrap());
                            expected.pop();
                        }
                        _ => {
                            // A replacement whose name may repeat another attribute's.
                            let at = random.next(expected.len());
                            let namespace = NAMESPACES[random.next(NAMESPACES.len())];
                            let local = LOCALS[random.next(LOCALS.len())];
                            let key = (namespace.to_string(), local.to_string());
                            let mut incoming = Some(qualified(namespace, None, local));
                            let outcome = (|| -> Result<(), HtmlConstructionError> {
                                let mut replacing = Some(edit.replace(at, &mut incoming)?);
                                while let Some(action) =
                                    replacing.as_mut().unwrap().next_action()?
                                {
                                    action.perform()?.accept();
                                }
                                slots::attribute_slot(&mut replacing)?.commit().unwrap();
                                Ok(())
                            })();
                            let other = expected
                                .iter()
                                .enumerate()
                                .position(|(position, held)| position != at && *held == key);
                            match (other, outcome) {
                                (None, Ok(())) => expected[at] = key,
                                (Some(first), Err(error)) => {
                                    assert_eq!(
                                        error,
                                        HtmlConstructionCause::DuplicateAttribute {
                                            first,
                                            duplicate: at
                                        }
                                        .error(),
                                        "round {round} step {step}"
                                    );
                                    aborted = true;
                                }
                                (other, outcome) => {
                                    panic!("round {round} step {step}: {other:?} {outcome:?}")
                                }
                            }
                        }
                    }
                    let mut completion = if aborted {
                        edit.abort()
                    } else {
                        let mut owner = Some(edit);
                        slots::edit(&mut owner).unwrap()
                    };
                    index = completion.take_index().unwrap();
                    let (next, retirement) = completion.into_parts();
                    retire(retirement);
                    storage = next;
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
                        HtmlConstructionCause::DuplicateAttribute {
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
            Err(HtmlConstructionCause::DuplicateAttribute {
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
                let mut edit = make_edit_with(
                    storage,
                    target,
                    HtmlConstructionAttributeEditMode::NamedMerge {
                        candidate_count: names.len(),
                    },
                    index,
                );
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
            let mut edit = HtmlConstructionAttributeEdit::begin_indexed(
                storage,
                target,
                HtmlConstructionAttributeEditMode::NamedMerge {
                    candidate_count: count,
                },
                new_index(count + 1),
            );
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

    #[test]
    fn an_indexed_append_notices_a_replacement_made_by_an_edit_that_did_not_hold_the_index() {
        let (mut storage, _) = make_storage(Vec::new());
        let mut index = new_index(16);
        let mut admitted = Admitted::default();
        for name in ["a", "b", "c"] {
            let (next, back, result) =
                append_in_new_edit(storage, index, attribute(name, "v"), &mut admitted);
            result.unwrap();
            storage = next;
            index = back;
        }
        // An edit that never held the index replaces b with z in place.
        let target = storage.node_at(0).unwrap().handle();
        let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
        let mut incoming = Some(attribute("z", "v"));
        slot(edit.replace(1, &mut incoming).unwrap());
        storage = finish(edit);
        // The index still names b where z now is, so z must be found by rebuilding it.
        let (_, _, result) = append_in_new_edit(storage, index, attribute("z", "v"), &mut admitted);
        assert_eq!(
            result,
            Err(HtmlConstructionCause::DuplicateAttribute {
                first: 1,
                duplicate: 3
            }
            .error())
        );
    }

    #[test]
    fn an_index_lent_when_the_edit_begins_decides_repeated_candidates_like_the_pairwise_plan() {
        // An index can only be lent at the start of an edit, so that it has seen every name the
        // plan has: lending one after a candidate was planned is not expressible.
        let names = ["x", "x", "y", "y"].map(DocumentString::from);
        let decide = |index: Option<HtmlAttributeNameIndex>| {
            let (storage, target) = make_storage(Vec::new());
            let mut edit = make_edit_with(
                storage,
                target,
                HtmlConstructionAttributeEditMode::NamedMerge {
                    candidate_count: names.len(),
                },
                index,
            );
            plan(&mut edit, &names);
            let decisions: Vec<bool> = (0..names.len())
                .map(|i| edit.decision(i).unwrap())
                .collect();
            retire(edit.abort().into_parts().1);
            decisions
        };
        assert_eq!(decide(None), [true, false, true, false]);
        assert_eq!(
            decide(Some(new_index(names.len()))),
            [true, false, true, false]
        );
    }

    #[test]
    fn a_probe_reads_a_name_only_where_the_tag_matches_and_moves_on_when_the_names_differ() {
        let (storage, _) = make_storage(Vec::new());
        let mut admitted = Admitted::default();
        let (storage, mut index, result) =
            append_in_new_edit(storage, new_index(16), attribute("a", "v"), &mut admitted);
        result.unwrap();
        // Without a collision no name is read: the empty slot or a tag that differs ends the
        // probe, so appending b compares nothing.
        let mut plain = Admitted::default();
        let (control, back, result) =
            append_in_new_edit(storage, index, attribute("b", "v"), &mut plain);
        result.unwrap();
        index = back;
        // The slot of a hand-built entry that carries the tag of c but names a, placed on the
        // path of c's probe, which the append of c must meet, read and pass.
        let hash = index.hash_local(&[u16::from(b'c')]);
        let mut probe = index.probe(hash).unwrap();
        loop {
            match index.advance(&mut probe).unwrap() {
                Some(HtmlNameIndexStep::Vacant(vacancy)) => {
                    let collide = entry(tag(hash), Occupant::Attribute(0)).unwrap();
                    index.insert(vacancy, collide).unwrap();
                    break;
                }
                Some(HtmlNameIndexStep::Occupied(_)) => {}
                None => panic!("a half full table has a vacancy"),
            }
        }
        let mut collided = Admitted::default();
        let (control, _, result) =
            append_in_new_edit(control, index, attribute("c", "v"), &mut collided);
        assert_eq!(
            result,
            Ok(()),
            "the colliding name differs, so c is appended"
        );
        assert_eq!(plain.1, 0, "no name is read without a collision");
        assert!(collided.1 > 0, "the colliding entry's name was read");
        let names: Vec<String> = match control.node_at(0).unwrap().kind() {
            HtmlNodeKind::Element(element) => element
                .attributes()
                .as_slice()
                .iter()
                .map(|held| held.name().local_name().to_utf8().unwrap())
                .collect(),
            _ => unreachable!(),
        };
        assert_eq!(names, ["a", "b", "c"]);
    }

    /// Adds an element with `attributes` to `storage`.
    fn add_element(
        storage: HtmlConstructionStorage,
        attributes: Vec<Attribute>,
    ) -> HtmlConstructionStorage {
        let mut operation = Some(storage.prepare(HtmlConstructionCommand::Create {
            kind: HtmlNodeKind::Element(HtmlElement::new(
                QualifiedName::new(Namespace::none(), None, "other"),
                OrderedAttributes::new(attributes).unwrap(),
            )),
        }));
        while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        let HtmlConstructionCommit::Changed { storage, .. } =
            slots::operation(&mut operation).unwrap().commit().unwrap()
        else {
            panic!("create")
        };
        storage
    }

    #[test]
    fn an_index_that_served_another_element_of_the_storage_is_rebuilt_for_this_one() {
        let (storage, _) = make_storage(Vec::new());
        let storage = add_element(storage, Vec::new());
        let mut admitted = Admitted::default();
        let (storage, index, result) = append_at(
            storage,
            0,
            new_index(16),
            attribute("x", "v"),
            &mut admitted,
        );
        result.unwrap();
        // The other element has no x yet: the same name is accepted there.
        let (storage, index, result) =
            append_at(storage, 1, index, attribute("x", "v"), &mut admitted);
        assert_eq!(result, Ok(()));
        // Back on the first element, the index describes the other one and must not be trusted.
        let (_, _, result) = append_at(storage, 0, index, attribute("x", "v"), &mut admitted);
        assert_eq!(
            result,
            Err(HtmlConstructionCause::DuplicateAttribute {
                first: 0,
                duplicate: 1
            }
            .error())
        );
    }

    #[test]
    fn an_index_kept_beside_another_storage_is_not_trusted_for_this_one() {
        let mut admitted = Admitted::default();
        let (first, _) = make_storage(Vec::new());
        let (second, _) = make_storage(Vec::new());
        let (first, index, result) =
            append_in_new_edit(first, new_index(16), attribute("x", "v"), &mut admitted);
        result.unwrap();
        // The index describes a one attribute element of the first storage; the second has the
        // same target ordinal, length and revision, and must not be taken for it.
        let (second, index, result) =
            append_in_new_edit(second, index, attribute("y", "v"), &mut admitted);
        assert_eq!(result, Ok(()));
        let (_, _, result) = append_in_new_edit(first, index, attribute("x", "v"), &mut admitted);
        assert!(result.is_err(), "x repeats the first storage's x");
        drop(second);
    }

    #[test]
    fn an_indexed_append_notices_attributes_replaced_by_an_operation() {
        let (mut storage, _) = make_storage(Vec::new());
        let mut index = new_index(16);
        let mut admitted = Admitted::default();
        for name in ["a", "b", "c"] {
            let (next, back, result) =
                append_in_new_edit(storage, index, attribute(name, "v"), &mut admitted);
            result.unwrap();
            storage = next;
            index = back;
        }
        // The operation replaces the whole list with another of the same length.
        let element = storage.node_at(0).unwrap().handle();
        let replacement =
            OrderedAttributes::new(["x", "y", "z"].map(|name| attribute(name, "w"))).unwrap();
        let mut operation = Some(storage.prepare(HtmlConstructionCommand::ReplaceAttributes {
            element,
            attributes: replacement.into(),
        }));
        while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        let HtmlConstructionCommit::Changed { storage, .. } =
            slots::operation(&mut operation).unwrap().commit().unwrap()
        else {
            panic!("replace")
        };
        // The index still names a, b and c, so x is found only by rebuilding it.
        let (_, _, result) = append_in_new_edit(storage, index, attribute("x", "v"), &mut admitted);
        assert_eq!(
            result,
            Err(HtmlConstructionCause::DuplicateAttribute {
                first: 0,
                duplicate: 3
            }
            .error())
        );
    }

    /// Every element's attributes as names and values, for comparing before and after a change.
    fn attribute_lists(storage: &HtmlConstructionStorage) -> Vec<Vec<(String, String)>> {
        (0..)
            .map_while(|position| storage.node_at(position))
            .filter_map(|node| match node.kind() {
                HtmlNodeKind::Element(element) => Some(
                    element
                        .attributes()
                        .as_slice()
                        .iter()
                        .map(|held| {
                            (
                                held.name().local_name().to_utf8().unwrap(),
                                held.value().to_utf8().unwrap(),
                            )
                        })
                        .collect(),
                ),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn every_change_to_an_attribute_list_advances_the_attribute_revision_and_nothing_else_does() {
        let (storage, _) = make_storage(vec![attribute("a", "1")]);
        let mut storage = add_element(storage, vec![attribute("p", "1")]);
        let mut admitted = Admitted::default();
        let mut index = new_index(16);
        // Each step changes the attribute lists, or only reserves capacity, as its label says.
        type Step = Box<dyn Fn(HtmlConstructionStorage) -> HtmlConstructionStorage>;
        let edit = |change: Box<dyn Fn(&mut HtmlConstructionAttributeEdit)>| -> Step {
            Box::new(move |storage| {
                let target = storage.node_at(0).unwrap().handle();
                let mut edit = make_edit(storage, target, HtmlConstructionAttributeEditMode::Slots);
                change(&mut edit);
                finish(edit)
            })
        };
        let steps: Vec<(&str, bool, Step)> = vec![
            (
                "reserve only",
                false,
                edit(Box::new(|edit| {
                    slot(edit.reserve(8).unwrap());
                })),
            ),
            (
                "append",
                true,
                edit(Box::new(|edit| {
                    slot(edit.reserve(1).unwrap());
                    let mut incoming = vec![attribute("b", "2")];
                    slot(edit.append(&mut incoming).unwrap());
                })),
            ),
            (
                "swap",
                true,
                edit(Box::new(|edit| {
                    slot(edit.swap_adjacent(0).unwrap());
                })),
            ),
            (
                "replace",
                true,
                edit(Box::new(|edit| {
                    let mut incoming = Some(attribute("c", "3"));
                    slot(edit.replace(1, &mut incoming).unwrap());
                })),
            ),
            (
                "pop",
                true,
                edit(Box::new(|edit| {
                    slot(edit.pop().unwrap());
                })),
            ),
            (
                "replace attributes",
                true,
                Box::new(|storage| {
                    let element = storage.node_at(0).unwrap().handle();
                    let replacement =
                        OrderedAttributes::new(["x", "y"].map(|name| attribute(name, "w")))
                            .unwrap();
                    let mut operation =
                        Some(storage.prepare(HtmlConstructionCommand::ReplaceAttributes {
                            element,
                            attributes: replacement.into(),
                        }));
                    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
                        action.perform().unwrap().accept();
                    }
                    let HtmlConstructionCommit::Changed { storage, .. } =
                        slots::operation(&mut operation).unwrap().commit().unwrap()
                    else {
                        panic!("replace")
                    };
                    storage
                }),
            ),
            (
                "named merge append",
                true,
                Box::new(|storage| {
                    let target = storage.node_at(0).unwrap().handle();
                    let names = [DocumentString::from("m")];
                    let mut edit = make_edit(
                        storage,
                        target,
                        HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 1 },
                    );
                    plan(&mut edit, &names);
                    slot(edit.reserve(1).unwrap());
                    let mut incoming = vec![attribute(names[0].clone(), "4")];
                    slot(edit.append(&mut incoming).unwrap());
                    finish(edit)
                }),
            ),
        ];
        for (label, changes, step) in steps {
            let (lists, revision) = (attribute_lists(&storage), storage.arena.attribute_revision);
            storage = step(storage);
            let changed = attribute_lists(&storage) != lists;
            assert_eq!(changed, changes, "{label}: the lists change as expected");
            assert_eq!(
                storage.arena.attribute_revision != revision,
                changed,
                "{label}: the revision advances exactly when an attribute list changes"
            );
        }
        // An indexed append advances it too, and the index keeps up with it.
        let revision = storage.arena.attribute_revision;
        let (next, back, result) = append_in_new_edit(
            storage,
            std::mem::replace(&mut index, new_index(4)),
            attribute("n", "5"),
            &mut admitted,
        );
        result.unwrap();
        assert_ne!(next.arena.attribute_revision, revision);
        drop(back);
    }
}
