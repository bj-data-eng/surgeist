//! Live local Image reflections over the canonical inactive request.

use super::{window_length::observe, *};
use surgeist_dom::url::DomImageUpdate;
use surgeist_html::elements::image::{HtmlImageUpdateFlags, HtmlImgFreshRequestState};

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit, NodeId) {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 1_000_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r"(() => {
        const owner = document.implementation.createHTMLDocument();
        const image = owner.createElement('img');
        globalThis.imageLocalProbe = {owner, image};
        return image.ownerDocument === owner && owner.defaultView === null ? 'ok' : 'inactive Image setup';
    })()"
        ),
        "ok"
    );
    let snapshot = host.snapshot().unwrap();
    let ids = snapshot
        .nodes()
        .filter(|&id| snapshot.node(id).unwrap().as_img().is_some())
        .collect::<Vec<_>>();
    let [id]: [NodeId; 1] = ids.try_into().unwrap();
    let owner = snapshot.node(id).unwrap().owner_document().unwrap();
    assert_ne!(owner, top.document());
    assert!(
        !snapshot
            .document(owner)
            .unwrap()
            .metadata()
            .document_environment()
            .has_ever_had_navigable()
    );
    assert_eq!(wait(&host, id).document(), owner);
    (host, permit, id)
}

fn wait(host: &DocumentScriptHost, id: NodeId) -> DomImageUpdate {
    let snapshot = host.snapshot().unwrap();
    let image = snapshot.node(id).unwrap().as_img().unwrap();
    let state = image.state();
    assert_eq!(
        state.current_request().state(),
        HtmlImgFreshRequestState::Unavailable
    );
    assert!(state.current_request().current_url().is_empty());
    assert!(state.pending_request_is_null());
    assert!(state.lazy_load_resumption_is_null());
    image.suspended_update().unwrap().unwrap()
}

fn descriptors(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit) {
    assert_eq!(
        observe(
            host,
            permit,
            r"(() => {
        const {owner, image} = imageLocalProbe;
        const prototype = Object.getPrototypeOf(image);
        const fields = ['alt', 'srcset', 'sizes', 'crossOrigin', 'useMap', 'isMap', 'controls',
            'referrerPolicy', 'decoding', 'loading', 'fetchPriority', 'name', 'align', 'hspace', 'vspace', 'border'];
        const missing = fields.filter(name => {
            const d = Object.getOwnPropertyDescriptor(prototype, name);
            return typeof d?.get !== 'function' || typeof d.set !== 'function';
        });
        if (missing.length) return 'missing HTMLImageElement local accessors: ' + missing.join(',');
        if (prototype !== HTMLImageElement.prototype || Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
            Object.keys(prototype).filter(name => fields.includes(name)).join() !== fields.join())
            return 'Image local prototype or source order';
        const foreign = owner.createElementNS('urn:foreign', 'img');
        Object.setPrototypeOf(foreign, prototype);
        const upper = owner.createElementNS('http://www.w3.org/1999/xhtml', 'IMG');
        for (const name of fields) {
            const d = Object.getOwnPropertyDescriptor(prototype, name);
            if (!d.enumerable || !d.configurable || 'value' in d || 'writable' in d ||
                d.get.name !== 'get ' + name || d.get.length !== 0 || d.set.name !== 'set ' + name ||
                d.set.length !== 1 || Object.hasOwn(d.get, 'prototype') || Object.hasOwn(d.set, 'prototype'))
                return 'Image local descriptor ' + name;
            for (const receiver of [null, {}, document.body, foreign, upper, Object.create(prototype), new Proxy(image, {})]) {
                let converted = false;
                const input = {[Symbol.toPrimitive]() { converted = true; return 'wrong'; }};
                try { d.get.call(receiver); return 'wrong Image getter receiver ' + name; }
                catch (error) { if (Object.getPrototypeOf(error) !== TypeError.prototype) throw error; }
                try { d.set.call(receiver, input); return 'wrong Image setter receiver ' + name; }
                catch (error) {
                    if (Object.getPrototypeOf(error) !== TypeError.prototype || converted)
                        return 'Image receiver must be checked before conversion ' + name;
                }
            }
        }
        return 'ok';
    })()"
        ),
        "ok"
    );
}

