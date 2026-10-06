#![forbid(unsafe_code)]
//! Authored Selectors 4 grammar through existing public parse, checked-name,
//! checked-compound and specified-output boundaries. Literal expectations follow
//! the selected 2026-01-22 grammar and Syntax 3, not captured provider output.
//! Selected shadow providers retain compound outer arguments and independent
//! relative :has() arguments, following their pinned operational source.
use surgeist_css::*;

fn context() -> CssNamespaceContext {
    CssNamespaceContext::from_bindings([
        (None, CssNamespaceName::new("urn:default")),
        (
            Some(CssNamespacePrefix::try_new("svg").unwrap()),
            CssNamespaceName::new("urn:svg"),
        ),
    ])
}

fn parsed_in(source: &str, context: &CssNamespaceContext) -> CssSelector {
    let report = parse_selector(source, context);
    assert!(report.is_clean(), "{source:?}: {report:?}");
    report
        .into_validation_result()
        .unwrap()
        .expect("one selector")
}

fn parsed(source: &str) -> CssSelector {
    parsed_in(source, &CssNamespaceContext::default())
}

fn exact_in(source: &str, expected: &str, context: &CssNamespaceContext) -> CssSelector {
    let selector = parsed_in(source, context);
    let before = selector.clone();
    assert_eq!(selector.to_specified_css().unwrap(), expected, "{source:?}");
    assert_eq!(selector, before, "specified output is immutable");
    let reparsed = parsed_in(expected, context);
    // Canonical output and the explicit typed assertions at each caller are the
    // independent oracle. Equality alone is insufficient (odd and 2n+1 differ
    // in authored pattern identity).
    assert_eq!(reparsed.to_specified_css().unwrap(), expected);
    selector
}

fn exact(source: &str, expected: &str) -> CssSelector {
    exact_in(source, expected, &CssNamespaceContext::default())
}

fn compound(selector: &CssSelector) -> &CssCompoundSelector {
    match selector {
        CssSelector::Compound(value) => value,
        other => panic!("compound selector: {other:?}"),
    }
}

fn logical_members(selector: &CssSelector) -> &[CssSelector] {
    match selector {
        CssSelector::PseudoClass(CssPseudoClass::Is(list) | CssPseudoClass::Where(list)) => {
            list.selectors()
        }
        other => panic!("forgiving selector: {other:?}"),
    }
}

fn reject_in(source: &str, context: &CssNamespaceContext) {
    let report = parse_selector(source, context);
    assert!(
        report.syntax().is_none(),
        "invalid selector retained: {source:?}: {report:?}"
    );
    let rejection = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
        .expect("one complete input rejection");
    assert_eq!(rejection.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(rejection.span().start().byte_offset().value(), 0);
    assert_eq!(rejection.span().end().byte_offset().value(), source.len());
    assert!(
        !report.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        }),
        "rejected syntax cannot publish retained EOF closure"
    );
    assert!(report.into_validation_result().is_err());
}

fn reject(source: &str) {
    reject_in(source, &CssNamespaceContext::default());
}

fn reject_list_in(source: &str, context: &CssNamespaceContext) {
    let report = parse_selector_list(source, context);
    assert!(
        report.syntax().is_none(),
        "partial strict list retained: {source:?}: {report:?}"
    );
    let rejection = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
        .expect("atomic list rejection");
    assert_eq!(rejection.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(rejection.span().start().byte_offset().value(), 0);
    assert_eq!(rejection.span().end().byte_offset().value(), source.len());
    assert!(report.into_validation_result().is_err());
}

fn pseudo_list(selector: CssSelector) -> CssPseudoSelectorList {
    CssPseudoSelectorList::try_new(vec![selector]).unwrap()
}

fn has(selector: CssSelector) -> CssSelector {
    CssSelector::PseudoClass(CssPseudoClass::Has(
        CssRelativeSelectorList::try_new(vec![CssRelativeSelector::new(
            CssSelectorCombinator::Child,
            selector,
        )])
        .unwrap(),
    ))
}

fn invalid_graph(selector: CssSelector) {
    let before = selector.clone();
    let result = selector.to_specified_css();
    assert!(result.is_err(), "invalid public graph emitted: {result:?}");
    assert_eq!(
        result.unwrap_err().kind(),
        CssSpecifiedValueSerializationErrorKind::UnrepresentableValue
    );
    assert_eq!(
        selector, before,
        "rejected emission must not mutate its input"
    );
}

#[test]
fn class_identifiers_decode_escapes_and_comment_adjacency() {
    for (source, name, canonical) in [
        (".class", "class", ".class"),
        (r".\63 lass", "class", ".class"),
        ("./**/class", "class", ".class"),
        (r".a\>b", "a>b", r".a\>b"),
        (r".\31 st", "1st", r".\31 st"),
        (r".\20 class", " class", r".\ class"),
    ] {
        assert_eq!(exact(source, canonical), CssSelector::Class(name.into()));
    }
}

#[test]
fn class_dot_cannot_be_separated_by_space() {
    assert_eq!(parsed("./**/class"), CssSelector::Class("class".into()));
    reject(". class");
}

#[test]
fn class_dot_cannot_be_separated_by_tab() {
    assert_eq!(parsed(r".\63 lass"), CssSelector::Class("class".into()));
    reject(".\tclass");
}

#[test]
fn class_dot_cannot_be_separated_by_line_feed() {
    assert_eq!(parsed(".class"), CssSelector::Class("class".into()));
    reject(".\nclass");
}

#[test]
fn class_dot_cannot_be_separated_by_carriage_return() {
    assert_eq!(parsed(".class"), CssSelector::Class("class".into()));
    reject(".\rclass");
}

#[test]
fn class_dot_cannot_be_separated_by_form_feed() {
    assert_eq!(parsed(".class"), CssSelector::Class("class".into()));
    reject(".\u{c}class");
}

#[test]
fn non_css_whitespace_is_identifier_content_or_an_invalid_token() {
    for non_ascii in ['\u{85}', '\u{a0}', '\u{2003}'] {
        let source = format!(".{non_ascii}class");
        assert_eq!(
            exact(&source, &source),
            CssSelector::Class(format!("{non_ascii}class"))
        );
        let source = format!("a{non_ascii}b");
        assert_eq!(
            exact(&source, &source),
            CssSelector::Tag(format!("a{non_ascii}b"))
        );
    }
    // Vertical tab is neither CSS whitespace nor a non-ASCII name code point.
    reject(".\u{b}class");
}

#[test]
fn class_whitespace_invalidates_the_complete_strict_list() {
    let control = parse_selector_list(".first,.class,.last", &CssNamespaceContext::default());
    assert!(control.is_clean());
    assert_eq!(control.syntax().as_ref().unwrap().selectors().len(), 3);
    reject_list_in(".first,. class,.last", &CssNamespaceContext::default());
}

#[test]
fn class_whitespace_drops_its_style_rule_and_preserves_neighbors() {
    let source = ".before{} . class { color:red; .child{} } .after{}";
    let report = parse_sheet(source);
    let rules = report.syntax().rules();
    assert_eq!(
        rules.len(),
        2,
        "invalid selector must not retain its rule/body"
    );
    for (rule, expected) in rules.iter().zip([".before", ".after"]) {
        let CssRule::Style(rule) = rule else {
            panic!("style neighbor")
        };
        assert_eq!(
            rule.selectors().selectors()[0]
                .selector()
                .to_specified_css()
                .unwrap(),
            expected
        );
    }
    let rejected = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.action() == CssRecoveryAction::DropQualifiedRule)
        .expect("whole qualified rule recovery");
    let failed = ". class { color:red; .child{} }";
    assert_eq!(
        rejected.span().start().byte_offset().value(),
        source.find(failed).unwrap()
    );
    assert_eq!(
        rejected.span().end().byte_offset().value(),
        source.find(failed).unwrap() + failed.len()
    );
    assert!(report.into_validation_result().is_err());
}

