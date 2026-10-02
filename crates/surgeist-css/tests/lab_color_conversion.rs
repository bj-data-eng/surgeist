#![forbid(unsafe_code)]

//! Expectations follow Color 4 CRD 2026-09-08 §§9.3–9.6, not sample-code outputs.
//! Cardinal/diagonal angles and scaled 3-4-5 triangles are independent geometry
//! oracles. Decimal 53.13010235415598° is the independently rounded atan(4/3).

use surgeist_css::{
    CssPolarColorConversionError as Error, CssPolarColorCoordinates as Polar, convert_lab_to_lch,
    convert_lch_to_lab, convert_oklab_to_oklch, convert_oklch_to_oklab,
};

type Forward = fn([f64; 3]) -> Result<Polar, Error>;
type Inverse = fn(&Polar) -> [f64; 3];

const CONVERSIONS: [(Forward, Inverse, f64); 2] = [
    (convert_lab_to_lch, convert_lch_to_lab, 0.0015),
    (convert_oklab_to_oklch, convert_oklch_to_oklab, 0.000004),
];

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= tolerance,
        "actual {actual:?}, expected {expected:?}, tolerance {tolerance:?}"
    );
}

#[test]
fn cardinals_and_diagonals_follow_axes_and_normalized_quadrants() {
    for (forward, inverse, _) in CONVERSIONS {
        for ([a, b], chroma, hue) in [
            ([5.0, 0.0], 5.0, 0.0),
            ([0.0, 5.0], 5.0, 90.0),
            ([-5.0, 0.0], 5.0, 180.0),
            ([0.0, -5.0], 5.0, 270.0),
            ([1.0, 1.0], std::f64::consts::SQRT_2, 45.0),
            ([-1.0, 1.0], std::f64::consts::SQRT_2, 135.0),
            ([-1.0, -1.0], std::f64::consts::SQRT_2, 225.0),
            ([1.0, -1.0], std::f64::consts::SQRT_2, 315.0),
        ] {
            let polar = forward([50.0, a, b]).unwrap();
            close(polar.chroma(), chroma, 2.0 * f64::EPSILON);
            close(polar.hue_degrees().unwrap(), hue, 1e-12);
            let reconstructed = inverse(&Polar::try_new(50.0, chroma, Some(hue)).unwrap());
            close(reconstructed[1], a, 1e-14);
            close(reconstructed[2], b, 1e-14);
        }
    }
}

#[test]
fn three_four_five_geometry_uses_each_spaces_concrete_number_units() {
    for (forward, inverse, scale, lightness) in [
        (
            convert_lab_to_lch as Forward,
            convert_lch_to_lab as Inverse,
            1.0,
            50.0,
        ),
        (
            convert_oklab_to_oklch as Forward,
            convert_oklch_to_oklab as Inverse,
            0.01,
            0.5,
        ),
    ] {
        for (a_sign, b_sign, hue) in [
            (1.0, 1.0, 53.13010235415598),
            (-1.0, 1.0, 126.86989764584402),
            (-1.0, -1.0, 233.13010235415598),
            (1.0, -1.0, 306.86989764584402),
        ] {
            let a = a_sign * 3.0 * scale;
            let b = b_sign * 4.0 * scale;
            let polar = forward([lightness, a, b]).unwrap();
            assert_eq!(polar.lightness(), lightness);
            close(polar.chroma(), 5.0 * scale, 8.0 * f64::EPSILON * scale);
            close(polar.hue_degrees().unwrap(), hue, 1e-12);
            let actual = inverse(&Polar::try_new(lightness, 5.0 * scale, Some(hue)).unwrap());
            close(actual[1], a, 1e-14 * scale);
            close(actual[2], b, 1e-14 * scale);
        }
    }
}

#[test]
fn exact_epsilon_and_adjacent_values_select_missing_hue_strictly() {
    for (forward, inverse, epsilon) in CONVERSIONS {
        let below = f64::from_bits(epsilon.to_bits() - 1);
        let above = f64::from_bits(epsilon.to_bits() + 1);
        for chroma in [0.0, below, epsilon] {
            let polar = forward([12.0, 0.0, -chroma]).unwrap();
            assert_eq!(polar.chroma(), chroma);
            assert_eq!(polar.hue_degrees(), None);
            assert_eq!(inverse(&polar), [12.0, 0.0, 0.0]);
        }
        let polar = forward([12.0, 0.0, -above]).unwrap();
        assert_eq!(polar.chroma(), above);
        assert_eq!(polar.hue_degrees(), Some(270.0));
    }
}

