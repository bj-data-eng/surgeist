#![forbid(unsafe_code)]

//! Independent Color 4 CRD 2026-09-08 §§7–8 algebraic expectations.
//! The hexagon vertices, dyadic mixtures, extended coordinates and threshold
//! controls are derived from the selected numerical contract, not from output
//! of the implementation. Round trips supplement explicit coordinate checks.

use surgeist_color::{
    Finite, HslCoordinates, HwbCoordinates, convert_hsl_to_srgb, convert_hwb_to_srgb,
    convert_srgb_to_hsl, convert_srgb_to_hwb,
};

fn finite(value: f64) -> Finite {
    Finite::new(value).expect("finite test coordinate")
}

/// Plain coordinates of the HSL carrier.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Hsl(HslCoordinates);
impl Hsl {
    fn new(hue: Option<f64>, saturation: f64, lightness: f64) -> Self {
        Self(HslCoordinates::new(
            hue.map(finite),
            finite(saturation),
            finite(lightness),
        ))
    }
    fn hue_degrees(&self) -> Option<f64> {
        self.0.hue_degrees().map(Finite::get)
    }
    fn saturation(&self) -> f64 {
        self.0.saturation().get()
    }
    fn lightness(&self) -> f64 {
        self.0.lightness().get()
    }
}
/// Plain coordinates of the HWB carrier.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Hwb(HwbCoordinates);
impl Hwb {
    fn new(hue: Option<f64>, whiteness: f64, blackness: f64) -> Self {
        Self(HwbCoordinates::new(
            hue.map(finite),
            finite(whiteness),
            finite(blackness),
        ))
    }
    fn hue_degrees(&self) -> Option<f64> {
        self.0.hue_degrees().map(Finite::get)
    }
    fn whiteness(&self) -> f64 {
        self.0.whiteness().get()
    }
    fn blackness(&self) -> f64 {
        self.0.blackness().get()
    }
}
fn hsl_to_srgb(hsl: &Hsl) -> [f64; 3] {
    convert_hsl_to_srgb(&hsl.0).map(Finite::get)
}
fn hwb_to_srgb(hwb: &Hwb) -> [f64; 3] {
    convert_hwb_to_srgb(&hwb.0).map(Finite::get)
}
fn srgb_to_hsl(rgb: [f64; 3]) -> Hsl {
    Hsl(convert_srgb_to_hsl(rgb.map(finite)))
}
fn srgb_to_hwb(rgb: [f64; 3]) -> Hwb {
    Hwb(convert_srgb_to_hwb(rgb.map(finite)))
}

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
        assert_eq!(hsl_to_srgb(&Hsl::new(Some(hue), 100.0, 50.0)), rgb);
        assert_eq!(hwb_to_srgb(&Hwb::new(Some(hue), 0.0, 0.0)), rgb);
        let hsl = srgb_to_hsl(rgb);
        assert_eq!(hsl.hue_degrees(), Some(hue));
        assert_eq!(hsl.saturation(), 100.0);
        assert_eq!(hsl.lightness(), 50.0);
        let hwb = srgb_to_hwb(rgb);
        assert_eq!(hwb.hue_degrees(), Some(hue));
        assert_eq!(hwb.whiteness(), 0.0);
        assert_eq!(hwb.blackness(), 0.0);
    }
    assert_eq!(
        hsl_to_srgb(&Hsl::new(Some(-90.0), 100.0, 50.0)),
        [0.5, 0.0, 1.0]
    );
    // A reference saturation of 1 means one percent, not full saturation.
    rgb_close(
        hsl_to_srgb(&Hsl::new(Some(0.0), 1.0, 50.0)),
        [0.505, 0.495, 0.495],
        2.0 * f64::EPSILON,
    );
}

#[test]
fn hwb_dyadic_mixture_and_sum_normalization_have_explicit_rgb_results() {
    assert_eq!(
        hwb_to_srgb(&Hwb::new(Some(150.0), 25.0, 12.5)),
        [0.25, 0.875, 0.5625]
    );
    for (white, black) in [(25.0, 75.0), (50.0, 150.0)] {
        for hue in [None, Some(0.0), Some(150.0), Some(300.0)] {
            let hwb = Hwb::new(hue, white, black);
            assert_eq!(hwb_to_srgb(&hwb), [0.25; 3]);
            assert_eq!(hwb.whiteness(), white);
            assert_eq!(hwb.blackness(), black);
            assert_eq!(hwb.hue_degrees(), hue);
        }
    }
    let hwb = srgb_to_hwb([0.25, 0.875, 0.5625]);
    assert_eq!(hwb.hue_degrees(), Some(150.0));
    assert_eq!(hwb.whiteness(), 25.0);
    assert_eq!(hwb.blackness(), 12.5);
}

