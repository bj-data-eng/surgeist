#![forbid(unsafe_code)]

//! Keyframe literals select binary64 conversion before inclusive range checking.
//! Percentage functions retain Values 4 authored math and defer range clamping.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-range
//! https://www.w3.org/TR/2023/WD-css-animations-1-20230302/#keyframes

use surgeist_css::*;

fn calculation(source: &str) -> CssPercentageCalculation {
    CssPercentageCalculation::try_from_components(parse_component_values(source).unwrap()).unwrap()
}

fn percentage(source: &str) -> CssKeyframePercent {
    CssKeyframePercent::try_from_calculation(calculation(source)).unwrap()
}

fn sheet(source: &str) -> CssSheet {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().clone()
}

fn keyframes(sheet: &CssSheet) -> &CssKeyframesRule {
    let [CssRule::Keyframes(rule)] = sheet.rules() else {
        panic!("one keyframes rule");
    };
    rule
}

#[test]
fn binary64_literal_constructor_preserves_finite_inclusive_boundaries() {
    for value in [0.0, -0.0, f64::from_bits(1), 1e-46, 99.999999, 100.0] {
        let percent = CssKeyframePercent::try_new(value).unwrap();
        assert_eq!(percent.literal_value().unwrap().to_bits(), value.to_bits());
        assert!(percent.calculation().is_none());
        assert_eq!(percent, percent.clone());
    }
    assert_eq!(
        CssKeyframePercent::try_new(0.0),
        CssKeyframePercent::try_new(-0.0)
    );
}

#[test]
fn binary64_literal_constructor_rejects_nonfinite_and_representable_out_of_range_values() {
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -f64::from_bits(1),
        -1e-46,
        100.000001,
        f64::from_bits(100.0_f64.to_bits() + 1),
    ] {
        assert!(CssKeyframePercent::try_new(value).is_none(), "{value}");
    }
}

#[test]
fn bare_percentage_roots_share_literal_bounds_after_binary64_conversion() {
    for source in ["-1e-46%", "-5e-324%", "100.000001%", "101%", "1e999%"] {
        assert!(
            CssKeyframePercent::try_from_calculation(calculation(source)).is_none(),
            "{source}"
        );
    }
    for (source, value) in [
        ("100.000000000000000000001%", 100.0),
        ("-1e-999%", -0.0),
        ("5e-324%", f64::from_bits(1)),
        ("99.999999%", 99.999999),
    ] {
        let percent = percentage(source);
        assert_eq!(percent.literal_value().unwrap().to_bits(), value.to_bits());
        assert!(percent.calculation().is_none());
    }
}

#[test]
fn root_classification_ignores_leading_and_trailing_numeric_trivia() {
    let literal = percentage(" /**/ 25% /* end */ ");
    assert_eq!(literal.literal_value(), Some(25.0));
    assert!(literal.calculation().is_none());
    assert!(CssKeyframePercent::try_from_calculation(calculation(" /**/ 101% ")).is_none());

    let source = " /* lead */ calc(120%) /* end */ ";
    let original = calculation(source);
    let percent = CssKeyframePercent::try_from_calculation(original.clone()).unwrap();
    assert!(percent.literal_value().is_none());
    assert_eq!(percent.calculation(), Some(&original));
    assert_eq!(
        percent.calculation().unwrap().components(),
        original.components()
    );
}

#[test]
fn percentage_functions_keep_checked_symbolic_roots_and_unclamped_ranges() {
    for source in [
        "calc(-10%)",
        "calc(120%)",
        "min(25%,50%)",
        "max(25%,50%)",
        "clamp(0%,25%,100%)",
        "calc(25% * sign(1em - 1px))",
    ] {
        let original = calculation(source);
        let percent = CssKeyframePercent::try_from_calculation(original.clone()).unwrap();
        assert!(percent.literal_value().is_none(), "{source}");
        assert_eq!(percent.calculation(), Some(&original));
        assert_eq!(
            percent.calculation().unwrap().result_type(),
            CssCalculationType::Percentage
        );
        assert_eq!(percent.calculation().unwrap().origin(), original.origin());
    }
}

