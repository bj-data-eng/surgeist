//! Independent checked-model and canonical-output witnesses for MQ5 custom media.
use surgeist_css::*;

fn components(css: &str) -> CssComponentValues {
    parse_component_values(css).unwrap()
}
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
        let rule = CssCustomMediaRule::try_new(name.clone(), CssCustomMediaBody::True).unwrap();
        let css = rule.serialize().unwrap();
        let parsed = definition(css.as_css());
        assert_eq!(parsed.name().as_str(), name.as_str());
        assert_eq!(rule.position(), None);
        assert!(matches!(rule.origin(), CssValueOrigin::Programmatic));
    }
    for name in ["ordinary", "-x", ""] {
        assert!(CssCustomMediaName::try_new(name).is_err());
    }
    let number = components("2").items()[0].clone();
    let origin = number.origin().clone();
    assert!(
        matches!(CssCustomMediaName::try_from_component(number),Err(CssCustomMediaConstructionError::InvalidName{origin:actual})if actual==origin)
    );
}

#[test]
fn component_prelude_preserves_name_and_boolean_origins_without_inventing_at_keyword() {
    let input = components(r"--\6d ode TRUE");
    let name_origin = input.items()[0].origin().clone();
    let body_origin = input.items().last().unwrap().origin().clone();
    let rule = CssCustomMediaRule::try_from_components(input).unwrap();
    assert_eq!(rule.name().as_str(), "--mode");
    assert!(matches!(rule.body(), CssCustomMediaBody::True));
    assert_eq!(rule.position(), None);
    assert_eq!(rule.name().origin(), &name_origin);
    let output = rule.serialize().unwrap();
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
            matches!(CssCustomMediaRule::try_new(programmatic_name(),CssCustomMediaBody::Media(media)),Err(CssCustomMediaConstructionError::AmbiguousMediaBody{origin})if origin==expected),
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
        )
        .unwrap();
        assert!(
            matches!(
                definition(rule.serialize().unwrap().as_css()).body(),
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
    ] {
        let rule = CssCustomMediaRule::try_from_components(components(input)).unwrap();
        assert_eq!(rule.serialize().unwrap().as_css(), expected);
    }
    assert!(matches!(
        CssCustomMediaRule::try_from_components(components("")),
        Err(CssCustomMediaConstructionError::InvalidPrelude { .. })
    ));
    for input in ["--x ,screen", "--x screen,", "--x (--x: 1)"] {
        assert!(
            matches!(
                CssCustomMediaRule::try_from_components(components(input)),
                Err(CssCustomMediaConstructionError::Media(_))
            ),
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
        CssComponentValues::try_new(vec![name.clone()]).unwrap(),
    )
    .unwrap();
    let condition = CssMediaCondition::try_from_components(
        CssComponentValues::try_new(vec![block.clone()]).unwrap(),
    )
    .unwrap();
    let CssMediaConditionKind::CustomMediaReference(reference) = condition.kind() else {
        panic!("symbolic reference")
    };
    assert_eq!(reference.name().as_str(), "--x");
    assert_eq!(reference.name().origin(), &expected);
    assert_eq!(reference.position(), None);
    assert_eq!(reference.component(), &block);
    let output = condition.serialize().unwrap();
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
    for source in ["(--x: 1)", "not ((--x: 1))", "(1 < --x)"] {
        assert!(
            CssMediaCondition::try_from_components(components(source)).is_err(),
            "{source}"
        );
    }
}

#[test]
fn parsed_rule_grammar_tokens_and_eof_terminators_keep_honest_origins() {
    let rule = definition("/* 😀 */ @CUSTOM-MEDIA --x FALSE;");
    let out = rule.serialize().unwrap();
    assert_eq!(out.as_css(), "@custom-media --x false;");
    assert_eq!(rule.position().unwrap().byte_offset().value(), 11);
    assert!(
        matches!(out.origin_at(0),Some(CssSerializedOrigin::Token(origin))if origin==rule.origin())
    );
    assert!(
        matches!(out.origin_at(out.as_css().len()-1),Some(CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)))if origin.span().start().byte_offset().value()==34)
    );
    let eof = definition("@custom-media --x true").serialize().unwrap();
    assert!(matches!(
        eof.origin_at(eof.as_css().len() - 1),
        Some(CssSerializedOrigin::Token(CssValueOrigin::Programmatic))
    ));
}

