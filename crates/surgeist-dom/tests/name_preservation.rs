//! Exact names and ordered attributes across ownership and retained snapshots.
//! Name validation and native installation inventories live in xml_elements and
//! native_html_installation; these cases compose copying, lifetime and no-op rules.
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomCommit, DomError, DomLimits, DomNameAtom,
    DomSnapshot, Namespace, NodeId,
};

fn units(value: &[u16]) -> DocumentString {
    DocumentString::from_utf16(value.to_vec())
}
fn namespace(last: u16) -> Namespace {
    Namespace::from_uri(units(&[117, 114, 110, 58, last]))
}
fn dom() -> Dom {
    Dom::new(
        DomLimits::new(
            32,
            4096,
            64,
            0,
            0,
            64,
            65_536,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn fixture() -> (Dom, DocumentId, DocumentId, NodeId) {
    let mut dom = dom();
    let (first, second, root) = populate(&mut dom);
    (dom, first, second, root)
}
fn populate(dom: &mut Dom) -> (DocumentId, DocumentId, NodeId) {
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
    let done = apply(dom, batch);
    let first = done.document(&first).unwrap();
    let second = done.document(&second).unwrap();
    let element = done.created(&element).unwrap().unwrap();
    (first, second, element)
}

fn assert_values(snapshot: &DomSnapshot, id: NodeId, document: DocumentId) {
    let node = snapshot.node(id).unwrap();
    assert_eq!(node.owner_document(), Some(document));
    let element = node.as_element().unwrap();
    let name = element.name();
    assert_eq!(
        name.namespace_uri().unwrap().code_units(),
        [117, 114, 110, 58, 0xd800]
    );
    assert_eq!(name.prefix().unwrap().code_units(), [80, 114]);
    assert_eq!(name.local_name().code_units(), [65, 0xd800]);
    let attributes = element.attributes();
    assert_eq!(attributes.len(), 3);
    let first = &attributes[0];
    assert_eq!(
        first.name().namespace_uri().unwrap().code_units(),
        [117, 114, 110, 58, 0xd800]
    );
    assert_eq!(first.name().prefix().unwrap().code_units(), [113]);
    assert_eq!(first.name().local_name().code_units(), [75, 0xdfff]);
    assert_eq!(first.value().code_units(), [0, 0xd800, 13, 10, 0xdc00]);
    let second = &attributes[1];
    assert!(second.name().namespace_uri().is_none());
    assert!(second.name().prefix().is_none());
    assert_eq!(second.name().local_name().code_units(), [112, 58, 120]);
    assert_eq!(second.value().code_units(), [101, 0x301]);
    let third = &attributes[2];
    assert_eq!(
        third.name().namespace_uri().unwrap().code_units(),
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
    assert!(std::ptr::eq(
        before
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes(),
        after.node(root).unwrap().as_element().unwrap().attributes(),
    ));
}

const BOUND_TEXTS: [&[u16]; 5] = [
    &[65, 0xd800],
    &[75, 0xdfff],
    &[117, 114, 110, 58, 0xd800],
    &[117, 114, 110, 58, 0xdc00],
    &[112, 58, 120],
];

fn bind_fixture_names(dom: &mut Dom) -> [DomNameAtom; 5] {
    BOUND_TEXTS.map(|text| {
        dom.bind_name(&units(text), &AtomicBool::new(false))
            .unwrap()
    })
}

fn assert_fixture_bindings(snapshot: &DomSnapshot, root: NodeId, names: &[DomNameAtom; 5]) {
    let node = snapshot.node(root).unwrap();
    let element = node.as_element().unwrap();
    assert_eq!(
        element.attributes()[1]
            .name()
            .local_name_atom()
            .same_identity(&names[4]),
        Ok(true)
    );
    assert_eq!(
        element.name().local_name_atom().same_identity(&names[0]),
        Ok(true)
    );
    assert_eq!(
        element.name().namespace().unwrap().same_identity(&names[2]),
        Ok(true)
    );
    for index in [0, 2] {
        assert_eq!(
            element.attributes()[index]
                .name()
                .local_name_atom()
                .same_identity(&names[1]),
            Ok(true)
        );
    }
    assert_eq!(
        element.attributes()[0]
            .name()
            .namespace()
            .unwrap()
            .same_identity(&names[2]),
        Ok(true)
    );
    assert_eq!(
        element.attributes()[2]
            .name()
            .namespace()
            .unwrap()
            .same_identity(&names[3]),
        Ok(true)
    );
}

fn value_hash(value: &(impl Hash + ?Sized)) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn prebound_identity_follows_owner_domains_while_values_survive_import_and_owner_drop() {
    let mut original = dom();
    let empty = original.snapshot();
    // Each binding receives a new Vec, before either Document or any node exists.
    let names = bind_fixture_names(&mut original);
    let equal_names = bind_fixture_names(&mut original);
    for (name, equal) in names.iter().zip(&equal_names) {
        assert_eq!(name.same_identity(equal), Ok(true));
        assert_eq!(value_hash(name), value_hash(equal));
    }
    assert_eq!(original.snapshot().stamp(), empty.stamp());
    assert!(
        original
            .changes_since(empty.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );

    let (first, second, root) = populate(&mut original);
    let mut batch = DomBatch::new();
    let same_local = batch.create_element_ns(first, namespace(0xd800), units(&[75, 0xdfff]));
    let same_local = apply(&mut original, batch)
        .created(&same_local)
        .unwrap()
        .unwrap();
    let before = original.snapshot();
    assert_values(&before, root, first);
    assert_fixture_bindings(&before, root, &names);
    {
        let element = before.node(same_local).unwrap();
        let attribute_owner = before.node(root).unwrap();
        let element_name = element.as_element().unwrap().name();
        let attribute_name = attribute_owner.as_element().unwrap().attributes()[0].name();
        assert_eq!(element_name.local_name().code_units(), [75, 0xdfff]);
        assert!(element_name.prefix().is_none());
        assert_eq!(attribute_name.prefix().unwrap().code_units(), [113]);
        assert_eq!(
            element_name.local_name_atom().same_identity(&names[1]),
            Ok(true)
        );
        assert_eq!(
            element_name.same_expanded_identity(attribute_name),
            Ok(true)
        );
        assert_ne!(element_name, attribute_name); // Qualified spelling preserves prefix.
    }

    let mut batch = DomBatch::new();
    let copy = batch.import_node(&before, root, second, true);
    let copy = apply(&mut original, batch).created(&copy).unwrap().unwrap();
    let copied = original.snapshot();
    assert_values(&copied, copy, second);
    assert_fixture_bindings(&copied, copy, &names);
    let mut batch = DomBatch::new();
    batch.adopt(root, second);
    apply(&mut original, batch);
    assert_values(&original.snapshot(), root, second);
    assert_fixture_bindings(&original.snapshot(), root, &names);
    assert_values(&before, root, first);

    let mut batch = DomBatch::new();
    batch.destroy(root);
    batch.destroy(copy);
    batch.destroy(same_local);
    apply(&mut original, batch);
    drop(original);
    assert_values(&before, root, first);
    assert_fixture_bindings(&copied, copy, &names);

    let mut destination = dom();
    let destination_names = bind_fixture_names(&mut destination);
    for (source, target) in names.iter().zip(&destination_names) {
        assert_ne!(source.owner(), target.owner());
        assert_eq!(source.same_identity(target), Err(DomError::ForeignOwner));
        assert_ne!(source, target);
    }
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
    assert_values(&after, imported, document);
    assert_fixture_bindings(&after, imported, &destination_names);
    {
        let source = before.node(root).unwrap();
        let target = after.node(imported).unwrap();
        let source = source.as_element().unwrap();
        let target = target.as_element().unwrap();
        assert_eq!(source.name(), target.name());
        assert_eq!(value_hash(source.name()), value_hash(target.name()));
        assert_eq!(
            source.name().same_expanded_identity(target.name()),
            Err(DomError::ForeignOwner)
        );
        for (source, target) in source.attributes().iter().zip(target.attributes()) {
            assert_eq!(source, target);
            assert_eq!(value_hash(source), value_hash(target));
            assert_eq!(source.name(), target.name());
            assert_eq!(value_hash(source.name()), value_hash(target.name()));
            assert_eq!(
                source
                    .name()
                    .local_name_atom()
                    .same_identity(target.name().local_name_atom()),
                Err(DomError::ForeignOwner)
            );
        }
    }
    drop(destination);
    assert_values(&after, imported, document);
    assert_fixture_bindings(&after, imported, &destination_names);
    drop(empty);
    drop(before);
    drop(copied);
    drop(after);
    for ((source, equal), (target, expected)) in names
        .iter()
        .zip(&equal_names)
        .zip(destination_names.iter().zip(BOUND_TEXTS))
    {
        assert_eq!(source.text().code_units(), expected);
        assert_eq!(target.text().code_units(), expected);
        assert_eq!(source.same_identity(equal), Ok(true));
        assert_eq!(source.same_identity(target), Err(DomError::ForeignOwner));
    }
    // Retained atoms remain usable independently of every snapshot and owner.
    assert_eq!(names[0].same_identity(&equal_names[0]), Ok(true));
}
