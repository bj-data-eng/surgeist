#![forbid(unsafe_code)]

//! CSS Logical 1 WD (2025-12-04) §4.5.4: one exact width/style/color
//! triple sets one flow-relative side or both sides of a flow-relative axis.
//! Source: https://www.w3.org/TR/2025/WD-css-logical-1-20251204/#border-shorthands

use surgeist_css::*;

const SIDES: [&str; 4] = [
    "border-block-start",
    "border-block-end",
    "border-inline-start",
    "border-inline-end",
];
const NAMES: [&str; 6] = [
    "border-block-start",
    "border-block-end",
    "border-inline-start",
    "border-inline-end",
    "border-block",
    "border-inline",
];

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("selected property: {name}"))
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}");
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
        panic!("one invalid-value diagnostic: {source}");
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
        panic!("{name} has a terminal longhand contribution");
    };
    let [item] = values.items() else {
        panic!("{name} has exactly one terminal contribution");
    };
    assert_eq!(item.property(), grammar(name).target_property());
    item.ordinary_value().unwrap().clone()
}

fn authored_color(value: &str) -> CssAuthoredColor {
    let source = declaration("border-top-color", value);
    let Some(CssKnownPropertyValueRef::BorderTopColor(color)) =
        source.known().unwrap().property_value()
    else {
        panic!("existing color longhand exposes the exact authored color");
    };
    color.current().clone()
}

fn members(name: &str) -> Vec<String> {
    let sides: &[&str] = match name {
        "border-block" => &["border-block-start", "border-block-end"],
        "border-inline" => &["border-inline-start", "border-inline-end"],
        side if SIDES.contains(&side) => &[side],
        _ => panic!("not one of the six selected shorthands: {name}"),
    };
    ["width", "style", "color"]
        .into_iter()
        .flat_map(|facet| sides.iter().map(move |side| format!("{side}-{facet}")))
        .collect()
}

fn expanded(name: &str, value: &str) -> CssDeclaration {
    let source = declaration(name, value);
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("{name}:{value} has terminal contributions");
    };
    let expected = members(name);
    assert_eq!(values.items().len(), expected.len());
    for (item, member) in values.items().iter().zip(expected) {
        assert_eq!(item.property(), grammar(&member).target_property());
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
    }
    source
}

fn rejection<T, E>(value: Result<T, E>) -> E {
    match value {
        Ok(_) => panic!("expected rejection"),
        Err(error) => error,
    }
}

#[test]
fn logical_border_names_have_distinct_selected_provenance_and_exact_membership() {
    let mut seen = Vec::new();
    for name in NAMES {
        let selected = grammar(name);
        let property = selected.target_property();
        assert_eq!(selected.name(), name);
        assert_eq!(property.canonical_name(), name);
        assert_eq!(
            selected.feature_id().as_str(),
            format!("official.property.{name}")
        );
        assert!(!seen.contains(&property), "{name} aliases another property");
        seen.push(property);
        let support = property_support_metadata(name).expect("selected support metadata");
        assert_eq!(support.property(), property);
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(
            support.feature().source().id().as_str(),
            "I-LOGICAL1-20251204"
        );
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2025/WD-css-logical-1-20251204/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
        let CssPropertyKindRef::Shorthand(metadata) = selected.metadata().unwrap().kind() else {
            panic!("{name} is a shorthand");
        };
        assert!(!metadata.is_legacy());
        assert_eq!(
            metadata
                .settable_members()
                .iter()
                .map(|member| member.known_property().canonical_name())
                .collect::<Vec<_>>(),
            members(name)
        );
        assert!(
            metadata.reset_only_members().is_empty(),
            "{name} resets an unrelated facet"
        );
        assert_eq!(metadata.members(), metadata.settable_members());
    }
    assert_eq!(seen.len(), 6);
}

