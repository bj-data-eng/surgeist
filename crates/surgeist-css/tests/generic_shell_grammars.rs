use surgeist_css::{
    CssErrorCode, CssFeatureKind, CssRecoveryAction, CssRule, CssSupportStatus, ErrorKind,
    feature_metadata, parse_sheet, parse_style_attribute,
};

fn assert_span(source: &str, diagnostic: &surgeist_css::CssRecoveryDiagnostic, authored: &str) {
    let start = source
        .find(authored)
        .unwrap_or_else(|| panic!("missing authored recovery unit `{authored}`"));
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + authored.len()
    );
}

#[test]
fn c14_generic_authored_shells_retain_structure() {
    let source = concat!(
        "@media screen { .inside { color: red; } } ",
        ".😀 { --tone: blue; width: 2px !important; }",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Media(media), CssRule::Style(non_bmp)] = report.syntax().rules() else {
        panic!("expected an at-rule and qualified-rule stylesheet");
    };
    assert_eq!(media.position().byte_offset().value(), 0);
    let [CssRule::Style(inside)] = media.rules() else {
        panic!("expected the retained nested rule-list member");
    };
    assert_eq!(
        inside.position().byte_offset().value(),
        source.find(".inside").expect("nested qualified rule")
    );
    assert_eq!(inside.declarations().len(), 1);
    assert_eq!(
        inside.declarations()[0].position().byte_offset().value(),
        source.find("color").expect("nested declaration")
    );

    let non_bmp_start = source.find(".😀").expect("non-BMP style block");
    assert_eq!(non_bmp.position().byte_offset().value(), non_bmp_start);
    assert_eq!(
        non_bmp.position().column().value(),
        source[..non_bmp_start].encode_utf16().count() as u32
    );
    assert_eq!(non_bmp.declarations().len(), 2);
    assert!(non_bmp.declarations()[0].custom().is_some());
    assert_eq!(
        non_bmp.declarations()[1]
            .known()
            .expect("known width declaration")
            .property()
            .canonical_name(),
        "width"
    );

    for (id, kind, spelling, production) in [
        (
            "official.rule.at-rule",
            CssFeatureKind::Rule,
            "generic at-rule",
            "#at-rules,#consume-at-rule",
        ),
        (
            "official.qualified-rule.generic",
            CssFeatureKind::Rule,
            "generic qualified rule",
            "#consume-qualified-rule",
        ),
        (
            "official.declaration.generic",
            CssFeatureKind::Declaration,
            "generic declaration",
            "#consume-declaration",
        ),
        (
            "official.value.stylesheet",
            CssFeatureKind::Value,
            "<stylesheet>",
            "#parser-entry-points",
        ),
        (
            "official.value.rule-list",
            CssFeatureKind::Value,
            "<rule-list>",
            "#declaration-rule-list",
        ),
        (
            "official.value.declaration-list",
            CssFeatureKind::Value,
            "<declaration-list>",
            "#declaration-rule-list",
        ),
        (
            "official.value.style-block",
            CssFeatureKind::Value,
            "<style-block>",
            "#declaration-rule-list",
        ),
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
        assert_eq!(metadata.kind(), kind, "{id}");
        assert_eq!(metadata.spelling(), spelling, "{id}");
        assert_eq!(metadata.source().id().as_str(), "O-SYNTAX3", "{id}");
        assert_eq!(metadata.production(), production, "{id}");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id}");
        assert_eq!(metadata.supported_subset(), None, "{id}");
        assert_eq!(metadata.unsupported_remainder(), None, "{id}");
        assert_eq!(metadata.recognized_unsupported_code(), None, "{id}");
    }
}

