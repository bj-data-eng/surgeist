//! Colour components beyond the single-precision float range (#2889).
//!
//! A component whose binary64 value in the domain it enters the colour (the
//! number, the percentage scaled by its slot's reference, the angle in
//! degrees) is at least `THRESHOLD = 2^128 - 2^103`, the point at which a
//! binary64 rounds to an infinite binary32, is infinite: specified text writes
//! `calc(infinity)` and used resolution limits it to `±f32::MAX`. A component
//! numeral below `1e-46` is zero with its authored sign, a hue that is
//! infinite is 0 (Color 4 §4.3), clamped slots keep their endpoints and
//! `color-mix()` weights keep their closest value. Only the component root is
//! cut: a numeral inside a calculation outside `[1e-46, 1e41)` converts to a
//! bounded-error binary64 estimate, and calculation intermediates are exact.
//!
//! Expectations come from the rule above, the specifications it cites and
//! independent arithmetic (Python integers and IEEE binary64 for the neighbours
//! of the threshold, the leaf estimates and the Lab conversion), never from
//! the output of the code under test.
//!
//! The neighbours of the threshold, as integers (`2^75` is one binary64 step
//! there):
//!
//! | Value | Integer | Binary64 | Beyond |
//! | --- | --- | --- | --- |
//! | `D = THRESHOLD - 2^75` | 340282356779733623858607532500980858880 | itself | no |
//! | `M - 1 = THRESHOLD - 2^74 - 1` | 340282356779733642748073463979561713663 | `D` | no |
//! | `M = THRESHOLD - 2^74` | 340282356779733642748073463979561713664 | `THRESHOLD` (tie to even) | yes |
//! | `THRESHOLD - 1` | 340282356779733661637539395458142568447 | `THRESHOLD` | yes |
//! | `THRESHOLD` | 340282356779733661637539395458142568448 | itself | yes |

use crate::css::support::checked_color as color;
use surgeist_color::{Color, ColorSpace, PredefinedColorSpace};
use surgeist_css::color::CssColorComponent;
use surgeist_css::color::{
    CssAuthoredColorOrdinaryLimits, CssAuthoredColorOutcome, CssColorFactIdentity, CssColorFacts,
    CssColorGrammarRequest, CssColorInput, CssColorOrdinaryLimits, CssColorOrdinaryOutcome,
    CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact, parse_used_color,
};

/// `f32::MAX` as a binary64: 340282346638528859811704183484516925440.
const LARGEST: f64 = f32::MAX as f64;

const D: &str = "340282356779733623858607532500980858880";
const M_MINUS_ONE: &str = "340282356779733642748073463979561713663";
const M: &str = "340282356779733642748073463979561713664";
const THRESHOLD_MINUS_ONE: &str = "340282356779733661637539395458142568447";
const THRESHOLD: &str = "340282356779733661637539395458142568448";

fn facts() -> CssColorFacts<'static> {
    CssColorFacts::new(
        CssColorFactIdentity::new(1, 1),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    )
}

/// The used colour through the source front and through authored resolution of
/// the checked value; the two must be identical.
pub(super) fn resolved(source: &str) -> Color {
    let facts = facts();
    let used = match parse_used_color(
        CssColorInput::Utf8(source.as_bytes()),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        CssColorOrdinaryLimits::default(),
    )
    .map(|result| result.outcome())
    {
        Ok(CssColorOrdinaryOutcome::Used(value)) => value,
        other => panic!("{source}: source front {other:?}"),
    };
    let authored = match color(source).resolve(
        &facts,
        facts.identity(),
        CssAuthoredColorOrdinaryLimits::default(),
    ) {
        Ok(result) => match result.outcome() {
            CssAuthoredColorOutcome::Used(value) => *value,
            other => panic!("{source}: authored outcome {other:?}"),
        },
        Err(error) => panic!("{source}: authored front {error:?}"),
    };
    assert!(
        used.identical(authored),
        "{source}: fronts disagree {used:?} {authored:?}"
    );
    used
}

thread_local! {
    static PROBLEMS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Collects every mismatch of one test and reports them together when the test
/// ends, so a table of cases shows all of its failures, not the first.
pub(super) struct Report;
impl Report {
    pub(super) fn new() -> Self {
        PROBLEMS.with(|problems| problems.borrow_mut().clear());
        Self
    }
}
impl Drop for Report {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        let problems = PROBLEMS.with(|problems| problems.take());
        assert!(
            problems.is_empty(),
            "{} mismatches:\n{}",
            problems.len(),
            problems.join("\n")
        );
    }
}
pub(super) fn problem(text: String) {
    PROBLEMS.with(|problems| problems.borrow_mut().push(text));
}

pub(super) fn channels(source: &str) -> [f64; 3] {
    resolved(source)
        .channels()
        .map(|channel| channel.unwrap_or_else(|| panic!("{source}: a missing channel")))
}

pub(super) fn assert_channels(source: &str, expected: [f64; 3]) {
    let actual = channels(source);
    for index in 0..3 {
        if actual[index].to_bits() != expected[index].to_bits() {
            problem(format!(
                "{source}: channel {index} is {:e} and not {:e}",
                actual[index], expected[index]
            ));
        }
    }
}

