//! Real entry-thread V8 ownership over an original native Runtime Session.

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use surgeist_bindings_js::{
    Callback, Completion, EngineKind, Error, Primitive, Process, PropertyKey, Scope, ValueKind,
};
use surgeist_css::{
    query::media::CssMediaScripting,
    serialization::{CssSerializationError, CssomIdentifierLimits},
    source::CssUtf16InputLimits,
};
use surgeist_dom::*;
use surgeist_html::scripting::HtmlScriptingPolicy;
use surgeist_html_semantics::{HtmlCaptureLimits, HtmlCapturedNodeKind, HtmlScriptingStatus};
use surgeist_runtime::{document_scripting::*, *};

#[path = "document_scripting/source_events.rs"]
mod source_events;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}

fn limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: surgeist_runtime::DomSourceEventRuntimeLimits {
            jobs: 0,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 32,
            reports: 32,
            storage_bytes: 16_000_000,
            work: 1_000_000,
            css_input: CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 32,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            tasks: 32,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        },
    }
}
fn session(policy: DomSessionLimits) -> DomSession {
    let dom = Dom::new(
        DomLimits::new(
            256,
            200_000,
            1024,
            1024,
            1024,
            32,
            4_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        ),
    )
    .unwrap();
    DomSession::new(dom, policy).unwrap()
}
fn create(session: &mut DomSession) -> (HostedRuntime, DomHostedTopLevel) {
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel()).unwrap();
    // Initial no-opener creation reaches no URL parsing service.
    let top = hosted.create_top_level(&mut dom, host, &cancel()).unwrap();
    (hosted, top)
}

const ALLOW: HtmlScriptingPolicy = HtmlScriptingPolicy {
    user_allows: true,
    automation_allows: true,
};
const STORAGE: usize = 1_000_000;

fn hooks(child_count: &[u16]) -> DocumentScriptHooks<'_> {
    const DETACH: [u16; 8] = [100, 101, 116, 97, 99, 104, 73, 102];
    DocumentScriptHooks {
        child_count,
        detach_source_node_if: &DETACH,
    }
}

fn hook() -> Vec<u16> {
    "readChildCount".encode_utf16().collect()
}

fn setup() -> (DomSession, DomHostedTopLevel) {
    let mut session = session(limits());
    let (_, top) = create(&mut session);
    (session, top)
}

fn prepare(session: DomSession, cancellation: Arc<AtomicBool>) -> PreparedDocumentScriptHost {
    PreparedDocumentScriptHost::prepare(session, STORAGE, cancellation).unwrap()
}

fn captured_status(capture: &DomHtmlCapture, document: DocumentId) -> HtmlScriptingStatus {
    let subject = capture.subject(document.node()).unwrap();
    match capture.capture().node(&subject).unwrap().kind() {
        HtmlCapturedNodeKind::Document(document) => document.scripting(),
        other => panic!("unexpected captured subject: {other:?}"),
    }
}

fn capture_limits() -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(32, 96, 32, 4096, 16, 8, 8, 4096)
}

fn assert_script_number(scope: &mut Scope<'_>, script: &str, expected: f64) -> Result<(), Error> {
    let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
    let number = scope.primitive(Primitive::Number(expected))?;
    assert!(scope.strict_equals(&value, &number)?);
    Ok(())
}

fn eval(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit, script: &str, expected: f64) {
    host.in_document(permit, |scope| {
        assert_script_number(scope, script, expected)
    })
    .unwrap();
}

