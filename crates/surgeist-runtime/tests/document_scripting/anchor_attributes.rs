//! HTML Anchor text and directly specified scalar attributes.

use super::*;

pub(super) fn anchor_text_and_scalar_attributes_use_live_native_source() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let result = host
        .in_document(&permit, |scope| {
            let source = r#"(() => {
              try {
                const proto = HTMLAnchorElement.prototype;
                const text = Object.getOwnPropertyDescriptor(proto, 'text');
                if (typeof text?.get !== 'function') return 'missing HTMLAnchorElement.text getter';
                const fields = [
                    'download', 'ping', 'rel', 'text', 'referrerPolicy',
                    'coords', 'charset', 'name', 'rev', 'shape',
                ];
                if (Object.keys(proto).filter(key => fields.includes(key)).join() !== fields.join()) {
                    return 'Anchor declaration order';
                }
                const detached = document.implementation.createHTMLDocument();
                const html = 'http://www.w3.org/1999/xhtml';
                const anchor = detached.createElementNS(html, 'h:a');
                const foreign = detached.createElementNS('urn:foreign', 'a');
                Object.setPrototypeOf(foreign, proto);
                for (const field of fields) {
                    const d = Object.getOwnPropertyDescriptor(proto, field);
                    if (!d.enumerable || !d.configurable ||
                        d.get.name !== 'get ' + field || d.get.length !== 0 ||
                        d.set.name !== 'set ' + field || d.set.length !== 1) return 'descriptor ' + field;
                    for (const wrong of [detached, foreign, Object.create(proto), new Proxy(anchor, {})]) {
                        let coerced = false;
                        const input = { toString() { coerced = true; return 'ignored'; } };
                        try { d.get.call(wrong); return 'bad getter receiver ' + field; }
                        catch (error) { if (!(error instanceof TypeError)) return 'getter brand error ' + field; }
                        try { d.set.call(wrong, input); return 'bad setter receiver ' + field; }
                        catch (error) { if (!(error instanceof TypeError) || coerced) return 'setter brand order ' + field; }
                    }
                }

                const literal = 'a\ud800\0\udc00z';
                const nested = detached.createElement('span');
                nested.appendChild(detached.createTextNode(literal));
                anchor.appendChild(detached.createTextNode('prefix'));
                anchor.appendChild(detached.createComment('ignored'));
                anchor.appendChild(nested);
                if (anchor.text !== 'prefix' + literal) return 'ordinary descendant text';
                nested.firstChild.data = literal + '!';
                if (anchor.text !== 'prefix' + literal + '!') return 'live nested text';
                const sentinel = {};
                try {
                    anchor.text = {toString() { throw sentinel; }};
                    return 'throwing text conversion returned';
                } catch (error) {
                    if (error !== sentinel || nested.parentNode !== anchor) return 'text abrupt completion';
                }
                let hint;
                const earlier = detached.createTextNode('earlier');
                anchor.text = { [Symbol.toPrimitive](arg) {
                    hint = arg;
                    anchor.appendChild(earlier);
                    return literal;
                }};
                if (hint !== 'string' || anchor.firstChild?.nextSibling !== null ||
                    anchor.firstChild.data !== literal || anchor.text !== literal ||
                    nested.parentNode !== null || earlier.parentNode !== null) return 'canonical replacement after conversion';
                anchor.text = null;
                if (anchor.text !== 'null') return 'non-nullable text conversion';
                anchor.text = '';
                if (anchor.firstChild !== null || anchor.text !== '') return 'empty replacement';

                for (const name of ['download', 'rel', 'coords', 'charset', 'name', 'rev', 'shape']) {
                    anchor.setAttributeNS('urn:foreign', 'q:' + name, 'foreign');
                    if (anchor[name] !== '') return 'null-namespace default ' + name;
                    anchor[name] = literal;
                    if (anchor.getAttributeNS(null, name) !== literal || anchor[name] !== literal ||
                        anchor.getAttributeNS('urn:foreign', name) !== 'foreign') return 'exact reflection ' + name;
                    anchor.removeAttributeNS(null, name);
                    if (anchor[name] !== '') return 'live removal ' + name;
                }
                const ping = 'a\ufffd\0\ufffdz\ud83d\ude42';
                anchor.setAttributeNS(null, 'ping', literal + '\ud83d\ude42');
                if (anchor.ping !== ping || anchor.getAttributeNS(null, 'ping') !== literal + '\ud83d\ude42') {
                    return 'USV ping getter';
                }
                anchor.ping = literal + '\ud83d\ude42';
                if (anchor.getAttributeNS(null, 'ping') !== ping || anchor.ping !== ping) return 'USV ping setter';
                anchor.referrerPolicy = 'NO-REFERRER';
                if (anchor.getAttributeNS(null, 'referrerpolicy') !== 'NO-REFERRER' ||
                    anchor.referrerPolicy !== 'no-referrer') return 'known policy reflection';
                anchor.setAttributeNS(null, 'referrerpolicy', ' no-referrer ');
                if (anchor.referrerPolicy !== '') return 'invalid policy reflection';

                const download = Object.getOwnPropertyDescriptor(proto, 'download');
                Object.setPrototypeOf(anchor, HTMLElement.prototype);
                download.set.call(anchor, {toString() {
                    anchor.setAttribute('download', 'earlier');
                    return literal;
                }});
                text.set.call(anchor, 'native receiver');
                if (download.get.call(anchor) !== literal || text.get.call(anchor) !== 'native receiver') {
                    return 'native Anchor survives prototype changes and scalar coercion reentry';
                }
                Object.setPrototypeOf(anchor, proto);
                const lifecycle = [];
                class AnchorChild extends HTMLElement {
                    connectedCallback() { lifecycle.push('connected'); }
                    disconnectedCallback() {
                        lifecycle.push(this.parentNode === null ? anchor.text : 'still attached');
                    }
                }
                customElements.define('x-anchor-child', AnchorChild);
                detached.body.appendChild(anchor);
                const custom = document.createElement('x-anchor-child');
                anchor.appendChild(custom);
                anchor.text = 'replacement';
                if (lifecycle.join() !== 'connected,replacement' || custom.parentNode !== null) {
                    return 'text Source drains native removal reactions after publication: ' + lifecycle;
                }
                globalThis.hostedAnchor = document.createElement('a');
                return 'ok';
              } catch (error) {
                return "unexpected guest exception: " + (error.stack ?? String(error));
              }
            })()"#;
            let value = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap_or_else(|error| {
            if let DocumentScriptError::Script(Error::HostFailure(failure)) = &error {
                panic!("Anchor native failure: {:?}", std::error::Error::source(failure));
            }
            panic!("Anchor guest execution: {error:?}");
        });
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");

    let before = host.snapshot().unwrap();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(
                &"hostedAnchor.rel = 'prefetch'"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            )
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("expected original hyperlink processing refusal, got {result:?}");
    };
    assert!(
        matches!(
            failure.cause::<DomSourceDriveError>(),
            Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                DomError::HyperlinkProcessingUnavailable
            )))
        ),
        "wrong refusal: {:?}",
        std::error::Error::source(&failure)
    );
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}
