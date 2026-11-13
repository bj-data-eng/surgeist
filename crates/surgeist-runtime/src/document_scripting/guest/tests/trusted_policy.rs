use super::*;

pub(in crate::document_scripting) fn guest_attribute_writes_reach_the_native_element() {
    use surgeist_dom::{DomBatch, Namespace};

    let mut host = public_node_host(5_000, 20_000);
    let permit = host.permit().unwrap();
    let element = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let element = batch.create_element_ns(permit.document, Namespace::html(), "div");
            dom.apply(batch, &AtomicBool::new(false))
                .unwrap()
                .created(&element)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    expose_public_node_for_source_test(&mut host, &permit, element, "attributeTarget");
    host.in_document(&permit, |scope| {
        assert_guest(scope, "typeof attributeTarget.setAttribute === 'function'")?;
        assert_guest(scope, r#"
            (() => {
                const target = attributeTarget;
                let conversions = 0;
                const input = { toString() { conversions++; return 'data-missing'; } };
                const throwsTypeError = operation => {
                    try { operation(); } catch (error) { return error instanceof TypeError; }
                    return false;
                };
                return throwsTypeError(() => target.setAttribute(input)) &&
                    throwsTypeError(() => target.setAttributeNS(input)) &&
                    throwsTypeError(() => target.setAttributeNS(input, input)) &&
                    conversions === 0;
            })()
        "#)?;
        assert_guest(scope, r#"
            (() => {
                const target = attributeTarget;
                const policy = trustedTypes.createPolicy('attributes', {createScript: value => value});
                const branded = policy.createScript('A\ud800\0');
                branded.toString = () => { throw new Error('coerced branded value'); };
                if (target.setAttribute('ONCLICK', branded) !== undefined ||
                    target.setAttribute('DATA-X', 'plain\udc00') !== undefined ||
                    target.setAttributeNS(null, 'onfocus', 'plain sink\ud800') !== undefined ||
                    target.setAttributeNS('urn:example', 'p:value', 'foreign\0') !== undefined) return false;
                try { target.setAttributeNS(null, 'p:invalid', 'x'); return false; }
                catch (error) { if (error.name !== 'NamespaceError') return false; }
                try { target.setAttribute('with space', 'x'); return false; }
                catch (error) { return error.name === 'InvalidCharacterError'; }
            })()
        "#)
    }).unwrap();
    let snapshot = host.snapshot().unwrap();
    let element = snapshot.node(element).unwrap();
    assert_eq!(
        element.attribute(&"onclick".into()).unwrap().code_units(),
        &[b'A' as u16, 0xd800, 0]
    );
    assert_eq!(
        element.attribute(&"onfocus".into()).unwrap().code_units(),
        &units("plain sink")
            .into_iter()
            .chain([0xd800])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        element.attribute(&"data-x".into()).unwrap().code_units(),
        &units("plain")
            .into_iter()
            .chain([0xdc00])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        element
            .attribute_ns(&Namespace::from_uri("urn:example"), &"value".into())
            .unwrap()
            .code_units(),
        &units("foreign\0")
    );
}

pub(in crate::document_scripting) fn window_factory_creates_the_exact_default_policy() {
    let mut host = public_node_host(5_000, 20_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = scope.evaluate(&units(r#"
            (() => {
                const factory = globalThis.trustedTypes;
                if (typeof factory !== 'object' || factory === null || factory !== window.trustedTypes ||
                    factory.defaultPolicy !== null) return 'factory exposure';
                const getter = Object.getOwnPropertyDescriptor(globalThis, 'trustedTypes').get;
                if (getter.call(window) !== factory || getter.call(globalThis) !== factory)
                    return 'borrowed Window getter identity';
                for (const receiver of [{}, Object.create(window), new Proxy(window, {})]) {
                    try { getter.call(receiver); return 'accepted a foreign Window receiver'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'wrong getter error'; }
                }
                const visited = [];
                const callback = input => input;
                const options = {
                    get createHTML() { visited.push('html'); return callback; },
                    get createParserOptions() { visited.push('parser'); },
                    get createScript() { visited.push('script'); return callback; },
                    get createScriptURL() { visited.push('url'); return callback; }
                };
                const unusual = factory.createPolicy('A\ud800\0', options);
                if (unusual.name !== 'A\ud800\0') return 'name:' + Array.from(unusual.name ?? '').map(c => c.charCodeAt(0));
                if (factory.defaultPolicy !== null) return 'unusual name became default';
                if (visited.join(',') !== 'html,parser,script,url') return 'dictionary order:' + visited.join(',');
                const original = {};
                try {
                    factory.createPolicy('default', {get createHTML() { throw original; }});
                    return 'getter returned';
                } catch (error) { if (error !== original) return 'getter throw identity'; }
                const policy = factory.createPolicy('default', options);
                if (policy.name !== 'default' || policy !== factory.defaultPolicy) return 'default identity';
                try { factory.createPolicy('default'); return 'duplicate accepted'; }
                catch (error) { return error instanceof TypeError && factory.defaultPolicy === policy ? 'ok' : 'duplicate rejection'; }
            })()
        "#))?;
        let status = String::from_utf16_lossy(&scope.to_utf16(&result)?);
        assert_eq!(status, "ok");
        assert_guest(scope, "typeof trustedTypes.createPolicy('method probe').createHTML === 'function'")?;
        assert_guest(scope, r#"
            (() => {
                const marker = {};
                let coercions = 0;
                const policy = trustedTypes.createPolicy('methods', {
                    createHTML(input, extra) {
                        'use strict';
                        if (this !== undefined || input !== 'A\ud800\0' ||
                            extra !== marker || arguments.length !== 2) throw marker;
                        return {toString() { return input; }};
                    },
                    createScript() { return null; },
                    createScriptURL() { return '\ud800\0'; }
                });
                const html = policy.createHTML({toString() { coercions++; return 'A\ud800\0'; }}, marker);
                if (coercions !== 1 || String(html) !== 'A\ud800\0' ||
                    String(policy.createScript('')) !== '' || String(policy.createScriptURL('')) !== '\ufffd\0') return false;
                globalThis.policyHtml = html;
                const throwing = trustedTypes.createPolicy('throwing', {createHTML() { throw marker; }});
                try { throwing.createHTML(''); return false; }
                catch (error) { if (error !== marker) return false; }
                try { throwing.createScript(''); return false; }
                catch (error) { return error instanceof TypeError; }
            })()
        "#)?;
        let html = scope.evaluate(&units("policyHtml"))?;
        let brand = scope.trusted_type(&html)?.unwrap();
        assert_eq!(brand.kind(), surgeist_bindings_js::TrustedTypeKind::Html);
        assert_eq!(brand.as_code_units(), [b'A' as u16, 0xd800, 0]);
        Ok(())
    })
    .unwrap();
}
