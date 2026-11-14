use super::*;
use surgeist_dom::{
    DomBatch, DomSourceEdit, HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
    NodeId,
};
use surgeist_html::values::Namespace;

struct Fixture {
    host: DocumentScriptHost,
    permit: DocumentScriptPermit,
    cancel: Arc<AtomicBool>,
    slot: NodeId,
}

fn fixture(source_work: u64, manual: bool) -> Fixture {
    fixture_with_policy(source_work, manual, false)
}

fn fixture_with_policy(source_work: u64, manual: bool, policy: bool) -> Fixture {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session_with_dom_limits(
            &cancel,
            0,
            0,
            |limits| {
                if policy {
                    limits.with_meta_processing_work(1_000_000)
                } else {
                    limits
                }
            },
        );
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(target.document(), Namespace::html(), "div");
    let root = batch.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            if manual {
                HtmlSlotAssignment::Manual
            } else {
                HtmlSlotAssignment::Named
            },
            false,
            false,
            false,
        ),
    );
    let slot = batch.create_element_ns(target.document(), Namespace::html(), "slot");
    let element = batch.create_element_ns(target.document(), Namespace::html(), "span");
    let text = batch.create_text(target.document(), "text");
    let meta = policy.then(|| {
        let meta = batch.create_element_ns(target.document(), Namespace::html(), "meta");
        batch.set_attribute(&meta, "http-equiv", "content-security-policy");
        batch.set_attribute(&meta, "content", "require-trusted-types-for 'script'");
        batch.set_attribute(&element, "onclick", "original");
        meta
    });
    batch.insert_before(&slot, &root, None);
    batch.insert_before(&element, &host, None);
    batch.insert_before(&text, &host, None);
    if manual {
        batch.assign_slot(&slot, vec![(&element).into()]);
    }
    let commit = session.dom().apply(batch, &cancel).unwrap();
    let slot = commit.created(&slot).unwrap().unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    if let Some(meta) = meta {
        session.enable_trusted_types_csp(&cancel).unwrap();
        let meta = commit.created(&meta).unwrap().unwrap();
        let snapshot = session.snapshot();
        let html = snapshot
            .document(target.document())
            .unwrap()
            .document_element()
            .unwrap();
        let head = snapshot.node(html).unwrap().children()[0];
        let full = public_node_source_limits(2_000_000, 64);
        let mut services = crate::DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        };
        let mut source = session.source_execution();
        let crate::DomSourceDriveProgress::Pending {
            request: surgeist_dom::DomSourceRequest::MetaCsp(request),
            ..
        } = source
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(meta, head, None),
                full,
                &mut services,
                &cancel,
            )
            .unwrap()
        else {
            panic!("the original Meta CSP operation must request its owning processor");
        };
        session
            .service_trusted_types_meta_csp(request, full, &cancel)
            .unwrap();
        assert!(matches!(
            source
                .resume(
                    &mut session.dom(),
                    request.operation(),
                    full,
                    &mut services,
                    &cancel
                )
                .unwrap(),
            crate::DomSourceDriveProgress::Complete(_)
        ));
    }
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, source_work);
    let permit = host.permit().unwrap();
    for (node, name) in [
        (slot, "probeSlot"),
        (element, "probeElement"),
        (text, "probeText"),
    ] {
        expose_public_node_for_source_test(&mut host, &permit, node, name);
    }
    Fixture {
        host,
        permit,
        cancel,
        slot,
    }
}

fn invoke(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    script: &str,
) -> Result<(), DocumentScriptError> {
    host.in_document(permit, |scope| assert_guest(scope, script))
}

