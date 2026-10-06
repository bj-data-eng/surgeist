#![forbid(unsafe_code)]
//! Grid mathematical roots and slot restrictions across authored lifecycle boundaries.
//! Independent authorities: selected Values4 §10.3/10.5/10.6/10.9/10.12,
//! Grid2 track-size/inflexible/fit-content grammar, accepted browser tiebreaker.
//! Known Number and absolute-unit goldens use independent arithmetic results.
//! Contextual Flex goldens preserve the adopted specified-phase basis policy.
//! Representative cases establish the actual Grid boundary, not arbitrary
//! crossproducts of all 21 recognized math functions.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;
const TRACKS: [P; 4] = [
    P::GridTemplateRows,
    P::GridTemplateColumns,
    P::GridAutoRows,
    P::GridAutoColumns,
];
const EXPLICIT: [P; 3] = [
    P::GridTemplateRows,
    P::GridTemplateColumns,
    P::GridTemplateAreas,
];
const GRID: [P; 6] = [
    P::GridTemplateRows,
    P::GridTemplateColumns,
    P::GridTemplateAreas,
    P::GridAutoRows,
    P::GridAutoColumns,
    P::GridAutoFlow,
];

fn parsed(property: P, text: &str) -> CssDeclaration {
    let css = format!("/*😀*/{}:{text}!important", property.canonical_name());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&css).is_ok());
    let [source] = report.syntax().as_slice() else {
        panic!("one occurrence")
    };
    assert_eq!(source.known().unwrap().property(), property);
    assert_eq!(source.importance(), CssImportance::Important);
    source.clone()
}
fn checked(property: P, text: &str, grammar: bool) -> CssDeclaration {
    let values = parse_component_values(text).unwrap();
    let before = values.clone();
    let result = if grammar {
        parse_property_value_for_grammar(
            property.grammar(),
            values.clone(),
            CssImportance::Important,
        )
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            values.clone(),
            CssImportance::Important,
        )
    };
    let source =
        result.unwrap_or_else(|error| panic!("{}:{text}: {error:?}", property.canonical_name()));
    assert_eq!(values, before);
    assert_eq!(source.value_components(), &values);
    assert!(source.position().is_none());
    source
}
fn fronts(property: P, text: &str) -> [CssDeclaration; 3] {
    [
        parsed(property, text),
        checked(property, text, false),
        checked(property, text, true),
    ]
}
fn first_axis_size(list: &CssGridTrackList) -> &CssGridTrackSize {
    let [CssGridGeneralTrackComponent::TrackSize(size)] = list.general_list().unwrap().components()
    else {
        panic!("one general track size")
    };
    size
}
fn first_size(source: &CssDeclaration) -> &CssGridTrackSize {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::GridTemplateRows(v) => first_axis_size(v.value()),
        CssKnownPropertyValueRef::GridTemplateColumns(v) => first_axis_size(v.value()),
        CssKnownPropertyValueRef::GridAutoRows(v) => &v.value().sizes()[0],
        CssKnownPropertyValueRef::GridAutoColumns(v) => &v.value().sizes()[0],
        _ => panic!("track property"),
    }
}
fn typed_output(
    source: &CssDeclaration,
    limits: Limits,
) -> Result<String, CssSpecifiedValueSerializationError> {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::GridTemplateRows(v) => {
            v.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::GridTemplateColumns(v) => {
            v.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::GridAutoRows(v) => {
            v.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::GridAutoColumns(v) => {
            v.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::GridTemplate(v) => {
            v.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::Grid(v) => v.value().serialize_specified_with_limits(limits),
        _ => panic!("Grid typed writer"),
    }
}
fn output(value: CssLonghandValueRef<'_>) -> String {
    match value {
        CssLonghandValueRef::GridTemplateRows(v) | CssLonghandValueRef::GridTemplateColumns(v) => {
            v.serialize_specified().unwrap()
        }
        CssLonghandValueRef::GridAutoRows(v) | CssLonghandValueRef::GridAutoColumns(v) => {
            v.serialize_specified().unwrap()
        }
        CssLonghandValueRef::GridTemplateAreas(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::GridAutoFlow(v) => v.serialize_specified().unwrap(),
        _ => panic!("Grid terminal"),
    }
}
fn complete(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("completed Grid contributions")
    };
    values
}
fn context(
    item: &CssLonghandContribution,
    source: &CssDeclaration,
    replacement: Option<&CssComponentValues>,
) {
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.source().position(), source.position());
    assert_eq!(item.source().value_components(), source.value_components());
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
    assert_eq!(item.replacement_components(), replacement);
}
fn invalid(property: P, text: &str) {
    let css = format!("color:red;{}:{text};color:blue", property.canonical_name());
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "{css}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one atomic failure: {css}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("typed Grid property error")
    };
    assert_eq!(detail.property(), property);
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let values = parse_component_values(text).unwrap();
    let before = values.clone();
    for result in [
        parse_property_value(
            CssPropertyNameRef::Known(property),
            values.clone(),
            CssImportance::Important,
        ),
        parse_property_value_for_grammar(
            property.grammar(),
            values.clone(),
            CssImportance::Important,
        ),
    ] {
        let error = result.unwrap_err();
        assert!(matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
    }
    assert_eq!(values, before);
}
fn flex_math<'a>(source: &'a CssDeclaration, text: &str) -> &'a CssFlexCalculation {
    let scalar = first_size(source).breadth().unwrap().flex().unwrap();
    assert!(scalar.literal_component().is_none());
    let calc = scalar.calculation().unwrap();
    assert_eq!(calc.result_type(), CssCalculationType::Flex);
    assert_eq!(calc.numeric_type().exponent(CssNumericDimension::Flex), 1);
    assert_eq!(calc.numeric_type().exponent(CssNumericDimension::Length), 0);
    assert_eq!(
        calc.numeric_type()
            .exponent(CssNumericDimension::Percentage),
        0
    );
    assert_eq!(calc.numeric_type().percent_hint(), None);
    // Retained component equality preserves original spelling, children and origins.
    assert_eq!(calc.components().items(), source.value_components().items());
    let CssValueOrigin::Parsed(origin) = calc.origin() else {
        panic!("parsed original root")
    };
    let start = origin.span().start().byte_offset().value();
    assert!(origin.source().as_str()[start..].starts_with(text.split('(').next().unwrap()));
    calc
}

#[test]
fn representative_type_preserving_functions_reach_real_explicit_and_implicit_flex_tracks() {
    for (text, expected, function) in [
        (
            "round(2.5fr, 1fr)",
            "round(2.5fr, 1fr)",
            CssMathFunction::Round,
        ),
        (
            "round(down, 2.5fr, 1fr)",
            "round(down, 2.5fr, 1fr)",
            CssMathFunction::Round,
        ),
        ("mod(-5fr, 3fr)", "mod(-5fr, 3fr)", CssMathFunction::Mod),
        ("rem(-5fr, 3fr)", "rem(-5fr, 3fr)", CssMathFunction::Rem),
        ("hypot(3fr, 4fr)", "hypot(3fr, 4fr)", CssMathFunction::Hypot),
        ("abs(-2fr)", "abs(-2fr)", CssMathFunction::Abs),
        ("clamp(1fr, 2fr, 3fr)", "calc(2fr)", CssMathFunction::Clamp),
    ] {
        for property in TRACKS {
            for source in fronts(property, text) {
                let before = source.clone();
                let calc = flex_math(&source, text);
                let CssCalculationExpressionRef::Function(root) = calc.expression() else {
                    panic!("retained function")
                };
                assert_eq!(root.function(), function);
                assert_eq!(root.origin(), calc.origin());
                assert_eq!(typed_output(&source, Limits::default()).unwrap(), expected);
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{}: {expected} !important;", property.canonical_name())
                );
                let values = complete(&source);
                let [item] = values.items() else {
                    panic!("one longhand")
                };
                context(item, &source, None);
                assert_eq!(item.property(), property);
                assert_eq!(output(item.ordinary_value().unwrap().view()), expected);
                assert_eq!(source, before);
                checked(property, expected, false);
                checked(property, expected, true);
            }
        }
    }
}

