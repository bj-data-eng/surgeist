use surgeist_css::{
    CssAuthoredFontFeatureSettings, CssAuthoredFontFeatureValue, CssErrorCode,
    CssFontFamilyNameKind, CssFontFeatureSettings, CssFontFeatureValue, CssFontSize, CssFontStyle,
    CssFontSynthesis, CssFontValue, CssFontVariant, CssFontVariantCaps, CssFontVariantEastAsian,
    CssFontVariantEastAsianVariant, CssFontVariantEastAsianWidth, CssFontVariantLigatureState,
    CssFontVariantLigatures, CssFontVariantNumeric, CssFontVariantNumericFigure,
    CssFontVariantNumericFraction, CssFontVariantNumericSpacing, CssFontVariantPosition,
    CssFontVariantValue, CssGenericFontFamily, CssKnownDeclaredValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssLineHeight, CssNonNegativeNumberValue, CssSystemFont,
    parse_style_attribute,
};

#[test]
fn font_variant_longhands_and_shorthand_enforce_keyword_groups() {
    let report = parse_style_attribute(concat!(
        "font-variant-caps: all-small-caps; ",
        "font-variant-east-asian: ruby jis04 full-width; ",
        "font-variant-ligatures: no-contextual common-ligatures; ",
        "font-variant-numeric: slashed-zero oldstyle-nums tabular-nums diagonal-fractions ordinal; ",
        "font-variant-position: super; ",
        "font-variant: no-contextual common-ligatures super all-small-caps ",
        "oldstyle-nums tabular-nums diagonal-fractions ordinal slashed-zero ",
        "ruby jis04 full-width; ",
        "color: red",
    ));

    if !report.is_clean() {
        assert_eq!(report.diagnostics().len(), 6);
        assert_eq!(
            report
                .diagnostics()
                .iter()
                .filter(|diagnostic| diagnostic.error().code() == CssErrorCode::UnknownProperty)
                .count(),
            5,
        );
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
        );
    }

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 7);
    assert_eq!(
        report
            .syntax()
            .iter()
            .map(|declaration| declaration.known().unwrap().property().canonical_name())
            .collect::<Vec<_>>(),
        [
            "font-variant-caps",
            "font-variant-east-asian",
            "font-variant-ligatures",
            "font-variant-numeric",
            "font-variant-position",
            "font-variant",
            "color",
        ],
    );

    let CssKnownPropertyValueRef::FontVariantCaps(caps) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-variant-caps");
    };
    assert_eq!(caps.caps(), &CssFontVariantCaps::AllSmallCaps);

    let CssKnownPropertyValueRef::FontVariantEastAsian(east_asian) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-variant-east-asian");
    };
    let CssFontVariantEastAsian::Values(east_asian) = east_asian.east_asian() else {
        panic!("expected East Asian values");
    };
    assert_eq!(
        east_asian.variant(),
        Some(CssFontVariantEastAsianVariant::Jis04)
    );
    assert_eq!(
        east_asian.width(),
        Some(CssFontVariantEastAsianWidth::FullWidth)
    );
    assert!(east_asian.ruby());

    let CssKnownPropertyValueRef::FontVariantLigatures(ligatures) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-variant-ligatures");
    };
    let CssFontVariantLigatures::Values(ligatures) = ligatures.ligatures() else {
        panic!("expected ligature values");
    };
    assert_eq!(
        ligatures.common(),
        Some(CssFontVariantLigatureState::Enabled)
    );
    assert_eq!(
        ligatures.contextual(),
        Some(CssFontVariantLigatureState::Disabled)
    );

    let CssKnownPropertyValueRef::FontVariantNumeric(numeric) = report.syntax()[3]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-variant-numeric");
    };
    let CssFontVariantNumeric::Values(numeric) = numeric.numeric() else {
        panic!("expected numeric values");
    };
    assert_eq!(
        numeric.figure(),
        Some(CssFontVariantNumericFigure::OldstyleNums)
    );
    assert_eq!(
        numeric.spacing(),
        Some(CssFontVariantNumericSpacing::TabularNums)
    );
    assert_eq!(
        numeric.fraction(),
        Some(CssFontVariantNumericFraction::DiagonalFractions)
    );
    assert!(numeric.ordinal());
    assert!(numeric.slashed_zero());

    let CssKnownPropertyValueRef::FontVariantPosition(position) = report.syntax()[4]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-variant-position");
    };
    assert_eq!(position.position(), &CssFontVariantPosition::Super);

    let CssKnownPropertyValueRef::FontVariant(variant) = report.syntax()[5]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-variant");
    };
    let CssFontVariantValue::Values(values) = variant.variant() else {
        panic!("expected shorthand component values");
    };
    assert_eq!(values.caps(), Some(CssFontVariantCaps::AllSmallCaps));
    assert_eq!(values.position(), Some(CssFontVariantPosition::Super));
    assert!(values.ligatures().is_some());
    assert!(values.numeric().is_some());
    assert!(values.east_asian().is_some());
    assert!(variant.i01_subset().is_none());
}

