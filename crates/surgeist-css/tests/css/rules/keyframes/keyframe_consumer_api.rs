//! Public-only Animations 1 and selected CSSOM detached keyframe contracts.

use crate::css::support::failure_classes::SerializationFailure as Resource;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::rules::ExpectedRejection;
use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::{
    CssDeclaration, CssImportance, CssInvalidKeyframeSourceReason, CssSpecifiedDeclarationBlock,
};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::{CssKnownProperty, CssPropertyNameRef, CssPropertyTarget};
use surgeist_css::rule::keyframes::{
    CssKeyframePercent, CssKeyframeRuleView, CssKeyframeRuleViewError, CssKeyframeSelector,
    CssKeyframeSelectorList, CssParsedKeyframeRule, CssParsedKeyframeSelectors,
};
use surgeist_css::serialization::CssSerializationLimits as Limits;
use surgeist_css::{CssParser, CssParserContext, CssParserMode};

/// The serialization resource whose budget a detached keyframe exhausted,
/// whether in its selectors or in its declaration block.
fn exhausted(error: &CssKeyframeRuleViewError) -> Resource {
    let serialization = error
        .serialization()
        .or_else(|| {
            error
                .declaration_block()
                .and_then(|block| block.serialization())
        })
        .unwrap_or_else(|| panic!("serialization resource failure: {error:?}"));
    serialization_failure(serialization)
}

fn selectors(source: &str) -> CssKeyframeSelectorList {
    let report = CssParsedKeyframeSelectors::parse(&CssParser::default(), source);
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().as_ref().unwrap().selectors().clone()
}

#[test]
fn raw_keytext_preserves_complete_source_order_duplicates_and_endpoint_observations() {
    let source = " /*😀*/\r\nFROM /*a*/, 0%, to, 100%, 50%, 50% /*z*/ ";
    let report = CssParsedKeyframeSelectors::parse(&CssParser::default(), source);
    assert!(report.is_clean(), "{report:?}");
    let parsed = report.syntax().as_ref().unwrap();
    assert_eq!(parsed.origin().source().as_str(), source);
    assert_eq!(parsed.origin().span().start().byte_offset().value(), 0);
    assert_eq!(
        parsed.origin().span().end().byte_offset().value(),
        source.len()
    );
    assert!(matches!(
        parsed.selectors().selectors()[0],
        CssKeyframeSelector::From
    ));
    assert!(matches!(
        parsed.selectors().selectors()[2],
        CssKeyframeSelector::To
    ));
    let before = report.clone();
    assert_eq!(
        parsed
            .selectors()
            .serialize_key_text(Default::default())
            .unwrap(),
        "0%, 0%, 100%, 100%, 50%, 50%"
    );
    assert_eq!(
        parsed
            .selectors()
            .offsets()
            .map(|p| p.literal_value().unwrap())
            .collect::<Vec<_>>(),
        vec![0.0, 0.0, 100.0, 100.0, 50.0, 50.0]
    );
    assert_eq!(report, before);
}

#[test]
fn ordered_offset_comparison_normalizes_aliases_without_rounding_or_deduplication() {
    let normalized = |source| selectors(source).offsets().collect::<Vec<_>>();
    assert_eq!(
        normalized("from, to, 50%, 50%"),
        normalized("0%,100%,50%,50%")
    );
    assert_ne!(normalized("0%, 50%"), normalized("50%, 0%"));
    assert_ne!(normalized("50%, 50%"), normalized("50%"));
    assert_ne!(normalized("0.0000001%"), normalized("0.0000002%"));
    assert_eq!(
        selectors("0.0000001%, 0.0000002%")
            .serialize_key_text(Default::default())
            .unwrap(),
        "0%, 0%"
    );
}

#[test]
fn invalid_or_trailing_raw_keytext_rejects_complete_input_with_original_diagnostic() {
    for source in [
        "",
        " /**/ ",
        "0",
        "-1%",
        "101%",
        "1e999%",
        "middle",
        "from,",
        ",to",
        "from to",
        "from;",
        "from{}",
        "from, var(--x)",
    ] {
        let report = CssParsedKeyframeSelectors::parse(&CssParser::default(), source);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
        for d in report.diagnostics() {
            assert!(d.error().position().byte_offset().value() <= source.len());
            assert_eq!(d.span().start().byte_offset().value(), 0);
            assert_eq!(d.span().end().byte_offset().value(), source.len());
        }
    }
}

