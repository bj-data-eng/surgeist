#![forbid(unsafe_code)]
//! Functional construction contracts for the previously absent attribute, simple,
//! compound and part front doors, plus complete complex admission. Expectations
//! use the selected authored grammar and independently computed output tariffs.

use surgeist_css::{
    CssAttributeCaseSensitivity as Case, CssAttributeMatcher as Matcher, CssAttributeName,
    CssAttributeSelector, CssComplexSelector, CssComplexSelectorPart, CssComponentValueError,
    CssComponentValueErrorKind, CssCompoundSelector, CssCompoundSelectorArgument, CssIdent,
    CssNamespaceConstraint, CssNamespaceContext, CssNamespaceName, CssNamespacePrefix,
    CssNthChildPattern, CssNthPattern, CssPseudoClass, CssPseudoSelectorList,
    CssQualifiedAttributeName, CssQualifiedNameError, CssQualifiedNamePrefix as Prefix,
    CssQualifiedSelectorName, CssRelativeSelector, CssRelativeSelectorList, CssSelector,
    CssSelectorCombinator as Combinator, CssSelectorConstructionError,
    CssSelectorConstructionErrorKind as ConstructionKind, CssSimpleSelector as Simple,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind as Kind,
    CssSpecifiedValueSerializationLimits as Limits, CssValueOrigin, parse_selector,
};

fn ident(value: &str) -> CssIdent {
    CssIdent::try_new(value).unwrap()
}

fn class(value: &str) -> Simple {
    Simple::Class(ident(value))
}

fn compound(value: &str) -> CssCompoundSelector {
    CssCompoundSelector::try_new(vec![class(value)]).unwrap()
}

