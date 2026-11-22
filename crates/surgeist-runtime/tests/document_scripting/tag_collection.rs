use super::*;

pub(super) fn reflects_live_tag_collection_properties() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let (first, second) = host
        .with_native(|dom, _, _| {
            let snapshot = dom.snapshot();
            let html = snapshot.node(top.document().node()).unwrap().children()[0];
            let body = snapshot.node(html).unwrap().children()[1];
            let mut edit = DomBatch::new();
            let first = edit.create_element_ns(top.document(), Namespace::html(), "div");
            let second = edit.create_element_ns(top.document(), Namespace::html(), "div");
            edit.set_attribute(&first, "id", "first");
            edit.set_attribute(&first, "name", "duplicate");
            edit.set_attribute(&second, "id", "item");
            edit.set_attribute(&second, "name", "duplicate");
            edit.insert_before(&first, body, None);
            edit.insert_before(&second, body, None);
            let receipt = dom.apply(edit, &cancel()).unwrap();
            (
                receipt.created(&first).unwrap().unwrap(),
                receipt.created(&second).unwrap().unwrap(),
            )
        })
        .unwrap();

    let observed = host.in_document(&permit, |scope| {
        let script = r#"(() => {
                const c = document.getElementsByTagName("DiV");
                globalThis.observedCollection = c;
                const first = document.getElementById("first");
                const second = document.getElementById("item");
                const indexed = Object.getOwnPropertyDescriptor(c, "0");
                const named = Object.getOwnPropertyDescriptor(c, "duplicate");
                if (c.length !== 2 || c.item(0) !== first || c[1] !== second ||
                    c.namedItem("duplicate") !== first || c.namedItem("item") !== second ||
                    c.namedItem("") !== null || c.namedItem("missing") !== null ||
                    c.duplicate !== first || c.first !== first || c.missing !== undefined ||
                    c.item !== HTMLCollection.prototype.item) return "named and indexed reads";
                if (c[Symbol.iterator] !== Array.prototype.values ||
                    [...c].length !== 2 || [...c][0] !== first || [...c][1] !== second) return "iterator";
                if (indexed?.value !== first || indexed.writable || !indexed.enumerable || !indexed.configurable ||
                    named?.value !== first || named.writable || named.enumerable || !named.configurable) return "descriptors";
                const names = Object.getOwnPropertyNames(c);
                if (Object.keys(c).join(",") !== "0,1" ||
                    !names.includes("duplicate") || names.includes("item")) return "own keys";
                let converted = 0;
                const name = { toString() { converted++; return "first"; } };
                try { HTMLCollection.prototype.namedItem.call({}, name); return "forged receiver"; }
                catch (error) { if (!(error instanceof TypeError) || converted !== 0) return "receiver order"; }
                if (Reflect.defineProperty(c, "0", { value: {} })) return "indexed definition";
                if (Reflect.deleteProperty(c, "0") || !Reflect.deleteProperty(c, "77")) return "indexed deletion";
                if (Reflect.defineProperty(c, "duplicate", { value: {} })) return "named definition";
                if (Reflect.deleteProperty(c, "duplicate")) return "named deletion";
                if (Reflect.set(c, "77", {}) || Object.hasOwn(c, "77")) return "indexed assignment";
                if (Reflect.preventExtensions(c) || !Object.isExtensible(c)) return "extensibility";
                const originalPrototype = Object.getPrototypeOf(c);
                const ordinary = {};
                if (!Reflect.setPrototypeOf(c, ordinary) ||
                    Reflect.setPrototypeOf(ordinary, c) ||
                    !Reflect.setPrototypeOf(c, originalPrototype)) return "ordinary prototype";
                return "ok";
            })()"#;
        let observed = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
        scope.to_utf16(&observed)
    })
    .unwrap();
    assert_eq!(String::from_utf16(&observed).unwrap(), "ok");

    host.with_native(|dom, _, _| {
        let mut edit = DomBatch::new();
        edit.detach(first);
        edit.set_attribute(second, "name", "changed");
        dom.apply(edit, &cancel()).unwrap();
    })
    .unwrap();
    eval(
        &mut host,
        &permit,
        r#"Number(observedCollection.length === 1 &&
            observedCollection[0] === document.getElementById("item") &&
            observedCollection.namedItem("duplicate") === null &&
            observedCollection.duplicate === undefined &&
            observedCollection.namedItem("changed") === observedCollection.changed)"#,
        1.0,
    );
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const xml = document.implementation.createDocument("urn:collection", "root", null);
            const upper = xml.createElementNS("urn:collection", "t:Meta");
            const lower = xml.createElementNS("urn:collection", "t:meta");
            const replacement = xml.createElementNS(null, "\uFFFD");
            replacement.setAttribute("id", "77");
            const root = xml.documentElement;
            root.appendChild(upper);
            root.appendChild(lower);
            root.appendChild(replacement);
            const wildcard = xml.getElementsByTagName("*");
            return xml.getElementsByTagName("t:Meta").item(0) === upper &&
                xml.getElementsByTagName("t:meta").item(0) === lower &&
                xml.getElementsByTagName("t:META").length === 0 &&
                xml.getElementsByTagName("\uFFFD").item(0) === replacement &&
                xml.getElementsByTagName("\uD800").length === 0 &&
                wildcard.length === 4 && wildcard.item(0) === root &&
                wildcard.item(1) === upper && wildcard.item(3) === replacement &&
                wildcard.namedItem("77") === replacement &&
                wildcard[77] === undefined &&
                Object.getOwnPropertyDescriptor(wildcard, "77") === undefined &&
                Object.getOwnPropertyNames(wildcard).includes("77");
        })())"#,
        1.0,
    );

    let stamp = host.snapshot().unwrap().stamp();
    let failure = native_failure(host.in_document(&permit, |scope| {
        let script =
            "for (let attempt = 0; attempt < 160_001; attempt++) observedCollection.length";
        scope
            .evaluate(&script.encode_utf16().collect::<Vec<_>>())
            .map(drop)
    }));
    let first = failure.cause::<DocumentGuestEventError>().unwrap();
    assert!(
        matches!(
            first,
            DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::StorageLimit
            ))
        ),
        "{first:?}"
    );
    let repeated = native_failure(host.checkpoint());
    assert!(std::ptr::eq(
        first,
        repeated.cause::<DocumentGuestEventError>().unwrap()
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), stamp);

    retire_attached(&mut host, top);
    assert!(matches!(
        host.in_document::<()>(&permit, |_| panic!(
            "retired collection reentered guest code"
        )),
        Err(DocumentScriptError::Native(
            DocumentScriptNativeError::Revoked
        ))
    ));
}
