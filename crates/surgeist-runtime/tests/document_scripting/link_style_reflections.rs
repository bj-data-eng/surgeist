//! Link and Style local reflections over the current native attribute and Source owners.

use super::{window_length::observe, *};
use surgeist_css::query::media::{CssMediaQuery, CssMediaType};
use surgeist_cssom::CssomStore;
use surgeist_dom::style::DomStyleCssomRoot;

const NATIVE_UNITS: [u16; 4] = [0x4e, 0xd800, 0, 0xdc00];

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(16_000_000, 1_000_000, 200_000, 64, 64, 64, 4)
}

pub(super) fn never_hosted_link_reflects_live_native_attributes() {
    let (session, top) = setup();
    let (mut host, _) = attach_event_window_with_source_limits(
        session,
        top,
        Arc::new(cancel()),
        300_000,
        source_limits(),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const prototype = HTMLLinkElement.prototype;
            if (typeof Object.getOwnPropertyDescriptor(prototype, 'crossOrigin')?.get !== 'function')
                return 'missing HTMLLinkElement.prototype.crossOrigin getter';
            const expected = ['crossOrigin', 'rel', 'as', 'relList', 'media', 'integrity', 'hreflang',
                'type', 'sizes', 'imageSrcset', 'imageSizes', 'referrerPolicy', 'blocking', 'disabled',
                'fetchPriority', 'charset', 'rev', 'target', 'constructor'];
            if (Object.getOwnPropertyNames(prototype).filter(key => expected.includes(key)).join() !== expected.join() ||
                Object.getPrototypeOf(prototype) !== HTMLElement.prototype)
                return 'pinned Link prototype order';
            const owner = document.implementation.createHTMLDocument();
            const link = owner.createElement('link');
            owner.head.appendChild(link);
            if (owner.defaultView !== null || !link.isConnected || Object.getPrototypeOf(link) !== prototype)
                return 'native Link owned by the never-hosted Document';
            const foreign = owner.createElementNS('urn:other', 'link');
            Object.setPrototypeOf(foreign, prototype);
            const forged = [foreign, new Proxy(link, {}), Object.create(prototype), document.body];
            const wrongType = action => {
                try { action(); } catch (error) { return error instanceof TypeError; }
                return false;
            };
            for (const member of expected.slice(0, -1)) {
                const d = Object.getOwnPropertyDescriptor(prototype, member);
                if (!d?.enumerable || !d.configurable || 'value' in d || 'writable' in d ||
                    typeof d.get !== 'function' || typeof d.set !== 'function' ||
                    d.get.name !== 'get ' + member || d.set.name !== 'set ' + member ||
                    d.get.length !== 0 || d.set.length !== 1 ||
                    Object.hasOwn(d.get, 'prototype') || Object.hasOwn(d.set, 'prototype') ||
                    !wrongType(() => Reflect.construct(function () {}, [], d.get)) ||
                    !wrongType(() => Reflect.construct(function () {}, [], d.set)))
                    return 'Link descriptor ' + member;
                for (const receiver of forged) {
                    let calls = 0;
                    const value = { toString() { calls++; return 'wrong'; } };
                    if (!wrongType(() => d.get.call(receiver)) || !wrongType(() => d.set.call(receiver, value)) || calls !== 0)
                        return 'Link brand before coercion for ' + member;
                }
            }
            const raw = ' \ud800\0\udc00 ';
            for (const [member, attribute] of [['rel','rel'], ['media','media'], ['integrity','integrity'],
                ['hreflang','hreflang'], ['type','type'], ['imageSizes','imagesizes'], ['charset','charset'],
                ['rev','rev'], ['target','target']]) {
                link.setAttributeNS('urn:other', 'p:' + attribute, 'foreign');
                link.setAttributeNS(null, attribute.toUpperCase(), 'upper');
                if (link[member] !== '' || link.getAttributeNS(null, attribute) !== null)
                    return 'exact null namespace and case for ' + member;
                link[member] = raw;
                if (link[member] !== raw || link.getAttributeNS(null, attribute) !== raw ||
                    link.getAttributeNS('urn:other', attribute) !== 'foreign')
                    return 'plain DOMString reflection for ' + member;
            }
            const image = 'x\ud800\0\udc00\ud83d\udc39 1x';
            const scalar = 'x\ufffd\0\ufffd\ud83d\udc39 1x';
            link.setAttributeNS(null, 'imagesrcset', image);
            if (link.imageSrcset !== scalar || link.getAttributeNS(null, 'imagesrcset') !== image)
                return 'USV read must normalize without editing the native attribute';
            link.imageSrcset = image;
            if (link.imageSrcset !== scalar || link.getAttributeNS(null, 'imagesrcset') !== scalar)
                return 'USV write must normalize before Source publication';
            const rows = [
                ['crossOrigin', 'crossorigin', null, 'UsE-CrEdEnTiAlS', 'use-credentials', 'anonymous'],
                ['as', 'as', '', 'AuDiOwOrKlEt', 'audioworklet', ''],
                ['referrerPolicy', 'referrerpolicy', '', 'StRiCt-OrIgIn', 'strict-origin', ''],
                ['fetchPriority', 'fetchpriority', 'auto', 'HiGh', 'high', 'auto']
            ];
            for (const [member, attribute, missing, known, canonical, invalid] of rows) {
                if (link[member] !== missing) return 'keyword missing state for ' + member;
                link[member] = known;
                if (link[member] !== canonical || link.getAttributeNS(null, attribute) !== known)
                    return 'keyword setter retains authored spelling for ' + member;
                link[member] = ' ' + known;
                if (link[member] !== invalid || link.getAttributeNS(null, attribute) !== ' ' + known)
                    return 'keyword invalid state for ' + member;
            }
            link.crossOrigin = '';
            if (link.crossOrigin !== 'anonymous') return 'empty CORS state';
            for (const nullish of [null, undefined]) {
                link.crossOrigin = 'use-credentials';
                link.crossOrigin = nullish;
                if (link.crossOrigin !== null || link.getAttributeNS(null, 'crossorigin') !== null)
                    return 'nullable CORS removes only the reflected slot';
            }
            const lists = [link.relList, link.sizes, link.blocking];
            link.rel = ' one\tone two ';
            lists[1].value = '16x16';
            lists[2].value = 'render';
            if (lists[0].value !== link.rel || lists[0].length !== 2)
                return 'rel reflects into the existing native token list';
            lists[0].add('three');
            if (link.rel !== 'one two three' || link.relList !== lists[0] ||
                link.sizes !== lists[1] || link.blocking !== lists[2] ||
                lists[1].value !== '16x16' || lists[2].value !== 'render')
                return 'token writes retain independent SameObject projections';
            link.setAttributeNS('urn:other', 'p:disabled', 'foreign');
            if (link.disabled) return 'foreign disabled must not select the Boolean state';
            link.disabled = { toString() { throw Error('Boolean reflection coerced object'); } };
            if (!link.disabled || link.getAttributeNS(null, 'disabled') !== '')
                return 'ordinary Boolean presence reflection';
            globalThis.linkReflection = link;
            return 'ok';
        })()"#,
    );
    assert_eq!(result, "ok");

    let snapshot = host.snapshot().unwrap();
    let link = snapshot
        .nodes()
        .find(|&id| snapshot.node(id).unwrap().as_link().is_some())
        .unwrap();
    let before = snapshot.node(link).unwrap();
    assert!(!before.as_link().unwrap().flags().explicitly_enabled());
    let owner = snapshot.document(before.owner_document().unwrap()).unwrap();
    assert_eq!(
        owner.metadata().document_environment().browsing_context(),
        surgeist_html::security::nonce::HtmlBrowsingContextPresence::Absent,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(
            link,
            Namespace::none(),
            "integrity",
            DocumentString::from_utf16(NATIVE_UNITS),
        );
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const link = linkReflection;
            const initial = 'N\ud800\0\udc00';
            if (link.integrity !== initial) return 'native edit missed the existing Link wrapper';
            const successor = link.ownerDocument.createElement('link');
            successor.integrity = 'successor';
            const sentinel = {};
            try {
                link.integrity = { toString() { throw sentinel; } };
                return 'Link setter swallowed coercion exception';
            } catch (error) {
                if (error !== sentinel || link.integrity !== initial) return 'Link setter changed the first exception';
            }
            let hint;
            link.integrity = { [Symbol.toPrimitive](actual) {
                hint = actual;
                if (link.integrity !== initial) throw Error('Link edit reached before conversion');
                link.parentNode.replaceChild(successor, link);
                return 'original\ud800';
            } };
            link.disabled = false;
            return hint === 'string' && link.parentNode === null && link.integrity === 'original\ud800' &&
                successor.integrity === 'successor' && !link.disabled &&
                link.getAttributeNS('urn:other', 'disabled') === 'foreign'
                ? 'ok' : 'Link Source lost the originally authenticated receiver';
        })()"#,
    );
    assert_eq!(result, "ok");
    let snapshot = host.snapshot().unwrap();
    let node = snapshot.node(link).unwrap();
    assert_eq!(node.parent(), None);
    assert!(node.as_link().unwrap().flags().explicitly_enabled());
    assert_eq!(
        node.attribute_ns(&Namespace::none(), &"integrity".into())
            .unwrap()
            .code_units(),
        "original"
            .encode_utf16()
            .chain([0xd800])
            .collect::<Vec<_>>(),
    );
    host.stop().into_session().unwrap();
}

