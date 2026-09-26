#![forbid(unsafe_code)]

//! The pinned Syntax 3 block consumer omits its final declaration transfer.
//! The catalog's terminal-retention reconciliation, controlled by Nesting 1
//! §§3.3 and 5, retains one nonempty final run at EOF in admitted style and
//! style-nested group bodies, as at a closing brace.

use surgeist_css::{
    CssDeclaration, CssErrorCode, CssNormalizedDeclaration, CssNormalizedItem, CssParseReport,
    CssRecoveryAction, CssRule, CssRuleContextKindRef, CssScopedRule, CssSheet, normalize_sheet,
    parse_sheet,
};

fn color(declaration: &CssDeclaration, expected: &str, offset: usize) {
    assert_eq!(
        declaration.known().unwrap().property().canonical_name(),
        "color"
    );
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        expected
    );
    assert_eq!(
        declaration.position().unwrap().byte_offset().value(),
        offset
    );
}

fn closures(source: &str, expected: usize) {
    let report = parse_sheet(source);
    assert_eq!(
        report.diagnostics().len(),
        expected,
        "{source}: {:?}",
        report.diagnostics()
    );
    for diagnostic in report.diagnostics() {
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            source.len()
        );
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }
}

fn normalized_colors(report: &CssParseReport<CssSheet>) -> Vec<CssNormalizedDeclaration> {
    normalize_sheet(report.syntax())
        .expect("supported color declarations normalize")
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn declaration_only_style_retains_one_terminal_run_with_or_without_semicolon() {
    for (source, expected_closures) in [
        (".p{color:red", 1),
        (".p{color:red;", 1),
        (".p{color:red}", 0),
    ] {
        closures(source, expected_closures);
        let report = parse_sheet(source);
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("one style parent: {source}")
        };
        let [declaration] = parent.declarations().as_slice() else {
            panic!("one leading declaration: {source}")
        };
        assert!(parent.rules().is_empty());
        color(declaration, "red", source.find("color:red").unwrap());
        let values = normalized_colors(&report);
        let [value] = values.as_slice() else {
            panic!("one normalized declaration: {source}")
        };
        assert_eq!(value.order(), 0);
        assert!(value.source().same_occurrence(declaration));
        assert!(matches!(
            value.rule_context().kind(),
            CssRuleContextKindRef::Style(_)
        ));
    }
}

#[test]
fn style_eof_after_child_retains_trailing_run_and_normalized_parent_identity() {
    for (source, expected_closures) in [
        (".p{color:red;.c{}color:blue", 1),
        (".p{color:red;.c{}color:blue;", 1),
        (".p{color:red;.c{}color:blue}", 0),
    ] {
        closures(source, expected_closures);
        let report = parse_sheet(source);
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("one style parent: {source}")
        };
        let [CssRule::Style(child), CssRule::NestedDeclarations(trailing)] = parent.rules() else {
            panic!("child then final run: {source}")
        };
        assert_eq!(parent.position().byte_offset().value(), 0);
        assert_eq!(
            child.position().byte_offset().value(),
            source.find(".c").unwrap()
        );
        assert!(child.declarations().is_empty());
        assert!(child.rules().is_empty());
        let [leading] = parent.declarations().as_slice() else {
            panic!("leading red")
        };
        let [last] = trailing.declarations().as_slice() else {
            panic!("final blue")
        };
        color(leading, "red", source.find("color:red").unwrap());
        color(last, "blue", source.find("color:blue").unwrap());
        assert_eq!(
            trailing.position().byte_offset().value(),
            source.find("color:blue").unwrap()
        );

        let values = normalized_colors(&report);
        let [first, second] = values.as_slice() else {
            panic!("two normalized colors")
        };
        assert_eq!((first.order(), second.order()), (0, 1));
        assert!(first.source().same_occurrence(leading));
        assert!(second.source().same_occurrence(last));
        assert!(matches!(
            first.rule_context().kind(),
            CssRuleContextKindRef::Style(_)
        ));
        assert!(matches!(
            second.rule_context().kind(),
            CssRuleContextKindRef::NestedDeclarations(_)
        ));
        assert!(
            second
                .rule_context()
                .parent()
                .unwrap()
                .same_context(first.rule_context())
        );
        assert!(
            first
                .selector_context()
                .same_context(second.selector_context())
        );
    }
}

