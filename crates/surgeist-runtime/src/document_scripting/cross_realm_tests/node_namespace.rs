use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

pub(super) fn namespace_lookups_follow_live_native_names_and_declarations() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const xmlns = 'http://www.w3.org/2000/xmlns/';
                const xml = 'http://www.w3.org/XML/1998/namespace';
                const doc = new Document();
                const root = doc.appendChild(doc.createElementNS('urn:root', 'r:root'));
                const child = root.appendChild(doc.createElementNS(null, 'child'));
                const text = child.appendChild(doc.createTextNode('data'));
                const fragment = doc.createDocumentFragment();
                const detached = doc.createAttribute('detached');
                for (const name of ['lookupPrefix', 'lookupNamespaceURI', 'isDefaultNamespace']) {
                    const method = Node.prototype[name];
                    if (typeof method !== 'function' || method.length !== 1) return 'missing ' + name;
                    let coerced = false;
                    try {
                        method.call(Object.create(child), {toString() { coerced = true; return 'r'; }});
                        return 'forged ' + name;
                    } catch (error) {
                        if (coerced || !(error instanceof TypeError)) return 'brand ' + name;
                    }
                    try { method.call(child); return 'omission ' + name; }
                    catch (error) { if (!(error instanceof TypeError)) return 'arity ' + name; }
                }
                root.setAttributeNS(xmlns, 'xmlns', 'urn:default');
                root.setAttributeNS(xmlns, 'xmlns:first', 'urn:shared');
                root.setAttributeNS(xmlns, 'xmlns:second', 'urn:shared');
                root.setAttributeNS(xmlns, 'xmlns:r', 'urn:declaration-loses');
                root.setAttributeNS(xmlns, 'xmlns:raw', 'urn:\ud800');
                if (doc.lookupNamespaceURI('xml') !== xml || text.lookupNamespaceURI('xmlns') !== xmlns ||
                    text.lookupNamespaceURI('r') !== 'urn:root' || doc.lookupPrefix('urn:root') !== 'r' ||
                    child.lookupPrefix('urn:shared') !== 'first' ||
                    child.lookupPrefix('urn:\ud800') !== 'raw' || child.lookupNamespaceURI('raw') !== 'urn:\ud800' ||
                    child.lookupNamespaceURI(undefined) !== 'urn:default' ||
                    child.lookupNamespaceURI('') !== 'urn:default' ||
                    !text.isDefaultNamespace('urn:default') || text.isDefaultNamespace('') ||
                    child.lookupPrefix(null) !== null || child.lookupPrefix('') !== null)
                    return 'names or declaration order';
                if (fragment.lookupNamespaceURI('xml') !== null ||
                    detached.lookupNamespaceURI('xml') !== null || !fragment.isDefaultNamespace(''))
                    return 'detached dispatch';
                const attr = root.getAttributeNodeNS(xmlns, 'first');
                if (attr.lookupNamespaceURI('r') !== 'urn:root') return 'owned attribute';
                root.removeAttributeNS(xmlns, 'first');
                if (attr.lookupNamespaceURI('r') !== null ||
                    child.lookupPrefix('urn:shared') !== 'second') return 'detached attribute';
                const changed = child.lookupNamespaceURI({
                    toString() {
                        child.setAttributeNS(xmlns, 'xmlns', '');
                        return '';
                    }
                });
                if (changed !== null || !child.isDefaultNamespace(null) ||
                    root.lookupNamespaceURI(null) !== 'urn:default') return 'empty override after coercion';
                const host = document.createElement('div');
                host.setAttributeNS(xmlns, 'xmlns:outside', 'urn:outside');
                const shadow = host.attachShadow({mode: 'open'});
                const inner = shadow.appendChild(document.createElementNS(null, 'inner'));
                if (shadow.lookupNamespaceURI('xml') !== null ||
                    inner.lookupNamespaceURI('outside') !== null ||
                    inner.lookupNamespaceURI('xml') !== xml) return 'shadow boundary';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
