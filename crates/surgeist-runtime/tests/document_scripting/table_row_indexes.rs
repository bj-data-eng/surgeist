//! Attached table row indexes track canonical ordinary parent and collection order.

use super::*;

pub(super) fn attached_table_row_indexes_follow_live_ordinary_order() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const prototype = HTMLTableRowElement.prototype;
        const order = ['rowIndex', 'sectionRowIndex', 'align'];
        const getters = [];
        for (const name of order.slice(0, 2)) {
            const d = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof d?.get !== 'function') return 'missing HTMLTableRowElement.prototype.' + name + ' getter';
            if (d.get.name !== 'get ' + name || d.get.length !== 0 || d.set !== undefined ||
                !d.enumerable || !d.configurable) return name + ' readonly Row descriptor';
            getters.push(d.get);
        }
        if (Object.getOwnPropertyNames(prototype).filter(name => order.includes(name)).join() !== order.join()) {
            return 'Row indexes precede the legacy attributes in their pinned order';
        }

        const append = (parent, local) => parent.appendChild(document.createElement(local));
        const html = 'http://www.w3.org/1999/xhtml';
        const table = append(document.body, 'table');
        const foot = append(table, 'tfoot');
        const f = append(foot, 'tr');
        const d0 = append(table, 'tr');
        table.appendChild(document.createTextNode('ignored'));
        const tbody = append(table, 'tbody');
        const foreign = tbody.appendChild(document.createElementNS('urn:foreign', 'tr'));
        const upper = tbody.appendChild(document.createElementNS(html, 'TR'));
        const b0 = append(tbody, 'tr');
        const wrapper = append(tbody, 'div');
        const nested = append(wrapper, 'tr');
        const nestedSectionRow = append(append(wrapper, 'tbody'), 'tr');
        const b1 = append(tbody, 'tr');
        const h0 = append(append(table, 'thead'), 'tr');
        const d1 = append(table, 'tr');
        const h1 = append(table, 'thead').appendChild(document.createElementNS(html, 'p:tr'));
        const foreignSection = table.appendChild(document.createElementNS('urn:foreign', 'tbody'));
        const foreignSectionRow = append(foreignSection, 'tr');
        const detached = document.createElement('tbody');
        const peer = append(detached, 'tr');

        if (Object.getPrototypeOf(h1) !== prototype) return 'prefixed native HTML Row receiver';
        Object.setPrototypeOf(foreign, prototype);
        const wrong = [foreign, upper, document.createElement('td'), Object.create(prototype)];
        for (const getter of getters) {
            for (const receiver of wrong) {
                try {
                    getter.call(receiver);
                    return getter.name + ' accepted a wrong native receiver';
                } catch (error) {
                    if (!(error instanceof TypeError)) return getter.name + ' wrong receiver requires TypeError';
                }
            }
        }

        const actual = [h0, h1, d0, b0, b1, d1, f].map(row => [row.rowIndex, row.sectionRowIndex]);
        const expected = [[0, 0], [1, 0], [2, 2], [3, 0], [4, 1], [5, 5], [6, 0]];
        if (actual.some((pair, i) => pair.some((index, j) => index !== expected[i][j]))) {
            return 'interleaved Table rows and direct Table section indexes';
        }
        if (nested.rowIndex !== -1 || nested.sectionRowIndex !== -1 ||
            nestedSectionRow.rowIndex !== -1 || nestedSectionRow.sectionRowIndex !== 0 ||
            foreignSectionRow.rowIndex !== -1 || foreignSectionRow.sectionRowIndex !== -1 ||
            peer.rowIndex !== -1 || peer.sectionRowIndex !== 0) {
            return 'Row indexes require exact ordinary parent and grandparent';
        }
        globalThis.rowIndexProbe = {table, tbody, d0, d1, b0, b1, f, detached, peer};
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
    let [table]: [NodeId; 1] = initial.node(body).unwrap().children().try_into().unwrap();
    let [foot, d0, text, tbody, head0, d1, head1, foreign_section]: [NodeId; 8] =
        initial.node(table).unwrap().children().try_into().unwrap();
    let [foreign, upper, b0, wrapper, b1]: [NodeId; 5] =
        initial.node(tbody).unwrap().children().try_into().unwrap();
    let [nested, nested_section]: [NodeId; 2] = initial
        .node(wrapper)
        .unwrap()
        .children()
        .try_into()
        .unwrap();
    let [h1]: [NodeId; 1] = initial.node(head1).unwrap().children().try_into().unwrap();

    let tbody_name = DocumentString::from("tbody");
    let detached = initial
        .nodes()
        .find(|&id| {
            let candidate = initial.node(id).unwrap();
            candidate.parent().is_none()
                && candidate
                    .as_element()
                    .is_some_and(|element| element.name().local_name() == &tbody_name)
        })
        .unwrap();
    let [peer]: [NodeId; 1] = initial
        .node(detached)
        .unwrap()
        .children()
        .try_into()
        .unwrap();

    assert!(matches!(
        initial.node(text).unwrap().kind(),
        DomNodeKind::Text(_)
    ));
    assert_eq!(initial.node(nested).unwrap().parent(), Some(wrapper));
    assert_eq!(
        initial.node(nested_section).unwrap().parent(),
        Some(wrapper)
    );
    assert_eq!(initial.node(foreign_section).unwrap().parent(), Some(table));
    assert_eq!(initial.node(h1).unwrap().parent(), Some(head1));
    let name = |id| initial.node(id).unwrap().as_element().unwrap().name();
    assert_eq!(name(h1).prefix(), Some(&"p".into()));
    assert_eq!(name(upper).local_name(), &DocumentString::from("TR"));
    assert_eq!(name(foreign).namespace_uri(), Some(&"urn:foreign".into()));
    assert_eq!(
        name(foreign_section).namespace_uri(),
        Some(&"urn:foreign".into())
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {table, tbody, d0, d1, b0, b1, f, detached, peer} = rowIndexProbe;
        if (table.insertBefore(d1, tbody) !== d1 || d1.parentNode !== table ||
            d1.rowIndex !== 3 || d1.sectionRowIndex !== 3 ||
            b0.rowIndex !== 4 || b0.sectionRowIndex !== 0 || d0.rowIndex !== 2) {
            return 'same direct Row observes current interleaved order';
        }
        if (detached.appendChild(b1) !== b1 || b1.parentNode !== detached || b1.isConnected ||
            b1.rowIndex !== -1 || b1.sectionRowIndex !== 1 ||
            peer.rowIndex !== -1 || peer.sectionRowIndex !== 0 || f.rowIndex !== 5) {
            return 'same Body Row moves into a detached section';
        }
        return 'ok';
    })()"#,
    );

    let moved = host.snapshot().unwrap();
    assert_eq!(
        moved.node(table).unwrap().children(),
        &[foot, d0, text, d1, tbody, head0, head1, foreign_section]
    );
    assert_eq!(
        moved.node(tbody).unwrap().children(),
        &[foreign, upper, b0, wrapper]
    );
    assert_eq!(moved.node(d1).unwrap().parent(), Some(table));
    assert_eq!(moved.node(detached).unwrap().parent(), None);
    assert_eq!(moved.node(detached).unwrap().children(), &[peer, b1]);
    assert_eq!(moved.node(b1).unwrap().parent(), Some(detached));
    assert_eq!(initial.node(b1).unwrap().parent(), Some(tbody));

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {b1, detached} = rowIndexProbe;
        return detached.removeChild(b1) === b1 && b1.parentNode === null &&
            b1.rowIndex === -1 && b1.sectionRowIndex === -1 ? 'ok' : 'detached Row has no index';
    })()"#,
    );

    assert_eq!(host.snapshot().unwrap().node(b1).unwrap().parent(), None);
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(b1);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let retired = host.snapshot().unwrap();
    assert!(matches!(retired.node(b1), Err(DomError::StaleNode)));
    assert_eq!(retired.node(detached).unwrap().children(), &[peer]);
    assert_eq!(retired.incomplete_source_operation(), None);

    let failure = native_failure(host.in_document(&permit, |scope| {
        let code = "rowIndexProbe.b1.rowIndex"
            .encode_utf16()
            .collect::<Vec<_>>();
        scope.evaluate(&code).map(drop)
    }));
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
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
