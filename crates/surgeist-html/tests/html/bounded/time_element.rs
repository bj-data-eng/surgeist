//! Public retained time interpretation, using the pinned time-element rules.
#![forbid(unsafe_code)]
use surgeist_html::{
    bounded::{
        numeric::{HtmlIntegerUnitStepError, HtmlNumericMaterializationError},
        temporal::{
            HtmlTimeElementInterpretation as Interpretation,
            HtmlTimeElementInterpretationActionKind as Kind,
        },
    },
    microsyntax::temporal::{HtmlMachineReadableTime, HtmlTimeElementValue},
    values::{DocumentString, HtmlValueAuthorState, HtmlValueInterpreter, HtmlValueRecoveryState},
};

fn drive(owner: &mut Interpretation<'_>) -> (usize, usize) {
    let mut work = 0;
    let mut capacity = 0;
    for _ in 0..100_000 {
        let Some(action) = owner.next_action().unwrap() else {
            return (work, capacity);
        };
        let layout = action.layout();
        work += layout.work();
        let step = action.advance(100_000, usize::MAX);
        assert!(layout.can_allocate() || step.capacity_bytes() == 0);
        capacity += step.capacity_bytes();
        step.outcome().unwrap();
    }
    panic!("bounded fixture failed to reach completion");
}

#[test]
fn early_finish_and_refused_guard_preserve_exact_raw_and_pending_recognition() {
    let raw = DocumentString::from_utf16([49, 0xd800, 0, 50]);
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
    assert!(
        Interpretation::try_finish(&mut owner)
            .unwrap_err()
            .is_not_ready()
    );
    let before = owner
        .as_mut()
        .unwrap()
        .next_action()
        .unwrap()
        .unwrap()
        .layout();
    assert_eq!(before.kind(), Kind::Recognition);
    assert_eq!(
        owner
            .as_mut()
            .unwrap()
            .next_action()
            .unwrap()
            .unwrap()
            .layout(),
        before
    );
    assert!(std::ptr::eq(owner.as_ref().unwrap().raw(), &raw));
    assert!(std::ptr::eq(owner.take().unwrap().abort(), &raw));
    assert!(
        Interpretation::try_finish(&mut owner)
            .unwrap_err()
            .is_consumed()
    );
}

#[test]
fn publication_requires_admitted_report_assembly_after_all_attempts() {
    let raw = DocumentString::from("0001");
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(4)).unwrap());
    let mut work = 0;
    let mut capacity = 0;
    let mut numeric = 0;
    loop {
        let action = owner.as_mut().unwrap().next_action().unwrap().unwrap();
        let layout = action.layout();
        if layout.kind() == Kind::ReportAssembly {
            break;
        }
        numeric += usize::from(layout.can_allocate());
        work += layout.work();
        let step = action.advance(4, usize::MAX);
        capacity += step.capacity_bytes();
        step.outcome().unwrap();
    }
    assert!(
        Interpretation::try_finish(&mut owner)
            .unwrap_err()
            .is_not_ready()
    );
    assert!(std::ptr::eq(owner.as_ref().unwrap().raw(), &raw));
    let action = owner.as_mut().unwrap().next_action().unwrap().unwrap();
    assert_eq!(action.layout().kind(), Kind::ReportAssembly);
    work += action.layout().work();
    let step = action.advance(0, 0);
    assert_eq!(step.capacity_bytes(), 0);
    step.outcome().unwrap();
    // Source-basis characterization: nine alternatives in each phase at
    // 16*4+256 work, two begin/year assemblies, two 5*4+8 digit operations,
    // and one report assembly. No extra grammar pass or fee is introduced.
    assert_eq!(work, 5821);
    assert_eq!(numeric, 2);
    assert_eq!(capacity, 2);
    let report = Interpretation::try_finish(&mut owner).unwrap();
    assert!(owner.is_none());
    assert_eq!(report.raw(), Some(&raw));
    assert!(
        matches!(report.author(), HtmlValueAuthorState::Valid(HtmlTimeElementValue::Year(year)) if year.decimal_digits() == "1")
    );
    assert!(
        matches!(report.recovery(), HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Year(year)) if year.decimal_digits() == "1")
    );
}

