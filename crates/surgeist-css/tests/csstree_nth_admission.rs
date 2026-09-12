#![forbid(unsafe_code)]
//! Independent Syntax 3 An+B and Selectors 4 child-indexed expectations.
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#anb-type
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#nth-child-pseudo
use surgeist_css::{
    CssNamespaceConstraint, CssNamespaceContext, CssNamespaceName, CssNamespacePrefix,
    CssNthPattern, CssPseudoClass, CssRecoveryAction, CssSelector, parse_selector,
};

fn coefficients(pattern: CssNthPattern) -> (i32, i32) {
    match pattern {
        CssNthPattern::Odd => (2, 1),
        CssNthPattern::Even => (2, 0),
        CssNthPattern::Integer(value) => (0, value),
        CssNthPattern::AnPlusB(value) => (value.a(), value.b()),
        _ => panic!("unexpected nth pattern: {pattern:?}"),
    }
}

fn assert_type_li(selector: &CssSelector) {
    match selector {
        CssSelector::Tag(name) => assert_eq!(name, "li"),
        CssSelector::Compound(compound) => {
            let name = compound.type_selector().expect("type selector li");
            assert_eq!(name.local_name(), Some("li"));
            assert_eq!(name.namespace(), &CssNamespaceConstraint::Any);
            assert!(compound.ids().is_empty());
            assert!(compound.classes().is_empty());
            assert!(compound.attributes().is_empty());
            assert!(compound.pseudo_classes().is_empty());
            assert!(!compound.has_pseudo_elements());
        }
        _ => panic!("expected type selector li: {selector:?}"),
    }
}

