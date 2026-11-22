//! Window properties observed over the original attached Document and child contexts.

use super::{
    window_length::{content_navigable, edit, observe},
    *,
};

pub(super) fn named_window_elements_follow_the_live_document() {
    let (session, top) = setup();
    let document = top.document();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let named_key = DocumentString::from_utf16([0x57, 0xd800, 0, 0xdc00]);
    let (element, second_duplicate) = host
        .with_native(|dom, _, _| {
            let snapshot = dom.snapshot();
            let html = snapshot.node(document.node()).unwrap().children()[0];
            let body = snapshot.node(html).unwrap().children()[1];
            let mut batch = DomBatch::new();
            let element = batch.create_element_ns(document, Namespace::html(), "div");
            let shadowed = batch.create_element_ns(document, Namespace::html(), "div");
            let first_duplicate = batch.create_element_ns(document, Namespace::html(), "span");
            let second_duplicate = batch.create_element_ns(document, Namespace::html(), "span");
            batch.set_attribute(&element, "id", named_key.clone());
            batch.set_attribute(&shadowed, "id", "addEventListener");
            batch.set_attribute(&first_duplicate, "id", "windowCollection");
            batch.set_attribute(&second_duplicate, "id", "windowCollection");
            batch.insert_before(&element, body, None);
            batch.insert_before(&shadowed, body, None);
            batch.insert_before(&first_duplicate, body, None);
            batch.insert_before(&second_duplicate, body, None);
            let commit = dom.apply(batch, &cancel()).unwrap();
            (
                commit.created(&element).unwrap().unwrap(),
                commit.created(&second_duplicate).unwrap().unwrap(),
            )
        })
        .unwrap();

    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const key = 'W\uD800\u0000\uDC00';
            const node = document.getElementById(key);
            if (node === null) return "fixture must contain the native element";
            if (window[key] !== node) return "Window must expose the exact live named element";
            let assignedReceiver;
            Object.defineProperty(window, 'windowAccessorReceiver', {
                get() { return this; },
                set(value) { assignedReceiver = [this, value]; },
                configurable: true
            });
            function readReceiver() { return windowAccessorReceiver; }
            if (readReceiver() !== window || readReceiver() !== window) return "global getter exposed the inner Window";
            windowAccessorReceiver = node;
            if (assignedReceiver?.[0] !== window || assignedReceiver?.[1] !== node) {
                return "global setter exposed the inner Window";
            }
            Reflect.deleteProperty(window, 'windowAccessorReceiver');
            const names = Object.getPrototypeOf(Window.prototype);
            const descriptor = Object.getOwnPropertyDescriptor(names, key);
            if (descriptor?.value !== node || !descriptor.writable || descriptor.enumerable
                || !descriptor.configurable || Reflect.ownKeys(names).includes(key)
                || Reflect.ownKeys(window).includes(key)) return "incorrect named descriptor or own keys";
            if (window.addEventListener !== EventTarget.prototype.addEventListener
                || Object.getOwnPropertyDescriptor(names, 'addEventListener') !== undefined) {
                return "the inherited EventTarget operation must mask the element name";
            }
            if (Reflect.defineProperty(names, key, {value: 1}) || Reflect.deleteProperty(names, key)
                || Reflect.set(names, key, 1)) return "named properties object must refuse own changes";
            const marker = {};
            if (!Reflect.set(window, key, marker) || window[key] !== marker
                || !Object.hasOwn(window, key) || !Reflect.deleteProperty(window, key)
                || window[key] !== node) return "an ordinary Window own property must shadow the name";
            const collection = window.windowCollection;
            if (!(collection instanceof HTMLCollection) || collection.length !== 2
                || collection[0] !== document.getElementById('windowCollection')
                || collection[0] === collection[1]) return "incorrect collection projection";
            globalThis.originalWindowCollection = collection;
            globalThis.originalNamedElement = node;
            return "ok";
        })()"#,
    );
    assert_eq!(result, "ok");

    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute(element, "id", "renamedWindowElement");
        batch.detach(second_duplicate);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let renamed = observe(
        &mut host,
        &permit,
        r#"(() => window['W\uD800\u0000\uDC00'] === undefined
            && window.renamedWindowElement === originalNamedElement
            && originalWindowCollection.length === 1
            && window.windowCollection === originalWindowCollection[0]
            ? 'ok' : 'stale Window name or collection')()"#,
    );
    assert_eq!(renamed, "ok");
}

