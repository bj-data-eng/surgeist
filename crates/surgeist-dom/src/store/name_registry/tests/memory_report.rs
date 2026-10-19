//! Small name-local payload/index and retained-key characterization.
//! Known targets/buffers and raw index bytes are separate from conservative
//! receipts. This is neither an allocator census nor a memory-savings claim.
use super::*;
use crate::{DomAttribute, DomQualifiedName};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::store) enum Population {
    UniqueText,
    RepeatedComponents,
}

impl Population {
    pub(in crate::store) fn name(self) -> &'static str {
        match self {
            Self::UniqueText => "unique-text",
            Self::RepeatedComponents => "repeated-components",
        }
    }

    pub(in crate::store) fn component_requests(self, count: usize) -> usize {
        match self {
            Self::UniqueText => count,
            Self::RepeatedComponents => count.checked_mul(2).unwrap(),
        }
    }

    pub(in crate::store) fn distinct(self, count: usize) -> usize {
        match self {
            Self::UniqueText => count,
            Self::RepeatedComponents => usize::from(count != 0) * 5,
        }
    }

    fn selected_text(self, count: usize) -> Option<&'static [u16]> {
        let text: &'static [u16] = match self {
            // The independently authored lexical first input in each pattern.
            Self::UniqueText => &[102, 105, 101, 108, 100, 45, 48, 48, 48],
            Self::RepeatedComponents => &[99, 101, 108, 108],
        };
        (count != 0).then_some(text)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::store) enum Stage {
    HarnessOnly,
    OwnerEmpty,
    AllKeysHeld,
    DeadRowsBeforeCleanup,
    DeadRowsCleaned,
    OwnerReleased,
    LastKeyReleased,
}

impl Stage {
    pub(in crate::store) const ALL: [Self; 7] = [
        Self::HarnessOnly,
        Self::OwnerEmpty,
        Self::AllKeysHeld,
        Self::DeadRowsBeforeCleanup,
        Self::DeadRowsCleaned,
        Self::OwnerReleased,
        Self::LastKeyReleased,
    ];

    pub(in crate::store) fn name(self) -> &'static str {
        match self {
            Self::HarnessOnly => "harness-only",
            Self::OwnerEmpty => "owner-empty",
            Self::AllKeysHeld => "all-keys-held",
            Self::DeadRowsBeforeCleanup => "dead-rows-before-cleanup",
            Self::DeadRowsCleaned => "dead-rows-cleaned",
            Self::OwnerReleased => "owner-released-key-held",
            Self::LastKeyReleased => "last-key-released",
        }
    }

    pub(in crate::store) fn owner_live(self) -> bool {
        matches!(
            self,
            Self::OwnerEmpty
                | Self::AllKeysHeld
                | Self::DeadRowsBeforeCleanup
                | Self::DeadRowsCleaned
        )
    }

    pub(in crate::store) fn accounting_live(self) -> bool {
        self.owner_live() || self == Self::OwnerReleased
    }
}

/// Scalar observations retain no registry, atom, backing, lock or diagnostic Weak.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::store) struct Observation {
    pub stage: Stage,
    pub live_names: usize,
    pub dead_rows: usize,
    pub keys: usize,
    pub key_capacity: usize,
}

fn observe(observer: &mut Option<&mut dyn FnMut(Observation)>, at: Observation) {
    if let Some(observer) = observer {
        observer(at);
    }
}

fn requests(population: Population, count: usize) -> Vec<Vec<u16>> {
    match population {
        Population::UniqueText => (0..count)
            .map(|ordinal| format!("field-{ordinal:03}").encode_utf16().collect())
            .collect(),
        Population::RepeatedComponents => {
            let locals = ["cell", "row", "class", "id"];
            (0..count)
                .flat_map(|ordinal| ["urn:shared", locals[ordinal % locals.len()]])
                .map(|text| text.encode_utf16().collect())
                .collect()
        }
    }
}

#[derive(Debug, Default)]
struct KnownNames {
    record_targets: usize,
    dead_record_targets: usize,
    string_vector_targets: usize,
    string_buffers: usize,
    record_bytes: usize,
    string_vector_bytes: usize,
    string_buffer_bytes: usize,
}

impl KnownNames {
    fn record(&mut self, record: Option<&NameRecord>) {
        self.record_targets += 1;
        self.record_bytes += size_of::<NameRecord>();
        let Some(record) = record else {
            self.dead_record_targets += 1;
            return;
        };
        self.string_vector_targets += 1;
        self.string_vector_bytes += size_of::<Vec<u16>>();
        let capacity = record.text().shared_code_units().capacity();
        if capacity != 0 {
            self.string_buffers += 1;
            self.string_buffer_bytes += capacity.checked_mul(size_of::<u16>()).unwrap();
        }
    }

