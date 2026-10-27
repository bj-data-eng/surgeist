//! Values 4 WD 2024-03-12 §6.2 fixes the absolute length units relative to
//! each other: 1in = 2.54cm = 96px, 1mm = 1/10th of 1cm, 1Q = 1/40th of 1cm,
//! 1pc = 1/6th of 1in and 1pt = 1/72nd of 1in. Equal absolute lengths in
//! different units are therefore mathematically equal inside math functions,
//! so their ratio is exactly 1 and their difference is exactly zero. Values 4 §7.2
//! and §7.4 do the same for 1s = 1000ms, 1dppx = 96dpi and 1dpcm = 2.54dpi, so
//! every rem(), mod(), round(), sign() and division over such operands is
//! arithmetic on exact rational multiples of the authored coefficients (#1433).

use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::numeric::{
    CssCalculation, CssLength, CssNumber, CssNumeric, CssResolution, CssTime,
};

/// Pairs of equal absolute lengths, each spelled in two different units.
const EQUAL_LENGTHS: [(&str, &str); 12] = [
    ("1cm", "10mm"),
    ("1cm", "40Q"),
    ("1mm", "4Q"),
    ("1in", "2.54cm"),
    ("1in", "25.4mm"),
    ("1in", "101.6Q"),
    ("1in", "96px"),
    ("1in", "72pt"),
    ("1in", "6pc"),
    ("1pc", "12pt"),
    ("3cm", "30mm"),
    ("7mm", "28Q"),
];

fn number(source: &str) -> String {
    CssNumeric::<CssNumber>::try_from_calculation(
        CssCalculation::<CssNumber>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            Default::default(),
        )
        .unwrap_or_else(|error| panic!("{source}: {error:?}")),
    )
    .unwrap()
    .serialize_specified(Default::default())
    .unwrap()
}

fn length(source: &str) -> String {
    CssNumeric::<CssLength>::try_from_calculation(
        CssCalculation::<CssLength>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            Default::default(),
        )
        .unwrap_or_else(|error| panic!("{source}: {error:?}")),
    )
    .unwrap()
    .serialize_specified(Default::default())
    .unwrap()
}

fn time(source: &str) -> String {
    CssNumeric::<CssTime>::try_from_calculation(
        CssCalculation::<CssTime>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            Default::default(),
        )
        .unwrap_or_else(|error| panic!("{source}: {error:?}")),
    )
    .unwrap()
    .serialize_specified(Default::default())
    .unwrap()
}

fn resolution(source: &str) -> String {
    CssNumeric::<CssResolution>::try_from_calculation(
        CssCalculation::<CssResolution>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            Default::default(),
        )
        .unwrap_or_else(|error| panic!("{source}: {error:?}")),
    )
    .unwrap()
    .serialize_specified(Default::default())
    .unwrap()
}

#[test]
fn ratio_of_equal_absolute_lengths_rounds_down_to_exactly_one() {
    for (left, right) in EQUAL_LENGTHS {
        for (a, b) in [(left, right), (right, left)] {
            assert_eq!(
                number(&format!("calc(round(down, {a} / {b}, 1))")),
                "calc(1)",
                "{a} / {b}"
            );
            assert_eq!(
                number(&format!("calc(round(up, {a} / {b}, 1))")),
                "calc(1)",
                "{a} / {b}"
            );
        }
    }
}

#[test]
fn sign_of_the_difference_of_equal_absolute_lengths_is_zero() {
    for (left, right) in EQUAL_LENGTHS {
        for (a, b) in [(left, right), (right, left)] {
            assert_eq!(
                number(&format!("calc(sign({a} - {b}))")),
                "calc(0)",
                "{a} - {b}"
            );
        }
    }
}

