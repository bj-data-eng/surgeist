use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

// HTML877d's HTMLDetailsElement IDL specifies name then open. This semantic
// fixture explicitly funds native transition intentions for the open changes.
pub(super) fn details_members_reflect_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session_with_details(&cancel, 0, 8);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const details = document.createElement('details');
                if (!(details instanceof HTMLDetailsElement)) return 'details interface';
                if (Object.keys(HTMLDetailsElement.prototype).join() !== 'name,open')
                    return 'details missing reflected members';
                for (const member of ['name', 'open']) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLDetailsElement.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return 'details descriptor ' + member;
                }
                if (details.name !== '' || details.open !== false) return 'details absent defaults';
                details.name = null;
                if (details.name !== 'null' || details.getAttributeNS(null, 'name') !== 'null')
                    return 'details ordinary name DOMString';
                details.open = true;
                if (details.open !== true || details.getAttributeNS(null, 'open') !== '')
                    return 'details Boolean presence';
                details.open = false;
                return details.open === false && !details.hasAttributeNS(null, 'open')
                    ? 'ok' : 'details Boolean removal';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
