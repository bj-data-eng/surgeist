#![forbid(unsafe_code)]
//! Typed authored contracts from Conditional5 sections 5.4 and 6.3; no matching.
use surgeist_css::*;

fn construct(source: &str) -> CssContainerCondition {
    CssContainerCondition::try_from_components(parse_component_values(source).unwrap()).unwrap()
}
fn scroll(condition: &CssContainerCondition) -> &CssContainerScrollQuery {
    let CssContainerConditionKind::ScrollState(query) = condition.kind() else {
        panic!("scroll query")
    };
    query
}
fn feature(query: &CssContainerScrollQuery) -> &CssContainerScrollFeature {
    let CssContainerScrollQueryKind::Feature(feature) = query.kind() else {
        panic!("feature")
    };
    feature
}
fn pending(feature: &CssContainerScrollFeature) -> &CssContainerPendingValue {
    match feature {
        CssContainerScrollFeature::Stuck(value) => {
            let CssContainerStuckValueRef::Pending(value) = value.view() else {
                panic!("pending stuck")
            };
            value
        }
        CssContainerScrollFeature::Snapped(value) => {
            let CssContainerSnappedValueRef::Pending(value) = value.view() else {
                panic!("pending snapped")
            };
            value
        }
        CssContainerScrollFeature::Scrollable(value)
        | CssContainerScrollFeature::Scrolled(value) => {
            let CssContainerScrollDirectionValueRef::Pending(value) = value.view() else {
                panic!("pending direction")
            };
            value
        }
        _ => panic!("pending feature"),
    }
}

#[test]
fn boolean_features_preserve_all_four_identities_without_invented_values() {
    for (name, expected) in [
        ("stuck", CssContainerScrollFeatureKind::Stuck),
        ("snapped", CssContainerScrollFeatureKind::Snapped),
        ("scrollable", CssContainerScrollFeatureKind::Scrollable),
        ("scrolled", CssContainerScrollFeatureKind::Scrolled),
    ] {
        let condition = construct(&format!("scroll-state({name})"));
        assert!(
            matches!(feature(scroll(&condition)), CssContainerScrollFeature::Boolean(kind) if *kind == expected)
        );
        assert_eq!(expected.name(), name);
    }
}

#[test]
fn stuck_keywords_have_the_exact_physical_and_logical_edge_domain() {
    use CssContainerStuckKeyword as K;
    for (spelling, expected) in [
        ("none", K::None),
        ("top", K::Top),
        ("right", K::Right),
        ("bottom", K::Bottom),
        ("left", K::Left),
        ("block-start", K::BlockStart),
        ("inline-start", K::InlineStart),
        ("block-end", K::BlockEnd),
        ("inline-end", K::InlineEnd),
    ] {
        let condition = construct(&format!("scroll-state(stuck:{spelling})"));
        let CssContainerScrollFeature::Stuck(value) = feature(scroll(&condition)) else {
            panic!("stuck")
        };
        assert!(
            matches!(value.view(), CssContainerStuckValueRef::Keyword(actual) if actual == expected)
        );
        assert_eq!(value.serialize().unwrap().as_css(), spelling);
        assert_eq!(expected.name(), spelling);
    }
}

#[test]
fn snapped_keywords_keep_both_and_axis_names_without_accepting_edges() {
    use CssContainerSnappedKeyword as K;
    for (spelling, expected) in [
        ("none", K::None),
        ("x", K::X),
        ("y", K::Y),
        ("block", K::Block),
        ("inline", K::Inline),
        ("both", K::Both),
    ] {
        let condition = construct(&format!("scroll-state(snapped:{spelling})"));
        let CssContainerScrollFeature::Snapped(value) = feature(scroll(&condition)) else {
            panic!("snapped")
        };
        assert!(
            matches!(value.view(), CssContainerSnappedValueRef::Keyword(actual) if actual == expected)
        );
        assert_eq!(value.serialize().unwrap().as_css(), spelling);
        assert_eq!(expected.name(), spelling);
    }
    assert!(matches!(
        construct("scroll-state(snapped:top)").kind(),
        CssContainerConditionKind::GeneralEnclosed(_)
    ));
}

#[test]
fn shared_direction_domain_retains_distinct_scrollable_and_scrolled_features() {
    use CssContainerScrollDirectionKeyword as K;
    for (spelling, expected) in [
        ("none", K::None),
        ("top", K::Top),
        ("right", K::Right),
        ("bottom", K::Bottom),
        ("left", K::Left),
        ("block-start", K::BlockStart),
        ("inline-start", K::InlineStart),
        ("block-end", K::BlockEnd),
        ("inline-end", K::InlineEnd),
        ("x", K::X),
        ("y", K::Y),
        ("block", K::Block),
        ("inline", K::Inline),
    ] {
        for name in ["scrollable", "scrolled"] {
            let condition = construct(&format!("scroll-state({name}:{spelling})"));
            let value = match (name, feature(scroll(&condition))) {
                ("scrollable", CssContainerScrollFeature::Scrollable(value))
                | ("scrolled", CssContainerScrollFeature::Scrolled(value)) => value,
                _ => panic!("distinct feature identity"),
            };
            assert!(
                matches!(value.view(), CssContainerScrollDirectionValueRef::Keyword(actual) if actual == expected)
            );
            assert_eq!(value.serialize().unwrap().as_css(), spelling);
            assert_eq!(expected.name(), spelling);
        }
    }
}

