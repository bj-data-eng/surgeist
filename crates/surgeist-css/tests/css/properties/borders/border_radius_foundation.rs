#![forbid(unsafe_code)]

//! Backgrounds 3 CRD (2024-03-11) §4.1 and Logical 1 WD (2025-12-04) §4.6:
//! authored physical and flow-relative corner radii, before box-size resolution.

use surgeist_css::*;

const PHYSICAL: [&str; 4] = [
    "border-top-left-radius",
    "border-top-right-radius",
    "border-bottom-right-radius",
    "border-bottom-left-radius",
];
const LOGICAL: [&str; 4] = [
    "border-start-start-radius",
    "border-start-end-radius",
    "border-end-start-radius",
    "border-end-end-radius",
];
const SHORTHAND: &str = "border-radius";

fn names() -> impl Iterator<Item = &'static str> {
    PHYSICAL.into_iter().chain(LOGICAL).chain([SHORTHAND])
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

fn one_value(name: &str, value: &str) -> CssLonghandValue {
    let source = declaration(name, value);
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("{name} has a terminal contribution")
    };
    let [item] = values.items() else {
        panic!("{name} has one terminal contribution")
    };
    assert_eq!(item.property(), grammar(name).target_property());
    assert!(item.source().same_occurrence(&source));
    item.ordinary_value().unwrap().clone()
}

#[test]
fn nine_radius_properties_have_distinct_stable_ids_and_dated_sources() {
    let mut seen = Vec::new();
    for name in names() {
        let grammar = grammar(name);
        let property = grammar.target_property();
        let logical = LOGICAL.contains(&name);
        assert_eq!(grammar.name(), name);
        assert_eq!(property.canonical_name(), name);
        assert_eq!(
            grammar.feature_id().as_str(),
            format!(
                "{}.property.{name}",
                if logical { "official" } else { "baseline" }
            )
        );
        assert!(
            !seen.contains(&property),
            "{name} aliases an earlier property"
        );
        seen.push(property);
        let support = property_support_metadata(name).expect("selected support metadata");
        assert_eq!(support.property(), property);
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(
            support.feature().source().id().as_str(),
            if logical {
                "I-LOGICAL1-20251204"
            } else {
                "O-BACKGROUNDS3"
            }
        );
        assert_eq!(
            support.feature().source().url(),
            Some(if logical {
                "https://www.w3.org/TR/2025/WD-css-logical-1-20251204/"
            } else {
                "https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/"
            })
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
    }
    assert_eq!(seen.len(), 9);
}

#[test]
fn all_corner_longhands_accept_exact_nonnegative_length_percentages_and_symbolic_math() {
    for name in PHYSICAL.into_iter().chain(LOGICAL) {
        for value in [
            "0",
            "-0",
            "-0px",
            "-0%",
            "1px",
            "25%",
            "1e100px",
            "1e100%",
            "1e-100px",
            "1e-100%",
            "1px 2%",
            "1e100px 1e-100%",
            "calc(1px + 2%)",
            "calc((-1px + 2%) * 3)",
        ] {
            accepted(name, value);
        }
        for value in [
            "",
            "-1px",
            "-1%",
            "-1e-100px",
            "-1e-100%",
            "1",
            "1fr",
            "auto",
            "min-content",
            "1px 2px 3px",
            "1px / 2px",
            "calc(1 + 2)",
        ] {
            invalid(name, value);
        }
    }
}

#[test]
fn eight_corner_longhands_have_fixed_zero_initials_and_do_not_inherit() {
    for name in PHYSICAL.into_iter().chain(LOGICAL) {
        let property = grammar(name).target_property();
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a corner longhand")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        let CssInitialValueRef::Value(initial_value) = initial.view() else {
            panic!("{name} has a fixed zero initial")
        };
        assert_eq!(initial_value, &one_value(name, "0"));
    }
}

