use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

pub(super) fn template_content_is_the_live_owned_inert_fragment() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 80_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const getter = Object.getOwnPropertyDescriptor(HTMLTemplateElement.prototype, 'content')?.get;
                if (typeof getter !== 'function') return 'missing content';
                const first = document.createElement('template');
                const second = document.createElement('template');
                const fragment = first.content;
                if (!(first instanceof HTMLTemplateElement) ||
                    !(fragment instanceof DocumentFragment) || fragment !== first.content ||
                    fragment === second.content || fragment.ownerDocument === document ||
                    second.content.ownerDocument !== fragment.ownerDocument || fragment.parentNode !== null)
                    return 'canonical inert identity';
                for (const receiver of [Object.create(first), document.createElement('div'), new Proxy(first, {})]) {
                    try { getter.call(receiver); return 'invalid receiver'; }
                    catch (error) { if (!(error instanceof TypeError)) return error.name; }
                }
                const text = document.createTextNode('a\ud800');
                fragment.appendChild(text);
                const nested = fragment.appendChild(document.createElement('template'));
                nested.content.appendChild(document.createTextNode('nested'));
                if (first.firstChild !== null || fragment.firstChild !== text ||
                    text.ownerDocument !== fragment.ownerDocument ||
                    nested.content.ownerDocument !== fragment.ownerDocument)
                    return 'live contents';
                const shallow = first.cloneNode();
                const deep = first.cloneNode(true);
                if (shallow.content.firstChild !== null || deep.content === fragment ||
                    deep.content.firstChild.data !== 'a\ud800' ||
                    deep.content.lastChild.content.firstChild.data !== 'nested')
                    return 'clone contents';
                const destination = new Document();
                const imported = destination.importNode(first, true);
                if (imported.content === fragment || imported.content.ownerDocument === fragment.ownerDocument ||
                    imported.content.firstChild.data !== 'a\ud800') return 'import contents';
                const oldOwner = fragment.ownerDocument;
                if (destination.adoptNode(first) !== first || first.content !== fragment ||
                    fragment.ownerDocument === oldOwner || fragment.ownerDocument !== imported.content.ownerDocument ||
                    text.ownerDocument !== fragment.ownerDocument || nested.content.firstChild.data !== 'nested')
                    return 'adopt contents';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
