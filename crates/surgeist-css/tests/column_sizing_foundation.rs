#![forbid(unsafe_code)]

//! Authored columns from Multicol 1 CR (2024-05-16) §§3.1–3.3, with the
//! `column-width` box-size extension from Sizing 4 WD (2026-09-04) §5.6.

use surgeist_css::*;

const NAMES: [&str; 3] = ["column-count", "column-width", "columns"];

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("selected property: {name}"))
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

fn accepted(name: &str, value: &str) {
    for source in [declaration(name, value), checked(name, value)] {
        assert_eq!(
            source.known().unwrap().property(),
            grammar(name).target_property()
        );
        assert_eq!(source.importance(), CssImportance::Important);
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            value
        );
    }
}

fn invalid(name: &str, value: &str) {
    let source = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid-value diagnostic: {source}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
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
    item.ordinary_value().unwrap().clone()
}

fn typed_count(value: &str) -> CssColumnCount {
    let source = declaration("column-count", value);
    let CssKnownPropertyValueRef::ColumnCount(checked) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("existing column-count wrapper")
    };
    checked.count().clone()
}

#[test]
fn three_names_retain_official_identities_and_complete_authored_support() {
    let mut properties = Vec::new();
    for name in NAMES {
        let grammar = grammar(name);
        let property = grammar.target_property();
        assert_eq!(grammar.name(), name);
        assert_eq!(property.canonical_name(), name);
        assert_eq!(
            grammar.feature_id().as_str(),
            format!("official.property.{name}")
        );
        assert!(
            !properties.contains(&property),
            "{name} aliases another property"
        );
        properties.push(property);
        let support = property_support_metadata(name).expect("selected support metadata");
        assert_eq!(support.property(), property);
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(support.feature().source().id().as_str(), "O-MULTICOL1");
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
    }
    assert_eq!(properties.len(), 3);
    let extension = feature_metadata("ext.value.column-width.box-size")
        .expect("selected additive box-size value record");
    assert_eq!(extension.kind(), CssFeatureKind::Value);
    assert_eq!(extension.status(), CssSupportStatus::Complete);
    assert_eq!(extension.source().id().as_str(), "X-SIZING4-20260904");
    assert_eq!(
        extension.source().url(),
        Some("https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/")
    );
    assert_eq!(extension.production(), "#column-sizing");
}

#[test]
fn count_accepts_positive_integers_without_saturating_distinct_authored_values() {
    for value in [
        "auto",
        "1",
        "+0001",
        "2",
        "2147483647",
        "2147483648",
        "2147483649",
        "9999999999999999999999",
        "calc(1 + 2)",
        "calc(-1)",
    ] {
        accepted("column-count", value);
    }
    for value in ["0", "-0", "-1", "1.0", "1e0", "1px", "1%", "none", "auto 2"] {
        invalid("column-count", value);
    }
    let at_limit = typed_count("2147483647");
    let just_above = typed_count("2147483648");
    let next = typed_count("2147483649");
    let very_large = typed_count("9999999999999999999999");
    for count in [&at_limit, &just_above, &next, &very_large] {
        assert!(matches!(count, CssColumnCount::Count(_)));
    }
    assert_ne!(at_limit, just_above, "the i32 boundary must not saturate");
    assert_ne!(just_above, next, "neighboring large counts stay distinct");
    assert_ne!(next, very_large, "very large counts stay distinct");
}

#[test]
fn width_uses_nonnegative_box_size_values_without_early_math_resolution() {
    for value in [
        "auto",
        "0",
        "-0px",
        "1px",
        "1%",
        "1e999px",
        "1e-999px",
        "1e-999%",
        "stretch",
        "contain",
        "min-content",
        "max-content",
        "fit-content",
        "fit-content(2%)",
        "calc(1px - 2px)",
        "calc-size(min-content, size + 1px)",
    ] {
        accepted("column-width", value);
    }
    for value in [
        "-1px",
        "-1e-999%",
        "1",
        "1fr",
        "none",
        "fit-content(-1px)",
        "calc(1 + 2)",
        "auto 1px",
    ] {
        invalid("column-width", value);
    }
    assert_ne!(
        one_value(&declaration("column-width", "1%")),
        one_value(&declaration("column-width", "1px")),
        "percentage width remains a distinct authored value"
    );
}

