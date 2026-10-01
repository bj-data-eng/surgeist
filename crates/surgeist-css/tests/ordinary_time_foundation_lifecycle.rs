#![forbid(unsafe_code)]
//! Ordinary time grammar: Values 4 WD 2024-03-12 #time, #numeric-ranges,
//! #numbers and #calc-range; Transitions 1 WD 2026-01-08
//! #propdef-transition-duration, #propdef-transition-delay and #single-transition;
//! Animations 1 WD 2023-03-02 #animation-duration, #animation-delay and
//! #typedef-single-animation. Exact coefficient identity and structural aggregate
//! equality are authored-model contracts. Raw component/calculation origins remain
//! observable. These existing-API tests do not require a time serializer.

use surgeist_css::*;

const TIME_PROPERTIES: [CssKnownProperty; 6] = [
    CssKnownProperty::TransitionDuration,
    CssKnownProperty::TransitionDelay,
    CssKnownProperty::AnimationDuration,
    CssKnownProperty::AnimationDelay,
    CssKnownProperty::Transition,
    CssKnownProperty::Animation,
];

#[derive(Debug, PartialEq)]
enum TimingAggregate {
    Durations(CssDurationList),
    Delays(CssDelayList),
    Transitions(CssTransitionList),
    Animations(CssAnimationList),
}

fn aggregate(declaration: &CssDeclaration) -> TimingAggregate {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::TransitionDuration(value) => {
            TimingAggregate::Durations(value.durations().clone())
        }
        CssKnownPropertyValueRef::AnimationDuration(value) => {
            TimingAggregate::Durations(value.durations().clone())
        }
        CssKnownPropertyValueRef::TransitionDelay(value) => {
            TimingAggregate::Delays(value.delays().clone())
        }
        CssKnownPropertyValueRef::AnimationDelay(value) => {
            TimingAggregate::Delays(value.delays().clone())
        }
        CssKnownPropertyValueRef::Transition(value) => {
            TimingAggregate::Transitions(value.transitions().clone())
        }
        CssKnownPropertyValueRef::Animation(value) => {
            TimingAggregate::Animations(value.animations().clone())
        }
        other => panic!("typed time property: {other:?}"),
    }
}

fn authored(declaration: &CssDeclaration) -> &str {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::TransitionDuration(value) => value.as_css(),
        CssKnownPropertyValueRef::AnimationDuration(value) => value.as_css(),
        CssKnownPropertyValueRef::TransitionDelay(value) => value.as_css(),
        CssKnownPropertyValueRef::AnimationDelay(value) => value.as_css(),
        CssKnownPropertyValueRef::Transition(value) => value.as_css(),
        CssKnownPropertyValueRef::Animation(value) => value.as_css(),
        other => panic!("authored time property: {other:?}"),
    }
}

fn value(property: CssKnownProperty, coefficient: &str, delay_slot: bool) -> String {
    match (property, delay_slot) {
        (CssKnownProperty::Transition, false) => format!("opacity {coefficient}"),
        (CssKnownProperty::Transition, true) => format!("opacity 2s {coefficient}"),
        (CssKnownProperty::Animation, false) => format!("fade {coefficient}"),
        (CssKnownProperty::Animation, true) => format!("fade 2s {coefficient}"),
        _ => coefficient.to_owned(),
    }
}

fn slots(property: CssKnownProperty) -> &'static [bool] {
    if matches!(
        property,
        CssKnownProperty::Transition | CssKnownProperty::Animation
    ) {
        &[false, true]
    } else {
        &[false]
    }
}

fn declaration(property: CssKnownProperty, body: &str, prefix: &str) -> CssDeclaration {
    let source = format!("{prefix}{}:{body}", property.canonical_name());
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(
        validate_style_attribute(&source),
        Ok(report.syntax().clone())
    );
    let declaration = report.syntax().last().expect("retained time declaration");
    assert_eq!(declaration.known().unwrap().property(), property);
    assert_eq!(authored(declaration), body);
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        body
    );
    declaration.clone()
}

fn huge_ordinary_times_are_clean(property: CssKnownProperty) {
    let mut failures = Vec::new();
    for &delay_slot in slots(property) {
        let body = value(property, "1e999s", delay_slot);
        let source = format!("{}:{body}", property.canonical_name());
        let report = parse_style_attribute(&source);
        if !report.is_clean() {
            failures.push(format!("{source}: {:?}", report.diagnostics()));
            continue;
        }
        let parsed = declaration(property, &body, "");
        let components = parse_component_values(&body).unwrap();
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                CssImportance::Normal,
            ),
            parse_property_value_for_grammar(
                property.grammar(),
                components.clone(),
                CssImportance::Normal,
            ),
        ] {
            let checked = result.expect("huge finite ordinary time through checked grammar");
            assert_eq!(checked.value_components(), &components);
            assert_eq!(authored(&checked), body);
            assert_eq!(aggregate(&parsed), aggregate(&checked));
        }
    }
    assert!(
        failures.is_empty(),
        "finite ordinary time must be admitted exactly:\n{}",
        failures.join("\n")
    );
}

