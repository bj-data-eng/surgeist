use super::*;
use surgeist_dom::{DomBatch, identity::NodeId, source::DomSourceEdit};
use surgeist_html::values::Namespace;

struct Fixture {
    host: DocumentScriptHost,
    permit: DocumentScriptPermit,
    cancel: Arc<AtomicBool>,
    pre: NodeId,
}

fn fixture(tag: &str, raw: &str, source_work: u64) -> Fixture {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let mut batch = DomBatch::new();
    let pre = batch.create_element_ns(target.document(), Namespace::html(), tag);
    let pre = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&pre)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, source_work);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, pre, "probePre");
    // The raw input is a native fixture. Neither JS conversion nor the setter's
    // formatting/copy work is charged to the getter observation under test.
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(pre, Namespace::none(), "width", raw);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    Fixture {
        host,
        permit,
        cancel,
        pre,
    }
}

fn observe(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    expected: i32,
) -> Result<(), DocumentScriptError> {
    host.in_document(permit, |scope| {
        let actual = scope.evaluate(&units("probePre.width"))?;
        let expected = scope.primitive(Primitive::Number(f64::from(expected)))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        Ok(())
    })
}

fn invoke(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    script: &str,
) -> Result<(), DocumentScriptError> {
    host.in_document(permit, |scope| scope.evaluate(&units(script)).map(drop))
}

