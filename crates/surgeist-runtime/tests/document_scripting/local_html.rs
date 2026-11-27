use super::standard_window::{guest_limits, new_registry};
use super::*;
use std::num::NonZeroUsize;
use surgeist_dom::hosted::DomHostedHtmlLoadStep;
use surgeist_html_parser::{HtmlParseLimits, HtmlSource, live::HtmlLiveParseProgress};
use surgeist_runtime::document_scripting::{
    DocumentHtmlCompletionProgress as Progress, WindowTimerLimits, WindowTimerProgress,
};

fn parse_limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(128)
        .buffered_utf16_units(1024)
        .token_attributes(8)
        .auxiliary_entries(256)
        .managed_nodes(64)
        .managed_edges(64)
        .managed_attributes(16)
        .managed_utf16_units(1024)
        .source_map_entries(0)
        .source_map_payload_units(0)
        .diagnostics(0)
        .diagnostic_payload_units(0)
        .context_effects(0)
        .open_elements(16)
        .active_formatting_entries(0)
        .tree_depth(16)
        .managed_capacity_bytes(1_048_576)
        .peak_transient_bytes(1_048_576)
        .atomic_layout_bytes(65_536)
        .work_credits(1_048_576)
        .finish()
        .unwrap()
}

fn session_with_event_recipients() -> DomSession {
    let mut dom = native_dom();
    dom.set_limits(
        dom.limits().with_event_recipients(
            surgeist_dom::event::DomEventRecipientLimits::new(8, 16_000_000, 16_000_000, 100_000)
                .unwrap(),
        ),
    )
    .unwrap();
    DomSession::new(dom, limits()).unwrap()
}

