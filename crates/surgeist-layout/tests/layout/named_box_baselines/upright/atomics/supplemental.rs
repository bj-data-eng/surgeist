//! Independently derived named-source, allocation and replay acceptance.
use super::*;
mod late;

struct Scene<S: l::LayoutScalar> {
    host: Host<S>,
    bindings: Vec<l::TextParagraphBinding<u32>>,
}

fn attach<S: l::LayoutScalar>(
    host: &mut Host<S>,
    container: u32,
    parent: &Parent,
    input: l::NodeInputOf<S>,
    child: Option<&Fixture>,
) -> Vec<l::TextParagraphBinding<u32>> {
    let origins = parent
        .prepared
        .origin_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    let mut nodes = Vec::new();
    let mut bound = Vec::new();
    if let Some(origin) = origins.first() {
        let node = host.push(
            Some(container),
            l::LayoutInputOf::inline_text(l::TextParticipation::new(origin.clone())),
        );
        nodes.push(node);
        bound.push(l::TextOriginBinding {
            origin: origin.clone(),
            node,
        });
    }
    let atomic_node = host.push(Some(container), l::LayoutInputOf::box_input(input));
    nodes.push(atomic_node);
    if let Some(origin) = origins.get(1) {
        let node = host.push(
            Some(container),
            l::LayoutInputOf::inline_text(l::TextParticipation::new(origin.clone())),
        );
        nodes.push(node);
        bound.push(l::TextOriginBinding {
            origin: origin.clone(),
            node,
        });
    }
    let occurrence = parent.prepared.atomic_views().next().unwrap().handle();
    assert_eq!(
        occurrence.view().original_box().id(),
        t::Id::from_u64(u64::from(atomic_node))
    );
    let mut bindings = vec![l::TextParagraphBinding {
        prepared: parent.prepared.clone(),
        container,
        nodes,
        origins: bound,
        atomics: vec![l::TextAtomicBinding {
            occurrence,
            node: atomic_node,
        }],
        controls: vec![],
        line_strut: Some(STRUT),
    }];
    if let Some(child) = child {
        let origin = child.prepared.origin_views().next().unwrap().handle();
        let text = host.push(
            Some(atomic_node),
            l::LayoutInputOf::inline_text(l::TextParticipation::new(origin)),
        );
        bindings.push(child_binding(child, atomic_node, text));
    }
    bindings
}

fn make_scene<S: l::LayoutScalar>(
    parent: &Parent,
    mode: l::WritingMode,
    size: (f64, f64),
    input: l::NodeInputOf<S>,
    child: Option<&Fixture>,
) -> Scene<S> {
    let mut outer = root(mode, l::Direction::Ltr, central());
    outer.size = l::Size::new(px(size.0), px(size.1));
    let mut host = Host::new(outer);
    let bindings = attach(&mut host, 0, parent, input, child);
    Scene { host, bindings }
}

fn wrapped<S: l::LayoutScalar>(
    parent: &Parent,
    mode: l::WritingMode,
    child: &Fixture,
    fractional: bool,
) -> Scene<S> {
    let mut input = root(l::WritingMode::VerticalLr, l::Direction::Ltr, central());
    input.size = l::Size::new(px(100.0), px(if fractional { 100.0 } else { 22.0 }));
    if fractional {
        input.padding.left = length(7.25);
        input.padding.top = length(9.5);
    }
    let mut host = Host::new(input);
    let mut paragraph = root(mode, l::Direction::Ltr, central());
    paragraph.size = l::Size::new(px(40.0), px(22.0));
    let container = host.push(Some(0), l::LayoutInputOf::box_input(paragraph));
    let bindings = attach(&mut host, container, parent, child_input(), Some(child));
    Scene { host, bindings }
}

fn run<S: l::LayoutScalar>(
    scene: &Scene<S>,
    space: l::Size<l::AvailableOf<S>>,
    changed: &[u32],
) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S>> {
    let flow = l::LayoutFlowInput::try_new(
        &scene.host,
        0,
        scene.bindings.clone(),
        l::WrappingInput::default(),
    )
    .expect("real tree membership and all issued bindings must validate before behavior");
    l::compute_layout_with_flow_invalidated(
        &flow,
        0,
        l::LayoutRootRequestOf::viewport(space).unwrap(),
        changed,
    )
}

