//! Range language decisions serviced by the real bounded numerical operations.
use surgeist_html::bounded::{
    BestFiniteNumberCursor, HtmlInputRangeNumber as RangeNumber, HtmlInputRangeReplacement as B,
    HtmlInputRangeSanitizationAction as A, HtmlInputRangeSanitizationCursor as Policy,
    HtmlInputRangeSanitizationReply as R, HtmlInputRangeSanitizationReplyError,
    HtmlInputRangeValue as V, RangeBufferHeader, RangeLatticeCursor, RangeMidpointCursor,
    RangeNumericHeaders, RangeNumericReply, RangeNumericRequest,
};
use surgeist_html::{
    DocumentString, HtmlFloatingPointNumber as Number, HtmlInputNumericBoundOrigin,
    HtmlInputNumericContent as C, HtmlInputNumericStep as S, HtmlInputNumericStepBaseOrigin,
    HtmlInputNumericStepOrigin, HtmlValueInterpreter, HtmlValueRecoveryState,
};

fn n(value: f64) -> Number {
    Number::try_from(value).unwrap()
}
fn r(value: f64) -> RangeNumber {
    RangeNumber::try_from(value).unwrap()
}

#[test]
fn calculation_values_preserve_actual_finite_bits_and_reject_nonfinite_values() {
    for bits in [
        0,
        1_u64 << 63,
        1,
        (1_u64 << 63) | 1,
        f64::MAX.to_bits(),
        (-f64::MAX).to_bits(),
    ] {
        assert_eq!(r(f64::from_bits(bits)).value().to_bits(), bits);
    }
    for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        assert_eq!(
            RangeNumber::try_from(value),
            Err(surgeist_html::HtmlNonFiniteNumberError)
        );
    }
}

