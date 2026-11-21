//! Legend reflection over current attached DOM and the stored Fieldset owner.

use super::*;

pub(super) fn attached_legend_uses_the_immediate_fieldsets_current_form_owner() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const prototype = HTMLLegendElement.prototype;
            const form = Object.getOwnPropertyDescriptor(prototype, 'form');
            if (typeof form?.get !== 'function') return 'missing HTMLLegendElement.prototype.form getter';
            const align = Object.getOwnPropertyDescriptor(prototype, 'align');
            if (typeof align?.get !== 'function' || typeof align.set !== 'function' ||
                align.set.name !== 'set align' || align.set.length !== 1 || form.set !== undefined ||
                Object.getPrototypeOf(prototype) !== HTMLElement.prototype) return 'Legend descriptor shape';
            const keys = Object.keys(prototype);
            if (keys.indexOf('form') < 0 || keys.indexOf('form') >= keys.indexOf('align')) return 'Legend declaration order';
            const detached = document.createElement('legend');
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:legend');
            const foreign = document.createElementNS('urn:foreign', 'legend');
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', 'LEGEND');
            if (Object.getPrototypeOf(detached) !== prototype || !(prefixed instanceof HTMLLegendElement) ||
                form.get.call(detached) !== null || form.get.call(prefixed) !== null || align.get.call(prefixed) !== '') {
                return 'detached and prefixed native Legend';
            }
            Object.setPrototypeOf(foreign, prototype);
            const throwsTypeError = invoke => {
                try { invoke(); } catch (error) { return error instanceof TypeError; }
                return false;
            };
            for (const [member, descriptor] of [['form', form], ['align', align]]) {
                if (descriptor.get.name !== 'get ' + member || descriptor.get.length !== 0 ||
                    !descriptor.enumerable || !descriptor.configurable || 'value' in descriptor || 'writable' in descriptor) {
                    return 'ordinary Legend descriptor for ' + member;
                }
                const accessors = descriptor.set ? [descriptor.get, descriptor.set] : [descriptor.get];
                for (const accessor of accessors) {
                    if (Object.hasOwn(accessor, 'prototype') ||
                        !throwsTypeError(() => Reflect.construct(function () {}, [], accessor))) {
                        return 'nonconstructible Legend accessor';
                    }
                    for (const receiver of [null, {}, Object.create(prototype), new Proxy(detached, {}),
                        document.createElement('fieldset'), foreign, upper]) {
                        let coerced = false;
                        const input = { [Symbol.toPrimitive]() { coerced = true; return 'wrong receiver'; } };
                        if (!throwsTypeError(() => Reflect.apply(accessor, receiver, [input])) || coerced) {
                            return 'native Legend brand before string conversion';
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
            const place = document.body.appendChild(document.createElement('div'));
            const firstOwner = place.appendChild(document.createElement('form'));
            const nextOwner = place.appendChild(document.createElement('form'));
            const ancestor = place.appendChild(document.createElement('form'));
            const firstId = 'legend-owner-\ud800';
            firstOwner.setAttributeNS(null, 'id', firstId);
            nextOwner.setAttributeNS(null, 'id', 'second-owner');
            const probe = {firstOwner, nextOwner, ancestor, firstId, ownerReads: [], alignment: [], reenter: false};
            class ObservedFieldset extends HTMLFieldSetElement {
                static observedAttributes = ['form'];
                attributeChangedCallback(name, previous, value, namespace) {
                    probe.ownerReads.push([this, name, previous, value, namespace, probe.first.form, probe.second.form]);
                    if (probe.reenter) {
                        probe.reenter = false;
                        this.setAttributeNS(null, 'form', firstId);
                    }
                }
            }
            class ObservedLegend extends HTMLLegendElement {
                static observedAttributes = ['align'];
                attributeChangedCallback(name, previous, value, namespace) {
                    probe.alignment.push([this, name, previous, value, namespace, this.align, this.form]);
                }
            }
            customElements.define('x-legend-owner-fieldset', ObservedFieldset, {extends: 'fieldset'});
            customElements.define('x-observed-legend', ObservedLegend, {extends: 'legend'});
            const fieldset = ancestor.appendChild(document.createElement('fieldset', {is: 'x-legend-owner-fieldset'}));
            const first = fieldset.appendChild(document.createElement('legend'));
            const second = fieldset.appendChild(document.createElement('legend', {is: 'x-observed-legend'}));
            Object.assign(probe, {fieldset, first, second});
            if (!(fieldset instanceof ObservedFieldset) || !(second instanceof ObservedLegend) ||
                first.form !== ancestor || second.form !== ancestor) return 'both direct Legends use the ancestor association';
            fieldset.setAttributeNS(null, 'form', firstId);
            const [callback] = probe.ownerReads;
            if (probe.ownerReads.length !== 1 || callback[0] !== fieldset || callback[1] !== 'form' ||
                callback[2] !== null || callback[3] !== firstId || callback[4] !== null ||
                callback[5] !== firstOwner || callback[6] !== firstOwner || first.form !== firstOwner ||
                second.form !== firstOwner || firstOwner === ancestor || !(firstOwner instanceof HTMLFormElement)) {
                return 'nonancestor Form identity during the published Fieldset callback segment';
            }
            const nested = fieldset.appendChild(document.createElement('div')).appendChild(document.createElement('legend'));
            const optgroup = fieldset.appendChild(document.createElement('optgroup')).appendChild(document.createElement('legend'));
            const foreign = ancestor.appendChild(document.createElementNS('urn:foreign', 'p:fieldset'))
                .appendChild(document.createElement('legend'));
            const shadowHost = fieldset.appendChild(document.createElement('div'));
            const shadow = shadowHost.attachShadow({mode: 'open'}).appendChild(document.createElement('legend'));
            const directFormChild = ancestor.appendChild(document.createElement('legend'));
            if ([nested, optgroup, foreign, shadow, directFormChild].some(legend => legend.form !== null)) {
                return 'only the immediate ordinary native Fieldset qualifies';
            }
            globalThis.legendProbe = probe;
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
    let place = *initial.node(body).unwrap().children().last().unwrap();
    let [first_owner, next_owner, ancestor] = initial.node(place).unwrap().children() else {
        panic!("three canonical Form fixture nodes");
    };
    let fieldset = initial.node(*ancestor).unwrap().children()[0];
    let second_legend = initial.node(fieldset).unwrap().children()[1];
    assert_eq!(fieldset_owner(&initial, fieldset), Some(*first_owner));
    let first_id = initial
        .node(*first_owner)
        .unwrap()
        .attribute_ns(&Namespace::none(), &"id".into())
        .unwrap()
        .clone();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(*first_owner, Namespace::none(), "id", "first-owner-renamed");
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(fieldset_owner(&host.snapshot().unwrap(), fieldset), None);
    observe(
        &mut host,
        &permit,
        "legendProbe.first.form === null && legendProbe.second.form === null ? 'ok' : 'lost external owner'",
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(*next_owner, Namespace::none(), "id", first_id);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(
        fieldset_owner(&host.snapshot().unwrap(), fieldset),
        Some(*next_owner)
    );
    assert_eq!(fieldset_owner(&initial, fieldset), Some(*first_owner));
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {first, second, fieldset, firstOwner, nextOwner, firstId, ownerReads} = legendProbe;
            if (first.form !== nextOwner || second.form !== nextOwner) return 'native ID edit reopens exact Form identity';
            legendProbe.reenter = true;
            fieldset.setAttributeNS(null, 'form', 'first-owner-renamed');
            const [, changed, reentered] = ownerReads;
            if (ownerReads.length !== 3 || changed[0] !== fieldset || reentered[0] !== fieldset ||
                changed[2] !== firstId || changed[3] !== 'first-owner-renamed' ||
                changed[5] !== firstOwner || changed[6] !== firstOwner ||
                reentered[2] !== 'first-owner-renamed' || reentered[3] !== firstId ||
                reentered[5] !== nextOwner || reentered[6] !== nextOwner ||
                first.form !== nextOwner || second.form !== nextOwner) {
                return 'each callback sees its current published Fieldset association';
            }
            return 'ok';
        })()"#,
    );
    assert_eq!(
        fieldset_owner(&host.snapshot().unwrap(), fieldset),
        Some(*next_owner)
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {first, second, fieldset, nextOwner, alignment} = legendProbe;
            first.setAttributeNS('urn:foreign', 'p:align', 'foreign');
            first.setAttributeNS(null, 'ALIGN', 'uppercase');
            if (first.align !== '' || second.align !== '') return 'missing and exact null namespace alignment';
            const raw = ' left\ud800\0\udc00 ';
            first.setAttributeNS(null, 'align', raw);
            if (first.align !== raw || first.getAttributeNS('urn:foreign', 'align') !== 'foreign' ||
                first.getAttributeNS(null, 'ALIGN') !== 'uppercase') return 'exact authored alignment';
            let coercions = 0;
            second.align = null;
            second.align = { [Symbol.toPrimitive](hint) {
                if (hint !== 'string' || second.align !== 'null') throw Error('wrong alignment conversion');
                coercions++;
                return raw;
            } };
            second.align = raw;
            const expected = [[null, 'null'], ['null', raw], [raw, raw]];
            if (coercions !== 1 || alignment.length !== expected.length) return 'original synchronous alignment reactions';
            for (const [index, [oldValue, value]] of expected.entries()) {
                const call = alignment[index];
                if (call[0] !== second || call[1] !== 'align' || call[2] !== oldValue || call[3] !== value ||
                    call[4] !== null || call[5] !== value || call[6] !== nextOwner) return 'alignment reaction and live form read';
            }
            const thrown = {};
            try { second.align = {toString() { throw thrown; }}; return 'throwing alignment coercion returned'; }
            catch (error) { if (error !== thrown) return 'original alignment exception'; }
            if (alignment.length !== expected.length || second.getAttributeNS(null, 'align') !== raw) {
                return 'failed alignment conversion changed native state';
            }
            fieldset.removeChild(second);
            return second.parentNode === null && second.form === null && second.align === raw && first.form === nextOwner
                ? 'ok' : 'detached Legend loses only the parent association';
        })()"#,
    );
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(second_legend)
            .unwrap()
            .parent(),
        None
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

fn fieldset_owner(snapshot: &DomSnapshot, fieldset: NodeId) -> Option<NodeId> {
    snapshot
        .node(fieldset)
        .unwrap()
        .as_fieldset()
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
