//! Bounded append to the actual construction owner's immutable text backing.

use super::*;
use crate::internal::accounting::{HtmlChargedError, HtmlResource, HtmlWork, wide};
use surgeist_ledger::{Change, Charge};

const CHUNK_UNITS: usize = 256 / size_of::<u16>();

/// Observed dimensions of the actual candidate allocation; no release authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlConstructionTextAppendCapacity {
    initialized_len: usize,
    capacity: usize,
    data_address: usize,
}
impl HtmlConstructionTextAppendCapacity {
    /// The number of initialized UTF-16 units in the candidate text backing.
    pub fn initialized_len(self) -> usize {
        self.initialized_len
    }
    /// The reserved unit capacity of the candidate text backing; at least `initialized_len`.
    pub fn capacity(self) -> usize {
        self.capacity
    }
    /// The address of the first backing unit. It identifies a reallocation, not ownership, and
    /// grants no access.
    pub fn data_address(self) -> usize {
        self.data_address
    }
    /// The size in bytes of one text unit: 2, a UTF-16 code unit.
    pub fn element_size(self) -> usize {
        size_of::<u16>()
    }
    /// The alignment in bytes of one text unit.
    pub fn element_alignment(self) -> usize {
        align_of::<u16>()
    }
    /// The reserved payload in bytes: capacity times the unit size.
    pub fn payload_bytes(self) -> usize {
        self.capacity * size_of::<u16>()
    }
    /// Known Vec payload inside Arc, excluding opaque Arc headers.
    pub fn wrapper_payload_bytes(self) -> usize {
        size_of::<Vec<u16>>()
    }
    fn of(value: &DocumentString) -> Self {
        Self {
            initialized_len: value.len(),
            capacity: value.shared_code_units().capacity(),
            data_address: value.code_units().as_ptr() as usize,
        }
    }
}

/// Bounds admitted before each action, excluding allocator internals.
///
/// The amounts are not readable. A consumer charges them through
/// [`HtmlConstructionTextAppendDescription::charge`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlConstructionTextAppendWork {
    /// The retained records the action reads or updates.
    pub(crate) record_visits: usize,
    /// The provider requests the action makes.
    pub(crate) provider_calls: usize,
    /// The bytes the action only reads, without copying them. The text-append steps copy rather
    /// than inspect, so every description reports zero.
    pub(crate) inspected_bytes: usize,
    /// The bytes the action copies, including bookkeeping values it moves.
    pub(crate) copied_bytes: usize,
    /// The record fields the action writes.
    pub(crate) record_writes: usize,
    /// The size in bytes of the largest single layout the action initializes or moves as one unit.
    /// A text copy is split so that no unit exceeds this chunk size.
    pub(crate) atomic_layout_bytes: usize,
    /// Conservative allocator relocation bound, separate from explicit copies.
    pub(crate) relocation_bound_bytes: usize,
}
impl HtmlConstructionTextAppendWork {
    /// Charges the step's work: its atomic layout as the largest reached, then the record visits,
    /// provider calls, inspected bytes, copied bytes and record writes, each as its own change
    /// in that order. The relocation bound is not charged: a consumer prices its own reservation
    /// plan. `prepaid_provider_calls` are calls of the step that the caller has already charged.
    pub(crate) fn charge<C: Charge<HtmlWork, HtmlResource>>(
        self,
        prepaid_provider_calls: u64,
        charge: &mut C,
    ) -> Result<(), HtmlChargedError<C::Stop, HtmlConstructionChargeFault>> {
        charge
            .record_largest(
                HtmlResource::AtomicLayoutBytes,
                wide(self.atomic_layout_bytes),
            )
            .map_err(HtmlChargedError::Stopped)?;
        let provider_calls = wide(self.provider_calls)
            .checked_sub(prepaid_provider_calls)
            .ok_or(HtmlChargedError::Faulted(
                HtmlConstructionChargeFault::PrepaidCallsExceedPlan,
            ))?;
        for change in [
            Change::Work(HtmlWork::RecordVisit, wide(self.record_visits)),
            Change::Work(HtmlWork::ProviderCall, provider_calls),
            Change::Work(HtmlWork::InspectByte, wide(self.inspected_bytes)),
            Change::Work(HtmlWork::CopyByte, wide(self.copied_bytes)),
            Change::Work(HtmlWork::RecordWrite, wide(self.record_writes)),
        ] {
            charge
                .charge(&[change])
                .map_err(HtmlChargedError::Stopped)?;
        }
        Ok(())
    }
}

/// The fixed next step of a text append, with the work the consumer admits before running it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlConstructionTextAppendDescription {
    /// Qualifies the target text node and computes the required length and the usage after the
    /// append.
    Inspect(HtmlConstructionTextAppendWork),
    /// Decides whether the text's backing is uniquely owned, which allows reuse, or aliased, which
    /// requires a separate copy.
    SelectBacking(HtmlConstructionTextAppendWork),
    /// Creates the empty separate backing that receives a preserved copy of the original text.
    InitializeBacking(HtmlConstructionTextAppendWork),
    /// Reserves capacity in the candidate backing.
    Reserve {
        /// The candidate backing's dimensions before the reservation.
        before: HtmlConstructionTextAppendCapacity,
        /// The least capacity that holds the original text and the appended text.
        required_capacity: usize,
        /// The capacity `perform` reserves when called without a minimum: twice the current
        /// capacity, raised to the required capacity and clipped to the largest representable
        /// payload.
        preferred_capacity: usize,
        /// The work the reservation costs, including the relocation bound.
        work: HtmlConstructionTextAppendWork,
    },
    /// Copies a chunk of the original text into the separate backing.
    CopyPrefix {
        /// The offset, in UTF-16 units, of the first unit to copy.
        offset: usize,
        /// The number of units this action copies; at most one chunk of 128 units.
        units: usize,
        /// The work the copy costs.
        work: HtmlConstructionTextAppendWork,
    },
    /// The only action lending the original construction storage.
    Preflight {
        /// The usage after the append. The consumer admits limits against it.
        usage: HtmlConstructionUsage,
        /// The work the action costs.
        work: HtmlConstructionTextAppendWork,
    },
    /// Confirms that the candidate backing is uniquely owned before the tail is written, so
    /// nothing outside the append can observe the written units.
    QualifyTail(HtmlConstructionTextAppendWork),
    /// Copies a chunk of the incoming text onto the end of the candidate backing.
    CopyTail {
        /// The offset, in UTF-16 units, of the first incoming unit to copy.
        offset: usize,
        /// The number of units this action copies; at most one chunk of 128 units.
        units: usize,
        /// The work the copy costs.
        work: HtmlConstructionTextAppendWork,
    },
    /// Describes the prepaid writes: swapping in a preserved copy if one was made and storing the
    /// new usage. Performing this action applies nothing; `HtmlConstructionTextAppendReady::commit`
    /// applies them.
    Commit {
        /// The usage that the commit stores.
        usage: HtmlConstructionUsage,
        /// The work the commit costs.
        work: HtmlConstructionTextAppendWork,
    },
}
impl HtmlConstructionTextAppendDescription {
    /// Charges the work of this step, whichever step it is, as
    /// [`HtmlConstructionTextAppendWork`] describes. `prepaid_provider_calls` are provider calls of
    /// the step that the caller has already charged.
    pub fn charge<C: Charge<HtmlWork, HtmlResource>>(
        self,
        prepaid_provider_calls: u64,
        charge: &mut C,
    ) -> Result<(), HtmlChargedError<C::Stop, HtmlConstructionChargeFault>> {
        self.work().charge(prepaid_provider_calls, charge)
    }
    /// The work of this step, whichever step it is.
    pub(crate) fn work(self) -> HtmlConstructionTextAppendWork {
        match self {
            Self::Inspect(w)
            | Self::SelectBacking(w)
            | Self::InitializeBacking(w)
            | Self::QualifyTail(w)
            | Self::Reserve { work: w, .. }
            | Self::CopyPrefix { work: w, .. }
            | Self::Preflight { work: w, .. }
            | Self::CopyTail { work: w, .. }
            | Self::Commit { work: w, .. } => w,
        }
    }
}

