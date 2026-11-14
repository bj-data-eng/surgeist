//! Real Source listener selection, guest reentry, report and mandatory tail.

use super::*;
use surgeist_bindings_js::{Root, ValueKind};

const ROUTE: u64 = 71;
const SOURCE: DomSourceLimits =
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 8);
const EVENT: DomSourceEventServiceLimits = DomSourceEventServiceLimits {
    source: SOURCE,
    dispatch: DomEventDispatchLimits {
        path_items: 16,
        snapshot_listeners: 16,
        failure_records: 16,
        callbacks: 16,
        work: 100_000,
    },
};

#[derive(Default)]
struct NativeTrace {
    later: usize,
}

struct Fixture {
    host: DocumentScriptHost,
    permit: DocumentScriptPermit,
    execution: DomSourceExecutionScope,
    dispatcher: DomEventDispatcher<NativeTrace>,
    context: NativeTrace,
    registration: DomEventRegistrationId,
    later_registration: DomEventRegistrationId,
    request: DomSourceEventRequest,
    handler: Root,
    dialog: NodeId,
    comment: NodeId,
    document: DocumentId,
}

fn fixture(
    handler: &str,
    cancellation: Arc<AtomicBool>,
    initialize: impl FnOnce(&mut Scope<'_>) -> Result<(), Error>,
) -> Fixture {
    fixture_with_storage(handler, cancellation, initialize, STORAGE).unwrap()
}

fn fixture_with_storage(
    handler: &str,
    cancellation: Arc<AtomicBool>,
    initialize: impl FnOnce(&mut Scope<'_>) -> Result<(), Error>,
    storage: usize,
) -> Result<Fixture, DocumentScriptError> {
    fixture_with_setup(handler, cancellation, initialize, storage, |_, _, _| ())
        .map(|(fixture, ())| fixture)
}

fn fixture_with_setup<T>(
    handler: &str,
    cancellation: Arc<AtomicBool>,
    initialize: impl FnOnce(&mut Scope<'_>) -> Result<(), Error>,
    storage: usize,
    setup: impl FnOnce(&mut DocumentScriptHost, &DocumentScriptPermit, DomHostedTopLevel) -> T,
) -> Result<(Fixture, T), DocumentScriptError> {
    let mut policy = limits();
    policy.source_events.jobs = 8;
    let mut session = session(policy);
    let top = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let host = hosted
            .create_headless_host(&mut dom, &cancellation)
            .unwrap();
        hosted
            .create_top_level(&mut dom, host, &cancellation)
            .unwrap()
    };
    let document = top.document();
    let execution = session
        .source_execution_scope(top.host(), &cancellation)
        .unwrap();
    let html = session.snapshot().node(document.node()).unwrap().children()[0];
    let body = session.snapshot().node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(document.node(), Namespace::html(), "dialog");
    let comment = batch.create_comment(document.node(), "delegated Source comment");
    batch.insert_before(&dialog, body, None);
    batch.insert_before(&comment, document.node(), None);
    let commit = session.dom().apply(batch, &cancellation).unwrap();
    let dialog = commit.created(&dialog).unwrap().unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    let opened = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::set_native_attribute(dialog, "open", ""),
            SOURCE,
            &mut DomSourceServices {
                meta: None,
                execution_scope: Some(&execution),
                style_policy: &mut |_| None,
                url: None,
            },
            &cancellation,
        )
        .unwrap();
    assert!(
        matches!(opened, DomSourceDriveProgress::Complete(_)),
        "{opened:?}"
    );

    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 8,
        nested_dispatches: 8,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    let registration = dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BEFORE_TOGGLE,
            DomEventCallback::host(ROUTE),
            DomEventListenerOptions {
                once: true,
                capture: false,
                passive: None,
            },
        )
        .unwrap();
    let later_registration = dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BEFORE_TOGGLE,
            DomEventCallback::new(move |dom, _, trace: &mut NativeTrace, event| {
                assert_eq!(event.target(), Some(dialog.into()));
                let snapshot = dom.snapshot();
                assert_eq!(snapshot.node(comment).unwrap().parent(), None);
                assert!(
                    snapshot
                        .node(dialog)
                        .unwrap()
                        .attribute(&"open".into())
                        .is_some()
                );
                trace.later += 1;
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();

    let mut host = PreparedDocumentScriptHost::prepare(session, storage, cancellation.clone())
        .map_err(|failure| DocumentScriptError::Native(failure.error))?
        .enable(top, ALLOW, hooks(&hook()))
        .map_err(|failure| failure.error)?;
    let permit = host.permit().unwrap();
    let additional = setup(&mut host, &permit, top);
    let handler = host.in_document(&permit, |scope| {
        initialize(scope)?;
        let function = scope.evaluate(&handler.encode_utf16().collect::<Vec<_>>())?;
        Ok(scope.retain(&function))
    })?;
    let request = host
        .with_native(|dom, _, mut source| {
            let progress = source
                .apply(
                    dom,
                    DomSourceEdit::close_dialog(dialog, None),
                    SOURCE,
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: Some(&execution),
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &cancellation,
                )
                .unwrap();
            let DomSourceDriveProgress::Pending {
                request: DomSourceRequest::Event(request),
                reason: DomSourceUnavailable::Event,
            } = progress
            else {
                panic!("the actual Dialog Close must reach beforetoggle: {progress:?}");
            };
            request
        })
        .unwrap();

    Ok((
        Fixture {
            host,
            permit,
            execution,
            dispatcher,
            context: NativeTrace::default(),
            registration,
            later_registration,
            request,
            handler,
            dialog,
            comment,
            document,
        },
        additional,
    ))
}

fn ordinary_throw_keeps_the_real_event_and_tail() {
    let mut fixture = fixture(
        "globalThis.thrown = {}; globalThis.before = 0; globalThis.after = 0;
         globalThis.neutral = 0; globalThis.rejectedForgedComment = 0;
         ((comment) => { globalThis.delegatedComment = comment;
                   if (window !== globalThis || self !== window || parent !== window || opener !== null) {
                       throw Error('wrong native Window relations during Source dispatch');
                   }
                   before = readChildCount();
                   if (comment !== null && typeof comment === 'object') {
                       const forged = Object.create(Object.getPrototypeOf(comment));
                       let coerced = false;
                       try { detachIf({ valueOf() { coerced = true; return 0; } }, forged); }
                       catch (error) { rejectedForgedComment = Number(error instanceof TypeError && !coerced); }
                   }
                   const coercionFailure = {};
                   try { detachIf({ valueOf() { throw coercionFailure; } }, comment); }
                   catch (error) { if (error !== coercionFailure) throw error; }
                   detachIf({ valueOf() { neutral += readChildCount(); return 0; } }, comment);
                   detachIf({ valueOf() { before += readChildCount(); return 1; } }, comment);
                   after = readChildCount(); throw thrown; })",
        Arc::new(cancel()),
        |_| Ok(()),
    );
    assert!(matches!(
        fixture.host.checkpoint(),
        Err(DocumentScriptError::Native(
            DocumentScriptNativeError::SourcePending
        ))
    ));
    let exception = {
        let progress = fixture
            .host
            .select_source_event(
                &fixture.permit,
                &fixture.request,
                &mut fixture.dispatcher,
                &mut fixture.context,
                EVENT,
            )
            .unwrap();
        let DocumentScriptEventProgress::Selected(selected) = progress else {
            panic!("original native dispatcher must select the routed host listener");
        };
        assert_eq!(selected.key(), ROUTE);
        assert_eq!(selected.registration(), fixture.registration);
        assert!(!fixture.dispatcher.remove_listener(fixture.registration));
        let mut report = selected.invoke(&fixture.handler, fixture.comment, SOURCE);
        let DocumentScriptGuestResult::Threw(exception) = report.outcome() else {
            panic!("ordinary guest exception required: {:?}", report.outcome());
        };
        let exception = exception.clone();
        let foreign = DomEventDispatcher::<NativeTrace>::new(DomEventRegistryLimits {
            registrations: 0,
            pending_events: 0,
            nested_dispatches: 0,
            storage_bytes: 16_000_000,
            work: 0,
        })
        .unwrap();
        assert!(matches!(
            report.report(&foreign),
            Err(DomEventDispatchError::WrongContinuation)
        ));
        report.report(&fixture.dispatcher).unwrap();
        assert!(matches!(
            report.report(&fixture.dispatcher),
            Err(DomEventDispatchError::AlreadyCompleted)
        ));
        exception
    };
    assert_eq!(fixture.context.later, 0);
    let after_listener = fixture.host.snapshot().unwrap();
    assert_eq!(
        fixture
            .host
            .with_native(|dom, _, _| dom.pending_source_edit().map(|status| status.operation()))
            .unwrap(),
        Some(fixture.request.operation())
    );
    assert_eq!(after_listener.node(fixture.comment).unwrap().parent(), None);
    assert!(
        after_listener
            .node(fixture.dialog)
            .unwrap()
            .attribute(&"open".into())
            .is_some()
    );

    {
        let progress = fixture
            .host
            .select_source_event(
                &fixture.permit,
                &fixture.request,
                &mut fixture.dispatcher,
                &mut fixture.context,
                EVENT,
            )
            .unwrap();
        let DocumentScriptEventProgress::Native(DomSourceEventServiceProgress::Serviced {
            receipt,
        }) = progress
        else {
            panic!("the real native dispatch must finish after its exact host report");
        };
        let failures = receipt.failures();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].registration, fixture.registration);
        assert_eq!(
            failures[0].kind,
            DomEventListenerFailureKind::Reported(DomEventCallbackError { code: ROUTE })
        );
    }
    assert_eq!(fixture.context.later, 1);
    fixture
        .host
        .with_native(|dom, _, mut source| {
            let mut services = DomSourceServices {
                meta: None,
                execution_scope: Some(&fixture.execution),
                style_policy: &mut |_| None,
                url: None,
            };
            let progress = source
                .resume(
                    dom,
                    fixture.request.operation(),
                    SOURCE,
                    &mut services,
                    &cancel(),
                )
                .unwrap();
            assert!(
                matches!(progress, DomSourceDriveProgress::Complete(ref complete)
            if complete.operation() == fixture.request.operation()),
                "{progress:?}"
            );
            let finished = dom.snapshot();
            assert!(
                finished
                    .node(fixture.dialog)
                    .unwrap()
                    .attribute(&"open".into())
                    .is_none()
            );
            assert_eq!(finished.incomplete_source_operation(), None);
            assert!(
                source
                    .resume(
                        dom,
                        fixture.request.operation(),
                        SOURCE,
                        &mut services,
                        &cancel()
                    )
                    .is_err()
            );
        })
        .unwrap();
    let (comment_kind, rejected_forged_comment) = fixture
        .host
        .in_document(&fixture.permit, |scope| {
            let result = scope.import(exception.value())?;
            let original = scope.evaluate(&"thrown".encode_utf16().collect::<Vec<_>>())?;
            assert!(scope.strict_equals(&result, &original)?);
            assert_script_number(scope, "neutral", 2.0)?;
            assert_script_number(scope, "before", 4.0)?;
            assert_script_number(scope, "after", 1.0)?;
            let comment = scope.evaluate(&"delegatedComment".encode_utf16().collect::<Vec<_>>())?;
            let rejected = scope.evaluate(
                &"rejectedForgedComment === 1"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            )?;
            Ok((scope.kind(&comment)?, scope.boolean(&rejected)?))
        })
        .unwrap();
    assert_eq!(fixture.context.later, 1);
    let session = fixture.host.stop().into_session().unwrap();
    let tasks = session.element_task_execution();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 2);
    let close = tasks
        .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
        .unwrap()
        .unwrap();
    assert_eq!(close.id().target(), fixture.dialog);
    assert!(close.dialog_close().is_some());
    assert_eq!(
        comment_kind,
        ValueKind::Object,
        "selected Source listener did not receive its delegated native Comment"
    );
    assert!(
        rejected_forged_comment,
        "detach must reject a forged Comment argument before coercing the condition"
    );
}

