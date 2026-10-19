#![forbid(unsafe_code)]

//! Backgrounds 3 CRD (2024-03-11) §§3.2, 3.4 and Logical 1 WD (2025-12-04)
//! §§4.5.2, 4.7. Logical 1 issue 3030 leaves complete four-side reset
//! membership unsettled, while the authored grammar remains defined.

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

const PHYSICAL: [&str; 4] = [
    "border-top-style",
    "border-right-style",
    "border-bottom-style",
    "border-left-style",
];
const LOGICAL: [&str; 4] = [
    "border-block-start-style",
    "border-block-end-style",
    "border-inline-start-style",
    "border-inline-end-style",
];
const PAIRS: [&str; 2] = ["border-block-style", "border-inline-style"];
const FOUR_SIDE: &str = "border-style";
const KEYWORDS: [&str; 10] = [
    "none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset", "outset",
];
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

fn expect_rejection<T, E>(result: Result<T, E>) -> E {
    match result {
        Err(error) => error,
        Ok(_) => panic!("expected typed rejection"),
    }
}

#[test]
fn eleven_style_names_keep_stable_ids_and_selected_spec_provenance() {
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
    let extension = feature_metadata("official.value.border-style-logical-values")
        .expect("selected logical marker value provenance");
    assert_eq!(extension.kind(), CssFeatureKind::Value);
    assert_eq!(extension.status(), CssSupportStatus::Complete);
    assert_eq!(extension.source().id().as_str(), "I-LOGICAL1-20251204");
    assert_eq!(
        extension.source().url(),
        Some("https://www.w3.org/TR/2025/WD-css-logical-1-20251204/")
    );
    assert_eq!(extension.production(), "#logical-shorthand-keyword");
}

#[test]
fn eight_style_longhands_accept_exact_ten_keywords_and_initial_none() {
    for name in PHYSICAL.into_iter().chain(LOGICAL) {
        for keyword in KEYWORDS {
            accepted(name, keyword);
        }
        for value in [
            "",
            "auto",
            "center",
            "1px",
            "red",
            "solid dashed",
            "logical solid",
        ] {
            invalid(name, value);
        }
        let property = grammar(name).target_property();
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a style longhand")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        let CssInitialValueRef::Value(initial_value) = initial.view() else {
            panic!("{name} has fixed initial none")
        };
        assert_eq!(initial_value, &one_value(name, "none"));
    }
}

#[test]
fn logical_style_pairs_assign_start_then_end_and_repeat_omitted_end() {
    for pair in PAIRS {
        let stem = pair.strip_suffix("-style").unwrap();
        let members = [format!("{stem}-start-style"), format!("{stem}-end-style")];
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
            ("solid", "solid", "solid"),
            ("dashed dotted", "dashed", "dotted"),
            ("none hidden", "none", "hidden"),
        ] {
            let source = declaration(pair, authored);
            checked(pair, authored);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("{pair} expands to two logical styles")
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
        for value in ["", "logical solid", "solid dashed dotted", "1px", "auto"] {
            invalid(pair, value);
        }
    }
}

#[test]
fn border_style_accepts_physical_or_prefixed_logical_one_to_four_values_and_expands_selected_sides()
{
    for value in [
        "solid",
        "solid dashed",
        "solid dashed dotted",
        "solid dashed dotted double",
        "logical solid",
        "logical solid dashed",
        "logical solid dashed dotted",
        "logical solid dashed dotted double",
    ] {
        accepted(FOUR_SIDE, value);
        assert_four_expansion(expand_declaration(&declaration(FOUR_SIDE, value)));
    }
    for value in [
        "",
        "logical",
        "logical logical solid",
        "solid logical",
        "logical 1px",
        "solid dashed dotted double groove",
        "logical solid dashed dotted double groove",
    ] {
        invalid(FOUR_SIDE, value);
    }
    let grammar = grammar(FOUR_SIDE);
    assert!(matches!(
        grammar.metadata().unwrap().kind(),
        CssPropertyKindRef::FourSideShorthand(_)
    ));
    let report = parse_sheet(&format!(
        ".a{{color:red;{FOUR_SIDE}:logical solid dotted;color:blue}}"
    ));
    assert!(report.is_clean());
    let before = report.syntax().clone();
    assert!(normalize_sheet(report.syntax()).is_ok());
    assert_eq!(report.syntax(), &before);
}

