#![forbid(unsafe_code)]
//! New checked construction uses pinned Conditional Rules 5 sections 5.4 and
//! 9.1: grouping and operators are retained without boolean simplification.
use surgeist_css::*;

fn construct(source: &str) -> CssContainerCondition {
    CssContainerCondition::try_from_components(parse_component_values(source).unwrap()).unwrap()
}
fn token(source: &str) -> CssComponentValue {
    CssComponentValue::try_token(source).unwrap()
}
fn values(items: Vec<CssComponentValue>) -> CssComponentValues {
    CssComponentValues::try_new(items).unwrap()
}
fn width() -> CssComponentValue {
    CssComponentValue::try_block(
        CssBlockKind::Parenthesis,
        values(vec![token("width"), token(":"), token("1px")]),
    )
    .unwrap()
}
fn programmatic(items: Vec<CssComponentValue>) -> CssContainerCondition {
    CssContainerCondition::try_from_components(values(items)).unwrap()
}

#[test]
fn every_authored_group_operator_and_leaf_has_an_inspectable_region() {
    let source = "NoT ( ((WIDTH > 01.00px)) aNd style(--Theme: r\\65 d) )";
    let condition = construct(source);
    assert_eq!(condition.serialize().unwrap().as_css(), source);
    assert_eq!(condition.position().unwrap().byte_offset().value(), 0);
    let CssContainerConditionKind::Not(outer) = condition.kind() else {
        panic!("not")
    };
    assert_eq!(
        outer.serialize().unwrap().as_css(),
        "( ((WIDTH > 01.00px)) aNd style(--Theme: r\\65 d) )"
    );
    let CssContainerConditionKind::Parenthesized(and) = outer.kind() else {
        panic!("outer grouping")
    };
    let CssContainerConditionKind::And(list) = and.kind() else {
        panic!("and")
    };
    let [width_group, style] = list.conditions() else {
        panic!("two operands")
    };
    let CssContainerConditionKind::Parenthesized(width) = width_group.kind() else {
        panic!("redundant width grouping")
    };
    let CssContainerConditionKind::Feature(CssContainerFeatureQuery::Width(range)) = width.kind()
    else {
        panic!("width")
    };
    assert_eq!(range.value().value().value(), 1.0);
    assert_eq!(width.serialize().unwrap().as_css(), "(WIDTH > 01.00px)");
    assert_eq!(
        style.serialize().unwrap().as_css(),
        "style(--Theme: r\\65 d)"
    );
    let CssContainerConditionKind::Style(CssContainerStyleQuery::CustomPropertyValue {
        name,
        value,
    }) = style.kind()
    else {
        panic!("style")
    };
    assert_eq!(name.as_str(), "--Theme");
    assert_eq!(value.as_css(), "r\\65 d");
    for child in [
        outer.as_ref(),
        and.as_ref(),
        width_group,
        width.as_ref(),
        style,
    ] {
        let CssValueOrigin::Parsed(origin) = child.origin() else {
            panic!("original token")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(child.position(), Some(origin.span().start()));
        assert!(!child.components().is_empty());
    }
}

#[test]
fn checked_programmatic_boolean_grammar_cannot_construct_ungrouped_mixtures() {
    let condition = programmatic(vec![token("not"), token(" "), width()]);
    assert!(matches!(
        condition.kind(),
        CssContainerConditionKind::Not(_)
    ));
    assert_eq!(condition.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(condition.position(), None);
    assert_eq!(condition.serialize().unwrap().as_css(), "not (width:1px)");
    for invalid in [
        vec![],
        vec![token("not")],
        vec![token("width")],
        vec![token("not"), token(" "), token("not"), token(" "), width()],
        vec![
            width(),
            token(" "),
            token("and"),
            token(" "),
            token("not"),
            token(" "),
            width(),
        ],
        vec![
            width(),
            token(" "),
            token("and"),
            token(" "),
            width(),
            token(" "),
            token("or"),
            token(" "),
            width(),
        ],
    ] {
        assert!(matches!(
            CssContainerCondition::try_from_components(values(invalid)),
            Err(CssContainerConstructionError::InvalidConditionGrammar {
                origin: CssValueOrigin::Programmatic
            })
        ));
    }
}

#[test]
fn child_equality_and_lifetimes_do_not_depend_on_unrelated_siblings() {
    fn parent(sibling: &str) -> CssContainerCondition {
        programmatic(vec![
            width(),
            token(" "),
            token("and"),
            token(" "),
            CssComponentValue::try_function(sibling, values(vec![])).unwrap(),
        ])
    }
    let left = parent("Future");
    let right = parent("Other");
    assert_ne!(left, right);
    let CssContainerConditionKind::And(left_children) = left.kind() else {
        panic!("and")
    };
    let CssContainerConditionKind::And(right_children) = right.kind() else {
        panic!("and")
    };
    let left_child = left_children.conditions()[0].clone();
    let right_child = right_children.conditions()[0].clone();
    assert_eq!(left_child, right_child);
    drop(left);
    drop(right);
    assert_eq!(left_child.serialize().unwrap().as_css(), "(width:1px)");
    assert_eq!(right_child.serialize().unwrap().as_css(), "(width:1px)");
    assert_eq!(left_child, right_child);
}

#[test]
fn original_token_boundaries_survive_recognized_style_serialization() {
    let numeric = parse_component_values("/*😀*/1")
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone();
    let expected_origin = numeric.origin().clone();
    let function = CssComponentValue::try_function(
        "style",
        values(vec![token("--Theme"), token(":"), numeric, token("e2")]),
    )
    .unwrap();
    let condition = programmatic(vec![function]);
    assert!(matches!(
        condition.kind(),
        CssContainerConditionKind::Style(_)
    ));
    let serialized = condition.serialize().unwrap();
    assert_eq!(serialized.as_css(), "style(--Theme:1/**/e2)");
    assert_eq!(
        serialized.origin_at(14),
        Some(&CssSerializedOrigin::Token(expected_origin))
    );
    assert!(matches!(
        serialized.origin_at(15),
        Some(CssSerializedOrigin::Separator { .. })
    ));
}

#[test]
fn checked_resource_limits_apply_before_recognized_and_opaque_classification() {
    for source in ["(width:1px)", "style(--x)", "Future()", "((width:1px))"] {
        let components = parse_component_values(source).unwrap();
        let exact = CssComponentValueLimits::try_new(
            components.nesting_depth(),
            components.component_count(),
            source.len(),
        )
        .unwrap();
        assert_eq!(
            CssContainerCondition::try_from_components_with_limits(components.clone(), exact)
                .unwrap()
                .serialize()
                .unwrap()
                .as_css(),
            source
        );
        for (limits, expected) in [
            (
                CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap(),
                CssComponentValueErrorKind::NestingLimit,
            ),
            (
                CssComponentValueLimits::try_new(256, 0, usize::MAX).unwrap(),
                CssComponentValueErrorKind::ComponentLimit,
            ),
            (
                CssComponentValueLimits::try_new(256, usize::MAX, source.len() - 1).unwrap(),
                CssComponentValueErrorKind::ByteLimit,
            ),
        ] {
            let Err(CssContainerConstructionError::Component(error)) =
                CssContainerCondition::try_from_components_with_limits(components.clone(), limits)
            else {
                panic!("typed component limit")
            };
            assert_eq!(error.kind(), expected);
            assert!(matches!(error.origin(), CssValueOrigin::Parsed(_)));
        }
        let condition = construct(source);
        assert_eq!(
            condition
                .serialize_with_limit(source.len())
                .unwrap()
                .as_css(),
            source
        );
        assert_eq!(
            condition
                .serialize_with_limit(source.len() - 1)
                .unwrap_err()
                .kind(),
            CssComponentValueErrorKind::ByteLimit
        );
    }
}

#[test]
fn trusted_enclosure_recovery_preserves_recognized_grouping_and_closure_origins() {
    let components = parse_component_values("((width:1px").unwrap();
    let from_components = CssContainerCondition::try_from_components(components.clone()).unwrap();
    let condition = CssContainerCondition::try_from_enclosed(
        CssGeneralEnclosed::try_from_component(components.items()[0].clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(condition, from_components);
    assert_eq!(
        from_components.serialize().unwrap().as_css(),
        "((width:1px))"
    );
    let CssContainerConditionKind::Parenthesized(child) = condition.kind() else {
        panic!("retained redundant group")
    };
    assert!(matches!(
        child.kind(),
        CssContainerConditionKind::Feature(_)
    ));
    let serialized = condition.serialize().unwrap();
    assert_eq!(serialized.as_css(), "((width:1px))");
    for offset in [11, 12] {
        assert!(matches!(
            serialized.origin_at(offset),
            Some(CssSerializedOrigin::Token(
                CssValueOrigin::ImplicitClosure { .. }
            ))
        ));
    }
}

#[test]
fn supported_container_depth_has_safe_owned_tree_lifetimes_on_an_ordinary_stack() {
    const CHILD: &str = "SURGEIST_CONTAINER_CONSTRUCTION_STACK_CHILD";
    const TEST: &str =
        "supported_container_depth_has_safe_owned_tree_lifetimes_on_an_ordinary_stack";
    if std::env::var(CHILD).as_deref() == Ok(TEST) {
        let worker = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                let grouped = format!("{}width:1px{}", "(".repeat(256), ")".repeat(256));
                let negated = format!("{}(width:1px){}", "not (".repeat(255), ")".repeat(255));
                let opaque = format!("{}1{}", "Future(".repeat(256), ")".repeat(256));
                let mut conjunction = "(width:1px)".to_owned();
                for _ in 0..255 {
                    conjunction = format!("({conjunction} and Future())");
                }
                for source in [grouped, negated, opaque, conjunction] {
                    let condition = construct(&source);
                    let cloned = condition.clone();
                    assert_eq!(condition, cloned);
                    assert_eq!(condition.serialize().unwrap().as_css(), source);
                    drop(condition);
                    assert_eq!(cloned.serialize().unwrap().as_css(), source);
                    drop(cloned);
                }
                let source = format!("{}width:1px{}", "(".repeat(255), ")".repeat(255));
                let parent = construct(&format!("{source} and Future()"));
                let CssContainerConditionKind::And(list) = parent.kind() else {
                    panic!("and")
                };
                let child = list.conditions()[0].clone();
                drop(parent);
                assert_eq!(child.serialize().unwrap().as_css(), source);
                let cloned = child.clone();
                assert_eq!(child, cloned);
                drop(child);
                drop(cloned);
            })
            .unwrap();
        if let Err(panic) = worker.join() {
            std::panic::resume_unwind(panic);
        }
        println!("container construction and owned lifecycle completed");
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
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("container construction and owned lifecycle completed")
    );
}
