//! The measurement of one retained payload and the ledger of what measurements retain.
//!
//! A [`CssSourceRetention`] collector accounts for everything it visits at once:
//! its steps, the nesting it reaches and the distinct snapshots it records.
//! [`CssSourceMeasurement`] is the same account of one payload alone, a value that
//! can be kept, compared and added to the account of another payload. A
//! [`CssSourceLedger`] totals measurements and gives the bytes the collector
//! would count for all of them, with the rule the collector applies: the bytes
//! of a snapshot once, and of a shared original UTF-16 backing once however
//! many distinct snapshots share it.
//!
//! The totals of a ledger do not depend on the order measurements were added or
//! removed in, so a ledger can follow a state that changes a payload at a time.
//! [`CssSourceLedger::replay`] answers what the collector would have refused
//! and after how many steps when a walk the ledger does not admit is run on top
//! of everything the ledger holds.
use super::{CssSourceRetention, CssSourceRetentionError, CssSourceRetentionResource};
use crate::source::CssSourceSnapshot;
use std::collections::HashMap;
use std::sync::Arc;

/// The bounds of a collector: retained bytes, steps and nesting depth.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssSourceBounds {
    bytes: usize,
    steps: usize,
    depth: usize,
}

impl CssSourceBounds {
    /// The bounds `bytes`, `steps` and `depth`, as [`CssSourceRetention::with_shared_original_utf16`] takes them.
    #[must_use]
    pub const fn new(bytes: usize, steps: usize, depth: usize) -> Self {
        Self {
            bytes,
            steps,
            depth,
        }
    }
    /// Returns the bound on retained bytes.
    #[must_use]
    pub const fn bytes(&self) -> usize {
        self.bytes
    }
    /// Returns the bound on charged steps.
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.steps
    }
    /// Returns the bound on nesting depth.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }
}

/// The identity of a snapshot, as the collector recognizes it.
fn identity(snapshot: &CssSourceSnapshot) -> usize {
    Arc::as_ptr(&snapshot.0).addr()
}

/// The original UTF-16 backing a snapshot shares: its identity and bytes.
#[derive(Clone, Copy, Debug)]
struct Backing {
    identity: usize,
    bytes: usize,
}

/// One distinct snapshot a measurement retains.
///
/// It keeps its snapshot alive, so the identity it was measured under cannot
/// be reused by another snapshot while the value exists.
#[derive(Clone, Debug)]
struct CssMeasuredSource {
    snapshot: CssSourceSnapshot,
    bytes: usize,
    backing: Option<Backing>,
}

impl CssMeasuredSource {
    fn new(snapshot: &CssSourceSnapshot) -> Self {
        Self {
            snapshot: snapshot.clone(),
            bytes: snapshot.storage_usage().retained_bytes(),
            backing: snapshot.original_utf16().map(|original| Backing {
                identity: Arc::as_ptr(original).addr(),
                bytes: original.capacity().saturating_mul(size_of::<u16>()),
            }),
        }
    }
}

/// The steps, nesting and distinct snapshots of one payload.
///
/// A measurement is made at a base depth and reports the nesting below it. It
/// does not depend on any other payload, so it can be kept and added to a
/// [`CssSourceLedger`].
#[derive(Clone, Debug)]
pub struct CssSourceMeasurement {
    steps: usize,
    depth: usize,
    sources: Vec<CssMeasuredSource>,
}

