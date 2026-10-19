//! HTML877d raw61117–61130: exact keyword interpretation, enable, actual
//! association reset, then the current document element's genuine focusing call.
//! #1243 selects an explicit no-active-Range branch, without creating a Range.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomActiveRangeState, DomBatch, DomChange, DomCommit,
    DomDesignModeRangeDisposition, DomError, DomLimits, DomRangeId, DomSnapshot,
    DomSourceCompletion, DomSourceDesignModeStep, DomSourceEdit, DomSourceLimits, DomSourcePhase,
    DomSourceProgress, DomSourceRequest, Namespace, NodeId,
};

fn limits() -> DomLimits {
    DomLimits::new(
        256,
        100_000,
        1024,
        1024,
        1024,
        64,
        4_000_000,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_ranges(16)
    .unwrap()
}
fn owner() -> Dom {
    Dom::new(limits()).unwrap()
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
fn enabled(snapshot: &DomSnapshot, document: DocumentId) -> bool {
    snapshot
        .document(document)
        .unwrap()
        .metadata()
        .design_mode_enabled()
}
fn bounds(snapshot: &DomSnapshot, range: DomRangeId) -> ((NodeId, usize), (NodeId, usize)) {
    let view = snapshot.range(range).unwrap();
    assert_eq!(view.id(), range);
    (
        (view.start().container(), view.start().offset()),
        (view.end().container(), view.end().offset()),
    )
}
fn active(snapshot: &DomSnapshot, document: DocumentId) -> Option<DomRangeId> {
    match snapshot.document(document).unwrap().active_range() {
        DomActiveRangeState::Known(range) => range,
        DomActiveRangeState::Unavailable => {
            panic!("native constructor supplies actual empty selection")
        }
    }
}
fn complete(dom: &mut Dom, edit: DomSourceEdit) -> DomSourceCompletion {
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .begin_source_edit(edit, source_limits(), &cancel)
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("this setter has no reached unavailable dependency");
    };
    assert_eq!(done.output(), None);
    done
}
fn source_set(
    dom: &mut Dom,
    document: DocumentId,
    value: impl Into<DocumentString>,
) -> DomSourceCompletion {
    complete(dom, DomSourceEdit::set_design_mode(document, value))
}
fn associated_range(dom: &mut Dom, document: DocumentId) -> DomRangeId {
    let mut batch = DomBatch::new();
    let ticket = batch.create_range(document);
    batch.set_active_range(document, Some((&ticket).into()));
    apply(dom, batch).created_range(&ticket).unwrap().unwrap()
}

#[test]
fn native_xml_null_root_and_known_null_range_complete_without_creating_selection() {
    for source in [false, true] {
        let mut dom = owner();
        let document = document(&mut dom);
        let before = dom.snapshot();
        let usage = dom.usage();
        assert_eq!(before.document(document).unwrap().document_element(), None);
        assert_eq!(active(&before, document), None);
        assert!(!enabled(&before, document));
        if source {
            let done = source_set(&mut dom, document, "oN");
            assert_eq!(
                done.design_mode_range_disposition(),
                Some(DomDesignModeRangeDisposition::NoActiveRange)
            );
            assert_eq!(
                done.commit().changes().changes(),
                [DomChange::DocumentDesignMode(document)]
            );
        } else {
            let mut batch = DomBatch::new();
            batch.set_design_mode(document, "oN");
            let commit = apply(&mut dom, batch);
            assert_eq!(
                commit.changes().changes(),
                [DomChange::DocumentDesignMode(document)]
            );
        }
        let after = dom.snapshot();
        assert!(enabled(&after, document));
        assert_eq!(active(&after, document), None);
        assert_eq!(dom.usage().ranges(), usage.ranges());
        assert!(!enabled(&before, document));
        assert_eq!(active(&before, document), None);
        assert!(dom.pending_source_edit().is_none());
        assert!(after.incomplete_source_operation().is_none());
    }
}

#[test]
fn exact_ascii_keywords_do_not_trim_and_invalid_values_preserve_enabled_state() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let document = document(&mut dom);
    for keyword in ["on", "On", "oN", "ON"] {
        source_set(&mut dom, document, "off");
        let before = dom.snapshot();
        let done = source_set(&mut dom, document, keyword);
        assert_eq!(
            done.design_mode_range_disposition(),
            Some(DomDesignModeRangeDisposition::NoActiveRange)
        );
        assert!(enabled(&dom.snapshot(), document));
        assert!(!enabled(&before, document));
    }
    let invalid = [
        "", " on", "on ", "\ton", "off\n", "ON\0", "ＯＮ", "true", "none",
    ];
    for initial in [false, true] {
        source_set(&mut dom, document, if initial { "on" } else { "off" });
        for value in invalid
            .iter()
            .map(|value| DocumentString::from(*value))
            .chain([
                DocumentString::from_utf16(vec![0xd800]),
                DocumentString::from_utf16(vec![0x6f, 0xdc00]),
            ])
        {
            let before = dom.snapshot();
            let done = source_set(&mut dom, document, value.clone());
            assert_eq!(done.design_mode_range_disposition(), None);
            assert!(done.commit().changes().changes().is_empty());
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(enabled(&dom.snapshot(), document), initial);
            let mut batch = DomBatch::new();
            batch.set_design_mode(document, value);
            assert!(
                dom.apply(batch, &cancel)
                    .unwrap()
                    .changes()
                    .changes()
                    .is_empty()
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
        }
    }
    for keyword in ["off", "Off", "oFf", "OFF"] {
        source_set(&mut dom, document, "on");
        let done = source_set(&mut dom, document, keyword);
        assert_eq!(done.design_mode_range_disposition(), None);
        assert!(!enabled(&dom.snapshot(), document));
        let before = dom.snapshot();
        let done = source_set(&mut dom, document, keyword);
        assert!(done.commit().changes().changes().is_empty());
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
}

#[test]
fn reset_reuses_exact_range_after_root_changes_and_preserves_every_document_association() {
    for source in [false, true] {
        let mut dom = owner();
        let document = document(&mut dom);
        let other = self::document(&mut dom);
        let range = associated_range(&mut dom, document);
        let unrelated = associated_range(&mut dom, other);
        let mut batch = DomBatch::new();
        batch.set_range_end(range, other.node(), 0);
        batch.set_active_range(other, Some(range.into()));
        let detached = batch.create_text(other, "A😀BC");
        batch.set_range_start(range, &detached, 1);
        batch.set_range_end(range, &detached, 4);
        let commit = apply(&mut dom, batch);
        let detached = commit.created(&detached).unwrap().unwrap();
        let before = dom.snapshot();
        let count = dom.usage().ranges();
        assert_eq!(active(&before, document), Some(range));
        assert_eq!(active(&before, other), Some(range));
        assert_eq!(bounds(&before, range), ((detached, 1), (detached, 4)));
        assert_eq!(before.node(detached).unwrap().parent(), None);
        let changes = if source {
            let done = source_set(&mut dom, document, "on");
            assert_eq!(
                done.design_mode_range_disposition(),
                Some(DomDesignModeRangeDisposition::RangeReset(range))
            );
            done.commit().changes().changes().to_vec()
        } else {
            let mut batch = DomBatch::new();
            batch.set_design_mode(document, "on");
            apply(&mut dom, batch).changes().changes().to_vec()
        };
        assert!(changes.contains(&DomChange::DocumentDesignMode(document)));
        assert!(changes.contains(&DomChange::RangeBoundary(range)));
        assert_eq!(changes.len(), 2);
        let after = dom.snapshot();
        assert!(enabled(&after, document));
        assert!(!enabled(&after, other));
        assert_eq!(
            bounds(&after, range),
            ((document.node(), 0), (document.node(), 0))
        );
        assert!(after.range(range).unwrap().collapsed());
        assert_eq!(active(&after, document), Some(range));
        assert_eq!(active(&after, other), Some(range));
        assert_eq!(
            bounds(&after, unrelated),
            ((other.node(), 0), (other.node(), 0))
        );
        assert_eq!(dom.usage().ranges(), count);
        assert_eq!(bounds(&before, range), ((detached, 1), (detached, 4)));
        assert!(!enabled(&before, document));
        assert!(dom.pending_source_edit().is_none());
    }
}

#[test]
fn already_on_and_off_do_not_reset_a_subsequently_moved_associated_range() {
    let mut dom = owner();
    let document = document(&mut dom);
    let range = associated_range(&mut dom, document);
    assert_eq!(
        source_set(&mut dom, document, "on").design_mode_range_disposition(),
        Some(DomDesignModeRangeDisposition::RangeReset(range))
    );
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "moved");
    batch.set_range_start(range, &text, 1);
    batch.set_range_end(range, &text, 4);
    let text = apply(&mut dom, batch).created(&text).unwrap().unwrap();
    let before = dom.snapshot();
    let done = source_set(&mut dom, document, "ON");
    assert_eq!(done.design_mode_range_disposition(), None);
    assert!(done.commit().changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(bounds(&dom.snapshot(), range), ((text, 1), (text, 4)));
    assert_eq!(
        source_set(&mut dom, document, "OFF").design_mode_range_disposition(),
        None
    );
    assert!(!enabled(&dom.snapshot(), document));
    assert_eq!(bounds(&dom.snapshot(), range), ((text, 1), (text, 4)));
    assert_eq!(active(&dom.snapshot(), document), Some(range));
    assert!(enabled(&before, document));
}

fn rooted(dom: &mut Dom) -> (DocumentId, NodeId, NodeId, DomRangeId) {
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
    let commit = apply(dom, batch);
    (
        commit.document(&document).unwrap(),
        commit.created(&root).unwrap().unwrap(),
        commit.created(&text).unwrap().unwrap(),
        commit.created_range(&range).unwrap().unwrap(),
    )
}

#[test]
fn document_element_focus_is_a_real_atomic_refusal_without_partial_flag_or_range_reset() {
    use surgeist_html::HtmlFocusingAction;
    let mut dom = owner();
    let (document, root, text, range) = rooted(&mut dom);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(root, "data-staged", "must roll back");
    batch.set_design_mode(document, "on");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::DesignModeProcessingUnavailable {
            step: DomSourceDesignModeStep::Focusing(
                HtmlFocusingAction::ReadInitialTargetFocusableArea
            ),
        }
    );
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(!enabled(&after, document));
    assert_eq!(bounds(&after, range), ((text, 1), (text, 4)));
    assert_eq!(active(&after, document), Some(range));
    assert!(
        after
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .is_empty()
    );
    assert_eq!(after.node(root).unwrap().parent(), Some(document.node()));
    assert!(dom.pending_source_edit().is_none());
    // Off and invalid inputs never enter the element's focus dependency.
    for value in ["off", " on"] {
        assert_eq!(
            source_set(&mut dom, document, value).design_mode_range_disposition(),
            None
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(bounds(&dom.snapshot(), range), ((text, 1), (text, 4)));
    }
}

#[test]
fn published_focus_prefix_retains_exact_reset_and_called_stage_across_drop_cancel_and_resume() {
    use surgeist_html::HtmlFocusingAction;
    let mut dom = owner();
    let (document, root, text, range) = rooted(&mut dom);
    let before = dom.snapshot();
    let cancel = AtomicBool::new(false);
    let operation;
    let request;
    let prefix;
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_design_mode(document, "on"),
                source_limits(),
                &cancel,
            )
            .unwrap();
        operation = lease.operation();
        let DomSourceProgress::Request(DomSourceRequest::DesignMode(actual)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("real document-element focus dependency");
        };
        assert_eq!(actual.operation(), operation);
        assert_eq!(actual.document(), document);
        assert_eq!(
            actual.step(),
            DomSourceDesignModeStep::Focusing(HtmlFocusingAction::ReadInitialTargetFocusableArea)
        );
        assert_eq!(actual.focus_target(), Some(root));
        assert_eq!(
            actual.range_disposition(),
            Some(DomDesignModeRangeDisposition::RangeReset(range))
        );
        assert_eq!(actual.command(), 0);
        assert_eq!(lease.status().phase(), DomSourcePhase::DesignModeService);
        assert!(lease.status().has_published());
        prefix = lease.snapshot();
        assert!(enabled(&prefix, document));
        assert_eq!(
            bounds(&prefix, range),
            ((document.node(), 0), (document.node(), 0))
        );
        assert_eq!(active(&prefix, document), Some(range));
        assert_eq!(prefix.incomplete_source_operation(), Some(operation));
        request = actual;
    }
    assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
    let mut illegal = DomBatch::new();
    illegal.retire_range(range);
    assert_eq!(
        dom.apply(illegal, &cancel).unwrap_err(),
        DomError::SourceOperationPending
    );
    assert_eq!(
        bounds(&dom.snapshot(), range),
        ((document.node(), 0), (document.node(), 0))
    );
    {
        let mut lease = dom
            .resume_source_edit(operation, source_limits(), &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::DesignMode(actual)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("already-enabled flag must not restart or complete the retained method");
        };
        assert_eq!(actual, request);
        assert_eq!(lease.snapshot().stamp(), prefix.stamp());
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(lease.advance(&cancel).unwrap_err(), DomError::Cancelled);
        assert_eq!(lease.snapshot().stamp(), prefix.stamp());
        cancel.store(false, Ordering::Relaxed);
        lease.request_cancel();
        let DomSourceProgress::Request(DomSourceRequest::DesignMode(actual)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("cancellation does not discharge the called focus obligation");
        };
        assert_eq!(actual, request);
        assert_eq!(lease.snapshot().stamp(), prefix.stamp());
        assert_eq!(
            bounds(&lease.snapshot(), range),
            ((document.node(), 0), (document.node(), 0))
        );
    }
    assert!(!enabled(&before, document));
    assert_eq!(bounds(&before, range), ((text, 1), (text, 4)));
    assert_eq!(before.incomplete_source_operation(), None);
    assert_eq!(dom.usage().ranges(), 1);
    // Generic positive eligibility is unavailable. Whole-owner disposal releases
    // this genuinely pending method; no successful focus receipt is fabricated.
    assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
}

