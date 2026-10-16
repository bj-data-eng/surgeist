use std::sync::atomic::AtomicBool;
use surgeist_html::*;
use surgeist_html_semantics::*;

#[test]
fn ordinary_sessions_preserve_send_and_sync() {
    fn assert_traits<T: Send + Sync>() {}
    assert_traits::<HtmlSemanticSession<'static>>();
}

fn envelope() -> HtmlOptionValueEnvelope {
    HtmlOptionValueEnvelope::new(16_000_000, 1_000_000, 10_000_000)
}

fn limits() -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(64, 256, 64, 100_000, 64, 0, 0)
}

fn semantic_limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(1_000_000, 16, 128, 1024, 1_000_000, 100_000)
}

#[test]
fn present_value_preserves_raw_units_and_spare_backing() {
    // HTML Option.value returns the present value unchanged, including ASCII
    // whitespace. A one-unit logical input can retain a much larger backing.
    let cancel = AtomicBool::new(false);
    let mut input = Vec::with_capacity(4096);
    input.extend("\t x \n".encode_utf16());
    let raw = DocumentString::from_utf16(input);
    let attrs = OrderedAttributes::new([Attribute::new(
        QualifiedName::new(Namespace::none(), None, "value"),
        raw.clone(),
    )])
    .unwrap();
    let mut owner = HtmlControlledOptionValueCapture::new(
        limits(),
        HtmlHostFactsRevision::fresh(),
        envelope(),
        &cancel,
    )
    .unwrap();
    let doc = owner
        .add_document(HtmlDocumentMode::NoQuirks, envelope(), &cancel)
        .unwrap();
    let option = owner
        .add_element(
            &doc,
            &QualifiedName::new(Namespace::html(), None, "option"),
            &attrs,
            None,
            envelope(),
            &cancel,
        )
        .unwrap();
    owner
        .establish_children(&option, &[], envelope(), &cancel)
        .unwrap();
    owner.finish(envelope(), &cancel).unwrap();
    let captured_units = owner.storage_usage().utf16_extent();
    let report = owner
        .query_option_value(
            &option,
            HtmlValueInterpreter::new(100_000),
            semantic_limits(),
            envelope(),
            &cancel,
        )
        .unwrap();
    let HtmlSemanticFact::OptionValue(HtmlSemanticValue::Known(value)) = report.results()[0].fact()
    else {
        panic!("known Option.value")
    };
    assert_eq!(value.value().code_units(), [9, 32, 120, 32, 10]);
    assert_eq!(value.origin(), HtmlOptionReadbackOrigin::ValueAttribute);
    assert!(owner.storage_usage().utf16_extent() >= 8192);
    assert_eq!(
        owner.storage_usage().utf16_extent() - captured_units,
        4096,
        "report adds one real retained value occurrence; identities do not retain another capture"
    );
}

