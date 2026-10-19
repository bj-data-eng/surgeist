//! Bounded weak sharing of already immutable HTML values.
//!
//! The concrete front doors share storage policy, not language interpretation.
//! Consumers describe actions under their owner lock, release it to admit work
//! and custody, then perform the owned action. No method calls consumer code.

use super::{Attribute, DocumentString, OrderedAttributes};
use std::{
    error::Error,
    fmt,
    mem::size_of,
    sync::{
        Arc, Weak,
        atomic::{AtomicU64, Ordering},
    },
};

mod policy;
use policy::{Operation, Pool, Value};

/// Known retained payload, excluding opaque Arc and allocator internals.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlValuePoolUsage {
    bytes: usize,
    utf16_units: usize,
}
impl HtmlValuePoolUsage {
    pub fn bytes(self) -> usize {
        self.bytes
    }
    pub fn utf16_units(self) -> usize {
        self.utf16_units
    }
    fn add(self, other: Self) -> Result<Self, HtmlValuePoolError> {
        Ok(Self {
            bytes: self
                .bytes
                .checked_add(other.bytes)
                .ok_or(HtmlValuePoolError::RepresentationExhausted)?,
            utf16_units: self
                .utf16_units
                .checked_add(other.utf16_units)
                .ok_or(HtmlValuePoolError::RepresentationExhausted)?,
        })
    }
}

/// Work admitted before an action reads operands or writes retained storage.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlValuePoolWork {
    record_visits: usize,
    inspected_bytes: usize,
    copied_bytes: usize,
    provider_calls: usize,
    reserve_bytes: usize,
}
impl HtmlValuePoolWork {
    pub fn record_visits(self) -> usize {
        self.record_visits
    }
    pub fn inspected_bytes(self) -> usize {
        self.inspected_bytes
    }
    pub fn copied_bytes(self) -> usize {
        self.copied_bytes
    }
    pub fn provider_calls(self) -> usize {
        self.provider_calls
    }
    pub fn reserve_bytes(self) -> usize {
        self.reserve_bytes
    }
}

/// An action's maximum known live storage during its execution.
///
/// `pool` is an absolute heap extent, including undo and weak target space.
/// `candidate` is a separately retained strong occurrence, never part of the
/// weak pool's steady extent. Input custody remains independently live.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlValuePoolLayout {
    pool: HtmlValuePoolUsage,
    candidate: HtmlValuePoolUsage,
    work: HtmlValuePoolWork,
    closing_work: HtmlValuePoolWork,
}
impl HtmlValuePoolLayout {
    pub fn pool(self) -> HtmlValuePoolUsage {
        self.pool
    }
    pub fn candidate(self) -> HtmlValuePoolUsage {
        self.candidate
    }
    pub fn work(self) -> HtmlValuePoolWork {
        self.work
    }
    /// Prepaid work for storage introduced by this action and for later inverse
    /// restoration or retirement. Charge this before performing the action,
    /// including a Prepare or Upgrade: abandoning its provisional storage or
    /// temporary strong candidate must never require failure-time admission.
    /// Payment remains spent when the value moves into another owner.
    pub fn closing_work(self) -> HtmlValuePoolWork {
        self.closing_work
    }
}

/// Protocol misuse and checked mandatory layout arithmetic.
/// Optional pool/qualifier capacity is reported as bypass, never this error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlValuePoolError {
    ForeignOwner,
    WrongAction,
    NotReady,
    RepresentationExhausted,
}
impl fmt::Display for HtmlValuePoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTML value pool protocol: {self:?}")
    }
}
impl Error for HtmlValuePoolError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Hash,
    Inspect,
    Upgrade,
    Compare,
    Touch,
    Prepare,
    Commit,
    Hit,
    Miss,
    Offered,
    Bypassed,
}

/// Which owner executes the action. Reads need no pool lock; pool actions
/// revalidate qualified storage before accessing or changing it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlValuePoolActionKind {
    Read,
    Pool,
}

