//! Actual successful reader calls, independent of admitted descriptor totals.
//! No pool or consumer backing-array copies are inferred from these counters.
use super::*;

const RAW: &[u16] = &[65, 0xd800, 0, 0xdc00];

fn report(case: &str, executed: Executed, admitted: &Budget) {
    eprintln!(
        "html_pool_reader case={case} executed={executed:?} admitted_inspected_bytes={} admitted_record_visits={} admitted_upgrade_actions={} metadata_scope=successful_Value_read_words consumer_backing_copies_not_observed=true",
        admitted.inspected, admitted.visits, admitted.upgrades
    );
}

#[test]
fn executed_string_hash_and_collision_comparison_count_raw_units_and_headers_separately() {
    for (literal, hit) in [
        (RAW, true),
        (&[66, 0xd800, 0, 0xdc00][..], false),
        (&[65, 0xd800, 0, 1][..], false),
    ] {
        let original = DocumentString::from_utf16(RAW.to_vec());
        let input = DocumentString::from_utf16(literal.to_vec());
        assert!(!Arc::ptr_eq(
            original.shared_code_units(),
            input.shared_code_units()
        ));
        let mut pool = HtmlStringPool::new();
        pool.inner.collision = true;
        seed_string(&mut pool, &original);
        let observed = ObserveExecution::start();
        let mut admitted = Budget::default();
        let outcome = strings(&mut pool, &input, false, &mut admitted).unwrap();
        match outcome {
            Outcome::Hit(output) => {
                assert!(hit);
                assert_eq!(output.code_units(), RAW);
                assert!(Arc::ptr_eq(
                    output.shared_code_units(),
                    original.shared_code_units()
                ));
            }
            Outcome::Miss => assert!(!hit),
            other => panic!("literal collision result: {other:?}"),
        }
        assert_eq!(input.code_units(), literal);
        // Each input has four raw units. Both first- and last-unit mismatch
        // still consume the entire prepaid four-pair run, including its suffix.
        assert_eq!(
            observed.read(),
            Executed {
                hash_utf16_units: 4,
                hash_metadata_reads: 1,
                compare_input_utf16_units: 4,
                compare_candidate_utf16_units: 4,
                compare_metadata_reads: 2,
                compare_read_pairs: 5,
                compare_utf16_pairs: 4,
                compare_metadata_pairs: 1,
                eligibility_checks: 1,
                ..Executed::default()
            }
        );
        // This remains the public ADMITTED inspected-byte quantity; it is not
        // used to manufacture the executed counts above.
        assert_eq!(admitted.inspected, 24);
        report("string_collision", observed.read(), &admitted);
    }
}

fn qualified(value: &[u16]) -> OrderedAttributes {
    bank(vec![name(
        Namespace::from_uri("urn:x"),
        Some(DocumentString::from("p")),
        &[97],
        value,
    )])
}
#[test]
fn executed_attribute_components_and_collision_pairs_preserve_order_and_raw_values() {
    for (literal, hit) in [
        (RAW, true),
        (&[66, 0xd800, 0, 0xdc00][..], false),
        (&[65, 0xd800, 0, 1][..], false),
    ] {
        let original = qualified(RAW);
        let input = qualified(literal);
        assert!(!Arc::ptr_eq(
            original.shared_attributes(),
            input.shared_attributes()
        ));
        let mut pool = HtmlAttributePool::new();
        pool.inner.collision = true;
        seed_attributes(&mut pool, &original);
        let observed = ObserveExecution::start();
        let mut admitted = Budget::default();
        let result = attributes(&mut pool, &input, false, &mut admitted).unwrap();
        match result {
            Outcome::Hit(output) => {
                assert!(hit);
                assert!(Arc::ptr_eq(
                    output.shared_attributes(),
                    original.shared_attributes()
                ));
                assert_eq!(output.as_slice()[0].value().code_units(), RAW);
            }
            Outcome::Miss => assert!(!hit),
            other => panic!("literal ordered-bank collision: {other:?}"),
        }
        assert_eq!(input.len(), 1);
        let attribute = &input.as_slice()[0];
        assert_eq!(
            attribute.name().namespace().uri().unwrap().code_units(),
            [117, 114, 110, 58, 120]
        );
        assert_eq!(attribute.name().prefix().unwrap().code_units(), [112]);
        assert_eq!(attribute.name().local_name().code_units(), [97]);
        assert_eq!(attribute.value().code_units(), literal);
        // One bank header plus four component headers; the actual components
        // contain 5 URI + 1 prefix + 1 local-name + 4 value units.
        assert_eq!(
            observed.read(),
            Executed {
                hash_utf16_units: 11,
                hash_metadata_reads: 5,
                compare_input_utf16_units: 11,
                compare_candidate_utf16_units: 11,
                compare_metadata_reads: 10,
                compare_read_pairs: 16,
                compare_utf16_pairs: 11,
                compare_metadata_pairs: 5,
                eligibility_checks: 1,
                ..Executed::default()
            }
        );
        assert_eq!(admitted.inspected, 66);
        report("attribute_collision", observed.read(), &admitted);
    }
}

