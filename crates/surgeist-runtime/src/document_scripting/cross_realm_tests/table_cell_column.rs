use super::{
    DocumentGuestEventError, DocumentScriptError, DomBatch, Error, Namespace, PropertyKey,
    evaluate, hosted_session,
};
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};
use std::{sync::Arc, sync::atomic::AtomicBool};

pub(super) fn table_ranged_attributes_read_from_and_write_to_the_native_elements() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const td = document.createElement('td');
                const th = document.createElement('th');
                const col = document.createElement('col');
                const group = document.createElement('colgroup');
                if (!(td instanceof HTMLTableCellElement) || !(th instanceof HTMLTableCellElement) ||
                    !(col instanceof HTMLTableColElement) || !(group instanceof HTMLTableColElement))
                    return 'native table fixture';
                const span = Object.getOwnPropertyDescriptor(HTMLTableCellElement.prototype, 'colSpan');
                if (typeof span?.get !== 'function' || typeof span?.set !== 'function')
                    return 'table cell missing colSpan accessor';
                if (td.colSpan !== 1 || th.rowSpan !== 1 || col.span !== 1 || group.span !== 1)
                    return 'range defaults';
                td.colSpan = 1001;
                th.rowSpan = 0;
                col.span = 0;
                group.span = -1;
                if (td.colSpan !== 1000 || td.getAttribute('colspan') !== '1001' ||
                    th.rowSpan !== 0 || th.getAttribute('rowspan') !== '0' ||
                    col.span !== 1 || col.getAttribute('span') !== '0' ||
                    group.span !== 1 || group.getAttribute('span') !== '1')
                    return 'getter clamp and setter serialization';
                const row = document.createElement('tr');
                if (td.cellIndex !== -1) return 'detached cell index';
                row.appendChild(td);
                row.appendChild(document.createElement('div'));
                row.appendChild(th);
                return td.cellIndex === 0 && th.cellIndex === 1 ? 'ok' : 'native row cell order';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

