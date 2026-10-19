#![forbid(unsafe_code)]
//! Animations ordered matching and adopted WebKit specified-tree math phase.

use surgeist_css::*;

fn selectors(input: &str) -> CssKeyframeSelectorList {
    let report = parse_keyframe_selector_list(input);
    assert!(report.is_clean(), "{input}: {:?}", report.diagnostics());
    report.syntax().as_ref().unwrap().selectors().clone()
}

fn source(origin: &CssValueOrigin) -> &str {
    let CssValueOrigin::Parsed(parsed) = origin else {
        panic!("actual parsed origin")
    };
    parsed.source().as_str()
}

#[test]
fn specified_products_keep_direct_division_and_numeric_leaf_admission() {
    for (a, b, equal) in [
        ("calc(10% * 1% / 3%)", "calc(3.3333333333333335%)", true),
        ("calc(10% * 1% / 3%)", "calc(3.333333333333333%)", false),
        (
            "calc(25% * sign((1px * 1px) / (1px * 1px)))",
            "calc(25%)",
            false,
        ),
        (
            "calc(25% * sign((1px * 1px) / (1px * 1px)))",
            "calc(25% * sign((1px * 1px) / (1px * 1px)))",
            true,
        ),
        ("calc(25% * sign(1px / 1px))", "calc(25%)", true),
        // Product 9.3's numeric-Invert replacement is a selected source quirk,
        // not ordinary reciprocal algebra: its numeric child's unit survives.
        ("calc(25% * sign(2 / 3%))", "calc(25%)", true),
        ("calc(25% * sign(-2 / 3%))", "calc(-25%)", true),
        ("calc(25% * sign(2 / -3%))", "calc(25%)", false),
    ] {
        assert_eq!(
            selectors(a).matches_normalized(&selectors(b)).unwrap(),
            equal,
            "{a} / {b}"
        );
        assert_eq!(
            selectors(b).matches_normalized(&selectors(a)).unwrap(),
            equal
        );
    }
}

#[test]
fn retained_operation_types_distinguish_identical_post_reduction_children() {
    // Both Products have [6%, 1em] children under the selected replacement.
    // The parser retains P^-1 L on the first Product and P L on the second;
    // pinned IndirectNode equality compares these stored dimensional types.
    let inverse = selectors("calc(25% * sign((2 / 3%) * 1em))");
    let direct = selectors("calc(25% * sign(6% * 1em))");
    assert!(!inverse.matches_normalized(&direct).unwrap());
    assert!(!direct.matches_normalized(&inverse).unwrap());
    assert!(
        inverse
            .matches_normalized(&selectors("calc(25% * sign(( 2 / 3% ) * 1em))"))
            .unwrap()
    );
    // A folded numeric leaf has only its actual leaf fields. The discarded
    // parser type is not compared as if it were retained operation metadata.
    assert!(
        selectors("calc(25% * sign(2 / 3%))")
            .matches_normalized(&selectors("calc(25%)"))
            .unwrap()
    );
}

#[test]
fn normalized_sequences_preserve_order_count_duplicates_and_endpoint_meaning() {
    for (a, b, equal) in [
        ("from, to", "0%, 100%", true),
        ("FROM , 10%, TO", "0%,10%,100%", true),
        ("10%, 10%", "10%,10%", true),
        ("10%, 20%", "20%,10%", false),
        ("10%,10%", "10%", false),
        ("25%,75%", "75%", false),
        ("-0%", "0%", true),
    ] {
        let left = selectors(a);
        let right = selectors(b);
        assert_eq!(left.matches_normalized(&right).unwrap(), equal, "{a} / {b}");
        assert_eq!(right.matches_normalized(&left).unwrap(), equal);
    }
}

