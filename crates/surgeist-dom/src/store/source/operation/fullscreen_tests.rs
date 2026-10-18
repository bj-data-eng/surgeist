//! Final-publication retry through the original live Source loan. Public Drop
//! intentionally discards a wholly unpublished operation without observations.
use super::*;
use crate::Namespace;

fn policy(outputs: usize) -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 100_000_000, 1, outputs, 0, 1)
}

#[test]
fn local_model_admits_only_its_final_publication_and_preserves_command_after_output_refusal() {
    let cancel = AtomicBool::new(false);
    let mut dom =
        Dom::new(DomLimits::new(64, 100_000, 128, 128, 128, 16, 1_000_000).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let first = batch.create_element_ns(&document, Namespace::none(), "first");
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let first = commit.created(&first).unwrap().unwrap();
    dom.revision = DomRevision(u64::MAX - 1);
    let before = dom.snapshot();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::fullscreen_element_model(first),
            policy(2),
            &cancel,
        )
        .unwrap();
    let operation = lease.operation();
    assert_eq!(
        lease.advance(&cancel).unwrap_err(),
        DomError::SourceOutputLimit
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
    assert!(
        lease
            .snapshot()
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );
    assert_eq!(lease.snapshot().incomplete_source_operation(), None);
    // Reborrow through real resume/policy validation while keeping the original
    // unpublished loan alive. Dropping that loan would intentionally discard it.
    let mut retry = lease
        .dom
        .resume_source_edit(operation, policy(3), &cancel)
        .unwrap();
    let DomSourceProgress::Complete(done) = retry.advance(&cancel).unwrap() else {
        panic!("local model has no external acknowledgment");
    };
    assert_eq!(done.operation(), operation);
    assert_eq!(done.commit().changes().changes().len(), 3);
    assert!(
        done.commit()
            .changes()
            .changes()
            .contains(&DomChange::ElementState(first))
    );
    assert!(
        done.commit()
            .changes()
            .changes()
            .contains(&DomChange::DocumentInteractiveState(document))
    );
    assert!(
        done.commit()
            .changes()
            .changes()
            .contains(&DomChange::DocumentFullscreenState(document))
    );
    assert_eq!(retry.snapshot().stamp().revision(), DomRevision(u64::MAX));
    assert_eq!(retry.snapshot().incomplete_source_operation(), None);
}
