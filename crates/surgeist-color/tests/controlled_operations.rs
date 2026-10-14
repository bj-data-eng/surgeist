//! Native numerical use and actual caller-owned continuation storage.
use surgeist_color::format::{FiniteFormatter, FormatError, RoundingMode};
use surgeist_color::{
    ActionDescription as Action, Color, ColorError, ColorSpace as Space, Computation,
    ComputationError, DigitSlot, InterpolationError, InterpolationMethod, NumericalRequest,
    OperationOwner, PredefinedColorSpace as Predefined, ProtocolError, Serialization,
    SerializationError, SerializationFamily, SerializationLimits, SerializationRequest,
    SerializationResource, Transformation, serialize,
};

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
fn finish<'owner>(
    operation: &mut Serialization<'owner>,
    digits: &mut Vec<DigitSlot<'owner>>,
    output: &mut Vec<u8>,
) {
    for _ in 0..1_000_000 {
        let pending = operation.action();
        if pending == Action::Ready {
            return;
        }
        {
            let refused = operation.next_action();
            assert_eq!(refused.description(), pending);
        }
        assert_eq!(
            operation.action(),
            pending,
            "dropping admission does no work"
        );
        let mut action = operation.next_action();
        match pending {
            Action::Compute(_) => action.compute().unwrap(),
            Action::ReadDigit { index } => action.read_digit(index, &digits[index]).unwrap(),
            Action::AppendDigit { index } => {
                assert_eq!(index, digits.len());
                assert_eq!(
                    action.layout().write_bytes,
                    std::mem::size_of::<DigitSlot<'_>>()
                );
                let _same_pending_record = action.record().unwrap();
                digits.push(action.record().unwrap());
                action.written(index, &digits[index]).unwrap();
            }
            Action::ReplaceDigit { index } => {
                digits[index] = action.record().unwrap();
                action.written(index, &digits[index]).unwrap();
            }
            Action::EmitByte { unit } => {
                assert!(unit.is_ascii());
                assert_eq!(action.layout().output_bytes, 1);
                output.push(unit);
                action.emitted(unit).unwrap();
            }
            other => panic!("unexpected live action {other:?}"),
        }
    }
    panic!("finite serialization did not terminate")
}

#[test]
fn computation_take_refusal_drop_and_recovery_preserve_exact_request() {
    let mut owner = OperationOwner::new();
    let original = Color::try_new(Space::Lab, [Some(-0.0), None, Some(2.0)], None).unwrap();
    let request = NumericalRequest::Transform {
        value: original,
        transformation: Transformation::ForceOpaqueAlpha,
    };
    let mut operation = Some(Computation::new(&mut owner, request).unwrap());
    let pending = operation.as_ref().unwrap().action();
    assert_eq!(
        Computation::take(&mut operation),
        Err(ComputationError::Protocol(ProtocolError::NotReady))
    );
    {
        let _withheld = operation.as_mut().unwrap().next_action();
    }
    assert_eq!(operation.as_ref().unwrap().action(), pending);
    operation.as_mut().unwrap().next_action().compute().unwrap();
    assert_eq!(
        operation.as_mut().unwrap().next_action().compute(),
        Err(ComputationError::Protocol(ProtocolError::WrongAction))
    );
    let result = Computation::take(&mut operation).unwrap();
    assert!(operation.is_none());
    assert_eq!(result.alpha(), Some(1.0));
    assert_eq!(
        result.channels()[0].unwrap().to_bits(),
        (-0.0_f64).to_bits()
    );
    assert_eq!(
        Computation::take(&mut operation),
        Err(ComputationError::Protocol(ProtocolError::EmptySlot))
    );
    let mut second_owner = OperationOwner::new();
    let mut operation = Some(Computation::new(&mut second_owner, request).unwrap());
    operation.as_mut().unwrap().abandon();
    assert_eq!(
        Computation::take(&mut operation),
        Err(ComputationError::Protocol(ProtocolError::Terminated))
    );
    let NumericalRequest::Transform {
        value,
        transformation,
    } = Computation::recover(&mut operation).unwrap()
    else {
        panic!("original transformation")
    };
    assert!(value.identical(original));
    assert_eq!(transformation, Transformation::ForceOpaqueAlpha);
    assert!(operation.is_none());
}

