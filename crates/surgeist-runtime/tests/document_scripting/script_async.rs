//! Native HTML Script force-async and reflected properties.

use super::*;

pub(super) fn script_async_keeps_native_flag_and_content_coherent() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let result = host
        .in_document(&permit, |scope| {
            let source = r#"(() => {
              try {
                const proto = HTMLScriptElement.prototype;
                const async = Object.getOwnPropertyDescriptor(proto, 'async');
                if (typeof async?.get !== 'function') return 'missing HTMLScriptElement.async getter';
                const fields = ['type', 'noModule', 'async', 'defer', 'integrity', 'charset', 'event', 'htmlFor'];
                if (Object.keys(proto).filter(key => fields.includes(key)).join() !== fields.join()) {
                    return 'Script declaration order';
                }

                const html = 'http://www.w3.org/1999/xhtml';
                const script = document.createElementNS(html, 'h:script');
                const foreign = document.createElementNS('urn:foreign', 'script');
                Object.setPrototypeOf(foreign, proto);
                for (const field of fields) {
                    const d = Object.getOwnPropertyDescriptor(proto, field);
                    if (!d.enumerable || !d.configurable || d.get.name !== 'get ' + field ||
                        d.get.length !== 0 || d.set.name !== 'set ' + field || d.set.length !== 1) {
                        return 'invalid descriptor ' + field;
                    }
                    for (const wrong of [document, foreign, Object.create(proto), new Proxy(script, {})]) {
                        let coerced = false;
                        try { d.get.call(wrong); return 'unbranded getter ' + field; }
                        catch (error) { if (!(error instanceof TypeError)) return 'getter brand error ' + field; }
                        try {
                            d.set.call(wrong, {toString() { coerced = true; return 'bad'; }});
                            return 'unbranded setter ' + field;
                        } catch (error) {
                            if (!(error instanceof TypeError) || coerced) return 'setter brand order ' + field;
                        }
                    }
                }

                if (!script.async || script.hasAttribute('async')) return 'force-async on a fresh Script';
                script.setAttributeNS('urn:foreign', 'q:async', 'unrelated');
                script.async = false;
                if (script.async || script.hasAttribute('async') ||
                    script.getAttributeNS('urn:foreign', 'async') !== 'unrelated') return 'clear absent async';
                if (!script.cloneNode().async) return 'clone has native fresh force-async without content';
                script.setAttribute('async', 'old value');
                if (!script.async) return 'live async attribute presence';
                const attr = script.getAttributeNode('async');
                script.async = {valueOf() { throw Error('Boolean conversion called valueOf'); }};
                if (!script.async || script.getAttributeNode('async') !== attr ||
                    script.getAttribute('async') !== '') return 'rewrite existing async in place';
                script.removeAttribute('async');
                if (script.async) return 'ordinary removal does not restore force-async';

                const added = document.createElement('script');
                added.setAttribute('async', 'added');
                added.removeAttribute('async');
                if (added.async) return 'ordinary Added hook clears force-async';

                const raw = 'a\ud800\0\udc00';
                for (const [member, attr] of [
                    ['type', 'type'], ['integrity', 'integrity'], ['charset', 'charset'],
                    ['event', 'event'], ['htmlFor', 'for'],
                ]) {
                    script.setAttributeNS('urn:foreign', 'q:' + attr, 'foreign');
                    if (script[member] !== '') return 'namespace default ' + member;
                    script[member] = raw;
                    if (script[member] !== raw || script.getAttributeNS(null, attr) !== raw ||
                        script.getAttributeNS('urn:foreign', attr) !== 'foreign') return 'raw reflection ' + member;
                }
                for (const [member, attr] of [['noModule', 'nomodule'], ['defer', 'defer']]) {
                    if (script[member]) return 'missing Boolean ' + member;
                    script[member] = 'false';
                    if (!script[member] || script.getAttributeNS(null, attr) !== '') return 'true Boolean ' + member;
                    script[member] = 0;
                    if (script[member] || script.hasAttributeNS(null, attr)) return 'false Boolean ' + member;
                }

                const effects = [];
                class ScriptObserver extends HTMLScriptElement {
                    static observedAttributes = ['async'];
                    attributeChangedCallback(name, oldValue, newValue) {
                        effects.push([name, oldValue, newValue, this.async]);
                    }
                }
                customElements.define('x-async-script', ScriptObserver, {extends: 'script'});
                const watched = document.createElement('script', {is: 'x-async-script'});
                watched.async = false;
                watched.async = true;
                watched.async = false;
                const expected = [['async', null, '', true], ['async', '', null, false]];
                if (JSON.stringify(effects) !== JSON.stringify(expected)) return 'Source reactions: ' + JSON.stringify(effects);
                return 'ok';
              } catch (error) {
                return 'unexpected guest exception: ' + (error.stack ?? String(error));
              }
            })()"#;
            let value = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap_or_else(|error| {
            if let DocumentScriptError::Script(Error::HostFailure(failure)) = &error {
                panic!("Script native failure: {:?}", std::error::Error::source(failure));
            }
            panic!("Script guest execution: {error:?}");
        });
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

