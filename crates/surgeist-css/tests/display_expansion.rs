#![forbid(unsafe_code)]
//! Display 3 longhand transport through shared expansion and normalization.
use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

#[test]
fn display_globals_transport_the_same_single_terminal_without_cascade() {
    for (text, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        for (suffix, importance) in [
            ("", CssImportance::Normal),
            ("!important", CssImportance::Important),
        ] {
            let source = declaration(&format!("display:{text}{suffix}"));
            let CssExpansion::Contributions(CssContributions::Longhands(items)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("global terminal")
            };
            let [item] = items.items() else {
                panic!("one terminal")
            };
            assert_eq!(item.property(), CssKnownProperty::Display);
            assert_eq!(item.value(), CssContributionValueRef::Global(expected));
            assert!(item.ordinary_value().is_none());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), importance);
        }
    }
}

#[test]
fn display_pending_reentry_is_complete_atomic_and_preserves_both_origins() {
    let source = declaration("display:var(--mode)!important");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending display")
    };
    assert!(handle.source().same_occurrence(&source));
    for input in [
        "list-item flex",
        "block grid-lanes",
        "inherit extra",
        "block; color:red",
        "inline!important",
    ] {
        let error = handle
            .reenter(parse_component_values(input).unwrap())
            .unwrap_err();
        assert!(
            matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(_)),
            "{input}: {error:?}"
        );
        assert!(handle.source().same_occurrence(&source));
    }
    assert_eq!(
        handle
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    for input in [
        "list-item flow-root inline",
        "inline-flex",
        "grid-lanes",
        "inline-grid-lanes",
    ] {
        let replacement = parse_component_values(input).unwrap();
        let direct = declaration(&format!("display:{input}"));
        let CssKnownPropertyValueRef::Display(direct) =
            direct.known().unwrap().property_value().unwrap()
        else {
            panic!("ordinary")
        };
        for _ in 0..2 {
            let CssContributions::Longhands(items) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("terminal replacement")
            };
            let [item] = items.items() else {
                panic!("one replacement")
            };
            assert_eq!(item.property(), CssKnownProperty::Display);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            let CssLonghandValueRef::Display(actual) = item.ordinary_value().unwrap().view() else {
                panic!("display payload")
            };
            assert_eq!(actual, direct.value());
            assert_eq!(item.replacement_components(), Some(&replacement));
            let CssValueOrigin::Parsed(original) = replacement.items()[0].origin() else {
                panic!("input origin")
            };
            let CssValueOrigin::Parsed(retained) =
                item.replacement_components().unwrap().items()[0].origin()
            else {
                panic!("retained origin")
            };
            assert!(original.source().same_snapshot(retained.source()));
        }
    }
    let CssContributions::Longhands(items) = handle
        .reenter(parse_component_values("inherit").unwrap())
        .unwrap()
    else {
        panic!("global replacement")
    };
    assert_eq!(items.items().len(), 1);
    assert_eq!(
        items.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );
}

fn normalized(sheet: &CssNormalizedSheet) -> Vec<&CssNormalizedDeclaration> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect()
}

#[test]
fn recovered_display_declaration_keeps_order_and_exact_normalization_budgets() {
    let source = ".a { display: inline; display: block grid-lanes; display: flex !important; }";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert!(validate_sheet(source).is_err());
    for (limits, resource, limit) in [
        (
            CssNormalizationLimits::try_new(0, 0, 2, 2).unwrap(),
            CssNormalizationResource::Rules,
            0,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 1, 2).unwrap(),
            CssNormalizationResource::Declarations,
            1,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 2, 1).unwrap(),
            CssNormalizationResource::Contributions,
            1,
        ),
    ] {
        assert_eq!(
            normalize_sheet_with_limits(report.syntax(), limits)
                .unwrap_err()
                .kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit }
        );
    }
    let sheet = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 2, 2).unwrap(),
    )
    .unwrap();
    let items = normalized(&sheet);
    assert_eq!(items.len(), 2);
    for (index, (text, importance)) in [
        ("inline", CssImportance::Normal),
        ("flex", CssImportance::Important),
    ]
    .into_iter()
    .enumerate()
    {
        let item = items[index];
        assert_eq!(item.order(), index);
        assert_eq!(item.source().importance(), importance);
        assert_eq!(
            item.source().position().unwrap().byte_offset().value(),
            source.find(&format!("display: {text}")).unwrap()
        );
        let CssKnownPropertyValueRef::Display(value) =
            item.source().known().unwrap().property_value().unwrap()
        else {
            panic!("display source")
        };
        assert_eq!(value.as_css(), text);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("ordinary terminal")
        };
        assert_eq!(values.items().len(), 1);
        assert!(values.items()[0].source().same_occurrence(item.source()));
    }
}

