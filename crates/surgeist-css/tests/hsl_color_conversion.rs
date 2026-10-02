#![forbid(unsafe_code)]

//! Independent Color 4 CRD 2026-09-08 §§7–8 algebraic expectations.
//! The hexagon vertices, dyadic mixtures, extended coordinates and threshold
//! controls are derived from the selected numerical contract, not from output
//! of the implementation. Round trips supplement explicit coordinate checks.

use surgeist_css::{
    CssHslColorCoordinates as Hsl, CssHslHwbConversionError as Error,
    CssHwbColorCoordinates as Hwb, convert_hsl_to_srgb, convert_hwb_to_srgb, convert_srgb_to_hsl,
    convert_srgb_to_hwb,
};

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= tolerance,
        "actual {actual:?}, expected {expected:?}, tolerance {tolerance:?}"
    );
}

fn rgb_close(actual: [f64; 3], expected: [f64; 3], tolerance: f64) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        close(actual, expected, tolerance);
    }
}

#[test]
fn hsl_and_hwb_hexagon_vertices_use_encoded_rgb_reference_units() {
    for (hue, rgb) in [
        (0.0, [1.0, 0.0, 0.0]),
        (60.0, [1.0, 1.0, 0.0]),
        (120.0, [0.0, 1.0, 0.0]),
        (180.0, [0.0, 1.0, 1.0]),
        (240.0, [0.0, 0.0, 1.0]),
        (300.0, [1.0, 0.0, 1.0]),
    ] {
        assert_eq!(
            convert_hsl_to_srgb(&Hsl::try_new(Some(hue), 100.0, 50.0).unwrap()).unwrap(),
            rgb
        );
        assert_eq!(
            convert_hwb_to_srgb(&Hwb::try_new(Some(hue), 0.0, 0.0).unwrap()),
            rgb
        );
        let hsl = convert_srgb_to_hsl(rgb).unwrap();
        assert_eq!(hsl.hue_degrees(), Some(hue));
        assert_eq!(hsl.saturation(), 100.0);
        assert_eq!(hsl.lightness(), 50.0);
        let hwb = convert_srgb_to_hwb(rgb).unwrap();
        assert_eq!(hwb.hue_degrees(), Some(hue));
        assert_eq!(hwb.whiteness(), 0.0);
        assert_eq!(hwb.blackness(), 0.0);
    }
    assert_eq!(
        convert_hsl_to_srgb(&Hsl::try_new(Some(-90.0), 100.0, 50.0).unwrap()).unwrap(),
        [0.5, 0.0, 1.0]
    );
    // A reference saturation of 1 means one percent, not full saturation.
    rgb_close(
        convert_hsl_to_srgb(&Hsl::try_new(Some(0.0), 1.0, 50.0).unwrap()).unwrap(),
        [0.505, 0.495, 0.495],
        2.0 * f64::EPSILON,
    );
}

#[test]
fn hwb_dyadic_mixture_and_sum_normalization_have_explicit_rgb_results() {
    assert_eq!(
        convert_hwb_to_srgb(&Hwb::try_new(Some(150.0), 25.0, 12.5).unwrap()),
        [0.25, 0.875, 0.5625]
    );
    for (white, black) in [(25.0, 75.0), (50.0, 150.0)] {
        for hue in [None, Some(0.0), Some(150.0), Some(300.0)] {
            let hwb = Hwb::try_new(hue, white, black).unwrap();
            assert_eq!(convert_hwb_to_srgb(&hwb), [0.25; 3]);
            assert_eq!(hwb.whiteness(), white);
            assert_eq!(hwb.blackness(), black);
            assert_eq!(hwb.hue_degrees(), hue);
        }
    }
    let hwb = convert_srgb_to_hwb([0.25, 0.875, 0.5625]).unwrap();
    assert_eq!(hwb.hue_degrees(), Some(150.0));
    assert_eq!(hwb.whiteness(), 25.0);
    assert_eq!(hwb.blackness(), 12.5);
}

#[test]
fn finite_extended_forward_coordinates_are_not_clipped() {
    for (hsl, expected) in [
        (
            Hsl::try_new(Some(0.0), 200.0, 50.0).unwrap(),
            [1.5, -0.5, -0.5],
        ),
        (
            Hsl::try_new(Some(0.0), 100.0, 150.0).unwrap(),
            [1.0, 2.0, 2.0],
        ),
        (
            Hsl::try_new(Some(30.0), 100.0, -50.0).unwrap(),
            [-1.0, -0.5, 0.0],
        ),
    ] {
        assert_eq!(convert_hsl_to_srgb(&hsl).unwrap(), expected);
    }
    assert_eq!(
        convert_hwb_to_srgb(&Hwb::try_new(Some(30.0), -25.0, -25.0).unwrap()),
        [1.25, 0.5, -0.25]
    );
}

