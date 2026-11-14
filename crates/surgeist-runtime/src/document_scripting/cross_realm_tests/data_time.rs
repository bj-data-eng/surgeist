use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

// HTML877d's HTMLDataElement/HTMLTimeElement IDLs require ordinary DOMString
// reflection. HTML's Reflect naming rule maps dateTime to datetime.
pub(super) fn data_and_time_reflect_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                for (const [tag, Type, member, attribute] of [
                    ['data', HTMLDataElement, 'value', 'value'],
                    ['time', HTMLTimeElement, 'dateTime', 'datetime'],
                ]) {
                    const element = document.createElement(tag);
                    if (!(element instanceof Type)) return tag + ' interface';
                    if (Object.keys(Type.prototype).join() !== member) return tag + ' missing reflected member';
                    const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return tag + ' descriptor';
                    element.appendChild(document.createTextNode('2026-11-14'));
                    if (element[member] !== '') return tag + ' reflection must not fall back to child text';
                    element[member] = null;
                    if (element[member] !== 'null' || element.getAttributeNS(null, attribute) !== 'null')
                        return tag + ' ordinary DOMString reflection';
                    if (tag === 'time' && element.hasAttributeNS(null, 'dateTime')) return 'camelCase content attribute';
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
