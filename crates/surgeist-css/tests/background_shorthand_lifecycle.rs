#![forbid(unsafe_code)]
//! Backgrounds 3 CRD20240311 §2.10 defines per-layer initial-then-explicit
//! projection into seven lists, followed by one final-layer color.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#background
//! These tests use the existing public metadata/expansion/normalization boundary.

use surgeist_css::*;

const MEMBERS: [CssKnownProperty; 8] = [
    CssKnownProperty::BackgroundImage,
    CssKnownProperty::BackgroundPosition,
    CssKnownProperty::BackgroundSize,
    CssKnownProperty::BackgroundRepeat,
    CssKnownProperty::BackgroundAttachment,
    CssKnownProperty::BackgroundOrigin,
    CssKnownProperty::BackgroundClip,
    CssKnownProperty::BackgroundColor,
];

fn declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one authored declaration")
    };
    source.clone()
}

fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("background has intrinsic expansion")
    else {
        panic!("completed background members")
    };
    assert_context(&values, source);
    values
}

fn assert_context(values: &CssLonghandContributions, source: &CssDeclaration) {
    assert_eq!(
        values
            .items()
            .iter()
            .map(CssLonghandContribution::property)
            .collect::<Vec<_>>(),
        MEMBERS
    );
    for item in values.items() {
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), source.importance());
    }
}

