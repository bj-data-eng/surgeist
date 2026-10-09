#![forbid(unsafe_code)]
//! CSSOM WD 2021-08-26 §2.1 specifies quoted local() and comma-space lists.
//! Fonts 4 WD 2026-09-07 §4.3.1 owns the concrete source/hint grammar.
//! https://www.w3.org/TR/2021/WD-cssom-1-20210826/#common-serializing-idioms
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-parsing
//! URL emission remains authored and context-free; CSSOM's URL phase question
//! and resource support do not change the stored model or select this output.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn parsed(text: &str) -> CssFontFaceSourceList {
    let report = parse_font_face_descriptor_value(text, CssFontFaceDescriptorKind::Src);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let Some(CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::Src(list))) =
        report.syntax()
    else {
        panic!("ordinary source list: {text}")
    };
    list.clone()
}

fn assert_output(value: &CssFontFaceSourceList, expected: &str) {
    let before = value.clone();
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(value, &before);
    let reparsed = parsed(expected);
    assert_eq!(reparsed, before);
    assert_eq!(reparsed.serialize_specified().unwrap(), expected);
}

#[test]
fn local_names_always_use_the_quoted_string_wrapper() {
    for (text, decoded, expected) in [
        (
            "local(Gentium Bold)",
            "Gentium Bold",
            "local(\"Gentium Bold\")",
        ),
        ("LOCAL('serif')", "serif", "local(\"serif\")"),
        ("local('')", "", "local(\"\")"),
        ("local('  ')", "  ", "local(\"  \")"),
        ("local(\\31 Font)", "1Font", "local(\"1Font\")"),
        ("local(é😀)", "é😀", "local(\"é😀\")"),
    ] {
        let list = parsed(text);
        assert_output(&list, expected);
        let CssFontFaceSource::Local(value) = &list.sources()[0] else {
            panic!("local source")
        };
        assert_eq!(value.as_str(), decoded);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let constructed = CssFontLocalName::try_new(decoded).unwrap();
        assert_eq!(constructed.serialize_specified().unwrap(), expected);
        assert_eq!(
            CssFontFaceSource::Local(constructed)
                .serialize_specified()
                .unwrap(),
            expected
        );
    }
    assert!(CssFontLocalName::try_new("a\0b").is_none());
    assert!(CssFontFaceSourceList::try_new(Vec::new()).is_none());
    for invalid in ["local(serif)", "local(inherit)", "local()"] {
        assert!(
            parse_font_face_descriptor_value(invalid, CssFontFaceDescriptorKind::Src)
                .syntax()
                .is_none()
        );
    }
}

#[test]
fn local_string_escaping_retains_decoded_case_controls_and_unicode() {
    let name = "é😀\"\\\n\u{1}\u{7f}'";
    let expected = "local(\"é😀\\\"\\\\\\a \\1 \\7f '\")";
    let value = CssFontLocalName::try_new(name).unwrap();
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(value.as_str(), name);
    assert_output(
        &CssFontFaceSourceList::try_new(vec![CssFontFaceSource::Local(value)]).unwrap(),
        expected,
    );
}

#[test]
fn every_format_keyword_uses_its_concrete_lowercase_component() {
    for (format, spelling) in [
        (CssFontFormatHint::Woff, "woff"),
        (CssFontFormatHint::Woff2, "woff2"),
        (CssFontFormatHint::TrueType, "truetype"),
        (CssFontFormatHint::OpenType, "opentype"),
        (CssFontFormatHint::Collection, "collection"),
        (CssFontFormatHint::EmbeddedOpenType, "embedded-opentype"),
        (CssFontFormatHint::Svg, "svg"),
    ] {
        let value = CssFontFaceUrlSource::new(
            CssUrl::new("face"),
            Some(CssFontFormat::Keyword(format)),
            Vec::new(),
        );
        let expected = format!("url(\"face\") format({spelling})");
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value, before);
        let list = CssFontFaceSourceList::try_new(vec![CssFontFaceSource::Url(value)]).unwrap();
        assert_output(&list, &expected);
        assert_eq!(
            parsed(&format!(
                "URL(face) FORMAT({})",
                spelling.to_ascii_uppercase()
            )),
            list
        );
    }
}

