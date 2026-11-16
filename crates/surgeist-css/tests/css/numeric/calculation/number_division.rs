//! Values 4 WD 2024-03-12 §5 assumes infinite precision and range in the
//! specifications and converts a value an implementation cannot support to the
//! closest supported one. Division by a number is therefore the one correctly
//! rounded binary64 quotient, not a multiplication by the rounded reciprocal,
//! which rounds twice (#1517). §10.10.1 step 7 reduces an Invert node over a
//! number to the reciprocal of its value; with the specifications' infinite
//! precision that reciprocal is exact, so `49 / 49` is exactly 1.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-simplification
//! A product with a number divisor evaluates its factors in source order, each
//! division one correctly rounded quotient; the range rule (#1297) and the
//! IEEE results of §10.9.1 (division by zero, infinities, NaN, signed zero)
//! are unchanged.

use crate::css::support::failure_classes::{SerializationFailure, serialization_failure};
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::numeric::{CssAngle, CssCalculation, CssLength, CssNumber, CssNumeric};
use surgeist_css::serialization::CssSerializationLimits as Limits;

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

#[test]
fn a_number_divided_by_itself_is_exactly_one() {
    // 49 * binary64(1 / 49) is 0.9999999999999999.
    assert_eq!(number("calc(49 / 49)"), "calc(1)");
    assert_eq!(number("calc(round(down, 49 / 49, 1))"), "calc(1)");
    assert_eq!(number("calc(round(down, 7 * 7 / 49, 1))"), "calc(1)");
    assert_eq!(number("calc(round(down, 98 / 98, 1))"), "calc(1)");
    assert_eq!(number("calc(round(down, 49 / (7 * 7), 1))"), "calc(1)");
}

#[test]
fn dividing_a_dimension_by_a_number_and_multiplying_back_restores_it() {
    // (10 / 49) * 49 is exactly 10 in binary64; 10 * binary64(1 / 49) * 49 is
    // 9.999999999999998.
    assert_eq!(
        length("calc(round(down, 10px / 49 * 49, 1px))"),
        "calc(10px)"
    );
    assert_eq!(
        length("calc(round(down, 100px / 49 * 49, 1px))"),
        "calc(100px)"
    );
    assert_eq!(number("calc(round(down, 7 / 49 * 49, 1))"), "calc(7)");
    assert_eq!(length("calc(round(down, 10px / 3 * 3, 1px))"), "calc(10px)");
}

#[test]
fn a_dimension_divided_by_a_number_is_the_correctly_rounded_quotient() {
    // 100 / 3 rounds to 33.333333333333336; 100 * binary64(1 / 3) is
    // 33.33333333333333, one unit in the last place lower.
    assert_eq!(
        number("calc(sign(100px / 3 - 33.333333333333336px))"),
        "calc(0)"
    );
}

#[test]
fn chained_divisions_divide_in_source_order() {
    // binary64(147 / 3) / 49 is exactly 1; 147 * binary64(1 / 3) * binary64(1 / 49)
    // is 0.9999999999999999.
    assert_eq!(number("calc(round(down, 147 / 3 / 49, 1))"), "calc(1)");
    assert_eq!(number("calc(round(down, 294 / 3 / 49, 1))"), "calc(2)");
    assert_eq!(
        length("calc(round(down, 147px / 3 / 49, 1px))"),
        "calc(1px)"
    );
}

#[test]
fn an_explicitly_written_reciprocal_is_a_separate_operation() {
    // `1 / 49` is calculated first and rounded, then multiplied: two authored
    // operations round twice, as the product of the two roundings.
    assert_eq!(number("calc(round(down, 49 * (1 / 49), 1))"), "calc(0)");
    // Left to right, 1 / 49 rounds before it is multiplied by 49.
    assert_eq!(number("calc(round(down, 1 / 49 * 49, 1))"), "calc(0)");
}

