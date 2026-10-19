//! Finite exact DOM-bank policy. Frames save the first displaced occurrence
//! only; turnover cannot append an unbounded history of Weak handles.
use super::*;
use crate::name::{AttributeValues, NameRecord};
use crate::store::payload;
use std::hash::{Hash, Hasher};

const BUCKETS: usize = 32;
const WAYS: usize = 4;
const SLOTS: usize = BUCKETS * WAYS;
const MARKS: usize = 16;
pub(super) const MAX_UNITS: usize = 8192;
const MAX_ENTRIES: usize = 64;
const MAX_BYTES: usize = 256 * 1024;
pub(super) const DISPOSAL_WORK: usize = SLOTS * (MARKS + 2) * 4;
const FRAMES: usize = MARKS + 1;
// Begin initializes the frame directory/cursors. Its base frame also prepays
// abort's any/frame-index/parent-allocation/final-empty header scans. Saved
// cells and a newly allocated slot directory pay their own closing work below.
pub(super) const ROOT_WORK: usize = 5 * FRAMES + 2 * BUCKETS;
// Creation: vacant-mark search, allocation-mask scan, cursor initialization.
// Closure: at most four frame scans and a cursor walk, including an outer
// restore selecting this frame. Accept needs only two scans and a cursor walk.
pub(super) const MARK_WORK: usize = MARKS + 5 * FRAMES + 2 * BUCKETS;