fn native_window_exposes_css_escape() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());

    let kind = host
        .in_document(&permit, |scope| {
            let script: Vec<_> = "globalThis.CSS".encode_utf16().collect();
            let value = scope.evaluate(&script)?;
            scope.kind(&value)
        })
        .unwrap();
    assert_eq!(
        kind,
        ValueKind::Object,
        "native Window CSS namespace is missing"
    );

    for check in [
        "Number(Object.getPrototypeOf(CSS) === Object.prototype)",
        "Number(Object.prototype.toString.call(CSS) === '[object CSS]')",
        "Number(CSS.escape.name === 'escape' && CSS.escape.length === 1)",
        r#"Number((() => {
            function data(object, key, value, writable, enumerable, configurable) {
                const descriptor = Object.getOwnPropertyDescriptor(object, key);
                return descriptor.value === value && descriptor.writable === writable
                    && descriptor.enumerable === enumerable
                    && descriptor.configurable === configurable;
            }
            const namespace = CSS;
            const escape = CSS.escape;
            return data(globalThis, "CSS", namespace, true, false, true)
                && data(namespace, "escape", escape, true, true, true)
                && data(namespace, Symbol.toStringTag, "CSS", false, false, true)
                && data(escape, "name", "escape", false, false, true)
                && data(escape, "length", 1, false, false, true)
                && Object.getPrototypeOf(escape) === Function.prototype
                && !Object.hasOwn(escape, "prototype")
                && namespace === globalThis.CSS && escape === CSS.escape;
        })())"#,
        r#"Number((() => {
            const escape = CSS.escape;
            const sentinel = {};
            const surplus = { toString() { throw sentinel; } };
            let coercions = 0;
            const result = escape.call({}, {
                [Symbol.toPrimitive](hint) {
                    if (hint !== "string") throw sentinel;
                    coercions++;
                    return "0a\u00a0\u00d7\ud800_\ud83d\ude42\udc00\0";
                }
            }, surplus);
            let originalThrow = false;
            try { escape({ toString() { throw sentinel; } }); }
            catch (error) { originalThrow = error === sentinel; }
            let symbolTypeError = false;
            try { escape(Symbol()); }
            catch (error) { symbolTypeError = error instanceof TypeError; }
            return result === "\\30 a\u00a0\u00d7\ud800_\ud83d\ude42\udc00\ufffd"
                && coercions === 1 && originalThrow && symbolTypeError
                && escape(undefined) === "undefined" && escape(null) === "null";
        })())"#,
        r"Number(CSS.escape('a\u00a0\ud800\0') === 'a\u00a0\ud800\ufffd')",
        "Number((() => { try { CSS.escape(); } catch (error) { return error instanceof TypeError; } return false; })())",
        "Number((() => { try { Reflect.construct(function() {}, [], CSS.escape); } catch (error) { return error instanceof TypeError; } return false; })())",
    ] {
        eval(&mut host, &permit, check, 1.0);
    }

    let (original_realm, original_css) = host
        .in_document(&permit, |scope| {
            let css = scope.evaluate(&"CSS".encode_utf16().collect::<Vec<_>>())?;
            Ok((scope.realm_authority()?.identity(), scope.retain(&css)))
        })
        .unwrap();
    let session = host.stop().into_session().unwrap();
    let mut replacement = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = replacement.permit().unwrap();
    replacement
        .in_document(&permit, |scope| {
            assert_ne!(scope.realm_authority()?.identity(), original_realm);
            assert!(matches!(
                scope.import(&original_css),
                Err(Error::ForeignOwner)
            ));
            Ok(())
        })
        .unwrap();
    eval(
        &mut replacement,
        &permit,
        "Number(CSS.escape('b') === 'b')",
        1.0,
    );
}

