//! Actual designMode results and unresolved focusing through the original
//! Session Source driver, without acknowledging an unavailable focus service.
use std::sync::atomic::AtomicBool;

use surgeist_dom::*;
use surgeist_html::HtmlFocusingAction;
use surgeist_runtime::*;

fn limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 100_000_000, 64, 1024, 128, 8)
}

fn session() -> DomSession {
    DomSession::new(
        Dom::new(
            DomLimits::new(
                128,
                100_000,
                256,
                128,
                128,
                32,
                4_000_000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_ranges(8)
            .unwrap()
            .with_dispatch_custody(
                DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
            ),
        )
        .unwrap(),
        DomSessionLimits {
            source_events: DomSourceEventRuntimeLimits {
                jobs: 4,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 4,
                completions: 8,
                reports: 8,
                storage_bytes: 16_000_000,
                work: 1_000_000,
                css_input: surgeist_css::CssUtf16InputLimits::default(),
            },
            element_tasks: DomElementTaskRuntimeLimits {
                tasks: 8,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            hosted: DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(8, 8, 8, 8, 8, 8, 8),
                    16_000_000,
                    16_000_000,
                    1_000_000,
                ),
                tasks: 8,
                storage_bytes: 16_000_000,
                scratch_bytes: 16_000_000,
                work: 1_000_000,
            },
        },
    )
    .unwrap()
}

fn apply(session: &mut DomSession, document: DocumentId) -> DomSourceDriveProgress {
    session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::set_design_mode(document, "oN"),
            limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &AtomicBool::new(false),
        )
        .unwrap()
}

fn resume(
    session: &mut DomSession,
    operation: DomSourceOperationId,
    cancel: &AtomicBool,
) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
    session.source_execution().resume(
        &mut session.dom(),
        operation,
        limits(),
        &mut DomSourceServices {
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        cancel,
    )
}

fn bounds(snapshot: &DomSnapshot, range: DomRangeId) -> ((NodeId, usize), (NodeId, usize)) {
    let view = snapshot.range(range).unwrap();
    (
        (view.start().container(), view.start().offset()),
        (view.end().container(), view.end().offset()),
    )
}

fn pending(progress: DomSourceDriveProgress) -> DomSourceDesignModeRequest {
    match progress {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::DesignMode(request),
            reason: DomSourceUnavailable::DesignMode(step),
        } => {
            assert_eq!(step, request.step());
            assert_eq!(
                step,
                DomSourceDesignModeStep::Focusing(
                    HtmlFocusingAction::ReadInitialTargetFocusableArea
                )
            );
            request
        }
        other => panic!("actual designMode focusing requirement must remain pending: {other:?}"),
    }
}

