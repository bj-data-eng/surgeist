//! Actual whitespace edge alignment, with original source and font custody.
use super::*;
use crate::{Affinity, Cursor, Range};
use skrifa::MetadataProvider;

mod additional_edges;

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn prepare(system: &mut System, text: &str, whitespace: WhiteSpace) -> PreparedText {
    let font = skrifa::FontRef::new(AHEM).unwrap();
    let cmap = font.charmap();
    for (code, glyph) in [(0x58_u32, 58), (0x59, 59), (0x20, 3), (0xa0, 153)] {
        assert_eq!(cmap.map(code).map(|id| id.to_u32()), Some(glyph));
    }
    let source = Source::new(text);
    let end = source.text().len();
    let input = ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        source: source.clone(),
        base_style: Style {
            size: 10.0,
            white_space: whitespace,
            ..style()
        },
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, end),
            aggregate: Range::new(0, end),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    system
        .prepare(
            &crate::test_wrapping::bind_missing(
                &AnalyzedSource::from_paragraph(input).unwrap(),
                crate::Wrap::Word,
            ),
            PreparationOptions::default(),
        )
        .unwrap()
}

fn centered(
    p: &PreparedText,
    start: PreparedBoundary,
    ending: PreparedOpportunity,
) -> FinalizedLine {
    let mut r = request(p);
    r.start = start;
    r.end = ending.view().boundary();
    let from = r.start.view().participant_offset();
    let to = r.end.view().participant_offset();
    r.participants = p.participants().skip(from).take(to - from).collect();
    r.context.start = if from == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    r.context.end = match ending.view().kind() {
        PreparedBreakKind::Forced => LineEnd::Forced,
        PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
        _ => unreachable!(),
    };
    r.termination = crate::PreparedTermination::Ordinary(ending);
    r.context.available_inline = Some(40.0);
    r.context.alignment = crate::Alignment::Center;
    p.select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}

fn paragraph(p: &PreparedText) -> FinalizedLine {
    centered(
        p,
        p.boundary_views().next().unwrap().handle(),
        p.opportunity_views()
            .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle(),
    )
}

fn glyphs(line: &FinalizedLine, before: usize) -> Vec<(Range, u32, f32, f64)> {
    let mut rows = vec![];
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.face().index(), 0);
        assert_eq!(run.font_size(), 10.0);
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(instance.font_size(), 10.0);
        for glyph in run.glyphs().filter(|g| g.range().start < before) {
            assert_eq!((glyph.x(), glyph.y()), (0.0, 0.0));
            rows.push((
                glyph.range(),
                glyph.id(),
                glyph.advance(),
                run.origin().x + glyph.pen(),
            ));
        }
    }
    rows
}

fn custody(p: &PreparedText, lines: Vec<FinalizedLine>, original: &[u16]) -> crate::Layout {
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: lines
            .into_iter()
            .enumerate()
            .map(|(i, line)| {
                crate::PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: 8.0 + i as f64 * 20.0,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap();
    assert_eq!(layout.source().text().code_units(), original);
    for index in 0..=original.len() {
        for affinity in [Affinity::Before, Affinity::After] {
            assert!(
                layout.cursor(Cursor::new(index, affinity)).is_ok(),
                "original caret {index}/{affinity:?}"
            );
        }
    }
    layout
}

fn controls(system: &mut System) {
    let ordinary = prepare(system, "XX", WhiteSpace::Collapse);
    let line = paragraph(&ordinary);
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        glyphs(&line, 2),
        [
            (Range::new(0, 1), 58, 10.0, 10.0),
            (Range::new(1, 2), 58, 10.0, 20.0)
        ]
    );
    custody(&ordinary, vec![line], &[88, 88]);

    let collapsing = prepare(system, "X ", WhiteSpace::Collapse);
    let line = paragraph(&collapsing);
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(glyphs(&line, 2), [(Range::new(0, 1), 58, 10.0, 15.0)]);
    let layout = custody(&collapsing, vec![line], &[88, 32]);
    for index in [1, 2] {
        assert_eq!(
            layout
                .cursor(Cursor::new(index, Affinity::Before))
                .unwrap()
                .rect()
                .origin
                .x,
            25.0
        );
    }
}

#[test]
fn ordinary_and_collapsed_space_centering_qualify_actual_font_and_custody() {
    controls(&mut system());
}

#[test]
fn trailing_nbsp_stays_in_the_center_alignment_measure() {
    let mut system = system();
    controls(&mut system);
    let p = prepare(&mut system, "X\u{a0}", WhiteSpace::Collapse);
    let line = paragraph(&p);
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        (line.feedback().hanging_start, line.feedback().hanging_end),
        (0.0, 0.0)
    );
    let actual = glyphs(&line, 2);
    assert_eq!(
        actual.iter().map(|r| (r.0, r.1, r.2)).collect::<Vec<_>>(),
        [(Range::new(0, 1), 58, 10.0), (Range::new(1, 2), 153, 10.0)]
    );
    let layout = custody(&p, vec![line], &[88, 160]);
    let x = |index| {
        layout
            .cursor(Cursor::new(index, Affinity::Before))
            .unwrap()
            .rect()
            .origin
            .x
    };
    assert_eq!((x(1) - x(0), x(2) - x(1)), (10.0, 10.0));
    // Text 4 §4.3 excludes NBSP from the other space separators. Its retained
    // 20px therefore centers like ordinary XX: (40-20)/2=10, not15.
    assert_eq!(actual.iter().map(|r| r.3).collect::<Vec<_>>(), [10.0, 20.0]);
}

#[test]
fn fitting_preserved_space_before_source_lf_counts_in_center_alignment() {
    let mut system = system();
    controls(&mut system);
    let p = prepare(&mut system, "X \nY", WhiteSpace::Preserve);
    assert!(p.control_views().next().is_none());
    let forced = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let split = forced.view().boundary();
    assert_eq!(split.view().source_after().get(), 3);
    let first = centered(&p, p.boundary_views().next().unwrap().handle(), forced);
    assert_eq!(first.feedback().advance, 20.0);
    assert_eq!(
        (first.feedback().hanging_start, first.feedback().hanging_end),
        (0.0, 0.0)
    );
    assert!(first.mapping_views().any(
        |m| m.source_range() == Range::new(2, 3) && m.kind() == PresentationMappingKind::Source
    ));
    let actual = glyphs(&first, 2);
    assert_eq!(
        actual.iter().map(|r| (r.0, r.1, r.2)).collect::<Vec<_>>(),
        [(Range::new(0, 1), 58, 10.0), (Range::new(1, 2), 3, 10.0)]
    );
    let ending = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let second = centered(&p, split, ending);
    assert_eq!(glyphs(&second, 4), [(Range::new(3, 4), 59, 10.0, 15.0)]);
    let layout = custody(&p, vec![first, second], &[88, 32, 10, 89]);
    let x = |index| {
        layout
            .cursor(Cursor::new(index, Affinity::Before))
            .unwrap()
            .rect()
            .origin
            .x
    };
    assert_eq!((x(1) - x(0), x(2) - x(1)), (10.0, 10.0));
    // Text 4 §4.3.2: actual source LF makes this sequence conditionally
    // hanging. Raw20 fits in40, so no part hangs: X10 and SPACE20.
    // No claim about CSS block end follows from a native ParagraphEnd.
    assert_eq!(actual.iter().map(|r| r.3).collect::<Vec<_>>(), [10.0, 20.0]);
}
