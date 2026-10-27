//! Counter Styles3 §§3.1.2/3.5/3.6/3.8 and imported Values4 (2021-07-15)
//! §§11.9/11.12: specified Number-result graphs precede integer computation.
//! Literal order checks remain intrinsic; symbolic comparisons are downstream.

use crate::css::support::rules::sole_counter_style as parsed;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValueLimits, CssComponentValues, CssValueOrigin};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::normalize::{CssNormalizedItem, CssRuleContextKindRef};
use surgeist_css::numeric::{CssCalculation, CssInteger, CssNumeric, CssNumericLiteral};
use surgeist_css::rule::counter_style::{
    CssCounterAdditiveSymbols, CssCounterAdditiveTuple, CssCounterStyleDescriptorKind,
    CssCounterStyleDescriptorValue, CssCounterStyleDescriptorValueRef, CssCounterStyleDescriptors,
    CssCounterStyleFixedSystem, CssCounterStylePad, CssCounterStyleRangeBound,
    CssCounterStyleRangeInterval, CssCounterStyleSystem, CssCounterSymbol,
};
use surgeist_css::rule::{CssBlockContext, CssBlockFragment, CssRule};
use surgeist_css::selector::CssNamespaceContext;
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::content::CssContentString;

fn calculation(text: &str) -> CssNumeric<CssInteger> {
    CssNumeric::<CssInteger>::try_from_calculation(
        CssCalculation::<CssInteger>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), text).unwrap(),
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn symbol() -> CssCounterSymbol {
    CssCounterSymbol::String(CssContentString::try_new("0").unwrap())
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
        let Some(graph) = pad.minimum_length().calculation() else {
            panic!("calculation variant must be retained")
        };
        assert_eq!(
            graph.components(),
            &CssComponentValues::parse(&CssParser::default(), text).unwrap()
        );
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
        let report = CssCounterStyleDescriptorValue::parse(
            &CssParser::default(),
            input,
            CssCounterStyleDescriptorKind::AdditiveSymbols,
        );
        assert!(report.is_clean(), "{input}: {:?}", report.diagnostics());
    }
    let tuple = |value| CssCounterAdditiveTuple::try_new(value, symbol()).unwrap();
    let literal = |value| {
        CssNumeric::<CssInteger>::try_from_literal(CssNumericLiteral::<CssInteger>::from_i32(value))
            .unwrap()
    };
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
        rule.descriptor_specified_css(CssCounterStyleDescriptorKind::System, Default::default())
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
    let block = CssBlockFragment::<CssCounterStyleDescriptors>::parse(
        &CssParser::default(),
        body,
        &CssBlockContext::default(),
    );
    assert_eq!(block.diagnostics().len(), 1);
    assert_eq!(
        block.diagnostics()[0].action(),
        CssRecoveryAction::DropDescriptor
    );
    let descriptors = block.syntax().as_ref().unwrap().body();
    assert_eq!(descriptors.occurrences().count(), 4);
    let source = format!("@counter-style x {body}");
    let isolated = CssRule::parse(
        &CssParser::default(),
        &source,
        &CssNamespaceContext::default(),
    );
    assert_eq!(isolated.diagnostics().len(), 1);
    let Some(CssRule::CounterStyle(rule)) = isolated.syntax() else {
        panic!("rule retained after descriptor recovery")
    };
    assert_eq!(rule.descriptors().occurrences().count(), 4);
    assert_eq!(
        rule.descriptor_specified_css(CssCounterStyleDescriptorKind::Pad, Default::default())
            .unwrap(),
        "calc(1) \"0\""
    );
    assert_eq!(
        rule.descriptor_specified_css(CssCounterStyleDescriptorKind::Range, Default::default())
            .unwrap(),
        "calc(8) calc(2)"
    );
}

