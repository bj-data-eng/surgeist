#![forbid(unsafe_code)]

//! Backgrounds 3 CRD (2024-03-11) §3.1 and Logical 1 WD (2025-12-04)
//! §§4.5.3, 4.7. Logical 1 issue 3030 leaves complete four-side reset
//! membership unsettled, while its authored color grammar and roles are defined.

use surgeist_css::*;

const PHYSICAL: [&str; 4] = [
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
];
const LOGICAL: [&str; 4] = [
    "border-block-start-color",
    "border-block-end-color",
    "border-inline-start-color",
    "border-inline-end-color",
];
const PAIRS: [&str; 2] = ["border-block-color", "border-inline-color"];
const FOUR_SIDE: &str = "border-color";
const TRIPLES: [&str; 5] = [
    "border",
    "border-top",
    "border-right",
    "border-bottom",
    "border-left",
];

fn names() -> impl Iterator<Item = &'static str> {
    PHYSICAL
        .into_iter()
        .chain(LOGICAL)
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

fn one_value(name: &str, value: &str) -> CssLonghandValue {
    let source = declaration(name, value);
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("{name} has a terminal contribution")
    };
    let [item] = values.items() else {
        panic!("{name} has one contribution")
    };
    assert_eq!(item.property(), grammar(name).target_property());
    assert!(item.source().same_occurrence(&source));
    item.ordinary_value().unwrap().clone()
}

fn unresolved_kind() -> CssExpansionErrorKind {
    CssExpansionErrorKind::UnresolvedStandard {
        property: grammar(FOUR_SIDE).target_property(),
        reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
    }
}

fn rejected<T, E>(result: Result<T, E>) -> E {
    match result {
        Err(error) => error,
        Ok(_) => panic!("expected typed rejection"),
    }
}

#[test]
fn eleven_color_names_have_distinct_ids_and_selected_dated_provenance() {
    let mut seen = Vec::new();
    for name in names() {
        let grammar = grammar(name);
        let property = grammar.target_property();
        let logical = LOGICAL.contains(&name) || PAIRS.contains(&name);
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
    assert_eq!(seen.len(), 11);
    let extension = feature_metadata("official.value.border-color-logical-values")
        .expect("selected logical marker value provenance");
    assert_eq!(extension.kind(), CssFeatureKind::Value);
    assert_eq!(extension.status(), CssSupportStatus::Complete);
    assert_eq!(extension.source().id().as_str(), "I-LOGICAL1-20251204");
    assert_eq!(extension.production(), "#logical-shorthand-keyword");
}

#[test]
fn eight_color_longhands_reuse_complete_authored_colors_and_symbolic_initial() {
    for name in PHYSICAL.into_iter().chain(LOGICAL) {
        for value in [
            "currentcolor",
            "red",
            "#12abef",
            "rgb(1 2 3 / none)",
            "lab(calc(50% + 10%) 20 -30 / 120%)",
        ] {
            accepted(name, value);
        }
        for value in ["", "none", "1px", "red blue", "logical red", "red, blue"] {
            invalid(name, value);
        }
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a color longhand")
        };
        assert!(!longhand.inherited_by_default());
        let initial_value = longhand.initial_value();
        let CssInitialValueRef::Value(initial) = initial_value.view() else {
            panic!("{name} has a symbolic currentcolor initial")
        };
        assert_eq!(initial, &one_value(name, "currentcolor"));
    }
}

#[test]
fn border_color_i01_is_single_compatible_physical_only() {
    for (authored, expected) in [
        ("black", Some(CssColor::BLACK)),
        ("currentcolor", Some(CssColor::CurrentColor)),
        ("black black", None),
        ("logical black", None),
        ("logical currentcolor", None),
    ] {
        let source = declaration(FOUR_SIDE, authored);
        let CssKnownPropertyValueRef::BorderColor(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed border-color for {authored}")
        };
        assert_eq!(value.i01_subset(), expected.as_ref(), "{authored}");
    }
}

