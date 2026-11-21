//! Attached integer control reflections use the pinned recovery and conversion.

use super::*;

pub(super) fn attached_controls_reflect_native_integer_attributes() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const input = {
            constructor: HTMLInputElement,
            members: ['maxLength', 'minLength', 'size'],
            order: ['accept', 'alpha', 'alt', 'defaultChecked', 'checked', 'colorSpace', 'dirName',
                'disabled', 'form', 'formEnctype', 'formMethod', 'formNoValidate', 'formTarget',
                'indeterminate', 'max', 'maxLength', 'min', 'minLength', 'multiple', 'name',
                'pattern', 'placeholder', 'readOnly', 'required', 'size', 'step', 'type',
                'defaultValue', 'value', 'align', 'useMap'],
        };
        const textarea = {
            constructor: HTMLTextAreaElement,
            members: ['cols', 'maxLength', 'minLength', 'rows'],
            order: ['cols', 'dirName', 'disabled', 'form', 'maxLength', 'minLength', 'name',
                'placeholder', 'readOnly', 'required', 'rows', 'wrap', 'type', 'defaultValue',
                'value', 'textLength'],
        };
        for (const family of [input, textarea]) {
            const p = family.constructor.prototype;
            family.callbacks = [];
            for (const key of family.members) {
                const d = Object.getOwnPropertyDescriptor(p, key);
                if (typeof d?.get !== 'function') return 'missing ' + family.constructor.name + '.prototype.' + key + ' getter';
                if (!d.enumerable || !d.configurable || d.get.name !== 'get ' + key || d.get.length !== 0 ||
                    typeof d.set !== 'function' || d.set.name !== 'set ' + key || d.set.length !== 1) {
                    return family.constructor.name + '.' + key + ' descriptor';
                }
                family.callbacks.push(d.get, d.set);
            }
        }
        for (const family of [input, textarea]) {
            if (Object.getOwnPropertyNames(family.constructor.prototype).filter(key => family.order.includes(key))
                    .join() !== family.order.join()) return family.constructor.name + ' combined property order';
        }
        const html = 'http://www.w3.org/1999/xhtml';
        const realInput = document.createElementNS(html, 'p:input');
        const realTextarea = document.createElementNS(html, 'p:textarea');
        let conversions = 0;
        const argument = { [Symbol.toPrimitive]() { conversions++; return 4; } };
        for (const [family, local, correct, cross] of [
            [input, 'input', realInput, realTextarea], [textarea, 'textarea', realTextarea, realInput],
        ]) {
            const p = family.constructor.prototype;
            const foreign = document.createElementNS('urn:foreign', local);
            Object.setPrototypeOf(foreign, p);
            const wrong = [cross, foreign, document.createElementNS(html, local.toUpperCase()),
                Object.create(p), new Proxy(correct, {})];
            if (Object.getPrototypeOf(correct) !== p) return family.constructor.name + ' native prefix brand';
            for (const callback of family.callbacks) {
                for (const receiver of wrong) {
                    try {
                        Reflect.apply(callback, receiver, [argument]);
                        return callback.name + ' accepts an incorrect native brand';
                    } catch (error) {
                        if (!(error instanceof TypeError) || conversions !== 0) {
                            return callback.name + ' must qualify the brand before coercion';
                        }
                    }
                }
            }
        }
        return 'ok';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const input = document.createElement('input');
        const textarea = document.createElement('textarea');
        input.type = 'checkbox';
        const members = [
            [input, 'maxLength', 'maxlength', -1], [input, 'minLength', 'minlength', -1],
            [textarea, 'maxLength', 'maxlength', -1], [textarea, 'minLength', 'minlength', -1],
            [input, 'size', 'size', 20], [textarea, 'cols', 'cols', 20], [textarea, 'rows', 'rows', 2],
        ];
        for (const [node, key, name, fallback] of members) {
            node.setAttributeNS('urn:foreign', 'p:' + name, '7');
            node.setAttributeNS(null, name.toUpperCase(), '8');
            if (node[key] !== fallback || node.getAttributeNS(null, name) !== null) return key + ' missing own content';
            for (const [raw, expected] of [
                ['', fallback], ['bad', fallback], ['-1', fallback],
                [' \t-0tail', fallback === -1 ? 0 : fallback],
                [' \n+007tail\ud800', 7], ['2147483647', 2147483647],
                ['2147483648', fallback], ['999999999999999999999999999999999999', fallback],
            ]) {
                node.setAttributeNS(null, name, raw);
                if (node[key] !== expected || node.getAttributeNS(null, name) !== raw) {
                    return key + ' recovered number and unchanged raw content for ' + raw.length + ' code units';
                }
            }
            node[key] = 2147483647;
            if (node[key] !== 2147483647 || node.getAttributeNS(null, name) !== '2147483647') {
                return key + ' maximum reflected setter';
            }
            node[key] = 2 ** 32 + 7.75;
            if (node[key] !== 7 || node.getAttributeNS(null, name) !== '7') return key + ' Web IDL wrap and truncation';
            if (key !== 'size') {
                node[key] = 2 ** 32;
                const zero = fallback === -1 ? 0 : fallback;
                if (node[key] !== zero || node.getAttributeNS(null, name) !== String(zero)) {
                    return key + ' converted-zero setter policy';
                }
            }
            if (fallback !== -1) {
                for (const raw of [2147483648, -1]) {
                    node[key] = raw;
                    if (node[key] !== fallback || node.getAttributeNS(null, name) !== String(fallback)) {
                        return key + ' oversized unsigned-long setter policy';
                    }
                }
            }
        }
        return 'ok';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const first = document.body.appendChild(document.createElement('form'));
        const next = document.body.appendChild(document.createElement('form'));
        const calls = [];
        class ObservedInput extends HTMLInputElement {
            static observedAttributes = ['size', 'maxlength', 'minlength'];
            attributeChangedCallback(name, previous, value, namespace) {
                calls.push([this, name, previous, value, namespace, this.size, this.maxLength, this.form]);
                if (name === 'size' && value === '7') this.minLength = 2;
            }
        }
        customElements.define('x-numeric-control-probe', ObservedInput, {extends: 'input'});
        const input = first.appendChild(document.createElement('input', {is: 'x-numeric-control-probe'}));
        const textarea = first.appendChild(document.createElement('textarea'));
        const other = next.appendChild(document.createElement('input'));
        input.size = 4;
        input.maxLength = 8;
        input.minLength = 6;
        textarea.cols = 0;
        textarea.rows = 2 ** 31;
        textarea.maxLength = 2 ** 32 + 7.5;
        textarea.minLength = -0.75;
        if (!(input instanceof ObservedInput) || input.form !== first || textarea.form !== first ||
            other.form !== next || textarea.cols !== 20 || textarea.rows !== 2 ||
            textarea.maxLength !== 7 || textarea.minLength !== 0) return 'live integer control setup';
        calls.length = 0;
        globalThis.numericControlProbe = {input, textarea, first, next, other, calls};
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
    let [input, textarea]: [NodeId; 2] = before.node(first).unwrap().children().try_into().unwrap();
    let [other]: [NodeId; 1] = before.node(next).unwrap().children().try_into().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {input, textarea, calls} = numericControlProbe;
        for (const [node, key, value, stored] of [
            [input, 'maxLength', 2 ** 31, '8'], [input, 'minLength', -1, '6'],
            [input, 'size', 0, '4'], [input, 'size', 2 ** 32, '4'],
            [textarea, 'maxLength', -1, '7'], [textarea, 'minLength', 2 ** 31, '0'],
        ]) {
            try {
                node[key] = value;
                return key + ' accepted a disallowed converted integer';
            } catch (error) {
                if (!(error instanceof DOMException) || Object.getPrototypeOf(error) !== DOMException.prototype ||
                    error.name !== 'IndexSizeError' || error.code !== 1) return key + ' realm IndexSizeError';
                if (node.getAttributeNS(null, key.toLowerCase()) !== stored || calls.length !== 0) {
                    return key + ' rejection edited the attribute or ran a reaction';
                }
            }
        }
        return 'ok';
    })()"#,
    );
    let rejected = host.snapshot().unwrap();
    assert_eq!(rejected.stamp(), before.stamp());
    assert_eq!(rejected.incomplete_source_operation(), None);

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {input, textarea, first, next, other, calls} = numericControlProbe;
        const setSize = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'size').set;
        let receiver = input;
        const hints = [];
        const result = setSize.call(receiver, { [Symbol.toPrimitive](hint) {
            hints.push(hint);
            receiver = other;
            next.appendChild(input);
            input.maxLength = 2 ** 32 + 9.75;
            return 2 ** 32 + 7.75;
        } });
        if (result !== undefined || hints.join() !== 'number' || receiver !== other || input.form !== next ||
            textarea.form !== first || input.size !== 7 || input.maxLength !== 9 || input.minLength !== 2 ||
            other.size !== 20 || other.maxLength !== -1 || other.minLength !== -1) {
            return 'numeric coercion keeps the original live receiver';
        }
        const expected = [
            ['maxlength', '8', '9', null, 4, 9, next],
            ['size', '4', '7', null, 7, 9, next],
            ['minlength', '6', '2', null, 7, 9, next],
        ];
        if (calls.length !== expected.length || calls.some((row, index) =>
            row[0] !== input || expected[index].some((value, offset) => value !== row[offset + 1]))) {
            return 'synchronous integer reactions see canonical decimal and allow reentry';
        }
        const thrown = {};
        try {
            setSize.call(input, {valueOf() {throw thrown;}});
            return 'abrupt integer conversion returned';
        } catch (error) {
            return error === thrown && input.size === 7 && calls.length === expected.length ?
                'ok' : 'abrupt integer conversion changed Input';
        }
    })()"#,
    );

    let after = host.snapshot().unwrap();
    for (node, name, expected) in [
        (input, "maxlength", "9"),
        (input, "minlength", "2"),
        (input, "size", "7"),
        (textarea, "cols", "20"),
        (textarea, "maxlength", "7"),
        (textarea, "minlength", "0"),
        (textarea, "rows", "2"),
    ] {
        assert_eq!(
            attribute(&after, node, name),
            Some(&DocumentString::from(expected)),
            "{name}"
        );
    }
    assert_eq!(after.node(input).unwrap().parent(), Some(next));
    assert_eq!(after.node(textarea).unwrap().parent(), Some(first));
    for name in ["maxlength", "minlength", "size"] {
        assert_eq!(attribute(&after, other, name), None, "{name}");
    }
    assert_eq!(after.incomplete_source_operation(), None);
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
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
