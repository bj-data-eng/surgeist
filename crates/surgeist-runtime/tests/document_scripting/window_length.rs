//! Window.length against actual native child creation and removal in one attached realm.

use super::*;
use surgeist_dom::hosted::DomHostedNavigableId;

pub(super) fn attached_window_length_tracks_current_native_children() {
    let (mut session, top) = setup();
    let document = top.document();
    let snapshot = session.snapshot();
    let html = snapshot.node(document.node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];

    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(document, None);
    let root = batch.create_element_ns(&detached, Namespace::html(), "div");
    let frame = batch.create_element_ns(&detached, Namespace::html(), "frame");
    let iframe = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    batch.insert_before(&frame, &root, None);
    batch.insert_before(&iframe, &root, None);
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let frame = commit.created(&frame).unwrap().unwrap();
    let iframe = commit.created(&iframe).unwrap().unwrap();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, None)
        .unwrap();
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), document);
    let before = host.snapshot().unwrap();
    assert_eq!(before.node(root).unwrap().parent(), None);
    for container in [frame, iframe] {
        assert_eq!(content_navigable(&before, container), None);
        assert!(!before.node(container).unwrap().is_connected());
    }

    let empty = observe(
        &mut host,
        &permit,
        r#"(() => {
            const receiver = window;
            const descriptor = Object.getOwnPropertyDescriptor(receiver, "length");
            if (typeof descriptor?.get !== "function") return "missing Window.length getter";
            const {get, set, configurable, enumerable} = descriptor;
            if (!configurable || !enumerable || "value" in descriptor || "writable" in descriptor
                || typeof set !== "function" || get.name !== "get length" || get.length !== 0
                || set.name !== "set length" || set.length !== 1) {
                return "Window.length must be an own configurable enumerable Replaceable accessor";
            }
            const throwsTypeError = invoke => {
                try { invoke(); }
                catch (error) { return error instanceof TypeError; }
                return false;
            };
            for (const accessor of [get, set]) {
                if (Object.hasOwn(accessor, "prototype") ||
                    !throwsTypeError(() => Reflect.construct(function () {}, [], accessor))) {
                    return "Window.length accessors must be built-in non-constructors";
                }
                for (const other of [{}, Object.create(receiver), new Proxy(receiver, {}), 1]) {
                    if (!throwsTypeError(() => Reflect.apply(accessor, other, [1]))) {
                        return "Window.length must reject a foreign receiver";
                    }
                }
            }
            if (receiver !== globalThis || receiver.length !== 0 || get() !== 0
                || Reflect.apply(get, null, []) !== 0 || Reflect.apply(get, undefined, []) !== 0) {
                return "the original empty Window must report zero for its default receivers";
            }
            const marker = { [Symbol.toPrimitive]() { throw new Error("unexpected length coercion"); } };
            globalThis.windowLengthProbe = {receiver, get, set, marker};
            return "ok";
        })()"#,
    );

    edit(&mut host, DomSourceEdit::insert_before(root, body, None));
    let populated = host.snapshot().unwrap();
    assert_eq!(populated.node(root).unwrap().children(), &[frame, iframe]);
    assert_eq!(populated.node(root).unwrap().parent(), Some(body));
    let [frame_child, iframe_child] = [frame, iframe].map(|container| {
        let id = content_navigable(&populated, container).expect("published native content edge");
        let navigable = populated.hosted_navigable(id).unwrap();
        assert_eq!(navigable.container(), Some(container));
        assert_eq!(navigable.parent(), Some(top.navigable()));
        assert_eq!(
            populated.node(container).unwrap().owner_document(),
            Some(document)
        );
        let child = navigable
            .active_document()
            .expect("active native child Document");
        assert_ne!(child, document);
        assert_eq!(
            populated.global_document(navigable.global()).unwrap(),
            child
        );
        assert!(
            populated
                .document(child)
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
        id
    });
    assert_ne!(frame_child, iframe_child);

    let two = observe(
        &mut host,
        &permit,
        r#"(() => {
            const probe = globalThis.windowLengthProbe;
            if (!probe) return "missing saved Window.length accessors";
            const {receiver, get, set, marker} = probe;
            if (window !== receiver || window.length !== 2 || get() !== 2) {
                return "the original Window must report both published children";
            }
            const replaced = (value) => {
                const own = Object.getOwnPropertyDescriptor(receiver, "length");
                return own?.value === value && own.writable && own.enumerable && own.configurable
                    && !("get" in own) && !("set" in own);
            };
            if (!Reflect.set(receiver, "length", marker) || !replaced(marker)) {
                return "assignment must keep the exact value as an own Replaceable data property";
            }
            Reflect.apply(set, null, [undefined]);
            if (!replaced(undefined)) return "the null setter receiver must default to Window";
            Reflect.apply(set, undefined, [marker]);
            if (!replaced(marker) || get.call(receiver) !== 2) {
                return "default setter and saved getter must keep the native Window identity";
            }
            return "ok";
        })()"#,
    );

    edit(&mut host, DomSourceEdit::detach(frame));
    let after_one = host.snapshot().unwrap();
    assert_detached(&after_one, frame);
    assert_eq!(content_navigable(&after_one, iframe), Some(iframe_child));
    assert!(
        after_one
            .hosted_navigable(frame_child)
            .unwrap()
            .is_retiring()
    );
    let one = observe(
        &mut host,
        &permit,
        r#"(() => {
            const probe = globalThis.windowLengthProbe;
            if (!probe) return "missing saved Window.length accessors";
            const {receiver, get, marker} = probe;
            if (window !== receiver || window.length !== marker || get.call(receiver) !== 1) {
                return "replacement must coexist with the live native count after removal";
            }
            if (!Reflect.deleteProperty(receiver, "length") || Object.hasOwn(receiver, "length")
                || get.call(null) !== 1) {
                return "deletion must remove the replacement and leave the saved native getter usable";
            }
            return "ok";
        })()"#,
    );

    edit(&mut host, DomSourceEdit::detach(iframe));
    let after_both = host.snapshot().unwrap();
    assert_detached(&after_both, iframe);
    assert!(
        after_both
            .hosted_navigable(iframe_child)
            .unwrap()
            .is_retiring()
    );
    let none = observe(
        &mut host,
        &permit,
        r#"(() => {
            const probe = globalThis.windowLengthProbe;
            if (!probe) return "missing saved Window.length accessors";
            const {receiver, get} = probe;
            if (window !== receiver || Object.hasOwn(receiver, "length") || get() !== 0) {
                return "the saved getter must return zero after both removals";
            }
            return "ok";
        })()"#,
    );

    // Check guest results after the arrangement has proved the actual native lifecycle.
    for (stage, observation) in [
        ("empty Document", empty),
        ("two published children", two),
        ("one child after removal", one),
        ("both children removed", none),
    ] {
        assert_eq!(observation, "ok", "{stage}");
    }
}

