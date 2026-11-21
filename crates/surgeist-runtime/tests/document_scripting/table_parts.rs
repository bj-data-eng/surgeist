//! Attached Table parts expose the pinned writable IDL over canonical Source.

use super::*;

pub(super) fn attached_table_parts_use_native_topology_and_reactions() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const prototype = HTMLTableElement.prototype;
        const attributes = ['caption', 'tHead', 'tFoot'];
        const methods = ['createCaption', 'deleteCaption', 'createTHead', 'deleteTHead',
            'createTFoot', 'deleteTFoot'];
        const callbacks = [];
        for (const name of attributes) {
            const d = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof d?.get !== 'function') return 'missing HTMLTableElement.prototype.' + name + ' getter';
            if (d.get.name !== 'get ' + name || d.get.length !== 0 || typeof d.set !== 'function' ||
                d.set.name !== 'set ' + name || d.set.length !== 1 || !d.enumerable || !d.configurable) {
                return name + ' writable Table accessor descriptor';
            }
            callbacks.push(d.get, d.set);
        }
        for (const name of methods) {
            const d = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof d?.value !== 'function') return 'missing HTMLTableElement.prototype.' + name + ' method';
            if (d.value.name !== name || d.value.length !== 0 ||
                !d.writable || !d.enumerable || !d.configurable) return name + ' Table method descriptor';
            callbacks.push(d.value);
        }
        const order = [...attributes, 'align', 'border', 'frame', 'rules', 'summary', 'width',
            'bgColor', 'cellPadding', 'cellSpacing', ...methods, 'constructor'];
        const keys = Object.getOwnPropertyNames(prototype);
        if (keys.filter(key => order.includes(key)).join() !== order.join() ||
            keys.at(-1) !== 'constructor') return 'Table regular attributes, operations, constructor order';

        const html = 'http://www.w3.org/1999/xhtml';
        const foreign = document.createElementNS('urn:foreign', 'table');
        Object.setPrototypeOf(foreign, prototype);
        const table = document.createElementNS(html, 'h:table');
        if (Object.getPrototypeOf(table) !== prototype) return 'prefixed native Table brand';
        const expectError = (action, check) => {
            try { action(); } catch (error) { return check(error); }
            return false;
        };
        const type = error => error instanceof TypeError;
        const invalidReceivers = [document.body, document.createElementNS(html, 'TABLE'),
            foreign, Object.create(prototype), new Proxy(table, {})];
        for (const callback of callbacks) {
            for (const receiver of invalidReceivers) {
                if (!expectError(() => Reflect.apply(callback, receiver, [null]), type)) {
                    return callback.name + ' exact Table receiver';
                }
            }
        }

        const oldCaption = table.appendChild(document.createElement('caption'));
        const head = table.appendChild(document.createElement('thead'));
        const foot = table.appendChild(document.createElement('tfoot'));
        const body = table.appendChild(document.createElement('tbody'));
        let proxyReads = 0;
        const handler = { get() { proxyReads++; throw Error('unexpected property read'); } };
        for (const [member, local, old, kind, wrong] of [
            ['caption', 'caption', oldCaption, HTMLTableCaptionElement, head],
            ['tHead', 'thead', head, HTMLTableSectionElement, oldCaption],
            ['tFoot', 'tfoot', foot, HTMLTableSectionElement, oldCaption],
        ]) {
            const foreign = document.createElementNS('urn:foreign', local);
            Object.setPrototypeOf(foreign, kind.prototype);
            const upper = document.createElementNS(html, local.toUpperCase());
            for (const value of [false, Object.create(kind.prototype), foreign, upper, wrong,
                new Proxy(old, handler)]) {
                if (!expectError(() => { table[member] = value; }, type) || table[member] !== old) {
                    return member + ' wrong interface TypeError preserves the old part';
                }
            }
        }
        if (proxyReads !== 0) return 'interface conversion performed a JS property read';
        const hierarchy = error => error instanceof DOMException && error.name === 'HierarchyRequestError';
        if (!expectError(() => { table.tHead = foot; }, hierarchy) ||
            !expectError(() => { table.tFoot = body; }, hierarchy) ||
            table.tHead !== head || table.tFoot !== foot || head.parentNode !== table || foot.parentNode !== table) {
            return 'wrong native Section subtype hierarchy error precedes removal';
        }
        return 'ok';
    })()"#,
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const html = 'http://www.w3.org/1999/xhtml';
        const table = document.body.appendChild(document.createElement('table'));
        const add = local => table.appendChild(document.createElement(local));
        const lead = table.appendChild(document.createComment('leading'));
        const group = add('colgroup');
        const gap = table.appendChild(document.createComment('before first eligible element'));
        const foreignGroup = table.appendChild(document.createElementNS('urn:foreign', 'colgroup'));
        const foreignCaption = table.appendChild(document.createElementNS('urn:foreign', 'caption'));
        add('div').appendChild(document.createElement('caption'));
        const body = add('tbody');
        const trail = table.appendChild(document.createTextNode('trailing'));
        if (table.caption !== null || table.tHead !== null || table.tFoot !== null ||
            table.deleteCaption() !== undefined || table.deleteTHead() !== undefined ||
            table.deleteTFoot() !== undefined) return 'absence ignores other namespaces and descendants';
        const caption = table.createCaption(Symbol());
        const head = table.createTHead(Symbol());
        const foot = table.createTFoot(Symbol());
        if (Object.getPrototypeOf(caption) !== HTMLTableCaptionElement.prototype ||
            Object.getPrototypeOf(head) !== HTMLTableSectionElement.prototype ||
            Object.getPrototypeOf(foot) !== HTMLTableSectionElement.prototype ||
            table.caption !== caption || table.tHead !== head || table.tFoot !== foot ||
            table.createCaption() !== caption || table.createTHead() !== head || table.createTFoot() !== foot) {
            return 'native wrappers and existing create identity';
        }
        if (table.firstChild !== caption || caption.nextSibling !== lead || group.nextSibling !== gap ||
            gap.nextSibling !== head || head.nextSibling !== foreignGroup || trail.nextSibling !== foot ||
            table.lastChild !== foot) return 'Caption first node, Head first eligible element, Foot last node';
        const detached = document.createElementNS(html, 'h:table');
        const comment = detached.appendChild(document.createComment('no eligible element'));
        const detachedHead = detached.createTHead();
        if (detached.isConnected || detached.tHead !== detachedHead || detached.firstChild !== comment ||
            comment.nextSibling !== detachedHead || detachedHead.ownerDocument !== document) {
            return 'detached Head appends after the last non-element node';
        }
        globalThis.tablePartProbe = {table, caption, head, foot, body, lead};
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
    let [
        caption,
        lead,
        group,
        gap,
        head,
        foreign_group,
        foreign_caption,
        wrapper,
        tbody,
        trail,
        foot,
    ]: [NodeId; 11] = initial.node(table).unwrap().children().try_into().unwrap();
    assert!(matches!(
        initial.node(lead).unwrap().kind(),
        DomNodeKind::Comment(_)
    ));
    assert!(matches!(
        initial.node(gap).unwrap().kind(),
        DomNodeKind::Comment(_)
    ));
    assert!(matches!(
        initial.node(trail).unwrap().kind(),
        DomNodeKind::Text(_)
    ));
    for (id, local) in [(caption, "caption"), (head, "thead"), (foot, "tfoot")] {
        let node = initial.node(id).unwrap();
        let name = node.as_element().unwrap().name();
        assert_eq!(name.local_name(), &local.into());
        assert_eq!(name.namespace_uri(), Namespace::html().uri());
        assert_eq!(node.owner_document(), Some(top.document()));
    }

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const {table, caption, head, foot, body, lead} = tablePartProbe;
        table.appendChild(caption);
        const second = table.appendChild(document.createElement('caption'));
        if (table.caption !== caption || table.createCaption() !== caption) return 'first matching duplicate';
        table.caption = caption;
        if (table.firstChild !== caption || caption.nextSibling !== lead) return 'same node moves to first position';
        table.caption = null;
        if (caption.parentNode !== null || table.caption !== second) return 'null removes only first part';
        table.caption = undefined;
        if (second.parentNode !== null || table.caption !== null) return 'undefined converts to nullable null';
        table.caption = caption;
        const kept = table.appendChild(document.createElement('caption'));
        if (table.deleteCaption() !== undefined || caption.parentNode !== null || table.caption !== kept) {
            return 'delete Caption removes only the first match';
        }
        table.caption = caption;

        const secondHead = table.appendChild(document.createElement('thead'));
        const secondFoot = table.appendChild(document.createElement('tfoot'));
        if (table.deleteTHead() !== undefined || head.parentNode !== null || table.tHead !== secondHead ||
            table.deleteTFoot() !== undefined || foot.parentNode !== null || table.tFoot !== secondFoot) {
            return 'delete section removes first and preserves next';
        }
        table.tFoot = foot;
        if (secondFoot.parentNode !== null || table.lastChild !== foot) return 'Foot setter appends';

        const other = document.implementation.createHTMLDocument();
        const remote = other.body.appendChild(other.createElement('table'));
        remote.id = 'table-part-remote';
        const remoteHead = remote.createTHead();
        const remoteRow = remoteHead.appendChild(other.createElement('tr'));
        if (remoteHead.ownerDocument !== other || remoteRow.ownerDocument !== other) {
            return 'creation uses the Table node document';
        }
        table.tHead = remoteHead;
        if (remote.firstChild !== null || remote.tHead !== null || table.tHead !== remoteHead ||
            remoteHead.ownerDocument !== document || remoteRow.ownerDocument !== document ||
            remoteHead.firstChild !== remoteRow || secondHead.parentNode !== null || body.parentNode !== table) {
            return 'setter adopts the original section subtree';
        }
        return 'ok';
    })()"#,
    );

    let adopted = host.snapshot().unwrap();
    let remote = adopted
        .nodes()
        .find(|&id| {
            adopted.node(id).unwrap().attribute(&"id".into()) == Some(&"table-part-remote".into())
        })
        .unwrap();
    let new_head = adopted.node(table).unwrap().children()[4];
    let [row]: [NodeId; 1] = adopted
        .node(new_head)
        .unwrap()
        .children()
        .try_into()
        .unwrap();
    assert_ne!(new_head, head);
    assert_eq!(adopted.node(head).unwrap().parent(), None);
    assert!(adopted.node(remote).unwrap().children().is_empty());
    assert_ne!(
        adopted.node(remote).unwrap().owner_document(),
        Some(top.document())
    );
    for id in [new_head, row] {
        assert_eq!(
            adopted.node(id).unwrap().owner_document(),
            Some(top.document())
        );
    }
    assert_eq!(
        adopted.node(table).unwrap().children(),
        [
            caption,
            lead,
            group,
            gap,
            new_head,
            foreign_group,
            foreign_caption,
            wrapper,
            tbody,
            trail,
            foot
        ]
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const mk = local => document.createElement(local);
        const table = document.body.appendChild(mk('table'));
        const old = table.appendChild(mk('caption'));
        const next = mk('caption');
        const ancestor = document.body.appendChild(mk('caption'));
        const doomed = ancestor.appendChild(mk('table'));
        const doomedOld = doomed.appendChild(mk('caption'));
        const calls = [];
        let current;
        customElements.define('x-table-part-reaction', class extends HTMLElement {
            connectedCallback() { this.record('connect'); }
            disconnectedCallback() { this.record('disconnect'); }
            record(kind) {
                if (!current) return;
                calls.push([kind, this, this.parentNode, this.isConnected, current.caption, current.firstChild]);
                if (kind === 'disconnect' && current.caption) {
                    current.caption.setAttribute('data-disconnect-saw', 'replacement');
                }
            }
        });
        const previous = old.appendChild(mk('x-table-part-reaction'));
        const incoming = next.appendChild(mk('x-table-part-reaction'));
        const failed = doomedOld.appendChild(mk('x-table-part-reaction'));
        const exact = expected => calls.length === expected.length && calls.every((entry, i) =>
            expected[i].every((value, j) => value === entry[j]));
        current = table;
        table.caption = next;
        if (!exact([['disconnect', previous, old, false, next, next],
            ['connect', incoming, next, true, next, next]]) ||
            next.getAttribute('data-disconnect-saw') !== 'replacement') {
            return 'ordered synchronous reactions see the final Table and can reenter its Source';
        }

        calls.length = 0;
        current = doomed;
        try {
            doomed.caption = ancestor;
            return 'ancestor Caption insertion must throw after removal';
        } catch (error) {
            if (!(error instanceof DOMException) || error.name !== 'HierarchyRequestError') {
                return 'post-removal realm hierarchy exception';
            }
        }
        if (doomed.parentNode !== ancestor || doomed.caption !== null || doomedOld.parentNode !== null ||
            !exact([['disconnect', failed, doomedOld, false, null, null]])) {
            return 'failed insertion preserves removal and its one owed reaction';
        }
        return 'ok';
    })()"#,
    );

    let settled = host.snapshot().unwrap();
    let [same_table, reacting_table, ancestor]: [NodeId; 3] =
        settled.node(body).unwrap().children().try_into().unwrap();
    assert_eq!(same_table, table);
    let [replacement]: [NodeId; 1] = settled
        .node(reacting_table)
        .unwrap()
        .children()
        .try_into()
        .unwrap();
    assert_eq!(
        settled
            .node(replacement)
            .unwrap()
            .attribute(&"data-disconnect-saw".into()),
        Some(&"replacement".into())
    );
    let [doomed]: [NodeId; 1] = settled
        .node(ancestor)
        .unwrap()
        .children()
        .try_into()
        .unwrap();
    assert!(settled.node(doomed).unwrap().children().is_empty());
    assert_eq!(settled.incomplete_source_operation(), None);
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
