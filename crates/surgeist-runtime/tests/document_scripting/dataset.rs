use super::{window_length::observe, *};

pub(super) fn dataset_observes_and_mutates_canonical_attributes() {
    let (session, top) = setup();
    let (mut host, _) = attach_event_window_with_source_limits(
        session,
        top,
        Arc::new(cancel()),
        300_000,
        DomSourceLimits::new(16_000_000, 1_000_000, 200_000, 64, 64, 64, 4),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
          try {
            const detached = document.implementation.createHTMLDocument();
            const element = detached.createElement('div');
            const svg = detached.createElementNS('http://www.w3.org/2000/svg', 'svg');
            const math = detached.createElementNS('http://www.w3.org/1998/Math/MathML', 'math');
            const map = element.dataset;
            if (typeof DOMStringMap !== 'function' || !(map instanceof DOMStringMap))
                return 'missing live DOMStringMap';
            if (map !== element.dataset || map === svg.dataset || svg.dataset === math.dataset ||
                !(svg.dataset instanceof DOMStringMap) || !(math.dataset instanceof DOMStringMap) ||
                detached.createElementNS('urn:other', 'div').dataset !== undefined)
                return 'dataset exposed on the wrong native interface or with unstable identity';
            const accessor = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'dataset');
            const svgAccessor = Object.getOwnPropertyDescriptor(SVGElement.prototype, 'dataset');
            const typeError = action => {
                try { action(); } catch (error) { return error instanceof TypeError; }
                return false;
            };
            if (!accessor?.enumerable || !accessor.configurable || accessor.set !== undefined ||
                accessor.get.name !== 'get dataset' || accessor.get.length !== 0 ||
                accessor.get === svgAccessor.get || !typeError(() => accessor.get.call(svg)) ||
                !typeError(() => svgAccessor.get.call(element)) ||
                !typeError(() => new DOMStringMap()) ||
                Object.getOwnPropertyNames(DOMStringMap.prototype).join() !== 'constructor' ||
                Object.prototype.toString.call(map) !== '[object DOMStringMap]')
                return 'incorrect getter brands or anonymous operation descriptors';
            element.setAttributeNS(null, 'data-z-z', 'html');
            element.setAttributeNS(null, 'data-', 'empty suffix');
            element.setAttributeNS(null, 'data-Caps', 'ignore uppercase');
            element.setAttributeNS('urn:foreign', 'p:data-same', 'first');
            element.setAttributeNS(null, 'data-same', 'later');
            element.setAttributeNS(null, 'data-2', 'named numeric');
            element.setAttributeNS(null, 'data-to-string', 'override');
            const expectedKeys = ['zZ', '', 'same', '2', 'toString'];
            const descriptor = Object.getOwnPropertyDescriptor(map, 'same');
            if (JSON.stringify(Object.keys(map)) !== JSON.stringify(expectedKeys) ||
                map.zZ !== 'html' || map[''] !== 'empty suffix' || Object.hasOwn(map, 'Caps') ||
                map.same !== 'first' || map.toString !== 'override' || map[2] !== 'named numeric' ||
                !descriptor?.configurable || !descriptor.enumerable || !descriptor.writable)
                return 'wrong native pair eligibility, order, value or descriptor';
            map.same = 'updated null namespace';
            if (map.same !== 'first' || element.getAttributeNS(null, 'data-same') !== 'updated null namespace' ||
                element.getAttributeNS('urn:foreign', 'data-same') !== 'first')
                return 'named setter must target the exact null-namespace local name';
            if (!Reflect.deleteProperty(map, 'same') ||
                element.getAttributeNS(null, 'data-same') !== null || map.same !== 'first')
                return 'deletion did not use the qualified unprefixed name';
            element.removeAttributeNS('urn:foreign', 'data-same');
            if (Object.hasOwn(map, 'same') || !Reflect.deleteProperty(map, 'same'))
                return 'retained map missed native removal';
            map['a:b'] = 'colon';
            const colon = element.getAttributeNodeNS(null, 'data-a:b');
            if (!colon || colon.prefix !== null || colon.localName !== 'data-a:b' ||
                colon.value !== 'colon' || map['a:b'] !== 'colon')
                return 'colon in a valid null-namespace local name was split';
            delete map['a:b'];
            if (element.hasAttributeNS(null, 'data-a:b'))
                return 'deletion lost exact colon spelling';
            element.setAttributeNS(null, 'data-foo--bar', 'double dash');
            if (map['foo-Bar'] !== 'double dash') return 'run of dashes folded incorrectly';
            const originalPrototype = Object.getPrototypeOf(map);
            map.__proto__ = 'named own value';
            if (!Object.hasOwn(map, '__proto__') || map.__proto__ !== 'named own value' ||
                Object.getPrototypeOf(map) !== originalPrototype ||
                element.getAttributeNS(null, 'data-__proto__') !== 'named own value')
                return 'named setter was displaced by a prototype accessor';
            delete map.__proto__;
            map['raw\ud800É'] = '\0\udc00';
            if (map['raw\ud800É'] !== '\0\udc00' ||
                element.getAttributeNS(null, 'data-raw\ud800É') !== '\0\udc00')
                return 'non-ASCII code units were altered';
            const receiver = Object.create(map);
            if (!Reflect.set(map, 'zZ', 'receiver', receiver) || receiver.zZ !== 'receiver' ||
                map.zZ !== 'html' || element.getAttributeNS(null, 'data-z-z') !== 'html')
                return 'foreign receiver reached the native named setter';
            if (!Reflect.defineProperty(map, 'fixed', {value: 12}) ||
                map.fixed !== '12' || element.getAttributeNS(null, 'data-fixed') !== '12' ||
                Reflect.defineProperty(map, 'rejected', {get: () => 1}) ||
                element.hasAttributeNS(null, 'data-rejected') || Reflect.preventExtensions(map))
                return 'named definition or extensibility did not follow Web IDL';
            const symbol = Symbol('ordinary');
            map[symbol] = 7;
            if (map[symbol] !== 7 || Reflect.ownKeys(map).at(-1) !== symbol ||
                !Reflect.deleteProperty(map, symbol) || map[symbol] !== undefined)
                return 'ordinary Symbol operations escaped the named-only distinction';
            const sentinel = {};
            let conversions = 0;
            try {
                map['invalid-name'] = {toString() { conversions++; throw sentinel; }};
                return 'expected value conversion exception';
            } catch (error) {
                if (error !== sentinel || conversions !== 1)
                    return 'name validation ran ahead of conversion';
            }
            try {
                map['invalid-name'] = 'cannot publish';
                return 'expected syntax exception';
            } catch (error) {
                if (error.name !== 'SyntaxError' || element.hasAttributeNS(null, 'data-invalid-name'))
                    return 'wrong syntax failure for dash followed by ASCII lowercase';
            }
            try {
                map['invalid name'] = 'cannot publish';
                return 'expected local-name exception';
            } catch (error) {
                if (error.name !== 'InvalidCharacterError' ||
                    element.getAttributeNS(null, 'data-invalid name') !== null)
                    return 'wrong native failure for a disallowed attribute local name';
            }
            const effects = [];
            class WatchedElement extends HTMLDivElement {
                static observedAttributes = ['data-current'];
                attributeChangedCallback(name, before, after, namespace) {
                    effects.push([name, before, after, namespace, this.dataset.current ?? null]);
                    if (after === 'first') this.dataset.current = 'reentered';
                }
            }
            customElements.define('x-dataset-edit', WatchedElement, {extends: 'div'});
            const watched = document.createElement('div', {is: 'x-dataset-edit'});
            const watchedMap = watched.dataset;
            watchedMap.current = {toString() {
                watchedMap.beside = 'converted';
                return 'first';
            }};
            delete watchedMap.current;
            if (watched.dataset !== watchedMap || watchedMap.current !== undefined ||
                watchedMap.beside !== 'converted' ||
                JSON.stringify(effects) !== JSON.stringify([
                    ['data-current', null, 'first', null, 'first'],
                    ['data-current', 'first', 'reentered', null, 'reentered'],
                    ['data-current', 'reentered', null, null, null],
                ]))
                return 'dataset did not drain the canonical Source reactions';
            return 'ok';
          } catch (error) {
            return 'unexpected dataset exception: ' + String(error) + '\n' + (error.stack ?? '');
          }
        })()"#,
    );
    assert_eq!(result, "ok");
    host.stop().into_session().unwrap();
}

