#![forbid(unsafe_code)]
//! Functional new body-front expectations, independently derived from the adopted
//! genuine-source contract, selected Syntax/Nesting/Cascade/Conditional sources,
//! and existing authored/context/partition contracts.
use surgeist_css::*;

fn envelope<'a, T>(
    report: &'a CssParseReport<Option<CssBlockFragment<T>>>,
    source: &str,
    start: usize,
    end: usize,
) -> &'a CssBlockFragment<T> {
    let fragment = report.syntax().as_ref().expect("retained genuine block");
    assert_eq!(fragment.origin().source().as_str(), source);
    assert_eq!(
        fragment.origin().span().start().byte_offset().value(),
        start
    );
    assert_eq!(fragment.origin().span().end().byte_offset().value(), end);
    fragment
}

fn clean<T: Clone>(report: &CssParseReport<Option<CssBlockFragment<T>>>) {
    assert!(report.is_clean());
    assert!(report.clone().into_validation_result().is_ok());
}

fn recovered<T: Clone + std::fmt::Debug>(report: &CssParseReport<Option<CssBlockFragment<T>>>) {
    assert!(!report.is_clean());
    assert_eq!(
        report
            .clone()
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

fn rejected<T: Clone + std::fmt::Debug>(report: &CssParseReport<Option<CssBlockFragment<T>>>) {
    assert!(report.syntax().is_none());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RejectInput)
    );
    recovered(report);
}

fn original(
    origin: &CssValueOrigin,
    source: &str,
    start: usize,
    end: usize,
    envelope: &CssParsedOrigin,
) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("original parsed occurrence")
    };
    assert_eq!(origin.source().as_str(), source);
    assert!(origin.source().same_snapshot(envelope.source()));
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), end);
}

fn property(list: &CssDeclarationList, name: CssKnownProperty, value: &str) {
    let [declaration] = list.as_slice() else {
        panic!("one retained declaration")
    };
    assert_eq!(declaration.known().unwrap().property(), name);
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        value
    );
}

fn ordinary_style<'a>(rule: &'a CssRule, class: &str) -> &'a CssStyleRule {
    let CssRule::Style(style) = rule else {
        panic!("ordinary style")
    };
    assert_eq!(
        style.selectors().selectors()[0].selector(),
        &CssSelector::Class(class.to_owned())
    );
    style
}

fn scoped_style<'a>(rule: &'a CssScopedRule, class: &str) -> &'a CssScopedStyleRule {
    let CssScopedRule::Style(style) = rule else {
        panic!("scoped style")
    };
    assert_eq!(
        style.selectors().selectors(),
        &[CssScopedStyleSelector::Selector(CssSelector::Class(
            class.to_owned()
        ))]
    );
    style
}

fn scoped_run(rule: &CssScopedRule) -> &CssDeclarationList {
    let CssScopedRule::NestedDeclarations(run) = rule else {
        panic!("direct scoped declaration run")
    };
    run.declarations()
}

fn namespace_context() -> CssNamespaceContext {
    CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("p").unwrap()),
        CssNamespaceName::new("urn:original:p"),
    )])
}

fn namespaced_selector(selector: &CssSelector) {
    let CssSelector::Compound(compound) = selector else {
        panic!("namespace-qualified selector")
    };
    let name = compound.type_selector().unwrap();
    assert_eq!(
        name.namespace(),
        &CssNamespaceConstraint::Named(CssNamespacePrefix::try_new("p").unwrap())
    );
    assert_eq!(name.local_name(), Some("a"));
}

fn limit(diagnostics: &[CssRecoveryDiagnostic], responsible: usize) {
    let diagnostic = diagnostics
        .iter()
        .find(|d| d.action() == CssRecoveryAction::StopAtNestingLimit)
        .expect("typed resource failure, independent of ordinary grammar recovery");
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("structural depth")
    };
    assert_eq!(detail.limit(), 256);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible
    );
}