/// What `Inspect` established about the target text node, which every later step reads.
#[derive(Clone, Copy, Debug)]
struct Target {
    /// The text record's ordinal.
    index: usize,
    /// The original text's length in UTF-16 units.
    old_len: usize,
    /// The least backing capacity that holds the original and the incoming text.
    required: usize,
    /// The usage after the append.
    usage: HtmlConstructionUsage,
}
/// The steps that follow the backing choice and every pending step of both. A step carries what
/// only it needs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step {
    /// Reserves capacity; `before` are the candidate's dimensions at that moment.
    Reserve {
        before: HtmlConstructionTextAppendCapacity,
    },
    Preflight,
    QualifyTail,
    /// Copies the incoming text onto the backing, from `tail` units on.
    CopyTail {
        tail: usize,
    },
    Commit,
}
impl Step {
    /// Whether the candidate backing has been qualified as uniquely owned for the tail writes,
    /// which only the steps after that qualification can follow.
    fn tail_is_exclusive(self) -> bool {
        matches!(self, Self::CopyTail { .. } | Self::Commit)
    }
}
/// Where the append is, together with exactly the data established so far.
#[derive(Debug)]
enum Progress {
    /// Nothing is qualified yet.
    Inspect,
    /// The target is qualified; the backing choice is pending.
    Select(Target),
    /// The original is aliased, so the append works in a separate copy that is not created yet.
    Initialize(Target),
    /// The original backing is uniquely owned, so the append reuses it.
    InPlace { target: Target, step: Step },
    /// The separate copy is being filled with the original text, from `prefix` units on. Only a
    /// copy has a prefix to copy.
    Prefix {
        target: Target,
        copy: DocumentString,
        prefix: usize,
    },
    /// The append works in a separate copy of the original text.
    Preserved {
        target: Target,
        copy: DocumentString,
        step: Step,
    },
    /// Every step of an in-place append was accepted.
    ReadyInPlace(Target),
    /// Every step of an append in a separate copy was accepted.
    ReadyPreserved {
        target: Target,
        copy: DocumentString,
    },
}
/// A step tag for tests that drive the append to a named point.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Inspect,
    SelectBacking,
    InitializeBacking,
    Reserve,
    CopyPrefix,
    Preflight,
    QualifyTail,
    CopyTail,
    Commit,
    Ready,
}

/// Exclusive pending owner. No mutable backing or reusable uniqueness proof escapes.
#[derive(Debug)]
pub struct HtmlConstructionTextAppend {
    storage: HtmlConstructionStorage,
    node: HtmlBuilderNode,
    incoming: DocumentString,
    progress: Progress,
    /// An observation awaits `accept`. The progress it reaches is already stored, so a dropped
    /// observation keeps what the action established.
    observing: bool,
    failure: Option<HtmlConstructionError>,
}
/// One pending text-append action. It borrows the append exclusively; `perform` consumes it.
#[derive(Debug)]
pub struct HtmlConstructionTextAppendAction<'a> {
    owner: &'a mut HtmlConstructionTextAppend,
    pending: Pending,
}
/// The observation after a text-append action. It borrows the append exclusively; `accept` resumes
/// it.
#[derive(Debug)]
pub struct HtmlConstructionTextAppendObservation<'a> {
    owner: &'a mut HtmlConstructionTextAppend,
}
/// What a ready append writes when it commits, and what it returns when it aborts.
#[derive(Debug)]
enum ReadyBacking {
    /// The original backing took the tail; an abort truncates it back to `old_len`.
    InPlace { old_len: usize },
    /// The separate copy that replaces the original on commit.
    Preserved(DocumentString),
}
/// A text append whose every action was accepted. `commit` applies it; `abort` abandons it.
#[derive(Debug)]
pub struct HtmlConstructionTextAppendReady {
    storage: HtmlConstructionStorage,
    node: HtmlBuilderNode,
    incoming: DocumentString,
    index: usize,
    usage: HtmlConstructionUsage,
    backing: ReadyBacking,
}

/// Original operands and actual storage, plus a distinct backing needing retirement.
/// On abort, a pending preserved copy retains all of its initialized units.
#[derive(Debug)]
pub struct HtmlConstructionTextAppendCompletion {
    storage: HtmlConstructionStorage,
    node: HtmlBuilderNode,
    incoming: DocumentString,
    discarded: Option<DocumentString>,
}
impl HtmlConstructionTextAppendCompletion {
    /// Returns the storage, the target node handle, the incoming text and the backing that the
    /// caller must retire. The backing is `Some` when a separate preserved copy existed that is no
    /// longer native-visible.
    pub fn into_parts(
        self,
    ) -> (
        HtmlConstructionStorage,
        HtmlBuilderNode,
        DocumentString,
        Option<DocumentString>,
    ) {
        (self.storage, self.node, self.incoming, self.discarded)
    }
}
/// The text record's value, if the record is a text node.
fn text_of(storage: &HtmlConstructionStorage, index: usize) -> Option<&DocumentString> {
    match &storage.arena.records.get(index)?.kind {
        HtmlNodeKind::Text(value) => Some(value),
        _ => None,
    }
}
fn text_of_mut(storage: &mut HtmlConstructionStorage, index: usize) -> Option<&mut DocumentString> {
    match &mut storage.arena.records.get_mut(index)?.kind {
        HtmlNodeKind::Text(value) => Some(value),
        _ => None,
    }
}
/// Removes initialized tail from a reused native backing. The tail qualification proved the
/// backing exclusively held, so this truncates in place. Were another holder to share it, a copy
/// would be an allocation nothing metered, so the truncation is refused with the
/// shared-allocation error and nothing changes. A record that is no longer text has no tail to
/// remove.
fn truncate_tail(
    storage: &mut HtmlConstructionStorage,
    index: usize,
    old_len: usize,
) -> Result<(), HtmlConstructionError> {
    let Some(value) = text_of_mut(storage, index) else {
        return Ok(());
    };
    let backing = Arc::get_mut(value.construction_backing_mut())
        .ok_or(HtmlConstructionErrorKind::SharedAllocationAliased)?;
    backing.truncate(old_len);
    Ok(())
}