#[test]
fn difference_and_rounding_of_equal_absolute_lengths_are_exact() {
    for (left, right) in EQUAL_LENGTHS {
        for (a, b) in [(left, right), (right, left)] {
            assert_eq!(
                length(&format!("calc({a} - {b})")),
                "calc(0px)",
                "{a} - {b}"
            );
            // A length rounded to a step equal to itself is unchanged: the
            // canonical px value of `a`, which equals `b`.
            assert_eq!(
                length(&format!("calc(round(down, {a}, {b}) - {b})")),
                "calc(0px)",
                "round(down, {a}, {b})"
            );
        }
    }
}

#[test]
fn modulus_and_remainder_of_equal_absolute_lengths_are_zero() {
    for (left, right) in EQUAL_LENGTHS {
        for (a, b) in [(left, right), (right, left)] {
            for function in ["mod", "rem"] {
                assert_eq!(
                    length(&format!("calc({function}({a}, {b}))")),
                    "calc(0px)",
                    "{function}({a}, {b})"
                );
                assert_eq!(
                    number(&format!("calc(sign({function}({a}, {b})))")),
                    "calc(0)",
                    "sign({function}({a}, {b}))"
                );
            }
        }
    }
}

#[test]
fn issue_reproductions_project_the_exact_mathematical_result() {
    assert_eq!(number("calc(round(down, 1cm / 10mm, 1))"), "calc(1)");
    assert_eq!(number("calc(sign(10mm - 1cm))"), "calc(0)");
    assert_eq!(number("calc(sign(1in - 96px))"), "calc(0)");
    assert_eq!(length("calc(1cm - 10mm)"), "calc(0px)");
}

#[test]
fn issue_1433_remainders_are_exactly_zero() {
    // The correctly rounded canonical px values of `10mm` and `1mm` are not
    // exact multiples of each other, so a remainder taken after projection is
    // one divisor short of zero. Values 4 §10.3 defines rem() and mod() on the
    // operands' own values: 10mm is exactly ten of 1mm, 3mm three of them, and
    // 1cm = 10mm (§6.2).
    for source in [
        "calc(rem(10mm, 1mm))",
        "calc(rem(3mm, 1mm))",
        "calc(rem(1cm, 1mm))",
        "calc(mod(10mm, 1mm))",
    ] {
        assert_eq!(length(source), "calc(0px)", "{source}");
    }
    assert_eq!(number("calc(round(down, 3cm / 3cm, 1))"), "calc(1)");
}

/// Dimensions whose absolute units Values 4 defines by exact ratios (§6.2, §7.2
/// and §7.4), each with the canonical-unit spelling of an exact zero.
#[derive(Clone, Copy)]
enum Dimension {
    Length,
    Time,
    Resolution,
}

impl Dimension {
    fn zero(self) -> &'static str {
        match self {
            Self::Length => "calc(0px)",
            Self::Time => "calc(0s)",
            Self::Resolution => "calc(0dppx)",
        }
    }

    fn evaluate(self, source: &str) -> String {
        match self {
            Self::Length => length(source),
            Self::Time => time(source),
            Self::Resolution => resolution(source),
        }
    }
}

/// Every unit the issue names, each of which is a non-power-of-two multiple of
/// its dimension's canonical unit, or a power-of-ten fraction of it.
const UNITS: [(Dimension, &str); 8] = [
    (Dimension::Length, "mm"),
    (Dimension::Length, "Q"),
    (Dimension::Length, "cm"),
    (Dimension::Length, "pt"),
    (Dimension::Length, "in"),
    (Dimension::Length, "pc"),
    (Dimension::Time, "ms"),
    (Dimension::Resolution, "dpi"),
];

