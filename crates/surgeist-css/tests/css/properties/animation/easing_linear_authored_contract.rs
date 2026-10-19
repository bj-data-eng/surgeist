#![forbid(unsafe_code)]
//! Transitions 1 WD 2026-01-08 imports Easing 2 WD 2024-08-29 §2.1.1:
//! Number && Percentage{1,2}?, with at least two authored stops (§2.1.2).
//! These specified-value contracts preserve omissions and symbolic inputs;
//! creation/fixup and computed serialization require the later numeric phase.

use surgeist_css::*;

const LONGHANDS: [CssKnownProperty; 2] = [
    CssKnownProperty::TransitionTimingFunction,
    CssKnownProperty::AnimationTimingFunction,
];
const CONSUMERS: [CssKnownProperty; 4] = [
    CssKnownProperty::TransitionTimingFunction,
    CssKnownProperty::AnimationTimingFunction,
    CssKnownProperty::Transition,
    CssKnownProperty::Animation,
];
const TRANSITION_MEMBERS: [CssKnownProperty; 4] = [
    CssKnownProperty::TransitionProperty,
    CssKnownProperty::TransitionDuration,
    CssKnownProperty::TransitionTimingFunction,
    CssKnownProperty::TransitionDelay,
];

fn body(property: CssKnownProperty, easing: &str) -> String {
    match property {
        CssKnownProperty::Transition => format!("opacity 1s {easing}"),
        CssKnownProperty::Animation => format!("fade 1s {easing}"),
        _ => easing.to_owned(),
    }
}

fn parsed(property: CssKnownProperty, source: &str) -> CssDeclaration {
    let css = format!("{}:{source}!important", property.canonical_name());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert_eq!(validate_style_attribute(&css), Ok(report.syntax().clone()));
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained authored declaration")
    };
    declaration.clone()
}

fn easing(declaration: &CssDeclaration) -> &CssEasing {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::TransitionTimingFunction(value) => {
            &value.timing_functions().values()[0]
        }
        CssKnownPropertyValueRef::AnimationTimingFunction(value) => {
            &value.timing_functions().values()[0]
        }
        CssKnownPropertyValueRef::Transition(value) => {
            value.transitions().values()[0].timing_function().unwrap()
        }
        CssKnownPropertyValueRef::Animation(value) => {
            value.animations().values()[0].timing_function().unwrap()
        }
        _ => panic!("shared easing consumer"),
    }
}

fn easing_list(declaration: &CssDeclaration) -> &CssEasingList {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::TransitionTimingFunction(value) => value.timing_functions(),
        CssKnownPropertyValueRef::AnimationTimingFunction(value) => value.timing_functions(),
        _ => panic!("timing longhand"),
    }
}

fn completed(declaration: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(declaration).unwrap()
    else {
        panic!("completed intrinsic contribution")
    };
    values
}

#[test]
fn unordered_stop_components_and_ordered_stop_lists_share_all_consumers() {
    for (source, canonical) in [
        ("linear(0, 1)", "linear(0, 1)"),
        ("LiNeAr(+000.0, 1e0)", "linear(0, 1)"),
        ("linear(-2 -10%, 3 120%)", "linear(-2 -10%, 3 120%)"),
        ("linear(0% 0, 100% 1)", "linear(0 0%, 1 100%)"),
        ("linear(20% 40% .5, 1)", "linear(0.5 20% 40%, 1)"),
        ("linear(0, .7, .3, 1)", "linear(0, 0.7, 0.3, 1)"),
    ] {
        for property in CONSUMERS {
            let value = body(property, source);
            let original = parsed(property, &value);
            assert_eq!(easing(&original).serialize_specified().unwrap(), canonical);
            let components = parse_component_values(&value).unwrap();
            for checked in [
                parse_property_value(
                    CssPropertyNameRef::Known(property),
                    components.clone(),
                    CssImportance::Important,
                ),
                parse_property_value_for_grammar(
                    property.grammar(),
                    components.clone(),
                    CssImportance::Important,
                ),
            ] {
                let checked = checked.expect("checked authored linear grammar");
                assert_eq!(easing(&checked), easing(&original));
                assert_eq!(checked.value_components(), &components);
                assert_eq!(checked.importance(), CssImportance::Important);
            }
            let roundtrip = parsed(property, &body(property, canonical));
            assert_eq!(easing(&roundtrip), easing(&original));
        }
    }
}