pub(super) fn assert_near(source: &str, expected: [f64; 3], relative: f64) {
    let actual = channels(source);
    for index in 0..3 {
        let tolerance = relative * expected[index].abs().max(f64::MIN_POSITIVE);
        let distance = (actual[index] - expected[index]).abs();
        // A NaN distance is never near.
        if distance.is_nan() || distance > tolerance {
            problem(format!(
                "{source}: channel {index} is {:e} and not within {relative} of {:e}",
                actual[index], expected[index]
            ));
        }
    }
}

pub(super) fn assert_channels_of(label: &str, color: &Color, expected: [f64; 3]) {
    for (index, channel) in color.channels().into_iter().enumerate() {
        assert_channel(
            &format!("{label}[{index}]"),
            channel.unwrap_or(f64::NAN),
            expected[index],
        );
    }
}

pub(super) fn assert_channel(label: &str, actual: f64, expected: f64) {
    if actual.to_bits() != expected.to_bits() {
        problem(format!("{label}: {actual:e} and not {expected:e}"));
    }
}

pub(super) fn assert_text(source: &str, expected: impl AsRef<str>) {
    let actual = match color(source).serialize_specified(Default::default()) {
        Ok(text) => text,
        Err(error) => format!("<{error:?}>"),
    };
    if actual != expected.as_ref() {
        problem(format!(
            "{source}: {actual:?} and not {:?}",
            expected.as_ref()
        ));
    }
}

// ---------------------------------------------------------------------------
// Used resolution
// ---------------------------------------------------------------------------

#[test]
fn unbounded_components_beyond_the_float_range_resolve_to_the_largest_float() {
    let _report = Report::new();
    // Values 4 §10.12 (the closest value the implementation supports) with the
    // implementation range of a binary32 component; surgeist-color is unchanged,
    // so the channel is the finite ±f32::MAX.
    for (source, expected) in [
        ("lab(0 1e308 0)", [0.0, LARGEST, 0.0]),
        ("lab(0 -1e308 0)", [0.0, -LARGEST, 0.0]),
        ("lab(0 1e39 0)", [0.0, LARGEST, 0.0]),
        ("lab(0 1e41 0)", [0.0, LARGEST, 0.0]),
        ("lab(0 1e999 0)", [0.0, LARGEST, 0.0]),
        ("lab(0 -1e999 0)", [0.0, -LARGEST, 0.0]),
        ("lab(0 1e2000000 0)", [0.0, LARGEST, 0.0]),
        ("lab(0 0 1e308)", [0.0, 0.0, LARGEST]),
        ("lab(0 calc(infinity) 0)", [0.0, LARGEST, 0.0]),
        ("lab(0 calc(-infinity) 0)", [0.0, -LARGEST, 0.0]),
        ("lab(0 calc(1e308 * 10) 0)", [0.0, LARGEST, 0.0]),
        ("oklab(0.5 1e39 -1e39)", [0.5, LARGEST, -LARGEST]),
        ("color(srgb 1e308 -1e308 0)", [LARGEST, -LARGEST, 0.0]),
        ("color(xyz-d65 1e999 0 -1e999)", [LARGEST, 0.0, -LARGEST]),
        ("lch(50 1e999 0)", [50.0, LARGEST, 0.0]),
    ] {
        assert_channels(source, expected);
    }
}

#[test]
fn the_largest_float_and_the_values_below_it_keep_their_binary64_value() {
    let _report = Report::new();
    // Below the threshold a value keeps its binary64 value; the used channel is
    // that value limited to f32::MAX.
    for (source, expected) in [
        ("lab(0 3.4028233e38 0)", 3.4028233e38),
        ("lab(0 3.4028234e38 0)", 3.4028234e38),
        ("lab(0 340282346638528859811704183484516925440 0)", LARGEST),
        ("lab(0 3.4028234663852886e38 0)", LARGEST),
        ("lab(0 1e38 0)", 1e38),
        ("lab(0 -1e38 0)", -1e38),
        ("lab(0 9.999999999999999e40 0)", LARGEST),
    ] {
        assert_channels(source, [0.0, expected, 0.0]);
    }
}

#[test]
fn the_neighbours_of_the_threshold_resolve_alike_whether_or_not_they_are_beyond() {
    let _report = Report::new();
    // 3.4028235e38 and the integers around THRESHOLD lie above f32::MAX: the
    // used channel is f32::MAX for the values below the threshold and for the
    // infinite ones alike.
    for value in [
        "3.4028235e38",
        D,
        M_MINUS_ONE,
        M,
        THRESHOLD_MINUS_ONE,
        THRESHOLD,
    ] {
        assert_channels(&format!("lab(0 {value} 0)"), [0.0, LARGEST, 0.0]);
        assert_channels(&format!("lab(0 -{value} 0)"), [0.0, -LARGEST, 0.0]);
    }
}

