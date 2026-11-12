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
    assert_eq!(snapshot.node(id).unwrap().node_name(), "p:Token".into());
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

    let mut foreign = owner();
    let mut transfer = DomBatch::new();
    let receiving_document = transfer.create_document();
    let imported = transfer.import_node(&original, attr, &receiving_document, false);
    let commit = foreign.apply(transfer, &cancel).unwrap();
    let foreign_snapshot = foreign.snapshot();
    let imported = foreign_snapshot
        .attr(commit.created(&imported).unwrap().unwrap())
        .unwrap();
    assert_eq!(
        imported.owner_document(),
        commit.document(&receiving_document).unwrap()
    );
    assert_eq!(imported.value(), &value);
    assert_eq!(imported.name(), original.attr(attr).unwrap().name());
    assert_ne!(
        imported.name().owner(),
        original.attr(attr).unwrap().name().owner()
    );
    assert!(matches!(
        foreign_snapshot.attr(attr),
        Err(DomError::ForeignOwner)
    ));

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

#[test]
fn string_attribute_edits_keep_identity_prefix_and_order_until_removal() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let namespace = Namespace::from_uri("urn:ordered");
    let local = DocumentString::from("token");
    let mut initial = DomBatch::new();
    let document = initial.create_document();
    let element = initial.create_element(&document, "root");
    initial.set_attribute(&element, "first", "a");
    initial.set_attribute_ns(&element, namespace.clone(), "p:token", "old");
    initial.set_attribute(&element, "last", "z");
    let commit = dom.apply(initial, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let element = commit.created(&element).unwrap().unwrap();

    let before = dom.snapshot();
    let map = before.node(element).unwrap().attributes().unwrap();
    assert_eq!(map.len(), 3);
    assert!(map.item(3).is_none());
    assert_eq!(map.item(0).unwrap().name().local_name(), &"first".into());
    assert_eq!(map.item(2).unwrap().name().local_name(), &"last".into());
    let old = map.get_named_item_ns(&namespace, &local).unwrap();
    let id = old.id();
    assert_eq!(map.item(1).unwrap().id(), id);
    assert_eq!(map.get_named_item(&"p:token".into()).unwrap().id(), id);
    assert_eq!(old.owner_element(), Some(element));
    assert_eq!(old.owner_document(), document);
    assert_eq!(before.node(id.node()).unwrap().parent(), None);

    let value = DocumentString::from_utf16(vec![0, 0xd800, 0x61]);
    let mut update = DomBatch::new();
    update.set_attribute_ns(element, namespace.clone(), "q:token", value.clone());
    let commit = dom.apply(update, &cancel).unwrap();
    assert!(
        commit
            .changes()
            .changes()
            .contains(&surgeist_dom::DomChange::AttrValue(id))
    );
    assert!(
        commit
            .changes()
            .changes()
            .contains(&surgeist_dom::DomChange::Attributes(element))
    );
    let edited = dom.snapshot();
    let map = edited.node(element).unwrap().attributes().unwrap();
    assert_eq!(map.item(1).unwrap().id(), id);
    assert_eq!(map.item(1).unwrap().value(), &value);
    assert_eq!(map.item(1).unwrap().name().prefix(), Some(&"p".into()));
    assert!(map.get_named_item(&"q:token".into()).is_none());
    assert_eq!(old.value(), &"old".into());

    let mut remove = DomBatch::new();
    remove.remove_attribute_ns(element, namespace.clone(), local.clone());
    dom.apply(remove, &cancel).unwrap();
    let removed = dom.snapshot();
    let detached = removed.attr(id.node()).unwrap();
    assert_eq!(detached.owner_element(), None);
    assert_eq!(detached.owner_document(), document);
    assert_eq!(detached.value(), &value);
    assert_eq!(
        edited.attr(id.node()).unwrap().owner_element(),
        Some(element)
    );

    let mut recreate = DomBatch::new();
    recreate.set_attribute_ns(element, namespace, "q:token", "fresh");
    dom.apply(recreate, &cancel).unwrap();
    let final_snapshot = dom.snapshot();
    let map = final_snapshot.node(element).unwrap().attributes().unwrap();
    assert_eq!(map.item(1).unwrap().name().local_name(), &"last".into());
    assert_eq!(map.item(2).unwrap().name().prefix(), Some(&"q".into()));
    assert_ne!(map.item(2).unwrap().id(), id);
    assert_eq!(final_snapshot.attr(id.node()).unwrap().value(), &value);
}

