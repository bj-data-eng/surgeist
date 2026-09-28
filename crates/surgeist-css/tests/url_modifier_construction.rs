#![forbid(unsafe_code)]
//! Values 4 (2024-03-12) §§4.5–4.5.3: URL modifiers are identifiers or
//! functional notation; their unknown arguments remain authored syntax.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#url-modifiers

use surgeist_css::{
    CssAuthoredFontFaceDescriptorValue, CssComponentValue, CssComponentValueErrorKind,
    CssComponentValueRef, CssComponentValues, CssFontFaceDescriptorKind,
    CssFontFaceDescriptorValue, CssFontFaceSource, CssIdent, CssImageValue, CssImportRule,
    CssImportTarget, CssImportance, CssKnownProperty, CssKnownPropertyValueRef,
    CssNamespaceContext, CssPropertyNameRef, CssRule, CssUrl, CssUrlFunction, CssUrlModifier,
    CssUrlModifierFunction, CssValueOrigin, CssValueTokenRef, parse_component_values,
    parse_property_value, parse_sheet, parse_style_attribute,
};

fn modifier(url: &CssUrl) -> &CssUrlModifierFunction {
    let [CssUrlModifier::Function(function)] = url.modifiers() else {
        panic!("one functional modifier: {:?}", url.modifiers());
    };
    function
}

fn background_url(value: CssKnownPropertyValueRef<'_>) -> &CssUrl {
    let CssKnownPropertyValueRef::BackgroundImage(images) = value else {
        panic!("background image value");
    };
    let [CssImageValue::Url(url)] = images.images().images() else {
        panic!("one URL image");
    };
    url
}

fn assert_parsed_child_from_source(function: &CssUrlModifierFunction, source: &str, text: &str) {
    let [child] = function.argument_components().items() else {
        panic!("one argument token");
    };
    let CssValueOrigin::Parsed(origin) = child.origin() else {
        panic!("original parsed child origin");
    };
    assert_eq!(origin.source().as_str(), source);
    let span = origin.span();
    assert_eq!(
        &source[span.start().byte_offset().value()..span.end().byte_offset().value()],
        text
    );
}

#[test]
fn checked_decoded_identifiers_and_modifier_function_follow_token_grammar() {
    for decoded in ["1mod", "a b", "x:y", "𝛼"] {
        assert_eq!(CssIdent::try_new(decoded).unwrap().as_str(), decoded);
    }
    for invalid in ["", "a\0b"] {
        assert_eq!(
            CssIdent::try_new(invalid).unwrap_err().kind(),
            CssComponentValueErrorKind::InvalidIdentifier
        );
    }

    let arguments = parse_component_values("\"asset\"").unwrap();
    let url_name = CssIdent::try_new("url").unwrap();
    assert!(CssUrlModifierFunction::try_new(url_name.clone(), arguments).is_ok());
    let invalid =
        CssUrlModifierFunction::try_new(url_name, parse_component_values("asset").unwrap())
            .unwrap_err();
    assert_eq!(invalid.kind(), CssComponentValueErrorKind::InvalidFunction);
    assert!(
        CssUrlModifierFunction::try_new(
            CssIdent::try_new("opaque").unwrap(),
            parse_component_values("asset").unwrap(),
        )
        .is_ok()
    );
}

