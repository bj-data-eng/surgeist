use super::{evaluate, hosted_session_with_dom_limits};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn iterator_tracks_the_native_pointer_through_removal() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_node_iterators(8));
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const root = document.createElement('div');
                const left = root.appendChild(document.createTextNode('left'));
                const right = root.appendChild(document.createTextNode('right'));
                if (root.firstChild !== left || left.nextSibling !== right) return 'native tree fixture';
                if (typeof document.createNodeIterator !== 'function') return 'missing native iterator factory';
                const cursor = document.createNodeIterator(root, NodeFilter.SHOW_TEXT);
                if (!(cursor instanceof NodeIterator) || cursor.root !== root || cursor.referenceNode !== root ||
                    !cursor.pointerBeforeReferenceNode || cursor.whatToShow !== 4 || cursor.filter !== null)
                    return 'native initial iterator state';
                if (cursor.nextNode() !== left || cursor.referenceNode !== left || cursor.pointerBeforeReferenceNode)
                    return 'first matching text and cursor direction';
                root.removeChild(left);
                if (cursor.referenceNode !== root || cursor.pointerBeforeReferenceNode || cursor.nextNode() !== right)
                    return 'native pre-remove cursor adjustment';
                if (cursor.previousNode() !== right || !cursor.pointerBeforeReferenceNode ||
                    cursor.previousNode() !== null || cursor.referenceNode !== right)
                    return 'backward traversal and exhausted cursor';
                return cursor.detach() === undefined && cursor.nextNode() === right ? 'ok' : 'detach no-op';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
