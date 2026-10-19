//! Public value-preservation characterizations for the canonical-name migration.
//! Name validation and native installation inventories live in xml_elements and
//! native_html_installation; these cases compose copying, lifetime and no-op rules.
use std::sync::{Arc, atomic::AtomicBool};

use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomCommit, DomError, DomLimits, DomSnapshot,
    Namespace, NodeId,
};

fn units(value: &[u16]) -> DocumentString {
    DocumentString::from_utf16(value.to_vec())
}
fn namespace(last: u16) -> Namespace {
    Namespace::from_uri(units(&[117, 114, 110, 58, last]))
}
fn dom() -> Dom {
    Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 64, 65_536).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn fixture() -> (Dom, DocumentId, DocumentId, NodeId) {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let first = batch.create_document();
    let second = batch.create_document();
    let element =
        batch.create_element_ns(&first, namespace(0xd800), units(&[80, 114, 58, 65, 0xd800]));
    batch.set_attribute_ns(
        &element,
        namespace(0xd800),
        units(&[113, 58, 75, 0xdfff]),
        units(&[0, 0xd800, 13, 10, 0xdc00]),
    );
    batch.set_attribute(&element, "p:x", units(&[101, 0x301]));
    batch.set_attribute_ns(
        &element,
        namespace(0xdc00),
        units(&[75, 0xdfff]),
        units(&[0xe9]),
    );
    let text = batch.create_text(&first, units(&[101, 0x301, 0xd800]));
    batch.insert_before(&text, &element, None);
    let done = apply(&mut dom, batch);
    let first = done.document(&first).unwrap();
    let second = done.document(&second).unwrap();
    let element = done.created(&element).unwrap().unwrap();
    (dom, first, second, element)
}

fn assert_values(snapshot: &DomSnapshot, id: NodeId, document: DocumentId) {
    let node = snapshot.node(id).unwrap();
    assert_eq!(node.owner_document(), Some(document));
    let element = node.as_element().unwrap();
    let name = element.name();
    assert_eq!(
        name.namespace().uri().unwrap().code_units(),
        [117, 114, 110, 58, 0xd800]
    );
    assert_eq!(name.prefix().unwrap().code_units(), [80, 114]);
    assert_eq!(name.local_name().code_units(), [65, 0xd800]);
    let attributes = element.attributes().as_slice();
    assert_eq!(attributes.len(), 3);
    let first = &attributes[0];
    assert_eq!(
        first.name().namespace().uri().unwrap().code_units(),
        [117, 114, 110, 58, 0xd800]
    );
    assert_eq!(first.name().prefix().unwrap().code_units(), [113]);
    assert_eq!(first.name().local_name().code_units(), [75, 0xdfff]);
    assert_eq!(first.value().code_units(), [0, 0xd800, 13, 10, 0xdc00]);
    let second = &attributes[1];
    assert!(second.name().namespace().uri().is_none());
    assert!(second.name().prefix().is_none());
    assert_eq!(second.name().local_name().code_units(), [112, 58, 120]);
    assert_eq!(second.value().code_units(), [101, 0x301]);
    let third = &attributes[2];
    assert_eq!(
        third.name().namespace().uri().unwrap().code_units(),
        [117, 114, 110, 58, 0xdc00]
    );
    assert!(third.name().prefix().is_none());
    assert_eq!(third.name().local_name().code_units(), [75, 0xdfff]);
    assert_eq!(third.value().code_units(), [0xe9]);
    assert_eq!(node.children().len(), 1);
    let text = snapshot.node(node.children()[0]).unwrap();
    assert_eq!(text.owner_document(), Some(document));
    assert_eq!(text.data().unwrap().code_units(), [101, 0x301, 0xd800]);
}

#[test]
fn raw_names_and_attribute_order_survive_cross_document_copy_adoption_and_owner_drop() {
    let (mut original, first, second, root) = fixture();
    let before = original.snapshot();
    assert_values(&before, root, first);

    // Same-owner import is the existing public cloning boundary. It creates a
    // detached node in the destination Document without changing the source.
    let mut batch = DomBatch::new();
    let copy = batch.import_node(&before, root, second, true);
    let copy = apply(&mut original, batch).created(&copy).unwrap().unwrap();
    let copied = original.snapshot();
    assert_ne!(copy, root);
    assert_eq!(copy.owner(), root.owner());
    assert!(copied.node(copy).unwrap().parent().is_none());
    assert_values(&copied, copy, second);
    assert_eq!(
        copied.node(copy).unwrap().as_element(),
        before.node(root).unwrap().as_element()
    );

    let mut batch = DomBatch::new();
    batch.adopt(root, second);
    apply(&mut original, batch);
    assert_values(&original.snapshot(), root, second);
    assert_values(&before, root, first);

    let mut batch = DomBatch::new();
    batch.destroy(root);
    batch.destroy(copy);
    apply(&mut original, batch);
    assert_eq!(
        original.snapshot().node(root).unwrap_err(),
        DomError::StaleNode
    );
    drop(original);
    assert_values(&before, root, first);
    assert_values(&copied, copy, second);

    let mut destination = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let document = apply(&mut destination, batch).document(&document).unwrap();
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&before, root, document, true);
    let imported = apply(&mut destination, batch)
        .created(&imported)
        .unwrap()
        .unwrap();
    let after = destination.snapshot();
    assert_ne!(imported.owner(), root.owner());
    assert_values(&after, imported, document);
    // Semantic value equality must remain meaningful across owner domains.
    assert_eq!(
        after.node(imported).unwrap().as_element(),
        before.node(root).unwrap().as_element()
    );
    drop(destination);
    assert_values(&after, imported, document);
    assert_values(&before, root, first);
}

#[test]
fn restored_expanded_attribute_values_keep_original_bank_and_empty_history() {
    let (mut dom, document, _, root) = fixture();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(
        root,
        namespace(0xd800),
        units(&[110, 58, 75, 0xdfff]),
        "temporary",
    );
    batch.set_attribute_ns(
        root,
        namespace(0xd800),
        units(&[122, 58, 75, 0xdfff]),
        units(&[0, 0xd800, 13, 10, 0xdc00]),
    );
    batch.set_attribute(root, "transient", "gone");
    batch.remove_attribute(root, "transient");
    let done = apply(&mut dom, batch);
    assert!(done.changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    let after = dom.snapshot();
    assert_values(&after, root, document);
    assert_values(&before, root, document);
    assert!(Arc::ptr_eq(
        before
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .shared_attributes(),
        after
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .shared_attributes(),
    ));
}
