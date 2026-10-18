use super::*;

#[test]
fn qualified_rows_keep_original_ordinals_and_refusal_preserves_prior_capture_rows() {
    let cancel = AtomicBool::new(false);
    let limits = DomLimits::new(64, 16384, 128, 0, 0, 8, 1_000_000)
        .unwrap()
        .with_select_processing(32, 8_000_000, 10_000_000)
        .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let first = batch.create_text(&document, "first");
    let second = batch.create_comment(&document, "second");
    let commit = dom.apply(batch, &cancel).unwrap();
    let first = commit.created(&first).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let empty = DomBatch::new();
    let generous =
        crate::DomSourceLimits::new(500_000_000, 20_000_000, 100_000_000, 64, 4096, 4096, 1);
    let mut draft = Draft::new_source(&dom, empty.owner, &empty.commands, generous, &cancel)
        .unwrap()
        .0;
    let original = draft.custody_ledger().total();
    let original_scratch = draft.select_temporary_bytes.load(Ordering::Relaxed);
    let mut rows = Rows::new(&draft, &cancel).unwrap();
    rows.insert(&draft, Key::Existing(first), 19, &cancel)
        .unwrap();
    assert_eq!(rows.get(&draft, Key::Existing(first), &cancel).unwrap(), 19);
    let wrong_generation = NodeId {
        generation: first.generation + 1,
        ..first
    };
    assert_eq!(
        rows.get(&draft, Key::Existing(wrong_generation), &cancel),
        Err(DomError::SelectProtocol)
    );
    let foreign = NodeId {
        owner: DomOwnerId::allocate().unwrap(),
        ..first
    };
    assert_eq!(
        rows.get(&draft, Key::Existing(foreign), &cancel),
        Err(DomError::SelectProtocol)
    );
    assert_eq!(
        rows.insert(&draft, Key::Existing(foreign), 3, &cancel),
        Err(DomError::ForeignOwner)
    );
    let mut no_headroom = generous;
    no_headroom.retained_bytes = draft.custody_ledger().total().bytes() + draft.source_other_bytes;
    draft.source_limits = Some(no_headroom);
    let retained = draft.custody_ledger().total();
    let scratch = draft.select_temporary_bytes.load(Ordering::Relaxed);
    assert!(matches!(
        rows.insert(&draft, Key::Existing(second), 2, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            ..
        })
    ));
    assert_eq!(rows.get(&draft, Key::Existing(first), &cancel).unwrap(), 19);
    assert_eq!(
        rows.get(&draft, Key::Existing(second), &cancel),
        Err(DomError::SelectProtocol)
    );
    assert_eq!(draft.custody_ledger().total(), retained);
    assert_eq!(
        draft.select_temporary_bytes.load(Ordering::Relaxed),
        scratch
    );
    draft.source_limits = Some(generous);
    let scratch_limit = draft.select_scratch_limit;
    draft.select_scratch_limit = draft.effect_bytes + scratch;
    assert!(matches!(
        rows.insert(&draft, Key::Existing(second), 2, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::SelectScratch,
            ..
        })
    ));
    assert_eq!(draft.custody_ledger().total(), retained);
    assert_eq!(
        draft.select_temporary_bytes.load(Ordering::Relaxed),
        scratch
    );
    assert_eq!(rows.get(&draft, Key::Existing(first), &cancel).unwrap(), 19);
    draft.select_scratch_limit = scratch_limit;
    cancel.store(true, Ordering::Release);
    assert_eq!(
        rows.insert(&draft, Key::Existing(second), 2, &cancel),
        Err(DomError::Cancelled)
    );
    assert_eq!(draft.custody_ledger().total(), retained);
    assert_eq!(
        draft.select_temporary_bytes.load(Ordering::Relaxed),
        scratch
    );
    cancel.store(false, Ordering::Release);
    rows.insert(&draft, Key::Existing(second), 2, &cancel)
        .unwrap();
    assert_eq!(rows.get(&draft, Key::Existing(second), &cancel).unwrap(), 2);
    drop(rows);
    draft.custody_registry().reconcile_releases();
    assert_eq!(draft.custody_ledger().total(), original);
    assert_eq!(
        draft.select_temporary_bytes.load(Ordering::Relaxed),
        original_scratch
    );
}

#[test]
fn local_admin_and_header_admit_exact_select_bytes_and_refuse_one_short_on_the_same_owner() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(64, 16384, 128, 0, 0, 8, 1_000_000)
            .unwrap()
            .with_select_processing(32, 8_000_000, 10_000_000)
            .unwrap(),
    )
    .unwrap();
    let mut setup = DomBatch::new();
    setup.create_document();
    dom.apply(setup, &cancel).unwrap();
    for source in [false, true] {
        let empty = DomBatch::new();
        let mut draft = if source {
            Draft::new_source(
                &dom,
                empty.owner,
                &empty.commands,
                crate::DomSourceLimits::new(
                    500_000_000,
                    20_000_000,
                    100_000_000,
                    64,
                    4096,
                    4096,
                    1,
                ),
                &cancel,
            )
            .unwrap()
            .0
        } else {
            Draft::new(&dom, empty.owner, &cancel).unwrap()
        };
        let before = draft.custody_ledger().total();
        let mirror = draft.select_temporary_bytes.load(Ordering::Relaxed);
        // This complete empty-map boundary consists only of the literal Rows
        // header, Ledger Mutex target and Registry Mutex target. There are no
        // record/page paths and no empirical full-operation price.
        let width = size_of::<Rows>()
            + custody::Ledger::administration_usage().bytes()
            + bank::CustodyRegistry::administration_usage().bytes();
        let exact = draft.effect_bytes + mirror + width;
        draft.select_scratch_limit = exact - 1;
        assert!(
            matches!(Rows::new(&draft,&cancel),Err(DomError::ResourceLimit{resource:DomResource::SelectScratch,limit}) if limit==exact-1)
        );
        assert_eq!(draft.custody_ledger().total(), before);
        assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), mirror);
        draft.select_scratch_limit = exact;
        let rows = Rows::new(&draft, &cancel).unwrap();
        assert_eq!(rows.bridge.usage(), custody::Usage::new(width, 0));
        assert_eq!(
            draft.custody_ledger().total(),
            before.checked_add(custody::Usage::new(width, 0)).unwrap()
        );
        assert_eq!(
            draft.select_temporary_bytes.load(Ordering::Relaxed),
            mirror + width
        );
        drop(rows);
        assert_eq!(draft.custody_ledger().total(), before);
        assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), mirror);
        if let Some(mut policy) = draft.source_limits {
            let source_exact = before.bytes() + draft.source_other_bytes + width;
            policy.retained_bytes = source_exact - 1;
            draft.source_limits = Some(policy);
            assert!(
                matches!(Rows::new(&draft,&cancel),Err(DomError::ResourceLimit{resource:DomResource::SourceRetainedBytes,limit}) if limit==source_exact-1)
            );
            assert_eq!(draft.custody_ledger().total(), before);
            assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), mirror);
            policy.retained_bytes = source_exact;
            draft.source_limits = Some(policy);
            let rows = Rows::new(&draft, &cancel).unwrap();
            assert_eq!(rows.bridge.usage(), custody::Usage::new(width, 0));
            drop(rows);
            assert_eq!(draft.custody_ledger().total(), before);
            assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), mirror);
        }
    }
}
