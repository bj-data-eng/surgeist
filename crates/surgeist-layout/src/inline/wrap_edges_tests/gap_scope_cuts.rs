//! Native gap authority retains actual formatter and fitting-run source cuts.
use super::gap_attachments::{append_empty_box, replace_boundary};
use super::*;
use crate::{InlineFormatterEdge, InlineGapAnchor, InlineGapGroupInput};

fn float_edge_fixture<S: LayoutScalar>(leading: bool) -> Fixture<S> {
    let mut f = Fixture::<S>::new(Direction::Ltr, 100.0);
    if !leading {
        f.atomic(
            10,
            WrapEdges::AUTO,
            InlineBreakOpportunity::Prohibited,
            false,
        );
    }
    append_empty_box(&mut f, 20, 21);
    if leading {
        f.atomic(
            10,
            WrapEdges::AUTO,
            InlineBreakOpportunity::Prohibited,
            false,
        );
    }
    f.tree.insert_input(
        30,
        LayoutInputOf::box_input(NodeInputOf {
            display: crate::Display::Block,
            float: crate::Float::Left,
            size: Size::new(PreferredSizeOf::px(n(10.0)), PreferredSizeOf::px(n(10.0))),
            ..NodeInputOf::default()
        }),
    );
    let float = GeneratedChildBoxInput {
        box_: GeneratedBoxRef::Node(30),
        participation: GeneratedBoxParticipation::OutOfFlow,
    };
    let children = if leading {
        f.nodes.insert(0, 30);
        vec![float, child(pair(20, 21)), child(GeneratedBoxRef::Node(10))]
    } else {
        f.nodes.push(30);
        vec![child(GeneratedBoxRef::Node(10)), child(pair(20, 21)), float]
    };
    f.wrapping
        .child_order
        .insert(0, order(GeneratedBoxRef::Node(0), children));
    f.finish();
    f
}

fn formatter_edge_outside_actual_run<S: LayoutScalar>(leading: bool) {
    let mut f = float_edge_fixture::<S>(leading);
    // Qualify the actual Native/float graph and derived neighboring witness
    // independently before requesting the stricter explicit source-edge fact.
    let baseline = f.compute();
    assert!(baseline.placed_text().is_empty());
    assert_eq!(output(&baseline, 10).size, Size::splat(n(10.0)));
    assert_eq!(output(&baseline, 30).size, Size::splat(n(10.0)));
    assert_eq!(output(&baseline, 20).size.width, n(3.0));
    assert_eq!(output(&baseline, 21).size.width, n(5.0));
    f.wrapping.gaps.push(InlineGapGroupInput {
        anchor: InlineGapAnchor::Formatter {
            node: 0,
            edge: if leading {
                InlineFormatterEdge::Start
            } else {
                InlineFormatterEdge::End
            },
        },
        boxes: vec![InlineBoxRef::NativePair { start: 20, end: 21 }],
    });
    let flow = f.flow();
    let error = compute_layout_with_flow(&flow, 0, request(f.width)).expect_err(
        "a real float outside the fitting run separates its cut from the formatter source edge",
    );
    assert!(
        matches!(
            error.kind(),
            crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
                WrappingScopeError::GapOutsideSelection
            ))
        ),
        "{error:?}"
    );
}

#[test]
fn numeric_gap_formatter_start_beyond_leading_float_refuses_f32() {
    formatter_edge_outside_actual_run::<f32>(true);
}
#[test]
fn numeric_gap_formatter_start_beyond_leading_float_refuses_f64() {
    formatter_edge_outside_actual_run::<f64>(true);
}
#[test]
fn numeric_gap_formatter_end_beyond_trailing_float_refuses_f32() {
    formatter_edge_outside_actual_run::<f32>(false);
}
#[test]
fn numeric_gap_formatter_end_beyond_trailing_float_refuses_f64() {
    formatter_edge_outside_actual_run::<f64>(false);
}

fn mixed_formatter_group<S: LayoutScalar>() {
    let mut f = Fixture::<S>::new(Direction::Ltr, 100.0);
    append_empty_box(&mut f, 20, 21);
    f.atomic(
        50,
        WrapEdges::AUTO,
        InlineBreakOpportunity::Prohibited,
        false,
    );
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for (node, kind, width) in [
        (30, InlineBoundaryKind::Start, 3.0),
        (31, InlineBoundaryKind::End, 5.0),
    ] {
        f.boundary(node, kind, width);
        replace_boundary(
            &mut f,
            node,
            InlineBoundaryInputOf::box_edge(
                kind,
                metrics(),
                InlineBoundaryEdgeOf::try_new(S::ZERO, n(width), S::ZERO).unwrap(),
                axes,
            ),
        );
    }
    f.nodes.truncate(3);
    f.participants.truncate(3);
    f.tree.insert_children(50, [30, 31]);
    f.wrapping.scopes.push(WrappingScopeInput::NativePair {
        start: 30,
        end: 31,
        wrap_inside: WrapInside::Auto,
        edges: WrapEdges::AUTO,
        metric_contribution: None,
    });
    f.wrapping.child_order.insert(
        0,
        order(
            GeneratedBoxRef::Node(0),
            vec![child(pair(20, 21)), child(GeneratedBoxRef::Node(50))],
        ),
    );
    let mut inner = order(GeneratedBoxRef::Node(50), vec![child(pair(30, 31))]);
    inner.formatting_context = 50;
    let mut inner_box = order(pair(30, 31), vec![]);
    inner_box.formatting_context = 50;
    f.wrapping.child_order.extend([inner, inner_box]);
    f.finish();
    // Both genuine boxes are individually admitted in their real formatters.
    drop(f.flow());
    f.wrapping.gaps.push(InlineGapGroupInput {
        anchor: InlineGapAnchor::Formatter {
            node: 0,
            edge: InlineFormatterEdge::Start,
        },
        boxes: vec![
            InlineBoxRef::NativePair { start: 20, end: 21 },
            InlineBoxRef::NativePair { start: 30, end: 31 },
        ],
    });
    let error = LayoutFlowInput::try_new(&f.tree, 0, vec![], f.wrapping.clone(), Vec::new())
        .err()
        .expect("different known formatter owners do not share a physical gap group");
    assert!(
        matches!(
            error.kind(),
            crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
                WrappingScopeError::InvalidGapAnchor
            ))
        ),
        "{error:?}"
    );
}

#[test]
fn numeric_gap_known_cross_formatter_group_refuses_as_invalid_anchor_f32() {
    mixed_formatter_group::<f32>();
}
#[test]
fn numeric_gap_known_cross_formatter_group_refuses_as_invalid_anchor_f64() {
    mixed_formatter_group::<f64>();
}
