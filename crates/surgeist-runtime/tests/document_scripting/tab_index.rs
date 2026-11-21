//! Live static tabIndex in an attached Window.

use super::*;

pub(super) fn attached_tab_index_uses_includer_brands_and_current_native_inputs() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 200_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
            if (typeof Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'tabIndex')?.get !== 'function') {
                return 'missing HTMLElement.prototype.tabIndex getter';
            }
            const htmlNS = 'http://www.w3.org/1999/xhtml';
            const svgNS = 'http://www.w3.org/2000/svg';
            const mathNS = 'http://www.w3.org/1998/Math/MathML';
            const foreignNS = 'urn:foreign';
            const html = name => document.createElement(name);
            const svg = name => document.createElementNS(svgNS, name);
            const math = name => document.createElementNS(mathNS, name);
            const nodes = [html('button'), svg('p:a'), math('p:a')];
            const prototypes = [HTMLElement.prototype, SVGElement.prototype, MathMLElement.prototype];
            const foreign = document.createElementNS(foreignNS, 'a');
            if ('tabIndex' in foreign || Object.hasOwn(Element.prototype, 'tabIndex') ||
                Object.hasOwn(HTMLButtonElement.prototype, 'tabIndex')) return 'tabIndex belongs to the including prototypes';

            const functions = new Set();
            for (const [index, prototype] of prototypes.entries()) {
                const names = Object.getOwnPropertyNames(prototype);
                const d = Object.getOwnPropertyDescriptor(prototype, 'tabIndex');
                if (typeof d?.get !== 'function' || typeof d.set !== 'function' ||
                    d.get.name !== 'get tabIndex' || d.get.length !== 0 ||
                    d.set.name !== 'set tabIndex' || d.set.length !== 1 ||
                    !d.enumerable || !d.configurable || 'value' in d ||
                    names.indexOf('nonce') < 0 || names.indexOf('nonce') >= names.indexOf('autofocus') ||
                    names.indexOf('autofocus') >= names.indexOf('tabIndex')) {
                    return 'tabIndex descriptor and declaration order for ' + prototype.constructor.name;
                }
                const node = nodes[index];
                if (node.tabIndex !== 0 || d.get.call(node) !== 0 ||
                    d.set.call(node, 4294967295) !== undefined || node.tabIndex !== -1 ||
                    node.getAttributeNS(null, 'tabindex') !== '-1') {
                    return 'native inherited default and signed setter on each includer';
                }
                Object.setPrototypeOf(foreign, prototype);
                for (const accessor of [d.get, d.set]) {
                    if (functions.has(accessor) || Object.hasOwn(accessor, 'prototype')) {
                        return 'each includer retains its own callable accessor';
                    }
                    functions.add(accessor);
                    for (const wrong of [null, foreign, Object.create(prototype), new Proxy(node, {}),
                        ...nodes.filter((_, other) => other !== index)]) {
                        let converted = false;
                        const operand = { [Symbol.toPrimitive]() { converted = true; return 7; } };
                        try { Reflect.apply(accessor, wrong, [operand]); return 'tabIndex accepted a wrong native receiver'; }
                        catch (error) {
                            if (!(error instanceof TypeError) || converted) return 'tabIndex brand before coercion';
                        }
                    }
                }
            }

            const generic = html('div');
            if (generic.tabIndex !== -1 || html('a').tabIndex !== 0 || svg('rect').tabIndex !== -1 ||
                math('mi').tabIndex !== -1 || svg('A').tabIndex !== -1 ||
                document.createElementNS(htmlNS, 'A').tabIndex !== -1) return 'exact local-name and namespace defaults';
            generic.setAttributeNS(foreignNS, 'f:tabindex', '77');
            generic.setAttributeNS(null, 'TABINDEX', '88');
            if (generic.tabIndex !== -1) return 'tabIndex reads only lowercase null-namespace content';

            const button = nodes[0];
            for (const [raw, expected] of [
                [' \t+0007junk', 7], ['-9\ud800', -9], ['2147483647tail', 2147483647],
                ['-2147483648tail', -2147483648], ['2147483648', null], ['-2147483649', null],
                ['\u00a01', null], ['+', null],
            ]) {
                generic.setAttributeNS(null, 'tabindex', raw);
                button.setAttributeNS(null, 'tabindex', raw);
                if (generic.tabIndex !== (expected ?? -1) || button.tabIndex !== (expected ?? 0) ||
                    generic.getAttributeNS(null, 'tabindex') !== raw) return 'recovered long or role default for ' + raw;
            }
            for (const [input, expected] of [
                [2147483648, '-2147483648'], [-2147483649, '2147483647'],
                [4294967303, '7'], [-2.75, '-2'], [Infinity, '0'],
            ]) {
                generic.tabIndex = input;
                if (generic.getAttributeNS(null, 'tabindex') !== expected || generic.tabIndex !== Number(expected) ||
                    generic.getAttributeNS(foreignNS, 'tabindex') !== '77' ||
                    generic.getAttributeNS(null, 'TABINDEX') !== '88') return 'signed wrapping setter serializes exact decimal';
            }

            const details = html('details');
            const first = html('summary');
            const second = html('summary');
            const nested = html('div');
            const deep = nested.appendChild(html('summary'));
            details.appendChild(document.createTextNode('leading text'));
            details.appendChild(svg('summary'));
            details.appendChild(nested);
            details.appendChild(first);
            details.appendChild(second);
            second.setAttributeNS(null, 'tabindex', '2147483648');
            if (first.tabIndex !== 0 || second.tabIndex !== -1 || deep.tabIndex !== -1) {
                return 'first ordinary HTML summary child';
            }
            details.insertBefore(second, first);
            if (first.tabIndex !== -1 || second.tabIndex !== 0) return 'summary default follows live child order';
            const foreignDetails = svg('details');
            foreignDetails.appendChild(second);
            if (second.tabIndex !== -1 || first.tabIndex !== 0) return 'foreign details do not qualify an HTML summary';
            return 'ok';
        })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
            const seen = [];
            class TabIndexObserver extends HTMLElement {
                static observedAttributes = ['tabindex'];
                attributeChangedCallback(name, oldValue, value, namespace) {
                    seen.push([this, name, oldValue, value, namespace, this.tabIndex]);
                }
            }
            customElements.define('x-tab-index-observer', TabIndexObserver);
            const original = document.body.appendChild(document.createElement('x-tab-index-observer'));
            const destination = document.body.appendChild(document.createElement('div'));
            const other = document.createElement('div');
            const hints = [];
            let receiver = original;
            receiver.tabIndex = { [Symbol.toPrimitive](hint) {
                hints.push(hint);
                receiver = other;
                destination.appendChild(original);
                original.setAttributeNS(null, 'tabindex', '+5tail');
                return 4294967294;
            } };
            if (hints.join() !== 'number' || receiver !== other || original.parentNode !== destination ||
                original.tabIndex !== -2 || original.getAttributeNS(null, 'tabindex') !== '-2' ||
                other.hasAttributeNS(null, 'tabindex')) return 'setter uses the original native receiver after reentry';
            const thrown = {};
            try {
                original.tabIndex = { valueOf() { throw thrown; } };
                return 'tabIndex swallowed the original coercion exception';
            } catch (error) {
                if (error !== thrown || original.tabIndex !== -2) return 'abrupt conversion leaves the native attribute unchanged';
            }
            const expected = [
                ['tabindex', null, '+5tail', null, 5],
                ['tabindex', '+5tail', '-2', null, -2],
            ];
            return original instanceof TabIndexObserver && seen.every(call => call[0] === original) &&
                JSON.stringify(seen.map(call => call.slice(1))) === JSON.stringify(expected)
                ? 'ok' : 'tabIndex setter runs the actual synchronous Source reactions';
        })()"#,
    );
    let snapshot = host.snapshot().unwrap();
    let root = snapshot
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(root).unwrap().children()[1];
    let destination = *snapshot.node(body).unwrap().children().last().unwrap();
    let target = snapshot.node(destination).unwrap().children()[0];
    let target = snapshot.node(target).unwrap();
    assert_eq!(target.parent(), Some(destination));
    assert_eq!(
        target.attribute_ns(&Namespace::none(), &"tabindex".into()),
        Some(&"-2".into())
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
