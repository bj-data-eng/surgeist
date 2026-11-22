use super::{window_length::observe, *};

pub(super) fn document_create_event_exposes_the_native_uninitialized_event() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 120_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let observed = observe(
        &mut host,
        &permit,
        r"(() => {
        const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, 'createEvent');
        if (typeof descriptor?.value !== 'function') return 'missing Document createEvent method';
        const {value: make, writable, configurable, enumerable} = descriptor;
        if (!writable || !configurable || !enumerable || make.name !== 'createEvent' ||
            make.length !== 1 || Object.hasOwn(make, 'prototype')) return 'incorrect method descriptor';
        let touched = 0;
        const input = {toString() { touched++; return 'eVeNt'; }};
        for (const receiver of [{}, Object.create(document), new Proxy(document, {}), document.body]) {
            try { make.call(receiver, input); return 'forged Document accepted'; }
            catch (error) { if (!(error instanceof TypeError) || touched !== 0) throw error; }
        }
        try { make.call(document); return 'missing required interface name accepted'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        const thrown = {};
        try { make.call(document, {toString() { throw thrown; }}); return 'coercion throw was lost'; }
        catch (error) { if (error !== thrown) throw error; }
        for (const bad of ['unknown-legacy-interface', ' Event', 'Event ', 'ſvgEvents']) {
            try { document.createEvent(bad); return 'invalid legacy interface name accepted'; }
            catch (error) {
                if (!(error instanceof DOMException) || error.name !== 'NotSupportedError') throw error;
            }
        }
        const event = document.createEvent(input);
        if (touched !== 1 || Object.getPrototypeOf(event) !== Event.prototype)
            return 'the factory must convert once and expose the original Event prototype';
        const identities = ['Event', 'eVeNtS', 'HTMLEvents', 'SVGEvents'].map(name => document.createEvent(name));
        if (new Set([event, ...identities]).size !== 5 || identities.some(value => !(value instanceof Event)))
            return 'each accepted alias must produce a fresh base Event';
        const trusted = Object.getOwnPropertyDescriptor(event, 'isTrusted');
        if (event.type !== '' || event.target !== null || event.currentTarget !== null ||
            event.bubbles || event.cancelable || event.composed || event.isTrusted !== false ||
            typeof trusted?.get !== 'function' || trusted.set !== undefined || trusted.configurable)
            return 'initial native Event state or own isTrusted property';
        try { document.dispatchEvent(event); return 'uninitialized Event dispatched'; }
        catch (error) {
            if (!(error instanceof DOMException) || error.name !== 'InvalidStateError') throw error;
        }
        if (!document.dispatchEvent(new Event(''))) return 'initialized empty-type event control';
        let delivered = 0;
        document.addEventListener('legacy-probe', function(value) {
            if (value !== event || this !== document || value.target !== document ||
                value.currentTarget !== document || value.isTrusted || !value.bubbles || !value.cancelable)
                throw new Error('native delivery used an incorrect Event');
            delivered++;
            value.preventDefault();
        });
        if (event.initEvent('legacy-probe', true, true) !== undefined ||
            document.dispatchEvent(event) !== false || delivered !== 1 || !event.defaultPrevented ||
            event.currentTarget !== null) return 'initialized Event must enter the native dispatcher';
        for (const detached of [new Document(), document.implementation.createHTMLDocument(),
            document.implementation.createDocument('urn:fixture', 'root')]) {
            const separate = make.call(detached, 'Event');
            if (Object.getPrototypeOf(separate) !== Event.prototype || separate === event ||
                separate.isTrusted !== false || separate.type !== '')
                return 'detached Documents must use their actual creation Global';
        }
        return 'ok';
    })()",
    );
    assert_eq!(observed, "ok");
    host.stop().into_session().unwrap();
}

pub(super) fn borrowed_create_event_uses_the_receiving_document_realm() {
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
        (8_000_000, 6_000_000, 180_000),
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
        const receiving = child.document;
        const localMake = Document.prototype.createEvent;
        const foreignMake = child.Document.prototype.createEvent;
        if (receiving === document || !receiving) return 'missing real same-origin child Document';
        if (typeof localMake !== 'function' || typeof foreignMake !== 'function')
            return 'both realms must expose Document createEvent';
        if (localMake === foreignMake || child.Event === Event) return 'realm interface identity lost';
        const foreign = localMake.call(receiving, 'Event');
        const local = foreignMake.call(document, 'Event');
        const detached = child.document.implementation.createHTMLDocument();
        const detachedEvent = localMake.call(detached, 'HTMLEvents');
        if (Object.getPrototypeOf(foreign) !== child.Event.prototype ||
            Object.getPrototypeOf(detachedEvent) !== child.Event.prototype ||
            Object.getPrototypeOf(local) !== Event.prototype ||
            foreign instanceof Event || local instanceof child.Event ||
            foreign.isTrusted || detachedEvent.isTrusted || local.isTrusted)
            return 'Event projection must use the receiver Document creation realm';
        for (const [receiver, name, expected] of [
            [{}, 'Event', child.TypeError], [document, 'UNKNOWN', child.DOMException]
        ]) {
            try { foreignMake.call(receiver, name); return 'expected foreign method exception'; }
            catch (error) {
                if (!(error instanceof expected) || error instanceof TypeError || error instanceof DOMException)
                    return 'exception must use the method realm';
            }
        }
        let observed = 0;
        document.addEventListener('realm-event', value => {
            if (value === foreign && value.target === document && value.currentTarget === document)
                observed++;
        });
        if (foreign.initEvent('realm-event', false, false) !== undefined ||
            foreign.type !== 'realm-event' || !document.dispatchEvent(foreign) || observed !== 1)
            return 'foreign Event init and dispatcher must use the same exact object';
        return 'ok';
    })()",
    );
    assert_eq!(observed, "ok");
    host.stop().into_session().unwrap();
}
