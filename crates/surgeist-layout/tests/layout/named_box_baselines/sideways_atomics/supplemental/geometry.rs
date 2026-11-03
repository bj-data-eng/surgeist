//! Independent finite/intrinsic and physical child geometry witnesses.
use super::*;

fn completion<S: l::LayoutScalar>() {
    let child = capture(
        AHEM,
        "A",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::Horizontal,
        5.0,
    );
    qualify_child::<S>(&child);
    for kind in [t::InlineBoxKind::InFlow, t::InlineBoxKind::OutOfFlow] {
        let parent = capture(
            AHEM,
            "",
            Some((kind, t::Id::from_u64(1))),
            t::ParagraphBaseDirection::LeftToRight,
            t::LineOrientation::SidewaysClockwise,
            10.0,
        );
        let line = selected(&parent, kind);
        assert_eq!(line.atomic_slots().count(), 1);
        assert_eq!(line.has_in_flow_content(), kind == t::InlineBoxKind::InFlow);
        let mut input = root(
            l::WritingMode::SidewaysRl,
            l::Direction::Ltr,
            l::InlineBaselineContext::Physical,
        );
        input.size = l::Size::new(px(40.0), px(40.0));
        let mut host = Host::<S>::new(input);
        host.push(Some(0), l::LayoutInputOf::box_input(child_input(kind)));
        let inner = child_binding(&child, 1, 2);
        host.push(
            Some(1),
            l::LayoutInputOf::inline_text(l::TextParticipation::new(
                inner.origins[0].origin.clone(),
            )),
        );
        let outer = l::TextParagraphBinding {
            inline_box_fragments: surgeist_layout::InlineBoxFragmentationDemand::NotRequested,
            prepared: parent.prepared.clone(),
            container: 0,
            nodes: vec![1],
            origins: vec![],
            atomics: vec![l::TextAtomicBinding {
                occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                node: 1,
            }],
            controls: vec![],
            line_strut: None,
        };
        let batch = run(&host, vec![outer, inner], &[], available(40.0, 40.0)).unwrap();
        let text = placed(&batch, 0);
        let x = if kind == t::InlineBoxKind::InFlow {
            32.0
        } else {
            34.0
        };
        assert_eq!(
            node(batch.unrounded_entries(), 1).location,
            l::Point::new(n(x), n(0.0))
        );
        if kind == t::InlineBoxKind::InFlow {
            assert_eq!(
                (
                    line.feedback().advance,
                    line.feedback().baseline,
                    line.feedback().line_height
                ),
                (14.0, 8.0, 10.0)
            );
            assert_eq!(text.lines().len(), 1);
            assert_eq!(
                text.inline_boxes()[0].rect(),
                t::Rect::new(32.0, 0.0, 6.0, 14.0)
            );
            unknown(node(batch.unrounded_entries(), 0).named_baselines);
        } else {
            // Keep the actual consumed receipt. Its tab context is part of
            // finalization identity even when this source contains no TAB.
            let completion = text.nonformatted_lines().next().unwrap().line.clone();
            let reference = t::Layout::from_finalized_lines(t::PlacementRequest {
                prepared: parent.prepared.clone(),
                coverage: t::PlacementCoverage::WholeParagraph,
                entries: vec![t::PlacementEntry::NonFormatted(
                    t::NonFormattedLinePlacement {
                        line: completion,
                        origin: t::Point::new(40.0, 0.0),
                        orientation: t::LineOrientation::SidewaysClockwise,
                        atomics: vec![t::AtomicPlacement {
                            occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                            border: t::Rect::new(34.0, 0.0, 6.0, 14.0),
                            content: t::Rect::new(34.0, 0.0, 6.0, 14.0),
                            baseline: 34.0,
                        }],
                    },
                )],
            })
            .unwrap();
            assert_eq!(text.key(), reference.key());
            assert!(text.lines().is_empty());
            assert_eq!(text.positions().count(), 0);
            assert_eq!(text.nonformatted_lines().count(), 1);
            assert_eq!(
                text.nonformatted_lines().next().unwrap().atomics[0].baseline,
                34.0
            );
            absent(node(batch.unrounded_entries(), 0).named_baselines);
        }
        assert_eq!(placed(&batch, 1).lines()[0].baseline(), 4.0);
    }
}

