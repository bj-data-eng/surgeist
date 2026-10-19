#![forbid(unsafe_code)]
//! Counter Styles3 §§3.1.2/3.5/3.6/3.8 and imported Values4 (2021-07-15)
//! §§11.9/11.12: specified Number-result graphs precede integer computation.
//! Literal order checks remain intrinsic; symbolic comparisons are downstream.

use surgeist_css::*;

fn calculation(text: &str) -> CssIntegerValue {
    CssIntegerValue::Calculation(
        CssIntegerCalculation::try_from_components(parse_component_values(text).unwrap()).unwrap(),
    )
}

fn symbol() -> CssCounterSymbol {
    CssCounterSymbol::String(CssContentString::try_new("0").unwrap())
}

fn parsed(source: &str) -> CssCounterStyleRule {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("one retained counter style")
    };
    rule.clone()
}

#[test]
fn typed_constructors_preserve_fractional_negative_and_nonfinite_calculation_graphs() {
    for text in ["calc(-1.5)", "calc(1.5)", "calc(infinity)", "calc(NaN)"] {
        let value = calculation(text);
        let fixed = CssCounterStyleFixedSystem::new(Some(value.clone()));
        assert_eq!(fixed.first_symbol_value(), Some(&value));
        let range = CssCounterStyleRangeInterval::try_new(
            CssCounterStyleRangeBound::Integer(value.clone()),
            CssCounterStyleRangeBound::Integer(calculation("calc(-20)")),
        )
        .unwrap();
        assert_eq!(
            range.lower(),
            &CssCounterStyleRangeBound::Integer(value.clone())
        );
        let pad = CssCounterStylePad::try_new(value.clone(), symbol()).unwrap();
        assert_eq!(pad.minimum_length(), &value);
        let tuple = CssCounterAdditiveTuple::try_new(value.clone(), symbol()).unwrap();
        assert_eq!(tuple.weight(), &value);
        assert_eq!(tuple, tuple.clone());
        let CssIntegerValue::Calculation(graph) = pad.minimum_length() else {
            panic!("calculation variant must be retained")
        };
        assert_eq!(graph.components(), &parse_component_values(text).unwrap());
    }
}

#[test]
fn symbolic_order_is_deferred_while_adjacent_literal_descent_remains_checked() {
    for input in [
        "calc(1) X, calc(2) Y",
        "calc(1.5) X, calc(1.5) Y",
        "0 X, calc(-1) Y",
        "calc(-1) X, 5 Y",
        "1 X, calc(2) Y, 2 Z",
    ] {
        let report = parse_counter_style_descriptor_value(
            input,
            CssCounterStyleDescriptorKind::AdditiveSymbols,
        );
        assert!(report.is_clean(), "{input}: {:?}", report.diagnostics());
    }
    let tuple = |value| CssCounterAdditiveTuple::try_new(value, symbol()).unwrap();
    let literal = |value| CssIntegerValue::Literal(CssIntegerLiteral::from_i32(value));
    assert!(
        CssCounterAdditiveSymbols::try_new(vec![
            tuple(calculation("calc(100)")),
            tuple(literal(1)),
            tuple(literal(2)),
        ])
        .is_none()
    );
    assert!(
        CssCounterAdditiveSymbols::try_new(vec![
            tuple(literal(1)),
            tuple(calculation("calc(100)")),
            tuple(literal(2)),
        ])
        .is_some()
    );
    assert!(
        CssCounterStyleRangeInterval::try_new(
            CssCounterStyleRangeBound::Integer(calculation("calc(10)")),
            CssCounterStyleRangeBound::Integer(literal(1)),
        )
        .is_some()
    );
}

#[test]
fn fixed_calculated_default_retains_its_wrapper_and_explicit_occurrence() {
    let rule = parsed("@counter-style x { system: fixed calc(1); }");
    assert_eq!(
        rule.descriptor_specified_css(CssCounterStyleDescriptorKind::System)
            .unwrap(),
        "fixed calc(1)"
    );
    let omitted = parsed("@counter-style x { system: fixed; }");
    assert_ne!(
        rule.descriptors().system().unwrap().value().view(),
        omitted.descriptors().system().unwrap().value().view()
    );
}

#[test]
fn isolated_rule_and_genuine_body_fronts_share_math_and_complete_member_recovery() {
    let body = "{system: fixed calc(1.5); range: calc(8) calc(2); pad: '0' min(1, 2); additive-symbols: max(1, 2) X, 0 N; pad: calc(1px) '0';}";
    let block = parse_counter_style_block(body);
    assert_eq!(block.diagnostics().len(), 1);
    assert_eq!(
        block.diagnostics()[0].action(),
        CssRecoveryAction::DropDescriptor
    );
    let descriptors = block.syntax().as_ref().unwrap().body();
    assert_eq!(descriptors.occurrences().count(), 4);
    let source = format!("@counter-style x {body}");
    let isolated = parse_rule(&source, &CssNamespaceContext::default());
    assert_eq!(isolated.diagnostics().len(), 1);
    let Some(CssRule::CounterStyle(rule)) = isolated.syntax() else {
        panic!("rule retained after descriptor recovery")
    };
    assert_eq!(rule.descriptors().occurrences().count(), 4);
    assert_eq!(
        rule.descriptor_specified_css(CssCounterStyleDescriptorKind::Pad)
            .unwrap(),
        "calc(1) \"0\""
    );
    assert_eq!(
        rule.descriptor_specified_css(CssCounterStyleDescriptorKind::Range)
            .unwrap(),
        "calc(8) calc(2)"
    );
}

