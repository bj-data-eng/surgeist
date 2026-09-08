mod common;

use common::CssParseReportTestExt;
use surgeist_css::{
    CssCustomPropertyDeclaredValue, CssCustomPropertyName, CssErrorCode, CssGlobalKeyword,
    CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef, CssRule, CssTokenKind,
    ErrorKind, parse_sheet,
};

fn declarations(source: &str) -> Vec<surgeist_css::CssDeclaration> {
    let sheet = parse_sheet(source).expect("valid authored declaration stylesheet");
    let [CssRule::Style(rule)] = sheet.rules() else {
        panic!("expected one style rule");
    };
    rule.declarations().as_slice().to_vec()
}

fn custom_value(
    declaration: &surgeist_css::CssDeclaration,
) -> &surgeist_css::CssCustomPropertyValue {
    declaration
        .custom()
        .expect("custom declaration")
        .value()
        .value()
        .expect("authored custom value")
}

#[test]
fn authored_declaration_empty_and_whitespace_only_custom_values_are_representable() {
    let declarations = declarations(".x { --empty:; --whitespace: \t  ; }");
    let [empty, whitespace] = declarations.as_slice() else {
        panic!("expected both empty custom declarations");
    };

    for declaration in [empty, whitespace] {
        let value = custom_value(declaration);
        assert_eq!(value.as_css(), "");
        assert!(value.is_empty());
    }
}

#[test]
fn authored_declaration_custom_value_preserves_interior_authored_utf8() {
    let declaration = &declarations(
        ".x { --Theme: /**/ RGB(1,  /**/ 2, VAR(--accent, calc(3px + 4px))), \\72 ed /**/ ; }",
    )[0];
    let value = custom_value(declaration);

    assert_eq!(
        value.as_css(),
        "RGB(1,  /**/ 2, VAR(--accent, calc(3px + 4px))), \\72 ed"
    );
    assert!(!value.is_empty());
}

#[test]
fn authored_declaration_custom_global_keywords_have_distinct_branch_views() {
    let declarations = declarations(
        ".x { --a: inherit; --b: initial; --c: unset; --d: revert; --e: revert-layer; }",
    );
    let expected = [
        CssGlobalKeyword::Inherit,
        CssGlobalKeyword::Initial,
        CssGlobalKeyword::Unset,
        CssGlobalKeyword::Revert,
        CssGlobalKeyword::RevertLayer,
    ];

    for (declaration, expected) in declarations.iter().zip(expected) {
        let value = declaration.custom().expect("custom declaration").value();
        assert_eq!(value.global(), Some(expected));
        assert!(value.value().is_none());
        match value {
            CssCustomPropertyDeclaredValue::Global(actual) => assert_eq!(*actual, expected),
            _ => panic!("expected whole-value global branch"),
        }
    }
}

#[test]
fn authored_declaration_custom_names_use_css_identifier_tokenization() {
    let cases = [
        ("--café", "--café"),
        ("--☃", "--☃"),
        ("--1", "--1"),
        (r"--\65 clair", "--eclair"),
        (r"--bad\ name", "--bad name"),
    ];

    for (authored, decoded) in cases {
        let name = CssCustomPropertyName::try_new(authored)
            .unwrap_or_else(|| panic!("`{authored}` should be a CSS custom-property name"));
        assert_eq!(name.as_str(), decoded);
    }

    for invalid in [
        "name",
        "-name",
        " --name",
        "--",
        "-- bad",
        "--name;",
        "--name extra",
        "--name\n",
        "--\\\n",
    ] {
        assert_eq!(
            CssCustomPropertyName::try_new(invalid),
            None,
            "`{invalid}` must be rejected as a full custom-property-name token"
        );
    }
}

