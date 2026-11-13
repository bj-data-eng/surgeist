use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

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
                root.append(target, sentinel);
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
                    parent.append(target);
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
}