#[test]
fn empty_reserved_and_ineligible_inputs_distinguish_headers_from_unread_capacity() {
    for capacity in [0, 7] {
        let original = DocumentString::from_utf16(Vec::with_capacity(capacity));
        let input = DocumentString::from_utf16(Vec::new());
        let mut pool = HtmlStringPool::new();
        seed_string(&mut pool, &original);
        let observed = ObserveExecution::start();
        let mut admitted = Budget::default();
        let Outcome::Hit(output) = strings(&mut pool, &input, false, &mut admitted).unwrap() else {
            panic!("exact empty value");
        };
        assert!(output.is_empty());
        assert_eq!(
            output.shared_code_units().capacity(),
            original.shared_code_units().capacity()
        );
        assert!(Arc::ptr_eq(
            output.shared_code_units(),
            original.shared_code_units()
        ));
        assert_eq!(
            observed.read(),
            Executed {
                hash_metadata_reads: 1,
                compare_metadata_reads: 2,
                compare_read_pairs: 1,
                compare_metadata_pairs: 1,
                eligibility_checks: 1,
                ..Executed::default()
            }
        );
        assert_eq!(admitted.inspected, 0);
        report("empty_string", observed.read(), &admitted);
    }
    for input in [
        DocumentString::from_utf16(vec![0xd800; MAX_UNITS + 1]),
        DocumentString::from_utf16(Vec::with_capacity(MAX_UNITS + 1)),
    ] {
        let backing = input.shared_code_units().clone();
        let mut pool = HtmlStringPool::new();
        let observed = ObserveExecution::start();
        let mut admitted = Budget::default();
        assert!(matches!(
            strings(&mut pool, &input, false, &mut admitted).unwrap(),
            Outcome::Bypassed
        ));
        assert_eq!(
            observed.read(),
            Executed {
                eligibility_checks: 1,
                eligibility_bypasses: 1,
                ..Executed::default()
            }
        );
        assert_eq!(admitted.inspected, 0);
        assert!(Arc::ptr_eq(input.shared_code_units(), &backing));
        assert_eq!(pool.heap_usage().bytes(), 0);
        report("ineligible_string", observed.read(), &admitted);
    }
    let input = bank(Vec::with_capacity(MAX_ATTRIBUTES + 1));
    let mut pool = HtmlAttributePool::new();
    let observed = ObserveExecution::start();
    assert!(matches!(
        attributes(&mut pool, &input, false, &mut Budget::default()).unwrap(),
        Outcome::Bypassed
    ));
    assert_eq!(
        observed.read(),
        Executed {
            eligibility_checks: 1,
            eligibility_bypasses: 1,
            ..Executed::default()
        }
    );
    assert!(input.is_empty());
}

#[test]
fn attribute_payload_bypass_records_only_the_completed_small_component_prefix() {
    let value = DocumentString::from_utf16(vec![0xd800; MAX_UNITS + 1]);
    let input = bank(vec![Attribute::new(
        QualifiedName::new(Namespace::none(), None, "a"),
        value,
    )]);
    let original = input.shared_attributes().clone();
    let mut pool = HtmlAttributePool::new();
    let observed = ObserveExecution::start();
    let mut admitted = Budget::default();
    assert!(matches!(
        attributes(&mut pool, &input, false, &mut admitted).unwrap(),
        Outcome::Bypassed
    ));
    assert_eq!(input.as_slice()[0].value().len(), MAX_UNITS + 1);
    assert!(
        input.as_slice()[0]
            .value()
            .code_units()
            .iter()
            .all(|unit| *unit == 0xd800)
    );
    assert!(Arc::ptr_eq(input.shared_attributes(), &original));
    // Bank, absent URI, absent prefix, local-name header, value header. Only
    // the one-unit local name executes before payload eligibility stops input.
    assert_eq!(
        observed.read(),
        Executed {
            hash_utf16_units: 1,
            hash_metadata_reads: 5,
            eligibility_checks: 1,
            payload_bypasses: 1,
            ..Executed::default()
        }
    );
    assert_eq!(admitted.inspected, 2);
    assert_eq!(pool.heap_usage().bytes(), 0);
    report("attribute_payload_bypass", observed.read(), &admitted);
}

