#![forbid(unsafe_code)]
//! Values 4 (2024-03-12) §4.5.2: an empty URL is valid authored syntax;
//! resource resolution later treats it as an invalid resource.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#url-empty

use surgeist_css::{
    CssComponentValue, CssComponentValues, CssContentValue, CssContentValueItem, CssCursor,
    CssCursorKeyword, CssImageValue, CssImportance, CssKnownProperty, CssKnownPropertyValueRef,
    CssMaskLayer, CssMaskList, CssPropertyNameRef, CssRecoveryAction, CssUrl, CssUrlModifier,
    CssValueOrigin, parse_property_value, parse_style_attribute, validate_style_attribute,
};

fn assert_url_payload(name: &str, value: CssKnownPropertyValueRef<'_>, expected: &str) {
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
        CssKnownPropertyValueRef::Content(value) => assert!(matches!(
            value.value(),
            CssContentValue::Generated(items)
                if matches!(items.items(), [CssContentValueItem::Image(image)] if matches!(image.value(), CssImageValue::Url(url) if url.as_str() == expected))
        )),
        CssKnownPropertyValueRef::Cursor(value) => assert!(matches!(
            value.value(),
            CssCursor::Urls(urls)
                if matches!(urls.urls().urls(), [url] if url.as_str() == expected)
                    && urls.fallback() == CssCursorKeyword::Auto
        )),
        CssKnownPropertyValueRef::Background(value) => assert!(matches!(
            value.background().layers(),
            [layer] if matches!(layer.image(), Some(CssImageValue::Url(url)) if url.as_str() == expected)
        )),
        CssKnownPropertyValueRef::BorderImage(value) => assert!(matches!(
            value.border_image().source(),
            Some(CssImageValue::Url(url)) if url.as_str() == expected
        )),
        CssKnownPropertyValueRef::ListStyle(value) => assert!(matches!(
            value.value().image(),
            Some(CssImageValue::Url(url)) if url.as_str() == expected
        )),
        CssKnownPropertyValueRef::Mask(value) => {
            let expected_mask = CssMaskList::try_new(vec![
                CssMaskLayer::try_new(
                    Some(CssImageValue::Url(CssUrl::new(expected))),
                    None,
                    None,
                    None,
                )
                .unwrap(),
            ])
            .unwrap();
            assert_eq!(value.value(), &expected_mask);
        }
        _ => panic!("expected typed URL consumer for {name}"),
    }
}

fn assert_clean_url(name: &str, authored: &str, expected: &str) {
    let value = if name == "cursor" {
        format!("{authored}, auto")
    } else {
        authored.to_owned()
    };
    let source = format!("{name}: {value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1, "{source}");
    assert!(validate_style_attribute(&source).is_ok(), "{source}");
    let known = report.syntax()[0].known().expect("known URL property");
    assert_eq!(known.property().canonical_name(), name);
    assert_url_payload(
        name,
        known.property_value().expect("ordinary URL"),
        expected,
    );
}

#[test]
fn empty_url_is_checked_rust_value_and_distinct_from_whitespace() {
    assert_eq!(CssUrl::new("").as_str(), "");
    assert_eq!(CssUrl::new(" ").as_str(), " ");
}

#[test]
fn all_shared_url_consumers_retain_each_empty_authored_form() {
    for name in [
        "background-image",
        "mask-image",
        "border-image-source",
        "list-style-image",
        "content",
        "cursor",
        "background",
        "mask",
        "list-style",
        "border-image",
    ] {
        for authored in ["url(\"\")", "url()", "url('')", "url(   )"] {
            assert_clean_url(name, authored, "");
        }
    }
}

#[test]
fn quoted_whitespace_remains_data_and_nonempty_urls_remain_valid() {
    for name in ["background-image", "content", "cursor"] {
        assert_clean_url(name, "url(\" \" )", " ");
        assert_clean_url(name, "url(icon.svg)", "icon.svg");
    }
}

#[test]
fn empty_quoted_url_keeps_modifiers_and_valid_neighbors() {
    let source =
        "color: red; background-image: url(\"\" cors integrity(sha256)) !important; width: 2px";
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
        panic!("expected typed background image");
    };
    let [CssImageValue::Url(url)] = value.images().images() else {
        panic!("expected one URL image");
    };
    assert_eq!(url.as_str(), "");
    assert!(matches!(
        url.modifiers(),
        [CssUrlModifier::Ident(ident), CssUrlModifier::Function(function)]
            if ident.as_str() == "cors" && function.name() == "integrity"
                && function.arguments().as_css() == "sha256"
    ));
}

#[test]
fn checked_component_construction_keeps_programmatic_empty_url_origin() {
    let url = CssComponentValue::try_url("").unwrap();
    let components = CssComponentValues::try_new(vec![url]).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage),
        components,
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert!(matches!(
        declaration.value_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    assert_url_payload(
        "background-image",
        declaration.known().unwrap().property_value().unwrap(),
        "",
    );
}

#[test]
fn malformed_urls_and_modifier_tokens_drop_only_their_declaration() {
    for invalid in ["url(foo\"bar)", "url(\"\" 2)"] {
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
