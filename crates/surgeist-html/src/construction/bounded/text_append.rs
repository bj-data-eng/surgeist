//! Bounded append to the actual construction owner's immutable text backing.

use super::*;

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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlConstructionTextAppendWork {
    /// The retained records the action reads or updates.
    pub record_visits: usize,
    /// The provider requests the action makes.
    pub provider_calls: usize,
    /// The bytes the action only reads, without copying them. The text-append steps copy rather
    /// than inspect, so every description reports zero.
    pub inspected_bytes: usize,
    /// The bytes the action copies, including bookkeeping values it moves.
    pub copied_bytes: usize,
    /// The record fields the action writes.
    pub record_writes: usize,
    /// The size in bytes of the largest single layout the action initializes or moves as one unit.
    /// A text copy is split so that no unit exceeds this chunk size.
    pub atomic_layout_bytes: usize,
    /// Conservative allocator relocation bound, separate from explicit copies.
    pub relocation_bound_bytes: usize,
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
    /// The work that the consumer admits for this step, whichever step it is.
    pub fn work(self) -> HtmlConstructionTextAppendWork {
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
/// The steps that follow the backing choice. A step carries what only it needs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step {
    /// Reserves capacity; `before` are the candidate's dimensions at that moment.
    Reserve {
        before: HtmlConstructionTextAppendCapacity,
    },
    /// Copies the original text into the separate backing, from `prefix` units on.
    CopyPrefix {
        prefix: usize,
    },
    Preflight,
    QualifyTail,
    /// Copies the incoming text onto the backing, from `tail` units on.
    CopyTail {
        tail: usize,
    },
    Commit,
    Ready,
}
impl Step {
    /// Whether the candidate backing has been qualified as uniquely owned for the tail writes,
    /// which only the steps after that qualification can follow.
    fn tail_is_exclusive(self) -> bool {
        matches!(self, Self::CopyTail { .. } | Self::Commit | Self::Ready)
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
    /// The append works in a separate copy of the original text.
    Preserved {
        target: Target,
        copy: DocumentString,
        step: Step,
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
/// backing uniquely owned, so a backing that is not (which nothing can make it) is left as is.
fn truncate_tail(storage: &mut HtmlConstructionStorage, index: usize, old_len: usize) {
    if let Some(backing) =
        text_of_mut(storage, index).and_then(|value| Arc::get_mut(value.construction_backing_mut()))
    {
        backing.truncate(old_len);
    }
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
            Step::CopyPrefix { .. } => Phase::CopyPrefix,
            Step::Preflight => Phase::Preflight,
            Step::QualifyTail => Phase::QualifyTail,
            Step::CopyTail { .. } => Phase::CopyTail,
            Step::Commit => Phase::Commit,
            Step::Ready => Phase::Ready,
        };
        match &self.progress {
            Progress::Inspect => Phase::Inspect,
            Progress::Select(_) => Phase::SelectBacking,
            Progress::Initialize(_) => Phase::InitializeBacking,
            Progress::InPlace { step: s, .. } | Progress::Preserved { step: s, .. } => step(*s),
        }
    }
    fn candidate(&self) -> Option<&DocumentString> {
        match &self.progress {
            Progress::Inspect | Progress::Initialize(_) => None,
            Progress::Select(target) | Progress::InPlace { target, .. } => {
                text_of(&self.storage, target.index)
            }
            Progress::Preserved { copy, .. } => Some(copy),
        }
    }
    fn tail_is_exclusive(&self) -> bool {
        match &self.progress {
            Progress::InPlace { step, .. } | Progress::Preserved { step, .. } => {
                step.tail_is_exclusive()
            }
            _ => false,
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
        matches!(
            self.progress,
            Progress::Initialize(_) | Progress::Preserved { .. }
        )
    }
    /// Only before final tail qualification. Reads and reference providers need admission.
    pub fn backing(&self) -> Option<&DocumentString> {
        if self.tail_is_exclusive() {
            None
        } else {
            self.candidate()
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
        let ready = matches!(
            self.progress,
            Progress::InPlace {
                step: Step::Ready,
                ..
            } | Progress::Preserved {
                step: Step::Ready,
                ..
            }
        );
        Ok((!ready).then_some(HtmlConstructionTextAppendAction { owner: self }))
    }
    /// Consumes a ready append. Returns the latched failure, or the not-ready error before the
    /// last action was accepted; either way the append is handed back unchanged.
    pub fn try_ready(
        self,
    ) -> Result<HtmlConstructionTextAppendReady, HtmlConstructionRefusal<Self>> {
        if let Some(error) = &self.failure {
            let error = error.clone();
            return Err(HtmlConstructionRefusal::new(self, error));
        }
        if self.observing {
            return Err(HtmlConstructionRefusal::new(
                self,
                HtmlConstructionErrorKind::OperationNotReady,
            ));
        }
        let Self {
            storage,
            node,
            incoming,
            progress,
            observing,
            failure,
        } = self;
        let (index, usage, backing) = match progress {
            Progress::InPlace {
                target,
                step: Step::Ready,
            } => (
                target.index,
                target.usage,
                ReadyBacking::InPlace {
                    old_len: target.old_len,
                },
            ),
            Progress::Preserved {
                target,
                copy,
                step: Step::Ready,
            } => (target.index, target.usage, ReadyBacking::Preserved(copy)),
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
            index,
            usage,
            backing,
        })
    }
    /// Removes initialized tail from reused native backing before storage escapes.
    /// A distinct pending copy keeps all initialized units for its actual ticket's
    /// stopped retention and disposal. Neither copy nor tail becomes native-visible.
    /// Successful growth is retained and remains visible in actual capacity.
    pub fn abort(mut self) -> HtmlConstructionTextAppendCompletion {
        let discarded = match self.progress {
            Progress::InPlace { target, step } => {
                if step.tail_is_exclusive() {
                    truncate_tail(&mut self.storage, target.index, target.old_len);
                }
                None
            }
            Progress::Preserved { copy, .. } => Some(copy),
            Progress::Inspect | Progress::Select(_) | Progress::Initialize(_) => None,
        };
        HtmlConstructionTextAppendCompletion {
            storage: self.storage,
            node: self.node,
            incoming: self.incoming,
            discarded,
        }
    }
}

impl<'a> HtmlConstructionTextAppendAction<'a> {
    /// The original visible tree, available only before tail mutation begins.
    pub fn storage_view(&self) -> Option<&HtmlConstructionStorage> {
        match self.owner.progress {
            Progress::InPlace {
                step: Step::Preflight,
                ..
            }
            | Progress::Preserved {
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
        let (target, step) = match &owner.progress {
            Progress::Inspect => {
                work.record_visits += 6;
                work.provider_calls += 1;
                work.atomic_layout_bytes = work
                    .atomic_layout_bytes
                    .max(size_of::<HtmlConstructionUsage>());
                return D::Inspect(work);
            }
            Progress::Select(_) => {
                work.record_visits += 3;
                work.provider_calls += 1;
                return D::SelectBacking(work);
            }
            Progress::Initialize(_) => {
                work.provider_calls += 1;
                work.record_writes += 1;
                work.copied_bytes += size_of::<Vec<u16>>();
                work.atomic_layout_bytes = work.atomic_layout_bytes.max(size_of::<Vec<u16>>());
                return D::InitializeBacking(work);
            }
            Progress::InPlace { target, step } | Progress::Preserved { target, step, .. } => {
                (target, *step)
            }
        };
        match step {
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
            Step::CopyPrefix { prefix: offset } | Step::CopyTail { tail: offset } => {
                let remaining = if matches!(step, Step::CopyPrefix { .. }) {
                    target.old_len - offset
                } else {
                    owner.incoming.len() - offset
                };
                let units = remaining.min(CHUNK_UNITS);
                work.record_visits += 3;
                work.provider_calls += 1;
                // The unit count is written into the backing; the progress counter is part of
                // the stored progress.
                work.record_writes += 1;
                work.copied_bytes += units * size_of::<u16>() + size_of::<usize>();
                work.atomic_layout_bytes = work
                    .atomic_layout_bytes
                    .max(units * size_of::<u16>())
                    .max(size_of::<usize>());
                if matches!(step, Step::CopyPrefix { .. }) {
                    D::CopyPrefix {
                        offset,
                        units,
                        work,
                    }
                } else {
                    D::CopyTail {
                        offset,
                        units,
                        work,
                    }
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
                let preserved = matches!(owner.progress, Progress::Preserved { .. });
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
            // A ready append has no pending action; `next_action` never issues one.
            Step::Ready => D::Commit {
                usage: target.usage,
                work,
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
        let result = (|| -> Result<(), HtmlConstructionError> {
            if minimum_capacity.is_some()
                && !matches!(
                    progress,
                    Progress::InPlace {
                        step: Step::Reserve { .. },
                        ..
                    } | Progress::Preserved {
                        step: Step::Reserve { .. },
                        ..
                    }
                )
            {
                return Err(HtmlConstructionErrorKind::CapacityMismatch.into());
            }
            match progress {
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
                    let target = *target;
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
                    let target = *target;
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
                    advance(storage, incoming, *target, step, None, minimum_capacity)?;
                }
                Progress::Preserved { target, copy, step } => {
                    advance(
                        storage,
                        incoming,
                        *target,
                        step,
                        Some(copy),
                        minimum_capacity,
                    )?;
                }
            }
            Ok(())
        })();
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
/// Performs the pending step of a backing that is chosen. `copy` is the separate backing when the
/// append preserves the original, and the candidate is the original text otherwise.
fn advance(
    storage: &mut HtmlConstructionStorage,
    incoming: &DocumentString,
    target: Target,
    step: &mut Step,
    mut copy: Option<&mut DocumentString>,
    minimum_capacity: Option<usize>,
) -> Result<(), HtmlConstructionError> {
    let preserved = copy.is_some();
    match *step {
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
            let candidate = match copy.as_deref_mut() {
                Some(copy) => copy,
                None => text_of_mut(storage, target.index)
                    .ok_or(HtmlConstructionErrorKind::WrongNodeKind)?,
            };
            let backing = Arc::get_mut(candidate.construction_backing_mut())
                .ok_or(HtmlConstructionErrorKind::SharedAllocationAliased)?;
            backing
                .try_reserve_exact(minimum - backing.len())
                .map_err(|_| HtmlConstructionErrorKind::AllocationFailed)?;
            *step = if preserved && target.old_len > 0 {
                Step::CopyPrefix { prefix: 0 }
            } else {
                Step::Preflight
            };
        }
        Step::CopyPrefix { prefix: start } => {
            // Only a preserved copy has a prefix to copy; an in-place append never reaches the
            // step, and its tail qualification below is the only uniqueness it needs.
            let Some(copy) = copy else {
                return Err(HtmlConstructionErrorKind::OperationNotReady.into());
            };
            let end = start + (target.old_len - start).min(CHUNK_UNITS);
            let original =
                text_of(storage, target.index).ok_or(HtmlConstructionErrorKind::WrongNodeKind)?;
            let backing = Arc::get_mut(copy.construction_backing_mut())
                .ok_or(HtmlConstructionErrorKind::SharedAllocationAliased)?;
            backing.extend_from_slice(&original.code_units()[start..end]);
            *step = if end == target.old_len {
                Step::Preflight
            } else {
                Step::CopyPrefix { prefix: end }
            };
        }
        Step::Preflight => *step = Step::QualifyTail,
        Step::QualifyTail => {
            let candidate = match copy.as_deref_mut() {
                Some(copy) => copy,
                None => text_of_mut(storage, target.index)
                    .ok_or(HtmlConstructionErrorKind::WrongNodeKind)?,
            };
            if Arc::get_mut(candidate.construction_backing_mut()).is_none() {
                return Err(HtmlConstructionErrorKind::SharedAllocationAliased.into());
            }
            *step = if incoming.is_empty() {
                Step::Commit
            } else {
                Step::CopyTail { tail: 0 }
            };
        }
        Step::CopyTail { tail: start } => {
            let end = start + (incoming.len() - start).min(CHUNK_UNITS);
            let candidate = match copy.as_deref_mut() {
                Some(copy) => copy,
                None => text_of_mut(storage, target.index)
                    .ok_or(HtmlConstructionErrorKind::WrongNodeKind)?,
            };
            // The tail qualification proved the backing uniquely owned, and nothing runs
            // between it and these writes that could share it again.
            let backing = Arc::get_mut(candidate.construction_backing_mut())
                .ok_or(HtmlConstructionErrorKind::SharedAllocationAliased)?;
            backing.extend_from_slice(&incoming.code_units()[start..end]);
            *step = if end == incoming.len() {
                Step::Commit
            } else {
                Step::CopyTail { tail: end }
            };
        }
        Step::Commit => *step = Step::Ready,
        Step::Ready => return Err(HtmlConstructionErrorKind::OperationNotReady.into()),
    }
    Ok(())
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
    /// for retirement.
    pub fn abort(mut self) -> HtmlConstructionTextAppendCompletion {
        let discarded = match self.backing {
            ReadyBacking::InPlace { old_len } => {
                truncate_tail(&mut self.storage, self.index, old_len);
                None
            }
            ReadyBacking::Preserved(copy) => Some(copy),
        };
        HtmlConstructionTextAppendCompletion {
            storage: self.storage,
            node: self.node,
            incoming: self.incoming,
            discarded,
        }
    }
    /// Only prepaid owning moves and fixed usage writes; no payload copy.
    pub fn commit(mut self) -> HtmlConstructionTextAppendCompletion {
        let discarded = match self.backing {
            ReadyBacking::InPlace { .. } => None,
            ReadyBacking::Preserved(copy) => match text_of_mut(&mut self.storage, self.index) {
                Some(original) => Some(std::mem::replace(original, copy)),
                // The target was qualified as text by the first action and the storage has been
                // exclusively owned since, so this cannot be reached; the copy is then returned
                // for retirement instead of replacing anything.
                None => Some(copy),
            },
        };
        self.storage.arena.usage = self.usage;
        HtmlConstructionTextAppendCompletion {
            storage: self.storage,
            node: self.node,
            incoming: self.incoming,
            discarded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        } = slots::operation(&mut operation).unwrap().commit()
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
            .into_parts();
        (storage, node, incoming, discarded, prefix_units, tail_units)
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
        let (storage, node, incoming, discarded) = owner.abort().into_parts();
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
        let (storage, node, _, pending) = owner.abort().into_parts();
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
                slots::text_append(&mut Some(owner)).unwrap().abort()
            } else {
                owner.abort()
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
            let (storage, node, _, discarded) = owner.abort().into_parts();
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
        let (storage, _, incoming, discarded) = owner.abort().into_parts();
        assert_eq!(storage.usage(), usage);
        assert_eq!(incoming.code_units(), [u16::from(b'x')]);
        assert!(discarded.is_none());
        let mut owner = HtmlConstructionTextAppend::begin(storage, node, incoming);
        assert!(matches!(
            owner.next_action().unwrap().unwrap().perform(None),
            Err(ref error) if error.kind == HtmlConstructionErrorKind::WrongNodeKind
        ));
        assert_eq!(owner.abort().into_parts().0.usage(), usage);
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
            let (storage, node, _, discarded) = owner.abort().into_parts();
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
                slots::text_append(&mut owner).unwrap().abort()
            } else {
                assert!(matches!(
                    slots::text_append(&mut owner),
                    Err(ref error) if error.kind == HtmlConstructionErrorKind::OperationNotReady
                ));
                owner.take().unwrap().abort()
            };
            let (storage, node, _, discarded) = completion.into_parts();
            assert_eq!(text(&storage, &node), [u16::from(b'x')]);
            assert_eq!(storage.usage().utf16_units(), 1);
            assert!(discarded.is_none());
        }
    }
}
