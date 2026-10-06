#![forbid(unsafe_code)]
//! Functional expectations for the new general relative-list front door.
//! Selected Selectors 4 sections 3.4, 16 and 17.2 define relations, grammar and
//! atomic contextual rejection. Has attachment retains its narrower context.

use surgeist_css::{
    CssAttributeCaseSensitivity, CssErrorCode, CssNamespaceConstraint, CssNamespaceContext,
    CssNamespaceName, CssNamespacePrefix, CssNthPattern, CssPseudoClass, CssQualifiedNamePrefix,
    CssRecoveryAction, CssRelativeSelectorList, CssSelector, CssSelectorCombinator as Combinator,
    CssSelectorConstructionErrorKind, CssSourcePosition,
    CssSpecifiedValueSerializationErrorKind as Kind,
    CssSpecifiedValueSerializationLimits as Limits, CssTokenKind, ErrorKind,
    parse_relative_selector_list, parse_selector, parse_selector_list,
};

fn clean_in(source: &str, context: &CssNamespaceContext) -> CssRelativeSelectorList {
    let report = parse_relative_selector_list(source, context);
    assert!(report.is_clean(), "{source:?}: {report:?}");
    report.into_validation_result().unwrap().unwrap()
}

fn clean(source: &str) -> CssRelativeSelectorList {
    clean_in(source, &CssNamespaceContext::default())
}

fn rejected_in(source: &str, context: &CssNamespaceContext) {
    let report = parse_relative_selector_list(source, context);
    assert!(
        report.syntax().is_none(),
        "no partial strict list: {source:?}: {report:?}"
    );
    assert!(!report.diagnostics().is_empty());
    assert_eq!(
        report.diagnostics().last().unwrap().action(),
        CssRecoveryAction::RejectInput
    );
    let diagnostics = report.diagnostics().to_vec();
    assert_eq!(
        report.into_validation_result().unwrap_err().diagnostics(),
        diagnostics
    );
}

fn position(actual: CssSourcePosition, expected: (usize, u32, u32)) {
    assert_eq!(actual.byte_offset().value(), expected.0);
    assert_eq!(actual.line().value(), expected.1);
    assert_eq!(actual.column().value(), expected.2);
}

#[test]
fn explicit_and_implicit_leading_relations_keep_ordered_typed_members() {
    let list = clean(".One, > .Two, + .Three, ~ .Four, || .Five");
    assert_eq!(list.selectors().len(), 5);
    for (member, relation, name) in [
        (&list.selectors()[0], Combinator::Descendant, "One"),
        (&list.selectors()[1], Combinator::Child, "Two"),
        (&list.selectors()[2], Combinator::NextSibling, "Three"),
        (&list.selectors()[3], Combinator::SubsequentSibling, "Four"),
        (&list.selectors()[4], Combinator::Column, "Five"),
    ] {
        assert_eq!(member.combinator(), relation);
        assert_eq!(member.selector(), &CssSelector::Class(name.into()));
    }
    let literal = ".One, > .Two, + .Three, ~ .Four, || .Five";
    assert_eq!(list.to_specified_css().unwrap(), literal);
    assert_eq!(clean(literal), list);
    assert_eq!(
        CssRelativeSelectorList::try_new(list.selectors().to_vec()).unwrap(),
        list
    );
}

#[test]
fn leading_relation_remains_separate_from_internal_complex_relationships() {
    let list = clean("+ .One > .Two ~ .Three || .Four");
    let [member] = list.selectors() else {
        panic!("one relative member");
    };
    assert_eq!(member.combinator(), Combinator::NextSibling);
    let CssSelector::Complex(complex) = member.selector() else {
        panic!("complete complex child");
    };
    assert_eq!(complex.first().classes(), ["One"]);
    for (part, relation, name) in [
        (&complex.rest()[0], Combinator::Child, "Two"),
        (&complex.rest()[1], Combinator::SubsequentSibling, "Three"),
        (&complex.rest()[2], Combinator::Column, "Four"),
    ] {
        assert_eq!(part.combinator(), relation);
        assert_eq!(part.selector().classes(), [name]);
    }
    assert_eq!(
        list.to_specified_css().unwrap(),
        "+ .One > .Two ~ .Three || .Four"
    );
}

