#![forbid(unsafe_code)]
//! Namespaces 3 §4: authored prefixes, context-dependent constraints and bounded
//! name serialization preserve decoded identifier identity.

use surgeist_css::{
    CssAttributeName, CssComponentValueErrorKind, CssIdent, CssNamespaceConstraint,
    CssNamespaceContext, CssNamespaceName, CssNamespacePrefix, CssQualifiedAttributeName,
    CssQualifiedNameError, CssQualifiedNamePrefix, CssQualifiedSelectorName, CssSelector,
    CssSerializedOrigin, CssValueOrigin, parse_selector,
};

fn context(default: Option<&str>) -> CssNamespaceContext {
    CssNamespaceContext::from_bindings(
        default
            .map(|name| (None, CssNamespaceName::new(name)))
            .into_iter()
            .chain(["svg", "1", "a b"].map(|prefix| {
                (
                    Some(CssNamespacePrefix::try_new(prefix).unwrap()),
                    CssNamespaceName::new("literal non-URI name"),
                )
            })),
    )
}

fn prefixes() -> Vec<CssQualifiedNamePrefix> {
    vec![
        CssQualifiedNamePrefix::Unqualified,
        CssQualifiedNamePrefix::ExplicitNone,
        CssQualifiedNamePrefix::Any,
        CssQualifiedNamePrefix::Named(CssNamespacePrefix::try_new("svg").unwrap()),
        CssQualifiedNamePrefix::Named(CssNamespacePrefix::try_new("1").unwrap()),
        CssQualifiedNamePrefix::Named(CssNamespacePrefix::try_new("a b").unwrap()),
    ]
}

#[test]
fn checked_names_keep_prefix_forms_separate_from_context_constraints() {
    for default in [None, Some(""), Some("urn:default")] {
        let context = context(default);
        for prefix in prefixes() {
            let selector = CssQualifiedSelectorName::try_new(
                prefix.clone(),
                CssIdent::try_new("Leaf").unwrap(),
                &context,
            )
            .unwrap();
            let universal =
                CssQualifiedSelectorName::try_universal(prefix.clone(), &context).unwrap();
            let attribute = CssQualifiedAttributeName::try_new(
                prefix.clone(),
                CssAttributeName::try_new("HREF").unwrap(),
                &context,
            )
            .unwrap();
            let type_constraint = match &prefix {
                CssQualifiedNamePrefix::Unqualified if default.is_some() => {
                    CssNamespaceConstraint::Default
                }
                CssQualifiedNamePrefix::Unqualified | CssQualifiedNamePrefix::Any => {
                    CssNamespaceConstraint::Any
                }
                CssQualifiedNamePrefix::ExplicitNone => CssNamespaceConstraint::ExplicitNone,
                CssQualifiedNamePrefix::Named(prefix) => {
                    CssNamespaceConstraint::Named(prefix.clone())
                }
                _ => unreachable!("known authored forms"),
            };
            assert_eq!(selector.prefix(), &prefix);
            assert_eq!(universal.prefix(), &prefix);
            assert_eq!(attribute.prefix(), &prefix);
            assert_eq!(selector.namespace(), &type_constraint);
            assert_eq!(universal.namespace(), &type_constraint);
            assert_eq!(selector.local_name(), Some("Leaf"));
            assert!(!selector.is_universal());
            assert_eq!(universal.local_name(), None);
            assert!(universal.is_universal());
            assert_eq!(attribute.local_name().as_str(), "HREF");
            if prefix == CssQualifiedNamePrefix::Unqualified {
                assert_eq!(attribute.namespace(), &CssNamespaceConstraint::ExplicitNone);
            } else {
                assert_eq!(attribute.namespace(), &type_constraint);
            }
        }
    }
}

#[test]
fn named_construction_rejects_missing_and_case_mismatched_bindings_with_typed_errors() {
    let context = context(None);
    for prefix in ["missing", "SVG"] {
        let prefix = CssNamespacePrefix::try_new(prefix).unwrap();
        let expected = CssQualifiedNameError::UndeclaredNamespacePrefix(prefix.clone());
        let authored = CssQualifiedNamePrefix::Named(prefix);
        assert_eq!(
            CssQualifiedSelectorName::try_new(
                authored.clone(),
                CssIdent::try_new("leaf").unwrap(),
                &context
            ),
            Err(expected.clone())
        );
        assert_eq!(
            CssQualifiedSelectorName::try_universal(authored.clone(), &context),
            Err(expected.clone())
        );
        assert_eq!(
            CssQualifiedAttributeName::try_new(
                authored,
                CssAttributeName::try_new("href").unwrap(),
                &context
            ),
            Err(expected)
        );
    }
    // A declared empty namespace name still establishes a named binding.
    let prefix = CssNamespacePrefix::try_new("empty").unwrap();
    let empty =
        CssNamespaceContext::from_bindings([(Some(prefix.clone()), CssNamespaceName::new(""))]);
    let name = CssQualifiedSelectorName::try_new(
        CssQualifiedNamePrefix::Named(prefix.clone()),
        CssIdent::try_new("leaf").unwrap(),
        &empty,
    )
    .unwrap();
    assert_eq!(name.namespace(), &CssNamespaceConstraint::Named(prefix));
}

