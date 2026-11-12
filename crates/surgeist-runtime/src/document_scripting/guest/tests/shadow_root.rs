//! Canonical ShadowRoot projection through native V8 callbacks.

use super::*;
use surgeist_dom::{
    DocumentId, DomBatch, DomShadowRootInit, HtmlShadowRootMode, HtmlShadowRootProperties,
    HtmlSlotAssignment,
};
use surgeist_html::values::Namespace;

fn guest_document(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    script: &str,
) -> DocumentId {
    let before = host.snapshot().unwrap();
    host.in_document(permit, |scope| scope.evaluate(&units(script)).map(drop))
        .unwrap();
    let snapshot = host.snapshot().unwrap();
    snapshot
        .nodes()
        .filter(|node| before.node(*node).is_err())
        .find_map(|node| {
            snapshot
                .node(node)
                .unwrap()
                .as_document()
                .map(|doc| doc.id())
        })
        .unwrap()
}

fn open_root_host() -> (DocumentScriptHost, DocumentScriptPermit) {
    let mut host = public_node_host(1_000, 20_000);
    let permit = host.permit().unwrap();
    let cancel = host.native.cancellation.clone();
    let document = guest_document(
        &mut host,
        &permit,
        "globalThis.shadowDocument = new Document()",
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        let element = batch.create_element_ns(document, Namespace::html(), "div");
        batch.insert_before(&element, document, None);
        batch.attach_shadow(&element, DomShadowRootInit::new(HtmlShadowRootMode::Open));
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "globalThis.openShadowHost = shadowDocument.documentElement",
            ))
            .map(drop)
    })
    .unwrap();
    (host, permit)
}

pub(in crate::document_scripting) fn native_open_shadow_root_uses_the_canonical_interface() {
    let (mut host, permit) = open_root_host();
    host.in_document(&permit, |scope| {
        let root = scope.evaluate(&units("openShadowHost.shadowRoot"))?;
        assert_eq!(scope.kind(&root)?, surgeist_bindings_js::ValueKind::Object);
        assert_guest(
            scope,
            r#"(() => {
                const host = shadowDocument.documentElement;
                const root = host.shadowRoot;
                const getter = Object.getOwnPropertyDescriptor(ShadowRoot.prototype, "host").get;
                let rejected = false;
                try { getter.call(new DocumentFragment()); }
                catch (error) { rejected = error instanceof TypeError; }
                let illegal = false;
                try { new ShadowRoot(); }
                catch (error) { illegal = error instanceof TypeError; }
                return rejected && illegal && root != null && typeof ShadowRoot === "function"
                    && root === host.shadowRoot && root.host === host && root.mode === "open"
                    && Object.getPrototypeOf(root) === ShadowRoot.prototype
                    && Object.getPrototypeOf(ShadowRoot) === DocumentFragment
                    && Object.getPrototypeOf(ShadowRoot.prototype) === DocumentFragment.prototype
                    && root instanceof DocumentFragment && root instanceof Node
                    && !root.delegatesFocus && !root.clonable && !root.serializable
                    && root.slotAssignment === "named"
                    && root.nodeType === 11 && root.getRootNode() === root
                    && root.getRootNode({composed: true}) === shadowDocument;
            })()"#,
        )
    })
    .unwrap();
}

pub(in crate::document_scripting) fn closed_root_keeps_identity_and_native_event_visibility() {
    let mut host = public_node_host(1_000, 20_000);
    let permit = host.permit().unwrap();
    let cancel = host.native.cancellation.clone();
    let original = guest_document(&mut host, &permit, "globalThis.original = new Document()");
    let destination = guest_document(
        &mut host,
        &permit,
        "globalThis.destination = new Document()",
    );
    let (element, root, child) = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let element = batch.create_element_ns(original, Namespace::html(), "div");
            let root = batch.attach_shadow(
                &element,
                HtmlShadowRootProperties::new(
                    HtmlShadowRootMode::Closed,
                    HtmlSlotAssignment::Manual,
                    true,
                    true,
                    true,
                ),
            );
            let child = batch.create_element_ns(destination, Namespace::html(), "span");
            batch.insert_before(&element, original, None);
            batch.insert_before(&child, destination, None);
            let commit = dom.apply(batch, &cancel).unwrap();
            (
                commit.created(&element).unwrap().unwrap(),
                commit.created(&root).unwrap().unwrap(),
                commit.created(&child).unwrap().unwrap(),
            )
        })
        .unwrap();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "globalThis.closedHost = original.documentElement; \
                 globalThis.shadowChild = destination.documentElement",
            ))
            .map(drop)
    })
    .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(child, original);
        batch.insert_before(child, root, None);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            r#"(() => {
                const root = globalThis.closedRoot = shadowChild.getRootNode();
                let inside = false;
                let outside = false;
                root.addEventListener("boundary", event => {
                    const path = event.composedPath();
                    inside = event.target === shadowChild && event.currentTarget === root
                        && path[0] === shadowChild && path.includes(root);
                }, { once: true });
                closedHost.addEventListener("boundary", event => {
                    const path = event.composedPath();
                    outside = event.target === closedHost && event.currentTarget === closedHost
                        && path[0] === closedHost && !path.includes(root) && !path.includes(shadowChild);
                }, { once: true });
                return closedHost.shadowRoot === null && root instanceof ShadowRoot
                    && root.mode === "closed" && root.host === closedHost
                    && root.clonable && root.serializable && root.delegatesFocus
                    && root.slotAssignment === "manual" && shadowChild.parentNode === root
                    && root.getRootNode({composed: true}) === original
                    && shadowChild.dispatchEvent(new Event("boundary", { bubbles: true, composed: true }))
                    && inside && outside;
            })()"#,
        )
    })
    .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(element, destination);
        batch.insert_before(element, destination, None);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "closedHost === destination.documentElement && closedHost.shadowRoot === null \
             && closedRoot === shadowChild.getRootNode() && closedRoot.host === closedHost \
             && closedRoot.ownerDocument === destination \
             && shadowChild.getRootNode({composed: true}) === destination",
        )
    })
    .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.detach(element);
        batch.destroy(element);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = host
        .in_document(&permit, |scope| {
            scope.evaluate(&units("closedRoot.mode")).map(drop)
        })
    else {
        panic!("the original ShadowRoot wrapper must observe retirement");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}

pub(in crate::document_scripting) fn shadow_projection_admits_exact_work_and_backing() {
    assert_native_projection_admission(open_root_host, "openShadowHost.shadowRoot");
}
