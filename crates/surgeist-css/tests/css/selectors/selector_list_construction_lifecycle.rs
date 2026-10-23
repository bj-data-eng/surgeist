//! Functional expectations for checked list errors, aggregate admission limits,
//! forgiving-empty construction, and ordinary/general-relative specified output.

use crate::css::support::color_and_selector_fixtures::assert_specified_construction_cause as specified_cause;
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::validated_selector as parsed;
use surgeist_css::{
    CssCompoundSelectorArgument, CssNthChildPattern, CssNthPattern, CssPseudoClass,
    CssPseudoSelectorList, CssPseudoSelectorListItem, CssRelativeSelector, CssRelativeSelectorList,
    CssSelector, CssSelectorCombinator, CssSelectorList, CssSerializationLimits,
};

type Limits = CssSerializationLimits;
type Kind = SerializationFailure;

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

#[test]
fn strict_empty_lists_report_their_typed_cause_before_resource_admission() {
    for error in [
        CssSelectorList::try_new(Vec::new(), Limits::new(0, 0, 0)).unwrap_err(),
        CssPseudoSelectorList::try_new(Vec::new(), Limits::new(0, 0, 0)).unwrap_err(),
        CssRelativeSelectorList::try_new(Vec::new(), Limits::new(0, 0, 0)).unwrap_err(),
    ] {
        assert!(error.is_empty_list(), "{error:?}");
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn ordinary_admission_and_output_share_three_nodes_and_exact_utf8_bytes() {
    // One list aggregate, one class and one ID. Japanese/é lengths are UTF-8
    // bytes; no function wrapper or per-member fresh budget is charged here.
    let literal = ".日本, #é";
    let members = ordinary_members();
    let list = CssSelectorList::try_new(members.clone(), Limits::new(3, 3, literal.len())).unwrap();
    assert_eq!(list.selectors(), members.as_slice());
    let before = list.clone();
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        literal
    );
    assert_eq!(
        list.serialize_specified(Limits::new(3, 3, literal.len()))
            .unwrap(),
        literal
    );
    for (limits, cause) in [
        (Limits::new(2, 3, literal.len()), Kind::InputNodeLimit),
        (Limits::new(3, 2, literal.len()), Kind::ProjectionNodeLimit),
        (Limits::new(3, 3, literal.len() - 1), Kind::ByteLimit),
    ] {
        specified_cause(
            &CssSelectorList::try_new(members.clone(), limits).unwrap_err(),
            cause,
        );
        assert_eq!(
            serialization_failure(&list.serialize_specified(limits).unwrap_err()),
            cause
        );
        assert_eq!(list, before);
    }
    assert_eq!(
        CssSelectorList::try_new(members, Default::default()).unwrap(),
        list
    );
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        literal
    );
}

#[test]
fn general_relative_admission_and_output_charge_each_leading_carrier() {
    // Aggregate plus two relative carriers plus two simple selectors = five.
    let literal = "> .日本, || #é";
    let members = relative_members();
    let list = CssRelativeSelectorList::try_new(members.clone(), Limits::new(5, 5, literal.len()))
        .unwrap();
    assert_eq!(list.selectors(), members.as_slice());
    let before = list.clone();
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        literal
    );
    assert_eq!(
        list.serialize_specified(Limits::new(5, 5, literal.len()))
            .unwrap(),
        literal
    );
    for (limits, cause) in [
        (Limits::new(4, 5, literal.len()), Kind::InputNodeLimit),
        (Limits::new(5, 4, literal.len()), Kind::ProjectionNodeLimit),
        (Limits::new(5, 5, literal.len() - 1), Kind::ByteLimit),
    ] {
        specified_cause(
            &CssRelativeSelectorList::try_new(members.clone(), limits).unwrap_err(),
            cause,
        );
        assert_eq!(
            serialization_failure(&list.serialize_specified(limits).unwrap_err()),
            cause
        );
        assert_eq!(list, before);
    }
    assert_eq!(
        CssRelativeSelectorList::try_new(members, Default::default()).unwrap(),
        list
    );
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        literal
    );
}

#[test]
fn pseudo_admission_charges_argument_bytes_without_guessing_an_attachment() {
    let members = ordinary_members();
    let arguments = ".日本, #é";
    let list = CssPseudoSelectorList::try_new(members.clone(), Limits::new(3, 3, arguments.len()))
        .unwrap();
    assert_eq!(
        list.items(),
        members
            .iter()
            .cloned()
            .map(CssPseudoSelectorListItem::Selector)
            .collect::<Vec<_>>()
    );
    for (limits, cause) in [
        (Limits::new(2, 3, arguments.len()), Kind::InputNodeLimit),
        (
            Limits::new(3, 2, arguments.len()),
            Kind::ProjectionNodeLimit,
        ),
        (Limits::new(3, 3, arguments.len() - 1), Kind::ByteLimit),
    ] {
        specified_cause(
            &CssPseudoSelectorList::try_new(members.clone(), limits).unwrap_err(),
            cause,
        );
    }
    assert_eq!(
        CssPseudoSelectorList::try_new(members, Default::default()).unwrap(),
        list
    );
    let selector = CssSelector::PseudoClass(CssPseudoClass::Is(list));
    assert_eq!(
        selector.serialize_specified(Default::default()).unwrap(),
        ":is(.日本, #é)"
    );
}

