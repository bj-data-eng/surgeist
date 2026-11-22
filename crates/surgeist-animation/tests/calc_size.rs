#![forbid(unsafe_code)]

use surgeist_animation::computed::{
    CalcSize, CalcSizeBasis, CalcSizeCalculation, CalcSizeFootprint, CalcSizeNode,
    ComputedDimension, ComputedErrorKind, ComputedNumeric, DimensionKind, MathContext, MathSource,
    NumericLiteral, NumericType, SizingKeyword,
};
use surgeist_animation::interpolation::InterpolationProgress;

#[test]
fn length_and_percentage_bases_combine_their_expanded_calculations() {
    let context = MathContext::new();
    let twice_size = context.source(NumericType::Dimension(DimensionKind::Length));
    let size_minus_twenty = context.source(NumericType::Dimension(DimensionKind::Length));
    let first = CalcSize::with_numeric_basis(&length(100.0), &source(&twice_size, 3, 1, 0))
        .expect("a pure-length basis is valid");
    let second = CalcSize::with_numeric_basis(
        &CalcSizeCalculation::from_numeric(ComputedNumeric::percentage(0.5).unwrap()).unwrap(),
        &source(&size_minus_twenty, 3, 1, 0),
    )
    .expect("a percentage basis is valid");

    assert_eq!(first.basis(), CalcSizeBasis::Any);
    assert_eq!(second.basis(), CalcSizeBasis::Percentage);
    for (from, to) in [(&first, &second), (&second, &first)] {
        let sampled = midpoint(from, to);
        assert_eq!(sampled.basis(), CalcSizeBasis::Percentage);
        let resolve = |source: &MathSource, size: f64, _: f64| {
            if source == &twice_size {
                size * 2.0
            } else {
                assert_eq!(source, &size_minus_twenty);
                size - 20.0
            }
        };
        // Values 5 §10.1 gives the endpoint results 200px and 30px.
        assert_eq!(
            evaluate(sampled.calculation(), 100.0, 100.0, resolve),
            115.0
        );
    }
}

#[test]
fn a_nested_calculation_substitutes_size_and_retains_ordinary_percentages() {
    let context = MathContext::new();
    let sizing_keyword = SizingKeyword::MinContent;
    let inner_source = context.source(NumericType::Dimension(DimensionKind::Length));
    let outer_source = context.source(NumericType::DimensionPercentage(DimensionKind::Length));
    let inner = CalcSize::new(
        CalcSizeBasis::Keyword(sizing_keyword),
        source(&inner_source, 3, 1, 0),
    )
    .unwrap();
    let nested = CalcSize::with_nested_basis(&inner, &source(&outer_source, 5, 1, 1)).unwrap();
    let sampled = midpoint(&nested, &CalcSize::from_keyword(sizing_keyword));

    let resolve = |source: &MathSource, size: f64, percentage_basis: f64| {
        if source == &inner_source {
            size + 10.0
        } else {
            assert_eq!(source, &outer_source);
            2.0 * size + 0.25 * percentage_basis
        }
    };
    assert_eq!(sampled.basis(), CalcSizeBasis::Keyword(sizing_keyword));
    // The nested endpoint is 120px and the plain keyword endpoint is 40px.
    assert_eq!(evaluate(sampled.calculation(), 40.0, 80.0, resolve), 80.0);
}

#[test]
fn different_concrete_bases_do_not_interpolate() {
    let auto = CalcSize::from_keyword(SizingKeyword::Auto);
    let min_content = CalcSize::from_keyword(SizingKeyword::MinContent);
    let percentage = CalcSize::new(CalcSizeBasis::Percentage, CalcSizeCalculation::size()).unwrap();
    let zero = CalcSize::from_numeric(ComputedNumeric::dimension(
        ComputedDimension::new(DimensionKind::Length, 0.0).unwrap(),
    ))
    .unwrap();

    assert!(!auto.can_interpolate(&min_content));
    assert!(!auto.can_interpolate(&percentage));
    assert_eq!(auto.interpolate(&min_content, half()).unwrap(), None);
    assert!(auto.can_interpolate(&zero));
    let sampled = midpoint(&auto, &zero);
    assert_eq!(sampled.basis(), CalcSizeBasis::Keyword(SizingKeyword::Auto));
    assert_eq!(
        evaluate(sampled.calculation(), 48.0, 100.0, |_, _, _| unreachable!()),
        24.0
    );
}

#[test]
fn any_and_numeric_bases_reject_unbound_size() {
    let size = CalcSizeCalculation::size();
    assert_eq!(
        CalcSize::new(CalcSizeBasis::Any, size.clone())
            .unwrap_err()
            .kind(),
        ComputedErrorKind::IncompatibleTypes
    );
    assert_eq!(
        CalcSize::with_numeric_basis(&size, &length(12.0))
            .unwrap_err()
            .kind(),
        ComputedErrorKind::IncompatibleTypes
    );
    assert_eq!(
        CalcSize::from_numeric(ComputedNumeric::number(2.0).unwrap())
            .unwrap_err()
            .kind(),
        ComputedErrorKind::IncompatibleTypes
    );
}

#[test]
fn a_numeric_basis_uses_declared_percentage_occurrences_instead_of_its_result_type() {
    let context = MathContext::new();
    let length_percentage = NumericType::DimensionPercentage(DimensionKind::Length);
    let basis_source = context.source(length_percentage);
    let captured = source(&basis_source, 1, 0, 0);
    let checked = CalcSize::with_numeric_basis(&captured, &CalcSizeCalculation::size()).unwrap();
    assert_eq!(checked.basis(), CalcSizeBasis::Any);
    assert_eq!(
        evaluate(checked.calculation(), 0.0, 100.0, |_, _, _| 40.0),
        40.0
    );

    let unknown = CalcSizeCalculation::from_numeric(ComputedNumeric::source(basis_source)).unwrap();
    assert_eq!(
        CalcSize::with_numeric_basis(&unknown, &CalcSizeCalculation::size())
            .unwrap_err()
            .kind(),
        ComputedErrorKind::IncompatibleTypes
    );
}