impl CssSourceMeasurement {
    /// Measures what `walk` retains, run on a collector of `bounds` positioned at `base_depth`.
    ///
    /// The retained bytes of `bounds` bound the snapshots `walk` records alone.
    /// A measurement is complete: the steps it reports are the steps the walk
    /// charged, whatever else a collector has recorded.
    ///
    /// # Errors
    ///
    /// The first refusal the collector reports, unchanged. A refused payload has no measurement.
    pub fn of(
        bounds: CssSourceBounds,
        base_depth: usize,
        walk: impl FnOnce(&mut CssSourceRetention<'_>) -> Result<(), CssSourceRetentionError>,
    ) -> Result<Self, CssSourceRetentionError> {
        let mut collector = CssSourceRetention::with_shared_original_utf16(
            bounds.bytes,
            bounds.steps,
            bounds.depth,
        );
        collector.at_depth(base_depth, walk)?;
        Self::of_collector(&collector, 0, base_depth)
    }

    /// The measurement of what `collector` recorded after its first `skipped` snapshots.
    fn of_collector(
        collector: &CssSourceRetention<'_>,
        skipped: usize,
        base_depth: usize,
    ) -> Result<Self, CssSourceRetentionError> {
        let recorded = collector.bank.recorded().get(skipped..).unwrap_or_default();
        let mut sources = Vec::new();
        sources
            .try_reserve_exact(recorded.len())
            .map_err(|_| CssSourceRetentionError::allocation())?;
        sources.extend(recorded.iter().map(CssMeasuredSource::new));
        Ok(Self {
            steps: collector.visits.saturating_sub(skipped),
            depth: collector.deepest().saturating_sub(base_depth),
            sources,
        })
    }

    /// Returns the steps charged.
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.steps
    }
    /// Returns the deepest nesting reached below the base depth.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// The distinct snapshots retained, in the order they were first recorded.
    #[cfg(test)]
    fn sources(&self) -> &[CssMeasuredSource] {
        &self.sources
    }
}

/// A collector that measures one payload after another, reusing its storage.
///
/// [`CssSourceMeasurement::of`] builds a collector for every payload it
/// measures. A meter keeps one, clears it before each payload and lets the
/// result be read, or added to a [`CssSourceLedger`], without storing it: a
/// run of payloads that record few snapshots measures without allocating.
/// The storage a meter keeps is that of the largest payload it measured, up to a
/// bank of 256 snapshots; the storage of a larger one is released before the
/// next payload.
#[derive(Debug)]
pub struct CssSourceMeter {
    collector: CssSourceRetention<'static>,
    base_depth: usize,
}

impl CssSourceMeter {
    /// A meter whose payloads are measured under `bounds`.
    #[must_use]
    pub const fn new(bounds: CssSourceBounds) -> Self {
        Self {
            collector: CssSourceRetention::with_shared_original_utf16(
                bounds.bytes,
                bounds.steps,
                bounds.depth,
            ),
            base_depth: 0,
        }
    }

    /// Measures what `walk` retains, run at `base_depth`, in place of the
    /// previous measurement.
    ///
    /// Nothing recorded by an earlier measurement counts as recorded: each
    /// payload is measured alone, and sharing between payloads is the
    /// ledger's to count.
    ///
    /// # Errors
    ///
    /// The first refusal the collector reports, unchanged. The meter is left
    /// empty and can measure the next payload.
    pub fn measure(
        &mut self,
        base_depth: usize,
        walk: impl FnOnce(&mut CssSourceRetention<'_>) -> Result<(), CssSourceRetentionError>,
    ) -> Result<(), CssSourceRetentionError> {
        self.collector.clear();
        self.base_depth = base_depth;
        let result = self.collector.at_depth(base_depth, walk);
        if result.is_err() {
            self.collector.clear();
        }
        result
    }

    /// Returns the steps the last measurement charged.
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.collector.visits
    }

    /// Returns the deepest nesting the last measurement reached below its base depth.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.collector.deepest().saturating_sub(self.base_depth)
    }

    /// The distinct snapshots the last measurement recorded, in the order they were first recorded.
    fn recorded(&self) -> &[CssSourceSnapshot] {
        self.collector.bank.recorded()
    }

    /// The last measurement as a value.
    #[cfg(test)]
    fn to_measurement(&self) -> Result<CssSourceMeasurement, CssSourceRetentionError> {
        CssSourceMeasurement::of_collector(&self.collector, 0, self.base_depth)
    }
}

