//! Canonical ShadowRoot projection through native V8 callbacks.

use super::*;

pub(in crate::document_scripting) fn native_open_shadow_root_uses_the_canonical_interface() {
    use surgeist_dom::{DomBatch, DomShadowRootInit, HtmlShadowRootMode};
    use surgeist_html::values::Namespace;

    let mut host = public_node_host(1_000, 20_000);
    let permit = host.permit().unwrap();
    let cancel = host.native.cancellation.clone();
    let before = host.snapshot().unwrap();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("globalThis.shadowDocument = new Document()"))
            .map(drop)
    })
    .unwrap();
    let snapshot = host.snapshot().unwrap();
    let document = snapshot
        .nodes()
        .filter(|node| before.node(*node).is_err())
        .filter_map(|node| snapshot.node(node).unwrap().as_document())
        .map(|document| document.id())
        .next()
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        let element = batch.create_element_ns(document, Namespace::html(), "div");
        batch.insert_before(&element, document, None);
        batch.attach_shadow(&element, DomShadowRootInit::new(HtmlShadowRootMode::Open));
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let root = scope.evaluate(&units("shadowDocument.documentElement.shadowRoot"))?;
        assert_eq!(scope.kind(&root)?, surgeist_bindings_js::ValueKind::Object);
        assert_guest(
            scope,
            r#"(() => {
                const host = shadowDocument.documentElement;
                const root = host.shadowRoot;
                return root != null && typeof ShadowRoot === "function"
                    && root === host.shadowRoot && root.host === host && root.mode === "open"
                    && Object.getPrototypeOf(root) === ShadowRoot.prototype
                    && Object.getPrototypeOf(ShadowRoot.prototype) === DocumentFragment.prototype
                    && root instanceof DocumentFragment && root instanceof Node
                    && root.nodeType === 11 && root.getRootNode() === root
                    && root.getRootNode({composed: true}) === shadowDocument;
            })()"#,
        )
    })
    .unwrap();
}
