use surgeist_css::{
    CssErrorCode, CssImportLayer, CssImportTarget, CssMediaQuery, CssRecoveryAction, CssRule,
    CssSupportsConditionKind, parse_sheet,
};

fn import_rule(source: &str) -> surgeist_css::CssParseReport<surgeist_css::CssSheet> {
    let report = parse_sheet(source);
    assert!(
        report
            .syntax()
            .rules()
            .iter()
            .any(|rule| matches!(rule, CssRule::Import(_))),
        "{source}: {:?}",
        report.diagnostics()
    );
    report
}

#[test]
fn import_conditions_and_prelude_phases_follow_cascade() {
    let conditional =
        parse_sheet("@import url(theme.css) layer(theme) supports(display: grid) screen;");
    assert!(conditional.is_clean(), "{:?}", conditional.diagnostics());
    assert!(matches!(conditional.syntax().rules(), [CssRule::Import(_)]));

    let initial_layer = parse_sheet("@layer reset; @import url(theme.css); .after { color: red; }");
    assert!(
        initial_layer.is_clean(),
        "{:?}",
        initial_layer.diagnostics()
    );
    assert!(matches!(
        initial_layer.syntax().rules(),
        [
            CssRule::LayerStatement(_),
            CssRule::Import(_),
            CssRule::Style(_)
        ]
    ));
}

