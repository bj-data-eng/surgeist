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
    pub fn initialized_len(self) -> usize {
        self.initialized_len
    }
    pub fn capacity(self) -> usize {
        self.capacity
    }
    pub fn data_address(self) -> usize {
        self.data_address
    }
    pub fn element_size(self) -> usize {
        size_of::<u16>()
    }
    pub fn element_alignment(self) -> usize {
        align_of::<u16>()
    }
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
    pub record_visits: usize,
    pub provider_calls: usize,
    pub inspected_bytes: usize,
    pub copied_bytes: usize,
    pub record_writes: usize,
    pub atomic_layout_bytes: usize,
    /// Conservative allocator relocation bound, separate from explicit copies.
    pub relocation_bound_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlConstructionTextAppendDescription {
    Inspect(HtmlConstructionTextAppendWork),
    SelectBacking(HtmlConstructionTextAppendWork),
    InitializeBacking(HtmlConstructionTextAppendWork),
    Reserve {
        before: HtmlConstructionTextAppendCapacity,
        required_capacity: usize,
        preferred_capacity: usize,
        work: HtmlConstructionTextAppendWork,
    },
    CopyPrefix {
        offset: usize,
        units: usize,
        work: HtmlConstructionTextAppendWork,
    },
    /// The only action lending the original construction storage.
    Preflight {
        usage: HtmlConstructionUsage,
        work: HtmlConstructionTextAppendWork,
    },
    QualifyTail(HtmlConstructionTextAppendWork),
    CopyTail {
        offset: usize,
        units: usize,
        work: HtmlConstructionTextAppendWork,
    },
    Commit {
        usage: HtmlConstructionUsage,
        work: HtmlConstructionTextAppendWork,
    },
}
impl HtmlConstructionTextAppendDescription {
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
    Observing,
    Ready,
    Failed,
}

/// Exclusive pending owner. No mutable backing or reusable uniqueness proof escapes.
pub struct HtmlConstructionTextAppend {
    storage: HtmlConstructionStorage,
    node: HtmlBuilderNode,
    incoming: DocumentString,
    pending: Option<DocumentString>,
    phase: Phase,
    next: Option<Phase>,
    failure: Option<HtmlConstructionError>,
    target: Option<usize>,
    old_len: usize,
    required: usize,
    usage: HtmlConstructionUsage,
    preserve: bool,
    exclusive_tail: bool,
    prefix: usize,
    tail: usize,
}
pub struct HtmlConstructionTextAppendAction<'a> {
    owner: &'a mut HtmlConstructionTextAppend,
}
pub struct HtmlConstructionTextAppendObservation<'a> {
    owner: &'a mut HtmlConstructionTextAppend,
}
pub struct HtmlConstructionTextAppendReady {
    owner: HtmlConstructionTextAppend,
}

