#![forbid(unsafe_code)]

//! Authored border widths from Backgrounds 3 CRD (2024-03-11) §§3.3–3.4 and
//! Logical 1 WD (2025-12-04) §§4.5.1, 4.7. Logical 1 issue 3030 leaves the
//! complete four-side shorthand reset footprint unsettled.

use surgeist_css::*;

const PHYSICAL: [&str; 4] = [
    "border-top-width",
    "border-right-width",
    "border-bottom-width",
    "border-left-width",
];
const LOGICAL: [&str; 4] = [
    "border-block-start-width",
    "border-block-end-width",
    "border-inline-start-width",
    "border-inline-end-width",
];
const PAIRS: [&str; 2] = ["border-block-width", "border-inline-width"];
const TRIPLES: [&str; 5] = [
    "border",
    "border-top",
    "border-right",
    "border-bottom",
    "border-left",
];
const FOUR_SIDE: &str = "border-width";

fn names() -> impl Iterator<Item = &'static str> {
    PHYSICAL
        .into_iter()
        .chain(LOGICAL)
        .chain(PAIRS)
        .chain([FOUR_SIDE])
        .chain(TRIPLES)
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
    item.ordinary_value().unwrap().clone()
}

fn unresolved_kind() -> CssExpansionErrorKind {
    CssExpansionErrorKind::UnresolvedStandard {
        property: grammar(FOUR_SIDE).target_property(),
        reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
    }
}

fn expect_rejection<T, E>(result: Result<T, E>) -> E {
    match result {
        Err(error) => error,
        Ok(_) => panic!("expected typed rejection"),
    }
}

#[test]
fn selected_names_have_distinct_stable_ids_and_dated_property_provenance() {
    let mut seen = Vec::new();
    for name in names() {
        let grammar = grammar(name);
        let property = grammar.target_property();
        assert_eq!(grammar.name(), name);
        assert_eq!(property.canonical_name(), name);
        let logical = LOGICAL.contains(&name) || PAIRS.contains(&name);
        assert_eq!(
            grammar.feature_id().as_str(),
            format!(
                "{}.property.{name}",
                if logical { "official" } else { "baseline" }
            )
        );
        assert!(!seen.contains(&property), "{name} aliases an earlier name");
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
    assert_eq!(seen.len(), 16);
}

#[test]
fn eight_width_longhands_accept_exact_nonnegative_lengths_and_deferred_pure_math() {
    for name in PHYSICAL.into_iter().chain(LOGICAL) {
        for value in [
            "thin",
            "medium",
            "thick",
            "0",
            "-0",
            "-0px",
            "1px",
            "1e100px",
            "1e-100px",
            "calc(1px + 2em)",
            "calc((-1px + 2px) * 3)",
            "calc(10% / 10% * 1px)",
        ] {
            accepted(name, value);
        }
        for value in [
            "-1px",
            "-1e-100px",
            "1%",
            "auto",
            "none",
            "1",
            "1fr",
            "calc(1px + 2%)",
            "calc(1 + 2)",
            "thin thick",
        ] {
            invalid(name, value);
        }
        let property = grammar(name).target_property();
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a width longhand")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        let CssInitialValueRef::Value(initial_value) = initial.view() else {
            panic!("{name} has a fixed medium initial")
        };
        assert_eq!(initial_value, &one_value(name, "medium"));
    }
}

