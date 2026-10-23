//! Functional evidence for the CSSOM media boundary: pinned CSSOM1 section 4,
//! selected MQ5 grammar/recovery, and the shared specified resource policy.

use crate::css::support::value_assertions::clean_media_query as query;
use crate::css::support::value_assertions::media_resource_kind as resource_kind;
use surgeist_css::{
    CssRecoveryAction, CssSerializationLimits as Limits, CssSerializedOrigin,
    CssSpecifiedValueSerializationErrorKind as ResourceKind, CssValueOrigin,
    parse_cssom_media_query, parse_media_query, parse_media_query_list,
};

#[test]
fn single_query_requires_exactly_one_list_member_and_preserves_recovery_diagnostics() {
    for source in ["", " /* empty */ ", "screen, print", "screen, ???"] {
        let expected = parse_media_query_list(source);
        let single = parse_cssom_media_query(source);
        assert!(single.syntax().is_none(), "{source}");
        assert_eq!(single.diagnostics(), expected.diagnostics(), "{source}");
    }
    assert_eq!(
        query("screen")
            .serialize_cssom(Default::default())
            .unwrap()
            .as_css(),
        "screen"
    );

    let ignored = parse_cssom_media_query("???");
    assert_eq!(ignored.diagnostics().len(), 1);
    assert_eq!(
        ignored.diagnostics()[0].action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    assert_eq!(
        ignored
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_cssom(Default::default())
            .unwrap()
            .as_css(),
        "not all"
    );
}

#[test]
fn ignored_members_project_to_not_all_in_place_without_mutating_authored_reports() {
    let source = "/* 😀 */\r\nscreen, ???, print, &test, speech";
    let report = parse_media_query_list(source);
    let original = report.clone();
    let output = report.syntax().serialize_cssom(Default::default()).unwrap();
    assert_eq!(output.as_css(), "screen, not all, print, not all, speech");
    assert_eq!(report, original);
    assert_eq!(report.diagnostics().len(), 2);
    for (diagnostic, malformed) in report.diagnostics().iter().zip(["???", "&test"]) {
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.find(malformed).unwrap()
        );
    }
    for (offset, member) in [(8, 1), (24, 3)] {
        assert!(matches!(
            output.origin_at(offset),
            Some(CssSerializedOrigin::Token(origin)) if origin == report.syntax().queries()[member].origin()
        ));
    }
    assert!(matches!(
        report.syntax().serialize_tokens(Default::default()),
        Err(ref error) if crate::css::support::is_recovered_media_failure(error)
    ));
}

#[test]
fn a_recovered_single_query_keeps_its_authored_fail_closed_serialization() {
    let report = parse_media_query("screen and");
    let original = report.clone();
    assert_eq!(
        report
            .syntax()
            .serialize_cssom(Default::default())
            .unwrap()
            .as_css(),
        "not all"
    );
    assert!(matches!(
        report.syntax().serialize_tokens(Default::default()),
        Err(ref error) if crate::css::support::is_recovered_media_failure(error)
    ));
    assert_eq!(report, original);
}