#[test]
fn nested_commas_and_strings_do_not_split_outer_relative_members() {
    let list = clean(r#"> :is(.One,#Two), [Data="a,b"], + :nth-child(odd of .A,#B)"#);
    assert_eq!(list.selectors().len(), 3);
    assert_eq!(list.selectors()[0].combinator(), Combinator::Child);
    assert_eq!(list.selectors()[1].combinator(), Combinator::Descendant);
    assert_eq!(list.selectors()[2].combinator(), Combinator::NextSibling);
    let literal = r#"> :is(.One, #Two), [Data="a,b"], + :nth-child(2n+1 of .A, #B)"#;
    assert_eq!(list.to_specified_css().unwrap(), literal);
    let CssSelector::PseudoClass(CssPseudoClass::NthChild(authored)) =
        list.selectors()[2].selector()
    else {
        panic!("authored nth-child context");
    };
    assert_eq!(authored.pattern(), CssNthPattern::Odd);
    assert_eq!(
        authored.selector_list().unwrap().selectors(),
        [CssSelector::Class("A".into()), CssSelector::Key("B".into())]
    );
    let canonical = clean(literal);
    assert_eq!(canonical.selectors().len(), 3);
    assert_eq!(&canonical.selectors()[..2], &list.selectors()[..2]);
    assert_eq!(
        canonical.selectors()[2].combinator(),
        Combinator::NextSibling
    );
    let CssSelector::PseudoClass(CssPseudoClass::NthChild(reparsed)) =
        canonical.selectors()[2].selector()
    else {
        panic!("canonical nth-child context");
    };
    let CssNthPattern::AnPlusB(coefficients) = reparsed.pattern() else {
        panic!("canonical coefficient representation");
    };
    assert_eq!((coefficients.a(), coefficients.b()), (2, 1));
    assert_eq!(reparsed.selector_list(), authored.selector_list());
    assert_eq!(canonical.to_specified_css().unwrap(), literal);
}

#[test]
fn general_terminal_pseudo_elements_are_valid_until_has_attachment() {
    for literal in [
        "> .Cell::before",
        "|| .Column > .Cell::after",
        ".Cell::marker",
    ] {
        let list = clean(literal);
        assert_eq!(list.to_specified_css().unwrap(), literal);
        assert!(list.has_pseudo_elements());
        let attached = CssSelector::PseudoClass(CssPseudoClass::Has(list));
        assert_eq!(
            attached.to_specified_css().unwrap_err().kind(),
            Kind::UnrepresentableValue
        );
    }
    for source in ["> .Cell::before > .Other", ".Cell::after + .Other"] {
        rejected_in(source, &CssNamespaceContext::default());
    }
}

#[test]
fn independent_has_selectors_remain_general_relatives_without_authorizing_nested_has() {
    let list = clean("> :has(+ .One), :not(:has(|| .Two))");
    assert_eq!(
        list.to_specified_css().unwrap(),
        "> :has(+ .One), :not(:has(|| .Two))"
    );
    let attached = CssSelector::PseudoClass(CssPseudoClass::Has(list));
    assert_eq!(
        attached.to_specified_css().unwrap_err().kind(),
        Kind::UnrepresentableValue
    );
    for source in [":has(:has(.One))", "> :has(:not(:has(.One)))"] {
        rejected_in(source, &CssNamespaceContext::default());
    }
    // Has's restriction invalidates the inner Has; Is forgives that member,
    // retaining valid empty Is syntax and the original contextual diagnostic.
    let source = "> :has(:is(:has(.One)))";
    let recovered = parse_relative_selector_list(source, &CssNamespaceContext::default());
    assert!(!recovered.is_clean());
    let [member] = recovered.syntax().as_ref().unwrap().selectors() else {
        panic!("one recovered general relative");
    };
    assert_eq!(member.combinator(), Combinator::Child);
    let CssSelector::PseudoClass(CssPseudoClass::Has(argument)) = member.selector() else {
        panic!("retained outer Has");
    };
    let [inner] = argument.selectors() else {
        panic!("one retained strict Has argument");
    };
    assert_eq!(inner.combinator(), Combinator::Descendant);
    let CssSelector::PseudoClass(CssPseudoClass::Is(forgiving)) = inner.selector() else {
        panic!("retained forgiving Is");
    };
    assert!(forgiving.selectors().is_empty());
    assert_eq!(
        recovered
            .syntax()
            .as_ref()
            .unwrap()
            .to_specified_css()
            .unwrap(),
        "> :has(:is())"
    );
    let [diagnostic] = recovered.diagnostics() else {
        panic!("one original contextual rejection");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropSelectorListItem);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    position(diagnostic.error().position(), (16, 0, 16));
    position(diagnostic.span().start(), (11, 0, 11));
    position(diagnostic.span().end(), (21, 0, 21));
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("typed inherited Has restriction");
    };
    assert_eq!(
        detail.production().unwrap().as_str(),
        "baseline.selector.complex"
    );
    assert_eq!(detail.expectation().as_str(), "a supported selector");
    let token = detail.encountered().unwrap();
    assert_eq!(token.kind(), CssTokenKind::Delim);
    assert_eq!(token.authored(), ".");
    let diagnostics = recovered.diagnostics().to_vec();
    assert_eq!(
        recovered
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        diagnostics
    );
    for source in [":has(> :has(+ .One))", ":has(> .Cell::before)"] {
        assert!(
            parse_selector(source, &CssNamespaceContext::default())
                .syntax()
                .is_none()
        );
    }
    let valid = clean("> .One, + .Two, ~ .Three, || .Four");
    assert_eq!(
        CssSelector::PseudoClass(CssPseudoClass::Has(valid))
            .to_specified_css()
            .unwrap(),
        ":has(> .One, + .Two, ~ .Three, || .Four)"
    );
}