#[test]
fn wrong_numeric_types_and_malformed_functions_fail_checked_percentage_construction() {
    for source in [
        "calc(25)",
        "calc(25px)",
        "calc(25% + 1px)",
        "calc(25% +)",
        "min()",
    ] {
        assert!(
            CssPercentageCalculation::try_from_components(parse_component_values(source).unwrap())
                .is_err(),
            "{source}"
        );
    }
}

#[test]
fn a_percentage_group_is_rejected_before_keyframe_construction() {
    let components = parse_component_values("(25%)").unwrap();
    let before = components.clone();
    let error = CssPercentageCalculation::try_from_components(components.clone()).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::RootDomainMismatch
    );
    assert_eq!(components, before);
    assert_eq!(components.serialize().unwrap().as_css(), "(25%)");
}

#[test]
fn standalone_checked_percentage_functions_reject_implicit_closures() {
    let components = parse_component_values("calc(25%").unwrap();
    let error = CssPercentageCalculation::try_from_components(components).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::RecoveredComponent
    );
}

#[test]
fn recovering_parsed_percentage_child_can_be_reused_without_erasing_implicit_origin() {
    let source = "opacity:calc(25%";
    let report = parse_style_attribute(source);
    assert!(!report.is_clean());
    let CssKnownPropertyValueRef::Opacity(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("retained opacity");
    };
    let CssOpacityValue::PercentageCalculation(original) = wrapper.value() else {
        panic!("recovering percentage function");
    };
    let percent = CssKeyframePercent::try_from_calculation(original.clone()).unwrap();
    assert_eq!(percent.calculation(), Some(original));
    let CssComponentValueRef::Function(function) =
        percent.calculation().unwrap().components().items()[0].view()
    else {
        panic!("retained function graph");
    };
    assert!(matches!(
        function.closing_origin(),
        CssValueOrigin::ImplicitClosure { .. }
    ));
    let CssValueOrigin::Parsed(origin) =
        percent.calculation().unwrap().components().items()[0].origin()
    else {
        panic!("original parsed function origin");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        "opacity:".len()
    );
    let specified =
        CssSpecifiedPercentage::try_from_calculation(percent.calculation().unwrap().clone())
            .unwrap();
    assert_eq!(specified.serialize_specified().unwrap(), "calc(25%)");
    assert_eq!(percent.calculation(), Some(original));
    assert!(CssPercentageCalculation::try_from_components(original.components().clone()).is_err());
}

#[test]
fn selector_offsets_clone_the_authored_domain_and_preserve_shared_source_snapshots() {
    let source = "/* 😀 */\r\n@keyframes k{calc(25%){}to{opacity:1}}";
    let input = sheet(source);
    let selector = &keyframes(&input).blocks()[0].selectors().selectors()[0];
    let offset = selector.offset();
    let clone = offset.clone();
    assert_eq!(offset, clone);
    assert!(offset.literal_value().is_none());
    let original = offset.calculation().unwrap();
    let copied = clone.calculation().unwrap();
    let CssValueOrigin::Parsed(root) = original.origin() else {
        panic!("parsed math root");
    };
    let CssValueOrigin::Parsed(copied_root) = copied.origin() else {
        panic!("cloned math root");
    };
    assert!(root.source().same_snapshot(copied_root.source()));
    assert_eq!(root.source().as_str(), source);
    assert_eq!(
        root.span().start().byte_offset().value(),
        source.find("calc(").unwrap()
    );
    assert_eq!(root.span().start().line().value(), 1);
    assert_eq!(
        root.span().start().column().value(),
        "@keyframes k{".len() as u32
    );
    let CssComponentValueRef::Function(function) = original.components().items()[0].view() else {
        panic!("function root");
    };
    let CssValueOrigin::Parsed(leaf) = function.values().items()[0].origin() else {
        panic!("percentage leaf origin");
    };
    assert!(root.source().same_snapshot(leaf.source()));
    assert_eq!(
        leaf.span().start().byte_offset().value(),
        source.find("25%").unwrap()
    );
    assert_eq!(selector, &CssKeyframeSelector::Percent(offset.clone()));
    assert_eq!(
        CssKeyframeSelector::From.offset().literal_value(),
        Some(0.0)
    );
    assert_eq!(
        CssKeyframeSelector::To.offset().literal_value(),
        Some(100.0)
    );
    assert!(CssKeyframeSelector::From.offset().calculation().is_none());
}