#[test]
fn real_empty_blocks_are_clean_in_every_selected_body_role() {
    let ns = CssNamespaceContext::default();
    let group = parse_group_block("{}", &ns);
    assert!(envelope(&group, "{}", 0, 2).body().rules().is_empty());
    clean(&group);
    for ancestry in [CssStyleAncestor::Absent, CssStyleAncestor::Present] {
        let scope = parse_scope_block("{}", &ns, ancestry);
        assert!(envelope(&scope, "{}", 0, 2).body().rules().is_empty());
        clean(&scope);
        let scoped_group = parse_scoped_group_block("{}", &ns, ancestry);
        assert!(
            envelope(&scoped_group, "{}", 0, 2)
                .body()
                .rules()
                .is_empty()
        );
        clean(&scoped_group);
    }
    let supports = parse_supports_test_block("{}");
    assert!(envelope(&supports, "{}", 0, 2).body().items().is_empty());
    clean(&supports);
}

#[test]
fn opener_and_exhaustion_are_whole_fragment_requirements() {
    let ns = CssNamespaceContext::default();
    for source in [
        "",
        " \t/*only*/",
        "[]",
        "color:red",
        "{} {}",
        "{}x",
        "{};",
        "{})",
    ] {
        rejected(&parse_group_block(source, &ns));
        rejected(&parse_scope_block(source, &ns, CssStyleAncestor::Absent));
        rejected(&parse_scoped_group_block(
            source,
            &ns,
            CssStyleAncestor::Present,
        ));
        rejected(&parse_supports_test_block(source));
    }
}

#[test]
fn implicit_real_outer_openers_retain_empty_bodies_with_original_eof_diagnostics() {
    let ns = CssNamespaceContext::default();
    let group = parse_group_block("{", &ns);
    assert!(envelope(&group, "{", 0, 1).body().rules().is_empty());
    recovered(&group);
    let scope = parse_scope_block("{", &ns, CssStyleAncestor::Absent);
    assert!(envelope(&scope, "{", 0, 1).body().rules().is_empty());
    recovered(&scope);
    let scoped = parse_scoped_group_block("{", &ns, CssStyleAncestor::Present);
    assert!(envelope(&scoped, "{", 0, 1).body().rules().is_empty());
    recovered(&scoped);
    let supports = parse_supports_test_block("{");
    assert!(envelope(&supports, "{", 0, 1).body().items().is_empty());
    recovered(&supports);
    for diagnostics in [
        group.diagnostics(),
        scope.diagnostics(),
        scoped.diagnostics(),
        supports.diagnostics(),
    ] {
        let fault = diagnostics
            .iter()
            .find(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
            .unwrap();
        assert_eq!(fault.error().code(), CssErrorCode::UnexpectedEnd);
        assert_eq!(fault.error().position().byte_offset().value(), 1);
        assert_eq!(fault.span().start().byte_offset().value(), 1);
        assert_eq!(fault.span().end().byte_offset().value(), 1);
    }
}

#[test]
fn ordinary_groups_retain_defined_children_but_do_not_admit_bare_declarations() {
    let ns = CssNamespaceContext::default();
    let source = "{.a{color:red}@media all{.b{color:green}}}";
    let report = parse_group_block(source, &ns);
    let body = envelope(&report, source, 0, 42).body();
    let [first, CssRule::Media(media)] = body.rules() else {
        panic!("style then media")
    };
    property(
        ordinary_style(first, "a").declarations(),
        CssKnownProperty::Color,
        "red",
    );
    let [child] = media.rules() else {
        panic!("one media child")
    };
    property(
        ordinary_style(child, "b").declarations(),
        CssKnownProperty::Color,
        "green",
    );
    clean(&report);

    let source = "{color:red;}";
    let report = parse_group_block(source, &ns);
    let body = envelope(&report, source, 0, 12).body();
    assert!(body.rules().is_empty());
    recovered(&report);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropQualifiedRule)
    );

    // Existing styled capability is a secondary permission control.
    let source = "{color:red;.b{color:green}color:blue}";
    let styled = parse_style_block(source, &ns);
    assert!(styled.is_clean());
    let styled = styled.syntax().as_ref().unwrap();
    property(styled.declarations(), CssKnownProperty::Color, "red");
    let [child, CssRule::NestedDeclarations(after)] = styled.rules() else {
        panic!("style and later run")
    };
    property(
        ordinary_style(child, "b").declarations(),
        CssKnownProperty::Color,
        "green",
    );
    property(after.declarations(), CssKnownProperty::Color, "blue");
}