/// A fixed-size, non-replayable action. It owns no Weak, strong value or lock.
#[derive(Debug)]
pub struct HtmlValuePoolAction {
    owner: u64,
    session: Option<u64>,
    operation: u64,
    serial: u64,
    slot: usize,
    generation: Option<u64>,
    phase: Phase,
    reads: usize,
    layout: HtmlValuePoolLayout,
}
impl HtmlValuePoolAction {
    pub fn kind(&self) -> HtmlValuePoolActionKind {
        if matches!(self.phase, Phase::Hash | Phase::Compare) {
            HtmlValuePoolActionKind::Read
        } else {
            HtmlValuePoolActionKind::Pool
        }
    }
    pub fn layout(&self) -> HtmlValuePoolLayout {
        self.layout
    }
}

/// The consumer pairs a newly acquired candidate with the reservation made
/// before the upgrade. Release destroys that strong value before its receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlValuePoolProgress {
    Continue,
    CandidateAcquired,
    CandidateReleased,
    Finished,
}

/// Opaque identity of a retained weak occurrence within its concrete pool owner.
/// Equality identifies an occurrence; it grants no payload or mutation authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlValuePoolOccurrence {
    owner: u64,
    slot: usize,
    generation: u64,
}

/// Allocation-free bookkeeping from an actual transition. A moved occurrence
/// keeps its existing allowance. Pool receipts may instead track the absolute
/// `after` extent, without a second per-entry receipt table.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlValuePoolChange {
    before: HtmlValuePoolUsage,
    after: HtmlValuePoolUsage,
    inserted: Option<HtmlValuePoolOccurrence>,
    moved_to_undo: Option<HtmlValuePoolOccurrence>,
    released: usize,
}
impl HtmlValuePoolChange {
    pub fn before(self) -> HtmlValuePoolUsage {
        self.before
    }
    pub fn after(self) -> HtmlValuePoolUsage {
        self.after
    }
    pub fn inserted(self) -> Option<HtmlValuePoolOccurrence> {
        self.inserted
    }
    pub fn moved_to_undo(self) -> Option<HtmlValuePoolOccurrence> {
        self.moved_to_undo
    }
    pub fn released_occurrences(self) -> usize {
        self.released
    }
}

/// Affine root authority. The consumer's owning session guard must abort this
/// root on Drop, or move it into its prepared publication before sealing.
#[derive(Debug)]
#[must_use = "abort the root or move it into a prepared seal"]
pub struct HtmlValuePoolRoot {
    owner: u64,
    id: u64,
}
/// Affine nested boundary. Explicit restore undoes its suffix; the consumer's
/// mark guard accepts it on Drop. Accepting never seals the root.
#[derive(Debug)]
#[must_use = "restore or accept the nested mark"]
pub struct HtmlValuePoolMark {
    owner: u64,
    root: u64,
    id: u64,
}
/// Frozen, already validated root authority. Closing consumes no new qualifier.
#[derive(Debug)]
#[must_use = "seal or abort the frozen root"]
pub struct HtmlValuePoolSeal {
    root: HtmlValuePoolRoot,
}

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
fn owner_id() -> Option<u64> {
    NEXT_OWNER
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .ok()
}

