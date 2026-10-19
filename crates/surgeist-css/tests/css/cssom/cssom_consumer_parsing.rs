#![forbid(unsafe_code)]
//! Public consumer oracles for the selected current Syntax block-contents witness
//! and CSSOM staged insertion algorithms. These are new boundaries: no executable
//! preimplementation RED is claimed. CSSOM exception/state policy stays downstream.
use surgeist_css::*;

fn candidate(source: &str) -> CssRuleSyntax {
    classify_rule_syntax(source)
        .into_parts()
        .0
        .expect("one generic rule")
}
fn admit(
    source: &str,
    context: CssRuleAdmissionContext,
) -> CssParseReport<Option<CssAdmittedRule>> {
    candidate(source).admit(&CssNamespaceContext::default(), context)
}
fn names(list: &CssDeclarationList) -> Vec<&str> {
    list.iter()
        .map(|d| match d.property_name() {
            CssPropertyNameRef::Known(p) => p.canonical_name(),
            CssPropertyNameRef::Custom(n) => n.as_str(),
            _ => panic!("ordinary name"),
        })
        .collect()
}
fn resource(report: &[CssRecoveryDiagnostic], expected: CssComponentValueErrorKind) {
    assert!(report.iter().any(|d| matches!(d.error().kind(), ErrorKind::InvalidComponentValue(error) if error.kind() == expected)), "{report:?}");
}

#[test]
fn preliminary_rule_classification_retains_semantically_rejected_at_rules() {
    for (source, name) in [
        ("@unknown {}", "unknown"),
        ("@import;", "import"),
        ("@\\69mport;", "import"),
    ] {
        let report = classify_rule_syntax(source);
        assert!(report.is_clean(), "{report:?}");
        let syntax = report.syntax().as_ref().unwrap();
        assert_eq!(syntax.kind(), CssRuleSyntaxKind::AtRule);
        assert_eq!(syntax.at_rule_name(), Some(name));
        let semantic = syntax.admit(
            &CssNamespaceContext::default(),
            CssRuleAdmissionContext::Stylesheet,
        );
        assert!(semantic.syntax().is_none());
        assert!(
            semantic
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
    }
    let qualified = candidate(".a {}");
    assert_eq!(qualified.kind(), CssRuleSyntaxKind::Qualified);
    assert_eq!(qualified.at_rule_name(), None);
    assert!(
        qualified
            .admit(
                &CssNamespaceContext::default(),
                CssRuleAdmissionContext::Stylesheet
            )
            .syntax()
            .is_some()
    );
}

#[test]
fn complete_input_rejects_empty_bodyless_custom_property_shaped_and_trailing_candidates() {
    for source in [
        "",
        " /**/ ",
        ".a",
        ".a {} .b {}",
        "@unknown {} .a {}",
        ".a {};",
        "--foo:hover {width:1px}",
    ] {
        let report = classify_rule_syntax(source);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
    }
    assert!(candidate("/**/ @unknown {} /**/").at_rule_name().is_some());
}

#[derive(Debug, Eq, PartialEq)]
enum InsertionFailure {
    Syntax,
    ConstructedImport,
    Index,
    Semantic,
}
fn sheet_stage(source: &str, constructed: bool, index_valid: bool) -> Result<(), InsertionFailure> {
    let syntax = classify_rule_syntax(source)
        .into_parts()
        .0
        .ok_or(InsertionFailure::Syntax)?;
    if constructed
        && syntax
            .at_rule_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("import"))
    {
        return Err(InsertionFailure::ConstructedImport);
    }
    if !index_valid {
        return Err(InsertionFailure::Index);
    }
    syntax
        .admit(
            &CssNamespaceContext::default(),
            CssRuleAdmissionContext::Stylesheet,
        )
        .into_parts()
        .0
        .ok_or(InsertionFailure::Semantic)?;
    Ok(())
}
fn group_stage(source: &str, index_valid: bool) -> Result<(), InsertionFailure> {
    if !index_valid {
        return Err(InsertionFailure::Index);
    }
    let syntax = classify_rule_syntax(source)
        .into_parts()
        .0
        .ok_or(InsertionFailure::Syntax)?;
    syntax
        .admit(
            &CssNamespaceContext::default(),
            CssRuleAdmissionContext::Group,
        )
        .into_parts()
        .0
        .ok_or(InsertionFailure::Semantic)?;
    Ok(())
}
#[test]
fn consumers_can_compose_sheet_and_group_precedence_without_css_owning_live_errors() {
    assert_eq!(
        sheet_stage(".a {} .b {}", false, false),
        Err(InsertionFailure::Syntax)
    );
    assert_eq!(
        group_stage(".a {} .b {}", false),
        Err(InsertionFailure::Index)
    );
    assert_eq!(
        sheet_stage("@unknown {}", false, false),
        Err(InsertionFailure::Index)
    );
    assert_eq!(
        sheet_stage("@unknown {}", false, true),
        Err(InsertionFailure::Semantic)
    );
    assert_eq!(
        sheet_stage("@import;", true, false),
        Err(InsertionFailure::ConstructedImport)
    );
    assert_eq!(
        sheet_stage("@import;", false, true),
        Err(InsertionFailure::Semantic)
    );
    assert_eq!(
        group_stage("@import 'a';", true),
        Err(InsertionFailure::Semantic)
    );
}

