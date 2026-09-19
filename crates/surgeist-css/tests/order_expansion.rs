#![forbid(unsafe_code)]
//! Display3 Order intrinsic semantics; ordering of layout items is downstream.
use surgeist_css::*;

fn declaration(text: &str) -> CssDeclaration {
    let report = parse_style_attribute(text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

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
    assert_eq!(value, &CssIntegerValue::Literal(0));
    for text in ["0", "-2", "2147483648", "-2147483649", "calc(1.5)"] {
        let source = declaration(&format!("order:{text}!important"));
        let CssKnownPropertyValueRef::Order(authored) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("order")
        };
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            expand_declaration(&source).unwrap()
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
fn order_globals_keep_one_terminal_and_importance() {
    for (text, global) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("order:{text}!important"));
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("global")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(item.property(), CssKnownProperty::Order);
        assert_eq!(item.value(), CssContributionValueRef::Global(global));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn order_reentry_is_atomic_and_preserves_exact_replacement_origin() {
    let source = declaration("order:var(--priority)!important");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
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
            .reenter(parse_component_values(input).unwrap())
            .unwrap_err();
        assert!(
            matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(_)),
            "{input}: {error:?}"
        );
    }
    assert_eq!(
        handle
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    for replacement in [
        parse_component_values("+0002147483648").unwrap(),
        CssComponentValues::try_new(vec![
            CssComponentValue::try_number("+0002147483648").unwrap(),
        ])
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
            let CssLonghandValueRef::Order(CssIntegerValue::ExactLiteral(literal)) =
                item.ordinary_value().unwrap().view()
            else {
                panic!("exact order")
            };
            assert_eq!(literal.component(), &replacement.items()[0]);
            assert_eq!(literal.origin(), replacement.items()[0].origin());
        }
    }
    let CssContributions::Longhands(items) = handle
        .reenter(parse_component_values("inherit").unwrap())
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
    let report = parse_sheet(text);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert!(validate_sheet(text).is_err());
    let sheet = normalize_sheet(report.syntax()).unwrap();
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
        assert_eq!(integer.serialize_specified().unwrap(), expected);
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
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(256, usize::MAX, 3, 2).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 2
        }
    );
}
