use super::*;
use crate::store::{bank, name_registry, payload};
use crate::{DomNameLimits, DomOwnerId, DomQualifiedName};
use std::sync::atomic::AtomicBool;
use surgeist_html::values::DocumentString;

mod fixture_controls;
mod work_measurement;

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
    let capacity = value.retained_units();
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
