#![forbid(unsafe_code)]
//! Supplemental public authored-graph composition contracts.
//! CSS Syntax 3 sections 9.3/10, CSSOM 1 number/percentage serialization,
//! Animations 1 keyframe endpoint equivalence, MQ4 member-local recovery,
//! and the selected generic specified-output/provider composition policy.
//! These exercise existing callable boundaries; wrapper RED is not direct
//! evidence of a defect in the distinct authored Import/CustomMedia serializers.

use std::error::Error;
use surgeist_css::{
    CssContainerPrelude, CssContainerRule, CssCustomMediaBody, CssCustomMediaSerializationError,
    CssImportSerializationError, CssMediaCssomSerializationError, CssMediaSerializationError,
    CssNamespaceContext, CssRecoveryAction, CssRule, CssScopedRule, CssSheet,
    CssSpecifiedRuleSerializationError, CssSpecifiedRuleSerializationErrorKind as RuleError,
    CssSpecifiedValueSerializationError as ValueError,
    CssSpecifiedValueSerializationErrorKind as Kind,
    CssSpecifiedValueSerializationLimits as Limits, CssSupportsCondition, CssSupportsRule,
    parse_component_values, parse_sheet,
};

fn sheet(source: &str) -> CssSheet {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().clone()
}

fn exact(source: &str, expected: &str) {
    let input = sheet(source);
    let before = input.clone();
    assert_eq!(input.to_specified_css().unwrap(), expected);
    assert_eq!(input, before);
    assert_eq!(sheet(expected).to_specified_css().unwrap(), expected);
}

fn resource(error: &CssSpecifiedRuleSerializationError, kind: Kind, index: Option<usize>) {
    assert_eq!(error.kind(), RuleError::Resource(kind));
    assert_eq!(error.rule_index(), index);
    assert!(error.source().is_some());
}

fn media_cause(error: &CssSpecifiedRuleSerializationError) -> &CssMediaCssomSerializationError {
    let mut source = error.source();
    while let Some(cause) = source {
        if let Some(media) = cause.downcast_ref::<CssMediaCssomSerializationError>() {
            return media;
        }
        source = cause.source();
    }
    panic!("expected preserved typed media provider source: {error:?}")
}

