#![forbid(unsafe_code)]
//! Conditional 5 section 6.2 admits custom-property style features. CSS Syntax
//! section 2.1 decodes escapes into the identifier token's value exactly once.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#style-container
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#escaping
use surgeist_css::*;

fn style_query(authored: &str) -> CssContainerStyleQuery {
    let source = format!("@container style({authored}) {{ .x {{ color:red }} }}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("one retained container rule")
    };
    let CssContainerConditionKind::Style(query) =
        rule.prelude().entries()[0].query().unwrap().kind()
    else {
        panic!(
            "a recognized custom-property style query: {:?}",
            rule.prelude().entries()[0].query().unwrap()
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
        let query = style_query(authored);
        let CssContainerStyleQueryKind::Feature(CssContainerStyleFeature::Boolean(
            CssContainerStyleFeatureName::Custom(name),
        )) = query.kind()
        else {
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
        let query = style_query(&format!("{authored}: red"));
        let CssContainerStyleQueryKind::Feature(CssContainerStyleFeature::Plain {
            name: CssContainerStyleFeatureName::Custom(name),
            value,
        }) = query.kind()
        else {
            panic!("value query")
        };
        assert_eq!(name.as_str(), expected);
        assert_eq!(value.serialize().unwrap().as_css(), " red");
    }
}

#[test]
fn plain_style_names_preserve_case_and_value() {
    let query = style_query("--Theme: red");
    let CssContainerStyleQueryKind::Feature(CssContainerStyleFeature::Plain {
        name: CssContainerStyleFeatureName::Custom(name),
        value,
    }) = query.kind()
    else {
        panic!("value query")
    };
    assert_eq!(name.as_str(), "--Theme");
    assert_eq!(value.serialize().unwrap().as_css(), " red");
}
