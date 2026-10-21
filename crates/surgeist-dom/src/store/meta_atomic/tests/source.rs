//! Actual original Source invocations; no status/receipt fabrication.
use super::*;
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 4)
}
fn detached(f: &mut Fixture, name: &str) -> NodeId {
    let mut batch = DomBatch::new();
    let meta = batch.create_element_ns(f.docs[0], Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "default-style");
    batch.set_attribute(&meta, "content", name);
    f.dom
        .apply(batch, &flag())
        .unwrap()
        .created(&meta)
        .unwrap()
        .unwrap()
}
#[test]
fn original_source_prepares_actual_meta_and_publishes_once_with_original_completion() {
    let mut f = fixture(u64::MAX);
    let meta = detached(&mut f, "B");
    let stamp = f.dom.snapshot().stamp();
    let caps = f.owners.iter().collect::<Vec<_>>();
    let mut lease = f
        .dom
        .begin_source_edit(
            DomSourceEdit::insert_before(meta, f.heads[0], None),
            source_limits(),
            &flag(),
        )
        .unwrap();
    let operation = lease.operation();
    let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
    let DomMetaSourceStep::Prepared(mut prepared) = hold.prepare(&caps, None, &flag()).unwrap()
    else {
        panic!("actual native boundary");
    };
    assert_eq!(prepared.occurrences().len(), 1);
    assert_eq!(
        prepared.occurrences()[0].content(),
        &DocumentString::from("B")
    );
    assert_eq!(prepared.occurrences()[0].document(), f.docs[0].node());
    prepared
        .set_disabled(f.docs[0].node(), &f.sheets[0], true, &flag())
        .unwrap();
    let published = prepared.commit(&flag()).unwrap();
    assert!(matches!(
        f.owners[0].snapshot(),
        Err(DomStyleCssomRootError::Busy)
    ));
    let (result, dead) = published.into_parts();
    assert_ne!(result.stamp(), stamp);
    assert_eq!(result.roots().len(), 1);
    let DomSourceProgress::Complete(completion) = result.source() else {
        panic!("actual original completion");
    };
    assert_eq!(completion.operation(), operation);
    assert!(!completion.cancellation_requested());
    assert!(matches!(
        hold.step(&flag()),
        Err(DomError::SourceOperationPending)
    ));
    drop(hold);
    drop(lease);
    drop(dead);
    assert!(f.dom.pending_source_edit().is_none());
    assert_eq!(
        f.dom.snapshot().node(meta).unwrap().parent(),
        Some(f.heads[0])
    );
    assert!(
        f.owners[0]
            .snapshot()
            .unwrap()
            .sheet(&f.sheets[0])
            .unwrap()
            .inputs()
            .disabled
    );
}
#[test]
fn normal_preparation_drop_parks_real_phase_and_raises_preserve_original_operation() {
    let mut f = fixture(u64::MAX);
    let meta = detached(&mut f, "B");
    let before = snapshots(&f);
    let caps = f.owners.iter().collect::<Vec<_>>();
    let operation;
    let subject;
    {
        let mut lease = f
            .dom
            .begin_source_edit(
                DomSourceEdit::insert_before(meta, f.heads[0], None),
                source_limits(),
                &flag(),
            )
            .unwrap();
        operation = lease.operation();
        let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
        let DomMetaSourceStep::Prepared(prepared) = hold.prepare(&caps, None, &flag()).unwrap()
        else {
            panic!("actual native phase");
        };
        subject = prepared.occurrences()[0].meta().clone();
        drop(prepared);
    }
    unchanged(&f, &before);
    assert_eq!(f.dom.pending_source_edit().unwrap().operation(), operation);
    let spent = f.dom.pending_source_edit().unwrap().spent_work();
    let mut lease = f
        .dom
        .resume_source_edit(operation, source_limits(), &flag())
        .unwrap();
    assert!(matches!(
        lease.advance(&flag()),
        Err(DomError::Meta(DomMetaError::DefaultStyleUnsupported))
    ));
    let lower = DomMetaAtomicLimits::new(16_000_000, 1_000_000, 999_999, 128).unwrap();
    assert!(matches!(
        lease.begin_meta_runtime(lower, &flag()),
        Err(DomError::SourceLimitReduction)
    ));
    let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
    let DomMetaSourceStep::Prepared(prepared) = hold.prepare(&caps, None, &flag()).unwrap() else {
        panic!("same parked native phase");
    };
    assert_eq!(prepared.occurrences().len(), 1);
    assert_eq!(prepared.occurrences()[0].meta(), &subject);
    let (result, dead) = prepared.commit(&flag()).unwrap().into_parts();
    assert!(
        matches!(result.source(), DomSourceProgress::Complete(c) if c.operation() == operation)
    );
    drop(hold);
    drop(lease);
    drop(dead);
    assert!(spent > 0);
    assert!(f.dom.pending_source_edit().is_none());
}
#[test]
fn bound_policy_survives_drop_before_prepare_and_forgotten_hold_cannot_be_reacquired() {
    for forget in [false, true] {
        let mut f = fixture(u64::MAX);
        let meta = detached(&mut f, "B");
        let caps = f.owners.iter().collect::<Vec<_>>();
        let operation;
        {
            let mut lease = f
                .dom
                .begin_source_edit(
                    DomSourceEdit::insert_before(meta, f.heads[0], None),
                    source_limits(),
                    &flag(),
                )
                .unwrap();
            operation = lease.operation();
            let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
            hold.admit_runtime(DomMetaRuntimeUsage::new(128, 0, 0).unwrap(), &flag())
                .unwrap();
            if forget {
                std::mem::forget(hold);
            } else {
                drop(hold);
            }
        }
        assert_eq!(f.dom.pending_source_edit().unwrap().operation(), operation);
        let mut lease = f
            .dom
            .resume_source_edit(operation, source_limits(), &flag())
            .unwrap();
        if forget {
            assert!(matches!(
                lease.begin_meta_runtime(limits(), &flag()),
                Err(DomError::SourceOperationPending)
            ));
        } else {
            let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
            let DomMetaSourceStep::Prepared(prepared) = hold.prepare(&caps, None, &flag()).unwrap()
            else {
                panic!("original retained invocation");
            };
            let _ = prepared.commit(&flag()).unwrap().into_parts();
        }
    }
}