pub(super) fn observe(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    script: &str,
) -> String {
    let result = host
        .in_document(permit, |scope| {
            let outcome = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&outcome)
        })
        .unwrap_or_else(|error| {
            if let DocumentScriptError::Script(Error::HostFailure(failure)) = &error {
                panic!(
                    "Window observation failed: {:?}",
                    std::error::Error::source(failure)
                );
            }
            panic!("Window observation failed: {error:?}");
        });
    String::from_utf16(&result).unwrap()
}

pub(super) fn edit(host: &mut DocumentScriptHost, edit: DomSourceEdit) {
    host.with_native(|dom, _, mut source| {
        let mut services = DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        };
        let limits = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1);
        let progress = source
            .apply(dom, edit, limits, &mut services, &cancel())
            .unwrap();
        assert!(
            matches!(progress, DomSourceDriveProgress::Complete(_)),
            "{progress:?}"
        );
    })
    .unwrap();
}

pub(super) fn content_navigable(
    snapshot: &DomSnapshot,
    container: NodeId,
) -> Option<DomHostedNavigableId> {
    snapshot
        .node(container)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .expect("native Frame or Iframe")
}

fn assert_detached(snapshot: &DomSnapshot, container: NodeId) {
    assert_eq!(snapshot.node(container).unwrap().parent(), None);
    assert_eq!(content_navigable(snapshot, container), None);
}