fn conversion_time_native_refusal_retains_original_source(conversion: &str) {
    let cancellation = Arc::new(cancel());
    let revoke = cancellation.clone();
    let jobs = Arc::new(AtomicUsize::new(0));
    let job_counter = jobs.clone();
    let handler = format!(
        "((comment) => {{ try {{ detachIf({{ valueOf() {{ revokeNative(); {conversion}; }} }}, comment); }} catch (_) {{}} sourceJob(); }})"
    );
    let mut fixture = fixture(&handler, cancellation.clone(), move |scope| {
        let job = Callback::new(move |_, _| {
            job_counter.fetch_add(1, Ordering::Relaxed);
            Ok(Completion::Undefined)
        });
        let global = scope.global()?;
        let function = scope.function(&job)?;
        let key = "sourceJob".encode_utf16().collect::<Vec<_>>();
        scope.set(&global, PropertyKey::String(&key), &function)?;
        let script = "Promise.resolve().then(sourceJob)"
            .encode_utf16()
            .collect::<Vec<_>>();
        scope.evaluate(&script)?;
        let callback = Callback::new(move |_, _| {
            revoke.store(true, Ordering::Relaxed);
            Ok(Completion::Undefined)
        });
        let global = scope.global()?;
        let function = scope.function(&callback)?;
        let key = "revokeNative".encode_utf16().collect::<Vec<_>>();
        scope.set(&global, PropertyKey::String(&key), &function)
    });
    let failure = {
        let progress = fixture
            .host
            .select_source_event(
                &fixture.permit,
                &fixture.request,
                &mut fixture.dispatcher,
                &mut fixture.context,
                EVENT,
            )
            .unwrap();
        let DocumentScriptEventProgress::Selected(selected) = progress else {
            panic!("original native dispatcher must select the routed host listener");
        };
        let mut report = selected.invoke(&fixture.handler, fixture.comment, SOURCE);
        let DocumentScriptGuestResult::Refused(Error::HostFailure(failure)) = report.outcome()
        else {
            panic!(
                "{conversion}: typed sticky native refusal required: {:?}, guest calls after revocation: {}",
                report.outcome(),
                jobs.load(Ordering::Relaxed)
            );
        };
        let failure = failure.clone();
        report.report(&fixture.dispatcher).unwrap();
        failure
    };
    cancellation.store(false, Ordering::Relaxed);
    let cause = failure.cause::<DocumentScriptNativeError>().unwrap();
    assert!(matches!(
        cause,
        DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Cancelled)
    ));
    let checkpoint = native_failure(fixture.host.checkpoint());
    assert!(std::ptr::eq(
        cause,
        checkpoint.cause::<DocumentScriptNativeError>().unwrap()
    ));

    fixture
        .host
        .with_native(|dom, _, source| {
            assert!(matches!(
                source.service_event(
                    dom,
                    &fixture.request,
                    &mut fixture.dispatcher,
                    &mut fixture.context,
                    EVENT,
                    &cancellation
                ),
                Err(DomSourceRuntimeError::SourceEvent(
                    DomSourceEventRuntimeError::Dispatch(DomEventDispatchError::HostRefused(
                        DomEventHostError { code: ROUTE }
                    ))
                ))
            ));
            let snapshot = dom.snapshot();
            assert_eq!(
                snapshot.node(fixture.comment).unwrap().parent(),
                Some(fixture.document.node())
            );
            assert!(
                snapshot
                    .node(fixture.dialog)
                    .unwrap()
                    .attribute(&"open".into())
                    .is_some()
            );
            assert_eq!(
                dom.pending_source_edit().map(|status| status.operation()),
                Some(fixture.request.operation())
            );
        })
        .unwrap();
    assert_eq!(fixture.context.later, 0);
    let mut session = fixture.host.stop().into_session().unwrap();
    assert_eq!(
        session
            .dom()
            .pending_source_edit()
            .map(|status| status.operation()),
        Some(fixture.request.operation())
    );
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Pending { .. }
    ));
    assert_eq!(jobs.load(Ordering::Relaxed), 0);
    assert_eq!(Arc::strong_count(&jobs), 1);
}

