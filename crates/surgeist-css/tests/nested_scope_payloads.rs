#![forbid(unsafe_code)]
//! Authored scoped runs preserve declarations and nearest-style identity without
//! retargeting that style's selectors to newly entered scopes.
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nested-declarations-rule
use surgeist_css::*;

fn run(rule: &CssScopedRule) -> &CssNestedDeclarationsRule {
    let CssScopedRule::NestedDeclarations(run) = rule else {
        panic!("declaration run")
    };
    assert!(!run.declarations().is_empty());
    run
}

fn color(declaration: &CssDeclaration, expected: &str, source: &str) {
    assert_eq!(
        declaration.known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        expected
    );
    let offset = source.find(&format!("color:{expected}")).unwrap();
    assert_eq!(
        declaration.position().unwrap().byte_offset().value(),
        offset
    );
    let origin = declaration.parsed_value().unwrap();
    assert_eq!(origin.span().start().byte_offset().value(), offset + 6);
    assert_eq!(
        origin.span().end().byte_offset().value(),
        offset + 6 + expected.len()
    );
}

fn scope_from_style(style: &CssStyleRule) -> &CssScopeRule {
    let [CssRule::Scope(scope)] = style.rules() else {
        panic!("scope")
    };
    scope
}

fn fragment_runs(scope: &CssScopeRule, source: &str) {
    let [first, CssScopedRule::Style(child), last] = scope.rules().rules() else {
        panic!("run, child, run")
    };
    let first = run(first);
    let last = run(last);
    assert_eq!(first.declarations().len(), 1);
    assert_eq!(last.declarations().len(), 1);
    color(&first.declarations()[0], "red", source);
    color(&child.declarations()[0], "blue", source);
    color(&last.declarations()[0], "green", source);
    assert_eq!(
        first.position(),
        first.declarations()[0].position().unwrap()
    );
    assert_eq!(last.position(), last.declarations()[0].position().unwrap());
}

#[test]
fn public_fragments_retain_ordered_typed_runs_and_original_value_origins() {
    let ns = CssNamespaceContext::default();
    let source = ".p{@scope{color:red;.child{color:blue}color:green}}";
    let report = parse_rule(source, &ns);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let Some(CssRule::Style(style)) = report.syntax() else {
        panic!("style")
    };
    fragment_runs(scope_from_style(style), source);
    let source = "{@scope{color:red;.child{color:blue}color:green}}";
    let report = parse_style_block(source, &ns);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Scope(scope)] = report.syntax().as_ref().unwrap().rules() else {
        panic!("scope")
    };
    fragment_runs(scope, source);
}

#[test]
fn rejected_rule_boundaries_preserve_distinct_scoped_runs_and_eof_retention() {
    // Completed rules transfer the list; a no-rule result does not. These are
    // the same Syntax 3 block-contents boundaries as an ordinary nested group.
    for rejected in ["bad,{}", "@unknown;", "@layer a,b{}"] {
        let source = format!(".p{{@scope{{color:red;{rejected}color:green");
        let report = parse_sheet(&source);
        assert!(!report.is_clean());
        let [CssRule::Style(style)] = report.syntax().rules() else {
            panic!("style")
        };
        let [red, green] = scope_from_style(style).rules().rules() else {
            panic!("distinct runs")
        };
        color(&run(red).declarations()[0], "red", &source);
        color(&run(green).declarations()[0], "green", &source);
    }
    for rejected in [
        "bad;",
        "--bad:var(){color:blue};",
        "\\2d -bad:var(){color:blue};",
    ] {
        let source = format!(".p{{@scope{{color:red;{rejected}color:green}}}}");
        let report = parse_sheet(&source);
        let [CssRule::Style(style)] = report.syntax().rules() else {
            panic!("style")
        };
        let [only] = scope_from_style(style).rules().rules() else {
            panic!("unpartitioned run")
        };
        assert_eq!(run(only).declarations().len(), 2);
        color(&run(only).declarations()[0], "red", &source);
        color(&run(only).declarations()[1], "green", &source);
    }
}

