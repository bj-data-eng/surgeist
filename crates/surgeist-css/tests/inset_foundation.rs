#![forbid(unsafe_code)]

//! Authored positioning and insets from Position 3 WD (2025-10-07) §§2, 3.1–3.2
//! and the optional logical four-side switch from Logical 1 WD (2025-12-04) §4.7.

use surgeist_css::*;

const LONGHANDS: [&str; 8] = [
    "top",
    "right",
    "bottom",
    "left",
    "inset-block-start",
    "inset-block-end",
    "inset-inline-start",
    "inset-inline-end",
];
const PAIRS: [&str; 2] = ["inset-block", "inset-inline"];
const FOUR_SIDE: &str = "inset";

fn names() -> impl Iterator<Item = &'static str> {
    ["position"]
        .into_iter()
        .chain(LONGHANDS)
        .chain(PAIRS)
        .chain([FOUR_SIDE])
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
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    item.ordinary_value().unwrap().clone()
}

fn unresolved_kind() -> CssExpansionErrorKind {
    CssExpansionErrorKind::UnresolvedStandard {
        property: grammar(FOUR_SIDE).target_property(),
        reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
    }
}

#[test]
fn twelve_names_keep_stable_ids_and_selected_position3_provenance() {
    let mut properties = Vec::new();
    for name in names() {
        let grammar = grammar(name);
        let property = grammar.target_property();
        assert_eq!(grammar.name(), name);
        assert_eq!(property.canonical_name(), name);
        assert_eq!(
            grammar.feature_id().as_str(),
            format!(
                "{}.property.{name}",
                if name.contains("block") || name.contains("inline") {
                    "official"
                } else {
                    "baseline"
                }
            )
        );
        assert!(
            !properties.contains(&property),
            "{name} aliases an earlier property"
        );
        properties.push(property);
        let support = property_support_metadata(name).expect("selected support metadata");
        assert_eq!(support.property(), property);
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(support.feature().source().id().as_str(), "I-POSITION3");
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2025/WD-css-position-3-20251007/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
    }
    assert_eq!(properties.len(), 12);
}

#[test]
fn position_accepts_exact_five_keywords_and_has_static_noninherited_initial() {
    for (keyword, expected) in [
        ("static", CssLayoutPosition::Static),
        ("relative", CssLayoutPosition::Relative),
        ("absolute", CssLayoutPosition::Absolute),
        ("sticky", CssLayoutPosition::Sticky),
        ("fixed", CssLayoutPosition::Fixed),
    ] {
        accepted("position", keyword);
        let source = declaration("position", keyword);
        let CssKnownPropertyValueRef::Position(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("existing position wrapper")
        };
        assert_eq!(
            value.value(),
            &expected,
            "{keyword} retains its position value"
        );
        assert!(
            one_value(&source).property().known_property() == grammar("position").target_property()
        );
    }
    for value in ["auto", "center", "1px", "static fixed", "initial fixed"] {
        invalid("position", value);
    }
    let CssPropertyKindRef::Longhand(longhand) = grammar("position").metadata().unwrap().kind()
    else {
        panic!("position is a terminal")
    };
    assert!(!longhand.inherited_by_default());
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("position has a fixed initial")
    };
    assert_eq!(initial, &one_value(&declaration("position", "static")));
}

#[test]
fn eight_insets_accept_exact_signed_length_percentages_or_auto() {
    for name in LONGHANDS {
        for value in [
            "auto",
            "0",
            "-0",
            "-0px",
            "1px",
            "-1px",
            "1%",
            "-1%",
            "1e999px",
            "-1e999px",
            "1e-999px",
            "-1e-999%",
            "calc(1px + 2%)",
        ] {
            accepted(name, value);
        }
        for value in [
            "1",
            "-1",
            "1fr",
            "min-content",
            "max-content",
            "fit-content(1px)",
            "stretch",
            "contain",
            "auto 1px",
            "calc(1 + 2)",
        ] {
            invalid(name, value);
        }
        let property = grammar(name).target_property();
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a terminal")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default());
        let initial_value = longhand.initial_value();
        let CssInitialValueRef::Value(initial) = initial_value.view() else {
            panic!("{name} has fixed auto initial")
        };
        assert_eq!(initial, &one_value(&declaration(name, "auto")));
    }
    for name in ["top", "right", "bottom", "left"] {
        let source = declaration(name, "1px");
        let inset = match source.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::Top(value) => value.value(),
            CssKnownPropertyValueRef::Right(value) => value.value(),
            CssKnownPropertyValueRef::Bottom(value) => value.value(),
            CssKnownPropertyValueRef::Left(value) => value.value(),
            _ => panic!("existing physical inset wrapper"),
        };
        assert_eq!(inset.serialize_specified().unwrap(), "1px");
    }
}

