//! Native numerical use and actual caller-owned continuation storage.
use surgeist_color::format::{FiniteFormatter, FormatError, RoundingMode};
use surgeist_color::{
    ActionDescription as Action, Color, ColorSpace as Space, Computation, DigitSlot, Finite,
    InterpolationMethod, NumericalRequest, OperationOwner, PredefinedColorSpace as Predefined,
    ProtocolError, Serialization, SerializationError, SerializationFamily, SerializationLimits,
    SerializationRequest, SerializationResource, Transformation, UnitInterval, serialize,
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
    let original = color(Space::Lab, [Some(-0.0), None, Some(2.0)], None);
    let request = NumericalRequest::Transform {
        value: original,
        transformation: Transformation::ForceOpaqueAlpha,
    };
    let mut operation = Some(Computation::new(&mut owner, request).unwrap());
    let pending = operation.as_ref().unwrap().action();
    assert_eq!(
        Computation::take(&mut operation),
        Err(ProtocolError::NotReady)
    );
    {
        let _withheld = operation.as_mut().unwrap().next_action();
    }
    assert_eq!(operation.as_ref().unwrap().action(), pending);
    operation.as_mut().unwrap().next_action().compute().unwrap();
    assert_eq!(
        operation.as_mut().unwrap().next_action().compute(),
        Err(ProtocolError::WrongAction)
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
        Err(ProtocolError::EmptySlot)
    );
    let mut second_owner = OperationOwner::new();
    let mut operation = Some(Computation::new(&mut second_owner, request).unwrap());
    operation.as_mut().unwrap().abandon();
    assert_eq!(
        Computation::take(&mut operation),
        Err(ProtocolError::Terminated)
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
fn proven_operand_requests_compute_once_and_keep_their_bits() {
    // Construction and interpolation over proof-typed operands are total: the
    // computation becomes ready, and recovery returns the exact request.
    let mut owner = OperationOwner::new();
    let request = NumericalRequest::Construct {
        space: Space::Lab,
        channels: [Finite::new(-0.0), Finite::new(f64::MAX), None],
        alpha: UnitInterval::new(-0.0),
    };
    let mut operation = Some(Computation::new(&mut owner, request).unwrap());
    operation.as_mut().unwrap().next_action().compute().unwrap();
    assert_eq!(operation.as_ref().unwrap().action(), Action::Ready);
    let value = Computation::take(&mut operation).unwrap();
    assert_eq!(value.channels()[0].unwrap().to_bits(), (-0.0_f64).to_bits());
    assert_eq!(value.channels()[1], Some(f64::MAX));
    assert_eq!(value.alpha().unwrap().to_bits(), (-0.0_f64).to_bits());
    let mut operation = Some(Computation::new(&mut owner, request).unwrap());
    let NumericalRequest::Construct { channels, .. } =
        Computation::recover(&mut operation).unwrap()
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
    let mut operation = Some(Computation::new(&mut owner, request).unwrap());
    operation.as_mut().unwrap().next_action().compute().unwrap();
    let mixed = Computation::take(&mut operation).unwrap();
    assert_eq!(mixed.channels(), [Some(f64::MAX / 2.0); 3]);
    assert!(mixed.identical(request.execute()));
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
        Err(ProtocolError::NotReady)
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
        Err(ProtocolError::Terminated)
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

#[test]
fn finite_formatter_exposes_real_plan_and_exact_integer_digits() {
    let mut owner = OperationOwner::new();
    let mut operation = Some(
        FiniteFormatter::new(
            &mut owner,
            Finite::new(f64::MAX).unwrap(),
            RoundingMode::AwayFromZero,
        )
        .unwrap(),
    );
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
fn finite_formatter_recovers_negative_zero_bits() {
    let mut owner = OperationOwner::new();
    let mut operation = Some(
        FiniteFormatter::new(
            &mut owner,
            Finite::new(-0.0).unwrap(),
            RoundingMode::TowardPositiveInfinity,
        )
        .unwrap(),
    );
    operation.as_mut().unwrap().abandon();
    assert!(matches!(
        FiniteFormatter::take(&mut operation),
        Err(FormatError::Protocol(ProtocolError::Terminated))
    ));
    let recovery = FiniteFormatter::recover(&mut operation).unwrap();
    assert_eq!(recovery.value().get().to_bits(), (-0.0_f64).to_bits());
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
    let mut operation =
        Some(Serialization::new(&mut owner, value, SerializationRequest::Css).unwrap());
    assert_eq!(operation.as_mut().unwrap().next_action().compute(), Ok(()));
    operation.as_mut().unwrap().abandon();
    let recovery = Serialization::recover(&mut operation).unwrap();
    assert!(recovery.value().identical(value));
    assert_eq!(recovery.length(), 0);
    assert_eq!(
        serialize(value, SerializationRequest::Css, limits()).unwrap(),
        "rgb(0, 255, 255)"
    );
}
