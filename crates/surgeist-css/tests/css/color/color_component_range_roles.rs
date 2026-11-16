//! The colour component range wherever a component appears (#2889): hues below
//! the window, relative overrides, profile expressions, calculated alpha and
//! origin colours, and the neighbours of the scaled threshold.
//!
//! Every colour component follows the rule in both phases wherever the colour
//! appears: a component whose binary64 value is at least `2^128 - 2^103` is
//! infinite (`calc(infinity)` in specified text, `f32::MAX` when used), and a
//! numeral below `1e-46` is zero. Every numeral that is not a component root
//! converts by the window, exactly inside it and by the estimate outside it.
//! An origin colour is a colour, so its components are written as the colour
//! standing alone writes them, without its clamps. Expectations come from that
//! rule and from independent binary64 and decimal arithmetic.

use super::color_component_range::{
    Report, assert_channel, assert_channels, assert_near, assert_text, channels, problem, resolved,
};
use crate::css::support::checked_color as color;
use surgeist_css::serialization::CssSerializationLimits;

#[test]
fn hues_below_the_window_are_zero_in_both_phases() {
    let _report = Report::new();
    // Color 4 §4.3 and the component rule: a hue numeral below 1e-46 is zero in
    // any unit, positive or negative, so it is written `0` and used as 0.
    for hue in [
        "1e-47",
        "-1e-47",
        "9.99e-47",
        "5e-324",
        "1e-47deg",
        "-1e-47deg",
        "1e-47turn",
        "-1e-47turn",
        "1e-47grad",
        "-1e-47grad",
        "1e-47rad",
        "-1e-47rad",
    ] {
        for (function, expected) in [
            ("lch(50 20 {})", "lch(50 20 0)"),
            ("oklch(0.5 0.1 {})", "oklch(0.5 0.1 0)"),
        ] {
            let source = function.replace("{}", hue);
            assert_text(&source, expected);
            assert_channel(&source, channels(&source)[2], 0.0);
        }
    }
    // A numeral inside the window keeps its exact digits (1e-46turn is 3.6e-44).
    assert_text(
        "oklch(0.5 0.1 1e-46turn)",
        format!("oklch(0.5 0.1 0.{}36)", "0".repeat(43)),
    );
    // A missing channel beside a tiny hue, and a hue inside a mix with an
    // interpolation method: the tiny hue is the same zero as the numeral 0.
    assert_text("hsl(1e-47turn none 50%)", "hsl(0 none 50%)");
    assert_text("hwb(-1e-47grad none 50%)", "hwb(0 none 50%)");
    for (tiny, zero) in [
        (
            "color-mix(in oklch longer hue, oklch(0.5 0.1 1e-47turn), oklch(0.5 0.1 90))",
            "color-mix(in oklch longer hue, oklch(0.5 0.1 0), oklch(0.5 0.1 90))",
        ),
        (
            "color-mix(in lch, lch(50 20 -1e-47deg), lch(50 20 350))",
            "color-mix(in lch, lch(50 20 0), lch(50 20 350))",
        ),
    ] {
        assert_text(tiny, zero);
        let (tiny_color, zero_color) = (resolved(tiny), resolved(zero));
        if !tiny_color.identical(zero_color) {
            problem(format!("{tiny}: {tiny_color:?} and not {zero_color:?}"));
        }
    }
}

