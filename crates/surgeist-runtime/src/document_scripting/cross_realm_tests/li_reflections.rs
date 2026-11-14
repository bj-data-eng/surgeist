use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

// HTML877d's modern HTMLLIElement and obsolete partial interface specify
// reflected signed-long value followed by ordinary DOMString type.
pub(super) fn li_members_reflect_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const item = document.createElement('li');
                if (!(item instanceof HTMLLIElement)) return 'li interface';
                if (Object.keys(HTMLLIElement.prototype).join() !== 'value,type')
                    return 'li missing reflected members';
                for (const member of ['value', 'type']) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLLIElement.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return 'li descriptor ' + member;
                }
                if (item.value !== 0 || item.type !== '') return 'li absent defaults';
                item.value = -7;
                if (item.value !== -7 || item.getAttributeNS(null, 'value') !== '-7')
                    return 'li canonical signed reflection';
                item.type = null;
                return item.type === 'null' && item.getAttributeNS(null, 'type') === 'null'
                    ? 'ok' : 'li ordinary DOMString reflection';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