#[test]
fn admission_retains_recovered_children_and_the_original_snapshot_after_report_drop() {
    let source = "/*😀*/\r\n.a {width:1px; rubbish; opacity:.5}";
    let syntax = candidate(source);
    let report = syntax.admit(
        &CssNamespaceContext::default(),
        CssRuleAdmissionContext::Stylesheet,
    );
    assert!(!report.is_clean());
    let Some(CssAdmittedRule::Ordinary(CssRule::Style(style))) = report.syntax() else {
        panic!("retained style: {report:?}")
    };
    assert_eq!(names(style.declarations()), ["width", "opacity"]);
    assert_eq!(style.position().byte_offset().value(), 10);
    assert_eq!(style.position().line().value(), 1);
    assert_eq!(style.position().column().value(), 0);
    let value = style.declarations()[1].parsed_value().unwrap().clone();
    assert!(syntax.origin().source().same_snapshot(value.source()));
    assert_eq!(syntax.origin().span().start().byte_offset().value(), 10);
    drop(report);
    drop(syntax);
    assert_eq!(value.source().as_str(), source);
    assert_eq!(
        &source
            [value.span().start().byte_offset().value()..value.span().end().byte_offset().value()],
        ".5"
    );
}

#[test]
fn preliminary_eof_recovery_survives_semantic_admission_without_double_publication() {
    let source = ".a {width:1px";
    let report = classify_rule_syntax(source);
    assert!(!report.is_clean());
    let syntax = report.syntax().as_ref().unwrap();
    assert_eq!(syntax.diagnostics(), report.diagnostics());
    let semantic = syntax.admit(
        &CssNamespaceContext::default(),
        CssRuleAdmissionContext::Stylesheet,
    );
    assert!(semantic.syntax().is_some());
    assert_eq!(
        semantic
            .diagnostics()
            .iter()
            .filter(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
            .count(),
        1
    );
    assert!(
        semantic
            .diagnostics()
            .iter()
            .any(|d| report.diagnostics().contains(d))
    );
    assert!(semantic.clone().into_validation_result().is_err());
}

#[test]
fn detached_relative_and_explicit_anchor_selectors_require_selected_context() {
    for source in ["> .b {width:1px}", "& > .b {width:1px}", ".b {width:1px}"] {
        let report = admit(source, CssRuleAdmissionContext::Style);
        assert!(report.is_clean(), "{source}: {report:?}");
        let Some(CssAdmittedRule::Ordinary(CssRule::Style(style))) = report.syntax() else {
            panic!("nested style")
        };
        assert_eq!(style.position().byte_offset().value(), 0);
        assert_eq!(names(style.declarations()), ["width"]);
    }
    assert!(
        admit("> .b {}", CssRuleAdmissionContext::Stylesheet)
            .syntax()
            .is_none()
    );
    assert!(
        admit(".a, :unknown {}", CssRuleAdmissionContext::Style)
            .syntax()
            .is_none()
    );
    assert!(
        admit(":is(.a, :unknown) {}", CssRuleAdmissionContext::Style)
            .syntax()
            .is_some()
    );
}

#[test]
fn detached_namespace_binding_rejection_is_atomic_and_retry_preserves_source_identity() {
    let syntax = candidate("/*😀*/\r\n> svg|leaf {width:1px}");
    let rejected = syntax.admit(
        &CssNamespaceContext::default(),
        CssRuleAdmissionContext::Style,
    );
    assert!(rejected.syntax().is_none());
    let prefix = CssNamespacePrefix::try_new("svg").unwrap();
    let bindings =
        CssNamespaceContext::from_bindings([(Some(prefix), CssNamespaceName::new("urn:svg"))]);
    let accepted = syntax.admit(&bindings, CssRuleAdmissionContext::Style);
    assert!(accepted.is_clean(), "{accepted:?}");
    let Some(CssAdmittedRule::Ordinary(CssRule::Style(style))) = accepted.syntax() else {
        panic!("style")
    };
    assert_eq!(style.position().byte_offset().value(), 10);
    assert!(
        syntax
            .origin()
            .source()
            .same_snapshot(style.declarations()[0].parsed_name().unwrap().source())
    );
    assert!(
        syntax
            .admit(
                &CssNamespaceContext::default(),
                CssRuleAdmissionContext::Style
            )
            .syntax()
            .is_none()
    );
}

#[test]
fn nested_group_and_scoped_ancestry_use_their_own_grammar() {
    let source = "@media all {width:1px; > .b {opacity:.5} rubbish; color:red}";
    let style = admit(source, CssRuleAdmissionContext::Style);
    assert!(style.syntax().is_some());
    assert!(!style.is_clean());
    for context in [
        CssRuleAdmissionContext::Scope(CssStyleAncestor::Present),
        CssRuleAdmissionContext::ScopedGroup(CssStyleAncestor::Present),
    ] {
        let report = admit(source, context);
        let Some(CssAdmittedRule::Scoped(CssScopedRule::Media(media))) = report.syntax() else {
            panic!("scoped media: {report:?}")
        };
        assert!(matches!(
            &media.rules().rules()[0],
            CssScopedRule::NestedDeclarations(_)
        ));
        assert!(matches!(&media.rules().rules()[1], CssScopedRule::Style(_)));
    }
    let absent = admit(
        "@media all {width:1px; .b {opacity:.5}}",
        CssRuleAdmissionContext::ScopedGroup(CssStyleAncestor::Absent),
    );
    let Some(CssAdmittedRule::Scoped(CssScopedRule::Media(media))) = absent.syntax() else {
        panic!("scoped group")
    };
    assert!(
        media
            .rules()
            .rules()
            .iter()
            .all(|rule| !matches!(rule, CssScopedRule::NestedDeclarations(_)))
    );
    assert!(
        admit("@font-face {font-family:x}", CssRuleAdmissionContext::Style)
            .syntax()
            .is_none()
    );
}

#[test]
fn raw_contents_stops_at_root_closer_and_preserves_the_historical_list_contract() {
    let source = "} opacity:0.5; width:2px";
    let report = parse_declaration_block_contents(source);
    assert!(report.syntax().is_empty());
    assert!(report.is_clean());
    assert_eq!(
        names(parse_declaration_list_text(source).syntax()),
        ["width"]
    );
    let report = parse_declaration_block_contents("width:2px} opacity:.5");
    assert_eq!(names(report.syntax()), ["width"]);
    assert!(report.is_clean(), "{report:?}");
    assert!(parse_declaration_block_contents("}/**").is_clean());
}

#[test]
fn raw_contents_projects_ordered_declarations_around_nested_rules_and_at_rules() {
    let source = "width:1px; .a {opacity:0;} opacity:.5; @unknown {width:9px;} width:2px; --Case:a!important";
    let report = parse_declaration_block_contents(source);
    assert_eq!(
        names(report.syntax()),
        ["width", "opacity", "width", "--Case"]
    );
    assert_eq!(report.syntax()[3].importance(), CssImportance::Important);
    assert_eq!(report.diagnostics().len(), 2, "{report:?}");
    assert!(!report.is_clean());
    let owner = report.syntax()[0].parsed_name().unwrap().source();
    assert_eq!(owner.as_str(), source);
    for declaration in report.syntax().iter() {
        assert!(owner.same_snapshot(declaration.parsed_name().unwrap().source()));
        assert!(owner.same_snapshot(declaration.parsed_value().unwrap().source()));
    }
}

#[test]
fn raw_contents_empty_recovered_empty_and_custom_property_failure_have_separate_reports() {
    for source in ["", " /**/ \t", ";; ;", "}", "/*comment*/}"] {
        let report = parse_declaration_block_contents(source);
        assert!(report.syntax().is_empty());
        assert!(report.is_clean(), "{source}: {report:?}");
    }
    for source in [
        "rubbish;",
        "@unknown {};",
        ".a{}",
        "width:nonsense;",
        "--x:var() {opacity:0;} color:red;",
    ] {
        let report = parse_declaration_block_contents(source);
        assert!(report.syntax().is_empty(), "{source}: {report:?}");
        assert!(!report.is_clean());
    }
    let report = parse_declaration_block_contents("--x:var() {opacity:0;}; color:red;");
    assert_eq!(names(report.syntax()), ["color"]);
    let report = parse_declaration_block_contents("width:1px/*");
    assert_eq!(names(report.syntax()), ["width"]);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::IgnoreUnterminatedComment)
    );
}

