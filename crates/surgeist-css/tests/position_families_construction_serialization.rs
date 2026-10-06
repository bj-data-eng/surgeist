#![forbid(unsafe_code)]
//! Functional evidence for the adopted Values 5 symbolic-position contract.
//! These wholly new models have no executable preimplementation RED. The oracle
//! is the adopted grammar, checked construction and cumulative resource contract,
//! with Cartesian horizontal/vertical and symbolic flow block/inline serialization.

use surgeist_css::{
    CssBlockPosition as B, CssHorizontalPosition as H, CssInlinePosition as I,
    CssPositionConstructionError as E, CssRelativeAxisPosition as R,
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L,
    CssVerticalPosition as V, *,
};

fn lp(text: &str) -> CssSpecifiedLengthPercentage {
    if text.contains('(') {
        CssSpecifiedLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::try_from_components(
                parse_component_values(text).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_token(text).unwrap(),
        )
        .unwrap()
    }
}
fn cartesian(horizontal: H, vertical: V) -> CssPosition {
    CssPosition::from_cartesian(CssCartesianPosition::try_new(horizontal, vertical).unwrap())
}
fn parsed(text: &str) -> CssPosition {
    let report = parse_style_attribute(&format!("/*😀*/ clip-path:circle(at {text})"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::ClipPath(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("clip-path")
    };
    let CssClipPath::BasicShape(value) = value.value() else {
        panic!("shape")
    };
    let CssBasicShape::Circle(value) = value.shape() else {
        panic!("circle")
    };
    value.position().unwrap().clone()
}
fn budget(
    expected: &str,
    input: usize,
    projection: usize,
    serialize: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    assert_eq!(
        serialize(L::new(input, projection, expected.len())).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            L::new(input - 1, projection, expected.len()),
            K::InputNodeLimit,
        ),
        (
            L::new(input, projection - 1, expected.len()),
            K::ProjectionNodeLimit,
        ),
        (L::new(input, projection, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), kind, "{expected}");
    }
    assert_eq!(serialize(L::default()).unwrap(), expected);
}

#[test]
fn cartesian_payload_getters_preserve_physical_and_axis_relative_identity() {
    for horizontal in [H::Left, H::Center, H::Right, H::XStart, H::XEnd] {
        for vertical in [V::Top, V::Center, V::Bottom, V::YStart, V::YEnd] {
            let payload =
                CssCartesianPosition::try_new(horizontal.clone(), vertical.clone()).unwrap();
            assert_eq!(payload.horizontal(), &horizontal);
            assert_eq!(payload.vertical(), &vertical);
            let position = CssPosition::from_cartesian(payload.clone());
            let CssPositionRef::Cartesian(view) = position.view() else {
                panic!("Cartesian")
            };
            assert_eq!(view, &payload);
        }
    }
    for horizontal in [
        H::LeftOffset(lp("-1e-999px")),
        H::RightOffset(lp("-1e-999px")),
        H::XStartOffset(lp("-1e-999px")),
        H::XEndOffset(lp("-1e-999px")),
    ] {
        for vertical in [
            V::TopOffset(lp("10%")),
            V::BottomOffset(lp("10%")),
            V::YStartOffset(lp("10%")),
            V::YEndOffset(lp("10%")),
        ] {
            let payload =
                CssCartesianPosition::try_new(horizontal.clone(), vertical.clone()).unwrap();
            assert_eq!(payload.horizontal(), &horizontal);
            assert_eq!(payload.vertical(), &vertical);
        }
    }
    let free = CssCartesianPosition::try_new(H::Offset(lp("-2px")), V::Offset(lp("25%"))).unwrap();
    assert_eq!(free.horizontal(), &H::Offset(lp("-2px")));
    assert_eq!(free.vertical(), &V::Offset(lp("25%")));
    // The separately typed keyword enums expose the same axis distinction.
    assert_ne!(
        CssHorizontalPositionKeyword::XStart,
        CssHorizontalPositionKeyword::XEnd
    );
    assert_ne!(
        CssVerticalPositionKeyword::YStart,
        CssVerticalPositionKeyword::YEnd
    );
}

#[test]
fn flow_payload_getters_preserve_noncentral_families_and_axis_order() {
    for block in [B::Center, B::Start, B::End] {
        for inline in [I::Center, I::Start, I::End] {
            let position = CssPosition::try_from_named_axes(block.clone(), inline.clone()).unwrap();
            if block == B::Center && inline == I::Center {
                continue;
            }
            let CssPositionRef::NamedFlow(view) = position.view() else {
                panic!("named flow")
            };
            assert_eq!(view.block(), &block);
            assert_eq!(view.inline(), &inline);
        }
    }
    for block in [R::Center, R::Start, R::End] {
        for inline in [R::Center, R::Start, R::End] {
            let position =
                CssPosition::try_from_relative_axes(block.clone(), inline.clone()).unwrap();
            if block == R::Center && inline == R::Center {
                continue;
            }
            let CssPositionRef::RelativeFlow(view) = position.view() else {
                panic!("relative flow")
            };
            assert_eq!(view.block(), &block);
            assert_eq!(view.inline(), &inline);
        }
    }
    for block in [B::StartOffset(lp("-2px")), B::EndOffset(lp("-2px"))] {
        for inline in [I::StartOffset(lp("10%")), I::EndOffset(lp("10%"))] {
            let position = CssPosition::try_from_named_axes(block.clone(), inline.clone()).unwrap();
            let CssPositionRef::NamedFlow(view) = position.view() else {
                panic!("named flow")
            };
            assert_eq!(view.block(), &block);
            assert_eq!(view.inline(), &inline);
        }
    }
    for block in [R::StartOffset(lp("-2px")), R::EndOffset(lp("-2px"))] {
        for inline in [R::StartOffset(lp("10%")), R::EndOffset(lp("10%"))] {
            let position =
                CssPosition::try_from_relative_axes(block.clone(), inline.clone()).unwrap();
            let CssPositionRef::RelativeFlow(view) = position.view() else {
                panic!("relative flow")
            };
            assert_eq!(view.block(), &block);
            assert_eq!(view.inline(), &inline);
        }
    }
}

#[test]
fn all_center_owning_paths_have_one_cartesian_value() {
    let canonical = cartesian(H::Center, V::Center);
    for value in [
        canonical.clone(),
        CssPhysicalPosition::try_new(H::Center, V::Center)
            .unwrap()
            .into(),
        CssPosition::try_from_named_axes(B::Center, I::Center).unwrap(),
        CssPosition::try_from_relative_axes(R::Center, R::Center).unwrap(),
        parsed("center"),
        parsed("center center"),
    ] {
        assert_eq!(value, canonical);
        let CssPositionRef::Cartesian(view) = value.view() else {
            panic!("canonical Cartesian")
        };
        assert_eq!(view.horizontal(), &H::Center);
        assert_eq!(view.vertical(), &V::Center);
    }
    assert_ne!(parsed("start center"), cartesian(H::Left, V::Center));
    assert_ne!(parsed("block-start center"), parsed("start center"));
    assert_ne!(parsed("start end"), parsed("end start"));
}

#[test]
fn every_checked_generic_family_rejects_unpaired_edge_offsets_with_typed_error() {
    for (horizontal, vertical) in [
        (H::XStartOffset(lp("1px")), V::Center),
        (H::Right, V::YEndOffset(lp("2%"))),
        (H::Offset(lp("1px")), V::BottomOffset(lp("2%"))),
        (H::LeftOffset(lp("1px")), V::Offset(lp("2%"))),
    ] {
        assert_eq!(
            CssCartesianPosition::try_new(horizontal, vertical).unwrap_err(),
            E::UnpairedEdgeOffsets
        );
    }
    for (block, inline) in [
        (B::StartOffset(lp("1px")), I::Center),
        (B::EndOffset(lp("1px")), I::End),
        (B::Start, I::StartOffset(lp("2%"))),
        (B::Center, I::EndOffset(lp("2%"))),
    ] {
        assert_eq!(
            CssPosition::try_from_named_axes(block, inline).unwrap_err(),
            E::UnpairedEdgeOffsets
        );
    }
    for (block, inline) in [
        (R::StartOffset(lp("1px")), R::Center),
        (R::EndOffset(lp("1px")), R::Start),
        (R::End, R::StartOffset(lp("2%"))),
        (R::Center, R::EndOffset(lp("2%"))),
    ] {
        assert_eq!(
            CssPosition::try_from_relative_axes(block, inline).unwrap_err(),
            E::UnpairedEdgeOffsets
        );
    }
    let error: &dyn std::error::Error = &E::UnpairedEdgeOffsets;
    assert_eq!(error.to_string(), "position edge offsets must be paired");
    assert!(error.source().is_none());
    assert_eq!(
        E::NonPhysicalKeyword.to_string(),
        "physical position cannot contain axis-relative keywords"
    );
}

#[test]
fn physical_and_background_construction_reject_every_new_cartesian_branch() {
    for horizontal in [H::XStart, H::XEnd] {
        assert_eq!(
            CssPhysicalPosition::try_new(horizontal.clone(), V::Top).unwrap_err(),
            E::NonPhysicalKeyword
        );
        assert!(CssBackgroundPosition::try_new(horizontal, V::Top).is_none());
    }
    for vertical in [V::YStart, V::YEnd] {
        assert_eq!(
            CssPhysicalPosition::try_new(H::Left, vertical.clone()).unwrap_err(),
            E::NonPhysicalKeyword
        );
        assert!(CssBackgroundPosition::try_new(H::Left, vertical).is_none());
    }
    for horizontal in [H::XStartOffset(lp("1px")), H::XEndOffset(lp("1px"))] {
        assert_eq!(
            CssPhysicalPosition::try_new(horizontal.clone(), V::TopOffset(lp("2%"))).unwrap_err(),
            E::NonPhysicalKeyword
        );
        assert!(
            CssBackgroundPosition::try_new(horizontal.clone(), V::TopOffset(lp("2%"))).is_none()
        );
        assert!(CssBackgroundPosition::try_new(horizontal, V::Top).is_none());
    }
    for vertical in [V::YStartOffset(lp("2%")), V::YEndOffset(lp("2%"))] {
        assert_eq!(
            CssPhysicalPosition::try_new(H::LeftOffset(lp("1px")), vertical.clone()).unwrap_err(),
            E::NonPhysicalKeyword
        );
        assert!(
            CssBackgroundPosition::try_new(H::LeftOffset(lp("1px")), vertical.clone()).is_none()
        );
        assert!(CssBackgroundPosition::try_new(H::Left, vertical).is_none());
    }
    assert_eq!(
        CssPhysicalPosition::try_new(H::LeftOffset(lp("1px")), V::Center).unwrap_err(),
        E::UnpairedEdgeOffsets
    );
    let background = CssBackgroundPosition::try_new(H::LeftOffset(lp("1px")), V::Center).unwrap();
    assert_eq!(background.horizontal(), &H::LeftOffset(lp("1px")));
    assert_eq!(background.vertical(), &V::Center);
    assert!(
        CssBackgroundPosition::try_new(H::LeftOffset(lp("1px")), V::Offset(lp("2%"))).is_none()
    );
    assert!(CssBackgroundPosition::try_new(H::Offset(lp("1px")), V::TopOffset(lp("2%"))).is_none());
}

#[test]
fn restriction_lifting_and_restricted_consumers_keep_one_physical_payload() {
    let physical = CssPhysicalPosition::try_new(H::Right, V::Bottom).unwrap();
    let lifted: CssPosition = physical.clone().into();
    let CssPositionRef::Cartesian(view) = lifted.view() else {
        panic!("Cartesian lift")
    };
    assert_eq!(view.horizontal(), physical.horizontal());
    assert_eq!(view.vertical(), physical.vertical());
    let layer =
        CssMaskLayer::try_new(None, Some(physical.clone()), None, None, None, None, None).unwrap();
    assert_eq!(layer.position(), Some(&physical));
    let list = CssPhysicalPositionList::try_new(vec![physical.clone()]).unwrap();
    assert_eq!(list.positions(), std::slice::from_ref(&physical));
    assert!(CssPhysicalPositionList::try_new(Vec::new()).is_none());
    budget("right bottom", 4, 4, |limits| {
        list.serialize_specified_with_limits(limits)
    });
    let layers = CssPhysicalPositionList::try_new(vec![
        physical.clone(),
        CssPhysicalPosition::try_new(H::Left, V::Top).unwrap(),
    ])
    .unwrap();
    budget("right bottom, left top", 7, 7, |limits| {
        layers.serialize_specified_with_limits(limits)
    });
    let origin = CssTransformOrigin::try_new(physical.clone(), None).unwrap();
    assert_eq!(origin.horizontal(), physical.horizontal());
    assert_eq!(origin.vertical(), physical.vertical());
    assert!(origin.z().is_none());
    let z = CssSpecifiedLength::try_from_component(CssComponentValue::try_token("-3px").unwrap())
        .unwrap();
    let origin = CssTransformOrigin::try_new(physical, Some(z.clone())).unwrap();
    assert_eq!(origin.z(), Some(&z));
    let paired =
        CssPhysicalPosition::try_new(H::RightOffset(lp("1px")), V::BottomOffset(lp("2%"))).unwrap();
    assert!(CssTransformOrigin::try_new(paired, None).is_none());
}

#[test]
fn specified_order_preserves_keyword_family_and_serializes_implied_center() {
    for (authored, expected) in [
        ("center", "center center"),
        ("x-start", "x-start center"),
        ("x-end", "x-end center"),
        ("y-start", "center y-start"),
        ("y-end", "center y-end"),
        ("y-start right", "right y-start"),
        ("bottom x-end", "x-end bottom"),
        ("x-start 10px", "x-start 10px"),
        ("-2% y-end", "-2% y-end"),
        ("y-start 2% x-end -1px", "x-end -1px y-start 2%"),
        ("block-start", "block-start center"),
        ("block-end", "block-end center"),
        ("inline-start", "center inline-start"),
        ("inline-end", "center inline-end"),
        ("inline-end block-start", "block-start inline-end"),
        ("center block-start", "block-start center"),
        ("center inline-start", "center inline-start"),
        ("start start", "start start"),
        ("start center", "start center"),
        ("start end", "start end"),
        ("center start", "center start"),
        ("center end", "center end"),
        ("end start", "end start"),
        ("end center", "end center"),
        ("end end", "end end"),
        (
            "inline-end -2px block-start 10%",
            "block-start 10% inline-end -2px",
        ),
        ("end -2px start 10%", "end -2px start 10%"),
    ] {
        let position = parsed(authored);
        assert_eq!(
            position.serialize_specified().unwrap(),
            expected,
            "{authored}"
        );
        assert_eq!(parsed(expected), position, "{authored}");
    }
}

#[test]
fn parsed_and_programmatic_families_compare_numeric_structure_and_preserve_origins() {
    let values = [
        (
            "y-end 10% x-start -1e-999px",
            cartesian(H::XStartOffset(lp("-1e-999px")), V::YEndOffset(lp("10%"))),
        ),
        (
            "inline-end -1e-999px block-start 10%",
            CssPosition::try_from_named_axes(
                B::StartOffset(lp("10%")),
                I::EndOffset(lp("-1e-999px")),
            )
            .unwrap(),
        ),
        (
            "start 10% end -1e-999px",
            CssPosition::try_from_relative_axes(
                R::StartOffset(lp("10%")),
                R::EndOffset(lp("-1e-999px")),
            )
            .unwrap(),
        ),
    ];
    for (source, constructed) in values {
        let value = parsed(source);
        assert_eq!(value, constructed);
        let (parsed_scalar, constructed_scalar) = match (value.view(), constructed.view()) {
            (CssPositionRef::Cartesian(a), CssPositionRef::Cartesian(b)) => {
                let (H::XStartOffset(a), H::XStartOffset(b)) = (a.horizontal(), b.horizontal())
                else {
                    panic!("x-start offsets")
                };
                (a, b)
            }
            (CssPositionRef::NamedFlow(a), CssPositionRef::NamedFlow(b)) => {
                let (I::EndOffset(a), I::EndOffset(b)) = (a.inline(), b.inline()) else {
                    panic!("inline-end offsets")
                };
                (a, b)
            }
            (CssPositionRef::RelativeFlow(a), CssPositionRef::RelativeFlow(b)) => {
                let (R::EndOffset(a), R::EndOffset(b)) = (a.inline(), b.inline()) else {
                    panic!("relative-end offsets")
                };
                (a, b)
            }
            _ => panic!("same family"),
        };
        let CssValueOrigin::Parsed(origin) = parsed_scalar.origin() else {
            panic!("parsed numeric origin")
        };
        let start = origin.source().as_str().find("-1e-999px").unwrap();
        assert_eq!(origin.span().start().byte_offset().value(), start);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            start + "-1e-999px".len()
        );
        assert_eq!(
            origin.span().start().column().value() as usize,
            origin.source().as_str()[..start].encode_utf16().count()
        );
        assert_eq!(constructed_scalar.origin(), &CssValueOrigin::Programmatic);
        assert_ne!(parsed_scalar, constructed_scalar);
        assert!(matches!(parsed_scalar.literal_component().unwrap().view(),
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
                if number.representation() == "-1e-999" && unit == "px"));
    }
    assert_ne!(
        parsed("x-start 1.0px y-end 2%"),
        parsed("x-start 1px y-end 2%")
    );
    assert_ne!(
        parsed("block-start 1px inline-end 2%"),
        parsed("block-end 1px inline-end 2%")
    );
    assert_ne!(parsed("start 1px end 2%"), parsed("end 1px start 2%"));
    assert_eq!(
        parsed("start calc(1px + 2em) end 3%"),
        CssPosition::try_from_relative_axes(
            R::StartOffset(lp("calc(1px + 2em)")),
            R::EndOffset(lp("3%"))
        )
        .unwrap()
    );
    assert_ne!(
        parsed("start calc(1px + 2em) end 3%"),
        parsed("start calc(2em + 1px) end 3%")
    );
}