// Oracles: pinned HTML #cors-settings-attributes, #referrer-policy-attributes,
// #fetch-priority-attributes, and the limited-to-known reflection algorithms.
pub(super) fn script_keyword_reflections_follow_exact_live_attributes() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 400_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    let result = super::window_length::observe(
        &mut host,
        &permit,
        r#"(() => {
            const prototype = HTMLScriptElement.prototype;
            const cases = [
                ['crossOrigin', 'crossorigin', null, 'anonymous', 'UsE-CrEdEnTiAlS', 'use-credentials'],
                ['referrerPolicy', 'referrerpolicy', '', '', 'StRiCt-OrIgIn-WhEn-CrOsS-OrIgIn',
                    'strict-origin-when-cross-origin'],
                ['fetchPriority', 'fetchpriority', 'auto', 'auto', 'HiGh', 'high'],
            ];
            for (const [member] of cases) {
                if (typeof Object.getOwnPropertyDescriptor(prototype, member)?.get !== 'function')
                    return 'missing HTMLScriptElement.' + member + ' getter';
            }
            const order = ['type', 'noModule', 'async', 'defer', 'blocking', 'crossOrigin',
                'referrerPolicy', 'integrity', 'fetchPriority', 'charset', 'event', 'htmlFor',
                'constructor'];
            if (Object.getOwnPropertyNames(prototype).filter(key => order.includes(key)).join() !== order.join())
                return 'Script keyword declaration order';

            const html = 'http://www.w3.org/1999/xhtml';
            const foreign = 'urn:script-keyword-foreign';
            const script = document.createElementNS(html, 'h:script');
            const wrongNamespace = document.createElementNS(foreign, 'script');
            Object.setPrototypeOf(wrongNamespace, prototype);
            const wrongCase = document.createElementNS(html, 'SCRIPT');
            const rawInvalid = ' \ud800\0\udc00 ';
            const typeError = action => {
                try { action(); } catch (error) {
                    return Object.getPrototypeOf(error) === TypeError.prototype;
                }
                return false;
            };

            for (const [member, attr, missing, invalid, mixed, canonical] of cases) {
                const descriptor = Object.getOwnPropertyDescriptor(prototype, member);
                const {get, set} = descriptor;
                if (typeof set !== 'function' || !descriptor.enumerable || !descriptor.configurable ||
                    'value' in descriptor || 'writable' in descriptor ||
                    get.name !== 'get ' + member || set.name !== 'set ' + member ||
                    get.length !== 0 || set.length !== 1)
                    return 'invalid descriptor ' + member;
                for (const accessor of [get, set]) {
                    if (Object.hasOwn(accessor, 'prototype') ||
                        !typeError(() => Reflect.construct(function () {}, [], accessor)))
                        return 'constructible accessor ' + member;
                }
                for (const wrong of [document, wrongNamespace, wrongCase, Object.create(prototype),
                    new Proxy(script, {})]) {
                    let coerced = false;
                    const value = {[Symbol.toPrimitive]() { coerced = true; return mixed; }};
                    if (!typeError(() => get.call(wrong)) ||
                        !typeError(() => set.call(wrong, value)) || coerced)
                        return 'receiver validation before conversion ' + member;
                }

                script.setAttributeNS(foreign, 'f:' + attr, 'foreign');
                script.setAttributeNS(null, attr.toUpperCase(), 'uppercase');
                if (get.call(script) !== missing || script.hasAttributeNS(null, attr))
                    return 'missing exact content attribute ' + member;

                script.setAttributeNS(null, attr, mixed);
                const original = script.getAttributeNodeNS(null, attr);
                if (original === null || script[member] !== canonical || original.value !== mixed)
                    return 'native attribute demonstrates canonical keyword ' + member;
                script[member] = rawInvalid;
                if (script[member] !== invalid || script.getAttributeNodeNS(null, attr) !== original ||
                    original.value !== rawInvalid)
                    return 'raw setter value and invalid default ' + member;
                script[member] = '';
                if (script[member] !== invalid || original.value !== '' ||
                    !script.hasAttributeNS(null, attr))
                    return 'present empty value ' + member;

                const padded = ' ' + mixed + ' ';
                script[member] = padded;
                if (script[member] !== invalid || original.value !== padded)
                    return 'keywords are matched as whole values ' + member;
                if (!typeError(() => set.call(script, Symbol('invalid'))) || original.value !== padded)
                    return 'abrupt DOMString conversion ' + member;
                const sentinel = {};
                try {
                    set.call(script, { toString() { throw sentinel; } });
                    return 'swallowed conversion exception ' + member;
                } catch (error) {
                    if (error !== sentinel || original.value !== padded)
                        return 'conversion did not preserve first exception or attribute ' + member;
                }

                let observed;
                let reentrant;
                let calls = 0;
                if (set.call(script, {[Symbol.toPrimitive](hint) {
                    ++calls;
                    observed = [hint, script.getAttributeNodeNS(null, attr) === original,
                        original.value, script[member]];
                    script.removeAttributeNS(null, attr);
                    script.setAttributeNS(null, attr, 'reentrant');
                    reentrant = script.getAttributeNodeNS(null, attr);
                    return mixed;
                }}) !== undefined ||
                    calls !== 1 || observed[0] !== 'string' || !observed[1] ||
                    observed[2] !== padded || observed[3] !== invalid ||
                    reentrant === original || original.ownerElement !== null ||
                    script.getAttributeNodeNS(null, attr) !== reentrant ||
                    reentrant.value !== mixed || script[member] !== canonical)
                    return 'conversion reentry did not resolve the current Attr ' + member;

                script[member] = null;
                if (script[member] !== (member === 'crossOrigin' ? null : invalid) ||
                    script.getAttributeNS(null, attr) !== (member === 'crossOrigin' ? null : 'null'))
                    return 'nullable versus ordinary DOMString null ' + member;
                script[member] = mixed;
                script[member] = undefined;
                if (script[member] !== (member === 'crossOrigin' ? null : invalid) ||
                    script.getAttributeNS(null, attr) !== (member === 'crossOrigin' ? null : 'undefined'))
                    return 'nullable versus ordinary DOMString undefined ' + member;
                script.removeAttributeNS(null, attr);
                if (script[member] !== missing ||
                    script.getAttributeNS(foreign, attr) !== 'foreign' ||
                    script.getAttributeNS(null, attr.toUpperCase()) !== 'uppercase')
                    return 'live absence or preservation of unrelated attributes ' + member;
            }

            const seen = [];
            class KeywordObserver extends HTMLScriptElement {
                static observedAttributes = ['crossorigin', 'referrerpolicy', 'fetchpriority'];
                attributeChangedCallback(name, previous, current) {
                    const member = cases.find(row => row[1] === name)[0];
                    seen.push([name, previous, current, this[member]]);
                }
            }
            customElements.define('x-keyword-script', KeywordObserver, {extends: 'script'});
            const watched = document.createElement('script', {is: 'x-keyword-script'});
            watched.crossOrigin = 'USE-CREDENTIALS';
            watched.crossOrigin = null;
            watched.referrerPolicy = 'NO-REFERRER';
            watched.fetchPriority = 'LOW';
            const expected = [
                ['crossorigin', null, 'USE-CREDENTIALS', 'use-credentials'],
                ['crossorigin', 'USE-CREDENTIALS', null, null],
                ['referrerpolicy', null, 'NO-REFERRER', 'no-referrer'],
                ['fetchpriority', null, 'LOW', 'low'],
            ];
            if (JSON.stringify(seen) !== JSON.stringify(expected))
                return 'Source reactions: ' + JSON.stringify(seen);

            script.setAttributeNS(null, 'data-script-keyword-probe', 'exact');
            script.crossOrigin = 'UsE-CrEdEnTiAlS';
            script.referrerPolicy = rawInvalid;
            script.fetchPriority = 'LoW';
            globalThis.scriptKeywordProbe = script;
            return 'ok';
        })()"#,
    );
    assert_eq!(result, "ok");

    host.with_native(|dom, _, _| {
        let snapshot = dom.snapshot();
        let namespace = Namespace::none();
        let marker = DocumentString::from("data-script-keyword-probe");
        let mut subjects = snapshot.nodes().filter_map(|id| {
            let node = snapshot.node(id).unwrap();
            (node.attribute_ns(&namespace, &marker)?.code_units()
                == [
                    u16::from(b'e'),
                    u16::from(b'x'),
                    u16::from(b'a'),
                    u16::from(b'c'),
                    u16::from(b't'),
                ])
            .then_some(node)
        });
        let script = subjects.next().unwrap();
        assert!(subjects.next().is_none());
        assert!(script.as_html_script().is_some());
        let raw = |name: &str| {
            script
                .attribute_ns(&namespace, &name.into())
                .unwrap()
                .code_units()
        };
        assert_eq!(
            raw("crossorigin"),
            "UsE-CrEdEnTiAlS".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(raw("referrerpolicy"), [0x20, 0xd800, 0, 0xdc00, 0x20]);
        assert_eq!(raw("fetchpriority"), [0x4c, 0x6f, 0x57]);
        let script = script.id();
        let mut batch = DomBatch::new();
        batch.remove_attribute_ns(script, namespace.clone(), "crossorigin");
        batch.set_attribute_ns(script, namespace.clone(), "referrerpolicy", "OrIgIn");
        batch.set_attribute_ns(script, namespace, "fetchpriority", "unrecognized");
        dom.apply(batch, &cancel()).unwrap();
        assert!(dom.pending_source_edit().is_none());
    })
    .unwrap();

    assert_eq!(
        super::window_length::observe(
            &mut host,
            &permit,
            r#"(scriptKeywordProbe.crossOrigin === null &&
                scriptKeywordProbe.referrerPolicy === 'origin' &&
                scriptKeywordProbe.fetchPriority === 'auto' &&
                scriptKeywordProbe.getAttributeNS(null, 'fetchpriority') === 'unrecognized' &&
                scriptKeywordProbe.getAttributeNS('urn:script-keyword-foreign', 'crossorigin') === 'foreign')
                ? 'ok' : 'native edits were not visible on the retained Script'"#,
        ),
        "ok",
    );
    host.stop().into_session().unwrap();
}
