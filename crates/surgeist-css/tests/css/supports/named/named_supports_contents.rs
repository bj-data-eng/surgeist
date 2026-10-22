//! Public consumer tests for the selected Surgeist #644 inert-child policy.
use crate::css::support::authored_syntax::{assert_diagnostics, assert_strict_rejection};
use surgeist_css::{
    CssComponentValueErrorKind, CssComponentValueLimits, CssEditedGroupPreludeRef, CssErrorCode,
    CssNamedSupportsConstructionError, CssNamespaceContext, CssRecoveryAction, CssRule,
    CssRuleCssomFormat, CssRuleCssomSerializationErrorKind, CssRuleGraphInput,
    CssRuleGraphSerializationLimits, CssSpecifiedValueSerializationErrorKind as Resource,
    CssSpecifiedValueSerializationLimits, CssStyleAncestor, CssSupportsConditionName,
    CssSupportsTestChild as Child, CssSupportsTestChildRef as Ref, CssSupportsTestItem,
    CssValueOrigin, ErrorKind, parse_component_values, parse_media_query_list, parse_sheet,
    parse_supports_test_child, parse_supports_test_child_with_limits, parse_supports_test_contents,
    parse_supports_test_contents_with_limits,
};

fn child(source: &str) -> Child {
    let report = parse_supports_test_child(source);
    assert!(report.is_clean(), "{source:?}: {:?}", report.diagnostics());
    report.into_parts().0.expect("one admitted child")
}
fn limits(depth: usize) -> CssRuleGraphSerializationLimits {
    CssRuleGraphSerializationLimits {
        values: CssSpecifiedValueSerializationLimits::default(),
        max_depth: depth,
    }
}
fn graph(
    input: CssRuleGraphInput<'_>,
    limits: CssRuleGraphSerializationLimits,
) -> Result<String, surgeist_css::CssRuleCssomSerializationError> {
    input.serialize_cssom_with_graph_limits(
        &CssNamespaceContext::default(),
        CssStyleAncestor::Absent,
        limits,
    )
}

#[test]
fn raw_projection_keeps_order_duplicates_importance_and_genuine_dropped_source() {
    let source = String::from(
        "/*🦊*/; Future:red; Future: blue !IMPORTANT; & { ; } @namespace n url(x); @future x;",
    );
    let report = parse_supports_test_contents(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let body = report.into_parts().0.unwrap();
    let expected_source = source.clone();
    drop(source);
    let children: Vec<_> = body.children().collect();
    assert_eq!(children.len(), 5);
    let [
        Ref::Declaration(first),
        Ref::Declaration(second),
        Ref::QualifiedRule(qualified),
        Ref::AtRule(namespace),
        Ref::AtRule(future),
    ] = children.as_slice()
    else {
        panic!("inert direct projection")
    };
    assert_eq!(first.property(), "Future");
    assert_eq!(second.property(), "Future");
    assert!(qualified.body().items().is_empty());
    assert_eq!(namespace.name(), "namespace");
    assert_eq!(future.name(), "future");
    let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) = (first.origin(), second.origin())
    else {
        panic!("actual source occurrences")
    };
    assert_eq!(a.source().as_str(), expected_source);
    assert!(a.source().same_snapshot(b.source()));
    assert_eq!(
        a.span().start().byte_offset().value(),
        expected_source.find("Future:red").unwrap()
    );
    assert_eq!(
        body.to_specified_css().unwrap(),
        "Future: red; Future: blue !important; & { } @namespace n url(x); @future x;"
    );
}

