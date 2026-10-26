//! Functional Backgrounds 3 list lifecycle and specified serialization.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/
//! CSSOM 1 §6.7.2 requires shorter equivalents subject to property-specific exceptions.
//! https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-value
//! Size height exception: https://github.com/w3c/csswg-drafts/issues/7802#issuecomment-2770612154

use crate::css::support::serialization_limits::assert_limits_with_zero as exact_bounds;
use crate::css::support::sole_declaration as declaration;
use surgeist_css::{CssBackgroundRepeatStyle as R, *};

fn size(css: &str) -> CssBackgroundSizeList {
    let source = declaration(&format!("background-size:{css}"));
    let CssKnownPropertyValueRef::BackgroundSize(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("size list")
    };
    value.value().clone()
}

fn length(css: &str) -> CssNumeric<CssLengthPercentage, CssNonNegative> {
    CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
        CssComponentValue::try_token(css).unwrap(),
    )
    .unwrap()
}

fn text(value: CssLonghandValueRef<'_>) -> String {
    match value {
        CssLonghandValueRef::BackgroundSize(v) => {
            v.serialize_specified(Default::default()).unwrap()
        }
        CssLonghandValueRef::BackgroundRepeat(v) => {
            v.serialize_specified(Default::default()).unwrap()
        }
        CssLonghandValueRef::BackgroundOrigin(v) | CssLonghandValueRef::BackgroundClip(v) => {
            v.serialize_specified(Default::default()).unwrap()
        }
        CssLonghandValueRef::BackgroundAttachment(v) => {
            v.serialize_specified(Default::default()).unwrap()
        }
        _ => panic!("selected background layer value"),
    }
}

#[test]
fn each_intrinsic_initial_has_its_source_defined_typed_single_layer() {
    for (property, expected) in [
        (CssKnownProperty::BackgroundSize, "auto"),
        (CssKnownProperty::BackgroundRepeat, "repeat"),
        (CssKnownProperty::BackgroundOrigin, "padding-box"),
        (CssKnownProperty::BackgroundClip, "border-box"),
        (CssKnownProperty::BackgroundAttachment, "scroll"),
    ] {
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("longhand")
        };
        assert!(!metadata.inherited_by_default());
        let initial = metadata.initial_value();
        assert_eq!(initial.property().known_property(), property);
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("intrinsic initial")
        };
        match value.view() {
            CssLonghandValueRef::BackgroundSize(v) => assert!(matches!(
                v.sizes(),
                [CssBackgroundSize::Explicit {
                    width: CssAutoOr::Auto,
                    height: None
                }]
            )),
            CssLonghandValueRef::BackgroundRepeat(v) => assert_eq!(
                v.repeats(),
                &[CssBackgroundRepeat::Axes {
                    x: R::Repeat,
                    y: R::Repeat
                },]
            ),
            CssLonghandValueRef::BackgroundOrigin(v) => {
                assert_eq!(v.boxes(), &[CssBackgroundBox::PaddingBox])
            }
            CssLonghandValueRef::BackgroundClip(v) => {
                assert_eq!(v.boxes(), &[CssBackgroundBox::BorderBox])
            }
            CssLonghandValueRef::BackgroundAttachment(v) => {
                assert_eq!(v.attachments(), &[CssBackgroundAttachment::Scroll])
            }
            _ => panic!("selected initial"),
        }
        assert_eq!(text(value.view()), expected);
    }
}

#[test]
fn ordinary_contributions_preserve_typed_list_order_and_parsed_numeric_origin() {
    for (property, css, expected) in [
        (
            CssKnownProperty::BackgroundSize,
            "cover, 10px auto",
            "cover, 10px auto",
        ),
        (
            CssKnownProperty::BackgroundRepeat,
            "repeat-x, space round",
            "repeat-x, space round",
        ),
        (
            CssKnownProperty::BackgroundOrigin,
            "border-box, content-box",
            "border-box, content-box",
        ),
        (
            CssKnownProperty::BackgroundClip,
            "content-box, padding-box",
            "content-box, padding-box",
        ),
        (
            CssKnownProperty::BackgroundAttachment,
            "fixed, local",
            "fixed, local",
        ),
    ] {
        let source = declaration(&format!("{}:{css}!important", property.canonical_name()));
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("completed ordinary")
        };
        let [item] = items.items() else {
            panic!("one terminal list")
        };
        assert_eq!(item.property(), property);
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        let value = item.ordinary_value().unwrap().view();
        match value {
            CssLonghandValueRef::BackgroundSize(v) => {
                let expected = CssBackgroundSizeList::try_new(vec![
                    CssBackgroundSize::Cover,
                    CssBackgroundSize::Explicit {
                        width: CssAutoOr::Value(length("10px")),
                        height: Some(CssAutoOr::Auto),
                    },
                ])
                .unwrap();
                assert_eq!(v, &expected);
                let CssBackgroundSize::Explicit {
                    width: CssAutoOr::Value(width),
                    ..
                } = &v.sizes()[1]
                else {
                    panic!("numeric width")
                };
                assert!(matches!(width.origin(), CssValueOrigin::Parsed(_)));
            }
            CssLonghandValueRef::BackgroundRepeat(v) => assert_eq!(
                v.repeats(),
                &[
                    CssBackgroundRepeat::RepeatX,
                    CssBackgroundRepeat::Axes {
                        x: R::Space,
                        y: R::Round
                    },
                ]
            ),
            CssLonghandValueRef::BackgroundOrigin(v) => assert_eq!(
                v.boxes(),
                &[CssBackgroundBox::BorderBox, CssBackgroundBox::ContentBox]
            ),
            CssLonghandValueRef::BackgroundClip(v) => assert_eq!(
                v.boxes(),
                &[CssBackgroundBox::ContentBox, CssBackgroundBox::PaddingBox]
            ),
            CssLonghandValueRef::BackgroundAttachment(v) => assert_eq!(
                v.attachments(),
                &[
                    CssBackgroundAttachment::Fixed,
                    CssBackgroundAttachment::Local
                ]
            ),
            _ => panic!("selected typed contribution"),
        }
        assert_eq!(text(value), expected);
    }
}