#[test]
fn explicit_style_ancestry_controls_direct_scoped_declaration_permission() {
    let ns = CssNamespaceContext::default();
    let source = "{color:red;}";
    for report in [
        parse_scope_block(source, &ns, CssStyleAncestor::Present),
        parse_scoped_group_block(source, &ns, CssStyleAncestor::Present),
    ] {
        let [run] = envelope(&report, source, 0, 12).body().rules() else {
            panic!("one direct declaration run")
        };
        property(scoped_run(run), CssKnownProperty::Color, "red");
        let declaration = &scoped_run(run)[0];
        assert_eq!(declaration.position().unwrap().byte_offset().value(), 1);
        assert_eq!(
            declaration.parsed_value().unwrap().source().as_str(),
            source
        );
        clean(&report);
    }
    for report in [
        parse_scope_block(source, &ns, CssStyleAncestor::Absent),
        parse_scoped_group_block(source, &ns, CssStyleAncestor::Absent),
    ] {
        assert!(envelope(&report, source, 0, 12).body().rules().is_empty());
        recovered(&report);
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::DropQualifiedRule)
        );
    }
}

#[test]
fn scope_role_and_scoped_group_role_keep_the_defined_page_permission_distinct() {
    let ns = CssNamespaceContext::default();
    let source = "{@page{margin-top:1px}.a{color:red}}";
    let scope = parse_scope_block(source, &ns, CssStyleAncestor::Absent);
    let [child] = envelope(&scope, source, 0, source.len()).body().rules() else {
        panic!("scope rejects direct page, keeps style")
    };
    property(
        scoped_style(child, "a").declarations(),
        CssKnownProperty::Color,
        "red",
    );
    recovered(&scope);
    assert!(
        scope
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropAtRule)
    );

    let group = parse_scoped_group_block(source, &ns, CssStyleAncestor::Absent);
    let [CssScopedRule::Page(page), child] =
        envelope(&group, source, 0, source.len()).body().rules()
    else {
        panic!("group keeps page then style")
    };
    property(page.declarations(), CssKnownProperty::MarginTop, "1px");
    assert_eq!(page.position().byte_offset().value(), 1);
    property(
        scoped_style(child, "a").declarations(),
        CssKnownProperty::Color,
        "red",
    );
    clean(&group);
    for report in [
        parse_scope_block(source, &ns, CssStyleAncestor::Present),
        parse_scoped_group_block(source, &ns, CssStyleAncestor::Present),
    ] {
        let [child] = envelope(&report, source, 0, source.len()).body().rules() else {
            panic!("style ancestry excludes page locally")
        };
        property(
            scoped_style(child, "a").declarations(),
            CssKnownProperty::Color,
            "red",
        );
        recovered(&report);
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::DropAtRule)
        );
    }
}