fn source_driver_refusal_keeps_the_original_cause() {
    let mut fixture = fixture(
        "((comment) => detachIf(1, comment))",
        Arc::new(cancel()),
        |_| Ok(()),
    );
    let no_nested_work = DomSourceLimits::new(32_000_000, 1_000_000, 0, 128, 4096, 4096, 8);
    {
        let progress = fixture
            .host
            .select_source_event(
                &fixture.permit,
                &fixture.request,
                &mut fixture.dispatcher,
                &mut fixture.context,
                EVENT,
            )
            .unwrap();
        let DocumentScriptEventProgress::Selected(selected) = progress else {
            panic!("original native dispatcher must select the routed host listener");
        };
        let mut report = selected.invoke(&fixture.handler, fixture.comment, no_nested_work);
        let DocumentScriptGuestResult::Refused(Error::HostFailure(failure)) = report.outcome()
        else {
            panic!(
                "the nested Source work cutoff must refuse the listener: {:?}",
                report.outcome()
            );
        };
        let cause = failure.cause::<DomSourceDriveError>();
        assert!(
            matches!(
                cause,
                Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                    DomError::SourceWorkLimit { limit: 0 }
                )))
            ),
            "{cause:?}"
        );
        report.report(&fixture.dispatcher).unwrap();
    }
    let snapshot = fixture.host.snapshot().unwrap();
    assert_eq!(
        snapshot.node(fixture.comment).unwrap().parent(),
        Some(fixture.document.node())
    );
    assert_eq!(fixture.context.later, 0);
    let _session = fixture.host.stop().into_session().unwrap();
}