#[test]
fn strict_child_admits_recursive_empty_semicolons_and_nonrendering_candidates() {
    for (input, expected) in [
        (";color:red;", "color: red;"),
        ("color:red;;", "color: red;"),
        ("; & { ; @future { ;; } ; };", "& { @future  { } }"),
        ("@future", "@future;"),
        ("future:;", "future: ;"),
        (
            r"f\75 ture: 1/**/e3 !important;",
            "future: 1/**/e3 !important;",
        ),
        ("{ color:red }", "{ color: red; }"),
        ("@font-face { future:yes; }", "@font-face  { future: yes; }"),
        ("@future   {}", "@future  { }"),
    ] {
        assert_eq!(
            child(input).as_ref().to_specified_css().unwrap(),
            expected,
            "{input}"
        );
    }
    // Input without any child component holds no child at all.
    for input in ["", ";;;"] {
        let empty = parse_supports_test_child(input);
        assert!(empty.syntax().is_none(), "{input:?}: {empty:?}");
        let [diagnostic] = empty.diagnostics() else {
            panic!("one empty-input rejection: {input:?}: {empty:?}")
        };
        let ErrorKind::InvalidComponentValue(error) = diagnostic.error().kind() else {
            panic!("empty input: {input:?}: {diagnostic:?}")
        };
        assert_eq!(error.kind(), CssComponentValueErrorKind::EmptyInput);
        assert_strict_rejection(empty.clone().into_validation_result(), empty.diagnostics());
    }
    for input in [
        "color:red;opacity:.5",
        "@future; @other",
        "& { bad:url(a b); }",
        "& { color:red",
        "color:red } @future;",
        "&{} junk",
    ] {
        let report = parse_supports_test_child(input);
        assert!(report.syntax().is_none(), "{input:?}: {report:?}");
        assert!(!report.is_clean(), "rejection carries diagnostic: {input}");
        assert_strict_rejection(
            report.clone().into_validation_result(),
            report.diagnostics(),
        );
    }
}