/// `(source, specified text, whether the text resolves to the same colour)`.
const ROLES: &[(&str, &str, bool)] = &[
    // Relative overrides: numbers, percentages, angles and calculations.
    (
        "lab(from red l 1e308 b)",
        "lab(from red l calc(infinity) b)",
        true,
    ),
    (
        "lab(from red l -1e308 b)",
        "lab(from red l calc(-infinity) b)",
        true,
    ),
    (
        "lab(from red l 1e2000000 b)",
        "lab(from red l calc(infinity) b)",
        true,
    ),
    (
        "lab(from red l calc(1e2000000) b)",
        "lab(from red l calc(infinity) b)",
        true,
    ),
    (
        "rgb(from red 1e2000000 g b)",
        "rgb(from red calc(infinity) g b)",
        true,
    ),
    (
        "color(from red srgb r g 1e2000000)",
        "color(from red srgb r g calc(infinity))",
        true,
    ),
    (
        "lab(from red l 1e999% b)",
        "lab(from red l calc(infinity) b)",
        true,
    ),
    (
        "lab(from red l -1e999% b)",
        "lab(from red l calc(-infinity) b)",
        true,
    ),
    ("lab(from red 1e-47 a b)", "lab(from red 0 a b)", true),
    ("lab(from red l 1e-47% b)", "lab(from red l 0 b)", true),
    (
        "lab(from red l calc(1e-2000000) b)",
        "lab(from red l calc(0) b)",
        true,
    ),
    (
        "lch(from red l c 1e308)",
        "lch(from red l c calc(infinity))",
        true,
    ),
    (
        "lch(from red l c 1e308deg)",
        "lch(from red l c calc(infinity * 1deg))",
        true,
    ),
    (
        "lch(from red l c -1e308grad)",
        "lch(from red l c calc(-infinity * 1deg))",
        true,
    ),
    (
        "hsl(from red 1e40turn s l)",
        "hsl(from red calc(infinity * 1deg) s l)",
        true,
    ),
    (
        "lch(from red l c 1e-47turn)",
        "lch(from red l c 0deg)",
        true,
    ),
    // Calculated alpha: leaves convert by the window and the root is cut.
    (
        "lab(0 0 0 / calc(1e2000000))",
        "lab(0 0 0 / calc(infinity))",
        true,
    ),
    (
        "lab(from red l a b / calc(1e2000000))",
        "lab(from red l a b / calc(infinity))",
        true,
    ),
    // Profile expressions (a profile has no facts here, so only the text).
    (
        "color(from red --P 1e308 calc(1e2000000) 1e-47 / calc(1e2000000))",
        "color(from red --P calc(infinity) calc(infinity) 0 / calc(infinity))",
        false,
    ),
    (
        "color(from red --P 50% 1e41% -1e41%)",
        "color(from red --P 0.5 calc(infinity) calc(-infinity))",
        false,
    ),
    // Origin colours are written as the colour standing alone writes them,
    // without clamps: the origin keeps its unclamped authored domains.
    (
        "lab(from lab(0 1e308 0) l a b)",
        "lab(from lab(0 calc(infinity) 0) l a b)",
        true,
    ),
    (
        "lab(from lab(0 1e2000000 0) l a b)",
        "lab(from lab(0 calc(infinity) 0) l a b)",
        true,
    ),
    (
        "lab(from lab(0 calc(1e2000000) 0) l a b)",
        "lab(from lab(0 calc(infinity) 0) l a b)",
        true,
    ),
    (
        "lab(from lab(0 1e-47 0) l a b)",
        "lab(from lab(0 0 0) l a b)",
        true,
    ),
    (
        "lab(from lab(0 1e41% 0) l a b)",
        "lab(from lab(0 calc(infinity * 1%) 0) l a b)",
        true,
    ),
    (
        "lab(from lab(1e999 0 0) l a b)",
        "lab(from lab(calc(infinity) 0 0) l a b)",
        true,
    ),
    (
        "rgb(from rgb(1e999 0 0) r g b)",
        "rgb(from rgb(calc(infinity) 0 0) r g b)",
        true,
    ),
    (
        "hsl(from hsl(1e308 100% 50%) h s l)",
        "hsl(from hsl(calc(infinity) 100% 50%) h s l)",
        true,
    ),
    (
        "lch(from lch(50 20 1e308deg) l c h)",
        "lch(from lch(50 20 calc(infinity * 1deg)) l c h)",
        true,
    ),
    (
        "lch(from lch(50 20 1e-47turn) l c h)",
        "lch(from lch(50 20 0deg) l c h)",
        true,
    ),
    (
        "alpha(from lab(0 0 0 / 1e999))",
        "alpha(from lab(0 0 0 / calc(infinity)))",
        true,
    ),
    (
        "alpha(from lab(0 0 0 / 1e999%))",
        "alpha(from lab(0 0 0 / calc(infinity * 1%)))",
        true,
    ),
    (
        "lab(from lab(none 1e308 none) l a b)",
        "lab(from lab(none calc(infinity) none) l a b)",
        true,
    ),
    // Channel keywords, none and ordinary overrides are untouched.
    ("lab(from red l a b)", "lab(from red l a b)", true),
    ("lab(from red none a b)", "lab(from red none a b)", true),
    (
        "lab(from red l a b / alpha)",
        "lab(from red l a b / alpha)",
        true,
    ),
    (
        "lab(from red l 1e40 b)",
        "lab(from red l calc(infinity) b)",
        true,
    ),
];