pub(in crate::document_scripting) fn slot_sequences_admit_exact_work_and_backing() {
    // Literal HTML/DOM sequence oracles are independent of the measured cost.
    // Every result is temporary; the canonical wrappers already have custody.
    for script in [
        "(() => { const nodes = probeSlot.assignedNodes(); return Array.isArray(nodes) && nodes.length === 2 && nodes[0] === probeElement && nodes[1] === probeText; })()",
        "(() => { const nodes = probeSlot.assignedElements({flatten:true}); return Array.isArray(nodes) && nodes.length === 1 && nodes[0] === probeElement; })()",
        "probeElement.assignedSlot === probeSlot && probeText.assignedSlot === probeSlot",
    ] {
        let Fixture {
            mut host, permit, ..
        } = fixture(2_000_000, false);
        let events = host.guest_events().unwrap();
        let before = (events.budget.work().unwrap(), events.budget.used());
        let stamp = host.snapshot().unwrap().stamp();
        let (result, peak) = events
            .budget
            .measure_peak(|| invoke(&mut host, &permit, script));
        result.unwrap();
        let work = events.budget.work().unwrap() - before.0;
        let bytes = peak - before.1;
        assert!(work > 0 && bytes > 0);
        assert_eq!(events.budget.used(), before.1);
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        let budget = events.budget.clone();
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);

        for (available_work, available_bytes, accepted) in [
            (work, bytes, true),
            (work - 1, bytes, false),
            (work, bytes - 1, false),
        ] {
            let Fixture {
                mut host, permit, ..
            } = fixture(2_000_000, false);
            let events = host.guest_events().unwrap();
            let spare = events.limits.storage_bytes - events.budget.used() - available_bytes;
            let fill = events.reserve(&host.native, spare).unwrap();
            let before = events.budget.used();
            let stamp = host.snapshot().unwrap().stamp();
            events.budget.set_work_remaining(available_work);
            let result = invoke(&mut host, &permit, script);
            assert_eq!(host.snapshot().unwrap().stamp(), stamp);
            assert_eq!(events.budget.used(), before);
            if accepted {
                result.unwrap();
                assert_eq!(events.budget.work_remaining().unwrap(), 0);
                assert_eq!(events.budget.peak_used(), events.limits.storage_bytes);
            } else {
                let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                    panic!("one-short Slot observation must refuse: {result:?}");
                };
                assert!(matches!(
                    (
                        available_work < work,
                        failure.cause::<DocumentGuestEventError>()
                    ),
                    (true, Some(DocumentGuestEventError::WorkLimit))
                        | (
                            false,
                            Some(DocumentGuestEventError::Document(
                                DocumentScriptError::Native(
                                    DocumentScriptNativeError::StorageLimit
                                )
                            ))
                        )
                ));
                drop(failure);
            }
            drop(fill);
            let budget = events.budget.clone();
            drop(events);
            host.stop().into_session().unwrap();
            assert_eq!(budget.used(), 0);
        }
    }
}

pub(in crate::document_scripting) fn slot_queries_and_assign_preserve_native_refusal() {
    let Fixture {
        mut host,
        permit,
        slot,
        ..
    } = fixture(0, true);
    let before = host.snapshot().unwrap();
    let events = host.guest_events().unwrap();
    let used = events.budget.used();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "probeSlot.assign(probeText, probeElement, probeText)",
            ))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("Slot assignment must consume the original Source allowance: {result:?}");
    };
    assert!(matches!(
        failure.cause::<crate::DomSourceDriveError>(),
        Some(crate::DomSourceDriveError::Source(
            crate::DomSourceRuntimeError::Dom(DomError::SourceWorkLimit { limit: 0 })
        ))
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(
        after.slot(slot).unwrap().manual_assignments(),
        before.slot(slot).unwrap().manual_assignments()
    );
    assert_eq!(
        after.slot(slot).unwrap().assigned_nodes(),
        before.slot(slot).unwrap().assigned_nodes()
    );
    assert_eq!(events.budget.used(), used);
    drop(failure);
    let budget = events.budget.clone();
    drop(events);
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);

    let Fixture {
        mut host,
        permit,
        cancel,
        ..
    } = fixture(2_000_000, false);
    let cancel_query = cancel.clone();
    host.in_document(&permit, |scope| {
        let callback = scope.function(&Callback::new(move |_, _| {
            cancel_query.store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(Completion::Undefined)
        }))?;
        let global = scope.global()?;
        scope.set(
            &global,
            PropertyKey::String(&units("cancelSlotQuery")),
            &callback,
        )
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    let events = host.guest_events().unwrap();
    let used = events.budget.used();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "probeSlot.assignedNodes({get flatten() { cancelSlotQuery(); return true; }})",
            ))
            .map(drop)
    });
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!(
            "the native query must observe cancellation after dictionary conversion: {result:?}"
        );
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::Cancelled))
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    assert_eq!(events.budget.used(), used);
    drop(failure);
    let budget = events.budget.clone();
    drop(events);
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);
}

