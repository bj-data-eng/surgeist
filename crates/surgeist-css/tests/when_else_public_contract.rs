#![forbid(unsafe_code)]
//! Functional new-API evidence for #642 and the adopted #647 grammar profile.
//! Diagnostics and compact output follow the adopted CSS-owned model contract;
//! matching, truth and literal browser when/else output are not inferred.
use surgeist_css::*;

fn declaration_text(declaration: &CssDeclaration) -> (&str, &str) {
    let name = declaration.parsed_name().unwrap();
    let value = declaration.parsed_value().unwrap();
    (
        &name.source().as_str()
            [name.span().start().byte_offset().value()..name.span().end().byte_offset().value()],
        value.source().as_str()
            [value.span().start().byte_offset().value()..value.span().end().byte_offset().value()]
            .trim(),
    )
}

fn declarations(sheet: &CssNormalizedSheet) -> Vec<&CssNormalizedDeclaration> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect()
}

fn rule_at<'a>(sheet: &'a CssNormalizedSheet, source: &str, text: &str) -> &'a CssRuleContext {
    let offset = source.find(text).unwrap();
    sheet
        .items()
        .iter()
        .find_map(|item| match item {
            CssNormalizedItem::Rule(rule)
                if rule
                    .position()
                    .is_some_and(|position| position.byte_offset().value() == offset) =>
            {
                Some(rule)
            }
            _ => None,
        })
        .expect("retained original rule header")
}

fn assert_original_declarations(
    source: &str,
    sheet: &CssNormalizedSheet,
    expected: &[(&str, &str)],
) {
    let values = declarations(sheet);
    assert_eq!(
        values
            .iter()
            .map(|value| declaration_text(value.source()))
            .collect::<Vec<_>>(),
        expected
    );
    for (ordinal, value) in values.iter().enumerate() {
        assert_eq!(value.order(), ordinal);
        assert_eq!(
            value.source().parsed_name().unwrap().source().as_str(),
            source
        );
        assert_eq!(
            value.source().parsed_value().unwrap().source().as_str(),
            source
        );
        let (name, css) = expected[ordinal];
        let start = source
            .find(&format!("{name}:{css}"))
            .or_else(|| source.find(&format!("{name}: {css}")))
            .unwrap();
        assert_eq!(
            value
                .source()
                .parsed_name()
                .unwrap()
                .span()
                .start()
                .byte_offset()
                .value(),
            start
        );
    }
}

#[test]
fn recognized_invalid_preludes_report_exact_production_token_and_whole_unit() {
    for (prelude, marker, kind, authored) in [
        ("", "{", CssTokenKind::CurlyBracketBlock, "{"),
        ("media(width) and", "and", CssTokenKind::Ident, "and"),
        (
            "media(width) trailing",
            "trailing",
            CssTokenKind::Ident,
            "trailing",
        ),
        (
            "media(width), supports(display:grid)",
            ",",
            CssTokenKind::Comma,
            ",",
        ),
        (
            "media(width) and supports(display:grid) or media(height)",
            " or ",
            CssTokenKind::Ident,
            "or",
        ),
    ] {
        for name in ["when", "else"] {
            if name == "else" && prelude.is_empty() {
                continue;
            }
            let prefix = if name == "else" { "@media all{}" } else { "" };
            let failed = format!("@{name} {prelude}{{.Lost{{color:red}}}}");
            let source = format!("{prefix}{failed}.After{{color:blue}}");
            let report = parse_sheet(&source);
            let normalized = normalize_sheet(report.syntax()).unwrap();
            assert_original_declarations(&source, &normalized, &[("color", "blue")]);
            let [diagnostic] = report.diagnostics() else {
                panic!("one invalid prelude unit: {:?}", report.diagnostics());
            };
            let expected_byte = source.find(marker).unwrap() + usize::from(marker == " or ");
            assert_eq!(
                diagnostic.error().code(),
                CssErrorCode::InvalidAtRulePrelude
            );
            let ErrorKind::InvalidAtRulePrelude(detail) = diagnostic.error().kind() else {
                panic!("recognized condition grammar failure");
            };
            assert_eq!(detail.name().as_str(), name);
            assert_eq!(
                detail.production().as_str(),
                if name == "when" {
                    "ext.rule.when"
                } else {
                    "ext.rule.else"
                }
            );
            assert_eq!(
                detail.expectation().as_str(),
                "a condition in the adopted when grammar"
            );
            let token = detail.encountered().unwrap();
            assert_eq!(token.kind(), kind);
            assert_eq!(token.authored(), authored);
            assert_eq!(
                diagnostic.error().position().byte_offset().value(),
                expected_byte
            );
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
            let start = source.find(&failed).unwrap();
            assert_eq!(diagnostic.span().start().byte_offset().value(), start);
            assert_eq!(
                diagnostic.span().end().byte_offset().value(),
                start + failed.len()
            );
        }
    }
}

