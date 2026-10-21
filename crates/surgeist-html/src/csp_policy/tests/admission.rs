use super::*;

#[test]
fn empty_and_interrupted_request_return_the_original_owned_input() {
    let (mut owner, association) = pair();
    let p = Processor::new();
    let error = owner
        .request_meta_insertion_with(
            &association,
            p.profile(),
            DocumentString::from(""),
            &mut allow,
        )
        .unwrap_err();
    assert!(matches!(
        error.error,
        HtmlCspError::Protocol(HtmlCspProtocolError::EmptyContent)
    ));
    assert!(error.retained.is_empty());
    let input = DocumentString::from_utf16([0xd800, 49]);
    let backing = input.shared_code_units().clone();
    let mut budget = Budget::unlimited();
    budget.cancelled = true;
    let error = owner
        .request_meta_insertion_with(&association, p.profile(), input, &mut |layout| {
            budget.admit(layout)
        })
        .unwrap_err();
    assert!(matches!(
        error.error,
        HtmlCspError::Interrupted(Stop::Cancelled)
    ));
    assert!(Arc::ptr_eq(&backing, error.retained.shared_code_units()));
    assert_eq!(budget.spent, 0);
    let r = owner
        .request_meta_insertion_with(&association, p.profile(), error.retained, &mut allow)
        .unwrap();
    assert_eq!(r.qualification().content().code_units(), [0xd800, 49]);
}

#[test]
fn request_one_short_work_storage_and_atomic_admission_never_change_association() {
    let (mut owner, association) = pair();
    let p = Processor::new();
    let mut baseline = Budget::unlimited();
    let r = owner
        .request_meta_insertion_with(
            &association,
            p.profile(),
            DocumentString::from("1"),
            &mut |layout| baseline.admit(layout),
        )
        .unwrap();
    drop(r);
    let peak = baseline
        .layouts
        .iter()
        .map(|layout| layout.peak().bytes())
        .max()
        .unwrap();
    let atomic = baseline
        .layouts
        .iter()
        .map(|layout| layout.atomic_bytes())
        .max()
        .unwrap();
    for resource in [Stop::Work, Stop::Storage, Stop::Atomic] {
        let mut budget = Budget::unlimited();
        match resource {
            Stop::Work => budget.work = baseline.spent - 1,
            Stop::Storage => budget.storage = peak - 1,
            Stop::Atomic => budget.atomic = atomic - 1,
            Stop::Cancelled => unreachable!(),
        }
        let input = DocumentString::from("1");
        let backing = input.shared_code_units().clone();
        let error = owner
            .request_meta_insertion_with(&association, p.profile(), input, &mut |layout| {
                budget.admit(layout)
            })
            .unwrap_err();
        assert!(matches!(error.error, HtmlCspError::Interrupted(actual) if actual == resource));
        assert!(Arc::ptr_eq(&backing, error.retained.shared_code_units()));
        assert!(association.latest().is_none());
        assert!(association.policy().profile().is_none());
    }
}

#[test]
fn publication_one_short_and_cancel_return_both_original_inputs_without_reprocessing() {
    let (mut owner, mut association) = pair();
    let mut p = Processor::new();
    let mut baseline = Budget::unlimited();
    let r = request(&mut owner, &association, p.profile(), "1");
    let value = prepared(&mut p, &r);
    let guard = owner
        .prepare_publication_with(&mut association, r, value, &mut |layout| {
            baseline.admit(layout)
        })
        .unwrap();
    let (mut r, mut value) = guard.abort();
    let expected_occurrence = r.qualification().occurrence().clone();
    let expected_policy = value.policy().clone();
    let peak = baseline
        .layouts
        .iter()
        .map(|layout| layout.peak().bytes())
        .max()
        .unwrap();
    let atomic = baseline
        .layouts
        .iter()
        .map(|layout| layout.atomic_bytes())
        .max()
        .unwrap();
    for resource in [Stop::Work, Stop::Storage, Stop::Atomic, Stop::Cancelled] {
        let mut budget = Budget::unlimited();
        match resource {
            Stop::Work => budget.work = baseline.spent - 1,
            Stop::Storage => budget.storage = peak - 1,
            Stop::Atomic => budget.atomic = atomic - 1,
            Stop::Cancelled => budget.cancelled = true,
        }
        let rejected = owner
            .prepare_publication_with(&mut association, r, value, &mut |layout| {
                budget.admit(layout)
            })
            .unwrap_err();
        assert!(matches!(rejected.error, HtmlCspError::Interrupted(actual) if actual == resource));
        (r, value) = rejected.retained;
        assert_eq!(r.qualification().occurrence(), &expected_occurrence);
        assert_eq!(value.policy(), &expected_policy);
        assert!(association.latest().is_none());
    }
    let completion = publish(&mut owner, &mut association, r, value);
    assert_eq!(
        completion.receipt().qualification().occurrence(),
        &expected_occurrence
    );
    assert_eq!(p.calls, 1);
    assert_eq!(p.observation.metadata.load(Ordering::Relaxed), 1);
}

