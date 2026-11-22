use super::{
    window_length::{content_navigable, edit, observe},
    *,
};

fn named_document_fixture() -> (DocumentScriptHost, DocumentScriptPermit) {
    let mut session = DomSession::new(native_dom(), limits()).unwrap();
    let (_, top) = create(&mut session);
    let (host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 180_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    (host, permit)
}

pub(super) fn document_names_are_virtual_own_properties() {
    let (mut host, permit) = named_document_fixture();
    let result = observe(
        &mut host,
        &permit,
        r"(() => {
        const doc = window.document;
        const body = doc.body;
        const add = name => {
            const form = doc.createElement('form');
            form.setAttribute('name', name);
            return body.appendChild(form);
        };
        const namedBody = add('body');
        const own = Object.getOwnPropertyDescriptor(doc, 'body');
        if (doc !== body.ownerDocument || doc !== document || doc.body !== namedBody || own?.value !== namedBody)
            return 'the canonical Document must expose live named own properties before its prototype';
        if (!own.enumerable || !own.configurable || own.writable || !Object.hasOwn(doc, 'body')
            || Reflect.deleteProperty(doc, 'body') || Reflect.defineProperty(doc, 'body', {value: 1}))
            return 'the standard named descriptor and mutations must be visible';
        const names = ['0', '01', '4294967295'];
        for (const name of names) {
            const form = add(name);
            if (doc[name] !== form || !Object.hasOwn(doc, name))
                return 'numeric-looking names must be distinguished as exact names';
        }
        const first = add('shared');
        const repeat = add('shared');
        const saved = doc.shared;
        if (!(saved instanceof HTMLCollection) || saved.length !== 2 || saved[0] !== first || saved[1] !== repeat)
            return 'multiple named elements must expose a live HTMLCollection';
        repeat.remove();
        if (doc.shared !== first || saved.length !== 1 || saved[0] !== first)
            return 'the retained multi-match collection must follow canonical mutations';
        const wrong = body.appendChild(doc.createElement('div'));
        wrong.setAttribute('id', 'unsupportedDocumentId');
        if (doc.unsupportedDocumentId !== undefined || doc.getElementById('unsupportedDocumentId') !== wrong)
            return 'Document name eligibility must use its own HTML predicate';
        if (Object.keys(doc).filter(key => ['body', ...names, 'shared'].includes(key)).join() !== 'body,0,01,4294967295,shared')
            return 'enumerable supported names must follow tree order with duplicates suppressed';
        namedBody.remove();
        if (doc.body !== body || Object.hasOwn(doc, 'body'))
            return 'retired name must reveal the prototype getter';
        const detached = doc.implementation.createHTMLDocument();
        const witness = detached.createElement('form');
        witness.setAttribute('name', 'detachedWitness');
        detached.body.appendChild(witness);
        if (detached.detachedWitness !== witness || witness.ownerDocument !== detached)
            return 'the factory Document must be its canonical named property holder';
        function Alternate() {}
        Alternate.prototype = {};
        const constructed = Reflect.construct(Document, [], Alternate);
        const child = Document.prototype.createElementNS.call(constructed, 'http://www.w3.org/1999/xhtml', 'form');
        child.setAttribute('name', 'constructedWitness');
        Node.prototype.appendChild.call(constructed, child);
        if (Object.getPrototypeOf(constructed) !== Alternate.prototype || constructed.constructedWitness !== child
            || child.ownerDocument !== constructed)
            return 'a selected custom prototype must preserve native Document named properties';
        return 'ok';
    })()",
    );
    assert_eq!(result, "ok");
}

