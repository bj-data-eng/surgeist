use super::*;

pub(super) fn reflects_live_tag_collection_properties() {
    let (mut host, top) = prepare_event_window();
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

    host.in_document(&permit, |scope| {
        assert_script_number(
            scope,
            r#"Number((() => {
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
                    c.item !== HTMLCollection.prototype.item ||
                    indexed?.value !== first || indexed.writable || !indexed.enumerable || !indexed.configurable ||
                    named?.value !== first || named.writable || named.enumerable || !named.configurable ||
                    Object.keys(c).join(",") !== "0,1" ||
                    !Object.getOwnPropertyNames(c).includes("duplicate")) {
                    return false;
                }
                let converted = 0;
                const name = { toString() { converted++; return "first"; } };
                try { HTMLCollection.prototype.namedItem.call({}, name); return false; }
                catch (error) { if (!(error instanceof TypeError) || converted !== 0) return false; }
                return !Reflect.defineProperty(c, "0", { value: {} }) &&
                    !Reflect.deleteProperty(c, "0") && Reflect.deleteProperty(c, "77") &&
                    !Reflect.defineProperty(c, "duplicate", { value: {} }) &&
                    !Reflect.deleteProperty(c, "duplicate") &&
                    (Reflect.set(c, "77", {}), !Object.hasOwn(c, "77")) &&
                    !Reflect.preventExtensions(c) && Object.isExtensible(c);
            })())"#,
            1.0,
        )
    })
    .unwrap();

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
}