    fn identified_bytes(&self) -> usize {
        self.record_bytes
            .checked_add(self.string_vector_bytes)
            .unwrap()
            .checked_add(self.string_buffer_bytes)
            .unwrap()
    }
}

fn report_registry(
    population: Population,
    population_size: usize,
    stage: &str,
    registry: &Registry,
    ledger: &custody::Ledger,
    expected_live: &BTreeSet<Vec<u16>>,
    expected_dead: usize,
) {
    let mut known = KnownNames::default();
    let mut live_text = BTreeSet::new();
    let (index_bytes, local_charge) = {
        let state = registry.lock();
        for row in state.index.inspect_rows() {
            let record = row.record.upgrade();
            if let Some(record) = &record {
                assert_eq!(record.identity(), row.key);
                assert_eq!(row._custody.identity(), row.key);
                assert!(live_text.insert(record.text().code_units().to_vec()));
            }
            known.record(record.as_deref());
            // A temporary upgrade is gone before the stage report and before
            // any subsequent owner/key destruction.
            drop(record);
        }
        assert_eq!(state.index.entries(), expected_live.len() + expected_dead);
        let index_bytes = state.index.inspect_storage_bytes();
        assert_eq!(
            state.storage.usage(),
            custody::Usage::new(size_of::<Mutex<State>>() + index_bytes, 0)
        );
        (index_bytes, state.usage().unwrap())
    };
    assert_eq!(&live_text, expected_live);
    assert_eq!(known.dead_record_targets, expected_dead);
    assert_eq!(known.string_vector_targets, expected_live.len());
    assert_eq!(known.string_buffers, expected_live.len());
    eprintln!(
        "name_memory population={population:?} population_size={population_size} stage={stage} \
         known={known:?} known_name_target_buffer_bytes={} owner_state_targets=1 \
         owner_state_target_bytes={} raw_name_index_bytes={index_bytes} \
         name_index_allocation_count=not_enumerated conservative_name_charge={local_charge:?} \
         conservative_whole_ledger={:?}",
        known.identified_bytes(),
        size_of::<Mutex<State>>(),
        ledger.total()
    );
}

fn report_keys(population: Population, population_size: usize, keys: &HashSet<DomNameAtom>) {
    let mut known = KnownNames::default();
    for key in keys {
        known.record(Some(key.record()));
    }
    assert_eq!(known.record_targets, keys.len());
    assert_eq!(known.dead_record_targets, 0);
    eprintln!(
        "name_memory population={population:?} population_size={population_size} \
         stage=owner_released_selector_keys_held known={known:?} \
         known_name_target_buffer_bytes={} owner_state_targets=0 raw_name_index_bytes=0 \
         selected_atoms_are_external_semantic_holders=true",
        known.identified_bytes()
    );
}

