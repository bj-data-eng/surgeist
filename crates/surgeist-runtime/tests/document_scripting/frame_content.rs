// Test-only draft for crates/surgeist-runtime/tests/document_scripting/frame_content.rs.
// Wire both functions into the aggregate main and its focused scenario dispatch
// (see frame-content-red.md). The first current failure is the Frame descriptor.
use super::{
    window_length::{content_navigable, edit, observe},
    *,
};
use surgeist_dom::{document::DomDocumentCreationCause, shadow::DomShadowRootInit};
use surgeist_html::construction::HtmlShadowRootMode;

fn attach(session: DomSession, top: DomHostedTopLevel) -> DocumentScriptHost {
    attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 220_000),
        surgeist_ledger::ProfileMode::Summary { timing: false },
        DomSourceLimits::new(16_000_000, 1_000_000, 200_000, 64, 64, 64, 4),
        |_, _, _| {},
    )
    .0
}

fn document_body(snapshot: &DomSnapshot, document: DocumentId) -> NodeId {
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    snapshot.node(html).unwrap().children()[1]
}

pub(super) fn frame_content_follows_native_child_edges_and_identity() {
    let (mut session, top) = setup();
    let document = top.document();
    let body = document_body(&session.snapshot(), document);
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(document, None);
    let group = batch.create_element_ns(&detached, Namespace::html(), "div");
    let frame = batch.create_element_ns(&detached, Namespace::html(), "frame");
    let iframe = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    batch.set_attribute(&frame, "id", "contentFrameProbe");
    batch.set_attribute(&iframe, "id", "contentIframeProbe");
    batch.insert_before(&frame, &group, None);
    batch.insert_before(&iframe, &group, None);
    let shadow_host = batch.create_element_ns(document, Namespace::html(), "div");
    batch.insert_before(&shadow_host, body, None);
    let shadow = batch.attach_shadow(
        &shadow_host,
        DomShadowRootInit::new(HtmlShadowRootMode::Open),
    );
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let [group, frame, iframe] =
        [&group, &frame, &iframe].map(|ticket| commit.created(ticket).unwrap().unwrap());
    let shadow = commit.attached_shadow_root(&shadow).unwrap().unwrap();
    let mut host = attach(session, top);
    let permit = host.permit().unwrap();

    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                for (const prototype of [HTMLFrameElement.prototype, HTMLIFrameElement.prototype]) {
                    for (const member of ['contentDocument', 'contentWindow']) {
                        const desc = Object.getOwnPropertyDescriptor(prototype, member);
                        if (typeof desc?.get !== 'function')
                            return 'missing ' + prototype.constructor.name + '.prototype.' + member + ' getter';
                        if (!desc.enumerable || !desc.configurable || desc.set !== undefined ||
                            desc.get.name !== 'get ' + member || desc.get.length !== 0)
                            return 'incorrect readonly accessor ' + member;
                    }
                }
                const svg = Object.getOwnPropertyDescriptor(HTMLIFrameElement.prototype, 'getSVGDocument');
                if (typeof svg?.value !== 'function')
                    return 'missing HTMLIFrameElement.prototype.getSVGDocument operation';
                if (!svg.writable || !svg.enumerable || !svg.configurable ||
                    svg.value.name !== 'getSVGDocument' || svg.value.length !== 0)
                    return 'incorrect HTMLIFrameElement.prototype.getSVGDocument operation';
                if (Object.hasOwn(HTMLFrameElement.prototype, 'getSVGDocument'))
                    return 'Frame has no getSVGDocument operation';
                for (const tag of ['frame', 'iframe']) {
                    const detached = document.implementation.createHTMLDocument().createElement(tag);
                    if (detached.contentDocument !== null || detached.contentWindow !== null)
                        return 'a never-inserted container has no content navigable';
                }
                return 'ok';
            })()"#,
        ),
        "ok",
        "frame content interface descriptors and fresh detached elements",
    );

    edit(&mut host, DomSourceEdit::insert_before(group, body, None));
    let snapshot = host.snapshot().unwrap();
    let [frame_child, iframe_child] = [frame, iframe].map(|container| {
        let id = content_navigable(&snapshot, container).unwrap();
        let nav = snapshot.hosted_navigable(id).unwrap();
        assert_eq!(nav.parent(), Some(top.navigable()));
        assert_eq!(nav.container(), Some(container));
        let child = nav.active_document().unwrap();
        assert_eq!(snapshot.global_document(nav.global()).unwrap(), child);
        assert_ne!(child, document);
        let child_metadata = snapshot.document(child).unwrap().metadata();
        assert_eq!(
            child_metadata.origin(),
            snapshot.document(document).unwrap().metadata().origin()
        );
        assert!(matches!(
            child_metadata.creation_cause(),
            DomDocumentCreationCause::HostedInitialAboutBlank { .. }
        ));
        (id, child, nav.browsing_context(), nav.global())
    });
    assert_ne!(frame_child.2, iframe_child.2);
    assert_ne!(frame_child.3, iframe_child.3);
    host.with_native(|dom, _, _| {
        let mut markers = DomBatch::new();
        markers.set_attribute(
            document_body(&snapshot, frame_child.1),
            "id",
            "nativeFrameChildBody",
        );
        markers.set_attribute(
            document_body(&snapshot, iframe_child.1),
            "id",
            "nativeIframeChildBody",
        );
        dom.apply(markers, &cancel()).unwrap();
    })
    .unwrap();

    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const frame = document.getElementById('contentFrameProbe');
                const iframe = document.getElementById('contentIframeProbe');
                const failType = call => {
                    try { call(); } catch (error) { return error instanceof TypeError; }
                    return false;
                };
                for (const [receiver, other] of [[frame, iframe], [iframe, frame]]) {
                    const proto = Object.getPrototypeOf(receiver);
                    for (const member of ['contentDocument', 'contentWindow']) {
                        const get = Object.getOwnPropertyDescriptor(proto, member).get;
                        for (const fake of [other, Object.create(receiver), new Proxy(receiver, {})]) {
                            if (!failType(() => get.call(fake))) return 'bad content getter brand';
                        }
                    }
                }
                // Expose the Frame Document first. Expose the Iframe WindowProxy
                // through indexed Window first, before calling either Iframe getter.
                const frameDoc = frame.contentDocument;
                const frameWindow = frame.contentWindow;
                const iframeWindow = window[1];
                const iframeDoc = iframe.contentWindow.document;
                if (frameDoc?.body.id !== 'nativeFrameChildBody' ||
                    iframeDoc?.body.id !== 'nativeIframeChildBody' ||
                    frameDoc === iframeDoc || frameWindow === iframeWindow ||
                    frameWindow !== window[0] || frameWindow.document !== frameDoc ||
                    frameDoc.defaultView !== frameWindow || iframeDoc.defaultView !== iframeWindow ||
                    iframe.contentWindow !== iframeWindow || iframe.contentDocument !== iframeDoc ||
                    Object.getPrototypeOf(frameDoc) !== frameWindow.HTMLDocument.prototype ||
                    Object.getPrototypeOf(iframeDoc) !== iframeWindow.HTMLDocument.prototype ||
                    frameWindow.HTMLDocument.prototype === iframeWindow.HTMLDocument.prototype)
                    return 'child content must reuse the exact native Document and WindowProxy';
                const borrowedFrame = Object.getOwnPropertyDescriptor(
                    frameWindow.HTMLFrameElement.prototype, 'contentDocument'
                ).get;
                const borrowedIframe = Object.getOwnPropertyDescriptor(
                    iframeWindow.HTMLIFrameElement.prototype, 'contentWindow'
                ).get;
                if (borrowedFrame.call(frame) !== frameDoc || borrowedIframe.call(iframe) !== iframeWindow)
                    return 'borrowed same-brand accessor must use the actual receiving container';
                if (iframe.getSVGDocument() !== null ||
                    !failType(() => HTMLIFrameElement.prototype.getSVGDocument.call(frame)))
                    return 'real initial HTML is not a navigated SVG Document';
                globalThis.frameContentProbe = {frame, iframe, frameDoc, iframeDoc, frameWindow, iframeWindow};
                return 'ok';
            })()"#,
        ),
        "ok",
    );

    edit(&mut host, DomSourceEdit::move_before(frame, shadow, None));
    edit(&mut host, DomSourceEdit::move_before(iframe, shadow, None));
    let shadowed = host.snapshot().unwrap();
    for (container, child) in [(frame, frame_child), (iframe, iframe_child)] {
        assert_eq!(shadowed.node(container).unwrap().parent(), Some(shadow));
        assert_eq!(content_navigable(&shadowed, container), Some(child.0));
        let nav = shadowed.hosted_navigable(child.0).unwrap();
        assert_eq!(nav.browsing_context(), child.2);
        assert_eq!(nav.global(), child.3);
        assert!(
            shadowed
                .document(child.1)
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
    }
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const {frame, iframe, frameDoc, iframeDoc, frameWindow, iframeWindow} = frameContentProbe;
                if (window.length !== 0 || frameWindow.closed || iframeWindow.closed ||
                    frame.contentDocument !== frameDoc || frame.contentWindow !== frameWindow ||
                    iframe.contentDocument !== iframeDoc || iframe.contentWindow !== iframeWindow)
                    return 'direct content access failed inside a connected shadow root';
                return 'ok';
            })()"#,
        ),
        "ok",
    );

    let accounting = host.guest_accounting().unwrap();
    let backing = || {
        accounting.snapshot().unwrap().unwrap().scopes[0]
            .usage
            .live(DocumentGuestResource::BackingBytes)
    };
    let before_listeners = backing();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const {frameWindow, iframeWindow} = frameContentProbe;
                const listener = () => {};
                frameWindow.addEventListener('frame-content-retirement', listener);
                iframeWindow.addEventListener('iframe-content-retirement', listener);
                return 'ok';
            })()"#,
        ),
        "ok",
    );
    assert!(backing() > before_listeners);
    edit(&mut host, DomSourceEdit::detach(frame));
    edit(&mut host, DomSourceEdit::detach(iframe));
    let removing = host.snapshot().unwrap();
    for (container, child) in [(frame, frame_child), (iframe, iframe_child)] {
        assert_eq!(content_navigable(&removing, container), None);
        assert!(removing.hosted_navigable(child.0).unwrap().is_retiring());
        assert_eq!(
            removing.document(child.1).unwrap().navigable(),
            Some(child.0)
        );
    }
    assert_eq!(backing(), before_listeners);
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const {frame, iframe, frameDoc, iframeDoc, frameWindow, iframeWindow} = frameContentProbe;
                if (frame.contentDocument !== null || frame.contentWindow !== null ||
                    iframe.contentDocument !== null || iframe.contentWindow !== null ||
                    !frameWindow.closed || !iframeWindow.closed ||
                    frameDoc.defaultView !== frameWindow || iframeDoc.defaultView !== iframeWindow)
                    return 'a cleared element slot must coexist with the original retiring proxy';
                return 'ok';
            })()"#,
        ),
        "ok",
    );

    host.with_native(|dom, mut hosted, _| {
        for _ in 0..128 {
            let snapshot = dom.snapshot();
            if [frame_child, iframe_child]
                .iter()
                .all(|child| snapshot.document(child.1).unwrap().navigable().is_none())
            {
                return;
            }
            hosted.run_next(dom, top.navigable(), &cancel()).unwrap();
        }
        panic!("native child retirement did not clear both Document associations");
    })
    .unwrap();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const {frame, iframe, frameDoc, iframeDoc, frameWindow, iframeWindow} = frameContentProbe;
                if (frame.contentDocument !== null || frame.contentWindow !== null ||
                    iframe.contentDocument !== null || iframe.contentWindow !== null ||
                    frameDoc.defaultView !== null || iframeDoc.defaultView !== null ||
                    !frameWindow.closed || !iframeWindow.closed)
                    return 'completed retirement did not disassociate both child Documents';
                return 'ok';
            })()"#,
        ),
        "ok",
    );

    edit(&mut host, DomSourceEdit::insert_before(frame, group, None));
    let reinserted = host.snapshot().unwrap();
    let new_id = content_navigable(&reinserted, frame).unwrap();
    let new_child = reinserted.hosted_navigable(new_id).unwrap();
    assert_ne!(new_id, frame_child.0);
    assert_ne!(new_child.active_document(), Some(frame_child.1));
    assert_ne!(new_child.browsing_context(), frame_child.2);
    assert_ne!(new_child.global(), frame_child.3);
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const {frame, frameDoc, frameWindow} = frameContentProbe;
                const nextWindow = frame.contentWindow;
                const nextDoc = frame.contentDocument;
                if (nextWindow === frameWindow || nextDoc === frameDoc || nextWindow !== window[0] ||
                    nextWindow.document !== nextDoc || nextDoc.defaultView !== nextWindow ||
                    nextWindow.closed || !frameWindow.closed)
                    return 'reinsertion must expose the new native child';
                return 'ok';
            })()"#,
        ),
        "ok",
    );

    host.checkpoint().unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(iframe);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let refusal = native_failure(host.in_document(&permit, |scope| {
        let source = "frameContentProbe.iframe.contentDocument";
        scope
            .evaluate(&source.encode_utf16().collect::<Vec<_>>())
            .map(drop)
    }));
    assert!(matches!(
        refusal.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}

pub(super) fn sandboxed_iframe_content_distinguishes_null_from_unavailable_proxy() {
    let (mut session, top) = setup();
    let document = top.document();
    let body = document_body(&session.snapshot(), document);
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(document, None);
    let iframe = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    batch.set_attribute(&iframe, "id", "opaqueContentIframe");
    batch.set_attribute(&iframe, "sandbox", "");
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let iframe = commit.created(&iframe).unwrap().unwrap();
    let mut host = attach(session, top);
    let permit = host.permit().unwrap();
    edit(&mut host, DomSourceEdit::insert_before(iframe, body, None));
    let snapshot = host.snapshot().unwrap();
    let child = content_navigable(&snapshot, iframe).unwrap();
    let nav = snapshot.hosted_navigable(child).unwrap();
    assert_eq!(nav.container(), Some(iframe));
    let child_doc = snapshot.document(nav.active_document().unwrap()).unwrap();
    assert!(child_doc.is_fully_active().unwrap());
    assert_ne!(
        child_doc.metadata().origin(),
        snapshot.document(document).unwrap().metadata().origin()
    );
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const iframe = document.getElementById('opaqueContentIframe');
                if (typeof Object.getOwnPropertyDescriptor(HTMLIFrameElement.prototype, 'contentDocument')?.get !== 'function')
                    return 'missing HTMLIFrameElement.prototype.contentDocument getter';
                return iframe.contentDocument === null ? 'ok' : 'opaque contentDocument must be null';
            })()"#,
        ),
        "ok",
    );

    // #3145 owns authentic foreign WindowProxy exposure. A present opaque child
    // must reach an explicit host refusal in the current profile.
    let refusal = native_failure(host.in_document(&permit, |scope| {
        let source = "document.getElementById('opaqueContentIframe').contentWindow";
        scope
            .evaluate(&source.encode_utf16().collect::<Vec<_>>())
            .map(drop)
    }));
    assert!(matches!(
        refusal.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::CreationGlobalUnavailable)
        ))
    ));
}