#[test]
fn checked_modifier_retains_child_origins_and_old_text_equality() {
    let parsed_child = parse_component_values("token").unwrap().items()[0].clone();
    let arguments = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("first").unwrap(),
        CssComponentValue::try_token(" ").unwrap(),
        parsed_child,
    ])
    .unwrap();
    let function =
        CssUrlModifierFunction::try_new(CssIdent::try_new("m").unwrap(), arguments).unwrap();
    assert_eq!(function.arguments().as_css(), "first token");
    assert!(matches!(
        function.argument_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    assert!(matches!(
        function.argument_components().items()[2].origin(),
        CssValueOrigin::Parsed(_)
    ));

    let same_text = CssUrlModifierFunction::try_new(
        CssIdent::try_new("m").unwrap(),
        parse_component_values("first token").unwrap(),
    )
    .unwrap();
    assert_eq!(function, same_text, "source origins are not URL equality");
    assert_eq!(
        CssUrl::from_parts(
            CssUrlFunction::Src,
            "x",
            vec![CssUrlModifier::Function(function)]
        ),
        CssUrl::from_parts(
            CssUrlFunction::Src,
            "x",
            vec![CssUrlModifier::Function(same_text)]
        ),
    );
}

#[test]
fn ordinary_and_checked_property_urls_keep_original_modifier_token_sources() {
    let source = "color:red;background-image:src(\"x\" m(original));width:2px";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let url = background_url(
        report.syntax()[1]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
    );
    let function = modifier(url);
    assert_eq!(function.arguments().as_css(), "original");
    assert_parsed_child_from_source(function, source, "original");

    let authored = "src(\"x\" m(original))";
    let components = parse_component_values(authored).unwrap();
    let original_source = match components.items()[0].origin() {
        CssValueOrigin::Parsed(origin) => origin.source().clone(),
        other => panic!("parsed input: {other:?}"),
    };
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage),
        components,
        CssImportance::Normal,
    )
    .unwrap();
    let url = background_url(checked.known().unwrap().property_value().unwrap());
    let [child] = modifier(url).argument_components().items() else {
        panic!("one argument");
    };
    assert!(matches!(
        child.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("original"))
    ));
    let CssValueOrigin::Parsed(origin) = child.origin() else {
        panic!("retained checked origin");
    };
    assert!(origin.source().same_snapshot(&original_source));
}

#[test]
fn ordinary_and_checked_import_and_font_urls_keep_original_sources() {
    let source =
        "@import src(\"a\" m(imported));@font-face{font-family:Demo;src:url(\"b\" m(fonted))}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Import(import), CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("import and font face");
    };
    let CssImportTarget::Url(target) = import.target() else {
        panic!("URL import");
    };
    assert_parsed_child_from_source(modifier(target.url()), source, "imported");
    let descriptor = face
        .descriptors()
        .effective(CssFontFaceDescriptorKind::Src)
        .unwrap();
    let CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::Src(sources)) =
        descriptor.value()
    else {
        panic!("font source descriptor");
    };
    let [CssFontFaceSource::Url(font)] = sources.sources() else {
        panic!("font URL");
    };
    assert_parsed_child_from_source(modifier(font.authored_url()), source, "fonted");

    let authored_import = "@import src(\"a\" m(imported));";
    let components = parse_component_values(authored_import).unwrap();
    let original_source = match components.items()[0].origin() {
        CssValueOrigin::Parsed(origin) => origin.source().clone(),
        other => panic!("parsed import: {other:?}"),
    };
    let checked =
        CssImportRule::try_from_components(components, &CssNamespaceContext::default()).unwrap();
    let CssImportTarget::Url(target) = checked.target() else {
        panic!("checked URL import");
    };
    let [child] = modifier(target.url()).argument_components().items() else {
        panic!("one child");
    };
    let CssValueOrigin::Parsed(origin) = child.origin() else {
        panic!("original checked import child");
    };
    assert!(origin.source().same_snapshot(&original_source));

    let authored_font = "url(\"b\" m(fonted))";
    let components = parse_component_values(authored_font).unwrap();
    let original_source = match components.items()[0].origin() {
        CssValueOrigin::Parsed(origin) => origin.source().clone(),
        other => panic!("parsed font: {other:?}"),
    };
    let checked = CssAuthoredFontFaceDescriptorValue::try_from_components(
        CssFontFaceDescriptorKind::Src,
        components,
    )
    .unwrap();
    let CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::Src(sources)) =
        checked
    else {
        panic!("checked font source");
    };
    let [CssFontFaceSource::Url(font)] = sources.sources() else {
        panic!("checked font URL");
    };
    let [child] = modifier(font.authored_url()).argument_components().items() else {
        panic!("one child");
    };
    let CssValueOrigin::Parsed(origin) = child.origin() else {
        panic!("original checked font child");
    };
    assert!(origin.source().same_snapshot(&original_source));
}

