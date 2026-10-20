//! Positive recipient ownership, independent of callbacks and immutable State.
use super::*;
use crate::{DomEventRecipient, DomEventRecipientLimits, DomEventTarget, DomEventType};
use source::formation::Formation;
use std::{cell::Cell, sync::Mutex};

#[cfg(test)]
mod tests;
mod topology;

pub(super) struct Processing<'a> {
    limits: DomEventRecipientLimits,
    spent: Cell<u64>,
    formation: Option<&'a Formation>,
    cancel: &'a AtomicBool,
}
impl<'a> Processing<'a> {
    pub(super) fn new(
        limits: DomEventRecipientLimits,
        formation: Option<&'a Formation>,
        cancel: &'a AtomicBool,
    ) -> Self {
        Self {
            limits,
            spent: Cell::new(0),
            formation,
            cancel,
        }
    }
    pub(super) fn step(&self) -> Result<(), DomError> {
        check_cancel(self.cancel)?;
        let next = self
            .spent
            .get()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.limits.work() {
            return Err(DomError::WorkLimit {
                resource: DomResource::EventRecipientWork,
                limit: self.limits.work(),
            });
        }
        source::formation::step(self.formation, self.cancel)?;
        self.spent.set(next);
        Ok(())
    }
    pub(super) fn scratch(&self, bytes: usize) -> Result<(), DomError> {
        admit(
            bytes,
            self.limits.scratch_bytes(),
            DomResource::EventRecipientScratch,
        )
    }
    pub(super) fn admit(&self, usage: custody::Usage) -> Result<(), DomError> {
        self.step()?;
        match self.formation {
            Some(formation) => formation.admit_custody_usage(usage, self.cancel),
            None => check_cancel(self.cancel),
        }
    }
    fn steps(&self, count: usize) -> Result<(), DomError> {
        for _ in 0..count {
            self.step()?;
        }
        Ok(())
    }
}

// This small operational target can outlive Dom through an affine lease. It
// contains no canonical data; its own original debit lasts through the last
// bucket/lease/index receipt, including after the exclusive owner is dropped.
struct Accounting {
    live: AtomicUsize,
    bytes: AtomicUsize,
    _storage: custody::Debit,
}
struct LocalDebit {
    accounting: Arc<Accounting>,
    global: Option<custody::Debit>,
    bytes: usize,
}
impl LocalDebit {
    fn new(
        accounting: &Arc<Accounting>,
        ledger: &custody::Ledger,
        bytes: usize,
        processing: &Processing<'_>,
    ) -> Result<Self, DomError> {
        accounting
            .bytes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current
                    .checked_add(bytes)
                    .filter(|next| *next <= processing.limits.retained_bytes())
            })
            .map_err(|_| DomError::ResourceLimit {
                resource: DomResource::EventRecipientRetainedBytes,
                limit: processing.limits.retained_bytes(),
            })?;
        let mut value = Self {
            accounting: accounting.clone(),
            global: None,
            bytes,
        };
        value.global = Some(ledger.retain(custody::Usage::new(bytes, 0), |usage| {
            processing.admit(usage)
        })?);
        Ok(value)
    }
    fn grow(
        &mut self,
        ledger: &custody::Ledger,
        bytes: usize,
        processing: &Processing<'_>,
    ) -> Result<(), DomError> {
        let extra = Self::new(&self.accounting, ledger, bytes, processing)?;
        self.absorb(extra);
        Ok(())
    }
    fn absorb(&mut self, mut other: Self) {
        assert!(
            Arc::ptr_eq(&self.accounting, &other.accounting),
            "same recipient owner"
        );
        self.global
            .as_mut()
            .expect("live local receipt")
            .absorb(other.global.take().expect("live incoming receipt"))
            .expect("original recipient ledger");
        self.bytes = self
            .bytes
            .checked_add(other.bytes)
            .expect("admitted combined extent");
        other.bytes = 0;
    }
    fn release(&mut self, bytes: usize) {
        if bytes == 0 {
            return;
        }
        // The represented unused allocations have already died.
        let released = self
            .global
            .as_mut()
            .expect("live receipt")
            .split(custody::Usage::new(bytes, 0))
            .expect("owned unused reserve");
        self.bytes = self.bytes.checked_sub(bytes).expect("owned local reserve");
        let old = self.accounting.bytes.fetch_sub(bytes, Ordering::AcqRel);
        assert!(old >= bytes, "owned recipient bytes");
        drop(released);
    }
}
impl Drop for LocalDebit {
    fn drop(&mut self) {
        let old = self
            .accounting
            .bytes
            .fetch_sub(self.bytes, Ordering::AcqRel);
        assert!(old >= self.bytes, "owned recipient extent");
    }
}