pub(super) fn indexed_window_children_follow_the_live_browsing_contexts() {
    let (mut session, top) = setup();
    let document = top.document();
    let snapshot = session.snapshot();
    let html = snapshot.node(document.node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];

    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(document, None);
    let root = batch.create_element_ns(&detached, Namespace::html(), "div");
    let first = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    let second = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    batch.set_attribute(&first, "name", "firstNativeChild");
    batch.set_attribute(&second, "name", "secondNativeChild");
    batch.insert_before(&first, &root, None);
    batch.insert_before(&second, &root, None);
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let first = commit.created(&first).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    // The attachment also funds one native interface graph per child realm.
    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 160_000),
        surgeist_ledger::ProfileMode::Summary { timing: false },
        DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    edit(&mut host, DomSourceEdit::insert_before(root, body, None));
    let snapshot = host.snapshot().unwrap();

    let [first_child, second_child] = [first, second].map(|container| {
        let child = content_navigable(&snapshot, container).unwrap();
        let navigable = snapshot.hosted_navigable(child).unwrap();
        assert_eq!(navigable.parent(), Some(top.navigable()));
        assert_eq!(navigable.container(), Some(container));
        assert!(navigable.active_document().is_some());
        child
    });
    assert_ne!(first_child, second_child);

    let indexed = observe(
        &mut host,
        &permit,
        r#"(() => {
            if (window.length !== 2) return "fixture must contain two live child navigables";
            const first = window[0];
            const second = window[1];
            if (first === undefined || second === undefined) {
                return "Window must expose both live indexed child contexts";
            }
            if (first === second || first !== window[0] || second !== window[1]
                || first !== window.firstNativeChild || second !== window.secondNativeChild) {
                return "indices and names must share stable child WindowProxy identities";
            }
            if (first.window !== first || first.self !== first || first.parent !== window
                || first.closed !== false) return "child proxy must expose its live Window relations";
            const descriptor = Reflect.getOwnPropertyDescriptor(window, '0');
            if (descriptor?.value !== first || descriptor.writable || !descriptor.enumerable
                || !descriptor.configurable || Reflect.ownKeys(window).slice(0, 2).join(',') !== '0,1') {
                return "the indexed descriptor and key order must reflect the native list";
            }
            if (window[2] !== undefined || Reflect.getOwnPropertyDescriptor(window, '2') !== undefined
                || Reflect.defineProperty(window, '0', {value: 1}) || Reflect.set(window, '2', 1)
                || Reflect.deleteProperty(window, '0') || !Reflect.deleteProperty(window, '2')
                || Reflect.preventExtensions(window) || !Object.isExtensible(window)) {
                return "indices must preserve exact supported and unsupported reflection";
            }
            globalThis.originalFirstChild = first;
            globalThis.originalSecondChild = second;
            globalThis.originalFirstPrototype = Object.getPrototypeOf(first);
            return "ok";
        })()"#,
    );
    assert_eq!(indexed, "ok");

    let reentrant = observe(
        &mut host,
        &permit,
        r#"(() => {
            let evidence = "connected callback did not run";
            class WindowPropertyObserver extends HTMLElement {
                connectedCallback() {
                    const own = Reflect.ownKeys(window);
                    const first = window[0];
                    evidence = own[0] === '0' && own[1] === '1'
                        && first === originalFirstChild && window.firstNativeChild === first
                        && first.closed === false ? 'ok' : 'incorrect Window read during Source';
                }
            }
            customElements.define('x-window-property-observer', WindowPropertyObserver);
            document.body.appendChild(document.createElement('x-window-property-observer'));
            return evidence;
        })()"#,
    );
    assert_eq!(reentrant, "ok");

    let (inner, child_body) = host
        .with_native(|dom, _, _| {
            let snapshot = dom.snapshot();
            let parent = snapshot.hosted_navigable(first_child).unwrap();
            let document = parent.active_document().unwrap();
            let html = snapshot.node(document.node()).unwrap().children()[0];
            let child_body = snapshot.node(html).unwrap().children()[1];
            let mut batch = DomBatch::new();
            let detached = batch.create_html_document(document, None);
            let inner = batch.create_element_ns(&detached, Namespace::html(), "iframe");
            batch.set_attribute(&inner, "name", "nestedNativeChild");
            let subject = batch.create_element_ns(document, Namespace::html(), "span");
            batch.set_attribute(&subject, "id", "nestedWindowElement");
            batch.insert_before(&subject, child_body, None);
            let shadowed = batch.create_element_ns(document, Namespace::html(), "div");
            batch.set_attribute(&shadowed, "id", "document");
            batch.insert_before(&shadowed, child_body, None);
            let commit = dom.apply(batch, &cancel()).unwrap();
            let subject = commit.created(&subject).unwrap().unwrap();
            assert_eq!(
                dom.snapshot()
                    .node(subject)
                    .unwrap()
                    .relevant_global()
                    .unwrap(),
                Some(parent.global())
            );
            (commit.created(&inner).unwrap().unwrap(), child_body)
        })
        .unwrap();
    edit(
        &mut host,
        DomSourceEdit::insert_before(inner, child_body, None),
    );
    let nested_native = content_navigable(&host.snapshot().unwrap(), inner).unwrap();
    assert_ne!(nested_native, first_child);

    let nested = observe(
        &mut host,
        &permit,
        r#"(() => {
            const parent = originalFirstChild;
            if (parent.length !== 1) return "fixture must have a live nested navigable";
            const child = parent[0];
            if (child === undefined || child !== parent.nestedNativeChild || child.parent !== parent
                || child === window[0] || Reflect.ownKeys(parent)[0] !== '0') {
                return "nested native context must have its own stable WindowProxy";
            }
            const element = parent.nestedWindowElement;
            if (element === undefined || element.id !== 'nestedWindowElement'
                || !(element instanceof parent.HTMLElement) || element instanceof HTMLElement) {
                return "nested native element must use the child creation realm";
            }
            const descriptor = Object.getOwnPropertyDescriptor(parent, 'document');
            const childDocument = parent.document;
            if (typeof descriptor?.get !== 'function' || descriptor.set !== undefined
                || descriptor.configurable || !descriptor.enumerable
                || childDocument !== element.ownerDocument
                || !(childDocument instanceof parent.Document) || childDocument === document) {
                return "child Window.document must expose its exact native Document";
            }
            const names = Object.getPrototypeOf(Object.getPrototypeOf(parent));
            if (parent.document.getElementById('document')?.id !== 'document'
                || Object.getOwnPropertyDescriptor(names, 'document') !== undefined) {
                return "the child Window document attribute must mask the colliding name";
            }
            if (Object.getOwnPropertyDescriptor(window, 'document').get.call(parent) !== childDocument
                || descriptor.get.call(window) !== document
                || Reflect.deleteProperty(parent, 'document')) {
                return "the unforgeable getter must use the actual Window receiver";
            }
            globalThis.originalNestedChild = child;
            return "ok";
        })()"#,
    );
    assert_eq!(nested, "ok");

    let receivers = observe(
        &mut host,
        &permit,
        r#"(() => {
            const child = originalFirstChild;
            const parentLength = Object.getOwnPropertyDescriptor(window, 'length').get;
            const childLength = Object.getOwnPropertyDescriptor(child, 'length').get;
            const childSelf = Object.getOwnPropertyDescriptor(child, 'self');
            try {
                if (parentLength.call(child) !== 1 || childLength.call(window) !== 2
                    || Reflect.get(child, 'length', originalSecondChild) !== 0
                    || Object.getOwnPropertyDescriptor(window, 'window').get.call(child) !== child) {
                    return "Window accessors must observe the supplied Window receiver";
                }
                const replacement = {};
                if (!Reflect.set(window, 'self', replacement, child)
                    || child.self !== replacement || window.self !== window) {
                    return "a replaceable member must be written on the supplied Window";
                }
                Object.defineProperty(child, 'self', childSelf);
            } catch (_) {
                return "Window accessors rejected an authentic alternate receiver";
            }
            try {
                childLength.call(Object.create(child));
                return "Window accessors accepted an unrelated receiver";
            } catch (error) {
                if (error.name !== "TypeError") return "incorrect Window brand error";
            }
            return "ok";
        })()"#,
    );
    let event_target = observe(
        &mut host,
        &permit,
        r#"(() => {
            const child = originalFirstChild;
            let calls = 0;
            function observe(event) {
                if (this === child && event.currentTarget === child && event.target === child) {
                    ++calls;
                }
            }
            try {
                child.addEventListener('window-receiver', observe);
                if (!EventTarget.prototype.dispatchEvent.call(child, new Event('window-receiver'))
                    || calls !== 1) return "borrowed dispatch did not use the native child target";
                EventTarget.prototype.removeEventListener.call(child, 'window-receiver', observe);
                if (!child.dispatchEvent(new Event('window-receiver')) || calls !== 1) {
                    return "native child listener removal did not settle";
                }
                EventTarget.prototype.addEventListener.call(child, 'window-receiver', observe);
                if (!child.dispatchEvent(new Event('window-receiver')) || calls !== 2) {
                    return "borrowed registration did not reach native child dispatch";
                }
                child.removeEventListener('window-receiver', observe);
            } catch (_) {
                return "inherited EventTarget operations rejected the authentic child";
            }
            return "ok";
        })()"#,
    );
    assert_eq!((receivers.as_str(), event_target.as_str()), ("ok", "ok"));

    let shadow = host
        .with_native(|dom, _, _| {
            use surgeist_dom::shadow::DomShadowRootInit;
            use surgeist_html::construction::HtmlShadowRootMode;

            let mut batch = DomBatch::new();
            let shadow_host = batch.create_element_ns(document, Namespace::html(), "div");
            batch.insert_before(&shadow_host, body, None);
            let shadow = batch.attach_shadow(
                &shadow_host,
                DomShadowRootInit::new(HtmlShadowRootMode::Open),
            );
            dom.apply(batch, &cancel())
                .unwrap()
                .attached_shadow_root(&shadow)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    edit(&mut host, DomSourceEdit::move_before(first, shadow, None));
    let snapshot = host.snapshot().unwrap();
    assert_eq!(snapshot.node(first).unwrap().parent(), Some(shadow));
    assert_eq!(content_navigable(&snapshot, first), Some(first_child));
    assert!(
        !snapshot
            .hosted_navigable(first_child)
            .unwrap()
            .is_retiring()
    );
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const child = originalFirstChild;
                if (window.length !== 1 || window[0] !== originalSecondChild
                    || window.firstNativeChild !== undefined) {
                    return 'ordinary Window lookup exposed a shadow-connected child';
                }
                if (child.closed || child.parent !== window || child.length !== 1
                    || child[0] !== originalNestedChild || originalNestedChild.closed
                    || child.nestedWindowElement?.id !== 'nestedWindowElement') {
                    return 'an admitted shadow-connected Window remains live';
                }
                const listener = () => {};
                child.addEventListener('while-in-shadow-tree', listener);
                child.removeEventListener('while-in-shadow-tree', listener);
                return 'ok';
            })()"#,
        ),
        "ok"
    );
    edit(
        &mut host,
        DomSourceEdit::move_before(first, root, Some(second)),
    );
    assert_eq!(
        content_navigable(&host.snapshot().unwrap(), first),
        Some(first_child)
    );
    assert_eq!(
        observe(
            &mut host,
            &permit,
            "window[0] === originalFirstChild && window.firstNativeChild === originalFirstChild ? 'ok' : 'move replaced the original WindowProxy'",
        ),
        "ok"
    );

    let accounting = host.guest_accounting().unwrap();
    let live_backing = || {
        accounting.snapshot().unwrap().unwrap().scopes[0]
            .usage
            .live(DocumentGuestResource::BackingBytes)
    };
    let before_retirement = live_backing();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const listener = () => {};
                for (let index = 0; index < 16; index++) {
                    originalFirstChild.addEventListener('child-retirement-' + index, listener);
                }
                return 'ok';
            })()"#,
        ),
        "ok"
    );
    edit(&mut host, DomSourceEdit::detach(first));
    let snapshot = host.snapshot().unwrap();
    assert!(
        snapshot
            .hosted_navigable(first_child)
            .unwrap()
            .is_retiring()
    );
    assert_eq!(content_navigable(&snapshot, second), Some(second_child));
    let after_retirement = live_backing();
    assert!(
        after_retirement < before_retirement,
        "native removal must release guest Window registry backing before the next guest read ({before_retirement} -> {after_retirement})"
    );
    let retired = observe(
        &mut host,
        &permit,
        r#"(() => window.length === 1 && window[0] === originalSecondChild
            && window[0] !== originalFirstChild && window[1] === undefined
            && window.firstNativeChild === undefined && window.secondNativeChild === originalSecondChild
            && originalFirstChild.closed === true && originalSecondChild.closed === false
            && originalFirstChild.parent === null && originalFirstChild.length === 0
            && originalFirstChild[0] === undefined && originalFirstChild.nestedNativeChild === undefined
            && originalNestedChild.closed === true && originalNestedChild.parent === null
            && Object.getPrototypeOf(originalFirstChild) === originalFirstPrototype
            ? 'ok' : 'stale or replaced WindowProxy after child removal')()"#,
    );
    assert_eq!(retired, "ok");
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const listener = () => {};
                window.addEventListener('after-child-retirement', listener);
                window.removeEventListener('after-child-retirement', listener);
                return 'ok';
            })()"#,
        ),
        "ok"
    );

    edit(&mut host, DomSourceEdit::insert_before(first, body, None));
    let replacement = content_navigable(&host.snapshot().unwrap(), first).unwrap();
    assert_ne!(replacement, first_child);
    let reinserted = observe(
        &mut host,
        &permit,
        r#"(() => window.length === 2 && window[0] === originalSecondChild
            && window[1] === window.firstNativeChild && window[1] !== originalFirstChild
            && window[1].closed === false && originalFirstChild.closed === true
            ? 'ok' : 'reinsertion reused the retired proxy')()"#,
    );
    assert_eq!(reinserted, "ok");
}

