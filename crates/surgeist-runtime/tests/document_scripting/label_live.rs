//! Attached labels resolve their current control and its stored Form owner.

use super::*;

pub(super) fn attached_labels_track_their_live_control_and_form() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    observe(
        &mut host,
        &permit,
        r#"(() => {
        const prototype = globalThis.HTMLLabelElement?.prototype;
        const form = prototype && Object.getOwnPropertyDescriptor(prototype, 'form');
        if (typeof form?.get !== 'function') return 'missing HTMLLabelElement.prototype.form getter';
        const htmlFor = Object.getOwnPropertyDescriptor(prototype, 'htmlFor');
        const control = Object.getOwnPropertyDescriptor(prototype, 'control');
        const constructor = Object.getOwnPropertyDescriptor(prototype, 'constructor');
        if (Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
            Object.getOwnPropertyNames(prototype).join() !== 'form,htmlFor,control,constructor' ||
            constructor.value !== HTMLLabelElement || constructor.enumerable ||
            !constructor.writable || !constructor.configurable || constructor.value.length !== 0) {
            return 'Label own member order and constructor';
        }
        const html = 'http://www.w3.org/1999/xhtml';
        const plain = document.createElement('label');
        const prefixed = document.createElementNS(html, 'p:label');
        if (Object.getPrototypeOf(plain) !== prototype || Object.getPrototypeOf(prefixed) !== prototype ||
            form.get.call(prefixed) !== null || control?.get.call(prefixed) !== null ||
            htmlFor?.get.call(prefixed) !== '') return 'native detached and prefixed Label brand';

        const throwsTypeError = action => {
            try { action(); } catch (error) { return error instanceof TypeError; }
            return false;
        };
        const foreign = document.createElementNS('urn:foreign', 'label');
        Object.setPrototypeOf(foreign, prototype);
        const invalid = [null, {}, Object.create(prototype), new Proxy(plain, {}), document.body,
            foreign, document.createElementNS(html, 'LABEL')];
        for (const [name, d, writable] of [['form', form, false], ['htmlFor', htmlFor, true], ['control', control, false]]) {
            if (typeof d?.get !== 'function' || d.get.name !== 'get ' + name || d.get.length !== 0 ||
                !d.enumerable || !d.configurable || 'value' in d ||
                (writable ? typeof d.set !== 'function' || d.set.name !== 'set htmlFor' || d.set.length !== 1
                    : d.set !== undefined)) return 'Label accessor descriptor for ' + name;
            for (const accessor of writable ? [d.get, d.set] : [d.get]) {
                if (!throwsTypeError(() => Reflect.construct(function () {}, [], accessor))) {
                    return 'nonconstructible Label accessor';
                }
                for (const target of invalid) {
                    let coerced = false;
                    const value = {toString() { coerced = true; return 'wrong receiver'; }};
                    if (!throwsTypeError(() => Reflect.apply(accessor, target, [value])) || coerced) {
                        return 'native Label brand before conversion';
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
        const add = (parent, local) => parent.appendChild(document.createElement(local));
        const zone = add(document.body, 'div');
        const label = add(zone, 'label');
        const inner = add(add(label, 'span'), 'input');
        const firstForm = add(zone, 'form');
        const nextForm = add(zone, 'form');
        const first = add(firstForm, 'input');
        const next = add(nextForm, 'input');
        const id = 'Label-target-\ud800';
        nextForm.setAttribute('id', 'label-next-form');
        first.setAttribute('id', id);
        next.setAttribute('id', id);
        label.setAttributeNS('urn:foreign', 'p:for', id);
        label.setAttributeNS(null, 'FOR', id);
        if (label.htmlFor !== '' || label.control !== inner || label.form !== null) {
            return 'missing exact for uses the first strict labelable descendant';
        }
        inner.setAttribute('id', '');
        label.htmlFor = '';
        if (label.getAttributeNS(null, 'for') !== '' || label.control !== null || label.form !== null) {
            return 'present empty for does not fall back to descendants';
        }
        label.htmlFor = 'label-target-\ud800';
        if (label.control !== null) return 'ID matches exact code units and case';
        label.htmlFor = id;
        if (label.control !== first || label.form !== firstForm) return 'first matching control and stored owner';

        const hidden = document.createElement('input');
        hidden.type = 'HiDdEn';
        hidden.setAttribute('id', id);
        const foreign = document.createElementNS('urn:foreign', 'input');
        foreign.setAttributeNS('urn:foreign', 'p:id', id);
        zone.insertBefore(foreign, label);
        if (label.control !== first) return 'namespaced id does not participate';
        zone.insertBefore(hidden, label);
        if (label.control !== null || label.form !== null) return 'first hidden Input blocks a later match';
        hidden.type = 'text';
        if (label.control !== hidden || label.form !== null) return 'actual Input type changes labelability';
        foreign.setAttributeNS(null, 'id', id);
        if (label.control !== null || label.form !== null) return 'first foreign ID match blocks a later match';
        zone.removeChild(foreign);
        zone.removeChild(hidden);
        if (label.control !== first || label.form !== firstForm) return 'later duplicate becomes the exact control';

        first.setAttribute('form', 'label-next-form');
        if (label.control !== first || label.form !== nextForm || !(label.form instanceof HTMLFormElement)) {
            return 'current nonancestor Form wrapper is used';
        }
        first.setAttribute('form', 'no-matching-form');
        if (label.control !== first || label.form !== null) return 'owner loss retains labeled control';
        first.removeAttribute('form');
        if (label.control !== first || label.form !== firstForm) return 'removing form restores the ancestor owner';
        globalThis.labelProbe = {zone, label, id, first, firstForm, nextForm};
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
    let zone = *before.node(body).unwrap().children().last().unwrap();
    let [_, first_form, next_form]: [NodeId; 3] =
        before.node(zone).unwrap().children().try_into().unwrap();
    let [first]: [NodeId; 1] = before
        .node(first_form)
        .unwrap()
        .children()
        .try_into()
        .unwrap();
    assert_eq!(input_form(&before, first), Some(first_form));
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(first, Namespace::none(), "form", "label-next-form");
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(
        input_form(&host.snapshot().unwrap(), first),
        Some(next_form)
    );
    assert_eq!(input_form(&before, first), Some(first_form));
    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {label, first, nextForm} = labelProbe;
        return label.control === first && label.form === nextForm ? 'ok' :
            'original Label wrapper observes native owner reassociation';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const add = (parent, local) => parent.appendChild(document.createElement(local));
        const fragment = document.createDocumentFragment();
        const label = add(fragment, 'label');
        const form = add(fragment, 'form');
        const meter = add(form, 'meter');
        const progress = add(form, 'progress');
        meter.setAttribute('id', 'detached-meter');
        progress.setAttribute('id', 'detached-progress');
        label.htmlFor = 'detached-meter';
        if (label.control !== meter || label.form !== null) return 'same detached tree Meter has no Form owner';
        label.htmlFor = 'detached-progress';
        if (label.control !== progress || label.form !== null) return 'same detached tree Progress has no Form owner';
        fragment.removeChild(label);
        if (label.control !== null || label.form !== null) return 'separate ordinary trees do not share an ID';

        const lightLabel = add(document.body, 'label');
        const shadowHost = add(lightLabel, 'div');
        const shadow = shadowHost.attachShadow({mode: 'open'});
        const shadowLabel = add(shadow, 'label');
        const shadowInput = add(shadow, 'input');
        shadowInput.setAttribute('id', 'label-shadow-only');
        const lightInput = add(lightLabel, 'input');
        lightInput.setAttribute('id', 'label-light-only');
        shadowLabel.htmlFor = 'label-shadow-only';
        if (shadowLabel.control !== shadowInput || lightLabel.control !== lightInput) {
            return 'same shadow ID and implicit ordinary descendant selection';
        }
        shadowLabel.htmlFor = 'label-light-only';
        lightLabel.htmlFor = 'label-shadow-only';
        if (shadowLabel.control !== null || lightLabel.control !== null) return 'shadow boundary cuts both ID searches';

        const {firstForm} = labelProbe;
        const faceLabel = add(firstForm, 'label');
        const ordinary = add(faceLabel, 'x-label-ordinary');
        const face = add(faceLabel, 'x-label-face');
        customElements.define('x-label-ordinary', class extends HTMLElement {});
        if (faceLabel.control !== null || faceLabel.form !== null) return 'unqualified custom descendants are not labelable';
        class Face extends HTMLElement { static formAssociated = true; }
        customElements.define('x-label-face', Face);
        return face instanceof Face && ordinary.parentNode === faceLabel && faceLabel.control === face &&
            faceLabel.form === firstForm ? 'ok' : 'successful autonomous FACE is labelable with its stored Form';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {zone, label, id, first, nextForm} = labelProbe;
        const calls = [];
        class ObservedLabel extends HTMLLabelElement {
            static observedAttributes = ['for'];
            attributeChangedCallback(name, old, value, namespace) {
                calls.push([this, name, old, value, namespace, this.htmlFor, this.control, this.form]);
            }
        }
        customElements.define('x-observed-label', ObservedLabel, {extends: 'label'});
        const observed = zone.appendChild(document.createElement('label', {is: 'x-observed-label'}));
        const set = Object.getOwnPropertyDescriptor(HTMLLabelElement.prototype, 'htmlFor').set;
        if (!(observed instanceof ObservedLabel)) return 'customized native Label';
        observed.htmlFor = null;
        Reflect.apply(set, observed, []);
        let coercions = 0;
        const raw = ' left\ud800\0\udc00 ';
        observed.htmlFor = {[Symbol.toPrimitive](hint) {
            if (hint !== 'string' || observed.htmlFor !== 'undefined') throw Error('wrong label conversion');
            coercions++;
            zone.insertBefore(observed, label);
            return raw;
        }};
        observed.htmlFor = id;
        const expected = [[null, 'null'], ['null', 'undefined'], ['undefined', raw], [raw, id]];
        if (coercions !== 1 || calls.length !== expected.length || observed.nextSibling !== label) {
            return 'synchronous for reactions on the original receiver after reentry';
        }
        for (const [index, [old, value]] of expected.entries()) {
            const call = calls[index];
            const target = value === id ? first : null;
            const owner = value === id ? nextForm : null;
            if (call[0] !== observed || call[1] !== 'for' || call[2] !== old || call[3] !== value ||
                call[4] !== null || call[5] !== value || call[6] !== target || call[7] !== owner) {
                return 'exact for callback payload and current association';
            }
        }
        try { observed.htmlFor = Symbol('invalid'); return 'Symbol converted to DOMString'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        return calls.length === expected.length && observed.getAttributeNS(null, 'for') === id &&
            label.getAttributeNS('urn:foreign', 'for') === id && label.getAttributeNS(null, 'FOR') === id
            ? 'ok' : 'failed conversion leaves reflection and same-spelled attributes intact';
    })()"#,
    );
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

fn input_form(snapshot: &DomSnapshot, input: NodeId) -> Option<NodeId> {
    snapshot
        .node(input)
        .unwrap()
        .as_input()
        .unwrap()
        .form_owner()
        .unwrap()
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
