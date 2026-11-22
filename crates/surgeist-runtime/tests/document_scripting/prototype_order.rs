use super::{window_length::observe, *};

pub(super) fn core_idl_prototypes_follow_source_order() {
    let (mut session, top) = setup();
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
        (8_000_000, 6_000_000, 220_000),
        surgeist_ledger::ProfileMode::Off,
        DomSourceLimits::new(16_000_000, 1_000_000, 200_000, 64, 64, 64, 4),
        |_, _, _| {},
    );
    let permit = host.permit().unwrap();
    window_length::edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
          const verify = realm => {
            const {Node, Attr, Document, DocumentFragment, HTMLDocument, HTMLCollection,
                   TreeWalker, NodeIterator, DOMImplementation, Range, Event} = realm;
            // Pinned Web IDL groups ordinary attributes, then operations,
            // indexed iteration, constants and the constructor backlink.
            const exact = (interface, expected) => {
                const actual = Object.getOwnPropertyNames(interface.prototype);
                return actual.join(' ') === expected
                    ? null : interface.name + ': ' + actual.join(' ');
            };
            const ordered = (interface, expected) => {
                const actual = Object.getOwnPropertyNames(interface.prototype);
                let last = -1;
                for (const key of expected.split(' ')) {
                    const found = actual.indexOf(key);
                    if (found <= last) return interface.name + ' relative: ' + actual.join(' ');
                    last = found;
                }
                return null;
            };
            for (const error of [
                exact(Node, 'nodeType nodeName baseURI isConnected ownerDocument parentNode parentElement ' +
                    'childNodes firstChild lastChild previousSibling nextSibling nodeValue textContent ' +
                    'getRootNode hasChildNodes normalize cloneNode isEqualNode isSameNode compareDocumentPosition ' +
                    'contains lookupPrefix lookupNamespaceURI isDefaultNamespace insertBefore appendChild ' +
                    'replaceChild removeChild ELEMENT_NODE ATTRIBUTE_NODE TEXT_NODE CDATA_SECTION_NODE ' +
                    'ENTITY_REFERENCE_NODE ENTITY_NODE PROCESSING_INSTRUCTION_NODE COMMENT_NODE DOCUMENT_NODE ' +
                    'DOCUMENT_TYPE_NODE DOCUMENT_FRAGMENT_NODE NOTATION_NODE DOCUMENT_POSITION_DISCONNECTED ' +
                    'DOCUMENT_POSITION_PRECEDING DOCUMENT_POSITION_FOLLOWING DOCUMENT_POSITION_CONTAINS ' +
                    'DOCUMENT_POSITION_CONTAINED_BY DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC constructor'),
                exact(Attr, 'namespaceURI prefix localName name value ownerElement specified constructor'),
                exact(DocumentFragment, 'children firstElementChild lastElementChild childElementCount ' +
                    'getElementById prepend append replaceChildren moveBefore constructor'),
                exact(HTMLCollection, 'length item namedItem constructor'),
                exact(TreeWalker, 'root whatToShow filter currentNode parentNode firstChild lastChild ' +
                    'previousSibling nextSibling previousNode nextNode constructor'),
                exact(NodeIterator, 'root referenceNode pointerBeforeReferenceNode whatToShow filter ' +
                    'nextNode previousNode detach constructor'),
                exact(DOMImplementation, 'createDocumentType createDocument createHTMLDocument hasFeature constructor'),
                ordered(Document, 'implementation URL documentURI compatMode characterSet charset inputEncoding ' +
                    'contentType doctype documentElement title body defaultView ' +
                    'getElementsByTagName getElementsByTagNameNS getElementsByClassName createElement ' +
                    'createElementNS createDocumentFragment createTextNode createCDATASection createComment ' +
                    'createProcessingInstruction importNode adoptNode createAttribute createAttributeNS createEvent ' +
                    'createRange createNodeIterator createTreeWalker getElementsByName constructor'),
                ordered(Range, 'commonAncestorContainer setStart setEnd detach START_TO_START constructor'),
            ]) {
                if (error !== null) return error;
            }
            const index = Object.getOwnPropertyDescriptor(HTMLCollection.prototype, 'item');
            const attr = Object.getOwnPropertyDescriptor(Node.prototype, 'nodeValue');
            const constant = Object.getOwnPropertyDescriptor(Node.prototype, 'ELEMENT_NODE');
            const constructor = Object.getOwnPropertyDescriptor(Node.prototype, 'constructor');
            if (!index?.enumerable || !index.configurable || !index.writable || index.value.length !== 1 ||
                !attr?.enumerable || !attr.configurable || attr.get.name !== 'get nodeValue' ||
                attr.set.name !== 'set nodeValue' || constant.value !== 1 || constant.writable ||
                constant.configurable || !constant.enumerable || constructor.value !== Node ||
                constructor.enumerable || !constructor.configurable || !constructor.writable ||
                HTMLDocument !== Document || HTMLCollection.prototype[Symbol.iterator] !== realm.Array.prototype.values ||
                Object.hasOwn(Event.prototype, 'isTrusted') || !(new Event('order') instanceof Event))
                return 'incorrect descriptor or original realm intrinsic';
            return null;
          };
          return verify(globalThis) ?? verify(window[0]) ?? 'ok';
        })()"#,
    );
    assert_eq!(result, "ok");
    host.stop().into_session().unwrap();
}