fn distinct_coefficients_remain_distinct(property: CssKnownProperty, first: &str, second: &str) {
    let mut collapsed = Vec::new();
    for &delay_slot in slots(property) {
        let first_body = value(property, first, delay_slot);
        let second_body = value(property, second, delay_slot);
        let first = declaration(property, &first_body, "");
        let second = declaration(property, &second_body, "");
        assert_ne!(first.value_components(), second.value_components());
        if aggregate(&first) == aggregate(&second) {
            collapsed.push(format!(
                "{}: {first_body:?} equals {second_body:?}",
                property.canonical_name()
            ));
        }
    }
    assert!(
        collapsed.is_empty(),
        "distinct authored coefficients collapsed in typed aggregates:\n{}",
        collapsed.join("\n")
    );
}

fn raw_calculations(declaration: &CssDeclaration) -> Vec<CssTimeCalculation> {
    declaration
        .value_components()
        .items()
        .iter()
        .filter_map(|component| {
            let CssComponentValueRef::Function(function) = component.view() else {
                return None;
            };
            if function.name() != "calc" {
                return None;
            }
            Some(
                CssTimeCalculation::try_from_components(
                    CssComponentValues::try_new(vec![component.clone()]).unwrap(),
                )
                .unwrap(),
            )
        })
        .collect()
}

fn aggregate_identity_ignores_calculation_origins(property: CssKnownProperty) {
    let body = match property {
        CssKnownProperty::Transition => "opacity calc(1s + 2s) calc(-1s)",
        CssKnownProperty::Animation => "fade calc(1s + 2s) calc(-1s)",
        _ => "calc(1s + 2s), calc(-1s)",
    };
    let first = declaration(property, body, "");
    let second = declaration(property, body, "color:red;");
    let first_raw = raw_calculations(&first);
    let second_raw = raw_calculations(&second);
    assert_eq!(first_raw.len(), 2);
    assert_eq!(second_raw.len(), 2);
    for (left, right) in first_raw.iter().zip(&second_raw) {
        assert_ne!(
            left.origin(),
            right.origin(),
            "original source origins differ"
        );
        assert_ne!(left, right, "raw calculation equality includes origins");
        assert_ne!(left.components(), right.components());
        assert_eq!(
            left.serialize().unwrap().as_css(),
            right.serialize().unwrap().as_css()
        );
    }
    assert!(
        aggregate(&first) == aggregate(&second),
        "{}: time aggregate identity ignores differing source offsets while raw calculations differ",
        property.canonical_name()
    );
}

fn concatenate(parts: &[CssComponentValues]) -> CssComponentValues {
    CssComponentValues::try_new(
        parts
            .iter()
            .flat_map(|part| part.items().iter().cloned())
            .collect(),
    )
    .unwrap()
}

fn recovered_function(source: &str) -> (CssComponentValues, CssValueOrigin) {
    let components = parse_component_values(source).unwrap();
    let CssComponentValueRef::Function(function) = components.items()[0].view() else {
        panic!("recovered function fixture")
    };
    let origin = function.closing_origin().clone();
    let CssValueOrigin::ImplicitClosure { opening, at } = &origin else {
        panic!("original missing function closure")
    };
    assert_eq!(opening.span().start().byte_offset().value(), 0);
    assert_eq!(at.source().as_str(), source);
    assert_eq!(at.span().start().byte_offset().value(), source.len());
    assert_eq!(at.span().start(), at.span().end());
    (components, origin)
}

fn constructors_reject_first_original_closure(property: CssKnownProperty) {
    let (nested, nested_origin) = recovered_function("calc((1s");
    let (pending, pending_origin) = recovered_function("var(--time");
    let (late, late_origin) = recovered_function("calc(-1s");
    let prefix = match property {
        CssKnownProperty::Transition => "opacity 1s ",
        CssKnownProperty::Animation => "fade 1s ",
        _ => "1s, ",
    };
    let cases = [
        ("nested math", nested.clone(), nested_origin.clone()),
        ("pending envelope", pending.clone(), pending_origin.clone()),
        (
            "CSS-wide envelope with later recovery",
            concatenate(&[parse_component_values("initial ").unwrap(), late.clone()]),
            late_origin.clone(),
        ),
        (
            "later time child",
            concatenate(&[parse_component_values(prefix).unwrap(), late.clone()]),
            late_origin.clone(),
        ),
        (
            "first of two mixed-source closures",
            concatenate(&[pending, parse_component_values(" ").unwrap(), nested]),
            pending_origin,
        ),
    ];
    let mut failures = Vec::new();
    for (label, components, first_origin) in cases {
        let before = components.clone();
        for (front_door, result) in [
            (
                "property",
                parse_property_value(
                    CssPropertyNameRef::Known(property),
                    components.clone(),
                    CssImportance::Important,
                ),
            ),
            (
                "grammar",
                parse_property_value_for_grammar(
                    property.grammar(),
                    components.clone(),
                    CssImportance::Important,
                ),
            ),
        ] {
            match result {
                Err(error) if matches!(error.kind(), CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_)))
                    && error.origin() == &CssSerializedOrigin::End(Some(first_origin.clone())) => {},
                Err(error) => failures.push(format!("{front_door}/{label}: expected UnexpectedEnd at {first_origin:?}; actual {:?} at {:?}", error.kind(), error.origin())),
                Ok(_) => failures.push(format!("{front_door}/{label}: expected UnexpectedEnd at {first_origin:?}; actual accepted declaration")),
            }
            assert_eq!(
                components, before,
                "construction leaves original components unchanged"
            );
        }
    }
    assert!(
        failures.is_empty(),
        "{} must reject the first original closure:\n{}",
        property.canonical_name(),
        failures.join("\n")
    );
}

