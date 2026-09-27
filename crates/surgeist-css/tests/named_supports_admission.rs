#![forbid(unsafe_code)]
//! Conditional Rules 5 §8 definition placement and Syntax block-context admission.

use surgeist_css::{
    CssNamespaceContext, CssNormalizationErrorKind, CssNormalizationLimits,
    CssNormalizationResource, CssNormalizedItem, CssRecoveryAction, CssRule, CssRuleContextKindRef,
    normalize_sheet_with_limits, parse_rule, parse_sheet, parse_style_block, validate_sheet,
};

#[test]
fn invalid_outer_definitions_drop_without_promoting_their_test_bodies() {
    for source in [
        "@supports-condition plain { leaked:yes; } .after {}",
        "@supports-condition --x extra { leaked:yes; } .after {}",
        "@supports-condition --x; .after {}",
    ] {
        let report = parse_sheet(source);
        assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
        assert!(!report.is_clean(), "{source:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropAtRule)
        );
        let failure = validate_sheet(source).expect_err("outer recovery is not clean validation");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }

    let missing_block = "@supports-condition --x";
    let report = parse_sheet(missing_block);
    assert!(report.syntax().rules().is_empty());
    assert!(!report.is_clean());
    assert!(validate_sheet(missing_block).is_err());
}

#[test]
fn named_definition_preserves_import_namespace_phase_without_admitting_late_charset() {
    for source in [
        "@supports-condition --x {} @import 'a.css'; @namespace svg 'urn:svg';",
        "@import 'a.css'; @supports-condition --x {} @namespace svg 'urn:svg';",
        "@import 'a.css'; @namespace svg 'urn:svg'; @supports-condition --x {}",
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source:?}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().rules().len(), 3);
        assert_eq!(
            report
                .syntax()
                .rules()
                .iter()
                .filter(|rule| matches!(rule, CssRule::SupportsCondition(_)))
                .count(),
            1
        );
        assert!(
            report
                .syntax()
                .rules()
                .iter()
                .any(|rule| matches!(rule, CssRule::Import(_)))
        );
        assert!(
            report
                .syntax()
                .rules()
                .iter()
                .any(|rule| matches!(rule, CssRule::Namespace(_)))
        );
    }

    let late_charset = "@supports-condition --x {} @charset \"UTF-8\"; .after {}";
    let report = parse_sheet(late_charset);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::SupportsCondition(_), CssRule::Style(_)]
    ));
    assert!(!report.is_clean());
    assert!(validate_sheet(late_charset).is_err());

    let closed = concat!(
        "@import 'early.css'; .ordinary {} @supports-condition --x {}",
        " @import 'late.css'; @namespace late 'urn:late'; .after {}",
    );
    let report = parse_sheet(closed);
    assert!(matches!(
        report.syntax().rules(),
        [
            CssRule::Import(_),
            CssRule::Style(_),
            CssRule::SupportsCondition(_),
            CssRule::Style(_)
        ]
    ));
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::DropAtRule)
            .count(),
        2
    );
}

#[test]
fn descriptor_and_keyframe_only_blocks_do_not_admit_named_definitions_or_style_tests() {
    for source in [
        concat!(
            "@font-face { font-family: Example;",
            " @supports-condition --x { leaked:yes; } font-display:swap;",
            "} .after {}",
        ),
        concat!(
            "@keyframes fade { from { opacity:0;",
            " @supports-condition --x { leaked:yes; } opacity:1;",
            "} } .after {}",
        ),
    ] {
        let report = parse_sheet(source);
        assert!(!report.is_clean(), "{source:?}");
        assert!(matches!(
            report.syntax().rules().last(),
            Some(CssRule::Style(_))
        ));
        assert!(
            report
                .syntax()
                .rules()
                .iter()
                .all(|rule| !matches!(rule, CssRule::SupportsCondition(_)))
        );
        let normalized = normalize_sheet_with_limits(
            report.syntax(),
            CssNormalizationLimits::try_new(3, 8, 0, 0).unwrap(),
        )
        .expect("excluded test bodies cannot produce style contributions");
        assert!(normalized.items().iter().all(|item| {
            matches!(
                item,
                CssNormalizedItem::Rule(rule)
                    if !matches!(rule.kind(), CssRuleContextKindRef::SupportsCondition(_))
            )
        }));
        assert!(validate_sheet(source).is_err());
    }
}

#[test]
fn isolated_rule_and_style_block_entries_share_named_definition_dispatch() {
    let context = CssNamespaceContext::default();
    let source = "@supports-condition --x { future:yes; }";
    let isolated = parse_rule(source, &context);
    assert!(isolated.is_clean(), "{:?}", isolated.diagnostics());
    let Some(CssRule::SupportsCondition(definition)) = isolated.syntax() else {
        panic!("exact-one entry retains the named definition")
    };
    assert_eq!(definition.name().as_str(), "--x");

    for source in [
        "@supports-condition plain { future:yes; }",
        "@supports-condition --x extra { future:yes; }",
        "@supports-condition --x;",
    ] {
        let isolated = parse_rule(source, &context);
        assert!(isolated.syntax().is_none(), "{source:?}");
        assert!(!isolated.is_clean());
        assert!(
            isolated
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
        );
    }

    let block = parse_style_block("{ @supports-condition --x { future:yes; } }", &context);
    assert!(block.is_clean(), "{:?}", block.diagnostics());
    let Some(style) = block.syntax() else {
        panic!("standalone style block")
    };
    assert!(matches!(style.rules(), [CssRule::SupportsCondition(_)]));
}

#[test]
fn named_test_payload_uses_one_rule_budget_and_zero_style_declaration_budgets() {
    let source = concat!(
        "@supports-condition --x { future:value;",
        " & { nested:value; } @future { another:value; } }",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let sufficient = CssNormalizationLimits::try_new(0, 1, 0, 0).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), sufficient).unwrap();
    let [CssNormalizedItem::Rule(rule)] = normalized.items() else {
        panic!("one terminal definition payload")
    };
    assert!(matches!(
        rule.kind(),
        CssRuleContextKindRef::SupportsCondition(_)
    ));
    assert!(rule.parent().is_none());

    let two = "@supports-condition --a {} @supports-condition --b {}";
    let report = parse_sheet(two);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.syntax().clone();
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 0, 0).unwrap(),
    )
    .unwrap_err();
    assert!(matches!(
        error.kind(),
        CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Rules,
            limit: 1,
        }
    ));
    assert_eq!(
        error.position().unwrap().byte_offset().value(),
        two.find("@supports-condition --b").unwrap()
    );
    assert_eq!(*report.syntax(), before);

    let nested = "@media all { @supports-condition --x { future:value; } }";
    let report = parse_sheet(nested);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.syntax().clone();
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 2, 0, 0).unwrap(),
    )
    .unwrap_err();
    assert!(matches!(
        error.kind(),
        CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::RuleDepth,
            limit: 0,
        }
    ));
    assert_eq!(
        error.position().unwrap().byte_offset().value(),
        nested.find("@supports-condition").unwrap()
    );
    assert_eq!(*report.syntax(), before);
}
