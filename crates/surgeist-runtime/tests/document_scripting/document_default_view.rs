use super::{window_length::observe, *};

pub(super) fn default_view_follows_the_native_document_context() {
    let (mut session, top) = setup();
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(top.document(), None);
    let iframe = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    let iframe = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&iframe)
        .unwrap()
        .unwrap();
    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 150_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    let initial = observe(
        &mut host,
        &permit,
        r"(() => {
        const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, 'defaultView');
        if (typeof descriptor?.get !== 'function') return 'missing Document.defaultView';
        const {get, set, enumerable, configurable} = descriptor;
        if (get.name !== 'get defaultView' || get.length !== 0 || set !== undefined ||
            !enumerable || !configurable || Object.hasOwn(get, 'prototype'))
            return 'incorrect getter descriptor';
        for (const receiver of [{}, document.body, new Proxy(document, {}), Object.create(document)]) {
            try { get.call(receiver); return 'accepted non-Document receiver'; }
            catch (error) { if (!(error instanceof TypeError)) throw error; }
        }
        try { Reflect.construct(function () {}, [], get); return 'getter was constructible'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        for (const receiver of [new Document(), document.implementation.createHTMLDocument(),
            document.implementation.createDocument('urn:fixture', 'root')]) {
            if (get.call(receiver) !== null || receiver.defaultView !== null)
                return 'factory Documents have no browsing-context association';
        }
        if (get.call(document) !== window || document.defaultView !== globalThis)
            return 'top-level Document did not return its exact WindowProxy';
        globalThis.defaultViewGetter = get;
        return 'ok';
    })()",
    );
    assert_eq!(initial, "ok");
    window_length::edit(&mut host, DomSourceEdit::insert_before(iframe, body, None));
    let live = observe(
        &mut host,
        &permit,
        r"(() => {
        const proxy = window[0];
        const child = proxy.document;
        const childGetter = Object.getOwnPropertyDescriptor(proxy.Document.prototype, 'defaultView').get;
        if (child === document || proxy === window || defaultViewGetter === childGetter ||
            child.defaultView !== proxy || defaultViewGetter.call(child) !== proxy ||
            childGetter.call(document) !== window || childGetter.call(child) !== proxy)
            return 'borrowed getters must choose the receiving Document association';
        globalThis.retainedDefaultViewDocument = child;
        globalThis.retainedDefaultViewProxy = proxy;
        return 'ok';
    })()",
    );
    assert_eq!(live, "ok");
    let attached = host.snapshot().unwrap();
    let child_context = window_length::content_navigable(&attached, iframe).unwrap();
    let child_document = attached
        .hosted_navigable(child_context)
        .unwrap()
        .active_document()
        .unwrap();
    window_length::edit(&mut host, DomSourceEdit::detach(iframe));
    let removing = host.snapshot().unwrap();
    assert!(
        removing
            .hosted_navigable(child_context)
            .unwrap()
            .is_retiring()
    );
    assert_eq!(
        removing.document(child_document).unwrap().navigable(),
        Some(child_context)
    );
    let still_associated = observe(
        &mut host,
        &permit,
        r"(() => {
        return window.length === 0 && retainedDefaultViewProxy.closed === true &&
            defaultViewGetter.call(retainedDefaultViewDocument) === retainedDefaultViewProxy ? 'ok' :
            'retiring Document lost its original associated proxy before disassociation';
    })()",
    );
    assert_eq!(still_associated, "ok");
    host.with_native(|dom, mut hosted, _| {
        for _ in 0..128 {
            if dom
                .snapshot()
                .document(child_document)
                .unwrap()
                .navigable()
                .is_none()
            {
                return;
            }
            hosted.run_next(dom, top.navigable(), &cancel()).unwrap();
        }
        panic!("native child retirement did not remove the Document association");
    })
    .unwrap();
    let retired = observe(
        &mut host,
        &permit,
        r"(() => {
        return window.length === 0 && retainedDefaultViewProxy.closed === true &&
            defaultViewGetter.call(retainedDefaultViewDocument) === null ? 'ok' :
            'retired Document retained a browsing-context association';
    })()",
    );
    assert_eq!(retired, "ok");
    host.stop().into_session().unwrap();
}
