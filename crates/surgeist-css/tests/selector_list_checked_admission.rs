#![forbid(unsafe_code)]
//! Intrinsic admission through the existing ordinary, complex-real and general
//! relative list factories. Literal grammar expectations use selected Selectors 4
//! and the adopted Shadow compound-context restriction.

use surgeist_css::{
    CssCompoundSelectorArgument, CssErrorCode, CssNamespaceContext, CssNthChildPattern,
    CssNthPattern, CssPseudoClass, CssPseudoSelectorList, CssRecoveryAction, CssRelativeSelector,
    CssRelativeSelectorList, CssSelector, CssSelectorCombinator, CssSelectorList,
    CssSpecifiedValueSerializationErrorKind, ErrorKind, parse_selector, parse_selector_list,
};

fn parsed(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source:?}: {report:?}");
    report.into_validation_result().unwrap().unwrap()
}

fn class() -> CssSelector {
    CssSelector::Class("One".into())
}

fn relative(selector: CssSelector) -> CssRelativeSelector {
    CssRelativeSelector::new(CssSelectorCombinator::Child, selector)
}

fn recovered_empty_pseudo(source: &str) -> CssSelector {
    // Current parsing retains these empty carriers after dropping its attempted
    // zero-width member. They are useful construction fixtures but not clean
    // parse reports. Keep the ordinary parsed helper strict.
    let (expected_offset, expected_column) = match source {
        ":is()" => (4, 4),
        ":where()" => (7, 7),
        _ => panic!("only explicit empty Is/Where fixtures are admitted"),
    };
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(!report.is_clean(), "expected recovered fixture: {report:?}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one zero-width empty-member diagnostic: {report:?}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropSelectorListItem);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert!(matches!(
        diagnostic.error().kind(),
        ErrorKind::InvalidSelector(_)
    ));
    for position in [
        diagnostic.error().position(),
        diagnostic.span().start(),
        diagnostic.span().end(),
    ] {
        assert_eq!(position.byte_offset().value(), expected_offset);
        assert_eq!(position.line().value(), 0);
        assert_eq!(position.column().value(), expected_column);
    }
    let selector = report
        .syntax()
        .as_ref()
        .expect("retained empty pseudo")
        .clone();
    let list = match (&selector, source) {
        (CssSelector::PseudoClass(CssPseudoClass::Is(list)), ":is()")
        | (CssSelector::PseudoClass(CssPseudoClass::Where(list)), ":where()") => list,
        _ => panic!("fixture preserves the requested empty function: {selector:?}"),
    };
    assert!(list.selectors().is_empty());
    let diagnostics = report.diagnostics().to_vec();
    assert_eq!(
        report.into_validation_result().unwrap_err().diagnostics(),
        diagnostics
    );
    selector
}

fn empty_forgiving_list() -> CssPseudoSelectorList {
    let CssSelector::PseudoClass(CssPseudoClass::Is(list)) = recovered_empty_pseudo(":is()") else {
        panic!("recovered empty Is list")
    };
    list
}

fn empty_not() -> CssSelector {
    CssSelector::PseudoClass(CssPseudoClass::Not(empty_forgiving_list()))
}

fn empty_nth_of() -> CssSelector {
    CssSelector::PseudoClass(CssPseudoClass::NthChild(CssNthChildPattern::new(
        CssNthPattern::Odd,
        Some(empty_forgiving_list()),
    )))
}

fn has(selector: CssSelector) -> CssSelector {
    CssSelector::PseudoClass(CssPseudoClass::Has(
        CssRelativeSelectorList::try_new(vec![relative(selector)])
            .expect("intrinsically valid general relative child"),
    ))
}

fn nested_has_through_is() -> CssSelector {
    // The inner Has is valid in an ordinary complex-real Is member, and that Is
    // is valid as a general relative child. Only raw outer Has attachment makes
    // the graph invalid, so fixture creation needs no invalid checked container.
    let logical = CssSelector::PseudoClass(CssPseudoClass::Is(
        CssPseudoSelectorList::try_new(vec![has(class())])
            .expect("valid Has in a complex-real member"),
    ));
    has(logical)
}

fn assert_unrepresentable(selector: &CssSelector) {
    assert_eq!(
        selector.to_specified_css().unwrap_err().kind(),
        CssSpecifiedValueSerializationErrorKind::UnrepresentableValue
    );
}

