#![forbid(unsafe_code)]
// Install as a cfg(test) child of parser; never a public fallback test hook.
use super::{
    BoundedParseContext, StyleContextCaptures, conditional_chains, parse_sheet_bounded, recovery,
};
use crate::{
    CssErrorCode, CssNamespaceConstraint, CssNamespaceContext, CssNamespaceName,
    CssNamespacePrefix, CssParseReport, CssRecoveryAction, CssRule, CssScopedRule, CssSelector,
    CssSheet, CssSourcePosition, CssSourceSnapshot, CssStyleRule, CssStyleSelector, CssTokenKind,
    ErrorKind, parse_rule, parse_sheet, parse_style_block, validate_sheet,
};

const BATCH_BOUNDARIES: [usize; 3] = [31, 32, 33];
const PREFIX: &str = "/*😀*/\r\n";
const TRIVIA: &str = " /*🦀;{}*/ \t";
const AFTER: &str = ".after{color:red}";

fn original_position(position: CssSourcePosition, source: &str, offset: usize) {
    assert_eq!(position.byte_offset().value(), offset);
    assert_eq!(position.line().value(), 1);
    assert_eq!(
        position.column().value() as usize,
        source[..offset]
            .rsplit_once("\r\n")
            .unwrap()
            .1
            .encode_utf16()
            .count()
    );
}

fn original_style(style: &CssStyleRule, source: &str, start: usize, color: &str) {
    original_position(style.position(), source, start);
    let [declaration] = style.declarations().as_slice() else {
        panic!("one retained color declaration")
    };
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        color
    );
    let value = declaration.parsed_value().unwrap();
    let name = declaration.parsed_name().unwrap();
    assert_eq!(value.source().as_str(), source);
    assert!(value.source().same_snapshot(name.source()));
}

fn root_after(sheet: &CssSheet, source: &str) {
    let [_, CssRule::Style(after)] = sheet.rules() else {
        panic!("enclosing chain and independent root sibling")
    };
    original_style(after, source, source.find(".after{").unwrap(), "red");
}

fn media_children(sheet: &CssSheet, depth: usize) -> &[CssRule] {
    let CssRule::Media(first) = &sheet.rules()[0] else {
        panic!("outer Media")
    };
    let mut parent = first;
    for _ in 1..depth {
        let [CssRule::Media(child)] = parent.rules() else {
            panic!("every admitted Media parent survives")
        };
        parent = child;
    }
    parent.rules()
}

fn assert_layer_chain(
    sheet: &CssSheet,
    depth: usize,
    scoped: bool,
    start: Option<usize>,
    source: &str,
) {
    if scoped {
        let CssRule::Scope(first) = &sheet.rules()[0] else {
            panic!("outer Scope")
        };
        let mut parent = first;
        for _ in 1..depth {
            let [CssScopedRule::Scope(child)] = parent.rules().rules() else {
                panic!("every admitted Scope parent survives")
            };
            parent = child;
        }
        if let Some(start) = start {
            let [CssScopedRule::LayerStatement(statement)] = parent.rules().rules() else {
                panic!("innermost retained layer")
            };
            assert_eq!(
                statement.names().names()[0].components(),
                &["café".to_owned()]
            );
            assert_eq!(statement.names().names().len(), 1);
            original_position(statement.position(), source, start);
        } else {
            assert!(
                parent.rules().rules().is_empty(),
                "only over-limit child is removed"
            );
        }
    } else {
        let children = media_children(sheet, depth);
        if let Some(start) = start {
            let [CssRule::LayerStatement(statement)] = children else {
                panic!("innermost retained layer")
            };
            assert_eq!(
                statement.names().names()[0].components(),
                &["café".to_owned()]
            );
            assert_eq!(statement.names().names().len(), 1);
            original_position(statement.position(), source, start);
        } else {
            assert!(children.is_empty(), "only over-limit child is removed");
        }
    }
    root_after(sheet, source);
}

fn layer_source(depth: usize, scoped: bool, terminated: bool) -> (String, usize, usize) {
    let opening = if scoped { "@scope{" } else { "@media all{" };
    let mut source = format!("{PREFIX}{}", opening.repeat(depth));
    let start = source.len();
    source.push_str("@layer café");
    if terminated {
        source.push(';');
    }
    source.push_str(TRIVIA);
    let end = source.len();
    source.push_str(&"}".repeat(depth));
    source.push_str(AFTER);
    (source, start, end)
}