#[test]
fn descendant_value_uses_original_ascii_whitespace_and_script_exclusion_rules() {
    let cancel = AtomicBool::new(false);
    let mut owner = HtmlControlledOptionValueCapture::new(
        limits(),
        HtmlHostFactsRevision::fresh(),
        envelope(),
        &cancel,
    )
    .unwrap();
    let doc = owner
        .add_document(HtmlDocumentMode::NoQuirks, envelope(), &cancel)
        .unwrap();
    let attrs = OrderedAttributes::new([]).unwrap();
    let option = owner
        .add_element(
            &doc,
            &QualifiedName::new(Namespace::html(), None, "option"),
            &attrs,
            None,
            envelope(),
            &cancel,
        )
        .unwrap();
    let first = owner
        .add_text(&doc, &" \ta\n".into(), envelope(), &cancel)
        .unwrap();
    let script = owner
        .add_element(
            &doc,
            &QualifiedName::new(Namespace::html(), None, "script"),
            &attrs,
            None,
            envelope(),
            &cancel,
        )
        .unwrap();
    let hidden = owner
        .add_text(&doc, &"excluded".into(), envelope(), &cancel)
        .unwrap();
    let last = owner
        .add_text(&doc, &"\rb\u{a0}c \n".into(), envelope(), &cancel)
        .unwrap();
    owner
        .establish_children(&script, &[hidden], envelope(), &cancel)
        .unwrap();
    owner
        .establish_children(&option, &[first, script, last], envelope(), &cancel)
        .unwrap();
    owner.finish(envelope(), &cancel).unwrap();
    let report = owner
        .query_option_value(
            &option,
            HtmlValueInterpreter::new(100_000),
            semantic_limits(),
            envelope(),
            &cancel,
        )
        .unwrap();
    let HtmlSemanticFact::OptionValue(HtmlSemanticValue::Known(value)) = report.results()[0].fact()
    else {
        panic!("known Option.value")
    };
    assert_eq!(value.value().code_units(), [97, 32, 98, 160, 99]);
    assert_eq!(value.origin(), HtmlOptionReadbackOrigin::DescendantText);
}

#[test]
fn borrowed_spare_text_is_admitted_before_retention_and_allowance_is_total_live() {
    let cancel = AtomicBool::new(false);
    let mut owner = HtmlControlledOptionValueCapture::new(
        limits(),
        HtmlHostFactsRevision::fresh(),
        envelope(),
        &cancel,
    )
    .unwrap();
    let doc = owner
        .add_document(HtmlDocumentMode::NoQuirks, envelope(), &cancel)
        .unwrap();
    let mut source = Vec::with_capacity(8192);
    source.push(120);
    let source = DocumentString::from_utf16(source);
    let before = owner.usage();
    let failure = owner
        .add_text(
            &doc,
            &source,
            HtmlOptionValueEnvelope::new(16_000_000, 8191, 10_000),
            &cancel,
        )
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::LimitExceeded {
            resource: HtmlOptionValueResource::LiveUtf16,
            required: 8192,
            ..
        })
    ));
    assert_eq!(owner.storage_usage(), before.storage());
    assert_eq!(owner.capture_usage().subjects(), 1);
    owner
        .add_text(
            &doc,
            &source,
            HtmlOptionValueEnvelope::new(16_000_000, 8192, 10_000),
            &cancel,
        )
        .unwrap();
    assert_eq!(owner.capture_usage().utf16_units(), 1);
    assert_eq!(owner.storage_usage().utf16_extent(), 8192);
    let retained = owner.storage_usage();
    let failure = owner
        .add_comment(
            &doc,
            &DocumentString::default(),
            HtmlOptionValueEnvelope::new(retained.retained_bytes() - 1, 8192, 10_000),
            &cancel,
        )
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::LimitExceeded {
            resource: HtmlOptionValueResource::LiveBytes,
            ..
        })
    ));
    assert_eq!(owner.storage_usage(), retained);
}

