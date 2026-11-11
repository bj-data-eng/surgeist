//! Element interfaces over a live canonical document root.

use super::*;
use surgeist_html::values::Namespace;

pub(super) fn document_projects_its_current_native_element() {
    let (mut host, top) = prepare_event_window();
    let permit = host.permit().unwrap();
    let empty = host
        .in_document(&permit, |scope| {
            let script = r#"(() => {
                globalThis.projectedDocument = new Document();
                const root = projectedDocument.documentElement;
                return root === null ? "null" : typeof root;
            })()"#;
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&empty).unwrap(), "null");

    let snapshot = host.snapshot().unwrap();
    let document = snapshot
        .nodes()
        .filter_map(|id| snapshot.node(id).unwrap().as_document())
        .map(|view| view.id())
        .find(|document| *document != top.document())
        .expect("canonical document created by the constructor");
    let native_root = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let preceding = batch.create_comment(document, "before the root");
            let root = batch.create_element_ns(document, Namespace::html(), "html");
            batch.insert_before(&preceding, document, None);
            batch.insert_before(&root, document, None);
            dom.apply(batch, &cancel())
                .unwrap()
                .created(&root)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    assert_eq!(
        host.snapshot()
            .unwrap()
            .document(document)
            .unwrap()
            .document_element(),
        Some(native_root)
    );
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const getter = Object.getOwnPropertyDescriptor(Document.prototype, "documentElement").get;
            let branded = false;
            try { Reflect.apply(getter, Object.create(Document.prototype), []); }
            catch (error) { branded = error instanceof TypeError; }
            const root = projectedDocument.documentElement;
            return branded && root === projectedDocument.documentElement
                && Object.getPrototypeOf(root) === HTMLHtmlElement.prototype
                && root instanceof HTMLElement && root instanceof Element
                && root instanceof Node && root instanceof EventTarget
                && root.ownerDocument === projectedDocument && root.parentNode === projectedDocument
                && root.nodeName === "html" && root.localName === "html" && root.tagName === "html"
                && root.namespaceURI === "http://www.w3.org/1999/xhtml" && root.prefix === null;
        })())"#,
        1.0,
    );
}
