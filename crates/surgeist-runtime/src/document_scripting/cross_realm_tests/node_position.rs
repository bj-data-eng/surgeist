use super::*;
use crate::document_scripting::guest::tests::{attach_public_nodes, projected_node_id};

pub(super) fn document_position_uses_live_order_and_native_node_identity() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 8_000, 80_000);
    let permit = host.permit().unwrap();
    let (parent, right, left) = host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const position = Node.prototype.compareDocumentPosition;
                if (typeof position !== 'function' || position.length !== 1)
                    return 'missing compareDocumentPosition';
                for (const [key, value] of [
                    ['DISCONNECTED', 1], ['PRECEDING', 2], ['FOLLOWING', 4],
                    ['CONTAINS', 8], ['CONTAINED_BY', 16], ['IMPLEMENTATION_SPECIFIC', 32],
                ]) {
                    for (const source of [Node, Node.prototype]) {
                        const descriptor = Object.getOwnPropertyDescriptor(source, 'DOCUMENT_POSITION_' + key);
                        if (!descriptor || descriptor.value !== value || !descriptor.enumerable ||
                            descriptor.writable || descriptor.configurable) return 'constant ' + key;
                    }
                }
                const parent = document.createElement('div');
                const left = parent.appendChild(document.createElement('span'));
                const right = parent.appendChild(document.createElement('span'));
                const leaf = left.appendChild(new Text('leaf'));
                const attr = document.createAttribute('a');
                const later = document.createAttribute('z');
                parent.setAttributeNode(attr);
                parent.setAttributeNode(later);
                if (parent.compareDocumentPosition(parent) !== 0 ||
                    parent.compareDocumentPosition(leaf) !== 20 ||
                    leaf.compareDocumentPosition(parent) !== 10 ||
                    left.compareDocumentPosition(right) !== 4 ||
                    right.compareDocumentPosition(left) !== 2 ||
                    parent.compareDocumentPosition(attr) !== 20 ||
                    attr.compareDocumentPosition(parent) !== 10 ||
                    attr.compareDocumentPosition(later) !== 36 ||
                    later.compareDocumentPosition(attr) !== 34 ||
                    attr.compareDocumentPosition(left) !== 4 ||
                    left.compareDocumentPosition(attr) !== 2) return 'position flags';
                const disconnected = (a, b) => {
                    const order = a.compareDocumentPosition(b);
                    return (order === 35 || order === 37) &&
                        b.compareDocumentPosition(a) === (order ^ 6) &&
                        a.compareDocumentPosition(b) === order;
                };
                const shadow = parent.attachShadow({mode: 'open'});
                const hidden = shadow.appendChild(new Text('hidden'));
                if (!disconnected(parent, shadow) || !disconnected(leaf, hidden) ||
                    shadow.compareDocumentPosition(hidden) !== 20) return 'shadow scope';
                parent.removeAttributeNode(attr);
                if (!disconnected(parent, attr) || !disconnected(attr, later)) return 'detached Attr';
                const other = new Document();
                if (!disconnected(other, parent)) return 'detached document';
                for (const invoke of [
                    () => parent.compareDocumentPosition(),
                    () => parent.compareDocumentPosition(null),
                    () => parent.compareDocumentPosition(undefined),
                    () => parent.compareDocumentPosition(new Proxy(left, {})),
                    () => position.call(Object.create(parent), right),
                ]) {
                    try { invoke(); return 'invalid accepted'; }
                    catch (error) { if (!(error instanceof TypeError)) return error.name; }
                }
                Object.assign(globalThis, {positionParent: parent, positionLeft: left, positionRight: right});
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        let parent = evaluate(scope, "positionParent")?;
        let right = evaluate(scope, "positionRight")?;
        let left = evaluate(scope, "positionLeft")?;
        Ok((
            projected_node_id(scope, &parent)?,
            projected_node_id(scope, &right)?,
            projected_node_id(scope, &left)?,
        ))
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.insert_before(right, parent, Some(left.into()));
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "positionLeft.compareDocumentPosition(positionRight) === 2 && positionRight.compareDocumentPosition(positionLeft) === 4")?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
}
