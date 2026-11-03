//! Visual content corners remain authoritative when border extents are unchanged.
use super::*;

fn retained_content_follows_actual_visual_corners<S: LayoutScalar>() {
    let nominal = surgeist_text::Size::new(10.0, 10.0);
    let paragraph = fractional_atomic_paragraph(nominal, false);
    for left in [0.0, 0.32] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(0, root_style(Direction::Ltr))
            .input(1, paragraph.text_input(1))
            .style(
                2,
                NodeInputOf {
                    display: Display::InlineBlock,
                    position: Position::Relative,
                    vertical_align: VerticalAlignOf::Baseline,
                    box_sizing: BoxSizing::BorderBox,
                    size: Size::new(
                        PreferredSizeOf::px(px::<S>(10.0)),
                        PreferredSizeOf::px(px::<S>(10.0)),
                    ),
                    border: Edges::all(LengthOf::px(px::<S>(0.4))),
                    inset: Edges {
                        left: LengthAutoOf::px(px::<S>(left)),
                        ..Edges::all(LengthAutoOf::AUTO)
                    },
                    ..NodeInputOf::default()
                },
            );
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![paragraph.binding(0)],
            crate::WrappingInput {
                native_inline_boxes: Vec::new(),
                gaps: Vec::new(),
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(
            &capture,
            0,
            crate::test_support::requests::NativeRootFrameOf::new(
                FlowAxes::new(
                    tree.node_input(0).writing_mode,
                    tree.node_input(0).direction,
                ),
                Size::new(Some(NonNegativeFiniteOf::new(px::<S>(50.0)).unwrap()), None),
                OrthogonalFallbackOf::new(
                    Size::new(Some(NonNegativeFiniteOf::new(px::<S>(50.0)).unwrap()), None),
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            )
            .request(Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(px::<S>(50.0)).unwrap()),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
        .unwrap();
        fractional_atomic_source_and_font(&batch, false);
        assert_eq!(
            output(&batch, 0).size,
            Size::new(px::<S>(50.0), px::<S>(12.0))
        );
        assert_eq!(batch.placed_text()[0].layout().lines()[0].baseline(), 10.0);
        let node = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 2)
            .unwrap()
            .output();
        assert_eq!(node.location, Point::new(px::<S>(10.0), S::ZERO));
        assert_eq!(node.size, Size::splat(px::<S>(10.0)));
        fractional_atomic_visual_agreement(&batch);
        let source = batch
            .unrounded_inline_fragments()
            .iter()
            .find(|entry| *entry.owner() == crate::InlineFragmentOwner::Node(1))
            .unwrap()
            .fragment();
        let line = source.finalized_fragment().finalized_line();
        assert_eq!(line.feedback().baseline, 10.0);
        let slot = line.atomic_slots().next().unwrap();
        assert_eq!(
            slot.occurrence,
            paragraph.prepared.atomic_views().next().unwrap().handle()
        );
        let surgeist_text::AtomicLineAllocation::InFlow { box_geometry, .. } = slot.allocation
        else {
            panic!("actual captured atomic remains in flow");
        };
        // Border10..20 remains10px after translation. With left.32 its raw
        // inner endpoints10.72..19.92 snap11..20: content x11,width9.
        // At left0,10.4..19.6 snap10..20: content x10,width10.
        // Vertical content.4..9.6 snaps0..10 in both cases.
        let expected_x = if left == 0.0 { 10.0 } else { 11.0 };
        let expected_width = if left == 0.0 { 10.0 } else { 9.0 };
        assert_eq!(slot.origin.x + box_geometry.content.origin.x, expected_x);
        assert_eq!(slot.origin.y + box_geometry.content.origin.y, 0.0);
        assert_eq!(
            box_geometry.content.size,
            surgeist_text::Size::new(expected_width, 10.0)
        );
        let content = node.scroll_geometry.unwrap().content_box();
        assert_eq!(
            Point::new(
                node.location.x + content.origin().x,
                node.location.y + content.origin().y
            ),
            Point::new(px::<S>(expected_x), S::ZERO)
        );
        assert_eq!(
            content.size(),
            Size::new(px::<S>(expected_width), px::<S>(10.0))
        );
    }
}

#[test]
fn unchanged_visual_border_extent_retains_actual_content_corners_f32() {
    retained_content_follows_actual_visual_corners::<f32>();
}
#[test]
fn unchanged_visual_border_extent_retains_actual_content_corners_f64() {
    retained_content_follows_actual_visual_corners::<f64>();
}