#[test]
fn calculations_ignore_authored_origins_and_whitespace_after_input_drop() {
    let left = {
        let input = "calc(10%)".to_owned();
        selectors(&input)
    };
    let right = {
        let input = "\n calc( 10% ) /*trivia*/".to_owned();
        selectors(&input)
    };
    assert_ne!(left, right);
    let original_left = left.clone();
    let original_right = right.clone();
    assert!(left.matches_normalized(&right).unwrap());
    assert_eq!(left, original_left);
    assert_eq!(right, original_right);
    let value = left.selectors()[0].offset();
    assert_eq!(source(value.calculation().unwrap().origin()), "calc(10%)");
}

#[test]
fn native_specified_simplification_compares_foldable_math_without_range_clamping() {
    for (a, b, equal) in [
        ("calc(5% + 5%)", "calc(10%)", true),
        ("calc((5% + 5%) * 2)", "calc(20%)", true),
        ("min(25%, 50%)", "calc(25%)", true),
        ("max(25%,50%)", "calc(50%)", true),
        ("clamp(0%,25%,100%)", "calc(25%)", true),
        ("calc(10% + 1%)", "calc(10%)", false),
        ("calc(120%)", "calc(100%)", false),
        ("calc(-1%)", "calc(0%)", false),
        ("calc(10%)", "10%", false),
        ("calc(0%)", "from", false),
        ("calc(100%)", "to", false),
    ] {
        assert_eq!(
            selectors(a).matches_normalized(&selectors(b)).unwrap(),
            equal,
            "{a} / {b}"
        );
    }
}

#[test]
fn comparison_preserves_preoutput_binary64_distinctions_and_source_order() {
    for (a, b, equal) in [
        ("10.0000001%", "10%", false),
        ("calc(.12345641%)", "calc(.12345642%)", false),
        ("calc(1e16% - 1e16% + 1%)", "calc(1%)", true),
        ("calc(1% + 1e16% - 1e16%)", "calc(0%)", true),
        (
            "calc(1e16% - 1e16% + 1%)",
            "calc(1% + 1e16% - 1e16%)",
            false,
        ),
    ] {
        assert_eq!(
            selectors(a).matches_normalized(&selectors(b)).unwrap(),
            equal,
            "{a} / {b}"
        );
    }
    let a = selectors("calc(.12345641%)");
    let b = selectors("calc(.12345642%)");
    assert_eq!(
        a.serialize_key_text().unwrap(),
        b.serialize_key_text().unwrap()
    );
    assert!(!a.matches_normalized(&b).unwrap());
}

#[test]
fn symbolic_math_preserves_context_without_guessing_external_unit_values() {
    for (a, b, equal) in [
        (
            "calc(25% * sign(1em - 1px))",
            "calc( 25% * sign( 1em - 1px ) )",
            true,
        ),
        (
            "calc(25% * sign(1em - 1px))",
            "calc(25% * sign(1rem - 1px))",
            false,
        ),
        ("calc(25% * sign(1em - 1px))", "calc(25%)", false),
        (
            "calc(25% * sign(1em - 1px))",
            "calc(25% * sign(1em - 2px))",
            false,
        ),
    ] {
        assert_eq!(
            selectors(a).matches_normalized(&selectors(b)).unwrap(),
            equal,
            "{a} / {b}"
        );
    }
}

#[test]
fn normalized_double_leaves_preserve_signed_infinity_and_nonreflexive_nan() {
    for (a, b, equal) in [
        ("calc(infinity * 1%)", "calc(1% / 0)", true),
        ("calc(infinity * 1%)", "calc(-infinity * 1%)", false),
        ("calc(infinity * 1%)", "calc(100%)", false),
        ("calc(-0%)", "calc(0%)", true),
        ("calc(NaN * 1%)", "calc(NaN * 1%)", false),
        ("calc(0% / 0)", "calc(NaN * 1%)", false),
    ] {
        assert_eq!(
            selectors(a).matches_normalized(&selectors(b)).unwrap(),
            equal,
            "{a} / {b}"
        );
    }
    let nan = selectors("calc(NaN * 1%)");
    assert!(!nan.matches_normalized(&nan).unwrap());
}