#[test]
fn parsed_integer_graphs_keep_exact_lexical_source_and_origin_sensitive_identity() {
    let source = "@counter-style x { system: fixed calc(+0001 + 1); }";
    let rule = parsed(source);
    let CssCounterStyleDescriptorValueRef::System(CssCounterStyleSystem::Fixed(fixed)) =
        rule.descriptors().system().unwrap().value().view()
    else {
        panic!("fixed system")
    };
    let CssIntegerValue::Calculation(graph) = fixed.first_symbol_value().unwrap() else {
        panic!("calculation graph")
    };
    let CssValueOrigin::Parsed(origin) = graph.origin() else {
        panic!("original parsed math origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        &source[origin.span().start().byte_offset().value()
            ..origin.span().end().byte_offset().value()],
        "calc("
    );
    assert_eq!(
        graph.components().serialize().unwrap().as_css(),
        "calc(+0001 + 1)"
    );
    let same = parsed(source);
    let shifted = parsed(&format!("/* shifted */{source}"));
    assert_eq!(
        rule.descriptors().system().unwrap().value().view(),
        same.descriptors().system().unwrap().value().view()
    );
    assert_ne!(
        rule.descriptors().system().unwrap().value().view(),
        shifted.descriptors().system().unwrap().value().view()
    );
}

#[test]
fn checked_pending_env_math_reentry_is_reusable_and_rejects_residual_var_and_env() {
    for token in ["env(weight)", "env(weight, calc(1.5) X)"] {
        let pending = CssCounterStyleDescriptorValue::try_from_components(
            CssCounterStyleDescriptorKind::AdditiveSymbols,
            parse_component_values(token).unwrap(),
        )
        .unwrap();
        let before = pending.clone();
        assert!(
            pending
                .reparse_after_substitution(parse_component_values("2 X, 2 Y").unwrap())
                .is_err()
        );
        for residual in ["calc(var(--weight)) X", "calc(env(weight)) X"] {
            assert!(
                pending
                    .reparse_after_substitution(parse_component_values(residual).unwrap())
                    .is_err()
            );
        }
        let completed = pending
            .reparse_after_substitution(
                parse_component_values("calc(1.5) X, calc(-1.5) Y").unwrap(),
            )
            .unwrap();
        assert!(matches!(
            completed.view(),
            CssCounterStyleDescriptorValueRef::AdditiveSymbols(_)
        ));
        assert!(matches!(
            pending.view(),
            CssCounterStyleDescriptorValueRef::Pending(_)
        ));
        assert_eq!(pending, before);
    }
}

#[test]
fn normalization_keeps_math_payloads_and_descriptor_origins_in_conditional_and_scoped_rules() {
    let source = "@media print { @counter-style A { system: fixed calc(1.5); } } @scope { @counter-style B { pad: calc(-1.5) '0'; } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let retained = report.syntax().clone();
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let counters = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::CounterStyle(rule) => Some(rule),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(counters.len(), 2);
    assert_eq!(
        counters[0]
            .descriptor_specified_css(CssCounterStyleDescriptorKind::System)
            .unwrap(),
        "fixed calc(1.5)"
    );
    assert_eq!(
        counters[1]
            .descriptor_specified_css(CssCounterStyleDescriptorKind::Pad)
            .unwrap(),
        "calc(-1.5) \"0\""
    );
    for descriptor in [
        counters[0].descriptors().system().unwrap(),
        counters[1].descriptors().pad().unwrap(),
    ] {
        assert_eq!(
            descriptor.value().origin().unwrap().source().as_str(),
            source
        );
    }
    assert_eq!(report.syntax(), &retained);
}

#[test]
fn suppressed_duplicate_math_spends_cumulative_input_budget_without_emitting_bytes() {
    let effective = parsed("@counter-style x { system: fixed; }");
    let duplicated =
        parsed("@counter-style x { system: fixed calc(1 + 2 + 3 + 4 + 5 + 6); system: fixed; }");
    let expected = "@counter-style x { system: fixed; }";
    let limits = CssSpecifiedValueSerializationLimits::new(10, 1000, expected.len());
    assert_eq!(
        effective.to_specified_css_with_limits(limits).unwrap(),
        expected
    );
    assert_eq!(
        duplicated
            .to_specified_css_with_limits(limits)
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(duplicated.to_specified_css().unwrap(), expected);
}

#[test]
fn checked_math_depth_failure_keeps_the_original_pending_descriptor() {
    let pending = CssCounterStyleDescriptorValue::try_from_components(
        CssCounterStyleDescriptorKind::Pad,
        parse_component_values("env(pad)").unwrap(),
    )
    .unwrap();
    let source = "calc(1 + (2 + (3 + (4 + 5)))) '0'";
    let components = parse_component_values(source).unwrap();
    let limits = CssComponentValueLimits::try_new(2, 1000, 10000).unwrap();
    let before = pending.clone();
    assert!(
        pending
            .reparse_after_substitution_with_limits(components.clone(), limits)
            .is_err()
    );
    let completed = pending.reparse_after_substitution(components).unwrap();
    assert!(matches!(
        completed.view(),
        CssCounterStyleDescriptorValueRef::Pad(_)
    ));
    assert_eq!(pending, before);
}
