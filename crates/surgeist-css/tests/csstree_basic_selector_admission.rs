//! Authored admission independently derived from the pinned selector grammar.
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#grammar
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#attrnmsp
//! The fixture context binds only `ns`; `a` and `xlink` are undeclared.
use surgeist_css::{
    CssNamespaceContext, CssNamespaceName, CssNamespacePrefix, CssRecoveryAction, parse_selector,
};

#[test]
fn basic_selector_authored_admission_matches_selected_context() {
    let rejected = [
        "selector/AttributeSelector.json#/attrib with namespace and dashmatch",
        "selector/AttributeSelector.json#/attrib with namespace, dashmatch and spaces",
        "selector/AttributeSelector.json#/attrib.2 with flags",
        "selector/AttributeSelector.json#/namespace",
        "selector/AttributeSelector.json#/namespace w~1o attrselector",
        "selector/AttributeSelector.json#/error/0",
        "selector/AttributeSelector.json#/error/1",
        "selector/AttributeSelector.json#/error/10",
        "selector/AttributeSelector.json#/error/11",
        "selector/AttributeSelector.json#/error/12",
        "selector/AttributeSelector.json#/error/13",
        "selector/AttributeSelector.json#/error/14",
        "selector/AttributeSelector.json#/error/2",
        "selector/AttributeSelector.json#/error/3",
        "selector/AttributeSelector.json#/error/4",
        "selector/AttributeSelector.json#/error/5",
        "selector/AttributeSelector.json#/error/6",
        "selector/AttributeSelector.json#/error/7",
        "selector/AttributeSelector.json#/error/8",
        "selector/AttributeSelector.json#/error/9",
        "selector/TypeSelector.json#/error/0",
        "selector/TypeSelector.json#/error/1",
    ];
    let context = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("ns").unwrap()),
        CssNamespaceName::new("surgeist-corpus-probe"),
    )]);
    let mut failures = Vec::new();
    let mut case_count = 0;
    for fixture in [
        include_str!("corpus/csstree/expectations/selector/AttributeSelector.json"),
        include_str!("corpus/csstree/expectations/selector/ClassSelector.json"),
        include_str!("corpus/csstree/expectations/selector/IdSelector.json"),
        include_str!("corpus/csstree/expectations/selector/TypeSelector.json"),
    ] {
        let fixture: serde_json::Value = serde_json::from_str(fixture).unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            case_count += 1;
            let id = case["id"].as_str().unwrap();
            let input = case["input"].as_str().unwrap();
            let report = parse_selector(input, &context);
            let valid = if rejected.contains(&id) {
                report.syntax().is_none()
                    && !report.is_clean()
                    && report
                        .diagnostics()
                        .iter()
                        .any(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
            } else {
                report.syntax().is_some() && report.is_clean()
            };
            if !valid {
                failures.push(format!("{id}: {report:?}"));
            }
        }
    }
    assert_eq!(case_count, 87, "the pinned four-fixture inventory");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