fn finish_projected_close(fixture: &mut Fixture, delegated: NodeId) {
    {
        let progress = fixture
            .host
            .select_source_event(
                &fixture.permit,
                &fixture.request,
                &mut fixture.dispatcher,
                &mut fixture.context,
                EVENT,
            )
            .unwrap();
        let DocumentScriptEventProgress::Selected(selected) = progress else {
            panic!("the original dispatcher must select the routed host listener");
        };
        let mut report = selected.invoke(&fixture.handler, delegated, SOURCE);
        assert!(
            matches!(report.outcome(), DocumentScriptGuestResult::Returned),
            "{:?}",
            report.outcome()
        );
        report.report(&fixture.dispatcher).unwrap();
    }
    fixture
        .host
        .with_native(|dom, _, _| {
            assert_eq!(
                dom.pending_source_edit().unwrap().operation(),
                fixture.request.operation()
            );
            let snapshot = dom.snapshot();
            assert_eq!(snapshot.node(delegated).unwrap().parent(), None);
            assert!(
                snapshot
                    .node(fixture.dialog)
                    .unwrap()
                    .attribute(&"open".into())
                    .is_some()
            );
        })
        .unwrap();
    let progress = fixture
        .host
        .select_source_event(
            &fixture.permit,
            &fixture.request,
            &mut fixture.dispatcher,
            &mut fixture.context,
            EVENT,
        )
        .unwrap();
    assert!(matches!(
        progress,
        DocumentScriptEventProgress::Native(DomSourceEventServiceProgress::Serviced { .. })
    ));
    drop(progress);
    fixture
        .host
        .with_native(|dom, _, mut source| {
            let progress = source
                .resume(
                    dom,
                    fixture.request.operation(),
                    SOURCE,
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: Some(&fixture.execution),
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &cancel(),
                )
                .unwrap();
            assert!(matches!(
                progress,
                DomSourceDriveProgress::Complete(complete)
                    if complete.operation() == fixture.request.operation()
            ));
            let snapshot = dom.snapshot();
            assert_eq!(snapshot.node(delegated).unwrap().parent(), None);
            assert!(
                snapshot
                    .node(fixture.dialog)
                    .unwrap()
                    .attribute(&"open".into())
                    .is_none()
            );
            assert_eq!(snapshot.incomplete_source_operation(), None);
        })
        .unwrap();
}

fn start_another_close(fixture: &mut Fixture, replacement: bool) -> NodeId {
    let original = fixture.comment;
    open_and_close(fixture, |dom, document| {
        if replacement {
            let mut create = DomBatch::new();
            let comment = create.create_comment(document, "delegated Source comment");
            create.insert_before(&comment, document, None);
            dom.apply(create, &cancel())
                .unwrap()
                .created(&comment)
                .unwrap()
                .unwrap()
        } else {
            let mut insert = DomBatch::new();
            insert.insert_before(original, document, None);
            dom.apply(insert, &cancel()).unwrap();
            original
        }
    })
}

fn open_and_close(
    fixture: &mut Fixture,
    delegate: impl FnOnce(&mut DomEventDom<'_>, NodeId) -> NodeId,
) -> NodeId {
    let (request, delegated) = fixture
        .host
        .with_native(|dom, _, mut source| {
            let delegated = delegate(dom, fixture.document.node());
            fixture
                .dispatcher
                .add_listener(
                    dom,
                    fixture.dialog,
                    DomEventType::BEFORE_TOGGLE,
                    DomEventCallback::host(ROUTE),
                    DomEventListenerOptions {
                        once: true,
                        capture: false,
                        passive: None,
                    },
                )
                .unwrap();
            let mut services = DomSourceServices {
                meta: None,
                execution_scope: Some(&fixture.execution),
                style_policy: &mut |_| None,
                url: None,
            };
            assert!(matches!(
                source
                    .apply(
                        dom,
                        DomSourceEdit::set_native_attribute(fixture.dialog, "open", ""),
                        SOURCE,
                        &mut services,
                        &cancel(),
                    )
                    .unwrap(),
                DomSourceDriveProgress::Complete(_)
            ));
            let progress = source
                .apply(
                    dom,
                    DomSourceEdit::close_dialog(fixture.dialog, None),
                    SOURCE,
                    &mut services,
                    &cancel(),
                )
                .unwrap();
            let DomSourceDriveProgress::Pending {
                request: DomSourceRequest::Event(request),
                reason: DomSourceUnavailable::Event,
            } = progress
            else {
                panic!("the next actual Dialog Close must reach beforetoggle: {progress:?}");
            };
            (request, delegated)
        })
        .unwrap();
    fixture.request = request;
    delegated
}

fn retire_projected_comment_during_unwind(fixture: &mut Fixture) {
    let original = fixture.comment;
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = fixture.host.with_native(|dom, _, mut source| {
            let no_work = DomSourceLimits::new(32_000_000, 1_000_000, 0, 128, 4096, 4096, 8);
            assert!(matches!(
                source.apply(
                    dom,
                    DomSourceEdit::destroy(original),
                    no_work,
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &cancel(),
                ),
                Err(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                    DomError::SourceWorkLimit { limit: 0 }
                )))
            ));
            assert!(dom.snapshot().node(original).is_ok());
            let mut destroy = DomBatch::new();
            destroy.destroy(original);
            dom.apply(destroy, &cancel()).unwrap();
            std::panic::panic_any("committed projected Comment destruction");
        });
    }));
    assert_eq!(
        unwound.unwrap_err().downcast_ref::<&str>(),
        Some(&"committed projected Comment destruction")
    );
    assert!(matches!(
        fixture.host.snapshot().unwrap().node(original),
        Err(DomError::StaleNode)
    ));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StorageAdmission {
    BeforeOperation,
    Refused,
    Admitted,
}