fn native_window_css_admission() {
    fn configured(limits: CssNamespaceLimits) -> DocumentScriptHost {
        let (session, top) = setup();
        prepare(session, Arc::new(cancel()))
            .with_css_namespace_limits(limits)
            .enable(top, ALLOW, hooks(&hook()))
            .unwrap()
    }

    const ESCAPE: &str = r#"CSS.escape("1\ud800_\ud83d\ude42\udc00")"#;
    const EXACT: CssomIdentifierLimits = CssomIdentifierLimits::new(6, 18);
    let mut host = configured(CssNamespaceLimits::new(2, 12, 12, EXACT));
    let permit = host.permit().unwrap();
    for _ in 0..2 {
        eval(
            &mut host,
            &permit,
            r#"Number(CSS.escape("1\ud800_\ud83d\ude42\udc00") === "\\31 \ud800_\ud83d\ude42\udc00")"#,
            1.0,
        );
    }
    assert!(matches!(
        host.in_document(&permit, |scope| scope
            .evaluate(&ESCAPE.encode_utf16().collect::<Vec<_>>())
            .map(drop)),
        Err(DocumentScriptError::Script(Error::ConversionLedger(_)))
    ));
    host.stop().into_session().unwrap();

    for limits in [
        CssNamespaceLimits::new(1, 5, 12, EXACT),
        CssNamespaceLimits::new(1, 6, 11, EXACT),
    ] {
        let mut host = configured(limits);
        let permit = host.permit().unwrap();
        assert!(matches!(
            host.in_document(&permit, |scope| scope
                .evaluate(&ESCAPE.encode_utf16().collect::<Vec<_>>())
                .map(drop)),
            Err(DocumentScriptError::Script(Error::ConversionLedger(_)))
        ));
        host.stop().into_session().unwrap();
    }

    for (identifier, input_refused) in [
        (CssomIdentifierLimits::new(5, 17), true),
        (CssomIdentifierLimits::new(6, 17), false),
    ] {
        let mut host = configured(CssNamespaceLimits::new(1, 6, 12, identifier));
        let permit = host.permit().unwrap();
        let script = format!("try {{ {ESCAPE}; }} catch (_) {{}} 42");
        let failure = native_failure(host.in_document(&permit, |scope| {
            scope
                .evaluate(&script.encode_utf16().collect::<Vec<_>>())
                .map(drop)
        }));
        let cause = failure.cause::<CssSerializationError>().unwrap();
        assert!(cause.is_resource_limit());
        assert_eq!(cause.is_input_unit_limit(), input_refused);
        assert_eq!(cause.is_byte_limit(), !input_refused);
        let first = native_failure(host.checkpoint());
        assert!(std::ptr::eq(
            cause,
            first.cause::<CssSerializationError>().unwrap()
        ));
        host.stop().into_session().unwrap();
        assert_eq!(cause.is_input_unit_limit(), input_refused);
    }
}

fn native_window_css_hook_collision_preserves_the_original_session() {
    const CSS: &[u16] = &[67, 83, 83];
    let global_this: Vec<_> = "globalThis".encode_utf16().collect();
    let (session, top) = setup();
    let before = session.snapshot();
    let mut prepared = prepare(session, Arc::new(cancel()));
    for names in [
        DocumentScriptHooks {
            child_count: CSS,
            detach_source_node_if: &[65],
        },
        DocumentScriptHooks {
            child_count: &[65],
            detach_source_node_if: CSS,
        },
        DocumentScriptHooks {
            child_count: &global_this,
            detach_source_node_if: &[65],
        },
        DocumentScriptHooks {
            child_count: &[65],
            detach_source_node_if: &global_this,
        },
    ] {
        let failure = match prepared.enable(top, ALLOW, names) {
            Err(failure) => failure,
            Ok(mut host) => {
                let permit = host.permit().unwrap();
                let observed = host
                    .in_document(&permit, |scope| {
                        let source: Vec<_> = "typeof globalThis.CSS + '/' + typeof CSS.escape"
                            .encode_utf16()
                            .collect();
                        let value = scope.evaluate(&source)?;
                        scope.to_utf16(&value)
                    })
                    .unwrap();
                panic!(
                    "conflicting hook published the attachment, guest globalThis.CSS/CSS.escape: {}",
                    String::from_utf16(&observed).unwrap()
                );
            }
        };
        assert!(matches!(
            failure.error,
            DocumentScriptError::Script(Error::PropertyRejected)
        ));
        prepared = failure.owner;
    }
    let session = prepared.into_session().unwrap();
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .scripting(),
        HtmlScriptingStatus::Disabled,
    );
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    eval(&mut host, &permit, "Number(CSS.escape('a') === 'a')", 1.0);
    eval(
        &mut host,
        &permit,
        "Number(Object.getOwnPropertyDescriptor(globalThis, 'globalThis').configurable)",
        1.0,
    );
}

