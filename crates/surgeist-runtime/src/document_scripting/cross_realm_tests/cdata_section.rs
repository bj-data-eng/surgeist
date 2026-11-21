use super::{evaluate, hosted_session_with_dom_limits};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::sync::{Arc, atomic::AtomicBool};

pub(super) fn cdata_factory_and_inherited_text_behavior() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_tree_walkers(2));
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const xml = document.implementation.createDocument(null, 'root', null);
                if (!(xml instanceof XMLDocument) || xml.documentElement.localName !== 'root')
                    return 'native XML document fixture';
                const create = Document.prototype.createCDATASection;
                if (typeof create !== 'function') return 'missing native CDATA factory';
                if (create.length !== 1 || create.name !== 'createCDATASection' ||
                    typeof CDATASection !== 'function' || CDATASection.length !== 0 ||
                    Object.getPrototypeOf(CDATASection.prototype) !== Text.prototype)
                    return 'standard CDATA interface and inheritance';
                function throws(action, name, Type) {
                    try { action(); return false; }
                    catch (error) { return error instanceof Type && error.name === name; }
                }
                if (!throws(() => new CDATASection(), 'TypeError', TypeError) ||
                    !throws(() => xml.createCDATASection(), 'TypeError', TypeError) ||
                    !throws(() => xml.createCDATASection(']]>'), 'InvalidCharacterError', DOMException) ||
                    !throws(() => document.createCDATASection(']]>'), 'NotSupportedError', DOMException))
                    return 'factory rejection and order';
                const abrupt = {};
                try { document.createCDATASection({toString() { throw abrupt; }}); return 'conversion skipped'; }
                catch (error) { if (error !== abrupt) return 'conversion must precede native rejection'; }
                const root = xml.documentElement;
                const leading = root.appendChild(xml.createTextNode('a'));
                const cdata = root.appendChild(xml.createCDATASection('\ud800Z'));
                const trailing = root.appendChild(xml.createTextNode('b'));
                if (!(cdata instanceof CDATASection) || !(cdata instanceof Text) || cdata.nodeType !== 4 ||
                    cdata.nodeName !== '#cdata-section' || cdata.ownerDocument !== xml || cdata.length !== 2 ||
                    cdata.data !== '\ud800Z' || cdata.nodeValue !== '\ud800Z' ||
                    cdata.wholeText !== 'a\ud800Zb' || root.textContent !== 'a\ud800Zb')
                    return 'canonical text identity and values';
                root.normalize();
                if (root.firstChild !== leading || leading.nextSibling !== cdata ||
                    cdata.nextSibling !== trailing || trailing.data !== 'b') return 'normalize across CDATA';
                const clone = cdata.cloneNode();
                const imported = document.importNode(cdata);
                if (clone === cdata || !(clone instanceof CDATASection) || clone.data !== cdata.data ||
                    !(imported instanceof CDATASection) || imported.ownerDocument !== document ||
                    imported.data !== '\ud800Z') return 'clone and import preserve node kind';
                const split = cdata.splitText(1);
                if (!(split instanceof Text) || split instanceof CDATASection || split.nodeType !== 3 ||
                    split.data !== 'Z' || cdata.data !== '\ud800' || cdata.nextSibling !== split)
                    return 'splitText creates ordinary Text';
                const walk = xml.createTreeWalker(root, NodeFilter.SHOW_CDATA_SECTION);
                if (walk.nextNode() !== cdata || walk.nextNode() !== null)
                    return 'correct CDATA traversal bit';
                if (document.adoptNode(cdata) !== cdata || cdata.ownerDocument !== document ||
                    cdata.parentNode !== null) return 'canonical adoption';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