#[test]
fn known_null_selection_skips_only_reset_and_still_calls_current_document_element_focus() {
    use surgeist_html::HtmlFocusingAction;
    let mut dom = owner();
    let (document, root, _, range) = rooted(&mut dom);
    let mut batch = DomBatch::new();
    batch.retire_range(range);
    apply(&mut dom, batch);
    let before = dom.snapshot();
    assert_eq!(active(&before, document), None);
    assert_eq!(dom.usage().ranges(), 0);
    let cancel = AtomicBool::new(false);
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_design_mode(document, "on"),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let operation = lease.operation();
        let DomSourceProgress::Request(DomSourceRequest::DesignMode(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("null selection cannot waive the nonnull root's focusing call");
        };
        assert_eq!(request.operation(), operation);
        assert_eq!(request.document(), document);
        assert_eq!(request.focus_target(), Some(root));
        assert_eq!(
            request.range_disposition(),
            Some(DomDesignModeRangeDisposition::NoActiveRange)
        );
        assert_eq!(
            request.step(),
            DomSourceDesignModeStep::Focusing(HtmlFocusingAction::ReadInitialTargetFocusableArea)
        );
        assert!(enabled(&lease.snapshot(), document));
        assert_eq!(active(&lease.snapshot(), document), None);
        assert_eq!(
            lease.snapshot().incomplete_source_operation(),
            Some(operation)
        );
    }
    assert_eq!(dom.usage().ranges(), 0);
    assert!(!enabled(&before, document));
}