#[test]
fn identifier_hashes_and_ordered_repetitions_keep_decoded_identity() {
    assert_eq!(
        exact(r"#\31 23", r"#\31 23"),
        CssSelector::Key("123".into())
    );
    assert_eq!(exact(r"#A\>B", r"#A\>B"), CssSelector::Key("A>B".into()));
    for source in ["#", "#123", "#-", ".123", ".", ".#id"] {
        reject(source);
    }
    let selector = exact("Leaf#A#B#A.One.Two.One", "Leaf#A#B#A.One.Two.One");
    let value = compound(&selector);
    assert_eq!(value.type_selector().unwrap().local_name(), Some("Leaf"));
    assert_eq!(value.ids(), ["A", "B", "A"]);
    assert_eq!(value.classes(), ["One", "Two", "One"]);
}

#[test]
fn pseudo_keywords_ignore_ascii_case_without_folding_document_names() {
    let selector = exact(
        r"L\65 af#I\44 .Cl\61 ss[Da\74 a=Mi\58 eD I]:H\6f VeR",
        r#"Leaf#ID.Class[Data="MiXeD" i]:hover"#,
    );
    let value = compound(&selector);
    assert_eq!(value.type_selector().unwrap().local_name(), Some("Leaf"));
    assert_eq!(value.ids(), ["ID"]);
    assert_eq!(value.classes(), ["Class"]);
    assert_eq!(value.attributes()[0].name().as_str(), "Data");
    assert_eq!(
        value.attributes()[0].matcher(),
        &CssAttributeMatcher::Equals("MiXeD".into())
    );
    assert_eq!(
        value.attributes()[0].case_sensitivity(),
        CssAttributeCaseSensitivity::AsciiCaseInsensitive
    );
    assert_eq!(value.pseudo_classes(), [CssPseudoClass::Hover]);
    // Escaped punctuation belongs to one decoded identifier, not a combinator.
    assert_eq!(exact(r"a\+b", r"a\+b"), CssSelector::Tag("a+b".into()));
    assert_eq!(exact(r".a\|b", r".a\|b"), CssSelector::Class("a|b".into()));
}

#[test]
fn every_attribute_matcher_retains_name_value_and_modifier() {
    let cases = [
        ("[Data]", "[Data]", CssAttributeMatcher::Exists),
        (
            "[Data=Word]",
            r#"[Data="Word"]"#,
            CssAttributeMatcher::Equals("Word".into()),
        ),
        (
            "[Data~=Word]",
            r#"[Data~="Word"]"#,
            CssAttributeMatcher::Includes("Word".into()),
        ),
        (
            "[Data|=Word]",
            r#"[Data|="Word"]"#,
            CssAttributeMatcher::DashMatch("Word".into()),
        ),
        (
            "[Data^=Word]",
            r#"[Data^="Word"]"#,
            CssAttributeMatcher::Prefix("Word".into()),
        ),
        (
            "[Data$=Word]",
            r#"[Data$="Word"]"#,
            CssAttributeMatcher::Suffix("Word".into()),
        ),
        (
            "[Data*=Word]",
            r#"[Data*="Word"]"#,
            CssAttributeMatcher::Substring("Word".into()),
        ),
    ];
    for (source, expected, matcher) in cases {
        let selector = exact(source, expected);
        let [attribute] = compound(&selector).attributes() else {
            panic!("one attribute")
        };
        assert_eq!(attribute.name().as_str(), "Data");
        assert_eq!(attribute.matcher(), &matcher);
        assert_eq!(
            attribute.case_sensitivity(),
            CssAttributeCaseSensitivity::DocumentDefault
        );
    }
    for operator in ["=", "~=", "|=", "^=", "$=", "*="] {
        let source = format!("[Data{operator}'MiXeD']");
        let expected = format!("[Data{operator}\"MiXeD\"]");
        exact(&source, &expected);
        let source = format!("[Data{operator}'']");
        let expected = format!("[Data{operator}\"\"]");
        exact(&source, &expected); // match-nothing operands are still valid grammar.
    }
}