pub(super) fn window_load_waits_for_original_object_representation_task() {
    use surgeist_dom::document::DomDocumentReadiness;
    use surgeist_html::elements::object::HtmlObjectRepresentation;
    use surgeist_runtime::element_task::{DomElementTaskProgress, DomElementTaskRunLimits};

    let source = HtmlSource::utf8(
        "<!doctype html><html><head></head><body><div>completion</div></body></html>",
    );
    let cancellation = Arc::new(cancel());
    let mut session = session_with_event_recipients();
    let mut hosted = session.hosted_execution();
    let mut registry = Some(new_registry());
    let (other, top) = {
        let mut dom = session.dom();
        let owner = hosted.create_host(&mut dom, &cancellation).unwrap();
        let other = hosted
            .create_top_level(&mut dom, owner, &cancellation)
            .unwrap();
        let local = hosted
            .create_local_html_top_level(&mut dom, owner, &cancellation)
            .unwrap();
        (other, local)
    };
    let limits = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1);
    let tasks = session.element_task_execution();
    let mut parser = {
        let mut dom = session.dom();
        let parser = hosted
            .begin_local_html_parser(&mut dom, top, &source, parse_limits(), &cancellation)
            .unwrap();
        registry
            .as_mut()
            .unwrap()
            .bind(&mut dom, &cancellation)
            .unwrap();
        parser
    };
    let mut host = prepare(session, cancellation.clone())
        .enable_with_guest_event_registry(top, ALLOW, None, &mut registry, guest_limits())
        .unwrap();
    let permit = host.permit().unwrap();
    eval(
        &mut host,
        &permit,
        "globalThis.loaded = 0; addEventListener('load', () => { loaded += 1 }); 0",
        0.0,
    );
    let mut continuation = host
        .with_native(|dom, hosted, _| {
            parser.accept_eof();
            assert_eq!(
                parser
                    .advance(&hosted, dom, NonZeroUsize::new(32).unwrap(), &cancellation)
                    .unwrap(),
                HtmlLiveParseProgress::StopParsing
            );
            parser
                .take_stop_parsing(&hosted, dom, &cancellation)
                .unwrap()
        })
        .unwrap();
    host.begin_html_completion(&permit, &mut continuation)
        .unwrap();
    let dispatch = guest_limits().dispatch;
    assert!(matches!(
        host.service_html_completion(&permit, dispatch).unwrap(),
        Progress::Interactive
    ));
    assert!(matches!(
        host.service_html_completion(&permit, dispatch).unwrap(),
        Progress::DomContentLoaded
    ));
    let apply = |host: &mut DocumentScriptHost, edit| {
        let result = host
            .with_native(|dom, _, mut source| {
                source
                    .apply(
                        dom,
                        edit,
                        limits,
                        &mut DomSourceServices {
                            meta: None,
                            execution_scope: None,
                            style_policy: &mut |_| None,
                            url: None,
                        },
                        &cancellation,
                    )
                    .unwrap()
            })
            .unwrap();
        let DomSourceDriveProgress::Complete(completion) = result else {
            panic!("the native Object task must be admitted");
        };
        completion
    };
    let create =
        || DomSourceEdit::create_element_ns(top.document().node(), Namespace::html(), "object");
    let object = apply(&mut host, create()).published_node().unwrap();
    let detached = apply(
        &mut host,
        DomSourceEdit::create_element_ns(top.document().node(), Namespace::html(), "div"),
    )
    .published_node()
    .unwrap();
    apply(
        &mut host,
        DomSourceEdit::insert_before(object, detached, None),
    );
    apply(
        &mut host,
        DomSourceEdit::set_attribute(object, "data", "unfetched"),
    );
    apply(&mut host, DomSourceEdit::remove_attribute(object, "data"));
    let retired = apply(&mut host, create()).published_node().unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(retired);
        dom.apply(batch, &cancellation).unwrap();
    })
    .unwrap();
    let source_kind = DomElementTaskSourceKind::DomManipulation;
    let queued = tasks.pending_task(source_kind, 0).unwrap().unwrap();
    assert_eq!(queued.id().target(), object);
    assert_eq!(tasks.pending_tasks(source_kind).unwrap(), 4);
    let waiting = host.snapshot().unwrap();
    assert!(matches!(
        host.service_html_completion(&permit, dispatch).unwrap(),
        Progress::WaitingForObjectTask(task) if task == queued.id()
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), waiting.stamp());
    assert_eq!(
        waiting
            .document(top.document())
            .unwrap()
            .metadata()
            .readiness(),
        DomDocumentReadiness::Interactive
    );
    eval(&mut host, &permit, "loaded", 0.0);
    assert!(matches!(
        host.run_attached_element_task(&permit, source_kind, DomElementTaskRunLimits {
            dispatch, source: limits,
        }).unwrap(),
        DomElementTaskProgress::ObjectRepresentationComplete { task, .. } if task.id() == queued.id()
    ));
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(object)
            .unwrap()
            .as_element()
            .unwrap()
            .object_representation(),
        Some(HtmlObjectRepresentation::FallbackContent)
    );
    for _ in 0..2 {
        let queued = tasks.pending_task(source_kind, 0).unwrap().unwrap();
        assert_eq!(queued.id().target(), object);
        assert!(matches!(
            host.service_html_completion(&permit, dispatch).unwrap(),
            Progress::WaitingForObjectTask(task) if task == queued.id()
        ));
        assert!(matches!(
            host.run_attached_element_task(&permit, source_kind, DomElementTaskRunLimits {
                dispatch, source: limits,
            }).unwrap(),
            DomElementTaskProgress::ObjectRepresentationComplete { task, .. } if task.id() == queued.id()
        ));
    }
    assert_eq!(
        tasks
            .pending_task(source_kind, 0)
            .unwrap()
            .unwrap()
            .id()
            .target(),
        retired
    );
    let transferred = apply(
        &mut host,
        DomSourceEdit::create_element_ns(other.document().node(), Namespace::html(), "object"),
    )
    .published_node()
    .unwrap();
    apply(
        &mut host,
        DomSourceEdit::adopt(transferred, top.document().node()),
    );
    let other_task = tasks.pending_task(source_kind, 1).unwrap().unwrap();
    assert_eq!(other_task.id().target(), transferred);
    assert_eq!(
        other_task.queue_association().unwrap().global(),
        other.global()
    );
    assert!(
        matches!(host.service_html_completion(&permit, dispatch).unwrap(),
        Progress::WaitingForObjectTask(task) if task == other_task.id())
    );
    apply(
        &mut host,
        DomSourceEdit::adopt(transferred, other.document().node()),
    );
    assert_eq!(
        tasks.pending_task(source_kind, 1).unwrap().unwrap().id(),
        other_task.id()
    );
    assert!(matches!(
        host.service_html_completion(&permit, dispatch).unwrap(),
        Progress::Loaded
    ));
    eval(&mut host, &permit, "loaded", 1.0);
}

