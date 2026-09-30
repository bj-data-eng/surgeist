#![forbid(unsafe_code)]
//! Backgrounds 3 (2024-03-11) §2.3: one noninherited `<bg-image>#` longhand,
//! initially a one-item `none` list.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#propdef-background-image
//! Masking 1 (2021-08-05) §7.1 uses the same checked image-list boundary for
//! `mask-image`: `[ none | <image> | <mask-source> ]#`, initially `none`, noninherited.
//! https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/#propdef-mask-image

use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [value] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    value.clone()
}

fn authored(source: &CssDeclaration) -> &CssImageValueList {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::BackgroundImage(value) => value.images(),
        CssKnownPropertyValueRef::MaskImage(value) => value.images(),
        _ => panic!("typed image-list longhand"),
    }
}

#[test]
fn mask_shorthand_retains_url_and_none_and_rejects_gradients() {
    for (css, image) in [
        (
            "url(mask.svg)",
            CssImageValue::Url(CssUrl::try_new("mask.svg").unwrap()),
        ),
        ("none", CssImageValue::None),
    ] {
        let source = declaration(&format!("mask: {css}"));
        let CssKnownPropertyValueRef::Mask(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("mask shorthand");
        };
        let expected = CssMaskList::try_new(vec![
            CssMaskLayer::try_new(Some(image), None, None, None).unwrap(),
        ])
        .unwrap();
        assert_eq!(value.value(), &expected);
        assert_eq!(value.as_css(), css);
    }
    let report = parse_style_attribute("mask: linear-gradient(red, blue); color: red");
    assert_eq!(report.diagnostics().len(), 1);
    let [retained] = report.syntax().as_slice() else {
        panic!("only the valid sibling remains");
    };
    assert_eq!(
        retained.known().unwrap().property(),
        CssKnownProperty::Color
    );
}

#[test]
fn mask_layer_construction_preserves_the_supported_image_subset_and_nonempty_boundary() {
    assert!(CssMaskLayer::try_new(None, None, None, None).is_none());
    assert!(CssMaskLayer::try_new(Some(CssImageValue::None), None, None, None).is_some());
    assert!(
        CssMaskLayer::try_new(
            Some(CssImageValue::Url(CssUrl::try_new("mask.svg").unwrap())),
            None,
            None,
            None,
        )
        .is_some()
    );
    assert!(CssMaskLayer::try_new(None, None, Some(CssBackgroundSize::Contain), None).is_some());
    let source = declaration("background-image: linear-gradient(red, blue)");
    let gradient = authored(&source).images()[0].clone();
    assert!(matches!(
        gradient,
        CssImageValue::Gradient(CssGradient::Linear(_))
    ));
    assert!(CssMaskLayer::try_new(Some(gradient.clone()), None, None, None).is_none());
    assert!(
        CssMaskLayer::try_new(Some(gradient), None, Some(CssBackgroundSize::Contain), None)
            .is_none()
    );
}

fn expanded(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one image-list longhand")
    };
    items.items().to_vec()
}

fn expanded_images(value: &CssLonghandValue) -> &CssImageValueList {
    match value.view() {
        CssLonghandValueRef::BackgroundImage(images) | CssLonghandValueRef::MaskImage(images) => {
            images
        }
        _ => panic!("typed image-list contribution"),
    }
}

