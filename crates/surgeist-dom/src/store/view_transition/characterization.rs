// Included inside the owning tests module to reuse its admitted CSSOM fixture.
fn sparse_fixture(records: usize) -> (Dom, DocumentId, NodeId, DomViewTransitionId) {
    let (mut dom, document) = fixture();
    dom.set_limits(
        DomLimits::new(32, 100_000, 512, 0, 0, 8, 100_000)
            .unwrap()
            .with_view_transitions(32, 4096, 20_000)
            .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let origin = batch.create_element(document, "root");
    batch.insert_before(&origin, document, None);
    let stamp = dom.snapshot().document(document).unwrap().stamp();
    let transition = batch.begin_view_transition(stamp);
    for index in 0..records {
        let record = batch.append_view_transition_record(
            stamp,
            &transition,
            CssViewTransitionIdent::try_new(
                CssCustomIdent::try_new(format!("participant-{index}")).unwrap(),
            )
            .unwrap(),
        );
        batch.set_view_transition_branches(stamp, &record, DomViewTransitionBranches::Old);
    }
    batch.setup_view_transition_pseudos(stamp, &transition);
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    crate::store::oracle_tests::assert_full_canonical(&dom);
    (
        dom,
        document,
        done.created(&origin).unwrap().unwrap(),
        done.view_transition(&transition).unwrap().unwrap(),
    )
}

#[test]
fn canonical_transition_capture_shares_roots_and_adds_no_retained_object() {
    use crate::store::scale_metrics::{self, Hooks, Measurement};
    let (dom, _, _, _) = sparse_fixture(128);
    let before = dom.custody_registry.ledger().total();
    let measurement = scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
    let captured =
        DraftTransitions::capture(&dom.state.view_transitions, &dom.state_custody.transitions);
    if let Some(measurement) = measurement {
        // An inline root capture traverses no producer; edit tests separately
        // require actual Bank and Accounting hook registration.
        let actual = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(actual.bank_page_visits, 0);
        assert_eq!(actual.structural_allocations, 0);
        assert_eq!(actual.record_materializations, 0);
    }
    assert_eq!(dom.custody_registry.ledger().total(), before);
    assert_eq!(captured.usage().unwrap(), (1, 128, 385));
}

#[test]
fn fixed_root_flag_edit_reuses_participants_and_matches_full_change_oracle() {
    use crate::store::scale_metrics::{self, Domain, Hooks, Measurement};
    for size in [1, 128] {
        let (mut dom, document, _, transition) = sparse_fixture(size);
        let before = dom.snapshot();
        let root = before.view_transition(transition).unwrap().root();
        let mut batch = DomBatch::new();
        batch.set_view_transition_captured(
            before.document(document).unwrap().stamp(),
            DomViewTransitionSubject::Pseudo(root),
            true,
        );
        let measurement = scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
        let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        if let Some(measurement) = measurement {
            let actual = measurement
                .finish(Hooks::one(Domain::Bank).union(Hooks::one(Domain::Accounting)))
                .unwrap();
            // The two literal changes are written to the VT output Vec, moved
            // into the combined output Vec, and copied into Arc<[DomChange]>.
            // Origin tracking writes one announced generation; publication
            // inserts one HistoryEntry. Tickets, record deltas and Document
            // deltas are empty, so resize/reorder guards emit nothing. The
            // unchanged participant bank stays shared below.
            assert_eq!(actual.sequence_materializations, 5, "{actual:?}");
            assert_eq!(
                actual.sequence_materialized_bytes,
                6 * std::mem::size_of::<DomChange>()
                    + std::mem::size_of::<u64>()
                    + std::mem::size_of::<crate::store::HistoryEntry>(),
                "{actual:?}"
            );
            assert!(actual.record_materializations < 256, "{actual:?}");
        }
        let after = dom.snapshot();
        let old = before
            .state
            .view_transitions
            .transition(dom.owner(), transition)
            .unwrap();
        let new = after
            .state
            .view_transitions
            .transition(dom.owner(), transition)
            .unwrap();
        assert!(old.records.shares_root(&new.records));
        assert_eq!(
            done.changes().changes(),
            &[
                DomChange::ViewTransition(transition),
                DomChange::ViewTransitionCaptured(DomViewTransitionSubject::Pseudo(root))
            ]
        );
        assert_eq!(
            full_difference(
                dom.owner(),
                &before.state,
                &after.state,
                &AtomicBool::new(false)
            )
            .unwrap(),
            done.changes().changes()
        );
        crate::store::oracle_tests::assert_full_canonical(&dom);
        let mut restore = DomBatch::new();
        let stamp = after.document(document).unwrap().stamp();
        restore.set_view_transition_captured(stamp, DomViewTransitionSubject::Pseudo(root), false);
        restore.set_view_transition_captured(stamp, DomViewTransitionSubject::Pseudo(root), true);
        assert!(
            dom.apply(restore, &AtomicBool::new(false))
                .unwrap()
                .changes()
                .changes()
                .is_empty()
        );
        assert_eq!(dom.snapshot().stamp(), after.stamp());
        crate::store::oracle_tests::assert_full_canonical(&dom);
    }
}

#[test]
fn origin_membership_includes_completed_transition_and_preserves_old_snapshot() {
    let (mut dom, document, origin, first) = sparse_fixture(2);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    let stamp = before.document(document).unwrap().stamp();
    batch.clear_view_transition_state(stamp, first);
    let second = batch.begin_view_transition(stamp);
    batch.setup_view_transition_pseudos(stamp, &second);
    let second = dom
        .apply(batch, &AtomicBool::new(false))
        .unwrap()
        .view_transition(&second)
        .unwrap()
        .unwrap();
    assert_eq!(
        dom.state
            .view_transitions
            .origin_transitions(document)
            .collect::<Vec<_>>(),
        vec![first, second]
    );
    let mut batch = DomBatch::new();
    batch.detach(origin);
    let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    for transition in [first, second] {
        assert!(
            done.changes()
                .changes()
                .contains(&DomChange::ViewTransition(transition))
        );
    }
    assert_eq!(
        before
            .view_transition_pseudo(before.view_transition(first).unwrap().root())
            .unwrap()
            .originating_element(),
        origin
    );
    let mut batch = DomBatch::new();
    batch.release_view_transition(dom.snapshot().document(document).unwrap().stamp(), first);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        dom.state
            .view_transitions
            .origin_transitions(document)
            .collect::<Vec<_>>(),
        vec![second]
    );
    assert_eq!(before.view_transition(first).unwrap().records().len(), 2);
    crate::store::oracle_tests::assert_full_canonical(&dom);
}

