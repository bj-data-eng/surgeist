use super::*;
use crate::document_scripting::guest::tests::{attach_public_nodes, projected_node_id};

pub(super) fn title_text_uses_direct_native_children_and_original_replacement() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 8_000, 80_000);
    let permit = host.permit().unwrap();
    let text = host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const descriptor = Object.getOwnPropertyDescriptor(HTMLTitleElement.prototype, 'text');
                if (!descriptor?.get || !descriptor?.set) return 'missing title text';
                const title = document.createElement('title');
                if (title.text !== '') return 'empty title';
                const nested = title.appendChild(document.createElement('span'));
                nested.appendChild(new Text('descendant'));
                title.appendChild(new Comment('comment'));
                title.insertBefore(new Text('a\ud800'), nested);
                title.appendChild(new Text('b\udc00'));
                if (title.text !== 'a\ud800b\udc00' || title.textContent !== 'a\ud800descendantb\udc00')
                    return 'direct children';
                for (const receiver of [Object.create(title), document.createElement('div'), new Proxy(title, {})]) {
                    let coerced = false;
                    try {
                        descriptor.set.call(receiver, {toString() { coerced = true; return 'x'; }});
                        return 'invalid setter brand';
                    } catch (error) {
                        if (coerced || !(error instanceof TypeError)) return 'setter brand order';
                    }
                    try { descriptor.get.call(receiver); return 'invalid getter brand'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'getter error'; }
                }
                const added = new Text('reentry');
                title.text = {toString() { title.appendChild(added); return 'changed\ud800'; }};
                if (title.text !== 'changed\ud800' || title.firstChild !== title.lastChild ||
                    added.parentNode !== null || nested.parentNode !== null)
                    return 'original replace all';
                title.text = '';
                if (title.text !== '' || title.hasChildNodes()) return 'empty replacement';
                title.text = null;
                if (title.text !== 'null') return 'DOMString null';
                globalThis.liveTitle = title;
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        let text = evaluate(scope, "liveTitle.firstChild")?;
        projected_node_id(scope, &text)
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.replace_data(text, 0, usize::MAX, "native");
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "liveTitle.text === 'native'")?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
}
