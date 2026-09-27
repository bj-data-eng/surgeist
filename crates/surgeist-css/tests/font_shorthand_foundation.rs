#![forbid(unsafe_code)]

//! CSS Fonts 4 WD (2026-09-07) §2.7: `font` sets seven longhands and resets
//! twelve others. These expectations follow the pinned specification's two
//! named groups, not the current shorthand implementation.

use surgeist_css::*;

const SETTABLE: [&str; 7] = [
    "font-family",
    "font-size",
    "font-width",
    "font-style",
    "font-variant-caps",
    "font-weight",
    "line-height",
];
const RESET_ONLY: [&str; 12] = [
    "font-feature-settings",
    "font-kerning",
    "font-language-override",
    "font-optical-sizing",
    "font-size-adjust",
    "font-variant-alternates",
    "font-variant-east-asian",
    "font-variant-emoji",
    "font-variant-ligatures",
    "font-variant-numeric",
    "font-variant-position",
    "font-variation-settings",
];

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("missing grammar: {name}"))
}

fn declaration(value: &str) -> CssDeclaration {
    let source = format!("font:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one font declaration: {source}")
    };
    declaration.clone()
}

fn direct(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).unwrap(),
        CssImportance::Normal,
    )
    .unwrap_or_else(|error| panic!("{name}:{value}: {error:?}"))
}

#[test]
fn font_metadata_has_seven_settable_and_twelve_reset_only_members() {
    let CssPropertyKindRef::Shorthand(metadata) = grammar("font").metadata().unwrap().kind() else {
        panic!("font shorthand metadata")
    };
    assert_eq!(
        metadata
            .settable_members()
            .iter()
            .map(|property| property.known_property().canonical_name())
            .collect::<Vec<_>>(),
        SETTABLE
    );
    assert_eq!(
        metadata
            .reset_only_members()
            .iter()
            .map(|property| property.known_property().canonical_name())
            .collect::<Vec<_>>(),
        RESET_ONLY
    );
}

#[test]
fn explicit_font_sets_present_values_and_resets_every_other_member() {
    let source = declaration("condensed oblique 25deg 753 16px/20px serif");
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("explicit font expands")
    };
    assert_eq!(values.items().len(), 19);
    for (item, name) in values
        .items()
        .iter()
        .zip(SETTABLE.into_iter().chain(RESET_ONLY))
    {
        assert_eq!(item.property(), grammar(name).target_property(), "{name}");
        assert!(item.source().same_occurrence(&source), "{name}");
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
        let authored = match name {
            "font-family" => Some("serif"),
            "font-size" => Some("16px"),
            "font-width" => Some("condensed"),
            "font-style" => Some("oblique 25deg"),
            "font-weight" => Some("753"),
            "line-height" => Some("20px"),
            _ => None,
        };
        if let Some(authored) = authored {
            let CssExpansion::Contributions(CssContributions::Longhands(expected)) =
                expand_declaration(&direct(name, authored)).unwrap()
            else {
                panic!("direct {name} expands")
            };
            assert_eq!(
                item.ordinary_value(),
                expected.items()[0].ordinary_value(),
                "{name}"
            );
        } else {
            let initial = match name {
                "font-variant-caps"
                | "font-feature-settings"
                | "font-language-override"
                | "font-variant-alternates"
                | "font-variant-east-asian"
                | "font-variant-emoji"
                | "font-variant-ligatures"
                | "font-variant-numeric"
                | "font-variant-position"
                | "font-variation-settings" => "normal",
                "font-kerning" | "font-optical-sizing" => "auto",
                "font-size-adjust" => "none",
                _ => unreachable!("every font member has a pinned initial"),
            };
            let CssExpansion::Contributions(CssContributions::Longhands(expected)) =
                expand_declaration(&direct(name, initial)).unwrap()
            else {
                panic!("direct {name} initial expands")
            };
            assert_eq!(
                item.ordinary_value(),
                expected.items()[0].ordinary_value(),
                "{name}"
            );
        }
    }
    assert!(matches!(
        values.items()[2]
            .ordinary_value()
            .map(CssLonghandValue::view),
        Some(CssLonghandValueRef::FontWidth(CssFontWidth::Keyword(
            CssFontWidthKeyword::Condensed
        )))
    ));
    assert!(matches!(
        values.items()[4]
            .ordinary_value()
            .map(CssLonghandValue::view),
        Some(CssLonghandValueRef::FontVariantCaps(
            &CssFontVariantCaps::Normal
        ))
    ));
    assert!(matches!(
        values.items()[8]
            .ordinary_value()
            .map(CssLonghandValue::view),
        Some(CssLonghandValueRef::FontKerning(&CssFontKerning::Auto))
    ));
    assert!(matches!(
        values.items()[10]
            .ordinary_value()
            .map(CssLonghandValue::view),
        Some(CssLonghandValueRef::FontOpticalSizing(
            &CssFontOpticalSizing::Auto
        ))
    ));
    assert!(matches!(
        values.items()[11]
            .ordinary_value()
            .map(CssLonghandValue::view),
        Some(CssLonghandValueRef::FontSizeAdjust(
            &CssFontSizeAdjust::None
        ))
    ));
    let Some(CssLonghandValueRef::FontWeight(weight)) = values.items()[5]
        .ordinary_value()
        .map(CssLonghandValue::view)
    else {
        panic!("authored weight")
    };
    assert_eq!(weight.serialize_specified().unwrap(), "753");

    let caps = declaration("small-caps 16px serif");
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&caps).unwrap()
    else {
        panic!("small-caps font expands")
    };
    let CssExpansion::Contributions(CssContributions::Longhands(expected)) =
        expand_declaration(&direct("font-variant-caps", "small-caps")).unwrap()
    else {
        panic!("small-caps longhand expands")
    };
    assert_eq!(
        values.items()[4].ordinary_value(),
        expected.items()[0].ordinary_value()
    );
    assert!(matches!(
        values.items()[2]
            .ordinary_value()
            .map(CssLonghandValue::view),
        Some(CssLonghandValueRef::FontWidth(CssFontWidth::Keyword(
            CssFontWidthKeyword::Normal
        )))
    ));
    assert!(matches!(
        values.items()[3]
            .ordinary_value()
            .map(CssLonghandValue::view),
        Some(CssLonghandValueRef::FontStyle(CssFontStyle::Keyword(
            CssFontStyleKeyword::Normal
        )))
    ));
    assert!(matches!(
        values.items()[6]
            .ordinary_value()
            .map(CssLonghandValue::view),
        Some(CssLonghandValueRef::LineHeight(&CssLineHeight::Normal))
    ));
}