fn length<S: l::LayoutScalar>(value: f64) -> l::LengthOf<S> {
    l::LengthOf::value(l::LengthPercentageOf::px(n(value)).unwrap())
}
fn margin<S: l::LayoutScalar>(value: f64) -> l::LengthAutoOf<S> {
    l::LengthAutoOf::value(l::LengthPercentageOf::px(n(value)).unwrap())
}

fn receipts<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    binding: &l::TextParagraphBinding<u32>,
) -> std::collections::BTreeMap<usize, t::FinalizedLine> {
    let mut lines = std::collections::BTreeMap::new();
    for row in batch.unrounded_inline_fragments() {
        if matches!(row.owner(), l::InlineFragmentOwner::Node(node) if binding.origins.iter().any(|origin| origin.node == *node))
        {
            let fragment = row.fragment();
            let line = fragment.finalized_fragment().finalized_line();
            assert_eq!(line.prepared_text(), binding.prepared);
            lines
                .entry(fragment.line_index())
                .or_insert_with(|| line.clone());
        }
    }
    lines
}

fn atomic_rect<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    binding: &l::TextParagraphBinding<u32>,
    expected: t::Rect,
) {
    let text = placed(batch, binding.container);
    let boxes = text.inline_boxes();
    assert_eq!(boxes.len(), 1);
    assert_eq!(
        boxes[0].id(),
        binding.atomics[0].occurrence.view().original_box().id()
    );
    assert_eq!(boxes[0].rect(), expected);
    let output = node(batch.unrounded_entries(), binding.atomics[0].node);
    assert_eq!(
        output.size,
        l::Size::new(n(expected.size.width), n(expected.size.height))
    );
}

fn qualify_native_pair<S: l::LayoutScalar>(
    child: &Fixture,
    input: l::NodeInputOf<S>,
    expected: [f64; 2],
    size: (f64, f64),
) {
    let mut host = Host::new(input);
    let binding = child_binding(child, 0, 1);
    host.push(
        Some(0),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(binding.origins[0].origin.clone())),
    );
    let batch = solve(&host, vec![binding], size.0, size.1);
    let pair = node(batch.unrounded_entries(), 0)
        .named_baselines
        .get(l::BaselineName::Central);
    for (fact, expected) in [(pair.first(), expected[0]), (pair.last(), expected[1])] {
        let coordinate =
            l::BaselineCoordinateOf::try_new(l::PhysicalAxis::Horizontal, n(expected)).unwrap();
        assert_eq!(fact, l::BaselineFactOf::Available(coordinate));
    }
}

