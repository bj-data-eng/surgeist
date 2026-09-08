use surgeist_css::{
    CssAnimationName, CssContent, CssContentItem, CssErrorCode, CssFontFamilyNameKind,
    CssGlobalKeyword, CssImageValue, CssKnownDeclaredValueRef, CssKnownPropertyValueRef,
    CssNthPattern, CssPseudoClass, CssRecoveryAction, CssRule, CssSelector, CssSupportStatus,
    CssSupportsConditionKind, CssUrlModifier, feature_metadata, parse_sheet, parse_style_attribute,
};

#[test]
fn c14_remaining_shared_values_are_typed() {
    let report = parse_style_attribute(
        "background-image: url(\"theme.css\" integrity(sha256) cors); width: 2px",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);

    let CssKnownPropertyValueRef::BackgroundImage(background) = report.syntax()[0]
        .known()
        .expect("known background-image")
        .property_value()
        .expect("ordinary background-image")
    else {
        panic!("expected background-image");
    };
    assert!(matches!(
        background.images().images(),
        [CssImageValue::Url(url)]
            if url.as_str() == "theme.css"
                && matches!(
                    url.modifiers(),
                    [CssUrlModifier::Function(function), CssUrlModifier::Ident(ident)]
                        if function.name() == "integrity"
                            && function.arguments().as_css() == "sha256"
                            && ident.as_str() == "cors"
                )
    ));

    let components = parse_style_attribute(
        r#"--stream: token fn([x]) {y} "z"; width: 2px; --bad: one ! two; height: 3px"#,
    );
    assert_eq!(components.syntax().len(), 3);
    assert_eq!(
        components.syntax()[0]
            .custom()
            .expect("custom token-stream declaration")
            .value()
            .value()
            .expect("authored declaration-value")
            .as_css(),
        r#"token fn([x]) {y} "z""#,
    );
    let [declaration_value_recovery] = components.diagnostics() else {
        panic!("expected one declaration-value rejection");
    };
    assert_eq!(
        declaration_value_recovery.error().code(),
        CssErrorCode::InvalidDeclarationAnnotation
    );
    assert_eq!(
        declaration_value_recovery.action(),
        CssRecoveryAction::DropDeclaration
    );

    let any_value = parse_sheet("@supports future(fn({x; !}) [y]) { .x { color: red; } }");
    assert!(any_value.is_clean(), "{:?}", any_value.diagnostics());
    let [CssRule::Supports(supports)] = any_value.syntax().rules() else {
        panic!("expected supports rule");
    };
    assert!(matches!(
        supports.condition().kind(),
        CssSupportsConditionKind::GeneralEnclosed(value)
            if value.authored() == "future(fn({x; !}) [y])"
    ));

    let nth = parse_sheet(".item:nth-child(-2n + 3) { color: red; }");
    assert!(nth.is_clean(), "{:?}", nth.diagnostics());
    let [CssRule::Style(rule)] = nth.syntax().rules() else {
        panic!("expected nth-child style rule");
    };
    let CssSelector::Compound(selector) = rule.selector() else {
        panic!("expected compound selector");
    };
    assert!(matches!(
        selector.pseudo_classes(),
        [CssPseudoClass::NthChild(pattern)]
            if matches!(
                pattern.pattern(),
                CssNthPattern::AnPlusB(value) if (value.a(), value.b()) == (-2, 3)
            )
    ));
    let invalid_nth = parse_sheet(".bad:nth-child(2n+-3) { color: red; } .after { width: 2px; }");
    assert!(matches!(invalid_nth.syntax().rules(), [CssRule::Style(_)]));
    assert_eq!(
        invalid_nth.diagnostics()[0].error().code(),
        CssErrorCode::InvalidSelector
    );

    let unicode = parse_sheet(concat!(
        "@font-face { font-family: Demo; src: url(demo.woff2); ",
        "unicode-range: U+4??, U+10FFFF; }",
    ));
    assert!(unicode.is_clean(), "{:?}", unicode.diagnostics());
    let [CssRule::FontFace(font_face)] = unicode.syntax().rules() else {
        panic!("expected font-face rule");
    };
    let ranges = font_face
        .descriptors()
        .unicode_range()
        .expect("unicode-range descriptor")
        .value()
        .ranges();
    assert_eq!((ranges[0].start(), ranges[0].end()), (0x400, 0x4ff));
    assert_eq!((ranges[1].start(), ranges[1].end()), (0x10ffff, 0x10ffff));

    let values = parse_style_attribute(
        r#"width: ReVeRt-LaYeR; animation-name: Main; font-family: "inherit", Avenir\ Next; content: "line\a break""#,
    );
    assert!(values.is_clean(), "{:?}", values.diagnostics());
    assert!(matches!(
        values.syntax()[0]
            .known()
            .expect("known width")
            .declared_value(),
        CssKnownDeclaredValueRef::Global(CssGlobalKeyword::RevertLayer)
    ));
    let CssKnownPropertyValueRef::AnimationName(names) = values.syntax()[1]
        .known()
        .expect("known animation-name")
        .property_value()
        .expect("ordinary animation-name")
    else {
        panic!("expected animation-name");
    };
    assert!(matches!(
        names.i01_subset().expect("I01 animation names").names(),
        [CssAnimationName::Custom(name)] if name.as_str() == "Main"
    ));
    let CssKnownPropertyValueRef::FontFamily(families) = values.syntax()[2]
        .known()
        .expect("known font-family")
        .property_value()
        .expect("ordinary font-family")
    else {
        panic!("expected font-family");
    };
    let families = families.i01_subset().expect("I01 font families").families();
    assert_eq!(families[0].kind(), CssFontFamilyNameKind::Quoted);
    assert_eq!(families[0].as_str(), "inherit");
    assert_eq!(families[1].kind(), CssFontFamilyNameKind::IdentSequence);
    assert_eq!(families[1].as_str(), "Avenir Next");
    let CssKnownPropertyValueRef::Content(content) = values.syntax()[3]
        .known()
        .expect("known content")
        .property_value()
        .expect("ordinary content")
    else {
        panic!("expected content");
    };
    assert!(matches!(
        content.i01_subset(),
        Some(CssContent::Items(items))
            if matches!(items.items(), [CssContentItem::String(value)] if value.as_str() == "line\nbreak")
    ));

    let rejected_url = parse_style_attribute("background-image: url(theme.css cors); width: 2px");
    assert_eq!(rejected_url.syntax().len(), 1);
    assert_eq!(
        rejected_url.diagnostics()[0].error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(
        rejected_url.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert_eq!(
        rejected_url.diagnostics()[0]
            .span()
            .start()
            .byte_offset()
            .value(),
        0
    );
    assert_eq!(
        rejected_url.diagnostics()[0]
            .span()
            .end()
            .byte_offset()
            .value(),
        "background-image: url(theme.css cors);".len()
    );

    for (id, source, production) in [
        (
            "official.value.syntax-token-stream",
            "O-SYNTAX3",
            "#tokenization",
        ),
        (
            "official.value.component-value",
            "O-SYNTAX3",
            "#consume-component-value",
        ),
        (
            "official.value.simple-block",
            "O-SYNTAX3",
            "#consume-simple-block",
        ),
        ("official.value.function", "O-SYNTAX3", "#consume-function"),
        (
            "official.value.declaration-value",
            "O-SYNTAX3",
            "#any-value",
        ),
        ("official.value.any-value", "O-SYNTAX3", "#any-value"),
        ("official.value.an-plus-b", "O-SYNTAX3", "#the-anb-type"),
        (
            "official.value.unicode-range",
            "O-SYNTAX3",
            "#urange-syntax",
        ),
        (
            "official.value.css-wide-keyword",
            "O-CASCADE4",
            "#defaulting-keywords",
        ),
        ("official.value.custom-ident", "O-VALUES3", "#custom-idents"),
        ("official.value.ident", "O-VALUES3", "#custom-idents"),
        ("official.value.string", "O-VALUES3", "#strings"),
        ("official.value.url", "O-VALUES3", "#urls"),
        ("official.value.url-modifier", "O-VALUES3", "#url-modifiers"),
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
        assert_eq!(metadata.source().id().as_str(), source, "{id}");
        assert_eq!(metadata.production(), production, "{id}");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id}");
        assert_eq!(metadata.supported_subset(), None, "{id}");
        assert_eq!(metadata.unsupported_remainder(), None, "{id}");
    }
}
