#![forbid(unsafe_code)]

//! Expectations follow Color 4 CRD 2026-09-08 §§4.4.1/9.3–9.6, not sample-code outputs.
//! Cardinal/diagonal angles and scaled 3-4-5 triangles are independent geometry
//! oracles. Decimal 53.13010235415598° is the independently rounded atan(4/3).

use surgeist_color::{
    Finite, PolarCoordinates, convert_lab_to_lch, convert_lch_to_lab, convert_oklab_to_oklch,
    convert_oklch_to_oklab,
};

type Forward = fn([f64; 3]) -> Polar;
type Inverse = fn(&Polar) -> [f64; 3];

const CONVERSIONS: [(Forward, Inverse, f64); 2] = [
    (lab_to_lch, lch_to_lab, 0.0015),
    (oklab_to_oklch, oklch_to_oklab, 0.000004),
];

fn finite(value: f64) -> Finite {
    Finite::new(value).expect("finite test coordinate")
}

/// Plain coordinates of the polar carrier.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Polar(PolarCoordinates);
impl Polar {
    fn new(lightness: f64, chroma: f64, hue: Option<f64>) -> Self {
        Self(PolarCoordinates::new(
            finite(lightness),
            finite(chroma),
            hue.map(finite),
        ))
    }
    fn lightness(&self) -> f64 {
        self.0.lightness().get()
    }
    fn chroma(&self) -> f64 {
        self.0.chroma().get()
    }
    fn hue_degrees(&self) -> Option<f64> {
        self.0.hue_degrees().map(Finite::get)
    }
}
fn lab_to_lch(lab: [f64; 3]) -> Polar {
    Polar(convert_lab_to_lch(lab.map(finite)))
}
fn oklab_to_oklch(oklab: [f64; 3]) -> Polar {
    Polar(convert_oklab_to_oklch(oklab.map(finite)))
}
fn lch_to_lab(lch: &Polar) -> [f64; 3] {
    convert_lch_to_lab(&lch.0).map(Finite::get)
}
fn oklch_to_oklab(oklch: &Polar) -> [f64; 3] {
    convert_oklch_to_oklab(&oklch.0).map(Finite::get)
}

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
            let polar = forward([50.0, a, b]);
            close(polar.chroma(), chroma, 2.0 * f64::EPSILON);
            close(polar.hue_degrees().unwrap(), hue, 1e-12);
            let reconstructed = inverse(&Polar::new(50.0, chroma, Some(hue)));
            close(reconstructed[1], a, 1e-14);
            close(reconstructed[2], b, 1e-14);
        }
    }
}

#[test]
fn three_four_five_geometry_uses_each_spaces_concrete_number_units() {
    for (forward, inverse, scale, lightness) in [
        (lab_to_lch as Forward, lch_to_lab as Inverse, 1.0, 50.0),
        (
            oklab_to_oklch as Forward,
            oklch_to_oklab as Inverse,
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
            let polar = forward([lightness, a, b]);
            assert_eq!(polar.lightness(), lightness);
            close(polar.chroma(), 5.0 * scale, 8.0 * f64::EPSILON * scale);
            close(polar.hue_degrees().unwrap(), hue, 1e-12);
            let actual = inverse(&Polar::new(lightness, 5.0 * scale, Some(hue)));
            close(actual[1], a, 1e-14 * scale);
            close(actual[2], b, 1e-14 * scale);
        }
    }
}

#[test]
fn exact_epsilon_and_adjacent_values_apply_inclusive_neutral_cleanup() {
    for (forward, inverse, epsilon) in CONVERSIONS {
        let below = f64::from_bits(epsilon.to_bits() - 1);
        let above = f64::from_bits(epsilon.to_bits() + 1);
        for chroma in [0.0, below, epsilon] {
            let polar = forward([12.0, 0.0, -chroma]);
            assert_eq!(polar.chroma().to_bits(), 0.0_f64.to_bits());
            assert_eq!(polar.hue_degrees(), None);
            assert_eq!(inverse(&polar), [12.0, 0.0, 0.0]);
        }
        let polar = forward([12.0, 0.0, -above]);
        assert_eq!(polar.chroma(), above);
        assert_eq!(polar.hue_degrees(), Some(270.0));
    }
}