#[test]
fn nested_declaration_runs_keep_display_occurrences_and_shared_parent_context() {
    let source = ".a { display:inline; .bad, {display:none} display:flex; .child {display:grid-lanes} display:contents!important }";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropQualifiedRule
    );
    let sheet = normalize_sheet(report.syntax()).unwrap();
    let items = normalized(&sheet);
    assert_eq!(items.len(), 4);
    for (index, text) in ["inline", "flex", "grid-lanes", "contents"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(items[index].order(), index);
        let CssKnownPropertyValueRef::Display(value) = items[index]
            .source()
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("display")
        };
        assert_eq!(value.as_css(), text);
    }
    for index in [1, 3] {
        assert!(matches!(
            items[index].rule_context().kind(),
            CssRuleContextKindRef::NestedDeclarations(_)
        ));
        assert!(
            items[index]
                .rule_context()
                .parent()
                .unwrap()
                .same_context(items[0].rule_context())
        );
        assert!(
            items[index]
                .selector_context()
                .same_context(items[0].selector_context())
        );
    }
    assert!(
        !items[1]
            .rule_context()
            .same_context(items[3].rule_context())
    );
    assert!(
        items[2]
            .rule_context()
            .parent()
            .unwrap()
            .same_context(items[0].rule_context())
    );
    assert!(
        items[2]
            .selector_context()
            .parent()
            .unwrap()
            .same_context(items[0].selector_context())
    );
    assert_eq!(items[3].source().importance(), CssImportance::Important);
}

#[test]
fn scoped_display_and_color_runs_survive_structural_chunks_without_duplication() {
    for depth in [63, 64, 65] {
        let source = format!(
            "@scope(.host){{{}.a{{display:inline;color:red;display:flex;.child{{display:grid}}display:contents}}{}}}",
            "@media all{".repeat(depth),
            "}".repeat(depth),
        );
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{depth}: {:?}", report.diagnostics());
        // Repeated traversal must not consume or duplicate the authored graph.
        for _ in 0..2 {
            let sheet = normalize_sheet(report.syntax()).unwrap();
            let items = normalized(&sheet);
            assert_eq!(items.len(), 5);
            let scope = items[0].selector_context().scope_context().unwrap();
            assert!(matches!(scope.kind(), CssRuleContextKindRef::Scope { .. }));
            for (index, (property, text)) in [
                (CssKnownProperty::Display, "inline"),
                (CssKnownProperty::Color, "red"),
                (CssKnownProperty::Display, "flex"),
                (CssKnownProperty::Display, "grid"),
                (CssKnownProperty::Display, "contents"),
            ]
            .into_iter()
            .enumerate()
            {
                let item = items[index];
                assert_eq!(item.order(), index);
                assert_eq!(item.source().known().unwrap().property(), property);
                assert_eq!(
                    item.source()
                        .value_components()
                        .serialize()
                        .unwrap()
                        .as_css(),
                    text
                );
                assert!(
                    item.selector_context()
                        .scope_context()
                        .unwrap()
                        .same_context(scope)
                );
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    item.expansion()
                else {
                    panic!("terminal")
                };
                assert_eq!(values.items().len(), 1);
                assert_eq!(values.items()[0].property(), property);
                assert!(values.items()[0].source().same_occurrence(item.source()));
            }
            assert!(
                items[3]
                    .selector_context()
                    .parent()
                    .unwrap()
                    .same_context(items[0].selector_context())
            );
            assert!(
                items[4]
                    .selector_context()
                    .same_context(items[0].selector_context())
            );
        }
    }
}
