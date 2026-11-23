//! Local Source reflections and the original native Picture update recipients.

use super::{window_length::observe, *};
use surgeist_dom::url::DomImageUpdateId;
use surgeist_html::elements::image::HtmlImageUpdateFlags;

struct Picture {
    owner: DocumentId,
    source: NodeId,
    images: [NodeId; 4],
}

impl Picture {
    fn updates(&self, snapshot: &DomSnapshot) -> [DomImageUpdateId; 4] {
        self.images.map(|id| {
            let image = snapshot.node(id).unwrap().as_img().unwrap();
            let update = image.suspended_update().unwrap().unwrap();
            assert_eq!(update.document(), self.owner);
            assert_eq!(update.flags(), HtmlImageUpdateFlags::new(false, false));
            assert!(image.state().current_request().current_url().is_empty());
            update.id()
        })
    }
}

fn picture(retained_units: usize) -> (DocumentScriptHost, DocumentScriptPermit, Picture) {
    let (session, top) = setup();
    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 300_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, retained_units, 200_000, 64, 64, 64, 4),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"typeof Object.getOwnPropertyDescriptor(HTMLSourceElement.prototype, 'type')?.get === 'function'
            ? 'ok' : 'missing HTMLSourceElement.prototype.type getter'"#,
        ),
        "ok"
    );
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
            const owner = document.implementation.createHTMLDocument();
            const picture = owner.createElement('picture');
            const [before, source, after, wrapper, nested, tail] =
                ['img', 'source', 'img', 'div', 'img', 'img'].map(tag => owner.createElement(tag));
            wrapper.appendChild(nested);
            for (const node of [before, source, after, wrapper, tail]) picture.appendChild(node);
            picture.id = 'canonical-source-picture';
            owner.body.appendChild(picture);
            globalThis.sourceReflectionProbe = {owner, picture, source};
            return owner.defaultView === null && source.ownerDocument === owner && source.isConnected
                ? 'ok' : 'admitted never-hosted Picture fixture';
        })()"#,
        ),
        "ok"
    );
    let snapshot = host.snapshot().unwrap();
    let id = snapshot
        .nodes()
        .find(|&id| {
            snapshot
                .node(id)
                .unwrap()
                .attribute(&"id".into())
                .is_some_and(|value| value == &DocumentString::from("canonical-source-picture"))
        })
        .unwrap();
    let node = snapshot.node(id).unwrap();
    let owner = node.owner_document().unwrap();
    assert_ne!(owner, top.document());
    assert!(!snapshot.document(owner).unwrap().is_fully_active().unwrap());
    let [before, source, after, wrapper, tail]: [NodeId; 5] = node.children().try_into().unwrap();
    let nested = snapshot.node(wrapper).unwrap().children()[0];
    let picture = Picture {
        owner,
        source,
        images: [before, after, nested, tail],
    };
    picture.updates(&snapshot);
    (host, permit, picture)
}