/// A snapshot held by a ledger and the number of measurements that hold it.
#[derive(Debug)]
struct Held {
    snapshot: CssSourceSnapshot,
    references: u32,
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

/// The distinct snapshots of many measurements, counted by reference, and the
/// retained bytes the collector counts for them.
///
/// The bytes are those of each distinct snapshot, less the bytes of an
/// original UTF-16 backing for every snapshot sharing it beyond the first.
/// They are the same for every order of [`Self::add`] and [`Self::remove`]
/// that leaves the same measurements added.
///
/// A snapshot costs the ledger 16 bytes of entry and 8 bytes of index, as many as the
/// collector's bank and index cost for it. The index is an open-addressing
/// table of entry positions, at most half full, probed linearly from a fixed
/// integer mix of the snapshot's address; a snapshot that leaves is removed
/// by shifting back the entries that follow it in its probe.
#[derive(Debug, Default)]
pub struct CssSourceLedger {
    entries: Vec<Held>,
    /// Entry positions plus one, or zero for a free slot; its length is zero or a power of two.
    slots: Vec<u32>,
    /// The number of held snapshots sharing each original UTF-16 backing.
    backings: HashMap<usize, usize>,
    bytes: usize,
}

fn allocation() -> CssSourceRetentionError {
    CssSourceRetentionError::allocation()
}

impl CssSourceLedger {
    /// An empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the retained bytes of the distinct snapshots held, the shared backings counted once.
    #[must_use]
    pub const fn bytes(&self) -> usize {
        self.bytes
    }

    /// The number of distinct snapshots held.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the ledger holds no snapshot.
    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Adds the snapshots of `measurement`, each reference counted.
    ///
    /// # Errors
    ///
    /// [`CssSourceRetentionResource::RetainedBytes`] when the bytes overflow `usize`, and
    /// [`CssSourceRetentionError::allocation`] when a snapshot cannot be stored. The ledger
    /// is unchanged.
    pub fn add(
        &mut self,
        measurement: &CssSourceMeasurement,
    ) -> Result<(), CssSourceRetentionError> {
        for (added, source) in measurement.sources.iter().enumerate() {
            if let Err(error) = self.add_source(source) {
                for source in measurement.sources.get(..added).unwrap_or_default() {
                    self.remove_source(source);
                }
                return Err(error);
            }
        }
        Ok(())
    }

    /// Adds the snapshots `meter` recorded, each reference counted.
    ///
    /// # Errors
    ///
    /// As [`Self::add`].
    pub fn add_metered(&mut self, meter: &CssSourceMeter) -> Result<(), CssSourceRetentionError> {
        let recorded = meter.recorded();
        for (added, snapshot) in recorded.iter().enumerate() {
            if let Err(error) = self.add_source(&CssMeasuredSource::new(snapshot)) {
                for snapshot in recorded.get(..added).unwrap_or_default() {
                    self.remove_source(&CssMeasuredSource::new(snapshot));
                }
                return Err(error);
            }
        }
        Ok(())
    }

    /// Removes the snapshots `meter` recorded, which an earlier [`Self::add_metered`] added.
    pub fn remove_metered(&mut self, meter: &CssSourceMeter) {
        for snapshot in meter.recorded() {
            self.remove_source(&CssMeasuredSource::new(snapshot));
        }
    }

    /// The slot and entry position of the snapshot identified by `key`.
    fn find(&self, key: usize) -> Option<(usize, usize)> {
        let mask = self.slots.len().checked_sub(1)?;
        let mut at = mix(key) & mask;
        for _ in 0..self.slots.len() {
            let position = usize::try_from(*self.slots.get(at)?).ok()?.checked_sub(1)?;
            if self
                .entries
                .get(position)
                .is_some_and(|held| identity(&held.snapshot) == key)
            {
                return Some((at, position));
            }
            at = at.wrapping_add(1) & mask;
        }
        None
    }

    /// Names the entry at `position`, whose identity is `key`, in the first
    /// free slot of its probe.
    fn name(slots: &mut [u32], key: usize, position: usize) -> Result<(), CssSourceRetentionError> {
        let mask = slots.len().checked_sub(1).ok_or_else(allocation)?;
        let named = u32::try_from(position.checked_add(1).ok_or_else(allocation)?)
            .map_err(|_| allocation())?;
        let mut at = mix(key) & mask;
        for _ in 0..slots.len() {
            let slot = slots.get_mut(at).ok_or_else(allocation)?;
            if *slot == 0 {
                *slot = named;
                return Ok(());
            }
            at = at.wrapping_add(1) & mask;
        }
        Err(allocation())
    }

