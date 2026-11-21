//! The Frameset's raw content-attribute reflections in the attached Window.

use super::*;

pub(super) fn attached_frameset_reflects_exact_strings_on_the_original_element() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 100_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const type = HTMLFrameSetElement;
            const prototype = type.prototype;
            if (typeof Object.getOwnPropertyDescriptor(prototype, 'cols')?.get !== 'function') {
                return 'missing HTMLFrameSetElement.prototype.cols getter';
            }
            const old = document.createElement('frameset');
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:frameset');
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', 'FRAMESET');
            const foreign = document.createElementNS('urn:foreign', 'frameset');
            const body = document.body;
            if (!(body instanceof HTMLBodyElement) || Object.getPrototypeOf(old) !== prototype ||
                Object.getPrototypeOf(type) !== HTMLElement || Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
                !(prefixed instanceof type) || prefixed.prefix !== 'p' || old.parentNode !== null ||
                'cols' in body || 'rows' in body || 'cols' in foreign || 'rows' in foreign) {
                return 'exact native Frameset fixtures';
            }
            Object.setPrototypeOf(foreign, prototype);
            const throwsTypeError = invoke => {
                try { invoke(); } catch (error) { return error instanceof TypeError; }
                return false;
            };
            const keys = Object.keys(prototype);
            if (keys.indexOf('cols') < 0 || keys.indexOf('cols') >= keys.indexOf('rows')) {
                return 'pinned Frameset accessor order';
            }
            for (const member of ['cols', 'rows']) {
                const d = Object.getOwnPropertyDescriptor(prototype, member);
                if (typeof d?.get !== 'function' || typeof d.set !== 'function' ||
                    d.get.name !== 'get ' + member || d.set.name !== 'set ' + member ||
                    d.get.length !== 0 || d.set.length !== 1 || !d.enumerable || !d.configurable ||
                    'value' in d || 'writable' in d) return 'ordinary descriptor for ' + member;
                if (d.get.call(old) !== '' || d.set.call(prefixed, 'raw') !== undefined ||
                    d.get.call(prefixed) !== 'raw' || prefixed.getAttributeNS(null, member) !== 'raw') {
                    return 'native detached and prefixed Frameset for ' + member;
                }
                for (const accessor of [d.get, d.set]) {
                    if (Object.hasOwn(accessor, 'prototype') ||
                        !throwsTypeError(() => Reflect.construct(function () {}, [], accessor))) {
                        return 'nonconstructible Frameset accessor';
                    }
                    for (const receiver of [null, undefined, {}, Object.create(prototype), new Proxy(old, {}),
                        body, foreign, upper]) {
                        let coerced = false;
                        const value = { [Symbol.toPrimitive]() { coerced = true; return 'wrong receiver'; } };
                        if (!throwsTypeError(() => Reflect.apply(accessor, receiver, [value])) || coerced) {
                            return 'Frameset brand before value conversion';
                        }
                    }
                }
            }
            document.body = old;
            if (document.body !== old || old.parentNode !== document.documentElement || body.parentNode !== null) {
                return 'Frameset actually selected by Document.body';
            }
            globalThis.framesetProbe = old;
            return 'ok';
        })()"#,
    );

    let snapshot = host.snapshot().unwrap();
    let root = snapshot
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let old = snapshot.node(root).unwrap().children()[1];
    assert_eq!(
        snapshot
            .node(old)
            .unwrap()
            .as_element()
            .unwrap()
            .name()
            .local_name(),
        &DocumentString::from("frameset")
    );
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const old = framesetProbe;
            const exact = ' 20%,*,\ud800\0\udc00 ';
            for (const member of ['cols', 'rows']) {
                const d = Object.getOwnPropertyDescriptor(HTMLFrameSetElement.prototype, member);
                old.setAttributeNS('urn:foreign', 'x:' + member, 'foreign');
                old.setAttributeNS(null, member.toUpperCase(), 'uppercase');
                if (old[member] !== '' || old.getAttributeNS(null, member) !== null) {
                    return 'exact lowercase and null namespace for ' + member;
                }
                old.setAttributeNS(null, member, exact);
                if (old[member] !== exact) return 'raw native value for ' + member;
                for (const [input, expected] of [[null, 'null'], [undefined, 'undefined'], ['', ''], [exact, exact]]) {
                    old[member] = input;
                    if (old[member] !== expected || old.getAttributeNS(null, member) !== expected) {
                        return 'ordinary DOMString setter for ' + member;
                    }
                }
                if (Reflect.apply(d.set, old, []) !== undefined || old[member] !== 'undefined' ||
                    old.getAttributeNS(null, member) !== 'undefined') return 'omitted Frameset setter argument';
                old.removeAttributeNS(null, member);
                if (old[member] !== '' || old.getAttributeNS(null, member) !== null ||
                    old.getAttributeNS(null, member.toUpperCase()) !== 'uppercase' ||
                    old.getAttributeNS('urn:foreign', member) !== 'foreign') return 'missing or foreign Frameset values';
            }
            return 'ok';
        })()"#,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(old, Namespace::none(), "cols", " invalid 7% , * ");
        batch.set_attribute_ns(
            old,
            Namespace::none(),
            "rows",
            DocumentString::from_utf16([0xd800, 0, 0xdc00]),
        );
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const old = framesetProbe;
            if (document.body !== old || old.cols !== ' invalid 7% , * ' || old.rows !== '\ud800\0\udc00') {
                return 'current native values on the existing selected Frameset';
            }
            const replacement = document.createElement('frameset');
            replacement.cols = 'replacement';
            const hints = [];
            const raw = ' original\ud800\0\udc00 ';
            old.cols = { [Symbol.toPrimitive](hint) {
                hints.push(hint);
                if (document.body !== old || old.cols !== ' invalid 7% , * ') throw Error('edit reached before conversion');
                document.body = replacement;
                return raw;
            } };
            return hints.join() === 'string' && document.body === replacement && old.parentNode === null &&
                old.cols === raw && old.getAttributeNS(null, 'cols') === raw && replacement.cols === 'replacement'
                ? 'ok' : 'coercion reentry writes the original detached Frameset';
        })()"#,
    );
    let snapshot = host.snapshot().unwrap();
    assert_eq!(snapshot.node(old).unwrap().parent(), None);
    let expected = " original"
        .encode_utf16()
        .chain([0xd800, 0, 0xdc00, 0x20])
        .collect::<Vec<_>>();
    assert_eq!(
        snapshot
            .node(old)
            .unwrap()
            .attribute_ns(&Namespace::none(), &"cols".into())
            .unwrap()
            .code_units(),
        expected.as_slice(),
    );
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const old = framesetProbe;
            const thrown = {};
            try { old.rows = { toString() { throw thrown; } }; return 'abrupt conversion returned'; }
            catch (error) { if (error !== thrown) return 'original conversion exception'; }
            try { old.rows = Symbol(); return 'Symbol accepted by DOMString'; }
            catch (error) { if (!(error instanceof TypeError)) return 'Symbol conversion exception'; }
            return old.rows === '\ud800\0\udc00' ? 'ok' : 'attribute changed after failed conversion';
        })()"#,
    );
    assert_eq!(host.snapshot().unwrap().stamp(), snapshot.stamp());

    observe(
        &mut host,
        &permit,
        r#"(() => {
            const calls = [];
            const replacement = document.createElement('body');
            let replace = false;
            class ObservedFrameset extends HTMLFrameSetElement {
                static observedAttributes = ['cols', 'rows'];
                attributeChangedCallback(name, previous, value, namespace) {
                    calls.push([this, name, previous, value, namespace, this.getAttributeNS(null, name), this[name]]);
                    if (replace) { replace = false; document.body = replacement; }
                }
            }
            customElements.define('x-frameset-observed', ObservedFrameset, {extends: 'frameset'});
            const frameset = document.createElement('frameset', {is: 'x-frameset-observed'});
            document.body = frameset;
            if (!(frameset instanceof ObservedFrameset) || document.body !== frameset) return 'actual customized Frameset';
            frameset.cols = null;
            frameset.rows = '*\ud800\0';
            frameset.rows = '*\ud800\0';
            replace = true;
            frameset.cols = 'original receiver';
            const expected = [
                ['cols', null, 'null', null, 'null', 'null'],
                ['rows', null, '*\ud800\0', null, '*\ud800\0', '*\ud800\0'],
                ['rows', '*\ud800\0', '*\ud800\0', null, '*\ud800\0', '*\ud800\0'],
                ['cols', 'null', 'original receiver', null, 'original receiver', 'original receiver'],
            ];
            return calls.every(call => call[0] === frameset) &&
                JSON.stringify(calls.map(call => call.slice(1))) === JSON.stringify(expected) &&
                frameset.parentNode === null && frameset.cols === 'original receiver' && document.body === replacement
                ? 'ok' : 'synchronous customized Frameset reactions on the original receiver';
        })()"#,
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