/// CSSOM "serialize a media query" (editor capture 2026-10-09 and WD
/// 2021-08-26): step 1 appends `not` for a negated query, step 3 appends the
/// lowercased type when there are no features, and step 4 appends `type and`
/// only "If type is not "all" or if the media query is negated". No step emits
/// `only` (#1378). Pinned WebKit 73aa6c89e2cb keeps `only`; that witness
/// divergence is recorded, not followed.
#[test]
fn cssom_output_drops_only_and_redundant_all_and_keeps_negated_types_and_list_order() {
    for (source, expected) in [
        ("only SCREEN", "screen"),
        ("only ALL", "all"),
        ("only ALL and (COLOR)", "(color)"),
        ("only SCREEN and (COLOR)", "screen and (color)"),
        ("ALL and (COLOR)", "(color)"),
        ("NOT ALL and (COLOR)", "not all and (color)"),
        ("NOT SCREEN and (COLOR)", "not screen and (color)"),
        ("not ALL", "not all"),
        ("not PRINT", "not print"),
        ("all", "all"),
        ("(COLOR) OR (MONOCHROME)", "(color) or (monochrome)"),
        (
            "NOT ((COLOR) OR (MONOCHROME))",
            "not ((color) or (monochrome))",
        ),
    ] {
        assert_eq!(
            query(source)
                .serialize_cssom(Default::default())
                .unwrap()
                .as_css(),
            expected,
            "{source}"
        );
    }
    for (source, expected) in [
        (
            "PRINT, ALL and (COLOR), ONLY SCREEN",
            "print, (color), screen",
        ),
        (
            "only screen and (color), NOT all and (monochrome), only all",
            "screen and (color), not all and (monochrome), all",
        ),
    ] {
        let list = parse_media_query_list(source);
        assert_eq!(
            list.syntax()
                .serialize_cssom(Default::default())
                .unwrap()
                .as_css(),
            expected,
            "{source}"
        );
        let members = list
            .syntax()
            .serialize_cssom_members(Default::default())
            .unwrap();
        assert_eq!(
            members
                .iter()
                .map(|m| m.as_css())
                .collect::<Vec<_>>()
                .join(", "),
            expected,
            "{source}"
        );
    }
}

#[test]
fn authored_output_keeps_the_authored_only_prefix() {
    for (source, expected) in [
        ("only SCREEN", "only screen"),
        ("only ALL and (COLOR)", "only all and (color)"),
        ("NOT ALL and (COLOR)", "not all and (color)"),
    ] {
        assert_eq!(
            parse_media_query(source)
                .syntax()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            expected,
            "{source}"
        );
    }
    assert_eq!(
        parse_media_query_list("PRINT, ONLY SCREEN")
            .syntax()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "print, only screen"
    );
}

#[test]
fn valid_unknown_syntax_is_preserved_without_context_free_truth_evaluation() {
    for (source, expected) in [
        ("not FuTuRe", "not future"),
        ("(FuTuRe: ACTIVE)", "(future: ACTIVE)"),
        ("Future(\"A\")", "Future(\"A\")"),
        ("(--CaseSensitive)", "(--CaseSensitive)"),
        ("not (grid: 2)", "not (grid: 2)"),
    ] {
        let authored = query(source);
        let original = authored.clone();
        assert_eq!(
            authored
                .serialize_cssom(Default::default())
                .unwrap()
                .as_css(),
            expected
        );
        assert_eq!(authored, original);
    }
    // The authored unknown-feature spelling remains separately available.
    assert_eq!(
        query("(FuTuRe: ACTIVE)")
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "(FuTuRe: ACTIVE)"
    );
}

#[test]
fn canonical_keyword_output_retains_the_original_value_token_origin() {
    for source in [
        "/* 😀 */\r\n(ORIENTATION: PORTRAIT)",
        "/* 😀 */\r\n(ORIENTATION: /* keyword */ PORTRAIT)",
    ] {
        let authored = query(source);
        let output = authored.serialize_cssom(Default::default()).unwrap();
        assert_eq!(output.as_css(), "(orientation: portrait)");
        let Some(CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin))) = output.origin_at(14)
        else {
            panic!("canonical keyword retains original token provenance");
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            source.find("PORTRAIT").unwrap()
        );
        assert_eq!(
            origin.span().end().byte_offset().value(),
            source.find("PORTRAIT").unwrap() + 8
        );
    }
}

#[test]
fn byte_equality_is_canonical_case_sensitive_and_distinct_from_authored_identity() {
    let first = query("ALL and (COLOR)");
    let second = query("(color)");
    assert_ne!(first, second);
    assert!(first.cssom_equals(&second, Default::default()).unwrap());
    assert!(
        !query("future(\"A\")")
            .cssom_equals(&query("future(\"a\")"), Default::default())
            .unwrap()
    );
    assert!(
        !query("(color) and (monochrome)")
            .cssom_equals(&query("(monochrome) and (color)"), Default::default())
            .unwrap()
    );
    let recovered = parse_cssom_media_query("???");
    assert!(
        recovered
            .syntax()
            .as_ref()
            .unwrap()
            .cssom_equals(&query("not all"), Default::default())
            .unwrap()
    );
}

