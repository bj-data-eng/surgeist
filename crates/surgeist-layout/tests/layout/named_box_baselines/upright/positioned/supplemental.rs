//! Nonformatted completion and existing physical absolute-box responsibilities.
use super::*;
mod late;

fn length<S: l::LayoutScalar>(value: f64) -> l::LengthOf<S> {
    l::LengthOf::value(l::LengthPercentageOf::px(n(value)).unwrap())
}
fn margin<S: l::LayoutScalar>(value: f64) -> l::LengthAutoOf<S> {
    l::LengthAutoOf::value(l::LengthPercentageOf::px(n(value)).unwrap())
}

fn attach<S: l::LayoutScalar>(
    host: &mut Host<S>,
    container: u32,
    parent: &Parent,
    child: &Fixture,
) -> Vec<l::TextParagraphBinding<u32>> {
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
            l::LayoutInputOf::inline_text(l::TextParticipation::new(origin.clone())),
        );
        nodes.push(node);
        bound.push(l::TextOriginBinding {
            origin: origin.clone(),
            node,
        });
    }
    let atomic = host.push(
        Some(container),
        l::LayoutInputOf::box_input(absolute_input()),
    );
    nodes.push(atomic);
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
        t::Id::from_u64(u64::from(atomic))
    );
    let text = host.push(
        Some(atomic),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(
            child.prepared.origin_views().next().unwrap().handle(),
        )),
    );
    vec![
        l::TextParagraphBinding {
            prepared: parent.prepared.clone(),
            container,
            nodes,
            origins: bound,
            atomics: vec![l::TextAtomicBinding {
                occurrence,
                node: atomic,
            }],
            controls: vec![],
            line_strut: Some(STRUT),
        },
        child_binding(child, atomic, text),
    ]
}

fn compute_space<S: l::LayoutScalar>(
    host: &Host<S>,
    bindings: Vec<l::TextParagraphBinding<u32>>,
    changed: &[u32],
    space: l::Size<l::AvailableOf<S>>,
) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S>> {
    let flow =
        l::LayoutFlowInput::try_new(host, 0, bindings, l::WrappingInput::default(), Vec::new())
            .unwrap();
    l::compute_layout_with_flow_invalidated(
        &flow,
        0,
        l::LayoutRootRequestOf::viewport(space).unwrap(),
        changed,
    )
}

fn receipt<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    source: u32,
) -> t::FinalizedLine {
    batch
        .unrounded_inline_fragments()
        .iter()
        .find(|row| matches!(row.owner(), l::InlineFragmentOwner::Node(node) if *node == source))
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line()
        .clone()
}

fn nonformatted<S: l::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::Size::new(6.0, 14.0),
        true,
        t::Id::from_u64(1),
    );
    let line = selected_parent(&parent);
    assert!(!line.has_in_flow_content());
    let completion = line.nonformatted().unwrap();
    let atomic = parent.prepared.atomic_views().next().unwrap().handle();
    let standalone = t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: parent.prepared.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: vec![t::PlacementEntry::NonFormatted(
            t::NonFormattedLinePlacement {
                line: completion,
                origin: t::Point::new(100.0, 0.0),
                orientation: t::LineOrientation::Vertical,
                atomics: vec![t::AtomicPlacement {
                    occurrence: atomic.clone(),
                    border: t::Rect::new(34.0, 0.0, 6.0, 14.0),
                    content: t::Rect::new(34.0, 0.0, 6.0, 14.0),
                    baseline: 36.0,
                }],
            },
        )],
    })
    .unwrap();
    assert!(standalone.lines().is_empty());
    assert_eq!(standalone.positions().count(), 0);
    for (mode, static_x, child_x, b) in [
        (l::WritingMode::VerticalRl, 40.0, 34.0, 36.0),
        (l::WritingMode::VerticalLr, 0.0, 0.0, 2.0),
    ] {
        let mut input = root(mode, l::Direction::Ltr, central());
        input.size = l::Size::new(px(40.0), px(22.0));
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
        assert_eq!(actual.origin, t::Point::new(static_x, 0.0));
        assert_eq!(actual.atomics.len(), 1);
        assert_eq!(
            actual.atomics[0],
            t::AtomicPlacement {
                occurrence: atomic.clone(),
                border: t::Rect::new(child_x, 0.0, 6.0, 14.0),
                content: t::Rect::new(child_x, 0.0, 6.0, 14.0),
                baseline: b,
            }
        );
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(node(entries, 1).location, l::Point::new(n(child_x), n(0.0)));
            let pair = node(entries, 0)
                .named_baselines
                .get(l::BaselineName::Central);
            assert_eq!(
                (pair.first(), pair.last()),
                (l::BaselineFactOf::Absent, l::BaselineFactOf::Absent)
            );
        }
        assert!(
            batch
                .unrounded_inline_fragments()
                .iter()
                .all(|row| !matches!(row.owner(), l::InlineFragmentOwner::Node(0 | 1)))
        );
        assert_eq!(placed(&batch, 1).lines()[0].baseline(), b);
    }
}

