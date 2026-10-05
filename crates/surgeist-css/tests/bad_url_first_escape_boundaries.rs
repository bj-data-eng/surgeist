#![forbid(unsafe_code)]
//! CSS Syntax 3 §4.3.6 leaves the first non-whitespace input for the bad-URL
//! remnants algorithm. Section 4.3.14 consumes valid escapes before recognizing
//! an unescaped closing parenthesis; the first escape cannot be discarded.

use surgeist_css::{
    CssComponentValueErrorKind, CssComponentValueRef, CssDeclarationList, CssErrorCode,
    CssRecoveryAction, CssRecoveryDiagnostic, CssRule, CssSupportsTestItem, CssTokenKind,
    CssValueOrigin, CssValueTokenRef, ErrorKind, parse_component_values, parse_sheet,
    parse_style_attribute, validate_style_attribute,
};

fn assert_bad_url_range(source: &str, end: usize) {
    let error = parse_component_values(source).expect_err("invalid trailing URL content");
    assert_eq!(error.kind(), CssComponentValueErrorKind::BadUrl);
    let CssValueOrigin::Parsed(origin) = error.origin() else {
        panic!("bad URL retains its genuine authored source");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), end);
}

#[test]
fn first_post_whitespace_escape_keeps_the_escaped_closer_inside_bad_url_remnants() {
    let source = r"url(a \)still)";
    assert_bad_url_range(source, source.len());
}

#[test]
fn first_post_whitespace_backslash_pair_keeps_the_literal_closer_as_the_boundary() {
    let source = r"url(a \\)tail";
    assert_bad_url_range(source, 9);
}

#[test]
fn preceding_nonmember_and_quote_controls_keep_their_unescaped_final_boundary() {
    for source in [r"url(a b\)still)", "url(a \"x)"] {
        assert_bad_url_range(source, source.len());
    }
}

fn assert_bad_url_origin(source: &str, range: std::ops::Range<usize>) {
    let error = parse_component_values(source).expect_err("bad URL rejects the component sequence");
    assert_eq!(error.kind(), CssComponentValueErrorKind::BadUrl);
    let CssValueOrigin::Parsed(origin) = error.origin() else {
        panic!("original bad-URL origin");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), range.start);
    assert_eq!(origin.span().end().byte_offset().value(), range.end);
}

fn assert_custom_values(declarations: &CssDeclarationList, expected: &[(&str, &str)]) {
    assert_eq!(
        declarations.len(),
        expected.len(),
        "later declarations must survive"
    );
    for (declaration, (name, value)) in declarations.iter().zip(expected) {
        let custom = declaration.custom().expect("retained custom declaration");
        assert_eq!(custom.name().as_str(), *name);
        assert_eq!(custom.value().value().unwrap().as_css(), *value);
    }
}

fn assert_bad_url_diagnostic(
    diagnostic: &CssRecoveryDiagnostic,
    authored: &str,
    position: (usize, u32, u32),
    unit: std::ops::Range<usize>,
) {
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedToken);
    let ErrorKind::UnexpectedToken(detail) = diagnostic.error().kind() else {
        panic!("custom bad-token diagnostic");
    };
    assert_eq!(detail.encountered().kind(), CssTokenKind::BadUrl);
    assert_eq!(
        detail.encountered().authored(),
        authored,
        "complete original bad token"
    );
    let actual = diagnostic.error().position();
    assert_eq!(actual.byte_offset().value(), position.0);
    assert_eq!(actual.line().value(), position.1);
    assert_eq!(actual.column().value(), position.2);
    assert_eq!(diagnostic.span().start().byte_offset().value(), unit.start);
    assert_eq!(diagnostic.span().end().byte_offset().value(), unit.end);
}

