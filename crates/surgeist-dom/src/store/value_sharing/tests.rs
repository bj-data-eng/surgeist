use super::*;
use crate::store::{bank, name_registry, payload};
use crate::{DomNameLimits, DomOwnerId, DomQualifiedName};
use std::sync::atomic::AtomicBool;

fn admitted<T>(f: impl FnOnce(&mut Admission<'_>) -> T) -> T {
    f(&mut Admission {
        global: &mut |_| Ok(()),
        step: &mut || Ok(()),
    })
}
struct Fixture {
    ledger: custody::Ledger,
    _registry: name_registry::Registry,
    _operation: name_registry::Operation,
    pool: Owner,
    name: DomQualifiedName,
}
fn fixture() -> Fixture {
    let ledger = custody::Ledger::default();
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let owner = DomOwnerId::allocate().unwrap();
    let names = name_registry::Registry::new(
        owner,
        &registry,
        DomNameLimits::new(32_000_000, 4096, 1_000_000, 32_000_000).unwrap(),
        &mut name_registry::Admission {
            global: &mut |_| Ok(()),
            step: &mut || Ok(()),
        },
    )
    .unwrap();
    let operation = names
        .begin(
            None,
            &mut name_registry::Admission {
                global: &mut |_| Ok(()),
                step: &mut || Ok(()),
            },
        )
        .unwrap();
    let local = operation
        .reserve(
            DocumentString::from("data-value"),
            &mut name_registry::Admission {
                global: &mut |_| Ok(()),
                step: &mut || Ok(()),
            },
        )
        .unwrap()
        .publish();
    let name = DomQualifiedName::from_atoms(None, None, local).unwrap();
    let pool = admitted(|admission| Owner::new(owner, ledger.clone(), admission)).unwrap();
    Fixture {
        ledger,
        _registry: names,
        _operation: operation,
        pool,
        name,
    }
}
fn value(f: &Fixture, units: &[u16], capacity: usize) -> AttributeStorage {
    let mut entries = Vec::with_capacity(capacity);
    entries.push(DomAttribute::from_parts(
        f.name.clone(),
        DocumentString::from_utf16(units.to_vec()),
    ));
    AttributeStorage::from_entries(entries)
}
fn active_session(f: &Fixture) -> Session {
    // The original leaf fixtures exercise actual mark/root authority, so make
    // that premise explicit. Cold acquisition is tested through eligible fronts
    // below; the real Source witness never calls this fixture helper.
    let session = admitted(|a| f.pool.attach(a)).unwrap();
    assert!(admitted(|a| session.ensure_active(a)).unwrap().is_some());
    session
}
fn offer(session: &Session, value: &AttributeStorage) {
    admitted(|a| session.offer_bank(value, None, a)).unwrap();
}
fn hit(session: &Session, value: &AttributeStorage) -> SelectedBank {
    match admitted(|a| session.probe_bank(value.entries(), a)).unwrap() {
        BankProbe::Hit(value) => value,
        _ => panic!("expected exact canonical bank"),
    }
}
fn seal(session: Session) {
    let ready = admitted(|a| session.prepare_close(&[], a)).unwrap();
    session.close(ready, &[]).commit();
}
fn assert_raw(f: &Fixture) {
    let state = lock(&f.pool.0);
    let (bytes, weak, live, saved) = state.banks.inspect();
    assert_eq!(state.banks.usage(), custody::Usage::new(bytes, 0));
    assert_eq!(weak, live + saved);
    assert!(live <= 128 && saved <= 128 * 17);
    assert_eq!(state.storage.usage(), state.usage().unwrap());
}

#[test]
fn exact_order_raw_utf16_and_capacity_reuse_are_independent_of_element_state() {
    let f = fixture();
    let session = active_session(&f);
    let first = value(&f, &[0xd800, 0, 0xdc00], 17);
    let equal = value(&f, &[0xd800, 0, 0xdc00], 1);
    offer(&session, &first);
    let selected = hit(&session, &equal);
    assert_eq!(
        selected.storage().entries()[0].value().code_units(),
        [0xd800, 0, 0xdc00]
    );
    assert_eq!(selected.storage().capacity(), 17);
    assert!(selected.storage().same_storage(&first));
    drop(selected);
    let different = value(&f, &[0xd800, 0, 0xdc01], 1);
    assert!(matches!(
        admitted(|a| session.probe_bank(different.entries(), a)).unwrap(),
        BankProbe::Miss(_)
    ));
    let mut ordered = first.entries().to_vec();
    ordered.push(DomAttribute::from_parts(f.name.clone(), "second".into()));
    let ordered = AttributeStorage::from_entries(ordered);
    offer(&session, &ordered);
    let reversed =
        AttributeStorage::from_entries(ordered.entries().iter().rev().cloned().collect());
    assert!(matches!(
        admitted(|a| session.probe_bank(reversed.entries(), a)).unwrap(),
        BankProbe::Miss(_)
    ));
    assert_raw(&f);
}

#[test]
fn nested_restore_and_abandoned_preparation_release_only_their_suffix() {
    let f = fixture();
    let session = active_session(&f);
    let base = value(&f, &[1], 1);
    offer(&session, &base);
    let baseline = lock(&f.pool.0).usage().unwrap();
    let outer = admitted(|a| session.checkpoint(a)).unwrap();
    let middle = value(&f, &[2], 1);
    offer(&session, &middle);
    let inner = admitted(|a| session.checkpoint(a)).unwrap();
    let newest = value(&f, &[3], 1);
    offer(&session, &newest);
    outer.restore();
    assert_eq!(lock(&f.pool.0).usage().unwrap(), baseline);
    drop(hit(&session, &base));
    assert!(matches!(
        admitted(|a| session.probe_bank(middle.entries(), a)).unwrap(),
        BankProbe::Miss(_)
    ));
    let ready = admitted(|a| session.prepare_close(&[&inner], a)).unwrap();
    drop(session.close(ready, &[&inner]));
    drop(inner); // still-held inner authority was invalidated by outer restore
    let next = active_session(&f);
    assert!(matches!(
        admitted(|a| next.probe_bank(base.entries(), a)).unwrap(),
        BankProbe::Miss(_)
    ));
    assert_raw(&f);
}

#[test]
fn paired_close_resolves_marks_and_an_independent_prepare_bypasses_occupied_root() {
    let f = fixture();
    let session = active_session(&f);
    let mark = admitted(|a| session.checkpoint(a)).unwrap();
    let value = value(&f, &[8], 2);
    offer(&session, &value);
    let ready = admitted(|a| session.prepare_close(&[&mark], a)).unwrap();
    let prepared = session.close(ready, &[&mark]);
    let independent = f
        .pool
        .attach(&mut Admission {
            global: &mut |_| panic!("occupied prepared root must bypass admission"),
            step: &mut || panic!("occupied prepared root must bypass work"),
        })
        .unwrap();
    assert!(matches!(
        admitted(|a| independent.probe_bank(value.entries(), a)).unwrap(),
        BankProbe::Bypassed
    ));
    prepared.commit();
    drop(mark);
    let next = active_session(&f);
    drop(hit(&next, &value));
    assert_raw(&f);
}

#[test]
fn close_refusal_keeps_original_mark_and_root_restorable() {
    let f = fixture();
    let session = active_session(&f);
    let mark = admitted(|a| session.checkpoint(a)).unwrap();
    let bank = value(&f, &[7], 1);
    offer(&session, &bank);
    let error = session
        .prepare_close(
            &[&mark],
            &mut Admission {
                global: &mut |_| panic!("close does not allocate"),
                step: &mut || Err(DomError::SourceWorkLimit { limit: 23 }),
            },
        )
        .err()
        .unwrap();
    assert!(matches!(error, DomError::SourceWorkLimit { limit: 23 }));
    mark.restore();
    assert!(matches!(
        admitted(|a| session.probe_bank(bank.entries(), a)).unwrap(),
        BankProbe::Miss(_)
    ));
    assert_raw(&f);
}

fn raw_string(value: &DocumentString) -> custody::Usage {
    let capacity = value.shared_code_units().capacity();
    custody::Usage::new(
        size_of::<Vec<u16>>() + capacity * size_of::<u16>(),
        capacity,
    )
}
fn raw_candidate(value: &AttributeStorage) -> custody::Usage {
    let mut raw = custody::Usage::new(
        size_of::<payload::Payload<crate::name::AttributeValues>>()
            + value.capacity() * size_of::<DomAttribute>(),
        0,
    );
    for entry in value.entries() {
        for text in [
            entry.name().namespace_uri(),
            entry.name().prefix(),
            Some(entry.name().local_name()),
            Some(entry.value()),
        ]
        .into_iter()
        .flatten()
        {
            raw = raw.checked_add(raw_string(text)).unwrap();
        }
        for atom in entry
            .name()
            .namespace()
            .into_iter()
            .chain([entry.name().local_name_atom()])
        {
            raw = raw
                .checked_add(custody::Usage::new(size_of::<crate::name::NameRecord>(), 0))
                .unwrap()
                .checked_add(raw_string(atom.text()))
                .unwrap();
        }
    }
    raw
}

#[test]
fn historical_bank_upgrade_preadmits_full_actual_graph_at_exact_and_one_short() {
    let f = fixture();
    let historical = value(&f, &[0xd800, 0, 0xdc00], 19);
    let first = active_session(&f);
    offer(&first, &historical);
    seal(first);
    let session = active_session(&f);
    let original = f.ledger.total();
    let expected = raw_candidate(&historical);
    let limit = original.bytes() + expected.bytes() - 1;
    let error = session
        .probe_bank(
            historical.entries(),
            &mut Admission {
                global: &mut |next| {
                    if next.bytes() > limit {
                        Err(DomError::ResourceLimit {
                            resource: crate::DomResource::SourceRetainedBytes,
                            limit,
                        })
                    } else {
                        Ok(())
                    }
                },
                step: &mut || Ok(()),
            },
        )
        .err()
        .unwrap();
    assert!(
        matches!(error, DomError::ResourceLimit { resource: crate::DomResource::SourceRetainedBytes, limit: actual } if actual == limit)
    );
    assert_eq!(f.ledger.total(), original);
    let limit = original.bytes() + expected.bytes();
    let result = session
        .probe_bank(
            historical.entries(),
            &mut Admission {
                global: &mut |next| {
                    assert!(next.bytes() <= limit);
                    Ok(())
                },
                step: &mut || Ok(()),
            },
        )
        .unwrap();
    let BankProbe::Hit(selected) = result else {
        panic!("actual historical target")
    };
    assert_eq!(selected.0.debit().usage(), expected);
    assert!(selected.storage().same_storage(&historical));
    drop(selected);
    assert_eq!(f.ledger.total(), original);
    assert_raw(&f);
}

#[test]
fn string_reuse_preserves_raw_units_and_cancellation_error_and_releases_owner() {
    let f = fixture();
    let session = active_session(&f);
    let original = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
    assert!(
        admitted(|a| session.share_string(&original, a))
            .unwrap()
            .is_none()
    );
    let equal = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
    let selected = admitted(|a| session.share_string(&equal, a))
        .unwrap()
        .unwrap();
    assert_eq!(selected.value().code_units(), [0xd800, 0, 0xdc00]);
    assert!(Arc::ptr_eq(
        selected.value().shared_code_units(),
        original.shared_code_units()
    ));
    drop(selected);
    let baseline = f.ledger.total();
    let cancelled = AtomicBool::new(true);
    let error = session
        .share_string(
            &equal,
            &mut Admission {
                global: &mut |_| Ok(()),
                step: &mut || crate::store::check_cancel(&cancelled),
            },
        )
        .err()
        .unwrap();
    assert!(matches!(error, DomError::Cancelled));
    assert_eq!(f.ledger.total(), baseline);
    // No session or mark retains the semantic pool owner after Dom/Owner drop.
    let weak = Arc::downgrade(&f.pool.0);
    let owner_bytes = size_of::<Mutex<State>>();
    drop(f.pool);
    assert!(weak.upgrade().is_none());
    assert!(f.ledger.total().bytes() >= owner_bytes); // live session's known Weak body
    drop(session);
}

#[test]
fn long_root_turnover_is_bounded_and_still_admits_new_repeated_banks() {
    let f = fixture();
    let baseline = f.ledger.total();
    let session = active_session(&f);
    for i in 0u16..2048 {
        let bank = value(&f, &[i, 0xd800], 1);
        offer(&session, &bank);
        if i % 128 == 0 {
            assert_raw(&f);
        }
    }
    let repeated = value(&f, &[60000, 0xd800], 11);
    offer(&session, &repeated);
    let equal = value(&f, &[60000, 0xd800], 1);
    assert!(hit(&session, &equal).storage().same_storage(&repeated));
    assert_raw(&f);
    drop(session);
    assert_eq!(f.ledger.total(), baseline);
    assert_raw(&f);
}

#[test]
fn first_real_metadata_allocation_has_exact_one_short_and_retry_boundaries() {
    let f = fixture();
    let session = active_session(&f);
    let bank = value(&f, &[12], 1);
    let baseline = f.ledger.total();
    let requested = banks::Banks::first_allocation_bytes();
    let limit = baseline.bytes() + requested - 1;
    let result = session.offer_bank(
        &bank,
        None,
        &mut Admission {
            global: &mut |next| {
                if next.bytes() > limit {
                    Err(DomError::ResourceLimit {
                        resource: crate::DomResource::SourceRetainedBytes,
                        limit,
                    })
                } else {
                    Ok(())
                }
            },
            step: &mut || Ok(()),
        },
    );
    assert!(
        matches!(result, Err(DomError::ResourceLimit { resource: crate::DomResource::SourceRetainedBytes, limit: actual }) if actual == limit)
    );
    assert_eq!(f.ledger.total(), baseline);
    let limit = baseline.bytes() + requested;
    session
        .offer_bank(
            &bank,
            None,
            &mut Admission {
                global: &mut |next| {
                    assert!(next.bytes() <= limit, "actual raw reservation extent");
                    Ok(())
                },
                step: &mut || Ok(()),
            },
        )
        .unwrap();
    drop(hit(&session, &bank));
    assert_raw(&f);
}

#[test]
fn callbacks_have_no_pool_lock_and_owned_closing_survives_qualifier_exhaustion() {
    let f = fixture();
    let mut steps = 0usize;
    let mut step = || {
        assert!(
            f.pool.0.try_lock().is_ok(),
            "work callback outside pool lock"
        );
        steps += 1;
        Ok(())
    };
    let mut global = |_| {
        assert!(
            f.pool.0.try_lock().is_ok(),
            "admission callback outside pool lock"
        );
        Ok(())
    };
    let mut admission = Admission {
        global: &mut global,
        step: &mut step,
    };
    let session = f.pool.attach(&mut admission).unwrap();
    assert!(session.ensure_active(&mut admission).unwrap().is_some());
    let mark = session.checkpoint(&mut admission).unwrap();
    let bank = value(&f, &[6], 1);
    session.offer_bank(&bank, None, &mut admission).unwrap();
    let text = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
    session.seed_string(&text, &mut admission).unwrap();
    lock(&f.pool.0).banks.exhaust_qualifiers();
    mark.restore();
    let ready = session.prepare_close(&[], &mut admission).unwrap();
    session.close(ready, &[]).commit();
    let bypassed = f.pool.attach(&mut admission).unwrap();
    assert!(matches!(
        bypassed.probe_bank(bank.entries(), &mut admission).unwrap(),
        BankProbe::Bypassed
    ));
    assert!(steps > 0);
    assert_raw(&f);
}

#[test]
fn miss_offers_reject_foreign_owner_and_superseded_sequence_without_retention() {
    let f = fixture();
    let other = fixture();
    let session = active_session(&f);
    let other_session = active_session(&other);
    let bank = value(&f, &[44], 1);
    let oversized = value(&f, &[44], 16_384);
    session
        .offer_bank(
            &oversized,
            None,
            &mut Admission {
                global: &mut |_| panic!("oversized bank bypasses before optional retention"),
                step: &mut || panic!("cached extent rejects before optional array traversal"),
            },
        )
        .unwrap();
    assert_eq!(oversized.entries()[0].value().code_units(), [44]);
    let other_bank = value(&other, &[44], 1);
    let BankProbe::Miss(foreign) =
        admitted(|a| other_session.probe_bank(other_bank.entries(), a)).unwrap()
    else {
        panic!("initial other-owner miss")
    };
    assert_eq!(
        foreign.root,
        session.active_id().unwrap(),
        "owner qualification matters even at the same root ordinal"
    );
    let baseline = f.ledger.total();
    session
        .offer_bank(
            &bank,
            Some(foreign),
            &mut Admission {
                global: &mut |_| panic!("foreign optional miss cannot retain"),
                step: &mut || Ok(()),
            },
        )
        .unwrap();
    assert_eq!(f.ledger.total(), baseline);
    let BankProbe::Miss(stale) = admitted(|a| session.probe_bank(bank.entries(), a)).unwrap()
    else {
        panic!("initial local miss")
    };
    let mark = admitted(|a| session.checkpoint(a)).unwrap();
    session
        .offer_bank(
            &bank,
            Some(stale),
            &mut Admission {
                global: &mut |_| panic!("superseded optional miss cannot retain"),
                step: &mut || Ok(()),
            },
        )
        .unwrap();
    drop(mark);
    assert_eq!(f.ledger.total(), baseline);
    let BankProbe::Miss(current) = admitted(|a| session.probe_bank(bank.entries(), a)).unwrap()
    else {
        panic!("qualified current miss")
    };
    admitted(|a| session.offer_bank(&bank, Some(current), a)).unwrap();
    drop(hit(&session, &bank));
    assert_raw(&f);
}

#[test]
fn unique_repeated_and_mixed_streams_report_bounded_actual_owner_overhead() {
    for mode in ["unique", "repeated", "mixed"] {
        for count in [0usize, 32, 256, 2048] {
            let f = fixture();
            let baseline = f.ledger.total();
            let session = active_session(&f);
            let hot_bank = value(&f, &[0xd800, 60_000], 7);
            let hot_string = DocumentString::from_utf16(vec![0xdc00, 60_000]);
            let mut steps = 0usize;
            let mut peak = baseline.bytes();
            for i in 0..count {
                let row = u16::try_from(i).unwrap();
                let bank = if mode == "repeated" {
                    if i == 0 {
                        hot_bank.clone()
                    } else {
                        value(&f, &[0xd800, 60_000], 1)
                    }
                } else {
                    value(&f, &[0xd800, row], 1)
                };
                let string = if mode == "unique" {
                    DocumentString::from_utf16(vec![0xdc00, row])
                } else if i == 0 {
                    hot_string.clone()
                } else {
                    DocumentString::from_utf16(vec![0xdc00, 60_000])
                };
                let mut admission = Admission {
                    global: &mut |next| {
                        peak = peak.max(next.bytes());
                        Ok(())
                    },
                    step: &mut || {
                        steps += 1;
                        Ok(())
                    },
                };
                let selected = session.share_string(&string, &mut admission).unwrap();
                if mode != "unique" && i != 0 {
                    assert!(Arc::ptr_eq(
                        selected.as_ref().unwrap().value().shared_code_units(),
                        hot_string.shared_code_units()
                    ));
                }
                drop(selected);
                match session.probe_bank(bank.entries(), &mut admission).unwrap() {
                    BankProbe::Hit(selected) => {
                        assert_eq!(mode, "repeated");
                        assert!(selected.storage().same_storage(&hot_bank));
                    }
                    BankProbe::Miss(miss) => session
                        .offer_bank(&bank, Some(miss), &mut admission)
                        .unwrap(),
                    BankProbe::Bypassed => panic!("small eligible stream stays available"),
                }
                assert_raw(&f);
            }
            {
                let state = lock(&f.pool.0);
                let (bank_bytes, weak, live, saved) = state.banks.inspect();
                let strings = state.strings.heap_usage();
                assert!(bank_bytes <= 256 * 1024);
                assert!(strings.bytes() <= 256 * 1024);
                // These are actual controlled occurrence receipts, not a
                // physical-dedup or total-DOM-memory measurement. Input bank
                // and string ownership is outside this private pool fixture.
                eprintln!(
                    "value-sharing-overhead mode={mode} count={count} bank_raw_bytes={bank_bytes} bank_live={live} bank_saved={saved} weak_occurrences={weak} string_receipt_bytes={} pair_receipt_bytes={} authority_bytes={} prospective_ledger_peak={peak} stream_work={steps}",
                    strings.bytes(),
                    state.storage.usage().bytes(),
                    size_of::<Mutex<State>>()
                );
            }
            drop(session);
            assert_eq!(f.ledger.total(), baseline);
            assert_raw(&f);
        }
    }
}

#[test]
fn empty_reserved_values_are_real_candidates_and_oversized_inputs_bypass_boundedly() {
    let f = fixture();
    let session = active_session(&f);
    let baseline = f.ledger.total();
    let absent = AttributeStorage::default();
    admitted(|a| session.offer_bank(&absent, None, a)).unwrap();
    assert_eq!(f.ledger.total(), baseline);
    assert_eq!(lock(&f.pool.0).banks.inspect(), (0, 0, 0, 0));
    let reserved = AttributeStorage::from_entries(Vec::with_capacity(7));
    assert_eq!(reserved.entries().len(), 0);
    offer(&session, &reserved);
    let selected = hit(
        &session,
        &AttributeStorage::from_entries(Vec::with_capacity(1)),
    );
    assert!(selected.storage().same_storage(&reserved));
    assert_eq!(selected.storage().capacity(), reserved.capacity());
    drop(selected);
    // Fresh absent input remains absent; production ingress returns before
    // probing it. A directly offered absent leaf never becomes an entry.
    assert!(absent.payload().is_none());

    let reserved_string = DocumentString::from_utf16(Vec::with_capacity(31));
    assert!(
        admitted(|a| session.share_string(&reserved_string, a))
            .unwrap()
            .is_none()
    );
    let empty = DocumentString::from_utf16(Vec::new());
    let selected = admitted(|a| session.share_string(&empty, a))
        .unwrap()
        .unwrap();
    assert!(Arc::ptr_eq(
        selected.value().shared_code_units(),
        reserved_string.shared_code_units()
    ));
    assert_eq!(
        selected.value().shared_code_units().capacity(),
        reserved_string.shared_code_units().capacity()
    );
    drop(selected);
    assert_eq!(empty.shared_code_units().capacity(), 0);

    let stable = f.ledger.total();
    let mut probe_steps = None;
    for length in [8193, 65_536] {
        let large = DocumentString::from_utf16(vec![0xd800; length]);
        let mut steps = 0;
        assert!(
            session
                .share_string(
                    &large,
                    &mut Admission {
                        global: &mut |_| panic!("oversized string has no optional retention"),
                        step: &mut || {
                            steps += 1;
                            Ok(())
                        },
                    }
                )
                .unwrap()
                .is_none()
        );
        assert_eq!(
            steps, 1,
            "one advertised Hash descriptor visit precedes O(1) eligibility; no unit reads"
        );
        let bank = value(&f, large.code_units(), 1);
        let mut steps = 0;
        assert!(matches!(
            session
                .probe_bank(
                    bank.entries(),
                    &mut Admission {
                        global: &mut |_| panic!(
                            "oversized borrowed bank has no candidate allocation"
                        ),
                        step: &mut || {
                            steps += 1;
                            Ok(())
                        },
                    }
                )
                .unwrap(),
            BankProbe::Bypassed
        ));
        // One attribute and its four presence/extent classifiers plus the
        // fixed ten-unit local name; no oversized value units are inspected.
        assert!(steps <= 1 + 4 + "data-value".len());
        if let Some(previous) = probe_steps {
            assert_eq!(steps, previous);
        }
        probe_steps = Some(steps);
        assert_eq!(large.len(), length);
        assert!(large.code_units().iter().all(|unit| *unit == 0xd800));
        assert_eq!(f.ledger.total(), stable);
    }
    assert_raw(&f);
}

#[test]
fn first_metadata_has_positive_counters_but_reuse_and_bypass_do_not_allocate() {
    if !super::super::scale_metrics::ENABLED {
        return;
    }
    use super::super::scale_metrics::{Hooks, Measurement};
    let f = fixture();
    let session = active_session(&f);
    let bank = value(&f, &[12], 1);
    let measurement = Measurement::begin().unwrap();
    offer(&session, &bank);
    let first = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(
        first.structural_allocations, 2,
        "slot directory and empty base inverse backing"
    );
    assert!(first.structural_bytes > 0);
    assert_eq!(
        first.sequence_materializations, 2,
        "initialized directory and first Before"
    );
    assert!(first.sequence_materialized_bytes > 0);
    let oversized = value(&f, &[12], 16_384);
    let measurement = Measurement::begin().unwrap();
    drop(hit(&session, &bank));
    admitted(|a| session.offer_bank(&oversized, None, a)).unwrap();
    let reuse = measurement.finish(Hooks::default()).unwrap();
    assert_eq!(reuse.structural_allocations, 0);
    assert_eq!(reuse.structural_bytes, 0);
    assert_eq!(reuse.sequence_materializations, 0);
    assert_eq!(reuse.sequence_materialized_bytes, 0);
}

#[test]
fn reentrant_admission_cannot_commit_an_underpriced_bank_shape() {
    let f = fixture();
    let session = active_session(&f);
    let outer = value(&f, &[21], 1);
    let inner = value(&f, &[22], 1);
    let mut entered = false;
    session
        .offer_bank(
            &outer,
            None,
            &mut Admission {
                global: &mut |_| {
                    if !entered {
                        entered = true;
                        offer(&session, &inner);
                    }
                    Ok(())
                },
                step: &mut || Ok(()),
            },
        )
        .unwrap();
    assert!(entered);
    assert!(hit(&session, &inner).storage().same_storage(&inner));
    assert!(matches!(
        admitted(|a| session.probe_bank(outer.entries(), a)).unwrap(),
        BankProbe::Miss(_)
    ));
    assert_raw(&f);
}

#[test]
fn dormant_attachment_defers_roots_and_cold_reset_only_aborts_its_own_pair() {
    let f = fixture();
    let baseline = f.ledger.total();
    let session = f
        .pool
        .attach(&mut Admission {
            global: &mut |_| Ok(()),
            step: &mut || panic!("attaching does not acquire a root"),
        })
        .unwrap();
    assert!(session.is_dormant());
    let attached = f.ledger.total();
    assert_eq!(
        attached.bytes(),
        baseline.bytes() + size_of::<Mutex<State>>()
    );
    let cold = session
        .checkpoint(&mut Admission {
            global: &mut |_| panic!("cold checkpoint has no new authority"),
            step: &mut || panic!("cold checkpoint has no root/mark work"),
        })
        .unwrap();
    assert!(cold.root.is_none());
    let absent = AttributeStorage::default();
    admitted(|a| session.offer_bank(&absent, None, a)).unwrap();
    let oversized = DocumentString::from_utf16(vec![0xd800; 8193]);
    assert!(
        session
            .share_string(
                &oversized,
                &mut Admission {
                    global: &mut |_| panic!("cold oversized value retains nothing"),
                    step: &mut || panic!("DOM extent policy bypasses before root acquisition"),
                }
            )
            .unwrap()
            .is_none()
    );
    assert!(session.is_dormant());
    assert!(lock(&f.pool.0).roots.is_none());
    assert_eq!(f.ledger.total(), attached);
    let bank = value(&f, &[4], 1);
    offer(&session, &bank); // the actual first eligible leaf front activates
    let first = session.active_id().unwrap();
    let active = admitted(|a| session.checkpoint(a)).unwrap();
    let later = value(&f, &[5], 1);
    offer(&session, &later);
    active.restore();
    drop(hit(&session, &bank));
    // Core validates the owning transaction's original cold journal mark
    // before invoking this reset. The leaf aborts only this attachment's ID.
    session.reset_dormant();
    assert!(session.is_dormant());
    assert!(lock(&f.pool.0).roots.is_none());
    assert_eq!(f.ledger.total(), attached);
    drop(cold); // acceptance of the old cold boundary has no reset side effect
    let other = active_session(&f);
    let other_id = other.active_id().unwrap();
    assert_ne!(first, other_id);
    session.reset_dormant();
    assert!(lock(&f.pool.0).active(other_id));
    assert!(matches!(
        admitted(|a| session.probe_bank(bank.entries(), a)).unwrap(),
        BankProbe::Bypassed
    ));
    assert!(!session.is_dormant());
    session.reset_dormant();
    assert!(session.is_dormant());
    assert!(lock(&f.pool.0).active(other_id));
    drop(other);
    drop(session);
    assert_eq!(f.ledger.total(), baseline);
}

#[test]
fn acquisition_errors_cancel_and_unwind_reset_without_lock_or_partial_authority() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let f = fixture();
    let session = admitted(|a| f.pool.attach(a)).unwrap();
    let baseline = f.ledger.total();
    let value = DocumentString::from("eligible");
    for failure in [0, 1, 2] {
        let mut entered = false;
        let attempt = catch_unwind(AssertUnwindSafe(|| {
            session.share_string(
                &value,
                &mut Admission {
                    global: &mut |_| panic!("acquisition retains no new authority"),
                    step: &mut || {
                        entered = true;
                        assert!(f.pool.0.try_lock().is_ok());
                        assert!(session.phase.try_lock().is_ok());
                        assert!(
                            session
                                .share_string(
                                    &value,
                                    &mut Admission {
                                        global: &mut |_| panic!("acquiring reentry cannot admit"),
                                        step: &mut || panic!("acquiring reentry cannot spend work"),
                                    }
                                )
                                .unwrap()
                                .is_none()
                        );
                        match failure {
                            0 => Err(DomError::SourceWorkLimit { limit: 71 }),
                            1 => Err(DomError::Cancelled),
                            _ => std::panic::panic_any(71u8),
                        }
                    },
                },
            )
        }));
        assert!(entered);
        match (failure, attempt) {
            (0, Ok(Err(DomError::SourceWorkLimit { limit: 71 })))
            | (1, Ok(Err(DomError::Cancelled))) => {}
            (2, Err(error)) => assert_eq!(error.downcast_ref::<u8>(), Some(&71)),
            _ => panic!("the original acquisition failure must survive unchanged"),
        }
        assert!(session.is_dormant());
        assert!(lock(&f.pool.0).roots.is_none());
        assert!(!lock(&f.pool.0).banks.occupied());
        assert_eq!(f.ledger.total(), baseline);
    }
    assert!(
        admitted(|a| session.share_string(&value, a))
            .unwrap()
            .is_none()
    );
    assert!(session.active_id().is_some());
    session.reset_dormant();
    assert_eq!(f.ledger.total(), baseline);
}