#[test]
fn extended_inverse_hue_rotation_is_specific_to_hsl() {
    for (rgb, hue, saturation, lightness) in [
        ([-1.0, -0.5, 0.0], 30.0, 100.0, -50.0),
        ([1.0, 2.0, 2.0], 0.0, 100.0, 150.0),
        ([1.5, -0.5, -0.5], 0.0, 200.0, 50.0),
    ] {
        let hsl = convert_srgb_to_hsl(rgb).unwrap();
        assert_eq!(hsl.hue_degrees(), Some(hue));
        assert_eq!(hsl.saturation(), saturation);
        assert_eq!(hsl.lightness(), lightness);
    }
    let hwb = convert_srgb_to_hwb([-1.0, -0.5, 0.0]).unwrap();
    assert_eq!(hwb.hue_degrees(), Some(210.0));
    assert_eq!(hwb.whiteness(), -100.0);
    assert_eq!(hwb.blackness(), 100.0);
}

#[test]
fn exact_gray_has_missing_hue_and_retained_neutral_coordinates() {
    let hsl = convert_srgb_to_hsl([0.375; 3]).unwrap();
    assert_eq!(hsl.hue_degrees(), None);
    assert_eq!(hsl.saturation(), 0.0);
    assert_eq!(hsl.lightness(), 37.5);
    let hwb = convert_srgb_to_hwb([0.375; 3]).unwrap();
    assert_eq!(hwb.hue_degrees(), None);
    assert_eq!(hwb.whiteness(), 37.5);
    assert_eq!(hwb.blackness(), 62.5);
    for hue in [None, Some(0.0), Some(120.0), Some(300.0)] {
        let hsl = Hsl::try_new(hue, 0.0, 25.0).unwrap();
        assert_eq!(hsl.hue_degrees(), hue);
        assert_eq!(convert_hsl_to_srgb(&hsl).unwrap(), [0.25; 3]);
    }
}

#[test]
fn singular_hsl_lightness_discards_saturation_and_hue_before_difference_overflow() {
    for (rgb, lightness) in [
        ([-1.0, 0.0, 1.0], 0.0),
        ([0.0, 1.0, 2.0], 100.0),
        ([f64::MAX, -f64::MAX, 0.0], 0.0),
    ] {
        let hsl = convert_srgb_to_hsl(rgb).unwrap();
        assert_eq!(hsl.hue_degrees(), None);
        assert_eq!(hsl.saturation().to_bits(), 0.0_f64.to_bits());
        assert_eq!(hsl.lightness(), lightness);
    }
}

#[test]
fn binary_exact_nearly_gray_controls_retain_small_channels_with_missing_hue() {
    let x = 2.0_f64.powi(-18);
    let rgb = [0.5 + x, 0.5, 0.5 - x];
    let hsl = convert_srgb_to_hsl(rgb).unwrap();
    assert_eq!(hsl.hue_degrees(), None);
    assert_eq!(hsl.saturation(), 0.000762939453125);
    assert_eq!(hsl.lightness(), 50.0);
    let hwb = convert_srgb_to_hwb(rgb).unwrap();
    assert_eq!(hwb.hue_degrees(), None);
    // 50 - 100/2^18 = 13107100/262144, the exact dyadic percentage.
    assert_eq!(hwb.whiteness(), 13_107_100.0 / 262_144.0);
    assert_eq!(hwb.blackness(), 13_107_100.0 / 262_144.0);
    let x = 2.0_f64.powi(-17);
    let rgb = [0.5 + x, 0.5, 0.5 - x];
    let hsl = convert_srgb_to_hsl(rgb).unwrap();
    assert_eq!(hsl.hue_degrees(), Some(30.0));
    assert_eq!(hsl.saturation(), 0.00152587890625);
    assert_eq!(hsl.lightness(), 50.0);
    let hwb = convert_srgb_to_hwb(rgb).unwrap();
    assert_eq!(hwb.hue_degrees(), Some(30.0));
    assert_eq!(hwb.whiteness(), 49.999237060546875);
    assert_eq!(hwb.blackness(), 49.999237060546875);
}

