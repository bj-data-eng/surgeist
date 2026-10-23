//! Native numerical use and actual caller-owned continuation storage.
use surgeist_color::format::{FiniteFormatter, FormatAction, FormatStep, RoundingMode};
use surgeist_color::{
    ActionLayout, Color, ColorSpace as Space, Computation, DigitSlot, Finite, InterpolationMethod,
    Kernel, NumericalRequest, OperationOwner, PredefinedColorSpace as Predefined, Serialization,
    SerializationAction as Action, SerializationError, SerializationFamily, SerializationLimits,
    SerializationRequest, SerializationResource, SerializationStep, Transformation, UnitInterval,
    serialize,
};

fn color(space: Space, channels: [Option<f64>; 3], alpha: Option<f64>) -> Color {
    Color::new(
        space,
        channels.map(|channel| channel.map(|value| Finite::new(value).unwrap())),
        alpha.map(|value| UnitInterval::new(value).unwrap()),
    )
}

fn limits() -> SerializationLimits {
    SerializationLimits {
        actions: 100_000,
        digits: 2048,
        output_bytes: 4096,
    }
}
fn spelling(value: Color, request: SerializationRequest) -> String {
    serialize(value, request, limits()).unwrap()
}
fn pending(serialization: Serialization<'_>) -> Action<'_> {
    match serialization.step() {
        SerializationStep::Pending(action) => action,
        SerializationStep::Complete(_) => panic!("the serialization has a pending action"),
    }
}
/// Services every action, first declining each one to show that a refused
/// admission does no work.
fn finish<'owner>(
    mut serialization: Serialization<'owner>,
    digits: &mut Vec<DigitSlot<'owner>>,
    output: &mut Vec<u8>,
) -> surgeist_color::SerializationReceipt {
    for _ in 0..1_000_000 {
        let action = match serialization.step() {
            SerializationStep::Complete(receipt) => return receipt,
            SerializationStep::Pending(action) => action,
        };
        let layout = action.layout();
        let refused = action.into_serialization();
        let action = pending(refused);
        assert_eq!(action.layout(), layout, "declining an action does no work");
        serialization = match action {
            Action::Compute(compute) => {
                assert_eq!(
                    layout,
                    ActionLayout {
                        read_bytes: 0,
                        write_bytes: 0,
                        output_bytes: 0
                    }
                );
                compute.compute()
            }
            Action::ReadDigit(read) => {
                let slot = &digits[read.index()];
                read.read(slot)
            }
            Action::AppendDigit(write) => {
                assert_eq!(write.index(), digits.len());
                assert_eq!(layout.write_bytes, std::mem::size_of::<DigitSlot<'_>>());
                let (record, receipt) = write.record();
                digits.push(record);
                receipt.stored(&digits[digits.len() - 1])
            }
            Action::ReplaceDigit(write) => {
                let index = write.index();
                let (record, receipt) = write.record();
                digits[index] = record;
                receipt.stored(&digits[index])
            }
            Action::Emit(emit) => {
                assert!(emit.unit().is_ascii());
                assert_eq!(layout.output_bytes, 1);
                output.push(emit.unit());
                emit.emitted()
            }
        };
    }
    panic!("finite serialization did not terminate")
}

#[test]
fn computation_computes_once_and_declining_returns_the_exact_request() {
    let mut owner = OperationOwner::new();
    let original = color(Space::Lab, [Some(-0.0), None, Some(2.0)], None);
    let request = NumericalRequest::Transform {
        value: original,
        transformation: Transformation::ForceOpaqueAlpha,
    };
    let computation = Computation::new(&mut owner, request);
    assert_eq!(
        computation.kernel(),
        Kernel::Transform(Transformation::ForceOpaqueAlpha)
    );
    assert_eq!(computation.layout().read_bytes, 0);
    let result = computation.compute();
    assert_eq!(result.alpha(), Some(1.0));
    assert_eq!(
        result.channels()[0].unwrap().to_bits(),
        (-0.0_f64).to_bits()
    );
    let NumericalRequest::Transform {
        value,
        transformation,
    } = Computation::new(&mut owner, request).into_request()
    else {
        panic!("original transformation")
    };
    assert!(value.identical(original));
    assert_eq!(transformation, Transformation::ForceOpaqueAlpha);
}

#[test]
fn proven_operand_requests_compute_once_and_keep_their_bits() {
    // Construction over proof-typed operands is total; declining returns the
    // exact request.
    let mut owner = OperationOwner::new();
    let request = NumericalRequest::Construct {
        space: Space::Lab,
        channels: [Finite::new(-0.0), Finite::new(f64::MAX), None],
        alpha: UnitInterval::new(-0.0),
    };
    let value = Computation::new(&mut owner, request).compute();
    assert_eq!(value.channels()[0].unwrap().to_bits(), (-0.0_f64).to_bits());
    assert_eq!(value.channels()[1], Some(f64::MAX));
    assert_eq!(value.alpha().unwrap().to_bits(), (-0.0_f64).to_bits());
    let NumericalRequest::Construct { channels, .. } =
        Computation::new(&mut owner, request).into_request()
    else {
        panic!("original construction")
    };
    assert_eq!(channels[1].map(Finite::get), Some(f64::MAX));
}

#[test]
fn an_overflowing_interpolation_request_computes_its_closest_finite_value() {
    // Converting sRGB red at f64::MAX to XYZ exceeds binary64 in every row;
    // the operand prepares as f64::MAX and an even mix with black halves it.
    let mut owner = OperationOwner::new();
    let red = color(
        Space::Predefined(Predefined::Srgb),
        [Some(f64::MAX), Some(0.0), Some(0.0)],
        Some(1.0),
    );
    let request = NumericalRequest::Interpolate {
        first: red,
        second: Color::opaque_black(),
        method: InterpolationMethod::try_new(Space::Predefined(Predefined::XyzD65), None).unwrap(),
        progress: UnitInterval::new(0.5).unwrap(),
    };
    let mixed = Computation::new(&mut owner, request).compute();
    assert_eq!(mixed.channels(), [Some(f64::MAX / 2.0); 3]);
    assert!(mixed.identical(request.execute()));
}

#[test]
fn native_serialization_completes_with_its_receipt_and_releases_the_owner() {
    eprintln!(
        "native compiled layouts: owner={} digit={} computation={} serialization={} serialization_action={} finite_formatter={} finite_action={}",
        OperationOwner::layout_bytes(),
        std::mem::size_of::<DigitSlot<'_>>(),
        Computation::register_bytes(),
        Serialization::register_bytes(),
        Serialization::action_bytes(),
        FiniteFormatter::register_bytes(),
        FiniteFormatter::action_bytes()
    );
    let value = Color::from_srgba8([255, 0, 255, 255]);
    let mut owner = OperationOwner::new();
    let mut digits = Vec::new();
    let mut output = Vec::new();
    let receipt = finish(
        Serialization::new(&mut owner, value, SerializationRequest::Css),
        &mut digits,
        &mut output,
    );
    assert_eq!(output, b"rgb(255, 0, 255)");
    assert_eq!(receipt.length(), output.len());
    assert!(receipt.value().identical(value));
    drop(digits);
    let second = Computation::new(
        &mut owner,
        NumericalRequest::WithAlpha { value, alpha: None },
    );
    assert_eq!(second.compute().alpha(), None);
    assert!(
        receipt.value().identical(value),
        "receipt has no numerical owner borrow"
    );
}

/// Services actions up to the first digit append and returns it.
fn reach_append(mut serialization: Serialization<'_>) -> surgeist_color::WriteDigit<'_> {
    loop {
        serialization = match pending(serialization) {
            Action::Compute(compute) => compute.compute(),
            Action::Emit(emit) => emit.emitted(),
            Action::AppendDigit(write) => return write,
            _ => panic!("a coefficient digit precedes every read"),
        };
    }
}
/// Services actions up to the first digit read, storing every appended record.
fn reach_read<'owner>(
    mut serialization: Serialization<'owner>,
    digits: &mut Vec<DigitSlot<'owner>>,
) -> surgeist_color::ReadDigit<'owner> {
    loop {
        serialization = match pending(serialization) {
            Action::Compute(compute) => compute.compute(),
            Action::Emit(emit) => emit.emitted(),
            Action::AppendDigit(write) => {
                let (record, receipt) = write.record();
                digits.push(record);
                receipt.stored(&digits[digits.len() - 1])
            }
            Action::ReadDigit(read) => return read,
            Action::ReplaceDigit(_) => panic!("an integer coefficient is never replaced"),
        };
    }
}