#[test]
fn forgotten_preparation_keeps_real_graph_debit_and_seals_every_public_hold_entry() {
    let mut f = fixture(u64::MAX);
    let meta = detached(&mut f, "B");
    let ledger = f.dom.custody_registry.ledger().clone();
    let caps = [&f.owners[0], &f.owners[1]];
    let mut lease = f
        .dom
        .begin_source_edit(
            DomSourceEdit::insert_before(meta, f.heads[0], None),
            source_limits(),
            &flag(),
        )
        .unwrap();
    let operation = lease.operation();
    let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
    let mut external = Vec::<u16>::new();
    hold.admit_runtime(DomMetaRuntimeUsage::new(34, 17, 0).unwrap(), &flag())
        .unwrap();
    external.try_reserve_exact(17).unwrap();
    external.extend([7; 17]);
    hold.admit_runtime(
        DomMetaRuntimeUsage::new(external.capacity() * 2, external.capacity(), 0).unwrap(),
        &flag(),
    )
    .unwrap();
    let units = external.capacity();
    let DomMetaSourceStep::Prepared(prepared) = hold.prepare(&caps, None, &flag()).unwrap() else {
        panic!("real phase before forgetting");
    };
    let retained = ledger.total();
    std::mem::forget(prepared);
    assert!(matches!(
        hold.step(&flag()),
        Err(DomError::SourceOperationPending)
    ));
    assert!(matches!(
        hold.admit_runtime(DomMetaRuntimeUsage::new(0, 0, 0).unwrap(), &flag()),
        Err(DomError::SourceOperationPending)
    ));
    assert!(matches!(
        hold.prepare(&caps, None, &flag()),
        Err(DomMetaAtomicError::Dom(DomError::SourceOperationPending))
    ));
    hold.request_cancel().unwrap();
    drop(external);
    drop(hold);
    assert_eq!(retained.units() - ledger.total().units(), units);
    drop(lease);
    assert!(matches!(
        f.owners[0].snapshot(),
        Err(DomStyleCssomRootError::Busy)
    ));
    assert_eq!(f.dom.pending_source_edit().unwrap().operation(), operation);
    assert!(
        f.dom
            .pending_source_edit()
            .unwrap()
            .cancellation_requested()
    );
    let mut lease = f
        .dom
        .resume_source_edit(operation, source_limits(), &flag())
        .unwrap();
    assert!(matches!(
        lease.begin_meta_runtime(limits(), &flag()),
        Err(DomError::SourceOperationPending)
    ));
    assert!(matches!(
        lease.advance(&flag()),
        Err(DomError::Meta(DomMetaError::DefaultStyleUnsupported))
    ));
    drop(lease);
    // Disposal of the actual canonical Operation cannot release the forgotten
    // preparation's own graph/debit or original counter dependencies.
    drop(f.dom);
    assert!(ledger.total().bytes() > 0);
}
#[test]
fn actual_late_producer_refusal_keeps_the_original_source_pending_and_all_owners_unchanged() {
    let mut f = fixture(1);
    let mut batch = DomBatch::new();
    let meta = batch.create_element_ns(f.docs[1], Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "default-style");
    batch.set_attribute(&meta, "content", "B");
    let meta = f
        .dom
        .apply(batch, &flag())
        .unwrap()
        .created(&meta)
        .unwrap()
        .unwrap();
    let before = snapshots(&f);
    let caps = [&f.owners[0], &f.owners[1]];
    let operation;
    {
        let mut lease = f
            .dom
            .begin_source_edit(
                DomSourceEdit::insert_before(meta, f.heads[1], None),
                source_limits(),
                &flag(),
            )
            .unwrap();
        operation = lease.operation();
        let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
        let DomMetaSourceStep::Prepared(mut prepared) = hold.prepare(&caps, None, &flag()).unwrap()
        else {
            panic!("actual second owner");
        };
        prepared
            .set_disabled(f.docs[1].node(), &f.sheets[1], true, &flag())
            .unwrap();
        assert!(
            matches!(prepared.commit(&flag()), Err(DomMetaAtomicError::Cssom { root, error: CssomError::RevisionExhausted }) if root == f.docs[1].node())
        );
    }
    unchanged(&f, &before);
    assert_eq!(f.dom.pending_source_edit().unwrap().operation(), operation);
    assert!(f.dom.pending_source_edit().unwrap().spent_work() > 0);
}
#[test]
fn source_retries_require_the_actual_retained_producer_basis() {
    let mut f = fixture(u64::MAX);
    let meta = detached(&mut f, "B");
    let caps = [&f.owners[0], &f.owners[1]];
    let operation;
    {
        let mut lease = f
            .dom
            .begin_source_edit(
                DomSourceEdit::insert_before(meta, f.heads[0], None),
                source_limits(),
                &flag(),
            )
            .unwrap();
        operation = lease.operation();
        let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
        let DomMetaSourceStep::Prepared(prepared) = hold.prepare(&caps, None, &flag()).unwrap()
        else {
            panic!("real private basis");
        };
        drop(prepared);
    }
    let before = f.dom.snapshot().stamp();
    let snapshot = f.owners[0].snapshot().unwrap();
    let mut batch = Some(
        f.owners[0]
            .batch(CssomEditGuard::from_snapshot(&snapshot))
            .unwrap(),
    );
    batch
        .as_mut()
        .unwrap()
        .set_disabled(&f.sheets[0], true)
        .unwrap();
    f.owners[0].commit(&mut batch).unwrap();
    let mut lease = f
        .dom
        .resume_source_edit(operation, source_limits(), &flag())
        .unwrap();
    let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
    assert!(
        matches!(hold.prepare(&caps, None, &flag()), Err(DomMetaAtomicError::StaleProducer { root }) if root == f.docs[0].node())
    );
    drop(hold);
    drop(lease);
    assert_eq!(f.dom.snapshot().stamp(), before);
    assert_eq!(f.dom.snapshot().node(meta).unwrap().parent(), None);
    assert_eq!(f.dom.pending_source_edit().unwrap().operation(), operation);
}
#[test]
fn authentic_observed_ancestor_pays_child_native_prefix_and_original_parent_progress_survives() {
    use std::sync::atomic::Ordering;
    let mut f = fixture(u64::MAX);
    f.dom
        .set_limits(f.dom.limits().with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        ))
        .unwrap();
    let meta = detached(&mut f, "B");
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(f.docs[0], Namespace::html(), "dialog");
    batch.insert_before(&dialog, f.heads[0], None);
    let dialog = f
        .dom
        .apply(batch, &flag())
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap();
    let executor = f.dom.bind_source_event_executor(&flag()).unwrap();
    let caps = [&f.owners[0], &f.owners[1]];
    let request = {
        let mut lease = f
            .dom
            .begin_source_edit(DomSourceEdit::show_dialog(dialog), source_limits(), &flag())
            .unwrap();
        let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
        let DomMetaSourceStep::Progress(progress) = hold.prepare(&caps, None, &flag()).unwrap()
        else {
            panic!("no Meta in the original event prefix");
        };
        assert!(progress.inline().is_none());
        let (DomSourceProgress::Request(DomSourceRequest::Event(request)), None) =
            progress.into_parts()
        else {
            panic!("actual beforetoggle event");
        };
        request
    };
    let mut permit = Some(executor.begin_dispatch(&f.dom, &request, &flag()).unwrap());
    let scope = executor
        .enter_dispatch(&f.dom, permit.as_ref().unwrap(), &flag())
        .unwrap();
    {
        let mut lease = f
            .dom
            .begin_source_edit(
                DomSourceEdit::insert_before(meta, f.heads[0], None),
                source_limits(),
                &flag(),
            )
            .unwrap();
        let child = lease.operation();
        let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
        let before = request.work.work.load(Ordering::Relaxed);
        hold.step(&flag()).unwrap();
        assert_eq!(request.work.work.load(Ordering::Relaxed), before + 1);
        let DomMetaSourceStep::Prepared(prepared) = hold.prepare(&caps, None, &flag()).unwrap()
        else {
            panic!("actual observer-created Meta child");
        };
        let (result, dead) = prepared.commit(&flag()).unwrap().into_parts();
        assert!(
            matches!(result.source(), DomSourceProgress::Complete(c) if c.operation() == child)
        );
        drop(dead);
    }
    assert_eq!(
        f.dom.pending_source_edit().unwrap().operation(),
        request.operation()
    );
    drop(scope);
    let mut receipt = Some(
        executor
            .seal_completed(
                &f.dom,
                &mut permit,
                DomSourceEventOutcome::NotCanceled,
                &flag(),
            )
            .unwrap(),
    );
    let mut lease = f
        .dom
        .resume_source_edit(request.operation(), source_limits(), &flag())
        .unwrap();
    lease
        .accept_source_event(&request, &mut receipt, &flag())
        .unwrap();
    let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
    let ledger = request.work.storage.ledger();
    let retained = ledger.total();
    let observed_bytes = request.work.bytes.load(Ordering::Relaxed);
    let observed_units = request.work.units.load(Ordering::Relaxed);
    hold.admit_runtime(DomMetaRuntimeUsage::new(60, 0, 0).unwrap(), &flag())
        .unwrap();
    let external = Box::new([0u8; 60]);
    let admitted = ledger.total();
    assert_eq!(admitted.bytes(), retained.bytes() + 60);
    assert_eq!(admitted.units(), retained.units());
    assert_eq!(
        request.work.bytes.load(Ordering::Relaxed),
        observed_bytes + 60
    );
    assert_eq!(request.work.units.load(Ordering::Relaxed), observed_units);
    for same_or_smaller in [60, 40] {
        hold.admit_runtime(
            DomMetaRuntimeUsage::new(same_or_smaller, 0, 0).unwrap(),
            &flag(),
        )
        .unwrap();
        assert_eq!(ledger.total(), admitted);
        assert_eq!(
            request.work.bytes.load(Ordering::Relaxed),
            observed_bytes + 60
        );
        assert_eq!(request.work.units.load(Ordering::Relaxed), observed_units);
    }
    let before = request.work.work.load(Ordering::Relaxed);
    let progress = hold.prepare(&caps, None, &flag()).unwrap();
    assert!(matches!(progress, DomMetaSourceStep::Progress(_)));
    assert!(request.work.work.load(Ordering::Relaxed) > before);
    drop(progress);
    drop(external);
    drop(hold);
    drop(lease);
    assert!(receipt.is_none());
    assert!(
        !f.dom
            .source_operation
            .as_ref()
            .unwrap()
            .is_observing_event()
    );
}