#[test]
fn font_variant_keywords_unordered_groups_and_i01_projection_are_exact() {
    for keyword in [
        "normal",
        "small-caps",
        "all-small-caps",
        "petite-caps",
        "all-petite-caps",
        "unicase",
        "titling-caps",
    ] {
        let report = parse_style_attribute(&format!("font-variant-caps: {keyword}"));
        assert!(report.is_clean(), "{keyword}: {:?}", report.diagnostics());
    }

    for keyword in [
        "normal",
        "none",
        "common-ligatures",
        "no-common-ligatures",
        "discretionary-ligatures",
        "no-discretionary-ligatures",
        "historical-ligatures",
        "no-historical-ligatures",
        "contextual",
        "no-contextual",
        "small-caps",
        "all-small-caps",
        "petite-caps",
        "all-petite-caps",
        "unicase",
        "titling-caps",
        "sub",
        "super",
        "lining-nums",
        "oldstyle-nums",
        "proportional-nums",
        "tabular-nums",
        "diagonal-fractions",
        "stacked-fractions",
        "ordinal",
        "slashed-zero",
        "jis78",
        "jis83",
        "jis90",
        "jis04",
        "simplified",
        "traditional",
        "full-width",
        "proportional-width",
        "ruby",
    ] {
        let report = parse_style_attribute(&format!("font-variant: {keyword}"));
        assert!(report.is_clean(), "{keyword}: {:?}", report.diagnostics());
    }
    for keyword in ["normal", "sub", "super"] {
        let report = parse_style_attribute(&format!("font-variant-position: {keyword}"));
        assert!(report.is_clean(), "{keyword}: {:?}", report.diagnostics());
    }
    for keyword in [
        "normal",
        "none",
        "common-ligatures",
        "no-common-ligatures",
        "discretionary-ligatures",
        "no-discretionary-ligatures",
        "historical-ligatures",
        "no-historical-ligatures",
        "contextual",
        "no-contextual",
    ] {
        let report = parse_style_attribute(&format!("font-variant-ligatures: {keyword}"));
        assert!(report.is_clean(), "{keyword}: {:?}", report.diagnostics());
    }
    for keyword in [
        "normal",
        "lining-nums",
        "oldstyle-nums",
        "proportional-nums",
        "tabular-nums",
        "diagonal-fractions",
        "stacked-fractions",
        "ordinal",
        "slashed-zero",
    ] {
        let report = parse_style_attribute(&format!("font-variant-numeric: {keyword}"));
        assert!(report.is_clean(), "{keyword}: {:?}", report.diagnostics());
    }
    for keyword in [
        "normal",
        "jis78",
        "jis83",
        "jis90",
        "jis04",
        "simplified",
        "traditional",
        "full-width",
        "proportional-width",
        "ruby",
    ] {
        let report = parse_style_attribute(&format!("font-variant-east-asian: {keyword}"));
        assert!(report.is_clean(), "{keyword}: {:?}", report.diagnostics());
    }

    let unordered = parse_style_attribute(concat!(
        "font-variant-ligatures: contextual historical-ligatures common-ligatures discretionary-ligatures; ",
        "font-variant-numeric: ordinal stacked-fractions proportional-nums slashed-zero lining-nums; ",
        "font-variant-east-asian: ruby proportional-width traditional",
    ));
    assert!(unordered.is_clean(), "{:?}", unordered.diagnostics());

    let compatibility =
        parse_style_attribute("font-variant: normal; font-variant: small-caps; font-variant: none");
    assert!(
        compatibility.is_clean(),
        "{:?}",
        compatibility.diagnostics()
    );
    for (index, expected) in [CssFontVariant::Normal, CssFontVariant::SmallCaps]
        .iter()
        .enumerate()
    {
        let CssKnownPropertyValueRef::FontVariant(value) = compatibility.syntax()[index]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("expected font-variant");
        };
        assert_eq!(value.i01_subset(), Some(expected));
    }
    let CssKnownPropertyValueRef::FontVariant(none) = compatibility.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-variant none");
    };
    assert!(matches!(none.variant(), CssFontVariantValue::None));
    assert!(none.i01_subset().is_none());
}