#[test]
fn refused_storage_keeps_the_same_write_pending() {
    let value = Color::from_srgba8([255, 0, 0, 255]);
    let mut owner = OperationOwner::new();
    let write = reach_append(Serialization::new(
        &mut owner,
        value,
        SerializationRequest::Css,
    ));
    let index = write.index();
    // The caller refuses storage: the minted record is never stored.
    let (_refused, receipt) = write.record();
    let Action::AppendDigit(write) = pending(receipt.into_serialization()) else {
        panic!("refused storage waits at the same write")
    };
    assert_eq!(write.index(), index);
    let mut digits = Vec::new();
    let (record, receipt) = write.record();
    digits.push(record);
    let serialization = receipt.stored(&digits[index]);
    assert!(
        !matches!(pending(serialization), Action::AppendDigit(write) if write.index() == index),
        "an acknowledged write advances"
    );
}

#[test]
#[should_panic(expected = "digit record belongs to another numerical operation")]
fn reading_a_digit_record_from_another_serialization_panics() {
    // P3: a foreign digit can only come from a defect in the driving code.
    let value = Color::from_srgba8([255, 0, 0, 255]);
    let mut owner = OperationOwner::new();
    let mut foreign_owner = OperationOwner::new();
    let (foreign, _receipt) = reach_append(Serialization::new(
        &mut foreign_owner,
        value,
        SerializationRequest::Css,
    ))
    .record();
    let mut digits = Vec::new();
    let read = reach_read(
        Serialization::new(&mut owner, value, SerializationRequest::Css),
        &mut digits,
    );
    let _ = read.read(&foreign);
}

