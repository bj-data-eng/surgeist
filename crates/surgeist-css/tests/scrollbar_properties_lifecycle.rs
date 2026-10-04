#![forbid(unsafe_code)]

//! Existing-API RED expectations from CSS Scrollbars 1 CR (2021-12-09), §§2–3: https://www.w3.org/TR/2021/CR-css-scrollbars-1-20211209/.
//! Canonical writers and new typed payload accessors are tested with their implementation;
//! this candidate exercises admission and the already-existing authored lifecycle APIs.

use surgeist_css::*;

const CASES: &[(&str, &str, bool, &[&str], &[&str])] = &[
    (
        "scrollbar-width",
        "auto",
        false,
        &["auto", "thin", "NONE", r"th\69 n"],
        &["", "8", "10%", "auto thin", "thin,none", "wide"],
    ),
    (
        "scrollbar-color",
        "auto",
        true,
        &[
            "auto",
            "red blue",
            "currentcolor Canvas",
            "rgb(10 20 30) color(srgb .2 .3 .4)",
        ],
        &["", "red", "red blue green", "auto red", "red,blue", "thin"],
    ),
];

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let text = format!("{name}:{value}!important");
    let report = parse_style_attribute(&text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&text).is_ok(), "{text}");
    let [source] = report.syntax().as_slice() else {
        panic!("one admitted declaration: {text}")
    };
    assert_eq!(source.known().unwrap().property().canonical_name(), name);
    assert_eq!(source.importance(), CssImportance::Important);
    assert_eq!(
        source.value_components().serialize().unwrap().as_css(),
        value
    );
    source.clone()
}

fn contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("selected authored longhand expands")
    else {
        panic!("one completed longhand")
    };
    let [item] = values.items() else {
        panic!("exactly one terminal contribution")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    item.clone()
}

#[test]
fn source_derived_valid_values_are_admitted_with_original_components() {
    for &(name, _, _, valid, _) in CASES {
        for &text in valid {
            let source = declaration(name, text);
            assert!(source.known().unwrap().property_value().is_some());
            assert!(source.parsed_value().is_some());
            assert!(
                source
                    .value_components()
                    .items()
                    .iter()
                    .all(|item| { !matches!(item.origin(), CssValueOrigin::Programmatic) })
            );
        }
    }
}

#[test]
fn checked_construction_admits_the_same_grammar_and_preserves_input_origins() {
    for &(name, _, _, valid, _) in CASES {
        // Obtain identity only after real successful admission; no new symbols are needed.
        let admitted = declaration(name, valid[0]);
        let property = admitted.known().unwrap().property();
        let grammar = CssPropertyGrammar::from_name(name).unwrap();
        assert_eq!(grammar.target_property(), property);
        for &text in valid {
            let components = parse_component_values(text).unwrap();
            for source in [
                parse_property_value(
                    CssPropertyNameRef::Known(property),
                    components.clone(),
                    CssImportance::Important,
                )
                .unwrap(),
                parse_property_value_for_grammar(
                    grammar,
                    components.clone(),
                    CssImportance::Important,
                )
                .unwrap(),
            ] {
                assert!(source.parsed_value().is_none());
                assert_eq!(source.value_components(), &components);
                assert_eq!(source.importance(), CssImportance::Important);
                assert_eq!(source.known().unwrap().property(), property);
            }
        }
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_ident(valid[0]).unwrap()])
                .unwrap();
        let source = parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(source.value_components(), &components);
        assert!(matches!(
            source.value_components().items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
        let invalid = CssComponentValues::try_new(vec![
            CssComponentValue::try_ident("definitely-invalid-keyword").unwrap(),
        ])
        .unwrap();
        let error = parse_property_value(
            CssPropertyNameRef::Known(property),
            invalid,
            CssImportance::Normal,
        )
        .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
        let responsible = match error.origin() {
            CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => origin,
            other => panic!("invalid checked value lost its original component origin: {other:?}"),
        };
        assert_eq!(responsible, &CssValueOrigin::Programmatic);
    }
}

#[test]
fn intrinsic_longhand_metadata_has_the_pinned_initial_and_inheritance() {
    for &(name, initial_text, inherited, _, _) in CASES {
        let source = declaration(name, initial_text);
        let property = source.known().unwrap().property();
        let CssPropertyKindRef::Longhand(metadata) = property
            .metadata()
            .expect("intrinsic longhand metadata")
            .kind()
        else {
            panic!("{name} is a longhand")
        };
        assert_eq!(metadata.property().known_property(), property);
        assert_eq!(metadata.inherited_by_default(), inherited);
        let initial_value = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial_value.view() else {
            panic!("{name} has a fixed intrinsic initial")
        };
        assert_eq!(initial, contribution(&source).ordinary_value().unwrap());
    }
}

#[test]
fn ordinary_values_contribute_once_without_losing_occurrence_or_importance() {
    for &(name, _, _, valid, _) in CASES {
        for &text in valid {
            let parsed = declaration(name, text);
            let property = parsed.known().unwrap().property();
            let checked = parse_property_value(
                CssPropertyNameRef::Known(property),
                parsed.value_components().clone(),
                CssImportance::Important,
            )
            .unwrap();
            let parsed_item = contribution(&parsed);
            let checked_item = contribution(&checked);
            assert!(parsed_item.ordinary_value().is_some());
            assert_eq!(parsed_item.ordinary_value(), checked_item.ordinary_value());
            assert!(parsed_item.replacement_components().is_none());
            assert!(checked_item.replacement_components().is_none());
        }
    }
}

