#![forbid(unsafe_code)]
//! Functional contracts for the widened Motion CssPosition payloads. The
//! immutable published parser RED covers preimplementation regression behavior;
//! these typed calls first become expressible with the full owning payload.
use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

fn scalar(text: &str) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_component(CssComponentValue::try_token(text).unwrap())
        .unwrap()
}

fn angle() -> CssAngleValue {
    CssAngleValue::from_literal(CssAngleLiteral::try_new(".25", CssAngleUnit::Turns).unwrap())
}

fn families() -> Vec<(CssPosition, &'static str, usize)> {
    vec![
        (
            CssPosition::from_cartesian(
                CssCartesianPosition::try_new(
                    CssHorizontalPosition::XStart,
                    CssVerticalPosition::YEnd,
                )
                .unwrap(),
            ),
            "x-start y-end",
            3,
        ),
        (
            CssPosition::try_from_named_axes(CssBlockPosition::Start, CssInlinePosition::End)
                .unwrap(),
            "block-start inline-end",
            3,
        ),
        (
            CssPosition::try_from_relative_axes(
                CssRelativeAxisPosition::End,
                CssRelativeAxisPosition::Start,
            )
            .unwrap(),
            "end start",
            3,
        ),
        (
            CssPosition::from_cartesian(
                CssCartesianPosition::try_new(
                    CssHorizontalPosition::XEndOffset(scalar("-2px")),
                    CssVerticalPosition::YStartOffset(scalar("10%")),
                )
                .unwrap(),
            ),
            "x-end -2px y-start 10%",
            5,
        ),
        (
            CssPosition::try_from_named_axes(
                CssBlockPosition::EndOffset(scalar("-2px")),
                CssInlinePosition::StartOffset(scalar("10%")),
            )
            .unwrap(),
            "block-end -2px inline-start 10%",
            5,
        ),
        (
            CssPosition::try_from_relative_axes(
                CssRelativeAxisPosition::StartOffset(scalar("-2px")),
                CssRelativeAxisPosition::EndOffset(scalar("10%")),
            )
            .unwrap(),
            "start -2px end 10%",
            5,
        ),
    ]
}

fn budget(
    expected: &str,
    nodes: usize,
    emit: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    assert_eq!(
        emit(L::new(nodes, nodes, expected.len())).unwrap(),
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
        assert_eq!(emit(limits).unwrap_err().kind(), kind);
    }
    assert_eq!(emit(L::default()).unwrap(), expected);
}

#[test]
fn direct_position_and_anchor_constructors_retain_each_checked_symbolic_family() {
    for (position, expected, nodes) in families() {
        let before = position.clone();
        let offset = CssOffsetPosition::Position(position.clone());
        let anchor = CssOffsetAnchor::Position(position.clone());
        budget(expected, nodes, |limits| {
            offset.serialize_specified_with_limits(limits)
        });
        budget(expected, nodes, |limits| {
            anchor.serialize_specified_with_limits(limits)
        });
        assert_eq!(offset, CssOffsetPosition::Position(before.clone()));
        assert_eq!(anchor, CssOffsetAnchor::Position(before.clone()));
        assert_eq!(position, before);
    }
}

#[test]
fn checked_ray_construction_retains_logical_positions_angle_units_and_omissions() {
    for (position, expected, nodes) in families() {
        let ray = CssRay::new(angle(), None, false, Some(position.clone()));
        let before = ray.clone();
        assert_eq!(ray.position(), Some(&position));
        assert!(ray.size().is_none());
        assert!(!ray.contain());
        assert_eq!(ray.angle().origin(), &CssValueOrigin::Programmatic);
        assert_eq!(ray.angle().literal().unwrap().unit(), CssAngleUnit::Turns);
        budget(
            &format!("ray(0.25turn at {expected})"),
            nodes + 2,
            |limits| ray.serialize_specified_with_limits(limits),
        );
        assert_eq!(ray, before);
    }
}

#[test]
fn checked_offset_composes_full_positions_without_fabricating_omitted_constituents() {
    for (position, expected, nodes) in families() {
        let position_only = CssOffset::try_new(
            Some(CssOffsetPosition::Position(position.clone())),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        budget(expected, nodes + 1, |limits| {
            position_only.serialize_specified_with_limits(limits)
        });
        assert!(position_only.path().is_none());
        assert!(position_only.distance().is_none());
        assert!(position_only.rotate().is_none());
        assert!(position_only.anchor().is_none());
        let ray = CssRay::new(
            angle(),
            Some(CssRaySize::ClosestSide),
            true,
            Some(position.clone()),
        );
        let path = CssOffsetPath::from_path(CssOffsetPathValue::new(
            CssOffsetPathKind::Ray(ray),
            Some(CssCoordBox::BorderBox),
        ));
        let full = CssOffset::try_new(
            Some(CssOffsetPosition::Position(position.clone())),
            Some(path),
            None,
            None,
            Some(CssOffsetAnchor::Position(position)),
        )
        .unwrap();
        let before = full.clone();
        // Offset, path-pair, ray, angle, explicit size, contain and box: seven
        // nodes in addition to the three independent full position providers.
        budget(
            &format!("{expected} ray(0.25turn contain at {expected}) border-box / {expected}"),
            nodes * 3 + 7,
            |limits| full.serialize_specified_with_limits(limits),
        );
        assert_eq!(full, before);
    }
}

#[test]
fn motion_composition_keeps_programmatic_logical_offset_scalar_origins() {
    for (position, _, nodes) in families().into_iter().filter(|(_, _, nodes)| *nodes == 5) {
        let ray = CssRay::new(angle(), None, false, Some(position));
        let position = ray.position().unwrap();
        let scalars = match position.view() {
            CssPositionRef::Cartesian(value) => {
                let CssHorizontalPosition::XEndOffset(x) = value.horizontal() else {
                    panic!("x")
                };
                let CssVerticalPosition::YStartOffset(y) = value.vertical() else {
                    panic!("y")
                };
                [x, y]
            }
            CssPositionRef::NamedFlow(value) => {
                let CssBlockPosition::EndOffset(block) = value.block() else {
                    panic!("block")
                };
                let CssInlinePosition::StartOffset(inline) = value.inline() else {
                    panic!("inline")
                };
                [block, inline]
            }
            CssPositionRef::RelativeFlow(value) => {
                let CssRelativeAxisPosition::StartOffset(block) = value.block() else {
                    panic!("block")
                };
                let CssRelativeAxisPosition::EndOffset(inline) = value.inline() else {
                    panic!("inline")
                };
                [block, inline]
            }
            _ => panic!("selected coordinate family"),
        };
        assert_eq!(nodes, 5);
        for scalar in scalars {
            assert_eq!(scalar.origin(), &CssValueOrigin::Programmatic);
        }
        assert_eq!(scalars[0].serialize_specified().unwrap(), "-2px");
        assert_eq!(scalars[1].serialize_specified().unwrap(), "10%");
    }
}
