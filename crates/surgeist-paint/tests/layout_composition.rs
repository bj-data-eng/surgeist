#![forbid(unsafe_code)]
//! Real Layout computation and caller-owned resolved paint facts. This is not a
//! computed Style adapter. Batch slice order supplies neither topology nor paint order.
use std::convert::Infallible;
use surgeist_layout as layout;
use surgeist_paint::{
    BoxFacts, BoxId, ChildOrder, ContextEffect, PaintLimits, PaintOptions, Participation, Position,
    ProfileMode, ZIndex, plan_stacking,
};

struct Tree {
    inputs: Vec<layout::LayoutInput>,
}
impl layout::Traverse for Tree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;
    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = match node {
            0 => &[1, 2, 3],
            1 => &[4],
            _ => &[],
        };
        children.iter().copied()
    }
    fn child_count(&self, node: usize) -> usize {
        self.children(node).count()
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}
impl layout::LayoutTree for Tree {
    type MeasureError = Infallible;
    fn node_input(&self, node: usize) -> &layout::NodeInput {
        self.inputs[node].as_box().unwrap()
    }
    fn layout_input(&self, node: usize) -> layout::LayoutInputView<'_> {
        self.inputs[node].as_view()
    }
}
fn px(n: f32) -> layout::PreferredSize {
    layout::PreferredSize::value(layout::LengthPercentageOf::px(n).unwrap())
}
fn request() -> layout::LayoutRootRequest {
    let extent = layout::Size::splat(Some(layout::NonNegativeFiniteOf::new(100.0).unwrap()));
    let axis = layout::AxisSizingRequest::new(
        layout::AvailableSpace::Definite(layout::NonNegativeFiniteOf::new(100.0).unwrap()),
        layout::SizingOperation::Normal,
    );
    layout::LayoutRootRequest::standalone(
        layout::SizingRequest::new(
            layout::Size::splat(axis),
            layout::OrthogonalFallback::new(
                extent,
                layout::NearestScrollport::Absent,
                layout::Size::splat(layout::OrthogonalFallbackLimit::Unbounded),
            ),
        ),
        layout::FlowAxes::new(layout::WritingMode::HorizontalTb, layout::Direction::Ltr),
        extent,
    )
}
#[test]
fn completed_geometry_stays_joined_to_identity_when_paint_slice_is_permuted() {
    for mode in [ChildOrder::Source, ChildOrder::Flex, ChildOrder::Grid] {
        let display = match mode {
            ChildOrder::Source => layout::Display::Block,
            ChildOrder::Flex => layout::Display::Flex,
            ChildOrder::Grid => layout::Display::Grid,
        };
        let tree = Tree {
            inputs: (0..5)
                .map(|node| {
                    layout::LayoutInput::box_input(layout::NodeInput {
                        display: if node == 0 {
                            display
                        } else {
                            layout::Display::Block
                        },
                        size: layout::Size::new(
                            px(if node == 0 { 100.0 } else { 20.0 }),
                            px(if node == 0 { 100.0 } else { 20.0 }),
                        ),
                        item_order: layout::ItemOrder::new(match node {
                            1 => 1,
                            2 => -1,
                            3 => -99,
                            _ => 0,
                        }),
                        position: if node == 3 {
                            layout::Position::Absolute
                        } else {
                            layout::Position::Relative
                        },
                        ..layout::NodeInput::DEFAULT
                    })
                })
                .collect(),
        };
        let batch = layout::compute_layout(&tree, 0, request()).unwrap();
        // Stable producer identities, deliberately unrelated to source/paint order.
        let identities = [4usize, 2, 0, 3, 1];
        let slot = |node| BoxId::new(identities.iter().position(|&id| id == node).unwrap());
        let geometry = identities.map(|node| {
            batch
                .final_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
        });
        for (node, output) in identities.into_iter().zip(geometry) {
            assert_eq!(output.size.width, if node == 0 { 100.0 } else { 20.0 });
            assert_eq!(output.size.height, if node == 0 { 100.0 } else { 20.0 });
        }
        let facts: Vec<_> = identities
            .iter()
            .zip(&geometry)
            .map(|(&node, output)| {
                let parent = match node {
                    0 => None,
                    4 => Some(slot(1)),
                    _ => Some(slot(0)),
                };
                let participation = if matches!(node, 1 | 2) {
                    match mode {
                        ChildOrder::Flex => {
                            Participation::FlexItem(tree.inputs[node].as_box().unwrap().item_order)
                        }
                        ChildOrder::Grid => {
                            Participation::GridItem(tree.inputs[node].as_box().unwrap().item_order)
                        }
                        ChildOrder::Source => Participation::Ordinary,
                    }
                } else {
                    Participation::Ordinary
                };
                // Explicit native resolved facts; Layout's Relative does not assert
                // CSS-relative positioning. CSS-static item semantics are supplied here.
                BoxFacts::new(
                    parent,
                    output.source_index,
                    if node == 0 { mode } else { ChildOrder::Source },
                    participation,
                    if node == 3 {
                        Position::Absolute
                    } else if node == 4 {
                        Position::Relative
                    } else {
                        Position::Static
                    },
                    if matches!(node, 2..=4) {
                        ZIndex::Integer(0)
                    } else {
                        ZIndex::Auto
                    },
                    ContextEffect::None,
                )
            })
            .collect();
        let outcome = plan_stacking(
            &facts,
            slot(0),
            PaintOptions::new(
                PaintLimits::unlimited(),
                ProfileMode::Summary { timing: false },
            ),
        );
        let plan = outcome.result.unwrap();
        let children: Vec<_> = plan
            .root_context()
            .children()
            .map(|view| identities[view.box_id().get()])
            .collect();
        // Static ordinary z-index on node 2 is inapplicable in block mode.
        assert_eq!(
            children,
            if mode == ChildOrder::Source {
                vec![4, 3]
            } else {
                vec![2, 3, 4]
            }
        );
        for context in plan.root_context().children() {
            let index = context.box_id().get();
            let node = identities[index];
            let retained_entry = batch
                .final_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap();
            assert_eq!(geometry[index], retained_entry.output());
            assert_eq!(geometry[index].size.width, 20.0);
        }
        assert_eq!(plan.context_for_box(slot(1)).unwrap().box_id(), slot(0));
        assert_eq!(
            geometry[slot(4).get()].source_index,
            layout::SourceIndex::ZERO
        );
    }
}
