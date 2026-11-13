use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test,
};

pub(super) fn adjacent_text_uses_the_original_compound_source_edit() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 200_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const descriptor = Object.getOwnPropertyDescriptor(Element.prototype, 'insertAdjacentText');
                if (!descriptor || descriptor.value.name !== 'insertAdjacentText' ||
                    descriptor.value.length !== 2 || !descriptor.writable ||
                    !descriptor.enumerable || !descriptor.configurable) return 'descriptor';
                const root = document.createElement('div');
                const target = document.createElement('span');
                const sentinel = document.createTextNode('sentinel');
                root.appendChild(target);
                root.appendChild(sentinel);
                const throws = (run, name) => {
                    try { run(); } catch (error) { return error.name === name; }
                    return false;
                };
                let coerced = false;
                if (!throws(() => descriptor.value.call(new Proxy(target, {}), {
                    toString() { coerced = true; return 'beforeend'; }
                }, ''), 'TypeError') || coerced ||
                    !throws(() => target.insertAdjacentText('beforeend'), 'TypeError'))
                    return 'brand and missing argument';
                if (target.insertAdjacentText('BeFoReBeGiN', 'before') !== undefined ||
                    target.insertAdjacentText('afterend', 'after') !== undefined ||
                    target.insertAdjacentText('afterbegin', '\ud800') !== undefined ||
                    target.insertAdjacentText('beforeend', '\udc00') !== undefined)
                    return 'return';
                if (root.firstChild.data !== 'before' || target.nextSibling.data !== 'after' ||
                    target.nextSibling.nextSibling !== sentinel ||
                    target.firstChild.data !== '\ud800' || target.lastChild.data !== '\udc00')
                    return 'native positions';
                target.insertAdjacentText('afterbegin', '');
                if (target.firstChild.data !== '' || !(target.firstChild instanceof Text) ||
                    target.firstChild.nextSibling.data !== '\ud800') return 'empty text';
                const outside = document.createElement('section');
                if (outside.insertAdjacentText('beforebegin', 'lost') !== undefined ||
                    outside.insertAdjacentText('afterend', 'lost') !== undefined ||
                    outside.firstChild !== null || outside.parentNode !== null) return 'detached';
                const order = [];
                const parent = document.createElement('article');
                target.insertAdjacentText({toString() {
                    order.push('where');
                    return 'afterend';
                }}, {toString() {
                    order.push('data');
                    parent.appendChild(target);
                    return '\ud800\u0000';
                }});
                if (order.join() !== 'where,data' || target.parentNode !== parent ||
                    parent.lastChild.data !== '\ud800\u0000') return 'conversion reentry';
                const failure = new Error('sentinel');
                try { target.insertAdjacentText('invalid', {toString() { throw failure; }}); return 'throw'; }
                catch (error) { if (error !== failure) return 'conversion error identity'; }
                if (!throws(() => target.insertAdjacentText(' beforeend', 'value'), 'SyntaxError') ||
                    !throws(() => document.documentElement.insertAdjacentText('beforebegin', 'value'),
                        'HierarchyRequestError')) return 'native exceptions';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    host.stop().into_session().unwrap();
    adjacent_text_admits_the_created_node_before_publication();
}

fn adjacent_text_admits_the_created_node_before_publication() {
    for outputs in [1, 2] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let element = batch.create_element(target.document(), "div");
        let element = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&element)
            .unwrap()
            .unwrap();
        let mut host =
            attach_public_nodes_with_source_output_limit(session, target, cancel, outputs);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, element, "limitedAdjacent");
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| {
            evaluate(
                scope,
                "limitedAdjacent.insertAdjacentText('beforeend', '\\ud800')",
            )
            .map(drop)
        });
        let after = host.snapshot().unwrap();
        if outputs == 1 {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short output budget must refuse before insertion: {result:?}");
            };
            assert!(matches!(
                failure.cause::<crate::DomSourceDriveError>(),
                Some(crate::DomSourceDriveError::Source(
                    crate::DomSourceRuntimeError::Dom(surgeist_dom::DomError::SourceOutputLimit)
                ))
            ));
            assert_eq!(after.stamp(), before.stamp());
            assert_eq!(after.nodes().count(), before.nodes().count());
            assert!(after.node(element).unwrap().children().is_empty());
        } else {
            result.unwrap();
            let children = after.node(element).unwrap().children();
            assert_eq!(children.len(), 1);
            assert_eq!(
                after
                    .node(children[0])
                    .unwrap()
                    .data()
                    .unwrap()
                    .code_units(),
                [0xd800]
            );
        }
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}
