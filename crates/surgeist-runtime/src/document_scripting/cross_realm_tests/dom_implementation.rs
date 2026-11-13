//! DOMImplementation identity in the native Window.

use super::*;

pub(super) fn document_implementation_is_the_same_object() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host =
        super::super::guest::tests::attach_public_nodes(session, target, cancel, 5_000, 20_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let value = evaluate(scope, r#"
            (() => {
                const original = document.implementation;
                if (original === undefined) return false;
                const getter = Object.getOwnPropertyDescriptor(Document.prototype, 'implementation').get;
                const hasFeature = DOMImplementation.prototype.hasFeature;
                const throwsTypeError = action => {
                    try { action(); } catch (error) { return error instanceof TypeError; }
                    return false;
                };
                Object.freeze(document);
                const separate = new Document();
                original.extra = 1;
                return original === document.implementation && original.extra === 1 &&
                    Object.getPrototypeOf(original) === DOMImplementation.prototype &&
                    original.hasFeature() === true &&
                    separate.implementation !== original && separate.implementation === separate.implementation &&
                    throwsTypeError(() => new DOMImplementation()) &&
                    throwsTypeError(() => getter.call(Object.create(document))) &&
                    throwsTypeError(() => hasFeature.call(Object.create(original)));
            })()
        "#)?;
        assert!(scope.boolean(&value)?, "each Document has its own branded SameObject implementation");
        Ok(())
    }).unwrap();
}

pub(super) fn implementation_creates_native_doctypes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let document = target.document();
    let mut host =
        super::super::guest::tests::attach_public_nodes(session, target, cancel, 5_000, 20_000);
    let permit = host.permit().unwrap();
    let before = host.snapshot().unwrap();
    host.in_document(&permit, |scope| {
        let value = evaluate(
            scope,
            r#"
            (() => {
                const implementation = document.implementation;
                if (typeof implementation.createDocumentType !== 'function') return false;
                const make = implementation.createDocumentType;
                let coerced = false;
                const input = { toString() { coerced = true; return 'html'; } };
                try { make.call(Object.create(implementation), input, '', ''); return false; }
                catch (error) { if (!(error instanceof TypeError) || coerced) return false; }
                try { implementation.createDocumentType('bad name', '', ''); return false; }
                catch (error) { if (error.name !== 'InvalidCharacterError') return false; }
                const doctype = implementation.createDocumentType(input, '\ud800\0', '\udc00');
                return coerced && doctype instanceof DocumentType && doctype.name === 'html' &&
                    doctype.publicId === '\ud800\0' && doctype.systemId === '\udc00' &&
                    doctype.parentNode === null && doctype.ownerDocument === document;
            })()
        "#,
        )?;
        assert!(
            scope.boolean(&value)?,
            "DOMImplementation creates an exact native doctype"
        );
        Ok(())
    })
    .unwrap();
    let after = host.snapshot().unwrap();
    let mut created = after.nodes().filter(|node| before.node(*node).is_err());
    let node = after.node(created.next().unwrap()).unwrap();
    assert!(created.next().is_none());
    assert_eq!(node.owner_document(), Some(document));
    assert!(matches!(
        node.kind(),
        surgeist_dom::DomNodeKind::DocumentType(_)
    ));
}

