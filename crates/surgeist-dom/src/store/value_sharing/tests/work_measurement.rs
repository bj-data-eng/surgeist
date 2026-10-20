//! Counter-qualified owner work. HTML coordinates below are admitted budgets;
//! actual HTML hash/comparison phases are qualified inside the HTML owner.
use super::*;
use crate::store::scale_metrics::{self, Hooks, Measurement};

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1"]
fn bank_collision_comparator_counts_actual_first_last_and_refused_prefix() {
    scale_metrics::require_enabled().expect("counter-qualified measurement");
    let f = fixture();
    let original = value(&f, &[97, 0xd800, 122], 1);
    for (units, pairs) in [(vec![98, 0xd800, 122], 11), (vec![97, 0xd800, 121], 13)] {
        let other = value(&f, &units, 1);
        let measurement = Measurement::begin().unwrap();
        assert!(
            !admitted(|a| banks::compare_for_test(original.entries(), other.entries(), a)).unwrap()
        );
        let counts = measurement.finish(Hooks::default()).unwrap();
        // Ten exact local-name pairs precede the first/last unequal value pair.
        assert_eq!(counts.sharing_bank_comparison_pairs, pairs);
        assert_eq!(counts.sharing_bank_hash_units, 0);
        assert_eq!(counts.sharing_bank_probe_slots, 0);
        assert_eq!(
            original.entries()[0].value().code_units(),
            &[97, 0xd800, 122]
        );
    }
    let equal = value(&f, &[97, 0xd800, 122], 7);
    let mut completed = 0;
    let measurement = Measurement::begin().unwrap();
    let result = banks::compare_for_test(
        original.entries(),
        equal.entries(),
        &mut Admission {
            global: &mut |_| Ok(()),
            step: &mut || {
                // Bank length, entry and namespace/prefix/local field checks are
                // five visits before the first raw local-name comparison pair.
                if completed == 7 {
                    return Err(DomError::Cancelled);
                }
                completed += 1;
                Ok(())
            },
        },
    );
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(result, Err(DomError::Cancelled));
    assert_eq!(counts.sharing_bank_comparison_pairs, 2);
    assert_eq!(counts.sharing_bank_hits, 0);
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1"]
fn bank_probe_offer_and_equal_raw_units_are_attributed_without_rehash() {
    scale_metrics::require_enabled().expect("counter-qualified measurement");
    let f = fixture();
    let session = active_session(&f);
    let bank = value(&f, &[0xd800, 0, 122], 7);
    let measurement = Measurement::begin().unwrap();
    let BankProbe::Miss(miss) = admitted(|a| session.probe_bank(bank.entries(), a)).unwrap() else {
        panic!("new bank misses")
    };
    let counts = measurement.finish(Hooks::default()).unwrap();
    // The fixture spells data-value (10 units), plus three raw value units.
    assert_eq!(counts.sharing_bank_hash_units, 13);
    assert_eq!(counts.sharing_bank_comparison_pairs, 0);
    assert_eq!(counts.sharing_bank_probe_slots, 4);
    assert_eq!(counts.sharing_bank_misses, 1);
    let measurement = Measurement::begin().unwrap();
    admitted(|a| session.offer_bank(&bank, Some(miss), a)).unwrap();
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(
        counts.sharing_bank_hash_units, 0,
        "qualified miss reuses its fingerprint"
    );
    assert_eq!(counts.sharing_bank_offers, 1);
    // The first offer initializes the slot directory, then one Before cell.
    assert_eq!(counts.sharing_metadata_materializations, 2);
    assert!(counts.sharing_metadata_materialized_bytes > 0);
    assert_eq!(
        counts.sequence_materializations,
        counts.sharing_metadata_materializations
    );
    assert_eq!(
        counts.sequence_materialized_bytes,
        counts.sharing_metadata_materialized_bytes
    );
    let equal = value(&f, &[0xd800, 0, 122], 1);
    assert!(!bank.same_storage(&equal));
    let measurement = Measurement::begin().unwrap();
    let selected = hit(&session, &equal);
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert!(selected.storage().same_storage(&bank));
    assert_eq!(
        selected.storage().entries()[0].value().code_units(),
        &[0xd800, 0, 122]
    );
    assert_eq!(counts.sharing_bank_hash_units, 13);
    assert_eq!(counts.sharing_bank_comparison_pairs, 13);
    assert_eq!(counts.sharing_bank_probe_slots, 1);
    assert_eq!(counts.sharing_bank_hits, 1);
    assert_eq!(counts.sharing_metadata_materializations, 0);
    drop(selected);
    drop(session);
    assert_raw(&f);
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1"]
fn empty_reserved_oversized_and_cancelled_bank_paths_count_only_completed_work() {
    scale_metrics::require_enabled().expect("counter-qualified measurement");
    let f = fixture();
    let session = active_session(&f);
    let baseline = f.ledger.total();
    let measurement = Measurement::begin().unwrap();
    admitted(|a| session.offer_bank(&AttributeStorage::default(), None, a)).unwrap();
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(counts.sharing_bank_offers, 0);
    assert_eq!(counts.sharing_bank_hash_units, 0);
    assert_eq!(f.ledger.total(), baseline);
    let reserved = AttributeStorage::from_entries(Vec::with_capacity(7));
    let measurement = Measurement::begin().unwrap();
    offer(&session, &reserved);
    let selected = hit(
        &session,
        &AttributeStorage::from_entries(Vec::with_capacity(1)),
    );
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert!(selected.storage().same_storage(&reserved));
    assert_eq!(selected.storage().capacity(), 7);
    assert_eq!(counts.sharing_bank_hash_units, 0);
    assert_eq!(counts.sharing_bank_comparison_pairs, 0);
    assert_eq!(counts.sharing_bank_offers, 1);
    assert_eq!(counts.sharing_bank_hits, 1);
    drop(selected);
    for extent in [8193, 65_536] {
        let large = value(&f, &vec![0xd800; extent], 1);
        let measurement = Measurement::begin().unwrap();
        assert!(matches!(
            admitted(|a| session.probe_bank(large.entries(), a)).unwrap(),
            BankProbe::Bypassed
        ));
        let counts = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(
            counts.sharing_bank_hash_units, 10,
            "only the preceding local name is hashed"
        );
        assert_eq!(counts.sharing_bank_comparison_pairs, 0);
        assert_eq!(counts.sharing_bank_probe_slots, 0);
        assert_eq!(counts.sharing_bank_bypasses, 1);
        assert_eq!(large.entries()[0].value().code_units()[0], 0xd800);
    }
    let small = value(&f, &[0xd800, 0, 122], 1);
    let mut completed = 0;
    let measurement = Measurement::begin().unwrap();
    let result = session.probe_bank(
        small.entries(),
        &mut Admission {
            global: &mut |_| Ok(()),
            step: &mut || {
                // Entry, absent namespace/prefix, and local-name header precede
                // the ten local-name units. Refuse the third unit's admission.
                if completed == 6 {
                    return Err(DomError::Cancelled);
                }
                completed += 1;
                Ok(())
            },
        },
    );
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(result.err(), Some(DomError::Cancelled));
    assert_eq!(counts.sharing_bank_hash_units, 2);
    assert_eq!(counts.sharing_bank_probe_slots, 0);
    assert_eq!(
        counts.sharing_bank_bypasses, 0,
        "an error is not an optional bypass"
    );
    drop(session);
    assert_raw(&f);
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1"]
fn paired_lifecycle_counts_reached_authority_and_distinguishes_admitted_html_work() {
    scale_metrics::require_enabled().expect("counter-qualified measurement");
    let f = fixture();
    let baseline = f.ledger.total();
    let bank = value(&f, &[0xd800], 1);
    let measurement = Measurement::begin().unwrap();
    let session = admitted(|a| f.pool.attach(a)).unwrap();
    let BankProbe::Miss(_) = admitted(|a| session.probe_bank(bank.entries(), a)).unwrap() else {
        panic!("first eligible probe")
    };
    let mark = admitted(|a| session.checkpoint(a)).unwrap();
    mark.restore();
    let mark = admitted(|a| session.checkpoint(a)).unwrap();
    let ready = admitted(|a| session.prepare_close(&[&mark], a)).unwrap();
    session.close(ready, &[&mark]).commit();
    drop(mark);
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(counts.sharing_pair_activations, 1);
    assert_eq!(counts.sharing_marks_created, 2);
    assert_eq!(counts.sharing_marks_restored, 1);
    assert_eq!(counts.sharing_marks_accepted, 1);
    assert_eq!(counts.sharing_close_preparations, 1);
    assert_eq!(counts.sharing_pairs_closed, 1);
    assert_eq!(counts.sharing_pairs_committed, 1);
    assert_eq!(counts.sharing_pairs_aborted, 0);
    // Two verifies each examine one nomination and two live frame headers;
    // deepest-first close repeats that same three-visit selection once.
    assert_eq!(counts.sharing_coordination_visits, 9);
    assert!(counts.sharing_html_admitted_immediate[0] > 0);
    assert!(counts.sharing_html_admitted_closing[0] > 0);
    assert_eq!(counts.sharing_html_admitted_immediate[1], 0);
    assert_eq!(counts.sharing_html_admitted_closing[1], 0);
    assert_eq!(f.ledger.total(), baseline);
    let measurement = Measurement::begin().unwrap();
    let session = active_session(&f);
    drop(session);
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(counts.sharing_pair_activations, 1);
    assert_eq!(counts.sharing_pairs_aborted, 1);
    assert_eq!(counts.sharing_pairs_committed, 0);
    assert_eq!(f.ledger.total(), baseline);
}

#[test]
#[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS=1"]
fn string_driver_reports_attempts_outcomes_and_admitted_coordinates_not_phase_counts() {
    scale_metrics::require_enabled().expect("counter-qualified measurement");
    let f = fixture();
    let session = active_session(&f);
    let original = DocumentString::from_utf16(vec![0xd800, 0, 122]);
    let measurement = Measurement::begin().unwrap();
    assert!(
        admitted(|a| session.share_string(&original, a))
            .unwrap()
            .is_none()
    );
    let selected =
        admitted(|a| session.share_string(&DocumentString::from_utf16(vec![0xd800, 0, 122]), a))
            .unwrap()
            .unwrap();
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert!(Arc::ptr_eq(
        selected.value().shared_code_units(),
        original.shared_code_units()
    ));
    assert_eq!(counts.sharing_string_probes, 2);
    assert_eq!(counts.sharing_string_offer_promotions, 1);
    assert_eq!(counts.sharing_string_offers, 1);
    assert_eq!(counts.sharing_string_hits, 1);
    assert_eq!(
        counts.sharing_string_misses, 0,
        "the cold miss was promoted into an offer"
    );
    assert_eq!(counts.sharing_string_bypasses, 0);
    assert!(counts.sharing_html_admitted_immediate[1] > 0);
    drop(selected);
    drop(session);
    let dormant = admitted(|a| f.pool.attach(a)).unwrap();
    assert!(dormant.is_dormant());
    let large = DocumentString::from_utf16(vec![0xd800; 65_536]);
    let measurement = Measurement::begin().unwrap();
    assert!(
        admitted(|a| dormant.share_string(&large, a))
            .unwrap()
            .is_none()
    );
    let counts = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(counts.sharing_string_bypasses, 1);
    assert_eq!(counts.sharing_string_probes, 0);
    assert_eq!(counts.sharing_html_admitted_immediate, [0; 5]);
    assert_eq!(large.code_units()[0], 0xd800);
}