#[test]
fn number_result_subexpressions_become_flex_only_after_dimensionally_valid_products() {
    for (text, expected) in [
        ("calc(pow(2, 3) * 1fr)", "calc(8fr)"),
        ("calc(sin(90deg) * 1fr)", "calc(1fr)"),
        ("calc(sign(-2fr) * 1fr)", "calc(1fr * sign(-2fr))"),
        ("calc(round(2.5) * 1fr)", "calc(3fr)"),
    ] {
        for property in TRACKS {
            for source in fronts(property, text) {
                flex_math(&source, text);
                assert_eq!(typed_output(&source, Limits::default()).unwrap(), expected);
            }
        }
    }
    for property in TRACKS {
        for text in ["sign(2fr)", "sin(90deg)", "pow(2, 3)", "atan(1)"] {
            invalid(property, text);
        }
    }
    // Frozen WPT witnesses this contextual sign-to-number-to-flex composition.
    // Authored CSS must retain it without guessing font metrics.
    let text = "calc(3fr + 1fr * sign(42px - 2em))";
    for property in TRACKS {
        for source in fronts(property, text) {
            let calc = flex_math(&source, text);
            let CssCalculationExpressionRef::NestedCalc(root) = calc.expression() else {
                panic!("outer calc")
            };
            let CssCalculationExpressionRef::Sum(sum) = root.operand() else {
                panic!("retained contextual sum")
            };
            let CssCalculationExpressionRef::Product(product) = sum.term(1).unwrap().expression()
            else {
                panic!("retained product")
            };
            let CssCalculationExpressionRef::Function(sign) =
                product.factor(1).unwrap().expression()
            else {
                panic!("retained sign")
            };
            assert_eq!(sign.function(), CssMathFunction::Sign);
            let sign_type = sign.argument(0).unwrap().unwrap().numeric_type();
            assert_eq!(sign_type.exponent(CssNumericDimension::Length), 1);
            assert_eq!(sign_type.percent_hint(), None);
        }
    }
}

