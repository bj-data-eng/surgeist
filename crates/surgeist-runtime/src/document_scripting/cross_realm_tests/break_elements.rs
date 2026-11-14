use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

pub(super) fn break_elements_reflect_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                for (const [tag, name, members] of [
                    ['br', 'HTMLBRElement', ['clear']],
                    ['hr', 'HTMLHRElement', ['align', 'color', 'noShade', 'size', 'width']],
                ]) {
                    const element = document.createElement(tag);
                    const type = globalThis[name];
                    if (!(element instanceof type)) return tag + ' interface';
                    if (Object.keys(type.prototype).join() !== members.join())
                        return tag + ' missing reflected members';
                    for (const member of members) {
                        const descriptor = Object.getOwnPropertyDescriptor(type.prototype, member);
                        if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                            !descriptor.enumerable || !descriptor.configurable ||
                            descriptor.get.length !== 0 || descriptor.set.length !== 1)
                            return tag + ' descriptor ' + member;
                        const attribute = member === 'noShade' ? 'noshade' : member;
                        const boolean = member === 'noShade';
                        if (element[member] !== (boolean ? false : '')) return tag + ' default ' + member;
                        element[member] = boolean ? true : 'value';
                        if (element[member] !== (boolean ? true : 'value') ||
                            element.getAttributeNS(null, attribute) !== (boolean ? '' : 'value'))
                            return tag + ' canonical reflection ' + member;
                    }
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
