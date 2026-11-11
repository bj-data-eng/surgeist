//! Native-backed DocumentFragment construction and Node projections.

use super::*;

pub(super) fn creates_fragments_through_the_canonical_document() {
    let (mut host, _) = prepare_event_window();
    let permit = host.permit().unwrap();
    let before = count_fragments(&host);
    let script = r##"(() => {
        if (typeof DocumentFragment !== "function") return "missing DocumentFragment interface";
        const current = window.document;
        const secondary = new Document();
        const factory = Document.prototype.createDocumentFragment;
        if (typeof factory !== "function") return "missing Document fragment factory";
        let wrongReceiver = false;
        try { Reflect.apply(factory, Object.create(Document.prototype), []); }
        catch (error) { wrongReceiver = error instanceof TypeError; }
        if (!wrongReceiver) return "Document fragment factory must qualify the receiver";
        const constructor = new DocumentFragment();
        class Derived extends DocumentFragment {}
        const derived = new Derived();
        const factoryFragment = secondary.createDocumentFragment();
        if (Object.getPrototypeOf(DocumentFragment) !== Node
            || Object.getPrototypeOf(DocumentFragment.prototype) !== Node.prototype
            || Object.getPrototypeOf(derived) !== Derived.prototype
            || new Set([constructor, derived, factoryFragment]).size !== 3) {
            return "Fragment must preserve interface inheritance and identity";
        }
        for (const [fragment, document] of [[constructor, current], [derived, current],
            [factoryFragment, secondary]]) {
            if (!(fragment instanceof DocumentFragment && fragment instanceof Node
                && fragment instanceof EventTarget)
                || fragment.nodeType !== 11 || fragment.nodeName !== "#document-fragment"
                || fragment.ownerDocument !== document || fragment.parentNode !== null
                || fragment.isConnected || fragment.getRootNode() !== fragment) {
                return "Fragment Node reads must reflect its original native creation";
            }
        }
        return "ok";
    })()"##;
    let result = host
        .in_document(&permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
    assert_eq!(count_fragments(&host), before + 3);
}

fn count_fragments(host: &DocumentScriptHost) -> usize {
    let snapshot = host.snapshot().unwrap();
    snapshot
        .nodes()
        .filter(|id| {
            matches!(
                snapshot.node(*id).unwrap().kind(),
                DomNodeKind::DocumentFragment
            )
        })
        .count()
}
