//! The names of a parse: a document of N elements, or N attributes, drawn from k distinct names
//! retains k name allocations, whatever N is.
//!
//! Each expectation is derived from the generated markup. The count of distinct local-name
//! allocations of the published tree is the count of distinct names: a name that the parse
//! repeats shares the one allocation of its first occurrence.

use crate::parser::support::limits_builder;
use std::fmt::Write as _;
use surgeist_html::{
    construction::{HtmlNode, HtmlNodeKind},
    values::DocumentString,
};
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};

/// The distinct names each document draws its elements and attributes from.
const DISTINCT: usize = 8;
/// The sizes of the family: each is double the one before.
const SIZES: [usize; 4] = [1000, 2000, 4000, 8000];
/// The elements an element-free document creates besides the ones it writes: `html`, `head` and
/// `body` are implied by the first start tag in the body.
const IMPLIED: usize = 3;

/// Limits that none of the documents reaches.
fn huge() -> HtmlParseLimits {
    limits_builder()
        .source_units(1 << 30)
        .buffered_utf16_units(1 << 30)
        .token_attributes(1 << 30)
        .auxiliary_entries(1 << 30)
        .managed_nodes(1 << 30)
        .managed_edges(1 << 30)
        .managed_attributes(1 << 30)
        .managed_utf16_units(1 << 30)
        .tree_depth(1 << 30)
        .managed_capacity_bytes(1 << 40)
        .peak_transient_bytes(1 << 40)
        .atomic_layout_bytes(1 << 40)
        .work_credits(u64::MAX)
        .finish()
        .expect("the limits are consistent")
}

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

/// How many separate allocations hold the strings, counting equal content in one allocation once.
fn allocations(strings: &[DocumentString]) -> usize {
    let mut representatives: Vec<&DocumentString> = Vec::new();
    for string in strings {
        if !representatives
            .iter()
            .any(|seen| seen.shares_storage_with(string))
        {
            representatives.push(string);
        }
    }
    representatives.len()
}

/// The local names of the elements and of the attributes of the parsed document.
fn names(raw: String) -> (Vec<DocumentString>, Vec<DocumentString>) {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), huge())
        .parse()
        .expect("the document parses");
    let document = report.document().expect("a document report");
    let (mut elements, mut attributes) = (Vec::new(), Vec::new());
    visit(document.root(), &|id| document.node(id).unwrap(), |node| {
        if let HtmlNodeKind::Element(element) = node.kind() {
            elements.push(element.name().local_name().clone());
            for attribute in element.attributes().iter() {
                attributes.push(attribute.name().local_name().clone());
            }
        }
    });
    (elements, attributes)
}

/// `count` empty elements named `x0` to `x7` in turn.
fn elements_of_few_names(count: usize) -> String {
    let mut html = String::new();
    for index in 0..count {
        let name = index % DISTINCT;
        let _ = write!(html, "<x{name}></x{name}>");
    }
    html
}

/// Elements of one name, each with attributes `a0` to `a7`, at least `count` attributes in all.
fn attributes_of_few_names(count: usize) -> String {
    let mut html = String::new();
    for _ in 0..count / DISTINCT {
        html.push_str("<span");
        for name in 0..DISTINCT {
            let _ = write!(html, " a{name}=\"v\"");
        }
        html.push_str("></span>");
    }
    html
}

#[test]
fn elements_drawn_from_few_names_retain_one_allocation_per_name() {
    for size in SIZES {
        let (elements, _) = names(elements_of_few_names(size));
        // The written elements and the three implied ones.
        assert_eq!(elements.len(), size + IMPLIED, "elements at {size}");
        // The distinct names are `x0` to `x7`, `html`, `head` and `body`.
        assert_eq!(
            allocations(&elements),
            DISTINCT + IMPLIED,
            "element name allocations at {size} elements"
        );
    }
}

#[test]
fn attributes_drawn_from_few_names_retain_one_allocation_per_name() {
    for size in SIZES {
        let (elements, attributes) = names(attributes_of_few_names(size));
        assert_eq!(attributes.len(), size, "attributes at {size}");
        // The distinct attribute names are `a0` to `a7`.
        assert_eq!(
            allocations(&attributes),
            DISTINCT,
            "attribute name allocations at {size} attributes"
        );
        // The elements are `span`, `html`, `head` and `body`.
        assert_eq!(
            allocations(&elements),
            1 + IMPLIED,
            "element name allocations at {size} attributes"
        );
    }
}