/// Original operands and actual storage, plus a distinct backing needing retirement.
/// On abort, a pending preserved copy retains all of its initialized units.
pub struct HtmlConstructionTextAppendCompletion {
    storage: HtmlConstructionStorage,
    node: HtmlBuilderNode,
    incoming: DocumentString,
    discarded: Option<DocumentString>,
}
impl HtmlConstructionTextAppendCompletion {
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
            pending: None,
            phase: Phase::Inspect,
            next: None,
            failure: None,
            target: None,
            old_len: 0,
            required: 0,
            usage: HtmlConstructionUsage::default(),
            preserve: false,
            exclusive_tail: false,
            prefix: 0,
            tail: 0,
        }
    }
    fn original(&self) -> Option<&DocumentString> {
        match &self.storage.arena.records.get(self.target?)?.kind {
            HtmlNodeKind::Text(value) => Some(value),
            _ => None,
        }
    }
    fn original_mut(&mut self) -> &mut DocumentString {
        match &mut self.storage.arena.records[self.target.expect("qualified target")].kind {
            HtmlNodeKind::Text(value) => value,
            _ => unreachable!("qualified text"),
        }
    }
    fn candidate(&self) -> Option<&DocumentString> {
        if self.preserve {
            self.pending.as_ref()
        } else {
            self.original()
        }
    }
    fn candidate_mut(&mut self) -> &mut DocumentString {
        if self.preserve {
            self.pending.as_mut().expect("initialized pending backing")
        } else {
            self.original_mut()
        }
    }
    pub fn capacity(&self) -> Option<HtmlConstructionTextAppendCapacity> {
        self.candidate().map(HtmlConstructionTextAppendCapacity::of)
    }
    /// True when immutable original aliases require a separate aggregate.
    pub fn preserves_original(&self) -> bool {
        self.preserve
    }
    /// Only before final tail qualification. Reads and reference providers need admission.
    pub fn backing(&self) -> Option<&DocumentString> {
        if self.exclusive_tail {
            None
        } else {
            self.candidate()
        }
    }
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionTextAppendAction<'_>>, HtmlConstructionError> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        if self.phase == Phase::Observing {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        Ok(
            (self.phase != Phase::Ready)
                .then_some(HtmlConstructionTextAppendAction { owner: self }),
        )
    }
    pub fn try_ready(
        owner: &mut Option<Self>,
    ) -> Result<HtmlConstructionTextAppendReady, HtmlConstructionError> {
        let append = owner
            .as_ref()
            .ok_or(HtmlConstructionError::OperationConsumed)?;
        if let Some(error) = &append.failure {
            return Err(error.clone());
        }
        if append.phase != Phase::Ready {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        Ok(HtmlConstructionTextAppendReady {
            owner: owner.take().expect("ready append"),
        })
    }
    /// Removes initialized tail from reused native backing before storage escapes.
    /// A distinct pending copy keeps all initialized units for its actual ticket's
    /// stopped retention and disposal. Neither copy nor tail becomes native-visible.
    /// Successful growth is retained and remains visible in actual capacity.
    pub fn abort(mut self) -> HtmlConstructionTextAppendCompletion {
        if self.exclusive_tail && !self.preserve {
            let old_len = self.old_len;
            Arc::get_mut(self.candidate_mut().construction_backing_mut())
                .expect("exclusive tail has no escaping references")
                .truncate(old_len);
        }
        HtmlConstructionTextAppendCompletion {
            storage: self.storage,
            node: self.node,
            incoming: self.incoming,
            discarded: self.pending,
        }
    }
    fn after_reserve(&self) -> Phase {
        if self.preserve && self.prefix < self.old_len {
            Phase::CopyPrefix
        } else {
            Phase::Preflight
        }
    }
}