#[test]
fn shorthand_accepts_one_to_four_radii_on_each_side_of_optional_slash() {
    for value in [
        "0",
        "1px",
        "1px 2px",
        "1px 2px 3px",
        "1px 2px 3px 4px",
        "1px / 2%",
        "1px 2px / 3% 4%",
        "1px 2px 3px / 4% 5%",
        "1px 2px 3px 4px / 5% 6% 7% 8%",
        "1e100px 1e-100% / calc(1px + 2%) 0",
    ] {
        accepted(SHORTHAND, value);
    }
    for value in [
        "",
        "/ 1px",
        "1px /",
        "1px / / 2px",
        "1px / 2px / 3px",
        "1px 2px 3px 4px 5px",
        "1px / 2px 3px 4px 5px 6px",
        "-1e-100px",
        "1px / -1e-100%",
        "logical 1px",
        "auto",
        "1px 2px / auto",
        "1px 2px 3px 4px 5px / 6px",
    ] {
        invalid(SHORTHAND, value);
    }
}

#[test]
fn physical_shorthand_expands_four_corners_in_top_left_clockwise_order() {
    let CssPropertyKindRef::Shorthand(metadata) = grammar(SHORTHAND).metadata().unwrap().kind()
    else {
        panic!("border-radius is a shorthand")
    };
    assert!(!metadata.is_legacy());
    assert!(metadata.reset_only_members().is_empty());
    assert_eq!(
        metadata
            .settable_members()
            .iter()
            .map(|member| member.known_property().canonical_name())
            .collect::<Vec<_>>(),
        PHYSICAL
    );
    for (authored, corners) in [
        ("1px", ["1px", "1px", "1px", "1px"]),
        ("1px 2px", ["1px", "2px", "1px", "2px"]),
        ("1px 2px 3px", ["1px", "2px", "3px", "2px"]),
        ("1px 2px 3px 4px", ["1px", "2px", "3px", "4px"]),
        ("1px / 2%", ["1px 2%", "1px 2%", "1px 2%", "1px 2%"]),
        (
            "1px 2px 3px / 4% 5%",
            ["1px 4%", "2px 5%", "3px 4%", "2px 5%"],
        ),
    ] {
        let source = declaration(SHORTHAND, authored);
        checked(SHORTHAND, authored);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("border-radius expands to physical corners")
        };
        assert_eq!(values.items().len(), 4);
        for (item, (member, expected)) in
            values.items().iter().zip(PHYSICAL.into_iter().zip(corners))
        {
            assert_eq!(item.property(), grammar(member).target_property());
            assert_eq!(item.ordinary_value(), Some(&one_value(member, expected)));
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
}

#[test]
fn css_wide_all_and_pending_reentry_preserve_members_and_occurrences() {
    let all_source = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source).unwrap()
    else {
        panic!("all is a symbolic universal reset")
    };
    assert!(reset.source().same_occurrence(&all_source));
    for name in names() {
        assert!(!reset.excludes(CssPropertyNameRef::Known(grammar(name).target_property())));
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
            assert_eq!(values.items().len(), if name == SHORTHAND { 4 } else { 1 });
            for item in values.items() {
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
        for pending in ["var(--radius)", "env(--radius)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name} retains pending substitution")
            };
            assert!(handle.source().same_occurrence(&source));
            assert!(matches!(
                handle
                    .reenter(parse_component_values("-1e-100px").unwrap())
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
            let replacement = parse_component_values("1px 2%").unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("{name} reenters to terminal contributions")
            };
            assert_eq!(values.items().len(), if name == SHORTHAND { 4 } else { 1 });
            for item in values.items() {
                assert!(item.ordinary_value().is_some());
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
        }
    }
}

#[test]
fn normalization_retains_logical_corner_names_and_order_without_mapping() {
    let report = parse_sheet(
        ".a{border-top-left-radius:1px;border-start-end-radius:2% 3px;border-radius:4px / 5%;border-end-start-radius:6px}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let authored = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => {
                Some(value.source().known().unwrap().property())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        authored,
        [
            "border-top-left-radius",
            "border-start-end-radius",
            "border-radius",
            "border-end-start-radius",
        ]
        .map(|name| grammar(name).target_property())
    );
    let report = parse_sheet(
        ".a{border-start-start-radius:var(--corner);border-end-end-radius:env(--corner)}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(normalized.items().iter().filter(|item| matches!(item,
        CssNormalizedItem::Declaration(value) if matches!(value.expansion(), CssExpansion::Pending(_))
    )).count(), 2);
}
