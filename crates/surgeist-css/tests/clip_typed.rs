#![forbid(unsafe_code)]

//! Checked `clip` construction and specified serialization per CSS Masking 1
//! Appendix A, selected 2021-08-05 Candidate Recommendation Draft:
//! https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/#propdef-clip
//! CSS Values 4 §10.13 defines canonical calculation serialization:
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-serialize

use surgeist_css::*;

fn px(value: f32) -> CssLength {
    CssLength::try_px(value).unwrap()
}

fn em(value: f32) -> CssLength {
    CssLength::try_dimension(value, CssLengthUnit::Em).unwrap()
}

fn typed_calc(source: &str) -> CssLength {
    CssLength::Calc(CssCalcLength::Typed(
        CssLengthPercentageCalculation::try_from_components(
            parse_component_values(source).unwrap(),
        )
        .unwrap(),
    ))
}

fn edge(value: CssLength) -> CssClipEdge {
    CssClipEdge::Length(CssClipLength::try_new(value).unwrap())
}

fn parsed(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn parsed_clip(declaration: &CssDeclaration) -> &CssClip {
    let CssKnownPropertyValueRef::Clip(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed clip")
    };
    value.clip()
}

#[test]
fn checked_edge_construction_accepts_signed_pure_lengths_and_rejects_other_domains() {
    for (value, css) in [(CssLength::Zero, "0"), (px(-1.0), "-1px"), (em(2.0), "2em")] {
        let checked = CssClipLength::try_new(value.clone()).unwrap();
        assert_eq!(checked.value(), &value);
        assert_eq!(checked.serialize_specified().unwrap(), css);
        let edge = CssClipEdge::Length(checked);
        assert_eq!(edge.serialize_specified().unwrap(), css);
    }
    for rejected in [
        CssLength::try_percent(1.0).unwrap(),
        CssLength::Auto,
        CssLength::MinContent,
        CssLength::MaxContent,
    ] {
        assert!(CssClipLength::try_new(rejected).is_none());
    }
    assert_eq!(CssClipEdge::Auto.serialize_specified().unwrap(), "auto");
    assert_eq!(CssClip::Auto.serialize_specified().unwrap(), "auto");
}

#[test]
fn constructed_rectangle_preserves_four_edge_order_and_canonical_commas() {
    let rect = CssClipRect::new(
        CssClipEdge::Auto,
        edge(px(-1.0)),
        edge(em(2.0)),
        edge(CssLength::Zero),
    );
    assert!(matches!(rect.top(), CssClipEdge::Auto));
    assert!(matches!(rect.right(), CssClipEdge::Length(length) if length.value() == &px(-1.0)));
    assert!(matches!(rect.bottom(), CssClipEdge::Length(length) if length.value() == &em(2.0)));
    assert!(
        matches!(rect.left(), CssClipEdge::Length(length) if length.value() == &CssLength::Zero)
    );
    assert_eq!(
        rect.serialize_specified().unwrap(),
        "rect(auto, -1px, 2em, 0)"
    );
    let clip = CssClip::Rect(rect);
    assert_eq!(
        clip.serialize_specified().unwrap(),
        "rect(auto, -1px, 2em, 0)"
    );

    let from_space = parsed("clip:rect(auto -1px 2em 0)");
    assert_eq!(
        parsed_clip(&from_space).serialize_specified().unwrap(),
        "rect(auto, -1px, 2em, 0)"
    );
    let reparsed = parsed("clip:rect(auto, -1px, 2em, 0)");
    let CssClip::Rect(reparsed_rect) = parsed_clip(&reparsed) else {
        panic!("canonical rectangle reparses")
    };
    assert!(matches!(reparsed_rect.top(), CssClipEdge::Auto));
    assert!(
        matches!(reparsed_rect.right(), CssClipEdge::Length(length) if matches!(length.value(), CssLength::Px(value) if value.value() == -1.0))
    );
    assert!(
        matches!(reparsed_rect.bottom(), CssClipEdge::Length(length) if matches!(length.value(), CssLength::Dimension(_)))
    );
    assert!(
        matches!(reparsed_rect.left(), CssClipEdge::Length(length) if length.value() == &CssLength::Zero)
    );
}

#[test]
fn legacy_and_typed_calculations_remain_wrapped_and_canonical() {
    let legacy = CssLength::Calc(CssCalcLength::try_px(-2.0).unwrap());
    let legacy_length = CssClipLength::try_new(legacy.clone()).unwrap();
    assert_eq!(legacy_length.value(), &legacy);
    assert_eq!(legacy_length.serialize_specified().unwrap(), "calc(-2px)");

    let folded = typed_calc("calc(1px - 2px)");
    let mixed = typed_calc("calc(1px + 2em)");
    assert_eq!(
        CssClipLength::try_new(folded.clone()).unwrap().value(),
        &folded
    );
    assert_eq!(
        CssClipLength::try_new(mixed.clone()).unwrap().value(),
        &mixed
    );
    let clip = CssClip::Rect(CssClipRect::new(
        edge(legacy),
        edge(folded),
        edge(mixed),
        CssClipEdge::Auto,
    ));
    assert_eq!(
        clip.serialize_specified().unwrap(),
        "rect(calc(-2px), calc(-1px), calc(2em + 1px), auto)"
    );
    let parsed = parsed(&format!("clip:{}", clip.serialize_specified().unwrap()));
    let CssClip::Rect(rect) = parsed_clip(&parsed) else {
        panic!("math rectangle")
    };
    assert!(
        matches!(rect.top(), CssClipEdge::Length(length) if matches!(length.value(), CssLength::Calc(_)))
    );
    assert!(
        matches!(rect.right(), CssClipEdge::Length(length) if matches!(length.value(), CssLength::Calc(_)))
    );
    assert!(
        matches!(rect.bottom(), CssClipEdge::Length(length) if matches!(length.value(), CssLength::Calc(_)))
    );
}

#[test]
fn initial_and_authored_expansion_have_typed_auto_and_rectangle_values() {
    let CssPropertyKindRef::Longhand(metadata) = CssKnownProperty::Clip.metadata().unwrap().kind()
    else {
        panic!("clip longhand")
    };
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("ordinary initial")
    };
    let CssLonghandValueRef::Clip(initial_clip) = value.view() else {
        panic!("typed initial")
    };
    assert_eq!(initial_clip.serialize_specified().unwrap(), "auto");

    let authored = parsed("clip:rect(auto -1px 2em 0)!important");
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(&authored).unwrap()
    else {
        panic!("clip longhand contribution")
    };
    let [item] = items.items() else {
        panic!("one contribution")
    };
    let CssLonghandValueRef::Clip(clip) = item.ordinary_value().unwrap().view() else {
        panic!("typed clip contribution")
    };
    assert_eq!(
        clip.serialize_specified().unwrap(),
        "rect(auto, -1px, 2em, 0)"
    );
    assert!(item.source().same_occurrence(&authored));
    assert_eq!(item.source().importance(), CssImportance::Important);
}

