//! Effective fixed max-track caps include a definite minimum that exceeds max.
use super::*;

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn body_style<S: LayoutScalar>(overflow: Overflow) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Flex,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        overflow: computed_overflow(overflow, overflow),
        ..NodeInputOf::default()
    }
}

fn fixed_descendant<S: LayoutScalar>() -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(n(120.0)), PreferredSizeOf::px(n(10.0))),
        flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
        ..NodeInputOf::default()
    }
}

fn output<S: LayoutScalar>(entries: &[LayoutOutputEntryOf<u32, S>], node: u32) -> NodeOutputOf<S> {
    let mut entries = entries.iter().filter(|entry| entry.node() == node);
    let result = entries
        .next()
        .expect("the real native box is published")
        .output();
    assert!(entries.next().is_none(), "one native box for node{node}");
    result
}

fn qualify_native_content<S: LayoutScalar>(overflow: Overflow) {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [3])
        .children(3, [])
        .style(1, body_style(overflow))
        .style(3, fixed_descendant());
    assert!(tree.node_input(1).size.width.is_auto());
    assert!(!tree.has_leaf_measurement(1));
    assert!(!tree.has_leaf_measurement(3));
    for available in [AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT] {
        let batch = compute_layout(
            &tree,
            1,
            LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::Definite(n(10.0))))
                .unwrap(),
        )
        .expect("the real native Flex intrinsic control completes");
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(output(entries, 1).size, Size::new(n(120.0), n(10.0)));
            assert_eq!(output(entries, 3).size, Size::new(n(120.0), n(10.0)));
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Constraint {
    Definite,
    MinContent,
}

fn grid<S: LayoutScalar>(
    overflow: Overflow,
    constraint: Constraint,
    normalize_first: bool,
) -> PublicLayoutTreeOf<S> {
    let first = if normalize_first {
        TrackComponentOf::px(n(100.0))
    } else {
        TrackComponentOf::minmax(
            MinTrackSizingOf::px(n(100.0)),
            MaxTrackSizingOf::px(n(10.0)),
        )
    };
    PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2])
        .children(1, [3])
        .children(2, [])
        .children(3, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                size: Size::new(
                    match constraint {
                        Constraint::Definite => PreferredSizeOf::px(n(100.0)),
                        Constraint::MinContent => PreferredSizeOf::MIN_CONTENT,
                    },
                    PreferredSizeOf::px(n(20.0)),
                ),
                grid_template_columns: vec![
                    first,
                    TrackComponentOf::minmax(MinTrackSizingOf::AUTO, MaxTrackSizingOf::px(n(10.0))),
                ],
                grid_template_rows: vec![TrackComponentOf::px(n(10.0)); 2],
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                ..body_style(overflow)
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Flex,
                grid_column: GridPlacement::try_lines(2, 3).unwrap(),
                grid_row: GridPlacement::try_lines(2, 3).unwrap(),
                justify_self: Some(AlignItems::Stretch),
                align_self: Some(AlignItems::Stretch),
                ..NodeInputOf::default()
            },
        )
        .style(3, fixed_descendant())
}

fn assert_caps<S: LayoutScalar>(
    overflow: Overflow,
    constraint: Constraint,
    second: f64,
    root_width: f64,
) {
    qualify_native_content::<S>(overflow);
    let mut equivalent_packets = None;
    // Qualify the explicitly fixed100 control before the equivalent minmax form.
    for normalize_first in [true, false] {
        let tree = grid::<S>(overflow, constraint, normalize_first);
        for node in [0, 1, 2, 3] {
            assert!(
                !tree.has_leaf_measurement(node),
                "every box uses native layout"
            );
        }
        assert_eq!(Traverse::children(&tree, 0).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(Traverse::children(&tree, 1).collect::<Vec<_>>(), [3]);
        assert!(tree.node_input(1).size.width.is_auto());
        assert!(tree.node_input(1).size.height.is_auto());
        assert!(tree.node_input(2).size.width.is_auto());
        let available = match constraint {
            Constraint::Definite => AvailableOf::Definite(n(100.0)),
            Constraint::MinContent => AvailableOf::MIN_CONTENT,
        };
        let request =
            LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::Definite(n(20.0))))
                .unwrap();
        assert_eq!(request.available().width, available);
        let batch =
            compute_layout(&tree, 0, request).expect("the native capped-span grid completes");
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(output(entries, 0).size, Size::new(n(root_width), n(20.0)));
            assert_eq!(output(entries, 1).location, Point::ZERO);
            assert_eq!(
                output(entries, 1).size,
                Size::new(n(100.0 + second), n(10.0))
            );
            assert_eq!(output(entries, 2).location, Point::new(n(100.0), n(10.0)));
            assert_eq!(output(entries, 2).size, Size::new(n(second), n(10.0)));
            assert_eq!(output(entries, 3).location, Point::ZERO);
            assert_eq!(output(entries, 3).size, Size::new(n(120.0), n(10.0)));
            for node in [0, 1, 2, 3] {
                let actual = output(entries, node);
                assert_eq!(
                    (actual.margin, actual.border, actual.padding),
                    (Edges::ZERO, Edges::ZERO, Edges::ZERO)
                );
            }
        }
        let packets = [
            output(batch.unrounded_entries(), 1),
            output(batch.unrounded_entries(), 2),
        ];
        if let Some(original) = equivalent_packets {
            assert_eq!(
                packets, original,
                "effective minmax(100,10) equals fixed100"
            );
        } else {
            equivalent_packets = Some(packets);
        }
    }
}

fn visible_minimum<S: LayoutScalar>() {
    // Grid1 §§7.2.1/6.6: floor first max10 to min100, then cap120 by100+10.
    assert_caps::<S>(Overflow::Visible, Constraint::Definite, 10.0, 100.0);
}

fn hidden_definite<S: LayoutScalar>() {
    // Hidden makes the used automatic minimum zero; fixed100 consumes all space.
    assert_caps::<S>(Overflow::Hidden, Constraint::Definite, 0.0, 100.0);
}

fn hidden_intrinsic<S: LayoutScalar>() {
    // Grid1 §11.5: formal min-content sizing uses the limited content120,
    // capped by effective fixed maxima110, independently of used minimum0.
    assert_caps::<S>(Overflow::Hidden, Constraint::MinContent, 10.0, 110.0);
}

#[test]
fn native_minmax_fixed_min_floors_visible_minimum_cap_f32() {
    visible_minimum::<f32>();
}
#[test]
fn native_minmax_fixed_min_floors_visible_minimum_cap_f64() {
    visible_minimum::<f64>();
}
#[test]
fn native_minmax_hidden_definite_zero_minimum_control_f32() {
    hidden_definite::<f32>();
}
#[test]
fn native_minmax_hidden_definite_zero_minimum_control_f64() {
    hidden_definite::<f64>();
}
#[test]
fn native_minmax_fixed_min_floors_hidden_intrinsic_cap_f32() {
    hidden_intrinsic::<f32>();
}
#[test]
fn native_minmax_fixed_min_floors_hidden_intrinsic_cap_f64() {
    hidden_intrinsic::<f64>();
}
