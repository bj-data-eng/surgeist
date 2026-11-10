//! Minimum sizes win over smaller maximum sizes for Grid items and for
//! absolutely positioned children of a Grid container.
//!
//! CSS 2.1 section 10.4 applies the minimum after the maximum, and CSS Sizing 3
//! section 3.1 states that the minimum size constraint is always the strongest
//! constraint. Block and Flex parents are controls for the same declarations.

use super::fixtures::{fri08_c01_placement_compute, fri08_c01_placement_output};
use super::*;

#[derive(Clone, Copy, Debug)]
enum Axis {
    Width,
    Height,
}

fn container<S: LayoutScalar>(display: Display, width: f64, height: f64) -> NodeInputOf<S> {
    let px = S::from_f64;
    NodeInputOf {
        display,
        size: Size::new(
            PreferredSizeOf::px(px(width)),
            PreferredSizeOf::px(px(height)),
        ),
        grid_template_columns: vec![TrackComponentOf::px(px(width))],
        grid_template_rows: vec![TrackComponentOf::px(px(height))],
        ..NodeInputOf::default()
    }
}

fn child_size<S: LayoutScalar>(
    display: Display,
    width: f64,
    height: f64,
    child: NodeInputOf<S>,
) -> Size<S> {
    let tree = PublicLayoutTreeOf::new()
        .children(1, [2])
        .children(2, [])
        .style(1, container(display, width, height))
        .style(2, child);
    let batch = fri08_c01_placement_compute(&tree);
    fri08_c01_placement_output(&batch, 2).size
}

fn used(size: Size<impl LayoutScalar>, axis: Axis) -> f64 {
    match axis {
        Axis::Width => size.width.to_f64(),
        Axis::Height => size.height.to_f64(),
    }
}

// The constrained axis carries `preferred`, a minimum of 100px and a maximum of
// 50px; the other axis is fixed at 10px.
fn item<S: LayoutScalar>(
    axis: Axis,
    preferred: PreferredSizeOf<S>,
    alignment: Option<AlignItems>,
) -> NodeInputOf<S> {
    let px = S::from_f64;
    let fixed = PreferredSizeOf::px(px(10.0));
    let (size, min_size, max_size) = match axis {
        Axis::Width => (
            Size::new(preferred, fixed),
            Size::new(MinSizeOf::px(px(100.0)), MinSizeOf::AUTO),
            Size::new(MaxSizeOf::px(px(50.0)), MaxSizeOf::NONE),
        ),
        Axis::Height => (
            Size::new(fixed, preferred),
            Size::new(MinSizeOf::AUTO, MinSizeOf::px(px(100.0))),
            Size::new(MaxSizeOf::NONE, MaxSizeOf::px(px(50.0))),
        ),
    };
    NodeInputOf {
        size,
        min_size,
        max_size,
        justify_self: alignment,
        align_self: alignment,
        ..NodeInputOf::default()
    }
}

fn assert_item_minimum_wins<S: LayoutScalar>(display: Display) {
    let px = S::from_f64;
    let mut failures = Vec::new();
    for axis in [Axis::Width, Axis::Height] {
        for alignment in [None, Some(AlignItems::Start), Some(AlignItems::Stretch)] {
            for (label, preferred) in [
                ("auto", PreferredSizeOf::AUTO),
                ("75px", PreferredSizeOf::px(px(75.0))),
            ] {
                let size = child_size::<S>(display, 200.0, 200.0, item(axis, preferred, alignment));
                let actual = used(size, axis);
                if actual != 100.0 {
                    failures.push(format!(
                        "{axis:?} alignment {alignment:?} preferred {label}: used {actual}"
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{display:?} parent, expected 100 for minimum 100 over maximum 50: {}",
        failures.join("; ")
    );
}

#[test]
fn grid_item_minimum_size_exceeding_maximum_size_wins_f32() {
    assert_item_minimum_wins::<f32>(Display::Grid);
}

#[test]
fn grid_item_minimum_size_exceeding_maximum_size_wins_f64() {
    assert_item_minimum_wins::<f64>(Display::Grid);
}

#[test]
fn block_child_minimum_size_exceeding_maximum_size_wins_f32() {
    assert_item_minimum_wins::<f32>(Display::Block);
}

#[test]
fn block_child_minimum_size_exceeding_maximum_size_wins_f64() {
    assert_item_minimum_wins::<f64>(Display::Block);
}

#[test]
fn flex_item_minimum_size_exceeding_maximum_size_wins_f32() {
    assert_item_minimum_wins::<f32>(Display::Flex);
}

#[test]
fn flex_item_minimum_size_exceeding_maximum_size_wins_f64() {
    assert_item_minimum_wins::<f64>(Display::Flex);
}

// Border-box absolutely positioned child with no insets and auto placement:
// the preferred size is 150px, the minimum 200px and the maximum 100px on the
// constrained axis; the other axis is fixed at 10px.
fn positioned<S: LayoutScalar>(axis: Axis) -> NodeInputOf<S> {
    let px = S::from_f64;
    let fixed = PreferredSizeOf::px(px(10.0));
    let preferred = PreferredSizeOf::px(px(150.0));
    let (size, min_size, max_size) = match axis {
        Axis::Width => (
            Size::new(preferred, fixed),
            Size::new(MinSizeOf::px(px(200.0)), MinSizeOf::AUTO),
            Size::new(MaxSizeOf::px(px(100.0)), MaxSizeOf::NONE),
        ),
        Axis::Height => (
            Size::new(fixed, preferred),
            Size::new(MinSizeOf::AUTO, MinSizeOf::px(px(200.0))),
            Size::new(MaxSizeOf::NONE, MaxSizeOf::px(px(100.0))),
        ),
    };
    NodeInputOf {
        position: Position::Absolute,
        box_sizing: BoxSizing::BorderBox,
        size,
        min_size,
        max_size,
        ..NodeInputOf::default()
    }
}

fn assert_positioned_minimum_wins<S: LayoutScalar>(display: Display) {
    let mut failures = Vec::new();
    for axis in [Axis::Width, Axis::Height] {
        let size = child_size::<S>(display, 400.0, 300.0, positioned(axis));
        let actual = used(size, axis);
        if actual != 200.0 {
            failures.push(format!("{axis:?}: used {actual}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{display:?} parent, expected 200 for minimum 200 over maximum 100: {}",
        failures.join("; ")
    );
}

#[test]
fn grid_positioned_child_minimum_size_exceeding_maximum_size_wins_f32() {
    assert_positioned_minimum_wins::<f32>(Display::Grid);
}

#[test]
fn grid_positioned_child_minimum_size_exceeding_maximum_size_wins_f64() {
    assert_positioned_minimum_wins::<f64>(Display::Grid);
}

#[test]
fn block_positioned_child_minimum_size_exceeding_maximum_size_wins_f32() {
    assert_positioned_minimum_wins::<f32>(Display::Block);
}

#[test]
fn block_positioned_child_minimum_size_exceeding_maximum_size_wins_f64() {
    assert_positioned_minimum_wins::<f64>(Display::Block);
}

#[test]
fn flex_positioned_child_minimum_size_exceeding_maximum_size_wins_f32() {
    assert_positioned_minimum_wins::<f32>(Display::Flex);
}

#[test]
fn flex_positioned_child_minimum_size_exceeding_maximum_size_wins_f64() {
    assert_positioned_minimum_wins::<f64>(Display::Flex);
}