#[test]
fn serialized_names_roundtrip_decoded_names_and_all_authored_prefixes() {
    for default in [None, Some(""), Some("urn:default")] {
        let context = context(default);
        for prefix in prefixes() {
            for local_name in ["Leaf", "HREF", "1", "a b", "svg|href", "*", "😀"] {
                let name = CssQualifiedSelectorName::try_new(
                    prefix.clone(),
                    CssIdent::try_new(local_name).unwrap(),
                    &context,
                )
                .unwrap();
                let serialized = name.serialize_with_limit(usize::MAX).unwrap();
                let source = format!("{}.mark", serialized.as_css());
                let report = parse_selector(&source, &context);
                assert!(report.is_clean(), "{source}: {report:?}");
                let Some(CssSelector::Compound(compound)) = report.syntax() else {
                    panic!("typed name plus class: {report:?}")
                };
                assert_eq!(compound.type_selector(), Some(&name));
                let attribute = CssQualifiedAttributeName::try_new(
                    prefix.clone(),
                    CssAttributeName::try_new(local_name).unwrap(),
                    &context,
                )
                .unwrap();
                let serialized = attribute.serialize_with_limit(usize::MAX).unwrap();
                let source = format!("[{}]", serialized.as_css());
                let report = parse_selector(&source, &context);
                assert!(report.is_clean(), "{source}: {report:?}");
                let Some(CssSelector::Compound(compound)) = report.syntax() else {
                    panic!("attribute: {report:?}")
                };
                let [parsed] = compound.attributes() else {
                    panic!("one attribute")
                };
                assert_eq!(parsed.qualified_name(), &attribute);
            }
            let universal = CssQualifiedSelectorName::try_universal(prefix, &context).unwrap();
            let serialized = universal.serialize_with_limit(usize::MAX).unwrap();
            let report = parse_selector(serialized.as_css(), &context);
            assert!(report.is_clean(), "{report:?}");
            let Some(CssSelector::Compound(compound)) = report.syntax() else {
                panic!("universal: {report:?}")
            };
            assert_eq!(compound.type_selector(), Some(&universal));
        }
    }
}

#[test]
fn name_serialization_bounds_complete_escaped_output_without_fabricating_source_origins() {
    let context = context(None);
    for prefix in prefixes() {
        let name = CssQualifiedSelectorName::try_new(
            prefix.clone(),
            CssIdent::try_new("a b|😀").unwrap(),
            &context,
        )
        .unwrap();
        let attribute = CssQualifiedAttributeName::try_new(
            prefix,
            CssAttributeName::try_new("a b|😀").unwrap(),
            &context,
        )
        .unwrap();
        for serialized in [
            name.serialize_with_limit(usize::MAX).unwrap(),
            attribute.serialize_with_limit(usize::MAX).unwrap(),
        ] {
            let bytes = serialized.as_css().len();
            assert_eq!(
                name.serialize_with_limit(bytes).unwrap().as_css(),
                serialized.as_css()
            );
            assert_eq!(
                attribute.serialize_with_limit(bytes).unwrap().as_css(),
                serialized.as_css()
            );
            for limit in [0, bytes - 1] {
                let error = name.serialize_with_limit(limit).unwrap_err();
                assert_eq!(error.kind(), CssComponentValueErrorKind::ByteLimit);
                assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
                assert_eq!(
                    attribute.serialize_with_limit(limit).unwrap_err().kind(),
                    CssComponentValueErrorKind::ByteLimit
                );
            }
            for byte in 0..bytes {
                assert!(matches!(
                    serialized.origin_at(byte),
                    Some(
                        CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
                            | CssSerializedOrigin::Separator {
                                before: CssValueOrigin::Programmatic,
                                after: CssValueOrigin::Programmatic
                            }
                    )
                ));
            }
        }
        assert_eq!(name.local_name(), Some("a b|😀"));
        assert_eq!(attribute.local_name().as_str(), "a b|😀");
    }
}

#[test]
fn literal_identifier_star_and_universal_remain_distinct() {
    let context = context(None);
    let prefix = CssQualifiedNamePrefix::Any;
    let identifier = CssQualifiedSelectorName::try_new(
        prefix.clone(),
        CssIdent::try_new("*").unwrap(),
        &context,
    )
    .unwrap();
    let universal = CssQualifiedSelectorName::try_universal(prefix, &context).unwrap();
    assert_ne!(identifier, universal);
    assert_eq!(
        identifier.serialize_with_limit(10).unwrap().as_css(),
        r"*|\*"
    );
    assert_eq!(universal.serialize_with_limit(10).unwrap().as_css(), "*|*");
    for invalid in ["", "\0", "a\0b"] {
        assert!(CssIdent::try_new(invalid).is_err());
        assert!(CssAttributeName::try_new(invalid).is_none());
    }
}
