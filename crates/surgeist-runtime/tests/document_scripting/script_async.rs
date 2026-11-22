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