#[test]
fn constructor_keeps_present_subthreshold_hue_and_missing_hue_at_any_chroma() {
    for (_, inverse, epsilon) in CONVERSIONS {
        for chroma in [0.0, epsilon / 2.0, epsilon] {
            let polar = Polar::try_new(0.5, chroma, Some(0.0)).unwrap();
            assert_eq!(polar.hue_degrees(), Some(0.0));
            assert_eq!(inverse(&polar), [0.5, chroma, 0.0]);
        }
        for chroma in [0.0, epsilon, 5.0, f64::MAX] {
            let polar = Polar::try_new(-0.0, chroma, None).unwrap();
            assert_eq!(polar.chroma(), chroma);
            assert_eq!(polar.hue_degrees(), None);
            let rectangular = inverse(&polar);
            assert_eq!(rectangular[0].to_bits(), (-0.0_f64).to_bits());
            assert_eq!(rectangular[1].to_bits(), 0.0_f64.to_bits());
            assert_eq!(rectangular[2].to_bits(), 0.0_f64.to_bits());
        }
    }
}

#[test]
fn finite_extended_lightness_is_copied_bit_for_bit_in_every_path() {
    for lightness in [
        -f64::MAX,
        -2.0,
        -0.0,
        0.0,
        f64::from_bits(1),
        0.5,
        200.0,
        f64::MAX,
    ] {
        for (forward, inverse, _) in CONVERSIONS {
            let polar = forward([lightness, -3.0, 4.0]).unwrap();
            assert_eq!(polar.lightness().to_bits(), lightness.to_bits());
            assert_eq!(inverse(&polar)[0].to_bits(), lightness.to_bits());
            let direct = Polar::try_new(lightness, 5.0, Some(45.0)).unwrap();
            assert_eq!(direct.lightness().to_bits(), lightness.to_bits());
            assert_eq!(inverse(&direct)[0].to_bits(), lightness.to_bits());
        }
    }
}

#[test]
fn hypot_preserves_subnormal_and_tiny_nonzero_chroma_without_squaring() {
    for (forward, _, _) in CONVERSIONS {
        let subnormal = forward([0.5, f64::from_bits(3), f64::from_bits(4)]).unwrap();
        assert_eq!(subnormal.chroma().to_bits(), 5);
        assert_eq!(subnormal.hue_degrees(), None);
        let scale = 2.0_f64.powi(-600);
        let tiny = forward([0.5, 3.0 * scale, 4.0 * scale]).unwrap();
        assert_eq!(tiny.chroma(), 5.0 * scale);
        assert_eq!(tiny.hue_degrees(), None);
    }
}

#[test]
fn hypot_avoids_intermediate_overflow_but_rejects_unrepresentable_chroma() {
    for (forward, _, _) in CONVERSIONS {
        let scale = 2.0_f64.powi(600);
        let large = forward([0.5, 3.0 * scale, 4.0 * scale]).unwrap();
        assert_eq!(large.chroma(), 5.0 * scale);
        close(large.hue_degrees().unwrap(), 53.13010235415598, 1e-12);
        assert_eq!(forward([0.5, f64::MAX, 0.0]).unwrap().chroma(), f64::MAX);
        for [a, b] in [[f64::MAX, f64::MAX], [-f64::MAX, f64::MAX]] {
            assert_eq!(forward([0.5, a, b]), Err(Error::UnrepresentableResult));
        }
    }
}

#[test]
fn every_nonfinite_coordinate_is_rejected_in_documented_order() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for component in 0..3 {
            let mut input = [0.5, 3.0, 4.0];
            input[component] = invalid;
            for (forward, _, _) in CONVERSIONS {
                assert_eq!(forward(input), Err(Error::NonFiniteInput { component }));
            }
            assert_eq!(
                Polar::try_new(input[0], input[1], Some(input[2])),
                Err(Error::NonFiniteInput { component })
            );
        }
    }
}

#[test]
fn nonfinite_input_precedes_negative_chroma_and_required_result_overflow() {
    for (input, component) in [
        ([f64::NAN, f64::INFINITY, f64::NEG_INFINITY], 0),
        ([0.5, f64::INFINITY, f64::NAN], 1),
        ([0.5, -1.0, f64::NAN], 2),
    ] {
        assert_eq!(
            Polar::try_new(input[0], input[1], Some(input[2])),
            Err(Error::NonFiniteInput { component })
        );
        for (forward, _, _) in CONVERSIONS {
            assert_eq!(forward(input), Err(Error::NonFiniteInput { component }));
        }
    }
    for (forward, _, _) in CONVERSIONS {
        assert_eq!(
            forward([f64::NAN, f64::MAX, f64::MAX]),
            Err(Error::NonFiniteInput { component: 0 })
        );
    }
    for chroma in [-f64::MAX, -1.0, -f64::from_bits(1)] {
        assert_eq!(
            Polar::try_new(0.5, chroma, None),
            Err(Error::NegativeChroma)
        );
        assert_eq!(
            Polar::try_new(0.5, chroma, Some(0.0)),
            Err(Error::NegativeChroma)
        );
    }
}