fn sources_and_signed_margins<S: l::LayoutScalar>() {
    let child = fixture();
    let mut small = child_input::<S>();
    small.size.height = px(4.0);
    qualify_native_pair(&child, small.clone(), [2.0, 6.0], (6.0, 4.0));
    let parent = captured_parent(
        &child.bytes,
        t::Size::new(6.0, 4.0),
        false,
        t::Id::from_u64(2),
    );
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        for (source, b, h) in [
            (l::BaselineSource::First, 2.0, 10.0),
            (l::BaselineSource::Last, 6.0, 14.0),
        ] {
            let mut input = small.clone();
            input.baseline_source = source;
            let scene = make_scene::<S>(&parent, mode, (40.0, 12.0), input, Some(&child));
            let batch = run(&scene, available(40.0, 12.0), &[]).unwrap();
            let c = if mode == l::WritingMode::VerticalRl {
                32.0
            } else {
                h - 8.0
            };
            atomic_rect(
                &batch,
                &scene.bindings[0],
                t::Rect::new(c - b, 4.0, 6.0, 4.0),
            );
            assert_central(node(batch.unrounded_entries(), 0).named_baselines, c);
            let lines = receipts(&batch, &scene.bindings[0]);
            assert_eq!(lines.len(), 1);
            assert_eq!(
                (
                    lines[&0].feedback().advance,
                    lines[&0].feedback().baseline,
                    lines[&0].feedback().line_height
                ),
                (12.0, 8.0, h)
            );
            let selected = lines[&0].selected_line();
            let t::AtomicLineAllocation::InFlow { flow, .. } =
                selected.request().atomics[0].allocation
            else {
                panic!("actual InFlow allocation required")
            };
            assert_eq!((flow.before_baseline, flow.after_baseline), (6.0 - b, b));
        }
    }

    let empty = child_input::<S>();
    let boxes = Host::new(empty.clone());
    let qualifier = l::compute_layout(
        &boxes,
        0,
        l::LayoutRootRequestOf::viewport(available(6.0, 14.0)).unwrap(),
    )
    .unwrap();
    let pair = node(qualifier.unrounded_entries(), 0)
        .named_baselines
        .get(l::BaselineName::Central);
    assert_eq!(
        (pair.first(), pair.last()),
        (l::BaselineFactOf::Absent, l::BaselineFactOf::Absent)
    );
    let parent = super::parent(&child.bytes);
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        let mut input = empty.clone();
        input.margin = l::Edges {
            left: margin(-2.0),
            right: margin(4.0),
            top: margin(-3.0),
            bottom: margin(1.0),
        };
        let scene = make_scene::<S>(&parent, mode, (40.0, 20.0), input, None);
        let batch = run(&scene, available(40.0, 20.0), &[]).unwrap();
        let c = if mode == l::WritingMode::VerticalRl {
            32.0
        } else {
            4.0
        };
        atomic_rect(
            &batch,
            &scene.bindings[0],
            t::Rect::new(c - 6.0, 1.0, 6.0, 14.0),
        );
        let lines = receipts(&batch, &scene.bindings[0]);
        assert_eq!(
            (
                lines[&0].feedback().advance,
                lines[&0].feedback().baseline,
                lines[&0].feedback().line_height
            ),
            (20.0, 8.0, 12.0)
        );
        let slot = lines[&0].atomic_slots().next().unwrap();
        let t::AtomicLineAllocation::InFlow { flow, .. } = slot.allocation else {
            panic!("actual InFlow allocation required")
        };
        assert_eq!(
            (
                flow.advance,
                flow.occupied_start,
                flow.occupied_end,
                flow.before_baseline,
                flow.after_baseline
            ),
            (12.0, -3.0, 12.0, 4.0, 4.0)
        );
        assert_eq!(
            placed(&batch, 0)
                .run_views()
                .flat_map(|run| run
                    .glyphs()
                    .map(|g| run.origin().y + g.pen())
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [0.0, 16.0]
        );
    }
}