#[test]
fn authored_declaration_parser_and_public_custom_names_share_decoded_identity() {
    let source = ".x { --bad\\ name: 1px; --caf\\e9 : 2px; }";
    let declarations = declarations(source);
    let expected = [
        CssCustomPropertyName::try_new(r"--bad\ name").unwrap(),
        CssCustomPropertyName::try_new("--caf\\e9 ").unwrap(),
    ];

    for (declaration, expected) in declarations.iter().zip(expected) {
        assert_eq!(declaration.custom().unwrap().name(), &expected);
    }

    assert_eq!(declarations[0].position().byte_offset().value(), 5);
    assert_eq!(declarations[0].position().line().value(), 0);
    assert_eq!(declarations[0].position().column().value(), 5);
}

#[test]
fn authored_declaration_known_substitution_preserves_complete_value_between_typed_values() {
    let declarations = declarations(
        ".x { width: 1px; height:  Calc( VAR(--Size, min(2px, 3px)) + 4px ) ; opacity: .5; }",
    );
    let [width, height, opacity] = declarations.as_slice() else {
        panic!("expected typed, substitution-dependent, typed declarations");
    };

    assert_eq!(width.known().unwrap().property(), CssKnownProperty::Width);
    let value = height
        .known()
        .expect("known height")
        .substitution_dependent()
        .expect("expected substitution-dependent height");
    assert_eq!(value.as_css(), "Calc( VAR(--Size, min(2px, 3px)) + 4px )");
    assert_eq!(
        opacity.known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    assert!(matches!(
        height.property_name(),
        CssPropertyNameRef::Known(CssKnownProperty::Height)
    ));
}

#[test]
fn authored_declaration_known_ordinary_wrappers_preserve_interior_css_text() {
    let declarations = declarations(
        ".x { font-family: /**/ \"A\\76 enir\"/**/,/**/ SeRiF /**/ !ImPoRtAnT; \
         transform: TrAnSlAtE(1px/**/, 2px)/**/; }",
    );
    let [family, transform] = declarations.as_slice() else {
        panic!("expected both ordinary declarations");
    };

    let Some(CssKnownPropertyValueRef::FontFamily(value)) =
        family.known().and_then(|known| known.property_value())
    else {
        panic!("expected a font-family wrapper");
    };
    assert_eq!(value.as_css(), "\"A\\76 enir\"/**/,/**/ SeRiF");
    assert_eq!(family.importance(), surgeist_css::CssImportance::Important);

    let Some(CssKnownPropertyValueRef::Transform(value)) =
        transform.known().and_then(|known| known.property_value())
    else {
        panic!("expected a transform wrapper");
    };
    assert_eq!(value.as_css(), "TrAnSlAtE(1px/**/, 2px)");
}

#[test]
fn authored_declaration_substitution_accepts_fallback_unusable_after_substitution() {
    let declarations =
        declarations(".x { color: var(--brand, 8px); all: VAR(--mode, definitely-not-a-global); }");
    let value = declarations[0].known().expect("expected color declaration");
    assert_eq!(
        value.substitution_dependent().unwrap().as_css(),
        "var(--brand, 8px)"
    );
    let value = declarations[1]
        .known()
        .expect("known all")
        .substitution_dependent()
        .expect("expected substitution-dependent all declaration");
    assert_eq!(value.as_css(), "VAR(--mode, definitely-not-a-global)");
}

#[test]
fn authored_declaration_rejects_top_level_semicolon_in_variable_fallback() {
    let source = ".x { width: var(--x, ;); }";
    let error = parse_sheet(source).expect_err("top-level fallback semicolon must be rejected");

    assert_eq!(error.code(), CssErrorCode::InvalidPropertyValue);
    assert_eq!(
        error.position().byte_offset().value(),
        source.find(';').unwrap()
    );
    assert_eq!(error.position().line().value(), 0);
    assert_eq!(
        error.position().column().value(),
        u32::try_from(source.find(';').unwrap()).unwrap()
    );
    let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
        panic!("expected structured property-value error");
    };
    assert_eq!(detail.property(), CssKnownProperty::Width);
    assert_eq!(
        detail.expectation().as_str(),
        "a value accepted by the property's grammar"
    );
    let token = detail.encountered().expect("semicolon token");
    assert_eq!(token.kind(), CssTokenKind::Semicolon);
    assert_eq!(token.authored(), ";");
}

