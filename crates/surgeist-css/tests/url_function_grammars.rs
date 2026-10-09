#![forbid(unsafe_code)]
//! Values 4 (2024-03-12) §4.5 defines `<url>` as quoted `url()` or `src()`
//! (with optional modifiers), or an unquoted URL token.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#urls

#[macro_use]
#[path = "support/font_face.rs"]
mod font_face_support;

use surgeist_css::{
    CssClipPath, CssContentValue, CssContentValueItem, CssCursor, CssCursorImageSource,
    CssCursorKeyword, CssFilter, CssFilterFunction, CssFontFaceSource, CssFontFormatHint,
    CssFontTechHint, CssImageValue, CssImportTarget, CssImportance, CssKnownProperty,
    CssKnownPropertyValueRef, CssRecoveryAction, CssRule, CssUrlModifier, parse_sheet,
    parse_style_attribute, validate_sheet, validate_style_attribute,
};

fn assert_url_payload(value: CssKnownPropertyValueRef<'_>, expected: &str) {
    match value {
        CssKnownPropertyValueRef::BackgroundImage(value) => assert!(matches!(
            value.images().images(),
            [CssImageValue::Url(url)] if url.as_str() == expected
        )),
        CssKnownPropertyValueRef::MaskImage(value) => assert!(matches!(
            value.images().images(),
            [CssImageValue::Url(url)] if url.as_str() == expected
        )),
        CssKnownPropertyValueRef::BorderImageSource(value) => assert!(matches!(
            value.source(),
            CssImageValue::Url(url) if url.as_str() == expected
        )),
        CssKnownPropertyValueRef::ListStyleImage(value) => assert!(matches!(
            value.value(),
            CssImageValue::Url(url) if url.as_str() == expected
        )),
        CssKnownPropertyValueRef::Cursor(value) => assert!(matches!(
            value.value(),
            CssCursor::Images(images)
                if matches!(images.images(), [image] if matches!(image.source(), CssCursorImageSource::Url(url) if url.as_str() == expected) && image.hotspot().is_none())
                    && images.fallback() == CssCursorKeyword::Auto
        )),
        CssKnownPropertyValueRef::Content(value) => assert!(matches!(
            value.value(),
            CssContentValue::Generated(items)
                if matches!(items.items(), [CssContentValueItem::Image(image)] if matches!(image.value(), CssImageValue::Url(url) if url.as_str() == expected))
        )),
        CssKnownPropertyValueRef::Filter(value) => assert!(matches!(
            value.value(),
            CssFilter::Functions(functions)
                if matches!(functions.functions(), [CssFilterFunction::Url(url)] if url.as_str() == expected)
        )),
        CssKnownPropertyValueRef::ClipPath(value) => assert!(matches!(
            value.value(),
            CssClipPath::Url(url) if url.as_str() == expected
        )),
        CssKnownPropertyValueRef::Background(value) => assert!(matches!(
            value.background().layers(),
            [layer] if matches!(layer.image(), Some(CssImageValue::Url(url)) if url.as_str() == expected)
        )),
        CssKnownPropertyValueRef::Mask(value) => {
            assert_eq!(value.value().layers().len(), 1);
        }
        CssKnownPropertyValueRef::BorderImage(value) => assert!(matches!(
            value.border_image().source(),
            Some(CssImageValue::Url(url)) if url.as_str() == expected
        )),
        CssKnownPropertyValueRef::ListStyle(value) => assert!(matches!(
            value.value().image(),
            Some(CssImageValue::Url(url)) if url.as_str() == expected
        )),
        other => panic!("expected a typed URL consumer, got {other:?}"),
    }
}

#[test]
fn quoted_src_is_an_ordinary_url_in_each_direct_consumer() {
    for name in [
        "background-image",
        "mask-image",
        "border-image-source",
        "list-style-image",
        "cursor",
        "content",
        "filter",
        "clip-path",
    ] {
        for function in ["src", "url"] {
            let value = if name == "cursor" {
                format!("{function}(\"asset.svg\"), auto")
            } else {
                format!("{function}(\"asset.svg\")")
            };
            let source = format!("{name}: {value}");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert!(validate_style_attribute(&source).is_ok(), "{source}");
            let [declaration] = report.syntax().as_slice() else {
                panic!("one retained declaration: {source}")
            };
            assert_eq!(
                declaration.known().unwrap().property().canonical_name(),
                name
            );
            assert_url_payload(
                declaration.known().unwrap().property_value().unwrap(),
                "asset.svg",
            );
        }
    }
}

#[test]
fn quoted_url_controls_remain_clean_in_ordinary_import_and_font_paths() {
    let ordinary = parse_style_attribute("background-image: url(\"asset.svg\")");
    assert!(ordinary.is_clean());
    let [declaration] = ordinary.syntax().as_slice() else {
        panic!("one background image")
    };
    assert_url_payload(
        declaration.known().unwrap().property_value().unwrap(),
        "asset.svg",
    );

    let rules = parse_sheet(concat!(
        "@import url(\"theme.css\") layer(theme) screen; ",
        "@font-face { font-family: Demo; src: url(\"font.woff2\") format(\"woff2\"); }"
    ));
    assert!(rules.is_clean(), "{:?}", rules.diagnostics());
    let [CssRule::Import(import), CssRule::FontFace(face)] = rules.syntax().rules() else {
        panic!("import and font face")
    };
    assert!(matches!(import.target(), CssImportTarget::Url(url) if url.as_str() == "theme.css"));
    assert!(import.layer().is_some());
    assert!(import.media().is_some());
    let [CssFontFaceSource::Url(font)] = ordinary_face!(face.descriptors(), Src).unwrap().sources()
    else {
        panic!("one font URL")
    };
    assert_eq!(font.url().as_str(), "font.woff2");
    assert_eq!(
        font.format()
            .and_then(surgeist_css::CssFontFormat::recognized_format),
        Some(CssFontFormatHint::Woff2)
    );
}

