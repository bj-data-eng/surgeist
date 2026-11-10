//! Bounded weak sharing of already immutable HTML values.
//!
//! The concrete front doors share storage policy, not language interpretation.
//! Consumers describe actions under their owner lock, release it to admit work
//! and custody, then perform the owned action. No method calls consumer code.

use super::{Attribute, DocumentString, OrderedAttributes};
use crate::internal::accounting::{HtmlResource, HtmlWork, wide};
use std::{
    error::Error,
    fmt,
    mem::size_of,
    sync::{
        Arc, Weak,
        atomic::{AtomicU64, Ordering},
    },
};
use surgeist_ledger::{Change, Charge};

pub(crate) mod policy;
use policy::{Operation, Pool, State, Value};

/// Known retained payload, excluding opaque Arc and allocator internals.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlValuePoolUsage {
    bytes: usize,
    utf16_units: usize,
}
impl HtmlValuePoolUsage {
    /// Known retained bytes, excluding `Arc` and allocator internals. For a candidate value it is
    /// the payload capacity plus the inline vector header. For the pool's own `heap_usage` it is
    /// the page slots, undo cells, directory and weak target `Vec` headers, which carry no
    /// payload, so `utf16_units` is 0 there.
    pub fn bytes(self) -> usize {
        self.bytes
    }
    /// Known retained UTF-16 units of string capacity, counted by capacity rather than by
    /// initialized length.
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
pub(crate) struct HtmlValuePoolWork {
    record_visits: usize,
    inspected_bytes: usize,
    copied_bytes: usize,
    provider_calls: usize,
    reserve_bytes: usize,
}
impl HtmlValuePoolWork {
    /// The retained entries, pages or records the action visits.
    #[cfg(test)]
    pub(crate) fn record_visits(self) -> usize {
        self.record_visits
    }
    /// The bytes of operand or entry text the action hashes or compares without copying.
    #[cfg(test)]
    pub(crate) fn inspected_bytes(self) -> usize {
        self.inspected_bytes
    }
    /// The bytes the action copies, including bookkeeping values it moves.
    #[cfg(test)]
    pub(crate) fn copied_bytes(self) -> usize {
        self.copied_bytes
    }
    /// The provider requests the action makes.
    #[cfg(test)]
    pub(crate) fn provider_calls(self) -> usize {
        self.provider_calls
    }
    /// The bytes of storage the action reserves.
    #[cfg(test)]
    pub(crate) fn reserve_bytes(self) -> usize {
        self.reserve_bytes
    }
    /// Charges the five dimensions as one batch, in a fixed order. An amount that
    /// does not fit the ledger's `u64` saturates, and every finite cap refuses it.
    pub(crate) fn charge<C: Charge<HtmlWork, HtmlResource>>(
        self,
        charge: &mut C,
    ) -> Result<(), C::Stop> {
        charge.charge(&[
            Change::Work(HtmlWork::RecordVisit, wide(self.record_visits)),
            Change::Work(HtmlWork::InspectByte, wide(self.inspected_bytes)),
            Change::Work(HtmlWork::CopyByte, wide(self.copied_bytes)),
            Change::Work(HtmlWork::ProviderCall, wide(self.provider_calls)),
            Change::Work(HtmlWork::ReserveByte, wide(self.reserve_bytes)),
        ])
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
    /// The absolute heap extent of the pool at the action's peak, including undo and weak target
    /// space.
    pub fn pool(self) -> HtmlValuePoolUsage {
        self.pool
    }
    /// The extent of the strongly held candidate occurrence at the action's peak. It is separate
    /// from the pool's steady extent.
    pub fn candidate(self) -> HtmlValuePoolUsage {
        self.candidate
    }
    /// The work the consumer admits before performing the action.
    #[cfg(test)]
    pub(crate) fn work(self) -> HtmlValuePoolWork {
        self.work
    }
    /// Prepaid work for storage introduced by this action and for later inverse
    /// restoration or retirement. Charge this before performing the action,
    /// including a Prepare or Upgrade: abandoning its provisional storage or
    /// temporary strong candidate must never require failure-time admission.
    /// Payment remains spent when the value moves into another owner.
    #[cfg(test)]
    pub(crate) fn closing_work(self) -> HtmlValuePoolWork {
        self.closing_work
    }
    /// Charges the work of the action and then its prepaid closing work, each as one
    /// batch of the five pool work dimensions, before the action is performed. The
    /// closing work is charged including for a Prepare or Upgrade, so abandoning its
    /// provisional storage or temporary strong candidate never needs failure-time
    /// admission. Payment remains spent when the value moves into another owner.
    pub fn charge<C: Charge<HtmlWork, HtmlResource>>(self, charge: &mut C) -> Result<(), C::Stop> {
        self.charge_work(charge)?;
        self.charge_closing_work(charge)
    }
    /// Charges only the work of the action, as one batch.
    pub fn charge_work<C: Charge<HtmlWork, HtmlResource>>(
        self,
        charge: &mut C,
    ) -> Result<(), C::Stop> {
        self.work.charge(charge)
    }
    /// Charges only the prepaid closing work, as one batch.
    pub fn charge_closing_work<C: Charge<HtmlWork, HtmlResource>>(
        self,
        charge: &mut C,
    ) -> Result<(), C::Stop> {
        self.closing_work.charge(charge)
    }
}

/// Protocol misuse and checked mandatory layout arithmetic.
/// Optional pool/qualifier capacity is reported as bypass, never this error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlValuePoolError {
    /// A root, mark, seal, operation or action came from a different pool owner.
    ForeignOwner,
    /// The action is not for this operation's pending step: it belongs to another operation, its
    /// serial or phase does not match, or it was passed to the wrong entry point (a pool action
    /// to `read`, or a read action to the pool). A stale action with an older serial is not
    /// this error: the operation bypasses sharing and finishes.
    WrongAction,
    /// The operation or root is not in the required state: an operation has an unfinished step, a
    /// root has an outstanding mark or is frozen, or the root or seal is stale.
    NotReady,
    /// Layout arithmetic overflowed on this platform.
    RepresentationExhausted,
}
impl fmt::Display for HtmlValuePoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ForeignOwner => "value pool action belongs to a different pool owner",
            Self::WrongAction => "action does not match the pending value pool operation",
            Self::NotReady => "value pool operation has an unfinished step",
            Self::RepresentationExhausted => {
                "value pool accounting size cannot be represented on this platform"
            }
        })
    }
}
impl Error for HtmlValuePoolError {}

