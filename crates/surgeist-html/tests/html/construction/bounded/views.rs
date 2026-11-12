//! The build view reports the records a storage holds as borrowed code units. Every expectation
//! is built from the values handed to the commands, never read back from the owner.
use super::*;
use surgeist_html::{construction::BuildStr, values::Attribute};

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
fn leaf(owner: HtmlConstructionStorage, leaf: Make) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    create(owner, leaf)
}
fn units(value: BuildStr<'_>) -> Vec<u16> {
    value.code_units().to_vec()
}

#[test]
fn text_and_comment_views_carry_the_exact_units_and_their_length() {
    let units_in = vec![0x61, 0, 0xD800, 0xDC00, 0xDFFF];
    let (owner, text) = leaf(storage(), Make::Text(units_in.clone()));
    let (owner, comment) = leaf(owner, Make::Comment(Vec::new()));
    let ConstructionNodeKind::Text(shown) = owner.node(&text).unwrap().kind() else {
        panic!("expected text");
    };
    assert_eq!(shown.code_units().unwrap(), units_in);
    assert_eq!(shown.len().unwrap(), 5);
    assert!(!shown.is_empty().unwrap());
    let ConstructionNodeKind::Comment(shown) = owner.node(&comment).unwrap().kind() else {
        panic!("expected a comment");
    };
    assert_eq!(shown.code_units().unwrap(), []);
    assert_eq!(shown.len().unwrap(), 0);
    assert!(shown.is_empty().unwrap());
    // The view is `Copy` and compares by units.
    let first = owner.node(&text).unwrap().kind();
    let second = first;
    assert_eq!(first, second);
}

#[test]
fn instruction_and_doctype_views_keep_absent_and_empty_identifiers_apart() {
    let (owner, pi) = leaf(
        storage(),
        Make::ProcessingInstruction(utf16("tool"), utf16("a b")),
    );
    let (owner, absent) = leaf(owner, Make::Doctype(utf16("html"), None, None));
    let (owner, present) = leaf(
        owner,
        Make::Doctype(utf16("svg"), Some(Vec::new()), Some(utf16("sys"))),
    );
    let ConstructionNodeKind::ProcessingInstruction(shown) = owner.node(&pi).unwrap().kind() else {
        panic!("expected a processing instruction");
    };
    assert_eq!(
        units(shown.target()),
        "tool".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        units(shown.data()),
        "a b".encode_utf16().collect::<Vec<_>>()
    );
    let ConstructionNodeKind::Doctype(shown) = owner.node(&absent).unwrap().kind() else {
        panic!("expected a doctype");
    };
    assert_eq!(
        units(shown.name()),
        "html".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(shown.public_id(), None);
    assert_eq!(shown.system_id(), None);
    let ConstructionNodeKind::Doctype(shown) = owner.node(&present).unwrap().kind() else {
        panic!("expected a doctype");
    };
    assert_eq!(shown.public_id().map(units), Some(vec![]));
    assert_eq!(
        shown.system_id().map(units),
        Some("sys".encode_utf16().collect::<Vec<_>>())
    );
}

#[test]
fn element_document_and_fragment_views_report_the_created_records() {
    let attributes = OrderedAttributes::new([
        Attribute::new(QualifiedName::new(Namespace::none(), None, "b"), "1"),
        Attribute::new(QualifiedName::new(Namespace::none(), None, "a"), "2"),
    ])
    .unwrap();
    let name = QualifiedName::new(Namespace::html(), None, "section");
    let (owner, element) = create(
        storage(),
        Make::Element {
            name: name.clone(),
            attributes: attributes.clone(),
            link_flags: None,
        },
    );
    let (owner, document) = create(
        owner,
        Make::Document {
            mode: HtmlDocumentMode::Quirks,
        },
    );
    let (owner, fragment) = create(owner, Make::Fragment);
    let ConstructionNodeKind::Element(shown) = owner.node(&element).unwrap().kind() else {
        panic!("expected an element");
    };
    assert_eq!(shown.name(), &name);
    assert_eq!(shown.attributes().to_ordered(), attributes);
    assert!(
        shown
            .attributes()
            .iter()
            .map(|a| a.name().local_name().to_utf8().unwrap())
            .eq(["b", "a"].map(String::from))
    );
    assert_eq!(
        owner.node(&document).unwrap().kind(),
        ConstructionNodeKind::Document(HtmlDocumentMode::Quirks)
    );
    assert_eq!(
        owner.node(&fragment).unwrap().kind(),
        ConstructionNodeKind::Fragment
    );
}