struct Bucket {
    target: DomEventTarget,
    counts: [AtomicUsize; 4],
    _storage: LocalDebit,
}
/// The callback owner holds this affine occurrence. Its bucket and accounting
/// cannot retain a tree, document, snapshot or callback registry.
pub(crate) struct Lease {
    bucket: Arc<Bucket>,
    category: usize,
    active: bool,
    _storage: LocalDebit,
}
impl Drop for Lease {
    fn drop(&mut self) {
        if self.active {
            let count = self.bucket.counts[self.category].fetch_sub(1, Ordering::AcqRel);
            assert!(count > 0, "one affine recipient occurrence");
            // Acquire of live==0 sees every preceding bucket decrement.
            let live = self._storage.accounting.live.fetch_sub(1, Ordering::AcqRel);
            assert!(live > 0, "one aggregate recipient occurrence");
        }
    }
}

enum Part {
    Branch(OwnedBank<Part>),
    Endpoint(Arc<Bucket>),
}
struct Index {
    nodes: OwnedBank<Part>,
    globals: OwnedBank<Part>,
    storage: LocalDebit,
}
pub(super) struct Owner {
    owner: DomOwnerId,
    index: Mutex<Option<Index>>,
    ledger: custody::Ledger,
    accounting: Arc<Accounting>,
    _storage: LocalDebit,
}
struct IndexLease<'a> {
    index: Option<Index>,
    owner: &'a Owner,
}
impl<'a> IndexLease<'a> {
    fn take(owner: &'a Owner) -> Result<Self, DomError> {
        let index = owner
            .index
            .lock()
            .expect("recipient index invariant")
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        Ok(Self {
            index: Some(index),
            owner,
        })
    }
}
impl Drop for IndexLease<'_> {
    fn drop(&mut self) {
        let mut index = self.owner.index.lock().expect("recipient index invariant");
        assert!(index.is_none(), "one exclusive index lease");
        *index = self.index.take();
    }
}

