use super::*;
use crate::document_scripting::guest::tests::{attach_public_nodes, projected_node_id};

pub(super) fn split_text_keeps_native_identity_and_range_boundaries() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_ranges(4).unwrap());
    let mut host = attach_public_nodes(session, target, cancel.clone(), 5_000, 60_000);
    let permit = host.permit().unwrap();
    let (parent, original) = host
        .in_document(&permit, |scope| {
            let original = evaluate(
                scope,
                r#"
            globalThis.parent = document.createElement('div');
            globalThis.original = parent.appendChild(document.createTextNode('A\ud800\udc00B'));
            globalThis.stop = parent.appendChild(document.createComment('stop'));
            original
        "#,
            )?;
            let parent = evaluate(scope, "parent")?;
            Ok((
                projected_node_id(scope, &parent)?,
                projected_node_id(scope, &original)?,
            ))
        })
        .unwrap();
    let (range, boundary) = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let range = batch.create_range(target.document());
            batch.set_range_start(&range, original, 2);
            batch.set_range_end(&range, original, 4);
            let boundary = batch.create_range(target.document());
            batch.set_range_start(&boundary, parent, 1);
            batch.set_range_end(&boundary, parent, 1);
            let commit = dom.apply(batch, &cancel).unwrap();
            (
                commit.created_range(&range).unwrap().unwrap(),
                commit.created_range(&boundary).unwrap().unwrap(),
            )
        })
        .unwrap();
    let tail = host
        .in_document(&permit, |scope| {
            let result = evaluate(
                scope,
                r#"
            (() => {
                const split = Text.prototype.splitText;
                if (typeof split !== 'function') return 'missing splitText';
                const fails = (call, name) => {
                    try { call(); } catch (error) { return error.name === name; }
                    return false;
                };
                let coerced = 0;
                const offset = {valueOf() { ++coerced; return 2.75; }};
                if (!fails(() => split.call(stop, offset), 'TypeError') || coerced ||
                    !fails(() => original.splitText(), 'TypeError') ||
                    !fails(() => original.splitText(-1), 'IndexSizeError')) return 'validation';
                globalThis.tail = original.splitText(offset);
                if (coerced !== 1 || tail === original || !(tail instanceof Text) ||
                    tail.ownerDocument !== document || tail.parentNode !== parent ||
                    parent.firstChild !== original || original.nextSibling !== tail ||
                    tail.previousSibling !== original || tail.nextSibling !== stop ||
                    original.data !== 'A\ud800' || tail.data !== '\udc00B')
                    return 'attached split identity or data';
                const detached = new Text('abc');
                const detachedTail = detached.splitText({valueOf() {
                    detached.data = 'r\ud800';
                    return 0x1_0000_0001;
                }});
                if (detached.data !== 'r' || detachedTail.data !== '\ud800' ||
                    detachedTail.parentNode !== null || detachedTail.ownerDocument !== document)
                    return 'detached split after reentry';
                const sentinel = {};
                try { detached.splitText({valueOf() { throw sentinel; }}); return 'no throw'; }
                catch (error) { if (error !== sentinel) return 'lost coercion exception'; }
                return 'ok';
            })()
        "#,
            )?;
            assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
            let tail = evaluate(scope, "tail")?;
            projected_node_id(scope, &tail)
        })
        .unwrap();
    let snapshot = host.snapshot().unwrap();
    let range = snapshot.range(range).unwrap();
    assert_eq!(
        (range.start().container(), range.start().offset()),
        (original, 2)
    );
    assert_eq!((range.end().container(), range.end().offset()), (tail, 2));
    let boundary = snapshot.range(boundary).unwrap();
    assert_eq!(
        (boundary.start().container(), boundary.start().offset()),
        (parent, 2)
    );
    assert_eq!(
        (boundary.end().container(), boundary.end().offset()),
        (parent, 2)
    );
}
