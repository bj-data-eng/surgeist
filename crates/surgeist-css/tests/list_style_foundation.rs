#![forbid(unsafe_code)]

//! Existing-API expectations from CSS Lists 3 (2020-11-17) §3.3–3.6:
//! https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#propdef-list-style
//! Counter-style names and `symbols()` follow Counter Styles 3 (2021-07-27);
//! generic custom identifiers follow CSS Values 4 (2024-03-12) §4.2.

use surgeist_css::*;

const TYPE: CssKnownProperty = CssKnownProperty::ListStyleType;
const POSITION: CssKnownProperty = CssKnownProperty::ListStylePosition;
const IMAGE: CssKnownProperty = CssKnownProperty::ListStyleImage;
const SHORTHAND: CssKnownProperty = CssKnownProperty::ListStyle;
const MEMBERS: [CssKnownProperty; 3] = [POSITION, IMAGE, TYPE];

fn declaration(property: CssKnownProperty, value: &str) -> CssDeclaration {
    let source = format!("{}:{value}!important", property.canonical_name());
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    assert_eq!(declaration.known().unwrap().property(), property);
    declaration.clone()
}

fn ordinary(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary list-style expansion")
    };
    values.items().to_vec()
}

fn rejected(property: CssKnownProperty, value: &str) {
    let source = format!("color:red;{}:{value};height:2px", property.canonical_name());
    let report = parse_style_attribute(&source);
    assert_eq!(
        report
            .syntax()
            .iter()
            .map(|declaration| declaration.known().unwrap().property())
            .collect::<Vec<_>>(),
        [CssKnownProperty::Color, CssKnownProperty::Height],
        "{source}"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one dropped declaration: {source}: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert!(validate_style_attribute(&source).is_err());
}

#[test]
fn four_names_have_selected_source_and_inherited_longhand_initials() {
    for (property, value, production) in [
        (TYPE, "disc", "#propdef-list-style-type"),
        (POSITION, "outside", "#propdef-list-style-position"),
        (IMAGE, "none", "#propdef-list-style-image"),
    ] {
        let feature = feature_metadata(property.grammar().feature_id().as_str()).unwrap();
        assert_eq!(feature.status(), CssSupportStatus::Complete);
        assert_eq!(feature.source().id().as_str(), "I-LISTS3");
        assert_eq!(feature.production(), production);
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("{} is a longhand", property.canonical_name())
        };
        assert!(metadata.inherited_by_default());
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("{} has an intrinsic initial", property.canonical_name())
        };
        assert_eq!(
            initial,
            ordinary(&declaration(property, value))[0]
                .ordinary_value()
                .unwrap()
        );
    }
    let feature = feature_metadata(SHORTHAND.grammar().feature_id().as_str()).unwrap();
    assert_eq!(feature.status(), CssSupportStatus::Complete);
    assert_eq!(feature.source().id().as_str(), "I-LISTS3");
    assert_eq!(feature.production(), "#propdef-list-style");
    let CssPropertyKindRef::Shorthand(metadata) = SHORTHAND.metadata().unwrap().kind() else {
        panic!("list-style is a shorthand")
    };
    assert_eq!(
        metadata
            .settable_members()
            .iter()
            .map(|member| member.known_property())
            .collect::<Vec<_>>(),
        MEMBERS
    );
    assert!(metadata.reset_only_members().is_empty());
}

#[test]
fn type_accepts_counter_style_strings_and_generic_custom_names() {
    for value in [
        "none",
        "disc",
        "JAPANESE-INFORMAL",
        "CustomMarker",
        "inside",
        "outside",
        r"\69 nside",
        r"\31 marker",
        "\"★\"",
        "symbols(\"*\")",
        "symbols(cyclic \"*\" url(\"#star\"))",
        "symbols(numeric \"0\" linear-gradient(red, blue))",
        "symbols(alphabetic \"a\" src(\"#bee\"))",
    ] {
        let source = declaration(TYPE, value);
        assert!(
            source.known().unwrap().property_value().is_some(),
            "{value}"
        );
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            value
        );
    }
}

#[test]
fn image_accepts_selected_images_and_position_keeps_its_two_keywords() {
    for value in [
        "none",
        "url(\"\")",
        "src(\"#marker\")",
        "linear-gradient(red, blue)",
        "radial-gradient(red, blue)",
        "repeating-linear-gradient(red, blue)",
        "repeating-radial-gradient(red, blue)",
    ] {
        let source = declaration(IMAGE, value);
        assert!(
            source.known().unwrap().property_value().is_some(),
            "{value}"
        );
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            value
        );
    }
    for value in ["inside", "outside", "INSIDE", r"\6f utside"] {
        assert!(
            declaration(POSITION, value)
                .known()
                .unwrap()
                .property_value()
                .is_some()
        );
    }
}

