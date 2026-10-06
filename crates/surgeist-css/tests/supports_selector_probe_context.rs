#![forbid(unsafe_code)]
//! Source-owned Supports selector probes preserve the outer lexical contract.
use surgeist_css::*;

fn checked(source: &str, namespaces: &CssNamespaceContext) -> CssSupportsCondition {
    let components = parse_component_values(source).unwrap();
    let original = components.clone();
    let condition = CssSupportsCondition::try_from_components(components, namespaces).unwrap();
    assert_eq!(condition.components(), original.items());
    assert_eq!(condition.origin(), original.items()[0].origin());
    assert_eq!(condition.serialize().unwrap().as_css(), source);
    condition
}

fn selector(condition: &CssSupportsCondition) -> &CssSelector {
    let CssSupportsConditionKind::Selector(selector) = condition.kind() else {
        panic!("clean typed selector")
    };
    selector
}

fn members(selector: &CssSelector) -> &[CssPseudoSelectorListItem] {
    let CssSelector::PseudoClass(CssPseudoClass::Is(list) | CssPseudoClass::Where(list)) = selector
    else {
        panic!("logical selector")
    };
    list.items()
}

#[test]
fn constructed_logical_selectors_have_decoded_typed_members_and_original_spelling() {
    for (source, specified, names) in [
        ("selector(:is(.A,.B))", ":is(.A, .B)", vec!["A", "B"]),
        ("selector(:where(.\\41))", ":where(.A)", vec!["A"]),
    ] {
        let condition = checked(source, &CssNamespaceContext::default());
        let selector = selector(&condition);
        assert_eq!(selector.to_specified_css().unwrap(), specified);
        let expected: Vec<_> = names
            .into_iter()
            .map(|name| CssPseudoSelectorListItem::Selector(CssSelector::Class(name.into())))
            .collect();
        assert_eq!(members(selector), expected);
    }
}

#[test]
fn parsed_unicode_crlf_rule_keeps_original_opener_and_serialized_token_origins() {
    let source = "/*😀*/\r\n@supports selector(:is(.A)) { .Keep {color:red;} }";
    let report = parse_sheet(source);
    assert!(report.is_clean());
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("Supports rule")
    };
    let condition = rule.condition();
    assert_eq!(
        members(selector(condition)),
        [CssPseudoSelectorListItem::Selector(CssSelector::Class(
            "A".into()
        ))]
    );
    let CssValueOrigin::Parsed(origin) = condition.origin() else {
        panic!("original opener")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 20);
    assert_eq!(origin.span().end().byte_offset().value(), 29);
    assert_eq!(origin.span().start().line().value(), 1);
    assert_eq!(origin.span().start().column().value(), 10);
    let serialized = condition.serialize().unwrap();
    assert_eq!(serialized.as_css(), " selector(:is(.A)) ");
    assert_eq!(
        serialized.origin_at(1),
        Some(&CssSerializedOrigin::Token(condition.origin().clone()))
    );
    let CssComponentValueRef::Function(function) = condition.components()[1].view() else {
        panic!("selector function")
    };
    let CssComponentValueRef::Function(is) = function.values().items()[1].view() else {
        panic!("Is function")
    };
    assert_eq!(
        serialized.origin_at(14),
        Some(&CssSerializedOrigin::Token(
            is.values().items()[0].origin().clone()
        ))
    );
    for (index, offset) in [(0, 0), (2, 18)] {
        assert!(matches!(
            condition.components()[index].view(),
            CssComponentValueRef::Token(CssValueTokenRef::Whitespace(" "))
        ));
        assert_eq!(
            serialized.origin_at(offset),
            Some(&CssSerializedOrigin::Token(
                condition.components()[index].origin().clone()
            ))
        );
    }
    assert!(matches!(rule.rules(), [CssRule::Style(_)]));
}

