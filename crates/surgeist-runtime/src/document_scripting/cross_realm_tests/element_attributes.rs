use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, exhaust_guest_reaction_work, expose_public_node_for_source_test,
    projected_node_id,
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
                const ordered = ['slot', 'hasAttributes', 'getAttributeNames', 'getAttribute',
                    'getAttributeNS', 'setAttribute', 'setAttributeNS', 'removeAttribute',
                    'removeAttributeNS', 'hasAttribute', 'hasAttributeNS', 'getAttributeNode',
                    'getAttributeNodeNS', 'setAttributeNode', 'setAttributeNodeNS',
                    'removeAttributeNode', 'attachShadow', 'shadowRoot', 'customElementRegistry'];
                if (keys.filter(key => ordered.includes(key)).join() !== ordered.join())
                    return 'attribute declaration order';
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
                const namespace = 'urn:\ud800';
                const localName = 'data-\udc00';
                const qualifiedName = 'p:' + localName;
                element.setAttributeNS(namespace, qualifiedName, 'code units');
                if (!element.getAttributeNames().includes(qualifiedName) ||
                    !element.hasAttribute(qualifiedName) || !element.hasAttributeNS(namespace, localName) ||
                    element.hasAttributeNS('urn:\ufffd', localName) ||
                    element.hasAttributeNS(namespace, 'data-\ufffd')) return 'exact code units';
                element.removeAttributeNS(namespace, localName);
                if (element.hasAttribute(qualifiedName)) return 'exact code unit removal';

                const reactions = [];
                customElements.define('x-attribute-removal', class extends HTMLElement {
                    static observedAttributes = ['data-watched'];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        reactions.push([this, name, oldValue, newValue, namespace,
                            this.hasAttributeNS(namespace, name)]);
                    }
                });
                const watched = document.createElement('x-attribute-removal');
                document.documentElement.appendChild(watched);
                watched.setAttribute('data-watched', 'qualified');
                watched.setAttributeNS('urn:watched', 'p:data-watched', 'expanded');
                reactions.length = 0;
                watched.removeAttribute('DATA-WATCHED');
                watched.removeAttribute('DATA-WATCHED');
                if (reactions.length !== 1 || reactions[0][0] !== watched ||
                    reactions[0][1] !== 'data-watched' || reactions[0][2] !== 'qualified' ||
                    reactions[0][3] !== null || reactions[0][4] !== null || reactions[0][5])
                    return 'qualified removal reaction';
                watched.removeAttributeNS('urn:watched', 'data-watched');
                watched.removeAttributeNS('urn:watched', 'data-watched');
                if (reactions.length !== 2 || reactions[1][0] !== watched ||
                    reactions[1][1] !== 'data-watched' || reactions[1][2] !== 'expanded' ||
                    reactions[1][3] !== null || reactions[1][4] !== 'urn:watched' || reactions[1][5])
                    return 'expanded removal reaction';
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
    host.stop().into_session().unwrap();

    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let mut batch = DomBatch::new();
    let element = batch.create_element(target.document(), "div");
    batch.set_attribute(&element, "data-original", "original");
    let element = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&element)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 0);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, element, "refusedElement");
    let before = host.snapshot().unwrap();
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "refusedElement.removeAttribute('data-original')").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("attribute removal must respect the Source work limit: {result:?}");
    };
    assert!(matches!(
        failure.cause::<crate::DomSourceDriveError>(),
        Some(crate::DomSourceDriveError::Source(
            crate::DomSourceRuntimeError::Dom(surgeist_dom::DomError::SourceWorkLimit { limit: 0 })
        ))
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    let original = before.node(element).unwrap().attributes().unwrap();
    let current = after.node(element).unwrap().attributes().unwrap();
    assert!(
        current
            .iter()
            .map(|attr| (attr.id(), attr.value()))
            .eq(original.iter().map(|attr| (attr.id(), attr.value())))
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}
