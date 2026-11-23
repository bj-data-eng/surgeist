//! Nonformatted completion and existing physical absolute-box responsibilities.
use super::*;
use surgeist_text::font::FontBaseline;
mod late;

fn length<S: l::scalar::LayoutScalar>(value: f64) -> l::length::LengthOf<S> {
    l::length::LengthOf::value(l::length::LengthPercentageOf::px(n(value)).unwrap())
}
fn margin<S: l::scalar::LayoutScalar>(value: f64) -> l::length::LengthAutoOf<S> {
    l::length::LengthAutoOf::value(l::length::LengthPercentageOf::px(n(value)).unwrap())
}

fn attach<S: l::scalar::LayoutScalar>(
    host: &mut Host<S>,
    container: u32,
    parent: &Parent,
    child: &Fixture,
) -> Vec<l::text_layout::TextParagraphBinding<u32>> {
    let origins = parent
        .prepared
        .origin_views()
        .map(|row| row.handle())
        .collect::<Vec<_>>();
    let mut nodes = Vec::new();
    let mut bound = Vec::new();
    if let Some(origin) = origins.first() {
        let node = host.push(
            Some(container),
            l::node::LayoutInputOf::inline_text(l::text_layout::TextParticipation::new(
                origin.clone(),
            )),
        );
        nodes.push(node);
        bound.push(l::text_layout::TextOriginBinding {
            origin: origin.clone(),
            node,
        });
    }
    let atomic = host.push(
        Some(container),
        l::node::LayoutInputOf::box_input(absolute_input()),
    );
    nodes.push(atomic);
    if let Some(origin) = origins.get(1) {
        let node = host.push(
            Some(container),
            l::node::LayoutInputOf::inline_text(l::text_layout::TextParticipation::new(
                origin.clone(),
            )),
        );
        nodes.push(node);
        bound.push(l::text_layout::TextOriginBinding {
            origin: origin.clone(),
            node,
        });
    }
    let occurrence = parent.prepared.atomic_views().next().unwrap().handle();
    assert_eq!(
        occurrence.view().original_box().id(),
        t::source::Id::from_u64(u64::from(atomic))
    );
    let text = host.push(
        Some(atomic),
        l::node::LayoutInputOf::inline_text(l::text_layout::TextParticipation::new(
            child.prepared.origin_views().next().unwrap().handle(),
        )),
    );
    vec![
        l::text_layout::TextParagraphBinding {
            inline_box_fragments:
                surgeist_layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
            prepared: parent.prepared.clone(),
            container,
            nodes,
            origins: bound,
            atomics: vec![l::text_layout::TextAtomicBinding {
                occurrence,
                node: atomic,
            }],
            controls: vec![],
            line_strut: Some(STRUT),
        },
        child_binding(child, atomic, text),
    ]
}