pub(super) fn close_work(nominated: usize, live: usize) -> Result<usize, DomError> {
    let add = |a: usize, b: usize| a.checked_add(b).ok_or(DomError::RepresentationExhausted);
    let mul = |a: usize, b: usize| a.checked_mul(b).ok_or(DomError::RepresentationExhausted);
    let duplicate_pairs = mul(nominated, nominated.saturating_sub(1))? / 2;
    // Each verify visits each nomination, its owner/token, and up to FRAMES
    // live-mark headers. Both complete passes are paid before the first one.
    let verify = add(duplicate_pairs, mul(nominated, FRAMES + 3)?)?;
    // Deepest-first close selects from all nominations for each live mark.
    let selection = add(mul(mul(live, nominated)?, FRAMES + 3)?, mul(2, live)?)?;
    add(add(mul(2, verify)?, selection)?, MARKS + FRAMES)
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Shape {
    frames: [Option<(u64, usize, usize)>; FRAMES],
    cursor: u8,
    next: Option<u64>,
    had_slots: bool,
}
struct OfferPlan {
    shape: Shape,
    frame: usize,
    first_before: bool,
    need_frames: [bool; FRAMES],
    requested: usize,
    before: custody::Usage,
    closing_work: usize,
}

struct Entry {
    value: Weak<payload::Payload<AttributeValues>>,
    hash: u64,
    prospective: custody::Usage,
}
struct Before {
    slot: usize,
    value: Option<Entry>,
}
struct Frame {
    serial: u64,
    had_slots: bool,
    allocated_before: u32,
    saved: Vec<Before>,
    cursors: [Option<u8>; BUCKETS],
}
impl Frame {
    fn new(serial: u64, had_slots: bool, allocated_before: u32) -> Self {
        Self {
            serial,
            had_slots,
            allocated_before,
            saved: Vec::new(),
            cursors: [None; BUCKETS],
        }
    }
}
struct Root {
    id: u64,
    frames: [Option<Frame>; MARKS + 1],
    frozen: bool,
}
pub(super) struct Mark {
    root: u64,
    serial: u64,
}
impl Mark {
    pub(super) fn serial(&self) -> u64 {
        self.serial
    }
}
pub(super) struct Banks {
    slots: Vec<Option<Entry>>,
    cursors: [u8; BUCKETS],
    root: Option<Root>,
    next: Option<u64>,
    weak: usize,
    buffers: usize,
}
impl Banks {
    pub(super) fn occupied(&self) -> bool {
        self.root.is_some()
    }
    pub(super) fn mark_is_live(&self, mark: &Mark) -> bool {
        self.root.as_ref().is_some_and(|root| {
            root.id == mark.root
                && !root.frozen
                && root
                    .frames
                    .iter()
                    .flatten()
                    .any(|frame| frame.serial == mark.serial)
        })
    }
    pub(super) fn new() -> Self {
        Self {
            slots: Vec::new(),
            cursors: [0; BUCKETS],
            root: None,
            next: Some(1),
            weak: 0,
            buffers: 0,
        }
    }
    fn issue(&mut self) -> Option<u64> {
        let id = self.next?;
        self.next = id.checked_add(1);
        Some(id)
    }
    pub(super) fn usage(&self) -> custody::Usage {
        custody::Usage::new(
            self.buffers + self.weak * size_of::<payload::Payload<AttributeValues>>(),
            0,
        )
    }
    pub(super) fn frozen(&self) -> bool {
        self.root.as_ref().is_some_and(|root| root.frozen)
    }
    fn shape(&self, bucket: usize) -> Shape {
        let root = self.root.as_ref().expect("active bank root");
        Shape {
            frames: std::array::from_fn(|i| {
                root.frames[i]
                    .as_ref()
                    .map(|frame| (frame.serial, frame.saved.len(), frame.saved.capacity()))
            }),
            cursor: self.cursors[bucket],
            next: self.next,
            had_slots: !self.slots.is_empty(),
        }
    }
    pub(super) fn begin(&mut self) -> Option<u64> {
        if self.root.is_some() {
            return None;
        }
        let id = self.issue()?;
        self.root = Some(Root {
            id,
            frames: std::array::from_fn(|i| {
                (i == 0).then(|| Frame::new(id, !self.slots.is_empty(), 0))
            }),
            frozen: false,
        });
        Some(id)
    }
    pub(super) fn mark(&mut self, id: u64) -> Option<Mark> {
        let root = self.root.as_ref()?;
        if root.id != id || root.frozen {
            return None;
        }
        let index = (1..=MARKS).find(|&i| root.frames[i].is_none())?;
        let allocated = root
            .frames
            .iter()
            .enumerate()
            .fold(0u32, |bits, (i, frame)| {
                bits | (u32::from(
                    frame
                        .as_ref()
                        .is_some_and(|frame| frame.saved.capacity() != 0),
                ) << i)
            });
        let serial = self.issue()?;
        self.root.as_mut()?.frames[index] =
            Some(Frame::new(serial, !self.slots.is_empty(), allocated));
        Some(Mark { root: id, serial })
    }
    fn frame_index(&self) -> usize {
        self.root
            .as_ref()
            .expect("active root")
            .frames
            .iter()
            .enumerate()
            .filter_map(|(i, frame)| frame.as_ref().map(|frame| (frame.serial, i)))
            .max()
            .expect("base frame")
            .1
    }
    fn drop_entry(&mut self, value: Option<Entry>) {
        if value.is_some() {
            self.weak -= 1;
        }
        drop(value);
    }
    pub(super) fn accept(&mut self, mark: Mark) -> bool {
        let Some(root) = self
            .root
            .as_mut()
            .filter(|root| root.id == mark.root && !root.frozen)
        else {
            return false;
        };
        let Some(index) = root.frames.iter().position(|frame| {
            frame
                .as_ref()
                .is_some_and(|frame| frame.serial == mark.serial)
        }) else {
            return false;
        };
        let frame = root.frames[index].take().expect("matching frame");
        let frame_bytes = frame.saved_capacity_bytes();
        let parent = root
            .frames
            .iter()
            .enumerate()
            .filter_map(|(i, frame)| {
                frame
                    .as_ref()
                    .filter(|frame| frame.serial < mark.serial)
                    .map(|frame| (frame.serial, i))
            })
            .max()
            .expect("base ancestor")
            .1;
        let parent = root.frames[parent].as_mut().expect("parent frame");
        for old in frame.saved {
            if parent.saved.iter().any(|entry| entry.slot == old.slot) {
                if old.value.is_some() {
                    self.weak -= 1;
                }
                drop(old);
            } else {
                assert!(
                    parent.saved.len() < parent.saved.capacity(),
                    "parent inverse backing prepaid by first mutation"
                );
                parent.saved.push(old);
                #[cfg(test)]
                copied::<Before>(1);
            }
        }
        for (before, old) in parent.cursors.iter_mut().zip(frame.cursors) {
            if before.is_none() {
                *before = old;
            }
        }
        // Moving initialized records does not retain the discarded capacity.
        self.buffers -= frame_bytes;
        true
    }
    fn restore_frame(&mut self, frame: Frame) {
        let bytes = frame.saved_capacity_bytes();
        let allocated = frame.allocated_before;
        for old in frame.saved {
            let current = std::mem::replace(&mut self.slots[old.slot], old.value);
            self.drop_entry(current);
        }
        for (cursor, before) in self.cursors.iter_mut().zip(frame.cursors) {
            if let Some(before) = before {
                *cursor = before;
            }
        }
        self.buffers -= bytes;
        for (i, parent) in self.root.as_mut().unwrap().frames.iter_mut().enumerate() {
            if let Some(parent) = parent
                && allocated & (1 << i) == 0
                && parent.saved.capacity() != 0
            {
                assert!(
                    parent.saved.is_empty(),
                    "only preallocated empty ancestor inverse storage"
                );
                self.buffers -= parent.saved_capacity_bytes();
                parent.saved = Vec::new();
            }
        }
        if !frame.had_slots && !self.slots.is_empty() {
            assert!(
                self.slots.iter().all(Option::is_none),
                "restored before first bank directory"
            );
            self.buffers -= self.slots.capacity() * size_of::<Option<Entry>>();
            self.slots = Vec::new();
        }
    }
    pub(super) fn restore(&mut self, mark: Mark) -> usize {
        if !self.root.as_ref().is_some_and(|root| {
            root.id == mark.root
                && !root.frozen
                && root.frames.iter().any(|frame| {
                    frame
                        .as_ref()
                        .is_some_and(|frame| frame.serial == mark.serial)
                })
        }) {
            return 0;
        }
        let mut count = 0;
        loop {
            let index = self.frame_index();
            if self.root.as_ref().unwrap().frames[index]
                .as_ref()
                .unwrap()
                .serial
                < mark.serial
            {
                break;
            }
            let frame = self.root.as_mut().unwrap().frames[index].take().unwrap();
            self.restore_frame(frame);
            count += 1;
        }
        count
    }
    pub(super) fn abort(&mut self, id: u64) {
        if !self.root.as_ref().is_some_and(|root| root.id == id) {
            return;
        }
        while self
            .root
            .as_ref()
            .unwrap()
            .frames
            .iter()
            .any(Option::is_some)
        {
            let index = self.frame_index();
            let frame = self.root.as_mut().unwrap().frames[index].take().unwrap();
            self.restore_frame(frame);
        }
        self.root = None;
    }
    pub(super) fn freeze(&mut self, id: u64) {
        let root = self.root.as_mut().expect("owned root");
        assert_eq!(root.id, id);
        assert!(!root.frozen && root.frames[1..].iter().all(Option::is_none));
        root.frozen = true;
    }
    pub(super) fn seal(&mut self, id: u64) {
        let root = self.root.take().expect("frozen root");
        assert!(root.id == id && root.frozen);
        for frame in root.frames.into_iter().flatten() {
            self.buffers -= frame.saved_capacity_bytes();
            for saved in frame.saved {
                self.drop_entry(saved.value);
            }
        }
    }
    #[cfg(test)]
    pub(super) fn inspect(&self) -> (usize, usize, usize, usize) {
        let live = self.slots.iter().flatten().count();
        let saved = self
            .root
            .iter()
            .flat_map(|root| root.frames.iter().flatten())
            .flat_map(|frame| frame.saved.iter())
            .filter(|before| before.value.is_some())
            .count();
        let buffers = self.slots.capacity() * size_of::<Option<Entry>>()
            + self
                .root
                .iter()
                .flat_map(|root| root.frames.iter().flatten())
                .map(|frame| frame.saved.capacity() * size_of::<Before>())
                .sum::<usize>();
        let weak = live + saved;
        (
            buffers + weak * size_of::<payload::Payload<AttributeValues>>(),
            weak,
            live,
            saved,
        )
    }
    #[cfg(test)]
    pub(super) fn exhaust_qualifiers(&mut self) {
        self.next = None;
    }
    #[cfg(test)]
    pub(super) fn leave_last_qualifier(&mut self) {
        self.next = Some(u64::MAX);
    }
    #[cfg(test)]
    pub(super) fn next_qualifier(&self) -> Option<u64> {
        self.next
    }
    #[cfg(test)]
    pub(super) fn first_allocation_bytes() -> usize {
        let slots = Vec::<Option<Entry>>::with_capacity(SLOTS);
        let inverse = Vec::<Before>::with_capacity(SLOTS);
        slots.capacity() * size_of::<Option<Entry>>()
            + inverse.capacity() * size_of::<Before>()
            + size_of::<payload::Payload<AttributeValues>>()
    }
}
impl Frame {
    fn saved_capacity_bytes(&self) -> usize {
        self.saved.capacity() * size_of::<Before>()
    }
}

fn text_hash(
    value: Option<&DocumentString>,
    hasher: &mut impl Hasher,
    admission: &mut Admission<'_>,
) -> Result<usize, DomError> {
    admission.work(1)?;
    value.is_some().hash(hasher);
    let Some(value) = value else {
        return Ok(0);
    };
    value.len().hash(hasher);
    for unit in value.code_units() {
        admission.work(1)?;
        unit.hash(hasher);
    }
    Ok(value.len())
}
fn hash(entries: &[DomAttribute], admission: &mut Admission<'_>) -> Result<Option<u64>, DomError> {
    if entries.len() > MAX_ENTRIES {
        return Ok(None);
    }
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    entries.len().hash(&mut hasher);
    let mut units = 0usize;
    for entry in entries {
        admission.work(1)?;
        for text in [
            entry.name().namespace_uri(),
            entry.name().prefix(),
            Some(entry.name().local_name()),
            Some(entry.value()),
        ] {
            let prospective = units
                .checked_add(text.map_or(0, DocumentString::len))
                .ok_or(DomError::RepresentationExhausted)?;
            // Borrowed probe inputs have no cached bank eligibility receipt.
            // Check this component's O(1) extent before reading any units;
            // an oversized single component must not cause an unbounded hash.
            if prospective > MAX_UNITS {
                return Ok(None);
            }
            text_hash(text, &mut hasher, admission)?;
            units = prospective;
        }
    }
    Ok(Some(hasher.finish()))
}
fn equal(
    left: &[DomAttribute],
    right: &[DomAttribute],
    admission: &mut Admission<'_>,
) -> Result<bool, DomError> {
    admission.work(1)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for (a, b) in left.iter().zip(right) {
        admission.work(1)?;
        for (a, b) in [
            (a.name().namespace_uri(), b.name().namespace_uri()),
            (a.name().prefix(), b.name().prefix()),
            (Some(a.name().local_name()), Some(b.name().local_name())),
            (Some(a.value()), Some(b.value())),
        ] {
            admission.work(1)?;
            let (Some(a), Some(b)) = (a, b) else {
                if a.is_some() != b.is_some() {
                    return Ok(false);
                }
                continue;
            };
            if a.len() != b.len() {
                return Ok(false);
            }
            for (a, b) in a.code_units().iter().zip(b.code_units()) {
                admission.work(1)?;
                if a != b {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}
fn prospective(
    storage: &AttributeStorage,
    admission: &mut Admission<'_>,
) -> Result<custody::Usage, DomError> {
    let text = storage.text_payload();
    let mut usage = storage
        .own_usage()
        .checked_add(custody::Usage::new(text.bytes, text.units))?;
    // A snapshot may be the last strong semantic holder. Until adoption, an
    // upgrade revives the actual NameRecord targets as well as array/text.
    for entry in storage.entries() {
        admission.work(1)?;
        for atom in entry
            .name()
            .namespace()
            .into_iter()
            .chain([entry.name().local_name_atom()])
        {
            admission.work(1)?;
            usage = usage.checked_add(NameRecord::usage(atom.text())?)?;
        }
    }
    Ok(usage)
}

struct Buffers {
    slots: Option<Vec<Option<Entry>>>,
    frames: [Option<Vec<Before>>; MARKS + 1],
    debit: custody::Debit,
}
impl Session {
    pub(in crate::store) fn probe_bank(
        &self,
        entries: &[DomAttribute],
        admission: &mut Admission<'_>,
    ) -> Result<BankProbe, DomError> {
        if !self.may_attempt() {
            return Ok(BankProbe::Bypassed);
        }
        let Some(hash) = hash(entries, admission)? else {
            return Ok(BankProbe::Bypassed);
        };
        let (Some(owner), Some(id)) = (self.owner.upgrade(), self.ensure_active(admission)?) else {
            return Ok(BankProbe::Bypassed);
        };
        if !lock(&owner).mutable(id) {
            return Ok(BankProbe::Bypassed);
        }
        let start = (hash as usize % BUCKETS) * WAYS;
        for slot in start..start + WAYS {
            admission.work(1)?;
            let info = {
                let state = lock(&owner);
                if !state.mutable(id) {
                    return Ok(BankProbe::Bypassed);
                }
                state
                    .banks
                    .slots
                    .get(slot)
                    .and_then(Option::as_ref)
                    .filter(|entry| entry.hash == hash)
                    .map(|entry| entry.prospective)
            };
            let Some(usage) = info else {
                continue;
            };
            let ledger = lock(&owner).ledger.clone();
            let debit = ledger.retain(usage, &mut *admission.global)?;
            let value = {
                let state = lock(&owner);
                if !state.mutable(id) {
                    return Ok(BankProbe::Bypassed);
                }
                state
                    .banks
                    .slots
                    .get(slot)
                    .and_then(Option::as_ref)
                    .filter(|entry| entry.hash == hash && entry.prospective == usage)
                    .and_then(|entry| entry.value.upgrade())
            };
            let Some(value) = value else {
                continue;
            };
            let selected = SelectedBank(custody::Held::new(
                AttributeStorage::from_payload(value),
                debit,
            ));
            if equal(entries, selected.storage().entries(), admission)? {
                return Ok(BankProbe::Hit(selected));
            }
        }
        let state = lock(&owner);
        if !state.mutable(id) {
            return Ok(BankProbe::Bypassed);
        }
        let Some(sequence) = state.banks.next else {
            return Ok(BankProbe::Bypassed);
        };
        Ok(BankProbe::Miss(BankMiss {
            owner: state.owner,
            root: id,
            sequence,
            hash,
        }))
    }
    pub(in crate::store) fn offer_bank(
        &self,
        value: &AttributeStorage,
        miss: Option<BankMiss>,
        admission: &mut Admission<'_>,
    ) -> Result<(), DomError> {
        let Some(payload) = value.payload() else {
            return Ok(());
        };
        if value.entries().len() > MAX_ENTRIES
            || value.logical_units() > MAX_UNITS
            || value.own_usage().bytes() > MAX_BYTES
        {
            return Ok(());
        }
        if !self.may_attempt() {
            return Ok(());
        }
        let (Some(owner), Some(id)) = (self.owner.upgrade(), self.ensure_active(admission)?) else {
            return Ok(());
        };
        let (domain, sequence) = {
            let state = lock(&owner);
            if !state.mutable(id) {
                return Ok(());
            }
            let Some(sequence) = state.banks.next else {
                return Ok(());
            };
            (state.owner, sequence)
        };
        for entry in value.entries() {
            admission.work(1)?;
            if entry.name().owner() != domain {
                return Ok(());
            }
        }
        let hash = match miss {
            Some(miss) if miss.owner == domain && miss.root == id && miss.sequence == sequence => {
                miss.hash
            }
            Some(_) => return Ok(()),
            None => match hash(value.entries(), admission)? {
                Some(hash) => hash,
                None => return Ok(()),
            },
        };
        let prospective = prospective(value, admission)?;
        if prospective.bytes() > MAX_BYTES || prospective.units() > MAX_UNITS {
            return Ok(());
        }
        // Twelve fixed header passes: active-frame selection; shape capture;
        // older-frame count; need-frame construction/count; temporary header
        // initialization; reserve loop; final shape capture/comparison;
        // installation; capacity reconciliation; temporary header destruction.
        // No saved cell or spare inverse capacity is included in this package.
        admission.work(12 * FRAMES)?;
        let bucket = hash as usize % BUCKETS;
        let (frame_index, serial, saved_len) = {
            let state = lock(&owner);
            if !state.mutable(id) || state.banks.next != Some(sequence) {
                return Ok(());
            }
            let index = state.banks.frame_index();
            let frame = state.banks.root.as_ref().unwrap().frames[index]
                .as_ref()
                .unwrap();
            (index, frame.serial, frame.saved.len())
        };
        admission.work(saved_len)?;
        let (ledger, plan) = {
            let state = lock(&owner);
            if !state.mutable(id) || state.banks.next != Some(sequence) {
                return Ok(());
            }
            let root = state.banks.root.as_ref().unwrap();
            let Some(frame) = root.frames[frame_index].as_ref() else {
                return Ok(());
            };
            if frame.serial != serial || frame.saved.len() != saved_len {
                return Ok(());
            }
            let mut ancestors = 0;
            for other in root.frames.iter().flatten() {
                if other.serial > serial {
                    return Ok(());
                }
                ancestors += usize::from(other.serial < serial);
            }
            let shape = state.banks.shape(bucket);
            let slot = bucket * WAYS + usize::from(shape.cursor);
            let first_before = !frame.saved.iter().any(|old| old.slot == slot);
            let need_frames = std::array::from_fn::<_, FRAMES, _>(|i| {
                shape.frames[i].is_some_and(|(_, _, capacity)| capacity == 0)
            });
            let count = need_frames.iter().filter(|need| **need).count();
            let requested = usize::from(!shape.had_slots) * SLOTS * size_of::<Option<Entry>>()
                + count * SLOTS * size_of::<Before>();
            // Later marks have greater serials and cannot become an older
            // destination for this cell. Each accept moves only to an older
            // surviving frame, at most `ancestors` times. Parent membership
            // searches visit at most SLOTS cells; the other two visits select
            // and move/drop the incoming cell. Final restore/seal needs three.
            // An outer restore consumes descendants; a subsequently created
            // first-before cell pays its own allowance again.
            let inverse = usize::from(first_before) * (ancestors * (SLOTS + 2) + 3);
            let closing_work = inverse + 3 * SLOTS * usize::from(!shape.had_slots) + 2 * count + 4;
            (
                state.ledger.clone(),
                OfferPlan {
                    shape,
                    frame: frame_index,
                    first_before,
                    need_frames,
                    requested,
                    before: state.banks.usage(),
                    closing_work,
                },
            )
        };
        // Directory initialization, its eventual empty check/destruction and
        // each newly empty backing's allocate/release are paid once here.
        admission.work(plan.closing_work)?;
        let need_slots = !plan.shape.had_slots;
        let need_frames = plan.need_frames;
        let requested = plan.requested;
        let before_usage = plan.before;
        let weak_body = size_of::<payload::Payload<AttributeValues>>();
        if before_usage
            .bytes()
            .checked_add(requested)
            .and_then(|n| n.checked_add(weak_body))
            .ok_or(DomError::RepresentationExhausted)?
            > MAX_BYTES
        {
            return Ok(());
        }
        let debit = ledger.retain(
            custody::Usage::new(requested + weak_body, 0),
            &mut *admission.global,
        )?;
        let mut buffers = Buffers {
            slots: None,
            frames: std::array::from_fn(|_| None),
            debit,
        };
        if need_slots {
            let mut slots = Vec::new();
            slots
                .try_reserve_exact(SLOTS)
                .map_err(|_| DomError::AllocationFailed)?;
            let extra = (slots.capacity() - SLOTS) * size_of::<Option<Entry>>();
            buffers.slots = Some(slots);
            buffers
                .debit
                .grow(custody::Usage::new(extra, 0), &mut *admission.global)?;
            let slots = buffers.slots.as_mut().unwrap();
            slots.resize_with(SLOTS, || None);
            #[cfg(test)]
            {
                allocation(slots.capacity() * size_of::<Option<Entry>>());
                copied::<Option<Entry>>(SLOTS);
            }
        }
        for (i, need) in need_frames.into_iter().enumerate() {
            if need {
                let mut saved = Vec::<Before>::new();
                saved
                    .try_reserve_exact(SLOTS)
                    .map_err(|_| DomError::AllocationFailed)?;
                let extra = (saved.capacity() - SLOTS) * size_of::<Before>();
                #[cfg(test)]
                allocation(saved.capacity() * size_of::<Before>());
                buffers.frames[i] = Some(saved);
                buffers
                    .debit
                    .grow(custody::Usage::new(extra, 0), &mut *admission.global)?;
            }
        }
        let mut state = lock(&owner);
        if !state.mutable(id) {
            return Ok(());
        }
        // Admission may reenter the owner. Never consume a work/storage plan
        // after any root/frame/capacity/selected-cursor/sequence change.
        if state.banks.shape(bucket) != plan.shape {
            return Ok(());
        }
        if state.banks.issue().is_none() {
            return Ok(());
        }
        let actual = before_usage
            .bytes()
            .checked_add(buffers.debit.usage().bytes())
            .ok_or(DomError::RepresentationExhausted)?;
        if actual > MAX_BYTES {
            return Ok(());
        }
        if let Some(slots) = buffers.slots.take() {
            assert!(state.banks.slots.is_empty(), "exclusive cache preparation");
            state.banks.buffers += slots.capacity() * size_of::<Option<Entry>>();
            state.banks.slots = slots;
        }
        for (frame, saved) in state
            .banks
            .root
            .as_mut()
            .expect("root")
            .frames
            .iter_mut()
            .zip(&mut buffers.frames)
        {
            if let Some(saved) = saved.take() {
                let frame = frame.as_mut().expect("same protected mark set");
                assert_eq!(frame.saved.capacity(), 0);
                frame.saved = saved;
            }
        }
        state.banks.buffers +=
            requested - usize::from(need_slots) * SLOTS * size_of::<Option<Entry>>();
        // Correct actual allocator capacity rather than assuming reserve_exact.
        state.banks.buffers = state.banks.slots.capacity() * size_of::<Option<Entry>>()
            + state
                .banks
                .root
                .as_ref()
                .unwrap()
                .frames
                .iter()
                .flatten()
                .map(Frame::saved_capacity_bytes)
                .sum::<usize>();
        let slot = bucket * WAYS + usize::from(state.banks.cursors[bucket]);
        let index = plan.frame;
        let old = state.banks.slots[slot].take();
        let cursor = state.banks.cursors[bucket];
        let frame = state.banks.root.as_mut().unwrap().frames[index]
            .as_mut()
            .unwrap();
        if frame.cursors[bucket].is_none() {
            frame.cursors[bucket] = Some(cursor);
        }
        // Reuse the priced membership result. Successful offers advance next;
        // checkpoint changes alter the protected frame shape. The final shape
        // check above therefore preserves membership, and no callback runs
        // between that check and this mutation. Installing empty preallocated
        // vectors also cannot add or remove an initialized Before cell.
        if plan.first_before {
            frame.saved.push(Before { slot, value: old });
            #[cfg(test)]
            copied::<Before>(1);
        } else {
            state.banks.drop_entry(old);
        }
        state.banks.slots[slot] = Some(Entry {
            value: Arc::downgrade(payload),
            hash,
            prospective,
        });
        state.banks.weak += 1;
        state.banks.cursors[bucket] = (state.banks.cursors[bucket] + 1) % WAYS as u8;
        state
            .storage
            .absorb(buffers.debit)
            .expect("original pool ledger");
        state.reconcile();
        Ok(())
    }
}

#[cfg(test)]
fn copied<T>(count: usize) {
    if crate::store::scale_metrics::ENABLED && count != 0 {
        crate::store::scale_metrics::record(
            crate::store::scale_metrics::Event::SequenceMaterialized {
                bytes: size_of::<T>()
                    .checked_mul(count)
                    .expect("bounded pool copy extent"),
            },
        );
    }
}
