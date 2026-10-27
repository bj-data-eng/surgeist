//! Display3 Order intrinsic semantics; ordering of layout items is downstream.
use crate::css::support::property_admission::assert_invalid_value;
use crate::css::support::sole_declaration as declaration;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues};
use surgeist_css::declaration::{CssGlobalKeyword, CssImportance};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssLonghandValueRef, CssPropertyKindRef,
};

#[test]
fn order_initial_is_literal_zero_and_ordinary_values_keep_identity() {
    let metadata = CssKnownProperty::Order.metadata().unwrap();
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("longhand")
    };
    assert!(!longhand.inherited_by_default());
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("ordinary initial")
    };
    let CssLonghandValueRef::Order(value) = initial.view() else {
        panic!("order")
    };
    assert_eq!(value.serialize_specified(Default::default()).unwrap(), "0");
    for text in ["0", "-2", "2147483648", "-2147483649", "calc(1.5)"] {
        let source = declaration(&format!("order:{text}!important"));
        let CssKnownPropertyValueRef::Order(authored) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("order")
        };
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            source.expand().unwrap()
        else {
            panic!("ordinary")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        let CssLonghandValueRef::Order(value) = item.ordinary_value().unwrap().view() else {
            panic!("order")
        };
        assert_eq!(value, authored.value());
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
    }
}

#[test]
fn order_reentry_is_atomic_and_preserves_exact_replacement_origin() {
    let source = declaration("order:var(--priority)!important");
    let CssExpansion::Pending(handle) = source.expand().unwrap() else {
        panic!("pending")
    };
    for input in [
        "1.0",
        "1e2",
        "1px",
        "calc(1px)",
        "inherit extra",
        "1; color:red",
        "1!important",
    ] {
        let error = handle
            .reenter(CssComponentValues::parse(&CssParser::default(), input).unwrap())
            .unwrap_err();
        assert!(error.replacement_error().is_some(), "{input}: {error:?}");
    }
    assert!(
        handle
            .reenter(CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap())
            .unwrap_err()
            .is_residual_substitution()
    );
    for replacement in [
        CssComponentValues::parse(&CssParser::default(), "+0002147483648").unwrap(),
        CssComponentValues::try_new(
            vec![CssComponentValue::try_number("+0002147483648").unwrap()],
            Default::default(),
        )
        .unwrap(),
    ] {
        for _ in 0..2 {
            let CssContributions::Longhands(items) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("replacement")
            };
            let [item] = items.items() else {
                panic!("one terminal")
            };
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            let Some(literal) = (match item.ordinary_value().unwrap().view() {
                CssLonghandValueRef::Order(value) => value.literal(),
                _ => None,
            }) else {
                panic!("exact order")
            };
            assert_eq!(literal.component(), &replacement.items()[0]);
            assert_eq!(literal.origin(), replacement.items()[0].origin());
        }
    }
    let CssContributions::Longhands(items) = handle
        .reenter(CssComponentValues::parse(&CssParser::default(), "inherit").unwrap())
        .unwrap()
    else {
        panic!("global")
    };
    assert_eq!(
        items.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );
}

#[test]
fn normalization_preserves_authored_order_around_nested_rules_instead_of_sorting_values() {
    let text = ".a{order:2147483648;order:1.0;.child{order:-2147483649}order:0!important}";
    let report = CssParser::default().parse_sheet(text);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert_invalid_value(
        report.diagnostics()[0].error(),
        CssKnownProperty::Order,
        Some("1.0"),
    );
    assert_eq!(
        CssParser::default()
            .parse_sheet(text)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
    let sheet = report.syntax().normalize(Default::default()).unwrap();
    let items: Vec<_> = sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(item) => Some(item),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 3);
    for (index, (item, expected)) in items
        .iter()
        .zip(["2147483648", "-2147483649", "0"])
        .enumerate()
    {
        assert_eq!(item.order(), index);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("ordinary")
        };
        let [value] = values.items() else {
            panic!("one terminal")
        };
        let CssLonghandValueRef::Order(integer) = value.ordinary_value().unwrap().view() else {
            panic!("order")
        };
        assert_eq!(
            integer.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert!(value.source().same_occurrence(item.source()));
        assert_eq!(
            item.source().position().unwrap().byte_offset().value(),
            text.find(&format!("order:{expected}")).unwrap()
        );
    }
    assert!(
        items[1]
            .selector_context()
            .parent()
            .unwrap()
            .same_context(items[0].selector_context())
    );
    assert!(
        items[2]
            .selector_context()
            .same_context(items[0].selector_context())
    );
    assert_eq!(items[2].source().importance(), CssImportance::Important);
    let error = report
        .syntax()
        .normalize(CssNormalizationLimits::try_new(256, usize::MAX, 3, 2).unwrap())
        .unwrap_err();
    assert_eq!(
        error.exhausted_limit(),
        Some((CssNormalizationResource::Contributions, 2))
    );
}