#[test]
fn finite_extended_forward_coordinates_are_not_clipped() {
    for (hsl, expected) in [
        (Hsl::new(Some(0.0), 200.0, 50.0), [1.5, -0.5, -0.5]),
        (Hsl::new(Some(0.0), 100.0, 150.0), [1.0, 2.0, 2.0]),
        (Hsl::new(Some(30.0), 100.0, -50.0), [-1.0, -0.5, 0.0]),
    ] {
        assert_eq!(hsl_to_srgb(&hsl), expected);
    }
    assert_eq!(
        hwb_to_srgb(&Hwb::new(Some(30.0), -25.0, -25.0)),
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
        let hsl = srgb_to_hsl(rgb);
        assert_eq!(hsl.hue_degrees(), Some(hue));
        assert_eq!(hsl.saturation(), saturation);
        assert_eq!(hsl.lightness(), lightness);
    }
    let hwb = srgb_to_hwb([-1.0, -0.5, 0.0]);
    assert_eq!(hwb.hue_degrees(), Some(210.0));
    assert_eq!(hwb.whiteness(), -100.0);
    assert_eq!(hwb.blackness(), 100.0);
}

#[test]
fn exact_gray_has_missing_hue_and_retained_neutral_coordinates() {
    let hsl = srgb_to_hsl([0.375; 3]);
    assert_eq!(hsl.hue_degrees(), None);
    assert_eq!(hsl.saturation(), 0.0);
    assert_eq!(hsl.lightness(), 37.5);
    let hwb = srgb_to_hwb([0.375; 3]);
    assert_eq!(hwb.hue_degrees(), None);
    assert_eq!(hwb.whiteness(), 37.5);
    assert_eq!(hwb.blackness(), 62.5);
    for hue in [None, Some(0.0), Some(120.0), Some(300.0)] {
        let hsl = Hsl::new(hue, 0.0, 25.0);
        assert_eq!(hsl.hue_degrees(), hue);
        assert_eq!(hsl_to_srgb(&hsl), [0.25; 3]);
    }
}

#[test]
fn singular_hsl_lightness_discards_saturation_and_hue_before_difference_overflow() {
    for (rgb, lightness) in [
        ([-1.0, 0.0, 1.0], 0.0),
        ([0.0, 1.0, 2.0], 100.0),
        ([f64::MAX, -f64::MAX, 0.0], 0.0),
    ] {
        let hsl = srgb_to_hsl(rgb);
        assert_eq!(hsl.hue_degrees(), None);
        assert_eq!(hsl.saturation().to_bits(), 0.0_f64.to_bits());
        assert_eq!(hsl.lightness(), lightness);
    }
}

#[test]
fn binary_exact_nearly_gray_controls_retain_small_channels_with_missing_hue() {
    let x = 2.0_f64.powi(-18);
    let rgb = [0.5 + x, 0.5, 0.5 - x];
    let hsl = srgb_to_hsl(rgb);
    assert_eq!(hsl.hue_degrees(), None);
    assert_eq!(hsl.saturation(), 0.000762939453125);
    assert_eq!(hsl.lightness(), 50.0);
    let hwb = srgb_to_hwb(rgb);
    assert_eq!(hwb.hue_degrees(), None);
    // 50 - 100/2^18 = 13107100/262144, the exact dyadic percentage.
    assert_eq!(hwb.whiteness(), 13_107_100.0 / 262_144.0);
    assert_eq!(hwb.blackness(), 13_107_100.0 / 262_144.0);
    let x = 2.0_f64.powi(-17);
    let rgb = [0.5 + x, 0.5, 0.5 - x];
    let hsl = srgb_to_hsl(rgb);
    assert_eq!(hsl.hue_degrees(), Some(30.0));
    assert_eq!(hsl.saturation(), 0.00152587890625);
    assert_eq!(hsl.lightness(), 50.0);
    let hwb = srgb_to_hwb(rgb);
    assert_eq!(hwb.hue_degrees(), Some(30.0));
    assert_eq!(hwb.whiteness(), 49.999237060546875);
    assert_eq!(hwb.blackness(), 49.999237060546875);
}

