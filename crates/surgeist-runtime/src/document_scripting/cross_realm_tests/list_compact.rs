use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

// HTML877d specifies compact as a CEReactions/Reflect boolean on both the
// HTMLDListElement partial interface and the obsolete HTMLDirectoryElement.
pub(super) fn list_compact_reflects_canonical_presence() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                for (const [tag, Type] of [['dl', HTMLDListElement], ['dir', HTMLDirectoryElement]]) {
                    const element = document.createElement(tag);
                    if (!(element instanceof Type)) return tag + ' interface';
                    if (Object.keys(Type.prototype).join() !== 'compact')
                        return tag + ' missing reflected compact';
                    const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, 'compact');
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return tag + ' compact descriptor';
                    if (element.compact !== false) return tag + ' absent compact default';
                    element.dir = 'rtl';
                    element.compact = true;
                    if (element.compact !== true || element.getAttributeNS(null, 'compact') !== '')
                        return tag + ' compact presence';
                    element.compact = false;
                    if (element.compact !== false || element.hasAttributeNS(null, 'compact'))
                        return tag + ' compact removal';
                    if (element.dir !== 'rtl' || element.getAttributeNS(null, 'dir') !== 'rtl' ||
                        Object.hasOwn(Type.prototype, 'dir')) return tag + ' inherited direction string';
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