pub(in crate::document_scripting) fn numeric_getters_admit_raw_units_and_large_leading_zeroes() {
    const COHORT_WORK: u64 = 1_000_000;
    const ZEROES: usize = 8193;
    for tag in ["pre", "listing", "xmp"] {
        let Fixture {
            mut host, permit, ..
        } = fixture(tag, "", 2_000_000);
        let events = host.guest_events().unwrap();
        events.budget.set_work_remaining(COHORT_WORK);
        let before = (events.budget.work().unwrap(), events.budget.used());
        let (result, peak) = events
            .budget
            .measure_peak(|| observe(&mut host, &permit, 0));
        result.unwrap();
        let empty_work = events.budget.work().unwrap() - before.0;
        // HTML's documented raw-unit envelope is 8*n+16. The same authentic
        // native family/attribute lookup and V8 observation add fixed overhead.
        let overhead = empty_work.checked_sub(16).unwrap();
        let temporary_bytes = peak - before.1;
        assert!(overhead > 0 && temporary_bytes > 0);
        assert_eq!(events.budget.used(), before.1);
        let budget = events.budget.clone();
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);

        let raw = format!("{}7", "0".repeat(ZEROES));
        let required = overhead + 8 * u64::try_from(raw.len()).unwrap() + 16;
        for (available_work, available_bytes, accepted) in [
            (required, temporary_bytes, true),
            (required - 1, temporary_bytes, false),
            (required, temporary_bytes - 1, false),
        ] {
            let Fixture {
                mut host, permit, ..
            } = fixture(tag, &raw, 2_000_000);
            let events = host.guest_events().unwrap();
            let spare = events.limits.storage_bytes - events.budget.used() - available_bytes;
            let fill = events.reserve(&host.native, spare).unwrap();
            let before = (
                events.budget.work().unwrap(),
                events.budget.used(),
                host.snapshot().unwrap().stamp(),
            );
            events.budget.set_work_remaining(available_work);
            let result = observe(&mut host, &permit, 7);
            assert_eq!(events.budget.used(), before.1);
            assert_eq!(host.snapshot().unwrap().stamp(), before.2);
            if accepted {
                result.unwrap();
                assert_eq!(events.budget.work().unwrap() - before.0, required);
                assert_eq!(events.budget.work_remaining().unwrap(), 0);
                assert_eq!(events.budget.peak_used(), events.limits.storage_bytes);
            } else {
                let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                    panic!("one-short numeric observation must refuse: {result:?}");
                };
                assert!(matches!(
                    (
                        available_work < required,
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
                let Err(DocumentScriptError::Script(Error::HostFailure(repeated))) =
                    host.in_document(&permit, |_| Ok(()))
                else {
                    panic!("the original numeric observation refusal must remain terminal");
                };
                assert!(std::ptr::eq(
                    failure.cause::<DocumentGuestEventError>().unwrap(),
                    repeated.cause::<DocumentGuestEventError>().unwrap()
                ));
            }
            drop(fill);
            let budget = events.budget.clone();
            drop(events);
            host.stop().into_session().unwrap();
            assert_eq!(budget.used(), 0);
        }
    }

    for (suffix, negative, expected) in [
        ("2147483647", false, i32::MAX),
        ("2147483648", true, i32::MIN),
        ("2147483648", false, 0),
        ("2147483649", true, 0),
    ] {
        let raw = format!(
            "{}{}{}tail",
            if negative { "-" } else { "" },
            "0".repeat(ZEROES),
            suffix
        );
        let Fixture {
            mut host, permit, ..
        } = fixture("pre", &raw, 2_000_000);
        let events = host.guest_events().unwrap();
        events.budget.set_work_remaining(COHORT_WORK);
        let stamp = host.snapshot().unwrap().stamp();
        observe(&mut host, &permit, expected).unwrap();
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        let budget = events.budget.clone();
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}

pub(in crate::document_scripting) fn numeric_setters_admit_actual_temporary_work_and_storage() {
    const SCRIPT: &str = "probePre.width = -2147483648";
    let Fixture {
        mut host,
        permit,
        pre,
        ..
    } = fixture("pre", "1", 2_000_000);
    let events = host.guest_events().unwrap();
    let before = (events.budget.work().unwrap(), events.budget.used());
    let (result, peak) = events
        .budget
        .measure_peak(|| invoke(&mut host, &permit, SCRIPT));
    result.unwrap();
    let required = events.budget.work().unwrap() - before.0;
    let bytes = peak - before.1;
    assert!(required > 0 && bytes > 0);
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(pre)
            .unwrap()
            .attribute(&"width".into()),
        Some(&"-2147483648".into())
    );
    assert_eq!(events.budget.used(), before.1);
    let budget = events.budget.clone();
    drop(events);
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);
    for (work, storage, accepted) in [
        (required, bytes, true),
        (required - 1, bytes, false),
        (required, bytes - 1, false),
    ] {
        let Fixture {
            mut host,
            permit,
            pre,
            ..
        } = fixture("pre", "1", 2_000_000);
        let events = host.guest_events().unwrap();
        let fill = events
            .reserve(
                &host.native,
                events.limits.storage_bytes - events.budget.used() - storage,
            )
            .unwrap();
        let before = (events.budget.used(), host.snapshot().unwrap());
        events.budget.set_work_remaining(work);
        let result = invoke(&mut host, &permit, SCRIPT);
        assert_eq!(events.budget.used(), before.0);
        if accepted {
            result.unwrap();
            assert_eq!(events.budget.work_remaining().unwrap(), 0);
            assert_eq!(
                host.snapshot()
                    .unwrap()
                    .node(pre)
                    .unwrap()
                    .attribute(&"width".into()),
                Some(&"-2147483648".into())
            );
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short numeric setter must refuse: {result:?}");
            };
            assert!(matches!(
                (work < required, failure.cause::<DocumentGuestEventError>()),
                (true, Some(DocumentGuestEventError::WorkLimit))
                    | (
                        false,
                        Some(DocumentGuestEventError::Document(
                            DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
                        ))
                    )
            ));
            assert_eq!(host.snapshot().unwrap().stamp(), before.1.stamp());
            assert_eq!(
                host.snapshot()
                    .unwrap()
                    .node(pre)
                    .unwrap()
                    .attribute(&"width".into()),
                Some(&"1".into())
            );
        }
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        drop(fill);
        let budget = events.budget.clone();
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }

    let Fixture {
        mut host,
        permit,
        pre,
        ..
    } = fixture("pre", "1", 0);
    let before = host.snapshot().unwrap();
    let result = invoke(&mut host, &permit, "probePre.width = 7");
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("the numeric setter must consume its original Source allowance: {result:?}");
    };
    assert!(matches!(
        failure.cause::<crate::internal::dom_source::DomSourceDriveError>(),
        Some(crate::internal::dom_source::DomSourceDriveError::Source(
            crate::internal::dom_source::DomSourceRuntimeError::Dom(DomError::SourceWorkLimit {
                limit: 0
            })
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(pre)
            .unwrap()
            .attribute(&"width".into()),
        Some(&"1".into())
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    drop(failure);
    host.stop().into_session().unwrap();
}

pub(in crate::document_scripting) fn numeric_queries_preserve_cancellation_and_original_source_custody()
 {
    let Fixture {
        mut host,
        permit,
        cancel,
        ..
    } = fixture("pre", "7", 2_000_000);
    let cancel_read = cancel.clone();
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        let callback = scope.function(&Callback::new(move |_, _| {
            cancel_read.store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(Completion::Undefined)
        }))?;
        scope.set(
            &global,
            PropertyKey::String(&units("cancelWidthRead")),
            &callback,
        )
    })
    .unwrap();
    let events = host.guest_events().unwrap();
    let before = (events.budget.used(), host.snapshot().unwrap().stamp());
    let result = invoke(&mut host, &permit, "cancelWidthRead(); probePre.width");
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("numeric cancellation must refuse: {result:?}")
    };
    // Cancellation is already set before V8 enters the width callback. Its
    // frame backing reaches hosted retention's guard before NativeRead.
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Hosted(
                crate::internal::dom_lifecycle::DomHostedRuntimeError::Cancelled
            ))
        ))
    ));
    assert_eq!(events.budget.used(), before.0);
    assert_eq!(host.snapshot().unwrap().stamp(), before.1);
    drop(failure);
    let budget = events.budget.clone();
    drop(events);
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);
    numeric_queries_consume_the_enclosing_attribute_source();
}

