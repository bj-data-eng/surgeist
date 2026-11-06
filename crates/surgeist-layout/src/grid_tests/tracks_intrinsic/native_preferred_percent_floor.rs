//! Cyclic percentage tracks resolve against the intrinsically chosen container.
use super::*;
use crate::engine::contracts::Compute;

fn percent_subject<S: LayoutScalar>(preferred: PreferredSizeOf<S>) -> DistinctLeaf<S> {
    let mut tree = source::<S>(100.0, GrowthCap::MaxContent, Overflow::Clip);
    let mut root = tree.node_input(0).clone();
    root.display = Display::InlineGrid;
    root.size = Size::new(preferred, PreferredSizeOf::AUTO);
    root.grid_template_columns = vec![TrackComponentOf::percent(S::ONE)];
    root.grid_template_rows = vec![TrackComponentOf::AUTO];
    tree.source.insert_input(0, LayoutInputOf::box_input(root));
    let mut child = tree.node_input(1).clone();
    child.size.height = PreferredSizeOf::AUTO;
    tree.source.insert_input(1, LayoutInputOf::box_input(child));
    tree
}

fn intrinsic_preference_owns_percent_basis<S: LayoutScalar>(
    preferred: PreferredSizeOf<S>,
    expected: f64,
) {
    let tree = percent_subject::<S>(preferred);
    qualify_actual_native_queries(&tree);
    let available = Size::new(
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(
                crate::NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
            ),
            SizingOperation::Normal,
        ),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
    );
    let input = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(S::from_f64(100.0)), None),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        SizingRequestOf::new(
            available,
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    );
    assert_eq!(input.known(), Size::NONE);
    assert_eq!(input.sizing_request().axes(), available);
    let mut session = crate::engine::ComputeSession::new(&tree, Vec::new());
    let intrinsic = session
        .compute_child(0, input)
        .expect("intrinsic preferred Grid size is admitted with an external finite host");
    let expected_size = Size::new(S::from_f64(expected), S::from_f64(10.0));
    assert_eq!(
        intrinsic.size, expected_size,
        "ComputeSize uses the authored intrinsic preference, not external available100"
    );
    drop(session);

    // Grid 2 sections 7.2 and 12: treat the cyclic100% track as Auto to size the
    // container, then resolve it against that resulting40/80 container size.
    // The external100 host is not the percentage track's containing grid.
    let batch = compute_layout(&tree, 0, {
        let axes = available;
        let definite = axes.map(|axis| axis.space().definite_size());
        LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            definite,
        )
    })
    .expect("normal layout of the intrinsically preferred InlineGrid is admitted");
    for (phase, entries) in [
        ("unrounded", batch.unrounded_entries()),
        ("final", batch.final_entries()),
    ] {
        for node in [0, 1] {
            let output = entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap_or_else(|| panic!("{phase} node{node}: missing actual layout"))
                .output();
            assert_eq!(
                output.size, expected_size,
                "{phase} node{node}: actual normal container and100% track use intrinsic{expected}"
            );
            assert_eq!(
                output.location,
                Point::ZERO,
                "{phase} node{node}: normal placement"
            );
            assert_eq!(
                output.content_box_size(),
                expected_size,
                "{phase} node{node}: zero-edge content box"
            );
            assert_eq!(
                (output.border, output.padding, output.margin),
                (Edges::ZERO, Edges::ZERO, Edges::ZERO),
                "{phase} node{node}: source has no box edges"
            );
            assert_eq!(
                output.source_index,
                SourceIndex::ZERO,
                "{phase} node{node}: actual sole child rank"
            );
        }
    }
}

#[test]
fn preferred_min_grid_does_not_floor_cyclic_percent_track_to_external_host_f32() {
    intrinsic_preference_owns_percent_basis::<f32>(PreferredSizeOf::MIN_CONTENT, 40.0);
}
#[test]
fn preferred_min_grid_does_not_floor_cyclic_percent_track_to_external_host_f64() {
    intrinsic_preference_owns_percent_basis::<f64>(PreferredSizeOf::MIN_CONTENT, 40.0);
}
#[test]
fn preferred_max_grid_does_not_floor_cyclic_percent_track_to_external_host_f32() {
    intrinsic_preference_owns_percent_basis::<f32>(PreferredSizeOf::MAX_CONTENT, 80.0);
}
#[test]
fn preferred_max_grid_does_not_floor_cyclic_percent_track_to_external_host_f64() {
    intrinsic_preference_owns_percent_basis::<f64>(PreferredSizeOf::MAX_CONTENT, 80.0);
}
fn cyclic_percent_root_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let px = S::from_f64;
    let axes = Size::new(200.0, 100.0).map(|value| {
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(px(value)).unwrap()),
            SizingOperation::Normal,
        )
    });
    let definite = axes.map(|axis| axis.space().definite_size());
    LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            axes,
            OrthogonalFallbackOf::new(
                definite,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        definite,
    )
}

