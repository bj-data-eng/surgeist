use super::*;
use crate::DomSourceDriveError;
use crate::document_scripting::guest::tests::{attach_public_nodes, projected_node_id};

pub(super) fn normalize_keeps_native_text_and_range_identity() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_ranges(4).unwrap());
    let mut host = attach_public_nodes(session, target, cancel.clone(), 5_000, 60_000);
    let permit = host.permit().unwrap();
    let (parent, first, tail) = host
        .in_document(&permit, |scope| {
            let parent = evaluate(
                scope,
                r#"
            globalThis.parent = document.createElement('div');
            globalThis.empty = parent.appendChild(document.createTextNode(''));
            globalThis.first = parent.appendChild(document.createTextNode('A'));
            globalThis.tail = parent.appendChild(document.createTextNode('\ud800\0'));
            globalThis.separator = parent.appendChild(document.createComment('stop'));
            globalThis.after = parent.appendChild(document.createTextNode('B'));
            globalThis.shadow = parent.attachShadow({mode: 'open'});
            shadow.appendChild(document.createTextNode('X'));
            shadow.appendChild(document.createTextNode('Y'));
            parent
        "#,
            )?;
            let first = evaluate(scope, "first")?;
            let tail = evaluate(scope, "tail")?;
            Ok((
                projected_node_id(scope, &parent)?,
                projected_node_id(scope, &first)?,
                projected_node_id(scope, &tail)?,
            ))
        })
        .unwrap();
    let range = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let range = batch.create_range(target.document());
            batch.set_range_start(&range, tail, 0);
            batch.set_range_end(&range, tail, 2);
            dom.apply(batch, &cancel)
                .unwrap()
                .created_range(&range)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                if (typeof parent.normalize !== 'function') return 'missing normalize';
                try { Node.prototype.normalize.call(Object.create(parent)); return 'forged receiver'; }
                catch (error) { if (!(error instanceof TypeError)) return error.name; }
                if (first.normalize() !== undefined || first.data !== 'A') return 'Text receiver';
                if (parent.normalize() !== undefined) return 'completion';
                if (parent.firstChild !== first || first.data !== 'A\ud800\0' ||
                    first.nextSibling !== separator || separator.nextSibling !== after ||
                    empty.parentNode !== null || tail.parentNode !== null || tail.data !== '\ud800\0' ||
                    shadow.firstChild.data !== 'X' || shadow.lastChild.data !== 'Y') return 'native topology';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    let snapshot = host.snapshot().unwrap();
    let range = snapshot.range(range).unwrap();
    assert_eq!(
        (range.start().container(), range.start().offset()),
        (first, 1)
    );
    assert_eq!((range.end().container(), range.end().offset()), (first, 3));
    assert_eq!(snapshot.node(first).unwrap().parent(), Some(parent));
}

pub(super) fn normalize_preserves_typed_native_refusal() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 5_000, 0);
    let permit = host.permit().unwrap();
    let before = host.snapshot().unwrap();
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "document.normalize()").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("expected native refusal: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(_)))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
}