#[test]
fn raw_empty_and_recovered_empty_are_distinct_from_typed_preparation_failure() {
    for input in ["", ";;;", " /*x*/ "] {
        let report = parse_supports_test_contents(input);
        assert!(report.is_clean());
        assert!(report.syntax().as_ref().unwrap().items().is_empty());
    }
    let recovered = parse_supports_test_contents("bad:url(a b);");
    // The bad URL token makes the declaration value invalid.
    assert_diagnostics(
        recovered.diagnostics(),
        &[(
            CssErrorCode::InvalidComponentValue,
            CssRecoveryAction::DropDeclaration,
            "bad:".len(),
        )],
    );
    assert!(recovered.syntax().as_ref().unwrap().items().is_empty());
    assert_eq!(
        recovered
            .syntax()
            .as_ref()
            .unwrap()
            .local_recovery_origins()
            .len(),
        1
    );
    for (bound, kind) in [
        (
            CssComponentValueLimits::try_new(256, 0, 100).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, 100, 1).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
        (
            CssComponentValueLimits::try_new(0, 100, 100).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
    ] {
        let report = parse_supports_test_contents_with_limits("& { color:red; }", bound);
        assert!(report.syntax().is_none());
        assert!(report.diagnostics().iter().any(|d| matches!(d.error().kind(), ErrorKind::InvalidComponentValue(e) if e.kind() == kind)), "{report:?}");
        let one = parse_supports_test_child_with_limits("& { color:red; }", bound);
        assert!(one.syntax().is_none());
        assert_eq!(one.diagnostics(), report.diagnostics());
    }
    assert!(parse_supports_test_contents("& { color:red; }").is_clean());
}

#[test]
fn raw_resource_budget_charges_discarded_units_and_trailing_input() {
    let input = "bad:url(a b); color:red; } @future { a:b;c:d;e:f; }";
    let report = parse_supports_test_contents_with_limits(
        input,
        CssComponentValueLimits::try_new(256, 10, 1000).unwrap(),
    );
    assert!(
        report.syntax().is_none(),
        "no retained prefix after whole input budget failure"
    );
    assert!(
        matches!(report.diagnostics()[0].error().kind(), ErrorKind::InvalidComponentValue(e) if e.kind() == CssComponentValueErrorKind::ComponentLimit)
    );
    let retry = parse_supports_test_contents(input);
    assert!(retry.syntax().is_some());
    assert!(!retry.is_clean());
}

#[test]
fn composition_coalesces_runs_keeps_occurrences_and_recomputes_descendants() {
    let report = parse_supports_test_contents("& { bad:url(a b); } outer:url(c d);");
    let history = report.diagnostics().to_vec();
    let body = report.syntax().as_ref().unwrap();
    assert_eq!(body.local_recovery_origins().len(), 1);
    assert_eq!(body.recovery_origins().count(), 2);
    let first = child("future:1;");
    let duplicate = child("future:2 !important;");
    let kept: Vec<_> = body.children().collect();
    let composed = body
        .recompose_children(&[first.as_ref(), duplicate.as_ref(), kept[0], first.as_ref()])
        .unwrap();
    let [
        CssSupportsTestItem::Declarations(run),
        CssSupportsTestItem::QualifiedRule(_),
        CssSupportsTestItem::Declarations(after),
    ] = composed.items()
    else {
        panic!("maximal runs")
    };
    assert_eq!(run.declarations().len(), 2);
    assert_eq!(after.declarations().len(), 1);
    assert_eq!(run.declarations()[0].origin(), first.as_ref().origin());
    assert_eq!(composed.recovery_origins().count(), 2);
    let removed = composed.recompose_children(&[]).unwrap();
    assert!(removed.items().is_empty());
    assert_eq!(
        removed.local_recovery_origins(),
        body.local_recovery_origins()
    );
    assert_eq!(removed.recovery_origins().count(), 1);
    assert_eq!(removed.recovery_origin(), body.recovery_origin());
    assert_eq!(report.diagnostics(), history);
    assert_eq!(body.recovery_origins().count(), 2);
}

#[test]
fn deleting_only_recovered_child_clears_current_union_not_historical_report() {
    let report = parse_supports_test_contents("& { bad:url(a b); }");
    let body = report.syntax().as_ref().unwrap();
    assert!(body.local_recovery_origins().is_empty());
    assert!(body.recovery_origin().is_some());
    let removed = body.recompose_children(&[]).unwrap();
    assert!(removed.recovery_origin().is_none());
    assert!(removed.recovery_origins().next().is_none());
    assert_diagnostics(
        report.diagnostics(),
        &[(
            CssErrorCode::InvalidComponentValue,
            CssRecoveryAction::DropDeclaration,
            "& { bad:".len(),
        )],
    );
}

#[test]
fn retained_component_closure_belongs_to_declaration_and_dropped_candidate_to_body() {
    let report = parse_supports_test_contents("color:fn(a");
    let body = report.syntax().as_ref().unwrap();
    assert!(body.local_recovery_origins().is_empty());
    assert!(body.recovery_origin().is_some());
    assert!(matches!(body.children().next(), Some(Ref::Declaration(_))));
    assert!(
        body.recompose_children(&[])
            .unwrap()
            .recovery_origin()
            .is_none()
    );
    let statement = parse_supports_test_contents("@future fn(a");
    let statement_body = statement.syntax().as_ref().unwrap();
    assert!(statement_body.local_recovery_origins().is_empty());
    assert!(statement_body.recovery_origin().is_some());
    assert!(
        statement_body
            .recompose_children(&[])
            .unwrap()
            .recovery_origin()
            .is_none()
    );
    let dropped = parse_supports_test_contents("url(a b) { inside:url(c d); } good:yes;");
    let body = dropped.syntax().as_ref().unwrap();
    assert_eq!(body.local_recovery_origins().len(), 2);
    assert_eq!(body.children().count(), 1);
    let removed = body.recompose_children(&[]).unwrap();
    assert_eq!(
        removed.recovery_origins().collect::<Vec<_>>(),
        body.local_recovery_origins().iter().collect::<Vec<_>>()
    );
}

#[test]
fn borrowed_composition_shares_one_component_byte_depth_budget_and_retries() {
    let empty = parse_supports_test_contents("").into_parts().0.unwrap();
    let a = child("a:one;");
    let b = child("b:two;");
    let refs = [a.as_ref(), b.as_ref()];
    let before = (a.clone(), b.clone());
    for (budget, exceeded) in [
        (
            CssComponentValueLimits::try_new(256, 7, 100).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, 100, 8).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            empty
                .recompose_children_with_limits(&[refs[0]], budget)
                .unwrap(),
            empty.recompose_children(&[refs[0]]).unwrap()
        );
        let Err(CssNamedSupportsConstructionError::Component(error)) =
            empty.recompose_children_with_limits(&refs, budget)
        else {
            panic!("{budget:?}")
        };
        assert_eq!(error.kind(), exceeded);
    }
    let nested = child("& { a:fn(x); }");
    let Err(CssNamedSupportsConstructionError::Component(error)) = empty
        .recompose_children_with_limits(
            &[nested.as_ref()],
            CssComponentValueLimits::try_new(1, 100, 100).unwrap(),
        )
    else {
        panic!("nested child exceeds one nesting level")
    };
    assert_eq!(error.kind(), CssComponentValueErrorKind::NestingLimit);
    assert_eq!(
        empty
            .recompose_children(&[nested.as_ref()])
            .unwrap()
            .children()
            .count(),
        1
    );
    assert_eq!((a.clone(), b.clone()), before);
    assert_eq!(
        empty.recompose_children(&refs).unwrap().children().count(),
        2
    );
}