fn failed_enable_preserves_original_owner() {
    let (original, target) = setup();
    let (foreign, other_target) = setup();
    let stamp = original.snapshot().stamp();
    let cancellation = Arc::new(cancel());
    let refused =
        PreparedDocumentScriptHost::prepare(original, 0, cancellation.clone()).unwrap_err();
    assert!(matches!(
        refused.error,
        DocumentScriptNativeError::StorageLimit
    ));
    assert_eq!(refused.session.snapshot().stamp(), stamp);
    let mut prepared = prepare(refused.session, cancellation);
    let refused = prepared
        .enable(other_target, ALLOW, hooks(&hook()))
        .unwrap_err();
    assert!(matches!(
        refused.error,
        DocumentScriptError::Native(DocumentScriptNativeError::Hosted(
            DomHostedRuntimeError::TargetNotFound
        ))
    ));
    prepared = refused.owner;
    drop(foreign);

    let occupied = Process::bootstrap(EngineKind::V8).unwrap();
    let refused = prepared.enable(target, ALLOW, hooks(&hook())).unwrap_err();
    assert!(matches!(
        refused.error,
        DocumentScriptError::Script(Error::OwnerAlreadyActive)
    ));
    drop(occupied);
    prepared = refused.owner;
    for policy in [
        HtmlScriptingPolicy {
            user_allows: false,
            ..ALLOW
        },
        HtmlScriptingPolicy {
            automation_allows: false,
            ..ALLOW
        },
    ] {
        let refused = prepared.enable(target, policy, hooks(&hook())).unwrap_err();
        assert!(matches!(
            refused.error,
            DocumentScriptError::Native(DocumentScriptNativeError::Hosted(
                DomHostedRuntimeError::Dom(DomHostedError::ScriptingDisabled)
            ))
        ));
        prepared = refused.owner;
    }
    let fixed_global = "Infinity".encode_utf16().collect::<Vec<_>>();
    let refused = prepared
        .enable(target, ALLOW, hooks(&fixed_global))
        .unwrap_err();
    assert!(matches!(
        refused.error,
        DocumentScriptError::Script(Error::PropertyRejected)
    ));
    let mut original = refused.owner.into_session().unwrap();
    let no_script = original.snapshot();
    assert_eq!(no_script.stamp(), stamp);
    let status = no_script.document(target.document()).unwrap().scripting();
    assert_eq!(status, HtmlScriptingStatus::Disabled);
    assert_eq!(
        media_scripting_for_status(status),
        Some(CssMediaScripting::None)
    );
    let mut native = DomBatch::new();
    let independent = native.create_document();
    let new = original
        .dom()
        .apply(native, &cancel())
        .unwrap()
        .document(&independent)
        .unwrap();
    let snapshot = original.snapshot();
    let environment = snapshot
        .document(new)
        .unwrap()
        .metadata()
        .document_environment();
    assert_eq!(
        environment.browsing_context(),
        surgeist_html::security::nonce::HtmlBrowsingContextPresence::Absent
    );
    assert_eq!(
        snapshot.document(new).unwrap().scripting(),
        HtmlScriptingStatus::Disabled
    );
}