#[test]
fn font_variant_globals_substitution_duplicates_and_conflicts_are_exact() {
    let symbolic = parse_style_attribute(concat!(
        "font-variant-caps: inherit; ",
        "font-variant-east-asian: var(--east); ",
        "font-variant-ligatures: unset; ",
        "font-variant-numeric: var(--numeric); ",
        "font-variant-position: revert-layer; ",
        "font-variant: initial",
    ));
    assert!(symbolic.is_clean(), "{:?}", symbolic.diagnostics());
    for (index, declaration) in symbolic.syntax().iter().enumerate() {
        assert!(
            if index == 1 || index == 3 {
                matches!(
                    declaration.known().unwrap().declared_value(),
                    CssKnownDeclaredValueRef::SubstitutionDependent(_)
                )
            } else {
                matches!(
                    declaration.known().unwrap().declared_value(),
                    CssKnownDeclaredValueRef::Global(_)
                )
            },
            "symbolic declaration {index}",
        );
    }

    for invalid in [
        "font-variant-caps: small-caps all-small-caps",
        "font-variant-caps: none",
        "font-variant-position: sub super",
        "font-variant-position: none",
        "font-variant-ligatures: common-ligatures no-common-ligatures",
        "font-variant-ligatures: discretionary-ligatures no-discretionary-ligatures",
        "font-variant-ligatures: historical-ligatures no-historical-ligatures",
        "font-variant-ligatures: contextual no-contextual",
        "font-variant-ligatures: contextual contextual",
        "font-variant-ligatures: normal common-ligatures",
        "font-variant-ligatures: none contextual",
        "font-variant-numeric: lining-nums oldstyle-nums",
        "font-variant-numeric: proportional-nums tabular-nums",
        "font-variant-numeric: diagonal-fractions stacked-fractions",
        "font-variant-numeric: ordinal ordinal",
        "font-variant-numeric: normal ordinal",
        "font-variant-numeric: none",
        "font-variant-east-asian: jis78 traditional",
        "font-variant-east-asian: full-width proportional-width",
        "font-variant-east-asian: ruby ruby",
        "font-variant-east-asian: normal ruby",
        "font-variant-east-asian: none",
        "font-variant: normal small-caps",
        "font-variant: none common-ligatures",
        "font-variant: small-caps petite-caps",
        "font-variant: sub super",
        "font-variant: common-ligatures no-common-ligatures",
        "font-variant: discretionary-ligatures no-discretionary-ligatures",
        "font-variant: historical-ligatures no-historical-ligatures",
        "font-variant: contextual no-contextual",
        "font-variant: lining-nums oldstyle-nums",
        "font-variant: proportional-nums tabular-nums",
        "font-variant: diagonal-fractions stacked-fractions",
        "font-variant: ordinal ordinal",
        "font-variant: jis78 jis90",
        "font-variant: full-width proportional-width",
        "font-variant: ruby ruby",
        "font-variant: small-caps, ordinal",
    ] {
        let source = format!("{invalid}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}",
        );
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
            "{source}",
        );

        #[cfg(feature = "app-strict")]
        assert_eq!(
            surgeist_css::validate_style_attribute(&source)
                .expect_err("strict parsing rejects the recovered declaration")
                .diagnostics(),
            report.diagnostics(),
            "{source}",
        );
    }
}

