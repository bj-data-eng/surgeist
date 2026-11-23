//! Local HTMLIFrameElement reflection on real hosted and inert nodes.

use super::{
    window_length::{content_navigable, edit, observe},
    *,
};
use surgeist_dom::hosted::DomHostedNavigableId;

const RAW: [u16; 5] = [0x20, 0xd800, 0, 0xdc00, 0x20];
const NATIVE: [u16; 3] = [0xdc00, 0, 0xd800];

fn fixture(source_work: u64) -> (DocumentScriptHost, NodeId, DomHostedNavigableId) {
    let (mut session, top) = setup();
    let document = top.document();
    let snapshot = session.snapshot();
    let html = snapshot.node(document.node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(document, None);
    let frame = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    batch.set_attribute(&frame, "id", "nativeReflectionFrame");
    batch.set_attribute(&frame, "loading", "LaZy");
    let receipt = session.dom().apply(batch, &cancel()).unwrap();
    let frame = receipt.created(&frame).unwrap().unwrap();
    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 900_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 1_000_000, source_work, 128, 4096, 4096, 8),
        |_, _, _| {},
    );
    edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    let child = content_navigable(&host.snapshot().unwrap(), frame).unwrap();
    (host, frame, child)
}

pub(super) fn local_reflections_follow_the_original_iframe_and_attributes() {
    let (mut host, iframe, child) = fixture(1_000_000);
    let permit = host.permit().unwrap();
    let before = host.snapshot().unwrap();
    let child_document = before.hosted_navigable(child).unwrap().active_document();
    assert!(child_document.is_some());
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const proto = HTMLIFrameElement.prototype;
            if (typeof Object.getOwnPropertyDescriptor(proto, 'name')?.get !== 'function')
                return 'missing HTMLIFrameElement.prototype.name getter';

            const frame = document.getElementById('nativeReflectionFrame');
            const windowProxy = window[0];
            const html = 'http://www.w3.org/1999/xhtml';
            const foreign = 'urn:iframe-reflection';
            const inert = document.implementation.createHTMLDocument('iframe reflection');
            const inertFrame = inert.createElementNS(html, 'p:iframe');
            inert.body.appendChild(inertFrame);
            const wrongNamespace = document.createElementNS(foreign, 'iframe');
            const wrongCase = document.createElementNS(html, 'IFRAME');
            Object.setPrototypeOf(wrongNamespace, proto);
            if (!windowProxy || window.length !== 1 || windowProxy.closed ||
                Object.getPrototypeOf(frame) !== proto || Object.getPrototypeOf(inertFrame) !== proto ||
                inertFrame.prefix !== 'p' || Object.getPrototypeOf(proto) !== HTMLElement.prototype)
                return 'real hosted and inert native Iframes';

            const order = ['name', 'sandbox', 'allow', 'allowFullscreen', 'width', 'height',
                'referrerPolicy', 'loading', 'align', 'scrolling', 'frameBorder',
                'marginHeight', 'marginWidth', 'constructor'];
            if (Object.getOwnPropertyNames(proto).filter(key => order.includes(key)).join() !== order.join())
                return 'Iframe declaration order';
            function typeError(action) {
                try { action(); } catch (error) {
                    return Object.getPrototypeOf(error) === TypeError.prototype;
                }
                return false;
            }
            for (const member of order.filter(key => key !== 'sandbox' && key !== 'constructor')) {
                const d = Object.getOwnPropertyDescriptor(proto, member);
                if (typeof d?.get !== 'function' || typeof d.set !== 'function' ||
                    !d.enumerable || !d.configurable || 'value' in d || 'writable' in d ||
                    d.get.name !== 'get ' + member || d.set.name !== 'set ' + member ||
                    d.get.length !== 0 || d.set.length !== 1)
                    return 'Iframe descriptor ' + member;
                for (const accessor of [d.get, d.set]) {
                    if (Object.hasOwn(accessor, 'prototype') ||
                        !typeError(() => Reflect.construct(function () {}, [], accessor)))
                        return 'constructible accessor ' + member;
                }
                for (const bad of [null, document.body, wrongCase, wrongNamespace,
                    Object.create(proto), new Proxy(frame, {})]) {
                    let coerced = false;
                    const value = {[Symbol.toPrimitive]() { coerced = true; return 'wrong'; }};
                    if (!typeError(() => d.get.call(bad)) ||
                        !typeError(() => d.set.call(bad, value)) || coerced)
                        return 'receiver must qualify before coercion ' + member;
                }
            }

            const raw = ' \ud800\0\udc00 ';
            for (const [member, attr] of [
                ['name', 'name'], ['allow', 'allow'], ['width', 'width'], ['height', 'height'],
                ['align', 'align'], ['scrolling', 'scrolling'], ['frameBorder', 'frameborder'],
                ['marginHeight', 'marginheight'], ['marginWidth', 'marginwidth']
            ]) {
                frame.setAttributeNS(foreign, 'p:' + attr, 'other namespace');
                frame.setAttributeNS(null, attr.toUpperCase(), 'other case');
                if (frame[member] !== '' || frame.getAttributeNS(null, attr) !== null)
                    return 'missing exact content attribute ' + member;
                frame[member] = null;
                const nullValue = member.startsWith('margin') ? '' : 'null';
                if (frame[member] !== nullValue || frame.getAttributeNS(null, attr) !== nullValue)
                    return 'null conversion ' + member;
                const attribute = frame.getAttributeNodeNS(null, attr);
                frame[member] = undefined;
                if (frame[member] !== 'undefined' || attribute.value !== 'undefined')
                    return 'undefined conversion and stable Attr ' + member;
                frame[member] = raw;
                if (frame[member] !== raw || frame.getAttributeNodeNS(null, attr) !== attribute ||
                    attribute.value !== raw || frame.getAttributeNS(foreign, attr) !== 'other namespace' ||
                    frame.getAttributeNS(null, attr.toUpperCase()) !== 'other case')
                    return 'exact string and original attribute ' + member;
            }

            frame.setAttributeNS(foreign, 'p:allowfullscreen', 'unrelated');
            if (frame.allowFullscreen) return 'absent allowfullscreen is false';
            frame.setAttribute('allowfullscreen', 'false');
            const present = frame.getAttributeNode('allowfullscreen');
            frame.allowFullscreen = {valueOf() { throw Error('ToBoolean called valueOf'); }};
            if (!frame.allowFullscreen || frame.getAttributeNode('allowfullscreen') !== present ||
                present.value !== '') return 'Boolean presence and original Attr';
            frame.allowFullscreen = 0;
            if (frame.allowFullscreen || frame.hasAttribute('allowfullscreen') ||
                frame.getAttributeNS(foreign, 'allowfullscreen') !== 'unrelated')
                return 'Boolean false removes only the original attribute';

            if (frame.loading !== 'lazy' || frame.getAttribute('loading') !== 'LaZy' ||
                inertFrame.loading !== 'eager' || inertFrame.hasAttribute('loading') ||
                frame.referrerPolicy !== '') return 'missing and native mixed-case keyword states';
            frame.referrerPolicy = 'No-ReFeRrEr';
            if (frame.referrerPolicy !== 'no-referrer' ||
                frame.getAttribute('referrerpolicy') !== 'No-ReFeRrEr')
                return 'canonical referrer keyword and raw setter';
            for (const [member, attr, fallback, invalid] of [
                ['loading', 'loading', 'eager', [' lazy ', '', raw]],
                ['referrerPolicy', 'referrerpolicy', '', [' origin ', '', raw]]
            ]) {
                for (const value of invalid) {
                    frame[member] = value;
                    if (frame[member] !== fallback || frame.getAttribute(attr) !== value)
                        return 'invalid keyword keeps raw content ' + member;
                }
                frame.removeAttribute(attr);
                if (frame[member] !== fallback || frame.hasAttribute(attr))
                    return 'missing state after native removal ' + member;
            }
            frame.loading = 'LaZy';
            frame.loading = 'EaGeR';
            if (frame.loading !== 'eager' || frame.getAttribute('loading') !== 'EaGeR')
                return 'real Lazy to Eager edit on the live child';

            const sandbox = inertFrame.sandbox;
            inertFrame.sandbox = 'allow-scripts allow-same-origin';
            if (inertFrame.sandbox !== sandbox || !(sandbox instanceof DOMTokenList) ||
                sandbox.value !== 'allow-scripts allow-same-origin' ||
                inertFrame.getAttribute('sandbox') !== sandbox.value)
                return 'existing SameObject and PutForwards sandbox';

            const hints = [];
            frame.name = {[Symbol.toPrimitive](hint) {
                hints.push(hint);
                if (frame.name !== raw) throw Error('outer edit preceded conversion');
                inertFrame.name = 'nested inert';
                frame.name = 'nested same receiver';
                return raw;
            }};
            const sentinel = {};
            try {
                frame.width = {toString() { throw sentinel; }};
                return 'abrupt conversion returned';
            } catch (error) {
                if (error !== sentinel) return 'abrupt conversion replaced the exception';
            }
            if (hints.join() !== 'string' || inertFrame.name !== 'nested inert' ||
                frame.name !== raw || frame.width !== raw || window[0] !== windowProxy ||
                window.length !== 1 || windowProxy.closed)
                return 'reentry must keep both original native receivers and the published child';

            globalThis.localReflectionFrame = frame;
            return 'ok';
        })()"#,
    );
    assert_eq!(result, "ok");
    let snapshot = host.snapshot().unwrap();
    assert_eq!(content_navigable(&snapshot, iframe), Some(child));
    assert_eq!(
        snapshot.hosted_navigable(child).unwrap().active_document(),
        child_document
    );
    let frame = snapshot.node(iframe).unwrap();
    for attr in [
        "name",
        "allow",
        "width",
        "height",
        "align",
        "scrolling",
        "frameborder",
        "marginheight",
        "marginwidth",
    ] {
        assert_eq!(
            frame
                .attribute_ns(&Namespace::none(), &attr.into())
                .unwrap()
                .code_units(),
            RAW
        );
    }
    assert_eq!(frame.attribute(&"loading".into()), Some(&"EaGeR".into()));

    edit(
        &mut host,
        DomSourceEdit::set_attribute_ns(
            iframe,
            Namespace::none(),
            "width",
            DocumentString::from_utf16(NATIVE),
        ),
    );
    edit(
        &mut host,
        DomSourceEdit::set_attribute_ns(
            iframe,
            Namespace::none(),
            "referrerpolicy",
            "StRiCt-OrIgIn",
        ),
    );
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const same = document.getElementById('nativeReflectionFrame') === localReflectionFrame;
                return same && localReflectionFrame.width === '\udc00\0\ud800' &&
                    localReflectionFrame.referrerPolicy === 'strict-origin' ? 'ok' : 'native Source readback';
            })()"#,
        ),
        "ok"
    );
    host.stop().into_session().unwrap();
}

pub(super) fn loading_setter_preserves_the_original_source_refusal() {
    let (mut host, iframe, child) = fixture(0);
    let permit = host.permit().unwrap();
    let before = host.snapshot().unwrap();
    let active = before.hosted_navigable(child).unwrap().active_document();
    let result = host.in_document(&permit, |scope| {
        let source = "Object.getOwnPropertyDescriptor(HTMLIFrameElement.prototype, 'loading').set.call(document.getElementById('nativeReflectionFrame'), 'eager')";
        scope.evaluate(&source.encode_utf16().collect::<Vec<_>>()).map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("Iframe loading must retain the original Source work refusal: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::SourceWorkLimit { limit: 0 }
        )))
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(content_navigable(&after, iframe), Some(child));
    assert_eq!(
        after.hosted_navigable(child).unwrap().active_document(),
        active
    );
    assert_eq!(
        after.node(iframe).unwrap().attribute(&"loading".into()),
        Some(&"LaZy".into())
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}
