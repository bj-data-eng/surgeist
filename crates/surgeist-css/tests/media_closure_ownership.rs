#![forbid(unsafe_code)]

//! An accepted query prelude is tentative until its owning rule survives.
//! Syntax 3 requires @media to have a block; EOF closure diagnostics may
//! describe retained raw queries, import rules, and enclosing rule blocks.

use surgeist_css::{CssRecoveryAction, CssRule, parse_media_query_list, parse_sheet};

#[test]
fn discarded_media_rule_does_not_publish_tentative_query_closures() {
    for source in ["@media (fo", "@media print,(fo"] {
        let report = parse_sheet(source);
        assert!(report.syntax().rules().is_empty(), "{source}");
        assert_eq!(
            report
                .diagnostics()
                .iter()
                .map(|d| d.action())
                .collect::<Vec<_>>(),
            [CssRecoveryAction::DropAtRule],
            "{source}",
        );
        assert_eq!(
            report.diagnostics()[0].span().end().byte_offset().value(),
            source.len()
        );
    }
}

#[test]
fn discarded_media_rule_preserves_examined_invalid_member_diagnostic() {
    let report = parse_sheet("@media ???,(fo");
    assert!(report.syntax().rules().is_empty());
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [
            CssRecoveryAction::ReplaceMediaQueryWithNever,
            CssRecoveryAction::DropAtRule
        ],
    );
}

#[test]
fn retained_enclosing_rule_keeps_only_its_own_implicit_closure() {
    let report = parse_sheet("@supports (display:grid){@media (fo");
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("retained enclosing supports rule");
    };
    assert!(rule.rules().is_empty());
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [
            CssRecoveryAction::DropAtRule,
            CssRecoveryAction::RetainWithImplicitClosure
        ],
    );
}

#[test]
fn retained_import_and_raw_query_keep_query_closure_diagnostics() {
    let import = parse_sheet("@import \"x\" (fo");
    assert!(matches!(import.syntax().rules(), [CssRule::Import(_)]));
    assert_eq!(
        import
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [CssRecoveryAction::RetainWithImplicitClosure],
    );
    let raw = parse_media_query_list("print,(fo");
    assert_eq!(raw.syntax().queries().len(), 2);
    assert!(!raw.syntax().queries()[1].is_guaranteed_false());
    assert_eq!(
        raw.diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [CssRecoveryAction::RetainWithImplicitClosure],
    );
}

#[test]
fn retained_import_keeps_valid_member_closure_after_invalid_neighbor() {
    let report = parse_sheet("@import \"x\" ???,(fo");
    let [CssRule::Import(rule)] = report.syntax().rules() else {
        panic!("retained import rule");
    };
    let queries = rule.media().unwrap().queries();
    assert_eq!(queries.len(), 2);
    assert!(queries[0].is_guaranteed_false());
    assert!(!queries[1].is_guaranteed_false());
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [
            CssRecoveryAction::ReplaceMediaQueryWithNever,
            CssRecoveryAction::RetainWithImplicitClosure
        ],
    );
}