fn observe_pin_admission(storage: usize, detach: bool) -> (StorageAdmission, usize) {
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    // Keep authored roots live across the operation so its boundary exceeds
    // the already-released host installation peak.
    let mut padding = Vec::new();
    let mut fixture = match fixture_with_storage(
        if detach {
            "((comment) => { reachedGuest(); detachIf(1, comment); })"
        } else {
            "((comment) => { reachedGuest(); })"
        },
        Arc::new(cancel()),
        |scope| {
            for _ in 0..64 {
                let value = scope.primitive(Primitive::Undefined)?;
                padding.push(scope.retain(&value));
            }
            let callback = Callback::new(move |_, _| {
                count.fetch_add(1, Ordering::Relaxed);
                Ok(Completion::Undefined)
            });
            let global = scope.global()?;
            let function = scope.function(&callback)?;
            let name = "reachedGuest".encode_utf16().collect::<Vec<_>>();
            scope.set(&global, PropertyKey::String(&name), &function)
        },
        storage,
    ) {
        Ok(fixture) => fixture,
        Err(DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)) => {
            return (StorageAdmission::BeforeOperation, 0);
        }
        Err(DocumentScriptError::Script(Error::HostFailure(error))) => {
            assert!(matches!(
                error.cause::<DocumentScriptNativeError>(),
                Some(DocumentScriptNativeError::StorageLimit)
            ));
            return (StorageAdmission::BeforeOperation, 0);
        }
        Err(error) => panic!("unexpected host admission failure: {error:?}"),
    };
    let outcome = {
        let progress = fixture.host.select_source_event(
            &fixture.permit,
            &fixture.request,
            &mut fixture.dispatcher,
            &mut fixture.context,
            EVENT,
        );
        let selected = match progress {
            Ok(DocumentScriptEventProgress::Selected(selected)) => selected,
            Err(DocumentScriptEventError::Access(DocumentScriptError::Native(
                DocumentScriptNativeError::StorageLimit,
            ))) => return (StorageAdmission::BeforeOperation, 0),
            Err(error) => panic!("unexpected listener selection failure: {error:?}"),
            Ok(_) => panic!("the original dispatcher must select the routed host listener"),
        };
        let mut report = selected.invoke(&fixture.handler, fixture.comment, SOURCE);
        let outcome = match report.outcome() {
            DocumentScriptGuestResult::Returned => StorageAdmission::Admitted,
            DocumentScriptGuestResult::Refused(Error::HostFailure(failure))
                if matches!(
                    failure.cause::<DocumentScriptNativeError>(),
                    Some(DocumentScriptNativeError::StorageLimit)
                ) =>
            {
                StorageAdmission::Refused
            }
            other => panic!("unexpected projection outcome: {other:?}"),
        };
        report.report(&fixture.dispatcher).unwrap();
        outcome
    };
    let admitted = outcome == StorageAdmission::Admitted;
    let reached = calls.load(Ordering::Relaxed);
    if admitted {
        assert_eq!(reached, 1);
    } else if !detach {
        assert_eq!(reached, 0);
    } else {
        assert!(reached <= 1);
    }
    assert_eq!(
        fixture
            .host
            .snapshot()
            .unwrap()
            .node(fixture.comment)
            .unwrap()
            .parent(),
        (!(admitted && detach)).then_some(fixture.document.node())
    );
    (outcome, reached)
}

fn exact_projection_and_handler_storage_are_required() {
    for detach in [false, true] {
        let mut short = 0;
        let mut exact = STORAGE;
        assert_eq!(
            observe_pin_admission(exact, detach),
            (StorageAdmission::Admitted, 1)
        );
        while exact - short > 1 {
            let probe = short + (exact - short) / 2;
            if observe_pin_admission(probe, detach).0 == StorageAdmission::Admitted {
                exact = probe;
            } else {
                short = probe;
            }
        }
        // The first native marker remains untouched when its preparation is short.
        // With the second callback, its own preparation can refuse after the first
        // marker; both refused paths leave the delegated Comment attached.
        assert_eq!(
            observe_pin_admission(short, detach),
            (StorageAdmission::Refused, usize::from(detach))
        );
    }
}

fn projected_comment_survives_detach_but_does_not_alias_a_replacement() {
    exact_projection_and_handler_storage_are_required();
    let mut fixture = fixture_with_storage(
        "globalThis.turn = 0; globalThis.same = 0; globalThis.fresh = 0;
         globalThis.rejected = 0; globalThis.coercions = 0; globalThis.traps = 0;
         ((comment) => {
             if (++turn === 1) { globalThis.original = comment; comment.mark = 17; }
             if (turn === 2) same = Number(comment === original && comment.mark === 17);
             if (turn === 3) {
                 fresh = Number(comment !== original && comment.mark === undefined);
                 const proxy = new Proxy(comment, {
                     getPrototypeOf() { ++traps; return null; },
                     get() { ++traps; }
                 });
                 for (const wrong of [original, proxy, undefined]) {
                     try { detachIf({ valueOf() { ++coercions; return 0; } }, wrong); }
                     catch (error) { if (error instanceof TypeError) ++rejected; }
                 }
             }
             detachIf(1, comment);
         })",
        Arc::new(cancel()),
        |_| Ok(()),
        STORAGE,
    )
    .unwrap();
    let original = fixture.comment;
    finish_projected_close(&mut fixture, original);
    assert!(
        fixture
            .dispatcher
            .remove_listener(fixture.later_registration)
    );
    let delegated = start_another_close(&mut fixture, false);
    assert_eq!(delegated, original);
    finish_projected_close(&mut fixture, delegated);
    retire_projected_comment_during_unwind(&mut fixture);
    let replacement = start_another_close(&mut fixture, true);
    assert_ne!(replacement, original);
    finish_projected_close(&mut fixture, replacement);
    fixture
        .host
        .in_document(&fixture.permit, |scope| {
            assert_script_number(scope, "same", 1.0)?;
            assert_script_number(scope, "fresh", 1.0)?;
            assert_script_number(scope, "rejected", 3.0)?;
            assert_script_number(scope, "coercions", 0.0)?;
            assert_script_number(scope, "traps", 0.0)
        })
        .unwrap();
}

