//! Test-only actual-execution counters for #1263.
//!
//! Root declares this module only under `cfg(test)`. Presence of the compile-time
//! SURGEIST_DOM_STORAGE_COUNTERS environment variable enables the collector;
//! remove the variable and use a separate target for the uninstrumented build.
//! Producer wrappers use `if scale_metrics::ENABLED { record(...) }`, allowing
//! the false constant to remove both the event and thread-local access.
//!
//! Events belong at executed producer operations, never at test expectations,
//! estimated node counts, touched-set iteration summaries, or command names.
//! Domain entry must be recorded by its actual owning stage. A required domain
//! without a real event hook refuses measurement rather than reporting zero.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::rc::Rc;

pub(crate) const ENABLED: bool = option_env!("SURGEIST_DOM_STORAGE_COUNTERS").is_some();

#[derive(Clone, Copy, Debug)]
pub(super) enum Domain {
    Bank,
    Transaction,
    Validation,
    Accounting,
    Source,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Hooks(u8);
impl Hooks {
    pub const fn one(domain: Domain) -> Self {
        Self(1 << domain as u8)
    }
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    fn missing(self, observed: Self) -> Self {
        Self(self.0 & !observed.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Event {
    Entered(Domain),
    BankLookup,
    BankPageVisited,
    VacancyPageVisited,
    /// Actual changed administrative allocation charge from the producer:
    /// Page/registry/release/custody/control targets and Vec backing capacities.
    /// Count Arc/Box targets after construction and Vec capacity after reserve;
    /// do not omit control allocations or charge already-shared backing anew.
    StructuralAllocation {
        bytes: usize,
    },
    CanonicalRecordRead,
    /// Actual materialized transaction record backing; shared fields excluded.
    RecordMaterialized {
        bytes: usize,
    },
    /// Bytes of actual initialized, moved or copied sequence elements, including
    /// a completed prefix before refusal. Unused capacity belongs to structural
    /// allocation; retained aliases emit no copy. Fixed in-place field or cell
    /// assignments, stack moves, standard sort internals and opaque lower-level
    /// allocations are outside this selected sequence-storage measurement.
    SequenceMaterialized {
        bytes: usize,
    },
    /// Actual elements copied, initialized or moved by a selected transaction
    /// Sequence or its canonical key/event publication. This subset excludes
    /// journal, arena, effect and other administrative sequence movement.
    RelationSequenceMaterialized {
        bytes: usize,
    },
    QueueRecordVisited,
    /// Exact suffix movement after a completed Vec insertion/removal.
    QueueRecordsMoved {
        records: usize,
    },
    /// One reached item of a source-required hosted history sequence/search.
    /// Exact-ID discovery and outer bank administration use bank counters.
    HostedSemanticItemVisited,
    /// Actual descriptor examination while preparing oldest-entry evictions.
    HistoryEvictionVisited,
    HistoryEntryEvicted,
    /// Actual entry inspection and completed output copies for changes_since.
    HistoryQueryEntryVisited,
    HistoryQueryChangesCopied {
        records: usize,
    },
    SlotDiscoveryStep,
    SlotOrdinalSuccessorRead,
    SlotAssignmentInvoked,
    ValidationRecordVisited,
    AccountingRecordVisited,
    JournalEntry,
    Savepoint,
    /// Actual simultaneous savepoint administration and newly retained backing
    /// after a retain/release. Shared persistent paths are counted once in the
    /// Source retained union, rather than recursively added for each savepoint.
    SavepointBytes {
        identity: usize,
        live: usize,
    },
    /// Actual custody-owned Source totals after a retain/release, not RSS.
    SourceRetained {
        bytes: usize,
        units: usize,
    },
    /// One actual cumulative work increment, including preflight/rollback work.
    SourceStep,
    SourcePreflight,
    PublishedRoot,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Counters {
    pub hooks: Hooks,
    pub bank_lookups: usize,
    pub bank_page_visits: usize,
    pub vacancy_page_visits: usize,
    pub structural_allocations: usize,
    pub structural_bytes: usize,
    pub canonical_record_reads: usize,
    pub record_materializations: usize,
    pub record_materialized_bytes: usize,
    pub sequence_materializations: usize,
    pub sequence_materialized_bytes: usize,
    pub relation_sequence_materializations: usize,
    pub relation_sequence_materialized_bytes: usize,
    pub queue_record_visits: usize,
    pub queue_records_moved: usize,
    pub hosted_semantic_visits: usize,
    pub history_eviction_visits: usize,
    pub history_entries_evicted: usize,
    pub history_query_entry_visits: usize,
    pub history_query_changes_copied: usize,
    pub slot_discovery_steps: usize,
    pub slot_ordinal_successor_reads: usize,
    pub slot_assignments: usize,
    pub validation_record_visits: usize,
    pub accounting_record_visits: usize,
    pub journal_entries: usize,
    pub savepoints: usize,
    pub savepoint_live_bytes: usize,
    pub savepoint_peak_bytes: usize,
    pub source_live_bytes: usize,
    pub source_peak_bytes: usize,
    pub source_live_units: usize,
    pub source_peak_units: usize,
    pub source_steps: usize,
    pub source_preflights: usize,
    pub published_roots: usize,
}

fn add(target: &mut usize, value: usize) {
    *target = target
        .checked_add(value)
        .expect("test measurement representation");
}

impl Counters {
    fn apply(&mut self, event: Event, journals: &mut BTreeMap<usize, usize>) {
        match event {
            Event::Entered(domain) => self.hooks = self.hooks.union(Hooks::one(domain)),
            Event::BankLookup => add(&mut self.bank_lookups, 1),
            Event::BankPageVisited => add(&mut self.bank_page_visits, 1),
            Event::VacancyPageVisited => add(&mut self.vacancy_page_visits, 1),
            Event::StructuralAllocation { bytes } => {
                add(&mut self.structural_allocations, 1);
                add(&mut self.structural_bytes, bytes);
            }
            Event::CanonicalRecordRead => add(&mut self.canonical_record_reads, 1),
            Event::RecordMaterialized { bytes } => {
                add(&mut self.record_materializations, 1);
                add(&mut self.record_materialized_bytes, bytes);
            }
            Event::SequenceMaterialized { bytes } => {
                add(&mut self.sequence_materializations, 1);
                add(&mut self.sequence_materialized_bytes, bytes);
            }
            Event::RelationSequenceMaterialized { bytes } => {
                add(&mut self.relation_sequence_materializations, 1);
                add(&mut self.relation_sequence_materialized_bytes, bytes);
            }
            Event::QueueRecordVisited => add(&mut self.queue_record_visits, 1),
            Event::QueueRecordsMoved { records } => add(&mut self.queue_records_moved, records),
            Event::HostedSemanticItemVisited => add(&mut self.hosted_semantic_visits, 1),
            Event::HistoryEvictionVisited => add(&mut self.history_eviction_visits, 1),
            Event::HistoryEntryEvicted => add(&mut self.history_entries_evicted, 1),
            Event::HistoryQueryEntryVisited => add(&mut self.history_query_entry_visits, 1),
            Event::HistoryQueryChangesCopied { records } => {
                add(&mut self.history_query_changes_copied, records);
            }
            Event::SlotDiscoveryStep => add(&mut self.slot_discovery_steps, 1),
            Event::SlotOrdinalSuccessorRead => add(&mut self.slot_ordinal_successor_reads, 1),
            Event::SlotAssignmentInvoked => add(&mut self.slot_assignments, 1),
            Event::ValidationRecordVisited => add(&mut self.validation_record_visits, 1),
            Event::AccountingRecordVisited => add(&mut self.accounting_record_visits, 1),
            Event::JournalEntry => add(&mut self.journal_entries, 1),
            Event::Savepoint => add(&mut self.savepoints, 1),
            Event::SavepointBytes { identity, live } => {
                let before = if live == 0 {
                    journals.remove(&identity)
                } else {
                    journals.insert(identity, live)
                }
                .unwrap_or(0);
                self.savepoint_live_bytes = self
                    .savepoint_live_bytes
                    .checked_sub(before)
                    .and_then(|total| total.checked_add(live))
                    .expect("simultaneous observed journal storage");
                self.savepoint_peak_bytes =
                    self.savepoint_peak_bytes.max(self.savepoint_live_bytes);
            }
            Event::SourceRetained { bytes, units } => {
                self.source_live_bytes = bytes;
                self.source_live_units = units;
                self.source_peak_bytes = self.source_peak_bytes.max(self.source_live_bytes);
                self.source_peak_units = self.source_peak_units.max(self.source_live_units);
            }
            Event::SourceStep => add(&mut self.source_steps, 1),
            Event::SourcePreflight => add(&mut self.source_preflights, 1),
            Event::PublishedRoot => add(&mut self.published_roots, 1),
        }
    }
}

#[derive(Clone, Copy)]
struct SourceObservation {
    identity: usize,
    operation: Option<crate::DomSourceOperationId>,
    ancestors: super::custody::Usage,
    controlled: super::custody::Usage,
    external: super::custody::Usage,
}
#[derive(Default)]
struct Collector {
    counters: Counters,
    journals: BTreeMap<usize, usize>,
}
thread_local! {
    static ACTIVE: RefCell<Option<Collector>> = const { RefCell::new(None) };
    static SOURCE: RefCell<Option<SourceObservation>> = const { RefCell::new(None) };
}

fn source_total(observation: SourceObservation) {
    let total = observation
        .controlled
        .checked_add(observation.external)
        .expect("actual admitted Source measurement total");
    record(Event::SourceRetained {
        bytes: total.bytes(),
        units: total.units(),
    });
}

/// Refresh a local Source envelope in its actual global custody domain. The
/// cached ancestor prefix is included once; no target or debit is retained.
pub(super) fn select_source_ledger(
    identity: usize,
    controlled: super::custody::Usage,
    external: super::custody::Usage,
) {
    if ENABLED {
        let observation = SOURCE.with(|source| {
            let mut source = source.borrow_mut();
            let operation = source
                .as_ref()
                .filter(|old| old.identity == identity)
                .and_then(|old| old.operation);
            let ancestors = source
                .as_ref()
                .filter(|old| old.identity == identity)
                .map_or(super::custody::Usage::default(), |old| old.ancestors);
            let observation = SourceObservation {
                identity,
                operation,
                ancestors,
                controlled,
                external: external
                    .checked_add(ancestors)
                    .expect("retained external ancestor sum"),
            };
            *source = Some(observation);
            observation
        });
        source_total(observation);
    }
}

/// Select the actual retained top after a Source lifecycle transition. Only
/// copied identities and numerical receipts are retained by the collector.
pub(super) fn source_owner_changed(
    identity: usize,
    controlled: super::custody::Usage,
    external: super::custody::Usage,
    ancestors: super::custody::Usage,
    operation: Option<crate::DomSourceOperationId>,
) {
    if ENABLED {
        let observation = SourceObservation {
            identity,
            operation,
            ancestors,
            controlled,
            external: external
                .checked_add(ancestors)
                .expect("retained external ancestor sum"),
        };
        SOURCE.with(|source| *source.borrow_mut() = Some(observation));
        source_total(observation);
    }
}

/// Last field of the actual Source owner, so its external envelope ends only
/// after its semantic fields and controlled dependencies have been destroyed.
pub(super) struct SourceOwnerLifetime(pub crate::DomSourceOperationId);
impl Drop for SourceOwnerLifetime {
    fn drop(&mut self) {
        source_owner_dropped(self.0);
    }
}

fn source_owner_dropped(operation: crate::DomSourceOperationId) {
    if ENABLED {
        let observation = SOURCE.with(|source| {
            let mut source = source.borrow_mut();
            let selected = source
                .as_mut()
                .filter(|value| value.operation == Some(operation))?;
            selected.operation = None;
            selected.external = super::custody::Usage::default();
            selected.ancestors = super::custody::Usage::default();
            Some(*selected)
        });
        if let Some(observation) = observation {
            source_total(observation);
        }
    }
}

#[cfg(test)]
pub(super) fn source_external_observation() -> Option<super::custody::Usage> {
    SOURCE.with(|source| source.borrow().as_ref().map(|value| value.external))
}

/// Observe a committed external-envelope change belonging to this same Source.
/// A refusal's prospective envelope is never reported as retained storage.
pub(super) fn source_external_changed(identity: usize, external: super::custody::Usage) {
    if ENABLED {
        let observation = SOURCE.with(|source| {
            let mut source = source.borrow_mut();
            let selected = source.as_mut().filter(|value| value.identity == identity)?;
            selected.external = external
                .checked_add(selected.ancestors)
                .expect("retained external ancestor sum");
            Some(*selected)
        });
        if let Some(observation) = observation {
            source_total(observation);
        }
    }
}

/// Called only after an actual retain/release commits in the selected ledger.
/// Changes in another private test domain cannot overwrite Source observations.
pub(super) fn source_ledger_changed(identity: usize, controlled: super::custody::Usage) {
    if ENABLED {
        let observation = SOURCE.with(|source| {
            let mut source = source.borrow_mut();
            let selected = source.as_mut().filter(|value| value.identity == identity)?;
            selected.controlled = controlled;
            Some(*selected)
        });
        if let Some(observation) = observation {
            source_total(observation);
        }
    }
}

/// Inline disabled wrappers compile away in the uninstrumented test build.
#[inline]
pub(super) fn record(event: Event) {
    if ENABLED {
        ACTIVE.with(|active| {
            if let Some(collector) = active.borrow_mut().as_mut() {
                collector.counters.apply(event, &mut collector.journals);
            }
        });
    }
}

#[derive(Debug)]
pub(super) enum Refusal {
    Disabled,
    Nested,
    MissingHooks {
        required: Hooks,
        missing: Hooks,
        observed: Hooks,
    },
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => f.write_str("execution counters were disabled at compile time"),
            Self::Nested => f.write_str("the owner thread already has an active measurement"),
            Self::MissingHooks {
                required,
                missing,
                observed,
            } => write!(
                f,
                "required producer hooks {required:?}; missing {missing:?}; observed {observed:?}"
            ),
        }
    }
}
impl std::error::Error for Refusal {}

/// Scoped to the executing owner thread; it cannot silently move to another
/// thread and miss that thread's execution. Drop clears custody on panic too.
pub(super) struct Measurement {
    active: bool,
    _thread: PhantomData<Rc<()>>,
}
impl Measurement {
    pub fn begin() -> Result<Self, Refusal> {
        if !ENABLED {
            return Err(Refusal::Disabled);
        }
        ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            if active.is_some() {
                return Err(Refusal::Nested);
            }
            *active = Some(Collector::default());
            SOURCE.with(|source| *source.borrow_mut() = None);
            Ok(Self {
                active: true,
                _thread: PhantomData,
            })
        })
    }