    /// Makes room for one more entry, doubling the entries and the index.
    fn grow(&mut self) -> Result<(), CssSourceRetentionError> {
        let room = self.entries.len().saturating_add(1);
        if room <= self.entries.capacity() && room.saturating_mul(2) <= self.slots.len() {
            return Ok(());
        }
        let capacity = self.entries.capacity().saturating_mul(2).max(4).max(room);
        let length = capacity
            .checked_mul(2)
            .and_then(usize::checked_next_power_of_two)
            .ok_or_else(allocation)?;
        let mut slots = Vec::new();
        slots.try_reserve_exact(length).map_err(|_| allocation())?;
        slots.resize(length, 0);
        self.entries
            .try_reserve_exact(capacity - self.entries.len())
            .map_err(|_| allocation())?;
        for (position, held) in self.entries.iter().enumerate() {
            Self::name(&mut slots, identity(&held.snapshot), position)?;
        }
        self.slots = slots;
        Ok(())
    }

    fn add_source(&mut self, source: &CssMeasuredSource) -> Result<(), CssSourceRetentionError> {
        let key = identity(&source.snapshot);
        if let Some((_, position)) = self.find(key) {
            let held = self.entries.get_mut(position).ok_or_else(allocation)?;
            held.references =
                held.references
                    .checked_add(1)
                    .ok_or(CssSourceRetentionError::limit(
                        CssSourceRetentionResource::RetainedBytes,
                    ))?;
            return Ok(());
        }
        // The bytes of a backing another snapshot already shares are not counted again.
        let shared = source
            .backing
            .filter(|backing| self.backings.contains_key(&backing.identity));
        let bytes = self
            .bytes
            .checked_add(
                source
                    .bytes
                    .saturating_sub(shared.map_or(0, |backing| backing.bytes)),
            )
            .ok_or(CssSourceRetentionError::limit(
                CssSourceRetentionResource::RetainedBytes,
            ))?;
        if source.backing.is_some() {
            self.backings.try_reserve(1).map_err(|_| allocation())?;
        }
        self.grow()?;
        let position = self.entries.len();
        Self::name(&mut self.slots, key, position)?;
        self.entries.push(Held {
            snapshot: source.snapshot.clone(),
            references: 1,
        });
        if let Some(backing) = source.backing {
            let users = self.backings.entry(backing.identity).or_insert(0);
            *users = users.saturating_add(1);
        }
        self.bytes = bytes;
        Ok(())
    }

    /// Removes the snapshots of `measurement`, which an earlier [`Self::add`] added.
    ///
    /// A snapshot leaves the ledger when its last reference does. A snapshot the ledger
    /// does not hold is ignored.
    pub fn remove(&mut self, measurement: &CssSourceMeasurement) {
        for source in &measurement.sources {
            self.remove_source(source);
        }
    }

    fn remove_source(&mut self, source: &CssMeasuredSource) {
        let key = identity(&source.snapshot);
        let Some((slot, position)) = self.find(key) else {
            return;
        };
        let Some(held) = self.entries.get_mut(position) else {
            return;
        };
        held.references = held.references.saturating_sub(1);
        if held.references > 0 {
            return;
        }
        self.unname(slot);
        // The last entry takes the place of the removed one.
        let last = self.entries.len().saturating_sub(1);
        if position != last {
            self.rename(last, position);
        }
        let removed = CssMeasuredSource::new(&self.entries.swap_remove(position).snapshot);
        let mut still_shared = 0;
        if let Some(backing) = removed.backing
            && let Some(users) = self.backings.get_mut(&backing.identity)
        {
            *users = users.saturating_sub(1);
            if *users == 0 {
                self.backings.remove(&backing.identity);
            } else {
                still_shared = backing.bytes;
            }
        }
        self.bytes = self
            .bytes
            .saturating_sub(removed.bytes.saturating_sub(still_shared));
    }