#[test]
fn exact_hsl_epsilon_is_inclusive_and_nearby_computed_ratios_cross_it() {
    // L = 50001, half-difference = 0.5, denominator = -50000:
    // abs(S) = 0.5 / 50000 = 1/100000, exactly the rounded epsilon.
    let hsl = srgb_to_hsl([50001.5, 50001.0, 50000.5]);
    assert_eq!(hsl.hue_degrees(), None);
    assert_eq!(hsl.saturation(), 0.001);
    assert_eq!(hsl.lightness(), 5000100.0);
    // Shift every input by an exactly representable amount. Difference stays
    // one; increasing abs(1-L) decreases S, and decreasing it increases S.
    let step = 2.0_f64.powi(-36);
    let below = srgb_to_hsl([50001.5 + step, 50001.0 + step, 50000.5 + step]);
    assert!(below.saturation() < 0.001);
    assert_eq!(below.hue_degrees(), None);
    let above = srgb_to_hsl([50001.5 - step, 50001.0 - step, 50000.5 - step]);
    assert!(above.saturation() > 0.001);
    assert_eq!(above.hue_degrees(), Some(210.0));
}

#[test]
fn exact_hwb_epsilon_sum_is_inclusive_and_proximity_controls_cross_it() {
    // W = 0 and B = 1-epsilon, using the same rounded binary64 threshold.
    let epsilon = 0.00001;
    let at = srgb_to_hwb([epsilon, 0.0, 0.0]);
    assert_eq!(at.hue_degrees(), None);
    assert_eq!(at.whiteness(), 0.0);
    close(at.blackness(), 99.999, 2e-14);
    // These offsets exceed the ULP of B near 1; adjacent authored saturation
    // inputs need not produce adjacent computed saturation or HWB sums.
    let step = 2.0_f64.powi(-51);
    assert_eq!(srgb_to_hwb([epsilon - step, 0.0, 0.0]).hue_degrees(), None);
    assert_eq!(
        srgb_to_hwb([epsilon + step, 0.0, 0.0]).hue_degrees(),
        Some(0.0)
    );
}

#[test]
fn constructors_retain_present_powerless_hue_and_missing_hue_at_any_coordinates() {
    for saturation in [0.0, 0.0005, 0.001] {
        let hsl = Hsl::new(Some(120.0), saturation, 50.0);
        assert_eq!(hsl.hue_degrees(), Some(120.0));
        assert_eq!(hsl.saturation(), saturation);
    }
    let hsl = Hsl::new(Some(120.0), 0.001, 50.0);
    rgb_close(hsl_to_srgb(&hsl), [0.499995, 0.500005, 0.499995], 2e-16);
    for (white, black) in [(25.0, 75.0), (50.0, 150.0), (49.99975, 49.99975)] {
        let hwb = Hwb::new(Some(120.0), white, black);
        assert_eq!(hwb.hue_degrees(), Some(120.0));
        assert_eq!(hwb.whiteness(), white);
        assert_eq!(hwb.blackness(), black);
    }
    for saturation in [0.0, 0.001, 100.0, f64::MAX] {
        assert_eq!(Hsl::new(None, saturation, 50.0).hue_degrees(), None);
    }
    for (white, black) in [(0.0, 0.0), (-25.0, -25.0), (f64::MAX, -f64::MAX)] {
        assert_eq!(Hwb::new(None, white, black).hue_degrees(), None);
    }
    assert_eq!(hsl_to_srgb(&Hsl::new(None, 100.0, 50.0)), [1.0, 0.0, 0.0]);
    assert_eq!(hwb_to_srgb(&Hwb::new(None, 0.0, 0.0)), [1.0, 0.0, 0.0]);
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
            Hsl::new(Some(input), 0.0, 0.0)
                .hue_degrees()
                .unwrap()
                .to_bits(),
            expected.to_bits()
        );
        assert_eq!(
            Hwb::new(Some(input), 0.0, 0.0)
                .hue_degrees()
                .unwrap()
                .to_bits(),
            expected.to_bits()
        );
    }
    for zero in [-0.0, 0.0] {
        let hsl = Hsl::new(Some(zero), zero, zero);
        assert_eq!(hsl.hue_degrees().unwrap().to_bits(), 0.0_f64.to_bits());
        assert_eq!(hsl.saturation().to_bits(), 0.0_f64.to_bits());
        assert_eq!(hsl.lightness().to_bits(), zero.to_bits());
        let hwb = Hwb::new(Some(zero), zero, zero);
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
            Hsl::new(None, 100.0, value).lightness().to_bits(),
            value.to_bits()
        );
        let hwb = Hwb::new(None, value, value);
        assert_eq!(hwb.whiteness().to_bits(), value.to_bits());
        assert_eq!(hwb.blackness().to_bits(), value.to_bits());
    }
}

