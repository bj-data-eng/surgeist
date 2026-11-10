//! The build view reports the records a storage holds as borrowed code units, and
//! `stored_string` names the shared string of a non-element record. Every expectation is built
//! from the values handed to the commands, never read back from the owner.
use super::*;
use surgeist_html::{
    bounded::construction::{HtmlConstructionLeaf, HtmlStoredString},
    construction::{BuildStr, HtmlDoctype, HtmlProcessingInstruction},
    values::Attribute,
};

fn leaf(
    owner: HtmlConstructionStorage,
    leaf: HtmlConstructionLeaf,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    create(owner, HtmlConstructionCommand::Create { leaf })
}
fn units(value: BuildStr<'_>) -> Vec<u16> {
    value.code_units().to_vec()
}

#[test]
fn text_and_comment_views_carry_the_exact_units_and_their_length() {
    let units_in = vec![0x61, 0, 0xD800, 0xDC00, 0xDFFF];
    let (owner, text) = leaf(
        storage(),
        HtmlConstructionLeaf::Text(DocumentString::from_utf16(units_in.clone())),
    );
    let (owner, comment) = leaf(
        owner,
        HtmlConstructionLeaf::Comment(DocumentString::default()),
    );
    let ConstructionNodeKind::Text(shown) = owner.node(&text).unwrap().kind() else {
        panic!("expected text");
    };
    assert_eq!(shown.code_units(), units_in);
    assert_eq!(shown.len(), 5);
    assert!(!shown.is_empty());
    let ConstructionNodeKind::Comment(shown) = owner.node(&comment).unwrap().kind() else {
        panic!("expected a comment");
    };
    assert_eq!(shown.code_units(), []);
    assert_eq!(shown.len(), 0);
    assert!(shown.is_empty());
    // The view is `Copy` and compares by units.
    let first = owner.node(&text).unwrap().kind();
    let second = first;
    assert_eq!(first, second);
}

#[test]
fn instruction_and_doctype_views_keep_absent_and_empty_identifiers_apart() {
    let (owner, pi) = leaf(
        storage(),
        HtmlConstructionLeaf::ProcessingInstruction(HtmlProcessingInstruction::new("tool", "a b")),
    );
    let (owner, absent) = leaf(
        owner,
        HtmlConstructionLeaf::Doctype(HtmlDoctype::new("html", None, None)),
    );
    let (owner, present) = leaf(
        owner,
        HtmlConstructionLeaf::Doctype(HtmlDoctype::new(
            "svg",
            Some(DocumentString::default()),
            Some("sys".into()),
        )),
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
        HtmlConstructionCommand::CreateElement {
            name: name.clone(),
            attributes: attributes.clone(),
            link_flags: None,
        },
    );
    let (owner, document) = create(
        owner,
        HtmlConstructionCommand::CreateDocument {
            mode: HtmlDocumentMode::Quirks,
        },
    );
    let (owner, fragment) = create(owner, HtmlConstructionCommand::CreateFragment);
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

#[test]
fn stored_string_names_the_shared_string_of_the_matching_record_only() {
    let text = DocumentString::from_utf16(vec![0x78, 0xD800]);
    let comment = DocumentString::from("c");
    let pi = HtmlProcessingInstruction::new("t", "d");
    let doctype = HtmlDoctype::new("n", None, Some("s".into()));
    let (owner, text_node) = leaf(storage(), HtmlConstructionLeaf::Text(text.clone()));
    let (owner, comment_node) = leaf(owner, HtmlConstructionLeaf::Comment(comment.clone()));
    let (owner, pi_node) = leaf(
        owner,
        HtmlConstructionLeaf::ProcessingInstruction(pi.clone()),
    );
    let (owner, doctype_node) = leaf(owner, HtmlConstructionLeaf::Doctype(doctype.clone()));
    let (owner, element_node) = create(owner, element("p"));

    let held = |node: &HtmlBuilderNode, field| owner.node(node).unwrap().stored_string(field);
    // The strings handed to the command are the strings the record holds.
    assert!(
        held(&text_node, HtmlStoredString::Text)
            .unwrap()
            .shares_storage_with(&text)
    );
    assert!(
        held(&comment_node, HtmlStoredString::Comment)
            .unwrap()
            .shares_storage_with(&comment)
    );
    assert!(
        held(&pi_node, HtmlStoredString::ProcessingInstructionTarget)
            .unwrap()
            .shares_storage_with(pi.target())
    );
    assert!(
        held(&pi_node, HtmlStoredString::ProcessingInstructionData)
            .unwrap()
            .shares_storage_with(pi.data())
    );
    assert!(
        held(&doctype_node, HtmlStoredString::DoctypeName)
            .unwrap()
            .shares_storage_with(doctype.name())
    );
    assert!(
        held(&doctype_node, HtmlStoredString::DoctypeSystemId)
            .unwrap()
            .shares_storage_with(doctype.system_id().unwrap())
    );
    // An absent identifier, another record kind and an element have no such string.
    assert_eq!(held(&doctype_node, HtmlStoredString::DoctypePublicId), None);
    assert_eq!(held(&text_node, HtmlStoredString::Comment), None);
    assert_eq!(held(&comment_node, HtmlStoredString::Text), None);
    assert_eq!(held(&pi_node, HtmlStoredString::DoctypeName), None);
    assert_eq!(
        held(&doctype_node, HtmlStoredString::ProcessingInstructionData),
        None
    );
    for field in [
        HtmlStoredString::Text,
        HtmlStoredString::Comment,
        HtmlStoredString::ProcessingInstructionTarget,
        HtmlStoredString::ProcessingInstructionData,
        HtmlStoredString::DoctypeName,
        HtmlStoredString::DoctypePublicId,
        HtmlStoredString::DoctypeSystemId,
    ] {
        assert_eq!(held(&element_node, field), None, "{field:?}");
    }
}
