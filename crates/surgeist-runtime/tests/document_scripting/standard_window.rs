//! Production Window attachment without delegated host services.

use super::*;

pub(super) fn window_without_host_hooks_uses_standard_globals() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, None)
        .unwrap();
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());
    assert_eq!(
        host.observe_scripting(&permit).unwrap().status(),
        HtmlScriptingStatus::Enabled
    );
    assert_no_hook_globals(&mut host, &permit);
    eval(
        &mut host,
        &permit,
        "Number(window === globalThis && self === window && parent === window && opener === null)",
        1.0,
    );
    eval(
        &mut host,
        &permit,
        r#"Number(CSS.escape("0start") === "\\30 start")"#,
        1.0,
    );
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const exception = new DOMException("refused", "InvalidStateError");
            return exception.name === "InvalidStateError" && exception.message === "refused"
                && exception.code === 11;
        })())"#,
        1.0,
    );
    let queued_calls = Arc::new(AtomicUsize::new(0));
    queue_observable_job(&mut host, &permit, queued_calls.clone());
    let session = host.stop().into_session().unwrap();
    assert_eq!(queued_calls.load(Ordering::Relaxed), 0);
    assert_eq!(
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .scripting(),
        HtmlScriptingStatus::Disabled
    );
}

pub(super) fn guest_window_without_host_hooks_recovers_from_policy_refusal() {
    let (mut session, top) = setup();
    let cancellation = Arc::new(cancel());
    let mut registry = new_registry();
    registry.bind(&mut session.dom(), &cancellation).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let native_calls = calls.clone();
    let document = DomEventTarget::from(top.document().node());
    let window = DomEventTarget::from(top.global());
    registry
        .add_listener(
            &mut session.dom(),
            window.clone(),
            "standard-window".encode_utf16().collect::<Vec<_>>(),
            DomEventCallback::new(move |_, _, _, event| {
                assert_eq!(event.target().as_ref(), Some(&document));
                assert_eq!(event.current_target().as_ref(), Some(&window));
                native_calls.fetch_add(1, Ordering::Relaxed);
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions {
                capture: true,
                ..Default::default()
            },
        )
        .unwrap();
    let mut registry = Some(registry);
    let stamp = session.snapshot().stamp();
    let failure = prepare(session, cancellation.clone())
        .enable_with_guest_event_registry(
            top,
            HtmlScriptingPolicy {
                user_allows: false,
                ..ALLOW
            },
            None,
            &mut registry,
            guest_limits(),
        )
        .unwrap_err();
    assert!(matches!(
        failure.error,
        DocumentGuestEventError::Document(DocumentScriptError::Native(
            DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Dom(
                DomHostedError::ScriptingDisabled
            ))
        ))
    ));
    assert!(registry.is_some());
    let session = failure.owner.into_session().unwrap();
    let unpublished = session.snapshot();
    assert_eq!(unpublished.stamp(), stamp);
    assert_eq!(
        unpublished.document(top.document()).unwrap().scripting(),
        HtmlScriptingStatus::Disabled
    );

    let mut host = prepare(session, cancellation.clone())
        .enable_with_guest_event_registry(top, ALLOW, None, &mut registry, guest_limits())
        .unwrap();
    assert!(registry.is_none());
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());
    assert_eq!(
        host.observe_scripting(&permit).unwrap().status(),
        HtmlScriptingStatus::Enabled
    );
    assert_no_hook_globals(&mut host, &permit);
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const doc = window.document;
            const node = doc.appendChild(doc.createComment("from standard Window"));
            return doc === document && node instanceof Comment && node.parentNode === doc
                && node.ownerDocument === doc;
        })())"#,
        1.0,
    );
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const event = new Event("standard-window", { bubbles: true, cancelable: true });
            let reached = false;
            document.addEventListener(event.type, function (delivered) {
                const [target, global] = delivered.composedPath();
                reached = delivered === event && delivered.defaultPrevented
                    && target === document && global === window && this === document;
            });
            return !document.dispatchEvent(event) && reached && event.currentTarget === null;
        })())"#,
        1.0,
    );
    assert_eq!(calls.load(Ordering::Relaxed), 1);

    let mut session = host.stop().into_session().unwrap();
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.document(top.document()).unwrap().scripting(),
        HtmlScriptingStatus::Disabled
    );
    let attached_document = snapshot.node(top.document().node()).unwrap();
    assert!(attached_document.children().iter().any(|node| matches!(
        snapshot.node(*node).unwrap().kind(),
        DomNodeKind::Comment(data) if data.code_units().iter().copied().eq("from standard Window".encode_utf16())
    )));
    new_registry()
        .bind(&mut session.dom(), &cancellation)
        .unwrap();
}

fn assert_no_hook_globals(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit) {
    eval(
        host,
        permit,
        r#"Number(["readChildCount", "detachIf", ""].every(key => !(key in globalThis)))"#,
        1.0,
    );
}

fn new_registry() -> DomEventDispatcher<()> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 128_000,
        work: 100_000,
    })
    .unwrap()
}

fn guest_limits() -> DocumentGuestEventLimits {
    DocumentGuestEventLimits {
        storage_bytes: 512 * 1024,
        work: 10_000,
        pending_exception_reports: 16,
        node_retirement: DomNodeRetirementLimits::new(16_000_000, 1_000),
        source_edits: DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
        dispatch: DomEventDispatchLimits {
            path_items: 4,
            snapshot_listeners: 16,
            failure_records: 8,
            callbacks: 16,
            work: 10_000,
        },
    }
}