#[test]
fn negative_saturation_is_the_same_color_at_the_opposite_hue() {
    // A negative saturation scales the hue offsets the other way, which is
    // the positive saturation of the complementary hue.
    for (hue, rotated) in [(None, 180.0), (Some(0.0), 180.0), (Some(270.0), 90.0)] {
        for saturation in [-f64::MAX, -1.0, -f64::from_bits(1)] {
            let hsl = Hsl::new(hue, saturation, 50.0);
            assert_eq!(hsl.saturation(), saturation.abs());
            assert_eq!(hsl.hue_degrees(), Some(rotated));
        }
    }
    assert_eq!(
        hsl_to_srgb(&Hsl::new(Some(0.0), -100.0, 50.0)),
        [0.0, 1.0, 1.0]
    );
}

#[test]
fn maximum_saturation_at_singular_lightness_and_extreme_hwb_remain_finite() {
    for hue in [None, Some(0.0), Some(45.0), Some(120.0), Some(300.0)] {
        for (lightness, expected) in [(0.0, [0.0; 3]), (100.0, [1.0; 3])] {
            let hsl = Hsl::new(hue, f64::MAX, lightness);
            assert_eq!(hsl_to_srgb(&hsl), expected);
        }
        assert_eq!(hwb_to_srgb(&Hwb::new(hue, f64::MAX, f64::MAX)), [0.5; 3]);
        for (white, black) in [
            (f64::MAX, -f64::MAX),
            (-f64::MAX, f64::MAX),
            (-f64::MAX, -f64::MAX),
        ] {
            assert!(
                hwb_to_srgb(&Hwb::new(hue, white, black))
                    .into_iter()
                    .all(f64::is_finite)
            );
        }
    }
}

#[test]
fn representable_extreme_gray_and_overflowing_difference_have_finite_hsl_results() {
    let gray = f64::MAX / 128.0;
    let hsl = srgb_to_hsl([gray; 3]);
    assert_eq!(hsl.hue_degrees(), None);
    assert_eq!(hsl.saturation(), 0.0);
    assert_eq!(hsl.lightness(), gray * 100.0);
    let hwb = srgb_to_hwb([gray; 3]);
    assert_eq!(hwb.hue_degrees(), None);
    assert_eq!(hwb.whiteness(), gray * 100.0);
    assert_eq!(hwb.blackness(), -gray * 100.0);

    // min=-766q, max=768q: L=q, half-difference=767q and 1-L
    // rounds to -q. The full difference 1534q overflows unnecessarily.
    let q = 2.0_f64.powi(1014);
    let hsl = srgb_to_hsl([768.0 * q, 0.0, -766.0 * q]);
    assert_eq!(hsl.saturation(), 76700.0);
    assert_eq!(hsl.lightness(), 100.0 * q);
    close(
        hsl.hue_degrees().unwrap(),
        180.0 + 60.0 * (766.0 / 1534.0),
        1e-12,
    );
}

#[test]
fn hsl_lightness_far_above_one_keeps_the_exact_white_vertex() {
    // Color 4 §7.1 with S = 1 and L > 1/2: f(n) = L - (1 - L) * t, which is
    // exactly 1 at t = -1 and 2L - 1 at t = 1. At L = 1e16 the binary64
    // difference 1 - L rounds to -L, so the direct form cancels to zero.
    let hsl = Hsl::new(Some(0.0), 100.0, 1e18);
    assert_eq!(hsl_to_srgb(&hsl), [1.0, 2e16, 2e16]);
    for hue in [120.0, 240.0] {
        let channels = hsl_to_srgb(&Hsl::new(Some(hue), 100.0, 1e18));
        assert_eq!(channels.iter().filter(|value| **value == 1.0).count(), 1);
        assert_eq!(channels.iter().filter(|value| **value == 2e16).count(), 2);
    }
}

