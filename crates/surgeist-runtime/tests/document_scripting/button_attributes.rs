//! The attached Button reflects native content and observes its live owner and type.

use super::*;

pub(super) fn attached_button_attributes_use_live_submitter_and_form_state() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const prototype = HTMLButtonElement.prototype;
        const names = ['command', 'disabled', 'form', 'formEnctype', 'formMethod', 'formNoValidate',
            'formTarget', 'name', 'type', 'value'];
        const callbacks = [];
        for (const name of names) {
            const d = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof d?.get !== 'function') return 'missing HTMLButtonElement.prototype.' + name + ' getter';
            if (!d.enumerable || !d.configurable || d.get.name !== 'get ' + name || d.get.length !== 0 ||
                (name === 'form' ? d.set !== undefined :
                    typeof d.set !== 'function' || d.set.name !== 'set ' + name || d.set.length !== 1)) {
                return name + ' Button accessor descriptor';
            }
            callbacks.push(d.get);
            if (d.set) callbacks.push(d.set);
        }
        if (Object.getOwnPropertyNames(prototype).filter(name => names.includes(name)).join() !== names.join()) {
            return 'Button declaration order';
        }
        const html = 'http://www.w3.org/1999/xhtml';
        const svg = 'http://www.w3.org/2000/svg';
        const native = document.createElement('button');
        const prefixed = document.createElementNS(html, 'h:button');
        const upper = document.createElementNS(html, 'BUTTON');
        const foreign = document.createElementNS(svg, 'button');
        Object.setPrototypeOf(foreign, prototype);
        if (Object.getPrototypeOf(native) !== prototype || Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
            !(prefixed instanceof HTMLButtonElement) || prefixed.type !== 'submit' || prefixed.form !== null) {
            return 'exact native Button brands';
        }
        let conversions = 0;
        const argument = { [Symbol.toPrimitive]() { conversions++; return 'premature'; } };
        for (const callback of callbacks) {
            for (const receiver of [document.body, upper, foreign, Object.create(prototype), new Proxy(native, {})]) {
                try {
                    Reflect.apply(callback, receiver, [argument]);
                    return callback.name + ' accepted a non-Button receiver';
                } catch (error) {
                    if (!(error instanceof TypeError) || conversions !== 0) return callback.name + ' native brand first';
                }
            }
        }

        for (const [raw, canonical] of [
            ['ToGgLe-PoPoVeR', 'toggle-popover'], ['ShOw-PoPoVeR', 'show-popover'], ['HiDe-PoPoVeR', 'hide-popover'],
            ['ClOsE', 'close'], ['ReQuEsT-ClOsE', 'request-close'], ['ShOw-MoDaL', 'show-modal'],
            [' unknown ', ''], [' --not-a-custom-prefix', ''], ['', ''], ['--', '--'],
            ['--MiXeD\ud800\0\udc00 ', '--MiXeD\ud800\0\udc00 '],
        ]) {
            native.command = raw;
            if (native.command !== canonical || native.getAttributeNS(null, 'command') !== raw) {
                return 'local command state and exact raw setter';
            }
        }
        native.removeAttributeNS(null, 'command');
        native.setAttributeNS('urn:foreign', 'f:command', 'close');
        native.setAttributeNS('urn:foreign', 'f:commandfor', '');
        native.setAttributeNS(null, 'TYPE', 'reset');
        if (native.command !== '' || native.type !== 'submit') return 'Auto and command use exact attribute names';
        native.command = 'unknown';
        if (native.command !== '' || native.type !== 'button') return 'Auto checks command presence';
        native.removeAttributeNS(null, 'command');
        native.setAttributeNS(null, 'commandfor', '');
        if (native.type !== 'button') return 'Auto checks commandfor presence without resolving a target';
        native.removeAttributeNS(null, 'commandfor');
        const select = document.createElement('select');
        const foreignSelect = document.createElementNS(svg, 'select');
        const upperSelect = document.createElementNS(html, 'SELECT');
        if (native.type !== 'submit') return 'Auto on a detached Button';
        select.appendChild(native);
        if (native.type !== 'button') return 'Auto sees the ordinary HTML Select parent';
        native.command = '';
        native.setAttributeNS(null, 'commandfor', 'unresolved');
        native.type = 'SuBMiT';
        if (native.type !== 'submit' || native.getAttributeNS(null, 'type') !== 'SuBMiT') return 'explicit Submit precedence';
        native.type = 'ReSeT';
        if (native.type !== 'reset') return 'explicit Reset precedence';
        native.type = ' auto\ud800 ';
        native.removeAttributeNS(null, 'command');
        native.removeAttributeNS(null, 'commandfor');
        if (native.type !== 'button') return 'invalid type enters live Auto';
        for (const parent of [foreignSelect, upperSelect]) {
            parent.appendChild(native);
            if (native.type !== 'submit' || native.getAttributeNS(null, 'type') !== ' auto\ud800 ') {
                return 'Auto distinguishes exact HTML Select from a same-spelled parent';
            }
        }
        return 'ok';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const first = document.body.appendChild(document.createElement('form'));
        const next = document.body.appendChild(document.createElement('form'));
        const ownerId = 'button-owner\ud800';
        next.setAttributeNS(null, 'id', ownerId);
        first.setAttributeNS(null, 'method', 'post');
        first.setAttributeNS(null, 'enctype', 'multipart/form-data');
        const select = next.appendChild(document.createElement('select'));
        const calls = [];
        const typeRaw = ' final \ud800 ';
        const reacted = 'reacted\0\udc00';
        class ObservedButton extends HTMLButtonElement {
            static observedAttributes = ['form', 'type', 'name'];
            attributeChangedCallback(attribute, previous, value, namespace) {
                calls.push([this, attribute, previous, value, namespace, this.type, this.form]);
                if (attribute === 'type' && value === typeRaw) this.name = reacted;
            }
        }
        customElements.define('x-button-attributes-probe', ObservedButton, {extends: 'button'});
        const button = first.appendChild(document.createElement('button', {is: 'x-button-attributes-probe'}));
        const companion = first.appendChild(document.createElement('button'));
        companion.value = 'untouched';
        companion.command = '--Raw\ud800\0\udc00 ';
        if (companion.command !== '--Raw\ud800\0\udc00 ') return 'exact custom command on attached Button';
        button.type = 'ReSeT';
        if (!(button instanceof ObservedButton) || button.type !== 'reset' || button.form !== first) {
            return 'real non-submit Button and ancestor Form';
        }
        for (const [member, attribute, fallback, authored, canonical] of [
            ['formMethod', 'formmethod', 'get', 'PoSt', 'post'],
            ['formEnctype', 'formenctype', 'application/x-www-form-urlencoded', 'TeXt/PlAiN', 'text/plain'],
        ]) {
            button.setAttributeNS('urn:foreign', 'f:' + attribute, authored);
            if (button[member] !== '') return member + ' missing differs from the owner settings';
            for (const present of ['', ' ' + authored, 'invalid\ud800']) {
                button[member] = present;
                if (button[member] !== fallback || button.getAttributeNS(null, attribute) !== present) {
                    return member + ' present invalid content';
                }
            }
            button[member] = authored;
            if (button[member] !== canonical || button.getAttributeNS(null, attribute) !== authored) {
                return member + ' reflects known content on a real non-submit Button';
            }
        }

        const exact = 'raw\ud800\0\udc00 ';
        for (const [member, attribute] of [['name', 'name'], ['value', 'value'], ['formTarget', 'formtarget']]) {
            button.setAttributeNS('urn:foreign', 'f:' + attribute, 'foreign');
            button.setAttributeNS(null, attribute.toUpperCase(), 'uppercase');
            if (button[member] !== '') return member + ' missing exact attribute';
            button[member] = null;
            if (button[member] !== 'null') return member + ' ordinary DOMString conversion';
            button[member] = exact;
            if (button[member] !== exact || button.getAttributeNS(null, attribute) !== exact ||
                button.getAttributeNS(null, attribute.toUpperCase()) !== 'uppercase' ||
                button.getAttributeNS('urn:foreign', attribute) !== 'foreign') return member + ' exact UTF-16 reflection';
        }
        let converted = false;
        const truthy = {toString() { converted = true; }};
        for (const [member, attribute] of [['disabled', 'disabled'], ['formNoValidate', 'formnovalidate']]) {
            button.setAttributeNS('urn:foreign', 'f:' + attribute, '');
            if (button[member]) return member + ' ignores foreign presence';
            button.setAttributeNS(null, attribute, 'false');
            if (!button[member]) return member + ' reflects presence';
            button[member] = truthy;
            if (!button[member] || converted || button.getAttributeNS(null, attribute) !== '') {
                return member + ' Web IDL boolean and empty content';
            }
            button[member] = false;
            if (button[member] || button.getAttributeNS(null, attribute) !== null ||
                button.getAttributeNS('urn:foreign', attribute) !== '') return member + ' clears own presence';
        }
        calls.length = 0;
        globalThis.buttonAttributesProbe = {button, companion, first, next, select, ownerId, calls, typeRaw, reacted, exact};
        return 'ok';
    })()"#,
    );

    let initial = host.snapshot().unwrap();
    let html = initial
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = initial.node(html).unwrap().children()[1];
    let [first, next]: [NodeId; 2] = initial.node(body).unwrap().children().try_into().unwrap();
    let [button, companion]: [NodeId; 2] =
        initial.node(first).unwrap().children().try_into().unwrap();
    let [select]: [NodeId; 1] = initial.node(next).unwrap().children().try_into().unwrap();
    assert_eq!(form_owner(&initial, button), Some(first));
    assert_eq!(form_owner(&initial, companion), Some(first));

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {button, companion, first, next, select, ownerId, calls, typeRaw, reacted, exact} = buttonAttributesProbe;
        const setter = Object.getOwnPropertyDescriptor(HTMLButtonElement.prototype, 'type').set;
        let receiver = button;
        const hints = [];
        let reassociated = false;
        const result = setter.call(receiver, { [Symbol.toPrimitive](hint) {
            hints.push(hint);
            receiver = companion;
            select.appendChild(button);
            reassociated = button.form === next && companion.form === first;
            button.setAttributeNS(null, 'form', ownerId);
            return typeRaw;
        } });
        if (result !== undefined || hints.join() !== 'string' || !reassociated || button.parentNode !== select ||
            button.form !== next || button.form !== button.form || companion.form !== first ||
            button.type !== 'button' || button.name !== reacted || companion.value !== 'untouched' ||
            companion.getAttributeNS(null, 'type') !== null) return 'coercion keeps the original Button and new Form';
        const expected = [
            ['form', null, ownerId, null, 'reset', next],
            ['type', 'ReSeT', typeRaw, null, 'button', next],
            ['name', exact, reacted, null, 'button', next],
        ];
        if (calls.length !== expected.length || calls.some((call, index) =>
            call[0] !== button || expected[index].some((value, offset) => value !== call[offset + 1]))) {
            return 'synchronous native Button reaction and original-receiver reentry';
        }
        const thrown = {};
        try {
            setter.call(button, {toString() { throw thrown; }});
            return 'abrupt Button type conversion returned';
        } catch (error) {
            return error === thrown && calls.length === expected.length && button.getAttributeNS(null, 'type') === typeRaw
                ? 'ok' : 'abrupt Button type conversion changed native state';
        }
    })()"#,
    );

    let final_state = host.snapshot().unwrap();
    assert_eq!(final_state.node(button).unwrap().parent(), Some(select));
    assert_eq!(form_owner(&final_state, button), Some(next));
    assert_eq!(form_owner(&initial, button), Some(first));
    assert_eq!(form_owner(&final_state, companion), Some(first));
    assert_eq!(
        raw(&final_state, button, "form"),
        Some(&units("button-owner", &[0xd800]))
    );
    assert_eq!(
        raw(&final_state, button, "type"),
        Some(&units(" final ", &[0xd800, 32]))
    );
    assert_eq!(
        raw(&final_state, button, "name"),
        Some(&units("reacted", &[0, 0xdc00]))
    );
    let exact = units("raw", &[0xd800, 0, 0xdc00, 32]);
    assert_eq!(raw(&final_state, button, "value"), Some(&exact));
    assert_eq!(raw(&final_state, button, "formtarget"), Some(&exact));
    assert_eq!(
        raw(&final_state, button, "formmethod"),
        Some(&DocumentString::from("PoSt"))
    );
    assert_eq!(
        raw(&final_state, button, "formenctype"),
        Some(&DocumentString::from("TeXt/PlAiN"))
    );
    assert_eq!(raw(&final_state, button, "disabled"), None);
    assert_eq!(raw(&final_state, button, "formnovalidate"), None);
    assert_eq!(raw(&final_state, companion, "type"), None);
    assert_eq!(
        raw(&final_state, companion, "command"),
        Some(&units("--Raw", &[0xd800, 0, 0xdc00, 32]))
    );
    assert_eq!(final_state.incomplete_source_operation(), None);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
}

