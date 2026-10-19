//! Fullscreen7c38 exit steps 1–2 reject an inactive Document before collection,
//! event enqueue, unfullscreen or parallel handoff. Native XML/HTML Documents
//! have actual absent browsing contexts; the hosted positive path belongs to
//! Runtime's public composition tests.
use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomCommit, DomError, DomFullscreenExitReturn, DomLimits,
    DomResource, DomSourceCompletion, DomSourceEdit, DomSourceLimits, DomSourceProgress, Namespace,
};

fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            64,
            100_000,
            256,
            128,
            128,
            32,
            4_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 100_000_000, 64, 1024, 128, 8)
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn document(dom: &mut Dom) -> DocumentId {
    let mut batch = DomBatch::new();
    let ticket = batch.create_document();
    apply(dom, batch).document(&ticket).unwrap()
}
fn complete(dom: &mut Dom, edit: DomSourceEdit) -> DomSourceCompletion {
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .begin_source_edit(edit, source_limits(), &cancel)
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("the actual negative source branch requires no hosted handoff");
    };
    done
}

#[test]
fn inactive_xml_document_returns_type_error_without_exit_or_event_capacity() {
    let mut dom = owner();
    let document = document(&mut dom);
    let before = dom.snapshot();
    let usage = dom.usage();
    assert!(
        !before
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let done = complete(&mut dom, DomSourceEdit::exit_fullscreen(document));
    assert_eq!(
        done.fullscreen_exit_return(),
        Some(DomFullscreenExitReturn::RejectedTypeError)
    );
    assert_eq!(done.output(), None);
    assert!(done.commit().changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .pending_fullscreen_events()
            .unwrap()
            .is_empty()
    );
    assert!(dom.pending_source_edit().is_none());
    assert!(dom.snapshot().incomplete_source_operation().is_none());
}

#[test]
fn inactive_fullscreen_element_is_not_unfullscreened_or_queued_by_the_rejected_call() {
    for attached in [false, true] {
        let mut dom = owner();
        let document = document(&mut dom);
        let mut batch = DomBatch::new();
        let node = batch.create_element_ns(document, Namespace::html(), "div");
        if attached {
            batch.insert_before(&node, document, None);
        }
        batch.fullscreen_element_model(&node);
        let node = apply(&mut dom, batch).created(&node).unwrap().unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        assert!(
            !before
                .document(document)
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
        let done = complete(&mut dom, DomSourceEdit::exit_fullscreen(document));
        assert_eq!(
            done.fullscreen_exit_return(),
            Some(DomFullscreenExitReturn::RejectedTypeError)
        );
        assert!(done.commit().changes().changes().is_empty());
        let after = dom.snapshot();
        assert_eq!(after.stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(
            after
                .node(node)
                .unwrap()
                .as_element()
                .unwrap()
                .fullscreen_flag()
        );
        assert_eq!(
            after.node(node).unwrap().parent(),
            attached.then_some(document.node())
        );
        assert_eq!(
            after
                .document(document)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members(),
            &[node]
        );
        assert!(
            after
                .document(document)
                .unwrap()
                .pending_fullscreen_events()
                .unwrap()
                .is_empty()
        );
        assert!(dom.pending_source_edit().is_none());
    }
}

#[test]
fn native_html_document_uses_actual_inactivity_despite_its_complete_skeleton() {
    let mut dom = owner();
    let associated = document(&mut dom);
    let mut batch = DomBatch::new();
    let html = batch.create_html_document(associated, None);
    let html = apply(&mut dom, batch).document(&html).unwrap();
    let before = dom.snapshot();
    assert!(before.document(html).unwrap().document_element().is_some());
    assert!(!before.document(html).unwrap().is_fully_active().unwrap());
    let done = complete(&mut dom, DomSourceEdit::exit_fullscreen(html));
    assert_eq!(
        done.fullscreen_exit_return(),
        Some(DomFullscreenExitReturn::RejectedTypeError)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(
        dom.snapshot()
            .document(html)
            .unwrap()
            .pending_fullscreen_events()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_different_source_method_has_no_fullscreen_method_return() {
    let mut dom = owner();
    let document = document(&mut dom);
    let done = complete(
        &mut dom,
        DomSourceEdit::create_comment(document.node(), "ordinary"),
    );
    assert_eq!(done.fullscreen_exit_return(), None);
    assert!(done.published_node().is_some());
}

#[test]
fn foreign_document_is_a_qualified_identity_error_instead_of_source_type_error() {
    let mut dom = owner();
    let document = document(&mut dom);
    let mut foreign = owner();
    let foreign_document = self::document(&mut foreign);
    let before = dom.snapshot();
    let cancel = AtomicBool::new(false);
    let error = match dom.begin_source_edit(
        DomSourceEdit::exit_fullscreen(foreign_document),
        source_limits(),
        &cancel,
    ) {
        Err(error) => error,
        Ok(mut lease) => lease.advance(&cancel).unwrap_err(),
    };
    assert_eq!(error, DomError::ForeignOwner);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .pending_fullscreen_events()
            .unwrap()
            .is_empty()
    );
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn source_storage_work_and_cancellation_refusal_do_not_publish_a_negative_return_prefix() {
    let mut dom = owner();
    let document = document(&mut dom);
    let before = dom.snapshot();
    let usage = dom.usage();
    for (policy, cancelled, expected) in [
        (
            DomSourceLimits::new(0, 1_000_000, 100_000_000, 8, 16, 0, 1),
            false,
            DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit: 0,
            },
        ),
        (
            DomSourceLimits::new(32_000_000, 1_000_000, 0, 8, 16, 0, 1),
            false,
            DomError::SourceWorkLimit { limit: 0 },
        ),
        (source_limits(), true, DomError::Cancelled),
    ] {
        let cancel = AtomicBool::new(cancelled);
        let error = match dom.begin_source_edit(
            DomSourceEdit::exit_fullscreen(document),
            policy,
            &cancel,
        ) {
            Err(error) => error,
            Ok(mut lease) => lease.advance(&cancel).unwrap_err(),
        };
        assert_eq!(error, expected);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(dom.pending_source_edit().is_none());
        assert!(
            dom.snapshot()
                .document(document)
                .unwrap()
                .pending_fullscreen_events()
                .unwrap()
                .is_empty()
        );
    }
    assert_eq!(
        complete(&mut dom, DomSourceEdit::exit_fullscreen(document)).fullscreen_exit_return(),
        Some(DomFullscreenExitReturn::RejectedTypeError)
    );
}

#[test]
fn empty_event_transfer_is_a_qualified_noop_with_its_actual_document_basis() {
    let mut dom = owner();
    let document = document(&mut dom);
    let before = dom.snapshot();
    let batch = dom
        .take_fullscreen_events(document, 0, &AtomicBool::new(false))
        .unwrap();
    assert!(batch.events().is_empty());
    assert_eq!(batch.basis(), before.document(document).unwrap().stamp());
    assert!(batch.changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let mut foreign = owner();
    let foreign_document = self::document(&mut foreign);
    assert_eq!(
        dom.take_fullscreen_events(foreign_document, 0, &AtomicBool::new(false))
            .unwrap_err(),
        DomError::ForeignOwner
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}