#[test]
fn columns_accepts_unordered_width_and_count_with_auto_for_omissions() {
    for value in [
        "auto",
        "auto auto",
        "0",
        "2",
        "10px",
        "10%",
        "min-content",
        "calc-size(min-content, size + 1px)",
        "auto 2",
        "2 auto",
        "auto 10px",
        "10px auto",
        "10% 2",
        "2 10%",
        "fit-content(1px) 3",
        "3 fit-content(1px)",
        "2147483648 1e999px",
    ] {
        accepted("columns", value);
    }
    for value in [
        "",
        "0 0",
        "1.0",
        "-1px 2",
        "2 3",
        "10px 20px",
        "auto auto auto",
        "2 10px auto",
        "1fr 2",
        "none 2",
        "initial 2",
    ] {
        invalid("columns", value);
    }
    let metadata = grammar("columns").metadata().unwrap();
    let CssPropertyKindRef::Shorthand(shorthand) = metadata.kind() else {
        panic!("columns is a shorthand")
    };
    assert!(!shorthand.is_legacy());
    assert!(shorthand.reset_only_members().is_empty());
    assert_eq!(
        shorthand
            .settable_members()
            .iter()
            .map(|member| member.known_property().canonical_name())
            .collect::<Vec<_>>(),
        ["column-width", "column-count"]
    );
    for (authored, width, count) in [
        ("auto", "auto", "auto"),
        ("2", "auto", "2"),
        ("10px", "10px", "auto"),
        ("2 10%", "10%", "2"),
        ("10% 2", "10%", "2"),
        ("auto 10px", "10px", "auto"),
        ("2147483648 1e999px", "1e999px", "2147483648"),
    ] {
        let source = declaration("columns", authored);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("columns expands to width and count")
        };
        let [width_item, count_item] = values.items() else {
            panic!("exactly width then count")
        };
        for (item, name, expected) in [
            (width_item, "column-width", width),
            (count_item, "column-count", count),
        ] {
            assert_eq!(item.property(), grammar(name).target_property());
            assert_eq!(
                item.ordinary_value(),
                Some(&one_value(&declaration(name, expected)))
            );
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
}

#[test]
fn two_terminals_have_auto_initials_and_all_three_participate_in_global_reset() {
    for name in ["column-count", "column-width"] {
        let property = grammar(name).target_property();
        let metadata = grammar(name).metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("{name} is a terminal")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), property);
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("{name} has fixed auto initial")
        };
        assert_eq!(value, &one_value(&declaration(name, "auto")));
    }
    let all_source = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source).unwrap()
    else {
        panic!("symbolic all reset")
    };
    for name in NAMES {
        assert!(!reset.excludes(CssPropertyNameRef::Known(grammar(name).target_property())));
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
                panic!("{name} has global contributions")
            };
            let expected_names: &[&str] = if name == "columns" {
                &["column-width", "column-count"]
            } else {
                &[name]
            };
            assert_eq!(values.items().len(), expected_names.len());
            for (item, expected_name) in values.items().iter().zip(expected_names) {
                assert_eq!(item.property(), grammar(expected_name).target_property());
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
    }
}

#[test]
fn pending_reentry_and_normalization_preserve_columns_and_neighbor_order() {
    for name in NAMES {
        for pending in ["var(--columns)", "env(--columns)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name} retains pending substitution")
            };
            assert!(handle.source().same_occurrence(&source));
            let bad = if name == "column-count" { "0" } else { "1fr" };
            assert!(matches!(
                handle
                    .reenter(parse_component_values(bad).unwrap())
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
            let replacement_text = match name {
                "column-count" => "2147483648",
                "column-width" => "10%",
                "columns" => "2 10%",
                _ => unreachable!(),
            };
            let replacement = parse_component_values(replacement_text).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("{name} reenters to terminal contributions")
            };
            let expected: &[(&str, &str)] = match name {
                "column-count" => &[("column-count", "2147483648")],
                "column-width" => &[("column-width", "10%")],
                "columns" => &[("column-width", "10%"), ("column-count", "2")],
                _ => unreachable!(),
            };
            assert_eq!(values.items().len(), expected.len());
            for (item, (member, value)) in values.items().iter().zip(expected) {
                assert_eq!(item.property(), grammar(member).target_property());
                assert_eq!(
                    item.ordinary_value(),
                    Some(&one_value(&declaration(member, value)))
                );
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
        }
    }
    let report = parse_sheet(
        ".a{column-count:2;column-width:10%;columns:3 min-content;columns:2 3;column-width:0}",
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid columns diagnostic: {:?}", report.diagnostics())
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 4);
    for (index, name) in ["column-count", "column-width", "columns", "column-width"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(declarations[index].order(), index);
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