#[test]
fn length_percentage_fallback_preserves_hints_and_real_inflexible_contexts() {
    for property in TRACKS {
        for source in fronts(property, "hypot(3px, 4%)") {
            let scalar = first_size(&source)
                .breadth()
                .unwrap()
                .length_percentage()
                .unwrap();
            let calc = scalar.calculation().unwrap();
            assert_eq!(calc.result_type(), CssCalculationType::LengthPercentage);
            assert_eq!(
                calc.numeric_type().percent_hint(),
                Some(CssNumericDimension::Length)
            );
            assert_eq!(calc.numeric_type().exponent(CssNumericDimension::Flex), 0);
            assert_eq!(calc.components().items(), source.value_components().items());
            let CssCalculationExpressionRef::Function(root) = calc.expression() else {
                panic!("hypot graph")
            };
            assert_eq!(root.function(), CssMathFunction::Hypot);
            let Some(Some(CssCalculationExpressionRef::Value(CssCalculationValueRef::Percentage(
                percent,
            )))) = root.argument(1)
            else {
                panic!("original percentage child")
            };
            assert_eq!(
                percent.numeric_type().percent_hint(),
                Some(CssNumericDimension::Length)
            );
        }
        for (text, expected) in [
            ("fit-content(abs(-2px))", "fit-content(calc(2px))"),
            (
                "minmax(round(2.5px, 1px), hypot(3fr, 4fr))",
                "minmax(calc(3px), hypot(3fr, 4fr))",
            ),
        ] {
            for source in fronts(property, text) {
                assert_eq!(typed_output(&source, Limits::default()).unwrap(), expected);
                if let Some((min, max)) = first_size(&source).minmax() {
                    assert!(min.length_percentage().is_some());
                    assert!(min.flex().is_none());
                    assert!(max.flex().is_some());
                } else {
                    assert!(first_size(&source).fit_content().is_some());
                }
            }
        }
        for text in [
            "abs(-2fr)",
            "round(2.5fr, 1fr)",
            "mod(5fr, 3fr)",
            "rem(5fr, 3fr)",
            "hypot(3fr, 4fr)",
        ] {
            invalid(property, &format!("fit-content({text})"));
            invalid(property, &format!("minmax({text}, 10px)"));
        }
    }
}