#[test]
fn logical_width_pairs_expand_start_then_end_and_repeat_omitted_end() {
    for pair in PAIRS {
        let stem = pair.strip_suffix("-width").unwrap();
        let members = [format!("{stem}-start-width"), format!("{stem}-end-width")];
        let CssPropertyKindRef::Shorthand(shorthand) = grammar(pair).metadata().unwrap().kind()
        else {
            panic!("{pair} is a logical pair")
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
            ("thin", "thin", "thin"),
            ("1e100px thick", "1e100px", "thick"),
            ("calc((-1px + 2px) * 3) 0", "calc((-1px + 2px) * 3)", "0"),
        ] {
            let source = declaration(pair, authored);
            checked(pair, authored);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("{pair} expands to logical longhands")
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
        for value in ["", "logical 1px", "1px 2px 3px", "1%", "-1e-100px"] {
            invalid(pair, value);
        }
    }
}

#[test]
fn four_side_width_accepts_both_roles_but_complete_expansion_is_undefined() {
    for value in [
        "thin",
        "thin medium",
        "thin medium thick",
        "thin medium thick 1e100px",
        "logical thin",
        "logical thin medium",
        "logical thin medium thick",
        "logical thin medium thick 1e100px",
    ] {
        accepted(FOUR_SIDE, value);
        assert_eq!(
            expect_rejection(expand_declaration(&declaration(FOUR_SIDE, value))).kind(),
            &unresolved_kind()
        );
    }
    for value in [
        "",
        "logical",
        "logical logical thin",
        "thin logical",
        "logical 1%",
        "thin medium thick 1px 2px",
        "logical thin medium thick 1px 2px",
    ] {
        invalid(FOUR_SIDE, value);
    }
    let grammar = grammar(FOUR_SIDE);
    assert_eq!(
        expect_rejection(grammar.metadata()),
        CssPropertyMetadataError::UnresolvedStandard {
            grammar,
            reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
        }
    );
    let report = parse_sheet(".a{color:red;border-width:logical thin medium;color:blue}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let original = report.syntax().clone();
    let error = expect_rejection(normalize_sheet(report.syntax()));
    assert!(
        matches!(error.kind(), CssNormalizationErrorKind::UnsupportedDeclaration(expansion)
        if expansion.kind() == &unresolved_kind())
    );
    assert_eq!(error.declaration_order(), Some(1));
    assert_eq!(report.syntax(), &original, "failed normalization is atomic");
}

#[test]
fn physical_border_triples_share_exact_widths_and_intrinsic_defaults() {
    for name in TRIPLES {
        for value in ["solid", "1e100px", "1e-100px solid red", "red solid 0"] {
            accepted(name, value);
        }
        for value in [
            "-1e-100px solid",
            "1% solid",
            "thin thick solid",
            "solid dashed",
            "red blue",
        ] {
            invalid(name, value);
        }
        let source = declaration(name, "solid");
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("{name} expands to border facets")
        };
        let sides: &[&str] = if name == "border" {
            &["top", "right", "bottom", "left"]
        } else {
            &[name.strip_prefix("border-").unwrap()]
        };
        let mut expected = Vec::new();
        for facet in ["width", "style", "color"] {
            for side in sides {
                let member = format!("border-{side}-{facet}");
                expected.push((
                    member,
                    match facet {
                        "width" => "medium",
                        "style" => "solid",
                        _ => "currentcolor",
                    },
                ));
            }
        }
        for (item, (member, value)) in values.items().iter().zip(&expected) {
            assert_eq!(item.property(), grammar(member).target_property());
            assert_eq!(item.ordinary_value(), Some(&one_value(member, value)));
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
        }
        let CssPropertyKindRef::Shorthand(metadata) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a shorthand")
        };
        assert_eq!(metadata.settable_members().len(), expected.len());
        let reset_names = metadata
            .reset_only_members()
            .iter()
            .map(|member| member.known_property().canonical_name())
            .collect::<Vec<_>>();
        if name == "border" {
            assert_eq!(
                reset_names,
                [
                    "border-image-source",
                    "border-image-slice",
                    "border-image-width",
                    "border-image-outset",
                    "border-image-repeat",
                ]
            );
            assert_eq!(values.items().len(), 17);
            for (item, (member, expected)) in values.items()[12..].iter().zip([
                ("border-image-source", "none"),
                ("border-image-slice", "100%"),
                ("border-image-width", "1"),
                ("border-image-outset", "0"),
                ("border-image-repeat", "stretch"),
            ]) {
                assert_eq!(item.property(), grammar(member).target_property());
                assert_eq!(item.ordinary_value(), Some(&one_value(member, expected)));
            }
        } else {
            assert!(reset_names.is_empty());
            assert_eq!(values.items().len(), 3);
        }
        let exact = declaration(name, "1e100px solid");
        let CssExpansion::Contributions(CssContributions::Longhands(exact_values)) =
            expand_declaration(&exact).unwrap()
        else {
            panic!("{name} retains exact width contributions")
        };
        for (item, side) in exact_values.items().iter().zip(sides) {
            let member = format!("border-{side}-width");
            assert_eq!(item.property(), grammar(&member).target_property());
            assert_eq!(item.ordinary_value(), Some(&one_value(&member, "1e100px")));
        }
    }
}

#[test]
fn css_wide_all_and_pending_reentry_preserve_original_grammar_and_occurrence() {
    let all_source = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source).unwrap()
    else {
        panic!("all is a symbolic universal reset")
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
                    expect_rejection(expand_declaration(&source)).kind(),
                    &unresolved_kind()
                );
            } else {
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    expand_declaration(&source).unwrap()
                else {
                    panic!("{name} has symbolic global contributions")
                };
                let count = if PAIRS.contains(&name) {
                    2
                } else if name == "border" {
                    17
                } else if TRIPLES.contains(&name) {
                    3
                } else {
                    1
                };
                assert_eq!(values.items().len(), count);
                for item in values.items() {
                    assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                }
            }
        }
        for pending in ["var(--width)", "env(--width)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name} retains pending substitution")
            };
            assert!(handle.source().same_occurrence(&source));
            assert!(matches!(
                expect_rejection(handle.reenter(parse_component_values("1%").unwrap())).kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
            assert_eq!(
                expect_rejection(handle.reenter(parse_component_values("var(--again)").unwrap()))
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            if PAIRS.contains(&name) {
                assert!(matches!(
                    expect_rejection(
                        handle.reenter(parse_component_values("logical 1px").unwrap()),
                    )
                    .kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ));
            }
            let replacement_text = if TRIPLES.contains(&name) {
                "1px solid"
            } else {
                "1px"
            };
            let replacement = parse_component_values(replacement_text).unwrap();
            if name == FOUR_SIDE {
                assert_eq!(
                    expect_rejection(handle.reenter(replacement)).kind(),
                    &unresolved_kind()
                );
                assert_eq!(
                    expect_rejection(
                        handle.reenter(parse_component_values("logical thin medium").unwrap()),
                    )
                    .kind(),
                    &unresolved_kind()
                );
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
fn normalization_keeps_ordered_logical_names_and_pending_four_side_values() {
    let report = parse_sheet(
        ".a{border-top-width:1px;border-block-width:thin thick;border-inline-start-width:2px;border-width:var(--edge);border-left:solid}",
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
            "border-top-width",
            "border-block-width",
            "border-inline-start-width",
            "border-width",
            "border-left",
        ]
        .map(|name| grammar(name).target_property())
    );
    assert!(normalized.items().iter().any(|item| matches!(item,
        CssNormalizedItem::Declaration(value)
            if value.source().known().unwrap().property() == grammar(FOUR_SIDE).target_property()
                && matches!(value.expansion(), CssExpansion::Pending(_))
    )));
}
