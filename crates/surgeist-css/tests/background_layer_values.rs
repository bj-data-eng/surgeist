#![forbid(unsafe_code)]
//! Functional Backgrounds 3 list lifecycle and specified serialization.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/
//! CSSOM 1 §6.7.2 requires omission/replacement by shorter equivalent syntax.
//! https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-value

use surgeist_css::{
    CssBackgroundRepeatStyle as R, CssSpecifiedValueSerializationErrorKind as E,
    CssSpecifiedValueSerializationLimits as L, *,
};

fn declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    source.clone()
}

fn size(css: &str) -> CssBackgroundSizeList {
    let source = declaration(&format!("background-size:{css}"));
    let CssKnownPropertyValueRef::BackgroundSize(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("size list")
    };
    value.sizes().clone()
}

fn length(css: &str) -> CssSpecifiedNonNegativeLengthPercentage {
    CssSpecifiedNonNegativeLengthPercentage::try_from_component(
        CssComponentValue::try_token(css).unwrap(),
    )
    .unwrap()
}

fn text(value: CssLonghandValueRef<'_>) -> String {
    match value {
        CssLonghandValueRef::BackgroundSize(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BackgroundRepeat(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BackgroundOrigin(v) | CssLonghandValueRef::BackgroundClip(v) => {
            v.serialize_specified().unwrap()
        }
        CssLonghandValueRef::BackgroundAttachment(v) => v.serialize_specified().unwrap(),
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
                    width: CssBackgroundSizeComponent::Auto,
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
            "cover, 10px",
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
                        width: CssBackgroundSizeComponent::Length(length("10px")),
                        height: Some(CssBackgroundSizeComponent::Auto),
                    },
                ])
                .unwrap();
                assert_eq!(v, &expected);
                let CssBackgroundSize::Explicit {
                    width: CssBackgroundSizeComponent::Length(width),
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
        let replacement =
            CssComponentValues::try_new(vec![CssComponentValue::try_ident(keyword).unwrap()])
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
    let replacement =
        CssComponentValues::try_new(vec![CssComponentValue::try_token("25%").unwrap()]).unwrap();
    let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("numeric replacement")
    };
    let CssLonghandValueRef::BackgroundSize(v) = items.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("typed replacement size")
    };
    let [
        CssBackgroundSize::Explicit {
            width: CssBackgroundSizeComponent::Length(width),
            height: None,
        },
    ] = v.sizes()
    else {
        panic!("one omitted-height size")
    };
    assert!(matches!(width.origin(), CssValueOrigin::Programmatic));
    assert!(
        matches!(width.literal_component().unwrap().view(), CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
        if number.representation() == "25")
    );
}

#[test]
fn specified_sizes_omit_only_implied_auto_height_and_preserve_symbolic_values() {
    for (css, expected) in [
        ("AUTO AUTO", "auto"),
        ("auto", "auto"),
        ("10px auto", "10px"),
        ("10px 20%", "10px 20%"),
        ("auto 20%", "auto 20%"),
        ("cover, contain, 50% auto", "cover, contain, 50%"),
        ("calc(10px + 5%) auto", "calc(5% + 10px)"),
        ("calc(10px - 5%)", "calc(-5% + 10px)"),
    ] {
        let value = size(css);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        for layer in value.sizes() {
            assert!(layer.serialize_specified().is_ok());
        }
        // Canonical auto omission/math reordering need not preserve authored AST Eq.
        assert_eq!(size(expected).serialize_specified().unwrap(), expected);
    }
    let value = size("calc(10px - 5%)");
    let [
        CssBackgroundSize::Explicit {
            width: CssBackgroundSizeComponent::Length(width),
            height: None,
        },
    ] = value.sizes()
    else {
        panic!("symbolic omitted-height size")
    };
    assert!(width.calculation().is_some());
    assert!(width.literal_component().is_none());
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
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let list = CssBackgroundRepeatList::try_new(vec![value]).unwrap();
        assert_eq!(list.serialize_specified().unwrap(), expected);
        let parsed = declaration(&format!("background-repeat:{expected}"));
        let CssKnownPropertyValueRef::BackgroundRepeat(reparsed) =
            parsed.known().unwrap().property_value().unwrap()
        else {
            panic!("repeat roundtrip")
        };
        assert_eq!(reparsed.repeats().serialize_specified().unwrap(), expected);
    }
}