#[test]
fn failed_computation_is_terminal_and_retains_first_error_and_original_bits() {
    let mut owner = OperationOwner::new();
    let nan = f64::from_bits(0x7ff8_0000_0000_0123);
    let request = NumericalRequest::Construct {
        space: Space::Lab,
        channels: [Some(nan), Some(f64::INFINITY), None],
        alpha: Some(-1.0),
    };
    let mut operation = Some(Computation::new(&mut owner, request).unwrap());
    let error = ComputationError::Numerical(ColorError::NonFinite { component: 0 });
    assert_eq!(
        operation.as_mut().unwrap().next_action().compute(),
        Err(error)
    );
    assert_eq!(operation.as_ref().unwrap().action(), Action::Failed);
    assert_eq!(Computation::take(&mut operation), Err(error));
    assert!(operation.is_some());
    assert_eq!(
        operation.as_mut().unwrap().next_action().compute(),
        Err(ComputationError::Protocol(ProtocolError::WrongAction))
    );
    operation.as_mut().unwrap().abandon();
    assert_eq!(operation.as_ref().unwrap().failure(), Some(&error));
    let NumericalRequest::Construct { channels, .. } =
        Computation::recover(&mut operation).unwrap()
    else {
        panic!("original construction")
    };
    assert_eq!(channels[0].unwrap().to_bits(), nan.to_bits());
}

#[test]
fn admitted_interpolation_keeps_its_distinct_validation_error() {
    let mut owner = OperationOwner::new();
    let value = Color::from_srgba8([0, 0, 0, 255]);
    let method = InterpolationMethod::try_new(Space::Lab, None).unwrap();
    let mut operation = Some(
        Computation::new(
            &mut owner,
            NumericalRequest::Interpolate {
                first: value,
                second: value,
                method,
                progress: f64::NAN,
            },
        )
        .unwrap(),
    );
    let error = ComputationError::Interpolation(InterpolationError::InvalidProgress);
    assert_eq!(
        operation.as_mut().unwrap().next_action().compute(),
        Err(error)
    );
    assert_eq!(Computation::take(&mut operation), Err(error));
    assert!(operation.is_some());
}

#[test]
fn native_serialization_is_complete_before_take_and_owner_reuse() {
    eprintln!(
        "native compiled layouts: owner={} digit={} computation={} computation_action={} serialization={} stream_action={} finite_formatter={} finite_action={}",
        OperationOwner::layout_bytes(),
        std::mem::size_of::<DigitSlot<'_>>(),
        Computation::register_bytes(),
        Computation::action_bytes(),
        Serialization::register_bytes(),
        Serialization::action_bytes(),
        FiniteFormatter::register_bytes(),
        FiniteFormatter::action_bytes()
    );
    let value = Color::from_srgba8([255, 0, 255, 255]);
    let mut owner = OperationOwner::new();
    let mut operation =
        Some(Serialization::new(&mut owner, value, SerializationRequest::Css).unwrap());
    assert!(matches!(
        Serialization::take(&mut operation),
        Err(SerializationError::Protocol(ProtocolError::NotReady))
    ));
    let mut digits = Vec::new();
    let mut output = Vec::new();
    finish(operation.as_mut().unwrap(), &mut digits, &mut output);
    let receipt = Serialization::take(&mut operation).unwrap();
    assert_eq!(output, b"rgb(255, 0, 255)");
    assert_eq!(receipt.length(), output.len());
    assert!(receipt.value().identical(value));
    assert!(operation.is_none());
    drop(digits);
    let mut second = Some(
        Computation::new(
            &mut owner,
            NumericalRequest::WithAlpha { value, alpha: None },
        )
        .unwrap(),
    );
    second.as_mut().unwrap().next_action().compute().unwrap();
    assert_eq!(Computation::take(&mut second).unwrap().alpha(), None);
    assert!(
        receipt.value().identical(value),
        "receipt has no numerical owner borrow"
    );
}

fn reach_append(operation: &mut Serialization<'_>) -> usize {
    loop {
        let action = operation.next_action();
        match action.description() {
            Action::Compute(_) => action.compute().unwrap(),
            Action::EmitByte { unit } => action.emitted(unit).unwrap(),
            Action::AppendDigit { index } => return index,
            other => panic!("unexpected prefix {other:?}"),
        }
    }
}