#[test]
fn nested_basis_skips_an_unused_inner_expansion() {
    let context = MathContext::new();
    let kind = NumericType::Dimension(DimensionKind::Length);
    let basis =
        CalcSizeCalculation::from_numeric(ComputedNumeric::percentage(0.5).unwrap()).unwrap();
    let inner = source(&context.source(kind), 3001, 1001, 0);
    let outer = length(12.0);

    // Substituting the percentage into all 1,001 size references would exceed
    // the budget. The outer expression discards the inner calculation first.
    let prepared = CalcSize::with_numeric_basis(&basis, &inner)
        .and_then(|inner| CalcSize::with_nested_basis(&inner, &outer))
        .expect("the ignored inner calculation takes no space in the result");
    assert_eq!(prepared.basis(), CalcSizeBasis::Percentage);
    assert_eq!(prepared.calculation(), &outer);
    let sampled = midpoint(
        &prepared,
        &CalcSize::new(CalcSizeBasis::Any, length(20.0)).unwrap(),
    );
    assert_eq!(sampled.basis(), CalcSizeBasis::Percentage);
    assert_eq!(
        evaluate(
            sampled.calculation(),
            100.0,
            100.0,
            |_, _, _| unreachable!()
        ),
        16.0
    );
}

#[test]
fn substitution_rejects_excessive_expansion_and_leaves_inputs_usable() {
    let context = MathContext::new();
    let kind = NumericType::Dimension(DimensionKind::Length);
    let basis_source = context.source(kind);
    let calculation_source = context.source(kind);
    let basis = CalcSize::new(CalcSizeBasis::Any, source(&basis_source, 3000, 0, 0)).unwrap();
    let repeated_size = source(&calculation_source, 3, 2, 0);

    assert_eq!(
        CalcSize::with_nested_basis(&basis, &repeated_size)
            .unwrap_err()
            .kind(),
        ComputedErrorKind::Capacity
    );
    let ignores_size = length(12.0);
    let result = CalcSize::with_nested_basis(&basis, &ignores_size).unwrap();
    assert_eq!(result.calculation(), &ignores_size);
    assert_eq!(
        basis.calculation().nodes(),
        &[CalcSizeNode::Source(basis_source)]
    );
}

#[test]
fn a_pair_exceeding_the_plan_budget_remains_available_for_discrete_sampling() {
    let context = MathContext::new();
    let kind = NumericType::Dimension(DimensionKind::Length);
    let first = CalcSize::new(
        CalcSizeBasis::Any,
        source(&context.source(kind), 2500, 0, 0),
    )
    .unwrap();
    let second = CalcSize::new(
        CalcSizeBasis::Any,
        source(&context.source(kind), 2500, 0, 0),
    )
    .unwrap();

    assert!(!first.can_interpolate(&second));
    assert_eq!(first.interpolate(&second, half()).unwrap(), None);
    assert_eq!(first.basis(), CalcSizeBasis::Any);
    assert_eq!(second.basis(), CalcSizeBasis::Any);
}

fn length(value: f64) -> CalcSizeCalculation {
    CalcSizeCalculation::from_numeric(ComputedNumeric::dimension(
        ComputedDimension::new(DimensionKind::Length, value).unwrap(),
    ))
    .unwrap()
}

fn source(
    source: &MathSource,
    nodes: usize,
    size: usize,
    percentages: usize,
) -> CalcSizeCalculation {
    CalcSizeCalculation::source(
        source.clone(),
        CalcSizeFootprint::new(nodes, size, percentages).unwrap(),
    )
    .unwrap()
}

fn half() -> InterpolationProgress {
    InterpolationProgress::new(0.5).unwrap()
}

fn midpoint(first: &CalcSize, second: &CalcSize) -> CalcSize {
    assert!(first.can_interpolate(second));
    first.interpolate(second, half()).unwrap().unwrap()
}

fn evaluate(
    calculation: &CalcSizeCalculation,
    size: f64,
    percentage_basis: f64,
    resolve: impl Fn(&MathSource, f64, f64) -> f64,
) -> f64 {
    fn at(
        nodes: &[CalcSizeNode],
        index: usize,
        size: f64,
        percentage_basis: f64,
        resolve: &impl Fn(&MathSource, f64, f64) -> f64,
    ) -> f64 {
        let eval = |index| at(nodes, index, size, percentage_basis, resolve);
        match &nodes[index] {
            CalcSizeNode::Size => size,
            CalcSizeNode::Numeric(numeric) => match numeric.as_literal().unwrap() {
                NumericLiteral::Dimension(value) => value.value(),
                NumericLiteral::Percentage(value) => value * percentage_basis,
                other => panic!("unexpected test payload {other:?}"),
            },
            CalcSizeNode::Source(source) => resolve(source, size, percentage_basis),
            CalcSizeNode::Depercentify { calculation } => {
                at(nodes, *calculation, size, size, resolve)
            }
            CalcSizeNode::Substitute { calculation, size } => {
                at(nodes, *calculation, eval(*size), percentage_basis, resolve)
            }
            CalcSizeNode::Interpolate {
                first,
                second,
                progress,
            } => (1.0 - progress.value()) * eval(*first) + progress.value() * eval(*second),
        }
    }
    let nodes = calculation.nodes();
    at(nodes, nodes.len() - 1, size, percentage_basis, &resolve)
}