#[test]
fn null_document_element_completes_with_actual_no_active_range_result() {
    let mut session = session();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let document = session
        .dom()
        .apply(batch, &AtomicBool::new(false))
        .unwrap()
        .document(&document)
        .unwrap();
    let before = session.snapshot();
    assert_eq!(before.document(document).unwrap().document_element(), None);
    assert_eq!(
        before.document(document).unwrap().active_range(),
        DomActiveRangeState::Known(None)
    );
    let DomSourceDriveProgress::Complete(done) = apply(&mut session, document) else {
        panic!("null-root native setter has no unavailable service");
    };
    assert_eq!(
        done.design_mode_range_disposition(),
        Some(DomDesignModeRangeDisposition::NoActiveRange)
    );
    assert_eq!(
        done.commit().changes().changes(),
        [DomChange::DocumentDesignMode(document)]
    );
    let after = session.snapshot();
    assert!(
        after
            .document(document)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
    assert!(
        !before
            .document(document)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
    assert_eq!(
        after.document(document).unwrap().active_range(),
        DomActiveRangeState::Known(None)
    );
    assert_eq!(session.dom().usage().ranges(), 0);
    assert!(session.dom().pending_source_edit().is_none());
    assert!(after.incomplete_source_operation().is_none());
}

#[test]
fn null_document_element_completes_with_exact_associated_range_reset_result() {
    let mut session = session();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let text = batch.create_text(&document, "A😀BC");
    let range = batch.create_range(&document);
    // Initial association requires the actual ordinary Document root. Later
    // setters preserve that same association when endpoints become detached.
    batch.set_active_range(&document, Some((&range).into()));
    batch.set_range_start(&range, &text, 1);
    batch.set_range_end(&range, &text, 4);
    let commit = session.dom().apply(batch, &AtomicBool::new(false)).unwrap();
    let document = commit.document(&document).unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let range = commit.created_range(&range).unwrap().unwrap();
    let before = session.snapshot();
    let DomSourceDriveProgress::Complete(done) = apply(&mut session, document) else {
        panic!("actual local range reset and null-root focus must complete");
    };
    assert_eq!(
        done.design_mode_range_disposition(),
        Some(DomDesignModeRangeDisposition::RangeReset(range))
    );
    assert_eq!(done.commit().changes().changes().len(), 2);
    assert!(
        done.commit()
            .changes()
            .changes()
            .contains(&DomChange::RangeBoundary(range))
    );
    assert!(
        done.commit()
            .changes()
            .changes()
            .contains(&DomChange::DocumentDesignMode(document))
    );
    let after = session.snapshot();
    assert!(
        after
            .document(document)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
    assert_eq!(bounds(&before, range), ((text, 1), (text, 4)));
    assert_eq!(
        bounds(&after, range),
        ((document.node(), 0), (document.node(), 0))
    );
    assert_eq!(
        after.document(document).unwrap().active_range(),
        DomActiveRangeState::Known(Some(range))
    );
    assert_eq!(session.dom().usage().ranges(), 1);
    assert!(session.dom().pending_source_edit().is_none());
    assert!(after.incomplete_source_operation().is_none());
}

#[test]
fn document_element_focus_preserves_exact_pending_request_and_published_reset_across_reacquisition()
{
    let mut session = session();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::none(), "root");
    let text = batch.create_text(&document, "A😀BC");
    batch.insert_before(&text, &root, None);
    batch.insert_before(&root, &document, None);
    let range = batch.create_range(&document);
    batch.set_range_start(&range, &text, 1);
    batch.set_range_end(&range, &text, 4);
    batch.set_active_range(&document, Some((&range).into()));
    let commit = session.dom().apply(batch, &AtomicBool::new(false)).unwrap();
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let range = commit.created_range(&range).unwrap().unwrap();
    let before = session.snapshot();

    // apply()'s execution permission and lease have both been released here.
    let request = pending(apply(&mut session, document));
    assert_eq!(request.document(), document);
    assert_eq!(request.focus_target(), Some(root));
    assert_eq!(request.command(), 0);
    assert_eq!(
        request.range_disposition(),
        Some(DomDesignModeRangeDisposition::RangeReset(range))
    );
    let prefix = session.snapshot();
    assert!(
        prefix
            .document(document)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
    assert_eq!(
        bounds(&prefix, range),
        ((document.node(), 0), (document.node(), 0))
    );
    assert_eq!(
        prefix.incomplete_source_operation(),
        Some(request.operation())
    );
    assert_eq!(
        prefix.document(document).unwrap().active_range(),
        DomActiveRangeState::Known(Some(range))
    );
    let status = session.dom().pending_source_edit().unwrap();
    assert_eq!(status.operation(), request.operation());
    assert_eq!(status.phase(), DomSourcePhase::DesignModeService);
    assert!(status.has_published());

    let mut illegal = DomBatch::new();
    illegal.retire_range(range);
    assert_eq!(
        session
            .dom()
            .apply(illegal, &AtomicBool::new(false))
            .unwrap_err(),
        DomError::SourceOperationPending
    );
    assert!(matches!(
        resume(&mut session, request.operation(), &AtomicBool::new(true)),
        Err(DomSourceRuntimeError::Dom(DomError::Cancelled))
    ));
    assert_eq!(session.snapshot().stamp(), prefix.stamp());

    for _ in 0..2 {
        let actual =
            pending(resume(&mut session, request.operation(), &AtomicBool::new(false)).unwrap());
        assert_eq!(actual, request);
        assert_eq!(session.snapshot().stamp(), prefix.stamp());
        assert_eq!(
            session
                .dom()
                .pending_source_edit()
                .unwrap()
                .published_phases(),
            status.published_phases()
        );
        assert_eq!(
            bounds(&session.snapshot(), range),
            ((document.node(), 0), (document.node(), 0))
        );
        assert_eq!(session.dom().usage().ranges(), 1);
    }
    assert!(
        !before
            .document(document)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
    assert_eq!(bounds(&before, range), ((text, 1), (text, 4)));
    assert_eq!(before.incomplete_source_operation(), None);
    // The real generic focusing service is unavailable; this operation remains
    // pending until whole-session disposal rather than inventing a receipt.
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        request.operation()
    );
}
