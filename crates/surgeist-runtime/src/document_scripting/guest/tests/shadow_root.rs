//! Canonical ShadowRoot projection through native V8 callbacks.

use super::*;
use surgeist_dom::{
    DocumentId, DomBatch, DomShadowRootInit, HtmlShadowRootMode, HtmlShadowRootProperties,
    HtmlSlotAssignment, NodeId,
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

fn root_host(
    mode: Option<HtmlShadowRootMode>,
) -> (DocumentScriptHost, DocumentScriptPermit, Option<NodeId>) {
    prepare_root_host(public_node_host(1_000, 20_000), mode)
}

fn prepare_root_host(
    mut host: DocumentScriptHost,
    mode: Option<HtmlShadowRootMode>,
) -> (DocumentScriptHost, DocumentScriptPermit, Option<NodeId>) {
    let permit = host.permit().unwrap();
    let cancel = host.native.cancellation.clone();
    let document = guest_document(
        &mut host,
        &permit,
        "globalThis.shadowDocument = new Document()",
    );
    let root = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let element = batch.create_element_ns(document, Namespace::html(), "div");
            batch.insert_before(&element, document, None);
            let root = mode.map(|mode| batch.attach_shadow(&element, DomShadowRootInit::new(mode)));
            let commit = dom.apply(batch, &cancel).unwrap();
            root.and_then(|root| commit.created(&root).unwrap())
        })
        .unwrap();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "globalThis.shadowHost = shadowDocument.documentElement",
            ))
            .map(drop)
    })
    .unwrap();
    (host, permit, root)
}

