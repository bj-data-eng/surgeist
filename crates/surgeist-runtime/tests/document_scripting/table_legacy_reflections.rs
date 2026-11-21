//! Attached table-family accessors reflect canonical attributes and Source reactions.

use super::*;

pub(super) fn attached_table_legacy_reflections_use_native_attributes() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const families = [
            [HTMLTableElement, ['table'], [
                ['align', 'align'], ['border', 'border'], ['frame', 'frame'], ['rules', 'rules'],
                ['summary', 'summary'], ['width', 'width'], ['bgColor', 'bgcolor', true],
                ['cellPadding', 'cellpadding', true], ['cellSpacing', 'cellspacing', true],
            ]],
            [HTMLTableCaptionElement, ['caption'], [['align', 'align']]],
            [HTMLTableSectionElement, ['thead', 'tbody', 'tfoot'], [
                ['align', 'align'], ['ch', 'char'], ['chOff', 'charoff'], ['vAlign', 'valign'],
            ]],
            [HTMLTableRowElement, ['tr'], [
                ['align', 'align'], ['ch', 'char'], ['chOff', 'charoff'], ['vAlign', 'valign'],
                ['bgColor', 'bgcolor', true],
            ]],
        ];

        for (const [constructor, , fields] of families) {
            const prototype = constructor.prototype;
            for (const [name] of fields) {
                const d = Object.getOwnPropertyDescriptor(prototype, name);
                if (typeof d?.get !== 'function') return 'missing ' + constructor.name + '.prototype.' + name + ' getter';
                if (d.get.name !== 'get ' + name || d.get.length !== 0 || typeof d.set !== 'function' ||
                    d.set.name !== 'set ' + name || d.set.length !== 1 || !d.enumerable || !d.configurable) {
                    return constructor.name + '.' + name + ' accessor descriptor';
                }
            }
            const order = fields.map(([name]) => name);
            if (Object.getOwnPropertyNames(prototype).filter(key => order.includes(key)).join() !== order.join()) {
                return constructor.name + ' legacy declaration order';
            }
        }

        const html = 'http://www.w3.org/1999/xhtml';
        const create = local => document.createElementNS(html, 'h:' + local);
        const table = document.body.appendChild(create('table'));
        const caption = table.appendChild(create('caption'));
        const sections = ['thead', 'tbody', 'tfoot'].map(local => table.appendChild(create(local)));
        const [, body] = sections;
        const calls = [];
        const inner = ' Inside \ud800';
        const outer = ' Outside \udc00';
        const nested = 'Nested\0';
        class WatchedRow extends HTMLTableRowElement {
            static observedAttributes = ['charoff', 'valign'];
            attributeChangedCallback(name, oldValue, newValue, namespace) {
                calls.push([this, name, oldValue, newValue, namespace, this.chOff, this.vAlign]);
                if (name === 'charoff' && newValue === inner) this.vAlign = nested;
            }
        }
        customElements.define('x-legacy-reflection-row', WatchedRow, {extends: 'tr'});
        const row = body.appendChild(new WatchedRow());
        const otherRow = body.appendChild(create('tr'));
        if (Object.getPrototypeOf(row) !== WatchedRow.prototype || !(row instanceof HTMLTableRowElement)) {
            return 'actual customized built-in Row';
        }

        const targets = [[table], [caption], sections, [row]];
        let coercions = 0;
        const poison = { [Symbol.toPrimitive]() { coercions++; return 'invalid brand'; } };
        for (const [i, [constructor, [local]]] of families.entries()) {
            const prototype = constructor.prototype;
            const foreign = document.createElementNS('urn:foreign', local);
            Object.setPrototypeOf(foreign, prototype);
            const wrong = [targets[(i + 1) % targets.length][0], foreign, Object.create(prototype),
                document.createElementNS(html, local.toUpperCase())];
            const {get, set} = Object.getOwnPropertyDescriptor(prototype, 'align');
            for (const receiver of wrong) {
                for (const method of [get, set]) {
                    try {
                        method.call(receiver, poison);
                        return constructor.name + ' accepted a wrong receiver';
                    } catch (error) {
                        if (!(error instanceof TypeError) || coercions !== 0) {
                            return constructor.name + ' must qualify the exact native brand before conversion';
                        }
                    }
                }
            }
            for (const target of targets[i]) {
                if (!prototype.isPrototypeOf(target)) return constructor.name + ' native prototype';
                if (target !== row && Object.getPrototypeOf(target) !== prototype) {
                    return constructor.name + ' exact namespace and prefix interface';
                }
            }
        }

        body.setAttributeNS('urn:foreign', 'p:char', 'foreign');
        body.setAttributeNS(null, 'CHAR', 'uppercase');
        const raw = ' MiXeD 09 ';
        for (const [i, [constructor, , fields]] of families.entries()) {
            for (const target of targets[i]) {
                for (const [member, attr, legacyNull] of fields) {
                    if (target[member] !== '' || target.getAttributeNS(null, attr) !== null) {
                        return constructor.name + '.' + member + ' absent null-namespace attribute';
                    }
                    target[member] = raw;
                    if (target[member] !== raw || target.getAttributeNS(null, attr) !== raw) {
                        return constructor.name + '.' + member + ' exact reflected name and value';
                    }
                    if (legacyNull) {
                        target[member] = undefined;
                        if (target[member] !== 'undefined' || target.getAttributeNS(null, attr) !== 'undefined') {
                            return constructor.name + '.' + member + ' undefined DOMString';
                        }
                        target[member] = null;
                        if (target[member] !== '' || target.getAttributeNS(null, attr) !== '') {
                            return constructor.name + '.' + member + ' null is a present empty string';
                        }
                    }
                }
            }
        }
        caption.align = null;
        if (caption.align !== 'null' || caption.getAttributeNS(null, 'align') !== 'null') {
            return 'ordinary DOMString null conversion';
        }
        table.border = undefined;
        table.width = ' Raw \ud800\0\udc00 ';
        if (table.border !== 'undefined' || table.width !== ' Raw \ud800\0\udc00 ' ||
            body.getAttributeNS('urn:foreign', 'char') !== 'foreign' ||
            body.getAttributeNS(null, 'CHAR') !== 'uppercase') {
            return 'exact UTF-16, undefined and independent content attributes';
        }
        calls.length = 0;
        globalThis.tableLegacyProbe = {row, otherRow, calls, raw, inner, outer, nested};
        return 'ok';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {row, otherRow, calls, raw, inner, outer, nested} = tableLegacyProbe;
        const {set} = Object.getOwnPropertyDescriptor(HTMLTableRowElement.prototype, 'chOff');
        const thrown = {};
        try {
            set.call(row, {toString() { throw thrown; }});
            return 'abrupt Row setter returned';
        } catch (error) {
            if (error !== thrown || row.chOff !== raw || calls.length !== 0) return 'abrupt Row setter mutated';
        }

        let receiver = row;
        const hints = [];
        const result = set.call(receiver, { [Symbol.toPrimitive](hint) {
            hints.push(hint);
            receiver = otherRow;
            row.chOff = inner;
            return outer;
        } });
        const expected = [
            [row, 'charoff', raw, inner, null, inner, raw],
            [row, 'valign', raw, nested, null, inner, nested],
            [row, 'charoff', inner, outer, null, outer, nested],
        ];
        if (result !== undefined || hints.join() !== 'string' || receiver !== otherRow ||
            calls.length !== expected.length || calls.some((entry, i) =>
                expected[i].some((value, j) => value !== entry[j]))) {
            return 'synchronous nested Row reflections and original receiver';
        }
        return row.getAttributeNS(null, 'charoff') === outer &&
            row.getAttributeNS(null, 'valign') === nested &&
            otherRow.getAttributeNS(null, 'charoff') === null ? 'ok' : 'reentrant native attribute state';
    })()"#,
    );

    let snapshot = host.snapshot().unwrap();
    let html = snapshot
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let root = snapshot.node(html).unwrap().children()[1];
    let [table]: [NodeId; 1] = snapshot.node(root).unwrap().children().try_into().unwrap();
    let [caption, head, body, foot]: [NodeId; 4] =
        snapshot.node(table).unwrap().children().try_into().unwrap();
    let [row, other_row]: [NodeId; 2] = snapshot.node(body).unwrap().children().try_into().unwrap();

    let raw = DocumentString::from(" MiXeD 09 ");
    let null_namespace = Namespace::none();
    let attribute = |node, name: &str| {
        snapshot
            .node(node)
            .unwrap()
            .attribute_ns(&null_namespace, &name.into())
    };
    for name in ["align", "frame", "rules", "summary"] {
        assert_eq!(attribute(table, name), Some(&raw), "table {name}");
    }
    assert_eq!(attribute(table, "border"), Some(&"undefined".into()));
    assert_eq!(
        attribute(table, "width"),
        Some(&DocumentString::from_utf16(
            " Raw "
                .encode_utf16()
                .chain([0xd800, 0, 0xdc00, 0x20])
                .collect::<Vec<_>>(),
        ))
    );
    for name in ["bgcolor", "cellpadding", "cellspacing"] {
        assert_eq!(attribute(table, name), Some(&"".into()), "table {name}");
    }
    assert_eq!(attribute(caption, "align"), Some(&"null".into()));
    for section in [head, body, foot] {
        for name in ["align", "char", "charoff", "valign"] {
            assert_eq!(attribute(section, name), Some(&raw), "section {name}");
        }
    }
    assert_eq!(attribute(body, "CHAR"), Some(&"uppercase".into()));
    assert_eq!(
        snapshot
            .node(body)
            .unwrap()
            .attribute_ns(&Namespace::from_uri("urn:foreign"), &"char".into()),
        Some(&"foreign".into())
    );
    assert_eq!(attribute(row, "align"), Some(&raw));
    assert_eq!(attribute(row, "char"), Some(&raw));
    assert_eq!(
        attribute(row, "charoff"),
        Some(&DocumentString::from_utf16(
            " Outside "
                .encode_utf16()
                .chain([0xdc00])
                .collect::<Vec<_>>(),
        ))
    );
    assert_eq!(attribute(row, "valign"), Some(&"Nested\0".into()));
    assert_eq!(attribute(row, "bgcolor"), Some(&"".into()));
    assert_eq!(attribute(other_row, "charoff"), None);
    assert_eq!(snapshot.incomplete_source_operation(), None);
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
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