#[test]
fn exact_hsl_epsilon_is_inclusive_and_nearby_computed_ratios_cross_it() {
    // L = 50001, half-difference = 0.5, denominator = -50000:
    // abs(S) = 0.5 / 50000 = 1/100000, exactly the rounded epsilon.
    let hsl = convert_srgb_to_hsl([50001.5, 50001.0, 50000.5]).unwrap();
    assert_eq!(hsl.hue_degrees(), None);
    assert_eq!(hsl.saturation(), 0.001);
    assert_eq!(hsl.lightness(), 5000100.0);
    // Shift every input by an exactly representable amount. Difference stays
    // one; increasing abs(1-L) decreases S, and decreasing it increases S.
    let step = 2.0_f64.powi(-36);
    let below = convert_srgb_to_hsl([50001.5 + step, 50001.0 + step, 50000.5 + step]).unwrap();
    assert!(below.saturation() < 0.001);
    assert_eq!(below.hue_degrees(), None);
    let above = convert_srgb_to_hsl([50001.5 - step, 50001.0 - step, 50000.5 - step]).unwrap();
    assert!(above.saturation() > 0.001);
    assert_eq!(above.hue_degrees(), Some(210.0));
}

#[test]
fn exact_hwb_epsilon_sum_is_inclusive_and_proximity_controls_cross_it() {
    // W = 0 and B = 1-epsilon, using the same rounded binary64 threshold.
    let epsilon = 0.00001;
    let at = convert_srgb_to_hwb([epsilon, 0.0, 0.0]).unwrap();
    assert_eq!(at.hue_degrees(), None);
    assert_eq!(at.whiteness(), 0.0);
    close(at.blackness(), 99.999, 2e-14);
    // These offsets exceed the ULP of B near 1; adjacent authored saturation
    // inputs need not produce adjacent computed saturation or HWB sums.
    let step = 2.0_f64.powi(-51);
    assert_eq!(
        convert_srgb_to_hwb([epsilon - step, 0.0, 0.0])
            .unwrap()
            .hue_degrees(),
        None
    );
    assert_eq!(
        convert_srgb_to_hwb([epsilon + step, 0.0, 0.0])
            .unwrap()
            .hue_degrees(),
        Some(0.0)
    );
}

#[test]
fn constructors_retain_present_powerless_hue_and_missing_hue_at_any_coordinates() {
    for saturation in [0.0, 0.0005, 0.001] {
        let hsl = Hsl::try_new(Some(120.0), saturation, 50.0).unwrap();
        assert_eq!(hsl.hue_degrees(), Some(120.0));
        assert_eq!(hsl.saturation(), saturation);
    }
    let hsl = Hsl::try_new(Some(120.0), 0.001, 50.0).unwrap();
    rgb_close(
        convert_hsl_to_srgb(&hsl).unwrap(),
        [0.499995, 0.500005, 0.499995],
        2e-16,
    );
    for (white, black) in [(25.0, 75.0), (50.0, 150.0), (49.99975, 49.99975)] {
        let hwb = Hwb::try_new(Some(120.0), white, black).unwrap();
        assert_eq!(hwb.hue_degrees(), Some(120.0));
        assert_eq!(hwb.whiteness(), white);
        assert_eq!(hwb.blackness(), black);
    }
    for saturation in [0.0, 0.001, 100.0, f64::MAX] {
        assert_eq!(
            Hsl::try_new(None, saturation, 50.0).unwrap().hue_degrees(),
            None
        );
    }
    for (white, black) in [(0.0, 0.0), (-25.0, -25.0), (f64::MAX, -f64::MAX)] {
        assert_eq!(
            Hwb::try_new(None, white, black).unwrap().hue_degrees(),
            None
        );
    }
    assert_eq!(
        convert_hsl_to_srgb(&Hsl::try_new(None, 100.0, 50.0).unwrap()).unwrap(),
        [1.0, 0.0, 0.0]
    );
    assert_eq!(
        convert_hwb_to_srgb(&Hwb::try_new(None, 0.0, 0.0).unwrap()),
        [1.0, 0.0, 0.0]
    );
}