#[test]
fn components_beside_their_slot_follow_the_rule_in_both_phases() {
    let _report = Report::new();
    for (source, text, resolves) in ROLES {
        assert_text(source, text);
        if *resolves {
            // The specified text resolves to the colour the source resolves to.
            let (original, written) = (resolved(source), resolved(text));
            if !original.identical(written) {
                problem(format!("{source}: {original:?} but {text} is {written:?}"));
            }
        }
    }
}

#[test]
fn former_byte_limit_cases_fit_a_small_budget() {
    let _report = Report::new();
    // Every one of these wrote its numeral's digits and failed with a byte limit;
    // the infinity form does not depend on the exponent.
    for source in [
        "lab(from red l 1e2000000 b)",
        "lab(from red l calc(1e2000000) b)",
        "rgb(from red 1e2000000 g b)",
        "color(from red srgb r g 1e2000000)",
        "lab(0 0 0 / calc(1e2000000))",
        "lab(from lab(0 1e2000000 0) l a b)",
        "alpha(from lab(0 0 0 / 1e2000000))",
        "color(from red --P 1e2000000 0 0)",
    ] {
        let limits = CssSerializationLimits::new(usize::MAX, usize::MAX, 128);
        if let Err(error) = color(source).serialize_specified(limits) {
            problem(format!("{source}: {error:?}"));
        }
    }
}

#[test]
fn the_estimate_reaches_calculated_alpha_and_overrides() {
    let _report = Report::new();
    // `LEAF` is beyond the window; its estimate and its correctly rounded value
    // give 971289280365.661255 and 971289280365.661011 as six-place text for the
    // factor below (the same pair the component test pins).
    for (source, text) in [
        (
            "lab(0 0 0 / calc(838206077758223174e25 * 1.15877145983404e-31))",
            "lab(0 0 0 / calc(971289280365.661255))",
        ),
        (
            "lab(from red l calc(838206077758223174e25 * 1.15877145983404e-31) b)",
            "lab(from red l calc(971289280365.661255) b)",
        ),
        (
            "lab(from lab(0 calc(838206077758223174e25 * 1.15877145983404e-31) 0) l a b)",
            "lab(from lab(0 calc(971289280365.661255) 0) l a b)",
        ),
    ] {
        assert_text(source, text);
    }
}