#[test]
fn complete_detached_rule_preserves_original_unicode_positions_and_snapshot_identity() {
    let source = " /*😀*/\r\n from, 0%, to /*s*/ { opacity:0; --X:日本; opacity:1 } /*tail*/ ";
    let report = CssParsedKeyframeRule::parse(&CssParser::default(), source);
    assert!(report.is_clean(), "{report:?}");
    let parsed = report.syntax().as_ref().unwrap();
    let start = source.find("from").unwrap();
    let end = source.find('}').unwrap() + 1;
    assert_eq!(parsed.origin().source().as_str(), source);
    assert_eq!(parsed.origin().span().start().byte_offset().value(), start);
    assert_eq!(parsed.origin().span().end().byte_offset().value(), end);
    assert_eq!(parsed.block().position().line().value(), 1);
    assert_eq!(parsed.block().position().column().value(), 1);
    assert_eq!(parsed.block().declarations().len(), 3);
    assert_eq!(
        parsed.block().declarations()[0]
            .position()
            .byte_offset()
            .value(),
        source.find("opacity").unwrap()
    );
    let value_origin = parsed.block().declarations()[0]
        .source()
        .parsed_value()
        .unwrap();
    assert!(
        parsed
            .origin()
            .source()
            .same_snapshot(value_origin.source())
    );
    assert_eq!(
        parsed
            .block()
            .selectors()
            .serialize_key_text(Default::default())
            .unwrap(),
        "0%, 0%, 100%"
    );
}

#[test]
fn detached_rule_recovers_declarations_but_rejects_invalid_outer_or_trailing_input() {
    let source = "from{opacity:0; mystery:1; opacity:2 !important; animation-name:x; opacity:1}";
    let report = CssParsedKeyframeRule::parse(&CssParser::default(), source);
    assert!(report.syntax().is_some(), "{report:?}");
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .block()
            .declarations()
            .len(),
        2
    );
    assert_eq!(report.diagnostics().len(), 3);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|d| d.action() == CssRecoveryAction::DropDeclaration)
    );
    assert!(report.into_validation_result().is_err());
    // Syntax 3 (2026-10-01) §5.5.5 and §7.1: a nested rule is invalid in the
    // keyframe's `<declaration-list>` and dropped alone.
    for (source, action) in [
        (
            "from{.x{opacity:0} opacity:1}",
            CssRecoveryAction::DropQualifiedRule,
        ),
        (
            "from{@media all{opacity:0} opacity:1}",
            CssRecoveryAction::DropAtRule,
        ),
    ] {
        let report = CssParsedKeyframeRule::parse(&CssParser::default(), source);
        let rule = report.syntax().as_ref().expect("recovered keyframe rule");
        assert_eq!(rule.block().declarations().len(), 1, "{source}");
        assert_eq!(
            report
                .diagnostics()
                .iter()
                .map(|d| d.action())
                .collect::<Vec<_>>(),
            [action],
            "{source}"
        );
    }
    for source in [
        "",
        "from",
        "{}",
        "@keyframes k{from{}}",
        "from{} to{}",
        "from{} ;",
        "from{} }",
        "101%{}",
        "from{opacity:bad} to{}",
    ] {
        let report = CssParsedKeyframeRule::parse(&CssParser::default(), source);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput),
            "{report:?}"
        );
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::DropDeclaration)
        );
    }
    let trailing = "from{opacity:bad} to{}";
    let report = CssParsedKeyframeRule::parse(&CssParser::default(), trailing);
    assert_eq!(
        report.diagnostics()[0]
            .error()
            .position()
            .byte_offset()
            .value(),
        trailing.find("to").unwrap()
    );
}