#[test]
fn source_phase_keeps_original_transition_and_record_ids_in_final_receipts() {
    let (mut dom, document) = fixture();
    let cancel = AtomicBool::new(false);
    let mut create = DomBatch::new();
    let parent = create.create_element(document, "parent");
    let style = create.create_element_ns(document, crate::Namespace::html(), "style");
    create.insert_before(&parent, document, None);
    let created = dom.apply(create, &cancel).unwrap();
    let parent = created.created(&parent).unwrap().unwrap();
    let style = created.created(&style).unwrap().unwrap();
    let before = dom.snapshot();
    let stamp = before.document(document).unwrap().stamp();
    let mut batch = DomBatch::new();
    let transition = batch.begin_view_transition(stamp);
    let record = batch.append_view_transition_record(
        stamp,
        &transition,
        CssViewTransitionIdent::try_new(CssCustomIdent::try_new("participant").unwrap()).unwrap(),
    );
    batch.set_view_transition_branches(stamp, &record, DomViewTransitionBranches::Old);
    batch.insert_before(style, parent, None);
    batch.set_attribute(parent, "data-tail", "finished");
    let mut source = dom
        .begin_source_commands(batch, source_append_limits(4_000_000), &cancel)
        .unwrap();
    let crate::DomSourceProgress::Request(crate::DomSourceRequest::Style(request)) =
        source.advance(&cancel).unwrap()
    else {
        panic!("actual inserted Style policy boundary");
    };
    let phase = source.snapshot();
    let original_transition = phase
        .document(document)
        .unwrap()
        .view_transitions()
        .unwrap()
        .active()
        .unwrap();
    let original_record = phase
        .view_transition(original_transition)
        .unwrap()
        .records()
        .next()
        .unwrap()
        .id();
    let observation = crate::DomStyleInlineCspObservation::evaluated(
        request.source(),
        Arc::new(vec!["style-src 'none'".into()]),
        1,
        false,
    );
    source
        .accept_inline_csp(&request, &observation, &cancel)
        .unwrap();
    let crate::DomSourceProgress::Complete(done) = source.advance(&cancel).unwrap() else {
        panic!("blocked Style resumes original synchronous tail");
    };
    assert_eq!(
        done.commit().view_transition(&transition).unwrap(),
        Some(original_transition)
    );
    assert_eq!(
        done.commit().view_transition_record(&record).unwrap(),
        Some(original_record)
    );
    assert_eq!(
        source
            .snapshot()
            .view_transition_record(original_record)
            .unwrap()
            .name()
            .as_str(),
        "participant"
    );
    assert_eq!(
        phase.view_transition_record(original_record).unwrap().id(),
        original_record
    );
    assert!(before.view_transition(original_transition).is_err());
}