fn inset_precedence<S: l::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::Size::new(6.0, 14.0),
        false,
        t::Id::from_u64(2),
    );
    qualify_parent(&parent);
    for (mode, c) in [
        (l::WritingMode::VerticalRl, 32.0),
        (l::WritingMode::VerticalLr, 2.0),
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
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_eq!(node(entries, 2).location, l::Point::new(n(x), n(y)));
                assert_eq!(node(entries, 2).size, l::Size::new(n(6.0), n(14.0)));
            }
            let border = t::Rect::new(x, y, 6.0, 14.0);
            let expected = explicit_parent(&parent, receipt(&batch, 1), c, border, border, x + 2.0);
            assert_eq!(placed(&batch, 0).key(), expected.key());
            assert_eq!(placed(&batch, 2).lines()[0].baseline(), x + 2.0);
            assert_eq!(receipt(&batch, 1).feedback().advance, 8.0);
        }
    }
}

fn physical_content<S: l::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::Size::new(6.0, 14.0),
        false,
        t::Id::from_u64(2),
    );
    qualify_parent(&parent);
    for (mode, c, raw_x, x, b) in [
        (l::WritingMode::VerticalRl, 32.0, 31.5, 32.0, 36.0),
        (l::WritingMode::VerticalLr, 2.0, -1.5, -1.0, 3.0),
    ] {
        let (mut host, bindings) = scene::<S>(&parent, &child, mode);
        host.change(2, |input| {
            input.box_sizing = l::BoxSizing::BorderBox;
            input.margin.left = margin(-1.5);
            input.margin.right = margin(2.5);
            input.margin.top = margin(-0.5);
            input.margin.bottom = margin(1.5);
            input.border = l::Edges::all(length(1.0));
            input.padding = l::Edges {
                left: length(1.0),
                right: length(0.0),
                top: length(2.0),
                bottom: length(1.0),
            };
            input.overflow =
                l::ComputedOverflow::try_new(l::Overflow::Auto, l::Overflow::Auto).unwrap();
        });
        let batch = compute(&host, bindings, &[]).unwrap();
        let output = node(batch.unrounded_entries(), 2);
        assert_eq!(output.location, l::Point::new(n(raw_x), n(3.5)));
        assert_eq!(output.size, l::Size::new(n(6.0), n(14.0)));
        assert_eq!(
            output.margin,
            l::Edges {
                left: n(-1.5),
                right: n(2.5),
                top: n(-0.5),
                bottom: n(1.5)
            }
        );
        let geometry = output.scroll_geometry.unwrap();
        assert_eq!(
            geometry.content_box().origin(),
            l::Point::new(n(2.0), n(3.0))
        );
        assert_eq!(geometry.content_box().size(), l::Size::new(n(3.0), n(9.0)));
        assert_eq!(geometry.physical_range().x().maximum(), n(1.0));
        let border = t::Rect::new(x, 4.0, 6.0, 14.0);
        let content = t::Rect::new(x + 2.0, 7.0, 3.0, 9.0);
        let expected = explicit_parent(&parent, receipt(&batch, 1), c, border, content, b);
        assert_eq!(placed(&batch, 0).key(), expected.key());
        assert_eq!(placed(&batch, 2).lines()[0].baseline(), b);
        assert_eq!(receipt(&batch, 1).feedback().advance, 8.0);
    }
}

fn wrapped<S: l::LayoutScalar>(
    parent: &Parent,
    child: &Fixture,
    mode: l::WritingMode,
) -> (Host<S>, Vec<l::TextParagraphBinding<u32>>) {
    let mut input = root(l::WritingMode::VerticalLr, l::Direction::Ltr, central());
    input.size = l::Size::new(px(100.0), px(100.0));
    input.padding.left = length(7.25);
    input.padding.top = length(9.5);
    let mut host = Host::new(input);
    let mut paragraph = root(mode, l::Direction::Ltr, central());
    paragraph.size = l::Size::new(px(40.0), px(22.0));
    let container = host.push(Some(0), l::LayoutInputOf::box_input(paragraph));
    let bindings = attach(&mut host, container, parent, child);
    (host, bindings)
}