pub(super) fn hosted_link_reflection_preserves_original_source_refusal() {
    let (session, top) = setup();
    let (mut host, _) = attach_event_window_with_source_limits(
        session,
        top,
        Arc::new(cancel()),
        160_000,
        source_limits(),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    let check = observe(
        &mut host,
        &permit,
        r#"(() => {
            const d = Object.getOwnPropertyDescriptor(HTMLLinkElement.prototype, 'rel');
            if (typeof d?.set !== 'function') return 'missing HTMLLinkElement.prototype.rel setter';
            globalThis.hostedLink = document.createElement('link');
            return hostedLink.parentNode === null && hostedLink.ownerDocument === document &&
                hostedLink.rel === '' && hostedLink.relList.length === 0 ? 'ok' : 'hosted Link fixture';
        })()"#,
    );
    assert_eq!(check, "ok");
    let snapshot = host.snapshot().unwrap();
    let stamp = snapshot.stamp();
    let link = snapshot
        .nodes()
        .find(|&id| snapshot.node(id).unwrap().as_link().is_some())
        .unwrap();
    let outcome = host.in_document(&permit, |scope| {
        scope
            .evaluate(
                &"hostedLink.rel = 'stylesheet'"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            )
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = outcome else {
        panic!("expected original native Link Source refusal: {outcome:?}");
    };
    let first = failure.cause::<DomSourceDriveError>().unwrap();
    assert!(matches!(
        first,
        DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::LinkProcessingUnavailable
        ))
    ));
    let retry = host.in_document::<()>(&permit, |_| {
        panic!("terminal refusal must precede guest reentry")
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(repeated))) = retry else {
        panic!("original Link refusal must survive retry: {retry:?}");
    };
    assert!(std::ptr::eq(
        first,
        repeated.cause::<DomSourceDriveError>().unwrap()
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), stamp);
    assert!(
        after
            .node(link)
            .unwrap()
            .attribute_ns(&Namespace::none(), &"rel".into())
            .is_none()
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

pub(super) fn active_style_media_updates_the_original_native_sheet() {
    let (mut session, top) = setup();
    let root = top.document().node();
    let mut styles = session.style_execution();
    styles
        .bind_root(
            &session.dom(),
            "guest-style-reflections".into(),
            &mut Some(DomStyleCssomRoot::new(
                root,
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            )),
            &cancel(),
        )
        .unwrap();
    let (mut host, _) = attach_event_window_with_source_limits(
        session,
        top,
        Arc::new(cancel()),
        300_000,
        source_limits(),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const prototype = HTMLStyleElement.prototype;
            if (typeof Object.getOwnPropertyDescriptor(prototype, 'media')?.get !== 'function')
                return 'missing HTMLStyleElement.prototype.media getter';
            const expected = ['media', 'blocking', 'type', 'constructor'];
            if (Object.getOwnPropertyNames(prototype).filter(key => expected.includes(key)).join() !== expected.join())
                return 'pinned Style prototype order';
            const detached = document.createElement('style');
            const raw = ' \ud800\0\udc00 ';
            for (const member of ['media', 'type']) {
                const d = Object.getOwnPropertyDescriptor(prototype, member);
                if (!d?.enumerable || !d.configurable || 'value' in d || 'writable' in d ||
                    typeof d.get !== 'function' || typeof d.set !== 'function' ||
                    d.get.name !== 'get ' + member || d.get.length !== 0 ||
                    d.set.name !== 'set ' + member || d.set.length !== 1)
                    return 'ordinary Style descriptor for ' + member;
                detached.setAttributeNS('urn:other', 'p:' + member, 'foreign');
                if (d.get.call(detached) !== '' || d.set.call(detached, raw) !== undefined ||
                    detached[member] !== raw || detached.getAttributeNS(null, member) !== raw ||
                    detached.getAttributeNS('urn:other', member) !== 'foreign')
                    return 'exact detached Style attribute for ' + member;
                let coerced = false;
                try {
                    d.set.call(new Proxy(detached, {}), {toString() { coerced = true; return raw; }});
                    return 'forged Style receiver accepted';
                } catch (error) {
                    if (!(error instanceof TypeError) || coerced) return 'Style must brand before coercion';
                }
            }
            globalThis.styleCalls = [];
            customElements.define('x-observed-style', class extends HTMLStyleElement {
                static observedAttributes = ['media', 'type'];
                attributeChangedCallback(name, oldValue, value, namespace) {
                    styleCalls.push([this, name, oldValue, value, namespace, this[name]]);
                }
            }, { extends: 'style' });
            const active = document.createElement('style', { is: 'x-observed-style' });
            active.textContent = 'p { color: red }';
            document.head.appendChild(active);
            globalThis.activeStyleReflection = active;
            globalThis.activeStyleBlocking = active.blocking;
            return active.isConnected && active.media === '' && active.type === '' ? 'ok' : 'connected Style fixture';
        })()"#,
    );
    assert_eq!(result, "ok");
    let before = styles.snapshot(root).unwrap().unwrap();
    assert_eq!(before.sheets().len(), 1);
    let id = before.sheets()[0].clone();
    let media = before.sheet(&id).unwrap().media().clone();
    assert!(before.media(&media).unwrap().queries().is_empty());
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const active = activeStyleReflection;
            const type = ' text/css\ud800\0\udc00 ';
            let coercions = 0;
            active.media = { [Symbol.toPrimitive](hint) {
                if (hint !== 'string' || active.media !== '') throw Error('Style write before conversion');
                coercions++;
                active.type = type;
                return 'print';
            } };
            const expected = [
                ['type', null, type, null, type],
                ['media', null, 'print', null, 'print']
            ];
            return coercions === 1 && active.blocking === activeStyleBlocking &&
                active.getAttributeNS(null, 'media') === 'print' && active.type === type &&
                styleCalls.every(call => call[0] === active) &&
                JSON.stringify(styleCalls.map(call => call.slice(1))) === JSON.stringify(expected)
                ? 'ok' : 'active Style Source effects and reaction order';
        })()"#,
    );
    assert_eq!(result, "ok");
    let after = styles.snapshot(root).unwrap().unwrap();
    assert_eq!(after.sheets(), std::slice::from_ref(&id));
    let active = after.sheet(&id).unwrap();
    let [CssMediaQuery::Typed(query)] = after.media(active.media()).unwrap().queries() else {
        panic!("native sheet media must contain the actual reflected query");
    };
    assert_eq!(query.media_type(), CssMediaType::Print);
    assert_eq!(after.sheet_rules(&id).unwrap().len(), 1);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}
