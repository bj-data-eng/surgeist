//! Detached Attr creation and lifecycle through the canonical arena.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentString, Dom, DomBatch, DomError, DomLimits, DomNameLimits, DomNodeKind, DomSourceEdit,
    DomSourceLimits, DomSourceProgress, Namespace, NodeId,
};

fn owner() -> Dom {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    Dom::new(
        DomLimits::new(64, 8192, 128, 0, 0, 64, 1_000_000, names)
            .unwrap()
            .with_ranges(8)
            .unwrap(),
    )
    .unwrap()
}

fn policy() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 4096, 4096, 8)
}

#[test]
fn attr_creation_uses_the_actual_document_and_exact_qualified_name() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let xml = batch.create_document();
    let html = batch.create_html_document(&xml, None);
    let xml_attr = batch.create_attribute(&xml, "DaTa-Mode");
    let html_attr = batch.create_attribute(&html, "DaTa-Mode");
    let commit = dom.apply(batch, &cancel).unwrap();
    let xml = commit.document(&xml).unwrap();
    let html = commit.document(&html).unwrap();
    let xml_attr = commit.created(&xml_attr).unwrap().unwrap();
    let html_attr = commit.created(&html_attr).unwrap().unwrap();
    let snapshot = dom.snapshot();
    for (id, document, spelling) in [(xml_attr, xml, "DaTa-Mode"), (html_attr, html, "data-mode")] {
        let attr = snapshot.attr(id).unwrap();
        assert_eq!(attr.id().node(), id);
        assert_eq!(attr.owner_document(), document);
        assert_eq!(attr.owner_element(), None);
        assert_eq!(attr.name().local_name(), &spelling.into());
        assert_eq!(attr.name().namespace_uri(), None);
        assert_eq!(attr.name().prefix(), None);
        assert_eq!(attr.value(), &DocumentString::from(""));
        let node = snapshot.node(id).unwrap();
        assert!(matches!(node.kind(), DomNodeKind::Attr(_)));
        assert_eq!(node.parent(), None);
        assert!(node.children().is_empty());
    }
    assert!(matches!(
        snapshot.attr(xml.node()),
        Err(DomError::WrongNodeKind)
    ));

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::create_attribute_ns(
                html,
                Namespace::from_uri("urn:attribute"),
                "p:Token",
            ),
            policy(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("a detached Attr has no external creation effect");
    };
    let id = done.published_node().unwrap();
    drop(lease);
    let snapshot = dom.snapshot();
    let attr = snapshot.attr(id).unwrap();
    assert_eq!(attr.name().local_name(), &"Token".into());
    assert_eq!(attr.name().prefix(), Some(&"p".into()));
    assert_eq!(attr.name().namespace_uri(), Some(&"urn:attribute".into()));
    assert_eq!(attr.owner_document(), html);
    assert_eq!(attr.owner_element(), None);
}

#[test]
fn detached_attr_value_clone_adoption_and_retirement_use_exact_node_identity() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let destination = batch.create_document();
    let element = batch.create_element(&document, "root");
    let attr = batch.create_attribute(&document, "value");
    batch.insert_before(&element, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let destination = commit.document(&destination).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let attr = commit.created(&attr).unwrap().unwrap();
    let value = DocumentString::from_utf16(vec![0xd800, 0x0041, 0]);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_text_content(attr, value.clone()),
            policy(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    let original = dom.snapshot();
    assert_eq!(original.attr(attr).unwrap().value(), &value);

    let mut batch = DomBatch::new();
    let range = batch.create_range(document);
    batch.set_range_start(&range, attr, 0);
    batch.set_range_end(&range, attr, 0);
    let commit = dom.apply(batch, &cancel).unwrap();
    let range = commit.created_range(&range).unwrap().unwrap();
    let mut bad_offset = DomBatch::new();
    bad_offset.set_range_start(range, attr, 1);
    assert!(matches!(
        dom.apply(bad_offset, &cancel),
        Err(DomError::IndexSize)
    ));
    let mut bad_insert = DomBatch::new();
    bad_insert.insert_before(attr, element, None);
    assert!(matches!(
        dom.apply(bad_insert, &cancel),
        Err(DomError::HierarchyRequest)
    ));

    let mut transfer = DomBatch::new();
    let cloned = transfer.clone_node(attr, true);
    let imported = transfer.import_node(&original, attr, destination, false);
    transfer.adopt(attr, destination);
    let commit = dom.apply(transfer, &cancel).unwrap();
    let cloned = commit.created(&cloned).unwrap().unwrap();
    let imported = commit.created(&imported).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert_ne!(attr, cloned);
    assert_ne!(attr, imported);
    assert_ne!(cloned, imported);
    for (node, owner) in [
        (attr, destination),
        (cloned, document),
        (imported, destination),
    ] {
        let observed = snapshot.attr(node).unwrap();
        assert_eq!(observed.id().node(), node);
        assert_eq!(observed.owner_document(), owner);
        assert_eq!(observed.owner_element(), None);
        assert_eq!(observed.value(), &value);
    }
    assert_eq!(original.attr(attr).unwrap().owner_document(), document);

    let mut retire = DomBatch::new();
    retire.retire_range(range);
    retire.destroy(attr);
    dom.apply(retire, &cancel).unwrap();
    assert!(matches!(
        dom.snapshot().attr(attr),
        Err(DomError::StaleNode)
    ));
    assert_eq!(original.attr(attr).unwrap().value(), &value);
    let mut replacement = DomBatch::new();
    let created = replacement.create_attribute(destination, "value");
    let replacement: NodeId = dom
        .apply(replacement, &cancel)
        .unwrap()
        .created(&created)
        .unwrap()
        .unwrap();
    assert_ne!(attr, replacement);
    assert_eq!(
        dom.snapshot().attr(replacement).unwrap().value(),
        &"".into()
    );
}
