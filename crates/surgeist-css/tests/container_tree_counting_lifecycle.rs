#![forbid(unsafe_code)]
//! Authored lifecycle oracles: Conditional 5 WD20251030 §6.1 and the imported
//! Values 5 attribute-substitution WD20241111 §9. The query-container, flat-tree
//! and ultimate-originating-element evaluation rules remain downstream; these
//! tests require the symbolic inputs and their authored provenance to survive.
use surgeist_css::*;

fn container(sheet: &CssSheet) -> &CssContainerRule {
    let [CssRule::Container(rule)] = sheet.rules() else {
        panic!("one enclosing container rule")
    };
    rule
}

fn query(rule: &CssContainerRule, index: usize) -> &CssContainerCondition {
    rule.prelude().entries()[index].query().unwrap()
}

fn assert_origin(origin: &CssValueOrigin, source: &str, opening: &str) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("original authored token provenance")
    };
    let start = source.find(opening).unwrap();
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(
        origin.span().end().byte_offset().value(),
        start + opening.len()
    );
    assert_eq!(
        origin.span().start().column().value() as usize,
        source[..start].encode_utf16().count()
    );
}

fn tree(expression: CssCalculationExpressionRef<'_>) -> CssCalculationTreeCountingRef<'_> {
    let CssCalculationExpressionRef::TreeCounting(value) = expression else {
        panic!("symbolic tree-counting identity, without a resolved scalar")
    };
    value
}

fn ratio_numerator(condition: &CssContainerCondition) -> &CssNumberCalculation {
    let CssContainerConditionKind::Feature(CssContainerFeatureQuery::AspectRatio(range)) =
        condition.kind()
    else {
        panic!("recognized aspect-ratio")
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("plain ratio")
    };
    let CssContainerRatioRef::Numeric(value) = value.view() else {
        panic!("numeric ratio")
    };
    value.numerator()
}

fn assert_context(sheet: &CssSheet, expected: &CssContainerPrelude) {
    let normalized = normalize_sheet(sheet).unwrap();
    let containers: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::Container { prelude } => Some((context, prelude)),
                _ => None,
            },
            _ => None,
        })
        .collect();
    let [(context, prelude)] = containers.as_slice() else {
        panic!("one intact container context")
    };
    assert_eq!(*prelude, expected);
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    let [declaration] = declarations.as_slice() else {
        panic!("one retained body declaration")
    };
    assert_eq!(
        declaration.source().known().unwrap().property(),
        CssKnownProperty::Color
    );
    // value_components retains source trivia; Syntax §5.4.6 excludes edge
    // whitespace from the semantic value. Specified text is checked separately.
    assert_eq!(
        declaration
            .source()
            .value_components()
            .serialize()
            .unwrap()
            .as_css()
            .trim(),
        "red"
    );
    assert!(
        declaration
            .rule_context()
            .parent()
            .unwrap()
            .same_context(context)
    );
    assert!(
        declaration.selector_context().selectors()[0]
            .selector()
            .has_pseudo_elements()
    );
}