#[test]
fn constructors_normalize_hue_and_canonicalize_hue_and_saturation_zero_only() {
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
        // MAX = (2^53-1)*2^971; integer reduction modulo 360 is 128.
        (f64::MAX, 128.0),
        (-f64::MAX, 232.0),
    ] {
        assert_eq!(
            Hsl::try_new(Some(input), 0.0, 0.0)
                .unwrap()
                .hue_degrees()
                .unwrap()
                .to_bits(),
            expected.to_bits()
        );
        assert_eq!(
            Hwb::try_new(Some(input), 0.0, 0.0)
                .unwrap()
                .hue_degrees()
                .unwrap()
                .to_bits(),
            expected.to_bits()
        );
    }
    for zero in [-0.0, 0.0] {
        let hsl = Hsl::try_new(Some(zero), zero, zero).unwrap();
        assert_eq!(hsl.hue_degrees().unwrap().to_bits(), 0.0_f64.to_bits());
        assert_eq!(hsl.saturation().to_bits(), 0.0_f64.to_bits());
        assert_eq!(hsl.lightness().to_bits(), zero.to_bits());
        let hwb = Hwb::try_new(Some(zero), zero, zero).unwrap();
        assert_eq!(hwb.hue_degrees().unwrap().to_bits(), 0.0_f64.to_bits());
        assert_eq!(hwb.whiteness().to_bits(), zero.to_bits());
        assert_eq!(hwb.blackness().to_bits(), zero.to_bits());
    }
}

#[test]
fn constructors_preserve_all_finite_lightness_whiteness_and_blackness_bits() {
    for value in [
        -f64::MAX,
        -200.0,
        -0.0,
        0.0,
        f64::from_bits(1),
        37.5,
        200.0,
        f64::MAX,
    ] {
        assert_eq!(
            Hsl::try_new(None, 100.0, value)
                .unwrap()
                .lightness()
                .to_bits(),
            value.to_bits()
        );
        let hwb = Hwb::try_new(None, value, value).unwrap();
        assert_eq!(hwb.whiteness().to_bits(), value.to_bits());
        assert_eq!(hwb.blackness().to_bits(), value.to_bits());
    }
}

#[test]
fn nonfinite_inputs_are_rejected_in_each_documented_slot() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for component in 0..3 {
            let mut input = [30.0, 25.0, 50.0];
            input[component] = invalid;
            assert_eq!(
                Hsl::try_new(Some(input[0]), input[1], input[2]),
                Err(Error::NonFiniteInput { component })
            );
            assert_eq!(
                Hwb::try_new(Some(input[0]), input[1], input[2]),
                Err(Error::NonFiniteInput { component })
            );
            assert_eq!(
                convert_srgb_to_hsl(input),
                Err(Error::NonFiniteInput { component })
            );
            assert_eq!(
                convert_srgb_to_hwb(input),
                Err(Error::NonFiniteInput { component })
            );
        }
    }
    assert_eq!(
        Hsl::try_new(None, f64::NAN, 50.0),
        Err(Error::NonFiniteInput { component: 1 })
    );
    assert_eq!(
        Hwb::try_new(None, 25.0, f64::NAN),
        Err(Error::NonFiniteInput { component: 2 })
    );
}

#[test]
fn first_nonfinite_input_precedes_negative_saturation_or_required_overflow() {
    for (input, component) in [
        ([f64::NAN, f64::INFINITY, f64::NEG_INFINITY], 0),
        ([30.0, f64::INFINITY, f64::NAN], 1),
        ([30.0, -1.0, f64::NAN], 2),
    ] {
        assert_eq!(
            Hsl::try_new(Some(input[0]), input[1], input[2]),
            Err(Error::NonFiniteInput { component })
        );
        assert_eq!(
            Hwb::try_new(Some(input[0]), input[1], input[2]),
            Err(Error::NonFiniteInput { component })
        );
        assert_eq!(
            convert_srgb_to_hsl(input),
            Err(Error::NonFiniteInput { component })
        );
        assert_eq!(
            convert_srgb_to_hwb(input),
            Err(Error::NonFiniteInput { component })
        );
    }
    for input in [
        [f64::NAN, f64::MAX, f64::MAX],
        [f64::MAX, f64::NAN, f64::MAX],
        [f64::MAX, f64::MAX, f64::NAN],
    ] {
        let component = input.iter().position(|value| !value.is_finite()).unwrap();
        assert_eq!(
            convert_srgb_to_hsl(input),
            Err(Error::NonFiniteInput { component })
        );
        assert_eq!(
            convert_srgb_to_hwb(input),
            Err(Error::NonFiniteInput { component })
        );
    }
    for saturation in [-f64::MAX, -1.0, -f64::from_bits(1)] {
        for hue in [None, Some(0.0)] {
            assert_eq!(
                Hsl::try_new(hue, saturation, 50.0),
                Err(Error::NegativeSaturation)
            );
        }
    }
}

