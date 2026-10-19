//! Original-owner tests. No native focusability, selection service reply or
//! externally completed reset is supplied by these fixtures.
use super::*;
use crate::{DomDesignModeRangeDisposition as Disposition, DomSourceDesignModeStep as Step};

fn policy(outputs: usize, phases: u64) -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, phases, outputs, 0, 1)
}
fn fixture(root: bool, range: bool) -> (Dom, DocumentId, Option<crate::DomRangeId>) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            32,
            100_000,
            64,
            0,
            0,
            16,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_ranges(4)
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let comment = batch.create_comment(&document, "a real boundary offset");
    batch.insert_before(&comment, &document, None);
    if root {
        let element = batch.create_element(&document, "Root");
        batch.insert_before(&element, &document, None);
    }
    let ticket = range.then(|| {
        let range = batch.create_range(&document);
        batch.set_range_start(&range, &document, 1);
        batch.set_range_end(&range, &document, 1);
        batch.set_active_range(&document, Some((&range).into()));
        range
    });
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let range = ticket.map(|ticket| commit.created_range(&ticket).unwrap().unwrap());
    (dom, document, range)
}

#[test]
fn completed_local_design_mode_frame_and_range_result_survive_final_output_refusal() {
    // The independent output domains are flag, and (when dirty) one whole Range
    // boundary record. No incomplete marker/extra prefix or identity is needed.
    for with_range in [false, true] {
        let cancel = AtomicBool::new(false);
        let (mut dom, document, range) = fixture(false, with_range);
        let before = dom.snapshot();
        let expected = range.map_or(Disposition::NoActiveRange, Disposition::RangeReset);
        let outputs = 1 + usize::from(with_range);
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_design_mode(document, "oN"),
                policy(outputs - 1, 1),
                &cancel,
            )
            .unwrap();
        let operation = lease.operation();
        assert!(matches!(
            lease.advance(&cancel),
            Err(DomError::SourceOutputLimit)
        ));
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        assert_eq!(lease.snapshot().incomplete_source_operation(), None);
        let op = lease.dom.source_operation.as_ref().unwrap();
        assert_eq!(op.phases, 0);
        assert!(!op.published);
        let Some(crate::store::method::Frame::DesignMode(frame)) = op.draft.method.as_ref() else {
            panic!("the actual completed original method survives refusal");
        };
        assert_eq!(
            frame.cursor.action(),
            surgeist_html::HtmlDesignModeAction::Complete
        );
        assert_eq!(frame.range, Some(expected));
        assert!(frame.focusing.is_none());
        if let Some(range) = range {
            let record = op.draft.range_record(range).unwrap();
            assert_eq!(record.start.container, frame.document);
            assert_eq!(record.start.offset, 0);
            assert_eq!(record.end.container, frame.document);
            assert_eq!(record.end.offset, 0);
        }
        // A cancelled poll does not consume or rewrite the completed result.
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(lease.advance(&cancel), Err(DomError::Cancelled)));
        cancel.store(false, Ordering::Relaxed);
        // Owner-only lifetime fixture: reborrow the original live loan through
        // the REAL resume/policy validation, without dropping its unpublished
        // owner. Public drop of such a loan intentionally discards it instead.
        let mut retry = lease
            .dom
            .resume_source_edit(operation, policy(outputs, 1), &cancel)
            .unwrap();
        let DomSourceProgress::Complete(done) = retry.advance(&cancel).unwrap() else {
            panic!("local null-root setter completes");
        };
        assert_eq!(done.operation(), operation);
        assert_eq!(done.design_mode_range_disposition(), Some(expected));
        assert_eq!(done.commit().changes().changes().len(), outputs);
        assert!(
            done.commit()
                .changes()
                .changes()
                .contains(&DomChange::DocumentDesignMode(document))
        );
        if let Some(range) = range {
            assert!(
                done.commit()
                    .changes()
                    .changes()
                    .contains(&DomChange::RangeBoundary(range))
            );
        }
        assert_eq!(done.commit().changes().from(), before.stamp());
        assert_eq!(
            done.commit().changes().to().revision().get(),
            before.stamp().revision().get() + 1
        );
        assert!(
            !before
                .document(document)
                .unwrap()
                .metadata()
                .design_mode_enabled()
        );
    }
}

#[test]
fn design_mode_reserves_only_actual_prefix_and_final_revisions_before_real_focus_wait() {
    for (available, phases, expected_refusal) in
        [(1, 2, Some(false)), (2, 1, Some(true)), (2, 2, None)]
    {
        let cancel = AtomicBool::new(false);
        let (mut dom, document, _) = fixture(true, false);
        // Canonical checked-counter boundary fixture; no native Window or
        // focus fact is claimed. Actual original Source executes every guard.
        dom.revision = DomRevision(u64::MAX - available);
        let before = dom.snapshot();
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_design_mode(document, "on"),
                policy(64, phases),
                &cancel,
            )
            .unwrap();
        let result = lease.advance(&cancel);
        if let Some(phase_refusal) = expected_refusal {
            if phase_refusal {
                assert!(matches!(result, Err(DomError::SourcePublicationLimit)));
            } else {
                assert!(matches!(result, Err(DomError::RevisionExhausted)));
            }
            assert_eq!(lease.snapshot().stamp(), before.stamp());
            assert!(
                !lease
                    .snapshot()
                    .document(document)
                    .unwrap()
                    .metadata()
                    .design_mode_enabled()
            );
        } else {
            let DomSourceProgress::Request(DomSourceRequest::DesignMode(request)) = result.unwrap()
            else {
                panic!("actual original focusing requirement");
            };
            assert_eq!(
                request.step(),
                Step::Focusing(surgeist_html::HtmlFocusingAction::ReadInitialTargetFocusableArea)
            );
            assert_eq!(
                request.range_disposition(),
                Some(Disposition::NoActiveRange)
            );
            assert_eq!(lease.status().published_phases(), 1);
            assert_eq!(
                lease
                    .dom
                    .source_operation
                    .as_ref()
                    .unwrap()
                    .headroom
                    .as_ref()
                    .unwrap()
                    .remaining_phases,
                1
            );
            assert_eq!(
                lease.snapshot().stamp().revision(),
                DomRevision(u64::MAX - 1)
            );
        }
    }
}

#[test]
fn local_null_root_design_mode_uses_only_its_final_revision_and_phase() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, _) = fixture(false, false);
    dom.revision = DomRevision(u64::MAX - 1);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_design_mode(document, "on"),
            policy(1, 1),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("no synthetic enabled-only prefix");
    };
    assert_eq!(
        done.design_mode_range_disposition(),
        Some(Disposition::NoActiveRange)
    );
    assert_eq!(
        done.commit().changes().changes(),
        &[DomChange::DocumentDesignMode(document)]
    );
    assert_eq!(lease.snapshot().stamp().revision(), DomRevision(u64::MAX));
}