#[test]
fn percentages_are_cut_after_the_slot_scaling() {
    let _report = Report::new();
    // The domain test applies to the scaled value: a Lab a/b axis is
    // 100% = 125, a color() channel 100% = 1 and an Oklab axis 100% = 0.4
    // (Color 4 §§9.3, 10.1, 11.1). The expected values are the binary64
    // `value / 100 * reference` that used scaling denotes.
    for (source, expected) in [
        ("lab(0 2.7e38% 0)", 3.375e38),
        ("lab(0 2.8e38% 0)", LARGEST),
        ("lab(0 -2.8e38% 0)", -LARGEST),
        ("lab(0 2.7222588e38% 0)", LARGEST),
        ("color(srgb 3.4e40% 0 0)", 3.4e38),
        ("color(srgb 1e41% 0 0)", LARGEST),
        ("oklab(0.5 8.5e40% 0)", 3.4e38),
        ("oklab(0.5 1e41% 0)", LARGEST),
        ("oklab(0.5 1e999% 0)", LARGEST),
    ] {
        let space = if source.starts_with("color") { 0 } else { 1 };
        assert_channel(source, channels(source)[space], expected);
    }
}

#[test]
fn an_infinite_hue_is_zero_and_a_finite_hue_keeps_its_binary64_modulo() {
    let _report = Report::new();
    // Color 4 §4.3 normalizes a hue to [0, 360) and an infinite hue to 0. A hue
    // is infinite when its binary64 degrees are at least THRESHOLD.
    for hue in [
        "1e308",
        "-1e308",
        "1e39",
        "-1e39",
        "1e41",
        "1e999",
        "-1e999",
        THRESHOLD,
        THRESHOLD_MINUS_ONE,
        M,
        "1e308deg",
        "1e40turn",
        "1e37turn",
        "-1e37turn",
        "4e38grad",
        "1e999turn",
        "1e999rad",
        "calc(1e308 * 10)",
        "calc(infinity)",
    ] {
        assert_channel(hue, channels(&format!("oklch(0.5 0.1 {hue})"))[2], 0.0);
    }
    // Below the threshold the binary64 degrees normalize as usual (Python:
    // `float(x) % 360.0`, and `float(9e35) * 360.0` for the turns).
    for (hue, expected) in [
        (D, 120.0),
        (M_MINUS_ONE, 120.0),
        ("3.4028235e38", 200.0),
        ("3.4028233e38", 24.0),
        ("3.3e38", 176.0),
        ("9e35turn", 200.0),
        ("1e30", 16.0),
    ] {
        assert_channel(hue, channels(&format!("oklch(0.5 0.1 {hue})"))[2], expected);
    }
    for source in [
        "hsl(1e308 100% 50%)",
        "hsl(-1e308 100% 50%)",
        "hsl(1e41 100% 50%)",
        "hsl(340282356779733661637539395458142568448 100% 50%)",
        "hsl(1e37turn 100% 50%)",
        "hwb(1e308 0% 0%)",
        "hwb(1e999grad 0% 0%)",
        "lch(from red l c 1e308)",
        "lch(from red l c 1e37turn)",
        "hsl(from red 1e308 s l)",
    ] {
        let color = resolved(source);
        match color.space() {
            ColorSpace::Predefined(PredefinedColorSpace::Srgb) => {
                // Hue 0 is red; both complete forms convert to sRGB.
                if !source.starts_with("lch") {
                    assert_channels_of(source, &color, [1.0, 0.0, 0.0]);
                }
            }
            ColorSpace::Lch => assert_channel(source, color.channels()[2].unwrap_or(f64::NAN), 0.0),
            other => panic!("{source}: {other:?}"),
        }
    }
    assert_channels(&format!("hsl({D} 100% 50%)"), [0.0, 1.0, 0.0]);
}

#[test]
fn a_component_numeral_below_1e_minus_46_is_zero_with_its_authored_sign() {
    let _report = Report::new();
    // Below half the smallest binary32 subnormal (2^-150 = 7.0e-46) a numeral
    // has no value in the component's range; it is the signed zero the exact
    // conversion of a rounded-to-zero nonzero numeral already gives.
    let negative_zero = -0.0_f64;
    for (source, expected) in [
        ("lab(0 1e-47 0)", 0.0),
        ("lab(0 -1e-47 0)", negative_zero),
        ("lab(0 9.99e-47 0)", 0.0),
        ("lab(0 5e-324 0)", 0.0),
        ("lab(0 -5e-324 0)", negative_zero),
        ("lab(0 2.2250738585072014e-308 0)", 0.0),
        ("lab(0 1e-47% 0)", 0.0),
        ("lab(0 -1e-47% 0)", negative_zero),
        (
            "lab(0 1e-999999999999999999999999999999999999999999 0)",
            0.0,
        ),
        (
            "lab(0 -1e-999999999999999999999999999999999999999999 0)",
            negative_zero,
        ),
        ("color(srgb 1e-50 -1e-50 0)", 0.0),
    ] {
        let actual = channels(source);
        let index = usize::from(!source.starts_with("color"));
        assert_channel(source, actual[index], expected);
    }
    // 1e-46 and the numerals above it keep their exact binary64 value.
    for (source, expected) in [
        ("lab(0 1e-46 0)", 1e-46),
        ("lab(0 -1e-46 0)", -1e-46),
        ("lab(0 1.1e-46 0)", 1.1e-46),
        ("lab(0 1e-45 0)", 1e-45),
        ("lab(0 1.401298464324817e-45 0)", 1.401298464324817e-45),
    ] {
        assert_channels(source, [0.0, expected, 0.0]);
    }
}

