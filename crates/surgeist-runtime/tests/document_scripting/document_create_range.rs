use super::{window_length::observe, *};

pub(super) fn document_range_keeps_canonical_live_boundaries() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 150_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let initial = observe(
        &mut host,
        &permit,
        r#"(() => {
            const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, 'createRange');
            if (typeof descriptor?.value !== 'function') return 'missing Document.createRange';
            const {value: make, enumerable, writable, configurable} = descriptor;
            if (make.name !== 'createRange' || make.length !== 0 ||
                !enumerable || !writable || !configurable || Object.hasOwn(make, 'prototype'))
                return 'incorrect factory descriptor';
            const rejects = (invoke, errorName) => {
                try { invoke(); } catch (error) { return error.name === errorName; }
                return false;
            };
            for (const wrong of [{}, Object.create(document), document.body, new Proxy(document, {})]) {
                if (!rejects(() => make.call(wrong), 'TypeError')) return 'Document brand required';
            }
            if (!rejects(() => new AbstractRange(), 'TypeError')) return 'AbstractRange constructed';
            const range = document.createRange();
            const another = document.createRange();
            const detached = document.implementation.createHTMLDocument();
            const other = make.call(detached);
            const direct = new Range();
            if (Object.getPrototypeOf(range) !== Range.prototype ||
                Object.getPrototypeOf(Range.prototype) !== AbstractRange.prototype ||
                Object.getPrototypeOf(Range) !== AbstractRange || !(range instanceof AbstractRange) ||
                range === another || range === other || range === direct ||
                range.startContainer !== document || range.endContainer !== document ||
                direct.startContainer !== document || direct.endContainer !== document ||
                other.startContainer !== detached || other.endContainer !== detached ||
                range.startOffset !== 0 || range.endOffset !== 0 || !range.collapsed)
                return 'fresh native Range initial boundaries and brands';
            for (const name of ['startContainer', 'endContainer', 'startOffset', 'endOffset', 'collapsed']) {
                const {get, set, enumerable, configurable} = Object.getOwnPropertyDescriptor(AbstractRange.prototype, name);
                if (typeof get !== 'function' || set !== undefined || !enumerable || !configurable ||
                    Object.hasOwn(Range.prototype, name) ||
                    !rejects(() => get.call(Object.create(range)), 'TypeError'))
                    return 'incorrect AbstractRange getter ' + name;
            }
            const parent = document.createElement('div');
            const text = document.createTextNode('A\ud83d\ude00B');
            parent.appendChild(text);
            document.body.appendChild(parent);
            const invalid = document.implementation.createDocumentType('r', '', '');
            const {setStart, setEnd} = Range.prototype;
            let converted = 0;
            const offset = {valueOf() { converted++; return 2; }};
            if (!rejects(() => setStart.call({}, text, offset), 'TypeError') ||
                !rejects(() => setEnd.call(range, {}, offset), 'TypeError') || converted !== 0)
                return 'brand the receiver and Node before conversion';
            const marker = {};
            try { range.setStart(text, {valueOf() { throw marker; }}); return 'conversion completed'; }
            catch (error) { if (error !== marker) throw error; }
            if (range.setStart(text, offset) !== undefined || converted !== 1 ||
                range.startContainer !== text || range.endContainer !== text ||
                range.startOffset !== 2 || range.endOffset !== 2 || !range.collapsed)
                return 'UTF-16 start and native opposite endpoint collapse';
            range.setEnd(text, 4);
            if (range.collapsed || range.endOffset !== 4 ||
                !rejects(() => range.setEnd(text, 5), 'IndexSizeError') ||
                !rejects(() => range.setStart(invalid, 0), 'InvalidNodeTypeError') ||
                range.startOffset !== 2 || range.endOffset !== 4)
                return 'validated native boundaries and unchanged state after invalid setters';
            const suffix = text.splitText(1);
            if (range.startContainer !== suffix || range.endContainer !== suffix ||
                range.startOffset !== 1 || range.endOffset !== 3 || range.collapsed)
                return 'native split transfers exact UTF-16 boundaries';
            suffix.remove();
            if (range.startContainer !== parent || range.endContainer !== parent ||
                range.startOffset !== 1 || range.endOffset !== 1 || !range.collapsed)
                return 'native removal moves both boundaries to the same exact parent';
            return 'ok';
        })()"#,
    );
    assert_eq!(initial, "ok");
    host.stop().into_session().unwrap();
}

pub(super) fn borrowed_range_factory_uses_the_receiving_document_realm() {
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
    window_length::edit(&mut host, DomSourceEdit::insert_before(iframe, body, None));
    let observed = observe(
        &mut host,
        &permit,
        r"(() => {
            const child = window[0];
            const local = Document.prototype.createRange;
            const foreign = child.Document.prototype.createRange;
            if (typeof local !== 'function' || typeof foreign !== 'function')
                return 'both realms must expose the native Range factory';
            const childRange = local.call(child.document);
            const topRange = foreign.call(document);
            const direct = new child.Range();
            const detached = child.document.implementation.createHTMLDocument();
            const detachedRange = local.call(detached);
            if (childRange.constructor !== child.Range || topRange.constructor !== Range ||
                direct.constructor !== child.Range || detachedRange.constructor !== child.Range ||
                childRange.startContainer !== child.document || topRange.startContainer !== document ||
                direct.startContainer !== child.document || detachedRange.startContainer !== detached)
                return 'factory and constructor did not use the correct native Document and realm';
            childRange.setStart(document.body, 0);
            if (childRange.startContainer !== document.body || childRange.endContainer !== document.body ||
                childRange.constructor !== child.Range || !childRange.collapsed)
                return 'changing both boundaries must preserve the Range creation identity';
            return 'ok';
        })()",
    );
    assert_eq!(observed, "ok");
    host.stop().into_session().unwrap();
}
