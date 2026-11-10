//! The published node view reports the authored values exactly. Every expectation is built from
//! the values handed to the builder, never read back from the result.

use crate::html::support::construction_limits as limits;
use surgeist_html::{
    construction::{
        HtmlAuthoredProcessingInstruction, HtmlDoctype, HtmlDocument, HtmlDocumentMode,
        HtmlFragment, HtmlNodeKind, HtmlProcessingInstruction,
    },
    elements::link::HtmlLinkFlags,
    values::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName},
};

const XLINK: &str = "http://www.w3.org/1999/xlink";

fn plain(local: &str, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, local), value)
}
fn xlink(local: &str, value: &str) -> Attribute {
    Attribute::new(
        QualifiedName::new(Namespace::from_uri(XLINK), Some("xlink".into()), local),
        value,
    )
}
/// `id`, `xlink:href`, `href` and `class`, with a null unit and a lone surrogate in values.
fn authored() -> Vec<Attribute> {
    vec![
        plain("id", "a"),
        xlink("href", "far"),
        plain("href", "near"),
        plain("class", DocumentString::from_utf16(vec![0x63, 0, 0xD800])),
    ]
}
/// A fragment whose only child is an element with `attributes`.
fn one_element(name: QualifiedName, attributes: OrderedAttributes) -> HtmlFragment {
    let mut builder = HtmlFragment::builder(limits()).unwrap();
    let root = builder.root();
    builder.append_element(&root, name, attributes).unwrap();
    builder.finish().unwrap()
}

#[test]
fn an_element_view_reports_the_authored_name_and_attributes_in_order() {
    let name = QualifiedName::new(Namespace::svg(), Some("s".into()), "Rect");
    let attributes = OrderedAttributes::new(authored()).unwrap();
    let fragment = one_element(name.clone(), attributes.clone());
    let child = fragment.root().children().next().unwrap();
    let HtmlNodeKind::Element(element) = fragment.node(&child).unwrap().kind() else {
        panic!("expected an element");
    };
    assert_eq!(element.name(), &name);
    let view = element.attributes();
    assert_eq!(view.len(), 4);
    assert!(!view.is_empty());
    let authored = authored();
    assert!(view.iter().eq(authored.iter()));
    assert!(view.iter().rev().eq(authored.iter().rev()));
    assert_eq!(view.iter().len(), 4);
    for (index, expected) in authored.iter().enumerate() {
        assert_eq!(view.get(index), Some(expected), "index {index}");
    }
    assert_eq!(view.get(4), None);
    assert_eq!(view.get(usize::MAX), None);
}

#[test]
fn find_matches_the_exact_namespace_and_local_name() {
    let fragment = one_element(
        QualifiedName::new(Namespace::html(), None, "a"),
        OrderedAttributes::new(authored()).unwrap(),
    );
    let child = fragment.root().children().next().unwrap();
    let HtmlNodeKind::Element(element) = fragment.node(&child).unwrap().kind() else {
        panic!("expected an element");
    };
    let view = element.attributes();
    let local = |name: &str| DocumentString::from(name);
    // The two `href` attributes differ only by namespace.
    assert_eq!(
        view.find(&Namespace::none(), &local("href"))
            .unwrap()
            .value(),
        &DocumentString::from("near")
    );
    assert_eq!(
        view.find(&Namespace::from_uri(XLINK), &local("href"))
            .unwrap()
            .value(),
        &DocumentString::from("far")
    );
    assert_eq!(
        view.find(&Namespace::none(), &local("id")).unwrap().value(),
        &DocumentString::from("a")
    );
    // A value with a null unit and a lone surrogate is found by its name and keeps every unit.
    assert_eq!(
        view.find(&Namespace::none(), &local("class"))
            .unwrap()
            .value()
            .code_units(),
        [0x63, 0, 0xD800]
    );
    // Absent by local name, by namespace and by case.
    assert_eq!(view.find(&Namespace::none(), &local("missing")), None);
    assert_eq!(view.find(&Namespace::from_uri(XLINK), &local("id")), None);
    assert_eq!(view.find(&Namespace::none(), &local("ID")), None);
    assert_eq!(view.find(&Namespace::html(), &local("href")), None);
}

