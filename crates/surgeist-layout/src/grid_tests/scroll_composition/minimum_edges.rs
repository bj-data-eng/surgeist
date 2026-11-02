// Private child of grid_tests/scroll_composition_tests.rs.
// Ordinary Grid shortage exposes the actual minimum contribution via the
// following fixed track, without treating opaque content as a minimum.
use super::*;
use crate::LayoutTree;

#[derive(Clone, Copy, Debug)]
enum Case {
    ExplicitMinimum,
    EdgesAndMargins,
    MinimumWithEdgesAndMargins,
}
impl Case {
    fn has_edges(self) -> bool {
        !matches!(self, Self::ExplicitMinimum)
    }
    fn has_explicit_minimum(self) -> bool {
        !matches!(self, Self::EdgesAndMargins)
    }
    fn expected_floor(self, overflow: Overflow) -> f64 {
        match (self, overflow) {
            (Self::ExplicitMinimum, Overflow::Hidden) => 12.0,
            (Self::EdgesAndMargins, Overflow::Hidden) => 17.0,
            (Self::MinimumWithEdgesAndMargins, Overflow::Hidden) => 29.0,
            (Self::EdgesAndMargins, Overflow::Clip) => 37.0,
            _ => unreachable!("the fixture defines only these four contribution cases"),
        }
    }
}
fn s<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn subject<S: LayoutScalar>(case: Case, overflow: Overflow) -> NodeInputOf<S> {
    let (padding, border, margin) = if case.has_edges() {
        (
            Edges {
                left: LengthOf::px(s(3.0)),
                right: LengthOf::px(s(2.0)),
                top: LengthOf::ZERO,
                bottom: LengthOf::ZERO,
            },
            Edges {
                left: LengthOf::px(s(2.0)),
                right: LengthOf::px(s(3.0)),
                top: LengthOf::ZERO,
                bottom: LengthOf::ZERO,
            },
            Edges {
                left: LengthAutoOf::px(s(3.0)),
                right: LengthAutoOf::px(s(4.0)),
                top: LengthAutoOf::ZERO,
                bottom: LengthAutoOf::ZERO,
            },
        )
    } else {
        (
            Edges::all(LengthOf::ZERO),
            Edges::all(LengthOf::ZERO),
            Edges::all(LengthAutoOf::ZERO),
        )
    };
    NodeInputOf {
        display: Display::Block,
        box_sizing: BoxSizing::ContentBox,
        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
        min_size: Size::new(
            if case.has_explicit_minimum() {
                MinSizeOf::px(s(12.0))
            } else {
                MinSizeOf::AUTO
            },
            MinSizeOf::AUTO,
        ),
        max_size: Size::new(MaxSizeOf::px(s(20.0)), MaxSizeOf::px(s(30.0))),
        padding,
        border,
        margin,
        overflow: computed_overflow(overflow, overflow),
        grid_column: GridPlacement::try_line(1).unwrap(),
        grid_row: GridPlacement::try_line(1).unwrap(),
        ..NodeInputOf::default()
    }
}
fn source<S: LayoutScalar>(case: Case, overflow: Overflow) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(PreferredSizeOf::px(s(5.0)), PreferredSizeOf::px(s(30.0))),
                grid_template_columns: vec![TrackComponentOf::AUTO, TrackComponentOf::px(s(1.0))],
                grid_template_rows: vec![TrackComponentOf::px(s(30.0))],
                gap: Size::new(LengthOf::ZERO, LengthOf::ZERO),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                justify_items: Some(AlignItems::Stretch),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(1, subject(case, overflow))
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(PreferredSizeOf::px(s(1.0)), PreferredSizeOf::px(s(1.0))),
                grid_column: GridPlacement::try_line(2).unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .measure(1, Size::new(s(80.0), s(90.0)))
}
fn qualify_leaf<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>, case: Case) {
    let style = tree.node_input(1);
    assert_eq!(style.box_sizing, BoxSizing::ContentBox);
    assert!(style.size.width.is_auto() && style.size.height.is_auto());
    assert!(tree.has_leaf_measurement(1));
    let output = crate::measurement::compute_tree_leaf(
        tree,
        1,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(s(5.0)), Some(s(30.0))),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::Grid,
            ),
            Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
        ),
        style,
    )
    .expect("actual native natural-size query succeeds with authored constraints and edges");
    // Content-box maximum20 plus padding5/border5 yields queried border30.
    // Native content_size includes measured80 plus padding5, not border5.
    let (border_width, content_width) = if case.has_edges() {
        (30.0, 85.0)
    } else {
        (20.0, 80.0)
    };
    assert_eq!(output.size, Size::new(s(border_width), s(30.0)));
    assert_eq!(output.content_size, Size::new(s(content_width), s(90.0)));
    assert_eq!(output.baselines(), BaselinesOf::NONE);
    assert_eq!(output.named_baselines, NamedBaselinesOf::unknown());
}
fn minimum_survives<S: LayoutScalar>(case: Case, overflow: Overflow) {
    let tree = source::<S>(case, overflow);
    qualify_leaf(&tree, case);
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(s(5.0)),
            AvailableOf::definite(s(30.0)),
        ))
        .unwrap(),
    )
    .expect("actual shortage Grid completes without erasing the native contribution");
    let root = fri08_c01_placement_output(&batch, 0);
    let marker = fri08_c01_placement_output(&batch, 2);
    let child = fri08_c01_placement_output(&batch, 1);
    assert_eq!(root.size, Size::new(s(5.0), s(30.0)));
    assert_eq!(root.location, Point::ZERO);
    let floor = case.expected_floor(overflow);
    // One AUTO track followed by a definite1px track, with no gap. This
    // actual marker origin observes the first track's used minimum floor.
    assert_eq!(
        marker.location,
        Point::new(s(floor), S::ZERO),
        "{case:?} {overflow:?}: zero auto content minimum must preserve actual minimum/PB/margin contribution"
    );
    assert_eq!(marker.size, Size::new(s(1.0), s(1.0)));
    let (start_margin, end_margin) = if case.has_edges() {
        (3.0, 4.0)
    } else {
        (0.0, 0.0)
    };
    let used_border = floor - start_margin - end_margin;
    assert_eq!(child.location, Point::new(s(start_margin), S::ZERO));
    assert_eq!(child.size, Size::new(s(used_border), s(30.0)));
    if case.has_edges() {
        assert_eq!(
            child.padding,
            Edges {
                left: s(3.0),
                right: s(2.0),
                top: S::ZERO,
                bottom: S::ZERO
            }
        );
        assert_eq!(
            child.border,
            Edges {
                left: s(2.0),
                right: s(3.0),
                top: S::ZERO,
                bottom: S::ZERO
            }
        );
        assert_eq!(
            child.margin,
            Edges {
                left: s(3.0),
                right: s(4.0),
                top: S::ZERO,
                bottom: S::ZERO
            }
        );
    } else {
        assert_eq!(
            (child.padding, child.border, child.margin),
            (Edges::ZERO, Edges::ZERO, Edges::ZERO)
        );
    }
    // No visual-overflow rectangle is inferred from the minimum floor. The
    // real natural80x90 producer and its query payload were qualified above.
}
#[test]
fn hidden_preserves_explicit_content_minimum_f32() {
    minimum_survives::<f32>(Case::ExplicitMinimum, Overflow::Hidden);
}
#[test]
fn hidden_preserves_explicit_content_minimum_f64() {
    minimum_survives::<f64>(Case::ExplicitMinimum, Overflow::Hidden);
}
#[test]
fn hidden_zero_auto_minimum_preserves_border_padding_margins_f32() {
    minimum_survives::<f32>(Case::EdgesAndMargins, Overflow::Hidden);
}
#[test]
fn hidden_zero_auto_minimum_preserves_border_padding_margins_f64() {
    minimum_survives::<f64>(Case::EdgesAndMargins, Overflow::Hidden);
}
#[test]
fn hidden_preserves_minimum_plus_border_padding_margins_f32() {
    minimum_survives::<f32>(Case::MinimumWithEdgesAndMargins, Overflow::Hidden);
}
#[test]
fn hidden_preserves_minimum_plus_border_padding_margins_f64() {
    minimum_survives::<f64>(Case::MinimumWithEdgesAndMargins, Overflow::Hidden);
}
#[test]
fn clip_content_based_auto_minimum_keeps_border_padding_margins_f32() {
    minimum_survives::<f32>(Case::EdgesAndMargins, Overflow::Clip);
}
#[test]
fn clip_content_based_auto_minimum_keeps_border_padding_margins_f64() {
    minimum_survives::<f64>(Case::EdgesAndMargins, Overflow::Clip);
}