#[test]
fn calculated_percentage_equality_preserves_authored_source_provenance() {
    let first = percentage("calc(25%)");
    let shifted = percentage("  calc(25%)");
    assert_ne!(first, shifted);
    assert_eq!(first, first.clone());
    assert_eq!(
        first
            .calculation()
            .unwrap()
            .components()
            .serialize()
            .unwrap()
            .as_css(),
        "calc(25%)"
    );
}

#[test]
fn calculated_selector_output_keeps_math_form_and_authored_out_of_range_coefficients() {
    let source = "@keyframes k{calc(120%){}calc(-10%){}min(25%,50%){}to{}}";
    let input = sheet(source);
    let before = input.clone();
    let expected = "@keyframes k { calc(120%) { } calc(-10%) { } calc(25%) { } 100% { } }";
    assert_eq!(input.to_specified_css().unwrap(), expected);
    assert_eq!(input, before);
    let reentered = validate_sheet(expected).unwrap();
    assert_eq!(reentered.to_specified_css().unwrap(), expected);
    assert_eq!(keyframes(&reentered).blocks().len(), 4);
    for block in &keyframes(&reentered).blocks()[..3] {
        assert!(
            block.selectors().selectors()[0]
                .offset()
                .literal_value()
                .is_none()
        );
        assert!(
            block.selectors().selectors()[0]
                .offset()
                .calculation()
                .is_some()
        );
    }
}

#[test]
fn normalization_preserves_calculated_selectors_duplicates_and_source_positions() {
    let input = sheet("@keyframes k{calc(25%),calc(25%){}calc(25%){}from{}}");
    let original = keyframes(&input);
    let normalized = normalize_sheet(&input).unwrap();
    let [CssNormalizedItem::Rule(context)] = normalized.items() else {
        panic!("terminal keyframes rule");
    };
    let CssRuleContextKindRef::Keyframes(retained) = context.kind() else {
        panic!("keyframes context");
    };
    assert_eq!(retained, original);
    assert_eq!(retained.blocks().len(), 3);
    assert_eq!(retained.blocks()[0].selectors().selectors().len(), 2);
    assert_eq!(
        retained.blocks()[0].selectors().selectors()[0].offset(),
        original.blocks()[0].selectors().selectors()[0].offset()
    );
}

#[test]
fn calculated_graph_output_shares_exact_cumulative_limits_and_remains_reusable_after_failures() {
    let input = sheet("@keyframes k{calc(120%){}calc(-10%){} }");
    let before = input.clone();
    let expected = "@keyframes k { calc(120%) { } calc(-10%) { } }";
    // Sheet/rule/name cost three. Each block/list/selector/declaration-list costs four.
    // Each calculation visits two input nodes and creates one projected percentage leaf.
    let exact = CssSpecifiedValueSerializationLimits::new(15, 13, expected.len());
    assert_eq!(input.to_specified_css_with_limits(exact).unwrap(), expected);
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(14, 13, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(15, 12, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(15, 13, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let error = input.to_specified_css_with_limits(limits).unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(kind)
        );
        assert_eq!(error.rule_index(), Some(0));
        assert_eq!(input, before);
        assert_eq!(input.to_specified_css_with_limits(exact).unwrap(), expected);
    }
    assert_eq!(input, before);
}

#[test]
fn infallible_string_names_preserve_unrestricted_decoded_contents() {
    for value in ["", " ", "\u{a0}", "none", "default", "Fade", "a\0b"] {
        let name = CssKeyframesString::new(value);
        assert_eq!(name.as_str(), value);
        assert_eq!(name, name.clone());
    }
    assert_ne!(
        CssKeyframesString::new("Fade"),
        CssKeyframesString::new("fade")
    );
}

#[test]
fn unrepresentable_constructed_string_name_fails_output_without_changing_identity() {
    let name = CssKeyframesString::new("a\0b");
    let names =
        CssAnimationNameList::try_new(vec![CssAnimationName::String(name.clone())]).unwrap();
    assert_eq!(
        names.serialize_specified().unwrap_err().kind(),
        CssSpecifiedValueSerializationErrorKind::UnrepresentableValue
    );
    assert!(matches!(names.names(), [CssAnimationName::String(retained)] if retained == &name));
    assert_eq!(name.as_str(), "a\0b");
}