fn parse_limits() -> surgeist_html_parser::HtmlParseLimits {
    surgeist_html_parser::HtmlParseLimits::builder()
        .source_units(4096)
        .buffered_utf16_units(4096)
        .token_attributes(1024)
        .auxiliary_entries(100_000)
        .managed_nodes(1024)
        .managed_edges(1024)
        .managed_attributes(1024)
        .managed_utf16_units(65536)
        .source_map_entries(65536)
        .source_map_payload_units(65536)
        .diagnostics(1024)
        .diagnostic_payload_units(65536)
        .context_effects(1024)
        .open_elements(64)
        .tree_depth(128)
        .managed_capacity_bytes(16_000_000)
        .peak_transient_bytes(16_000_000)
        .atomic_layout_bytes(16_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}
fn parsed_document(dom: &mut Dom) -> DocumentId {
    use surgeist_dom::{
        DomNativeInputPlatform, DomParsedDocumentEnvironment, DomParsedHtmlInstallLimits,
    };
    use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlSource};
    use surgeist_html_semantics::{
        HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
        HtmlBaseUrlScheme,
    };
    let native = document(dom);
    let before = dom.snapshot();
    let url = HtmlBaseUrlRecord::observed(
        HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1),
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let environment = DomParsedDocumentEnvironment::headless(
        before.document(native).unwrap().metadata().origin().clone(),
        url,
        HtmlBaseEncoding::fresh(),
        "UTF-8".into(),
        DomNativeInputPlatform::headless(),
    );
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><html><head></head><body></body></html>"),
            parse_limits(),
        )
        .parse()
        .unwrap(),
    );
    dom.install_parsed_html_document(
        before.stamp(),
        report,
        environment,
        DomParsedHtmlInstallLimits::new(1024, 10_000_000, 32_000_000, 100_000_000),
        &AtomicBool::new(false),
    )
    .unwrap()
    .document()
}