#[test]
fn all_ten_authored_alternatives_preserve_their_processing_discriminators() {
    for (text, expected) in [
        ("2011-11", 0),
        ("2011-11-18", 1),
        ("--02-29", 2),
        ("14:54:39.929", 3),
        ("2011-11-18 14:54", 4),
        ("-0800", 5),
        ("2011-11-18T06:54-0800", 6),
        ("2011-W47", 7),
        ("0001", 8),
        ("PT4H18M3S", 9),
    ] {
        let raw = DocumentString::from(text);
        let mut owner =
            Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
        drive(owner.as_mut().unwrap());
        let report = Interpretation::try_finish(&mut owner).unwrap();
        let author = match report.author() {
            HtmlValueAuthorState::Valid(HtmlTimeElementValue::Month(_)) => 0,
            HtmlValueAuthorState::Valid(HtmlTimeElementValue::Date(_)) => 1,
            HtmlValueAuthorState::Valid(HtmlTimeElementValue::YearlessDate(_)) => 2,
            HtmlValueAuthorState::Valid(HtmlTimeElementValue::Time(_)) => 3,
            HtmlValueAuthorState::Valid(HtmlTimeElementValue::LocalDateTime(_)) => 4,
            HtmlValueAuthorState::Valid(HtmlTimeElementValue::TimeZoneOffset(_)) => 5,
            HtmlValueAuthorState::Valid(HtmlTimeElementValue::GlobalDateTime(_)) => 6,
            HtmlValueAuthorState::Valid(HtmlTimeElementValue::Week(_)) => 7,
            HtmlValueAuthorState::Valid(HtmlTimeElementValue::Year(_)) => 8,
            HtmlValueAuthorState::Valid(HtmlTimeElementValue::Duration(_)) => 9,
            other => panic!("{text}: {other:?}"),
        };
        let recovery = match report.recovery() {
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Month(_)) => 0,
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Date(_)) => 1,
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::YearlessDate(_)) => 2,
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Time(_)) => 3,
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::LocalDateTime(_)) => 4,
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::TimeZoneOffset(_)) => 5,
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::GlobalDateTime(_)) => 6,
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Week(_)) => 7,
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Year(_)) => 8,
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Duration(_)) => 9,
            other => panic!("{text}: {other:?}"),
        };
        assert_eq!((author, recovery), (expected, expected), "{text}");
    }
}

#[test]
fn recovery_keeps_exact_fraction_duration_sum_and_offset_day_carry() {
    for text in ["12:34:56.1234", "PT1S1S", "0001-01-01T00:00+00:01"] {
        let raw = DocumentString::from(text);
        let mut owner =
            Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
        let (_, capacity) = drive(owner.as_mut().unwrap());
        assert!(capacity > 0);
        let report = Interpretation::try_finish(&mut owner).unwrap();
        match report.recovery() {
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Time(value)) => {
                assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
                assert_eq!(value.second().fractional_digits(), Some("1234"));
            }
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Duration(value)) => {
                assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
                assert_eq!(value.seconds().integer_part().decimal_digits(), "2");
            }
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::GlobalDateTime(value)) => {
                assert_eq!(value.utc().day().to_i64().unwrap(), -1);
                assert_eq!(
                    (value.utc().time().hour(), value.utc().time().minute()),
                    (23, 59)
                );
            }
            other => panic!("{text}: {other:?}"),
        }
    }
}

#[test]
fn failed_fraction_retains_owner_and_observes_earlier_capacity_once() {
    let raw = DocumentString::from("12:00:01.1234");
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
    let (error, observed) = loop {
        let action = owner.as_mut().unwrap().next_action().unwrap().unwrap();
        let step = action.advance(100, 1);
        if let Err(error) = step.outcome() {
            break (error, step.capacity_bytes());
        }
        assert_eq!(step.capacity_bytes(), 0);
    };
    assert_eq!(observed, 1);
    assert!(matches!(
        error.processing_failure(),
        Some(HtmlNumericMaterializationError::Boundary(
            HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded { required: 5, .. }
        ))
    ));
    assert_eq!(Interpretation::try_finish(&mut owner).unwrap_err(), error);
    assert!(matches!(owner.as_mut().unwrap().next_action(), Err(actual) if actual == error));
    assert!(std::ptr::eq(owner.as_ref().unwrap().raw(), &raw));
    assert!(std::ptr::eq(owner.take().unwrap().abort(), &raw));
}

#[test]
fn caller_cancellation_at_each_action_recovers_raw_without_a_partial_report() {
    let raw = DocumentString::from("2011-11-18T06:54:00.1234-0800");
    for stop_after in 0..100 {
        let mut owner =
            Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
        let mut completed = false;
        for _ in 0..stop_after {
            let Some(action) = owner.as_mut().unwrap().next_action().unwrap() else {
                completed = true;
                break;
            };
            action.advance(100, usize::MAX).outcome().unwrap();
        }
        if completed {
            break;
        }
        // The caller's stop performs no next operation; recover the retained raw.
        assert!(std::ptr::eq(owner.take().unwrap().abort(), &raw));
        assert!(
            Interpretation::try_finish(&mut owner)
                .unwrap_err()
                .is_consumed()
        );
    }
}

