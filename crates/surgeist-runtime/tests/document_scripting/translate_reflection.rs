//! HTMLElement.translate over the attached Window's current ordinary tree.

use super::*;

pub(super) fn attached_translate_reads_native_ordinary_ancestry_and_writes_keywords() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 100_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let document = top.document();
    assert_eq!(permit.document(), document);
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const prototype = HTMLElement.prototype;
            const d = Object.getOwnPropertyDescriptor(prototype, 'translate');
            if (typeof d?.get !== 'function') return 'missing HTMLElement.prototype.translate getter';
            if (typeof d.set !== 'function' || d.get.name !== 'get translate' || d.get.length !== 0 ||
                d.set.name !== 'set translate' || d.set.length !== 1 || !d.enumerable || !d.configurable ||
                'value' in d || 'writable' in d) return 'ordinary translate descriptor';
            const div = document.createElement('div');
            const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
            const foreign = document.createElementNS('urn:foreign', 'div');
            if (Object.getPrototypeOf(div) !== HTMLDivElement.prototype ||
                Object.hasOwn(HTMLDivElement.prototype, 'translate') || Object.hasOwn(Element.prototype, 'translate') ||
                'translate' in svg || 'translate' in foreign || div.translate !== true || d.get.call(div) !== true ||
                document.documentElement.translate !== true) {
                return 'HTML interface and Document or detached defaults';
            }
            const throwsTypeError = invoke => {
                try { invoke(); } catch (error) { return error instanceof TypeError; }
                return false;
            };
            for (const accessor of [d.get, d.set]) {
                if (Object.hasOwn(accessor, 'prototype') ||
                    !throwsTypeError(() => Reflect.construct(function () {}, [], accessor))) {
                    return 'translate accessors must be nonconstructible';
                }
                for (const receiver of [null, undefined, {}, Object.create(prototype), new Proxy(div, {}), svg, foreign]) {
                    if (!throwsTypeError(() => Reflect.apply(accessor, receiver, [true]))) {
                        return 'translate accessors must reject a non-HTMLElement receiver';
                    }
                }
            }
            return 'ok';
        })()"#,
    );
    let [closer, leaf, destination] = host
        .with_native(|dom, _, _| {
            let parent = dom
                .snapshot()
                .document(document)
                .unwrap()
                .document_element()
                .unwrap();
            let mut batch = DomBatch::new();
            let outer = batch.create_element_ns(document, Namespace::html(), "div");
            let svg = batch.create_element_ns(document, Namespace::svg(), "svg");
            let group = batch.create_element_ns(document, Namespace::svg(), "g");
            let math = batch.create_element_ns(document, Namespace::mathml(), "math");
            let row = batch.create_element_ns(document, Namespace::mathml(), "mrow");
            let foreign =
                batch.create_element_ns(document, Namespace::from_uri("urn:foreign"), "x:div");
            let closer = batch.create_element_ns(document, Namespace::html(), "section");
            let leaf = batch.create_element_ns(document, Namespace::html(), "span");
            let destination = batch.create_element_ns(document, Namespace::html(), "div");
            batch.set_attribute_ns(&outer, Namespace::none(), "translate", "nO");
            batch.set_attribute_ns(&destination, Namespace::none(), "translate", "no");
            for element in [&svg, &group, &math, &row, &foreign] {
                batch.set_attribute_ns(element, Namespace::none(), "translate", "yes");
            }
            batch.set_attribute_ns(
                &leaf,
                Namespace::from_uri("urn:foreign"),
                "x:translate",
                "yes",
            );
            batch.insert_before(&leaf, &closer, None);
            batch.insert_before(&closer, &foreign, None);
            batch.insert_before(&foreign, &row, None);
            batch.insert_before(&row, &math, None);
            batch.insert_before(&math, &group, None);
            batch.insert_before(&group, &svg, None);
            batch.insert_before(&svg, &outer, None);
            batch.insert_before(&destination, &outer, None);
            batch.insert_before(&outer, parent, None);
            let commit = dom.apply(batch, &cancel()).unwrap();
            [&closer, &leaf, &destination].map(|ticket| commit.created(ticket).unwrap().unwrap())
        })
        .unwrap();
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const outer = document.documentElement.lastChild;
            const svg = outer.firstChild;
            const group = svg.firstChild;
            const math = group.firstChild;
            const row = math.firstChild;
            const foreign = row.firstChild;
            const closer = foreign.firstChild;
            const leaf = closer.firstChild;
            const destination = svg.nextSibling;
            if (outer.localName !== 'div' || svg.localName !== 'svg' || group.localName !== 'g' ||
                math.localName !== 'math' || row.localName !== 'mrow' || foreign.namespaceURI !== 'urn:foreign' ||
                leaf.localName !== 'span' || leaf.getAttributeNS(null, 'translate') !== null ||
                leaf.getAttributeNS('urn:foreign', 'translate') !== 'yes' ||
                outer.translate !== false || closer.translate !== false || leaf.translate !== false) {
                return 'ordinary inheritance through exact SVG, MathML and other foreign Elements';
            }
            globalThis.translateProbe = {outer, closer, leaf, destination};
            for (const [raw, mode] of [
                ['YeS', true], ['', true], ['NO', false], ['yes ', false], ['Invalid', false],
                ['yes\0', false], ['\ud800\0\udc00', false],
            ]) {
                leaf.setAttributeNS(null, 'translate', raw);
                if (leaf.translate !== mode || leaf.getAttributeNS(null, 'translate') !== raw) {
                    return 'exact raw translation keyword or inherited default';
                }
            }
            leaf.removeAttributeNS(null, 'translate');
            if (leaf.translate !== false) return 'foreign attributes are ignored during inheritance';

            const shadowHost = outer.appendChild(document.createElement('div'));
            const shadow = shadowHost.attachShadow({mode: 'open'});
            const shadowChild = shadow.appendChild(document.createElement('span'));
            if (shadowHost.translate !== false || shadowChild.translate !== true) return 'shadow-root boundary';
            const fragment = document.createDocumentFragment();
            const moving = outer.appendChild(document.createElement('span'));
            if (moving.translate !== false) return 'ordinary initial parent';
            fragment.appendChild(moving);
            if (moving.parentNode !== fragment || moving.translate !== true) return 'ordinary fragment boundary';
            fragment.removeChild(moving);
            return moving.parentNode === null && moving.translate === true ? 'ok' : 'detached default';
        })()"#,
    );

    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(closer, Namespace::none(), "translate", "yEs");
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {outer, closer, leaf} = translateProbe;
            return leaf.parentNode === closer && !outer.translate && closer.translate && leaf.translate &&
                closer.getAttributeNS(null, 'translate') === 'yEs'
                ? 'ok' : 'existing wrappers read the nearest native HTML override';
        })()"#,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.insert_before(leaf, destination, None);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(
        host.snapshot().unwrap().node(leaf).unwrap().parent(),
        Some(destination)
    );
    observe(
        &mut host,
        &permit,
        r#"translateProbe.leaf.parentNode === translateProbe.destination && translateProbe.leaf.translate === false
            ? 'ok' : 'existing wrapper reopens ordinary ancestry after native reparenting'"#,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.detach(leaf);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(host.snapshot().unwrap().node(leaf).unwrap().parent(), None);
    observe(
        &mut host,
        &permit,
        "translateProbe.leaf.translate === true ? 'ok' : 'existing wrapper reopens native detach'",
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
            const leaf = translateProbe.leaf;
            const object = {
                get [Symbol.toPrimitive]() { throw new Error('Boolean must not coerce objects'); },
                get valueOf() { throw new Error('Boolean must not read valueOf'); },
                get toString() { throw new Error('Boolean must not read toString'); },
            };
            for (const input of [false, null, undefined, 0, -0, NaN, 0n, '']) {
                leaf.translate = input;
                if (leaf.translate !== false || leaf.getAttributeNS(null, 'translate') !== 'no') {
                    return 'false conversion must write the exact no keyword';
                }
            }
            for (const input of [true, 1, Infinity, 1n, 'false', 'no', Symbol('value'), [], new Boolean(false), object]) {
                leaf.translate = input;
                if (leaf.translate !== true || leaf.getAttributeNS(null, 'translate') !== 'yes') {
                    return 'true conversion must write the exact yes keyword';
                }
            }
            const d = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'translate');
            Reflect.apply(d.set, leaf, []);
            if (leaf.translate !== false || leaf.getAttributeNS(null, 'translate') !== 'no' ||
                leaf.getAttributeNS('urn:foreign', 'translate') !== 'yes') return 'missing input writes no in the null namespace';

            const calls = [];
            customElements.define('x-translate-observed', class extends HTMLElement {
                static observedAttributes = ['translate'];
                attributeChangedCallback(name, oldValue, value, namespace) {
                    calls.push([this, name, oldValue, value, namespace,
                        this.getAttributeNS(null, name), this.firstChild.translate]);
                }
            });
            const watched = translateProbe.outer.appendChild(document.createElement('x-translate-observed'));
            const child = watched.appendChild(document.createElement('span'));
            if (child.translate !== false) return 'inherited callback fixture setup';
            watched.translate = object;
            watched.translate = true;
            watched.translate = false;
            const expected = [
                ['translate', null, 'yes', null, 'yes', true],
                ['translate', 'yes', 'yes', null, 'yes', true],
                ['translate', 'yes', 'no', null, 'no', false],
            ];
            return calls.every(call => call[0] === watched) &&
                JSON.stringify(calls.map(call => call.slice(1))) === JSON.stringify(expected)
                ? 'ok' : 'every setter writes and delivers synchronous reactions with current descendant translation';
        })()"#,
    );
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(leaf)
            .unwrap()
            .attribute_ns(&Namespace::none(), &"translate".into()),
        Some(&"no".into()),
    );
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
