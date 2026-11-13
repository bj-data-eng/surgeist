use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, exhaust_guest_reaction_work, projected_node_id,
};

pub(super) fn node_comparisons_use_live_ordinary_ancestry_and_exact_identity() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 5_000, 60_000);
    let permit = host.permit().unwrap();
    let (parent, child) = host
        .in_document(&permit, |scope| {
            let result = evaluate(scope, r#"
                (() => {
                    const parent = document.createElement('div');
                    const middle = parent.appendChild(document.createElement('span'));
                    const child = middle.appendChild(new Text('value'));
                    const distinct = new Text('value');
                    const shadow = parent.attachShadow({mode: 'open'});
                    const hidden = shadow.appendChild(new Text('value'));
                    const foreignDocument = new Document();
                    for (const name of ['contains', 'isSameNode']) {
                        const method = Node.prototype[name];
                        if (typeof method !== 'function' || method.name !== name || method.length !== 1)
                            return 'missing or malformed ' + name;
                        if (method.call(parent, null) !== false ||
                            method.call(parent, undefined) !== false ||
                            method.call(child, child) !== true || method.call(child, distinct) !== false)
                            return 'nullable or identity ' + name;
                        for (const invoke of [
                            () => method.call(parent),
                            () => method.call(parent, Object.create(child)),
                            () => method.call(Object.create(parent), child),
                            () => method.call(parent, new Proxy(child, {})),
                            () => method.call(parent, 1),
                        ]) {
                            try { invoke(); return 'accepted invalid ' + name; }
                            catch (error) { if (!(error instanceof TypeError)) return error.name; }
                        }
                    }
                    if (!parent.contains(child) || child.contains(parent) ||
                        parent.isSameNode(child) || parent.contains(shadow) || parent.contains(hidden) ||
                        !shadow.contains(hidden) || shadow.contains(parent) ||
                        document.contains(parent) || foreignDocument.contains(child) ||
                        document.isSameNode(foreignDocument)) return 'ordinary ancestry';
                    Object.assign(globalThis, {comparisonParent: parent, comparisonChild: child});
                    return 'ok';
                })()
            "#)?;
            assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
            let parent = evaluate(scope, "comparisonParent")?;
            let child = evaluate(scope, "comparisonChild")?;
            Ok((projected_node_id(scope, &parent)?, projected_node_id(scope, &child)?))
        })
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.detach(child);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "!comparisonParent.contains(comparisonChild) && comparisonChild.isSameNode(comparisonChild)")?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.insert_before(child, parent, None);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "comparisonParent.contains(comparisonChild)")?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    exhaust_guest_reaction_work(&host);
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "comparisonParent.contains(comparisonChild)").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("containment must respect the native work budget: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::WorkLimit)
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
}