#[test]
fn authored_omissions_pairs_and_descending_inputs_wait_for_numeric_fixup() {
    // Easing 2 §2.1.2 creates endpoints, expands pairs, clamps descending
    // positions and spaces omission runs. Authored syntax retains the inputs
    // needed by that later operation; it does not pretend they were computed.
    for source in [
        "linear(0, 0.25, 1)",
        "linear(0 20%, 0.5 10%, 1)",
        "linear(0, 0.25 25% 75%, 1)",
        "linear(0 150%, 1)",
        "linear(0 80% 20%, 0.5, 1 50%)",
        "linear(0, 0.1, 0.9, 1)",
    ] {
        for property in LONGHANDS {
            let declaration = parsed(property, source);
            assert_eq!(easing(&declaration).serialize_specified().unwrap(), source);
            assert_eq!(
                declaration.value_components().serialize().unwrap().as_css(),
                source
            );
            assert_eq!(easing(&parsed(property, source)), easing(&declaration),);
        }
    }
}

#[test]
fn linear_keyword_and_equivalent_function_keep_separate_authored_identity() {
    for property in CONSUMERS {
        let keyword = parsed(property, &body(property, "linear"));
        let function = parsed(property, &body(property, "linear(0, 1)"));
        assert_eq!(
            easing(&keyword),
            &CssEasing::Keyword(CssEasingKeyword::Linear)
        );
        assert_ne!(easing(&keyword), easing(&function));
        assert_eq!(easing(&keyword).serialize_specified().unwrap(), "linear");
        assert_eq!(
            easing(&function).serialize_specified().unwrap(),
            "linear(0, 1)"
        );
    }
}

#[test]
fn symbolic_number_and_percentage_graphs_remain_authored_and_roundtrip() {
    for (source, canonical) in [
        (
            "linear(calc(1 + 1) calc(10% + 10%), sign(-2em) calc(80%))",
            "linear(calc(2) calc(20%), sign(-2em) calc(80%))",
        ),
        (
            "linear(calc(0 / 0) calc(-10%), calc(infinity) calc(120%))",
            "linear(calc(NaN) calc(-10%), calc(infinity) calc(120%))",
        ),
        (
            "linear(sign(-2em), sign(3em) calc(50%) calc(90%))",
            "linear(sign(-2em), sign(3em) calc(50%) calc(90%))",
        ),
    ] {
        for property in CONSUMERS {
            let value = parsed(property, &body(property, source));
            let before = easing(&value).clone();
            assert_eq!(easing(&value).serialize_specified().unwrap(), canonical);
            assert_eq!(easing(&value), &before);
            assert_eq!(
                easing(&parsed(property, &body(property, canonical)))
                    .serialize_specified()
                    .unwrap(),
                canonical
            );
            assert_eq!(
                value.value_components().serialize().unwrap().as_css(),
                body(property, source)
            );
        }
    }
}

#[test]
fn malformed_singleton_and_wrong_domain_stops_reject_atomically_and_recover() {
    for invalid in [
        "linear()",
        "linear(0)",
        "linear(0 0% 100%)",
        "linear(0,)",
        "linear(,1)",
        "linear(0,,1)",
        "linear(0 1)",
        "linear(0%, 1)",
        "linear(0 10px, 1)",
        "linear(1s, 1)",
        "linear(0 0% 20% 40%, 1)",
        "linear(0% 0 20%, 1)",
        "linear(calc(1px), 1)",
        "linear(0 calc(1), 1)",
        "linear(0, 1) junk",
    ] {
        for property in CONSUMERS {
            let value = body(property, invalid);
            let components = parse_component_values(&value).unwrap();
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
                assert!(result.is_err(), "{}:{value}", property.canonical_name());
            }
            let css = format!("{}:{value};transition-delay:2s", property.canonical_name());
            let report = parse_style_attribute(&css);
            assert!(!report.is_clean(), "{css}");
            let [survivor] = report.syntax().as_slice() else {
                panic!("invalid declaration drops as a unit: {css}")
            };
            assert_eq!(
                survivor.known().unwrap().property(),
                CssKnownProperty::TransitionDelay
            );
            assert!(validate_style_attribute(&css).is_err());
        }
    }
}

#[test]
fn implicit_linear_closure_is_retained_in_browser_parsing_but_checked_reentry_rejects_it() {
    for property in CONSUMERS {
        let value = body(property, "linear(0, 1");
        let css = format!("{}:{value}", property.canonical_name());
        let report = parse_style_attribute(&css);
        assert!(!report.is_clean());
        let [recovered] = report.syntax().as_slice() else {
            panic!("browser retains implicit function closure: {css}")
        };
        assert_eq!(
            easing(recovered).serialize_specified().unwrap(),
            "linear(0, 1)"
        );
        assert!(report.diagnostics().iter().any(|diagnostic| {
            diagnostic.error().code() == CssErrorCode::UnexpectedEnd
                && diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        }));
        let components = parse_component_values(&value).unwrap();
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
            let error = result.unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
            ));
            let CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { at, .. })) =
                error.origin()
            else {
                panic!("original implicit closure provenance: {error:?}")
            };
            assert_eq!(at.source().as_str(), value);
            assert_eq!(at.span().start().byte_offset().value(), value.len());
        }
    }
}