    /// Frees `slot`, shifting back the entries that follow it in their probes
    /// so that none is left beyond a free slot.
    fn unname(&mut self, slot: usize) {
        let Some(mask) = self.slots.len().checked_sub(1) else {
            return;
        };
        let mut hole = slot;
        let mut next = slot.wrapping_add(1) & mask;
        for _ in 0..self.slots.len() {
            let Some(position) = self
                .slots
                .get(next)
                .and_then(|value| usize::try_from(*value).ok())
                .and_then(|value| value.checked_sub(1))
            else {
                break;
            };
            let Some(held) = self.entries.get(position) else {
                break;
            };
            let home = mix(identity(&held.snapshot)) & mask;
            // The entry moves into the hole when the hole is in its probe, before it.
            if next.wrapping_sub(home) & mask >= next.wrapping_sub(hole) & mask {
                let named = self.slots.get(next).copied().unwrap_or(0);
                if let Some(target) = self.slots.get_mut(hole) {
                    *target = named;
                }
                hole = next;
            }
            next = next.wrapping_add(1) & mask;
        }
        if let Some(target) = self.slots.get_mut(hole) {
            *target = 0;
        }
    }

    /// Names the entry at `to` in the slot that names the entry at `from`.
    fn rename(&mut self, from: usize, to: usize) {
        let Some(key) = self.entries.get(from).map(|held| identity(&held.snapshot)) else {
            return;
        };
        let Some(mask) = self.slots.len().checked_sub(1) else {
            return;
        };
        let (Some(old), Some(new)) = (
            u32::try_from(from.saturating_add(1)).ok(),
            u32::try_from(to.saturating_add(1)).ok(),
        ) else {
            return;
        };
        let mut at = mix(key) & mask;
        for _ in 0..self.slots.len() {
            let Some(slot) = self.slots.get_mut(at) else {
                return;
            };
            if *slot == old {
                *slot = new;
                return;
            }
            at = at.wrapping_add(1) & mask;
        }
    }

    /// The distinct snapshots held, in no particular order.
    fn snapshots(&self) -> impl Iterator<Item = &CssSourceSnapshot> {
        self.entries.iter().map(|held| &held.snapshot)
    }

    /// Runs `walk` as a collector that has recorded every snapshot of this
    /// ledger would run it, and reports the steps it took and how it ended.
    ///
    /// `bounds` are the bounds of the whole account: the retained bytes the
    /// ledger and the walk may hold together, the steps the walk may still
    /// charge (the steps already charged are not counted) and the depth. The
    /// answer is that of a collector that had visited the ledger's payloads
    /// before `walk`: a refusal arrives at the same step, with the same resource.
    #[must_use]
    pub fn replay(
        &self,
        bounds: CssSourceBounds,
        base_depth: usize,
        walk: impl FnOnce(&mut CssSourceRetention<'_>) -> Result<(), CssSourceRetentionError>,
    ) -> CssSourceReplay {
        let held = self.entries.len();
        let mut collector = CssSourceRetention::with_shared_original_utf16(
            bounds.bytes,
            bounds.steps.saturating_add(held),
            bounds.depth,
        );
        for snapshot in self.snapshots() {
            if let Err(error) = collector.source(snapshot) {
                return CssSourceReplay {
                    steps: 0,
                    result: Err(error),
                };
            }
        }
        let result = collector
            .at_depth(base_depth, walk)
            .and_then(|()| CssSourceMeasurement::of_collector(&collector, held, base_depth));
        CssSourceReplay {
            steps: collector.visits.saturating_sub(held),
            result,
        }
    }
}

/// How a walk run on top of a ledger ended: see [`CssSourceLedger::replay`].
#[derive(Clone, Debug)]
pub struct CssSourceReplay {
    steps: usize,
    result: Result<CssSourceMeasurement, CssSourceRetentionError>,
}

impl CssSourceReplay {
    /// Returns the steps the walk charged, the one a limit refused not counted.
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.steps
    }
    /// Returns the refusal, or the snapshots the walk recorded that the ledger did not hold.
    pub const fn result(&self) -> &Result<CssSourceMeasurement, CssSourceRetentionError> {
        &self.result
    }
}

#[cfg(test)]
#[path = "measure_tests.rs"]
mod tests;

#[cfg(test)]
impl CssSourceLedger {
    /// Panics unless every held snapshot is found where the index says it is, and
    /// the index names nothing else.
    fn assert_consistent(&self) {
        for (position, held) in self.entries.iter().enumerate() {
            assert_eq!(
                self.find(identity(&held.snapshot)).map(|(_, found)| found),
                Some(position)
            );
        }
        let named = self.slots.iter().filter(|slot| **slot != 0).count();
        assert_eq!(named, self.entries.len());
        assert!(self.slots.len() >= self.entries.len() * 2);
    }
}

#[cfg(test)]
mod index_tests {
    use super::*;

