use surgeist_css::{
    CssCompoundSelector, CssErrorCode, CssLanguageRange, CssNamespaceConstraint, CssNamespaceName,
    CssNamespacePrefix, CssPseudoClass, CssPseudoElement, CssRecoveryAction, CssRule,
    CssScopedRule, CssSelector, CssSelectorCombinator, CssSupportsConditionKind, ErrorKind,
    parse_sheet,
};

fn compound_selector(rule: &CssRule) -> &CssCompoundSelector {
    let CssRule::Style(rule) = rule else {
        panic!("expected style rule")
    };
    let CssSelector::Compound(selector) = rule.selectors().selectors()[0].selector() else {
        panic!("expected namespace-aware compound selector")
    };
    selector
}

#[test]
fn escaped_type_selector_names_preserve_decoded_public_names_without_unwind() {
    let cases = [
        ("selector/TypeSelector.json#/escaping/0", r"x\:y", "x:y"),
        ("selector/TypeSelector.json#/escaping/1", r"\1 ", "\u{1}"),
        ("selector/TypeSelector.json#/escaping/2", r"\a ", "\n"),
        ("selector/TypeSelector.json#/escaping/3", r"\d ", "\r"),
        ("selector/TypeSelector.json#/escaping/4", r"\7f ", "\u{7f}"),
        ("selector/TypeSelector.json#/escaping/5", r"\30 ", "0"),
        ("selector/TypeSelector.json#/escaping/6", r"\35 ", "5"),
        ("selector/TypeSelector.json#/escaping/7", r"\39 ", "9"),
        ("selector/TypeSelector.json#/escaping/8", r"\-", "-"),
        ("selector/TypeSelector.json#/escaping/9", r"\-\30 ", "-0"),
        ("selector/TypeSelector.json#/escaping/10", r"\-\30 -", "-0-"),
        (
            "selector/TypeSelector.json#/escaping/11",
            r"\30 123",
            "0123",
        ),
        (
            "selector/TypeSelector.json#/escaping/12",
            r"\--\+\a \d \30 asd",
            "--+\n\r0asd",
        ),
        (
            "selector/TypeSelector.json#/escaping/14",
            r"asd\30 \a -asd",
            "asd0\n-asd",
        ),
        (
            "selector/TypeSelector.json#/escaping/15",
            r"\20 123",
            " 123",
        ),
    ];

    for (case_id, selector, expected_local_name) in cases {
        let source = format!("@namespace \"\"; {selector} {{ color: red; }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{case_id}: {:?}", report.diagnostics());
        let [CssRule::Namespace(_), rule @ CssRule::Style(_)] = report.syntax().rules() else {
            panic!("{case_id}: expected a namespace and one retained style rule")
        };
        let type_selector = compound_selector(rule)
            .type_selector()
            .unwrap_or_else(|| panic!("{case_id}: expected a retained type selector"));
        assert_eq!(
            type_selector.local_name(),
            Some(expected_local_name),
            "{case_id}"
        );
    }
}

#[test]
fn selectors3_pseudos_legacy_forms_and_repeated_ids_are_typed() {
    let report = parse_sheet(concat!(
        ".target:target { color: red; }",
        ".language:lang(en) { color: red; }",
        ".visited:visited { color: red; }",
        ".line::first-line { color: red; }",
        ".before:before { color: red; }",
        ".after:after { color: red; }",
        ".legacy-line:first-line { color: red; }",
        ".letter:first-letter { color: red; }",
        "article#first#second.card[data-ready] > a:link { color: red; }",
    ));

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 9);

    let target = compound_selector(&report.syntax().rules()[0]);
    assert!(matches!(target.pseudo_classes(), [CssPseudoClass::Target]));
    let language = compound_selector(&report.syntax().rules()[1]);
    assert!(matches!(
        language.pseudo_classes(),
        [CssPseudoClass::Lang(range)] if range.as_str() == "en"
    ));
    let visited = compound_selector(&report.syntax().rules()[2]);
    assert!(matches!(
        visited.pseudo_classes(),
        [CssPseudoClass::Visited]
    ));

    for (index, expected) in [
        (3, CssPseudoElement::FirstLine),
        (4, CssPseudoElement::Before),
        (5, CssPseudoElement::After),
        (6, CssPseudoElement::FirstLine),
        (7, CssPseudoElement::FirstLetter),
    ] {
        let selector = compound_selector(&report.syntax().rules()[index]);
        assert_eq!(
            selector
                .pseudo_elements()
                .expect("typed pseudo-element")
                .pseudo_elements(),
            [expected]
        );
    }

    let CssRule::Style(complex) = &report.syntax().rules()[8] else {
        panic!("expected complex selector style rule")
    };
    let CssSelector::Complex(complex) = complex.selectors().selectors()[0].selector() else {
        panic!("expected preserved child combinator")
    };
    assert_eq!(complex.first().ids(), ["first", "second"]);
    assert_eq!(complex.first().key().map(String::as_str), Some("second"));
    assert_eq!(complex.first().classes(), ["card"]);
    assert_eq!(complex.first().attributes().len(), 1);
    let [subject] = complex.rest() else {
        panic!("expected one complex selector part")
    };
    assert_eq!(subject.combinator(), CssSelectorCombinator::Child);
    assert!(matches!(
        subject.selector().pseudo_classes(),
        [CssPseudoClass::Link]
    ));

    assert_eq!(
        CssLanguageRange::try_new("en-US")
            .expect("decoded identifier")
            .as_str(),
        "en-US"
    );
    assert!(CssLanguageRange::try_new("en US").is_none());
}