#[test]
fn supplied_namespaces_apply_without_authored_namespace_insertion_and_origins_are_shared() {
    let ns = namespace_context();
    let before = ns.clone();
    let source = " /*é*/\r\n{p|a{color:red}} /*💡*/ ";
    let report = parse_group_block(source, &ns);
    let fragment = envelope(&report, source, 9, 25);
    let [CssRule::Style(style)] = fragment.body().rules() else {
        panic!("namespace-qualified style")
    };
    namespaced_selector(style.selectors().selectors()[0].selector());
    let value = style.declarations()[0].parsed_value().unwrap();
    assert_eq!(style.position().byte_offset().value(), 10);
    assert_eq!(
        (
            style.position().line().value(),
            style.position().column().value()
        ),
        (1, 1)
    );
    assert_eq!(value.span().start().byte_offset().value(), 20);
    assert_eq!(value.span().end().byte_offset().value(), 23);
    assert!(value.source().same_snapshot(fragment.origin().source()));
    assert_eq!(value.source().as_str(), source);
    clean(&report);
    for report in [
        parse_scope_block(source, &ns, CssStyleAncestor::Absent),
        parse_scoped_group_block(source, &ns, CssStyleAncestor::Absent),
    ] {
        let fragment = envelope(&report, source, 9, 25);
        let [CssScopedRule::Style(style)] = fragment.body().rules() else {
            panic!("supplied scoped namespace")
        };
        let [CssScopedStyleSelector::Selector(selector)] = style.selectors().selectors() else {
            panic!("absolute namespace-qualified selector")
        };
        namespaced_selector(selector);
        assert!(
            style.declarations()[0]
                .parsed_value()
                .unwrap()
                .source()
                .same_snapshot(fragment.origin().source())
        );
        clean(&report);
    }
    assert_eq!(ns, before);
    let absent = parse_group_block(source, &CssNamespaceContext::default());
    assert!(envelope(&absent, source, 9, 25).body().rules().is_empty());
    recovered(&absent);
    let placement = parse_group_block("{@namespace p 'urn:new';.a{}}", &ns);
    let [child] = placement.syntax().as_ref().unwrap().body().rules() else {
        panic!("namespace statement is excluded from inner lists")
    };
    ordinary_style(child, "a");
    recovered(&placement);
    assert_eq!(ns, before);
}

#[test]
fn all_three_context_methods_carry_quirks_into_actual_child_property_providers() {
    let ns = namespace_context();
    let source = "{p|a{width:7;color:red}}";
    let standards = CssParserContext::new(CssParserMode::Standards);
    let quirks = CssParserContext::new(CssParserMode::Quirks);
    let std = standards.parse_group_block(source, &ns);
    let [CssRule::Style(style)] = std.syntax().as_ref().unwrap().body().rules() else {
        panic!("retained child")
    };
    property(style.declarations(), CssKnownProperty::Color, "red");
    recovered(&std);
    assert_eq!(parse_group_block(source, &ns), std);
    let q = quirks.parse_group_block(source, &ns);
    let [CssRule::Style(style)] = q.syntax().as_ref().unwrap().body().rules() else {
        panic!("quirks child")
    };
    namespaced_selector(style.selectors().selectors()[0].selector());
    assert_eq!(style.declarations().len(), 2);
    assert_eq!(
        style.declarations()[0].to_specified_css().unwrap(),
        "width: 7px;"
    );
    clean(&q);

    for (std, q) in [
        (
            standards.parse_scope_block(source, &ns, CssStyleAncestor::Absent),
            quirks.parse_scope_block(source, &ns, CssStyleAncestor::Absent),
        ),
        (
            standards.parse_scoped_group_block(source, &ns, CssStyleAncestor::Absent),
            quirks.parse_scoped_group_block(source, &ns, CssStyleAncestor::Absent),
        ),
    ] {
        let [CssScopedRule::Style(style)] = std.syntax().as_ref().unwrap().body().rules() else {
            panic!("standards scoped child")
        };
        property(style.declarations(), CssKnownProperty::Color, "red");
        recovered(&std);
        let [CssScopedRule::Style(style)] = q.syntax().as_ref().unwrap().body().rules() else {
            panic!("quirks scoped child")
        };
        assert_eq!(style.declarations().len(), 2);
        assert_eq!(
            style.declarations()[0].to_specified_css().unwrap(),
            "width: 7px;"
        );
        clean(&q);
    }
    for report in [
        standards.parse_scope_block("{width:7;color:red;}", &ns, CssStyleAncestor::Present),
        standards.parse_scoped_group_block("{width:7;color:red;}", &ns, CssStyleAncestor::Present),
    ] {
        let [run] = report.syntax().as_ref().unwrap().body().rules() else {
            panic!("standards direct run")
        };
        property(scoped_run(run), CssKnownProperty::Color, "red");
        recovered(&report);
    }
    for report in [
        quirks.parse_scope_block("{width:7;}", &ns, CssStyleAncestor::Present),
        quirks.parse_scoped_group_block("{width:7;}", &ns, CssStyleAncestor::Present),
    ] {
        let [run] = report.syntax().as_ref().unwrap().body().rules() else {
            panic!("quirks direct run")
        };
        assert_eq!(
            scoped_run(run)[0].to_specified_css().unwrap(),
            "width: 7px;"
        );
        clean(&report);
    }
}