#[test]
fn whole_input_resource_limits_are_cumulative_and_admission_reuses_the_charged_occurrence() {
    let source = "a{width:1px}";
    let limits = CssComponentValueLimits::try_new(1, 5, source.len()).unwrap();
    let report = classify_rule_syntax_with_limits(source, limits);
    let syntax = report.syntax().as_ref().unwrap();
    assert_eq!(syntax.component_count(), 5);
    assert_eq!(syntax.limits(), limits);
    assert!(
        syntax
            .admit(
                &CssNamespaceContext::default(),
                CssRuleAdmissionContext::Stylesheet
            )
            .is_clean()
    );
    let denied = classify_rule_syntax_with_limits(
        "a{width:1px;opacity:.5}",
        CssComponentValueLimits::try_new(1, 5, 100).unwrap(),
    );
    assert!(denied.syntax().is_none());
    resource(
        denied.diagnostics(),
        CssComponentValueErrorKind::ComponentLimit,
    );
    let denied = classify_rule_syntax_with_limits(
        source,
        CssComponentValueLimits::try_new(1, 100, source.len() - 1).unwrap(),
    );
    assert!(denied.syntax().is_none());
    resource(denied.diagnostics(), CssComponentValueErrorKind::ByteLimit);
    assert!(
        classify_rule_syntax_with_limits(source, limits)
            .syntax()
            .is_some()
    );
    let raw = "width:1px;opacity:.5";
    let denied = parse_declaration_block_contents_with_limits(
        raw,
        CssComponentValueLimits::try_new(0, 6, 100).unwrap(),
    );
    assert!(denied.syntax().is_empty());
    resource(
        denied.diagnostics(),
        CssComponentValueErrorKind::ComponentLimit,
    );
    let accepted = parse_declaration_block_contents_with_limits(
        raw,
        CssComponentValueLimits::try_new(0, 7, raw.len()).unwrap(),
    );
    assert_eq!(names(accepted.syntax()), ["width", "opacity"]);
    assert!(accepted.is_clean());
}