fn retained_eof<T>(report: &CssParseReport<T>, source: &str, start: usize, end: usize) {
    assert_eq!(&source[end..end + 1], "}", "actual explicit parent bound");
    assert!(end < source.len());
    let [diagnostic] = report.diagnostics() else {
        panic!("one retained statement EOF fault")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainNonconformingRule
    );
    let ErrorKind::UnexpectedEnd(detail) = diagnostic.error().kind() else {
        panic!("typed statement EOF")
    };
    assert_eq!(
        detail.expectation().as_str(),
        "a semicolon or block terminating an at-rule"
    );
    original_position(diagnostic.error().position(), source, end);
    original_position(diagnostic.span().start(), source, start);
    original_position(diagnostic.span().end(), source, end);
}

fn public_layer_boundaries(scoped: bool) {
    for depth in BATCH_BOUNDARIES {
        for terminated in [true, false] {
            let (source, start, end) = layer_source(depth, scoped, terminated);
            let report = parse_sheet(&source);
            assert_layer_chain(report.syntax(), depth, scoped, Some(start), &source);
            if terminated {
                assert!(
                    report.is_clean(),
                    "depth {depth}: {:?}",
                    report.diagnostics()
                );
                assert_eq!(validate_sheet(&source).unwrap(), *report.syntax());
            } else {
                retained_eof(&report, &source, start, end);
                assert_eq!(
                    validate_sheet(&source).unwrap_err().diagnostics(),
                    report.diagnostics()
                );
                assert_eq!(
                    report
                        .clone()
                        .into_validation_result()
                        .unwrap_err()
                        .diagnostics(),
                    report.diagnostics()
                );
            }
        }
    }
}

#[test]
fn media_parent_eof_and_semicolon_controls_survive_batch_boundaries() {
    public_layer_boundaries(false);
}

#[test]
fn scoped_parent_eof_and_semicolon_controls_survive_batch_boundaries() {
    public_layer_boundaries(true);
}

#[test]
fn nested_html_and_semicolon_prefixes_keep_one_failed_unit_at_batch_boundaries() {
    for (prefix, kind) in [
        ("<!--", CssTokenKind::Cdo),
        ("-->", CssTokenKind::Cdc),
        (";/**/", CssTokenKind::Semicolon),
    ] {
        let root = parse_sheet(&format!(
            "{prefix}@supports (color:red){{.hidden{{color:red}}}}"
        ));
        if kind != CssTokenKind::Semicolon {
            assert!(root.is_clean());
            assert!(matches!(root.syntax().rules(), [CssRule::Supports(_)]));
        } else {
            assert!(root.syntax().rules().is_empty());
            let style = parse_sheet(".host{;color:red;;}");
            assert!(style.is_clean(), "style separators are a distinct grammar");
            let [CssRule::Style(host)] = style.syntax().rules() else {
                panic!("host")
            };
            assert_eq!(host.declarations().len(), 1);
        }
        for depth in BATCH_BOUNDARIES {
            let unit = format!("{prefix}@supports (color:red){{.hidden{{color:red}}}}");
            let source = format!(
                "{PREFIX}{}{unit}.kept{{color:blue}}{}{AFTER}",
                "@media all{".repeat(depth),
                "}".repeat(depth)
            );
            let report = parse_sheet(&source);
            let [CssRule::Style(kept)] = media_children(report.syntax(), depth) else {
                panic!("failed qualified unit cannot promote its hidden descendant")
            };
            original_style(kept, &source, source.find(".kept{").unwrap(), "blue");
            root_after(report.syntax(), &source);
            let [diagnostic] = report.diagnostics() else {
                panic!("one whole failed unit")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
            assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
            let start = source.find(&unit).unwrap();
            original_position(diagnostic.error().position(), &source, start);
            original_position(diagnostic.span().start(), &source, start);
            original_position(diagnostic.span().end(), &source, start + unit.len());
            let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
                panic!("selector fault")
            };
            let encountered = detail.encountered().unwrap();
            assert_eq!(encountered.kind(), kind);
            assert_eq!(
                encountered.authored(),
                if kind == CssTokenKind::Semicolon {
                    ";"
                } else {
                    prefix
                }
            );
            assert_eq!(
                validate_sheet(&source).unwrap_err().diagnostics(),
                report.diagnostics()
            );
        }
    }
}

