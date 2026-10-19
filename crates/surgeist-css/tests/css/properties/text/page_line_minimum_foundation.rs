#![forbid(unsafe_code)]

//! Authored line minima from Fragmentation 3 CR (2018-12-04) §3.3 and
//! lexical integers from Values 4 §5.2.

use surgeist_css::*;

const NAMES: [&str; 2] = ["orphans", "widows"];

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap()
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn checked(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"))
}

fn minimum(source: &CssDeclaration) -> &CssPageLineMinimum {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Orphans(value) => value.minimum(),
        CssKnownPropertyValueRef::Widows(value) => value.minimum(),
        _ => panic!("expected a line minimum"),
    }
}

fn one_value(source: &CssDeclaration) -> CssLonghandValue {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one terminal contribution")
    };
    let [item] = values.items() else {
        panic!("one terminal contribution")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    item.ordinary_value().unwrap().clone()
}

#[test]
fn positive_integer_literals_preserve_exact_authored_magnitude() {
    for name in NAMES {
        for value in [
            "1",
            "+0002",
            "2147483647",
            "2147483648",
            "2147483649",
            "9999999999999999999999",
        ] {
            for source in [declaration(name, value), checked(name, value)] {
                assert_eq!(source.importance(), CssImportance::Important);
                assert_eq!(
                    source.value_components().serialize().unwrap().as_css(),
                    value
                );
            }
        }
        assert_eq!(
            minimum(&declaration(name, "2147483647")).literal(),
            Some(i32::MAX)
        );
        for value in ["2147483648", "2147483649", "9999999999999999999999"] {
            assert_eq!(
                minimum(&declaration(name, value)).literal(),
                None,
                "{name}:{value} must not saturate to i32::MAX"
            );
        }
        assert_ne!(
            minimum(&declaration(name, "2147483648")),
            minimum(&declaration(name, "2147483649")),
            "adjacent large positive integers remain distinct"
        );
    }
}

#[test]
fn line_minima_identify_the_fragmentation_three_definitions() {
    for name in NAMES {
        let support = property_support_metadata(name).expect("known line-minimum property");
        assert_eq!(support.property(), grammar(name).target_property());
        assert_eq!(support.feature().source().id().as_str(), "S-BREAK3");
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2018/CR-css-break-3-20181204/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
    }
}

#[test]
fn invalid_integer_syntax_drops_only_its_declaration() {
    for name in NAMES {
        for value in ["0", "-0", "-1", "1.0", "1e0", "1px", "1%"] {
            let source = format!("color:red;{name}:{value};color:blue");
            let report = parse_style_attribute(&source);
            assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
            let [diagnostic] = report.diagnostics() else {
                panic!("one invalid-value diagnostic: {source}")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
            assert!(validate_style_attribute(&source).is_err());
            assert!(
                parse_property_value_for_grammar(
                    grammar(name),
                    parse_component_values(value).unwrap(),
                    CssImportance::Normal,
                )
                .is_err(),
                "checked construction accepted {name}:{value}"
            );
        }
    }
}

#[test]
fn calculated_integer_stays_symbolic_until_value_resolution() {
    for name in NAMES {
        for value in ["calc(2 + 1)", "calc(-1)"] {
            let source = declaration(name, value);
            assert!(minimum(&source).calculation().is_some());
            assert_eq!(minimum(&source).literal(), None);
            checked(name, value);
        }
    }
}

#[test]
fn line_minima_are_inherited_longhands_with_initial_two() {
    for name in NAMES {
        let property = grammar(name).target_property();
        let metadata = grammar(name).metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("{name} is a longhand")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(longhand.inherited_by_default(), "{name} inherits");
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), property);
        let CssInitialValueRef::Value(initial_value) = initial.view() else {
            panic!("{name} has initial two")
        };
        assert_eq!(initial_value, &one_value(&declaration(name, "2")));
        let source = declaration(name, "3");
        assert_eq!(one_value(&source).property().known_property(), property);
    }
}

#[test]
fn css_wide_values_and_all_reset_include_line_minima() {
    let all = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all).unwrap()
    else {
        panic!("symbolic all reset")
    };
    for name in NAMES {
        let property = grammar(name).target_property();
        assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, text);
            checked(name, text);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("one symbolic global contribution")
            };
            let [item] = values.items() else {
                panic!("one symbolic global contribution")
            };
            assert_eq!(item.property(), property);
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.source().same_occurrence(&source));
        }
    }
}