fn compute_space<S: l::scalar::LayoutScalar>(
    host: &Host<S>,
    bindings: Vec<l::text_layout::TextParagraphBinding<u32>>,
    changed: &[u32],
    space: surgeist_layout::constraint::SizingRequestOf<S>,
) -> Result<l::batch::CompletedLayoutBatchOf<u32, S>, l::error::LayoutErrorOf<u32, S>> {
    let flow = l::text_layout::LayoutFlowInput::try_new(
        host,
        0,
        bindings,
        l::inline_wrapping::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    l::compute_layout_with_flow_invalidated(
        &flow,
        0,
        {
            let sizing_request = space;
            l::request::LayoutRootRequestOf::standalone(
                sizing_request,
                surgeist_layout::geometry::FlowAxes::new(
                    surgeist_layout::tree::LayoutTree::node_input(host, 0).writing_mode,
                    surgeist_layout::tree::LayoutTree::node_input(host, 0).direction,
                ),
                sizing_request
                    .orthogonal_fallback()
                    .initial_containing_block(),
            )
        },
        changed,
    )
}

fn receipt<S: l::scalar::LayoutScalar>(
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
    source: u32,
) -> t::finalized::FinalizedLine {
    batch
        .unrounded_inline_fragments()
        .iter()
        .find(|row| matches!(row.owner(), l::text_layout::InlineFragmentOwner::Node(node) if *node == source))
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line()
        .clone()
}

fn nonformatted<S: l::scalar::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::geometry::Size::new(6.0, 14.0),
        true,
        t::source::Id::from_u64(1),
    );
    let line = selected_parent(&parent);
    assert!(!line.has_in_flow_content());
    let completion = line.nonformatted().unwrap();
    let atomic = parent.prepared.atomic_views().next().unwrap().handle();
    let standalone = t::layout::Layout::from_finalized_lines(t::placement::PlacementRequest {
        prepared: parent.prepared.clone(),
        coverage: t::placement::PlacementCoverage::WholeParagraph,
        entries: vec![t::placement::PlacementEntry::NonFormatted(
            t::placement::NonFormattedLinePlacement {
                line: completion,
                origin: t::geometry::Point::new(100.0, 0.0),
                orientation: t::selection::LineOrientation::Vertical,
                atomics: vec![t::placement::AtomicPlacement {
                    occurrence: atomic.clone(),
                    border: t::geometry::Rect::new(34.0, 0.0, 6.0, 14.0),
                    content: t::geometry::Rect::new(34.0, 0.0, 6.0, 14.0),
                    baseline: 36.0,
                }],
            },
        )],
    })
    .unwrap();
    assert!(standalone.lines().is_empty());
    assert_eq!(standalone.positions().count(), 0);
    for (mode, static_x, child_x, b) in [
        (l::geometry::WritingMode::VerticalRl, 40.0, 34.0, 36.0),
        (l::geometry::WritingMode::VerticalLr, 0.0, 0.0, 2.0),
    ] {
        let mut input = root(mode, l::geometry::Direction::Ltr, central());
        input.size = l::geometry::Size::new(px(40.0), px(22.0));
        let mut host = Host::<S>::new(input);
        let bindings = attach(&mut host, 0, &parent, &child);
        let batch = compute(&host, bindings, &[]).unwrap();
        let text = placed(&batch, 0);
        assert!(text.lines().is_empty());
        assert!(text.inline_boxes().is_empty());
        assert_eq!(text.positions().count(), 0);
        let completions = text.nonformatted_lines().collect::<Vec<_>>();
        assert_eq!(completions.len(), 1);
        let actual = completions[0];
        assert_eq!(actual.origin, t::geometry::Point::new(static_x, 0.0));
        assert_eq!(actual.atomics.len(), 1);
        assert_eq!(
            actual.atomics[0],
            t::placement::AtomicPlacement {
                occurrence: atomic.clone(),
                border: t::geometry::Rect::new(child_x, 0.0, 6.0, 14.0),
                content: t::geometry::Rect::new(child_x, 0.0, 6.0, 14.0),
                baseline: b,
            }
        );
        for entries in [
            crate::layout::observation::phase(&batch, false),
            crate::layout::observation::phase(&batch, true),
        ] {
            assert_eq!(
                node(entries, 1).location,
                l::geometry::Point::new(n(child_x), n(0.0))
            );
            let pair = node(entries, 0).named_baselines.get(FontBaseline::Central);
            assert_eq!(
                (pair.first(), pair.last()),
                (
                    l::baseline::BaselineFactOf::Absent,
                    l::baseline::BaselineFactOf::Absent
                )
            );
        }
        assert!(
            batch
                .unrounded_inline_fragments()
                .iter()
                .all(|row| !matches!(
                    row.owner(),
                    l::text_layout::InlineFragmentOwner::Node(0 | 1)
                ))
        );
        assert_eq!(placed(&batch, 1).lines()[0].baseline(), b);
    }
}

