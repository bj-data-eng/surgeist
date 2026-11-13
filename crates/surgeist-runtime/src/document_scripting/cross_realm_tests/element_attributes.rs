use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, exhaust_guest_reaction_work, projected_node_id,
};

pub(super) fn element_attribute_methods_observe_and_remove_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 80_000);
    let permit = host.permit().unwrap();
    let (element, removed) = host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const prototype = Element.prototype;
                const methods = [['hasAttributes', 0], ['getAttributeNames', 0],
                    ['removeAttribute', 1], ['removeAttributeNS', 2],
                    ['hasAttribute', 1], ['hasAttributeNS', 2]];
                const keys = Object.keys(prototype);
                let last = -1;
                for (const [name, arity] of methods) {
                    const desc = Object.getOwnPropertyDescriptor(prototype, name);
                    const index = keys.indexOf(name);
                    if (!desc || index <= last || desc.value.name !== name || desc.value.length !== arity ||
                        !desc.enumerable || !desc.writable || !desc.configurable) return 'descriptor ' + name;
                    last = index;
                }
                const element = document.createElement('div');
                if (element.hasAttributes() || element.getAttributeNames().length !== 0) return 'empty';
                let coerced = false;
                for (const [name, arity] of methods) {
                    const method = prototype[name];
                    try { method.call(new Proxy(element, {}), {toString() { coerced = true; }}); return 'brand'; }
                    catch (error) { if (!(error instanceof TypeError) || coerced) return 'brand order'; }
                    if (arity) {
                        try { method.call(element); return 'missing argument'; }
                        catch (error) { if (!(error instanceof TypeError)) return 'missing error'; }
                    }
                }
                element.setAttributeNS('urn:one', 'p:Spelling', 'first');
                element.setAttribute('data-a', 'second');
                element.setAttributeNS('urn:two', 'p:Spelling', 'third');
                const names = element.getAttributeNames();
                if (!element.hasAttributes() || names.join() !== 'p:Spelling,data-a,p:Spelling' ||
                    names === element.getAttributeNames() || !Array.isArray(names) ||
                    Object.getPrototypeOf(names) !== Array.prototype) return 'native name order';
                names.length = 0;
                if (element.getAttributeNames().length !== 3 ||
                    !element.hasAttribute('DATA-A') || element.hasAttribute('P:SPELLING') ||
                    !element.hasAttributeNS('urn:one', 'Spelling') ||
                    element.hasAttributeNS('urn:one', 'spelling') ||
                    !element.hasAttributeNS('', 'data-a')) return 'qualified and expanded lookup';
                const order = [];
                const uri = {toString() { order.push('namespace'); return 'urn:one'; }};
                const name = {toString() {
                    order.push('name');
                    element.setAttributeNS('urn:one', 'p:reached', 'new');
                    return 'reached';
                }};
                if (!element.hasAttributeNS(uri, name) || order.join() !== 'namespace,name')
                    return 'lookup reentry';
                const removed = element.getAttributeNodeNS('urn:one', 'Spelling');
                const other = element.getAttributeNodeNS('urn:two', 'Spelling');
                if (element.removeAttributeNS('urn:one', 'Spelling') !== undefined ||
                    removed.ownerElement !== null || removed.value !== 'first' ||
                    other.ownerElement !== element || !element.hasAttributeNS('urn:two', 'Spelling'))
                    return 'expanded removal';
                element.removeAttribute({toString() {
                    element.setAttribute('data-a', 'nested');
                    return 'DATA-A';
                }});
                if (element.hasAttribute('data-a') || element.removeAttribute('invalid name') !== undefined)
                    return 'qualified removal';
                const svg = document.createElementNS('http://www.w3.org/2000/svg', 'g');
                svg.setAttributeNS(null, 'Spelling', 'foreign');
                svg.removeAttribute('SPELLING');
                if (!svg.hasAttribute('Spelling') || svg.hasAttribute('spelling'))
                    return 'foreign case';
                globalThis.attributeElement = element;
                globalThis.removedAttribute = removed;
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        let element = evaluate(scope, "attributeElement")?;
        let removed = evaluate(scope, "removedAttribute")?;
        Ok((projected_node_id(scope, &element)?, projected_node_id(scope, &removed)?))
    }).unwrap();
    let snapshot = host.snapshot().unwrap();
    assert!(
        snapshot
            .node(removed)
            .unwrap()
            .as_attr()
            .unwrap()
            .owner_element()
            .is_none()
    );
    assert_eq!(
        snapshot.node(element).unwrap().attributes().unwrap().len(),
        2
    );
    let before = snapshot.stamp();
    exhaust_guest_reaction_work(&host);
    let result = host.in_document(&permit, |scope| {
        evaluate(
            scope,
            "attributeElement.removeAttributeNS('urn:two', 'Spelling')",
        )
        .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("attribute removal must respect the native work budget: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::WorkLimit)
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
}