#[test]
fn decoded_at_keyword_and_non_bmp_prefix_keep_original_utf16_diagnostic_coordinates() {
    let source = ".Before{} /*🦀*/ @\\77 hen media(width) and{.Lost{color:red}}.After{color:blue}";
    let report = parse_sheet(source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid when prelude");
    };
    let ErrorKind::InvalidAtRulePrelude(detail) = diagnostic.error().kind() else {
        panic!("decoded recognized when");
    };
    assert_eq!(detail.name().as_str(), "when");
    assert_eq!(detail.production().as_str(), "ext.rule.when");
    let byte = source.find("and{").unwrap();
    assert_eq!(diagnostic.error().position().byte_offset().value(), byte);
    assert_eq!(
        diagnostic.error().position().column().value() as usize,
        source[..byte].encode_utf16().count()
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find("@\\77").unwrap()
    );
    assert_original_declarations(
        source,
        &normalize_sheet(report.syntax()).unwrap(),
        &[("color", "blue")],
    );
}

#[test]
fn free_else_placement_is_distinct_from_intrinsic_grammar_and_isolated_rule_rejection() {
    let source = "@else{.Lost{mystery:1}}.After{color:blue}";
    let report = parse_sheet(source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one outer placement diagnostic; descendant errors discarded");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePlacement
    );
    let ErrorKind::InvalidAtRulePlacement(detail) = diagnostic.error().kind() else {
        panic!("free else placement");
    };
    assert_eq!(detail.name().as_str(), "else");
    assert_eq!(detail.production().as_str(), "ext.rule.else");
    assert_eq!(
        detail.expected_context().as_str(),
        "after a conditional group separated only by whitespace or comments"
    );
    assert_eq!(diagnostic.error().position().byte_offset().value(), 0);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_original_declarations(
        source,
        &normalize_sheet(report.syntax()).unwrap(),
        &[("color", "blue")],
    );
    let isolated = parse_rule("@else{}", &CssNamespaceContext::default());
    assert!(isolated.syntax().is_none());
    assert_eq!(
        isolated.diagnostics()[0].action(),
        CssRecoveryAction::RejectInput
    );
    assert_eq!(
        isolated.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRulePlacement
    );
    let invalid = parse_sheet("@else media(width) and{.Lost{color:red}}");
    assert_eq!(
        invalid.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );
}

#[test]
fn missing_when_block_and_terminal_components_keep_owned_failure_categories() {
    let statement = parse_sheet("@when media(width);.After{color:blue}");
    let [diagnostic] = statement.diagnostics() else {
        panic!("one missing block");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidAtRuleBody);
    let ErrorKind::InvalidAtRuleBody(detail) = diagnostic.error().kind() else {
        panic!("required when block");
    };
    assert_eq!(detail.production().as_str(), "ext.rule.when");
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    for prelude in ["future(\"bad\n)", "future(url(a b))", "future(])"] {
        let source = format!("@when {prelude}{{.Lost{{color:red}}}}.After{{color:blue}}");
        let report = parse_sheet(&source);
        assert_original_declarations(
            &source,
            &normalize_sheet(report.syntax()).unwrap(),
            &[("color", "blue")],
        );
        assert_eq!(
            report.diagnostics()[0].error().code(),
            CssErrorCode::InvalidComponentValue
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
    }
}

#[test]
fn rejected_eof_else_discards_child_closures_but_keeps_ancestor_and_global_comment() {
    let source = "@media all{.A{}@else{.Lost{mystery:1}/*";
    let report = parse_sheet(source);
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("retained media ancestor");
    };
    assert!(matches!(media.rules(), [CssRule::Style(_)]));
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::DropAtRule)
            .count(),
        1
    );
    // Only media's actual unmatched opening remains a retained closure owner.
    assert_eq!(report.diagnostics().iter().filter(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure).count(), 1);
    assert_eq!(report.diagnostics().iter().filter(|diagnostic| diagnostic.action() == CssRecoveryAction::IgnoreUnterminatedComment).count(), 1);
    assert!(
        !report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.error().code() == CssErrorCode::UnknownProperty)
    );
    let drop = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.action() == CssRecoveryAction::DropAtRule)
        .unwrap();
    assert_eq!(
        drop.span().start().byte_offset().value(),
        source.find("@else").unwrap()
    );
    assert_eq!(drop.span().end().byte_offset().value(), source.len());
}

#[test]
fn rejected_else_does_not_suppress_independent_global_escape_recovery() {
    let source = "@media all{.A{}@else{.Lost{color:\\\n}}}";
    let report = parse_sheet(source);
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("media parent");
    };
    assert!(matches!(media.rules(), [CssRule::Style(_)]));
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::DropAtRule)
            .count(),
        1
    );
    let escapes = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::RecoverEscape)
        .collect::<Vec<_>>();
    let [escape] = escapes.as_slice() else {
        panic!("independent tokenizer escape diagnostic");
    };
    assert_eq!(
        escape.span().start().byte_offset().value(),
        source.find('\\').unwrap()
    );
    assert!(
        !report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );
}

