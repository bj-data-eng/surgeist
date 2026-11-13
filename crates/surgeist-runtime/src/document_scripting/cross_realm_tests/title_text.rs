use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, exhaust_guest_reaction_work, projected_node_id,
};

pub(super) fn title_text_uses_direct_native_children_and_original_replacement() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_ranges(4).unwrap());
    let mut host = attach_public_nodes(session, target, cancel.clone(), 8_000, 80_000);
    let permit = host.permit().unwrap();
    let (title, text) = host.in_document(&permit, |scope| {
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
        let title = evaluate(scope, "liveTitle")?;
        Ok((projected_node_id(scope, &title)?, projected_node_id(scope, &text)?))
    }).unwrap();
    let range = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            batch.replace_data(text, 0, usize::MAX, "native");
            let range = batch.create_range(target.document());
            batch.set_range_start(&range, text, 2);
            batch.set_range_end(&range, text, 5);
            dom.apply(batch, &cancel)
                .unwrap()
                .created_range(&range)
                .unwrap()
                .unwrap()
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
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "(liveTitle.text = 'replacement', liveTitle.text)")?;
        assert_eq!(
            scope.to_utf16(&result)?,
            "replacement".encode_utf16().collect::<Vec<_>>()
        );
        Ok(())
    })
    .unwrap();
    let snapshot = host.snapshot().unwrap();
    let range = snapshot.range(range).unwrap();
    assert_eq!(
        (range.start().container(), range.start().offset()),
        (title, 0)
    );
    assert_eq!((range.end().container(), range.end().offset()), (title, 0));
    let before = snapshot.stamp();
    exhaust_guest_reaction_work(&host);
    let result = host.in_document(&permit, |scope| evaluate(scope, "liveTitle.text").map(drop));
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("the title getter must respect the native work budget: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::WorkLimit)
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
}