#[test]
fn clamped_slots_keep_their_endpoints() {
    let _report = Report::new();
    for (source, expected) in [
        ("rgb(1e999 -1e999 1e308)", [1.0, 0.0, 1.0]),
        ("rgb(1e39 -1e39 1e-47)", [1.0, 0.0, 0.0]),
        ("rgb(100% -1e999% 1e999%)", [1.0, 0.0, 1.0]),
        ("lab(1e999 0 0)", [100.0, 0.0, 0.0]),
        ("lab(-1e999 0 0)", [0.0, 0.0, 0.0]),
        ("lab(1e39% 0 0)", [100.0, 0.0, 0.0]),
        ("oklab(1e999 0 0)", [1.0, 0.0, 0.0]),
        ("oklch(0.5 -1e999 0)", [0.5, 0.0, 0.0]),
        ("lch(50 -1e39 0)", [50.0, 0.0, 0.0]),
    ] {
        assert_channels(source, expected);
    }
    for (source, alpha) in [
        ("rgb(0 0 0 / 1e39)", 1.0),
        ("rgb(0 0 0 / -1e39)", 0.0),
        ("rgb(0 0 0 / 1e999)", 1.0),
        ("color(srgb 0 0 0 / 1e999%)", 1.0),
        ("rgb(0 0 0 / 1e-47)", 0.0),
    ] {
        assert_channel(source, resolved(source).alpha().unwrap_or(f64::NAN), alpha);
    }
}

#[test]
fn mix_weights_convert_like_calculation_leaves() {
    let _report = Report::new();
    // A weight is a percentage in [0%, 100%] (Color 5 §3.3), so it can only lie
    // below the window. It is not a component root, so it converts as a
    // calculation leaf does: exactly inside the window and by the estimate below
    // 1e-46, never as zero or as the largest value. The expectations apply the
    // Color 5 weights (weight / total, then the second weight over the running
    // total, the first channel 1 - progress and the second the progress, and the
    // alpha multiplier total / 100) to the leaf conversions in binary64, with the
    // estimates from the independent model (Python):
    //
    // | Weights | Estimate | Correctly rounded leaves |
    // | --- | --- | --- |
    // | `3e-60%`, `7e-60%` | 0.29999999999999993, 0.7000000000000001 | 0.30000000000000004, 0.7 |
    // | `1e-47%`, `3e-47%` | 0.25, 0.75 | 0.2499999999999999, 0.7500000000000001 |
    for (source, red, blue, alpha) in [
        // Below the window: the estimates differ from the rounded leaves.
        (
            "color-mix(in srgb, red 3e-60%, blue 7e-60%)",
            0.29999999999999993,
            0.7000000000000001,
            1e-61,
        ),
        (
            "color-mix(in srgb, red 1e-47%, blue 3e-47%)",
            0.25,
            0.75,
            4e-49,
        ),
        // Estimates that equal the rounded leaves.
        (
            "color-mix(in srgb, red 1e-47%, blue 1e-46%)",
            0.09090909090909083,
            0.9090909090909092,
            1.1e-48,
        ),
        (
            "color-mix(in srgb, red 1e-300%, blue 1e-299%)",
            0.09090909090909083,
            0.9090909090909092,
            1.1e-301,
        ),
        // Inside the window the leaves are exact, and unchanged.
        (
            "color-mix(in srgb, red 1e-46%, blue 3e-46%)",
            0.25,
            0.75,
            4e-48,
        ),
        (
            "color-mix(in srgb, red 1e-45%, blue 3e-45%)",
            0.2499999999999999,
            0.7500000000000001,
            4e-47,
        ),
        ("color-mix(in srgb, red 25%, blue 75%)", 0.25, 0.75, 1.0),
    ] {
        assert_channels(source, [red, 0.0, blue]);
        assert_channel(source, resolved(source).alpha().unwrap_or(f64::NAN), alpha);
    }
}

#[test]
fn a_mix_of_a_component_beyond_the_range_converts_the_largest_float() {
    let _report = Report::new();
    // lab(0 f32::MAX 0) has Y = Z = 0, so XYZ is (X, 0, 0) with X ~ 3e107 and
    // the sRGB channels are about (1.03e45, -6.37e44, 2.15e44). The expectation
    // is that conversion evaluated independently in binary64 (Color 4 §10.3
    // D50 Lab, Bradford adaptation, sRGB matrix and transfer function), mixed
    // with red and then blue at equal weights.
    assert_near(
        "color-mix(in srgb, lab(0 1e308 0), red)",
        [
            5.169468752860828e44,
            -3.183111155186853e44,
            1.0727698315012536e44,
        ],
        1e-6,
    );
    assert_near(
        "color-mix(in srgb, color-mix(in srgb, lab(0 1e308 0), red), blue)",
        [
            2.584734376430414e44,
            -1.5915555775934266e44,
            5.363849157506268e43,
        ],
        1e-6,
    );
}