fn constraints<S: l::LayoutScalar>() {
    let child = capture(
        AHEM,
        "A",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::Horizontal,
        5.0,
    );
    qualify_child::<S>(&child);
    for kind in [t::InlineBoxKind::InFlow, t::InlineBoxKind::OutOfFlow] {
        let parent = capture(
            AHEM,
            "AB",
            Some((kind, t::Id::from_u64(2))),
            t::ParagraphBaseDirection::LeftToRight,
            t::LineOrientation::SidewaysClockwise,
            10.0,
        );
        qualify_parent(&parent, kind);
        let finite = if kind == t::InlineBoxKind::InFlow {
            [(23.0, 3), (24.0, 2)]
        } else {
            [(10.0, 2), (20.0, 1)]
        };
        for (height, columns) in finite {
            let (host, bindings) = scene::<S>(&parent, &child, kind, l::Direction::Ltr, height);
            let batch = run(&host, bindings, &[], available(40.0, height)).unwrap();
            assert_eq!(placed(&batch, 0).lines().len(), columns);
            assert_eq!(
                placed(&batch, 0).inline_boxes()[0].rect().size,
                t::Size::new(6.0, 14.0)
            );
            unknown(node(batch.unrounded_entries(), 0).named_baselines);
        }
        let (min_width, min_height, max_height) = if kind == t::InlineBoxKind::InFlow {
            (30.0, 14.0, 34.0)
        } else {
            (20.0, 10.0, 20.0)
        };
        for (space, width, height) in [
            (l::AvailableOf::MIN_CONTENT, min_width, min_height),
            (l::AvailableOf::MAX_CONTENT, 10.0, max_height),
        ] {
            let (mut host, bindings) = scene::<S>(&parent, &child, kind, l::Direction::Ltr, 40.0);
            host.change(0, |input| {
                input.size = l::Size::new(l::PreferredSizeOf::AUTO, l::PreferredSizeOf::AUTO)
            });
            let batch = run(
                &host,
                bindings,
                &[],
                l::Size::new(l::AvailableOf::MAX_CONTENT, space),
            )
            .unwrap();
            assert_eq!(
                node(batch.unrounded_entries(), 0).size,
                l::Size::new(n(width), n(height))
            );
            assert_eq!(
                placed(&batch, 0).inline_boxes()[0].rect().size,
                t::Size::new(6.0, 14.0)
            );
        }
    }
}

