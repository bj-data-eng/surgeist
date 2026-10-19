//! Exact executed-unit counters, independently qualified by tiny raw inputs.
use super::*;
use crate::store::scale_metrics::{self, Hooks, Measurement};

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1"]
fn name_hash_and_equal_candidate_count_completed_raw_units() {
    scale_metrics::require_enabled().expect("counter-qualified measurement");
    let (_, _, registry) = fixture();
    let operation = admitted(|a| registry.begin(None, a)).unwrap();
    let input = DocumentString::from_utf16(vec![97, 0xd800, 0]);
    let measurement = Measurement::begin().unwrap();
    let first = admitted(|a| operation.reserve(input.clone(), a))
        .unwrap()
        .publish();
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(counts.name_hash_units, 3);
    assert_eq!(counts.name_comparison_pairs, 0);
    let equal = DocumentString::from_utf16(vec![97, 0xd800, 0]);
    assert!(!Arc::ptr_eq(
        input.shared_code_units(),
        equal.shared_code_units()
    ));
    let measurement = Measurement::begin().unwrap();
    let second = admitted(|a| operation.reserve(equal, a)).unwrap().publish();
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.text().code_units(), &[97, 0xd800, 0]);
    assert_eq!(counts.name_hash_units, 3);
    assert_eq!(counts.name_comparison_pairs, 3);
    assert_eq!(counts.name_probe_rows, 1);
    assert_raw_local(&registry);
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1"]
fn forced_name_collision_counts_only_compared_prefix_and_cancelled_hash_prefix() {
    scale_metrics::require_enabled().expect("counter-qualified measurement");
    for (incoming, pairs) in [(vec![98, 0xd800, 122], 1), (vec![97, 0xd800, 121], 3)] {
        let (_, _, registry) = fixture();
        let operation = admitted(|a| registry.begin(None, a)).unwrap();
        let first = admitted(|a| {
            operation.reserve_hashed(DocumentString::from_utf16(vec![97, 0xd800, 122]), 7, a)
        })
        .unwrap()
        .publish();
        let measurement = Measurement::begin().unwrap();
        let second = admitted(|a| {
            operation.reserve_hashed(DocumentString::from_utf16(incoming.clone()), 7, a)
        })
        .unwrap()
        .publish();
        let counts = measurement.finish(Hooks::default()).unwrap();
        assert_ne!(first, second);
        assert_eq!(second.text().code_units(), incoming);
        // Forced fingerprints exercise real collision equality, not hashing.
        assert_eq!(counts.name_hash_units, 0);
        assert_eq!(counts.name_comparison_pairs, pairs);
        assert_eq!(counts.name_probe_rows, 1);
        assert_raw_local(&registry);
    }
    let (_, _, registry) = fixture();
    let operation = admitted(|a| registry.begin(None, a)).unwrap();
    let mut completed = 0;
    let measurement = Measurement::begin().unwrap();
    let result = operation.reserve(
        DocumentString::from_utf16(vec![97, 0xd800, 122]),
        &mut Admission {
            global: &mut |_| Ok(()),
            step: &mut || {
                if completed == 2 {
                    return Err(DomError::Cancelled);
                }
                completed += 1;
                Ok(())
            },
        },
    );
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(result.err(), Some(DomError::Cancelled));
    assert_eq!(counts.name_hash_units, 2);
    assert_eq!(counts.name_comparison_pairs, 0);
    assert_eq!(counts.name_probe_rows, 0);
    assert_eq!(registry.lock().index.entries(), 0);
}
