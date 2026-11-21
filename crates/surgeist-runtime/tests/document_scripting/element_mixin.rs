//! Shared element mixin accessors in an attached Window.

use super::*;

pub(super) fn attached_mixin_accessors_use_their_includers_and_native_state() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 200_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());
    observe(
        &mut host,
        &permit,
        r#"(() => {
            if (typeof Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'nonce')?.get !== 'function') {
                return 'missing HTMLElement.prototype.nonce getter';
            }
            const html = document.body.appendChild(document.createElement('div'));
            const svg = document.createElementNS('http://www.w3.org/2000/svg', 'p:rect');
            const math = document.createElementNS('http://www.w3.org/1998/Math/MathML', 'mi');
            if (Object.getPrototypeOf(html) !== HTMLDivElement.prototype ||
                Object.getPrototypeOf(svg) !== SVGRectElement.prototype ||
                !(svg instanceof SVGElement) || svg.prefix !== 'p' ||
                Object.getPrototypeOf(math) !== MathMLElement.prototype) return 'actual native includers';
            const types = [HTMLElement, SVGElement, MathMLElement];
            const nodes = [html, svg, math];
            const foreign = document.createElementNS('urn:foreign', 'div');
            for (const name of ['nonce', 'autofocus']) {
                if (Object.hasOwn(Element.prototype, name) ||
                    Object.hasOwn(HTMLDivElement.prototype, name) ||
                    Object.hasOwn(SVGRectElement.prototype, name)) return 'mixin descriptor on the including prototype';
                const functions = new Set();
                for (const [index, type] of types.entries()) {
                    const prototype = type.prototype;
                    const own = Object.getOwnPropertyNames(prototype);
                    const d = Object.getOwnPropertyDescriptor(prototype, name);
                    if (typeof d?.get !== 'function' || typeof d.set !== 'function' ||
                        !d.enumerable || !d.configurable || 'value' in d || 'writable' in d ||
                        d.get.name !== 'get ' + name || d.get.length !== 0 ||
                        d.set.name !== 'set ' + name || d.set.length !== 1 ||
                        own.indexOf('nonce') >= own.indexOf('autofocus')) return type.name + ' descriptor for ' + name;
                    const node = nodes[index];
                    const initial = name === 'nonce' ? '' : false;
                    if (d.get.call(node) !== initial || node[name] !== initial ||
                        d.set.call(node, initial) !== undefined || d.get.call(node) !== initial) {
                        return type.name + ' inherited or own native access for ' + name;
                    }
                    Object.setPrototypeOf(foreign, prototype);
                    const bad = [null, undefined, {}, foreign, Object.create(prototype), new Proxy(node, {}),
                        ...nodes.filter((_, other) => other !== index)];
                    for (const accessor of [d.get, d.set]) {
                        if (Object.hasOwn(accessor, 'prototype') || functions.has(accessor)) return 'separate callable descriptors';
                        functions.add(accessor);
                        try { Reflect.construct(function () {}, [], accessor); return 'constructible mixin accessor'; }
                        catch (error) { if (!(error instanceof TypeError)) return 'mixin constructor error type'; }
                        for (const other of bad) {
                            let converted = false;
                            const input = { [Symbol.toPrimitive]() { converted = true; return 'wrong receiver'; } };
                            try { Reflect.apply(accessor, other, [input]); return 'accepted wrong ' + type.name + ' receiver'; }
                            catch (error) {
                                if (!(error instanceof TypeError) || converted) return 'qualify mixin receiver before conversion';
                            }
                        }
                    }
                }
            }
            globalThis.mixinProbe = {html, nodes};
            return 'ok';
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
    assert_eq!(
        snapshot
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .name()
            .local_name(),
        &DocumentString::from("div")
    );
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {html, nodes} = mixinProbe;
            document.body.removeChild(html);
            const exact = '\ud800\0\udc00';
            for (const node of nodes) {
                if (node.parentNode !== null || node.nonce !== '' || node.autofocus) return 'detached native initial state';
                node.setAttributeNS('urn:foreign', 'f:nonce', 'foreign nonce');
                node.setAttributeNS(null, 'NONCE', 'uppercase nonce');
                node.setAttributeNS('urn:foreign', 'f:autofocus', 'foreign autofocus');
                node.setAttributeNS(null, 'AUTOFOCUS', 'uppercase autofocus');
                if (node.nonce !== '' || node.autofocus) return 'exact lowercase null-namespace inputs';
                node.setAttributeNS(null, 'nonce', 'public');
                if (node.nonce !== 'public') return 'native nonce content update';
                node.nonce = exact;
                if (node.nonce !== exact || node.getAttributeNS(null, 'nonce') !== 'public') return 'separate nonce slot write';
                const clone = node.cloneNode();
                if (clone === node || Object.getPrototypeOf(clone) !== Object.getPrototypeOf(node) ||
                    clone.nonce !== exact || clone.getAttributeNS(null, 'nonce') !== 'public') return 'clone copies actual slot and content';
                node.setAttributeNS(null, 'nonce', 'public');
                if (node.nonce !== 'public') return 'same content runs original nonce hook';
                node.removeAttributeNS(null, 'nonce');
                if (node.nonce !== '' || node.getAttributeNS(null, 'nonce') !== null ||
                    node.getAttributeNS(null, 'NONCE') !== 'uppercase nonce' ||
                    node.getAttributeNS('urn:foreign', 'nonce') !== 'foreign nonce') return 'nonce removal and separate Attr names';

                node.setAttributeNS(null, 'autofocus', 'false');
                if (!node.autofocus) return 'autofocus reflects presence';
                node.autofocus = true;
                if (node.getAttributeNS(null, 'autofocus') !== '') return 'true autofocus rewrites existing nonempty content';
                node.autofocus = 0;
                if (node.autofocus || node.hasAttributeNS(null, 'autofocus') ||
                    node.getAttributeNS(null, 'AUTOFOCUS') !== 'uppercase autofocus' ||
                    node.getAttributeNS('urn:foreign', 'autofocus') !== 'foreign autofocus') return 'false autofocus preserves separate names';
            }
            const nonce = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'nonce');
            html.nonce = null;
            if (html.nonce !== 'null' || html.hasAttributeNS(null, 'nonce')) return 'nonce ordinary null conversion';
            if (Reflect.apply(nonce.set, html, []) !== undefined || html.nonce !== 'undefined') return 'nonce omitted argument';

            const fragment = document.createDocumentFragment();
            const other = document.createElement('div');
            const hints = [];
            let receiver = html;
            receiver.nonce = { [Symbol.toPrimitive](hint) {
                hints.push(hint);
                receiver = other;
                fragment.appendChild(html);
                html.setAttributeNS(null, 'nonce', 'reentrant content');
                return exact;
            } };
            return hints.join() === 'string' && html.parentNode === fragment && receiver === other &&
                html.nonce === exact && other.nonce === '' && html.getAttributeNS(null, 'nonce') === 'reentrant content'
                ? 'ok' : 'nonce keeps the original native receiver after one conversion and reentry';
        })()"#,
    );
    let snapshot = host.snapshot().unwrap();
    let detached = snapshot.node(node).unwrap();
    assert_ne!(detached.parent(), Some(body));
    let exact = DocumentString::from_utf16([0xd800, 0, 0xdc00]);
    assert_eq!(detached.as_element().unwrap().nonce(), Some(&exact));
    assert_eq!(
        detached.attribute(&"nonce".into()),
        Some(&"reentrant content".into())
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_nonce(node, DocumentString::from_utf16([0xdc00, 0, 0xd800]));
        batch.set_attribute_ns(node, Namespace::none(), "autofocus", "native presence");
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    observe(
        &mut host,
        &permit,
        r#"mixinProbe.html.nonce === '\udc00\0\ud800' && mixinProbe.html.autofocus &&
            mixinProbe.html.getAttributeNS(null, 'nonce') === 'reentrant content' &&
            mixinProbe.html.getAttributeNS(null, 'autofocus') === 'native presence'
            ? 'ok' : 'original wrapper observes live native slot and attribute edits'"#,
    );
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const seen = [];
            class ObservedElement extends HTMLElement {
                static observedAttributes = ['nonce', 'autofocus'];
                attributeChangedCallback(name, previous, current, namespace) {
                    seen.push([this, name, previous, current, namespace, this.nonce, this.autofocus]);
                }
            }
            customElements.define('x-observed-mixin', ObservedElement);
            const watched = document.createElement('x-observed-mixin');
            if (!(watched instanceof ObservedElement)) return 'real upgraded HTMLElement fixture';
            watched.setAttributeNS(null, 'nonce', 'content');
            watched.nonce = 'slot';
            if (seen.length !== 1 || watched.getAttributeNS(null, 'nonce') !== 'content') {
                return 'nonce property assignment has no attribute callback';
            }
            watched.setAttributeNS(null, 'autofocus', 'authored');
            watched.autofocus = true;
            watched.autofocus = false;
            watched.autofocus = {get [Symbol.toPrimitive]() { throw Error('Boolean invoked object coercion'); }};
            const setter = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'autofocus').set;
            if (Reflect.apply(setter, watched, []) !== undefined || watched.autofocus) return 'omitted Boolean argument';
            const expected = [
                ['nonce', null, 'content', null, 'content', false],
                ['autofocus', null, 'authored', null, 'slot', true],
                ['autofocus', 'authored', '', null, 'slot', true],
                ['autofocus', '', null, null, 'slot', false],
                ['autofocus', null, '', null, 'slot', true],
                ['autofocus', '', null, null, 'slot', false],
            ];
            return seen.every(call => call[0] === watched) &&
                JSON.stringify(seen.map(call => call.slice(1))) === JSON.stringify(expected)
                ? 'ok' : 'synchronous autofocus reactions and live reentrant native reads';
        })()"#,
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