#[test]
fn dormant_close_and_suspended_empty_mark_preserve_distinct_authority() {
    let f = fixture();
    let baseline = f.ledger.total();
    let dormant = admitted(|a| f.pool.attach(a)).unwrap();
    let ready = dormant
        .prepare_close(
            &[],
            &mut Admission {
                global: &mut |_| panic!("dormant close does not allocate"),
                step: &mut || panic!("dormant close has no seal work"),
            },
        )
        .unwrap();
    dormant.close(ready, &[]).commit();
    assert_eq!(f.ledger.total(), baseline);
    let session = admitted(|a| f.pool.attach(a)).unwrap();
    assert!(
        admitted(|a| session.share_string(&DocumentString::from_utf16(Vec::new()), a))
            .unwrap()
            .is_none()
    );
    let id = session.active_id().unwrap();
    let outer = admitted(|a| session.checkpoint(a)).unwrap();
    lock(&f.pool.0).banks.exhaust_qualifiers();
    let suspended = admitted(|a| session.checkpoint(a)).unwrap();
    assert!(suspended.root.is_none());
    assert!(
        !session.is_dormant(),
        "an empty suspended token is not cold authority"
    );
    suspended.restore();
    assert!(lock(&f.pool.0).active(id));
    assert!(lock(&f.pool.0).roots.as_ref().unwrap().suspended);
    outer.restore();
    assert!(lock(&f.pool.0).active(id));
    session.reset_dormant();
    assert!(session.is_dormant());
    assert!(lock(&f.pool.0).roots.is_none());
    drop(session);
    assert_eq!(f.ledger.total(), baseline);
}