pub(super) fn inactive_image_attributes_preserve_raw_and_reflected_values() {
    let (mut host, permit, id) = fixture();
    let first = wait(&host, id);
    descriptors(&mut host, &permit);
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r"(() => {
        const {image} = imageLocalProbe;
        const literal = 'a\ud800\0\udc00z';
        const scalar = 'a\ufffd\0\ufffdz';
        for (const [name, attribute] of [['alt','alt'], ['sizes','sizes'], ['useMap','usemap'], ['name','name'], ['align','align']]) {
            image.setAttributeNS('urn:other', 'p:' + attribute, 'foreign');
            if (image[name] !== '') return 'null namespace default ' + name;
            image[name] = literal;
            if (image[name] !== literal || image.getAttributeNS(null, attribute) !== literal ||
                image.getAttributeNS('urn:other', attribute) !== 'foreign') return 'DOMString ' + name;
            image[name] = null;
            if (image[name] !== 'null' || image.getAttribute(attribute) !== 'null') return 'DOMString null ' + name;
        }
        image.setAttribute('srcset', literal);
        if (image.srcset !== scalar || image.getAttribute('srcset') !== literal) return 'USV srcset getter';
        image.srcset = literal;
        if (image.srcset !== scalar || image.getAttribute('srcset') !== scalar) return 'USV srcset setter';
        image.border = literal;
        if (image.border !== literal) return 'legacy border retains UTF16';
        image.border = null;
        if (image.border !== '' || image.getAttribute('border') !== '') return 'legacy null-to-empty border';
        image.border = undefined;
        if (image.border !== 'undefined') return 'legacy border undefined is DOMString';
        image.border = null;
        for (const [name, attribute] of [['isMap','ismap'], ['controls','controls']]) {
            image[name] = {};
            if (!image[name] || image.getAttribute(attribute) !== '') return 'Boolean presence ' + name;
            image[name] = 0;
            if (image[name] || image.hasAttribute(attribute)) return 'Boolean removal ' + name;
        }
        const keywords = [
            ['crossOrigin','crossorigin',null,'USE-CREDENTIALS','use-credentials',' invalid ','anonymous'],
            ['referrerPolicy','referrerpolicy','','NO-REFERRER','no-referrer',' no-referrer ',''],
            ['decoding','decoding','auto','ASYNC','async',' async ','auto'],
            ['loading','loading','eager','LAZY','lazy',' lazy ','eager'],
            ['fetchPriority','fetchpriority','auto','HIGH','high',' high ','auto']
        ];
        for (const [name, attribute, missing, raw, canonical, invalid, fallback] of keywords) {
            image.removeAttribute(attribute);
            if (image[name] !== missing) return 'missing keyword ' + name;
            image[name] = raw;
            if (image[name] !== canonical || image.getAttribute(attribute) !== raw) return 'raw/canonical keyword ' + name;
            image.setAttribute(attribute, invalid);
            if (image[name] !== fallback || image.getAttribute(attribute) !== invalid) return 'invalid keyword ' + name;
        }
        image.crossOrigin = null;
        if (image.crossOrigin !== null || image.hasAttribute('crossorigin')) return 'nullable CORS removal';
        for (const name of ['hspace', 'vspace']) {
            for (const [raw, expected] of [['',0], ['  +19junk',19], ['-1',0], ['2147483647',2147483647], ['2147483648',0]]) {
                image.setAttribute(name, raw);
                if (image[name] !== expected) return 'unsigned getter ' + name + ':' + raw;
            }
            for (const [input, expected] of [[2 ** 32 + 7.75,7], [-1,0], [2147483648,0], [Infinity,0], [NaN,0]]) {
                image[name] = input;
                if (image[name] !== expected || image.getAttribute(name) !== String(expected)) return 'unsigned setter ' + name;
            }
        }
        let hint;
        image.hspace = {[Symbol.toPrimitive](value) { hint = value; return 7.75; }};
        if (hint !== 'number' || image.hspace !== 7) return 'unsigned conversion hint';
        const sentinel = {};
        try { image.alt = {toString() { throw sentinel; }}; return 'abrupt conversion returned'; }
        catch (error) { if (error !== sentinel || image.alt !== 'null') return 'original abrupt conversion'; }
        image.alt = {toString() { image.setAttribute('alt', 'earlier'); return literal; }};
        if (image.alt !== literal) return 'canonical setter after coercion reentry';
        return 'ok';
    })()"
        ),
        "ok"
    );
    let later = wait(&host, id);
    assert_ne!(later.id(), first.id());
    assert_eq!(later.flags(), HtmlImageUpdateFlags::new(false, false));
    let snapshot = host.snapshot().unwrap();
    for (name, expected) in [
        ("alt", vec![97, 0xd800, 0, 0xdc00, 122]),
        ("srcset", vec![97, 0xfffd, 0, 0xfffd, 122]),
        ("border", vec![]),
        ("hspace", vec![55]),
        ("vspace", vec![48]),
    ] {
        assert_eq!(
            snapshot
                .node(id)
                .unwrap()
                .attribute(&name.into())
                .unwrap()
                .code_units(),
            expected
        );
    }
    // Reverse native raw edits must be reinterpreted by the same live wrappers.
    host.with_native(|dom, _, _| {
        let mut edit = DomBatch::new();
        for (name, raw) in [
            ("decoding", "AsYnC"),
            ("loading", "LAZY"),
            ("crossorigin", ""),
            ("fetchpriority", "LoW"),
            ("referrerpolicy", " no-referrer "),
            ("hspace", "  +19junk"),
        ] {
            edit.set_attribute(id, name, raw);
        }
        dom.apply(edit, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r"(() => {
        const {image} = imageLocalProbe;
        return image.decoding === 'async' && image.loading === 'lazy' && image.crossOrigin === 'anonymous' &&
            image.fetchPriority === 'low' && image.referrerPolicy === '' && image.hspace === 19 ? 'ok' : 'native edit readback';
    })()"
        ),
        "ok"
    );
    wait(&host, id);
    host.stop().into_session().unwrap();
}

