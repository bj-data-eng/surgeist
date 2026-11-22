use surgeist_animation::easing::{CubicBezier, Easing, EasingError, StepPosition, Steps};
use surgeist_animation::progress::EasingInput;

#[test]
fn cubic_tangent_keeps_finite_output_when_its_slope_overflows() {
    let x = f64::MIN_POSITIVE / 4.0;
    let curve = CubicBezier::new(x, 1.0, 0.5, 0.5).unwrap();
    // The extension through (0, 0) and (x, 1) passes through (-x, -1).
    let input = EasingInput::new(-x).unwrap();
    assert_eq!(
        curve.evaluate_unrestricted(input).map(|p| p.value()),
        Ok(-1.0)
    );

    let x = f64::from_bits(1);
    let curve = CubicBezier::new(x, 2.0_f64.powi(-30), 0.5, 0.5).unwrap();
    // Both the slope and the distance ratio overflow on their own.
    let input = EasingInput::new(-2.0_f64.powi(-40)).unwrap();
    assert_eq!(
        curve.evaluate_unrestricted(input).map(|p| p.value()),
        Ok(-2.0_f64.powi(1004))
    );

    let before_one = 1.0_f64.next_down();
    let after_one = 1.0_f64.next_up();
    let curve = CubicBezier::new(0.5, 0.5, before_one, -f64::MAX / 4.0).unwrap();
    // The input is twice as far from 1 as the last control point.
    let input = EasingInput::new(after_one).unwrap();
    assert_eq!(
        curve.evaluate_unrestricted(input).map(|p| p.value()),
        Ok(f64::MAX / 2.0)
    );
}

#[test]
fn cubic_uses_secondary_or_constant_endpoint_tangents() {
    for (control, input, expected) in [
        ((0.0, 0.75, 0.5, 1.0), -0.5, -1.0),
        ((0.0, 0.75, 0.0, 1.0), -0.5, 0.0),
        ((0.5, 0.0, 1.0, 0.25), 1.5, 2.0),
        ((1.0, 0.0, 1.0, 0.25), 1.5, 1.0),
    ] {
        let (x1, y1, x2, y2) = control;
        let curve = CubicBezier::new(x1, y1, x2, y2).unwrap();
        assert_eq!(
            curve
                .evaluate_unrestricted(EasingInput::new(input).unwrap())
                .unwrap()
                .value(),
            expected
        );
    }
}

#[test]
fn cubic_tangent_does_not_round_a_subnormal_slope_before_multiplication() {
    let curve = CubicBezier::new(0.75, f64::from_bits(1), 0.9, 0.5).unwrap();
    let input = EasingInput::new(-2.0_f64.powi(1023)).unwrap();
    // Multiplying the smallest subnormal by the input gives exactly -2^-51.
    assert_eq!(
        curve.evaluate_unrestricted(input).map(|p| p.value()),
        Ok(-2.0_f64.powi(-51) / 0.75)
    );
}

#[test]
fn cubic_tangent_preserves_the_largest_finite_rounded_result() {
    let curve = CubicBezier::new(0.01, 0.03, 0.5, 0.5).unwrap();
    let input = EasingInput::new(-f64::from_bits(0x7fd5_5555_5555_5555)).unwrap();
    // The exact quotient using these binary64 operands rounds to -MAX.
    // Rounding the slope to 3 first moves the product across overflow.
    assert_eq!(
        curve.evaluate_unrestricted(input).map(|p| p.value()),
        Ok(-f64::MAX)
    );
}

#[test]
fn unrestricted_easing_can_feed_overshoot_to_the_next_function() {
    let curve = Easing::cubic_bezier(CubicBezier::new(0.25, 0.5, 0.75, 0.5).unwrap());
    for (input, expected) in [(-0.5, -1.0), (1.5, 2.0)] {
        let input = EasingInput::new(input).unwrap();
        let output = curve.evaluate_unrestricted(input).unwrap();
        assert_eq!(curve.evaluate_unrestricted(input), Ok(output));
        let next = EasingInput::new(output.value()).unwrap();
        assert_eq!(
            Easing::linear()
                .evaluate_unrestricted(next)
                .unwrap()
                .value(),
            expected
        );
    }
}