pub(in crate::document_scripting) fn native_open_shadow_root_uses_the_canonical_interface() {
    let (mut host, permit, _) = root_host(Some(HtmlShadowRootMode::Open));
    host.in_document(&permit, |scope| {
        let root = scope.evaluate(&units("shadowHost.shadowRoot"))?;
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
                    inside = globalThis.event === undefined
                        && event.target === shadowChild && event.currentTarget === root
                        && path[0] === shadowChild && path.includes(root);
                }, { once: true });
                closedHost.addEventListener("boundary", event => {
                    const path = event.composedPath();
                    outside = globalThis.event === event
                        && event.target === closedHost && event.currentTarget === closedHost
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
    assert_native_projection_admission(
        || {
            let (host, permit, _) = root_host(Some(HtmlShadowRootMode::Open));
            (host, permit)
        },
        "shadowHost.shadowRoot",
    );
}

pub(in crate::document_scripting) fn closed_root_visibility_admits_the_canonical_lookup() {
    let read = |scope: &mut Scope<'_>| assert_guest(scope, "shadowHost.shadowRoot === null");
    let rootless_work = {
        let (mut host, permit, _) = root_host(None);
        let events = host.guest_events().unwrap();
        let before = events.budget.work.get();
        host.in_document(&permit, read).unwrap();
        events.budget.work.get() - before
    };
    let (mut host, permit, root) = root_host(Some(HtmlShadowRootMode::Closed));
    let snapshot = host.snapshot().unwrap();
    let mut root_lookup_work = 0;
    snapshot
        .node_with(root.unwrap(), || {
            root_lookup_work += 1;
            Ok::<(), DomError>(())
        })
        .unwrap();
    assert!(root_lookup_work >= 2);
    let events = host.guest_events().unwrap();
    let bytes = events.budget.used.get();
    events
        .budget
        .work
        .set(events.limits.work - rootless_work - root_lookup_work + 1);
    let result = host.in_document(&permit, read);
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("closed-root lookup must refuse one unit short of the actual lookup: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::WorkLimit)
    ));
    assert_eq!(events.budget.used.get(), bytes);
    assert_eq!(host.snapshot().unwrap().stamp(), snapshot.stamp());
}

pub(in crate::document_scripting) fn scripted_attach_publishes_the_exact_native_shadow_root() {
    let (mut host, permit, _) = root_host(None);
    let before = host.snapshot().unwrap();
    host.in_document(&permit, |scope| {
        let method = scope.evaluate(&units("Element.prototype.attachShadow"))?;
        assert!(scope.is_callable(&method)?, "Element.attachShadow is missing");
        assert_guest(
            scope,
            r#"(() => {
                const descriptor = Object.getOwnPropertyDescriptor(Element.prototype, "attachShadow");
                const root = descriptor.value.call(shadowHost, {
                    mode: "open",
                    delegatesFocus: true,
                    slotAssignment: "manual",
                    clonable: true,
                    serializable: true,
                });
                globalThis.scriptedRoot = root;
                return descriptor.value.length === 1 && descriptor.value.name === "attachShadow"
                    && descriptor.writable && descriptor.enumerable && descriptor.configurable
                    && root === shadowHost.shadowRoot && root.host === shadowHost
                    && Object.getPrototypeOf(root) === ShadowRoot.prototype
                    && root.mode === "open" && root.delegatesFocus && root.clonable
                    && root.serializable && root.slotAssignment === "manual";
            })()"#,
        )
    })
    .unwrap();
    let after = host.snapshot().unwrap();
    assert_ne!(after.stamp(), before.stamp());
    let mut created = after.nodes().filter(|node| before.node(*node).is_err());
    let root = after.shadow_root(created.next().unwrap()).unwrap();
    assert!(created.next().is_none());
    assert_eq!(root.state().mode(), HtmlShadowRootMode::Open);
    assert_eq!(root.state().slot_assignment(), HtmlSlotAssignment::Manual);
    assert!(root.state().delegates_focus());
    assert!(root.state().clonable());
    assert!(root.state().serializable());
    assert_eq!(
        after.node(root.host()).unwrap().shadow_root(),
        Some(root.id())
    );
    let cancel = host.native.cancellation.clone();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.detach(root.host());
        batch.destroy(root.host());
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    assert!(matches!(
        host.snapshot().unwrap().node(root.id()),
        Err(DomError::StaleNode)
    ));
    let refused = host.in_document(&permit, |scope| {
        scope.evaluate(&units("scriptedRoot.mode")).map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = refused else {
        panic!("the scripted ShadowRoot must observe native retirement: {refused:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}

pub(in crate::document_scripting) fn scripted_shadow_conversion_preserves_order_and_identity() {
    let (mut host, permit, _) = root_host(None);
    let before = host.snapshot().unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(scope, r#"(() => {
                const attach = Element.prototype.attachShadow;
                let calls = 0;
                const forged = Object.create(Element.prototype);
                try {
                    attach.call(forged, { get mode() { ++calls; return "open"; } });
                    return false;
                } catch (error) {
                    if (!(error instanceof TypeError) || calls !== 0) return false;
                }
                for (const options of [undefined, null, {}, { mode: "Open" }]) {
                    try { attach.call(shadowHost, options); return false; }
                    catch (error) { if (!(error instanceof TypeError)) return false; }
                }
                try { attach.call(shadowHost); return false; }
                catch (error) { if (!(error instanceof TypeError)) return false; }
                const seen = [];
                const abort = {};
                try {
                    attach.call(shadowHost, {
                        get clonable() { seen.push("clonable"); return false; },
                        get customElementRegistry() { seen.push("registry"); return {}; },
                        get delegatesFocus() { ++calls; return false; },
                    });
                    return false;
                } catch (error) {
                    if (!(error instanceof TypeError) || seen.join() !== "clonable,registry" || calls !== 0)
                        return false;
                }
                try {
                    attach.call(shadowHost, {
                        get mode() { throw abort; },
                        get serializable() { ++calls; return true; },
                    });
                    return false;
                } catch (error) {
                    if (error !== abort || calls !== 0 || shadowHost.shadowRoot !== null) return false;
                }
                seen.length = 0;
                const closed = attach.call(shadowHost, {
                    get clonable() { seen.push("clonable"); return undefined; },
                    get customElementRegistry() { seen.push("registry"); return null; },
                    get delegatesFocus() { seen.push("focus"); return undefined; },
                    get mode() { seen.push("mode"); return "closed"; },
                    get serializable() { seen.push("serializable"); return undefined; },
                    get slotAssignment() { seen.push("slot"); return undefined; },
                });
                try { attach.call(shadowHost, { mode: "closed" }); return false; }
                catch (error) {
                    if (!(error instanceof DOMException) || error.name !== "NotSupportedError") return false;
                }
                return seen.join() === "clonable,registry,focus,mode,serializable,slot"
                    && closed.mode === "closed" && closed.host === shadowHost && shadowHost.shadowRoot === null
                    && closed.slotAssignment === "named" && !closed.delegatesFocus
                    && !closed.clonable && !closed.serializable;
            })()"#)
    }).unwrap();
    let after = host.snapshot().unwrap();
    let mut roots = after.nodes().filter(|node| before.node(*node).is_err());
    let closed = after.shadow_root(roots.next().unwrap()).unwrap();
    assert!(roots.next().is_none());
    assert_eq!(closed.state().mode(), HtmlShadowRootMode::Closed);
    assert_eq!(
        after.node(closed.host()).unwrap().hosted_shadow_root(),
        Some(closed.id())
    );
    let events = host.guest_events().unwrap();
    let bytes = events.budget.used.get();
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            r#"(() => {
            for (let attempt = 0; attempt < 20; ++attempt) {
                try { shadowHost.attachShadow({mode: "closed"}); return false; }
                catch (error) {
                    if (!(error instanceof DOMException) || error.name !== "NotSupportedError")
                        return false;
                }
            }
            return true;
        })()"#,
        )
    })
    .unwrap();
    assert_eq!(events.budget.used.get(), bytes);
    assert_eq!(host.snapshot().unwrap().stamp(), after.stamp());
}

