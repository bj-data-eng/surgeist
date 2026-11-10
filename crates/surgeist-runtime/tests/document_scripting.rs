//! Real entry-thread V8 ownership over an original native Runtime Session.

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use surgeist_bindings_js::{
    Callback, Completion, EngineKind, Error, Primitive, Process, PropertyKey, Scope,
};
use surgeist_css::{query::media::CssMediaScripting, source::CssUtf16InputLimits};
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
    let status = no_script
        .document(target.document())
        .unwrap()
        .metadata()
        .document_environment()
        .scripting();
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
    assert_eq!(environment.scripting(), HtmlScriptingStatus::Disabled);
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
        HtmlScriptingStatus::Unavailable
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
    host.with_native(|dom, mut hosted, _| {
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
    })
    .unwrap();
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
    let environment = snapshot
        .document(top.document())
        .unwrap()
        .metadata()
        .document_environment();
    assert_eq!(environment.scripting(), HtmlScriptingStatus::Disabled);
    let _session = host.stop().into_session().unwrap();
}

fn native_refusal_stays_typed_and_sticky() {
    let (session, top) = setup();
    let cancellation = Arc::new(cancel());
    let mut host = prepare(session, cancellation.clone())
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
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

fn main() {
    failed_enable_preserves_original_owner();
    let expired_permit = native_read_snapshot_and_explicit_queue();
    real_retirement_revokes_and_abandons(&expired_permit);
    native_refusal_stays_typed_and_sticky();
    source_events::run();
}
