//! Live Map.areas and Datalist.options over canonical descendants.

use super::{window_length::observe, *};

pub(super) fn map_areas_and_datalist_options_are_distinct_live_associations() {
    let (mut session, top) = setup();
    let document = top.document();
    let snapshot = session.snapshot();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let map = batch.create_element_ns(document, Namespace::html(), "map");
    let datalist = batch.create_element_ns(document, Namespace::html(), "datalist");
    for (root, id) in [(&map, "map-root"), (&datalist, "datalist-root")] {
        batch.set_attribute_ns(root, Namespace::none(), "id", id);
        batch.insert_before(root, body, None);
    }
    let created = session.dom().apply(batch, &cancel()).unwrap();
    let roots = [map, datalist].map(|root| created.created(&root).unwrap().unwrap());
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 300_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
            const cases = [
                [HTMLMapElement, 'map-root', 'map', 'areas', 'area'],
                [HTMLDataListElement, 'datalist-root', 'datalist', 'options', 'option']
            ];
            const missing = cases.filter(([type, , , member]) =>
                typeof Object.getOwnPropertyDescriptor(type.prototype, member)?.get !== 'function');
            if (missing.length) return 'missing ' + missing.map(([type, , , member]) =>
                type.name + '.' + member).join(',');
            const html = 'http://www.w3.org/1999/xhtml';
            const svg = 'http://www.w3.org/2000/svg';
            const saved = [];
            for (const [type, id, local, member, tag] of cases) {
                const root = document.getElementById(id);
                const descriptor = Object.getOwnPropertyDescriptor(type.prototype, member);
                if (descriptor.set !== undefined || !descriptor.enumerable || !descriptor.configurable ||
                    'value' in descriptor || 'writable' in descriptor || descriptor.get.length !== 0 ||
                    descriptor.get.name !== 'get ' + member || Object.hasOwn(descriptor.get, 'prototype') ||
                    Object.getOwnPropertyNames(type.prototype).at(-1) !== 'constructor')
                    return 'readonly descriptor and placement for ' + member;
                for (const other of [null, {}, document.body, Object.create(type.prototype),
                    new Proxy(root, {}), document.createElementNS(svg, local)]) {
                    try { descriptor.get.call(other); return 'accepted wrong receiver for ' + member; }
                    catch (error) { if (Object.getPrototypeOf(error) !== TypeError.prototype) throw error; }
                }
                const view = root[member];
                const arbitrary = root.getElementsByTagNameNS(html, tag);
                const second = document.createElement(local);
                if (Object.getPrototypeOf(view) !== HTMLCollection.prototype || root[member] !== view ||
                    view === arbitrary || view === root.children || view === second[member] ||
                    view.length !== 0 || view.item(0) !== null || view[0] !== undefined)
                    return 'distinct SameObject association for ' + member;
                const first = root.appendChild(document.createElement(tag));
                first.id = 'member-' + tag;
                // Matching descendants of a matching element are not pruned.
                const nested = first.appendChild(document.createElement(tag));
                const foreignWrapper = root.appendChild(document.createElementNS('urn:wrapper', 'box'));
                const prefixed = foreignWrapper.appendChild(document.createElementNS(html, 'p:' + tag));
                // Foreign names and uppercase HTML local names do not qualify.
                root.appendChild(document.createElementNS(svg, tag));
                root.appendChild(document.createElementNS(html, tag.toUpperCase()));
                const template = root.appendChild(document.createElement('template'));
                template.content.appendChild(document.createElement(tag));
                const shadowHost = root.appendChild(document.createElement('div'));
                shadowHost.attachShadow({mode: 'open'}).appendChild(document.createElement(tag));
                if (tag === 'option') {
                    first.disabled = true;
                    first.value = '';
                    nested.disabled = true;
                }
                const iterated = [...view];
                if (view !== root[member] || view.length !== 3 || view[0] !== first || view[1] !== nested ||
                    view[2] !== prefixed || view.namedItem(first.id) !== first || view[first.id] !== first ||
                    view.item(3) !== null || arbitrary.length !== 3 || arbitrary[1] !== nested ||
                    view[Symbol.iterator] !== Array.prototype.values || iterated.length !== 3 ||
                    iterated.some((node, index) => node !== [first, nested, prefixed][index]))
                    return 'exact ordinary HTML descendants for ' + member;
                saved.push({root, member, tag, view, arbitrary, first, nested, prefixed});
            }
            globalThis.savedMemberCollections = saved;
            return 'ok';
        })()"#
        ),
        "ok",
    );

    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        for (root, tag) in roots.into_iter().zip(["area", "option"]) {
            let item = batch.create_element_ns(document, Namespace::html(), tag);
            batch.set_attribute_ns(&item, Namespace::none(), "id", format!("native-{tag}"));
            batch.insert_before(&item, root, None);
        }
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r#"(() => {
        for (const {root, member, tag, view, arbitrary, first, nested, prefixed} of savedMemberCollections) {
            if (root[member] !== view || view.length !== 4 || arbitrary.length !== 4 ||
                view[0] !== first || view[1] !== nested || view[2] !== prefixed ||
                view[3] !== document.getElementById('native-' + tag))
                return 'retained association missed native insertion for ' + member;
            first.remove();
            if (root[member] !== view || view.length !== 2 || arbitrary.length !== 2 ||
                view[0] !== prefixed || view[1].id !== 'native-' + tag)
                return 'retained association missed subtree removal for ' + member;
        }
        return 'ok';
    })()"#
        ),
        "ok"
    );
    host.stop().into_session().unwrap();
}