#[test]
fn kerning_size_adjust_and_synthesis_follow_fonts3() {
    let report = parse_style_attribute(concat!(
        "font-kerning: normal; ",
        "font-size-adjust: 0.5; ",
        "font-synthesis: style weight; ",
        "color: red",
    ));

    if !report.is_clean() {
        assert_eq!(report.diagnostics().len(), 3);
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|diagnostic| diagnostic.error().code() == CssErrorCode::UnknownProperty)
        );
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
        );
    }

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 4);
    assert_eq!(
        report
            .syntax()
            .iter()
            .map(|declaration| declaration.known().unwrap().property().canonical_name())
            .collect::<Vec<_>>(),
        [
            "font-kerning",
            "font-size-adjust",
            "font-synthesis",
            "color",
        ],
    );

    let CssKnownPropertyValueRef::FontKerning(kerning) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-kerning");
    };
    assert!(matches!(
        kerning.kerning(),
        surgeist_css::CssFontKerning::Normal
    ));
    assert_eq!(kerning.as_css(), "normal");

    let CssKnownPropertyValueRef::FontSizeAdjust(size_adjust) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-size-adjust");
    };
    assert!(matches!(
        size_adjust.size_adjust(),
        surgeist_css::CssFontSizeAdjust::Number(value) if value.value() == 0.5
    ));

    let CssKnownPropertyValueRef::FontSynthesis(synthesis) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-synthesis");
    };
    assert!(matches!(
        synthesis.synthesis(),
        CssFontSynthesis::Values(values) if values.weight() && values.style()
    ));
}

#[test]
fn font_control_branches_globals_substitution_and_mutations_are_exact() {
    let branches = parse_style_attribute(concat!(
        "font-kerning: auto; font-kerning: normal; font-kerning: none; ",
        "font-size-adjust: none; font-size-adjust: 0; ",
        "font-synthesis: none; font-synthesis: weight; font-synthesis: style; ",
        "font-synthesis: weight style; font-synthesis: style weight; ",
        "font-kerning: inherit; font-size-adjust: var(--ratio); font-synthesis: unset",
    ));
    assert!(branches.is_clean(), "{:?}", branches.diagnostics());
    assert_eq!(branches.syntax().len(), 13);
    assert!(matches!(
        branches.syntax()[10].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(_)
    ));
    assert!(matches!(
        branches.syntax()[11].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
    assert!(matches!(
        branches.syntax()[12].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(_)
    ));

    for invalid in [
        "font-kerning: optimizeSpeed",
        "font-kerning: normal none",
        "font-size-adjust: -0.01",
        "font-size-adjust: 1e999",
        "font-size-adjust: 1px",
        "font-size-adjust: calc(1)",
        "font-size-adjust: none 1",
        "font-synthesis: weight weight",
        "font-synthesis: style style",
        "font-synthesis: none weight",
        "font-synthesis: weight none",
        "font-synthesis: weightstyle",
        "font-synthesis: weight, style",
    ] {
        let source = format!("{invalid}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}",
        );
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
            "{source}",
        );

        #[cfg(feature = "app-strict")]
        assert_eq!(
            surgeist_css::validate_style_attribute(&source)
                .expect_err("strict parsing rejects the recovered declaration")
                .diagnostics(),
            report.diagnostics(),
            "{source}",
        );
    }
}

#[test]
fn font_size_family_line_height_and_shorthand_follow_fonts3() {
    let report = parse_style_attribute(concat!(
        "font-size: medium; ",
        "line-height: 1.25; ",
        "font-family: \"inherit\", Avenir Next, serif; ",
        "font: condensed 700 small-caps italic medium/1.2 \"Avenir Next\", sans-serif; ",
        "font: menu",
    ));

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 5);

    let CssKnownPropertyValueRef::FontSize(size) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-size");
    };
    assert!(matches!(size.size(), CssFontSize::Medium));
    assert!(size.i01_subset().is_none());

    let CssKnownPropertyValueRef::LineHeight(line_height) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected line-height");
    };
    assert!(matches!(
        line_height.line_height(),
        CssLineHeight::Number(CssNonNegativeNumberValue::Literal(value))
            if value.value() == 1.25
    ));
    assert!(line_height.i01_subset().is_none());

    let CssKnownPropertyValueRef::FontFamily(family) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-family");
    };
    let families = family.families().families();
    assert_eq!(families[0].kind(), CssFontFamilyNameKind::Quoted);
    assert_eq!(families[0].as_str(), "inherit");
    assert_eq!(families[1].kind(), CssFontFamilyNameKind::IdentSequence);
    assert_eq!(families[1].as_str(), "Avenir Next");
    assert_eq!(families[2].kind(), CssFontFamilyNameKind::Generic);
    assert_eq!(
        families[2].generic_family(),
        Some(CssGenericFontFamily::Serif)
    );
    assert_eq!(
        family.i01_subset().unwrap().families()[2].kind(),
        CssFontFamilyNameKind::IdentSequence,
    );

    let CssKnownPropertyValueRef::Font(explicit) = report.syntax()[3]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font shorthand");
    };
    let CssFontValue::Explicit(explicit) = explicit.font() else {
        panic!("expected explicit font shorthand");
    };
    assert!(matches!(explicit.size(), CssFontSize::Medium));
    assert!(matches!(
        explicit.line_height(),
        Some(CssLineHeight::Number(_))
    ));
    assert!(
        report.syntax()[3]
            .known()
            .unwrap()
            .property_value()
            .is_some()
    );
    assert!(matches!(
        report.syntax()[4]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
        CssKnownPropertyValueRef::Font(value)
            if matches!(value.font(), CssFontValue::System(CssSystemFont::Menu))
                && value.i01_subset().is_none()
    ));
}