#[test]
fn shorthand_accepts_unordered_members_and_allocates_ambiguous_none() {
    for value in [
        "none",
        "none none",
        "inside none none",
        "none inside none",
        "none square",
        "none url(\"a\")",
        "inside url(\"a\") square",
        "square src(\"#mark\") outside",
        "symbols(cyclic \"*\") inside linear-gradient(red, blue)",
        "symbols(numeric \"0\" \"1\") repeating-radial-gradient(red, blue)",
        "inside outside",
        "outside inside",
        "inside inside",
    ] {
        let source = declaration(SHORTHAND, value);
        assert!(
            source.known().unwrap().property_value().is_some(),
            "{value}"
        );
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            value
        );
    }
}

#[test]
fn invalid_longhand_domains_and_shorthand_duplicates_drop_only_their_declaration() {
    for (property, value) in [
        (TYPE, "inherit disc"),
        (TYPE, "default"),
        (TYPE, r"\64 efault"),
        (TYPE, "symbols()"),
        (TYPE, "symbols(numeric \"0\")"),
        (TYPE, "symbols(cyclic none)"),
        (IMAGE, "red"),
        (IMAGE, "linear-gradient(red)"),
        (IMAGE, "none url(\"a\")"),
        (POSITION, "center"),
        (POSITION, "inside outside"),
        (SHORTHAND, ""),
        (SHORTHAND, "none none none"),
        (SHORTHAND, "inside outside square"),
        (SHORTHAND, "none none square"),
        (SHORTHAND, "none none url(\"a\")"),
        (SHORTHAND, "none square url(\"a\")"),
        (SHORTHAND, "inside outside inside"),
        (SHORTHAND, "linear-gradient(red)"),
    ] {
        rejected(property, value);
    }
}

#[test]
fn checked_construction_uses_the_same_grammar_and_retains_programmatic_origin() {
    for (property, keyword) in [
        (TYPE, "outside"),
        (POSITION, "inside"),
        (IMAGE, "none"),
        (SHORTHAND, "inside"),
    ] {
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_ident(keyword).unwrap()])
                .unwrap();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(checked.known().unwrap().property(), property);
        assert!(checked.known().unwrap().property_value().is_some());
        assert!(matches!(
            checked.value_components().items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
    }
    for (property, invalid) in [(TYPE, "default"), (POSITION, "none"), (IMAGE, "outside")] {
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_ident(invalid).unwrap()])
                .unwrap();
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(property),
                components,
                CssImportance::Normal,
            )
            .is_err()
        );
    }
}