    fn snapshots(count: usize) -> Vec<CssSourceSnapshot> {
        (0..count)
            .map(|index| {
                crate::CssParser::default()
                    .parse_sheet(format!(".s{index} {{ color: red }}"))
                    .syntax()
                    .parse_source()
                    .unwrap()
                    .source()
                    .clone()
            })
            .collect()
    }

    fn one(snapshot: &CssSourceSnapshot) -> CssSourceMeasurement {
        CssSourceMeasurement::of(
            CssSourceBounds::new(usize::MAX, usize::MAX, 4),
            0,
            |sources| sources.source(snapshot),
        )
        .unwrap()
    }

    #[test]
    fn the_index_finds_every_snapshot_after_any_additions_and_removals() {
        let pool = snapshots(700);
        let measurements = pool.iter().map(one).collect::<Vec<_>>();
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            usize::try_from(seed % 700).unwrap()
        };
        let mut ledger = CssSourceLedger::new();
        let mut references = vec![0usize; 700];
        for round in 0..4000 {
            let index = next();
            // Grow first, then churn, then drain.
            let remove = round > 700 && references[index] > 0 && (round % 3 != 0 || round > 3000);
            if remove {
                ledger.remove(&measurements[index]);
                references[index] -= 1;
            } else {
                ledger.add(&measurements[index]).unwrap();
                references[index] += 1;
            }
            if round % 97 == 0 {
                ledger.assert_consistent();
            }
        }
        ledger.assert_consistent();
        let held = references.iter().filter(|count| **count > 0).count();
        assert_eq!(ledger.len(), held);
        let expected = references
            .iter()
            .zip(&pool)
            .filter(|(count, _)| **count > 0)
            .map(|(_, snapshot)| snapshot.storage_usage().retained_bytes())
            .sum::<usize>();
        assert_eq!(ledger.bytes(), expected);
        for (index, count) in references.iter().enumerate() {
            for _ in 0..*count {
                ledger.remove(&measurements[index]);
            }
        }
        ledger.assert_consistent();
        assert!(ledger.is_empty());
        assert_eq!(ledger.bytes(), 0);
    }
}

#[cfg(test)]
mod meter_tests {
    use super::*;
    use crate::source::source_retention::CssRetainedSources;

    struct Records<'a>(&'a [&'a CssSourceSnapshot]);
    impl CssRetainedSources for Records<'_> {
        fn visit_retained_sources(
            &self,
            sources: &mut CssSourceRetention<'_>,
        ) -> Result<(), CssSourceRetentionError> {
            for snapshot in self.0 {
                sources.source(snapshot)?;
            }
            Ok(())
        }
    }

    fn snapshot(original: &Arc<Vec<u16>>) -> CssSourceSnapshot {
        crate::CssParser::default()
            .parse_sheet_utf16(
                original.clone(),
                crate::source::CssUtf16InputLimits::default(),
                || Ok::<_, ()>(()),
            )
            .unwrap()
            .syntax()
            .parse_source()
            .unwrap()
            .source()
            .clone()
    }

    const BOUNDS: CssSourceBounds = CssSourceBounds::new(usize::MAX, usize::MAX, 16);

