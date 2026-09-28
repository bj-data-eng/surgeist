#![forbid(unsafe_code)]

use surgeist_css::*;

fn font(source: &str) -> CssFontValue {
    let report = parse_style_attribute(&format!("font:{source}"));
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Font(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("font property")
    };
    value.font().clone()
}

fn declaration(value: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("font:{value}!important"));
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

fn longhands(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("font longhands")
    };
    values
}

fn direct_longhand(name: &str, value: &str) -> CssLonghandValue {
    let report = parse_style_attribute(&format!("{name}:{value}"));
    assert!(
        report.is_clean(),
        "{name}:{value}: {:?}",
        report.diagnostics()
    );
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&report.syntax()[0]).unwrap()
    else {
        panic!("direct longhand: {name}")
    };
    assert_eq!(values.items().len(), 1);
    values.items()[0].ordinary_value().unwrap().clone()
}

#[test]
fn explicit_shorthand_serializes_in_grammar_order_and_preserves_family_boundaries() {
    let authored =
        font(r#"condensed small-caps bold oblique 25deg 16px/1.2 A\ B, A B, "serif", serif"#);
    let serialized = authored.serialize_specified().unwrap();
    assert_eq!(
        serialized,
        "oblique 25deg small-caps bold condensed 16px/1.2 A\\ B, A B, \"serif\", serif"
    );
    let CssFontValue::Explicit(reparsed) = font(&serialized) else {
        panic!("explicit font")
    };
    let CssFontValue::Explicit(original) = authored else {
        panic!("explicit source")
    };
    assert_eq!(reparsed, original);
    let families = reparsed.families().families();
    assert_eq!(families[0].identifier_tokens().unwrap(), &["A B"]);
    assert_eq!(families[1].identifier_tokens().unwrap(), &["A", "B"]);
    assert_eq!(families[2].kind(), CssFontFamilyNameKind::Quoted);
    assert_eq!(families[3].kind(), CssFontFamilyNameKind::Generic);
}

#[test]
fn system_keywords_and_programmatic_font_serialize_without_environment_lookup() {
    for keyword in [
        "caption",
        "icon",
        "menu",
        "message-box",
        "small-caption",
        "status-bar",
    ] {
        assert_eq!(font(keyword).serialize_specified().unwrap(), keyword);
    }
    let families = CssFontFamilyList::try_new(vec![
        CssFontFamilyName::try_ident_sequence(vec!["A B".into()]).unwrap(),
        CssFontFamilyName::generic(CssGenericFontFamily::Serif),
    ])
    .unwrap();
    let explicit =
        CssExplicitFont::try_new(None, None, None, None, CssFontSize::Medium, None, families)
            .unwrap();
    let css = explicit.serialize_specified().unwrap();
    assert_eq!(css, "medium A\\ B, serif");
    assert_eq!(font(&css), CssFontValue::Explicit(explicit));
}

#[test]
fn constructed_normal_components_have_the_same_expansion_as_their_canonical_form() {
    let constructed = CssExplicitFont::try_new(
        None,
        None,
        Some(CssFontWeight::Absolute(CssAbsoluteFontWeight::Normal)),
        None,
        CssFontSize::Medium,
        Some(CssLineHeight::Normal),
        CssFontFamilyList::try_new(vec![CssFontFamilyName::generic(
            CssGenericFontFamily::Serif,
        )])
        .unwrap(),
    )
    .unwrap();
    assert!(constructed.style().is_none());
    assert!(constructed.variant().is_none());
    assert_eq!(
        constructed.weight(),
        Some(&CssFontWeight::Absolute(CssAbsoluteFontWeight::Normal))
    );
    assert!(constructed.stretch().is_none());
    assert_eq!(constructed.line_height(), Some(&CssLineHeight::Normal));
    let css = constructed.serialize_specified().unwrap();
    assert_eq!(css, "medium serif");

    // Expansion accepts declarations, so serialization is the public bridge
    // from a constructed shorthand value back into that boundary.
    let normalized = longhands(&declaration(&css));
    for (item, (name, expected)) in normalized.items()[..7].iter().zip([
        ("font-family", "serif"),
        ("font-size", "medium"),
        ("font-width", "normal"),
        ("font-style", "normal"),
        ("font-variant-caps", "normal"),
        ("font-weight", "normal"),
        ("line-height", "normal"),
    ]) {
        assert_eq!(item.property().canonical_name(), name);
        assert_eq!(
            item.ordinary_value(),
            Some(&direct_longhand(name, expected))
        );
    }

    let authored = declaration("normal normal normal normal medium/normal serif");
    let original = longhands(&authored);
    assert_eq!(original.items().len(), 19);
    assert_eq!(normalized.items().len(), 19);
    for (left, right) in original.items().iter().zip(normalized.items()) {
        assert_eq!(left.property(), right.property());
        assert_eq!(left.value(), right.value());
    }
}

#[test]
fn six_system_fonts_keep_only_settable_members_symbolic() {
    for keyword in [
        "caption",
        "icon",
        "menu",
        "message-box",
        "small-caption",
        "status-bar",
    ] {
        let source = declaration(keyword);
        let CssKnownPropertyValueRef::Font(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("font value")
        };
        let CssFontValue::System(system) = wrapper.font() else {
            panic!("system font")
        };
        let values = longhands(&source);
        assert_eq!(values.items().len(), 19);
        for item in &values.items()[..7] {
            assert_eq!(item.value(), CssContributionValueRef::SystemFont(*system));
            assert!(item.ordinary_value().is_none());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
        }
        for (item, (name, initial)) in values.items()[7..].iter().zip([
            ("font-feature-settings", "normal"),
            ("font-kerning", "auto"),
            ("font-language-override", "normal"),
            ("font-optical-sizing", "auto"),
            ("font-size-adjust", "none"),
            ("font-variant-alternates", "normal"),
            ("font-variant-east-asian", "normal"),
            ("font-variant-emoji", "normal"),
            ("font-variant-ligatures", "normal"),
            ("font-variant-numeric", "normal"),
            ("font-variant-position", "normal"),
            ("font-variation-settings", "normal"),
        ]) {
            assert_eq!(item.property().canonical_name(), name);
            assert_eq!(item.ordinary_value(), Some(&direct_longhand(name, initial)));
            assert!(item.source().same_occurrence(&source));
        }
    }
}

#[test]
fn globals_and_pending_system_reentry_preserve_all_members_and_provenance() {
    for (keyword, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(keyword);
        let values = longhands(&source);
        assert_eq!(values.items().len(), 19);
        assert!(values.items().iter().all(|item| item.value()
            == CssContributionValueRef::Global(expected)
            && item.source().same_occurrence(&source)));
    }
    let source = declaration("env(font)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending system font")
    };
    let replacement = parse_component_values("menu").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("reentered system font")
    };
    assert_eq!(values.items().len(), 19);
    for item in values.items() {
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
    assert!(
        values.items()[..7]
            .iter()
            .all(|item| item.value() == CssContributionValueRef::SystemFont(CssSystemFont::Menu))
    );
}