#[test]
fn selectors3_official_pseudo_matrix_and_escaped_language_identifier_are_clean() {
    let report = parse_sheet(concat!(
        ":root,:link,:visited,:target,:lang(e\\6e),:hover,:active,:focus,",
        ":enabled,:disabled,:checked,:indeterminate,:first-child,:last-child,",
        ":only-child,:empty,:nth-child(2n+1),:nth-last-child(2),",
        ":first-of-type,:last-of-type,:only-of-type,:nth-of-type(odd),",
        ":nth-last-of-type(even),:not(.excluded) { color: red; }",
        ".line::first-line { color: red; }",
        ".letter::first-letter { color: red; }",
        ".before::before { color: red; }",
        ".after::after { color: red; }",
    ));

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 5);
    let CssRule::Style(pseudos) = &report.syntax().rules()[0] else {
        panic!("expected the authored pseudo-selector list")
    };
    assert_eq!(pseudos.selectors().selectors().len(), 24);
    assert!(matches!(
        pseudos.selectors().selectors()[4].selector(),
        CssSelector::PseudoClass(CssPseudoClass::Lang(range)) if range.as_str() == "en"
    ));
}

#[test]
fn namespace_rules_obey_namespaces3_prelude_ordering() {
    let source = concat!(
        "@namespace \"not a URI\";",
        "@namespace svg url(https://example.test/one);",
        "@namespace SVG \"\";",
        "@namespace s\\76 g \"replacement\";",
        "@namespace \"\";",
    );
    let report = parse_sheet(source);

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [
        CssRule::Namespace(first_default),
        CssRule::Namespace(first_named),
        CssRule::Namespace(case_distinct),
        CssRule::Namespace(replacement),
        CssRule::Namespace(last_default),
    ] = report.syntax().rules()
    else {
        panic!("expected five retained namespace rules in authored order")
    };

    assert!(first_default.prefix().is_none());
    assert_eq!(first_default.name().as_str(), "not a URI");
    assert_eq!(first_named.prefix().expect("named prefix").as_str(), "svg");
    assert_eq!(first_named.name().as_str(), "https://example.test/one");
    assert_eq!(
        case_distinct
            .prefix()
            .expect("case-distinct prefix")
            .as_str(),
        "SVG"
    );
    assert_eq!(case_distinct.name().as_str(), "");
    assert_eq!(
        replacement.prefix().expect("escaped prefix").as_str(),
        "svg"
    );
    assert_eq!(replacement.name().as_str(), "replacement");
    assert!(last_default.prefix().is_none());
    assert_eq!(last_default.name().as_str(), "");
    assert_eq!(first_default.position().byte_offset().value(), 0);
}

#[test]
fn late_namespace_is_a_placement_error_not_an_unsupported_rule() {
    let source = concat!(
        ".before { color: red; } ",
        "@namespace svg url(http://example.test/a;b); ",
        ".after { color: blue; }",
    );
    let report = parse_sheet(source);

    assert!(matches!(
        report.syntax().rules(),
        [CssRule::Style(_), CssRule::Style(_)]
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one late-namespace recovery diagnostic")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePlacement
    );
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnsupportedAtRule);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    let ErrorKind::InvalidAtRulePlacement(detail) = diagnostic.error().kind() else {
        panic!("expected namespace placement payload")
    };
    assert_eq!(detail.name().as_str(), "namespace");
    assert_eq!(
        detail.expected_context().as_str(),
        "after initial layer statements and imports, before later layers or body rules"
    );
}

