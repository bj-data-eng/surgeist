//! The attached Fieldset uses canonical form association and custom validity.

use super::*;

pub(super) fn attached_fieldset_reads_native_form_and_stores_custom_validity() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const prototype = HTMLFieldSetElement.prototype;
        const attributes = [
            ['disabled', true], ['form', false], ['name', true], ['type', false],
            ['willValidate', false], ['validationMessage', false],
        ];
        const methods = [['checkValidity', 0], ['reportValidity', 0], ['setCustomValidity', 1]];
        const callbacks = [];
        for (const [name, writable] of attributes) {
            const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof descriptor?.get !== 'function') {
                return 'missing HTMLFieldSetElement.prototype.' + name + ' getter';
            }
            if (descriptor.get.name !== 'get ' + name || descriptor.get.length !== 0 ||
                !descriptor.enumerable || !descriptor.configurable ||
                (writable ? typeof descriptor.set !== 'function' || descriptor.set.length !== 1 ||
                    descriptor.set.name !== 'set ' + name : descriptor.set !== undefined)) {
                return name + ' Fieldset accessor descriptor';
            }
            callbacks.push(descriptor.get);
            if (writable) callbacks.push(descriptor.set);
        }
        for (const [name, length] of methods) {
            const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof descriptor?.value !== 'function' || descriptor.value.name !== name ||
                descriptor.value.length !== length || !descriptor.writable ||
                !descriptor.enumerable || !descriptor.configurable) return name + ' Fieldset method descriptor';
            callbacks.push(descriptor.value);
        }
        const names = [...attributes, ...methods].map(([name]) => name);
        if (Object.getOwnPropertyNames(prototype).filter(name => names.includes(name)).join() !== names.join()) {
            return 'Fieldset declaration order';
        }
        const html = 'http://www.w3.org/1999/xhtml';
        const native = document.createElement('fieldset');
        const prefixed = document.createElementNS(html, 'h:fieldset');
        const upper = document.createElementNS(html, 'FIELDSET');
        const foreign = document.createElementNS('urn:foreign', 'fieldset');
        Object.setPrototypeOf(foreign, prototype);
        let conversions = 0;
        const argument = { [Symbol.toPrimitive]() { conversions++; return 'premature'; } };
        for (const callback of callbacks) {
            for (const receiver of [document.body, upper, foreign, Object.create(prototype), new Proxy(native, {})]) {
                try {
                    Reflect.apply(callback, receiver, [argument]);
                    return callback.name + ' accepted a non-Fieldset receiver';
                } catch (error) {
                    if (!(error instanceof TypeError) || conversions !== 0) return callback.name + ' native brand first';
                }
            }
        }
        if (Object.getPrototypeOf(native) !== prototype || Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
            !(prefixed instanceof HTMLFieldSetElement) || native.form !== null || prefixed.form !== null) {
            return 'exact detached and prefixed Fieldset receivers';
        }
        prefixed.setAttributeNS(null, 'type', 'authored');
        prefixed.setAttributeNS('urn:foreign', 'f:name', 'foreign');
        prefixed.setAttributeNS(null, 'NAME', 'uppercase');
        prefixed.setAttributeNS('urn:foreign', 'f:disabled', '');
        if (prefixed.type !== 'fieldset' || prefixed.name !== '' || prefixed.disabled) return 'intrinsic type and exact reflections';
        prefixed.setAttributeNS(null, 'disabled', 'false');
        if (!prefixed.disabled) return 'disabled reflects presence';
        prefixed.disabled = argument;
        if (!prefixed.disabled || prefixed.getAttributeNS(null, 'disabled') !== '' || conversions !== 0) {
            return 'disabled setter uses Web IDL boolean';
        }
        prefixed.disabled = false;
        prefixed.name = null;
        return !prefixed.disabled && prefixed.getAttributeNS(null, 'disabled') === null &&
            prefixed.getAttributeNS('urn:foreign', 'disabled') === '' && prefixed.name === 'null' &&
            prefixed.getAttributeNS(null, 'NAME') === 'uppercase' &&
            prefixed.getAttributeNS('urn:foreign', 'name') === 'foreign'
            ? 'ok' : 'exact null-namespace Fieldset mutations';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const place = document.body.appendChild(document.createElement('div'));
        const first = place.appendChild(document.createElement('form'));
        const next = place.appendChild(document.createElement('form'));
        const ancestor = place.appendChild(document.createElement('form'));
        const ownerId = 'fieldset-owner-\ud800';
        first.setAttributeNS(null, 'id', ownerId);
        const calls = [];
        class ObservedFieldset extends HTMLFieldSetElement {
            static observedAttributes = ['disabled', 'name', 'form'];
            attributeChangedCallback(attribute, previous, value, namespace) {
                calls.push([this, attribute, previous, value, namespace, this.form, this.disabled, this.name]);
            }
        }
        customElements.define('x-observed-fieldset', ObservedFieldset, {extends: 'fieldset'});
        const fieldset = ancestor.appendChild(document.createElement('fieldset', {is: 'x-observed-fieldset'}));
        const spare = ancestor.appendChild(document.createElement('fieldset'));
        const descendant = fieldset.appendChild(document.createElement('input'));
        descendant.setAttribute('required', '');
        let invalidEvents = 0;
        fieldset.addEventListener('invalid', () => invalidEvents++, true);
        if (!(fieldset instanceof ObservedFieldset) || fieldset.form !== ancestor || spare.form !== ancestor) {
            return 'actual Fieldset ancestor Form identity';
        }
        const exactName = ' name\ud800\0\udc00 ';
        fieldset.disabled = true;
        fieldset.name = exactName;
        fieldset.disabled = false;
        fieldset.setAttributeNS(null, 'form', ownerId);
        const expected = [
            ['disabled', null, '', null, ancestor, true, ''],
            ['name', null, exactName, null, ancestor, true, exactName],
            ['disabled', '', null, null, ancestor, false, exactName],
            ['form', null, ownerId, null, first, false, exactName],
        ];
        if (calls.length !== expected.length || calls.some((call, index) =>
            call[0] !== fieldset || expected[index].some((value, offset) => call[offset + 1] !== value)) ||
            fieldset.form !== first || fieldset.form !== fieldset.form || !(fieldset.form instanceof HTMLFormElement)) {
            return 'synchronous reflection and currently published Form association';
        }
        if (fieldset.setCustomValidity(null) !== undefined || spare.setCustomValidity(undefined) !== undefined) {
            return 'setCustomValidity returns undefined';
        }
        globalThis.fieldsetProbe = {first, next, ancestor, fieldset, spare, calls, invalidEvents: () => invalidEvents};
        return 'ok';
    })()"#,
    );

    let initial = host.snapshot().unwrap();
    let root = initial
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = initial.node(root).unwrap().children()[1];
    let place = initial.node(body).unwrap().children()[0];
    let [first, next, ancestor]: [NodeId; 3] =
        initial.node(place).unwrap().children().try_into().unwrap();
    let [fieldset, spare]: [NodeId; 2] = initial
        .node(ancestor)
        .unwrap()
        .children()
        .try_into()
        .unwrap();
    assert_eq!(fieldset_owner(&initial, fieldset), Some(first));
    assert_eq!(
        fieldset_state(&initial, fieldset).custom_validity_message(),
        &DocumentString::from("null")
    );
    assert_eq!(
        fieldset_state(&initial, spare).custom_validity_message(),
        &DocumentString::from("undefined")
    );

    let owner_id = initial
        .node(first)
        .unwrap()
        .attribute_ns(&Namespace::none(), &"id".into())
        .unwrap()
        .clone();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(first, Namespace::none(), "id", "removed-owner-id");
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(fieldset_owner(&host.snapshot().unwrap(), fieldset), None);
    observe(
        &mut host,
        &permit,
        "fieldsetProbe.fieldset.form === null ? 'ok' : 'the native ID change removes the Form owner'",
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(next, Namespace::none(), "id", owner_id);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(
        fieldset_owner(&host.snapshot().unwrap(), fieldset),
        Some(next)
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {fieldset, spare, ancestor, next, calls, invalidEvents} = fieldsetProbe;
        if (fieldset.form !== next || fieldset.form !== fieldset.form || spare.form !== ancestor) {
            return 'native ID changes select the original Form wrappers';
        }
        const setter = HTMLFieldSetElement.prototype.setCustomValidity;
        try {
            setter.call(spare);
            return 'setCustomValidity accepted missing required DOMString';
        } catch (error) {
            if (!(error instanceof TypeError)) return 'setCustomValidity missing argument TypeError';
        }
        const thrown = {};
        try {
            setter.call(spare, {toString() { throw thrown; }});
            return 'setCustomValidity accepted abrupt conversion';
        } catch (error) {
            if (error !== thrown) return 'setCustomValidity original conversion exception';
        }
        let receiver = fieldset;
        const hints = [];
        const result = setter.call(receiver, { [Symbol.toPrimitive](hint) {
            hints.push(hint);
            receiver = spare;
            next.appendChild(fieldset);
            fieldset.name = 'reentered\udc00';
            return 'first\r\nsecond\r\ud800\0\udc00';
        } });
        const last = calls.at(-1);
        if (result !== undefined || hints.join() !== 'string' || fieldset.parentNode !== next ||
            fieldset.form !== next || spare.form !== ancestor || calls.length !== 5 ||
            last[0] !== fieldset || last[1] !== 'name' || last[3] !== 'reentered\udc00' || last[5] !== next) {
            return 'setCustomValidity reentry keeps original Fieldset and current native Form';
        }
        const excess = {toString() { throw new Error('unused argument was converted'); }};
        return !fieldset.disabled && fieldset.willValidate === false && fieldset.validationMessage === '' &&
            fieldset.checkValidity(excess) === true && fieldset.reportValidity(excess) === true &&
            invalidEvents() === 0 ? 'ok' : 'intrinsic Fieldset validation early returns despite a custom message';
    })()"#,
    );

    let final_state = host.snapshot().unwrap();
    assert_eq!(final_state.node(fieldset).unwrap().parent(), Some(next));
    assert_eq!(final_state.node(spare).unwrap().parent(), Some(ancestor));
    assert_eq!(fieldset_owner(&final_state, fieldset), Some(next));
    assert_eq!(fieldset_owner(&final_state, spare), Some(ancestor));
    assert_eq!(fieldset_owner(&initial, fieldset), Some(first));
    let name = "reentered"
        .encode_utf16()
        .chain([0xdc00])
        .collect::<Vec<_>>();
    assert_eq!(
        final_state
            .node(fieldset)
            .unwrap()
            .attribute_ns(&Namespace::none(), &"name".into()),
        Some(&DocumentString::from_utf16(name)),
    );
    let expected = "first\nsecond\n"
        .encode_utf16()
        .chain([0xd800, 0, 0xdc00])
        .collect::<Vec<_>>();
    assert_eq!(
        fieldset_state(&final_state, fieldset)
            .custom_validity_message()
            .code_units(),
        expected.as_slice(),
    );
    assert_eq!(
        fieldset_state(&final_state, spare).custom_validity_message(),
        &DocumentString::from("undefined")
    );
    assert_eq!(final_state.incomplete_source_operation(), None);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
}

fn fieldset_owner(snapshot: &DomSnapshot, node: NodeId) -> Option<NodeId> {
    snapshot
        .node(node)
        .unwrap()
        .as_fieldset()
        .unwrap()
        .form_owner()
        .unwrap()
}

fn fieldset_state(snapshot: &DomSnapshot, node: NodeId) -> &surgeist_dom::form::DomFieldsetState {
    snapshot.node(node).unwrap().as_fieldset().unwrap().state()
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