fn assert_intrinsic_grid_percent_with_flex_sibling<S: LayoutScalar>() {
    let px = S::from_f64;
    let fraction =
        |value| TrackComponentOf::flex(crate::TrackFlexFactorOf::try_new(px(value)).unwrap());
    let request = cyclic_percent_root_request::<S>();
    let mut differences = Vec::new();
    for (display, preferred, trailing, width, marker) in [
        (
            Display::InlineGrid,
            PreferredSizeOf::MAX_CONTENT,
            fraction(1.0),
            40.0,
            20.0,
        ),
        (
            Display::InlineGrid,
            PreferredSizeOf::MIN_CONTENT,
            fraction(1.0),
            40.0,
            20.0,
        ),
        (
            Display::InlineGrid,
            PreferredSizeOf::MAX_CONTENT,
            TrackComponentOf::px(S::ZERO),
            40.0,
            20.0,
        ),
        (
            Display::Grid,
            PreferredSizeOf::AUTO,
            fraction(1.0),
            200.0,
            100.0,
        ),
        (
            Display::InlineGrid,
            PreferredSizeOf::AUTO,
            fraction(0.5),
            150.0,
            75.0,
        ),
    ] {
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display,
                    position: Position::Relative,
                    size: Size::new(preferred.clone(), PreferredSizeOf::px(px(20.0))),
                    grid_template_columns: vec![TrackComponentOf::percent(px(0.5)), trailing],
                    grid_template_rows: vec![TrackComponentOf::px(px(20.0))],
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(PreferredSizeOf::px(px(40.0)), PreferredSizeOf::px(px(10.0))),
                    raw_grid_column: RawGridPlacement::lines(1, 2),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    position: Position::Absolute,
                    size: Size::ZERO.map(PreferredSizeOf::px),
                    raw_grid_column: RawGridPlacement::lines(2, 3),
                    inset: Edges::all(LengthAutoOf::ZERO),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(&tree, 0, request).unwrap();
        let output = |node| {
            batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
        };
        let actual = (output(0).size.width, output(2).location.x);
        let expected = (px(width), px(marker));
        if actual != expected {
            differences.push(format!(
                "{display:?} {preferred:?}: {actual:?}, expected {expected:?}"
            ));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn intrinsic_grid_percent_with_flex_sibling_f32() {
    assert_intrinsic_grid_percent_with_flex_sibling::<f32>();
}

#[test]
fn intrinsic_grid_percent_with_flex_sibling_f64() {
    assert_intrinsic_grid_percent_with_flex_sibling::<f64>();
}

fn assert_intrinsic_grid_row_percent_with_flex_sibling<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for (preferred, trailing, height, marker) in [
        (
            PreferredSizeOf::AUTO,
            TrackComponentOf::flex(crate::TrackFlexFactorOf::try_new(S::ONE).unwrap()),
            40.0,
            20.0,
        ),
        (
            PreferredSizeOf::AUTO,
            TrackComponentOf::px(S::ZERO),
            40.0,
            20.0,
        ),
        (
            PreferredSizeOf::px(px(100.0)),
            TrackComponentOf::flex(crate::TrackFlexFactorOf::try_new(S::ONE).unwrap()),
            100.0,
            50.0,
        ),
    ] {
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    position: Position::Relative,
                    size: Size::new(PreferredSizeOf::px(px(20.0)), preferred.clone()),
                    grid_template_columns: vec![TrackComponentOf::px(px(20.0))],
                    grid_template_rows: vec![TrackComponentOf::percent(px(0.5)), trailing],
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(PreferredSizeOf::px(px(10.0)), PreferredSizeOf::px(px(40.0))),
                    raw_grid_row: RawGridPlacement::lines(1, 2),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    position: Position::Absolute,
                    size: Size::ZERO.map(PreferredSizeOf::px),
                    raw_grid_row: RawGridPlacement::lines(2, 3),
                    inset: Edges::all(LengthAutoOf::ZERO),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(&tree, 0, cyclic_percent_root_request::<S>()).unwrap();
        let output = |node| {
            batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
        };
        let actual = (output(0).size.height, output(2).location.y);
        let expected = (px(height), px(marker));
        if actual != expected {
            differences.push(format!("{preferred:?}: {actual:?}, expected {expected:?}"));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn intrinsic_grid_row_percent_with_flex_sibling_f32() {
    assert_intrinsic_grid_row_percent_with_flex_sibling::<f32>();
}

#[test]
fn intrinsic_grid_row_percent_with_flex_sibling_f64() {
    assert_intrinsic_grid_row_percent_with_flex_sibling::<f64>();
}