#[test]
fn depth_resource_rejection_and_bounded_nested_replay_have_distinct_outcomes() {
    let denied = classify_rule_syntax_with_limits(
        "a{width:calc(1px)}",
        CssComponentValueLimits::try_new(1, 100, 100).unwrap(),
    );
    assert!(denied.syntax().is_none());
    resource(
        denied.diagnostics(),
        CssComponentValueErrorKind::NestingLimit,
    );
    let source = format!(
        "{}& .leaf{{width:1px}}{}",
        "@media all{".repeat(130),
        "}".repeat(130)
    );
    let report = admit(&source, CssRuleAdmissionContext::Style);
    assert!(report.is_clean(), "{report:?}");
    let Some(CssAdmittedRule::Ordinary(mut rule)) = report.into_parts().0 else {
        panic!("ordinary group")
    };
    for _ in 0..130 {
        let CssRule::Media(media) = rule else {
            panic!("group")
        };
        assert_eq!(media.rules().len(), 1);
        rule = media.rules()[0].clone();
    }
    let CssRule::Style(style) = rule else {
        panic!("leaf")
    };
    assert_eq!(names(style.declarations()), ["width"]);
    assert_eq!(
        style.declarations()[0]
            .parsed_name()
            .unwrap()
            .source()
            .as_str(),
        source
    );
}