#[test]
fn conditional_starters_continue_else_but_nonconditional_or_child_starters_do_not() {
    for starter in [
        "@media all{}",
        "@supports (display:grid){}",
        "@container (width > 1px){}",
        "@when media(width){}",
    ] {
        let source = format!(
            "{starter}/**/@else{{.Take{{color:red}}}}@else supports(display:grid){{.Later{{color:blue}}}}"
        );
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_original_declarations(
            &source,
            &normalize_sheet(report.syntax()).unwrap(),
            &[("color", "red"), ("color", "blue")],
        );
        assert!(
            matches!(report.syntax().rules()[1], CssRule::Else(ref rule) if rule.condition().is_none())
        );
        assert!(
            matches!(report.syntax().rules()[2], CssRule::Else(ref rule) if rule.condition().is_some())
        );
    }
    for prefix in [
        "@layer theme{}",
        "@scope{}",
        ".P{@when media(width){}}",
        "@when media(width) and{}",
        "@when media(width){}@else media(width) and{}",
    ] {
        let source = format!("{prefix}@else{{.Lost{{color:red}}}}.After{{color:blue}}");
        let report = parse_sheet(&source);
        assert_original_declarations(
            &source,
            &normalize_sheet(report.syntax()).unwrap(),
            &[("color", "blue")],
        );
        assert!(report.diagnostics().iter().any(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidAtRulePlacement));
    }
}

#[test]
fn original_separators_and_condition_origins_survive_every_structural_boundary() {
    for depth in [63, 64, 65, 127, 128, 129, 255, 256] {
        for gap in [
            "",
            "/* ; @unknown{} */",
            ";",
            "@unknown;",
            "@unknown{.Hidden{width:1px}}",
        ] {
            let source = format!(
                "{}@when media(width){{.Take{{color:red}}}}{gap}@else{{.ElseTake{{color:blue}}}}{}.Outside{{height:2px}}",
                "@media all{".repeat(depth - 2),
                "}".repeat(depth - 2)
            );
            let report = parse_sheet(&source);
            let original = report.clone();
            let normalized = normalize_sheet(report.syntax()).unwrap();
            let valid_gap = gap.is_empty() || gap.starts_with("/*");
            let expected = if valid_gap {
                vec![("color", "red"), ("color", "blue"), ("height", "2px")]
            } else {
                vec![("color", "red"), ("height", "2px")]
            };
            assert_original_declarations(&source, &normalized, &expected);
            let when = rule_at(&normalized, &source, "@when");
            let CssRuleContextKindRef::When(condition) = when.kind() else {
                panic!("symbolic when context");
            };
            assert_eq!(condition.serialize().unwrap().as_css(), " media(width)");
            let CssValueOrigin::Parsed(origin) = condition.origin() else {
                panic!("original condition source");
            };
            assert_eq!(origin.source().as_str(), source);
            assert_eq!(
                origin.span().start().byte_offset().value(),
                source.find("media(width)").unwrap()
            );
            let take = rule_at(&normalized, &source, ".Take");
            assert!(take.parent().unwrap().same_context(when));
            if valid_gap {
                let otherwise = rule_at(&normalized, &source, "@else");
                assert!(matches!(
                    otherwise.kind(),
                    CssRuleContextKindRef::Else(None)
                ));
                assert!(
                    rule_at(&normalized, &source, ".ElseTake")
                        .parent()
                        .unwrap()
                        .same_context(otherwise)
                );
                assert!(
                    otherwise
                        .parent()
                        .unwrap()
                        .same_context(when.parent().unwrap())
                );
                assert!(
                    report.is_clean(),
                    "depth {depth}: {:?}",
                    report.diagnostics()
                );
            } else {
                let start = source.find("@else").unwrap();
                assert!(!normalized.items().iter().any(|item| matches!(item, CssNormalizedItem::Rule(rule) if rule.position().is_some_and(|position| position.byte_offset().value() == start))));
                if gap == ";" {
                    // Ordinary rule-list grammar consumes ';@else{...}' as one
                    // invalid qualified rule before an Else member exists.
                    let unit = ";@else{.ElseTake{color:blue}}";
                    let failed_start = source.find(unit).unwrap();
                    let qualified = report
                        .diagnostics()
                        .iter()
                        .find(|diagnostic| {
                            diagnostic.span().start().byte_offset().value() == failed_start
                        })
                        .unwrap();
                    assert_eq!(qualified.action(), CssRecoveryAction::DropQualifiedRule);
                    assert_eq!(
                        qualified.span().end().byte_offset().value(),
                        failed_start + unit.len()
                    );
                } else {
                    let placement = report
                        .diagnostics()
                        .iter()
                        .find(|diagnostic| {
                            diagnostic.error().code() == CssErrorCode::InvalidAtRulePlacement
                        })
                        .unwrap();
                    assert_eq!(placement.error().position().byte_offset().value(), start);
                    assert_eq!(placement.action(), CssRecoveryAction::DropAtRule);
                    assert_eq!(placement.span().start().byte_offset().value(), start);
                    assert_eq!(
                        placement.span().end().byte_offset().value(),
                        start + "@else{.ElseTake{color:blue}}".len()
                    );
                }
            }
            assert_eq!(report, original);
        }
    }
}

