#![forbid(unsafe_code)]
//! Test-only runtime checkpoint for #1020. Selected CSSOM20261009
//! #serialize-a-simple-selector attribute step2 emits a namespace separator
//! only for a nonnull namespace. Wildcard spelling and compact authored output
//! keep the previously accepted independent contracts.
use surgeist_css::{CssNamespaceContext, CssRule, parse_rule, parse_sheet};

#[test]
fn literal_null_attribute_separator_is_omitted_without_changing_authored_identity() {
    let report = parse_rule("/*😀*/ [|a] {}", &CssNamespaceContext::default());
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let value = report.syntax().as_ref().unwrap();
    assert_eq!(value.to_specified_css().unwrap(), "[|a] { }");
    assert_eq!(value.serialize_cssom().unwrap(), "[a] { }");
    assert_eq!(report, before);
}

#[test]
fn null_attribute_names_inside_logical_arguments_keep_wildcard_and_flags_distinct() {
    let report = parse_sheet("[|a='é' s]:is([|b], [*|c]) {}");
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        "[|a=\"é\" s]:is([|b], [*|c]) { }"
    );
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        "[a=\"é\" s]:is([b], [*|c]) { }"
    );
    assert_eq!(report, before);
}

#[test]
fn unprefixed_attributes_do_not_inherit_default_namespace_and_any_remains_raw() {
    let report = parse_sheet("@namespace 'urn:default'; [|a][a][*|a] {}");
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let CssRule::Style(_) = &report.syntax().rules()[1] else {
        panic!("style")
    };
    let value = &report.syntax().rules()[1];
    assert_eq!(value.to_specified_css().unwrap(), "[|a][a][*|a] { }");
    assert_eq!(value.serialize_cssom().unwrap(), "[a][a][*|a] { }");
    assert_eq!(report, before);
}