#[test]
fn local_url_flag_tracks_decoded_fragment_prefix_without_resolution() {
    for function in [CssUrlFunction::Url, CssUrlFunction::Src] {
        for target in ["#", "#image"] {
            assert!(CssUrl::from_parts(function, target, vec![]).is_local_url());
        }
        for target in ["", " #image", "asset.svg#image"] {
            assert!(!CssUrl::from_parts(function, target, vec![]).is_local_url());
        }
    }
    for (source, expected) in [("url(\"\\23 image\")", true), ("src(\" #image\")", false)] {
        let report = parse_style_attribute(&format!("background-image:{source}"));
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let url = background_url(
            report.syntax()[0]
                .known()
                .unwrap()
                .property_value()
                .unwrap(),
        );
        assert_eq!(url.is_local_url(), expected, "{source}");
    }
}

fn programmatic_url_function() -> CssComponentValue {
    let modifier = CssComponentValue::try_function(
        "m",
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("token").unwrap()]).unwrap(),
    )
    .unwrap();
    CssComponentValue::try_function(
        "src",
        CssComponentValues::try_new(vec![
            CssComponentValue::try_string("x").unwrap(),
            CssComponentValue::try_token(" ").unwrap(),
            modifier,
        ])
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn checked_property_import_and_font_preserve_programmatic_modifier_children() {
    let property = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage),
        CssComponentValues::try_new(vec![programmatic_url_function()]).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let url = background_url(property.known().unwrap().property_value().unwrap());
    assert!(matches!(
        modifier(url).argument_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));

    let import = CssImportRule::try_from_components(
        CssComponentValues::try_new(vec![
            CssComponentValue::try_token("@import").unwrap(),
            CssComponentValue::try_token(" ").unwrap(),
            programmatic_url_function(),
            CssComponentValue::try_token(";").unwrap(),
        ])
        .unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    let CssImportTarget::Url(target) = import.target() else {
        panic!("checked URL import");
    };
    assert!(matches!(
        modifier(target.url()).argument_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));

    let font = CssAuthoredFontFaceDescriptorValue::try_from_components(
        CssFontFaceDescriptorKind::Src,
        CssComponentValues::try_new(vec![programmatic_url_function()]).unwrap(),
    )
    .unwrap();
    let CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::Src(sources)) =
        font
    else {
        panic!("checked font source");
    };
    let [CssFontFaceSource::Url(font)] = sources.sources() else {
        panic!("checked font URL");
    };
    assert!(matches!(
        modifier(font.authored_url()).argument_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
}

#[test]
fn parsed_eof_modifier_arguments_retain_implied_closures_and_decoded_escape() {
    let source = "background-image:url(\"x\" m(f(";
    let report = parse_style_attribute(source);
    let [declaration] = report.syntax().as_slice() else {
        panic!("retained URL at EOF: {:?}", report.diagnostics());
    };
    let url = background_url(declaration.known().unwrap().property_value().unwrap());
    let function = modifier(url);
    assert_eq!(function.arguments().as_css(), "f(");
    let [child] = function.argument_components().items() else {
        panic!("one nested function");
    };
    let CssComponentValueRef::Function(inner) = child.view() else {
        panic!("nested function token");
    };
    assert!(matches!(
        inner.closing_origin(),
        CssValueOrigin::ImplicitClosure { .. }
    ));

    let source = "background-image:url(\"x\" m(a\\";
    let report = parse_style_attribute(source);
    let [declaration] = report.syntax().as_slice() else {
        panic!("retained escaped URL at EOF: {:?}", report.diagnostics());
    };
    let url = background_url(declaration.known().unwrap().property_value().unwrap());
    let function = modifier(url);
    assert_eq!(function.arguments().as_css(), "a\\");
    assert!(matches!(
        function.argument_components().items(),
        [child] if matches!(child.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident("a\u{fffd}")))
    ));
}
