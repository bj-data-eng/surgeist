use super::{Error, PropertyKey, evaluate, hosted_session, hosted_session_with_dom_limits};
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};
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
                const empty = root.appendChild(xml.createCDATASection(''));
                const trailing = root.appendChild(xml.createTextNode('b'));
                if (!(cdata instanceof CDATASection) || !(cdata instanceof Text) || cdata.nodeType !== 4 ||
                    cdata.nodeName !== '#cdata-section' || cdata.ownerDocument !== xml || cdata.length !== 2 ||
                    cdata.data !== '\ud800Z' || cdata.nodeValue !== '\ud800Z' ||
                    cdata.wholeText !== 'a\ud800Zb' || root.textContent !== 'a\ud800Zb')
                    return 'canonical text identity and values';
                root.normalize();
                if (root.firstChild !== leading || leading.nextSibling !== cdata ||
                    cdata.nextSibling !== empty || empty.nextSibling !== trailing ||
                    trailing.data !== 'b') return 'normalize across and preserve empty CDATA';
                const clone = cdata.cloneNode();
                const imported = document.importNode(cdata);
                if (clone === cdata || !(clone instanceof CDATASection) || clone.data !== cdata.data ||
                    !(imported instanceof CDATASection) || imported.ownerDocument !== document ||
                    imported.data !== '\ud800Z') return 'clone and import preserve node kind';
                if (!throws(() => xml.appendChild(clone), 'HierarchyRequestError', DOMException) ||
                    clone.parentNode !== null) return 'CDATA cannot be a Document child';
                clone.data = ']]>';
                if (clone.data !== ']]>' || clone.cloneNode().data !== ']]>')
                    return 'data mutation and clone do not rerun the factory delimiter test';
                const split = cdata.splitText(1);
                if (!(split instanceof Text) || split instanceof CDATASection || split.nodeType !== 3 ||
                    split.data !== 'Z' || cdata.data !== '\ud800' || cdata.nextSibling !== split)
                    return 'splitText creates ordinary Text';
                const walk = xml.createTreeWalker(root, NodeFilter.SHOW_CDATA_SECTION);
                if (walk.nextNode() !== cdata || walk.nextNode() !== empty || walk.nextNode() !== null)
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

pub(super) fn borrowed_cdata_factory_preserves_creation_and_exception_realms() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(
        &mut host,
        &permit,
        origin.document().node(),
        "otherDocument",
    );
    let (section_type, exception_type) = {
        let engine = host.engine.as_mut().unwrap();
        let realm = engine
            .creations
            .find(origin.global(), || Ok::<_, Error>(()))
            .unwrap()
            .unwrap();
        engine
            .process
            .execution()
            .in_realm(realm, |scope| {
                let section_type = evaluate(scope, "CDATASection")?;
                let exception_type = evaluate(scope, "DOMException")?;
                Ok((scope.retain(&section_type), scope.retain(&exception_type)))
            })
            .unwrap()
    };
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        for (name, root) in [("OtherCDATASection", section_type), ("OtherDOMException", exception_type)] {
            let key: Vec<_> = name.encode_utf16().collect();
            let input = scope.import(&root)?;
            scope.set(&global, PropertyKey::String(&key), &input)?;
        }
        let result = evaluate(scope, r#"
            (() => {
                const localFactory = Document.prototype.createCDATASection;
                const otherFactory = otherDocument.createCDATASection;
                const localXml = document.implementation.createDocument(null, 'root');
                const otherXml = otherDocument.implementation.createDocument(null, 'root');
                const local = otherFactory.call(localXml, 'local');
                const other = localFactory.call(otherXml, 'other');
                if (Object.getPrototypeOf(local) !== CDATASection.prototype || local.ownerDocument !== localXml ||
                    Object.getPrototypeOf(other) !== OtherCDATASection.prototype || other.ownerDocument !== otherXml)
                    return 'factory outputs use the receiver document creation realm';
                try { otherFactory.call(localXml, ']]>'); return 'invalid delimiter accepted'; }
                catch (error) {
                    if (!(error instanceof OtherDOMException) || error instanceof DOMException ||
                        error.name !== 'InvalidCharacterError') return 'XML failure realm';
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