#[test]
fn namespace_public_values_check_prefixes_and_preserve_literal_names() {
    assert_eq!(
        CssNamespacePrefix::try_new("svg")
            .expect("decoded identifier")
            .as_str(),
        "svg"
    );
    assert_eq!(
        CssNamespacePrefix::try_new("SVG")
            .expect("case-sensitive identifier")
            .as_str(),
        "SVG"
    );
    assert!(CssNamespacePrefix::try_new("").is_none());
    assert!(CssNamespacePrefix::try_new("two names").is_none());
    assert!(CssNamespacePrefix::try_new("s\\76 g").is_none());

    assert_eq!(CssNamespaceName::new("").as_str(), "");
    assert_eq!(CssNamespaceName::new("not a URI").as_str(), "not a URI");
}

#[test]
fn malformed_late_and_nested_namespace_rules_drop_one_at_rule_and_keep_siblings() {
    for invalid in [
        "@namespace;",
        "@namespace svg;",
        "@namespace svg ident;",
        "@namespace 123 \"urn:test\";",
        "@namespace \"urn:test\" extra;",
        "@namespace svg url(\"urn:test\") {}",
    ] {
        let source = format!("{invalid} .kept {{ color: red; }}");
        let report = parse_sheet(&source);
        assert!(
            matches!(report.syntax().rules(), [CssRule::Style(_)]),
            "{source}: {:?}",
            report.syntax().rules()
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected one namespace diagnostic")
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropAtRule,
            "{source}"
        );
    }

    let missing_semicolon = parse_sheet("@namespace svg \"urn:test\"");
    assert!(missing_semicolon.syntax().rules().is_empty());
    assert_eq!(missing_semicolon.diagnostics().len(), 1);
    assert_eq!(
        missing_semicolon.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );

    for source in [
        ".body {} @namespace \"urn:late\"; .kept {}",
        "@media screen { @namespace \"urn:nested\"; .kept {} }",
        ".parent { @namespace \"urn:nested\"; color: red; }",
        "@scope { @namespace \"urn:nested\"; .kept {} }",
    ] {
        let report = parse_sheet(source);
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidAtRulePlacement)
            .unwrap_or_else(|| panic!("{source}: expected namespace placement diagnostic"));
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropAtRule,
            "{source}"
        );
        let ErrorKind::InvalidAtRulePlacement(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected placement payload")
        };
        assert_eq!(detail.name().as_str(), "namespace", "{source}");
        if source.starts_with(".body") {
            assert_eq!(
                detail.expected_context().as_str(),
                "after initial layer statements and imports, before later layers or body rules"
            );
        } else {
            assert_eq!(
                detail.expected_context().as_str(),
                "the stylesheet top level"
            );
        }
    }
}

#[test]
fn namespace_qualified_type_universal_and_attribute_selectors_use_active_bindings() {
    let report = parse_sheet(concat!(
        "@namespace \"urn:default\";",
        "@namespace svg \"urn:svg\";",
        "svg|a { color: red; }",
        "svg|* { color: red; }",
        "*|a { color: red; }",
        "|a { color: red; }",
        "a { color: red; }",
        ".attributes[svg|href][*|title][|lang][plain] { color: red; }",
    ));

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 8);
    assert!(matches!(
        report.syntax().rules(),
        [
            CssRule::Namespace(_),
            CssRule::Namespace(_),
            CssRule::Style(_),
            CssRule::Style(_),
            CssRule::Style(_),
            CssRule::Style(_),
            CssRule::Style(_),
            CssRule::Style(_),
        ]
    ));

    let named_type = compound_selector(&report.syntax().rules()[2]);
    let named_type_name = named_type.type_selector().expect("named type selector");
    assert!(matches!(
        named_type_name.namespace(),
        CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg"
    ));
    assert_eq!(named_type_name.local_name(), Some("a"));
    assert!(!named_type_name.is_universal());
    assert_eq!(named_type.tag().map(String::as_str), Some("a"));

    let named_universal = compound_selector(&report.syntax().rules()[3]);
    let named_universal = named_universal
        .type_selector()
        .expect("named universal selector");
    assert!(matches!(
        named_universal.namespace(),
        CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg"
    ));
    assert_eq!(named_universal.local_name(), None);
    assert!(named_universal.is_universal());

    for (index, expected_namespace) in [
        (4, CssNamespaceConstraint::Any),
        (5, CssNamespaceConstraint::ExplicitNone),
        (6, CssNamespaceConstraint::Default),
    ] {
        let type_selector = compound_selector(&report.syntax().rules()[index])
            .type_selector()
            .expect("qualified type selector");
        assert_eq!(type_selector.namespace(), &expected_namespace);
        assert_eq!(type_selector.local_name(), Some("a"));
    }

    let attributes = compound_selector(&report.syntax().rules()[7]);
    let [named, any, explicit_none, unqualified] = attributes.attributes() else {
        panic!("expected four qualified attribute selectors")
    };
    assert!(matches!(
        named.namespace(),
        CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg"
    ));
    assert_eq!(named.name().as_str(), "href");
    assert_eq!(any.namespace(), &CssNamespaceConstraint::Any);
    assert_eq!(any.name().as_str(), "title");
    assert_eq!(
        explicit_none.namespace(),
        &CssNamespaceConstraint::ExplicitNone
    );
    assert_eq!(explicit_none.name().as_str(), "lang");
    assert_eq!(
        unqualified.namespace(),
        &CssNamespaceConstraint::ExplicitNone
    );
    assert_eq!(unqualified.name().as_str(), "plain");
}