#[test]
fn source_adjacency_is_admitted_after_deep_body_siblings_are_reconstructed() {
    let ns = CssNamespaceContext::default();
    // The actual outer brace plus 128 entered child groups requires bounded replay.
    let deep = format!("{}{}", "@media all{".repeat(128), "}".repeat(128));
    let source = format!("{{{deep}@when media(width){{}}/*real gap*/@else{{}}}}");
    let group = parse_group_block(&source, &ns);
    let [CssRule::Media(_), CssRule::When(when), CssRule::Else(other)] =
        group.syntax().as_ref().unwrap().body().rules()
    else {
        panic!("original adjacent chain after reconstructed sibling")
    };
    assert_eq!(when.position().unwrap().byte_offset().value(), 1 + 128 * 12);
    assert!(other.condition().is_none());
    clean(&group);
    for report in [
        parse_scope_block(&source, &ns, CssStyleAncestor::Absent),
        parse_scoped_group_block(&source, &ns, CssStyleAncestor::Present),
    ] {
        assert!(matches!(
            report.syntax().as_ref().unwrap().body().rules(),
            [
                CssScopedRule::Media(_),
                CssScopedRule::When(_),
                CssScopedRule::Else(_)
            ]
        ));
        clean(&report);
    }

    let broken =
        format!("{{{deep}@when media(width){{}}@unknown{{}}@else{{}}.after{{color:red}}}}");
    let group = parse_group_block(&broken, &ns);
    let [CssRule::Media(_), CssRule::When(_), child] =
        group.syntax().as_ref().unwrap().body().rules()
    else {
        panic!("failed source unit breaks adjacency even after it is dropped")
    };
    property(
        ordinary_style(child, "after").declarations(),
        CssKnownProperty::Color,
        "red",
    );
    recovered(&group);
    for report in [
        parse_scope_block(&broken, &ns, CssStyleAncestor::Absent),
        parse_scoped_group_block(&broken, &ns, CssStyleAncestor::Present),
    ] {
        let [CssScopedRule::Media(_), CssScopedRule::When(_), child] =
            report.syntax().as_ref().unwrap().body().rules()
        else {
            panic!("same original-gap scoped admission")
        };
        property(
            scoped_style(child, "after").declarations(),
            CssKnownProperty::Color,
            "red",
        );
        recovered(&report);
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.error().code() == CssErrorCode::InvalidAtRulePlacement
                    && d.action() == CssRecoveryAction::DropAtRule)
        );
    }
    for report in [
        parse_scope_block("{@else{}}", &ns, CssStyleAncestor::Absent),
        parse_scoped_group_block("{@else{}}", &ns, CssStyleAncestor::Present),
    ] {
        assert!(report.syntax().as_ref().unwrap().body().rules().is_empty());
        recovered(&report);
    }
    let free = parse_group_block("{@else{}}", &ns);
    assert!(free.syntax().as_ref().unwrap().body().rules().is_empty());
    recovered(&free);
}