#[test]
fn shorthand_uses_one_cumulative_budget_for_all_components() {
    let authored = font("italic bold 16px/1.2 serif");
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 100, 100),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(100, 2, 100),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(100, 100, 9),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            authored
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(
        authored.serialize_specified().unwrap(),
        "italic bold 16px/1.2 serif"
    );

    let normal_components = CssExplicitFont::try_new(
        Some(CssFontStyle::Keyword(CssFontStyleKeyword::Normal)),
        Some(CssFontVariant::Normal),
        Some(CssFontWeight::Absolute(CssAbsoluteFontWeight::Normal)),
        Some(CssFontStretch::Normal),
        CssFontSize::Medium,
        Some(CssLineHeight::Normal),
        CssFontFamilyList::try_new(vec![CssFontFamilyName::generic(
            CssGenericFontFamily::Serif,
        )])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        normal_components.serialize_specified().unwrap(),
        "medium serif"
    );
    assert_eq!(
        normal_components
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(6, 100, 100))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
}

#[test]
fn mixed_symbolic_math_remains_parseable_after_canonical_serialization() {
    let authored = font("oblique calc(10deg + 5deg) calc(700) calc(14px) / calc(1.2) serif");
    let serialized = authored.serialize_specified().unwrap();
    assert_eq!(
        serialized,
        "oblique calc(15deg) calc(700) calc(14px)/calc(1.2) serif"
    );
    let reparsed = font(&serialized);
    assert_eq!(reparsed.serialize_specified().unwrap(), serialized);
}