#[test]
fn same_unit_remainders_of_every_multiple_are_exactly_zero() {
    // n of a unit is exactly n times that unit, so both remainders are zero by
    // arithmetic on the operands' own coefficients, for every n.
    for (dimension, unit) in UNITS {
        for n in 1..=100 {
            for source in [
                format!("calc(rem({n}{unit}, 1{unit}))"),
                format!("calc(mod({n}{unit}, 1{unit}))"),
                format!("calc(rem({n}{unit}, {n}{unit}))"),
                format!("calc(mod({n}{unit}, {n}{unit}))"),
                format!("calc(rem({}{unit}, {n}{unit}))", n * 3),
                format!("calc(mod({}{unit}, {n}{unit}))", n * 3),
            ] {
                assert_eq!(dimension.evaluate(&source), dimension.zero(), "{source}");
            }
        }
    }
}

#[test]
fn same_unit_remainders_with_nonzero_results_are_exact() {
    // rem(7, 2) = 1, mod(-7, 2) = 1 (the sign of the divisor) and
    // rem(-7, 2) = -1, in the operands' own unit.
    for (dimension, unit) in UNITS {
        for source in [
            format!("calc(rem(7{unit}, 2{unit}) - 1{unit})"),
            format!("calc(mod(-7{unit}, 2{unit}) - 1{unit})"),
            format!("calc(rem(-7{unit}, 2{unit}) + 1{unit})"),
            format!("calc(mod(7{unit}, -2{unit}) + 1{unit})"),
            format!("calc(rem(1000{unit}, 7{unit}) - 6{unit})"),
        ] {
            assert_eq!(dimension.evaluate(&source), dimension.zero(), "{source}");
        }
        // Scaling both operands by n scales the result: rem(7n, 2n) = n.
        for n in 1..=60 {
            for source in [
                format!("calc(rem({}{unit}, {}{unit}) - {n}{unit})", 7 * n, 2 * n),
                format!("calc(mod({}{unit}, {}{unit}) - {n}{unit})", 7 * n, 2 * n),
            ] {
                assert_eq!(dimension.evaluate(&source), dimension.zero(), "{source}");
            }
        }
    }
}

#[test]
fn same_unit_division_is_exact() {
    // n of a unit divided by n of that unit is exactly one, and 3n by n is
    // exactly three, whatever the binary64 value of the unit.
    for (_, unit) in UNITS {
        for n in 1..=100 {
            for (source, expected) in [
                (format!("calc(round(down, {n}{unit} / {n}{unit}, 1))"), 1),
                (format!("calc(round(up, {n}{unit} / {n}{unit}, 1))"), 1),
                (
                    format!("calc(round(down, {}{unit} / {n}{unit}, 1))", n * 3),
                    3,
                ),
                (
                    format!("calc(round(up, {}{unit} / {n}{unit}, 1))", n * 3),
                    3,
                ),
            ] {
                assert_eq!(number(&source), format!("calc({expected})"), "{source}");
            }
            let source = format!("calc(sign({n}{unit} / {n}{unit} - 1))");
            assert_eq!(number(&source), "calc(0)", "{source}");
        }
    }
}

#[test]
fn same_unit_rounding_is_exact() {
    // Values 4 §10.3: 7 rounds down to 6 and up to 8 in steps of 2, nearest in
    // steps of 3 to 6, and 10 rounds up to 12 in steps of 3; -7 rounds toward
    // zero to -6.
    for (dimension, unit) in UNITS {
        for source in [
            format!("calc(round(down, 7{unit}, 2{unit}) - 6{unit})"),
            format!("calc(round(up, 7{unit}, 2{unit}) - 8{unit})"),
            format!("calc(round(nearest, 7{unit}, 3{unit}) - 6{unit})"),
            format!("calc(round(up, 10{unit}, 3{unit}) - 12{unit})"),
            format!("calc(round(to-zero, -7{unit}, 2{unit}) + 6{unit})"),
            format!("calc(round(down, 1000{unit}, 7{unit}) - 994{unit})"),
        ] {
            assert_eq!(dimension.evaluate(&source), dimension.zero(), "{source}");
        }
        // Scaling both operands by n scales the result: 7n in steps of 2n.
        for n in 1..=60 {
            for source in [
                format!(
                    "calc(round(down, {}{unit}, {}{unit}) - {}{unit})",
                    7 * n,
                    2 * n,
                    6 * n
                ),
                format!(
                    "calc(round(up, {}{unit}, {}{unit}) - {}{unit})",
                    7 * n,
                    2 * n,
                    8 * n
                ),
                format!(
                    "calc(round(to-zero, {}{unit}, {}{unit}) - {}{unit})",
                    7 * n,
                    2 * n,
                    6 * n
                ),
            ] {
                assert_eq!(dimension.evaluate(&source), dimension.zero(), "{source}");
            }
        }
    }
}