#[test]
fn generated_snapshot_preserves_default_and_case_sensitive_named_namespace_context() {
    let namespaces = CssNamespaceContext::from_bindings([
        (None, CssNamespaceName::new("urn:default")),
        (
            Some(CssNamespacePrefix::try_new("Svg").unwrap()),
            CssNamespaceName::new("urn:svg"),
        ),
    ]);
    for (source, named) in [
        ("selector(:is(Leaf))", false),
        ("selector(:where(Svg|Leaf))", true),
    ] {
        let condition = checked(source, &namespaces);
        let [CssPseudoSelectorListItem::Selector(CssSelector::Compound(compound))] =
            members(selector(&condition))
        else {
            panic!("qualified type")
        };
        let namespace = compound.type_selector().unwrap().namespace();
        if named {
            assert!(
                matches!(namespace, CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "Svg")
            );
        } else {
            assert_eq!(namespace, &CssNamespaceConstraint::Default);
        }
    }
    let condition = checked("selector(:is(svg|Leaf))", &namespaces);
    assert!(matches!(
        condition.kind(),
        CssSupportsConditionKind::GeneralEnclosed(_)
    ));
    let source =
        "@namespace Svg 'urn:svg'; @namespace 'urn:default'; @supports selector(:is(Svg|Leaf)) {}";
    let report = parse_sheet(source);
    assert!(report.is_clean());
    let [
        CssRule::Namespace(_),
        CssRule::Namespace(_),
        CssRule::Supports(rule),
    ] = report.syntax().rules()
    else {
        panic!("active namespace rule")
    };
    let [CssPseudoSelectorListItem::Selector(CssSelector::Compound(compound))] =
        members(selector(rule.condition()))
    else {
        panic!("qualified type")
    };
    assert!(
        matches!(compound.type_selector().unwrap().namespace(), CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "Svg")
    );
}

#[test]
fn diagnosed_selector_probes_remain_original_general_enclosed_without_raw_carrier_leak() {
    for source in [
        "selector(:is(:future(&),.A))",
        "selector(:where(:future(inner(&))))",
        "selector(:has(:is(:has(&))))",
        "selector(:is(:future(x),.A))",
        "selector(:is())",
        "selector(:is(.A),.B)",
    ] {
        let condition = checked(source, &CssNamespaceContext::default());
        let CssSupportsConditionKind::GeneralEnclosed(opaque) = condition.kind() else {
            panic!("diagnosed or incomplete selector remains opaque")
        };
        assert_eq!(opaque.authored(), Some(source));
        assert_eq!(opaque.component(), &condition.components()[0]);
        let report = parse_sheet(&format!(
            "@supports {source} {{ .Keep {{color:red;}} }} .After {{color:blue;}}"
        ));
        assert!(report.is_clean());
        let [CssRule::Supports(rule), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("retained outer and sibling")
        };
        assert!(matches!(
            rule.condition().kind(),
            CssSupportsConditionKind::GeneralEnclosed(_)
        ));
        assert!(matches!(rule.rules(), [CssRule::Style(_)]));
    }
}

#[test]
fn nested_style_context_and_outer_eof_closures_stay_with_the_original_owner() {
    let context =
        CssParserContext::new(CssParserMode::Quirks).with_svg_glyph_orientation_vertical();
    let report = context.parse_sheet(
        ".Parent { @supports selector(:is(.A)) {color:red; & .Child {color:blue;} } }",
    );
    assert!(report.is_clean());
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent")
    };
    let [CssRule::Supports(rule)] = parent.rules() else {
        panic!("nested Supports")
    };
    assert_eq!(
        members(selector(rule.condition())),
        [CssPseudoSelectorListItem::Selector(CssSelector::Class(
            "A".into()
        ))]
    );
    assert!(matches!(
        rule.rules(),
        [CssRule::NestedDeclarations(_), CssRule::Style(_)]
    ));
    let source = "@supports selector(:is(.A)) { .Keep {color:red;";
    let report = parse_sheet(source);
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("retained EOF blocks")
    };
    assert_eq!(
        members(selector(rule.condition())),
        [CssPseudoSelectorListItem::Selector(CssSelector::Class(
            "A".into()
        ))]
    );
    assert_eq!(report.diagnostics().len(), 2);
    for diagnostic in report.diagnostics() {
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
    }
}