#[test]
fn mathematical_negative_flex_stays_authored_while_bare_negative_literals_fail() {
    let negative =
        CssFlexCalculation::try_from_components(parse_component_values("rem(-5fr, 3fr)").unwrap())
            .unwrap();
    let before = negative.clone();
    let scalar = CssSpecifiedNonNegativeFlex::try_from_calculation(negative.clone()).unwrap();
    assert_eq!(scalar.serialize_specified().unwrap(), "rem(-5fr, 3fr)");
    assert_eq!(negative, before);
    let breadth = CssGridTrackBreadth::from_flex(scalar);
    assert!(CssGridTrackSize::try_minmax(breadth.clone(), CssGridTrackBreadth::auto()).is_none());
    assert!(CssGridTrackSize::try_minmax(CssGridTrackBreadth::auto(), breadth).is_some());
    let bare = CssComponentValue::try_dimension("-2", "fr").unwrap();
    assert_eq!(
        CssSpecifiedNonNegativeFlex::try_from_component(bare)
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::OutOfRange
    );
    // This negative scalar is independently known without resolving a Flex basis.
    // The remainder graph above retains its contextual operands and step.
    let direct_negative =
        CssFlexCalculation::try_from_components(parse_component_values("calc(-2fr)").unwrap())
            .unwrap();
    let direct_before = direct_negative.clone();
    let direct_scalar =
        CssSpecifiedNonNegativeFlex::try_from_calculation(direct_negative.clone()).unwrap();
    assert_eq!(direct_scalar.serialize_specified().unwrap(), "calc(-2fr)");
    assert_eq!(direct_scalar.calculation(), Some(&direct_negative));
    assert_eq!(direct_negative, direct_before);
    for property in TRACKS {
        invalid(property, "-2fr");
        for (text, expected) in [
            ("rem(-5fr, 3fr)", "rem(-5fr, 3fr)"),
            ("calc(-2fr)", "calc(-2fr)"),
        ] {
            for source in fronts(property, text) {
                flex_math(&source, text);
                assert_eq!(typed_output(&source, Limits::default()).unwrap(), expected);
            }
        }
    }
}

#[test]
fn invalid_math_has_typed_numeric_errors_original_origins_and_atomic_property_recovery() {
    for (text, kind, responsible) in [
        (
            "round(1fr)",
            CssNumericConstructionErrorKind::InvalidArgumentType,
            "round",
        ),
        ("mod(1fr)", CssNumericConstructionErrorKind::Arity, "mod"),
        ("hypot()", CssNumericConstructionErrorKind::Arity, "hypot"),
        (
            "abs(1fr, 2fr)",
            CssNumericConstructionErrorKind::Arity,
            "abs",
        ),
        (
            "calc(1fr + 10px)",
            CssNumericConstructionErrorKind::IncompatibleTypes,
            "10px",
        ),
        (
            "calc(1fr + 1%)",
            CssNumericConstructionErrorKind::RootDomainMismatch,
            "calc",
        ),
        (
            "sin(1fr)",
            CssNumericConstructionErrorKind::InvalidArgumentType,
            "sin",
        ),
    ] {
        let values = parse_component_values(text).unwrap();
        let before = values.clone();
        for _ in 0..2 {
            let error = CssFlexCalculation::try_from_components(values.clone()).unwrap_err();
            assert_eq!(error.kind(), &kind, "{text}");
            let Some(CssValueOrigin::Parsed(origin)) = error.origin() else {
                panic!("original failure origin")
            };
            assert_eq!(
                origin.span().start().byte_offset().value(),
                text.find(responsible).unwrap()
            );
        }
        assert_eq!(values, before);
        for property in TRACKS {
            invalid(property, text);
        }
    }
    // Checked property mapping keeps the responsible original token, including
    // a UTF-8 comment prefix; this is not a recreated canonical-input origin.
    let text = "/*😀*/calc(1fr + 10px)";
    let values = parse_component_values(text).unwrap();
    let serialized = values.serialize().unwrap();
    let expected = serialized
        .origin_at(serialized.as_css().find("10px").unwrap())
        .unwrap()
        .clone();
    for property in TRACKS {
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(property),
                values.clone(),
                CssImportance::Important,
            ),
            parse_property_value_for_grammar(
                property.grammar(),
                values.clone(),
                CssImportance::Important,
            ),
        ] {
            assert_eq!(result.unwrap_err().origin(), &expected);
        }
    }
}