#[test]
fn steps_distinguishes_a_rounded_product_from_an_exact_discontinuity() {
    let steps = Steps::new(3, StepPosition::JumpEnd).unwrap();
    // Binary64 rounds 1/3 down. The product rounds up to 1 in binary64.
    let below = EasingInput::new(1.0 / 3.0).unwrap();
    assert_eq!(steps.evaluate_unrestricted(below).unwrap().value(), 0.0);

    // This binary64 input is above 2/3 even though its product rounds to 2.
    let above = EasingInput::new((2.0_f64 / 3.0).next_up()).unwrap();
    assert_eq!(
        steps
            .evaluate_unrestricted_with_before_flag(above, true)
            .unwrap()
            .value(),
        2.0 / 3.0
    );

    let steps = Steps::new(4, StepPosition::JumpEnd).unwrap();
    let exact = EasingInput::new(0.75).unwrap();
    assert_eq!(steps.evaluate_unrestricted(exact).unwrap().value(), 0.75);
    assert_eq!(
        steps
            .evaluate_unrestricted_with_before_flag(exact, true)
            .unwrap()
            .value(),
        0.5
    );
}

#[test]
fn steps_keeps_representable_outputs_when_the_step_number_overflows() {
    let input = EasingInput::new(f64::MAX).unwrap();
    for position in [StepPosition::JumpStart, StepPosition::JumpEnd] {
        let steps = Steps::new(2, position).unwrap();
        assert_eq!(
            steps.evaluate_unrestricted(input).map(|p| p.value()),
            Ok(f64::MAX)
        );
    }

    let steps = Steps::new(3, StepPosition::JumpBoth).unwrap();
    let magnitude = 2.0_f64.powi(1023);
    // Three steps and four jumps scale an integral input by exactly 3/4
    // at this magnitude, where the increment is below binary64 precision.
    for sign in [-1.0, 1.0] {
        let input = EasingInput::new(sign * magnitude).unwrap();
        assert_eq!(
            steps.evaluate_unrestricted(input).map(|p| p.value()),
            Ok(sign * 0.75 * magnitude)
        );
    }

    let steps = Steps::new(2, StepPosition::JumpNone).unwrap();
    assert!(matches!(
        steps.evaluate_unrestricted(input),
        Err(EasingError::NonFiniteEasedProgress { .. })
    ));
}

#[test]
fn steps_classifies_both_sides_of_the_finite_output_limit() {
    let finite = Steps::new(10, StepPosition::JumpNone).unwrap();
    let input = EasingInput::new(f64::from_bits(0x7fec_cccc_cccc_cccc)).unwrap();
    // The exact quotient x*10/9 rounds to MAX.
    assert_eq!(
        finite.evaluate_unrestricted(input).map(|p| p.value()),
        Ok(f64::MAX)
    );

    let overflowing = Steps::new(12, StepPosition::JumpNone).unwrap();
    let input = EasingInput::new(f64::from_bits(0x7fed_5555_5555_5555)).unwrap();
    // The exact quotient x*12/11 is above the binary64 overflow midpoint.
    assert!(matches!(
        overflowing.evaluate_unrestricted(input),
        Err(EasingError::NonFiniteEasedProgress { .. })
    ));
}

#[test]
fn large_step_numbers_preserve_the_before_flag_through_division() {
    let steps = Steps::new(3, StepPosition::JumpEnd).unwrap();
    let boundary = 2.0_f64.powi(52);
    let input = EasingInput::new(boundary).unwrap();
    assert_eq!(
        steps.evaluate_unrestricted(input).unwrap().value(),
        boundary
    );
    // The preceding step is 2^52 - 1/3, which rounds to 2^52 - 0.5.
    assert_eq!(
        steps
            .evaluate_unrestricted_with_before_flag(input, true)
            .unwrap()
            .value(),
        boundary - 0.5
    );
}

#[test]
fn a_single_step_can_break_a_rounding_tie_at_a_large_magnitude() {
    let steps = Steps::new(3, StepPosition::JumpBoth).unwrap();
    for progress in [
        f64::from_bits(0x4738_0000_0000_0006),
        -f64::from_bits(0x4738_0000_0000_0002),
    ] {
        let input = EasingInput::new(progress).unwrap();
        // These binary64 products by 3/4 are exact halfway cases whose
        // even result is the lower neighbor. The start jump adds 1/4.
        let before = progress * 0.75;
        assert_eq!(
            steps
                .evaluate_unrestricted_with_before_flag(input, true)
                .unwrap()
                .value(),
            before
        );
        assert_eq!(
            steps.evaluate_unrestricted(input).unwrap().value(),
            before.next_up()
        );
    }
}