#[test]
fn sign_of_same_unit_differences_is_exact() {
    // Each triple has a = b + c, so a - b - c is exactly zero in the unit.
    for (_, unit) in UNITS {
        for (a, b, c) in [
            (3, 1, 2),
            (10, 3, 7),
            (7, 2, 5),
            (100, 33, 67),
            (1000, 1, 999),
        ] {
            for source in [
                format!("calc(sign({a}{unit} - {b}{unit} - {c}{unit}))"),
                format!("calc(sign({b}{unit} + {c}{unit} - {a}{unit}))"),
                format!("calc(sign({b}{unit} - {a}{unit} + {c}{unit}))"),
            ] {
                assert_eq!(number(&source), "calc(0)", "{source}");
            }
        }
        assert_eq!(
            number(&format!("calc(sign(5{unit} - 2{unit} - 2{unit}))")),
            "calc(1)"
        );
        assert_eq!(
            number(&format!("calc(sign(2{unit} - 5{unit} + 2{unit}))")),
            "calc(-1)"
        );
    }
}

#[test]
fn cross_unit_operands_with_exact_rational_answers_are_exact() {
    // Values 4 fixes 1in = 2.54cm = 6pc = 72pt = 96px, 1cm = 10mm = 40Q,
    // 1s = 1000ms, 1dppx = 96dpi and 1dpcm = 2.54dpi, so each expectation below
    // is exact rational arithmetic on the authored coefficients.
    for (dimension, source, expected) in [
        (Dimension::Length, "calc(rem(1cm, 1mm))", "calc(0px)"),
        (Dimension::Length, "calc(rem(3cm, 1mm))", "calc(0px)"),
        (Dimension::Length, "calc(rem(5cm, 5mm))", "calc(0px)"),
        (Dimension::Length, "calc(mod(1cm, 1Q))", "calc(0px)"),
        (Dimension::Length, "calc(rem(7in, 1pc))", "calc(0px)"),
        (Dimension::Length, "calc(rem(2in, 1pt))", "calc(0px)"),
        (Dimension::Length, "calc(rem(1pc, 1pt))", "calc(0px)"),
        (Dimension::Length, "calc(mod(1in, 96px))", "calc(0px)"),
        (Dimension::Length, "calc(rem(2.54cm, 1in))", "calc(0px)"),
        (Dimension::Length, "calc(rem(1in, 2.54cm))", "calc(0px)"),
        (Dimension::Length, "calc(rem(25.4mm, 1in))", "calc(0px)"),
        (Dimension::Length, "calc(rem(101.6Q, 1in))", "calc(0px)"),
        // rem(10mm, 3mm) = 1mm; mod(-10mm, 3mm) = 2mm; both across units.
        (Dimension::Length, "calc(rem(1cm, 3mm) - 1mm)", "calc(0px)"),
        (Dimension::Length, "calc(mod(-1cm, 3mm) - 2mm)", "calc(0px)"),
        (Dimension::Length, "calc(rem(1in, 7pt) - 2pt)", "calc(0px)"),
        (
            Dimension::Length,
            "calc(round(down, 1cm, 3mm) - 9mm)",
            "calc(0px)",
        ),
        (
            Dimension::Length,
            "calc(round(up, 1cm, 3mm) - 12mm)",
            "calc(0px)",
        ),
        (Dimension::Length, "calc(3cm - 30mm)", "calc(0px)"),
        (Dimension::Length, "calc(1in - 1pc - 5pc)", "calc(0px)"),
        (Dimension::Length, "calc(1in - 72pt)", "calc(0px)"),
        (Dimension::Length, "calc(2.54cm - 1in)", "calc(0px)"),
        (Dimension::Time, "calc(rem(1s, 1ms))", "calc(0s)"),
        (Dimension::Time, "calc(rem(2.5s, 500ms))", "calc(0s)"),
        (Dimension::Time, "calc(rem(1s, 3ms) - 1ms)", "calc(0s)"),
        (Dimension::Time, "calc(1s - 999ms - 1ms)", "calc(0s)"),
        (
            Dimension::Resolution,
            "calc(rem(1dppx, 1dpi))",
            "calc(0dppx)",
        ),
        (
            Dimension::Resolution,
            "calc(rem(2.54dpi, 1dpcm))",
            "calc(0dppx)",
        ),
        (Dimension::Resolution, "calc(1dppx - 96dpi)", "calc(0dppx)"),
        (
            Dimension::Resolution,
            "calc(rem(1dppx, 5dpi) - 1dpi)",
            "calc(0dppx)",
        ),
    ] {
        assert_eq!(dimension.evaluate(source), expected, "{source}");
    }
    for (source, expected) in [
        ("calc(round(down, 1cm / 10mm, 1))", "calc(1)"),
        ("calc(round(down, 30mm / 3cm, 1))", "calc(1)"),
        ("calc(round(down, 1in / 1pt, 1))", "calc(72)"),
        ("calc(round(up, 1in / 1pt, 1))", "calc(72)"),
        ("calc(round(down, 1cm / 1Q, 1))", "calc(40)"),
        ("calc(round(down, 1in / 6pc, 1))", "calc(1)"),
        ("calc(round(down, 1in / 2.54cm, 1))", "calc(1)"),
        ("calc(round(down, 1s / 1ms, 1))", "calc(1000)"),
        ("calc(round(up, 1s / 1ms, 1))", "calc(1000)"),
        ("calc(round(down, 1dppx / 1dpi, 1))", "calc(96)"),
        ("calc(round(up, 1dppx / 1dpi, 1))", "calc(96)"),
        ("calc(sign(1s - 1000ms))", "calc(0)"),
        ("calc(sign(1dppx - 96dpi))", "calc(0)"),
        ("calc(sign(1dpcm - 2.54dpi))", "calc(0)"),
        ("calc(sign(3cm - 30mm))", "calc(0)"),
        ("calc(sign(1in - 1pc - 5pc))", "calc(0)"),
    ] {
        assert_eq!(number(source), expected, "{source}");
    }
}

