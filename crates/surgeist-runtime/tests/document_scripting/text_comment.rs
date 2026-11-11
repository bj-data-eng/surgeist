//! Attached V8 tests over canonical Text and Comment nodes.

use super::*;

pub(super) fn creates_canonical_text_and_comments() {
    let (mut host, top) = prepare_event_window();
    let permit = host.permit().unwrap();
    let before = character_nodes(&host, top.document()).len();
    let script = r##"(() => {
        if (typeof Text !== "function" || typeof Comment !== "function"
            || typeof CharacterData !== "function") return "missing character data interfaces";
        const d = window.document;
        const factoryText = Document.prototype.createTextNode;
        const factoryComment = Document.prototype.createComment;
        if (typeof factoryText !== "function" || typeof factoryComment !== "function") {
            return "missing Document character data factories";
        }
        const poison = {};
        const order = [];
        const derived = new Proxy(function () {}, {
            get(target, key) {
                if (key === "prototype") { order.push("prototype"); throw poison; }
                return Reflect.get(target, key);
            }
        });
        let original = false;
        try {
            Reflect.construct(Text, [{ toString() { order.push("data"); return "unused"; } }], derived);
        } catch (error) { original = error === poison; }
        if (!original || order.join(",") !== "data,prototype") return "constructor ordering";
        let forged = false;
        try { Reflect.apply(factoryComment, Object.create(Document.prototype),
            [{ toString() { order.push("forged"); return "unused"; } }]); }
        catch (error) { forged = error instanceof TypeError; }
        if (!forged || order.includes("forged")) return "factory receiver ordering";
        let missing = false;
        try { d.createTextNode(); }
        catch (error) { missing = error instanceof TypeError; }
        if (!missing) return "factory requires data";
        let illegal = false;
        try { new CharacterData(); }
        catch (error) { illegal = error instanceof TypeError; }
        if (!illegal) return "CharacterData is abstract";

        class DerivedText extends Text {}
        const t = new DerivedText("\ud800A\0");
        const c = new Comment();
        const tc = d.createTextNode(undefined);
        const cc = d.createComment("\udc00");
        if (Object.getPrototypeOf(Text) !== CharacterData
            || Object.getPrototypeOf(Comment) !== CharacterData
            || Object.getPrototypeOf(CharacterData) !== Node
            || Object.getPrototypeOf(t) !== DerivedText.prototype) return "prototype hierarchy";
        for (const [node, data, name, type] of [
            [t, "\ud800A\0", "#text", 3], [c, "", "#comment", 8],
            [tc, "undefined", "#text", 3], [cc, "\udc00", "#comment", 8],
        ]) {
            if (!(node instanceof CharacterData && node instanceof Node && node instanceof EventTarget)
                || node.ownerDocument !== d || node.parentNode !== null || node.isConnected
                || node.getRootNode() !== node || node.nodeType !== type || node.nodeName !== name
                || node.data !== data || node.length !== data.length) return "native character node facts";
        }
        globalThis.projectedText = t;
        return "ok";
    })()"##;
    let actual = host
        .in_document(&permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&actual).unwrap(), "ok");
    let snapshot = host.snapshot().unwrap();
    let nodes = character_nodes(&host, top.document());
    assert_eq!(nodes.len(), before + 4);
    let text = nodes
        .into_iter()
        .find(|node| {
            matches!(snapshot.node(*node).unwrap().kind(), DomNodeKind::Text(data)
                if data.code_units() == [0xd800, 65, 0])
        })
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.replace_data(text, 1, 1, "B");
        let destination = batch.create_document();
        batch.adopt(text, &destination);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    eval(
        &mut host,
        &permit,
        r#"Number(projectedText.data === "\ud800B\0" && projectedText.length === 3
            && projectedText.ownerDocument instanceof Document
            && projectedText.ownerDocument !== document
            && projectedText.ownerDocument === projectedText.ownerDocument
            && projectedText.getRootNode() === projectedText)"#,
        1.0,
    );
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(text);
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let failure = native_failure(host.in_document(&permit, |scope| {
        scope
            .evaluate(&"projectedText.data".encode_utf16().collect::<Vec<_>>())
            .map(drop)
    }));
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}

fn character_nodes(host: &DocumentScriptHost, document: DocumentId) -> Vec<NodeId> {
    let snapshot = host.snapshot().unwrap();
    snapshot
        .nodes()
        .filter(|id| {
            let node = snapshot.node(*id).unwrap();
            node.owner_document() == Some(document)
                && matches!(node.kind(), DomNodeKind::Text(_) | DomNodeKind::Comment(_))
        })
        .collect()
}

pub(super) fn edits_canonical_character_data_and_live_ranges() {
    let (mut host, top) = prepare_event_window();
    let permit = host.permit().unwrap();
    eval(
        &mut host,
        &permit,
        r#"Number((globalThis.edited = document.createTextNode("A\ud800B\udc00")).length === 4)"#,
        1.0,
    );
    let snapshot = host.snapshot().unwrap();
    let node = character_nodes(&host, top.document())
        .into_iter()
        .find(|id| {
            matches!(snapshot.node(*id).unwrap().kind(), DomNodeKind::Text(data)
                if data.code_units() == [65, 0xd800, 66, 0xdc00])
        })
        .unwrap();
    let range = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let range = batch.create_range(top.document());
            batch.set_range_start(&range, node, 2);
            batch.set_range_end(&range, node, 4);
            dom.apply(batch, &cancel())
                .unwrap()
                .created_range(&range)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            if (edited.substringData(1, 1) !== "\ud800"
                || edited.substringData(3, -1) !== "\udc00"
                || edited.substringData(4, 1) !== "") return false;
            let rejected = false;
            try { edited.deleteData(-1, 0); }
            catch (error) { rejected = error instanceof DOMException && error.name === "IndexSizeError"; }
            if (!rejected || edited.data !== "A\ud800B\udc00") return false;
            return edited.replaceData(1, 2, "xyz") === undefined && edited.data === "Axyz\udc00";
        })())"#,
        1.0,
    );
    let snapshot = host.snapshot().unwrap();
    let range = snapshot.range(range).unwrap();
    assert_eq!((range.start().offset(), range.end().offset()), (1, 5));
    assert_eq!(
        (range.start().container(), range.end().container()),
        (node, node)
    );
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            edited.appendData({ toString() { edited.insertData(0, "I"); return "Z"; } });
            if (edited.data !== "IAxyz\udc00Z") return false;
            edited.deleteData(1, 4);
            if (edited.data !== "I\udc00Z") return false;
            edited.data = null;
            return edited.data === "" && edited.length === 0;
        })())"#,
        1.0,
    );
    assert!(
        host.snapshot()
            .unwrap()
            .node(node)
            .unwrap()
            .data()
            .unwrap()
            .is_empty()
    );
}