#[test]
fn optional_bank_exhaustion_does_not_break_real_style_preflight_or_acknowledged_tail() {
    use crate::{
        Dom, DomBatch, DomChange, DomLimits, DomSourceEdit, DomSourceProgress, DomSourceRequest,
        DomStyleAction, DomStyleInlineCspObservation, Namespace,
    };
    use surgeist_html::HtmlStyleUpdateAction;
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            32,
            1_000_000,
            128,
            0,
            0,
            32,
            8_000_000,
            DomNameLimits::new(32_000_000, 4096, 1_000_000, 32_000_000).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let style = setup.create_element_ns(&document, Namespace::html(), "style");
    setup.set_attribute(&style, "type", "text/plain");
    let text = setup.create_text(&document, "a{}");
    setup.insert_before(&text, &style, None);
    setup.insert_before(&style, &document, None);
    let commit = dom.apply(setup, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let mut enable = DomBatch::new();
    enable.set_attribute(style, "type", "text/css");
    dom.apply(enable, &cancel).unwrap();
    let before = dom.snapshot();
    let limits =
        || crate::DomSourceLimits::new(128_000_000, 1_000_000, 20_000_000, 64, 4096, 4096, 4);
    let owner = Arc::downgrade(&dom.value_sharing.0);
    // Leave exactly one optional DOM-bank root qualifier for the real first
    // eligible operation. Its later marks/replay may bypass, but cleanup of
    // that acquired root must never issue another qualifier. This does not
    // alter HTML's independent private issuer.
    {
        let owner = owner.upgrade().unwrap();
        let mut state = lock(&owner);
        assert!(state.roots.is_none() && !state.banks.occupied());
        state.banks.leave_last_qualifier();
    }
    let measurement = if super::super::scale_metrics::ENABLED {
        Some(super::super::scale_metrics::Measurement::begin().unwrap())
    } else {
        None
    };
    let (operation, first) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::replace_data(text, 0, usize::MAX, DocumentString::from("b{}")),
                limits(),
                &cancel,
            )
            .unwrap();
        // A cold Source attachment has not spent the last root qualifier.
        // The actual Text command below, not a test helper, activates it.
        {
            let owner = owner.upgrade().unwrap();
            let state = lock(&owner);
            assert!(
                state.roots.is_none() && !state.banks.occupied(),
                "Source attachment stays dormant before eligible formation"
            );
            assert_eq!(state.banks.next_qualifier(), Some(u64::MAX));
        }
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("connected Style Text change reaches inline CSP");
        };
        assert_eq!(request.source().element(), style);
        assert_eq!(request.source().text().as_slice(), [98, 123, 125]);
        assert_eq!(
            request.action(),
            DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp)
        );
        assert!(lease.status().has_published());
        (lease.operation(), request)
    };
    let published = dom.snapshot();
    assert_eq!(
        before.node(text).unwrap().data().unwrap().code_units(),
        [97, 123, 125]
    );
    assert_eq!(
        published.node(text).unwrap().data().unwrap().code_units(),
        [98, 123, 125]
    );
    {
        let mut lease = dom
            .resume_source_edit(operation, limits(), &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("same pending CSP request survives permission drop");
        };
        assert_eq!(request.source().ordinal(), first.source().ordinal());
        assert!(Arc::ptr_eq(request.source().text(), first.source().text()));
        let observation = DomStyleInlineCspObservation::evaluated(
            request.source(),
            Arc::new(vec!["style-src 'none'".into()]),
            19,
            false,
        );
        lease
            .accept_inline_csp(&request, &observation, &cancel)
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("actual blocked CSP acknowledgement completes original Source");
        };
        assert_eq!(done.operation(), operation);
    }
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(
        lock(&owner.upgrade().unwrap()).banks.next_qualifier(),
        None,
        "the actual first eligible activation consumed the last optional root qualifier"
    );
    assert!(!dom.has_style_cssom_roots());
    let after = dom.snapshot();
    assert_eq!(
        after.node(text).unwrap().data().unwrap().code_units(),
        [98, 123, 125]
    );
    assert!(
        after
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    let changes = dom.changes_since(before.stamp()).unwrap();
    assert_eq!(changes.to(), after.stamp());
    assert!(changes.changes().contains(&DomChange::Data(text)));
    assert!(changes.changes().contains(&DomChange::SourceOperation));
    if let Some(measurement) = measurement {
        let counts = measurement
            .finish(super::super::scale_metrics::Hooks::default())
            .unwrap();
        assert!(
            counts.source_steps > 0 && counts.source_preflights > 0 && counts.published_roots > 0
        );
    }
    // Future optional activity bypasses rather than demanding a fresh bank
    // qualifier. Native value formation still publishes exact literal data.
    let mut later = DomBatch::new();
    let node = later.create_element_ns(document, Namespace::html(), "div");
    later.set_attribute(&node, "data-value", "later");
    let detached_text = later.create_text(document, "kept");
    let commit = dom.apply(later, &cancel).unwrap();
    let node = commit.created(&node).unwrap().unwrap();
    let detached_text = commit.created(&detached_text).unwrap().unwrap();
    assert_eq!(
        dom.snapshot()
            .node(node)
            .unwrap()
            .attribute(&DocumentString::from("data-value")),
        Some(&DocumentString::from("later"))
    );
    // ReplaceData always invokes a parent's native children-changed hook.
    // Keep this no-op ownership witness detached from the connected Style,
    // whose equal Text edit legitimately needs another CSP composition.
    let published = dom.snapshot();
    assert!(published.node(detached_text).unwrap().parent().is_none());
    let mut unchanged = DomBatch::new();
    unchanged.replace_data(detached_text, 0, usize::MAX, "kept");
    dom.apply(unchanged, &cancel).unwrap();
    let after_noop = dom.snapshot();
    assert_eq!(after_noop.stamp(), published.stamp());
    assert!(
        dom.changes_since(published.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    let before_text = published.node(detached_text).unwrap().data().unwrap();
    let after_text = after_noop.node(detached_text).unwrap().data().unwrap();
    assert_eq!(after_text.code_units(), [107, 101, 112, 116]);
    assert!(Arc::ptr_eq(
        before_text.shared_code_units(),
        after_text.shared_code_units()
    ));
}