#[test]
fn to_ordered_returns_the_authored_collection() {
    let attributes = OrderedAttributes::new(authored()).unwrap();
    let fragment = one_element(
        QualifiedName::new(Namespace::html(), None, "a"),
        attributes.clone(),
    );
    let child = fragment.root().children().next().unwrap();
    let HtmlNodeKind::Element(element) = fragment.node(&child).unwrap().kind() else {
        panic!("expected an element");
    };
    let ordered = element.attributes().to_ordered();
    assert_eq!(ordered, attributes);
    assert_eq!(ordered.as_slice(), authored().as_slice());
    // The owned collection takes the same attributes in the same order a second time.
    assert_eq!(element.attributes().to_ordered(), ordered);
}

#[test]
fn an_element_without_attributes_has_an_empty_view() {
    let fragment = one_element(
        QualifiedName::new(Namespace::html(), None, "div"),
        OrderedAttributes::default(),
    );
    let child = fragment.root().children().next().unwrap();
    let HtmlNodeKind::Element(element) = fragment.node(&child).unwrap().kind() else {
        panic!("expected an element");
    };
    let view = element.attributes();
    assert_eq!(view.len(), 0);
    assert!(view.is_empty());
    assert_eq!(view.iter().count(), 0);
    assert_eq!(view.get(0), None);
    assert_eq!(
        view.find(&Namespace::none(), &DocumentString::from("id")),
        None
    );
    assert_eq!(view.to_ordered(), OrderedAttributes::default());
}

#[test]
fn link_flags_exist_only_for_an_html_link_element() {
    for (local, expected) in [
        ("link", Some(HtmlLinkFlags::fresh())),
        ("div", None),
        ("LINK", None),
    ] {
        let fragment = one_element(
            QualifiedName::new(Namespace::html(), None, local),
            OrderedAttributes::default(),
        );
        let child = fragment.root().children().next().unwrap();
        let HtmlNodeKind::Element(element) = fragment.node(&child).unwrap().kind() else {
            panic!("expected an element");
        };
        assert_eq!(element.link_flags(), expected, "{local}");
    }
    let svg_link = one_element(
        QualifiedName::new(Namespace::svg(), None, "link"),
        OrderedAttributes::default(),
    );
    let child = svg_link.root().children().next().unwrap();
    let HtmlNodeKind::Element(element) = svg_link.node(&child).unwrap().kind() else {
        panic!("expected an element");
    };
    assert_eq!(element.link_flags(), None);
}

#[test]
fn text_comment_instruction_and_doctype_views_carry_the_authored_data() {
    let text = DocumentString::from_utf16(vec![0x61, 0, 0xDC00, 0xD800]);
    let comment = DocumentString::from("<!-- raw -->");
    let instruction = HtmlAuthoredProcessingInstruction::new("tool", "a=\"b\"").unwrap();
    let doctype = HtmlDoctype::new("html", Some("pub".into()), Some(DocumentString::default()));

    let mut builder = HtmlDocument::builder(limits(), HtmlDocumentMode::LimitedQuirks).unwrap();
    let root = builder.root();
    builder.append_doctype(&root, doctype.clone()).unwrap();
    builder.append_text(&root, text.clone()).unwrap();
    builder.append_comment(&root, comment.clone()).unwrap();
    builder
        .append_processing_instruction(&root, instruction.clone())
        .unwrap();
    let document = builder.finish().unwrap();

    assert_eq!(
        document.root().kind(),
        HtmlNodeKind::Document(HtmlDocumentMode::LimitedQuirks)
    );
    let children: Vec<_> = document.root().children().collect();
    let kinds: Vec<_> = children
        .iter()
        .map(|id| document.node(id).unwrap().kind())
        .collect();
    assert_eq!(
        kinds,
        [
            HtmlNodeKind::Doctype(doctype),
            HtmlNodeKind::Text(text.clone()),
            HtmlNodeKind::Comment(comment),
            HtmlNodeKind::ProcessingInstruction(HtmlProcessingInstruction::from(instruction)),
        ]
    );
    let HtmlNodeKind::Text(shown) = &kinds[1] else {
        panic!("expected text");
    };
    assert_eq!(shown.code_units(), [0x61, 0, 0xDC00, 0xD800]);
    assert!(shown.shares_storage_with(&text));
    // A present empty system identifier stays distinct from an absent one.
    let HtmlNodeKind::Doctype(shown) = &kinds[0] else {
        panic!("expected a doctype");
    };
    assert_eq!(shown.system_id().map(DocumentString::len), Some(0));
    assert_eq!(shown.public_id().map(DocumentString::len), Some(3));
}

#[test]
fn a_fragment_root_view_is_a_fragment() {
    let fragment = HtmlFragment::builder(limits()).unwrap().finish().unwrap();
    assert_eq!(fragment.root().kind(), HtmlNodeKind::Fragment);
}