#[test]
fn document_mode_reaches_raw_and_detached_declarations_without_namespace_mutation() {
    let quirks = CssParserContext::new(CssParserMode::Quirks);
    let raw = quirks.parse_declaration_block_contents("width:2");
    assert_eq!(names(raw.syntax()), ["width"]);
    assert_eq!(raw.syntax()[0].parser_context(), quirks);
    let syntax = candidate("& .a {width:2}");
    let report = syntax.admit_with_context(
        &CssNamespaceContext::default(),
        CssRuleAdmissionContext::Style,
        quirks,
    );
    let Some(CssAdmittedRule::Ordinary(CssRule::Style(style))) = report.syntax() else {
        panic!("style: {report:?}")
    };
    assert_eq!(style.declarations()[0].parser_context(), quirks);
    assert!(
        !syntax
            .admit(
                &CssNamespaceContext::default(),
                CssRuleAdmissionContext::Style
            )
            .is_clean()
    );
}

#[test]
fn preliminary_bad_tokens_and_statement_eof_keep_original_recovery_ranges() {
    for source in ["@unknown 'bad\n;", "@unknown url(bad x);", "@unknown ) ;"] {
        let syntax = candidate(source);
        let diagnostic = syntax
            .diagnostics()
            .iter()
            .find(|d| d.action() == CssRecoveryAction::RetainSyntaxCandidate)
            .expect("generic fault");
        assert!(diagnostic.span().start().byte_offset().value() > 0);
        assert!(diagnostic.span().end().byte_offset().value() < source.len());
        let rejected = syntax.admit(
            &CssNamespaceContext::default(),
            CssRuleAdmissionContext::Stylesheet,
        );
        assert!(rejected.syntax().is_none());
        assert!(rejected.diagnostics().contains(diagnostic));
    }
    let syntax = candidate("@unknown");
    assert_eq!(syntax.at_rule_name(), Some("unknown"));
    assert!(
        syntax
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RetainSyntaxCandidate
                && d.error().position().byte_offset().value() == 8)
    );
}

#[test]
fn scoped_destination_keeps_scope_anchors_and_group_placement_permission_explicit() {
    let syntax = candidate("& > .b {width:1px}");
    for context in [
        CssRuleAdmissionContext::Scope(CssStyleAncestor::Absent),
        CssRuleAdmissionContext::ScopedGroup(CssStyleAncestor::Present),
    ] {
        let report = syntax.admit(&CssNamespaceContext::default(), context);
        assert!(report.is_clean(), "{report:?}");
        assert!(matches!(
            report.syntax(),
            Some(CssAdmittedRule::Scoped(CssScopedRule::Style(_)))
        ));
    }
    let page = candidate("@page {margin:1px}");
    let namespaces = CssNamespaceContext::default();
    assert!(
        page.admit(
            &namespaces,
            CssRuleAdmissionContext::Scope(CssStyleAncestor::Absent)
        )
        .syntax()
        .is_none()
    );
    assert!(
        page.admit(
            &namespaces,
            CssRuleAdmissionContext::ScopedGroup(CssStyleAncestor::Present)
        )
        .syntax()
        .is_none()
    );
    assert!(
        page.admit(
            &namespaces,
            CssRuleAdmissionContext::ScopedGroup(CssStyleAncestor::Absent)
        )
        .syntax()
        .is_some()
    );
}

#[test]
fn malformed_custom_property_name_without_colon_uses_the_nested_rule_boundary() {
    let report = parse_declaration_block_contents("--x .a{}width:2px; opacity:.5");
    assert_eq!(names(report.syntax()), ["width", "opacity"]);
    assert!(!report.is_clean());
}

#[test]
fn recovered_string_and_url_token_endings_keep_preliminary_and_declaration_eof_diagnostics() {
    for source in ["@unknown 'unterminated", "@unknown url(unterminated"] {
        let syntax = candidate(source);
        assert!(
            syntax
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
        let report = syntax.admit(
            &CssNamespaceContext::default(),
            CssRuleAdmissionContext::Stylesheet,
        );
        assert!(report.syntax().is_none());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
    }
    for (source, name) in [
        ("--x:'unterminated", "--x"),
        ("background-image:url(unterminated", "background-image"),
    ] {
        let report = parse_declaration_block_contents(source);
        assert_eq!(names(report.syntax()), [name]);
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure),
            "{report:?}"
        );
    }
}