#[test]
fn current_named_graph_opt_in_keeps_standard_unavailable_and_original_unchanged() {
    let report = parse_sheet("@supports-condition --feature { old:no; }");
    let CssRule::SupportsCondition(original) = &report.syntax().rules()[0] else {
        panic!()
    };
    assert!(matches!(
        report.syntax().rules()[0]
            .serialize_cssom()
            .unwrap_err()
            .kind(),
        CssRuleCssomSerializationErrorKind::FormatUnavailable(
            CssRuleCssomFormat::SupportsCondition
        )
    ));
    let replacement = child("new:yes;");
    let current = original
        .body()
        .recompose_children(&[replacement.as_ref()])
        .unwrap();
    assert_eq!(
        graph(
            CssRuleGraphInput::named_supports(original.name(), &current),
            limits(1)
        )
        .unwrap(),
        "@supports-condition --feature { new: yes; }"
    );
    assert_eq!(
        original.to_specified_css().unwrap(),
        "@supports-condition --feature { old: no; }"
    );
    assert_eq!(
        graph(
            CssRuleGraphInput::supports_test_child(replacement.as_ref()),
            limits(0)
        )
        .unwrap(),
        "new: yes;"
    );
}

#[test]
fn graph_zero_exact_and_under_depth_include_opaque_candidates_and_components() {
    let name = CssSupportsConditionName::try_new("--x").unwrap();
    let empty = parse_supports_test_contents("").into_parts().0.unwrap();
    assert_eq!(
        graph(CssRuleGraphInput::named_supports(&name, &empty), limits(0)).unwrap(),
        "@supports-condition --x { }"
    );
    let body = parse_supports_test_contents("& { color:fn(x); }")
        .into_parts()
        .0
        .unwrap();
    let selected = CssRuleGraphInput::named_supports(&name, &body);
    assert_eq!(
        graph(selected, limits(3)).unwrap(),
        graph(selected, limits(usize::MAX)).unwrap()
    );
    for depth in [0, 1, 2] {
        assert_eq!(
            graph(selected, limits(depth)).unwrap_err().kind(),
            CssRuleCssomSerializationErrorKind::Resource(Resource::DepthLimit)
        );
    }
    assert_eq!(
        body.to_specified_css_with_graph_limits(limits(2)).unwrap(),
        body.to_specified_css_with_graph_limits(limits(usize::MAX))
            .unwrap()
    );
    assert_eq!(
        body.to_specified_css_with_graph_limits(limits(1))
            .unwrap_err()
            .kind(),
        Resource::DepthLimit
    );
    let no_children = child("&{}");
    assert_eq!(
        no_children
            .as_ref()
            .to_specified_css_with_graph_limits(limits(0))
            .unwrap(),
        no_children
            .as_ref()
            .to_specified_css_with_graph_limits(limits(usize::MAX))
            .unwrap()
    );
    let empty_function = child("color:fn();");
    assert_eq!(
        empty_function
            .as_ref()
            .to_specified_css_with_graph_limits(limits(0))
            .unwrap_err()
            .kind(),
        Resource::DepthLimit
    );
}

