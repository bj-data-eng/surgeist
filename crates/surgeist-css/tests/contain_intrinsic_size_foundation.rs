#![forbid(unsafe_code)]

//! Authored contained intrinsic sizes from Sizing 4 (2026-09-04) §§5.2–5.2.1.
//! Last-remembered sizes and the logical-to-physical mapping need downstream context.

use surgeist_css::*;

const LONGHANDS: [&str; 4] = [
    "contain-intrinsic-width",
    "contain-intrinsic-height",
    "contain-intrinsic-block-size",
    "contain-intrinsic-inline-size",
];
const SHORTHAND: &str = "contain-intrinsic-size";

fn names() -> impl Iterator<Item = &'static str> {
    LONGHANDS.into_iter().chain([SHORTHAND])
}

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
    assert!(item.source().same_occurrence(source));
    item.ordinary_value().unwrap().clone()
}

#[test]
fn five_properties_have_distinct_identities_and_complete_dated_sizing4_support() {
    let mut identities = Vec::new();
    for name in names() {
        let grammar = grammar(name);
        assert_eq!(grammar.name(), name);
        assert_eq!(grammar.target_property().canonical_name(), name);
        assert_eq!(
            grammar.feature_id().as_str(),
            format!("ext.property.{name}")
        );
        assert!(
            !identities.contains(&grammar.target_property()),
            "{name} aliases an earlier property"
        );
        identities.push(grammar.target_property());
        let support = property_support_metadata(name).expect("selected support metadata");
        assert_eq!(support.property(), grammar.target_property());
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(
            support.feature().source().id().as_str(),
            "X-SIZING4-20260904"
        );
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
        accepted(name, "none");
    }
    assert_eq!(identities.len(), 5);
}

#[test]
fn four_longhands_accept_optional_auto_then_exact_nonnegative_length_or_none() {
    for name in LONGHANDS {
        for value in [
            "none",
            "auto none",
            "0",
            "-0",
            "-0px",
            "1px",
            "1e999px",
            "1e-999px",
            "auto 1e999px",
            "auto 1e-999px",
            "calc(1px + 2px)",
            "auto calc(-1px)",
        ] {
            accepted(name, value);
        }
        for value in [
            "auto",
            "none auto",
            "auto auto none",
            "1px auto",
            "-1px",
            "-1e-999px",
            "auto -1e-999px",
            "1%",
            "auto 1%",
            "1",
            "1e999",
            "1fr",
            "min-content",
            "calc(1px + 2%)",
            "calc(1 + 2)",
        ] {
            invalid(name, value);
        }
    }
}

#[test]
fn shorthand_parses_one_or_two_whole_axis_values_with_ordered_auto_prefixes() {
    for value in [
        "none",
        "1px",
        "auto none",
        "auto 1px",
        "1px 2px",
        "none auto 1px",
        "auto 1px 2px",
        "1px auto 2px",
        "auto none auto 3px",
        "auto 1e999px auto 1e-999px",
        "calc(1px + 2px) auto none",
    ] {
        accepted(SHORTHAND, value);
    }
    for value in [
        "auto",
        "none auto",
        "auto auto none",
        "auto none auto",
        "1px 2px 3px",
        "auto 1px 2px 3px",
        "none auto 1px 2px",
        "auto none auto 1px 2px",
        "logical 1px",
        "1px logical 2px",
        "1px 2%",
        "auto 1px -1e-999px",
        "1px auto -1px",
    ] {
        invalid(SHORTHAND, value);
    }
}

