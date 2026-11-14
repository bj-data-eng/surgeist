use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};

// HTML877d tree accessors select direct HTML children. Document direction
// reflects the html element; obsolete color properties reflect the body.
pub(super) fn document_tree_properties_select_and_forward_native_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let snapshot = session.snapshot();
    let html = snapshot
        .document(target.document())
        .unwrap()
        .document_element()
        .unwrap();
    let children = snapshot.node(html).unwrap().children();
    let head = children[0];
    let body = children[1];
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(html, Namespace::none(), "dir", "RtL");
    for (attribute, value) in [
        ("text", " raw text "),
        ("link", " raw link "),
        ("vlink", " raw vlink "),
        ("alink", " raw alink "),
        ("bgcolor", " raw bgcolor "),
    ] {
        batch.set_attribute_ns(body, Namespace::none(), attribute, value);
    }
    session.dom().apply(batch, &cancel).unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    for (node, name) in [
        (html, "expectedHtml"),
        (head, "expectedHead"),
        (body, "expectedBody"),
    ] {
        expose_public_node_for_source_test(&mut host, &permit, node, name);
    }
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                if (!(document instanceof Document) || document.documentElement !== expectedHtml ||
                    expectedHtml.localName !== 'html' || expectedHead.localName !== 'head' ||
                    expectedBody.localName !== 'body') return 'native document fixture';
                for (const member of ['head', 'body', 'dir', 'fgColor', 'linkColor', 'vlinkColor', 'alinkColor', 'bgColor']) {
                    const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, member);
                    if (typeof descriptor?.get !== 'function') return 'document missing tree accessors';
                    if (!descriptor.enumerable || !descriptor.configurable || descriptor.get.length !== 0 ||
                        descriptor.get.name !== 'get ' + member ||
                        (member === 'head' ? descriptor.set !== undefined :
                            typeof descriptor.set !== 'function' || descriptor.set.length !== 1 ||
                            descriptor.set.name !== 'set ' + member)) return 'document accessor descriptor ' + member;
                }
                if (document.head !== expectedHead || document.body !== expectedBody)
                    return 'native head and body identity';
                document.body = expectedBody;
                if (document.body !== expectedBody || expectedBody.parentNode !== expectedHtml)
                    return 'same body identity';
                if (document.dir !== 'rtl' || expectedHtml.getAttributeNS(null, 'dir') !== 'RtL')
                    return 'direction keyword and raw attribute';
                document.dir = 'AuTo';
                if (document.dir !== 'auto' || expectedHtml.getAttributeNS(null, 'dir') !== 'AuTo')
                    return 'direction setter stores raw text';
                document.dir = null;
                if (document.dir !== '' || expectedHtml.getAttributeNS(null, 'dir') !== 'null')
                    return 'direction ordinary DOMString null';
                for (const [member, attribute, raw] of [
                    ['fgColor', 'text', ' raw text '], ['linkColor', 'link', ' raw link '],
                    ['vlinkColor', 'vlink', ' raw vlink '], ['alinkColor', 'alink', ' raw alink '],
                    ['bgColor', 'bgcolor', ' raw bgcolor ']]) {
                    if (document[member] !== raw) return 'raw body color getter ' + member;
                    document[member] = null;
                    if (document[member] !== '' || expectedBody.getAttributeNS(null, attribute) !== '')
                        return 'color null to empty ' + member;
                    const value = 'Not-A-Color ' + member;
                    document[member] = value;
                    if (document[member] !== value || expectedBody.getAttributeNS(null, attribute) !== value ||
                        expectedHtml.hasAttributeNS(null, attribute)) return 'exact body color forwarding ' + member;
                }
                const empty = new Document();
                return empty.head === null && empty.body === null && empty.dir === '' &&
                    empty.fgColor === '' && empty.linkColor === '' && empty.vlinkColor === '' &&
                    empty.alinkColor === '' && empty.bgColor === '' ? 'ok' : 'empty document defaults';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