#[test]
fn quoted_src_accepts_ordered_modifiers_and_keeps_important_neighbors() {
    let source = concat!(
        "color: red; background-image: src(\"icon.svg\" cors integrity(\"sha256\")) ",
        "!important; width: 2px"
    );
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 3);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(
        report.syntax()[2].known().unwrap().property(),
        CssKnownProperty::Width
    );
    let declaration = &report.syntax()[1];
    assert_eq!(declaration.importance(), CssImportance::Important);
    let CssKnownPropertyValueRef::BackgroundImage(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("background image with modifiers")
    };
    let [CssImageValue::Url(url)] = value.images().images() else {
        panic!("one URL image")
    };
    assert_eq!(url.as_str(), "icon.svg");
    assert!(matches!(
        url.modifiers(),
        [CssUrlModifier::Ident(ident), CssUrlModifier::Function(function)]
            if ident.as_str() == "cors" && function.name() == "integrity"
                && function.arguments().as_css() == "\"sha256\""
    ));
}

#[test]
fn shared_url_shorthands_admit_quoted_src_images() {
    for name in ["background", "mask", "border-image", "list-style"] {
        let source = format!("{name}: src(\"asset.svg\")");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(validate_style_attribute(&source).is_ok(), "{source}");
        let [declaration] = report.syntax().as_slice() else {
            panic!("one retained shorthand: {source}")
        };
        assert_eq!(
            declaration.known().unwrap().property().canonical_name(),
            name
        );
        assert_url_payload(
            declaration.known().unwrap().property_value().unwrap(),
            "asset.svg",
        );
    }
}

#[test]
fn import_accepts_src_and_quoted_url_modifiers_without_losing_clauses() {
    let source = concat!(
        "@import src(\"theme.css\" cors) layer(theme) screen; ",
        "@import url(\"base.css\" integrity(\"sha256\")); ",
        ".after { color: red; }"
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(validate_sheet(source).unwrap(), *report.syntax());
    let [
        CssRule::Import(first),
        CssRule::Import(second),
        CssRule::Style(_),
    ] = report.syntax().rules()
    else {
        panic!("two ordered imports and a following style")
    };
    assert!(matches!(first.target(), CssImportTarget::Url(url) if url.as_str() == "theme.css"));
    assert!(first.layer().is_some());
    assert!(first.media().is_some());
    assert!(matches!(second.target(), CssImportTarget::Url(url) if url.as_str() == "base.css"));
    assert!(second.layer().is_none());
    assert!(second.media().is_none());
}

#[test]
fn font_face_accepts_src_url_sources_with_ordered_hints() {
    let source = concat!(
        "@font-face { font-family: Demo; src: ",
        "src(\"first.woff2\" cors) format(\"woff2\"), ",
        "url(\"second.woff2\" integrity(\"sha256\")) tech(variations); } ",
        ".after { color: red; }"
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(validate_sheet(source).is_ok());
    let [CssRule::FontFace(face), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("retained font face and next style")
    };
    let sources = ordinary_face!(face.descriptors(), Src).unwrap().sources();
    let [
        CssFontFaceSource::Url(first),
        CssFontFaceSource::Url(second),
    ] = sources
    else {
        panic!("two ordered URL sources")
    };
    assert_eq!(first.url().as_str(), "first.woff2");
    assert_eq!(
        first
            .format()
            .and_then(surgeist_css::CssFontFormat::recognized_format),
        Some(CssFontFormatHint::Woff2)
    );
    assert_eq!(second.url().as_str(), "second.woff2");
    assert_eq!(second.tech(), &[CssFontTechHint::Variations]);
}

#[test]
fn namespace_keeps_literal_uri_grammar_without_src_function() {
    let accepted = parse_sheet("@namespace svg url(\"urn:svg\"); .after { color: red; }");
    assert!(accepted.is_clean(), "{:?}", accepted.diagnostics());
    let [CssRule::Namespace(namespace), CssRule::Style(_)] = accepted.syntax().rules() else {
        panic!("URL namespace and sibling")
    };
    assert_eq!(namespace.name().as_str(), "urn:svg");

    let rejected = parse_sheet("@namespace svg src(\"urn:svg\"); .after { color: red; }");
    assert!(!rejected.is_clean());
    assert!(matches!(rejected.syntax().rules(), [CssRule::Style(_)]));
    assert!(validate_sheet("@namespace svg src(\"urn:svg\"); .after { color: red; }").is_err());
}

#[test]
fn malformed_src_and_unquoted_url_functions_recover_atomically() {
    for invalid in [
        "src(foo)",
        "src()",
        "src(\"x\", junk)",
        "src(\"x\" 2)",
        "url(var(--x))",
    ] {
        let source = format!("color: red; background-image: {invalid}; width: 2px");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Width
        );
        assert!(validate_style_attribute(&source).is_err(), "{source}");
    }
}

#[test]
fn src_with_var_remains_a_pending_known_property_value() {
    let report = parse_style_attribute("background-image: src(var(--asset)) !important");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one pending declaration")
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    let known = declaration.known().unwrap();
    assert_eq!(known.property(), CssKnownProperty::BackgroundImage);
    assert!(known.property_value().is_none());
    assert_eq!(
        known.substitution_dependent().unwrap().as_css(),
        "src(var(--asset))"
    );
}