#[test]
fn four_terminals_have_fixed_none_initials_and_shorthand_sets_width_then_height() {
    for name in LONGHANDS {
        let property = grammar(name).target_property();
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a terminal longhand")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default(), "{name}");
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), property);
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("{name} has a fixed initial")
        };
        assert_eq!(value, &one_value(&declaration(name, "none")), "{name}");
    }

    let CssPropertyKindRef::Shorthand(shorthand) = grammar(SHORTHAND).metadata().unwrap().kind()
    else {
        panic!("contain-intrinsic-size is a shorthand")
    };
    assert!(!shorthand.is_legacy());
    assert!(shorthand.reset_only_members().is_empty());
    assert_eq!(
        shorthand
            .settable_members()
            .iter()
            .map(|member| member.known_property().canonical_name())
            .collect::<Vec<_>>(),
        ["contain-intrinsic-width", "contain-intrinsic-height"]
    );
    for (authored, width, height) in [
        ("none", "none", "none"),
        ("auto 1px", "auto 1px", "auto 1px"),
        ("1px 2px", "1px", "2px"),
        ("auto 1px 2px", "auto 1px", "2px"),
        ("1px auto 2px", "1px", "auto 2px"),
        ("auto none auto 3px", "auto none", "auto 3px"),
    ] {
        let source = declaration(SHORTHAND, authored);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("two contained intrinsic size contributions")
        };
        let [width_item, height_item] = values.items() else {
            panic!("exactly width then height")
        };
        for (item, member, expected) in [
            (width_item, LONGHANDS[0], width),
            (height_item, LONGHANDS[1], height),
        ] {
            assert_eq!(item.property(), grammar(member).target_property());
            assert_eq!(
                item.ordinary_value(),
                Some(&one_value(&declaration(member, expected)))
            );
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
}

#[test]
fn css_wide_values_all_reset_and_substitutions_keep_symbolic_provenance() {
    let all_source = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source).unwrap()
    else {
        panic!("symbolic all reset")
    };
    for name in names() {
        assert!(
            !reset.excludes(CssPropertyNameRef::Known(grammar(name).target_property())),
            "all includes {name}"
        );
        invalid(name, "initial 1px");
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
                panic!("{name} has symbolic global contributions")
            };
            assert_eq!(values.items().len(), if name == SHORTHAND { 2 } else { 1 });
            for item in values.items() {
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
        for pending in ["var(--intrinsic)", "env(--intrinsic)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name} keeps substitution pending")
            };
            assert!(handle.source().same_occurrence(&source));
            for invalid in ["auto", "-1e-999px", "1%"] {
                assert!(
                    matches!(
                        handle
                            .reenter(parse_component_values(invalid).unwrap())
                            .unwrap_err()
                            .kind(),
                        CssExpansionErrorKind::InvalidReplacement(_)
                    ),
                    "{name}: {invalid}"
                );
            }
            assert_eq!(
                handle
                    .reenter(parse_component_values("var(--again)").unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let replacement = parse_component_values("auto 2px").unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("{name} reenters to terminal contributions")
            };
            assert_eq!(values.items().len(), if name == SHORTHAND { 2 } else { 1 });
            for item in values.items() {
                assert_eq!(
                    item.ordinary_value(),
                    Some(&one_value(&declaration(
                        item.property().canonical_name(),
                        "auto 2px",
                    )))
                );
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
            let CssContributions::Longhands(globals) = handle
                .reenter(parse_component_values("inherit").unwrap())
                .unwrap()
            else {
                panic!("{name} reenters to global contributions")
            };
            for item in globals.items() {
                assert_eq!(
                    item.value(),
                    CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                );
                assert!(item.source().same_occurrence(&source));
            }
        }
    }
}

#[test]
fn normalization_keeps_authored_physical_and_logical_order_after_invalid_recovery() {
    let sheet = ".a{contain-intrinsic-width:auto 1px;contain-intrinsic-block-size:2px;contain-intrinsic-size:auto none auto 3px;contain-intrinsic-height:-1px;contain-intrinsic-inline-size:none}";
    let report = parse_sheet(sheet);
    let [diagnostic] = report.diagnostics() else {
        panic!("one negative-height diagnostic: {:?}", report.diagnostics())
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
    for (index, name) in [
        "contain-intrinsic-width",
        "contain-intrinsic-block-size",
        "contain-intrinsic-size",
        "contain-intrinsic-inline-size",
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations[index].order(), index);
        assert_eq!(
            declarations[index].source().known().unwrap().property(),
            grammar(name).target_property()
        );
    }
}
