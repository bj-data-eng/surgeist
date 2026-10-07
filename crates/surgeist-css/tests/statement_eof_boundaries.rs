#![forbid(unsafe_code)]
//! Existing-callable Syntax 3 §5.4.2 EOF/semicolon boundaries, with the adopted
//! UnexpectedEnd + RetainNonconformingRule diagnostic contract.
use surgeist_css::{
    CssCustomMediaBody, CssErrorCode, CssImportTarget, CssNamespaceContext, CssParseReport,
    CssRecoveryAction, CssRecoveryDiagnostic, CssRule, CssScopedRule, CssSheet, ErrorKind,
    parse_rule, parse_sheet, validate_sheet,
};

fn assert_eof(diagnostic: &CssRecoveryDiagnostic, source: &str, start: usize, end: usize) {
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainNonconformingRule
    );
    let ErrorKind::UnexpectedEnd(detail) = diagnostic.error().kind() else {
        panic!("at-rule EOF")
    };
    assert_eq!(
        detail.expectation().as_str(),
        "a semicolon or block terminating an at-rule"
    );
    assert_eq!(diagnostic.error().position().byte_offset().value(), end);
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(diagnostic.span().end().byte_offset().value(), end);
    // All specimens use one original CRLF before their statement/group. Unicode
    // in the last line contributes UTF-16 units, independently of UTF-8 bytes.
    let last_line = source[..end].rsplit_once("\r\n").unwrap().1;
    assert_eq!(diagnostic.error().position().line().value(), 1);
    assert_eq!(
        diagnostic.error().position().column().value() as usize,
        last_line.encode_utf16().count()
    );
}