#[test]
fn font_keywords_generics_globals_and_signed_boundaries_are_exact() {
    let source = concat!(
        "font-size: xx-small; font-size: x-small; font-size: small; ",
        "font-size: medium; font-size: large; font-size: x-large; ",
        "font-size: xx-large; font-size: larger; font-size: smaller; ",
        "font-size: +12px; line-height: +1.5; line-height: 120%; ",
        "font-family: serif, sans-serif, cursive, fantasy, monospace; ",
        "font-family: inherit; font-family: \"inherit\"",
    );
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 15);
    for (index, expected) in [
        CssFontSize::XxSmall,
        CssFontSize::XSmall,
        CssFontSize::Small,
        CssFontSize::Medium,
        CssFontSize::Large,
        CssFontSize::XLarge,
        CssFontSize::XxLarge,
        CssFontSize::Larger,
        CssFontSize::Smaller,
    ]
    .iter()
    .enumerate()
    {
        let CssKnownPropertyValueRef::FontSize(value) = report.syntax()[index]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("expected font-size keyword");
        };
        assert_eq!(value.size(), expected);
        assert!(value.i01_subset().is_none());
    }
    let CssKnownPropertyValueRef::FontFamily(generics) = report.syntax()[12]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected generic font-family list");
    };
    assert_eq!(
        generics
            .families()
            .families()
            .iter()
            .map(|family| family.generic_family().unwrap())
            .collect::<Vec<_>>(),
        vec![
            CssGenericFontFamily::Serif,
            CssGenericFontFamily::SansSerif,
            CssGenericFontFamily::Cursive,
            CssGenericFontFamily::Fantasy,
            CssGenericFontFamily::Monospace,
        ]
    );
    assert!(matches!(
        report.syntax()[13].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(_)
    ));

    let globals = parse_style_attribute(concat!(
        "font-size: initial; line-height: unset; ",
        "font-family: revert; font: inherit",
    ));
    assert!(globals.is_clean(), "{:?}", globals.diagnostics());
    assert!(globals.syntax().iter().all(|declaration| matches!(
        declaration.known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(_)
    )));
    assert!(matches!(
        report.syntax()[14]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
        CssKnownPropertyValueRef::FontFamily(value)
            if value.families().families()[0].kind() == CssFontFamilyNameKind::Quoted
    ));

    for invalid in [
        "font-size: -1px",
        "font-size: -1%",
        "line-height: -0.1",
        "line-height: -1px",
        "font-family: inherit, serif",
        "font-family: My serif",
    ] {
        let source = format!("{invalid}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
            "{source}",
        );
    }
}

#[test]
fn system_fonts_are_whole_values_and_explicit_components_are_unique() {
    for (keyword, expected) in [
        ("caption", CssSystemFont::Caption),
        ("icon", CssSystemFont::Icon),
        ("menu", CssSystemFont::Menu),
        ("message-box", CssSystemFont::MessageBox),
        ("small-caption", CssSystemFont::SmallCaption),
        ("status-bar", CssSystemFont::StatusBar),
    ] {
        let report = parse_style_attribute(&format!("font: {keyword}"));
        assert!(report.is_clean(), "{keyword}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::Font(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("expected font");
        };
        assert_eq!(value.font(), &CssFontValue::System(expected));
        assert!(value.i01_subset().is_none());
    }

    let legacy = parse_style_attribute("font: italic small-caps 700 condensed 16px/normal Arial");
    assert!(legacy.is_clean(), "{:?}", legacy.diagnostics());
    let CssKnownPropertyValueRef::Font(legacy) = legacy.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font");
    };
    assert!(legacy.i01_subset().is_some());

    for invalid in [
        "font: menu serif",
        "font: italic menu",
        "font: italic italic 16px serif",
        "font: small-caps small-caps 16px serif",
        "font: bold 700 16px serif",
        "font: condensed expanded 16px serif",
        "font: 16px/normal",
        "font: 16px serif/1.2",
    ] {
        let source = format!("{invalid}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
    }
}