fn retiring_a_second_pin_preserves_the_other_wrapper() {
    let mut fixture = fixture(
        "globalThis.turn = 0; globalThis.preserved = 0;
         ((comment) => {
             if (++turn === 1) { globalThis.original = comment; comment.mark = 23; }
             if (turn === 2 && comment === original) throw Error('same native wrapper');
             if (turn === 3) preserved = Number(comment === original && comment.mark === 23);
             detachIf(1, comment);
         })",
        Arc::new(cancel()),
        |_| Ok(()),
    );
    let original = fixture.comment;
    finish_projected_close(&mut fixture, original);
    assert!(
        fixture
            .dispatcher
            .remove_listener(fixture.later_registration)
    );
    let second = start_another_close(&mut fixture, true);
    finish_projected_close(&mut fixture, second);
    fixture
        .host
        .with_native(|dom, _, _| {
            let mut destroy = DomBatch::new();
            destroy.destroy(second);
            dom.apply(destroy, &cancel()).unwrap();
            assert!(dom.snapshot().node(original).is_ok());
        })
        .unwrap();
    let delegated = start_another_close(&mut fixture, false);
    finish_projected_close(&mut fixture, delegated);
    fixture
        .host
        .in_document(&fixture.permit, |scope| {
            assert_script_number(scope, "preserved", 1.0)
        })
        .unwrap();
}

fn native_creation_host(host: &mut DocumentScriptHost) -> (DomHostedTopLevel, [NodeId; 2]) {
    host.with_native(|dom, mut hosted, _| {
        let owner = hosted.create_headless_host(dom, &cancel()).unwrap();
        let top = hosted.create_top_level(dom, owner, &cancel()).unwrap();
        let document = top.document().node();
        let mut batch = DomBatch::new();
        let original = batch.create_comment(document, "project before creation Host retirement");
        let first_after = batch.create_comment(document, "project after creation Host retirement");
        batch.insert_before(&original, document, None);
        batch.insert_before(&first_after, document, None);
        let committed = dom.apply(batch, &cancel()).unwrap();
        let nodes =
            [original, first_after].map(|ticket| committed.created(&ticket).unwrap().unwrap());
        (top, nodes)
    })
    .unwrap()
}

fn insert_comment(dom: &mut DomEventDom<'_>, node: NodeId, document: NodeId) -> NodeId {
    let mut batch = DomBatch::new();
    batch.insert_before(node, document, None);
    dom.apply(batch, &cancel()).unwrap();
    assert_eq!(dom.snapshot().node(node).unwrap().parent(), Some(document));
    node
}

fn retire_creation_host(fixture: &mut Fixture, origin: DomHostedTopLevel) {
    fixture
        .host
        .with_native(|dom, mut hosted, _| {
            let task = hosted
                .destroy_top_level(dom, origin.navigable(), &cancel())
                .unwrap();
            let completed = (0..256).any(|_| {
                matches!(
                    hosted.run_next(dom, task.traversable(), &cancel()).unwrap(),
                    DomHostedTaskProgress::Complete { operation } if operation == task.operation()
                )
            });
            assert!(completed);
            hosted.destroy_host(dom, origin.host(), &cancel()).unwrap();
            let snapshot = dom.snapshot();
            assert!(
                !snapshot
                    .document(origin.document())
                    .unwrap()
                    .is_fully_active()
                    .unwrap()
            );
            assert!(
                snapshot
                    .document(fixture.document)
                    .unwrap()
                    .is_fully_active()
                    .unwrap()
            );
            assert_eq!(
                snapshot.global_document(origin.global()).unwrap(),
                origin.document()
            );
        })
        .unwrap();
}

fn unregistered_creation_is_refused_before_guest() {
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let (mut fixture, delegated) = fixture_with_setup(
        "((comment) => { reachedGuest(); detachIf(1, comment); })",
        Arc::new(cancel()),
        |scope| {
            let callback = Callback::new(move |_, _| {
                count.fetch_add(1, Ordering::Relaxed);
                Ok(Completion::Undefined)
            });
            let global = scope.global()?;
            let function = scope.function(&callback)?;
            let name = "reachedGuest".encode_utf16().collect::<Vec<_>>();
            scope.set(&global, PropertyKey::String(&name), &function)
        },
        STORAGE,
        |host, permit, receiving| {
            let (_foreign_session, foreign) = super::setup();
            assert!(matches!(
                host.register_node_creation_realm(permit, foreign),
                Err(DocumentScriptError::Native(
                    DocumentScriptNativeError::Hosted(DomHostedRuntimeError::TargetNotFound)
                ))
            ));
            let (origin, [delegated, _]) = native_creation_host(host);
            host.with_native(|dom, _, _| {
                insert_comment(dom, delegated, receiving.document().node());
                let snapshot = dom.snapshot();
                assert_eq!(
                    snapshot.relevant_global(delegated).unwrap(),
                    Some(origin.global())
                );
            })
            .unwrap();
            delegated
        },
    )
    .unwrap();

    {
        let DocumentScriptEventProgress::Selected(selected) = fixture
            .host
            .select_source_event(
                &fixture.permit,
                &fixture.request,
                &mut fixture.dispatcher,
                &mut fixture.context,
                EVENT,
            )
            .unwrap()
        else {
            panic!("the original native dispatcher must select the routed host listener");
        };
        let mut report = selected.invoke(&fixture.handler, delegated, SOURCE);
        assert!(
            matches!(
                report.outcome(),
                DocumentScriptGuestResult::Refused(Error::HostFailure(failure))
                    if matches!(
                        failure.cause::<DocumentScriptNativeError>(),
                        Some(DocumentScriptNativeError::CreationGlobalUnavailable)
                    )
            ),
            "{:?}",
            report.outcome()
        );
        report.report(&fixture.dispatcher).unwrap();
    }
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    fixture
        .host
        .with_native(|dom, _, _| {
            assert_eq!(
                dom.pending_source_edit().unwrap().operation(),
                fixture.request.operation()
            );
            assert_eq!(
                dom.snapshot().node(delegated).unwrap().parent(),
                Some(fixture.document.node())
            );
            assert!(
                dom.snapshot()
                    .node(fixture.dialog)
                    .unwrap()
                    .attribute(&"open".into())
                    .is_some()
            );
        })
        .unwrap();
}

