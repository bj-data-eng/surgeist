#![forbid(unsafe_code)]
//! Public-only Animations 1 and selected CSSOM detached keyframe contracts.

use surgeist_css::{
    CssDeclarationBlockErrorKind, CssImportance, CssInvalidKeyframeSourceReason,
    CssKeyframePercent, CssKeyframeRuleView, CssKeyframeRuleViewError, CssKeyframeSelector,
    CssKeyframeSelectorList, CssKnownProperty, CssParserContext, CssParserMode, CssPropertyNameRef,
    CssRecoveryAction, CssSpecifiedDeclarationBlock,
    CssSpecifiedValueSerializationErrorKind as Resource,
    CssSpecifiedValueSerializationLimits as Limits, expand_declaration, parse_component_values,
    parse_keyframe_rule, parse_keyframe_selector_list, parse_property_value, parse_style_attribute,
};

fn selectors(source: &str) -> CssKeyframeSelectorList {
    let report = parse_keyframe_selector_list(source);
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().as_ref().unwrap().selectors().clone()
}

#[test]
fn raw_keytext_preserves_complete_source_order_duplicates_and_endpoint_observations() {
    let source = " /*😀*/\r\nFROM /*a*/, 0%, to, 100%, 50%, 50% /*z*/ ";
    let report = parse_keyframe_selector_list(source);
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
        parsed.selectors().serialize_key_text().unwrap(),
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
            .serialize_key_text()
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
        let report = parse_keyframe_selector_list(source);
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
    let report = parse_keyframe_rule(source);
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
        parsed.block().selectors().serialize_key_text().unwrap(),
        "0%, 0%, 100%"
    );
}

#[test]
fn detached_rule_recovers_declarations_but_rejects_invalid_outer_or_trailing_input() {
    let source = "from{opacity:0; mystery:1; opacity:2 !important; animation-name:x; opacity:1}";
    let report = parse_keyframe_rule(source);
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
    for source in [
        "",
        "from",
        "{}",
        "@keyframes k{from{}}",
        "from{} to{}",
        "from{} ;",
        "from{} }",
        "101%{}",
        "from{.x{opacity:0}}",
        "from{@media all{opacity:0}}",
        "from{opacity:bad} to{}",
    ] {
        let report = parse_keyframe_rule(source);
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
    let report = parse_keyframe_rule(trailing);
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
    let clean = parse_keyframe_rule("from{}");
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
    let recovered = parse_keyframe_rule("from{mystery:1}");
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
    let eof = parse_keyframe_rule(source);
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
    let comment = parse_keyframe_selector_list("from /*open");
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
    let report = parse_keyframe_selector_list(source);
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
        parsed.selectors().serialize_key_text().unwrap(),
        "calc(120%), calc(-10%), 0%, calc(50%)"
    );
    assert_eq!(parsed.origin().source().as_str(), source);
    assert!(parse_keyframe_rule("calc(120%), calc(-10%){}").is_clean());
}

#[test]
fn selector_depth_and_genuine_rule_body_share_the_fixed_parser_ceiling() {
    let at_limit = format!("calc({}1%{})", "(".repeat(255), ")".repeat(255));
    assert!(parse_keyframe_selector_list(&at_limit).is_clean());
    let past_limit = format!("calc({}1%{})", "(".repeat(256), ")".repeat(256));
    let rejected = parse_keyframe_selector_list(&past_limit);
    assert!(rejected.syntax().is_none());
    assert!(
        rejected
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::StopAtNestingLimit)
    );
    for (depth, admitted) in [(255, true), (256, false)] {
        let source = format!("from{{--x:{}1{}}}", "(".repeat(depth), ")".repeat(depth));
        let report = parse_keyframe_rule(&source);
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
        parse_keyframe_rule("from{}").is_clean(),
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
    assert_eq!(
        literals.serialize_key_text_with_limits(adequate).unwrap(),
        expected
    );
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
            literals
                .serialize_key_text_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(literals, before);
        assert_eq!(
            literals.serialize_key_text_with_limits(adequate).unwrap(),
            expected
        );
    }
    // Two selectors each cost 1 + the calc function/leaf's two inputs and one projection.
    let symbolic = selectors("calc(120%), calc(-10%)");
    let expected = "calc(120%), calc(-10%)";
    assert_eq!(
        symbolic
            .serialize_key_text_with_limits(Limits::new(7, 5, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        symbolic
            .serialize_key_text_with_limits(Limits::new(6, 5, expected.len()))
            .unwrap_err()
            .kind(),
        Resource::InputNodeLimit
    );
    assert_eq!(
        symbolic
            .serialize_key_text_with_limits(Limits::new(7, 4, expected.len()))
            .unwrap_err()
            .kind(),
        Resource::ProjectionNodeLimit
    );
}

#[test]
fn checked_assembly_formats_programmatic_inputs_without_manufacturing_source_positions() {
    let source = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Opacity),
        parse_component_values("1").unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(source.position(), None);
    let expansion = expand_declaration(&source).unwrap();
    let declarations =
        CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(&[expansion]).unwrap();
    let selectors = CssKeyframeSelectorList::try_new(vec![
        CssKeyframeSelector::From,
        CssKeyframeSelector::Percent(CssKeyframePercent::try_new(0.0).unwrap()),
        CssKeyframeSelector::To,
    ])
    .unwrap();
    let view = CssKeyframeRuleView::try_new(&selectors, &declarations).unwrap();
    assert_eq!(
        view.serialize_cssom().unwrap(),
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
    let ordinary =
        CssSpecifiedDeclarationBlock::try_from_declarations(parse_style_attribute("").syntax())
            .unwrap();
    assert!(matches!(
        CssKeyframeRuleView::try_new(&selectors, &ordinary),
        Err(CssKeyframeRuleViewError::NonKeyframeDeclarations)
    ));
    let parsed = parse_keyframe_rule("from{}");
    let declarations = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
        parsed.syntax().as_ref().unwrap().block().declarations(),
    )
    .unwrap();
    let view = CssKeyframeRuleView::try_new(&selectors, &declarations).unwrap();
    assert_eq!(view.serialize_cssom().unwrap(), "0% { }");
    for (source, reason) in [
        (
            "opacity:1!important",
            CssInvalidKeyframeSourceReason::Importance,
        ),
        ("animation-name:x", CssInvalidKeyframeSourceReason::Property),
    ] {
        let report = parse_style_attribute(source);
        let expansion = expand_declaration(&report.syntax()[0]).unwrap();
        let error =
            CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(&[expansion]).unwrap_err();
        assert!(
            matches!(error.kind(), CssDeclarationBlockErrorKind::InvalidKeyframeSource {reason: actual} if *actual == reason)
        );
    }
}

