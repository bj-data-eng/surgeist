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
            DomSourceEdit::set_attribute(dialog, "open", ""),
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
    })
    .unwrap();
    let registration = dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::host(ROUTE),
            DomEventListenerOptions {
                once: true,
                capture: false,
            },
        )
        .unwrap();
    dispatcher
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, trace: &mut NativeTrace, event| {
                assert_eq!(event.target(), Some(dialog));
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

    let mut host = prepare(session, cancellation.clone())
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let handler = host
        .in_document(&permit, |scope| {
            initialize(scope)?;
            let function = scope.evaluate(&handler.encode_utf16().collect::<Vec<_>>())?;
            Ok(scope.retain(&function))
        })
        .unwrap();
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

    Fixture {
        host,
        permit,
        execution,
        dispatcher,
        context: NativeTrace::default(),
        registration,
        request,
        handler,
        dialog,
        comment,
        document,
    }
}

fn ordinary_throw_keeps_the_real_event_and_tail() {
    let mut fixture = fixture(
        "globalThis.thrown = {}; globalThis.before = 0; globalThis.after = 0;
         globalThis.neutral = 0; globalThis.rejectedForgedComment = 0;
         ((comment) => { globalThis.delegatedComment = comment;
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
        "(() => {{ try {{ detachIf({{ valueOf() {{ revokeNative(); {conversion}; }} }}); }} catch (_) {{}} sourceJob(); }})"
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
    let mut fixture = fixture("(() => detachIf(1))", Arc::new(cancel()), |_| Ok(()));
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
}