#[test]
fn division_by_zero_and_infinities_keep_their_ieee_results() {
    // §10.9.1: division follows IEEE-754; the signed zero of a literal is
    // positive, and a quotient with an infinite operand is not converted.
    for (source, expected) in [
        ("calc(1 / 0)", "calc(infinity)"),
        ("calc(-1 / 0)", "calc(-infinity)"),
        ("calc(1 / -0)", "calc(infinity)"),
        ("calc(0 / 0)", "calc(NaN)"),
        ("calc(5 / infinity)", "calc(0)"),
        // A signed zero is not emitted, but it keeps its sign inside the tree.
        ("calc(1 / (5 / -infinity))", "calc(-infinity)"),
        ("calc(1 / (5 / infinity))", "calc(infinity)"),
        ("calc(1 / (0 / -5))", "calc(-infinity)"),
        ("calc(infinity / infinity)", "calc(NaN)"),
        ("calc(infinity / 2)", "calc(infinity)"),
        ("calc(-infinity / 2)", "calc(-infinity)"),
        ("calc(NaN / 2)", "calc(NaN)"),
        ("calc(2 / NaN)", "calc(NaN)"),
        ("calc(1 / 0 / 0)", "calc(infinity)"),
        ("calc(1 / 0 * 0)", "calc(NaN)"),
    ] {
        assert_eq!(number(source), expected, "{source}");
    }
    assert_eq!(length("calc(10px / 0)"), "calc(infinity * 1px)");
    assert_eq!(length("calc(-10px / 0)"), "calc(-infinity * 1px)");
    assert_eq!(length("calc(0px / 0)"), "calc(NaN * 1px)");
}

#[test]
fn quotients_beyond_binary64_convert_to_the_closest_finite_value() {
    let max = format!("{:.0}", f64::MAX);
    // A finite quotient above the range is MAX, not an infinity (#1297).
    assert_eq!(number("calc(1 / 1e-320)"), format!("calc({max})"));
    assert_eq!(number("calc(1e300 / 1e-300)"), format!("calc({max})"));
    assert_eq!(number("calc(-1e300 / 1e-300)"), format!("calc(-{max})"));
    // Each source-order step converts: 1e308 * 10 is MAX, and MAX / 10 is finite.
    assert_eq!(
        number("calc(round(down, 1e308 * 10 / 10, 1e300))"),
        number("calc(round(down, 1.7976931348623157e307, 1e300))")
    );
}

#[test]
fn context_dependent_operands_fold_their_number_divisors() {
    // A factor that needs its context cannot evaluate to one scalar, but a
    // Number divisor still folds into it as the specification simplifies a
    // Product with one Number child (§10.10.1 step 9).
    for (source, expected) in [
        ("calc(10em / 4)", "calc(2.5em)"),
        ("calc(10em / 4 * 2)", "calc(5em)"),
        ("calc(1em / 49 * 49)", "calc(1em)"),
        ("calc(2 * 3em / 4)", "calc(1.5em)"),
        ("calc((1em + 2px) / 4)", "calc(0.25em + 0.5px)"),
        ("calc(1em / 0)", "calc(infinity * 1em)"),
    ] {
        assert_eq!(length(source), expected, "{source}");
    }
}

fn length_value(source: &str) -> CssNumeric<CssLength> {
    CssNumeric::<CssLength>::try_from_calculation(
        CssCalculation::<CssLength>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            Default::default(),
        )
        .unwrap_or_else(|error| panic!("{source}: {error:?}")),
    )
    .unwrap()
}

fn angle_value(source: &str) -> CssNumeric<CssAngle> {
    CssNumeric::<CssAngle>::try_from_calculation(
        CssCalculation::<CssAngle>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            Default::default(),
        )
        .unwrap_or_else(|error| panic!("{source}: {error:?}")),
    )
    .unwrap()
}

/// The smallest projection-node limit that serializes, with every smaller
/// limit failing as a projection-node limit.
fn minimal_projection_nodes(
    source: &str,
    serialize: impl Fn(Limits) -> Result<String, SerializationFailure>,
) -> usize {
    let minimal = (2..=32)
        .find(|&nodes| serialize(Limits::new(64, nodes, 256)).is_ok())
        .unwrap_or_else(|| panic!("{source}: no limit up to 32 serializes"));
    for nodes in 1..minimal {
        assert_eq!(
            serialize(Limits::new(64, nodes, 256)),
            Err(SerializationFailure::ProjectionNodeLimit),
            "{source}: {nodes} projection nodes"
        );
    }
    minimal
}

