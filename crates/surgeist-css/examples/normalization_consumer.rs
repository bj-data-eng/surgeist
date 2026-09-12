#![forbid(unsafe_code)]

//! Public-consumer assertions for immutable stylesheet normalization.
//!
//! The authored-order, shared-reference, atomic-failure, and resource-limit
//! expectations are Surgeist contracts. Shorthand values follow Box3 and
//! Backgrounds3; nesting bindings and nested declaration behavior follow the
//! selected 22 January 2026 Nesting draft, sections 3.1, 4, and 5:
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/
//! This consumer asserts the symbolic matching contract without implementing
//! selector matching, specificity evaluation, cascade, or variable resolution.
//! Resource budgets count visited rules, emitted declaration occurrences, and
//! emitted contribution members. Keyframe, page, and font declarations remain
//! intact inside terminal payloads and are not individually counted or expanded.
//! These bounds cover traversal and output counters, not total process memory
//! or payload bytes. Normalization errors and rule contexts expose optional
//! source positions; programmatic syntax must not invent authored coordinates.

use surgeist_css::{
    CssComponentValue, CssComponentValues, CssContributionValueRef, CssContributions,
    CssCustomPropertyDeclaredValue, CssCustomPropertyName, CssDeclaration, CssExpansion,
    CssExpansionErrorKind, CssGlobalKeyword, CssImportance, CssKnownProperty as Property,
    CssLength, CssLonghandContributions, CssLonghandValueRef, CssMediaConditionKind, CssMediaQuery,
    CssNormalizationErrorKind, CssNormalizationLimits, CssNormalizationResource,
    CssNormalizedDeclaration, CssNormalizedItem, CssNormalizedSheet, CssPropertyNameRef,
    CssRecoveryAction, CssRule, CssRuleContext, CssRuleContextKindRef, CssScopedRule, CssSelector,
    CssSelectorBinding, CssSelectorCombinator, CssStyleRule, CssValueOrigin, expand_declaration,
    normalize_report, normalize_sheet, normalize_sheet_with_limits, parse_component_values,
    parse_property_value, parse_sheet,
};

fn declaration_items(sheet: &CssNormalizedSheet) -> Vec<&CssNormalizedDeclaration> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(declaration) => Some(declaration),
            CssNormalizedItem::Rule(_) => None,
            other => panic!("unexpected normalized item: {other:?}"),
        })
        .collect()
}

fn rules(sheet: &CssNormalizedSheet) -> Vec<&CssRuleContext> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(rule) => Some(rule),
            CssNormalizedItem::Declaration(_) => None,
            other => panic!("unexpected normalized item: {other:?}"),
        })
        .collect()
}

fn style(rule: &CssRule) -> &CssStyleRule {
    match rule {
        CssRule::Style(style) => style,
        other => panic!("expected style rule: {other:?}"),
    }
}

fn completed(declaration: &CssNormalizedDeclaration) -> &CssLonghandContributions {
    match declaration.expansion() {
        CssExpansion::Contributions(CssContributions::Longhands(values)) => values,
        other => panic!("expected longhand contributions: {other:?}"),
    }
}

fn properties(values: &CssLonghandContributions) -> Vec<Property> {
    values
        .items()
        .iter()
        .map(|value| value.property())
        .collect()
}

fn margin_lengths(values: &CssLonghandContributions) -> Vec<&CssLength> {
    values
        .items()
        .iter()
        .map(|value| match value.value() {
            CssContributionValueRef::Ordinary(CssLonghandValueRef::MarginTop(value))
            | CssContributionValueRef::Ordinary(CssLonghandValueRef::MarginRight(value))
            | CssContributionValueRef::Ordinary(CssLonghandValueRef::MarginBottom(value))
            | CssContributionValueRef::Ordinary(CssLonghandValueRef::MarginLeft(value)) => value,
            other => panic!("expected an ordinary margin: {other:?}"),
        })
        .collect()
}

fn assert_occurrence(normalized: &CssNormalizedDeclaration, authored: &CssDeclaration) {
    assert!(normalized.source().same_occurrence(authored));
    assert_eq!(normalized.source().importance(), authored.importance());
    assert_eq!(normalized.source().parsed_name(), authored.parsed_name());
    assert_eq!(normalized.source().parsed_value(), authored.parsed_value());
    match normalized.expansion() {
        CssExpansion::Contributions(CssContributions::Longhands(values)) => {
            for value in values.items() {
                assert!(value.source().same_occurrence(authored));
            }
        }
        CssExpansion::Contributions(CssContributions::UniversalReset(value)) => {
            assert!(value.source().same_occurrence(authored));
        }
        CssExpansion::Contributions(CssContributions::Custom(value)) => {
            assert!(value.source().same_occurrence(authored));
            assert_eq!(Some(value.declaration()), authored.custom());
        }
        CssExpansion::Pending(value) => assert!(value.source().same_occurrence(authored)),
        other => panic!("unexpected expansion: {other:?}"),
    }
}