#[test]
fn authored_declaration_rejects_top_level_bang_in_variable_fallback() {
    let source = ".x { width: var(--x, !); }";
    let error = parse_sheet(source).expect_err("top-level fallback bang must be rejected");

    assert_eq!(error.code(), CssErrorCode::InvalidPropertyValue);
    assert_eq!(
        error.position().byte_offset().value(),
        source.find('!').unwrap()
    );
    assert_eq!(error.position().line().value(), 0);
    assert_eq!(
        error.position().column().value(),
        u32::try_from(source.find('!').unwrap()).unwrap()
    );
    let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
        panic!("expected structured property-value error");
    };
    assert_eq!(detail.property(), CssKnownProperty::Width);
    assert_eq!(
        detail.expectation().as_str(),
        "a value accepted by the property's grammar"
    );
    let token = detail.encountered().expect("bang token");
    assert_eq!(token.kind(), CssTokenKind::Delim);
    assert_eq!(token.authored(), "!");
}

#[test]
fn authored_declaration_retains_balanced_fallback_with_nested_restricted_tokens() {
    let declarations = declarations(".x { width: var(--x, fn(; !)); }");
    let value = declarations[0]
        .known()
        .expect("known width")
        .substitution_dependent()
        .expect("expected substitution-dependent width");

    assert_eq!(value.as_css(), "var(--x, fn(; !))");
}

#[test]
fn authored_declaration_rejects_malformed_custom_names_and_nonterminal_globals() {
    for (source, code, byte_offset) in [
        (".x { --: 1px; }", CssErrorCode::UnknownProperty, 5),
        (".x { --bad name: 1px; }", CssErrorCode::UnexpectedToken, 11),
        (
            ".x { --x: inherit 1px; }",
            CssErrorCode::InvalidQualifiedRule,
            18,
        ),
    ] {
        let error = parse_sheet(source).expect_err("invalid custom declaration must fail strictly");
        assert_eq!(error.code(), code, "unexpected error for `{source}`");
        assert_eq!(
            error.position().byte_offset().value(),
            byte_offset,
            "unexpected error position for `{source}`"
        );
    }
}

#[test]
fn authored_declaration_malformed_substitution_and_tokens_remain_structured_errors() {
    let cases = [
        (
            ".x { width: var(); }",
            CssErrorCode::InvalidPropertyValue,
            true,
        ),
        (
            ".x { width: var(color); }",
            CssErrorCode::InvalidPropertyValue,
            true,
        ),
        (
            ".x { width: var(--x --y); }",
            CssErrorCode::InvalidPropertyValue,
            true,
        ),
        (
            ".x { width: \"unterminated; }",
            CssErrorCode::InvalidPropertyValue,
            true,
        ),
        (
            ".x { width: url(\"unterminated); }",
            CssErrorCode::InvalidPropertyValue,
            true,
        ),
        (
            ".x { width: var(--x)); }",
            CssErrorCode::InvalidPropertyValue,
            true,
        ),
        (
            ".x { width: var(--x; 1px); }",
            CssErrorCode::InvalidPropertyValue,
            true,
        ),
        (
            ".x { width: [var(--x); }",
            CssErrorCode::InvalidPropertyValue,
            true,
        ),
        (
            ".x { --x: one; bogus: two; }",
            CssErrorCode::UnknownProperty,
            false,
        ),
    ];

    for (source, code, is_width_value_error) in cases {
        let error = parse_sheet(source).expect_err("malformed authored value must fail strictly");
        assert_eq!(error.code(), code, "unexpected error root for `{source}`");
        match (error.kind(), is_width_value_error) {
            (ErrorKind::InvalidPropertyValue(detail), true) => {
                assert_eq!(detail.property(), CssKnownProperty::Width);
            }
            (ErrorKind::UnknownProperty(detail), false) => {
                assert_eq!(detail.name().as_str(), "bogus");
            }
            _ => panic!("expected structured property-value context for `{source}`"),
        }
    }
}
