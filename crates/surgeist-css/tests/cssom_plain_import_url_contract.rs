#![forbid(unsafe_code)]
//! CSSOM WD20210826 §2.1 URL/string escaping and §6.4 CSSImportRule text.
//! Locations remain decoded authored values; resolution and loading are external.

use std::error::Error;
use surgeist_css::{
    CssImportSerializationError, CssImportTarget, CssMediaCssomSerializationError,
    CssMediaSerializationError, CssNamespaceContext, CssRule, CssRuleCssomSerializationErrorKind,
    CssSerializedOrigin, CssSpecifiedValueSerializationErrorKind as Resource,
    CssSpecifiedValueSerializationLimits as Limits, CssValueOrigin, parse_rule, parse_sheet,
};

fn rule(source: &str) -> CssRule {
    let report = parse_rule(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().clone().expect("one admitted import")
}

fn exact_location(target: &str, decoded: &str, expected: &str) {
    let source = format!("@import {target};");
    let value = rule(&source);
    let before = value.clone();
    let CssRule::Import(import) = &value else {
        panic!("import");
    };
    match import.target() {
        CssImportTarget::String(value) => assert_eq!(value.as_str(), decoded),
        CssImportTarget::Url(value) => assert_eq!(value.as_str(), decoded),
        other => panic!("unexpected target: {other:?}"),
    }
    assert_eq!(import.serialize().unwrap().as_css(), source);
    assert_eq!(value.to_specified_css().unwrap(), source);
    assert_eq!(value.serialize_cssom().unwrap(), expected, "{source}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {report:?}");
    assert_eq!(report.syntax().serialize_cssom().unwrap(), expected);
    assert_eq!(
        value, before,
        "literal output preserves target and provenance"
    );
}

#[test]
fn quoted_and_url_targets_emit_the_same_decoded_cssom_url() {
    for target in [
        "'theme.css'",
        "\"theme.css\"",
        "url(theme.css)",
        "url('theme.css')",
    ] {
        exact_location(target, "theme.css", "@import url(\"theme.css\");");
    }
}

#[test]
fn empty_relative_and_whitespace_locations_are_serialized_without_resolution() {
    for (target, decoded, expected) in [
        ("''", "", "@import url(\"\");"),
        ("url()", "", "@import url(\"\");"),
        (
            "'../theme.css?x=1#frag'",
            "../theme.css?x=1#frag",
            "@import url(\"../theme.css?x=1#frag\");",
        ),
        (
            "' theme file.css '",
            " theme file.css ",
            "@import url(\" theme file.css \");",
        ),
        (
            r"url(theme\20 file.css)",
            "theme file.css",
            "@import url(\"theme file.css\");",
        ),
    ] {
        exact_location(target, decoded, expected);
    }
}

#[test]
fn decoded_quotes_backslashes_controls_and_unicode_use_cssom_string_escaping() {
    for (target, decoded, expected) in [
        (
            r#"'a\22 b\5c c.css'"#,
            "a\"b\\c.css",
            r#"@import url("a\"b\\c.css");"#,
        ),
        (
            r#"'a\a b\9 c\1f d\7f e'"#,
            "a\nb\tc\u{1f}d\u{7f}e",
            r#"@import url("a\a b\9 c\1f d\7f e");"#,
        ),
        (r#"'a\0 b'"#, "a\u{fffd}b", "@import url(\"a\u{fffd}b\");"),
        (
            r#"url('\74 heme.css')"#,
            "theme.css",
            "@import url(\"theme.css\");",
        ),
        (
            "\"café's.css\"",
            "café's.css",
            "@import url(\"café's.css\");",
        ),
    ] {
        exact_location(target, decoded, expected);
    }
}

#[test]
fn optional_media_list_follows_the_cssom_url_before_the_semicolon() {
    for (source, compact, expected) in [
        (
            "@import 'x' PRINT;",
            "@import 'x' print;",
            "@import url(\"x\") print;",
        ),
        (
            "@import 'x' SCREEN, PRINT;",
            "@import 'x' screen, print;",
            "@import url(\"x\") screen, print;",
        ),
        (
            "@import 'x' (COLOR);",
            "@import 'x' (color);",
            "@import url(\"x\") (color);",
        ),
    ] {
        let value = rule(source);
        let before = value.clone();
        let CssRule::Import(import) = &value else {
            panic!("import")
        };
        assert_eq!(import.serialize().unwrap().as_css(), compact);
        assert_eq!(value.to_specified_css().unwrap(), compact);
        assert_eq!(value.serialize_cssom().unwrap(), expected);
        assert_eq!(
            parse_sheet(source).syntax().serialize_cssom().unwrap(),
            expected
        );
        assert_eq!(value, before);
    }
}

#[test]
fn compact_tokens_keep_original_coordinates_and_modern_clause_structure() {
    let source = "/* prefix */ @IMPORT 'x';";
    let report = parse_sheet(source);
    let [CssRule::Import(import)] = report.syntax().rules() else {
        panic!("import")
    };
    let output = import.serialize().unwrap();
    assert_eq!(output.as_css(), "@import 'x';");
    for (offset, original) in [(0, 13), (8, 21), (11, 24)] {
        let Some(CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin))) =
            output.origin_at(offset)
        else {
            panic!("original token at {offset}");
        };
        assert_eq!(origin.span().start().byte_offset().value(), original);
        assert_eq!(origin.source().as_str(), source);
    }
    for source in [
        "@import 'x' layer(theme) supports(display:grid) print;",
        "@import url(x) layer supports(display:grid) print;",
    ] {
        let value = rule(source);
        let before = value.clone();
        assert_eq!(value.to_specified_css().unwrap(), source);
        assert_eq!(value.serialize_cssom().unwrap(), source);
        assert_eq!(value, before);
    }
}