fn rule_name(rule: &CssRuleContext) -> &'static str {
    match rule.kind() {
        CssRuleContextKindRef::Style(_) => "style",
        CssRuleContextKindRef::ScopedStyle(_) => "scoped-style",
        CssRuleContextKindRef::NestedDeclarations(_) => "declarations",
        CssRuleContextKindRef::Media(_) => "media",
        CssRuleContextKindRef::Supports(_) => "supports",
        CssRuleContextKindRef::Container { .. } => "container",
        CssRuleContextKindRef::LayerBlock(_) => "layer",
        CssRuleContextKindRef::Scope { .. } => "scope",
        CssRuleContextKindRef::LayerStatement(_) => "layer-statement",
        CssRuleContextKindRef::Import(_) => "import",
        CssRuleContextKindRef::Namespace(_) => "namespace",
        CssRuleContextKindRef::FontFace(_) => "font-face",
        CssRuleContextKindRef::Keyframes(_) => "keyframes",
        CssRuleContextKindRef::CounterStyle(_) => "counter-style",
        CssRuleContextKindRef::Page(_) => "page",
        other => panic!("unexpected rule header: {other:?}"),
    }
}

fn ancestors(rule: &CssRuleContext) -> Vec<&'static str> {
    let mut result = Vec::new();
    let mut current = Some(rule);
    while let Some(rule) = current {
        result.push(rule_name(rule));
        current = rule.parent();
    }
    result.reverse();
    result
}

fn ordered_grouped_contributions() {
    let report = parse_sheet(".a, #b { margin:1px 2px; margin-left:3px; all:revert-layer }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let authored = style(&report.syntax().rules()[0]);
    let normalized = normalize_sheet(report.syntax()).expect("supported declarations");
    let [
        CssNormalizedItem::Rule(rule),
        CssNormalizedItem::Declaration(first),
        CssNormalizedItem::Declaration(second),
        CssNormalizedItem::Declaration(reset),
    ] = normalized.items()
    else {
        panic!("one rule and three declaration occurrences in authored order");
    };
    assert_eq!(rule_name(rule), "style");
    assert!(rule.parent().is_none());
    let CssRuleContextKindRef::Style(header_selectors) = rule.kind() else {
        unreachable!()
    };
    assert!(header_selectors.same_context(first.selector_context()));
    assert_eq!([first.order(), second.order(), reset.order()], [0, 1, 2]);
    assert!(first.rule_context().same_context(rule));
    assert!(second.rule_context().same_context(rule));
    assert!(
        first
            .selector_context()
            .same_context(second.selector_context())
    );
    assert_eq!(first.selector_context().selectors().len(), 2);
    assert_eq!(
        properties(completed(first)),
        [
            Property::MarginTop,
            Property::MarginRight,
            Property::MarginBottom,
            Property::MarginLeft,
        ]
    );
    let one = CssLength::try_px(1.0).unwrap();
    let two = CssLength::try_px(2.0).unwrap();
    assert_eq!(margin_lengths(completed(first)), [&one, &two, &one, &two]);
    assert_eq!(properties(completed(second)), [Property::MarginLeft]);
    assert_eq!(
        margin_lengths(completed(second)),
        [&CssLength::try_px(3.0).unwrap()]
    );
    let CssExpansion::Contributions(CssContributions::UniversalReset(value)) = reset.expansion()
    else {
        panic!("all remains symbolic")
    };
    assert_eq!(value.keyword(), CssGlobalKeyword::RevertLayer);
    assert!(value.excludes(CssPropertyNameRef::Known(Property::Direction)));
    for (actual, expected) in [first, second, reset]
        .into_iter()
        .zip(authored.declarations().iter())
    {
        assert_occurrence(actual, expected);
    }
    let empty_report = parse_sheet(".empty, #E {}");
    assert!(empty_report.is_clean(), "{:?}", empty_report.diagnostics());
    let empty_sheet = normalize_sheet(empty_report.syntax()).unwrap();
    let [CssNormalizedItem::Rule(empty_rule)] = empty_sheet.items() else {
        panic!("an empty style rule remains exactly one rule item")
    };
    assert!(declaration_items(&empty_sheet).is_empty());
    let CssRuleContextKindRef::Style(empty_selectors) = empty_rule.kind() else {
        panic!("empty style header preserves its selector context")
    };
    assert!(empty_selectors.parent().is_none());
    assert_eq!(empty_selectors.selectors().len(), 2);
    assert_eq!(
        empty_selectors.selectors()[0].selector(),
        &CssSelector::Class("empty".to_owned())
    );
    assert_eq!(
        empty_selectors.selectors()[1].selector(),
        &CssSelector::Key("E".to_owned())
    );
    for selector in empty_selectors.selectors() {
        assert_eq!(selector.binding(), CssSelectorBinding::Absolute);
    }
    println!("ordered grouped contributions: ok");
}

fn pending_shorthand_and_reentry() {
    let report = parse_sheet(".a { margin:var(--m)!important; margin-left:9px }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = declaration_items(&normalized);
    let CssExpansion::Pending(pending) = declarations[0].expansion() else {
        panic!("substitution must remain pending")
    };
    let authored = &style(&report.syntax().rules()[0]).declarations()[0];
    assert_occurrence(declarations[0], authored);
    assert_eq!(pending.source().importance(), CssImportance::Important);
    assert_eq!(
        pending
            .source()
            .value_components()
            .serialize()
            .unwrap()
            .as_css(),
        "var(--m)"
    );
    let replacement = parse_component_values("4px 5px").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("completed margin contributions")
    };
    let four = CssLength::try_px(4.0).unwrap();
    let five = CssLength::try_px(5.0).unwrap();
    assert_eq!(margin_lengths(&values), [&four, &five, &four, &five]);
    for value in values.items() {
        assert!(value.source().same_occurrence(authored));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert_eq!(
            value
                .replacement_components()
                .unwrap()
                .serialize()
                .unwrap()
                .as_css(),
            "4px 5px"
        );
        for (retained, supplied) in value
            .replacement_components()
            .unwrap()
            .items()
            .iter()
            .zip(replacement.items())
        {
            assert_eq!(retained.origin(), supplied.origin());
        }
    }
    let error = pending
        .reenter(parse_component_values("var(--still-pending)").unwrap())
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        CssExpansionErrorKind::ResidualSubstitution
    ));
    assert!(matches!(
        declarations[0].expansion(),
        CssExpansion::Pending(_)
    ));
    assert_eq!(declarations[1].order(), 1);
    println!("pending shorthand and reentry: ok");
}

