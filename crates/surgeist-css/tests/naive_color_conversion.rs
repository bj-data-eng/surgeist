#![forbid(unsafe_code)]

//! Numerical expectations are derived from Color 5 WD 2026-09-08 §6.1.
//! Section 14's informative JavaScript is deliberately not an oracle.
use surgeist_css::{
    CssNaiveColorConversionError as Error, naively_convert_cmyk_to_srgba as forward,
    naively_convert_srgba_to_cmyk as inverse,
};

#[test]
fn ordinary_fractional_sample_has_expected_rgb_and_loses_ink_decomposition() {
    let rgba = forward([0.25, 0.5, 0.75, 0.5, 0.375]).unwrap();
    assert_eq!(rgba, [0.375, 0.25, 0.125, 0.375]);
    let cmyka = inverse([0.375, 0.25, 0.125, 0.375]).unwrap();
    assert_eq!([cmyka[0], cmyka[3], cmyka[4]], [0.0, 0.625, 0.375]);
    // Thirds need a tolerance: division and subtraction can differ by an ulp
    // from the independently rounded real fractions. One epsilon bounds this.
    assert!((cmyka[1] - 1.0 / 3.0).abs() <= f64::EPSILON);
    assert!((cmyka[2] - 2.0 / 3.0).abs() <= f64::EPSILON);
    assert_ne!(cmyka, [0.25, 0.5, 0.75, 0.5, 0.375]);
}

#[test]
fn white_black_and_primary_samples_follow_explicit_channel_order() {
    for (cmyka, rgba) in [
        ([0.0, 0.0, 0.0, 0.0, 0.75], [1.0, 1.0, 1.0, 0.75]),
        ([0.0, 0.0, 0.0, 1.0, 0.75], [0.0, 0.0, 0.0, 0.75]),
        ([0.0, 1.0, 1.0, 0.0, 0.75], [1.0, 0.0, 0.0, 0.75]),
        ([1.0, 0.0, 1.0, 0.0, 0.75], [0.0, 1.0, 0.0, 0.75]),
        ([1.0, 1.0, 0.0, 0.0, 0.75], [0.0, 0.0, 1.0, 0.75]),
    ] {
        assert_eq!(forward(cmyka).unwrap(), rgba);
        assert_eq!(inverse(rgba).unwrap(), cmyka);
    }
}

#[test]
fn unit_black_saturates_every_finite_ink_without_overflow() {
    for ink in [-f64::MAX, -2.0, -0.0, 0.0, 0.5, 1.0, 2.0, f64::MAX] {
        assert_eq!(
            forward([ink, ink, ink, 1.0, 0.25]).unwrap(),
            [0.0, 0.0, 0.0, 0.25]
        );
    }
}

#[test]
fn true_zero_maximum_uses_zero_inks_even_with_negative_siblings() {
    for rgba in [
        [0.0, -0.25, -2.0, 0.5],
        [-3.0, -0.0, -1.0, 0.5],
        [-1.0, -2.0, 0.0, 0.5],
    ] {
        assert_eq!(inverse(rgba).unwrap(), [0.0, 0.0, 0.0, 1.0, 0.5]);
    }
}

#[test]
fn near_black_power_of_two_keeps_ratios_without_an_epsilon_cutoff() {
    let maximum = 2.0_f64.powi(-40);
    assert_eq!(
        inverse([maximum, maximum / 2.0, 0.0, 0.25]).unwrap(),
        [0.0, 0.5, 1.0, 1.0 - maximum, 0.25]
    );
}

#[test]
fn unequal_subnormals_keep_ratios_when_black_rounds_to_one() {
    // Four, two, and one units of the smallest positive subnormal give exact
    // ratios 1, 1/2, 1/4. Subtracting their maximum from one rounds to one.
    let cmyka = inverse([
        f64::from_bits(4),
        f64::from_bits(2),
        f64::from_bits(1),
        0.75,
    ])
    .unwrap();
    assert_eq!(cmyka, [0.0, 0.5, 0.75, 1.0, 0.75]);
    assert_eq!(forward(cmyka).unwrap(), [0.0, 0.0, 0.0, 0.75]);
}

#[test]
fn factored_forward_preserves_small_positive_channel_near_unit_ink() {
    let just_below_one = f64::from_bits(1.0_f64.to_bits() - 1);
    assert_eq!(
        forward([just_below_one, 0.5, 0.0, 0.5, 0.75]).unwrap(),
        [2.0_f64.powi(-54), 0.25, 0.5, 0.75]
    );
}

#[test]
fn extended_coordinates_are_not_range_clamped_in_either_direction() {
    assert_eq!(
        inverse([2.0, 1.0, 0.0, 1.5]).unwrap(),
        [0.0, 0.5, 1.0, -1.0, 1.5]
    );
    assert_eq!(
        forward([-1.0, 0.0, 2.0, 0.0, -2.0]).unwrap(),
        [2.0, 1.0, 0.0, -2.0]
    );
    assert_eq!(
        forward([2.0, 3.0, 4.0, 2.0, 0.5]).unwrap(),
        [1.0, 2.0, 3.0, 0.5]
    );
    assert_eq!(
        inverse([-1.0, -2.0, -3.0, 0.5]).unwrap(),
        [0.0, -1.0, -2.0, 2.0, 0.5]
    );
}

