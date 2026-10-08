#![forbid(unsafe_code)]

//! Cascade 5 CR 2022-01-13 §§6.4.2 and 6.4.4 retain relative layer names
//! and ordered nesting. Public normalization preserves authored occurrences
//! and their immediate parents without assigning effective cascade identities.

use surgeist_css::{
    CssErrorCode, CssLayerBlockRule, CssNormalizedItem, CssNormalizedSheet, CssRecoveryAction,
    CssRule, CssRuleContext, CssRuleContextKindRef, CssSelector, CssSourcePosition,
    CssStyleSelector, normalize_report, normalize_sheet, parse_sheet, validate_sheet,
};

fn position(actual: CssSourcePosition, byte: usize) {
    assert_eq!(actual.byte_offset().value(), byte);
    assert_eq!(actual.line().value(), 0);
    assert_eq!(actual.column().value() as usize, byte);
}

fn layer<'a>(rule: &'a CssRule, expected: Option<&[&str]>, byte: usize) -> &'a CssLayerBlockRule {
    let CssRule::LayerBlock(layer) = rule else {
        panic!("expected a retained layer block at byte {byte}");
    };
    let actual = layer.name().map(|name| {
        name.components()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    });
    assert_eq!(actual.as_deref(), expected);
    position(layer.position().expect("authored layer position"), byte);
    layer
}

fn empty_style(rule: &CssRule, class: &str, byte: usize) {
    let CssRule::Style(style) = rule else {
        panic!("expected an authored style at byte {byte}");
    };
    assert_eq!(
        style.selectors().selectors(),
        &[CssStyleSelector::Selector(CssSelector::Class(
            class.to_owned()
        ))],
    );
    assert!(style.declarations().is_empty());
    assert!(style.rules().is_empty());
    position(style.position(), byte);
}

fn contexts(sheet: &CssNormalizedSheet) -> Vec<&CssRuleContext> {
    sheet
        .items()
        .iter()
        .map(|item| match item {
            CssNormalizedItem::Rule(rule) => rule,
            _ => panic!("empty style bodies produce only rule occurrences"),
        })
        .collect()
}

fn normalized_layer(rule: &CssRuleContext, expected: Option<&[&str]>) {
    let CssRuleContextKindRef::LayerBlock(name) = rule.kind() else {
        panic!("expected a normalized layer header");
    };
    let actual = name.map(|name| {
        name.components()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    });
    assert_eq!(actual.as_deref(), expected);
}

