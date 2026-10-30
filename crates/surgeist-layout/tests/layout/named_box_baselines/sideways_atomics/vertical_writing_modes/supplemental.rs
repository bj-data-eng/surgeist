//! Mode-sensitive qualification reuses the accepted physical clockwise producers.
use super::super::supplemental::rtl_font;
use super::*;
use std::cell::RefCell;

const MODES: [l::WritingMode; 2] = [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr];
fn c(mode: l::WritingMode) -> f64 {
    if mode == l::WritingMode::VerticalRl {
        32.0
    } else {
        2.0
    }
}
fn length<S: l::LayoutScalar>(value: f64) -> l::LengthOf<S> {
    l::LengthOf::value(l::LengthPercentageOf::px(n(value)).unwrap())
}
fn standalone(parent: &Captured, kind: Option<t::InlineBoxKind>) -> t::FinalizedLine {
    let p = &parent.prepared;
    let end = p
        .opportunity_views()
        .find(|row| row.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let rtl = p.analyzed_source().direction() == t::Direction::RightToLeft;
    let line = p
        .select(t::LineSelectionRequest {
            start: p.boundary_views().next().unwrap().handle(),
            end: end.view().boundary(),
            termination: t::PreparedTermination::Ordinary(end),
            participants: p.participants().collect(),
            alternative: None,
            incoming: None,
            atomics: kind
                .map(|kind| vec![allocation(parent, kind)])
                .unwrap_or_default(),
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: t::LineContext {
                alignment_insets: t::LineAlignmentInsets::ZERO,
                formatted_line: t::FormattedLineContext::default(),
                base_direction: if rtl {
                    t::ParagraphBaseDirection::RightToLeft
                } else {
                    t::ParagraphBaseDirection::LeftToRight
                },
                line_box_direction: t::LineBoxDirection::LeftToRight,
                orientation: t::LineOrientation::SidewaysClockwise,
                start: t::LineStart::Paragraph,
                end: t::LineEnd::Paragraph,
                available_inline: Some(40.0),
                indent: 0.0,
                alignment: t::Alignment::Start,
                tabs: t::TabContext::NoTabs,
                spacing: t::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(surgeist_text::LineAllocations {
            spacing: t::SpacingAllocations::None,
            scopes: surgeist_text::ScopeAllocations::None,
        })
        .unwrap();
    let text_advance = if parent.source.text().is_empty() {
        0.0
    } else {
        20.0
    };
    let advance = text_advance
        + if kind == Some(t::InlineBoxKind::InFlow) {
            14.0
        } else {
            0.0
        };
    assert_eq!(line.feedback().advance, advance);
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (8.0, 10.0)
    );
    assert_eq!(line.prepared_text(), parent.prepared);
    assert!(
        line.run_views()
            .all(|run| run.glyph_basis() == t::GlyphBasis::Clockwise)
    );
    line
}
fn paragraph_binding<S: l::LayoutScalar>(
    host: &mut Host<S>,
    container: u32,
    parent: &Captured,
) -> l::TextParagraphBinding<u32> {
    let origin = parent.prepared.origin_views().next().unwrap().handle();
    let text = host.push(
        Some(container),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origin.clone())),
    );
    l::TextParagraphBinding {
        prepared: parent.prepared.clone(),
        container,
        nodes: vec![text],
        origins: vec![l::TextOriginBinding { origin, node: text }],
        atomics: vec![],
        controls: vec![],
        line_strut: None,
    }
}
fn paragraph_scene<S: l::LayoutScalar>(
    parent: &Captured,
    mode: l::WritingMode,
    direction: l::Direction,
    height: f64,
) -> (Host<S>, l::TextParagraphBinding<u32>) {
    let mut input = root(mode, direction, l::InlineBaselineContext::Physical);
    input.size = l::Size::new(px(40.0), px(height));
    let mut host = Host::new(input);
    let binding = paragraph_binding(&mut host, 0, parent);
    (host, binding)
}
fn atomic_scene<S: l::LayoutScalar>(
    parent: &Captured,
    child: &Captured,
    kind: t::InlineBoxKind,
    mode: l::WritingMode,
    direction: l::Direction,
    height: f64,
) -> (Host<S>, Vec<l::TextParagraphBinding<u32>>) {
    let (mut host, bindings) = scene(parent, child, kind, direction, height);
    host.change(0, |input| input.writing_mode = mode);
    (host, bindings)
}

fn directions<S: l::LayoutScalar>() {
    let bytes = rtl_font();
    for body in [
        t::ParagraphBaseDirection::LeftToRight,
        t::ParagraphBaseDirection::RightToLeft,
    ] {
        let rtl = body == t::ParagraphBaseDirection::RightToLeft;
        let parent = capture(
            &bytes,
            if rtl { "\u{5d0}\u{5d1}" } else { "AB" },
            None,
            body,
            t::LineOrientation::SidewaysClockwise,
            10.0,
        );
        let whole = standalone(&parent, None);
        assert!(
            parent
                .prepared
                .unit_views()
                .all(|unit| unit.level().get() == u8::from(rtl))
        );
        assert_eq!(
            whole
                .run_views()
                .flat_map(|run| run.glyphs().map(|g| g.id()).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            if rtl { vec![36, 35] } else { vec![35, 36] }
        );
        for mode in MODES {
            for direction in [l::Direction::Ltr, l::Direction::Rtl] {
                let (host, binding) = paragraph_scene::<S>(&parent, mode, direction, 10.0);
                let batch = run(&host, vec![binding], &[], available(40.0, 10.0)).unwrap();
                let text = placed(&batch, 0);
                let bs = if mode == l::WritingMode::VerticalRl {
                    [32.0, 22.0]
                } else {
                    [2.0, 12.0]
                };
                assert_eq!(
                    text.lines()
                        .iter()
                        .map(|line| line.baseline())
                        .collect::<Vec<_>>(),
                    bs,
                    "scalar={} body={body:?} containing={direction:?} mode={mode:?}",
                    std::any::type_name::<S>()
                );
                let glyphs = text
                    .run_views()
                    .flat_map(|run| {
                        assert_eq!(run.glyph_basis(), t::GlyphBasis::Clockwise);
                        run.glyphs()
                            .map(|g| (g.id(), run.origin().x, run.origin().y + g.pen()))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                // Capacity10 is exactly one glyph. Logical source order chooses
                // the columns; odd-level order still governs each line's carets.
                assert_eq!(glyphs, [(35, bs[0], 0.0), (36, bs[1], 0.0)]);
                assert_eq!(text.source(), &parent.source);
                assert_eq!(
                    text.analyzed_source().identity(),
                    parent.prepared.analyzed_source().identity()
                );
                unknown(node(batch.unrounded_entries(), 0).named_baselines);
                let start = text
                    .cursor(t::Cursor::new(0, t::Affinity::After))
                    .unwrap()
                    .rect();
                assert_eq!(
                    start,
                    t::Rect::new(bs[0] - 2.0, if rtl { 10.0 } else { 0.0 }, 10.0, 1.0)
                );
                let mut receipts = std::collections::BTreeMap::new();
                for row in batch.unrounded_inline_fragments() {
                    if matches!(row.owner(), l::InlineFragmentOwner::Node(1)) {
                        let fragment = row.fragment();
                        let line = fragment.finalized_fragment().finalized_line();
                        assert_eq!(
                            line.selected_line().request().context.orientation,
                            t::LineOrientation::SidewaysClockwise
                        );
                        assert_eq!(line.selected_line().request().context.base_direction, body);
                        assert_eq!(
                            line.selected_line().request().context.line_box_direction,
                            if direction == l::Direction::Ltr {
                                t::LineBoxDirection::LeftToRight
                            } else {
                                t::LineBoxDirection::RightToLeft
                            }
                        );
                        receipts
                            .entry(fragment.line_index())
                            .or_insert_with(|| line.clone());
                    }
                }
                assert_eq!(receipts.len(), 2);
            }
        }
    }
}

fn atomics<S: l::LayoutScalar>() {
    let bytes = rtl_font();
    let child = capture(
        AHEM,
        "A",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::Horizontal,
        5.0,
    );
    qualify_child::<S>(&child);
    // Four rows cover the changed mode choice together with normal bidi, both
    // containing directions, the three keyword alignments and inline alignment.
    let rows = [
        (
            false,
            l::Direction::Ltr,
            l::VerticalAlignOf::Baseline,
            l::TextAlign::Start,
            0.0,
        ),
        (
            true,
            l::Direction::Ltr,
            l::VerticalAlignOf::Top,
            l::TextAlign::Center,
            2.0,
        ),
        (
            false,
            l::Direction::Rtl,
            l::VerticalAlignOf::Bottom,
            l::TextAlign::End,
            -2.0,
        ),
        (
            true,
            l::Direction::Rtl,
            l::VerticalAlignOf::Baseline,
            l::TextAlign::Start,
            0.0,
        ),
    ];
    for kind in [t::InlineBoxKind::InFlow, t::InlineBoxKind::OutOfFlow] {
        for (rtl, direction, alignment, text_align, cross_offset) in rows {
            let body = if rtl {
                t::ParagraphBaseDirection::RightToLeft
            } else {
                t::ParagraphBaseDirection::LeftToRight
            };
            let parent = capture(
                &bytes,
                if rtl { "\u{5d0}\u{5d1}" } else { "AB" },
                Some((kind, t::Id::from_u64(2))),
                body,
                t::LineOrientation::SidewaysClockwise,
                10.0,
            );
            standalone(&parent, Some(kind));
            for mode in MODES {
                let (mut host, bindings) =
                    atomic_scene::<S>(&parent, &child, kind, mode, direction, 40.0);
                host.change(0, |input| {
                    input.text_align = text_align;
                    input.text_align_last = l::TextAlignLast::Auto;
                });
                host.change(2, |input| input.vertical_align = alignment);
                let batch = run(&host, bindings, &[], available(40.0, 40.0)).unwrap();
                let advance = if kind == t::InlineBoxKind::InFlow {
                    34.0
                } else {
                    20.0
                };
                let shift = match text_align {
                    l::TextAlign::Center => (40.0 - advance) / 2.0,
                    l::TextAlign::Start if direction == l::Direction::Rtl => 40.0 - advance,
                    _ => 0.0,
                };
                let x = if kind == t::InlineBoxKind::OutOfFlow {
                    if mode == l::WritingMode::VerticalRl {
                        34.0
                    } else {
                        0.0
                    }
                } else {
                    c(mode) + cross_offset
                };
                // AbsoluteAxis aligns an all-auto inline axis to inline-start.
                // Under containing RTL that is the bottom edge, so the actual
                // physical child starts one child-height before its static slot.
                let box_y = shift + 10.0
                    - if kind == t::InlineBoxKind::OutOfFlow && direction == l::Direction::Rtl {
                        14.0
                    } else {
                        0.0
                    };
                let border = t::Rect::new(x, box_y, 6.0, 14.0);
                let text = placed(&batch, 0);
                assert_eq!(text.inline_boxes()[0].rect(), border);
                assert_eq!(
                    text.hit(t::Point::new(x + 1.0, box_y + 2.0)),
                    t::Hit::InlineBox(t::Id::from_u64(2))
                );
                let line = receipt(&batch, 1);
                assert_eq!(
                    (
                        line.feedback().advance,
                        line.feedback().baseline,
                        line.feedback().line_height
                    ),
                    (advance, 8.0, 10.0)
                );
                let expected = explicit(
                    &parent,
                    line,
                    t::LineAnchor {
                        inline_origin: 0.0,
                        baseline: c(mode),
                    },
                    t::AtomicPlacement {
                        occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                        border,
                        content: border,
                        baseline: x,
                    },
                );
                assert_eq!(text.key(), expected.key());
                assert_eq!(text.source(), &parent.source);
                unknown(node(batch.unrounded_entries(), 0).named_baselines);
                unknown(node(batch.unrounded_entries(), 2).named_baselines);
                assert_eq!(placed(&batch, 2).lines()[0].baseline(), box_y + 4.0);
                assert!(
                    placed(&batch, 2)
                        .run_views()
                        .all(|run| run.glyph_basis() == t::GlyphBasis::Unrotated)
                );
                let ids = if rtl { [36, 35] } else { [35, 36] };
                let second = if kind == t::InlineBoxKind::InFlow {
                    24.0
                } else {
                    10.0
                };
                assert_eq!(
                    text.run_views()
                        .flat_map(|run| {
                            assert_eq!(run.glyph_basis(), t::GlyphBasis::Clockwise);
                            run.glyphs()
                                .map(|g| (g.id(), run.origin().y + g.pen()))
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                    [(ids[0], shift), (ids[1], shift + second)]
                );
            }
        }
    }
}

fn constraints<S: l::LayoutScalar>() {
    let parent = capture(
        AHEM,
        "AB",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::SidewaysClockwise,
        10.0,
    );
    standalone(&parent, None);
    for mode in MODES {
        for (inline, width, height, columns) in [
            (l::AvailableOf::MIN_CONTENT, 20.0, 10.0, 2),
            (l::AvailableOf::MAX_CONTENT, 10.0, 20.0, 1),
        ] {
            let (mut host, binding) = paragraph_scene::<S>(&parent, mode, l::Direction::Ltr, 40.0);
            host.change(0, |input| {
                input.size = l::Size::new(l::PreferredSizeOf::AUTO, l::PreferredSizeOf::AUTO)
            });
            let batch = run(
                &host,
                vec![binding],
                &[],
                l::Size::new(l::AvailableOf::MAX_CONTENT, inline),
            )
            .unwrap();
            assert_eq!(
                node(batch.unrounded_entries(), 0).size,
                l::Size::new(n(width), n(height))
            );
            assert_eq!(placed(&batch, 0).lines().len(), columns);
            unknown(node(batch.unrounded_entries(), 0).named_baselines);
        }
        // Finite block space anchors an actual AUTO20-wide Vrl root at x20,
        // while the Vlr root starts at x0. No provisional frame is fabricated.
        let (mut host, binding) = paragraph_scene::<S>(&parent, mode, l::Direction::Ltr, 10.0);
        host.change(0, |input| input.size.width = l::PreferredSizeOf::AUTO);
        let batch = run(&host, vec![binding], &[], available(40.0, 10.0)).unwrap();
        assert_eq!(
            node(batch.unrounded_entries(), 0).size,
            l::Size::new(n(20.0), n(10.0))
        );
        assert_eq!(
            node(batch.unrounded_entries(), 0).location,
            l::Point::new(
                n(if mode == l::WritingMode::VerticalRl {
                    20.0
                } else {
                    0.0
                }),
                n(0.0)
            )
        );
        assert_eq!(
            placed(&batch, 0)
                .lines()
                .iter()
                .map(|line| line.baseline())
                .collect::<Vec<_>>(),
            if mode == l::WritingMode::VerticalRl {
                vec![32.0, 22.0]
            } else {
                vec![2.0, 12.0]
            }
        );
    }
}

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
        let line = standalone(&parent, Some(kind));
        assert_eq!(line.has_in_flow_content(), kind == t::InlineBoxKind::InFlow);
        for mode in MODES {
            let mut input = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
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
                c(mode)
            } else if mode == l::WritingMode::VerticalRl {
                34.0
            } else {
                0.0
            };
            assert_eq!(
                node(batch.unrounded_entries(), 1).location,
                l::Point::new(n(x), n(0.0))
            );
            if kind == t::InlineBoxKind::InFlow {
                assert_eq!(text.lines().len(), 1);
                assert_eq!(
                    text.inline_boxes()[0].rect(),
                    t::Rect::new(x, 0.0, 6.0, 14.0)
                );
                unknown(node(batch.unrounded_entries(), 0).named_baselines);
            } else {
                assert!(text.lines().is_empty());
                assert_eq!(text.positions().count(), 0);
                let actual = text.nonformatted_lines().next().unwrap();
                assert_eq!(
                    actual.origin,
                    t::Point::new(
                        if mode == l::WritingMode::VerticalRl {
                            40.0
                        } else {
                            0.0
                        },
                        0.0
                    )
                );
                assert_eq!(actual.atomics[0].border, t::Rect::new(x, 0.0, 6.0, 14.0));
                assert_eq!(actual.atomics[0].baseline, x);
                absent(node(batch.unrounded_entries(), 0).named_baselines);
            }
        }
    }
}

fn wrapped<S: l::LayoutScalar>(
    parent: &Captured,
    child: &Captured,
    kind: t::InlineBoxKind,
    mode: l::WritingMode,
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
    let mut paragraph = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
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
        standalone(&parent, Some(kind));
        for mode in MODES {
            let (mut host, bindings) = wrapped::<S>(&parent, &child, kind, mode);
            let old = run(&host, bindings.clone(), &[], available(100.0, 100.0)).unwrap();
            assert_eq!(
                node(old.unrounded_entries(), 1).location,
                l::Point::new(n(7.25), n(9.5))
            );
            let (baseline, x) = if mode == l::WritingMode::VerticalRl {
                (
                    39.0,
                    if kind == t::InlineBoxKind::InFlow {
                        39.0
                    } else {
                        41.0
                    },
                )
            } else {
                (
                    9.0,
                    if kind == t::InlineBoxKind::InFlow {
                        9.0
                    } else {
                        7.0
                    },
                )
            };
            let border = t::Rect::new(x, 20.0, 6.0, 14.0);
            let expected = explicit(
                &parent,
                receipt(&old, 2),
                t::LineAnchor {
                    inline_origin: 10.0,
                    baseline,
                },
                t::AtomicPlacement {
                    occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                    border,
                    content: border,
                    baseline: x,
                },
            );
            assert_eq!(placed(&old, 1).key(), expected.key());
            old.apply_to(&mut host).unwrap();
            let warm = run(&host, bindings.clone(), &[], available(100.0, 100.0)).unwrap();
            assert_eq!(semantic(&old), semantic(&warm));
            assert_eq!(placed(&old, 1).key(), placed(&warm, 1).key());
            warm.apply_to(&mut host).unwrap();
            host.change(3, |input| input.size.height = px(18.0));
            let changed = run(&host, bindings, &[3], available(100.0, 100.0)).unwrap();
            let (mut fresh, bindings) = wrapped::<S>(&parent, &child, kind, mode);
            fresh.change(3, |input| input.size.height = px(18.0));
            let fresh = run(&fresh, bindings, &[], available(100.0, 100.0)).unwrap();
            assert_eq!(semantic(&changed), semantic(&fresh));
            assert_eq!(changed.final_entries(), fresh.final_entries());
            assert_eq!(placed(&changed, 1).key(), placed(&fresh, 1).key());
            assert_eq!(
                placed(&changed, 1).inline_boxes()[0].rect(),
                t::Rect::new(x, 20.0, 6.0, 18.0)
            );
            assert_eq!(
                receipt(&changed, 2).feedback().advance,
                if kind == t::InlineBoxKind::InFlow {
                    38.0
                } else {
                    20.0
                }
            );
            assert_eq!(placed(&old, 1).inline_boxes()[0].rect(), border);
            assert_eq!(placed(&warm, 1).inline_boxes()[0].rect(), border);
        }
    }
}

fn forced_line(
    parent: &Captured,
    start: t::PreparedBoundary,
    offer: t::PreparedOpportunity,
) -> t::FinalizedLine {
    let from = start.view().participant_offset();
    let end = offer.view().boundary();
    let to = end.view().participant_offset();
    let terminal = offer.view().kind() == t::PreparedBreakKind::ParagraphEnd;
    parent
        .prepared
        .select(t::LineSelectionRequest {
            start,
            end,
            termination: t::PreparedTermination::Ordinary(offer),
            participants: parent
                .prepared
                .participants()
                .skip(from)
                .take(to - from)
                .collect(),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: t::LineContext {
                alignment_insets: t::LineAlignmentInsets::ZERO,
                formatted_line: t::FormattedLineContext::default(),
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                line_box_direction: t::LineBoxDirection::LeftToRight,
                orientation: t::LineOrientation::SidewaysClockwise,
                start: if from == 0 {
                    t::LineStart::Paragraph
                } else {
                    t::LineStart::Continuation
                },
                end: if terminal {
                    t::LineEnd::Paragraph
                } else {
                    t::LineEnd::Forced
                },
                available_inline: Some(40.0),
                indent: 0.0,
                alignment: t::Alignment::Start,
                tabs: t::TabContext::NoTabs,
                spacing: t::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(surgeist_text::LineAllocations {
            spacing: t::SpacingAllocations::None,
            scopes: surgeist_text::ScopeAllocations::None,
        })
        .unwrap()
}
fn forced<S: l::LayoutScalar>() {
    let parent = capture(
        AHEM,
        "A\nB",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::SidewaysClockwise,
        10.0,
    );
    let p = &parent.prepared;
    let offer = p
        .opportunity_views()
        .find(|row| row.kind() == t::PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let first = forced_line(
        &parent,
        p.boundary_views().next().unwrap().handle(),
        offer.clone(),
    );
    let terminal = p
        .opportunity_views()
        .find(|row| row.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let second = forced_line(&parent, offer.view().boundary(), terminal);
    for line in [&first, &second] {
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (10.0, 8.0, 10.0)
        );
    }
    let reference = t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: p.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: [(first, 100.0), (second, 90.0)]
            .into_iter()
            .map(|(line, baseline)| {
                t::PlacementEntry::Formatted(t::LinePlacement {
                    line,
                    anchor: t::LineAnchor {
                        inline_origin: 0.0,
                        baseline,
                    },
                    orientation: t::LineOrientation::SidewaysClockwise,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap();
    assert_eq!(reference.source(), &parent.source);
    assert_eq!(reference.lines().len(), 2);
    for mode in MODES {
        let (host, binding) = paragraph_scene::<S>(&parent, mode, l::Direction::Ltr, 40.0);
        let batch = run(&host, vec![binding], &[], available(40.0, 40.0)).unwrap();
        let text = placed(&batch, 0);
        assert_eq!(
            text.lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [t::Range::new(0, 2), t::Range::new(2, 3)]
        );
        assert_eq!(
            text.lines()
                .iter()
                .map(|line| line.baseline())
                .collect::<Vec<_>>(),
            if mode == l::WritingMode::VerticalRl {
                vec![32.0, 22.0]
            } else {
                vec![2.0, 12.0]
            }
        );
        assert!(
            text.run_views()
                .all(|run| run.glyph_basis() == t::GlyphBasis::Clockwise)
        );
        unknown(node(batch.unrounded_entries(), 0).named_baselines);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Refusal {
    LaterBand,
}
struct ShapeHost<S: l::LayoutScalar> {
    inner: Host<S>,
    mode: l::WritingMode,
    capacity: f64,
    refuse: Cell<bool>,
    queries: RefCell<Vec<(l::FloatExclusionQueryOf<S>, bool)>>,
}
impl<S: l::LayoutScalar> l::Traverse for ShapeHost<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        l::Traverse::children(&self.inner, node)
    }
    fn child_count(&self, node: u32) -> usize {
        l::Traverse::child_count(&self.inner, node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        l::Traverse::child(&self.inner, node, index)
    }
}
impl<S: l::LayoutScalar> l::LayoutTree for ShapeHost<S> {
    type MeasureError = Refusal;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        l::LayoutTree::node_input(&self.inner, node)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        l::LayoutTree::layout_input(&self.inner, node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &l::ComputeInputOf<S>,
        context: l::CacheKeyContext,
    ) -> Option<l::ComputeOutputOf<S>> {
        l::LayoutTree::cache_get(&self.inner, node, input, context)
    }
    fn unrounded_layout(&self, node: u32) -> Option<l::NodeOutputOf<S>> {
        l::LayoutTree::unrounded_layout(&self.inner, node)
    }
    fn float_exclusion_interval(
        &self,
        node: u32,
        query: l::FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<l::FloatExclusionIntervalOf<S>>, Refusal>> {
        if node != 1 {
            return None;
        }
        assert_eq!(
            query.flow_axes(),
            l::FlowAxes::new(self.mode, l::Direction::Ltr)
        );
        assert_eq!(query.margin_box().origin(), l::Point::ZERO);
        assert_eq!(
            query.margin_box().size(),
            l::Size::new(n(40.0), n(self.capacity))
        );
        let nonempty = query.band_minimum() < query.band_maximum();
        let later = nonempty
            && if self.mode == l::WritingMode::VerticalRl {
                query.band_maximum() <= n(30.0)
            } else {
                query.band_minimum() >= n(10.0)
            };
        let refuse = self.refuse.get() && later;
        self.queries.borrow_mut().push((query, refuse));
        // An empty shape exclusion is a faithful successful provider result.
        // The failure stimulus is this later physical band, never call count.
        Some(if refuse {
            Err(Refusal::LaterBand)
        } else {
            Ok(None)
        })
    }
}
impl<S: l::LayoutScalar> ShapeHost<S> {
    fn compute(
        &self,
        bindings: &[l::TextParagraphBinding<u32>],
        changed: &[u32],
    ) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S, Refusal>> {
        let flow = l::LayoutFlowInput::try_new(
            self,
            0,
            bindings.to_vec(),
            l::WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        l::compute_layout_with_flow_invalidated(
            &flow,
            0,
            l::LayoutRootRequestOf::viewport(available(40.0, self.capacity)).unwrap(),
            changed,
        )
    }
}
fn shape_scene<S: l::LayoutScalar>(
    parent: &Captured,
    child: &Captured,
    kind: t::InlineBoxKind,
    mode: l::WritingMode,
) -> (ShapeHost<S>, Vec<l::TextParagraphBinding<u32>>) {
    let capacity = if kind == t::InlineBoxKind::InFlow {
        24.0
    } else {
        10.0
    };
    let mut input = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
    input.size = l::Size::new(px(40.0), px(capacity));
    let mut inner = Host::<S>::new(input);
    assert_eq!(
        inner.push(
            Some(0),
            l::LayoutInputOf::box_input(l::NodeInputOf {
                display: l::Display::Block,
                writing_mode: mode,
                float: l::Float::Left,
                float_exclusion: l::FloatExclusion::Shape,
                size: l::Size::new(px(40.0), px(capacity)),
                ..l::NodeInputOf::default()
            })
        ),
        1
    );
    let bindings = attach(&mut inner, 0, parent, child, kind);
    assert_eq!(bindings[0].nodes, [2, 3, 4]);
    (
        ShapeHost {
            inner,
            mode,
            capacity,
            refuse: Cell::new(false),
            queries: RefCell::new(Vec::new()),
        },
        bindings,
    )
}
fn same_text<S: l::LayoutScalar>(
    a: &l::CompletedLayoutBatchOf<u32, S>,
    b: &l::CompletedLayoutBatchOf<u32, S>,
) {
    assert_eq!(a.placed_text().len(), b.placed_text().len());
    for row in a.placed_text() {
        let actual = placed(b, row.container());
        assert_eq!(row.layout().source(), actual.source());
        assert_eq!(row.layout().key(), actual.key());
        let positions = |text: &t::Layout| {
            text.positions()
                .map(|p| {
                    (
                        p.source_cursor(),
                        text.position_geometry(&p).unwrap().rect(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(positions(row.layout()), positions(actual));
    }
}
fn late_refusal<S: l::LayoutScalar>() {
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
        standalone(&parent, Some(kind));
        for mode in MODES {
            let (mut host, bindings) = shape_scene::<S>(&parent, &child, kind, mode);
            let old = host.compute(&bindings, &[]).unwrap();
            let text = placed(&old, 0);
            assert_eq!(text.lines().len(), 2);
            assert_eq!(text.inline_boxes().len(), 1);
            assert_eq!(text.inline_boxes()[0].kind(), kind);
            assert_eq!(text.inline_boxes()[0].rect().size, t::Size::new(6.0, 14.0));
            if kind == t::InlineBoxKind::InFlow {
                assert_eq!(text.inline_boxes()[0].line(), 0);
                assert_eq!(receipt(&old, 2).feedback().advance, 24.0);
            }
            old.apply_to(&mut host.inner).unwrap();
            let cache = host.inner.published.caches.clone();
            let nodes = host.inner.published.unrounded.clone();
            let published = host.inner.published.batch.as_ref().unwrap().clone();
            host.queries.borrow_mut().clear();
            host.refuse.set(true);
            let error = host.compute(&bindings, &[0, 1]).unwrap_err();
            assert_eq!(error.operation(), l::LayoutOperation::FloatExclusionQuery);
            assert_eq!(
                error.site(),
                l::LayoutErrorSiteOf::ContainerSubject {
                    container: 0,
                    subject: 1
                }
            );
            assert!(matches!(
                error.kind(),
                l::LayoutErrorKindOf::Measurement(Refusal::LaterBand)
            ));
            let queries = host.queries.borrow();
            let failed = queries.iter().position(|(_, fail)| *fail).unwrap();
            let q = &queries[failed].0;
            assert!(q.band_minimum() < q.band_maximum());
            assert!(if mode == l::WritingMode::VerticalRl {
                q.band_maximum() <= n(30.0)
            } else {
                q.band_minimum() >= n(10.0)
            });
            assert!(queries[..failed].iter().any(|(q, fail)| !*fail
                && q.band_minimum() < q.band_maximum()
                && if mode == l::WritingMode::VerticalRl {
                    q.band_maximum() > n(30.0)
                } else {
                    q.band_minimum() < n(10.0)
                }));
            drop(queries);
            assert_eq!(host.inner.published.caches, cache);
            assert_eq!(host.inner.published.unrounded, nodes);
            let retained = host.inner.published.batch.as_ref().unwrap();
            assert_eq!(retained.unrounded_entries(), published.unrounded_entries());
            assert_eq!(retained.final_entries(), published.final_entries());
            assert_eq!(
                retained.unrounded_inline_fragments(),
                published.unrounded_inline_fragments()
            );
            assert_eq!(
                retained.final_inline_fragments(),
                published.final_inline_fragments()
            );
            for row in published.placed_text() {
                assert_eq!(
                    placed(retained, row.container())
                        .positions()
                        .collect::<Vec<_>>(),
                    row.layout().positions().collect::<Vec<_>>()
                );
            }
            same_text(retained, &published);
            host.refuse.set(false);
            let restored = host.compute(&bindings, &[0, 1]).unwrap();
            let (fresh, bindings) = shape_scene::<S>(&parent, &child, kind, mode);
            let fresh = fresh.compute(&bindings, &[]).unwrap();
            for other in [&old, &fresh] {
                assert_eq!(semantic(&restored), semantic(other));
                assert_eq!(restored.final_entries(), other.final_entries());
                same_text(&restored, other);
            }
        }
    }
}

#[test]
fn clockwise_vertical_capacity_ten_keeps_real_bidi_and_containing_directions() {
    directions::<f32>();
    directions::<f64>();
}
#[test]
fn clockwise_vertical_atoms_keep_physical_alignment_static_reference_and_source() {
    atomics::<f32>();
    atomics::<f64>();
}
#[test]
fn clockwise_vertical_intrinsic_and_auto_frames_keep_physical_progression() {
    constraints::<f32>();
    constraints::<f64>();
}
#[test]
fn clockwise_vertical_only_atomic_and_nonformatted_completion_keep_zero_extent() {
    completion::<f32>();
    completion::<f64>();
}
#[test]
fn clockwise_vertical_fractional_changed_atomic_replay_matches_fresh() {
    replay::<f32>();
    replay::<f64>();
}
#[test]
fn clockwise_vertical_forced_breaks_keep_actual_receipts_and_column_order() {
    forced::<f32>();
    forced::<f64>();
}
#[test]
fn clockwise_vertical_later_provider_refusal_preserves_all_applied_channels() {
    late_refusal::<f32>();
    late_refusal::<f64>();
}

fn guards<S: l::LayoutScalar>() {
    let cw = capture(
        AHEM,
        "AB",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::SidewaysClockwise,
        10.0,
    );
    standalone(&cw, None);
    for mode in MODES {
        for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
            let (mut host, binding) = paragraph_scene::<S>(&cw, mode, l::Direction::Ltr, 40.0);
            host.change(0, |input| {
                input.inline_baseline_context = l::InlineBaselineContext::Named(name)
            });
            let error = run(&host, vec![binding], &[], available(40.0, 40.0)).unwrap_err();
            assert_eq!(error.operation(), l::LayoutOperation::TextCapture);
            assert!(matches!(
                error.kind(),
                l::LayoutErrorKindOf::MissingContext(l::LayoutMissingContext::NamedInlineBaseline)
            ));
        }
    }
    let horizontal = capture(
        AHEM,
        "AB",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::Horizontal,
        10.0,
    );
    for (parent, mode, unsupported) in [
        (&horizontal, l::WritingMode::VerticalRl, true),
        (&horizontal, l::WritingMode::VerticalLr, true),
        (&cw, l::WritingMode::HorizontalTb, false),
        (&horizontal, l::WritingMode::SidewaysRl, false),
    ] {
        let (host, binding) = paragraph_scene::<S>(parent, mode, l::Direction::Ltr, 40.0);
        let error = run(&host, vec![binding], &[], available(40.0, 40.0)).unwrap_err();
        assert_eq!(error.operation(), l::LayoutOperation::TextSelection);
        let l::LayoutErrorKindOf::Text(text) = error.kind() else {
            panic!("expected actual orientation diagnostic: {error:?}")
        };
        let reason = if unsupported {
            t::ProtocolError::UnsupportedContext {
                field: t::ProtocolContextField::Orientation,
            }
        } else {
            t::ProtocolError::MismatchedContext {
                field: t::ProtocolContextField::Orientation,
            }
        };
        assert_eq!(
            text.detail(),
            Some(&t::ErrorDetail::InvalidTextProtocol { reason })
        );
    }
    let (host, binding) =
        paragraph_scene::<S>(&cw, l::WritingMode::SidewaysLr, l::Direction::Ltr, 40.0);
    let error = run(&host, vec![binding], &[], available(40.0, 40.0)).unwrap_err();
    assert_eq!(error.operation(), l::LayoutOperation::TextCapture);
    assert!(matches!(
        error.kind(),
        l::LayoutErrorKindOf::UnsupportedCapability(
            l::LayoutUnsupportedCapability::TextWritingMode(l::WritingMode::SidewaysLr)
        )
    ));
}
#[test]
fn clockwise_vertical_keeps_named_authority_and_other_orientation_refusals() {
    guards::<f32>();
    guards::<f64>();
}
