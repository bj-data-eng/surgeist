//! The collector's bank of recorded snapshots and the identity indexes behind it.
//!
//! The bank keeps the snapshots in the order they were first recorded. Two
//! open-addressing indexes, one by snapshot identity and one by original
//! UTF-16 backing identity (naming the first recorded snapshot that has it),
//! find an entry without scanning the bank.
//!
//! An index is a table of bank positions with linear probing. Its length is
//! the smallest power of two that is at least twice the bank's room, so a table
//! is at most half full and a probe meets a free slot after a few steps. A probe
//! starts at a fixed integer mix of the identity's address: no hash seed is
//! involved, so table lengths, reported storage and charges depend only on the
//! sequence of snapshots recorded, never on their addresses. Growing the bank
//! rebuilds the tables from it in one pass. Unlike a bank sorted for bisection,
//! recording moves no other entry and recording order is kept.

use std::sync::Arc;

use super::{CssRetainedPayloadError, CssSourceRetentionError, CssSourceRetentionResource};
use crate::source::CssSourceSnapshot;

fn storage() -> CssSourceRetentionError {
    CssSourceRetentionError::limit(CssSourceRetentionResource::Storage)
}

/// The address that identifies the snapshot `source`.
fn snapshot_address(source: &CssSourceSnapshot) -> usize {
    Arc::as_ptr(&source.0).addr()
}

/// The identity an index finds a bank entry by.
#[derive(Clone, Copy)]
enum Identity {
    /// The snapshot itself.
    Snapshot,
    /// The original UTF-16 backing, for a snapshot that has one.
    Original,
}
impl Identity {
    /// The address that identifies `source` under this identity.
    fn of(self, source: &CssSourceSnapshot) -> Option<usize> {
        match self {
            Self::Snapshot => Some(snapshot_address(source)),
            Self::Original => source
                .original_utf16()
                .map(|original| Arc::as_ptr(original).addr()),
        }
    }
}

/// A fixed integer mix of an identity (the finalizer of splitmix64), whose
/// high half selects the first slot of a probe.
fn mix(identity: usize) -> usize {
    let mut value = u64::try_from(identity).unwrap_or(u64::MAX);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    usize::try_from(value >> 32).unwrap_or(0)
}

/// An open-addressing table of bank positions.
///
/// A slot holds the position plus one of the bank entry it names, or zero
/// when it is free. An entry is found by the identity of the bank entry it
/// names, so the table stores no identity of its own.
struct Index {
    kind: Identity,
    slots: Vec<usize>,
    used: usize,
}
impl Index {
    const fn new(kind: Identity) -> Self {
        Self {
            kind,
            slots: Vec::new(),
            used: 0,
        }
    }
    /// The length of an index for a bank of `capacity` entries.
    fn length(capacity: usize) -> Result<usize, CssSourceRetentionError> {
        capacity
            .checked_mul(2)
            .and_then(usize::checked_next_power_of_two)
            .ok_or_else(storage)
    }
    /// Whether one more entry keeps the table at most half full.
    fn has_room(&self) -> bool {
        self.used < self.slots.len() / 2
    }
    /// The position of the bank entry identified by `identity`, if recorded.
    fn find(&self, identity: usize, bank: &[CssSourceSnapshot]) -> Option<usize> {
        let mask = self.slots.len().checked_sub(1)?;
        let mut at = mix(identity) & mask;
        for _ in 0..self.slots.len() {
            // A free slot (zero) ends the probe: it names no position.
            let position = self.slots.get(at)?.checked_sub(1)?;
            if bank
                .get(position)
                .and_then(|entry| self.kind.of(entry))
                .is_some_and(|recorded| recorded == identity)
            {
                return Some(position);
            }
            // `at <= mask`, and the mask drops the carry: modular on the table length.
            at = at.wrapping_add(1) & mask;
        }
        None
    }
    /// Forgets every entry, keeping the slots.
    fn clear(&mut self) {
        if self.used > 0 {
            self.slots.fill(0);
            self.used = 0;
        }
    }
    /// Names the bank entry at `position`, whose identity is `identity`.
    fn insert(&mut self, identity: usize, position: usize) -> Result<(), CssSourceRetentionError> {
        if !self.has_room() {
            return Err(storage());
        }
        let mask = self.slots.len().checked_sub(1).ok_or_else(storage)?;
        let named = position.checked_add(1).ok_or_else(storage)?;
        let used = self.used.checked_add(1).ok_or_else(storage)?;
        let mut at = mix(identity) & mask;
        for _ in 0..self.slots.len() {
            let slot = self.slots.get_mut(at).ok_or_else(storage)?;
            if *slot == 0 {
                *slot = named;
                self.used = used;
                return Ok(());
            }
            // `at <= mask`, and the mask drops the carry: modular on the table length.
            at = at.wrapping_add(1) & mask;
        }
        Err(storage())
    }
}

/// The storage a growth step replaces: element counts, `None` where the
/// storage held is kept.
#[derive(Clone, Copy)]
pub(super) struct Capacities {
    sources: Option<usize>,
    snapshots: Option<usize>,
    originals: Option<usize>,
}