#[test]
fn the_neighbours_of_the_scaled_threshold_are_classified_alike_in_both_phases() {
    let _report = Report::new();
    // For each slot, the largest decimal whose binary64 scaled value (the models
    // `value / 100 * reference` for a percentage, `value * 360` and `value * 9 /
    // 10` for a turn and a gradian, evaluated in Python over every neighbouring
    // double) is below THRESHOLD, and the next one up, which is at or above it.
    // The below value writes the exact digits of its scaled decimal and the
    // above value writes calc(infinity); both resolve alike.
    for (source, text) in [
        (
            "lab(0 2.722258854237869e38% 0)",
            "lab(0 340282356779733625000000000000000000000 0)",
        ),
        ("lab(0 2.7222588542378694e38% 0)", "lab(0 calc(infinity) 0)"),
        (
            "oklab(0.5 8.507058919493341e40% 0)",
            "oklab(0.5 340282356779733640000000000000000000000 0)",
        ),
        (
            "oklab(0.5 8.507058919493342e40% 0)",
            "oklab(0.5 calc(infinity) 0)",
        ),
        (
            "color(srgb 3.402823567797336e40% 0 0)",
            "color(srgb 340282356779733600000000000000000000000 0 0)",
        ),
        (
            "color(srgb 3.4028235677973366e40% 0 0)",
            "color(srgb calc(infinity) 0 0)",
        ),
        // A turn: 9.452287688325934e35turn is 3.4028235677973362e38deg (hue 120)
        // and the next double up is at the threshold (infinite, hue 0).
        ("hsl(9.452287688325934e35turn 100% 50%)", "rgb(0, 255, 0)"),
        ("hsl(9.452287688325936e35turn 100% 50%)", "rgb(255, 0, 0)"),
        // A gradian: 3.780915075330373e38grad is 3.402823567797336e38deg (hue 112)
        // and the next double up is 3.4028235677973366e38deg (hue 0).
        ("hsl(3.780915075330373e38grad 100% 50%)", "rgb(34, 255, 0)"),
        ("hsl(3.780915075330374e38grad 100% 50%)", "rgb(255, 0, 0)"),
    ] {
        assert_text(source, text);
    }
    // Used: the percentages and the infinite hues resolve to the limit or zero.
    for source in [
        "lab(0 2.722258854237869e38% 0)",
        "lab(0 2.7222588542378694e38% 0)",
        "oklab(0.5 8.507058919493341e40% 0)",
        "oklab(0.5 8.507058919493342e40% 0)",
        "color(srgb 3.402823567797336e40% 0 0)",
        "color(srgb 3.4028235677973366e40% 0 0)",
    ] {
        let value = channels(source);
        let index = usize::from(source.starts_with("lab") || source.starts_with("oklab"));
        assert_channel(source, value[index], f64::from(f32::MAX));
    }
    assert_channels("hsl(9.452287688325934e35turn 100% 50%)", [0.0, 1.0, 0.0]);
    assert_channels("hsl(9.452287688325936e35turn 100% 50%)", [1.0, 0.0, 0.0]);
    assert_near(
        "hsl(3.780915075330373e38grad 100% 50%)",
        [0.1333333333333333, 1.0, 0.0],
        1e-12,
    );
    assert_channels("hsl(3.780915075330374e38grad 100% 50%)", [1.0, 0.0, 0.0]);
}

// ---------------------------------------------------------------------------
// A component is cut as the colour holds it
// ---------------------------------------------------------------------------

/// The largest value a used component has.
const LARGEST: f64 = f32::MAX as f64;

/// What a source resolves to: one channel, or the three channels of a colour
/// that is converted (relative `hsl()` holds an sRGB colour).
enum Used {
    Channel(usize, f64),
    Near([f64; 3]),
}

/// Checks each row in both phases: the specified text, the used value and that
/// the specified text resolves to the colour its source resolves to, which the
/// last cell of a row says it does (a percentage written as the exact decimal
/// of its scale can differ from the source in the last bit of the binary64).
fn check_held(rows: &[(&str, &str, Used, bool)]) {
    for (source, text, used, resolves) in rows {
        assert_text(source, text);
        match used {
            Used::Channel(index, expected) => {
                assert_channel(source, channels(source)[*index], *expected);
            }
            Used::Near(expected) => assert_near(source, *expected, 1e-12),
        }
        if *resolves {
            let (original, written) = (resolved(source), resolved(text));
            if !original.identical(written) {
                problem(format!("{source}: {original:?} but {text} is {written:?}"));
            }
        }
    }
}