impl HtmlConstructionTextAppend {
    /// Allocation-free. Target identity and kind are qualified by the first action.
    pub fn begin(
        storage: HtmlConstructionStorage,
        node: HtmlBuilderNode,
        incoming: DocumentString,
    ) -> Self {
        Self {
            storage,
            node,
            incoming,
            progress: Progress::Inspect,
            observing: false,
            failure: None,
        }
    }
    #[cfg(test)]
    fn phase(&self) -> Phase {
        let step = |step: Step| match step {
            Step::Reserve { .. } => Phase::Reserve,
            Step::Preflight => Phase::Preflight,
            Step::QualifyTail => Phase::QualifyTail,
            Step::CopyTail { .. } => Phase::CopyTail,
            Step::Commit => Phase::Commit,
        };
        match &self.progress {
            Progress::Inspect => Phase::Inspect,
            Progress::Select(_) => Phase::SelectBacking,
            Progress::Initialize(_) => Phase::InitializeBacking,
            Progress::Prefix { .. } => Phase::CopyPrefix,
            Progress::InPlace { step: s, .. } | Progress::Preserved { step: s, .. } => step(*s),
            Progress::ReadyInPlace(_) | Progress::ReadyPreserved { .. } => Phase::Ready,
        }
    }
    fn candidate(&self) -> Option<&DocumentString> {
        match &self.progress {
            Progress::Inspect | Progress::Initialize(_) => None,
            Progress::Select(target)
            | Progress::InPlace { target, .. }
            | Progress::ReadyInPlace(target) => text_of(&self.storage, target.index),
            Progress::Prefix { copy, .. }
            | Progress::Preserved { copy, .. }
            | Progress::ReadyPreserved { copy, .. } => Some(copy),
        }
    }
    fn tail_is_exclusive(&self) -> bool {
        match &self.progress {
            Progress::InPlace { step, .. } | Progress::Preserved { step, .. } => {
                step.tail_is_exclusive()
            }
            Progress::ReadyInPlace(_) | Progress::ReadyPreserved { .. } => true,
            Progress::Inspect
            | Progress::Select(_)
            | Progress::Initialize(_)
            | Progress::Prefix { .. } => false,
        }
    }
    /// The dimensions of the candidate backing. `None` before the first action has qualified the
    /// target, and, when the append works in a preserved copy, until the backing initialization
    /// has created that copy.
    pub fn capacity(&self) -> Option<HtmlConstructionTextAppendCapacity> {
        self.candidate().map(HtmlConstructionTextAppendCapacity::of)
    }
    /// True when immutable original aliases require a separate aggregate.
    pub fn preserves_original(&self) -> bool {
        match self.progress {
            Progress::Initialize(_)
            | Progress::Prefix { .. }
            | Progress::Preserved { .. }
            | Progress::ReadyPreserved { .. } => true,
            Progress::Inspect
            | Progress::Select(_)
            | Progress::InPlace { .. }
            | Progress::ReadyInPlace(_) => false,
        }
    }
    /// Only before final tail qualification. Reads and reference providers need admission.
    pub fn backing(&self) -> Option<&DocumentString> {
        if self.tail_is_exclusive() {
            None
        } else {
            self.candidate()
        }
    }
    /// The pending action's kind, or `None` once every step was accepted.
    fn pending(&self) -> Option<Pending> {
        match &self.progress {
            Progress::Inspect => Some(Pending::Inspect),
            Progress::Select(_) => Some(Pending::Select),
            Progress::Initialize(_) => Some(Pending::Initialize),
            Progress::InPlace { target, step } => Some(Pending::Step {
                target: *target,
                step: *step,
                preserved: false,
            }),
            Progress::Preserved { target, step, .. } => Some(Pending::Step {
                target: *target,
                step: *step,
                preserved: true,
            }),
            Progress::Prefix { target, prefix, .. } => Some(Pending::Prefix {
                target: *target,
                prefix: *prefix,
            }),
            Progress::ReadyInPlace(_) | Progress::ReadyPreserved { .. } => None,
        }
    }
    /// The next action, `Ok(None)` when the append is ready. It returns the latched failure, or
    /// the not-ready error while an observation awaits `accept`.
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionTextAppendAction<'_>>, HtmlConstructionError> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        if self.observing {
            return Err(HtmlConstructionErrorKind::OperationNotReady.into());
        }
        Ok(self
            .pending()
            .map(|pending| HtmlConstructionTextAppendAction {
                owner: self,
                pending,
            }))
    }
    /// Consumes a ready append. Returns the latched failure, or the not-ready error before the
    /// last action was accepted, or when the qualified target is no longer a text node; either
    /// way the append is handed back unchanged.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn try_ready(
        self,
    ) -> Result<HtmlConstructionTextAppendReady, HtmlConstructionRefusal<Self>> {
        if let Some(error) = &self.failure {
            let error = error.clone();
            return Err(HtmlConstructionRefusal::new(self, error));
        }
        let target = match &self.progress {
            Progress::ReadyInPlace(target) | Progress::ReadyPreserved { target, .. }
                if !self.observing && text_of(&self.storage, target.index).is_some() =>
            {
                *target
            }
            _ => {
                return Err(HtmlConstructionRefusal::new(
                    self,
                    HtmlConstructionErrorKind::OperationNotReady,
                ));
            }
        };
        let Self {
            storage,
            node,
            incoming,
            progress,
            observing,
            failure,
        } = self;
        let backing = match progress {
            Progress::ReadyInPlace(target) => ReadyBacking::InPlace {
                old_len: target.old_len,
            },
            Progress::ReadyPreserved { copy, .. } => ReadyBacking::Preserved(copy),
            // The check above admitted only the two ready progresses.
            progress => {
                return Err(HtmlConstructionRefusal::new(
                    Self {
                        storage,
                        node,
                        incoming,
                        progress,
                        observing,
                        failure,
                    },
                    HtmlConstructionErrorKind::OperationNotReady,
                ));
            }
        };
        Ok(HtmlConstructionTextAppendReady {
            storage,
            node,
            incoming,
            index: target.index,
            usage: target.usage,
            backing,
        })
    }
    /// Removes initialized tail from reused native backing before storage escapes.
    /// A distinct pending copy keeps all initialized units for its actual ticket's
    /// stopped retention and disposal. Neither copy nor tail becomes native-visible.
    /// Successful growth is retained and remains visible in actual capacity.
    ///
    /// A backing that another holder shares cannot be truncated without an unmetered copy; the
    /// abort is then refused with the shared-allocation error and the append is handed back
    /// unchanged.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn abort(
        mut self,
    ) -> Result<HtmlConstructionTextAppendCompletion, HtmlConstructionRefusal<Self>> {
        let truncated = match &self.progress {
            Progress::InPlace { target, step } if step.tail_is_exclusive() => {
                truncate_tail(&mut self.storage, target.index, target.old_len)
            }
            Progress::ReadyInPlace(target) => {
                truncate_tail(&mut self.storage, target.index, target.old_len)
            }
            _ => Ok(()),
        };
        if let Err(error) = truncated {
            return Err(HtmlConstructionRefusal::new(self, error));
        }
        let discarded = match self.progress {
            Progress::Prefix { copy, .. }
            | Progress::Preserved { copy, .. }
            | Progress::ReadyPreserved { copy, .. } => Some(copy),
            Progress::InPlace { .. }
            | Progress::ReadyInPlace(_)
            | Progress::Inspect
            | Progress::Select(_)
            | Progress::Initialize(_) => None,
        };
        Ok(HtmlConstructionTextAppendCompletion {
            storage: self.storage,
            node: self.node,
            incoming: self.incoming,
            discarded,
        })
    }
}
/// The action the append is about to issue, which carries what its description needs.
#[derive(Clone, Copy, Debug)]
enum Pending {
    Inspect,
    Select,
    Initialize,
    /// A step of the reused backing, or of the separate copy when `preserved`.
    Step {
        target: Target,
        step: Step,
        preserved: bool,
    },
    /// Copying the original text into the separate backing, from `prefix` units on.
    Prefix {
        target: Target,
        prefix: usize,
    },
}