    #[test]
    fn a_metered_payload_is_the_measurement_of_that_payload() {
        let original = Arc::new(".a { color: red }".encode_utf16().collect::<Vec<_>>());
        let (first, second) = (snapshot(&original), snapshot(&original));
        let walk = |sources: &mut CssSourceRetention<'_>| {
            sources.visit(&Records(&[&first, &second, &first]))
        };
        let mut meter = CssSourceMeter::new(BOUNDS);
        meter.measure(2, walk).unwrap();
        let alone = CssSourceMeasurement::of(BOUNDS, 2, walk).unwrap();
        assert_eq!(meter.steps(), alone.steps());
        assert_eq!(meter.depth(), alone.depth());
        let kept = meter.to_measurement().unwrap();
        assert_eq!(kept.steps(), alone.steps());
        assert_eq!(kept.sources().len(), 2);
        let (mut by_meter, mut by_value) = (CssSourceLedger::new(), CssSourceLedger::new());
        by_meter.add_metered(&meter).unwrap();
        by_value.add(&alone).unwrap();
        assert_eq!(by_meter.bytes(), by_value.bytes());
        by_meter.remove_metered(&meter);
        assert!(by_meter.is_empty());
        assert_eq!(by_meter.bytes(), 0);
    }

    #[test]
    fn payloads_measured_in_turn_share_nothing_but_the_ledger_counts_a_shared_backing_once() {
        let original = Arc::new(".s { margin: 1px }".encode_utf16().collect::<Vec<_>>());
        let backing = original.capacity() * size_of::<u16>();
        let (left, right) = (snapshot(&original), snapshot(&original));
        for order in [[&left, &right], [&right, &left]] {
            let mut meter = CssSourceMeter::new(BOUNDS);
            let mut ledger = CssSourceLedger::new();
            let mut figures = Vec::new();
            for snapshot in order {
                meter
                    .measure(0, |sources| sources.visit(&Records(&[snapshot])))
                    .unwrap();
                // The earlier payload's snapshot and backing are not recorded here.
                assert_eq!(meter.recorded().len(), 1);
                assert_eq!(meter.steps(), 2);
                let mut alone = CssSourceLedger::new();
                alone.add_metered(&meter).unwrap();
                figures.push(alone.bytes());
                ledger.add_metered(&meter).unwrap();
            }
            // Each payload counts the backing in its own figure.
            assert_eq!(figures[0], order[0].storage_usage().retained_bytes());
            assert_eq!(figures[1], order[1].storage_usage().retained_bytes());
            // The ledger counts it once.
            assert_eq!(ledger.bytes(), figures[0] + figures[1] - backing);
        }
    }

    #[test]
    fn a_refused_measurement_leaves_the_meter_empty_and_usable() {
        let original = Arc::new(".r { color: blue }".encode_utf16().collect::<Vec<_>>());
        let snapshot = snapshot(&original);
        let mut meter = CssSourceMeter::new(CssSourceBounds::new(usize::MAX, 2, 16));
        let refused = meter.measure(0, |sources| {
            sources.visit(&Records(&[&snapshot, &snapshot]))
        });
        assert_eq!(
            refused.unwrap_err().exhausted_limit(),
            Some(CssSourceRetentionResource::Visits)
        );
        assert_eq!(
            (meter.steps(), meter.depth(), meter.recorded().len()),
            (0, 0, 0)
        );
        meter
            .measure(0, |sources| sources.visit(&Records(&[&snapshot])))
            .unwrap();
        assert_eq!((meter.steps(), meter.recorded().len()), (2, 1));
    }

    #[test]
    fn the_storage_a_meter_keeps_is_bounded_by_the_largest_small_payload() {
        let originals = (0..300_u16)
            .map(|index| Arc::new(vec![u16::from(b'a') + index % 20, index]))
            .collect::<Vec<_>>();
        let snapshots = originals.iter().map(snapshot).collect::<Vec<_>>();
        let refs = snapshots.iter().collect::<Vec<_>>();
        let mut meter = CssSourceMeter::new(BOUNDS);
        meter
            .measure(0, |sources| sources.visit(&Records(&refs[..3])))
            .unwrap();
        let small = meter.collector.bank.capacity();
        assert!((3..16).contains(&small));
        meter
            .measure(0, |sources| sources.visit(&Records(&refs)))
            .unwrap();
        assert_eq!(meter.recorded().len(), 300);
        // A payload past the kept size releases its storage before the next one.
        meter
            .measure(0, |sources| sources.visit(&Records(&refs[..1])))
            .unwrap();
        assert!(meter.collector.bank.capacity() < 16);
        assert_eq!(meter.recorded().len(), 1);
    }
}
