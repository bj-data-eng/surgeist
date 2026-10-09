#![forbid(unsafe_code)]
//! Values 4 (2024-03-12) §4.5 authored `<url>` functions and modifiers:
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#urls

use surgeist_css::{
    CssComponentValue, CssComponentValueErrorKind, CssComponentValueRef, CssComponentValues,
    CssIdent, CssImageValue, CssKnownPropertyValueRef, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssUrl, CssUrlFunction, CssUrlModifier,
    CssUrlModifierFunction, CssValueTokenRef, parse_component_values, parse_style_attribute,
};

fn limits(nodes: usize, bytes: usize) -> CssSpecifiedValueSerializationLimits {
    CssSpecifiedValueSerializationLimits::new(nodes, nodes, bytes)
}

fn parsed_background_url(source: &str) -> (CssUrl, bool) {
    let report = parse_style_attribute(source);
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained declaration: {source}")
    };
    let CssKnownPropertyValueRef::BackgroundImage(image) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed background image: {source}")
    };
    let [CssImageValue::Url(url)] = image.images().images() else {
        panic!("one retained URL image: {source}")
    };
    (url.clone(), report.is_clean())
}

fn function_modifier(name: &str, arguments: CssComponentValues) -> CssUrlModifier {
    CssUrlModifier::Function(
        CssUrlModifierFunction::try_new(CssIdent::try_new(name).unwrap(), arguments).unwrap(),
    )
}

#[test]
fn checked_urls_emit_explicit_lowercase_function_identity_and_decoded_target() {
    for (function, target, expected) in [
        (CssUrlFunction::Url, "", "url(\"\")"),
        (CssUrlFunction::Src, "", "src(\"\")"),
        (CssUrlFunction::Url, " ", "url(\" \")"),
        (CssUrlFunction::Src, "#id", "src(\"#id\")"),
    ] {
        let value = CssUrl::from_parts(function, target, Vec::new());
        assert_eq!(value.function(), function);
        assert_eq!(value.as_str(), target);
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
    assert_eq!(
        CssUrl::new("icon.svg").serialize_specified().unwrap(),
        "url(\"icon.svg\")"
    );
}

#[test]
fn parsed_function_and_modifier_names_keep_decoded_case_and_order() {
    let (value, clean) = parsed_background_url(
        "background-image: SRC(\"asset.svg\" CORS integrity(\"sha256\") a\\ b)",
    );
    assert!(clean);
    assert_eq!(value.function(), CssUrlFunction::Src);
    assert_eq!(value.modifiers().len(), 3);
    assert_eq!(
        value.serialize_specified().unwrap(),
        "src(\"asset.svg\" CORS integrity(\"sha256\") a\\ b)"
    );

    let constructed = CssUrl::from_parts(
        CssUrlFunction::Src,
        "asset.svg",
        vec![
            CssUrlModifier::Ident(CssIdent::try_new("CORS").unwrap()),
            function_modifier("integrity", parse_component_values("\"sha256\"").unwrap()),
            CssUrlModifier::Ident(CssIdent::try_new("a b").unwrap()),
        ],
    );
    assert_eq!(
        constructed.serialize_specified().unwrap(),
        "src(\"asset.svg\" CORS integrity(\"sha256\") a\\ b)"
    );
}

#[test]
fn decoded_string_escaping_preserves_quotes_backslashes_controls_and_unicode() {
    let target = "é\"\\\n\u{0001}";
    let value = CssUrl::new(target);
    let expected = "url(\"é\\\"\\\\\\a \\1 \")";
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(value.as_str(), target);

    let nul = CssUrl::from_parts(CssUrlFunction::Src, "a\0b", Vec::new());
    assert_eq!(nul.serialize_specified().unwrap(), "src(\"a�b\")");
    assert_eq!(nul.as_str(), "a\0b");
}

#[test]
fn retained_eof_modifier_components_gain_explicit_closures_without_losing_identity() {
    let (nested, clean) = parsed_background_url("background-image: url(\"x\" m(f(");
    assert!(!clean);
    let [CssUrlModifier::Function(modifier)] = nested.modifiers() else {
        panic!("one function modifier")
    };
    assert_eq!(modifier.arguments().as_css(), "f(");
    assert_eq!(nested.serialize_specified().unwrap(), "url(\"x\" m(f()))");

    let (escaped, clean) = parsed_background_url("background-image: src(\"x\" m(a\\");
    assert!(!clean);
    let [CssUrlModifier::Function(modifier)] = escaped.modifiers() else {
        panic!("one function modifier")
    };
    assert_eq!(modifier.arguments().as_css(), "a\\");
    assert!(matches!(
        modifier.argument_components().items(),
        [component] if matches!(component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Ident("a�")))
    ));
    assert_eq!(
        escaped.serialize_specified().unwrap(),
        "src(\"x\" m(a\\fffd ))"
    );
}

