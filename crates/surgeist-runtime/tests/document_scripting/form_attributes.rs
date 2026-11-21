//! Form settings reflect canonical attributes on the native Form prototype.

use super::*;

pub(super) fn attached_form_settings_reflect_live_native_attributes() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    observe(
        &mut host,
        &permit,
        r#"(() => {
        const prototype = HTMLFormElement.prototype;
        const names = ['acceptCharset', 'autocomplete', 'enctype', 'encoding', 'method',
            'name', 'noValidate', 'target', 'rel'];
        for (const name of names) {
            const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof descriptor?.get !== 'function') {
                return 'missing HTMLFormElement.prototype.' + name + ' getter';
            }
            if (typeof descriptor.set !== 'function' || descriptor.get.name !== 'get ' + name ||
                descriptor.set.name !== 'set ' + name || descriptor.get.length !== 0 ||
                descriptor.set.length !== 1 || !descriptor.enumerable || !descriptor.configurable) {
                return 'Form accessor descriptor for ' + name;
            }
        }
        if (Object.getOwnPropertyNames(prototype).filter(key => names.includes(key)).join() !== names.join()) {
            return 'Form settings declaration order';
        }
        const html = 'http://www.w3.org/1999/xhtml';
        const form = document.body.appendChild(document.createElement('form'));
        const prefixed = document.createElementNS(html, 'h:form');
        const upper = document.createElementNS(html, 'FORM');
        const foreign = document.createElementNS('urn:foreign', 'form');
        Object.setPrototypeOf(foreign, prototype);
        if (Object.getPrototypeOf(form) !== prototype || Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
            !(prefixed instanceof HTMLFormElement) || 'acceptCharset' in HTMLElement.prototype) {
            return 'exact Form interface projection';
        }
        const invalid = [document.body, foreign, upper, Object.create(prototype), new Proxy(form, {})];
        let conversions = 0;
        const argument = { [Symbol.toPrimitive]() { conversions++; return 'too early'; } };
        for (const name of names) {
            const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
            for (const other of invalid) {
                for (const accessor of [descriptor.get, descriptor.set]) {
                    try {
                        Reflect.apply(accessor, other, [argument]);
                        return name + ' accepted a non-Form receiver';
                    } catch (error) {
                        if (!(error instanceof TypeError) || conversions !== 0) return name + ' native brand first';
                    }
                }
            }
        }
        const accept = Object.getOwnPropertyDescriptor(prototype, 'acceptCharset');
        if (accept.set.call(prefixed, ' raw charset ') !== undefined || accept.get.call(prefixed) !== ' raw charset ') {
            return 'detached prefixed Form receiver';
        }

        const exact = ' Raw\ud800\0\udc00 ';
        for (const [member, attribute] of [
            ['acceptCharset', 'accept-charset'], ['name', 'name'], ['target', 'target'], ['rel', 'rel'],
        ]) {
            form.setAttributeNS('urn:foreign', 'f:' + attribute, 'foreign');
            form.setAttributeNS(null, attribute.toUpperCase(), 'uppercase');
            if (form[member] !== '' || form.getAttributeNS(null, attribute) !== null) return member + ' missing value';
            form.setAttributeNS(null, attribute, exact);
            if (form[member] !== exact) return member + ' exact authored value';
            form[member] = null;
            if (form[member] !== 'null' || form.getAttributeNS(null, attribute) !== 'null') return member + ' DOMString null';
            form[member] = exact;
            if (form[member] !== exact || form.getAttributeNS(null, attribute) !== exact ||
                form.getAttributeNS('urn:foreign', attribute) !== 'foreign' ||
                form.getAttributeNS(null, attribute.toUpperCase()) !== 'uppercase') return member + ' exact native attribute';
        }

        form.setAttributeNS('urn:foreign', 'f:novalidate', '');
        form.setAttributeNS(null, 'NOVALIDATE', '');
        if (form.noValidate) return 'boolean reads exact null-namespace name';
        form.setAttributeNS(null, 'novalidate', 'false');
        if (!form.noValidate) return 'boolean reads presence';
        form.noValidate = argument;
        if (!form.noValidate || form.getAttributeNS(null, 'novalidate') !== '' || conversions !== 0) {
            return 'boolean setter uses truthiness';
        }
        form.noValidate = false;
        if (form.noValidate || form.getAttributeNS(null, 'novalidate') !== null ||
            form.getAttributeNS('urn:foreign', 'novalidate') !== '' ||
            form.getAttributeNS(null, 'NOVALIDATE') !== '') return 'boolean clears only its content attribute';

        const fallback = 'application/x-www-form-urlencoded';
        for (const [member, initial, authored, canonical] of [
            ['autocomplete', 'on', 'OfF', 'off'],
            ['enctype', fallback, 'MuLtIpArT/FoRm-DaTa', 'multipart/form-data'],
            ['method', 'get', 'PoSt', 'post'],
        ]) {
            form.setAttributeNS('urn:foreign', 'f:' + member, authored);
            if (form[member] !== initial) return member + ' missing canonical default';
            for (const raw of ['', ' ' + authored, 'bad\ud800']) {
                form[member] = raw;
                if (form[member] !== initial || form.getAttributeNS(null, member) !== raw) {
                    return member + ' exact invalid content and canonical default';
                }
            }
            form.setAttributeNS(null, member, authored);
            if (form[member] !== canonical) return member + ' canonical authored keyword';
            form[member] = authored;
            if (form[member] !== canonical || form.getAttributeNS(null, member) !== authored) {
                return member + ' canonical getter and raw setter';
            }
        }

        const enctype = Object.getOwnPropertyDescriptor(prototype, 'enctype');
        const encoding = Object.getOwnPropertyDescriptor(prototype, 'encoding');
        if (enctype.get === encoding.get || enctype.set === encoding.set) return 'distinct encoding accessor pair';
        form.setAttributeNS(null, 'encoding', 'unrelated content');
        form.encoding = 'TeXt/PlaIn';
        if (form.enctype !== 'text/plain' || form.encoding !== 'text/plain' ||
            form.getAttributeNS(null, 'enctype') !== 'TeXt/PlaIn' ||
            form.getAttributeNS(null, 'encoding') !== 'unrelated content') return 'encoding writes the enctype attribute';
        form.enctype = '';
        return form.enctype === fallback && form.encoding === fallback &&
            form.getAttributeNS(null, 'enctype') === '' ? 'ok' : 'shared encoding default';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const origin = document.body.appendChild(document.createElement('div'));
        const destination = document.body.appendChild(document.createElement('div'));
        const replacement = origin.appendChild(document.createElement('form'));
        replacement.enctype = 'text/plain';
        const calls = [];
        class ObservedForm extends HTMLFormElement {
            static observedAttributes = ['enctype', 'name'];
            attributeChangedCallback(attribute, previous, value, namespace) {
                calls.push([this, attribute, previous, value, namespace, this.getAttributeNS(null, attribute),
                    this.enctype, this.encoding]);
                if (attribute === 'enctype' && value === 'TeXt/PlaIn') this.name = 'callback\ud800';
            }
        }
        customElements.define('x-observed-form-settings', ObservedForm, {extends: 'form'});
        const form = origin.appendChild(document.createElement('form', {is: 'x-observed-form-settings'}));
        if (!(form instanceof ObservedForm)) return 'native customized Form fixture';
        form.enctype = 'MULTIPART/FORM-DATA';
        let receiver = form;
        const exact = 'Bad\ud800\0\udc00';
        const fallback = 'application/x-www-form-urlencoded';
        const setter = Object.getOwnPropertyDescriptor(HTMLFormElement.prototype, 'encoding').set;
        const hints = [];
        let duringConversion = false;
        const result = setter.call(receiver, { [Symbol.toPrimitive](hint) {
            hints.push(hint);
            duringConversion = form.parentNode === origin && form.encoding === 'multipart/form-data' &&
                form.getAttributeNS(null, 'enctype') === 'MULTIPART/FORM-DATA' && calls.length === 1;
            receiver = replacement;
            destination.appendChild(form);
            form.enctype = 'TeXt/PlaIn';
            return exact;
        } });
        if (result !== undefined || hints.join() !== 'string' || !duringConversion || form.parentNode !== destination ||
            form.getAttributeNS(null, 'enctype') !== exact || form.hasAttributeNS(null, 'encoding') ||
            form.encoding !== fallback || replacement.getAttributeNS(null, 'enctype') !== 'text/plain') {
            return 'conversion reentry keeps the original native Form';
        }
        const expected = [
            ['enctype', null, 'MULTIPART/FORM-DATA', null, 'MULTIPART/FORM-DATA', 'multipart/form-data', 'multipart/form-data'],
            ['enctype', 'MULTIPART/FORM-DATA', 'TeXt/PlaIn', null, 'TeXt/PlaIn', 'text/plain', 'text/plain'],
            ['name', null, 'callback\ud800', null, 'callback\ud800', 'text/plain', 'text/plain'],
            ['enctype', 'TeXt/PlaIn', exact, null, exact, fallback, fallback],
        ];
        if (calls.length !== expected.length || calls.some(call => call[0] !== form) ||
            JSON.stringify(calls.map(call => call.slice(1))) !== JSON.stringify(expected)) {
            return 'canonical Form reactions run synchronously on the original receiver';
        }
        const thrown = {};
        try {
            setter.call(form, { toString() { throw thrown; } });
            return 'encoding accepted abrupt conversion';
        } catch (error) {
            return error === thrown && calls.length === expected.length &&
                form.getAttributeNS(null, 'enctype') === exact ? 'ok' : 'encoding abrupt conversion changed the Form';
        }
    })()"#,
    );

    let snapshot = host.snapshot().unwrap();
    let root = snapshot
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(root).unwrap().children()[1];
    let [ordinary, origin, destination]: [NodeId; 3] =
        snapshot.node(body).unwrap().children().try_into().unwrap();
    let [replacement]: [NodeId; 1] = snapshot
        .node(origin)
        .unwrap()
        .children()
        .try_into()
        .unwrap();
    let [watched]: [NodeId; 1] = snapshot
        .node(destination)
        .unwrap()
        .children()
        .try_into()
        .unwrap();
    let raw = |node, attribute: &str| {
        snapshot
            .node(node)
            .unwrap()
            .attribute_ns(&Namespace::none(), &attribute.into())
    };
    let exact = DocumentString::from_utf16(
        " Raw"
            .encode_utf16()
            .chain([0xd800, 0, 0xdc00, 32])
            .collect::<Vec<_>>(),
    );
    for attribute in ["accept-charset", "name", "target", "rel"] {
        assert_eq!(raw(ordinary, attribute), Some(&exact));
    }
    assert_eq!(raw(ordinary, "autocomplete"), Some(&"OfF".into()));
    assert_eq!(raw(ordinary, "method"), Some(&"PoSt".into()));
    assert_eq!(raw(ordinary, "novalidate"), None);
    assert_eq!(raw(ordinary, "enctype"), Some(&"".into()));
    assert_eq!(raw(ordinary, "encoding"), Some(&"unrelated content".into()));
    assert_eq!(raw(replacement, "enctype"), Some(&"text/plain".into()));
    let invalid = "Bad"
        .encode_utf16()
        .chain([0xd800, 0, 0xdc00])
        .collect::<Vec<_>>();
    let callback = "callback"
        .encode_utf16()
        .chain([0xd800])
        .collect::<Vec<_>>();
    assert_eq!(
        raw(watched, "enctype"),
        Some(&DocumentString::from_utf16(invalid))
    );
    assert_eq!(
        raw(watched, "name"),
        Some(&DocumentString::from_utf16(callback))
    );
    assert_eq!(raw(watched, "encoding"), None);
    assert_eq!(snapshot.incomplete_source_operation(), None);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
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