fn assert_bad_declaration_keeps_after(bad_url: &str, suffix: &str) {
    let discarded = format!("--bad:{bad_url}{suffix}");
    let source = format!("{discarded}--after:kept");
    let report = parse_style_attribute(&source);
    assert_custom_values(report.syntax(), &[("--after", "kept")]);
    let [diagnostic] = report.diagnostics() else {
        panic!("one bad URL, no extra lexical diagnostics: {report:?}");
    };
    assert_bad_url_diagnostic(diagnostic, bad_url, (6, 0, 6), 0..discarded.len());
    assert_eq!(
        validate_style_attribute(&source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert_bad_url_origin(&source, 6..6 + bad_url.len());
}

#[test]
fn odd_and_even_first_escape_runs_preserve_following_declarations() {
    // The odd cases consume an escaped ')' plus the later unescaped ')'. The
    // even cases end at the first ')' and leave "tail" outside the token.
    for bad_url in [
        r"url(a \)still)",
        r"url(a \\)",
        r"url(a \\\)still)",
        r"url(a \\\\)",
    ] {
        assert_bad_declaration_keeps_after(bad_url, "tail;");
    }
}

#[test]
fn odd_and_even_first_escape_runs_preserve_following_rules() {
    for bad_url in [
        r"url(a \)still)",
        r"url(a \\)",
        r"url(a \\\)still)",
        r"url(a \\\\)",
    ] {
        let bad_declaration = format!("--bad:{bad_url}tail;");
        let source = format!(".bad{{{bad_declaration}}}.after{{--after:kept}}");
        let report = parse_sheet(&source);
        let [CssRule::Style(before), CssRule::Style(after)] = report.syntax().rules() else {
            panic!("both surrounding style rules survive: {report:?}");
        };
        assert!(before.declarations().is_empty());
        assert_custom_values(after.declarations(), &[("--after", "kept")]);
        let [diagnostic] = report.diagnostics() else {
            panic!(
                "one dropped declaration, no spurious structural or escape recovery: {report:?}"
            );
        };
        assert_bad_url_diagnostic(
            diagnostic,
            bad_url,
            (11, 0, 11),
            5..5 + bad_declaration.len(),
        );
    }
}

#[test]
fn bad_url_boundaries_preserve_original_utf8_crlf_coordinates_and_spelling() {
    let source = "--prefix:😀;--bad:url(a \r\n\\)still);--after:kept";
    let bad_url = "url(a \r\n\\)still)";
    let report = parse_style_attribute(source);
    assert_custom_values(report.syntax(), &[("--prefix", "😀"), ("--after", "kept")]);
    let [diagnostic] = report.diagnostics() else {
        panic!("one original bad-URL diagnostic");
    };
    assert_bad_url_diagnostic(diagnostic, bad_url, (20, 0, 18), 14..37);
    assert_eq!(diagnostic.span().start().column().value(), 12);
    assert_eq!(diagnostic.span().end().line().value(), 1);
    assert_eq!(diagnostic.span().end().column().value(), 9);
    assert_eq!(&source[20..36], bad_url);
    assert_bad_url_origin(source, 20..36);
}

#[test]
fn escaped_url_names_and_prefix_hex_whitespace_keep_the_same_recovery_boundary() {
    for bad_url in [
        r"u\72 l(a \)still)",
        r"u\72 l(a \\)",
        r"url(\61  \)still)",
        r"url(\000061  \\)",
    ] {
        assert_bad_declaration_keeps_after(bad_url, "tail;");
    }
}

#[test]
fn nested_function_collectors_keep_the_actual_bad_url_source_range() {
    for bad_url in [r"url(a \)still)", r"url(a \\)"] {
        let source = format!("outer({bad_url} tail)");
        assert_bad_url_origin(&source, 6..6 + bad_url.len());
        let value = format!("outer({bad_url} tail)");
        let discarded = format!("--bad:{value};");
        let source = format!("{discarded}--after:kept");
        let report = parse_style_attribute(&source);
        assert_custom_values(report.syntax(), &[("--after", "kept")]);
        let [diagnostic] = report.diagnostics() else {
            panic!("one bad nested URL");
        };
        assert_bad_url_diagnostic(diagnostic, bad_url, (12, 0, 12), 0..discarded.len());
    }
}

#[test]
fn recovering_named_tests_preserve_declaration_neighbors_and_bad_token_origins() {
    let prefix = "@supports-condition --probe{head:yes;bad:";
    for bad_url in [r"url(a \)still)", r"url(a \\)"] {
        let source = format!("{prefix}{bad_url}tail;tail:yes;}}");
        let report = parse_sheet(&source);
        let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
            panic!("retained named definition: {report:?}");
        };
        let names = rule
            .body()
            .items()
            .iter()
            .flat_map(|item| match item {
                CssSupportsTestItem::Declarations(run) => run
                    .declarations()
                    .iter()
                    .map(|declaration| declaration.property())
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            })
            .collect::<Vec<_>>();
        assert_eq!(names, ["head", "tail"]);
        let [diagnostic] = report.diagnostics() else {
            panic!("one bad token, no payload escape event: {report:?}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let ErrorKind::InvalidComponentValue(error) = diagnostic.error().kind() else {
            panic!("named collector owns its typed bad token");
        };
        assert_eq!(error.kind(), CssComponentValueErrorKind::BadUrl);
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("original bad-URL origin");
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(origin.span().start().byte_offset().value(), prefix.len());
        assert_eq!(
            origin.span().end().byte_offset().value(),
            prefix.len() + bad_url.len()
        );
        assert_eq!(diagnostic.span(), origin.span());
    }
}

#[test]
fn quote_and_comment_looking_remnants_end_at_their_first_unescaped_closer() {
    // A quoted/comment token read after the provider's premature ')' would skip
    // the real boundary. These bytes remain bad-URL remnants until that ')'.
    for (bad_url, suffix) in [
        (r#"url(a \)"x)"#, r#""tail";"#),
        (r"url(a \)/*x)", "tail*/;"),
    ] {
        assert_bad_declaration_keeps_after(bad_url, suffix);
    }
}

#[test]
fn adjacent_bad_urls_report_once_each_without_swallowing_middle_or_final_values() {
    let first_url = r"url(a \\)";
    let second_url = r"url(a \)still)";
    let first = format!("--one:{first_url};");
    let middle = "--middle:kept;";
    let second = format!("--two:{second_url};");
    let source = format!("{first}{middle}{second}--after:kept");
    let report = parse_style_attribute(&source);
    assert_custom_values(
        report.syntax(),
        &[("--middle", "kept"), ("--after", "kept")],
    );
    let [one, two] = report.diagnostics() else {
        panic!("two independently ordered bad-URL events: {report:?}");
    };
    assert_bad_url_diagnostic(one, first_url, (6, 0, 6), 0..first.len());
    let start = first.len() + middle.len();
    assert_bad_url_diagnostic(
        two,
        second_url,
        (start + 6, 0, (start + 6) as u32),
        start..start + second.len(),
    );
}

#[test]
fn eof_remnants_stay_inside_the_bad_token_without_extra_escape_or_closure_events() {
    for bad_url in [r"url(a \)still", "url(a \\", r"url(a b\)still"] {
        assert_bad_url_range(bad_url, bad_url.len());
        let source = format!("--bad:{bad_url}");
        let report = parse_style_attribute(&source);
        assert!(report.syntax().is_empty());
        let [diagnostic] = report.diagnostics() else {
            panic!("bad URL owns EOF recovery: {report:?}");
        };
        assert_bad_url_diagnostic(diagnostic, bad_url, (6, 0, 6), 0..source.len());
    }
}

#[test]
fn valid_url_escape_and_hex_terminator_whitespace_controls_remain_clean() {
    for (source, expected) in [
        (r"url(a\)still)", "a)still"),
        (r"url(a\\)", "a\\"),
        (r"url(\61 \)still)", "a)still"),
        (r"url( \000061 \)still)", "a)still"),
        (r"url(a\29 still)", "a)still"),
    ] {
        let values = parse_component_values(source).unwrap();
        let [component] = values.items() else {
            panic!("one complete valid URL");
        };
        assert!(
            matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Url(actual)) if actual == expected)
        );
        let attribute = format!("--url:{source};--after:kept");
        let report = parse_style_attribute(&attribute);
        assert!(report.is_clean(), "{attribute:?}: {report:?}");
        assert_custom_values(report.syntax(), &[("--url", source), ("--after", "kept")]);
    }
    let quoted = r#"URL("a \)still")"#;
    let values = parse_component_values(quoted).unwrap();
    let [component] = values.items() else {
        panic!("one quoted URL function");
    };
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("quoted URL dispatch");
    };
    assert_eq!(function.name(), "URL");
    let [argument] = function.values().items() else {
        panic!("one quoted argument");
    };
    assert!(matches!(
        argument.view(),
        CssComponentValueRef::Token(CssValueTokenRef::String("a )still"))
    ));
    for bad_url in [r"url(a \29 )", r"url(a b\)still)", "url(a \"x)"] {
        assert_bad_declaration_keeps_after(bad_url, "tail;");
    }
}
