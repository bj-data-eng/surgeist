//! Existing Runtime's live HTML Table structural methods.

use super::*;

pub(super) fn attached_table_edits_use_the_original_source() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    let script = r#"(() => {
        const element = local => document.createElement(local);
        const check = (action, kind) => {
            try { action(); } catch (error) {
                return kind === 'TypeError' ? error instanceof TypeError :
                    error instanceof DOMException && error.name === kind;
            }
            return false;
        };
        for (const [prototype, methods] of [
            [HTMLTableElement.prototype, [['createTBody', 0], ['insertRow', 0], ['deleteRow', 1]]],
            [HTMLTableSectionElement.prototype, [['insertRow', 0], ['deleteRow', 1]]],
            [HTMLTableRowElement.prototype, [['insertCell', 0], ['deleteCell', 1]]],
        ]) {
            for (const [name, length] of methods) {
                const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
                if (typeof descriptor?.value !== 'function') return 'missing native method ' + name;
                if (descriptor.value.length !== length || descriptor.value.name !== name ||
                    !descriptor.writable || !descriptor.enumerable || !descriptor.configurable) {
                    return name + ' method descriptor';
                }
                let read = false;
                const argument = {valueOf() { read = true; return 0; }};
                if (!check(() => Reflect.apply(descriptor.value, Object.create(prototype), [argument]), 'TypeError')
                    || read) return name + ' receiver check before index conversion';
            }
        }

        const bare = element('table');
        const tail = bare.appendChild(document.createComment('kept'));
        if (bare.deleteRow(-1) !== undefined || bare.firstChild !== tail ||
            !check(() => bare.deleteRow(), 'TypeError') ||
            !check(() => bare.deleteRow(undefined), 'IndexSizeError') ||
            !check(() => bare.insertRow(-2), 'IndexSizeError') ||
            !check(() => bare.insertRow(1), 'IndexSizeError') ||
            bare.lastChild !== tail) return 'empty table index preconditions';
        const row = bare.insertRow();
        const implicit = row.parentNode;
        if (row.ownerDocument !== document || implicit.localName !== 'tbody' ||
            implicit.parentNode !== bare || bare.lastChild !== implicit ||
            tail.nextSibling !== implicit || implicit.firstChild !== row ||
            Object.getPrototypeOf(row) !== HTMLTableRowElement.prototype) return 'implicit body and row identity';
        const first = bare.insertRow(0);
        if (implicit.firstChild !== first || first.nextSibling !== row) return 'insert at explicit zero';
        if (bare.deleteRow(undefined) !== undefined || first.parentNode !== null ||
            implicit.firstChild !== row) return 'explicit undefined converts required signed index to zero';

        const unrelated = implicit.appendChild(element('div'));
        const nested = unrelated.appendChild(element('table')).insertRow();
        const comment = bare.appendChild(document.createComment('between body and foot'));
        const foot = bare.appendChild(element('tfoot'));
        const footRow = foot.insertRow();
        const direct = bare.appendChild(element('tr'));
        const body = bare.createTBody();
        if (body === implicit || body.localName !== 'tbody' || implicit.nextSibling !== body ||
            body.nextSibling !== comment || body.ownerDocument !== document) return 'fresh body after direct body';
        if (bare.createTBody() === body || body.nextSibling === comment) return 'another fresh body';
        const ending = bare.insertRow(undefined);
        if (ending.parentNode !== foot || footRow.nextSibling !== ending ||
            nested.parentNode.parentNode !== unrelated.firstChild ||
            direct.parentNode !== bare) return 'table append uses last member actual parent';

        const h = ending.appendChild(element('th'));
        const gap = ending.appendChild(document.createTextNode('gap'));
        const x = ending.appendChild(element('span'));
        const c = ending.insertCell(0);
        const appended = ending.insertCell();
        if (c.localName !== 'td' || appended.localName !== 'td' ||
            Object.getPrototypeOf(c) !== HTMLTableCellElement.prototype ||
            ending.firstChild !== c || c.nextSibling !== h || h.nextSibling !== gap ||
            x.nextSibling !== appended) return 'direct cell membership and actual append';
        const reentered = ending.insertCell({valueOf() {
            ending.deleteCell(0);
            return 0x100000001;
        }});
        if (c.parentNode !== null || ending.firstChild !== h ||
            gap.nextSibling !== x || x.nextSibling !== reentered ||
            reentered.nextSibling !== appended) return 'signed wrapping conversion before native index lookup';
        if (ending.deleteCell(-1) !== undefined || ending.lastChild !== reentered ||
            !check(() => ending.deleteCell(-2), 'IndexSizeError') ||
            !check(() => ending.deleteCell(2), 'IndexSizeError')) return 'cell deletion boundaries';
        return 'ok';
    })()"#;
    let result = host
        .in_document(&permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
    host.stop().into_session().unwrap();
}
