#![forbid(unsafe_code)]

//! Existing-public-API contracts from CSS Fonts 4 WD (2026-09-07) §§2.3, 2.3.1, 2.7.
//! Exact typed width payloads follow with the functional model.

use surgeist_css::*;

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("known grammar: {name}"))
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained declaration: {source}")
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

fn one_contribution(
    source: &CssDeclaration,
    property: CssKnownProperty,
) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("font width intrinsic expansion")
    else {
        panic!("one longhand contribution")
    };
    let [item] = values.items() else {
        panic!("one longhand contribution")
    };
    assert_eq!(item.property(), property);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    item.clone()
}

#[test]
fn legacy_stretch_name_resolves_to_one_canonical_font_width_identity() {
    let canonical = grammar("font-width");
    let alias = grammar("FONT-STRETCH");
    let property = canonical.target_property();
    assert_eq!(canonical.name(), "font-width");
    assert_eq!(property.canonical_name(), "font-width");
    assert_eq!(alias, canonical);
    assert_eq!(CssKnownProperty::from_name("font-stretch"), Some(property));
    assert_eq!(property.aliases(), &["font-stretch"]);
    assert_eq!(
        canonical.feature_id().as_str(),
        "baseline.property.font-stretch"
    );
    assert_eq!(alias.feature_id(), canonical.feature_id());
    for name in ["font-width", "font-stretch"] {
        let support = property_support_metadata(name).expect("width support");
        assert_eq!(support.property(), property);
        assert_eq!(support.canonical_name(), "font-width");
        assert_eq!(support.aliases(), &["font-stretch"]);
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/")
        );
        assert_eq!(support.feature().production(), "#propdef-font-width");
    }
    let source = declaration("FoNt-StReTcH", "75%");
    assert_eq!(source.known().unwrap().property(), property);
    let origin = source.parsed_name().expect("parsed alias name");
    let span = origin.span();
    assert_eq!(
        &origin.source().as_str()
            [span.start().byte_offset().value()..span.end().byte_offset().value()],
        "FoNt-StReTcH"
    );
}

#[test]
fn both_names_accept_keywords_and_nonnegative_percentages_but_recover_bad_values() {
    for name in ["font-width", "font-stretch"] {
        for value in [
            "normal",
            "ultra-condensed",
            "extra-condensed",
            "condensed",
            "semi-condensed",
            "semi-expanded",
            "expanded",
            "extra-expanded",
            "ultra-expanded",
            "0%",
            "75%",
            "125.125%",
            "200%",
            "100000000000000000000000000000000000%",
        ] {
            for source in [declaration(name, value), checked(name, value)] {
                assert_eq!(
                    source.known().unwrap().property(),
                    grammar("font-width").target_property()
                );
                assert_eq!(source.importance(), CssImportance::Important);
            }
        }
        for value in ["0", "-1%", "-1e-999%", "75% 125%", "wide", "normal 75%"] {
            let source = format!("color:red;{name}:{value};color:blue");
            let report = parse_style_attribute(&source);
            let [diagnostic] = report.diagnostics() else {
                panic!("one invalid value: {source}")
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
                    CssImportance::Normal
                )
                .is_err()
            );
        }
    }
}

#[test]
fn inherited_normal_width_expands_once_and_pending_reentry_uses_same_grammar() {
    let property = grammar("font-width").target_property();
    let CssPropertyKindRef::Longhand(longhand) = grammar("font-width")
        .metadata()
        .expect("width metadata")
        .kind()
    else {
        panic!("font width is a longhand")
    };
    assert!(longhand.inherited_by_default());
    assert_eq!(longhand.property().known_property(), property);
    let initial = longhand.initial_value();
    let CssInitialValueRef::Value(initial_value) = initial.view() else {
        panic!("fixed normal initial")
    };
    let normal = declaration("font-width", "normal");
    assert_eq!(
        initial_value,
        one_contribution(&normal, property)
            .ordinary_value()
            .unwrap()
    );

    for (name, value) in [("font-width", "75%"), ("font-stretch", "condensed")] {
        for source in [declaration(name, value), checked(name, value)] {
            let item = one_contribution(&source, property);
            assert!(item.ordinary_value().is_some());
            assert!(item.replacement_components().is_none());
        }
    }
    for (value, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
    ] {
        let source = declaration("font-stretch", value);
        let item = one_contribution(&source, property);
        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
    }
    let source = declaration("font-stretch", "var(--width)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("substitution remains pending")
    };
    assert!(pending.source().same_occurrence(&source));
    assert_eq!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    assert!(matches!(
        pending
            .reenter(parse_component_values("-1%").unwrap())
            .unwrap_err()
            .kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
    let replacement = parse_component_values("75%").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("strict replacement completes")
    };
    let [item] = values.items() else {
        panic!("one substituted contribution")
    };
    assert_eq!(item.property(), property);
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.replacement_components(), Some(&replacement));
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&declaration("all", "initial")).unwrap()
    else {
        panic!("all reset")
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
}

#[test]
fn font_shorthand_keeps_its_nine_keyword_width_component() {
    for value in ["condensed 16px serif", "ultra-expanded 16px serif"] {
        declaration("font", value);
        checked("font", value);
    }
    let report = parse_style_attribute("color:red;font:75% 16px serif;color:blue");
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("percentage is not a shorthand width")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert!(validate_style_attribute("font:75% 16px serif").is_err());
}
