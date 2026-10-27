//! CSS Text 4 §§8.1–8.2, selected 2026-08-14 Working Draft:
//! https://www.w3.org/TR/2026/WD-css-text-4-20260814/#propdef-word-spacing
//! https://www.w3.org/TR/2026/WD-css-text-4-20260814/#propdef-letter-spacing
//! Both inherited properties accept `normal | <length-percentage>`; percentage
//! resolution waits for the used font size of the current element.

use crate::css::support::important_declaration as declaration;
use crate::css::support::named_property::longhand_contributions;
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::conformance::{CssSupportStatus, feature_metadata};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty, CssLonghandContribution,
    CssPropertyGrammar, CssPropertyKindRef, CssPropertyTarget,
};

const PROPERTIES: &[(CssKnownProperty, &str, &str)] = &[
    (
        CssKnownProperty::WordSpacing,
        "word-spacing",
        "official.property.word-spacing",
    ),
    (
        CssKnownProperty::LetterSpacing,
        "letter-spacing",
        "baseline.property.letter-spacing",
    ),
];

fn expanded(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    longhand_contributions(source).items().to_vec()
}

#[test]
fn both_text_four_properties_have_inherited_normal_longhand_initials_and_effective_sources() {
    for &(property, name, id) in PROPERTIES {
        let feature = feature_metadata(id).expect("stable property feature");
        assert_eq!(feature.status(), CssSupportStatus::Complete, "{name}");
        assert_eq!(feature.source().id().as_str(), "X-TEXT4", "{name}");
        assert_eq!(feature.production(), format!("#propdef-{name}"));
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("{name} is one longhand")
        };
        assert_eq!(metadata.property().known_property(), property);
        assert!(metadata.inherited_by_default(), "{name}");
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("{name} normal initial")
        };
        assert_eq!(initial.property().known_property(), property);
        let normal = declaration(name, "normal");
        let [item] = expanded(&normal)
            .try_into()
            .unwrap_or_else(|_| panic!("one normal terminal"));
        assert_eq!(initial, item.ordinary_value().unwrap());
    }
}

#[test]
fn signed_percentages_and_mixed_math_remain_authored_symbolic_values() {
    for &(_, name, _) in PROPERTIES {
        for value in ["10%", "-12.5%", "calc(1em + 10%)", "calc(2% - 1px)"] {
            let source = declaration(name, value);
            assert_eq!(source.known().unwrap().property().canonical_name(), name);
            assert_eq!(
                source
                    .value_components()
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                value
            );
            let [item] = expanded(&source)
                .try_into()
                .unwrap_or_else(|_| panic!("one terminal"));
            assert_eq!(item.property().canonical_name(), name);
            assert!(item.ordinary_value().is_some(), "{name}:{value}");
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            let checked = CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Grammar(CssPropertyGrammar::from_name(name).unwrap()),
                CssComponentValues::parse(&CssParser::default(), value).unwrap(),
                CssImportance::Important,
            )
            .unwrap();
            assert_eq!(checked.known().unwrap().property().canonical_name(), name);
            assert_eq!(
                checked
                    .value_components()
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                value
            );
        }
    }
}

#[test]
fn exact_large_exponent_authored_tokens_and_signed_lengths_are_retained() {
    for &(_, name, _) in PROPERTIES {
        for value in [
            "1e999%",
            "-1e-999%",
            "1e999px",
            "normal",
            "-2em",
            "calc(1em - 2px)",
        ] {
            let source = declaration(name, value);
            assert_eq!(
                source
                    .value_components()
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                value
            );
            let [item] = expanded(&source)
                .try_into()
                .unwrap_or_else(|_| panic!("one terminal"));
            assert_eq!(item.property().canonical_name(), name);
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(&source));
        }
    }
}

#[test]
fn invalid_dimensions_keywords_and_multiple_values_drop_without_losing_neighbors() {
    for &(_, name, _) in PROPERTIES {
        for invalid in ["1", "1fr", "auto", "normal 1px", "10% 1px", "calc(1 + 2)"] {
            let source = format!("color:red;{name}:{invalid};height:2px");
            let report = CssParser::default().parse_block_contents(&source);
            assert_eq!(report.syntax().len(), 2, "{name}:{invalid}");
            assert_eq!(
                report.syntax()[0].known().unwrap().property(),
                CssKnownProperty::Color
            );
            assert_eq!(
                report.syntax()[1].known().unwrap().property(),
                CssKnownProperty::Height
            );
            let [diagnostic] = report.diagnostics() else {
                panic!("one invalid declaration")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert!(
                CssParser::default()
                    .parse_block_contents(&source)
                    .into_validation_result()
                    .is_err()
            );
        }
    }
}

#[test]
fn pending_spacing_reentry_is_strict_repeatable_and_preserves_source_and_replacement() {
    for &(property, name, _) in PROPERTIES {
        let source = declaration(name, "var(--spacing)");
        let CssExpansion::Pending(pending) = source.expand().unwrap() else {
            panic!("pending spacing")
        };
        assert!(pending.source().same_occurrence(&source));
        for invalid in ["1fr", "normal 1px", "10% 1px"] {
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
        let replacement =
            CssComponentValues::parse(&CssParser::default(), "calc(1em + 10%)").unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("one reentered longhand")
            };
            let [item] = items.items() else {
                panic!("one terminal")
            };
            assert_eq!(item.property(), property);
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn normalization_preserves_mixed_spacing_order_and_exact_contribution_failure() {
    let report = CssParser::default()
        .parse_sheet(".a{word-spacing:normal;letter-spacing:-2em;word-spacing:10%}");
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
        CssKnownProperty::WordSpacing,
        CssKnownProperty::LetterSpacing,
        CssKnownProperty::WordSpacing,
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
            panic!("one terminal")
        };
        let [item] = items.items() else {
            panic!("one terminal")
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
        CssKnownProperty::WordSpacing
    );
}