#[test]
fn background_image_is_one_noninherited_longhand_initially_none() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::BackgroundImage.metadata().unwrap().kind()
    else {
        panic!("background-image is a longhand")
    };
    assert_eq!(
        metadata.property().known_property(),
        CssKnownProperty::BackgroundImage
    );
    assert!(!metadata.inherited_by_default());
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("one intrinsic none image")
    };
    assert_eq!(
        initial.property().known_property(),
        CssKnownProperty::BackgroundImage
    );
    let CssLonghandValueRef::BackgroundImage(initial_images) = initial.view() else {
        panic!("typed background-image initial")
    };
    assert!(matches!(initial_images.images(), [CssImageValue::None]));
    assert_eq!(initial_images.serialize_specified().unwrap(), "none");
    let source = declaration("background-image:none");
    assert!(matches!(authored(&source).images(), [CssImageValue::None]));
    let [contribution] = expanded(&source)
        .try_into()
        .unwrap_or_else(|_| panic!("one item"));
    assert_eq!(initial, contribution.ordinary_value().unwrap());
    assert_eq!(
        expanded_images(contribution.ordinary_value().unwrap()),
        authored(&source)
    );
}

#[test]
fn mask_image_is_one_noninherited_longhand_initially_none() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::MaskImage.metadata().unwrap().kind()
    else {
        panic!("mask-image is a longhand")
    };
    assert_eq!(
        metadata.property().known_property(),
        CssKnownProperty::MaskImage
    );
    assert!(!metadata.inherited_by_default());
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("one intrinsic none mask image")
    };
    assert_eq!(
        initial.property().known_property(),
        CssKnownProperty::MaskImage
    );
    let CssLonghandValueRef::MaskImage(initial_images) = initial.view() else {
        panic!("typed mask-image initial")
    };
    assert!(matches!(initial_images.images(), [CssImageValue::None]));
    assert_eq!(initial_images.serialize_specified().unwrap(), "none");
    let source = declaration("mask-image:none");
    assert!(matches!(authored(&source).images(), [CssImageValue::None]));
    let [contribution] = expanded(&source)
        .try_into()
        .unwrap_or_else(|_| panic!("one item"));
    assert_eq!(initial, contribution.ordinary_value().unwrap());
    assert_eq!(
        expanded_images(contribution.ordinary_value().unwrap()),
        authored(&source)
    );
}

#[test]
fn ordered_images_expand_once_with_source_and_importance() {
    let source = declaration(concat!(
        "background-image:none, src(\"hero.svg\"), linear-gradient(red, blue), ",
        "radial-gradient(circle, red, blue), ",
        "repeating-linear-gradient(45deg, red, blue), ",
        "repeating-radial-gradient(red, blue)!important"
    ));
    let current = authored(&source);
    assert!(matches!(
        current.images(),
        [
            CssImageValue::None,
            CssImageValue::Url(_),
            CssImageValue::Gradient(CssGradient::Linear(_)),
            CssImageValue::Gradient(CssGradient::Radial(_)),
            CssImageValue::Gradient(CssGradient::RepeatingLinear(_)),
            CssImageValue::Gradient(CssGradient::RepeatingRadial(_)),
        ]
    ));
    assert!(matches!(
        current.images()[1],
        CssImageValue::Url(ref url)
            if url.function() == CssUrlFunction::Src && url.as_str() == "hero.svg"
    ));
    assert_eq!(current.images().len(), 6);
    let [item] = expanded(&source)
        .try_into()
        .unwrap_or_else(|_| panic!("one terminal"));
    assert_eq!(item.property(), CssKnownProperty::BackgroundImage);
    assert!(item.ordinary_value().is_some());
    let contributed = expanded_images(item.ordinary_value().unwrap());
    assert_eq!(contributed, current);
    assert_eq!(
        contributed.serialize_specified().unwrap(),
        concat!(
            "none, src(\"hero.svg\"), linear-gradient(red, blue), ",
            "radial-gradient(circle, red, blue), ",
            "repeating-linear-gradient(45deg, red, blue), ",
            "repeating-radial-gradient(red, blue)"
        )
    );
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert!(item.replacement_components().is_none());
}