/// One original registry lifecycle. Inventories add only the existing local
/// diagnostic reports; literal, identity, cleanup and release assertions always
/// execute. Every report/upgrade/diagnostic Weak ends before an observation.
pub(in crate::store) fn run_lifecycle(
    population: Population,
    count: usize,
    inventories: bool,
    mut observer: Option<&mut dyn FnMut(Observation)>,
) {
    assert!([0, 32, 128].contains(&count), "bounded name population");
    let distinct = population.distinct(count);
    let kept_text = population.selected_text(count);
    observe(
        &mut observer,
        Observation {
            stage: Stage::HarnessOnly,
            live_names: 0,
            dead_rows: 0,
            keys: 0,
            key_capacity: 0,
        },
    );
    let (ledger, control, registry) = fixture();
    assert_eq!(registry.lock().index.entries(), 0);
    observe(
        &mut observer,
        Observation {
            stage: Stage::OwnerEmpty,
            live_names: 0,
            dead_rows: 0,
            keys: 0,
            key_capacity: 0,
        },
    );

    // Independently authored input/expected collections and the operation are
    // gone before the first populated checkpoint. Only the actual key set lives.
    let keys = {
        let inputs = requests(population, count);
        let expected: BTreeSet<_> = inputs.iter().cloned().collect();
        assert_eq!(inputs.len(), population.component_requests(count));
        assert_eq!(expected.len(), distinct);
        assert_eq!(expected.iter().next().map(Vec::as_slice), kept_text);
        if inventories {
            eprintln!(
                "name_memory inputs population={population:?} population_size={count} \
                 component_requests={} independently_expected_distinct_texts={distinct}",
                inputs.len()
            );
        }
        let operation = admitted(|admission| registry.begin(None, admission)).unwrap();
        let mut keys = HashSet::new();
        for units in &inputs {
            let atom = admitted(|admission| {
                operation.reserve(DocumentString::from_utf16(units.clone()), admission)
            })
            .unwrap()
            .publish();
            assert_eq!(atom.text().code_units(), units);
            keys.insert(atom);
        }
        assert_eq!(keys.len(), distinct);
        for key in &keys {
            assert!(expected.contains(key.text().code_units()));
            let again = admitted(|admission| {
                operation.reserve(
                    DocumentString::from_utf16(key.text().code_units().to_vec()),
                    admission,
                )
            })
            .unwrap()
            .publish();
            assert_eq!(again.same_identity(key), Ok(true));
        }
        drop(operation);
        assert_raw_local(&registry);
        assert_eq!(registry.lock().index.entries(), distinct);
        if inventories {
            report_registry(
                population,
                count,
                "all_selector_keys_held",
                &registry,
                &ledger,
                &expected,
                0,
            );
        }
        keys
    };
    observe(
        &mut observer,
        Observation {
            stage: Stage::AllKeysHeld,
            live_names: distinct,
            dead_rows: 0,
            keys: keys.len(),
            key_capacity: keys.capacity(),
        },
    );

    let mut selected = HashSet::new();
    let mut retired = Vec::new();
    let mut record_observers = Vec::new();
    let mut string_observers = Vec::new();
    for key in keys {
        if Some(key.text().code_units()) == kept_text {
            selected.insert(key);
        } else {
            record_observers.push(Arc::downgrade(key.record()));
            string_observers.push(Arc::downgrade(key.text().shared_code_units()));
            retired.push(key);
        }
    }
    assert_eq!(selected.len(), usize::from(count != 0));
    let dead = distinct - selected.len();
    let charged_before_release = registry.lock().records;
    drop(retired);
    assert!(record_observers.iter().all(|weak| weak.upgrade().is_none()));
    assert!(string_observers.iter().all(|weak| weak.upgrade().is_none()));
    // Only the registry's own Weak rows retain the dead record target space.
    drop(record_observers);
    drop(string_observers);
    assert_eq!(registry.lock().records, charged_before_release);
    assert_eq!(registry.lock().index.entries(), selected.len() + dead);
    if inventories {
        let selected_text: BTreeSet<_> = kept_text.map(<[u16]>::to_vec).into_iter().collect();
        report_registry(
            population,
            count,
            "dead_rows_before_cleanup",
            &registry,
            &ledger,
            &selected_text,
            dead,
        );
    }
    observe(
        &mut observer,
        Observation {
            stage: Stage::DeadRowsBeforeCleanup,
            live_names: selected.len(),
            dead_rows: dead,
            keys: selected.len(),
            key_capacity: selected.capacity(),
        },
    );

    let operation = admitted(|admission| registry.begin(None, admission)).unwrap();
    let extent = registry.lock().index.inspect_row_extent();
    let mut removed = 0;
    for _ in 0..extent {
        let step = admitted(|admission| operation.cleanup(1, admission)).unwrap();
        assert!(step <= 1, "one incremental cleanup visit");
        removed += step;
    }
    assert_eq!(removed, dead);
    drop(operation);
    control.reconcile_releases();
    assert_raw_local(&registry);
    assert_eq!(registry.lock().index.entries(), selected.len());
    for key in &selected {
        assert_eq!(Some(key.text().code_units()), kept_text);
    }
    if inventories {
        let selected_text: BTreeSet<_> = kept_text.map(<[u16]>::to_vec).into_iter().collect();
        report_registry(
            population,
            count,
            "dead_rows_cleaned_live_selector_unchanged",
            &registry,
            &ledger,
            &selected_text,
            0,
        );
    }
    observe(
        &mut observer,
        Observation {
            stage: Stage::DeadRowsCleaned,
            live_names: selected.len(),
            dead_rows: 0,
            keys: selected.len(),
            key_capacity: selected.capacity(),
        },
    );

    let selected_identity = selected.iter().next().map(DomNameAtom::identity);
    let original_owner = registry.lock().owner;
    let owner_observer = Arc::downgrade(&registry.0);
    drop(registry);
    assert!(owner_observer.upgrade().is_none());
    drop(owner_observer);
    control.reconcile_releases();
    assert_eq!(
        ledger.total(),
        bank::CustodyRegistry::administration_usage()
    );
    for key in &selected {
        assert_eq!(Some(key.text().code_units()), kept_text);
        assert_eq!(key.owner(), original_owner);
        assert_eq!(Some(key.identity()), selected_identity);
    }
    if inventories {
        report_keys(population, count, &selected);
    }
    // This checkpoint deliberately retains the ledger/registry administration,
    // plus the disclosed selected HashSet, but no operational registry or Weak.
    observe(
        &mut observer,
        Observation {
            stage: Stage::OwnerReleased,
            live_names: selected.len(),
            dead_rows: 0,
            keys: selected.len(),
            key_capacity: selected.capacity(),
        },
    );
    let record_observers: Vec<_> = selected
        .iter()
        .map(|key| Arc::downgrade(key.record()))
        .collect();
    let string_observers: Vec<_> = selected
        .iter()
        .map(|key| Arc::downgrade(key.text().shared_code_units()))
        .collect();
    drop(selected);
    assert!(record_observers.iter().all(|weak| weak.upgrade().is_none()));
    assert!(string_observers.iter().all(|weak| weak.upgrade().is_none()));
    drop(record_observers);
    drop(string_observers);
    assert_eq!(
        ledger.total(),
        bank::CustodyRegistry::administration_usage()
    );
    drop(control);
    drop(ledger);
    if inventories {
        eprintln!(
            "name_memory population={population:?} population_size={count} \
             stage=last_selector_key_released known_name_targets=0 known_utf16_vectors=0 \
             known_utf16_buffers=0 allocator_reclamation_not_observed=true"
        );
    }
    observe(
        &mut observer,
        Observation {
            stage: Stage::LastKeyReleased,
            live_names: 0,
            dead_rows: 0,
            keys: 0,
            key_capacity: 0,
        },
    );
}