#[test]
fn selected_shadow_compound_has_admission_survives_the_general_front() {
    for literal in [
        "> :host(:has(> .One))",
        "+ :host-context(:has(.One))",
        "|| ::slotted(:has(:first-child:last-child))",
    ] {
        let list = clean(literal);
        assert_eq!(list.to_specified_css().unwrap(), literal);
        assert_eq!(clean(literal), list);
    }
    rejected_in(
        "> :host(:not(.One > .Two))",
        &CssNamespaceContext::default(),
    );
}

#[test]
fn empty_or_malformed_outer_members_reject_the_entire_strict_list() {
    for source in [
        "",
        " \t\r\n\u{c}/**/",
        ">",
        "+",
        "~",
        "||",
        ",.One",
        ".One,",
        ".One,,.Two",
        "> .One, + , .Two",
        "> .One, ???, .Two",
        "|| !",
        ">> .One",
        "> + .One",
        ".One; .Two",
        ".One}",
        ".One)",
        ".One]",
        ".One{color:red}",
        ".One,:not(.Two,???),.Three",
        ".One,:nth-child(odd of .Two,???),.Three",
        ".One,:NoPe,.Three",
        "> .One, > .Two;",
    ] {
        rejected_in(source, &CssNamespaceContext::default());
    }
}

#[test]
fn ordinary_fronts_reject_explicit_relatives_while_general_front_retains_carriers() {
    for source in ["> .One", "+ .One", "~ .One", "|| .One", ".One, > .Two"] {
        assert!(
            parse_selector(source, &CssNamespaceContext::default())
                .syntax()
                .is_none()
        );
        assert!(
            parse_selector_list(source, &CssNamespaceContext::default())
                .syntax()
                .is_none()
        );
        assert!(!clean(source).selectors().is_empty());
    }
    let ordinary = parse_selector_list(".One, .Two", &CssNamespaceContext::default())
        .into_validation_result()
        .unwrap()
        .unwrap();
    let relative = clean(".One, .Two");
    for (ordinary, relative) in ordinary.selectors().iter().zip(relative.selectors()) {
        assert_eq!(ordinary.selector(), relative.selector());
        assert_eq!(relative.combinator(), Combinator::Descendant);
    }
}