#[test]
#[should_panic(expected = "digit record belongs to another numerical operation")]
fn acknowledging_storage_of_another_serializations_record_panics() {
    let value = Color::from_srgba8([255, 0, 0, 255]);
    let mut owner = OperationOwner::new();
    let mut foreign_owner = OperationOwner::new();
    let (foreign, _foreign_receipt) = reach_append(Serialization::new(
        &mut foreign_owner,
        value,
        SerializationRequest::Css,
    ))
    .record();
    let (_local, receipt) = reach_append(Serialization::new(
        &mut owner,
        value,
        SerializationRequest::Css,
    ))
    .record();
    let _ = receipt.stored(&foreign);
}

#[test]
#[should_panic(expected = "digit record was minted for index 0, not the requested index")]
fn reading_a_digit_record_at_another_index_panics() {
    // 255 has three coefficient digits; its first emitted digit is the last
    // appended one, so index 0 is not the first read.
    let value = Color::from_srgba8([255, 0, 0, 255]);
    let mut owner = OperationOwner::new();
    let mut digits = Vec::new();
    let read = reach_read(
        Serialization::new(&mut owner, value, SerializationRequest::Css),
        &mut digits,
    );
    assert_ne!(read.index(), 0);
    let _ = read.read(&digits[0]);
}

#[test]
fn declining_an_emission_leaves_the_same_byte_pending() {
    let mut owner = OperationOwner::new();
    let value = Color::from_srgba8([255, 0, 0, 255]);
    let mut serialization =
        Serialization::new(&mut owner, value, SerializationRequest::HtmlCompatible);
    let emit = loop {
        serialization = match pending(serialization) {
            Action::Compute(compute) => compute.compute(),
            Action::Emit(emit) => break emit,
            _ => panic!("the hexadecimal form reads no digit before its first byte"),
        };
    };
    assert_eq!(emit.unit(), b'#');
    let Action::Emit(emit) = pending(Action::Emit(emit).into_serialization()) else {
        panic!("a declined emission stays pending")
    };
    assert_eq!(emit.unit(), b'#');
    let mut serialization = emit.emitted();
    let next = loop {
        serialization = match pending(serialization) {
            Action::Compute(compute) => compute.compute(),
            Action::Emit(emit) => break emit,
            _ => panic!("hexadecimal digits follow the hash"),
        };
    };
    assert_eq!(next.unit(), b'f');
}

#[test]
fn byte_proof_and_floating_alpha_determine_native_spelling() {
    let value = Color::from_srgba8([255, 0, 255, 255]);
    assert_eq!(
        spelling(value, SerializationRequest::HtmlCompatible),
        "#ff00ff"
    );
    assert_eq!(
        spelling(value, SerializationRequest::Css),
        "rgb(255, 0, 255)"
    );
    assert_eq!(
        spelling(
            value.with_color_function(),
            SerializationRequest::HtmlCompatible
        ),
        "#ff00ff"
    );
    let floating = color(value.space(), value.channels(), value.alpha());
    assert_eq!(
        spelling(floating, SerializationRequest::HtmlCompatible),
        "color(srgb 1 0 1)"
    );
    for (byte, expected) in [(236, "rgba(0, 0, 0, 0.925)"), (237, "rgba(0, 0, 0, 0.93)")] {
        assert_eq!(
            spelling(
                Color::from_srgba8([0, 0, 0, byte]),
                SerializationRequest::Css
            ),
            expected
        );
    }
    let floating = color(value.space(), value.channels(), Some(237.0 / 255.0));
    assert_eq!(
        spelling(floating, SerializationRequest::Css),
        "color(srgb 1 0 1 / 0.929412)"
    );
}