#[test]
fn container_reuse_and_normalization_keep_tree_relative_unit_and_pending_inputs() {
    let source = "/*😀*/@container Card (width: calc(sibling-count() * 2em)), (aspect-ratio: sibling-index()), (height: calc(sibling-index() * var(--Unit, 1cqw))){.child::before{color:red}}";
    let expected = "@container Card (width: calc(sibling-count() * 2em)), (aspect-ratio: sibling-index()), (height: calc(sibling-index() * var(--Unit, 1cqw))) { .child::before { color: red; } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let rule = container(report.syntax());
    assert_eq!(rule.prelude().entries()[0].name().unwrap().as_str(), "Card");
    let CssContainerConditionKind::Feature(CssContainerFeatureQuery::Width(range)) =
        query(rule, 0).kind()
    else {
        panic!("typed width operand")
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("plain width")
    };
    let CssContainerLengthRef::Numeric(value) = value.view() else {
        panic!("symbolic length")
    };
    assert_eq!(value.result_type(), CssCalculationType::Length);
    let CssCalculationExpressionRef::NestedCalc(calc) = value.expression() else {
        panic!("authored calc")
    };
    let CssCalculationExpressionRef::Product(product) = calc.operand() else {
        panic!("tree times relative length")
    };
    let count = tree(product.factor(0).unwrap().expression());
    assert_eq!(count.function(), CssTreeCountingFunction::SiblingCount);
    assert_eq!(
        count.numeric_type().exponent(CssNumericDimension::Length),
        0
    );
    assert_origin(count.origin(), source, "sibling-count(");
    let CssCalculationExpressionRef::Value(unit) = product.factor(1).unwrap().expression() else {
        panic!("unresolved em operand")
    };
    assert_eq!(unit.literal().representation(), "2");
    assert_eq!(unit.literal().unit(), Some("em"));
    assert_origin(unit.literal().origin(), source, "2em");
    let index = tree(ratio_numerator(query(rule, 1)).expression());
    assert_eq!(index.function(), CssTreeCountingFunction::SiblingIndex);
    assert_origin(index.origin(), source, "sibling-index(");
    let CssContainerConditionKind::Feature(CssContainerFeatureQuery::Height(range)) =
        query(rule, 2).kind()
    else {
        panic!("recognized pending height")
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("plain height")
    };
    let CssContainerLengthRef::Pending(value) = value.view() else {
        panic!("substitution remains pending")
    };
    assert_eq!(value.domain(), CssContainerValueDomain::Length);
    assert_eq!(
        value.serialize().unwrap().as_css(),
        "calc(sibling-index() * var(--Unit, 1cqw))"
    );
    assert_context(report.syntax(), rule.prelude());

    let checked_prelude = CssContainerPrelude::try_from_components(
        CssComponentValues::try_new(rule.prelude().components().to_vec()).unwrap(),
    )
    .unwrap();
    assert_eq!(&checked_prelude, rule.prelude());
    let checked = CssContainerRule::try_new(
        checked_prelude,
        rule.rules().to_vec(),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    assert_eq!(checked.position(), None);
    assert_eq!(checked.rules(), rule.rules());
    let reused = CssSheet::try_from_rules(vec![CssRule::Container(checked)]).unwrap();
    assert_context(&reused, rule.prelude());
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    assert_eq!(reused.to_specified_css().unwrap(), expected);
    assert_eq!(report, before);
    drop(report);
    assert_origin(
        tree(ratio_numerator(query(container(&reused), 1)).expression()).origin(),
        source,
        "sibling-index(",
    );
    let emitted = parse_sheet(expected);
    assert!(emitted.is_clean(), "{:?}", emitted.diagnostics());
    assert_eq!(emitted.syntax().to_specified_css().unwrap(), expected);
    assert_context(emitted.syntax(), container(emitted.syntax()).prelude());
    assert_eq!(
        tree(ratio_numerator(query(container(emitted.syntax()), 1)).expression()).function(),
        CssTreeCountingFunction::SiblingIndex
    );
}

#[test]
fn escaped_tree_function_identity_keeps_authored_trivia_and_original_opening_origin() {
    let source =
        r"/*😀*/@container (aspect-ratio: SIBLING-\63 OUNT( /**/ )){.child::before{color:red}}";
    let expected =
        r"@container (aspect-ratio: SIBLING-\63 OUNT( /**/ )) { .child::before { color: red; } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let rule = container(report.syntax());
    let numerator = ratio_numerator(query(rule, 0));
    let count = tree(numerator.expression());
    assert_eq!(count.function(), CssTreeCountingFunction::SiblingCount);
    assert_origin(count.origin(), source, r"SIBLING-\63 OUNT(");
    // Numeric projection canonicalizes the name; the container lexical provider
    // preserves the authored spelling and trivia needed for specified replay.
    let numeric = numerator.serialize().unwrap();
    assert_eq!(numeric.as_css(), "sibling-count()");
    assert_eq!(
        numeric.origin_at(0),
        Some(&CssSerializedOrigin::Token(count.origin().clone()))
    );
    assert_context(report.syntax(), rule.prelude());
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    let emitted = parse_sheet(expected);
    assert!(emitted.is_clean(), "{:?}", emitted.diagnostics());
    assert_eq!(
        tree(ratio_numerator(query(container(emitted.syntax()), 0)).expression()).function(),
        CssTreeCountingFunction::SiblingCount
    );
    assert_eq!(emitted.syntax().to_specified_css().unwrap(), expected);
}

#[test]
fn unsupported_tree_arguments_and_wrong_domains_keep_opaque_query_and_body_on_replay() {
    // §9 defines zero-argument integer functions; its future selector argument
    // is not admitted. Invalid feature values remain valid general-enclosed CSS.
    for condition in [
        "(width: sibling-count())",
        "(aspect-ratio: sibling-index(1))",
        "(width: calc(sibling-count() + 1em))",
        "(aspect-ratio: sibling-count(of .item))",
    ] {
        let source = format!("@container {condition}{{.child::before{{color:red}}}}");
        let expected = format!("@container {condition} {{ .child::before {{ color: red; }} }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{condition}: {:?}", report.diagnostics());
        let rule = container(report.syntax());
        assert!(matches!(
            query(rule, 0).kind(),
            CssContainerConditionKind::GeneralEnclosed(_)
        ));
        let checked = CssContainerPrelude::try_from_components(
            CssComponentValues::try_new(rule.prelude().components().to_vec()).unwrap(),
        )
        .unwrap();
        assert_eq!(&checked, rule.prelude());
        assert_context(report.syntax(), &checked);
        assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
        let emitted = parse_sheet(&expected);
        assert!(emitted.is_clean(), "{:?}", emitted.diagnostics());
        assert!(matches!(
            query(container(emitted.syntax()), 0).kind(),
            CssContainerConditionKind::GeneralEnclosed(_)
        ));
        assert_context(emitted.syntax(), container(emitted.syntax()).prelude());
        assert_eq!(emitted.syntax().to_specified_css().unwrap(), expected);
    }
}

#[test]
fn tree_query_output_byte_limit_is_cumulative_across_rules_and_atomic_on_failure() {
    let source = "@container (aspect-ratio: sibling-count(/*é*/)){}@container (aspect-ratio: sibling-index()){}";
    let expected = "@container (aspect-ratio: sibling-count(/*é*/)) { }\n@container (aspect-ratio: sibling-index()) { }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let exact = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len());
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    let short =
        CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len() - 1);
    // Both individual rules fit; the second rule exhausts the shared byte budget.
    for rule in report.syntax().rules() {
        assert!(rule.to_specified_css_with_limits(short).is_ok());
    }
    let error = report
        .syntax()
        .to_specified_css_with_limits(short)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CssSpecifiedRuleSerializationErrorKind::Resource(
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        )
    );
    assert_eq!(error.rule_index(), Some(1));
    assert_eq!(report, before);
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    let emitted = parse_sheet(expected);
    assert!(emitted.is_clean(), "{:?}", emitted.diagnostics());
    assert_eq!(emitted.syntax().to_specified_css().unwrap(), expected);
}
