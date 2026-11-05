//! Ordinary block-size percentages do not resolve against a content-derived auto block size.
//! The percentage owner is a real Block; only its Auto-sized descendant is measured.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;

fn scalar<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn physical<S: LayoutScalar>(mode: WritingMode, inline: f64, block: f64) -> Size<S> {
    match mode {
        WritingMode::HorizontalTb => Size::new(scalar(inline), scalar(block)),
        WritingMode::VerticalRl | WritingMode::VerticalLr => {
            Size::new(scalar(block), scalar(inline))
        }
        _ => unreachable!("the fixture authors only horizontal and upright vertical flow"),
    }
}

fn layout<S: LayoutScalar>(
    mode: WritingMode,
    parent_inline: Option<f64>,
    parent_block: Option<f64>,
    percentage_child: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let inline = parent_inline.map_or(PreferredSizeOf::AUTO, |value| {
        PreferredSizeOf::px(scalar(value))
    });
    let block = parent_block.map_or(PreferredSizeOf::AUTO, |value| {
        PreferredSizeOf::px(scalar(value))
    });
    let child_inline = PreferredSizeOf::px(scalar(20.0));
    let child_block = if percentage_child {
        PreferredSizeOf::value(LengthPercentageOf::from_percent_fraction(scalar(0.5)).unwrap())
    } else {
        PreferredSizeOf::AUTO
    };
    let (root_size, child_size) = match mode {
        WritingMode::HorizontalTb => (
            Size::new(inline, block),
            Size::new(child_inline, child_block),
        ),
        WritingMode::VerticalRl | WritingMode::VerticalLr => (
            Size::new(block, inline),
            Size::new(child_block, child_inline),
        ),
        _ => unreachable!("the fixture authors only horizontal and upright vertical flow"),
    };
    let host_inline = parent_inline.map(|value| NonNegativeFiniteOf::new(scalar(value)).unwrap());
    let inline_request = AxisSizingRequestOf::new(
        host_inline.map_or(AvailableSpaceOf::Indefinite, AvailableSpaceOf::Definite),
        SizingOperation::Normal,
    );
    let block_request =
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal);
    let (requests, percentage_parent) = match mode {
        WritingMode::HorizontalTb => (
            Size::new(inline_request, block_request),
            Size::new(host_inline, None),
        ),
        WritingMode::VerticalRl | WritingMode::VerticalLr => (
            Size::new(block_request, inline_request),
            Size::new(None, host_inline),
        ),
        _ => unreachable!("the fixture authors only horizontal and upright vertical flow"),
    };
    let frame = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(mode, Direction::Ltr),
        percentage_parent,
        OrthogonalFallbackOf::new(
            percentage_parent,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    );
    let root = NodeInputOf {
        display: Display::Block,
        writing_mode: mode,
        direction: Direction::Ltr,
        size: root_size,
        ..NodeInputOf::default()
    };
    let child = NodeInputOf {
        display: Display::Block,
        writing_mode: mode,
        direction: Direction::Ltr,
        size: child_size,
        ..NodeInputOf::default()
    };
    let leaf = NodeInputOf {
        display: Display::Block,
        writing_mode: mode,
        direction: Direction::Ltr,
        size: match mode {
            WritingMode::HorizontalTb => {
                Size::new(PreferredSizeOf::px(scalar(20.0)), PreferredSizeOf::AUTO)
            }
            WritingMode::VerticalRl | WritingMode::VerticalLr => {
                Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(20.0)))
            }
            _ => unreachable!("the fixture authors only horizontal and upright vertical flow"),
        },
        ..NodeInputOf::default()
    };
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(0, root)
        .style(1, child)
        .style(2, leaf)
        .measure(2, physical(mode, 20.0, 10.0));
    compute_layout(&tree, 0, frame.request(requests))
        .unwrap_or_else(|error| panic!(
            "ordinary Block percentage owner: {mode:?}, parent inline{parent_inline:?} block{parent_block:?}, percentage{percentage_child}: {error:?}"
        ))
}