#[test]
fn logical_color_pairs_assign_exact_start_and_end_with_omission() {
    for pair in PAIRS {
        let stem = pair.strip_suffix("-color").unwrap();
        let members = [format!("{stem}-start-color"), format!("{stem}-end-color")];
        let CssPropertyKindRef::Shorthand(metadata) = grammar(pair).metadata().unwrap().kind()
        else {
            panic!("{pair} is an axis pair")
        };
        assert!(!metadata.is_legacy());
        assert!(metadata.reset_only_members().is_empty());
        assert_eq!(
            metadata
                .settable_members()
                .iter()
                .map(|member| member.known_property().canonical_name())
                .collect::<Vec<_>>(),
            members
        );
        for (authored, start, end) in [
            ("currentcolor", "currentcolor", "currentcolor"),
            ("red #12abef", "red", "#12abef"),
            (
                "rgb(1 2 3 / none) lab(50% 20 -30)",
                "rgb(1 2 3 / none)",
                "lab(50% 20 -30)",
            ),
        ] {
            let source = declaration(pair, authored);
            checked(pair, authored);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("{pair} expands to two logical colors")
            };
            let [first, second] = values.items() else {
                panic!("start then end")
            };
            for (item, member, expected) in [
                (first, members[0].as_str(), start),
                (second, members[1].as_str(), end),
            ] {
                assert_eq!(item.property(), grammar(member).target_property());
                assert_eq!(item.ordinary_value(), Some(&one_value(member, expected)));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert!(item.replacement_components().is_none());
            }
        }
        for value in ["", "logical red", "red blue green", "none", "red, blue"] {
            invalid(pair, value);
        }
    }
}

#[test]
fn four_side_colors_accept_both_role_modes_and_report_unresolved_resets() {
    for value in [
        "red",
        "red #12abef",
        "red #12abef currentcolor",
        "red #12abef currentcolor rgb(1 2 3 / none)",
        "logical red",
        "logical red #12abef",
        "logical red #12abef currentcolor",
        "logical red #12abef currentcolor rgb(1 2 3 / none)",
    ] {
        accepted(FOUR_SIDE, value);
        assert_eq!(
            rejected(expand_declaration(&declaration(FOUR_SIDE, value))).kind(),
            &unresolved_kind()
        );
    }
    for value in [
        "",
        "logical",
        "logical logical red",
        "red logical",
        "logical none",
        "red blue green black white",
        "logical red blue green black white",
    ] {
        invalid(FOUR_SIDE, value);
    }
    let grammar = grammar(FOUR_SIDE);
    assert_eq!(
        rejected(grammar.metadata()),
        CssPropertyMetadataError::UnresolvedStandard {
            grammar,
            reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
        }
    );

    let source = ".a{color:red;border-color:logical currentcolor #12abef;color:blue}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(style)] = report.syntax().rules() else {
        panic!("one authored .a style rule")
    };
    let [_before, border, _after] = style.declarations().as_slice() else {
        panic!("border-color between two color declarations")
    };
    let original = report.syntax().clone();
    let error = rejected(normalize_sheet(report.syntax()));
    assert!(matches!(
        error.kind(),
        CssNormalizationErrorKind::UnsupportedDeclaration(expansion)
            if expansion.kind() == &unresolved_kind()
    ));
    assert_eq!(error.declaration_order(), Some(1));
    assert_eq!(
        error.position().unwrap().byte_offset().value(),
        source.find("border-color").unwrap()
    );
    assert!(error.declaration().unwrap().same_occurrence(border));
    let context = error.rule_context().expect("owning .a style rule");
    let CssRuleContextKindRef::Style(selectors) = context.kind() else {
        panic!("owning style context")
    };
    assert_eq!(
        selectors.selectors()[0].selector(),
        &CssSelector::Class("a".into())
    );
    assert_eq!(
        context.position().unwrap().byte_offset().value(),
        source.find(".a").unwrap()
    );
    assert_eq!(report.syntax(), &original, "failed normalization is atomic");
}

