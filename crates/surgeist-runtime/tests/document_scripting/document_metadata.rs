use super::{window_length::observe, *};

pub(super) fn metadata_comes_from_the_canonical_document_receiver() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 40_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let result = observe(
        &mut host,
        &permit,
        r"(() => {
        const getters = {};
        const names = ['URL', 'documentURI', 'compatMode', 'characterSet', 'charset', 'inputEncoding', 'contentType'];
        for (const name of names) {
            const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, name);
            if (typeof descriptor?.get !== 'function') return 'missing Document metadata getter ' + name;
            const {get, set, enumerable, configurable} = descriptor;
            if (set !== undefined || !enumerable || !configurable || get.name !== 'get ' + name ||
                get.length !== 0 || Object.hasOwn(get, 'prototype') || Object.hasOwn(document, name))
                return 'incorrect Document metadata descriptor ' + name;
            for (const receiver of [null, 1, {}, Object.create(document), new Proxy(document, {}), document.body]) {
                try { Reflect.apply(get, receiver, []); return 'incorrect receiver accepted by ' + name; }
                catch (error) { if (!(error instanceof TypeError)) throw error; }
            }
            getters[name] = get;
        }
        const check = (doc, url, mode, contentType) => {
            const values = [url, url, mode, 'UTF-8', 'UTF-8', 'UTF-8', contentType];
            return names.every((name, index) =>
                doc[name] === values[index] && Reflect.apply(getters[name], doc, []) === values[index]);
        };
        if (!check(document, 'about:blank', 'BackCompat', 'text/html'))
            return 'initial HTML Document must supply its actual URL, quirks mode, encoding and MIME type';
        const html = document.implementation.createHTMLDocument('receiver');
        if (!check(html, 'about:blank', 'CSS1Compat', 'text/html'))
            return 'the detached HTML Document must read its own metadata';
        const xml = document.implementation.createDocument('http://www.w3.org/2000/svg', 'svg');
        if (!check(xml, 'about:blank', 'CSS1Compat', 'image/svg+xml'))
            return 'the SVG XML Document must read its own source-selected MIME type';
        if (!check(new Document(), 'about:blank', 'CSS1Compat', 'application/xml'))
            return 'the directly constructed Document must read its native XML facts';
        const named = document.body.appendChild(document.createElement('form'));
        named.setAttribute('name', 'URL');
        if (document.URL !== named || getters.URL.call(document) !== 'about:blank')
            return 'the named property and borrowed metadata getter must use the same Document';
        return 'ok';
    })()",
    );
    assert_eq!(result, "ok");
}

pub(super) fn metadata_getters_read_during_an_original_native_source_reaction() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 100_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let result = observe(
        &mut host,
        &permit,
        r"(() => {
        const observed = [];
        const name = 'x-document-metadata';
        class MetadataObserver extends HTMLElement {
            static observedAttributes = ['data-trigger'];
            attributeChangedCallback(attribute, previous, next) {
                const native = this.ownerDocument;
                observed.push([this, attribute, previous, next, native, native.URL, native.documentURI,
                    native.compatMode, native.charset, native.contentType]);
            }
        }
        customElements.define(name, MetadataObserver);
        const witness = document.createElement(name);
        if (!(witness instanceof MetadataObserver)) return 'missing canonical reaction fixture';
        witness.setAttribute('data-trigger', 'original');
        const expected = [witness, 'data-trigger', null, 'original', document,
            'about:blank', 'about:blank', 'BackCompat', 'UTF-8', 'text/html'];
        if (observed.length !== 1 || expected.some((item, index) => observed[0][index] !== item))
            return 'metadata getters must read the original native Source occurrence';
        return witness.getAttribute('data-trigger') === 'original' ? 'ok' : 'missing native edit';
    })()",
    );
    assert_eq!(result, "ok");
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

pub(super) fn borrowed_metadata_getters_read_the_other_live_document() {
    let (mut session, top) = setup();
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(top.document(), None);
    let iframe = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    let receipt = session.dom().apply(batch, &cancel()).unwrap();
    let iframe = receipt.created(&iframe).unwrap().unwrap();
    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 160_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    window_length::edit(&mut host, DomSourceEdit::insert_before(iframe, body, None));
    let result = observe(
        &mut host,
        &permit,
        r"(() => {
        const child = window[0];
        const foreign = child.document;
        const detached = document.implementation.createHTMLDocument();
        if (foreign === document || Object.getPrototypeOf(foreign) !== child.Document.prototype)
            return 'the fixture must expose a canonical foreign-realm Document';
        const localMode = Object.getOwnPropertyDescriptor(Document.prototype, 'compatMode')?.get;
        const otherMode = Object.getOwnPropertyDescriptor(child.Document.prototype, 'compatMode')?.get;
        if (typeof localMode !== 'function' || typeof otherMode !== 'function')
            return 'both realms must install Document metadata getters';
        if (localMode === otherMode || localMode.call(foreign) !== 'BackCompat' ||
            otherMode.call(detached) !== 'CSS1Compat')
            return 'borrowed getters must read the native receiver Document metadata';
        try { otherMode.call({}); return 'foreign metadata getter accepted a forged receiver'; }
        catch (error) {
            return error instanceof child.TypeError && !(error instanceof TypeError)
                ? 'ok' : 'the brand error must use the getter realm';
        }
    })()",
    );
    assert_eq!(result, "ok");
}
