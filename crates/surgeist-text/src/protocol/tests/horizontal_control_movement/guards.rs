//! Reverse movement and physical-x selection use actual Horizontal rows.
use super::*;

fn real_lines(
    text: &str,
    at: usize,
    breaks: usize,
    anchors: &[(f64, f64)],
) -> (crate::Layout, Vec<FinalizedLine>) {
    let mut s = system();
    let bytes = s.registered_font_faces().next().unwrap().bytes().to_vec();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.hhea().unwrap().ascender().to_i16(), 800);
    assert_eq!(font.hhea().unwrap().descender().to_i16(), -200);
    let metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());
    for (ch, id) in [('A', 35), ('B', 36), ('C', 37), ('D', 38)] {
        assert_eq!(font.charmap().map(ch), Some(GlyphId::new(id)));
        assert_eq!(metrics.advance_width(GlyphId::new(id)), Some(1000.0));
    }
    let source = Source::identified(text, Id::from_u64(1519008), 1);
    let len = source.text().len();
    let a = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: Style {
                font: Font::new().family("Ahem"),
                size: 10.0,
                line_height: LineHeight::Absolute(10.0),
                white_space: WhiteSpace::Preserve,
                ..Style::default()
            },
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![crate::ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, len),
                aggregate: Range::new(0, len),
                kind: crate::ParagraphOriginKind::Authored,
            }],
            controls: (0..breaks)
                .map(|_| crate::ParagraphControl {
                    at,
                    kind: crate::ParagraphControlKind::ForcedBreak,
                })
                .collect(),
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: (0..breaks)
                .map(|control| crate::ParagraphAnchor {
                    item: crate::ParagraphAnchorItem::Control(control),
                    affinity: Affinity::After,
                })
                .collect(),
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let a = crate::test_wrapping::bind_missing(&a, crate::Wrap::Word);
    let p = s.prepare(&a, PreparationOptions::default()).unwrap();
    let controls = p
        .control_views()
        .map(|c| {
            assert_eq!(c.kind(), &crate::ParagraphControlKind::ForcedBreak);
            assert_eq!(c.source_position().get(), at);
            c.handle()
        })
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), breaks);
    let mut offers = p
        .opportunity_views()
        .filter(|o| o.kind() == PreparedBreakKind::Forced)
        .map(|o| o.handle())
        .collect::<Vec<_>>();
    assert_eq!(offers.len(), breaks);
    offers.push(
        p.opportunity_views()
            .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle(),
    );
    let mut start = p.boundary_views().next().unwrap().handle();
    let mut rows = Vec::new();
    for (i, offer) in offers.into_iter().enumerate() {
        let mut r = request(&p);
        r.start = start;
        r.end = offer.view().boundary();
        let first = r.start.view().participant_offset();
        let last = r.end.view().participant_offset();
        r.participants = p.participants().skip(first).take(last - first).collect();
        r.controls = r
            .participants
            .iter()
            .filter_map(|part| match part {
                PreparedParticipant::Control(c) => Some(ControlAllocation {
                    occurrence: c.clone(),
                    strut: None,
                    boundary: None,
                }),
                _ => None,
            })
            .collect();
        r.context.start = if i == 0 {
            LineStart::Paragraph
        } else {
            LineStart::Continuation
        };
        r.context.end = if i < breaks {
            LineEnd::Forced
        } else {
            LineEnd::Paragraph
        };
        r.context.available_inline = None;
        r.context.spacing = SpacingPolicy::None;
        r.termination = crate::PreparedTermination::Ordinary(offer);
        let line = p
            .select(r)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(line.feedback().line_height, 10.0);
        let slots = line
            .control_slots()
            .map(|c| c.control())
            .collect::<Vec<_>>();
        if i < breaks {
            assert_eq!(slots, [controls[i].clone()]);
            assert_eq!(
                reason(&line.nonformatted().unwrap_err()),
                &ProtocolError::NonFormattedForcedLine
            );
        } else {
            assert!(slots.is_empty());
            assert_eq!(
                reason(&line.nonformatted().unwrap_err()),
                &ProtocolError::NonFormattedInFlowContent
            );
        }
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.font_size(), 10.0);
        }
        start = line.selected_line().request().end.clone();
        rows.push(line);
    }
    assert_eq!(rows.len(), anchors.len());
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p,
        coverage: PlacementCoverage::WholeParagraph,
        entries: rows
            .iter()
            .zip(anchors)
            .map(|(line, &(inline_origin, baseline))| {
                crate::PlacementEntry::Formatted(LinePlacement {
                    line: line.clone(),
                    anchor: LineAnchor {
                        inline_origin,
                        baseline,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap();
    assert_eq!(layout.source().id(), Some(Id::from_u64(1519008)));
    assert_eq!(layout.source().revision(), 1);
    assert_eq!(
        layout.source().text().code_units(),
        text.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(layout.nonformatted_lines().count(), 0);
    (layout, rows)
}

fn ids(line: &FinalizedLine) -> Vec<u32> {
    line.run_views()
        .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
        .collect()
}

#[test]
fn previous_line_from_unique_document_end_refuses_real_control_target_and_keeps_handles() {
    let (layout, rows) = real_lines("AB", 1, 2, &[(20.0, 100.0), (20.0, 120.0), (20.0, 140.0)]);
    assert_eq!(
        rows.iter()
            .map(|r| r.feedback().advance)
            .collect::<Vec<_>>(),
        [10.0, 0.0, 10.0]
    );
    assert_eq!(ids(&rows[0]), [35]);
    assert!(ids(&rows[1]).is_empty());
    assert_eq!(ids(&rows[2]), [36]);
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|l| (l.range(), l.bounds()))
            .collect::<Vec<_>>(),
        [
            (Range::new(0, 1), Rect::new(20.0, 92.0, 10.0, 10.0)),
            (Range::new(1, 1), Rect::new(20.0, 112.0, 0.0, 10.0)),
            (Range::new(1, 2), Rect::new(20.0, 132.0, 10.0, 10.0)),
        ]
    );
    let input = Cursor::new(2, Affinity::After);
    assert_eq!(
        layout.cursor(input).unwrap().rect(),
        Rect::new(30.0, 132.0, 1.0, 10.0)
    );
    // Actual final glyph trailing edge projects Before; legacy end/After is
    // separately admitted above and has the same real physical coordinate.
    let exact_end = layout
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(2, Affinity::Before))
        .unwrap();
    assert_eq!(
        layout.position_geometry(&exact_end).unwrap().rect(),
        Rect::new(30.0, 132.0, 1.0, 10.0)
    );
    let exact_middle = layout
        .move_position(&exact_end, Movement::PreviousLine)
        .unwrap();
    assert_eq!(
        exact_middle.source_cursor(),
        Cursor::new(1, Affinity::After)
    );
    assert_eq!(
        layout.position_geometry(&exact_middle).unwrap().rect(),
        Rect::new(20.0, 112.0, 1.0, 10.0)
    );
    let ambiguity = ProtocolError::AmbiguousSourcePosition {
        index: 1,
        affinity: Affinity::After,
    };
    assert_eq!(
        reason(&layout.cursor(exact_middle.source_cursor()).unwrap_err()),
        &ambiguity
    );
    let error = layout
        .move_cursor(input, Movement::PreviousLine)
        .expect_err("the actual adjacent control-only target has an ambiguous source projection");
    assert_eq!(reason(&error), &ambiguity);
    assert_eq!(
        layout.position_geometry(&exact_middle).unwrap().rect(),
        Rect::new(20.0, 112.0, 1.0, 10.0)
    );
    assert_eq!(
        layout.position_geometry(&exact_end).unwrap().rect(),
        Rect::new(30.0, 132.0, 1.0, 10.0)
    );
    assert_eq!(layout.source().text().code_units(), [65, 66]);
}

#[test]
fn differently_anchored_horizontal_rows_use_physical_x_in_both_directions() {
    let (layout, rows) = real_lines("ABCD", 2, 1, &[(30.0, 100.0), (0.0, 120.0)]);
    assert_eq!(
        rows.iter()
            .map(|r| r.feedback().advance)
            .collect::<Vec<_>>(),
        [20.0, 20.0]
    );
    assert_eq!(ids(&rows[0]), [35, 36]);
    assert_eq!(ids(&rows[1]), [37, 38]);
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|l| (l.range(), l.bounds()))
            .collect::<Vec<_>>(),
        [
            (Range::new(0, 2), Rect::new(30.0, 92.0, 20.0, 10.0)),
            (Range::new(2, 4), Rect::new(0.0, 112.0, 20.0, 10.0)),
        ]
    );
    let input = Cursor::new(0, Affinity::After);
    assert_eq!(
        layout.cursor(input).unwrap().rect(),
        Rect::new(30.0, 92.0, 1.0, 10.0)
    );
    let start = layout
        .positions()
        .find(|p| p.source_cursor() == input)
        .unwrap();
    let exact_next = layout.move_position(&start, Movement::NextLine).unwrap();
    // Physical x30 is nearest actual row x20, not local-inline0 at x0.
    assert_eq!(exact_next.source_cursor(), Cursor::new(4, Affinity::Before));
    assert_eq!(
        layout.position_geometry(&exact_next).unwrap().rect(),
        Rect::new(20.0, 112.0, 1.0, 10.0)
    );
    assert_eq!(
        layout.move_cursor(input, Movement::NextLine).unwrap(),
        Cursor::new(4, Affinity::Before)
    );
    let exact_previous = layout
        .move_position(&exact_next, Movement::PreviousLine)
        .unwrap();
    assert_eq!(exact_previous.source_cursor(), input);
    assert_eq!(
        layout.position_geometry(&exact_previous).unwrap().rect(),
        Rect::new(30.0, 92.0, 1.0, 10.0)
    );
    assert_eq!(
        layout
            .move_cursor(Cursor::new(4, Affinity::Before), Movement::PreviousLine)
            .unwrap(),
        input
    );
    assert_eq!(
        layout.position_geometry(&start).unwrap().rect(),
        Rect::new(30.0, 92.0, 1.0, 10.0)
    );
    assert_eq!(layout.source().text().code_units(), [65, 66, 67, 68]);
}