#[test]
fn coefficient_overflow_keeps_the_closest_supported_value_rule() {
    // #1297: a finite sum outside binary64 converts to f64::MAX with its sign
    // rather than an infinity, whichever unit the coefficients are authored in.
    let largest = format!("calc({:.0}px)", f64::MAX);
    for source in [
        "calc(1e308px + 1e308px)",
        "calc(1e308mm + 1e308mm)",
        "calc(1.7e308Q + 1.7e308Q)",
        "calc(1e308in + 1e308in)",
    ] {
        assert_eq!(length(source), largest, "{source}");
    }
}

#[test]
fn decimal_coefficients_keep_the_six_place_projection_of_their_exact_value() {
    // 0.1mm + 0.2mm = 0.3mm = 0.3 * 480/127px = 1.133858px, and 2 * 1mm is
    // 960/127px = 7.559055px, at six places.
    assert_eq!(length("calc(0.1mm + 0.2mm)"), "calc(1.133858px)");
    assert_eq!(length("calc(2 * 1mm)"), "calc(7.559055px)");
}

#[test]
fn px_and_pt_operands_combine_exactly_in_thirds_of_a_pixel() {
    // 1pt = 4/3px (Values 4 §6.2), so 96px = 72pt, 100px = 75pt and 4px = 3pt.
    // Converting pt to px by one binary64 division would round 4/3; the common
    // unit is 1/3px, with multipliers 3 (px) and 4 (pt).
    for (source, expected) in [
        ("calc(round(up, 96px, 1pt))", "calc(96px)"),
        ("calc(round(up, 96px, 2pt))", "calc(96px)"),
        ("calc(round(up, 100px, 1pt))", "calc(100px)"),
        ("calc(round(down, 100px, 1pt))", "calc(100px)"),
        // 75pt is halfway between 70pt and 80pt; nearest chooses the upper one.
        ("calc(round(nearest, 100px, 10pt))", "calc(106.666667px)"),
        ("calc(round(down, 100px, 10pt))", "calc(93.333333px)"),
        ("calc(mod(-4px, 1pt))", "calc(0px)"),
        ("calc(rem(-4px, 1pt))", "calc(0px)"),
        ("calc(mod(-4px, 0.5pt))", "calc(0px)"),
        ("calc(rem(96px, 7pt) - 2pt)", "calc(0px)"),
        ("calc(round(up, 96px, 1pt) - 72pt)", "calc(0px)"),
        ("calc(1px + 1pt - 7pt + 3pt)", "calc(-3px)"),
    ] {
        assert_eq!(length(source), expected, "{source}");
    }
    assert_eq!(number("calc(sign(96px - 72pt))"), "calc(0)");
    assert_eq!(number("calc(sign(4px - 3pt))"), "calc(0)");
    assert_eq!(number("calc(round(down, 96px / 1pt, 1))"), "calc(72)");
    assert_eq!(number("calc(round(up, 100px / 1pt, 1))"), "calc(75)");
}

