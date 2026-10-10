#![forbid(unsafe_code)]
//! Counter Styles 3 §§3.1 and 3.8 distinguish valid at-rules from definitions
//! with enough symbols to produce a counter style. Insufficient symbols do not
//! invalidate the authored at-rule. §3.1.7 separately invalidates extends rules
//! with authored symbols or additive-symbols.

use surgeist_css::{
    CssCounterStyleDescriptorKind, CssCounterStyleRule, CssErrorCode, CssNormalizedItem,
    CssRecoveryAction, CssRule, CssRuleContextKindRef, CssScopedRule, normalize_report,
    parse_sheet, validate_sheet,
};

fn assert_retained(body: &str, canonical: &str) -> CssCounterStyleRule {
    let source = format!(".before{{}} @counter-style Incomplete {{ {body} }} .after{{}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [
        CssRule::Style(_),
        CssRule::CounterStyle(rule),
        CssRule::Style(_),
    ] = report.syntax().rules()
    else {
        panic!("a valid counter at-rule must survive between its siblings");
    };
    assert_eq!(rule.name().as_str(), "Incomplete");
    assert_eq!(
        rule.position().byte_offset().value(),
        source.find('@').unwrap()
    );
    assert_eq!(
        rule.to_specified_css().unwrap(),
        if canonical.is_empty() {
            String::from("@counter-style Incomplete { }")
        } else {
            format!("@counter-style Incomplete {{ {canonical} }}")
        }
    );
    assert!(validate_sheet(&source).is_ok());
    let normalized = normalize_report(&report).unwrap();
    let retained: Vec<_> = normalized
        .syntax()
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::CounterStyle(value) => Some(value),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(retained, [rule]);
    rule.clone()
}

macro_rules! valid_rule {
    ($name:ident, $body:literal, $canonical:literal) => {
        #[test]
        fn $name() {
            assert_retained($body, $canonical);
        }
    };
}

valid_rule!(empty_rule_keeps_omitted_descriptors, "", "");
valid_rule!(
    implicit_symbolic_keeps_missing_symbols,
    "range:auto;",
    "range: auto;"
);
valid_rule!(
    cyclic_keeps_missing_symbols,
    "system:cyclic;",
    "system: cyclic;"
);
valid_rule!(
    symbolic_keeps_missing_symbols,
    "system:symbolic;",
    "system: symbolic;"
);
valid_rule!(
    fixed_keeps_missing_symbols,
    "system:fixed -2;",
    "system: fixed -2;"
);
valid_rule!(
    numeric_keeps_missing_symbols,
    "system:numeric;",
    "system: numeric;"
);
valid_rule!(
    alphabetic_keeps_missing_symbols,
    "system:alphabetic;",
    "system: alphabetic;"
);
valid_rule!(
    additive_keeps_missing_tuples,
    "system:additive;",
    "system: additive;"
);
valid_rule!(
    numeric_keeps_one_authored_symbol,
    "system:numeric; symbols:a;",
    "system: numeric; symbols: a;"
);
valid_rule!(
    alphabetic_keeps_one_authored_symbol,
    "system:alphabetic; symbols:a;",
    "system: alphabetic; symbols: a;"
);
valid_rule!(
    last_valid_system_can_make_symbols_insufficient,
    "system:cyclic; symbols:a; system:numeric;",
    "system: numeric; symbols: a;"
);
valid_rule!(
    last_valid_symbols_can_make_numeric_definition_insufficient,
    "system:numeric; symbols:a b; symbols:c;",
    "system: numeric; symbols: c;"
);

#[test]
fn omitted_system_stays_omitted_in_an_empty_authored_rule() {
    let rule = assert_retained("", "");
    assert!(rule.descriptors().system().is_none());
    assert_eq!(
        rule.descriptor_specified_css(CssCounterStyleDescriptorKind::System)
            .unwrap(),
        ""
    );
    assert_eq!(rule.descriptors().occurrences().count(), 0);
}

#[test]
fn invalid_symbol_descriptor_drops_only_that_descriptor_on_incomplete_rule() {
    let source =
        "@counter-style Incomplete { system:numeric; symbols:inherit; suffix:'.'; } .after{}";
    let report = parse_sheet(source);
    let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("recovery must keep the valid but non-defining rule");
    };
    let [diagnostic] = report.diagnostics() else {
        panic!("only invalid symbols produces a diagnostic");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidDescriptorValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    assert!(rule.descriptors().symbols().is_none());
    assert_eq!(
        rule.to_specified_css().unwrap(),
        "@counter-style Incomplete { system: numeric; suffix: \".\"; }"
    );
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert_eq!(
        normalize_report(&report).unwrap().diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn conditional_and_scoped_incomplete_rules_keep_original_payloads() {
    let source = "@media print { @counter-style Incomplete { system:additive; } } @scope { @counter-style Other { system:numeric; symbols:a; } .after{} }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Media(media), CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("two wrapper rules");
    };
    let [CssRule::CounterStyle(first)] = media.rules() else {
        panic!("conditional counter rule");
    };
    let [CssScopedRule::CounterStyle(second), CssScopedRule::Style(_)] = scope.rules().rules()
    else {
        panic!("scoped counter rule and sibling");
    };
    let normalized = normalize_report(&report).unwrap();
    let values: Vec<_> = normalized
        .syntax()
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::CounterStyle(rule) => Some(rule),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(values, [first, second]);
}

#[test]
fn eof_closed_empty_rule_is_retained_with_its_implicit_closure_diagnostic() {
    let report = parse_sheet("@counter-style Incomplete {");
    let [diagnostic] = report.diagnostics() else {
        panic!("one implicit rule closure: {:?}", report.diagnostics());
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), 27);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 27);
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("implicit EOF closure");
    };
    assert_eq!(
        rule.to_specified_css().unwrap(),
        "@counter-style Incomplete { }"
    );
}

// Controls protect genuine grammar/combination failures and defining styles.
valid_rule!(
    numeric_two_symbols_remain_valid,
    "system:numeric; symbols:a b;",
    "system: numeric; symbols: a b;"
);
valid_rule!(
    alphabetic_two_symbols_remain_valid,
    "system:alphabetic; symbols:a b;",
    "system: alphabetic; symbols: a b;"
);
valid_rule!(
    extends_omission_remains_valid,
    "system:extends Missing;",
    "system: extends Missing;"
);
valid_rule!(
    additive_with_one_zero_tuple_remains_valid,
    "system:additive; additive-symbols:0 N;",
    "system: additive; additive-symbols: 0 N;"
);

#[test]
fn extends_with_valid_symbols_retains_rule_without_defining_style() {
    for descriptor in ["symbols:a;", "additive-symbols:0 N;"] {
        let source =
            format!("@counter-style Invalid {{ system:extends Missing; {descriptor} }} .after{{}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean());
        let [CssRule::CounterStyle(counter), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("retained grammar-valid rule")
        };
        assert_eq!(
            counter
                .descriptors()
                .prospective()
                .unwrap()
                .definition_status(),
            surgeist_css::CssCounterStyleDefinitionStatus::Undefined(
                surgeist_css::CssCounterStyleDefinitionIssue::ExtendsWithSymbols
            )
        );
    }
}

#[test]
fn missing_block_remains_an_invalid_at_rule() {
    let report = parse_sheet("@counter-style Incomplete;");
    assert!(report.syntax().rules().is_empty());
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
}