fn native_read_snapshot_and_explicit_queue() -> DocumentScriptPermit {
    let (session, top) = setup();
    let original = session.snapshot();
    let native_only = original
        .html_capture(
            top.document(),
            None,
            capture_limits(),
            DomHtmlCaptureLimits::new(32, 10_000),
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        captured_status(&native_only, top.document()),
        HtmlScriptingStatus::Disabled
    );
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());

    let attached = host.snapshot().unwrap();
    let observation = host.observe_scripting(&permit).unwrap();
    assert_eq!(observation.status(), HtmlScriptingStatus::Enabled);
    assert_eq!(
        media_scripting_for_status(observation.status()),
        Some(CssMediaScripting::Enabled)
    );
    assert_eq!(
        media_scripting_for_status(HtmlScriptingStatus::Unavailable),
        None
    );
    assert!(matches!(
        original.with_document_scripting(observation),
        Err(DomHtmlCaptureError::StaleScriptingObservation)
    ));
    let adapter_limits = DomHtmlCaptureLimits::new(32, 10_000);
    let unqualified = attached
        .html_capture(
            top.document(),
            None,
            capture_limits(),
            adapter_limits,
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        captured_status(&unqualified, top.document()),
        HtmlScriptingStatus::Unavailable
    );
    let capture = attached
        .with_document_scripting(observation)
        .unwrap()
        .html_capture(None, capture_limits(), adapter_limits, &cancel())
        .unwrap();
    assert_eq!(
        captured_status(&capture, top.document()),
        HtmlScriptingStatus::Enabled
    );

    let thrown = host.in_document(&permit, |scope| {
        let source = "globalThis.thrown = {}; throw thrown"
            .encode_utf16()
            .collect::<Vec<_>>();
        scope.evaluate(&source)?;
        Ok(())
    });
    let Err(DocumentScriptError::Script(Error::Exception(exception))) = thrown else {
        panic!("expected the original guest exception: {thrown:?}");
    };
    host.in_document(&permit, |scope| {
        let thrown = scope.import(exception.value())?;
        let original = scope.evaluate(&"thrown".encode_utf16().collect::<Vec<_>>())?;
        assert!(scope.strict_equals(&thrown, &original)?);
        Ok(())
    })
    .unwrap();

    eval(&mut host, &permit, "readChildCount()", 1.0);
    eval(
        &mut host,
        &permit,
        "globalThis.job = 0; Promise.resolve().then(() => { job = readChildCount(); }); job",
        0.0,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        let comment = batch.create_comment(top.document().node(), "new native child");
        batch.insert_before(&comment, top.document().node(), None);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let after = host.snapshot().unwrap();
    assert!(matches!(
        after.with_document_scripting(observation),
        Err(DomHtmlCaptureError::StaleScriptingObservation)
    ));
    eval(&mut host, &permit, "readChildCount()", 2.0);
    eval(&mut host, &permit, "job", 0.0);
    host.checkpoint().unwrap();
    eval(&mut host, &permit, "job", 2.0);

    let queued_calls = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, queued_calls.clone());
    let restored = host.stop().into_session().unwrap();
    assert_eq!(restored.snapshot().stamp(), after.stamp());
    let fresh = restored.snapshot();
    assert!(matches!(
        fresh.with_document_scripting(observation),
        Err(DomHtmlCaptureError::StaleScriptingObservation)
    ));
    let capture = fresh
        .html_capture(
            top.document(),
            None,
            capture_limits(),
            adapter_limits,
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        captured_status(&capture, top.document()),
        HtmlScriptingStatus::Disabled
    );
    assert_eq!(queued_calls.load(Ordering::Relaxed), 0);
    assert_eq!(Arc::strong_count(&queued_calls), 1);
    let recapture = attached
        .with_document_scripting(observation)
        .unwrap()
        .html_capture(None, capture_limits(), adapter_limits, &cancel())
        .unwrap();
    assert_eq!(
        captured_status(&recapture, top.document()),
        HtmlScriptingStatus::Enabled
    );
    permit
}

fn queue_observable_job(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    calls: Arc<AtomicUsize>,
) {
    host.in_document(permit, |scope| {
        let callback = Callback::new(move |_, _| {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok(Completion::Undefined)
        });
        let global = scope.global()?;
        let function = scope.function(&callback)?;
        let key = "nativeJob".encode_utf16().collect::<Vec<_>>();
        scope.set(&global, PropertyKey::String(&key), &function)?;
        let script = "Promise.resolve().then(nativeJob);"
            .encode_utf16()
            .collect::<Vec<_>>();
        scope.evaluate(&script)?;
        Ok(())
    })
    .unwrap();
}

fn destroy_native_top(
    dom: &mut DomEventDom<'_>,
    mut hosted: HostedRuntime,
    top: DomHostedTopLevel,
) {
    let cancellation = cancel();
    let queued = hosted
        .destroy_top_level(dom, top.navigable(), &cancellation)
        .unwrap();
    for _ in 0..256 {
        if matches!(
            hosted.run_next(dom, queued.traversable(), &cancellation).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == queued.operation()
        ) {
            return;
        }
    }
    panic!("the original native retirement never completed");
}