#[test]
fn small_name_populations_report_index_keys_and_dead_weak_lifetimes() {
    eprintln!(
        "name_memory fields atom={} qualified_name={} attribute={} record={} utf16_vector={} \
         current_build_html_component_string={} current_build_html_qualified_name={} current_build_html_attribute={} \
         index_row={} index_header={} owner_state={}; \
         exclusions=opaque_Arc_control_blocks,allocator_rounding_slack_metadata,\
         generic_custody_registry_release_machinery,instrumentation_storage; \
         name_index_bytes_are_raw_but_its_allocation_count_is_not_enumerated; \
         component_field_comparison_only=true no_old_abi_or_physical_baseline_claim=true \
         no_physical_total_or_savings_or_baseline_claim=true",
        size_of::<DomNameAtom>(),
        size_of::<DomQualifiedName>(),
        size_of::<DomAttribute>(),
        size_of::<NameRecord>(),
        size_of::<Vec<u16>>(),
        size_of::<DocumentString>(),
        size_of::<surgeist_html::QualifiedName>(),
        size_of::<surgeist_html::Attribute>(),
        size_of::<index::Row>(),
        size_of::<Index>(),
        size_of::<Mutex<State>>()
    );
    for population in [Population::UniqueText, Population::RepeatedComponents] {
        for count in [0, 32, 128] {
            run_lifecycle(population, count, true, None);
        }
    }
}

#[test]
fn scalar_observer_preserves_both_name_lifecycles_without_inventories() {
    for population in [Population::UniqueText, Population::RepeatedComponents] {
        for count in [32, 128] {
            let mut ordinal = 0;
            run_lifecycle(
                population,
                count,
                false,
                Some(&mut |at| {
                    assert_eq!(at.stage, Stage::ALL[ordinal]);
                    let distinct = match population {
                        Population::UniqueText => count,
                        Population::RepeatedComponents => 5,
                    };
                    let (live, dead, keys) = match at.stage {
                        Stage::HarnessOnly | Stage::OwnerEmpty | Stage::LastKeyReleased => {
                            (0, 0, 0)
                        }
                        Stage::AllKeysHeld => (distinct, 0, distinct),
                        Stage::DeadRowsBeforeCleanup => (1, distinct - 1, 1),
                        Stage::DeadRowsCleaned | Stage::OwnerReleased => (1, 0, 1),
                    };
                    assert_eq!((at.live_names, at.dead_rows, at.keys), (live, dead, keys));
                    assert!(at.key_capacity >= at.keys);
                    ordinal += 1;
                }),
            );
            assert_eq!(ordinal, 7);
        }
    }
}
