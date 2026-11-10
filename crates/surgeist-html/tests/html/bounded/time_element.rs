//! Public retained time interpretation, using the pinned time-element rules.
#![forbid(unsafe_code)]
use crate::html::support::Recorder;
use surgeist_html::{
    accounting::HtmlWork,
    bounded::{
        numeric::{HtmlIntegerUnitStepError, HtmlNumericMaterializationError},
        temporal::{
            HtmlTimeElementInterpretation as Interpretation,
            HtmlTimeElementInterpretationActionKind as Kind, HtmlTimeElementInterpretationError,
        },
    },
    microsyntax::temporal::{HtmlMachineReadableTime, HtmlTimeElementValue},
    values::{
        DocumentString, HtmlValueAuthorState, HtmlValueInterpreter, HtmlValueRecoveryState,
        HtmlValueReport,
    },
};

/// Finishes the owner in `slot`. A refused finish hands the owner back, so it returns to the slot
/// for the test to keep driving, and success leaves the slot empty.
fn try_finish<'a>(
    slot: &mut Option<Interpretation<'a>>,
) -> Result<
    HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime>,
    HtmlTimeElementInterpretationError,
> {
    let owner = slot.take().expect("an owner to finish");
    owner.finish().map_err(|failure| {
        let (owner, cause) = failure.into_parts();
        *slot = Some(owner);
        cause
    })
}

fn drive(owner: &mut Interpretation<'_>) -> (usize, usize) {
    let mut meter = Recorder::default();
    let mut capacity = 0;
    for _ in 0..100_000 {
        let Some(action) = owner.next_action().unwrap() else {
            return (meter.total(HtmlWork::SemanticStep) as usize, capacity);
        };
        let layout = action.layout();
        let step = action.advance(&mut meter, 100_000, usize::MAX).unwrap();
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
    assert!(try_finish(&mut owner).unwrap_err().is_not_ready());
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
}

#[test]
fn publication_requires_admitted_report_assembly_after_all_attempts() {
    let raw = DocumentString::from("0001");
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(4)).unwrap());
    let mut meter = Recorder::default();
    let mut capacity = 0;
    let mut numeric = 0;
    loop {
        let action = owner.as_mut().unwrap().next_action().unwrap().unwrap();
        let layout = action.layout();
        if layout.kind() == Kind::ReportAssembly {
            break;
        }
        numeric += usize::from(layout.can_allocate());
        let step = action.advance(&mut meter, 4, usize::MAX).unwrap();
        capacity += step.capacity_bytes();
        step.outcome().unwrap();
    }
    assert!(try_finish(&mut owner).unwrap_err().is_not_ready());
    assert!(std::ptr::eq(owner.as_ref().unwrap().raw(), &raw));
    let action = owner.as_mut().unwrap().next_action().unwrap().unwrap();
    assert_eq!(action.layout().kind(), Kind::ReportAssembly);
    let step = action.advance(&mut meter, 0, 0).unwrap();
    let work = meter.total(HtmlWork::SemanticStep);
    assert_eq!(step.capacity_bytes(), 0);
    step.outcome().unwrap();
    // Source-basis characterization: nine alternatives in each phase at
    // 16*4+256 work, two begin/year assemblies, two 5*4+8 digit operations,
    // and one report assembly. No extra grammar pass or fee is introduced.
    assert_eq!(work, 5821);
    assert_eq!(numeric, 2);
    assert_eq!(capacity, 2);
    let report = try_finish(&mut owner).unwrap();
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
        let report = try_finish(&mut owner).unwrap();
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
        let report = try_finish(&mut owner).unwrap();
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
        let step = action.advance(&mut Recorder::default(), 100, 1).unwrap();
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
    assert_eq!(try_finish(&mut owner).unwrap_err(), error);
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
            action
                .advance(&mut Recorder::default(), 100, usize::MAX)
                .unwrap()
                .outcome()
                .unwrap();
        }
        if completed {
            break;
        }
        // The caller's stop performs no next operation; recover the retained raw.
        assert!(std::ptr::eq(owner.take().unwrap().abort(), &raw));
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
        let step = action
            .advance(&mut Recorder::default(), 4, usize::MAX)
            .unwrap();
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
    assert_eq!(try_finish(&mut owner).unwrap_err(), error);
    assert!(std::ptr::eq(owner.take().unwrap().abort(), &raw));
}

