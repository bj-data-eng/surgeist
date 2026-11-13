use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, exhaust_guest_reaction_work, projected_node_id,
};

pub(super) fn element_reflections_use_the_exact_native_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 8_000, 80_000);
    let permit = host.permit().unwrap();
    let (node, slot) = host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const prototype = Element.prototype;
                const members = ['id', 'className', 'slot'];
                const positions = members.map(member => Object.keys(prototype).indexOf(member));
                if (!(positions[0] >= 0 && positions[0] < positions[1] && positions[1] < positions[2]))
                    return 'missing or unordered Element reflections';
                const descriptor = Object.getOwnPropertyDescriptor(prototype, Symbol.unscopables);
                if (!descriptor || descriptor.writable || descriptor.enumerable || !descriptor.configurable ||
                    Object.getPrototypeOf(descriptor.value) !== null ||
                    Object.keys(descriptor.value).join() !== 'slot' || descriptor.value.slot !== true)
                    return 'unscopables';
                const name = Object.getOwnPropertyDescriptor(prototype, 'id');
                const element = document.createElement('div');
                const attrs = ['id', 'class', 'slot'];
                for (const receiver of [
                    element,
                    document.createElementNS('http://www.w3.org/2000/svg', 'g'),
                    document.createElementNS('http://www.w3.org/1998/Math/MathML', 'mi'),
                    document.createElementNS('urn:foreign', 'example'),
                ]) {
                    for (const [index, member] of members.entries()) {
                        const property = Object.getOwnPropertyDescriptor(prototype, member);
                        if (property.get.name !== 'get ' + member || property.set.name !== 'set ' + member ||
                            property.get.length !== 0 || property.set.length !== 1 ||
                            !property.enumerable || !property.configurable || property.get.call(receiver) !== '')
                            return 'descriptor ' + member;
                        const value = 'v\ud800\0' + index;
                        receiver.setAttributeNS('urn:attribute', attrs[index], 'foreign');
                        if (property.get.call(receiver) !== '') return 'foreign attribute';
                        property.set.call(receiver, {toString() {
                            receiver.setAttributeNS(null, attrs[index], 'nested');
                            return value;
                        }});
                        if (property.get.call(receiver) !== value ||
                            receiver.getAttributeNS(null, attrs[index]) !== value ||
                            receiver.getAttributeNS('urn:attribute', attrs[index]) !== 'foreign')
                            return 'native reflection ' + member;
                    }
                }
                for (const receiver of [new Text(), new Proxy(element, {}), Object.create(element)]) {
                    try { name.get.call(receiver); return 'getter receiver'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'getter error'; }
                    let coerced = false;
                    try { name.set.call(receiver, {toString() { coerced = true; }}); return 'setter receiver'; }
                    catch (error) { if (!(error instanceof TypeError) || coerced) return 'setter error order'; }
                }
                const host = document.createElement('div');
                const root = host.attachShadow({mode: 'open'});
                const slot = root.appendChild(document.createElement('slot'));
                slot.setAttribute('name', 'selected');
                host.appendChild(element);
                element.slot = 'selected';
                globalThis.reflectionSlot = slot;
                globalThis.reflectedElement = element;
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        let element = evaluate(scope, "reflectedElement")?;
        let slot = evaluate(scope, "reflectionSlot")?;
        Ok((projected_node_id(scope, &element)?, projected_node_id(scope, &slot)?))
    }).unwrap();
    let snapshot = host.snapshot().unwrap();
    assert_eq!(
        snapshot.node(node).unwrap().assigned_slot(&cancel).unwrap(),
        Some(slot)
    );
    assert_eq!(snapshot.slot(slot).unwrap().assigned_nodes(), &[node]);
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const element = reflectedElement;
                element.slot = null;
                if (element.slot !== 'null' || element.getAttributeNS(null, 'slot') !== 'null')
                    return 'DOMString null';
                if (Function('element', 'var slot = 42; with (element) { return slot; }')(element) !== 42)
                    return 'slot scopes';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    let snapshot = host.snapshot().unwrap();
    assert_eq!(
        snapshot.node(node).unwrap().assigned_slot(&cancel).unwrap(),
        None
    );
    assert!(snapshot.slot(slot).unwrap().assigned_nodes().is_empty());
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(node, Namespace::none(), "id", "native");
        batch.set_attribute_ns(node, Namespace::none(), "class", "edit");
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(
            scope,
            "reflectedElement.id === 'native' && reflectedElement.className === 'edit'",
        )?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    exhaust_guest_reaction_work(&host);
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "reflectedElement.slot = 'refused'").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("reflected Element mutation must respect native work: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::WorkLimit)
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
}