fn assert_geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    mode: WritingMode,
    root_inline: f64,
    root_block: f64,
    child_block: f64,
    context: &str,
) {
    let flow = FlowAxes::new(mode, Direction::Ltr);
    let child_x = if mode == WritingMode::VerticalRl {
        root_block - child_block
    } else {
        0.0
    };
    let leaf_x = if mode == WritingMode::VerticalRl {
        child_block - 10.0
    } else {
        0.0
    };
    for (phase, entries) in [
        ("unrounded", batch.unrounded_entries()),
        ("final", batch.final_entries()),
    ] {
        for (node, size, location) in [
            (0, physical(mode, root_inline, root_block), Point::ZERO),
            (
                1,
                physical(mode, 20.0, child_block),
                Point::new(scalar(child_x), S::ZERO),
            ),
            (
                2,
                physical(mode, 20.0, 10.0),
                Point::new(scalar(leaf_x), S::ZERO),
            ),
        ] {
            let output = entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap_or_else(|| panic!("{context} {phase} node{node}: missing output"))
                .output();
            assert_eq!(
                output.size, size,
                "{context} {phase} node{node}: border size"
            );
            assert_eq!(
                output.location, location,
                "{context} {phase} node{node}: normal location"
            );
            assert_eq!(
                output.content_box_size(),
                size,
                "{context} {phase} node{node}: content box"
            );
            assert_eq!(
                (output.margin, output.border, output.padding),
                (Edges::ZERO, Edges::ZERO, Edges::ZERO),
                "{context} {phase} node{node}: zero edges"
            );
            assert_eq!(
                output.source_index,
                SourceIndex::ZERO,
                "{context} {phase} node{node}: original child rank"
            );
            let scroll = output
                .scroll_geometry
                .unwrap_or_else(|| panic!("{context} {phase} node{node}: missing scroll geometry"));
            assert_eq!(
                scroll.flow_axes(),
                flow,
                "{context} {phase} node{node}: scroll flow"
            );
            for (field, rect) in [
                ("border", scroll.border_box()),
                ("padding", scroll.padding_box()),
                ("content", scroll.content_box()),
                ("scrollport", scroll.scrollport()),
                ("overflow", scroll.scrollable_overflow()),
            ] {
                assert_eq!(
                    rect.origin(),
                    Point::ZERO,
                    "{context} {phase} node{node}: {field} origin"
                );
                assert_eq!(
                    rect.size(),
                    size,
                    "{context} {phase} node{node}: {field} size"
                );
            }
            let range = scroll.physical_range();
            assert_eq!(
                (
                    range.x().minimum(),
                    range.x().maximum(),
                    range.y().minimum(),
                    range.y().maximum()
                ),
                (S::ZERO, S::ZERO, S::ZERO, S::ZERO),
                "{context} {phase} node{node}: scroll range"
            );
        }
    }
}

fn horizontal_auto_inline_keeps_percentage_block_automatic<S: LayoutScalar>() {
    let mode = WritingMode::HorizontalTb;
    let natural = layout::<S>(mode, None, None, false);
    assert_geometry(
        &natural,
        mode,
        20.0,
        10.0,
        10.0,
        "auto parent and natural child",
    );
    let definite_inline = layout::<S>(mode, Some(200.0), None, true);
    assert_geometry(
        &definite_inline,
        mode,
        200.0,
        10.0,
        10.0,
        "definite inline does not make the auto block definite",
    );
    let definite_block = layout::<S>(mode, None, Some(100.0), true);
    assert_geometry(
        &definite_block,
        mode,
        20.0,
        100.0,
        50.0,
        "authored definite block resolves fifty percent",
    );

    // Sizing 3 section 5.2.1: an ordinary cyclic block percentage behaves as Auto
    // when the containing block's auto block size depends on that content.
    let automatic = layout::<S>(mode, None, None, true);
    assert_geometry(
        &automatic,
        mode,
        20.0,
        10.0,
        10.0,
        "auto-inline parent retains automatic child block size",
    );
}

fn decreasing_auto_block_keeps_percentage_block_automatic<S: LayoutScalar>() {
    let increasing = layout::<S>(WritingMode::VerticalLr, Some(200.0), None, true);
    assert_geometry(
        &increasing,
        WritingMode::VerticalLr,
        200.0,
        10.0,
        10.0,
        "increasing auto block stays indefinite for child percentages",
    );
    let mode = WritingMode::VerticalRl;
    let natural = layout::<S>(mode, Some(200.0), None, false);
    assert_geometry(
        &natural,
        mode,
        200.0,
        10.0,
        10.0,
        "decreasing natural block control",
    );
    let definite = layout::<S>(mode, Some(200.0), Some(100.0), true);
    assert_geometry(
        &definite,
        mode,
        200.0,
        100.0,
        50.0,
        "decreasing authored definite block resolves fifty percent",
    );
    let automatic = layout::<S>(mode, Some(200.0), None, true);
    assert_geometry(
        &automatic,
        mode,
        200.0,
        10.0,
        10.0,
        "decreasing auto block retains automatic child block size",
    );
}