#[test]
fn generic_consumers_keep_unknown_and_recognized_unsupported_at_rules_distinct() {
    let source = concat!(
        ".before { color: red; } ",
        "@mystery fn({x; y}); ",
        "@font-feature-values Demo { @styleset { nice: 1; } } ",
        ".after { width: 2px; }",
    );
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::Style(_), CssRule::Style(_)]
    ));
    let [unknown, unsupported] = report.diagnostics() else {
        panic!("expected unknown and recognized-unsupported diagnostics");
    };

    assert_eq!(unknown.error().code(), CssErrorCode::UnknownAtRule);
    assert_eq!(unknown.action(), CssRecoveryAction::DropAtRule);
    assert_span(source, unknown, "@mystery fn({x; y});");
    let ErrorKind::UnknownAtRule(detail) = unknown.error().kind() else {
        panic!("expected unknown at-rule detail");
    };
    assert_eq!(detail.name().as_str(), "mystery");

    assert_eq!(unsupported.error().code(), CssErrorCode::UnsupportedAtRule);
    assert_eq!(unsupported.action(), CssRecoveryAction::DropAtRule);
    assert_span(
        source,
        unsupported,
        "@font-feature-values Demo { @styleset { nice: 1; } }",
    );
    let ErrorKind::UnsupportedAtRule(detail) = unsupported.error().kind() else {
        panic!("expected recognized-unsupported at-rule detail");
    };
    assert_eq!(detail.name().as_str(), "font-feature-values");
    assert_eq!(detail.feature().as_str(), "later.rule.font-feature-values");
}

#[test]
fn authored_lists_recover_repeated_failures_and_retain_valid_siblings() {
    let source = concat!(
        ".before { color: red; mystery: 1; width: 1px; } ",
        "@unknown value; ",
        "??? { height: 3px; } ",
        ".after { color: blue; }",
    );
    let report = parse_sheet(source);
    let [CssRule::Style(before), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("expected both valid qualified-rule siblings");
    };
    assert_eq!(before.declarations().len(), 2);
    assert_eq!(after.declarations().len(), 1);

    let [declaration, at_rule, qualified_rule] = report.diagnostics() else {
        panic!("expected three independently recovered failures");
    };
    assert_eq!(declaration.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(declaration.action(), CssRecoveryAction::DropDeclaration);
    assert_span(source, declaration, "mystery: 1;");
    assert_eq!(at_rule.error().code(), CssErrorCode::UnknownAtRule);
    assert_eq!(at_rule.action(), CssRecoveryAction::DropAtRule);
    assert_span(source, at_rule, "@unknown value;");
    assert_eq!(qualified_rule.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(
        qualified_rule.action(),
        CssRecoveryAction::DropQualifiedRule
    );
    assert_span(source, qualified_rule, "??? { height: 3px; }");

    let style_source = "color: red; broken; width: 2px";
    let style = parse_style_attribute(style_source);
    assert_eq!(style.syntax().len(), 2);
    let [diagnostic] = style.diagnostics() else {
        panic!("expected one generic declaration recovery");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_span(style_source, diagnostic, "broken;");
}

#[test]
fn generic_shell_eof_closure_preserves_non_bmp_coordinates() {
    let source = ".😀 { color: red;";
    let report = parse_sheet(source);
    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("expected implicitly closed style rule");
    };
    assert_eq!(rule.declarations().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one EOF closure diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.len()
    );
    assert_eq!(
        diagnostic.error().position().column().value(),
        source.encode_utf16().count() as u32
    );
    assert_eq!(diagnostic.span().start(), diagnostic.span().end());
    assert_eq!(diagnostic.span().end(), diagnostic.error().position());
}

#[cfg(feature = "app-strict")]
#[test]
fn generic_shell_strict_entry_points_match_ordinary_reports() {
    for source in [
        "@media screen { .inside { color: red; } } .after { width: 2px; }",
        ".before { color: red; } @unknown x; ??? { width: 1px; } .after { color: blue; }",
        ".😀 { color: red;",
    ] {
        let ordinary = parse_sheet(source);
        let strict = surgeist_css::validate_sheet(source);
        if ordinary.is_clean() {
            assert_eq!(strict, Ok(ordinary.syntax().clone()), "{source}");
        } else {
            let failure = strict.expect_err("recovered stylesheet must fail strict validation");
            assert_eq!(failure.diagnostics(), ordinary.diagnostics(), "{source}");
        }
    }

    for source in ["color: red; width: 2px", "color: red; broken; width: 2px"] {
        let ordinary = parse_style_attribute(source);
        let strict = surgeist_css::validate_style_attribute(source);
        if ordinary.is_clean() {
            assert_eq!(strict, Ok(ordinary.syntax().clone()), "{source}");
        } else {
            let failure =
                strict.expect_err("recovered declaration list must fail strict validation");
            assert_eq!(failure.diagnostics(), ordinary.diagnostics(), "{source}");
        }
    }
}
