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
    let element = host
        .with_native(|dom, _, _| {
            let snapshot = dom.snapshot();
            let html = snapshot.node(document.node()).unwrap().children()[0];
            let body = snapshot.node(html).unwrap().children()[1];
            let mut batch = DomBatch::new();
            let element = batch.create_element_ns(document, Namespace::html(), "div");
            let shadowed = batch.create_element_ns(document, Namespace::html(), "div");
            batch.set_attribute(&element, "id", named_key.clone());
            batch.set_attribute(&shadowed, "id", "addEventListener");
            batch.insert_before(&element, body, None);
            batch.insert_before(&shadowed, body, None);
            let commit = dom.apply(batch, &cancel()).unwrap();
            commit.created(&element).unwrap().unwrap()
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
            globalThis.originalNamedElement = node;
            return "ok";
        })()"#,
    );
    assert_eq!(result, "ok");

    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute(element, "id", "renamedWindowElement");
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let renamed = observe(
        &mut host,
        &permit,
        r#"(() => window['W\uD800\u0000\uDC00'] === undefined
            && window.renamedWindowElement === originalNamedElement ? 'ok' : 'stale Window name')()"#,
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
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
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
            return "ok";
        })()"#,
    );
    assert_eq!(indexed, "ok");

    edit(&mut host, DomSourceEdit::detach(first));
    let snapshot = host.snapshot().unwrap();
    assert!(
        snapshot
            .hosted_navigable(first_child)
            .unwrap()
            .is_retiring()
    );
    assert_eq!(content_navigable(&snapshot, second), Some(second_child));
    let retired = observe(
        &mut host,
        &permit,
        r#"(() => window.length === 1 && window[0] === originalSecondChild
            && window[0] !== originalFirstChild && window[1] === undefined
            && window.firstNativeChild === undefined && window.secondNativeChild === originalSecondChild
            ? 'ok' : 'stale or replaced WindowProxy after child removal')()"#,
    );
    assert_eq!(retired, "ok");
}