// ---------------------------------------------------------------------------
// Calculations: the root is cut, leaves are estimates, intermediates are exact
// ---------------------------------------------------------------------------

#[test]
fn a_calculation_root_beyond_the_threshold_is_the_largest_float() {
    let _report = Report::new();
    for (source, expected) in [
        ("lab(0 calc(3.5e38 * 1) 0)", LARGEST),
        ("lab(0 calc(-3.5e38 * 1) 0)", -LARGEST),
        ("lab(0 calc(3.4e38 * 1) 0)", 3.4e38),
        ("lab(0 calc(1e39% / 1) 0)", LARGEST),
        ("lab(0 calc(1e308 * 1e308) 0)", LARGEST),
    ] {
        assert_channels(source, [0.0, expected, 0.0]);
    }
}

#[test]
fn a_calculation_leaf_outside_the_window_is_an_estimate_and_intermediates_are_exact() {
    let _report = Report::new();
    // The leaves 1e308 and 1e-300 are binary64 values the model reproduces
    // exactly, and their product is exact: 1e8.
    assert_channels("lab(0 calc(1e308 * 1e-300) 0)", [0.0, 1e8, 0.0]);
    // Leaves inside the window are exact: 1e39 / 1e30 is the binary64 quotient
    // of the two correctly rounded values (Python: 999999999.9999999).
    assert_channels("lab(0 calc(1e39 / 1e30) 0)", [0.0, 999999999.9999999, 0.0]);
    // A leaf above the window multiplies back down: the high-end leaf below has
    // an estimate two encodings above the correctly rounded numeral, and the
    // quotient is the binary64 quotient of the estimates (Python, from the
    // estimate model: 99606795660663.31, bits 0x42d6a5e0eb0b5dd4).
    assert_channels(
        "lab(0 calc(9960679566066331e58 / 1e60) 0)",
        [0.0, f64::from_bits(0x42d6_a5e0_eb0b_5dd4), 0.0],
    );
    // The low-end leaf's estimate is two encodings below the correctly rounded
    // 9.997510968117049e-272 (0x07ab0abd121ff449); the product with the leaf
    // 1e280 is 999751096.8117048 (bits 0x41cdcb7edc67e5f1), where exact leaves
    // would give 999751096.811705 (bits 0x41cdcb7edc67e5f3).
    assert_channels(
        "lab(0 calc(9997510968117049e-287 * 1e280) 0)",
        [0.0, f64::from_bits(0x41cd_cb7e_dc67_e5f1), 0.0],
    );
    // A leaf below the window is not zero inside a calculation: it is the
    // estimate, and the product with its inverse leaf is 1.
    assert_channels("lab(0 calc(1e-47 * 1e47) 0)", [0.0, 1.0, 0.0]);
    // An intermediate beyond the threshold is not cut: 1e38 * 1e3 is about 1e41
    // and the quotient by 1e3 is the binary64 quotient of the binary64 product.
    assert_channels(
        "lab(0 calc(1e38 * 1e3 / 1e3) 0)",
        [0.0, (1e38_f64 * 1e3_f64) / 1e3_f64, 0.0],
    );
}

#[test]
fn a_literal_beyond_binary64_in_a_calculation_is_the_closest_finite_value() {
    let _report = Report::new();
    // Values 4 §10.12: the leaf is f64::MAX; min() keeps it, and a quotient by
    // the same leaf is 1.
    assert_channels("lab(0 calc(1e999 / 1e999) 0)", [0.0, 1.0, 0.0]);
    assert_channels("lab(0 calc(min(1e999, 3e38)) 0)", [0.0, 3e38, 0.0]);
}

#[test]
fn relative_colours_cut_the_component_that_uses_a_channel_not_the_channel() {
    let _report = Report::new();
    // The origin channel is the resolved binary64 value; the component that
    // uses it is cut at its own root.
    assert_channels("lab(from lab(0 1e39 0) l a b)", [0.0, LARGEST, 0.0]);
    assert_channels(
        "lab(from lab(0 1e30 0) l calc(a * 1e10) b)",
        [0.0, LARGEST, 0.0],
    );
    // The product is the binary64 product of the origin channel and the leaf: all
    // operands are inside the window, so there is no cut and no estimate.
    assert_channels(
        "lab(from lab(0 1e30 0) l calc(a * 1e3) b)",
        [0.0, 1e30_f64 * 1e3_f64, 0.0],
    );
    assert_channels("lab(from lab(0 1e-47 0) l a b)", [0.0, 0.0, 0.0]);
    // 1e-46 is exact; the leaf 1e45 is above 1e41 and a bounded-error estimate.
    assert_near(
        "lab(from lab(0 1e-46 0) l calc(a * 1e45) b)",
        [0.0, 0.1, 0.0],
        1e-14,
    );
}

