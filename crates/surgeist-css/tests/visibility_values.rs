#![forbid(unsafe_code)]
//! Display3 visibility is inherited, initially visible, with three keywords.
use surgeist_css::*;

#[test]
fn visibility_initial_and_constructed_values_have_exact_typed_payloads() {
    let metadata = CssKnownProperty::Visibility.metadata().unwrap();
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("longhand")
    };
    assert!(longhand.inherited_by_default());
    let initial = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("ordinary initial")
    };
    let CssLonghandValueRef::Visibility(value) = initial.view() else {
        panic!("visibility")
    };
    assert_eq!(value, &CssVisibility::Visible);
    for (text, expected, canonical) in [
        ("visible", CssVisibility::Visible, "visible"),
        ("HIDDEN", CssVisibility::Hidden, "hidden"),
        (r"c\6f llapse", CssVisibility::Collapse, "collapse"),
    ] {
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()]).unwrap();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Visibility),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        let report = parse_style_attribute(&format!("visibility:{text}!important"));
        assert!(report.is_clean());
        for declaration in [checked, report.syntax()[0].clone()] {
            let before = declaration.clone();
            let CssKnownPropertyValueRef::Visibility(value) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("visibility")
            };
            assert_eq!(value.current(), &expected);
            assert_eq!(value.i01_subset(), Some(&expected));
            assert_eq!(value.as_css(), text);
            assert_eq!(value.current().serialize_specified().unwrap(), canonical);
            assert_eq!(
                value
                    .current()
                    .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                        1, 1, 0
                    ))
                    .unwrap_err()
                    .kind(),
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            );
            assert_eq!(value.as_css(), text);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&declaration).unwrap()
            else {
                panic!("ordinary")
            };
            let [contribution] = values.items() else {
                panic!("one terminal")
            };
            let CssLonghandValueRef::Visibility(value) =
                contribution.ordinary_value().unwrap().view()
            else {
                panic!("visibility")
            };
            assert_eq!(value, &expected);
            assert_eq!(declaration.value_components(), before.value_components());
            assert!(contribution.source().same_occurrence(&declaration));
        }
    }
}

#[test]
fn visibility_canonical_keywords_obey_exact_limits_without_mutation() {
    for (value, expected) in [
        (CssVisibility::Visible, "visible"),
        (CssVisibility::Hidden, "hidden"),
        (CssVisibility::Collapse, "collapse"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let exact = CssSpecifiedValueSerializationLimits::new(1, 1, expected.len());
        assert_eq!(
            value.serialize_specified_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 0, 0),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, 0),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, expected.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, 0),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        let report = parse_style_attribute(&format!("visibility:{expected}"));
        assert!(report.is_clean());
        let CssKnownPropertyValueRef::Visibility(parsed) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("visibility")
        };
        assert_eq!(parsed.current(), &value);
        assert_eq!(parsed.current().serialize_specified().unwrap(), expected);
        assert_eq!(parsed.as_css(), expected);
    }
}

#[test]
fn recovered_visibility_normalization_preserves_occurrences_and_exact_budgets() {
    let text = ".a{visibility:visible;visibility:auto;visibility:hidden!important}";
    let report = parse_sheet(text);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert!(validate_sheet(text).is_err());
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
    let items: Vec<_> = sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 2);
    for (index, (item, (keyword, expected, importance))) in items
        .iter()
        .zip([
            ("visible", CssVisibility::Visible, CssImportance::Normal),
            ("hidden", CssVisibility::Hidden, CssImportance::Important),
        ])
        .enumerate()
    {
        assert_eq!(item.order(), index);
        assert_eq!(item.source().importance(), importance);
        assert_eq!(
            item.source().position().unwrap().byte_offset().value(),
            text.find(&format!("visibility:{keyword}")).unwrap()
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("ordinary")
        };
        let [contribution] = values.items() else {
            panic!("one terminal")
        };
        let CssLonghandValueRef::Visibility(value) = contribution.ordinary_value().unwrap().view()
        else {
            panic!("visibility")
        };
        assert_eq!(value, &expected);
        assert!(contribution.source().same_occurrence(item.source()));
    }
}

fn media_ancestors(mut context: &CssRuleContext) -> Vec<&CssRuleContext> {
    let mut media = Vec::new();
    loop {
        if matches!(context.kind(), CssRuleContextKindRef::Media(_)) {
            media.push(context);
        }
        match context.parent() {
            Some(parent) => context = parent,
            None => return media,
        }
    }
}

#[test]
fn nested_visibility_runs_preserve_contexts_across_structural_chunks() {
    for depth in [0, 63, 64, 65] {
        let source = format!(
            "@scope(.host){{{}.a{{visibility:hidden;@media all{{visibility:collapse;.child{{visibility:visible}}visibility:hidden}}visibility:visible}}{}}}",
            "@media all{".repeat(depth),
            "}".repeat(depth),
        );
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{depth}: {:?}", report.diagnostics());
        for _ in 0..2 {
            let sheet = normalize_sheet(report.syntax()).unwrap();
            let items: Vec<_> = sheet
                .items()
                .iter()
                .filter_map(|item| match item {
                    CssNormalizedItem::Declaration(value) => Some(value),
                    _ => None,
                })
                .collect();
            assert_eq!(items.len(), 5);
            let scope = items[0].selector_context().scope_context().unwrap();
            assert!(matches!(scope.kind(), CssRuleContextKindRef::Scope { .. }));
            for (index, (item, expected)) in items
                .iter()
                .zip([
                    CssVisibility::Hidden,
                    CssVisibility::Collapse,
                    CssVisibility::Visible,
                    CssVisibility::Hidden,
                    CssVisibility::Visible,
                ])
                .enumerate()
            {
                assert_eq!(item.order(), index);
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    item.expansion()
                else {
                    panic!("ordinary")
                };
                let [value] = values.items() else {
                    panic!("one terminal")
                };
                let CssLonghandValueRef::Visibility(actual) =
                    value.ordinary_value().unwrap().view()
                else {
                    panic!("visibility")
                };
                assert_eq!(actual, &expected);
                assert!(value.source().same_occurrence(item.source()));
                assert!(
                    item.selector_context()
                        .scope_context()
                        .unwrap()
                        .same_context(scope)
                );
                assert_eq!(
                    media_ancestors(item.rule_context()).len(),
                    depth + usize::from((1..=3).contains(&index))
                );
            }
            assert!(
                items[2]
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
            let media = media_ancestors(items[1].rule_context())[0];
            assert!(media.same_context(media_ancestors(items[2].rule_context())[0]));
            assert!(media.same_context(media_ancestors(items[3].rule_context())[0]));
            assert!(
                media
                    .parent()
                    .unwrap()
                    .same_context(items[0].rule_context())
            );
            // A declaration run after a child rule has its own rule context,
            // while retaining the containing selector and conditional ancestry.
            assert!(
                !items[1]
                    .rule_context()
                    .same_context(items[3].rule_context())
            );
            assert!(matches!(
                items[3].rule_context().kind(),
                CssRuleContextKindRef::NestedDeclarations(_)
            ));
            assert!(
                items[4]
                    .rule_context()
                    .parent()
                    .unwrap()
                    .same_context(items[0].rule_context())
            );
        }
    }
}
