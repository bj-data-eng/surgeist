#![forbid(unsafe_code)]
//! Conditional 5 section 6.2 admits custom-property style features. CSS Syntax
//! section 2.1 decodes escapes into the identifier token's value exactly once.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#style-container
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#escaping
use surgeist_css::{CssContainerConditionKind, CssContainerStyleQuery, CssRule, parse_sheet};

fn style_query(authored: &str) -> CssContainerStyleQuery {
    let source = format!("@container style({authored}) {{ .x {{ color:red }} }}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("one retained container rule")
    };
    let CssContainerConditionKind::Style(query) = rule.condition().kind() else {
        panic!(
            "a recognized custom-property style query: {:?}",
            rule.condition()
        )
    };
    query.clone()
}

#[test]
fn escaped_style_presence_names_retain_the_decoded_identifier() {
    for (authored, expected) in [
        (r"--\3a tone", "--:tone"),
        (r"--a\20 b", "--a b"),
        (r"--a\5c b", r"--a\b"),
    ] {
        let CssContainerStyleQuery::CustomPropertyPresence(name) = style_query(authored) else {
            panic!("presence query")
        };
        assert_eq!(name.as_str(), expected);
    }
}

#[test]
fn escaped_style_value_names_are_not_tokenized_twice() {
    for (authored, expected) in [
        (r"--\3a tone", "--:tone"),
        (r"--a\20 b", "--a b"),
        (r"--a\5c b", r"--a\b"),
    ] {
        let CssContainerStyleQuery::CustomPropertyValue { name, value } =
            style_query(&format!("{authored}: red"))
        else {
            panic!("value query")
        };
        assert_eq!(name.as_str(), expected);
        assert_eq!(value.as_css(), "red");
    }
}

#[test]
fn plain_style_names_preserve_case_and_value() {
    let CssContainerStyleQuery::CustomPropertyValue { name, value } = style_query("--Theme: red")
    else {
        panic!("value query")
    };
    assert_eq!(name.as_str(), "--Theme");
    assert_eq!(value.as_css(), "red");
}
