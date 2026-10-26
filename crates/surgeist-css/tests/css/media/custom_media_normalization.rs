//! Authored custom-media definitions remain ordered payloads, not evaluated aliases.
use crate::css::support::authored_syntax::{assert_diagnostics, parsed_origin};
use surgeist_css::{
    CssCustomMediaBody, CssErrorCode, CssNormalizationLimits, CssNormalizationResource,
    CssNormalizedItem, CssParser, CssRecoveryAction, CssRule, CssRuleContextKindRef,
};

#[test]
fn normalization_retains_distinct_boolean_and_empty_list_definitions() {
    let report = CssParser::default().parse_sheet(concat!(
        "@custom-media --choice true;",
        "@custom-media --choice false;",
        "@custom-media --choice;",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = report.syntax().normalize(Default::default()).unwrap();
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
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = report.syntax().normalize(Default::default()).unwrap();
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
    let source = "@custom-media --x (color), ???, screen;";
    let report = CssParser::default().parse_sheet(source);
    // The second `<media-query>` member first fails at its `?` delimiter.
    let malformed = source.find("???").unwrap();
    assert_diagnostics(
        report.diagnostics(),
        &[(
            CssErrorCode::InvalidMediaQuery,
            CssRecoveryAction::ReplaceMediaQueryWithNever,
            malformed,
        )],
    );
    let normalized = report.normalize(Default::default()).unwrap();
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
    let error = definition.serialize_tokens(Default::default()).unwrap_err();
    assert!(
        crate::css::support::is_recovered_media_failure(&error),
        "recovered member: {error:?}"
    );
    let origin = parsed_origin(error.origin().expect("recovered member origin"));
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), malformed);
}

#[test]
fn definition_occurrences_consume_rule_budget_without_partial_output() {
    let source = "@custom-media --a true; @custom-media --b false;";
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean());
    let before = report.syntax().clone();
    let limits = CssNormalizationLimits::try_new(0, 1, 0, 0).unwrap();
    let error = report.syntax().normalize(limits).unwrap_err();
    assert!(error.exhausted_limit() == Some((CssNormalizationResource::Rules, 1)));
    assert_eq!(
        error.position().unwrap().byte_offset().value(),
        source.find("@custom-media --b").unwrap()
    );
    assert_eq!(*report.syntax(), before);
    let sufficient = CssNormalizationLimits::try_new(0, 2, 0, 0).unwrap();
    assert_eq!(
        report.syntax().normalize(sufficient).unwrap().items().len(),
        2
    );
}
