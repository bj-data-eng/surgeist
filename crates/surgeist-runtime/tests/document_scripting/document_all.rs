use super::{window_length::observe, *};

pub(super) fn uses_the_canonical_live_document() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let (named, numeric, unsupported) = host
        .with_native(|dom, _, _| {
            let document = top.document();
            let snapshot = dom.snapshot();
            let html = snapshot.node(document.node()).unwrap().children()[0];
            let body = snapshot.node(html).unwrap().children()[1];
            let mut batch = DomBatch::new();
            let named = batch.create_element_ns(document, Namespace::html(), "input");
            batch.set_attribute(&named, "id", "unique");
            batch.set_attribute(&named, "name", "shared");
            batch.insert_before(&named, body, None);
            let numeric = batch.create_element_ns(document, Namespace::html(), "div");
            batch.set_attribute(&numeric, "id", "99");
            batch.insert_before(&numeric, body, None);
            let unsupported = batch.create_element_ns(document, Namespace::html(), "div");
            batch.set_attribute(&unsupported, "id", "shared");
            batch.set_attribute(&unsupported, "name", "only-div-name");
            batch.insert_before(&unsupported, body, None);
            let receipt = dom.apply(batch, &cancel()).unwrap();
            (
                receipt.created(&named).unwrap().unwrap(),
                receipt.created(&numeric).unwrap().unwrap(),
                receipt.created(&unsupported).unwrap().unwrap(),
            )
        })
        .unwrap();

    let first = observe(
        &mut host,
        &permit,
        r#"(() => {
            const all = document.all;
            if (typeof HTMLAllCollection !== "function" || all === undefined)
                return "the native Document must expose a callable collection";
            const same = document.all;
            if (all !== same || !(all instanceof HTMLAllCollection) || Boolean(all)
                || typeof all !== "undefined" || all != null || all != undefined
                || all === null || all === undefined || all.length !== 6
                || Object.getPrototypeOf(all) !== HTMLAllCollection.prototype) return "identity and type";
            const numbered = document.getElementById("99");
            const input = document.getElementById("unique");
            const matches = all.namedItem("shared");
            const inherited = Object.getOwnPropertyDescriptor(all, "unique");
            if (all[0] !== document.documentElement || all[3] !== input || all[4] !== numbered
                || all.item("3") !== input || all.item("99") !== null
                || all.namedItem("99") !== numbered || all[99] !== undefined
                || all.item() !== null || all() !== null || all(undefined) !== null
                || all.namedItem("only-div-name") !== null || all.unique !== input
                || inherited?.value !== input || inherited.enumerable || inherited.writable
                || all.namedItem("") !== null || !(matches instanceof HTMLCollection)
                || matches.length !== 2 || matches[0] !== input
                || matches[1] !== document.getElementById("shared")) return "native collection semantics";
            const keys = Reflect.ownKeys(all);
            if (keys.join() !== "0,1,2,3,4,5,unique,shared,99"
                || Object.keys(all).join() !== "0,1,2,3,4,5") return "legacy keys";
            if (Reflect.apply(all, {}, ["unique"]) !== input || all("shared").length !== 2)
                return "callable collection ignores the supplied receiver";
            const marker = {};
            const abrupt = {toString() { throw marker; }};
            try { all(abrupt); return "call swallowed the original exception"; }
            catch (error) { if (error !== marker) return "call replaced the original exception"; }
            const second = document.implementation.createDocument("urn:all", "root", null);
            const other = second.all;
            if (other === all || other !== second.all || other.length !== 1
                || Reflect.apply(other, all, ["0"]) !== second.documentElement
                || Reflect.apply(all, other, ["0"]) !== document.documentElement) return "call holder";
            globalThis.documentAllProbe = {all, matches, input};
            return "ok";
        })()"#,
    );
    assert_eq!(first, "ok");

    host.with_native(|dom, _, _| {
        let mut edit = DomBatch::new();
        edit.detach(named);
        edit.detach(numeric);
        edit.set_attribute(unsupported, "id", "changed");
        dom.apply(edit, &cancel()).unwrap();
    })
    .unwrap();
    let after = observe(
        &mut host,
        &permit,
        r#"(() => {
            const {all, matches, input} = documentAllProbe;
            if (document.all !== all || all.length !== 4 || all.unique !== undefined
                || all.namedItem("shared") !== null || all("99") !== null
                || all("changed") !== all[3] || matches.length !== 0 || input.parentNode !== null)
                return "retained collections must observe native changes";
            return "ok";
        })()"#,
    );
    assert_eq!(after, "ok");
}