#[test]
fn exact_literal_aggregate_tariffs_need_no_output_bytes_and_retry_is_atomic() {
    let a = selectors("from, to");
    let b = selectors("0%,100%");
    let before = (a.clone(), b.clone());
    assert!(
        a.matches_normalized_with_limits(&b, CssSpecifiedValueSerializationLimits::new(6, 8, 0))
            .unwrap()
    );
    let error = a
        .matches_normalized_with_limits(&b, CssSpecifiedValueSerializationLimits::new(5, 8, 0))
        .unwrap_err();
    assert_eq!(
        error.cause().kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        error.stage(),
        CssKeyframeSelectorComparisonStage::RightNormalization
    );
    assert_eq!(error.selector_index(), Some(1));
    assert!(error.left_origin().is_none());
    assert!(error.right_origin().is_none());
    let error = a
        .matches_normalized_with_limits(&b, CssSpecifiedValueSerializationLimits::new(6, 7, 0))
        .unwrap_err();
    assert_eq!(
        error.cause().kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        error.stage(),
        CssKeyframeSelectorComparisonStage::Comparison
    );
    assert_eq!(error.selector_index(), Some(1));
    assert_eq!((a.clone(), b.clone()), before);
    assert!(a.matches_normalized(&b).unwrap());
}

#[test]
fn later_math_normalization_and_comparison_failures_retain_both_actual_origins() {
    let a = selectors("from, calc(10%)");
    let b = selectors("0%, calc( 10% )");
    let error = a
        .matches_normalized_with_limits(&b, CssSpecifiedValueSerializationLimits::new(9, 11, 0))
        .unwrap_err();
    assert_eq!(
        error.cause().kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        error.stage(),
        CssKeyframeSelectorComparisonStage::RightNormalization
    );
    assert_eq!(error.selector_index(), Some(1));
    assert!(error.left_origin().is_none());
    assert_eq!(source(error.right_origin().unwrap()), "0%, calc( 10% )");
    let error = a
        .matches_normalized_with_limits(&b, CssSpecifiedValueSerializationLimits::new(10, 10, 0))
        .unwrap_err();
    assert_eq!(
        error.cause().kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        error.stage(),
        CssKeyframeSelectorComparisonStage::Comparison
    );
    assert_eq!(error.selector_index(), Some(1));
    assert_eq!(source(error.left_origin().unwrap()), "from, calc(10%)");
    assert_eq!(source(error.right_origin().unwrap()), "0%, calc( 10% )");
    assert!(
        a.matches_normalized_with_limits(&b, CssSpecifiedValueSerializationLimits::new(10, 11, 0))
            .unwrap()
    );
}

#[test]
fn a_request_meter_prepares_query_once_and_accumulates_all_candidate_work() {
    let query = selectors("10%");
    let same = selectors("10%");
    let different = selectors("20%");
    let mut matcher = CssKeyframeSelectorMatcher::try_new_with_limits(
        &query,
        CssSpecifiedValueSerializationLimits::new(6, 8, 0),
    )
    .unwrap();
    assert!(matcher.matches(&same).unwrap());
    assert!(!matcher.matches(&different).unwrap());
    let error = matcher.matches(&same).unwrap_err();
    assert_eq!(
        error.cause().kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        error.stage(),
        CssKeyframeSelectorComparisonStage::RightNormalization
    );
    assert_eq!(error.selector_index(), None);
    let mut retry = CssKeyframeSelectorMatcher::try_new_with_limits(
        &query,
        CssSpecifiedValueSerializationLimits::new(8, 11, 0),
    )
    .unwrap();
    assert!(retry.matches(&same).unwrap());
    assert!(!retry.matches(&different).unwrap());
    assert!(retry.matches(&same).unwrap());
}

