#![forbid(unsafe_code)]
//! Public composition with actual completed Layout geometry. Paint categories,
//! CSS positioning and appearance are explicit native caller facts, not a Style
//! adapter or deductions from Layout coordinates or output storage order.

use std::{convert::Infallible, ptr};
use surgeist_layout as layout;
use surgeist_paint::{
    BoxFacts, BoxId, BoxPaint, BoxPaintKind, CanvasPaintKind, ChildOrder, ContextEffect,
    FloatParticipation, FragmentId, FragmentOwner, FragmentPaintKind, PaintFragment, PaintLimits,
    PaintOptions, PaintOrder, PaintOrderEvent, PaintPrimitive, PaintRequest, Participation,
    Position, PrimitiveKind, ProfileMode, ZIndex, plan_paint_order,
};

struct Tree {
    inputs: Vec<layout::LayoutInput>,
    children: Vec<Vec<usize>>,
}

impl layout::Traverse for Tree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;
    fn children(&self, node: usize) -> Self::Children<'_> {
        self.children[node].iter().copied()
    }
    fn child_count(&self, node: usize) -> usize {
        self.children[node].len()
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children[node][index]
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

fn px(value: f32) -> layout::PreferredSize {
    layout::PreferredSize::value(layout::LengthPercentageOf::px(value).unwrap())
}

fn root_request() -> layout::LayoutRootRequest {
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

fn options() -> PaintOptions {
    PaintOptions::new(
        PaintLimits::unlimited(),
        ProfileMode::Summary { timing: false },
    )
}

fn slot(identities: &[usize], node: usize) -> usize {
    identities.iter().position(|&value| value == node).unwrap()
}

#[derive(Debug, Eq, PartialEq)]
enum Event {
    Begin(usize),
    Draw(usize, PrimitiveKind),
    End(usize),
}

fn color(node: usize) -> Event {
    Event::Draw(
        node,
        PrimitiveKind::Fragment(FragmentPaintKind::BackgroundColor),
    )
}

fn outline(node: usize) -> Event {
    Event::Draw(node, PrimitiveKind::Fragment(FragmentPaintKind::Outline))
}

fn observed(plan: &PaintOrder<'_>, identities: &[usize]) -> Vec<Event> {
    plan.events()
        .map(|event| match event {
            PaintOrderEvent::BeginContext(owner) => {
                assert!(ptr::eq(
                    owner.facts(),
                    &plan.request().boxes()[owner.id().get()]
                ));
                assert!(ptr::eq(
                    owner.paint(),
                    &plan.request().box_paint()[owner.id().get()]
                ));
                Event::Begin(identities[owner.id().get()])
            }
            PaintOrderEvent::Draw(primitive) => {
                assert!(ptr::eq(
                    primitive.record(),
                    &plan.request().primitives()[primitive.id().get()]
                ));
                if let Some(fragment) = primitive.fragment() {
                    assert!(ptr::eq(
                        fragment.record(),
                        &plan.request().fragments()[fragment.id().get()]
                    ));
                    assert_eq!(fragment.owner().id(), primitive.owner().id());
                }
                Event::Draw(
                    identities[primitive.owner().id().get()],
                    primitive.record().kind(),
                )
            }
            PaintOrderEvent::EndContext(owner) => Event::End(identities[owner.id().get()]),
        })
        .collect()
}

#[test]
fn real_boxes_keep_geometry_and_appearance_across_source_flex_grid_paint_permutations() {
    for mode in [ChildOrder::Source, ChildOrder::Flex, ChildOrder::Grid] {
        let tree = Tree {
            inputs: (0..9)
                .map(|node| {
                    layout::LayoutInput::box_input(layout::NodeInput {
                        display: if node == 0 {
                            match mode {
                                ChildOrder::Source => layout::Display::Block,
                                ChildOrder::Flex => layout::Display::Flex,
                                ChildOrder::Grid => layout::Display::Grid,
                            }
                        } else {
                            layout::Display::Block
                        },
                        size: layout::Size::new(
                            px(if node == 0 { 100.0 } else { 10.0 + node as f32 }),
                            px(if node == 0 { 100.0 } else { 10.0 + node as f32 }),
                        ),
                        item_order: layout::ItemOrder::new(match node {
                            1 => 2,
                            2 => -1,
                            3 => 1,
                            _ => 0,
                        }),
                        position: if node == 6 {
                            layout::Position::Absolute
                        } else {
                            layout::Position::Relative
                        },
                        ..layout::NodeInput::DEFAULT
                    })
                })
                .collect(),
            children: vec![
                vec![1, 2, 3, 4, 5, 6],
                vec![7],
                vec![8],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
        };
        let batch = layout::compute_layout(&tree, 0, root_request()).unwrap();
        for identities in [[0, 1, 2, 3, 4, 5, 6, 7, 8], [8, 4, 2, 0, 6, 1, 7, 5, 3]] {
            let box_id = |node| BoxId::new(slot(&identities, node));
            // Occurrences and appearance records use independent permutations.
            let fragment_nodes = [5, 7, 0, 3, 8, 1, 6, 4, 2];
            let fragment_id = |node| FragmentId::new(slot(&fragment_nodes, node));
            let geometry =
                fragment_nodes.map(|node| batch.node_layout(node).unwrap().single_box().unwrap());
            let facts: Vec<_> = identities
                .iter()
                .map(|&node| {
                    let output = batch
                        .node_layout(node)
                        .unwrap()
                        .single_box()
                        .unwrap()
                        .final_output();
                    BoxFacts::new(
                        match node {
                            0 => None,
                            7 => Some(box_id(1)),
                            8 => Some(box_id(2)),
                            _ => Some(box_id(0)),
                        },
                        output.source_index,
                        if node == 0 { mode } else { ChildOrder::Source },
                        if matches!(node, 1..=5) {
                            let order = tree.inputs[node].as_box().unwrap().item_order;
                            match mode {
                                ChildOrder::Source => Participation::Ordinary,
                                ChildOrder::Flex => Participation::FlexItem(order),
                                ChildOrder::Grid => Participation::GridItem(order),
                            }
                        } else {
                            Participation::Ordinary
                        },
                        match node {
                            4 | 5 => Position::Relative,
                            6 => Position::Absolute,
                            _ => Position::Static,
                        },
                        match node {
                            3 | 6 => ZIndex::Integer(0),
                            4 => ZIndex::Integer(i64::MIN),
                            5 => ZIndex::Integer(i64::MAX),
                            _ => ZIndex::Auto,
                        },
                        ContextEffect::None,
                    )
                })
                .collect();
            let paints = vec![
                BoxPaint::new(BoxPaintKind::Block, FloatParticipation::None);
                identities.len()
            ];
            let fragments: Vec<_> = fragment_nodes
                .iter()
                .map(|&node| {
                    PaintFragment::box_fragment(
                        box_id(node),
                        match node {
                            0 => None,
                            7 => Some(fragment_id(1)),
                            8 => Some(fragment_id(2)),
                            _ => Some(fragment_id(0)),
                        },
                        None,
                        0,
                    )
                })
                .collect();
            let appearance = [6, 1, 5, 0, 8, 4, 7, 3, 2, 1, 2];
            let primitives: Vec<_> = appearance
                .iter()
                .enumerate()
                .map(|(index, &node)| {
                    if node == 0 {
                        PaintPrimitive::canvas(CanvasPaintKind::BackgroundColor, 0)
                    } else {
                        PaintPrimitive::fragment(
                            fragment_id(node),
                            if index >= 9 {
                                FragmentPaintKind::Outline
                            } else {
                                FragmentPaintKind::BackgroundColor
                            },
                            0,
                        )
                    }
                })
                .collect();
            let request = PaintRequest::new(
                &facts,
                &paints,
                &[],
                &[],
                &fragments,
                &primitives,
                box_id(0),
            );
            let plan = plan_paint_order(request, options()).result.unwrap();
            assert!(ptr::eq(plan.request().boxes().as_ptr(), facts.as_ptr()));
            assert!(ptr::eq(
                plan.request().primitives().as_ptr(),
                primitives.as_ptr()
            ));
            let expected = if mode == ChildOrder::Source {
                vec![
                    Event::Begin(0),
                    Event::Draw(0, PrimitiveKind::Canvas(CanvasPaintKind::BackgroundColor)),
                    Event::Begin(4),
                    color(4),
                    Event::End(4),
                    color(1),
                    color(7),
                    color(2),
                    color(8),
                    color(3),
                    Event::Begin(6),
                    color(6),
                    Event::End(6),
                    Event::Begin(5),
                    color(5),
                    Event::End(5),
                    outline(1),
                    outline(2),
                    Event::End(0),
                ]
            } else {
                // Auto items are atomic inline scopes in order-modified order;
                // integer-zero items are actual contexts. Each auto item's late
                // outline stays inside that item's recursively painted scope.
                // The absolute ordinary child's effective order is zero, so its
                // zero-level context precedes the order-one item's context.
                vec![
                    Event::Begin(0),
                    Event::Draw(0, PrimitiveKind::Canvas(CanvasPaintKind::BackgroundColor)),
                    Event::Begin(4),
                    color(4),
                    Event::End(4),
                    color(2),
                    color(8),
                    outline(2),
                    color(1),
                    color(7),
                    outline(1),
                    Event::Begin(6),
                    color(6),
                    Event::End(6),
                    Event::Begin(3),
                    color(3),
                    Event::End(3),
                    Event::Begin(5),
                    color(5),
                    Event::End(5),
                    Event::End(0),
                ]
            };
            assert_eq!(observed(&plan, &identities), expected, "{mode:?}");
            for event in plan.events() {
                if let PaintOrderEvent::Draw(primitive) = event {
                    let node = appearance[primitive.id().get()];
                    assert_eq!(identities[primitive.owner().id().get()], node);
                    if let Some(fragment) = primitive.fragment() {
                        let retained = geometry[fragment.id().get()];
                        let original = batch.node_layout(node).unwrap().single_box().unwrap();
                        assert!(ptr::eq(retained.id(), original.id()));
                        assert_eq!(retained.final_output(), original.final_output());
                        assert_eq!(retained.final_output().size.width, 10.0 + node as f32);
                        assert_eq!(fragment.record().owner(), FragmentOwner::Box(box_id(node)));
                    }
                }
            }
        }
    }
}

#[test]
fn actual_fragmented_root_continuations_retain_exact_occurrence_geometry_and_parentage() {
    let tree = Tree {
        inputs: [None, Some(30.0), Some(40.0)]
            .map(|height| {
                layout::LayoutInput::box_input(layout::NodeInput {
                    display: layout::Display::Block,
                    size: layout::Size::new(
                        layout::PreferredSize::AUTO,
                        height.map_or(layout::PreferredSize::AUTO, px),
                    ),
                    ..layout::NodeInput::DEFAULT
                })
            })
            .into(),
        children: vec![vec![1, 2], vec![], vec![]],
    };
    let axes = layout::FlowAxes::new(layout::WritingMode::HorizontalTb, layout::Direction::Ltr);
    let bound = layout::LayoutBoundRequestOf::block_fragmentation(
        layout::LayoutSourceBasis::fresh(),
        root_request(),
        layout::FragmentationKind::Page {
            progression: layout::PageProgression::LeftToRight,
        },
        axes,
        vec![
            layout::BlockFragmentainerOf {
                origin: layout::Point::ZERO,
                size: layout::Size::new(100.0, 50.0),
                breaks: layout::FragmentationBreakKinds::NONE,
                page_side: None,
            },
            // Physical x is deliberately earlier despite later logical continuation.
            layout::BlockFragmentainerOf {
                origin: layout::Point::new(-200.0, 0.0),
                size: layout::Size::new(60.0, 80.0),
                breaks: layout::FragmentationBreakKinds::PAGE,
                page_side: None,
            },
        ],
    )
    .unwrap();
    let flow = layout::LayoutFlowInput::try_new_bound(
        &tree,
        0,
        bound.clone(),
        Vec::new(),
        layout::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(&flow, 0, bound.root_request()).unwrap();
    let root = batch.node_layout(0).unwrap();
    assert!(root.single_box().is_none());
    let continuations = root.fragments().unwrap();
    assert_eq!(continuations.len(), 2);
    assert_eq!(continuations[0].previous(), None);
    assert_eq!(continuations[0].next(), Some(continuations[1].id()));
    assert_eq!(continuations[1].previous(), Some(continuations[0].id()));
    assert_eq!(continuations[1].next(), None);
    assert_eq!(
        continuations[0]
            .principal_box()
            .unwrap()
            .final_geometry()
            .size,
        layout::Size::new(100.0, 50.0)
    );
    assert_eq!(
        continuations[1]
            .principal_box()
            .unwrap()
            .final_geometry()
            .size,
        layout::Size::new(60.0, 40.0)
    );
    assert_eq!(
        batch
            .fragmentainer(continuations[1].fragmentainer().expect("external frame"))
            .unwrap()
            .origin(),
        layout::Point::new(-200.0, 0.0)
    );
    assert_eq!(
        continuations[0].source_children(),
        [(layout::SourceIndex::new(0), 1)]
    );
    assert_eq!(
        continuations[1].source_children(),
        [(layout::SourceIndex::new(1), 2)]
    );
    let identities = [2, 0, 1];
    let box_id = |node| BoxId::new(slot(&identities, node));
    let facts: Vec<_> = identities
        .iter()
        .map(|&node| {
            BoxFacts::new(
                if node == 0 { None } else { Some(box_id(0)) },
                if node == 0 {
                    layout::SourceIndex::ZERO
                } else {
                    batch
                        .node_layout(node)
                        .unwrap()
                        .single_box()
                        .unwrap()
                        .final_output()
                        .source_index
                },
                ChildOrder::Source,
                Participation::Ordinary,
                Position::Static,
                ZIndex::Auto,
                ContextEffect::None,
            )
        })
        .collect();
    let paints = [BoxPaint::new(BoxPaintKind::Block, FloatParticipation::None); 3];
    // Map each Paint slot to its actual fragment; no first/union root rectangle.
    let fragments = [
        PaintFragment::box_fragment(box_id(0), None, None, continuations[1].continuation_order()),
        PaintFragment::box_fragment(box_id(1), Some(FragmentId::new(3)), None, 0),
        PaintFragment::box_fragment(box_id(2), Some(FragmentId::new(0)), None, 0),
        PaintFragment::box_fragment(box_id(0), None, None, continuations[0].continuation_order()),
    ];
    let first_child = batch.node_layout(1).unwrap().single_box().unwrap();
    let second_child = batch.node_layout(2).unwrap().single_box().unwrap();
    let occurrence_ids = [
        continuations[1].id(),
        first_child.id(),
        second_child.id(),
        continuations[0].id(),
    ];
    let primitives = [
        PaintPrimitive::fragment(FragmentId::new(2), FragmentPaintKind::BackgroundColor, 0),
        PaintPrimitive::fragment(FragmentId::new(0), FragmentPaintKind::Border, 0),
        PaintPrimitive::fragment(FragmentId::new(1), FragmentPaintKind::BackgroundColor, 0),
        PaintPrimitive::canvas(CanvasPaintKind::BackgroundColor, 0),
        PaintPrimitive::fragment(FragmentId::new(3), FragmentPaintKind::Border, 0),
    ];
    let plan = plan_paint_order(
        PaintRequest::new(
            &facts,
            &paints,
            &[],
            &[],
            &fragments,
            &primitives,
            box_id(0),
        ),
        options(),
    )
    .result
    .unwrap();
    let emitted: Vec<_> = plan
        .events()
        .map(|event| match event {
            PaintOrderEvent::BeginContext(owner) => ('[', identities[owner.id().get()]),
            PaintOrderEvent::Draw(primitive) => ('d', primitive.id().get()),
            PaintOrderEvent::EndContext(owner) => (']', identities[owner.id().get()]),
        })
        .collect();
    // Root border occurrences follow logical continuation order, then ordinary
    // child backgrounds follow semantic source order rather than physical x.
    assert_eq!(
        emitted,
        [
            ('[', 0),
            ('d', 3),
            ('d', 4),
            ('d', 1),
            ('d', 2),
            ('d', 0),
            (']', 0)
        ]
    );
    for event in plan.events() {
        if let PaintOrderEvent::Draw(primitive) = event {
            assert!(ptr::eq(
                primitive.record(),
                &primitives[primitive.id().get()]
            ));
            if let Some(fragment) = primitive.fragment() {
                let id = occurrence_ids[fragment.id().get()];
                let actual = batch.fragment(id).unwrap();
                match batch.node_layout(actual.node()).unwrap() {
                    layout::NodeLayoutRefOf::Fragments(_) => {
                        let expected = if fragment.id().get() == 0 {
                            &continuations[1]
                        } else {
                            &continuations[0]
                        };
                        assert!(ptr::eq(actual, expected));
                        assert_eq!(
                            actual.continuation_order(),
                            fragment.record().logical_order()
                        );
                        assert!(ptr::eq(
                            actual.principal_box().unwrap(),
                            expected.principal_box().unwrap()
                        ));
                        assert_eq!(batch.fragment(actual.id()).unwrap().id(), id);
                    }
                    layout::NodeLayoutRefOf::SingleBox(actual) => {
                        let node = identities[fragment.owner().id().get()];
                        assert_eq!(actual.node(), node);
                        let parent = fragment.parent().unwrap();
                        assert_eq!(
                            actual.coordinate_parent(),
                            Some(&layout::FragmentCoordinateSpace::Fragment(
                                occurrence_ids[parent.id().get()].clone()
                            ))
                        );
                        assert_eq!(
                            actual.final_output().size,
                            layout::Size::new(
                                if node == 1 { 100.0 } else { 60.0 },
                                if node == 1 { 30.0 } else { 40.0 }
                            )
                        );
                    }
                    layout::NodeLayoutRefOf::NonBox(_) => {
                        panic!("paint occurrence must resolve to an actual box")
                    }
                }
            }
        }
    }
}

#[test]
fn sized_border_image_uses_completed_border_geometry_without_changing_layout() {
    use surgeist_paint::{
        BorderImageOutset, BorderImageRequest, BorderImageSlice, BorderImageSource,
        BorderImageWidth, resolve_border_image,
    };

    let border = |value| layout::LengthOf::value(layout::LengthPercentageOf::px(value).unwrap());
    let tree = Tree {
        inputs: vec![layout::LayoutInput::box_input(layout::NodeInput {
            display: layout::Display::Block,
            box_sizing: layout::BoxSizing::BorderBox,
            size: layout::Size::new(px(40.0), px(20.0)),
            border: layout::Edges::new(border(2.0), border(3.0), border(4.0), border(5.0)),
            ..layout::NodeInput::DEFAULT
        })],
        children: vec![vec![]],
    };
    let batch = layout::compute_layout(&tree, 0, root_request()).unwrap();
    let node = batch.node_layout(0).unwrap().single_box().unwrap();
    let output = node.final_output();
    assert_eq!(output.size, layout::Size::new(40.0, 20.0));
    assert_eq!(output.border, layout::Edges::new(2.0, 3.0, 4.0, 5.0));

    let geometry = resolve_border_image(
        BorderImageRequest {
            border_origin: output.location.map(f64::from),
            border_size: output.size.map(f64::from),
            border_widths: output.border.map(f64::from),
            source: BorderImageSource {
                coordinate_size: layout::Size::new(20.0, 10.0),
                natural_size: layout::Size::new(Some(10.0), Some(5.0)),
            },
            slices: layout::Edges::all(BorderImageSlice::SourceUnits(2.0)),
            widths: layout::Edges::all(BorderImageWidth::BorderWidthMultiple(2.0)),
            outsets: layout::Edges::all(BorderImageOutset::BorderWidthMultiple(1.0)),
            fill_center: true,
        },
        options(),
    )
    .result
    .unwrap();
    assert_eq!(geometry.area().min(), layout::Point::new(-5.0, -2.0));
    assert_eq!(geometry.area().max(), layout::Point::new(43.0, 24.0));
    assert_eq!(
        geometry.used_widths(),
        layout::Edges::new(4.0, 6.0, 8.0, 10.0)
    );
    assert_eq!(node.final_output(), output);
}