// Split opaque u64 components into bounded radix indices on either pointer
// width. Unlike a raw u64->usize cast, no valid generation aliases another.
const PART_BITS: u32 = usize::BITS / 2;
const U64_PARTS: usize = 64usize.div_ceil(PART_BITS as usize);
const MAX_PARTS: usize = U64_PARTS * 2 + 1;
struct Address {
    global: bool,
    parts: [usize; MAX_PARTS],
    len: usize,
}
impl Address {
    fn new(target: DomEventTarget) -> Self {
        let mut value = Self {
            global: matches!(target, DomEventTarget::Global(_)),
            parts: [0; MAX_PARTS],
            len: 0,
        };
        let number = |number: u64, value: &mut Self| {
            for part in 0..U64_PARTS {
                value.parts[value.len] =
                    ((number >> (part as u32 * PART_BITS)) & ((1u64 << PART_BITS) - 1)) as usize;
                value.len += 1;
            }
        };
        match target {
            DomEventTarget::Node(node) => {
                value.parts[0] = node.slot;
                value.len = 1;
                number(node.generation, &mut value);
            }
            DomEventTarget::Global(global) => {
                number(global.source.serial, &mut value);
                value.parts[value.len] = global.slot;
                value.len += 1;
                number(global.generation, &mut value);
            }
        }
        value
    }
}
fn read_steps<T>(bank: &OwnedBank<T>, index: usize) -> usize {
    if index >= bank.len() {
        return 1;
    }
    let mut last = bank.len().saturating_sub(1);
    let mut depth = 1;
    while last >= 16 {
        last >>= 4;
        depth += 1;
    }
    depth
}
fn lookup(
    index: &Index,
    address: &Address,
    processing: &Processing<'_>,
) -> Result<Option<Arc<Bucket>>, DomError> {
    let mut bank = if address.global {
        &index.globals
    } else {
        &index.nodes
    };
    for position in 0..address.len {
        let key = address.parts[position];
        match bank.get_with(key, &mut || processing.step())? {
            Some(Part::Endpoint(bucket)) if position + 1 == address.len => {
                return Ok(Some(bucket.clone()));
            }
            Some(Part::Branch(next)) if position + 1 < address.len => bank = next,
            None => return Ok(None),
            _ => unreachable!("fixed endpoint component depth"),
        }
    }
    unreachable!("nonempty endpoint address")
}

struct Insertion {
    value: Part,
    path: EmptyPath<Part>,
    prefix: [usize; MAX_PARTS],
    prefix_len: usize,
    global: bool,
    storage: LocalDebit,
}
fn reserve_path(
    bank: &OwnedBank<Part>,
    key: usize,
    storage: &mut LocalDebit,
    owner: &Owner,
    processing: &Processing<'_>,
) -> Result<EmptyPath<Part>, DomError> {
    // Reserve/insert touch only this radix path. Existing paths prepay their
    // traversal/refreshes. Each actual new page additionally prepays its 16
    // initialized cells, initial summary and at most two insertion/growth
    // refreshes, before the allocation or any resident mutation.
    processing.steps(
        read_steps(bank, key)
            .checked_mul(34)
            .ok_or(DomError::RepresentationExhausted)?,
    )?;
    bank.reserve(key, |charge| {
        processing.steps(
            charge
                .pages
                .checked_mul(64)
                .and_then(|n| n.checked_add(1))
                .ok_or(DomError::RepresentationExhausted)?,
        )?;
        processing.scratch(
            storage
                .bytes
                .checked_add(charge.bytes)
                .ok_or(DomError::RepresentationExhausted)?,
        )?;
        storage.grow(&owner.ledger, charge.bytes, processing)?;
        Ok::<_, DomError>(None)
    })
}
fn insert_private(
    bank: &mut OwnedBank<Part>,
    path: EmptyPath<Part>,
    value: Part,
    storage: &mut LocalDebit,
) {
    let total = path.charge().bytes;
    let moved = bank.insert(path, value, false).bytes;
    storage.release(total.checked_sub(moved).expect("used reserve subset"));
}
impl Insertion {
    fn prepare(
        index: &Index,
        address: &Address,
        bucket: &Arc<Bucket>,
        owner: &Owner,
        processing: &Processing<'_>,
    ) -> Result<Self, DomError> {
        let mut resident = if address.global {
            &index.globals
        } else {
            &index.nodes
        };
        let mut position = 0;
        loop {
            match resident.get_with(address.parts[position], &mut || processing.step())? {
                Some(Part::Branch(next)) => {
                    resident = next;
                    position += 1;
                }
                None => break,
                Some(Part::Endpoint(_)) => unreachable!("new endpoint absent"),
            }
        }
        let mut storage = LocalDebit::new(&owner.accounting, &owner.ledger, 0, processing)?;
        // Entire missing suffix remains private. Any failure destroys its data
        // before the original aggregate receipt, leaving resident index intact.
        let mut value = Part::Endpoint(bucket.clone());
        for suffix in ((position + 1)..address.len).rev() {
            let mut next = OwnedBank::default();
            let path = reserve_path(
                &next,
                address.parts[suffix],
                &mut storage,
                owner,
                processing,
            )?;
            insert_private(&mut next, path, value, &mut storage);
            value = Part::Branch(next);
        }
        let path = reserve_path(
            resident,
            address.parts[position],
            &mut storage,
            owner,
            processing,
        )?;
        Ok(Self {
            value,
            path,
            prefix: address.parts,
            prefix_len: position,
            global: address.global,
            storage,
        })
    }
    fn commit(self, index: &mut Index) {
        let Self {
            value,
            path,
            prefix,
            prefix_len,
            global,
            storage,
        } = self;
        // Transfer all already-admitted target storage to the resident owner
        // before moving data into it. Even a private invariant unwind cannot
        // destroy its receipt ahead of a partly installed resident path.
        index.storage.absorb(storage);
        let Index {
            nodes,
            globals,
            storage,
        } = index;
        let mut bank = if global { globals } else { nodes };
        for key in &prefix[..prefix_len] {
            let Some(Part::Branch(next)) = bank.get_mut(*key) else {
                unreachable!("original index prefix")
            };
            bank = next;
        }
        insert_private(bank, path, value, storage);
    }
}

