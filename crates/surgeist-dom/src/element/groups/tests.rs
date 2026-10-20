use super::*;
use crate::store::custody::Ledger;
use std::sync::atomic::Ordering;

#[test]
fn fresh_nonce_admits_the_actual_target_and_empty_string_before_allocation() {
    let required = size_of::<payload::Payload<NonceSlot>>() + size_of::<Vec<u16>>();
    for allowance in [required - 1, required] {
        let ledger = Ledger::default();
        let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
        let baseline = ledger.total();
        let cancel = AtomicBool::new(false);
        let mut target_admitted = false;
        let mut admit = |next: Usage| {
            // This is the first target gate, not a whole registry-plan bound.
            // Later shared-debit and custody-path administration is separate.
            if !target_admitted && next.bytes() > baseline.bytes() {
                assert_eq!(next.bytes() - baseline.bytes(), required);
                target_admitted = true;
                if required > allowance {
                    return Err(DomError::AllocationFailed);
                }
            }
            Ok(())
        };
        let mut formation = ElementFormation::new(&registry, None, &cancel, &mut admit);
        let result = formation.fresh_nonce();
        if allowance < required {
            assert!(matches!(result, Err(DomError::AllocationFailed)));
            drop(formation);
            registry.reconcile_releases();
            assert_eq!(ledger.total(), baseline);
        } else {
            let value = result.unwrap();
            assert!(value.value.value.is_empty());
            assert_eq!(value.value.value.shared_code_units().capacity(), 0);
            assert_eq!(value.own_usage, Usage::new(required, 0));
            drop(value);
            drop(formation);
            registry.reconcile_releases();
            assert_eq!(ledger.total(), baseline);
        }
        assert!(target_admitted);
    }
}

#[test]
fn reserved_nonce_preserves_its_actual_backing_and_releases_after_semantics() {
    let ledger = Ledger::default();
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let baseline = ledger.total();
    let cancel = AtomicBool::new(false);
    let raw = Vec::<u16>::with_capacity(37);
    let text = DocumentString::from_utf16(raw);
    let weak = Arc::downgrade(text.shared_code_units());
    let raw_capacity = text.shared_code_units().capacity();
    let mut admit = |_| Ok(());
    let mut formation = ElementFormation::new(&registry, None, &cancel, &mut admit);
    let value = formation
        .nonce(text.clone(), storage::Payload::default())
        .unwrap();
    assert!(Arc::ptr_eq(
        value.value.value.shared_code_units(),
        text.shared_code_units()
    ));
    assert_eq!(
        value.own_usage,
        Usage::new(
            size_of::<payload::Payload<NonceSlot>>()
                + size_of::<Vec<u16>>()
                + raw_capacity * size_of::<u16>(),
            raw_capacity
        )
    );
    drop(text);
    let snapshot = value.clone();
    let controls = formation.into_children();
    drop(value);
    assert!(weak.upgrade().is_some());
    drop(snapshot);
    assert!(
        weak.upgrade().is_none(),
        "semantic targets die before their controls"
    );
    assert!(ledger.total().bytes() > baseline.bytes());
    drop(controls);
    registry.reconcile_releases();
    assert_eq!(ledger.total(), baseline);
}

#[test]
fn cancelled_group_construction_keeps_original_semantics_and_controls() {
    let ledger = Ledger::default();
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let baseline = ledger.total();
    let cancel = AtomicBool::new(true);
    let text = DocumentString::from("actual-original");
    let weak = Arc::downgrade(text.shared_code_units());
    let mut admit = |_| panic!("cancellation precedes allocation admission");
    let mut formation = ElementFormation::new(&registry, None, &cancel, &mut admit);
    assert!(matches!(
        formation.nonce(text.clone(), storage::Payload::default()),
        Err(DomError::Cancelled)
    ));
    assert_eq!(ledger.total(), baseline);
    assert_eq!(
        text.code_units(),
        "actual-original".encode_utf16().collect::<Vec<_>>()
    );
    assert!(weak.upgrade().is_some());
    cancel.store(false, Ordering::Relaxed);
    drop(formation);
    drop(text);
    assert!(weak.upgrade().is_none());
}
