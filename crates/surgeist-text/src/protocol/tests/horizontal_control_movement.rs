//! Horizontal movement through genuine control-only formatted lines.
//! Existing Horizontal public APIs only; no Sideways implementation dependency.
use super::*;
use crate::{Affinity, Cursor, Id, LineHeight, Movement, Range, Rect};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
    raw::TableProvider,
};

#[test]
fn horizontal_next_line_refuses_an_ambiguous_real_control_only_target() {
    let mut s = system(); // Existing RegisteredOnly, owned original Ahem.
    let bytes = s.registered_font_faces().next().unwrap().bytes().to_vec();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.charmap().map(0x41_u32), Some(GlyphId::new(35)));
    assert_eq!(font.charmap().map(0x42_u32), Some(GlyphId::new(36)));
    assert_eq!(font.hhea().unwrap().ascender().to_i16(), 800);
    assert_eq!(font.hhea().unwrap().descender().to_i16(), -200);
    let metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());
    assert_eq!(metrics.advance_width(GlyphId::new(35)), Some(1000.0));
    assert_eq!(metrics.advance_width(GlyphId::new(36)), Some(1000.0));
    let source = Source::identified("AB", Id::from_u64(1515007), 1);
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
                local: Range::new(0, 2),
                aggregate: Range::new(0, 2),
                kind: crate::ParagraphOriginKind::Authored,
            }],
            controls: (0..2)
                .map(|_| crate::ParagraphControl {
                    at: 1,
                    kind: crate::ParagraphControlKind::ForcedBreak,
                })
                .collect(),
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: (0..2)
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
            assert_eq!(c.source_position().get(), 1);
            c.handle()
        })
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 2);
    let forces = p
        .opportunity_views()
        .filter(|o| o.kind() == PreparedBreakKind::Forced)
        .map(|o| o.handle())
        .collect::<Vec<_>>();
    assert_eq!(forces.len(), 2);
    assert_ne!(forces[0].view().boundary(), forces[1].view().boundary());
    let finish = |from: PreparedBoundary, end: PreparedOpportunity| {
        let mut r = request(&p);
        r.start = from;
        r.end = end.view().boundary();
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
        r.context.start = if first == 0 {
            LineStart::Paragraph
        } else {
            LineStart::Continuation
        };
        r.context.end = if end.view().kind() == PreparedBreakKind::Forced {
            LineEnd::Forced
        } else {
            LineEnd::Paragraph
        };
        r.context.available_inline = None;
        r.context.spacing = SpacingPolicy::None;
        r.termination = crate::PreparedTermination::Ordinary(end);
        p.select(r)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap()
    };
    let end = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let first = finish(
        p.boundary_views().next().unwrap().handle(),
        forces[0].clone(),
    );
    let middle = finish(forces[0].view().boundary(), forces[1].clone());
    let last = finish(forces[1].view().boundary(), end);
    for (line, advance, nonformatted_error) in [
        (&first, 10.0, ProtocolError::NonFormattedForcedLine),
        (&middle, 0.0, ProtocolError::NonFormattedForcedLine),
        (&last, 10.0, ProtocolError::NonFormattedInFlowContent),
    ] {
        assert_eq!(line.feedback().advance, advance);
        assert_eq!(line.feedback().line_height, 10.0);
        assert_eq!(
            reason(&line.nonformatted().unwrap_err()),
            &nonformatted_error
        );
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), bytes);
        }
    }
    assert_eq!(
        first
            .control_slots()
            .map(|c| c.control())
            .collect::<Vec<_>>(),
        [controls[0].clone()]
    );
    assert_eq!(
        middle
            .control_slots()
            .map(|c| c.control())
            .collect::<Vec<_>>(),
        [controls[1].clone()]
    );
    assert_eq!(last.control_slots().count(), 0);
    let ids = |line: &FinalizedLine| {
        line.run_views()
            .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(&first), [35]);
    assert!(ids(&middle).is_empty());
    assert_eq!(ids(&last), [36]);
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: [(first, 100.0), (middle, 120.0), (last, 140.0)]
            .into_iter()
            .map(|(line, baseline)| {
                crate::PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 20.0,
                        baseline,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap();
    assert_eq!(layout.source().id(), Some(Id::from_u64(1515007)));
    assert_eq!(layout.source().revision(), 1);
    assert_eq!(layout.source().text().code_units(), [65, 66]);
    assert_eq!(layout.nonformatted_lines().count(), 0);
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
    let original = Cursor::new(0, Affinity::After);
    assert_eq!(
        layout.cursor(original).unwrap().rect(),
        Rect::new(20.0, 92.0, 1.0, 10.0)
    );
    let start = layout
        .positions()
        .find(|p| p.source_cursor() == original)
        .unwrap();
    let middle = layout.move_position(&start, Movement::NextLine).unwrap();
    let last = layout.move_position(&middle, Movement::NextLine).unwrap();
    assert_eq!(middle.source_cursor(), Cursor::new(1, Affinity::After));
    assert_eq!(last.source_cursor(), Cursor::new(1, Affinity::After));
    assert_ne!(middle, last);
    assert_eq!(
        layout.position_geometry(&middle).unwrap().rect(),
        Rect::new(20.0, 112.0, 1.0, 10.0)
    );
    assert_eq!(
        layout.position_geometry(&last).unwrap().rect(),
        Rect::new(20.0, 132.0, 1.0, 10.0)
    );
    let ambiguity = ProtocolError::AmbiguousSourcePosition {
        index: 1,
        affinity: Affinity::After,
    };
    assert_eq!(
        reason(&layout.cursor(middle.source_cursor()).unwrap_err()),
        &ambiguity
    );
    // Only this detector distinguishes the potential legacy fallback. Returning
    // source2 skips the real adjacent target; source1 cannot identify it uniquely.
    let error = layout
        .move_cursor(original, Movement::NextLine)
        .expect_err("the actual adjacent control occurrence has an ambiguous source projection");
    assert_eq!(reason(&error), &ambiguity);
    assert_eq!(layout.source().text().code_units(), [65, 66]);
}

mod guards;