#[test]
fn css_wide_and_pending_font_values_cover_all_nineteen_members() {
    let source = declaration("inherit");
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("inherit font expands")
    };
    assert_eq!(values.items().len(), 19);
    for (item, name) in values
        .items()
        .iter()
        .zip(SETTABLE.into_iter().chain(RESET_ONLY))
    {
        assert_eq!(item.property(), grammar(name).target_property());
        assert_eq!(
            item.value(),
            CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
        );
        assert!(item.source().same_occurrence(&source));
    }

    let pending_source = declaration("var(--typeface)");
    let CssExpansion::Pending(pending) = expand_declaration(&pending_source).unwrap() else {
        panic!("font substitution remains pending")
    };
    assert!(pending.source().same_occurrence(&pending_source));
    assert!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .is_err()
    );
    assert!(
        pending
            .reenter(parse_component_values("bold 700 16px serif").unwrap())
            .is_err()
    );
    let CssContributions::Longhands(values) = pending
        .reenter(parse_component_values("small-caps 16px serif").unwrap())
        .unwrap()
    else {
        panic!("valid substitution expands")
    };
    assert_eq!(values.items().len(), 19);
    for item in values.items() {
        assert!(item.source().same_occurrence(&pending_source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_some());
    }
}

#[test]
fn font_grammar_retains_pinned_controls_and_neighbor_recovery() {
    for value in [
        "condensed oblique 25deg 753 16px serif",
        "normal normal normal normal 16px serif",
    ] {
        let source = format!("font:{value}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(direct("font", value).known().is_some());
    }
    for value in [
        "normal normal normal normal normal 16px serif",
        "bold 700 16px serif",
        "75% condensed 16px serif",
    ] {
        let source = format!("color:red;font:{value};color:blue");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert!(direct("color", "red").known().is_some());
        assert!(
            parse_property_value_for_grammar(
                grammar("font"),
                parse_component_values(value).unwrap(),
                CssImportance::Normal,
            )
            .is_err()
        );
    }
}

#[test]
fn normalized_font_keeps_order_and_enforces_nineteen_contribution_limit() {
    let report = parse_sheet(".a{font:small-caps 16px serif!important;color:red}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 2, 19).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 19,
        }
    );
    let normalized = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 2, 20).unwrap(),
    )
    .unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(declaration) => Some(declaration),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 2);
    assert_eq!(declarations[0].order(), 0);
    assert_eq!(declarations[1].order(), 1);
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        declarations[0].expansion()
    else {
        panic!("font normalized expansion")
    };
    assert_eq!(values.items().len(), 19);
    assert!(
        values
            .items()
            .iter()
            .all(|item| item.source().same_occurrence(declarations[0].source()))
    );
}