#[test]
fn every_family_charges_one_aggregate_two_axes_and_authored_scalars() {
    for (position, expected) in [
        (cartesian(H::XStart, V::YEnd), "x-start y-end"),
        (
            CssPosition::try_from_named_axes(B::Start, I::End).unwrap(),
            "block-start inline-end",
        ),
        (
            CssPosition::try_from_relative_axes(R::Start, R::End).unwrap(),
            "start end",
        ),
    ] {
        let before = position.clone();
        budget(expected, 3, 3, |limits| {
            position.serialize_specified_with_limits(limits)
        });
        assert_eq!(position, before);
    }
    for (position, expected) in [
        (
            cartesian(H::XStartOffset(lp("-2px")), V::BottomOffset(lp("10%"))),
            "x-start -2px bottom 10%",
        ),
        (
            CssPosition::try_from_named_axes(B::EndOffset(lp("-2px")), I::StartOffset(lp("10%")))
                .unwrap(),
            "block-end -2px inline-start 10%",
        ),
        (
            CssPosition::try_from_relative_axes(
                R::EndOffset(lp("-2px")),
                R::StartOffset(lp("10%")),
            )
            .unwrap(),
            "end -2px start 10%",
        ),
    ] {
        let before = position.clone();
        budget(expected, 5, 5, |limits| {
            position.serialize_specified_with_limits(limits)
        });
        assert_eq!(position, before);
    }
    let physical = CssPhysicalPosition::try_new(H::Right, V::Bottom).unwrap();
    budget("right bottom", 3, 3, |limits| {
        physical.serialize_specified_with_limits(limits)
    });
    let full: CssPosition = physical.into();
    budget("right bottom", 3, 3, |limits| {
        full.serialize_specified_with_limits(limits)
    });
    let background = CssBackgroundPosition::try_new(H::RightOffset(lp("2%")), V::Top).unwrap();
    budget("right 2% top", 4, 4, |limits| {
        background.serialize_specified_with_limits(limits)
    });
}

