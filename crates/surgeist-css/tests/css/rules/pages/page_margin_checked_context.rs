//! Detached Page declaration construction through the checked Page front.
use crate::css::support::property_admission::property_value_facts;
use crate::css::support::rules::ExpectedRejection;
use surgeist_css::*;

const MARGINS: [CssKnownProperty; 5] = [
    CssKnownProperty::Margin,
    CssKnownProperty::MarginTop,
    CssKnownProperty::MarginRight,
    CssKnownProperty::MarginBottom,
    CssKnownProperty::MarginLeft,
];

fn checked(property: CssKnownProperty, value: &str, context: CssParserContext) -> CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let original = components.clone();
    let declaration = context
        .parse_page_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Important,
        )
        .unwrap();
    assert_eq!(declaration.known().unwrap().property(), property);
    assert_eq!(declaration.value_components(), &original);
    assert_eq!(declaration.parser_context(), context);
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    declaration
}

#[test]
fn checked_page_front_admits_symbolic_values_with_both_modes_and_preserved_origins() {
    for mode in [CssParserMode::Standards, CssParserMode::Quirks] {
        let context = CssParserContext::new(mode);
        for property in MARGINS {
            for value in [
                "initial",
                "inherit",
                "unset",
                "revert",
                "revert-layer",
                "-4Q",
                "2ch",
                "3rem",
                "calc(1px + 2%)",
                "calc(1cm - 2mm)",
                "1cqw",
                "2cqh",
                "3cqi",
                "4cqb",
                "5cqmin",
                "6cqmax",
                "var(--m, 1em)",
            ] {
                let declaration = checked(property, value, context);
                let text = declaration.serialize_specified(Default::default()).unwrap();
                assert!(context.parse_page_block(&format!("{{{text}}}")).is_clean());
                if mode == CssParserMode::Standards {
                    let free = parse_page_property_value(
                        CssPropertyNameRef::Known(property),
                        declaration.value_components().clone(),
                        CssImportance::Important,
                    )
                    .unwrap();
                    assert_eq!(free, declaration);
                }
            }
        }
    }
}

#[test]
fn checked_page_restrictions_preserve_original_offending_component() {
    for mode in [CssParserMode::Standards, CssParserMode::Quirks] {
        let context = CssParserContext::new(mode);
        for property in MARGINS {
            for value in ["1zz", "1yy", "0zz", "0yy", r"1\7a z", "7"] {
                let components = parse_component_values(value).unwrap();
                let origin = components.items()[0].origin().clone();
                let original = components.clone();
                let error = context
                    .parse_page_property_value(
                        CssPropertyNameRef::Known(property),
                        components.clone(),
                        CssImportance::Normal,
                    )
                    .unwrap_err();
                assert!(
                    (error.code() == Some(CssErrorCode::InvalidPropertyValue)
                        && error.known_property().unwrap() == property),
                    "{value}: {error:?}"
                );
                assert_eq!(error.origin(), CssSerializedOrigin::Token(origin));
                assert_eq!(components, original);
            }
        }
    }
}

#[test]
fn page_reentry_maps_nested_invalid_unit_and_matches_direct_checked_grammar() {
    let source = checked(
        CssKnownProperty::MarginLeft,
        "var(--m)",
        CssParserContext::default(),
    );
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("Page pending")
    };
    for value in ["calc(1px + 2zz)", "calc(0 * 1yy)", "calc(1px + (0 * 1zz))"] {
        let replacement = parse_component_values(&format!("/*😀*/{value}")).unwrap();
        let before = replacement.clone();
        let direct = parse_page_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::MarginLeft),
            replacement.clone(),
            CssImportance::Important,
        )
        .unwrap_err();
        let error = handle.reenter(replacement.clone()).unwrap_err();
        let Some(actual) = error.replacement_error() else {
            panic!("typed Page replacement failure")
        };
        assert_eq!(property_value_facts(actual), property_value_facts(&direct));
        assert_eq!(actual.origin(), direct.origin());
        let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = actual.origin() else {
            panic!("original dimension token")
        };
        assert!(matches!(
            &origin.source().as_str()[origin.span().start().byte_offset().value()
                ..origin.span().end().byte_offset().value()],
            "2zz" | "1yy" | "1zz"
        ));
        assert_eq!(replacement, before);
    }
    assert!(
        handle
            .reenter(parse_component_values("calc(1cqw + 2%)").unwrap())
            .is_ok()
    );
}

