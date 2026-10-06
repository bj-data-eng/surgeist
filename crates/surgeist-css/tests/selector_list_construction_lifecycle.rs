#![forbid(unsafe_code)]
//! Functional expectations for checked list errors, aggregate admission limits,
//! forgiving-empty construction, and ordinary/general-relative specified output.

use surgeist_css::{
    CssCompoundSelectorArgument, CssNamespaceContext, CssNthChildPattern, CssNthPattern,
    CssPseudoClass, CssPseudoSelectorList, CssRelativeSelector, CssRelativeSelectorList,
    CssSelector, CssSelectorCombinator, CssSelectorConstructionError,
    CssSelectorConstructionErrorKind, CssSelectorList, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, parse_selector,
};

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn parsed(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source:?}: {report:?}");
    report.into_validation_result().unwrap().unwrap()
}

fn ordinary_members() -> Vec<CssSelector> {
    vec![
        CssSelector::Class("日本".into()),
        CssSelector::Key("é".into()),
    ]
}

fn relative_members() -> Vec<CssRelativeSelector> {
    vec![
        CssRelativeSelector::new(
            CssSelectorCombinator::Child,
            CssSelector::Class("日本".into()),
        ),
        CssRelativeSelector::new(CssSelectorCombinator::Column, CssSelector::Key("é".into())),
    ]
}

fn specified_cause(error: &CssSelectorConstructionError, expected: Kind) {
    let CssSelectorConstructionErrorKind::Specified(cause) = error.kind() else {
        panic!("specified construction cause: {error:?}")
    };
    assert_eq!(cause.kind(), expected);
    let source = std::error::Error::source(error).expect("original specified source");
    assert_eq!(
        source.downcast_ref::<CssSpecifiedValueSerializationError>(),
        Some(cause)
    );
}