#[test]
fn scoped_anchor_child_and_ordinary_grandchild_have_independent_selector_parents() {
    let source = ".p{@scope{color:red;& .child{color:blue;.grand{color:green}}color:black}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let sheet = normalize_sheet(report.syntax()).unwrap();
    let values: Vec<_> = sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 4);
    for (value, expected) in values.iter().zip(["red", "blue", "green", "black"]) {
        color(value.source(), expected, source);
    }
    let outer = values[0].selector_context();
    let child = values[1].selector_context();
    let grand = values[2].selector_context();
    assert!(child.parent().is_none());
    assert!(!child.same_context(outer));
    assert!(grand.parent().unwrap().same_context(child));
    assert!(values[3].selector_context().same_context(outer));
    let CssSelector::Complex(selector) = child.selectors()[0].selector() else {
        panic!("scoped complex")
    };
    assert!(selector.first().has_scope_anchor());
    assert_eq!(selector.first().nesting_selectors(), 0);
    assert!(outer.scope_context().is_none());
    assert!(
        child
            .scope_context()
            .unwrap()
            .same_context(values[0].rule_context().parent().unwrap())
    );
    assert!(
        grand
            .scope_context()
            .unwrap()
            .same_context(child.scope_context().unwrap())
    );
}

#[test]
fn bounded_scoped_splicing_splits_a_run_around_the_deferred_child() {
    for depth in [63, 64, 65, 127] {
        // Runs sit on both sides of a deep child, rather than only at its leaf.
        let source = format!(
            ".p{{@scope{{color:red;{}color:blue{}color:green}}}}",
            "@media all{".repeat(depth),
            "}".repeat(depth)
        );
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let [CssRule::Style(style)] = report.syntax().rules() else {
            panic!("style")
        };
        let [red, CssScopedRule::Media(_), green] = scope_from_style(style).rules().rules() else {
            panic!("split outer run")
        };
        color(&run(red).declarations()[0], "red", &source);
        color(&run(green).declarations()[0], "green", &source);
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let values: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        for (index, expected) in ["red", "blue", "green"].into_iter().enumerate() {
            color(values[index].source(), expected, &source);
            assert_eq!(values[index].order(), index);
            assert!(
                values[index]
                    .selector_context()
                    .same_context(values[0].selector_context())
            );
        }
    }
}

#[test]
fn depth_ceiling_keeps_detached_run_payload_and_selector_identity_on_small_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            // One style and 255 scope blocks reach the authored block ceiling.
            let source = format!(
                ".p{{{}color:red{} }}",
                "@scope{".repeat(255),
                "}".repeat(255)
            );
            let report = parse_sheet(&source);
            assert!(report.is_clean(), "{:?}", report.diagnostics());
            let [CssRule::Style(style)] = report.syntax().rules() else {
                panic!("style")
            };
            let mut scope = scope_from_style(style);
            for _ in 1..255 {
                let [CssScopedRule::Scope(inner)] = scope.rules().rules() else {
                    panic!("scope chain")
                };
                scope = inner;
            }
            let [leaf] = scope.rules().rules() else {
                panic!("run")
            };
            let detached = run(leaf).clone();
            let normalized = normalize_sheet(report.syntax()).unwrap();
            let declaration = normalized
                .items()
                .iter()
                .find_map(|item| match item {
                    CssNormalizedItem::Declaration(value) => Some(value.clone()),
                    _ => None,
                })
                .unwrap();
            let copy = report.syntax().clone();
            drop(report);
            drop(copy);
            drop(normalized);
            color(&detached.clone().declarations()[0], "red", &source);
            color(declaration.source(), "red", &source);
            assert_eq!(
                declaration.selector_context().selectors()[0].selector(),
                &CssSelector::Class("p".into())
            );
            drop(detached);
            drop(declaration);
            let source = format!(
                ".p{{{}color:red{} }}tail{{}}",
                "@scope{".repeat(256),
                "}".repeat(256)
            );
            let report = parse_sheet(&source);
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|d| d.error().code() == CssErrorCode::NestingLimit
                        && d.action() == CssRecoveryAction::StopAtNestingLimit)
            );
            assert!(matches!(
                report.syntax().rules().last(),
                Some(CssRule::Style(_))
            ));
        })
        .unwrap()
        .join()
        .unwrap();
}