    pub fn finish(mut self, required: Hooks) -> Result<Counters, Refusal> {
        let counters = ACTIVE
            .with(|active| active.borrow_mut().take())
            .expect("live measurement")
            .counters;
        self.active = false;
        SOURCE.with(|source| *source.borrow_mut() = None);
        let missing = required.missing(counters.hooks);
        if missing != Hooks::default() {
            return Err(Refusal::MissingHooks {
                required,
                missing,
                observed: counters.hooks,
            });
        }
        Ok(counters)
    }
}
impl Drop for Measurement {
    fn drop(&mut self) {
        if self.active {
            SOURCE.with(|source| *source.borrow_mut() = None);
            ACTIVE.with(|active| {
                active.borrow_mut().take();
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::custody::Usage;

    #[test]
    fn source_observation_combines_only_selected_committed_union_with_external_envelope() {
        if !ENABLED {
            return;
        }
        let measurement = Measurement::begin().unwrap();
        select_source_ledger(73, Usage::new(100, 10), Usage::new(20, 3));
        source_ledger_changed(73, Usage::new(150, 15));
        source_ledger_changed(99, Usage::new(5_000, 5_000));
        source_external_changed(99, Usage::new(9_000, 9_000));
        source_external_changed(73, Usage::new(5, 1));
        source_ledger_changed(73, Usage::new(70, 7));
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(counters.source_live_bytes, 75);
        assert_eq!(counters.source_live_units, 8);
        assert_eq!(counters.source_peak_bytes, 170);
        assert_eq!(counters.source_peak_units, 18);
    }

    #[test]
    fn ancestor_prefix_is_counted_once_and_parent_restoration_replaces_child_envelope() {
        if !ENABLED {
            return;
        }
        let measurement = Measurement::begin().unwrap();
        source_owner_changed(
            73,
            Usage::new(100, 10),
            Usage::new(5, 1),
            Usage::new(20, 3),
            None,
        );
        select_source_ledger(73, Usage::new(100, 10), Usage::new(7, 2));
        assert_eq!(source_external_observation(), Some(Usage::new(27, 5)));
        source_external_changed(73, Usage::new(8, 4));
        assert_eq!(source_external_observation(), Some(Usage::new(28, 7)));
        source_owner_changed(
            73,
            Usage::new(90, 8),
            Usage::new(20, 3),
            Usage::default(),
            None,
        );
        source_ledger_changed(73, Usage::new(80, 7));
        assert_eq!(source_external_observation(), Some(Usage::new(20, 3)));
        source_owner_changed(
            73,
            Usage::new(80, 7),
            Usage::default(),
            Usage::default(),
            None,
        );
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(counters.source_live_bytes, 80);
        assert_eq!(counters.source_live_units, 7);
        assert_eq!(counters.source_peak_bytes, 128);
        assert_eq!(counters.source_peak_units, 17);
    }

    #[test]
    fn simultaneous_journals_accumulate_and_release_independently() {
        if !ENABLED {
            return;
        }
        let measurement = Measurement::begin().unwrap();
        record(Event::SavepointBytes {
            identity: 1,
            live: 100,
        });
        record(Event::SavepointBytes {
            identity: 2,
            live: 70,
        });
        record(Event::SavepointBytes {
            identity: 1,
            live: 130,
        });
        record(Event::SavepointBytes {
            identity: 2,
            live: 0,
        });
        record(Event::SavepointBytes {
            identity: 1,
            live: 0,
        });
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(counters.savepoint_peak_bytes, 200);
        assert_eq!(counters.savepoint_live_bytes, 0);
    }

    #[test]
    fn measurement_drop_clears_selected_source_without_retaining_an_owner() {
        if !ENABLED {
            return;
        }
        let measurement = Measurement::begin().unwrap();
        select_source_ledger(73, Usage::new(100, 10), Usage::new(20, 3));
        drop(measurement);
        let measurement = Measurement::begin().unwrap();
        source_ledger_changed(73, Usage::new(900, 90));
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(counters.source_live_bytes, 0);
        assert_eq!(counters.source_peak_bytes, 0);
    }
}