#[test]
fn import_clauses_expose_target_layer_supports_and_media_in_order() {
    let report = import_rule(concat!(
        "@import url(theme.css) layer(theme.components) ",
        "supports((display: grid) and (color: red)) screen;",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Import(import)] = report.syntax().rules() else {
        panic!("expected one import rule")
    };

    assert!(matches!(
        import.target(),
        CssImportTarget::Url(url) if url.as_str() == "theme.css"
    ));
    assert!(matches!(
        import.layer(),
        Some(CssImportLayer::Named(name))
            if name.components() == ["theme", "components"]
    ));
    assert!(matches!(
        import
            .supports()
            .expect("supports clause")
            .condition()
            .kind(),
        CssSupportsConditionKind::And(list) if list.conditions().len() == 2
    ));
    assert!(matches!(
        import.media().expect("media clause").queries(),
        [CssMediaQuery::Typed(_)]
    ));
}

#[test]
fn import_supports_accepts_bare_declarations_and_full_conditions() {
    let bare = import_rule("@import 'bare.css' supports(display: grid);");
    assert!(bare.is_clean());
    let [CssRule::Import(bare)] = bare.syntax().rules() else {
        panic!("expected bare import")
    };
    let CssSupportsConditionKind::Declaration(declaration) =
        bare.supports().expect("bare supports").condition().kind()
    else {
        panic!("expected declaration test")
    };
    assert_eq!(declaration.authored(), "display: grid");
    assert_eq!(declaration.property(), "display");
    assert!(declaration.known().is_some());
    assert!(bare.layer().is_none());
    assert!(bare.media().is_none());

    let full = import_rule("@import 'full.css' supports(not (display: grid)) print;");
    assert!(full.is_clean());
    let [CssRule::Import(full)] = full.syntax().rules() else {
        panic!("expected full-condition import")
    };
    assert!(matches!(
        full.supports().expect("full supports").condition().kind(),
        CssSupportsConditionKind::Not(_)
    ));
    assert!(full.media().is_some());

    let eof = import_rule("@import 'eof.css' supports(display: grid)");
    assert!(eof.is_clean(), "{:?}", eof.diagnostics());
}

#[test]
fn duplicate_swapped_and_trailing_import_clauses_drop_only_the_import() {
    for invalid in [
        "@import 'x.css' layer layer;",
        "@import 'x.css' layer(a) layer(b);",
        "@import 'x.css' supports(display: grid) supports(color: red);",
        "@import 'x.css' supports(display: grid) layer(a);",
        "@import 'x.css' screen supports(display: grid);",
        "@import 'x.css' screen layer(a);",
        "@import 'x.css' supports(not);",
    ] {
        let source = format!("{invalid} .after {{ color: red; }}");
        let report = parse_sheet(&source);
        assert!(
            matches!(report.syntax().rules(), [CssRule::Style(_)]),
            "{invalid}"
        );
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
        assert_eq!(
            report.diagnostics()[0].error().code(),
            CssErrorCode::InvalidAtRulePrelude,
            "{invalid}"
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule,
            "{invalid}"
        );
    }
}

#[test]
fn only_successful_top_level_rules_advance_the_prelude_phase() {
    let recovered = parse_sheet(concat!(
        "@unknown value; ",
        "@layer ; ",
        "@import; ",
        "@import 'kept.css';",
    ));
    assert!(matches!(recovered.syntax().rules(), [CssRule::Import(_)]));
    assert_eq!(recovered.diagnostics().len(), 3);

    let closed = parse_sheet(concat!(
        "@import 'first.css'; ",
        "@layer reset; ",
        "@import 'late.css'; ",
        ".after { color: red; }",
    ));
    assert!(matches!(
        closed.syntax().rules(),
        [
            CssRule::Import(_),
            CssRule::LayerStatement(_),
            CssRule::Style(_)
        ]
    ));
    assert_eq!(closed.diagnostics().len(), 1);
    assert_eq!(
        closed.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRulePlacement
    );

    for body in [
        ".body {}",
        "@layer body {}",
        "@media screen {}",
        "@supports (display: grid) {}",
    ] {
        let source = format!("{body} @import 'late.css';");
        let report = parse_sheet(&source);
        assert_eq!(
            report
                .diagnostics()
                .last()
                .expect("late import")
                .error()
                .code(),
            CssErrorCode::InvalidAtRulePlacement,
            "{body}"
        );
    }
}

#[test]
fn encoding_is_independent_from_import_phase_and_imports_remain_top_level_only() {
    let encoded = parse_sheet(concat!(
        "@charset \"UTF-8\"; ",
        "@layer reset; ",
        "@import 'theme.css';",
    ));
    assert!(encoded.is_clean(), "{:?}", encoded.diagnostics());
    assert_eq!(encoded.syntax().encoding().unwrap().label(), "UTF-8");
    assert!(matches!(
        encoded.syntax().rules(),
        [CssRule::LayerStatement(_), CssRule::Import(_)]
    ));

    let nonleading_encoding = parse_sheet(concat!(
        "@import 'first.css'; ",
        "@charset \"UTF-8\"; ",
        "@import 'second.css';",
    ));
    assert!(matches!(
        nonleading_encoding.syntax().rules(),
        [CssRule::Import(_), CssRule::Import(_)]
    ));
    assert_eq!(nonleading_encoding.diagnostics().len(), 1);
    assert_eq!(
        nonleading_encoding.diagnostics()[0].error().code(),
        CssErrorCode::InvalidEncodingDeclaration
    );

    let nested = parse_sheet("@media screen { @import 'nested.css'; .kept {} }");
    let [CssRule::Media(media)] = nested.syntax().rules() else {
        panic!("expected retained media rule")
    };
    assert!(matches!(media.rules(), [CssRule::Style(_)]));
    assert_eq!(nested.diagnostics().len(), 1);
    assert_eq!(
        nested.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRulePlacement
    );
}

#[test]
fn namespace_and_import_preludes_follow_the_six_state_ordering_contract() {
    let namespace_path = parse_sheet(concat!(
        "@charset \"UTF-8\"; ",
        "@import 'theme.css'; ",
        "@namespace \"urn:default\"; ",
        "@namespace svg url(urn:svg); ",
        "@layer theme; ",
        ".body {} ",
        "@namespace late \"urn:late\";",
    ));
    assert_eq!(namespace_path.syntax().encoding().unwrap().label(), "UTF-8");
    assert!(matches!(
        namespace_path.syntax().rules(),
        [
            CssRule::Import(_),
            CssRule::Namespace(_),
            CssRule::Namespace(_),
            CssRule::LayerStatement(_),
            CssRule::Style(_),
        ]
    ));
    assert_eq!(namespace_path.diagnostics().len(), 1);
    assert_eq!(
        namespace_path.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRulePlacement
    );

    let initial_layers_path = parse_sheet(concat!(
        "@layer reset; ",
        "@layer tokens; ",
        "@import 'first.css'; ",
        "@namespace blocked \"urn:blocked\"; ",
        "@import 'second.css'; ",
        "@layer theme; ",
        "@import 'late.css';",
    ));
    assert!(matches!(
        initial_layers_path.syntax().rules(),
        [
            CssRule::LayerStatement(_),
            CssRule::LayerStatement(_),
            CssRule::Import(_),
            CssRule::Import(_),
            CssRule::LayerStatement(_),
        ]
    ));
    assert_eq!(initial_layers_path.diagnostics().len(), 2);
    assert!(initial_layers_path.diagnostics().iter().all(|diagnostic| {
        diagnostic.error().code() == CssErrorCode::InvalidAtRulePlacement
            && diagnostic.action() == CssRecoveryAction::DropAtRule
    }));
}

#[test]
fn only_successful_namespace_or_layer_rules_change_the_prelude_phase() {
    let report = parse_sheet(concat!(
        "@namespace missing-name; ",
        "@layer ; ",
        "@unknown ignored; ",
        "@namespace kept \"urn:kept\"; ",
        "@namespace replacement \"urn:replacement\";",
    ));

    assert!(matches!(
        report.syntax().rules(),
        [CssRule::Namespace(_), CssRule::Namespace(_)]
    ));
    assert_eq!(report.diagnostics().len(), 3);
    assert_eq!(
        report.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );
    assert_eq!(
        report.diagnostics()[1].error().code(),
        CssErrorCode::UnexpectedToken
    );
    assert_eq!(
        report.diagnostics()[2].error().code(),
        CssErrorCode::UnknownAtRule
    );
}