#[test]
fn refused_or_cancelled_reads_do_not_execute_and_same_action_retry_keeps_completed_prefix() {
    for cancelled in [false, true] {
        let original = DocumentString::from_utf16(RAW.to_vec());
        let input = DocumentString::from_utf16(RAW.to_vec());
        let mut pool = HtmlStringPool::new();
        seed_string(&mut pool, &original);
        let observed = ObserveExecution::start();
        let mut operation = Some(pool.probe(&input));
        let refused = loop {
            let action = pool
                .next_action(operation.as_mut().unwrap())
                .unwrap()
                .unwrap();
            if action.phase == Phase::Compare && action.layout().work().inspected_bytes() != 0 {
                break action;
            }
            Budget::default().admit(&action).unwrap();
            match action.kind() {
                HtmlValuePoolActionKind::Read => {
                    operation.as_mut().unwrap().read(action).unwrap();
                }
                HtmlValuePoolActionKind::Pool => {
                    pool.perform(operation.as_mut().unwrap(), action).unwrap();
                }
            }
        };
        let prefix = Executed {
            hash_utf16_units: 4,
            hash_metadata_reads: 1,
            compare_metadata_reads: 2,
            compare_read_pairs: 1,
            compare_metadata_pairs: 1,
            eligibility_checks: 1,
            ..Executed::default()
        };
        assert_eq!(observed.read(), prefix);
        let mut stopped = Budget {
            cancel: cancelled,
            work: 0,
            ..Budget::default()
        };
        assert_eq!(
            stopped.admit(&refused),
            Err(if cancelled {
                Stop::Cancelled
            } else {
                Stop::Work
            })
        );
        assert_eq!(observed.read(), prefix);
        assert_eq!(
            HtmlStringPoolOperation::try_finish(&mut operation).unwrap_err(),
            HtmlValuePoolError::NotReady
        );
        assert!(operation.is_some());
        assert_eq!(input.code_units(), RAW);
        Budget::default().admit(&refused).unwrap();
        operation.as_mut().unwrap().read(refused).unwrap();
        while let Some(action) = pool.next_action(operation.as_mut().unwrap()).unwrap() {
            Budget::default().admit(&action).unwrap();
            match action.kind() {
                HtmlValuePoolActionKind::Read => {
                    operation.as_mut().unwrap().read(action).unwrap();
                }
                HtmlValuePoolActionKind::Pool => {
                    pool.perform(operation.as_mut().unwrap(), action).unwrap();
                }
            }
        }
        let HtmlStringPoolResult::Hit(output) =
            HtmlStringPoolOperation::try_finish(&mut operation).unwrap()
        else {
            panic!("same admitted comparison retry");
        };
        assert!(operation.is_none());
        assert!(Arc::ptr_eq(
            output.value().shared_code_units(),
            original.shared_code_units()
        ));
        assert_eq!(
            observed.read(),
            Executed {
                compare_input_utf16_units: 4,
                compare_candidate_utf16_units: 4,
                compare_read_pairs: 5,
                compare_utf16_pairs: 4,
                ..prefix
            }
        );
    }
}

#[test]
fn first_action_cancellation_and_next_run_refusal_leave_exact_hash_prefixes() {
    for cancelled in [false, true] {
        let input = DocumentString::from_utf16(RAW.to_vec());
        let original = input.shared_code_units().clone();
        let mut pool = HtmlStringPool::new();
        let observed = ObserveExecution::start();
        let mut admitted = Budget {
            cancel: cancelled,
            work: 1,
            ..Budget::default()
        };
        assert_eq!(
            strings(&mut pool, &input, false, &mut admitted).unwrap_err(),
            if cancelled {
                Stop::Cancelled
            } else {
                Stop::Work
            }
        );
        assert_eq!(
            observed.read(),
            if cancelled {
                Executed::default()
            } else {
                Executed {
                    hash_metadata_reads: 1,
                    eligibility_checks: 1,
                    ..Executed::default()
                }
            }
        );
        assert_eq!(admitted.inspected, 0);
        assert_eq!(admitted.visits, usize::from(!cancelled));
        assert_eq!(input.code_units(), RAW);
        assert!(Arc::ptr_eq(input.shared_code_units(), &original));
        assert_eq!(pool.heap_usage().bytes(), 0);
    }
}