// ---------------------------------------------------------------------------
// Specified text
// ---------------------------------------------------------------------------

#[test]
fn a_specified_component_beyond_the_float_range_is_calc_infinity() {
    let _report = Report::new();
    // Values 4 §10.12 serializes an infinite calculation as `calc(infinity)`;
    // WebKit writes the same text for these components.
    for (source, expected) in [
        ("lab(0 1e308 0)", "lab(0 calc(infinity) 0)"),
        ("lab(0 -1e308 0)", "lab(0 calc(-infinity) 0)"),
        ("lab(0 1e39 0)", "lab(0 calc(infinity) 0)"),
        ("lab(0 1e41 0)", "lab(0 calc(infinity) 0)"),
        ("lab(0 1e999 0)", "lab(0 calc(infinity) 0)"),
        ("lab(0 -1e999 0)", "lab(0 calc(-infinity) 0)"),
        ("lab(0 0 1e999)", "lab(0 0 calc(infinity))"),
        // Specified text is not clamped (Values 4 section 10.12), so an ink beyond the
        // range is an infinity like any unclamped component.
        (
            "device-cmyk(1e999 0 0 0)",
            "device-cmyk(calc(infinity) 0 0 0)",
        ),
        (
            "device-cmyk(0.5 -1e999 0 0)",
            "device-cmyk(0.5 calc(-infinity) 0 0)",
        ),
        ("lab(50 1e2000000 0)", "lab(50 calc(infinity) 0)"),
        (
            "lab(50 1e999999999999999999999999999999999999999999 0)",
            "lab(50 calc(infinity) 0)",
        ),
        ("lch(50 1e2000000 30)", "lch(50 calc(infinity) 30)"),
        ("lch(50 -1e999 30)", "lch(50 0 30)"),
        (
            "oklab(0.5 1e39 -1e39)",
            "oklab(0.5 calc(infinity) calc(-infinity))",
        ),
        (
            "color(srgb 1e308 -1e308 0)",
            "color(srgb calc(infinity) calc(-infinity) 0)",
        ),
        ("lab(0 calc(infinity) 0)", "lab(0 calc(infinity) 0)"),
        ("lab(0 calc(1e308 * 10) 0)", "lab(0 calc(infinity) 0)"),
        ("lab(0 calc(3.5e38 * 1) 0)", "lab(0 calc(infinity) 0)"),
        ("lab(0 calc(-3.5e38 * 1) 0)", "lab(0 calc(-infinity) 0)"),
        ("lab(0 calc(2.8e38% * 1) 0)", "lab(0 calc(infinity) 0)"),
    ] {
        assert_text(source, expected);
    }
}

#[test]
fn specified_and_used_agree_on_which_values_are_beyond_the_range() {
    let _report = Report::new();
    // The classification is the binary64 value of the numeral. M rounds to
    // THRESHOLD (a tie to the even mantissa) and THRESHOLD - 1 rounds to it, so
    // both are infinite; M - 1 rounds to D, below the threshold, and its exact
    // digits are written.
    for value in [M, THRESHOLD_MINUS_ONE, THRESHOLD] {
        assert_text(&format!("lab(0 {value} 0)"), "lab(0 calc(infinity) 0)");
        assert_text(&format!("lab(0 -{value} 0)"), "lab(0 calc(-infinity) 0)");
    }
    for value in [D, M_MINUS_ONE, "3.4028235e38"] {
        let digits = match value {
            "3.4028235e38" => "340282350000000000000000000000000000000",
            other => other,
        };
        assert_text(&format!("lab(0 {value} 0)"), format!("lab(0 {digits} 0)"));
    }
    // Percentages: the scaled binary64 value decides. 2.7222588e38% of a Lab
    // axis is 3.4028235e38 exactly, between f32::MAX and THRESHOLD.
    for (source, expected) in [
        (
            "lab(0 2.7222588e38% 0)",
            "lab(0 340282350000000000000000000000000000000 0)",
        ),
        ("lab(0 2.8e38% 0)", "lab(0 calc(infinity) 0)"),
        (
            "color(srgb 3.4e40% 0 0)",
            "color(srgb 340000000000000000000000000000000000000 0 0)",
        ),
        ("color(srgb 1e41% 0 0)", "color(srgb calc(infinity) 0 0)"),
        (
            "oklab(0.5 8.5e40% 0)",
            "oklab(0.5 340000000000000000000000000000000000000 0)",
        ),
        ("oklab(0.5 1e41% 0)", "oklab(0.5 calc(infinity) 0)"),
    ] {
        assert_text(source, expected);
    }
}

