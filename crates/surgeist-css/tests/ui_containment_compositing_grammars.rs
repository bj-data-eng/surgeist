use surgeist_css::{
    CssBlendMode, CssBlendModeList, CssBoxEdgeKeyword, CssCaretColor, CssContain,
    CssContainComponent, CssContainComponentList, CssErrorCode, CssGlobalKeyword, CssIsolation,
    CssKnownDeclaredValueRef, CssKnownProperty, CssKnownPropertyValueRef, CssLength,
    CssOutlineOffset, CssRecoveryAction, CssResize, CssTransformBox, ErrorKind,
    parse_style_attribute,
};

#[test]
fn residual_ui_containment_and_compositing_properties_are_typed() {
    let source = concat!(
        "caret-color: rebeccapurple; ",
        "outline-offset: -2px; ",
        "resize: horizontal; ",
        "contain: paint size; ",
        "transform-box: view-box; ",
        "background-blend-mode: multiply, luminosity; ",
        "isolation: isolate; ",
        "mix-blend-mode: soft-light; ",
        "color: red",
    );
    let report = parse_style_attribute(source);

    if !report.is_clean() {
        assert_eq!(report.diagnostics().len(), 8);
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
    assert_eq!(report.syntax().len(), 9);
    assert_eq!(
        report
            .syntax()
            .iter()
            .map(|declaration| declaration.known().unwrap().property().canonical_name())
            .collect::<Vec<_>>(),
        [
            "caret-color",
            "outline-offset",
            "resize",
            "contain",
            "transform-box",
            "background-blend-mode",
            "isolation",
            "mix-blend-mode",
            "color",
        ],
    );

    let CssKnownPropertyValueRef::CaretColor(caret) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected caret-color");
    };
    let CssCaretColor::Color(color) = caret.caret() else {
        panic!("expected authored caret color");
    };
    assert_eq!(color.named().unwrap().name(), "rebeccapurple");
    assert_eq!(caret.as_css(), "rebeccapurple");

    let CssKnownPropertyValueRef::OutlineOffset(offset) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected outline-offset");
    };
    assert!(matches!(
        offset.offset().value(),
        CssLength::Px(value) if value.value() == -2.0
    ));

    let CssKnownPropertyValueRef::Resize(resize) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected resize");
    };
    assert_eq!(resize.resize(), &CssResize::Horizontal);

    let CssKnownPropertyValueRef::Contain(contain) = report.syntax()[3]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected contain");
    };
    let CssContain::Components(components) = contain.containment() else {
        panic!("expected authored contain components");
    };
    assert_eq!(
        components.components(),
        &[CssContainComponent::Paint, CssContainComponent::Size],
    );

    let CssKnownPropertyValueRef::TransformBox(transform_box) = report.syntax()[4]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transform-box");
    };
    assert_eq!(
        transform_box.reference_box().edge(),
        CssBoxEdgeKeyword::ViewBox,
    );

    let CssKnownPropertyValueRef::BackgroundBlendMode(background) = report.syntax()[5]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected background-blend-mode");
    };
    assert_eq!(
        background.modes().modes(),
        &[CssBlendMode::Multiply, CssBlendMode::Luminosity],
    );

    let CssKnownPropertyValueRef::Isolation(isolation) = report.syntax()[6]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected isolation");
    };
    assert_eq!(isolation.isolation(), &CssIsolation::Isolate);

    let CssKnownPropertyValueRef::MixBlendMode(mix) = report.syntax()[7]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected mix-blend-mode");
    };
    assert_eq!(mix.mode(), &CssBlendMode::SoftLight);

    #[cfg(feature = "app-strict")]
    assert_eq!(
        surgeist_css::validate_style_attribute(source)
            .expect("strict validation accepts selected UI and compositing grammars"),
        report.syntax().clone(),
    );
}

#[test]
fn ui_containment_transform_and_isolation_keyword_domains_are_exact() {
    for source in [
        "caret-color: auto",
        "caret-color: color(display-p3 1 0 0)",
        "outline-offset: 0",
        "outline-offset: -3em",
        "outline-offset: calc(-1em + 2px)",
        "resize: none",
        "resize: both",
        "resize: horizontal",
        "resize: vertical",
        "contain: none",
        "contain: strict",
        "contain: content",
        "transform-box: content-box",
        "transform-box: border-box",
        "transform-box: fill-box",
        "transform-box: stroke-box",
        "transform-box: view-box",
        "isolation: auto",
        "isolation: isolate",
    ] {
        let report = parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{source}");
    }

    for source in [
        "contain: size",
        "contain: layout",
        "contain: paint",
        "contain: size layout",
        "contain: layout size",
        "contain: size paint",
        "contain: paint size",
        "contain: layout paint",
        "contain: paint layout",
        "contain: size layout paint",
        "contain: size paint layout",
        "contain: layout size paint",
        "contain: layout paint size",
        "contain: paint size layout",
        "contain: paint layout size",
    ] {
        let report = parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{source}");
    }
}

