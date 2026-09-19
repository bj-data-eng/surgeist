#![forbid(unsafe_code)]
//! Conditional5 style-query structure, lexical ownership and checked boundaries.
use surgeist_css::*;

fn construct(source: &str) -> CssContainerCondition {
    CssContainerCondition::try_from_components(parse_component_values(source).unwrap()).unwrap()
}
fn style(condition: &CssContainerCondition) -> &CssContainerStyleQuery {
    let CssContainerConditionKind::Style(query) = condition.kind() else {
        panic!("style")
    };
    query
}
fn plain(
    query: &CssContainerStyleQuery,
) -> (&CssContainerStyleFeatureName, &CssContainerStyleValue) {
    let CssContainerStyleQueryKind::Feature(CssContainerStyleFeature::Plain { name, value }) =
        query.kind()
    else {
        panic!("plain")
    };
    (name, value)
}
fn range(query: &CssContainerStyleQuery) -> &CssContainerStyleRange {
    let CssContainerStyleQueryKind::Feature(CssContainerStyleFeature::Range(range)) = query.kind()
    else {
        panic!("range")
    };
    range
}
fn reference(operand: &CssContainerStyleRangeOperand) -> &str {
    let CssContainerStyleRangeOperandRef::CustomProperty(name) = operand.view() else {
        panic!("bare custom reference")
    };
    name.as_str()
}

#[test]
fn inner_logic_and_unknowns_expose_independent_original_regions() {
    let source = "style(NoT (((--Theme: dark) aNd future(1))))";
    let condition = construct(source);
    let query = style(&condition);
    let CssContainerStyleQueryKind::Not(group) = query.kind() else {
        panic!("not")
    };
    let CssContainerStyleQueryKind::Parenthesized(group) = group.kind() else {
        panic!("group")
    };
    let CssContainerStyleQueryKind::Parenthesized(and) = group.kind() else {
        panic!("redundant group")
    };
    let CssContainerStyleQueryKind::And(list) = and.kind() else {
        panic!("and")
    };
    let [grouped_feature, unknown] = list.queries() else {
        panic!("two operands")
    };
    let CssContainerStyleQueryKind::Parenthesized(feature) = grouped_feature.kind() else {
        panic!("feature group")
    };
    let (CssContainerStyleFeatureName::Custom(name), value) = plain(feature) else {
        panic!("custom name")
    };
    assert_eq!(name.as_str(), "--Theme");
    assert_eq!(value.serialize().unwrap().as_css(), " dark");
    let CssContainerStyleQueryKind::GeneralEnclosed(opaque) = unknown.kind() else {
        panic!("opaque")
    };
    assert_eq!(opaque.authored(), Some("future(1)"));
    for node in [query, and, grouped_feature, feature, unknown] {
        let CssValueOrigin::Parsed(origin) = node.origin() else {
            panic!("parsed origin")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(node.position(), Some(origin.span().start()));
        assert!(!node.components().is_empty());
    }
    let detached = feature.as_ref().clone();
    drop(condition);
    assert_eq!(detached.serialize().unwrap().as_css(), "--Theme: dark");
    assert_eq!(plain(&detached).1.serialize().unwrap().as_css(), " dark");
}

#[test]
fn ordinary_name_keeps_its_grammar_while_plain_custom_names_remain_literal_values() {
    let condition = construct("style(GLYPH-ORIENTATION-VERTICAL: --Angle)");
    let (CssContainerStyleFeatureName::Property(grammar), value) = plain(style(&condition)) else {
        panic!("property grammar")
    };
    assert_eq!(
        *grammar,
        CssPropertyGrammar::from_name("glyph-orientation-vertical").unwrap()
    );
    assert_eq!(grammar.name(), "glyph-orientation-vertical");
    assert_eq!(grammar.target_property(), CssKnownProperty::TextOrientation);
    assert_eq!(value.serialize().unwrap().as_css(), " --Angle");
    assert!(matches!(
        value.components().last().unwrap().view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("--Angle"))
    ));
}

