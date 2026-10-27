//! CSS2 §17.6.1 (`border-spacing`), selected 2011-06-07 Recommendation:
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html#propdef-border-spacing
//! The one property is an inherited longhand with an effective ordered pair.

use crate::css::support::sole_declaration as declaration;
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::{CssDeclaration, CssGlobalKeyword, CssImportance};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::numeric::{CssLength, CssNonNegative, CssNumeric};
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssLonghandContribution, CssPropertyKindRef,
};
use surgeist_css::values::common::CssSidePair;

fn spacing(source: &CssDeclaration) -> &CssSidePair<CssNumeric<CssLength, CssNonNegative>> {
    let CssKnownPropertyValueRef::BorderSpacing(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed border-spacing")
    };
    value.value()
}

fn expanded(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) = source.expand().unwrap()
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
    assert!(exact_literal(
        spacing(&authored)
            .first()
            .literal()
            .map(|literal| literal.component()),
        "0"
    ));
    assert!(exact_literal(
        spacing(&authored)
            .second()
            .literal()
            .map(|literal| literal.component()),
        "0"
    ));
    let items = expanded(&authored);
    let [contribution] = items.as_slice() else {
        panic!("one zero-valued terminal")
    };
    assert_eq!(value, contribution.ordinary_value().unwrap());
}

#[test]
fn one_or_two_lengths_keep_horizontal_then_vertical_effective_values() {
    for (authored, horizontal, vertical) in [
        ("2px", "2px", "2px"),
        ("2px 3px", "2px", "3px"),
        ("0 4px", "0", "4px"),
    ] {
        let source = declaration(&format!("border-spacing:{authored}!important"));
        let value = spacing(&source);
        assert!(exact_literal(
            value.first().literal().map(|literal| literal.component()),
            horizontal
        ));
        assert!(exact_literal(
            value.second().literal().map(|literal| literal.component()),
            vertical
        ));
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
        assert!(value.first().calculation().is_some());
        if authored.ends_with("3px") {
            assert!(exact_dimension(
                value.second().literal().map(|literal| literal.component()),
                "3",
                "px"
            ));
        } else {
            assert_eq!(value.first(), value.second());
        }
        assert_eq!(expanded(&source).len(), 1);
    }
    for invalid in ["-1px", "1%", "1px 2px 3px"] {
        let report = CssParser::default()
            .parse_block_contents(format!("border-spacing:{invalid};color:red"));
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
fn pending_border_spacing_reentry_is_strict_repeatable_and_provenanced() {
    let source = declaration("border-spacing:var(--spacing)!important");
    let CssExpansion::Pending(pending) = source.expand().unwrap() else {
        panic!("substitution-dependent border-spacing remains pending")
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid in ["1%", "-1px", "1px 2px 3px", "1px / 2px"] {
        assert!(
            pending
                .reenter(CssComponentValues::parse(&CssParser::default(), invalid).unwrap())
                .unwrap_err()
                .replacement_error()
                .is_some()
        );
    }
    assert!(
        pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap())
            .unwrap_err()
            .is_residual_substitution()
    );
    let replacement = CssComponentValues::parse(&CssParser::default(), "2px 3em").unwrap();
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
        .reenter(CssComponentValues::parse(&CssParser::default(), "inherit").unwrap())
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
    let report = CssParser::default()
        .parse_sheet(".a{border-spacing:1px;flex-direction:column;border-spacing:2px 3em}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = report.syntax().normalize(Default::default()).unwrap();
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
    let error = report.syntax().normalize(limits).unwrap_err();
    assert_eq!(
        error.exhausted_limit(),
        Some((CssNormalizationResource::Contributions, 2))
    );
    assert_eq!(error.declaration_order(), Some(2));
    assert_eq!(
        error.declaration().unwrap().known().unwrap().property(),
        CssKnownProperty::BorderSpacing
    );
}

fn exact_dimension(
    component: Option<&surgeist_css::component::CssComponentValue>,
    representation: &str,
    expected_unit: &str,
) -> bool {
    matches!(component.map(surgeist_css::component::CssComponentValue::view), Some(surgeist_css::component::CssComponentValueRef::Token(surgeist_css::component::CssValueTokenRef::Dimension { number, unit })) if number.representation() == representation && unit == expected_unit)
}
fn exact_literal(
    component: Option<&surgeist_css::component::CssComponentValue>,
    css: &str,
) -> bool {
    use surgeist_css::component::CssComponentValueRef as Component;
    use surgeist_css::component::CssValueTokenRef as Token;
    let expected = surgeist_css::component::CssComponentValue::try_token(css).unwrap();
    match (
        component.map(surgeist_css::component::CssComponentValue::view),
        expected.view(),
    ) {
        (
            Some(Component::Token(Token::Number(actual))),
            Component::Token(Token::Number(expected)),
        )
        | (
            Some(Component::Token(Token::Percentage(actual))),
            Component::Token(Token::Percentage(expected)),
        ) => actual.representation() == expected.representation(),
        (
            Some(Component::Token(Token::Dimension {
                number: actual,
                unit: actual_unit,
            })),
            Component::Token(Token::Dimension {
                number: expected,
                unit: expected_unit,
            }),
        ) => actual.representation() == expected.representation() && actual_unit == expected_unit,
        _ => false,
    }
}