#[test]
fn real_parsed_unavailable_selection_is_not_null_even_after_actual_root_removal() {
    let mut dom = owner();
    let document = parsed_document(&mut dom);
    let root = dom
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut remove = DomBatch::new();
    remove.detach(root);
    apply(&mut dom, remove);
    let before = dom.snapshot();
    assert_eq!(
        before.document(document).unwrap().active_range(),
        DomActiveRangeState::Unavailable
    );
    assert_eq!(before.document(document).unwrap().document_element(), None);
    let mut atomic = DomBatch::new();
    atomic.set_design_mode(document, "on");
    assert_eq!(
        dom.apply(atomic, &AtomicBool::new(false)).unwrap_err(),
        DomError::DesignModeProcessingUnavailable {
            step: DomSourceDesignModeStep::ReadActiveRange
        }
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let cancel = AtomicBool::new(false);
    let operation;
    let request;
    let prefix;
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_design_mode(document, "on"),
                source_limits(),
                &cancel,
            )
            .unwrap();
        operation = lease.operation();
        let DomSourceProgress::Request(DomSourceRequest::DesignMode(actual)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("real imported selection availability must be supplied, never defaulted");
        };
        assert_eq!(actual.step(), DomSourceDesignModeStep::ReadActiveRange);
        assert_eq!(actual.document(), document);
        assert_eq!(actual.focus_target(), None);
        assert_eq!(actual.range_disposition(), None);
        prefix = lease.snapshot();
        assert!(enabled(&prefix, document));
        assert_eq!(
            prefix.document(document).unwrap().active_range(),
            DomActiveRangeState::Unavailable
        );
        assert_eq!(prefix.incomplete_source_operation(), Some(operation));
        request = actual;
    }
    {
        let mut lease = dom
            .resume_source_edit(operation, source_limits(), &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::DesignMode(actual)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("resume cannot treat enabled prefix as method completion");
        };
        assert_eq!(actual, request);
        assert_eq!(lease.snapshot().stamp(), prefix.stamp());
    }
    assert_eq!(dom.usage().ranges(), 0);
    assert!(!enabled(&before, document));
    assert_eq!(
        before.document(document).unwrap().active_range(),
        DomActiveRangeState::Unavailable
    );
    // Actual missing selection support remains a mandatory dependency. Dropping
    // the whole DOM is terminal disposal, not a claimed successful setter.
    assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
}