#[derive(Clone, Copy, Debug, Default)]
struct Cursor {
    attribute: usize,
    part: usize,
    offset: usize,
    len: usize,
    units: bool,
    started: bool,
    done: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Word {
    tag: u8,
    value: usize,
}
struct Read {
    word: Word,
    payload: HtmlValuePoolUsage,
}
fn string_payload(value: &DocumentString) -> Result<HtmlValuePoolUsage, HtmlValuePoolError> {
    let capacity = value.shared_code_units().capacity();
    Ok(HtmlValuePoolUsage {
        bytes: capacity
            .checked_mul(size_of::<u16>())
            .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
            .ok_or(HtmlValuePoolError::RepresentationExhausted)?,
        utf16_units: capacity,
    })
}
fn read_string(
    value: &DocumentString,
    cursor: &mut Cursor,
    tag: u8,
) -> Result<Read, HtmlValuePoolError> {
    if !cursor.units {
        cursor.len = value.len();
        cursor.units = cursor.len != 0;
        cursor.offset = 0;
        Ok(Read {
            word: Word {
                tag,
                value: cursor.len,
            },
            payload: string_payload(value)?,
        })
    } else {
        let unit = value.code_units()[cursor.offset];
        cursor.offset += 1;
        if cursor.offset == cursor.len {
            cursor.units = false;
        }
        Ok(Read {
            word: Word {
                tag: 8,
                value: usize::from(unit),
            },
            payload: HtmlValuePoolUsage::default(),
        })
    }
}

impl Value for DocumentString {
    type Item = u16;
    fn shared(&self) -> &Arc<Vec<u16>> {
        self.shared_code_units()
    }
    fn from_shared(value: Arc<Vec<u16>>) -> Self {
        Self::from_shared_code_units(value)
    }
    fn eligible(&self) -> bool {
        self.len() <= policy::MAX_UNITS && self.shared_code_units().capacity() <= policy::MAX_UNITS
    }
    fn candidate_cleanup_work() -> HtmlValuePoolWork {
        // A last strong alias destroys the Vec target and can free its buffer
        // and shared target. u16 has no per-element drop traversal.
        HtmlValuePoolWork {
            record_visits: 1,
            provider_calls: 2,
            ..HtmlValuePoolWork::default()
        }
    }
    fn read(&self, cursor: &mut Cursor) -> Result<Read, HtmlValuePoolError> {
        let result = read_string(self, cursor, 1)?;
        cursor.started = true;
        cursor.done = !cursor.units;
        Ok(result)
    }
}
impl Value for OrderedAttributes {
    type Item = Attribute;
    fn shared(&self) -> &Arc<Vec<Attribute>> {
        self.shared_attributes()
    }
    fn from_shared(value: Arc<Vec<Attribute>>) -> Self {
        Self { attributes: value }
    }
    fn eligible(&self) -> bool {
        self.len() <= policy::MAX_ATTRIBUTES
            && self.shared_attributes().capacity() <= policy::MAX_ATTRIBUTES
    }
    fn candidate_cleanup_work() -> HtmlValuePoolWork {
        // Each initialized Attribute has up to four DocumentString handles:
        // namespace URI, prefix, local name and value. Independently of caller
        // liveness, each can be the last strong alias of a Vec<u16>. The bank
        // itself has one Vec target and buffer. This is a length bound, not a
        // claim that spare capacity or UTF-16 units run element destructors.
        HtmlValuePoolWork {
            record_visits: 1 + policy::MAX_ATTRIBUTES * 5,
            provider_calls: 2 + policy::MAX_ATTRIBUTES * 8,
            ..HtmlValuePoolWork::default()
        }
    }
    fn read(&self, cursor: &mut Cursor) -> Result<Read, HtmlValuePoolError> {
        if !cursor.started {
            cursor.started = true;
            cursor.done = self.is_empty();
            return Ok(Read {
                word: Word {
                    tag: 0,
                    value: self.len(),
                },
                payload: HtmlValuePoolUsage {
                    bytes: self
                        .shared_attributes()
                        .capacity()
                        .checked_mul(size_of::<Attribute>())
                        .and_then(|n| n.checked_add(size_of::<Vec<Attribute>>()))
                        .ok_or(HtmlValuePoolError::RepresentationExhausted)?,
                    utf16_units: 0,
                },
            });
        }
        let attribute = &self.as_slice()[cursor.attribute];
        let string = match cursor.part {
            0 => attribute.name().namespace().uri(),
            1 => attribute.name().prefix(),
            2 => Some(attribute.name().local_name()),
            3 => Some(attribute.value()),
            _ => unreachable!("four components per attribute"),
        };
        let result = if let Some(string) = string {
            let tag = 2 + cursor.part as u8;
            read_string(string, cursor, tag)?
        } else {
            Read {
                word: Word {
                    tag: 9 + cursor.part as u8,
                    value: 0,
                },
                payload: HtmlValuePoolUsage::default(),
            }
        };
        if !cursor.units {
            cursor.part += 1;
            if cursor.part == 4 {
                cursor.part = 0;
                cursor.attribute += 1;
            }
            cursor.done = cursor.attribute == self.len();
        }
        Ok(result)
    }
}

macro_rules! concrete_pool {
    ($pool:ident, $operation:ident, $candidate:ident, $result:ident, $value:ty) => {
        /// Owner-scoped bounded weak sharing. No semantic value depends on this
        /// owner's continued existence. Constructing the empty owner allocates
        /// nothing; consumers account for its inline header in their own owner.
        pub struct $pool {
            inner: Pool<$value>,
        }
        impl Default for $pool {
            fn default() -> Self {
                Self { inner: Pool::new() }
            }
        }
        impl $pool {
            pub fn new() -> Self {
                Self::default()
            }
            /// Actual inline owner layout, independent of dynamic pool storage.
            pub fn inline_bytes() -> usize {
                size_of::<Self>()
            }
            /// Fixed upper bound for destroying this whole owner in any state.
            /// Admit once BEFORE constructing it, including an empty owner.
            /// This covers live/old Weak entries, initialized pages and undo
            /// records, and backing release, even after sealing. It is separate
            /// from root metadata and each action's temporary/inverse work.
            /// Drop invokes no admission callback and issues no qualifier.
            pub fn disposal_work() -> HtmlValuePoolWork {
                policy::disposal_work()
            }
            /// Admit before beginning root authority; includes its fixed close.
            pub fn root_work() -> HtmlValuePoolWork {
                policy::root_work::<$value>()
            }
            /// Admit before issuing a nested mark; includes accept or restore's
            /// fixed metadata work. Inverse work is paid by mutation layouts.
            pub fn mark_work() -> HtmlValuePoolWork {
                policy::mark_work::<$value>()
            }
            /// Admit before each attempt to freeze a root, including a retry
            /// after unresolved marks refused preparation. This pays the
            /// bounded readiness/metadata checks, not another root copy.
            /// Root construction already prepays fixed closing/header moves;
            /// mutation layouts prepay inverse and backing cleanup. Validation
            /// and closing afterward need no further admission callback.
            pub fn seal_work() -> HtmlValuePoolWork {
                policy::seal_work()
            }
            /// O(1) admitted heap extent, including undo and conservative
            /// per-Weak target space. This is not distinct physical allocation
            /// counting; multiple occurrences can refer to the same allocation.
            pub fn heap_usage(&self) -> HtmlValuePoolUsage {
                self.inner.usage()
            }
            pub fn probe<'a>(&mut self, input: &'a $value) -> $operation<'a> {
                $operation {
                    inner: self.inner.operation(input, false),
                }
            }
            pub fn offer<'a>(&mut self, input: &'a $value) -> $operation<'a> {
                $operation {
                    inner: self.inner.operation(input, true),
                }
            }
            /// Promotes this owner's completed miss for the identical still-
            /// borrowed immutable input. No fingerprint or candidate scan is
            /// repeated. Call before consuming the miss with `try_finish`.
            /// Supersession/checkpoint changes bypass without mutation; an
            /// completed non-miss from this owner returns false without
            /// consuming its result or candidate, even after supersession.
            /// An unfinished operation returns `NotReady`.
            pub fn offer_miss(
                &mut self,
                operation: &mut $operation<'_>,
            ) -> Result<bool, HtmlValuePoolError> {
                self.inner.offer_miss(&mut operation.inner)
            }
            /// Describes without retaining a candidate or calling consumer code.
            pub fn next_action(
                &mut self,
                operation: &mut $operation<'_>,
            ) -> Result<Option<HtmlValuePoolAction>, HtmlValuePoolError> {
                self.inner.next(&mut operation.inner)
            }
            /// Performs only a preadmitted pool action. Read actions instead use
            /// Operation::read outside the consumer's pool mutex.
            pub fn perform(
                &mut self,
                operation: &mut $operation<'_>,
                action: HtmlValuePoolAction,
            ) -> Result<HtmlValuePoolProgress, HtmlValuePoolError> {
                self.inner.perform(&mut operation.inner, action)
            }
            pub fn begin_root(&mut self) -> Option<HtmlValuePoolRoot> {
                self.inner.begin_root()
            }
            pub fn mark(
                &mut self,
                root: &HtmlValuePoolRoot,
            ) -> Result<Option<HtmlValuePoolMark>, HtmlValuePoolError> {
                self.inner.mark(root)
            }
            /// One-way suspension for a paired consumer whose other pool could
            /// not acquire a nested mark. Existing inverse authority survives;
            /// no later mutation is admitted until this root closes.
            pub fn suspend_mutations(
                &mut self,
                root: &HtmlValuePoolRoot,
            ) -> Result<(), HtmlValuePoolError> {
                self.inner.suspend(root)
            }
            pub fn accept_mark(
                &mut self,
                mark: HtmlValuePoolMark,
            ) -> Result<(), HtmlValuePoolError> {
                self.inner.accept(mark)
            }
            pub fn restore_mark(
                &mut self,
                mark: HtmlValuePoolMark,
            ) -> Result<HtmlValuePoolChange, HtmlValuePoolError> {
                self.inner.restore(mark)
            }
            pub fn abort_root(
                &mut self,
                root: HtmlValuePoolRoot,
            ) -> Result<HtmlValuePoolChange, HtmlValuePoolError> {
                self.inner.abort(root)
            }
            pub fn prepare_seal(
                &mut self,
                root: HtmlValuePoolRoot,
            ) -> Result<HtmlValuePoolSeal, (HtmlValuePoolError, HtmlValuePoolRoot)> {
                self.inner.prepare_seal(root)
            }
            /// Pair adapters validate both plans before either seal. A valid
            /// frozen plan needs no allocation, fresh ID or fallible admission.
            pub fn validate_seal(
                &self,
                seal: &HtmlValuePoolSeal,
            ) -> Result<(), HtmlValuePoolError> {
                self.inner.validate_seal(seal)
            }
            pub fn seal(
                &mut self,
                seal: HtmlValuePoolSeal,
            ) -> Result<HtmlValuePoolChange, HtmlValuePoolError> {
                self.inner.seal(seal)
            }
            pub fn abort_seal(
                &mut self,
                seal: HtmlValuePoolSeal,
            ) -> Result<HtmlValuePoolChange, HtmlValuePoolError> {
                self.inner.abort(seal.root)
            }
        }
        /// A caller-input-borrowing operation, independent of the pool lock.
        /// On refusal, drop this owner before its provisional/candidate receipts.
        pub struct $operation<'a> {
            inner: Operation<'a, $value>,
        }
        impl $operation<'_> {
            pub fn read(
                &mut self,
                action: HtmlValuePoolAction,
            ) -> Result<HtmlValuePoolProgress, HtmlValuePoolError> {
                self.inner.read(action)
            }
            /// Most recent insertion or release bookkeeping; no allocation.
            pub fn change(&self) -> HtmlValuePoolChange {
                self.inner.change
            }
            /// Completes without cloning backing. A hit transfers its existing
            /// temporary receipt with Candidate::into_value in the consumer.
            pub fn try_finish(owner: &mut Option<Self>) -> Result<$result, HtmlValuePoolError> {
                let phase = owner
                    .as_ref()
                    .ok_or(HtmlValuePoolError::NotReady)?
                    .inner
                    .phase;
                if !matches!(
                    phase,
                    Phase::Hit | Phase::Miss | Phase::Offered | Phase::Bypassed
                ) {
                    return Err(HtmlValuePoolError::NotReady);
                }
                let operation = owner.take().expect("occupied completed operation");
                match phase {
                    Phase::Hit => Ok($result::Hit($candidate {
                        value: operation
                            .inner
                            .candidate
                            .expect("completed hit retains candidate"),
                    })),
                    Phase::Miss => Ok($result::Miss),
                    Phase::Offered => Ok($result::Offered(operation.inner.change)),
                    Phase::Bypassed => Ok($result::Bypassed),
                    _ => unreachable!("completed operation checked"),
                }
            }
        }
        /// A strongly held exact match. Its consumer-side receipt must remain
        /// live until this value is destroyed or transferred into guarded output.
        #[derive(Debug)]
        pub struct $candidate {
            value: $value,
        }
        impl $candidate {
            pub fn value(&self) -> &$value {
                &self.value
            }
            pub fn into_value(self) -> $value {
                self.value
            }
        }
        #[derive(Debug)]
        pub enum $result {
            Hit($candidate),
            Miss,
            Offered(HtmlValuePoolChange),
            Bypassed,
        }
    };
}
concrete_pool!(
    HtmlStringPool,
    HtmlStringPoolOperation,
    HtmlStringPoolCandidate,
    HtmlStringPoolResult,
    DocumentString
);
concrete_pool!(
    HtmlAttributePool,
    HtmlAttributePoolOperation,
    HtmlAttributePoolCandidate,
    HtmlAttributePoolResult,
    OrderedAttributes
);
