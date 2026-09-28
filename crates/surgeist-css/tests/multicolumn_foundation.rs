#![forbid(unsafe_code)]

//! Authored fill, rule, and span from Multicol 1 CR (2024-05-16) §§4, 6.1, 7.1:
//! https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/

use surgeist_css::*;

const NAMES: [&str; 6] = [
    "column-fill",
    "column-rule",
    "column-rule-color",
    "column-rule-style",
    "column-rule-width",
    "column-span",
];
const RULE_MEMBERS: [&str; 3] = [
    "column-rule-width",
    "column-rule-style",
    "column-rule-color",
];

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
    }
}

fn invalid(name: &str, value: &str) {
    let source = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one invalid-value diagnostic: {source}: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(
        report.syntax().len(),
        2,
        "valid neighbors survive: {source}"
    );
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

fn expanded(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!(
            "terminal contributions for {:?}",
            source.known().unwrap().property()
        )
    };
    values.items().to_vec()
}

fn one_value(name: &str, value: &str) -> CssLonghandValue {
    let source = declaration(name, value);
    let items = expanded(&source);
    let [item] = items.as_slice() else {
        panic!("one terminal for {name}")
    };
    assert_eq!(item.property(), grammar(name).target_property());
    item.ordinary_value().unwrap().clone()
}

#[test]
fn six_properties_have_dated_provenance_and_intrinsic_longhand_or_shorthand_shapes() {
    let mut seen = Vec::new();
    for name in NAMES {
        let property = grammar(name).target_property();
        assert_eq!(property.canonical_name(), name);
        assert!(
            !seen.contains(&property),
            "duplicate property identity: {name}"
        );
        seen.push(property);
        let support = property_support_metadata(name).unwrap();
        assert_eq!(support.property(), property);
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(support.feature().source().id().as_str(), "O-MULTICOL1");
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
        if name == "column-rule" {
            let CssPropertyKindRef::Shorthand(shorthand) = grammar(name).metadata().unwrap().kind()
            else {
                panic!("column-rule is a shorthand")
            };
            assert!(!shorthand.is_legacy());
            assert!(shorthand.reset_only_members().is_empty());
            assert_eq!(
                shorthand
                    .settable_members()
                    .iter()
                    .map(|member| member.known_property().canonical_name())
                    .collect::<Vec<_>>(),
                RULE_MEMBERS
            );
        } else {
            let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
            else {
                panic!("{name} is a longhand")
            };
            assert_eq!(longhand.property().known_property(), property);
            assert!(!longhand.inherited_by_default(), "{name} is non-inherited");
            let initial = longhand.initial_value();
            let CssInitialValueRef::Value(initial) = initial.view() else {
                panic!("{name} has a fixed initial value")
            };
            let initial_text = match name {
                "column-fill" => "balance",
                "column-rule-color" => "currentcolor",
                "column-rule-style" => "none",
                "column-rule-width" => "medium",
                "column-span" => "none",
                _ => unreachable!(),
            };
            assert_eq!(initial, &one_value(name, initial_text));
        }
    }
}

#[test]
fn fill_span_and_rule_longhands_accept_their_authored_grammars() {
    for value in ["auto", "balance", "balance-all"] {
        accepted("column-fill", value);
    }
    for value in ["none", "all"] {
        accepted("column-span", value);
    }
    for value in [
        "none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset",
        "outset",
    ] {
        accepted("column-rule-style", value);
    }
    for value in [
        "currentcolor",
        "red",
        "#123456",
        "rgb(10 20 30)",
        "lab(50 0 0)",
    ] {
        accepted("column-rule-color", value);
    }
    for value in [
        "thin",
        "medium",
        "thick",
        "0",
        "-0px",
        "1px",
        "1e100px",
        "1e-100px",
        "calc(1px + 2em)",
        "calc((-1px + 2px) * 3)",
    ] {
        accepted("column-rule-width", value);
    }
    for (name, value) in [
        ("column-fill", "justify"),
        ("column-span", "2"),
        ("column-rule-style", "solid dashed"),
        ("column-rule-color", "none"),
        ("column-rule-width", "-1px"),
        ("column-rule-width", "1%"),
        ("column-rule-width", "calc(1px + 2%)"),
    ] {
        invalid(name, value);
    }
    assert_ne!(
        one_value("column-rule-width", "1e100px"),
        one_value("column-rule-width", "1e101px"),
        "distinct large finite authored widths must not round into one legacy float"
    );
}