#[test]
fn ordered_mask_sources_and_gradients_expand_once_with_source_and_importance() {
    let source = declaration(concat!(
        "mask-image:url(\"#clip\"), src(\"#mask\"), none, ",
        "linear-gradient(red, blue)!important"
    ));
    let current = authored(&source);
    assert!(matches!(
        current.images(),
        [
            CssImageValue::Url(first),
            CssImageValue::Url(second),
            CssImageValue::None,
            CssImageValue::Gradient(CssGradient::Linear(_)),
        ] if first.as_str() == "#clip" && first.function() == CssUrlFunction::Url
            && second.as_str() == "#mask" && second.function() == CssUrlFunction::Src
    ));
    let [item] = expanded(&source)
        .try_into()
        .unwrap_or_else(|_| panic!("one terminal"));
    assert_eq!(item.property(), CssKnownProperty::MaskImage);
    assert!(item.ordinary_value().is_some());
    let contributed = expanded_images(item.ordinary_value().unwrap());
    assert_eq!(contributed, current);
    assert_eq!(
        contributed.serialize_specified().unwrap(),
        "url(\"#clip\"), src(\"#mask\"), none, linear-gradient(red, blue)"
    );
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert!(item.replacement_components().is_none());
}

#[test]
fn globals_and_universal_reset_keep_background_image_symbolic() {
    for (spelling, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("background-image:{spelling}!important"));
        let [item] = expanded(&source)
            .try_into()
            .unwrap_or_else(|_| panic!("one global"));
        assert_eq!(item.property(), CssKnownProperty::BackgroundImage);
        assert_eq!(item.value(), CssContributionValueRef::Global(expected));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
    let all = declaration("all:initial!important");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all).unwrap()
    else {
        panic!("symbolic all reset")
    };
    assert_eq!(reset.keyword(), CssGlobalKeyword::Initial);
    assert!(!reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage)));
    assert!(reset.source().same_occurrence(&all));
}

#[test]
fn globals_and_universal_reset_keep_mask_image_symbolic() {
    for (spelling, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("mask-image:{spelling}!important"));
        let [item] = expanded(&source)
            .try_into()
            .unwrap_or_else(|_| panic!("one global"));
        assert_eq!(item.property(), CssKnownProperty::MaskImage);
        assert_eq!(item.value(), CssContributionValueRef::Global(expected));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
    let all = declaration("all:initial!important");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all).unwrap()
    else {
        panic!("symbolic all reset")
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::MaskImage)));
    assert!(reset.source().same_occurrence(&all));
}