#[test]
fn math_shorthands_project_stable_members_defaults_and_original_child_origins() {
    let text = "round(2.5fr, 1fr) / hypot(3fr, 4fr)";
    for property in [P::GridTemplate, P::Grid] {
        for source in fronts(property, text) {
            assert_eq!(
                typed_output(&source, Limits::default()).unwrap(),
                "round(2.5fr, 1fr) / hypot(3fr, 4fr)"
            );
            let values = complete(&source);
            let names = if property == P::Grid {
                &GRID[..]
            } else {
                &EXPLICIT[..]
            };
            let expected = [
                "round(2.5fr, 1fr)",
                "hypot(3fr, 4fr)",
                "none",
                "auto",
                "auto",
                "normal",
            ];
            assert_eq!(values.items().len(), names.len());
            for (index, item) in values.items().iter().enumerate() {
                context(item, &source, None);
                assert_eq!(item.property(), names[index]);
                assert_eq!(
                    output(item.ordinary_value().unwrap().view()),
                    expected[index]
                );
                if index < 2 {
                    let axis = match item.ordinary_value().unwrap().view() {
                        CssLonghandValueRef::GridTemplateRows(v)
                        | CssLonghandValueRef::GridTemplateColumns(v) => v,
                        _ => panic!("projected axis"),
                    };
                    let calc = first_axis_size(axis)
                        .breadth()
                        .unwrap()
                        .flex()
                        .unwrap()
                        .calculation()
                        .unwrap();
                    let original = source.value_components().items().iter().find(|value| {
                        matches!(value.view(), CssComponentValueRef::Function(f) if f.name() == if index == 0 { "round" } else { "hypot" })
                    }).unwrap();
                    assert_eq!(calc.components().items(), std::slice::from_ref(original));
                    assert_eq!(calc.origin(), original.origin());
                }
            }
        }
    }
    for (text, implicit_property, expected) in [
        (
            "auto-flow abs(-2fr) / 10px",
            P::GridAutoRows,
            ["none", "10px", "none", "abs(-2fr)", "auto", "row"],
        ),
        (
            "10px / auto-flow abs(-2fr)",
            P::GridAutoColumns,
            ["10px", "none", "none", "auto", "abs(-2fr)", "column"],
        ),
    ] {
        for source in fronts(P::Grid, text) {
            let values = complete(&source);
            for (index, item) in values.items().iter().enumerate() {
                context(item, &source, None);
                assert_eq!(item.property(), GRID[index]);
                assert_eq!(
                    output(item.ordinary_value().unwrap().view()),
                    expected[index]
                );
            }
            assert!(
                values
                    .items()
                    .iter()
                    .any(|item| item.property() == implicit_property)
            );
        }
    }
}

#[test]
fn pending_math_reentry_preserves_occurrence_replacement_origins_and_retry_after_failure() {
    for property in TRACKS.into_iter().chain([P::GridTemplate, P::Grid]) {
        let replacement_text = if TRACKS.contains(&property) {
            "hypot(3fr, 4fr)"
        } else {
            "hypot(3fr, 4fr) / abs(-2fr)"
        };
        for pending_text in ["var(--grid)", "env(grid)", "attr(data-grid *)"] {
            for source in fronts(property, pending_text) {
                let before = source.clone();
                let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                    panic!("whole pending handle")
                };
                for invalid_text in ["round(1fr)", "var(--still-pending)"] {
                    let invalid_values = parse_component_values(invalid_text).unwrap();
                    let invalid_before = invalid_values.clone();
                    for _ in 0..2 {
                        let error = handle.reenter(invalid_values.clone()).unwrap_err();
                        if invalid_text.starts_with("var") {
                            assert!(matches!(
                                error.kind(),
                                CssExpansionErrorKind::ResidualSubstitution
                            ));
                        } else {
                            assert!(matches!(
                                error.kind(),
                                CssExpansionErrorKind::InvalidReplacement(_)
                            ));
                        }
                        assert!(handle.source().same_occurrence(&source));
                    }
                    assert_eq!(invalid_values, invalid_before);
                }
                let replacement = parse_component_values(replacement_text).unwrap();
                let replacement_before = replacement.clone();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("completed retry")
                };
                let expected_count = if property == P::Grid {
                    6
                } else if property == P::GridTemplate {
                    3
                } else {
                    1
                };
                assert_eq!(values.items().len(), expected_count);
                for item in values.items() {
                    context(item, &source, Some(&replacement));
                }
                assert_eq!(
                    output(values.items()[0].ordinary_value().unwrap().view()),
                    "hypot(3fr, 4fr)"
                );
                assert_eq!(replacement, replacement_before);
                assert_eq!(source, before);
                for (text, keyword) in [
                    ("initial", CssGlobalKeyword::Initial),
                    ("inherit", CssGlobalKeyword::Inherit),
                    ("unset", CssGlobalKeyword::Unset),
                    ("revert", CssGlobalKeyword::Revert),
                    ("revert-layer", CssGlobalKeyword::RevertLayer),
                ] {
                    let replacement = parse_component_values(text).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("global retry")
                    };
                    assert_eq!(values.items().len(), expected_count);
                    for item in values.items() {
                        context(item, &source, Some(&replacement));
                        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                    }
                }
            }
        }
    }
}