#[test]
fn opposite_factor_signs_saturate_before_an_extreme_product_overflows() {
    assert_eq!(
        forward([-f64::MAX, -2.0, 1.0, f64::MAX, 0.375]).unwrap(),
        [0.0, 0.0, 0.0, 0.375]
    );
    assert_eq!(
        forward([f64::MAX, 2.0, 1.0, -f64::MAX, 0.375]).unwrap(),
        [0.0, 0.0, 0.0, 0.375]
    );
}

#[test]
fn positive_product_overflow_is_an_error_for_both_factor_signs() {
    for sample in [
        [-f64::MAX, 0.0, 0.0, -f64::MAX, 1.0],
        [f64::MAX, 2.0, 2.0, f64::MAX, 1.0],
        [1.0, -f64::MAX, 0.0, -f64::MAX, 1.0],
        [1.0, 1.0, -f64::MAX, -f64::MAX, 1.0],
    ] {
        assert_eq!(forward(sample), Err(Error::UnrepresentableResult));
    }
}

#[test]
fn inverse_ratio_overflow_is_an_error_with_positive_and_negative_maxima() {
    let tiny = f64::from_bits(1);
    for sample in [
        [tiny, -f64::MAX, 0.0, 0.5],
        [-tiny, -f64::MAX, -tiny, 0.5],
        [-f64::MAX, tiny, 0.0, 0.5],
        [0.0, tiny, -f64::MAX, 0.5],
    ] {
        assert_eq!(inverse(sample), Err(Error::UnrepresentableResult));
    }
}

#[test]
fn every_nonfinite_input_position_is_rejected_in_documented_order() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for component in 0..5 {
            let mut input = [0.0, 0.0, 0.0, 0.0, 1.0];
            input[component] = invalid;
            assert_eq!(forward(input), Err(Error::NonFiniteInput { component }));
        }
        for component in 0..4 {
            let mut input = [0.25, 0.5, 0.75, 1.0];
            input[component] = invalid;
            assert_eq!(inverse(input), Err(Error::NonFiniteInput { component }));
        }
    }
}

#[test]
fn first_nonfinite_index_has_precedence_over_other_invalid_inputs_and_overflow() {
    assert_eq!(
        forward([f64::NAN, f64::INFINITY, 0.0, f64::NEG_INFINITY, f64::NAN]),
        Err(Error::NonFiniteInput { component: 0 })
    );
    assert_eq!(
        inverse([0.0, f64::NEG_INFINITY, f64::INFINITY, f64::NAN]),
        Err(Error::NonFiniteInput { component: 1 })
    );
    assert_eq!(
        forward([-f64::MAX, 0.0, 0.0, -f64::MAX, f64::NAN]),
        Err(Error::NonFiniteInput { component: 4 })
    );
    assert_eq!(
        inverse([f64::from_bits(1), -f64::MAX, 0.0, f64::INFINITY]),
        Err(Error::NonFiniteInput { component: 3 })
    );
}

#[test]
fn alpha_bits_are_copied_including_negative_zero_and_extended_values() {
    for alpha in [
        -0.0_f64,
        0.0,
        0.375,
        -2.0,
        1.5,
        f64::MAX,
        -f64::MAX,
        f64::from_bits(1),
    ] {
        assert_eq!(
            forward([0.25, 0.5, 0.75, 0.5, alpha]).unwrap()[3].to_bits(),
            alpha.to_bits()
        );
        assert_eq!(
            inverse([0.375, 0.25, 0.125, alpha]).unwrap()[4].to_bits(),
            alpha.to_bits()
        );
    }
}

#[test]
fn ordinary_rgb_round_trips_with_a_bound_for_composed_f64_rounding() {
    // All coordinates here lie in [0,1]. Eight epsilons conservatively cover
    // max/subtraction/division and the subsequent subtract/multiply operations.
    for rgb in [
        [0.2, 0.4, 0.8],
        [0.1, 0.9, 0.5],
        [0.01, 0.02, 0.03],
        [0.9, 0.99, 1.0],
    ] {
        let input = [rgb[0], rgb[1], rgb[2], 0.625];
        let result = forward(inverse(input).unwrap()).unwrap();
        for component in 0..3 {
            assert!((result[component] - input[component]).abs() <= 8.0 * f64::EPSILON);
        }
        assert_eq!(result[3].to_bits(), input[3].to_bits());
    }
}

#[test]
fn error_has_standard_traits_and_does_not_invent_source_coordinates() {
    fn standard_error<T: std::error::Error + Clone + Copy + Eq>(error: T) -> String {
        assert!(error.source().is_none());
        error.to_string()
    }
    assert!(!standard_error(Error::NonFiniteInput { component: 3 }).is_empty());
    assert!(!standard_error(Error::UnrepresentableResult).is_empty());
}