#[test]
fn strict_nonempty_factories_reject_empty_vectors() {
    assert!(CssSelectorList::try_new(Vec::new()).is_none());
    assert!(CssPseudoSelectorList::try_new(Vec::new()).is_none());
    assert!(CssRelativeSelectorList::try_new(Vec::new()).is_none());
}

#[test]
fn ordinary_factory_preserves_order_duplicates_and_decoded_identifiers() {
    let members = vec![
        CssSelector::Class("a b".into()),
        CssSelector::Class("a.b".into()),
        CssSelector::Key("1Lead".into()),
        CssSelector::Class("a b".into()),
    ];
    let list = CssSelectorList::try_new(members.clone()).unwrap();
    assert_eq!(list.selectors(), members);
    for (member, literal) in
        list.selectors()
            .iter()
            .zip([".a\\ b", ".a\\.b", "#\\31 Lead", ".a\\ b"])
    {
        assert_eq!(member.to_specified_css().unwrap(), literal);
        assert_eq!(parsed(literal), *member);
    }
}

#[test]
fn ordinary_factory_admits_legal_terminal_pseudo_elements() {
    let members = vec![parsed(".Cell::before"), parsed(".Parent > .Cell::after")];
    let list = CssSelectorList::try_new(members.clone()).unwrap();
    assert_eq!(list.selectors(), members);
    assert_eq!(
        list.selectors()[0].to_specified_css().unwrap(),
        ".Cell::before"
    );
    assert_eq!(
        list.selectors()[1].to_specified_css().unwrap(),
        ".Parent > .Cell::after"
    );
}

#[test]
fn pseudo_factory_admits_complex_real_and_has_members() {
    let members = vec![
        parsed(".One > .Two"),
        has(class()),
        recovered_empty_pseudo(":where()"),
    ];
    let list = CssPseudoSelectorList::try_new(members.clone()).unwrap();
    assert_eq!(list.selectors(), members);
    let selector = CssSelector::PseudoClass(CssPseudoClass::Is(list));
    let before = selector.clone();
    let literal = ":is(.One > .Two, :has(> .One), :where())";
    assert_eq!(selector.to_specified_css().unwrap(), literal);
    assert_eq!(selector, before);
}

#[test]
fn general_relative_factory_keeps_all_relations_and_member_order() {
    let members: Vec<_> = [
        CssSelectorCombinator::Descendant,
        CssSelectorCombinator::Child,
        CssSelectorCombinator::NextSibling,
        CssSelectorCombinator::SubsequentSibling,
        CssSelectorCombinator::Column,
    ]
    .into_iter()
    .map(|combinator| CssRelativeSelector::new(combinator, class()))
    .collect();
    let list = CssRelativeSelectorList::try_new(members.clone()).unwrap();
    assert_eq!(list.selectors(), members);
    let selector = CssSelector::PseudoClass(CssPseudoClass::Has(list));
    let literal = ":has(.One, > .One, + .One, ~ .One, || .One)";
    assert_eq!(selector.to_specified_css().unwrap(), literal);
    assert_eq!(parsed(literal), selector);
}

#[test]
fn general_relative_factory_admits_legal_terminal_pseudo_elements() {
    let members = vec![relative(parsed(".Cell::before"))];
    let list = CssRelativeSelectorList::try_new(members.clone()).unwrap();
    assert_eq!(list.selectors(), members);
    assert_eq!(
        list.selectors()[0].selector().to_specified_css().unwrap(),
        ".Cell::before"
    );
    // General relative admission does not authorize Has's narrower attachment.
    assert_unrepresentable(&CssSelector::PseudoClass(CssPseudoClass::Has(list)));
}

#[test]
fn general_relative_has_member_is_valid_until_nested_has_attachment() {
    let member = relative(has(class()));
    let list = CssRelativeSelectorList::try_new(vec![member.clone()]).unwrap();
    assert_eq!(list.selectors(), [member]);
    assert_eq!(
        list.selectors()[0].selector().to_specified_css().unwrap(),
        ":has(> .One)"
    );
    assert_unrepresentable(&CssSelector::PseudoClass(CssPseudoClass::Has(list)));
}

#[test]
fn forgiving_empty_is_where_graphs_are_valid_members_of_all_three_lists() {
    for literal in [":is()", ":where()"] {
        let selector = recovered_empty_pseudo(literal);
        assert_eq!(selector.to_specified_css().unwrap(), literal);
        let ordinary = CssSelectorList::try_new(vec![selector.clone()]).unwrap();
        assert_eq!(ordinary.selectors(), std::slice::from_ref(&selector));
        let pseudo = CssPseudoSelectorList::try_new(vec![selector.clone()]).unwrap();
        assert_eq!(pseudo.selectors(), std::slice::from_ref(&selector));
        let member = relative(selector);
        let relative = CssRelativeSelectorList::try_new(vec![member.clone()]).unwrap();
        assert_eq!(relative.selectors(), [member]);
    }
}