// Each row's expectation is derived in Python from the rule alone. A component
// is infinite when its binary64 value as the colour holds it is at least
// 2^128 - 2^103: the percentage is scaled by its slot's reference (value / 100 *
// reference, with 100% = 0.4 for an Oklab axis, 125 for a Lab axis and 100 for
// HSL saturation), and an Oklab, Lab or HSL component is held in that unit. An
// override and an origin colour both write the text of an infinite component as
// the component standing alone does (`calc(infinity)`, and `calc(infinity * 1%)`
// where the percentage unit is kept), and write a finite one with its exact
// digits. The decimal each row names is, for a neighbour of the cut, the
// largest binary64 below it or the next one up (searched over every
// neighbouring double). A percentage is cut after its scale in each position:
// 1e40% of an Oklab axis is 4e37 and finite though 1e40 is beyond the cut, and
// 2.8e38% of a Lab axis is 3.5e38 and infinite though 2.8e38 is not; HSL
// saturation has 100% = 100, whose scale does not move any double across the
// cut (checked over the 300 doubles below and above it).
const PERCENTAGES_AS_HELD: &[(&str, &str, Used, bool)] = &[
    (
        "oklab(from red l 1e40% b)",
        "oklab(from red l 40000000000000000000000000000000000000 b)",
        Used::Channel(1, 4.0000000000000003e37),
        false,
    ),
    (
        "oklab(from red l calc(1e40% * 1) b)",
        "oklab(from red l calc(10000000000000000303786028427003666890752%) b)",
        Used::Channel(1, 4.0000000000000003e37),
        true,
    ),
    (
        "oklab(from oklab(0.5 1e40% 0) l a b)",
        "oklab(from oklab(0.5 10000000000000000000000000000000000000000% 0) l a b)",
        Used::Channel(1, 4.0000000000000003e37),
        true,
    ),
    (
        "oklab(from oklab(0.5 calc(1e40% * 1) 0) l a b)",
        "oklab(from oklab(0.5 calc(10000000000000000303786028427003666890752%) 0) l a b)",
        Used::Channel(1, 4.0000000000000003e37),
        true,
    ),
    (
        "oklab(from red l 8.507058919493341e40% b)",
        "oklab(from red l 340282356779733640000000000000000000000 b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "oklab(from red l calc(8.507058919493341e40% * 1) b)",
        "oklab(from red l calc(85070589194933405737978291947502244462592%) b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "oklab(from oklab(0.5 8.507058919493341e40% 0) l a b)",
        "oklab(from oklab(0.5 85070589194933410000000000000000000000000% 0) l a b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "oklab(from oklab(0.5 calc(8.507058919493341e40% * 1) 0) l a b)",
        "oklab(from oklab(0.5 calc(85070589194933405737978291947502244462592%) 0) l a b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "oklab(from red l 8.507058919493342e40% b)",
        "oklab(from red l calc(infinity) b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "oklab(from red l calc(8.507058919493342e40% * 1) b)",
        "oklab(from red l calc(infinity * 1%) b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "oklab(from oklab(0.5 8.507058919493342e40% 0) l a b)",
        "oklab(from oklab(0.5 calc(infinity * 1%) 0) l a b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "oklab(from oklab(0.5 calc(8.507058919493342e40% * 1) 0) l a b)",
        "oklab(from oklab(0.5 calc(infinity * 1%) 0) l a b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "lab(from red l 2.7e38% b)",
        "lab(from red l 337500000000000000000000000000000000000 b)",
        Used::Channel(1, 3.375e38),
        true,
    ),
    (
        "lab(from red l calc(2.7e38% * 1) b)",
        "lab(from red l calc(269999999999999995810733116479149965312%) b)",
        Used::Channel(1, 3.375e38),
        true,
    ),
    (
        "lab(from lab(0 2.7e38% 0) l a b)",
        "lab(from lab(0 270000000000000000000000000000000000000% 0) l a b)",
        Used::Channel(1, 3.375e38),
        true,
    ),
    (
        "lab(from lab(0 calc(2.7e38% * 1) 0) l a b)",
        "lab(from lab(0 calc(269999999999999995810733116479149965312%) 0) l a b)",
        Used::Channel(1, 3.375e38),
        true,
    ),
    (
        "lab(from red l 2.8e38% b)",
        "lab(from red l calc(infinity) b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "lab(from red l calc(2.8e38% * 1) b)",
        "lab(from red l calc(infinity * 1%) b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "lab(from lab(0 2.8e38% 0) l a b)",
        "lab(from lab(0 calc(infinity * 1%) 0) l a b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "lab(from lab(0 calc(2.8e38% * 1) 0) l a b)",
        "lab(from lab(0 calc(infinity * 1%) 0) l a b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "lab(from red l 2.722258854237869e38% b)",
        "lab(from red l 340282356779733625000000000000000000000 b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "lab(from red l calc(2.722258854237869e38% * 1) b)",
        "lab(from red l calc(272225885423786899086886026000784687104%) b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "lab(from lab(0 2.722258854237869e38% 0) l a b)",
        "lab(from lab(0 272225885423786900000000000000000000000% 0) l a b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "lab(from lab(0 calc(2.722258854237869e38% * 1) 0) l a b)",
        "lab(from lab(0 calc(272225885423786899086886026000784687104%) 0) l a b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "lab(from red l 2.7222588542378694e38% b)",
        "lab(from red l calc(infinity) b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "lab(from red l calc(2.7222588542378694e38% * 1) b)",
        "lab(from red l calc(infinity * 1%) b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "lab(from lab(0 2.7222588542378694e38% 0) l a b)",
        "lab(from lab(0 calc(infinity * 1%) 0) l a b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "lab(from lab(0 calc(2.7222588542378694e38% * 1) 0) l a b)",
        "lab(from lab(0 calc(infinity * 1%) 0) l a b)",
        Used::Channel(1, LARGEST),
        true,
    ),
    (
        "hsl(from red h 3.5e38% l)",
        "hsl(from red h calc(infinity) l)",
        Used::Near([
            1.7014117331926442e36,
            -1.7014117331926442e36,
            -1.7014117331926442e36,
        ]),
        true,
    ),
    (
        "hsl(from red h calc(3.5e38% * 1) l)",
        "hsl(from red h calc(infinity * 1%) l)",
        Used::Near([
            1.7014117331926442e36,
            -1.7014117331926442e36,
            -1.7014117331926442e36,
        ]),
        true,
    ),
    (
        "hsl(from hsl(0 3.5e38% 50%) h s l)",
        "hsl(from hsl(0 calc(infinity * 1%) 50%) h s l)",
        Used::Near([
            1.7014117331926442e36,
            -1.7014117331926442e36,
            -1.7014117331926442e36,
        ]),
        true,
    ),
    (
        "hsl(from hsl(0 calc(3.5e38% * 1) 50%) h s l)",
        "hsl(from hsl(0 calc(infinity * 1%) 50%) h s l)",
        Used::Near([
            1.7014117331926442e36,
            -1.7014117331926442e36,
            -1.7014117331926442e36,
        ]),
        true,
    ),
    (
        "hsl(from red h 3.4028235677973362e38% l)",
        "hsl(from red h 340282356779733620000000000000000000000 l)",
        Used::Near([
            1.7014117331926442e36,
            -1.7014117331926442e36,
            -1.7014117331926442e36,
        ]),
        true,
    ),
    (
        "hsl(from red h calc(3.4028235677973362e38% * 1) l)",
        "hsl(from red h calc(340282356779733623858607532500980858880%) l)",
        Used::Near([
            1.7014117331926442e36,
            -1.7014117331926442e36,
            -1.7014117331926442e36,
        ]),
        true,
    ),
    (
        "hsl(from hsl(0 3.4028235677973362e38% 50%) h s l)",
        "hsl(from hsl(0 340282356779733620000000000000000000000% 50%) h s l)",
        Used::Near([
            1.7014117331926442e36,
            -1.7014117331926442e36,
            -1.7014117331926442e36,
        ]),
        true,
    ),
    (
        "hsl(from hsl(0 calc(3.4028235677973362e38% * 1) 50%) h s l)",
        "hsl(from hsl(0 calc(340282356779733623858607532500980858880%) 50%) h s l)",
        Used::Near([
            1.7014117331926442e36,
            -1.7014117331926442e36,
            -1.7014117331926442e36,
        ]),
        true,
    ),
    (
        "hsl(from red h 3.4028235677973366e38% l)",
        "hsl(from red h calc(infinity) l)",
        Used::Near([
            1.7014117331926442e36,
            -1.7014117331926442e36,
            -1.7014117331926442e36,
        ]),
        true,
    ),
    (
        "hsl(from red h calc(3.4028235677973366e38% * 1) l)",
        "hsl(from red h calc(infinity * 1%) l)",
        Used::Near([
            1.7014117331926442e36,
            -1.7014117331926442e36,
            -1.7014117331926442e36,
        ]),
        true,
    ),
    (
        "hsl(from hsl(0 3.4028235677973366e38% 50%) h s l)",
        "hsl(from hsl(0 calc(infinity * 1%) 50%) h s l)",
        Used::Near([
            1.7014117331926442e36,
            -1.7014117331926442e36,
            -1.7014117331926442e36,
        ]),
        true,
    ),
    (
        "hsl(from hsl(0 calc(3.4028235677973366e38% * 1) 50%) h s l)",
        "hsl(from hsl(0 calc(infinity * 1%) 50%) h s l)",
        Used::Near([
            1.7014117331926442e36,
            -1.7014117331926442e36,
            -1.7014117331926442e36,
        ]),
        true,
    ),
];

