use super::{evaluate, hosted_session};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::sync::{Arc, atomic::AtomicBool};

// HTML877d 58387..58393 declares disabled Boolean and label DOMString
// reflections. A detached group reaches no ancestor-select reset operation.
pub(super) fn optgroup_members_reflect_their_own_native_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const group = document.createElement('optgroup');
                if (!(group instanceof HTMLOptGroupElement) || group.localName !== 'optgroup')
                    return 'native optgroup interface';
                const parent = document.body;
                if (parent.appendChild(group) !== group || group.parentNode !== parent)
                    return 'native optgroup body parent';
                parent.removeChild(group);
                if (group.parentNode !== null) return 'detached optgroup fixture';
                for (const member of ['disabled', 'label']) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLOptGroupElement.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function')
                        return 'optgroup missing reflected member ' + member;
                    if (!descriptor.enumerable || !descriptor.configurable || descriptor.get.length !== 0 ||
                        descriptor.set.length !== 1 || descriptor.get.name !== 'get ' + member ||
                        descriptor.set.name !== 'set ' + member) return 'optgroup accessor descriptor ' + member;
                }
                if (Object.keys(HTMLOptGroupElement.prototype).join() !== 'disabled,label')
                    return 'optgroup declaration order';
                if (group.disabled !== false || group.label !== '') return 'optgroup absent defaults';
                group.disabled = true;
                if (group.disabled !== true || group.getAttributeNS(null, 'disabled') !== '')
                    return 'own disabled presence';
                group.setAttributeNS(null, 'disabled', 'false');
                if (!group.disabled) return 'false-looking disabled text is present';
                group.disabled = false;
                if (group.disabled !== false || group.hasAttributeNS(null, 'disabled'))
                    return 'disabled false removes own attribute';
                group.label = null;
                if (group.label !== 'null' || group.getAttributeNS(null, 'label') !== 'null')
                    return 'label ordinary DOMString null';
                const raw = ' Raw\ud800\0\udc00 ';
                group.label = raw;
                return group.label === raw && group.getAttributeNS(null, 'label') === raw &&
                    group.label.length === 8 && group.label.charCodeAt(4) === 0xd800 &&
                    group.label.charCodeAt(5) === 0 && group.label.charCodeAt(6) === 0xdc00
                    ? 'ok' : 'label exact raw UTF16';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}