#[test]
fn foreign_digit_and_wrong_index_refusals_leave_real_write_pending() {
    let value = Color::from_srgba8([255, 0, 0, 255]);
    let mut owner = OperationOwner::new();
    let mut foreign_owner = OperationOwner::new();
    let mut operation =
        Some(Serialization::new(&mut owner, value, SerializationRequest::Css).unwrap());
    let mut other =
        Serialization::new(&mut foreign_owner, value, SerializationRequest::Css).unwrap();
    let index = reach_append(operation.as_mut().unwrap());
    assert_eq!(reach_append(&mut other), index);
    let foreign = other.next_action().record().unwrap();
    let pending = operation.as_ref().unwrap().action();
    assert_eq!(
        operation
            .as_mut()
            .unwrap()
            .next_action()
            .written(index, &foreign),
        Err(ProtocolError::ForeignSlot)
    );
    assert_eq!(operation.as_ref().unwrap().action(), pending);
    let local = operation.as_mut().unwrap().next_action().record().unwrap();
    assert_eq!(
        operation
            .as_mut()
            .unwrap()
            .next_action()
            .written(index + 1, &local),
        Err(ProtocolError::WrongIndex)
    );
    assert_eq!(operation.as_ref().unwrap().action(), pending);
    operation
        .as_mut()
        .unwrap()
        .next_action()
        .written(index, &local)
        .unwrap();
    operation.as_mut().unwrap().abandon();
    assert!(matches!(
        Serialization::take(&mut operation),
        Err(SerializationError::Protocol(ProtocolError::Terminated))
    ));
    let recovery = Serialization::recover(&mut operation).unwrap();
    assert!(recovery.value().identical(value));
    assert_eq!(recovery.request(), SerializationRequest::Css);
    assert_eq!(recovery.length(), 4, "only acknowledged rgb( prefix");
}

#[test]
fn wrong_output_acknowledgement_and_prefix_recovery_do_not_publish_success() {
    let mut owner = OperationOwner::new();
    let value = Color::from_srgba8([255, 0, 0, 255]);
    let mut operation =
        Some(Serialization::new(&mut owner, value, SerializationRequest::HtmlCompatible).unwrap());
    operation.as_mut().unwrap().next_action().compute().unwrap();
    while matches!(operation.as_ref().unwrap().action(), Action::Compute(_)) {
        operation.as_mut().unwrap().next_action().compute().unwrap();
    }
    let pending = operation.as_ref().unwrap().action();
    assert_eq!(pending, Action::EmitByte { unit: b'#' });
    assert_eq!(
        operation.as_mut().unwrap().next_action().emitted(b'x'),
        Err(ProtocolError::WrongReply)
    );
    assert_eq!(operation.as_ref().unwrap().action(), pending);
    operation
        .as_mut()
        .unwrap()
        .next_action()
        .emitted(b'#')
        .unwrap();
    let recovery = Serialization::recover(&mut operation).unwrap();
    assert_eq!(recovery.length(), 1);
    assert!(recovery.value().identical(value));
    assert!(operation.is_none());
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
            value
                .with_serialization_family(SerializationFamily::ColorFunction)
                .unwrap(),
            SerializationRequest::HtmlCompatible
        ),
        "#ff00ff"
    );
    let floating = Color::try_new(value.space(), value.channels(), value.alpha()).unwrap();
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
    let floating = Color::try_new(value.space(), value.channels(), Some(237.0 / 255.0)).unwrap();
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
            spelling(
                Color::try_new(space, channels, alpha).unwrap(),
                SerializationRequest::Css
            ),
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