#[test]
fn pending_domains_preserve_complete_values_and_do_not_evaluate_fallbacks() {
    for (name, body, domain) in [
        ("stuck", "top var(--empty)", CssContainerValueDomain::Stuck),
        (
            "snapped",
            "var(--a) var(--b)",
            CssContainerValueDomain::Snapped,
        ),
        (
            "scrollable",
            "var(--direction,)",
            CssContainerValueDomain::ScrollDirection,
        ),
        (
            "scrolled",
            "var(--direction, banana)",
            CssContainerValueDomain::ScrollDirection,
        ),
        ("stuck", "top > var(--edge)", CssContainerValueDomain::Stuck),
    ] {
        let source = format!("scroll-state({name}:{body})");
        let condition = construct(&source);
        let pending = pending(feature(scroll(&condition)));
        assert_eq!(pending.domain(), domain);
        assert_eq!(pending.serialize().unwrap().as_css(), body);
        let first = pending.components().items().first().unwrap();
        let CssValueOrigin::Parsed(origin) = first.origin() else {
            panic!("original operand")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            source.find(':').unwrap() + 1
        );
    }
    let condition = construct("scroll-state(scrollable:var(--direction,))");
    let pending = pending(feature(scroll(&condition)));
    let CssComponentValueRef::Function(function) = pending.components().items()[0].view() else {
        panic!("var")
    };
    assert_eq!(function.values().items().len(), 2);
    assert!(matches!(
        function.values().items()[1].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Comma)
    ));
}

#[test]
fn recursive_queries_expose_inner_unknowns_without_dynamic_query_substitution() {
    let condition = construct("scroll-state(not ((stuck) and (future: top)))");
    let CssContainerScrollQueryKind::Not(group) = scroll(&condition).kind() else {
        panic!("not")
    };
    let CssContainerScrollQueryKind::Parenthesized(and) = group.kind() else {
        panic!("group")
    };
    let CssContainerScrollQueryKind::And(list) = and.kind() else {
        panic!("and")
    };
    let [known, unknown] = list.queries() else {
        panic!("two operands")
    };
    assert!(matches!(
        known.kind(),
        CssContainerScrollQueryKind::Parenthesized(_)
    ));
    let CssContainerScrollQueryKind::GeneralEnclosed(opaque) = unknown.kind() else {
        panic!("unknown")
    };
    assert_eq!(opaque.authored(), Some("(future: top)"));
    let detached = unknown.clone();
    drop(condition);
    assert_eq!(detached.serialize().unwrap().as_css(), "(future: top)");

    let condition = construct("scroll-state(var(--query))");
    let CssContainerScrollQueryKind::GeneralEnclosed(opaque) = scroll(&condition).kind() else {
        panic!("opaque function, not query substitution")
    };
    assert_eq!(opaque.authored(), Some("var(--query)"));
    assert!(matches!(
        construct("scroll-state(future: top)").kind(),
        CssContainerConditionKind::GeneralEnclosed(_)
    ));
    let condition = construct("scroll-state((stuck: both))");
    assert!(matches!(
        scroll(&condition).kind(),
        CssContainerScrollQueryKind::GeneralEnclosed(_)
    ));
}

#[test]
fn escaped_names_keywords_and_function_case_keep_original_lexical_origins() {
    let source = r"ScRoLl-StAtE(s\74 uck: T\4f P)";
    let condition = construct(source);
    let query = scroll(&condition);
    let CssContainerScrollFeature::Stuck(value) = feature(query) else {
        panic!("stuck")
    };
    assert!(matches!(
        value.view(),
        CssContainerStuckValueRef::Keyword(CssContainerStuckKeyword::Top)
    ));
    assert_eq!(value.serialize().unwrap().as_css(), r"T\4f P");
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("parsed keyword")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        value.position().unwrap().byte_offset().value(),
        source.find('T').unwrap()
    );
    assert_eq!(condition.serialize().unwrap().as_css(), source);
    let input = format!("@container {source}{{}}");
    let report = parse_sheet(&input);
    assert!(report.is_clean());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("rule")
    };
    let CssContainerScrollFeature::Stuck(parsed) =
        feature(scroll(rule.prelude().entries()[0].query().unwrap()))
    else {
        panic!("parsed stuck")
    };
    assert!(matches!(
        parsed.view(),
        CssContainerStuckValueRef::Keyword(CssContainerStuckKeyword::Top)
    ));
    assert_eq!(parsed.serialize().unwrap().as_css(), r"T\4f P");
}