#[test]
fn ordinary_ancestor_and_current_named_payload_share_depth_and_output_budget() {
    let queries = parse_media_query_list("screen").into_parts().0;
    let name = CssSupportsConditionName::try_from_component(
        parse_component_values(r"--f\65 ature").unwrap().items()[0].clone(),
    )
    .unwrap();
    let body = parse_supports_test_contents("& { color:fn(x); }")
        .into_parts()
        .0
        .unwrap();
    let children = [CssRuleGraphInput::named_supports(&name, &body)];
    let root = CssRuleGraphInput::group(CssEditedGroupPreludeRef::Media(&queries), &children);
    let expected = "@media screen {\n  @supports-condition --feature { & { color: fn(x); } }\n}";
    assert_eq!(graph(root, limits(4)).unwrap(), expected);
    let error = graph(root, limits(3)).unwrap_err();
    assert_eq!(
        error.kind(),
        CssRuleCssomSerializationErrorKind::Resource(Resource::DepthLimit)
    );
    assert_eq!(error.rule_path(), &[0]);
    let cause = std::error::Error::source(&error)
        .unwrap()
        .downcast_ref::<surgeist_css::CssSpecifiedValueSerializationError>()
        .unwrap();
    let CssValueOrigin::Parsed(origin) = cause.origin().expect("actual opaque component origin")
    else {
        panic!()
    };
    assert_eq!(origin.source().as_str(), "& { color:fn(x); }");
    assert_eq!(origin.span().start().byte_offset().value(), 10);
    let mut bounded = limits(4);
    bounded.values = CssSpecifiedValueSerializationLimits::new(1000, 1000, expected.len() - 1);
    assert_eq!(
        graph(root, bounded).unwrap_err().kind(),
        CssRuleCssomSerializationErrorKind::Resource(Resource::ByteLimit)
    );
    assert_eq!(graph(root, limits(4)).unwrap(), expected);
}

#[test]
fn whole_named_graph_node_and_projection_work_is_not_reset_per_child() {
    let name = CssSupportsConditionName::try_new("--x").unwrap();
    let body = parse_supports_test_contents("a:one;b:two;c:three;")
        .into_parts()
        .0
        .unwrap();
    let input = CssRuleGraphInput::named_supports(&name, &body);
    let children: Vec<_> = body.children().collect();
    for (input_quota, expected_kind) in [
        (true, Resource::InputNodeLimit),
        (false, Resource::ProjectionNodeLimit),
    ] {
        let found = (1..100).find(|n| {
            let l = CssRuleGraphSerializationLimits { values: CssSpecifiedValueSerializationLimits::new(if input_quota { *n } else { 1000 },if input_quota { 1000 } else { *n },1000), max_depth:10 };
            children.iter().all(|child| graph(CssRuleGraphInput::supports_test_child(*child), l).is_ok()) && matches!(graph(input,l),Err(e) if e.kind() == CssRuleCssomSerializationErrorKind::Resource(expected_kind))
        });
        assert!(found.is_some());
    }
    assert_eq!(
        graph(input, limits(1)).unwrap(),
        graph(input, limits(usize::MAX)).unwrap()
    );
}

#[test]
fn native_component_work_is_admitted_before_depth_inspection() {
    let leaf = child("color:fn(x);");
    let bound = CssRuleGraphSerializationLimits {
        values: CssSpecifiedValueSerializationLimits::new(2, 100, 100),
        max_depth: 0,
    };
    let error = graph(CssRuleGraphInput::supports_test_child(leaf.as_ref()), bound).unwrap_err();
    assert_eq!(
        error.kind(),
        CssRuleCssomSerializationErrorKind::Resource(Resource::InputNodeLimit)
    );
    assert_eq!(
        graph(
            CssRuleGraphInput::supports_test_child(leaf.as_ref()),
            limits(0)
        )
        .unwrap_err()
        .kind(),
        CssRuleCssomSerializationErrorKind::Resource(Resource::DepthLimit)
    );
}