pub(super) fn window_listener_attaches_to_the_visible_prefix_before_eof() {
    let foreign = initial_document_has_no_load_task();
    let source = HtmlSource::utf16(
        "<!doctype html><html><head></head><body><div id=p>A😀&amp;β</div></body></html>".into(),
    );
    let cancellation = Arc::new(cancel());
    let mut session = session_with_event_recipients();
    let mut hosted = session.hosted_execution();
    let mut registry = Some(new_registry());
    let (top, mut parser) = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancellation).unwrap();
        let top = hosted
            .create_local_html_top_level(&mut dom, host, &cancellation)
            .unwrap();
        let mut parser = hosted
            .begin_local_html_parser(&mut dom, top, &source, parse_limits(), &cancellation)
            .unwrap();
        assert_eq!(
            parser
                .advance(
                    &hosted,
                    &mut dom,
                    NonZeroUsize::new(7).unwrap(),
                    &cancellation
                )
                .unwrap(),
            HtmlLiveParseProgress::Paused
        );
        registry
            .as_mut()
            .unwrap()
            .bind(&mut dom, &cancellation)
            .unwrap();
        (top, parser)
    };
    let mut guest = guest_limits();
    // Leave adapter work for the retained load and pageshow sequence.
    guest.work = 20_000;
    let mut host = prepare(session, cancellation.clone())
        .with_window_timer_limits(WindowTimerLimits::new(4, 2, 4096))
        .enable_with_guest_event_registry(top, ALLOW, None, &mut registry, guest)
        .unwrap();
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            globalThis.prefix = document.getElementById("p");
            globalThis.prefixText = prefix.firstChild;
            globalThis.observed = 0;
            globalThis.lifecycle = [];
            document.addEventListener("readystatechange", function(event) {
                lifecycle.push(this === document && event.target === document &&
                    event.currentTarget === document && event.isTrusted &&
                    !event.bubbles && !event.cancelable ? document.readyState : "bad ready");
            });
            window.addEventListener("DOMContentLoaded", function(event) {
                lifecycle.push(this === window && event.target === document &&
                    event.currentTarget === window && event.isTrusted && event.bubbles &&
                    !event.cancelable && document.readyState === "interactive" ? "content" : "bad content");
            });
            document.addEventListener("load", () => lifecycle.push("bad Document load"));
            window.addEventListener("load", function(event) {
                lifecycle.push(this === window && self === window && parent === window &&
                    opener === null && event.target === document && event.currentTarget === window &&
                    event.eventPhase === Event.AT_TARGET && event.isTrusted && !event.bubbles &&
                    !event.cancelable && event.timeStamp >= 0 &&
                    document.readyState === "complete" ? "load" : "bad load");
                Promise.resolve().then(() => {
                    lifecycle.push(event.currentTarget === window && window.event === event &&
                        event.eventPhase === Event.AT_TARGET ? "microtask" : "bad microtask");
                    window.removeEventListener("load", removedByJob);
                });
                setTimeout(() => lifecycle.push("timer"), 0);
                globalThis.trustedLoad = event;
            });
            function removedByJob() { lifecycle.push("bad removed listener"); }
            window.addEventListener("load", removedByJob);
            window.addEventListener("load", () => lifecycle.push("last load"), { once: true });
            window.addEventListener("pageshow", function(event) {
                const path = event.composedPath();
                lifecycle.push(event instanceof PageTransitionEvent && event.persisted === false &&
                    this === window && event.target === document && event.currentTarget === window &&
                    path.length === 1 && path[0] === window && event.isTrusted &&
                    event.bubbles && event.cancelable && !event.composed ? "pageshow" : "bad pageshow");
            });
            window.addEventListener("parser-observation", function(event) {
                if (this === window && event.currentTarget === window) observed++;
            }, { once: true });
            return document.readyState === "loading" &&
                prefixText.data === "A" && prefix.ownerDocument === document;
        })())"#,
        1.0,
    );
    let before_eof = host
        .with_native(|dom, hosted, _| {
            parser
                .advance(&hosted, dom, NonZeroUsize::new(32).unwrap(), &cancellation)
                .unwrap()
        })
        .unwrap();
    assert_eq!(before_eof, HtmlLiveParseProgress::AwaitingEof);
    let handoff = host
        .with_native(|dom, hosted, _| {
            assert!(
                parser
                    .take_stop_parsing(&hosted, dom, &cancellation)
                    .unwrap()
                    .is_none()
            );
            parser.accept_eof();
            assert_eq!(
                parser
                    .advance(&hosted, dom, NonZeroUsize::MIN, &cancellation)
                    .unwrap(),
                HtmlLiveParseProgress::StopParsing
            );
            let handoff = parser
                .take_stop_parsing(&hosted, dom, &cancellation)
                .unwrap()
                .expect("the original EOF supplies one handoff");
            assert!(
                parser
                    .take_stop_parsing(&hosted, dom, &cancellation)
                    .unwrap()
                    .is_none()
            );
            handoff
        })
        .unwrap();
    assert_eq!(handoff.top_level(), top);
    assert_eq!(
        handoff.parser_continuation().document(),
        top.document().node()
    );
    assert_eq!(
        handoff.parser_continuation().eof().source(),
        source.identity()
    );
    assert_eq!(handoff.parser_continuation().eof().start(), source.len());
    assert!(handoff.parser_continuation().parser_scripts_are_absent());

    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            window.dispatchEvent(new Event("parser-observation"));
            return observed === 1 && prefix === document.getElementById("p")
                && prefixText === prefix.firstChild && prefixText.data === "A😀&β";
        })())"#,
        1.0,
    );
    let mut handoff = Some(handoff);
    assert!(matches!(
        host.begin_html_completion(&foreign, &mut handoff),
        Err(
            surgeist_runtime::document_scripting::DocumentHtmlCompletionError::Script(
                DocumentScriptError::Native(DocumentScriptNativeError::ForeignPermit)
            )
        )
    ));
    assert!(handoff.is_some());
    host.begin_html_completion(&permit, &mut handoff).unwrap();
    assert!(handoff.is_none());
    let dispatch = guest_limits().dispatch;
    let mut shut = dispatch;
    shut.callbacks = 0;
    assert!(matches!(
        host.service_html_completion(&permit, shut).unwrap(),
        Progress::Retained(_)
    ));
    eval(
        &mut host,
        &permit,
        r#"Number(document.readyState === "interactive" && lifecycle.length === 0)"#,
        1.0,
    );
    cancellation.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(host.service_html_completion(&permit, dispatch).is_err());
    cancellation.store(false, std::sync::atomic::Ordering::Relaxed);
    assert!(matches!(
        host.service_html_completion(&permit, dispatch).unwrap(),
        Progress::Interactive
    ));
    assert!(matches!(
        host.service_html_completion(&permit, dispatch).unwrap(),
        Progress::DomContentLoaded
    ));
    eval(
        &mut host,
        &permit,
        r#"Number(lifecycle.join(",") === "interactive,content")"#,
        1.0,
    );
    let mut one = dispatch;
    one.callbacks = 1;
    assert!(matches!(
        host.service_html_completion(&permit, one).unwrap(),
        Progress::Retained(_)
    ));
    assert!(matches!(
        host.service_window_timer(&permit),
        Err(DocumentScriptError::Native(DocumentScriptNativeError::Busy))
    ));
    eval(
        &mut host,
        &permit,
        r#"Number(lifecycle.join(",") === "interactive,content,complete,load,microtask")"#,
        1.0,
    );
    assert!(matches!(
        host.service_html_completion(&permit, dispatch).unwrap(),
        Progress::Loaded
    ));
    eval(
        &mut host,
        &permit,
        r#"Number(lifecycle.join(",") === "interactive,content,complete,load,microtask,last load,pageshow" &&
            trustedLoad.target === document && trustedLoad.currentTarget === null &&
            trustedLoad.eventPhase === Event.NONE && trustedLoad.composedPath().length === 0)"#,
        1.0,
    );
    let snapshot = host.snapshot().unwrap();
    let timing = snapshot
        .document(top.document())
        .unwrap()
        .metadata()
        .local_html_load_timing()
        .unwrap();
    assert_eq!(timing.step(), DomHostedHtmlLoadStep::Finished);
    for [before, after] in [
        [
            DomHostedHtmlLoadStep::Interactive,
            DomHostedHtmlLoadStep::ContentLoadedStart,
        ],
        [
            DomHostedHtmlLoadStep::ContentLoadedStart,
            DomHostedHtmlLoadStep::ContentLoadedEnd,
        ],
        [
            DomHostedHtmlLoadStep::ContentLoadedEnd,
            DomHostedHtmlLoadStep::Complete,
        ],
        [
            DomHostedHtmlLoadStep::Complete,
            DomHostedHtmlLoadStep::LoadStart,
        ],
        [
            DomHostedHtmlLoadStep::LoadStart,
            DomHostedHtmlLoadStep::LoadEnd,
        ],
        [
            DomHostedHtmlLoadStep::LoadEnd,
            DomHostedHtmlLoadStep::PageShowing,
        ],
        [
            DomHostedHtmlLoadStep::PageShowing,
            DomHostedHtmlLoadStep::Finished,
        ],
    ] {
        assert!(timing.microseconds(before).unwrap() <= timing.microseconds(after).unwrap());
    }
    assert!(matches!(
        host.service_window_timer(&permit).unwrap(),
        WindowTimerProgress::Fired {
            exception: None,
            ..
        }
    ));
    eval(
        &mut host,
        &permit,
        r#"Number(lifecycle.join(",") === "interactive,content,complete,load,microtask,last load,pageshow,timer")"#,
        1.0,
    );
    assert!(matches!(
        host.service_html_completion(&permit, dispatch).unwrap(),
        Progress::Idle
    ));
    assert!(host.stop().into_session().is_ok());
    refuses_retired_document();
}

