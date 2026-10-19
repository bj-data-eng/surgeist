#![forbid(unsafe_code)]
//! Namespaces 3 §4 and Selectors 4 §16: qualified attribute local names are
//! decoded identifiers, including values whose CSS spelling needs escapes.
//! https://www.w3.org/TR/2014/REC-css-namespaces-3-20140320/#css-qnames
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#grammar

use surgeist_css::{
    CssAttributeName, CssComponentValueRef, CssErrorCode, CssNamespaceConstraint,
    CssNamespaceContext, CssNamespaceName, CssNamespacePrefix, CssNormalizedItem,
    CssRecoveryAction, CssRule, CssRuleContextKindRef, CssSelector, CssValueTokenRef,
    normalize_report, parse_component_values, parse_selector, parse_sheet, validate_sheet,
};

fn assert_identifier_spelling_decodes_to(spelling: &str, decoded: &str) {
    let components = parse_component_values(spelling).expect("valid escaped identifier spelling");
    let [identifier] = components.items() else {
        panic!("one complete identifier token: {spelling}: {components:?}");
    };
    assert!(matches!(
        identifier.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident(value)) if value == decoded
    ));
}

fn assert_attribute_name(
    selector: &CssSelector,
    decoded: &str,
    namespace: &CssNamespaceConstraint,
) {
    let CssSelector::Compound(compound) = selector else {
        panic!("attribute compound retained: {selector:?}");
    };
    let [attribute] = compound.attributes() else {
        panic!("one retained attribute: {selector:?}");
    };
    assert_eq!(attribute.name().as_str(), decoded);
    assert_eq!(attribute.namespace(), namespace);
}

#[test]
fn decoded_attribute_name_construction_preserves_identifiers_requiring_escapes() {
    for (spelling, decoded) in [(r"\31 ", "1"), (r"a\ b", "a b")] {
        // Establish the normative IDENT condition before requesting construction
        // from the decoded value rather than from its escaped CSS spelling.
        assert_identifier_spelling_decodes_to(spelling, decoded);
        let name = CssAttributeName::try_new(decoded)
            .expect("decoded attribute identifier may require CSS escapes");
        assert_eq!(name.as_str(), decoded);
    }
}

#[test]
fn decoded_attribute_name_construction_rejects_empty_and_nul_without_changing_case() {
    assert!(CssAttributeName::try_new("").is_none());
    assert!(CssAttributeName::try_new("a\0b").is_none());
    for decoded in ["href", "HREF"] {
        assert_eq!(
            CssAttributeName::try_new(decoded).unwrap().as_str(),
            decoded
        );
    }
}

#[test]
fn escaped_attribute_local_names_parse_without_unwinding_in_each_namespace_form() {
    let prefix = CssNamespacePrefix::try_new("svg").unwrap();
    let context = CssNamespaceContext::from_bindings([(
        Some(prefix.clone()),
        CssNamespaceName::new("urn:svg"),
    )]);
    for (spelling, decoded) in [(r"\31 ", "1"), (r"a\ b", "a b")] {
        assert_identifier_spelling_decodes_to(spelling, decoded);
        for (qualifier, namespace) in [
            ("", CssNamespaceConstraint::ExplicitNone),
            ("|", CssNamespaceConstraint::ExplicitNone),
            ("*|", CssNamespaceConstraint::Any),
            ("svg|", CssNamespaceConstraint::Named(prefix.clone())),
        ] {
            let source = format!("[{qualifier}{spelling}].mark");
            let report = std::panic::catch_unwind(|| parse_selector(&source, &context))
                .expect("valid escaped attribute local identifier must not unwind");
            assert!(report.is_clean(), "{source}: {report:?}");
            assert_attribute_name(report.syntax().as_ref().unwrap(), decoded, &namespace);
            assert_eq!(
                report.clone().into_validation_result().unwrap(),
                *report.syntax()
            );
        }
    }
}

#[test]
fn escaped_attribute_names_survive_sheet_normalization_with_recovery_and_authored_origins() {
    let invalid = "[missing|name]{}";
    let prefix = CssNamespacePrefix::try_new("svg").unwrap();
    for (spelling, decoded) in [(r"\31 ", "1"), (r"a\ b", "a b")] {
        assert_identifier_spelling_decodes_to(spelling, decoded);
        let selector = format!("[svg|{spelling}].mark");
        let source = format!(
            "/*😀*/\r\n@namespace svg 'urn:svg';\r\n{invalid}\r\n  {selector}{{color:red}}"
        );
        let report = std::panic::catch_unwind(|| parse_sheet(&source))
            .expect("valid escaped local name must not unwind after an invalid sibling");
        let [CssRule::Namespace(_), CssRule::Style(style)] = report.syntax().rules() else {
            panic!("namespace and valid later style retained: {report:?}");
        };
        let authored_selector = style.selectors().selectors()[0].selector();
        assert_attribute_name(
            authored_selector,
            decoded,
            &CssNamespaceConstraint::Named(prefix.clone()),
        );
        let position = style.position();
        assert_eq!(
            position.byte_offset().value(),
            source.find(&selector).unwrap()
        );
        assert_eq!(position.line().value(), 3);
        assert_eq!(position.column().value(), 2);
        let [diagnostic] = report.diagnostics() else {
            panic!("only the undeclared-prefix sibling is diagnosed: {report:?}");
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
        let invalid_start = source.find(invalid).unwrap();
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            invalid_start
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            invalid_start + invalid.len()
        );
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );

        let normalized = normalize_report(&report).expect("retained attribute name normalizes");
        assert!(!normalized.is_clean());
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let (context, selectors) = normalized
            .syntax()
            .items()
            .iter()
            .find_map(|item| match item {
                CssNormalizedItem::Rule(context) => match context.kind() {
                    CssRuleContextKindRef::Style(selectors) => Some((context, selectors)),
                    _ => None,
                },
                _ => None,
            })
            .expect("retained style normalization occurrence");
        assert_eq!(context.position(), Some(position));
        assert_eq!(selectors.selectors().len(), 1);
        let normalized_selector = selectors.selectors()[0].selector();
        assert_eq!(normalized_selector, authored_selector);
        assert_attribute_name(
            normalized_selector,
            decoded,
            &CssNamespaceConstraint::Named(prefix.clone()),
        );
    }
}
