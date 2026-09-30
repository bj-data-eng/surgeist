#![forbid(unsafe_code)]
//! Values 4 (2024-03-12) §4.5 keeps `url()` and `src()` distinct authored
//! `<url>` forms, including empty targets and ordered quoted-form modifiers.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#urls

#[macro_use]
#[path = "support/font_face.rs"]
mod font_face_support;

use surgeist_css::{
    CssFontFaceSource, CssFontFaceUrlSource, CssImportTarget, CssImportUrl,
    CssKnownPropertyValueRef, CssRecoveryAction, CssRule, CssUrl, CssUrlFunction, CssUrlModifier,
    parse_sheet, parse_style_attribute,
};

fn background_url(value: &str) -> CssUrl {
    let report = parse_style_attribute(&format!("background-image: {value}"));
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one background image")
    };
    let CssKnownPropertyValueRef::BackgroundImage(image) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed image")
    };
    let [surgeist_css::CssImageValue::Url(url)] = image.images().images() else {
        panic!("one URL image")
    };
    url.clone()
}

#[test]
fn checked_constructors_preserve_function_identity_and_decoded_empty_targets() {
    for target in ["", " ", "asset.svg"] {
        let url = CssUrl::new(target);
        let src = CssUrl::from_parts(CssUrlFunction::Src, target, Vec::new());
        assert_eq!(url.function(), CssUrlFunction::Url);
        assert_eq!(src.function(), CssUrlFunction::Src);
        assert_eq!(url.as_str(), target);
        assert_eq!(src.as_str(), target);
        assert_ne!(url, src);
        assert_eq!(
            CssUrl::from_parts(CssUrlFunction::Url, target, Vec::new()),
            url
        );

        let url_import = CssImportUrl::new(surgeist_css::CssUrl::new(target));
        let import = CssImportUrl::new(src.clone());
        assert_eq!(url_import.url().function(), CssUrlFunction::Url);
        assert_eq!(url_import.as_str(), target);
        assert_eq!(import.url(), &src);
        assert_eq!(import.as_str(), target);
        assert_ne!(url_import, import);

        let url_font =
            CssFontFaceUrlSource::new(surgeist_css::CssUrl::new(target), None, Vec::new());
        let font = CssFontFaceUrlSource::new(src.clone(), None, Vec::new());
        assert_eq!(url_font.url().function(), CssUrlFunction::Url);
        assert_eq!(url_font.url().as_str(), target);
        assert_eq!(font.url(), &src);
        assert_eq!(font.url().as_str(), target);
        assert_ne!(url_font, font);
    }
}

#[test]
fn parsed_src_casefolds_function_name_and_keeps_modifier_order() {
    let parsed = background_url("SRC(\"asset.svg\" cors integrity(sha256))");
    assert_eq!(parsed.function(), CssUrlFunction::Src);
    assert_eq!(parsed.as_str(), "asset.svg");
    assert!(matches!(
        parsed.modifiers(),
        [CssUrlModifier::Ident(ident), CssUrlModifier::Function(function)]
            if ident.as_str() == "cors"
                && function.name() == "integrity"
                && function.arguments().as_css() == "sha256"
    ));
    let constructed = CssUrl::from_parts(
        CssUrlFunction::Src,
        "asset.svg",
        parsed.modifiers().to_vec(),
    );
    assert_eq!(parsed, constructed);
    assert_ne!(
        parsed,
        CssUrl::from_parts(
            CssUrlFunction::Url,
            "asset.svg",
            parsed.modifiers().to_vec()
        )
    );

    assert_eq!(background_url("src(\"\")").function(), CssUrlFunction::Src);
    assert_eq!(background_url("src(\"\")").as_str(), "");
    assert_eq!(background_url("SRC(\" \" )").as_str(), " ");
    assert_eq!(background_url("url(\"\")").function(), CssUrlFunction::Url);
}

#[test]
fn import_and_font_sources_expose_shared_payload_and_preserve_import_spelling() {
    let source = concat!(
        "@import SRC(\"theme.css\" cors) layer(theme); ",
        "@font-face { font-family: Demo; src: ",
        "src(\"first.woff2\" integrity(sha256)), ",
        "url(\"second.woff2\" cors); }"
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Import(import), CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("import then font face")
    };
    let CssImportTarget::Url(target) = import.target() else {
        panic!("URL import target")
    };
    assert_eq!(target.as_str(), "theme.css");
    assert_eq!(target.url().function(), CssUrlFunction::Src);
    assert!(
        matches!(target.url().modifiers(), [CssUrlModifier::Ident(ident)] if ident.as_str() == "cors")
    );
    assert_eq!(
        import.serialize().unwrap().as_css(),
        "@import SRC(\"theme.css\" cors) layer(theme);"
    );

    let [
        CssFontFaceSource::Url(first),
        CssFontFaceSource::Url(second),
    ] = ordinary_face!(face.descriptors(), Src).unwrap().sources()
    else {
        panic!("two ordered font URLs")
    };
    assert_eq!(first.url().as_str(), "first.woff2");
    assert_eq!(first.url().function(), CssUrlFunction::Src);
    assert!(
        matches!(first.url().modifiers(), [CssUrlModifier::Function(function)]
        if function.name() == "integrity" && function.arguments().as_css() == "sha256")
    );
    assert_eq!(second.url().as_str(), "second.woff2");
    assert_eq!(second.url().function(), CssUrlFunction::Url);
    assert!(
        matches!(second.url().modifiers(), [CssUrlModifier::Ident(ident)] if ident.as_str() == "cors")
    );
}

#[test]
fn malformed_src_members_recover_without_losing_valid_font_and_rule_neighbors() {
    let source = concat!(
        "@import src(foo); ",
        "@font-face { font-family: Demo; src: ",
        "src(\"first.woff2\"), src(foo), src(\"last.woff2\"); } ",
        ".after { color: red; }"
    );
    let report = parse_sheet(source);
    assert!(!report.is_clean());
    let [CssRule::FontFace(face), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("font face and style survive invalid import")
    };
    let sources = ordinary_face!(face.descriptors(), Src).unwrap().sources();
    let [CssFontFaceSource::Url(first), CssFontFaceSource::Url(last)] = sources else {
        panic!("valid font source order preserved")
    };
    assert_eq!(first.url().as_str(), "first.woff2");
    assert_eq!(last.url().as_str(), "last.woff2");
    assert_eq!(first.url().function(), CssUrlFunction::Src);
    assert_eq!(last.url().function(), CssUrlFunction::Src);
    assert_eq!(report.diagnostics().len(), 2);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
    assert_eq!(
        report.diagnostics()[1].action(),
        CssRecoveryAction::DropFontSourceListItem
    );
}