/// An exact rational number of canonical units, for expectations that never
/// touch binary64: `numerator / denominator`, with a positive denominator.
#[derive(Clone, Copy)]
struct Rational(i128, i128);

impl Rational {
    fn new(numerator: i128, denominator: i128) -> Self {
        let sign = denominator.signum();
        let divisor = gcd(numerator.abs(), denominator.abs()).max(1);
        Self(sign * numerator / divisor, sign * denominator / divisor)
    }
    fn times(self, other: Self) -> Self {
        Self::new(self.0 * other.0, self.1 * other.1)
    }
    fn minus(self, other: Self) -> Self {
        Self::new(self.0 * other.1 - other.0 * self.1, self.1 * other.1)
    }
    fn divided_by(self, other: Self) -> Self {
        Self::new(self.0 * other.1, self.1 * other.0)
    }
    fn floor(self) -> i128 {
        self.0.div_euclid(self.1)
    }
    fn ceil(self) -> i128 {
        -(-self.0).div_euclid(self.1)
    }
    fn whole(value: i128) -> Self {
        Self(value, 1)
    }
    /// The six-place decimal text of this length, as specified output prints it.
    fn px(self) -> String {
        let scaled = Self::new(self.0.abs() * 1_000_000, self.1);
        // No tie occurs: denominators here are 1, 3 or 127.
        let micro = (scaled.0 * 2 + scaled.1).div_euclid(scaled.1 * 2);
        let sign = if self.0 < 0 && micro != 0 { "-" } else { "" };
        let (whole, fraction) = (micro / 1_000_000, micro % 1_000_000);
        if fraction == 0 {
            format!("calc({sign}{whole}px)")
        } else {
            let fraction = format!("{fraction:06}");
            format!("calc({sign}{whole}.{}px)", fraction.trim_end_matches('0'))
        }
    }
}

fn gcd(a: i128, b: i128) -> i128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Values 4 §6.2: each absolute length unit as an exact rational number of px.
const LENGTH_UNITS: [(&str, i128, i128); 7] = [
    ("px", 1, 1),
    ("pt", 4, 3),
    ("pc", 16, 1),
    ("in", 96, 1),
    ("cm", 4800, 127),
    ("mm", 480, 127),
    ("Q", 120, 127),
];

