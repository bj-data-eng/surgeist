#![forbid(unsafe_code)]
//! Intrinsic Namespaces 3 construction uses checked decoded identifiers and
//! literal strings. Authored coordinates exist only for parsed declarations.

use surgeist_css::{
    CssNamespaceContext, CssNamespaceName, CssNamespacePrefix, CssNamespaceRule, CssRule,
    parse_sheet,
};

#[test]
fn typed_namespace_construction_preserves_prefix_case_literal_names_and_absent_source() {
    for (prefix, name) in [
        (None, ""),
        (None, "not a URI"),
        (Some("svg"), "urn:svg"),
        (Some("SVG"), ""),
        (Some("1"), "relative/../name%2f"),
        (Some("a b"), "HTTP://EXAMPLE.TEST/../name"),
    ] {
        let checked_prefix = prefix.map(|value| CssNamespacePrefix::try_new(value).unwrap());
        let rule = CssNamespaceRule::new(checked_prefix, CssNamespaceName::new(name));
        assert_eq!(rule.prefix().map(CssNamespacePrefix::as_str), prefix);
        assert_eq!(rule.name().as_str(), name);
        assert_eq!(rule.position(), None);
    }
}

#[test]
fn constructed_bindings_distinguish_absent_defaults_from_declared_null_names() {
    let absent = CssNamespaceContext::default();
    assert!(absent.default_namespace().is_none());
    let default = CssNamespaceRule::new(None, CssNamespaceName::new(""));
    let named = CssNamespaceRule::new(
        Some(CssNamespacePrefix::try_new("svg").unwrap()),
        CssNamespaceName::new(""),
    );
    let context = CssNamespaceContext::from_bindings(
        [&default, &named]
            .into_iter()
            .map(|rule| (rule.prefix().cloned(), rule.name().clone())),
    );
    assert_eq!(context.default_namespace().unwrap().as_str(), "");
    assert_eq!(
        context
            .named_namespace(named.prefix().unwrap())
            .unwrap()
            .as_str(),
        ""
    );
    assert!(
        context
            .named_namespace(&CssNamespacePrefix::try_new("SVG").unwrap())
            .is_none()
    );
    assert!(absent.default_namespace().is_none());
}

#[test]
fn parsed_and_constructed_declarations_agree_semantically_without_fabricating_coordinates() {
    for (source, prefix, name) in [
        ("/*😀*/\n  @namespace '';", None, ""),
        (
            "/*😀*/\n  @namespace svg url(HTTP://EXAMPLE.TEST/a/../b%2f);",
            Some("svg"),
            "HTTP://EXAMPLE.TEST/a/../b%2f",
        ),
        (
            "/*😀*/\n  @namespace SVG 'not a URI';",
            Some("SVG"),
            "not a URI",
        ),
        (
            r"/*😀*/
  @namespace \31 'urn:digit';",
            Some("1"),
            "urn:digit",
        ),
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let [CssRule::Namespace(parsed)] = report.syntax().rules() else {
            panic!("one retained namespace declaration: {report:?}");
        };
        let constructed = CssNamespaceRule::new(
            prefix.map(|value| CssNamespacePrefix::try_new(value).unwrap()),
            CssNamespaceName::new(name),
        );
        assert_eq!(constructed.prefix(), parsed.prefix());
        assert_eq!(constructed.name(), parsed.name());
        assert_eq!(constructed.position(), None);
        let position = parsed.position().expect("original authored position");
        assert_eq!(position.byte_offset().value(), 11);
        assert_eq!(position.line().value(), 1);
        assert_eq!(position.column().value(), 2);
    }
}