#[test]
fn unordered_rule_components_expand_with_omissions_reset_to_initials() {
    for (authored, width, style, color) in [
        ("2px solid red", "2px", "solid", "red"),
        ("red solid 2px", "2px", "solid", "red"),
        ("dashed", "medium", "dashed", "currentcolor"),
        ("thin blue", "thin", "none", "blue"),
        ("currentcolor", "medium", "none", "currentcolor"),
    ] {
        let source = declaration("column-rule", authored);
        let items = expanded(&source);
        assert_eq!(items.len(), 3);
        for (item, (name, value)) in items.iter().zip([
            ("column-rule-width", width),
            ("column-rule-style", style),
            ("column-rule-color", color),
        ]) {
            assert_eq!(item.property(), grammar(name).target_property());
            assert_eq!(item.ordinary_value(), Some(&one_value(name, value)));
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
    for value in [
        "",
        "solid solid",
        "1px 2px",
        "red blue",
        "-1px solid",
        "1% solid",
    ] {
        invalid("column-rule", value);
    }
}

#[test]
fn globals_and_pending_values_expand_all_six_properties_with_strict_reentry() {
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&declaration("all", "initial")).unwrap()
    else {
        panic!("symbolic universal reset")
    };
    for name in NAMES {
        assert!(!reset.excludes(CssPropertyNameRef::Known(grammar(name).target_property())));
        let expected: &[&str] = if name == "column-rule" {
            &RULE_MEMBERS
        } else {
            &[name]
        };
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, text);
            checked(name, text);
            let items = expanded(&source);
            assert_eq!(items.len(), expected.len());
            for (item, member) in items.iter().zip(expected.iter()) {
                assert_eq!(item.property(), grammar(member).target_property());
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
        for pending in ["var(--multicol)", "env(--multicol)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name} remains pending until substitution")
            };
            assert!(handle.source().same_occurrence(&source));
            assert_eq!(
                handle
                    .reenter(parse_component_values("var(--again)").unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let (valid, invalid_text) = match name {
                "column-fill" => ("balance-all", "justify"),
                "column-rule" => ("red dashed 2px", "red blue"),
                "column-rule-color" => ("lab(50 0 0)", "none"),
                "column-rule-style" => ("double", "solid dashed"),
                "column-rule-width" => ("1e100px", "1%"),
                "column-span" => ("all", "2"),
                _ => unreachable!(),
            };
            assert!(matches!(
                handle
                    .reenter(parse_component_values(invalid_text).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
            let replacement = parse_component_values(valid).unwrap();
            for _ in 0..2 {
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("{name} reenters to terminal longhands")
                };
                assert_eq!(values.items().len(), expected.len());
                for (item, member) in values.items().iter().zip(expected.iter()) {
                    assert_eq!(item.property(), grammar(member).target_property());
                    assert!(item.ordinary_value().is_some());
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
            }
            let CssContributions::Longhands(values) = handle
                .reenter(parse_component_values("inherit").unwrap())
                .unwrap()
            else {
                panic!("{name} reenters a global keyword")
            };
            assert_eq!(values.items().len(), expected.len());
            for item in values.items() {
                assert_eq!(
                    item.value(),
                    CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                );
            }
        }
        invalid(name, "initial red");
    }
}

#[test]
fn normalization_keeps_mixed_order_and_fails_atomically_at_contribution_limit() {
    let report = parse_sheet(concat!(
        ".a{column-fill:balance;column-rule:red solid 2px;",
        "column-span:all;column-rule-width:1%}",
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid width diagnostic: {:?}", report.diagnostics())
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
    for (index, (name, members)) in [("column-fill", 1), ("column-rule", 3), ("column-span", 1)]
        .into_iter()
        .enumerate()
    {
        assert_eq!(declarations[index].order(), index);
        assert_eq!(
            declarations[index].source().known().unwrap().property(),
            grammar(name).target_property()
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declarations[index].expansion()
        else {
            panic!("normalized terminal for {name}")
        };
        assert_eq!(values.items().len(), members);
        for item in values.items() {
            assert!(item.source().same_occurrence(declarations[index].source()));
        }
    }
    let limit = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 3).unwrap();
    let error = normalize_sheet_with_limits(report.syntax(), limit).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 3,
        }
    );
    assert_eq!(error.declaration_order(), Some(1));
    assert_eq!(
        error.declaration().unwrap().known().unwrap().property(),
        CssKnownProperty::ColumnRule
    );
}