#[test]
fn programmatic_components_and_output_retry_do_not_invent_original_coordinates() {
    let class = CssComponentValues::try_new(vec![
        CssComponentValue::try_token(".").unwrap(),
        CssComponentValue::try_ident("A").unwrap(),
    ])
    .unwrap();
    let is = CssComponentValue::try_function("is", class).unwrap();
    let arguments =
        CssComponentValues::try_new(vec![CssComponentValue::try_token(":").unwrap(), is]).unwrap();
    for arguments in [arguments, parse_component_values(":is(.A)").unwrap()] {
        let original = arguments.clone();
        let outer = CssComponentValue::try_function("selector", arguments).unwrap();
        let condition = CssSupportsCondition::try_from_components(
            CssComponentValues::try_new(vec![outer]).unwrap(),
            &CssNamespaceContext::default(),
        )
        .unwrap();
        let CssComponentValueRef::Function(function) = condition.components()[0].view() else {
            panic!("programmatic outer")
        };
        assert_eq!(function.values().items(), original.items());
        assert_eq!(condition.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(condition.position(), None);
        assert_eq!(
            members(selector(&condition)),
            [CssPseudoSelectorListItem::Selector(CssSelector::Class(
                "A".into()
            ))]
        );
        let before = condition.clone();
        assert!(condition.serialize_with_limit(16).is_err());
        assert_eq!(condition, before);
        assert_eq!(
            condition.serialize_with_limit(17).unwrap().as_css(),
            "selector(:is(.A))"
        );
    }
}

#[test]
fn component_backed_dir_lang_and_part_arguments_use_the_generated_owner() {
    let condition = checked("selector(:dir(rtl))", &CssNamespaceContext::default());
    let CssSelector::PseudoClass(CssPseudoClass::Dir(dir)) = selector(&condition) else {
        panic!("Dir")
    };
    assert_eq!(dir.as_str(), "rtl");
    let CssValueOrigin::Parsed(origin) = dir.origin() else {
        panic!("generated component origin")
    };
    assert_eq!(origin.source().as_str(), ":dir(rtl)");
    let condition = checked(
        "selector(:lang(en-US,\"de\"))",
        &CssNamespaceContext::default(),
    );
    let CssSelector::PseudoClass(CssPseudoClass::Lang(list)) = selector(&condition) else {
        panic!("Lang")
    };
    assert_eq!(
        list.ranges()
            .iter()
            .map(CssLanguageRange::as_str)
            .collect::<Vec<_>>(),
        ["en-US", "de"]
    );
    for range in list.ranges() {
        let CssValueOrigin::Parsed(origin) = range.origin() else {
            panic!("generated component origin")
        };
        assert_eq!(origin.source().as_str(), ":lang(en-US,\"de\")");
    }
    let condition = checked("selector(::part(Label))", &CssNamespaceContext::default());
    let CssSelector::Compound(compound) = selector(&condition) else {
        panic!("Part compound")
    };
    let [CssPseudoElementSegment::PseudoElement(CssPseudoElement::Part(list))] =
        compound.pseudo_elements().unwrap().segments()
    else {
        panic!("Part segment")
    };
    assert_eq!(
        list.names()
            .iter()
            .map(CssPartName::as_str)
            .collect::<Vec<_>>(),
        ["Label"]
    );
    let CssValueOrigin::Parsed(origin) = list.names()[0].origin() else {
        panic!("generated component origin")
    };
    assert_eq!(origin.source().as_str(), "::part(Label)");
}

#[test]
fn scoped_and_import_supports_share_the_same_source_owning_operand() {
    let report = parse_sheet("@scope (.Root) { @supports selector(:where(.A)) { .Keep {} } }");
    assert!(report.is_clean());
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("scope")
    };
    let [CssScopedRule::Supports(rule)] = scope.rules().rules() else {
        panic!("scoped Supports")
    };
    assert_eq!(
        members(selector(rule.condition())),
        [CssPseudoSelectorListItem::Selector(CssSelector::Class(
            "A".into()
        ))]
    );
    let report = parse_sheet("@import 'x.css' supports(selector(:is(.A))); .Keep {}");
    assert!(report.is_clean());
    let [CssRule::Import(import), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("import and sibling")
    };
    let condition = import.supports().unwrap().condition();
    assert_eq!(
        members(selector(condition)),
        [CssPseudoSelectorListItem::Selector(CssSelector::Class(
            "A".into()
        ))]
    );
    assert_eq!(condition.serialize().unwrap().as_css(), "selector(:is(.A))");
}