pub(super) fn retired_child_keeps_the_creation_origin_of_native_nodes() {
    verify_child_creation_origin(FirstProjection::Window);
}

pub(super) fn live_child_nodes_can_be_projected_before_the_window() {
    verify_child_creation_origin(FirstProjection::LiveAdoptedNode);
}

pub(super) fn retired_child_nodes_can_be_projected_before_the_window() {
    verify_child_creation_origin(FirstProjection::RetiredAdoptedNode);
}

#[derive(Clone, Copy)]
enum FirstProjection {
    Window,
    LiveAdoptedNode,
    RetiredAdoptedNode,
}

fn verify_child_creation_origin(first: FirstProjection) {
    let (mut session, top) = setup();
    let document = top.document();
    let snapshot = session.snapshot();
    let html = snapshot.node(document.node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(document, None);
    let iframe = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let iframe = commit.created(&iframe).unwrap().unwrap();
    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 160_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    edit(&mut host, DomSourceEdit::insert_before(iframe, body, None));
    let snapshot = host.snapshot().unwrap();
    let child = content_navigable(&snapshot, iframe).unwrap();
    let child_state = snapshot.hosted_navigable(child).unwrap();
    let child_document = child_state.active_document().unwrap();
    let child_global = child_state.global();
    if matches!(first, FirstProjection::Window) {
        assert_eq!(
            observe(
                &mut host,
                &permit,
                r#"(() => {
                    const child = window[0];
                    child.HTMLDivElement.prototype.originWitness = 'original child';
                    return 'ok';
                })()"#,
            ),
            "ok"
        );
    }

    let place_child_node = |host: &mut DocumentScriptHost, name: &str| {
        host.with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let node = batch.create_element_ns(child_document, Namespace::html(), "div");
            batch.set_attribute(&node, "id", name);
            batch.adopt(&node, document.node());
            batch.insert_before(&node, body, None);
            let commit = dom.apply(batch, &cancel()).unwrap();
            let node = commit.created(&node).unwrap().unwrap();
            let snapshot = dom.snapshot();
            assert_eq!(snapshot.relevant_global(node).unwrap(), Some(child_global));
            assert_eq!(
                snapshot.node(node).unwrap().owner_document(),
                Some(document)
            );
        })
        .unwrap();
    };
    place_child_node(&mut host, "adoptedBeforeRetirement");
    if matches!(first, FirstProjection::LiveAdoptedNode) {
        assert_eq!(
            observe(
                &mut host,
                &permit,
                r#"(() => {
                    const adopted = window.adoptedBeforeRetirement;
                    const child = window[0];
                    const prototype = Object.getPrototypeOf(adopted);
                    if (adopted !== document.getElementById('adoptedBeforeRetirement')
                        || prototype !== child.HTMLDivElement.prototype
                        || prototype === HTMLDivElement.prototype) {
                        return 'the first native projection must use the original child realm';
                    }
                    prototype.originWitness = 'original child';
                    return 'ok';
                })()"#,
            ),
            "ok"
        );
    }
    edit(&mut host, DomSourceEdit::detach(iframe));
    host.with_native(|dom, mut hosted, _| {
        for _ in 0..32 {
            match dom.snapshot().hosted_navigable(child) {
                Ok(record) if record.is_retired() => return,
                Err(DomHostedError::StaleIdentity) => return,
                Ok(_) => (),
                Err(error) => panic!("child retirement failed: {error:?}"),
            }
            hosted.run_next(dom, top.navigable(), &cancel()).unwrap();
        }
        panic!("child retirement did not complete");
    })
    .unwrap();
    assert_eq!(
        host.snapshot()
            .unwrap()
            .global_document(child_global)
            .unwrap(),
        child_document
    );
    place_child_node(&mut host, "adoptedAfterRetirement");
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                if (window[0] !== undefined) return 'retired child still occupies an index';
                const before = window.adoptedBeforeRetirement;
                const after = window.adoptedAfterRetirement;
                if (before !== document.getElementById('adoptedBeforeRetirement')
                    || after !== document.getElementById('adoptedAfterRetirement')
                    || Object.getPrototypeOf(before) !== Object.getPrototypeOf(after)
                    || Object.getPrototypeOf(before) === HTMLDivElement.prototype) {
                    return 'child nodes lost their original creation prototype';
                }
                return 'ok';
            })()"#,
        ),
        "ok"
    );
    if !matches!(first, FirstProjection::RetiredAdoptedNode) {
        assert_eq!(
            observe(
                &mut host,
                &permit,
                r#"(() => window.adoptedBeforeRetirement.originWitness === 'original child'
                    && window.adoptedAfterRetirement.originWitness === 'original child'
                    ? 'ok' : 'the original mutated child prototype was lost')()"#,
            ),
            "ok"
        );
    }
}