#[test]
fn pending_substitution_reenters_original_grammar_and_occurrence() {
    for name in NAMES {
        for pending in ["var(--minimum)", "env(--minimum)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending {name} substitution")
            };
            assert!(handle.source().same_occurrence(&source));
            assert!(matches!(
                handle
                    .reenter(parse_component_values("0").unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
            assert_eq!(
                handle
                    .reenter(parse_component_values("var(--again)").unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let replacement = parse_component_values("3").unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("one substituted longhand")
            };
            let [item] = values.items() else {
                panic!("one substituted longhand")
            };
            assert_eq!(item.property(), grammar(name).target_property());
            assert_eq!(
                item.ordinary_value(),
                Some(&one_value(&declaration(name, "3")))
            );
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn checked_construction_preserves_exact_tokens_and_origins() {
    for text in ["1", "+0002", "2147483648", "9999999999999999999999"] {
        let programmatic =
            CssPageLineMinimum::try_from_component(CssComponentValue::try_number(text).unwrap())
                .unwrap();
        assert_eq!(programmatic.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(
            programmatic
                .exact_literal()
                .unwrap()
                .integer()
                .numeric()
                .representation(),
            text
        );
        let parsed = declaration("orphans", text);
        assert!(matches!(
            minimum(&parsed).origin(),
            CssValueOrigin::Parsed(_)
        ));
        assert_eq!(
            minimum(&parsed)
                .exact_literal()
                .unwrap()
                .integer()
                .numeric()
                .representation(),
            text
        );
    }
    for text in ["0", "-1", "1.0", "1e0"] {
        let error =
            CssPageLineMinimum::try_from_component(CssComponentValue::try_number(text).unwrap())
                .unwrap_err();
        assert_eq!(error.origin(), Some(&CssValueOrigin::Programmatic));
        let expected = if text == "0" || text == "-1" {
            CssNumericConstructionErrorKind::OutOfRange
        } else {
            CssNumericConstructionErrorKind::RootDomainMismatch
        };
        assert_eq!(error.kind(), &expected);
    }
}

#[test]
fn calculation_constructor_checks_bare_roots_and_retains_math() {
    for text in ["0", "-1"] {
        let calculation =
            CssIntegerCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap();
        let error = CssPageLineMinimum::try_from_calculation(calculation).unwrap_err();
        assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
        assert!(matches!(error.origin(), Some(CssValueOrigin::Parsed(_))));
    }
    let error =
        CssPageLineMinimum::try_from_calculation(CssIntegerCalculation::literal(-1)).unwrap_err();
    assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
    assert_eq!(error.origin(), Some(&CssValueOrigin::Programmatic));
    let bare =
        CssIntegerCalculation::try_from_components(parse_component_values("2147483648").unwrap())
            .unwrap();
    let admitted = CssPageLineMinimum::try_from_calculation(bare).unwrap();
    assert!(admitted.exact_literal().is_some());
    assert_eq!(admitted.literal(), None);
    for (text, expected) in [("calc(-1)", "calc(-1)"), ("calc(2 + 1)", "calc(3)")] {
        let calculation =
            CssIntegerCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap();
        let value = CssPageLineMinimum::try_from_calculation(calculation).unwrap();
        assert!(value.calculation().is_some());
        assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn specified_serialization_is_exact_and_resource_bounded() {
    for (authored, expected) in [
        ("+0002", "2"),
        ("2147483648", "2147483648"),
        ("9999999999999999999999", "9999999999999999999999"),
    ] {
        let value = minimum(&declaration("widows", authored)).clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
    let value = minimum(&declaration("orphans", "2147483648")).clone();
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 8, 20),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(8, 0, 20),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(8, 8, 3),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let error = value.serialize_specified_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(value.serialize_specified().unwrap(), "2147483648");
    }
}

#[test]
fn normalization_keeps_valid_line_minima_in_source_order_after_recovery() {
    let report =
        parse_sheet(".a{orphans:2!important;widows:3;orphans:0;widows:2147483648!important}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid line minimum: {:?}", report.diagnostics())
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 3);
    for (index, (name, importance)) in [
        ("orphans", CssImportance::Important),
        ("widows", CssImportance::Normal),
        ("widows", CssImportance::Important),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations[index].order(), index);
        assert_eq!(declarations[index].source().importance(), importance);
        assert_eq!(
            declarations[index].source().known().unwrap().property(),
            grammar(name).target_property()
        );
        assert!(matches!(
            declarations[index].expansion(),
            CssExpansion::Contributions(CssContributions::Longhands(_))
        ));
    }
}

#[test]
fn line_minimum_normalization_reports_cumulative_contribution_limit() {
    let report = parse_sheet(".a{orphans:2;widows:3}");
    assert!(report.is_clean());
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 2, 1).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 1,
        }
    );
    let normalized = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 2, 2).unwrap(),
    )
    .unwrap();
    assert_eq!(
        normalized
            .items()
            .iter()
            .filter(|item| matches!(item, CssNormalizedItem::Declaration(_)))
            .count(),
        2
    );
}
