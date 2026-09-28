#![forbid(unsafe_code)]

//! CSS2 §17.6.1 (`border-spacing`), selected 2011-06-07 Recommendation:
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html#propdef-border-spacing
//! The one property is an inherited longhand with an effective ordered pair.

use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn spacing(source: &CssDeclaration) -> &CssBorderSpacing {
    let CssKnownPropertyValueRef::BorderSpacing(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed border-spacing")
    };
    value.spacing()
}

fn expanded(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one border-spacing longhand contribution")
    };
    items.items().to_vec()
}

#[test]
fn border_spacing_metadata_is_inherited_with_zero_on_both_axes() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::BorderSpacing.metadata().unwrap().kind()
    else {
        panic!("border-spacing is one longhand")
    };
    assert_eq!(
        metadata.property().known_property(),
        CssKnownProperty::BorderSpacing
    );
    assert!(metadata.inherited_by_default());
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("fixed initial effective 0 0")
    };
    assert_eq!(
        value.property().known_property(),
        CssKnownProperty::BorderSpacing
    );
    let authored = declaration("border-spacing:0");
    assert_eq!(spacing(&authored).horizontal().value(), &CssLength::Zero);
    assert_eq!(spacing(&authored).vertical().value(), &CssLength::Zero);
    let items = expanded(&authored);
    let [contribution] = items.as_slice() else {
        panic!("one zero-valued terminal")
    };
    assert_eq!(value, contribution.ordinary_value().unwrap());
}

#[test]
fn one_or_two_lengths_keep_horizontal_then_vertical_effective_values() {
    for (authored, horizontal, vertical) in [
        ("2px", 2.0, 2.0),
        ("2px 3px", 2.0, 3.0),
        ("0 4px", 0.0, 4.0),
    ] {
        let source = declaration(&format!("border-spacing:{authored}!important"));
        let value = spacing(&source);
        let horizontal_value = value.horizontal().value();
        let vertical_value = value.vertical().value();
        match horizontal_value {
            CssLength::Px(value) => assert_eq!(value.value(), horizontal),
            CssLength::Zero => assert_eq!(horizontal, 0.0),
            _ => panic!("horizontal length: {authored}"),
        }
        match vertical_value {
            CssLength::Px(value) => assert_eq!(value.value(), vertical),
            CssLength::Zero => assert_eq!(vertical, 0.0),
            _ => panic!("vertical length: {authored}"),
        }
        let items = expanded(&source);
        let [item] = items.as_slice() else {
            panic!("one terminal: {authored}")
        };
        assert_eq!(item.property(), CssKnownProperty::BorderSpacing);
        assert!(item.ordinary_value().is_some());
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
    }
}

#[test]
fn symbolic_pure_length_math_remains_in_the_authored_pair() {
    for authored in ["calc(1px + 2em)", "calc(1px + 2em) 3px"] {
        let source = declaration(&format!("border-spacing:{authored}"));
        let value = spacing(&source);
        assert!(matches!(value.horizontal().value(), CssLength::Calc(_)));
        if authored.ends_with("3px") {
            assert!(matches!(
                value.vertical().value(),
                CssLength::Px(length) if length.value() == 3.0
            ));
        } else {
            assert_eq!(value.horizontal(), value.vertical());
        }
        assert_eq!(expanded(&source).len(), 1);
    }
    for invalid in ["-1px", "1%", "1px 2px 3px"] {
        let report = parse_style_attribute(&format!("border-spacing:{invalid};color:red"));
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        assert_eq!(
            report.syntax().len(),
            1,
            "valid neighbor survives: {invalid}"
        );
    }
}

#[test]
fn css_wide_border_spacing_keywords_stay_symbolic_single_contributions() {
    for (spelling, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("border-spacing:{spelling}!important"));
        let items = expanded(&source);
        let [item] = items.as_slice() else {
            panic!("one global terminal: {spelling}")
        };
        assert_eq!(item.property(), CssKnownProperty::BorderSpacing);
        assert_eq!(item.value(), CssContributionValueRef::Global(expected));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn pending_border_spacing_reentry_is_strict_repeatable_and_provenanced() {
    let source = declaration("border-spacing:var(--spacing)!important");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("substitution-dependent border-spacing remains pending")
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid in ["1%", "-1px", "1px 2px 3px", "1px / 2px"] {
        assert!(matches!(
            pending
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
    assert_eq!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    let replacement = parse_component_values("2px 3em").unwrap();
    for _ in 0..2 {
        let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("reentered one terminal")
        };
        let [item] = items.items() else {
            panic!("one reentered border-spacing contribution")
        };
        assert_eq!(item.property(), CssKnownProperty::BorderSpacing);
        assert!(item.ordinary_value().is_some());
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
    let CssContributions::Longhands(items) = pending
        .reenter(parse_component_values("inherit").unwrap())
        .unwrap()
    else {
        panic!("reentered symbolic global")
    };
    assert_eq!(items.items().len(), 1);
    assert_eq!(
        items.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );
}

#[test]
fn normalization_keeps_mixed_order_and_fails_at_exact_contribution_limit() {
    let report = parse_sheet(".a{border-spacing:1px;flex-direction:column;border-spacing:2px 3em}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 3);
    for (index, property) in [
        CssKnownProperty::BorderSpacing,
        CssKnownProperty::FlexDirection,
        CssKnownProperty::BorderSpacing,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations[index].order(), index);
        assert_eq!(
            declarations[index].source().known().unwrap().property(),
            property
        );
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            declarations[index].expansion()
        else {
            panic!("normalized intrinsic terminal")
        };
        let [item] = items.items() else {
            panic!("one normalized contribution")
        };
        assert_eq!(item.property(), property);
        assert!(item.source().same_occurrence(declarations[index].source()));
    }
    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 2).unwrap();
    let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 2,
        }
    );
    assert_eq!(error.declaration_order(), Some(2));
    assert_eq!(
        error.declaration().unwrap().known().unwrap().property(),
        CssKnownProperty::BorderSpacing
    );
}