#[test]
fn explicit_native_null_association_makes_real_parsed_null_root_setter_complete() {
    let mut dom = owner();
    let document = parsed_document(&mut dom);
    let before = dom.snapshot();
    assert_eq!(
        before.document(document).unwrap().active_range(),
        DomActiveRangeState::Unavailable
    );
    let root = before
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    batch.detach(root);
    batch.set_active_range(document, None);
    apply(&mut dom, batch);
    assert_eq!(active(&dom.snapshot(), document), None);
    let done = source_set(&mut dom, document, "ON");
    assert_eq!(
        done.design_mode_range_disposition(),
        Some(DomDesignModeRangeDisposition::NoActiveRange)
    );
    assert!(enabled(&dom.snapshot(), document));
    assert_eq!(dom.usage().ranges(), 0);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(
        before.document(document).unwrap().active_range(),
        DomActiveRangeState::Unavailable
    );
}

#[test]
fn batch_reads_actual_association_after_earlier_commands_and_resets_only_that_record() {
    let mut dom = owner();
    let document = document(&mut dom);
    let original = associated_range(&mut dom, document);
    let mut batch = DomBatch::new();
    let replacement = batch.create_range(document);
    batch.set_active_range(document, Some((&replacement).into()));
    let text = batch.create_text(document, "latest");
    batch.set_range_start(&replacement, &text, 2);
    batch.set_range_end(&replacement, &text, 5);
    batch.set_design_mode(document, "on");
    let commit = apply(&mut dom, batch);
    let replacement = commit.created_range(&replacement).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    assert_ne!(replacement, original);
    let after = dom.snapshot();
    assert_eq!(active(&after, document), Some(replacement));
    assert_eq!(
        bounds(&after, replacement),
        ((document.node(), 0), (document.node(), 0))
    );
    assert_eq!(
        bounds(&after, original),
        ((document.node(), 0), (document.node(), 0))
    );
    assert_eq!(
        after.node(text).unwrap().data(),
        Some(&DocumentString::from("latest"))
    );
    source_set(&mut dom, document, "off");
    let mut batch = DomBatch::new();
    batch.set_range_start(original, text, 1);
    batch.set_range_end(original, text, 4);
    batch.set_active_range(document, None);
    batch.set_design_mode(document, "on");
    apply(&mut dom, batch);
    assert_eq!(active(&dom.snapshot(), document), None);
    assert_eq!(bounds(&dom.snapshot(), original), ((text, 1), (text, 4)));
    assert!(enabled(&dom.snapshot(), document));
}