#[test]
fn style_nested_groups_retain_final_eof_run_after_child_with_original_context() {
    for (prefix, group) in [
        ("@media all{", "media"),
        ("@supports (display:grid){", "supports"),
        ("@container (width > 1px){", "container"),
        ("@layer theme{", "layer"),
        ("@scope{", "scope"),
    ] {
        for (suffix, expected_closures) in [("", 2), ("}}", 0)] {
            let source = format!(".p{{{prefix}.c{{}}color:blue{suffix}");
            closures(&source, expected_closures);
            let report = parse_sheet(&source);
            let [CssRule::Style(parent)] = report.syntax().rules() else {
                panic!("style parent")
            };
            assert!(parent.declarations().is_empty());
            let [group_rule] = parent.rules() else {
                panic!("one nested group")
            };
            let trailing = match group_rule {
                CssRule::Media(rule) => {
                    assert_eq!(group, "media");
                    let [CssRule::Style(child), CssRule::NestedDeclarations(run)] = rule.rules()
                    else {
                        panic!("media child and run")
                    };
                    assert!(child.declarations().is_empty());
                    assert_eq!(
                        child.position().byte_offset().value(),
                        source.find(".c").unwrap()
                    );
                    run
                }
                CssRule::Supports(rule) => {
                    assert_eq!(group, "supports");
                    let [CssRule::Style(child), CssRule::NestedDeclarations(run)] = rule.rules()
                    else {
                        panic!("supports child and run")
                    };
                    assert!(child.declarations().is_empty());
                    assert_eq!(
                        child.position().byte_offset().value(),
                        source.find(".c").unwrap()
                    );
                    run
                }
                CssRule::Container(rule) => {
                    assert_eq!(group, "container");
                    let [CssRule::Style(child), CssRule::NestedDeclarations(run)] = rule.rules()
                    else {
                        panic!("container child and run")
                    };
                    assert!(child.declarations().is_empty());
                    assert_eq!(
                        child.position().byte_offset().value(),
                        source.find(".c").unwrap()
                    );
                    run
                }
                CssRule::LayerBlock(rule) => {
                    assert_eq!(group, "layer");
                    let [CssRule::Style(child), CssRule::NestedDeclarations(run)] = rule.rules()
                    else {
                        panic!("layer child and run")
                    };
                    assert!(child.declarations().is_empty());
                    assert_eq!(
                        child.position().byte_offset().value(),
                        source.find(".c").unwrap()
                    );
                    run
                }
                CssRule::Scope(rule) => {
                    assert_eq!(group, "scope");
                    let [
                        CssScopedRule::Style(child),
                        CssScopedRule::NestedDeclarations(run),
                    ] = rule.rules().rules()
                    else {
                        panic!("scope child and run")
                    };
                    assert!(child.declarations().is_empty());
                    assert_eq!(
                        child.position().byte_offset().value(),
                        source.find(".c").unwrap()
                    );
                    run
                }
                _ => panic!("unexpected nested group: {group}"),
            };
            let [last] = trailing.declarations().as_slice() else {
                panic!("one terminal color")
            };
            color(last, "blue", source.find("color:blue").unwrap());
            assert_eq!(
                trailing.position().byte_offset().value(),
                source.find("color:blue").unwrap()
            );

            let values = normalized_colors(&report);
            let [value] = values.as_slice() else {
                panic!("one normalized terminal color")
            };
            assert_eq!(value.order(), 0);
            assert!(value.source().same_occurrence(last));
            assert!(matches!(
                value.rule_context().kind(),
                CssRuleContextKindRef::NestedDeclarations(_)
            ));
            let context = value.rule_context().parent().unwrap();
            assert_eq!(
                context.position().unwrap().byte_offset().value(),
                source.find('@').unwrap()
            );
            assert!(matches!(
                (group, context.kind()),
                ("media", CssRuleContextKindRef::Media(_))
                    | ("supports", CssRuleContextKindRef::Supports(_))
                    | ("container", CssRuleContextKindRef::Container { .. })
                    | ("layer", CssRuleContextKindRef::LayerBlock(_))
                    | ("scope", CssRuleContextKindRef::Scope { .. })
            ));
            assert!(matches!(
                context.parent().unwrap().kind(),
                CssRuleContextKindRef::Style(_)
            ));
            assert!(value.selector_context().same_context(
                match context.parent().unwrap().kind() {
                    CssRuleContextKindRef::Style(selectors) => selectors,
                    _ => unreachable!(),
                }
            ));
        }
    }
}
