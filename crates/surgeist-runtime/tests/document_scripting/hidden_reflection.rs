//! HTMLElement.hidden over the attached Window and its canonical attributes.

use super::*;

pub(super) fn attached_hidden_reflects_the_native_attribute_and_web_idl_union() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 100_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const prototype = HTMLElement.prototype;
            const d = Object.getOwnPropertyDescriptor(prototype, 'hidden');
            if (typeof d?.get !== 'function') return 'missing HTMLElement.prototype.hidden getter';
            if (typeof d.set !== 'function' || d.get.name !== 'get hidden' || d.get.length !== 0 ||
                d.set.name !== 'set hidden' || d.set.length !== 1 || !d.enumerable || !d.configurable ||
                'value' in d || 'writable' in d) return 'ordinary hidden accessor descriptor';
            const div = document.createElement('div');
            const foreign = document.createElementNS('urn:foreign', 'div');
            if (Object.getPrototypeOf(div) !== HTMLDivElement.prototype ||
                Object.hasOwn(HTMLDivElement.prototype, 'hidden') ||
                Object.hasOwn(Element.prototype, 'hidden') || 'hidden' in foreign ||
                div.hidden !== false || d.get.call(div) !== false) return 'native HTML inheritance';
            const throwsTypeError = invoke => {
                try { invoke(); } catch (error) { return error instanceof TypeError; }
                return false;
            };
            for (const accessor of [d.get, d.set]) {
                if (Object.hasOwn(accessor, 'prototype') ||
                    !throwsTypeError(() => Reflect.construct(function () {}, [], accessor))) {
                    return 'hidden accessors must be nonconstructible';
                }
                for (const receiver of [null, undefined, {}, Object.create(prototype), new Proxy(div, {}), foreign]) {
                    let coerced = false;
                    const input = { [Symbol.toPrimitive]() { coerced = true; return 'until-found'; } };
                    if (!throwsTypeError(() => Reflect.apply(accessor, receiver, [input])) || coerced) {
                        return 'hidden must reject a non-HTMLElement receiver before conversion';
                    }
                }
            }
            return 'ok';
        })()"#,
    );

    let document = top.document();
    let node = host
        .with_native(|dom, _, _| {
            let root = dom
                .snapshot()
                .document(document)
                .unwrap()
                .document_element()
                .unwrap();
            let mut batch = DomBatch::new();
            let node = batch.create_element_ns(document, Namespace::html(), "div");
            batch.set_attribute_ns(
                &node,
                Namespace::from_uri("urn:foreign"),
                "x:hidden",
                "until-found",
            );
            batch.insert_before(&node, root, None);
            dom.apply(batch, &cancel())
                .unwrap()
                .created(&node)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const live = document.documentElement.lastChild;
            if (!(live instanceof HTMLDivElement) || live.hidden !== false ||
                live.getAttributeNS('urn:foreign', 'hidden') !== 'until-found' ||
                live.getAttributeNS(null, 'hidden') !== null) return 'native null-namespace fixture';
            globalThis.hiddenNativeProbe = live;
            for (const [raw, expected] of [
                ['uNTIL-fOUND', 'until-found'], ['', true], ['HIDDEN', true], ['false', true],
                ['until-found ', true], ['until-found\0', true], ['\ud800\0\udc00', true],
            ]) {
                live.setAttributeNS(null, 'hidden', raw);
                if (live.hidden !== expected || live.getAttributeNS(null, 'hidden') !== raw) {
                    return 'getter state and untouched raw UTF-16 attribute';
                }
            }
            for (const [label, input, expected] of [
                ['null', null, null], ['undefined', undefined, null],
                ['false', false, null], ['true', true, ''],
                ['positive zero', 0, null], ['negative zero', -0, null], ['NaN', NaN, null],
                ['positive double', 1.5, ''], ['negative double', -1, ''], ['small double', Number.MIN_VALUE, ''],
                ['positive infinity', Infinity, ''], ['negative infinity', -Infinity, ''],
                ['empty string', '', null], ['zero string', '0', ''], ['false string', 'false', ''],
                ['keyword', 'uNTIL-fOUND', 'until-found'], ['no trimming', ' until-found', ''],
                ['embedded null', 'until-found\0', ''], ['exact UTF-16', '\ud800\0\udc00', ''],
                ['boxed zero', new Number(0), ''], ['boxed NaN', new Number(NaN), ''],
                ['boxed false', new Boolean(false), ''], ['boxed empty', new String(''), null],
                ['BigInt zero', 0n, ''], ['negative BigInt', -1n, ''],
                ['empty array', [], null], ['array zero', [0], ''], ['array keyword', ['UNTIL-FOUND'], 'until-found'],
            ]) {
                live.setAttributeNS(null, 'hidden', 'before');
                live.hidden = input;
                const reflected = expected === null ? false : expected === '' ? true : expected;
                if (live.getAttributeNS(null, 'hidden') !== expected || live.hidden !== reflected) {
                    return 'hidden union branch for ' + label;
                }
            }
            const d = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'hidden');
            live.hidden = true;
            Reflect.apply(d.set, live, []);
            if (live.hidden !== false || live.hasAttributeNS(null, 'hidden')) return 'missing setter argument selects null';
            for (const primitive of [0, false, null, undefined]) {
                const hints = [];
                live.hidden = { [Symbol.toPrimitive](hint) { hints.push(hint); return primitive; } };
                if (hints.join() !== 'string' || live.hidden !== true || live.getAttributeNS(null, 'hidden') !== '') {
                    return 'DOMString branch selected before object coercion';
                }
            }
            live.hidden = { toString() { return 'UnTiL-FoUnD'; }, valueOf() { throw Error('wrong hint'); } };
            return live.hidden === 'until-found' && live.getAttributeNS('urn:foreign', 'hidden') === 'until-found'
                ? 'ok' : 'canonical setter must preserve foreign hidden';
        })()"#,
    );
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(node)
            .unwrap()
            .attribute_ns(&Namespace::none(), &"hidden".into()),
        Some(&"until-found".into()),
    );

    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(
            node,
            Namespace::none(),
            "hidden",
            DocumentString::from_utf16([0xd800, 0, 0xdc00]),
        );
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    observe(
        &mut host,
        &permit,
        r#"hiddenNativeProbe === document.documentElement.lastChild && hiddenNativeProbe.hidden === true &&
            hiddenNativeProbe.getAttributeNS(null, 'hidden') === '\ud800\0\udc00'
            ? 'ok' : 'existing wrapper must read current native UTF-16 state'"#,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.remove_attribute_ns(node, Namespace::none(), "hidden");
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    observe(
        &mut host,
        &permit,
        r#"hiddenNativeProbe.hidden === false && hiddenNativeProbe.getAttributeNS('urn:foreign', 'hidden') === 'until-found'
            ? 'ok' : 'live native removal preserves the namespaced attribute'"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
            const calls = [];
            customElements.define('x-hidden-observed', class extends HTMLElement {
                static observedAttributes = ['hidden'];
                attributeChangedCallback(name, oldValue, value, namespace) {
                    calls.push([this, name, oldValue, value, namespace, this.getAttributeNS(null, name), this.hidden]);
                }
            });
            const watched = document.createElement('x-hidden-observed');
            let conversions = 0;
            watched.hidden = true;
            watched.hidden = { [Symbol.toPrimitive](hint) {
                if (hint !== 'string' || watched.getAttributeNS(null, 'hidden') !== '') throw Error('wrong conversion order');
                conversions++;
                watched.setAttributeNS(null, 'hidden', 'nested');
                return 'UNTIL-FOUND';
            } };
            watched.hidden = false;
            watched.hidden = 'something else';
            const expected = [
                ['hidden', null, '', null, '', true],
                ['hidden', '', 'nested', null, 'nested', true],
                ['hidden', 'nested', 'until-found', null, 'until-found', 'until-found'],
                ['hidden', 'until-found', null, null, null, false],
                ['hidden', null, '', null, '', true],
            ];
            if (conversions !== 1 || !calls.every(call => call[0] === watched) ||
                JSON.stringify(calls.map(call => call.slice(1))) !== JSON.stringify(expected)) {
                return 'reentrant conversion and synchronous native reactions';
            }
            globalThis.hiddenReactionProbe = {watched, calls};
            return 'ok';
        })()"#,
    );
    let before = host.snapshot().unwrap();
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {watched, calls} = hiddenReactionProbe;
            const count = calls.length;
            const original = {};
            try {
                watched.hidden = { toString() { throw original; } };
                return 'throwing DOMString conversion returned';
            } catch (error) {
                if (error !== original) return 'hidden setter must propagate the original thrown object';
            }
            let conversions = 0;
            for (const input of [Symbol('raw'), Object(Symbol('boxed')),
                { [Symbol.toPrimitive]() { conversions++; return Symbol('converted'); } }]) {
                try { watched.hidden = input; return 'Symbol DOMString conversion succeeded'; }
                catch (error) { if (!(error instanceof TypeError)) return 'Symbol conversion must throw TypeError'; }
            }
            return conversions === 1 && calls.length === count && watched.hidden === true &&
                watched.getAttributeNS(null, 'hidden') === ''
                ? 'ok' : 'failed conversion changed the native attribute or delivered reactions';
        })()"#,
    );
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
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
