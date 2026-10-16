use super::*;

fn envelope() -> HtmlOptionValueEnvelope {
    HtmlOptionValueEnvelope::new(16_000_000, 1_000_000, 10_000_000)
}
fn build() -> (
    HtmlControlledOptionValueCapture,
    HtmlCaptureBuilderSubject,
    HtmlCaptureBuilderSubject,
) {
    let cancel = AtomicBool::new(false);
    let mut owner = HtmlControlledOptionValueCapture::new(
        HtmlCaptureLimits::new(64, 256, 64, 100_000, 64, 0, 0),
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
    (owner, doc, option)
}

fn semantic() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(1_000_000, 1, 128, 128, 1_000_000, 100_000)
}

#[test]
fn setup_is_admitted_before_owner_formation_and_failure_keeps_spent_work() {
    let cancel = AtomicBool::new(false);
    let result = HtmlControlledOptionValueCapture::new(
        HtmlCaptureLimits::new(1, 0, 0, 0, 0, 0, 0),
        HtmlHostFactsRevision::fresh(),
        HtmlOptionValueEnvelope::new(size_of::<HtmlControlledOptionValueCapture>() - 1, 0, 10),
        &cancel,
    );
    let Err(failure) = result else {
        panic!("fixed owner header exceeds allowance")
    };
    assert_eq!(failure.usage().work(), 1);
    assert_eq!(failure.usage().storage().retained_bytes(), 0);
    assert!(matches!(
        failure.kind(),
        HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::LimitExceeded {
            resource: HtmlOptionValueResource::LiveBytes,
            ..
        })
    ));
}

#[test]
fn controlled_allocation_attempt_failure_keeps_its_distinct_cause() {
    let (owner, _, _) = build();
    let error = owner.capture_failure(crate::capture::preflight::Failure::AllocationFailed);
    assert_eq!(
        error.kind(),
        &HtmlControlledOptionValueError::AllocationFailed
    );
    let usage = error.usage();
    assert_eq!(
        error.into_error(),
        HtmlControlledOptionValueError::AllocationFailed
    );
    assert_eq!(usage, owner.usage());
}

#[test]
fn final_finish_step_is_not_refunded_and_the_original_builder_can_retry() {
    let cancel = AtomicBool::new(false);
    let (mut owner, _, _) = build();
    let before = owner.usage();
    // Two root records: reference pass 2*4; depth pass fixed5 + incoming
    // 2*4 + state2 + depths2 + two root walks (2+5+2+2); publication step1.
    let finish_work = 2 * 4 + 5 + 2 * 4 + 2 + 2 + 2 * (2 + 5 + 2 + 2) + 1;
    assert_eq!(finish_work, 48);
    let error = owner
        .finish(
            HtmlOptionValueEnvelope::new(16_000_000, 1_000_000, finish_work - 1),
            &cancel,
        )
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::LimitExceeded {
            resource: HtmlOptionValueResource::Work,
            required: 48,
            ..
        })
    ));
    assert_eq!(owner.usage().work() - before.work(), 47);
    assert_eq!(owner.storage_usage(), before.storage());
    owner
        .finish(
            HtmlOptionValueEnvelope::new(16_000_000, 1_000_000, finish_work),
            &cancel,
        )
        .unwrap();
    assert_eq!(owner.usage().work() - before.work(), 47 + 48);
}

#[test]
fn finish_keeps_old_graph_and_admits_the_new_capture_header_before_publication() {
    let cancel = AtomicBool::new(false);
    let (mut owner, _, _) = build();
    let before = owner.storage_usage();
    let header = HtmlCaptureBuilder::controlled_data_header_bytes();
    let scratch = 2 * (size_of::<Option<usize>>() + size_of::<u8>() + 2 * size_of::<usize>());
    assert!(
        header > scratch,
        "this fixture's late allocation is the binding extent"
    );
    let error = owner
        .finish(
            HtmlOptionValueEnvelope::new(before.bytes + header - 1, before.units, 10_000),
            &cancel,
        )
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::LimitExceeded {
            resource: HtmlOptionValueResource::LiveBytes,
            ..
        })
    ));
    assert_eq!(owner.storage_usage(), before);
    owner
        .finish(
            HtmlOptionValueEnvelope::new(before.bytes + header, before.units, 10_000),
            &cancel,
        )
        .unwrap();
    assert_eq!(owner.storage_usage().bytes, before.bytes + header);
}

