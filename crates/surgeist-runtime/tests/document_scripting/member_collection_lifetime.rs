//! Native origin, liveness and retirement of specialized collections.

use super::{window_length::observe, *};

pub(super) fn member_collections_keep_the_root_creation_realm_after_retirement() {
    let (mut session, top) = super::map_datalist_collections::collection_session();
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
        (8_000_000, 6_000_000, 300_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 1_000_000, 200_000, 64, 64, 64, 4),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    window_length::edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r"(() => {
            const child = window[0];
            const cases = [
                [HTMLMapElement, child.HTMLMapElement, 'map', 'areas', 'area'],
                [HTMLDataListElement, child.HTMLDataListElement, 'datalist', 'options', 'option']
            ];
            const saved = [];
            for (const [topType, childType, local, member, tag] of cases) {
                const first = child.document.createElement(local);
                const late = child.document.createElement(local);
                const accessor = Object.getOwnPropertyDescriptor(topType.prototype, member)?.get;
                const childAccessor = Object.getOwnPropertyDescriptor(childType.prototype, member)?.get;
                if (typeof accessor !== 'function' || typeof childAccessor !== 'function')
                    return 'missing ' + topType.name + '.' + member;
                const list = accessor.call(first);
                if (childAccessor.call(first) !== list || accessor === childAccessor ||
                    Object.getPrototypeOf(first) !== childType.prototype ||
                    Object.getPrototypeOf(list) !== child.HTMLCollection.prototype ||
                    list instanceof HTMLCollection ||
                    list[Symbol.iterator] !== child.Array.prototype.values || list.length !== 0)
                    return 'borrowed ' + member + ' getter lost the original creation realm';
                // Keep Area descendants absent during adoption: their reached
                // hosted moving effects are a separate native producer scope.
                if (document.adoptNode(first) !== first || document.adoptNode(late) !== late ||
                    first.ownerDocument !== document || late.ownerDocument !== document ||
                    first[member] !== list)
                    return 'adoption lost the live member collection association';
                saved.push({first, late, list, accessor, member, tag,
                    prototype: child.HTMLCollection.prototype, values: child.Array.prototype.values});
            }
            globalThis.memberRealmProbe = saved;
            return 'ok';
        })()"
        ),
        "ok",
    );
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
        panic!("native collection creation Document was not disassociated");
    })
    .unwrap();
    assert_eq!(
        observe(
            &mut host,
            &permit,
            r"(() => {
            for (const {first, late, list, accessor, member, tag, prototype, values} of memberRealmProbe) {
                const lateList = accessor.call(late);
                if (first[member] !== list || late[member] !== lateList || lateList === list ||
                    Object.getPrototypeOf(list) !== prototype ||
                    Object.getPrototypeOf(lateList) !== prototype ||
                    lateList[Symbol.iterator] !== values || lateList.length !== 0)
                    return 'creation realm retirement lost warm or first-exposed ' + member;
                for (const [root, view] of [[first, list], [late, lateList]]) {
                    const added = document.createElement(tag);
                    root.appendChild(added);
                    if (root[member] !== view || view.length !== 1 || view[0] !== added ||
                        view.item(0) !== added || [...view][0] !== added)
                        return 'retained ' + member + ' missed mutation after creation realm retirement';
                    added.remove();
                    if (view.length !== 0 || view.item(0) !== null)
                        return 'retained ' + member + ' missed removal after retirement';
                }
            }
            return 'ok';
        })()"
        ),
        "ok",
    );
    host.stop().into_session().unwrap();
}

pub(super) fn destroyed_member_roots_preserve_the_first_native_failure() {
    for (local, member) in [("map", "areas"), ("datalist", "options")] {
        // Each operation gets its own host: an earlier terminal failure must
        // not masquerade as fresh validation by a different collection route.
        for operation in ["cold-root", "warm-root", "length", "item", "name", "keys"] {
            let (session, top) = setup();
            let (mut host, _) = attach_event_window_with_work(
                session,
                top,
                Arc::new(cancel()),
                200_000,
                |_, _, _| {},
            );
            let permit = host.permit().unwrap();
            let before = host.snapshot().unwrap();
            assert_eq!(
                observe(
                    &mut host,
                    &permit,
                    &format!("globalThis.memberRoot = document.createElement('{local}'); 'ok'")
                ),
                "ok"
            );
            if operation != "cold-root" {
                assert_eq!(
                    observe(
                        &mut host,
                        &permit,
                        &format!(
                            "globalThis.memberList = memberRoot.{member}; String(memberList.length)"
                        )
                    ),
                    "0"
                );
            }
            host.with_native(|dom, _, _| {
                let snapshot = dom.snapshot();
                let mut created = snapshot.nodes().filter(|node| before.node(*node).is_err());
                let root = created.next().unwrap();
                assert!(created.next().is_none());
                let mut edit = DomBatch::new();
                edit.destroy(root);
                dom.apply(edit, &cancel()).unwrap();
                assert!(matches!(
                    dom.snapshot().node(root),
                    Err(DomError::StaleNode)
                ));
            })
            .unwrap();
            let stamp = host.snapshot().unwrap().stamp();
            let script = match operation {
                "cold-root" | "warm-root" => format!("memberRoot.{member}"),
                "length" => "memberList.length".into(),
                "item" => "memberList.item(0)".into(),
                "name" => "memberList.namedItem('missing')".into(),
                "keys" => "Reflect.ownKeys(memberList)".into(),
                _ => unreachable!(),
            };
            let failure = native_failure(host.in_document(&permit, |scope| {
                scope
                    .evaluate(&script.encode_utf16().collect::<Vec<_>>())
                    .map(drop)
            }));
            let first = failure.cause::<DocumentGuestEventError>().unwrap();
            assert!(
                matches!(
                    first,
                    DocumentGuestEventError::Document(DocumentScriptError::Native(
                        DocumentScriptNativeError::Dom(DomError::StaleNode)
                    ))
                ),
                "{local}.{member}: {operation}: {first:?}"
            );
            let repeated = native_failure(host.checkpoint());
            assert!(std::ptr::eq(
                first,
                repeated.cause::<DocumentGuestEventError>().unwrap()
            ));
            assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        }
    }
}
