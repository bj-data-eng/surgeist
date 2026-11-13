//! Real native DocumentType projection in an attached scripting realm.

use super::*;
use surgeist_html::values::DocumentString;

pub(super) fn document_type_tracks_native_tree() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let document = target.document();
    let mut host = super::super::guest::tests::attach_public_nodes(
        session,
        target,
        cancel.clone(),
        5_000,
        20_000,
    );
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let value = evaluate(scope, "document.doctype === null")?;
        assert!(
            scope.boolean(&value)?,
            "a Document with no doctype returns null"
        );
        Ok(())
    })
    .unwrap();

    let node = host
        .with_native(|dom, _, _| {
            let root = dom
                .snapshot()
                .document(document)
                .unwrap()
                .document_element()
                .unwrap();
            let mut batch = DomBatch::new();
            let doctype = batch.create_document_type(
                document,
                "html",
                DocumentString::from_utf16(vec![0xd800, 0]),
                DocumentString::from_utf16(vec![0xdc00, 0x78]),
            );
            batch.insert_before(&doctype, document, Some(root.into()));
            dom.apply(batch, &cancel)
                .unwrap()
                .created(&doctype)
                .unwrap()
                .unwrap()
        })
        .unwrap();

    host.in_document(&permit, |scope| {
        let value = evaluate(scope, r#"
            (() => {
                const doctype = globalThis.savedDoctype = document.doctype;
                const getName = Object.getOwnPropertyDescriptor(DocumentType.prototype, 'name').get;
                const getDoctype = Object.getOwnPropertyDescriptor(Document.prototype, 'doctype').get;
                const throwsTypeError = action => {
                    try { action(); } catch (error) { return error instanceof TypeError; }
                    return false;
                };
                return doctype === document.doctype &&
                    Object.getPrototypeOf(doctype) === DocumentType.prototype &&
                    doctype instanceof Node &&
                    doctype.nodeType === Node.DOCUMENT_TYPE_NODE &&
                    doctype.nodeName === 'html' && doctype.name === 'html' &&
                    doctype.publicId === '\ud800\0' && doctype.systemId === '\udc00x' &&
                    doctype.parentNode === document && doctype.ownerDocument === document &&
                    throwsTypeError(() => new DocumentType()) &&
                    throwsTypeError(() => getName.call(Object.create(doctype))) &&
                    throwsTypeError(() => getDoctype.call(doctype));
            })()
        "#)?;
        assert!(scope.boolean(&value)?, "the native doctype has its standard interface");
        Ok(())
    })
    .unwrap();

    host.in_document(&permit, |scope| {
        let value = evaluate(
            scope,
            r#"
            (() => {
                document.removeChild(savedDoctype);
                if (document.doctype !== null || savedDoctype.parentNode !== null) return false;
                savedDoctype.extra = 1;
                document.insertBefore(savedDoctype, document.documentElement);
                return document.doctype === savedDoctype && document.doctype.extra === 1;
            })()
        "#,
        )?;
        assert!(
            scope.boolean(&value)?,
            "doctype readback follows native removal and reinsertion"
        );
        Ok(())
    })
    .unwrap();
    assert_eq!(
        host.snapshot().unwrap().node(node).unwrap().parent(),
        Some(document.node())
    );
}