#[test]
fn axis_pairs_repeat_start_at_end_without_physical_mapping() {
    for pair in PAIRS {
        let members = [format!("{pair}-start"), format!("{pair}-end")];
        let CssPropertyKindRef::Shorthand(shorthand) = grammar(pair).metadata().unwrap().kind()
        else {
            panic!("{pair} is an axis shorthand")
        };
        assert!(!shorthand.is_legacy());
        assert!(shorthand.reset_only_members().is_empty());
        assert_eq!(
            shorthand
                .settable_members()
                .iter()
                .map(|member| member.known_property().canonical_name())
                .collect::<Vec<_>>(),
            members
        );
        for (authored, start, end) in [
            ("auto", "auto", "auto"),
            ("-1px", "-1px", "-1px"),
            ("1px -2%", "1px", "-2%"),
            ("auto 1e999px", "auto", "1e999px"),
        ] {
            let source = declaration(pair, authored);
            checked(pair, authored);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("{pair} expands to two logical longhands")
            };
            let [start_item, end_item] = values.items() else {
                panic!("start then end")
            };
            for (item, member, expected) in [
                (start_item, members[0].as_str(), start),
                (end_item, members[1].as_str(), end),
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
        for value in ["", "logical 1px", "1px 2px 3px", "1fr", "initial auto"] {
            invalid(pair, value);
        }
    }
}

#[test]
fn inset_grammar_is_complete_while_four_side_reset_membership_remains_typed_unresolved() {
    for value in [
        "auto",
        "1px 2%",
        "1px 2% 3px",
        "1px 2% 3px 4%",
        "logical auto",
        "logical -1px 2%",
        "logical 1px 2% 3px 4%",
    ] {
        accepted(FOUR_SIDE, value);
        let source = declaration(FOUR_SIDE, value);
        assert_eq!(
            expand_declaration(&source).unwrap_err().kind(),
            &unresolved_kind()
        );
    }
    for value in [
        "",
        "logical",
        "1px logical",
        "logical logical 1px",
        "1px 2px 3px 4px 5px",
        "logical 1px 2px 3px 4px 5px",
        "logical 1fr",
    ] {
        invalid(FOUR_SIDE, value);
    }
    let grammar = grammar(FOUR_SIDE);
    assert_eq!(
        grammar.metadata().unwrap_err(),
        CssPropertyMetadataError::UnresolvedStandard {
            grammar,
            reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
        }
    );
    for value in ["initial", "inherit", "unset", "revert", "revert-layer"] {
        let source = declaration(FOUR_SIDE, value);
        checked(FOUR_SIDE, value);
        assert_eq!(
            expand_declaration(&source).unwrap_err().kind(),
            &unresolved_kind()
        );
    }
    let report = parse_sheet(".a{color:red;inset:logical 1px 2%;color:blue}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let syntax = report.syntax().clone();
    let error = normalize_sheet(report.syntax()).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssNormalizationErrorKind::UnsupportedDeclaration(expansion)
            if expansion.kind() == &unresolved_kind()
    ));
    assert_eq!(error.declaration_order(), Some(1));
    assert_eq!(
        error.declaration().unwrap().known().unwrap().property(),
        grammar.target_property()
    );
    assert_eq!(report.syntax(), &syntax, "failed normalization is atomic");
}

#[test]
fn all_css_wide_and_pending_reentry_preserve_members_and_occurrence() {
    let all_source = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source).unwrap()
    else {
        panic!("symbolic all reset")
    };
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
            if name == FOUR_SIDE {
                assert_eq!(
                    expand_declaration(&source).unwrap_err().kind(),
                    &unresolved_kind()
                );
            } else {
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    expand_declaration(&source).unwrap()
                else {
                    panic!("{name} has symbolic globals")
                };
                let expected = if PAIRS.contains(&name) { 2 } else { 1 };
                assert_eq!(values.items().len(), expected);
                for item in values.items() {
                    assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                }
            }
        }
        for pending in ["var(--inset)", "env(--inset)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name} retains pending substitution")
            };
            assert!(handle.source().same_occurrence(&source));
            assert!(matches!(
                handle
                    .reenter(parse_component_values("1fr").unwrap())
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
            let replacement_text = if name == "position" { "fixed" } else { "auto" };
            let replacement = parse_component_values(replacement_text).unwrap();
            if name == FOUR_SIDE {
                assert_eq!(
                    handle.reenter(replacement).unwrap_err().kind(),
                    &unresolved_kind()
                );
            } else {
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("{name} reenters to terminal contributions")
                };
                let expected_names: Vec<&str> = if name == "inset-block" {
                    vec!["inset-block-start", "inset-block-end"]
                } else if name == "inset-inline" {
                    vec!["inset-inline-start", "inset-inline-end"]
                } else {
                    vec![name]
                };
                assert_eq!(values.items().len(), expected_names.len());
                for (item, member) in values.items().iter().zip(expected_names) {
                    assert_eq!(item.property(), grammar(member).target_property());
                    assert_eq!(
                        item.ordinary_value(),
                        Some(&one_value(&declaration(member, replacement_text)))
                    );
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
            }
        }
    }
}

#[test]
fn normalization_preserves_authored_logical_pairs_and_pending_four_side_order() {
    let report = parse_sheet(
        ".a{position:relative;top:-1px;inset-block:2% auto;inset-inline-end:3px;inset:var(--inset);right:auto}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 6);
    for (index, name) in [
        "position",
        "top",
        "inset-block",
        "inset-inline-end",
        "inset",
        "right",
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations[index].order(), index);
        assert_eq!(
            declarations[index].source().known().unwrap().property(),
            grammar(name).target_property()
        );
        if name == FOUR_SIDE {
            assert!(matches!(
                declarations[index].expansion(),
                CssExpansion::Pending(_)
            ));
        } else {
            assert!(matches!(
                declarations[index].expansion(),
                CssExpansion::Contributions(CssContributions::Longhands(_))
            ));
        }
    }
}
