//! Exact native HTML, Base, Mod and Map content-attribute reflections.

use super::{window_length::observe, *};

const GUEST_VALUE: [u16; 5] = [0x20, 0xd800, 0, 0xdc00, 0x20];
const NATIVE_VALUE: [u16; 5] = [0x4e, 0xdc00, 0, 0xd800, 0x21];

pub(super) fn attached_reflections_preserve_live_attributes_and_creation_realms() {
    let mut dom = native_dom();
    let base_limits = dom.limits().with_base_processing(128, 1_000_000).unwrap();
    dom.set_limits(base_limits).unwrap();
    let mut session = DomSession::new(dom, limits()).unwrap();
    let (_, top) = create(&mut session);
    let document = top.document();
    let snapshot = session.snapshot();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let [head, body]: [NodeId; 2] = snapshot.node(html).unwrap().children().try_into().unwrap();

    let mut batch = DomBatch::new();
    let base = batch.create_element_ns(document, Namespace::html(), "base");
    batch.insert_before(&base, head, None);
    let insertion = batch.create_element_ns(document, Namespace::html(), "ins");
    let deletion = batch.create_element_ns(document, Namespace::html(), "del");
    let map = batch.create_element_ns(document, Namespace::html(), "map");
    for node in [&insertion, &deletion, &map] {
        batch.insert_before(node, body, None);
    }
    let detached = batch.create_html_document(document, None);
    let frame = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    let initial = session.dom().apply(batch, &cancel()).unwrap();
    let [base, insertion, deletion, map, frame] = [base, insertion, deletion, map, frame]
        .map(|node| initial.created(&node).unwrap().unwrap());
    let attributes = [
        (html, "version"),
        (base, "target"),
        (insertion, "datetime"),
        (deletion, "datetime"),
        (map, "name"),
    ];

    let (mut host, _) = attach_event_window_with_limits(
        session,
        top,
        Arc::new(cancel()),
        (8_000_000, 6_000_000, 300_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 1_000_000, 200_000, 64, 64, 64, 4),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), document);
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            if (typeof Object.getOwnPropertyDescriptor(HTMLHtmlElement.prototype, 'version')?.get !== 'function')
                return 'missing HTMLHtmlElement.prototype.version getter';
            const subjects = [
                [HTMLHtmlElement, 'html', 'version', 'version', ['version', 'constructor']],
                [HTMLBaseElement, 'base', 'target', 'target', ['href', 'target', 'constructor']],
                [HTMLModElement, 'ins', 'dateTime', 'datetime', ['cite', 'dateTime', 'constructor']],
                [HTMLModElement, 'del', 'dateTime', 'datetime', ['cite', 'dateTime', 'constructor']],
                [HTMLMapElement, 'map', 'name', 'name', ['name', 'areas', 'constructor']]
            ].map(([type, local, member, attr, order]) =>
                ({type, local, member, attr, order, node: document.getElementsByTagName(local)[0]}));
            const html = 'http://www.w3.org/1999/xhtml';
            const foreign = 'urn:foreign-reflection';
            const usesTypeError = action => {
                try { action(); } catch (error) { return Object.getPrototypeOf(error) === TypeError.prototype; }
                return false;
            };
            for (const {type, local, member, attr, order, node} of subjects) {
                const prototype = type.prototype;
                const label = type.name + '.' + member + ' on ' + local;
                const descriptor = Object.getOwnPropertyDescriptor(prototype, member);
                if (typeof descriptor?.get !== 'function' || typeof descriptor.set !== 'function' ||
                    !descriptor.enumerable || !descriptor.configurable || 'value' in descriptor ||
                    'writable' in descriptor || descriptor.get.name !== 'get ' + member ||
                    descriptor.set.name !== 'set ' + member || descriptor.get.length !== 0 ||
                    descriptor.set.length !== 1 || !Object.hasOwn(prototype, 'constructor') ||
                    Object.getOwnPropertyNames(prototype).filter(key => order.includes(key)).join() !==
                        order.filter(key => Object.hasOwn(prototype, key)).join())
                    return 'authored descriptor and prototype order for ' + label;
                const prefixed = document.createElementNS(html, 'p:' + local);
                const wrongNamespace = document.createElementNS(foreign, local);
                const wrongCase = document.createElementNS(html, local.toUpperCase());
                if (node.localName !== local || Object.getPrototypeOf(node) !== prototype ||
                    Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
                    Object.getPrototypeOf(prefixed) !== prototype || prefixed.prefix !== 'p' ||
                    member in HTMLElement.prototype || member in wrongNamespace || member in wrongCase)
                    return 'exact native interface selection for ' + label;
                Object.setPrototypeOf(wrongNamespace, prototype);
                for (const accessor of [descriptor.get, descriptor.set]) {
                    if (Object.hasOwn(accessor, 'prototype') ||
                        !usesTypeError(() => Reflect.construct(function () {}, [], accessor)))
                        return 'nonconstructible accessor for ' + label;
                    for (const other of [null, {}, document.body, Object.create(prototype),
                        new Proxy(node, {}), wrongCase, wrongNamespace]) {
                        let coerced = false;
                        const value = {[Symbol.toPrimitive]() { coerced = true; return 'wrong receiver'; }};
                        if (!usesTypeError(() => Reflect.apply(accessor, other, [value])) || coerced)
                            return 'native brand before value conversion for ' + label;
                    }
                }
                if (descriptor.get.call(prefixed) !== '' ||
                    descriptor.set.call(prefixed, 'detached') !== undefined ||
                    prefixed.getAttributeNS(null, attr) !== 'detached')
                    return 'borrowed accessors work on a detached prefixed native ' + label;
                node.setAttributeNS(foreign, 'p:' + attr, 'foreign');
                node.setAttributeNS(null, attr.toUpperCase(), 'uppercase');
                if (node[member] !== '' || node.getAttributeNS(null, attr) !== null)
                    return 'only the exact null-namespace content attribute reflects for ' + label;
                node.setAttributeNS(null, attr, '\udc00\0\ud800');
                if (node[member] !== '\udc00\0\ud800') return 'attribute to property ' + label;
                node.removeAttributeNS(null, attr);
                if (node[member] !== '' || node.getAttributeNS(null, attr) !== null)
                    return 'missing content attribute for ' + label;
                for (const [value, expected] of [[null, 'null'], [undefined, 'undefined'], ['', ''],
                    [' \ud800\0\udc00 ', ' \ud800\0\udc00 ']]) {
                    node[member] = value;
                    if (node[member] !== expected || node.getAttributeNS(null, attr) !== expected)
                        return 'ordinary DOMString and same native attribute for ' + label;
                }
                const sentinel = {};
                try { descriptor.set.call(node, {toString() { throw sentinel; }}); return 'swallowed throw for ' + label; }
                catch (error) { if (error !== sentinel) return 'changed thrown value for ' + label; }
                if (node[member] !== ' \ud800\0\udc00 ' ||
                    node.getAttributeNS(null, attr.toUpperCase()) !== 'uppercase' ||
                    node.getAttributeNS(foreign, attr) !== 'foreign')
                    return 'abrupt setter or namespace mutation for ' + label;
            }
            globalThis.reflectionSubjects = subjects;
            return 'ok';
        })()"#,
    );
    assert_eq!(result, "ok");

    let snapshot = host.snapshot().unwrap();
    for (node, attribute) in attributes {
        assert_eq!(
            snapshot
                .node(node)
                .unwrap()
                .attribute_ns(&Namespace::none(), &attribute.into())
                .unwrap()
                .code_units(),
            GUEST_VALUE
        );
    }
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        for (node, attribute) in attributes {
            batch.set_attribute_ns(
                node,
                Namespace::none(),
                attribute,
                DocumentString::from_utf16(NATIVE_VALUE),
            );
        }
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const native = 'N\udc00\0\ud800!';
            for (const {local, node, member} of reflectionSubjects) {
                if (document.getElementsByTagName(local)[0] !== node || node[member] !== native)
                    return 'existing wrapper missed native readback for ' + local;
            }
            const calls = [];
            class ObservedMap extends HTMLMapElement {
                static observedAttributes = ['name'];
                attributeChangedCallback(attr, old, value, namespace) {
                    calls.push([this, attr, old, value, namespace, this.name]);
                }
            }
            customElements.define('x-observed-map', ObservedMap, {extends: 'map'});
            const original = reflectionSubjects[4].node;
            const replacement = document.createElement('map', {is: 'x-observed-map'});
            if (!(replacement instanceof ObservedMap)) return 'actual customized native Map fixture';
            replacement.name = 'replacement';
            replacement.name = 'replacement';
            const raw = ' \ud800\0\udc00 ';
            const hints = [];
            original.name = { [Symbol.toPrimitive](hint) {
                hints.push(hint);
                if (original.name !== native || original.parentNode !== document.body)
                    throw Error('write ran before conversion');
                original.parentNode.replaceChild(replacement, original);
                return raw;
            }};
            if (hints.join() !== 'string' || original.parentNode !== null || original.name !== raw ||
                document.getElementsByTagName('map')[0] !== replacement || replacement.name !== 'replacement')
                return 'conversion reentry must keep the originally branded Map';
            const expected = [
                [replacement, 'name', null, 'replacement', null, 'replacement'],
                [replacement, 'name', 'replacement', 'replacement', null, 'replacement']
            ];
            return calls.length === expected.length && calls.every((row, i) =>
                row.every((value, j) => value === expected[i][j]))
                ? 'ok' : 'real Map Source reactions must run after both writes';
        })()"#,
    );
    assert_eq!(result, "ok");
    let snapshot = host.snapshot().unwrap();
    let original = snapshot.node(map).unwrap();
    assert_eq!(original.parent(), None);
    assert_eq!(
        original
            .attribute_ns(&Namespace::none(), &"name".into())
            .unwrap()
            .code_units(),
        GUEST_VALUE
    );

    window_length::edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const child = window[0];
            const foreign = child.document.createElement('map');
            const prototype = child.HTMLMapElement.prototype;
            const local = Object.getOwnPropertyDescriptor(HTMLMapElement.prototype, 'name');
            const childAccess = Object.getOwnPropertyDescriptor(prototype, 'name');
            if (Object.getPrototypeOf(foreign) !== prototype || childAccess.get === local.get ||
                childAccess.set === local.set || Object.getPrototypeOf(childAccess.set) !== child.Function.prototype)
                return 'actual distinct creation realm and accessors';
            const other = reflectionSubjects[4].node;
            local.set.call(foreign, 'written across realms');
            if (childAccess.get.call(foreign) !== 'written across realms' || childAccess.get.call(other) !== other.name)
                return 'borrowed accessors must use the actual native receiver';
            if (document.adoptNode(foreign) !== foreign || foreign.ownerDocument !== document ||
                Object.getPrototypeOf(foreign) !== prototype)
                return 'adoption must keep the original interface prototype';
            globalThis.foreignReflection = {foreign, prototype, local, childAccess, errorPrototype: child.TypeError.prototype};
            return 'ok';
        })()"#,
    );
    assert_eq!(result, "ok");
    let snapshot = host.snapshot().unwrap();
    let child = window_length::content_navigable(&snapshot, frame).unwrap();
    let child_document = snapshot
        .hosted_navigable(child)
        .unwrap()
        .active_document()
        .unwrap();
    window_length::edit(&mut host, DomSourceEdit::detach(frame));
    host.with_native(|dom, mut hosted, _| {
        for _ in 0..128 {
            if dom
                .snapshot()
                .document(child_document)
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
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const {foreign, prototype, local, childAccess, errorPrototype} = foreignReflection;
            childAccess.set.call(foreign, 'retired\ud800\0');
            if (Object.getPrototypeOf(foreign) !== prototype || foreign.ownerDocument !== document ||
                local.get.call(foreign) !== 'retired\ud800\0' ||
                foreign.getAttributeNS(null, 'name') !== 'retired\ud800\0')
                return 'retired origin changed the reflected receiver or its prototype';
            let coerced = false;
            try {
                childAccess.set.call({}, {toString() { coerced = true; return 'wrong receiver'; }});
                return 'retained child setter accepted a forged receiver';
            } catch (error) {
                return !coerced && Object.getPrototypeOf(error) === errorPrototype
                    ? 'ok' : 'retained child setter must use its original TypeError intrinsic';
            }
        })()"#,
    );
    assert_eq!(result, "ok");
    host.stop().into_session().unwrap();
}

pub(super) fn attached_reflection_setter_preserves_source_refusal() {
    let (session, top) = setup();
    let (mut host, _) = attach_event_window_with_source_limits(
        session,
        top,
        Arc::new(cancel()),
        160_000,
        DomSourceLimits::new(16_000_000, 200_000, 0, 64, 64, 64, 2),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    let before = host.snapshot().unwrap();
    let result = host.in_document(&permit, |scope| {
        let script = "Object.getOwnPropertyDescriptor(HTMLHtmlElement.prototype, 'version').set.call(document.documentElement, 'value')";
        scope.evaluate(&script.encode_utf16().collect::<Vec<_>>()).map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("version setter must preserve the original Source work refusal: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::SourceWorkLimit { limit: 0 }
        )))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}
