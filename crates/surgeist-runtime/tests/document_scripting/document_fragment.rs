//! Native-backed DocumentFragment construction and Node projections.

use super::*;

pub(super) fn creates_fragments_through_the_canonical_document() {
    let (mut host, _) = prepare_event_window();
    let permit = host.permit().unwrap();
    let before = count_fragments(&host);
    let script = r##"(() => {
        if (typeof DocumentFragment !== "function") return "missing DocumentFragment interface";
        const current = window.document;
        const secondary = new Document();
        const factory = Document.prototype.createDocumentFragment;
        if (typeof factory !== "function") return "missing Document fragment factory";
        let wrongReceiver = false;
        try { Reflect.apply(factory, Object.create(Document.prototype), []); }
        catch (error) { wrongReceiver = error instanceof TypeError; }
        if (!wrongReceiver) return "Document fragment factory must qualify the receiver";
        const sentinel = {};
        const newTarget = new Proxy(function () {}, {
            get(target, key) {
                if (key === "prototype") throw sentinel;
                return Reflect.get(target, key);
            }
        });
        let originalThrow = false;
        try { Reflect.construct(DocumentFragment, [], newTarget); }
        catch (error) { originalThrow = error === sentinel; }
        if (!originalThrow) return "Fragment must preserve abrupt prototype lookup";
        const constructor = new DocumentFragment();
        class Derived extends DocumentFragment {}
        const derived = new Derived();
        const factoryFragment = secondary.createDocumentFragment();
        if (Object.getPrototypeOf(DocumentFragment) !== Node
            || Object.getPrototypeOf(DocumentFragment.prototype) !== Node.prototype
            || Object.getPrototypeOf(derived) !== Derived.prototype
            || new Set([constructor, derived, factoryFragment]).size !== 3) {
            return "Fragment must preserve interface inheritance and identity";
        }
        for (const [fragment, document] of [[constructor, current], [derived, current],
            [factoryFragment, secondary]]) {
            if (!(fragment instanceof DocumentFragment && fragment instanceof Node
                && fragment instanceof EventTarget)
                || fragment.nodeType !== 11 || fragment.nodeName !== "#document-fragment"
                || fragment.ownerDocument !== document || fragment.parentNode !== null
                || fragment.isConnected || fragment.getRootNode() !== fragment) {
                return "Fragment Node reads must reflect its original native creation";
            }
        }
        return "ok";
    })()"##;
    let result = host
        .in_document(&permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
    assert_eq!(count_fragments(&host), before + 3);
}

fn count_fragments(host: &DocumentScriptHost) -> usize {
    let snapshot = host.snapshot().unwrap();
    snapshot
        .nodes()
        .filter(|id| {
            matches!(
                snapshot.node(*id).unwrap().kind(),
                DomNodeKind::DocumentFragment
            )
        })
        .count()
}

pub(super) fn fragment_projection_observes_adoption_dispatch_and_native_retirement() {
    let (mut host, top) = prepare_event_window();
    let permit = host.permit().unwrap();
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            globalThis.original = new DocumentFragment();
            globalThis.destination = new Document();
            let observed = false;
            original.addEventListener("fragment", function (event) {
                const path = event.composedPath();
                observed = this === original && event.target === original
                    && event.currentTarget === original && path.length === 1
                    && path[0] === original;
            });
            return original.dispatchEvent(new Event("fragment")) && observed;
        })())"#,
        1.0,
    );
    let original = fragment_node(&host);
    let snapshot = host.snapshot().unwrap();
    let destination = snapshot
        .nodes()
        .filter_map(|id| snapshot.node(id).unwrap().as_document())
        .map(|document| document.id())
        .find(|document| *document != top.document())
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(original, destination);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    eval(
        &mut host,
        &permit,
        "Number(original.ownerDocument === destination && original.getRootNode() === original)",
        1.0,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        let native_document = batch.create_document();
        batch.adopt(original, &native_document);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const projected = original.ownerDocument;
            return projected instanceof Document && projected !== destination
                && projected !== window.document && projected === original.ownerDocument
                && projected.ownerDocument === null && projected.nodeType === 9
                && projected.getRootNode() === projected;
        })())"#,
        1.0,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(original);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert!(matches!(
        host.snapshot().unwrap().node(original),
        Err(DomError::StaleNode)
    ));
    eval(
        &mut host,
        &permit,
        "Number((new DocumentFragment()).ownerDocument === window.document)",
        1.0,
    );
    assert_ne!(fragment_node(&host), original);
    let failure = native_failure(host.in_document(&permit, |scope| {
        scope
            .evaluate(&"original.nodeName".encode_utf16().collect::<Vec<_>>())
            .map(drop)
    }));
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}

fn fragment_node(host: &DocumentScriptHost) -> NodeId {
    let snapshot = host.snapshot().unwrap();
    snapshot
        .nodes()
        .find(|id| {
            matches!(
                snapshot.node(*id).unwrap().kind(),
                DomNodeKind::DocumentFragment
            )
        })
        .unwrap()
}