/// A refused seal preparation. The root is returned unfrozen and still open, so
/// the caller can abort it or resolve the cause and prepare it again.
#[derive(Debug)]
#[must_use = "abort the returned root or prepare it again"]
pub struct HtmlValuePoolSealRefusal {
    error: HtmlValuePoolError,
    root: HtmlValuePoolRoot,
}

impl HtmlValuePoolSealRefusal {
    pub(super) fn new(error: HtmlValuePoolError, root: HtmlValuePoolRoot) -> Self {
        Self { error, root }
    }
    /// The protocol misuse that refused the seal.
    pub fn error(&self) -> HtmlValuePoolError {
        self.error
    }
    /// Returns the refusal cause and the unfrozen root.
    pub fn into_parts(self) -> (HtmlValuePoolError, HtmlValuePoolRoot) {
        (self.error, self.root)
    }
}

impl fmt::Display for HtmlValuePoolSealRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value pool seal refused: {}", self.error)
    }
}
impl Error for HtmlValuePoolSealRefusal {}

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
    /// A read action. The consumer runs it outside its pool lock with the operation's `read`.
    Read,
    /// A pool action. The consumer runs it under its pool lock with the pool's `perform`; it
    /// revalidates qualified storage before accessing it.
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
    /// Whether the consumer performs this action with the operation's `read` or the pool's
    /// `perform`.
    pub fn kind(&self) -> HtmlValuePoolActionKind {
        if matches!(self.phase, Phase::Hash | Phase::Compare) {
            HtmlValuePoolActionKind::Read
        } else {
            HtmlValuePoolActionKind::Pool
        }
    }
    /// The storage and work the consumer admits before performing the action.
    pub fn layout(&self) -> HtmlValuePoolLayout {
        self.layout
    }
}

