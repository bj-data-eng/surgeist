use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomDocumentClass, DomError, DomLimits,
    DomRegistryBinding, DomResource, HtmlDocumentMode,
};

// DOM's native Document defaults retain three strings: application/xml (15),
// UTF-8 (5) and about:blank (11). These are stored slots, not derived display.
const DEFAULT_METADATA_UNITS: usize = 31;

fn dom() -> Dom {
    Dom::new(DomLimits::new(32, 1024, 64, 0, 0, 8, 8192).unwrap()).unwrap()
}

fn document(dom: &mut Dom) -> DocumentId {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    dom.apply(batch, &AtomicBool::new(false))
        .unwrap()
        .document(&document)
        .unwrap()
}

#[test]
fn native_document_charges_its_source_default_string_slots() {
    let mut dom =
        Dom::new(DomLimits::new(1, DEFAULT_METADATA_UNITS, 1, 0, 0, 0, 0).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    batch.create_document();
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(dom.usage().string_code_units(), DEFAULT_METADATA_UNITS);
}

#[test]
fn insufficient_default_metadata_budget_rejects_atomic_creation() {
    let mut dom = Dom::new(DomLimits::new(1, 30, 1, 0, 0, 0, 0).unwrap()).unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.create_document();
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: 30
        }
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.snapshot().nodes().count(), 0);
    assert_eq!(dom.usage(), usage);
}

#[test]
fn native_defaults_are_canonical_and_discoverable_in_a_snapshot() {
    let mut dom = dom();
    let document = document(&mut dom);
    let snapshot = dom.snapshot();
    let node = snapshot.node(snapshot.nodes().next().unwrap()).unwrap();
    let view = node.as_document().unwrap();
    assert_eq!(view.id(), document);
    assert_eq!(node.owner_document(), None);
    let metadata = view.metadata();
    assert_eq!(metadata.class(), DomDocumentClass::Xml);
    assert_eq!(
        metadata.content_type(),
        &DocumentString::from("application/xml")
    );
    assert_eq!(metadata.encoding(), &DocumentString::from("UTF-8"));
    assert_eq!(metadata.url(), &DocumentString::from("about:blank"));
    assert_eq!(metadata.mode(), HtmlDocumentMode::NoQuirks);
    assert!(!metadata.allow_declarative_shadow_roots());
    assert_eq!(metadata.custom_element_registry(), DomRegistryBinding::Null);
    assert_eq!(metadata, snapshot.document(document).unwrap().metadata());
}

#[test]
fn opaque_origins_are_distinct_despite_identical_urls_in_and_across_stores() {
    let mut first = dom();
    let mut batch = DomBatch::new();
    let a = batch.create_document();
    let b = batch.create_document();
    let commit = first.apply(batch, &AtomicBool::new(false)).unwrap();
    let snapshot = first.snapshot();
    let a = snapshot
        .document(commit.document(&a).unwrap())
        .unwrap()
        .metadata();
    let b = snapshot
        .document(commit.document(&b).unwrap())
        .unwrap()
        .metadata();
    let mut second = dom();
    let c = document(&mut second);
    let other_snapshot = second.snapshot();
    let c = other_snapshot.document(c).unwrap().metadata();
    assert_eq!(a.url(), b.url());
    assert_eq!(a.url(), c.url());
    assert_ne!(a.origin(), b.origin());
    assert_ne!(a.origin(), c.origin());
    assert_ne!(b.origin(), c.origin());
    assert_eq!(a.origin().clone(), *a.origin());
    assert_eq!(
        first.usage().string_code_units(),
        2 * DEFAULT_METADATA_UNITS
    );
}

#[test]
fn failed_creation_does_not_publish_metadata_identity_usage_or_history() {
    let mut dom = dom();
    let original = document(&mut dom);
    let before = dom.snapshot();
    let metadata = before.document(original).unwrap().metadata().clone();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    let failed_document = batch.create_document();
    let text = batch.create_text(&failed_document, "forbidden child");
    batch.insert_before(&text, &failed_document, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.snapshot().nodes().count(), 1);
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    assert_eq!(
        dom.snapshot().document(original).unwrap().metadata(),
        &metadata
    );

    let mut next = DomBatch::new();
    let next_document = next.create_document();
    let committed = dom.apply(next, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        committed.document(&failed_document),
        Err(DomError::ForeignBatch)
    );
    let next_document = committed.document(&next_document).unwrap();
    assert_eq!(
        committed.changes().changes(),
        [DomChange::Created(next_document.node())]
    );
    assert_ne!(
        dom.snapshot()
            .document(next_document)
            .unwrap()
            .metadata()
            .origin(),
        metadata.origin()
    );
    assert_eq!(
        dom.snapshot().document(original).unwrap().metadata(),
        &metadata
    );
    assert_eq!(dom.usage().string_code_units(), 2 * DEFAULT_METADATA_UNITS);
}

#[test]
fn ordinary_doctype_edits_never_change_native_document_metadata_or_origin() {
    let mut dom = dom();
    let document = document(&mut dom);
    let before = dom.snapshot();
    let metadata = before.document(document).unwrap().metadata().clone();
    let mut batch = DomBatch::new();
    let doctype = batch.create_document_type(document, "html", "", "");
    batch.insert_before(&doctype, document, None);
    let doctype = dom
        .apply(batch, &AtomicBool::new(false))
        .unwrap()
        .created(&doctype)
        .unwrap()
        .unwrap();
    assert_eq!(
        dom.snapshot().document(document).unwrap().metadata(),
        &metadata
    );
    let mut batch = DomBatch::new();
    batch.detach(doctype);
    batch.destroy(doctype);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        dom.snapshot().document(document).unwrap().metadata(),
        &metadata
    );
    assert_eq!(dom.usage().string_code_units(), DEFAULT_METADATA_UNITS);
}

#[test]
fn native_metadata_survives_later_edits_noops_and_store_destruction() {
    let mut dom = dom();
    let document = document(&mut dom);
    let before = dom.snapshot();
    let metadata = before.document(document).unwrap().metadata().clone();
    let mut batch = DomBatch::new();
    let comment = batch.create_comment(document, "later");
    let comment = dom
        .apply(batch, &AtomicBool::new(false))
        .unwrap()
        .created(&comment)
        .unwrap()
        .unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(comment);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let stamp = dom.snapshot().stamp();
    dom.apply(DomBatch::new(), &AtomicBool::new(false)).unwrap();
    assert_eq!(dom.snapshot().stamp(), stamp);
    let current = dom.snapshot();
    drop(dom);
    assert_eq!(before.document(document).unwrap().metadata(), &metadata);
    assert_eq!(current.document(document).unwrap().metadata(), &metadata);
    assert!(before.node(document.node()).unwrap().children().is_empty());
}

#[test]
fn lowered_capacity_cannot_hide_or_discard_admitted_metadata_strings() {
    let mut dom = dom();
    let document = document(&mut dom);
    let before = dom.snapshot();
    let usage = dom.usage();
    assert_eq!(
        dom.set_limits(DomLimits::new(1, 30, 1, 0, 0, 0, 0).unwrap()),
        Err(DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: 30
        })
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.snapshot().document(document).unwrap().metadata(),
        before.document(document).unwrap().metadata()
    );
    dom.set_limits(DomLimits::new(1, DEFAULT_METADATA_UNITS, 1, 0, 0, 0, 0).unwrap())
        .unwrap();
    assert_eq!(dom.usage().string_code_units(), DEFAULT_METADATA_UNITS);
}
