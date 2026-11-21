//! Obsolete Body content-attribute reflections in the attached Window.

use super::*;

pub(super) fn attached_body_reflections_preserve_native_identity_and_exact_strings() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const prototype = HTMLBodyElement.prototype;
            if (typeof Object.getOwnPropertyDescriptor(prototype, 'text')?.get !== 'function') {
                return 'missing HTMLBodyElement.prototype.text getter';
            }
            const body = document.body;
            const frameset = document.createElement('frameset');
            const foreign = document.createElementNS('urn:elsewhere', 'body');
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', 'BODY');
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:body');
            if (Object.getPrototypeOf(body) !== prototype || Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
                !(frameset instanceof HTMLFrameSetElement) || !(prefixed instanceof HTMLBodyElement) ||
                'text' in frameset || 'text' in foreign || 'text' in upper || 'text' in HTMLElement.prototype) {
                return 'exact Body interface membership';
            }
            document.body = frameset;
            if (document.body !== frameset || body.parentNode !== null) return 'actual selected Frameset fixture';
            Object.setPrototypeOf(foreign, prototype);
            const throwsTypeError = invoke => {
                try { invoke(); } catch (error) { return error instanceof TypeError; }
                return false;
            };
            const fields = [['text','text'], ['link','link'], ['vLink','vlink'],
                ['aLink','alink'], ['bgColor','bgcolor'], ['background','background']];
            const keys = Object.keys(prototype);
            let prior = -1;
            for (const [member, attribute] of fields) {
                const d = Object.getOwnPropertyDescriptor(prototype, member);
                if (typeof d?.get !== 'function' || typeof d.set !== 'function' || d.get.name !== 'get ' + member ||
                    d.set.name !== 'set ' + member || d.get.length !== 0 || d.set.length !== 1 ||
                    !d.enumerable || !d.configurable || 'value' in d || 'writable' in d || keys.indexOf(member) <= prior) {
                    return 'ordinary Body descriptor for ' + member;
                }
                prior = keys.indexOf(member);
                if (d.get.call(body) !== '' || d.set.call(prefixed, 'prefixed') !== undefined ||
                    prefixed.getAttributeNS(null, attribute) !== 'prefixed' || d.get.call(prefixed) !== 'prefixed') {
                    return 'exact Body brand for detached or prefixed receiver';
                }
                for (const accessor of [d.get, d.set]) {
                    if (Object.hasOwn(accessor, 'prototype') ||
                        !throwsTypeError(() => Reflect.construct(function () {}, [], accessor))) {
                        return 'Body accessors must be nonconstructible';
                    }
                    for (const other of [null, undefined, {}, document, frameset, foreign, upper,
                        Object.create(prototype), new Proxy(body, {})]) {
                        let coerced = false;
                        const value = { [Symbol.toPrimitive]() { coerced = true; return 'wrong receiver'; } };
                        if (!throwsTypeError(() => Reflect.apply(accessor, other, [value])) || coerced) {
                            return 'authenticate native Body before conversion for ' + member;
                        }
                    }
                }
            }
            document.body = body;
            if (document.body !== body) return 'restore the original Body fixture';
            globalThis.bodyReflection = {body, fields};
            return 'ok';
        })()"#,
    );

    let snapshot = host.snapshot().unwrap();
    let root = snapshot
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(root).unwrap().children()[1];
    assert_eq!(
        snapshot
            .node(body)
            .unwrap()
            .as_element()
            .unwrap()
            .name()
            .local_name(),
        &DocumentString::from("body")
    );
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {body, fields} = bodyReflection;
            for (const [member, attribute] of fields) {
                const d = Object.getOwnPropertyDescriptor(HTMLBodyElement.prototype, member);
                const exact = member === 'background' ? '../ raw\ud800\0\udc00.png?x=1#part' : ' Raw\ud800\0\udc00 ';
                body.setAttributeNS('urn:elsewhere', 'x:' + attribute, 'foreign');
                body.setAttributeNS(null, attribute.toUpperCase(), 'uppercase');
                if (body[member] !== '' || body.getAttributeNS(null, attribute) !== null) {
                    return 'qualify lowercase null-namespace Body attribute for ' + member;
                }
                body.setAttributeNS(null, attribute, exact);
                if (body[member] !== exact) return 'raw authored DOMString for ' + member;
                body[member] = '';
                if (!body.hasAttributeNS(null, attribute) || body[member] !== '') return 'explicit empty string for ' + member;
                body[member] = null;
                const fromNull = member === 'background' ? 'null' : '';
                if (body[member] !== fromNull || body.getAttributeNS(null, attribute) !== fromNull) {
                    return 'pinned null conversion for ' + member;
                }
                body[member] = undefined;
                if (body[member] !== 'undefined' || body.getAttributeNS(null, attribute) !== 'undefined') {
                    return 'ordinary undefined DOMString for ' + member;
                }
                body[member] = 'before omission';
                if (Reflect.apply(d.set, body, []) !== undefined || body[member] !== 'undefined') {
                    return 'omitted setter argument for ' + member;
                }
                body[member] = exact;
                if (body[member] !== exact || body.getAttributeNS(null, attribute) !== exact ||
                    body.getAttributeNS(null, attribute.toUpperCase()) !== 'uppercase' ||
                    body.getAttributeNS('urn:elsewhere', attribute) !== 'foreign') {
                    return 'exact UTF-16 setter and independent attribute names for ' + member;
                }
                body.removeAttributeNS(null, attribute);
                if (body[member] !== '' || body.getAttributeNS(null, attribute) !== null) return 'missing result for ' + member;
            }
            return 'ok';
        })()"#,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(
            body,
            Namespace::none(),
            "vlink",
            DocumentString::from_utf16([0xd800, 0, 0xdc00]),
        );
        batch.set_attribute_ns(
            body,
            Namespace::none(),
            "background",
            "../native raw image.svg?color=x#part",
        );
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    observe(
        &mut host,
        &permit,
        r#"bodyReflection.body === document.body && bodyReflection.body.vLink === '\ud800\0\udc00' &&
            bodyReflection.body.background === '../native raw image.svg?color=x#part'
            ? 'ok' : 'existing attached wrapper reads native exact values'"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {body} = bodyReflection;
            const replacement = document.createElement('body');
            replacement.setAttributeNS(null, 'background', 'replacement');
            const d = Object.getOwnPropertyDescriptor(HTMLBodyElement.prototype, 'background');
            const exact = './source\ud800\0\udc00.png';
            const hints = [];
            d.set.call(body, { [Symbol.toPrimitive](hint) {
                hints.push(hint);
                if (document.body !== body || body.background !== '../native raw image.svg?color=x#part') {
                    throw Error('mutation reached before conversion');
                }
                document.body = replacement;
                return exact;
            } } );
            if (hints.join() !== 'string' || document.body !== replacement || body.parentNode !== null ||
                body.background !== exact || replacement.background !== 'replacement') {
                return 'conversion reentry retains the original Body receiver';
            }
            const other = document.implementation.createHTMLDocument();
            other.body = body;
            body.bgColor = 'adopted original';
            return other.body === body && body.ownerDocument === other && document.body === replacement &&
                d.get.call(body) === exact && body.getAttributeNS(null, 'bgcolor') === 'adopted original' &&
                replacement.getAttributeNS(null, 'bgcolor') === null ? 'ok' : 'exact Body custody after adoption';
        })()"#,
    );
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(body)
            .unwrap()
            .attribute_ns(&Namespace::none(), &"bgcolor".into()),
        Some(&"adopted original".into()),
    );
    let before = host.snapshot().unwrap();
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {body, fields} = bodyReflection;
            const thrown = {};
            for (const [member, attribute] of fields) {
                const raw = body.getAttributeNS(null, attribute);
                try { body[member] = { toString() { throw thrown; } }; return 'abrupt conversion returned'; }
                catch (error) { if (error !== thrown) return 'original thrown value for ' + member; }
                try { body[member] = Symbol(); return 'Symbol converted to DOMString'; }
                catch (error) { if (!(error instanceof TypeError)) return 'Symbol conversion exception'; }
                if (body.getAttributeNS(null, attribute) !== raw) return 'mutation after failed conversion';
            }
            return 'ok';
        })()"#,
    );
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());

    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {fields} = bodyReflection;
            const calls = [];
            let reenter = false;
            let reportNext = false;
            const replacement = document.createElement('body');
            replacement.setAttributeNS(null, 'text', 'replacement');
            class ObservedBody extends HTMLBodyElement {
                static observedAttributes = fields.map(([, attribute]) => attribute);
                attributeChangedCallback(name, previous, value, namespace) {
                    const [member] = fields.find(([, attribute]) => attribute === name);
                    calls.push([this, name, previous, value, namespace, this.getAttributeNS(null, name), this[member]]);
                    if (reenter) { reenter = false; document.body = replacement; }
                    if (reportNext) { reportNext = false; throw globalThis.bodyReflectionReactionThrow = {}; }
                }
            }
            customElements.define('x-body-reflections', ObservedBody, {extends: 'body'});
            const watched = document.createElement('body', {is: 'x-body-reflections'});
            document.body = watched;
            if (!(watched instanceof ObservedBody) || document.body !== watched) return 'actual customized Body';
            for (const [member] of fields) watched[member] = null;
            watched.text = null;
            reenter = true;
            watched.text = 'outer';
            reportNext = true;
            watched.background = '../reaction\ud800.png';
            const expected = fields.map(([member, attribute]) => {
                const value = member === 'background' ? 'null' : '';
                return [attribute, null, value, null, value, value];
            }).concat([
                ['text', '', '', null, '', ''],
                ['text', '', 'outer', null, 'outer', 'outer'],
                ['background', 'null', '../reaction\ud800.png', null, '../reaction\ud800.png', '../reaction\ud800.png'],
            ]);
            return calls.every(call => call[0] === watched) &&
                JSON.stringify(calls.map(call => call.slice(1))) === JSON.stringify(expected) &&
                watched.parentNode === null && watched.text === 'outer' && watched.background === '../reaction\ud800.png' &&
                document.body === replacement && replacement.text === 'replacement'
                ? 'ok' : 'synchronous reactions retain the original Body through callback reentry';
        })()"#,
    );
    let report = host
        .take_guest_custom_element_callback_exception(&permit)
        .unwrap()
        .unwrap();
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.in_document(&permit, |scope| {
        let original = scope.evaluate(
            &"bodyReflectionReactionThrow"
                .encode_utf16()
                .collect::<Vec<_>>(),
        )?;
        let reported = scope.import(report.exception.value())?;
        assert!(scope.strict_equals(&original, &reported)?);
        Ok(())
    })
    .unwrap();
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
