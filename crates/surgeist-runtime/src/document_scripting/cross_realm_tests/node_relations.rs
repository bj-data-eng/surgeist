use super::*;

pub(super) fn node_relations_follow_the_live_ordinary_tree() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host =
        super::super::guest::tests::attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const fragment = document.createDocumentFragment();
                const parent = document.createElement('div');
                const text = document.createTextNode('a');
                const element = document.createElement('span');
                const comment = document.createComment('b');
                const relations = ['firstChild', 'lastChild', 'previousSibling', 'nextSibling', 'parentElement'];
                for (const name of relations) {
                    const getter = Object.getOwnPropertyDescriptor(Node.prototype, name)?.get;
                    if (!getter) return 'missing ' + name;
                    try { getter.call(Object.create(parent)); return 'forged ' + name; }
                    catch (error) { if (!(error instanceof TypeError)) return name + ': ' + error.name; }
                }
                if (typeof Node.prototype.hasChildNodes !== 'function') return 'missing hasChildNodes';
                try { Node.prototype.hasChildNodes.call(Object.create(parent)); return 'forged method'; }
                catch (error) { if (!(error instanceof TypeError)) return 'method error'; }
                if (fragment.hasChildNodes() || fragment.firstChild !== null || fragment.lastChild !== null)
                    return 'empty fragment';
                fragment.appendChild(parent);
                parent.appendChild(text);
                parent.appendChild(element);
                parent.appendChild(comment);
                if (parent.parentElement !== null || parent.parentNode !== fragment ||
                    fragment.firstChild !== parent || parent.firstChild !== text ||
                    parent.lastChild !== comment || !parent.hasChildNodes() ||
                    text.previousSibling !== null || text.nextSibling !== element ||
                    element.previousSibling !== text || element.nextSibling !== comment ||
                    comment.nextSibling !== null || comment.parentElement !== parent)
                    return 'ordinary tree';
                parent.insertBefore(comment, text);
                if (parent.firstChild !== comment || parent.lastChild !== element ||
                    comment.nextSibling !== text || text.previousSibling !== comment ||
                    element.nextSibling !== null) return 'reorder';
                const other = new Document();
                if (other.adoptNode(text) !== text || text.ownerDocument !== other ||
                    text.parentElement !== null || text.previousSibling !== null ||
                    text.nextSibling !== null || comment.nextSibling !== element)
                    return 'adoption';
                const shadow = parent.attachShadow({mode: 'open'});
                shadow.appendChild(text);
                if (!shadow.hasChildNodes() || shadow.firstChild !== text || shadow.lastChild !== text ||
                    text.parentElement !== null || text.parentNode !== shadow ||
                    parent.firstChild !== comment || parent.lastChild !== element)
                    return 'shadow boundary';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