#[test]
fn empty_recovered_empty_and_eof_closed_detached_blocks_remain_distinct() {
    let clean = CssParsedKeyframeRule::parse(&CssParser::default(), "from{}");
    assert!(clean.is_clean());
    assert!(
        clean
            .syntax()
            .as_ref()
            .unwrap()
            .block()
            .declarations()
            .is_empty()
    );
    let recovered = CssParsedKeyframeRule::parse(&CssParser::default(), "from{mystery:1}");
    assert!(!recovered.is_clean());
    assert!(
        recovered
            .syntax()
            .as_ref()
            .unwrap()
            .block()
            .declarations()
            .is_empty()
    );
    let source = "from{opacity:1";
    let eof = CssParsedKeyframeRule::parse(&CssParser::default(), source);
    assert!(eof.syntax().is_some(), "{eof:?}");
    assert!(!eof.is_clean());
    assert_eq!(
        eof.syntax()
            .as_ref()
            .unwrap()
            .origin()
            .span()
            .end()
            .byte_offset()
            .value(),
        source.len()
    );
    assert!(
        eof.diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    assert!(eof.into_validation_result().is_err());
    let comment = CssParsedKeyframeSelectors::parse(&CssParser::default(), "from /*open");
    assert!(comment.syntax().is_some());
    assert!(
        comment
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::IgnoreUnterminatedComment)
    );
}

#[test]
fn symbolic_percentages_preserve_authored_math_without_range_evaluation() {
    let source = "calc(120%), calc(-10%), from, clamp(0%, 50%, 100%)";
    let report = CssParsedKeyframeSelectors::parse(&CssParser::default(), source);
    assert!(report.is_clean(), "{report:?}");
    let parsed = report.syntax().as_ref().unwrap();
    assert!(
        parsed
            .selectors()
            .offsets()
            .next()
            .unwrap()
            .literal_value()
            .is_none()
    );
    assert_eq!(
        parsed
            .selectors()
            .serialize_key_text(Default::default())
            .unwrap(),
        "calc(120%), calc(-10%), 0%, calc(50%)"
    );
    assert_eq!(parsed.origin().source().as_str(), source);
    assert!(
        CssParsedKeyframeRule::parse(&CssParser::default(), "calc(120%), calc(-10%){}").is_clean()
    );
}

#[test]
fn selector_depth_and_genuine_rule_body_share_the_fixed_parser_ceiling() {
    let at_limit = format!("calc({}1%{})", "(".repeat(255), ")".repeat(255));
    assert!(CssParsedKeyframeSelectors::parse(&CssParser::default(), &at_limit).is_clean());
    let past_limit = format!("calc({}1%{})", "(".repeat(256), ")".repeat(256));
    let rejected = CssParsedKeyframeSelectors::parse(&CssParser::default(), &past_limit);
    assert!(rejected.syntax().is_none());
    assert!(
        rejected
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::StopAtNestingLimit)
    );
    for (depth, admitted) in [(255, true), (256, false)] {
        let source = format!("from{{--x:{}1{}}}", "(".repeat(depth), ")".repeat(depth));
        let report = CssParsedKeyframeRule::parse(&CssParser::default(), &source);
        let block = report.syntax().as_ref().unwrap().block();
        assert_eq!(
            block.declarations().len(),
            usize::from(admitted),
            "{depth}: {report:?}"
        );
        assert_eq!(report.is_clean(), admitted);
        if !admitted {
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|d| d.action() == CssRecoveryAction::StopAtNestingLimit)
            );
        }
    }
    assert!(
        CssParsedKeyframeRule::parse(&CssParser::default(), "from{}").is_clean(),
        "a fresh retry has no spent parser depth"
    );
}