pub(super) fn dataset_keeps_the_element_creation_realm() {
    let (mut session, top) = setup();
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(top.document(), None);
    let frame = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    let frame = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&frame)
        .unwrap()
        .unwrap();
    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 220_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 1_000_000, 200_000, 64, 64, 64, 4),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    window_length::edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    let before = observe(
        &mut host,
        &permit,
        r"(() => {
            const child = window[0];
            const first = child.document.createElement('div');
            const late = child.document.createElement('div');
            const accessor = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'dataset').get;
            const childAccessor = Object.getOwnPropertyDescriptor(child.HTMLElement.prototype, 'dataset').get;
            const map = accessor.call(first);
            if (childAccessor.call(first) !== map || accessor === childAccessor ||
                Object.getPrototypeOf(map) !== child.DOMStringMap.prototype || map instanceof DOMStringMap)
                return 'borrowed getter did not use the native creation realm';
            map.inChild = 'original';
            if (document.adoptNode(first) !== first || document.adoptNode(late) !== late ||
                first.dataset !== map || first.ownerDocument !== document)
                return 'adoption lost the live dataset association';
            globalThis.datasetProbe = {
                first, late, map, accessor, prototype: child.DOMStringMap.prototype
            };
            return 'ok';
        })()",
    );
    assert_eq!(before, "ok");
    let snapshot = host.snapshot().unwrap();
    let child = window_length::content_navigable(&snapshot, frame).unwrap();
    let document = snapshot
        .hosted_navigable(child)
        .unwrap()
        .active_document()
        .unwrap();
    window_length::edit(&mut host, DomSourceEdit::detach(frame));
    host.with_native(|dom, mut hosted, _| {
        for _ in 0..128 {
            if dom
                .snapshot()
                .document(document)
                .unwrap()
                .navigable()
                .is_none()
            {
                return;
            }
            hosted.run_next(dom, top.navigable(), &cancel()).unwrap();
        }
        panic!("native child Document was not disassociated");
    })
    .unwrap();
    let after = observe(
        &mut host,
        &permit,
        r"(() => {
            const {first, late, map, accessor, prototype} = datasetProbe;
            const lateMap = accessor.call(late);
            first.setAttribute('data-in-child', 'after retirement');
            lateMap.afterRetirement = 'live';
            if (first.dataset !== map || map.inChild !== 'after retirement' ||
                late.dataset !== lateMap || lateMap === map || Object.getPrototypeOf(lateMap) !== prototype ||
                late.getAttribute('data-after-retirement') !== 'live')
                return 'realm retirement lost cached or first-exposed map behavior';
            return 'ok';
        })()",
    );
    assert_eq!(after, "ok");
    host.stop().into_session().unwrap();
}