#[test]
fn pending_image_list_reentry_is_strict_repeatable_and_provenanced() {
    let source = declaration("background-image:var(--art)!important");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("known image substitution must be pending")
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid in [
        "url(\"ok.svg\"),",
        "none, linear-gradient(red)",
        "none, url(\"ok.svg\" 1)",
        "inherit none",
    ] {
        assert!(
            matches!(
                pending
                    .reenter(parse_component_values(invalid).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ),
            "{invalid}"
        );
    }
    assert_eq!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    for replacement in [
        "url(\"ok.svg\"), none",
        "src(\"hero.svg\"), repeating-radial-gradient(red, blue)",
        "linear-gradient(red, blue), radial-gradient(red, blue)",
    ] {
        let replacement = parse_component_values(replacement).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("one reentered longhand")
            };
            let [item] = items.items() else {
                panic!("one terminal")
            };
            assert_eq!(item.property(), CssKnownProperty::BackgroundImage);
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
    let CssContributions::Longhands(items) = pending
        .reenter(parse_component_values("inherit").unwrap())
        .unwrap()
    else {
        panic!("global replacement")
    };
    assert_eq!(
        items.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );
}

#[test]
fn pending_mask_sources_reenter_strictly_and_retain_replacement_provenance() {
    let source = declaration("mask-image:var(--mask)!important");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("known mask image substitution must be pending")
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid in [
        "none,",
        "url(\"#mask\" 1)",
        "radial-gradient(red)",
        "inherit none",
    ] {
        assert!(
            matches!(
                pending
                    .reenter(parse_component_values(invalid).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ),
            "{invalid}"
        );
    }
    assert_eq!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    for replacement in [
        "url(\"#clip\"), src(\"#mask\"), none",
        "repeating-linear-gradient(red, blue), radial-gradient(red, blue)",
    ] {
        let replacement = parse_component_values(replacement).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("one reentered mask-image longhand")
            };
            let [item] = items.items() else {
                panic!("one terminal")
            };
            assert_eq!(item.property(), CssKnownProperty::MaskImage);
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn image_components_keep_parsed_and_checked_programmatic_origins() {
    assert!(CssImageValueList::try_new(Vec::new()).is_none());
    let current = CssImageValueList::try_new(vec![
        CssImageValue::None,
        CssImageValue::Url(CssUrl::from_parts(CssUrlFunction::Src, "hero.svg", vec![])),
    ])
    .unwrap();
    assert!(
        matches!(current.images(), [CssImageValue::None, CssImageValue::Url(url)] if url.function() == CssUrlFunction::Src)
    );

    let parsed = declaration("background-image:src(\"hero.svg\")");
    assert!(matches!(
        parsed.value_components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    let component = CssComponentValue::try_url("hero.svg").unwrap();
    let components = CssComponentValues::try_new(vec![component]).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage),
        components,
        CssImportance::Normal,
    )
    .unwrap();
    assert!(matches!(
        checked.value_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    assert!(
        matches!(authored(&checked).images(), [CssImageValue::Url(url)] if url.as_str() == "hero.svg")
    );
    let parsed_mask = declaration("mask-image:src(\"#mask\")");
    assert!(matches!(
        parsed_mask.value_components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    let checked_mask = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::MaskImage),
        CssComponentValues::try_new(vec![CssComponentValue::try_url("#mask").unwrap()]).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert!(matches!(
        checked_mask.value_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    assert!(
        matches!(authored(&checked_mask).images(), [CssImageValue::Url(url)] if url.as_str() == "#mask")
    );
}

#[test]
fn both_initial_image_lists_use_one_cumulative_serialization_budget() {
    for property in [
        CssKnownProperty::BackgroundImage,
        CssKnownProperty::MaskImage,
    ] {
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("image list longhand")
        };
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("ordinary image-list initial")
        };
        let list = expanded_images(initial);
        // A checked list aggregate and its one `none` leaf each charge one
        // input and projection node; the complete specified text is four bytes.
        assert_eq!(
            list.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                2, 2, 4
            ))
            .unwrap(),
            "none"
        );
        for (limits, expected) in [
            (
                CssSpecifiedValueSerializationLimits::new(1, 2, 4),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(2, 1, 4),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(2, 2, 3),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                list.serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                expected
            );
        }
        assert_eq!(list.serialize_specified().unwrap(), "none");
    }
}

#[test]
fn normalization_preserves_image_order_and_fails_atomically_on_contribution_limit() {
    let report = parse_sheet(concat!(
        ".a{background-image:url(\"a.svg\");",
        "border-image-source:none;",
        "background-image:linear-gradient(red, blue), none}"
    ));
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
        CssKnownProperty::BackgroundImage,
        CssKnownProperty::BorderImageSource,
        CssKnownProperty::BackgroundImage,
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
            panic!("one normalized longhand")
        };
        let [item] = items.items() else {
            panic!("one contribution")
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
        CssKnownProperty::BackgroundImage
    );
}

#[test]
fn normalization_preserves_mask_image_order_and_exact_resource_failure() {
    let report = parse_sheet(concat!(
        ".a{mask-image:url(\"#first\");",
        "border-image-source:none;",
        "mask-image:src(\"#second\"), none}"
    ));
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
        CssKnownProperty::MaskImage,
        CssKnownProperty::BorderImageSource,
        CssKnownProperty::MaskImage,
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
            panic!("one normalized longhand")
        };
        let [item] = items.items() else {
            panic!("one contribution")
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
        CssKnownProperty::MaskImage
    );
}