#[test]
fn downstream_nested_fallback_distinguishes_nonempty_declarations_from_empty_recovery() {
    fn has_nested_payload(source: &str) -> bool {
        if classify_rule_syntax(source)
            .into_parts()
            .0
            .is_some_and(|rule| {
                rule.admit(
                    &CssNamespaceContext::default(),
                    CssRuleAdmissionContext::Style,
                )
                .syntax()
                .is_some()
            })
        {
            return true;
        }
        !parse_declaration_block_contents(source).syntax().is_empty()
    }
    assert!(has_nested_payload("> .a {width:1px}"));
    assert!(has_nested_payload("width:1px;opacity:.5"));
    assert!(!has_nested_payload("@unknown {}"));
    assert!(!has_nested_payload("rubbish;"));
    assert!(!has_nested_payload("} width:1px"));
}

#[test]
fn declaration_projection_retains_the_neighbor_after_an_unterminated_run_of_rule_slots() {
    let source = format!("{}width:2px", ".a{}div{}".repeat(512));
    let report = parse_declaration_block_contents(&source);
    assert_eq!(names(report.syntax()), ["width"]);
    assert_eq!(report.diagnostics().len(), 1024);
    assert_eq!(
        report.syntax()[0].parsed_name().unwrap().source().as_str(),
        source
    );
}

#[test]
fn selected_block_declaration_rejects_mixed_top_level_curly_values_before_substitution_admission() {
    // Selected Syntax §5.5.6 step 8 applies before ordinary property grammar.
    // var() must not bypass the shape constraint; importance has already been
    // removed when deciding whether the curly block is the sole component.
    for source in [
        "width:var(--x){};opacity:.5",
        "width:var(--x) /**/ {} !important;opacity:.5",
        "width:{}var(--x);opacity:.5",
        "width:{var(--x)}{};opacity:.5",
    ] {
        let report = parse_declaration_block_contents(source);
        assert_eq!(names(report.syntax()), ["opacity"], "{source}: {report:?}");
        assert!(!report.is_clean(), "{source}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropQualifiedRule
        );
    }
}

#[test]
fn selected_block_declaration_mixed_curly_fallback_stops_at_the_first_rule_body() {
    let source = "width:var(--x){}opacity:.5";
    let report = parse_declaration_block_contents(source);
    assert_eq!(names(report.syntax()), ["opacity"]);
    assert_eq!(report.diagnostics().len(), 1);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        "width:var(--x){}".len()
    );
    let declaration = &report.syntax()[0];
    let origin = declaration.parsed_name().unwrap();
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        &origin.source().as_str()[origin.span().start().byte_offset().value()
            ..origin.span().end().byte_offset().value()],
        "opacity"
    );
    assert_eq!(
        names(parse_declaration_block_contents("width:var(--x){}opacity:.5}width:1px").syntax()),
        ["opacity"]
    );
}

#[test]
fn selected_block_declaration_preserves_custom_and_single_curly_or_nested_curly_controls() {
    for source in [
        "--x:var(--x){};opacity:.5",
        "--x:{}var(--x);opacity:.5",
        "--x:{}{};opacity:.5",
        "\\2d \\2d x:var(--x){};opacity:.5",
    ] {
        let report = parse_declaration_block_contents(source);
        assert_eq!(names(report.syntax()), ["--x", "opacity"], "{source}");
        assert!(report.is_clean(), "{source}: {report:?}");
    }
    for source in [
        "width:{var(--x)};opacity:.5",
        "width: /**/ {var(--x)} /*trivia*/ !important;opacity:.5",
        "width:var(--x, {});opacity:.5",
    ] {
        let report = parse_declaration_block_contents(source);
        assert_eq!(names(report.syntax()), ["width", "opacity"], "{source}");
        assert!(report.is_clean(), "{source}: {report:?}");
    }
    let report = parse_declaration_block_contents("--x:var(--x){} !important;opacity:.5");
    assert_eq!(names(report.syntax()), ["--x", "opacity"]);
    assert_eq!(report.syntax()[0].importance(), CssImportance::Important);
    assert!(report.is_clean());
}

#[test]
fn selected_block_declaration_constraint_leaves_the_historical_raw_list_contract_intact() {
    let report = parse_declaration_list_text("width:var(--x){};opacity:.5");
    assert_eq!(names(report.syntax()), ["width", "opacity"]);
    assert!(report.is_clean());
    let report = parse_declaration_list_text("width:var(--x){}opacity:.5");
    assert_eq!(names(report.syntax()), ["width"]);
    assert!(report.is_clean());
}
