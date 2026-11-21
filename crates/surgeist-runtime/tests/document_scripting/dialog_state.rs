//! Native Dialog attributes and slot in the attached Window.

use super::*;

pub(super) fn attached_dialog_accessors_use_current_native_state() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 200_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const type = HTMLDialogElement;
            const prototype = type.prototype;
            if (typeof Object.getOwnPropertyDescriptor(prototype, 'open')?.get !== 'function') {
                return 'missing HTMLDialogElement.prototype.open getter';
            }
            const fields = ['open', 'returnValue', 'closedBy'];
            const own = Object.keys(prototype);
            if (Object.getPrototypeOf(type) !== HTMLElement || Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
                own.filter(name => fields.includes(name)).join() !== fields.join()) return 'Dialog accessor declaration order';
            const detached = document.createElement('dialog');
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:dialog');
            const foreign = document.createElementNS('http://www.w3.org/2000/svg', 'dialog');
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', 'DIALOG');
            if (Object.getPrototypeOf(detached) !== prototype || !(prefixed instanceof type) ||
                prefixed.prefix !== 'p' || detached.isConnected) return 'actual native Dialog receivers';
            Object.setPrototypeOf(foreign, prototype);
            for (const [index, name] of fields.entries()) {
                const d = Object.getOwnPropertyDescriptor(prototype, name);
                if (typeof d?.get !== 'function' || typeof d.set !== 'function' ||
                    d.get.name !== 'get ' + name || d.get.length !== 0 || d.set.name !== 'set ' + name || d.set.length !== 1 ||
                    !d.enumerable || !d.configurable || 'value' in d || 'writable' in d ||
                    d.get.call(prefixed) !== [false, '', 'none'][index]) return 'native Dialog descriptor for ' + name;
                for (const accessor of [d.get, d.set]) {
                    if (Object.hasOwn(accessor, 'prototype')) return 'Dialog accessor has constructor state';
                    try { Reflect.construct(function () {}, [], accessor); return 'constructible Dialog accessor'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'Dialog accessor constructor error'; }
                    for (const bad of [null, {}, new Proxy(detached, {}), Object.create(prototype),
                        document.createElement('div'), foreign, upper]) {
                        let coerced = false;
                        const input = { [Symbol.toPrimitive]() { coerced = true; return 'wrong receiver'; } };
                        try { Reflect.apply(accessor, bad, [input]); return 'accepted wrong Dialog receiver for ' + name; }
                        catch (error) {
                            if (!(error instanceof TypeError) || coerced) return 'Dialog brand before conversion';
                        }
                    }
                }
            }
            globalThis.dialogProbe = {detached, seen: [], events: [], exact: '\ud800\0\r\n\udc00'};
            return 'ok';
        })()"#,
    );
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {detached, exact} = dialogProbe;
            const namespace = 'urn:elsewhere';
            detached.setAttributeNS(namespace, 'p:open', '');
            detached.setAttributeNS(null, 'OPEN', '');
            if (detached.open) return 'open only sees exact null-namespace presence';
            detached.setAttributeNS(null, 'open', 'false');
            if (!detached.open || detached.isConnected) return 'detached authored open presence';
            detached.open = true;
            if (detached.getAttributeNS(null, 'open') !== '') return 'true replaces nonempty open content';
            detached.open = 0;
            if (detached.open || detached.hasAttributeNS(null, 'open') ||
                !detached.hasAttributeNS(null, 'OPEN') || !detached.hasAttributeNS(namespace, 'open')) {
                return 'false removes only exact open content';
            }

            detached.setAttributeNS(null, 'returnValue', 'content');
            if (detached.returnValue !== '') return 'returnValue starts in its independent slot';
            detached.returnValue = exact;
            if (detached.returnValue !== exact || detached.getAttributeNS(null, 'returnValue') !== 'content') {
                return 'returnValue exact slot write';
            }
            detached.returnValue = null;
            if (detached.returnValue !== 'null') return 'returnValue ordinary null conversion';
            detached.returnValue = undefined;
            if (detached.returnValue !== 'undefined') return 'returnValue ordinary undefined conversion';
            const setter = Object.getOwnPropertyDescriptor(HTMLDialogElement.prototype, 'returnValue').set;
            if (Reflect.apply(setter, detached, []) !== undefined || detached.returnValue !== 'undefined') {
                return 'returnValue omitted argument';
            }

            detached.setAttributeNS(namespace, 'p:closedby', 'any');
            detached.setAttributeNS(null, 'CLOSEDBY', 'any');
            if (detached.closedBy !== 'none') return 'nonmodal missing closedby uses Auto';
            for (const [input, raw, expected] of [
                ['AnY', 'AnY', 'any'], ['CLOSEREQUEST', 'CLOSEREQUEST', 'closerequest'],
                ['NoNe', 'NoNe', 'none'], ['', '', 'none'], ['auto', 'auto', 'none'],
                [' any ', ' any ', 'none'], [exact, exact, 'none'],
                [null, 'null', 'none'], [undefined, 'undefined', 'none'],
            ]) {
                detached.closedBy = input;
                if (detached.closedBy !== expected || detached.getAttributeNS(null, 'closedby') !== raw) {
                    return 'closedBy canonical getter and raw DOMString setter';
                }
            }
            detached.removeAttributeNS(null, 'closedby');
            if (detached.closedBy !== 'none' || detached.getAttributeNS(namespace, 'closedby') !== 'any' ||
                detached.getAttributeNS(null, 'CLOSEDBY') !== 'any') return 'closedBy leaves separate attributes';

            class ObservedDialog extends HTMLDialogElement {
                static observedAttributes = ['open', 'returnvalue', 'closedby'];
                attributeChangedCallback(name, previous, current, namespace) {
                    dialogProbe.seen.push([this, name, previous, current, namespace,
                        this.open, this.returnValue, this.closedBy]);
                }
            }
            customElements.define('x-state-dialog', ObservedDialog, {extends: 'dialog'});
            const dialog = document.body.appendChild(document.createElement('dialog', {is: 'x-state-dialog'}));
            if (!(dialog instanceof ObservedDialog) || !dialog.isConnected) return 'actual connected customized Dialog';
            for (const type of ['beforetoggle', 'toggle', 'cancel', 'close']) {
                dialog.addEventListener(type, event => dialogProbe.events.push(event.type));
            }
            dialogProbe.dialog = dialog;
            dialog.open = true;
            return dialog.open && dialog.getAttributeNS(null, 'open') === '' &&
                dialog.closedBy === 'none' && dialogProbe.seen.length === 1 ? 'ok' : 'connected first-add property hook';
        })()"#,
    );
    let snapshot = host.snapshot().unwrap();
    let root = snapshot
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(root).unwrap().children()[1];
    let node = *snapshot.node(body).unwrap().children().last().unwrap();
    let current = snapshot.node(node).unwrap();
    assert!(current.is_connected());
    assert_eq!(current.attribute(&"open".into()), Some(&"".into()));
    let view = current.as_dialog().unwrap();
    assert!(!view.is_modal());
    assert!(!view.has_toggle_task_tracker());
    let watcher = view
        .close_watcher()
        .expect("connected open established a watcher");
    assert_eq!(watcher.subject(), node);
    assert_eq!(watcher.global(), top.global());

    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {dialog} = dialogProbe;
            dialog.setAttributeNS(null, 'open', 'authored');
            dialog.open = {get [Symbol.toPrimitive]() { throw Error('Boolean coerced an object'); }};
            return dialog.open && dialog.getAttributeNS(null, 'open') === '' &&
                dialogProbe.seen.length === 3 ? 'ok' : 'true open reuses the connected watcher and rewrites raw content';
        })()"#,
    );
    let snapshot = host.snapshot().unwrap();
    assert_eq!(
        snapshot
            .node(node)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(watcher)
    );
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {dialog, detached, exact, seen, events} = dialogProbe;
            dialog.returnValue = 'before';
            const hints = [];
            let receiver = dialog;
            receiver.returnValue = { [Symbol.toPrimitive](hint) {
                hints.push(hint);
                receiver = detached;
                dialog.returnValue = 'inside conversion';
                dialog.closedBy = 'AnY';
                return exact;
            } };
            if (hints.join() !== 'string' || receiver !== detached || detached.returnValue !== 'undefined' ||
                dialog.returnValue !== exact || dialog.hasAttributeNS(null, 'returnvalue')) {
                return 'returnValue retains the original node and converts once';
            }
            const thrown = {};
            try { dialog.returnValue = {toString() { throw thrown; }}; return 'returnValue accepted thrown conversion'; }
            catch (error) { if (error !== thrown || dialog.returnValue !== exact) return 'original returnValue exception'; }
            dialog.closedBy = exact;
            dialog.open = false;
            const expected = [
                ['open', null, '', null, true, '', 'none'],
                ['open', '', 'authored', null, true, '', 'none'],
                ['open', 'authored', '', null, true, '', 'none'],
                ['closedby', null, 'AnY', null, true, 'inside conversion', 'any'],
                ['closedby', 'AnY', exact, null, true, exact, 'none'],
                ['open', '', null, null, false, exact, 'none'],
            ];
            return !dialog.open && dialog.closedBy === 'none' && dialog.returnValue === exact && events.length === 0 &&
                seen.every(call => call[0] === dialog) &&
                JSON.stringify(seen.map(call => call.slice(1))) === JSON.stringify(expected)
                ? 'ok' : 'original Dialog reactions and independent slot after direct removal';
        })()"#,
    );
    let snapshot = host.snapshot().unwrap();
    let current = snapshot.node(node).unwrap();
    let exact = DocumentString::from_utf16([0xd800, 0, 0x0d, 0x0a, 0xdc00]);
    assert_eq!(current.attribute(&"open".into()), None);
    assert_eq!(current.attribute(&"closedby".into()), Some(&exact));
    let dialog = current.as_dialog().unwrap();
    assert_eq!(dialog.return_value(), &exact);
    assert!(!dialog.has_close_watcher());
    assert!(!dialog.is_modal());
    assert!(!dialog.has_toggle_task_tracker());
    observe(
        &mut host,
        &permit,
        "dialogProbe.dialog.returnValue = dialogProbe.exact; dialogProbe.seen.length === 6 ? 'ok' : 'equal slot write'",
    );
    assert_eq!(host.snapshot().unwrap().stamp(), snapshot.stamp());

    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_dialog_return_value(node, DocumentString::from_utf16([0xdc00, 0, 0xd800]));
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    observe(
        &mut host,
        &permit,
        r#"dialogProbe.dialog.returnValue === '\udc00\0\ud800' && dialogProbe.seen.length === 6
            ? 'ok' : 'existing wrapper observes the exact native slot edit'"#,
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

fn observe(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit, script: &str) {
    let result = host
        .in_document(permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
}