#[test]
fn selected_shadow_compound_has_remains_valid_under_list_admission() {
    let argument = CssCompoundSelectorArgument::try_new(has(class())).unwrap();
    let host = CssSelector::PseudoClass(CssPseudoClass::HostFunction(argument));
    assert_eq!(host.to_specified_css().unwrap(), ":host(:has(> .One))");
    let ordinary = CssSelectorList::try_new(vec![host.clone()]).unwrap();
    assert_eq!(ordinary.selectors(), std::slice::from_ref(&host));
    let pseudo = CssPseudoSelectorList::try_new(vec![host.clone()]).unwrap();
    assert_eq!(pseudo.selectors(), std::slice::from_ref(&host));
    let member = relative(host);
    let relative = CssRelativeSelectorList::try_new(vec![member.clone()]).unwrap();
    assert_eq!(relative.selectors(), [member]);
}

#[test]
fn ordinary_parser_rejects_relative_syntax_while_relative_factory_keeps_carrier() {
    let report = parse_selector_list("> .One", &CssNamespaceContext::default());
    assert!(report.syntax().is_none());
    assert!(report.into_validation_result().is_err());
    let member = relative(class());
    let list = CssRelativeSelectorList::try_new(vec![member.clone()]).unwrap();
    assert_eq!(list.selectors(), [member]);
    assert_eq!(
        list.selectors()[0].combinator(),
        CssSelectorCombinator::Child
    );
    let ordinary = CssSelectorList::try_new(vec![class()]).unwrap();
    assert_eq!(ordinary.selectors(), [class()]);
}

#[test]
fn ordinary_factory_rejects_empty_class_member_atomically() {
    assert!(CssSelectorList::try_new(vec![class()]).is_some());
    let actual =
        CssSelectorList::try_new(vec![class(), CssSelector::Class(String::new()), class()]);
    assert!(actual.is_none(), "empty class admitted: {actual:?}");
}

#[test]
fn pseudo_factory_rejects_empty_class_member_atomically() {
    assert!(CssPseudoSelectorList::try_new(vec![class()]).is_some());
    let actual =
        CssPseudoSelectorList::try_new(vec![class(), CssSelector::Class(String::new()), class()]);
    assert!(actual.is_none(), "empty class admitted: {actual:?}");
}

#[test]
fn relative_factory_rejects_empty_class_member_atomically() {
    assert!(CssRelativeSelectorList::try_new(vec![relative(class())]).is_some());
    let actual = CssRelativeSelectorList::try_new(vec![
        relative(class()),
        relative(CssSelector::Class(String::new())),
        relative(class()),
    ]);
    assert!(
        actual.is_none(),
        "empty relative class admitted: {actual:?}"
    );
}

#[test]
fn ordinary_factory_rejects_nul_identifier_member() {
    assert!(CssSelectorList::try_new(vec![class()]).is_some());
    let invalid = CssSelector::Key("bad\0name".into());
    assert_unrepresentable(&invalid);
    let actual = CssSelectorList::try_new(vec![invalid]);
    assert!(actual.is_none(), "NUL identifier admitted: {actual:?}");
}

#[test]
fn pseudo_factory_rejects_nul_identifier_member() {
    assert!(CssPseudoSelectorList::try_new(vec![class()]).is_some());
    let invalid = CssSelector::Class("bad\0name".into());
    assert_unrepresentable(&invalid);
    let actual = CssPseudoSelectorList::try_new(vec![invalid]);
    assert!(actual.is_none(), "NUL identifier admitted: {actual:?}");
}

#[test]
fn relative_factory_rejects_nul_identifier_member() {
    assert!(CssRelativeSelectorList::try_new(vec![relative(class())]).is_some());
    let invalid = CssSelector::Tag("bad\0name".into());
    assert_unrepresentable(&invalid);
    let actual = CssRelativeSelectorList::try_new(vec![relative(invalid)]);
    assert!(
        actual.is_none(),
        "relative NUL identifier admitted: {actual:?}"
    );
}

