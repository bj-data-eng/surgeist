#![forbid(unsafe_code)]
//! Nested rule-list context remains semantic across bounded structural replay.
use surgeist_css::{
    CssErrorCode, CssRecoveryAction, CssRule, CssSelector, CssSheet, CssTokenKind, ErrorKind,
    parse_sheet, validate_sheet,
};

fn assert_style(rule: &CssRule, source: &str, class: &str, value: &str) {
    let CssRule::Style(style) = rule else {
        panic!("retained style {class}")
    };
    let [selector] = style.selectors().selectors() else {
        panic!("one style selector")
    };
    assert_eq!(selector.selector(), &CssSelector::Class(class.into()));
    assert_eq!(
        style.position().byte_offset().value(),
        source.find(&format!(".{class}{{")).unwrap()
    );
    let [declaration] = style.declarations().as_slice() else {
        panic!("one retained declaration")
    };
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        value
    );
    assert_eq!(
        declaration.parsed_value().unwrap().source().as_str(),
        source
    );
}

fn innermost_rules(sheet: &CssSheet, depth: usize, max_depth: usize) -> &[CssRule] {
    let [CssRule::Media(first), CssRule::Style(_)] = sheet.rules() else {
        panic!(
            "enclosing ordinary chain and root sibling at Media depth {depth}, maximum structural depth {max_depth}"
        )
    };
    let mut parent = first;
    for level in 1..depth {
        let [CssRule::Media(nested)] = parent.rules() else {
            panic!(
                "missing Media at level {}, expected enclosing depth {depth}, maximum structural depth {max_depth}",
                level + 1
            )
        };
        parent = nested;
    }
    parent.rules()
}

fn assert_nested_siblings(sheet: &CssSheet, source: &str, depth: usize, max_depth: usize) {
    let rules = innermost_rules(sheet, depth, max_depth);
    let [kept] = rules else {
        panic!(
            "failed unit must leave only its independent later child at Media depth {depth}, maximum structural depth {max_depth}; got {} retained children",
            rules.len()
        )
    };
    assert_style(kept, source, "kept", "blue");
    assert_style(&sheet.rules()[1], source, "after", "red");
}

fn html_unit(html: &str, body_media_depth: usize) -> String {
    format!(
        "{html}@supports (color:red){{{}.hidden{{color:red}}{}}}",
        "@media all{".repeat(body_media_depth),
        "}".repeat(body_media_depth)
    )
}

fn check_html_token(html: &str, kind: CssTokenKind) {
    // 63 ordinary ancestors put Supports at the first selected chunk (64).
    // Its simple body gives maximum depth 65; 64 body Media groups give 129.
    // One ancestor is a shallow semantic control for the identical grammar.
    let cases: Vec<_> = [(1, 0), (63, 0), (63, 64)]
        .into_iter()
        .map(|(ancestors, body_media_depth)| {
            let unit = html_unit(html, body_media_depth);
            let source = format!(
                "/*😀*/\r\n{}{unit}.kept{{color:blue}}{}.after{{color:red}}",
                "@media all{".repeat(ancestors),
                "}".repeat(ancestors)
            );
            let report = parse_sheet(&source);
            (
                ancestors,
                ancestors + body_media_depth + 2,
                unit,
                source,
                report,
            )
        })
        .collect();

    // Run the genuine root control before any nested assertion can fail.
    // At the real stylesheet root only, both HTML tokens are discarded.
    let source = format!("{} .kept{{color:blue}}", html_unit(html, 0));
    let report = parse_sheet(&source);
    assert!(
        report.is_clean(),
        "root control: {:?}",
        report.diagnostics()
    );
    let [CssRule::Supports(supports), kept] = report.syntax().rules() else {
        panic!("root HTML token permits a real Supports rule")
    };
    let [hidden] = supports.rules() else {
        panic!("root Supports retains its body")
    };
    assert_style(hidden, &source, "hidden", "red");
    assert_style(kept, &source, "kept", "blue");
    assert_eq!(validate_sheet(&source).unwrap(), *report.syntax());

    for (ancestors, max_depth, unit, source, report) in cases {
        assert_nested_siblings(report.syntax(), &source, ancestors, max_depth);
        let [diagnostic] = report.diagnostics() else {
            panic!(
                "one CDO/CDC-started failed qualified unit at Media depth {ancestors}, maximum structural depth {max_depth}"
            )
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        let start = source.find(html).unwrap();
        assert_eq!(diagnostic.error().position().byte_offset().value(), start);
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + unit.len()
        );
        let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
            panic!("selector owns the qualified prelude")
        };
        let token = detail.encountered().unwrap();
        assert_eq!(token.kind(), kind);
        assert_eq!(token.authored(), html);
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn cdo_before_supports_stays_in_one_nested_qualified_unit_across_replay() {
    check_html_token("<!--", CssTokenKind::Cdo);
}

#[test]
fn cdc_before_supports_stays_in_one_nested_qualified_unit_across_replay() {
    check_html_token("-->", CssTokenKind::Cdc);
}

fn check_root_only_statement(statement: &str, name: &str) {
    let cases: Vec<_> = [1, 65, 129]
        .into_iter()
        .map(|depth| {
            let source = format!(
                "/*😀*/\r\n{}{statement}.kept{{color:blue}}{}.after{{color:red}}",
                "@media all{".repeat(depth),
                "}".repeat(depth)
            );
            let report = parse_sheet(&source);
            (depth, source, report)
        })
        .collect();
    for (depth, source, report) in cases {
        assert_nested_siblings(report.syntax(), &source, depth, depth + 1);
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejected nested root-only statement")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidAtRulePlacement
        );
        let start = source.find(statement).unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + statement.len()
        );
        let ErrorKind::InvalidAtRulePlacement(detail) = diagnostic.error().kind() else {
            panic!("nested placement owns rejection")
        };
        assert_eq!(detail.name().as_str(), name);
        assert_eq!(
            detail.expected_context().as_str(),
            "the stylesheet top level"
        );
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }

    // The same valid spelling is admitted in the actual initial root phase.
    let source = format!("{statement}.after{{color:red}}");
    let report = parse_sheet(&source);
    assert!(
        report.is_clean(),
        "root control: {:?}",
        report.diagnostics()
    );
    let [statement, after] = report.syntax().rules() else {
        panic!("root statement and body rule")
    };
    match (name, statement) {
        ("import", CssRule::Import(import)) => {
            assert!(matches!(
                import.target(),
                surgeist_css::CssImportTarget::String(target) if target.as_str() == "theme.css"
            ));
        }
        ("namespace", CssRule::Namespace(namespace)) => {
            assert_eq!(namespace.prefix().unwrap().as_str(), "p");
            assert_eq!(namespace.name().as_str(), "urn:p");
        }
        _ => panic!("context-valid typed root owner"),
    }
    assert_style(after, &source, "after", "red");
    assert_eq!(validate_sheet(&source).unwrap(), *report.syntax());
}

#[test]
fn nested_import_remains_misplaced_at_shallow_and_replayed_depths() {
    check_root_only_statement("@import 'theme.css';", "import");
}

#[test]
fn nested_namespace_remains_misplaced_at_shallow_and_replayed_depths() {
    check_root_only_statement("@namespace p 'urn:p';", "namespace");
}
