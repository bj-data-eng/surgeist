#![forbid(unsafe_code)]
//! Failed complete rules partition pending declaration lists, while a no-rule
//! result preserves the list. The first nonempty transfer occupies the leading
//! slot even when a rejected child leaves no rule in the retained tree.
//! https://github.com/w3c/csswg-drafts/blob/f971255463f01fb740e2a3a7ecfe83e319cddab9/css-syntax-3/Overview.bs
use surgeist_css::*;

fn assert_color(declaration: &CssDeclaration, expected: &str) {
    assert_eq!(
        declaration.known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(
        declaration
            .value_components()
            .serialize()
            .unwrap()
            .as_css()
            .trim(),
        expected
    );
}

#[test]
fn identifier_fallback_and_at_keyword_callbacks_partition_once_per_nonempty_run() {
    for rejected in ["bad,{}", "@unknown;", "@\\75 nknown{}", "@layer a,b{}"] {
        let source = format!(".p{{color:red;{rejected}@unknown; color:green}}tail{{}}");
        let report = parse_sheet(&source);
        assert_eq!(
            report.diagnostics().len(),
            2,
            "{source}: {:?}",
            report.diagnostics()
        );
        let [CssRule::Style(parent), CssRule::Style(tail)] = report.syntax().rules() else {
            panic!("parent and tail")
        };
        assert_eq!(parent.declarations().len(), 1);
        assert_color(&parent.declarations()[0], "red");
        let [CssRule::NestedDeclarations(green)] = parent.rules() else {
            panic!("one trailing run, no empty run")
        };
        assert_eq!(green.declarations().len(), 1);
        assert_color(&green.declarations()[0], "green");
        assert_eq!(
            green.position().byte_offset().value(),
            source.find("color:green").unwrap()
        );
        assert_eq!(
            tail.position().byte_offset().value(),
            source.find("tail{}").unwrap()
        );
    }
}

#[test]
fn escaped_custom_property_lookalike_and_identifier_no_rule_do_not_partition() {
    for rejected in [
        "bad;",
        "--bad:var(){color:blue};",
        "\\2d -bad:var(){color:blue};",
    ] {
        let source = format!(".p{{color:red;{rejected}color:green}}");
        let report = parse_sheet(&source);
        assert_eq!(
            report.diagnostics().len(),
            1,
            "{source}: {:?}",
            report.diagnostics()
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("parent")
        };
        assert_eq!(parent.declarations().len(), 2);
        assert_color(&parent.declarations()[0], "red");
        assert_color(&parent.declarations()[1], "green");
        assert!(parent.rules().is_empty());
    }
}

#[test]
fn masked_rejected_child_at_chunk_boundary_keeps_leading_slot_semantics() {
    // 62 scopes + style parent put the rejected child at depth 64 itself.
    // Empty child syntax must still partition an earlier nonempty list; without
    // that earlier list the first later declarations remain leading.
    for before in ["color:red;", ""] {
        for rejected in [".bad,{}", "@layer a,b{}"] {
            let source = format!(
                "{}.p{{{before}{rejected}color:green}}{}",
                "@scope{".repeat(62),
                "}".repeat(62)
            );
            let report = parse_sheet(&source);
            assert_eq!(report.diagnostics().len(), 1, "{:?}", report.diagnostics());
            let [CssRule::Scope(scope)] = report.syntax().rules() else {
                panic!("scope")
            };
            let mut rules = scope.rules().rules();
            for _ in 1..62 {
                let [CssScopedRule::Scope(scope)] = rules else {
                    panic!("scope chain")
                };
                rules = scope.rules().rules();
            }
            let [CssScopedRule::Style(parent)] = rules else {
                panic!("scoped style")
            };
            assert_eq!(parent.declarations().len(), 1);
            if before.is_empty() {
                assert_color(&parent.declarations()[0], "green");
                assert!(parent.rules().is_empty());
            } else {
                assert_color(&parent.declarations()[0], "red");
                let [CssRule::NestedDeclarations(green)] = parent.rules() else {
                    panic!("separate green run")
                };
                assert_eq!(green.declarations().len(), 1);
                assert_color(&green.declarations()[0], "green");
            }
        }
    }
}

#[test]
fn failed_block_resource_error_retains_partition_and_original_error_identity() {
    // The style and failed qualified block already consume two structural levels.
    let source = format!(
        ".p{{color:red;bad,{{{}x{}}}color:green}}tail{{}}",
        "f(".repeat(255),
        ")".repeat(255)
    );
    let report = parse_sheet(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one resource failure: {:?}", report.diagnostics())
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find("bad,").unwrap()
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.find("color:green").unwrap()
    );
    let [CssRule::Style(parent), CssRule::Style(tail)] = report.syntax().rules() else {
        panic!("parent and tail")
    };
    assert_eq!(parent.declarations().len(), 1);
    assert_color(&parent.declarations()[0], "red");
    let [CssRule::NestedDeclarations(green)] = parent.rules() else {
        panic!("green run")
    };
    assert_eq!(green.declarations().len(), 1);
    assert_color(&green.declarations()[0], "green");
    assert_eq!(
        tail.position().byte_offset().value(),
        source.find("tail{}").unwrap()
    );
}