#[test]
fn child_stop_keeps_qualification_and_usage_and_is_terminal_for_this_query_owner() {
    let cancel = AtomicBool::new(false);
    let (mut owner, _, option) = build();
    owner.finish(envelope(), &cancel).unwrap();
    let before = owner.usage();
    let limits = HtmlSemanticLimits::new(0, 1, 128, 128, 1_000_000, 100_000);
    let failure = owner
        .query_option_value(
            &option,
            HtmlValueInterpreter::new(100_000),
            limits,
            envelope(),
            &cancel,
        )
        .unwrap_err();
    assert!(
        matches!(failure.kind(), HtmlControlledOptionValueError::Semantic(error)
        if matches!(error.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: HtmlSemanticResource::Work, required: 1, .. })
        && error.qualification().is_some())
    );
    assert_eq!(owner.usage().semantic().work(), 0);
    assert_eq!(owner.usage().semantic().allocated_bytes(), 0);
    assert_eq!(owner.usage().work() - before.work(), 4);
    assert_eq!(
        owner.storage_usage().bytes,
        before.storage().bytes + size_of::<HtmlSemanticQualification>()
    );
    let used = owner.usage();
    let repeated = owner
        .query_option_value(
            &option,
            HtmlValueInterpreter::new(100_000),
            semantic(),
            envelope(),
            &cancel,
        )
        .unwrap_err();
    assert_eq!(repeated.kind(), failure.kind());
    assert_eq!(owner.usage(), used);
}

#[test]
fn cancellation_inside_finish_preserves_work_and_drops_its_scratch_only() {
    let cancel = AtomicBool::new(false);
    let (mut owner, _, _) = build();
    let before = owner.usage();
    owner.control.cancel_after.set(before.work() + 17);
    let error = owner.finish(envelope(), &cancel).unwrap_err();
    assert!(matches!(
        error.kind(),
        HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::Cancelled)
    ));
    assert!(owner.usage().work() >= before.work() + 17);
    assert_eq!(owner.storage_usage(), before.storage());
    assert_eq!(
        owner.control.temporary.get(),
        HtmlOptionValueStorageUsage::default()
    );
}

#[test]
fn full_old_and_new_query_backing_is_admitted_before_growth() {
    let control = Control::new(HtmlOptionValueEnvelope::new(29, 20, 10));
    control
        .add(HtmlOptionValueStorageUsage { bytes: 8, units: 4 }, false)
        .unwrap();
    let cancel = AtomicBool::new(false);
    let query = QueryControl {
        control: &control,
        cancel: &cancel,
    };
    // An existing four-u16 bank (8 bytes) plus a new eleven-u16 bank
    // (22 bytes) needs30, although the net final bank needs only22.
    assert!(matches!(
        query.before_growth(4, 11, 2, HtmlSemanticResource::NestedOutputRecords),
        Err(HtmlOptionValueControlError::LimitExceeded {
            resource: HtmlOptionValueResource::LiveBytes,
            required: 30,
            ..
        })
    ));
    assert_eq!(control.temporary.get().bytes, 8);
    control
        .envelope
        .set(HtmlOptionValueEnvelope::new(30, 15, 10));
    query
        .before_growth(4, 11, 2, HtmlSemanticResource::NestedOutputRecords)
        .unwrap();
    query
        .after_growth(4, 11, 2, HtmlSemanticResource::NestedOutputRecords)
        .unwrap();
    assert_eq!(
        control.temporary.get(),
        HtmlOptionValueStorageUsage {
            bytes: 22,
            units: 11
        }
    );
    assert_eq!(control.peak.get().bytes, 30);
    assert_eq!(control.peak.get().units, 15);
}

#[test]
fn empty_query_setup_has_a_binding_fixed_extent_before_child_query_admission() {
    let cancel = AtomicBool::new(false);
    let fixed = size_of::<HtmlSemanticContext>()
        + size_of::<HtmlSemanticContextInput>()
        + size_of::<HtmlSemanticSession<'_>>()
        + size_of::<HtmlSemanticRequest>()
        + size_of::<Vec<u16>>()
        + size_of::<AtomicBool>()
        + 3 * size_of::<HtmlSemanticQualification>();
    for short in [true, false] {
        let (mut owner, _, option) = build();
        owner.finish(envelope(), &cancel).unwrap();
        let before = owner.usage();
        // Original output_records=0 permits session setup, then rejects the
        // one requested result before original work/allocation is consumed.
        let failure = owner
            .query_option_value(
                &option,
                HtmlValueInterpreter::new(100_000),
                HtmlSemanticLimits::new(100_000, 0, 128, 128, 1_000_000, 100_000),
                HtmlOptionValueEnvelope::new(
                    before.storage().bytes + fixed - usize::from(short),
                    1_000_000,
                    100_000,
                ),
                &cancel,
            )
            .unwrap_err();
        if short {
            assert!(matches!(
                failure.kind(),
                HtmlControlledOptionValueError::Control(
                    HtmlOptionValueControlError::LimitExceeded {
                        resource: HtmlOptionValueResource::LiveBytes,
                        ..
                    }
                )
            ));
        } else {
            assert!(
                matches!(failure.kind(), HtmlControlledOptionValueError::Semantic(error)
                if matches!(error.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: HtmlSemanticResource::OutputRecords, required: 1, .. }))
            );
        }
        assert_eq!(owner.usage().work() - before.work(), 4);
        assert_eq!(owner.usage().semantic().work(), 0);
        assert_eq!(
            owner.control.temporary.get(),
            HtmlOptionValueStorageUsage::default()
        );
    }
}