#[test]
fn attr_document_and_element_custody_are_independent() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut initial = DomBatch::new();
    let source = initial.create_document();
    let second = initial.create_document();
    let third = initial.create_document();
    let element = initial.create_element(&source, "root");
    initial.set_attribute(&element, "owned", "value");
    let commit = dom.apply(initial, &cancel).unwrap();
    let source = commit.document(&source).unwrap();
    let second = commit.document(&second).unwrap();
    let third = commit.document(&third).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let captured = dom.snapshot();
    let attr = captured
        .node(element)
        .unwrap()
        .attributes()
        .unwrap()
        .item(0)
        .unwrap()
        .id();

    let mut invalid = DomBatch::new();
    invalid.destroy(attr.node());
    assert!(matches!(
        dom.apply(invalid, &cancel),
        Err(DomError::InvalidDestruction)
    ));

    let mut adopt = DomBatch::new();
    adopt.adopt(attr.node(), second);
    adopt.adopt(element, source);
    dom.apply(adopt, &cancel).unwrap();
    let divergent = dom.snapshot();
    let view = divergent.attr(attr.node()).unwrap();
    assert_eq!(view.owner_element(), Some(element));
    assert_eq!(view.owner_document(), second);

    let mut adopt = DomBatch::new();
    adopt.adopt(element, third);
    dom.apply(adopt, &cancel).unwrap();
    let reconciled = dom.snapshot();
    let view = reconciled.attr(attr.node()).unwrap();
    assert_eq!(view.owner_element(), Some(element));
    assert_eq!(view.owner_document(), third);

    let mut destroy = DomBatch::new();
    destroy.destroy(element);
    dom.apply(destroy, &cancel).unwrap();
    assert!(matches!(
        dom.snapshot().attr(attr.node()),
        Err(DomError::StaleNode)
    ));
    assert_eq!(captured.attr(attr.node()).unwrap().owner_document(), source);
    assert_eq!(
        reconciled.attr(attr.node()).unwrap().value(),
        &"value".into()
    );
}

#[test]
fn shallow_element_clone_and_foreign_import_create_new_attached_attributes() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let namespace = Namespace::from_uri("urn:copy");
    let value = DocumentString::from_utf16(vec![0xd800, 0, 0x51]);
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let element = setup.create_element(&document, "source");
    setup.set_attribute(&element, "first", "");
    setup.set_attribute_ns(&element, namespace.clone(), "p:token", value.clone());
    let created = dom.apply(setup, &cancel).unwrap();
    let document = created.document(&document).unwrap();
    let element = created.created(&element).unwrap().unwrap();
    let before = dom.snapshot();
    let original = before.node(element).unwrap().attributes().unwrap();
    let original_ids: Vec<_> = original.iter().map(|attr| attr.id()).collect();

    let mut batch = DomBatch::new();
    let clone = batch.clone_node(element, false);
    let created = dom.apply(batch, &cancel).unwrap();
    let clone = created.created(&clone).unwrap().unwrap();
    let after = dom.snapshot();
    let cloned = after.node(clone).unwrap().attributes().unwrap();

    let mut foreign = owner();
    let mut batch = DomBatch::new();
    let target = batch.create_document();
    let import = batch.import_node(&before, element, &target, false);
    let created = foreign.apply(batch, &cancel).unwrap();
    let target = created.document(&target).unwrap();
    let import = created.created(&import).unwrap().unwrap();
    let imported_snapshot = foreign.snapshot();
    let imported = imported_snapshot
        .node(import)
        .unwrap()
        .attributes()
        .unwrap();

    for (map, owner, document) in [(cloned, clone, document), (imported, import, target)] {
        assert_eq!(map.len(), 2);
        for (index, attr) in map.iter().enumerate() {
            assert_ne!(attr.id(), original_ids[index]);
            assert_eq!(attr.name(), original.item(index).unwrap().name());
            assert_eq!(attr.owner_element(), Some(owner));
            assert_eq!(attr.owner_document(), document);
        }
        assert_eq!(map.item(1).unwrap().value(), &value);
    }
    assert_ne!(
        original.item(1).unwrap().name().owner(),
        imported.item(1).unwrap().name().owner()
    );
}