fn inset_precedence<S: l::scalar::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::geometry::Size::new(6.0, 14.0),
        false,
        t::source::Id::from_u64(2),
    );
    qualify_parent(&parent);
    for (mode, c) in [
        (l::geometry::WritingMode::VerticalRl, 32.0),
        (l::geometry::WritingMode::VerticalLr, 2.0),
    ] {
        for end_only in [false, true] {
            let (mut host, bindings) = scene::<S>(&parent, &child, mode);
            host.change(2, |input| {
                input.inset.right = margin(5.0);
                input.inset.bottom = margin(3.0);
                if !end_only {
                    input.inset.left = margin(3.0);
                    input.inset.top = margin(2.0);
                }
            });
            let batch = compute(&host, bindings, &[]).unwrap();
            let (x, y) = if end_only { (29.0, 5.0) } else { (3.0, 2.0) };
            for entries in [
                crate::layout::observation::phase(&batch, false),
                crate::layout::observation::phase(&batch, true),
            ] {
                assert_eq!(
                    node(entries, 2).location,
                    l::geometry::Point::new(n(x), n(y))
                );
                assert_eq!(
                    node(entries, 2).size,
                    l::geometry::Size::new(n(6.0), n(14.0))
                );
            }
            let border = t::geometry::Rect::new(x, y, 6.0, 14.0);
            let expected = explicit_parent(&parent, receipt(&batch, 1), c, border, border, x + 2.0);
            assert_eq!(placed(&batch, 0).key(), expected.key());
            assert_eq!(placed(&batch, 2).lines()[0].baseline(), x + 2.0);
            assert_eq!(receipt(&batch, 1).feedback().advance, 8.0);
        }
    }
}

fn physical_content<S: l::scalar::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::geometry::Size::new(6.0, 14.0),
        false,
        t::source::Id::from_u64(2),
    );
    qualify_parent(&parent);
    for (mode, c, raw_x, x, b) in [
        (l::geometry::WritingMode::VerticalRl, 32.0, 31.5, 32.0, 36.0),
        (l::geometry::WritingMode::VerticalLr, 2.0, -1.5, -1.0, 3.0),
    ] {
        let (mut host, bindings) = scene::<S>(&parent, &child, mode);
        host.change(2, |input| {
            input.box_sizing = l::box_model::BoxSizing::BorderBox;
            input.margin.left = margin(-1.5);
            input.margin.right = margin(2.5);
            input.margin.top = margin(-0.5);
            input.margin.bottom = margin(1.5);
            input.border = l::geometry::Edges::all(length(1.0));
            input.padding = l::geometry::Edges {
                left: length(1.0),
                right: length(0.0),
                top: length(2.0),
                bottom: length(1.0),
            };
            input.overflow = l::overflow::ComputedOverflow::try_new(
                l::overflow::Overflow::Auto,
                l::overflow::Overflow::Auto,
            )
            .unwrap();
        });
        let batch = compute(&host, bindings, &[]).unwrap();
        let output = node(crate::layout::observation::phase(&batch, false), 2);
        assert_eq!(output.location, l::geometry::Point::new(n(raw_x), n(3.5)));
        assert_eq!(output.size, l::geometry::Size::new(n(6.0), n(14.0)));
        assert_eq!(
            output.margin,
            l::geometry::Edges {
                left: n(-1.5),
                right: n(2.5),
                top: n(-0.5),
                bottom: n(1.5)
            }
        );
        let geometry = output.scroll_geometry.unwrap();
        assert_eq!(
            geometry.content_box().origin(),
            l::geometry::Point::new(n(2.0), n(3.0))
        );
        assert_eq!(
            geometry.content_box().size(),
            l::geometry::Size::new(n(3.0), n(9.0))
        );
        assert_eq!(geometry.physical_range().x().maximum(), n(1.0));
        let border = t::geometry::Rect::new(x, 4.0, 6.0, 14.0);
        let content = t::geometry::Rect::new(x + 2.0, 7.0, 3.0, 9.0);
        let expected = explicit_parent(&parent, receipt(&batch, 1), c, border, content, b);
        assert_eq!(placed(&batch, 0).key(), expected.key());
        assert_eq!(placed(&batch, 2).lines()[0].baseline(), b);
        assert_eq!(receipt(&batch, 1).feedback().advance, 8.0);
    }
}

fn wrapped<S: l::scalar::LayoutScalar>(
    parent: &Parent,
    child: &Fixture,
    mode: l::geometry::WritingMode,
) -> (Host<S>, Vec<l::text_layout::TextParagraphBinding<u32>>) {
    let mut input = root(
        l::geometry::WritingMode::VerticalLr,
        l::geometry::Direction::Ltr,
        central(),
    );
    input.size = l::geometry::Size::new(px(100.0), px(100.0));
    input.padding.left = length(7.25);
    input.padding.top = length(9.5);
    let mut host = Host::new(input);
    let mut paragraph = root(mode, l::geometry::Direction::Ltr, central());
    paragraph.size = l::geometry::Size::new(px(40.0), px(22.0));
    let container = host.push(Some(0), l::node::LayoutInputOf::box_input(paragraph));
    let bindings = attach(&mut host, container, parent, child);
    (host, bindings)
}

