use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomDocumentClass, DomError, DomLimits,
    DomRegistryBinding, DomResource, HtmlDocumentMode,
};

// DOM's native Document defaults retain three strings: application/xml (15),
// UTF-8 (5) and about:blank (11). These are stored slots, not derived display.
const DEFAULT_METADATA_UNITS: usize = 31;

fn dom() -> Dom {
    Dom::new(
        DomLimits::new(
            32,
            1024,
            64,
            0,
            0,
            8,
            8192,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
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
    let mut dom = Dom::new(
        DomLimits::new(
            1,
            DEFAULT_METADATA_UNITS,
            1,
            0,
            0,
            0,
            0,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    batch.create_document();
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(dom.usage().string_code_units(), DEFAULT_METADATA_UNITS);
}

#[test]
fn insufficient_default_metadata_budget_rejects_atomic_creation() {
    let mut dom = Dom::new(
        DomLimits::new(
            1,
            30,
            1,
            0,
            0,
            0,
            0,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
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
        dom.set_limits(
            DomLimits::new(
                1,
                30,
                1,
                0,
                0,
                0,
                0,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap()
            )
            .unwrap()
        ),
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
    dom.set_limits(
        DomLimits::new(
            1,
            DEFAULT_METADATA_UNITS,
            1,
            0,
            0,
            0,
            0,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(dom.usage().string_code_units(), DEFAULT_METADATA_UNITS);
}

#[test]
fn focused_designation_survives_projection_and_rejected_document_destruction() {
    use surgeist_dom::DomFocusedAreaDesignation::KnownNull;
    let mut owner = dom();
    let original = document(&mut owner);
    let captured = owner.snapshot();
    assert_eq!(
        captured
            .document(original)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
    let mut batch = DomBatch::new();
    let comment = batch.create_comment(original, "projection");
    batch.insert_before(&comment, original, None);
    let comment = owner
        .apply(batch, &AtomicBool::new(false))
        .unwrap()
        .created(&comment)
        .unwrap()
        .unwrap();
    assert_eq!(
        owner
            .snapshot()
            .document(original)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
    let mut batch = DomBatch::new();
    batch.destroy(original);
    assert_eq!(
        owner.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidDestruction
    );
    assert_eq!(
        owner
            .snapshot()
            .document(original)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
    let mut batch = DomBatch::new();
    batch.detach(comment);
    batch.destroy(comment);
    owner.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        owner.snapshot().node(comment).unwrap_err(),
        DomError::StaleNode
    );
    let replacement = document(&mut owner);
    assert_ne!(replacement, original);
    assert_eq!(
        owner
            .snapshot()
            .document(replacement)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
    let mut foreign = dom();
    let foreign_document = document(&mut foreign);
    assert_eq!(
        owner.snapshot().document(foreign_document).unwrap_err(),
        DomError::ForeignOwner
    );
    drop(owner);
    assert_eq!(
        captured
            .document(original)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
}

#[test]
fn focused_designation_failed_creation_is_atomic_and_retry_is_fresh() {
    use surgeist_dom::DomFocusedAreaDesignation::KnownNull;
    let mut owner = dom();
    let original = document(&mut owner);
    let before = owner.snapshot();
    let usage = owner.usage();
    let mut batch = DomBatch::new();
    batch.create_document();
    assert_eq!(
        owner.apply(batch, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(owner.snapshot().stamp(), before.stamp());
    assert_eq!(owner.usage(), usage);
    // Actual finite metadata capacity: one existing Document fits, a second
    // cannot publish any partial initial state.
    let limited = DomLimits::new(
        32,
        DEFAULT_METADATA_UNITS,
        64,
        0,
        0,
        8,
        8192,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap();
    owner.set_limits(limited).unwrap();
    let mut batch = DomBatch::new();
    batch.create_document();
    assert_eq!(
        owner.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: DEFAULT_METADATA_UNITS
        }
    );
    assert_eq!(owner.snapshot().stamp(), before.stamp());
    assert_eq!(owner.usage(), usage);
    assert_eq!(
        owner
            .snapshot()
            .document(original)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
    owner.set_limits(dom().limits()).unwrap();
    let next = document(&mut owner);
    assert_eq!(
        owner
            .snapshot()
            .document(next)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
}

#[test]
fn focused_designation_source_output_refusal_discards_unpublished_initialization() {
    use surgeist_dom::{
        DomFocusedAreaDesignation::KnownNull, DomSourceEdit, DomSourceLimits, DomSourceOutput,
        DomSourceProgress,
    };
    let mut owner = dom();
    let before = owner.snapshot();
    let bounded = DomSourceLimits::new(8_000_000, 65_536, 1_000_000, 16, 0, 64, 4);
    let operation = {
        let mut lease = owner
            .begin_source_edit(
                DomSourceEdit::create_document(),
                bounded,
                &AtomicBool::new(false),
            )
            .unwrap();
        assert_eq!(
            lease.advance(&AtomicBool::new(false)).unwrap_err(),
            DomError::SourceOutputLimit
        );
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        assert!(!lease.status().has_published());
        assert_eq!(lease.snapshot().nodes().count(), 0);
        assert!(lease.status().spent_work() > 0);
        lease.operation()
    };
    let raised = DomSourceLimits::new(8_000_000, 65_536, 1_000_000, 16, 64, 64, 4);
    assert!(owner.pending_source_edit().is_none());
    let mut lease = owner
        .begin_source_edit(
            DomSourceEdit::create_document(),
            raised,
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_ne!(lease.operation(), operation);
    let DomSourceProgress::Complete(completion) = lease.advance(&AtomicBool::new(false)).unwrap()
    else {
        panic!("native Document creation has no external service");
    };
    assert_eq!(completion.operation(), lease.operation());
    let Some(DomSourceOutput::Document(document)) = completion.output() else {
        panic!("actual Document output");
    };
    assert_eq!(
        lease
            .snapshot()
            .document(document)
            .unwrap()
            .focused_area_designation(),
        KnownNull
    );
}