#[test]
fn auto_inline_block_parent_does_not_resolve_cyclic_child_block_percentage_f32() {
    horizontal_auto_inline_keeps_percentage_block_automatic::<f32>();
}
#[test]
fn auto_inline_block_parent_does_not_resolve_cyclic_child_block_percentage_f64() {
    horizontal_auto_inline_keeps_percentage_block_automatic::<f64>();
}
#[test]
fn decreasing_auto_block_parent_does_not_resolve_cyclic_child_block_percentage_f32() {
    decreasing_auto_block_keeps_percentage_block_automatic::<f32>();
}
#[test]
fn decreasing_auto_block_parent_does_not_resolve_cyclic_child_block_percentage_f64() {
    decreasing_auto_block_keeps_percentage_block_automatic::<f64>();
}

fn assert_auto_height_retains_percentage_margins<S: LayoutScalar>() {
    use crate::geometry::{LogicalEdgesOf, LogicalSizeOf};

    for mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalLr,
        WritingMode::VerticalRl,
    ] {
        let flow = FlowAxes::new(mode, Direction::Ltr);
        for definite_inline in [false, true] {
            for width_on_margin_child in [false, true] {
                let fixed_inline = |fixed| {
                    if fixed {
                        PreferredSizeOf::px(scalar::<S>(50.0))
                    } else {
                        PreferredSizeOf::AUTO
                    }
                };
                let root = NodeInputOf {
                    display: Display::Block,
                    writing_mode: mode,
                    size: flow.physical_size(LogicalSizeOf::new(
                        fixed_inline(definite_inline),
                        PreferredSizeOf::AUTO,
                    )),
                    ..NodeInputOf::default()
                };
                let percentage = LengthAutoOf::percent(scalar(0.1));
                let margin_child = NodeInputOf {
                    display: Display::Block,
                    writing_mode: mode,
                    size: flow.physical_size(LogicalSizeOf::new(
                        fixed_inline(width_on_margin_child),
                        PreferredSizeOf::px(scalar(10.0)),
                    )),
                    margin: flow.physical_edges(LogicalEdgesOf::new(
                        LengthAutoOf::ZERO,
                        LengthAutoOf::ZERO,
                        percentage,
                        percentage,
                    )),
                    ..NodeInputOf::default()
                };
                let sibling = NodeInputOf {
                    display: Display::Block,
                    writing_mode: mode,
                    size: flow.physical_size(LogicalSizeOf::new(
                        fixed_inline(!width_on_margin_child),
                        PreferredSizeOf::px(scalar(10.0)),
                    )),
                    ..NodeInputOf::default()
                };
                let tree = PublicLayoutTreeOf::new()
                    .children(0, [1, 2])
                    .children(1, [])
                    .children(2, [])
                    .style(0, root)
                    .style(1, margin_child)
                    .style(2, sibling);
                let inline =
                    definite_inline.then(|| NonNegativeFiniteOf::new(scalar(50.0)).unwrap());
                let parent = flow.physical_size(LogicalSizeOf::new(inline, None));
                let axes = flow.physical_size(LogicalSizeOf::new(
                    AxisSizingRequestOf::new(
                        inline.map_or(AvailableSpaceOf::Indefinite, AvailableSpaceOf::Definite),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ));
                let request = crate::test_support::requests::NativeRootFrameOf::new(
                    flow,
                    parent,
                    OrthogonalFallbackOf::new(
                        parent,
                        NearestScrollportOf::Absent,
                        Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                    ),
                )
                .request(axes);
                let batch = compute_layout(&tree, 0, request).unwrap();
                for entries in [batch.unrounded_entries(), batch.final_entries()] {
                    let output = |node| {
                        entries
                            .iter()
                            .find(|entry| entry.node() == node)
                            .unwrap()
                            .output()
                    };
                    let context = (mode, definite_inline, width_on_margin_child);
                    assert_eq!(output(0).size, physical(mode, 50.0, 30.0), "{context:?}");
                    for (node, offset) in [(1, 5.0), (2, 20.0)] {
                        let position = match mode {
                            WritingMode::HorizontalTb => Point::new(S::ZERO, scalar(offset)),
                            WritingMode::VerticalLr => Point::new(scalar(offset), S::ZERO),
                            WritingMode::VerticalRl => Point::new(scalar(20.0 - offset), S::ZERO),
                            _ => unreachable!("fixture only uses horizontal and upright vertical"),
                        };
                        assert_eq!(output(node).size, physical(mode, 50.0, 10.0), "{context:?}");
                        assert_eq!(output(node).location, position, "{context:?}");
                    }
                    let margins = flow.logical_edges(output(1).margin);
                    assert_eq!(
                        (margins.block_start, margins.block_end),
                        (scalar(5.0), scalar(5.0)),
                        "{context:?}",
                    );
                }
            }
        }
    }
}

#[test]
fn final_auto_height_retains_resolved_percentage_margins_f32() {
    assert_auto_height_retains_percentage_margins::<f32>();
}

#[test]
fn final_auto_height_retains_resolved_percentage_margins_f64() {
    assert_auto_height_retains_percentage_margins::<f64>();
}