fn alignment_and_sizing<S: l::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = parent(&child.bytes);
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        let first = if mode == l::WritingMode::VerticalRl {
            32.0
        } else {
            2.0
        };
        for alignment in [l::VerticalAlign::Top, l::VerticalAlign::Bottom] {
            let mut input = child_input::<S>();
            input.vertical_align = alignment;
            let scene = make_scene::<S>(&parent, mode, (40.0, 22.0), input, Some(&child));
            let batch = run(&scene, available(40.0, 22.0), &[]).unwrap();
            atomic_rect(
                &batch,
                &scene.bindings[0],
                t::Rect::new(
                    first
                        + if alignment == l::VerticalAlign::Top {
                            2.0
                        } else {
                            -2.0
                        },
                    4.0,
                    6.0,
                    14.0,
                ),
            );
            let lines = receipts(&batch, &scene.bindings[0]);
            assert_eq!(
                (
                    lines[&0].feedback().baseline,
                    lines[&0].feedback().line_height
                ),
                (8.0, 10.0)
            );
            let slot = lines[&0].atomic_slots().next().unwrap();
            let t::AtomicLineAllocation::InFlow {
                alignment: actual, ..
            } = slot.allocation
            else {
                panic!("InFlow alignment required")
            };
            assert_eq!(
                actual,
                if alignment == l::VerticalAlign::Top {
                    t::LineBoxAlignment::Top
                } else {
                    t::LineBoxAlignment::Bottom
                }
            );
        }
        for (capacity, columns, atomic_line) in [(17.0, 3, 1), (18.0, 2, 0)] {
            let scene =
                make_scene::<S>(&parent, mode, (40.0, capacity), child_input(), Some(&child));
            let batch = run(&scene, available(40.0, capacity), &[]).unwrap();
            let text = placed(&batch, 0);
            assert_eq!(text.lines().len(), columns);
            let c = if mode == l::WritingMode::VerticalRl {
                first - 10.0 * [0.0, 1.0][atomic_line]
            } else {
                first + 10.0 * [0.0, 1.0][atomic_line]
            };
            atomic_rect(
                &batch,
                &scene.bindings[0],
                t::Rect::new(c - 2.0, if atomic_line == 0 { 4.0 } else { 0.0 }, 6.0, 14.0),
            );
            assert_eq!(text.inline_boxes()[0].line(), atomic_line);
            let lines = receipts(&batch, &scene.bindings[0]);
            let last = lines.values().last().unwrap();
            assert_eq!(
                last.selected_line().request().context.end,
                t::LineEnd::Paragraph
            );
            assert_eq!(last.feedback().advance, 4.0);
        }
        for (inline, width, height, columns) in [
            (l::AvailableOf::MIN_CONTENT, 30.0, 14.0, 3),
            (l::AvailableOf::MAX_CONTENT, 10.0, 22.0, 1),
        ] {
            let mut scene =
                make_scene::<S>(&parent, mode, (40.0, 22.0), child_input(), Some(&child));
            scene.host.change(0, |input| {
                input.size = l::Size::new(l::PreferredSizeOf::AUTO, l::PreferredSizeOf::AUTO)
            });
            let batch = run(
                &scene,
                l::Size::new(l::AvailableOf::MAX_CONTENT, inline),
                &[],
            )
            .unwrap();
            assert_eq!(
                node(batch.unrounded_entries(), 0).size,
                l::Size::new(n(width), n(height))
            );
            assert_eq!(placed(&batch, 0).lines().len(), columns);
            let c = if columns == 1 { 2.0 } else { 12.0 };
            atomic_rect(
                &batch,
                &scene.bindings[0],
                t::Rect::new(c - 2.0, if columns == 1 { 4.0 } else { 0.0 }, 6.0, 14.0),
            );
        }
        let only = captured_parent(
            &child.bytes,
            t::Size::new(6.0, 14.0),
            true,
            t::Id::from_u64(1),
        );
        let scene = make_scene::<S>(&only, mode, (40.0, 14.0), child_input(), Some(&child));
        let batch = run(&scene, available(40.0, 14.0), &[]).unwrap();
        let text = placed(&batch, 0);
        assert!(text.source().text().is_empty());
        assert_eq!(text.lines().len(), 1);
        assert_eq!(text.lines()[0].range(), t::Range::new(0, 0));
        assert_eq!(text.lines()[0].baseline(), first);
        assert_eq!(
            text.lines()[0].bounds(),
            t::Rect::new(first - 2.0, 0.0, 10.0, 14.0)
        );
        assert!(text.run_views().next().is_none());
        assert_eq!(text.nonformatted_lines().count(), 0);
        atomic_rect(
            &batch,
            &scene.bindings[0],
            t::Rect::new(first - 2.0, 0.0, 6.0, 14.0),
        );
        assert_eq!(
            text.hit(t::Point::new(first, 5.0)),
            t::Hit::InlineBox(t::Id::from_u64(1))
        );
    }
}

fn numeric_descendant<S: l::LayoutScalar>() -> l::NodeInputOf<S> {
    let mut input = root(
        l::WritingMode::HorizontalTb,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    );
    input.display = l::Display::InlineBlock;
    input.size = l::Size::new(px(2.0), px(8.0));
    input.atomic_inline_participation = Some(l::AtomicInlineParticipation::new(
        t::BidiLevel::try_new(0).unwrap(),
        l::InlineBreakOpportunity::Allowed,
    ));
    input
}