#[test]
fn namespace_and_empty_font_controls_share_existing_sheet_costs() {
    let input = sheet("@namespace n '日本';@font-face{}");
    let before = input.clone();
    let expected = "@namespace n url(\"日本\");\n@font-face { }";
    // Sheet 1, namespace (rule + prefix + literal) 3, empty font rule 1.
    assert_eq!(
        input
            .to_specified_css_with_limits(Limits::new(5, 5, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(4, 5, expected.len()), Kind::InputNodeLimit),
        (Limits::new(5, 4, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(5, 5, expected.len() - 1), Kind::ByteLimit),
    ] {
        let error = input.to_specified_css_with_limits(limits).unwrap_err();
        resource(&error, kind, Some(1));
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<ValueError>()
                .unwrap()
                .kind(),
            kind
        );
        assert_eq!(input, before);
    }
}

#[test]
fn recovered_media_provider_control_keeps_member_order_and_exact_work_costs() {
    let report = parse_sheet("@media screen,???,print{}");
    let before = report.clone();
    let [CssRule::Media(rule)] = report.syntax().rules() else {
        panic!("media")
    };
    let expected = "screen, not all, print";
    assert_eq!(
        rule.query()
            .serialize_cssom_with_limits(Limits::new(6, 8, expected.len()))
            .unwrap()
            .as_css(),
        expected
    );
    let error = rule
        .query()
        .serialize_cssom_with_limits(Limits::new(6, 5, expected.len()))
        .unwrap_err();
    assert!(matches!(
        error,
        CssMediaCssomSerializationError::Resource { .. }
    ));
    assert_eq!(error.origin(), rule.query().queries()[1].origin());
    assert_eq!(report, before);
}

#[test]
fn keyframe_custom_and_pending_occurrences_keep_token_spelling_and_comments() {
    exact(
        "@keyframes k{from{--Case:A/**/B;opacity:var(--o,/*fallback*/.5);--Case:C;transform:var(--move)}}",
        "@keyframes k { 0% { --Case: A/**/B; opacity: var(--o,/*fallback*/.5); --Case: C; transform: var(--move); } }",
    );
}

#[test]
fn keyframe_custom_implicit_function_closure_emits_complete_tokens() {
    let report = parse_sheet("@keyframes k{from{--Case:Future(A/**/B");
    let before = report.clone();
    assert!(!report.is_clean());
    let [CssRule::Keyframes(rule)] = report.syntax().rules() else {
        panic!("retained keyframes")
    };
    assert_eq!(rule.blocks().len(), 1);
    assert_eq!(rule.blocks()[0].declarations().len(), 1);
    let expected = "@keyframes k { 0% { --Case: Future(A/**/B); } }";
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    assert_eq!(report, before);
    assert_eq!(sheet(expected).to_specified_css().unwrap(), expected);
}

#[test]
fn keyframe_pending_implicit_var_closure_emits_complete_tokens() {
    let report = parse_sheet("@keyframes k{to{opacity:var(--o");
    let before = report.clone();
    assert!(!report.is_clean());
    let [CssRule::Keyframes(rule)] = report.syntax().rules() else {
        panic!("retained keyframes")
    };
    assert!(
        rule.blocks()[0].declarations()[0]
            .known()
            .unwrap()
            .substitution_dependent()
            .is_some()
    );
    let expected = "@keyframes k { 100% { opacity: var(--o); } }";
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    assert_eq!(report, before);
    assert_eq!(sheet(expected).to_specified_css().unwrap(), expected);
}

#[test]
fn tiny_keyframe_offsets_project_to_zero_without_discarding_selectors_or_blocks() {
    let input = sheet("@keyframes k{0.0000001%,0.0000002%,from{}0%{opacity:.5}to,100%{opacity:1}}");
    let before = input.clone();
    let [CssRule::Keyframes(rule)] = input.rules() else {
        panic!("keyframes")
    };
    let selectors = rule.blocks()[0].selectors().selectors();
    let offsets = selectors
        .iter()
        .map(|selector| selector.offset().literal_value().unwrap())
        .collect::<Vec<_>>();
    assert!(offsets[0] > 0.0 && offsets[1] > offsets[0]);
    assert_eq!(offsets[2], 0.0);
    let expected =
        "@keyframes k { 0%, 0%, 0% { } 0% { opacity: 0.5; } 100%, 100% { opacity: 1; } }";
    assert_eq!(input.to_specified_css().unwrap(), expected);
    assert_eq!(input, before);
    let output = sheet(expected);
    let [CssRule::Keyframes(emitted)] = output.rules() else {
        panic!("emitted keyframes")
    };
    assert_eq!(emitted.blocks().len(), 3);
    assert_eq!(emitted.blocks()[0].selectors().selectors().len(), 3);
    assert_eq!(emitted.blocks()[2].selectors().selectors().len(), 2);
    assert_eq!(output.to_specified_css().unwrap(), expected);
}

#[test]
fn distinct_keyframe_offsets_round_to_one_millionth_without_coalescing() {
    let input = sheet("@keyframes k{0.0000011%{}0.0000012%{--v:A/**/B}}");
    let before = input.clone();
    let [CssRule::Keyframes(rule)] = input.rules() else {
        panic!("keyframes")
    };
    let first = rule.blocks()[0].selectors().selectors()[0]
        .offset()
        .literal_value()
        .unwrap();
    let second = rule.blocks()[1].selectors().selectors()[0]
        .offset()
        .literal_value()
        .unwrap();
    assert!(second > first);
    let expected = "@keyframes k { 0.000001% { } 0.000001% { --v: A/**/B; } }";
    assert_eq!(input.to_specified_css().unwrap(), expected);
    assert_eq!(input, before);
    assert_eq!(sheet(expected).to_specified_css().unwrap(), expected);
}

#[test]
fn generic_import_projects_recovered_members_while_authored_import_rejects_them() {
    let report =
        parse_sheet("/* 😀 */\r\n@import \"x\" layer(theme) screen,???,print;@namespace n 'u';");
    let before = report.clone();
    let [CssRule::Import(rule), CssRule::Namespace(_)] = report.syntax().rules() else {
        panic!("import and following namespace")
    };
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    let list = rule.media().unwrap();
    let origins = list
        .queries()
        .iter()
        .map(|query| query.origin().clone())
        .collect::<Vec<_>>();
    let authored = rule.serialize().unwrap_err();
    assert!(matches!(
        authored,
        CssImportSerializationError::Media(CssMediaSerializationError::RecoveredNever { .. })
    ));
    assert_eq!(authored.origin(), &origins[1]);
    let expected = "@import \"x\" layer(theme) screen, not all, print;\n@namespace n url(\"u\");";
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    assert_eq!(report, before);
    assert_eq!(
        list.queries()
            .iter()
            .map(|query| query.origin().clone())
            .collect::<Vec<_>>(),
        origins
    );
    assert_eq!(sheet(expected).to_specified_css().unwrap(), expected);
}

#[test]
fn generic_custom_media_projects_recovered_members_while_authored_definition_rejects_them() {
    let report = parse_sheet("/* λ */\n@custom-media --x (color),???,screen;");
    let before = report.clone();
    let [CssRule::CustomMedia(rule)] = report.syntax().rules() else {
        panic!("custom media")
    };
    let CssCustomMediaBody::Media(list) = rule.body() else {
        panic!("media body")
    };
    let origins = list
        .queries()
        .iter()
        .map(|query| query.origin().clone())
        .collect::<Vec<_>>();
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    let authored = rule.serialize().unwrap_err();
    assert!(matches!(
        authored,
        CssCustomMediaSerializationError::Media(CssMediaSerializationError::RecoveredNever { .. })
    ));
    assert_eq!(authored.origin(), &origins[1]);
    let expected = "@custom-media --x (color), not all, screen;";
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    assert_eq!(report, before);
    assert_eq!(
        list.queries()
            .iter()
            .map(|query| query.origin().clone())
            .collect::<Vec<_>>(),
        origins
    );
    assert_eq!(sheet(expected).to_specified_css().unwrap(), expected);
}

#[test]
fn generic_import_keeps_optional_clause_interpretation_of_opaque_media_functions() {
    for tail in [
        "layer()",
        "supports(2px)",
        "layer(theme) and (color)",
        "supports(display:grid) and (color)",
    ] {
        let source = format!("@import \"x\" {tail};");
        let input = sheet(&source);
        let before = input.clone();
        let [CssRule::Import(rule)] = input.rules() else {
            panic!("import")
        };
        assert!(rule.layer().is_none() && rule.supports().is_none());
        assert_eq!(rule.serialize().unwrap().as_css(), source);
        assert_eq!(input.to_specified_css().unwrap(), source);
        assert_eq!(input, before);
        let emitted = sheet(&source);
        let [CssRule::Import(rule)] = emitted.rules() else {
            panic!("emitted import")
        };
        assert!(rule.layer().is_none() && rule.supports().is_none());
    }
}

#[test]
fn lexical_conditions_keep_opaque_case_comments_and_grouping_inside_scoped_groups() {
    exact(
        "@scope{@supports ((width:1px) and Future(A/**/B)){@container ((width:1px) and Future(A/**/B)){.child{}}}}",
        "@scope { @supports ((width:1px) and Future(A/**/B)) { @container ((width:1px) and Future(A/**/B)) { .child { } } } }",
    );
}

fn exact_lexical_region_budget(input: CssSheet, expected: &str, nodes: usize) {
    let before = input.clone();
    assert_eq!(
        input
            .to_specified_css_with_limits(Limits::new(nodes, nodes, expected.len()))
            .unwrap(),
        expected
    );
    // The same rule fits without the sheet aggregate; the sheet cannot reset
    // its budget before consuming the condition/prelude provider.
    assert_eq!(
        input.rules()[0]
            .to_specified_css_with_limits(Limits::new(nodes - 1, nodes - 1, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            Limits::new(nodes - 1, nodes, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(nodes, nodes - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(nodes, nodes, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        resource(
            &input.to_specified_css_with_limits(limits).unwrap_err(),
            kind,
            Some(0),
        );
        assert_eq!(input, before);
    }
    assert_eq!(input, before);
}

#[test]
fn supports_omits_edge_whitespace_but_charges_the_complete_retained_region() {
    let context = CssNamespaceContext::default();
    let source = format!("{}(display:grid){}", " ".repeat(128), " ".repeat(256));
    let components = parse_component_values(&source).unwrap();
    // Two whitespace components, one block, and its three inner tokens.
    assert_eq!(components.component_count(), 6);
    let condition =
        CssSupportsCondition::try_from_components(components.clone(), &context).unwrap();
    let original = condition.clone();
    assert_eq!(condition.components(), components.items());
    // Existing lexical provider control: authored spelling includes both edges.
    assert_eq!(
        condition
            .serialize_with_limit(source.len())
            .unwrap()
            .as_css(),
        source
    );
    let rule = CssSupportsRule::try_new(condition, Vec::new(), &context).unwrap();
    let input = CssSheet::try_from_rules(vec![CssRule::Supports(rule)]).unwrap();
    // Lexical region root 1 + six components + rule 1 + sheet 1. No typed AST
    // or second component-list root is additionally charged. Omitted whitespace
    // participates in work, but not in final bytes or serialization scratch.
    exact_lexical_region_budget(input.clone(), "@supports (display:grid) { }", 9);
    let [CssRule::Supports(rule)] = input.rules() else {
        panic!("supports")
    };
    assert_eq!(rule.condition(), &original);

    let condition = CssSupportsCondition::try_from_components(
        parse_component_values("(display:grid)").unwrap(),
        &context,
    )
    .unwrap();
    let rule = CssSupportsRule::try_new(condition, Vec::new(), &context).unwrap();
    exact_lexical_region_budget(
        CssSheet::try_from_rules(vec![CssRule::Supports(rule)]).unwrap(),
        "@supports (display:grid) { }",
        7,
    );
}

#[test]
fn container_omits_edge_whitespace_but_charges_the_complete_retained_region() {
    let context = CssNamespaceContext::default();
    let source = format!("{}(width>1px){}", " ".repeat(128), " ".repeat(256));
    let components = parse_component_values(&source).unwrap();
    assert_eq!(components.component_count(), 6);
    let prelude = CssContainerPrelude::try_from_components(components.clone()).unwrap();
    let original = prelude.clone();
    assert_eq!(prelude.components(), components.items());
    assert_eq!(
        prelude.serialize_with_limit(source.len()).unwrap().as_css(),
        source
    );
    let rule = CssContainerRule::try_new(prelude, Vec::new(), &context).unwrap();
    let input = CssSheet::try_from_rules(vec![CssRule::Container(rule)]).unwrap();
    // The full lexical region has the same six components + one aggregate;
    // query-entry/condition views do not duplicate their backing component costs.
    exact_lexical_region_budget(input.clone(), "@container (width>1px) { }", 9);
    let [CssRule::Container(rule)] = input.rules() else {
        panic!("container")
    };
    assert_eq!(rule.prelude(), &original);

    let prelude =
        CssContainerPrelude::try_from_components(parse_component_values("(width>1px)").unwrap())
            .unwrap();
    let rule = CssContainerRule::try_new(prelude, Vec::new(), &context).unwrap();
    exact_lexical_region_budget(
        CssSheet::try_from_rules(vec![CssRule::Container(rule)]).unwrap(),
        "@container (width>1px) { }",
        7,
    );
}

#[test]
fn mixed_scope_style_and_ordinary_children_keep_relative_selectors_and_declaration_runs() {
    exact(
        "@namespace n 'u';@scope(.root){@media screen{.parent{opacity:.25;@supports (display:grid){> .child{opacity:.5}}opacity:.75;@scope to (> .stop){opacity:1}}}}@font-face{}",
        "@namespace n url(\"u\");\n@scope (.root) { @media screen { .parent { opacity: 0.25; @supports (display:grid) { > .child { opacity: 0.5; } } opacity: 0.75; @scope to (> .stop) { opacity: 1; } } } }\n@font-face { }",
    );
}

fn wide_group(width: usize) -> (CssSheet, String) {
    let source = format!("@media screen{{{}}}", "@font-face{}".repeat(width));
    let expected = format!(
        "@media screen {{ {} }}",
        vec!["@font-face { }"; width].join(" ")
    );
    (sheet(&source), expected)
}

#[test]
fn wide_group_charges_each_logical_leaf_once_without_enum_or_child_slice_costs() {
    let (input, expected) = wide_group(1024);
    let before = input.clone();
    // Sheet 1 + Media 1 + query list 1 + screen query/identifier 2 + 1024 leaves.
    let nodes = 1029;
    assert_eq!(
        input
            .to_specified_css_with_limits(Limits::new(nodes, nodes, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            Limits::new(nodes - 1, nodes, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(nodes, nodes - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(nodes, nodes, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        resource(
            &input.to_specified_css_with_limits(limits).unwrap_err(),
            kind,
            Some(0),
        );
        assert_eq!(input, before);
    }
}

#[test]
fn wide_group_stops_atomically_under_small_independent_work_budgets_and_can_retry() {
    let (input, expected) = wide_group(1024);
    let before = input.clone();
    for (limits, kind) in [
        (
            Limits::new(1, usize::MAX, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(usize::MAX, 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
    ] {
        resource(
            &input.to_specified_css_with_limits(limits).unwrap_err(),
            kind,
            Some(0),
        );
        assert_eq!(input, before);
    }
    assert_eq!(input.to_specified_css().unwrap(), expected);
}

#[test]
fn scoped_recovered_media_failure_keeps_provider_origin_and_enclosing_top_level_index() {
    let report = parse_sheet("@namespace n 'u';@scope{@media screen,???,print{}}");
    let before = report.clone();
    let [CssRule::Namespace(_), CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("namespace and scope")
    };
    let [CssScopedRule::Media(media)] = scope.rules().rules() else {
        panic!("scoped media")
    };
    // Sheet/namespace/scope/media/list/screen/recovered query cost 10 projected
    // nodes before the recovered member's two generated keyword nodes.
    let error = report
        .syntax()
        .to_specified_css_with_limits(Limits::new(usize::MAX, 11, 512))
        .unwrap_err();
    resource(&error, Kind::ProjectionNodeLimit, Some(1));
    assert_eq!(
        media_cause(&error).origin(),
        media.query().queries()[1].origin()
    );
    assert_eq!(report, before);
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        "@namespace n url(\"u\");\n@scope { @media screen, not all, print { } }"
    );
}