#[test]
fn hsl_channel_overflow_is_the_signed_closest_finite_value() {
    // Values 4: a value that cannot be supported for range reasons converts
    // to the closest supported value. At hue 0, S = L = 1e298 makes the red
    // channel L + S(1 - L) ~ -1e596 and green/blue L + S(L - 1) ~ +1e596.
    let hsl = Hsl::new(Some(0.0), 1e300, 1e300);
    assert_eq!(hsl_to_srgb(&hsl), [-f64::MAX, f64::MAX, f64::MAX]);
    assert_eq!(
        hsl_to_srgb(&Hsl::new(None, f64::MAX, f64::MAX)),
        [-f64::MAX, f64::MAX, f64::MAX]
    );
    // L <= 1/2: f(n) = L(1 - S t); a hugely negative L with huge S.
    let hsl = Hsl::new(Some(0.0), 1e300, -1e300);
    assert_eq!(hsl_to_srgb(&hsl), [-f64::MAX, f64::MAX, f64::MAX]);
}

#[test]
fn hwb_extreme_whiteness_or_blackness_keeps_exact_vertex_channels() {
    // Color 4 §8.1 with W + B < 1: channel = basis (1 - W - B) + W, which is
    // 1 - B at basis 1 and W at basis 0. At |W| = 1e16 the direct sum
    // 1 - W rounds away the 1.
    let hwb = Hwb::new(Some(0.0), -1e18, 0.0);
    assert_eq!(hwb_to_srgb(&hwb), [1.0, -1e16, -1e16]);
    let hwb = Hwb::new(Some(0.0), 0.0, -1e18);
    assert_eq!(hwb_to_srgb(&hwb), [1e16 + 1.0, 0.0, 0.0]);
}

#[test]
fn unrepresentable_required_percentages_are_the_signed_closest_finite_value() {
    // A gray at ±MAX has lightness and whiteness 100·(±MAX) and blackness
    // 100·(1 ∓ MAX), all beyond binary64; each is the closest finite value.
    for (rgb, sign) in [([f64::MAX; 3], 1.0), ([-f64::MAX; 3], -1.0)] {
        let hsl = srgb_to_hsl(rgb);
        assert_eq!(hsl.hue_degrees(), None);
        assert_eq!(hsl.saturation(), 0.0);
        assert_eq!(hsl.lightness(), sign * f64::MAX);
        let hwb = srgb_to_hwb(rgb);
        assert_eq!(hwb.hue_degrees(), None);
        assert_eq!(hwb.whiteness(), sign * f64::MAX);
        assert_eq!(hwb.blackness(), -sign * f64::MAX);
    }
}

#[test]
fn ordinary_underflow_and_powerless_loss_are_explicit_in_supplementary_round_trips() {
    // Dividing the smallest subnormal percentage by 100 underflows ordinarily.
    assert_eq!(
        hsl_to_srgb(&Hsl::new(None, f64::from_bits(1), f64::from_bits(1))),
        [0.0; 3]
    );
    for rgb in [[0.125, 0.5, 0.875], [1.5, -0.5, -0.5], [-1.0, -0.5, 0.0]] {
        rgb_close(hsl_to_srgb(&srgb_to_hsl(rgb)), rgb, 64.0 * f64::EPSILON);
        rgb_close(hwb_to_srgb(&srgb_to_hwb(rgb)), rgb, 64.0 * f64::EPSILON);
    }
    let x = 2.0_f64.powi(-18);
    let rgb = [0.5 + x, 0.5, 0.5 - x];
    let hsl_rgb = hsl_to_srgb(&srgb_to_hsl(rgb));
    assert_eq!(hsl_rgb, [0.5 + x, 0.5 - x, 0.5 - x]);
    let hwb_rgb = hwb_to_srgb(&srgb_to_hwb(rgb));
    assert_eq!(hwb_rgb, [0.5 + x, 0.5 - x, 0.5 - x]);
    assert_ne!(hsl_rgb, rgb);
    assert_ne!(hwb_rgb, rgb);
}