#[test]
fn shape_composition_charges_no_extra_family_or_view_nodes() {
    let position = CssPosition::try_from_relative_axes(R::Start, R::End).unwrap();
    let circle = CssCircleShape::new(CssCircleRadius::Default, Some(position.clone()));
    budget("circle(at start end)", 4, 4, |limits| {
        circle.serialize_specified_with_limits(limits)
    });
    let ellipse = CssEllipseShape::new(None, Some(position));
    budget("ellipse(at start end)", 4, 4, |limits| {
        ellipse.serialize_specified_with_limits(limits)
    });
    let clip = CssClipPath::BasicShape(CssClipPathShape::new(
        CssBasicShape::Circle(circle.clone()),
        None,
    ));
    budget("circle(at start end)", 5, 5, |limits| {
        clip.serialize_specified_with_limits(limits)
    });
    let boxed = CssClipPath::BasicShape(CssClipPathShape::new(
        CssBasicShape::Circle(circle),
        Some(CssBoxEdgeKeyword::BorderBox),
    ));
    budget("circle(at start end) border-box", 6, 6, |limits| {
        boxed.serialize_specified_with_limits(limits)
    });
    let omitted = CssCircleShape::new(CssCircleRadius::Default, None);
    budget("circle()", 1, 1, |limits| {
        omitted.serialize_specified_with_limits(limits)
    });
}