#[test]
fn level_257_children_recover_locally_while_over_limit_group_rejects_as_resource() {
    let source = format!(
        "{}@when media(width){{.Lost{{color:red}}}}@else{{.AlsoLost{{color:blue}}}}{}.Outside{{height:2px}}",
        "@media all{".repeat(255),
        "}".repeat(255)
    );
    let report = parse_sheet(&source);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_original_declarations(&source, &normalized, &[("height", "2px")]);
    assert!(matches!(
        rule_at(&normalized, &source, "@when").kind(),
        CssRuleContextKindRef::When(_)
    ));
    assert!(matches!(
        rule_at(&normalized, &source, "@else").kind(),
        CssRuleContextKindRef::Else(None)
    ));
    for unit in [".Lost{color:red}", ".AlsoLost{color:blue}"] {
        let start = source.find(unit).unwrap();
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.span().start().byte_offset().value() == start)
            .unwrap();
        assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
        assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + unit.len()
        );
    }
    let over = format!(
        "{}@when media(width){{.Lost{{color:red}}}}{}.Outside{{height:2px}}",
        "@media all{".repeat(256),
        "}".repeat(256)
    );
    let report = parse_sheet(&over);
    assert_original_declarations(
        &over,
        &normalize_sheet(report.syntax()).unwrap(),
        &[("height", "2px")],
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(
                |diagnostic| diagnostic.error().code() == CssErrorCode::NestingLimit
                    && diagnostic.action() == CssRecoveryAction::StopAtNestingLimit
            )
    );
}

#[test]
fn scoped_conditional_groups_preserve_scope_relative_styles_across_replay() {
    for depth in [64, 128, 256] {
        let source = format!(
            "{}@when media(width){{> .Take{{color:red}}}}@else{{.Other{{color:blue}}}}{}",
            "@scope{".repeat(depth - 2),
            "}".repeat(depth - 2)
        );
        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        let normalized = normalize_sheet(report.syntax()).unwrap();
        assert_original_declarations(&source, &normalized, &[("color", "red"), ("color", "blue")]);
        let values = declarations(&normalized);
        assert!(values[0].selector_context().parent().is_none());
        assert_eq!(
            values[0].selector_context().selectors()[0].binding(),
            CssSelectorBinding::LeadingCombinator(CssSelectorCombinator::Child)
        );
        assert!(
            values[0]
                .selector_context()
                .scope_context()
                .unwrap()
                .same_context(values[1].selector_context().scope_context().unwrap())
        );
        assert!(
            values[0]
                .rule_context()
                .parent()
                .unwrap()
                .same_context(rule_at(&normalized, &source, "@when"))
        );
        assert!(
            values[1]
                .rule_context()
                .parent()
                .unwrap()
                .same_context(rule_at(&normalized, &source, "@else"))
        );
    }
}

#[test]
fn scoped_style_descendants_regain_nearest_style_binding_and_group_direct_runs() {
    let source = "@scope (.S){.P{color:red;@when media(width){> &.Child{height:1px}width:2px}@else{opacity:1}}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_original_declarations(
        source,
        &normalized,
        &[
            ("color", "red"),
            ("height", "1px"),
            ("width", "2px"),
            ("opacity", "1"),
        ],
    );
    let values = declarations(&normalized);
    assert!(
        values[1]
            .selector_context()
            .parent()
            .unwrap()
            .same_context(values[0].selector_context())
    );
    assert_eq!(
        values[1].selector_context().selectors()[0].binding(),
        CssSelectorBinding::LeadingCombinator(CssSelectorCombinator::Child)
    );
    assert!(
        values[2]
            .selector_context()
            .same_context(values[0].selector_context())
    );
    assert!(
        values[3]
            .selector_context()
            .same_context(values[0].selector_context())
    );
    assert!(
        values[2]
            .rule_context()
            .parent()
            .unwrap()
            .same_context(rule_at(&normalized, source, "@when"))
    );
    assert!(
        values[3]
            .rule_context()
            .parent()
            .unwrap()
            .same_context(rule_at(&normalized, source, "@else"))
    );
}