pub(super) fn table_reflection_descriptors_conversions_and_reactions() {
    assert_table_script(
        r#"
            (() => {
                const td = document.createElement('td');
                const th = document.createElement('th');
                const group = document.createElement('colgroup');
                const common = {align: 'align', ch: 'char', chOff: 'charoff', vAlign: 'valign', width: 'width'};
                for (const [ctor, members] of [
                    [HTMLTableCellElement, 'colSpan,rowSpan,headers,cellIndex,scope,abbr,align,axis,height,width,ch,chOff,noWrap,vAlign,bgColor'],
                    [HTMLTableColElement, 'span,align,ch,chOff,vAlign,width'],
                ]) {
                    const prototype = ctor.prototype;
                    if (Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
                        prototype.constructor !== ctor || Object.keys(prototype).join() !== members)
                        return 'table constructor or declaration order';
                    for (const member of members.split(',')) {
                        const {get, set, configurable, enumerable} = Object.getOwnPropertyDescriptor(prototype, member);
                        if (!configurable || !enumerable || get.length !== 0 || get.name !== 'get ' + member ||
                            (member === 'cellIndex' ? set !== undefined : set?.length !== 1 || set.name !== 'set ' + member))
                            return 'accessor descriptor ' + member;
                    }
                }
                const raw = ' raw\ud800\0\udc00 ';
                for (const [element, attrs] of [
                    [td, {...common, headers: 'headers', abbr: 'abbr', axis: 'axis', height: 'height', bgColor: 'bgcolor'}],
                    [group, common],
                ]) {
                    for (const [prop, attr] of Object.entries(attrs)) {
                        if (element[prop] !== '') return 'missing string ' + prop;
                        element[prop] = raw;
                        if (element[prop] !== raw || element.getAttributeNS(null, attr) !== raw)
                            return 'exact UTF16 or attribute alias ' + prop;
                    }
                }
                td.bgColor = null;
                if (td.bgColor !== '' || td.getAttributeNS(null, 'bgcolor') !== '') return 'legacy null conversion';
                td.headers = null;
                if (td.headers !== 'null') return 'ordinary null conversion';
                td.setAttributeNS('urn:foreign', 'x:nowrap', '');
                if (td.noWrap) return 'foreign attribute has no presence';
                td.setAttribute('nowrap', 'false');
                if (!td.noWrap) return 'Boolean presence';
                td.noWrap = false;
                if (td.noWrap || td.hasAttributeNS(null, 'nowrap')) return 'Boolean removal';
                td.noWrap = {};
                if (td.getAttributeNS(null, 'nowrap') !== '') return 'Boolean write';
                for (const cell of [td, th]) {
                    if (cell.scope !== '') return 'missing scope';
                    for (const [input, keyword] of [['RoW', 'row'], ['COL', 'col'], ['RowGroup', 'rowgroup'],
                        ['COLGROUP', 'colgroup'], ['Auto', ''], ['col ', ''], ['', '']]) {
                        cell.scope = input;
                        if (cell.scope !== keyword || cell.getAttributeNS(null, 'scope') !== input)
                            return 'limited scope keyword ' + input;
                    }
                }
                return 'ok';
            })()
        "#,
    );
    for (tag, prop, attr, upper) in [
        ("td", "colSpan", "colspan", 1000),
        ("th", "rowSpan", "rowspan", 65534),
        ("col", "span", "span", 1000),
        ("colgroup", "span", "span", 1000),
    ] {
        let script = r#"
            ((tag, prop, attr, upper) => {
                    const element = document.createElement(tag);
                    element.setAttribute(attr, '-1');
                    if (element[prop] !== 1) return 'invalid raw value';
                    element.setAttribute(attr, ' +002end');
                    if (element[prop] !== 2) return 'content integer parsing';
                    for (const [input, stored, expected] of [[upper, String(upper), upper],
                        [upper + 1, String(upper + 1), upper], [2147483647, '2147483647', upper],
                        [2147483648, '1', 1], [4294967299, '3', 3], [undefined, '0', prop === 'rowSpan' ? 0 : 1]]) {
                        element[prop] = input;
                        if (element.getAttributeNS(null, attr) !== stored || element[prop] !== expected)
                            return 'span setter and getter boundary ' + prop + ' ' + input;
                    }
                    element.removeAttribute(attr);
                    return element[prop] === 1 ? 'ok' : 'removed span default';
            })"#;
        assert_table_script(&format!("{script}('{tag}', '{prop}', '{attr}', {upper})"));
    }
    assert_table_script(
        r#"
            (() => {
                const td = document.createElement('td');
                const col = document.createElement('col');
                const {get, set} = Object.getOwnPropertyDescriptor(HTMLTableCellElement.prototype, 'colSpan');
                const html = 'http://www.w3.org/1999/xhtml';
                const xml = document.implementation.createDocument(null, '');
                for (const receiver of [col, {}, new Proxy(td, {}), xml.createElementNS(html, 'TD'),
                    xml.createElementNS('urn:foreign', 'td')]) {
                    let converted = false;
                    const input = {valueOf() { converted = true; return 2; }};
                    try { get.call(receiver); return 'invalid getter accepted'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'getter brand error'; }
                    try { set.call(receiver, input); return 'invalid setter accepted'; }
                    catch (error) { if (!(error instanceof TypeError) || converted) return 'brand before coercion'; }
                }
                td.colSpan = 3;
                const sentinel = {};
                try { td.colSpan = {valueOf() { throw sentinel; }}; return 'lost coercion exception'; }
                catch (error) { if (error !== sentinel || td.getAttribute('colspan') !== '3') return 'abrupt assignment'; }
                const calls = [];
                class Watched extends HTMLTableCellElement {
                    static observedAttributes = ['colspan'];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        calls.push([this, name, oldValue, newValue, namespace, this.colSpan]);
                    }
                }
                customElements.define('x-watched-table-cell', Watched, {extends: 'td'});
                const watched = new Watched();
                watched.colSpan = {valueOf() { watched.colSpan = 7; return 1001; }};
                const expected = [[watched, 'colspan', null, '7', null, 7], [watched, 'colspan', '7', '1001', null, 1000]];
                return watched instanceof HTMLTableCellElement && calls.length === 2 &&
                    calls.every((row, i) => row.every((value, j) => value === expected[i][j]))
                    ? 'ok' : 'reentrant conversion and attribute reactions';
            })()
        "#,
    );
}

