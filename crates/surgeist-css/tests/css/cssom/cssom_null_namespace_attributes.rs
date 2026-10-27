//! Test-only runtime checkpoint for #1020. Selected CSSOM20261009
//! #serialize-a-simple-selector attribute step2 emits a namespace separator
//! only for a nonnull namespace. Wildcard spelling and compact authored output
//! keep the previously accepted independent contracts.
use surgeist_css::CssParser;
use surgeist_css::rule::CssRule;
use surgeist_css::selector::CssNamespaceContext;

#[test]
fn literal_null_attribute_separator_is_omitted_without_changing_authored_identity() {
    let report = CssRule::parse(
        &CssParser::default(),
        "/*😀*/ [|a] {}",
        &CssNamespaceContext::default(),
    );
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let value = report.syntax().as_ref().unwrap();
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "[|a] { }"
    );
    assert_eq!(
        value.serialize_cssom(Default::default()).unwrap(),
        "[a] { }"
    );
    assert_eq!(report, before);
}

#[test]
fn null_attribute_names_inside_logical_arguments_keep_wildcard_and_flags_distinct() {
    let report = CssParser::default().parse_sheet("[|a='é' s]:is([|b], [*|c]) {}");
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    assert_eq!(
        report
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        "[|a=\"é\" s]:is([|b], [*|c]) { }"
    );
    assert_eq!(
        report.syntax().serialize_cssom(Default::default()).unwrap(),
        "[a=\"é\" s]:is([b], [*|c]) { }"
    );
    assert_eq!(report, before);
}

#[test]
fn unprefixed_attributes_do_not_inherit_default_namespace_and_any_remains_raw() {
    let report = CssParser::default().parse_sheet("@namespace 'urn:default'; [|a][a][*|a] {}");
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let CssRule::Style(_) = &report.syntax().rules()[1] else {
        panic!("style")
    };
    let value = &report.syntax().rules()[1];
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "[|a][a][*|a] { }"
    );
    assert_eq!(
        value.serialize_cssom(Default::default()).unwrap(),
        "[a][a][*|a] { }"
    );
    assert_eq!(report, before);
}