fn form_owner(snapshot: &DomSnapshot, node: NodeId) -> Option<NodeId> {
    snapshot
        .node(node)
        .unwrap()
        .as_button()
        .unwrap()
        .form_owner()
        .unwrap()
}

fn raw<'a>(snapshot: &'a DomSnapshot, node: NodeId, attribute: &str) -> Option<&'a DocumentString> {
    snapshot
        .node(node)
        .unwrap()
        .attribute_ns(&Namespace::none(), &attribute.into())
}

fn units(prefix: &str, suffix: &[u16]) -> DocumentString {
    DocumentString::from_utf16(
        prefix
            .encode_utf16()
            .chain(suffix.iter().copied())
            .collect::<Vec<_>>(),
    )
}

pub(super) fn attached_button_resolves_its_live_command_target() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    observe(
        &mut host,
        &permit,
        r#"(() => {
        const proto = HTMLButtonElement.prototype;
        const d = Object.getOwnPropertyDescriptor(proto, 'commandForElement');
        if (typeof d?.get !== 'function') return 'missing native Button commandForElement';
        if (!d.enumerable || !d.configurable || d.get.name !== 'get commandForElement' ||
            d.get.length !== 0 || typeof d.set !== 'function' ||
            d.set.name !== 'set commandForElement' || d.set.length !== 1) return 'accessor metadata';
        if (Object.getOwnPropertyNames(proto).filter(name =>
            ['command', 'commandForElement', 'disabled'].includes(name)).join() !==
            'command,commandForElement,disabled') return 'Button declaration order';
        const zone = document.body.appendChild(document.createElement('section'));
        const button = zone.appendChild(document.createElementNS('http://www.w3.org/1999/xhtml', 'p:button'));
        const input = zone.appendChild(document.createElement('input'));
        const first = zone.appendChild(document.createElementNS('http://www.w3.org/2000/svg', 'svg'));
        const second = zone.appendChild(document.createElement('div'));
        const reject = f => {
            try { f(); } catch (error) { return error instanceof TypeError; }
            return false;
        };
        for (const receiver of [input, Object.create(proto), new Proxy(button, {}), null]) {
            if (!reject(() => d.get.call(receiver)) ||
                !reject(() => d.set.call(receiver, first))) return 'native Button receiver';
        }
        if (!reject(() => { button.commandForElement = document; }) ||
            !reject(() => { button.commandForElement = new Proxy(first, {}); }) ||
            button.hasAttribute('commandfor')) return 'native nullable Element operand';
        const id = 'Target\ud800\0';
        first.id = id;
        second.id = id;
        button.setAttributeNS('urn:foreign', 'p:commandfor', id);
        if (button.commandForElement !== null) return 'namespace is exact';
        button.setAttribute('commandfor', id);
        if (button.commandForElement !== first) return 'first exact Element ID';
        zone.insertBefore(second, first);
        if (button.commandForElement !== second) return 'live tree order';
        button.popoverTargetElement = first;
        button.commandForElement = second;
        if (button.getAttribute('commandfor') !== '' || button.popoverTargetElement !== first ||
            button.commandForElement !== second) return 'separate native weak slots';
        second.remove();
        if (button.commandForElement !== null) return 'explicit target is now out of scope';
        zone.appendChild(second);
        if (button.commandForElement !== second) return 'reconnected explicit target';
        button.setAttribute('commandfor', 'target\ud800\0');
        if (button.commandForElement !== null) return 'raw change clears explicit target and preserves exact case';
        const shadowHost = zone.appendChild(document.createElement('div'));
        const shadow = shadowHost.attachShadow({mode: 'open'});
        const inside = shadow.appendChild(document.createElement('button'));
        inside.commandForElement = first;
        button.commandForElement = inside;
        if (inside.commandForElement !== first || button.commandForElement !== null) {
            return 'reflected target scope uses ordinary target and shadow-including source ancestry';
        }
        button.commandForElement = undefined;
        if (button.hasAttribute('commandfor') || button.commandForElement !== null ||
            button.popoverTargetElement !== first) return 'nullable removal leaves popover intact';
        const changes = [];
        class CommandProbe extends HTMLButtonElement {
            static observedAttributes = ['commandfor', 'popovertarget'];
            attributeChangedCallback(name, previous, current) {
                changes.push([name, previous, current, this.commandForElement]);
                if (name === 'commandfor' && current === '') this.popoverTargetElement = first;
            }
        }
        customElements.define('x-command-probe', CommandProbe, {extends: 'button'});
        const observed = zone.appendChild(document.createElement('button', {is: 'x-command-probe'}));
        observed.commandForElement = second;
        if (changes.length !== 2 || changes[0][0] !== 'commandfor' || changes[0][1] !== null ||
            changes[0][2] !== '' || changes[0][3] !== second || changes[1][0] !== 'popovertarget' ||
            changes[1][3] !== second || observed.popoverTargetElement !== first) {
            return 'native command association and nested popover reaction';
        }
        zone.remove();
        return 'ok';
    })()"#,
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