fn text(value: CssLonghandValueRef<'_>) -> String {
    match value {
        CssLonghandValueRef::BackgroundImage(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BackgroundPosition(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BackgroundSize(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BackgroundRepeat(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BackgroundAttachment(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BackgroundOrigin(v) | CssLonghandValueRef::BackgroundClip(v) => {
            v.serialize_specified().unwrap()
        }
        CssLonghandValueRef::BackgroundColor(v) => v.to_specified_css().unwrap(),
        _ => panic!("background terminal"),
    }
}

fn assert_text(values: &CssLonghandContributions, expected: [&str; 8]) {
    for (item, expected) in values.items().iter().zip(expected) {
        let ordinary = item.ordinary_value().expect("ordinary background terminal");
        assert_eq!(ordinary.property().known_property(), item.property());
        assert_eq!(text(ordinary.view()), expected, "{:?}", item.property());
    }
}

#[test]
fn background_metadata_has_eight_ordered_settable_members_and_no_extra_resets() {
    let metadata = CssKnownProperty::Background
        .metadata()
        .expect("intrinsic background metadata");
    let CssPropertyKindRef::Shorthand(shorthand) = metadata.kind() else {
        panic!("background is a shorthand")
    };
    assert_eq!(
        shorthand
            .settable_members()
            .iter()
            .map(|p| p.known_property())
            .collect::<Vec<_>>(),
        MEMBERS
    );
    assert_eq!(shorthand.members(), shorthand.settable_members());
    assert!(shorthand.reset_only_members().is_empty());
}

#[test]
fn color_only_background_supplies_each_schema_initial_list_and_one_color() {
    // The red-only example in §2.10 explicitly enumerates these seven initials.
    let source = declaration("background:red!important");
    let values = completed(&source);
    assert_text(
        &values,
        [
            "none",
            "0% 0%",
            "auto",
            "repeat",
            "scroll",
            "padding-box",
            "border-box",
            "red",
        ],
    );
    for item in &values.items()[..7] {
        let CssPropertyKindRef::Longhand(metadata) = item.property().metadata().unwrap().kind()
        else {
            panic!("terminal metadata")
        };
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("intrinsic initial")
        };
        assert_eq!(item.ordinary_value().unwrap().view(), initial.view());
        assert!(item.replacement_components().is_none());
    }
    // Expansion never fills omissions in the authored shorthand graph itself.
    let CssKnownPropertyValueRef::Background(authored) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("background wrapper")
    };
    let [layer] = authored.background().layers() else {
        panic!("one authored layer")
    };
    assert!(layer.image().is_none() && layer.position().is_none() && layer.size().is_none());
    assert!(layer.repeat().is_none() && layer.attachment().is_none() && layer.boxes().is_none());
    assert_eq!(layer.color().unwrap().to_specified_css().unwrap(), "red");
}

#[test]
fn sparse_layers_each_receive_defaults_without_matching_image_counts() {
    for (final_layer, color) in [("red", "red"), ("none", "transparent")] {
        let source = declaration(&format!(
            "background:none, center / 1px content-box, {final_layer}"
        ));
        let values = completed(&source);
        // One visual box sets both origin and clip; absent final fields receive initials.
        assert_text(
            &values,
            [
                "none, none, none",
                "0% 0%, center center, 0% 0%",
                "auto, 1px auto, auto",
                "repeat, repeat, repeat",
                "scroll, scroll, scroll",
                "padding-box, content-box, padding-box",
                "border-box, content-box, border-box",
                color,
            ],
        );
        let CssLonghandValueRef::BackgroundSize(sizes) =
            values.items()[2].ordinary_value().unwrap().view()
        else {
            panic!("size list")
        };
        assert!(matches!(
            sizes.sizes()[1],
            CssBackgroundSize::Explicit { height: None, .. }
        ));
    }
}

#[test]
fn explicit_layers_preserve_all_components_and_single_final_color() {
    let source = declaration(concat!(
        "background:url(hero.png) left top / cover repeat-x fixed border-box content-box, ",
        "none right bottom / 1px no-repeat local padding-box #123456!important"
    ));
    let values = completed(&source);
    assert_text(
        &values,
        [
            "url(\"hero.png\"), none",
            "left top, right bottom",
            "cover, 1px auto",
            "repeat-x, no-repeat",
            "fixed, local",
            "border-box, padding-box",
            "content-box, padding-box",
            "rgb(18, 52, 86)",
        ],
    );
    assert!(
        values
            .items()
            .iter()
            .all(|item| item.replacement_components().is_none())
    );
}

#[test]
fn all_five_css_wide_keywords_expand_symbolically_to_eight_members() {
    for (css, keyword) in [
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("background:{css}!important"));
        let values = completed(&source);
        for item in values.items() {
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.ordinary_value().is_none());
            assert!(item.replacement_components().is_none());
        }
    }
}

#[test]
fn pending_background_reentry_is_strict_reusable_and_preserves_both_origins() {
    let programmatic_source = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Background),
        CssComponentValues::try_new(vec![
            CssComponentValue::try_function(
                "var",
                CssComponentValues::try_new(vec![
                    CssComponentValue::try_ident("--layers").unwrap(),
                ])
                .unwrap(),
            )
            .unwrap(),
        ])
        .unwrap(),
        CssImportance::Important,
    )
    .unwrap();
    assert!(programmatic_source.position().is_none());
    assert_eq!(
        programmatic_source.value_components().items()[0].origin(),
        &CssValueOrigin::Programmatic
    );
    for source in [
        declaration("background:var(--layers)!important"),
        programmatic_source,
    ] {
        let CssExpansion::Pending(pending) =
            expand_declaration(&source).expect("background can defer expansion")
        else {
            panic!("pending background")
        };
        assert!(pending.source().same_occurrence(&source));
        for invalid in ["red, none", "/ cover", "none,"] {
            let error = pending
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err();
            assert!(matches!(
                error.kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
        }
        for residual in ["var(--again)", "env(layer)", "attr(layer)"] {
            assert_eq!(
                pending
                    .reenter(parse_component_values(residual).unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
        }
        let parsed = parse_component_values("none, red").unwrap();
        let programmatic =
            CssComponentValues::try_new(vec![CssComponentValue::try_ident("red").unwrap()])
                .unwrap();
        assert!(matches!(
            parsed.items()[0].origin(),
            CssValueOrigin::Parsed(_)
        ));
        assert_eq!(
            programmatic.items()[0].origin(),
            &CssValueOrigin::Programmatic
        );
        for replacement in [parsed, programmatic] {
            for _ in 0..2 {
                let CssContributions::Longhands(values) =
                    pending.reenter(replacement.clone()).unwrap()
                else {
                    panic!("completed replacement")
                };
                assert_context(&values, &source);
                for item in values.items() {
                    assert_eq!(item.replacement_components(), Some(&replacement));
                    for (retained, supplied) in item
                        .replacement_components()
                        .unwrap()
                        .items()
                        .iter()
                        .zip(replacement.items())
                    {
                        assert_eq!(retained.origin(), supplied.origin());
                    }
                }
                let layers = if replacement.items().len() == 1 {
                    "none"
                } else {
                    "none, none"
                };
                assert_eq!(
                    text(values.items()[0].ordinary_value().unwrap().view()),
                    layers
                );
                assert_eq!(
                    text(values.items()[7].ordinary_value().unwrap().view()),
                    "red"
                );
            }
        }
        let replacement = parse_component_values("revert-layer").unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("eight symbolic replacement members")
        };
        assert_context(&values, &source);
        for item in values.items() {
            assert_eq!(
                item.value(),
                CssContributionValueRef::Global(CssGlobalKeyword::RevertLayer)
            );
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn normalization_keeps_one_shorthand_ordinal_but_charges_eight_contributions() {
    let css = ".a{background:red!important; background-size:cover}.b{background:var(--layers)}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 10).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), limits).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 3);
    for (index, value) in declarations.iter().enumerate() {
        assert_eq!(value.order(), index);
    }
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        declarations[0].expansion()
    else {
        panic!("eight background contributions")
    };
    assert_context(values, declarations[0].source());
    assert_eq!(
        declarations[0].source().importance(),
        CssImportance::Important
    );
    assert_eq!(
        declarations[0]
            .source()
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        css.find("background:").unwrap()
    );
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        declarations[1].expansion()
    else {
        panic!("ordinary sibling")
    };
    assert_eq!(values.items().len(), 1);
    assert_eq!(
        values.items()[0].property(),
        CssKnownProperty::BackgroundSize
    );
    assert_eq!(
        text(values.items()[0].ordinary_value().unwrap().view()),
        "cover"
    );
    let CssExpansion::Pending(pending) = declarations[2].expansion() else {
        panic!("one deferred group")
    };
    assert!(pending.source().same_occurrence(declarations[2].source()));
    for (limit, order) in [(7, 0), (8, 1), (9, 2)] {
        let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, limit).unwrap();
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded {
                resource: CssNormalizationResource::Contributions,
                limit
            }
        );
        assert_eq!(error.declaration_order(), Some(order));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(declarations[order].source())
        );
    }
}

#[test]
fn forbidden_nonfinal_color_and_size_without_position_drop_only_the_background() {
    // §2.10 allows color only in final-bg-layer and slash-size only after position.
    // Existing parser/background.rs reports these branches as InvalidPropertyValue.
    for (invalid, responsible) in [
        ("red, none", "red"),
        ("/ cover", "/"),
        ("left no-repeat / cover", "/"),
    ] {
        let css = format!("background:{invalid}; background-size:cover");
        let report = parse_style_attribute(&css);
        let [source] = report.syntax().as_slice() else {
            panic!("valid sibling retained")
        };
        assert_eq!(
            source.known().unwrap().property(),
            CssKnownProperty::BackgroundSize
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid background")
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            "background:".len() + invalid.find(responsible).unwrap()
        );
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("property diagnostic")
        };
        assert_eq!(detail.property().canonical_name(), "background");
    }
}