pub(in crate::document_scripting) fn scripted_shadow_revalidates_after_reentry() {
    let (mut host, permit, _) = root_host(None);
    let before = host.snapshot().unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            r#"(() => {
                let inner;
                try {
                    shadowHost.attachShadow({get mode() {
                        inner = shadowHost.attachShadow({mode: "closed"});
                        return "open";
                    }});
                    return false;
                } catch (error) {
                    return error instanceof DOMException && error.name === "NotSupportedError"
                        && inner.mode === "closed" && inner.host === shadowHost
                        && shadowHost.shadowRoot === null;
                }
            })()"#,
        )
    })
    .unwrap();
    let after = host.snapshot().unwrap();
    assert_eq!(
        after
            .nodes()
            .filter(|node| before.node(*node).is_err())
            .count(),
        1
    );
}

pub(in crate::document_scripting) fn scripted_shadow_source_refusal_keeps_the_original_state() {
    let (mut host, permit, _) = prepare_root_host(public_node_host(1_000, 1), None);
    let before = host.snapshot().unwrap();
    let events = host.guest_events().unwrap();
    let bytes = events.budget.used.get();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("shadowHost.attachShadow({mode: 'open'})"))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("one Source work unit cannot publish a native shadow root: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(
                DomError::SourceWorkLimit { limit: 1 }
            ))
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    assert_eq!(events.budget.used.get(), bytes);
}

pub(in crate::document_scripting) fn scripted_shadow_selects_the_hosted_registry_and_clones_null() {
    use surgeist_dom::DomRegistryBinding;

    let mut host = public_node_host(1_000, 20_000);
    let permit = host.permit().unwrap();
    let cancel = host.native.cancellation.clone();
    let document = permit.document();
    let before = host.snapshot().unwrap();
    let initial = before.document(document).unwrap();
    let document_registry = initial.metadata().custom_element_registry();
    assert!(matches!(document_registry, DomRegistryBinding::Registry(_)));
    let old = initial.document_element().unwrap();
    let first = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let first = batch.create_element_ns(document, Namespace::html(), "div");
            batch.replace_child(&first, document, old);
            dom.apply(batch, &cancel)
                .unwrap()
                .created(&first)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            r#"(() => {
            const host = document.documentElement;
            return host.attachShadow({mode: "open", clonable: true, customElementRegistry: null})
                === host.shadowRoot;
        })()"#,
        )
    })
    .unwrap();
    let null_registry = |snapshot: &DomSnapshot, element| {
        let root = snapshot.node(element).unwrap().shadow_root().unwrap();
        let state = snapshot.shadow_root(root).unwrap().state();
        assert_eq!(state.custom_element_registry(), DomRegistryBinding::Null);
        assert!(!state.keep_custom_element_registry_null());
    };
    null_registry(&host.snapshot().unwrap(), first);
    let (copy, second) = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let copy = batch.clone_node(first, false);
            let second = batch.create_element_ns(document, Namespace::html(), "span");
            batch.replace_child(&second, document, first);
            let commit = dom.apply(batch, &cancel).unwrap();
            (
                commit.created(&copy).unwrap().unwrap(),
                commit.created(&second).unwrap().unwrap(),
            )
        })
        .unwrap();
    null_registry(&host.snapshot().unwrap(), copy);
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            r#"(() => {
            const host = document.documentElement;
            return host.attachShadow({mode: "open"}) === host.shadowRoot;
        })()"#,
        )
    })
    .unwrap();
    let after = host.snapshot().unwrap();
    let root = after.node(second).unwrap().shadow_root().unwrap();
    assert_eq!(
        after
            .shadow_root(root)
            .unwrap()
            .state()
            .custom_element_registry(),
        document_registry
    );
}