#[test]
fn child_scratch_failure_preserves_original_usage_after_frame_cleanup() {
    let cancel = AtomicBool::new(false);
    let mut owner = HtmlControlledOptionValueCapture::new(
        limits(),
        HtmlHostFactsRevision::fresh(),
        envelope(),
        &cancel,
    )
    .unwrap();
    let doc = owner
        .add_document(HtmlDocumentMode::NoQuirks, envelope(), &cancel)
        .unwrap();
    let empty = OrderedAttributes::new([]).unwrap();
    let option = owner
        .add_element(
            &doc,
            &QualifiedName::new(Namespace::html(), None, "option"),
            &empty,
            None,
            envelope(),
            &cancel,
        )
        .unwrap();
    let mut parent = option.clone();
    for _ in 0..4 {
        let child = owner
            .add_element(
                &doc,
                &QualifiedName::new(Namespace::html(), None, "span"),
                &empty,
                None,
                envelope(),
                &cancel,
            )
            .unwrap();
        owner
            .establish_children(&parent, std::slice::from_ref(&child), envelope(), &cancel)
            .unwrap();
        parent = child;
    }
    owner
        .establish_children(&parent, &[], envelope(), &cancel)
        .unwrap();
    owner.finish(envelope(), &cancel).unwrap();
    let failure = owner
        .query_option_value(
            &option,
            HtmlValueInterpreter::new(100_000),
            HtmlSemanticLimits::new(1_000_000, 1, 3, 128, 1_000_000, 100_000),
            envelope(),
            &cancel,
        )
        .unwrap_err();
    assert!(
        matches!(failure.kind(), HtmlControlledOptionValueError::Semantic(error)
        if matches!(error.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: HtmlSemanticResource::ScratchRecords, required: 4, .. }))
    );
    assert_eq!(owner.usage().semantic().scratch_records(), 0);
    assert_eq!(owner.usage().semantic().peak_scratch_records(), 3);
    assert!(owner.usage().semantic().work() > 0);
    assert!(
        owner.usage().semantic().allocated_bytes()
            >= 3 * std::mem::size_of::<HtmlCapturedChildIds<'_>>()
    );
}

#[test]
fn controlled_children_preserve_foreign_duplicate_conflict_and_depth_failures() {
    let cancel = AtomicBool::new(false);
    let make = || {
        HtmlControlledOptionValueCapture::new(
            limits(),
            HtmlHostFactsRevision::fresh(),
            envelope(),
            &cancel,
        )
        .unwrap()
    };
    let mut owner = make();
    let doc = owner
        .add_document(HtmlDocumentMode::NoQuirks, envelope(), &cancel)
        .unwrap();
    let empty = OrderedAttributes::new([]).unwrap();
    let name = QualifiedName::new(Namespace::html(), None, "div");
    let a = owner
        .add_element(&doc, &name, &empty, None, envelope(), &cancel)
        .unwrap();
    let b = owner
        .add_element(&doc, &name, &empty, None, envelope(), &cancel)
        .unwrap();
    let child = owner
        .add_text(&doc, &"x".into(), envelope(), &cancel)
        .unwrap();
    let mut other = make();
    let foreign = other
        .add_document(HtmlDocumentMode::NoQuirks, envelope(), &cancel)
        .unwrap();
    assert!(matches!(
        owner
            .establish_children(&a, &[foreign.clone(), foreign], envelope(), &cancel)
            .unwrap_err()
            .kind(),
        HtmlControlledOptionValueError::Capture(HtmlCaptureError::ForeignBuilder)
    ));
    assert!(matches!(
        owner
            .establish_children(&a, &[child.clone(), child.clone()], envelope(), &cancel)
            .unwrap_err()
            .kind(),
        HtmlControlledOptionValueError::Capture(HtmlCaptureError::DuplicateChild)
    ));
    owner
        .establish_children(&a, std::slice::from_ref(&child), envelope(), &cancel)
        .unwrap();
    assert!(matches!(
        owner
            .establish_children(&b, &[child.clone(), child], envelope(), &cancel)
            .unwrap_err()
            .kind(),
        HtmlControlledOptionValueError::Capture(HtmlCaptureError::ParentChildConflict)
    ));

    let mut shallow = HtmlControlledOptionValueCapture::new(
        HtmlCaptureLimits::new(4, 16, 0, 100, 0, 0, 0),
        HtmlHostFactsRevision::fresh(),
        envelope(),
        &cancel,
    )
    .unwrap();
    let doc = shallow
        .add_document(HtmlDocumentMode::NoQuirks, envelope(), &cancel)
        .unwrap();
    let parent = shallow
        .add_element(&doc, &name, &empty, None, envelope(), &cancel)
        .unwrap();
    let child = shallow
        .add_text(&doc, &"x".into(), envelope(), &cancel)
        .unwrap();
    assert!(matches!(
        shallow
            .establish_children(&parent, &[child], envelope(), &cancel)
            .unwrap_err()
            .kind(),
        HtmlControlledOptionValueError::Capture(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::Depth,
            required: 1,
            ..
        })
    ));
}

