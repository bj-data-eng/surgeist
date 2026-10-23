//! Independent checked-model and canonical-output witnesses for MQ5 custom media.
use crate::css::support::authored_syntax::assert_diagnostics;
use crate::css::support::component_values as components;
use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::failure_classes::component_failure;
use surgeist_css::*;

fn list(css: &str) -> CssMediaQueryList {
    let report = parse_media_query_list(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    report.syntax().clone()
}
fn definition(css: &str) -> CssCustomMediaRule {
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [CssRule::CustomMedia(rule)] = report.syntax().rules() else {
        panic!("definition")
    };
    rule.clone()
}
fn programmatic_name() -> CssCustomMediaName {
    CssCustomMediaName::try_new("--mode").unwrap()
}

#[test]
fn decoded_names_escape_without_changing_extension_identity() {
    for name in ["--", "--Case", "--é", "--two words"] {
        let name = CssCustomMediaName::try_new(name).unwrap();
        assert_eq!(name.position(), None);
        let rule =
            CssCustomMediaRule::try_new(name.clone(), CssCustomMediaBody::True, Default::default())
                .unwrap();
        let css = rule.serialize_tokens(Default::default()).unwrap();
        let parsed = definition(css.as_css());
        assert_eq!(parsed.name().as_str(), name.as_str());
        assert_eq!(rule.position(), None);
        assert!(matches!(rule.origin(), CssValueOrigin::Programmatic));
    }
    // `<extension-name>` is a dashed ident; the empty string is no ident.
    for name in ["ordinary", "-x"] {
        assert!(
            CssCustomMediaName::try_new(name)
                .as_ref()
                .is_err_and(|error| error.is_invalid_name()
                    && error.origin() == &CssValueOrigin::Programmatic),
            "{name:?}"
        );
    }
    let Err(outer) = CssCustomMediaName::try_new("") else {
        panic!("empty name")
    };
    let Some(error) = outer.component_error() else {
        panic!("empty name")
    };
    assert!(error.is_invalid_identifier(), "{error:?}");
    assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
    let number = components("2").items()[0].clone();
    let origin = number.origin().clone();
    assert!(
        CssCustomMediaName::try_from_component(number)
            .is_err_and(|error| error.is_invalid_name() && *error.origin() == origin)
    );
}

#[test]
fn component_prelude_preserves_name_and_boolean_origins_without_inventing_at_keyword() {
    let input = components(r"--\6d ode TRUE");
    let name_origin = input.items()[0].origin().clone();
    let body_origin = input.items().last().unwrap().origin().clone();
    let rule = CssCustomMediaRule::try_from_components(input, Default::default()).unwrap();
    assert_eq!(rule.name().as_str(), "--mode");
    assert!(matches!(rule.body(), CssCustomMediaBody::True));
    assert_eq!(rule.position(), None);
    assert_eq!(rule.name().origin(), &name_origin);
    let output = rule.serialize_tokens(Default::default()).unwrap();
    assert_eq!(output.as_css(), r"@custom-media --\6d ode true;");
    assert!(matches!(
        output.origin_at(0),
        Some(CssSerializedOrigin::Token(CssValueOrigin::Programmatic))
    ));
    assert!(
        matches!(output.origin_at(14),Some(CssSerializedOrigin::Token(origin))if origin==&name_origin)
    );
    assert!(
        matches!(output.origin_at(output.as_css().find("true").unwrap()),Some(CssSerializedOrigin::Token(origin))if origin==&body_origin)
    );
}

#[test]
fn standalone_boolean_media_bodies_are_rejected_with_the_exact_type_origin() {
    for source in ["true", "FALSE", r"\74 rue", r"\46 alse"] {
        let media = list(source);
        let expected = media.queries()[0].origin().clone();
        assert!(
            CssCustomMediaRule::try_new(
                programmatic_name(),
                CssCustomMediaBody::Media(media),
                Default::default()
            )
            .is_err_and(|error| error.is_ambiguous_media_body() && *error.origin() == expected),
            "{source}"
        );
    }
    for source in [
        "",
        "only true",
        "not false",
        "true and (color)",
        "true, false",
    ] {
        let rule = CssCustomMediaRule::try_new(
            programmatic_name(),
            CssCustomMediaBody::Media(list(source)),
            Default::default(),
        )
        .unwrap();
        assert!(
            matches!(
                definition(rule.serialize_tokens(Default::default()).unwrap().as_css()).body(),
                CssCustomMediaBody::Media(_)
            ),
            "{source}"
        );
    }
}

#[test]
fn component_boolean_selection_and_empty_media_body_remain_distinct() {
    for (input, expected) in [
        ("--x true", "@custom-media --x true;"),
        ("--x FALSE", "@custom-media --x false;"),
        ("--x", "@custom-media --x;"),
        ("--x true, false", "@custom-media --x true, false;"),
        // Syntax 3 (2026-10-01) §5.4.10: a final comma adds no member.
        ("--x screen,", "@custom-media --x screen;"),
    ] {
        let rule =
            CssCustomMediaRule::try_from_components(components(input), Default::default()).unwrap();
        assert_eq!(
            rule.serialize_tokens(Default::default()).unwrap().as_css(),
            expected
        );
    }
    assert!(
        CssCustomMediaRule::try_from_components(components(""), Default::default())
            .as_ref()
            .is_err_and(|error| error.is_invalid_prelude())
    );
    for input in ["--x ,screen", "--x screen, ", "--x (--x: 1)"] {
        assert!(
            CssCustomMediaRule::try_from_components(components(input), Default::default())
                .is_err_and(|error| error.is_invalid_query_grammar()
                    || error.is_invalid_condition_grammar()),
            "{input}"
        );
    }
}

#[test]
fn reference_inspection_agrees_across_parsed_and_mixed_origin_checked_admission() {
    let parsed = components(r"--\78");
    let name = parsed.items()[0].clone();
    let expected = name.origin().clone();
    let block = CssComponentValue::try_block(
        CssBlockKind::Parenthesis,
        CssComponentValues::try_new(vec![name.clone()], Default::default()).unwrap(),
    )
    .unwrap();
    let condition = CssMediaCondition::try_from_components(
        CssComponentValues::try_new(vec![block.clone()], Default::default()).unwrap(),
        Default::default(),
    )
    .unwrap();
    let CssMediaConditionKind::CustomMediaReference(reference) = condition.kind() else {
        panic!("symbolic reference")
    };
    assert_eq!(reference.name().as_str(), "--x");
    assert_eq!(reference.name().origin(), &expected);
    assert_eq!(reference.position(), None);
    assert_eq!(reference.component(), &block);
    let output = condition.serialize_tokens(Default::default()).unwrap();
    assert_eq!(output.as_css(), r"(--\78)");
    let again = parse_media_query(output.as_css());
    assert!(again.is_clean());
    let CssMediaQuery::Condition(condition) = again.syntax() else {
        panic!("condition")
    };
    assert!(matches!(
        condition.kind(),
        CssMediaConditionKind::CustomMediaReference(_)
    ));
    // A dashed name is only a boolean custom-media reference, so as a
    // feature name that name first violates the condition, in either form.
    for (source, responsible) in [
        ("(--x: 1)", Some(1)),
        ("not ((--x: 1))", Some(6)),
        ("(1 < --x)", Some(5)),
    ] {
        let Err(error) =
            CssMediaCondition::try_from_components(components(source), Default::default())
        else {
            panic!(
                "{source}: {:?}",
                CssMediaCondition::try_from_components(components(source), Default::default())
            )
        };
        assert!(error.is_invalid_condition_grammar(), "{source}: {error:?}");
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("{source}: parsed failing token")
        };
        assert_eq!(origin.source().as_str(), source);
        if let Some(responsible) = responsible {
            assert_eq!(
                origin.span().start().byte_offset().value(),
                responsible,
                "{source}"
            );
        }
    }
}