#[test]
fn strict_reentry_retains_programmatic_components_and_expected_typed_values() {
    for (property, keyword) in [
        (CssKnownProperty::BackgroundSize, "auto"),
        (CssKnownProperty::BackgroundRepeat, "repeat"),
        (CssKnownProperty::BackgroundOrigin, "content-box"),
        (CssKnownProperty::BackgroundClip, "padding-box"),
        (CssKnownProperty::BackgroundAttachment, "local"),
    ] {
        let source = declaration(&format!(
            "{}:var(--layers)!important",
            property.canonical_name()
        ));
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        let replacement = CssComponentValues::try_new(
            vec![CssComponentValue::try_ident(keyword).unwrap()],
            Default::default(),
        )
        .unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed replacement")
            };
            let [item] = items.items() else {
                panic!("one terminal")
            };
            assert_eq!(item.property(), property);
            assert_eq!(text(item.ordinary_value().unwrap().view()), keyword);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            assert!(matches!(
                item.replacement_components().unwrap().items()[0].origin(),
                CssValueOrigin::Programmatic
            ));
        }
    }
    let source = declaration("background-size:var(--size)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending size")
    };
    let replacement = CssComponentValues::try_new(
        vec![CssComponentValue::try_token("25%").unwrap()],
        Default::default(),
    )
    .unwrap();
    let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("numeric replacement")
    };
    let CssLonghandValueRef::BackgroundSize(v) = items.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("typed replacement size")
    };
    let [
        CssBackgroundSize::Explicit {
            width: CssAutoOr::Value(width),
            height: None,
        },
    ] = v.sizes()
    else {
        panic!("one omitted-height size")
    };
    assert!(matches!(width.origin(), CssValueOrigin::Programmatic));
    assert!(
        matches!(width.literal().map(|literal| literal.component()).unwrap().view(), CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
        if number.representation() == "25")
    );
}