#[test]
fn checked_new_composition_reuses_source_mixed_members_and_rejects_free_else_atomically() {
    let source = "@when media(width){}@else supports(display:grid){.B{color:red}}";
    let parsed = parse_sheet(source);
    assert!(parsed.is_clean());
    let [CssRule::When(_), CssRule::Else(otherwise)] = parsed.syntax().rules() else {
        panic!("admitted source chain");
    };
    let starter = parse_sheet("@media all{}").syntax().rules()[0].clone();
    let before = otherwise.clone();
    let assembled =
        CssSheet::try_from_rules(vec![starter.clone(), CssRule::Else(otherwise.clone())]).unwrap();
    assert!(assembled.location().is_none());
    let normalized = normalize_sheet(&assembled).unwrap();
    let values = declarations(&normalized);
    assert_eq!(declaration_text(values[0].source()), ("color", "red"));
    assert_eq!(
        values[0].source().parsed_name().unwrap().source().as_str(),
        source
    );
    assert!(
        matches!(normalized.items()[1], CssNormalizedItem::Rule(ref rule) if matches!(rule.kind(), CssRuleContextKindRef::Else(Some(_))))
    );
    let alone = CssSheet::try_from_rules(vec![CssRule::Else(otherwise.clone())]).unwrap_err();
    assert_eq!(alone.kind(), CssRuleConstructionErrorKind::InvalidPlacement);
    assert_eq!(alone.path(), &[0]);
    assert_eq!(alone.position(), otherwise.position());
    let style = parse_sheet(".Intervening{}").syntax().rules()[0].clone();
    let broken = CssSheet::try_from_rules(vec![starter, style, CssRule::Else(otherwise.clone())])
        .unwrap_err();
    assert_eq!(
        broken.kind(),
        CssRuleConstructionErrorKind::InvalidPlacement
    );
    assert_eq!(broken.path(), &[2]);
    assert_eq!(otherwise, &before);
    let detached = CssElseRule::try_new(None, Vec::new(), &CssNamespaceContext::default()).unwrap();
    assert!(detached.condition().is_none() && detached.position().is_none());
    assert_eq!(
        CssSheet::try_from_rules(vec![CssRule::Else(detached)])
            .unwrap_err()
            .kind(),
        CssRuleConstructionErrorKind::InvalidPlacement
    );
}

#[test]
fn required_optional_and_scoped_checked_carriers_preserve_conditions_without_positions() {
    let context = CssNamespaceContext::default();
    let condition = parse_when_condition("media(width)").unwrap();
    let when = CssWhenRule::try_new(condition.clone(), Vec::new(), &context).unwrap();
    assert_eq!(when.condition(), &condition);
    assert!(when.position().is_none());
    let omitted = CssElseRule::try_new(None, Vec::new(), &context).unwrap();
    let later = CssElseRule::try_new(Some(condition.clone()), Vec::new(), &context).unwrap();
    let sheet = CssSheet::try_from_rules(vec![
        CssRule::When(when),
        CssRule::Else(omitted),
        CssRule::Else(later),
    ])
    .unwrap();
    assert_eq!(
        sheet.to_specified_css().unwrap(),
        "@when media(width) { }\n@else { }\n@else media(width) { }"
    );
    let scoped_when = CssScopedWhenRule::try_new(condition.clone(), Vec::new(), &context).unwrap();
    let scoped_else = CssScopedElseRule::try_new(None, Vec::new(), &context).unwrap();
    assert_eq!(scoped_when.condition(), &condition);
    assert!(scoped_when.position().is_none() && scoped_else.condition().is_none());
    let scope = CssScopeRule::try_new(
        None,
        None,
        vec![
            CssScopedRule::When(scoped_when),
            CssScopedRule::Else(scoped_else),
        ],
        &context,
        CssScopeNestingContext::None,
    )
    .unwrap();
    assert_eq!(
        CssRule::Scope(scope).to_specified_css().unwrap(),
        "@scope { @when media(width) { } @else { } }"
    );
}

#[test]
fn style_fragment_constructor_claim_is_rechecked_by_actual_enclosing_assembly() {
    let context = CssNamespaceContext::default();
    let report = parse_style_block("{.Before{}width:1px}", &context);
    assert!(report.is_clean());
    let block = report.syntax().as_ref().unwrap();
    let [CssRule::Style(_), run @ CssRule::NestedDeclarations(_)] = block.rules() else {
        panic!("original parsed declaration run");
    };
    let run = run.clone();
    let condition = parse_when_condition("media(width)").unwrap();
    assert_eq!(
        CssWhenRule::try_new(condition.clone(), vec![run.clone()], &context)
            .unwrap_err()
            .kind(),
        CssRuleConstructionErrorKind::InvalidPlacement
    );
    let fragment = CssWhenRule::try_new_in_style(condition, vec![run], &context).unwrap();
    assert_eq!(
        CssSheet::try_from_rules(vec![CssRule::When(fragment.clone())])
            .unwrap_err()
            .kind(),
        CssRuleConstructionErrorKind::InvalidPlacement
    );
    assert_eq!(fragment.rules().len(), 1);
    // Existing public style rules are parser-produced. Positive style assembly
    // is qualified through the parsed scoped-style case above.
}

#[test]
fn lexical_condition_and_compact_group_bytes_preserve_omission_and_grouping() {
    let source = "@when   MEDIA(width)/**/AnD (supports(display:grid) or future())   {.A{color:red}}@else{}@else supports(display:flex){.B{color:blue}}";
    let expected = "@when MEDIA(width)/**/AnD (supports(display:grid) or future()) { .A { color: red; } }\n@else { }\n@else supports(display:flex) { .B { color: blue; } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    let reparsed = parse_sheet(expected);
    assert!(reparsed.is_clean());
    let [
        CssRule::When(when),
        CssRule::Else(omitted),
        CssRule::Else(later),
    ] = reparsed.syntax().rules()
    else {
        panic!("three ordered conditional members");
    };
    assert!(matches!(
        when.condition().kind(),
        CssWhenConditionKind::And(_)
    ));
    assert!(omitted.condition().is_none());
    assert!(matches!(
        later.condition().unwrap().kind(),
        CssWhenConditionKind::SupportsDeclaration(_)
    ));
    assert_original_declarations(
        expected,
        &normalize_sheet(reparsed.syntax()).unwrap(),
        &[("color", "red"), ("color", "blue")],
    );
    assert_eq!(report, before);
}