#[test]
fn parsed_rule_grammar_tokens_and_eof_terminators_keep_honest_origins() {
    let rule = definition("/* 😀 */ @CUSTOM-MEDIA --x FALSE;");
    let out = rule.serialize_tokens(Default::default()).unwrap();
    assert_eq!(out.as_css(), "@custom-media --x false;");
    assert_eq!(rule.position().unwrap().byte_offset().value(), 11);
    assert!(
        matches!(out.origin_at(0),Some(CssSerializedOrigin::Token(origin))if origin==rule.origin())
    );
    assert!(
        matches!(out.origin_at(out.as_css().len()-1),Some(CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)))if origin.span().start().byte_offset().value()==34)
    );
    let source = "@custom-media --x true";
    let report = parse_sheet(source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one retained statement EOF diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainNonconformingRule
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.len()
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    let detail = diagnostic.error();
    assert_eq!(
        detail.code(),
        CssErrorCode::UnexpectedEnd,
        "typed statement EOF"
    );
    assert_eq!(
        detail.expectation().unwrap().as_str(),
        "a semicolon or block terminating an at-rule"
    );
    let [CssRule::CustomMedia(eof_rule)] = report.syntax().rules() else {
        panic!("retained definition");
    };
    assert_eq!(eof_rule.name().as_str(), "--x");
    assert!(matches!(eof_rule.body(), CssCustomMediaBody::True));
    let CssValueOrigin::Parsed(origin) = eof_rule.origin() else {
        panic!("original at-keyword");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    let eof = eof_rule.serialize_tokens(Default::default()).unwrap();
    assert!(matches!(
        eof.origin_at(0), Some(CssSerializedOrigin::Token(origin)) if origin == eof_rule.origin()
    ));
    assert!(matches!(
        eof.origin_at(eof.as_css().len() - 1),
        Some(CssSerializedOrigin::Token(CssValueOrigin::Programmatic))
    ));
    // Statement recovery does not invent a recovered body component. Checked
    // reconstruction owns a new semicolon; canonical terminated input is clean.
    let checked = CssCustomMediaRule::try_new(
        eof_rule.name().clone(),
        eof_rule.body().clone(),
        Default::default(),
    )
    .unwrap();
    assert_eq!(checked.name().origin(), eof_rule.name().origin());
    assert_eq!(
        checked
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        eof.as_css()
    );
    assert_eq!(
        definition(eof.as_css())
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        eof.as_css()
    );
    assert_eq!(
        validate_sheet(source),
        report.clone().into_validation_result()
    );
}

#[test]
fn canonical_rule_output_preserves_exact_values_and_opaque_token_boundaries() {
    let rule = definition("@CUSTOM-MEDIA --Case (WIDTH>=+001.5PX), (ASPECT-RATIO:2);");
    let output = rule.serialize_tokens(Default::default()).unwrap();
    assert_eq!(
        output.as_css(),
        "@custom-media --Case (width >= +001.5PX), (aspect-ratio: 2 / 1);"
    );
    assert_eq!(
        definition(output.as_css())
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        output.as_css()
    );
    let opaque = CssComponentValue::try_function(
        "Future",
        CssComponentValues::try_new(
            vec![
                CssComponentValue::try_number("1").unwrap(),
                CssComponentValue::try_ident("e2").unwrap(),
            ],
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap();
    let query = CssMediaQuery::try_from_components(
        CssComponentValues::try_new(vec![opaque], Default::default()).unwrap(),
        Default::default(),
    )
    .unwrap();
    let rule = CssCustomMediaRule::try_new(
        programmatic_name(),
        CssCustomMediaBody::Media(CssMediaQueryList::new(vec![query])),
        Default::default(),
    )
    .unwrap();
    assert_eq!(
        rule.serialize_tokens(Default::default()).unwrap().as_css(),
        "@custom-media --mode Future(1/**/e2);"
    );
}

#[test]
fn recovered_members_and_implicit_components_are_rejected_by_checked_construction() {
    let report = parse_media_query_list("(color), ???, screen");
    // The second member first fails at its `?` delimiter.
    assert_diagnostics(
        report.diagnostics(),
        &[(
            CssErrorCode::InvalidMediaQuery,
            CssRecoveryAction::ReplaceMediaQueryWithNever,
            "(color), ".len(),
        )],
    );
    let never = report.syntax().queries()[1].origin().clone();
    assert!(
        CssCustomMediaRule::try_new(
            programmatic_name(),
            CssCustomMediaBody::Media(report.syntax().clone()),
            Default::default()
        )
        .is_err_and(|error| error.is_recovered_input() && *error.origin() == never)
    );
    for source in ["--x (color", "--x true/*"] {
        assert!(
            CssCustomMediaRule::try_from_components(components(source), Default::default())
                .as_ref()
                .is_err_and(|error| error.is_recovered_input()),
            "{source}"
        );
    }
    let recovered = parse_media_query_list("(color");
    assert!(
        CssCustomMediaRule::try_new(
            programmatic_name(),
            CssCustomMediaBody::Media(recovered.syntax().clone()),
            Default::default(),
        )
        .as_ref()
        .is_err_and(|error| error.is_recovered_input())
    );
}

#[test]
fn parsed_recovery_prevents_partial_output_but_discarded_blocks_do_not_leak_member_diagnostics() {
    let report = parse_sheet("@custom-media --x (color), ???, screen;");
    let [CssRule::CustomMedia(rule)] = report.syntax().rules() else {
        panic!("retained definition")
    };
    assert!(matches!(
        rule.serialize_tokens(Default::default()),
        Err(ref error) if crate::css::support::is_recovered_media_failure(error)
    ));
    for source in [
        "@custom-media --x ??? {} .after{}",
        "@custom-media --x ??? { .discard{}",
    ] {
        let report = parse_sheet(source);
        assert_eq!(
            report.diagnostics().len(),
            1,
            "{source}: {:?}",
            report.diagnostics()
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
        assert!(
            !report
                .syntax()
                .rules()
                .iter()
                .any(|r| matches!(r, CssRule::CustomMedia(_)))
        );
    }
}

#[test]
fn selected_component_limits_apply_to_input_and_canonical_output_with_original_causes() {
    for (input, limits, kind) in [
        (
            "--x (color)",
            CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap(),
            ComponentFailure::NestingLimit,
        ),
        (
            "--x true",
            CssComponentValueLimits::try_new(256, 1, usize::MAX).unwrap(),
            ComponentFailure::ComponentLimit,
        ),
        (
            "--x true",
            CssComponentValueLimits::try_new(256, usize::MAX, 3).unwrap(),
            ComponentFailure::ByteLimit,
        ),
    ] {
        let values = components(input);
        let Err(outer) = CssCustomMediaRule::try_from_components(values, limits) else {
            panic!("typed {kind:?}")
        };
        let Some(error) = outer.component_error() else {
            panic!("typed {kind:?}")
        };
        assert_eq!(component_failure(error), kind);
        assert!(matches!(error.origin(), CssValueOrigin::Parsed(_)));
    }
    let input = components("--x true");
    let expected = input.items()[2].origin().clone();
    let limits = CssComponentValueLimits::try_new(256, usize::MAX, 19).unwrap();
    let Err(outer) = CssCustomMediaRule::try_from_components(input, limits) else {
        panic!("canonical expansion exceeds 19 bytes")
    };
    let Some(error) = outer.component_error() else {
        panic!("canonical expansion exceeds 19 bytes")
    };
    assert!(error.is_byte_limit(), "{error:?}");
    assert_eq!(error.origin(), &expected);
    let limits = CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap();
    assert!(
        CssCustomMediaRule::try_new(
            programmatic_name(),
            CssCustomMediaBody::Media(list("(color)")),
            limits
        )
        .as_ref()
        .is_err_and(|error| error
            .component_error()
            .is_some_and(|error| error.is_nesting_limit()))
    );
}

#[test]
fn deep_checked_definition_admission_and_serialization_fit_an_ordinary_stack() {
    const CHILD: &str = "SURGEIST_CUSTOM_MEDIA_STACK_CHILD";
    const TEST: &str = "css::media::custom_media_construction::deep_checked_definition_admission_and_serialization_fit_an_ordinary_stack";
    const COMPLETED: &str = "deep custom media admission and serialization completed";
    crate::css::support::isolated_stack::run(TEST, CHILD, COMPLETED, None, || {
        for terminal in ["color", "--external"] {
            let query = format!("{}{terminal}{}", "(".repeat(256), ")".repeat(256));
            let components = components(&format!("--deep {query}"));
            let rule =
                CssCustomMediaRule::try_from_components(components, Default::default()).unwrap();
            assert_eq!(
                rule.serialize_tokens(Default::default()).unwrap().as_css(),
                format!("@custom-media --deep {query};")
            );
        }
    });
}

#[test]
fn lexical_bad_members_recover_locally_inside_retained_definitions() {
    let source = "@custom-media --x url(a b), screen; .after{}";
    let report = parse_sheet(source);
    let [CssRule::CustomMedia(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("definition and sibling: {:?}", report)
    };
    let CssCustomMediaBody::Media(list) = rule.body() else {
        panic!("list body")
    };
    assert!(matches!(
        list.queries(),
        [CssMediaQuery::Never(_), CssMediaQuery::Typed(_)]
    ));
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    assert!(report.diagnostics()[0].error().code() == CssErrorCode::InvalidComponentValue);
}

#[test]
fn atomic_metadata_distinguishes_definition_rules_from_boolean_references() {
    for (id, kind, anchor) in [
        (
            "ext.rule.custom-media",
            CssFeatureKind::Rule,
            "#at-ruledef-custom-media",
        ),
        (
            "ext.media.custom-media",
            CssFeatureKind::MediaQuery,
            "#custom-mq",
        ),
    ] {
        let metadata = feature_metadata(id).unwrap();
        assert_eq!(metadata.kind(), kind);
        assert_eq!(metadata.status(), CssSupportStatus::Complete);
        assert_eq!(metadata.production(), anchor);
        assert_eq!(
            metadata.source().url(),
            Some("https://www.w3.org/TR/2026/WD-mediaqueries-5-20260219/")
        );
    }
    let rule =
        CssCustomMediaRule::try_from_components(components("--x (--other)"), Default::default())
            .unwrap();
    assert!(
        matches!(rule.body(),CssCustomMediaBody::Media(list) if matches!(&list.queries()[0],CssMediaQuery::Condition(c) if matches!(c.kind(),CssMediaConditionKind::CustomMediaReference(_))))
    );
    assert!(matches!(
        definition(rule.serialize_tokens(Default::default()).unwrap().as_css()).body(),
        CssCustomMediaBody::Media(_)
    ));
}