#[test]
fn a_specified_hue_beyond_the_range_is_zero_in_both_phases() {
    let _report = Report::new();
    for (source, expected) in [
        ("oklch(0.5 0.1 1e308)", "oklch(0.5 0.1 0)"),
        ("oklch(0.5 0.1 -1e308)", "oklch(0.5 0.1 0)"),
        ("oklch(0.5 0.1 1e39)", "oklch(0.5 0.1 0)"),
        ("oklch(0.5 0.1 1e41)", "oklch(0.5 0.1 0)"),
        ("oklch(0.5 0.1 1e999)", "oklch(0.5 0.1 0)"),
        ("lch(50 20 1e308deg)", "lch(50 20 0)"),
        ("lch(50 20 1e40turn)", "lch(50 20 0)"),
        ("lch(50 20 1e999turn)", "lch(50 20 0)"),
        ("lch(50 20 4e38grad)", "lch(50 20 0)"),
        (
            "lch(50 20 340282356779733661637539395458142568448)",
            "lch(50 20 0)",
        ),
        (
            "lch(50 20 340282356779733642748073463979561713664)",
            "lch(50 20 0)",
        ),
        ("lch(50 20 calc(1e308 * 10))", "lch(50 20 calc(infinity))"),
        ("hsl(1e308 100% 50%)", "rgb(255, 0, 0)"),
        ("hwb(1e999grad 0% 0%)", "rgb(255, 0, 0)"),
    ] {
        assert_text(source, expected);
    }
    // Inside the range the specified hue is the exact decimal modulo (the used
    // hue is the binary64 modulo; the two agree only where binary64 is exact).
    for (source, expected) in [
        (format!("lch(50 20 {D})"), "lch(50 20 120)"),
        (format!("lch(50 20 {M_MINUS_ONE})"), "lch(50 20 303)"),
        ("lch(50 20 3.3e38)".to_owned(), "lch(50 20 240)"),
        ("lch(50 20 1e30)".to_owned(), "lch(50 20 280)"),
    ] {
        assert_text(&source, expected);
    }
    assert_text(&format!("hsl({D} 100% 50%)"), "rgb(0, 255, 0)");
}

#[test]
fn a_specified_component_numeral_below_1e_minus_46_is_zero() {
    let _report = Report::new();
    for (source, expected) in [
        ("lab(0 1e-47 0)", "lab(0 0 0)".to_owned()),
        ("lab(0 -1e-47 0)", "lab(0 0 0)".to_owned()),
        ("lab(0 5e-324 0)", "lab(0 0 0)".to_owned()),
        ("lab(0 1e-47% 0)", "lab(0 0 0)".to_owned()),
        ("rgb(1e-47 0 0)", "rgb(0, 0, 0)".to_owned()),
        ("rgb(1e-47% 0 0)", "rgb(0, 0, 0)".to_owned()),
        (
            "device-cmyk(1e-50 0 0 0)",
            "device-cmyk(0 0 0 0)".to_owned(),
        ),
        ("hsl(none 1e-47% 50%)", "hsl(none 0% 50%)".to_owned()),
        (
            "lab(1e-999999999999999999999999999999999999999999 0 0)",
            "lab(0 0 0)".to_owned(),
        ),
        (
            "lab(0 1e-999999999999999999999999999999999999999999 0)",
            "lab(0 0 0)".to_owned(),
        ),
        ("lab(0 1e-46 0)", format!("lab(0 0.{}1 0)", "0".repeat(45))),
    ] {
        assert_text(source, expected);
    }
}

#[test]
fn specified_clamped_slots_and_missing_channel_text_keep_their_forms() {
    let _report = Report::new();
    for (source, expected) in [
        ("rgb(1e999 -1e999 1e308)", "rgb(255, 0, 255)"),
        ("lab(1e999 0 0)", "lab(100 0 0)"),
        ("lab(-1e999 0 0)", "lab(0 0 0)"),
        ("rgb(0 0 0 / 1e999)", "rgb(0, 0, 0)"),
        ("rgb(0 0 0 / -1e999)", "rgba(0, 0, 0, 0)"),
        // A percentage-typed slot of a colour with a missing channel writes the
        // percentage form of the infinity (the existing slot-typed mapping).
        ("hsl(none 1e308% 50%)", "hsl(none calc(infinity * 1%) 50%)"),
        ("hsl(none 1e999 50%)", "hsl(none calc(infinity * 1%) 50%)"),
        (
            "hwb(none -1e308% 50%)",
            "hwb(none calc(-infinity * 1%) 50%)",
        ),
        ("hwb(none 1e39 50%)", "hwb(none calc(infinity * 1%) 50%)"),
    ] {
        assert_text(source, expected);
    }
}

#[test]
fn a_specified_mix_writes_calc_infinity_for_its_children() {
    let _report = Report::new();
    for (source, expected) in [
        (
            "color-mix(in lab, lab(0 1e308 0), red)",
            "color-mix(in lab, lab(0 calc(infinity) 0), red)",
        ),
        (
            "color-mix(in srgb, lab(50 1e2000000 0), blue 30%)",
            "color-mix(in srgb, lab(50 calc(infinity) 0) 70%, blue 30%)",
        ),
        (
            "color-mix(in lab, lch(50 1e999 1e308), red)",
            "color-mix(in lab, lch(50 calc(infinity) 0), red)",
        ),
    ] {
        assert_text(source, expected);
    }
}