#[test]
fn physical_border_triples_preserve_color_and_currentcolor_default() {
    for name in TRIPLES {
        for value in ["solid", "1px #12abef", "rgb(1 2 3 / none)"] {
            accepted(name, value);
        }
        let side_names: &[&str] = if name == "border" {
            &["top", "right", "bottom", "left"]
        } else {
            &[name.strip_prefix("border-").unwrap()]
        };
        for (authored, expected) in [
            ("solid", "currentcolor"),
            ("1px #12abef", "#12abef"),
            ("rgb(1 2 3 / none)", "rgb(1 2 3 / none)"),
        ] {
            let source = declaration(name, authored);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("{name} expands to border facets")
            };
            let color_start = side_names.len() * 2;
            for (item, side) in values.items()[color_start..color_start + side_names.len()]
                .iter()
                .zip(side_names)
            {
                let member = format!("border-{side}-color");
                assert_eq!(item.property(), grammar(&member).target_property());
                assert_eq!(item.ordinary_value(), Some(&one_value(&member, expected)));
                assert!(item.source().same_occurrence(&source));
            }
            assert_eq!(values.items().len(), if name == "border" { 17 } else { 3 });
        }
    }
}

#[test]
fn globals_all_and_pending_reentry_keep_color_members_and_occurrence() {
    let all_source = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source).unwrap()
    else {
        panic!("all is a symbolic universal reset")
    };
    assert!(reset.source().same_occurrence(&all_source));
    for name in names() {
        assert!(!reset.excludes(CssPropertyNameRef::Known(grammar(name).target_property())));
        invalid(name, "initial red");
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
                    rejected(expand_declaration(&source)).kind(),
                    &unresolved_kind()
                );
            } else {
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    expand_declaration(&source).unwrap()
                else {
                    panic!("{name} has symbolic global contributions")
                };
                assert_eq!(
                    values.items().len(),
                    if PAIRS.contains(&name) { 2 } else { 1 }
                );
                for item in values.items() {
                    assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                }
            }
        }
        for pending in ["var(--border-color)", "env(--border-color)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name} retains pending substitution")
            };
            assert!(handle.source().same_occurrence(&source));
            assert_eq!(
                rejected(handle.reenter(parse_component_values("var(--again)").unwrap())).kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            assert!(matches!(
                rejected(handle.reenter(parse_component_values("none").unwrap())).kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
            if name == FOUR_SIDE {
                for replacement in ["red blue", "logical red #12abef"] {
                    assert_eq!(
                        rejected(handle.reenter(parse_component_values(replacement).unwrap()))
                            .kind(),
                        &unresolved_kind()
                    );
                }
            } else {
                let replacement = if PAIRS.contains(&name) {
                    "red #12abef"
                } else {
                    "red"
                };
                let components = parse_component_values(replacement).unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(components.clone()).unwrap()
                else {
                    panic!("{name} reenters to longhands")
                };
                let members: Vec<String> = if PAIRS.contains(&name) {
                    let stem = name.strip_suffix("-color").unwrap();
                    vec![format!("{stem}-start-color"), format!("{stem}-end-color")]
                } else {
                    vec![name.to_owned()]
                };
                assert_eq!(values.items().len(), members.len());
                for (index, (item, member)) in values.items().iter().zip(&members).enumerate() {
                    assert_eq!(item.property(), grammar(member).target_property());
                    assert_eq!(
                        item.ordinary_value(),
                        Some(&one_value(
                            member,
                            if index == 1 { "#12abef" } else { "red" }
                        ))
                    );
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    assert_eq!(item.replacement_components(), Some(&components));
                }
            }
        }
    }
}

#[test]
fn normalization_retains_authored_order_with_pending_four_side_color() {
    let report = parse_sheet(
        ".a{border-top-color:red;border-block-color:#12abef currentcolor;border-inline-start-color:rgb(1 2 3 / none);border-color:var(--c);border-left-color:blue}",
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
            "border-top-color",
            "border-block-color",
            "border-inline-start-color",
            "border-color",
            "border-left-color",
        ]
        .map(|name| grammar(name).target_property())
    );
    assert!(normalized.items().iter().any(|item| matches!(
        item,
        CssNormalizedItem::Declaration(value)
            if value.source().known().unwrap().property() == grammar(FOUR_SIDE).target_property()
                && matches!(value.expansion(), CssExpansion::Pending(_))
    )));
}
