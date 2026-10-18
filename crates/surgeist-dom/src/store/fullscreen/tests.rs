//! Owning unavailable-history, raw/pending and original checkpoint boundaries.
//! Private structural inputs do not certify popover or platform execution.
use super::*;
use crate::dialog::DocumentInteractiveState;
use crate::{DomSourceEdit, DomSourceLimits, DomTopLayerUaRule, Namespace};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn policy(outputs: usize, phases: u64) -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 100_000_000, phases, outputs, 0, 1)
}
fn fixture() -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom =
        Dom::new(DomLimits::new(64, 100_000, 128, 128, 128, 16, 1_000_000).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let first = batch.create_element_ns(&document, Namespace::none(), "first");
    let other = batch.create_element_ns(&document, Namespace::html(), "div");
    let commit = dom.apply(batch, &cancel()).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&first).unwrap().unwrap(),
        commit.created(&other).unwrap().unwrap(),
    )
}
fn draft(dom: &Dom) -> Draft {
    let batch = DomBatch::new();
    Draft::new_source(
        dom,
        batch.owner.clone(),
        &batch.commands,
        policy(128, 16),
        &cancel(),
    )
    .unwrap()
    .0
}
fn publish(dom: &mut Dom, draft: &Draft) {
    let prepared = draft.finish(dom, &cancel()).unwrap();
    dom.publish(prepared, &cancel()).unwrap();
}

#[test]
fn unavailable_document_lists_refuse_the_first_read_without_flag_or_layer_publication() {
    let (mut dom, document, first, _) = fixture();
    let mut staged = draft(&dom);
    let document_key = staged.resolve(document.into()).unwrap();
    // Actual typed unknown-history input; no showing popover is invented.
    staged
        .with_node_fields(document_key, &cancel(), |fields| {
            fields.interactive_document = Some(DocumentInteractiveState::Unavailable)
        })
        .unwrap();
    publish(&mut dom, &staged);
    let before = dom.snapshot();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::fullscreen_element_model(first),
            policy(128, 1),
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        lease.advance(&cancel()).unwrap_err(),
        DomError::FullscreenModelUnavailable {
            step: Step::ReadTopLayer
        }
    );
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert!(
        !lease
            .snapshot()
            .node(first)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert_eq!(
        before
            .document(document)
            .unwrap()
            .fullscreen_model_element()
            .unwrap_err(),
        DomError::DialogDocumentStateUnavailable
    );
}

#[test]
fn pending_flagged_member_counts_for_simple_document_while_unflagged_raw_members_survive_unfullscreen()
 {
    let (mut dom, document, first, other) = fixture();
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(first);
    dom.apply(batch, &cancel()).unwrap();
    let mut staged = draft(&dom);
    let first_key = staged.resolve(first.into()).unwrap();
    let other_key = staged.resolve(other.into()).unwrap();
    // Exercise canonical list primitives, without claiming ShowModal/Close ran.
    staged.add_top_layer(other_key, &cancel()).unwrap();
    staged
        .request_top_layer_removal(first_key, &cancel())
        .unwrap();
    publish(&mut dom, &staged);
    let before = dom.snapshot();
    let view = before.document(document).unwrap();
    assert_eq!(view.fullscreen_model_element().unwrap(), Some(first));
    assert!(view.is_simple_fullscreen_document().unwrap());
    assert_eq!(
        view.top_layer().unwrap().ua_rules().collect::<Vec<_>>(),
        [
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(other)
        ]
    );
    let mut batch = DomBatch::new();
    batch.unfullscreen_document_model(document);
    dom.apply(batch, &cancel()).unwrap();
    let after = dom.snapshot();
    let view = after.document(document).unwrap();
    assert_eq!(view.top_layer().unwrap().raw_members(), [other]);
    assert!(view.top_layer().unwrap().pending_removals().is_empty());
    assert!(!view.fullscreen().unwrap());
    assert!(!view.is_simple_fullscreen_document().unwrap());
    assert!(
        before
            .node(first)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
}

#[test]
fn checkpoint_restores_model_flags_and_lists_after_the_real_unfullscreen_command() {
    let (mut dom, document, first, _) = fixture();
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(first);
    dom.apply(batch, &cancel()).unwrap();
    let mut staged = draft(&dom);
    let first_key = staged.resolve(first.into()).unwrap();
    let document_key = staged.resolve(document.into()).unwrap();
    let checkpoint = staged.source_checkpoint(0, 0, &cancel()).unwrap();
    staged
        .unfullscreen_element_model(first_key, &cancel())
        .unwrap();
    assert!(!staged.element(first_key).unwrap().fullscreen_flag());
    assert!(
        staged
            .fullscreen_raw_members(document_key, &cancel())
            .unwrap()
            .is_empty()
    );
    checkpoint.restore(&mut staged);
    assert!(staged.element(first_key).unwrap().fullscreen_flag());
    assert_eq!(
        staged
            .fullscreen_raw_members(document_key, &cancel())
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        vec![first_key]
    );
    assert_eq!(staged.select_temporary_bytes.load(Ordering::Relaxed), 0);
    publish(&mut dom, &staged);
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .fullscreen_model_element()
            .unwrap(),
        Some(first)
    );
}