#[test]
fn format_strings_preserve_empty_unknown_case_and_legacy_authored_identity() {
    for (text, expected) in [
        ("", "url(\"face\") format(\"\")"),
        ("WOFF2", "url(\"face\") format(\"WOFF2\")"),
        ("zebra", "url(\"face\") format(\"zebra\")"),
        (
            "woff2-variations",
            "url(\"face\") format(\"woff2-variations\")",
        ),
        ("é\"\\\n'", "url(\"face\") format(\"é\\\"\\\\\\a '\")"),
    ] {
        let value = CssFontFaceUrlSource::new(
            CssUrl::new("face"),
            Some(CssFontFormat::String(CssFontFormatString::new(text))),
            Vec::new(),
        );
        let required = value.required_technologies().collect::<Vec<_>>();
        assert_output(
            &CssFontFaceSourceList::try_new(vec![CssFontFaceSource::Url(value.clone())]).unwrap(),
            expected,
        );
        assert_eq!(value.required_technologies().collect::<Vec<_>>(), required);
        assert!(value.tech().is_empty());
        let Some(CssFontFormat::String(format)) = value.format() else {
            panic!("string production")
        };
        assert_eq!(format.as_str(), text);
    }
    // Common CSSOM string replacement applies even to this infallible string owner.
    let value = CssFontFaceUrlSource::new(
        CssUrl::new("face"),
        Some(CssFontFormat::String(CssFontFormatString::new("a\0b"))),
        Vec::new(),
    );
    assert_eq!(
        value.serialize_specified().unwrap(),
        "url(\"face\") format(\"a�b\")"
    );
    let Some(CssFontFormat::String(format)) = value.format() else {
        panic!("string production")
    };
    assert_eq!(format.as_str(), "a\0b");
}

#[test]
fn technology_keywords_keep_authored_order_and_repetitions() {
    let technology = vec![
        CssFontTechHint::Variations,
        CssFontTechHint::Palettes,
        CssFontTechHint::ColorCOLRv0,
        CssFontTechHint::ColorCOLRv1,
        CssFontTechHint::ColorSVG,
        CssFontTechHint::ColorSbix,
        CssFontTechHint::ColorCBDT,
        CssFontTechHint::FeaturesOpenType,
        CssFontTechHint::FeaturesAAT,
        CssFontTechHint::FeaturesGraphite,
        CssFontTechHint::Incremental,
        CssFontTechHint::Palettes,
    ];
    let value = CssFontFaceUrlSource::new(CssUrl::new("face"), None, technology.clone());
    let required = value.required_technologies().collect::<Vec<_>>();
    let expected = "url(\"face\") tech(variations, palettes, color-colrv0, color-colrv1, color-svg, color-sbix, color-cbdt, features-opentype, features-aat, features-graphite, incremental, palettes)";
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(value.tech(), technology);
    assert_eq!(value.required_technologies().collect::<Vec<_>>(), required);
    let list = CssFontFaceSourceList::try_new(vec![CssFontFaceSource::Url(value)]).unwrap();
    assert_output(&list, expected);
    let uppercase = "url(face) TECH(VARIATIONS, PALETTES, COLOR-COLRV0, COLOR-COLRV1, COLOR-SVG, COLOR-SBIX, COLOR-CBDT, FEATURES-OPENTYPE, FEATURES-AAT, FEATURES-GRAPHITE, INCREMENTAL, PALETTES)";
    assert_eq!(parsed(uppercase), list);
}

#[test]
fn mixed_source_order_and_url_function_modifiers_use_the_shared_provider() {
    let input = "local(First), SRC(\"relative.face\" CORS integrity(\"sha256\") a\\ b) format(WOFF2) tech(PALETTES, PALETTES), url(), local('last')";
    let expected = "local(\"First\"), src(\"relative.face\" CORS integrity(\"sha256\") a\\ b) format(woff2) tech(palettes, palettes), url(\"\"), local(\"last\")";
    let value = parsed(input);
    assert_output(&value, expected);
    let CssFontFaceSource::Url(url) = &value.sources()[1] else {
        panic!("URL source")
    };
    assert_eq!(url.url().function(), CssUrlFunction::Src);
    assert_eq!(url.url().as_str(), "relative.face");
    assert_eq!(url.url().modifiers().len(), 3);
    let CssUrlModifier::Function(modifier) = &url.url().modifiers()[1] else {
        panic!("function modifier")
    };
    let before = modifier.argument_components().clone();
    url.serialize_specified().unwrap();
    assert_eq!(modifier.argument_components(), &before);
    assert_output(&parsed("url('  ')"), "url(\"  \")");
}