#[test]
fn checked_adjacent_argument_tokens_keep_their_distinct_meaning() {
    let arguments = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("a").unwrap(),
        CssComponentValue::try_number("1").unwrap(),
    ])
    .unwrap();
    let value = CssUrl::from_parts(
        CssUrlFunction::Url,
        "x",
        vec![function_modifier("m", arguments)],
    );
    let output = value.serialize_specified().unwrap();
    assert_eq!(output, "url(\"x\" m(a/**/1))");
    let (reparsed, clean) = parsed_background_url(&format!("background-image: {output}"));
    assert!(clean);
    let [CssUrlModifier::Function(modifier)] = reparsed.modifiers() else {
        panic!("retained function modifier")
    };
    let components = modifier.argument_components().items();
    assert!(matches!(
        components[0].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("a"))
    ));
    assert!(
        matches!(components[2].view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.representation() == "1")
    );
}

#[test]
fn comments_and_whitespace_in_opaque_arguments_remain_meaningful() {
    let value = CssUrl::from_parts(
        CssUrlFunction::Src,
        "x",
        vec![function_modifier(
            "m",
            parse_component_values(" a/**/b ").unwrap(),
        )],
    );
    assert_eq!(
        value.serialize_specified().unwrap(),
        "src(\"x\" m( a/**/b ))"
    );
    assert_eq!(value.modifiers().len(), 1);
}

#[test]
fn one_url_charges_aggregate_target_modifiers_and_nested_argument_nodes() {
    let value = CssUrl::from_parts(
        CssUrlFunction::Url,
        "x",
        vec![
            CssUrlModifier::Ident(CssIdent::try_new("cors").unwrap()),
            function_modifier("m", parse_component_values("a b").unwrap()),
        ],
    );
    let expected = "url(\"x\" cors m(a b))";
    assert_eq!(
        value
            .serialize_specified_with_limits(limits(7, expected.len()))
            .unwrap(),
        expected
    );
    for (limit, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(6, 7, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(7, 6, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            limits(7, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limit)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(value.serialize_specified().unwrap(), expected);
}

#[test]
fn escaped_utf8_bytes_and_many_modifiers_share_one_budget() {
    let value = CssUrl::from_parts(
        CssUrlFunction::Src,
        "é\n",
        (0..12)
            .map(|_| CssUrlModifier::Ident(CssIdent::try_new("cors").unwrap()))
            .collect(),
    );
    let expected = format!("src(\"é\\a \"{})", " cors".repeat(12));
    assert_eq!(
        value
            .serialize_specified_with_limits(limits(14, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(limits(14, expected.len() - 1))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(limits(13, expected.len()))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                14,
                13,
                expected.len()
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
}

#[test]
fn malformed_checked_components_fail_before_url_construction() {
    assert_eq!(
        CssIdent::try_new("").unwrap_err().kind(),
        CssComponentValueErrorKind::InvalidIdentifier
    );
    assert!(CssIdent::try_new("a\0b").is_err());
    assert!(CssComponentValue::try_token("a b").is_err());
    assert!(CssComponentValue::try_string("a\0b").is_err());
    let valid = CssUrl::from_parts(CssUrlFunction::Url, "x", Vec::new());
    assert_eq!(
        valid
            .serialize_specified_with_limits(limits(0, 8))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        valid
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 0, 8))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        valid
            .serialize_specified_with_limits(limits(2, 0))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}
