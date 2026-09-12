#![forbid(unsafe_code)]

//! Focused MQ4/MQ5 grouping/operator correction using existing known MQ3 leaves.
//! Grammar authority: pinned MQ4 2026-02-19 section 3, mq-syntax.
//! Balanced enclosures retain their own grammar classification alongside grouping.

use surgeist_css::{
    CssErrorCode, CssMediaConditionKind, CssMediaQuery, CssMediaType, CssRecoveryAction, CssRule,
    parse_media_query, parse_media_query_list, parse_sheet,
};

#[test]
fn ungrouped_known_conditions_retain_their_operators() {
    for (source, expected) in [
        ("not (color)", "not"),
        ("(color) and (monochrome)", "and"),
        ("(color) or (monochrome)", "or"),
    ] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let CssMediaQuery::Condition(condition) = report.syntax() else {
            panic!("condition query");
        };
        let actual = match condition.kind() {
            CssMediaConditionKind::Not(_) => "not",
            CssMediaConditionKind::And(_) => "and",
            CssMediaConditionKind::Or(_) => "or",
            _ => panic!("expected operator"),
        };
        assert_eq!(actual, expected);
    }
}

#[test]
fn grouped_known_features_and_legal_typed_conditions_are_clean() {
    for source in [
        "((color))",
        "(not (color)) or (monochrome)",
        "not ((color) or (monochrome))",
        "(color) and ((width) or (monochrome))",
        "(not (color)) and (not (monochrome))",
        "screen and not (color)",
        "screen and ((color) or (monochrome))",
        "only screen and ((color) or (monochrome)) and (width)",
    ] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(!report.syntax().is_guaranteed_false(), "{source}");
        assert!(report.into_validation_result().is_ok());
    }
}