/// One growth step the bank needs before it can record another snapshot.
#[derive(Clone, Copy)]
pub(super) struct Growth {
    /// The number of snapshots the bank holds once the step is done.
    room: usize,
    replace_sources: bool,
    snapshots: Option<usize>,
    originals: Option<usize>,
}
impl Growth {
    /// The element counts the step requests.
    pub(super) fn requested(&self) -> Capacities {
        Capacities {
            sources: self.replace_sources.then_some(self.room),
            snapshots: self.snapshots,
            originals: self.originals,
        }
    }
}

/// Storage allocated for a growth step and not yet in use.
pub(super) struct Replacement {
    room: usize,
    sources: Option<Vec<CssSourceSnapshot>>,
    snapshots: Option<Vec<usize>>,
    originals: Option<Vec<usize>>,
}
impl Replacement {
    /// The element counts the allocator actually provided.
    pub(super) fn actual(&self) -> Capacities {
        Capacities {
            sources: self.sources.as_ref().map(Vec::capacity),
            snapshots: self.snapshots.as_ref().map(Vec::capacity),
            originals: self.originals.as_ref().map(Vec::capacity),
        }
    }
    /// The number of snapshots the replacement bank can hold, when it replaces the bank.
    pub(super) fn bank_capacity(&self) -> Option<usize> {
        self.sources.as_ref().map(Vec::capacity)
    }
}

fn reserve<T>(length: usize) -> Result<Vec<T>, CssSourceRetentionError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(length)
        .map_err(|_| CssSourceRetentionError::allocation())?;
    Ok(values)
}