#[test]
fn shorthand_contributions_follow_position_image_type_and_reset_omissions() {
    for (authored, expected) in [
        (
            "inside url(\"a\") square",
            ["inside", "url(\"a\")", "square"],
        ),
        ("inside", ["inside", "none", "disc"]),
        ("none", ["outside", "none", "none"]),
        ("none square", ["outside", "none", "square"]),
        ("none url(\"a\")", ["outside", "url(\"a\")", "none"]),
        ("none none", ["outside", "none", "none"]),
        ("inside outside", ["inside", "none", "outside"]),
    ] {
        let source = declaration(SHORTHAND, authored);
        let values = ordinary(&source);
        assert_eq!(values.len(), 3, "{authored}");
        for ((item, property), expected) in values.iter().zip(MEMBERS).zip(expected) {
            let explicit = ordinary(&declaration(property, expected));
            assert_eq!(item.property(), property, "{authored}");
            assert_eq!(
                item.ordinary_value(),
                explicit[0].ordinary_value(),
                "{authored}"
            );
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
}

#[test]
fn global_values_and_all_reset_keep_every_list_style_terminal_symbolic() {
    for property in [TYPE, POSITION, IMAGE, SHORTHAND] {
        let expected_members = if property == SHORTHAND {
            &MEMBERS[..]
        } else {
            &[property][..]
        };
        for (spelling, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(property, spelling);
            let values = ordinary(&source);
            assert_eq!(values.len(), expected_members.len());
            for (item, member) in values.iter().zip(expected_members) {
                assert_eq!(item.property(), *member);
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
    }
    let source = declaration(CssKnownProperty::All, "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("all has a universal reset")
    };
    for property in MEMBERS {
        assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
    }
}

#[test]
fn pending_shorthand_reentry_is_strict_atomic_and_preserves_source_identity() {
    let source = declaration(SHORTHAND, "var(--marker)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("substitution-dependent list-style shorthand is pending")
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid in [
        "none none square",
        "inside outside square",
        "linear-gradient(red)",
    ] {
        assert!(matches!(
            pending
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
    assert_eq!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    for replacement_css in [
        "inside outside",
        "none none",
        "inside src(\"#mark\") symbols(cyclic \"*\")",
        "inherit",
    ] {
        let replacement = parse_component_values(replacement_css).unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("three reentered terminals")
        };
        assert_eq!(values.items().len(), 3);
        for (item, property) in values.items().iter().zip(MEMBERS) {
            assert_eq!(item.property(), property);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn pending_list_style_longhands_reenter_their_own_grammar_without_losing_source() {
    for (property, valid_values, invalid) in [
        (TYPE, &["\"★\"", "CustomMarker"][..], "default"),
        (POSITION, &["inside"][..], "none"),
        (IMAGE, &["url(\"#mark\")"][..], "outside"),
    ] {
        let source = declaration(property, "var(--marker)");
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("{property:?} remains pending until substitution")
        };
        assert!(pending.source().same_occurrence(&source));

        for residual in [
            "var(--again)",
            "env(safe-area-inset-top)",
            "attr(data-marker)",
        ] {
            assert_eq!(
                pending
                    .reenter(parse_component_values(residual).unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution,
                "{property:?}: {residual}"
            );
        }
        for rejected in [invalid, ""] {
            assert!(matches!(
                pending
                    .reenter(parse_component_values(rejected).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
        }

        for replacement_css in valid_values.iter().copied().chain(["inherit"]) {
            let replacement = parse_component_values(replacement_css).unwrap();
            for _ in 0..2 {
                let CssContributions::Longhands(values) =
                    pending.reenter(replacement.clone()).unwrap()
                else {
                    panic!("{property:?} contributes one longhand")
                };
                let [item] = values.items() else {
                    panic!("one {property:?} terminal")
                };
                assert_eq!(item.property(), property);
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));

                if replacement_css == "inherit" {
                    assert_eq!(
                        item.value(),
                        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                    );
                } else {
                    match (
                        property,
                        replacement_css,
                        item.ordinary_value().unwrap().view(),
                    ) {
                        (
                            TYPE,
                            "\"★\"",
                            CssLonghandValueRef::ListStyleType(CssListStyleTypeValue::String(
                                value,
                            )),
                        ) => assert_eq!(value.as_str(), "★"),
                        (
                            TYPE,
                            "CustomMarker",
                            CssLonghandValueRef::ListStyleType(
                                CssListStyleTypeValue::CounterStyle(value),
                            ),
                        ) => assert_eq!(value.named().unwrap().as_str(), "CustomMarker"),
                        (POSITION, "inside", CssLonghandValueRef::ListStylePosition(value)) => {
                            assert_eq!(*value, CssListStylePosition::Inside)
                        }
                        (
                            IMAGE,
                            "url(\"#mark\")",
                            CssLonghandValueRef::ListStyleImage(CssImageValue::Url(value)),
                        ) => assert_eq!(value.as_str(), "#mark"),
                        _ => panic!("{property:?} accepted the expected authored value"),
                    }
                }
            }
        }
    }
}

#[test]
fn normalization_preserves_mixed_declaration_order_and_fails_at_member_limit() {
    let report = parse_sheet(".a{list-style-type:disc;color:red;list-style:inside}");
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
    assert_eq!(declarations.len(), 3);
    for (order, members) in [vec![TYPE], vec![CssKnownProperty::Color], MEMBERS.to_vec()]
        .into_iter()
        .enumerate()
    {
        assert_eq!(declarations[order].order(), order);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declarations[order].expansion()
        else {
            panic!("ordinary normalized declaration")
        };
        assert_eq!(
            values
                .items()
                .iter()
                .map(|item| item.property())
                .collect::<Vec<_>>(),
            members
        );
        for item in values.items() {
            assert!(item.source().same_occurrence(declarations[order].source()));
        }
    }
    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 4).unwrap();
    let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 4,
        }
    );
    assert_eq!(error.declaration_order(), Some(2));
    assert_eq!(
        error.declaration().unwrap().known().unwrap().property(),
        SHORTHAND
    );
}
