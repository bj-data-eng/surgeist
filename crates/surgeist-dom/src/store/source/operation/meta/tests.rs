//! Two real phase owners share a ledger/counter, never their diagnostic mirrors.
use super::*;

#[test]
fn reconstructed_segment_charge_uses_its_own_mirrors_and_releases_only_its_positive_increment() {
    let cancel = AtomicBool::new(false);
    let policy = crate::DomLimits::new(
        32,
        100_000,
        128,
        0,
        0,
        16,
        4_000_000,
        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_dispatch_custody(
        crate::DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
    );
    let limits = crate::DomSourceLimits::new(64_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 4);
    let mut dom = Dom::new(policy).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let dialog = setup.create_element_ns(&document, crate::Namespace::html(), "dialog");
    setup.insert_before(&dialog, &document, None);
    let created = dom.apply(setup, &cancel).unwrap();
    let dialog = created.created(&dialog).unwrap().unwrap();
    let executor = dom.bind_source_event_executor(&cancel).unwrap();
    let request = {
        let mut lease = dom
            .begin_source_edit(crate::DomSourceEdit::show_dialog(dialog), limits, &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual beforetoggle observation");
        };
        request
    };
    let ledger = dom.custody_registry.ledger().clone();
    let observed_owner = dom.source_observation_formation(&cancel).unwrap().unwrap();
    let old_bytes = request.work.bytes.clone();
    let old_units = request.work.units.clone();
    let fixed = ledger.total();
    let old_before = old_bytes.load(Ordering::Relaxed);
    let old_units_before = old_units.load(Ordering::Relaxed);
    let mut slots = [None, None];
    let first = observed_owner.fork();
    first
        .retain_payload(
            storage::Payload {
                bytes: 60,
                units: 0,
            },
            &cancel,
        )
        .unwrap();
    let first_backing = Box::new([0u8; 60]);
    charge_slot(&mut slots, first.into_charge()).unwrap();
    assert_eq!(ledger.total().bytes(), fixed.bytes() + 60);
    assert_eq!(ledger.total().units(), fixed.units());
    assert_eq!(old_bytes.load(Ordering::Relaxed), old_before + 60);
    assert_eq!(old_units.load(Ordering::Relaxed), old_units_before);
    // Same/decreased whole reports contribute zero. This owning primitive
    // receives their real zero debit without creating another phase slot.
    for _ in [60, 40] {
        let unchanged = ledger.total();
        charge_slot(&mut slots, observed_owner.fork().into_charge()).unwrap();
        assert_eq!(ledger.total(), unchanged);
        assert!(slots[1].is_none());
    }
    let mut permit = Some(executor.begin_dispatch(&dom, &request, &cancel).unwrap());
    let scope = executor
        .enter_dispatch(&dom, permit.as_ref().unwrap(), &cancel)
        .unwrap();
    drop(scope);
    let mut receipt = Some(
        executor
            .seal_completed(
                &dom,
                &mut permit,
                crate::DomSourceEventOutcome::NotCanceled,
                &cancel,
            )
            .unwrap(),
    );
    let mut lease = dom
        .resume_source_edit(request.operation(), limits, &cancel)
        .unwrap();
    lease
        .accept_source_event(&request, &mut receipt, &cancel)
        .unwrap();
    let progress = lease.advance(&cancel).unwrap();
    assert!(matches!(progress, DomSourceProgress::Request(_)));
    let operation = lease.dom.source_operation.as_ref().unwrap();
    assert!(matches!(
        operation.draft,
        dialog::OperationDraft::Segment(_)
    ));
    assert!(Arc::ptr_eq(
        operation.draft.source_work.source_arc(),
        &request.work.work
    ));
    let new_bytes = operation.draft.source_formation_bytes.clone();
    let new_units = operation.draft.source_temporary_units.clone();
    assert!(!Arc::ptr_eq(&old_bytes, &new_bytes));
    assert!(!Arc::ptr_eq(&old_units, &new_units));
    let segment_owner = formation::Formation::new(&operation.draft, &cancel)
        .unwrap()
        .unwrap();
    let before_increment = ledger.total();
    let old_at_transition = old_bytes.load(Ordering::Relaxed);
    let new_before = new_bytes.load(Ordering::Relaxed);
    let new_units_before = new_units.load(Ordering::Relaxed);
    // The already-retained60 and newly admitted30 represent a whole90-byte
    // union. This is a lower-owner proof, not a public Meta preparation at90.
    let increment = segment_owner.fork();
    increment
        .retain_payload(
            storage::Payload {
                bytes: 30,
                units: 0,
            },
            &cancel,
        )
        .unwrap();
    let second_backing = Box::new([0u8; 30]);
    let increment = increment.into_charge();
    assert!(!slots[0].as_ref().unwrap().same_phase(&increment));
    charge_slot(&mut slots, increment).unwrap();
    assert_eq!(ledger.total().bytes(), before_increment.bytes() + 30);
    assert_eq!(ledger.total().units(), before_increment.units());
    assert_eq!(old_bytes.load(Ordering::Relaxed), old_at_transition);
    assert_eq!(new_bytes.load(Ordering::Relaxed), new_before + 30);
    assert_eq!(new_units.load(Ordering::Relaxed), new_units_before);
    for _ in [90, 40] {
        let unchanged = ledger.total();
        charge_slot(&mut slots, segment_owner.fork().into_charge()).unwrap();
        assert_eq!(ledger.total(), unchanged);
        assert_eq!(old_bytes.load(Ordering::Relaxed), old_at_transition);
        assert_eq!(new_bytes.load(Ordering::Relaxed), new_before + 30);
    }
    let work = request.work.work.load(Ordering::Relaxed);
    drop(second_backing);
    drop(slots[1].take());
    assert_eq!(ledger.total(), before_increment);
    assert_eq!(old_bytes.load(Ordering::Relaxed), old_at_transition);
    assert_eq!(new_bytes.load(Ordering::Relaxed), new_before);
    assert_eq!(new_units.load(Ordering::Relaxed), new_units_before);
    drop(first_backing);
    drop(slots[0].take());
    assert_eq!(ledger.total().bytes(), before_increment.bytes() - 60);
    assert_eq!(ledger.total().units(), before_increment.units());
    assert_eq!(old_bytes.load(Ordering::Relaxed), old_at_transition - 60);
    assert_eq!(old_units.load(Ordering::Relaxed), old_units_before);
    assert_eq!(new_bytes.load(Ordering::Relaxed), new_before);
    assert_eq!(request.work.work.load(Ordering::Relaxed), work);
}