#[test]
fn authored_syntax_keeps_its_exact_numeral() {
    let _report = Report::new();
    // Retained authored values are not cut: `representation()` and `==` of a
    // parsed colour keep the authored numeral whatever its range.
    for numeral in [
        "1e999",
        "1e308",
        "1e-47",
        "340282356779733661637539395458142568448",
    ] {
        let source = format!("lab(0 {numeral} 0)");
        let value = color(&source);
        let Some(lab) = value.lab_value() else {
            panic!("{source}: a lab colour")
        };
        let CssColorComponent::Number(literal) = lab.a() else {
            panic!("{source}: a number literal")
        };
        assert_eq!(literal.numeric().representation(), numeral);
        assert_eq!(value, color(&source));
    }
    assert_ne!(color("lab(0 1e999 0)"), color("lab(0 1e998 0)"));
    assert_ne!(color("lab(0 1e-47 0)"), color("lab(0 0 0)"));
}

/// The leaf `LEAF` is beyond the window (8.4e42), so used resolution converts it
/// to the model's estimate, two encodings above its correctly rounded value. The
/// factors below were found by searching, with the independent model in binary64
/// (Python), for multipliers where the two conversions give roots that the
/// output shows.
const LEAF: &str = "838206077758223174e25";
/// `LEAF * THRESHOLD_FACTOR` is `THRESHOLD` with the estimate and `D` (the
/// binary64 below it) with the correctly rounded leaf.
const THRESHOLD_FACTOR: &str = "4.059650315228167e-05";
/// `LEAF * TEXT_FACTOR` is 971289280365.6613 with the estimate and
/// 971289280365.661 with the correctly rounded leaf, which the text shows.
const TEXT_FACTOR: &str = "1.15877145983404e-31";

#[test]
fn specified_calc_leaves_convert_as_used_resolution_does() {
    let _report = Report::new();
    // The premise: the products of the two conversions are different binary64
    // values (the literals are the shortest round-trip spellings, from Python).
    assert_ne!(
        971289280365.6613_f64.to_bits(),
        971289280365.661_f64.to_bits()
    );
    assert_eq!(
        3.4028235677973366e38_f64,
        f64::from(f32::MAX) + 2f64.powi(103)
    );
    assert_eq!(
        3.4028235677973362e38_f64.to_bits(),
        (f64::from(f32::MAX) + 2f64.powi(103) - 2f64.powi(75)).to_bits()
    );
    // A root on different sides of the threshold: the estimate makes the hue
    // infinite, so it is 0 in the used colour and in the specified text.
    let hue = format!("calc({LEAF} * {THRESHOLD_FACTOR})");
    assert_channel(&hue, channels(&format!("lch(50 20 {hue})"))[2], 0.0);
    assert_text(&format!("lch(50 20 {hue})"), "lch(50 20 calc(infinity))");
    assert_channels(&format!("hsl({hue} 100% 50%)"), [1.0, 0.0, 0.0]);
    assert_text(&format!("hsl({hue} 100% 50%)"), "rgb(255, 0, 0)");
    // A root whose text differs: a colour component calculation writes the
    // shortest round-trip text of its root, which is that of the root the used
    // channel holds (971289280365.6613, not 971289280365.661).
    let text = format!("calc({LEAF} * {TEXT_FACTOR})");
    assert_channels(&format!("lab(0 {text} 0)"), [0.0, 971289280365.6613, 0.0]);
    assert_text(
        &format!("lab(0 {text} 0)"),
        "lab(0 calc(971289280365.6613) 0)",
    );
    // A calculation that is not a colour component keeps the correctly rounded
    // leaf (971289280365.661011 is the six-place text of 971289280365.661).
    assert_eq!(
        generic_text(&format!("opacity:{text}")),
        "opacity: calc(971289280365.661011);"
    );
}

fn generic_text(declaration: &str) -> String {
    let report = surgeist_css::declaration::CssDeclaration::parse(
        &surgeist_css::CssParser::default(),
        declaration,
    );
    assert!(
        report.is_clean(),
        "{declaration}: {:?}",
        report.diagnostics()
    );
    report
        .syntax()
        .as_ref()
        .expect("retained declaration")
        .serialize_specified(Default::default())
        .unwrap()
}

#[test]
fn an_angle_leaf_beyond_binary64_is_the_nearest_multiple_of_360_in_both_phases() {
    let _report = Report::new();
    // Values 4 section 5: an angle beyond the supported range is the nearest
    // supported multiple of 360deg, whose cosine is 1. The leaf of a
    // calculation beyond binary64 is that endpoint in the used colour and in the
    // specified text, not the largest binary64 value in degrees.
    for source in [
        "rgb(calc(255 * cos(1e400deg)) 0 0)",
        "rgb(calc(255 * cos(-1e400deg)) 0 0)",
        "rgb(calc(255 * cos(1e308turn)) 0 0)",
        "rgb(calc(255 * cos(1e400grad)) 0 0)",
    ] {
        assert_channels(source, [1.0, 0.0, 0.0]);
        assert_text(source, "rgb(255, 0, 0)");
    }
}