#[test]
fn cached_disposal_is_prepaid_for_each_new_occurrence_without_reinvoking_native_metadata() {
    let (mut owner, mut association) = pair();
    let mut p = Processor::new();
    let r = request(&mut owner, &association, p.profile(), "1");
    let value = prepared(&mut p, &r);
    drop(publish(&mut owner, &mut association, r, value));
    let mut budget = Budget::unlimited();
    let moved = owner
        .associate_document_with(
            &association,
            HtmlCspAssociationCause::Inheritance,
            &mut |layout| budget.admit(layout),
        )
        .unwrap();
    let retained = moved.retained_layout().unwrap();
    let acquisition = budget.layouts.last().unwrap();
    assert_eq!(acquisition.closing_work(), retained.disposal());
    assert!(
        acquisition.closing_work().work_credits()
            >= NativePolicy::layout().disposal().work_credits()
    );
    assert_eq!(p.observation.metadata.load(Ordering::Relaxed), 1);
    let expected = budget.spent;
    let mut short = Budget::unlimited();
    short.work = expected - 1;
    assert!(matches!(
        owner.associate_document_with(
            &association,
            HtmlCspAssociationCause::Installation,
            &mut |layout| short.admit(layout)
        ),
        Err(HtmlCspError::Interrupted(Stop::Work))
    ));
    assert_eq!(p.observation.drops.load(Ordering::Relaxed), 0);
    drop(moved);
    assert_eq!(budget.spent, expected, "disposal does not refund paid work");
}

#[test]
fn rejected_attestation_returns_actual_native_custody_and_processor_releases_after_drop() {
    let (mut owner, association) = pair();
    let mut p = Processor::new();
    let r = request(&mut owner, &association, p.profile(), "1");
    let artifact = Arc::new(NativePolicy {
        values: [101, 0, 0, 0],
        len: 1,
        observation: p.observation.clone(),
    });
    let original = Arc::as_ptr(&artifact);
    let mut calls = 0;
    let rejected = p
        .authority
        .attest_changed_with(&r, artifact, &mut |_| {
            calls += 1;
            if calls == 2 {
                Err(Stop::Storage)
            } else {
                Ok(())
            }
        })
        .unwrap_err();
    assert!(matches!(
        rejected.error,
        HtmlCspError::Interrupted(Stop::Storage)
    ));
    assert_eq!(Arc::as_ptr(&rejected.retained), original);
    assert_eq!(p.observation.drops.load(Ordering::Relaxed), 0);
    drop(rejected.retained);
    assert_eq!(p.observation.drops.load(Ordering::Relaxed), 1);

    let mut callbacks = 0;
    let observation = p.observation.clone();
    let mut released = 0;
    let outcome = p.prepare(
        &r,
        &mut |_| {
            callbacks += 1;
            // Selected profile, prior, work, partial construction, attestation read,
            // then attestation retention.
            if callbacks == 6 {
                Err(Stop::Storage)
            } else {
                Ok(())
            }
        },
        &mut |storage| {
            assert_eq!(observation.drops.load(Ordering::Relaxed), 2);
            assert_eq!(storage, NativePolicy::layout().storage());
            released += 1;
        },
    );
    assert!(matches!(
        outcome,
        Err(HtmlCspError::Interrupted(Stop::Storage))
    ));
    assert_eq!(released, 1);
}

#[test]
fn layout_arithmetic_rejects_unrepresentable_storage_and_disposal_before_admission() {
    assert_eq!(
        HtmlCspStorage::try_new(1, 1, 0),
        Err(HtmlCspProtocolError::InvalidStorage)
    );
    assert_eq!(
        HtmlCspStorage::try_new(usize::MAX, usize::MAX, 0),
        Err(HtmlCspProtocolError::InvalidStorage)
    );
    let storage = HtmlCspStorage::try_new(usize::MAX, 0, 0).unwrap();
    let work = HtmlCspDisposalWork::try_new(1, 1).unwrap();
    assert_eq!(
        HtmlCspArtifactLayout::try_new(storage, work),
        Err(HtmlCspProtocolError::RepresentationExhausted)
    );
    let closing = HtmlCspDisposalWork::try_new(1, 0).unwrap();
    assert_eq!(
        HtmlCspLayout::processing(u64::MAX, HtmlCspStorage::default(), closing, 0),
        Err(HtmlCspProtocolError::RepresentationExhausted)
    );
}