#[test]
fn query_resource_limits_are_exact_atomic_and_preserve_responsible_origins() {
    // One query aggregate plus its media-type component: two input/projection nodes.
    let authored = query("screen");
    let original = authored.clone();
    assert_eq!(
        authored
            .serialize_cssom(Limits::new(2, 2, 6))
            .unwrap()
            .as_css(),
        "screen"
    );
    for (limits, expected) in [
        (Limits::new(1, 2, 6), ResourceKind::InputNodeLimit),
        (Limits::new(2, 1, 6), ResourceKind::ProjectionNodeLimit),
        (Limits::new(2, 2, 5), ResourceKind::ByteLimit),
    ] {
        let error = authored.serialize_cssom(limits).unwrap_err();
        assert_eq!(resource_kind(&error), expected);
        assert!(matches!(error.origin(), Some(CssValueOrigin::Parsed(_))));
        assert_eq!(authored, original);
    }
}

#[test]
fn list_members_share_node_and_byte_limits_including_the_list_aggregate() {
    let list = parse_media_query_list("screen, print").into_parts().0;
    assert_eq!(
        list.serialize_cssom(Limits::new(5, 5, 13))
            .unwrap()
            .as_css(),
        "screen, print"
    );
    for member in list.queries() {
        assert!(member.serialize_cssom(Limits::new(2, 2, 6)).is_ok());
    }
    for (limits, expected) in [
        (Limits::new(4, 5, 13), ResourceKind::InputNodeLimit),
        (Limits::new(5, 4, 13), ResourceKind::ProjectionNodeLimit),
        (Limits::new(5, 5, 12), ResourceKind::ByteLimit),
    ] {
        assert_eq!(
            resource_kind(&list.serialize_cssom(limits).unwrap_err()),
            expected
        );
    }
    let empty = parse_media_query_list("").into_parts().0;
    assert_eq!(
        empty
            .serialize_cssom(Limits::new(1, 1, 0))
            .unwrap()
            .as_css(),
        ""
    );
}

#[test]
fn recovery_charges_synthetic_projection_and_comparison_shares_both_query_budgets() {
    let recovered = parse_cssom_media_query("???").into_parts().0.unwrap();
    // One retained query node and two generated keyword projection nodes.
    assert_eq!(
        recovered
            .serialize_cssom(Limits::new(1, 3, 7))
            .unwrap()
            .as_css(),
        "not all"
    );
    assert_eq!(
        resource_kind(&recovered.serialize_cssom(Limits::new(1, 2, 7)).unwrap_err()),
        ResourceKind::ProjectionNodeLimit
    );
    let screen = query("screen");
    assert!(screen.cssom_equals(&screen, Limits::new(4, 4, 12)).unwrap());
    for (limits, expected) in [
        (Limits::new(3, 4, 12), ResourceKind::InputNodeLimit),
        (Limits::new(4, 3, 12), ResourceKind::ProjectionNodeLimit),
        (Limits::new(4, 4, 11), ResourceKind::ByteLimit),
    ] {
        assert_eq!(
            resource_kind(&screen.cssom_equals(&screen, limits).unwrap_err()),
            expected
        );
    }
}

#[test]
fn deep_and_eof_recovered_conditions_use_the_same_owning_grammar_and_emitter() {
    let source = format!("{}(color){}", "(".repeat(90), ")".repeat(90));
    assert_eq!(
        query(&source)
            .serialize_cssom(Default::default())
            .unwrap()
            .as_css(),
        source
    );
    let eof = parse_cssom_media_query("((COLOR)");
    assert_eq!(eof.diagnostics().len(), 1);
    assert_eq!(
        eof.diagnostics()[0].action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(
        eof.syntax()
            .as_ref()
            .unwrap()
            .serialize_cssom(Default::default())
            .unwrap()
            .as_css(),
        "((color))"
    );
}
