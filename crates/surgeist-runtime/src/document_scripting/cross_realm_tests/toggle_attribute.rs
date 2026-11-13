use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

pub(super) fn element_toggle_attribute_uses_one_canonical_edit() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 200_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const prototype = Element.prototype;
                const descriptor = Object.getOwnPropertyDescriptor(prototype, 'toggleAttribute');
                const keys = Object.keys(prototype);
                if (!descriptor || descriptor.value.length !== 1 ||
                    descriptor.value.name !== 'toggleAttribute' || !descriptor.enumerable ||
                    !descriptor.configurable || !descriptor.writable ||
                    keys.indexOf('removeAttributeNS') >= keys.indexOf('toggleAttribute') ||
                    keys.indexOf('toggleAttribute') >= keys.indexOf('hasAttribute'))
                    return 'descriptor toggleAttribute';
                const element = document.createElement('div');
                const toggle = descriptor.value;
                const throws = (callback, name) => {
                    try { callback(); } catch (error) { return error.name === name; }
                    return false;
                };
                let coerced = false;
                if (!throws(() => toggle.call(new Proxy(element, {}), {
                    toString() { coerced = true; }
                }), 'TypeError') || coerced || !throws(() => toggle.call(element), 'TypeError'))
                    return 'brand and required argument';
                if (!throws(() => element.toggleAttribute('invalid name', false), 'InvalidCharacterError'))
                    return 'validation before absence';
                if (element.toggleAttribute('DATA-OPTIONAL', false) !== false ||
                    element.toggleAttribute('DATA-OPTIONAL', null) !== false ||
                    element.toggleAttribute('DATA-OPTIONAL', undefined) !== true ||
                    element.getAttribute('data-optional') !== '' ||
                    element.toggleAttribute('DATA-OPTIONAL') !== false ||
                    element.hasAttribute('data-optional')) return 'force conversion';
                const object = { valueOf() { throw Error('unexpected coercion'); } };
                if (element.toggleAttribute('data-object', object) !== true ||
                    element.toggleAttribute('data-object', object) !== true)
                    return 'object force';
                element.setAttributeNS('urn:first', 'p:x', 'first');
                element.setAttributeNS('urn:second', 'p:x', 'second');
                const first = element.getAttributeNodeNS('urn:first', 'x');
                const second = element.getAttributeNodeNS('urn:second', 'x');
                if (element.toggleAttribute('P:X', true) !== true ||
                    element.getAttributeNode('p:x') !== first ||
                    element.toggleAttribute('P:X') !== false ||
                    first.ownerElement !== null || first.value !== 'first' ||
                    element.getAttributeNode('p:x') !== second ||
                    element.toggleAttribute('p:x', false) !== false ||
                    element.toggleAttribute('p:x', true) !== true ||
                    element.getAttributeNode('p:x').namespaceURI !== null ||
                    element.getAttributeNode('p:x').prefix !== null)
                    return 'first qualified identity';
                const svg = document.createElementNS('http://www.w3.org/2000/svg', 'g');
                if (!svg.toggleAttribute('Mixed') || !svg.toggleAttribute('mixed') ||
                    svg.getAttributeNames().join() !== 'Mixed,mixed')
                    return 'foreign case';
                if (element.toggleAttribute({toString() {
                    element.setAttribute('data-reentry', 'nested');
                    return 'data-reentry';
                }}) !== false || element.hasAttribute('data-reentry')) return 'conversion reentry';
                const changes = [];
                customElements.define('x-toggle-observed', class extends HTMLElement {
                    static observedAttributes = ['data-watch'];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        changes.push([name, oldValue, newValue, namespace, this.hasAttribute(name)]);
                    }
                });
                const watched = document.createElement('x-toggle-observed');
                document.documentElement.appendChild(watched);
                if (watched.toggleAttribute('DATA-WATCH', false) !== false || changes.length !== 0 ||
                    !watched.toggleAttribute('DATA-WATCH') || changes.length !== 1 ||
                    watched.toggleAttribute('DATA-WATCH', true) !== true || changes.length !== 1 ||
                    watched.toggleAttribute('DATA-WATCH', false) !== false || changes.length !== 2)
                    return 'reaction count and timing';
                if (changes[0][0] !== 'data-watch' || changes[0][1] !== null ||
                    changes[0][2] !== '' || changes[0][3] !== null || !changes[0][4] ||
                    changes[1][0] !== 'data-watch' || changes[1][1] !== '' ||
                    changes[1][2] !== null || changes[1][3] !== null || changes[1][4])
                    return 'reaction state';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