#[test]
fn physical_border_triples_keep_style_facets_and_omitted_none_defaults() {
    for name in TRIPLES {
        accepted(name, "solid");
        accepted(name, "1px red");
        let side_names: &[&str] = if name == "border" {
            &["top", "right", "bottom", "left"]
        } else {
            &[name.strip_prefix("border-").unwrap()]
        };
        for (authored, expected) in [("solid", "solid"), ("1px red", "none")] {
            let source = declaration(name, authored);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("{name} expands to border facets")
            };
            let style_start = side_names.len();
            for (item, side) in values.items()[style_start..style_start + side_names.len()]
                .iter()
                .zip(side_names)
            {
                let member = format!("border-{side}-style");
                assert_eq!(item.property(), grammar(&member).target_property());
                assert_eq!(item.ordinary_value(), Some(&one_value(&member, expected)));
                assert!(item.source().same_occurrence(&source));
            }
            assert_eq!(values.items().len(), if name == "border" { 17 } else { 3 });
        }
        let CssPropertyKindRef::Shorthand(metadata) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a border shorthand")
        };
        let resets = metadata
            .reset_only_members()
            .iter()
            .map(|member| member.known_property().canonical_name())
            .collect::<Vec<_>>();
        if name == "border" {
            assert_eq!(
                resets,
                [
                    "border-image-source",
                    "border-image-slice",
                    "border-image-width",
                    "border-image-outset",
                    "border-image-repeat"
                ]
            );
        } else {
            assert!(resets.is_empty());
        }
    }
}

#[test]
fn css_wide_all_and_pending_reentry_keep_style_members_and_original_occurrence() {
    let all_source = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source).unwrap()
    else {
        panic!("all is a symbolic universal reset")
    };
    assert!(reset.source().same_occurrence(&all_source));
    for name in names() {
        assert!(!reset.excludes(CssPropertyNameRef::Known(grammar(name).target_property())));
        invalid(name, "initial solid");
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
                assert_four_expansion(expand_declaration(&source));
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
        for pending in ["var(--style)", "env(--style)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name} retains pending substitution")
            };
            assert!(handle.source().same_occurrence(&source));
            for invalid_value in ["1px", "var(--again)"] {
                let error = expect_rejection(
                    handle.reenter(parse_component_values(invalid_value).unwrap()),
                );
                if invalid_value.starts_with("var(") {
                    assert_eq!(error.kind(), &CssExpansionErrorKind::ResidualSubstitution);
                } else {
                    assert!(matches!(
                        error.kind(),
                        CssExpansionErrorKind::InvalidReplacement(_)
                    ));
                }
            }
            if PHYSICAL.contains(&name) || LOGICAL.contains(&name) {
                assert!(matches!(
                    expect_rejection(
                        handle.reenter(parse_component_values("solid dashed").unwrap())
                    )
                    .kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ));
            }
            let replacement = parse_component_values("solid dashed").unwrap();
            if name == FOUR_SIDE {
                assert_four_contributions(handle.reenter(replacement));
                assert_four_contributions(
                    handle.reenter(parse_component_values("logical solid dotted").unwrap()),
                );
            } else {
                let replacement = if PAIRS.contains(&name) {
                    replacement
                } else {
                    parse_component_values("solid").unwrap()
                };
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("{name} reenters to terminal contributions")
                };
                let members: Vec<String> = if PAIRS.contains(&name) {
                    let stem = name.strip_suffix("-style").unwrap();
                    vec![format!("{stem}-start-style"), format!("{stem}-end-style")]
                } else {
                    vec![name.to_owned()]
                };
                assert_eq!(values.items().len(), members.len());
                for (index, (item, member)) in values.items().iter().zip(&members).enumerate() {
                    assert_eq!(item.property(), grammar(member).target_property());
                    let expected = if index == 1 { "dashed" } else { "solid" };
                    assert_eq!(item.ordinary_value(), Some(&one_value(member, expected)));
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
            }
        }
    }
}

#[test]
fn normalization_keeps_ordered_logical_style_names_and_pending_four_side() {
    let report = parse_sheet(
        ".a{border-top-style:dotted;border-block-style:solid dashed;border-inline-start-style:groove;border-style:var(--s);border-left-style:ridge}",
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
            "border-top-style",
            "border-block-style",
            "border-inline-start-style",
            "border-style",
            "border-left-style",
        ]
        .map(|name| grammar(name).target_property())
    );
    assert!(normalized.items().iter().any(|item| matches!(item,
        CssNormalizedItem::Declaration(value)
            if value.source().known().unwrap().property() == grammar(FOUR_SIDE).target_property()
                && matches!(value.expansion(), CssExpansion::Pending(_))
    )));
}
