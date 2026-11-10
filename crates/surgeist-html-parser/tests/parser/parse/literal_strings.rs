//! The constant strings of a parse: the namespace URIs of the elements it creates.

use crate::parser::support::limits;
use surgeist_html::{construction::HtmlNodeKind, values::DocumentString};
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlSource};

const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";
const MATHML_NAMESPACE: &str = "http://www.w3.org/1998/Math/MathML";

/// The namespace URI of every element of the parsed document, in document order.
fn namespaces(raw: &str) -> Vec<DocumentString> {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw.to_owned()), limits())
        .parse()
        .unwrap_or_else(|_| panic!("the document parses: {raw:.80}"));
    let document = report.document().unwrap();
    let mut found = Vec::new();
    let mut pending: Vec<_> = document.root().children().collect();
    pending.reverse();
    while let Some(id) = pending.pop() {
        let node = document.node(&id).unwrap();
        if let HtmlNodeKind::Element(element) = node.kind() {
            found.push(element.name().namespace().uri().unwrap().clone());
        }
        let mut children: Vec<_> = node.children().collect();
        children.reverse();
        pending.extend(children);
    }
    found
}

/// Asserts that every namespace is `uri` and that all of them are one allocation.
fn assert_one_string(namespaces: &[DocumentString], uri: &str, count: usize) {
    assert_eq!(namespaces.len(), count);
    for namespace in namespaces {
        assert_eq!(namespace.to_utf8().unwrap(), uri);
        assert!(
            namespace.shares_storage_with(&namespaces[0]),
            "equal namespace strings of one parse are separate allocations"
        );
    }
}

#[test]
fn html_elements_of_a_parse_share_one_namespace_string() {
    // html, head, body, div, three p elements and a span.
    let namespaces = namespaces("<div><p>a</p><p>b</p><p>c</p><span></span></div>");
    assert_one_string(&namespaces, HTML_NAMESPACE, 8);
}

#[test]
fn svg_elements_of_a_parse_share_one_namespace_string() {
    // The svg element and its two children.
    let all = namespaces("<svg><circle></circle><rect></rect></svg>");
    let svg: Vec<_> = all
        .into_iter()
        .filter(|namespace| namespace.to_utf8().unwrap() == SVG_NAMESPACE)
        .collect();
    assert_one_string(&svg, SVG_NAMESPACE, 3);
}

#[test]
fn mathml_elements_of_a_parse_share_one_namespace_string() {
    // The math element and its two children.
    let all = namespaces("<math><mi>x</mi><mn>1</mn></math>");
    let math: Vec<_> = all
        .into_iter()
        .filter(|namespace| namespace.to_utf8().unwrap() == MATHML_NAMESPACE)
        .collect();
    assert_one_string(&math, MATHML_NAMESPACE, 3);
}