#[test]
fn a_later_resource_failure_remains_an_error_even_after_an_earlier_unequal_member() {
    let a = selectors("10%, calc(10%)");
    let b = selectors("20%, calc(10%)");
    let error = a
        .matches_normalized_with_limits(&b, CssSpecifiedValueSerializationLimits::new(10, 10, 0))
        .unwrap_err();
    assert_eq!(
        error.stage(),
        CssKeyframeSelectorComparisonStage::Comparison
    );
    assert_eq!(error.selector_index(), Some(1));
    assert!(!a.matches_normalized(&b).unwrap());
}

#[test]
fn specified_tree_keeps_merged_sum_units_at_first_positions_among_symbolic_terms() {
    for (a, b, equal) in [
        (
            "calc(25% * sign(1em + 1px))",
            "calc(25% * sign(1px + 1em))",
            false,
        ),
        (
            "calc(25% * sign(1em + 1px + 2em))",
            "calc(25% * sign(3em + 1px))",
            true,
        ),
        (
            "calc(25% * sign(1px + 1em + 2em))",
            "calc(25% * sign(3em + 1px))",
            false,
        ),
        (
            "calc(25% * sign(1em + sign(1em - 1px) * 1px + 2em))",
            "calc(25% * sign(3em + sign(1em - 1px) * 1px))",
            true,
        ),
    ] {
        assert_eq!(
            selectors(a).matches_normalized(&selectors(b)).unwrap(),
            equal,
            "{a} / {b}"
        );
        assert_eq!(
            selectors(b).matches_normalized(&selectors(a)).unwrap(),
            equal
        );
    }
}

#[test]
fn specified_product_places_combined_number_after_non_numbers() {
    for a in [
        "calc(2 * 25% * sign(1em - 1px))",
        "calc(25% * 2 * sign(1em - 1px))",
        "calc(25% * sign(1em - 1px) * 2)",
        "calc(2 * 25% * sign(1em - 1px) * 1)",
    ] {
        let b = "calc(25% * sign(1em - 1px) * 2)";
        assert!(
            selectors(a).matches_normalized(&selectors(b)).unwrap(),
            "{a}"
        );
    }
}

#[test]
fn specified_min_max_merge_only_matching_numeric_subsets_and_keep_remaining_order() {
    for (a, b, equal) in [
        (
            "calc(25% * sign(min(1em, 2em, 1px)))",
            "calc(25% * sign(min(1em, 1px)))",
            true,
        ),
        (
            "calc(25% * sign(max(1em, 2em, 1px)))",
            "calc(25% * sign(max(2em, 1px)))",
            true,
        ),
        (
            "calc(25% * sign(min(1px, 1em, 2em)))",
            "calc(25% * sign(min(1px, 1em)))",
            true,
        ),
        (
            "calc(25% * sign(min(1em, 2em, 1px)))",
            "calc(25% * sign(min(1px, 1em)))",
            false,
        ),
        (
            "calc(25% * sign(max(1em, 2em, 1px)))",
            "calc(25% * sign(max(1em, 1px)))",
            false,
        ),
    ] {
        assert_eq!(
            selectors(a).matches_normalized(&selectors(b)).unwrap(),
            equal,
            "{a} / {b}"
        );
    }
}

