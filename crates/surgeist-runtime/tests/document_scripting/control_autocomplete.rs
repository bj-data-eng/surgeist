//! Attached controls expose HTML's processed autocomplete value and raw Source setters.

use super::*;

pub(super) fn attached_controls_project_live_autocomplete() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const families = [
            [HTMLInputElement, 'input', ['alt', 'autocomplete', 'defaultChecked']],
            [HTMLSelectElement, 'select', ['autocomplete', 'disabled', 'form']],
            [HTMLTextAreaElement, 'textarea', ['autocomplete', 'cols', 'dirName']],
        ];
        const callbacks = [];
        for (const [constructor, , order] of families) {
            const p = constructor.prototype;
            const d = Object.getOwnPropertyDescriptor(p, 'autocomplete');
            if (typeof d?.get !== 'function') return 'missing ' + constructor.name + '.prototype.autocomplete getter';
            if (!d.enumerable || !d.configurable || d.get.name !== 'get autocomplete' || d.get.length !== 0 ||
                typeof d.set !== 'function' || d.set.name !== 'set autocomplete' || d.set.length !== 1) {
                return constructor.name + ' autocomplete descriptor';
            }
            if (Object.getOwnPropertyNames(p).filter(key => order.includes(key)).join() !== order.join()) {
                return constructor.name + ' autocomplete declaration order';
            }
            callbacks.push(d);
        }
        const html = 'http://www.w3.org/1999/xhtml';
        const real = families.map(([, local]) => document.createElementNS(html, 'p:' + local));
        let conversions = 0;
        const proposed = { [Symbol.toPrimitive]() { conversions++; return 'off'; } };
        for (const [i, [constructor, local]] of families.entries()) {
            const p = constructor.prototype;
            const d = callbacks[i];
            const foreign = document.createElementNS('urn:foreign', local);
            Object.setPrototypeOf(foreign, p);
            const wrong = [real[(i + 1) % real.length], foreign, document.createElementNS(html, local.toUpperCase()),
                Object.create(p), new Proxy(real[i], {})];
            if (Object.getPrototypeOf(real[i]) !== p || d.get.call(real[i]) !== '') {
                return constructor.name + ' exact prefix brand and absent default';
            }
            for (const callback of [d.get, d.set]) {
                for (const receiver of wrong) {
                    try {
                        Reflect.apply(callback, receiver, [proposed]);
                        return constructor.name + ' incorrect autocomplete receiver';
                    } catch (error) {
                        if (!(error instanceof TypeError) || conversions !== 0) {
                            return constructor.name + ' must qualify the brand before coercion';
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
        const select = document.createElement('select');
        const textarea = document.createElement('textarea');
        const ordered = '  SECTION-Home\tSHIPPING HOME TeL WEBAUTHN ';
        for (const node of [input, select, textarea]) {
            node.setAttributeNS('urn:foreign', 'p:autocomplete', 'on');
            node.setAttributeNS(null, 'AUTOCOMPLETE', 'off');
            if (node.autocomplete !== '') return node.localName + ' exact null-namespace attribute';
            node.autocomplete = '\t On ';
            if (node.autocomplete !== 'on' || node.getAttributeNS(null, 'autocomplete') !== '\t On ') {
                return node.localName + ' single keyword readback';
            }
            node.autocomplete = ordered;
            if (node.autocomplete !== 'section-home shipping home TeL WEBAUTHN' ||
                node.getAttributeNS(null, 'autocomplete') !== ordered) return node.localName + ' ordered IDL tokens';
            node.autocomplete = 'HOME name';
            if (node.autocomplete !== '') return node.localName + ' ordered-token default';
        }
        textarea.autocomplete = 'WEBAUTHN';
        if (textarea.autocomplete !== 'WEBAUTHN') return 'pinned bare credential recovery';
        input.autocomplete = '  oFf\t';
        if (input.autocomplete !== 'off') return 'Text Input expectation';
        input.type = 'HiDdEn';
        if (input.type !== 'hidden' || input.autocomplete !== '' ||
            input.getAttributeNS(null, 'autocomplete') !== '  oFf\t') return 'live Hidden Input anchor';
        for (const type of ['checkbox', 'radio', 'file', 'submit', 'image', 'reset', 'button']) {
            input.type = type;
            if (input.type !== type || input.autocomplete !== 'off' ||
                input.getAttributeNS(null, 'autocomplete') !== '  oFf\t') return type + ' Input expectation';
        }
        input.type = 'hidden';
        input.autocomplete = 'SECTION-A USERNAME WEBAUTHN';
        return input.autocomplete === 'section-a USERNAME WEBAUTHN' ? 'ok' : 'Hidden Input detail tokens';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const first = document.body.appendChild(document.createElement('form'));
        const next = document.body.appendChild(document.createElement('form'));
        first.autocomplete = 'oFF';
        next.autocomplete = 'on';
        const calls = [];
        const raw = '  SECTION-Trade\t SHIPPING HOME TeL  ';
        const nested = '\ud800\0\udc00';
        class ObservedSelect extends HTMLSelectElement {
            static observedAttributes = ['autocomplete'];
            attributeChangedCallback(name, previous, value, namespace) {
                calls.push([this, name, previous, value, namespace, this.autocomplete, this.form]);
                if (value === raw) this.autocomplete = nested;
            }
        }
        customElements.define('x-control-autocomplete-probe', ObservedSelect, {extends: 'select'});
        const input = first.appendChild(document.createElement('input'));
        const select = first.appendChild(document.createElement('select', {is: 'x-control-autocomplete-probe'}));
        const textarea = first.appendChild(document.createElement('textarea'));
        const other = next.appendChild(document.createElement('select'));
        if (!(select instanceof ObservedSelect) || first.autocomplete !== 'off' || next.autocomplete !== 'on' ||
            [input, select, textarea].some(node => node.form !== first || node.autocomplete !== '') ||
            other.form !== next || other.autocomplete !== '') return 'control defaults independent of live Form policy';

        input.autocomplete = '  EMAIL ';
        textarea.autocomplete = null;
        if (input.autocomplete !== 'EMAIL' || textarea.autocomplete !== '' ||
            textarea.getAttributeNS(null, 'autocomplete') !== 'null') return 'exact normal DOMString setter';
        globalThis.controlAutocompleteProbe = {first, next, select, other, calls, raw, nested};
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
    let [input, select, textarea]: [NodeId; 3] =
        before.node(first).unwrap().children().try_into().unwrap();
    let [other]: [NodeId; 1] = before.node(next).unwrap().children().try_into().unwrap();
    assert_eq!(select_owner(&before, select), Some(first));
    assert_eq!(attribute(&before, select), None);

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {first, next, select, other, calls, raw, nested} = controlAutocompleteProbe;
        const setter = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'autocomplete').set;
        let receiver = select;
        const hints = [];
        const result = setter.call(receiver, { [Symbol.toPrimitive](hint) {
            hints.push(hint);
            receiver = other;
            next.appendChild(select);
            if (select.form !== next || select.autocomplete !== '') throw new Error('live Form reassociation');
            return raw;
        } });
        if (result !== undefined || hints.join() !== 'string' || receiver !== other ||
            select.form === first || select.form !== next || select.autocomplete !== '' ||
            select.getAttributeNS(null, 'autocomplete') !== nested ||
            other.autocomplete !== '' || other.getAttributeNS(null, 'autocomplete') !== null) {
            return 'original live Select through setter coercion and reaction';
        }
        const expected = [
            ['autocomplete', null, raw, null, 'section-trade shipping home TeL', next],
            ['autocomplete', raw, nested, null, '', next],
        ];
        if (calls.length !== expected.length || calls.some((row, i) =>
            row[0] !== select || expected[i].some((value, j) => value !== row[j + 1]))) {
            return 'synchronous autocomplete Source reactions read each published value';
        }
        const thrown = {};
        try {
            setter.call(select, {toString() {throw thrown;}});
            return 'abrupt autocomplete conversion returned';
        } catch (error) {
            return error === thrown && calls.length === expected.length &&
                select.getAttributeNS(null, 'autocomplete') === nested ? 'ok' : 'abrupt conversion changed Select';
        }
    })()"#,
    );

    let after = host.snapshot().unwrap();
    assert_eq!(after.node(select).unwrap().parent(), Some(next));
    assert_eq!(select_owner(&after, select), Some(next));
    assert_eq!(select_owner(&after, other), Some(next));
    assert_eq!(select_owner(&before, select), Some(first));
    assert_eq!(
        attribute(&after, input),
        Some(&DocumentString::from("  EMAIL "))
    );
    assert_eq!(
        attribute(&after, textarea),
        Some(&DocumentString::from("null"))
    );
    assert_eq!(
        attribute(&after, select),
        Some(&DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]))
    );
    assert_eq!(attribute(&after, other), None);
    assert_eq!(after.incomplete_source_operation(), None);
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
}

fn select_owner(snapshot: &DomSnapshot, node: NodeId) -> Option<NodeId> {
    snapshot
        .node(node)
        .unwrap()
        .as_select()
        .unwrap()
        .form_owner()
        .unwrap()
}

fn attribute(snapshot: &DomSnapshot, node: NodeId) -> Option<&DocumentString> {
    snapshot
        .node(node)
        .unwrap()
        .attribute_ns(&Namespace::none(), &"autocomplete".into())
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