#[test]
fn normalized_hue_wraps_turns_and_canonicalizes_zero_and_rounded_endpoint() {
    for (input, expected) in [
        (-0.0, 0.0_f64),
        (0.0, 0.0),
        (360.0, 0.0),
        (-360.0, 0.0),
        (720.0, 0.0),
        (-720.0, 0.0),
        (450.0, 90.0),
        (-90.0, 270.0),
        (-450.0, 270.0),
        (-f64::from_bits(1), 0.0),
        (360.0 * 2.0_f64.powi(40) + 90.0, 90.0),
    ] {
        let hue = Polar::try_new(0.5, 1.0, Some(input))
            .unwrap()
            .hue_degrees()
            .unwrap();
        assert_eq!(hue.to_bits(), expected.to_bits(), "input {input:?}");
    }
    for input in [f64::MAX, -f64::MAX] {
        // MAX is (2^53-1)*2^971; reducing that integer modulo 360 gives 128.
        let expected: f64 = if input.is_sign_positive() {
            128.0
        } else {
            232.0
        };
        assert_eq!(
            Polar::try_new(0.5, 1.0, Some(input)).unwrap().hue_degrees(),
            Some(expected)
        );
    }
}

#[test]
fn zero_chroma_and_forward_zero_hue_have_positive_zero_bits() {
    for chroma in [-0.0, 0.0] {
        let polar = Polar::try_new(-0.0, chroma, Some(-0.0)).unwrap();
        assert_eq!(polar.chroma().to_bits(), 0.0_f64.to_bits());
        assert_eq!(polar.hue_degrees().unwrap().to_bits(), 0.0_f64.to_bits());
    }
    for (forward, _, _) in CONVERSIONS {
        for a in [-0.0, 0.0] {
            for b in [-0.0, 0.0] {
                let polar = forward([0.5, a, b]).unwrap();
                assert_eq!(polar.chroma().to_bits(), 0.0_f64.to_bits());
                assert_eq!(polar.hue_degrees(), None);
            }
        }
        for b in [-0.0, 0.0, -f64::from_bits(1)] {
            assert_eq!(
                forward([0.5, 5.0, b])
                    .unwrap()
                    .hue_degrees()
                    .unwrap()
                    .to_bits(),
                0.0_f64.to_bits()
            );
        }
    }
}

#[test]
fn maximum_chroma_inverse_stays_finite_for_present_normalized_hues() {
    for hue in [0.0, 45.0, 90.0, 180.0, 270.0, -f64::MAX, f64::MAX] {
        let polar = Polar::try_new(-0.0, f64::MAX, Some(hue)).unwrap();
        for (_, inverse, _) in CONVERSIONS {
            let actual = inverse(&polar);
            assert_eq!(actual[0].to_bits(), (-0.0_f64).to_bits());
            assert!(actual[1].is_finite() && actual[2].is_finite());
            assert!(actual[1].abs() <= f64::MAX && actual[2].abs() <= f64::MAX);
        }
    }
}

#[test]
fn ordinary_round_trips_allow_composed_rounding_and_neutral_loss_is_explicit() {
    for (forward, inverse, epsilon) in CONVERSIONS {
        for input in [[0.5, 0.1, -0.2], [50.0, -30.0, 40.0], [-2.0, -2.0, -1.0]] {
            let actual = inverse(&forward(input).unwrap());
            assert_eq!(actual[0].to_bits(), input[0].to_bits());
            // 64 epsilons times the largest axis bounds hypot, atan2, degree
            // normalization, radian conversion, trig and multiplication here.
            let tolerance = 64.0 * f64::EPSILON * input[1].abs().max(input[2].abs());
            close(actual[1], input[1], tolerance);
            close(actual[2], input[2], tolerance);
        }
        let input = [0.5, epsilon / 4.0, -epsilon / 4.0];
        let actual = inverse(&forward(input).unwrap());
        assert_eq!(actual, [0.5, 0.0, 0.0]);
        assert_ne!(actual, input);
    }
}

#[test]
fn numerical_error_has_standard_traits_and_no_fabricated_source() {
    fn standard_error<T: std::error::Error + Clone + Copy + Eq>(error: T) {
        assert!(error.source().is_none());
        assert!(!error.to_string().is_empty());
    }
    standard_error(Error::NonFiniteInput { component: 2 });
    standard_error(Error::NegativeChroma);
    standard_error(Error::UnrepresentableResult);
}