// ROW counterparts: preserve the original eight COLUMN cases above.
fn row_source<S: LayoutScalar>(overflow: Overflow) -> PublicLayoutTreeOf<S> {
    let minimum = match overflow {
        Overflow::Hidden => MinSizeOf::px(s(12.0)),
        Overflow::Clip => MinSizeOf::AUTO,
        _ => unreachable!("the row qualifier selects Hidden and Clip"),
    };
    PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(PreferredSizeOf::px(s(30.0)), PreferredSizeOf::px(s(5.0))),
                grid_template_columns: vec![TrackComponentOf::px(s(30.0))],
                grid_template_rows: vec![TrackComponentOf::AUTO, TrackComponentOf::px(s(1.0))],
                gap: Size::new(LengthOf::ZERO, LengthOf::ZERO),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                justify_items: Some(AlignItems::Start),
                align_items: Some(AlignItems::Stretch),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                box_sizing: BoxSizing::ContentBox,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                min_size: Size::new(MinSizeOf::AUTO, minimum),
                max_size: Size::new(MaxSizeOf::px(s(30.0)), MaxSizeOf::px(s(20.0))),
                padding: Edges {
                    top: LengthOf::px(s(3.0)),
                    bottom: LengthOf::px(s(2.0)),
                    left: LengthOf::ZERO,
                    right: LengthOf::ZERO,
                },
                border: Edges {
                    top: LengthOf::px(s(2.0)),
                    bottom: LengthOf::px(s(3.0)),
                    left: LengthOf::ZERO,
                    right: LengthOf::ZERO,
                },
                margin: Edges {
                    top: LengthAutoOf::px(s(3.0)),
                    bottom: LengthAutoOf::px(s(4.0)),
                    left: LengthAutoOf::ZERO,
                    right: LengthAutoOf::ZERO,
                },
                overflow: computed_overflow(overflow, overflow),
                grid_column: GridPlacement::try_line(1).unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(PreferredSizeOf::px(s(1.0)), PreferredSizeOf::px(s(1.0))),
                grid_column: GridPlacement::try_line(1).unwrap(),
                grid_row: GridPlacement::try_line(2).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .measure(1, Size::new(s(90.0), s(80.0)))
}
fn qualify_row_leaf<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>) {
    let style = tree.node_input(1);
    assert_eq!(style.box_sizing, BoxSizing::ContentBox);
    assert!(style.size.width.is_auto() && style.size.height.is_auto());
    assert!(tree.has_leaf_measurement(1));
    let output = crate::measurement::compute_tree_leaf(
        tree,
        1,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(s(30.0)), Some(s(5.0))),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::Grid,
            ),
            Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
        ),
        style,
    )
    .expect("actual row-oriented native query preserves constrained border and natural content");
    // Width90 is capped at30. Height80 is capped at content20; padding5
    // plus border5 yields border30, while content_size is80+padding5=85.
    assert_eq!(output.size, Size::new(s(30.0), s(30.0)));
    assert_eq!(output.content_size, Size::new(s(90.0), s(85.0)));
    assert_eq!(output.baselines(), BaselinesOf::NONE);
    assert_eq!(output.named_baselines, NamedBaselinesOf::unknown());
}
fn row_minimum_survives<S: LayoutScalar>(overflow: Overflow) {
    let tree = row_source::<S>(overflow);
    qualify_row_leaf(&tree);
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(s(30.0)),
            AvailableOf::definite(s(5.0)),
        ))
        .unwrap(),
    )
    .expect("actual row-shortage Grid completes");
    let root = fri08_c01_placement_output(&batch, 0);
    let marker = fri08_c01_placement_output(&batch, 2);
    let child = fri08_c01_placement_output(&batch, 1);
    let (floor, border_height) = match overflow {
        // Explicit CONTENT minimum12 survives Hidden's zero AUTO minimum.
        // 12+padding5+border5+margin7=29; subject border12+10=22.
        Overflow::Hidden => (29.0, 22.0),
        // Clip's content-based automatic minimum caps at max-content20:
        // 20+padding5+border5+margin7=37; subject border20+10=30.
        Overflow::Clip => (37.0, 30.0),
        _ => unreachable!("the row qualifier selects Hidden and Clip"),
    };
    assert_eq!(root.location, Point::ZERO);
    assert_eq!(root.size, Size::new(s(30.0), s(5.0)));
    assert_eq!(
        marker.location,
        Point::new(S::ZERO, s(floor)),
        "{overflow:?}: actual second-row origin must preserve the first row's own minimum role"
    );
    assert_eq!(marker.size, Size::new(s(1.0), s(1.0)));
    assert_eq!(child.location, Point::new(S::ZERO, s(3.0)));
    assert_eq!(child.size, Size::new(s(30.0), s(border_height)));
    assert_eq!(
        child.padding,
        Edges {
            top: s(3.0),
            bottom: s(2.0),
            left: S::ZERO,
            right: S::ZERO
        }
    );
    assert_eq!(
        child.border,
        Edges {
            top: s(2.0),
            bottom: s(3.0),
            left: S::ZERO,
            right: S::ZERO
        }
    );
    assert_eq!(
        child.margin,
        Edges {
            top: s(3.0),
            bottom: s(4.0),
            left: S::ZERO,
            right: S::ZERO
        }
    );
    // No normal/visual overflow hull or minimum is borrowed from the opposite
    // axis. The definite30px column is an actual authored receiving basis.
}
#[test]
fn hidden_row_preserves_explicit_minimum_padding_border_margins_f32() {
    row_minimum_survives::<f32>(Overflow::Hidden);
}
#[test]
fn hidden_row_preserves_explicit_minimum_padding_border_margins_f64() {
    row_minimum_survives::<f64>(Overflow::Hidden);
}
#[test]
fn clip_row_preserves_content_auto_minimum_padding_border_margins_f32() {
    row_minimum_survives::<f32>(Overflow::Clip);
}
#[test]
fn clip_row_preserves_content_auto_minimum_padding_border_margins_f64() {
    row_minimum_survives::<f64>(Overflow::Clip);
}
