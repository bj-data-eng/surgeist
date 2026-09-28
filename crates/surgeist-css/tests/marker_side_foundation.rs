#![forbid(unsafe_code)]

//! Existing-API expectations from CSS Lists 3 (2020-11-17) §3.7:
//! https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#propdef-marker-side
//! The two specified keywords remain symbolic; marker placement is downstream.

use surgeist_css::*;

const NAME: &str = "marker-side";

fn property() -> CssKnownProperty {
    CssKnownProperty::from_name(NAME).expect("marker-side is a known property")
}

fn declaration(value: &str) -> CssDeclaration {
    let source = format!("{NAME}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained marker-side declaration: {source}")
    };
    assert_eq!(declaration.known().unwrap().property(), property());
    assert!(validate_style_attribute(&source).is_ok(), "{source}");
    declaration.clone()
}

fn contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
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
    let property = property();
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
        contribution(&declaration("match-self"))
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
        let source = declaration(value);
        assert!(source.known().unwrap().property_value().is_some());
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            value
        );
        let item = contribution(&source);
        assert_eq!(item.property(), property());
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
        let report = parse_style_attribute(&source);
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
        assert!(validate_style_attribute(&source).is_err());
    }

    // The sibling list-style shorthand has its own three members and grammar.
    let report = parse_style_attribute("list-style:outside none disc");
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
    let property = property();
    let components =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("match-parent").unwrap()])
            .unwrap();
    let source = parse_property_value(
        CssPropertyNameRef::Known(property),
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
    let invalid =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("outside").unwrap()])
            .unwrap();
    assert!(parse_property_value(
        CssPropertyNameRef::Known(property),
        invalid,
        CssImportance::Normal,
    )
    .is_err());
}

#[test]
fn css_wide_keywords_and_universal_reset_include_marker_side() {
    for (value, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(value);
        let item = contribution(&source);
        assert_eq!(item.property(), property());
        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
    let escaped = declaration(r"\69 nherit");
    assert_eq!(
        contribution(&escaped).value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );

    let all = parse_style_attribute("all:initial");
    assert!(all.is_clean(), "{:?}", all.diagnostics());
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all.syntax()[0]).unwrap()
    else {
        panic!("all yields a universal reset")
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(property())));
}

#[test]
fn pending_reentry_is_strict_repeatable_and_preserves_source_and_replacement() {
    let source = declaration("var(--side)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("known marker-side with var() is pending")
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid in ["outside", "match-parent match-self", ""] {
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
    for value in ["match-self", "match-parent", "inherit"] {
        let replacement = parse_component_values(value).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("one reentered longhand")
            };
            let [item] = values.items() else {
                panic!("one marker-side terminal")
            };
            assert_eq!(item.property(), property());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn normalization_preserves_order_and_reports_an_atomic_contribution_limit() {
    let report = parse_sheet(".a{marker-side:match-parent;color:red;marker-side:match-self}");
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
    for (order, property) in [property(), CssKnownProperty::Color, property()]
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
}