#[test]
fn edge_and_rectangle_limits_share_exact_cumulative_budgets() {
    let one = CssClipEdge::Auto;
    assert_eq!(
        one.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 4))
            .unwrap(),
        "auto"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, 4),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, 4),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, 3),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            one.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    let rect = CssClipRect::new(
        CssClipEdge::Auto,
        edge(px(-1.0)),
        edge(em(2.0)),
        edge(CssLength::Zero),
    );
    let expected = "rect(auto, -1px, 2em, 0)";
    assert_eq!(expected.len(), 24);
    assert_eq!(
        rect.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(5, 5, 24))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(4, 5, 24),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 4, 24),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 5, 23),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            rect.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(
        rect.serialize_specified().unwrap(),
        expected,
        "failed serialization leaves value unchanged"
    );
}

#[test]
fn symbolic_math_edges_obey_one_shared_rectangle_byte_budget() {
    let mixed = edge(typed_calc("calc(1px + 2em)"));
    assert_eq!(mixed.serialize_specified().unwrap(), "calc(2em + 1px)");
    assert_eq!(
        mixed
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                1_000, 1_000, 15
            ))
            .unwrap(),
        "calc(2em + 1px)"
    );
    let rect = CssClipRect::new(mixed.clone(), mixed, CssClipEdge::Auto, CssClipEdge::Auto);
    let expected = "rect(calc(2em + 1px), calc(2em + 1px), auto, auto)";
    assert_eq!(rect.serialize_specified().unwrap(), expected);
    assert_eq!(
        rect.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            1_000,
            1_000,
            expected.len()
        ))
        .unwrap(),
        expected
    );
    assert_eq!(
        rect.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            1_000, 1_000, 15
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(
        rect.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            0, 1_000, 1_000
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        rect.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            1_000, 0, 1_000
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
}
