#![forbid(unsafe_code)]

//! Box 3 REC (2024-04-11) §§3–4 and Logical 1 WD (2025-12-04)
//! §§4.2, 4.4, 4.7. Logical 1 issue 3030 leaves the complete reset
//! footprint of the four-side shorthands unsettled.

use surgeist_css::*;

fn assert_four_expansion(result: Result<CssExpansion, CssExpansionError>) {
    let CssExpansion::Contributions(values) = result.unwrap() else {
        panic!("completed four-side expansion")
    };
    assert_four_contributions(Ok(values));
}
fn assert_four_contributions(result: Result<CssContributions, CssExpansionError>) {
    let CssContributions::Longhands(values) = result.unwrap() else {
        panic!("four selected sides")
    };
    assert_eq!(values.items().len(), 4);
}

const SIDES: [&str; 8] = [
    "top",
    "right",
    "bottom",
    "left",
    "block-start",
    "block-end",
    "inline-start",
    "inline-end",
];
const FAMILIES: [&str; 2] = ["margin", "padding"];

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
    let property = grammar(name).target_property();
    for source in [declaration(name, value), checked(name, value)] {
        assert_eq!(source.known().unwrap().property(), property);
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
    assert_eq!(report.syntax().len(), 2, "{source}");
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

fn programmatic_zero(name: &str) -> CssLonghandValue {
    let components =
        CssComponentValues::try_new(vec![CssComponentValue::try_number("0").unwrap()]).unwrap();
    let source =
        parse_property_value_for_grammar(grammar(name), components, CssImportance::Normal).unwrap();
    one_value(&source)
}

#[test]
fn all_twenty_two_names_have_distinct_known_identities_and_spec_support() {
    let mut names = Vec::new();
    for family in FAMILIES {
        names.push(family.to_string());
        for side in SIDES {
            names.push(format!("{family}-{side}"));
        }
        names.push(format!("{family}-block"));
        names.push(format!("{family}-inline"));
    }
    assert_eq!(names.len(), 22);
    let mut properties = Vec::new();
    for name in &names {
        let grammar = grammar(name);
        assert_eq!(grammar.name(), name);
        assert_eq!(grammar.target_property().canonical_name(), name);
        let logical = name.contains("block") || name.contains("inline");
        let expected_id = if logical {
            format!("official.property.{name}")
        } else {
            format!("baseline.property.{name}")
        };
        assert_eq!(grammar.feature_id().as_str(), expected_id);
        let support = property_support_metadata(name).expect("selected support metadata");
        assert_eq!(support.property(), grammar.target_property());
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        let source_url = if logical {
            "https://www.w3.org/TR/2025/WD-css-logical-1-20251204/"
        } else {
            "https://www.w3.org/TR/2024/REC-css-box-3-20240411/"
        };
        assert_eq!(support.feature().source().url(), Some(source_url), "{name}");
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
        assert!(
            !properties.contains(&grammar.target_property()),
            "{name} aliases an earlier property"
        );
        properties.push(grammar.target_property());
        accepted(name, "1px");
    }
    assert_eq!(properties.len(), 22);
}

#[test]
fn margin_accepts_signed_length_percentages_and_auto_but_padding_is_nonnegative() {
    for family in FAMILIES {
        let mut names = vec![family.to_string()];
        names.extend(SIDES.map(|side| format!("{family}-{side}")));
        names.extend([format!("{family}-block"), format!("{family}-inline")]);
        for name in names {
            for value in [
                "0",
                "1px",
                "1%",
                "1e999px",
                "1e999%",
                "1e-999px",
                "1e-999%",
                "calc(1px + 2%)",
            ] {
                accepted(&name, value);
            }
            if family == "margin" {
                for value in ["auto", "-1px", "-1%", "-1e999px", "-1e-999px", "-1e-999%"] {
                    accepted(&name, value);
                }
            } else {
                for value in ["auto", "-1px", "-1%", "-1e-999px", "-1e-999%"] {
                    invalid(&name, value);
                }
            }
            for value in ["1", "1fr", "min-content", "fit-content(1px)", "calc(1 + 2)"] {
                invalid(&name, value);
            }
        }
    }
}

#[test]
fn four_side_shorthands_accept_physical_or_prefixed_logical_one_to_four_values() {
    for family in FAMILIES {
        for values in ["1px", "1px 2%", "1px 2% 3px", "1px 2% 3px 4px"] {
            accepted(family, values);
            accepted(family, &format!("logical {values}"));
        }
        for values in [
            "",
            "logical",
            "logical logical 1px",
            "1px logical",
            "1px 2px 3px 4px 5px",
            "logical 1px 2px 3px 4px 5px",
        ] {
            invalid(family, values);
        }
    }
    accepted("margin", "logical auto -1px 2% -3px");
    invalid("padding", "logical 1px -1e-999px");
    for name in [
        "margin-block",
        "margin-inline",
        "padding-block",
        "padding-inline",
    ] {
        accepted(name, "1px");
        accepted(name, "1px 2px");
        invalid(name, "logical 1px");
        invalid(name, "1px 2px 3px");
    }
}

#[test]
fn sixteen_longhands_have_fixed_zero_initials_and_do_not_inherit() {
    for family in FAMILIES {
        for side in SIDES {
            let name = format!("{family}-{side}");
            let property = grammar(&name).target_property();
            let CssPropertyKindRef::Longhand(longhand) = grammar(&name).metadata().unwrap().kind()
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
            assert_eq!(value, &programmatic_zero(&name), "{name}");
        }
    }
}

#[test]
fn logical_axis_pairs_expand_start_then_end_without_physical_mapping() {
    for family in FAMILIES {
        for axis in ["block", "inline"] {
            let name = format!("{family}-{axis}");
            let members = [format!("{name}-start"), format!("{name}-end")];
            let CssPropertyKindRef::Shorthand(shorthand) =
                grammar(&name).metadata().unwrap().kind()
            else {
                panic!("{name} is an axis shorthand")
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
            for (value, start, end) in [("1px", "1px", "1px"), ("1px 2%", "1px", "2%")] {
                let source = declaration(&name, value);
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    expand_declaration(&source).unwrap()
                else {
                    panic!("{name} expands to two logical longhands")
                };
                assert_eq!(values.items().len(), 2);
                for (item, (member, expected)) in
                    values.items().iter().zip(members.iter().zip([start, end]))
                {
                    assert_eq!(item.property().canonical_name(), member);
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
    }
}

#[test]
fn css_wide_values_and_pending_reentry_participate_at_every_property() {
    let all_source = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source).unwrap()
    else {
        panic!("all has a symbolic universal reset")
    };
    assert!(reset.source().same_occurrence(&all_source));
    assert_eq!(reset.source().importance(), CssImportance::Important);
    for name in ["direction", "unicode-bidi"] {
        assert!(reset.excludes(CssPropertyNameRef::Known(grammar(name).target_property())));
    }
    let custom = CssCustomPropertyName::try_new("--space").unwrap();
    assert!(reset.excludes(CssPropertyNameRef::Custom(&custom)));

    for family in FAMILIES {
        let mut names = vec![family.to_string()];
        names.extend(SIDES.map(|side| format!("{family}-{side}")));
        names.extend([format!("{family}-block"), format!("{family}-inline")]);
        for name in names {
            assert!(
                !reset.excludes(CssPropertyNameRef::Known(grammar(&name).target_property())),
                "all includes {name}"
            );
            let four_side = name == family;
            for (text, keyword) in [
                ("initial", CssGlobalKeyword::Initial),
                ("inherit", CssGlobalKeyword::Inherit),
                ("unset", CssGlobalKeyword::Unset),
                ("revert", CssGlobalKeyword::Revert),
                ("revert-layer", CssGlobalKeyword::RevertLayer),
            ] {
                let source = declaration(&name, text);
                checked(&name, text);
                if four_side {
                    assert_four_expansion(expand_declaration(&source));
                } else {
                    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                        expand_declaration(&source).unwrap()
                    else {
                        panic!("{name} has symbolic global contributions")
                    };
                    for item in values.items() {
                        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                        assert!(item.source().same_occurrence(&source));
                        assert_eq!(item.source().importance(), CssImportance::Important);
                    }
                }
            }
            invalid(&name, "initial 1px");
            let source = declaration(&name, "var(--space)");
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
            let replacement = parse_component_values("1px").unwrap();
            if four_side {
                assert_four_contributions(handle.reenter(replacement));
            } else {
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("{name} reenters to longhands")
                };
                for item in values.items() {
                    assert!(item.ordinary_value().is_some());
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
            }
        }
    }
}

#[test]
fn four_side_shorthands_parse_cleanly_but_expand_selected_sides() {
    for family in FAMILIES {
        let grammar = grammar(family);
        assert!(matches!(
            grammar.metadata().unwrap().kind(),
            CssPropertyKindRef::FourSideShorthand(_)
        ));
        for value in ["1px", "1px 2px 3px 4px", "logical 1px 2px 3px 4px"] {
            let source = declaration(family, value);
            checked(family, value);
            assert_four_expansion(expand_declaration(&source));
        }
        let report = parse_sheet(&format!(
            ".a{{color:red;{family}:logical 1px 2px;color:blue}}"
        ));
        assert!(report.is_clean());
        let before = report.syntax().clone();
        assert!(normalize_sheet(report.syntax()).is_ok());
        assert_eq!(report.syntax(), &before);
    }
}

#[test]
fn normalization_preserves_authored_logical_pairs_and_pending_four_side_values() {
    let report = parse_sheet(
        ".a{margin-top:1px;margin-block:2px 3px;padding-inline:4px;margin-inline-end:auto;padding-left:5%}",
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
            "margin-top",
            "margin-block",
            "padding-inline",
            "margin-inline-end",
            "padding-left"
        ]
        .map(|name| grammar(name).target_property())
    );

    for family in FAMILIES {
        let report = parse_sheet(&format!(".a{{{family}:var(--space)}}"));
        assert!(report.is_clean());
        let normalized = normalize_sheet(report.syntax()).unwrap();
        assert!(normalized.items().iter().any(|item| matches!(item,
            CssNormalizedItem::Declaration(value) if matches!(value.expansion(), CssExpansion::Pending(_))
        )));
    }
}
