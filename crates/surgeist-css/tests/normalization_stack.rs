#![forbid(unsafe_code)]
//! Public normalization must handle parser-valid authored nesting on the
//! ordinary 2 MiB thread stack while preserving source order and provenance.

use surgeist_css::*;

const INNER_GROUPS: usize = 252;

fn deep_sheet() -> (String, Vec<usize>) {
    // The leaf has 255 enclosing/open authored blocks: .p, the outer @scope,
    // 252 alternating groups, and .child. This stays below the parser's
    // 256-level structural ceiling, leaving the whole sheet clean.
    let mut source = String::from(".p{@scope{color:red;");
    let mut group_offsets = Vec::with_capacity(INNER_GROUPS);
    for index in 0..INNER_GROUPS {
        group_offsets.push(source.len());
        source.push_str(if index % 2 == 0 {
            "@media all{"
        } else {
            "@scope{"
        });
    }
    source.push_str(".child{color:blue}");
    source.push_str(&"}".repeat(INNER_GROUPS));
    source.push_str("color:green}}.tail{color:black}");
    (source, group_offsets)
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

fn assert_color(value: &CssNormalizedDeclaration, source: &str, color: &str, order: usize) {
    let offset = source.find(&format!("color:{color}")).unwrap();
    assert_eq!(value.order(), order);
    assert_eq!(
        value.source().known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(
        value
            .source()
            .value_components()
            .serialize()
            .unwrap()
            .as_css(),
        color
    );
    assert_eq!(
        value.source().position().unwrap().byte_offset().value(),
        offset
    );
    let origin = value.source().parsed_value().unwrap().span();
    assert_eq!(
        origin.start().byte_offset().value(),
        offset + "color:".len()
    );
    assert_eq!(
        origin.end().byte_offset().value(),
        offset + "color:".len() + color.len()
    );
}

#[test]
fn alternating_groups_at_parser_ceiling_normalize_in_source_order_on_small_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let (source, group_offsets) = deep_sheet();
            let report = parse_sheet(&source);
            assert!(report.is_clean(), "{:?}", report.diagnostics());
            let sheet = normalize_sheet(report.syntax()).expect("parser-valid nesting normalizes");
            let values = declarations(&sheet);
            assert_eq!(values.len(), 4);
            for (index, color) in ["red", "blue", "green", "black"].into_iter().enumerate() {
                assert_color(values[index], &source, color, index);
            }

            let parent_selectors = values[0].selector_context();
            assert_eq!(
                parent_selectors.selectors()[0].selector(),
                &CssSelector::Class("p".into())
            );
            assert!(values[2].selector_context().same_context(parent_selectors));
            assert!(!values[1].selector_context().same_context(parent_selectors));
            assert_eq!(
                values[1].selector_context().selectors()[0].selector(),
                &CssSelector::Class("child".into())
            );
            assert!(values[1].selector_context().parent().is_none());
            assert_eq!(
                values[3].selector_context().selectors()[0].selector(),
                &CssSelector::Class("tail".into())
            );
            assert!(!values[3].selector_context().same_context(parent_selectors));
            assert!(values[3].rule_context().parent().is_none());

            let red_run = values[0].rule_context();
            let green_run = values[2].rule_context();
            assert!(matches!(
                red_run.kind(),
                CssRuleContextKindRef::NestedDeclarations(selectors)
                    if selectors.same_context(parent_selectors)
            ));
            assert!(matches!(
                green_run.kind(),
                CssRuleContextKindRef::NestedDeclarations(selectors)
                    if selectors.same_context(parent_selectors)
            ));
            assert!(!red_run.same_context(green_run));
            let outer_scope = red_run.parent().unwrap();
            assert!(matches!(
                outer_scope.kind(),
                CssRuleContextKindRef::Scope { .. }
            ));
            assert!(green_run.parent().unwrap().same_context(outer_scope));

            let child_rule = values[1].rule_context();
            assert!(matches!(
                child_rule.kind(),
                CssRuleContextKindRef::ScopedStyle(_)
            ));
            let mut ancestor = child_rule.parent().unwrap();
            assert!(
                values[1]
                    .selector_context()
                    .scope_context()
                    .unwrap()
                    .same_context(ancestor)
            );
            for (index, offset) in group_offsets.iter().enumerate().rev() {
                assert_eq!(ancestor.position().unwrap().byte_offset().value(), *offset);
                if index % 2 == 0 {
                    assert!(matches!(ancestor.kind(), CssRuleContextKindRef::Media(_)));
                } else {
                    assert!(matches!(
                        ancestor.kind(),
                        CssRuleContextKindRef::Scope { .. }
                    ));
                }
                ancestor = ancestor.parent().unwrap();
            }
            assert!(ancestor.same_context(outer_scope));
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn pure_scope_chain_at_structural_ceiling_normalizes_on_small_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            // The style plus 255 scope blocks reach the parser's 256-level
            // structural ceiling without crossing it.
            let source = format!(
                ".p{{{}color:red{} }}",
                "@scope{".repeat(255),
                "}".repeat(255)
            );
            let report = parse_sheet(&source);
            assert!(report.is_clean(), "{:?}", report.diagnostics());
            let sheet = normalize_sheet(report.syntax()).expect("ceiling-depth scopes normalize");
            let values = declarations(&sheet);
            assert_eq!(values.len(), 1);
            assert_color(values[0], &source, "red", 0);
            assert_eq!(
                values[0].selector_context().selectors()[0].selector(),
                &CssSelector::Class("p".into())
            );
            assert!(matches!(
                values[0].rule_context().kind(),
                CssRuleContextKindRef::NestedDeclarations(_)
            ));
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn colliding_depth_and_rule_budgets_report_depth_without_consuming_source() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let (source, group_offsets) = deep_sheet();
            let report = parse_sheet(&source);
            assert!(report.is_clean(), "{:?}", report.diagnostics());

            // Before inner group 239: style depth 0, outer scope depth 1,
            // one red declaration run, and inner groups 0..238 are admitted.
            // Thus group 239 has depth 241 with 242 rules already admitted.
            let limits = CssNormalizationLimits::try_new(240, 242, usize::MAX, usize::MAX)
                .expect("both budgets are within the structural ceiling");
            let error = normalize_sheet_with_limits(report.syntax(), limits)
                .expect_err("the next group exceeds both budgets");
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded {
                    resource: CssNormalizationResource::RuleDepth,
                    limit: 240,
                }
            );
            assert_eq!(
                error.position().unwrap().byte_offset().value(),
                group_offsets[239]
            );
            let enclosing = error.rule_context().expect("containing media context");
            assert!(matches!(enclosing.kind(), CssRuleContextKindRef::Media(_)));
            assert_eq!(
                enclosing.position().unwrap().byte_offset().value(),
                group_offsets[238]
            );
            assert!(error.declaration().is_none());
            assert!(error.declaration_order().is_none());

            // The failed attempt leaves the same retained syntax available to
            // the default, unrestricted count policy.
            let sheet = normalize_sheet(report.syntax()).expect("source remains usable");
            let values = declarations(&sheet);
            assert_eq!(values.len(), 4);
            for (index, color) in ["red", "blue", "green", "black"].into_iter().enumerate() {
                assert_color(values[index], &source, color, index);
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