fn retire_attached(host: &mut DocumentScriptHost, top: DomHostedTopLevel) {
    host.with_native(|dom, hosted, _| destroy_native_top(dom, hosted, top))
        .unwrap();
}

fn retirement_unwind_disposes_the_revoked_guest_owner() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, calls.clone());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        host.with_native::<()>(|dom, hosted, _| {
            destroy_native_top(dom, hosted, top);
            panic!("native retirement unwind");
        })
        .unwrap();
    }));
    let original = result.expect_err("the actual host unwind must propagate");
    assert_eq!(
        original.downcast_ref::<&'static str>().copied(),
        Some("native retirement unwind")
    );
    assert_immediate_retirement(&host, &calls);
    let _session = host.stop().into_session().unwrap();
}

fn assert_immediate_retirement(host: &DocumentScriptHost, queued_calls: &Arc<AtomicUsize>) {
    let revoked = matches!(host.permit(), Err(DocumentScriptNativeError::Revoked));
    let calls = queued_calls.load(Ordering::Relaxed);
    let captures = Arc::strong_count(queued_calls);
    let replacement = Process::bootstrap(EngineKind::V8);
    let available = replacement.is_ok();
    drop(replacement);
    assert!(
        revoked && calls == 0 && captures == 1 && available,
        "retirement must return with revoked permit, abandoned queue and released process: revoked={revoked}, calls={calls}, captures={captures}, available={available}"
    );
}

fn retirement_refusals_follow_actual_canonical_publication() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let attempt = |host: &mut DocumentScriptHost, operations| {
        host.with_native(|dom, mut hosted, _| {
            let cancellation = cancel();
            let mut hosted_limits = limits().hosted;
            let canonical = hosted_limits.canonical;
            let records = canonical.records();
            hosted_limits.canonical = DomHostedLimits::new(
                DomHostedRecordLimits::new(
                    records.hosts(),
                    records.browsing_contexts(),
                    records.navigables(),
                    records.entries(),
                    records.globals(),
                    operations,
                    records.registries(),
                ),
                canonical.retained_bytes(),
                canonical.scratch_bytes(),
                canonical.work(),
            );
            hosted
                .set_limits(dom, hosted_limits, &cancellation)
                .unwrap();
            hosted.destroy_top_level(dom, top.navigable(), &cancellation)
        })
        .unwrap()
    };

    let before = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, before.clone());
    // Zero operation identities refuses the initial invocation itself.
    assert_eq!(
        attempt(&mut host, 0).unwrap_err(),
        DomHostedRuntimeError::Dom(DomHostedError::Limit {
            resource: DomHostedResource::Operations,
            limit: 0,
        })
    );
    assert!(host.permit().is_ok());
    assert_eq!(before.load(Ordering::Relaxed), 0);
    assert!(matches!(
        Process::bootstrap(EngineKind::V8),
        Err(Error::OwnerAlreadyActive)
    ));
    host.checkpoint().unwrap();
    assert_eq!(before.load(Ordering::Relaxed), 1);

    let after = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, after.clone());
    // One identity admits the invocation and its canonical retiring mark,
    // but refuses issuance of the distinct cleanup operation identity.
    assert_eq!(
        attempt(&mut host, 1).unwrap_err(),
        DomHostedRuntimeError::Dom(DomHostedError::Limit {
            resource: DomHostedResource::Operations,
            limit: 1,
        })
    );
    assert!(
        host.snapshot()
            .unwrap()
            .hosted_navigable(top.navigable())
            .unwrap()
            .is_retiring()
    );
    assert_immediate_retirement(&host, &after);
    host.with_native(|dom, mut hosted, _| {
        hosted.set_limits(dom, limits().hosted, &cancel()).unwrap();
        destroy_native_top(dom, hosted, top);
    })
    .unwrap();
    let _session = host.stop().into_session().unwrap();
}