#[test]
fn ordinary_factory_rejects_empty_strict_not_graph() {
    assert!(CssSelectorList::try_new(vec![parsed(":not(.One)")]).is_some());
    let invalid = empty_not();
    assert_unrepresentable(&invalid);
    let actual = CssSelectorList::try_new(vec![invalid]);
    assert!(actual.is_none(), "empty strict Not admitted: {actual:?}");
}

#[test]
fn pseudo_factory_rejects_empty_strict_not_graph() {
    assert!(CssPseudoSelectorList::try_new(vec![parsed(":not(.One)")]).is_some());
    let invalid = empty_not();
    assert_unrepresentable(&invalid);
    let actual = CssPseudoSelectorList::try_new(vec![invalid]);
    assert!(actual.is_none(), "empty strict Not admitted: {actual:?}");
}

#[test]
fn relative_factory_rejects_empty_strict_not_graph() {
    assert!(CssRelativeSelectorList::try_new(vec![relative(parsed(":not(.One)"))]).is_some());
    let invalid = empty_not();
    assert_unrepresentable(&invalid);
    let actual = CssRelativeSelectorList::try_new(vec![relative(invalid)]);
    assert!(
        actual.is_none(),
        "relative empty strict Not admitted: {actual:?}"
    );
}

#[test]
fn ordinary_factory_rejects_empty_nth_of_graph() {
    assert!(CssSelectorList::try_new(vec![parsed(":nth-child(odd of .One)")]).is_some());
    let invalid = empty_nth_of();
    assert_unrepresentable(&invalid);
    let actual = CssSelectorList::try_new(vec![invalid]);
    assert!(actual.is_none(), "empty nth of admitted: {actual:?}");
}

#[test]
fn pseudo_factory_rejects_empty_nth_of_graph() {
    assert!(CssPseudoSelectorList::try_new(vec![parsed(":nth-child(odd of .One)")]).is_some());
    let invalid = empty_nth_of();
    assert_unrepresentable(&invalid);
    let actual = CssPseudoSelectorList::try_new(vec![invalid]);
    assert!(actual.is_none(), "empty nth of admitted: {actual:?}");
}

#[test]
fn relative_factory_rejects_empty_nth_of_graph() {
    assert!(
        CssRelativeSelectorList::try_new(vec![relative(parsed(":nth-child(odd of .One)"))])
            .is_some()
    );
    let invalid = empty_nth_of();
    assert_unrepresentable(&invalid);
    let actual = CssRelativeSelectorList::try_new(vec![relative(invalid)]);
    assert!(
        actual.is_none(),
        "relative empty nth of admitted: {actual:?}"
    );
}

#[test]
fn ordinary_factory_rejects_has_nested_through_logical_graph() {
    assert!(CssSelectorList::try_new(vec![has(class())]).is_some());
    let invalid = nested_has_through_is();
    assert_unrepresentable(&invalid);
    let actual = CssSelectorList::try_new(vec![invalid]);
    assert!(actual.is_none(), "nested Has admitted: {actual:?}");
}

#[test]
fn pseudo_factory_rejects_has_nested_through_logical_graph() {
    assert!(CssPseudoSelectorList::try_new(vec![has(class())]).is_some());
    let invalid = nested_has_through_is();
    assert_unrepresentable(&invalid);
    let actual = CssPseudoSelectorList::try_new(vec![invalid]);
    assert!(actual.is_none(), "nested Has admitted: {actual:?}");
}

#[test]
fn relative_factory_rejects_has_nested_through_logical_graph() {
    assert!(CssRelativeSelectorList::try_new(vec![relative(has(class()))]).is_some());
    let invalid = nested_has_through_is();
    assert_unrepresentable(&invalid);
    let actual = CssRelativeSelectorList::try_new(vec![relative(invalid)]);
    assert!(actual.is_none(), "relative nested Has admitted: {actual:?}");
}

#[test]
fn pseudo_factory_rejects_terminal_pseudo_element_member_atomically() {
    assert!(CssPseudoSelectorList::try_new(vec![class()]).is_some());
    let actual = CssPseudoSelectorList::try_new(vec![class(), parsed(".Cell::before"), class()]);
    assert!(
        actual.is_none(),
        "pseudo-element member admitted: {actual:?}"
    );
}

#[test]
fn pseudo_factory_rejects_terminal_pseudo_element_in_complex_member() {
    assert!(CssPseudoSelectorList::try_new(vec![parsed(".Parent > .Cell")]).is_some());
    let actual = CssPseudoSelectorList::try_new(vec![parsed(".Parent > .Cell::before")]);
    assert!(
        actual.is_none(),
        "complex pseudo-element member admitted: {actual:?}"
    );
}