#[test]
fn namespaces_universals_and_original_case_are_retained_in_relative_children() {
    let prefix = CssNamespacePrefix::try_new("Svg").unwrap();
    let context = CssNamespaceContext::from_bindings([
        (None, CssNamespaceName::new("urn:default")),
        (Some(prefix.clone()), CssNamespaceName::new("urn:svg")),
    ]);
    let source = "> Leaf, + Svg|Thing[Svg|DATA=\"MiX\" I], ~ *|*, || |*, .Implicit";
    let list = clean_in(source, &context);
    for (member, namespace, authored_prefix, local) in [
        (
            &list.selectors()[0],
            CssNamespaceConstraint::Default,
            CssQualifiedNamePrefix::Unqualified,
            Some("Leaf"),
        ),
        (
            &list.selectors()[1],
            CssNamespaceConstraint::Named(prefix.clone()),
            CssQualifiedNamePrefix::Named(prefix.clone()),
            Some("Thing"),
        ),
        (
            &list.selectors()[2],
            CssNamespaceConstraint::Any,
            CssQualifiedNamePrefix::Any,
            None,
        ),
        (
            &list.selectors()[3],
            CssNamespaceConstraint::ExplicitNone,
            CssQualifiedNamePrefix::ExplicitNone,
            None,
        ),
    ] {
        let CssSelector::Compound(compound) = member.selector() else {
            panic!("qualified compound");
        };
        let name = compound.type_selector().unwrap();
        assert_eq!(name.namespace(), &namespace);
        assert_eq!(name.prefix(), &authored_prefix);
        assert_eq!(name.local_name(), local);
        assert_eq!(name.is_universal(), local.is_none());
    }
    let CssSelector::Compound(named) = list.selectors()[1].selector() else {
        panic!("named compound");
    };
    let attribute = &named.attributes()[0];
    assert_eq!(
        attribute.namespace(),
        &CssNamespaceConstraint::Named(prefix)
    );
    assert_eq!(attribute.name().as_str(), "DATA");
    assert_eq!(
        attribute.case_sensitivity(),
        CssAttributeCaseSensitivity::AsciiCaseInsensitive
    );
    assert_eq!(
        list.selectors()[4].selector(),
        &CssSelector::Class("Implicit".into())
    );
    let literal = "> Leaf, + Svg|Thing[Svg|DATA=\"MiX\" i], ~ *|*, || |*, .Implicit";
    assert_eq!(list.to_specified_css().unwrap(), literal);
    assert_eq!(clean_in(literal, &context), list);
    for source in [
        "> Svg|Leaf, + svg|Thing",
        "> .One, + Missing|*",
        "> [Missing|A], .Two",
    ] {
        rejected_in(source, &context);
    }
}

#[test]
fn comments_preserve_token_adjacency_and_only_css_whitespace_is_descendant() {
    for whitespace in [" ", "\t", "\n", "\r", "\u{c}"] {
        let source = format!(
            "{whitespace}>{whitespace}.One{whitespace},{whitespace}.Two{whitespace}.Three{whitespace}"
        );
        assert_eq!(
            clean(&source).to_specified_css().unwrap(),
            "> .One, .Two .Three"
        );
    }
    for (source, literal) in [
        ("/**/>/**/.One/**/,/**/.Two", "> .One, .Two"),
        (".One/**/.Two", ".One.Two"),
        (".One /**/.Two", ".One .Two"),
        ("|/**/|/**/.One", "|| .One"),
    ] {
        assert_eq!(clean(source).to_specified_css().unwrap(), literal);
    }
    for source in [">\u{b}.One", "| | .One", "| /**/|.One"] {
        rejected_in(source, &CssNamespaceContext::default());
    }
}