#[test]
fn relative_named_layers_retain_order_and_distinct_anonymous_parent_occurrences() {
    const SOURCE: &str = concat!(
        "@layer{@layer shared{.a{}}}",
        "@layer{@layer shared{.b{}}}",
        "@layer named{@layer shared{.c{}}@layer shared{.d{}}}",
        "@layer named.shared{.e{}}",
    );
    let report = parse_sheet(SOURCE);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [first, second, named, dotted] = report.syntax().rules() else {
        panic!("four original outer layer occurrences in authored order");
    };
    let first = layer(first, None, 0);
    let [first_child] = first.rules() else {
        panic!("first named child")
    };
    let first_child = layer(first_child, Some(&["shared"]), 7);
    let [a] = first_child.rules() else {
        panic!("first original style")
    };
    empty_style(a, "a", 21);

    let second = layer(second, None, 27);
    let [second_child] = second.rules() else {
        panic!("second named child")
    };
    let second_child = layer(second_child, Some(&["shared"]), 34);
    let [b] = second_child.rules() else {
        panic!("second original style")
    };
    empty_style(b, "b", 48);

    let named = layer(named, Some(&["named"]), 54);
    let [third_child, fourth_child] = named.rules() else {
        panic!("repeated same-named children remain two authored occurrences");
    };
    let third_child = layer(third_child, Some(&["shared"]), 67);
    let [c] = third_child.rules() else {
        panic!("third original style")
    };
    empty_style(c, "c", 81);
    let fourth_child = layer(fourth_child, Some(&["shared"]), 86);
    let [d] = fourth_child.rules() else {
        panic!("fourth original style")
    };
    empty_style(d, "d", 100);

    let dotted = layer(dotted, Some(&["named", "shared"]), 106);
    let [e] = dotted.rules() else {
        panic!("external original style")
    };
    empty_style(e, "e", 126);

    let normalized = normalize_sheet(report.syntax()).expect("empty bodies normalize");
    let rules = contexts(&normalized);
    let bytes = [0, 7, 21, 27, 34, 48, 54, 67, 81, 86, 100, 106, 126];
    let parents = [
        None,
        Some(0),
        Some(1),
        None,
        Some(3),
        Some(4),
        None,
        Some(6),
        Some(7),
        Some(6),
        Some(9),
        None,
        Some(11),
    ];
    assert_eq!(rules.len(), bytes.len());
    for (index, (&byte, parent)) in bytes.iter().zip(parents).enumerate() {
        position(
            rules[index]
                .position()
                .expect("original normalized position"),
            byte,
        );
        match parent {
            Some(parent) => assert!(
                rules[index]
                    .parent()
                    .expect("immediate authored parent")
                    .same_context(rules[parent]),
            ),
            None => assert!(rules[index].parent().is_none()),
        }
    }
    for (index, name) in [
        (0, None),
        (1, Some(&["shared"][..])),
        (3, None),
        (4, Some(&["shared"][..])),
        (6, Some(&["named"][..])),
        (7, Some(&["shared"][..])),
        (9, Some(&["shared"][..])),
        (11, Some(&["named", "shared"][..])),
    ] {
        normalized_layer(rules[index], name);
    }
    for (index, class) in [(2, "a"), (5, "b"), (8, "c"), (10, "d"), (12, "e")] {
        let CssRuleContextKindRef::Style(selectors) = rules[index].kind() else {
            panic!("original style occurrence in depth-first order");
        };
        let [selector] = selectors.selectors() else {
            panic!("one original selector")
        };
        assert_eq!(selector.selector(), &CssSelector::Class(class.to_owned()));
    }

    assert!(!rules[0].same_context(rules[3]));
    assert!(!rules[1].same_context(rules[4]));
    assert!(
        !rules[1]
            .parent()
            .unwrap()
            .same_context(rules[4].parent().unwrap())
    );
    assert!(
        rules[7]
            .parent()
            .unwrap()
            .same_context(rules[9].parent().unwrap())
    );
    assert!(!rules[7].same_context(rules[9]));
    assert!(!rules[7].same_context(rules[11]));
    assert!(rules[11].parent().is_none());
}

#[test]
fn nested_import_is_dropped_without_erasing_later_layer_children_or_source_lineage() {
    const SOURCE: &str = "@layer host{@import \"lost.css\";@layer kept{.after{}}}.outside{}";
    let report = parse_sheet(SOURCE);
    assert!(!report.is_clean());
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejected nested import source unit");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePlacement
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    position(diagnostic.span().start(), 12);
    position(diagnostic.span().end(), 31);
    assert_eq!(
        validate_sheet(SOURCE)
            .expect_err("recovered sheet is not clean")
            .diagnostics(),
        report.diagnostics(),
    );
    let [host, outside] = report.syntax().rules() else {
        panic!("layer and outer sibling both survive");
    };
    let host = layer(host, Some(&["host"]), 0);
    let [kept] = host.rules() else {
        panic!("only the later valid child remains")
    };
    let kept = layer(kept, Some(&["kept"]), 31);
    let [after] = kept.rules() else {
        panic!("later child style survives")
    };
    empty_style(after, "after", 43);
    empty_style(outside, "outside", 53);

    let normalized = normalize_report(&report).expect("retained bodies normalize");
    assert!(!normalized.is_clean());
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    let rules = contexts(normalized.syntax());
    assert_eq!(rules.len(), 4);
    for (rule, byte) in rules.iter().zip([0, 31, 43, 53]) {
        position(rule.position().expect("original retained position"), byte);
    }
    normalized_layer(rules[0], Some(&["host"]));
    normalized_layer(rules[1], Some(&["kept"]));
    assert!(rules[0].parent().is_none());
    assert!(rules[1].parent().unwrap().same_context(rules[0]));
    assert!(rules[2].parent().unwrap().same_context(rules[1]));
    assert!(rules[3].parent().is_none());
    for (index, class) in [(2, "after"), (3, "outside")] {
        let CssRuleContextKindRef::Style(selectors) = rules[index].kind() else {
            panic!("retained original style");
        };
        let [selector] = selectors.selectors() else {
            panic!("one original selector")
        };
        assert_eq!(selector.selector(), &CssSelector::Class(class.to_owned()));
    }
}