pub(super) fn cell_index_tracks_current_ordinary_html_row_children() {
    assert_table_script(
        r#"
            (() => {
                const html = 'http://www.w3.org/1999/xhtml';
                const xml = document.implementation.createDocument(null, '');
                const row = xml.createElementNS(html, 'h:tr');
                const td = xml.createElementNS(html, 'h:td');
                const th = xml.createElementNS(html, 'th');
                const nested = xml.createElementNS(html, 'td');
                const div = xml.createElementNS(html, 'div');
                div.appendChild(nested);
                row.appendChild(xml.createElementNS('urn:foreign', 'td'));
                row.appendChild(xml.createElementNS(html, 'TD'));
                row.appendChild(div);
                row.appendChild(td);
                row.appendChild(th);
                if (td.cellIndex !== 0 || th.cellIndex !== 1 || nested.cellIndex !== -1)
                    return 'direct exact HTML children';
                if (row.insertBefore(th, td) !== th || th.cellIndex !== 0 || td.cellIndex !== 1)
                    return 'live reorder and identity';
                for (const parent of [xml.createElementNS(html, 'TR'), xml.createElementNS('urn:foreign', 'tr')]) {
                    parent.appendChild(th);
                    if (th.cellIndex !== -1 || td.cellIndex !== 0) return 'wrong parent expanded name';
                }
                row.removeChild(td);
                return td.cellIndex === -1 ? 'ok' : 'detached after reorder';
            })()
        "#,
    );
}

pub(super) fn borrowed_cell_accessors_preserve_the_creation_realm() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let mut batch = DomBatch::new();
    let cell = batch.create_element_ns(origin.document(), Namespace::html(), "td");
    let cell = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&cell)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(&mut host, &permit, cell, "foreignTableCell");
    let type_error = {
        let engine = host.engine.as_mut().unwrap();
        let realm = engine
            .creations
            .find(origin.global(), || Ok::<_, Error>(()))
            .unwrap()
            .unwrap();
        engine
            .process
            .execution()
            .in_realm(realm, |scope| {
                let value = evaluate(scope, "TypeError")?;
                Ok(scope.retain(&value))
            })
            .unwrap()
    };
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        let type_error = scope.import(&type_error)?;
        scope.set(&global, PropertyKey::String(&"foreignTableTypeError".encode_utf16().collect::<Vec<_>>()), &type_error)?;
        let result = evaluate(scope, r#"
            (() => {
                const local = document.createElement('th');
                const origin = Object.getPrototypeOf(foreignTableCell);
                if (origin === HTMLTableCellElement.prototype || origin.constructor.name !== 'HTMLTableCellElement')
                    return 'cell creation realm';
                const span = Object.getOwnPropertyDescriptor(origin, 'colSpan');
                const index = Object.getOwnPropertyDescriptor(origin, 'cellIndex');
                span.set.call(local, 12);
                if (span.get.call(local) !== 12 || index.get.call(local) !== -1 ||
                    Object.getOwnPropertyDescriptor(HTMLTableCellElement.prototype, 'colSpan').get.call(foreignTableCell) !== 1)
                    return 'borrowed accessor on authentic cell';
                for (const getter of [span.get, index.get]) {
                    try { getter.call({}); return 'foreign receiver accepted'; }
                    catch (error) {
                        if (!(error instanceof foreignTableTypeError) || error instanceof TypeError) return 'error realm';
                    }
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

#[track_caller]
fn assert_table_script(script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let observed = host.in_document(&permit, |scope| {
        let result = evaluate(scope, script)?;
        Ok(String::from_utf16_lossy(&scope.to_utf16(&result)?))
    });
    let message = match observed {
        Ok(message) => message,
        Err(DocumentScriptError::Script(Error::HostFailure(failure))) => {
            panic!(
                "table script host failure: {:?}",
                failure.cause::<DocumentGuestEventError>()
            );
        }
        Err(other) => panic!("table script failed: {other:?}"),
    };
    assert_eq!(message, "ok");
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}