pub(in crate::document_scripting) fn slot_queries_consume_the_enclosing_attribute_source() {
    const SOURCE_WORK: u64 = 4_000;
    const QUERIES: usize = 256;
    let Fixture {
        mut host,
        permit,
        slot,
        cancel,
    } = fixture_with_policy(SOURCE_WORK, false, true);
    let entered = Rc::new(Cell::new(0usize));
    let accepted = Rc::new(Cell::new(0usize));
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        for (name, count) in [
            ("beginSlotVerification", entered.clone()),
            ("acceptedSlotQuery", accepted.clone()),
        ] {
            let callback = scope.function(&Callback::new(move |_, _| {
                count.set(count.get() + 1);
                Ok(Completion::Undefined)
            }))?;
            scope.set(&global, PropertyKey::String(&units(name)), &callback)?;
        }
        scope.evaluate(&units(
            r#"
            trustedTypes.createPolicy('default', {createScript(value) {
                beginSlotVerification();
                for (let index = 0; index < 256; index++) {
                    const nodes = probeSlot.assignedNodes({flatten:true});
                    if (nodes.length !== 2 || nodes[0] !== probeElement || nodes[1] !== probeText)
                        throw new Error('wrong original Source sequence');
                    acceptedSlotQuery();
                }
                return value + '!';
            }});
        "#,
        ))?;
        Ok(())
    })
    .unwrap();
    let before = host.snapshot().unwrap();
    let events = host.guest_events().unwrap();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("probeElement.setAttribute('onclick', 'unreached')"))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("Slot traversal must spend the enclosing native Source allowance: {result:?}");
    };
    assert!(
        matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(
                    DomError::SourceWorkLimit { limit: SOURCE_WORK }
                ))
            ))
        ),
        "expected original Source limit, got {:?}",
        failure.cause::<DocumentGuestEventError>()
    );
    assert_eq!(entered.get(), 1);
    assert!(
        accepted.get() > 0 && accepted.get() < QUERIES,
        "accepted query prefix={}",
        accepted.get()
    );
    assert!(events.budget.work_remaining().unwrap() > 0);
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    let pending = host
        .with_native(|dom, _, _| dom.pending_source_edit())
        .unwrap()
        .unwrap();
    assert!(pending.spent_work() <= SOURCE_WORK);
    let element = before.slot(slot).unwrap().assigned_nodes()[0];
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(element)
            .unwrap()
            .attribute(&"onclick".into()),
        Some(&"original".into())
    );
    drop(failure);
    // A query-only verifier has no retained nested attribute reply. Its host
    // failure already cancelled the closed native callback; the exhausted
    // native Source alone owns the unfinished cancellation tail.
    assert!(pending.cancellation_requested());
    let completion = host
        .with_native(|dom, _, _| {
            let mut lease = dom
                .resume_source_edit(
                    pending.operation(),
                    public_node_source_limits(2_000_000, 64),
                    &cancel,
                )
                .unwrap();
            // This cancelled, unaccepted attribute has no reached event or
            // host operation; the native progress front settles its own tail.
            let progress = lease.advance(&cancel).unwrap();
            let surgeist_dom::DomSourceProgress::Complete(completion) = progress else {
                panic!(
                    "the original cancelled verifier must complete its native tail: {progress:?}"
                );
            };
            completion
        })
        .unwrap();
    assert_eq!(completion.operation(), pending.operation());
    assert!(completion.cancellation_requested());
    let budget = events.budget.clone();
    drop(events);
    let mut session = host.stop().into_session().unwrap();
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(
        session
            .snapshot()
            .node(element)
            .unwrap()
            .attribute(&"onclick".into()),
        Some(&"original".into())
    );
    assert_eq!(budget.used(), 0);
}