#[test]
fn font_shorthand_retains_ambiguous_normal_before_explicit_style() {
    let report = parse_style_attribute(concat!(
        "font: normal italic 16px serif; ",
        "font: normal normal italic 16px serif; ",
        "font: normal normal normal italic 16px serif; ",
        "font: normal small-caps italic 16px serif; ",
        "font: normal 700 italic 16px serif; ",
        "font: normal condensed oblique 16px serif",
    ));

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 6);

    for (index, expected_style) in [
        CssFontStyle::Italic,
        CssFontStyle::Italic,
        CssFontStyle::Italic,
        CssFontStyle::Italic,
        CssFontStyle::Italic,
        CssFontStyle::Oblique,
    ]
    .iter()
    .enumerate()
    {
        let CssKnownPropertyValueRef::Font(value) = report.syntax()[index]
            .known()
            .expect("retained known font declaration")
            .property_value()
            .expect("retained current font value")
        else {
            panic!("expected font shorthand at declaration {index}");
        };
        let CssFontValue::Explicit(explicit) = value.font() else {
            panic!("expected explicit font shorthand at declaration {index}");
        };
        assert_eq!(explicit.style(), Some(*expected_style));
    }
}

#[test]
fn opentype_tags_and_indices_enforce_ascii_and_nonnegative_domains() {
    let report = parse_style_attribute(
        "font-feature-settings: \"éabc\"; font-feature-settings: \"kern\" -1; color: red",
    );

    assert_eq!(report.syntax().len(), 1);
    assert_eq!(report.diagnostics().len(), 2);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color,
    );
}

#[test]
fn font_feature_settings_preserve_authored_values_and_i01_projection() {
    let report = parse_style_attribute(
        r#"font-feature-settings: "kern", "\6c iga" on, "zero" off, "ss01" 0, "cv01" 12"#,
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::FontFeatureSettings(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-feature-settings");
    };
    let CssAuthoredFontFeatureSettings::Features(features) = value.settings() else {
        panic!("expected feature list");
    };
    assert_eq!(
        features
            .features()
            .iter()
            .map(|feature| feature.tag().as_str())
            .collect::<Vec<_>>(),
        ["kern", "liga", "zero", "ss01", "cv01"],
    );
    assert!(matches!(
        features.features()[0].value(),
        CssAuthoredFontFeatureValue::Omitted
    ));
    assert!(matches!(
        features.features()[1].value(),
        CssAuthoredFontFeatureValue::On
    ));
    assert!(matches!(
        features.features()[2].value(),
        CssAuthoredFontFeatureValue::Off
    ));
    assert!(matches!(
        features.features()[3].value(),
        CssAuthoredFontFeatureValue::Index(index) if index.value() == 0
    ));
    assert!(matches!(
        features.features()[4].value(),
        CssAuthoredFontFeatureValue::Index(index) if index.value() == 12
    ));

    let Some(CssFontFeatureSettings::Features(legacy)) = value.i01_subset() else {
        panic!("expected exact I01 projection");
    };
    assert_eq!(legacy.features()[0].value(), None);
    assert_eq!(legacy.features()[1].value(), Some(CssFontFeatureValue::On));
    assert_eq!(legacy.features()[2].value(), Some(CssFontFeatureValue::Off));
    assert_eq!(
        legacy.features()[3].value(),
        Some(CssFontFeatureValue::Integer(0))
    );
    assert_eq!(
        legacy.features()[4].value(),
        Some(CssFontFeatureValue::Integer(12))
    );
}

#[test]
fn opentype_tag_length_unicode_and_index_boundaries_recover() {
    for invalid in [
        r#"font-feature-settings: "abc""#,
        r#"font-feature-settings: "abcde""#,
        r#"font-feature-settings: "éabc""#,
        r#"font-feature-settings: "😀abc""#,
        r#"font-feature-settings: "\E9 abc""#,
        r#"font-feature-settings: "kern" -1"#,
    ] {
        let source = format!("{invalid}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
            "{source}",
        );
    }
}