fn assert_nonclean_validation(source: &str, report: &CssParseReport<CssSheet>) {
    assert!(!report.is_clean());
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert_eq!(
        report
            .clone()
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

fn assert_ordinary_statement(rule: &CssRule, kind: &str) {
    match (rule, kind) {
        (CssRule::LayerStatement(layer), "layer") => {
            let [name] = layer.names().names() else {
                panic!("one named layer")
            };
            assert_eq!(name.components(), &["café".to_owned()]);
        }
        (CssRule::CustomMedia(custom), "custom-media") => {
            assert_eq!(custom.name().as_str(), "--n");
            assert!(matches!(custom.body(), CssCustomMediaBody::True));
        }
        (CssRule::Import(import), "import") => {
            let CssImportTarget::String(target) = import.target() else {
                panic!("string target")
            };
            assert_eq!(target.as_str(), ";{}");
            assert!(
                import.layer().is_none() && import.supports().is_none() && import.media().is_none()
            );
        }
        (CssRule::Namespace(namespace), "namespace") => {
            assert_eq!(namespace.prefix().unwrap().as_str(), "p");
            assert_eq!(namespace.name().as_str(), "urn:;{}");
        }
        _ => panic!("{kind}: {rule:?}"),
    }
}

fn assert_scoped_statement(rule: &CssScopedRule, kind: &str) {
    match (rule, kind) {
        (CssScopedRule::LayerStatement(layer), "layer") => {
            let [name] = layer.names().names() else {
                panic!("one scoped layer")
            };
            assert_eq!(name.components(), &["café".to_owned()]);
        }
        (CssScopedRule::CustomMedia(custom), "custom-media") => {
            assert_eq!(custom.name().as_str(), "--n");
            assert!(matches!(custom.body(), CssCustomMediaBody::True));
        }
        _ => panic!("{kind}: {rule:?}"),
    }
}

#[test]
fn all_admitted_root_statement_owners_report_eof_but_semicolon_and_trailing_comment_are_clean() {
    let context = CssNamespaceContext::default();
    let cases: Vec<_> = [
        ("@layer café", "layer"),
        ("@custom-media --n true", "custom-media"),
        ("@import ';{}'", "import"),
        ("@namespace p url(\"urn:;{}\")", "namespace"),
    ]
    .into_iter()
    .map(|(statement, kind)| {
        let eof = format!("/*😀*/\r\n{statement} /*tail;{{}}*/ \t");
        let terminated = format!("/*😀*/\r\n{statement}; /*tail;{{}}*/ \t");
        let eof_sheet = parse_sheet(&eof);
        let eof_fragment = parse_rule(&eof, &context);
        let terminated_sheet = parse_sheet(&terminated);
        let terminated_fragment = parse_rule(&terminated, &context);
        (
            eof,
            terminated,
            kind,
            eof_sheet,
            eof_fragment,
            terminated_sheet,
            terminated_fragment,
        )
    })
    .collect();
    // All sixteen fronts above execute before the first missing-diagnostic assertion.
    for (eof, terminated, kind, sheet, fragment, control, fragment_control) in cases {
        let [rule] = sheet.syntax().rules() else {
            panic!("retained {kind}: {sheet:?}")
        };
        assert_ordinary_statement(rule, kind);
        assert_eq!(fragment.syntax().as_ref().unwrap(), rule);
        assert_eq!(fragment.diagnostics(), sheet.diagnostics());
        let [diagnostic] = sheet.diagnostics() else {
            panic!("one admitted-statement EOF: {sheet:?}")
        };
        assert_eof(diagnostic, &eof, eof.find('@').unwrap(), eof.len());
        assert_nonclean_validation(&eof, &sheet);
        assert!(fragment.clone().into_validation_result().is_err());
        assert!(control.is_clean(), "{control:?}");
        assert!(fragment_control.is_clean(), "{fragment_control:?}");
        let [rule] = control.syntax().rules() else {
            panic!("one terminated statement")
        };
        assert_ordinary_statement(rule, kind);
        assert_eq!(fragment_control.syntax().as_ref().unwrap(), rule);
        assert_eq!(validate_sheet(&terminated).unwrap(), *control.syntax());
    }
}

#[test]
fn nested_statement_eof_is_the_parent_input_bound_not_the_complete_source_end() {
    let cases: Vec<_> = [
        ("@media all", "@layer café", "layer"),
        ("@media all", "@custom-media --n true", "custom-media"),
        ("@scope", "@layer café", "layer"),
        ("@scope", "@custom-media --n true", "custom-media"),
        (".host", "@layer café", "layer"),
    ]
    .into_iter()
    .map(|(parent, statement, kind)| {
        let source = format!("/*😀*/\r\n{parent}{{{statement} /*tail;{{}}*/ }}.after{{color:red}}");
        let terminated =
            format!("/*😀*/\r\n{parent}{{{statement}; /*tail;{{}}*/ }}.after{{color:red}}");
        let report = parse_sheet(&source);
        let control = parse_sheet(&terminated);
        (source, terminated, kind, report, control)
    })
    .collect();
    for (source, terminated, kind, report, control) in cases {
        let [parent, CssRule::Style(after)] = report.syntax().rules() else {
            panic!("parent and independent later sibling: {report:?}")
        };
        match parent {
            CssRule::Media(media) => {
                let [statement] = media.rules() else {
                    panic!("ordinary statement")
                };
                assert_ordinary_statement(statement, kind);
            }
            CssRule::Scope(scope) => {
                let [statement] = scope.rules().rules() else {
                    panic!("scoped statement")
                };
                assert_scoped_statement(statement, kind);
            }
            CssRule::Style(style) => {
                let [statement] = style.rules() else {
                    panic!("style-context layer statement")
                };
                assert_ordinary_statement(statement, kind);
            }
            _ => panic!("selected parent"),
        }
        assert_eq!(
            after.position().byte_offset().value(),
            source.find(".after").unwrap()
        );
        assert_eq!(
            after.declarations()[0]
                .parsed_value()
                .unwrap()
                .source()
                .as_str(),
            source
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one bounded statement EOF: {report:?}")
        };
        let start = source
            .find(if kind == "layer" {
                "@layer"
            } else {
                "@custom-media"
            })
            .unwrap();
        let end = source.find("}.after").unwrap();
        assert!(end < source.len());
        assert_eof(diagnostic, &source, start, end);
        assert_nonclean_validation(&source, &report);
        assert!(control.is_clean(), "{control:?}");
        assert_eq!(validate_sheet(&terminated).unwrap(), *control.syntax());
    }
}

#[test]
fn inner_function_block_and_string_terminators_do_not_become_statement_eof() {
    let statement = "@custom-media --n future(\";{}\" [a;b] {c;d} (@hidden;))";
    let eof_source = format!("/*😀*/\r\n{statement}");
    let control_source = format!("/*😀*/\r\n{statement}; /*after;{{}}*/");
    let eof = parse_sheet(&eof_source);
    let control = parse_sheet(&control_source);
    for report in [&eof, &control] {
        let [CssRule::CustomMedia(custom)] = report.syntax().rules() else {
            panic!("one complete function-bearing definition")
        };
        assert_eq!(custom.name().as_str(), "--n");
        let CssCustomMediaBody::Media(media) = custom.body() else {
            panic!("symbolic query body")
        };
        assert!(
            matches!(media.queries(), [surgeist_css::CssMediaQuery::Condition(condition)]
            if matches!(condition.kind(), surgeist_css::CssMediaConditionKind::GeneralEnclosed(_)))
        );
    }
    let [diagnostic] = eof.diagnostics() else {
        panic!("only actual statement EOF is a fault")
    };
    assert_eof(
        diagnostic,
        &eof_source,
        eof_source.find('@').unwrap(),
        eof_source.len(),
    );
    assert_nonclean_validation(&eof_source, &eof);
    assert!(control.is_clean(), "{control:?}");
    assert_eq!(validate_sheet(&control_source).unwrap(), *control.syntax());
}

#[test]
fn discarded_candidates_keep_their_outer_failure_without_retained_statement_or_component_closure() {
    let cases: Vec<_> = [
        ("@UnKnOwN future(", None, CssErrorCode::UnknownAtRule),
        ("@layer 2px", None, CssErrorCode::InvalidAtRulePrelude),
        ("@when media(width)", None, CssErrorCode::InvalidAtRuleBody),
        (
            "@import 'x.css'",
            Some("@scope"),
            CssErrorCode::InvalidAtRulePlacement,
        ),
        (
            "@namespace p 'urn:p'",
            Some("@media all"),
            CssErrorCode::InvalidAtRulePlacement,
        ),
        (
            "@custom-media --n true",
            Some(".host"),
            CssErrorCode::InvalidAtRulePlacement,
        ),
    ]
    .into_iter()
    .map(|(candidate, parent, code)| {
        let source = if let Some(parent) = parent {
            format!("/*😀*/\r\n{parent}{{{candidate}}}.after{{color:red}}")
        } else {
            format!("/*😀*/\r\n{candidate}")
        };
        let report = parse_sheet(&source);
        (source, candidate, parent, code, report)
    })
    .collect();
    for (source, candidate, parent, code, report) in cases {
        if parent.is_none() {
            assert!(report.syntax().rules().is_empty());
        } else {
            let [parent, CssRule::Style(_)] = report.syntax().rules() else {
                panic!("discarding candidate keeps parent/sibling")
            };
            let empty = match parent {
                CssRule::Scope(scope) => scope.rules().rules().is_empty(),
                CssRule::Media(media) => media.rules().is_empty(),
                CssRule::Style(style) => style.rules().is_empty(),
                _ => false,
            };
            assert!(empty);
        }
        let [diagnostic] = report.diagnostics() else {
            panic!("only discarded owner's failure: {report:?}")
        };
        assert_eq!(diagnostic.error().code(), code);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        let start = source.find(candidate).unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + candidate.len()
        );
        assert_nonclean_validation(&source, &report);
    }
}

#[test]
fn real_implicit_closures_and_admitted_statement_eof_have_distinct_owners() {
    let function_source = "/*😀*/\r\n@custom-media --n (color";
    let block_source = "/*😀*/\r\n@layer café{.kept{color:red";
    let function = parse_sheet(function_source);
    let block = parse_sheet(block_source);
    assert!(matches!(
        function.syntax().rules(),
        [CssRule::CustomMedia(_)]
    ));
    let eof: Vec<_> = function
        .diagnostics()
        .iter()
        .filter(|d| d.action() == CssRecoveryAction::RetainNonconformingRule)
        .collect();
    let [diagnostic] = eof.as_slice() else {
        panic!("one returned statement termination fault")
    };
    assert_eof(
        diagnostic,
        function_source,
        function_source.find('@').unwrap(),
        function_source.len(),
    );
    let closures: Vec<_> = function
        .diagnostics()
        .iter()
        .filter(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        .collect();
    assert_eq!(closures.len(), 1, "one actual unmatched query parenthesis");
    assert_eq!(
        function.diagnostics().len(),
        2,
        "distinct one-function closure and one at-rule EOF"
    );
    let [CssRule::LayerBlock(layer)] = block.syntax().rules() else {
        panic!("retained block rule")
    };
    assert!(matches!(layer.rules(), [CssRule::Style(_)]));
    assert_eq!(
        block.diagnostics().len(),
        2,
        "two actual retained unmatched rule braces"
    );
    for diagnostic in block.diagnostics() {
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
    }
    let block_closures: Vec<_> = block.diagnostics().iter().collect();
    for (source, diagnostics) in [
        (function_source, closures.as_slice()),
        (block_source, block_closures.as_slice()),
    ] {
        for diagnostic in diagnostics {
            assert_eq!(
                diagnostic.error().position().byte_offset().value(),
                source.len()
            );
            assert_eq!(
                diagnostic.span().start().byte_offset().value(),
                source.len()
            );
            assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
        }
    }
    assert_nonclean_validation(function_source, &function);
    assert_nonclean_validation(block_source, &block);
}