pub(super) fn local_source_reflections_preserve_canonical_picture_recipients() {
    let (mut host, permit, picture) = picture(1_000_000);
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
            const {owner, source} = sourceReflectionProbe;
            const prototype = HTMLSourceElement.prototype;
            const order = ['type', 'srcset', 'sizes', 'media', 'width', 'height', 'constructor'];
            if (Object.getPrototypeOf(source) !== prototype ||
                Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
                Object.getPrototypeOf(HTMLSourceElement) !== HTMLElement ||
                Object.getOwnPropertyNames(prototype).filter(key => order.includes(key)).join() !== order.join())
                return 'pinned Source interface and declaration order';
            const html = 'http://www.w3.org/1999/xhtml';
            const foreign = owner.createElementNS('urn:foreign', 'source');
            const wrongCase = owner.createElementNS(html, 'SOURCE');
            const prefixed = owner.createElementNS(html, 'p:source');
            Object.setPrototypeOf(foreign, prototype);
            const typeError = invoke => {
                try { invoke(); } catch (error) { return Object.getPrototypeOf(error) === TypeError.prototype; }
                return false;
            };
            if (Object.getPrototypeOf(prefixed) !== prototype || prefixed.prefix !== 'p')
                return 'native Source family also recognizes its HTML prefix';
            for (const key of order.slice(0, -1)) {
                const d = Object.getOwnPropertyDescriptor(prototype, key);
                const missing = key === 'width' || key === 'height' ? 0 : '';
                if (!d?.enumerable || !d.configurable || 'value' in d || 'writable' in d ||
                    typeof d.get !== 'function' || typeof d.set !== 'function' ||
                    d.get.name !== 'get ' + key || d.get.length !== 0 ||
                    d.set.name !== 'set ' + key || d.set.length !== 1 ||
                    Object.hasOwn(d.get, 'prototype') || Object.hasOwn(d.set, 'prototype') ||
                    d.get.call(prefixed) !== missing || d.get.call(source) !== missing)
                    return 'own Source descriptor and missing default for ' + key;
                for (const accessor of [d.get, d.set]) {
                    if (!typeError(() => Reflect.construct(function () {}, [], accessor)))
                        return 'Source accessors are nonconstructible';
                    for (const receiver of [null, {}, Object.create(prototype), new Proxy(source, {}), foreign, wrongCase]) {
                        let coerced = false;
                        if (!typeError(() => Reflect.apply(accessor, receiver,
                            [{[Symbol.toPrimitive]() { coerced = true; return 1; }}])) || coerced)
                            return 'Source brand must precede conversion';
                    }
                }
            }
            for (const key of ['width', 'height']) {
                for (const [raw, expected] of [[' +17px', 17], ['-1', 0], ['NaN', 0],
                    ['2147483647', 2147483647], ['2147483648', 0]]) {
                    prefixed.setAttributeNS(null, key, raw);
                    if (prefixed[key] !== expected) return 'unsigned content recovery for ' + key;
                }
                for (const [input, expected] of [[0, 0], [-1, 0], [Infinity, 0],
                    [2147483647, 2147483647], [2147483648, 0], [4294967303, 7], [12.9, 12]]) {
                    prefixed[key] = input;
                    if (prefixed[key] !== expected || prefixed.getAttributeNS(null, key) !== String(expected))
                        return 'IDL wrapping and HTML unsigned setter bounds for ' + key;
                }
            }
            return 'ok';
        })()"#,
        ),
        "ok"
    );

    let before = picture.updates(&host.snapshot().unwrap());
    let native = DocumentString::from_utf16([0x4e, 0xd800, 0, 0xdc00]);
    host.with_native(|dom, _, _| {
        let mut edit = DomBatch::new();
        edit.set_attribute_ns(
            picture.source,
            Namespace::from_uri("urn:foreign"),
            "p:media",
            "foreign",
        );
        edit.set_attribute_ns(picture.source, Namespace::none(), "MEDIA", "upper");
        dom.apply(edit, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(picture.updates(&host.snapshot().unwrap()), before);
    window_length::edit(
        &mut host,
        DomSourceEdit::set_attribute_ns(picture.source, Namespace::none(), "srcset", native),
    );
    let current = picture.updates(&host.snapshot().unwrap());
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| current[i] != before[i]),
        [false, true, false, true]
    );
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
            const {source} = sourceReflectionProbe;
            return source.srcset === 'N\ufffd\0\ufffd' && source.getAttributeNS(null, 'srcset') === 'N\ud800\0\udc00' &&
                source.media === '' && source.getAttributeNS('urn:foreign', 'media') === 'foreign' &&
                source.getAttributeNS(null, 'MEDIA') === 'upper' ? 'ok' : 'live native edits and USV readback';
        })()"#,
        ),
        "ok"
    );

    let updates: [(&str, &str, DocumentString); 6] = [
        (
            "type",
            r#"' image/png\ud800\0 '"#,
            DocumentString::from_utf16(
                " image/png"
                    .encode_utf16()
                    .chain([0xd800, 0, 0x20])
                    .collect::<Vec<_>>(),
            ),
        ),
        (
            "srcset",
            r#"'img\ud800\0\udc00\ud83d\udc39 1x'"#,
            "img\u{fffd}\0\u{fffd}🐹 1x".into(),
        ),
        ("sizes", "'20px'", "20px".into()),
        ("media", "'print'", "print".into()),
        ("width", "4294967303", "7".into()),
        ("height", "-1", "0".into()),
    ];
    for (member, value, expected) in updates {
        let before = picture.updates(&host.snapshot().unwrap());
        let script = format!("sourceReflectionProbe.source.{member} = {value}; 'ok'");
        assert_eq!(observe(&mut host, &permit, &script), "ok");
        let snapshot = host.snapshot().unwrap();
        assert_eq!(
            snapshot
                .node(picture.source)
                .unwrap()
                .attribute_ns(&Namespace::none(), &member.into(),),
            Some(&expected),
            "{member}"
        );
        let after = picture.updates(&snapshot);
        assert_eq!(
            std::array::from_fn::<_, 4, _>(|i| after[i] != before[i]),
            [false, true, false, true],
            "{member}"
        );
    }
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
            const {owner, picture, source} = sourceReflectionProbe;
            const replacement = owner.createElement('source');
            replacement.media = 'replacement';
            const sentinel = {};
            try { source.media = {toString() { throw sentinel; }}; return 'lost author throw'; }
            catch (error) { if (error !== sentinel || source.media !== 'print') return 'wrong conversion failure'; }
            let calls = 0;
            source.media = {[Symbol.toPrimitive](hint) {
                if (hint !== 'string' || source.media !== 'print') throw Error('early Source publication');
                calls++;
                picture.replaceChild(replacement, source);
                return 'original\ud800\0';
            }};
            return calls === 1 && source.parentNode === null && source.media === 'original\ud800\0' &&
                replacement.parentNode === picture && replacement.media === 'replacement'
                ? 'ok' : 'coercion must edit the original Source receiver';
        })()"#,
        ),
        "ok"
    );
    let snapshot = host.snapshot().unwrap();
    let body = snapshot
        .node(
            snapshot
                .document(permit.document())
                .unwrap()
                .document_element()
                .unwrap(),
        )
        .unwrap()
        .children()[1];
    let frame = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let detached = batch.create_html_document(permit.document(), None);
            let frame = batch.create_element_ns(&detached, Namespace::html(), "iframe");
            dom.apply(batch, &cancel())
                .unwrap()
                .created(&frame)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    window_length::edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
            const parentSource = sourceReflectionProbe.source;
            const child = window[0];
            const childSource = child.document.createElement('source');
            const local = Object.getOwnPropertyDescriptor(HTMLSourceElement.prototype, 'type');
            const foreign = Object.getOwnPropertyDescriptor(child.HTMLSourceElement.prototype, 'type');
            if (Object.getPrototypeOf(childSource) !== child.HTMLSourceElement.prototype ||
                Object.getPrototypeOf(foreign.get) !== child.Function.prototype ||
                foreign.get === local.get || foreign.get.call(parentSource) !== parentSource.type)
                return 'actual Source accessor creation realm and borrowed native read';
            local.set.call(childSource, 'child');
            if (foreign.get.call(childSource) !== 'child' ||
                document.adoptNode(childSource) !== childSource ||
                Object.getPrototypeOf(childSource) !== child.HTMLSourceElement.prototype)
                return 'borrowed Source writer and adoption preserve original projection';
            try { foreign.get.call({}); return 'accepted forged receiver'; }
            catch (error) { return Object.getPrototypeOf(error) === child.TypeError.prototype ? 'ok' : 'wrong error realm'; }
        })()"#,
        ),
        "ok"
    );
    host.stop().into_session().unwrap();
}