fn exact_bounds(
    expected: &str,
    input: usize,
    projection: usize,
    serialize: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    assert_eq!(
        serialize(L::new(input, projection, expected.len())).unwrap(),
        expected
    );
    for (limits, error) in [
        (
            L::new(input - 1, projection, expected.len()),
            E::InputNodeLimit,
        ),
        (
            L::new(input, projection - 1, expected.len()),
            E::ProjectionNodeLimit,
        ),
        (L::new(input, projection, expected.len() - 1), E::ByteLimit),
        (L::new(0, projection, expected.len()), E::InputNodeLimit),
        (L::new(input, 0, expected.len()), E::ProjectionNodeLimit),
        (L::new(input, projection, 0), E::ByteLimit),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), error);
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
            value.serialize_specified_with_limits(limits)
        });
        assert_eq!(value, original);
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
    for (value, expected, nodes) in [
        (
            CssBackgroundSize::Explicit {
                width: CssBackgroundSizeComponent::Auto,
                height: None,
            },
            "auto",
            2,
        ),
        (
            CssBackgroundSize::Explicit {
                width: CssBackgroundSizeComponent::Auto,
                height: Some(CssBackgroundSizeComponent::Auto),
            },
            "auto",
            3,
        ),
        (
            CssBackgroundSize::Explicit {
                width: CssBackgroundSizeComponent::Length(length("10px")),
                height: Some(CssBackgroundSizeComponent::Auto),
            },
            "10px",
            4,
        ),
    ] {
        let original = value.clone();
        exact_bounds(expected, nodes, nodes, |limits| {
            value.serialize_specified_with_limits(limits)
        });
        assert_eq!(value, original);
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
    exact_bounds("padding-box", 1, 1, |limits| {
        CssBackgroundBox::PaddingBox.serialize_specified_with_limits(limits)
    });
    exact_bounds("scroll", 1, 1, |limits| {
        CssBackgroundAttachment::Scroll.serialize_specified_with_limits(limits)
    });
}

#[test]
fn list_budgets_cover_all_owned_layers_numeric_children_and_separators() {
    let sizes = size("cover, 10px auto");
    let original = sizes.clone();
    // List + cover + explicit size + width component/literal + omitted auto.
    exact_bounds("cover, 10px", 6, 6, |limits| {
        sizes.serialize_specified_with_limits(limits)
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
        repeats.serialize_specified_with_limits(limits)
    });
    let boxes = CssBackgroundBoxList::try_new(vec![
        CssBackgroundBox::ContentBox,
        CssBackgroundBox::BorderBox,
    ])
    .unwrap();
    exact_bounds("content-box, border-box", 3, 3, |limits| {
        boxes.serialize_specified_with_limits(limits)
    });
    let attachments = CssBackgroundAttachmentList::try_new(vec![
        CssBackgroundAttachment::Local,
        CssBackgroundAttachment::Fixed,
    ])
    .unwrap();
    exact_bounds("local, fixed", 3, 3, |limits| {
        attachments.serialize_specified_with_limits(limits)
    });
    assert!(CssBackgroundSizeList::try_new(Vec::new()).is_none());
    assert!(CssBackgroundRepeatList::try_new(Vec::new()).is_none());
    assert!(CssBackgroundBoxList::try_new(Vec::new()).is_none());
    assert!(CssBackgroundAttachmentList::try_new(Vec::new()).is_none());
}

#[test]
fn symbolic_size_projection_uses_one_monotonic_budget_across_layers() {
    let sizes = size("calc(1px + 2em), calc(1px + 2em)");
    let expected = "calc(2em + 1px), calc(2em + 1px)";
    // One list plus two explicit size/component/math subgraphs. Each math
    // contributes four input nodes and five projection nodes.
    exact_bounds(expected, 13, 15, |limits| {
        sizes.serialize_specified_with_limits(limits)
    });
    let original = sizes.clone();
    assert_eq!(sizes.serialize_specified().unwrap(), expected);
    assert_eq!(sizes, original);
}
