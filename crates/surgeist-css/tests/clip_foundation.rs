#![forbid(unsafe_code)]

//! CSS Masking 1 Appendix A, selected 2021-08-05 Candidate Recommendation Draft:
//! https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/#propdef-clip
//! `clip` is one noninherited longhand, initially `auto`; `rect()` has four
//! ordered length-or-auto edges and permits signed lengths.

use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn clip(source: &CssDeclaration) -> &CssClip {
    let CssKnownPropertyValueRef::Clip(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed clip")
    };
    value.clip()
}

fn expanded(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one clip longhand contribution")
    };
    items.items().to_vec()
}

#[test]
fn masking_one_clip_metadata_has_one_noninherited_auto_longhand() {
    let feature = feature_metadata("official.property.clip").unwrap();
    assert_eq!(feature.source().id().as_str(), "S-MASKING1");
    assert_eq!(feature.production(), "#propdef-clip");
    let CssPropertyKindRef::Longhand(metadata) = CssKnownProperty::Clip.metadata().unwrap().kind()
    else {
        panic!("clip is one longhand")
    };
    assert_eq!(metadata.property().known_property(), CssKnownProperty::Clip);
    assert!(!metadata.inherited_by_default());
    let initial_value = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("clip initial is an ordinary auto value")
    };
    assert_eq!(initial.property().known_property(), CssKnownProperty::Clip);
    let auto = declaration("clip:auto");
    assert_eq!(clip(&auto), &CssClip::Auto);
    let [item] = expanded(&auto)
        .try_into()
        .unwrap_or_else(|_| panic!("one terminal"));
    assert_eq!(initial, item.ordinary_value().unwrap());
}

#[test]
fn rect_edges_retain_order_signed_lengths_and_symbolic_math() {
    for source in [
        "clip:rect(auto, -1px, calc(1px + 2em), 0)",
        "clip:rect(auto -1px calc(1px + 2em) 0)",
    ] {
        let declaration = declaration(source);
        let CssClip::Rect(rect) = clip(&declaration) else {
            panic!("rectangle: {source}")
        };
        assert!(matches!(rect.top(), CssClipEdge::Auto));
        assert!(
            matches!(rect.right(), CssClipEdge::Length(length) if exact_dimension(length.literal_component(), "-1", "px"))
        );
        assert!(
            matches!(rect.bottom(), CssClipEdge::Length(length) if length.calculation().is_some())
        );
        assert!(
            matches!(rect.left(), CssClipEdge::Length(length) if matches!(length.literal_component().unwrap().view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.representation()=="0"))
        );
        let [item] = expanded(&declaration)
            .try_into()
            .unwrap_or_else(|_| panic!("one terminal"));
        assert_eq!(item.property(), CssKnownProperty::Clip);
        assert!(item.ordinary_value().is_some());
        assert!(item.source().same_occurrence(&declaration));
    }
}

#[test]
fn mixed_separators_percentages_and_wrong_arity_drop_only_the_invalid_declaration() {
    for invalid in [
        "rect(0, 1px 2px, 3px)",
        "rect(0, 1px, 2px)",
        "rect(0, 1px, 2px, 3px, 4px)",
        "rect(0, 1%, 2px, 3px)",
        "rect(0, calc(1px + 2%), 2px, 3px)",
    ] {
        let source = format!("color:red;clip:{invalid};height:2px");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2, "{invalid}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Height
        );
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
    }
}

#[test]
fn all_globals_are_single_symbolic_clip_contributions() {
    for (spelling, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("clip:{spelling}!important"));
        let [item] = expanded(&source)
            .try_into()
            .unwrap_or_else(|_| panic!("one terminal"));
        assert_eq!(item.property(), CssKnownProperty::Clip);
        assert_eq!(item.value(), CssContributionValueRef::Global(expected));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn pending_clip_reentry_is_strict_repeatable_and_retains_replacement_provenance() {
    let source = declaration("clip:var(--region)!important");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("clip substitution is pending")
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid in [
        "rect(0, 1%, 2px, 3px)",
        "rect(0, 1px 2px, 3px)",
        "rect(0, 1px, 2px)",
    ] {
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
    let replacement = parse_component_values("rect(auto, -1px, 2em, 0)").unwrap();
    for _ in 0..2 {
        let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("one longhand")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(item.property(), CssKnownProperty::Clip);
        assert!(item.ordinary_value().is_some());
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
    let CssContributions::Longhands(items) = pending
        .reenter(parse_component_values("inherit").unwrap())
        .unwrap()
    else {
        panic!("one global")
    };
    assert_eq!(
        items.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );
}

#[test]
fn normalization_preserves_mixed_order_and_fails_at_contribution_boundary() {
    let report = parse_sheet(".a{clip:auto;flex-direction:column;clip:rect(auto, -1px, 2em, 0)}");
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
        CssKnownProperty::Clip,
        CssKnownProperty::FlexDirection,
        CssKnownProperty::Clip,
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
        CssKnownProperty::Clip
    );
}

fn exact_dimension(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
    expected_unit: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit })) if number.representation() == representation && unit == expected_unit)
}