#[test]
fn attribute_result_retention_obeys_parent_capacity_before_the_shared_clone() {
    let cancel = AtomicBool::new(false);
    let mut owner = HtmlControlledOptionValueCapture::new(
        limits(),
        HtmlHostFactsRevision::fresh(),
        envelope(),
        &cancel,
    )
    .unwrap();
    let doc = owner
        .add_document(HtmlDocumentMode::NoQuirks, envelope(), &cancel)
        .unwrap();
    let mut raw = Vec::with_capacity(4096);
    raw.push(120);
    let attrs = OrderedAttributes::new([Attribute::new(
        QualifiedName::new(Namespace::none(), None, "value"),
        DocumentString::from_utf16(raw),
    )])
    .unwrap();
    let option = owner
        .add_element(
            &doc,
            &QualifiedName::new(Namespace::html(), None, "option"),
            &attrs,
            None,
            envelope(),
            &cancel,
        )
        .unwrap();
    owner
        .establish_children(&option, &[], envelope(), &cancel)
        .unwrap();
    owner.finish(envelope(), &cancel).unwrap();
    let retained = owner.storage_usage();
    let failure = owner
        .query_option_value(
            &option,
            HtmlValueInterpreter::new(1),
            semantic_limits(),
            HtmlOptionValueEnvelope::new(16_000_000, retained.utf16_extent() + 4095, 10_000_000),
            &cancel,
        )
        .unwrap_err();
    assert!(
        matches!(failure.kind(), HtmlControlledOptionValueError::Semantic(error)
        if matches!(error.kind(), HtmlSemanticFailureKind::ControlledOptionValue(HtmlOptionValueControlError::LimitExceeded {
            resource: HtmlOptionValueResource::LiveUtf16, required, .. }) if *required == u64::try_from(retained.utf16_extent() + 4096).unwrap()))
    );
    assert_eq!(
        owner.storage_usage().utf16_extent(),
        retained.utf16_extent()
    );
    assert!(owner.usage().semantic().work() > 0);
}

fn query_entry_stop_is_terminal(cancel_first: bool) {
    let cancel = AtomicBool::new(false);
    let mut owner = HtmlControlledOptionValueCapture::new(
        limits(),
        HtmlHostFactsRevision::fresh(),
        envelope(),
        &cancel,
    )
    .unwrap();
    let doc = owner
        .add_document(HtmlDocumentMode::NoQuirks, envelope(), &cancel)
        .unwrap();
    let option = owner
        .add_element(
            &doc,
            &QualifiedName::new(Namespace::html(), None, "option"),
            &OrderedAttributes::new([]).unwrap(),
            None,
            envelope(),
            &cancel,
        )
        .unwrap();
    owner
        .establish_children(&option, &[], envelope(), &cancel)
        .unwrap();
    owner.finish(envelope(), &cancel).unwrap();
    let before = owner.usage();
    let initial = if cancel_first {
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        envelope()
    } else {
        HtmlOptionValueEnvelope::new(before.storage().retained_bytes() - 1, 1_000_000, 10_000_000)
    };
    let first = owner
        .query_option_value(
            &option,
            HtmlValueInterpreter::new(100_000),
            semantic_limits(),
            initial,
            &cancel,
        )
        .unwrap_err();
    if cancel_first {
        assert_eq!(
            first.kind(),
            &HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::Cancelled)
        );
    } else {
        assert!(
            matches!(first.kind(), HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::LimitExceeded {
            resource: HtmlOptionValueResource::LiveBytes, required, ..
        }) if *required == before.storage().retained_bytes() as u64)
        );
    }
    assert_eq!(first.usage().semantic(), HtmlSemanticUsage::default());
    assert_eq!(first.usage().work(), before.work());
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    let retry = owner.query_option_value(
        &option,
        HtmlValueInterpreter::new(100_000),
        semantic_limits(),
        envelope(),
        &cancel,
    );
    assert!(
        retry.is_err(),
        "the first valid Finished-owner query attempt is terminal even before session setup"
    );
    let retry = retry.unwrap_err();
    assert_eq!(retry.kind(), first.kind());
    assert_eq!(
        retry.usage(),
        first.usage(),
        "entry refusal cannot reset or restart child/parent execution"
    );
}

