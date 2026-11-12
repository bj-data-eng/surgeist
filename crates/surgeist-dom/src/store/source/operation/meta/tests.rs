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
    let dialog = setup.create_element_ns(
        &document,
        surgeist_html::values::Namespace::html(),
        "dialog",
    );
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

/// A block of text with spare capacity, whose first eight units are the text `abcdefgh`.
fn spare_block() -> surgeist_html::values::TextBlock {
    let mut units = Vec::with_capacity(512);
    units.extend("abcdefgh".encode_utf16());
    surgeist_html::values::TextBlock::from_utf16(units)
}

#[test]
fn a_rollback_releases_the_block_with_its_carrier_and_the_earlier_occurrences_keep_theirs() {
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
    .unwrap();
    let mut dom = Dom::new(policy).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let metas = [(); 4].map(|()| {
        setup.create_element_ns(&document, surgeist_html::values::Namespace::html(), "meta")
    });
    let created = dom.apply(setup, &cancel).unwrap();
    let document = created.document(&document).unwrap().node();
    let metas = metas.map(|meta| created.created(&meta).unwrap().unwrap());
    let limits = crate::DomSourceLimits::new(64_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 4);
    let batch = DomBatch::new();
    let (mut draft, _) =
        Draft::new_source(&dom, batch.owner.clone(), &batch.commands, limits, &cancel).unwrap();
    let native = DomMetaAtomicLimits::new(16_000_000, 1_000_000, 1_000_000, 128).unwrap();
    // The original native accounting and the recording, as `prepare_meta_execution` makes them.
    let formation = formation::Formation::new(&draft, &cancel).unwrap().unwrap();
    formation
        .retain_bytes(size_of::<NativeWork>(), &cancel)
        .unwrap();
    let shared = Arc::new(NativeWork {
        work: AtomicU64::new(1),
        outputs: AtomicUsize::new(0),
        peak_bytes: AtomicUsize::new(size_of::<NativeWork>()),
        peak_units: AtomicUsize::new(0),
        private_bytes: AtomicUsize::new(size_of::<NativeWork>()),
        private_units: AtomicUsize::new(0),
        external_bytes: AtomicUsize::new(0),
        external_units: AtomicUsize::new(0),
        active: AtomicBool::new(false),
        _charge: formation.into_charge(),
    });
    let formation = formation::Formation::new(&draft, &cancel).unwrap().unwrap();
    let mut recording = crate::store::meta_atomic::Recording {
        entries: Vec::new(),
        payloads: Default::default(),
        envelope: crate::store::meta_atomic::Envelope::source(
            native,
            DomMetaRuntimeUsage::new(0, 0, 0).unwrap(),
            shared.clone(),
            formation,
            (0, 128),
        )
        .unwrap(),
    };
    recording.attach(&cancel).unwrap();
    draft.meta_recording = Some(recording);
    let units = || shared.private_units.load(Ordering::Relaxed);
    let bytes = || shared.private_bytes.load(Ordering::Relaxed);
    let record =
        |draft: &mut Draft, meta: usize, content: &surgeist_html::values::DocumentString| {
            draft.admit_meta_content(content, &cancel).unwrap();
            draft
                .record_meta_preferred(
                    crate::store::Key::Existing(document),
                    crate::store::Key::Existing(metas[meta]),
                    content.clone(),
                    &cancel,
                )
                .unwrap();
        };
    let (first, second) = (spare_block(), spare_block());
    assert!(first.capacity() >= 512 && second.capacity() >= 512);
    // Each range is charged by its length, and the block once, in the payload of the first
    // occurrence that holds it. The recording's banks grow as entries are added and keep their
    // room on a rollback, so the bytes are compared as differences.
    let occurrence = |len: usize| (len * 2 + size_of::<Vec<u16>>(), len);
    let held = || (bytes(), units());
    let grew =
        |before: (usize, usize), after: (usize, usize)| (after.0 - before.0, after.1 - before.1);
    let start = held();
    record(&mut draft, 0, &first.string(0..4).unwrap());
    let first_charge = (
        occurrence(4).0 + first.retained_bytes(),
        occurrence(4).1 + first.capacity(),
    );
    assert_eq!(grew(start, held()).1, first_charge.1);
    assert!(grew(start, held()).0 >= first_charge.0);
    let ledger = dom.custody_registry.ledger().total();
    let early = control::Snapshot::capture(&draft);
    let before_carrier = held();
    record(&mut draft, 1, &second.string(0..4).unwrap());
    let carrier_charge = (
        occurrence(4).0 + second.retained_bytes(),
        occurrence(4).1 + second.capacity(),
    );
    assert_eq!(grew(before_carrier, held()).1, carrier_charge.1);
    assert!(grew(before_carrier, held()).0 >= carrier_charge.0);
    let late = control::Snapshot::capture(&draft);
    let before_later = held();
    record(&mut draft, 2, &second.string(4..8).unwrap());
    // A later occurrence of the block adds its own length and no block.
    assert_eq!(grew(before_later, held()).1, occurrence(4).1);
    // A rollback of the later occurrence alone releases its length and keeps the block, which its
    // carrier charges.
    let before_restore = held();
    late.restore(&mut draft);
    assert_eq!(
        grew(held(), before_restore),
        (occurrence(4).0, occurrence(4).1)
    );
    assert_eq!(units(), before_later.1);
    record(&mut draft, 3, &second.string(4..8).unwrap());
    // A rollback of the carrier and the later occurrence releases the block with them, and the
    // first occurrence keeps its own charge and its block.
    let before_restore = held();
    early.restore(&mut draft);
    assert_eq!(
        grew(held(), before_restore),
        (
            carrier_charge.0 + occurrence(4).0,
            carrier_charge.1 + occurrence(4).1
        )
    );
    assert_eq!(units(), start.1 + first_charge.1);
    assert_eq!(draft.meta_recording.as_ref().unwrap().entries.len(), 1);
    // The custody ledger holds the same units again; its bytes keep the room of the banks.
    assert_eq!(
        dom.custody_registry.ledger().total().units(),
        ledger.units()
    );
}