#[test]
fn failed_query_preparation_exposes_its_actual_origin_and_can_retry_unchanged() {
    let query = {
        let input = "calc(10%)".to_owned();
        selectors(&input)
    };
    let before = query.clone();
    let error = match CssKeyframeSelectorMatcher::try_new_with_limits(
        &query,
        CssSpecifiedValueSerializationLimits::new(2, 100, 0),
    ) {
        Ok(_) => panic!("native input normalization must exceed this limit"),
        Err(error) => error,
    };
    assert_eq!(
        error.stage(),
        CssKeyframeSelectorComparisonStage::LeftNormalization
    );
    assert_eq!(error.selector_index(), Some(0));
    assert_eq!(
        error.cause().kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(source(error.left_origin().unwrap()), "calc(10%)");
    assert!(error.right_origin().is_none());
    assert_eq!(query, before);
    let mut retry = CssKeyframeSelectorMatcher::try_new(&query).unwrap();
    assert!(retry.matches(&selectors("calc(5% + 5%)")).unwrap());
}

#[test]
fn selected_numeric_container_negation_preserves_source_roots_and_unequal_controls() {
    for (a, b, equal) in [
        (
            "calc(25% * sign(1px - (1em + 2px)))",
            "calc(25% * sign(-1px - 1em))",
            true,
        ),
        (
            "calc(25% * sign(1px - (1em + 2px)))",
            "calc(25% * sign(-1em - 1px))",
            false,
        ),
        (
            "calc(25% * sign(1px - (1em + 2px)))",
            "calc(25% * sign(-2px - 1em))",
            false,
        ),
        (
            "calc(25% * sign(1px - (1em + sign(1em - 1px) * 2px)))",
            "calc(25% * sign(1px - 1em - sign(1em - 1px) * 2px))",
            false,
        ),
        // This is the selected pin's numeric Product childwise-negation root
        // relation, rather than a claim of computed arithmetic equivalence.
        (
            "calc(25% * sign(1em * 1px - (2em * 1px)))",
            "calc(25% * sign(1em * 1px + (-2em * -1px)))",
            true,
        ),
    ] {
        assert_eq!(
            selectors(a).matches_normalized(&selectors(b)).unwrap(),
            equal,
            "{a} / {b}"
        );
        assert_eq!(
            selectors(b).matches_normalized(&selectors(a)).unwrap(),
            equal
        );
    }
}

#[test]
fn selected_one_sided_clamp_requires_numeric_middle_and_preserves_bound_order() {
    for (a, b, equal) in [
        (
            "calc(25% * sign(clamp(none, 1em, 1px)))",
            "calc(25% * sign(min(1em, 1px)))",
            true,
        ),
        (
            "calc(25% * sign(clamp(none, 1em, 1px)))",
            "calc(25% * sign(min(1px, 1em)))",
            false,
        ),
        (
            "calc(25% * sign(clamp(1em, 1px, none)))",
            "calc(25% * sign(max(1em, 1px)))",
            true,
        ),
        (
            "calc(25% * sign(clamp(1em, 1px, none)))",
            "calc(25% * sign(max(1px, 1em)))",
            false,
        ),
        (
            "calc(25% * sign(clamp(none, 1em + 1px, 1px)))",
            "calc(25% * sign(min(1em + 1px, 1px)))",
            false,
        ),
        (
            "calc(25% * sign(clamp(none, 1em + 1px, none)))",
            "calc(25% * sign(1em + 1px))",
            true,
        ),
        (
            "calc(25% * sign(clamp(1em, 1px, 1rem)))",
            "calc(25% * sign(max(1em, min(1px, 1rem))))",
            false,
        ),
    ] {
        assert_eq!(
            selectors(a).matches_normalized(&selectors(b)).unwrap(),
            equal,
            "{a} / {b}"
        );
    }
}

#[test]
fn selected_leaf_predicates_distinguish_magnitudes_resolution_and_nonfinite_roots() {
    for (a, b, equal) in [
        ("calc(25% * sign(1em))", "calc(25%)", true),
        ("calc(25% * sign(-1em))", "calc(-25%)", true),
        ("calc(25% * sign(abs(-1ch)))", "calc(25%)", true),
        (
            "calc(25% * sign(hypot(3em, 4em)))",
            "calc(25% * sign(5em))",
            false,
        ),
        (
            "calc(25% * sign(hypot(3em, 4em)))",
            "calc(25% * sign(hypot(3em, 4em)))",
            true,
        ),
        ("calc(25% * sign(hypot(3fr, 4fr)))", "calc(25%)", true),
        (
            "calc(25% * sin(1e308turn))",
            "calc(25% * sin(1e308turn))",
            false,
        ),
        (
            "calc(25% * sin(1e308deg + 1e308deg))",
            "calc(25% * sin(1e308deg + 1e308deg))",
            false,
        ),
        (
            "calc(25% * sin(1e308deg * 2))",
            "calc(25% * sin(1e308deg * 2))",
            false,
        ),
        ("calc(25% * sign(1e308turn))", "calc(25%)", true),
        ("calc(25% * sin(.25turn))", "calc(25%)", true),
    ] {
        assert_eq!(
            selectors(a).matches_normalized(&selectors(b)).unwrap(),
            equal,
            "{a} / {b}"
        );
    }
}

#[test]
fn derived_numeric_negation_is_charged_cumulatively_with_real_origins_and_retry() {
    let query = selectors("calc(25% * sign(1px - (1em + 2px)))");
    let candidate = selectors("calc(25% * sign(-1px - 1em))");
    let before = (query.clone(), candidate.clone());
    // The native checked walk visits ten query nodes/seven candidate nodes.
    // Aggregate/selector slots add two input nodes each. Query normalization
    // uses 17 projection nodes, candidate 11; selector/root comparison adds 7.
    assert!(
        query
            .matches_normalized_with_limits(
                &candidate,
                CssSpecifiedValueSerializationLimits::new(21, 35, 0)
            )
            .unwrap()
    );
    let error = query
        .matches_normalized_with_limits(
            &candidate,
            CssSpecifiedValueSerializationLimits::new(21, 34, 0),
        )
        .unwrap_err();
    assert_eq!(
        error.cause().kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        error.stage(),
        CssKeyframeSelectorComparisonStage::Comparison
    );
    assert_eq!(
        source(error.left_origin().unwrap()),
        "calc(25% * sign(1px - (1em + 2px)))"
    );
    assert_eq!(
        source(error.right_origin().unwrap()),
        "calc(25% * sign(-1px - 1em))"
    );
    let mut matcher = CssKeyframeSelectorMatcher::try_new_with_limits(
        &query,
        CssSpecifiedValueSerializationLimits::new(30, 52, 0),
    )
    .unwrap();
    assert!(matcher.matches(&candidate).unwrap());
    assert_eq!(
        matcher.matches(&candidate).unwrap_err().stage(),
        CssKeyframeSelectorComparisonStage::Comparison
    );
    let mut retry = CssKeyframeSelectorMatcher::try_new_with_limits(
        &query,
        CssSpecifiedValueSerializationLimits::new(30, 53, 0),
    )
    .unwrap();
    assert!(retry.matches(&candidate).unwrap());
    assert!(retry.matches(&candidate).unwrap());
    assert_eq!((query, candidate), before);
}

#[test]
fn product_resolution_uses_normalized_leaf_units_and_retains_unfolded_type() {
    for (a, b, equal) in [
        // The pin's 9.3 replacement is a Percentage leaf, while the parser
        // keeps its inverse-Percentage type. Actual leaves in 9.4 form P^2,
        // so this Product and its enclosing sign remain symbolic.
        ("calc(25% * sign((2 / 3%) * 1%))", "calc(25%)", false),
        (
            "calc(25% * sign((2 / 3%) * 1%))",
            "calc( 25% * sign( ( 2 / 3% ) * 1% ) )",
            true,
        ),
        // Identical resulting children still have different retained Product
        // types (Number versus P^2), which are part of specified-tree equality.
        (
            "calc(25% * sign((2 / 3%) * 1%))",
            "calc(25% * sign(6% * 1%))",
            false,
        ),
        // Conversely the actual Percentage / Percentage leaves resolve to a
        // Number even though the original parser type is inverse-P^2.
        ("calc(25% * sign((2 / 3%) / 1%))", "calc(25%)", true),
        ("calc(25% * sign((2 / 3%) / -1%))", "calc(-25%)", true),
        ("calc(25% * sign((2 / 3%) * 1px))", "calc(25%)", false),
    ] {
        let left = selectors(a);
        let right = selectors(b);
        let before = (left.clone(), right.clone());
        assert_eq!(left.matches_normalized(&right).unwrap(), equal, "{a} / {b}");
        assert_eq!(right.matches_normalized(&left).unwrap(), equal, "{b} / {a}");
        assert_eq!((left, right), before);
    }
}