fn custom_values_and_construction() {
    let report =
        parse_sheet(".a { --Theme:var(--fallback, 2px)!important; --reset:inherit; --empty: }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = declaration_items(&normalized);
    assert_eq!(declarations.len(), 3);
    for (index, expected_name) in ["--Theme", "--reset", "--empty"].into_iter().enumerate() {
        let CssExpansion::Contributions(CssContributions::Custom(value)) =
            declarations[index].expansion()
        else {
            panic!("custom declaration is a symbolic custom contribution")
        };
        assert_eq!(value.declaration().name().as_str(), expected_name);
        assert_occurrence(
            declarations[index],
            &style(&report.syntax().rules()[0]).declarations()[index],
        );
    }
    let CssExpansion::Contributions(CssContributions::Custom(global)) = declarations[1].expansion()
    else {
        unreachable!()
    };
    assert_eq!(
        global.declaration().value(),
        &CssCustomPropertyDeclaredValue::Global(CssGlobalKeyword::Inherit)
    );
    let parsed = parse_component_values("var(--fallback, 2px)").unwrap();
    let name = CssCustomPropertyName::try_new("--Theme").unwrap();
    let constructed = parse_property_value(
        CssPropertyNameRef::Custom(&name),
        parsed,
        CssImportance::Important,
    )
    .unwrap();
    assert!(constructed.position().is_none());
    let CssExpansion::Contributions(CssContributions::Custom(direct)) =
        expand_declaration(&constructed).unwrap()
    else {
        panic!("same expansion boundary for constructed declarations")
    };
    assert!(direct.source().same_occurrence(&constructed));
    assert_eq!(
        Some(direct.declaration()),
        declarations[0].source().custom()
    );
    let left = parse_component_values("2px").unwrap();
    let right = parse_component_values("3px").unwrap();
    let mixed = CssComponentValues::try_new(vec![
        left.items()[0].clone(),
        CssComponentValue::try_token(" ").unwrap(),
        right.items()[0].clone(),
    ])
    .unwrap();
    let constructed_margin = parse_property_value(
        CssPropertyNameRef::Known(Property::Margin),
        mixed,
        CssImportance::Normal,
    )
    .unwrap();
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&constructed_margin).unwrap()
    else {
        panic!("constructed margin contribution")
    };
    assert_eq!(
        margin_lengths(&values),
        [
            &CssLength::try_px(2.0).unwrap(),
            &CssLength::try_px(3.0).unwrap(),
            &CssLength::try_px(2.0).unwrap(),
            &CssLength::try_px(3.0).unwrap(),
        ]
    );
    assert!(constructed_margin.position().is_none());
    let origins = constructed_margin.value_components().items();
    assert!(matches!(origins[1].origin(), CssValueOrigin::Programmatic));
    assert_eq!(origins[0].origin(), left.items()[0].origin());
    assert_eq!(origins[2].origin(), right.items()[0].origin());
    let parsed_margin = parse_sheet(".a {margin:2px 3px}");
    assert!(
        parsed_margin.is_clean(),
        "{:?}",
        parsed_margin.diagnostics()
    );
    let normalized_margin = normalize_sheet(parsed_margin.syntax()).unwrap();
    let parsed_declarations = declaration_items(&normalized_margin);
    assert_eq!(
        parsed_declarations[0].source().body(),
        constructed_margin.body()
    );
    assert_eq!(
        properties(completed(parsed_declarations[0])),
        properties(&values)
    );
    assert_eq!(
        margin_lengths(completed(parsed_declarations[0])),
        margin_lengths(&values)
    );
    assert!(parsed_declarations[0].source().position().is_some());
    println!("custom values and construction: ok");
}