#[test]
fn finite_formatter_exposes_real_plan_and_exact_integer_digits() {
    let mut owner = OperationOwner::new();
    let mut operation =
        Some(FiniteFormatter::new(&mut owner, f64::MAX, RoundingMode::AwayFromZero).unwrap());
    assert!(matches!(
        FiniteFormatter::take(&mut operation),
        Err(FormatError::Protocol(ProtocolError::NotReady))
    ));
    let mut digits = Vec::new();
    let mut output = Vec::new();
    let mut planned_before_emit = false;
    for _ in 0..1_000_000 {
        let live = operation.as_mut().unwrap();
        if let Some(length) = live.planned_length() {
            assert_eq!(length, 309);
            planned_before_emit = true;
        }
        let mut action = live.next_action();
        match action.description() {
            Action::Compute(_) => action.compute().unwrap(),
            Action::ReadDigit { index } => action.read_digit(index, &digits[index]).unwrap(),
            Action::AppendDigit { index } => {
                assert_eq!(index, digits.len());
                digits.push(action.record().unwrap());
                action.written(index, &digits[index]).unwrap();
            }
            Action::ReplaceDigit { index } => {
                digits[index] = action.record().unwrap();
                action.written(index, &digits[index]).unwrap();
            }
            Action::EmitByte { unit } => {
                assert!(planned_before_emit);
                output.push(unit);
                action.emitted(unit).unwrap();
            }
            Action::Ready => break,
            other => panic!("unexpected format state {other:?}"),
        }
    }
    assert_eq!(operation.as_ref().unwrap().action(), Action::Ready);
    assert_eq!(digits.len(), 309);
    assert_eq!(output.len(), 309);
    assert_eq!(&output[..18], b"179769313486231570");
    let summary = FiniteFormatter::take(&mut operation).unwrap();
    assert_eq!(summary.emitted(), 309);
    assert!(!summary.is_zero());
}

#[test]
fn finite_formatter_rejects_nonfinite_and_recovers_negative_zero_bits() {
    let mut owner = OperationOwner::new();
    assert!(matches!(
        FiniteFormatter::new(&mut owner, f64::NAN, RoundingMode::AwayFromZero),
        Err(FormatError::NonFinite)
    ));
    let mut operation =
        Some(FiniteFormatter::new(&mut owner, -0.0, RoundingMode::TowardPositiveInfinity).unwrap());
    operation.as_mut().unwrap().abandon();
    assert!(matches!(
        FiniteFormatter::take(&mut operation),
        Err(FormatError::Protocol(ProtocolError::Terminated))
    ));
    let recovery = FiniteFormatter::recover(&mut operation).unwrap();
    assert_eq!(recovery.value().to_bits(), (-0.0_f64).to_bits());
    assert_eq!(recovery.rounding(), RoundingMode::TowardPositiveInfinity);
    assert_eq!(recovery.length(), 0);
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
                alpha: Some(0.5),
            },
            Space::Predefined(Predefined::Srgb),
            [Some(1.0), Some(0.0), Some(0.0)],
            Some(0.5),
            Some([255, 0, 0]),
        ),
        (
            NumericalRequest::WithFamily {
                value: red,
                family: SerializationFamily::ColorFunction,
            },
            Space::Predefined(Predefined::Srgb),
            [Some(1.0), Some(0.0), Some(0.0)],
            Some(1.0),
            Some([255, 0, 0]),
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
        let mut operation = Some(Computation::new(&mut owner, request).unwrap());
        operation.as_mut().unwrap().next_action().compute().unwrap();
        let result = Computation::take(&mut operation).unwrap();
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
        if matches!(request, NumericalRequest::WithFamily { .. }) {
            assert_eq!(
                result.serialization_family(),
                SerializationFamily::ColorFunction
            );
        }
    }
}

#[test]
fn failed_serialization_keeps_original_and_first_numerical_error() {
    let value =
        Color::try_new(Space::Hsl, [Some(0.0), Some(1e308), Some(1e308)], Some(1.0)).unwrap();
    let mut owner = OperationOwner::new();
    let mut operation =
        Some(Serialization::new(&mut owner, value, SerializationRequest::Css).unwrap());
    let error = SerializationError::Numerical(ColorError::Unrepresentable);
    assert_eq!(
        operation.as_mut().unwrap().next_action().compute(),
        Err(error)
    );
    assert_eq!(operation.as_ref().unwrap().action(), Action::Failed);
    assert!(matches!(Serialization::take(&mut operation), Err(e) if e == error));
    assert!(operation.is_some());
    assert_eq!(
        operation.as_mut().unwrap().next_action().compute(),
        Err(SerializationError::Protocol(ProtocolError::WrongAction))
    );
    operation.as_mut().unwrap().abandon();
    assert_eq!(operation.as_ref().unwrap().failure(), Some(&error));
    let recovery = Serialization::recover(&mut operation).unwrap();
    assert!(recovery.value().identical(value));
    assert_eq!(recovery.length(), 0);
}