#[test]
fn compositing_blend_mode_domain_and_background_order_are_exact() {
    let expected = [
        ("normal", CssBlendMode::Normal),
        ("darken", CssBlendMode::Darken),
        ("multiply", CssBlendMode::Multiply),
        ("color-burn", CssBlendMode::ColorBurn),
        ("lighten", CssBlendMode::Lighten),
        ("screen", CssBlendMode::Screen),
        ("color-dodge", CssBlendMode::ColorDodge),
        ("overlay", CssBlendMode::Overlay),
        ("soft-light", CssBlendMode::SoftLight),
        ("hard-light", CssBlendMode::HardLight),
        ("difference", CssBlendMode::Difference),
        ("exclusion", CssBlendMode::Exclusion),
        ("hue", CssBlendMode::Hue),
        ("saturation", CssBlendMode::Saturation),
        ("color", CssBlendMode::Color),
        ("luminosity", CssBlendMode::Luminosity),
    ];

    for (authored, mode) in expected {
        assert_eq!(CssBlendMode::from_keyword(authored), Some(mode));
        assert_eq!(
            CssBlendMode::from_keyword(&authored.to_ascii_uppercase()),
            Some(mode),
        );
        assert_eq!(mode.as_css_str(), authored);

        for property in ["mix-blend-mode", "background-blend-mode"] {
            let source = format!("{property}: {authored}");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert_eq!(report.syntax().len(), 1, "{source}");
        }
    }

    let source = "background-blend-mode: hue, normal, multiply, hue";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BackgroundBlendMode(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected background blend-mode list");
    };
    assert_eq!(
        value.modes().modes(),
        &[
            CssBlendMode::Hue,
            CssBlendMode::Normal,
            CssBlendMode::Multiply,
            CssBlendMode::Hue,
        ],
    );

    assert_eq!(CssBlendMode::from_keyword("plus-lighter"), None);
    assert_eq!(CssBlendMode::from_keyword("normal multiply"), None);
    assert_eq!(CssBlendMode::from_keyword(""), None);
}

#[test]
fn checked_public_containment_transform_and_list_construction_rejects_invalid_states() {
    assert!(CssContainComponentList::try_new(Vec::new()).is_none());
    assert!(
        CssContainComponentList::try_new(vec![
            CssContainComponent::Size,
            CssContainComponent::Size,
        ])
        .is_none()
    );
    assert_eq!(
        CssContainComponentList::try_new(vec![
            CssContainComponent::Paint,
            CssContainComponent::Layout,
        ])
        .unwrap()
        .components(),
        &[CssContainComponent::Paint, CssContainComponent::Layout],
    );

    assert_eq!(
        CssTransformBox::try_new(CssBoxEdgeKeyword::PaddingBox),
        None,
    );
    assert_eq!(CssTransformBox::try_new(CssBoxEdgeKeyword::MarginBox), None,);
    assert_eq!(
        CssTransformBox::try_new(CssBoxEdgeKeyword::FillBox)
            .unwrap()
            .edge(),
        CssBoxEdgeKeyword::FillBox,
    );

    assert!(CssBlendModeList::try_new(Vec::new()).is_none());
    assert_eq!(
        CssBlendModeList::try_new(vec![CssBlendMode::Screen, CssBlendMode::Screen])
            .unwrap()
            .modes(),
        &[CssBlendMode::Screen, CssBlendMode::Screen],
    );

    assert!(CssOutlineOffset::try_new(CssLength::Auto).is_none());
    assert!(CssOutlineOffset::try_new(CssLength::try_percent(10.0).unwrap()).is_none());
    assert!(CssOutlineOffset::try_new(CssLength::try_px(-2.0).unwrap()).is_some());
}

