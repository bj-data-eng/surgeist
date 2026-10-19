#![forbid(unsafe_code)]
//! CSS2 REC 2011-06-07 §9.9.1: auto | <integer>, initial auto, noninherited.
//! references/css2--visuren.html--3f334c530cf4.md#propdef-z-index
//! Positioned Layout WD 2025-10-07 §2.2 adds contextual stacking semantics,
//! which are deliberately downstream. Generic authored lifecycle expectations
//! follow #272 and existing public front doors. Values4 math remains unresolved.
//! New z-index longhand payload and canonical Auto writer APIs require functional
//! implementation alongside independent tests; this file does not call absent APIs.
use surgeist_css::*;

const VALID: &[&str] = &[
    "auto",
    "AUTO",
    r"a\75 to",
    "0",
    "-2",
    "+0002",
    "2147483648",
    "-2147483649",
    "calc(1.5)",
    "calc(2px / 1px)",
];
const INVALID: &[&str] = &[
    "",
    "none",
    "1.0",
    "1e2",
    "1px",
    "1%",
    "1 2",
    "auto 1",
    "calc(1px)",
    "calc(1%)",
    "inherit auto",
];

fn declaration(value: &str) -> CssDeclaration {
    let text = format!("Z-INDEX:{value}!important");
    let report = parse_style_attribute(&text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&text).is_ok());
    assert_eq!(report.syntax().len(), 1);
    report.syntax()[0].clone()
}

fn value(source: &CssDeclaration) -> &CssZIndexPropertyValue {
    let CssKnownPropertyValueRef::ZIndex(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("z-index wrapper")
    };
    value
}

fn contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("z-index intrinsic expansion")
    else {
        panic!("completed longhand")
    };
    let [item] = values.items() else {
        panic!("one longhand")
    };
    assert_eq!(item.property(), CssKnownProperty::ZIndex);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    item.clone()
}

#[test]
fn parsed_and_checked_z_index_keep_the_selected_grammar_and_exact_origins() {
    for &text in VALID {
        let parsed = declaration(text);
        assert_eq!(parsed.known().unwrap().property(), CssKnownProperty::ZIndex);
        assert_eq!(value(&parsed).as_css(), text);
        assert_eq!(parsed.importance(), CssImportance::Important);
        let components = parsed.value_components().clone();
        for item in components.items() {
            let CssValueOrigin::Parsed(origin) = item.origin() else {
                panic!("parsed origin")
            };
            assert!(
                origin
                    .source()
                    .same_snapshot(parsed.parsed_value().unwrap().source())
            );
        }
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ZIndex),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(checked.value_components(), &components);
        assert_eq!(value(&checked).value(), value(&parsed).value());
        assert_eq!(value(&checked).as_css(), text);
        assert!(checked.parsed_value().is_none());
        for (before, after) in components
            .items()
            .iter()
            .zip(checked.value_components().items())
        {
            assert_eq!(before.origin(), after.origin());
        }
    }
    for token in [
        CssComponentValue::try_ident("AUTO").unwrap(),
        CssComponentValue::try_number("+0002").unwrap(),
    ] {
        let components = CssComponentValues::try_new(vec![token]).unwrap();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ZIndex),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(checked.value_components(), &components);
        assert_eq!(
            checked.value_components().items()[0].origin(),
            &CssValueOrigin::Programmatic
        );
    }
    for text in ["auto", "AUTO", r"a\75 to"] {
        assert_eq!(value(&declaration(text)).value(), &CssZIndexValue::Auto);
    }
    for (text, expected) in [("+0002", "2"), ("-0000", "0"), ("2147483648", "2147483648")] {
        let source = declaration(text);
        let CssZIndexValue::Integer(integer) = value(&source).value() else {
            panic!("integer")
        };
        assert_eq!(integer.serialize_specified().unwrap(), expected);
        let CssIntegerValue::Literal(literal) = integer else {
            panic!("literal")
        };
        assert_eq!(literal.component(), &source.value_components().items()[0]);
        assert_eq!(
            literal.origin(),
            source.value_components().items()[0].origin()
        );
    }
}