fn midpoint(a: RangeNumber, b: RangeNumber) -> RangeNumber {
    let mut cursor = RangeMidpointCursor::new(a.value().to_bits(), b.value().to_bits()).unwrap();
    while let Some(request) = cursor.request() {
        assert_eq!(request, RangeNumericRequest::ScalarStep);
        cursor.reply(RangeNumericReply::Ack).unwrap();
    }
    r(f64::from_bits(cursor.result().unwrap()))
}
fn lattice(
    current: RangeNumber,
    min: RangeNumber,
    max: Option<RangeNumber>,
    base: RangeNumber,
    step: RangeNumber,
) -> Option<RangeNumber> {
    let mut cursor = RangeLatticeCursor::new(
        current.value().to_bits(),
        min.value().to_bits(),
        max.map(|n| n.value().to_bits()),
        base.value().to_bits(),
        step.value().to_bits(),
    )
    .unwrap();
    while let Some(request) = cursor.request() {
        assert_eq!(request, RangeNumericRequest::ScalarStep);
        cursor.reply(RangeNumericReply::Ack).unwrap();
    }
    cursor.result().unwrap().map(|bits| r(f64::from_bits(bits)))
}
fn best(value: RangeNumber) -> String {
    let mut cells: [Vec<u8>; 8] = std::array::from_fn(|_| Vec::new());
    let mut output = Vec::new();
    let headers = |cells: &[Vec<u8>; 8], output: &Vec<u16>| {
        RangeNumericHeaders::new(
            std::array::from_fn(|i| {
                RangeBufferHeader::new(cells[i].len(), cells[i].capacity()).unwrap()
            }),
            RangeBufferHeader::new(output.len(), output.capacity()).unwrap(),
        )
    };
    let mut cursor = BestFiniteNumberCursor::new(value.value(), headers(&cells, &output)).unwrap();
    while let Some(request) = cursor.request() {
        let reply = match request {
            RangeNumericRequest::ReadCells { cells: reads } => RangeNumericReply::Cells(
                reads.map(|cell| cell.map(|cell| cells[cell.role().index()][cell.index()])),
            ),
            RangeNumericRequest::Reserve { role, required_len } => {
                let target = &mut cells[role.index()];
                target.reserve_exact(required_len - target.len());
                RangeNumericReply::Reserved {
                    actual_capacity: target.capacity(),
                    actual_len: target.len(),
                }
            }
            RangeNumericRequest::AppendCell { role, value } => {
                cells[role.index()].push(value);
                RangeNumericReply::Ack
            }
            RangeNumericRequest::WriteCell { cell, value } => {
                cells[cell.role().index()][cell.index()] = value;
                RangeNumericReply::Ack
            }
            RangeNumericRequest::PopCell { role } => {
                cells[role.index()].pop().unwrap();
                RangeNumericReply::Ack
            }
            RangeNumericRequest::ScalarStep => RangeNumericReply::Ack,
            RangeNumericRequest::ReserveOutput { required_len } => {
                output.reserve_exact(required_len - output.len());
                RangeNumericReply::OutputReserved {
                    actual_capacity: output.capacity(),
                    actual_len: output.len(),
                }
            }
            RangeNumericRequest::EmitUnit { unit } => {
                output.push(unit);
                RangeNumericReply::Ack
            }
            RangeNumericRequest::SourceUnit { .. } => {
                panic!("best serialization reads no authored source")
            }
        };
        cursor.reply(reply).unwrap();
        assert_eq!(cursor.headers(), headers(&cells, &output));
    }
    assert_eq!(cursor.result(), Some(output.len()));
    let text = String::from_utf16(&output).unwrap();
    let raw = DocumentString::from_utf16(output);
    let report = HtmlValueInterpreter::new(25)
        .floating_point_number(Some(&raw))
        .unwrap();
    let HtmlValueRecoveryState::Value(recovered) = report.recovery() else {
        panic!("finite best serialization must convert");
    };
    let expected = if value.value() == 0.0 {
        0
    } else {
        value.value().to_bits()
    };
    assert_eq!(recovered.value().to_bits(), expected);
    text
}
fn run(
    value: V,
    min: C,
    max: C,
    step: S,
    content: C,
) -> (Vec<(B, String)>, Vec<&'static str>, Policy) {
    let mut p = Policy::new(value, min, max);
    let mut writes = Vec::new();
    let mut reads = Vec::new();
    loop {
        let reply = match p.action() {
            A::Midpoint { minimum, maximum } => R::Midpoint(midpoint(minimum, maximum)),
            A::Replace { value, branch } => {
                writes.push((branch, best(value)));
                R::Applied
            }
            A::ReadStep => {
                reads.push("step");
                R::Step(step)
            }
            A::ReadValueContent => {
                reads.push("value-content");
                R::ValueContent(content)
            }
            A::Lattice {
                current,
                minimum,
                maximum,
                base,
                step,
            } => R::Lattice(lattice(current, minimum, maximum, base, step)),
            A::Complete => break,
        };
        p.reply(reply).unwrap();
    }
    (writes, reads, p)
}

