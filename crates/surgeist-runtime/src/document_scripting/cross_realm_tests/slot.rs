use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

pub(super) fn slot_members_read_the_canonical_named_assignment() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const slot = document.createElement('slot');
                if (!(slot instanceof HTMLSlotElement)) return 'slot interface';
                const members = ['name', 'assignedNodes', 'assignedElements', 'assign'];
                if (Object.keys(HTMLSlotElement.prototype).join() !== members.join())
                    return 'slot declaration order';
                const name = Object.getOwnPropertyDescriptor(HTMLSlotElement.prototype, 'name');
                if (!name?.get || !name?.set || !name.enumerable || !name.configurable ||
                    name.get.length !== 0 || name.set.length !== 1 || slot.name !== '')
                    return 'slot name descriptor';
                for (const member of members.slice(1)) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLSlotElement.prototype, member);
                    if (typeof descriptor?.value !== 'function' || !descriptor.enumerable ||
                        !descriptor.configurable || !descriptor.writable || descriptor.value.length !== 0)
                        return 'slot method descriptor ' + member;
                }
                const host = document.createElement('div');
                const root = host.attachShadow({mode: 'open'});
                root.appendChild(slot);
                const element = host.appendChild(document.createElement('span'));
                const text = host.appendChild(document.createTextNode('text'));
                host.appendChild(document.createComment('excluded'));
                const first = slot.assignedNodes();
                if (!Array.isArray(first) || Object.getPrototypeOf(first) !== Array.prototype ||
                    first.length !== 2 || first[0] !== element || first[1] !== text ||
                    slot.assignedNodes() === first || slot.assignedElements().length !== 1 ||
                    slot.assignedElements()[0] !== element || element.assignedSlot !== slot ||
                    text.assignedSlot !== slot) return 'canonical named assignment';
                slot.name = 'chosen';
                if (slot.getAttribute('name') !== 'chosen' || slot.assignedNodes().length !== 0 ||
                    element.assignedSlot !== null || text.assignedSlot !== null || first.length !== 2)
                    return 'live name reflection';
                element.setAttribute('slot', 'chosen');
                const assigned = slot.assignedNodes({flatten: false});
                const flattened = slot.assignedNodes({flatten: true});
                return assigned.length === 1 && assigned[0] === element &&
                    flattened.length === 1 && flattened[0] === element &&
                    element.assignedSlot === slot && text.assignedSlot === null ? 'ok' : 'fresh assignment';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