#[test]
fn completed_invalid_syntax_is_a_report_without_processing_failure_or_allocation() {
    let raw = DocumentString::from_utf16([49, 0xd800, 0, 50]);
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
    let (_, capacity) = drive(owner.as_mut().unwrap());
    assert_eq!(capacity, 0);
    let report = try_finish(&mut owner).unwrap();
    assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
    assert!(matches!(
        report.recovery(),
        HtmlValueRecoveryState::Failed(_)
    ));
    assert!(report.raw().unwrap().shares_storage_with(&raw));
}

/// Total work charged to interpret `text` through both phases.
fn interpretation_work(text: &str) -> usize {
    let raw = DocumentString::from(text);
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
    let (work, _) = drive(owner.as_mut().unwrap());
    let report = try_finish(&mut owner).unwrap();
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
    let report = try_finish(&mut owner).unwrap();
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

/// A literal transcription of "parse a duration string" (foundations capture lines 3350-3430),
/// with schoolbook decimal arithmetic that shares nothing with the crate. It returns the
/// canonical integer and fractional digits of the seconds, or `None` where the algorithm fails.
mod spec_duration {
    #[derive(Clone, Copy, PartialEq)]
    enum Units {
        Years,
        Months,
        Weeks,
        Days,
        Hours,
        Minutes,
        Seconds,
    }

    /// An unsigned number as little-endian decimal digits, scaled by 10^scale.
    fn scaled(integer: &str, fraction: &str, scale: usize) -> Vec<u8> {
        let mut text = String::from(integer);
        text.push_str(fraction);
        text.push_str(&"0".repeat(scale - fraction.len()));
        text.bytes().rev().map(|digit| digit - b'0').collect()
    }

    fn multiply(value: &[u8], multiplier: u64) -> Vec<u8> {
        let mut result = Vec::with_capacity(value.len() + 20);
        let mut carry = 0_u128;
        for &digit in value {
            let product = u128::from(digit) * u128::from(multiplier) + carry;
            result.push((product % 10) as u8);
            carry = product / 10;
        }
        while carry != 0 {
            result.push((carry % 10) as u8);
            carry /= 10;
        }
        result
    }

    fn add(left: &[u8], right: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(left.len().max(right.len()) + 1);
        let mut carry = 0;
        for index in 0..left.len().max(right.len()) {
            let total = left.get(index).copied().unwrap_or(0)
                + right.get(index).copied().unwrap_or(0)
                + carry;
            result.push(total % 10);
            carry = total / 10;
        }
        if carry != 0 {
            result.push(carry);
        }
        result
    }

    fn is_whitespace(c: char) -> bool {
        matches!(c, '\t' | '\n' | '\u{c}' | '\r' | ' ')
    }

    fn skip_whitespace(input: &[char], position: &mut usize) {
        while *position < input.len() && is_whitespace(input[*position]) {
            *position += 1;
        }
    }

    fn collect_digits(input: &[char], position: &mut usize) -> String {
        let start = *position;
        while *position < input.len() && input[*position].is_ascii_digit() {
            *position += 1;
        }
        input[start..*position].iter().collect()
    }

    pub fn parse(text: &str) -> Option<(String, String)> {
        let input: Vec<char> = text.chars().collect();
        let mut position = 0;
        let mut m_is_months = false;
        let mut component_count = 0;
        // Each component's number as (integer, fraction) digits with its multiplier into seconds
        // or, for months, into months.
        let mut seconds_terms: Vec<(String, String, u64)> = Vec::new();
        let mut months_nonzero = false;
        skip_whitespace(&input, &mut position);
        if position >= input.len() {
            return None;
        }
        if input[position] == 'P' {
            position += 1;
            m_is_months = true;
            skip_whitespace(&input, &mut position);
        }
        loop {
            if position >= input.len() {
                break;
            }
            if input[position] == 'T' {
                position += 1;
                m_is_months = false;
                skip_whitespace(&input, &mut position);
                continue;
            }
            let mut next = input[position];
            let integer = if next == '.' {
                String::from("0")
            } else if next.is_ascii_digit() {
                collect_digits(&input, &mut position)
            } else {
                return None;
            };
            if position >= input.len() {
                return None;
            }
            next = input[position];
            position += 1;
            let (fraction, units) = if next == '.' {
                let fraction = collect_digits(&input, &mut position);
                if fraction.is_empty() {
                    return None;
                }
                skip_whitespace(&input, &mut position);
                if position >= input.len() {
                    return None;
                }
                next = input[position];
                position += 1;
                if next != 'S' && next != 's' {
                    return None;
                }
                (fraction, Units::Seconds)
            } else {
                if is_whitespace(next) {
                    skip_whitespace(&input, &mut position);
                    if position >= input.len() {
                        return None;
                    }
                    next = input[position];
                    position += 1;
                }
                let units = match next {
                    'Y' | 'y' => {
                        m_is_months = true;
                        Units::Years
                    }
                    'M' | 'm' if m_is_months => Units::Months,
                    'W' | 'w' => {
                        m_is_months = false;
                        Units::Weeks
                    }
                    'D' | 'd' => {
                        m_is_months = false;
                        Units::Days
                    }
                    'H' | 'h' => {
                        m_is_months = false;
                        Units::Hours
                    }
                    'M' | 'm' => Units::Minutes,
                    'S' | 's' => {
                        m_is_months = false;
                        Units::Seconds
                    }
                    _ => return None,
                };
                (String::new(), units)
            };
            component_count += 1;
            let nonzero = integer.bytes().chain(fraction.bytes()).any(|d| d != b'0');
            let multiplier = match units {
                Units::Years | Units::Months => {
                    months_nonzero |= nonzero;
                    0
                }
                Units::Weeks => 7 * 24 * 60 * 60,
                Units::Days => 24 * 60 * 60,
                Units::Hours => 60 * 60,
                Units::Minutes => 60,
                Units::Seconds => 1,
            };
            if multiplier != 0 {
                seconds_terms.push((integer, fraction, multiplier));
            }
            skip_whitespace(&input, &mut position);
        }
        if component_count == 0 || months_nonzero {
            return None;
        }
        let scale = seconds_terms
            .iter()
            .map(|(_, fraction, _)| fraction.len())
            .max()
            .unwrap_or(0);
        let mut seconds = vec![0];
        for (integer, fraction, multiplier) in &seconds_terms {
            seconds = add(
                &seconds,
                &multiply(&scaled(integer, fraction, scale), *multiplier),
            );
        }
        while seconds.len() <= scale {
            seconds.push(0);
        }
        let digits: String = seconds
            .iter()
            .rev()
            .map(|&d| char::from(b'0' + d))
            .collect();
        let (integer, fraction) = digits.split_at(digits.len() - scale);
        let integer = integer.trim_start_matches('0');
        Some((
            if integer.is_empty() { "0" } else { integer }.to_owned(),
            fraction.trim_end_matches('0').to_owned(),
        ))
    }
}

/// The report that the controlled interpretation publishes for `text`.
fn controlled_report(
    text: &str,
) -> surgeist_html::values::HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime> {
    let raw = DocumentString::from(text);
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
    drive(owner.as_mut().unwrap());
    try_finish(&mut owner).unwrap()
}

/// Checks `text` against the ordinary path and the literal algorithm.
fn assert_controlled_duration_matches(text: &str) {
    let controlled = controlled_report(text);
    let raw = DocumentString::from(text);
    let ordinary = HtmlValueInterpreter::new(raw.len())
        .time_element_value(Some(&raw))
        .unwrap();
    assert_eq!(controlled, ordinary, "{text:?}");
    match (spec_duration::parse(text), controlled.recovery()) {
        (
            Some((integer, fraction)),
            HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Duration(duration)),
        ) => {
            assert_eq!(
                duration.seconds().integer_part().decimal_digits(),
                integer,
                "{text:?}"
            );
            let expected = (!fraction.is_empty()).then_some(fraction.as_str());
            assert_eq!(duration.seconds().fractional_digits(), expected, "{text:?}");
        }
        (None, HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Duration(_))) => {
            panic!("{text:?}: the algorithm fails, yet a duration was recovered")
        }
        (Some(expected), other) => {
            panic!("{text:?}: expected the duration {expected:?}, got {other:?}")
        }
        (None, _) => {}
    }
}

