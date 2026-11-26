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

/// A snapshot held by a ledger.
#[derive(Debug)]
struct Held {
    source: CssMeasuredSource,
    references: usize,
}

/// The snapshots that share one original UTF-16 backing.
#[derive(Debug)]
struct Shared {
    users: usize,
}

/// The distinct snapshots of many measurements, counted by reference, and the
/// retained bytes the collector counts for them.
///
/// The bytes are those of each distinct snapshot, less the bytes of an
/// original UTF-16 backing for every snapshot sharing it beyond the first.
/// They are the same for every order of [`Self::add`] and [`Self::remove`]
/// that leaves the same measurements added.
#[derive(Debug, Default)]
pub struct CssSourceLedger {
    snapshots: HashMap<usize, Held>,
    backings: HashMap<usize, Shared>,
    bytes: usize,
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
        self.snapshots.len()
    }

    /// Whether the ledger holds no snapshot.
    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
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

    fn add_source(&mut self, source: &CssMeasuredSource) -> Result<(), CssSourceRetentionError> {
        let key = identity(&source.snapshot);
        if let Some(held) = self.snapshots.get_mut(&key) {
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
        self.snapshots
            .try_reserve(1)
            .map_err(|_| CssSourceRetentionError::allocation())?;
        if let Some(backing) = source.backing
            && !self.backings.contains_key(&backing.identity)
        {
            self.backings
                .try_reserve(1)
                .map_err(|_| CssSourceRetentionError::allocation())?;
            self.backings.insert(backing.identity, Shared { users: 0 });
        }
        if let Some(backing) = source.backing
            && let Some(shared) = self.backings.get_mut(&backing.identity)
        {
            shared.users = shared.users.saturating_add(1);
        }
        self.snapshots.insert(
            key,
            Held {
                source: source.clone(),
                references: 1,
            },
        );
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
        let Some(held) = self.snapshots.get_mut(&key) else {
            return;
        };
        held.references = held.references.saturating_sub(1);
        if held.references > 0 {
            return;
        }
        let Some(held) = self.snapshots.remove(&key) else {
            return;
        };
        let mut still_shared = 0;
        if let Some(backing) = held.source.backing
            && let Some(shared) = self.backings.get_mut(&backing.identity)
        {
            shared.users = shared.users.saturating_sub(1);
            if shared.users == 0 {
                self.backings.remove(&backing.identity);
            } else {
                still_shared = backing.bytes;
            }
        }
        self.bytes = self
            .bytes
            .saturating_sub(held.source.bytes.saturating_sub(still_shared));
    }

    /// The distinct snapshots held, in no particular order.
    fn snapshots(&self) -> impl Iterator<Item = &CssSourceSnapshot> {
        self.snapshots.values().map(|held| &held.source.snapshot)
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
        let held = self.snapshots.len();
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