#[test]
fn finite_native_prefix_refusal_preserves_spent_source_work_for_a_raised_retry() {
    let mut f = fixture(u64::MAX);
    let meta = detached(&mut f, "B");
    let before = snapshots(&f);
    let operation;
    {
        let mut lease = f
            .dom
            .begin_source_edit(
                DomSourceEdit::insert_before(meta, f.heads[0], None),
                source_limits(),
                &flag(),
            )
            .unwrap();
        operation = lease.operation();
        let small = DomMetaAtomicLimits::new(16_000_000, 1_000_000, 2, 128).unwrap();
        let hold = lease.begin_meta_runtime(small, &flag()).unwrap();
        assert!(matches!(
            hold.step(&flag()),
            Err(DomError::Meta(DomMetaError::WorkLimit { limit: 2 }))
        ));
    }
    unchanged(&f, &before);
    let spent = f.dom.pending_source_edit().unwrap().spent_work();
    assert!(spent > 0);
    let caps = [&f.owners[0], &f.owners[1]];
    let mut lease = f
        .dom
        .resume_source_edit(operation, source_limits(), &flag())
        .unwrap();
    // Reacquisition itself is admitted by the same original controller.
    assert!(lease.status().spent_work() >= spent);
    let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
    let DomMetaSourceStep::Prepared(prepared) = hold.prepare(&caps, None, &flag()).unwrap() else {
        panic!("same actual invocation after raising native work");
    };
    let (result, dead) = prepared.commit(&flag()).unwrap().into_parts();
    assert!(
        matches!(result.source(), DomSourceProgress::Complete(c) if c.operation() == operation)
    );
    drop(dead);
}