pub(super) fn implementation_creates_native_documents() {
    use crate::document_scripting::guest::tests::projected_node_id;
    use surgeist_dom::{DomDocumentClass, DomDocumentInterface, DomNodeKind};

    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host =
        super::super::guest::tests::attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
    let (xml, omitted, undefined, empty, titled) = host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const impl = document.implementation;
                if (typeof impl.createDocument !== 'function' ||
                    typeof impl.createHTMLDocument !== 'function') return 'missing factories';
                const throws = (action, name) => {
                    try { action(); } catch (error) { return error.name === name; }
                    return false;
                };
                let coerced = false;
                const operand = {toString() { coerced = true; return 'p:root'; }};
                const doctype = impl.createDocumentType('root', 'pub', 'sys');
                const before = impl.createDocument(null, null, doctype);
                const valid = before.doctype === doctype &&
                    throws(() => impl.createDocument(null, operand, Object.create(doctype)), 'TypeError') &&
                    coerced &&
                    throws(() => impl.createDocument.call({}, operand, ''), 'TypeError') &&
                    throws(() => impl.createDocument(null, 'p:root', doctype), 'NamespaceError') &&
                    throws(() => impl.createDocument(null, 'bad name', doctype), 'InvalidCharacterError') &&
                    before.doctype === doctype;
                if (!valid) return 'argument validation or adoption ordering';
                const xml = impl.createDocument('http://www.w3.org/2000/svg', 's:svg', doctype);
                if (before.doctype !== null || xml.doctype !== doctype ||
                    doctype.ownerDocument !== xml || doctype.parentNode !== xml ||
                    Object.getPrototypeOf(xml) !== XMLDocument.prototype || !(xml instanceof Document) ||
                    Object.getPrototypeOf(XMLDocument.prototype) !== Document.prototype ||
                    !throws(() => new XMLDocument(), 'TypeError') ||
                    xml.documentElement.ownerDocument !== xml || xml.documentElement.prefix !== 's' ||
                    xml.documentElement.localName !== 'svg' || xml.implementation === impl ||
                    !throws(() => impl.createHTMLDocument.call({}, operand), 'TypeError') ||
                    impl.createDocument.length !== 2 || impl.createHTMLDocument.length !== 0) {
                    return 'XML identity, interface, or signature';
                }
                const omitted = impl.createHTMLDocument();
                const undef = impl.createHTMLDocument(undefined);
                const empty = impl.createHTMLDocument('');
                const titled = impl.createHTMLDocument('\ud800\0');
                for (const html of [omitted, undef, empty, titled]) {
                    if (Object.getPrototypeOf(html) !== Document.prototype ||
                        html.doctype.name !== 'html' || html.doctype.ownerDocument !== html ||
                        html.documentElement.localName !== 'html' ||
                        html.documentElement.ownerDocument !== html) return 'HTML skeleton or interface';
                }
                return {xml, omitted, undef, empty, titled};
            })()
        "#)?;
        if scope.kind(&result)? == surgeist_bindings_js::ValueKind::String {
            panic!("{}", String::from_utf16_lossy(&scope.to_utf16(&result)?));
        }
        let mut document = |name: &str| {
            let name: Vec<_> = name.encode_utf16().collect();
            let value = scope.get(&result, PropertyKey::String(&name))?;
            projected_node_id(scope, &value)
        };
        Ok((document("xml")?, document("omitted")?,
            document("undef")?, document("empty")?, document("titled")?))
    }).unwrap();
    let snapshot = host.snapshot().unwrap();
    let xml = snapshot.node(xml).unwrap().as_document().unwrap();
    assert_eq!(
        xml.metadata().interface(),
        DomDocumentInterface::XmlDocument
    );
    assert_eq!(
        xml.metadata().content_type().code_units(),
        "image/svg+xml".encode_utf16().collect::<Vec<_>>()
    );
    for (document, title) in [
        (omitted, None),
        (undefined, None),
        (empty, Some([].as_slice())),
        (titled, Some([0xd800, 0].as_slice())),
    ] {
        let document = snapshot.node(document).unwrap().as_document().unwrap();
        assert_eq!(document.metadata().class(), DomDocumentClass::Html);
        let root = snapshot.node(document.document_element().unwrap()).unwrap();
        let [head, _body] = root.children() else {
            panic!("HTML skeleton")
        };
        let head = snapshot.node(*head).unwrap();
        if let Some(title) = title {
            let [element] = head.children() else {
                panic!("title element")
            };
            let element = snapshot.node(*element).unwrap();
            let [text] = element.children() else {
                panic!("title text node")
            };
            let DomNodeKind::Text(text) = snapshot.node(*text).unwrap().kind() else {
                panic!("native title text")
            };
            assert_eq!(text.code_units(), title);
        } else {
            assert!(head.children().is_empty());
        }
    }
}