/// Hand-derived totals, which also pin the literal algorithm above.
#[test]
fn controlled_duration_totals_match_hand_derived_values() {
    for (text, integer, fraction) in [
        ("0s", "0", None),
        ("1s1s", "2", None),
        ("0.5s0.5s", "1", None),
        ("9.99s0.01s", "10", None),
        ("0.0001s0.9999s", "1", None),
        ("1h1m1s", "3661", None),
        ("1w1d", "691200", None),
        ("1.5s2w", "1209601", Some("5")),
        ("1d 25H", "176400", None),
        ("PT1H30M", "5400", None),
        ("99999s1s", "100000", None),
        ("0.125s0.125s0.125s", "0", Some("375")),
        ("3 s\t1\u{c}s", "4", None),
    ] {
        assert_eq!(
            spec_duration::parse(text),
            Some((integer.to_owned(), fraction.unwrap_or("").to_owned())),
            "{text:?}"
        );
        let controlled = controlled_report(text);
        let HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Duration(duration)) =
            controlled.recovery()
        else {
            panic!("{text:?}: expected a duration");
        };
        assert_eq!(duration.seconds().integer_part().decimal_digits(), integer);
        assert_eq!(duration.seconds().fractional_digits(), fraction, "{text:?}");
        assert_controlled_duration_matches(text);
    }
    for text in [
        "", " ", "P", "T", "1", "1y", "1m1y", "P1M", "1.s", ".s", "1 ", "s", "1x",
    ] {
        assert_eq!(spec_duration::parse(text), None, "{text:?}");
        assert_controlled_duration_matches(text);
    }
}