#[test]
fn transition_linear_only_expands_four_defaults_and_retains_source_importance() {
    let source = parsed(CssKnownProperty::Transition, "linear(0, 1)");
    let values = completed(&source);
    assert_eq!(
        values
            .items()
            .iter()
            .map(CssLonghandContribution::property)
            .collect::<Vec<_>>(),
        TRANSITION_MEMBERS
    );
    for (item, expected) in values
        .items()
        .iter()
        .zip(["all", "0s", "linear(0, 1)", "0s"])
    {
        let explicit = parsed(item.property(), expected);
        let explicit_values = completed(&explicit);
        assert_eq!(
            item.ordinary_value(),
            explicit_values.items()[0].ordinary_value()
        );
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn pending_linear_reentry_is_strict_repeatable_and_preserves_replacement_origins() {
    for property in CONSUMERS {
        let source = parsed(property, "var(--easing)");
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("whole-value substitution remains pending")
        };
        for invalid in ["linear(0)", "linear(0, 1", "linear(0, var(--again))"] {
            assert!(
                pending
                    .reenter(parse_component_values(&body(property, invalid)).unwrap())
                    .is_err()
            );
        }
        let replacement = parse_component_values(&body(property, "linear(0 20% 40%, 1)")).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("valid strict replacement")
            };
            let explicit = completed(&parsed(property, &body(property, "linear(0 20% 40%, 1)")));
            assert_eq!(values.items().len(), explicit.items().len());
            for (item, expected) in values.items().iter().zip(explicit.items()) {
                assert_eq!(item.ordinary_value(), expected.ordinary_value());
                assert_eq!(item.replacement_components(), Some(&replacement));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                for (actual, original) in item
                    .replacement_components()
                    .unwrap()
                    .items()
                    .iter()
                    .zip(replacement.items())
                {
                    assert_eq!(actual.origin(), original.origin());
                }
            }
        }
    }
}

#[test]
fn css_wide_transition_reentry_keeps_all_four_symbolic_contributions() {
    for keyword in ["initial", "inherit", "unset", "revert", "revert-layer"] {
        let explicit = parsed(CssKnownProperty::Transition, keyword);
        let expected = completed(&explicit);
        let source = parsed(CssKnownProperty::Transition, "var(--easing)");
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending shorthand")
        };
        let CssContributions::Longhands(values) = pending
            .reenter(parse_component_values(keyword).unwrap())
            .unwrap()
        else {
            panic!("CSS-wide replacement")
        };
        assert_eq!(
            values
                .items()
                .iter()
                .map(CssLonghandContribution::property)
                .collect::<Vec<_>>(),
            TRANSITION_MEMBERS
        );
        for (item, expected) in values.items().iter().zip(expected.items()) {
            assert_eq!(item.value(), expected.value());
            assert!(matches!(item.value(), CssContributionValueRef::Global(_)));
            assert_eq!(item.source().importance(), CssImportance::Important);
        }
    }
}

#[test]
fn normalization_keeps_linear_declaration_order_and_grouped_transition_members() {
    let report = parse_sheet(
        ".a { transition: linear(0, 1)!important; transition-timing-function: linear(0 80%, 1 20%); }",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 2);
    for (index, declaration) in declarations.iter().enumerate() {
        assert_eq!(declaration.order(), index);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declaration.expansion()
        else {
            panic!("ordinary grouped expansion")
        };
        assert_eq!(values.items().len(), if index == 0 { 4 } else { 1 });
        for item in values.items() {
            assert!(item.source().same_occurrence(declaration.source()));
            assert_eq!(
                item.source().importance(),
                if index == 0 {
                    CssImportance::Important
                } else {
                    CssImportance::Normal
                }
            );
        }
    }
    assert_eq!(
        easing(declarations[1].source())
            .serialize_specified()
            .unwrap(),
        "linear(0 80%, 1 20%)"
    );
}

#[test]
fn multiple_linear_functions_share_atomic_numeric_and_byte_budgets() {
    let expected = "linear(0, 1), linear(1 20% 40%, 0)";
    for property in LONGHANDS {
        let declaration = parsed(property, expected);
        let list = easing_list(&declaration);
        let original = list.clone();
        assert_eq!(list.values().len(), 2);
        assert_eq!(list.serialize_specified().unwrap(), expected);
        assert_eq!(
            list.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                usize::MAX,
                usize::MAX,
                expected.len()
            ))
            .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(1, usize::MAX, usize::MAX),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(usize::MAX, 1, usize::MAX),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(
                    usize::MAX,
                    usize::MAX,
                    expected.len() - 1,
                ),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
            // Each individual function fits, but the combined list does not.
            (
                CssSpecifiedValueSerializationLimits::new(
                    usize::MAX,
                    usize::MAX,
                    "linear(1 20% 40%, 0)".len(),
                ),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                list.serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(list, &original);
            assert_eq!(list.serialize_specified().unwrap(), expected);
        }
    }
}