fn real_retirement_revokes_and_abandons(foreign: &DocumentScriptPermit) {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    assert!(matches!(
        host.in_document(foreign, |_| panic!("foreign permit entered guest")),
        Err::<(), _>(DocumentScriptError::Native(
            DocumentScriptNativeError::ForeignPermit
        ))
    ));
    let permit = host.permit().unwrap();
    let queued_calls = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, queued_calls.clone());
    retire_attached(&mut host, top);
    assert_immediate_retirement(&host, &queued_calls);
    assert!(matches!(
        host.checkpoint(),
        Err(DocumentScriptError::Native(
            DocumentScriptNativeError::Hosted(
                DomHostedRuntimeError::TargetNotFound
                    | DomHostedRuntimeError::Inactive
                    | DomHostedRuntimeError::Dom(DomHostedError::StaleIdentity)
            )
        ))
    ));
    assert!(matches!(
        host.in_document(&permit, |_| panic!("revoked permit entered guest")),
        Err::<(), _>(DocumentScriptError::Native(
            DocumentScriptNativeError::Revoked
        ))
    ));
    assert_eq!(queued_calls.load(Ordering::Relaxed), 0);
    assert_eq!(Arc::strong_count(&queued_calls), 1);
    let replacement = Process::bootstrap(EngineKind::V8).unwrap();
    drop(replacement);
    let snapshot = host.snapshot().unwrap();
    assert_eq!(
        snapshot.document(top.document()).unwrap().scripting(),
        HtmlScriptingStatus::Disabled
    );
    let _session = host.stop().into_session().unwrap();
}

fn native_refusal_stays_typed_and_sticky() {
    let (session, top) = setup();
    let cancellation = Arc::new(cancel());
    let mut host = prepare(session, cancellation.clone())
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let queued_calls = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, queued_calls.clone());
    let caught = host.in_document(&permit, |scope| {
        cancellation.store(true, Ordering::Relaxed);
        let script = "try { readChildCount(); } catch (_) {} 42"
            .encode_utf16()
            .collect::<Vec<_>>();
        scope.evaluate(&script)?;
        Ok(())
    });
    let original = native_failure(caught);
    let cause = original.cause::<DocumentScriptNativeError>().unwrap();
    assert!(matches!(
        cause,
        DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Cancelled)
    ));
    cancellation.store(false, Ordering::Relaxed);
    let first = native_failure(host.checkpoint());
    assert!(std::ptr::eq(
        cause,
        first.cause::<DocumentScriptNativeError>().unwrap()
    ));
    retire_attached(&mut host, top);
    assert_immediate_retirement(&host, &queued_calls);
    let retained_first = native_failure(host.checkpoint());
    assert!(std::ptr::eq(
        cause,
        retained_first.cause::<DocumentScriptNativeError>().unwrap()
    ));
    let _session = host.stop().into_session().unwrap();
    assert!(std::ptr::eq(
        cause,
        original.cause::<DocumentScriptNativeError>().unwrap()
    ));
}

fn native_failure(result: Result<(), DocumentScriptError>) -> surgeist_bindings_js::HostFailure {
    match result {
        Err(DocumentScriptError::Script(Error::HostFailure(failure))) => failure,
        other => panic!("unexpected native refusal: {other:?}"),
    }
}

fn stopped_scripting_snapshots() -> (
    DocumentId,
    DomSnapshot,
    DomDocumentScriptingObservation,
    DomSnapshot,
) {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let observation = host.observe_scripting(&host.permit().unwrap()).unwrap();
    assert_eq!(observation.status(), HtmlScriptingStatus::Enabled);
    let historical = host.snapshot().unwrap();
    let fresh = host.stop().into_session().unwrap().snapshot();
    (top.document(), historical, observation, fresh)
}

fn stopped_host_captures_known_no_engine() {
    let (document, _, _, fresh) = stopped_scripting_snapshots();
    let capture = fresh
        .html_capture(
            document,
            None,
            capture_limits(),
            DomHtmlCaptureLimits::new(32, 10_000),
            &cancel(),
        )
        .unwrap();
    let status = captured_status(&capture, document);
    assert_eq!(status, HtmlScriptingStatus::Disabled);
    assert_eq!(
        media_scripting_for_status(status),
        Some(CssMediaScripting::None)
    );
}