#[test]
fn missing_and_extended_native_coordinates_have_selected_natural_forms() {
    for (space, channels, alpha, expected) in [
        (
            Space::Hsl,
            [None, Some(50.0), Some(20.0)],
            None,
            "hsl(none 50% 20% / none)",
        ),
        (
            Space::Hwb,
            [Some(270.0), None, Some(2.0)],
            Some(1.0),
            "hwb(270 none 2%)",
        ),
        (
            Space::Lab,
            [Some(29.0), Some(39.0), Some(20.0)],
            Some(1.0),
            "lab(29 39 20)",
        ),
        (
            Space::Predefined(Predefined::DisplayP3),
            [Some(-0.2), Some(1.2), None],
            Some(0.5),
            "color(display-p3 -0.2 1.2 none / 0.5)",
        ),
        (
            Space::Predefined(Predefined::XyzD65),
            [Some(1.0), Some(2.0), Some(3.0)],
            Some(1.0),
            "color(xyz-d65 1 2 3)",
        ),
    ] {
        assert_eq!(
            spelling(color(space, channels, alpha), SerializationRequest::Css),
            expected
        );
    }
}

#[test]
fn ordinary_native_output_has_exact_one_short_limit() {
    let value = Color::from_srgba8([255, 0, 0, 255]);
    let mut allowance = limits();
    allowance.output_bytes = 6;
    assert_eq!(
        serialize(value, SerializationRequest::HtmlCompatible, allowance),
        Err(SerializationError::Limit {
            resource: SerializationResource::OutputBytes,
            required: 7,
            allowed: 6
        })
    );
    allowance.output_bytes = 7;
    assert_eq!(
        serialize(value, SerializationRequest::HtmlCompatible, allowance).unwrap(),
        "#ff0000"
    );
    allowance.actions = 0;
    assert_eq!(
        serialize(value, SerializationRequest::HtmlCompatible, allowance),
        Err(SerializationError::Limit {
            resource: SerializationResource::Actions,
            required: 1,
            allowed: 0
        })
    );
    allowance = limits();
    allowance.digits = 0;
    assert_eq!(
        serialize(value, SerializationRequest::Css, allowance),
        Err(SerializationError::Limit {
            resource: SerializationResource::Digits,
            required: 1,
            allowed: 0
        })
    );
}

/// Services a finite formatter to completion, recording the output and
/// whether the planned length was known before the first emission.
fn format(
    value: Finite,
    rounding: RoundingMode,
) -> (
    Vec<u8>,
    usize,
    surgeist_color::format::FormatSummary,
    Option<usize>,
) {
    let mut owner = OperationOwner::new();
    let mut formatter = FiniteFormatter::new(&mut owner, value, rounding);
    let mut digits: Vec<DigitSlot<'_>> = Vec::new();
    let mut output = Vec::new();
    let mut planned_before_emit = None;
    for _ in 0..1_000_000 {
        let planned = formatter.planned_length();
        let action = match formatter.step() {
            FormatStep::Complete(summary) => {
                return (output, digits.len(), summary, planned_before_emit);
            }
            FormatStep::Pending(action) => action,
        };
        formatter = match action {
            FormatAction::Compute(compute) => compute.compute(),
            FormatAction::ReadDigit(read) => {
                let slot = &digits[read.index()];
                read.read(slot)
            }
            FormatAction::AppendDigit(write) => {
                assert_eq!(write.index(), digits.len());
                let (record, receipt) = write.record();
                digits.push(record);
                receipt.stored(&digits[digits.len() - 1])
            }
            FormatAction::ReplaceDigit(write) => {
                let index = write.index();
                let (record, receipt) = write.record();
                digits[index] = record;
                receipt.stored(&digits[index])
            }
            FormatAction::Emit(emit) => {
                planned_before_emit = planned_before_emit.or(planned);
                assert!(planned.is_some(), "the length is planned before any byte");
                output.push(emit.unit());
                emit.emitted()
            }
        };
    }
    panic!("finite formatting did not terminate")
}

#[test]
fn finite_formatter_exposes_real_plan_and_exact_integer_digits() {
    let (output, digits, summary, planned) =
        format(Finite::new(f64::MAX).unwrap(), RoundingMode::AwayFromZero);
    assert_eq!(planned, Some(309));
    assert_eq!(digits, 309);
    assert_eq!(output.len(), 309);
    assert_eq!(&output[..18], b"179769313486231570");
    assert_eq!(summary.emitted(), 309);
    assert!(!summary.is_zero());
}