#[test]
fn complete_specified_graph_shares_declared_provider_work_and_fails_atomically() {
    let context = CssNamespaceContext::default();
    let when = CssWhenRule::try_new(
        parse_when_condition("media(width)").unwrap(),
        Vec::new(),
        &context,
    )
    .unwrap();
    let otherwise = CssElseRule::try_new(None, Vec::new(), &context).unwrap();
    let sheet =
        CssSheet::try_from_rules(vec![CssRule::When(when), CssRule::Else(otherwise)]).unwrap();
    let before = sheet.clone();
    let expected = "@when media(width) { }\n@else { }";
    // Sheet1 + When1 + condition aggregate1 + function1 + ident1 + Else1.
    let exact = CssSpecifiedValueSerializationLimits::new(6, 6, expected.len());
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(5, 6, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 5, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 6, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let error = sheet.to_specified_css_with_limits(limits).unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(kind)
        );
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(sheet, before);
        assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    }
}

#[test]
fn literal_when_else_cssom_is_source_undefined_at_real_ordinary_paths() {
    for (source, path, kind) in [
        ("@when media(width){}", vec![0], CssRuleCssomKind::When),
        (
            "@media all{}.a{}@when media(width){}",
            vec![2],
            CssRuleCssomKind::When,
        ),
        (
            "@media all{.a{}@when media(width){}}",
            vec![0, 1],
            CssRuleCssomKind::When,
        ),
        ("@media all{}@else{}", vec![1], CssRuleCssomKind::Else),
        (
            "@media all{@media all{}@else{}}",
            vec![0, 1],
            CssRuleCssomKind::Else,
        ),
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let before = report.clone();
        let error = report.syntax().serialize_cssom().unwrap_err();
        assert_eq!(
            error.kind(),
            CssRuleCssomSerializationErrorKind::SourceUndefined(kind)
        );
        assert_eq!(error.rule_path(), path);
        assert!(std::error::Error::source(&error).is_none());
        assert!(report.syntax().to_specified_css().is_ok());
        assert_eq!(report, before);
    }
}

#[test]
fn scoped_literal_cssom_reports_the_existing_scope_front_before_descendants() {
    for (source, path) in [
        ("@scope{@when media(width){}@else{}}", vec![0]),
        (
            "@media all{.A{}@scope{@when media(width){}@else{}}}",
            vec![0, 1],
        ),
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let before = report.clone();
        let error = report.syntax().serialize_cssom().unwrap_err();
        assert_eq!(
            error.kind(),
            CssRuleCssomSerializationErrorKind::FormatUnavailable(CssRuleCssomFormat::Scope)
        );
        assert_eq!(error.rule_path(), path);
        assert!(report.syntax().to_specified_css().is_ok());
        assert_eq!(report, before);
    }
}

#[test]
fn rejected_outer_else_does_not_advance_retained_stylesheet_prelude_phase() {
    // Existing parser ordering follows successfully retained rules. Ignoring a
    // free Else must preserve initial import/namespace admission and bindings.
    let source = "@else{}@import 'early.css';@namespace p 'urn:p';p|a{}";
    let report = parse_sheet(source);
    let before = report.clone();
    let [
        CssRule::Import(import),
        CssRule::Namespace(namespace),
        CssRule::Style(style),
    ] = report.syntax().rules()
    else {
        panic!(
            "retained import, namespace and bound style in original order: {:?}",
            report.syntax().rules()
        );
    };
    assert_eq!(
        import.position().unwrap().byte_offset().value(),
        source.find("@import").unwrap()
    );
    assert_eq!(
        namespace.position().unwrap().byte_offset().value(),
        source.find("@namespace").unwrap()
    );
    assert_eq!(namespace.prefix().unwrap().as_str(), "p");
    assert_eq!(namespace.name().as_str(), "urn:p");
    assert_eq!(
        style.position().byte_offset().value(),
        source.find("p|a").unwrap()
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("only free Else placement loss: {:?}", report.diagnostics());
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePlacement
    );
    let ErrorKind::InvalidAtRulePlacement(detail) = diagnostic.error().kind() else {
        panic!("free Else owning placement diagnostic");
    };
    assert_eq!(detail.name().as_str(), "else");
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        "@else{}".len()
    );
    assert_eq!(report, before);
}