#[test]
fn triples_accept_unordered_partial_and_exact_authored_components() {
    for name in NAMES {
        for value in [
            "solid",
            "thin",
            "red",
            "red solid 2px",
            "2px red solid",
            "solid 1e100px oklch(50% 0.2 30deg)",
        ] {
            accepted(name, value);
            expanded(name, value);
        }
        for value in [
            "",
            "solid dashed",
            "thin thick solid",
            "red blue",
            "-1e-100px solid",
            "1% solid",
            "logical solid",
        ] {
            invalid(name, value);
        }
    }
}

#[test]
fn omitted_facets_use_logical_longhand_initials_in_facet_major_order() {
    for name in NAMES {
        let source = expanded(name, "solid");
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("{name} expands");
        };
        for (item, member) in values.items().iter().zip(members(name)) {
            let initial_or_authored = if member.ends_with("-width") {
                "medium"
            } else if member.ends_with("-style") {
                "solid"
            } else {
                "currentcolor"
            };
            assert_eq!(
                item.ordinary_value(),
                Some(&one_value(&member, initial_or_authored))
            );
        }
    }
    // Existing checked aggregate independently models exactly the same omissions
    // and canonical specified order without a new logical-only domain type.
    let partial = CssBorderValue::try_new(None, Some(CssBorderStyle::Solid), None).unwrap();
    assert_eq!(partial.width(), None);
    assert_eq!(partial.style(), Some(CssBorderStyle::Solid));
    assert_eq!(partial.color(), None);
    assert_eq!(partial.serialize_specified().unwrap(), "solid");
    let full = CssBorderValue::try_new(
        Some(CssBorderWidth::Thin),
        Some(CssBorderStyle::Solid),
        Some(authored_color("currentcolor")),
    )
    .unwrap();
    assert_eq!(
        full.serialize_specified().unwrap(),
        "thin solid currentcolor"
    );
    let physical = declaration("border-top", "thin solid currentcolor");
    let Some(CssKnownPropertyValueRef::BorderTop(value)) =
        physical.known().unwrap().property_value()
    else {
        panic!("existing physical border triple exposes the shared typed model");
    };
    assert_eq!(value.current(), &full);
    assert!(CssBorderValue::try_new(None, None, None).is_none());
}

#[test]
fn explicit_width_style_and_color_reach_each_logical_target_without_mapping() {
    for name in NAMES {
        let source = expanded(name, "oklch(50% 0.2 30deg) solid 1e100px");
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("{name} expands");
        };
        for (item, member) in values.items().iter().zip(members(name)) {
            let authored = if member.ends_with("-width") {
                "1e100px"
            } else if member.ends_with("-style") {
                "solid"
            } else {
                "oklch(50% 0.2 30deg)"
            };
            if member.ends_with("-color") {
                let color = match item.ordinary_value().unwrap().view() {
                    CssLonghandValueRef::BorderBlockStartColor(color)
                    | CssLonghandValueRef::BorderBlockEndColor(color)
                    | CssLonghandValueRef::BorderInlineStartColor(color)
                    | CssLonghandValueRef::BorderInlineEndColor(color) => color,
                    _ => panic!("{member} contributes its authored color"),
                };
                let oklch = color.oklch_value().expect("exact OKLCH color");
                assert!(
                    matches!(oklch.lightness(), CssAuthoredColorComponent::Percentage(value) if value.value() == 50.0)
                );
                assert!(
                    matches!(oklch.chroma(), CssAuthoredColorComponent::ExactNumber(value) if value.numeric().representation() == "0.2")
                );
                assert!(
                    matches!(oklch.hue(), CssAuthoredHue::Angle(value) if value.value() == 30.0 && value.unit() == CssAngleUnit::Degrees)
                );
                assert_eq!(oklch.alpha(), None);
                let color_only = CssBorderValue::try_new(None, None, Some(color.clone())).unwrap();
                assert_eq!(
                    color_only.serialize_specified().unwrap(),
                    "oklch(0.5 0.2 30)"
                );
            } else {
                assert_eq!(item.ordinary_value(), Some(&one_value(&member, authored)));
            }
        }
    }
}