#[test]
fn retained_implicit_children_have_original_eof_and_fail_clean_validation() {
    let ns = CssNamespaceContext::default();
    let source = "{.a{color:red";
    let report = parse_group_block(source, &ns);
    let fragment = envelope(&report, source, 0, 13);
    let [child] = fragment.body().rules() else {
        panic!("retained child style")
    };
    let child = ordinary_style(child, "a");
    property(child.declarations(), CssKnownProperty::Color, "red");
    assert_eq!(child.position().byte_offset().value(), 1);
    let parsed = child.declarations()[0].parsed_value().unwrap();
    assert!(parsed.source().same_snapshot(fragment.origin().source()));
    assert_eq!(parsed.span().start().byte_offset().value(), 10);
    assert_eq!(parsed.span().end().byte_offset().value(), 13);
    assert!(report.diagnostics().iter().any(|d| d.action()
        == CssRecoveryAction::RetainWithImplicitClosure
        && d.error().position().byte_offset().value() == 13));
    recovered(&report);
    for report in [
        parse_scope_block(source, &ns, CssStyleAncestor::Absent),
        parse_scoped_group_block(source, &ns, CssStyleAncestor::Present),
    ] {
        let fragment = envelope(&report, source, 0, 13);
        let [child] = fragment.body().rules() else {
            panic!("retained implicitly closed scoped style")
        };
        property(
            scoped_style(child, "a").declarations(),
            CssKnownProperty::Color,
            "red",
        );
        recovered(&report);
    }
}

#[test]
fn native_group_unit_depth_counts_the_outer_brace_and_retains_later_source_siblings() {
    let ns = CssNamespaceContext::default();
    for nested in [255_usize, 256] {
        let source = format!(
            "{{{}{}.after{{color:red}}}}",
            "@media all{".repeat(nested),
            "}".repeat(nested)
        );
        let report = parse_group_block(&source, &ns);
        let fragment = envelope(&report, &source, 0, source.len());
        let [CssRule::Media(first), after] = fragment.body().rules() else {
            panic!("native outer group and later sibling")
        };
        let after = ordinary_style(after, "after");
        property(after.declarations(), CssKnownProperty::Color, "red");
        assert_eq!(after.position().byte_offset().value(), 1 + nested * 12);
        if nested == 255 {
            let mut children = first.rules();
            for _ in 1..255 {
                let [CssRule::Media(child)] = children else {
                    panic!("retained authored group chain")
                };
                children = child.rules();
            }
            assert!(children.is_empty());
            clean(&report);
        } else {
            limit(report.diagnostics(), 1 + 255 * 11 + 10);
            recovered(&report);
        }
        for report in [
            parse_scope_block(&source, &ns, CssStyleAncestor::Absent),
            parse_scoped_group_block(&source, &ns, CssStyleAncestor::Present),
        ] {
            let [CssScopedRule::Media(_), after] = report.syntax().as_ref().unwrap().body().rules()
            else {
                panic!("scoped native unit recovery and later sibling")
            };
            let after = scoped_style(after, "after");
            property(after.declarations(), CssKnownProperty::Color, "red");
            assert_eq!(after.position().byte_offset().value(), 1 + nested * 12);
            if nested == 255 {
                clean(&report);
            } else {
                limit(report.diagnostics(), 1 + 255 * 11 + 10);
                recovered(&report);
            }
        }
    }
}