#[test]
fn specified_sizes_preserve_effective_auto_height_and_symbolic_values() {
    for (css, expected) in [
        ("AUTO AUTO", "auto"),
        ("auto", "auto"),
        ("10px auto", "10px auto"),
        ("10px 20%", "10px 20%"),
        ("auto 20%", "auto 20%"),
        ("cover, contain, 50% auto", "cover, contain, 50% auto"),
        ("calc(10px + 5%) auto", "calc(5% + 10px) auto"),
        ("calc(10px - 5%)", "calc(-5% + 10px) auto"),
    ] {
        let value = size(css);
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
        for layer in value.sizes() {
            assert!(layer.serialize_specified(Default::default()).is_ok());
        }
        // Canonical auto collapse/math reordering need not preserve authored AST Eq.
        assert_eq!(
            size(expected)
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
    }
    let value = size("calc(10px - 5%)");
    let [
        CssBackgroundSize::Explicit {
            width: CssAutoOr::Value(width),
            height: None,
        },
    ] = value.sizes()
    else {
        panic!("symbolic omitted-height size")
    };
    assert!(width.calculation().is_some());
    assert!(width.literal().map(|literal| literal.component()).is_none());
}

#[test]
fn repeat_aliases_equal_axes_and_directional_pairs_use_shorter_equivalents() {
    for (value, expected) in [
        (CssBackgroundRepeat::RepeatX, "repeat-x"),
        (CssBackgroundRepeat::RepeatY, "repeat-y"),
        (
            CssBackgroundRepeat::Axes {
                x: R::Repeat,
                y: R::Repeat,
            },
            "repeat",
        ),
        (
            CssBackgroundRepeat::Axes {
                x: R::Space,
                y: R::Space,
            },
            "space",
        ),
        (
            CssBackgroundRepeat::Axes {
                x: R::Round,
                y: R::Round,
            },
            "round",
        ),
        (
            CssBackgroundRepeat::Axes {
                x: R::NoRepeat,
                y: R::NoRepeat,
            },
            "no-repeat",
        ),
        (
            CssBackgroundRepeat::Axes {
                x: R::Repeat,
                y: R::NoRepeat,
            },
            "repeat-x",
        ),
        (
            CssBackgroundRepeat::Axes {
                x: R::NoRepeat,
                y: R::Repeat,
            },
            "repeat-y",
        ),
        (
            CssBackgroundRepeat::Axes {
                x: R::Space,
                y: R::Round,
            },
            "space round",
        ),
    ] {
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
        let list = CssBackgroundRepeatList::try_new(vec![value]).unwrap();
        assert_eq!(
            list.serialize_specified(Default::default()).unwrap(),
            expected
        );
        let parsed = declaration(&format!("background-repeat:{expected}"));
        let CssKnownPropertyValueRef::BackgroundRepeat(reparsed) =
            parsed.known().unwrap().property_value().unwrap()
        else {
            panic!("repeat roundtrip")
        };
        assert_eq!(
            reparsed
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
    }
}

#[test]
fn scalar_budgets_account_for_authored_alias_axes_and_omitted_auto_components() {
    for (value, expected, nodes) in [
        (CssBackgroundRepeat::RepeatX, "repeat-x", 1),
        (
            CssBackgroundRepeat::Axes {
                x: R::Repeat,
                y: R::NoRepeat,
            },
            "repeat-x",
            3,
        ),
        (
            CssBackgroundRepeat::Axes {
                x: R::Repeat,
                y: R::Repeat,
            },
            "repeat",
            3,
        ),
    ] {
        let original = value;
        exact_bounds(expected, nodes, nodes, |limits| {
            value.serialize_specified(limits)
        });
        assert_eq!(value, original);
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
    }
    for (value, expected, nodes) in [
        (
            CssBackgroundSize::Explicit {
                width: CssAutoOr::Auto,
                height: None,
            },
            "auto",
            2,
        ),
        (
            CssBackgroundSize::Explicit {
                width: CssAutoOr::Auto,
                height: Some(CssAutoOr::Auto),
            },
            "auto",
            3,
        ),
        (
            CssBackgroundSize::Explicit {
                width: CssAutoOr::Value(length("10px")),
                height: Some(CssAutoOr::Auto),
            },
            "10px auto",
            4,
        ),
    ] {
        let original = value.clone();
        exact_bounds(expected, nodes, nodes, |limits| {
            value.serialize_specified(limits)
        });
        assert_eq!(value, original);
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
    }
    exact_bounds("padding-box", 1, 1, |limits| {
        CssBackgroundBox::PaddingBox.serialize_specified(limits)
    });
    exact_bounds("scroll", 1, 1, |limits| {
        CssBackgroundAttachment::Scroll.serialize_specified(limits)
    });
}

#[test]
fn list_budgets_cover_all_owned_layers_numeric_children_and_separators() {
    let sizes = size("cover, 10px auto");
    let original = sizes.clone();
    // List + cover + explicit size + width component/literal + explicit auto.
    exact_bounds("cover, 10px auto", 6, 6, |limits| {
        sizes.serialize_specified(limits)
    });
    assert_eq!(sizes, original);
    let repeats = CssBackgroundRepeatList::try_new(vec![
        CssBackgroundRepeat::RepeatX,
        CssBackgroundRepeat::Axes {
            x: R::Space,
            y: R::Round,
        },
    ])
    .unwrap();
    exact_bounds("repeat-x, space round", 5, 5, |limits| {
        repeats.serialize_specified(limits)
    });
    let boxes = CssBackgroundBoxList::try_new(vec![
        CssBackgroundBox::ContentBox,
        CssBackgroundBox::BorderBox,
    ])
    .unwrap();
    exact_bounds("content-box, border-box", 3, 3, |limits| {
        boxes.serialize_specified(limits)
    });
    let attachments = CssBackgroundAttachmentList::try_new(vec![
        CssBackgroundAttachment::Local,
        CssBackgroundAttachment::Fixed,
    ])
    .unwrap();
    exact_bounds("local, fixed", 3, 3, |limits| {
        attachments.serialize_specified(limits)
    });
    assert!(CssBackgroundSizeList::try_new(Vec::new()).is_none());
    assert!(CssBackgroundRepeatList::try_new(Vec::new()).is_none());
    assert!(CssBackgroundBoxList::try_new(Vec::new()).is_none());
    assert!(CssBackgroundAttachmentList::try_new(Vec::new()).is_none());
}

#[test]
fn symbolic_size_projection_uses_one_monotonic_budget_across_layers() {
    let sizes = size("calc(1px + 2em), calc(1px + 2em)");
    let expected = "calc(2em + 1px) auto, calc(2em + 1px) auto";
    // One list plus two explicit size/component/math subgraphs. Each math
    // contributes four input nodes and five projection nodes, and each generated
    // auto adds one projection node without adding authored input.
    exact_bounds(expected, 13, 17, |limits| sizes.serialize_specified(limits));
    let original = sizes.clone();
    assert_eq!(
        sizes.serialize_specified(Default::default()).unwrap(),
        expected
    );
    assert_eq!(sizes, original);
}