#[test]
fn attribute_modifiers_have_one_trailing_ascii_insensitive_identity() {
    for (source, expected, identity) in [
        (
            r#"[Data="V" I]"#,
            r#"[Data="V" i]"#,
            CssAttributeCaseSensitivity::AsciiCaseInsensitive,
        ),
        (
            r#"[Data="V"s]"#,
            r#"[Data="V" s]"#,
            CssAttributeCaseSensitivity::ExplicitSensitive,
        ),
        (
            r"[Data=V \69]",
            r#"[Data="V" i]"#,
            CssAttributeCaseSensitivity::AsciiCaseInsensitive,
        ),
        (
            r"[Data=V \53]",
            r#"[Data="V" s]"#,
            CssAttributeCaseSensitivity::ExplicitSensitive,
        ),
    ] {
        let selector = exact(source, expected);
        assert_eq!(
            compound(&selector).attributes()[0].case_sensitivity(),
            identity
        );
        assert_eq!(
            compound(&selector).attributes()[0].matcher(),
            &CssAttributeMatcher::Equals("V".into())
        );
    }
    for source in [
        "[Data i]",
        "[Data=V i s]",
        "[Data=V s s]",
        "[Data=V q]",
        "[Data=V 'i']",
        "[Data=V i extra]",
        "[Data=]",
        "[Data~=]",
        "[Data|=]",
        "[Data^=]",
        "[Data$=]",
        "[Data*=]",
        "[Data==V]",
        "[Data~V]",
        "[Data^V]",
        "[Data$V]",
        "[Data*V]",
        "[Data=1]",
        "[Data=var(--x)]",
        "[Data=V,]",
        "[*]",
        "[Data|]",
    ] {
        reject(source);
    }
    let escaped = exact(r"[Data\=V]", r"[Data\=V]");
    assert_eq!(compound(&escaped).attributes()[0].name().as_str(), "Data=V");
    assert_eq!(
        compound(&escaped).attributes()[0].matcher(),
        &CssAttributeMatcher::Exists
    );
    for (source, expected, name) in [
        (r"[Data\~=V]", r#"[Data\~="V"]"#, "Data~"),
        (r"[Data\|=V]", r#"[Data\|="V"]"#, "Data|"),
    ] {
        let selector = exact(source, expected);
        assert_eq!(compound(&selector).attributes()[0].name().as_str(), name);
        assert_eq!(
            compound(&selector).attributes()[0].matcher(),
            &CssAttributeMatcher::Equals("V".into())
        );
    }
}

#[test]
fn qualified_names_preserve_prefix_case_and_default_attribute_exclusion() {
    let context = context();
    let selector = exact_in(
        r"s\76 g|Leaf[svg|Data=V][*|Any][|None][Plain]",
        r#"svg|Leaf[svg|Data="V"][*|Any][|None][Plain]"#,
        &context,
    );
    let value = compound(&selector);
    assert_eq!(
        value.type_selector().unwrap().namespace(),
        &CssNamespaceConstraint::Named(CssNamespacePrefix::try_new("svg").unwrap())
    );
    assert_eq!(value.type_selector().unwrap().local_name(), Some("Leaf"));
    assert_eq!(
        value.attributes()[0].namespace(),
        &CssNamespaceConstraint::Named(CssNamespacePrefix::try_new("svg").unwrap())
    );
    assert_eq!(
        value.attributes()[1].namespace(),
        &CssNamespaceConstraint::Any
    );
    assert_eq!(
        value.attributes()[2].namespace(),
        &CssNamespaceConstraint::ExplicitNone
    );
    assert_eq!(
        value.attributes()[3].namespace(),
        &CssNamespaceConstraint::ExplicitNone
    );
    for source in [
        "SVG|Leaf",
        "missing|Leaf",
        "[SVG|Data]",
        "[missing|Data]",
        "svg| Leaf",
        "[svg |Data]",
        "[svg| Data]",
    ] {
        reject_in(source, &context);
    }
    let descendant = exact_in("svg |Leaf", "svg |Leaf", &context);
    let CssSelector::Complex(descendant) = descendant else {
        panic!("two type selectors separated by the descendant combinator")
    };
    let first = descendant.first().type_selector().unwrap();
    assert_eq!(first.namespace(), &CssNamespaceConstraint::Default);
    assert_eq!(first.local_name(), Some("svg"));
    let [part] = descendant.rest() else {
        panic!("one descendant part")
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Descendant);
    let second = part.selector().type_selector().unwrap();
    assert_eq!(second.namespace(), &CssNamespaceConstraint::ExplicitNone);
    assert_eq!(second.local_name(), Some("Leaf"));
    reject_list_in(".ok, missing|Leaf, .last", &context);
}

#[test]
fn checked_qualified_names_reject_undeclared_prefixes_and_preserve_decoded_names() {
    let context = context();
    let named = CssQualifiedNamePrefix::Named(CssNamespacePrefix::try_new("svg").unwrap());
    let name = CssQualifiedSelectorName::try_new(
        named.clone(),
        CssIdent::try_new("Leaf").unwrap(),
        &context,
    )
    .unwrap();
    assert_eq!(name.serialize_with_limit(8).unwrap().as_css(), "svg|Leaf");
    let attribute = CssQualifiedAttributeName::try_new(
        named,
        CssAttributeName::try_new("Data").unwrap(),
        &context,
    )
    .unwrap();
    assert_eq!(
        attribute.serialize_with_limit(8).unwrap().as_css(),
        "svg|Data"
    );
    let missing = CssNamespacePrefix::try_new("SVG").unwrap();
    let expected = CssQualifiedNameError::UndeclaredNamespacePrefix(missing.clone());
    assert_eq!(
        CssQualifiedSelectorName::try_new(
            CssQualifiedNamePrefix::Named(missing.clone()),
            CssIdent::try_new("Leaf").unwrap(),
            &context
        ),
        Err(expected.clone())
    );
    assert_eq!(
        CssQualifiedSelectorName::try_universal(
            CssQualifiedNamePrefix::Named(missing.clone()),
            &context
        ),
        Err(expected.clone())
    );
    assert_eq!(
        CssQualifiedAttributeName::try_new(
            CssQualifiedNamePrefix::Named(missing),
            CssAttributeName::try_new("Data").unwrap(),
            &context
        ),
        Err(expected)
    );
}

#[test]
fn explicit_universals_survive_logical_arguments_and_implicit_types_remain_absent() {
    let context = context();
    for (source, constraint) in [
        ("*.One", CssNamespaceConstraint::Default),
        ("*|*.One", CssNamespaceConstraint::Any),
        ("|*.One", CssNamespaceConstraint::ExplicitNone),
        (
            "svg|*.One",
            CssNamespaceConstraint::Named(CssNamespacePrefix::try_new("svg").unwrap()),
        ),
    ] {
        let selector = exact_in(source, source, &context);
        let name = compound(&selector).type_selector().unwrap();
        assert!(name.is_universal());
        assert_eq!(name.local_name(), None);
        assert_eq!(name.namespace(), &constraint);
    }
    let selector = parsed_in(":is(svg|*.One, |*#Two, *.Three)", &context);
    let members = logical_members(&selector);
    assert_eq!(members.len(), 3);
    assert_eq!(
        compound(&members[0]).type_selector().unwrap().namespace(),
        &CssNamespaceConstraint::Named(CssNamespacePrefix::try_new("svg").unwrap())
    );
    assert_eq!(
        compound(&members[1]).type_selector().unwrap().namespace(),
        &CssNamespaceConstraint::ExplicitNone
    );
    assert_eq!(
        compound(&members[2]).type_selector().unwrap().namespace(),
        &CssNamespaceConstraint::Default
    );
    assert_eq!(parsed(".One"), CssSelector::Class("One".into()));
    for source in [
        "a*",
        "*a",
        "a|b|c",
        ".One|Leaf",
        "[Data]Leaf",
        "Leaf::before.One",
    ] {
        reject_in(source, &context);
    }
}

#[test]
fn combinators_keep_their_relation_and_one_rightmost_subject() {
    let selector = exact("A\tB >\nC+\rD~\u{c}E", "A B > C + D ~ E");
    let CssSelector::Complex(value) = selector else {
        panic!("complex selector")
    };
    assert_eq!(
        value.first().type_selector().unwrap().local_name(),
        Some("A")
    );
    assert_eq!(
        value
            .rest()
            .iter()
            .map(CssComplexSelectorPart::combinator)
            .collect::<Vec<_>>(),
        [
            CssSelectorCombinator::Descendant,
            CssSelectorCombinator::Child,
            CssSelectorCombinator::NextSibling,
            CssSelectorCombinator::SubsequentSibling
        ]
    );
    assert_eq!(
        value
            .rest()
            .iter()
            .map(|part| part
                .selector()
                .type_selector()
                .unwrap()
                .local_name()
                .unwrap())
            .collect::<Vec<_>>(),
        ["B", "C", "D", "E"]
    );
    for whitespace in [" ", "\t", "\n", "\r", "\u{c}"] {
        exact(&format!("A{whitespace}B"), "A B");
        exact(&format!("A{whitespace}>{whitespace}B"), "A > B");
    }
    exact("A/**/.One", "A.One"); // comments alone do not create descendant relation.
    for source in [
        "> A", "+ A", "~ A", "A >", "A +", "A ~", "A > + B", "A >> B", "A / B",
    ] {
        reject(source);
    }
}

#[test]
fn column_combinator_is_admitted_by_the_complete_selected_complex_grammar() {
    exact("col > td", "col > td");
    // Selectors 4's selected formal grammar includes the adjacent '|' '|'
    // combinator. This existing parser/output witness does not require a new
    // public enum symbol in order to establish executable RED.
    exact("col||td", "col || td");
}

#[test]
fn column_combinator_is_admitted_at_a_relative_has_boundary() {
    exact(":has(> td)", ":has(> td)");
    exact(":has(||td)", ":has(|| td)");
}

#[test]
fn checked_complex_and_list_construction_preserve_nonempty_order() {
    let control = parsed("A > B");
    let CssSelector::Complex(value) = control else {
        panic!("complex")
    };
    let checked =
        CssComplexSelector::try_new(value.first().clone(), value.rest().to_vec()).unwrap();
    assert_eq!(
        CssSelector::Complex(checked).to_specified_css().unwrap(),
        "A > B"
    );
    assert!(CssComplexSelector::try_new(value.first().clone(), Vec::new()).is_err());
    let pseudo = parsed("A::before");
    assert!(CssComplexSelector::try_new(compound(&pseudo).clone(), value.rest().to_vec()).is_err());
    let checked = CssSelectorList::try_new(vec![
        CssSelector::Class("One".into()),
        CssSelector::Key("Two".into()),
    ])
    .unwrap();
    assert_eq!(
        checked.selectors(),
        [
            CssSelector::Class("One".into()),
            CssSelector::Key("Two".into())
        ]
    );
    assert!(CssSelectorList::try_new(Vec::new()).is_err());
    assert!(CssPseudoSelectorList::try_new(Vec::new()).is_err());
    assert!(CssRelativeSelectorList::try_new(Vec::new()).is_err());
}

#[test]
fn strict_lists_keep_root_commas_distinct_from_nested_commas_and_reject_atomically() {
    let report = parse_selector_list(
        r#".One,:is(.Two,#Three),[Data="a,b"]"#,
        &CssNamespaceContext::default(),
    );
    assert!(report.is_clean(), "{report:?}");
    let members = report.syntax().as_ref().unwrap().selectors();
    assert_eq!(members.len(), 3);
    for (member, expected) in members
        .iter()
        .zip([".One", ":is(.Two, #Three)", r#"[Data="a,b"]"#])
    {
        assert!(matches!(member, CssStyleSelector::Selector(_)));
        assert_eq!(member.selector().to_specified_css().unwrap(), expected);
    }
    for source in [
        "",
        " \r\n/**/",
        ".One,",
        ",.One",
        ".One,,.Two",
        ".One,???,.Two",
        ".One,> .Two",
        ".One,:not(.ok,???),.Two",
    ] {
        reject_list_in(source, &CssNamespaceContext::default());
    }
    reject(".One,.Two"); // one-selector entry point cannot consume a list.
}

#[test]
fn positional_pseudos_preserve_anb_and_strict_of_list_identity() {
    for (source, expected, a, b) in [
        (
            ":nth-child(ODD OF .One,#Two)",
            ":nth-child(2n+1 of .One, #Two)",
            2,
            1,
        ),
        (":nth-child(EVEN)", ":nth-child(2n)", 2, 0),
        (":nth-child(-n + 3)", ":nth-child(-n+3)", -1, 3),
        (":nth-child(+n-2)", ":nth-child(n-2)", 1, -2),
        (":nth-child(0n + 2)", ":nth-child(2)", 0, 2),
        (":nth-child(-3n + 4)", ":nth-child(-3n+4)", -3, 4),
    ] {
        let selector = exact(source, expected);
        let CssSelector::PseudoClass(CssPseudoClass::NthChild(value)) = selector else {
            panic!("nth-child")
        };
        let coefficients = match value.pattern() {
            CssNthPattern::Odd => (2, 1),
            CssNthPattern::Even => (2, 0),
            CssNthPattern::Integer(b) => (0, b),
            CssNthPattern::AnPlusB(value) => (value.a(), value.b()),
            other => panic!("unexpected pattern: {other:?}"),
        };
        assert_eq!(coefficients, (a, b));
        if source.contains("OF") {
            assert_eq!(
                value.selector_list().unwrap().selectors(),
                [
                    CssSelector::Class("One".into()),
                    CssSelector::Key("Two".into())
                ]
            );
        } else {
            assert!(value.selector_list().is_none());
        }
    }
    assert!(matches!(
        exact(
            ":nth-last-child(3 of .One > .Two)",
            ":nth-last-child(3 of .One > .Two)"
        ),
        CssSelector::PseudoClass(CssPseudoClass::NthLastChild(_))
    ));
    assert!(matches!(
        exact(":nth-of-type(odd)", ":nth-of-type(2n+1)"),
        CssSelector::PseudoClass(CssPseudoClass::NthOfType(CssNthPattern::Odd))
    ));
    assert!(matches!(
        exact(":nth-last-of-type(even)", ":nth-last-of-type(2n)"),
        CssSelector::PseudoClass(CssPseudoClass::NthLastOfType(CssNthPattern::Even))
    ));
    for (source, expected) in [
        (":FIRST-OF-TYPE", CssPseudoClass::FirstOfType),
        (":LAST-OF-TYPE", CssPseudoClass::LastOfType),
        (":ONLY-OF-TYPE", CssPseudoClass::OnlyOfType),
        (":FIRST-CHILD", CssPseudoClass::FirstChild),
        (":LAST-CHILD", CssPseudoClass::LastChild),
        (":ONLY-CHILD", CssPseudoClass::OnlyChild),
    ] {
        assert_eq!(parsed(source), CssSelector::PseudoClass(expected));
    }
    for source in [
        ":nth-child()",
        ":nth-child(2 n)",
        ":nth-child(n 2)",
        ":nth-child(2n+)",
        ":nth-child(2n of)",
        ":nth-child(2n of .ok,)",
        ":nth-child(2n of .ok,???)",
        ":nth-child(2n of > .ok)",
        ":nth-child(2n of ::before)",
        ":nth-child(2n of .ok,::before)",
        ":nth-child(2n of :not(::before))",
        ":nth-of-type(2n of .ok)",
        ":nth-last-of-type(odd of .ok)",
        ":first-of-type()",
        ":only-child(x)",
    ] {
        reject(source);
    }
}

#[test]
fn valid_checked_nth_and_logical_graphs_emit_canonical_grammar() {
    let selector = CssSelector::PseudoClass(CssPseudoClass::NthChild(CssNthChildPattern::new(
        CssNthPattern::AnPlusB(CssNthAnPlusB::new(-1, 3)),
        Some(
            CssPseudoSelectorList::try_new(vec![
                CssSelector::Class("One".into()),
                CssSelector::Key("Two".into()),
            ])
            .unwrap(),
        ),
    )));
    assert_eq!(
        selector.to_specified_css().unwrap(),
        ":nth-child(-n+3 of .One, #Two)"
    );
    exact(
        ":nth-child(-n+3 of .One, #Two)",
        ":nth-child(-n+3 of .One, #Two)",
    );
    let selector = CssSelector::PseudoClass(CssPseudoClass::Not(pseudo_list(CssSelector::Class(
        "One".into(),
    ))));
    assert_eq!(selector.to_specified_css().unwrap(), ":not(.One)");
    assert!(CssCompoundSelectorArgument::try_new(selector).is_some());
}

#[test]
fn forgiving_members_drop_invalid_selectors_without_becoming_strict_clean_reports() {
    for name in ["is", "where"] {
        let source = format!(":{name}(.One, :unknown(x,y), #Two)");
        let report = parse_selector(&source, &CssNamespaceContext::default());
        let selector = report
            .syntax()
            .as_ref()
            .expect("retained forgiving selector");
        assert_eq!(
            logical_members(selector),
            [
                CssSelector::Class("One".into()),
                CssSelector::Key("Two".into())
            ]
        );
        assert_eq!(
            selector.to_specified_css().unwrap(),
            format!(":{name}(.One, #Two)")
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one discarded member: {report:?}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropSelectorListItem);
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        let dropped = " :unknown(x,y)";
        assert_eq!(
            &source[diagnostic.span().start().byte_offset().value()
                ..diagnostic.span().end().byte_offset().value()],
            dropped
        );
        assert!(report.into_validation_result().is_err());
    }
    for source in [":not(.One,:unknown(x))", ":has(.One,:unknown(x))"] {
        reject(source);
    }
}

#[test]
fn forgiving_empty_lists_remain_valid_authored_states() {
    for name in ["is", "where"] {
        let empty = parse_selector(&format!(":{name}()"), &CssNamespaceContext::default());
        assert!(logical_members(empty.syntax().as_ref().expect("valid empty result")).is_empty());
        assert_eq!(
            empty.syntax().as_ref().unwrap().to_specified_css().unwrap(),
            format!(":{name}()")
        );
        let report = parse_selector(
            &format!(":{name}(::before,???)"),
            &CssNamespaceContext::default(),
        );
        assert!(logical_members(report.syntax().as_ref().unwrap()).is_empty());
        assert_eq!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .to_specified_css()
                .unwrap(),
            format!(":{name}()")
        );
        assert_eq!(report.diagnostics().len(), 2);
        assert!(
            report.diagnostics().iter().all(|diagnostic| {
                diagnostic.action() == CssRecoveryAction::DropSelectorListItem
            })
        );
        assert!(report.into_validation_result().is_err());
    }
    reject(":not()");
    reject(":has()");
}

#[test]
fn pseudo_elements_and_nested_has_are_restricted_through_nested_logical_functions() {
    for source in [
        ":not(::before)",
        ":not(.One,::before)",
        ":has(::before)",
        ":has(.One,::before)",
        ":has(:not(::before))",
        ":has(:has(.One))",
        ":has(:not(:has(.One)))",
        ":has(:nth-child(2n of :has(.One)))",
    ] {
        reject(source);
    }
    for (source, expected) in [
        (":is(::before,.One)", ":is(.One)"),
        (":where(:not(::before),.One)", ":where(.One)"),
        (":is(:where(::before,.Two),.One)", ":is(:where(.Two), .One)"),
        (":has(:is(:has(.bad),.One))", ":has(:is(.One))"),
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert_eq!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .to_specified_css()
                .unwrap(),
            expected
        );
        assert!(
            report.diagnostics().iter().any(|diagnostic| {
                diagnostic.action() == CssRecoveryAction::DropSelectorListItem
            })
        );
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn lexical_envelope_failure_cannot_be_forgiven_even_with_valid_members() {
    for source in [
        ":is(.One,])",
        ":where(.One,url(a b))",
        ":is(.One,\"bad\n)",
        ":is(.One,.Two{)",
        ":where(.One,:is(.Two{))",
    ] {
        reject(source);
    }
}

#[test]
fn missing_eof_closure_is_reported_only_for_retained_syntax() {
    let source = ":is(.One";
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert_eq!(
        logical_members(report.syntax().as_ref().unwrap()),
        [CssSelector::Class("One".into())]
    );
    let [closure] = report.diagnostics() else {
        panic!("one actual EOF closure")
    };
    assert_eq!(
        closure.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(closure.error().code(), CssErrorCode::UnexpectedEnd);
    assert_eq!(
        closure.error().position().byte_offset().value(),
        source.len()
    );
    assert_eq!(closure.span().start().byte_offset().value(), source.len());
    assert_eq!(closure.span().end().byte_offset().value(), source.len());
    assert!(report.into_validation_result().is_err());
    reject(":not(.One,???");
}

#[test]
fn nested_logical_members_inherit_compound_only_and_suffix_restrictions() {
    for function in [":host", ":host-context", "::slotted"] {
        for name in ["is", "where"] {
            let source = format!("{function}(:{name}(.bad > .child,.One))");
            let report = parse_selector(&source, &CssNamespaceContext::default());
            assert_eq!(
                report
                    .syntax()
                    .as_ref()
                    .unwrap()
                    .to_specified_css()
                    .unwrap(),
                format!("{function}(:{name}(.One))")
            );
            assert_eq!(report.diagnostics().len(), 1);
            assert_eq!(
                report.diagnostics()[0].action(),
                CssRecoveryAction::DropSelectorListItem
            );
            assert!(report.into_validation_result().is_err());
            let source = format!("{function}(:{name}(:not(.bad > .child),.One))");
            let report = parse_selector(&source, &CssNamespaceContext::default());
            assert_eq!(
                report
                    .syntax()
                    .as_ref()
                    .unwrap()
                    .to_specified_css()
                    .unwrap(),
                format!("{function}(:{name}(.One))")
            );
            assert!(report.into_validation_result().is_err());
            let empty_source = format!("{function}(:{name}())");
            let empty = parse_selector(&empty_source, &CssNamespaceContext::default());
            assert_eq!(
                empty
                    .syntax()
                    .as_ref()
                    .expect("valid empty compound-context result")
                    .to_specified_css()
                    .unwrap(),
                empty_source
            );
        }
        reject(&format!("{function}(:not(.bad > .child))"));
        reject(&format!("{function}(:not(:not(.bad > .child)))"));
    }
    let report = parse_selector(
        "::before:where(:focus,.bad)",
        &CssNamespaceContext::default(),
    );
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .to_specified_css()
            .unwrap(),
        "::before:where(:focus)"
    );
    assert!(report.into_validation_result().is_err());
    reject("::before:not(.bad)");
    exact(":not(.One > .Two)", ":not(.One > .Two)");
}

#[test]
fn checked_compound_arguments_reject_nested_logical_complex_members() {
    let class = CssSelector::Class("One".into());
    assert!(CssCompoundSelectorArgument::try_new(class).is_some());
    let complex = parsed(".One > .Two");
    for pseudo in [
        CssPseudoClass::Not(pseudo_list(complex.clone())),
        CssPseudoClass::Is(pseudo_list(complex.clone())),
        CssPseudoClass::Where(pseudo_list(CssSelector::PseudoClass(CssPseudoClass::Not(
            pseudo_list(complex),
        )))),
    ] {
        assert!(CssCompoundSelectorArgument::try_new(CssSelector::PseudoClass(pseudo)).is_none());
    }
}

#[test]
fn relative_has_keeps_each_explicit_or_implicit_anchor_relation() {
    let selector = exact(
        ":has(.One,>.Two,+.Three,~.Four)",
        ":has(.One, > .Two, + .Three, ~ .Four)",
    );
    let CssSelector::PseudoClass(CssPseudoClass::Has(list)) = selector else {
        panic!("has")
    };
    assert_eq!(
        list.selectors()
            .iter()
            .map(CssRelativeSelector::combinator)
            .collect::<Vec<_>>(),
        [
            CssSelectorCombinator::Descendant,
            CssSelectorCombinator::Child,
            CssSelectorCombinator::NextSibling,
            CssSelectorCombinator::SubsequentSibling
        ]
    );
    assert_eq!(
        list.selectors()
            .iter()
            .map(|relative| relative.selector().clone())
            .collect::<Vec<_>>(),
        [
            CssSelector::Class("One".into()),
            CssSelector::Class("Two".into()),
            CssSelector::Class("Three".into()),
            CssSelector::Class("Four".into())
        ]
    );
    let checked = has(CssSelector::Class("Two".into()));
    assert_eq!(checked.to_specified_css().unwrap(), ":has(> .Two)");
    for source in [
        ":has(>)",
        ":has(> .One,+)",
        ":has(>.One,???)",
        ":not(>.One)",
        ":nth-child(2n of >.One)",
    ] {
        reject(source);
    }
    for source in ["> .One", "+ .One", "~ .One"] {
        reject(source);
        reject_list_in(source, &CssNamespaceContext::default());
    }
}

#[test]
fn admitted_nested_relative_forms_keep_boundary_without_rewriting_anchors() {
    let report = parse_sheet(".Parent { > .One { } + .Two { } ~ .Three { } & > .Four { } }");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent")
    };
    assert_eq!(parent.rules().len(), 4);
    for (rule, (relation, expected)) in parent.rules()[..3].iter().zip([
        (CssSelectorCombinator::Child, ".One"),
        (CssSelectorCombinator::NextSibling, ".Two"),
        (CssSelectorCombinator::SubsequentSibling, ".Three"),
    ]) {
        let CssRule::Style(rule) = rule else {
            panic!("relative child")
        };
        let [CssStyleSelector::Relative(relative)] = rule.selectors().selectors() else {
            panic!("relative boundary")
        };
        assert_eq!(relative.combinator(), relation);
        assert_eq!(relative.selector().to_specified_css().unwrap(), expected);
    }
    let CssRule::Style(rule) = &parent.rules()[3] else {
        panic!("anchored child")
    };
    let [CssStyleSelector::Selector(selector)] = rule.selectors().selectors() else {
        panic!("explicit anchor")
    };
    assert_eq!(selector.to_specified_css().unwrap(), "& > .Four");
    let CssSelector::Complex(value) = selector else {
        panic!("anchor complex")
    };
    assert_eq!(value.first().nesting_selectors(), 1);
    assert_eq!(value.first().scope_anchors(), 0);
}

#[test]
fn shadow_compound_arguments_admit_has_with_independent_relative_arguments() {
    // Selected Shadow's own has test references and frozen WebKit corroborate
    // this shadow grammar behavior. Actual complex outer arguments remain
    // forbidden, and logical pseudos inherit that outer compound restriction.
    for source in [
        ":host(:has(.One))",
        ":host-context(:has(.One > .Two))",
        "::slotted(:has(> .One))",
        ":host(:is(:has(.One),.Two))",
    ] {
        let expected = match source {
            ":host(:is(:has(.One),.Two))" => ":host(:is(:has(.One), .Two))",
            other => other,
        };
        exact(source, expected);
    }
    assert!(CssCompoundSelectorArgument::try_new(has(CssSelector::Class("One".into()))).is_some());
    reject(":host(.One > .Two)");
    reject(":host(:not(.One > .Two))");
    reject(":has(:host(:has(.One)))");
}

#[test]
fn public_not_graph_cannot_emit_a_pseudo_element_member() {
    exact(":not(.One)", ":not(.One)");
    // Checked admission now prevents this graph. The exact later writer
    // defense is retained in the owning private selector-list tests.
    assert_rejects_pseudo_element_list();
}

#[test]
fn public_is_graph_cannot_emit_a_pseudo_element_member() {
    exact(":is(.One)", ":is(.One)");
    assert_rejects_pseudo_element_list();
}

#[test]
fn public_where_graph_cannot_emit_a_pseudo_element_member() {
    exact(":where(.One)", ":where(.One)");
    assert_rejects_pseudo_element_list();
}

#[test]
fn public_nth_child_graph_cannot_emit_a_pseudo_element_of_member() {
    exact(":nth-child(2n+1 of .One)", ":nth-child(2n+1 of .One)");
    assert_rejects_pseudo_element_list();
}

#[test]
fn public_nth_last_child_graph_cannot_emit_a_pseudo_element_of_member() {
    exact(":nth-last-child(2 of .One)", ":nth-last-child(2 of .One)");
    assert_rejects_pseudo_element_list();
}

fn assert_rejects_pseudo_element_list() {
    let error = CssPseudoSelectorList::try_new(vec![parsed("::before")]).unwrap_err();
    let CssSelectorConstructionErrorKind::Specified(cause) = error.kind() else {
        panic!("intrinsic pseudo-list cause: {error:?}")
    };
    assert_eq!(
        cause.kind(),
        CssSpecifiedValueSerializationErrorKind::UnrepresentableValue
    );
}

#[test]
fn public_has_graph_cannot_emit_a_pseudo_element_relative_member() {
    assert_eq!(
        has(CssSelector::Class("One".into()))
            .to_specified_css()
            .unwrap(),
        ":has(> .One)"
    );
    invalid_graph(has(parsed("::before")));
}

#[test]
fn public_has_graph_cannot_emit_has_nested_through_logical_arguments() {
    let valid = has(CssSelector::PseudoClass(CssPseudoClass::Not(pseudo_list(
        CssSelector::Class("One".into()),
    ))));
    assert_eq!(valid.to_specified_css().unwrap(), ":has(> :not(.One))");
    invalid_graph(has(CssSelector::PseudoClass(CssPseudoClass::Is(
        pseudo_list(has(CssSelector::Class("One".into()))),
    ))));
}

fn parsed_empty_list() -> CssPseudoSelectorList {
    let report = parse_selector(":is(???)", &CssNamespaceContext::default());
    let Some(CssSelector::PseudoClass(CssPseudoClass::Is(list))) = report.syntax() else {
        panic!("empty is")
    };
    assert!(list.selectors().is_empty());
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropSelectorListItem
    );
    list.clone()
}

#[test]
fn empty_forgiving_list_cannot_be_transplanted_to_public_not_graph() {
    let empty = parsed_empty_list();
    assert!(empty.selectors().is_empty());
    invalid_graph(CssSelector::PseudoClass(CssPseudoClass::Not(empty)));
}

#[test]
fn empty_forgiving_list_cannot_be_transplanted_to_public_nth_of_graph() {
    let empty = parsed_empty_list();
    assert!(empty.selectors().is_empty());
    invalid_graph(CssSelector::PseudoClass(CssPseudoClass::NthChild(
        CssNthChildPattern::new(CssNthPattern::Odd, Some(empty)),
    )));
}

#[test]
fn malformed_strict_list_discards_complete_rule_contents_with_exact_recovery_unit() {
    let failed = ".One, ??? { color:red; .never { color:blue } }";
    let source = format!("/*😀*/\r\n.before{{}} {failed} .after{{}}");
    let report = parse_sheet(&source);
    let [CssRule::Style(before), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("only valid surrounding rules retained: {report:?}");
    };
    assert_eq!(
        before.selectors().selectors()[0]
            .selector()
            .to_specified_css()
            .unwrap(),
        ".before"
    );
    assert_eq!(
        after.selectors().selectors()[0]
            .selector()
            .to_specified_css()
            .unwrap(),
        ".after"
    );
    assert!(before.rules().is_empty() && after.rules().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("one complete rule rejection")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    let start = source.find(failed).unwrap();
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + failed.len()
    );
    assert_eq!(
        after.position().byte_offset().value(),
        source.find(".after").unwrap()
    );
    assert!(report.into_validation_result().is_err());
}

#[test]
fn lexical_bad_token_keeps_original_unicode_byte_and_utf16_coordinates() {
    let source = "/*😀*/\r\n/*é*/:is(.One,])";
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.syntax().is_none());
    let [diagnostic] = report.diagnostics() else {
        panic!("one lexical rejection: {report:?}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    let position = diagnostic.error().position();
    let expected = source.find(']').unwrap();
    assert_eq!(position.byte_offset().value(), expected);
    assert_eq!(position.line().value(), 1);
    let line = source.find('\n').unwrap() + 1;
    assert_eq!(
        position.column().value(),
        u32::try_from(source[line..expected].encode_utf16().count()).unwrap()
    );
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("selector error")
    };
    let encountered = detail.encountered().unwrap();
    assert_eq!(encountered.kind(), CssTokenKind::CloseSquareBracket);
    assert_eq!(encountered.authored(), "]");
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
}

#[test]
fn compound_children_share_cumulative_node_and_utf8_byte_limits_atomically() {
    // Independently counted semantic visits: compound, attribute, is, its two
    // members, nth-child, An+B, and its of member = eight input/projection nodes.
    let expected = r#"[Data="V" s]:is(.One, #Two):nth-child(2n+1 of .Three)"#;
    let selector = parsed(expected);
    let before = selector.clone();
    assert_eq!(
        selector
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                8,
                8,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(7, 8, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(8, 7, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(8, 8, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            selector
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(selector, before);
    }
    assert_eq!(selector.to_specified_css().unwrap(), expected);
    let selector = CssSelector::Class("日本".into());
    assert_eq!(
        selector
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 7))
            .unwrap(),
        ".日本"
    );
    assert_eq!(
        selector
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 6))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}

#[test]
fn relative_children_share_one_cumulative_budget() {
    // Has plus two relative carriers and two class selectors = five nodes.
    let expected = ":has(> .One, + .Two)";
    let selector = parsed(expected);
    assert_eq!(
        selector
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                5,
                5,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    assert_eq!(
        selector
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                4,
                5,
                expected.len()
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(selector.to_specified_css().unwrap(), expected);
}

#[test]
fn deep_logical_parse_clone_emit_and_limit_recovery_work_on_an_ordinary_stack() {
    std::thread::Builder::new()
        .name("selector-ordinary-stack".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let context = CssNamespaceContext::default();
            for depth in [128usize, 129, 255, 256] {
                let source = format!("{}.One{}", ":is(".repeat(depth), ")".repeat(depth));
                let selector = parsed_in(&source, &context);
                let mut current = &selector;
                for _ in 0..depth {
                    let [child] = logical_members(current) else {
                        panic!("one logical child")
                    };
                    current = child;
                }
                assert_eq!(current, &CssSelector::Class("One".into()));
                let clone = selector.clone();
                assert_eq!(clone.to_specified_css().unwrap(), source);
                let report = parse_selector_list(&source, &context);
                assert!(report.is_clean());
                assert_eq!(report.syntax().as_ref().unwrap().selectors().len(), 1);
            }
            let source = format!("{}.One{}", ":is(".repeat(257), ")".repeat(257));
            let report = parse_selector(&source, &context);
            assert!(report.syntax().is_none());
            let [limit] = report.diagnostics() else {
                panic!("one first over-limit failure")
            };
            assert_eq!(limit.action(), CssRecoveryAction::StopAtNestingLimit);
            assert_eq!(limit.error().code(), CssErrorCode::NestingLimit);
            assert_eq!(limit.error().position().byte_offset().value(), 1025);
            assert_eq!(limit.span().start().byte_offset().value(), 0);
            assert_eq!(limit.span().end().byte_offset().value(), source.len());
            let ErrorKind::NestingLimit(detail) = limit.error().kind() else {
                panic!("typed nesting limit")
            };
            assert_eq!(detail.limit(), 256);
            assert_eq!(
                detail.enclosing_production().as_str(),
                "baseline.selector.complex"
            );
            assert!(report.into_validation_result().is_err());
            exact(".after", ".after");
        })
        .unwrap()
        .join()
        .unwrap();
}