#[test]
fn generic_supports_candidates_keep_order_payloads_and_the_complete_original_snapshot() {
    let source = " /*é*/\r\n{future:yes;future x{child:yes}@future{tail:yes;}} /*💡*/ ";
    let report = parse_supports_test_block(source);
    let fragment = envelope(&report, source, 9, 59);
    let [
        CssSupportsTestItem::Declarations(run),
        CssSupportsTestItem::QualifiedRule(qualified),
        CssSupportsTestItem::AtRule(at),
    ] = fragment.body().items()
    else {
        panic!("mixed declaration, qualified, at-rule tests")
    };
    let [declaration] = run.declarations() else {
        panic!("one generic test declaration")
    };
    assert_eq!(declaration.property(), "future");
    original(declaration.origin(), source, 10, 16, fragment.origin());
    let [value] = declaration.value_components() else {
        panic!("one original ident value")
    };
    assert!(matches!(
        value.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("yes"))
    ));
    original(value.origin(), source, 17, 20, fragment.origin());
    assert_eq!(
        qualified.prelude().serialize().unwrap().as_css(),
        "future x"
    );
    assert!(
        matches!(qualified.body().items(), [CssSupportsTestItem::Declarations(run)] if run.declarations()[0].property() == "child")
    );
    assert_eq!(at.name(), "future");
    original(at.origin(), source, 40, 47, fragment.origin());
    assert!(at.prelude().items().is_empty());
    assert!(
        matches!(at.body().unwrap().items(), [CssSupportsTestItem::Declarations(run)] if run.declarations()[0].property() == "tail")
    );
    assert!(fragment.body().recovery_origin().is_none());
    clean(&report);
}

#[test]
fn named_test_at_rule_at_real_parent_close_does_not_promote_baseline_eof_fault() {
    for source in ["{@future}", "{@future;}"] {
        let report = parse_supports_test_block(source);
        let fragment = envelope(&report, source, 0, source.len());
        let [CssSupportsTestItem::AtRule(at)] = fragment.body().items() else {
            panic!("one retained generic statement test")
        };
        assert_eq!(at.name(), "future");
        assert!(at.body().is_none());
        assert!(at.prelude().items().is_empty());
        original(at.origin(), source, 1, 8, fragment.origin());
        assert!(fragment.body().recovery_origin().is_none());
        clean(&report);
        // Output punctuation has its separate checked owner; original input is unchanged.
        assert_eq!(fragment.body().serialize().unwrap().as_css(), "@future;");
    }
    let source = "{@future";
    let report = parse_supports_test_block(source);
    let fragment = envelope(&report, source, 0, 8);
    assert!(matches!(
        fragment.body().items(),
        [CssSupportsTestItem::AtRule(_)]
    ));
    let CssValueOrigin::ImplicitClosure { opening, at } =
        fragment.body().recovery_origin().unwrap()
    else {
        panic!("only actual outer missing brace is a recovery origin")
    };
    assert_eq!(opening.span().start().byte_offset().value(), 0);
    assert_eq!(at.span().start().byte_offset().value(), 8);
    assert!(opening.source().same_snapshot(fragment.origin().source()));
    assert!(at.source().same_snapshot(fragment.origin().source()));
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|d| d.action() != CssRecoveryAction::RetainNonconformingRule)
    );
    recovered(&report);
}

#[test]
fn raw_bad_supports_declaration_recovers_locally_without_checked_carrier_relex() {
    let source = "{first:yes;broken:url(a b);last:yes;}";
    let report = parse_supports_test_block(source);
    let fragment = envelope(&report, source, 0, source.len());
    let [CssSupportsTestItem::Declarations(run)] = fragment.body().items() else {
        panic!("one declaration run around failed declaration")
    };
    assert_eq!(
        run.declarations()
            .iter()
            .map(CssSupportsTestDeclaration::property)
            .collect::<Vec<_>>(),
        ["first", "last"]
    );
    original(
        run.declarations()[0].origin(),
        source,
        1,
        6,
        fragment.origin(),
    );
    original(
        run.declarations()[1].origin(),
        source,
        27,
        31,
        fragment.origin(),
    );
    assert!(fragment.body().recovery_origin().is_some());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.error().code() == CssErrorCode::InvalidComponentValue
                && d.action() == CssRecoveryAction::DropDeclaration)
    );
    recovered(&report);
}

#[test]
fn rejected_generic_rule_candidate_partitions_supports_runs_without_leaking_child_tests() {
    let source = "{before:yes;@future url(a b){child:yes}after:yes;}";
    let report = parse_supports_test_block(source);
    let [
        CssSupportsTestItem::Declarations(before),
        CssSupportsTestItem::Declarations(after),
    ] = report.syntax().as_ref().unwrap().body().items()
    else {
        panic!("rule boundary still splits runs when rejected")
    };
    assert_eq!(before.declarations()[0].property(), "before");
    assert_eq!(after.declarations()[0].property(), "after");
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropAtRule)
    );
    recovered(&report);
}

