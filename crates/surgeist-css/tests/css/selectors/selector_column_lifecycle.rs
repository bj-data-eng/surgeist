#![forbid(unsafe_code)]
//! Literal selected Selectors 4 grammar expectations for the new Column model,
//! its existing authored providers and cumulative specified-output boundary.
use surgeist_css::*;

fn parsed(source: &str, namespaces: &CssNamespaceContext) -> CssSelector {
    let report = parse_selector(source, namespaces);
    assert!(report.is_clean(), "{source:?}: {report:?}");
    report.into_validation_result().unwrap().unwrap()
}

fn complex(selector: &CssSelector) -> &CssComplexSelector {
    let CssSelector::Complex(value) = selector else {
        panic!("complex selector: {selector:?}")
    };
    value
}

fn column_has(selector: CssSelector) -> CssSelector {
    CssSelector::PseudoClass(CssPseudoClass::Has(
        CssRelativeSelectorList::try_new(vec![CssRelativeSelector::new(
            CssSelectorCombinator::Column,
            selector,
        )])
        .unwrap(),
    ))
}

#[test]
fn column_tokens_preserve_relation_and_namespace_boundaries() {
    let namespaces = CssNamespaceContext::from_bindings([
        (None, CssNamespaceName::new("urn:default")),
        (
            Some(CssNamespacePrefix::try_new("svg").unwrap()),
            CssNamespaceName::new("urn:svg"),
        ),
    ]);
    for (source, expected) in [
        ("col||td", "col || td"),
        ("*||td", "* || td"),
        (".Col||.Cell", ".Col || .Cell"),
        ("col |/**/| td", "col || td"),
        ("svg|Col||svg|Cell", "svg|Col || svg|Cell"),
        ("*|Col||*|Cell", "*|Col || *|Cell"),
        ("|Col|||Cell", "|Col || |Cell"),
    ] {
        let selector = parsed(source, &namespaces);
        let before = selector.clone();
        let [part] = complex(&selector).rest() else {
            panic!("one relation")
        };
        assert_eq!(part.combinator(), CssSelectorCombinator::Column);
        assert_eq!(selector.to_specified_css().unwrap(), expected);
        let roundtrip = parsed(expected, &namespaces);
        assert_eq!(roundtrip, selector);
        assert_eq!(selector, before);
    }
    let selector = parsed("svg |Leaf", &namespaces);
    let value = complex(&selector);
    assert_eq!(
        value.first().type_selector().unwrap().prefix(),
        &CssQualifiedNamePrefix::Unqualified
    );
    assert_eq!(
        value.first().type_selector().unwrap().namespace(),
        &CssNamespaceConstraint::Default
    );
    assert_eq!(
        value.rest()[0].combinator(),
        CssSelectorCombinator::Descendant
    );
    assert_eq!(
        value.rest()[0].selector().type_selector().unwrap().prefix(),
        &CssQualifiedNamePrefix::ExplicitNone
    );
    for source in ["col||", "col|||", "||td", "svg| Col", "[svg |Data]"] {
        let report = parse_selector(source, &namespaces);
        assert!(report.syntax().is_none(), "{source:?}: {report:?}");
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn constructed_column_relatives_emit_and_reparse_with_shadow_outer_compounds() {
    let value = column_has(CssSelector::Class("Cell".into()));
    let before = value.clone();
    assert_eq!(value.to_specified_css().unwrap(), ":has(|| .Cell)");
    let reparsed = parsed(":has(|| .Cell)", &CssNamespaceContext::default());
    let CssSelector::PseudoClass(CssPseudoClass::Has(list)) = &reparsed else {
        panic!("has")
    };
    assert_eq!(
        list.selectors()[0].combinator(),
        CssSelectorCombinator::Column
    );
    assert_eq!(
        list.selectors()[0].selector(),
        &CssSelector::Class("Cell".into())
    );
    assert_eq!(value, before);
    let argument = CssCompoundSelectorArgument::try_new(value).unwrap();
    let host = CssSelector::PseudoClass(CssPseudoClass::HostFunction(argument.clone()));
    assert_eq!(host.to_specified_css().unwrap(), ":host(:has(|| .Cell))");
    assert_eq!(
        parsed(":host(:has(|| .Cell))", &CssNamespaceContext::default()),
        host
    );
    let slotted = parsed("::slotted(:has(|| .Cell))", &CssNamespaceContext::default());
    assert_eq!(
        slotted.to_specified_css().unwrap(),
        "::slotted(:has(|| .Cell))"
    );
    let CssSelector::Compound(compound) = slotted else {
        panic!("slotted compound")
    };
    let [CssPseudoElementSegment::PseudoElement(CssPseudoElement::Slotted(actual))] =
        compound.pseudo_elements().unwrap().segments()
    else {
        panic!("slotted argument")
    };
    assert_eq!(actual, &argument);
}

#[test]
fn constructed_column_relatives_share_existing_node_and_utf8_limits() {
    let value = column_has(CssSelector::Class("日本".into()));
    let before = value.clone();
    let expected = ":has(|| .日本)";
    // Has, relative carrier and class: three semantic visits. The combinator
    // contributes punctuation bytes without adding a fourth node.
    assert_eq!(
        value
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                3,
                3,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, before);
    }
    assert_eq!(value.to_specified_css().unwrap(), expected);
}

#[test]
fn column_nested_rules_keep_the_complete_parent_binding_in_normalization() {
    let source = ".Parent, #Alternate { || .Cell { color:red; } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent")
    };
    let [CssRule::Style(child)] = parent.rules() else {
        panic!("child")
    };
    let [CssStyleSelector::Relative(relative)] = child.selectors().selectors() else {
        panic!("relative")
    };
    assert_eq!(relative.combinator(), CssSelectorCombinator::Column);
    assert_eq!(relative.selector(), &CssSelector::Class("Cell".into()));
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    let [declaration] = declarations.as_slice() else {
        panic!("one declaration")
    };
    let context = declaration.selector_context();
    assert_eq!(context.selectors().len(), 1);
    assert_eq!(
        context.selectors()[0].binding(),
        CssSelectorBinding::LeadingCombinator(CssSelectorCombinator::Column)
    );
    assert_eq!(
        context.selectors()[0].selector(),
        &CssSelector::Class("Cell".into())
    );
    let parent = context.parent().unwrap();
    assert_eq!(parent.selectors().len(), 2);
    assert_eq!(
        parent.selectors()[0].selector(),
        &CssSelector::Class("Parent".into())
    );
    assert_eq!(
        parent.selectors()[1].selector(),
        &CssSelector::Key("Alternate".into())
    );
    let emitted = report.syntax().to_specified_css().unwrap();
    assert!(emitted.contains("|| .Cell"), "{emitted}");
    assert!(parse_sheet(&emitted).is_clean());
    assert_eq!(
        child.position().byte_offset().value(),
        source.find("||").unwrap()
    );
}