fn name() -> CssQualifiedAttributeName {
    CssQualifiedAttributeName::try_new(
        Prefix::Unqualified,
        CssAttributeName::try_new("A").unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap()
}

fn parsed(source: &str, context: &CssNamespaceContext) -> CssSelector {
    let report = parse_selector(source, context);
    assert!(report.is_clean(), "{source:?}: {report:?}");
    report.into_validation_result().unwrap().unwrap()
}

fn specified_cause(error: &CssSelectorConstructionError, expected: Kind) {
    let ConstructionKind::Specified(cause) = error.kind() else {
        panic!("specified construction cause: {error:?}");
    };
    assert_eq!(cause.kind(), expected);
    assert_eq!(
        std::error::Error::source(error)
            .unwrap()
            .downcast_ref::<CssSpecifiedValueSerializationError>(),
        Some(cause)
    );
}

#[test]
fn attributes_preserve_each_matcher_operand_and_explicit_modifier() {
    for (matcher, operator) in [
        (Matcher::Equals("MiX".into()), "="),
        (Matcher::Includes("MiX".into()), "~="),
        (Matcher::DashMatch("MiX".into()), "|="),
        (Matcher::Prefix("MiX".into()), "^="),
        (Matcher::Suffix("MiX".into()), "$="),
        (Matcher::Substring("MiX".into()), "*="),
    ] {
        for (case, suffix) in [
            (Case::DocumentDefault, ""),
            (Case::AsciiCaseInsensitive, " i"),
            (Case::ExplicitSensitive, " s"),
        ] {
            let attribute = CssAttributeSelector::try_new(name(), matcher.clone(), case).unwrap();
            assert_eq!(attribute.name().as_str(), "A");
            assert_eq!(attribute.matcher(), &matcher);
            assert_eq!(attribute.case_sensitivity(), case);
            let value = CssCompoundSelector::try_new(vec![Simple::Attribute(attribute)]).unwrap();
            let selector = CssSelector::Compound(value);
            let literal = format!("[A{operator}\"MiX\"{suffix}]");
            assert_eq!(selector.to_specified_css().unwrap(), literal);
            assert_eq!(parsed(&literal, &CssNamespaceContext::default()), selector);
        }
    }
    let attribute =
        CssAttributeSelector::try_new(name(), Matcher::Exists, Case::DocumentDefault).unwrap();
    let value = CssCompoundSelector::try_new(vec![Simple::Attribute(attribute)]).unwrap();
    assert_eq!(
        CssSelector::Compound(value).to_specified_css().unwrap(),
        "[A]"
    );
}

#[test]
fn decoded_attribute_values_keep_empty_whitespace_and_escaped_identity() {
    for (operand, literal) in [
        ("", "[A~=\"\"]"),
        ("a b", "[A~=\"a b\"]"),
        ("\"\\", "[A~=\"\\\"\\\\\"]"),
    ] {
        let attribute = CssAttributeSelector::try_new(
            name(),
            Matcher::Includes(operand.into()),
            Case::DocumentDefault,
        )
        .unwrap();
        let selector = CssSelector::Compound(
            CssCompoundSelector::try_new(vec![Simple::Attribute(attribute)]).unwrap(),
        );
        assert_eq!(selector.to_specified_css().unwrap(), literal);
        assert_eq!(parsed(literal, &CssNamespaceContext::default()), selector);
    }
    let from_ident = parsed("[A=MiX I]", &CssNamespaceContext::default());
    let from_string = parsed("[A=\"MiX\" i]", &CssNamespaceContext::default());
    let attribute = CssAttributeSelector::try_new(
        name(),
        Matcher::Equals("MiX".into()),
        Case::AsciiCaseInsensitive,
    )
    .unwrap();
    let selector = CssSelector::Compound(
        CssCompoundSelector::try_new(vec![Simple::Attribute(attribute)]).unwrap(),
    );
    assert_eq!(from_ident, selector);
    assert_eq!(from_string, selector);
    assert_eq!(selector.to_specified_css().unwrap(), "[A=\"MiX\" i]");
}

#[test]
fn qualified_composition_preserves_namespaces_case_and_universal_identity() {
    let named = CssNamespacePrefix::try_new("Svg").unwrap();
    let context = CssNamespaceContext::from_bindings([
        (None, CssNamespaceName::new("urn:default")),
        (Some(named.clone()), CssNamespaceName::new("urn:svg")),
    ]);
    for (prefix, prefix_css, namespace) in [
        (Prefix::Unqualified, "", CssNamespaceConstraint::Default),
        (
            Prefix::ExplicitNone,
            "|",
            CssNamespaceConstraint::ExplicitNone,
        ),
        (Prefix::Any, "*|", CssNamespaceConstraint::Any),
        (
            Prefix::Named(named.clone()),
            "Svg|",
            CssNamespaceConstraint::Named(named.clone()),
        ),
    ] {
        let qualified =
            CssQualifiedSelectorName::try_new(prefix.clone(), ident("Leaf"), &context).unwrap();
        assert_eq!(qualified.namespace(), &namespace);
        let attribute_name = CssQualifiedAttributeName::try_new(
            prefix.clone(),
            CssAttributeName::try_new("DATA").unwrap(),
            &context,
        )
        .unwrap();
        let attribute_namespace = if prefix == Prefix::Unqualified {
            CssNamespaceConstraint::ExplicitNone
        } else {
            namespace
        };
        assert_eq!(attribute_name.namespace(), &attribute_namespace);
        let attribute = CssAttributeSelector::try_new(
            attribute_name.clone(),
            Matcher::Equals("VaL".into()),
            Case::ExplicitSensitive,
        )
        .unwrap();
        assert_eq!(attribute.qualified_name(), &attribute_name);
        let value = CssCompoundSelector::try_new(vec![
            Simple::Type(qualified.clone()),
            Simple::Attribute(attribute),
        ])
        .unwrap();
        assert_eq!(value.type_selector(), Some(&qualified));
        let selector = CssSelector::Compound(value);
        let literal = format!("{prefix_css}Leaf[{prefix_css}DATA=\"VaL\" s]");
        assert_eq!(selector.to_specified_css().unwrap(), literal);
        assert_eq!(parsed(&literal, &context), selector);

        let universal = CssQualifiedSelectorName::try_universal(prefix, &context).unwrap();
        let value =
            CssCompoundSelector::try_new(vec![Simple::Type(universal.clone()), class("MiX")])
                .unwrap();
        assert!(value.type_selector().unwrap().is_universal());
        assert_eq!(value.type_selector(), Some(&universal));
        let selector = CssSelector::Compound(value);
        let literal = format!("{prefix_css}*.MiX");
        assert_eq!(selector.to_specified_css().unwrap(), literal);
        assert_eq!(parsed(&literal, &context), selector);
    }
    let missing = CssNamespacePrefix::try_new("svg").unwrap();
    assert_eq!(
        CssQualifiedAttributeName::try_new(
            Prefix::Named(missing.clone()),
            CssAttributeName::try_new("A").unwrap(),
            &context
        )
        .unwrap_err(),
        CssQualifiedNameError::UndeclaredNamespacePrefix(missing)
    );
}

#[test]
fn attribute_modifier_and_nul_failures_keep_typed_programmatic_causes() {
    for case in [Case::AsciiCaseInsensitive, Case::ExplicitSensitive] {
        let error = CssAttributeSelector::try_new_with_limits(
            name(),
            Matcher::Exists,
            case,
            Limits::new(0, 0, 0),
        )
        .unwrap_err();
        assert_eq!(error.kind(), &ConstructionKind::ModifierWithoutValue);
        assert!(std::error::Error::source(&error).is_none());
    }
    for matcher in [
        Matcher::Equals("a\0b".into()),
        Matcher::Includes("a\0b".into()),
        Matcher::DashMatch("a\0b".into()),
        Matcher::Prefix("a\0b".into()),
        Matcher::Suffix("a\0b".into()),
        Matcher::Substring("a\0b".into()),
    ] {
        let error =
            CssAttributeSelector::try_new(name(), matcher, Case::DocumentDefault).unwrap_err();
        let ConstructionKind::InvalidAttributeValue(cause) = error.kind() else {
            panic!("decoded attribute string cause: {error:?}");
        };
        assert_eq!(cause.kind(), CssComponentValueErrorKind::InvalidString);
        assert_eq!(cause.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(
            std::error::Error::source(&error)
                .unwrap()
                .downcast_ref::<CssComponentValueError>(),
            Some(cause)
        );
    }
    let retry =
        CssAttributeSelector::try_new(name(), Matcher::Equals("ab".into()), Case::DocumentDefault)
            .unwrap();
    assert_eq!(retry.matcher(), &Matcher::Equals("ab".into()));
}

#[test]
fn attribute_limits_charge_one_node_and_exact_utf8_output_atomically() {
    let literal = "[A=\"é\" i]";
    let matcher = Matcher::Equals("é".into());
    let attribute = CssAttributeSelector::try_new_with_limits(
        name(),
        matcher.clone(),
        Case::AsciiCaseInsensitive,
        Limits::new(1, 1, literal.len()),
    )
    .unwrap();
    let selector = CssSelector::Compound(
        CssCompoundSelector::try_new(vec![Simple::Attribute(attribute.clone())]).unwrap(),
    );
    assert_eq!(selector.to_specified_css().unwrap(), literal);
    for (limits, cause) in [
        (Limits::new(0, 1, literal.len()), Kind::InputNodeLimit),
        (Limits::new(1, 0, literal.len()), Kind::ProjectionNodeLimit),
        (Limits::new(1, 1, literal.len() - 1), Kind::ByteLimit),
    ] {
        specified_cause(
            &CssAttributeSelector::try_new_with_limits(
                name(),
                matcher.clone(),
                Case::AsciiCaseInsensitive,
                limits,
            )
            .unwrap_err(),
            cause,
        );
        assert_eq!(attribute.matcher(), &matcher);
        assert_eq!(
            CssAttributeSelector::try_new(name(), matcher.clone(), Case::AsciiCaseInsensitive)
                .unwrap(),
            attribute
        );
    }
}

#[test]
fn compound_rejects_empty_duplicate_and_misplaced_type_members() {
    let error =
        CssCompoundSelector::try_new_with_limits(Vec::new(), Limits::new(0, 0, 0)).unwrap_err();
    assert_eq!(error.kind(), &ConstructionKind::EmptyCompound);
    let name = CssQualifiedSelectorName::try_new(
        Prefix::Unqualified,
        ident("Leaf"),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    let universal =
        CssQualifiedSelectorName::try_universal(Prefix::Any, &CssNamespaceContext::default())
            .unwrap();
    for (members, member_index) in [
        (vec![class("One"), Simple::Type(name.clone())], 1),
        (
            vec![Simple::Type(name.clone()), Simple::Type(universal.clone())],
            1,
        ),
        (
            vec![
                Simple::Type(universal.clone()),
                class("One"),
                Simple::Type(name),
            ],
            2,
        ),
    ] {
        let error =
            CssCompoundSelector::try_new_with_limits(members, Limits::new(0, 0, 0)).unwrap_err();
        assert_eq!(
            error.kind(),
            &ConstructionKind::InvalidTypePosition { member_index }
        );
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn compound_keeps_repeated_ids_classes_and_decoded_whitespace_inside_identifiers() {
    let members = vec![
        class("One"),
        Simple::Id(ident("X")),
        class("One"),
        Simple::Id(ident("X")),
        class("a b"),
        Simple::PseudoClass(CssPseudoClass::Hover),
    ];
    let value = CssCompoundSelector::try_new(members).unwrap();
    assert_eq!(value.ids(), ["X", "X"]);
    assert_eq!(value.classes(), ["One", "One", "a b"]);
    assert_eq!(value.pseudo_classes(), [CssPseudoClass::Hover]);
    assert_eq!(value.scope_anchors(), 0);
    assert_eq!(value.nesting_selectors(), 0);
    let selector = CssSelector::Compound(value);
    let literal = "#X#X.One.One.a\\ b:hover";
    assert_eq!(selector.to_specified_css().unwrap(), literal);
    assert_eq!(parsed(literal, &CssNamespaceContext::default()), selector);
    assert_ne!(
        parsed(".a .b", &CssNamespaceContext::default()),
        parsed(".a\\ b", &CssNamespaceContext::default())
    );
}

#[test]
fn compound_limits_aggregate_independently_admitted_children_and_allow_retry() {
    let attribute =
        CssAttributeSelector::try_new(name(), Matcher::Exists, Case::DocumentDefault).unwrap();
    let members = vec![
        Simple::Id(ident("X")),
        class("é"),
        Simple::Attribute(attribute.clone()),
    ];
    let literal = "#X.é[A]";
    // Compound aggregate + ID + class + attribute = four nodes.
    let value =
        CssCompoundSelector::try_new_with_limits(members.clone(), Limits::new(4, 4, literal.len()))
            .unwrap();
    assert_eq!(value.attributes(), [attribute]);
    assert_eq!(
        CssSelector::Compound(value.clone())
            .to_specified_css()
            .unwrap(),
        literal
    );
    for (limits, cause) in [
        (Limits::new(3, 4, literal.len()), Kind::InputNodeLimit),
        (Limits::new(4, 3, literal.len()), Kind::ProjectionNodeLimit),
        (Limits::new(4, 4, literal.len() - 1), Kind::ByteLimit),
    ] {
        specified_cause(
            &CssCompoundSelector::try_new_with_limits(members.clone(), limits).unwrap_err(),
            cause,
        );
        assert_eq!(
            CssCompoundSelector::try_new(members.clone()).unwrap(),
            value
        );
    }
}

#[test]
fn compound_rechecks_raw_strict_and_nested_has_graphs_in_their_containing_context() {
    let empty = CssPseudoSelectorList::try_new_forgiving(Vec::new()).unwrap();
    let inner_has = CssSelector::PseudoClass(CssPseudoClass::Has(
        CssRelativeSelectorList::try_new(vec![CssRelativeSelector::new(
            Combinator::Child,
            CssSelector::Class("One".into()),
        )])
        .unwrap(),
    ));
    let outer_has = CssPseudoClass::Has(
        CssRelativeSelectorList::try_new(vec![CssRelativeSelector::new(
            Combinator::Child,
            inner_has,
        )])
        .unwrap(),
    );
    for pseudo in [
        CssPseudoClass::Not(empty.clone()),
        CssPseudoClass::NthChild(CssNthChildPattern::new(
            CssNthPattern::Odd,
            Some(empty.clone()),
        )),
        CssPseudoClass::NthLastChild(CssNthChildPattern::new(
            CssNthPattern::Even,
            Some(empty.clone()),
        )),
        outer_has,
    ] {
        specified_cause(
            &CssCompoundSelector::try_new(vec![class("One"), Simple::PseudoClass(pseudo)])
                .unwrap_err(),
            Kind::UnrepresentableValue,
        );
    }
    for (pseudo, literal) in [
        (CssPseudoClass::Is(empty.clone()), ".One:is()"),
        (CssPseudoClass::Where(empty), ".One:where()"),
    ] {
        let value =
            CssCompoundSelector::try_new(vec![class("One"), Simple::PseudoClass(pseudo)]).unwrap();
        assert_eq!(
            CssSelector::Compound(value).to_specified_css().unwrap(),
            literal
        );
    }
}

#[test]
fn ordinary_compound_allows_complex_logical_arguments_but_shadow_attachment_rechecks() {
    let list = CssPseudoSelectorList::try_new(vec![parsed(
        ".One > .Two",
        &CssNamespaceContext::default(),
    )])
    .unwrap();
    let value =
        CssCompoundSelector::try_new(vec![Simple::PseudoClass(CssPseudoClass::Not(list))]).unwrap();
    assert_eq!(
        CssSelector::Compound(value.clone())
            .to_specified_css()
            .unwrap(),
        ":not(.One > .Two)"
    );
    assert!(CssCompoundSelectorArgument::try_new(CssSelector::Compound(value)).is_none());

    let has = CssPseudoClass::Has(
        CssRelativeSelectorList::try_new(vec![CssRelativeSelector::new(
            Combinator::Child,
            CssSelector::Class("One".into()),
        )])
        .unwrap(),
    );
    let inner = CssCompoundSelector::try_new(vec![Simple::PseudoClass(has)]).unwrap();
    let argument = CssCompoundSelectorArgument::try_new(CssSelector::Compound(inner)).unwrap();
    let value = CssCompoundSelector::try_new(vec![Simple::PseudoClass(
        CssPseudoClass::HostFunction(argument),
    )])
    .unwrap();
    let selector = CssSelector::Compound(value);
    assert_eq!(selector.to_specified_css().unwrap(), ":host(:has(> .One))");
}

#[test]
fn parts_charge_compound_nodes_and_their_own_relationship_bytes() {
    let value = compound("é");
    for (combinator, relationship) in [
        (Combinator::Descendant, " "),
        (Combinator::Child, " > "),
        (Combinator::NextSibling, " + "),
        (Combinator::SubsequentSibling, " ~ "),
        (Combinator::Column, " || "),
    ] {
        let part_css = format!("{relationship}.é");
        // A part adds no semantic aggregate beyond its two-node compound.
        let part = CssComplexSelectorPart::try_new_with_limits(
            combinator,
            value.clone(),
            Limits::new(2, 2, part_css.len()),
        )
        .unwrap();
        assert_eq!(part.combinator(), combinator);
        assert_eq!(part.selector(), &value);
        let complex = CssComplexSelector::try_new(compound("A"), vec![part.clone()]).unwrap();
        let selector = CssSelector::Complex(complex);
        let literal = format!(".A{part_css}");
        assert_eq!(selector.to_specified_css().unwrap(), literal);
        assert_eq!(parsed(&literal, &CssNamespaceContext::default()), selector);
        for (limits, cause) in [
            (Limits::new(1, 2, part_css.len()), Kind::InputNodeLimit),
            (Limits::new(2, 1, part_css.len()), Kind::ProjectionNodeLimit),
            (Limits::new(2, 2, part_css.len() - 1), Kind::ByteLimit),
        ] {
            specified_cause(
                &CssComplexSelectorPart::try_new_with_limits(combinator, value.clone(), limits)
                    .unwrap_err(),
                cause,
            );
            assert_eq!(
                CssComplexSelectorPart::try_new(combinator, value.clone()).unwrap(),
                part
            );
        }
    }
}

#[test]
fn complex_limits_charge_first_rest_and_aggregate_with_exact_typed_causes() {
    let first = compound("A");
    let part = CssComplexSelectorPart::try_new(Combinator::Child, compound("é")).unwrap();
    let literal = ".A > .é";
    // One complex + two compounds + two classes = five nodes.
    let value = CssComplexSelector::try_new_with_limits(
        first.clone(),
        vec![part.clone()],
        Limits::new(5, 5, literal.len()),
    )
    .unwrap();
    assert_eq!(value.first(), &first);
    assert_eq!(value.rest(), std::slice::from_ref(&part));
    assert_eq!(
        CssSelector::Complex(value.clone())
            .to_specified_css()
            .unwrap(),
        literal
    );
    for (limits, cause) in [
        (Limits::new(4, 5, literal.len()), Kind::InputNodeLimit),
        (Limits::new(5, 4, literal.len()), Kind::ProjectionNodeLimit),
        (Limits::new(5, 5, literal.len() - 1), Kind::ByteLimit),
    ] {
        specified_cause(
            &CssComplexSelector::try_new_with_limits(first.clone(), vec![part.clone()], limits)
                .unwrap_err(),
            cause,
        );
        assert_eq!(
            CssComplexSelector::try_new(first.clone(), vec![part.clone()]).unwrap(),
            value
        );
    }
}

#[test]
fn valid_terminal_part_is_rechecked_when_placed_before_a_following_compound() {
    let parsed = parsed(".A > .B::before", &CssNamespaceContext::default());
    let CssSelector::Complex(parsed) = parsed else {
        panic!("parsed complex");
    };
    let terminal =
        CssComplexSelectorPart::try_new(Combinator::Child, parsed.rest()[0].selector().clone())
            .unwrap();
    let value =
        CssComplexSelector::try_new(parsed.first().clone(), vec![terminal.clone()]).unwrap();
    assert_eq!(
        CssSelector::Complex(value).to_specified_css().unwrap(),
        ".A > .B::before"
    );
    let error = CssComplexSelector::try_new_with_limits(
        parsed.first().clone(),
        Vec::new(),
        Limits::new(0, 0, 0),
    )
    .unwrap_err();
    assert_eq!(error.kind(), &ConstructionKind::EmptyComplexRest);
    let following = CssComplexSelectorPart::try_new(Combinator::Child, compound("C")).unwrap();
    let error = CssComplexSelector::try_new_with_limits(
        terminal.selector().clone(),
        vec![following.clone()],
        Limits::new(0, 0, 0),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &ConstructionKind::NonTerminalPseudoElement { compound_index: 0 }
    );
    let error = CssComplexSelector::try_new_with_limits(
        parsed.first().clone(),
        vec![terminal, following],
        Limits::new(0, 0, 0),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &ConstructionKind::NonTerminalPseudoElement { compound_index: 1 }
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn parser_proved_complex_keeps_its_grammar_budget_before_programmatic_readmission() {
    // A clean parsed graph is not rejected or diagnosed as a pseudo-element
    // placement error merely because later default specified admission is smaller.
    let source = format!(".One.Two.Three{}", " > .Two".repeat(32_766));
    let selector = parsed(&source, &CssNamespaceContext::default());
    let CssSelector::Complex(value) = selector else {
        panic!("parsed complete complex");
    };
    assert_eq!(value.first().classes(), ["One", "Two", "Three"]);
    assert_eq!(value.rest().len(), 32_766);
    specified_cause(
        &CssComplexSelector::try_new(value.first().clone(), value.rest().to_vec()).unwrap_err(),
        Kind::InputNodeLimit,
    );
    let admitted = CssComplexSelector::try_new_with_limits(
        value.first().clone(),
        value.rest().to_vec(),
        Limits::new(65_537, 65_537, source.len()),
    )
    .unwrap();
    assert!(
        admitted == value,
        "explicit admission preserves the complete parsed graph"
    );
    assert!(
        CssSelector::Complex(admitted)
            .to_specified_css_with_limits(Limits::new(65_537, 65_537, source.len()))
            .unwrap()
            == source,
        "larger explicit output preserves the independently authored canonical source"
    );
}

#[test]
fn default_complex_byte_failure_keeps_its_cause_and_explicit_retry_preserves_identity() {
    let decoded = format!("{}a", "é".repeat(524_284));
    let first = CssCompoundSelector::try_new(vec![Simple::Class(
        CssIdent::try_new(decoded.clone()).unwrap(),
    )])
    .unwrap();
    let part = CssComplexSelectorPart::try_new(Combinator::Child, compound("Two")).unwrap();
    let literal = format!(".{decoded} > .Two");
    assert_eq!(literal.len(), 1_048_577);
    specified_cause(
        &CssComplexSelector::try_new(first.clone(), vec![part.clone()]).unwrap_err(),
        Kind::ByteLimit,
    );
    assert!(
        first.classes() == [decoded],
        "first compound retains decoded identity after failure"
    );
    let value = CssComplexSelector::try_new_with_limits(
        first.clone(),
        vec![part.clone()],
        Limits::new(5, 5, literal.len()),
    )
    .unwrap();
    assert!(
        value.first() == &first,
        "explicit retry keeps the complete first compound"
    );
    assert_eq!(value.rest(), [part]);
    assert!(
        CssSelector::Complex(value)
            .to_specified_css_with_limits(Limits::new(5, 5, literal.len()))
            .unwrap()
            == literal,
        "explicit retry preserves the independently authored UTF-8 output"
    );
}
