use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

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
}