#[test]
fn escaped_leading_punctuation_loses_combinator_meaning() {
    for (source, decoded, literal) in [
        (r"\>", ">", r"\>"),
        (r"\+", "+", r"\+"),
        (r"\~", "~", r"\~"),
        (r"\|\|", "||", r"\|\|"),
        (r"\,", ",", r"\,"),
    ] {
        let list = clean(source);
        let [member] = list.selectors() else {
            panic!("one escaped type member");
        };
        assert_eq!(member.combinator(), Combinator::Descendant);
        assert_eq!(member.selector(), &CssSelector::Tag(decoded.into()));
        assert_eq!(list.to_specified_css().unwrap(), literal);
    }
}

#[test]
fn generic_relative_carriers_do_not_apply_nesting_prelude_classification() {
    let list = clean("&.One, > &.Two");
    assert_eq!(list.selectors()[0].combinator(), Combinator::Descendant);
    assert_eq!(list.selectors()[1].combinator(), Combinator::Child);
    for (member, class) in list.selectors().iter().zip(["One", "Two"]) {
        let CssSelector::Compound(compound) = member.selector() else {
            panic!("symbolic anchored compound");
        };
        assert_eq!(compound.nesting_selectors(), 1);
        assert_eq!(compound.scope_anchors(), 0);
        assert_eq!(compound.classes(), [class]);
    }
    assert_eq!(list.to_specified_css().unwrap(), "&.One, > &.Two");
}

#[test]
fn outer_error_keeps_original_unicode_crlf_token_and_complete_source_span() {
    let source = "/*😀*/\r\n> .é, + :NoPe";
    let report = parse_relative_selector_list(source, &CssNamespaceContext::default());
    assert!(report.syntax().is_none());
    let [diagnostic] = report.diagnostics() else {
        panic!("one outer rejection");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    position(diagnostic.error().position(), (20, 1, 9));
    position(diagnostic.span().start(), (0, 0, 0));
    position(diagnostic.span().end(), (24, 1, 13));
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("typed selector error");
    };
    let token = detail.encountered().unwrap();
    assert_eq!(token.kind(), CssTokenKind::Ident);
    assert_eq!(token.authored(), "NoPe");
    let diagnostics = report.diagnostics().to_vec();
    assert_eq!(
        report.into_validation_result().unwrap_err().diagnostics(),
        diagnostics
    );
}

#[test]
fn inner_forgiveness_retains_members_and_exact_original_diagnostics_without_becoming_clean() {
    let source = "/*😀*/\r\n> :is(.é, :NoPe), + .Two";
    let report = parse_relative_selector_list(source, &CssNamespaceContext::default());
    let list = report.syntax().as_ref().unwrap();
    assert_eq!(list.selectors().len(), 2);
    let CssSelector::PseudoClass(CssPseudoClass::Is(inner)) = list.selectors()[0].selector() else {
        panic!("retained forgiving Is");
    };
    assert_eq!(inner.selectors(), [CssSelector::Class("é".into())]);
    assert_eq!(list.to_specified_css().unwrap(), "> :is(.é), + .Two");
    let [diagnostic] = report.diagnostics() else {
        panic!("one dropped inner member");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropSelectorListItem);
    position(diagnostic.error().position(), (22, 1, 11));
    position(diagnostic.span().start(), (20, 1, 9));
    position(diagnostic.span().end(), (26, 1, 15));
    let diagnostics = report.diagnostics().to_vec();
    assert_eq!(
        report.into_validation_result().unwrap_err().diagnostics(),
        diagnostics
    );
}

#[test]
fn later_outer_error_keeps_prior_inner_diagnostics_but_returns_no_partial_list() {
    let report = parse_relative_selector_list(
        "> :is(:NoPe,.One), + :NoPe",
        &CssNamespaceContext::default(),
    );
    assert!(report.syntax().is_none());
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [
            CssRecoveryAction::DropSelectorListItem,
            CssRecoveryAction::RejectInput
        ]
    );
    let diagnostics = report.diagnostics().to_vec();
    assert_eq!(
        report.into_validation_result().unwrap_err().diagnostics(),
        diagnostics
    );
}