fn fractional_replay<S: l::scalar::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::geometry::Size::new(6.0, 14.0),
        false,
        t::source::Id::from_u64(3),
    );
    qualify_parent(&parent);
    for (mode, c, x, b) in [
        (l::geometry::WritingMode::VerticalRl, 39.0, 41.0, 43.0),
        (l::geometry::WritingMode::VerticalLr, 9.0, 7.0, 9.0),
    ] {
        let (mut host, bindings) = wrapped::<S>(&parent, &child, mode);
        let cold = compute_space(&host, bindings.clone(), &[], available(100.0, 100.0)).unwrap();
        assert_eq!(
            node(crate::layout::observation::phase(&cold, false), 1).location,
            l::geometry::Point::new(n(7.25), n(9.5))
        );
        let border = t::geometry::Rect::new(x, 14.0, 6.0, 14.0);
        let line = receipt(&cold, 2);
        let expected = t::layout::Layout::from_finalized_lines(t::placement::PlacementRequest {
            prepared: parent.prepared.clone(),
            coverage: t::placement::PlacementCoverage::WholeParagraph,
            entries: vec![t::placement::PlacementEntry::Formatted(
                t::placement::LinePlacement {
                    line,
                    anchor: t::placement::LineAnchor {
                        inline_origin: 10.0,
                        baseline: c,
                    },
                    orientation: t::selection::LineOrientation::Vertical,
                    atomics: vec![t::placement::AtomicPlacement {
                        occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                        border,
                        content: border,
                        baseline: b,
                    }],
                },
            )],
        })
        .unwrap();
        assert_eq!(placed(&cold, 1).key(), expected.key());
        cold.apply_to(&mut host).unwrap();
        let warm = compute_space(&host, bindings.clone(), &[], available(100.0, 100.0)).unwrap();
        assert_eq!(semantic(&cold, &host), semantic(&warm, &host));
        assert_eq!(placed(&cold, 1).key(), placed(&warm, 1).key());
        warm.apply_to(&mut host).unwrap();
        host.change(3, |input| input.size.height = px(18.0));
        let changed = compute_space(&host, bindings, &[3], available(100.0, 100.0)).unwrap();
        let (mut fresh, fresh_bindings) = wrapped::<S>(&parent, &child, mode);
        fresh.change(3, |input| input.size.height = px(18.0));
        let fresh = compute_space(&fresh, fresh_bindings, &[], available(100.0, 100.0)).unwrap();
        assert_eq!(semantic(&changed, &host), semantic(&fresh, &host));
        assert_eq!(
            changed
                .node_layouts()
                .map(|(node, view)| (
                    node,
                    match view {
                        surgeist_layout::fragmentation::NodeLayoutRefOf::SingleBox(view) =>
                            view.final_entry(),
                        surgeist_layout::fragmentation::NodeLayoutRefOf::NonBox(view) =>
                            view.final_entry(),
                        surgeist_layout::fragmentation::NodeLayoutRefOf::Fragments(_) =>
                            panic!("continuous fixture unexpectedly fragmented"),
                    }
                ))
                .collect::<Vec<_>>(),
            fresh
                .node_layouts()
                .map(|(node, view)| (
                    node,
                    match view {
                        surgeist_layout::fragmentation::NodeLayoutRefOf::SingleBox(view) =>
                            view.final_entry(),
                        surgeist_layout::fragmentation::NodeLayoutRefOf::NonBox(view) =>
                            view.final_entry(),
                        surgeist_layout::fragmentation::NodeLayoutRefOf::Fragments(_) =>
                            panic!("continuous fixture unexpectedly fragmented"),
                    }
                ))
                .collect::<Vec<_>>()
        );
        assert_eq!(placed(&changed, 1).key(), placed(&fresh, 1).key());
        assert_eq!(
            placed(&changed, 1).inline_boxes()[0].rect(),
            t::geometry::Rect::new(x, 14.0, 6.0, 18.0)
        );
        assert_eq!(placed(&changed, 1).lines().len(), 1);
        assert_eq!(receipt(&changed, 2).feedback().advance, 8.0);
        // Older immutable placements retain the original child extent.
        assert_eq!(placed(&cold, 1).inline_boxes()[0].rect(), border);
        assert_eq!(placed(&warm, 1).inline_boxes()[0].rect(), border);
    }
}