fn initial_document_has_no_load_task() -> DocumentScriptPermit {
    let (session, initial) = setup();
    let mut initial = prepare(session, Arc::new(cancel()))
        .enable(initial, ALLOW, None)
        .unwrap();
    let permit = initial.permit().unwrap();
    assert_eq!(
        initial
            .snapshot()
            .unwrap()
            .document(permit.document())
            .unwrap()
            .metadata()
            .readiness(),
        surgeist_dom::document::DomDocumentReadiness::Complete
    );
    assert!(matches!(
        initial.begin_html_completion(&permit, &mut None),
        Err(surgeist_runtime::document_scripting::DocumentHtmlCompletionError::MissingContinuation)
    ));
    assert!(matches!(
        initial
            .service_html_completion(&permit, guest_limits().dispatch)
            .unwrap(),
        Progress::Idle
    ));
    assert!(initial.stop().into_session().is_ok());
    permit
}

fn refuses_retired_document() {
    let src = HtmlSource::utf16("<!doctype html><html><head></head><body></body></html>".into());
    let cancellation = Arc::new(cancel());
    let mut session = session(limits());
    let mut hosted = session.hosted_execution();
    let mut registry = Some(new_registry());
    let (top, mut parser) = {
        let mut dom = session.dom();
        let owner = hosted.create_host(&mut dom, &cancellation).unwrap();
        let top = hosted
            .create_local_html_top_level(&mut dom, owner, &cancellation)
            .unwrap();
        let parser = hosted
            .begin_local_html_parser(&mut dom, top, &src, parse_limits(), &cancellation)
            .unwrap();
        registry
            .as_mut()
            .unwrap()
            .bind(&mut dom, &cancellation)
            .unwrap();
        (top, parser)
    };
    let mut script = prepare(session, cancellation.clone())
        .enable_with_guest_event_registry(top, ALLOW, None, &mut registry, guest_limits())
        .unwrap();
    let permit = script.permit().unwrap();
    let mut continuation = script
        .with_native(|dom, hosted, _| {
            parser.accept_eof();
            assert_eq!(
                parser
                    .advance(&hosted, dom, NonZeroUsize::new(32).unwrap(), &cancellation)
                    .unwrap(),
                HtmlLiveParseProgress::StopParsing
            );
            parser
                .take_stop_parsing(&hosted, dom, &cancellation)
                .unwrap()
        })
        .unwrap();
    script
        .begin_html_completion(&permit, &mut continuation)
        .unwrap();
    let dispatch = guest_limits().dispatch;
    assert!(matches!(
        script.service_html_completion(&permit, dispatch).unwrap(),
        Progress::Interactive
    ));
    assert!(matches!(
        script.service_html_completion(&permit, dispatch).unwrap(),
        Progress::DomContentLoaded
    ));
    retire_attached(&mut script, top);
    assert!(matches!(
        script.service_html_completion(&permit, dispatch),
        Err(
            surgeist_runtime::document_scripting::DocumentHtmlCompletionError::Script(
                DocumentScriptError::Native(DocumentScriptNativeError::Revoked)
            )
        )
    ));
}

fn eval(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit, source: &str, expected: f64) {
    if let Err(error) = host.in_document(permit, |scope| {
        let observed = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
        let expected = scope.primitive(Primitive::Number(expected))?;
        assert!(
            scope.strict_equals(&observed, &expected)?,
            "failed assertion: {source}"
        );
        Ok(())
    }) {
        let mut cause = std::error::Error::source(&error);
        while let Some(error) = cause {
            eprintln!("{error:?}");
            cause = error.source();
        }
        panic!("local HTML script refused: {error:?}");
    }
}