#[test]
fn ungrouped_not_chains_and_typed_bare_or_recover_one_whole_query() {
    for source in [
        "not not (color)",
        "not (color) and (monochrome)",
        "not (color) or (monochrome)",
        "(color) and not (monochrome)",
        "(color) or not (monochrome)",
        "(color) and (width) or (monochrome)",
        "screen and (color) or (monochrome)",
        "screen and not (color) and (monochrome)",
    ] {
        let report = parse_media_query(source);
        assert!(
            matches!(report.syntax(), CssMediaQuery::Never(_)),
            "{source}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!(
                "one whole-query recovery: {source}: {:?}",
                report.diagnostics()
            );
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever
        );
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn grouped_query_and_malformed_neighbor_retain_exact_list_ownership() {
    let source = "/*😀*/ ((color)), not not (color), print";
    let report = parse_media_query_list(source);
    let [first, middle, CssMediaQuery::Typed(last)] = report.syntax().queries() else {
        panic!("exactly three ordered members");
    };
    assert!(!first.is_guaranteed_false());
    assert!(middle.is_guaranteed_false());
    assert_eq!(last.media_type(), CssMediaType::Print);
    let offset = source.find("((color))").unwrap();
    assert_eq!(
        first
            .position()
            .expect("parsed media position")
            .byte_offset()
            .value(),
        offset
    );
    assert_eq!(
        first
            .position()
            .expect("parsed media position")
            .column()
            .value(),
        u32::try_from(source[..offset].encode_utf16().count()).unwrap()
    );
    assert_eq!(
        middle
            .position()
            .expect("parsed media position")
            .byte_offset()
            .value(),
        source.find("not not").unwrap()
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one malformed member");
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find(", not").unwrap() + 1
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.find(", print").unwrap()
    );
}

#[test]
fn media_and_import_share_grouping_and_comma_local_recovery() {
    for source in [
        "@media ((color)), not not (color), print {} .after {color:red}",
        "@import \"x\" ((color)), not not (color), print; .after {color:red}",
    ] {
        let report = parse_sheet(source);
        let [owner, CssRule::Style(_)] = report.syntax().rules() else {
            panic!("owner and sibling");
        };
        let queries = match owner {
            CssRule::Media(rule) => rule.query().queries(),
            CssRule::Import(rule) => rule.media().unwrap().queries(),
            _ => panic!("media/import owner"),
        };
        let [first, middle, CssMediaQuery::Typed(last)] = queries else {
            panic!("three members");
        };
        assert!(!first.is_guaranteed_false());
        assert!(middle.is_guaranteed_false());
        assert_eq!(last.media_type(), CssMediaType::Print);
        let [diagnostic] = report.diagnostics() else {
            panic!("one malformed member");
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            source.find(", not").unwrap() + 1
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            source.find(", print").unwrap()
        );
    }
}

fn grouped(depth: usize) -> String {
    format!("{}color{}", "(".repeat(depth), ")".repeat(depth))
}

#[test]
fn grouped_depth_limit_retains_comma_sibling_at_255_256_and_257() {
    for depth in [255, 256, 257] {
        let query = grouped(depth);
        let source = format!("screen,{query},print");
        let report = parse_media_query_list(&source);
        let [
            CssMediaQuery::Typed(first),
            middle,
            CssMediaQuery::Typed(last),
        ] = report.syntax().queries()
        else {
            panic!("three comma members at depth {depth}");
        };
        assert_eq!(first.media_type(), CssMediaType::Screen);
        assert_eq!(last.media_type(), CssMediaType::Print);
        if depth <= 256 {
            assert!(
                report.is_clean(),
                "depth {depth}: {:?}",
                report.diagnostics()
            );
            assert!(!middle.is_guaranteed_false());
        } else {
            assert!(middle.is_guaranteed_false());
            let [diagnostic] = report.diagnostics() else {
                panic!("one limit diagnostic");
            };
            assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
            assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
            assert_eq!(diagnostic.span().start().byte_offset().value(), 7);
            assert_eq!(
                diagnostic.span().end().byte_offset().value(),
                7 + query.len()
            );
            assert_eq!(diagnostic.error().position().byte_offset().value(), 263);
        }
    }
}

#[test]
fn inherited_sheet_depth_limits_groups_without_consuming_later_members() {
    for total_depth in [255, 256, 257] {
        let query = grouped(total_depth - 1);
        let source = format!("@media screen {{ @media {query},print {{}} }} .after {{color:red}}");
        let report = parse_sheet(&source);
        let [CssRule::Media(outer), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("outer owner and sibling");
        };
        let [CssRule::Media(inner)] = outer.rules() else {
            panic!("inner owner");
        };
        let [condition, CssMediaQuery::Typed(last)] = inner.query().queries() else {
            panic!("comma continuation");
        };
        assert_eq!(last.media_type(), CssMediaType::Print);
        if total_depth <= 256 {
            assert!(
                report.is_clean(),
                "depth {total_depth}: {:?}",
                report.diagnostics()
            );
            assert!(!condition.is_guaranteed_false());
        } else {
            assert!(condition.is_guaranteed_false());
            let [diagnostic] = report.diagnostics() else {
                panic!("one inherited limit");
            };
            assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
            assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
        }
    }
}

#[test]
fn grouped_eof_closure_is_published_only_for_retained_owners() {
    let source = "((color)";
    let raw = parse_media_query(source);
    assert!(!raw.syntax().is_guaranteed_false());
    let [diagnostic] = raw.diagnostics() else {
        panic!("one group closure");
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
        diagnostic.span().start().byte_offset().value(),
        source.len()
    );
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());

    let import = parse_sheet("@import \"x\" ((color)");
    assert!(matches!(import.syntax().rules(), [CssRule::Import(_)]));
    assert_eq!(
        import
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [CssRecoveryAction::RetainWithImplicitClosure]
    );

    let discarded = parse_sheet("@media ((color)");
    assert!(discarded.syntax().rules().is_empty());
    assert_eq!(
        discarded
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [CssRecoveryAction::DropAtRule]
    );
}