fn adopted_comments_keep_creation_realm_through_retirement() {
    let (mut fixture, (origin, [original, first_after])) = fixture_with_setup(
        "globalThis.turn = 0; globalThis.preserved = 0; globalThis.afterRetirement = 0;
         globalThis.otherPreserved = 0;
         ((comment) => {
             const prototype = Object.getPrototypeOf(comment);
             switch (++turn) {
                 case 1:
                     if (prototype !== Object.prototype) throw Error('receiving realm control');
                     break;
                 case 2:
                     if (prototype === Object.prototype || prototype === null)
                         throw Error('creation realm must differ from receiving realm');
                     globalThis.saved = comment;
                     globalThis.originPrototype = prototype;
                     saved.mark = 51;
                     break;
                 case 3:
                     preserved = Number(comment === saved && comment.mark === 51
                         && prototype === originPrototype);
                     break;
                 case 4:
                     afterRetirement = Number(comment !== saved && comment.mark === undefined
                         && prototype === originPrototype);
                     globalThis.other = comment;
                     other.mark = 63;
                     break;
                 case 5:
                     otherPreserved = Number(comment === other && comment.mark === 63
                         && prototype === originPrototype);
                     break;
             }
             detachIf({ valueOf() { if (readChildCount() < 2) throw Error('wrong receiving Document');
                                    return 1; } }, comment);
         })",
        Arc::new(cancel()),
        |_| Ok(()),
        STORAGE,
        |host, permit, receiving| {
            let (origin, nodes) = native_creation_host(host);
            assert_ne!(origin.host(), receiving.host());
            host.register_node_creation_realm(permit, origin).unwrap();
            assert!(matches!(
                host.register_node_creation_realm(permit, origin),
                Err(DocumentScriptError::Native(
                    DocumentScriptNativeError::CreationRealmAlreadyRegistered
                ))
            ));
            let snapshot = host.snapshot().unwrap();
            assert_eq!(
                snapshot.document(origin.document()).unwrap().scripting(),
                HtmlScriptingStatus::Disabled
            );
            assert_eq!(
                host.observe_scripting(permit).unwrap().status(),
                HtmlScriptingStatus::Enabled
            );
            (origin, nodes)
        },
    )
    .unwrap();

    let receiving_control = fixture.comment;
    finish_projected_close(&mut fixture, receiving_control);
    assert!(
        fixture
            .dispatcher
            .remove_listener(fixture.later_registration)
    );

    let delegated = open_and_close(&mut fixture, |dom, document| {
        insert_comment(dom, original, document);
        let snapshot = dom.snapshot();
        assert_eq!(
            snapshot
                .node(original)
                .unwrap()
                .owner_document()
                .unwrap()
                .node(),
            document
        );
        assert_eq!(
            snapshot.relevant_global(original).unwrap(),
            Some(origin.global())
        );
        original
    });
    finish_projected_close(&mut fixture, delegated);
    fixture
        .host
        .with_native(|dom, _, _| {
            let document = fixture.document.node();
            insert_comment(dom, original, origin.document().node());
            insert_comment(dom, original, document);
            insert_comment(dom, first_after, document);
        })
        .unwrap();
    retire_creation_host(&mut fixture, origin);
    assert!(matches!(
        fixture
            .host
            .register_node_creation_realm(&fixture.permit, origin),
        Err(DocumentScriptError::Native(
            DocumentScriptNativeError::Hosted(DomHostedRuntimeError::TargetNotFound)
        ))
    ));
    for node in [original, first_after] {
        let snapshot = fixture.host.snapshot().unwrap();
        assert_eq!(
            snapshot.node(node).unwrap().owner_document(),
            Some(fixture.document)
        );
        assert_eq!(
            snapshot.relevant_global(node).unwrap(),
            Some(origin.global())
        );
        let delegated = open_and_close(&mut fixture, |dom, document| {
            assert_eq!(dom.snapshot().node(node).unwrap().parent(), Some(document));
            node
        });
        finish_projected_close(&mut fixture, delegated);
    }

    fixture
        .host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            batch.destroy(original);
            dom.apply(batch, &cancel()).unwrap();
            assert!(matches!(
                dom.snapshot().node(original),
                Err(DomError::StaleNode)
            ));
            assert!(dom.snapshot().node(first_after).is_ok());
        })
        .unwrap();
    let delegated = open_and_close(&mut fixture, |dom, document| {
        insert_comment(dom, first_after, document)
    });
    finish_projected_close(&mut fixture, delegated);
    fixture
        .host
        .in_document(&fixture.permit, |scope| {
            assert_script_number(scope, "preserved", 1.0)?;
            assert_script_number(scope, "afterRetirement", 1.0)?;
            assert_script_number(scope, "otherPreserved", 1.0)
        })
        .unwrap();
}

