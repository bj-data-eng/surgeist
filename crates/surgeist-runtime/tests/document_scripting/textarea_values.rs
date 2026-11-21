//! The attached Textarea keeps native raw, normalized and child Text distinct.

use super::*;

pub(super) fn attached_textarea_values_follow_canonical_text_and_dirty_state() {
    let (session, top) = setup();
    let cancellation = Arc::new(cancel());
    let (mut host, _) =
        attach_event_window_with_work(session, top, cancellation.clone(), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const prototype = HTMLTextAreaElement.prototype;
        const names = ['dirName', 'disabled', 'form', 'name', 'placeholder', 'readOnly', 'required',
            'wrap', 'type', 'defaultValue', 'value', 'textLength'];
        const readonly = ['form', 'type', 'textLength'];
        const callbacks = [];
        for (const name of names) {
            const d = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof d?.get !== 'function') return 'missing HTMLTextAreaElement.prototype.' + name + ' getter';
            if (!d.enumerable || !d.configurable || d.get.name !== 'get ' + name || d.get.length !== 0 ||
                (readonly.includes(name) ? d.set !== undefined :
                    typeof d.set !== 'function' || d.set.name !== 'set ' + name || d.set.length !== 1)) {
                return name + ' Textarea accessor descriptor';
            }
            callbacks.push(d.get);
            if (d.set) callbacks.push(d.set);
        }
        if (Object.getOwnPropertyNames(prototype).filter(name => names.includes(name)).join() !== names.join()) {
            return 'Textarea declaration order';
        }
        const html = 'http://www.w3.org/1999/xhtml';
        const native = document.createElement('textarea');
        const prefixed = document.createElementNS(html, 'h:textarea');
        const upper = document.createElementNS(html, 'TEXTAREA');
        const foreign = document.createElementNS('urn:foreign', 'textarea');
        Object.setPrototypeOf(foreign, prototype);
        if (Object.getPrototypeOf(native) !== prototype || Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
            !(prefixed instanceof HTMLTextAreaElement) || prefixed.form !== null ||
            prefixed.type !== 'textarea' || prefixed.textLength !== 0) return 'exact native Textarea brands';
        let conversions = 0;
        const argument = { [Symbol.toPrimitive]() { conversions++; return 'premature'; } };
        for (const callback of callbacks) {
            for (const receiver of [document.body, upper, foreign, Object.create(prototype), new Proxy(native, {})]) {
                try {
                    Reflect.apply(callback, receiver, [argument]);
                    return callback.name + ' accepted a non-Textarea receiver';
                } catch (error) {
                    if (!(error instanceof TypeError) || conversions !== 0) return callback.name + ' native brand first';
                }
            }
        }
        const exact = ' Raw\ud800\0\udc00 ';
        for (const [member, attribute] of [['dirName', 'dirname'], ['name', 'name'],
            ['placeholder', 'placeholder'], ['wrap', 'wrap']]) {
            prefixed.setAttributeNS('urn:foreign', 'f:' + attribute, 'foreign');
            prefixed.setAttributeNS(null, attribute.toUpperCase(), 'uppercase');
            if (prefixed[member] !== '') return member + ' ignores other attribute names';
            prefixed[member] = exact;
            if (prefixed[member] !== exact || prefixed.getAttributeNS(null, attribute) !== exact ||
                prefixed.getAttributeNS(null, attribute.toUpperCase()) !== 'uppercase' ||
                prefixed.getAttributeNS('urn:foreign', attribute) !== 'foreign') return member + ' exact DOMString';
        }
        for (const [member, attribute] of [['disabled', 'disabled'], ['readOnly', 'readonly'], ['required', 'required']]) {
            prefixed.setAttributeNS('urn:foreign', 'f:' + attribute, '');
            if (prefixed[member]) return member + ' ignores foreign presence';
            prefixed.setAttributeNS(null, attribute, 'false');
            if (!prefixed[member]) return member + ' reflects presence';
            prefixed[member] = argument;
            if (!prefixed[member] || conversions !== 0 || prefixed.getAttributeNS(null, attribute) !== '') {
                return member + ' Web IDL Boolean';
            }
            prefixed[member] = false;
            if (prefixed[member] || prefixed.getAttributeNS(null, attribute) !== null ||
                prefixed.getAttributeNS('urn:foreign', attribute) !== '') return member + ' removes only own presence';
        }
        prefixed.setAttributeNS(null, 'type', 'authored');
        if (prefixed.type !== 'textarea') return 'intrinsic Textarea type';
        native.defaultValue = null;
        if (native.value !== 'null' || native.defaultValue !== 'null') return 'defaultValue plain DOMString null';
        const original = native.firstChild;
        native.value = null;
        if (native.value !== '' || native.textLength !== 0 || native.defaultValue !== 'null' ||
            native.firstChild !== original) return 'only value uses legacy null to empty';
        native.value = undefined;
        return native.value === 'undefined' && native.textLength === 9 && native.defaultValue === 'null'
            ? 'ok' : 'value ordinary undefined conversion';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const first = document.body.appendChild(document.createElement('form'));
        const next = document.body.appendChild(document.createElement('form'));
        const ownerId = 'textarea-owner\ud800';
        next.setAttributeNS(null, 'id', ownerId);
        const textarea = first.appendChild(document.createElement('textarea'));
        const spare = first.appendChild(document.createElement('textarea'));
        textarea.name = 'live\ud800';
        const calls = [];
        customElements.define('x-textarea-default-probe', class extends HTMLElement {
            disconnectedCallback() {
                calls.push([this, this.parentNode, textarea.defaultValue, textarea.value, textarea.textLength, textarea.form]);
            }
        });
        const head = textarea.appendChild(document.createTextNode('A\r'));
        const nested = textarea.appendChild(document.createElement('span'));
        nested.appendChild(document.createTextNode('excluded nested'));
        textarea.appendChild(document.createComment('excluded comment'));
        const custom = textarea.appendChild(document.createElement('x-textarea-default-probe'));
        custom.appendChild(document.createTextNode('excluded custom text'));
        const tail = textarea.appendChild(document.createTextNode('\nB\r\ud800\0\udc00\u{1f642}'));
        spare.value = 'spare\r\n\udc00';
        if (textarea.form !== first || spare.form !== first || textarea.getAttributeNS(null, 'name') !== 'live\ud800' ||
            textarea.defaultValue !== 'A\r\nB\r\ud800\0\udc00\u{1f642}' ||
            textarea.value !== 'A\nB\n\ud800\0\udc00\u{1f642}' || textarea.textLength !== 9) {
            return 'direct child Text, raw newlines and API UTF-16 length';
        }
        globalThis.textareaProbe = {first, next, ownerId, textarea, spare, head, nested, custom, tail, calls};
        return 'ok';
    })()"#,
    );

    let before = host.snapshot().unwrap();
    let html = before
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = before.node(html).unwrap().children()[1];
    let [first, next]: [NodeId; 2] = before.node(body).unwrap().children().try_into().unwrap();
    let [node, spare]: [NodeId; 2] = before.node(first).unwrap().children().try_into().unwrap();
    let removed = before.node(node).unwrap().children();
    assert_eq!(removed.len(), 5);
    assert_eq!(textarea(&before, node).form_owner().unwrap(), Some(first));
    assert!(!textarea(&before, node).state().flags().dirty_value());
    let initial = units("A\r\nB\r", &[0xd800, 0, 0xdc00, 0xd83d, 0xde42]);
    assert_eq!(textarea(&before, node).state().raw_value(), &initial);
    assert!(textarea(&before, spare).state().flags().dirty_value());

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {textarea, head, nested, custom, tail, first, calls} = textareaProbe;
        textarea.defaultValue = null;
        const [event] = calls;
        return textarea.defaultValue === 'null' && textarea.value === 'null' && textarea.textLength === 4 &&
            textarea.firstChild === textarea.lastChild && textarea.firstChild.nodeType === Node.TEXT_NODE &&
            [head, nested, custom, tail].every(child => child.parentNode === null) && calls.length === 1 &&
            event[0] === custom && event[1] === null && event[2] === 'null' && event[3] === 'null' &&
            event[4] === 4 && event[5] === first
            ? 'ok' : 'clean default replacement and one real synchronous disconnection';
    })()"#,
    );

    let clean = host.snapshot().unwrap();
    assert_eq!(
        textarea(&clean, node).state().raw_value(),
        &DocumentString::from("null")
    );
    assert!(!textarea(&clean, node).state().flags().dirty_value());
    for &child in removed {
        assert_eq!(clean.node(child).unwrap().parent(), None);
        assert_eq!(before.node(child).unwrap().parent(), Some(node));
    }

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {textarea, spare, first, next, ownerId, calls} = textareaProbe;
        const setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
        let receiver = textarea;
        let newChild;
        let reassociated = false;
        const hints = [];
        const result = setter.call(receiver, { [Symbol.toPrimitive](hint) {
            hints.push(hint);
            receiver = spare;
            next.appendChild(textarea);
            reassociated = textarea.form === next && spare.form === first;
            textarea.setAttributeNS(null, 'form', ownerId);
            textarea.defaultValue = 'during\r\n\udc00';
            newChild = textarea.firstChild;
            return 'current\r\n\ud800\0\udc00\r\u{1f642}';
        } });
        const api = 'current\n\ud800\0\udc00\n\u{1f642}';
        if (result !== undefined || hints.join() !== 'string' || !reassociated || textarea.form !== next ||
            textarea.form !== textarea.form || textarea.firstChild !== newChild ||
            textarea.defaultValue !== 'during\r\n\udc00' || textarea.value !== api || textarea.textLength !== 14 ||
            spare.value !== 'spare\n\udc00' || calls.length !== 1) return 'the current setter keeps the original Textarea';
        textarea.defaultValue = 'tail\r\n\ud800\0\udc00\r';
        if (textarea.firstChild === newChild || textarea.value !== api || textarea.textLength !== 14 ||
            textarea.defaultValue !== 'tail\r\n\ud800\0\udc00\r') return 'dirty current value survives default replacement';
        const thrown = {};
        try {
            setter.call(textarea, {toString() { throw thrown; }});
            return 'Textarea accepted abrupt value conversion';
        } catch (error) {
            return error === thrown && textarea.value === api && calls.length === 1
                ? 'ok' : 'abrupt Textarea conversion altered native state';
        }
    })()"#,
    );

    let dirty = host.snapshot().unwrap();
    let state = textarea(&dirty, node).state();
    let current = units("current\r\n", &[0xd800, 0, 0xdc00, 13, 0xd83d, 0xde42]);
    let default_text = units("tail\r\n", &[0xd800, 0, 0xdc00, 13]);
    assert_eq!(state.raw_value(), &current);
    assert!(state.flags().dirty_value());
    assert_eq!(state.selection().cursor_position(), Some(14));
    assert_eq!(textarea(&dirty, node).form_owner().unwrap(), Some(next));
    assert_eq!(textarea(&before, node).form_owner().unwrap(), Some(first));
    assert_eq!(dirty.node(node).unwrap().parent(), Some(next));
    assert_eq!(
        dirty
            .node(node)
            .unwrap()
            .attribute_ns(&Namespace::none(), &"form".into()),
        Some(&units("textarea-owner", &[0xd800]))
    );
    assert_eq!(textarea(&dirty, spare).form_owner().unwrap(), Some(first));
    assert_eq!(
        textarea(&dirty, spare).state().raw_value(),
        &units("spare\r\n", &[0xdc00])
    );
    let [default_child]: [NodeId; 1] = dirty.node(node).unwrap().children().try_into().unwrap();
    assert_eq!(
        dirty.node(default_child).unwrap().data(),
        Some(&default_text)
    );
    assert_eq!(dirty.incomplete_source_operation(), None);

    host.with_native(|dom, _, mut source| {
        let result = source
            .apply(
                dom,
                DomSourceEdit::reset_textarea(node),
                DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancellation,
            )
            .unwrap();
        assert!(
            matches!(result, DomSourceDriveProgress::Complete(_)),
            "{result:?}"
        );
        assert!(dom.pending_source_edit().is_none());
    })
    .unwrap();
    let reset = host.snapshot().unwrap();
    assert_eq!(textarea(&reset, node).state().raw_value(), &default_text);
    assert!(!textarea(&reset, node).state().flags().dirty_value());
    assert_eq!(reset.node(node).unwrap().children(), &[default_child]);
    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {textarea, next} = textareaProbe;
        if (textarea.value !== 'tail\n\ud800\0\udc00\n' || textarea.textLength !== 9 || textarea.form !== next) {
            return 'native reset restores the API default';
        }
        textarea.firstChild.data = 'live\r\n\udc00';
        return textarea.defaultValue === 'live\r\n\udc00' && textarea.value === 'live\n\udc00' &&
            textarea.textLength === 6 ? 'ok' : 'native reset restored clean children-following mode';
    })()"#,
    );
    let final_state = host.snapshot().unwrap();
    assert_eq!(
        textarea(&final_state, node).state().raw_value(),
        &units("live\r\n", &[0xdc00])
    );
    assert_eq!(final_state.incomplete_source_operation(), None);
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
}

fn textarea(snapshot: &DomSnapshot, node: NodeId) -> surgeist_dom::form::DomTextarea<'_> {
    snapshot.node(node).unwrap().as_textarea().unwrap()
}

fn units(prefix: &str, suffix: &[u16]) -> DocumentString {
    DocumentString::from_utf16(
        prefix
            .encode_utf16()
            .chain(suffix.iter().copied())
            .collect::<Vec<_>>(),
    )
}

fn observe(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit, script: &str) {
    let result = host
        .in_document(permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
}