fn unknown_and_wrong_axis<S: l::LayoutScalar>() {
    let child = fixture();
    let parent = parent(&child.bytes);
    for unknown in [true, false] {
        let mut input = child_input::<S>();
        if unknown {
            input.display = l::Display::InlineGrid;
        } else {
            input.writing_mode = l::WritingMode::HorizontalTb;
        }
        let mut qualifier = Host::new(input.clone());
        if !unknown {
            qualifier.push(Some(0), l::LayoutInputOf::box_input(numeric_descendant()));
        }
        let qualified = l::compute_layout(
            &qualifier,
            0,
            l::LayoutRootRequestOf::viewport(available(6.0, 14.0)).unwrap(),
        )
        .unwrap();
        let pair = node(qualified.unrounded_entries(), 0)
            .named_baselines
            .get(l::BaselineName::Central);
        if unknown {
            assert_eq!(
                (pair.first(), pair.last()),
                (l::BaselineFactOf::Unknown, l::BaselineFactOf::Unknown)
            );
        } else {
            let coordinate =
                l::BaselineCoordinateOf::try_new(l::PhysicalAxis::Vertical, n(4.0)).unwrap();
            assert_eq!(
                (pair.first(), pair.last()),
                (
                    l::BaselineFactOf::Available(coordinate),
                    l::BaselineFactOf::Available(coordinate)
                )
            );
        }
        for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
            let mut scene = make_scene::<S>(&parent, mode, (40.0, 22.0), input.clone(), None);
            if !unknown {
                scene
                    .host
                    .push(Some(2), l::LayoutInputOf::box_input(numeric_descendant()));
            }
            let error = run(&scene, available(40.0, 22.0), &[]).unwrap_err();
            if unknown {
                assert!(
                    matches!(
                        error.kind(),
                        l::LayoutErrorKindOf::MissingContext(
                            l::LayoutMissingContext::NamedInlineBaseline
                        )
                    ),
                    "{error:?}"
                );
            } else {
                assert!(
                    matches!(
                        error.kind(),
                        l::LayoutErrorKindOf::InvalidInput(
                            l::LayoutInvalidInputOf::InlineBaseline(
                                l::InlineBaselineError::AxisMismatch {
                                    expected: l::PhysicalAxis::Horizontal,
                                    actual: l::PhysicalAxis::Vertical
                                }
                            )
                        )
                    ),
                    "{error:?}"
                );
            }
        }
    }
}

