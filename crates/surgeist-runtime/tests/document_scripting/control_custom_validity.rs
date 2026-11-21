//! The attached native controls preserve the canonical custom validity message.

use super::*;

pub(super) fn attached_controls_store_native_custom_validity() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const families = [
            [HTMLButtonElement, 'button', ['type', 'value', 'setCustomValidity']],
            [HTMLInputElement, 'input', ['defaultValue', 'value', 'setCustomValidity', 'align']],
            [HTMLSelectElement, 'select', ['selectedIndex', 'value', 'setCustomValidity']],
            [HTMLTextAreaElement, 'textarea', ['value', 'textLength', 'setCustomValidity']],
        ];
        const methods = [];
        for (const [constructor, , order] of families) {
            const prototype = constructor.prototype;
            const d = Object.getOwnPropertyDescriptor(prototype, 'setCustomValidity');
            if (typeof d?.value !== 'function') {
                return 'missing ' + constructor.name + '.prototype.setCustomValidity method';
            }
            if (d.value.name !== 'setCustomValidity' || d.value.length !== 1 ||
                !d.writable || !d.enumerable || !d.configurable) {
                return constructor.name + ' custom validity descriptor';
            }
            if (Object.getOwnPropertyNames(prototype).filter(key => order.includes(key)).join() !== order.join()) {
                return constructor.name + ' custom validity declaration order';
            }
            methods.push(d.value);
        }
        const html = 'http://www.w3.org/1999/xhtml';
        const place = document.body.appendChild(document.createElement('div'));
        const controls = families.map(([, local]) => place.appendChild(document.createElementNS(html, 'p:' + local)));
        const spare = place.appendChild(document.createElement('textarea'));
        const [button, input] = controls;
        button.disabled = true;
        input.type = 'hidden';
        if (!button.disabled || input.type !== 'hidden') return 'native barred control setup';

        let conversions = 0;
        const poison = { [Symbol.toPrimitive]() { conversions++; return 'wrong'; } };
        const thrown = {};
        for (const [i, [constructor, local]] of families.entries()) {
            const prototype = constructor.prototype;
            const target = controls[i];
            const method = methods[i];
            if (Object.getPrototypeOf(target) !== prototype) return constructor.name + ' prefixed HTML receiver';
            const foreign = document.createElementNS('urn:foreign', local);
            Object.setPrototypeOf(foreign, prototype);
            const wrong = [controls[(i + 1) % controls.length], foreign, Object.create(prototype),
                document.createElementNS(html, local.toUpperCase())];
            for (const receiver of wrong) {
                try {
                    method.call(receiver, poison);
                    return constructor.name + ' accepted a wrong receiver';
                } catch (error) {
                    if (!(error instanceof TypeError) || conversions !== 0) {
                        return constructor.name + ' brand must precede argument conversion';
                    }
                }
            }
            try {
                method.call(target);
                return constructor.name + ' accepted an omitted required DOMString';
            } catch (error) {
                if (!(error instanceof TypeError)) return constructor.name + ' omission TypeError';
            }
            try {
                method.call(target, {toString() { throw thrown; }});
                return constructor.name + ' accepted abrupt conversion';
            } catch (error) {
                if (error !== thrown) return constructor.name + ' original thrown value';
            }
        }
        let invalidEvents = 0;
        for (const node of [...controls, spare]) node.addEventListener('invalid', () => invalidEvents++);
        globalThis.controlCustomValidityProbe = {controls, spare, methods, poison,
            conversions: () => conversions, invalidEvents: () => invalidEvents};
        return 'ok';
    })()"#,
    );

    let before = host.snapshot().unwrap();
    let html = before
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = before.node(html).unwrap().children()[1];
    let [place]: [NodeId; 1] = before.node(body).unwrap().children().try_into().unwrap();
    let controls: [NodeId; 5] = before.node(place).unwrap().children().try_into().unwrap();
    let [button, input, select, textarea, spare] = controls;
    assert_eq!(messages(&before, controls), [&[][..]; 5]);

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {controls, spare, methods, poison, conversions, invalidEvents} = controlCustomValidityProbe;
        const [button, input, select, textarea] = controls;
        if (methods[0].call(button, null, poison) !== undefined || conversions() !== 0 ||
            methods[1].call(input, undefined) !== undefined ||
            methods[2].call(select, 'one\r\ntwo\rthree') !== undefined) {
            return 'custom validity DOMString conversion and completion';
        }
        let receiver = textarea;
        const hints = [];
        const result = methods[3].call(receiver, { [Symbol.toPrimitive](hint) {
            hints.push(hint);
            receiver = spare;
            textarea.remove();
            if (textarea.parentNode !== null || methods[3].call(spare, 'nested') !== undefined) {
                throw new Error('native reentry did not complete');
            }
            return 'first\r\nsecond\r\ud800\0\udc00';
        } });
        return result === undefined && hints.join() === 'string' && receiver === spare &&
            textarea.parentNode === null && spare.parentNode !== null && invalidEvents() === 0
            ? 'ok' : 'original detached receiver and no invalid event from message storage';
    })()"#,
    );

    let stored = host.snapshot().unwrap();
    let expected_textarea = "first\nsecond\n"
        .encode_utf16()
        .chain([0xd800, 0, 0xdc00])
        .collect::<Vec<_>>();
    assert_eq!(
        messages(&stored, controls),
        [
            ascii_units(b"null").as_slice(),
            ascii_units(b"undefined").as_slice(),
            ascii_units(b"one\ntwo\nthree").as_slice(),
            expected_textarea.as_slice(),
            ascii_units(b"nested").as_slice(),
        ]
    );
    assert_eq!(stored.node(textarea).unwrap().parent(), None);
    assert_eq!(stored.node(spare).unwrap().parent(), Some(place));
    for control in [button, input, select] {
        assert_eq!(stored.node(control).unwrap().parent(), Some(place));
    }

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {controls, methods, invalidEvents} = controlCustomValidityProbe;
        for (const [i, node] of controls.entries()) {
            if (methods[i].call(node, '') !== undefined) return node.localName + ' empty string completion';
        }
        return invalidEvents() === 0 ? 'ok' : 'clearing a custom message fired invalid';
    })()"#,
    );

    let cleared = host.snapshot().unwrap();
    assert_eq!(
        messages(&cleared, controls),
        [&[], &[], &[], &[], ascii_units(b"nested").as_slice()]
    );
    assert_eq!(cleared.incomplete_source_operation(), None);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

fn messages(snapshot: &DomSnapshot, controls: [NodeId; 5]) -> [&[u16]; 5] {
    let [button, input, select, textarea, spare] = controls.map(|id| snapshot.node(id).unwrap());
    [
        button
            .as_button()
            .unwrap()
            .state()
            .custom_validity_message()
            .code_units(),
        input
            .as_input()
            .unwrap()
            .state()
            .custom_validity_message()
            .code_units(),
        select
            .as_select()
            .unwrap()
            .state()
            .custom_validity_message()
            .code_units(),
        textarea
            .as_textarea()
            .unwrap()
            .state()
            .custom_validity_message()
            .code_units(),
        spare
            .as_textarea()
            .unwrap()
            .state()
            .custom_validity_message()
            .code_units(),
    ]
}

fn ascii_units(text: &[u8]) -> Vec<u16> {
    text.iter().copied().map(u16::from).collect()
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