#[test]
fn globals_and_all_cover_every_logical_member_without_image_resets() {
    let all = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all).unwrap()
    else {
        panic!("all remains a symbolic universal reset");
    };
    for name in NAMES {
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
                panic!("{name}:{text} expands the CSS-wide keyword");
            };
            assert_eq!(values.items().len(), members(name).len());
            for (item, member) in values.items().iter().zip(members(name)) {
                assert_eq!(item.property(), grammar(&member).target_property());
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
    }
}

#[test]
fn pending_var_and_env_reenter_the_same_logical_grammar_and_occurrence() {
    for name in NAMES {
        for token in ["var(--border)", "env(--border)"] {
            let source = declaration(name, token);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name}:{token} remains pending");
            };
            assert!(handle.source().same_occurrence(&source));
            assert!(matches!(
                rejection(handle.reenter(parse_component_values("1%").unwrap())).kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
            assert!(matches!(
                rejection(handle.reenter(parse_component_values("solid solid").unwrap())).kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
            assert_eq!(
                rejection(handle.reenter(parse_component_values("var(--again)").unwrap())).kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let replacement = parse_component_values("solid 1e100px red").unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("{name} reenters to logical longhands");
            };
            assert_eq!(values.items().len(), members(name).len());
            for (item, member) in values.items().iter().zip(members(name)) {
                assert_eq!(item.property(), grammar(&member).target_property());
                assert!(item.ordinary_value().is_some());
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
        }
    }
}

#[test]
fn normalization_keeps_logical_order_context_and_atomic_six_member_budget() {
    let report = parse_sheet(
        ".p{border-block-start:solid;@media all{border-inline:thin dashed red}border-block-end:blue}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let occurrences: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(occurrences.len(), 3);
    for (index, (item, (name, count))) in occurrences
        .iter()
        .zip([
            ("border-block-start", 3),
            ("border-inline", 6),
            ("border-block-end", 3),
        ])
        .enumerate()
    {
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().known().unwrap().property(),
            grammar(name).target_property()
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("{name} has grouped logical contributions");
        };
        assert_eq!(values.items().len(), count);
        for (contribution, member) in values.items().iter().zip(members(name)) {
            assert_eq!(contribution.property(), grammar(&member).target_property());
            assert!(contribution.source().same_occurrence(item.source()));
        }
        assert_eq!(item.source().importance(), CssImportance::Normal);
        assert!(item.source().position().is_some());
    }
    assert!(
        occurrences[0]
            .selector_context()
            .same_context(occurrences[1].selector_context())
    );
    assert!(
        occurrences[0]
            .selector_context()
            .same_context(occurrences[2].selector_context())
    );
    assert!(
        occurrences[1]
            .rule_context()
            .parent()
            .is_some_and(|parent| matches!(parent.kind(), CssRuleContextKindRef::Media(_)))
    );

    let report = parse_sheet(".p{color:red;border-inline:solid;color:blue}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(style)] = report.syntax().rules() else {
        panic!("one style rule");
    };
    let source = &style.declarations()[1];
    let original = report.syntax().clone();
    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 6).unwrap();
    let error = rejection(normalize_sheet_with_limits(report.syntax(), limits));
    assert!(matches!(
        error.kind(),
        CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 6
        }
    ));
    assert_eq!(error.declaration_order(), Some(1));
    assert_eq!(error.position(), source.position());
    assert_eq!(error.declaration(), Some(source));
    assert!(matches!(
        error.rule_context().map(CssRuleContext::kind),
        Some(CssRuleContextKindRef::Style(_))
    ));
    assert_eq!(report.syntax(), &original, "failure did not mutate source");
}

#[test]
fn physical_border_controls_keep_existing_side_and_image_footprints() {
    for (name, value, expected_count, reset_count) in
        [("border-top", "solid", 3, 0), ("border", "solid", 17, 5)]
    {
        let source = declaration(name, value);
        let CssPropertyKindRef::Shorthand(metadata) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a shorthand");
        };
        assert_eq!(metadata.reset_only_members().len(), reset_count);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("{name} expands");
        };
        assert_eq!(values.items().len(), expected_count);
    }
}