#[test]
fn valid_when_else_preludes_require_a_body_at_original_eof_or_statement_boundary() {
    for (name, prelude) in [
        ("when", " media(width)"),
        ("else", " media(width)"),
        ("else", ""),
    ] {
        for boundary in ["", ";"] {
            let prefix = if name == "else" { "@media all{}" } else { "" };
            let failed = format!("@{name}{prelude}{boundary}");
            let source = format!("{prefix}{failed}");
            let report = parse_sheet(&source);
            assert_eq!(report.syntax().rules().len(), usize::from(name == "else"));
            let [diagnostic] = report.diagnostics() else {
                panic!("only the missing outer body: {:?}", report.diagnostics());
            };
            assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidAtRuleBody);
            let ErrorKind::InvalidAtRuleBody(detail) = diagnostic.error().kind() else {
                panic!("recognized required body");
            };
            assert_eq!(detail.name().as_str(), name);
            assert_eq!(
                detail.production().as_str(),
                if name == "when" {
                    "ext.rule.when"
                } else {
                    "ext.rule.else"
                }
            );
            assert!(detail.encountered().is_none());
            // Existing missing-body diagnostics locate the boundary after a
            // consumed semicolon, or actual EOF (structured_errors.rs).
            assert_eq!(
                diagnostic.error().position().byte_offset().value(),
                source.len()
            );
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
            assert_eq!(
                diagnostic.span().start().byte_offset().value(),
                prefix.len()
            );
            assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
        }
    }
}

#[test]
fn intrinsic_prelude_and_terminal_url_failures_precede_missing_block_retagging() {
    for name in ["when", "else"] {
        let prefix = if name == "else" { "@media all{}" } else { "" };
        for (prelude, expected) in [
            ("media(width) and", CssErrorCode::InvalidAtRulePrelude),
            ("future(url(a b))", CssErrorCode::InvalidComponentValue),
        ] {
            let failed = format!("@{name} {prelude};");
            let source = format!("{prefix}{failed}.After{{color:blue}}");
            let report = parse_sheet(&source);
            assert_original_declarations(
                &source,
                &normalize_sheet(report.syntax()).unwrap(),
                &[("color", "blue")],
            );
            let [diagnostic] = report.diagnostics() else {
                panic!("one intrinsic outer failure: {:?}", report.diagnostics());
            };
            assert_eq!(diagnostic.error().code(), expected);
            match diagnostic.error().kind() {
                ErrorKind::InvalidAtRulePrelude(detail) => {
                    assert_eq!(detail.name().as_str(), name);
                    assert_eq!(
                        detail.production().as_str(),
                        if name == "when" {
                            "ext.rule.when"
                        } else {
                            "ext.rule.else"
                        }
                    );
                    assert_eq!(detail.encountered().unwrap().authored(), "and");
                    assert_eq!(
                        diagnostic.error().position().byte_offset().value(),
                        source.find("and;").unwrap()
                    );
                }
                ErrorKind::InvalidComponentValue(component) => {
                    assert_eq!(component.kind(), CssComponentValueErrorKind::BadUrl);
                    let CssValueOrigin::Parsed(origin) = component.origin() else {
                        panic!("original bad URL origin");
                    };
                    assert_eq!(origin.source().as_str(), source);
                    assert_eq!(
                        origin.span().start().byte_offset().value(),
                        source.find("url(").unwrap()
                    );
                }
                _ => panic!("intrinsic prelude or terminal component cause"),
            }
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
            assert_eq!(
                diagnostic.span().start().byte_offset().value(),
                prefix.len()
            );
            assert_eq!(
                diagnostic.span().end().byte_offset().value(),
                prefix.len() + failed.len()
            );
        }
    }
}

#[test]
fn discarded_incomplete_prelude_does_not_claim_a_retained_component_closure() {
    for name in ["when", "else"] {
        let prefix = if name == "else" { "@media all{}" } else { "" };
        let source = format!("{prefix}@{name} media(width");
        let report = parse_sheet(&source);
        assert_eq!(report.syntax().rules().len(), usize::from(name == "else"));
        let [diagnostic] = report.diagnostics() else {
            panic!(
                "discarded outer rule owns no retained implicit closure: {:?}",
                report.diagnostics()
            );
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidAtRuleBody);
        let ErrorKind::InvalidAtRuleBody(detail) = diagnostic.error().kind() else {
            panic!("missing body after an authored recoverable prelude");
        };
        assert_eq!(detail.name().as_str(), name);
        assert_eq!(
            detail.production().as_str(),
            if name == "when" {
                "ext.rule.when"
            } else {
                "ext.rule.else"
            }
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            prefix.len()
        );
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }
}