#[test]
fn checked_forgiving_empty_list_costs_one_aggregate_and_retains_strict_consumers() {
    let empty = CssPseudoSelectorList::try_new_forgiving(Vec::new(), Limits::new(1, 1, 0)).unwrap();
    assert!(empty.items().is_empty());
    assert_eq!(
        CssPseudoSelectorList::try_new_forgiving(Vec::new(), Default::default()).unwrap(),
        empty
    );
    for (limits, cause) in [
        (Limits::new(0, 1, 0), Kind::InputNodeLimit),
        (Limits::new(1, 0, 0), Kind::ProjectionNodeLimit),
    ] {
        specified_cause(
            &CssPseudoSelectorList::try_new_forgiving(Vec::new(), limits).unwrap_err(),
            cause,
        );
    }
    assert_eq!(
        CssSelector::PseudoClass(CssPseudoClass::Is(empty.clone()))
            .serialize_specified(Default::default())
            .unwrap(),
        ":is()"
    );
    assert_eq!(
        CssSelector::PseudoClass(CssPseudoClass::Where(empty.clone()))
            .serialize_specified(Default::default())
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
            &CssSelectorList::try_new(vec![invalid.clone()], Default::default()).unwrap_err(),
            Kind::UnrepresentableValue,
        );
        assert!(
            invalid
                .serialize_specified(Default::default())
                .unwrap_err()
                .is_unrepresentable_value()
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
            &CssPseudoSelectorList::try_new_forgiving(members, Default::default()).unwrap_err(),
            Kind::UnrepresentableValue,
        );
    }
    let members = vec![parsed(".One > .Two"), parsed(":has(> .Cell)")];
    let list =
        CssPseudoSelectorList::try_new_forgiving(members.clone(), Default::default()).unwrap();
    assert_eq!(
        list.items(),
        members
            .into_iter()
            .map(CssPseudoSelectorListItem::Selector)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        CssSelector::PseudoClass(CssPseudoClass::Where(list))
            .serialize_specified(Default::default())
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
    let list = CssRelativeSelectorList::try_new(members.clone(), Default::default()).unwrap();
    assert_eq!(list.selectors(), members.as_slice());
    let before = list.clone();
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        "> .Cell::before, || :has(> .One)"
    );
    assert_eq!(list, before);
    assert!(
        CssSelector::PseudoClass(CssPseudoClass::Has(list))
            .serialize_specified(Default::default())
            .unwrap_err()
            .is_unrepresentable_value()
    );
}

#[test]
fn general_list_admission_does_not_erase_actual_compound_attachment_restrictions() {
    let members = vec![parsed(".One > .Two")];
    let list = CssPseudoSelectorList::try_new_forgiving(members, Default::default()).unwrap();
    let logical = CssSelector::PseudoClass(CssPseudoClass::Is(list));
    assert_eq!(
        logical.serialize_specified(Default::default()).unwrap(),
        ":is(.One > .Two)"
    );
    assert!(CssCompoundSelectorArgument::try_new(logical).is_none());
    let argument = CssCompoundSelectorArgument::try_new(parsed(":has(> .One)")).unwrap();
    let shadow = CssSelector::PseudoClass(CssPseudoClass::HostFunction(argument));
    let list = CssSelectorList::try_new(vec![shadow], Default::default()).unwrap();
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        ":host(:has(> .One))"
    );
}

#[test]
fn finite_nested_and_wide_admission_share_the_iterative_budget() {
    let mut selector = CssSelector::Class("One".into());
    for _ in 0..128 {
        selector = CssSelector::PseudoClass(CssPseudoClass::Is(
            CssPseudoSelectorList::try_new(vec![selector], Default::default()).unwrap(),
        ));
    }
    let literal = format!("{}.One{}", ":is(".repeat(128), ")".repeat(128));
    // The outer ordinary list adds one aggregate to 128 Is nodes and one class.
    let list =
        CssSelectorList::try_new(vec![selector.clone()], Limits::new(130, 130, literal.len()))
            .unwrap();
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        literal
    );
    specified_cause(
        &CssSelectorList::try_new(vec![selector], Limits::new(129, usize::MAX, usize::MAX))
            .unwrap_err(),
        Kind::InputNodeLimit,
    );
    let members = vec![CssSelector::Class("One".into()); 1024];
    let wide = CssSelectorList::try_new(members.clone(), Default::default()).unwrap();
    assert_eq!(wide.selectors(), members.as_slice());
    specified_cause(
        &CssSelectorList::try_new(members, Limits::new(1024, usize::MAX, usize::MAX)).unwrap_err(),
        Kind::InputNodeLimit,
    );
}
