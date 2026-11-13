use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

pub(super) fn native_mutation_mixins_preserve_original_operands_and_conversion_order() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 12_000, 300_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const parents = ['prepend', 'append', 'replaceChildren'];
                const children = ['before', 'after', 'replaceWith', 'remove'];
                const throws = (run, name) => {
                    try { run(); } catch (error) { return error.name === name; }
                    return false;
                };
                for (const [prototype, methods] of [
                    [Document.prototype, parents], [DocumentFragment.prototype, parents],
                    [Element.prototype, parents.concat(children)],
                    [DocumentType.prototype, children], [CharacterData.prototype, children]
                ]) {
                    let last = -1;
                    const keys = Object.keys(prototype);
                    const unscopables = Object.getOwnPropertyDescriptor(prototype, Symbol.unscopables);
                    for (const name of methods) {
                        const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
                        const index = keys.indexOf(name);
                        if (!descriptor || descriptor.value.name !== name || descriptor.value.length !== 0 ||
                            !descriptor.enumerable || !descriptor.writable || !descriptor.configurable ||
                            index <= last) return 'descriptor ' + name;
                        last = index;
                        if (!unscopables || unscopables.writable || unscopables.enumerable ||
                            !unscopables.configurable || Object.getPrototypeOf(unscopables.value) !== null ||
                            unscopables.value[name] !== true) return 'unscopables ' + name;
                    }
                }
                if (Element.prototype[Symbol.unscopables].slot !== true ||
                    Object.hasOwn(Text.prototype, 'before') ||
                    Object.hasOwn(Comment.prototype, 'remove') ||
                    Object.hasOwn(ProcessingInstruction.prototype, 'replaceWith'))
                    return 'inherited mixins';
                const root = document.createElement('div');
                const a = document.createElement('a');
                const b = document.createElement('span');
                const c = document.createElement('section');
                let coerced = false;
                const text = {toString() {coerced = true; return 'text';}};
                if (!throws(() => Element.prototype.append.call(new Proxy(root, {}), text), 'TypeError') ||
                    coerced || !throws(() => Element.prototype.append.call(new DocumentFragment(), text), 'TypeError') ||
                    coerced || !throws(() => CharacterData.prototype.before.call(root, text), 'TypeError') ||
                    coerced) return 'receiver before conversion';
                if (root.append(a, b, c) !== undefined || root.firstChild !== a || a.nextSibling !== b ||
                    b.nextSibling !== c) return 'append operands';
                const fragment = new DocumentFragment();
                fragment.append('fragment', document.createComment('comment'));
                root.prepend(a, '\ud800', fragment, a);
                if (root.firstChild.data !== '\ud800' || root.firstChild.nextSibling.data !== 'fragment' ||
                    a.nextSibling !== b || b.nextSibling !== c || root.lastChild !== c ||
                    fragment.firstChild !== null) return 'duplicate conversion';
                const order = [];
                a.before({toString() {order.push(1); return 'first';}},
                    {toString() {order.push(2); b.remove(); return '\udc00';}});
                if (order.join() !== '1,2' || b.parentNode !== null ||
                    a.previousSibling.data !== '\udc00' || a.previousSibling.previousSibling.data !== 'first')
                    return 'conversion order';
                const prefix = document.createTextNode('prefix');
                a.after(prefix, a, prefix);
                if (a.nextSibling !== prefix || prefix.nextSibling !== c || root.lastChild !== c)
                    return 'viable sibling';
                a.replaceWith(a, 'replacement', a);
                if (a.previousSibling.data !== 'replacement' || a.nextSibling !== prefix)
                    return 'replaceWith duplicate';
                const failure = new Error('sentinel');
                const parent = a.parentNode;
                try { a.after('private', {toString() {throw failure;}}); return 'throw'; }
                catch (error) {if (error !== failure) return 'original exception';}
                if (a.parentNode !== parent || a.nextSibling !== prefix) return 'conversion before native';
                const detached = document.createElement('aside');
                detached.before({toString() {coerced = true; return 'ignored';}});
                if (!coerced || detached.parentNode !== null) return 'detached conversion';
                if (detached.remove({toString() {throw Error('extra coercion');}}) !== undefined)
                    return 'remove extra arguments';
                root.replaceChildren(null, undefined, 4, '\ud800\u0000');
                if (root.firstChild.data !== 'null' || root.firstChild.nextSibling.data !== 'undefined' ||
                    root.lastChild.data !== '\ud800\u0000' || a.parentNode !== null)
                    return 'replaceChildren strings';
                root.replaceChildren();
                if (root.firstChild !== null) return 'empty replacement';
                root.append();
                root.prepend();
                const comment = new Comment('comment');
                root.append(comment);
                comment.before('before');
                comment.after('after');
                comment.replaceWith('replaced');
                if (root.firstChild.data !== 'before' || root.lastChild.data !== 'after' ||
                    root.firstChild.nextSibling.data !== 'replaced' || comment.parentNode !== null)
                    return 'CharacterData mixin';
                const xml = new Document();
                const doctype = document.implementation.createDocumentType('root', '', '');
                xml.append(doctype);
                doctype.before(new Comment('before'));
                doctype.after(new Comment('after'));
                doctype.remove();
                if (xml.firstChild.data !== 'before' || xml.lastChild.data !== 'after' ||
                    doctype.parentNode !== null) return 'DocumentType mixin';
                if (!throws(() => xml.append('text'), 'HierarchyRequestError') ||
                    xml.firstChild.data !== 'before') return 'Document hierarchy';
                const observed = [];
                customElements.define('x-mixin-reaction', class extends HTMLElement {
                    connectedCallback() {observed.push('connect');}
                    disconnectedCallback() {observed.push('disconnect');}
                });
                const custom = document.createElement('x-mixin-reaction');
                document.documentElement.append(custom);
                custom.remove();
                if (observed.join() !== 'connect,disconnect') return 'native reactions';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