fn frames_content_and_replay<S: l::LayoutScalar>() {
    let child = fixture();
    let parent = parent(&child.bytes);
    let wrapped_parent = captured_parent(
        &child.bytes,
        t::Size::new(6.0, 14.0),
        false,
        t::Id::from_u64(3),
    );
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        let c = if mode == l::WritingMode::VerticalRl {
            32.0
        } else {
            2.0
        };
        let fractional = wrapped::<S>(&wrapped_parent, mode, &child, true);
        let batch = run(&fractional, available(100.0, 100.0), &[]).unwrap();
        let c_global = if mode == l::WritingMode::VerticalRl {
            39.0
        } else {
            9.0
        };
        assert_eq!(
            node(batch.unrounded_entries(), 1).location,
            l::Point::new(n(7.25), n(9.5))
        );
        assert_eq!(placed(&batch, 1).lines()[0].baseline(), c_global);
        atomic_rect(
            &batch,
            &fractional.bindings[0],
            t::Rect::new(c_global - 2.0, 14.0, 6.0, 14.0),
        );
        let final_parent = node(batch.final_entries(), 1);
        let pair = final_parent.named_baselines.get(l::BaselineName::Central);
        let l::BaselineFactOf::Available(named) = pair.first() else {
            panic!("actual named frame required")
        };
        assert_eq!(
            final_parent.location.x.to_f64() + named.offset().to_f64(),
            c_global
        );

        let mut relative = child_input::<S>();
        relative.position = l::Position::Relative;
        relative.inset.left = margin(3.0);
        relative.inset.top = margin(2.0);
        let scene = make_scene::<S>(&parent, mode, (40.0, 22.0), relative, Some(&child));
        let batch = run(&scene, available(40.0, 22.0), &[]).unwrap();
        atomic_rect(
            &batch,
            &scene.bindings[0],
            t::Rect::new(c + 1.0, 6.0, 6.0, 14.0),
        );
        assert_eq!(placed(&batch, 0).lines()[0].baseline(), c);
        assert_eq!(placed(&batch, 2).lines()[0].baseline(), c + 3.0);
        assert_eq!(
            receipts(&batch, &scene.bindings[0])[&0].feedback().advance,
            22.0
        );

        let mut content = child_input::<S>();
        content.box_sizing = l::BoxSizing::BorderBox;
        content.border = l::Edges::all(length(1.0));
        content.padding = l::Edges {
            left: length(1.0),
            right: length(0.0),
            top: length(2.0),
            bottom: length(1.0),
        };
        content.overflow =
            l::ComputedOverflow::try_new(l::Overflow::Auto, l::Overflow::Auto).unwrap();
        // CSS Align3's block-scroll-container rule exports Auto's last set at
        // block-end (right6 here); the genuine first content reference is4.
        qualify_native_pair(&child, content.clone(), [4.0, 6.0], (6.0, 14.0));
        content.baseline_source = l::BaselineSource::First;
        qualify_native_pair(&child, content.clone(), [4.0, 4.0], (6.0, 14.0));
        let scene = make_scene::<S>(&parent, mode, (40.0, 22.0), content, Some(&child));
        let batch = run(&scene, available(40.0, 22.0), &[]).unwrap();
        let c = if mode == l::WritingMode::VerticalRl {
            32.0
        } else {
            4.0
        };
        atomic_rect(
            &batch,
            &scene.bindings[0],
            t::Rect::new(c - 4.0, 4.0, 6.0, 14.0),
        );
        let output = node(batch.unrounded_entries(), 2);
        let content_box = output.scroll_geometry.unwrap().content_box();
        assert_eq!(content_box.origin(), l::Point::new(n(2.0), n(3.0)));
        assert_eq!(content_box.size(), l::Size::new(n(3.0), n(9.0)));
        // The zero-gutter padding scrollport ends at x5, while the genuine
        // H4 child allocation extends from content-left2 through x6.
        let range = output.scroll_geometry.unwrap().physical_range();
        assert_eq!((range.x().minimum(), range.x().maximum()), (n(0.0), n(1.0)));
        assert_eq!((range.y().minimum(), range.y().maximum()), (n(0.0), n(0.0)));
        let line = &receipts(&batch, &scene.bindings[0])[&0];
        let slot = line.atomic_slots().next().unwrap();
        let t::AtomicLineAllocation::InFlow { box_geometry, .. } = slot.allocation else {
            panic!("InFlow content required")
        };
        assert_eq!(box_geometry.content.size, t::Size::new(9.0, 3.0));
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (8.0, 12.0)
        );

        replay_and_failure::<S>(&child, &wrapped_parent, mode);
    }
}