impl Owner {
    fn new(
        owner: DomOwnerId,
        ledger: &custody::Ledger,
        processing: &Processing<'_>,
    ) -> Result<Arc<Self>, DomError> {
        let bytes = size_of::<Accounting>();
        admit(
            bytes,
            processing.limits.retained_bytes(),
            DomResource::EventRecipientRetainedBytes,
        )?;
        let storage = ledger.retain(custody::Usage::new(bytes, 0), |usage| {
            processing.admit(usage)
        })?;
        let accounting = source::formation::allocated_arc(Accounting {
            live: AtomicUsize::new(0),
            bytes: AtomicUsize::new(bytes),
            _storage: storage,
        });
        let body = LocalDebit::new(&accounting, ledger, size_of::<Self>(), processing)?;
        let index = Index {
            nodes: OwnedBank::default(),
            globals: OwnedBank::default(),
            storage: LocalDebit::new(&accounting, ledger, 0, processing)?,
        };
        Ok(source::formation::allocated_arc(Self {
            owner,
            index: Mutex::new(Some(index)),
            ledger: ledger.clone(),
            accounting,
            _storage: body,
        }))
    }
    pub(super) fn has_live(&self) -> bool {
        self.accounting.live.load(Ordering::Acquire) != 0
    }
    pub(super) fn admit_limits(&self, limits: DomEventRecipientLimits) -> Result<(), DomError> {
        admit(
            self.accounting.live.load(Ordering::Acquire),
            limits.registrations(),
            DomResource::EventRecipients,
        )?;
        admit(
            self.accounting.bytes.load(Ordering::Acquire),
            limits.retained_bytes(),
            DomResource::EventRecipientRetainedBytes,
        )
    }
    pub(super) fn has_recipient(
        &self,
        target: DomEventTarget,
        event_type: DomEventType,
        at_target: bool,
        processing: &Processing<'_>,
    ) -> Result<bool, DomError> {
        let owner = match target {
            DomEventTarget::Node(node) => node.owner,
            DomEventTarget::Global(global) => global.source.owner,
        };
        if owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        let Some(event) = event_type.notification() else {
            return Ok(false);
        };
        if !self.has_live() {
            return Ok(false);
        }
        let lease = IndexLease::take(self)?;
        let bucket = lookup(
            lease.index.as_ref().expect("leased index"),
            &Address::new(target),
            processing,
        )?;
        let Some(bucket) = bucket else {
            return Ok(false);
        };
        assert_eq!(bucket.target, target, "complete endpoint qualification");
        processing.step()?;
        if bucket.counts[event * 2 + 1].load(Ordering::Acquire) != 0 {
            return Ok(true);
        }
        if at_target {
            processing.step()?;
            return Ok(bucket.counts[event * 2].load(Ordering::Acquire) != 0);
        }
        Ok(false)
    }
    fn retain(
        &self,
        target: DomEventTarget,
        event: usize,
        capture: bool,
        processing: &Processing<'_>,
    ) -> Result<DomEventRecipient, DomError> {
        processing.step()?;
        admit(
            self.accounting
                .live
                .load(Ordering::Acquire)
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
            processing.limits.registrations(),
            DomResource::EventRecipients,
        )?;
        let mut index = IndexLease::take(self)?;
        let address = Address::new(target);
        let found = lookup(
            index.index.as_ref().expect("leased index"),
            &address,
            processing,
        )?;
        let fresh = found.is_none();
        let bucket = match found {
            Some(bucket) => bucket,
            None => {
                let storage = LocalDebit::new(
                    &self.accounting,
                    &self.ledger,
                    size_of::<Bucket>(),
                    processing,
                )?;
                source::formation::allocated_arc(Bucket {
                    target,
                    counts: std::array::from_fn(|_| AtomicUsize::new(0)),
                    _storage: storage,
                })
            }
        };
        let storage = LocalDebit::new(
            &self.accounting,
            &self.ledger,
            size_of::<DomEventRecipient>(),
            processing,
        )?;
        let category = event * 2 + usize::from(capture);
        bucket.counts[category]
            .load(Ordering::Acquire)
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let insertion = if fresh {
            Some(Insertion::prepare(
                index.index.as_ref().expect("leased index"),
                &address,
                &bucket,
                self,
                processing,
            )?)
        } else {
            None
        };
        check_cancel(processing.cancel)?;
        let mut lease = Lease {
            bucket,
            category,
            active: false,
            _storage: storage,
        };
        if let Some(insertion) = insertion {
            insertion.commit(index.index.as_mut().expect("leased index"));
        }
        // The exclusive Dom permits no competing positive admission. Concurrent
        // lease drops can only reduce both checked counters. No fallible work,
        // allocation or user callback follows this fence. IndexLease's Drop
        // only returns the resident index under its short internal lock.
        lease.bucket.counts[category].fetch_add(1, Ordering::AcqRel);
        self.accounting.live.fetch_add(1, Ordering::AcqRel);
        lease.active = true;
        Ok(DomEventRecipient { _lease: lease })
    }
}