#[test]
fn active_outer_custody_prunes_nested_participant_adoption_without_describing_records() {
    use crate::store::scale_metrics::{self, Domain, Hooks, Measurement};
    let (dom, _, _, _) = sparse_fixture(128);
    dom.custody_registry.reconcile_releases();
    let before = dom.custody_registry.ledger().total();
    let measurement = scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
    let mut descriptions = 0usize;
    let adopted = dom
        .state
        .view_transitions
        .adopt_custody(
            &dom.custody_registry,
            |_| Ok(()),
            || {
                descriptions += 1;
                Ok(())
            },
        )
        .unwrap();
    if let Some(measurement) = measurement {
        let actual = measurement.finish(Hooks::one(Domain::Accounting)).unwrap();
        assert_eq!(actual.accounting_record_visits, 0);
        assert_eq!(actual.record_materializations, 0);
        assert_eq!(actual.structural_allocations, 0);
    }
    assert_eq!(descriptions, 0);
    assert_eq!(dom.custody_registry.ledger().total(), before);
    let captured = DraftTransitions::capture(&dom.state.view_transitions, &adopted);
    assert_eq!(captured.usage().unwrap(), (1, 128, 385));
}

#[test]
fn historical_transition_adoption_visits_new_frontier_once_and_releases_all_child_custody() {
    let (dom, _, _, transition) = sparse_fixture(128);
    let historical = dom.snapshot();
    drop(dom);
    let ledger = custody::Ledger::default();
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    let baseline = ledger.total();
    let mut described = 0usize;
    let adopted = historical
        .state
        .view_transitions
        .adopt_custody(
            &registry,
            |_| Ok(()),
            || {
                described += 1;
                Ok(())
            },
        )
        .unwrap();
    // One Document state, one Transition, 128 Records, one Document membership
    // header and its one retained Transition identity are actual new records.
    assert_eq!(described, 132);
    let current = ledger.total();
    let mut described_again = 0usize;
    let again = historical
        .state
        .view_transitions
        .adopt_custody(
            &registry,
            |_| Ok(()),
            || {
                described_again += 1;
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(described_again, 0);
    assert_eq!(ledger.total(), current);
    let captured = DraftTransitions::capture(&historical.state.view_transitions, &again);
    assert_eq!(
        captured
            .get(TransitionKey::Existing(transition))
            .unwrap()
            .record_count(),
        128
    );
    drop(captured);
    drop(adopted);
    drop(again);
    registry.reconcile_releases();
    assert_eq!(ledger.total(), baseline);
    assert_eq!(
        historical
            .view_transition(transition)
            .unwrap()
            .records()
            .len(),
        128
    );
}

#[test]
fn document_deltas_preserve_reverse_initialization_order_through_noop_and_retirement() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(DomLimits::new(32, 10_000, 128, 0, 0, 8, 100_000).unwrap()).unwrap();
    let mut create = DomBatch::new();
    let first = create.create_document();
    let second = create.create_document();
    let third = create.create_document();
    let done = dom.apply(create, &cancel).unwrap();
    let first = done.document(&first).unwrap();
    let second = done.document(&second).unwrap();
    let third = done.document(&third).unwrap();
    let mut css = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = css
        .batch(CssomEditGuard::from_snapshot(&css.snapshot()))
        .unwrap();
    let sheet = edit
        .create_parsed_sheet(
            "",
            CssomSheetInputs::constructed(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "document delta order".into(),
                    revision: 1,
                },
                "fixture".into(),
                None,
            ),
        )
        .unwrap();
    let sheet = css.commit(edit).unwrap().sheet(&sheet).unwrap().clone();
    let input = DomPreparedViewTransitionSheet::capture(
        &css.snapshot(),
        &sheet,
        DomViewTransitionCaptureLimits::new(16_384, 1_000_000),
        &cancel,
    )
    .unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    for document in [third, second, first] {
        batch
            .initialize_view_transitions(before.document(document).unwrap().stamp(), input.clone());
    }
    let done = dom.apply(batch, &cancel).unwrap();
    assert_eq!(
        done.changes().changes(),
        &[
            DomChange::ViewTransitionDocument(third),
            DomChange::ViewTransitionDocument(second),
            DomChange::ViewTransitionDocument(first)
        ]
    );
    let current = dom.snapshot();
    let mut batch = DomBatch::new();
    for document in [first, third, second] {
        batch.set_view_transition_rendering_suppression(
            current.document(document).unwrap().stamp(),
            true,
        );
    }
    let done = dom.apply(batch, &cancel).unwrap();
    assert_eq!(
        done.changes().changes(),
        &[
            DomChange::ViewTransitionDocument(third),
            DomChange::ViewTransitionDocument(second),
            DomChange::ViewTransitionDocument(first)
        ]
    );
    let retained = dom.snapshot();
    let usage = dom.usage();
    let next_ordinal = dom.state.view_transitions.next_document;
    let mut noop = DomBatch::new();
    let stamp = retained.document(second).unwrap().stamp();
    noop.initialize_view_transitions(stamp, input.clone());
    noop.set_view_transition_rendering_suppression(stamp, false);
    noop.set_view_transition_rendering_suppression(stamp, true);
    assert!(
        dom.apply(noop, &cancel)
            .unwrap()
            .changes()
            .changes()
            .is_empty()
    );
    assert_eq!(dom.snapshot().stamp(), retained.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(dom.state.view_transitions.next_document, next_ordinal);
    assert!(
        dom.state
            .view_transitions
            .documents
            .shares_root(&retained.state.view_transitions.documents)
    );
    let before_refusal = dom.snapshot();
    let mut retire = DomBatch::new();
    retire.destroy(second.node());
    assert_eq!(
        dom.apply(retire, &cancel).unwrap_err(),
        DomError::InvalidDestruction
    );
    assert_eq!(*dom.state, *before_refusal.state);
    assert_eq!(dom.snapshot().stamp(), before_refusal.stamp());
    let mut create = DomBatch::new();
    let replacement = create.create_document();
    let done = dom.apply(create, &cancel).unwrap();
    let replacement = done.document(&replacement).unwrap();
    let current = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.initialize_view_transitions(
        current.document(replacement).unwrap().stamp(),
        input.clone(),
    );
    for document in [first, third] {
        batch.set_view_transition_rendering_suppression(
            current.document(document).unwrap().stamp(),
            false,
        );
    }
    let done = dom.apply(batch, &cancel).unwrap();
    assert_eq!(
        done.changes().changes(),
        &[
            DomChange::ViewTransitionDocument(third),
            DomChange::ViewTransitionDocument(first),
            DomChange::ViewTransitionDocument(replacement)
        ]
    );
    assert!(
        retained
            .document(second)
            .unwrap()
            .view_transitions()
            .unwrap()
            .rendering_suppressed()
    );
    let mut create = DomBatch::new();
    let uninitialized = create.create_document();
    let uninitialized = dom
        .apply(create, &cancel)
        .unwrap()
        .document(&uninitialized)
        .unwrap();
    let mut initialize = DomBatch::new();
    initialize.initialize_view_transitions(
        dom.snapshot().document(uninitialized).unwrap().stamp(),
        input,
    );
    let (mut draft, _) = Draft::new_source(
        &dom,
        initialize.owner.clone(),
        &initialize.commands,
        source_append_limits(4_000_000),
        &cancel,
    )
    .unwrap();
    draft.view_transitions.next_document = usize::MAX;
    let retained = draft.controlled_storage_usage(&cancel).unwrap();
    let Command::ViewTransition { basis, edit } = &initialize.commands[0] else {
        unreachable!()
    };
    assert_eq!(
        draft.apply_view_transition(*basis, edit, dom.limits(), &cancel),
        Err(DomError::RepresentationExhausted)
    );
    assert_eq!(draft.controlled_storage_usage(&cancel).unwrap(), retained);
    assert_eq!(draft.view_transitions.next_document, usize::MAX);
}