fn replay_and_failure<S: l::LayoutScalar>(child: &Fixture, parent: &Parent, mode: l::WritingMode) {
    let mut scene = wrapped::<S>(parent, mode, child, false);
    let mut leaf = root(
        l::WritingMode::VerticalLr,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    );
    leaf.block_flow_context = l::BlockFlowContext::Independent;
    leaf.size = l::Size::new(px(5.0), px(22.0));
    let leaf_node = scene
        .host
        .push(Some(0), l::LayoutInputOf::box_input(leaf.clone()));
    let cold = run(&scene, available(100.0, 22.0), &[]).unwrap();
    let cache = cold
        .cache_store_entries()
        .iter()
        .find(|row| row.node() == leaf_node)
        .expect("actual numeric sibling must participate in cache storage");
    cold.apply_to(&mut scene.host).unwrap();
    let hits = scene.host.hits[leaf_node as usize].get();
    let warm = run(&scene, available(100.0, 22.0), &[]).unwrap();
    assert!(scene.host.hits[leaf_node as usize].get() > hits);
    assert_eq!(semantic(&cold), semantic(&warm));
    warm.apply_to(&mut scene.host).unwrap();
    assert_eq!((scene.host.prepares.get(), scene.host.commits), (2, 2));
    let atomic = scene.bindings[0].atomics[0].node;
    scene
        .host
        .change(atomic, |input| input.size.height = px(18.0));
    let changed = run(&scene, available(100.0, 22.0), &[atomic]).unwrap();
    let centers = if mode == l::WritingMode::VerticalRl {
        [32.0, 22.0]
    } else {
        [2.0, 12.0]
    };
    assert_eq!(
        placed(&changed, 1)
            .lines()
            .iter()
            .map(|line| line.baseline())
            .collect::<Vec<_>>(),
        centers
    );
    atomic_rect(
        &changed,
        &scene.bindings[0],
        t::Rect::new(centers[0] - 2.0, 4.0, 6.0, 18.0),
    );
    let mut fresh = wrapped::<S>(parent, mode, child, false);
    fresh
        .host
        .change(atomic, |input| input.size.height = px(18.0));
    fresh.host.push(Some(0), l::LayoutInputOf::box_input(leaf));
    let fresh_batch = run(&fresh, available(100.0, 22.0), &[]).unwrap();
    assert_eq!(semantic(&changed), semantic(&fresh_batch));
    let changed_text = placed(&changed, 1);
    let fresh_text = placed(&fresh_batch, 1);
    assert_eq!(changed_text.lines(), fresh_text.lines());
    assert_eq!(changed_text.inline_boxes(), fresh_text.inline_boxes());
    let old_position = changed_text.positions().next().unwrap();
    let error = fresh_text.position_geometry(&old_position).unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&t::ErrorDetail::InvalidTextProtocol {
            reason: t::ProtocolError::ForeignPlacement
        })
    );
    changed.apply_to(&mut scene.host).unwrap();
    let published = semantic(scene.host.published.batch.as_ref().unwrap());
    let unrounded = scene.host.published.unrounded.clone();
    let cache_before = scene.host.published.caches[leaf_node as usize]
        .get_with_context(cache.input(), cache.context());
    assert_eq!(cache_before, Some(cache.output()));
    let inner_text = scene.bindings[1].nodes[0];
    scene.bindings.truncate(1);
    scene.host.change(atomic, |input| {
        input.writing_mode = l::WritingMode::HorizontalTb
    });
    scene.host.inputs[inner_text as usize] = l::LayoutInputOf::box_input(numeric_descendant());
    let error = run(&scene, available(100.0, 22.0), &[atomic, inner_text]).unwrap_err();
    assert!(
        matches!(
            error.kind(),
            l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::InlineBaseline(
                l::InlineBaselineError::AxisMismatch {
                    expected: l::PhysicalAxis::Horizontal,
                    actual: l::PhysicalAxis::Vertical
                }
            ))
        ),
        "{error:?}"
    );
    assert_eq!(
        semantic(scene.host.published.batch.as_ref().unwrap()),
        published
    );
    assert_eq!(scene.host.published.unrounded, unrounded);
    assert_eq!(
        scene.host.published.caches[leaf_node as usize]
            .get_with_context(cache.input(), cache.context()),
        cache_before
    );
    assert_eq!(placed(&cold, 1).lines().len(), 1);
    assert_eq!(placed(&warm, 1).lines().len(), 1);
    assert_eq!(
        placed(&cold, 1).inline_boxes()[0].rect().size,
        t::Size::new(6.0, 14.0)
    );
    assert_eq!(
        placed(scene.host.published.batch.as_ref().unwrap(), 1).inline_boxes()[0]
            .rect()
            .size,
        t::Size::new(6.0, 18.0)
    );
}

#[test]
fn native_actual_first_last_and_absent_signed_margin_sources_f32() {
    sources_and_signed_margins::<f32>();
}
#[test]
fn native_actual_first_last_and_absent_signed_margin_sources_f64() {
    sources_and_signed_margins::<f64>();
}
#[test]
fn native_alignment_finite_intrinsic_and_source_free_atomics_f32() {
    alignment_and_sizing::<f32>();
}
#[test]
fn native_alignment_finite_intrinsic_and_source_free_atomics_f64() {
    alignment_and_sizing::<f64>();
}
#[test]
fn native_real_unknown_and_incompatible_child_names_refuse_f32() {
    unknown_and_wrong_axis::<f32>();
}
#[test]
fn native_real_unknown_and_incompatible_child_names_refuse_f64() {
    unknown_and_wrong_axis::<f64>();
}
#[test]
fn native_frames_content_actual_cache_replay_and_failed_publication_f32() {
    frames_content_and_replay::<f32>();
}
#[test]
fn native_frames_content_actual_cache_replay_and_failed_publication_f64() {
    frames_content_and_replay::<f64>();
}