fn stopped_host_rejects_the_last_live_observation() {
    let (document, historical, observation, fresh) = stopped_scripting_snapshots();
    let adapter = DomHtmlCaptureLimits::new(32, 10_000);
    let prior = historical
        .with_document_scripting(observation)
        .unwrap()
        .html_capture(None, capture_limits(), adapter, &cancel())
        .unwrap();
    assert_eq!(
        captured_status(&prior, document),
        HtmlScriptingStatus::Enabled
    );
    match fresh.with_document_scripting(observation) {
        Err(DomHtmlCaptureError::StaleScriptingObservation) => {}
        Err(error) => panic!("the old observation must be stale: {error:?}"),
        Ok(capture) => {
            let capture = capture
                .html_capture(None, capture_limits(), adapter, &cancel())
                .unwrap();
            panic!(
                "a fresh stopped snapshot accepted the prior live observation: {:?}",
                captured_status(&capture, document)
            );
        }
    }
}

fn stopped_session_reattaches_with_a_new_scripting_basis() {
    let (session, top) = setup();
    let cancellation = Arc::new(cancel());
    let mut host = prepare(session, cancellation.clone())
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let observation = host.observe_scripting(&host.permit().unwrap()).unwrap();
    let historical = host.snapshot().unwrap();
    cancellation.store(true, Ordering::Relaxed);
    let session = host.stop().into_session().unwrap();
    let stopped = session.snapshot();
    assert_eq!(historical.stamp(), stopped.stamp());
    assert_eq!(
        stopped.document(top.document()).unwrap().scripting(),
        HtmlScriptingStatus::Disabled
    );
    cancellation.store(false, Ordering::Relaxed);
    let mut replacement = prepare(session, cancellation)
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let current = replacement.snapshot().unwrap();
    assert_ne!(current.stamp(), stopped.stamp());
    assert!(matches!(
        current.with_document_scripting(observation),
        Err(DomHtmlCaptureError::StaleScriptingObservation)
    ));
    let new_observation = replacement
        .observe_scripting(&replacement.permit().unwrap())
        .unwrap();
    current.with_document_scripting(new_observation).unwrap();
    historical.with_document_scripting(observation).unwrap();
    assert_eq!(
        stopped.document(top.document()).unwrap().scripting(),
        HtmlScriptingStatus::Disabled
    );
    let _session = replacement.stop().into_session().unwrap();
}

fn main() {
    if let Some(scenario) = std::env::args().nth(1) {
        match scenario.as_str() {
            "css-namespace" => native_window_exposes_css_escape(),
            "css-namespace-admission" => native_window_css_admission(),
            "css-namespace-collision" => {
                native_window_css_hook_collision_preserves_the_original_session()
            }
            "conversion-revocation" => source_events::run(),
            "creation-realms" => source_events::run_creation_realms(),
            "native-retirement" => {
                let expired = native_read_snapshot_and_explicit_queue();
                real_retirement_revokes_and_abandons(&expired);
            }
            "retirement-after-refusal" => native_refusal_stays_typed_and_sticky(),
            "retirement-unwind" => retirement_unwind_disposes_the_revoked_guest_owner(),
            "retirement-admission" => retirement_refusals_follow_actual_canonical_publication(),
            "stop-known-no-engine" => stopped_host_captures_known_no_engine(),
            "stop-stale-observation" => stopped_host_rejects_the_last_live_observation(),
            "stop-reattach" => stopped_session_reattaches_with_a_new_scripting_basis(),
            _ => panic!("unknown scripting scenario: {scenario}"),
        }
        return;
    }
    native_window_exposes_css_escape();
    native_window_css_admission();
    native_window_css_hook_collision_preserves_the_original_session();
    failed_enable_preserves_original_owner();
    let expired_permit = native_read_snapshot_and_explicit_queue();
    stopped_host_captures_known_no_engine();
    stopped_host_rejects_the_last_live_observation();
    stopped_session_reattaches_with_a_new_scripting_basis();
    real_retirement_revokes_and_abandons(&expired_permit);
    retirement_unwind_disposes_the_revoked_guest_owner();
    retirement_refusals_follow_actual_canonical_publication();
    native_refusal_stays_typed_and_sticky();
    source_events::run();
    source_events::run_creation_realms();
}