#[test]
fn lab_conversion_zeros_positive_powerless_chroma_without_changing_manual_coordinates() {
    // Color 4 §4.4.1 requires cleanup after conversion; manually supplied
    // powerless channels retain their values. Half the Lab epsilon is positive.
    let chroma = 0.0015 / 2.0;
    let converted = lab_to_lch([-0.0, chroma, 0.0]);
    assert_eq!(converted.lightness().to_bits(), (-0.0_f64).to_bits());
    assert_eq!(converted.chroma().to_bits(), 0.0_f64.to_bits());
    assert_eq!(converted.hue_degrees(), None);
    let manual = Polar::new(-0.0, chroma, Some(90.0));
    assert_eq!(manual.chroma(), chroma);
    assert_eq!(manual.hue_degrees(), Some(90.0));
}

#[test]
fn oklab_conversion_zeros_positive_powerless_chroma_without_changing_manual_coordinates() {
    // The same cross-cutting cleanup uses Oklab's own 0.000004 epsilon.
    let chroma = 0.000004 / 2.0;
    let converted = oklab_to_oklch([0.5, 0.0, -chroma]);
    assert_eq!(converted.lightness(), 0.5);
    assert_eq!(converted.chroma().to_bits(), 0.0_f64.to_bits());
    assert_eq!(converted.hue_degrees(), None);
    let manual = Polar::new(0.5, chroma, Some(270.0));
    assert_eq!(manual.chroma(), chroma);
    assert_eq!(manual.hue_degrees(), Some(270.0));
}

#[test]
fn constructor_keeps_present_subthreshold_hue_and_missing_hue_at_any_chroma() {
    for (_, inverse, epsilon) in CONVERSIONS {
        for chroma in [0.0, epsilon / 2.0, epsilon] {
            let polar = Polar::new(0.5, chroma, Some(0.0));
            assert_eq!(polar.hue_degrees(), Some(0.0));
            assert_eq!(inverse(&polar), [0.5, chroma, 0.0]);
        }
        for chroma in [0.0, epsilon, 5.0, f64::MAX] {
            let polar = Polar::new(-0.0, chroma, None);
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
            let polar = forward([lightness, -3.0, 4.0]);
            assert_eq!(polar.lightness().to_bits(), lightness.to_bits());
            assert_eq!(inverse(&polar)[0].to_bits(), lightness.to_bits());
            let direct = Polar::new(lightness, 5.0, Some(45.0));
            assert_eq!(direct.lightness().to_bits(), lightness.to_bits());
            assert_eq!(inverse(&direct)[0].to_bits(), lightness.to_bits());
        }
    }
}

#[test]
fn conversion_cleans_subnormal_and_tiny_positive_chroma_to_neutral() {
    for (forward, _, _) in CONVERSIONS {
        let subnormal = forward([0.5, f64::from_bits(3), f64::from_bits(4)]);
        assert_eq!(subnormal.chroma().to_bits(), 0.0_f64.to_bits());
        assert_eq!(subnormal.hue_degrees(), None);
        let scale = 2.0_f64.powi(-600);
        let tiny = forward([0.5, 3.0 * scale, 4.0 * scale]);
        assert_eq!(tiny.chroma().to_bits(), 0.0_f64.to_bits());
        assert_eq!(tiny.hue_degrees(), None);
    }
}

