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
