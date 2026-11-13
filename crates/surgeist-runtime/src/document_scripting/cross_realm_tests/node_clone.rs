use super::*;
use crate::document_scripting::guest::tests::{attach_public_nodes, projected_node_id};

pub(super) fn clone_node_returns_fresh_canonical_subtrees() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
    let (source, copied) = host
        .in_document(&permit, |scope| {
            let outcome = evaluate(
                scope,
                r#"
                (() => {
                    const clone = Node.prototype.cloneNode;
                    if (typeof clone !== 'function') return 'missing cloneNode';
                    const source = document.createElement('div');
                    const text = source.appendChild(new Text('A\ud800\0'));
                    const pi = source.appendChild(new ProcessingInstruction('target', 'a="1"'));
                    const shadow = source.attachShadow({mode: 'open'});
                    shadow.appendChild(new Text('unclonable'));
                    const shallow = source.cloneNode();
                    const copied = source.cloneNode(true);
                    if (shallow === source || shallow.firstChild !== null ||
                        copied === source || copied.parentNode !== null ||
                        copied.ownerDocument !== document ||
                        Object.getPrototypeOf(copied) !== Object.getPrototypeOf(source) ||
                        copied.shadowRoot !== null || copied.firstChild === text ||
                        copied.firstChild.data !== 'A\ud800\0' ||
                        copied.lastChild === pi || copied.lastChild.target !== 'target')
                        return 'ordinary clone';
                    try { clone.call(Object.create(source), true); return 'forged receiver'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'wrong brand error'; }
                    try { shadow.cloneNode(true); return 'copied ShadowRoot directly'; }
                    catch (error) { if (error.name !== 'NotSupportedError') return error.name; }
                    const implementation = document.implementation;
                    const type = implementation.createDocumentType('root', '', '');
                    const xml = implementation.createDocument(null, 'root', type);
                    const xmlCopy = xml.cloneNode(true);
                    if (!(xmlCopy instanceof XMLDocument) || xmlCopy === xml ||
                        xmlCopy.ownerDocument !== null || xmlCopy.doctype === type ||
                        xmlCopy.doctype.name !== 'root' ||
                        xmlCopy.documentElement === xml.documentElement ||
                        xmlCopy.documentElement.ownerDocument !== xmlCopy)
                        return 'Document clone';
                    Object.assign(globalThis, {cloneSource: source, cloneCopy: copied});
                    return 'ok';
                })()
            "#,
            )?;
            assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&outcome)?), "ok");
            let source = evaluate(scope, "cloneSource")?;
            let copied = evaluate(scope, "cloneCopy")?;
            Ok((
                projected_node_id(scope, &source)?,
                projected_node_id(scope, &copied)?,
            ))
        })
        .unwrap();
    let snapshot = host.snapshot().unwrap();
    let source = snapshot.node(source).unwrap();
    let copied = snapshot.node(copied).unwrap();
    assert_ne!(source.id(), copied.id());
    assert_eq!(copied.owner_document(), source.owner_document());
    assert_eq!(copied.parent(), None);
    assert_eq!(copied.children().len(), 2);
    assert_ne!(copied.children(), source.children());
    assert_eq!(
        snapshot
            .node(copied.children()[0])
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        &[0x41, 0xd800, 0]
    );
}
