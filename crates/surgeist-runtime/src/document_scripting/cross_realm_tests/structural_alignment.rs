use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

// HTML877d's modern interfaces and obsolete partial interfaces independently
// specify align as a CEReactions/Reflect DOMString for these three families.
pub(super) fn structural_alignment_reflects_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const families = [['div', HTMLDivElement], ['p', HTMLParagraphElement],
                    ...['h1', 'h2', 'h3', 'h4', 'h5', 'h6'].map(tag => [tag, HTMLHeadingElement])];
                for (const [tag, Type] of families) {
                    if (!(document.createElement(tag) instanceof Type)) return tag + ' interface';
                }
                if (document.createElement('hgroup') instanceof HTMLHeadingElement)
                    return 'hgroup is not a heading interface';
                for (const [tag, Type] of families) {
                    if (Object.keys(Type.prototype).join() !== 'align')
                        return tag + ' missing reflected align';
                    const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, 'align');
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return tag + ' align descriptor';
                    const element = document.createElement(tag);
                    if (element.align !== '') return tag + ' absent align default';
                    element.align = null;
                    if (element.align !== 'null' || element.getAttributeNS(null, 'align') !== 'null')
                        return tag + ' plain DOMString reflection';
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