#[test]
fn local_null_root_publication_has_exact_change_headroom_and_never_allocates_an_identity() {
    // One flag row; a dirty actual Range adds one whole-record boundary row.
    // There is no externally visible prefix, hence no incomplete marker row,
    // no called service, and no newly published identity.
    for with_range in [false, true] {
        let mut dom = owner();
        let document = document(&mut dom);
        let range = with_range.then(|| associated_range(&mut dom, document));
        let text = if let Some(range) = range {
            let mut batch = DomBatch::new();
            let text = batch.create_text(document, "dirty");
            batch.set_range_start(range, &text, 1);
            batch.set_range_end(range, &text, 3);
            Some(apply(&mut dom, batch).created(&text).unwrap().unwrap())
        } else {
            None
        };
        let before = dom.snapshot();
        let usage = dom.usage();
        let outputs = if with_range { 2 } else { 1 };
        let cancel = AtomicBool::new(false);
        let short = DomSourceLimits::new(32_000_000, 1_000_000, 100_000_000, 1, outputs - 1, 0, 1);
        let error = match dom.begin_source_edit(
            DomSourceEdit::set_design_mode(document, "on"),
            short,
            &cancel,
        ) {
            Err(error) => error,
            Ok(mut lease) => {
                let error = lease.advance(&cancel).unwrap_err();
                assert!(!lease.status().has_published());
                assert_eq!(lease.snapshot().stamp(), before.stamp());
                error
            }
        };
        assert_eq!(error, DomError::SourceOutputLimit);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(!enabled(&dom.snapshot(), document));
        if let (Some(range), Some(text)) = (range, text) {
            assert_eq!(bounds(&dom.snapshot(), range), ((text, 1), (text, 3)));
            assert_eq!(active(&dom.snapshot(), document), Some(range));
        }
        // A neverpublished lease has no externally incurred obligation; dropping
        // it discards the candidate. Retry is a fresh actual request, not a
        // fabricated resume authority or a budget refund on a retained frame.
        assert!(dom.pending_source_edit().is_none());
        let exact = DomSourceLimits::new(32_000_000, 1_000_000, 100_000_000, 1, outputs, 0, 1);
        {
            let mut lease = dom
                .begin_source_edit(
                    DomSourceEdit::set_design_mode(document, "on"),
                    exact,
                    &cancel,
                )
                .unwrap();
            let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
                panic!("exact local change headroom suffices");
            };
            assert_eq!(
                done.design_mode_range_disposition(),
                Some(match range {
                    Some(range) => DomDesignModeRangeDisposition::RangeReset(range),
                    None => DomDesignModeRangeDisposition::NoActiveRange,
                })
            );
            assert_eq!(done.output(), None);
            assert_eq!(done.commit().changes().changes().len(), outputs);
            assert!(
                done.commit()
                    .changes()
                    .changes()
                    .contains(&DomChange::DocumentDesignMode(document))
            );
        }
        assert!(enabled(&dom.snapshot(), document));
        assert_eq!(dom.usage().ranges(), usage.ranges());
        if let Some(range) = range {
            assert_eq!(
                bounds(&dom.snapshot(), range),
                ((document.node(), 0), (document.node(), 0))
            );
            assert_eq!(active(&dom.snapshot(), document), Some(range));
        }
        assert_eq!(before.incomplete_source_operation(), None);
        assert_eq!(dom.snapshot().incomplete_source_operation(), None);
        assert!(dom.pending_source_edit().is_none());
    }
}