#[test]
fn literal_url_bytes_and_retained_nodes_share_rule_budgets_atomically() {
    let value = rule("@import 'é';");
    let before = value.clone();
    let expected = "@import url(\"é\");";
    // The public rule owns one node; its retained string target owns one node.
    for (limits, kind) in [
        (Limits::new(1, 2, expected.len()), Resource::InputNodeLimit),
        (
            Limits::new(2, 1, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(2, 2, expected.len() - 1), Resource::ByteLimit),
    ] {
        let error = value.serialize_cssom_with_limits(limits).unwrap_err();
        assert_eq!(
            error.kind(),
            CssRuleCssomSerializationErrorKind::Resource(kind)
        );
        assert_eq!(error.rule_path(), &[]);
        assert_eq!(value, before);
    }
    assert_eq!(
        value
            .serialize_cssom_with_limits(Limits::new(2, 2, expected.len()))
            .unwrap(),
        expected
    );
}

#[test]
fn literal_import_sheet_charges_cumulative_nodes_separator_and_url_bytes() {
    let report = parse_sheet("@import 'a';@import 'b';");
    let before = report.clone();
    let expected = "@import url(\"a\");\n@import url(\"b\");";
    // Sheet aggregate1 + two (rule1 + string target1), including the LF byte.
    for (limits, kind) in [
        (Limits::new(4, 5, expected.len()), Resource::InputNodeLimit),
        (
            Limits::new(5, 4, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(5, 5, expected.len() - 1), Resource::ByteLimit),
    ] {
        let error = report
            .syntax()
            .serialize_cssom_with_limits(limits)
            .unwrap_err();
        assert_eq!(
            error.kind(),
            CssRuleCssomSerializationErrorKind::Resource(kind)
        );
        assert_eq!(error.rule_path(), &[1]);
        assert_eq!(report, before);
    }
    assert_eq!(
        report
            .syntax()
            .serialize_cssom_with_limits(Limits::new(5, 5, expected.len()))
            .unwrap(),
        expected
    );
}

#[test]
fn recovered_media_keeps_provider_failure_origin_and_retry_without_mutation() {
    let report = parse_sheet("@import 'x' screen,???;");
    assert!(!report.is_clean());
    let before = report.clone();
    let [value @ CssRule::Import(import)] = report.syntax().rules() else {
        panic!("import")
    };
    assert!(matches!(
        import.serialize(),
        Err(CssImportSerializationError::Media(
            CssMediaSerializationError::RecoveredNever { .. }
        ))
    ));
    let origin = import.media().unwrap().queries()[1].origin();
    // Rule1 + target1 + list1 + typed query1 + type token1 + Never1;
    // the recovered query's `not all` adds two projection nodes.
    let error = value
        .serialize_cssom_with_limits(Limits::new(6, 7, 128))
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CssRuleCssomSerializationErrorKind::Resource(Resource::ProjectionNodeLimit)
    );
    assert_eq!(error.rule_path(), &[]);
    let provider = error
        .source()
        .unwrap()
        .downcast_ref::<CssMediaCssomSerializationError>()
        .expect("media provider error");
    assert_eq!(provider.origin(), origin);
    assert_eq!(report, before);
    assert_eq!(
        value
            .serialize_cssom_with_limits(Limits::new(6, 8, 128))
            .unwrap(),
        "@import url(\"x\") screen, not all;"
    );
    assert_eq!(report, before);
}