#[test]
fn invalid_z_index_values_fail_typed_checking_and_recovery_preserves_neighbors() {
    for &text in INVALID {
        let input = format!("color:red;z-index:{text};height:2px!important");
        let report = parse_style_attribute(&input);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid occurrence: {input}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(report.syntax().len(), 2);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Height
        );
        assert_eq!(report.syntax()[1].importance(), CssImportance::Important);
        assert!(validate_style_attribute(&input).is_err());
        let error = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ZIndex),
            parse_component_values(text).unwrap(),
            CssImportance::Normal,
        )
        .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
    }
    let components =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("none").unwrap()]).unwrap();
    let error = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ZIndex),
        components,
        CssImportance::Normal,
    )
    .unwrap_err();
    let responsible = match error.origin() {
        CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => origin,
        other => panic!("responsible origin: {other:?}"),
    };
    assert_eq!(responsible, &CssValueOrigin::Programmatic);
}

#[test]
fn z_index_initial_metadata_is_noninherited_auto() {
    let CssPropertyKindRef::Longhand(metadata) = CssKnownProperty::ZIndex
        .metadata()
        .expect("intrinsic z-index metadata")
        .kind()
    else {
        panic!("longhand")
    };
    assert!(!metadata.inherited_by_default());
    assert_eq!(
        metadata.property().known_property(),
        CssKnownProperty::ZIndex
    );
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("ordinary auto initial")
    };
    assert_eq!(
        initial,
        contribution(&declaration("auto")).ordinary_value().unwrap()
    );
}

#[test]
fn ordinary_and_css_wide_z_index_values_contribute_once_with_source_identity() {
    for &text in VALID {
        let source = declaration(text);
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ZIndex),
            source.value_components().clone(),
            CssImportance::Important,
        )
        .unwrap();
        let item = contribution(&source);
        assert!(item.ordinary_value().is_some());
        assert!(item.replacement_components().is_none());
        assert_eq!(
            item.ordinary_value(),
            contribution(&checked).ordinary_value()
        );
    }
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(text);
        assert_eq!(source.known().unwrap().global(), Some(keyword));
        assert_eq!(
            contribution(&source).value(),
            CssContributionValueRef::Global(keyword)
        );
    }
}

#[test]
fn pending_z_index_reentry_is_strict_atomic_and_preserves_replacement_origins() {
    let source = declaration("var(--level)");
    let CssExpansion::Pending(handle) = expand_declaration(&source).expect("pending z-index")
    else {
        panic!("pending")
    };
    assert!(handle.source().same_occurrence(&source));
    for text in INVALID
        .iter()
        .copied()
        .chain(["1; color:red", "1!important"])
    {
        assert!(
            matches!(
                handle
                    .reenter(parse_component_values(text).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ),
            "{text}"
        );
    }
    for text in ["var(--again)", "env(level)", "attr(data-level)"] {
        assert_eq!(
            handle
                .reenter(parse_component_values(text).unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
    }
    for text in ["auto", "+0002147483648", "calc(1.5)", "inherit"] {
        let replacement = parse_component_values(text).unwrap();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ZIndex),
            replacement.clone(),
            CssImportance::Important,
        )
        .unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("reentered longhand")
            };
            let [item] = values.items() else {
                panic!("one terminal")
            };
            assert_eq!(item.property(), CssKnownProperty::ZIndex);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            assert_eq!(item.value(), contribution(&checked).value());
        }
    }
}

#[test]
fn normalized_z_index_keeps_order_importance_and_atomic_cumulative_limits() {
    let text = ".a{z-index:auto;.child{z-index:-2}z-index:0!important}";
    let report = parse_sheet(text);
    assert!(report.is_clean());
    let normalized = normalize_sheet(report.syntax()).expect("normalized z-index");
    let items: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(item) => Some(item),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 3);
    for (index, (item, spelling)) in items.iter().zip(["auto", "-2", "0"]).enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(value(item.source()).as_css(), spelling);
        assert_eq!(
            item.source().position().unwrap().byte_offset().value(),
            text.find(&format!("z-index:{spelling}")).unwrap()
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("one contribution")
        };
        assert_eq!(values.items().len(), 1);
        assert!(values.items()[0].source().same_occurrence(item.source()));
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
        CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 2).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 2
        }
    );
    assert_eq!(error.declaration_order(), Some(2));
    assert!(
        error
            .declaration()
            .unwrap()
            .same_occurrence(items[2].source())
    );
    assert_eq!(
        normalize_sheet(report.syntax()).unwrap().items().len(),
        normalized.items().len()
    );
    assert!(
        normalize_sheet_with_limits(
            report.syntax(),
            CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 3).unwrap()
        )
        .is_ok()
    );
}