#[test]
fn math_arenas_are_cumulative_across_both_flow_axes_and_shape_composition() {
    // Each sum has four input nodes and five projected nodes. Position aggregate
    // and its two axes add three; no wrapper or discriminator adds a node.
    for (position, expected) in [
        (
            CssPosition::try_from_named_axes(
                B::StartOffset(lp("calc(1px + 2em)")),
                I::EndOffset(lp("calc(1px + 2em)")),
            )
            .unwrap(),
            "block-start calc(2em + 1px) inline-end calc(2em + 1px)",
        ),
        (
            CssPosition::try_from_relative_axes(
                R::EndOffset(lp("calc(1px + 2em)")),
                R::StartOffset(lp("calc(1px + 2em)")),
            )
            .unwrap(),
            "end calc(2em + 1px) start calc(2em + 1px)",
        ),
    ] {
        let before = position.clone();
        budget(expected, 11, 13, |limits| {
            position.serialize_specified_with_limits(limits)
        });
        assert_eq!(position, before);
        let circle = CssCircleShape::new(CssCircleRadius::Default, Some(position));
        budget(&format!("circle(at {expected})"), 12, 14, |limits| {
            circle.serialize_specified_with_limits(limits)
        });
    }
}

#[test]
fn mixed_math_offsets_keep_family_order_and_original_numeric_provenance() {
    for (source, expected, constructed) in [
        (
            "y-end -2px x-start calc(10% - 1px)",
            "x-start calc(10% - 1px) y-end -2px",
            cartesian(
                H::XStartOffset(lp("calc(10% - 1px)")),
                V::YEndOffset(lp("-2px")),
            ),
        ),
        (
            "inline-end -2px block-start calc(10% - 1px)",
            "block-start calc(10% - 1px) inline-end -2px",
            CssPosition::try_from_named_axes(
                B::StartOffset(lp("calc(10% - 1px)")),
                I::EndOffset(lp("-2px")),
            )
            .unwrap(),
        ),
        (
            "end calc(10% - 1px) start -2px",
            "end calc(10% - 1px) start -2px",
            CssPosition::try_from_relative_axes(
                R::EndOffset(lp("calc(10% - 1px)")),
                R::StartOffset(lp("-2px")),
            )
            .unwrap(),
        ),
    ] {
        let value = parsed(source);
        assert_eq!(value, constructed);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let offset = match value.view() {
            CssPositionRef::Cartesian(view) => {
                let H::XStartOffset(offset) = view.horizontal() else {
                    panic!("x math")
                };
                offset
            }
            CssPositionRef::NamedFlow(view) => {
                let B::StartOffset(offset) = view.block() else {
                    panic!("block math")
                };
                offset
            }
            CssPositionRef::RelativeFlow(view) => {
                let R::EndOffset(offset) = view.block() else {
                    panic!("relative math")
                };
                offset
            }
            _ => panic!("selected family"),
        };
        let before = offset.origin().clone();
        assert!(matches!(before, CssValueOrigin::Parsed(_)));
        let calculation = offset.calculation().unwrap();
        assert_eq!(
            calculation.components().serialize().unwrap().as_css(),
            "calc(10% - 1px)"
        );
        let CssComponentValueRef::Function(function) = calculation.components().items()[0].view()
        else {
            panic!("math function")
        };
        let CssValueOrigin::Parsed(closing) = function.closing_origin() else {
            panic!("authored math closure")
        };
        let end =
            closing.source().as_str().find("calc(10% - 1px)").unwrap() + "calc(10% - 1px)".len();
        assert_eq!(closing.span().end().byte_offset().value(), end);
        assert_eq!(
            closing.span().end().column().value() as usize,
            closing.source().as_str()[..end].encode_utf16().count()
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(0, 100, 100))
                .unwrap_err()
                .kind(),
            K::InputNodeLimit
        );
        assert_eq!(offset.origin(), &before);
    }
}