impl<'a> HtmlConstructionTextAppendAction<'a> {
    /// The original visible tree, available only before tail mutation begins.
    pub fn storage_view(&self) -> Option<&HtmlConstructionStorage> {
        (self.owner.phase == Phase::Preflight).then_some(&self.owner.storage)
    }
    /// Actual reuse or preserved-copy candidate at the pre-tail boundary.
    pub fn backing(&self) -> Option<&DocumentString> {
        if self.owner.phase == Phase::Preflight {
            self.owner.backing()
        } else {
            None
        }
    }
    pub fn description(&self) -> HtmlConstructionTextAppendDescription {
        use HtmlConstructionTextAppendDescription as D;
        let owner = &self.owner;
        // Every action performs and accepts a phase transition, or latches its
        // first failure. Include both concrete paths rather than assuming enum
        // and Option<enum> have identical layouts.
        let mut work = HtmlConstructionTextAppendWork {
            record_writes: 4,
            copied_bytes: (2 * size_of::<Phase>() + 2 * size_of::<Option<Phase>>())
                .max(size_of::<Option<HtmlConstructionError>>() + size_of::<Phase>()),
            atomic_layout_bytes: size_of::<Option<HtmlConstructionError>>()
                .max(size_of::<Phase>())
                .max(size_of::<Option<Phase>>()),
            ..Default::default()
        };
        match owner.phase {
            Phase::Inspect => {
                work.record_visits += 6;
                work.provider_calls += 1;
                work.record_writes += 4;
                work.copied_bytes += size_of::<Option<usize>>()
                    + 2 * size_of::<usize>()
                    + size_of::<HtmlConstructionUsage>();
                work.atomic_layout_bytes = work
                    .atomic_layout_bytes
                    .max(size_of::<HtmlConstructionUsage>());
                D::Inspect(work)
            }
            Phase::SelectBacking => {
                work.record_visits += 3;
                work.provider_calls += 1;
                work.record_writes += 1;
                work.copied_bytes += size_of::<bool>();
                D::SelectBacking(work)
            }
            Phase::InitializeBacking => {
                work.provider_calls += 1;
                work.record_writes += 1;
                work.copied_bytes += size_of::<Option<DocumentString>>() + size_of::<Vec<u16>>();
                work.atomic_layout_bytes = work
                    .atomic_layout_bytes
                    .max(size_of::<Option<DocumentString>>())
                    .max(size_of::<Vec<u16>>());
                D::InitializeBacking(work)
            }
            Phase::Reserve => {
                let before = owner.capacity().expect("actual candidate");
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
                    .max(owner.required)
                    .min(max_units);
                D::Reserve {
                    before,
                    required_capacity: owner.required,
                    preferred_capacity,
                    work,
                }
            }
            Phase::CopyPrefix | Phase::CopyTail => {
                let (offset, remaining) = if owner.phase == Phase::CopyPrefix {
                    (owner.prefix, owner.old_len - owner.prefix)
                } else {
                    (owner.tail, owner.incoming.len() - owner.tail)
                };
                let units = remaining.min(CHUNK_UNITS);
                work.record_visits += 3;
                work.provider_calls += 1;
                work.record_writes += 2;
                work.copied_bytes += units * size_of::<u16>() + 2 * size_of::<usize>();
                work.atomic_layout_bytes = work
                    .atomic_layout_bytes
                    .max(units * size_of::<u16>())
                    .max(size_of::<usize>());
                if owner.phase == Phase::CopyPrefix {
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
            Phase::Preflight => D::Preflight {
                usage: owner.usage,
                work,
            },
            Phase::QualifyTail => {
                work.record_visits += 3;
                work.provider_calls += 1;
                work.record_writes += 1;
                work.copied_bytes += size_of::<bool>();
                D::QualifyTail(work)
            }
            Phase::Commit => {
                // These writes are prepaid here and executed only by Ready::commit.
                work.record_writes += if owner.preserve { 3 } else { 1 };
                work.copied_bytes += size_of::<HtmlConstructionUsage>();
                work.atomic_layout_bytes = work
                    .atomic_layout_bytes
                    .max(size_of::<HtmlConstructionUsage>());
                if owner.preserve {
                    work.copied_bytes +=
                        2 * size_of::<DocumentString>() + size_of::<Option<DocumentString>>();
                    work.atomic_layout_bytes = work
                        .atomic_layout_bytes
                        .max(size_of::<Option<DocumentString>>());
                }
                D::Commit {
                    usage: owner.usage,
                    work,
                }
            }
            _ => unreachable!("only pending actions are issued"),
        }
    }
    /// Reserve alone accepts a clipped capacity between required and preferred.
    /// Caller admission precedes this action; observation precedes its next poll.
    pub fn perform(
        self,
        minimum_capacity: Option<usize>,
    ) -> Result<HtmlConstructionTextAppendObservation<'a>, HtmlConstructionError> {
        let owner = self.owner;
        let result = (|| {
            if owner.phase != Phase::Reserve && minimum_capacity.is_some() {
                return Err(HtmlConstructionError::CapacityMismatch);
            }
            Ok(match owner.phase {
                Phase::Inspect => {
                    let target = owner.storage.position(&owner.node)?;
                    let HtmlNodeKind::Text(value) = &owner.storage.arena.records[target].kind
                    else {
                        return Err(HtmlConstructionError::WrongNodeKind);
                    };
                    let old_len = value.len();
                    let required = old_len
                        .checked_add(owner.incoming.len())
                        .ok_or(HtmlConstructionError::RepresentationExhausted)?;
                    if required > isize::MAX as usize / size_of::<u16>() {
                        return Err(HtmlConstructionError::RepresentationExhausted);
                    }
                    let mut usage = owner.storage.usage();
                    usage.utf16_units = usage
                        .utf16_units
                        .checked_add(owner.incoming.len())
                        .ok_or(HtmlConstructionError::RepresentationExhausted)?;
                    owner.target = Some(target);
                    owner.old_len = old_len;
                    owner.required = required;
                    owner.usage = usage;
                    Phase::SelectBacking
                }
                Phase::SelectBacking => {
                    owner.preserve =
                        Arc::get_mut(owner.original_mut().construction_backing_mut()).is_none();
                    if owner.preserve {
                        Phase::InitializeBacking
                    } else if owner.capacity().expect("candidate").capacity < owner.required {
                        Phase::Reserve
                    } else {
                        Phase::Preflight
                    }
                }
                Phase::InitializeBacking => {
                    owner.pending = Some(DocumentString::from_utf16(Vec::new()));
                    if owner.required == 0 {
                        Phase::Preflight
                    } else {
                        Phase::Reserve
                    }
                }
                Phase::Reserve => {
                    let before = owner.capacity().expect("actual candidate");
                    let preferred = before
                        .capacity
                        .saturating_mul(2)
                        .max(owner.required)
                        .min(isize::MAX as usize / size_of::<u16>());
                    let minimum = minimum_capacity.unwrap_or(preferred);
                    if minimum < owner.required || minimum > preferred {
                        return Err(HtmlConstructionError::CapacityMismatch);
                    }
                    let backing = Arc::get_mut(owner.candidate_mut().construction_backing_mut())
                        .ok_or(HtmlConstructionError::SharedAllocationAliased)?;
                    backing
                        .try_reserve_exact(minimum - backing.len())
                        .map_err(|_| HtmlConstructionError::AllocationFailed)?;
                    owner.after_reserve()
                }
                Phase::CopyPrefix => {
                    let start = owner.prefix;
                    let end = start + (owner.old_len - start).min(CHUNK_UNITS);
                    let original =
                        match &owner.storage.arena.records[owner.target.expect("target")].kind {
                            HtmlNodeKind::Text(value) => value,
                            _ => unreachable!("qualified text"),
                        };
                    let backing = Arc::get_mut(
                        owner
                            .pending
                            .as_mut()
                            .expect("pending copy")
                            .construction_backing_mut(),
                    )
                    .ok_or(HtmlConstructionError::SharedAllocationAliased)?;
                    backing.extend_from_slice(&original.code_units()[start..end]);
                    owner.prefix = end;
                    if end == owner.old_len {
                        Phase::Preflight
                    } else {
                        Phase::CopyPrefix
                    }
                }
                Phase::Preflight => Phase::QualifyTail,
                Phase::QualifyTail => {
                    if Arc::get_mut(owner.candidate_mut().construction_backing_mut()).is_none() {
                        return Err(HtmlConstructionError::SharedAllocationAliased);
                    }
                    owner.exclusive_tail = true;
                    if owner.incoming.is_empty() {
                        Phase::Commit
                    } else {
                        Phase::CopyTail
                    }
                }
                Phase::CopyTail => {
                    let start = owner.tail;
                    let end = start + (owner.incoming.len() - start).min(CHUNK_UNITS);
                    let source = &owner.incoming;
                    let candidate = if owner.preserve {
                        owner.pending.as_mut().expect("pending copy")
                    } else {
                        match &mut owner.storage.arena.records[owner.target.expect("target")].kind {
                            HtmlNodeKind::Text(value) => value,
                            _ => unreachable!("qualified text"),
                        }
                    };
                    Arc::get_mut(candidate.construction_backing_mut())
                        .expect("exclusive tail")
                        .extend_from_slice(&source.code_units()[start..end]);
                    owner.tail = end;
                    if end == owner.incoming.len() {
                        Phase::Commit
                    } else {
                        Phase::CopyTail
                    }
                }
                Phase::Commit => Phase::Ready,
                _ => return Err(HtmlConstructionError::OperationNotReady),
            })
        })();
        match result {
            Ok(next) => {
                owner.next = Some(next);
                owner.phase = Phase::Observing;
                Ok(HtmlConstructionTextAppendObservation { owner })
            }
            Err(error) => {
                owner.failure = Some(error.clone());
                owner.phase = Phase::Failed;
                Err(error)
            }
        }
    }
}
impl HtmlConstructionTextAppendObservation<'_> {
    pub fn capacity(&self) -> Option<HtmlConstructionTextAppendCapacity> {
        self.owner.capacity()
    }
    pub fn preserves_original(&self) -> bool {
        self.owner.preserves_original()
    }
    /// Before tail qualification only. Suspend newly issued Weak before resuming.
    pub fn backing(&self) -> Option<&DocumentString> {
        self.owner.backing()
    }
    pub fn accept(self) {
        self.owner.phase = self.owner.next.take().expect("pending observation");
    }
}
impl HtmlConstructionTextAppendReady {
    pub fn abort(self) -> HtmlConstructionTextAppendCompletion {
        self.owner.abort()
    }
    /// Only prepaid owning moves and fixed usage writes; no payload copy.
    pub fn commit(mut self) -> HtmlConstructionTextAppendCompletion {
        let owner = &mut self.owner;
        let discarded = if owner.preserve {
            let pending = owner.pending.take().expect("completed preserved copy");
            Some(std::mem::replace(owner.original_mut(), pending))
        } else {
            None
        };
        owner.storage.arena.usage = owner.usage;
        HtmlConstructionTextAppendCompletion {
            storage: self.owner.storage,
            node: self.owner.node,
            incoming: self.owner.incoming,
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
        let storage = HtmlConstructionInitialization::try_ready(&mut initialization)
            .unwrap()
            .commit();
        let mut operation = Some(
            storage
                .prepare(HtmlConstructionCommand::Create { kind })
                .unwrap_or_else(|_| panic!("new construction operation")),
        );
        while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        let HtmlConstructionCommit::Changed {
            storage,
            created: Some(node),
            ..
        } = HtmlConstructionOperation::try_ready(&mut operation)
            .unwrap()
            .commit()
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
        let (storage, node, incoming, discarded) =
            HtmlConstructionTextAppend::try_ready(&mut owner)
                .unwrap()
                .commit()
                .into_parts();
        (storage, node, incoming, discarded, prefix_units, tail_units)
    }
    fn drive_to(owner: &mut HtmlConstructionTextAppend, phase: Phase) {
        while owner.phase != phase {
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
            Err(HtmlConstructionError::OperationNotReady)
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
                HtmlConstructionTextAppend::try_ready(&mut Some(owner))
                    .unwrap()
                    .abort()
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
        let (storage, node, _, _) = HtmlConstructionTextAppend::try_ready(&mut Some(owner))
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
                Err(HtmlConstructionError::SharedAllocationAliased)
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
            Err(HtmlConstructionError::ForeignBuilder)
        ));
        let (storage, _, incoming, discarded) = owner.abort().into_parts();
        assert_eq!(storage.usage(), usage);
        assert_eq!(incoming.code_units(), [u16::from(b'x')]);
        assert!(discarded.is_none());
        let mut owner = HtmlConstructionTextAppend::begin(storage, node, incoming);
        assert!(matches!(
            owner.next_action().unwrap().unwrap().perform(None),
            Err(HtmlConstructionError::WrongNodeKind)
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
                    Err(HtmlConstructionError::CapacityMismatch)
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
                HtmlConstructionTextAppend::try_ready(&mut owner)
                    .unwrap()
                    .abort()
            } else {
                assert!(matches!(
                    HtmlConstructionTextAppend::try_ready(&mut owner),
                    Err(HtmlConstructionError::OperationNotReady)
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