#[test]
fn parsed_integer_graphs_keep_exact_lexical_source_and_origin_blind_identity() {
    let source = "@counter-style x { system: fixed calc(+0001 + 1); }";
    let rule = parsed(source);
    let CssCounterStyleDescriptorValueRef::System(CssCounterStyleSystem::Fixed(fixed)) =
        rule.descriptors().system().unwrap().value().view()
    else {
        panic!("fixed system")
    };
    let Some(graph) = fixed.first_symbol_value().unwrap().calculation() else {
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
        graph
            .components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "calc(+0001 + 1)"
    );
    let same = parsed(source);
    let shifted = parsed(&format!("/* shifted */{source}"));
    assert_eq!(
        rule.descriptors().system().unwrap().value().view(),
        same.descriptors().system().unwrap().value().view()
    );
    assert_eq!(
        rule.descriptors().system().unwrap().value().view(),
        shifted.descriptors().system().unwrap().value().view()
    );
}

#[test]
fn checked_pending_env_math_reentry_is_reusable_and_rejects_residual_var_and_env() {
    for token in ["env(weight)", "env(weight, calc(1.5) X)"] {
        let pending = CssCounterStyleDescriptorValue::try_from_components(
            CssCounterStyleDescriptorKind::AdditiveSymbols,
            CssComponentValues::parse(&CssParser::default(), token).unwrap(),
            Default::default(),
        )
        .unwrap();
        let before = pending.clone();
        // Counter Styles 3 §3.6: literal weights must strictly descend, so
        // the second weight is responsible.
        let error = pending
            .reparse_after_substitution(
                CssComponentValues::parse(&CssParser::default(), "2 X, 2 Y").unwrap(),
                Default::default(),
            )
            .unwrap_err();
        let Some(detail) = error.diagnostic() else {
            panic!("grammar failure: {error:?}")
        };
        crate::css::support::rules::invalid_descriptor("counter-style", "additive-symbols")
            .at("2")
            .assert_kind(detail);
        for residual in ["calc(var(--weight)) X", "calc(env(weight)) X"] {
            assert!(
                pending
                    .reparse_after_substitution(
                        CssComponentValues::parse(&CssParser::default(), residual).unwrap(),
                        Default::default()
                    )
                    .unwrap_err()
                    .is_residual_substitution(),
                "{residual}"
            );
        }
        let completed = pending
            .reparse_after_substitution(
                CssComponentValues::parse(&CssParser::default(), "calc(1.5) X, calc(-1.5) Y")
                    .unwrap(),
                Default::default(),
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
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let retained = report.syntax().clone();
    let normalized = report.syntax().normalize(Default::default()).unwrap();
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
            .descriptor_specified_css(CssCounterStyleDescriptorKind::System, Default::default())
            .unwrap(),
        "fixed calc(1.5)"
    );
    assert_eq!(
        counters[1]
            .descriptor_specified_css(CssCounterStyleDescriptorKind::Pad, Default::default())
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
    let limits = CssSerializationLimits::new(10, 1000, expected.len());
    assert_eq!(effective.serialize_specified(limits).unwrap(), expected);
    assert!(
        duplicated
            .serialize_specified(limits)
            .unwrap_err()
            .is_input_node_limit()
    );
    assert_eq!(
        duplicated.serialize_specified(Default::default()).unwrap(),
        expected
    );
}

#[test]
fn checked_math_depth_failure_keeps_the_original_pending_descriptor() {
    let pending = CssCounterStyleDescriptorValue::try_from_components(
        CssCounterStyleDescriptorKind::Pad,
        CssComponentValues::parse(&CssParser::default(), "env(pad)").unwrap(),
        Default::default(),
    )
    .unwrap();
    let source = "calc(1 + (2 + (3 + (4 + 5)))) '0'";
    let components = CssComponentValues::parse(&CssParser::default(), source).unwrap();
    let limits = CssComponentValueLimits::try_new(2, 1000, 10000).unwrap();
    let before = pending.clone();
    assert!(
        pending
            .reparse_after_substitution(components.clone(), limits)
            .unwrap_err()
            .component_error()
            .is_some_and(|error| error.is_nesting_limit())
    );
    let completed = pending
        .reparse_after_substitution(components, Default::default())
        .unwrap();
    assert!(matches!(
        completed.view(),
        CssCounterStyleDescriptorValueRef::Pad(_)
    ));
    assert_eq!(pending, before);
}
