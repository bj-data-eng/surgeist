//! Authored `content-visibility` from CSS Containment 2 WD (2022-09-17) §4.
//! Skipping, user relevance, and used containment require downstream context.

use crate::css::support::property_admission::{
    assert_checked_value_rejected, assert_declaration_rejected, assert_invalid_value,
};
use crate::css::support::{important_declaration, known_property};
use surgeist_css::*;

const NAME: &str = "content-visibility";

fn property() -> CssKnownProperty {
    known_property(NAME)
}

fn declaration(value: &str) -> CssDeclaration {
    let source = important_declaration(NAME, value);
    assert!(validate_style_attribute(&format!("{NAME}:{value}!important")).is_ok());
    source
}

fn contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("content-visibility contributes one terminal")
    };
    let [item] = values.items() else {
        panic!("one content-visibility terminal")
    };
    item.clone()
}

#[test]
fn content_visibility_is_a_noninherited_containment2_longhand_initially_visible() {
    let property = property();
    let grammar = CssPropertyGrammar::from_name(NAME).unwrap();
    assert_eq!(grammar.target_property(), property);
    assert_eq!(grammar.name(), NAME);
    assert_eq!(
        grammar.feature_id().as_str(),
        "baseline.property.content-visibility"
    );
    let support = property_support_metadata(NAME).unwrap();
    assert_eq!(support.property(), property);
    assert_eq!(support.feature().source().id().as_str(), "I-CONTAIN2");
    assert_eq!(
        support.feature().source().url(),
        Some("https://www.w3.org/TR/2022/WD-css-contain-2-20220917/")
    );
    assert_eq!(
        support.feature().production(),
        "#propdef-content-visibility"
    );

    let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
        panic!("content-visibility is a longhand")
    };
    assert_eq!(metadata.property().known_property(), property);
    assert!(!metadata.inherited_by_default());
    let initial = metadata.initial_value();
    assert_eq!(initial.property().known_property(), property);
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("visible is an intrinsic ordinary value")
    };
    assert_eq!(
        initial,
        contribution(&declaration("visible"))
            .ordinary_value()
            .unwrap()
    );
}

#[test]
fn only_three_keywords_construct_exact_typed_authored_values_and_one_terminal() {
    let mut terminals = Vec::new();
    for (text, expected) in [
        ("visible", CssContentVisibility::Visible),
        ("HIDDEN", CssContentVisibility::Hidden),
        (r"a\75 to", CssContentVisibility::Auto),
    ] {
        let checked = parse_property_value(
            CssPropertyNameRef::Known(property()),
            parse_component_values(text).unwrap(),
            CssImportance::Important,
        )
        .unwrap();
        for source in [declaration(text), checked] {
            let CssKnownPropertyValueRef::ContentVisibility(value) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("checked content-visibility value")
            };
            assert_eq!(value.value(), &expected);
            assert_eq!(value.as_css(), text);
            assert_eq!(
                source
                    .value_components()
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                text
            );
            assert_eq!(source.importance(), CssImportance::Important);
            let item = contribution(&source);
            assert_eq!(item.property(), property());
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
            terminals.push(item.ordinary_value().unwrap().clone());
        }
    }
    assert_eq!(terminals[0], terminals[1]);
    assert_eq!(terminals[2], terminals[3]);
    assert_eq!(terminals[4], terminals[5]);
    assert_ne!(terminals[0], terminals[2]);
    assert_ne!(terminals[0], terminals[4]);
    assert_ne!(terminals[2], terminals[4]);
}

#[test]
fn invalid_values_drop_only_their_occurrence_and_fail_checked_construction() {
    for (value, token) in [
        ("", None),
        ("collapse", Some("collapse")),
        ("none", Some("none")),
        ("auto hidden", Some("hidden")),
        ("visible,hidden", Some(",")),
        ("10px", Some("10px")),
    ] {
        assert_declaration_rejected(NAME, value, token);
        assert_checked_value_rejected(property(), value, token);
    }
}

#[test]
fn pending_substitutions_reenter_whole_grammar_without_losing_original_occurrence() {
    for pending_text in [
        "var(--visibility)",
        "env(--visibility)",
        "attr(data-visibility)",
    ] {
        let source = declaration(pending_text);
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("{pending_text} remains pending")
        };
        assert!(pending.source().same_occurrence(&source));
        for invalid in ["", "collapse", "auto hidden"] {
            assert!(matches!(
                pending
                    .reenter(parse_component_values(invalid).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
        }
        for unresolved in ["var(--again)", "env(--again)", "attr(data-again)"] {
            assert_eq!(
                pending
                    .reenter(parse_component_values(unresolved).unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
        }
        for (replacement_text, expected) in [
            ("hidden", CssContentVisibility::Hidden),
            ("auto", CssContentVisibility::Auto),
        ] {
            let replacement = parse_component_values(replacement_text).unwrap();
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("valid keyword completes the pending declaration")
            };
            let [item] = values.items() else {
                panic!("one substituted terminal")
            };
            assert_eq!(item.property(), property());
            assert_eq!(
                item.ordinary_value(),
                contribution(&declaration(replacement_text)).ordinary_value()
            );
            let replacement_source = declaration(replacement_text);
            let CssKnownPropertyValueRef::ContentVisibility(parsed) = replacement_source
                .known()
                .unwrap()
                .property_value()
                .unwrap()
            else {
                panic!("valid replacement has its expected typed value")
            };
            assert_eq!(parsed.value(), &expected);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            let CssContributions::Longhands(retried) =
                pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("same replacement remains retryable")
            };
            assert_eq!(retried.items()[0].ordinary_value(), item.ordinary_value());
            assert!(retried.items()[0].source().same_occurrence(&source));
            assert_eq!(
                retried.items()[0].replacement_components(),
                Some(&replacement)
            );
        }
        let CssContributions::Longhands(values) = pending
            .reenter(parse_component_values("revert-layer").unwrap())
            .unwrap()
        else {
            panic!("whole-value CSS-wide replacement completes")
        };
        assert_eq!(
            values.items()[0].value(),
            CssContributionValueRef::Global(CssGlobalKeyword::RevertLayer)
        );
        assert!(pending.source().same_occurrence(&source));
    }
}

#[test]
fn normalization_preserves_valid_order_and_fails_atomically_at_contribution_budget() {
    let text = ".a{content-visibility:visible;content-visibility:collapse;content-visibility:auto!important;content-visibility:hidden}";
    let report = parse_sheet(text);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert_invalid_value(
        report.diagnostics()[0].error().kind(),
        property(),
        Some("collapse"),
    );
    assert_eq!(
        validate_sheet(text).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let original = report.syntax().clone();
    let limits = CssNormalizationLimits::try_new(0, 1, 3, 2).unwrap();
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
        property()
    );
    assert_eq!(report.syntax(), &original);

    let normalized = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 3, 3).unwrap(),
    )
    .unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 3);
    for (order, (item, (keyword, importance))) in declarations
        .iter()
        .zip([
            ("visible", CssImportance::Normal),
            ("auto", CssImportance::Important),
            ("hidden", CssImportance::Normal),
        ])
        .enumerate()
    {
        assert_eq!(item.order(), order);
        assert_eq!(item.source().importance(), importance);
        assert_eq!(
            item.source()
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            keyword
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("normalized declaration has one terminal")
        };
        assert_eq!(values.items().len(), 1);
        assert!(values.items()[0].source().same_occurrence(item.source()));
    }
}