/// A small deterministic generator (xorshift64*), so that the cases never change.
struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    fn below(&mut self, bound: usize) -> usize {
        (self.next() >> 11) as usize % bound
    }
    fn digits(&mut self, length: usize) -> String {
        let style = self.below(4);
        (0..length)
            .map(|_| match style {
                0 => '9',
                1 => char::from(b'0' + self.below(10) as u8),
                2 => ['0', '9'][self.below(2)],
                _ => ['0', '1', '9'][self.below(3)],
            })
            .collect()
    }
    fn whitespace(&mut self) -> String {
        let length = [0, 0, 0, 1, 1, 2][self.below(6)];
        (0..length)
            .map(|_| ['\t', '\n', '\u{c}', '\r', ' '][self.below(5)])
            .collect()
    }
}

fn random_duration(random: &mut Random) -> String {
    let mut text = random.whitespace();
    if random.below(4) == 0 {
        text.push('P');
        text.push_str(&random.whitespace());
    }
    for _ in 0..1 + random.below(12) {
        if random.below(6) == 0 {
            text.push('T');
            text.push_str(&random.whitespace());
        }
        let long = random.below(8) == 0;
        let length = if long {
            20 + random.below(200)
        } else {
            1 + random.below(12)
        };
        if random.below(10) != 0 {
            text.push_str(&random.digits(length));
        }
        if random.below(3) == 0 {
            text.push('.');
            let length = if long {
                random.below(120)
            } else {
                random.below(14)
            };
            text.push_str(&random.digits(length));
            text.push_str(&random.whitespace());
            text.push(['s', 'S'][random.below(2)]);
        } else {
            text.push_str(&random.whitespace());
            text.push_str(
                ["s", "S", "m", "M", "h", "H", "d", "D", "w", "W", "y", "Y"][random.below(12)],
            );
        }
        text.push_str(&random.whitespace());
    }
    // Some strings are damaged, to reach the failures.
    if random.below(6) == 0 {
        let at = random.below(text.len() + 1);
        text.insert(
            at,
            ['x', '.', 'T', 'P', '-', ' ', '0', 'm'][random.below(8)],
        );
    }
    if random.below(10) == 0 {
        let at = random.below(text.len() + 1);
        text.truncate(at);
    }
    text
}