impl Dom {
    /// Retain a positive native Load/Error recipient. Other event classes need
    /// no closure capability and return None. Dropping the affine result ends
    /// this occurrence. It grants no dispatch, callback or execution authority.
    pub fn retain_event_recipient(
        &mut self,
        target: DomEventTarget,
        event_type: DomEventType,
        capture: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<DomEventRecipient>, crate::DomHostedError> {
        let Some(event) = event_type.notification() else {
            check_cancel(cancel)?;
            return Ok(None);
        };
        self.require_source_entry()?;
        check_cancel(cancel)?;
        match target {
            DomEventTarget::Node(node) => {
                self.state.node(self.owner, node)?;
            }
            DomEventTarget::Global(global) => {
                self.dispatch_global_document(global, cancel)?;
            }
        }
        let formation = self.source_observation_formation(cancel)?;
        let processing =
            Processing::new(self.limits.event_recipients(), formation.as_ref(), cancel);
        let next = self
            .event_recipients
            .as_ref()
            .map_or(0, |owner| owner.accounting.live.load(Ordering::Acquire))
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            next,
            self.limits.event_recipients().registrations(),
            DomResource::EventRecipients,
        )?;
        let owner = match &self.event_recipients {
            Some(owner) => owner.clone(),
            None => Owner::new(self.owner, self.custody_registry.ledger(), &processing)?,
        };
        let recipient = owner.retain(target, event, capture, &processing)?;
        self.event_recipients = Some(owner);
        Ok(Some(recipient))
    }
}