#[test]
fn detached_formatting_budget_is_cumulative_across_keytext_braces_and_declarations() {
    let report = parse_keyframe_rule("from,0%{--x:1}");
    let parsed = report.syntax().as_ref().unwrap();
    let declarations =
        CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(parsed.block().declarations())
            .unwrap();
    let view = CssKeyframeRuleView::try_new(parsed.block().selectors(), &declarations).unwrap();
    let expected = "0%, 0% { --x: 1; }";
    let before = report.clone();
    // Rule/list/two selectors = 4; custom declaration aggregate/occurrence/name/component-list/token = 5.
    let adequate = Limits::new(9, 9, expected.len());
    assert_eq!(
        view.serialize_cssom_with_limits(adequate).unwrap(),
        expected
    );
    for limits in [
        Limits::new(8, 9, expected.len()),
        Limits::new(9, 8, expected.len()),
        Limits::new(9, 9, expected.len() - 1),
    ] {
        assert!(view.serialize_cssom_with_limits(limits).is_err());
        assert_eq!(report, before);
        assert_eq!(
            view.serialize_cssom_with_limits(adequate).unwrap(),
            expected
        );
    }
    let empty = parse_keyframe_rule("to{}");
    let declarations = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
        empty.syntax().as_ref().unwrap().block().declarations(),
    )
    .unwrap();
    let list = selectors("to");
    let view = CssKeyframeRuleView::try_new(&list, &declarations).unwrap();
    assert_eq!(
        view.serialize_cssom_with_limits(Limits::new(4, 4, 8))
            .unwrap(),
        "100% { }"
    );
}

#[test]
fn parser_context_preserves_owning_keyframe_declaration_admission() {
    let source = "from{width:1}";
    assert!(!parse_keyframe_rule(source).is_clean());
    let context = CssParserContext::new(CssParserMode::Quirks);
    let report = context.parse_keyframe_rule(source);
    assert!(report.is_clean(), "{report:?}");
    let declaration = &report.syntax().as_ref().unwrap().block().declarations()[0];
    assert_eq!(declaration.source().parser_context(), context);
    let declarations = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
        report.syntax().as_ref().unwrap().block().declarations(),
    )
    .unwrap();
    assert_eq!(declarations.serialize_cssom().unwrap(), "width: 1px;");
}

#[test]
fn normalized_matching_inputs_retain_every_duplicate_rule_candidate() {
    let report = surgeist_css::parse_keyframes_block("{from{} 0%{} from{} to{}}");
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
    let authored = parse_style_attribute("opacity:1!important");
    let original = &authored.syntax()[0];
    let ordinary = CssSpecifiedDeclarationBlock::try_from_declarations(authored.syntax()).unwrap();
    let selected = ordinary.entries()[0].with_importance(CssImportance::Normal);
    let checked = CssSpecifiedDeclarationBlock::try_from_keyframe_entries(&[selected]).unwrap();
    let list = selectors("from");
    let view = CssKeyframeRuleView::try_new(&list, &checked).unwrap();
    assert_eq!(view.serialize_cssom().unwrap(), "0% { opacity: 1; }");
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
        !parse_keyframe_rule("from{opacity:1!important}").is_clean(),
        "selected-phase assembly does not loosen authored grammar"
    );
}