#[test]
fn late_parent_query_stop_keeps_the_completed_child_usage_and_cannot_restart_it() {
    let cancel = AtomicBool::new(false);
    // For the attribute path, parent-only work is fixed setup4 + unchanged
    // string observation1 + report observation1 + retained string observation1.
    // First establish the original child's independently controlled work on the
    // same public capture: the expected late stop is after child completion.
    let mut ordinary = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(4, 16, 2, 1000, 4, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let doc = ordinary
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let attrs = OrderedAttributes::new([Attribute::new(
        QualifiedName::new(Namespace::none(), None, "value"),
        "x",
    )])
    .unwrap();
    let name = QualifiedName::new(Namespace::html(), None, "option");
    let option = ordinary
        .add_element(
            &doc,
            name.clone(),
            attrs.clone(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    ordinary.establish_children(&option, &[]).unwrap();
    let capture = ordinary.finish().unwrap();
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(Vec::new()),
        0,
    )
    .unwrap();
    let mut session = HtmlSemanticSession::new(
        &capture,
        &context,
        HtmlValueInterpreter::new(1000),
        semantic(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    session
        .query(&[HtmlSemanticRequest::new(
            capture.subject_id(&option).unwrap(),
            HtmlSemanticQuery::OptionValue,
        )])
        .unwrap();
    let child = session.usage();
    let mut owner = HtmlControlledOptionValueCapture::new(
        HtmlCaptureLimits::new(4, 16, 2, 1000, 4, 0, 0),
        HtmlHostFactsRevision::fresh(),
        envelope(),
        &cancel,
    )
    .unwrap();
    let doc = owner
        .add_document(HtmlDocumentMode::NoQuirks, envelope(), &cancel)
        .unwrap();
    let option = owner
        .add_element(&doc, &name, &attrs, None, envelope(), &cancel)
        .unwrap();
    owner
        .establish_children(&option, &[], envelope(), &cancel)
        .unwrap();
    owner.finish(envelope(), &cancel).unwrap();
    let before = owner.usage();
    let work = u64::try_from(child.work()).unwrap() + 6;
    let failure = owner
        .query_option_value(
            &option,
            HtmlValueInterpreter::new(1000),
            semantic(),
            HtmlOptionValueEnvelope::new(16_000_000, 1_000_000, work),
            &cancel,
        )
        .unwrap_err();
    assert!(
        matches!(failure.kind(), HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::LimitExceeded {
        resource: HtmlOptionValueResource::Work, required, .. }) if *required == work + 1)
    );
    assert_eq!(owner.usage().semantic(), child);
    assert_eq!(owner.usage().work() - before.work(), work);
    let used = owner.usage();
    assert_eq!(
        owner
            .query_option_value(
                &option,
                HtmlValueInterpreter::new(1000),
                semantic(),
                envelope(),
                &cancel
            )
            .unwrap_err()
            .kind(),
        failure.kind()
    );
    assert_eq!(owner.usage(), used);
}

#[test]
fn parent_cancellation_inside_the_original_query_keeps_its_first_child_usage() {
    let cancel = AtomicBool::new(false);
    let (mut owner, _, option) = build();
    owner.finish(envelope(), &cancel).unwrap();
    let before = owner.usage();
    // Setup4 then the original result-bank charge1. Cancellation is observed
    // before reservation, preserving that original admitted unit but no bank.
    owner.control.cancel_after.set(before.work() + 5);
    let failure = owner
        .query_option_value(
            &option,
            HtmlValueInterpreter::new(1000),
            semantic(),
            envelope(),
            &cancel,
        )
        .unwrap_err();
    assert!(
        matches!(failure.kind(), HtmlControlledOptionValueError::Semantic(error)
        if matches!(error.kind(), HtmlSemanticFailureKind::ControlledOptionValue(HtmlOptionValueControlError::Cancelled)))
    );
    assert_eq!(owner.usage().work() - before.work(), 5);
    assert_eq!(owner.usage().semantic().work(), 1);
    assert_eq!(owner.usage().semantic().allocated_bytes(), 0);
    assert_eq!(
        owner.control.temporary.get(),
        HtmlOptionValueStorageUsage::default()
    );
}

#[test]
fn wrong_phase_query_does_not_consume_the_finished_query_attempt() {
    let cancel = AtomicBool::new(false);
    let (mut owner, _, option) = build();
    let error = owner
        .query_option_value(
            &option,
            HtmlValueInterpreter::new(100_000),
            semantic(),
            envelope(),
            &cancel,
        )
        .unwrap_err();
    assert_eq!(error.kind(), &HtmlControlledOptionValueError::InvalidPhase);
    owner.finish(envelope(), &cancel).unwrap();
    let report = owner
        .query_option_value(
            &option,
            HtmlValueInterpreter::new(100_000),
            semantic(),
            envelope(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        report.results()[0].fact(),
        HtmlSemanticFact::OptionValue(HtmlSemanticValue::Known(_))
    ));
}