#[test]
fn every_pair_of_absolute_length_units_gives_the_exact_rational_remainder_and_rounding() {
    // Integer coefficients are exactly representable, so every expectation is
    // exact rational arithmetic on the unit ratios, whatever the pair of units.
    for (a_unit, a_n, a_d) in LENGTH_UNITS {
        for (b_unit, b_n, b_d) in LENGTH_UNITS {
            for x in [-13_i128, -4, 1, 7, 96, 100, 254] {
                for y in [-5_i128, -1, 1, 2, 3, 7, 10] {
                    let a = Rational::new(x * a_n, a_d);
                    let b = Rational::new(y * b_n, b_d);
                    let quotient = a.divided_by(b);
                    let truncated = if quotient.0 < 0 {
                        quotient.ceil()
                    } else {
                        quotient.floor()
                    };
                    let operands = format!("{x}{a_unit}, {y}{b_unit}");
                    let mut cases = vec![
                        (
                            format!("rem({operands})"),
                            a.minus(b.times(Rational::whole(truncated))),
                        ),
                        (
                            format!("mod({operands})"),
                            a.minus(b.times(Rational::whole(quotient.floor()))),
                        ),
                    ];
                    if y > 0 {
                        let half = Rational::new(2 * quotient.0 + quotient.1, 2 * quotient.1);
                        for (strategy, multiple) in [
                            ("down", quotient.floor()),
                            ("up", quotient.ceil()),
                            ("to-zero", truncated),
                            // Halfway chooses the upper multiple.
                            ("nearest", half.floor()),
                        ] {
                            cases.push((
                                format!("round({strategy}, {operands})"),
                                b.times(Rational::whole(multiple)),
                            ));
                        }
                    }
                    for (function, expected) in cases {
                        let source = format!("calc({function})");
                        assert_eq!(length(&source), expected.px(), "{source}");
                    }
                }
            }
        }
    }
}

#[test]
fn every_pair_of_absolute_length_units_gives_the_exact_rational_ratio_and_sign() {
    for (a_unit, a_n, a_d) in LENGTH_UNITS {
        for (b_unit, b_n, b_d) in LENGTH_UNITS {
            for x in [-13_i128, -4, 1, 7, 96, 100, 254] {
                for y in [-5_i128, -1, 1, 2, 3, 7, 10] {
                    let a = Rational::new(x * a_n, a_d);
                    let b = Rational::new(y * b_n, b_d);
                    let quotient = a.divided_by(b);
                    let source = format!("calc(sign({x}{a_unit} - {y}{b_unit}))");
                    let expected = a.minus(b).0.signum();
                    assert_eq!(number(&source), format!("calc({expected})"), "{source}");
                    for (strategy, multiple) in
                        [("down", quotient.floor()), ("up", quotient.ceil())]
                    {
                        let source =
                            format!("calc(round({strategy}, {x}{a_unit} / {y}{b_unit}, 1))");
                        assert_eq!(number(&source), format!("calc({multiple})"), "{source}");
                    }
                }
            }
        }
    }
}

#[test]
fn unit_homogeneous_functions_keep_the_source_of_their_result() {
    // min(), max(), abs() and clamp() return one of their operands exactly, but
    // the result keeps its authored unit, so a following remainder is exact.
    for source in [
        "calc(rem(min(10mm, 20mm), 1mm))",
        "calc(rem(max(10mm, 5mm), 1mm))",
        "calc(rem(abs(-30mm), 1mm))",
        "calc(rem(clamp(1mm, 70mm, 90mm), 1mm))",
        "calc(rem(min(3cm, 31mm), 1mm))",
    ] {
        assert_eq!(length(source), "calc(0px)", "{source}");
    }
}
