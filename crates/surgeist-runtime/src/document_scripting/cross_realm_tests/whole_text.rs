use super::*;
use crate::document_scripting::guest::tests::{attach_public_nodes, projected_node_id};

pub(super) fn whole_text_reads_the_current_contiguous_native_run() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 5_000, 60_000);
    let permit = host.permit().unwrap();
    let (left, barrier) = host
        .in_document(&permit, |scope| {
            let result = evaluate(scope, r#"
                (() => {
                    const getter = Object.getOwnPropertyDescriptor(Text.prototype, 'wholeText')?.get;
                    if (typeof getter !== 'function') return 'missing wholeText';
                    const parent = document.createElement('div');
                    const left = parent.appendChild(new Text('A\ud800'));
                    const empty = parent.appendChild(new Text(''));
                    const subject = parent.appendChild(new Text('\udc00B'));
                    const barrier = parent.appendChild(new Comment('separator'));
                    const after = parent.appendChild(new Text('C'));
                    if ([left, empty, subject].some(node => node.wholeText !== 'A\ud800\udc00B') ||
                        after.wholeText !== 'C' || new Text('detached\ud800').wholeText !== 'detached\ud800')
                        return 'ordinary run';
                    try { getter.call(barrier); return 'wrong brand'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'wrong error'; }
                    const shadow = parent.attachShadow({mode: 'open'});
                    const shadowText = shadow.appendChild(new Text('shadow'));
                    if (shadowText.wholeText !== 'shadow' || subject.wholeText !== 'A\ud800\udc00B')
                        return 'shadow boundary';
                    Object.assign(globalThis, {wholeLeft: left, wholeBarrier: barrier, wholeSubject: subject, wholeAfter: after});
                    return 'ok';
                })()
            "#)?;
            assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
            let left = evaluate(scope, "wholeLeft")?;
            let barrier = evaluate(scope, "wholeBarrier")?;
            Ok((projected_node_id(scope, &left)?, projected_node_id(scope, &barrier)?))
        })
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.replace_data(left, 0, usize::MAX, "live");
        batch.detach(barrier);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "wholeSubject.wholeText === 'live\\udc00BC' && wholeAfter.wholeText === wholeLeft.wholeText")?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
}
