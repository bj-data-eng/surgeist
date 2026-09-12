//! Authored custom-media definitions remain ordered payloads, not evaluated aliases.
use surgeist_css::{
    CssCustomMediaBody, CssNormalizationErrorKind, CssNormalizationLimits,
    CssNormalizationResource, CssNormalizedItem, CssRule, CssRuleContextKindRef, normalize_report,
    normalize_sheet, normalize_sheet_with_limits, parse_sheet,
};

#[test]
fn normalization_retains_distinct_boolean_and_empty_list_definitions() {
    let report = parse_sheet(concat!(
        "@custom-media --choice true;",
        "@custom-media --choice false;",
        "@custom-media --choice;",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(normalized.items().len(), 3);
    for (index, (item, authored)) in normalized
        .items()
        .iter()
        .zip(report.syntax().rules())
        .enumerate()
    {
        let CssNormalizedItem::Rule(context) = item else {
            panic!("definition occurrence");
        };
        let CssRuleContextKindRef::CustomMedia(definition) = context.kind() else {
            panic!("custom-media payload");
        };
        let CssRule::CustomMedia(original) = authored else {
            panic!("authored definition");
        };
        assert_eq!(definition, original);
        assert_eq!(definition.name().as_str(), "--choice");
        assert!(context.parent().is_none());
        match (index, definition.body()) {
            (0, CssCustomMediaBody::True) | (1, CssCustomMediaBody::False) => {}
            (2, CssCustomMediaBody::Media(list)) => assert!(list.queries().is_empty()),
            _ => panic!("distinct authored body at {index}"),
        }
    }
    let [
        CssNormalizedItem::Rule(first),
        CssNormalizedItem::Rule(second),
        _,
    ] = normalized.items()
    else {
        unreachable!()
    };
    assert!(!first.same_context(second));
}

#[test]
fn scoped_and_conditional_contexts_remain_on_each_definition() {
    let source = concat!(
        "@scope (.root) {",
        "@media screen { @custom-media --nested (--external); }",
        "@custom-media --direct false;",
        "}",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let [
        CssNormalizedItem::Rule(scope),
        CssNormalizedItem::Rule(media),
        CssNormalizedItem::Rule(nested),
        CssNormalizedItem::Rule(direct),
    ] = normalized.items()
    else {
        panic!("two groups and two definitions");
    };
    assert!(matches!(scope.kind(), CssRuleContextKindRef::Scope { .. }));
    assert!(matches!(media.kind(), CssRuleContextKindRef::Media(_)));
    assert!(media.parent().unwrap().same_context(scope));
    assert!(nested.parent().unwrap().same_context(media));
    assert!(direct.parent().unwrap().same_context(scope));
    for (context, name) in [(nested, "--nested"), (direct, "--direct")] {
        let CssRuleContextKindRef::CustomMedia(definition) = context.kind() else {
            panic!("definition");
        };
        assert_eq!(definition.name().as_str(), name);
        assert_eq!(
            context.position().unwrap().byte_offset().value(),
            source.find(&format!("@custom-media {name}")).unwrap()
        );
    }
}

#[test]
fn normalized_report_preserves_recovered_definition_and_original_diagnostics() {
    let report = parse_sheet("@custom-media --x (color), ???, screen;");
    assert!(!report.is_clean());
    assert_eq!(report.diagnostics().len(), 1);
    let normalized = normalize_report(&report).unwrap();
    assert!(!normalized.is_clean());
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    let [CssNormalizedItem::Rule(context)] = normalized.syntax().items() else {
        panic!("retained definition");
    };
    let CssRuleContextKindRef::CustomMedia(definition) = context.kind() else {
        panic!("definition payload");
    };
    let [CssRule::CustomMedia(original)] = report.syntax().rules() else {
        panic!("authored definition");
    };
    assert_eq!(definition, original);
    assert!(definition.serialize().is_err());
}

#[test]
fn definition_occurrences_consume_rule_budget_without_partial_output() {
    let source = "@custom-media --a true; @custom-media --b false;";
    let report = parse_sheet(source);
    assert!(report.is_clean());
    let before = report.syntax().clone();
    let limits = CssNormalizationLimits::try_new(0, 1, 0, 0).unwrap();
    let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Rules,
            limit: 1,
        }
    ));
    assert_eq!(
        error.position().unwrap().byte_offset().value(),
        source.find("@custom-media --b").unwrap()
    );
    assert_eq!(*report.syntax(), before);
    let sufficient = CssNormalizationLimits::try_new(0, 2, 0, 0).unwrap();
    assert_eq!(
        normalize_sheet_with_limits(report.syntax(), sufficient)
            .unwrap()
            .items()
            .len(),
        2
    );
}
