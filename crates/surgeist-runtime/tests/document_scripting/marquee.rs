//! The attached Marquee's exact reflections and native running state.

use super::*;

pub(super) fn attached_marquee_uses_native_reflections_and_running_state() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 200_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const type = HTMLMarqueeElement;
            const prototype = type.prototype;
            if (typeof Object.getOwnPropertyDescriptor(prototype, 'stop')?.value !== 'function') {
                return 'missing HTMLMarqueeElement.prototype.stop method';
            }
            const attributes = ['behavior', 'bgColor', 'direction', 'height', 'hspace', 'loop',
                'scrollAmount', 'scrollDelay', 'trueSpeed', 'vspace', 'width'];
            if (Object.keys(prototype).join() !== [...attributes, 'start', 'stop'].join() ||
                Object.getPrototypeOf(type) !== HTMLElement || Object.getPrototypeOf(prototype) !== HTMLElement.prototype) {
                return 'native Marquee prototype and declaration order';
            }
            for (const name of attributes) {
                const d = Object.getOwnPropertyDescriptor(prototype, name);
                if (typeof d?.get !== 'function' || typeof d.set !== 'function' ||
                    d.get.name !== 'get ' + name || d.get.length !== 0 || d.set.name !== 'set ' + name || d.set.length !== 1 ||
                    !d.enumerable || !d.configurable || 'value' in d || 'writable' in d) return 'Marquee accessor ' + name;
            }
            for (const name of ['start', 'stop']) {
                const d = Object.getOwnPropertyDescriptor(prototype, name);
                if (typeof d?.value !== 'function' || d.value.name !== name || d.value.length !== 0 ||
                    !d.writable || !d.enumerable || !d.configurable || 'get' in d || 'set' in d) return 'Marquee method ' + name;
            }
            const marquee = document.body.appendChild(document.createElement('marquee'));
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:marquee');
            const foreign = document.createElementNS('urn:foreign', 'marquee');
            const uppercase = document.createElementNS('http://www.w3.org/1999/xhtml', 'MARQUEE');
            if (Object.getPrototypeOf(marquee) !== prototype || !(prefixed instanceof type) ||
                prefixed.behavior !== '' || prefixed.stop() !== undefined || prefixed.start() !== undefined) {
                return 'native Marquee and prefixed receiver';
            }
            Object.setPrototypeOf(foreign, prototype);
            const loop = Object.getOwnPropertyDescriptor(prototype, 'loop');
            for (const callable of [loop.get, loop.set, prototype.start, prototype.stop]) {
                try { Reflect.construct(function () {}, [], callable); return 'constructible Marquee member'; }
                catch (error) { if (!(error instanceof TypeError)) return 'nonconstructible Marquee member'; }
                for (const other of [null, {}, Object.create(prototype), new Proxy(marquee, {}),
                    document.createElement('div'), foreign, uppercase]) {
                    let coerced = false;
                    const input = { [Symbol.toPrimitive]() { coerced = true; return 1; } };
                    try { Reflect.apply(callable, other, [input]); return 'accepted foreign Marquee receiver'; }
                    catch (error) { if (!(error instanceof TypeError) || coerced) return 'Marquee brand before conversion'; }
                }
            }
            globalThis.marqueeProbe = marquee;
            return 'ok';
        })()"#,
    );
    let initial = host.snapshot().unwrap();
    let root = initial
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = initial.node(root).unwrap().children()[1];
    let node = *initial.node(body).unwrap().children().last().unwrap();
    let (running, index) = running_state(&initial, node);
    assert!(running);
    assert_eq!(index, 0);
    observe(
        &mut host,
        &permit,
        "marqueeProbe.start() === undefined ? 'ok' : 'start return'",
    );
    assert_eq!(host.snapshot().unwrap().stamp(), initial.stamp());
    observe(
        &mut host,
        &permit,
        r#"marqueeProbe.stop({toString() { throw Error('method argument conversion'); }}) === undefined
            ? 'ok' : 'stop return'"#,
    );
    let stopped = host.snapshot().unwrap();
    assert_ne!(stopped.stamp(), initial.stamp());
    assert_eq!(running_state(&stopped, node), (false, index));
    assert_eq!(running_state(&initial, node), (true, index));
    observe(
        &mut host,
        &permit,
        "marqueeProbe.stop() === undefined ? 'ok' : 'stop no-op return'",
    );
    assert_eq!(host.snapshot().unwrap().stamp(), stopped.stamp());
    observe(
        &mut host,
        &permit,
        "marqueeProbe.start() === undefined ? 'ok' : 'restart return'",
    );
    let restarted = host.snapshot().unwrap();
    assert_ne!(restarted.stamp(), stopped.stamp());
    assert_eq!(running_state(&restarted, node), (true, index));

    observe(
        &mut host,
        &permit,
        r#"(() => {
            const marquee = marqueeProbe;
            const exact = ' Raw\ud800\0\udc00 ';
            for (const [name, attr] of [['behavior','behavior'], ['bgColor','bgcolor'],
                ['direction','direction'], ['height','height'], ['width','width']]) {
                marquee.setAttributeNS('urn:foreign', 'x:' + attr, 'foreign');
                if (marquee[name] !== '') return 'missing null-namespace string ' + name;
                marquee[name] = null;
                if (marquee[name] !== 'null' || marquee.getAttributeNS(null, attr) !== 'null') return 'ordinary null conversion ' + name;
                marquee[name] = exact;
                if (marquee[name] !== exact || marquee.getAttributeNS(null, attr) !== exact ||
                    marquee.getAttributeNS('urn:foreign', attr) !== 'foreign') return 'raw UTF-16 ' + name;
            }
            marquee.setAttributeNS('urn:foreign', 'x:truespeed', 'foreign');
            if (marquee.trueSpeed !== false) return 'null namespace of trueSpeed';
            marquee.setAttributeNS(null, 'truespeed', 'false');
            if (marquee.trueSpeed !== true) return 'trueSpeed reflects presence';
            marquee.trueSpeed = false;
            if (marquee.trueSpeed || marquee.hasAttributeNS(null, 'truespeed')) return 'false trueSpeed removes';
            marquee.trueSpeed = {get [Symbol.toPrimitive]() { throw Error('Boolean coerced an object'); }};
            if (!marquee.trueSpeed || marquee.getAttributeNS(null, 'truespeed') !== '') return 'Boolean writes empty presence';
            for (const [name, attr, fallback] of [['hspace','hspace',0], ['vspace','vspace',0],
                ['scrollAmount','scrollamount',6], ['scrollDelay','scrolldelay',85]]) {
                marquee.setAttributeNS('urn:foreign', 'x:' + attr, '91');
                if (marquee[name] !== fallback) return 'missing unsigned attribute ' + name;
                for (const [raw, expected] of [['0',0], [' +17tail',17], ['2147483647',2147483647],
                    ['2147483648',fallback], ['-2',fallback], ['bad',fallback], ['8\ud800',8]]) {
                    marquee.setAttributeNS(null, attr, raw);
                    if (marquee[name] !== expected || marquee.getAttributeNS(null, attr) !== raw) return 'unsigned getter ' + name;
                }
                for (const [input, value] of [[0,0], [2147483647,2147483647], [2147483648,fallback],
                    [-1,fallback], [4294967296,0], [4294967297,1], [7.9,7]]) {
                    marquee[name] = input;
                    if (marquee[name] !== value || marquee.getAttributeNS(null, attr) !== String(value)) return 'unsigned setter ' + name;
                }
            }
            return 'ok';
        })()"#,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(node, Namespace::none(), "scrolldelay", "2147483648");
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    observe(
        &mut host,
        &permit,
        "marqueeProbe.scrollDelay === 85 ? 'ok' : 'live native unsigned read'",
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
            const marquee = marqueeProbe;
            marquee.setAttributeNS('urn:foreign', 'x:loop', '7');
            if (marquee.loop !== -1) return 'missing null-namespace loop';
            for (const [raw, expected] of [['bad',-1], ['0',-1], ['-4',-1], [' +003tail',3],
                ['2147483647',2147483647], ['2147483648',-1], ['9999999999999999999999',-1]]) {
                marquee.setAttributeNS(null, 'loop', raw);
                if (marquee.loop !== expected || marquee.getAttributeNS(null, 'loop') !== raw) return 'loop getter and raw range';
            }
            marquee.setAttributeNS(null, 'loop', ' +003tail');
            for (const ignored of [3, 0, -2, 4294967296, 2147483648, NaN]) {
                marquee.loop = ignored;
                if (marquee.getAttributeNS(null, 'loop') !== ' +003tail') return 'equal or disallowed loop write';
            }
            marquee.loop = 4294967298;
            if (marquee.loop !== 2 || marquee.getAttributeNS(null, 'loop') !== '2') return 'canonical wrapped signed loop';
            marquee.setAttributeNS(null, 'loop', '2147483648');
            let conversions = 0;
            marquee.loop = {valueOf() {
                conversions++;
                if (marquee.loop !== -1 || marquee.getAttributeNS(null, 'loop') !== '2147483648') throw Error('wrong overflow basis');
                return -1;
            }};
            if (conversions !== 1 || marquee.loop !== -1 || marquee.getAttributeNS(null, 'loop') !== '-1') {
                return 'overflow read fallback must remain distinct for loop equality';
            }
            marquee.setAttributeNS(null, 'loop', 'invalid');
            marquee.loop = -1;
            return marquee.getAttributeNS(null, 'loop') === 'invalid' && marquee.getAttributeNS('urn:foreign', 'loop') === '7'
                ? 'ok' : 'invalid raw loop maps to the equal default';
        })()"#,
    );
    let before_ignore = host.snapshot().unwrap();
    observe(
        &mut host,
        &permit,
        "marqueeProbe.loop = -1; marqueeProbe.loop === -1 ? 'ok' : 'ignored loop result'",
    );
    assert_eq!(host.snapshot().unwrap().stamp(), before_ignore.stamp());

    observe(
        &mut host,
        &permit,
        r#"(() => {
            const calls = [];
            class ObservedMarquee extends HTMLMarqueeElement {
                static observedAttributes = ['loop', 'scrollamount'];
                attributeChangedCallback(name, previous, value, namespace) {
                    calls.push([this, name, previous, value, namespace,
                        name === 'loop' ? this.loop : this.scrollAmount]);
                }
            }
            customElements.define('x-marquee-observed', ObservedMarquee, {extends: 'marquee'});
            const watched = document.body.appendChild(document.createElement('marquee', {is: 'x-marquee-observed'}));
            if (!(watched instanceof ObservedMarquee)) return 'actual customized Marquee';
            watched.loop = 3;
            const hints = [];
            watched.loop = { [Symbol.toPrimitive](hint) {
                hints.push(hint);
                watched.setAttributeNS(null, 'loop', '0007tail');
                return 7;
            } };
            if (hints.join() !== 'number' || watched.getAttributeNS(null, 'loop') !== '0007tail') {
                return 'loop setter compares the current raw count after one conversion';
            }
            watched.loop = 0;
            watched.loop = 8;
            watched.scrollAmount = 2147483648;
            const thrown = {};
            try { watched.loop = {valueOf() { throw thrown; }}; return 'throwing conversion returned'; }
            catch (error) { if (error !== thrown) return 'original loop exception'; }
            const expected = [
                ['loop', null, '3', null, 3],
                ['loop', '3', '0007tail', null, 7],
                ['loop', '0007tail', '8', null, 8],
                ['scrollamount', null, '6', null, 6],
            ];
            return calls.every(call => call[0] === watched) &&
                JSON.stringify(calls.map(call => call.slice(1))) === JSON.stringify(expected) && watched.loop === 8
                ? 'ok' : 'native Marquee reactions, canonical updates and ignored writes';
        })()"#,
    );
    assert_eq!(
        running_state(&host.snapshot().unwrap(), node),
        (true, index)
    );
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

fn running_state(snapshot: &DomSnapshot, node: NodeId) -> (bool, u128) {
    let state = snapshot
        .node(node)
        .unwrap()
        .as_element()
        .unwrap()
        .marquee_state()
        .unwrap();
    (state.turned_on(), state.current_loop_index())
}

fn observe(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit, script: &str) {
    let result = host
        .in_document(permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
}
