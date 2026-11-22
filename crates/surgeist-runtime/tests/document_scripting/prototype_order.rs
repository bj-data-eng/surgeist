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
                   HTMLAllCollection, NodeList, TreeWalker, NodeIterator, DOMImplementation,
                   DocumentType, CharacterData, Text, Comment, CDATASection, ProcessingInstruction,
                   ShadowRoot, Element, SVGElement, MathMLElement, Range, Event,
                   PageTransitionEvent, ToggleEvent, SecurityPolicyViolationEvent,
                   CustomElementRegistry} = realm;
            // Pinned Web IDL groups ordinary attributes, then operations,
            // indexed iteration, constants and the constructor backlink.
            const exact = (interface, expected) => {
                const actual = Object.getOwnPropertyNames(interface.prototype);
                return actual.join(' ') === expected
                    ? null : interface.name + ': ' + actual.join(' ');
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
                exact(DocumentType, 'name publicId systemId before after replaceWith remove constructor'),
                exact(CharacterData, 'data length previousElementSibling nextElementSibling ' +
                    'substringData appendData insertData deleteData replaceData before after replaceWith remove constructor'),
                exact(Text, 'wholeText assignedSlot splitText constructor'),
                exact(Comment, 'constructor'),
                exact(CDATASection, 'constructor'),
                exact(ProcessingInstruction, 'target hasAttributes getAttributeNames getAttribute ' +
                    'setAttribute removeAttribute toggleAttribute hasAttribute constructor'),
                exact(ShadowRoot, 'mode delegatesFocus serializable slotAssignment clonable host ' +
                    'customElementRegistry constructor'),
                exact(DocumentFragment, 'children firstElementChild lastElementChild childElementCount ' +
                    'getElementById prepend append replaceChildren moveBefore constructor'),
                exact(HTMLCollection, 'length item namedItem constructor'),
                exact(HTMLAllCollection, 'length namedItem item constructor'),
                exact(NodeList, 'length item entries keys values forEach constructor'),
                exact(TreeWalker, 'root whatToShow filter currentNode parentNode firstChild lastChild ' +
                    'previousSibling nextSibling previousNode nextNode constructor'),
                exact(NodeIterator, 'root referenceNode pointerBeforeReferenceNode whatToShow filter ' +
                    'nextNode previousNode detach constructor'),
                exact(DOMImplementation, 'createDocumentType createDocument createHTMLDocument hasFeature constructor'),
                exact(Document, 'implementation URL documentURI compatMode characterSet charset inputEncoding ' +
                    'contentType doctype documentElement referrer cookie readyState title dir body head ' +
                    'images embeds plugins links forms scripts defaultView fgColor linkColor vlinkColor alinkColor ' +
                    'bgColor anchors applets all customElementRegistry children firstElementChild lastElementChild ' +
                    'childElementCount ' +
                    'getElementsByTagName getElementsByTagNameNS getElementsByClassName createElement ' +
                    'createElementNS createDocumentFragment createTextNode createCDATASection createComment ' +
                    'createProcessingInstruction importNode adoptNode createAttribute createAttributeNS createEvent ' +
                    'createRange createNodeIterator createTreeWalker getElementsByName clear captureEvents ' +
                    'releaseEvents getElementById prepend append replaceChildren moveBefore constructor'),
                exact(Element, 'namespaceURI prefix localName tagName id className classList slot attributes ' +
                    'shadowRoot customElementRegistry children firstElementChild lastElementChild childElementCount ' +
                    'previousElementSibling nextElementSibling assignedSlot hasAttributes getAttributeNames ' +
                    'getAttribute getAttributeNS setAttribute setAttributeNS removeAttribute removeAttributeNS ' +
                    'toggleAttribute hasAttribute hasAttributeNS getAttributeNode getAttributeNodeNS setAttributeNode ' +
                    'setAttributeNodeNS removeAttributeNode attachShadow getElementsByTagName getElementsByTagNameNS ' +
                    'getElementsByClassName insertAdjacentElement insertAdjacentText prepend append replaceChildren ' +
                    'moveBefore before after replaceWith remove constructor'),
                exact(SVGElement, 'dataset nonce autofocus tabIndex constructor'),
                exact(MathMLElement, 'dataset nonce autofocus tabIndex constructor'),
                exact(Range, 'commonAncestorContainer setStart setEnd detach START_TO_START START_TO_END ' +
                    'END_TO_END END_TO_START constructor'),
                exact(Event, 'type target srcElement currentTarget eventPhase cancelBubble bubbles cancelable ' +
                    'returnValue defaultPrevented composed timeStamp composedPath stopPropagation ' +
                    'stopImmediatePropagation preventDefault initEvent NONE CAPTURING_PHASE AT_TARGET ' +
                    'BUBBLING_PHASE constructor'),
                exact(PageTransitionEvent, 'persisted constructor'),
                exact(ToggleEvent, 'oldState newState source constructor'),
                exact(SecurityPolicyViolationEvent, 'documentURI referrer blockedURI effectiveDirective ' +
                    'violatedDirective originalPolicy sourceFile sample disposition statusCode lineNumber ' +
                    'columnNumber constructor'),
                exact(CustomElementRegistry, 'define get getName whenDefined upgrade initialize constructor'),
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
                HTMLAllCollection.prototype[Symbol.iterator] !== realm.Array.prototype.values ||
                NodeList.prototype[Symbol.iterator] !== NodeList.prototype.values ||
                NodeList.prototype.values !== realm.Array.prototype.values ||
                Object.getPrototypeOf(DocumentFragment.prototype) !== Node.prototype ||
                Object.getPrototypeOf(ShadowRoot.prototype) !== DocumentFragment.prototype ||
                Object.getPrototypeOf(SVGElement.prototype) !== Element.prototype ||
                Object.getPrototypeOf(MathMLElement.prototype) !== Element.prototype ||
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