impl<'a> HtmlConstructionTextAppendAction<'a> {
    /// The original visible tree, available only before tail mutation begins.
    pub fn storage_view(&self) -> Option<&HtmlConstructionStorage> {
        match self.pending {
            Pending::Step {
                step: Step::Preflight,
                ..
            } => Some(&self.owner.storage),
            _ => None,
        }
    }
    /// Actual reuse or preserved-copy candidate at the pre-tail boundary.
    pub fn backing(&self) -> Option<&DocumentString> {
        self.storage_view()?;
        self.owner.backing()
    }
    /// Describes the pending action's work. The consumer admits that work before calling
    /// `perform`.
    pub fn description(&self) -> HtmlConstructionTextAppendDescription {
        use HtmlConstructionTextAppendDescription as D;
        let owner = &self.owner;
        // Every action stores the progress it reaches and holds the append with the observing
        // flag, and accept releases it; a failed action stores its typed failure instead. Admit
        // the larger concrete path.
        let mut work = HtmlConstructionTextAppendWork {
            record_writes: 3,
            copied_bytes: (size_of::<Progress>() + 2 * size_of::<bool>())
                .max(size_of::<Option<HtmlConstructionError>>()),
            atomic_layout_bytes: size_of::<Option<HtmlConstructionError>>()
                .max(size_of::<Progress>()),
            ..Default::default()
        };
        match self.pending {
            Pending::Inspect => {
                work.record_visits += 6;
                work.provider_calls += 1;
                work.atomic_layout_bytes = work
                    .atomic_layout_bytes
                    .max(size_of::<HtmlConstructionUsage>());
                D::Inspect(work)
            }
            Pending::Select => {
                work.record_visits += 3;
                work.provider_calls += 1;
                D::SelectBacking(work)
            }
            Pending::Initialize => {
                work.provider_calls += 1;
                work.record_writes += 1;
                work.copied_bytes += size_of::<Vec<u16>>();
                work.atomic_layout_bytes = work.atomic_layout_bytes.max(size_of::<Vec<u16>>());
                D::InitializeBacking(work)
            }
            Pending::Prefix { target, prefix } => {
                let units = (target.old_len - prefix).min(CHUNK_UNITS);
                copy_work(&mut work, units);
                D::CopyPrefix {
                    offset: prefix,
                    units,
                    work,
                }
            }
            Pending::Step {
                target,
                step,
                preserved,
            } => match step {
                Step::Reserve { before } => {
                    work.record_visits += 3;
                    work.provider_calls += 1;
                    work.record_writes += 2;
                    work.copied_bytes += size_of::<Vec<u16>>();
                    work.atomic_layout_bytes = work.atomic_layout_bytes.max(size_of::<Vec<u16>>());
                    work.relocation_bound_bytes = before.capacity * size_of::<u16>();
                    let max_units = isize::MAX as usize / size_of::<u16>();
                    let preferred_capacity = before
                        .capacity
                        .saturating_mul(2)
                        .max(target.required)
                        .min(max_units);
                    D::Reserve {
                        before,
                        required_capacity: target.required,
                        preferred_capacity,
                        work,
                    }
                }
                Step::CopyTail { tail } => {
                    let units = (owner.incoming.len() - tail).min(CHUNK_UNITS);
                    copy_work(&mut work, units);
                    D::CopyTail {
                        offset: tail,
                        units,
                        work,
                    }
                }
                Step::Preflight => D::Preflight {
                    usage: target.usage,
                    work,
                },
                Step::QualifyTail => {
                    work.record_visits += 3;
                    work.provider_calls += 1;
                    D::QualifyTail(work)
                }
                Step::Commit => {
                    // These writes are prepaid here and executed only by Ready::commit.
                    work.record_writes += if preserved { 3 } else { 1 };
                    work.copied_bytes += size_of::<HtmlConstructionUsage>();
                    work.atomic_layout_bytes = work
                        .atomic_layout_bytes
                        .max(size_of::<HtmlConstructionUsage>());
                    if preserved {
                        work.copied_bytes +=
                            2 * size_of::<DocumentString>() + size_of::<Option<DocumentString>>();
                        work.atomic_layout_bytes = work
                            .atomic_layout_bytes
                            .max(size_of::<Option<DocumentString>>());
                    }
                    D::Commit {
                        usage: target.usage,
                        work,
                    }
                }
            },
        }
    }
    /// Reserve alone accepts a clipped capacity between required and preferred.
    /// Caller admission precedes this action; observation precedes its next poll.
    pub fn perform(
        self,
        minimum_capacity: Option<usize>,
    ) -> Result<HtmlConstructionTextAppendObservation<'a>, HtmlConstructionError> {
        let owner = self.owner;
        let HtmlConstructionTextAppend {
            storage,
            node,
            incoming,
            progress,
            observing,
            failure,
        } = &mut *owner;
        let result = if minimum_capacity.is_some()
            && !matches!(
                self.pending,
                Pending::Step {
                    step: Step::Reserve { .. },
                    ..
                }
            ) {
            Err(HtmlConstructionErrorKind::CapacityMismatch.into())
        } else {
            advance_progress(storage, node, incoming, progress, minimum_capacity)
        };
        match result {
            Ok(()) => {
                *observing = true;
                Ok(HtmlConstructionTextAppendObservation { owner })
            }
            Err(error) => {
                *failure = Some(error.clone());
                Err(error)
            }
        }
    }
}
/// The work of copying `units` units of text into a backing: the units themselves and the length
/// they extend. The progress counter is part of the stored progress.
fn copy_work(work: &mut HtmlConstructionTextAppendWork, units: usize) {
    work.record_visits += 3;
    work.provider_calls += 1;
    work.record_writes += 1;
    work.copied_bytes += units * size_of::<u16>() + size_of::<usize>();
    work.atomic_layout_bytes = work
        .atomic_layout_bytes
        .max(units * size_of::<u16>())
        .max(size_of::<usize>());
}
/// The progress that one pending step reaches.
enum Next {
    /// Another step of the same backing.
    Step(Step),
    /// The reservation finished.
    Reserved,
    /// The commit step finished.
    Ready,
}
/// Takes the progress by value so that its payload moves into the next progress, and puts it
/// back unchanged when the step fails.
fn advance_progress(
    storage: &mut HtmlConstructionStorage,
    node: &HtmlBuilderNode,
    incoming: &DocumentString,
    progress: &mut Progress,
    minimum_capacity: Option<usize>,
) -> Result<(), HtmlConstructionError> {
    match std::mem::replace(progress, Progress::Inspect) {
        Progress::Inspect => {
            let target = storage.position(node)?;
            let HtmlNodeKind::Text(value) = &storage.arena.records[target].kind else {
                return Err(HtmlConstructionErrorKind::WrongNodeKind.into());
            };
            let old_len = value.len();
            let required = old_len
                .checked_add(incoming.len())
                .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?;
            if required > isize::MAX as usize / size_of::<u16>() {
                return Err(HtmlConstructionErrorKind::RepresentationExhausted.into());
            }
            let mut usage = storage.usage();
            usage.utf16_units = usage
                .utf16_units
                .checked_add(incoming.len())
                .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?;
            *progress = Progress::Select(Target {
                index: target,
                old_len,
                required,
                usage,
            });
        }
        Progress::Select(target) => {
            *progress = Progress::Select(target);
            let original = text_of_mut(storage, target.index)
                .ok_or(HtmlConstructionErrorKind::WrongNodeKind)?;
            let preserve = Arc::get_mut(original.construction_backing_mut()).is_none();
            *progress = if preserve {
                Progress::Initialize(target)
            } else {
                let before = HtmlConstructionTextAppendCapacity::of(original);
                Progress::InPlace {
                    target,
                    step: if before.capacity < target.required {
                        Step::Reserve { before }
                    } else {
                        Step::Preflight
                    },
                }
            };
        }
        Progress::Initialize(target) => {
            let copy = DocumentString::from_utf16(Vec::new());
            let before = HtmlConstructionTextAppendCapacity::of(&copy);
            *progress = Progress::Preserved {
                target,
                copy,
                step: if target.required == 0 {
                    Step::Preflight
                } else {
                    Step::Reserve { before }
                },
            };
        }
        Progress::InPlace { target, step } => {
            match run(storage, incoming, target, step, None, minimum_capacity) {
                Ok(Next::Step(step)) => *progress = Progress::InPlace { target, step },
                Ok(Next::Reserved) => {
                    *progress = Progress::InPlace {
                        target,
                        step: Step::Preflight,
                    }
                }
                Ok(Next::Ready) => *progress = Progress::ReadyInPlace(target),
                Err(error) => {
                    *progress = Progress::InPlace { target, step };
                    return Err(error);
                }
            }
        }
        Progress::Preserved {
            target,
            mut copy,
            step,
        } => match run(
            storage,
            incoming,
            target,
            step,
            Some(&mut copy),
            minimum_capacity,
        ) {
            Ok(Next::Step(step)) => *progress = Progress::Preserved { target, copy, step },
            Ok(Next::Reserved) => {
                *progress = if target.old_len > 0 {
                    Progress::Prefix {
                        target,
                        copy,
                        prefix: 0,
                    }
                } else {
                    Progress::Preserved {
                        target,
                        copy,
                        step: Step::Preflight,
                    }
                }
            }
            Ok(Next::Ready) => *progress = Progress::ReadyPreserved { target, copy },
            Err(error) => {
                *progress = Progress::Preserved { target, copy, step };
                return Err(error);
            }
        },
        Progress::Prefix {
            target,
            mut copy,
            prefix,
        } => {
            let result = (|| -> Result<usize, HtmlConstructionError> {
                let end = prefix + (target.old_len - prefix).min(CHUNK_UNITS);
                let original = text_of(storage, target.index)
                    .ok_or(HtmlConstructionErrorKind::WrongNodeKind)?;
                let backing = Arc::get_mut(copy.construction_backing_mut())
                    .ok_or(HtmlConstructionErrorKind::SharedAllocationAliased)?;
                backing.extend_from_slice(&original.code_units()[prefix..end]);
                Ok(end)
            })();
            match result {
                Ok(end) if end == target.old_len => {
                    *progress = Progress::Preserved {
                        target,
                        copy,
                        step: Step::Preflight,
                    }
                }
                Ok(end) => {
                    *progress = Progress::Prefix {
                        target,
                        copy,
                        prefix: end,
                    }
                }
                Err(error) => {
                    *progress = Progress::Prefix {
                        target,
                        copy,
                        prefix,
                    };
                    return Err(error);
                }
            }
        }
        // A ready append issues no action, so no step performs on it.
        ready @ (Progress::ReadyInPlace(_) | Progress::ReadyPreserved { .. }) => {
            *progress = ready;
            return Err(HtmlConstructionErrorKind::OperationNotReady.into());
        }
    }
    Ok(())
}
/// Performs one step of a backing that is chosen. `copy` is the separate backing when the append
/// preserves the original, and the candidate is the original text otherwise.
fn run(
    storage: &mut HtmlConstructionStorage,
    incoming: &DocumentString,
    target: Target,
    step: Step,
    copy: Option<&mut DocumentString>,
    minimum_capacity: Option<usize>,
) -> Result<Next, HtmlConstructionError> {
    match step {
        Step::Reserve { before } => {
            let preferred = before
                .capacity
                .saturating_mul(2)
                .max(target.required)
                .min(isize::MAX as usize / size_of::<u16>());
            let minimum = minimum_capacity.unwrap_or(preferred);
            if minimum < target.required || minimum > preferred {
                return Err(HtmlConstructionErrorKind::CapacityMismatch.into());
            }
            let candidate = match copy {
                Some(copy) => copy,
                None => text_of_mut(storage, target.index)
                    .ok_or(HtmlConstructionErrorKind::WrongNodeKind)?,
            };
            let backing = Arc::get_mut(candidate.construction_backing_mut())
                .ok_or(HtmlConstructionErrorKind::SharedAllocationAliased)?;
            backing
                .try_reserve_exact(minimum - backing.len())
                .map_err(|_| HtmlConstructionErrorKind::AllocationFailed)?;
            Ok(Next::Reserved)
        }
        Step::Preflight => Ok(Next::Step(Step::QualifyTail)),
        Step::QualifyTail => {
            let candidate = match copy {
                Some(copy) => copy,
                None => text_of_mut(storage, target.index)
                    .ok_or(HtmlConstructionErrorKind::WrongNodeKind)?,
            };
            if Arc::get_mut(candidate.construction_backing_mut()).is_none() {
                return Err(HtmlConstructionErrorKind::SharedAllocationAliased.into());
            }
            Ok(Next::Step(if incoming.is_empty() {
                Step::Commit
            } else {
                Step::CopyTail { tail: 0 }
            }))
        }
        Step::CopyTail { tail: start } => {
            let end = start + (incoming.len() - start).min(CHUNK_UNITS);
            let candidate = match copy {
                Some(copy) => copy,
                None => text_of_mut(storage, target.index)
                    .ok_or(HtmlConstructionErrorKind::WrongNodeKind)?,
            };
            // The tail qualification proved the backing uniquely owned, and nothing runs
            // between it and these writes that could share it again.
            let backing = Arc::get_mut(candidate.construction_backing_mut())
                .ok_or(HtmlConstructionErrorKind::SharedAllocationAliased)?;
            backing.extend_from_slice(&incoming.code_units()[start..end]);
            Ok(Next::Step(if end == incoming.len() {
                Step::Commit
            } else {
                Step::CopyTail { tail: end }
            }))
        }
        Step::Commit => Ok(Next::Ready),
    }
}
impl HtmlConstructionTextAppendObservation<'_> {
    /// The dimensions of the candidate backing after the performed action. `None` under the same
    /// conditions as the owner's `capacity`.
    pub fn capacity(&self) -> Option<HtmlConstructionTextAppendCapacity> {
        self.owner.capacity()
    }
    /// Whether aliases of the original text require the append to work in a separate preserved
    /// copy.
    pub fn preserves_original(&self) -> bool {
        self.owner.preserves_original()
    }
    /// Before tail qualification only. Suspend newly issued Weak before resuming.
    pub fn backing(&self) -> Option<&DocumentString> {
        self.owner.backing()
    }
    /// Accepts the observation, which resumes the append at the progress the action reached.
    pub fn accept(self) {
        self.owner.observing = false;
    }
}
impl HtmlConstructionTextAppendReady {
    /// Abandons the ready append and returns its operands. Any separate preserved copy is returned
    /// for retirement. A backing that another holder shares cannot be truncated without an
    /// unmetered copy; the abort is then refused with the shared-allocation error and the append
    /// is handed back unchanged.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn abort(
        mut self,
    ) -> Result<HtmlConstructionTextAppendCompletion, HtmlConstructionRefusal<Self>> {
        if let ReadyBacking::InPlace { old_len } = self.backing
            && let Err(error) = truncate_tail(&mut self.storage, self.index, old_len)
        {
            return Err(HtmlConstructionRefusal::new(self, error));
        }
        let discarded = match self.backing {
            ReadyBacking::InPlace { .. } => None,
            ReadyBacking::Preserved(copy) => Some(copy),
        };
        Ok(HtmlConstructionTextAppendCompletion {
            storage: self.storage,
            node: self.node,
            incoming: self.incoming,
            discarded,
        })
    }
    /// Only prepaid owning moves and fixed usage writes; no payload copy. A target that is no
    /// longer a text node is refused, which `try_ready` checked and the exclusively owned storage
    /// cannot change; the ready append is handed back unchanged to abort.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn commit(
        mut self,
    ) -> Result<HtmlConstructionTextAppendCompletion, HtmlConstructionRefusal<Self>> {
        let Some(original) = text_of_mut(&mut self.storage, self.index) else {
            return Err(HtmlConstructionRefusal::new(
                self,
                HtmlConstructionErrorKind::OperationNotReady,
            ));
        };
        let discarded = match self.backing {
            ReadyBacking::InPlace { .. } => None,
            ReadyBacking::Preserved(copy) => Some(std::mem::replace(original, copy)),
        };
        self.storage.arena.usage = self.usage;
        Ok(HtmlConstructionTextAppendCompletion {
            storage: self.storage,
            node: self.node,
            incoming: self.incoming,
            discarded,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::construction::bounded::operations::HtmlConstructionCommit;

    fn make(kind: HtmlNodeKind) -> (HtmlConstructionStorage, HtmlBuilderNode) {
        let mut initialization = Some(HtmlConstructionStorage::begin());
        initialization
            .as_mut()
            .unwrap()
            .next_action()
            .unwrap()
            .unwrap()
            .perform()
            .accept();
        let storage = slots::initialization(&mut initialization).unwrap().commit();
        let mut operation = Some(storage.prepare(HtmlConstructionCommand::Create { kind }));
        while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        let HtmlConstructionCommit::Changed {
            storage,
            created: Some(node),
            ..
        } = slots::operation(&mut operation).unwrap().commit().unwrap()
        else {
            panic!("created actual node")
        };
        (storage, node)
    }
    fn text(storage: &HtmlConstructionStorage, node: &HtmlBuilderNode) -> Vec<u16> {
        let HtmlNodeKind::Text(value) = storage.node(node).unwrap().kind() else {
            panic!("text")
        };
        value.code_units().to_vec()
    }
    fn append(
        storage: HtmlConstructionStorage,
        node: HtmlBuilderNode,
        incoming: DocumentString,
    ) -> (
        HtmlConstructionStorage,
        HtmlBuilderNode,
        DocumentString,
        Option<DocumentString>,
        usize,
        usize,
    ) {
        let mut owner = Some(HtmlConstructionTextAppend::begin(storage, node, incoming));
        let mut prefix_units = 0;
        let mut tail_units = 0;
        while let Some(action) = owner.as_mut().unwrap().next_action().unwrap() {
            match action.description() {
                HtmlConstructionTextAppendDescription::CopyPrefix { units, work, .. } => {
                    assert!(units <= 128);
                    assert!(work.atomic_layout_bytes <= 256);
                    prefix_units += units;
                }
                HtmlConstructionTextAppendDescription::CopyTail { units, work, .. } => {
                    assert!(units <= 128);
                    assert!(work.atomic_layout_bytes <= 256);
                    tail_units += units;
                }
                _ => {}
            }
            action.perform(None).unwrap().accept();
        }
        let (storage, node, incoming, discarded) = slots::text_append(&mut owner)
            .unwrap()
            .commit()
            .unwrap()
            .into_parts();
        (storage, node, incoming, discarded, prefix_units, tail_units)
    }
    #[test]
    fn actions_charge_the_progress_store_and_the_two_observation_flag_writes() {
        // Derived from the writes themselves: an action stores the progress it reaches and sets
        // the observing flag, accept clears the flag, and a failed action stores its typed failure
        // instead, so admission covers the larger of the two paths.
        let transition_bytes = (size_of::<Progress>() + 2 * size_of::<bool>())
            .max(size_of::<Option<HtmlConstructionError>>());
        let transition_atomic =
            size_of::<Progress>().max(size_of::<Option<HtmlConstructionError>>());
        let (storage, node) = make(HtmlNodeKind::Text(DocumentString::from_utf16(vec![1, 2])));
        let mut owner = HtmlConstructionTextAppend::begin(
            storage,
            node,
            DocumentString::from_utf16(vec![3, 4, 5]),
        );
        let inspect = owner.next_action().unwrap().unwrap();
        assert_eq!(
            inspect.description(),
            HtmlConstructionTextAppendDescription::Inspect(HtmlConstructionTextAppendWork {
                record_visits: 6,
                provider_calls: 1,
                record_writes: 3,
                copied_bytes: transition_bytes,
                atomic_layout_bytes: transition_atomic.max(size_of::<HtmlConstructionUsage>()),
                ..Default::default()
            })
        );
        inspect.perform(None).unwrap().accept();
        drive_to(&mut owner, Phase::CopyTail);
        let copy = owner.next_action().unwrap().unwrap();
        let units = 3;
        assert_eq!(
            copy.description(),
            HtmlConstructionTextAppendDescription::CopyTail {
                offset: 0,
                units,
                work: HtmlConstructionTextAppendWork {
                    record_visits: 3,
                    provider_calls: 1,
                    // The units extend the backing; the offset counter is part of the progress.
                    record_writes: 3 + 1,
                    copied_bytes: transition_bytes + units * size_of::<u16>() + size_of::<usize>(),
                    atomic_layout_bytes: transition_atomic
                        .max(units * size_of::<u16>())
                        .max(size_of::<usize>()),
                    ..Default::default()
                },
            }
        );
    }

    fn drive_to(owner: &mut HtmlConstructionTextAppend, phase: Phase) {
        while owner.phase() != phase {
            owner
                .next_action()
                .unwrap()
                .unwrap()
                .perform(None)
                .unwrap()
                .accept();
        }
    }

    #[test]
    fn spare_unique_backing_appends_only_incoming_and_retains_allocation() {
        let mut units = Vec::with_capacity(512);
        units.extend_from_slice(&[0, 0xd800, 0xdc00]);
        let original = DocumentString::from_utf16(units);
        let address = original.code_units().as_ptr() as usize;
        let (storage, node) = make(HtmlNodeKind::Text(original));
        let (storage, node, _, discarded, prefix, tail) =
            append(storage, node, DocumentString::from_utf16(vec![0xffff, 0]));
        assert!(discarded.is_none());
        assert_eq!((prefix, tail), (0, 2));
        assert_eq!(text(&storage, &node), [0, 0xd800, 0xdc00, 0xffff, 0]);
        assert_eq!(storage.usage().utf16_units(), 5);
        let HtmlNodeKind::Text(value) = storage.node(&node).unwrap().kind() else {
            panic!("text")
        };
        assert_eq!(value.code_units().as_ptr() as usize, address);
        assert_eq!(value.shared_code_units().capacity(), 512);
    }

    #[test]
    fn shared_history_is_copied_once_then_unique_aggregate_is_reused() {
        let original = DocumentString::from_utf16(vec![0xd800; 400]);
        let history = original.clone();
        let (storage, node) = make(HtmlNodeKind::Text(original));
        let (storage, node, _, discarded, prefix, tail) =
            append(storage, node, DocumentString::from("a"));
        assert_eq!((prefix, tail), (400, 1));
        let discarded = discarded.expect("preserved original backing");
        assert!(Arc::ptr_eq(
            discarded.shared_code_units(),
            history.shared_code_units()
        ));
        assert_eq!(history.code_units(), vec![0xd800; 400]);
        drop(discarded);
        let (storage, node, _, discarded, prefix, tail) =
            append(storage, node, DocumentString::from("b"));
        assert!(discarded.is_none());
        assert_eq!((prefix, tail), (0, 1));
        let mut expected = vec![0xd800; 400];
        expected.extend_from_slice(&[u16::from(b'a'), u16::from(b'b')]);
        assert_eq!(text(&storage, &node), expected);
        assert_eq!(history.code_units(), vec![0xd800; 400]);
    }

    #[test]
    fn external_weak_requires_preservation_and_keeps_original_immutable() {
        let original = DocumentString::from("original");
        let weak = Arc::downgrade(original.shared_code_units());
        let (storage, node) = make(HtmlNodeKind::Text(original));
        let (storage, node, _, discarded, prefix, tail) =
            append(storage, node, DocumentString::from("+"));
        assert_eq!((prefix, tail), (8, 1));
        assert_eq!(
            weak.upgrade().unwrap().as_slice(),
            "original".encode_utf16().collect::<Vec<_>>()
        );
        assert!(discarded.is_some());
        assert_eq!(
            text(&storage, &node),
            "original+".encode_utf16().collect::<Vec<_>>()
        );
    }

    #[test]
    fn abort_after_tail_chunk_restores_text_and_retains_actual_growth() {
        let (storage, node) = make(HtmlNodeKind::Text(DocumentString::from("x")));
        let mut owner = HtmlConstructionTextAppend::begin(
            storage,
            node,
            DocumentString::from_utf16(vec![0xd800; 300]),
        );
        drive_to(&mut owner, Phase::CopyTail);
        let before = owner.capacity().unwrap();
        assert!(before.capacity() >= 301);
        {
            let observation = owner.next_action().unwrap().unwrap().perform(None).unwrap();
            assert_eq!(observation.capacity().unwrap().initialized_len(), 129);
            assert!(observation.backing().is_none());
            // Losing this unaccepted observation must still recover real writes.
        }
        assert!(matches!(
            owner.next_action(),
            Err(ref error) if error.kind == HtmlConstructionErrorKind::OperationNotReady
        ));
        let (storage, node, incoming, discarded) = owner.abort().unwrap().into_parts();
        assert!(discarded.is_none());
        assert_eq!(incoming.len(), 300);
        assert_eq!(text(&storage, &node), [u16::from(b'x')]);
        assert_eq!(storage.usage().utf16_units(), 1);
        let HtmlNodeKind::Text(value) = storage.node(&node).unwrap().kind() else {
            panic!("text")
        };
        assert_eq!(value.shared_code_units().capacity(), before.capacity());
        assert_eq!(value.code_units().as_ptr() as usize, before.data_address());
    }

    #[test]
    fn abort_during_prefix_returns_original_and_actual_partial_pending() {
        let original = DocumentString::from_utf16(vec![0xffff; 300]);
        let history = original.clone();
        let (storage, node) = make(HtmlNodeKind::Text(original));
        let mut owner = HtmlConstructionTextAppend::begin(storage, node, DocumentString::from("z"));
        drive_to(&mut owner, Phase::CopyPrefix);
        owner
            .next_action()
            .unwrap()
            .unwrap()
            .perform(None)
            .unwrap()
            .accept();
        assert_eq!(owner.capacity().unwrap().initialized_len(), 128);
        let (storage, node, _, pending) = owner.abort().unwrap().into_parts();
        assert_eq!(text(&storage, &node), history.code_units());
        let pending = pending.expect("actual pending allocation");
        assert_eq!(pending.code_units(), vec![0xffff; 128]);
        assert!(pending.shared_code_units().capacity() >= 301);
        assert!(!Arc::ptr_eq(
            pending.shared_code_units(),
            history.shared_code_units()
        ));
    }

    #[test]
    fn abort_preserved_tail_retains_actual_pending_units_and_unchanged_native_text() {
        for complete_tail in [false, true] {
            let original = DocumentString::from_utf16(vec![0xffff; 300]);
            let history = original.clone();
            let (storage, node) = make(HtmlNodeKind::Text(original));
            let mut owner = HtmlConstructionTextAppend::begin(
                storage,
                node,
                DocumentString::from_utf16(vec![0xd800; 300]),
            );
            drive_to(&mut owner, Phase::CopyTail);
            if complete_tail {
                drive_to(&mut owner, Phase::Ready);
            } else {
                // Actual tail writes remain owned even if their observation is lost.
                let observation = owner.next_action().unwrap().unwrap().perform(None).unwrap();
                assert_eq!(observation.capacity().unwrap().initialized_len(), 428);
                assert!(observation.backing().is_none());
            }
            let actual = owner.capacity().unwrap();
            let completion = if complete_tail {
                slots::text_append(&mut Some(owner))
                    .unwrap()
                    .abort()
                    .unwrap()
            } else {
                owner.abort().unwrap()
            };
            let (storage, node, incoming, pending) = completion.into_parts();
            assert_eq!(text(&storage, &node), vec![0xffff; 300]);
            assert_eq!(history.code_units(), vec![0xffff; 300]);
            assert_eq!(storage.usage().utf16_units(), 300);
            assert_eq!(incoming.code_units(), vec![0xd800; 300]);
            let pending = pending.expect("actual initialized pending aggregate");
            let tail_len = if complete_tail { 300 } else { 128 };
            let mut expected = vec![0xffff; 300];
            expected.extend(std::iter::repeat_n(0xd800, tail_len));
            assert_eq!(pending.code_units(), expected);
            assert_eq!(pending.len(), actual.initialized_len());
            assert_eq!(pending.shared_code_units().capacity(), actual.capacity());
            assert_eq!(
                pending.code_units().as_ptr() as usize,
                actual.data_address()
            );
            assert!(!Arc::ptr_eq(
                pending.shared_code_units(),
                history.shared_code_units()
            ));
        }
    }

    #[test]
    fn pending_observation_can_issue_actual_weak_before_bounded_copy() {
        let original = DocumentString::from_utf16(vec![7; 300]);
        let _history = original.clone();
        let (storage, node) = make(HtmlNodeKind::Text(original));
        let mut owner = HtmlConstructionTextAppend::begin(storage, node, DocumentString::from("z"));
        drive_to(&mut owner, Phase::InitializeBacking);
        let observation = owner.next_action().unwrap().unwrap().perform(None).unwrap();
        let actual = observation.backing().expect("real empty pending backing");
        assert!(actual.is_empty());
        let weak = Arc::downgrade(actual.shared_code_units());
        assert_eq!(
            observation.capacity().unwrap().wrapper_payload_bytes(),
            size_of::<Vec<u16>>()
        );
        drop(weak);
        observation.accept();
        drive_to(&mut owner, Phase::Preflight);
        let action = owner.next_action().unwrap().unwrap();
        assert_eq!(
            text(action.storage_view().unwrap(), &action.owner.node),
            vec![7; 300]
        );
        assert_eq!(action.backing().unwrap().code_units(), vec![7; 300]);
        let weak = Arc::downgrade(action.backing().unwrap().shared_code_units());
        drop(weak);
        action.perform(None).unwrap().accept();
        drive_to(&mut owner, Phase::Ready);
        let (storage, node, _, _) = slots::text_append(&mut Some(owner))
            .unwrap()
            .commit()
            .unwrap()
            .into_parts();
        assert_eq!(text(&storage, &node).last(), Some(&u16::from(b'z')));
    }

    #[test]
    fn alias_acquired_at_preflight_refuses_final_qualification_without_mutation() {
        for weak_only in [false, true] {
            let mut units = Vec::with_capacity(32);
            units.extend_from_slice(&[1, 2, 3]);
            let (storage, node) = make(HtmlNodeKind::Text(DocumentString::from_utf16(units)));
            let mut owner =
                HtmlConstructionTextAppend::begin(storage, node, DocumentString::from("z"));
            drive_to(&mut owner, Phase::Preflight);
            let action = owner.next_action().unwrap().unwrap();
            let alias = (!weak_only).then(|| action.backing().unwrap().clone());
            let weak =
                weak_only.then(|| Arc::downgrade(action.backing().unwrap().shared_code_units()));
            action.perform(None).unwrap().accept();
            assert!(matches!(
                owner.next_action().unwrap().unwrap().perform(None),
                Err(ref error) if error.kind == HtmlConstructionErrorKind::SharedAllocationAliased
            ));
            let (storage, node, _, discarded) = owner.abort().unwrap().into_parts();
            assert_eq!(text(&storage, &node), [1, 2, 3]);
            assert_eq!(storage.usage().utf16_units(), 3);
            assert!(discarded.is_none());
            drop((alias, weak));
        }
    }

    #[test]
    fn foreign_and_wrong_kind_refusals_return_unchanged_actual_storage() {
        let (_, foreign) = make(HtmlNodeKind::Text(DocumentString::from("foreign")));
        let (storage, node) = make(HtmlNodeKind::Comment(DocumentString::from("comment")));
        let usage = storage.usage();
        let mut owner =
            HtmlConstructionTextAppend::begin(storage, foreign, DocumentString::from("x"));
        assert!(matches!(
            owner.next_action().unwrap().unwrap().perform(None),
            Err(ref error) if error.kind == HtmlConstructionErrorKind::ForeignBuilder
        ));
        let (storage, _, incoming, discarded) = owner.abort().unwrap().into_parts();
        assert_eq!(storage.usage(), usage);
        assert_eq!(incoming.code_units(), [u16::from(b'x')]);
        assert!(discarded.is_none());
        let mut owner = HtmlConstructionTextAppend::begin(storage, node, incoming);
        assert!(matches!(
            owner.next_action().unwrap().unwrap().perform(None),
            Err(ref error) if error.kind == HtmlConstructionErrorKind::WrongNodeKind
        ));
        assert_eq!(owner.abort().unwrap().into_parts().0.usage(), usage);
    }

    #[test]
    fn clipped_growth_is_real_and_invalid_capacity_refuses_before_reservation() {
        for invalid in [false, true] {
            let mut units = Vec::with_capacity(8);
            units.extend_from_slice(&[1; 8]);
            let (storage, node) = make(HtmlNodeKind::Text(DocumentString::from_utf16(units)));
            let mut owner =
                HtmlConstructionTextAppend::begin(storage, node, DocumentString::from("z"));
            drive_to(&mut owner, Phase::Reserve);
            let action = owner.next_action().unwrap().unwrap();
            let HtmlConstructionTextAppendDescription::Reserve {
                before,
                required_capacity,
                preferred_capacity,
                ..
            } = action.description()
            else {
                panic!("real reserve")
            };
            assert_eq!(
                (before.capacity(), required_capacity, preferred_capacity),
                (8, 9, 16)
            );
            if invalid {
                assert!(matches!(
                    action.perform(Some(8)),
                    Err(ref error) if error.kind == HtmlConstructionErrorKind::CapacityMismatch
                ));
                assert_eq!(owner.capacity().unwrap().capacity(), 8);
            } else {
                action.perform(Some(9)).unwrap().accept();
                assert!(owner.capacity().unwrap().capacity() >= 9);
            }
            let (storage, node, _, discarded) = owner.abort().unwrap().into_parts();
            assert_eq!(text(&storage, &node), [1; 8]);
            assert_eq!(storage.usage().utf16_units(), 8);
            assert!(discarded.is_none());
        }
    }

    #[test]
    fn unaccepted_final_observation_cannot_commit_and_ready_abort_restores_original() {
        for accept in [false, true] {
            let (storage, node) = make(HtmlNodeKind::Text(DocumentString::from("x")));
            let mut owner =
                HtmlConstructionTextAppend::begin(storage, node, DocumentString::from("y"));
            drive_to(&mut owner, Phase::Commit);
            let observation = owner.next_action().unwrap().unwrap().perform(None).unwrap();
            assert!(observation.backing().is_none());
            if accept {
                observation.accept();
            } else {
                #[expect(
                    clippy::forget_non_drop,
                    reason = "forgotten observations must not authorize readiness"
                )]
                std::mem::forget(observation);
            }
            let mut owner = Some(owner);
            let completion = if accept {
                slots::text_append(&mut owner).unwrap().abort().unwrap()
            } else {
                assert!(matches!(
                    slots::text_append(&mut owner),
                    Err(ref error) if error.kind == HtmlConstructionErrorKind::OperationNotReady
                ));
                owner.take().unwrap().abort().unwrap()
            };
            let (storage, node, _, discarded) = completion.into_parts();
            assert_eq!(text(&storage, &node), [u16::from(b'x')]);
            assert_eq!(storage.usage().utf16_units(), 1);
            assert!(discarded.is_none());
        }
    }

    /// The abort restores the original length of the record's own text in place. The qualification
    /// found the backing exclusive, so nothing outside the append can hold it; were something to,
    /// truncating would need a copy that nothing metered, so the abort is refused, hands the
    /// append back with the shared backing and the other holder's view untouched, and succeeds
    /// once the backing is exclusive again.
    #[test]
    fn abort_refuses_a_backing_another_holder_shares_and_succeeds_once_it_is_alone() {
        let mut units = Vec::with_capacity(8);
        units.push(u16::from(b'x'));
        let (storage, node) = make(HtmlNodeKind::Text(DocumentString::from_shared_code_units(
            Arc::new(units),
        )));
        let mut owner =
            HtmlConstructionTextAppend::begin(storage, node, DocumentString::from("yz"));
        drive_to(&mut owner, Phase::Commit);
        owner
            .next_action()
            .unwrap()
            .unwrap()
            .perform(None)
            .unwrap()
            .accept();
        let mut owner = Some(owner);
        let ready = slots::text_append(&mut owner).unwrap();
        let holder = text_of(&ready.storage, ready.index).unwrap().clone();
        let Err(refusal) = ready.abort() else {
            panic!("a shared backing cannot be truncated without a copy");
        };
        assert!(refusal.reason().is_shared_allocation_aliased());
        let (ready, _) = refusal.into_parts();
        assert_eq!(
            text_of(&ready.storage, ready.index)
                .unwrap()
                .code_units()
                .as_ptr(),
            holder.code_units().as_ptr(),
            "no copy was made"
        );
        assert_eq!(holder.len(), 3, "the other holder keeps what it saw");
        drop(holder);
        let (storage, node, _, discarded) = ready.abort().unwrap().into_parts();
        assert!(discarded.is_none());
        assert_eq!(text(&storage, &node), [u16::from(b'x')]);
        assert_eq!(storage.usage().utf16_units(), 1);
    }

    /// A ready append whose target is no longer a text node cannot commit. The commit refuses and
    /// hands the append back, so the abort returns the storage, and the separate copy of a
    /// preserved append, to the caller; nothing is reported as committed.
    #[test]
    fn commit_refuses_a_target_that_is_no_longer_text_and_hands_the_append_back() {
        for aliased in [false, true] {
            let original = DocumentString::from("x");
            let history = aliased.then(|| original.clone());
            let (storage, node) = make(HtmlNodeKind::Text(original));
            let mut owner =
                HtmlConstructionTextAppend::begin(storage, node, DocumentString::from("y"));
            drive_to(&mut owner, Phase::Commit);
            owner
                .next_action()
                .unwrap()
                .unwrap()
                .perform(None)
                .unwrap()
                .accept();
            let mut owner = Some(owner);
            let mut ready = slots::text_append(&mut owner).unwrap();
            let usage = ready.storage.usage();
            ready.storage.arena.records[ready.index].kind = HtmlNodeKind::Comment("c".into());
            let Err(refusal) = ready.commit() else {
                panic!("a target that is not text cannot commit");
            };
            assert!(refusal.reason().is_operation_not_ready());
            let (_, discarded) = {
                let (ready, _) = refusal.into_parts();
                assert_eq!(ready.storage.usage(), usage, "no usage was written");
                let (storage, _, incoming, discarded) = ready.abort().unwrap().into_parts();
                assert_eq!(incoming, DocumentString::from("y"));
                (storage, discarded)
            };
            assert_eq!(
                discarded.is_some(),
                aliased,
                "a preserved copy is handed back"
            );
            drop(history);
        }
    }
}