#[test]
fn column_scope_and_supports_hosts_reuse_the_same_typed_provider() {
    let report = parse_sheet("@scope (col||td) { || .Cell {} } @supports selector(.x || .y) {}");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Scope(scope), CssRule::Supports(supports)] = report.syntax().rules() else {
        panic!("two hosts")
    };
    let root = scope.root().unwrap().selectors()[0].selector();
    assert_eq!(
        complex(root).rest()[0].combinator(),
        CssSelectorCombinator::Column
    );
    let [CssScopedRule::Style(style)] = scope.rules().rules() else {
        panic!("scoped child")
    };
    let [CssScopedStyleSelector::Relative(relative)] = style.selectors().selectors() else {
        panic!("scoped relative")
    };
    assert_eq!(relative.combinator(), CssSelectorCombinator::Column);
    let CssSupportsConditionKind::Selector(selector) = supports.condition().kind() else {
        panic!("typed selector test")
    };
    assert_eq!(
        complex(selector).rest()[0].combinator(),
        CssSelectorCombinator::Column
    );
    assert_eq!(selector.to_specified_css().unwrap(), ".x || .y");
    let emitted = report.syntax().to_specified_css().unwrap();
    let roundtrip = parse_sheet(&emitted);
    assert!(roundtrip.is_clean(), "{emitted}: {roundtrip:?}");
}

#[test]
fn column_relative_graphs_inherit_nested_has_and_pseudo_element_exclusions() {
    let nested = column_has(CssSelector::Class("Nested".into()));
    let logical = CssSelector::PseudoClass(CssPseudoClass::Is(
        CssPseudoSelectorList::try_new(vec![nested.clone()]).unwrap(),
    ));
    let nth = CssSelector::PseudoClass(CssPseudoClass::NthChild(CssNthChildPattern::new(
        CssNthPattern::Odd,
        Some(CssPseudoSelectorList::try_new(vec![nested.clone()]).unwrap()),
    )));
    let shadow = CssSelector::PseudoClass(CssPseudoClass::HostFunction(
        CssCompoundSelectorArgument::try_new(nested).unwrap(),
    ));
    let pseudo_element = parsed(".Cell::before", &CssNamespaceContext::default());
    for child in [logical, nth, shadow, pseudo_element] {
        let value = column_has(child);
        let before = value.clone();
        assert_eq!(
            value.to_specified_css().unwrap_err().kind(),
            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue
        );
        assert_eq!(value, before);
        assert_eq!(
            value.to_specified_css().unwrap_err().kind(),
            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue
        );
    }
}

#[test]
fn class_dot_reports_the_first_whitespace_after_comments_in_original_coordinates() {
    for source in ["./*😀*/ Name", "./*😀\r\né*/ Name"] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{report:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejection")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        let offset = source.find(' ').unwrap();
        let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
        let position = diagnostic.error().position();
        assert_eq!(position.byte_offset().value(), offset);
        assert_eq!(position.line().value(), u32::from(line_start != 0));
        assert_eq!(
            position.column().value(),
            u32::try_from(source[line_start..offset].encode_utf16().count()).unwrap()
        );
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }
}

#[test]
fn class_dot_adjacency_is_shared_by_nested_scope_logical_has_and_supports_hosts() {
    for source in [
        ".Parent { > ./**/Cell {} }",
        "@scope (.Root) { ./**/Cell {} }",
        "@supports selector(./**/Cell) {}",
        ":is(./**/Cell) {}",
        ":has(> ./**/Cell) {}",
    ] {
        assert!(parse_sheet(source).is_clean(), "{source:?}");
    }
    for source in [
        ".Parent { > . Cell {} }",
        "@scope (. Root) {}",
        ":is(. Cell) {}",
        ":has(> . Cell) {}",
    ] {
        let report = parse_sheet(source);
        assert!(!report.is_clean(), "{source:?}");
        assert!(report.into_validation_result().is_err());
    }
    // Balanced unsupported selector syntax is the existing general-enclosed
    // supports alternative; it must not become a typed selector test.
    let report = parse_sheet("@supports selector(. Cell) {}");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Supports(supports)] = report.syntax().rules() else {
        panic!("supports")
    };
    assert!(matches!(
        supports.condition().kind(),
        CssSupportsConditionKind::GeneralEnclosed(_)
    ));
}