#[test]
fn invalid_defaults_and_reversed_bounds_follow_literal_source_branches() {
    assert_eq!(
        run(V::InvalidGrammar, C::Absent, C::Absent, S::Any, C::Absent).0,
        vec![(B::InvalidDefaultMidpoint, "50".into())]
    );
    assert_eq!(
        run(
            V::InvalidGrammar,
            C::Value(n(80.0)),
            C::Value(n(20.0)),
            S::Any,
            C::Absent
        )
        .0,
        vec![(B::InvalidDefaultMinimum, "80".into())]
    );
    assert!(
        run(
            V::Finite(r(100.0)),
            C::Value(n(80.0)),
            C::Value(n(20.0)),
            S::Any,
            C::Absent
        )
        .0
        .is_empty()
    );
    assert_eq!(
        run(V::Finite(r(-2.0)), C::Absent, C::Absent, S::Any, C::Absent).0,
        vec![(B::Underflow, "0".into())]
    );
    assert_eq!(
        run(V::Finite(r(101.0)), C::Absent, C::Absent, S::Any, C::Absent).0,
        vec![(B::Overflow, "100".into())]
    );
}
#[test]
fn exact_lattice_ties_positive_and_negative_use_existing_numeric_owner() {
    assert_eq!(
        run(
            V::Finite(r(50.0)),
            C::Value(n(0.0)),
            C::Value(n(100.0)),
            S::Value(n(20.0)),
            C::Value(n(50.0))
        )
        .0,
        vec![(B::StepCorrection, "60".into())]
    );
    assert_eq!(
        run(
            V::Finite(r(0.0)),
            C::Value(n(-30.0)),
            C::Value(n(10.0)),
            S::Value(n(20.0)),
            C::Absent
        )
        .0,
        vec![(B::StepCorrection, "10".into())]
    );
    assert_eq!(
        run(
            V::Finite(r(0.3)),
            C::Value(n(0.0)),
            C::Value(n(100.0)),
            S::Value(n(0.1)),
            C::Absent
        )
        .0,
        vec![(B::StepCorrection, "0.2".into())]
    );
    assert_eq!(
        run(
            V::Finite(r(3.0)),
            C::Failed,
            C::Value(n(1.0)),
            S::Value(n(4.0)),
            C::Value(n(3.0))
        )
        .0,
        vec![(B::Overflow, "1".into())]
    );
}
#[test]
fn late_parameter_reads_preserve_base_precedence_and_actual_none_outcomes() {
    let (writes, reads, p) = run(
        V::Finite(r(50.0)),
        C::Absent,
        C::Absent,
        S::Value(n(20.0)),
        C::Value(n(50.0)),
    );
    assert!(writes.is_empty());
    assert_eq!(reads, vec!["step", "value-content"]);
    assert_eq!(
        p.constraints().unwrap().step_base_origin(),
        HtmlInputNumericStepBaseOrigin::ValueContent
    );
    let (_, reads, p) = run(
        V::Finite(r(50.0)),
        C::Value(n(0.0)),
        C::Absent,
        S::Any,
        C::Value(n(50.0)),
    );
    assert_eq!(reads, vec!["step"]);
    assert_eq!(
        p.constraints().unwrap().step_base_origin(),
        HtmlInputNumericStepBaseOrigin::MinimumContent
    );
    let (writes, reads, p) = run(
        V::ConversionFailed,
        C::Absent,
        C::Absent,
        S::Absent,
        C::Absent,
    );
    assert!(writes.is_empty());
    assert!(!p.replaced());
    assert_eq!(reads, vec!["step", "value-content"]);
    for value in [0.0, -0.0, f64::from_bits(1)] {
        assert!(
            run(V::Finite(r(value)), C::Absent, C::Absent, S::Any, C::Absent)
                .0
                .is_empty()
        );
    }
    assert_eq!(
        lattice(r(-0.0), r(0.0), Some(r(100.0)), r(0.0), r(1.0))
            .unwrap()
            .value()
            .to_bits(),
        (-0.0f64).to_bits()
    );
    assert!(
        run(
            V::Finite(r(-0.0)),
            C::Absent,
            C::Absent,
            S::Absent,
            C::Absent
        )
        .0
        .is_empty()
    );
}
#[test]
fn missing_failed_and_nonpositive_steps_keep_origin_qualified_defaults() {
    for (step, origin) in [
        (S::Absent, HtmlInputNumericStepOrigin::MissingDefault),
        (S::Failed, HtmlInputNumericStepOrigin::ParseFailureDefault),
        (
            S::Value(n(0.0)),
            HtmlInputNumericStepOrigin::NonPositiveDefault,
        ),
        (
            S::Value(n(-1.0)),
            HtmlInputNumericStepOrigin::NonPositiveDefault,
        ),
    ] {
        let (writes, _, p) = run(
            V::Finite(r(1.5)),
            C::Value(n(0.0)),
            C::Failed,
            step,
            C::Absent,
        );
        assert_eq!(writes, vec![(B::StepCorrection, "2".into())]);
        assert_eq!(p.constraints().unwrap().step_origin(), origin);
        assert_eq!(
            p.constraints().unwrap().maximum_origin(),
            HtmlInputNumericBoundOrigin::ParseFailureFallback
        );
    }
}
#[test]
fn midpoint_avoids_finite_intermediate_overflow_and_best_output_roundtrips() {
    assert_eq!(
        run(
            V::InvalidGrammar,
            C::Value(n(-f64::MAX)),
            C::Value(n(f64::MAX)),
            S::Any,
            C::Absent
        )
        .0,
        vec![(B::InvalidDefaultMidpoint, "0".into())]
    );
    assert_eq!(
        run(
            V::InvalidGrammar,
            C::Value(n(f64::MAX)),
            C::Value(n(f64::MAX)),
            S::Any,
            C::Absent
        )
        .0,
        vec![(B::InvalidDefaultMidpoint, "1.7976931348623157e+308".into())]
    );
}
#[test]
fn wrong_phase_and_out_of_bound_replies_preserve_the_actual_policy() {
    let mut p = Policy::new(V::InvalidGrammar, C::Absent, C::Absent);
    assert_eq!(p.constraints(), None);
    let before = (p.action(), p.constraints(), p.bound_origins(), p.replaced());
    assert_eq!(
        p.reply(R::Applied),
        Err(HtmlInputRangeSanitizationReplyError)
    );
    assert_eq!(
        (p.action(), p.constraints(), p.bound_origins(), p.replaced()),
        before
    );
    assert_eq!(
        p.reply(R::Midpoint(r(-1.0))),
        Err(HtmlInputRangeSanitizationReplyError)
    );
    assert_eq!(
        (p.action(), p.constraints(), p.bound_origins(), p.replaced()),
        before
    );
    let (_, _, mut done) = run(V::ConversionFailed, C::Absent, C::Absent, S::Any, C::Absent);
    let before = (
        done.action(),
        done.constraints(),
        done.bound_origins(),
        done.replaced(),
    );
    assert_eq!(
        done.reply(R::Applied),
        Err(HtmlInputRangeSanitizationReplyError)
    );
    assert_eq!(
        (
            done.action(),
            done.constraints(),
            done.bound_origins(),
            done.replaced()
        ),
        before
    );
}