fn physical_none_signed_margins<S: l::LayoutScalar>() {
    let font = fixture();
    let parent = super::parent(&font.bytes);
    let empty = child_input::<S>();
    let qualifier = Host::new(empty.clone());
    let qualified = l::compute_layout(
        &qualifier,
        0,
        l::LayoutRootRequestOf::viewport(available(6.0, 14.0)).unwrap(),
    )
    .unwrap();
    let pair = node(qualified.unrounded_entries(), 0)
        .named_baselines
        .get(l::BaselineName::Central);
    assert_eq!(
        (pair.first(), pair.last()),
        (l::BaselineFactOf::Absent, l::BaselineFactOf::Absent)
    );
    for (mode, cross) in [
        (l::WritingMode::VerticalRl, 32.0),
        (l::WritingMode::VerticalLr, 2.0),
    ] {
        let mut input = empty.clone();
        input.margin = l::Edges {
            left: margin(-2.0),
            right: margin(4.0),
            top: margin(-3.0),
            bottom: margin(1.0),
        };
        let mut scene = make_scene::<S>(&parent, mode, (40.0, 20.0), input, None);
        scene.host.change(0, |input| {
            input.inline_baseline_context = l::InlineBaselineContext::Physical;
        });
        let batch = run(&scene, available(40.0, 20.0), &[]).unwrap();
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(
                node(entries, 0).named_baselines,
                l::NamedBaselinesOf::unknown()
            );
            let child = node(entries, 2);
            assert_eq!(child.size, l::Size::new(n(6.0), n(14.0)));
            assert_eq!(child.location, l::Point::new(n(cross - 2.0), n(1.0)));
            let pair = child.named_baselines.get(l::BaselineName::Central);
            assert_eq!(
                (pair.first(), pair.last()),
                (l::BaselineFactOf::Absent, l::BaselineFactOf::Absent)
            );
        }
        let binding = &scene.bindings[0];
        atomic_rect(&batch, binding, t::Rect::new(cross - 2.0, 1.0, 6.0, 14.0));
        let lines = receipts(&batch, binding);
        assert_eq!(lines.len(), 1);
        let line = &lines[&0];
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (20.0, 8.0, 10.0)
        );
        let slots = line.atomic_slots().collect::<Vec<_>>();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].occurrence, binding.atomics[0].occurrence);
        assert_eq!(slots[0].origin, t::Point::new(4.0, 8.0));
        let t::AtomicLineAllocation::InFlow {
            flow, alignment, ..
        } = slots[0].allocation
        else {
            panic!("actual empty child must retain its InFlow allocation");
        };
        assert_eq!(alignment, t::LineBoxAlignment::Baseline);
        assert_eq!(
            (
                flow.advance,
                flow.occupied_start,
                flow.occupied_end,
                flow.before_baseline,
                flow.after_baseline
            ),
            (12.0, -3.0, 12.0, 8.0, 0.0)
        );
        let text = placed(&batch, 0);
        assert_eq!(text.lines()[0].baseline(), cross);
        assert_eq!(
            text.lines()[0].bounds(),
            t::Rect::new(cross - 2.0, 0.0, 10.0, 20.0)
        );
        assert_eq!(
            text.run_views()
                .flat_map(|run| run
                    .glyphs()
                    .map(|g| run.origin().y + g.pen())
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [0.0, 16.0]
        );
        for row in batch.unrounded_inline_fragments() {
            let owner = match row.owner() {
                l::InlineFragmentOwner::Node(node) if [1, 3].contains(node) => *node,
                _ => continue,
            };
            let fragment = row.fragment();
            assert_eq!(fragment.baseline(), l::Point::new(n(cross), n(0.0)));
            let rect = fragment.rect().unwrap();
            assert_eq!(
                rect.origin(),
                l::Point::new(n(cross - 2.0), n(if owner == 1 { 0.0 } else { 16.0 }))
            );
            assert_eq!(rect.size(), l::Size::new(n(4.0), n(4.0)));
        }
    }
}

#[test]
fn native_physical_none_reference_keeps_signed_margin_edge_f32() {
    physical_none_signed_margins::<f32>();
}
#[test]
fn native_physical_none_reference_keeps_signed_margin_edge_f64() {
    physical_none_signed_margins::<f64>();
}