#[test]
fn implicit_supports_function_and_body_retain_actual_origins_but_reject_clean_reuse() {
    let source = "{child:fn(a";
    let report = parse_supports_test_block(source);
    let fragment = envelope(&report, source, 0, 11);
    let [CssSupportsTestItem::Declarations(run)] = fragment.body().items() else {
        panic!("retained generic function test")
    };
    let [declaration] = run.declarations() else {
        panic!("retained child")
    };
    assert_eq!(declaration.property(), "child");
    let [value] = declaration.value_components() else {
        panic!("one function")
    };
    let CssComponentValueRef::Function(function) = value.view() else {
        panic!("retained function")
    };
    let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
        panic!("actual function EOF")
    };
    assert_eq!(opening.span().start().byte_offset().value(), 7);
    assert_eq!(at.span().start().byte_offset().value(), 11);
    assert!(at.source().same_snapshot(fragment.origin().source()));
    recovered(&report);
    assert!(matches!(
        CssSupportsConditionRule::try_new(
            CssSupportsConditionName::try_new("--x").unwrap(),
            fragment.body().clone()
        ),
        Err(CssNamedSupportsConstructionError::RecoveredInput { .. })
    ));
}

#[test]
fn named_test_body_depth_counts_its_actual_outer_brace_before_checked_promotion() {
    for nested in [255_usize, 256] {
        let source = format!("{{future:{}x{};}}", "f(".repeat(nested), ")".repeat(nested));
        let report = parse_supports_test_block(&source);
        if nested == 255 {
            let fragment = envelope(&report, &source, 0, source.len());
            let [CssSupportsTestItem::Declarations(run)] = fragment.body().items() else {
                panic!("deep source test declaration")
            };
            assert_eq!(run.declarations()[0].property(), "future");
            clean(&report);
        } else {
            assert!(report.syntax().is_none());
            limit(report.diagnostics(), 8 + 255 * 2);
            recovered(&report);
        }
    }
}

#[test]
fn nested_scope_root_and_limit_are_symbolic_while_style_ancestry_passes_through() {
    let ns = CssNamespaceContext::default();
    let source = "{@scope (.s) to (:scope .limit){color:red;}}";
    for ancestry in [CssStyleAncestor::Absent, CssStyleAncestor::Present] {
        for report in [
            parse_scope_block(source, &ns, ancestry),
            parse_scoped_group_block(source, &ns, ancestry),
        ] {
            let fragment = envelope(&report, source, 0, source.len());
            let [CssScopedRule::Scope(scope)] = fragment.body().rules() else {
                panic!("actual nested scope remains")
            };
            assert_eq!(scope.position().unwrap().byte_offset().value(), 1);
            assert_eq!(
                scope.root().unwrap().selectors(),
                &[CssScopeSelector::Selector(CssSelector::Class(
                    "s".to_owned()
                ))]
            );
            let [CssScopeSelector::Selector(CssSelector::Complex(limit))] =
                scope.limit().unwrap().selectors()
            else {
                panic!("symbolic scope-relative descendant limit")
            };
            assert_eq!(limit.first().pseudo_classes(), &[CssPseudoClass::Scope]);
            assert_eq!(
                limit.rest()[0].combinator(),
                CssSelectorCombinator::Descendant
            );
            assert_eq!(limit.rest()[0].selector().classes(), &["limit".to_owned()]);
            if ancestry == CssStyleAncestor::Present {
                let [run] = scope.rules().rules() else {
                    panic!("direct run remains style-eligible through scope")
                };
                property(scoped_run(run), CssKnownProperty::Color, "red");
                clean(&report);
            } else {
                assert!(scope.rules().rules().is_empty());
                recovered(&report);
            }
        }
    }
}