#[test]
fn mixed_origin_pending_components_keep_token_boundaries_after_parent_drop() {
    let number = parse_component_values("/*😀*/1")
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone();
    let variable = CssComponentValue::try_function(
        "var",
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("--empty").unwrap()])
            .unwrap(),
    )
    .unwrap();
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("stuck").unwrap(),
        CssComponentValue::try_token(":").unwrap(),
        number,
        CssComponentValue::try_ident("e2").unwrap(),
        variable,
    ])
    .unwrap();
    let condition = CssContainerCondition::try_from_enclosed(
        CssGeneralEnclosed::try_function("scroll-state", components).unwrap(),
    )
    .unwrap();
    assert_eq!(condition.origin(), &CssValueOrigin::Programmatic);
    let leaf = pending(feature(scroll(&condition))).clone();
    drop(condition);
    assert_eq!(leaf.domain(), CssContainerValueDomain::Stuck);
    assert_eq!(leaf.components().items().len(), 3);
    assert!(matches!(
        leaf.components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(
        leaf.components().items()[1].origin(),
        &CssValueOrigin::Programmatic
    );
    assert_eq!(
        leaf.serialize().unwrap().as_css(),
        "1/**/e2/**/var(--empty)"
    );
}

#[test]
fn limits_and_recovered_closures_keep_typed_resource_and_source_information() {
    let source = "scroll-state((stuck:var(--edge,)))";
    let values = parse_component_values(source).unwrap();
    let exact = CssComponentValueLimits::try_new(
        values.nesting_depth(),
        values.component_count(),
        source.len(),
    )
    .unwrap();
    assert!(CssContainerCondition::try_from_components_with_limits(values.clone(), exact).is_ok());
    for (limits, expected) in [
        (
            CssComponentValueLimits::try_new(1, usize::MAX, usize::MAX).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, 1, usize::MAX).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, usize::MAX, 1).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let Err(CssContainerConstructionError::Component(error)) =
            CssContainerCondition::try_from_components_with_limits(values.clone(), limits)
        else {
            panic!("resource failure")
        };
        assert_eq!(error.kind(), expected);
    }
    let recovered = "scroll-state(stuck:var(--edge,";
    let condition = construct(recovered);
    let pending = pending(feature(scroll(&condition)));
    let output = pending.serialize().unwrap();
    assert_eq!(output.as_css(), "var(--edge,)");
    assert!(matches!(
        output.origin_at(output.as_css().len() - 1),
        Some(CssSerializedOrigin::Token(
            CssValueOrigin::ImplicitClosure { .. }
        ))
    ));
    assert_eq!(
        pending.serialize_with_limit(1).unwrap_err().kind(),
        CssComponentValueErrorKind::ByteLimit
    );
    assert!(matches!(
        parse_component_values(&format!(
            "scroll-state({}stuck{})",
            "(".repeat(256),
            ")".repeat(256)
        ))
        .unwrap_err()
        .kind(),
        CssComponentValueErrorKind::NestingLimit
    ));
}

#[test]
fn maximum_scroll_depth_and_deep_pending_lifetimes_fit_an_ordinary_stack() {
    const CHILD: &str = "SURGEIST_SCROLL_LIFECYCLE_CHILD";
    const TEST: &str = "maximum_scroll_depth_and_deep_pending_lifetimes_fit_an_ordinary_stack";
    if std::env::var(CHILD).as_deref() == Ok(TEST) {
        let worker = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                let grouped = format!("{}stuck{}", "(".repeat(255), ")".repeat(255));
                let negated = format!("{}stuck{}", "not (".repeat(255), ")".repeat(255));
                let mut conjunction = "(stuck) and future()".to_owned();
                for _ in 0..254 {
                    conjunction = format!("({conjunction}) and future()");
                }
                let opaque = format!("{}1{}", "future(".repeat(255), ")".repeat(255));
                for inner in [grouped, negated, conjunction, opaque] {
                    let source = format!("scroll-state({inner})");
                    let condition = construct(&source);
                    let detached = scroll(&condition).clone();
                    drop(condition);
                    let cloned = detached.clone();
                    assert_eq!(detached, cloned);
                    assert_eq!(cloned.serialize().unwrap().as_css(), inner);
                    drop(detached);
                    drop(cloned);
                }
                let body = format!("{}{}", "var(--edge,".repeat(255), ")".repeat(255));
                let condition = construct(&format!("scroll-state(stuck:{body})"));
                let leaf = pending(feature(scroll(&condition))).clone();
                let detached = scroll(&condition).clone();
                drop(condition);
                let cloned = detached.clone();
                assert_eq!(detached, cloned);
                drop(detached);
                drop(cloned);
                let cloned_leaf = leaf.clone();
                assert_eq!(leaf, cloned_leaf);
                drop(leaf);
                assert_eq!(cloned_leaf.domain(), CssContainerValueDomain::Stuck);
                assert_eq!(cloned_leaf.serialize().unwrap().as_css(), body);
                drop(cloned_leaf);
            })
            .unwrap();
        if let Err(panic) = worker.join() {
            std::panic::resume_unwind(panic);
        }
        println!("scroll lifecycle completed");
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
        .env(CHILD, TEST)
        .env_remove("RUST_MIN_STACK")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("scroll lifecycle completed"));
}
