//! The constant strings of a parse: the namespace URIs of the elements it creates.

use crate::parser::support::limits;
use std::collections::BTreeMap;
use surgeist_html::{
    construction::{HtmlDocumentMode, HtmlNode, HtmlNodeKind},
    values::{DocumentString, Namespace, OrderedAttributes, QualifiedName},
};
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlSource};
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCapturedIsValue,
    HtmlDocumentClass, HtmlFragmentContext, HtmlHostFactsRevision, HtmlScriptingStatus,
    HtmlTemplateDocumentRole,
};

const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";
const MATHML_NAMESPACE: &str = "http://www.w3.org/1998/Math/MathML";

/// Visits every node below `root` in document order.
fn visit<'a>(
    root: HtmlNode<'a>,
    lookup: &dyn Fn(&surgeist_html::construction::HtmlNodeId) -> HtmlNode<'a>,
    mut each: impl FnMut(HtmlNode<'a>),
) {
    let mut pending: Vec<_> = root.children().collect();
    pending.reverse();
    while let Some(id) = pending.pop() {
        let node = lookup(&id);
        each(node);
        let mut children: Vec<_> = node.children().collect();
        children.reverse();
        pending.extend(children);
    }
}

/// The namespace URI of every element of the parsed document, in document order.
fn namespaces(raw: &str) -> Vec<DocumentString> {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw.to_owned()), limits())
        .parse()
        .unwrap_or_else(|_| panic!("the document parses: {raw:.80}"));
    let document = report.document().unwrap();
    let mut found = Vec::new();
    visit(document.root(), &|id| document.node(id).unwrap(), |node| {
        if let HtmlNodeKind::Element(element) = node.kind() {
            found.push(element.name().namespace().uri().unwrap().clone());
        }
    });
    found
}

/// The element namespace URIs of the fragment parsed in an SVG context.
fn fragment_namespaces(raw: &str, context_namespace: &str) -> Vec<DocumentString> {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_document_template_facts(
            &document,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            surgeist_html_semantics::HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    let element = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::from_uri(context_namespace), None, "svg"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&element).unwrap();
    builder.establish_no_shadow_root(&element).unwrap();
    let capture = builder.finish().unwrap();
    let context: HtmlFragmentContext =
        HtmlFragmentContext::element(&capture, &capture.subject_id(&element).unwrap()).unwrap();
    let report = HtmlFragmentParseRequest::new(HtmlSource::utf8(raw.to_owned()), context, limits())
        .unwrap()
        .parse()
        .unwrap_or_else(|_| panic!("the fragment parses: {raw:.80}"));
    let fragment = report.fragment().unwrap();
    let mut found = Vec::new();
    visit(fragment.root(), &|id| fragment.node(id).unwrap(), |node| {
        if let HtmlNodeKind::Element(element) = node.kind() {
            found.push(element.name().namespace().uri().unwrap().clone());
        }
    });
    found
}

/// Asserts that all the namespaces are `uri` and one allocation, and that there are at least
/// `least` of them.
fn assert_shared(namespaces: &[DocumentString], uri: &str, least: usize) {
    assert!(namespaces.len() >= least, "{} namespaces", namespaces.len());
    assert_one_string(namespaces, uri, namespaces.len());
}

/// The namespace URIs and prefixes of every attribute of the parsed document, grouped by their
/// text.
fn attribute_strings(raw: &str) -> BTreeMap<String, Vec<DocumentString>> {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw.to_owned()), limits())
        .parse()
        .unwrap_or_else(|_| panic!("the document parses: {raw:.80}"));
    let document = report.document().unwrap();
    let mut found: BTreeMap<String, Vec<DocumentString>> = BTreeMap::new();
    visit(document.root(), &|id| document.node(id).unwrap(), |node| {
        if let HtmlNodeKind::Element(element) = node.kind() {
            for attribute in element.attributes().iter() {
                let name = attribute.name();
                for string in [name.namespace().uri(), name.prefix()]
                    .into_iter()
                    .flatten()
                {
                    found
                        .entry(string.to_utf8().unwrap())
                        .or_default()
                        .push(string.clone());
                }
            }
        }
    });
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

#[test]
fn elements_cloned_by_the_adoption_agency_share_the_namespace_string() {
    // The adoption agency clones the b element for the text after its end tag.
    let namespaces = namespaces("<b><p>x</b>y");
    assert_shared(&namespaces, HTML_NAMESPACE, 6);
}

#[test]
fn elements_cloned_by_reconstruction_share_the_namespace_string() {
    // The b element is reconstructed for the second paragraph.
    let namespaces = namespaces("<p><b>x</p><p>y");
    assert_shared(&namespaces, HTML_NAMESPACE, 7);
}

#[test]
fn elements_of_a_fragment_in_an_svg_context_share_the_context_namespace_string() {
    let namespaces = fragment_namespaces("<g></g><g></g><g></g>", SVG_NAMESPACE);
    assert_shared(&namespaces, SVG_NAMESPACE, 3);
}

#[test]
fn foreign_attributes_share_their_namespace_and_prefix_strings() {
    let strings = attribute_strings(
        "<svg><a xlink:href=x xml:lang=y xmlns:xlink=z></a><a xlink:href=u xml:space=v xmlns:xlink=w></a></svg>",
    );
    for (text, least) in [
        ("http://www.w3.org/1999/xlink", 2),
        ("http://www.w3.org/XML/1998/namespace", 2),
        ("http://www.w3.org/2000/xmlns/", 2),
        ("xlink", 2),
        ("xml", 2),
        ("xmlns", 2),
    ] {
        let found = &strings[text];
        assert!(found.len() >= least, "{text}: {} strings", found.len());
        for string in found {
            assert!(
                string.shares_storage_with(&found[0]),
                "{text} is more than one allocation"
            );
        }
    }
}

#[test]
fn elements_of_a_fragment_in_a_context_of_another_literal_namespace_share_its_string() {
    // The elements take the context's namespace, which no other rule names.
    let xlink = "http://www.w3.org/1999/xlink";
    let namespaces = fragment_namespaces("<x></x><x></x><x></x>", xlink);
    assert_shared(&namespaces, xlink, 3);
}

#[test]
fn elements_cloned_into_selectedcontent_share_the_namespace_string() {
    let namespaces =
        namespaces("<select><option><b>c</b></option><selectedcontent></selectedcontent></select>");
    assert_shared(&namespaces, HTML_NAMESPACE, 8);
}

#[test]
fn foreign_attributes_cloned_into_selectedcontent_share_their_strings() {
    let strings = attribute_strings(
        "<select><option><svg><a xlink:href=x></a></svg></option><selectedcontent></selectedcontent></select>",
    );
    for text in ["http://www.w3.org/1999/xlink", "xlink"] {
        let found = &strings[text];
        assert!(found.len() >= 2, "{text}: {} strings", found.len());
        for string in found {
            assert!(
                string.shares_storage_with(&found[0]),
                "{text} is more than one allocation"
            );
        }
    }
}