pub(super) fn local_source_refusal_preserves_attribute_and_waiters() {
    const UNITS: usize = 8_192;
    let (mut host, permit, picture) = picture(UNITS);
    let before = host.snapshot().unwrap();
    let original = picture.updates(&before);
    let outcome = host.in_document(&permit, |scope| {
        scope
            .evaluate(
                &"sourceReflectionProbe.source.media = 'x'.repeat(16384)"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            )
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = outcome else {
        panic!("expected the original Source retained-unit refusal: {outcome:?}");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::ResourceLimit {
                resource: DomResource::SourceRetainedUnits,
                limit: UNITS
            }
        )))
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(picture.updates(&after), original);
    assert!(
        after
            .node(picture.source)
            .unwrap()
            .attribute_ns(&Namespace::none(), &"media".into())
            .is_none()
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

pub(super) fn borrowed_child_source_setter_keeps_its_realm_and_native_recipients() {
    let (mut host, permit, picture) = picture(1_000_000);
    // picture() explicitly funds 8M host / 6M guest and creates every Img in
    // the never-hosted owner; no active child Img is constructed.
    let snapshot = host.snapshot().unwrap();
    let html = snapshot
        .document(permit.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(html).unwrap().children()[1];
    let frame = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let detached = batch.create_html_document(permit.document(), None);
            let frame = batch.create_element_ns(&detached, Namespace::html(), "iframe");
            dom.apply(batch, &cancel())
                .unwrap()
                .created(&frame)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    window_length::edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    let before = picture.updates(&host.snapshot().unwrap());
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r"(() => {
        const {owner, picture, source} = sourceReflectionProbe;
        const child = window[0];
        const childType = Object.getOwnPropertyDescriptor(child.HTMLSourceElement.prototype, 'type');
        const topType = Object.getOwnPropertyDescriptor(HTMLSourceElement.prototype, 'type');
        const childWidth = Object.getOwnPropertyDescriptor(child.HTMLSourceElement.prototype, 'width');
        if (typeof childType?.set !== 'function' || typeof childWidth?.set !== 'function')
            return 'missing child Source setters';
        if (childType.set === topType.set ||
            Object.getPrototypeOf(childType.set) !== child.Function.prototype ||
            Object.getPrototypeOf(childWidth.set) !== child.Function.prototype ||
            child.TypeError.prototype === TypeError.prototype)
            return 'Source setters must retain their actual child Function realm';

        let converted = false, rejectedReceiver = false;
        try { childType.set.call({}, {toString() { converted = true; return 'wrong'; }}); }
        catch (error) {
            rejectedReceiver = Object.getPrototypeOf(error) === child.TypeError.prototype;
        }
        if (!rejectedReceiver || converted)
            return 'borrowed child setter receiver error must precede conversion in its own realm';
        let rejected = false;
        try { childWidth.set.call(source, 1n); }
        catch (error) {
            rejected = Object.getPrototypeOf(error) === child.TypeError.prototype;
        }
        if (!rejected || source.hasAttribute('width'))
            return 'borrowed child numeric setter must retain its conversion TypeError realm';

        // The method's active child Document must not replace this receiver's
        // inactive owner when the actual Picture recipients are processed.
        childType.set.call(source, 'child\ud800\0');
        if (source.type !== 'child\ud800\0' || source.ownerDocument !== owner || source.parentNode !== picture)
            return 'borrowed child setter lost the original Picture Source receiver';

        const childSource = child.document.createElement('source');
        const prototype = child.HTMLSourceElement.prototype;
        topType.set.call(childSource, 'top writes child');
        if (childType.get.call(childSource) !== 'top writes child' || Object.getPrototypeOf(childSource) !== prototype)
            return 'borrowed top setter lost child creation identity';
        if (owner.adoptNode(childSource) !== childSource || childSource.ownerDocument !== owner ||
            Object.getPrototypeOf(childSource) !== prototype)
            return 'Source adoption must preserve its original child projection';
        childType.set.call(childSource, 'after adoption');
        if (childSource.type !== 'after adoption' || childType.get.call(childSource) !== 'after adoption')
            return 'original child setter must write the adopted native Source';
        return 'ok';
    })()"
        ),
        "ok"
    );
    let after = host.snapshot().unwrap();
    assert_eq!(
        after
            .node(picture.source)
            .unwrap()
            .attribute(&"type".into())
            .unwrap()
            .code_units(),
        vec![99, 104, 105, 108, 100, 0xd800, 0]
    );
    let updates = picture.updates(&after);
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|i| updates[i] != before[i]),
        [false, true, false, true]
    );
    for id in picture.images {
        let image = after.node(id).unwrap().as_img().unwrap();
        assert_eq!(
            image.state().current_request().state(),
            surgeist_html::elements::image::HtmlImgFreshRequestState::Unavailable
        );
        assert!(image.state().pending_request_is_null());
    }
    host.stop().into_session().unwrap();
}