fn fractional_replay<S: l::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::Size::new(6.0, 14.0),
        false,
        t::Id::from_u64(3),
    );
    qualify_parent(&parent);
    for (mode, c, x, b) in [
        (l::WritingMode::VerticalRl, 39.0, 41.0, 43.0),
        (l::WritingMode::VerticalLr, 9.0, 7.0, 9.0),
    ] {
        let (mut host, bindings) = wrapped::<S>(&parent, &child, mode);
        let cold = compute_space(&host, bindings.clone(), &[], available(100.0, 100.0)).unwrap();
        assert_eq!(
            node(cold.unrounded_entries(), 1).location,
            l::Point::new(n(7.25), n(9.5))
        );
        let border = t::Rect::new(x, 14.0, 6.0, 14.0);
        let line = receipt(&cold, 2);
        let expected = t::Layout::from_finalized_lines(t::PlacementRequest {
            prepared: parent.prepared.clone(),
            coverage: t::PlacementCoverage::WholeParagraph,
            entries: vec![t::PlacementEntry::Formatted(t::LinePlacement {
                line,
                anchor: t::LineAnchor {
                    inline_origin: 10.0,
                    baseline: c,
                },
                orientation: t::LineOrientation::Vertical,
                atomics: vec![t::AtomicPlacement {
                    occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                    border,
                    content: border,
                    baseline: b,
                }],
            })],
        })
        .unwrap();
        assert_eq!(placed(&cold, 1).key(), expected.key());
        cold.apply_to(&mut host).unwrap();
        let warm = compute_space(&host, bindings.clone(), &[], available(100.0, 100.0)).unwrap();
        assert_eq!(semantic(&cold), semantic(&warm));
        assert_eq!(placed(&cold, 1).key(), placed(&warm, 1).key());
        warm.apply_to(&mut host).unwrap();
        host.change(3, |input| input.size.height = px(18.0));
        let changed = compute_space(&host, bindings, &[3], available(100.0, 100.0)).unwrap();
        let (mut fresh, fresh_bindings) = wrapped::<S>(&parent, &child, mode);
        fresh.change(3, |input| input.size.height = px(18.0));
        let fresh = compute_space(&fresh, fresh_bindings, &[], available(100.0, 100.0)).unwrap();
        assert_eq!(semantic(&changed), semantic(&fresh));
        assert_eq!(changed.final_entries(), fresh.final_entries());
        assert_eq!(placed(&changed, 1).key(), placed(&fresh, 1).key());
        assert_eq!(
            placed(&changed, 1).inline_boxes()[0].rect(),
            t::Rect::new(x, 14.0, 6.0, 18.0)
        );
        assert_eq!(placed(&changed, 1).lines().len(), 1);
        assert_eq!(receipt(&changed, 2).feedback().advance, 8.0);
        // Older immutable placements retain the original child extent.
        assert_eq!(placed(&cold, 1).inline_boxes()[0].rect(), border);
        assert_eq!(placed(&warm, 1).inline_boxes()[0].rect(), border);
    }
}

fn sizing<S: l::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::Size::new(6.0, 14.0),
        false,
        t::Id::from_u64(2),
    );
    qualify_parent(&parent);
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        for (capacity, columns) in [(4.0, 2), (8.0, 1)] {
            let (mut host, bindings) = scene::<S>(&parent, &child, mode);
            host.change(0, |input| input.size.height = px(capacity));
            let batch = compute_space(&host, bindings, &[], available(40.0, capacity)).unwrap();
            assert_eq!(placed(&batch, 0).lines().len(), columns);
            assert_eq!(
                placed(&batch, 0).inline_boxes()[0].rect().size,
                t::Size::new(6.0, 14.0)
            );
        }
        for (inline, width, height, columns) in [
            (l::AvailableOf::MIN_CONTENT, 20.0, 4.0, 2),
            (l::AvailableOf::MAX_CONTENT, 10.0, 8.0, 1),
        ] {
            let (mut host, bindings) = scene::<S>(&parent, &child, mode);
            host.change(0, |input| {
                input.size = l::Size::new(l::PreferredSizeOf::AUTO, l::PreferredSizeOf::AUTO)
            });
            let batch = compute_space(
                &host,
                bindings,
                &[],
                l::Size::new(l::AvailableOf::MAX_CONTENT, inline),
            )
            .unwrap();
            assert_eq!(
                node(batch.unrounded_entries(), 0).size,
                l::Size::new(n(width), n(height))
            );
            assert_eq!(placed(&batch, 0).lines().len(), columns);
            assert_eq!(
                placed(&batch, 0).inline_boxes()[0].rect().size,
                t::Size::new(6.0, 14.0)
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
        let (c, x, b) = if mode == l::WritingMode::VerticalRl {
            (38.0, 34.0, 36.0)
        } else {
            (2.0, 0.0, 2.0)
        };
        let border = t::Rect::new(x, 4.0, 6.0, 14.0);
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