#[test]
fn directional_ranges_expose_all_three_operands_and_exact_comparisons() {
    let condition = construct(r"style(1px < --\53 ize <= var(--max,))");
    let CssContainerStyleRangeRef::Ascending {
        left,
        left_inclusive,
        middle,
        right_inclusive,
        right,
    } = range(style(&condition)).view()
    else {
        panic!("ascending")
    };
    assert!(!left_inclusive);
    assert!(right_inclusive);
    assert!(matches!(
        left.view(),
        CssContainerStyleRangeOperandRef::Value(_)
    ));
    assert_eq!(left.value().serialize().unwrap().as_css(), "1px ");
    assert_eq!(reference(middle), "--Size");
    assert_eq!(middle.value().serialize().unwrap().as_css(), r" --\53 ize ");
    assert!(matches!(
        right.view(),
        CssContainerStyleRangeOperandRef::Value(_)
    ));
    assert_eq!(right.value().serialize().unwrap().as_css(), " var(--max,)");

    let condition = construct("style(--max >= --current > --min)");
    let CssContainerStyleRangeRef::Descending {
        left,
        left_inclusive,
        middle,
        right_inclusive,
        right,
    } = range(style(&condition)).view()
    else {
        panic!("descending")
    };
    assert!(left_inclusive);
    assert!(!right_inclusive);
    assert_eq!(
        (reference(left), reference(middle), reference(right)),
        ("--max", "--current", "--min")
    );
}

#[test]
fn generic_range_values_do_not_become_numeric_or_custom_reference_nodes() {
    for (source, left_expected) in [
        ("style(--/**/x = red)", "--/**/x "),
        ("style(\"--x\" = red)", "\"--x\" "),
        ("style(var(--x) = red)", "var(--x) "),
        ("style(--x: a > b)", "--x: a "),
    ] {
        let condition = construct(source);
        let CssContainerStyleRangeRef::Binary {
            left,
            comparison,
            right,
        } = range(style(&condition)).view()
        else {
            panic!("binary")
        };
        assert!(matches!(
            left.view(),
            CssContainerStyleRangeOperandRef::Value(_)
        ));
        assert!(matches!(
            right.view(),
            CssContainerStyleRangeOperandRef::Value(_)
        ));
        assert_eq!(left.value().serialize().unwrap().as_css(), left_expected);
        assert_eq!(
            comparison,
            if source.contains('>') {
                CssQueryComparison::GreaterThan
            } else {
                CssQueryComparison::Equal
            }
        );
    }
}

#[test]
fn whitespace_only_value_and_range_operands_keep_parsed_origins() {
    let source = "style(--x: )";
    let condition = construct(source);
    let (_, value) = plain(style(&condition));
    assert_eq!(value.serialize().unwrap().as_css(), " ");
    assert_eq!(value.components().len(), 1);
    assert!(matches!(
        value.components()[0].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Whitespace(" "))
    ));
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("whitespace origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(value.position().unwrap().byte_offset().value(), 10);

    let condition = construct("style(1px< <3px)");
    let CssContainerStyleRangeRef::Ascending { middle, .. } = range(style(&condition)).view()
    else {
        panic!("whitespace middle")
    };
    assert_eq!(middle.value().serialize().unwrap().as_css(), " ");
    assert_eq!(middle.value().position().unwrap().byte_offset().value(), 10);
    assert!(matches!(
        middle.view(),
        CssContainerStyleRangeOperandRef::Value(_)
    ));
}