#[test]
fn namespace_prefix_escapes_redeclarations_and_case_remain_exact() {
    let report = parse_sheet(concat!(
        "@namespace svg \"urn:first\";",
        "@namespace SVG \"urn:upper\";",
        "@namespace s\\76 g \"urn:replacement\";",
        "s\\76 g|\\61 ,SVG|a,*|*,|* { color: red; }",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 4);
    let CssRule::Style(style) = &report.syntax().rules()[3] else {
        panic!("expected one authored qualified selector list")
    };
    assert_eq!(style.selectors().selectors().len(), 4);
    for (index, expected) in [(0, Some("svg")), (1, Some("SVG")), (2, None), (3, None)] {
        let CssSelector::Compound(compound) = style.selectors().selectors()[index].selector()
        else {
            panic!("expected qualified selector compound")
        };
        let type_selector = compound.type_selector().expect("qualified selector");
        match expected {
            Some(prefix) => assert!(matches!(
                type_selector.namespace(),
                CssNamespaceConstraint::Named(active) if active.as_str() == prefix
            )),
            None if index == 2 => {
                assert_eq!(type_selector.namespace(), &CssNamespaceConstraint::Any)
            }
            None => assert_eq!(
                type_selector.namespace(),
                &CssNamespaceConstraint::ExplicitNone
            ),
        }
        assert!(type_selector.is_universal() || type_selector.local_name() == Some("a"));
    }
}

#[test]
fn default_namespace_applies_to_universal_types_but_not_omitted_types_or_attributes() {
    let report = parse_sheet(concat!(
        "@namespace \"urn:default\";",
        "* { color: red; }",
        ".class { color: red; }",
        "[plain] { color: red; }",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [
        CssRule::Namespace(_),
        CssRule::Style(universal),
        CssRule::Style(class),
        CssRule::Style(attribute),
    ] = report.syntax().rules()
    else {
        panic!("expected namespace and three style rules")
    };

    let CssSelector::Compound(universal) = universal.selectors().selectors()[0].selector() else {
        panic!("expected universal compound selector")
    };
    let universal = universal.type_selector().expect("universal type selector");
    assert_eq!(universal.namespace(), &CssNamespaceConstraint::Default);
    assert!(universal.is_universal());

    assert!(
        matches!(class.selectors().selectors()[0].selector(), CssSelector::Class(name) if name == "class")
    );
    let CssSelector::Compound(attribute) = attribute.selectors().selectors()[0].selector() else {
        panic!("expected attribute compound selector")
    };
    assert!(attribute.type_selector().is_none());
    let [plain] = attribute.attributes() else {
        panic!("expected one unqualified attribute")
    };
    assert_eq!(plain.namespace(), &CssNamespaceConstraint::ExplicitNone);
}

#[test]
fn malformed_namespace_qualified_names_drop_one_rule_and_keep_siblings() {
    for failed in [
        "svg| { color: red; }",
        "svg|.class { color: red; }",
        "*| { color: red; }",
        ".x[svg|*] { color: red; }",
        ".x[svg| href] { color: red; }",
    ] {
        let source = format!("@namespace svg \"urn:svg\"; .before {{}} {failed} .after {{}}");
        let report = parse_sheet(&source);
        assert!(
            matches!(
                report.syntax().rules(),
                [CssRule::Namespace(_), CssRule::Style(_), CssRule::Style(_)]
            ),
            "{failed}: {:?}",
            report.syntax().rules()
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("{failed}: expected one selector diagnostic")
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
        let failed_start = source.find(failed).expect("failed rule start");
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            failed_start,
            "{failed}"
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            failed_start + failed.len(),
            "{failed}"
        );
    }
}

#[test]
fn namespace_bindings_reach_every_selector_consumer_without_changing_recovery() {
    let report = parse_sheet(concat!(
        "@namespace svg \"urn:svg\";",
        "svg|top { color: red; }",
        "@media screen { svg|media { color: red; } }",
        "@supports selector(svg|supported) { svg|supports { color: red; } }",
        "@container (width > 1px) { svg|container { color: red; } }",
        "@layer theme { svg|layer { color: red; } }",
        ".host { & > svg|nested { color: red; } }",
        "@scope (svg|root) to (svg|limit) { svg|scoped { color: red; } }",
    ));

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [
        CssRule::Namespace(_),
        CssRule::Style(_),
        CssRule::Media(media),
        CssRule::Supports(supports),
        CssRule::Container(container),
        CssRule::LayerBlock(layer),
        CssRule::Style(_),
        CssRule::Scope(scope),
    ] = report.syntax().rules()
    else {
        panic!(
            "expected every namespace-aware selector consumer to be retained: {:?}",
            report.syntax().rules()
        )
    };

    assert!(matches!(media.rules(), [CssRule::Style(_)]));
    assert!(matches!(
        supports.condition().kind(),
        CssSupportsConditionKind::Selector(_)
    ));
    assert!(matches!(supports.rules(), [CssRule::Style(_)]));
    assert!(matches!(container.rules(), [CssRule::Style(_)]));
    assert!(matches!(layer.rules(), [CssRule::Style(_)]));
    assert!(scope.root().is_some());
    assert!(scope.limit().is_some());
    assert!(matches!(scope.rules().rules(), [CssScopedRule::Style(_)]));
}

#[test]
fn namespace_context_keeps_forgiving_members_and_unforgiving_rules_local() {
    let source = concat!(
        "@namespace svg \"urn:svg\";/*🦊*/",
        "@media screen {",
        ".kept:is(svg|a, missing|b) { color: red; }",
        ".failed, missing|b { color: blue; }",
        "svg|after { color: green; }",
        "}",
    );
    let report = parse_sheet(source);

    let [CssRule::Namespace(_), CssRule::Media(media)] = report.syntax().rules() else {
        panic!("expected namespace and retained media group")
    };
    assert!(matches!(
        media.rules(),
        [CssRule::Style(_), CssRule::Style(_)]
    ));
    let [forgiving, unforgiving] = report.diagnostics() else {
        panic!("expected one member-local and one whole-rule recovery")
    };
    assert_eq!(forgiving.action(), CssRecoveryAction::DropSelectorListItem);
    assert_eq!(unforgiving.action(), CssRecoveryAction::DropQualifiedRule);
    assert!(
        forgiving.span().start().byte_offset().value()
            < unforgiving.span().start().byte_offset().value()
    );
    assert_eq!(
        source
            .get(
                forgiving.span().start().byte_offset().value()
                    ..forgiving.span().end().byte_offset().value()
            )
            .expect("forgiving recovery span"),
        " missing|b"
    );
    assert_eq!(
        source
            .get(
                unforgiving.span().start().byte_offset().value()
                    ..unforgiving.span().end().byte_offset().value()
            )
            .expect("unforgiving recovery span"),
        ".failed, missing|b { color: blue; }"
    );
}

#[test]
fn namespace_context_survives_implicit_eof_closure_in_nested_groups() {
    let source = "@namespace svg \"urn:svg\";@media screen { svg|kept { color: red; }";
    let report = parse_sheet(source);

    let [CssRule::Namespace(_), CssRule::Media(media)] = report.syntax().rules() else {
        panic!("expected namespace and implicitly closed media group")
    };
    assert!(matches!(media.rules(), [CssRule::Style(_)]));
    assert!(
        report.diagnostics().iter().all(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        })
    );
}