#[test]
fn finite_formatter_emits_unsigned_zero_for_negative_zero() {
    let (output, digits, summary, planned) = format(
        Finite::new(-0.0).unwrap(),
        RoundingMode::TowardPositiveInfinity,
    );
    assert_eq!(output, b"0");
    assert_eq!(digits, 0);
    assert_eq!(planned, Some(1));
    assert_eq!(summary.emitted(), 1);
    assert!(summary.is_zero());
}

#[test]
fn fixed_requests_use_checked_numerical_operands_without_language_context() {
    let red = Color::from_srgba8([255, 0, 0, 255]);
    for (request, space, channels, alpha, bytes) in [
        (
            NumericalRequest::Convert {
                value: red,
                destination: Space::Predefined(Predefined::SrgbLinear),
            },
            Space::Predefined(Predefined::SrgbLinear),
            [Some(1.0), Some(0.0), Some(0.0)],
            Some(1.0),
            None,
        ),
        (
            NumericalRequest::Prepare {
                value: red,
                destination: Space::Hsl,
            },
            Space::Hsl,
            [Some(0.0), Some(100.0), Some(50.0)],
            Some(1.0),
            None,
        ),
        (
            NumericalRequest::WithAlpha {
                value: red,
                alpha: UnitInterval::new(0.5),
            },
            Space::Predefined(Predefined::Srgb),
            [Some(1.0), Some(0.0), Some(0.0)],
            Some(0.5),
            Some([255, 0, 0]),
        ),
        (
            NumericalRequest::WithColorFunction { value: red },
            Space::Predefined(Predefined::Srgb),
            [Some(1.0), Some(0.0), Some(0.0)],
            Some(1.0),
            Some([255, 0, 0]),
        ),
        (
            NumericalRequest::ToLegacySrgb {
                value: color(
                    Space::Hsl,
                    [Some(120.0), Some(100.0), Some(50.0)],
                    Some(1.0),
                ),
            },
            Space::Predefined(Predefined::Srgb),
            [Some(0.0), Some(1.0), Some(0.0)],
            Some(1.0),
            None,
        ),
        (
            NumericalRequest::BlackWhiteContrast {
                value: Color::opaque_black(),
            },
            Space::Predefined(Predefined::Srgb),
            [Some(1.0); 3],
            Some(1.0),
            Some([255; 3]),
        ),
    ] {
        let mut owner = OperationOwner::new();
        let result = Computation::new(&mut owner, request).compute();
        assert_eq!(result.space(), space);
        for (actual, expected) in result.channels().into_iter().zip(channels) {
            match (actual, expected) {
                (Some(a), Some(e)) => assert!((a - e).abs() < 1e-12),
                (None, None) => {}
                pair => panic!("missingness changed: {pair:?}"),
            }
        }
        assert_eq!(result.alpha(), alpha);
        assert_eq!(result.srgb_bytes(), bytes);
        if matches!(request, NumericalRequest::WithColorFunction { .. }) {
            assert_eq!(
                result.serialization_family(),
                SerializationFamily::ColorFunction
            );
        }
        if matches!(request, NumericalRequest::ToLegacySrgb { .. }) {
            assert_eq!(
                result.serialization_family(),
                SerializationFamily::LegacySrgb
            );
        }
        assert!(result.identical(request.execute()), "one numerical path");
    }
}

#[test]
fn overflowing_hsl_serializes_its_clipped_closest_finite_channels() {
    // Formerly the numerical-failure witness. Values 4: an unsupported range
    // converts to the closest supported value, so at hue 0 with S = L = 1e308
    // the red channel is -MAX and green/blue +MAX, clipped to 0 and 255.
    let value = color(Space::Hsl, [Some(0.0), Some(1e308), Some(1e308)], Some(1.0));
    let mut owner = OperationOwner::new();
    let Action::Compute(plan) = pending(Serialization::new(
        &mut owner,
        value,
        SerializationRequest::Css,
    )) else {
        panic!("planning is the first action")
    };
    assert_eq!(plan.kernel(), Kernel::SerializationPlan);
    let after_plan = plan.compute();
    assert!(matches!(pending(after_plan), Action::Emit(emit) if emit.unit() == b'r'));
    assert_eq!(
        serialize(value, SerializationRequest::Css, limits()).unwrap(),
        "rgb(0, 255, 255)"
    );
}