#[test]
fn keytext_cumulative_limits_count_duplicates_and_symbolic_work_and_allow_atomic_retry() {
    // List + each of three selectors. Literal formatting adds no extra nodes.
    let literals = selectors("from, 50%, 50%");
    let expected = "0%, 50%, 50%";
    let before = literals.clone();
    let adequate = Limits::new(4, 4, expected.len());
    assert_eq!(literals.serialize_key_text(adequate).unwrap(), expected);
    for (limits, kind) in [
        (Limits::new(3, 4, expected.len()), Resource::InputNodeLimit),
        (
            Limits::new(4, 3, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(4, 4, expected.len() - 1), Resource::ByteLimit),
        (Limits::new(0, 4, expected.len()), Resource::InputNodeLimit),
        (
            Limits::new(4, 0, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(4, 4, 0), Resource::ByteLimit),
    ] {
        assert_eq!(
            serialization_failure(&literals.serialize_key_text(limits).unwrap_err()),
            kind
        );
        assert_eq!(literals, before);
        assert_eq!(literals.serialize_key_text(adequate).unwrap(), expected);
    }
    // Two selectors each cost 1 + the calc function/leaf's two inputs and one projection.
    let symbolic = selectors("calc(120%), calc(-10%)");
    let expected = "calc(120%), calc(-10%)";
    assert_eq!(
        symbolic
            .serialize_key_text(Limits::new(7, 5, expected.len()))
            .unwrap(),
        expected
    );
    assert!(
        symbolic
            .serialize_key_text(Limits::new(6, 5, expected.len()))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        symbolic
            .serialize_key_text(Limits::new(7, 4, expected.len()))
            .unwrap_err()
            .is_projection_node_limit()
    );
}

#[test]
fn checked_assembly_formats_programmatic_inputs_without_manufacturing_source_positions() {
    let source = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Opacity)),
        CssComponentValues::parse(&CssParser::default(), "1").unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(source.position(), None);
    let expansion = source.expand().unwrap();
    let declarations = CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(
        &[expansion],
        Default::default(),
    )
    .unwrap();
    let selectors = CssKeyframeSelectorList::try_new(vec![
        CssKeyframeSelector::From,
        CssKeyframeSelector::Percent(CssKeyframePercent::try_new(0.0).unwrap()),
        CssKeyframeSelector::To,
    ])
    .unwrap();
    let view = CssKeyframeRuleView::try_new(&selectors, &declarations).unwrap();
    assert_eq!(
        view.serialize_cssom(Default::default()).unwrap(),
        "0%, 0%, 100% { opacity: 1; }"
    );
    assert!(std::ptr::eq(view.selectors(), &selectors));
    assert!(std::ptr::eq(view.declarations(), &declarations));
    assert_eq!(declarations.entries()[0].source().position(), None);
    assert!(declarations.entries()[0].source().same_occurrence(&source));
}

#[test]
fn checked_assembly_preserves_keyframe_domain_including_empty_blocks() {
    let selectors = selectors("from");
    let ordinary = CssSpecifiedDeclarationBlock::try_from_declarations(
        CssParser::default().parse_block_contents("").syntax(),
        Default::default(),
    )
    .unwrap();
    assert!(
        CssKeyframeRuleView::try_new(&selectors, &ordinary)
            .as_ref()
            .is_err_and(|error| error.is_non_keyframe_declarations())
    );
    let parsed = CssParsedKeyframeRule::parse(&CssParser::default(), "from{}");
    let declarations = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
        parsed.syntax().as_ref().unwrap().block().declarations(),
        Default::default(),
    )
    .unwrap();
    let view = CssKeyframeRuleView::try_new(&selectors, &declarations).unwrap();
    assert_eq!(view.serialize_cssom(Default::default()).unwrap(), "0% { }");
    for (source, reason) in [
        (
            "opacity:1!important",
            CssInvalidKeyframeSourceReason::Importance,
        ),
        ("animation-name:x", CssInvalidKeyframeSourceReason::Property),
    ] {
        let report = CssParser::default().parse_block_contents(source);
        let expansion = report.syntax()[0].expand().unwrap();
        let error = CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(
            &[expansion],
            Default::default(),
        )
        .unwrap_err();
        assert!(error.invalid_keyframe_source() == Some(reason));
    }
}

#[test]
fn detached_formatting_budget_is_cumulative_across_keytext_braces_and_declarations() {
    let report = CssParsedKeyframeRule::parse(&CssParser::default(), "from,0%{--x:1}");
    let parsed = report.syntax().as_ref().unwrap();
    let declarations = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
        parsed.block().declarations(),
        Default::default(),
    )
    .unwrap();
    let view = CssKeyframeRuleView::try_new(parsed.block().selectors(), &declarations).unwrap();
    let expected = "0%, 0% { --x: 1; }";
    let before = report.clone();
    // Rule/list/two selectors = 4; custom declaration aggregate/occurrence/name/component-list/token = 5.
    let adequate = Limits::new(9, 9, expected.len());
    assert_eq!(view.serialize_cssom(adequate).unwrap(), expected);
    for (limits, resource) in [
        (Limits::new(8, 9, expected.len()), Resource::InputNodeLimit),
        (
            Limits::new(9, 8, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(9, 9, expected.len() - 1), Resource::ByteLimit),
    ] {
        let error = view.serialize_cssom(limits).unwrap_err();
        assert_eq!(exhausted(&error), resource, "{error:?}");
        assert_eq!(report, before);
        assert_eq!(view.serialize_cssom(adequate).unwrap(), expected);
    }
    let empty = CssParsedKeyframeRule::parse(&CssParser::default(), "to{}");
    let declarations = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
        empty.syntax().as_ref().unwrap().block().declarations(),
        Default::default(),
    )
    .unwrap();
    let list = selectors("to");
    let view = CssKeyframeRuleView::try_new(&list, &declarations).unwrap();
    assert_eq!(
        view.serialize_cssom(Limits::new(4, 4, 8)).unwrap(),
        "100% { }"
    );
}