#[test]
fn query_entry_stop_live_limit_is_terminal_for_a_finished_owner() {
    query_entry_stop_is_terminal(false);
}

#[test]
fn query_entry_stop_cancellation_is_terminal_for_a_finished_owner() {
    query_entry_stop_is_terminal(true);
}

fn report_lender_fixture() -> (HtmlControlledOptionValueCapture, HtmlCaptureBuilderSubject) {
    let cancel = AtomicBool::new(false);
    let mut owner = HtmlControlledOptionValueCapture::new(
        limits(),
        HtmlHostFactsRevision::fresh(),
        envelope(),
        &cancel,
    )
    .unwrap();
    let doc = owner
        .add_document(HtmlDocumentMode::NoQuirks, envelope(), &cancel)
        .unwrap();
    let option = owner
        .add_element(
            &doc,
            &QualifiedName::new(Namespace::html(), None, "option"),
            &OrderedAttributes::new([]).unwrap(),
            None,
            envelope(),
            &cancel,
        )
        .unwrap();
    owner
        .establish_children(&option, &[], envelope(), &cancel)
        .unwrap();
    (owner, option)
}

#[test]
fn report_lender_is_absent_until_success_and_after_terminal_entry_refusal() {
    let cancel = AtomicBool::new(false);
    let (mut owner, option) = report_lender_fixture();
    let building = owner.usage();
    assert!(owner.report().is_none());
    assert_eq!(owner.usage(), building);
    owner.finish(envelope(), &cancel).unwrap();
    let finished = owner.usage();
    assert!(owner.report().is_none());
    assert_eq!(owner.usage(), finished);
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(matches!(
        owner
            .query_option_value(
                &option,
                HtmlValueInterpreter::new(100_000),
                semantic_limits(),
                envelope(),
                &cancel,
            )
            .unwrap_err()
            .kind(),
        HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::Cancelled)
    ));
    let failed = owner.usage();
    for _ in 0..3 {
        assert!(owner.report().is_none());
        assert_eq!(owner.usage(), failed);
    }
}

#[test]
fn report_lender_borrows_the_same_retained_report_after_usage_join() {
    let cancel = AtomicBool::new(false);
    let (mut owner, option) = report_lender_fixture();
    owner.finish(envelope(), &cancel).unwrap();
    let original = std::ptr::from_ref(
        owner
            .query_option_value(
                &option,
                HtmlValueInterpreter::new(100_000),
                semantic_limits(),
                envelope(),
                &cancel,
            )
            .unwrap(),
    );
    // The mutable query loan ends here. A consumer can now join actual current
    // storage/work before borrowing this same completed output for retention.
    let usage = owner.usage();
    let storage = owner.storage_usage();
    for _ in 0..3 {
        let report = owner.report().expect("actual successful retained report");
        assert_eq!(std::ptr::from_ref(report), original);
        let HtmlSemanticFact::OptionValue(HtmlSemanticValue::Known(readback)) =
            report.results()[0].fact()
        else {
            panic!("the original empty Option.value report")
        };
        assert!(readback.value().is_empty());
        assert_eq!(readback.origin(), HtmlOptionReadbackOrigin::DescendantText);
        assert_eq!(owner.usage(), usage);
        assert_eq!(owner.storage_usage(), storage);
    }
}