#[test]
fn ui_containment_and_compositing_globals_and_substitutions_are_whole_property_branches() {
    const PROPERTIES: &[&str] = &[
        "caret-color",
        "outline-offset",
        "resize",
        "contain",
        "transform-box",
        "background-blend-mode",
        "isolation",
        "mix-blend-mode",
    ];
    const GLOBALS: &[(&str, CssGlobalKeyword)] = &[
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ];

    for property in PROPERTIES {
        for (keyword, expected) in GLOBALS {
            let source = format!("{property}: {keyword}");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert!(matches!(
                report.syntax()[0].known().unwrap().declared_value(),
                CssKnownDeclaredValueRef::Global(actual) if actual == *expected
            ));
        }

        let source = format!("{property}: var(--ui, initial)");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(matches!(
            report.syntax()[0].known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::SubstitutionDependent(value)
                if value.as_css() == "var(--ui, initial)"
        ));
    }
}

#[test]
fn ui_containment_and_compositing_invalid_values_drop_only_their_declaration() {
    for (property, value) in [
        ("caret-color", "auto red"),
        ("caret-color", "not-a-color"),
        ("outline-offset", "10%"),
        ("outline-offset", "auto"),
        ("outline-offset", "1px 2px"),
        ("resize", "block"),
        ("resize", "horizontal vertical"),
        ("contain", "none size"),
        ("contain", "strict paint"),
        ("contain", "size size"),
        ("contain", "size, paint"),
        ("contain", "inline-size"),
        ("transform-box", "padding-box"),
        ("transform-box", "margin-box"),
        ("transform-box", "view-box border-box"),
        ("background-blend-mode", "normal multiply"),
        ("background-blend-mode", "normal,"),
        ("background-blend-mode", "normal,, multiply"),
        ("background-blend-mode", "plus-lighter"),
        ("isolation", "none"),
        ("isolation", "auto isolate"),
        ("mix-blend-mode", "normal, multiply"),
        ("mix-blend-mode", "normal multiply"),
        ("mix-blend-mode", "plus-lighter"),
    ] {
        let invalid = format!("{property}: {value}");
        let source = format!("{invalid}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
            "{source}",
        );
        let diagnostic = &report.diagnostics()[0];
        let expected_code = if property == "caret-color" && value == "not-a-color" {
            CssErrorCode::InvalidColorSyntax
        } else {
            CssErrorCode::InvalidPropertyValue
        };
        assert_eq!(diagnostic.error().code(), expected_code, "{source}",);
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropDeclaration,
            "{source}",
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            0,
            "{source}"
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            invalid.len() + 1,
            "{source}",
        );
        if expected_code == CssErrorCode::InvalidPropertyValue {
            let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
                panic!("{source}: expected property-specific error");
            };
            assert_eq!(detail.property().canonical_name(), property, "{source}");
        } else {
            assert!(matches!(
                diagnostic.error().kind(),
                ErrorKind::InvalidColorSyntax(_)
            ));
        }
        assert!(
            diagnostic.error().position().byte_offset().value() < invalid.len() + 1,
            "{source}: responsible position must be inside the dropped declaration",
        );

        #[cfg(feature = "app-strict")]
        assert_eq!(
            surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects recovered T3 syntax")
                .diagnostics(),
            report.diagnostics(),
            "{source}",
        );
    }
}

#[test]
fn ui_containment_and_compositing_eof_and_non_bmp_recovery_preserve_coordinates() {
    let clean_eof = parse_style_attribute("background-blend-mode: normal, luminosity");
    assert!(clean_eof.is_clean(), "{:?}", clean_eof.diagnostics());
    assert_eq!(clean_eof.syntax().len(), 1);

    let invalid_eof_source = "background-blend-mode: normal,";
    let invalid_eof = parse_style_attribute(invalid_eof_source);
    let [diagnostic] = invalid_eof.diagnostics() else {
        panic!("trailing blend-mode comma at EOF must recover exactly once");
    };
    assert!(invalid_eof.syntax().is_empty());
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue,
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        invalid_eof_source.len(),
    );

    let source = "--😀: 1; background-blend-mode: normal multiply; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("missing blend-mode comma must recover exactly once");
    };
    let responsible = source.find("multiply").unwrap();
    let declaration_start = source.find("background-blend-mode").unwrap();
    let declaration_end = declaration_start + source[declaration_start..].find(';').unwrap() + 1;
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible,
    );
    assert_eq!(
        diagnostic.error().position().column().value() as usize,
        source[..responsible].encode_utf16().count(),
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        declaration_start,
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        declaration_end,
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert!(matches!(
        diagnostic.error().kind(),
        ErrorKind::InvalidPropertyValue(detail)
            if detail.property() == CssKnownProperty::BackgroundBlendMode
    ));
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color,
    );
}
