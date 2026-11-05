use super::*;

struct PlacedStandalone<S: LayoutScalar> {
    subgrid: NodeOutputOf<S>,
    standalone: NodeOutputOf<S>,
    leaf: NodeOutputOf<S>,
    synthesized_x: S,
}

fn normal_box<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> NodeOutputOf<S> {
    let normal = batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .expect("the source node has an accepted normal output")
        .output();
    let published = fri08_c01_placement_output(batch, node);
    assert_eq!(published.location, normal.location);
    assert_eq!(published.size, normal.size);
    normal
}

fn qualify<S: LayoutScalar>(
    source: &Fri08C04BaselineTree<S>,
    case: Fri08C04BaselineFlowCase,
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> PlacedStandalone<S> {
    let s = S::from_f64;
    let root_flow = FlowAxes::new(case.root_writing_mode, case.root_direction);
    let child_flow = FlowAxes::new(case.child_writing_mode, case.child_direction);
    let subgrid = normal_box(batch, 3);
    let standalone = normal_box(batch, 5);
    let leaf = normal_box(batch, 7);
    assert_eq!(
        source.measurements.get(&7),
        Some(&Size::new(s(12.0), s(18.0)))
    );
    assert!(!source.measurements.contains_key(&3));
    assert!(!source.measurements.contains_key(&5));
    assert_eq!(
        source.tree.node_input(5).align_self,
        Some(AlignItems::LastBaseline)
    );

    // Authored parent rows40/40 plus gap10 span90. In the150-wide
    // decreasing block frame they occupy60..150. The second60-high
    // inline track starts at72 (or is mirrored within the220-high root).
    let subgrid_y = if root_flow
        .physical_axis_progression(PhysicalAxis::Vertical)
        .is_decreasing()
    {
        s(220.0 - 72.0 - 60.0)
    } else {
        s(72.0)
    };
    assert_eq!(subgrid.location, Point::new(s(60.0), subgrid_y));
    assert_eq!(subgrid.size, Size::new(s(90.0), s(60.0)));
    assert_eq!(standalone.size, Size::new(s(12.0), s(18.0)));
    assert_eq!(leaf.location, Point::ZERO);
    assert_eq!(leaf.size, Size::new(s(12.0), s(18.0)));
    let standalone_y = if child_flow
        .physical_axis_progression(PhysicalAxis::Vertical)
        .is_decreasing()
    {
        s(60.0 - 18.0)
    } else {
        S::ZERO
    };
    assert_eq!(standalone.location.y, standalone_y);

    // The maintained host supplies only a Size, not baseline coordinates.
    // Inspect the actual accepted leaf query before synthesizing its port.
    let mut performed = 0;
    for receipt in batch
        .cache_store_entries()
        .iter()
        .filter(|entry| entry.node() == 7 && entry.input().run_mode() == RunMode::PerformLayout)
    {
        let output = receipt.output();
        assert_eq!(output.size, leaf.size);
        assert_eq!(output.first_baselines, Point::NONE);
        assert_eq!(output.last_baselines, Point::NONE);
        assert_eq!(output.baselines().last_block(child_flow), None);
        let synthesized = output
            .baselines()
            .last_or_synthesize_block_baseline(child_flow, output.size);
        let expected_x = match case.child_writing_mode {
            WritingMode::VerticalLr => S::ZERO,
            WritingMode::SidewaysLr => s(12.0),
            _ => unreachable!("the two real profiles have specified line-under sides"),
        };
        assert_eq!(
            synthesized.coordinate_on(PhysicalAxis::Horizontal),
            Some(expected_x)
        );
        performed += 1;
    }
    assert!(performed > 0, "the real performed leaf query was accepted");
    let synthesized_x = match case.child_writing_mode {
        WritingMode::VerticalLr => S::ZERO,
        WritingMode::SidewaysLr => s(12.0),
        _ => unreachable!("qualified above"),
    };
    PlacedStandalone {
        subgrid,
        standalone,
        leaf,
        synthesized_x,
    }
}

fn assert_same_used_border_preserves_last_baseline<S: LayoutScalar>(
    root_writing_mode: WritingMode,
    root_direction: Direction,
    child_writing_mode: WritingMode,
    child_direction: Direction,
) {
    let s = S::from_f64;
    let case = Fri08C04BaselineFlowCase {
        parent_axis: Fri08C04BaselineParentAxis::Row,
        root_writing_mode,
        root_direction,
        child_writing_mode,
        child_direction,
    };
    let auto = fri08_c04_baseline_area_implicit_tree::<S>(case, AlignItems::LastBaseline);
    assert!(auto.tree.node_input(5).size.width.is_auto());
    let mut explicit = auto.clone();
    let mut style = explicit.tree.node_input(5).clone();
    style.size.width = PreferredSizeOf::px(s(12.0));
    explicit
        .tree
        .insert_input(5, crate::LayoutInputOf::box_input(style));
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
    let auto_batch = compute_layout(&auto, 1, request).expect("Auto standalone computes");
    let explicit_batch =
        compute_layout(&explicit, 1, request).expect("equivalent definite standalone computes");
    let auto = qualify(&auto, case, &auto_batch);
    let explicit = qualify(&explicit, case, &explicit_batch);
    assert_eq!(auto.subgrid.location, explicit.subgrid.location);
    assert_eq!(auto.subgrid.size, explicit.subgrid.size);
    assert_eq!(auto.standalone.size, explicit.standalone.size);
    assert_eq!(auto.leaf.location, explicit.leaf.location);
    assert_eq!(auto.leaf.size, explicit.leaf.size);
    let point = |value: &PlacedStandalone<S>| {
        value.subgrid.location.x
            + value.standalone.location.x
            + value.leaf.location.x
            + value.synthesized_x
    };

    // Changing Auto to its actual used12-wide border changes neither its
    // selected Last port nor its containing frame. Check that invariance
    // BEFORE the absolute self-end oracle, including on the older geometry.
    assert_eq!(
        point(&auto),
        point(&explicit),
        "equivalent actual normal border/port facts preserve the physical Last point"
    );
    assert_eq!(
        auto.standalone.location.x, explicit.standalone.location.x,
        "an opposite-axis query must not donate a different border width to the cohort"
    );

    // Parent40/40,gap10 becomes actual child35/35,gap20: row2 is55..90.
    // Its sole compatible Last member uses safe self-end (Align3§9.3).
    // A12-wide border therefore starts78; line-under is0(VLR) or12(SLR).
    assert_eq!(explicit.standalone.location.x, s(78.0));
    let expected_world = match child_writing_mode {
        WritingMode::VerticalLr => s(138.0),
        WritingMode::SidewaysLr => s(150.0),
        _ => unreachable!("the two qualified profiles"),
    };
    assert_eq!(point(&explicit), expected_world);
    assert_eq!(point(&auto), expected_world);
}

#[test]
fn auto_and_used_width_preserve_vlr_last_baseline_f32() {
    assert_same_used_border_preserves_last_baseline::<f32>(
        WritingMode::VerticalRl,
        Direction::Ltr,
        WritingMode::VerticalLr,
        Direction::Rtl,
    );
}

#[test]
fn auto_and_used_width_preserve_vlr_last_baseline_f64() {
    assert_same_used_border_preserves_last_baseline::<f64>(
        WritingMode::VerticalRl,
        Direction::Ltr,
        WritingMode::VerticalLr,
        Direction::Rtl,
    );
}

#[test]
fn auto_and_used_width_preserve_sideways_lr_last_baseline_f32() {
    assert_same_used_border_preserves_last_baseline::<f32>(
        WritingMode::SidewaysRl,
        Direction::Rtl,
        WritingMode::SidewaysLr,
        Direction::Ltr,
    );
}

#[test]
fn auto_and_used_width_preserve_sideways_lr_last_baseline_f64() {
    assert_same_used_border_preserves_last_baseline::<f64>(
        WritingMode::SidewaysRl,
        Direction::Rtl,
        WritingMode::SidewaysLr,
        Direction::Ltr,
    );
}
