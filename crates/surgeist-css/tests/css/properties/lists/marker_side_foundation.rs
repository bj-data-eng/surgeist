//! Existing-API expectations from CSS Lists 3 (2020-11-17) §3.7:
//! https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#propdef-marker-side
//! The two specified keywords remain symbolic; marker placement is downstream.

use crate::css::support::named_property::assert_programmatic_rejection;
use crate::css::support::{known_property, validated_important_declaration};
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues, CssValueOrigin};
use surgeist_css::conformance::{CssSupportStatus, feature_metadata};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty, CssLonghandContribution,
    CssPropertyGrammar, CssPropertyKindRef, CssPropertyNameRef, CssPropertyTarget,
};

const NAME: &str = "marker-side";

fn contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = source.expand().unwrap()
    else {
        panic!("marker-side expands to one longhand")
    };
    let [item] = values.items() else {
        panic!("one marker-side terminal")
    };
    item.clone()
}

#[test]
fn marker_side_is_an_inherited_lists3_longhand_with_match_self_initial() {
    let property = known_property(NAME);
    let grammar = CssPropertyGrammar::from_name(NAME).unwrap();
    assert_eq!(grammar.target_property(), property);
    assert_eq!(grammar.name(), NAME);
    let feature = feature_metadata(grammar.feature_id().as_str()).unwrap();
    assert_eq!(feature.status(), CssSupportStatus::Complete);
    assert_eq!(feature.source().id().as_str(), "I-LISTS3");
    assert_eq!(feature.production(), "#propdef-marker-side");

    let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
        panic!("marker-side is a single longhand")
    };
    assert!(metadata.inherited_by_default());
    let initial_value = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("match-self is a fixed initial")
    };
    assert_eq!(
        initial,
        contribution(&validated_important_declaration(NAME, "match-self"))
            .ordinary_value()
            .unwrap()
    );
}

#[test]
fn two_keywords_and_case_insensitive_identifiers_retain_authored_spelling() {
    for value in [
        "match-self",
        "match-parent",
        "MATCH-SELF",
        r"m\61 tch-parent",
    ] {
        let source = validated_important_declaration(NAME, value);
        assert!(source.known().unwrap().property_value().is_some());
        assert_eq!(
            source
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            value
        );
        let item = contribution(&source);
        assert_eq!(item.property(), known_property(NAME));
        assert!(item.ordinary_value().is_some());
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn malformed_marker_side_values_drop_only_their_declaration() {
    for invalid in [
        "",
        "outside",
        "inside",
        "match-self match-parent",
        "match-parent, match-self",
        "1px",
    ] {
        let source = format!("color:red;{NAME}:{invalid};height:2px");
        let report = CssParser::default().parse_block_contents(&source);
        assert_eq!(
            report
                .syntax()
                .iter()
                .map(|item| item.known().unwrap().property())
                .collect::<Vec<_>>(),
            [CssKnownProperty::Color, CssKnownProperty::Height],
            "{source}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one whole-declaration diagnostic: {source}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .is_err()
        );
    }

    // The sibling list-style shorthand has its own three members and grammar.
    let report = CssParser::default().parse_block_contents("list-style:outside none disc");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let shorthand = CssKnownProperty::from_name("list-style").unwrap();
    let CssPropertyKindRef::Shorthand(metadata) = shorthand.metadata().unwrap().kind() else {
        panic!("list-style remains a shorthand")
    };
    assert_eq!(
        metadata
            .settable_members()
            .iter()
            .map(|member| member.known_property().canonical_name())
            .collect::<Vec<_>>(),
        ["list-style-position", "list-style-image", "list-style-type"]
    );
    assert!(metadata.reset_only_members().is_empty());
}

#[test]
fn programmatic_components_use_the_same_grammar_and_origin() {
    let property = known_property(NAME);
    let components = CssComponentValues::try_new(
        vec![CssComponentValue::try_ident("match-parent").unwrap()],
        Default::default(),
    )
    .unwrap();
    let source = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
        components,
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(source.known().unwrap().property(), property);
    assert!(matches!(
        source.value_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    assert!(contribution(&source).ordinary_value().is_some());
    let invalid = CssComponentValues::try_new(
        vec![CssComponentValue::try_ident("outside").unwrap()],
        Default::default(),
    )
    .unwrap();
    assert_programmatic_rejection(
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
            invalid,
            CssImportance::Normal,
        ),
        property,
    );
}

#[test]
fn pending_reentry_is_strict_repeatable_and_preserves_source_and_replacement() {
    let source = validated_important_declaration(NAME, "var(--side)");
    let CssExpansion::Pending(pending) = source.expand().unwrap() else {
        panic!("known marker-side with var() is pending")
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid in ["outside", "match-parent match-self", ""] {
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
    for value in ["match-self", "match-parent", "inherit"] {
        let replacement = CssComponentValues::parse(&CssParser::default(), value).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("one reentered longhand")
            };
            let [item] = values.items() else {
                panic!("one marker-side terminal")
            };
            assert_eq!(item.property(), known_property(NAME));
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn normalization_preserves_order_and_reports_an_atomic_contribution_limit() {
    let report = CssParser::default()
        .parse_sheet(".a{marker-side:match-parent;color:red;marker-side:match-self}");
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
    for (order, property) in [
        known_property(NAME),
        CssKnownProperty::Color,
        known_property(NAME),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations[order].order(), order);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declarations[order].expansion()
        else {
            panic!("one normalized terminal")
        };
        let [item] = values.items() else {
            panic!("one contribution per declaration")
        };
        assert_eq!(item.property(), property);
        assert!(item.source().same_occurrence(declarations[order].source()));
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
        known_property(NAME)
    );
}