/// The consumer pairs a newly acquired candidate with the reservation made
/// before the upgrade. Release destroys that strong value before its receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlValuePoolProgress {
    /// The action completed. `next_action` then describes a further action, or returns `None`
    /// when the operation is already terminal, as after the touch of a hit.
    Continue,
    /// The action upgraded a weak occurrence into a strongly held candidate. The consumer pairs
    /// the candidate with the reservation it made before the upgrade.
    CandidateAcquired,
    /// The action destroyed the candidate, for example after a comparison mismatch, or found the
    /// weak occurrence already expired so that no candidate could be acquired. The consumer then
    /// releases the reservation it made for the candidate.
    CandidateReleased,
    /// The operation reached a terminal state. Complete it with `try_finish`.
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
    /// The pool's usage before the transition.
    pub fn before(self) -> HtmlValuePoolUsage {
        self.before
    }
    /// The pool's usage after the transition.
    pub fn after(self) -> HtmlValuePoolUsage {
        self.after
    }
    /// The weak occurrence the transition inserted, if any.
    pub fn inserted(self) -> Option<HtmlValuePoolOccurrence> {
        self.inserted
    }
    /// The weak occurrence the transition moved into undo storage, if any.
    pub fn moved_to_undo(self) -> Option<HtmlValuePoolOccurrence> {
        self.moved_to_undo
    }
    /// The number of weak occurrences the transition released.
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
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .ok()
}