pub(super) fn document_names_preserve_ordinary_own_keys_and_current_mutation() {
    let (mut host, permit) = named_document_fixture();
    let result = observe(
        &mut host,
        &permit,
        r"(() => {
        const doc = document;
        const body = doc.body;
        const add = name => {
            const form = doc.createElement('form');
            form.setAttribute('name', name);
            return body.appendChild(form);
        };
        const warm = value => { 'use strict'; doc.moving = value; };
        for (let index = 0; index < 32; ++index) warm(index);
        const form = add('moving');
        if (doc.moving !== 31) return 'a preexisting ordinary own descriptor must mask the named getter';
        try { warm(42); return 'warm assignment bypassed current native named definition rules'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        if (Reflect.defineProperty(doc, 'moving', {value: 42}) || doc.moving !== 31
            || !Reflect.deleteProperty(doc, 'moving') || doc.moving !== form)
            return 'definition and deletion must distinguish ordinary and supported names';
        const alternate = {};
        if (!Reflect.set(doc, 'moving', 42, alternate) || alternate.moving !== 42 || doc.moving !== form)
            return 'the exotic setter must preserve ordinary alternate receivers';
        if (Reflect.defineProperty(doc, 'during', {
            get value() { add('during'); return 7; }
        }) || doc.during?.getAttribute('name') !== 'during')
            return 'native definition must observe reentrant descriptor conversion';
        doc['8'] = 1;
        doc.late = 2;
        doc['4'] = 3;
        const marker = Symbol('ordinary symbol');
        doc[marker] = 4;
        const selected = Reflect.ownKeys(doc).filter(key => ['moving', 'during', '8', 'late', '4', marker].includes(key));
        if (selected.length !== 6 || selected.slice(0, 5).join() !== 'moving,during,8,late,4' || selected[5] !== marker)
            return 'named, ordinary string and symbol key groups must preserve their required order';
        doc['2'] = 8;
        const namedNumber = add('2');
        if (doc['2'] !== 8 || Reflect.defineProperty(doc, '2', {value: 9})
            || !Reflect.deleteProperty(doc, '2') || doc['2'] !== namedNumber)
            return 'ordinary numeric keys and exact supported names must use one property identity';
        const prototype = Object.getPrototypeOf(doc);
        let probes = 0;
        Object.setPrototypeOf(doc, new Proxy(prototype, {
            getOwnPropertyDescriptor() { probes++; throw new Error('named visibility examined the prototype'); }
        }));
        const visible = doc.moving === form && Object.hasOwn(doc, 'moving') && 'moving' in doc;
        return visible && probes === 0 ? 'ok' : 'supported named reads must bypass prototype visibility checks';
    })()",
    );
    assert_eq!(result, "ok");
}

pub(super) fn document_named_iframe_shares_the_native_window_proxy() {
    let mut session = DomSession::new(native_dom(), limits()).unwrap();
    let (_, top) = create(&mut session);
    let document = top.document();
    let snapshot = session.snapshot();
    let html = snapshot.node(document.node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(document, None);
    let iframe = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    batch.set_attribute(&iframe, "id", "nativeFrameElement");
    batch.set_attribute(&iframe, "name", "nativeFrameWitness");
    let receipt = session.dom().apply(batch, &cancel()).unwrap();
    let iframe = receipt.created(&iframe).unwrap().unwrap();
    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 180_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    edit(&mut host, DomSourceEdit::insert_before(iframe, body, None));
    let child = content_navigable(&host.snapshot().unwrap(), iframe).unwrap();
    let one = observe(
        &mut host,
        &permit,
        r"(() => {
        const child = window[0];
        const frame = document.getElementById('nativeFrameElement');
        if (!child || !frame || child.closed || child.parent !== window)
            return 'the fixture must expose a live same-origin child';
        if (document.nativeFrameWitness !== child || document.nativeFrameElement !== undefined)
            return 'a sole qualifying iframe must resolve to its canonical WindowProxy by name';
        const form = document.createElement('form');
        form.setAttribute('name', 'nativeFrameWitness');
        document.body.appendChild(form);
        const collection = document.nativeFrameWitness;
        if (!(collection instanceof HTMLCollection) || collection.length !== 2
            || collection[0] !== frame || collection[1] !== form)
            return 'a multi-match result must expose the iframe Element alongside the other Element';
        globalThis.savedNamedFrame = child;
        globalThis.savedNamedForm = form;
        globalThis.savedNamedFrameCollection = collection;
        return 'ok';
    })()",
    );
    assert_eq!(one, "ok");
    edit(&mut host, DomSourceEdit::detach(iframe));
    host.with_native(|dom, mut hosted, _| {
        for _ in 0..32 {
            match dom.snapshot().hosted_navigable(child) {
                Ok(record) if record.is_retired() => return,
                Err(DomHostedError::StaleIdentity) => return,
                Ok(_) => (),
                Err(error) => panic!("child retirement failed: {error:?}"),
            }
            hosted.run_next(dom, top.navigable(), &cancel()).unwrap();
        }
        panic!("child retirement did not complete");
    })
    .unwrap();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r"(() => document.nativeFrameWitness === savedNamedForm
            && savedNamedFrameCollection.length === 1 && savedNamedFrameCollection[0] === savedNamedForm
            && window[0] === undefined && savedNamedFrame.closed ? 'ok' : 'stale named result after native retirement')()"
        ),
        "ok"
    );
}
