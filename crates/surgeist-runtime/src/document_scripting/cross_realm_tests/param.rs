use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

// HTML877d's complete obsolete HTMLParamElement IDL and Reflect naming rule
// specify this order, ordinary DOMStrings, and valueType's valuetype attribute.
pub(super) fn param_members_reflect_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const param = document.createElement('param');
                if (!(param instanceof HTMLParamElement)) return 'param interface';
                if (Object.keys(HTMLParamElement.prototype).join() !== 'name,value,type,valueType')
                    return 'param missing reflected members';
                for (const [member, attribute] of [['name', 'name'], ['value', 'value'],
                    ['type', 'type'], ['valueType', 'valuetype']]) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLParamElement.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return 'param descriptor ' + member;
                    if (param[member] !== '') return 'param absent default ' + member;
                    param[member] = null;
                    if (param[member] !== 'null' || param.getAttributeNS(null, attribute) !== 'null')
                        return 'param plain DOMString reflection ' + member;
                }
                return param.hasAttributeNS(null, 'valueType') ? 'camelCase content attribute' : 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
