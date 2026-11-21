//! The attached Output reads and writes the current native text and override.

use super::*;

pub(super) fn attached_output_reads_native_values_and_stores_custom_validity() {
    let (session, top) = setup();
    let cancellation = Arc::new(cancel());
    let (mut host, _) =
        attach_event_window_with_work(session, top, cancellation.clone(), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const prototype = HTMLOutputElement.prototype;
        const attributes = [['form', false], ['name', true], ['type', false], ['defaultValue', true],
            ['value', true], ['willValidate', false], ['validationMessage', false]];
        const methods = [['checkValidity', 0], ['reportValidity', 0], ['setCustomValidity', 1]];
        const callbacks = [];
        for (const [name, writable] of attributes) {
            const d = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof d?.get !== 'function') return 'missing HTMLOutputElement.prototype.' + name + ' getter';
            if (d.get.name !== 'get ' + name || d.get.length !== 0 || !d.enumerable || !d.configurable ||
                (writable ? typeof d.set !== 'function' || d.set.name !== 'set ' + name || d.set.length !== 1 :
                    d.set !== undefined)) return name + ' Output accessor descriptor';
            callbacks.push(d.get);
            if (writable) callbacks.push(d.set);
        }
        for (const [name, length] of methods) {
            const d = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof d?.value !== 'function' || d.value.name !== name || d.value.length !== length ||
                !d.writable || !d.enumerable || !d.configurable) return name + ' Output method descriptor';
            callbacks.push(d.value);
        }
        const order = [...attributes, ...methods].map(([name]) => name);
        if (Object.getOwnPropertyNames(prototype).filter(name => order.includes(name)).join() !== order.join()) {
            return 'Output declaration order';
        }
        const html = 'http://www.w3.org/1999/xhtml';
        const ordinary = document.createElement('output');
        const prefixed = document.createElementNS(html, 'h:output');
        const upper = document.createElementNS(html, 'OUTPUT');
        const foreign = document.createElementNS('urn:foreign', 'output');
        Object.setPrototypeOf(foreign, prototype);
        if (Object.getPrototypeOf(ordinary) !== prototype || Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
            !(prefixed instanceof HTMLOutputElement) || ordinary.form !== null || prefixed.type !== 'output') {
            return 'exact native Output projection';
        }
        let conversions = 0;
        const unexpected = { [Symbol.toPrimitive]() { conversions++; return 'premature'; } };
        for (const callback of callbacks) {
            for (const invalid of [document.body, upper, foreign, Object.create(prototype), new Proxy(ordinary, {})]) {
                try {
                    Reflect.apply(callback, invalid, [unexpected]);
                    return callback.name + ' accepted a non-Output receiver';
                } catch (error) {
                    if (!(error instanceof TypeError) || conversions !== 0) return callback.name + ' native brand first';
                }
            }
        }
        ordinary.defaultValue = null;
        if (ordinary.value !== 'null' || ordinary.defaultValue !== 'null') return 'defaultValue DOMString null';
        ordinary.value = undefined;
        return ordinary.value === 'undefined' && ordinary.defaultValue === 'null' ? 'ok' : 'value DOMString undefined';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const first = document.body.appendChild(document.createElement('form'));
        const next = document.body.appendChild(document.createElement('form'));
        const formId = 'output-owner\ud800';
        next.setAttributeNS(null, 'id', formId);
        const flow = first.appendChild(document.createElement('output'));
        const stored = first.appendChild(document.createElement('output'));
        flow.setAttributeNS(null, 'type', 'authored');
        flow.setAttributeNS(null, 'NAME', 'uppercase');
        flow.setAttributeNS('urn:foreign', 'f:name', 'foreign');
        if (flow.form !== first || stored.form !== first || flow.type !== 'output' || flow.name !== '') {
            return 'Output Form owner, intrinsic type, and exact name namespace';
        }
        flow.name = 'name\ud800\0\udc00';
        if (flow.name !== 'name\ud800\0\udc00' || flow.getAttributeNS(null, 'name') !== flow.name ||
            flow.getAttributeNS(null, 'NAME') !== 'uppercase' || flow.getAttributeNS('urn:foreign', 'name') !== 'foreign') {
            return 'exact Output name reflection';
        }
        const firstText = flow.appendChild(document.createTextNode(' A'));
        const nested = flow.appendChild(document.createElement('span'));
        nested.appendChild(document.createTextNode('B\ud800 '));
        flow.appendChild(document.createComment('excluded comment'));
        const template = flow.appendChild(document.createElement('template'));
        template.content.appendChild(document.createTextNode('excluded template contents'));
        if (flow.value !== ' AB\ud800 ' || flow.defaultValue !== ' AB\ud800 ') return 'only ordinary descendant Text';
        const base = 'base\ud800\0\udc00\r';
        flow.defaultValue = base;
        if (flow.value !== base || flow.defaultValue !== base || flow.firstChild !== flow.lastChild ||
            flow.firstChild.nodeType !== Node.TEXT_NODE || firstText.parentNode !== null || nested.parentNode !== null) {
            return 'null override default setter replaces all children';
        }
        stored.value = 'stored\udc00';
        if (stored.value !== 'stored\udc00' || stored.defaultValue !== '') return 'explicitly empty stored default';
        const callbacks = [];
        customElements.define('x-output-replacement-probe', class extends HTMLElement {
            disconnectedCallback() {
                callbacks.push([this, this.parentNode, flow.value, flow.defaultValue, flow.form]);
            }
        });
        const removed = flow.appendChild(document.createElement('x-output-replacement-probe'));
        const childText = removed.appendChild(document.createTextNode('before'));
        let invalidEvents = 0;
        flow.addEventListener('invalid', () => invalidEvents++, true);
        if (flow.setCustomValidity(null) !== undefined || stored.setCustomValidity(undefined) !== undefined) {
            return 'required DOMString conversion results';
        }
        globalThis.outputProbe = {flow, stored, first, next, formId, base, removed, childText,
            callbacks, invalidEvents: () => invalidEvents};
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
    let [flow, stored]: [NodeId; 2] = before.node(first).unwrap().children().try_into().unwrap();
    let [initial_text, removed]: [NodeId; 2] =
        before.node(flow).unwrap().children().try_into().unwrap();
    assert_eq!(
        before.node(initial_text).unwrap().data(),
        Some(&units("base", &[0xd800, 0, 0xdc00, 13]))
    );
    let (stored_text, stored_content) = only_text(&before, stored);
    assert_eq!(stored_content, &units("stored", &[0xdc00]));
    assert_eq!(output(&before, flow).state().default_value_override(), None);
    assert_eq!(
        output(&before, stored).state().default_value_override(),
        Some(&DocumentString::from(""))
    );
    assert_eq!(output(&before, flow).form_owner().unwrap(), Some(first));
    assert_eq!(
        output(&before, flow).state().custom_validity_message(),
        &DocumentString::from("null")
    );
    assert_eq!(
        output(&before, stored).state().custom_validity_message(),
        &DocumentString::from("undefined")
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {flow, stored, first, next, formId, base, removed, childText, callbacks, invalidEvents} = outputProbe;
        const storedText = stored.firstChild;
        stored.defaultValue = 'reset\ud800';
        if (stored.value !== 'stored\udc00' || stored.defaultValue !== 'reset\ud800' ||
            stored.firstChild !== storedText || stored.form !== first) return 'override-only edit preserves text identity';
        const setter = Object.getOwnPropertyDescriptor(HTMLOutputElement.prototype, 'value').set;
        let receiver = flow;
        const hints = [];
        const exact = 'current\r\n\ud800\0\udc00';
        const frozen = base + 'during\udc00';
        const result = setter.call(receiver, { [Symbol.toPrimitive](hint) {
            hints.push(hint);
            receiver = stored;
            childText.data = 'during\udc00';
            flow.setAttributeNS(null, 'form', formId);
            return exact;
        } });
        if (result !== undefined || hints.join() !== 'string' || flow.value !== exact ||
            flow.defaultValue !== frozen || flow.form !== next || flow.form !== flow.form ||
            flow.firstChild !== flow.lastChild || stored.firstChild !== storedText ||
            stored.value !== 'stored\udc00' || stored.defaultValue !== 'reset\ud800') {
            return 'value freezes the current default on the original Output';
        }
        const [event] = callbacks;
        if (callbacks.length !== 1 || event[0] !== removed || event[1] !== null ||
            event[2] !== exact || event[3] !== frozen || event[4] !== next) {
            return 'one native disconnection reaction reads the published replacement';
        }
        const method = HTMLOutputElement.prototype.setCustomValidity;
        try {
            method.call(stored);
            return 'setCustomValidity accepted omission';
        } catch (error) {
            if (!(error instanceof TypeError)) return 'setCustomValidity omission TypeError';
        }
        const thrown = {};
        try {
            method.call(stored, {toString() { throw thrown; }});
            return 'setCustomValidity accepted abrupt conversion';
        } catch (error) {
            if (error !== thrown) return 'setCustomValidity original exception';
        }
        let validationReceiver = flow;
        const message = {toString() {
            validationReceiver = stored;
            flow.setAttributeNS(null, 'data-validation', 'converted');
            return 'first\r\nsecond\r\udc00';
        }};
        if (method.call(validationReceiver, message) !== undefined) return 'setCustomValidity completion';
        const unused = {toString() { throw Error('validation converted extra arguments'); }};
        return flow.willValidate === false && flow.validationMessage === '' &&
            flow.checkValidity(unused) === true && flow.reportValidity(unused) === true && invalidEvents() === 0
            ? 'ok' : 'barred validation with a stored custom error';
    })()"#,
    );

    let replaced = host.snapshot().unwrap();
    assert_eq!(output(&replaced, flow).form_owner().unwrap(), Some(next));
    assert_eq!(output(&before, flow).form_owner().unwrap(), Some(first));
    assert_eq!(replaced.node(initial_text).unwrap().parent(), None);
    assert_eq!(replaced.node(removed).unwrap().parent(), None);
    let (new_text, current) = only_text(&replaced, flow);
    assert_ne!(new_text, initial_text);
    assert_eq!(current, &units("current\r\n", &[0xd800, 0, 0xdc00]));
    let frozen = DocumentString::from_utf16(
        "base"
            .encode_utf16()
            .chain([0xd800, 0, 0xdc00, 13])
            .chain("during".encode_utf16())
            .chain([0xdc00])
            .collect::<Vec<_>>(),
    );
    let flow_node = replaced.node(flow).unwrap();
    assert_eq!(
        flow_node.attribute_ns(&Namespace::none(), &"name".into()),
        Some(&units("name", &[0xd800, 0, 0xdc00]))
    );
    assert_eq!(
        flow_node.attribute_ns(&Namespace::none(), &"data-validation".into()),
        Some(&DocumentString::from("converted"))
    );
    let state = output(&replaced, flow).state();
    assert_eq!(state.default_value_override(), Some(&frozen));
    assert_eq!(
        state.custom_validity_message(),
        &units("first\nsecond\n", &[0xdc00])
    );
    assert_eq!(only_text(&replaced, stored).0, stored_text);
    assert_eq!(
        output(&replaced, stored).state().default_value_override(),
        Some(&units("reset", &[0xd800]))
    );
    assert_eq!(
        output(&replaced, stored).state().custom_validity_message(),
        &DocumentString::from("undefined")
    );
    assert_eq!(replaced.incomplete_source_operation(), None);

    host.with_native(|dom, _, mut source| {
        let progress = source
            .apply(
                dom,
                DomSourceEdit::reset_output(stored),
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
            matches!(progress, DomSourceDriveProgress::Complete(_)),
            "{progress:?}"
        );
        assert!(dom.pending_source_edit().is_none());
    })
    .unwrap();
    let reset = host.snapshot().unwrap();
    assert_eq!(
        output(&reset, stored).state().custom_validity_message(),
        &DocumentString::from("undefined")
    );
    assert_eq!(
        output(&reset, stored).state().default_value_override(),
        None
    );
    assert_eq!(reset.node(stored_text).unwrap().parent(), None);
    assert_eq!(only_text(&reset, stored).1, &units("reset", &[0xd800]));
    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {stored} = outputProbe;
        if (stored.value !== 'reset\ud800' || stored.defaultValue !== 'reset\ud800') return 'native reset output';
        stored.firstChild.data = 'live\udc00';
        return stored.value === 'live\udc00' && stored.defaultValue === 'live\udc00'
            ? 'ok' : 'native reset restored descendant default mode';
    })()"#,
    );
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
}

fn output(snapshot: &DomSnapshot, node: NodeId) -> surgeist_dom::form::DomOutput<'_> {
    snapshot.node(node).unwrap().as_output().unwrap()
}

fn only_text(snapshot: &DomSnapshot, node: NodeId) -> (NodeId, &DocumentString) {
    let [child] = snapshot.node(node).unwrap().children() else {
        panic!("expected exactly one canonical Text child");
    };
    let text = snapshot.node(*child).unwrap();
    assert!(matches!(text.kind(), DomNodeKind::Text(_)));
    (*child, text.data().unwrap())
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
