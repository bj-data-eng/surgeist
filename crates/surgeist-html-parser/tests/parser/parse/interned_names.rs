//! The names of a parse: a document of N elements, or N attributes, drawn from k distinct names
//! retains k name allocations, whatever N is.
//!
//! Each expectation is derived from the generated markup. The count of distinct local-name
//! allocations of the published tree is the count of distinct names: a name that the parse
//! repeats shares the one allocation of its first occurrence.

use crate::parser::support::limits_builder;
use std::fmt::Write as _;
use std::hash::{DefaultHasher, Hash, Hasher};
use surgeist_html::{
    construction::{HtmlNode, HtmlNodeKind},
    values::{DocumentString, Namespace, QualifiedName},
    vocabulary::{ElementKey, KnownHtmlElement, KnownSvgElement},
};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseResource, HtmlParseStop, HtmlSource,
};

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

/// Elements `x0` to `x7`, each inside the one before, so that a node limit refuses a creation
/// after the names of the earlier elements were interned.
fn nested_elements() -> String {
    (0..DISTINCT).fold(String::new(), |mut html, index| {
        let _ = write!(html, "<x{index}>");
        html
    })
}

#[test]
fn a_parse_refused_after_interning_names_releases_everything_it_held() {
    let source = nested_elements();
    // The least node limit that admits the document: every smaller limit refuses a creation.
    let admitted = (1_u64..100)
        .find(|nodes| {
            HtmlDocumentParseRequest::offline(
                HtmlSource::utf8(source.clone()),
                huge()
                    .into_builder()
                    .managed_nodes(*nodes)
                    .finish()
                    .unwrap(),
            )
            .parse()
            .is_ok()
        })
        .expect("a limit admits the document");
    // The document, the doctype-less root, html, head, body and the eight elements.
    assert!(admitted >= DISTINCT as u64 + 3, "{admitted}");
    // What remains after a refusal at each limit: a name the parse interned before the refusal
    // would leave its bytes here if they were not credited with the rest of the storage.
    let mut retained = Vec::new();
    for nodes in 1..admitted {
        let limits = huge().into_builder().managed_nodes(nodes).finish().unwrap();
        let failure = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source.clone()), limits)
            .parse()
            .expect_err("a limit below the admitted one refuses");
        assert_eq!(failure.cleanup_error(), None);
        let after = failure.after_cleanup().expect("cleanup evidence").clone();
        for resource in [
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::TokenAttributes,
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::ContextEffects,
            HtmlParseResource::OpenElements,
        ] {
            assert_eq!(
                after.resource(resource).current(),
                0,
                "{resource:?} after a refusal at {nodes} nodes"
            );
        }
        retained.push(
            after
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current(),
        );
    }
    assert!(retained.len() > DISTINCT);
    assert!(
        retained.iter().all(|bytes| *bytes == retained[0]),
        "capacity left after each refusal: {retained:?}"
    );
}

/// The names of the elements of the parsed document, in tree order.
fn parsed_names(raw: &str) -> Vec<QualifiedName> {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw.to_owned()), huge())
        .parse()
        .expect("the document parses");
    let document = report.document().expect("a document report");
    let mut found = Vec::new();
    visit(document.root(), &|id| document.node(id).unwrap(), |node| {
        if let HtmlNodeKind::Element(element) = node.kind() {
            found.push(element.name().clone());
        }
    });
    found
}

fn hash_of(name: &QualifiedName) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
}

fn named<'a>(names: &'a [QualifiedName], local: &str) -> &'a QualifiedName {
    let wanted: Vec<u16> = local.encode_utf16().collect();
    names
        .iter()
        .find(|name| name.local_name().code_units() == wanted)
        .unwrap_or_else(|| panic!("no {local} element"))
}

#[test]
fn a_name_equals_and_hashes_like_the_same_name_of_another_parse_and_a_fresh_name() {
    let first = parsed_names("<section><p>a</p></section>");
    let second = parsed_names("<p>b</p><section></section>");
    for local in ["section", "p"] {
        let (one, two) = (named(&first, local), named(&second, local));
        let fresh = QualifiedName::new(Namespace::html(), None, local);
        // Two parses keep the name in separate allocations.
        assert!(!one.local_name().shares_storage_with(two.local_name()));
        assert_eq!(one, two);
        assert_eq!(one, &fresh);
        assert_eq!(hash_of(one), hash_of(two));
        assert_eq!(hash_of(one), hash_of(&fresh));
    }
}

#[test]
fn the_names_of_parsed_elements_are_the_names_of_the_vocabulary() {
    let names = parsed_names(
        "<template></template><x-unknown></x-unknown><svg><desc></desc><foreignobject></foreignobject></svg>",
    );
    assert_eq!(
        KnownHtmlElement::from_name(named(&names, "template")),
        Some(KnownHtmlElement::Template)
    );
    assert_eq!(
        KnownHtmlElement::from_name(named(&names, "body")),
        Some(KnownHtmlElement::Body)
    );
    assert_eq!(
        KnownHtmlElement::from_name(named(&names, "x-unknown")),
        None
    );
    // The tag name adjustment of an SVG name is in the name: `foreignobject` is stored as
    // `foreignObject`.
    assert_eq!(
        ElementKey::from_name(named(&names, "foreignObject")).svg(),
        Some(KnownSvgElement::ForeignObject)
    );
    assert_eq!(
        ElementKey::from_name(named(&names, "desc")).svg(),
        Some(KnownSvgElement::Desc)
    );
}

#[test]
fn a_parse_refused_at_any_capacity_limit_releases_everything_it_held() {
    // The capacity limit is swept one byte at a time from 1 to the peak of the parse, so every
    // growth the parse records, among them those of the name table (the vector of entries, the
    // slots, each entry and each copied name), is the first one a limit refuses. No
    // allocator-fault hook of the table is reachable from this crate without a production
    // interface, and the sweep is the refusal of each admission point by the limit instead.
    let source = "<x0><x1><x2><x3>".to_owned();
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source.clone()), huge())
        .parse()
        .expect("the document parses");
    let peak = report
        .usage()
        .resource(HtmlParseResource::ManagedCapacityBytes)
        .high_water();
    drop(report);
    let mut retained = Vec::new();
    let mut refused_at = std::collections::BTreeSet::new();
    for cap in 1..peak {
        let limits = huge()
            .into_builder()
            .managed_capacity_bytes(cap)
            .finish()
            .unwrap();
        let failure = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source.clone()), limits)
            .parse()
            .expect_err("a limit below the peak refuses");
        assert_eq!(failure.cleanup_error(), None);
        if let Some(HtmlParseStop::LimitExceeded {
            requested_delta, ..
        }) = failure.first_stop()
        {
            refused_at.insert(requested_delta);
        }
        let after = failure.after_cleanup().expect("cleanup evidence");
        for resource in [
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::OpenElements,
        ] {
            assert_eq!(
                after.resource(resource).current(),
                0,
                "{resource:?} after a refusal at {cap} bytes"
            );
        }
        retained.push(
            after
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current(),
        );
    }
    assert!(
        retained.iter().all(|bytes| *bytes == retained[0]),
        "capacity left after a refusal depends on the limit"
    );
    // Among the refused growths are a copied two-unit name (2 x 2 + 24 bytes), an entry (80
    // bytes) and the first 16 slots (128 bytes).
    for bytes in [28, 80, 128] {
        assert!(
            refused_at.contains(&bytes),
            "no refusal of a {bytes} byte growth"
        );
    }
}