#[test]
fn math_graph_and_sibling_serialization_limits_remain_cumulative_atomic_and_reusable() {
    let values = parse_component_values("hypot(3fr, 4fr)").unwrap();
    let before = values.clone();
    let short = CssComponentValueLimits::try_new(256, 1, 100).unwrap();
    for _ in 0..2 {
        assert_eq!(
            CssFlexCalculation::try_from_components_with_limits(values.clone(), short)
                .unwrap_err()
                .kind(),
            &CssNumericConstructionErrorKind::ResourceLimit
        );
        assert_eq!(values, before);
    }
    assert!(CssFlexCalculation::try_from_components(values).is_ok());
    let source = checked(P::GridAutoRows, "hypot(3fr, 4fr) abs(-2fr)", true);
    let before = source.clone();
    let expected = "hypot(3fr, 4fr) abs(-2fr)";
    assert_eq!(expected.len(), 25);
    let limits = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    // Each independently serialized child fits; the enclosing list cannot.
    let CssKnownPropertyValueRef::GridAutoRows(list) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("implicit list")
    };
    assert!(
        list.value()
            .sizes()
            .iter()
            .all(|size| size.serialize_specified_with_limits(limits).is_ok())
    );
    for _ in 0..2 {
        assert_eq!(
            typed_output(&source, limits).unwrap_err().kind(),
            Kind::ByteLimit
        );
        assert_eq!(source, before);
    }
    assert_eq!(
        typed_output(&source, Limits::new(usize::MAX, usize::MAX, expected.len())).unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(0, usize::MAX, usize::MAX), Kind::InputNodeLimit),
        (
            Limits::new(usize::MAX, 0, usize::MAX),
            Kind::ProjectionNodeLimit,
        ),
    ] {
        assert_eq!(typed_output(&source, limits).unwrap_err().kind(), kind);
        assert_eq!(source, before);
    }
    assert_eq!(typed_output(&source, Limits::default()).unwrap(), expected);
}

#[test]
fn math_normalization_counts_three_and_six_terminals_without_changing_order_or_importance() {
    let css = "@media screen{.a{grid:round(2.5fr, 1fr) / hypot(3fr, 4fr)!important;grid-template:abs(-2fr) / mod(5fr, 3fr)}}";
    let report = parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 2, 2, 9).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let items: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 2);
    for (index, item) in items.iter().enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().importance(),
            if index == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        assert!(item.rule_context().same_context(items[0].rule_context()));
        assert!(
            item.selector_context()
                .same_context(items[0].selector_context())
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("ordinary Grid contributions")
        };
        assert_eq!(values.items().len(), if index == 0 { 6 } else { 3 });
        for terminal in values.items() {
            context(terminal, item.source(), None);
        }
    }
    let short = CssNormalizationLimits::try_new(1, 2, 2, 8).unwrap();
    for _ in 0..2 {
        let error = normalize_sheet_with_limits(report.syntax(), short).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded {
                resource: CssNormalizationResource::Contributions,
                limit: 8
            }
        );
        assert_eq!(error.declaration_order(), Some(1));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(items[1].source())
        );
        assert_eq!(report, before);
    }
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}