pub(super) fn inactive_image_keyword_effects_and_refused_reentry_keep_native_state() {
    let (mut host, permit, id) = fixture();
    descriptors(&mut host, &permit);
    let first = wait(&host, id);
    assert_eq!(
        observe(
            &mut host,
            &permit,
            "imageLocalProbe.image.crossOrigin = 'USE-CREDENTIALS'; 'ok'"
        ),
        "ok"
    );
    let changed = wait(&host, id);
    assert_ne!(changed.id(), first.id());
    assert_eq!(changed.flags(), HtmlImageUpdateFlags::new(false, false));
    assert_eq!(
        observe(
            &mut host,
            &permit,
            "imageLocalProbe.image.crossOrigin = 'uSe-CrEdEnTiAlS'; 'ok'"
        ),
        "ok"
    );
    assert_eq!(wait(&host, id).id(), changed.id());
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r"(() => {
        const {image} = imageLocalProbe;
        image.crossOrigin = null;
        return image.crossOrigin === null && !image.hasAttribute('crossorigin') ? 'ok' : 'CORS null removal';
    })()"
        ),
        "ok"
    );
    let removed = wait(&host, id);
    assert_ne!(removed.id(), changed.id());
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r"(() => {
        const {image} = imageLocalProbe;
        image.loading = 'LAZY';
        if (image.loading !== 'lazy') return 'lazy reflection';
        image.loading = 'EaGeR';
        return image.loading === 'eager' && image.getAttribute('loading') === 'EaGeR' ? 'ok' : 'eager reflection';
    })()"
        ),
        "ok"
    );
    // The actual null lazy resumption returns without starting a new update.
    assert_eq!(wait(&host, id).id(), removed.id());

    let before = host.snapshot().unwrap();
    let owner = before.node(id).unwrap().owner_document();
    let failure = native_failure(host.in_document(&permit, |scope| {
        scope
            .evaluate(
                &r"imageLocalProbe.image.alt = {toString() {
            document.adoptNode(imageLocalProbe.image);
            return 'must-not-publish';
        }}"
                .encode_utf16()
                .collect::<Vec<_>>(),
            )
            .map(drop)
    }));
    let first = failure.cause::<DocumentGuestEventError>().unwrap();
    assert!(
        matches!(
            first,
            DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Dom(DomError::ImageProcessingUnavailable)
            ))
        ),
        "unexpected reentry failure: {first:?}"
    );
    let repeated = native_failure(host.checkpoint());
    assert!(std::ptr::eq(
        first,
        repeated.cause::<DocumentGuestEventError>().unwrap()
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.node(id).unwrap().owner_document(), owner);
    assert!(after.node(id).unwrap().attribute(&"alt".into()).is_none());
    assert_eq!(wait(&host, id).id(), removed.id());
    host.stop().into_session().unwrap();
}
pub(super) fn inactive_image_setters_follow_the_original_child_realm() {
    let (mut session, top) = setup();
    let document = top.document();
    let snapshot = session.snapshot();
    let root = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(root).unwrap().children()[1];
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
        (8_000_000, 6_000_000, 200_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 200_000, 2_000_000, 64, 64, 64, 3),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    super::window_length::edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const child = window[0];
                const owner = child.document.implementation.createHTMLDocument();
                const image = owner.createElement('img');
                const prototype = child.HTMLImageElement.prototype;
                if (owner.defaultView !== null || image.ownerDocument !== owner ||
                    Object.getPrototypeOf(image) !== prototype || prototype === HTMLImageElement.prototype)
                    return 'the inactive Image must retain the original child prototype';
                const local = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'crossOrigin')?.set;
                const original = Object.getOwnPropertyDescriptor(prototype, 'crossOrigin')?.set;
                if (typeof local !== 'function' || typeof original !== 'function')
                    return 'missing Image crossOrigin setter in the top or original child realm';
                if (local === original || Object.getPrototypeOf(original) !== child.Function.prototype ||
                    Object.getPrototypeOf(local) !== Function.prototype)
                    return 'Image setters must retain their original functions and realms';
                let converted = false;
                try {
                    original.call(Object.create(prototype), {
                        toString() { converted = true; return 'anonymous'; }
                    });
                    return 'original Image setter accepted a fabricated receiver';
                } catch (error) {
                    if (converted || Object.getPrototypeOf(error) !== child.TypeError.prototype)
                        return 'original child TypeError must precede setter conversion';
                }
                globalThis.imageSetterRealmProbe = {owner, image, local, original};
                return 'ok';
            })()"#,
        ),
        "ok",
    );
    let snapshot = host.snapshot().unwrap();
    let mut images = snapshot
        .nodes()
        .filter(|&id| snapshot.node(id).unwrap().as_img().is_some());
    let id = images.next().unwrap();
    assert!(images.next().is_none());
    let owner = snapshot.node(id).unwrap().owner_document().unwrap();
    assert!(
        !snapshot
            .document(owner)
            .unwrap()
            .metadata()
            .document_environment()
            .has_ever_had_navigable()
    );
    let before = wait(&host, id);
    assert_eq!(before.document(), owner);

    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const {owner, image, local} = imageSetterRealmProbe;
                local.call(image, 'UsE-CrEdEnTiAlS');
                return image.ownerDocument === owner && image.crossOrigin === 'use-credentials' &&
                    image.getAttributeNS(null, 'crossorigin') === 'UsE-CrEdEnTiAlS'
                    ? 'ok' : 'borrowed top setter missed the original child Image';
            })()"#,
        ),
        "ok",
    );
    let written = wait(&host, id);
    assert_ne!(written.id(), before.id());
    assert_eq!(written.document(), owner);
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(id)
            .unwrap()
            .attribute(&"crossorigin".into()),
        Some(&"UsE-CrEdEnTiAlS".into()),
    );

    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
                const {owner, image, original} = imageSetterRealmProbe;
                original.call(image, null);
                return image.ownerDocument === owner && image.crossOrigin === null &&
                    !image.hasAttributeNS(null, 'crossorigin')
                    ? 'ok' : 'original child CORS setter did not remove its native attribute';
            })()"#,
        ),
        "ok",
    );
    let removed = wait(&host, id);
    assert_ne!(removed.id(), written.id());
    assert_eq!(removed.document(), owner);
    assert!(
        host.snapshot()
            .unwrap()
            .node(id)
            .unwrap()
            .attribute(&"crossorigin".into())
            .is_none()
    );
    host.stop().into_session().unwrap();
}
