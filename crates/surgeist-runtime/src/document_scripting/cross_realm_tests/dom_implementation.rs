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