#[test]
fn mixed_parsed_and_programmatic_math_children_keep_distinct_origins_through_grid_construction() {
    let parsed_child = parse_component_values("3fr").unwrap().items()[0].clone();
    let programmatic_child = CssComponentValue::try_dimension("4", "fr").unwrap();
    let values = CssComponentValues::try_new(vec![
        CssComponentValue::try_function(
            "hypot",
            CssComponentValues::try_new(vec![
                parsed_child.clone(),
                CssComponentValue::try_token(",").unwrap(),
                programmatic_child.clone(),
            ])
            .unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();
    let before = values.clone();
    let calc = CssFlexCalculation::try_from_components(values.clone()).unwrap();
    assert_eq!(calc.origin(), &CssValueOrigin::Programmatic);
    let CssCalculationExpressionRef::Function(root) = calc.expression() else {
        panic!("hypot graph")
    };
    for (index, expected) in [parsed_child.origin(), programmatic_child.origin()]
        .into_iter()
        .enumerate()
    {
        let Some(Some(CssCalculationExpressionRef::Value(CssCalculationValueRef::Flex(child)))) =
            root.argument(index)
        else {
            panic!("flex child")
        };
        assert_eq!(child.origin(), expected);
    }
    for property in TRACKS {
        for source in [
            parse_property_value(
                CssPropertyNameRef::Known(property),
                values.clone(),
                CssImportance::Important,
            )
            .unwrap(),
            parse_property_value_for_grammar(
                property.grammar(),
                values.clone(),
                CssImportance::Important,
            )
            .unwrap(),
        ] {
            let retained = first_size(&source)
                .breadth()
                .unwrap()
                .flex()
                .unwrap()
                .calculation()
                .unwrap();
            assert_eq!(retained, &calc);
            assert_eq!(
                typed_output(&source, Limits::default()).unwrap(),
                "hypot(3fr, 4fr)"
            );
            assert_eq!(source.value_components(), &before);
        }
    }
    assert_eq!(values, before);
}

#[test]
fn checked_math_closure_and_browser_recovery_remain_distinct_existing_controls() {
    for property in TRACKS.into_iter().chain([P::GridTemplate, P::Grid]) {
        let recovered = if TRACKS.contains(&property) {
            "hypot(3fr, 4fr"
        } else {
            "10px / hypot(3fr, 4fr"
        };
        let values = parse_component_values(recovered).unwrap();
        let before = values.clone();
        let origin = values
            .items()
            .iter()
            .find_map(|value| match value.view() {
                CssComponentValueRef::Function(function) => Some(function.closing_origin().clone()),
                _ => None,
            })
            .unwrap();
        assert!(matches!(origin, CssValueOrigin::ImplicitClosure { .. }));
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(property),
                values.clone(),
                CssImportance::Important,
            ),
            parse_property_value_for_grammar(
                property.grammar(),
                values.clone(),
                CssImportance::Important,
            ),
        ] {
            let error = result.unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
            ));
            assert_eq!(
                error.origin(),
                &CssSerializedOrigin::End(Some(origin.clone()))
            );
        }
        let source = checked(property, "var(--grid)", true);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for _ in 0..2 {
            let error = handle.reenter(values.clone()).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(inner) = error.kind() else {
                panic!("strict replacement")
            };
            assert!(matches!(
                inner.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
            ));
            assert_eq!(
                inner.origin(),
                &CssSerializedOrigin::End(Some(origin.clone()))
            );
        }
        let closed = format!("{recovered})");
        assert!(
            handle
                .reenter(parse_component_values(&closed).unwrap())
                .is_ok()
        );
        assert_eq!(values, before);
        assert!(handle.source().same_occurrence(&source));
        let css = format!("{}:{recovered}", property.canonical_name());
        let report = parse_style_attribute(&css);
        assert!(!report.is_clean());
        assert_eq!(report.syntax().len(), 1);
        assert!(validate_style_attribute(&css).is_err());
        assert_eq!(
            typed_output(&report.syntax()[0], Limits::default()).unwrap(),
            if TRACKS.contains(&property) {
                "hypot(3fr, 4fr)"
            } else {
                "10px / hypot(3fr, 4fr)"
            }
        );
    }
}
