#![forbid(unsafe_code)]
//! Backgrounds 3 (2024-03-11) §2.6: one noninherited `<bg-position>#`
//! longhand, initially `0% 0%`. Values stay authored and symbolic.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#background-position
//! This test-only checkpoint uses callable metadata, expansion and normalization
//! boundaries. Exact typed initial/payload and new serialization APIs are tested
//! separately when their functional implementation exists.

use surgeist_css::*;

fn declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("expected one clean declaration");
    };
    source.clone()
}

fn positions(source: &CssDeclaration) -> &CssBackgroundPositionList {
    let CssKnownPropertyValueRef::BackgroundPosition(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("expected authored background position");
    };
    value.positions()
}

fn completed(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).expect("background-position must expand")
    else {
        panic!("expected completed longhand contribution");
    };
    let [item] = items.items() else {
        panic!("one background-position contribution, independently of layer count");
    };
    item.clone()
}

fn assert_ordinary(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert_eq!(item.property(), CssKnownProperty::BackgroundPosition);
    assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
    assert_eq!(
        item.ordinary_value().unwrap().property().known_property(),
        CssKnownProperty::BackgroundPosition
    );
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
}

#[test]
fn background_position_metadata_is_a_noninherited_terminal_property() {
    let CssPropertyKindRef::Longhand(metadata) = CssKnownProperty::BackgroundPosition
        .metadata()
        .unwrap()
        .kind()
    else {
        panic!("background-position is a longhand");
    };
    assert_eq!(
        metadata.property().known_property(),
        CssKnownProperty::BackgroundPosition
    );
    assert!(!metadata.inherited_by_default());
}

#[test]
fn ordinary_layer_list_expands_to_its_terminal_with_original_importance() {
    let source = declaration("background-position: bottom 20% right 10px, left 5px top !important");
    let [first, second] = positions(&source).positions() else {
        panic!("two authored position layers");
    };
    assert!(matches!(
        first.horizontal(),
        CssHorizontalPosition::RightOffset(_)
    ));
    assert!(matches!(
        first.vertical(),
        CssVerticalPosition::BottomOffset(_)
    ));
    assert!(matches!(
        second.horizontal(),
        CssHorizontalPosition::LeftOffset(_)
    ));
    assert!(matches!(second.vertical(), CssVerticalPosition::Top));
    let item = completed(&source);
    assert_ordinary(&item, &source);
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert!(item.replacement_components().is_none());
}

#[test]
fn css_wide_keywords_remain_exact_symbolic_longhand_contributions() {
    for (css, keyword) in [
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("background-position:{css}!important"));
        let item = completed(&source);
        assert_eq!(item.property(), CssKnownProperty::BackgroundPosition);
        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
        assert!(item.ordinary_value().is_none());
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
    }
}

#[test]
fn pending_position_reentry_is_strict_repeatable_and_retains_origins() {
    let source = declaration("background-position:var(--position)!important");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("substitution-dependent background-position must be pending");
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid in ["left right", "left,", "inherit left"] {
        let error = pending
            .reenter(parse_component_values(invalid).unwrap())
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
    for residual in ["var(--again)", "env(position)", "attr(position)"] {
        assert_eq!(
            pending
                .reenter(parse_component_values(residual).unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
    }
    let replacement = parse_component_values("top 20px left, -10% calc(2px + 3%)").unwrap();
    for _ in 0..2 {
        let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("completed position replacement");
        };
        let [item] = items.items() else {
            panic!("one replacement contribution");
        };
        assert_ordinary(item, &source);
        assert_eq!(item.replacement_components(), Some(&replacement));
        assert!(matches!(
            item.replacement_components().unwrap().items()[0].origin(),
            CssValueOrigin::Parsed(_)
        ));
    }
    let CssContributions::Longhands(items) = pending
        .reenter(parse_component_values("unset").unwrap())
        .unwrap()
    else {
        panic!("global replacement");
    };
    let [item] = items.items() else {
        panic!("one global replacement")
    };
    assert_eq!(item.property(), CssKnownProperty::BackgroundPosition);
    assert_eq!(
        item.value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Unset)
    );
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert!(pending.source().same_occurrence(&source));
}

#[test]
fn normalization_preserves_position_occurrences_and_pending_state() {
    let report = parse_sheet(concat!(
        ".a{background-position:left!important; background-image:none; ",
        "background-position:var(--position)}"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    let [first, image, pending] = declarations.as_slice() else {
        panic!("three ordered declaration occurrences");
    };
    assert_eq!(first.order(), 0);
    assert_eq!(image.order(), 1);
    assert_eq!(pending.order(), 2);
    let CssExpansion::Contributions(CssContributions::Longhands(items)) = first.expansion() else {
        panic!("ordinary first position")
    };
    let [item] = items.items() else {
        panic!("one position contribution")
    };
    assert_ordinary(item, first.source());
    assert_eq!(item.source().importance(), CssImportance::Important);
    let CssExpansion::Contributions(CssContributions::Longhands(items)) = image.expansion() else {
        panic!("ordinary image control")
    };
    assert_eq!(
        items.items()[0].property(),
        CssKnownProperty::BackgroundImage
    );
    let CssExpansion::Pending(handle) = pending.expansion() else {
        panic!("last position remains pending");
    };
    assert!(handle.source().same_occurrence(pending.source()));
    assert_eq!(
        pending.source().known().unwrap().property(),
        CssKnownProperty::BackgroundPosition
    );
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
        CssKnownProperty::BackgroundPosition
    );
}

#[test]
fn authored_signed_and_symbolic_offsets_keep_axis_and_edge_meaning() {
    let source = declaration("background-position:bottom -20% right -10px, -5% calc(2px + 3%)");
    let [edges, free] = positions(&source).positions() else {
        panic!("two position layers")
    };
    assert!(
        matches!(edges.horizontal(), CssHorizontalPosition::RightOffset(offset)
        if matches!(offset.literal_component().unwrap().view(),
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
            if number.representation() == "-10" && unit == "px"))
    );
    assert!(
        matches!(edges.vertical(), CssVerticalPosition::BottomOffset(offset)
        if matches!(offset.literal_component().unwrap().view(),
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
            if number.representation() == "-20"))
    );
    assert!(
        matches!(free.horizontal(), CssHorizontalPosition::Offset(offset)
        if matches!(offset.literal_component().unwrap().view(),
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
            if number.representation() == "-5"))
    );
    assert!(
        matches!(free.vertical(), CssVerticalPosition::Offset(offset)
        if offset.calculation().is_some() && offset.literal_component().is_none())
    );
    assert!(matches!(
        source.value_components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
}

#[test]
fn invalid_position_forms_drop_the_declaration_and_preserve_valid_sibling() {
    for invalid in [
        "left right",
        "top bottom",
        "left,",
        "left 1px 2px",
        "inherit left",
    ] {
        let css = format!("background-position:{invalid}; background-image:none");
        let report = parse_style_attribute(&css);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid position diagnostic")
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("structured invalid-property-value diagnostic");
        };
        assert_eq!(detail.property(), CssKnownProperty::BackgroundPosition);
        let [sibling] = report.syntax().as_slice() else {
            panic!("only valid image sibling")
        };
        assert_eq!(
            sibling.known().unwrap().property(),
            CssKnownProperty::BackgroundImage
        );
        let validation = validate_style_attribute(&css).unwrap_err();
        assert_eq!(validation.diagnostics(), report.diagnostics());
    }
}
