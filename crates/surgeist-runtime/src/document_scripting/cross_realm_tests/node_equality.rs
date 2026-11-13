use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, exhaust_guest_reaction_work, expose_public_node_for_source_test,
};

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

pub(super) fn structural_node_equality_observes_template_boundaries_and_native_refusal() {
    use surgeist_html::values::Namespace;

    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let document = target.document();
    let mut batch = DomBatch::new();
    let left = batch.create_element_ns(document, Namespace::html(), "template");
    let right = batch.create_element_ns(document, Namespace::html(), "template");
    let obsolete = batch.create_text(document, "obsolete");
    let commit = session.dom().apply(batch, &cancel).unwrap();
    let left = commit.created(&left).unwrap().unwrap();
    let right = commit.created(&right).unwrap().unwrap();
    let obsolete = commit.created(&obsolete).unwrap().unwrap();
    let snapshot = session.snapshot();
    let contents = [left, right].map(|id| {
        snapshot
            .node(id)
            .unwrap()
            .as_template()
            .unwrap()
            .contents()
            .unwrap()
    });
    let inert = snapshot
        .node(contents[0])
        .unwrap()
        .owner_document()
        .unwrap();
    let mut batch = DomBatch::new();
    let text = batch.create_text(inert, "only left");
    batch.insert_before(&text, contents[0], None);
    session.dom().apply(batch, &cancel).unwrap();
    let mut host = attach_public_nodes(session, target, cancel.clone(), 8_000, 80_000);
    let permit = host.permit().unwrap();
    for (id, name) in [
        (left, "equalityTemplateLeft"),
        (right, "equalityTemplateRight"),
        (obsolete, "equalityObsolete"),
    ] {
        expose_public_node_for_source_test(&mut host, &permit, id, name);
    }
    host.in_document(&permit, |scope| {
        let result = evaluate(
            scope,
            "equalityTemplateLeft.isEqualNode(equalityTemplateRight)",
        )?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(obsolete);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    let retired = host.in_document(&permit, |scope| {
        evaluate(scope, "equalityTemplateLeft.isEqualNode(equalityObsolete)").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = retired else {
        panic!("retired equality operand must refuse: {retired:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    drop(host);
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 80_000);
    let permit = host.permit().unwrap();
    let before = host.snapshot().unwrap().stamp();
    exhaust_guest_reaction_work(&host);
    let exhausted = host.in_document(&permit, |scope| {
        evaluate(scope, "document.isEqualNode(document)").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = exhausted else {
        panic!("equality must respect the work limit: {exhausted:?}");
    };
    assert!(
        matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::WorkLimit)
        ),
        "{:?}",
        failure.cause::<DocumentGuestEventError>()
    );
    assert_eq!(host.snapshot().unwrap().stamp(), before);
}