#[test]
fn hypot_avoids_intermediate_overflow_and_saturates_unrepresentable_chroma() {
    for (forward, _, _) in CONVERSIONS {
        let scale = 2.0_f64.powi(600);
        let large = forward([0.5, 3.0 * scale, 4.0 * scale]);
        assert_eq!(large.chroma(), 5.0 * scale);
        close(large.hue_degrees().unwrap(), 53.13010235415598, 1e-12);
        assert_eq!(forward([0.5, f64::MAX, 0.0]).chroma(), f64::MAX);
        // The exact chroma sqrt(2)·MAX exceeds binary64, so it is the closest
        // finite value (CSS Values 4 closest supported value); the hue keeps
        // its exact diagonal.
        for ([a, b], hue) in [([f64::MAX, f64::MAX], 45.0), ([-f64::MAX, f64::MAX], 135.0)] {
            let polar = forward([0.5, a, b]);
            assert_eq!(polar.chroma(), f64::MAX);
            close(polar.hue_degrees().unwrap(), hue, 1e-12);
        }
    }
}

#[test]
fn negative_chroma_is_the_same_point_at_the_opposite_hue() {
    // A negative radius points the other way: C cos(H) = |C| cos(H + 180°).
    for (hue, rotated) in [(None, 180.0), (Some(0.0), 180.0), (Some(270.0), 90.0)] {
        for chroma in [-f64::MAX, -1.0, -f64::from_bits(1)] {
            let polar = Polar::new(0.5, chroma, hue);
            assert_eq!(polar.chroma(), chroma.abs());
            assert_eq!(polar.hue_degrees(), Some(rotated));
        }
    }
    for (_, inverse, _) in CONVERSIONS {
        let [_, a, b] = inverse(&Polar::new(0.5, -2.0, Some(0.0)));
        close(a, -2.0, 1e-15);
        close(b, 0.0, 1e-15);
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
        let hue = Polar::new(0.5, 1.0, Some(input)).hue_degrees().unwrap();
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
            Polar::new(0.5, 1.0, Some(input)).hue_degrees(),
            Some(expected)
        );
    }
}

#[test]
fn zero_chroma_and_forward_zero_hue_have_positive_zero_bits() {
    for chroma in [-0.0, 0.0] {
        let polar = Polar::new(-0.0, chroma, Some(-0.0));
        assert_eq!(polar.chroma().to_bits(), 0.0_f64.to_bits());
        assert_eq!(polar.hue_degrees().unwrap().to_bits(), 0.0_f64.to_bits());
    }
    for (forward, _, _) in CONVERSIONS {
        for a in [-0.0, 0.0] {
            for b in [-0.0, 0.0] {
                let polar = forward([0.5, a, b]);
                assert_eq!(polar.chroma().to_bits(), 0.0_f64.to_bits());
                assert_eq!(polar.hue_degrees(), None);
            }
        }
        for b in [-0.0, 0.0, -f64::from_bits(1)] {
            assert_eq!(
                forward([0.5, 5.0, b]).hue_degrees().unwrap().to_bits(),
                0.0_f64.to_bits()
            );
        }
    }
}

#[test]
fn maximum_chroma_inverse_stays_finite_for_present_normalized_hues() {
    for hue in [0.0, 45.0, 90.0, 180.0, 270.0, -f64::MAX, f64::MAX] {
        let polar = Polar::new(-0.0, f64::MAX, Some(hue));
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
            let actual = inverse(&forward(input));
            assert_eq!(actual[0].to_bits(), input[0].to_bits());
            // 64 epsilons times the largest axis bounds hypot, atan2, degree
            // normalization, radian conversion, trig and multiplication here.
            let tolerance = 64.0 * f64::EPSILON * input[1].abs().max(input[2].abs());
            close(actual[1], input[1], tolerance);
            close(actual[2], input[2], tolerance);
        }
        let input = [0.5, epsilon / 4.0, -epsilon / 4.0];
        let actual = inverse(&forward(input));
        assert_eq!(actual, [0.5, 0.0, 0.0]);
        assert_ne!(actual, input);
    }
}