fn nested_declaration_context_identity() {
    let report = parse_sheet(".a, #b { margin:0; & .c { padding:1px } margin:2px }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let all_rules = rules(&normalized);
    assert_eq!(
        all_rules
            .iter()
            .map(|rule| rule_name(rule))
            .collect::<Vec<_>>(),
        ["style", "style", "declarations"]
    );
    let declarations = declaration_items(&normalized);
    assert_eq!(
        declarations
            .iter()
            .map(|declaration| declaration.order())
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let outer = declarations[0].selector_context();
    assert!(outer.same_context(declarations[2].selector_context()));
    assert!(!outer.same_context(declarations[1].selector_context()));
    assert!(
        declarations[1]
            .selector_context()
            .parent()
            .unwrap()
            .same_context(outer)
    );
    assert_eq!(
        declarations[1].selector_context().selectors()[0].binding(),
        CssSelectorBinding::ExplicitAnchors
    );
    assert!(declarations[2].rule_context().same_context(all_rules[2]));
    assert!(all_rules[1].parent().unwrap().same_context(all_rules[0]));
    assert!(all_rules[2].parent().unwrap().same_context(all_rules[0]));
    assert_eq!(outer.selectors().len(), 2);
    assert_eq!(
        outer.selectors()[0].selector(),
        &CssSelector::Class("a".to_owned())
    );
    assert_eq!(
        outer.selectors()[1].selector(),
        &CssSelector::Key("b".to_owned())
    );
    println!("nested declaration context identity: ok");
}

fn pseudo_element_context_preservation() {
    let report = parse_sheet(
        ".a, .a::before { margin:0; @media screen { margin:1px } & { margin:2px } margin:3px }",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = declaration_items(&normalized);
    assert_eq!(declarations.len(), 4);
    let parent = declarations[0].selector_context();
    assert!(parent.same_context(declarations[1].selector_context()));
    assert!(parent.same_context(declarations[3].selector_context()));
    assert_eq!(parent.selectors().len(), 2);
    assert!(!parent.selectors()[0].selector().has_pseudo_elements());
    assert!(parent.selectors()[1].selector().has_pseudo_elements());
    let explicit = declarations[2].selector_context();
    assert!(!explicit.same_context(parent));
    assert!(explicit.parent().unwrap().same_context(parent));
    assert_eq!(
        explicit.selectors()[0].binding(),
        CssSelectorBinding::ExplicitAnchors
    );
    assert_eq!(
        ancestors(declarations[1].rule_context()),
        ["style", "media", "declarations"]
    );
    println!("pseudo-element context preservation: ok");
}

fn nested_selector_binding_classification() {
    let report = parse_sheet(concat!(
        ".a, #b { ",
        "> .c {margin:1px} .c {margin:2px} .c & {margin:3px} ",
        "& + & {margin:4px} :is(&,.x) {margin:5px} ",
        ":where(&) {margin:6px} > :is(&,.x) {margin:7px} ",
        ":not(:is(&,.x)) {margin:8px} }",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = declaration_items(&normalized);
    let expected = [
        CssSelectorBinding::LeadingCombinator(CssSelectorCombinator::Child),
        CssSelectorBinding::ImplicitDescendant,
        CssSelectorBinding::ExplicitAnchors,
        CssSelectorBinding::ExplicitAnchors,
        CssSelectorBinding::ExplicitAnchors,
        CssSelectorBinding::ExplicitAnchors,
        CssSelectorBinding::LeadingCombinator(CssSelectorCombinator::Child),
        CssSelectorBinding::ExplicitAnchors,
    ];
    assert_eq!(declarations.len(), expected.len());
    for (declaration, expected) in declarations.iter().zip(expected) {
        let context = declaration.selector_context();
        assert_eq!(context.selectors().len(), 1);
        assert_eq!(context.selectors()[0].binding(), expected);
        assert_eq!(context.parent().unwrap().selectors().len(), 2);
    }
    let CssSelector::Complex(repeated) =
        declarations[3].selector_context().selectors()[0].selector()
    else {
        panic!("the two authored anchors stay in their two compounds")
    };
    assert_eq!(repeated.first().nesting_selectors(), 1);
    assert_eq!(repeated.rest()[0].selector().nesting_selectors(), 1);
    println!("nested selector binding classification: ok");
}

fn linear_selector_representation() {
    // Each level has four authored selectors and one child; the public stream
    // must retain those lists rather than enumerate their Cartesian product.
    let mut css = String::new();
    for depth in 0..8 {
        css.push_str(&format!(
            ".a{depth},.b{depth},#c{depth},.d{depth} {{ margin:0; "
        ));
    }
    css.push_str(&"}".repeat(8));
    let report = parse_sheet(&css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(normalized.items().len(), 16);
    let declarations = declaration_items(&normalized);
    assert_eq!(declarations.len(), 8);
    for (index, declaration) in declarations.iter().enumerate() {
        assert_eq!(declaration.order(), index);
        assert_eq!(declaration.selector_context().selectors().len(), 4);
        assert_eq!(completed(declaration).items().len(), 4);
        if index == 0 {
            assert!(declaration.selector_context().parent().is_none());
        } else {
            assert!(
                declaration
                    .selector_context()
                    .parent()
                    .unwrap()
                    .same_context(declarations[index - 1].selector_context())
            );
        }
    }
    let repeated = parse_sheet(".same {margin:0} .same {margin:0}");
    let normalized = normalize_sheet(repeated.syntax()).unwrap();
    let occurrences = declaration_items(&normalized);
    assert!(
        !occurrences[0]
            .selector_context()
            .same_context(occurrences[1].selector_context())
    );
    assert!(
        !occurrences[0]
            .rule_context()
            .same_context(occurrences[1].rule_context())
    );
    assert!(
        !occurrences[0]
            .source()
            .same_occurrence(occurrences[1].source())
    );
    println!("linear selector representation: ok");
}

fn symbolic_conditional_and_layer_contexts() {
    let report = parse_sheet(concat!(
        "@layer theme { @media screen { @supports (display:grid) { ",
        "@container sidebar (inline-size > 30rem) { .a { margin:1px; ",
        "@media print { padding:2px } margin:3px } } } } }",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = declaration_items(&normalized);
    assert_eq!(declarations.len(), 3);
    assert_eq!(
        ancestors(declarations[0].rule_context()),
        ["layer", "media", "supports", "container", "style"]
    );
    assert_eq!(
        ancestors(declarations[1].rule_context()),
        [
            "layer",
            "media",
            "supports",
            "container",
            "style",
            "media",
            "declarations"
        ]
    );
    assert_eq!(
        ancestors(declarations[2].rule_context()),
        [
            "layer",
            "media",
            "supports",
            "container",
            "style",
            "declarations"
        ]
    );
    assert!(
        declarations[0]
            .selector_context()
            .same_context(declarations[1].selector_context())
    );
    assert!(
        declarations[0]
            .selector_context()
            .same_context(declarations[2].selector_context())
    );
    let [CssRule::LayerBlock(layer)] = report.syntax().rules() else {
        unreachable!()
    };
    let [CssRule::Media(media)] = layer.rules() else {
        unreachable!()
    };
    let [CssRule::Supports(supports)] = media.rules() else {
        unreachable!()
    };
    let [CssRule::Container(container)] = supports.rules() else {
        unreachable!()
    };
    let contexts = rules(&normalized);
    let CssRuleContextKindRef::LayerBlock(name) = contexts[0].kind() else {
        unreachable!()
    };
    assert_eq!(name, layer.name());
    let CssRuleContextKindRef::Media(query) = contexts[1].kind() else {
        unreachable!()
    };
    assert_eq!(query, media.query());
    let CssRuleContextKindRef::Supports(condition) = contexts[2].kind() else {
        unreachable!()
    };
    assert_eq!(condition, supports.condition());
    let CssRuleContextKindRef::Container { name, condition } = contexts[3].kind() else {
        unreachable!()
    };
    assert_eq!(name, container.name());
    assert_eq!(condition, container.condition());
    println!("symbolic conditional and layer contexts: ok");
}

fn ordered_terminal_payloads() {
    let report = parse_sheet(concat!(
        "@charset \"UTF-8\"; ",
        "@layer reset, theme; ",
        "@import url(theme.css) layer(theme) supports(display:grid) print; ",
        "@namespace svg \"urn:svg\"; ",
        "@layer theme {} @layer {} @layer {} ",
        "@font-face {font-family:\"A\";src:local(\"A\"),url(a.woff2) format(woff2)} ",
        "@keyframes pulse {from {opacity:0} to {opacity:1}} ",
        "@counter-style marks {system:cyclic;symbols:x y} @page {margin:1cm} ",
        ".a {margin:0}",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(normalized.encoding(), report.syntax().encoding());
    assert_eq!(normalized.encoding().unwrap().label(), "UTF-8");
    let contexts = rules(&normalized);
    assert_eq!(
        contexts
            .iter()
            .map(|rule| rule_name(rule))
            .collect::<Vec<_>>(),
        [
            "layer-statement",
            "import",
            "namespace",
            "layer",
            "layer",
            "layer",
            "font-face",
            "keyframes",
            "counter-style",
            "page",
            "style",
        ]
    );
    assert_eq!(
        declaration_items(&normalized).len(),
        1,
        "nonstyle declarations stay in their payloads"
    );
    for (context, authored) in contexts.iter().zip(report.syntax().rules()) {
        assert!(context.parent().is_none());
        match (context.kind(), authored) {
            (CssRuleContextKindRef::LayerStatement(names), CssRule::LayerStatement(rule)) => {
                assert_eq!(names, rule.names())
            }
            (CssRuleContextKindRef::Import(value), CssRule::Import(rule)) => {
                assert_eq!(value, rule)
            }
            (CssRuleContextKindRef::Namespace(value), CssRule::Namespace(rule)) => {
                assert_eq!(value, rule)
            }
            (CssRuleContextKindRef::LayerBlock(name), CssRule::LayerBlock(rule)) => {
                assert_eq!(name, rule.name())
            }
            (CssRuleContextKindRef::FontFace(value), CssRule::FontFace(rule)) => {
                assert_eq!(value, rule)
            }
            (CssRuleContextKindRef::Keyframes(value), CssRule::Keyframes(rule)) => {
                assert_eq!(value, rule)
            }
            (CssRuleContextKindRef::CounterStyle(value), CssRule::CounterStyle(rule)) => {
                assert_eq!(value, rule)
            }
            (CssRuleContextKindRef::Page(value), CssRule::Page(rule)) => assert_eq!(value, rule),
            (CssRuleContextKindRef::Style(_), CssRule::Style(_)) => {}
            other => panic!("terminal payload/order mismatch: {other:?}"),
        }
    }
    assert!(
        !contexts[4].same_context(contexts[5]),
        "anonymous layers retain distinct occurrences"
    );
    let CssRuleContextKindRef::FontFace(face) = contexts[6].kind() else {
        unreachable!()
    };
    assert_eq!(face.descriptors().src().unwrap().sources().len(), 2);
    println!("ordered terminal payloads: ok");
}

fn atomic_unsupported_declaration() {
    let css = ".a { margin:0; @media screen { width:1px; padding:1px } }";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let error = normalize_sheet(report.syntax()).expect_err("width expansion is not yet selected");
    let CssNormalizationErrorKind::UnsupportedDeclaration(expansion) = error.kind() else {
        panic!("typed expansion capability failure: {error:?}")
    };
    assert_eq!(
        expansion.kind(),
        &CssExpansionErrorKind::UnsupportedProperty(Property::Width)
    );
    let [CssRule::Media(media)] = style(&report.syntax().rules()[0]).rules() else {
        unreachable!()
    };
    let [CssRule::NestedDeclarations(run)] = media.rules() else {
        unreachable!()
    };
    assert!(
        error
            .declaration()
            .unwrap()
            .same_occurrence(&run.declarations()[0])
    );
    assert_eq!(error.declaration_order(), Some(1));
    assert_eq!(
        error.position().unwrap().byte_offset().value(),
        css.find("width:").unwrap()
    );
    assert_eq!(
        ancestors(error.rule_context().unwrap()),
        ["style", "media", "declarations"]
    );
    assert_eq!(
        report, before,
        "failed normalization cannot change authored syntax or diagnostics"
    );
    let again =
        normalize_report(&report).expect_err("report entry uses the same capability boundary");
    assert_eq!(again.declaration_order(), Some(1));
    assert!(
        again
            .declaration()
            .unwrap()
            .same_occurrence(error.declaration().unwrap())
    );
    println!("atomic unsupported declaration: ok");
}

fn normalization_resource_boundaries() {
    let css = ".a { margin:0; .b { padding:1px } }";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let exact = CssNormalizationLimits::try_new(1, 2, 2, 8).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    assert_eq!(normalized.items().len(), 4);
    assert!(CssNormalizationLimits::try_new(257, 2, 2, 8).is_none());
    for (limits, resource, limit, position, ordinal) in [
        (
            CssNormalizationLimits::try_new(0, 2, 2, 8).unwrap(),
            CssNormalizationResource::RuleDepth,
            0,
            css.find(".b").unwrap(),
            None,
        ),
        (
            CssNormalizationLimits::try_new(1, 1, 2, 8).unwrap(),
            CssNormalizationResource::Rules,
            1,
            css.find(".b").unwrap(),
            None,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 1, 8).unwrap(),
            CssNormalizationResource::Declarations,
            1,
            css.find("padding:").unwrap(),
            Some(1),
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 2, 7).unwrap(),
            CssNormalizationResource::Contributions,
            7,
            css.find("padding:").unwrap(),
            Some(1),
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 2, 3).unwrap(),
            CssNormalizationResource::Contributions,
            3,
            css.find("margin:").unwrap(),
            Some(0),
        ),
    ] {
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert!(
            matches!(error.kind(), CssNormalizationErrorKind::LimitExceeded { resource: actual, limit: actual_limit } if *actual == resource && *actual_limit == limit)
        );
        assert_eq!(error.position().unwrap().byte_offset().value(), position);
        assert_eq!(error.declaration_order(), ordinal);
        assert_eq!(error.declaration().is_some(), ordinal.is_some());
    }
    let empty = parse_sheet("");
    assert!(
        normalize_sheet_with_limits(
            empty.syntax(),
            CssNormalizationLimits::try_new(0, 0, 0, 0).unwrap()
        )
        .unwrap()
        .items()
        .is_empty()
    );
    let empty_rule = parse_sheet(".a {}");
    assert!(
        normalize_sheet_with_limits(
            empty_rule.syntax(),
            CssNormalizationLimits::try_new(0, 1, 0, 0).unwrap()
        )
        .is_ok()
    );
    let nested_run = parse_sheet(".a {margin:0; @media screen {padding:1px}}");
    assert!(nested_run.is_clean(), "{:?}", nested_run.diagnostics());
    assert!(
        normalize_sheet_with_limits(
            nested_run.syntax(),
            CssNormalizationLimits::try_new(2, 3, 2, 8).unwrap()
        )
        .is_ok()
    );
    let error = normalize_sheet_with_limits(
        nested_run.syntax(),
        CssNormalizationLimits::try_new(2, 2, 2, 8).unwrap(),
    )
    .unwrap_err();
    assert!(matches!(
        error.kind(),
        CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Rules,
            limit: 2
        }
    ));
    assert_eq!(
        error.declaration_order(),
        None,
        "admitting a nested-declaration rule precedes admitting its declaration"
    );
    let symbolic = parse_sheet(".a {--x:var(--y);all:unset;margin:var(--m)}");
    assert!(symbolic.is_clean(), "{:?}", symbolic.diagnostics());
    assert_eq!(
        declaration_items(
            &normalize_sheet_with_limits(
                symbolic.syntax(),
                CssNormalizationLimits::try_new(0, 1, 3, 3).unwrap()
            )
            .unwrap()
        )
        .len(),
        3
    );
    let error = normalize_sheet_with_limits(
        symbolic.syntax(),
        CssNormalizationLimits::try_new(0, 1, 3, 2).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.declaration_order(), Some(2));
    assert!(matches!(
        error.kind(),
        CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 2
        }
    ));
    println!("normalization resource boundaries: ok");
}

fn unchanged_recovery_diagnostics() {
    let report = parse_sheet(
        ".a {margin:0; padding:nope; @media (width:) and {padding:1px} margin-left:2px}",
    );
    assert!(!report.is_clean());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::ReplaceMediaQueryWithNever)
    );
    let normalized = normalize_report(&report).unwrap();
    assert!(!normalized.is_clean());
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    assert_eq!(declaration_items(normalized.syntax()).len(), 3);
    let media = rules(normalized.syntax())
        .into_iter()
        .find(|rule| matches!(rule.kind(), CssRuleContextKindRef::Media(_)))
        .unwrap();
    let CssRuleContextKindRef::Media(query) = media.kind() else {
        unreachable!()
    };
    assert!(matches!(query.queries(), [CssMediaQuery::Never(_)]));
    let clean = parse_sheet(".a {margin:0}");
    let normalized_clean = normalize_report(&clean).unwrap();
    assert!(normalized_clean.is_clean());
    assert!(normalized_clean.diagnostics().is_empty());
    assert_eq!(declaration_items(normalized_clean.syntax()).len(), 1);
    let opaque = parse_sheet(".a {@media (width:) {padding:1px}}");
    assert!(opaque.is_clean(), "{:?}", opaque.diagnostics());
    let normalized_opaque = normalize_report(&opaque).unwrap();
    assert!(normalized_opaque.is_clean());
    assert_eq!(normalized_opaque.diagnostics(), opaque.diagnostics());
    assert_eq!(declaration_items(normalized_opaque.syntax()).len(), 1);
    let opaque_media = rules(normalized_opaque.syntax())
        .into_iter()
        .find(|rule| matches!(rule.kind(), CssRuleContextKindRef::Media(_)))
        .unwrap();
    let CssRuleContextKindRef::Media(query) = opaque_media.kind() else {
        unreachable!()
    };
    let [CssMediaQuery::Condition(condition)] = query.queries() else {
        panic!("expected retained opaque media condition")
    };
    let CssMediaConditionKind::GeneralEnclosed(enclosed) = condition.kind() else {
        panic!("expected balanced general enclosure")
    };
    assert_eq!(enclosed.authored(), Some("(width:)"));
    println!("unchanged recovery diagnostics: ok");
}

fn complete_scoped_rule_traversal() {
    let report = parse_sheet(concat!(
        "@scope (.root) to (.stop) { @layer reset; @media screen { ",
        "@supports (display:grid) { @container box (inline-size > 30rem) { ",
        "@layer theme { @scope (.inner) { ",
        ".a, > .b {margin:1px; & .child {padding:2px} margin:3px} ",
        "&:hover {margin:4px} } } } } } }",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let contexts = rules(&normalized);
    assert_eq!(
        contexts
            .iter()
            .map(|rule| rule_name(rule))
            .collect::<Vec<_>>(),
        [
            "scope",
            "layer-statement",
            "media",
            "supports",
            "container",
            "layer",
            "scope",
            "scoped-style",
            "style",
            "declarations",
            "scoped-style",
        ]
    );
    let declarations = declaration_items(&normalized);
    assert_eq!(declarations.len(), 4);
    assert_eq!(
        declarations
            .iter()
            .map(|declaration| declaration.order())
            .collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    assert_eq!(
        ancestors(declarations[0].rule_context()),
        [
            "scope",
            "media",
            "supports",
            "container",
            "layer",
            "scope",
            "scoped-style"
        ]
    );
    let scoped = declarations[0].selector_context();
    let CssRuleContextKindRef::ScopedStyle(header_selectors) = contexts[7].kind() else {
        panic!("scoped style header exposes its matching context")
    };
    assert!(header_selectors.same_context(scoped));
    let CssRuleContextKindRef::NestedDeclarations(run_selectors) = contexts[9].kind() else {
        panic!("nested declaration header exposes the inherited matching context")
    };
    assert!(run_selectors.same_context(scoped));
    assert!(run_selectors.same_context(declarations[2].selector_context()));
    assert!(
        scoped.parent().is_none(),
        "a scoped style starts its own style-parent binding"
    );
    assert!(scoped.scope_context().unwrap().same_context(contexts[6]));
    assert!(scoped.same_context(declarations[2].selector_context()));
    assert_eq!(
        scoped.selectors()[0].binding(),
        CssSelectorBinding::Absolute
    );
    assert_eq!(
        scoped.selectors()[1].binding(),
        CssSelectorBinding::LeadingCombinator(CssSelectorCombinator::Child)
    );
    let child = declarations[1].selector_context();
    assert!(child.parent().unwrap().same_context(scoped));
    assert!(child.scope_context().unwrap().same_context(contexts[6]));
    assert_eq!(
        child.selectors()[0].binding(),
        CssSelectorBinding::ExplicitAnchors
    );
    let scope_anchor = declarations[3].selector_context();
    assert!(scope_anchor.parent().is_none());
    assert!(
        scope_anchor
            .scope_context()
            .unwrap()
            .same_context(contexts[6])
    );
    assert_eq!(
        scope_anchor.selectors()[0].binding(),
        CssSelectorBinding::ScopeAnchors
    );
    let CssSelector::Compound(selector) = scope_anchor.selectors()[0].selector() else {
        unreachable!()
    };
    assert!(selector.has_scope_anchor());
    assert_eq!(selector.nesting_selectors(), 0);
    let [CssRule::Scope(authored_scope)] = report.syntax().rules() else {
        unreachable!()
    };
    let CssRuleContextKindRef::Scope { root, limit } = contexts[0].kind() else {
        unreachable!()
    };
    assert_eq!(root, authored_scope.root());
    assert_eq!(limit, authored_scope.limit());
    let [
        CssScopedRule::LayerStatement(statement),
        CssScopedRule::Media(media),
    ] = authored_scope.rules().rules()
    else {
        unreachable!()
    };
    let CssRuleContextKindRef::LayerStatement(names) = contexts[1].kind() else {
        unreachable!()
    };
    assert_eq!(names, statement.names());
    let CssRuleContextKindRef::Media(query) = contexts[2].kind() else {
        unreachable!()
    };
    assert_eq!(query, media.query());
    println!("complete scoped rule traversal: ok");
}

fn main() {
    ordered_grouped_contributions();
    pending_shorthand_and_reentry();
    custom_values_and_construction();
    nested_declaration_context_identity();
    pseudo_element_context_preservation();
    nested_selector_binding_classification();
    linear_selector_representation();
    symbolic_conditional_and_layer_contexts();
    ordered_terminal_payloads();
    atomic_unsupported_declaration();
    normalization_resource_boundaries();
    unchanged_recovery_diagnostics();
    complete_scoped_rule_traversal();
}