#[test]
fn products_dividing_by_numbers_spend_one_node_for_their_merged_coefficient() {
    // Charging rule (docs/reference.md, calculation projection): every leaf and
    // every generated scalar costs one projection node, `calc()` itself none.
    // The reciprocal of a Number divisor is one generated scalar; the Numbers of
    // a product merge into one coefficient that "spends a shared projection
    // node" when there are at least two of them, a divisor counting as a Number;
    // the typed product is replaced by one scalar; a Sum replacement is one more.
    //   10px / 3 * 3         leaves 10px, 3, 3 (3) + reciprocal (1)
    //                        + merged coefficient (1) + product (1)        = 6
    //   10px * 3 / 3         the same four kinds of node                   = 6
    //   10px / 49 * 49       the same                                      = 6
    //   10px * 3 * 5 / 7     four leaves + reciprocal + merged + product   = 7
    //   10px / 3 / 7         three leaves + two reciprocals + merged
    //                        + product                                     = 7
    //   10px / 3 * 3 + 1px   the six above, the leaf 1px and the Sum       = 8
    //   0px * -1 / 5         leaves 0px, -1, 5 + reciprocal + merged
    //                        + product                                     = 6
    //   10deg / 49 * 49, 100deg / 3 * 3 (angle products, as for px)        = 6
    let lengths = [
        ("calc(10px / 3 * 3)", 6),
        ("calc(10px * 3 / 3)", 6),
        ("calc(10px / 49 * 49)", 6),
        ("calc(10px * 3 * 5 / 7)", 7),
        ("calc(10px / 3 / 7)", 7),
        ("calc(10px / 3 * 3 + 1px)", 8),
        ("calc(0px * -1 / 5)", 6),
    ];
    let angles = [("calc(10deg / 49 * 49)", 6), ("calc(100deg / 3 * 3)", 6)];
    let mut observed = Vec::new();
    for (source, _) in lengths {
        let value = length_value(source);
        observed.push((
            source,
            minimal_projection_nodes(source, |limits| {
                value
                    .serialize_specified(limits)
                    .map_err(|error| serialization_failure(&error))
            }),
        ));
    }
    for (source, _) in angles {
        let value = angle_value(source);
        observed.push((
            source,
            minimal_projection_nodes(source, |limits| {
                value
                    .serialize_specified(limits)
                    .map_err(|error| serialization_failure(&error))
            }),
        ));
    }
    let expected: Vec<_> = lengths.into_iter().chain(angles).collect();
    assert_eq!(observed, expected);
}

#[test]
fn products_with_one_number_or_only_numbers_keep_their_projection_nodes() {
    // `10px / 3`: two leaves, the reciprocal and the product; the single Number
    // merges with nothing. `49 / 49`: two leaves, the reciprocal and the merged
    // coefficient, which is the result. `7 * 7 / 49`: three leaves, the
    // reciprocal and the merged coefficient.
    let source = "calc(10px / 3)";
    let value = length_value(source);
    let nodes = minimal_projection_nodes(source, |limits| {
        value
            .serialize_specified(limits)
            .map_err(|error| serialization_failure(&error))
    });
    assert_eq!(nodes, 4, "{source}");
    for (source, expected) in [("calc(49 / 49)", 4), ("calc(7 * 7 / 49)", 5)] {
        let value = CssNumeric::<CssNumber>::try_from_calculation(
            CssCalculation::<CssNumber>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), source).unwrap(),
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap();
        let nodes = minimal_projection_nodes(source, |limits| {
            value
                .serialize_specified(limits)
                .map_err(|error| serialization_failure(&error))
        });
        assert_eq!(nodes, expected, "{source}");
    }
}

#[test]
fn a_product_with_a_divisor_converts_each_source_order_step_to_the_closest_finite_value() {
    // §5: a value that cannot be supported converts to the closest supported
    // one, and every finite intermediate must itself be supported (#1297). With
    // a divisor the factors evaluate in source order, so 1e308 * 10 first
    // converts to f64::MAX and the division then gives f64::MAX / 10.
    let quotient = format!("{:.0}", f64::MAX / 10.0);
    assert_eq!(
        length("calc(1e308px * 10 / 10)"),
        format!("calc({quotient}px)")
    );
    assert_eq!(length("calc(1e308px * 1e308 / 1e308)"), "calc(1.797693px)");
    assert_eq!(
        angle_value("calc(1e308deg * 10 / 10)")
            .serialize_specified(Default::default())
            .unwrap(),
        format!("calc({quotient}deg)")
    );
    // Without a divisor the Number factors merge first, as Product step 9.2
    // says: 10 * 0.1 is 1, so no intermediate overflows.
    assert_eq!(
        length("calc(1e308px * 10 * 0.1)"),
        format!("calc({:.0}px)", 1e308_f64)
    );
}