#[test]
fn percentage_calculations_are_cut_after_the_scale_of_their_slot() {
    let _report = Report::new();
    check_held(PERCENTAGES_AS_HELD);
}

// An `rgb()` channel is held as the unit coordinate: a number divided by 255 and
// a percentage scaled to 255 and divided by 255, so a channel needs a value of at
// least 255 times the threshold to be infinite: 8.677200097883208e40 as a number
// and 3.4028235677973366e40 as a percentage are the first doubles that are. A
// numeral of 1e41 or more is infinite by the window of exact conversion. Below the
// cut the used value is the held one limited to f32::MAX (8.6772e40 and
// 3.40282350e40% are held between f32::MAX and the cut and are written with their
// digits). An origin `rgb()` colour holds its channel clamped to [0, 1], so its
// used value is 1.
const RGB_AS_HELD: &[(&str, &str, Used, bool)] = &[
    (
        "rgb(from red 5e38 g b)",
        "rgb(from red 500000000000000000000000000000000000000 g b)",
        Used::Channel(0, 1.96078431372549e36),
        true,
    ),
    (
        "rgb(from red calc(5e38 * 1) g b)",
        "rgb(from red calc(499999999999999969854583185801589293056) g b)",
        Used::Channel(0, 1.96078431372549e36),
        true,
    ),
    (
        "rgb(from rgb(5e38 0 0) r g b)",
        "rgb(from rgb(500000000000000000000000000000000000000 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from rgb(calc(5e38 * 1) 0 0) r g b)",
        "rgb(from rgb(calc(499999999999999969854583185801589293056) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from red 1e40 g b)",
        "rgb(from red 10000000000000000000000000000000000000000 g b)",
        Used::Channel(0, 3.9215686274509803e37),
        true,
    ),
    (
        "rgb(from red calc(1e40 * 1) g b)",
        "rgb(from red calc(10000000000000000303786028427003666890752) g b)",
        Used::Channel(0, 3.9215686274509803e37),
        true,
    ),
    (
        "rgb(from rgb(1e40 0 0) r g b)",
        "rgb(from rgb(10000000000000000000000000000000000000000 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from rgb(calc(1e40 * 1) 0 0) r g b)",
        "rgb(from rgb(calc(10000000000000000303786028427003666890752) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from red 8.6772e40 g b)",
        "rgb(from red 86772000000000000000000000000000000000000 g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from red calc(8.6772e40 * 1) g b)",
        "rgb(from red calc(86772000000000003138441696498636103352320) g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from rgb(8.6772e40 0 0) r g b)",
        "rgb(from rgb(86772000000000000000000000000000000000000 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from rgb(calc(8.6772e40 * 1) 0 0) r g b)",
        "rgb(from rgb(calc(86772000000000003138441696498636103352320) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from red 8.677200097883207e40 g b)",
        "rgb(from red 86772000978832070000000000000000000000000 g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from red calc(8.677200097883207e40 * 1) g b)",
        "rgb(from red calc(86772000978832074046165988924792957304832) g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from rgb(8.677200097883207e40 0 0) r g b)",
        "rgb(from rgb(86772000978832070000000000000000000000000 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from rgb(calc(8.677200097883207e40 * 1) 0 0) r g b)",
        "rgb(from rgb(calc(86772000978832074046165988924792957304832) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from red 8.677200097883208e40 g b)",
        "rgb(from red calc(infinity) g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from red calc(8.677200097883208e40 * 1) g b)",
        "rgb(from red calc(infinity) g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from rgb(8.677200097883208e40 0 0) r g b)",
        "rgb(from rgb(calc(infinity) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from rgb(calc(8.677200097883208e40 * 1) 0 0) r g b)",
        "rgb(from rgb(calc(infinity) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from red 1e41 g b)",
        "rgb(from red calc(infinity) g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from red calc(1e41 * 1) g b)",
        "rgb(from red calc(infinity) g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from rgb(1e41 0 0) r g b)",
        "rgb(from rgb(calc(infinity) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from rgb(calc(1e41 * 1) 0 0) r g b)",
        "rgb(from rgb(calc(infinity) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from red 1e39% g b)",
        "rgb(from red 2550000000000000000000000000000000000000 g b)",
        Used::Channel(0, 1e37),
        true,
    ),
    (
        "rgb(from red calc(1e39% * 1) g b)",
        "rgb(from red calc(999999999999999939709166371603178586112%) g b)",
        Used::Channel(0, 1e37),
        true,
    ),
    (
        "rgb(from rgb(1e39% 0 0) r g b)",
        "rgb(from rgb(1000000000000000000000000000000000000000% 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from rgb(calc(1e39% * 1) 0 0) r g b)",
        "rgb(from rgb(calc(999999999999999939709166371603178586112%) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from red 3.40282350e40% g b)",
        "rgb(from red 86771999250000000000000000000000000000000 g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from red calc(3.40282350e40% * 1) g b)",
        "rgb(from red calc(34028235000000000384404693992772443570176%) g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from rgb(3.40282350e40% 0 0) r g b)",
        "rgb(from rgb(34028235000000000000000000000000000000000% 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from rgb(calc(3.40282350e40% * 1) 0 0) r g b)",
        "rgb(from rgb(calc(34028235000000000384404693992772443570176%) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from red 3.402823567797336e40% g b)",
        "rgb(from red 86772000978832068000000000000000000000000 g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from red calc(3.402823567797336e40% * 1) g b)",
        "rgb(from red calc(34028235677973361328050661087297558020096%) g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from rgb(3.402823567797336e40% 0 0) r g b)",
        "rgb(from rgb(34028235677973360000000000000000000000000% 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from rgb(calc(3.402823567797336e40% * 1) 0 0) r g b)",
        "rgb(from rgb(calc(34028235677973361328050661087297558020096%) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from red 3.4028235677973366e40% g b)",
        "rgb(from red calc(infinity) g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from red calc(3.4028235677973366e40% * 1) g b)",
        "rgb(from red calc(infinity * 1%) g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from rgb(3.4028235677973366e40% 0 0) r g b)",
        "rgb(from rgb(calc(infinity * 1%) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from rgb(calc(3.4028235677973366e40% * 1) 0 0) r g b)",
        "rgb(from rgb(calc(infinity * 1%) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from red 1e41% g b)",
        "rgb(from red calc(infinity) g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from red calc(1e41% * 1) g b)",
        "rgb(from red calc(infinity * 1%) g b)",
        Used::Channel(0, LARGEST),
        true,
    ),
    (
        "rgb(from rgb(1e41% 0 0) r g b)",
        "rgb(from rgb(calc(infinity * 1%) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
    (
        "rgb(from rgb(calc(1e41% * 1) 0 0) r g b)",
        "rgb(from rgb(calc(infinity * 1%) 0 0) r g b)",
        Used::Channel(0, 1.0),
        true,
    ),
];

#[test]
fn rgb_components_are_cut_as_the_unit_coordinate_they_become() {
    let _report = Report::new();
    check_held(RGB_AS_HELD);
}