#[test]
fn nth_corpus_preserves_token_grammar_and_typed_patterns() {
    let clean = [
        ("selector/Nth.json#/+n", "nth-last-child", 1, -2, false),
        (
            "selector/Nth.json#/big numbers",
            "nth-last-child",
            123456,
            -12345678,
            false,
        ),
        ("selector/Nth.json#/even keyword", "nth-child", 2, 0, false),
        (
            "selector/Nth.json#/even keyword should be case insensitive",
            "nth-child",
            2,
            0,
            false,
        ),
        ("selector/Nth.json#/nth-child", "nth-child", 2, 1, false),
        (
            "selector/Nth.json#/nth-child case insensetive",
            "nth-child",
            2,
            1,
            false,
        ),
        (
            "selector/Nth.json#/nth-last-child",
            "nth-last-child",
            2,
            1,
            false,
        ),
        (
            "selector/Nth.json#/nth-last-child case insensetive",
            "nth-last-child",
            2,
            1,
            false,
        ),
        (
            "selector/Nth.json#/nth-last-of-type",
            "nth-last-of-type",
            2,
            1,
            false,
        ),
        (
            "selector/Nth.json#/nth-last-of-type case insensetive",
            "nth-last-of-type",
            2,
            1,
            false,
        ),
        ("selector/Nth.json#/nth-of-type", "nth-of-type", 2, 1, false),
        (
            "selector/Nth.json#/nth-of-type case insensetive",
            "nth-of-type",
            2,
            1,
            false,
        ),
        ("selector/Nth.json#/nth.0", "nth-child", 0, 10, false),
        ("selector/Nth.json#/nth.1", "nth-child", 2, 0, false),
        ("selector/Nth.json#/nth.4", "nth-child", 1, 0, false),
        ("selector/Nth.json#/nth.5", "nth-child", -1, 0, false),
        (
            "selector/Nth.json#/nthselector.0 case insensitive",
            "nth-child",
            2,
            1,
            false,
        ),
        (
            "selector/Nth.json#/nthselector.1",
            "nth-last-child",
            3,
            -2,
            false,
        ),
        (
            "selector/Nth.json#/nthselector.c.0",
            "nth-child",
            2,
            1,
            false,
        ),
        (
            "selector/Nth.json#/nthselector.c.1",
            "nth-last-child",
            3,
            -2,
            false,
        ),
        (
            "selector/Nth.json#/nthselector.s.0",
            "nth-child",
            2,
            1,
            false,
        ),
        (
            "selector/Nth.json#/nthselector.s.1",
            "nth-last-child",
            3,
            -2,
            false,
        ),
        ("selector/Nth.json#/odd keyword", "nth-child", 2, 1, false),
        (
            "selector/Nth.json#/odd keyword be case insensitive",
            "nth-child",
            2,
            1,
            false,
        ),
        ("selector/Nth.json#/of clause", "nth-child", 2, 1, true),
    ];
    let rejected = [
        "selector/Nth.json#/error/0",
        "selector/Nth.json#/error/1",
        "selector/Nth.json#/error/10",
        "selector/Nth.json#/error/11",
        "selector/Nth.json#/error/12",
        "selector/Nth.json#/error/13",
        "selector/Nth.json#/error/14",
        "selector/Nth.json#/error/15",
        "selector/Nth.json#/error/16",
        "selector/Nth.json#/error/17",
        "selector/Nth.json#/error/18",
        "selector/Nth.json#/error/19",
        "selector/Nth.json#/error/2",
        "selector/Nth.json#/error/20",
        "selector/Nth.json#/error/21",
        "selector/Nth.json#/error/22",
        "selector/Nth.json#/error/23",
        "selector/Nth.json#/error/24",
        "selector/Nth.json#/error/25",
        "selector/Nth.json#/error/26",
        "selector/Nth.json#/error/27",
        "selector/Nth.json#/error/28",
        "selector/Nth.json#/error/29",
        "selector/Nth.json#/error/3",
        "selector/Nth.json#/error/30",
        "selector/Nth.json#/error/31",
        "selector/Nth.json#/error/32",
        "selector/Nth.json#/error/33",
        "selector/Nth.json#/error/34",
        "selector/Nth.json#/error/35",
        "selector/Nth.json#/error/36",
        "selector/Nth.json#/error/37",
        "selector/Nth.json#/error/38",
        "selector/Nth.json#/error/4",
        "selector/Nth.json#/error/5",
        "selector/Nth.json#/error/6",
        "selector/Nth.json#/error/7",
        "selector/Nth.json#/error/8",
        "selector/Nth.json#/error/9",
    ];
    let context = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("ns").unwrap()),
        CssNamespaceName::new("surgeist-corpus-probe"),
    )]);
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "corpus/csstree/expectations/selector/Nth.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), clean.len() + rejected.len());
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let input = case["input"].as_str().unwrap();
        let report = parse_selector(input, &context);
        if rejected.contains(&id) {
            assert!(report.syntax().is_none(), "{id}: {input}: {report:?}");
            assert!(!report.is_clean(), "{id}: {report:?}");
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::RejectInput }),
                "{id}: {report:?}"
            );
            continue;
        }
        let (_, expected_kind, a, b, has_filter) = clean
            .iter()
            .find(|(expected_id, ..)| *expected_id == id)
            .unwrap_or_else(|| panic!("missing independent expectation for {id}"));
        assert!(report.is_clean(), "{id}: {report:?}");
        let Some(CssSelector::PseudoClass(pseudo)) = report.syntax() else {
            panic!("expected typed nth pseudo-class for {id}: {report:?}");
        };
        let (kind, pattern, filter) = match pseudo {
            CssPseudoClass::NthChild(value) => {
                ("nth-child", value.pattern(), value.selector_list())
            }
            CssPseudoClass::NthLastChild(value) => {
                ("nth-last-child", value.pattern(), value.selector_list())
            }
            CssPseudoClass::NthOfType(value) => ("nth-of-type", *value, None),
            CssPseudoClass::NthLastOfType(value) => ("nth-last-of-type", *value, None),
            _ => panic!("wrong pseudo-class for {id}: {pseudo:?}"),
        };
        assert_eq!(kind, *expected_kind, "{id}");
        assert_eq!(coefficients(pattern), (*a, *b), "{id}");
        if *has_filter {
            let members = filter.expect("authored of selector list").selectors();
            let [first, second] = members else {
                panic!("two authored of-list members for {id}: {members:?}");
            };
            assert_type_li(first);
            assert_eq!(second, &CssSelector::Class("test".into()), "{id}");
        } else {
            assert!(filter.is_none(), "no authored of-list for {id}");
        }
    }
}