#[test]
fn out_of_bound_lattice_reply_preserves_pending_parameters_and_allows_actual_result() {
    let mut policy = Policy::new(V::Finite(r(50.0)), C::Value(n(0.0)), C::Value(n(100.0)));
    assert_eq!(policy.action(), A::ReadStep);
    policy.reply(R::Step(S::Value(n(20.0)))).unwrap();
    let pending = policy.action();
    assert_eq!(
        pending,
        A::Lattice {
            current: r(50.0),
            minimum: r(0.0),
            maximum: Some(r(100.0)),
            base: r(0.0),
            step: r(20.0),
        }
    );
    assert_eq!(
        policy.reply(R::Lattice(Some(r(101.0)))),
        Err(HtmlInputRangeSanitizationReplyError)
    );
    assert_eq!(policy.action(), pending);
    assert!(!policy.replaced());
    let selected = lattice(r(50.0), r(0.0), Some(r(100.0)), r(0.0), r(20.0));
    assert_eq!(selected, Some(r(60.0)));
    policy.reply(R::Lattice(selected)).unwrap();
    assert_eq!(
        policy.action(),
        A::Replace {
            value: r(60.0),
            branch: B::StepCorrection,
        }
    );
    assert_eq!(best(r(60.0)), "60");
    policy.reply(R::Applied).unwrap();
    assert_eq!(policy.action(), A::Complete);
    assert!(policy.replaced());
}
