use surgeist_css::{
    CssErrorCode, CssGlobalKeyword, CssKnownDeclaredValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssRecoveryAction, CssTextCombineUpright, CssTextOrientation,
    CssUnicodeBidi, ErrorKind, parse_style_attribute,
};

#[test]
fn writing_modes_and_legacy_alias_are_typed() {
    let source = concat!(
        "text-combine-upright: all; ",
        "text-orientation: sideways; ",
        "unicode-bidi: isolate-override; ",
        "glyph-orientation-vertical: 90; ",
        "color: red",
    );
    let report = parse_style_attribute(source);

    if !report.is_clean() {
        assert_eq!(report.diagnostics().len(), 4);
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
    assert_eq!(report.syntax().len(), 5);
    assert_eq!(
        report
            .syntax()
            .iter()
            .map(|declaration| declaration.known().unwrap().property().canonical_name())
            .collect::<Vec<_>>(),
        [
            "text-combine-upright",
            "text-orientation",
            "unicode-bidi",
            "text-orientation",
            "color",
        ],
    );

    let CssKnownPropertyValueRef::TextCombineUpright(combine) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected text-combine-upright");
    };
    assert_eq!(combine.combine(), &CssTextCombineUpright::All);
    assert_eq!(combine.as_css(), "all");

    let CssKnownPropertyValueRef::TextOrientation(orientation) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected text-orientation");
    };
    assert_eq!(orientation.orientation(), &CssTextOrientation::Sideways);

    let CssKnownPropertyValueRef::UnicodeBidi(bidi) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected unicode-bidi");
    };
    assert_eq!(bidi.bidi(), &CssUnicodeBidi::IsolateOverride);

    let alias = report.syntax()[3].known().unwrap();
    assert_eq!(alias.property(), CssKnownProperty::TextOrientation);
    let CssKnownPropertyValueRef::TextOrientation(mapped) = alias.property_value().unwrap() else {
        panic!("legacy shorthand must map to text-orientation");
    };
    assert_eq!(mapped.orientation(), &CssTextOrientation::Sideways);
    assert_eq!(mapped.as_css(), "90");
    assert!(CssKnownProperty::TextOrientation.aliases().is_empty());
    assert_eq!(
        CssKnownProperty::from_name("glyph-orientation-vertical"),
        None,
        "the legacy shorthand is not a name-equivalent schema alias",
    );

    #[cfg(feature = "app-strict")]
    assert_eq!(
        surgeist_css::validate_style_attribute(source)
            .expect("strict validation accepts the complete Writing Modes values"),
        report.syntax().clone(),
    );
}

#[test]
fn writing_modes_keyword_and_legacy_number_domains_are_exact() {
    for (property, values) in [
        ("text-combine-upright", &["none", "all"][..]),
        ("text-orientation", &["mixed", "upright", "sideways"]),
        (
            "unicode-bidi",
            &[
                "normal",
                "embed",
                "isolate",
                "bidi-override",
                "isolate-override",
                "plaintext",
            ],
        ),
    ] {
        for value in values {
            let source = format!("{property}: {value}");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert_eq!(report.syntax().len(), 1, "{source}");
        }
    }

    for (value, expected) in [
        ("auto", CssTextOrientation::Mixed),
        ("0deg", CssTextOrientation::Upright),
        ("0", CssTextOrientation::Upright),
        ("90deg", CssTextOrientation::Sideways),
        ("90", CssTextOrientation::Sideways),
    ] {
        let source = format!("GLYPH-ORIENTATION-VERTICAL: {value}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let known = report.syntax()[0].known().unwrap();
        assert_eq!(known.property(), CssKnownProperty::TextOrientation);
        let CssKnownPropertyValueRef::TextOrientation(mapped) = known.property_value().unwrap()
        else {
            panic!("{source}: expected mapped text-orientation");
        };
        assert_eq!(mapped.orientation(), &expected, "{source}");
        assert_eq!(mapped.as_css(), value, "{source}");
    }
}

#[test]
fn writing_modes_globals_and_substitutions_remain_whole_property_branches() {
    const PROPERTIES: &[&str] = &[
        "text-combine-upright",
        "text-orientation",
        "unicode-bidi",
        "glyph-orientation-vertical",
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
            if *property == "glyph-orientation-vertical" {
                assert_eq!(
                    report.syntax()[0].known().unwrap().property(),
                    CssKnownProperty::TextOrientation,
                );
            }
        }

        let source = format!("{property}: var(--writing, initial)");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(matches!(
            report.syntax()[0].known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::SubstitutionDependent(value)
                if value.as_css() == "var(--writing, initial)"
        ));
    }
}

#[test]
fn writing_modes_invalid_values_drop_exact_declaration_and_keep_sibling() {
    for (property, value, expected_property) in [
        (
            "text-combine-upright",
            "digits 2",
            CssKnownProperty::TextCombineUpright,
        ),
        (
            "text-combine-upright",
            "none all",
            CssKnownProperty::TextCombineUpright,
        ),
        (
            "text-orientation",
            "sideways-right",
            CssKnownProperty::TextOrientation,
        ),
        (
            "text-orientation",
            "mixed upright",
            CssKnownProperty::TextOrientation,
        ),
        (
            "unicode-bidi",
            "isolate embed",
            CssKnownProperty::UnicodeBidi,
        ),
        ("unicode-bidi", "override", CssKnownProperty::UnicodeBidi),
        (
            "glyph-orientation-vertical",
            "45",
            CssKnownProperty::TextOrientation,
        ),
        (
            "glyph-orientation-vertical",
            "45deg",
            CssKnownProperty::TextOrientation,
        ),
        (
            "glyph-orientation-vertical",
            "180deg",
            CssKnownProperty::TextOrientation,
        ),
        (
            "glyph-orientation-vertical",
            "1.5708rad",
            CssKnownProperty::TextOrientation,
        ),
        (
            "glyph-orientation-vertical",
            "mixed",
            CssKnownProperty::TextOrientation,
        ),
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
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}",
        );
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
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected property-specific error");
        };
        assert_eq!(detail.property(), expected_property, "{source}");
        assert!(
            diagnostic.error().position().byte_offset().value() < invalid.len() + 1,
            "{source}: responsible position must be inside the dropped declaration",
        );

        #[cfg(feature = "app-strict")]
        assert_eq!(
            surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects recovered writing-mode syntax")
                .diagnostics(),
            report.diagnostics(),
            "{source}",
        );
    }
}

#[test]
fn writing_modes_eof_and_non_bmp_recovery_preserve_exact_coordinates() {
    let valid_eof = parse_style_attribute("glyph-orientation-vertical: 90deg");
    assert!(valid_eof.is_clean(), "{:?}", valid_eof.diagnostics());
    assert_eq!(valid_eof.syntax().len(), 1);

    let invalid_eof_source = "unicode-bidi: isolate embed";
    let invalid_eof = parse_style_attribute(invalid_eof_source);
    let [diagnostic] = invalid_eof.diagnostics() else {
        panic!("adjacent unicode-bidi keyword at EOF must recover exactly once");
    };
    assert!(invalid_eof.syntax().is_empty());
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        invalid_eof_source.len(),
    );

    let source = "--😀: 1; text-orientation: sideways upright; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("adjacent text-orientation keyword must recover exactly once");
    };
    let responsible = source.find("upright").unwrap();
    let declaration_start = source.find("text-orientation").unwrap();
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
            if detail.property() == CssKnownProperty::TextOrientation
    ));
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color,
    );
}