fn sizing<S: l::scalar::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::geometry::Size::new(6.0, 14.0),
        false,
        t::source::Id::from_u64(2),
    );
    qualify_parent(&parent);
    for mode in [
        l::geometry::WritingMode::VerticalRl,
        l::geometry::WritingMode::VerticalLr,
    ] {
        for (capacity, columns) in [(4.0, 2), (8.0, 1)] {
            let (mut host, bindings) = scene::<S>(&parent, &child, mode);
            host.change(0, |input| input.size.height = px(capacity));
            let batch = compute_space(&host, bindings, &[], available(40.0, capacity)).unwrap();
            assert_eq!(placed(&batch, 0).lines().len(), columns);
            assert_eq!(
                placed(&batch, 0).inline_boxes()[0].rect().size,
                t::geometry::Size::new(6.0, 14.0)
            );
        }
        for (inline, width, height, columns) in [
            (
                surgeist_layout::constraint::SizingOperation::MinContent,
                20.0,
                4.0,
                2,
            ),
            (
                surgeist_layout::constraint::SizingOperation::MaxContent,
                10.0,
                8.0,
                1,
            ),
        ] {
            let (mut host, bindings) = scene::<S>(&parent, &child, mode);
            host.change(0, |input| {
                input.size = l::geometry::Size::new(
                    l::sizing::PreferredSizeOf::AUTO,
                    l::sizing::PreferredSizeOf::AUTO,
                )
            });
            let batch = compute_space(
                &host,
                bindings,
                &[],
                crate::layout::native_request::sizing(
                    surgeist_layout::geometry::Size::new(None, None),
                    surgeist_layout::geometry::Size::new(
                        surgeist_layout::constraint::SizingOperation::Normal,
                        inline,
                    ),
                ),
            )
            .unwrap();
            assert_eq!(
                node(crate::layout::observation::phase(&batch, false), 0).size,
                l::geometry::Size::new(n(width), n(height))
            );
            assert_eq!(placed(&batch, 0).lines().len(), columns);
            assert_eq!(
                placed(&batch, 0).inline_boxes()[0].rect().size,
                t::geometry::Size::new(6.0, 14.0)
            );
        }
        let (host, mut bindings) = scene::<S>(&parent, &child, mode);
        bindings[0].line_strut = None;
        let batch = compute(&host, bindings, &[]).unwrap();
        let line = receipt(&batch, 1);
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (8.0, 2.0, 4.0)
        );
        let (c, x, b) = if mode == l::geometry::WritingMode::VerticalRl {
            (38.0, 34.0, 36.0)
        } else {
            (2.0, 0.0, 2.0)
        };
        let border = t::geometry::Rect::new(x, 4.0, 6.0, 14.0);
        let expected = explicit_parent(&parent, line, c, border, border, b);
        assert_eq!(placed(&batch, 0).key(), expected.key());
    }
}

#[test]
fn native_upright_positioned_only_completion_keeps_zero_used_line_and_physical_baseline() {
    nonformatted::<f32>();
    nonformatted::<f64>();
}
#[test]
fn native_upright_positioned_physical_insets_override_static_edges() {
    inset_precedence::<f32>();
    inset_precedence::<f64>();
}
#[test]
fn native_upright_positioned_signed_margins_keep_physical_content_and_overflow() {
    physical_content::<f32>();
    physical_content::<f64>();
}
#[test]
fn native_upright_positioned_fractional_ancestor_and_changed_child_replay_match_fresh() {
    fractional_replay::<f32>();
    fractional_replay::<f64>();
}

#[test]
fn native_upright_positioned_zero_flow_preserves_finite_intrinsic_and_natural_metrics() {
    sizing::<f32>();
    sizing::<f64>();
}