#[test]
fn finite_storage_work_and_cancellation_refusals_preserve_real_range_and_retry_once() {
    use surgeist_dom::DomResource;
    let mut dom = owner();
    let document = document(&mut dom);
    let range = associated_range(&mut dom, document);
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "before");
    batch.set_range_start(range, &text, 1);
    batch.set_range_end(range, &text, 5);
    let text = apply(&mut dom, batch).created(&text).unwrap().unwrap();
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
            DomSourceEdit::set_design_mode(document, "on"),
            policy,
            &cancel,
        ) {
            Err(error) => error,
            Ok(mut lease) => lease.advance(&cancel).unwrap_err(),
        };
        assert_eq!(error, expected);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(!enabled(&dom.snapshot(), document));
        assert_eq!(bounds(&dom.snapshot(), range), ((text, 1), (text, 5)));
        assert_eq!(active(&dom.snapshot(), document), Some(range));
        assert!(dom.pending_source_edit().is_none());
    }
    let done = source_set(&mut dom, document, "on");
    assert_eq!(
        done.design_mode_range_disposition(),
        Some(DomDesignModeRangeDisposition::RangeReset(range))
    );
    assert_eq!(done.commit().changes().changes().len(), 2);
    assert!(enabled(&dom.snapshot(), document));
    assert_eq!(
        bounds(&dom.snapshot(), range),
        ((document.node(), 0), (document.node(), 0))
    );
    assert_eq!(bounds(&before, range), ((text, 1), (text, 5)));
    assert_eq!(dom.usage().ranges(), usage.ranges());
}