/// The four strings of one attribute, in the order they are read.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum AttributePart {
    #[default]
    NamespaceUri,
    Prefix,
    LocalName,
    Value,
}
impl AttributePart {
    /// The position of this part among the four, which offsets the part's read tags.
    fn position(self) -> u8 {
        match self {
            Self::NamespaceUri => 0,
            Self::Prefix => 1,
            Self::LocalName => 2,
            Self::Value => 3,
        }
    }
    /// The part read after this one, or `None` after the last.
    fn next(self) -> Option<Self> {
        match self {
            Self::NamespaceUri => Some(Self::Prefix),
            Self::Prefix => Some(Self::LocalName),
            Self::LocalName => Some(Self::Value),
            Self::Value => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Cursor {
    attribute: usize,
    part: AttributePart,
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
    let capacity = value.retained_units();
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
        self.whole_block_vec()
    }
    fn from_shared(value: Arc<Vec<u16>>) -> Self {
        Self::from_shared_utf16(value)
    }
    /// The pool holds the blocks of strings that own their allocation, so a range of a larger
    /// block is never pooled.
    fn eligible(&self) -> bool {
        self.is_whole_block()
            && self.len() <= policy::MAX_UNITS
            && self.retained_units() <= policy::MAX_UNITS
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
            AttributePart::NamespaceUri => attribute.name().namespace().uri(),
            AttributePart::Prefix => attribute.name().prefix(),
            AttributePart::LocalName => Some(attribute.name().local_name()),
            AttributePart::Value => Some(attribute.value()),
        };
        let result = if let Some(string) = string {
            let tag = 2 + cursor.part.position();
            read_string(string, cursor, tag)?
        } else {
            Read {
                word: Word {
                    tag: 9 + cursor.part.position(),
                    value: 0,
                },
                payload: HtmlValuePoolUsage::default(),
            }
        };
        if !cursor.units {
            match cursor.part.next() {
                Some(part) => cursor.part = part,
                None => {
                    cursor.part = AttributePart::NamespaceUri;
                    cursor.attribute += 1;
                }
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
            /// Creates an empty owner without allocating.
            pub fn new() -> Self {
                Self::default()
            }
            /// Actual inline owner layout, independent of dynamic pool storage.
            pub fn inline_bytes() -> usize {
                size_of::<Self>()
            }
            /// Fixed upper bound for destroying this whole owner in any state.
            /// Charge once BEFORE constructing it, including an empty owner.
            /// This covers live/old Weak entries, initialized pages and undo
            /// records, and backing release, even after sealing. It is separate
            /// from root metadata and each action's temporary/inverse work.
            /// Drop charges nothing and issues no qualifier.
            pub fn charge_disposal<C: Charge<HtmlWork, HtmlResource>>(
                charge: &mut C,
            ) -> Result<(), C::Stop> {
                policy::disposal_work().charge(charge)
            }
            /// Charge before beginning root authority; includes its fixed close.
            pub fn charge_root<C: Charge<HtmlWork, HtmlResource>>(
                charge: &mut C,
            ) -> Result<(), C::Stop> {
                policy::root_work::<$value>().charge(charge)
            }
            /// Charge before issuing a nested mark; includes accept or restore's
            /// fixed metadata work. Inverse work is paid by mutation layouts.
            pub fn charge_mark<C: Charge<HtmlWork, HtmlResource>>(
                charge: &mut C,
            ) -> Result<(), C::Stop> {
                policy::mark_work::<$value>().charge(charge)
            }
            /// Charge before each attempt to freeze a root, including a retry
            /// after unresolved marks refused preparation. This pays the
            /// bounded readiness/metadata checks, not another root copy.
            /// Root construction already prepays fixed closing/header moves;
            /// mutation layouts prepay inverse and backing cleanup. Validation
            /// and closing afterward need no further admission callback.
            pub fn charge_seal<C: Charge<HtmlWork, HtmlResource>>(
                charge: &mut C,
            ) -> Result<(), C::Stop> {
                policy::seal_work().charge(charge)
            }
            #[cfg(test)]
            pub(crate) fn disposal_work() -> HtmlValuePoolWork {
                policy::disposal_work()
            }
            #[cfg(test)]
            pub(crate) fn root_work() -> HtmlValuePoolWork {
                policy::root_work::<$value>()
            }
            #[cfg(test)]
            pub(crate) fn mark_work() -> HtmlValuePoolWork {
                policy::mark_work::<$value>()
            }
            #[cfg(test)]
            pub(crate) fn seal_work() -> HtmlValuePoolWork {
                policy::seal_work()
            }
            /// O(1) admitted heap extent, including undo and conservative
            /// per-Weak target space. This is not distinct physical allocation
            /// counting; multiple occurrences can refer to the same allocation.
            pub fn heap_usage(&self) -> HtmlValuePoolUsage {
                self.inner.usage()
            }
            /// Starts a lookup of `input` that never inserts. A hit yields a strongly held
            /// candidate that equals `input`.
            pub fn probe<'a>(&mut self, input: &'a $value) -> $operation<'a> {
                $operation {
                    inner: self.inner.operation(input, false),
                }
            }
            /// Starts a lookup of `input` that inserts it as a weak occurrence on a miss, when the
            /// pool admits it. A string matches only the same UTF-16 code units. An attribute
            /// list matches only the same attributes in the same order, each with its exact
            /// namespace URI, prefix, local name and value.
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
            /// Begins the single rollback root. Returns `None` when a root is already open or an
            /// identifier cannot be issued. Opening a root discards any unfinished operation.
            pub fn begin_root(&mut self) -> Option<HtmlValuePoolRoot> {
                self.inner.begin_root()
            }
            /// Begins a nested mark for `root`. Returns `Ok(None)` when `root` is stale, frozen or
            /// suspended, or when no mark can be issued; exhausting marks or identifiers suspends
            /// further mutations of this root. Returns `ForeignOwner` for another pool's root.
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
            /// Accepts `mark`: its changes merge into the nearest older mark, or the root, and
            /// stay. A stale mark is ignored. Returns `NotReady` once the root is frozen and
            /// `ForeignOwner` for another pool's mark.
            pub fn accept_mark(
                &mut self,
                mark: HtmlValuePoolMark,
            ) -> Result<(), HtmlValuePoolError> {
                self.inner.accept(mark)
            }
            /// Undoes the mutations made since `mark`, including those of younger marks, and
            /// returns the resulting change. A stale mark returns an unchanged value. Returns
            /// `NotReady` once the root is frozen and `ForeignOwner` for another pool's mark.
            pub fn restore_mark(
                &mut self,
                mark: HtmlValuePoolMark,
            ) -> Result<HtmlValuePoolChange, HtmlValuePoolError> {
                self.inner.restore(mark)
            }
            /// Aborts the root: every mark and the root's own mutations are undone, and the root
            /// closes. A stale root returns an unchanged value.
            pub fn abort_root(
                &mut self,
                root: HtmlValuePoolRoot,
            ) -> Result<HtmlValuePoolChange, HtmlValuePoolError> {
                self.inner.abort(root)
            }
            /// Freezes `root` for sealing when no mark is outstanding. Otherwise the refusal
            /// returns the root unfrozen, with `NotReady` for outstanding marks, a frozen root or a
            /// stale root, and `ForeignOwner` for another pool's root.
            pub fn prepare_seal(
                &mut self,
                root: HtmlValuePoolRoot,
            ) -> Result<HtmlValuePoolSeal, HtmlValuePoolSealRefusal> {
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
            /// Seals a validated frozen root: the mutations stay, the undo records are released
            /// and the root closes. Returns `NotReady` when `seal` is stale and `ForeignOwner` for
            /// another pool's seal.
            pub fn seal(
                &mut self,
                seal: HtmlValuePoolSeal,
            ) -> Result<HtmlValuePoolChange, HtmlValuePoolError> {
                self.inner.seal(seal)
            }
            /// Aborts a frozen root: every mutation is undone and the root closes.
            pub fn abort_seal(
                &mut self,
                seal: HtmlValuePoolSeal,
            ) -> Result<HtmlValuePoolChange, HtmlValuePoolError> {
                self.inner.abort(seal.root)
            }
        }
        impl fmt::Debug for $pool {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($pool))
                    .field("heap_usage", &self.heap_usage())
                    .finish_non_exhaustive()
            }
        }
        /// A caller-input-borrowing operation, independent of the pool lock.
        /// On refusal, drop this owner before its provisional/candidate receipts.
        pub struct $operation<'a> {
            inner: Operation<'a, $value>,
        }
        impl fmt::Debug for $operation<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($operation))
                    .field("phase", &self.inner.state.phase())
                    .field("change", &self.inner.change)
                    .finish_non_exhaustive()
            }
        }
        impl $operation<'_> {
            /// Performs a read action, such as hashing or comparing, outside the consumer's pool
            /// lock. Returns `WrongAction` for an action of another operation, with a serial or
            /// phase that does not match, or for the wrong entry point, and `ForeignOwner` for
            /// another pool's action. A stale action with an older serial bypasses sharing and
            /// yields `Finished` instead of an error.
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
                    .state
                    .phase();
                if !matches!(
                    phase,
                    Phase::Hit | Phase::Miss | Phase::Offered | Phase::Bypassed
                ) {
                    return Err(HtmlValuePoolError::NotReady);
                }
                let operation = owner.take().ok_or(HtmlValuePoolError::NotReady)?;
                let change = operation.inner.change;
                match operation.inner.state {
                    State::Hit(value) => Ok($result::Hit($candidate { value })),
                    State::Miss => Ok($result::Miss),
                    State::Offered => Ok($result::Offered(change)),
                    State::Bypassed => Ok($result::Bypassed),
                    // The phase check above admitted only the four finished states.
                    _ => Err(HtmlValuePoolError::NotReady),
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
            /// The matched value, as stored.
            pub fn value(&self) -> &$value {
                &self.value
            }
            /// Takes the matched value. The consumer's receipt for it must stay live until the
            /// value is destroyed or moved into guarded output.
            pub fn into_value(self) -> $value {
                self.value
            }
        }
        /// How a completed operation ended.
        #[derive(Debug)]
        pub enum $result {
            /// An existing equal value was found. The candidate holds it strongly.
            Hit($candidate),
            /// A probe found no equal value and inserted nothing.
            Miss,
            /// No equal value was found and the input was inserted. The change reports the
            /// insertion and any released occurrences.
            Offered(HtmlValuePoolChange),
            /// Sharing was skipped, for example because the value is ineligible, an optional
            /// reservation was refused, a private qualifier or the metadata cap was exhausted, the
            /// root is not open or active, the operation was superseded or a checkpoint changed,
            /// or reserving metadata failed. The input is unchanged and remains usable.
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

#[cfg(test)]
mod attribute_part_tests {
    use super::AttributePart;

    /// An attribute is read as its namespace URI, prefix, local name and value, in that order,
    /// and the tags of the four reads are offset by the part's position among them.
    #[test]
    fn the_four_parts_of_an_attribute_are_read_in_order() {
        let mut parts = vec![AttributePart::default()];
        while let Some(next) = parts.last().and_then(|part| part.next()) {
            parts.push(next);
        }
        assert_eq!(
            parts,
            [
                AttributePart::NamespaceUri,
                AttributePart::Prefix,
                AttributePart::LocalName,
                AttributePart::Value
            ]
        );
        assert_eq!(
            parts.iter().map(|part| part.position()).collect::<Vec<_>>(),
            [0, 1, 2, 3]
        );
    }
}
