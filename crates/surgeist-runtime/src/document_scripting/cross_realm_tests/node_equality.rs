use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

pub(super) fn structural_node_equality_reads_the_live_ordinary_tree() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 80_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const left = document.createElement('div');
                const right = document.createElement('div');
                const equals = Node.prototype.isEqualNode;
                if (typeof equals !== 'function' || equals.length !== 1) return 'missing isEqualNode';
                if (!left.isEqualNode(left) || !left.isEqualNode(right) ||
                    left.isSameNode(right) || left.isEqualNode(null) || left.isEqualNode(undefined))
                    return 'identity and nullable';
                for (const invoke of [
                    () => left.isEqualNode(),
                    () => left.isEqualNode(new Proxy(right, {})),
                    () => equals.call(Object.create(left), right),
                ]) {
                    try { invoke(); return 'invalid accepted'; }
                    catch (error) { if (!(error instanceof TypeError)) return error.name; }
                }
                left.setAttribute('a', '1');
                left.setAttribute('b', '\ud800');
                right.setAttribute('b', '\ud800');
                right.setAttribute('a', '1');
                const first = left.appendChild(new Text('left'));
                const next = left.appendChild(new Comment('\udc00'));
                const rightFirst = right.appendChild(new Text('left'));
                const rightNext = right.appendChild(new Comment('\udc00'));
                if (!left.isEqualNode(right) || !right.isEqualNode(left) ||
                    !first.isEqualNode(rightFirst) || first.isEqualNode(next))
                    return 'equal structures or wrong kinds';
                right.insertBefore(rightNext, rightFirst);
                if (left.isEqualNode(right)) return 'ordered children';
                right.appendChild(rightNext);
                rightNext.data = '\ud800';
                if (left.isEqualNode(right)) return 'exact UTF-16';
                rightNext.data = '\udc00';
                const shadow = left.attachShadow({mode: 'open'});
                shadow.appendChild(new Text('shadow'));
                if (!left.isEqualNode(right)) return 'ordinary children only';
                const uri = 'urn:test';
                const a = document.createAttributeNS(uri, 'a:label');
                const b = document.createAttributeNS(uri, 'b:label');
                a.value = 'value';
                b.value = 'value';
                if (!a.isEqualNode(b)) return 'attribute prefix';
                if (document.createElementNS(uri, 'a:label').isEqualNode(
                    document.createElementNS(uri, 'b:label'))) return 'element prefix';
                if (document.createProcessingInstruction('a', 'value').isEqualNode(
                    document.createProcessingInstruction('b', 'value'))) return 'PI target';
                const another = new Document();
                another.appendChild(another.importNode(right, true));
                const other = another.documentElement;
                if (!left.isEqualNode(other)) return 'other owner document';
                const xml = document.implementation.createDocument(null, '', null);
                if (xml.isEqualNode(new Document())) return 'different document interfaces';
                right.setAttribute('b', '\udc00');
                if (left.isEqualNode(right)) return 'live attributes';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