#[test]
fn decoded_authored_case_survives_when_else_prelude_diagnostics() {
    // CssAtRuleName retains decoded authored identity. Case-insensitive rule
    // dispatch does not authorize lowercasing this diagnostic payload.
    for (authored, decoded, production) in [
        ("WhEn", "WhEn", "ext.rule.when"),
        ("\\57 hEn", "WhEn", "ext.rule.when"),
        ("ELSE", "ELSE", "ext.rule.else"),
        ("\\45 LSE", "ELSE", "ext.rule.else"),
    ] {
        let failed = format!("@{authored} media(width) and{{}}");
        let check = |diagnostic: &CssRecoveryDiagnostic,
                     source: &str,
                     start: usize,
                     action: CssRecoveryAction| {
            assert_eq!(
                diagnostic.error().code(),
                CssErrorCode::InvalidAtRulePrelude
            );
            let ErrorKind::InvalidAtRulePrelude(detail) = diagnostic.error().kind() else {
                panic!("recognized authored prelude error");
            };
            assert_eq!(detail.name().as_str(), decoded);
            assert_eq!(detail.production().as_str(), production);
            assert_eq!(
                detail.expectation().as_str(),
                "a condition in the adopted when grammar"
            );
            let token = detail.encountered().unwrap();
            assert_eq!(token.kind(), CssTokenKind::Ident);
            assert_eq!(token.authored(), "and");
            assert_eq!(
                diagnostic.error().position().byte_offset().value(),
                source.find("and{}").unwrap()
            );
            assert_eq!(diagnostic.action(), action);
            assert_eq!(diagnostic.span().start().byte_offset().value(), start);
            assert_eq!(
                diagnostic.span().end().byte_offset().value(),
                start + failed.len()
            );
        };
        for (prefix, suffix) in [
            ("", ".After{color:blue}"),
            (".Parent{", ".After{color:blue}}"),
            ("@scope{", ".After{color:blue}}"),
        ] {
            let source = format!("{prefix}{failed}{suffix}");
            let report = parse_sheet(&source);
            let [diagnostic] = report.diagnostics() else {
                panic!("one authored prelude loss: {:?}", report.diagnostics());
            };
            check(
                diagnostic,
                &source,
                prefix.len(),
                CssRecoveryAction::DropAtRule,
            );
            assert_original_declarations(
                &source,
                &normalize_sheet(report.syntax()).unwrap(),
                &[("color", "blue")],
            );
        }
        let isolated = parse_rule(&failed, &CssNamespaceContext::default());
        assert!(isolated.syntax().is_none());
        let [diagnostic] = isolated.diagnostics() else {
            panic!("one isolated authored prelude rejection");
        };
        check(diagnostic, &failed, 0, CssRecoveryAction::RejectInput);

        for (prefix, suffix) in [("", ""), (".Parent{", "}"), ("@scope{", "}")] {
            let starter = if decoded == "ELSE" {
                "@media all{}"
            } else {
                ""
            };
            let source = format!("{prefix}{starter}@{authored} media(width){{}}{suffix}");
            let report = parse_sheet(&source);
            assert!(
                report.is_clean(),
                "case-independent admission: {:?}",
                report.diagnostics()
            );
            let normalized = normalize_sheet(report.syntax()).unwrap();
            let context = rule_at(&normalized, &source, &format!("@{authored}"));
            if decoded == "ELSE" {
                assert!(matches!(
                    context.kind(),
                    CssRuleContextKindRef::Else(Some(_))
                ));
            } else {
                assert!(matches!(context.kind(), CssRuleContextKindRef::When(_)));
            }
        }
    }
}

#[test]
fn decoded_authored_case_survives_free_else_placement_diagnostics() {
    for authored in ["ELSE", "\\45 LSE"] {
        let failed = format!("@{authored}{{}}");
        let check =
            |diagnostic: &CssRecoveryDiagnostic, start: usize, action: CssRecoveryAction| {
                assert_eq!(
                    diagnostic.error().code(),
                    CssErrorCode::InvalidAtRulePlacement
                );
                let ErrorKind::InvalidAtRulePlacement(detail) = diagnostic.error().kind() else {
                    panic!("free authored Else placement");
                };
                assert_eq!(detail.name().as_str(), "ELSE");
                assert_eq!(detail.production().as_str(), "ext.rule.else");
                assert_eq!(
                    detail.expected_context().as_str(),
                    "after a conditional group separated only by whitespace or comments"
                );
                assert_eq!(diagnostic.error().position().byte_offset().value(), start);
                assert_eq!(diagnostic.action(), action);
                assert_eq!(diagnostic.span().start().byte_offset().value(), start);
                assert_eq!(
                    diagnostic.span().end().byte_offset().value(),
                    start + failed.len()
                );
            };
        for (prefix, suffix) in [
            ("", ".After{color:blue}"),
            (".Parent{", ".After{color:blue}}"),
            ("@scope{", ".After{color:blue}}"),
        ] {
            let source = format!("{prefix}{failed}{suffix}");
            let report = parse_sheet(&source);
            let [diagnostic] = report.diagnostics() else {
                panic!("one free authored Else loss: {:?}", report.diagnostics());
            };
            check(diagnostic, prefix.len(), CssRecoveryAction::DropAtRule);
            assert_original_declarations(
                &source,
                &normalize_sheet(report.syntax()).unwrap(),
                &[("color", "blue")],
            );
        }
        let isolated = parse_rule(&failed, &CssNamespaceContext::default());
        assert!(isolated.syntax().is_none());
        let [diagnostic] = isolated.diagnostics() else {
            panic!("one isolated free authored Else rejection");
        };
        check(diagnostic, 0, CssRecoveryAction::RejectInput);
    }
}