#[test]
fn css_wide_values_contribute_symbolic_keywords() {
    for &(name, _, _, _, _) in CASES {
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, text);
            assert_eq!(source.known().unwrap().global(), Some(keyword));
            assert_eq!(
                contribution(&source).value(),
                CssContributionValueRef::Global(keyword)
            );
        }
    }
}

#[test]
fn pending_reentry_is_strict_repeatable_and_retains_replacement_provenance() {
    for &(name, _, _, valid, invalid) in CASES {
        let source = declaration(name, "var(--value)");
        assert!(source.known().unwrap().substitution_dependent().is_some());
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("whole-value substitution remains pending")
        };
        assert!(handle.source().same_occurrence(&source));
        for &text in invalid {
            assert!(matches!(
                handle
                    .reenter(parse_component_values(text).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
        }
        for text in ["var(--again)", "env(value)", "attr(data-value)"] {
            assert_eq!(
                handle
                    .reenter(parse_component_values(text).unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
        }
        for text in [valid[0], "inherit"] {
            let replacement = parse_component_values(text).unwrap();
            for _ in 0..2 {
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("one reentered terminal")
                };
                let [item] = values.items() else {
                    panic!("one replacement contribution")
                };
                assert_eq!(item.property(), source.known().unwrap().property());
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
        }
    }
}

#[test]
fn normalization_retains_order_and_importance_with_an_exact_contribution_limit() {
    for &(name, _, _, valid, _) in CASES {
        let text = format!(
            ".a{{{name}:{}!important;color:red;{name}:{}}}",
            valid[0], valid[1]
        );
        let report = parse_sheet(&text);
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let declarations: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(declarations.len(), 3);
        for (order, expected) in [name, "color", name].into_iter().enumerate() {
            let item = declarations[order];
            assert_eq!(item.order(), order);
            assert_eq!(
                item.source().known().unwrap().property().canonical_name(),
                expected
            );
            let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
            else {
                panic!("one normalized terminal")
            };
            assert_eq!(values.items().len(), 1);
            assert!(values.items()[0].source().same_occurrence(item.source()));
        }
        assert_eq!(
            declarations[0].source().importance(),
            CssImportance::Important
        );
        assert_eq!(declarations[2].source().importance(), CssImportance::Normal);
        let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 2).unwrap();
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded {
                resource: CssNormalizationResource::Contributions,
                limit: 2,
            }
        );
        assert_eq!(error.declaration_order(), Some(2));
        assert_eq!(
            error
                .declaration()
                .unwrap()
                .known()
                .unwrap()
                .property()
                .canonical_name(),
            name
        );
        assert_eq!(
            normalize_sheet(report.syntax()).unwrap().items().len(),
            normalized.items().len()
        );
    }
}

#[test]
fn invalid_recovery_control_preserves_neighbors_and_the_original_snapshot() {
    // This is a recovery control on the baseline even where the property is absent.
    // Admission tests above independently require valid values to be supported.
    for &(name, _, _, _, invalid) in CASES {
        for &value in invalid {
            let text = format!("color:red;{name}:{value};height:2px!important");
            let report = parse_style_attribute(&text);
            let snapshot = report.syntax().to_vec();
            let [diagnostic] = report.diagnostics() else {
                panic!("one dropped invalid occurrence: {text}")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert_eq!(report.syntax().len(), 2);
            assert_eq!(
                report.syntax()[0].known().unwrap().property(),
                CssKnownProperty::Color
            );
            assert_eq!(
                report.syntax()[1].known().unwrap().property(),
                CssKnownProperty::Height
            );
            assert_eq!(report.syntax()[1].importance(), CssImportance::Important);
            assert_eq!(
                report.syntax()[0]
                    .value_components()
                    .serialize()
                    .unwrap()
                    .as_css(),
                "red"
            );
            assert_eq!(
                report.syntax()[1]
                    .value_components()
                    .serialize()
                    .unwrap()
                    .as_css(),
                "2px"
            );
            assert!(validate_style_attribute(&text).is_err());
            for (source, original) in report.syntax().iter().zip(&snapshot) {
                assert!(source.same_occurrence(original));
                assert_eq!(source.value_components(), original.value_components());
            }
        }
    }
}

#[test]
fn existing_color_lifecycle_control_exercises_the_same_shared_front_doors() {
    let source = declaration("color", "red");
    let item = contribution(&source);
    assert_eq!(item.property(), CssKnownProperty::Color);
    let source = declaration("color", "var(--control)");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("existing color control is pending")
    };
    assert!(matches!(
        handle
            .reenter(parse_component_values("inherit").unwrap())
            .unwrap(),
        CssContributions::Longhands(_)
    ));
    assert!(normalize_sheet(parse_sheet(".a{color:red!important;color:blue}").syntax()).is_ok());
}

#[test]
fn existing_scrollbar_width_admission_control_preserves_typed_keywords_and_origins() {
    for (text, expected) in [
        ("auto", CssScrollbarWidth::Auto),
        ("THIN", CssScrollbarWidth::Thin),
        (r"n\6f ne", CssScrollbarWidth::None),
    ] {
        let source = declaration("scrollbar-width", text);
        let CssKnownPropertyValueRef::ScrollbarWidth(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("existing scrollbar-width typed value")
        };
        assert_eq!(value.value(), &expected);
        assert_eq!(value.as_css(), text);
    }
}