#[test]
fn root_only_statements_stay_misplaced_after_batch_replay() {
    for (statement, name) in [
        ("@import 'theme.css';", "import"),
        ("@namespace p 'urn:p';", "namespace"),
    ] {
        assert!(parse_sheet(&format!("{statement}{AFTER}")).is_clean());
        for depth in BATCH_BOUNDARIES {
            let source = format!(
                "{PREFIX}{}{statement}.kept{{color:blue}}{}{AFTER}",
                "@media all{".repeat(depth),
                "}".repeat(depth)
            );
            let report = parse_sheet(&source);
            let [CssRule::Style(kept)] = media_children(report.syntax(), depth) else {
                panic!("later child")
            };
            original_style(kept, &source, source.find(".kept{").unwrap(), "blue");
            root_after(report.syntax(), &source);
            let [diagnostic] = report.diagnostics() else {
                panic!("one misplaced statement")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
            assert_eq!(
                diagnostic.error().code(),
                CssErrorCode::InvalidAtRulePlacement
            );
            let start = source.find(statement).unwrap();
            original_position(diagnostic.span().start(), &source, start);
            original_position(diagnostic.span().end(), &source, start + statement.len());
            let ErrorKind::InvalidAtRulePlacement(detail) = diagnostic.error().kind() else {
                panic!("placement")
            };
            assert_eq!(detail.name().as_str(), name);
            assert_eq!(
                detail.expected_context().as_str(),
                "the stylesheet top level"
            );
            assert_eq!(
                validate_sheet(&source).unwrap_err().diagnostics(),
                report.diagnostics()
            );
        }
    }
}

fn namespaced_leaf(
    mut rules: &[CssRule],
    groups: usize,
    source: &str,
    prefix: &CssNamespacePrefix,
) {
    for _ in 0..groups {
        let [CssRule::Media(media)] = rules else {
            panic!("retained Media group")
        };
        rules = media.rules();
    }
    let [CssRule::Style(style)] = rules else {
        panic!("one retained namespaced style leaf")
    };
    let [CssStyleSelector::Selector(CssSelector::Compound(compound))] =
        style.selectors().selectors()
    else {
        panic!("qualified anchor")
    };
    assert_eq!(compound.nesting_selectors(), 1);
    assert_eq!(
        compound.type_selector().unwrap().namespace(),
        &CssNamespaceConstraint::Named(prefix.clone())
    );
    original_style(style, source, source.find("svg|leaf&{").unwrap(), "red");
}

fn atomic_rejection<T>(report: &CssParseReport<Option<T>>, source: &str) {
    assert!(report.syntax().is_none());
    let [diagnostic] = report.diagnostics() else {
        panic!("one atomic rejection")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
}

#[test]
fn exact_one_rule_preserves_supplied_namespace_and_rejects_trailing_unit_at_batch_boundaries() {
    let prefix = CssNamespacePrefix::try_new("svg").unwrap();
    let context = CssNamespaceContext::from_bindings([(
        Some(prefix.clone()),
        CssNamespaceName::new("urn:original"),
    )]);
    for depth in BATCH_BOUNDARIES {
        let groups = depth - 1;
        let source = format!(
            "{PREFIX}{}svg|leaf&{{color:red}}{} /*edge*/",
            "@media all{".repeat(groups),
            "}".repeat(groups)
        );
        let report = parse_rule(&source, &context);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        namespaced_leaf(
            std::slice::from_ref(report.syntax().as_ref().unwrap()),
            groups,
            &source,
            &prefix,
        );
        for suffix in [".second{}", " junk"] {
            let trailing = format!("{source}{suffix}");
            atomic_rejection(&parse_rule(&trailing, &context), &trailing);
        }
        assert_eq!(
            context.named_namespace(&prefix).unwrap().as_str(),
            "urn:original"
        );
    }
}

#[test]
fn real_brace_style_block_preserves_supplied_namespace_and_rejects_trailing_input_at_batch_boundaries()
 {
    let prefix = CssNamespacePrefix::try_new("svg").unwrap();
    let context = CssNamespaceContext::from_bindings([(
        Some(prefix.clone()),
        CssNamespaceName::new("urn:original"),
    )]);
    for depth in BATCH_BOUNDARIES {
        let groups = depth - 2;
        let source = format!(
            "{PREFIX}{{{}svg|leaf&{{color:red}}{}}} /*edge*/",
            "@media all{".repeat(groups),
            "}".repeat(groups)
        );
        let report = parse_style_block(&source, &context);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        let block = report.syntax().as_ref().unwrap();
        assert!(block.declarations().is_empty());
        namespaced_leaf(block.rules(), groups, &source, &prefix);
        assert_eq!(block.origin().source().as_str(), source);
        original_position(block.origin().span().start(), &source, PREFIX.len());
        original_position(
            block.origin().span().end(),
            &source,
            source.find(" /*edge*/").unwrap(),
        );
        for suffix in ["{}", " junk"] {
            let trailing = format!("{source}{suffix}");
            atomic_rejection(&parse_style_block(&trailing, &context), &trailing);
        }
        assert_eq!(
            context.named_namespace(&prefix).unwrap().as_str(),
            "urn:original"
        );
    }
}

#[test]
fn real_brace_style_block_keeps_statement_eof_at_the_actual_inner_parent_bound() {
    for depth in BATCH_BOUNDARIES {
        for terminated in [true, false] {
            let groups = depth - 1;
            let mut source = format!("{PREFIX}{{{}", "@media all{".repeat(groups));
            let start = source.len();
            source.push_str("@layer café");
            if terminated {
                source.push(';');
            }
            source.push_str(TRIVIA);
            let end = source.len();
            source.push_str(&"}".repeat(groups + 1));
            source.push_str(" /*edge*/");
            let report = parse_style_block(&source, &CssNamespaceContext::default());
            let block = report.syntax().as_ref().unwrap();
            assert!(block.declarations().is_empty());
            let mut rules = block.rules();
            for _ in 0..groups {
                let [CssRule::Media(parent)] = rules else {
                    panic!("admitted style-context Media")
                };
                rules = parent.rules();
            }
            let [CssRule::LayerStatement(statement)] = rules else {
                panic!("retained statement")
            };
            assert_eq!(
                statement.names().names()[0].components(),
                &["café".to_owned()]
            );
            assert_eq!(statement.names().names().len(), 1);
            original_position(statement.position(), &source, start);
            assert_eq!(block.origin().source().as_str(), source);
            original_position(block.origin().span().start(), &source, PREFIX.len());
            original_position(
                block.origin().span().end(),
                &source,
                source.find(" /*edge*/").unwrap(),
            );
            if terminated {
                assert!(
                    report.is_clean(),
                    "depth {depth}: {:?}",
                    report.diagnostics()
                );
            } else {
                retained_eof(&report, &source, start, end);
            }
        }
    }
}

fn direct_bounded(source: &str) -> CssParseReport<CssSheet> {
    let snapshot = CssSourceSnapshot::new(source);
    let report = parse_sheet_bounded(
        source,
        &snapshot,
        0,
        BoundedParseContext::Rules { top_level: true },
        StyleContextCaptures::default(),
        crate::CssParserContext::default(),
    );
    recovery::finish_report(source, conditional_chains::sheet(source, report))
}

fn direct_depth_on_two_mib(depth: usize) {
    std::thread::Builder::new()
        .name(format!("direct-css-depth-{depth}"))
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            for scoped in [false, true] {
                for terminated in [true, false] {
                    let (source, start, end) = layer_source(depth, scoped, terminated);
                    let report = direct_bounded(&source);
                    if depth <= 256 {
                        assert_layer_chain(report.syntax(), depth, scoped, Some(start), &source);
                        if terminated {
                            assert!(
                                report.is_clean(),
                                "depth {depth}: {:?}",
                                report.diagnostics()
                            );
                        } else {
                            retained_eof(&report, &source, start, end);
                        }
                    } else {
                        assert_layer_chain(report.syntax(), 256, scoped, None, &source);
                        let [diagnostic] = report.diagnostics() else {
                            panic!("only resource fault survives")
                        };
                        assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
                        assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
                        let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
                            panic!("typed limit")
                        };
                        assert_eq!(detail.limit(), 256);
                        let opening = if scoped { "@scope{" } else { "@media all{" };
                        assert_eq!(
                            detail.enclosing_production().as_str(),
                            if scoped {
                                "baseline.rule.scope"
                            } else {
                                "baseline.rule.media"
                            }
                        );
                        let failed_start = PREFIX.len() + opening.len() * 256;
                        original_position(
                            diagnostic.error().position(),
                            &source,
                            failed_start + opening.len() - 1,
                        );
                        original_position(diagnostic.span().start(), &source, failed_start);
                        original_position(diagnostic.span().end(), &source, end + 1);
                    }
                    let cloned = report.clone();
                    assert_eq!(cloned.syntax(), report.syntax());
                    assert_eq!(cloned.diagnostics(), report.diagnostics());
                    let output = report.syntax().to_specified_css().unwrap();
                    assert_eq!(
                        output
                            .matches(if scoped { "@scope" } else { "@media" })
                            .count(),
                        depth.min(256)
                    );
                    assert_eq!(output.contains("@layer café"), depth <= 256);
                    assert!(output.contains(".after"));
                    drop(cloned);
                    drop(report);
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn direct_bounded_owner_retains_255_groups_and_completes_lifecycle_on_two_mib() {
    direct_depth_on_two_mib(255);
}

#[test]
fn direct_bounded_owner_retains_256_groups_and_completes_lifecycle_on_two_mib() {
    direct_depth_on_two_mib(256);
}

#[test]
fn direct_bounded_owner_drops_only_group_257_on_two_mib() {
    direct_depth_on_two_mib(257);
}