#[test]
fn maximum_saturation_at_singular_lightness_and_extreme_hwb_remain_finite() {
    for hue in [None, Some(0.0), Some(45.0), Some(120.0), Some(300.0)] {
        for (lightness, expected) in [(0.0, [0.0; 3]), (100.0, [1.0; 3])] {
            let hsl = Hsl::try_new(hue, f64::MAX, lightness).unwrap();
            assert_eq!(convert_hsl_to_srgb(&hsl).unwrap(), expected);
        }
        assert_eq!(
            convert_hwb_to_srgb(&Hwb::try_new(hue, f64::MAX, f64::MAX).unwrap()),
            [0.5; 3]
        );
        for (white, black) in [
            (f64::MAX, -f64::MAX),
            (-f64::MAX, f64::MAX),
            (-f64::MAX, -f64::MAX),
        ] {
            assert!(
                convert_hwb_to_srgb(&Hwb::try_new(hue, white, black).unwrap())
                    .into_iter()
                    .all(f64::is_finite)
            );
        }
    }
}

#[test]
fn representable_extreme_gray_and_overflowing_difference_have_finite_hsl_results() {
    let gray = f64::MAX / 128.0;
    let hsl = convert_srgb_to_hsl([gray; 3]).unwrap();
    assert_eq!(hsl.hue_degrees(), None);
    assert_eq!(hsl.saturation(), 0.0);
    assert_eq!(hsl.lightness(), gray * 100.0);
    let hwb = convert_srgb_to_hwb([gray; 3]).unwrap();
    assert_eq!(hwb.hue_degrees(), None);
    assert_eq!(hwb.whiteness(), gray * 100.0);
    assert_eq!(hwb.blackness(), -gray * 100.0);

    // min=-766q, max=768q: L=q, half-difference=767q and 1-L
    // rounds to -q. The full difference 1534q overflows unnecessarily.
    let q = 2.0_f64.powi(1014);
    let hsl = convert_srgb_to_hsl([768.0 * q, 0.0, -766.0 * q]).unwrap();
    assert_eq!(hsl.saturation(), 76700.0);
    assert_eq!(hsl.lightness(), 100.0 * q);
    close(
        hsl.hue_degrees().unwrap(),
        180.0 + 60.0 * (766.0 / 1534.0),
        1e-12,
    );
}

#[test]
fn genuinely_unrepresentable_required_results_use_the_typed_result_error() {
    assert_eq!(
        convert_hsl_to_srgb(&Hsl::try_new(None, f64::MAX, f64::MAX).unwrap()),
        Err(Error::UnrepresentableResult)
    );
    for rgb in [[f64::MAX; 3], [-f64::MAX; 3]] {
        assert_eq!(convert_srgb_to_hsl(rgb), Err(Error::UnrepresentableResult));
        assert_eq!(convert_srgb_to_hwb(rgb), Err(Error::UnrepresentableResult));
    }
}

#[test]
fn ordinary_underflow_and_powerless_loss_are_explicit_in_supplementary_round_trips() {
    // Dividing the smallest subnormal percentage by 100 underflows ordinarily.
    assert_eq!(
        convert_hsl_to_srgb(&Hsl::try_new(None, f64::from_bits(1), f64::from_bits(1)).unwrap())
            .unwrap(),
        [0.0; 3]
    );
    for rgb in [[0.125, 0.5, 0.875], [1.5, -0.5, -0.5], [-1.0, -0.5, 0.0]] {
        rgb_close(
            convert_hsl_to_srgb(&convert_srgb_to_hsl(rgb).unwrap()).unwrap(),
            rgb,
            64.0 * f64::EPSILON,
        );
        rgb_close(
            convert_hwb_to_srgb(&convert_srgb_to_hwb(rgb).unwrap()),
            rgb,
            64.0 * f64::EPSILON,
        );
    }
    let x = 2.0_f64.powi(-18);
    let rgb = [0.5 + x, 0.5, 0.5 - x];
    let hsl_rgb = convert_hsl_to_srgb(&convert_srgb_to_hsl(rgb).unwrap()).unwrap();
    assert_eq!(hsl_rgb, [0.5 + x, 0.5 - x, 0.5 - x]);
    let hwb_rgb = convert_hwb_to_srgb(&convert_srgb_to_hwb(rgb).unwrap());
    assert_eq!(hwb_rgb, [0.5 + x, 0.5 - x, 0.5 - x]);
    assert_ne!(hsl_rgb, rgb);
    assert_ne!(hwb_rgb, rgb);
}

#[test]
fn numerical_error_implements_standard_traits_without_a_fabricated_source() {
    fn standard_error<T: std::error::Error + Clone + Copy + Eq>(error: T) {
        assert!(error.source().is_none());
        assert!(!error.to_string().is_empty());
    }
    standard_error(Error::NonFiniteInput { component: 2 });
    standard_error(Error::NegativeSaturation);
    standard_error(Error::UnrepresentableResult);
}