/// The recorded snapshots in recording order, with their identity indexes.
pub(super) struct SourceBank {
    sources: Vec<CssSourceSnapshot>,
    /// The number of snapshots the bank and its indexes are sized for.
    room: usize,
    snapshots: Index,
    originals: Index,
}
impl SourceBank {
    pub(super) const fn new() -> Self {
        Self {
            sources: Vec::new(),
            room: 0,
            snapshots: Index::new(Identity::Snapshot),
            originals: Index::new(Identity::Original),
        }
    }
    pub(super) const fn len(&self) -> usize {
        self.sources.len()
    }
    /// The largest bank whose storage `clear` keeps: a larger one is released.
    const KEPT: usize = 256;
    /// Forgets the snapshots recorded, keeping the storage of a small bank so
    /// that the next snapshots are recorded without allocating.
    pub(super) fn clear(&mut self) {
        if self.room > Self::KEPT {
            *self = Self::new();
            return;
        }
        self.sources.clear();
        self.snapshots.clear();
        self.originals.clear();
    }
    /// The recorded snapshots, in the order they were first recorded.
    pub(super) fn recorded(&self) -> &[CssSourceSnapshot] {
        &self.sources
    }
    #[cfg(test)]
    pub(super) fn capacity(&self) -> usize {
        self.sources.capacity()
    }
    /// Whether `source` is already recorded, by snapshot identity.
    pub(super) fn contains(&self, source: &CssSourceSnapshot) -> bool {
        self.snapshots
            .find(snapshot_address(source), &self.sources)
            .is_some()
    }
    /// The position of the first recorded snapshot that shares the original
    /// UTF-16 backing of `source`.
    pub(super) fn first_sharing_original(&self, source: &CssSourceSnapshot) -> Option<usize> {
        let identity = Identity::Original.of(source)?;
        self.originals.find(identity, &self.sources)
    }
    /// The step, if any, that must precede recording `incoming`.
    ///
    /// A bank at its limit doubles, within `maximum` snapshots; the indexes
    /// follow the bank, and the original index is allocated with the first
    /// snapshot that has an original backing.
    pub(super) fn growth(
        &self,
        incoming: &CssSourceSnapshot,
        maximum: usize,
    ) -> Result<Option<Growth>, CssSourceRetentionError> {
        let length = self.sources.len();
        let replace_sources = length >= self.room;
        let room = if replace_sources {
            let required = length.checked_add(1).ok_or_else(storage)?;
            let room = self.room.saturating_mul(2).max(required).min(maximum);
            if room < required {
                return Err(storage());
            }
            room
        } else {
            self.room
        };
        let slots = Index::length(room)?;
        let snapshots = (self.snapshots.slots.len() < slots).then_some(slots);
        let originals = (self.originals.used > 0 || Identity::Original.of(incoming).is_some())
            .then_some(slots)
            .filter(|slots| self.originals.slots.len() < *slots);
        if !replace_sources && snapshots.is_none() && originals.is_none() {
            return Ok(None);
        }
        Ok(Some(Growth {
            room,
            replace_sources,
            snapshots,
            originals,
        }))
    }
    /// The live bytes of the bank's storage while `replaced` is allocated beside
    /// what it replaces: the storage held, and for each storage allocated a
    /// replacement of that many elements and its temporary header.
    pub(super) fn live_bytes(
        &self,
        replaced: Capacities,
    ) -> Result<usize, CssSourceRetentionError> {
        let part = |held: usize, element: usize, new: Option<usize>, header: usize| {
            let held = held.checked_mul(element)?;
            match new {
                None => Some(held),
                Some(count) => held
                    .checked_add(count.checked_mul(element)?)?
                    .checked_add(header),
            }
        };
        part(
            self.sources.capacity(),
            std::mem::size_of::<CssSourceSnapshot>(),
            replaced.sources,
            std::mem::size_of::<Vec<CssSourceSnapshot>>(),
        )
        .and_then(|bytes| {
            bytes.checked_add(part(
                self.snapshots.slots.capacity(),
                std::mem::size_of::<usize>(),
                replaced.snapshots,
                std::mem::size_of::<Vec<usize>>(),
            )?)
        })
        .and_then(|bytes| {
            bytes.checked_add(part(
                self.originals.slots.capacity(),
                std::mem::size_of::<usize>(),
                replaced.originals,
                std::mem::size_of::<Vec<usize>>(),
            )?)
        })
        .ok_or_else(storage)
    }
    /// Allocates the storage of `growth` without using it.
    pub(super) fn allocate(&self, growth: &Growth) -> Result<Replacement, CssSourceRetentionError> {
        let table = |slots: Option<usize>| {
            slots
                .map(|slots| {
                    let mut table = reserve::<usize>(slots)?;
                    table.resize(slots, 0);
                    Ok(table)
                })
                .transpose()
        };
        Ok(Replacement {
            room: growth.room,
            sources: growth
                .replace_sources
                .then(|| reserve(growth.room))
                .transpose()?,
            snapshots: table(growth.snapshots)?,
            originals: table(growth.originals)?,
        })
    }
    /// Moves the snapshots into the replacement storage and rebuilds the
    /// indexes it replaces from them. It allocates nothing.
    pub(super) fn adopt(
        &mut self,
        replacement: Replacement,
    ) -> Result<(), CssSourceRetentionError> {
        let Replacement {
            room,
            sources,
            snapshots,
            originals,
        } = replacement;
        if sources
            .as_ref()
            .is_some_and(|sources| sources.capacity() < self.sources.len())
        {
            return Err(storage());
        }
        let snapshots = snapshots
            .map(|slots| self.rebuilt(Identity::Snapshot, slots))
            .transpose()?;
        let originals = originals
            .map(|slots| self.rebuilt(Identity::Original, slots))
            .transpose()?;
        if let Some(snapshots) = snapshots {
            self.snapshots = snapshots;
        }
        if let Some(originals) = originals {
            self.originals = originals;
        }
        if let Some(mut sources) = sources {
            sources.append(&mut self.sources);
            self.sources = sources;
        }
        self.room = room;
        Ok(())
    }
    /// An index of the snapshots recorded, over `slots` zeroed slots; the
    /// first recorded snapshot names each original backing.
    fn rebuilt(&self, kind: Identity, slots: Vec<usize>) -> Result<Index, CssSourceRetentionError> {
        let mut index = Index {
            kind,
            slots,
            used: 0,
        };
        for (position, source) in self.sources.iter().enumerate() {
            let Some(identity) = kind.of(source) else {
                continue;
            };
            if index.find(identity, &self.sources).is_none() {
                index.insert(identity, position)?;
            }
        }
        Ok(index)
    }
    /// Records `source`, which the bank does not hold, after `growth` has
    /// made room for it.
    pub(super) fn record(
        &mut self,
        source: &CssSourceSnapshot,
    ) -> Result<(), CssSourceRetentionError> {
        let position = self.sources.len();
        let original = Identity::Original
            .of(source)
            .filter(|identity| self.originals.find(*identity, &self.sources).is_none());
        if position >= self.room
            || position >= self.sources.capacity()
            || !self.snapshots.has_room()
            || (original.is_some() && !self.originals.has_room())
        {
            return Err(storage());
        }
        self.sources.push(source.clone());
        self.snapshots.insert(snapshot_address(source), position)?;
        if let Some(identity) = original {
            self.originals.insert(identity, position)?;
        }
        Ok(())
    }
    /// The complete payload of the recorded snapshots, each shared original
    /// UTF-16 backing once; see [`CssSourceRetention::complete_payload_bytes_with`].
    pub(super) fn payload_bytes<E>(
        &self,
        admit: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<usize, CssRetainedPayloadError<E>> {
        admit().map_err(CssRetainedPayloadError::admission)?;
        let mut total = 0usize;
        for (position, source) in self.sources.iter().enumerate() {
            admit().map_err(CssRetainedPayloadError::admission)?;
            let mut include_original = true;
            if source.original_utf16().is_some() {
                admit().map_err(CssRetainedPayloadError::admission)?;
                include_original = self
                    .first_sharing_original(source)
                    .is_none_or(|first| first == position);
            }
            admit().map_err(CssRetainedPayloadError::admission)?;
            total = total
                .checked_add(
                    source
                        .payload_bytes(include_original)
                        .ok_or_else(CssRetainedPayloadError::capacity_overflow)?,
                )
                .ok_or_else(CssRetainedPayloadError::capacity_overflow)?;
        }
        Ok(total)
    }
}
