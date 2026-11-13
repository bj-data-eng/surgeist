use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test,
};

pub(super) fn adjacent_element_uses_the_current_native_branch_and_original_operand() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 200_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const prototype = Element.prototype;
                const desc = Object.getOwnPropertyDescriptor(prototype, 'insertAdjacentElement');
                const keys = Object.keys(prototype);
                if (!desc || desc.value.name !== 'insertAdjacentElement' || desc.value.length !== 2 ||
                    !desc.enumerable || !desc.writable || !desc.configurable ||
                    keys.indexOf('customElementRegistry') >= keys.indexOf('insertAdjacentElement') ||
                    keys.indexOf('insertAdjacentElement') >= keys.indexOf('insertAdjacentText'))
                    return 'descriptor';
                const root = document.createElement('div');
                const a = document.createElement('span');
                const b = document.createElement('section');
                const c = document.createElement('p');
                root.appendChild(a);
                root.appendChild(b);
                root.appendChild(c);
                const throws = (run, name) => {
                    try { run(); } catch (error) { return error.name === name; }
                    return false;
                };
                let coerced = false;
                if (!throws(() => desc.value.call(new Proxy(a, {}), {
                    toString() { coerced = true; return 'beforeend'; }
                }, b), 'TypeError') || coerced ||
                    !throws(() => a.insertAdjacentElement('beforeend'), 'TypeError'))
                    return 'brand and required argument';
                const where = {toString() { coerced = true; return 'beforeend'; }};
                if (!throws(() => a.insertAdjacentElement(where, new Proxy(b, {})), 'TypeError') ||
                    !coerced) return 'ordered conversion';
                if (a.insertAdjacentElement('beforebegin', a) !== a || root.firstChild !== a ||
                    a.insertAdjacentElement('AFTEREND', a) !== a || a.nextSibling !== b)
                    return 'self insertion';
                if (!throws(() => a.insertAdjacentElement('afterbegin', a), 'HierarchyRequestError') ||
                    !throws(() => a.insertAdjacentElement('beforeend', root), 'HierarchyRequestError'))
                    return 'hierarchy';
                if (a.insertAdjacentElement('beforeend', b) !== b || a.lastChild !== b ||
                    a.insertAdjacentElement('afterbegin', c) !== c || a.firstChild !== c)
                    return 'inside positions';
                const detached = document.createElement('aside');
                if (detached.insertAdjacentElement('beforebegin', b) !== null || b.parentNode !== a ||
                    detached.insertAdjacentElement('afterend', c) !== null || c.parentNode !== a)
                    return 'detached outside branches';
                const parent = document.createElement('article');
                if (a.insertAdjacentElement({toString() {
                    parent.appendChild(a);
                    return 'beforebegin';
                }}, b) !== b || parent.firstChild !== b || b.nextSibling !== a)
                    return 'reentrant destination';
                const foreign = document.implementation.createHTMLDocument();
                const adopted = foreign.createElement('span');
                adopted.marker = 79;
                if (a.insertAdjacentElement('afterend', adopted) !== adopted ||
                    adopted.ownerDocument !== document || adopted.parentNode !== parent ||
                    adopted.marker !== 79) return 'adoption identity';
                if (!throws(() => a.insertAdjacentElement(' beforeend', b), 'SyntaxError') ||
                    !throws(() => a.insertAdjacentElement('\ud800', b), 'SyntaxError'))
                    return 'syntax';
                const calls = [];
                customElements.define('x-adjacent-reaction', class extends HTMLElement {
                    connectedCallback() { calls.push(this); }
                });
                const observed = document.createElement('x-adjacent-reaction');
                document.documentElement.appendChild(parent);
                if (a.insertAdjacentElement('beforebegin', observed) !== observed ||
                    calls.length !== 1 || calls[0] !== observed || observed.parentNode !== parent)
                    return 'reaction before return';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    host.stop().into_session().unwrap();
    adjacent_element_admits_each_return_branch_before_publication();
}

fn adjacent_element_admits_each_return_branch_before_publication() {
    for outputs in [0, 1, 3, 4] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let container = batch.create_element(target.document(), "div");
        let receiver = batch.create_element(target.document(), "span");
        let operand = batch.create_element(target.document(), "section");
        batch.insert_before(&receiver, &container, None);
        batch.insert_before(&operand, &container, None);
        let done = session.dom().apply(batch, &cancel).unwrap();
        let container = done.created(&container).unwrap().unwrap();
        let receiver = done.created(&receiver).unwrap().unwrap();
        let operand = done.created(&operand).unwrap().unwrap();
        let mut host =
            attach_public_nodes_with_source_output_limit(session, target, cancel, outputs);
        let permit = host.permit().unwrap();
        let detached = outputs < 2;
        expose_public_node_for_source_test(
            &mut host,
            &permit,
            if detached { container } else { receiver },
            "limitedAdjacent",
        );
        expose_public_node_for_source_test(&mut host, &permit, operand, "limitedOperand");
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| {
            let script = if detached {
                "limitedAdjacent.insertAdjacentElement('beforebegin', limitedOperand) === null"
            } else {
                "limitedAdjacent.insertAdjacentElement('beforeend', limitedOperand) === limitedOperand"
            };
            let actual = evaluate(scope, script)?;
            let expected = scope.primitive(Primitive::Boolean(true))?;
            assert!(scope.strict_equals(&actual, &expected)?);
            Ok(())
        });
        let after = host.snapshot().unwrap();
        if matches!(outputs, 0 | 3) {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!(
                    "one-short adjacent Element result must refuse before publication: {result:?}"
                );
            };
            assert!(matches!(
                failure.cause::<crate::DomSourceDriveError>(),
                Some(crate::DomSourceDriveError::Source(
                    crate::DomSourceRuntimeError::Dom(surgeist_dom::DomError::SourceOutputLimit)
                ))
            ));
            assert_eq!(after.stamp(), before.stamp());
            assert_eq!(after.node(operand).unwrap().parent(), Some(container));
        } else {
            result.unwrap();
            if detached {
                assert_eq!(after.stamp(), before.stamp());
                assert_eq!(after.node(operand).unwrap().parent(), Some(container));
            } else {
                assert_eq!(after.node(operand).unwrap().parent(), Some(receiver));
            }
        }
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}
