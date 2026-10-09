#![forbid(unsafe_code)]
//! Functional new-model contracts from Motion WD 2024-11-05, Box 3 coord-box
//! and the imported full position, angle, URL and complete Shapes owners.
//! These wholly new typed APIs have no executable preimplementation boundary.
use CssSpecifiedValueSerializationErrorKind as K;
use CssSpecifiedValueSerializationLimits as L;
use surgeist_css::*;

fn angle(number: &str, unit: CssAngleUnit) -> CssAngleValue {
    CssAngleValue::from_literal(CssAngleLiteral::try_new(number, unit).unwrap())
}
fn position() -> CssPosition {
    CssPhysicalPosition::try_new(CssHorizontalPosition::Left, CssVerticalPosition::Top)
        .unwrap()
        .into()
}
fn declaration(name: &str, input: &str) -> CssDeclaration {
    let css = format!("/*😀*/{name}:{input}!important");
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}
fn offset_path(source: &CssDeclaration) -> &CssOffsetPath {
    let CssKnownPropertyValueRef::OffsetPath(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("path wrapper")
    };
    value.value()
}
fn ray(source: &CssDeclaration) -> &CssRay {
    let CssOffsetPathRef::Path(value) = offset_path(source).view() else {
        panic!("path pair")
    };
    let CssOffsetPathKind::Ray(value) = value.path() else {
        panic!("ray")
    };
    value
}
fn offset(source: &CssDeclaration) -> &CssOffset {
    let CssKnownPropertyValueRef::Offset(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("offset wrapper")
    };
    value.value()
}
fn contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(value)) =
        expand_declaration(source).unwrap()
    else {
        panic!("terminal contributions")
    };
    value
}
fn budget(
    expected: &str,
    nodes: usize,
    operation: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    assert_eq!(
        operation(L::new(nodes, nodes, expected.len())).unwrap(),
        expected
    );
    for (limits, kind) in [
        (L::new(nodes - 1, nodes, expected.len()), K::InputNodeLimit),
        (
            L::new(nodes, nodes - 1, expected.len()),
            K::ProjectionNodeLimit,
        ),
        (L::new(nodes, nodes, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(operation(limits).unwrap_err().kind(), kind);
    }
    assert_eq!(
        operation(L::new(nodes, nodes, expected.len())).unwrap(),
        expected
    );
}

#[test]
fn checked_coord_conversion_admits_six_boxes_and_rejects_margin() {
    for (edge, coord, keyword) in [
        (
            CssBoxEdgeKeyword::ContentBox,
            CssCoordBox::ContentBox,
            "content-box",
        ),
        (
            CssBoxEdgeKeyword::PaddingBox,
            CssCoordBox::PaddingBox,
            "padding-box",
        ),
        (
            CssBoxEdgeKeyword::BorderBox,
            CssCoordBox::BorderBox,
            "border-box",
        ),
        (CssBoxEdgeKeyword::FillBox, CssCoordBox::FillBox, "fill-box"),
        (
            CssBoxEdgeKeyword::StrokeBox,
            CssCoordBox::StrokeBox,
            "stroke-box",
        ),
        (CssBoxEdgeKeyword::ViewBox, CssCoordBox::ViewBox, "view-box"),
    ] {
        assert_eq!(CssCoordBox::try_from(edge), Ok(coord));
        assert_eq!(
            CssCoordBox::from_keyword(&keyword.to_ascii_uppercase()),
            Some(coord)
        );
        assert_eq!(coord.as_css_str(), keyword);
        budget(keyword, 1, |limits| {
            coord.serialize_specified_with_limits(limits)
        });
    }
    assert_eq!(
        CssCoordBox::try_from(CssBoxEdgeKeyword::MarginBox),
        Err(CssMotionConstructionError::InvalidCoordBox)
    );
    for keyword in ["margin-box", "text", "no-clip", "none"] {
        assert!(CssCoordBox::from_keyword(keyword).is_none());
    }
}

#[test]
fn ray_constructor_retains_required_bearing_and_optional_presence() {
    let bearing = angle("+450.000", CssAngleUnit::Degrees);
    let omitted = CssRay::new(bearing.clone(), None, false, None);
    assert_eq!(
        omitted
            .angle()
            .literal()
            .unwrap()
            .numeric()
            .representation(),
        "+450.000"
    );
    assert_eq!(omitted.angle().origin(), &CssValueOrigin::Programmatic);
    assert_eq!(omitted.size(), None);
    assert!(!omitted.contain());
    assert!(omitted.position().is_none());
    budget("ray(450deg)", 2, |limits| {
        omitted.serialize_specified_with_limits(limits)
    });
    let explicit = CssRay::new(bearing.clone(), Some(CssRaySize::ClosestSide), false, None);
    assert_ne!(explicit, omitted);
    assert_eq!(explicit.size(), Some(CssRaySize::ClosestSide));
    budget("ray(450deg)", 3, |limits| {
        explicit.serialize_specified_with_limits(limits)
    });
    let full = CssRay::new(bearing, Some(CssRaySize::Sides), true, Some(position()));
    let CssPositionRef::Cartesian(axes) = full.position().unwrap().view() else {
        panic!("Cartesian position")
    };
    assert_eq!(axes.horizontal(), &CssHorizontalPosition::Left);
    budget("ray(450deg sides contain at left top)", 7, |limits| {
        full.serialize_specified_with_limits(limits)
    });
}

#[test]
fn ray_sizes_have_independent_direct_keyword_and_retained_default_contracts() {
    for (size, text) in [
        (CssRaySize::ClosestSide, "closest-side"),
        (CssRaySize::ClosestCorner, "closest-corner"),
        (CssRaySize::FarthestSide, "farthest-side"),
        (CssRaySize::FarthestCorner, "farthest-corner"),
        (CssRaySize::Sides, "sides"),
    ] {
        assert_eq!(
            CssRaySize::from_keyword(&text.to_ascii_uppercase()),
            Some(size)
        );
        assert_eq!(size.as_css_str(), text);
        budget(text, 1, |limits| {
            size.serialize_specified_with_limits(limits)
        });
    }
    assert!(CssRaySize::from_keyword("contain").is_none());
    let source = declaration(
        "offset-path",
        "ray(at top left 100grad contain closest-side)",
    );
    let value = ray(&source);
    assert_eq!(value.size(), Some(CssRaySize::ClosestSide));
    assert!(value.contain());
    assert_eq!(
        value.angle().literal().unwrap().unit(),
        CssAngleUnit::Gradians
    );
    assert_eq!(
        value.angle().literal().unwrap().numeric().representation(),
        "100"
    );
    assert_eq!(value.position().unwrap(), &position());
    assert_eq!(
        value.serialize_specified().unwrap(),
        "ray(100grad contain at left top)"
    );
}

#[test]
fn rotation_rejects_empty_and_preserves_modifier_angle_omissions() {
    assert_eq!(
        CssOffsetRotate::try_new(None, None),
        Err(CssMotionConstructionError::EmptyRotate)
    );
    let literal = angle(".25", CssAngleUnit::Turns);
    let alone = CssOffsetRotate::try_new(None, Some(literal.clone())).unwrap();
    assert_eq!(alone.modifier(), None);
    assert_eq!(
        alone.angle().unwrap().origin(),
        &CssValueOrigin::Programmatic
    );
    budget("0.25turn", 2, |limits| {
        alone.serialize_specified_with_limits(limits)
    });
    let automatic = CssOffsetRotate::auto();
    assert_eq!(automatic.modifier(), Some(CssOffsetRotateModifier::Auto));
    assert!(automatic.angle().is_none());
    budget("auto", 2, |limits| {
        automatic.serialize_specified_with_limits(limits)
    });
    for (modifier, text) in [
        (CssOffsetRotateModifier::Auto, "auto"),
        (CssOffsetRotateModifier::Reverse, "reverse"),
    ] {
        budget(text, 1, |limits| {
            modifier.serialize_specified_with_limits(limits)
        });
        let both = CssOffsetRotate::try_new(Some(modifier), Some(literal.clone())).unwrap();
        assert_eq!(both.modifier(), Some(modifier));
        assert_eq!(
            both.angle()
                .unwrap()
                .literal()
                .unwrap()
                .numeric()
                .representation(),
            ".25"
        );
        assert_ne!(both, alone);
        budget(&format!("{text} 0.25turn"), 3, |limits| {
            both.serialize_specified_with_limits(limits)
        });
    }
}

#[test]
fn motion_angle_construction_uses_strict_existing_owner_errors_and_origins() {
    let component = parse_component_values("0").unwrap().items()[0].clone();
    let origin = component.origin().clone();
    let error = CssAngleLiteral::try_from_component(component).unwrap_err();
    assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
    assert_eq!(error.origin(), &origin);
    let source = declaration("offset-path", "ray(calc(1turn - 90deg))");
    let bearing = ray(&source).angle();
    let components = bearing.calculation().unwrap().components().clone();
    let constructed = CssAngleValue::try_from_calculation(
        CssAngleCalculation::try_from_components(components.clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(constructed.calculation().unwrap().components(), &components);
    assert_eq!(constructed.origin(), bearing.origin());
    let model = CssRay::new(constructed, None, false, None);
    assert_eq!(model.serialize_specified().unwrap(), "ray(calc(270deg))");
    assert_eq!(
        model.angle().calculation().unwrap().components(),
        &components
    );
}

#[test]
fn recovered_bearing_graph_cannot_reenter_a_checked_motion_constructor() {
    let report = parse_style_attribute("/*😀*/offset-path:ray(calc(90deg");
    assert!(!report.is_clean());
    let [source] = report.syntax().as_slice() else {
        panic!("retained recovered authored occurrence")
    };
    let bearing = ray(source).angle();
    let calculation = bearing.calculation().unwrap();
    let CssComponentValueRef::Function(function) = calculation.components().items()[0].view()
    else {
        panic!("calc root")
    };
    let before = calculation.components().clone();
    let error = CssAngleValue::try_from_calculation(calculation.clone()).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::RecoveredComponent
    );
    assert_eq!(error.origin(), Some(function.closing_origin()));
    assert!(error.path().is_none());
    assert!(std::error::Error::source(&error).is_none());
    assert_eq!(calculation.components(), &before);
    assert_eq!(
        CssAngleValue::try_from_calculation(calculation.clone()).unwrap_err(),
        error
    );
}

#[test]
fn ray_catalog_reports_exact_selected_production_and_authored_complete_status() {
    let feature = feature_metadata("official.value.ray").unwrap();
    assert_eq!(feature.kind(), CssFeatureKind::Value);
    assert_eq!(feature.spelling(), "ray()");
    assert_eq!(feature.production(), "#ray-function");
    assert_eq!(feature.status(), CssSupportStatus::Complete);
    assert_eq!(feature.source().id().as_str(), "I-MOTION1");
    assert_eq!(
        feature.source().tier(),
        CssSpecificationTier::Snapshot2026Interop
    );
    assert!(feature.supported_subset().is_none() && feature.unsupported_remainder().is_none());
    let source = declaration("offset-path", "ray(90deg sides contain at left top)");
    assert_eq!(
        ray(&source).serialize_specified().unwrap(),
        "ray(90deg sides contain at left top)"
    );
}

#[test]
fn whole_path_views_preserve_none_box_only_and_each_coupled_path_kind() {
    let none = CssOffsetPath::none();
    assert_eq!(none.view(), CssOffsetPathRef::None);
    budget("none", 1, |limits| {
        none.serialize_specified_with_limits(limits)
    });
    let box_only = CssOffsetPath::from_coord_box(CssCoordBox::BorderBox);
    assert_eq!(
        box_only.view(),
        CssOffsetPathRef::CoordBox(CssCoordBox::BorderBox)
    );
    budget("border-box", 1, |limits| {
        box_only.serialize_specified_with_limits(limits)
    });
    let kinds = [
        (
            CssOffsetPathKind::Ray(CssRay::new(
                angle("90", CssAngleUnit::Degrees),
                None,
                false,
                None,
            )),
            "ray(90deg)",
            2,
        ),
        (CssOffsetPathKind::Url(CssUrl::new("#p")), "url(\"#p\")", 2),
        (
            CssOffsetPathKind::BasicShape(CssBasicShape::Circle(CssCircleShape::new(
                CssCircleRadius::Default,
                None,
            ))),
            "circle()",
            1,
        ),
    ];
    for (kind, css, nodes) in kinds {
        budget(css, nodes, |limits| {
            kind.serialize_specified_with_limits(limits)
        });
        let pair = CssOffsetPathValue::new(kind.clone(), Some(CssCoordBox::BorderBox));
        assert_eq!(pair.path(), &kind);
        assert_eq!(pair.coord_box(), Some(CssCoordBox::BorderBox));
        let path = CssOffsetPath::from_path(pair.clone());
        assert_eq!(path.view(), CssOffsetPathRef::Path(&pair));
        assert_ne!(path, box_only);
        let expected = format!("{css} border-box");
        budget(&expected, nodes + 2, |limits| {
            pair.serialize_specified_with_limits(limits)
        });
        budget(&expected, nodes + 2, |limits| {
            path.serialize_specified_with_limits(limits)
        });
        let omitted = CssOffsetPathValue::new(kind, None);
        assert!(omitted.coord_box().is_none());
        assert_ne!(omitted, pair);
        budget(css, nodes + 1, |limits| {
            omitted.serialize_specified_with_limits(limits)
        });
    }
}

#[test]
fn path_url_view_retains_src_target_modifier_order_and_never_becomes_image() {
    let source = declaration(
        "offset-path",
        "view-box SRC('é.svg' CORS integrity(\"sha256\"))",
    );
    let CssOffsetPathRef::Path(pair) = offset_path(&source).view() else {
        panic!("coupled path")
    };
    assert_eq!(pair.coord_box(), Some(CssCoordBox::ViewBox));
    let CssOffsetPathKind::Url(url) = pair.path() else {
        panic!("url path, no image wrapper")
    };
    assert_eq!(url.function(), CssUrlFunction::Src);
    assert_eq!(url.as_str(), "é.svg");
    assert_eq!(url.modifiers().len(), 2);
    assert_eq!(
        pair.serialize_specified().unwrap(),
        "src(\"é.svg\" CORS integrity(\"sha256\")) view-box"
    );
    let empty = CssOffsetPath::from_path(CssOffsetPathValue::new(
        CssOffsetPathKind::Url(CssUrl::new(String::new())),
        None,
    ));
    assert_eq!(empty.serialize_specified().unwrap(), "url(\"\")");
}

#[test]
fn physical_position_anchor_and_position_branches_retain_checked_axes_and_origins() {
    let scalar = CssSpecifiedLengthPercentage::try_from_component(
        CssComponentValue::try_token("-25%").unwrap(),
    )
    .unwrap();
    let physical = CssPhysicalPosition::try_new(
        CssHorizontalPosition::Offset(scalar.clone()),
        CssVerticalPosition::Bottom,
    )
    .unwrap();
    assert_eq!(
        physical.horizontal(),
        &CssHorizontalPosition::Offset(scalar.clone())
    );
    assert_eq!(scalar.origin(), &CssValueOrigin::Programmatic);
    let start = CssOffsetPosition::Position(physical.clone().into());
    let anchor = CssOffsetAnchor::Position(physical.into());
    // Position aggregate + two axis nodes + the retained percentage leaf.
    budget("-25% bottom", 4, |limits| {
        start.serialize_specified_with_limits(limits)
    });
    budget("-25% bottom", 4, |limits| {
        anchor.serialize_specified_with_limits(limits)
    });
    budget("normal", 1, |limits| {
        CssOffsetPosition::Normal.serialize_specified_with_limits(limits)
    });
    budget("auto", 1, |limits| {
        CssOffsetPosition::Auto.serialize_specified_with_limits(limits)
    });
    budget("auto", 1, |limits| {
        CssOffsetAnchor::Auto.serialize_specified_with_limits(limits)
    });
    assert_eq!(
        CssPhysicalPosition::try_new(CssHorizontalPosition::XStart, CssVerticalPosition::Top),
        Err(CssPositionConstructionError::NonPhysicalKeyword)
    );
    assert_eq!(
        CssPhysicalPosition::try_new(
            CssHorizontalPosition::LeftOffset(scalar),
            CssVerticalPosition::Top
        ),
        Err(CssPositionConstructionError::UnpairedEdgeOffsets)
    );
}

#[test]
fn shorthand_constructor_rejects_anchor_only_and_pathless_distance_or_rotation() {
    assert_eq!(
        CssOffset::try_new(None, None, None, None, None),
        Err(CssMotionConstructionError::EmptyOffset)
    );
    assert_eq!(
        CssOffset::try_new(None, None, None, None, Some(CssOffsetAnchor::Auto)),
        Err(CssMotionConstructionError::EmptyOffset)
    );
    assert_eq!(
        CssOffset::try_new(
            Some(CssOffsetPosition::Normal),
            None,
            Some(CssSpecifiedLengthPercentage::zero()),
            None,
            None
        ),
        Err(CssMotionConstructionError::MissingPath)
    );
    assert_eq!(
        CssOffset::try_new(
            Some(CssOffsetPosition::Auto),
            None,
            None,
            Some(CssOffsetRotate::auto()),
            None
        ),
        Err(CssMotionConstructionError::MissingPath)
    );
    let position_only =
        CssOffset::try_new(Some(CssOffsetPosition::Normal), None, None, None, None).unwrap();
    assert_eq!(position_only.position(), Some(&CssOffsetPosition::Normal));
    assert!(
        position_only.path().is_none()
            && position_only.distance().is_none()
            && position_only.rotate().is_none()
            && position_only.anchor().is_none()
    );
    budget("normal", 2, |limits| {
        position_only.serialize_specified_with_limits(limits)
    });
    let full = CssOffset::try_new(
        Some(CssOffsetPosition::Normal),
        Some(CssOffsetPath::none()),
        Some(CssSpecifiedLengthPercentage::zero()),
        Some(CssOffsetRotate::auto()),
        Some(CssOffsetAnchor::Auto),
    )
    .unwrap();
    assert_eq!(full.path().unwrap().view(), CssOffsetPathRef::None);
    assert_eq!(
        full.distance().unwrap().origin(),
        &CssValueOrigin::Programmatic
    );
    assert_eq!(
        full.rotate().unwrap().modifier(),
        Some(CssOffsetRotateModifier::Auto)
    );
    assert_eq!(full.anchor(), Some(&CssOffsetAnchor::Auto));
    budget("normal none 0 auto / auto", 7, |limits| {
        full.serialize_specified_with_limits(limits)
    });
}

fn assert_initial(value: CssLonghandValueRef<'_>) {
    match value {
        CssLonghandValueRef::OffsetPosition(value) => assert_eq!(*value, CssOffsetPosition::Normal),
        CssLonghandValueRef::OffsetPath(value) => assert_eq!(value.view(), CssOffsetPathRef::None),
        CssLonghandValueRef::OffsetDistance(value) => {
            assert!(value.calculation().is_none());
            assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
            assert!(
                matches!(value.literal_component().unwrap().view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.representation() == "0")
            );
        }
        CssLonghandValueRef::OffsetRotate(value) => {
            assert_eq!(value.modifier(), Some(CssOffsetRotateModifier::Auto));
            assert!(value.angle().is_none());
        }
        CssLonghandValueRef::OffsetAnchor(value) => assert_eq!(*value, CssOffsetAnchor::Auto),
        _ => panic!("Motion terminal"),
    }
}

#[test]
fn intrinsic_initial_payloads_and_omitted_shorthand_members_use_exact_central_values() {
    let properties = [
        CssKnownProperty::OffsetPosition,
        CssKnownProperty::OffsetPath,
        CssKnownProperty::OffsetDistance,
        CssKnownProperty::OffsetRotate,
        CssKnownProperty::OffsetAnchor,
    ];
    for property in properties {
        let metadata = property.metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("longhand")
        };
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("fixed intrinsic initial")
        };
        assert_initial(value.view());
    }
    for text in ["normal", "none", "normal none"] {
        let source = declaration("offset", text);
        let items = contributions(&source);
        assert_eq!(items.items().len(), 5);
        for (item, property) in items.items().iter().zip(properties) {
            assert_eq!(item.property(), property);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
            assert_initial(item.ordinary_value().unwrap().view());
        }
        assert_eq!(offset(&source).serialize_specified().unwrap(), text);
    }
}

#[test]
fn typed_extraction_preserves_parsed_programmatic_and_replacement_numeric_provenance() {
    let source = declaration("offset", "left top ray(.25turn) -25% reverse 90deg / auto");
    let model = offset(&source);
    assert!(matches!(
        model.position(),
        Some(CssOffsetPosition::Position(_))
    ));
    let CssOffsetPathRef::Path(path) = model.path().unwrap().view() else {
        panic!("ray path")
    };
    let CssOffsetPathKind::Ray(ray) = path.path() else {
        panic!("ray")
    };
    let CssValueOrigin::Parsed(origin) = ray.angle().origin() else {
        panic!("parsed bearing origin")
    };
    assert_eq!(
        origin.source().as_str(),
        "/*😀*/offset:left top ray(.25turn) -25% reverse 90deg / auto!important"
    );
    let original = source
        .value_components()
        .items()
        .iter()
        .find_map(|component| {
            let CssComponentValueRef::Function(function) = component.view() else {
                return None;
            };
            function
                .name()
                .eq_ignore_ascii_case("ray")
                .then(|| &function.values().items()[0])
        })
        .expect("original authored ray angle");
    assert_eq!(ray.angle().literal().unwrap().component(), original);
    assert_eq!(
        ray.angle().literal().unwrap().numeric().representation(),
        ".25"
    );
    assert!(model.rotate().unwrap().angle().unwrap().literal().is_some());
    let pending = declaration("offset", "var(--m)");
    let CssExpansion::Pending(handle) = expand_declaration(&pending).unwrap() else {
        panic!("pending")
    };
    let replacement = parse_component_values("ray(450deg) -2em reverse 90deg").unwrap();
    let before = replacement.clone();
    let CssContributions::Longhands(items) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("reentered")
    };
    assert_eq!(replacement, before);
    for item in items.items() {
        assert!(item.source().same_occurrence(&pending));
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
    let CssLonghandValueRef::OffsetPath(path) = items.items()[1].ordinary_value().unwrap().view()
    else {
        panic!("path terminal")
    };
    let CssOffsetPathRef::Path(path) = path.view() else {
        panic!("path pair")
    };
    let CssOffsetPathKind::Ray(ray) = path.path() else {
        panic!("ray")
    };
    let CssValueOrigin::Parsed(origin) = ray.angle().origin() else {
        panic!("replacement bearing")
    };
    assert_eq!(origin.source().as_str(), "ray(450deg) -2em reverse 90deg");
    assert!(
        handle
            .reenter(parse_component_values("none").unwrap())
            .is_ok()
    );
}

#[test]
fn typed_checked_construction_preserves_supplied_angle_and_distance_component_identity() {
    let bearing_component = CssComponentValue::try_dimension("+450.00", "GRAD").unwrap();
    let bearing = CssAngleValue::from_literal(
        CssAngleLiteral::try_from_component(bearing_component.clone()).unwrap(),
    );
    let distance_component = CssComponentValue::try_token("-25.00%").unwrap();
    let distance =
        CssSpecifiedLengthPercentage::try_from_component(distance_component.clone()).unwrap();
    let ray = CssRay::new(bearing, None, false, None);
    let model = CssOffset::try_new(
        None,
        Some(CssOffsetPath::from_path(CssOffsetPathValue::new(
            CssOffsetPathKind::Ray(ray),
            None,
        ))),
        Some(distance),
        None,
        None,
    )
    .unwrap();
    let CssOffsetPathRef::Path(path) = model.path().unwrap().view() else {
        panic!("path pair")
    };
    let CssOffsetPathKind::Ray(ray) = path.path() else {
        panic!("ray")
    };
    assert_eq!(
        ray.angle().literal().unwrap().component(),
        &bearing_component
    );
    assert_eq!(
        model.distance().unwrap().literal_component(),
        Some(&distance_component)
    );
    let before = model.clone();
    budget("ray(450grad) -25%", 5, |limits| {
        model.serialize_specified_with_limits(limits)
    });
    assert_eq!(model, before);
    assert_eq!(
        ray.angle().literal().unwrap().component(),
        &bearing_component
    );
    assert_eq!(
        model.distance().unwrap().literal_component(),
        Some(&distance_component)
    );
}

#[test]
fn composed_motion_numeric_graphs_share_existing_projection_limits_without_mutation() {
    let source = declaration("offset-path", "ray(calc(cos(1e400deg) * 1deg)) view-box");
    let value = offset_path(&source);
    let before = value.clone();
    let expected = "ray(calc(1deg)) view-box";
    // Pair/ray/box are three nodes. The Angle owner visits calc/product/cos
    // and two literals, projecting four retained canonical math nodes.
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(8, 7, expected.len()))
            .unwrap(),
        expected
    );
    let error = value
        .serialize_specified_with_limits(L::new(8, 6, expected.len()))
        .unwrap_err();
    assert_eq!(error.kind(), K::ProjectionNodeLimit);
    assert_eq!(value, &before);
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(8, 6, expected.len()))
            .unwrap_err(),
        error
    );
    assert_eq!(value.serialize_specified().unwrap(), expected);
}
