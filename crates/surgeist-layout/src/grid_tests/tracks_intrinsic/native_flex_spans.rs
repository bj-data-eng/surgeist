//! Font-free public geometry for spanning items crossing sub-one flex tracks.
use super::*;
use crate::test_support::readback::{self, Phase};

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn tree<S: LayoutScalar>(width: Option<f64>) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(
                    width.map_or(PreferredSizeOf::AUTO, |width| PreferredSizeOf::px(n(width))),
                    PreferredSizeOf::AUTO,
                ),
                grid_template_columns: vec![
                    track_component_flex(n(0.2)),
                    track_component_flex(n(0.3)),
                ],
                grid_template_rows: vec![TrackComponentOf::px(n(40.0)); 2],
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::px(n(60.0)), PreferredSizeOf::AUTO),
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Flex,
                grid_column: GridPlacement::try_lines(1, 2).unwrap(),
                grid_row: GridPlacement::try_lines(2, 3).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Flex,
                grid_column: GridPlacement::try_lines(2, 3).unwrap(),
                grid_row: GridPlacement::try_lines(2, 3).unwrap(),
                ..NodeInputOf::default()
            },
        )
}

fn output<S: LayoutScalar>(entries: &[LayoutOutputEntryOf<u32, S>], node: u32) -> NodeOutputOf<S> {
    let mut matching = entries.iter().filter(|entry| entry.node() == node);
    let output = matching.next().expect("every box is published").output();
    assert!(
        matching.next().is_none(),
        "one actual geometry entry per box"
    );
    output
}

fn compute<S: LayoutScalar>(width: Option<f64>) -> CompletedLayoutBatchOf<u32, S> {
    let tree = tree::<S>(width);
    assert_eq!(Traverse::children(&tree, 0).collect::<Vec<_>>(), [1, 2, 3]);
    for node in [0, 1, 2, 3] {
        assert!(
            !tree.has_leaf_measurement(node),
            "native layout, no measured output"
        );
    }
    for node in [1, 2, 3] {
        assert_eq!(Traverse::children(&tree, node).count(), 0);
        assert_eq!(tree.node_input(node).display, Display::Flex);
    }
    assert_eq!(tree.node_input(1).size.width, PreferredSizeOf::px(n(60.0)));
    assert_eq!(
        tree.node_input(1).grid_column,
        GridPlacement::try_lines(1, 3).unwrap()
    );
    let request = {
        let axes = Size::splat(AxisSizingRequestOf::new(
            AvailableSpaceOf::Indefinite,
            SizingOperation::MaxContent,
        ));
        let definite = axes.map(|axis| axis.space().definite_size());
        crate::LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    };
    assert_eq!(
        request.sizing_request().axes(),
        Size::splat(AxisSizingRequestOf::new(
            AvailableSpaceOf::Indefinite,
            SizingOperation::MaxContent
        ))
    );
    let batch =
        compute_layout(&tree, 0, request).expect("real native flex-spanning grid completes");
    for entries in [
        readback::continuous_custody(&batch, Phase::Unrounded).as_slice(),
        readback::continuous_custody(&batch, Phase::Final).as_slice(),
    ] {
        assert_eq!(entries.len(), 4);
        for (node, index) in [(0, 0), (1, 0), (2, 1), (3, 2)] {
            let actual = output(entries, node);
            assert_eq!(actual.source_index, SourceIndex::new(index));
            assert_eq!(actual.margin, Edges::ZERO);
            assert_eq!(actual.border, Edges::ZERO);
            assert_eq!(actual.padding, Edges::ZERO);
        }
        // This fixed body's native geometry is a premise, independent of the
        // two unknown track widths. It has no font or host measurement.
        assert_eq!(output(entries, 1).location, Point::ZERO);
        assert_eq!(output(entries, 1).size, Size::new(n(60.0), n(40.0)));
    }
    batch
}

fn assert_geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    container: f64,
    first: f64,
    second: f64,
    normal_tracks: [S; 2],
) {
    // Static zero-edge inputs have no relative displacement: their unrounded
    // packets are the actual normal boxes; final packets are published geometry.
    for (published, entries) in [
        (
            false,
            readback::continuous_custody(batch, Phase::Unrounded).as_slice(),
        ),
        (
            true,
            readback::continuous_custody(batch, Phase::Final).as_slice(),
        ),
    ] {
        for (node, x, y, width, height) in [
            (0, 0.0, 0.0, container, 80.0),
            (1, 0.0, 0.0, 60.0, 40.0),
            (2, 0.0, 40.0, first, 40.0),
            (3, first, 40.0, second, 40.0),
        ] {
            let actual = output(entries, node);
            let expected_x = if !published && node == 3 {
                normal_tracks[0]
            } else {
                n(x)
            };
            let expected_width = if !published {
                match node {
                    2 => normal_tracks[0],
                    3 => normal_tracks[1],
                    _ => n(width),
                }
            } else {
                n(width)
            };
            assert_eq!(
                actual.location,
                Point::new(expected_x, n(y)),
                "node{node} position"
            );
            assert_eq!(
                actual.size,
                Size::new(expected_width, n(height)),
                "node{node} size"
            );
            assert_eq!(
                actual.content_box_size(),
                actual.size,
                "zero-edge native box"
            );
        }
    }
}

fn spanning_intrinsic<S: LayoutScalar>() {
    let batch = compute::<S>(None);
    // Grid2 section12.5 step4: the sum is .5, so half of the real
    // spanning contribution60 is split in the .2:.3 ratio (12,18),
    // and the remaining30 equally (15,15). These are floors27/33;
    // later flexible sizing cannot reduce them or enlarge the container
    // merely to restore the original ratio.
    assert_geometry(&batch, 60.0, 27.0, 33.0, [n(27.0), n(33.0)]);
}

fn spanning_definite<S: LayoutScalar>() {
    let batch = compute::<S>(Some(200.0));
    // In definite available200, the factors request40 and60, above both
    // intrinsic floors. The unrequested100 remains host space at End.
    // Normal coordinates use the represented input factors times the actual
    // definite space. In f32, 0.3 * 200 is one ULP above60; publication rounds
    // that coordinate to60. Both contracts are checked without a tolerance.
    assert_geometry(
        &batch,
        200.0,
        40.0,
        60.0,
        [n::<S>(0.2) * n::<S>(200.0), n::<S>(0.3) * n::<S>(200.0)],
    );
    for entries in [
        readback::continuous_custody(&batch, Phase::Unrounded).as_slice(),
        readback::continuous_custody(&batch, Phase::Final).as_slice(),
    ] {
        let root = output(entries, 0);
        let last = output(entries, 3);
        assert_eq!(
            root.size.width - (last.location.x + last.size.width),
            n(100.0)
        );
    }
}

#[test]
fn native_sub_one_flex_span_intrinsic_f32() {
    spanning_intrinsic::<f32>();
}

#[test]
fn native_sub_one_flex_span_intrinsic_f64() {
    spanning_intrinsic::<f64>();
}

#[test]
fn native_sub_one_flex_span_definite_f32() {
    spanning_definite::<f32>();
}

#[test]
fn native_sub_one_flex_span_definite_f64() {
    spanning_definite::<f64>();
}