fn physical_boxes<S: l::LayoutScalar>() {
    let child = capture(
        AHEM,
        "A",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::Horizontal,
        5.0,
    );
    qualify_child::<S>(&child);
    for kind in [t::InlineBoxKind::InFlow, t::InlineBoxKind::OutOfFlow] {
        let parent = capture(
            AHEM,
            "AB",
            Some((kind, t::Id::from_u64(2))),
            t::ParagraphBaseDirection::LeftToRight,
            t::LineOrientation::SidewaysClockwise,
            10.0,
        );
        qualify_parent(&parent, kind);
        let (mut host, bindings) = scene::<S>(&parent, &child, kind, l::Direction::Ltr, 40.0);
        host.change(2, |input| {
            input.box_sizing = l::BoxSizing::BorderBox;
            input.margin = l::Edges {
                left: margin(-1.0),
                right: margin(2.0),
                top: margin(-2.0),
                bottom: margin(3.0),
            };
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
        let batch = run(&host, bindings, &[], available(40.0, 40.0)).unwrap();
        let x = if kind == t::InlineBoxKind::InFlow {
            31.0
        } else {
            32.0
        };
        let output = node(batch.unrounded_entries(), 2);
        assert_eq!(output.location, l::Point::new(n(x), n(8.0)));
        assert_eq!(output.size, l::Size::new(n(6.0), n(14.0)));
        let geometry = output.scroll_geometry.unwrap();
        assert_eq!(
            geometry.content_box().origin(),
            l::Point::new(n(2.0), n(3.0))
        );
        assert_eq!(geometry.content_box().size(), l::Size::new(n(3.0), n(9.0)));
        assert_eq!(geometry.physical_range().x().maximum(), n(2.0));
        let border = t::Rect::new(x, 8.0, 6.0, 14.0);
        let content = t::Rect::new(x + 2.0, 11.0, 3.0, 9.0);
        let line = receipt(&batch, 1);
        assert_eq!(
            line.feedback().advance,
            if kind == t::InlineBoxKind::InFlow {
                35.0
            } else {
                20.0
            }
        );
        let expected = explicit(
            &parent,
            line,
            t::LineAnchor {
                inline_origin: 0.0,
                baseline: 32.0,
            },
            t::AtomicPlacement {
                occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                border,
                content,
                baseline: 32.0,
            },
        );
        assert_eq!(placed(&batch, 0).key(), expected.key());
        assert_eq!(placed(&batch, 2).lines()[0].baseline(), 15.0);
        assert!(
            placed(&batch, 2)
                .run_views()
                .all(|run| run.glyph_basis() == t::GlyphBasis::Unrotated)
        );

        let (mut host, bindings) = scene::<S>(&parent, &child, kind, l::Direction::Ltr, 40.0);
        host.change(2, |input| {
            input.inset.left = margin(2.0);
            input.inset.top = margin(3.0);
        });
        let batch = run(&host, bindings, &[], available(40.0, 40.0)).unwrap();
        let (x, y) = if kind == t::InlineBoxKind::InFlow {
            (34.0, 13.0)
        } else {
            (2.0, 3.0)
        };
        assert_eq!(
            placed(&batch, 0).inline_boxes()[0].rect(),
            t::Rect::new(x, y, 6.0, 14.0)
        );
        assert_eq!(
            receipt(&batch, 1).feedback().advance,
            if kind == t::InlineBoxKind::InFlow {
                34.0
            } else {
                20.0
            }
        );
    }
}

fn wrapped<S: l::LayoutScalar>(
    parent: &Captured,
    child: &Captured,
    kind: t::InlineBoxKind,
) -> (Host<S>, Vec<l::TextParagraphBinding<u32>>) {
    let mut input = root(
        l::WritingMode::HorizontalTb,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    );
    input.size = l::Size::new(px(100.0), px(100.0));
    input.padding.left = length(7.25);
    input.padding.top = length(9.5);
    let mut host = Host::new(input);
    let mut paragraph = root(
        l::WritingMode::SidewaysRl,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    );
    paragraph.size = l::Size::new(px(40.0), px(40.0));
    let container = host.push(Some(0), l::LayoutInputOf::box_input(paragraph));
    let bindings = attach(&mut host, container, parent, child, kind);
    (host, bindings)
}
fn replay<S: l::LayoutScalar>() {
    let child = capture(
        AHEM,
        "A",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::Horizontal,
        5.0,
    );
    qualify_child::<S>(&child);
    for kind in [t::InlineBoxKind::InFlow, t::InlineBoxKind::OutOfFlow] {
        let parent = capture(
            AHEM,
            "AB",
            Some((kind, t::Id::from_u64(3))),
            t::ParagraphBaseDirection::LeftToRight,
            t::LineOrientation::SidewaysClockwise,
            10.0,
        );
        qualify_parent(&parent, kind);
        let (mut host, bindings) = wrapped::<S>(&parent, &child, kind);
        let cold = run(&host, bindings.clone(), &[], available(100.0, 100.0)).unwrap();
        assert_eq!(
            node(cold.unrounded_entries(), 1).location,
            l::Point::new(n(7.25), n(9.5))
        );
        let x = if kind == t::InlineBoxKind::InFlow {
            39.0
        } else {
            41.0
        };
        let border = t::Rect::new(x, 20.0, 6.0, 14.0);
        let expected = explicit(
            &parent,
            receipt(&cold, 2),
            t::LineAnchor {
                inline_origin: 10.0,
                baseline: 39.0,
            },
            t::AtomicPlacement {
                occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                border,
                content: border,
                baseline: x,
            },
        );
        assert_eq!(placed(&cold, 1).key(), expected.key());
        cold.apply_to(&mut host).unwrap();
        let warm = run(&host, bindings.clone(), &[], available(100.0, 100.0)).unwrap();
        assert_eq!(semantic(&cold), semantic(&warm));
        assert_eq!(placed(&cold, 1).key(), placed(&warm, 1).key());
        warm.apply_to(&mut host).unwrap();
        host.change(3, |input| input.size.height = px(18.0));
        let changed = run(&host, bindings, &[3], available(100.0, 100.0)).unwrap();
        let (mut fresh, bindings) = wrapped::<S>(&parent, &child, kind);
        fresh.change(3, |input| input.size.height = px(18.0));
        let fresh = run(&fresh, bindings, &[], available(100.0, 100.0)).unwrap();
        assert_eq!(semantic(&changed), semantic(&fresh));
        assert_eq!(changed.final_entries(), fresh.final_entries());
        assert_eq!(placed(&changed, 1).key(), placed(&fresh, 1).key());
        assert_eq!(
            placed(&changed, 1).inline_boxes()[0].rect(),
            t::Rect::new(x, 20.0, 6.0, 18.0)
        );
        assert_eq!(placed(&changed, 1).lines().len(), 1);
        assert_eq!(
            receipt(&changed, 2).feedback().advance,
            if kind == t::InlineBoxKind::InFlow {
                38.0
            } else {
                20.0
            }
        );
        assert_eq!(placed(&cold, 1).inline_boxes()[0].rect(), border);
        assert_eq!(placed(&warm, 1).inline_boxes()[0].rect(), border);
    }
}

fn mappings<S: l::LayoutScalar>() {
    let child = capture(
        AHEM,
        "A",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::Horizontal,
        5.0,
    );
    qualify_child::<S>(&child);
    for kind in [t::InlineBoxKind::InFlow, t::InlineBoxKind::OutOfFlow] {
        let parent = capture(
            AHEM,
            "AB",
            Some((kind, t::Id::from_u64(2))),
            t::ParagraphBaseDirection::LeftToRight,
            t::LineOrientation::SidewaysClockwise,
            10.0,
        );
        qualify_parent(&parent, kind);
        let (host, bindings) = scene::<S>(&parent, &child, kind, l::Direction::Ltr, 40.0);
        let batch = run(&host, bindings, &[], available(40.0, 40.0)).unwrap();
        let text = placed(&batch, 0);
        let advance = if kind == t::InlineBoxKind::InFlow {
            34.0
        } else {
            20.0
        };
        assert_eq!(
            text.cursor(t::Cursor::new(0, t::Affinity::After))
                .unwrap()
                .rect(),
            t::Rect::new(30.0, 0.0, 10.0, 1.0)
        );
        assert_eq!(
            text.cursor(t::Cursor::new(2, t::Affinity::Before))
                .unwrap()
                .rect(),
            t::Rect::new(30.0, advance, 10.0, 1.0)
        );
        assert!(
            matches!(text.hit(t::Point::new(33.0,2.0)),t::Hit::Text(position) if position.source_cursor().index()==0)
        );
        let second = if kind == t::InlineBoxKind::InFlow {
            24.0
        } else {
            10.0
        };
        let mut ranges = Vec::new();
        for row in batch.unrounded_inline_fragments() {
            let (range, y) = match row.owner() {
                l::InlineFragmentOwner::Node(1) => (t::Range::new(0, 1), 0.0),
                l::InlineFragmentOwner::Node(3) => (t::Range::new(1, 2), second),
                _ => continue,
            };
            let fragment = row.fragment();
            let receipt = fragment.finalized_fragment();
            assert_eq!(receipt.view().aggregate_range(), Some(range));
            assert_eq!(receipt.view().local_range(), Some(range));
            assert_eq!(fragment.baseline(), l::Point::new(n(32.0), n(0.0)));
            assert_eq!(
                fragment.rect().unwrap().origin(),
                l::Point::new(n(30.0), n(y))
            );
            assert_eq!(
                fragment.rect().unwrap().size(),
                l::Size::new(n(10.0), n(10.0))
            );
            ranges.push(range);
        }
        ranges.sort_by_key(|range| range.start);
        assert_eq!(ranges, [t::Range::new(0, 1), t::Range::new(1, 2)]);
    }
}

#[test]
fn physical_clockwise_atomic_only_and_nonformatted_completion_keep_real_geometry() {
    completion::<f32>();
    completion::<f64>();
}
#[test]
fn physical_clockwise_atoms_preserve_finite_and_intrinsic_zero_or_nonzero_flow() {
    constraints::<f32>();
    constraints::<f64>();
}
#[test]
fn physical_clockwise_atoms_preserve_signed_margins_content_overflow_and_displacement() {
    physical_boxes::<f32>();
    physical_boxes::<f64>();
}
#[test]
fn physical_clockwise_atoms_replay_fractional_and_changed_child_geometry_like_fresh() {
    replay::<f32>();
    replay::<f64>();
}

#[test]
fn physical_clockwise_atoms_retain_source_fragments_carets_and_physical_hits() {
    mappings::<f32>();
    mappings::<f64>();
}
