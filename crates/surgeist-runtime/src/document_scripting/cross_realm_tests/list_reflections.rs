use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

// HTML877d's modern/obsolete list IDLs specify these member orders and
// OL.start's reflected default 1, independently of its list starting value.
pub(super) fn list_members_reflect_canonical_attributes() {
    for tag in ["ol", "ul", "menu"] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (session, target) = hosted_session(&cancel, 0);
        let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
        let permit = host.permit().unwrap();
        let script = format!(
            r#"
            (() => {{
                const tag = '{tag}';
                const Type = tag === 'ol' ? HTMLOListElement : tag === 'ul' ? HTMLUListElement : HTMLMenuElement;
                const members = tag === 'ol' ? ['reversed', 'start', 'type', 'compact'] :
                    tag === 'ul' ? ['compact', 'type'] : ['compact'];
                const element = document.createElement(tag);
                if (!(element instanceof Type)) return tag + ' interface';
                if (Object.keys(Type.prototype).join() !== members.join()) return tag + ' missing reflected members';
                for (const member of members) {{
                    const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return tag + ' descriptor ' + member;
                    const expected = member === 'start' ? 1 : member === 'type' ? '' : false;
                    if (element[member] !== expected) return tag + ' absent default ' + member;
                }}
                if (tag === 'ol') {{
                    for (let index = 0; index < 3; index++) element.appendChild(document.createElement('li'));
                    element.reversed = true;
                    if (element.start !== 1 || element.hasAttributeNS(null, 'start'))
                        return 'reflected start is not the reversed list starting value';
                    element.start = undefined;
                    if (element.start !== 0 || element.getAttributeNS(null, 'start') !== '0')
                        return 'undefined setter writes canonical zero';
                }}
                for (const member of members.filter(member => member === 'reversed' || member === 'compact')) {{
                    element[member] = true;
                    if (element[member] !== true || element.getAttributeNS(null, member) !== '')
                        return tag + ' Boolean presence ' + member;
                    element[member] = false;
                    if (element[member] !== false || element.hasAttributeNS(null, member))
                        return tag + ' Boolean removal ' + member;
                }}
                if (tag !== 'menu') {{
                    element.type = null;
                    if (element.type !== 'null' || element.getAttributeNS(null, 'type') !== 'null')
                        return tag + ' ordinary type DOMString';
                }}
                return 'ok';
            }})()
        "#
        );
        host.in_document(&permit, |scope| {
            let result = evaluate(scope, &script)?;
            assert_eq!(
                String::from_utf16_lossy(&scope.to_utf16(&result)?),
                "ok",
                "{tag}"
            );
            Ok(())
        })
        .unwrap();
    }
}