fn numeric_queries_consume_the_enclosing_attribute_source() {
    const SOURCE_WORK: u64 = 4_000;
    const QUERIES: usize = 128;
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session_with_dom_limits(
            &cancel,
            0,
            0,
            |limits| limits.with_meta_processing_work(1_000_000),
        );
    let mut batch = DomBatch::new();
    let pre = batch.create_element_ns(target.document(), Namespace::html(), "pre");
    batch.set_attribute_ns(&pre, Namespace::none(), "width", "00000007");
    let element = batch.create_element_ns(target.document(), Namespace::html(), "span");
    batch.set_attribute(&element, "onclick", "original");
    let meta = batch.create_element_ns(target.document(), Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "content-security-policy");
    batch.set_attribute(&meta, "content", "require-trusted-types-for 'script'");
    let commit = session.dom().apply(batch, &cancel).unwrap();
    let pre = commit.created(&pre).unwrap().unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let meta = commit.created(&meta).unwrap().unwrap();
    session.enable_trusted_types_csp(&cancel).unwrap();
    let snapshot = session.snapshot();
    let html = snapshot
        .document(target.document())
        .unwrap()
        .document_element()
        .unwrap();
    let head = snapshot.node(html).unwrap().children()[0];
    let full = public_node_source_limits(2_000_000, 64);
    let mut services = crate::internal::dom_source::DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let mut source = session.source_execution();
    let crate::internal::dom_source::DomSourceDriveProgress::Pending {
        request: surgeist_dom::source::DomSourceRequest::MetaCsp(request),
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
        panic!("the real Meta CSP operation must reach its native processor");
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
        crate::internal::dom_source::DomSourceDriveProgress::Complete(_)
    ));
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, SOURCE_WORK);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, pre, "probePre");
    expose_public_node_for_source_test(&mut host, &permit, element, "probeElement");
    let entered = Rc::new(Cell::new(0usize));
    let accepted = Rc::new(Cell::new(0usize));
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        for (name, count) in [("beginWidthVerification", entered.clone()), ("acceptedWidthQuery", accepted.clone())] {
            let callback = scope.function(&Callback::new(move |_, _| {
                count.set(count.get() + 1);
                Ok(Completion::Undefined)
            }))?;
            scope.set(&global, PropertyKey::String(&units(name)), &callback)?;
        }
        scope.evaluate(&units(r#"
            trustedTypes.createPolicy('default', {createScript(value) {
                beginWidthVerification();
                for (let index = 0; index < 128; index++) {
                    if (probePre.width !== 7) throw new Error('incorrect signed width in original Source callback');
                    acceptedWidthQuery();
                }
                return value + '!';
            }});
        "#))?;
        Ok(())
    }).unwrap();
    let events = host.guest_events().unwrap();
    events.budget.set_work_remaining(1_000_000);
    let before = host.snapshot().unwrap();
    let result = invoke(
        &mut host,
        &permit,
        "probeElement.setAttribute('onclick', 'unreached')",
    );
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("numeric inspection must spend the enclosing Source bank: {result:?}");
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
        "original Source cause: {:?}",
        failure.cause::<DocumentGuestEventError>()
    );
    assert_eq!(entered.get(), 1);
    assert!(
        accepted.get() > 0 && accepted.get() < QUERIES,
        "accepted width prefix={}",
        accepted.get()
    );
    assert!(events.budget.work_remaining().unwrap() > 0);
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(element)
            .unwrap()
            .attribute(&"onclick".into()),
        Some(&"original".into())
    );
    let pending = host
        .with_native(|dom, _, _| dom.pending_source_edit())
        .unwrap()
        .unwrap();
    assert!(pending.spent_work() <= SOURCE_WORK && pending.cancellation_requested());
    drop(failure);
    let completion = host
        .with_native(|dom, _, _| {
            let mut lease = dom
                .resume_source_edit(
                    pending.operation(),
                    public_node_source_limits(2_000_000, 64),
                    &cancel,
                )
                .unwrap();
            let surgeist_dom::source::DomSourceProgress::Complete(completion) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("the original cancelled verifier must settle its own native tail");
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