#[test]
fn actual_eof_closure_keeps_retained_relative_syntax_and_original_eof_position() {
    let source = "/*😀*/\r\n> :is(.é";
    let report = parse_relative_selector_list(source, &CssNamespaceContext::default());
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .to_specified_css()
            .unwrap(),
        "> :is(.é)"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one retained closure");
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    for actual in [
        diagnostic.error().position(),
        diagnostic.span().start(),
        diagnostic.span().end(),
    ] {
        position(actual, (19, 1, 8));
    }
    let diagnostics = report.diagnostics().to_vec();
    assert_eq!(
        report.into_validation_result().unwrap_err().diagnostics(),
        diagnostics
    );
}

#[test]
fn relative_fragments_share_the_exact_256_function_depth_and_257_rejection_boundary() {
    for depth in [255, 256] {
        let source = format!("> {}.One{}", ":is(".repeat(depth), ")".repeat(depth));
        let list = clean(&source);
        assert_eq!(list.selectors()[0].combinator(), Combinator::Child);
        let mut selector = list.selectors()[0].selector();
        for _ in 0..depth {
            let CssSelector::PseudoClass(CssPseudoClass::Is(inner)) = selector else {
                panic!("one logical level");
            };
            let [child] = inner.selectors() else {
                panic!("one retained member");
            };
            selector = child;
        }
        assert_eq!(selector, &CssSelector::Class("One".into()));
        assert_eq!(list.to_specified_css().unwrap(), source);
    }
    let source = format!("> {}.One{}", ":is(".repeat(257), ")".repeat(257));
    let report = parse_relative_selector_list(&source, &CssNamespaceContext::default());
    assert!(report.syntax().is_none());
    let [diagnostic] = report.diagnostics() else {
        panic!("one depth limit");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    position(diagnostic.error().position(), (1027, 0, 1027));
    position(diagnostic.span().start(), (0, 0, 0));
    position(
        diagnostic.span().end(),
        (source.len(), 0, source.len() as u32),
    );
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("typed limit");
    };
    assert_eq!(detail.limit(), 256);
    assert_eq!(
        detail.enclosing_production().as_str(),
        "baseline.selector.complex"
    );
    assert!(report.into_validation_result().is_err());
    assert_eq!(clean("> .After").to_specified_css().unwrap(), "> .After");
}

#[test]
fn large_clean_relative_list_keeps_parse_budget_separate_from_default_specified_admission() {
    // One list aggregate + 32,768 relative carriers + 32,768 scalar classes.
    let source = format!("{}.A", ".A, ".repeat(32_767));
    let report = parse_relative_selector_list(&source, &CssNamespaceContext::default());
    assert!(
        report.is_clean(),
        "large flat relative list must parse cleanly"
    );
    let list = report.into_validation_result().unwrap().unwrap();
    assert_eq!(list.selectors().len(), 32_768);
    assert!(
        list.selectors()
            .iter()
            .all(|member| member.combinator() == Combinator::Descendant
                && matches!(member.selector(), CssSelector::Class(name) if name == "A"))
    );
    assert_eq!(
        list.to_specified_css().unwrap_err().kind(),
        Kind::InputNodeLimit
    );
    let error = CssRelativeSelectorList::try_new(list.selectors().to_vec()).unwrap_err();
    let CssSelectorConstructionErrorKind::Specified(cause) = error.kind() else {
        panic!("typed aggregate error");
    };
    assert_eq!(cause.kind(), Kind::InputNodeLimit);
    assert!(
        list.to_specified_css_with_limits(Limits::new(65_537, 65_537, source.len()))
            .unwrap()
            == source,
        "explicit output preserves the independently authored ordered list"
    );
}