#[test]
fn older_consumer_parsers_reject_logical_families_and_recover_to_siblings() {
    for position in [
        "x-start y-end",
        "block-start inline-end",
        "start end",
        "x-start 1px y-end 2%",
        "block-start 1px inline-end 2%",
        "start 1px end 2%",
    ] {
        for declaration in [
            format!("object-position:{position}"),
            format!("mask-position:{position}"),
            format!("mask:url(mask.png) {position}"),
            format!("background-position:{position}"),
            format!("background:url(bg.png) {position}"),
            format!("transform-origin:{position}"),
            format!("background-image:radial-gradient(at {position},red,blue)"),
            format!("background-image:repeating-radial-gradient(at {position},red,blue)"),
        ] {
            let source = format!("{declaration};color:blue");
            let report = parse_style_attribute(&source);
            assert_eq!(report.diagnostics().len(), 1, "{source}");
            assert_eq!(report.syntax().len(), 1, "{source}");
            assert_eq!(
                report.syntax()[0].known().unwrap().property(),
                CssKnownProperty::Color
            );
            assert_eq!(
                report.diagnostics()[0].action(),
                CssRecoveryAction::DropDeclaration
            );
            assert_eq!(
                validate_style_attribute(&source).unwrap_err().diagnostics(),
                report.diagnostics()
            );
        }
    }
}
