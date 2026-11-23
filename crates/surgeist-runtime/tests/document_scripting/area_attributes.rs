//! Canonical live Area attribute reflection and Source effects.

// The pinned Area IDL supplies the own-member order. The referrer policy
// vocabulary and ordinary HTML reflection rules supply independent values.
// #2658 owns the speculative-load producer reached by hosted rel/policy writes.

use super::{window_length::observe, *};

pub(super) fn area_attributes_follow_canonical_source_and_receivers() {
    let (mut session, top) = setup();
    let document = top.document();
    let snapshot = session.snapshot();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(document, None);
    let frame = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    let frame = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&frame)
        .unwrap()
        .unwrap();

    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 400_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 1_000_000, 200_000, 64, 64, 64, 4),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const prototype = HTMLAreaElement.prototype;
            const names = ['alt', 'coords', 'shape', 'download', 'ping', 'rel', 'referrerPolicy', 'noHref'];
            for (const name of names) {
                if (typeof Object.getOwnPropertyDescriptor(prototype, name)?.get !== 'function')
                    return 'missing HTMLAreaElement.' + name + ' getter';
            }
            const order = ['alt', 'coords', 'shape', 'download', 'ping', 'rel', 'relList',
                'referrerPolicy', 'noHref', 'constructor'];
            if (Object.getOwnPropertyNames(prototype).filter(key => order.includes(key)).join() !== order.join())
                return 'Area declaration order';

            const inertDocument = document.implementation.createHTMLDocument();
            const html = 'http://www.w3.org/1999/xhtml';
            const foreign = 'urn:area-reflection-foreign';
            const area = inertDocument.createElementNS(html, 'h:area');
            const wrong = inertDocument.createElementNS(foreign, 'area');
            Object.setPrototypeOf(wrong, prototype);
            const wrongCase = inertDocument.createElementNS(html, 'AREA');
            const pending = new Proxy(area, {});
            const typeError = action => {
                try { action(); } catch (error) {
                    return Object.getPrototypeOf(error) === TypeError.prototype;
                }
                return false;
            };
            for (const name of names) {
                const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
                const {get, set} = descriptor;
                if (typeof set !== 'function' || !descriptor.enumerable || !descriptor.configurable ||
                    'value' in descriptor || 'writable' in descriptor ||
                    get.name !== 'get ' + name || set.name !== 'set ' + name ||
                    get.length !== 0 || set.length !== 1 ||
                    [get, set].some(f => Object.hasOwn(f, 'prototype') ||
                        !typeError(() => Reflect.construct(function () {}, [], f))))
                    return 'invalid descriptor ' + name;
                for (const receiver of [wrong, wrongCase, pending, document, Object.create(prototype)]) {
                    let coerced = false;
                    const value = {toString() {coerced = true; return 'bad';}};
                    if (!typeError(() => get.call(receiver)) ||
                        !typeError(() => set.call(receiver, value)) || coerced)
                        return 'receiver before conversion ' + name;
                }
            }
            if (Object.getPrototypeOf(area) !== prototype || area.ownerDocument !== inertDocument ||
                inertDocument.defaultView !== null || area.prefix !== 'h' || area.isConnected)
                return 'actual detached Area fixture';

            const raw = ' \ud800\0\udc00 ';
            for (const name of ['alt', 'coords', 'shape', 'download', 'rel']) {
                area.setAttributeNS(foreign, 'f:' + name, 'foreign');
                area.setAttributeNS(null, name.toUpperCase(), 'uppercase');
                if (area[name] !== '' || area.hasAttributeNS(null, name))
                    return 'exact missing value ' + name;
                for (const [value, expected] of [[null, 'null'], [undefined, 'undefined'], ['', ''], [raw, raw]]) {
                    area[name] = value;
                    if (area[name] !== expected || area.getAttributeNS(null, name) !== expected)
                        return 'raw DOMString reflection ' + name;
                }
                area.removeAttributeNS(null, name);
                if (area[name] !== '' || area.getAttributeNS(foreign, name) !== 'foreign' ||
                    area.getAttributeNS(null, name.toUpperCase()) !== 'uppercase')
                    return 'removal or unrelated attribute change ' + name;
            }

            const tokens = area.relList;
            area.rel = 'noopener NOOPENER';
            if (area.relList !== tokens || tokens.value !== 'noopener NOOPENER' ||
                !tokens.contains('noopener'))
                return 'rel setter and existing live list';
            tokens.value = 'nofollow';
            if (area.rel !== 'nofollow' || area.getAttributeNS(null, 'rel') !== 'nofollow')
                return 'existing live list and rel getter';

            if (area.ping !== '') return 'missing USVString';
            area.ping = '';
            if (area.getAttributeNS(null, 'ping') !== '') return 'present empty USVString';
            const usv = ' \ufffd\0\ufffd \ud83d\ude42';
            area.setAttributeNS(null, 'ping', raw + '\ud83d\ude42');
            if (area.ping !== usv || area.getAttributeNS(null, 'ping') !== raw + '\ud83d\ude42')
                return 'USVString getter preserved raw content';
            area.ping = raw + '\ud83d\ude42';
            if (area.ping !== usv || area.getAttributeNS(null, 'ping') !== usv)
                return 'USVString setter replaced isolated surrogates';

            if (area.referrerPolicy !== '') return 'missing referrer policy';
            area.referrerPolicy = 'NO-REFERRER';
            if (area.referrerPolicy !== 'no-referrer' ||
                area.getAttributeNS(null, 'referrerpolicy') !== 'NO-REFERRER')
                return 'canonical referrer policy and raw content';
            area.setAttributeNS(null, 'referrerpolicy', ' no-referrer ');
            if (area.referrerPolicy !== '' ||
                area.getAttributeNS(null, 'referrerpolicy') !== ' no-referrer ')
                return 'invalid whole referrer policy';
            area.referrerPolicy = '';
            if (area.referrerPolicy !== '' || area.getAttributeNS(null, 'referrerpolicy') !== '')
                return 'present empty referrer policy';

            area.setAttributeNS(foreign, 'f:nohref', 'foreign');
            if (area.noHref) return 'foreign nohref is not present in null namespace';
            area.noHref = {valueOf() { throw Error('Boolean valueOf called'); }};
            if (area.noHref !== true || area.getAttributeNS(null, 'nohref') !== '')
                return 'true Boolean presence';
            area.noHref = 0;
            if (area.noHref !== false || area.hasAttributeNS(null, 'nohref') ||
                area.getAttributeNS(foreign, 'nohref') !== 'foreign')
                return 'false Boolean removal';

            const alt = Object.getOwnPropertyDescriptor(prototype, 'alt');
            area.alt = 'earlier';
            const old = area.getAttributeNodeNS(null, 'alt');
            const sentinel = {};
            try {
                alt.set.call(area, {toString() { throw sentinel; }});
                return 'ignored abrupt conversion';
            } catch (error) {
                if (error !== sentinel || area.getAttributeNodeNS(null, 'alt') !== old)
                    return 'lost first exception or changed Attr';
            }
            let seen;
            let reentrant;
            alt.set.call(area, {[Symbol.toPrimitive](hint) {
                seen = [hint, alt.get.call(area)];
                area.removeAttributeNS(null, 'alt');
                area.setAttributeNS(null, 'alt', 'reentrant');
                reentrant = area.getAttributeNodeNS(null, 'alt');
                return raw;
            }});
            if (seen.join() !== 'string,earlier' || old.ownerElement !== null ||
                reentrant === old || area.getAttributeNodeNS(null, 'alt') !== reentrant ||
                reentrant.value !== raw || area.alt !== raw)
                return 'conversion reentry resolved current Attr';

            const effects = [];
            class ObservedArea extends HTMLAreaElement {
                static observedAttributes = ['alt', 'nohref'];
                attributeChangedCallback(name, previous, current, namespace) {
                    effects.push([name, previous, current, namespace, this.alt, this.noHref]);
                }
            }
            customElements.define('x-reflected-area', ObservedArea, {extends: 'area'});
            const hosted = document.createElement('area', {is: 'x-reflected-area'});
            if (!(hosted instanceof ObservedArea) || hosted.ownerDocument !== document || hosted.isConnected)
                return 'actual customized Area in the hosted owner';
            hosted.alt = 'allowed';
            hosted.noHref = true;
            hosted.noHref = false;
            const expected = [
                ['alt', null, 'allowed', null, 'allowed', false],
                ['nohref', null, '', null, 'allowed', true],
                ['nohref', '', null, null, 'allowed', false],
            ];
            if (JSON.stringify(effects) !== JSON.stringify(expected))
                return 'native Area attribute reactions: ' + JSON.stringify(effects);

            area.setAttributeNS(null, 'data-area-reflection-probe', 'exact');
            globalThis.areaReflection = {area, tokens, hosted};
            return 'ok';
        })()"#,
    );
    assert_eq!(result, "ok");

    host.with_native(|dom, _, _| {
        let snapshot = dom.snapshot();
        let namespace = Namespace::none();
        let marker = DocumentString::from("data-area-reflection-probe");
        let mut matching = snapshot.nodes().filter_map(|id| {
            let node = snapshot.node(id).unwrap();
            node.attribute_ns(&namespace, &marker).map(|_| node)
        });
        let node = matching.next().unwrap();
        assert!(matching.next().is_none());
        assert!(node.as_hyperlink().is_some());
        assert_eq!(
            node.attribute_ns(&namespace, &"alt".into())
                .unwrap()
                .code_units(),
            [0x20, 0xd800, 0, 0xdc00, 0x20]
        );
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(node.id(), namespace.clone(), "rel", "external");
        batch.set_attribute_ns(node.id(), namespace, "referrerpolicy", "ORIGIN");
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const {area, tokens} = areaReflection;
                return area.rel === 'external' && area.relList === tokens &&
                    tokens.contains('external') && area.referrerPolicy === 'origin'
                    ? 'ok' : 'native Area edits were not visible through retained accessors';
            })()"#,
        ),
        "ok",
    );

    for assignment in [
        "areaReflection.hosted.rel = 'nofollow'",
        "areaReflection.hosted.referrerPolicy = 'no-referrer'",
    ] {
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| {
            scope
                .evaluate(&assignment.encode_utf16().collect::<Vec<_>>())
                .map(drop)
        });
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("expected original hosted Area refusal for {assignment}, got {result:?}");
        };
        assert!(
            matches!(
                failure.cause::<DomSourceDriveError>(),
                Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                    DomError::HyperlinkProcessingUnavailable
                )))
            ),
            "wrong refusal for {assignment}: {:?}",
            std::error::Error::source(&failure)
        );
        assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
    }

    window_length::edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const {area, hosted} = areaReflection;
            if (hosted.hasAttributeNS(null, 'rel') ||
                hosted.hasAttributeNS(null, 'referrerpolicy') || hosted.alt !== 'allowed')
                return 'hosted refusal changed native attributes';
            const child = window[0];
            const childArea = child.document.createElement('area');
            const prototype = child.HTMLAreaElement.prototype;
            const foreign = Object.getOwnPropertyDescriptor(prototype, 'alt');
            const local = Object.getOwnPropertyDescriptor(HTMLAreaElement.prototype, 'alt');
            if (Object.getPrototypeOf(childArea) !== prototype || foreign.set === local.set ||
                Object.getPrototypeOf(foreign.set) !== child.Function.prototype)
                return 'actual Area creation realm';
            local.set.call(childArea, 'borrowed');
            if (foreign.get.call(childArea) !== 'borrowed' ||
                foreign.get.call(area) !== area.alt || childArea.ownerDocument !== child.document)
                return 'borrowed Area accesses exact current receiver';
            let coerced = false;
            try {
                foreign.set.call({}, {toString() { coerced = true; return 'invalid receiver'; }});
            } catch (error) {
                if (!coerced && Object.getPrototypeOf(error) === child.TypeError.prototype) return 'ok';
            }
            return 'borrowed setter threw in the wrong realm or used the forged receiver';
        })()"#,
    );
    assert_eq!(result, "ok");

    host.stop().into_session().unwrap();
}