#[test]
fn checked_page_pending_context_survives_priority_clones_list_assembly_and_retries() {
    let source = checked(
        CssKnownProperty::Margin,
        "var(--m)",
        CssParserContext::new(CssParserMode::Quirks),
    );
    let ordinary = CssParserContext::new(CssParserMode::Quirks)
        .parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Margin),
            source.value_components().clone(),
            CssImportance::Important,
        )
        .unwrap();
    assert_ne!(source, ordinary);
    let changed = source.with_importance(CssImportance::Normal);
    assert!(!source.same_occurrence(&changed));
    let list = CssDeclarationList::try_new(
        vec![source.clone(), changed.clone(), changed.clone()],
        Default::default(),
    )
    .unwrap();
    for (index, declaration) in list.iter().enumerate() {
        let CssExpansion::Pending(handle) = expand_declaration(declaration).unwrap() else {
            panic!("checked Page pending")
        };
        assert!(
            handle
                .source()
                .same_occurrence(if index == 0 { &source } else { &changed })
        );
        for _ in 0..2 {
            // Box 3 §3.1: margin is <'margin-top'>{1,4}; page context admits
            // no unitless length and no unknown unit.
            for (invalid, token) in [
                ("7", "7"),
                ("1px 2zz", "2zz"),
                ("1px 2px 3yy", "3yy"),
                ("1px 2px 3px calc(0 * 1zz)", "1zz"),
                ("1px 2px 3px 4px 5px", "5px"),
            ] {
                let error = handle
                    .reenter(parse_component_values(invalid).unwrap())
                    .unwrap_err();
                let Some(replacement) = error.replacement_error() else {
                    panic!("{invalid}: invalid replacement: {error:?}")
                };
                ExpectedRejection::new(CssErrorCode::InvalidPropertyValue)
                    .named("margin")
                    .at(token)
                    .assert_checked(replacement);
            }
            let replacement = parse_component_values("1Q 2ch 3rem calc(1cqmin + 2%)").unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("all physical terminals")
            };
            assert_eq!(values.items().len(), 4);
            for item in values.items() {
                assert!(item.source().same_occurrence(declaration));
                assert_eq!(item.replacement_components(), Some(&replacement));
                assert_eq!(item.source().importance(), declaration.importance());
            }
        }
    }
    let CssExpansion::Pending(handle) = expand_declaration(&ordinary).unwrap() else {
        panic!("ordinary pending")
    };
    assert!(
        handle
            .reenter(parse_component_values("1em").unwrap())
            .is_ok()
    );
}

#[test]
fn checked_page_custom_values_retain_own_tokens_without_margin_restrictions() {
    let name = CssCustomPropertyName::try_new("--M").unwrap();
    for value in ["", "1em", "var(--other,)", "{[var(--x)]}", "initial"] {
        let components = parse_component_values(value).unwrap();
        let declaration = parse_page_property_value(
            CssPropertyNameRef::Custom(&name),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(declaration.value_components(), &components);
        assert_eq!(declaration.custom().unwrap().name(), &name);
        assert!(matches!(
            expand_declaration(&declaration).unwrap(),
            CssExpansion::Contributions(CssContributions::Custom(_))
        ));
    }
    for (property, value) in [
        (CssKnownProperty::Opacity, ".5"),
        (CssKnownProperty::Display, "block"),
        (CssKnownProperty::MarginInlineStart, "1px"),
        (CssKnownProperty::All, "initial"),
    ] {
        assert!(
            parse_page_property_value(
                CssPropertyNameRef::Known(property),
                parse_component_values(value).unwrap(),
                CssImportance::Normal
            )
            .is_err()
        );
    }
    assert!(
        parse_page_property_value(
            CssPropertyNameRef::SvgGlyphOrientationVertical,
            parse_component_values("0deg").unwrap(),
            CssImportance::Normal
        )
        .is_err()
    );
}

#[test]
fn checked_page_rejects_recovered_components_before_pending_or_intrinsic_grammar() {
    for value in ["calc(1px + 2%", "var(--m", "1px/*unfinished"] {
        let components = parse_component_values(value).unwrap();
        let error = parse_page_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::MarginLeft),
            components,
            CssImportance::Normal,
        )
        .unwrap_err();
        assert!(error.code() == Some(CssErrorCode::UnexpectedEnd));
        assert!(matches!(
            error.origin(),
            CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { .. }))
        ));
    }
}