#[test]
fn strict_empty_lists_report_their_typed_cause_before_resource_admission() {
    for error in [
        CssSelectorList::try_new_with_limits(Vec::new(), Limits::new(0, 0, 0)).unwrap_err(),
        CssPseudoSelectorList::try_new_with_limits(Vec::new(), Limits::new(0, 0, 0)).unwrap_err(),
        CssRelativeSelectorList::try_new_with_limits(Vec::new(), Limits::new(0, 0, 0)).unwrap_err(),
    ] {
        assert_eq!(error.kind(), &CssSelectorConstructionErrorKind::EmptyList);
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn ordinary_admission_and_output_share_three_nodes_and_exact_utf8_bytes() {
    // One list aggregate, one class and one ID. Japanese/é lengths are UTF-8
    // bytes; no function wrapper or per-member fresh budget is charged here.
    let literal = ".日本, #é";
    let members = ordinary_members();
    let list =
        CssSelectorList::try_new_with_limits(members.clone(), Limits::new(3, 3, literal.len()))
            .unwrap();
    assert_eq!(list.selectors(), members.as_slice());
    let before = list.clone();
    assert_eq!(list.to_specified_css().unwrap(), literal);
    assert_eq!(
        list.to_specified_css_with_limits(Limits::new(3, 3, literal.len()))
            .unwrap(),
        literal
    );
    for (limits, cause) in [
        (Limits::new(2, 3, literal.len()), Kind::InputNodeLimit),
        (Limits::new(3, 2, literal.len()), Kind::ProjectionNodeLimit),
        (Limits::new(3, 3, literal.len() - 1), Kind::ByteLimit),
    ] {
        specified_cause(
            &CssSelectorList::try_new_with_limits(members.clone(), limits).unwrap_err(),
            cause,
        );
        assert_eq!(
            list.to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            cause
        );
        assert_eq!(list, before);
    }
    assert_eq!(CssSelectorList::try_new(members).unwrap(), list);
    assert_eq!(list.to_specified_css().unwrap(), literal);
}

#[test]
fn general_relative_admission_and_output_charge_each_leading_carrier() {
    // Aggregate plus two relative carriers plus two simple selectors = five.
    let literal = "> .日本, || #é";
    let members = relative_members();
    let list = CssRelativeSelectorList::try_new_with_limits(
        members.clone(),
        Limits::new(5, 5, literal.len()),
    )
    .unwrap();
    assert_eq!(list.selectors(), members.as_slice());
    let before = list.clone();
    assert_eq!(list.to_specified_css().unwrap(), literal);
    assert_eq!(
        list.to_specified_css_with_limits(Limits::new(5, 5, literal.len()))
            .unwrap(),
        literal
    );
    for (limits, cause) in [
        (Limits::new(4, 5, literal.len()), Kind::InputNodeLimit),
        (Limits::new(5, 4, literal.len()), Kind::ProjectionNodeLimit),
        (Limits::new(5, 5, literal.len() - 1), Kind::ByteLimit),
    ] {
        specified_cause(
            &CssRelativeSelectorList::try_new_with_limits(members.clone(), limits).unwrap_err(),
            cause,
        );
        assert_eq!(
            list.to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            cause
        );
        assert_eq!(list, before);
    }
    assert_eq!(CssRelativeSelectorList::try_new(members).unwrap(), list);
    assert_eq!(list.to_specified_css().unwrap(), literal);
}

#[test]
fn pseudo_admission_charges_argument_bytes_without_guessing_an_attachment() {
    let members = ordinary_members();
    let arguments = ".日本, #é";
    let list = CssPseudoSelectorList::try_new_with_limits(
        members.clone(),
        Limits::new(3, 3, arguments.len()),
    )
    .unwrap();
    assert_eq!(list.selectors(), members.as_slice());
    for (limits, cause) in [
        (Limits::new(2, 3, arguments.len()), Kind::InputNodeLimit),
        (
            Limits::new(3, 2, arguments.len()),
            Kind::ProjectionNodeLimit,
        ),
        (Limits::new(3, 3, arguments.len() - 1), Kind::ByteLimit),
    ] {
        specified_cause(
            &CssPseudoSelectorList::try_new_with_limits(members.clone(), limits).unwrap_err(),
            cause,
        );
    }
    assert_eq!(CssPseudoSelectorList::try_new(members).unwrap(), list);
    let selector = CssSelector::PseudoClass(CssPseudoClass::Is(list));
    assert_eq!(selector.to_specified_css().unwrap(), ":is(.日本, #é)");
}

#[test]
fn checked_forgiving_empty_list_costs_one_aggregate_and_retains_strict_consumers() {
    let empty =
        CssPseudoSelectorList::try_new_forgiving_with_limits(Vec::new(), Limits::new(1, 1, 0))
            .unwrap();
    assert!(empty.selectors().is_empty());
    assert_eq!(
        CssPseudoSelectorList::try_new_forgiving(Vec::new()).unwrap(),
        empty
    );
    for (limits, cause) in [
        (Limits::new(0, 1, 0), Kind::InputNodeLimit),
        (Limits::new(1, 0, 0), Kind::ProjectionNodeLimit),
    ] {
        specified_cause(
            &CssPseudoSelectorList::try_new_forgiving_with_limits(Vec::new(), limits).unwrap_err(),
            cause,
        );
    }
    assert_eq!(
        CssSelector::PseudoClass(CssPseudoClass::Is(empty.clone()))
            .to_specified_css()
            .unwrap(),
        ":is()"
    );
    assert_eq!(
        CssSelector::PseudoClass(CssPseudoClass::Where(empty.clone()))
            .to_specified_css()
            .unwrap(),
        ":where()"
    );
    for invalid in [
        CssSelector::PseudoClass(CssPseudoClass::Not(empty.clone())),
        CssSelector::PseudoClass(CssPseudoClass::NthChild(CssNthChildPattern::new(
            CssNthPattern::Odd,
            Some(empty),
        ))),
    ] {
        specified_cause(
            &CssSelectorList::try_new(vec![invalid.clone()]).unwrap_err(),
            Kind::UnrepresentableValue,
        );
        assert_eq!(
            invalid.to_specified_css().unwrap_err().kind(),
            Kind::UnrepresentableValue
        );
    }
}

#[test]
fn checked_forgiving_factory_rejects_supplied_invalid_members_without_dropping() {
    for invalid in [CssSelector::Class(String::new()), parsed(".Cell::before")] {
        let members = vec![
            CssSelector::Class("One".into()),
            invalid,
            CssSelector::Key("Two".into()),
        ];
        specified_cause(
            &CssPseudoSelectorList::try_new_forgiving(members).unwrap_err(),
            Kind::UnrepresentableValue,
        );
    }
    let members = vec![parsed(".One > .Two"), parsed(":has(> .Cell)")];
    let list = CssPseudoSelectorList::try_new_forgiving(members.clone()).unwrap();
    assert_eq!(list.selectors(), members.as_slice());
    assert_eq!(
        CssSelector::PseudoClass(CssPseudoClass::Where(list))
            .to_specified_css()
            .unwrap(),
        ":where(.One > .Two, :has(> .Cell))"
    );
}

#[test]
fn general_relative_output_preserves_members_that_has_attachment_disallows() {
    let members = vec![
        CssRelativeSelector::new(CssSelectorCombinator::Child, parsed(".Cell::before")),
        CssRelativeSelector::new(CssSelectorCombinator::Column, parsed(":has(> .One)")),
    ];
    let list = CssRelativeSelectorList::try_new(members.clone()).unwrap();
    assert_eq!(list.selectors(), members.as_slice());
    let before = list.clone();
    assert_eq!(
        list.to_specified_css().unwrap(),
        "> .Cell::before, || :has(> .One)"
    );
    assert_eq!(list, before);
    assert_eq!(
        CssSelector::PseudoClass(CssPseudoClass::Has(list))
            .to_specified_css()
            .unwrap_err()
            .kind(),
        Kind::UnrepresentableValue
    );
}

#[test]
fn general_list_admission_does_not_erase_actual_compound_attachment_restrictions() {
    let members = vec![parsed(".One > .Two")];
    let list = CssPseudoSelectorList::try_new_forgiving(members).unwrap();
    let logical = CssSelector::PseudoClass(CssPseudoClass::Is(list));
    assert_eq!(logical.to_specified_css().unwrap(), ":is(.One > .Two)");
    assert!(CssCompoundSelectorArgument::try_new(logical).is_none());
    let argument = CssCompoundSelectorArgument::try_new(parsed(":has(> .One)")).unwrap();
    let shadow = CssSelector::PseudoClass(CssPseudoClass::HostFunction(argument));
    let list = CssSelectorList::try_new(vec![shadow]).unwrap();
    assert_eq!(list.to_specified_css().unwrap(), ":host(:has(> .One))");
}

#[test]
fn finite_nested_and_wide_admission_share_the_iterative_budget() {
    let mut selector = CssSelector::Class("One".into());
    for _ in 0..128 {
        selector = CssSelector::PseudoClass(CssPseudoClass::Is(
            CssPseudoSelectorList::try_new(vec![selector]).unwrap(),
        ));
    }
    let literal = format!("{}.One{}", ":is(".repeat(128), ")".repeat(128));
    // The outer ordinary list adds one aggregate to 128 Is nodes and one class.
    let list = CssSelectorList::try_new_with_limits(
        vec![selector.clone()],
        Limits::new(130, 130, literal.len()),
    )
    .unwrap();
    assert_eq!(list.to_specified_css().unwrap(), literal);
    specified_cause(
        &CssSelectorList::try_new_with_limits(
            vec![selector],
            Limits::new(129, usize::MAX, usize::MAX),
        )
        .unwrap_err(),
        Kind::InputNodeLimit,
    );
    let members = vec![CssSelector::Class("One".into()); 1024];
    let wide = CssSelectorList::try_new(members.clone()).unwrap();
    assert_eq!(wide.selectors(), members.as_slice());
    specified_cause(
        &CssSelectorList::try_new_with_limits(members, Limits::new(1024, usize::MAX, usize::MAX))
            .unwrap_err(),
        Kind::InputNodeLimit,
    );
}