#[test]
fn programmatic_operand_boundaries_and_mixed_origins_remain_inspectable() {
    let parsed = parse_component_values("/*source*/1").unwrap();
    let number = parsed.items().last().unwrap().clone();
    let argument = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("--x").unwrap(),
        CssComponentValue::try_token(":").unwrap(),
        number,
        CssComponentValue::try_ident("e2").unwrap(),
    ])
    .unwrap();
    let function = CssComponentValue::try_function("style", argument).unwrap();
    let condition = CssContainerCondition::try_from_components(
        CssComponentValues::try_new(vec![function]).unwrap(),
    )
    .unwrap();
    let (_, value) = plain(style(&condition));
    assert_eq!(value.serialize().unwrap().as_css(), "1/**/e2");
    assert_eq!(value.components().len(), 2);
    assert!(matches!(
        value.components()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(
        value.components()[1].origin(),
        &CssValueOrigin::Programmatic
    );
    assert_eq!(condition.origin(), &CssValueOrigin::Programmatic);
}

#[test]
fn style_limits_apply_before_admission_and_recovered_groups_keep_closure_origins() {
    let source = "style((--x) and future())";
    let values = parse_component_values(source).unwrap();
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
            panic!("component limit")
        };
        assert_eq!(error.kind(), expected);
    }
    let condition = construct("style((--x");
    let serialized = condition.serialize().unwrap();
    assert_eq!(serialized.as_css(), "style((--x))");
    for offset in [10, 11] {
        assert!(matches!(
            serialized.origin_at(offset),
            Some(CssSerializedOrigin::Token(
                CssValueOrigin::ImplicitClosure { .. }
            ))
        ));
    }
    let CssContainerStyleQueryKind::Parenthesized(child) = style(&condition).kind() else {
        panic!("retained group")
    };
    assert!(matches!(
        child.kind(),
        CssContainerStyleQueryKind::Feature(CssContainerStyleFeature::Boolean(_))
    ));
    assert_eq!(
        child.serialize_with_limit(2).unwrap_err().kind(),
        CssComponentValueErrorKind::ByteLimit
    );
}

#[test]
fn maximum_style_depth_supports_detached_clone_and_drop_on_an_ordinary_stack() {
    const CHILD: &str = "SURGEIST_STYLE_LIFECYCLE_CHILD";
    const TEST: &str = "maximum_style_depth_supports_detached_clone_and_drop_on_an_ordinary_stack";
    if std::env::var(CHILD).as_deref() == Ok(TEST) {
        let worker = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                let grouped = format!("{}--x{}", "(".repeat(255), ")".repeat(255));
                let negated = format!("{}--x{}", "not (".repeat(255), ")".repeat(255));
                let mut conjunction = "(--x) and future()".to_owned();
                for _ in 0..254 {
                    conjunction = format!("({conjunction}) and future()");
                }
                let opaque = format!("{}1{}", "future(".repeat(255), ")".repeat(255));
                for inner in [grouped, negated, conjunction, opaque] {
                    let source = format!("style({inner})");
                    let original = construct(&source);
                    let detached = style(&original).clone();
                    drop(original);
                    let cloned = detached.clone();
                    assert_eq!(detached, cloned);
                    assert_eq!(detached.serialize().unwrap().as_css(), inner);
                    drop(detached);
                    assert_eq!(cloned.serialize().unwrap().as_css(), inner);
                    drop(cloned);
                }
            })
            .unwrap();
        if let Err(panic) = worker.join() {
            std::panic::resume_unwind(panic);
        }
        println!("style lifecycle completed");
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
    assert!(String::from_utf8_lossy(&output.stdout).contains("style lifecycle completed"));
}

#[test]
fn escaped_comparison_identifiers_are_values_and_grouped_failures_are_inner_opaque() {
    let condition = construct(r"style(--x: \>)");
    let (_, value) = plain(style(&condition));
    assert!(matches!(
        value.components().last().unwrap().view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident(">"))
    ));
    let condition = construct("style((--x:var(color)))");
    let CssContainerStyleQueryKind::GeneralEnclosed(opaque) = style(&condition).kind() else {
        panic!("inner general-enclosed")
    };
    assert_eq!(opaque.authored(), Some("(--x:var(color))"));
    let condition = construct("style(--x:var(color))");
    assert!(matches!(
        condition.kind(),
        CssContainerConditionKind::GeneralEnclosed(_)
    ));
}