#[test]
fn canonical_rule_output_preserves_exact_values_and_opaque_token_boundaries() {
    let rule = definition("@CUSTOM-MEDIA --Case (WIDTH>=+001.5PX), (ASPECT-RATIO:2);");
    let output = rule.serialize().unwrap();
    assert_eq!(
        output.as_css(),
        "@custom-media --Case (width >= +001.5PX), (aspect-ratio: 2 / 1);"
    );
    assert_eq!(
        definition(output.as_css()).serialize().unwrap().as_css(),
        output.as_css()
    );
    let opaque = CssComponentValue::try_function(
        "Future",
        CssComponentValues::try_new(vec![
            CssComponentValue::try_number("1").unwrap(),
            CssComponentValue::try_ident("e2").unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    let query =
        CssMediaQuery::try_from_components(CssComponentValues::try_new(vec![opaque]).unwrap())
            .unwrap();
    let rule = CssCustomMediaRule::try_new(
        programmatic_name(),
        CssCustomMediaBody::Media(CssMediaQueryList::try_new(vec![query]).unwrap()),
    )
    .unwrap();
    assert_eq!(
        rule.serialize().unwrap().as_css(),
        "@custom-media --mode Future(1/**/e2);"
    );
}

#[test]
fn recovered_members_and_implicit_components_are_rejected_by_checked_construction() {
    let report = parse_media_query_list("(color), ???, screen");
    assert!(!report.is_clean());
    let never = report.syntax().queries()[1].origin().clone();
    assert!(
        matches!(CssCustomMediaRule::try_new(programmatic_name(),CssCustomMediaBody::Media(report.syntax().clone())),Err(CssCustomMediaConstructionError::RecoveredInput{origin})if origin==never)
    );
    for source in ["--x (color", "--x true/*"] {
        assert!(
            matches!(
                CssCustomMediaRule::try_from_components(components(source)),
                Err(CssCustomMediaConstructionError::RecoveredInput { .. })
            ),
            "{source}"
        );
    }
    let recovered = parse_media_query_list("(color");
    assert!(matches!(
        CssCustomMediaRule::try_new(
            programmatic_name(),
            CssCustomMediaBody::Media(recovered.syntax().clone())
        ),
        Err(CssCustomMediaConstructionError::RecoveredInput { .. })
    ));
}

#[test]
fn parsed_recovery_prevents_partial_output_but_discarded_blocks_do_not_leak_member_diagnostics() {
    let report = parse_sheet("@custom-media --x (color), ???, screen;");
    let [CssRule::CustomMedia(rule)] = report.syntax().rules() else {
        panic!("retained definition")
    };
    assert!(matches!(
        rule.serialize(),
        Err(CssCustomMediaSerializationError::Media(
            CssMediaSerializationError::RecoveredNever { .. }
        ))
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
            CssComponentValueErrorKind::NestingLimit,
        ),
        (
            "--x true",
            CssComponentValueLimits::try_new(256, 1, usize::MAX).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            "--x true",
            CssComponentValueLimits::try_new(256, usize::MAX, 3).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let values = components(input);
        let Err(CssCustomMediaConstructionError::Component(error)) =
            CssCustomMediaRule::try_from_components_with_limits(values, limits)
        else {
            panic!("typed {kind:?}")
        };
        assert_eq!(error.kind(), kind);
        assert!(matches!(error.origin(), CssValueOrigin::Parsed(_)));
    }
    let input = components("--x true");
    let expected = input.items()[2].origin().clone();
    let limits = CssComponentValueLimits::try_new(256, usize::MAX, 19).unwrap();
    let Err(CssCustomMediaConstructionError::Component(error)) =
        CssCustomMediaRule::try_from_components_with_limits(input, limits)
    else {
        panic!("canonical expansion exceeds 19 bytes")
    };
    assert_eq!(error.kind(), CssComponentValueErrorKind::ByteLimit);
    assert_eq!(error.origin(), &expected);
    let limits = CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap();
    assert!(
        matches!(CssCustomMediaRule::try_new_with_limits(programmatic_name(),CssCustomMediaBody::Media(list("(color)")),limits),Err(CssCustomMediaConstructionError::Component(error))if error.kind()==CssComponentValueErrorKind::NestingLimit)
    );
}

#[test]
fn deep_checked_definition_admission_and_serialization_fit_an_ordinary_stack() {
    const CHILD: &str = "SURGEIST_CUSTOM_MEDIA_STACK_CHILD";
    const TEST: &str = "deep_checked_definition_admission_and_serialization_fit_an_ordinary_stack";
    if std::env::var(CHILD).as_deref() != Ok(TEST) {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
            .env(CHILD, TEST)
            .env_remove("RUST_MIN_STACK")
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for terminal in ["color", "--external"] {
                let query = format!("{}{terminal}{}", "(".repeat(256), ")".repeat(256));
                let components = components(&format!("--deep {query}"));
                let rule = CssCustomMediaRule::try_from_components(components).unwrap();
                assert_eq!(
                    rule.serialize().unwrap().as_css(),
                    format!("@custom-media --deep {query};")
                );
            }
        })
        .unwrap()
        .join()
        .unwrap();
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
    assert!(matches!(
        report.diagnostics()[0].error().kind(),
        ErrorKind::InvalidComponentValue(_)
    ));
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
    let rule = CssCustomMediaRule::try_from_components(components("--x (--other)")).unwrap();
    assert!(
        matches!(rule.body(),CssCustomMediaBody::Media(list) if matches!(&list.queries()[0],CssMediaQuery::Condition(c) if matches!(c.kind(),CssMediaConditionKind::CustomMediaReference(_))))
    );
    assert!(matches!(
        definition(rule.serialize().unwrap().as_css()).body(),
        CssCustomMediaBody::Media(_)
    ));
}