macro_rules! time_property_tests {
    ($module:ident, $property:ident, $tiny_first:literal, $tiny_second:literal) => {
        mod $module {
            use super::*;
            const PROPERTY: CssKnownProperty = CssKnownProperty::$property;
            #[test]
            fn huge_finite_ordinary_time_is_clean() {
                huge_ordinary_times_are_clean(PROPERTY);
            }
            #[test]
            fn signed_exponent_zero_is_clean() {
                for &delay_slot in slots(PROPERTY) {
                    declaration(PROPERTY, &value(PROPERTY, "-0e999s", delay_slot), "");
                }
            }
            #[test]
            fn neighboring_precise_coefficients_have_distinct_typed_identity() {
                distinct_coefficients_remain_distinct(
                    PROPERTY,
                    "0.10000000000000000001s",
                    "0.10000000000000000002s",
                );
            }
            #[test]
            fn tiny_nonzero_coefficients_have_distinct_typed_identity() {
                distinct_coefficients_remain_distinct(PROPERTY, $tiny_first, $tiny_second);
            }
            #[test]
            fn calculation_aggregate_identity_ignores_original_origins() {
                aggregate_identity_ignores_calculation_origins(PROPERTY);
            }
            #[test]
            fn both_constructors_reject_first_original_closure() {
                constructors_reject_first_original_closure(PROPERTY);
            }
            #[test]
            fn lexical_coefficient_spelling_remains_distinct() {
                distinct_coefficients_remain_distinct(PROPERTY, "1s", "1.0s");
            }
        }
    };
}

time_property_tests!(
    transition_duration,
    TransitionDuration,
    "1e-999s",
    "2e-999s"
);
time_property_tests!(animation_duration, AnimationDuration, "1e-999s", "2e-999s");
time_property_tests!(transition_delay, TransitionDelay, "-1e-999s", "-2e-999s");
time_property_tests!(animation_delay, AnimationDelay, "-1e-999s", "-2e-999s");
time_property_tests!(transition, Transition, "1e-999s", "2e-999s");
time_property_tests!(animation, Animation, "1e-999s", "2e-999s");