/// Random lenient durations, valid and damaged, equal the ordinary path and the literal algorithm
/// in the author state, the recovered value and the failure.
#[test]
fn controlled_duration_equals_the_ordinary_path_and_the_literal_algorithm_on_random_inputs() {
    let mut random = Random(0x9e37_79b9_7f4a_7c15);
    let mut recovered = 0;
    for _ in 0..4_000 {
        let text = random_duration(&mut random);
        if spec_duration::parse(&text).is_some() {
            recovered += 1;
        }
        assert_controlled_duration_matches(&text);
    }
    // The generator is deterministic, so it reaches the same successes and failures every run.
    assert_eq!(recovered, 1_083);
}

/// The failure that driving `text` under the digit limit reaches, with the number of steps taken.
fn digit_limited_failure(
    text: &str,
    max_digits: usize,
) -> (usize, HtmlTimeElementInterpretationError) {
    let raw = DocumentString::from(text);
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
    for steps in 0..100_000 {
        let action = owner.as_mut().unwrap().next_action().unwrap().unwrap();
        if let Err(error) = action
            .advance(&mut Recorder::default(), max_digits, usize::MAX)
            .unwrap()
            .outcome()
        {
            assert_eq!(try_finish(&mut owner).unwrap_err(), error);
            return (steps, error);
        }
    }
    panic!("{text:?} reached no failure");
}

/// The running total is held to the digit limit exactly where a sum first needs one more digit,
/// whether the carry that adds it is short or longer than one step walks.
#[test]
fn a_running_duration_total_fails_at_the_addition_that_exceeds_the_digit_limit() {
    for (text, limit, required) in [
        ("99s1s", 2, 3),
        ("99s1s100s", 2, 3),
        (&format!("{}s1s", "9".repeat(40)), 40, 41),
        (&format!("{}s1s", "9".repeat(400)), 400, 401),
    ] {
        let (_, error) = digit_limited_failure(text, limit);
        assert!(
            matches!(
                error.processing_failure(),
                Some(HtmlNumericMaterializationError::Boundary(
                    HtmlIntegerUnitStepError::DigitLimitExceeded { limit: l, required: r }
                )) if l == limit && r == required
            ),
            "{text:?}: {error:?}"
        );
    }
    // The same number without the final carry stays within the limit.
    let raw = DocumentString::from(format!("{}s0s", "9".repeat(400)).as_str());
    let mut owner = Some(Interpretation::new(&raw, HtmlValueInterpreter::new(raw.len())).unwrap());
    while let Some(action) = owner.as_mut().unwrap().next_action().unwrap() {
        action
            .advance(&mut Recorder::default(), 400, usize::MAX)
            .unwrap()
            .outcome()
            .unwrap();
    }
    let report = try_finish(&mut owner).unwrap();
    assert!(matches!(
        report.recovery(),
        HtmlValueRecoveryState::Value(HtmlMachineReadableTime::Duration(_))
    ));
}
