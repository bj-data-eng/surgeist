//! Small name-local payload/index and retained-key characterization.
//! Known targets/buffers and raw index bytes are separate from conservative
//! receipts. This is neither an allocator census nor a memory-savings claim.
use super::*;
use crate::{DomAttribute, DomQualifiedName};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug)]
enum Population {
    UniqueText,
    RepeatedComponents,
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

#[test]
fn small_name_populations_report_index_keys_and_dead_weak_lifetimes() {
    eprintln!(
        "name_memory fields atom={} qualified_name={} attribute={} record={} utf16_vector={} \
         index_row={} index_header={} owner_state={}; \
         exclusions=opaque_Arc_control_blocks,allocator_rounding_slack_metadata,\
         generic_custody_registry_release_machinery,instrumentation_storage; \
         name_index_bytes_are_raw_but_its_allocation_count_is_not_enumerated; \
         no_physical_total_or_savings_or_baseline_claim=true",
        size_of::<DomNameAtom>(),
        size_of::<DomQualifiedName>(),
        size_of::<DomAttribute>(),
        size_of::<NameRecord>(),
        size_of::<Vec<u16>>(),
        size_of::<index::Row>(),
        size_of::<Index>(),
        size_of::<Mutex<State>>()
    );
    for population in [Population::UniqueText, Population::RepeatedComponents] {
        for count in [0, 32, 128] {
            // All expected text and distinct-component counts come from these
            // declared inputs, independently of the actual name-index rows.
            let inputs = requests(population, count);
            let expected: BTreeSet<_> = inputs.iter().cloned().collect();
            assert_eq!(
                expected.len(),
                match population {
                    Population::UniqueText => count,
                    Population::RepeatedComponents => usize::from(count != 0) * 5,
                }
            );
            eprintln!(
                "name_memory inputs population={population:?} population_size={count} \
                 component_requests={} independently_expected_distinct_texts={}",
                inputs.len(),
                expected.len()
            );
            let (ledger, control, registry) = fixture();
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
            assert_eq!(keys.len(), expected.len());
            for key in &keys {
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
            report_registry(
                population,
                count,
                "all_selector_keys_held",
                &registry,
                &ledger,
                &expected,
                0,
            );

            // Keep the independently selected first text as a selector-like
            // strong key. Retire every other semantic holder in this original
            // owner; do not transfer a measured budget to a fresh owner.
            let kept_text = expected.iter().next();
            let mut selected = HashSet::new();
            let mut retired = Vec::new();
            let mut record_observers = Vec::new();
            let mut string_observers = Vec::new();
            for key in keys {
                if Some(key.text().code_units()) == kept_text.map(Vec::as_slice) {
                    selected.insert(key);
                } else {
                    record_observers.push(Arc::downgrade(key.record()));
                    string_observers.push(Arc::downgrade(key.text().shared_code_units()));
                    retired.push(key);
                }
            }
            let dead = expected.len() - selected.len();
            let charged_before_release = registry.lock().records;
            drop(retired);
            assert!(record_observers.iter().all(|weak| weak.upgrade().is_none()));
            assert!(string_observers.iter().all(|weak| weak.upgrade().is_none()));
            // The registry's Weak still owns dead NameRecord target space.
            // Diagnostic Weaks must not prolong either target after cleanup;
            // dropping the Vec observer also releases its fixed target space.
            drop(record_observers);
            drop(string_observers);
            assert_eq!(registry.lock().records, charged_before_release);
            let selected_text: BTreeSet<_> = kept_text.cloned().into_iter().collect();
            report_registry(
                population,
                count,
                "dead_rows_before_cleanup",
                &registry,
                &ledger,
                &selected_text,
                dead,
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
            report_registry(
                population,
                count,
                "dead_rows_cleaned_live_selector_unchanged",
                &registry,
                &ledger,
                &selected_text,
                0,
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
                assert!(selected_text.contains(key.text().code_units()));
                assert_eq!(key.owner(), original_owner);
                assert_eq!(Some(key.identity()), selected_identity);
            }
            report_keys(population, count, &selected);
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
            eprintln!(
                "name_memory population={population:?} population_size={count} \
                 stage=last_selector_key_released known_name_targets=0 known_utf16_vectors=0 \
                 known_utf16_buffers=0 allocator_reclamation_not_observed=true"
            );
        }
    }
}