#[test]
fn raw_admission_and_processing_digit_limits_preserve_the_original_operand() {
    let raw = DocumentString::from("12345");
    let admission = Interpretation::new(&raw, HtmlValueInterpreter::new(4))
        .err()
        .unwrap();
    assert_eq!((admission.capacity(), admission.required()), (4, 5));
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(5)).unwrap());
    let error = loop {
        let action = owner.as_mut().unwrap().next_action().unwrap().unwrap();
        let step = action.advance(4, usize::MAX);
        assert_eq!(step.capacity_bytes(), 0);
        if let Err(error) = step.outcome() {
            break error;
        }
    };
    assert!(matches!(
        error.processing_failure(),
        Some(HtmlNumericMaterializationError::Boundary(
            HtmlIntegerUnitStepError::DigitLimitExceeded {
                limit: 4,
                required: 5
            }
        ))
    ));
    assert_eq!(Interpretation::try_finish(&mut owner).unwrap_err(), error);
    assert!(std::ptr::eq(owner.take().unwrap().abort(), &raw));
}

#[test]
fn completed_invalid_syntax_is_a_report_without_processing_failure_or_allocation() {
    let raw = DocumentString::from_utf16([49, 0xd800, 0, 50]);
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
    let (_, capacity) = drive(owner.as_mut().unwrap());
    assert_eq!(capacity, 0);
    let report = Interpretation::try_finish(&mut owner).unwrap();
    assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
    assert!(matches!(
        report.recovery(),
        HtmlValueRecoveryState::Failed(_)
    ));
    assert!(std::sync::Arc::ptr_eq(
        report.raw().unwrap().shared_code_units(),
        raw.shared_code_units()
    ));
}

/// Total work charged to interpret `text` through both phases.
fn interpretation_work(text: &str) -> usize {
    let raw = DocumentString::from(text);
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
    let (work, _) = drive(owner.as_mut().unwrap());
    let report = Interpretation::try_finish(&mut owner).unwrap();
    assert!(matches!(
        report.recovery(),
        HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Duration(_))
    ));
    work
}

/// "parse a duration string" (html-source:7160-7340) steps 6-8 repeat per
/// component, and each component's scan consumes only its own units. The
/// charge must therefore grow with the input, not with the input times the
/// number of components.
#[test]
fn recovery_duration_charge_grows_linearly_with_component_count() {
    let charges: Vec<usize> = [256, 512, 1024, 2048]
        .into_iter()
        .map(|components| interpretation_work(&"1s".repeat(components)))
        .collect();
    for pair in charges.windows(2) {
        assert!(
            pair[1] * 10 <= pair[0] * 22,
            "doubling the components grew the charge from {} to {}: {charges:?}",
            pair[0],
            pair[1]
        );
    }
}

/// Total work charged to interpret `text` through both phases, and the recovered seconds.
fn interpretation_work_and_seconds(text: &str) -> (usize, String) {
    let raw = DocumentString::from(text);
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
    let (work, _) = drive(owner.as_mut().unwrap());
    let report = Interpretation::try_finish(&mut owner).unwrap();
    let HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Duration(duration)) =
        report.recovery()
    else {
        panic!(
            "expected a recovered duration for a string of {} units",
            raw.len()
        );
    };
    assert_eq!(duration.seconds().fractional_digits(), None);
    (
        work,
        duration
            .seconds()
            .integer_part()
            .decimal_digits()
            .to_owned(),
    )
}

/// "parse a duration string" (foundations capture lines 3350-3430) steps 8.6-8.13 add every
/// component to the running total, and a unit may repeat (step 8 has no uniqueness rule). The
/// first token is the `digits` nines and the first `1`, 10^(digits+1) - 9, and the other
/// `components - 1` tokens add one each, so the total is 10^(digits+1) + components - 10. The
/// controlled interpretation must charge one long number and many short components in work that
/// grows with the input, not with the product of the number's length and the component count.
#[test]
fn recovery_duration_charge_grows_linearly_with_a_long_first_number() {
    let mut charges = Vec::new();
    for digits in [1_200_usize, 2_400, 4_800, 9_600] {
        let components = digits / 3;
        let text = format!("{}{}", "9".repeat(digits), "1s".repeat(components));
        let (work, seconds) = interpretation_work_and_seconds(&text);
        let tail = (components - 10).to_string();
        let expected = format!("1{}{}", "0".repeat(digits + 1 - tail.len()), tail);
        assert_eq!(seconds, expected);
        charges.push(work);
    }
    for pair in charges.windows(2) {
        assert!(
            pair[1] * 10 <= pair[0] * 22,
            "doubling the first number and the component count grew the charge from {} to {}: {charges:?}",
            pair[0],
            pair[1]
        );
    }
}
