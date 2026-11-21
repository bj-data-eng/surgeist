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

pub(super) fn guest_media_removal_reaches_the_owning_checkpoint() {
    let (mut session, top) = setup();
    let cancellation = Arc::new(cancel());
    let mut registry = new_registry();
    registry.bind(&mut session.dom(), &cancellation).unwrap();
    let mut host = prepare(session, cancellation)
        .enable_with_guest_event_registry(top, ALLOW, None, &mut Some(registry), guest_limits())
        .unwrap();
    let permit = host.permit().unwrap();
    let parent = host
        .snapshot()
        .unwrap()
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap_or(top.document().node());
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const parent = document.documentElement ?? document;
            const box = parent.appendChild(document.createElement('div'));
            const media = document.createElement('audio');
            globalThis.removalFixture = { box, media };
            return box.appendChild(media) === media && media.parentNode === box;
        })())"#,
        1.0,
    );
    let initial = host.snapshot().unwrap();
    let box_node = *initial.node(parent).unwrap().children().last().unwrap();
    let media = initial.node(box_node).unwrap().children()[0];
    let removed = host.in_document(&permit, |scope| {
        assert_script_number(
            scope,
            r#"Number((() => {
            const {box, media} = removalFixture;
            return box.removeChild(media) === media && media.parentNode === null;
        })())"#,
            1.0,
        )
    });
    if let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = &removed {
        panic!(
            "guest media removal refused: {:?}",
            failure.cause::<DomSourceDriveError>()
        );
    }
    removed.unwrap();
    assert_eq!(autoplay(&host.snapshot().unwrap(), media), Some(true));
    host.checkpoint().unwrap();
    assert_eq!(autoplay(&host.snapshot().unwrap(), media), Some(false));
    assert_eq!(autoplay(&initial, media), Some(true));
    eval(
        &mut host,
        &permit,
        "Number(removalFixture.media.parentNode === null)",
        1.0,
    );
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const {box} = removalFixture;
            const early = box.appendChild(document.createElement('audio'));
            const late = box.appendChild(document.createElement('video'));
            Object.assign(removalFixture, {early, late});
            return early.parentNode === box && late.parentNode === box;
        })())"#,
        1.0,
    );
    let both_attached = host.snapshot().unwrap();
    let [early, late] = *both_attached.node(box_node).unwrap().children() else {
        panic!("the script created two real child nodes");
    };
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const {box, early, late} = removalFixture;
            box.removeChild(early);
            Promise.resolve().then(() => {
                box.appendChild(early);
                box.appendChild(late);
            });
            box.removeChild(late);
            return early.parentNode === null && late.parentNode === null;
        })())"#,
        1.0,
    );
    assert_eq!(autoplay(&host.snapshot().unwrap(), early), Some(true));
    assert_eq!(autoplay(&host.snapshot().unwrap(), late), Some(true));
    host.checkpoint().unwrap();
    let reinserted = host.snapshot().unwrap();
    assert_eq!(reinserted.node(early).unwrap().parent(), Some(box_node));
    assert_eq!(reinserted.node(late).unwrap().parent(), Some(box_node));
    assert_eq!(autoplay(&reinserted, early), Some(false));
    assert_eq!(autoplay(&reinserted, late), Some(true));

    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.detach(late);
        dom.apply(batch, &cancel())
    })
    .unwrap()
    .unwrap();
    assert_eq!(autoplay(&host.snapshot().unwrap(), late), Some(true));
    host.checkpoint().unwrap();
    assert_eq!(autoplay(&host.snapshot().unwrap(), late), Some(false));

    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const {box} = removalFixture;
            const fresh = box.appendChild(document.createElement('video'));
            globalThis.textContentMedia = fresh;
            return fresh.parentNode === box;
        })())"#,
        1.0,
    );
    let before_text = host.snapshot().unwrap();
    let text_media = *before_text
        .node(box_node)
        .unwrap()
        .children()
        .last()
        .unwrap();
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const {box} = removalFixture;
            box.textContent = "";
            Promise.resolve().then(() => box.appendChild(textContentMedia));
            return textContentMedia.parentNode === null;
        })())"#,
        1.0,
    );
    host.checkpoint().unwrap();
    let after_text = host.snapshot().unwrap();
    assert_eq!(
        after_text.node(text_media).unwrap().parent(),
        Some(box_node)
    );
    assert_eq!(autoplay(&after_text, text_media), Some(false));

    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const {box} = removalFixture;
            const stopped = box.appendChild(document.createElement('audio'));
            globalThis.stoppedMedia = stopped;
            return stopped.parentNode === box;
        })())"#,
        1.0,
    );
    let before_stop = host.snapshot().unwrap();
    let stopped = *before_stop
        .node(box_node)
        .unwrap()
        .children()
        .last()
        .unwrap();
    eval(
        &mut host,
        &permit,
        "Number(removalFixture.box.removeChild(stoppedMedia) === stoppedMedia)",
        1.0,
    );
    let mut session = host.stop().into_session().unwrap();
    assert_eq!(session.media_checkpoint_step(&cancel()).unwrap(), None);
    assert_eq!(autoplay(&session.snapshot(), stopped), Some(true));
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

fn autoplay(snapshot: &DomSnapshot, node: NodeId) -> Option<bool> {
    let DomNodeKind::Element(element) = snapshot.node(node).unwrap().kind() else {
        panic!("the exact node must remain a media element");
    };
    element.media_can_autoplay()
}
