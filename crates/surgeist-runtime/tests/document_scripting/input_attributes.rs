//! Attached Input reflections preserve exact authored values and live Source reactions.

use super::*;

pub(super) fn attached_input_reflections_use_canonical_attributes() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const prototype = HTMLInputElement.prototype;
        const added = ['accept', 'alpha', 'alt', 'colorSpace', 'formEnctype', 'formMethod',
            'formNoValidate', 'formTarget', 'max', 'min', 'pattern', 'step', 'align', 'useMap'];
        const expected = ['accept', 'alpha', 'alt', 'defaultChecked', 'checked', 'colorSpace',
            'dirName', 'disabled', 'form', 'formEnctype', 'formMethod', 'formNoValidate', 'formTarget',
            'indeterminate', 'max', 'min', 'multiple', 'name', 'pattern', 'placeholder',
            'readOnly', 'required', 'step', 'type', 'defaultValue', 'value', 'align', 'useMap'];
        const callbacks = [];
        for (const name of added) {
            const d = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof d?.get !== 'function') return 'missing HTMLInputElement.prototype.' + name + ' getter';
            if (!d.enumerable || !d.configurable || d.get.name !== 'get ' + name || d.get.length !== 0 ||
                typeof d.set !== 'function' || d.set.name !== 'set ' + name || d.set.length !== 1) {
                return name + ' Input reflection descriptor';
            }
            callbacks.push(d.get, d.set);
        }
        if (Object.getOwnPropertyNames(prototype).filter(name => expected.includes(name)).join() !== expected.join()) {
            return 'combined Input declaration order';
        }
        const html = 'http://www.w3.org/1999/xhtml';
        const native = document.createElement('input');
        const prefixed = document.createElementNS(html, 'p:input');
        const upper = document.createElementNS(html, 'INPUT');
        const foreign = document.createElementNS('urn:foreign', 'input');
        Object.setPrototypeOf(foreign, prototype);
        if (Object.getPrototypeOf(native) !== prototype || Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
            !(prefixed instanceof HTMLInputElement)) return 'exact native Input prototypes';
        let conversions = 0;
        const proposed = { [Symbol.toPrimitive]() { conversions++; return 'premature'; } };
        const wrong = [document.body, upper, foreign, Object.create(prototype), new Proxy(native, {})];
        for (const callback of callbacks) {
            for (const receiver of wrong) {
                try {
                    Reflect.apply(callback, receiver, [proposed]);
                    return callback.name + ' accepted a non-Input receiver';
                } catch (error) {
                    if (!(error instanceof TypeError) || conversions !== 0) return callback.name + ' native brand first';
                }
            }
        }
        prefixed.setAttributeNS('urn:foreign', 'f:colorspace', 'display-p3');
        prefixed.setAttributeNS(null, 'COLORSPACE', 'display-p3');
        if (prefixed.colorSpace !== 'limited-srgb') return 'ColorSpace missing own content';
        prefixed.colorSpace = 'dIsPlAy-P3';
        if (prefixed.colorSpace !== 'display-p3' || prefixed.getAttributeNS(null, 'colorspace') !== 'dIsPlAy-P3') {
            return 'ColorSpace canonical getter and raw setter';
        }
        prefixed.type = 'checkbox';
        if (prefixed.colorSpace !== 'display-p3') return 'ColorSpace reflection applies to all Input types';
        for (const value of ['', ' display-p3 ', 'unknown\ud800', null]) {
            prefixed.colorSpace = value;
            if (prefixed.colorSpace !== 'limited-srgb' || prefixed.getAttributeNS(null, 'colorspace') !== String(value)) {
                return 'ColorSpace invalid-present default and exact content';
            }
        }
        prefixed.colorSpace = 'LiMiTeD-sRgB';
        return prefixed.colorSpace === 'limited-srgb' &&
            prefixed.getAttributeNS(null, 'colorspace') === 'LiMiTeD-sRgB' ? 'ok' : 'known limited-srgb';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const first = document.body.appendChild(document.createElement('form'));
        const next = document.body.appendChild(document.createElement('form'));
        first.enctype = 'multipart/form-data';
        first.method = 'dialog';
        first.target = 'owner-target';
        first.noValidate = true;
        next.enctype = 'application/x-www-form-urlencoded';
        next.method = 'get';

        const exact = ' Raw\ud800\0\udc00 ';
        const colorRaw = 'DiSpLaY-P3';
        const reacted = 'map\0\udc00';
        const calls = [];
        class ObservedInput extends HTMLInputElement {
            static observedAttributes = ['colorspace', 'formmethod', 'usemap'];
            attributeChangedCallback(name, previous, value, namespace) {
                calls.push([this, name, previous, value, namespace, this.colorSpace, this.form, this.formMethod]);
                if (name === 'colorspace' && value === colorRaw) this.useMap = reacted;
            }
        }
        customElements.define('x-input-attributes-probe', ObservedInput, {extends: 'input'});
        const input = first.appendChild(document.createElement('input', {is: 'x-input-attributes-probe'}));
        const other = next.appendChild(document.createElement('input'));
        if (!(input instanceof ObservedInput) || input.type !== 'text' || input.form !== first ||
            input.formTarget !== '' || input.formNoValidate || first.target !== 'owner-target' ||
            !first.noValidate) return 'ordinary Text Input does not inherit its Form settings';

        for (const [member, attribute, fallback, authored, canonical] of [
            ['formEnctype', 'formenctype', 'application/x-www-form-urlencoded', 'TeXt/PlAiN', 'text/plain'],
            ['formMethod', 'formmethod', 'get', 'PoSt', 'post'],
        ]) {
            input.setAttributeNS('urn:foreign', 'f:' + attribute, authored);
            input.setAttributeNS(null, attribute.toUpperCase(), authored);
            if (input[member] !== '') return member + ' missing override is empty';
            input[member] = '';
            if (input[member] !== fallback || input.getAttributeNS(null, attribute) !== '') {
                return member + ' present-empty override selects invalid default';
            }
            input[member] = exact;
            if (input[member] !== fallback || input.getAttributeNS(null, attribute) !== exact) {
                return member + ' invalid override preserves raw content';
            }
            input[member] = authored;
            if (input[member] !== canonical || input.getAttributeNS(null, attribute) !== authored) {
                return member + ' known override canonical readback';
            }
        }
        if (input.form.enctype !== 'multipart/form-data' || input.form.method !== 'dialog') return 'real Form policy';

        for (const [member, attribute] of [['accept','accept'], ['alt','alt'], ['formTarget','formtarget'],
            ['max','max'], ['min','min'], ['pattern','pattern'], ['step','step'], ['align','align'], ['useMap','usemap']]) {
            input.setAttributeNS('urn:foreign', 'f:' + attribute, 'foreign');
            input.setAttributeNS(null, attribute.toUpperCase(), 'uppercase');
            if (input[member] !== '') return member + ' missing own content';
            input[member] = exact;
            if (input[member] !== exact || input.getAttributeNS(null, attribute) !== exact ||
                input.getAttributeNS('urn:foreign', attribute) !== 'foreign' ||
                input.getAttributeNS(null, attribute.toUpperCase()) !== 'uppercase') return member + ' exact DOMString';
        }
        let conversions = 0;
        const truthy = { [Symbol.toPrimitive]() { conversions++; return ''; } };
        for (const [member, attribute] of [['alpha','alpha'], ['formNoValidate','formnovalidate']]) {
            input.setAttributeNS('urn:foreign', 'f:' + attribute, '');
            if (input[member]) return member + ' ignores foreign presence';
            input.setAttributeNS(null, attribute, 'false');
            if (!input[member]) return member + ' uses own presence';
            input[member] = false;
            if (input[member] || input.getAttributeNS(null, attribute) !== null ||
                input.getAttributeNS('urn:foreign', attribute) !== '') return member + ' removes own attribute';
            input[member] = truthy;
            if (!input[member] || input.getAttributeNS(null, attribute) !== '' || conversions !== 0) {
                return member + ' Web IDL Boolean';
            }
        }
        input.useMap = null;
        input.formMethod = 'GET';
        if (input.useMap !== 'null' || input.getAttributeNS(null, 'usemap') !== 'null') {
            return 'obsolete reflection uses ordinary DOMString null conversion';
        }
        calls.length = 0;
        globalThis.inputAttributesProbe = {input, first, next, other, calls, colorRaw, reacted};
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
    let [subject]: [NodeId; 1] = before.node(first).unwrap().children().try_into().unwrap();
    let [other]: [NodeId; 1] = before.node(next).unwrap().children().try_into().unwrap();
    assert_eq!(form_owner(&before, subject), Some(first));
    assert_eq!(form_owner(&before, other), Some(next));

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {input, first, next, other, calls, colorRaw, reacted} = inputAttributesProbe;
        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'colorSpace').set;
        let receiver = input;
        const hints = [];
        const result = setter.call(receiver, { [Symbol.toPrimitive](hint) {
            hints.push(hint);
            receiver = other;
            next.appendChild(input);
            input.formMethod = 'PoSt';
            return colorRaw;
        } });
        if (result !== undefined || hints.join() !== 'string' || receiver !== other ||
            input.form === first || input.form !== next || input.colorSpace !== 'display-p3' ||
            input.useMap !== reacted || input.formMethod !== 'post' || input.formEnctype !== 'text/plain' ||
            input.form.method !== 'get' || input.form.enctype !== 'application/x-www-form-urlencoded' ||
            other.colorSpace !== 'limited-srgb' || other.formMethod !== '' || other.useMap !== '') {
            return 'original Input and own overrides survive coercion and reassociation';
        }
        const expected = [
            ['formmethod', 'GET', 'PoSt', null, 'limited-srgb', next, 'post'],
            ['colorspace', null, colorRaw, null, 'display-p3', next, 'post'],
            ['usemap', 'null', reacted, null, 'display-p3', next, 'post'],
        ];
        if (calls.length !== expected.length || calls.some((call, index) =>
            call[0] !== input || expected[index].some((value, offset) => value !== call[offset + 1]))) {
            return 'Input Source reactions see published state and reenter the original Input';
        }
        const thrown = {};
        try {
            setter.call(input, { toString() { throw thrown; } });
            return 'abrupt ColorSpace conversion returned';
        } catch (error) {
            return error === thrown && calls.length === expected.length &&
                input.getAttributeNS(null, 'colorspace') === colorRaw ? 'ok' : 'abrupt conversion edited Input';
        }
    })()"#,
    );

    let after = host.snapshot().unwrap();
    assert_eq!(form_owner(&after, subject), Some(next));
    assert_eq!(form_owner(&after, other), Some(next));
    assert_eq!(after.node(subject).unwrap().parent(), Some(next));
    assert_eq!(form_owner(&before, subject), Some(first));

    let exact = DocumentString::from_utf16(vec![32, 82, 97, 119, 0xd800, 0, 0xdc00, 32]);
    for name in [
        "accept",
        "alt",
        "formtarget",
        "max",
        "min",
        "pattern",
        "step",
        "align",
    ] {
        assert_eq!(attribute(&after, subject, name), Some(&exact), "{name}");
    }
    for (name, value) in [
        ("alpha", ""),
        ("formnovalidate", ""),
        ("colorspace", "DiSpLaY-P3"),
        ("formenctype", "TeXt/PlAiN"),
        ("formmethod", "PoSt"),
    ] {
        assert_eq!(
            attribute(&after, subject, name),
            Some(&DocumentString::from(value)),
            "{name}"
        );
    }
    assert_eq!(
        attribute(&after, subject, "usemap"),
        Some(&DocumentString::from_utf16(vec![109, 97, 112, 0, 0xdc00]))
    );
    for name in ["colorspace", "formmethod", "usemap"] {
        assert_eq!(attribute(&after, other, name), None, "{name}");
    }
    assert_eq!(after.incomplete_source_operation(), None);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
}

fn form_owner(snapshot: &DomSnapshot, node: NodeId) -> Option<NodeId> {
    snapshot
        .node(node)
        .unwrap()
        .as_input()
        .unwrap()
        .form_owner()
        .unwrap()
}

fn attribute<'a>(
    snapshot: &'a DomSnapshot,
    node: NodeId,
    name: &str,
) -> Option<&'a DocumentString> {
    snapshot
        .node(node)
        .unwrap()
        .attribute_ns(&Namespace::none(), &name.into())
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