#[test]
fn parser_context_preserves_owning_keyframe_declaration_admission() {
    let source = "from{width:1}";
    // Values 4 #lengths: outside quirks mode only zero may omit its unit.
    ExpectedRejection::new(CssErrorCode::InvalidPropertyValue)
        .named("width")
        .at("1")
        .assert_first(&CssParsedKeyframeRule::parse(&CssParser::default(), source));
    let context = CssParserContext::new(CssParserMode::Quirks);
    let report = CssParsedKeyframeRule::parse(&CssParser::new(context, Default::default()), source);
    assert!(report.is_clean(), "{report:?}");
    let declaration = &report.syntax().as_ref().unwrap().block().declarations()[0];
    assert_eq!(declaration.source().parser_context(), context);
    let declarations = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
        report.syntax().as_ref().unwrap().block().declarations(),
        Default::default(),
    )
    .unwrap();
    assert_eq!(
        declarations.serialize_cssom(Default::default()).unwrap(),
        "width: 1px;"
    );
}

#[test]
fn normalized_matching_inputs_retain_every_duplicate_rule_candidate() {
    let report = surgeist_css::rule::CssBlockFragment::<
        Vec<surgeist_css::rule::keyframes::CssKeyframeBlock>,
    >::parse(
        &surgeist_css::CssParser::default(),
        "{from{} 0%{} from{} to{}}",
        &surgeist_css::rule::CssBlockContext::default(),
    );
    assert!(report.is_clean());
    let candidates = report.syntax().as_ref().unwrap().body();
    assert_eq!(candidates.len(), 4);
    let target = selectors("0%").offsets().collect::<Vec<_>>();
    let matching = candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            (candidate.selectors().offsets().collect::<Vec<_>>() == target).then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(matching, vec![0, 1, 2]);
    assert!(matches!(
        candidates[0].selectors().selectors()[0],
        CssKeyframeSelector::From
    ));
    assert!(matches!(
        candidates[1].selectors().selectors()[0],
        CssKeyframeSelector::Percent(_)
    ));
}

#[test]
fn selected_terminal_keyframe_assembly_keeps_original_source_observations() {
    let authored = CssParser::default().parse_block_contents("opacity:1!important");
    let original = &authored.syntax()[0];
    let ordinary =
        CssSpecifiedDeclarationBlock::try_from_declarations(authored.syntax(), Default::default())
            .unwrap();
    let selected = ordinary.entries()[0].with_importance(CssImportance::Normal);
    let checked =
        CssSpecifiedDeclarationBlock::try_from_keyframe_entries(&[selected], Default::default())
            .unwrap();
    let list = selectors("from");
    let view = CssKeyframeRuleView::try_new(&list, &checked).unwrap();
    assert_eq!(
        view.serialize_cssom(Default::default()).unwrap(),
        "0% { opacity: 1; }"
    );
    assert_eq!(checked.entries()[0].importance(), CssImportance::Normal);
    assert_eq!(
        checked.entries()[0].source().importance(),
        CssImportance::Important
    );
    assert!(checked.entries()[0].source().same_occurrence(original));
    assert_eq!(
        checked.entries()[0].source().position(),
        original.position()
    );
    assert_eq!(
        original.parsed_value().unwrap().source().as_str(),
        "opacity:1!important"
    );
    assert!(
        !CssParsedKeyframeRule::parse(&CssParser::default(), "from{opacity:1!important}")
            .is_clean(),
        "selected-phase assembly does not loosen authored grammar"
    );
}
