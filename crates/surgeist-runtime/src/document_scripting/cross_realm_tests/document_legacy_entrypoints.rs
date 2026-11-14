use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

// HTML877d 156302 defines these three operations without effects; 97492
// defines HTMLDocument as an alias of the existing Document interface object.
pub(super) fn document_legacy_entrypoints_preserve_tree_and_constructor_identity() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const root = document.documentElement;
                const head = document.head;
                const body = document.body;
                if (!(document instanceof Document) || root?.localName !== 'html' ||
                    head?.parentNode !== root || head.localName !== 'head' ||
                    body?.parentNode !== root || body.localName !== 'body') return 'native Document fixture';
                for (const member of ['clear', 'captureEvents', 'releaseEvents']) {
                    const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, member);
                    if (typeof descriptor?.value !== 'function') return 'document missing legacy operation ' + member;
                    const method = descriptor.value;
                    if (!descriptor.writable || !descriptor.enumerable || !descriptor.configurable ||
                        method.name !== member || method.length !== 0) return 'legacy operation descriptor ' + member;
                    if (method.call(document) !== undefined) return 'legacy operation return ' + member;
                    if (document.documentElement !== root || document.head !== head || document.body !== body ||
                        head.parentNode !== root || body.parentNode !== root) return 'legacy operation changed tree ' + member;
                }
                const alias = Object.getOwnPropertyDescriptor(globalThis, 'HTMLDocument');
                if (!alias) return 'missing HTMLDocument alias';
                return alias.value === Document && alias.writable && !alias.enumerable && alias.configurable &&
                    Document.name === 'Document' && Document.prototype.constructor === Document &&
                    Object.prototype.toString.call(document) === '[object Document]'
                    ? 'ok' : 'HTMLDocument alias identity and descriptor';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}