fn observe_creation_admission(storage: usize) -> StorageAdmission {
    let (session, receiving) = super::setup();
    let prepared = match PreparedDocumentScriptHost::prepare(session, storage, Arc::new(cancel())) {
        Ok(prepared) => prepared,
        Err(failure) if matches!(failure.error, DocumentScriptNativeError::StorageLimit) => {
            return StorageAdmission::BeforeOperation;
        }
        Err(failure) => panic!("unexpected native owner admission: {failure:?}"),
    };
    let mut host = match prepared.enable(receiving, ALLOW, hooks(&hook())) {
        Ok(host) => host,
        Err(failure)
            if matches!(
                failure.error,
                DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
            ) =>
        {
            return StorageAdmission::BeforeOperation;
        }
        Err(failure)
            if matches!(
                &failure.error,
                DocumentScriptError::Script(Error::HostFailure(error))
                    if matches!(error.cause::<DocumentScriptNativeError>(), Some(DocumentScriptNativeError::StorageLimit))
            ) =>
        {
            return StorageAdmission::BeforeOperation;
        }
        Err(failure) => panic!("unexpected script owner admission: {failure:?}"),
    };
    let permit = host.permit().unwrap();
    // Keep the realm operation's accepted live prefix above the freed setup
    // peak, without assuming the private physical extent of either owner.
    let padding = match host.in_document(&permit, |scope| {
        let mut roots = Vec::new();
        for _ in 0..64 {
            let value = scope.primitive(Primitive::Undefined)?;
            roots.push(scope.retain(&value));
        }
        Ok(roots)
    }) {
        Ok(roots) => roots,
        Err(DocumentScriptError::Script(Error::HostFailure(error))) => {
            assert!(matches!(
                error.cause::<DocumentScriptNativeError>(),
                Some(DocumentScriptNativeError::StorageLimit)
            ));
            return StorageAdmission::BeforeOperation;
        }
        Err(error) => panic!("unexpected creation realm padding admission: {error:?}"),
    };
    let (origin, _) = native_creation_host(&mut host);
    let stamp = host.snapshot().unwrap().stamp();
    let observation = host.observe_scripting(&permit).unwrap();
    assert_eq!(observation.status(), HtmlScriptingStatus::Enabled);
    let mut terminal = None;
    let outcome = match host.register_node_creation_realm(&permit, origin) {
        Ok(()) => StorageAdmission::Admitted,
        Err(DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)) => {
            assert!(matches!(
                host.register_node_creation_realm(&permit, origin),
                Err(DocumentScriptError::Native(
                    DocumentScriptNativeError::StorageLimit
                ))
            ));
            StorageAdmission::Refused
        }
        Err(DocumentScriptError::Script(Error::HostFailure(error))) => {
            let cause = error.cause::<DocumentScriptNativeError>().unwrap();
            assert!(matches!(cause, DocumentScriptNativeError::StorageLimit));
            let Err(DocumentScriptError::Script(Error::HostFailure(sticky))) =
                host.register_node_creation_realm(&permit, origin)
            else {
                panic!("a refused realm producer must retain its first failure");
            };
            assert!(std::ptr::eq(
                cause,
                sticky.cause::<DocumentScriptNativeError>().unwrap()
            ));
            terminal = Some(error);
            StorageAdmission::Refused
        }
        Err(error) => panic!("unexpected creation realm admission: {error:?}"),
    };
    if let Some(original) = terminal {
        let Err(DocumentScriptError::Script(Error::HostFailure(sticky))) =
            host.observe_scripting(&permit)
        else {
            panic!("scripting observation checks the original Realm's active owner");
        };
        assert!(std::ptr::eq(
            original.cause::<DocumentScriptNativeError>().unwrap(),
            sticky.cause::<DocumentScriptNativeError>().unwrap()
        ));
    } else {
        assert_eq!(
            host.observe_scripting(&permit).unwrap().status(),
            HtmlScriptingStatus::Enabled
        );
    }
    let snapshot = host.snapshot().unwrap();
    if outcome == StorageAdmission::Refused {
        assert_eq!(snapshot.stamp(), stamp);
        snapshot.with_document_scripting(observation).unwrap();
    }
    assert_eq!(
        snapshot.document(origin.document()).unwrap().scripting(),
        HtmlScriptingStatus::Disabled
    );
    if outcome == StorageAdmission::Admitted {
        // Duplicate qualification reserves its own temporary record first.
        // Release the exact-envelope filler before this separate semantic probe.
        drop(padding);
        host.in_document(&permit, |_| Ok(())).unwrap();
        assert!(matches!(
            host.register_node_creation_realm(&permit, origin),
            Err(DocumentScriptError::Native(
                DocumentScriptNativeError::CreationRealmAlreadyRegistered
            ))
        ));
    }
    outcome
}

fn exact_creation_storage_is_required() {
    let mut short = 0;
    let mut exact = STORAGE;
    assert_eq!(
        observe_creation_admission(exact),
        StorageAdmission::Admitted
    );
    while exact - short > 1 {
        let probe = short + (exact - short) / 2;
        if observe_creation_admission(probe) == StorageAdmission::Admitted {
            exact = probe;
        } else {
            short = probe;
        }
    }
    assert_eq!(
        observe_creation_admission(exact),
        StorageAdmission::Admitted
    );
    assert_eq!(observe_creation_admission(short), StorageAdmission::Refused);
}

pub(super) fn run_creation_realms() {
    unregistered_creation_is_refused_before_guest();
    adopted_comments_keep_creation_realm_through_retirement();
    exact_creation_storage_is_required();
}

pub(super) fn run() {
    ordinary_throw_keeps_the_real_event_and_tail();
    for conversion in [
        "return 1",
        "return 0",
        "return NaN",
        "throw { sentinel: true }",
    ] {
        conversion_time_native_refusal_retains_original_source(conversion);
    }
    source_driver_refusal_keeps_the_original_cause();
    projected_comment_survives_detach_but_does_not_alias_a_replacement();
    retiring_a_second_pin_preserves_the_other_wrapper();
}