fn assert_invalid_duration(property: CssKnownProperty, coefficient: &str) {
    let body = value(property, coefficient, false);
    let source = format!("{}:{body};color:red", property.canonical_name());
    let report = parse_style_attribute(&source);
    let [retained] = report.syntax().as_slice() else {
        panic!(
            "invalid duration dropped; only color remains: {source}: actual declaration count {}",
            report.syntax().len()
        )
    };
    assert_eq!(
        retained.known().unwrap().property(),
        CssKnownProperty::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one invalid duration diagnostic: {source}: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("property grammar error")
    };
    assert_eq!(detail.property(), property);
    assert_eq!(
        validate_style_attribute(&source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
}

macro_rules! negative_duration_test {
    ($name:ident, $property:ident) => {
        #[test]
        fn $name() {
            assert_invalid_duration(CssKnownProperty::$property, "-1e-999s");
        }
    };
}
negative_duration_test!(
    tiny_negative_transition_duration_is_invalid,
    TransitionDuration
);
negative_duration_test!(
    tiny_negative_animation_duration_is_invalid,
    AnimationDuration
);
negative_duration_test!(
    tiny_negative_transition_first_time_is_not_reinterpreted_as_delay,
    Transition
);
negative_duration_test!(
    tiny_negative_animation_first_time_is_not_reinterpreted_as_delay,
    Animation
);

#[test]
fn ordinary_range_malformed_unit_and_trailing_junk_controls() {
    for property in [
        CssKnownProperty::TransitionDuration,
        CssKnownProperty::AnimationDuration,
        CssKnownProperty::Transition,
        CssKnownProperty::Animation,
    ] {
        assert_invalid_duration(property, "-1s");
        for invalid in ["0", "1px", "1s junk junk", "1s,"] {
            // Bare zero may occupy animation's iteration slot. Occupy that slot
            // first so the remaining Number must be rejected as a time.
            let body = if property == CssKnownProperty::Animation && invalid == "0" {
                "fade 2 0".to_owned()
            } else {
                value(property, invalid, false)
            };
            let source = format!("{}:{body}", property.canonical_name());
            let report = parse_style_attribute(&source);
            assert!(report.syntax().is_empty(), "{source}");
            assert!(!report.is_clean(), "{source}");
            assert!(validate_style_attribute(&source).is_err(), "{source}");
        }
    }
    for property in [
        CssKnownProperty::TransitionDelay,
        CssKnownProperty::AnimationDelay,
    ] {
        for invalid in ["0", "1hz", "1s junk", "1s,"] {
            let source = format!("{}:{invalid}", property.canonical_name());
            let report = parse_style_attribute(&source);
            assert!(report.syntax().is_empty(), "{source}");
            assert!(!report.is_clean(), "{source}");
            assert!(validate_style_attribute(&source).is_err());
        }
    }
}

#[test]
fn zero_signed_delay_and_negative_calculation_controls_are_clean() {
    for property in TIME_PROPERTIES {
        for coefficient in ["0s", "-0s", "0e-999ms", "calc(-1s)", "calc(1s + 2s)"] {
            declaration(property, &value(property, coefficient, false), "");
        }
        if matches!(
            property,
            CssKnownProperty::TransitionDelay | CssKnownProperty::AnimationDelay
        ) {
            declaration(property, "-250ms", "");
            declaration(property, "-1e-999s", "");
        } else if matches!(
            property,
            CssKnownProperty::Transition | CssKnownProperty::Animation
        ) {
            declaration(property, &value(property, "-250ms", true), "");
            declaration(property, &value(property, "-1e-999s", true), "");
        }
    }
}

#[test]
fn two_time_shorthands_assign_duration_then_delay_and_preserve_omissions() {
    for property in [CssKnownProperty::Transition, CssKnownProperty::Animation] {
        let source = declaration(property, &value(property, "-250ms", true), "");
        let one_time = declaration(property, &value(property, "2s", false), "");
        match (aggregate(&source), aggregate(&one_time)) {
            (TimingAggregate::Transitions(two), TimingAggregate::Transitions(one)) => {
                let two = &two.values()[0];
                let one = &one.values()[0];
                assert_eq!(two.duration(), one.duration());
                assert!(two.delay().is_some());
                assert!(one.delay().is_none());
                assert!(two.timing_function().is_none());
                assert_eq!(
                    TimingAggregate::Delays(
                        CssDelayList::try_new(vec![two.delay().unwrap().clone()]).unwrap()
                    ),
                    aggregate(&declaration(
                        CssKnownProperty::TransitionDelay,
                        "-250ms",
                        ""
                    ))
                );
            }
            (TimingAggregate::Animations(two), TimingAggregate::Animations(one)) => {
                let two = &two.values()[0];
                let one = &one.values()[0];
                assert_eq!(two.duration(), one.duration());
                assert!(two.delay().is_some());
                assert!(one.delay().is_none());
                assert!(two.timing_function().is_none());
                assert!(two.iteration_count().is_none());
                assert_eq!(
                    TimingAggregate::Delays(
                        CssDelayList::try_new(vec![two.delay().unwrap().clone()]).unwrap()
                    ),
                    aggregate(&declaration(CssKnownProperty::AnimationDelay, "-250ms", ""))
                );
            }
            _ => panic!("shorthand models"),
        }
    }
}

#[test]
fn parsed_recovered_math_is_retained_with_non_clean_validation() {
    for property in TIME_PROPERTIES {
        let body = value(property, "calc(-1s", false);
        let source = format!("{}:{body}", property.canonical_name());
        let report = parse_style_attribute(&source);
        assert_eq!(
            report.syntax().len(),
            1,
            "{source}: {:?}",
            report.diagnostics()
        );
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(
                    |diagnostic| diagnostic.error().code() == CssErrorCode::UnexpectedEnd
                        && diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
                )
        );
        assert_eq!(
            validate_style_attribute(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn timing_substitution_remains_unsupported_for_terminal_expansion() {
    for property in TIME_PROPERTIES {
        let source = format!("{}:var(--time)", property.canonical_name());
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(
            expand_declaration(&report.syntax()[0]).unwrap_err().kind(),
            &CssExpansionErrorKind::UnsupportedProperty(property)
        );
    }
}

#[test]
fn animation_component_assembly_compares_time_children_without_provenance() {
    let body = "fade calc(1s + 2s) calc(-1s)";
    let first = declaration(CssKnownProperty::Animation, body, "");
    let second = declaration(CssKnownProperty::Animation, body, "color:red;");
    let components = |declaration: &CssDeclaration| {
        let TimingAggregate::Animations(list) = aggregate(declaration) else {
            panic!("animation")
        };
        let animation = &list.values()[0];
        CssAnimationComponents {
            name: animation.name().cloned(),
            duration: animation.duration().cloned(),
            delay: animation.delay().cloned(),
            ..CssAnimationComponents::default()
        }
    };
    assert_ne!(raw_calculations(&first), raw_calculations(&second));
    assert!(
        components(&first) == components(&second),
        "animation component assembly ignores time origins"
    );
}

#[test]
fn authored_units_and_literal_math_branches_remain_distinct() {
    for property in TIME_PROPERTIES {
        for (first, second) in [("1s", "1000ms"), ("1s", "calc(1s)")] {
            distinct_coefficients_remain_distinct(property, first, second);
        }
    }
}

#[test]
fn shorthand_signed_tiny_delays_have_distinct_typed_identity() {
    let mut collapsed = Vec::new();
    for property in [CssKnownProperty::Transition, CssKnownProperty::Animation] {
        let first_body = value(property, "-1e-999s", true);
        let second_body = value(property, "-2e-999s", true);
        let first = declaration(property, &first_body, "");
        let second = declaration(property, &second_body, "");
        assert_ne!(first.value_components(), second.value_components());
        if aggregate(&first) == aggregate(&second) {
            collapsed.push(property.canonical_name());
        }
    }
    assert!(
        collapsed.is_empty(),
        "signed tiny delays collapsed: {collapsed:?}"
    );
}

#[test]
fn non_time_iteration_and_easing_controls_preserve_exact_values_and_ranges() {
    let source = "animation-iteration-count:2.5,calc(1 + 1),infinite;transition-timing-function:cubic-bezier(0.1,-2,0.9,3),steps(2,jump-none)";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        validate_style_attribute(source),
        Ok(report.syntax().clone())
    );
    let CssKnownPropertyValueRef::AnimationIterationCount(counts) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("iteration")
    };
    assert_eq!(counts.iteration_counts().values().len(), 3);
    let CssAnimationIterationCount::Number(number) = &counts.iteration_counts().values()[0] else {
        panic!("literal iteration")
    };
    assert_eq!(number.serialize_specified().unwrap(), "2.5");
    assert!(matches!(
        counts.iteration_counts().values()[1],
        CssAnimationIterationCount::Number(ref value) if value.calculation().is_some()
    ));
    assert!(matches!(
        counts.iteration_counts().values()[2],
        CssAnimationIterationCount::Infinite
    ));
    let CssKnownPropertyValueRef::TransitionTimingFunction(easing) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("easing")
    };
    assert!(matches!(
        easing.timing_functions().values(),
        [CssEasing::CubicBezier(_), CssEasing::Steps(_)]
    ));
    for source in [
        "animation-iteration-count:-1e-999",
        "transition-timing-function:cubic-bezier(-0.1,0,0.9,1)",
        "transition-timing-function:steps(1,jump-none)",
    ] {
        assert!(validate_style_attribute(source).is_err(), "{source}");
    }
}

// Functional evidence for the adopted public time API, appended after the immutable RED prefix.
mod construction {
    use std::error::Error as _;
    use surgeist_css::*;

    fn literal(number: &str, unit: CssTimeUnit) -> CssTimeLiteral {
        CssTimeLiteral::try_new(number, unit).unwrap()
    }
    fn time(number: &str, unit: CssTimeUnit) -> CssTimeValue {
        CssTimeValue::from_literal(literal(number, unit))
    }
    fn calculation(source: &str) -> CssTimeCalculation {
        CssTimeCalculation::try_from_components(parse_component_values(source).unwrap()).unwrap()
    }
    fn parsed_duration(source: &str) -> CssDuration {
        let report = parse_style_attribute(source);
        let wrapper = report
            .syntax()
            .iter()
            .find_map(|declaration| match declaration.known()?.property_value()? {
                CssKnownPropertyValueRef::TransitionDuration(wrapper) => Some(wrapper),
                _ => None,
            })
            .expect("duration fixture");
        wrapper.durations().values()[0].clone()
    }

    #[test]
    fn literal_construction_retains_exact_units_tokens_and_borrowed_origins() {
        for (source, unit, number, kind) in [
            (
                "+1S",
                CssTimeUnit::Seconds,
                "+1",
                CssNumericTokenKind::Integer,
            ),
            (
                "1.0mS",
                CssTimeUnit::Milliseconds,
                "1.0",
                CssNumericTokenKind::Number,
            ),
            (
                "-1e-999s",
                CssTimeUnit::Seconds,
                "-1e-999",
                CssNumericTokenKind::Number,
            ),
            (
                "1e999ms",
                CssTimeUnit::Milliseconds,
                "1e999",
                CssNumericTokenKind::Number,
            ),
            (
                "1m\\73",
                CssTimeUnit::Milliseconds,
                "1",
                CssNumericTokenKind::Integer,
            ),
        ] {
            let components = parse_component_values(source).unwrap();
            let component = components.items()[0].clone();
            let value = CssTimeLiteral::try_from_component(component.clone()).unwrap();
            assert_eq!(value.component(), &component);
            assert_eq!(value.origin(), component.origin());
            assert_eq!(value.unit(), unit);
            assert_eq!(value.numeric().representation(), number);
            assert_eq!(value.numeric().kind(), kind);
        }
        for number in ["1e999", "-1e-999", "1.234567890123456789"] {
            assert_eq!(
                literal(number, CssTimeUnit::Milliseconds)
                    .numeric()
                    .representation(),
                number
            );
        }
    }

    #[test]
    fn intrinsic_literal_errors_retain_the_responsible_component_origin() {
        for source in ["0", "1px", "1%", "infinity", "NaN", "\"1s\"", "calc(1s)"] {
            let components = parse_component_values(source).unwrap();
            let component = components.items()[0].clone();
            let error = CssTimeLiteral::try_from_component(component.clone()).unwrap_err();
            assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
            assert_eq!(error.origin(), component.origin());
        }
        for invalid in ["", "NaN", "infinity", "1 2", "1s"] {
            assert!(CssTimeLiteral::try_new(invalid, CssTimeUnit::Seconds).is_err());
        }
    }

    #[test]
    fn exact_calculation_convenience_normalizes_only_an_ordinary_root() {
        let raw = CssTimeCalculation::try_literal("-1e999", CssTimeUnit::Milliseconds).unwrap();
        let root_component = raw.components().items()[0].clone();
        let value = CssTimeValue::try_from_calculation(raw).unwrap();
        assert!(value.calculation().is_none());
        assert_eq!(value.literal().unwrap().component(), &root_component);
        for source in ["calc(1s)", "calc((1s))", "min(1s, 2s)"] {
            let raw = calculation(source);
            let origin = raw.origin().clone();
            let value = CssTimeValue::try_from_calculation(raw.clone()).unwrap();
            assert!(value.literal().is_none(), "{source}");
            assert_eq!(value.calculation(), Some(&raw));
            assert_eq!(value.origin(), &origin);
        }
        for source in ["1", "calc(1px)", "calc(1%)"] {
            let error =
                CssTimeCalculation::try_from_components(parse_component_values(source).unwrap())
                    .unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNumericConstructionErrorKind::RootDomainMismatch
            );
        }
    }

    #[test]
    fn duration_checks_the_exact_nonzero_sign_without_projecting_calculations() {
        for number in ["-1", "-1e-999", "-0.00000000000000001", "-1e999"] {
            let value = time(number, CssTimeUnit::Milliseconds);
            let origin = value.origin().clone();
            let error = CssDuration::try_new(value).unwrap_err();
            assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
            assert_eq!(error.origin(), Some(&origin));
            assert_eq!(error.path(), None);
            assert!(error.source().is_none());
        }
        for number in ["-0", "-0e999", "0e-999", "1e-999", "1e999"] {
            let value = time(number, CssTimeUnit::Seconds);
            let origin = value.origin().clone();
            let duration = CssDuration::try_new(value.clone()).unwrap();
            assert_eq!(duration.time(), &value);
            assert_eq!(duration.origin(), &origin);
        }
        let raw = calculation("calc(-1s)");
        let duration =
            CssDuration::try_new(CssTimeValue::try_from_calculation(raw.clone()).unwrap()).unwrap();
        assert_eq!(duration.time().calculation(), Some(&raw));
        assert_eq!(duration.serialize_specified().unwrap(), "calc(-1s)");
    }

    #[test]
    fn strict_admission_and_aggregate_construction_reject_original_recovery() {
        let source = "transition-duration: calc(1s";
        let report = parse_style_attribute(source);
        assert!(!report.is_clean());
        assert!(validate_style_attribute(source).is_err());
        let duration = parsed_duration(source);
        let raw = duration.time().calculation().unwrap().clone();
        let component = raw.components().items()[0].clone();
        let CssComponentValueRef::Function(function) = component.view() else {
            panic!("calc");
        };
        let closing = function.closing_origin().clone();
        assert!(matches!(closing, CssValueOrigin::ImplicitClosure { .. }));
        let error = CssTimeValue::try_from_calculation(raw.clone()).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RecoveredComponent
        );
        assert_eq!(error.origin(), Some(&closing));
        assert_eq!(error.path(), None);
        assert!(error.source().is_none());
        let error = CssDuration::try_new(duration.time().clone()).unwrap_err();
        assert_eq!(error.origin(), Some(&closing));
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RecoveredComponent
        );
        assert!(CssDurationList::try_new(vec![duration.clone()]).is_none());
        assert!(CssDelayList::try_new(vec![duration.time().clone()]).is_none());
        assert!(CssTransition::try_new(None, Some(duration.clone()), None, None).is_none());
        assert!(CssTransition::try_new(None, None, Some(duration.time().clone()), None).is_none());
        assert!(
            CssAnimation::try_new(CssAnimationComponents {
                duration: Some(duration.clone()),
                ..Default::default()
            })
            .is_none()
        );
        assert!(
            CssAnimation::try_new(CssAnimationComponents {
                delay: Some(duration.time().clone()),
                ..Default::default()
            })
            .is_none()
        );
        let report = parse_style_attribute("transition: opacity calc(1s");
        let CssKnownPropertyValueRef::Transition(wrapper) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("transition recovery");
        };
        assert!(CssTransitionList::try_new(wrapper.transitions().values().to_vec()).is_none());
        let report = parse_style_attribute("animation: fade calc(1s");
        let CssKnownPropertyValueRef::Animation(wrapper) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("animation recovery");
        };
        assert!(CssAnimationList::try_new(wrapper.animations().values().to_vec()).is_none());
        // Observation of recovered syntax is not clean construction.
        assert_eq!(raw.serialize().unwrap().as_css(), "calc(1s)");
    }

    #[test]
    fn raw_identity_and_semantic_timing_identity_have_distinct_origin_policies() {
        let a = parsed_duration("transition-duration: 1s");
        let b = parsed_duration("color:red; transition-duration: 1s");
        assert_ne!(a.time(), b.time());
        assert_ne!(a.time().literal(), b.time().literal());
        assert_ne!(a.origin(), b.origin());
        assert_eq!(a, b);
        assert_eq!(
            CssDurationList::try_new(vec![a.clone()]),
            CssDurationList::try_new(vec![b.clone()])
        );
        assert_eq!(
            CssDelayList::try_new(vec![a.time().clone()]),
            CssDelayList::try_new(vec![b.time().clone()])
        );
        assert_eq!(
            CssTransition::try_new(None, Some(a.clone()), Some(a.time().clone()), None),
            CssTransition::try_new(None, Some(b.clone()), Some(b.time().clone()), None)
        );
        assert_eq!(
            CssAnimationComponents {
                duration: Some(a.clone()),
                delay: Some(a.time().clone()),
                ..Default::default()
            },
            CssAnimationComponents {
                duration: Some(b.clone()),
                delay: Some(b.time().clone()),
                ..Default::default()
            }
        );
        for source in [
            "transition-duration: 1.0s",
            "transition-duration: 1000ms",
            "transition-duration: calc(1s)",
        ] {
            assert_ne!(a, parsed_duration(source));
        }
    }

    #[test]
    fn aggregate_construction_preserves_empty_rejection_and_explicit_zero_omissions() {
        assert!(CssDurationList::try_new(vec![]).is_none());
        assert!(CssDelayList::try_new(vec![]).is_none());
        assert!(CssTransition::try_new(None, None, None, None).is_none());
        assert!(CssAnimation::try_new(Default::default()).is_none());
        let zero = CssDuration::try_new(time("-0e999", CssTimeUnit::Milliseconds)).unwrap();
        let transition = CssTransition::try_new(None, Some(zero.clone()), None, None).unwrap();
        assert_eq!(transition.duration(), Some(&zero));
        assert!(transition.delay().is_none());
        assert!(transition.property().is_none());
        assert!(transition.timing_function().is_none());
        assert_eq!(
            zero.time().literal().unwrap().numeric().representation(),
            "-0e999"
        );
        assert_ne!(
            transition,
            CssTransition::try_new(None, None, Some(time("0", CssTimeUnit::Seconds)), None)
                .unwrap()
        );
    }

    #[test]
    fn ordinary_emission_rounds_after_unit_conversion_and_preserves_raw_precision() {
        for (number, unit, expected) in [
            ("1000", CssTimeUnit::Milliseconds, "1s"),
            ("1", CssTimeUnit::Milliseconds, "0.001s"),
            (
                "1.234567890123456789",
                CssTimeUnit::Milliseconds,
                "0.001235s",
            ),
            ("-250", CssTimeUnit::Milliseconds, "-0.25s"),
            ("0.0001", CssTimeUnit::Milliseconds, "0s"),
            ("1e20", CssTimeUnit::Milliseconds, "100000000000000000s"),
            ("1e-12", CssTimeUnit::Milliseconds, "0s"),
        ] {
            let value = literal(number, unit);
            let before = value.clone();
            assert_eq!(value.serialize_specified().unwrap(), expected);
            assert_eq!(value.numeric().representation(), number);
            assert_eq!(value.unit(), unit);
            assert_eq!(value, before);
            assert_eq!(
                CssTimeValue::from_literal(value.clone())
                    .serialize_specified()
                    .unwrap(),
                expected
            );
        }
        // CSSOM rounds emitted ordinary text; exact authored zero spelling remains retained.
        for unit in [CssTimeUnit::Seconds, CssTimeUnit::Milliseconds] {
            for number in ["-0", "+0e999", "-0e-999"] {
                let value = literal(number, unit);
                assert_eq!(value.serialize_specified().unwrap(), "0s");
                assert_eq!(value.numeric().representation(), number);
                assert_eq!(value.unit(), unit);
            }
        }
    }

    #[test]
    fn ordinary_and_calculation_resource_boundaries_are_independently_counted() {
        use CssSpecifiedValueSerializationErrorKind as K;
        use CssSpecifiedValueSerializationLimits as L;
        let ordinary = time("1", CssTimeUnit::Milliseconds);
        let ordinary_literal = ordinary.literal().unwrap().clone();
        let ordinary_duration = CssDuration::try_new(ordinary.clone()).unwrap();
        assert_eq!(
            ordinary_literal
                .serialize_specified_with_limits(L::new(1, 1, 6))
                .unwrap(),
            "0.001s"
        );
        assert_eq!(
            ordinary_duration
                .serialize_specified_with_limits(L::new(1, 1, 6))
                .unwrap(),
            "0.001s"
        );
        assert_eq!(
            ordinary
                .serialize_specified_with_limits(L::new(1, 1, 6))
                .unwrap(),
            "0.001s"
        );
        for (limits, kind) in [
            (L::new(0, 1, 6), K::InputNodeLimit),
            (L::new(1, 0, 6), K::ProjectionNodeLimit),
            (L::new(1, 1, 5), K::ByteLimit),
        ] {
            let before = ordinary.clone();
            assert_eq!(
                ordinary
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(
                ordinary_literal
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(
                ordinary_duration
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(ordinary, before);
        }
        // Calc + Sum + two leaves: 4 inputs; two scalar leaves + one combined scalar: 3 projections.
        let value = CssTimeValue::try_from_calculation(calculation("calc(1s + 2s)")).unwrap();
        let duration = CssDuration::try_new(value.clone()).unwrap();
        assert_eq!(
            duration
                .serialize_specified_with_limits(L::new(4, 3, 8))
                .unwrap(),
            "calc(3s)"
        );
        for (limits, kind) in [
            (L::new(3, 3, 8), K::InputNodeLimit),
            (L::new(4, 2, 8), K::ProjectionNodeLimit),
            (L::new(4, 3, 7), K::ByteLimit),
        ] {
            let before = value.clone();
            let raw_before = value.calculation().unwrap().serialize().unwrap();
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(
                duration
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(value, before);
            assert_eq!(
                value.calculation().unwrap().serialize().unwrap().as_css(),
                raw_before.as_css()
            );
        }
        assert_eq!(
            time("1", CssTimeUnit::Seconds)
                .serialize_specified_with_limits(L::new(1, 1, 2))
                .unwrap(),
            "1s"
        );
    }

    #[test]
    fn large_decimal_expansion_fails_atomically_without_a_scientific_fallback() {
        let limits = CssSpecifiedValueSerializationLimits::new(1, 1, 32);
        for (number, tiny) in [
            ("1e999", false),
            ("1e-999", true),
            ("1e999999999999999999999999999999999999999999", false),
        ] {
            let value = literal(number, CssTimeUnit::Milliseconds);
            let before = value.clone();
            if tiny {
                assert_eq!(value.serialize_specified_with_limits(limits).unwrap(), "0s");
            } else {
                assert_eq!(
                    value
                        .serialize_specified_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    CssSpecifiedValueSerializationErrorKind::ByteLimit
                );
            }
            assert_eq!(value, before);
            assert_eq!(value.numeric().representation(), number);
        }
    }

    #[test]
    fn calculation_emission_keeps_existing_simplification_and_unfinished_projection() {
        for (source, expected) in [
            ("calc(1000ms + 1s)", "calc(2s)"),
            ("calc(1e999s)", "calc(infinity * 1s)"),
            ("calc(1e-999s)", "calc(0s)"),
        ] {
            let value = CssTimeValue::try_from_calculation(calculation(source)).unwrap();
            let before = value.clone();
            assert_eq!(value.serialize_specified().unwrap(), expected);
            assert_eq!(value, before);
        }
        let value = CssTimeValue::try_from_calculation(calculation("min(1s, 2s)")).unwrap();
        assert_eq!(value.serialize_specified().unwrap(), "calc(1s)");
        assert!(value.calculation().is_some());
        for (source, kind) in [
            ("(1s)", CssNumericConstructionErrorKind::RootDomainMismatch),
            (
                "calc(sibling-index() * 1s)",
                CssNumericConstructionErrorKind::UnknownFunction,
            ),
        ] {
            assert_eq!(
                CssTimeCalculation::try_from_components(parse_component_values(source).unwrap())
                    .unwrap_err()
                    .kind(),
                &kind
            );
        }
    }
    #[test]
    fn checked_negative_duration_maps_to_the_original_later_mixed_source_token() {
        let negative = parse_component_values("-1e-999s").unwrap().items()[0].clone();
        let mut items = parse_component_values("1s, ").unwrap().items().to_vec();
        items.push(negative.clone());
        let components = CssComponentValues::try_new(items).unwrap();
        let before = components.clone();
        let error = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::TransitionDuration),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap_err();
        assert_eq!(
            error.origin(),
            &CssSerializedOrigin::Token(negative.origin().clone())
        );
        assert_eq!(components, before);
    }
}