#[test]
fn actual_late_cancel_and_zero_output_refusal_preserve_the_original_phase_without_publication() {
    for cancel_late in [false, true] {
        let mut f = fixture(u64::MAX);
        let meta = detached(&mut f, "B");
        let before = snapshots(&f);
        let caps = [&f.owners[0], &f.owners[1]];
        let operation;
        {
            let mut lease = f
                .dom
                .begin_source_edit(
                    DomSourceEdit::insert_before(meta, f.heads[0], None),
                    source_limits(),
                    &flag(),
                )
                .unwrap();
            operation = lease.operation();
            let policy = if cancel_late {
                limits()
            } else {
                DomMetaAtomicLimits::new(16_000_000, 1_000_000, 1_000_000, 0).unwrap()
            };
            let mut hold = lease.begin_meta_runtime(policy, &flag()).unwrap();
            let DomMetaSourceStep::Prepared(mut prepared) =
                hold.prepare(&caps, None, &flag()).unwrap()
            else {
                panic!("actual Meta phase before output admission");
            };
            prepared
                .set_disabled(f.docs[0].node(), &f.sheets[0], true, &flag())
                .unwrap();
            if cancel_late {
                let canceled = AtomicBool::new(true);
                assert!(matches!(
                    prepared.commit(&canceled),
                    Err(DomMetaAtomicError::Dom(DomError::Cancelled))
                ));
            } else {
                assert!(matches!(
                    prepared.commit(&flag()),
                    Err(DomMetaAtomicError::Dom(DomError::ResourceLimit {
                        resource: DomResource::MetaAtomicOutputs,
                        limit: 0
                    }))
                ));
            }
        }
        unchanged(&f, &before);
        assert_eq!(f.dom.pending_source_edit().unwrap().operation(), operation);
        let mut lease = f
            .dom
            .resume_source_edit(operation, source_limits(), &flag())
            .unwrap();
        let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
        let DomMetaSourceStep::Prepared(prepared) = hold.prepare(&caps, None, &flag()).unwrap()
        else {
            panic!("original pending native invocation");
        };
        let (result, dead) = prepared.commit(&flag()).unwrap().into_parts();
        assert!(
            matches!(result.source(), DomSourceProgress::Complete(c) if c.operation() == operation)
        );
        drop(dead);
    }
}