#[test]
fn source_children_share_exact_input_projection_and_final_byte_limits() {
    let local = CssFontLocalName::try_new("é").unwrap();
    let url = CssFontFaceUrlSource::new(
        CssUrl::from_parts(
            CssUrlFunction::Src,
            "face",
            vec![CssUrlModifier::Ident(CssIdent::try_new("CORS").unwrap())],
        ),
        Some(CssFontFormat::String(CssFontFormatString::new(""))),
        vec![CssFontTechHint::Palettes, CssFontTechHint::Palettes],
    );
    // Local wrapper/string = 2. URL source + URL aggregate/target/modifier +
    // format argument + technology list/two hints = 8. The outer list adds 1.
    let local_text = "local(\"é\")";
    let url_text = "src(\"face\" CORS) format(\"\") tech(palettes, palettes)";
    for (limits, kind) in [
        (Limits::new(1, 2, local_text.len()), Kind::InputNodeLimit),
        (
            Limits::new(2, 1, local_text.len()),
            Kind::ProjectionNodeLimit,
        ),
        (Limits::new(2, 2, local_text.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(
            local
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(
            CssFontFaceSource::Local(local.clone())
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    for (limits, kind) in [
        (Limits::new(7, 8, url_text.len()), Kind::InputNodeLimit),
        (Limits::new(8, 7, url_text.len()), Kind::ProjectionNodeLimit),
        (Limits::new(8, 8, url_text.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(
            url.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(
            CssFontFaceSource::Url(url.clone())
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(
        local
            .serialize_specified_with_limits(Limits::new(2, 2, local_text.len()))
            .unwrap(),
        local_text
    );
    assert_eq!(
        url.serialize_specified_with_limits(Limits::new(8, 8, url_text.len()))
            .unwrap(),
        url_text
    );
    let value = CssFontFaceSourceList::try_new(vec![
        CssFontFaceSource::Local(local),
        CssFontFaceSource::Url(url),
    ])
    .unwrap();
    let before = value.clone();
    let expected = "local(\"é\"), src(\"face\" CORS) format(\"\") tech(palettes, palettes)";
    for (limits, kind) in [
        (Limits::new(10, 11, expected.len()), Kind::InputNodeLimit),
        (
            Limits::new(11, 10, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (Limits::new(11, 11, expected.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, before);
    }
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(11, 11, expected.len()))
            .unwrap(),
        expected
    );
    assert_output(&value, expected);
}

#[test]
fn recovered_source_emission_preserves_the_original_report_and_diagnostic_spans() {
    let source = "/*😀*/\r\n@font-face{src:local(First),local(inherit),url(last);font-display:swap}.after{color:red}";
    let report = parse_sheet(source);
    let syntax = report.syntax().clone();
    let diagnostics = report.diagnostics().to_vec();
    let [diagnostic] = diagnostics.as_slice() else {
        panic!("one rejected member")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropFontSourceListItem
    );
    let start = source.find("local(inherit)").unwrap();
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + "local(inherit)".len()
    );
    assert_eq!(diagnostic.span().start().line().value(), 1);
    let [CssRule::FontFace(face), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("retained sibling")
    };
    let descriptor = face
        .descriptors()
        .effective(CssFontFaceDescriptorKind::Src)
        .unwrap();
    let CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::Src(value)) =
        descriptor.value()
    else {
        panic!("retained sources")
    };
    assert_output(value, "local(\"First\"), url(\"last\")");
    assert_eq!(report.syntax(), &syntax);
    assert_eq!(report.diagnostics(), diagnostics);
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        diagnostics
    );
}