#[test]
fn accepted_style_receipt_holder_is_admitted_before_busy_owner_access_and_retry_keeps_real_sheet() {
    let mut f = fixture_binding(u64::MAX, false);
    // The Style creation protocol admits the actual associated sheet order.
    // This genuine empty owner has no unrelated constructed sheet to remove.
    f.owners[0] = DomStyleCssomRoot::new(
        f.docs[0].node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    let mut setup = DomBatch::new();
    let style = setup.create_element_ns(f.docs[0], Namespace::html(), "style");
    let style = f
        .dom
        .apply(setup, &flag())
        .unwrap()
        .created(&style)
        .unwrap()
        .unwrap();
    let caps = [&f.owners[0], &f.owners[1]];
    let mut lease = f
        .dom
        .begin_source_edit(
            DomSourceEdit::insert_before(style, f.heads[0], None),
            source_limits(),
            &flag(),
        )
        .unwrap();
    let operation = lease.operation();
    let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
        lease.advance(&flag()).unwrap()
    else {
        panic!("real inline CSP request");
    };
    lease
        .accept_inline_csp(
            &policy,
            &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
            &flag(),
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
        lease.advance(&flag()).unwrap()
    else {
        panic!("real Style creation request");
    };
    lease
        .bind_cssom_root(&create, &f.owners[0], &flag())
        .unwrap();
    let create = lease
        .qualify_cssom(&create, f.docs[0].node(), &f.owners[0].snapshot().unwrap())
        .unwrap();
    let version = CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: "actual-source-style".into(),
        revision: 1,
    };
    lease
        .bind_cssom_owner(
            &create,
            DomStyleCssomOwnerBinding::new(&create, version.clone(), "real-style".into()).unwrap(),
            &flag(),
        )
        .unwrap();
    let parser = CssParserContext::default();
    let body = parser
        .parse_sheet_utf16_with_limits(
            create.source().text().clone(),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let media = parser
        .parse_media_query_list_utf16_with_limits(
            std::sync::Arc::new(Vec::new()),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let mut inputs = CssomSheetInputs::external(version, true);
    inputs.owner = Some("real-style".into());
    let mut batch = f.owners[0]
        .batch(CssomEditGuard::from_snapshot(
            &f.owners[0].snapshot().unwrap(),
        ))
        .unwrap();
    let construction = batch
        .create_parsed_sheet_with_media_reports(body, media, inputs)
        .unwrap();
    let commit = f.owners[0].commit(&mut Some(batch)).unwrap();
    let sheet = commit.sheet(construction.ticket()).unwrap().clone();
    lease
        .accept_cssom_publication(
            &create,
            commit,
            f.owners[0].snapshot().unwrap(),
            Some(construction.ticket()),
        )
        .unwrap();
    let accepted = f.owners[0].snapshot().unwrap();
    let stamp = lease.snapshot().stamp();
    let busy = f.owners[0].acquire().unwrap();
    let bytes = size_of::<crate::store::source::operation::meta::NativeWork>()
        + size_of::<DomMetaSourceRuntimeHold<'_, '_>>();
    let small = DomMetaAtomicLimits::new(bytes, 1_000_000, 1_000_000, 128).unwrap();
    let mut hold = lease.begin_meta_runtime(small, &flag()).unwrap();
    assert!(
        matches!(hold.prepare(&caps, None, &flag()), Err(DomMetaAtomicError::Dom(DomError::ResourceLimit { resource: DomResource::MetaAtomicRetainedBytes, limit })) if limit == bytes)
    );
    drop(hold);
    drop(busy);
    assert_eq!(lease.snapshot().stamp(), stamp);
    drop(lease);
    assert_eq!(f.dom.pending_source_edit().unwrap().operation(), operation);
    assert_eq!(
        f.owners[0].snapshot().unwrap().revision(),
        accepted.revision()
    );
    let mut lease = f
        .dom
        .resume_source_edit(operation, source_limits(), &flag())
        .unwrap();
    let mut hold = lease.begin_meta_runtime(limits(), &flag()).unwrap();
    let DomMetaSourceStep::Progress(progress) = hold.prepare(&caps, None, &flag()).unwrap() else {
        panic!("actual no-Meta Style completion");
    };
    assert!(
        matches!(progress.source(), DomSourceProgress::Complete(c) if c.operation() == operation)
    );
    drop(hold);
    drop(lease);
    let final_cssom = f.owners[0].snapshot().unwrap();
    assert_eq!(final_cssom.revision(), accepted.revision());
    assert!(final_cssom.sheet(&sheet).is_ok());
    assert_eq!(
        f.dom
            .snapshot()
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap()
            .id(),
        &sheet
    );
}
