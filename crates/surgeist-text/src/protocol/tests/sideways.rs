//! Independent whole-paragraph clockwise composition witnesses.
//! Literal geometry uses horizontal Ahem10 metrics8/2 and the exact CW map;
//! the qualified GSUB fixture supplies real distinct retained outlines.
mod atomics;
mod break_all;
mod constraints;
mod movement;
mod source_ranges;

use super::*;
use crate::{Affinity, Cursor, Hit, Id, Movement, Point, Range, Rect, Selection, Size};

const CW: LineOrientation = LineOrientation::Sideways;
const H: LineOrientation = LineOrientation::Horizontal;

fn own_style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: crate::LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

fn input(source: Source, style: Style, rtl: bool) -> ParagraphInput {
    let len = source.text().len();
    ParagraphInput {
        source: source.clone(),
        base_style: style,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: if rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        },
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
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: if len == 0 {
            vec![crate::ParagraphAnchor {
                item: crate::ParagraphAnchorItem::EmptyOrigin(0),
                affinity: Affinity::After,
            }]
        } else {
            vec![]
        },
    }
}
fn source(value: &str) -> Source {
    Source::identified(value, Id::from_u64(1515002), 11)
}
fn system_with(bytes: Vec<u8>) -> System {
    let mut s = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    s.register_fonts(bytes).unwrap();
    s
}
fn analyze(value: ParagraphInput) -> AnalyzedSource {
    crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    )
}
fn prepare(s: &mut System, value: ParagraphInput, orientation: LineOrientation) -> PreparedText {
    s.prepare(
        &analyze(value),
        PreparationOptions::default().with_orientation(orientation),
    )
    .unwrap()
}
fn capture(
    s: &mut System,
    value: &str,
    style: Style,
    rtl: bool,
    orientation: LineOrientation,
) -> PreparedText {
    prepare(s, input(source(value), style, rtl), orientation)
}
fn selected_request(
    p: &PreparedText,
    from: PreparedBoundary,
    offer: PreparedOpportunity,
) -> LineSelectionRequest {
    let mut r = request(p);
    r.context.orientation = p.orientation();
    let rtl = p.analyzed_source().direction() == crate::Direction::RightToLeft;
    r.context.base_direction = if rtl {
        ParagraphBaseDirection::RightToLeft
    } else {
        ParagraphBaseDirection::LeftToRight
    };
    r.context.line_box_direction = if rtl {
        crate::LineBoxDirection::RightToLeft
    } else {
        crate::LineBoxDirection::LeftToRight
    };
    r.start = from;
    r.end = offer.view().boundary();
    let a = r.start.view().participant_offset();
    let b = r.end.view().participant_offset();
    r.participants = p.participants().skip(a).take(b - a).collect();
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
    r.context.start = if a == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    r.context.end = match offer.view().kind() {
        PreparedBreakKind::Forced => LineEnd::Forced,
        PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
        _ => LineEnd::Soft,
    };
    r.context.spacing = SpacingPolicy::None;
    r.context.available_inline = None;
    r.termination = crate::PreparedTermination::Ordinary(offer);
    r
}
fn start(p: &PreparedText) -> PreparedBoundary {
    p.boundary_views().next().unwrap().handle()
}
fn terminal(p: &PreparedText) -> PreparedOpportunity {
    p.opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle()
}
fn whole(p: &PreparedText) -> FinalizedLine {
    p.select(selected_request(p, start(p), terminal(p)))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn placed(p: &PreparedText, rows: Vec<(FinalizedLine, f64, f64)>) -> crate::Layout {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: rows
            .into_iter()
            .map(|(line, inline_origin, baseline)| {
                crate::PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin,
                        baseline,
                    },
                    orientation: p.orientation(),
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap()
}
fn glyphs(line: &FinalizedLine) -> Vec<(u32, f64, f32, f32, f32)> {
    line.run_views()
        .flat_map(|r| {
            r.glyphs()
                .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>()
        })
        .collect()
}
fn text_hit(layout: &crate::Layout, x: f64, y: f64) -> usize {
    let Hit::Text(p) = layout.hit(Point::new(x, y)) else {
        panic!("expected actual text hit at {x},{y}")
    };
    p.source_cursor().index()
}
fn assert_frames(layout: &crate::Layout, expected: Rect) {
    assert_eq!(layout.orientation(), CW);
    assert_eq!(layout.metrics().orientation(), CW);
    assert_eq!(layout.lines()[0].orientation(), CW);
    assert_eq!(layout.lines()[0].bounds(), expected);
    assert_eq!(layout.bounds(), expected);
    for run in layout.run_views() {
        assert_eq!(run.orientation(), CW);
    }
    for run in layout.glyph_runs() {
        assert_eq!(run.orientation(), CW);
    }
    let facts = layout.inline_metric_facts();
    for line in facts.lines() {
        assert_eq!(line.orientation(), CW);
    }
    for run in facts.runs() {
        assert_eq!(run.orientation(), CW);
    }
}

#[test]
fn automatic_and_authored_features_keep_real_glyphs_source_and_clockwise_frame() {
    let bytes = crate::shaping_test_fonts::sideways_ahem();
    let mut s = system_with(bytes.clone());
    for (settings, id) in [
        (&[][..], 274),
        (&[r#""vrtr" 0"#][..], 58),
        (&[r#""vrtr" 1"#][..], 274),
        (&[r#""vrtr" 1"#, r#""vrtr" 0"#][..], 58),
        (&[r#""vrtr" 0"#, r#""vrtr" 1"#][..], 274),
        (&[r#""vert" 1"#, r#""vrtr" 0"#][..], 82),
        (&[r#""vrt2" 1"#, r#""vrtr" 0"#][..], 277),
        (&[r#""vert" 1"#][..], 274),
        (&[r#""vrt2" 1"#][..], 274),
        (&[r#""vert" 1"#, r#""vrt2" 1"#, r#""vrtr" 1"#][..], 274),
    ] {
        let mut style = own_style();
        for setting in settings {
            style.font = style.font.feature(*setting);
        }
        let p = capture(&mut s, "X", style, false, CW);
        assert_eq!(p.orientation(), CW);
        assert_eq!(p.preparation_options().orientation(), CW);
        let line = whole(&p);
        assert_eq!(line.feedback().advance, 10.0);
        assert_eq!(glyphs(&line), [(id, 0.0, 0.0, 0.0, 10.0)]);
        let layout = placed(&p, vec![(line, 20.0, 100.0)]);
        assert_frames(&layout, Rect::new(98.0, 20.0, 10.0, 10.0));
        assert_eq!(layout.metrics().size(), Size::new(10.0, 10.0));
        assert_eq!(layout.metrics().full_width(), 10.0);
        assert_eq!(layout.source().id(), Some(Id::from_u64(1515002)));
        assert_eq!(layout.source().revision(), 11);
        assert_eq!(layout.source().text().code_units(), [88]);
        let run = layout.run_views().next().unwrap();
        assert_eq!(run.origin(), Point::new(100.0, 20.0));
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.face().index(), 0);
        assert_eq!(run.face().catalogue_identity(), p.font_catalogue_identity());
        assert_eq!(run.range(), Range::new(0, 1));
        assert_eq!(layout.glyph_runs()[0].glyphs()[0].range(), Range::new(0, 1));
        assert_eq!(
            layout
                .cursor(Cursor::new(0, Affinity::After))
                .unwrap()
                .rect(),
            Rect::new(98.0, 20.0, 10.0, 1.0)
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(1, Affinity::Before))
                .unwrap()
                .rect(),
            Rect::new(98.0, 30.0, 10.0, 1.0)
        );
        let selected = layout
            .selection(Selection::new(
                Cursor::new(0, Affinity::After),
                Cursor::new(1, Affinity::Before),
            ))
            .unwrap();
        assert_eq!(selected.rects().len(), 1);
        assert_eq!(
            selected.rects()[0].rect(),
            Rect::new(98.0, 20.0, 10.0, 10.0)
        );
        assert_eq!(text_hit(&layout, 103.0, 22.0), 0);
        assert_eq!(text_hit(&layout, 103.0, 28.0), 1);
        for position in layout.positions() {
            assert_eq!(
                layout.position_geometry(&position).unwrap().rect(),
                layout.cursor(position.source_cursor()).unwrap().rect()
            );
        }
    }
}

#[test]
fn genuine_odd_level_rotates_geometry_without_reversing_original_source_authority() {
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let p = capture(&mut s, "\u{5d0}", own_style(), true, CW);
    assert_eq!(
        p.unit_views()
            .map(|u| (u.source_range(), u.level().get()))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), 1)]
    );
    assert_eq!(p.analyzed_source().source().text().code_units(), [1488]);
    let line = whole(&p);
    assert_eq!(glyphs(&line), [(274, 0.0, 0.0, 0.0, 10.0)]);
    let layout = placed(&p, vec![(line, 20.0, 100.0)]);
    assert_frames(&layout, Rect::new(98.0, 20.0, 10.0, 10.0));
    assert_eq!(
        layout
            .cursor(Cursor::new(0, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(98.0, 30.0, 10.0, 1.0)
    );
    assert_eq!(
        layout
            .cursor(Cursor::new(1, Affinity::Before))
            .unwrap()
            .rect(),
        Rect::new(98.0, 20.0, 10.0, 1.0)
    );
    assert_eq!(text_hit(&layout, 103.0, 22.0), 1);
    assert_eq!(text_hit(&layout, 103.0, 28.0), 0);
}

#[test]
fn actual_mark_attachment_stays_canonical_while_cursor_and_selection_are_physical() {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    let mut s = system_with(bytes.clone());
    let p = capture(&mut s, "A\u{301}B", own_style(), false, CW);
    assert_eq!(
        p.analyzed_source().graphemes().collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 3)]
    );
    let line = whole(&p);
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        glyphs(&line),
        [
            (35, 0.0, 0.0, 0.0, 10.0),
            (100, 10.0, -7.0, -4.0, 0.0),
            (36, 10.0, 0.0, 0.0, 10.0)
        ]
    );
    let layout = placed(&p, vec![(line, 20.0, 100.0)]);
    let ranges = layout
        .glyph_runs()
        .iter()
        .flat_map(|r| r.glyphs().iter().map(|g| g.range()))
        .collect::<Vec<_>>();
    assert_eq!(
        ranges,
        [Range::new(0, 2), Range::new(0, 2), Range::new(2, 3)]
    );
    let run = layout.run_views().next().unwrap();
    assert_eq!(run.orientation(), CW);
    assert_eq!(run.face().bytes(), bytes);
    assert_eq!(run.origin(), Point::new(100.0, 20.0));
    // Pen10 +raw offset−7 =3, raw y−4: physical mark baseline point104,23.
    let mark = run.glyphs().nth(1).unwrap();
    assert_eq!(
        (
            run.origin().x - f64::from(mark.y()),
            run.origin().y + mark.pen() + f64::from(mark.x())
        ),
        (104.0, 23.0)
    );
    assert_eq!(
        layout
            .cursor(Cursor::new(2, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(98.0, 30.0, 10.0, 1.0)
    );
    assert_eq!(
        layout
            .selection(Selection::new(
                Cursor::new(0, Affinity::After),
                Cursor::new(2, Affinity::Before)
            ))
            .unwrap()
            .rects()[0]
            .rect(),
        Rect::new(98.0, 20.0, 10.0, 10.0)
    );
}

#[test]
fn profile_mismatch_vertical_and_foreign_receipts_refuse_without_mutating_old_output() {
    let bytes = crate::shaping_test_fonts::sideways_ahem();
    let mut s = system_with(bytes);
    let p = capture(&mut s, "X", own_style(), false, CW);
    let line = whole(&p);
    let old = placed(&p, vec![(line.clone(), 20.0, 100.0)]);
    let mut wrong = selected_request(&p, start(&p), terminal(&p));
    wrong.context.orientation = H;
    assert_eq!(
        reason(&p.select(wrong).unwrap_err()),
        &ProtocolError::MismatchedContext {
            field: ProtocolContextField::Orientation
        }
    );
    let mut vertical = selected_request(&p, start(&p), terminal(&p));
    vertical.context.orientation = LineOrientation::Vertical;
    assert_eq!(
        reason(&p.select(vertical).unwrap_err()),
        &ProtocolError::UnsupportedContext {
            field: ProtocolContextField::Orientation
        }
    );
    let analysis = analyze(input(source("X"), own_style(), false));
    assert_eq!(
        reason(
            &s.prepare(
                &analysis,
                PreparationOptions::default().with_orientation(LineOrientation::Vertical)
            )
            .unwrap_err()
        ),
        &ProtocolError::UnsupportedContext {
            field: ProtocolContextField::Orientation
        }
    );
    let error = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: H,
            atomics: vec![],
        })],
    })
    .unwrap_err();
    assert_eq!(
        reason(&error),
        &ProtocolError::MismatchedContext {
            field: ProtocolContextField::Orientation
        }
    );
    let other = capture(&mut s, "X", own_style(), false, CW);
    let error = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: other,
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: CW,
            atomics: vec![],
        })],
    })
    .unwrap_err();
    assert_eq!(
        reason(&error),
        &ProtocolError::ForeignFinalizedLine { line: 0 }
    );
    assert_eq!(old.lines()[0].bounds(), Rect::new(98.0, 20.0, 10.0, 10.0));
    assert_eq!(old.glyph_runs()[0].glyphs()[0].id(), 274);
}

#[test]
fn forced_columns_use_real_anchors_nearest_rectangles_and_line_ordinals() {
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let p = capture(&mut s, "A\nB", own_style(), false, CW);
    let forced = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let first = p
        .select(selected_request(&p, start(&p), forced.clone()))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let second = p
        .select(selected_request(&p, forced.view().boundary(), terminal(&p)))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (10.0, 10.0)
    );
    for (next, tie) in [(80.0, 93.0), (120.0, 113.0)] {
        let layout = placed(
            &p,
            vec![(first.clone(), 20.0, 100.0), (second.clone(), 20.0, next)],
        );
        assert_eq!(
            layout.lines()[0].bounds(),
            Rect::new(98.0, 20.0, 10.0, 10.0)
        );
        assert_eq!(
            layout.lines()[1].bounds(),
            Rect::new(next - 2.0, 20.0, 10.0, 10.0)
        );
        assert_eq!(layout.lines()[0].range(), Range::new(0, 2));
        assert_eq!(layout.lines()[1].range(), Range::new(2, 3));
        assert_eq!(layout.metrics().size(), Size::new(30.0, 10.0));
        assert_eq!(layout.metrics().full_width(), 30.0);
        assert_eq!(text_hit(&layout, next + 3.0, 22.0), 2);
        assert_eq!(
            text_hit(&layout, tie, 22.0),
            0,
            "equidistant rectangles retain ordinal tie"
        );
        assert_eq!(
            layout
                .move_cursor(Cursor::new(0, Affinity::After), Movement::NextLine)
                .unwrap()
                .index(),
            2
        );
        assert_eq!(
            layout
                .move_cursor(Cursor::new(2, Affinity::After), Movement::PreviousLine)
                .unwrap()
                .index(),
            0
        );
    }
}

#[test]
fn terminal_forced_receipt_and_nonformatted_completion_keep_distinct_custody() {
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let mut value = input(source("A"), own_style(), false);
    value.controls = vec![crate::ParagraphControl {
        at: 1,
        kind: crate::ParagraphControlKind::ForcedBreak,
    }];
    value.anchor_order = vec![crate::ParagraphAnchor {
        item: crate::ParagraphAnchorItem::Control(0),
        affinity: Affinity::After,
    }];
    let p = prepare(&mut s, value, CW);
    let forced = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    assert_eq!(p.control_views().count(), 1);
    let first = p
        .select(selected_request(&p, start(&p), forced.clone()))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let last = p
        .select(selected_request(&p, forced.view().boundary(), terminal(&p)))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(first.feedback().advance, 10.0);
    assert_eq!(last.feedback().advance, 0.0);
    assert_eq!(last.feedback().line_height, 10.0);
    let capability = last.nonformatted().unwrap();
    let entries = vec![
        crate::PlacementEntry::Formatted(LinePlacement {
            line: first,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: CW,
            atomics: vec![],
        }),
        crate::PlacementEntry::NonFormatted(crate::NonFormattedLinePlacement {
            line: capability,
            origin: Point::new(80.0, 20.0),
            orientation: CW,
            atomics: vec![],
        }),
    ];
    let error = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: entries[..1].to_vec(),
    })
    .unwrap_err();
    assert!(matches!(
        reason(&error),
        ProtocolError::IncompleteLineCoverage { .. }
    ));
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: entries.clone(),
    })
    .unwrap();
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.bounds(), Rect::new(98.0, 20.0, 10.0, 10.0));
    assert_eq!(layout.nonformatted_lines().count(), 1);
    assert_eq!(
        layout.nonformatted_lines().next().unwrap().origin,
        Point::new(80.0, 20.0)
    );
    assert_eq!(
        layout
            .nonformatted_lines()
            .next()
            .unwrap()
            .line
            .finalized_line(),
        &last
    );
    assert_eq!(layout.source().text().code_units(), [65]);
    let replay = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p,
        coverage: PlacementCoverage::WholeParagraph,
        entries,
    })
    .unwrap();
    assert_eq!(replay.bounds(), layout.bounds());
    assert_eq!(
        replay
            .positions()
            .map(|p| p.source_cursor())
            .collect::<Vec<_>>(),
        layout
            .positions()
            .map(|p| p.source_cursor())
            .collect::<Vec<_>>()
    );
}

#[test]
fn spacing_and_css_tabs_remain_logical_while_physical_extent_turns_once() {
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let p = capture(&mut s, "A A", own_style(), false, CW);
    let mut r = selected_request(&p, start(&p), terminal(&p));
    r.context.spacing = SpacingPolicy::Spaces;
    let selected = p.select(r).unwrap();
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let line = selected
        .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
            slot: slots[0].clone(),
            amount: 20.0,
        }]))
        .unwrap();
    assert_eq!(line.feedback().advance, 50.0);
    assert_eq!(
        glyphs(&line),
        [
            (35, 0.0, 0.0, 0.0, 10.0),
            (3, 10.0, 0.0, 0.0, 10.0),
            (35, 40.0, 0.0, 0.0, 10.0)
        ]
    );
    let layout = placed(&p, vec![(line, 20.0, 100.0)]);
    assert_frames(&layout, Rect::new(98.0, 20.0, 10.0, 50.0));
    assert_eq!(
        layout
            .cursor(Cursor::new(2, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(98.0, 60.0, 10.0, 1.0)
    );
    let style = Style {
        tab_size: crate::TabSize::length(40.0).unwrap(),
        ..own_style()
    };
    let mut value = input(source("A\tB"), style.clone(), false);
    value.tab_reference = Some(
        crate::ParagraphTabReference::new(style, ParagraphBaseDirection::LeftToRight).unwrap(),
    );
    let p = s
        .prepare(
            &analyze(value),
            PreparationOptions::try_new(1.0, crate::TabPreparation::CssStops)
                .unwrap()
                .with_orientation(CW),
        )
        .unwrap();
    let mut r = selected_request(&p, start(&p), terminal(&p));
    r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
    let line = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 50.0);
    assert_eq!(
        glyphs(&line),
        [(35, 0.0, 0.0, 0.0, 10.0), (36, 40.0, 0.0, 0.0, 10.0)]
    );
    let layout = placed(&p, vec![(line, 20.0, 100.0)]);
    assert_frames(&layout, Rect::new(98.0, 20.0, 10.0, 50.0));
    assert_eq!(layout.source().text().code_units(), [65, 9, 66]);
}

#[test]
fn before_after_and_both_indications_keep_one_source_choice_and_rotated_receivers() {
    use crate::{HyphenationPlacement as P, NativeHyphenationConvention, Text};
    let bytes = crate::shaping_test_fonts::sideways_ahem();
    let mut s = system_with(bytes.clone());
    for (mode, advances) in [
        (P::LogicalBefore, (30.0, 30.0)),
        (P::LogicalAfter, (20.0, 40.0)),
        (P::BothSides, (30.0, 40.0)),
    ] {
        let style = Style {
            locale: Some("und-Latn".into()),
            hyphens: crate::Hyphens::Manual,
            native_hyphenation_convention: Some(
                NativeHyphenationConvention::language_script(
                    "und-Latn",
                    "Latn",
                    Text::from("X"),
                    mode,
                )
                .unwrap(),
            ),
            ..own_style()
        };
        let p = capture(&mut s, "AB\u{ad}CCC", style, false, CW);
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            [65, 66, 173, 67, 67, 67]
        );
        let offer = p
            .hyphenation_cut_views()
            .find(|c| c.boundary().view().source_before().get() == 3)
            .unwrap()
            .opportunity()
            .unwrap();
        let choice = offer.view().alternatives().next().unwrap().handle();
        assert_eq!(choice.view().text().code_units(), [88]);
        assert_eq!(choice.view().source_range(), Some(Range::new(2, 3)));
        let mut r = selected_request(&p, start(&p), offer.clone());
        r.alternative = Some(choice.clone());
        let first = p
            .select(r)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        let mut r = selected_request(&p, offer.view().boundary(), terminal(&p));
        r.incoming = first.hyphenation_continuation();
        let second = p
            .select(r)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(
            (first.feedback().advance, second.feedback().advance),
            advances
        );
        assert_eq!(
            glyphs(&first).iter().filter(|g| g.0 == 274).count(),
            usize::from(mode != P::LogicalAfter)
        );
        assert_eq!(
            glyphs(&second).iter().filter(|g| g.0 == 274).count(),
            usize::from(mode != P::LogicalBefore)
        );
        assert_eq!(first.selected_line().request().alternative, Some(choice));
        assert!(second.hyphenation_continuation().is_none());
        for line in [&first, &second] {
            for run in line.run_views() {
                assert_eq!(run.face().bytes(), bytes);
            }
        }
        let layout = placed(&p, vec![(first, 20.0, 100.0), (second, 20.0, 80.0)]);
        assert_eq!(
            layout.lines()[0].bounds(),
            Rect::new(98.0, 20.0, 10.0, advances.0)
        );
        assert_eq!(
            layout.lines()[1].bounds(),
            Rect::new(78.0, 20.0, 10.0, advances.1)
        );
        assert_eq!(
            layout.source().text().code_units(),
            [65, 66, 173, 67, 67, 67]
        );
    }
}

#[test]
fn profile_and_feature_switches_reuse_real_system_without_changing_retained_packets() {
    let bytes = crate::shaping_test_fonts::sideways_ahem();
    let mut s = system_with(bytes.clone());
    let a = analyze(input(source("X"), own_style(), false));
    let h = s.prepare(&a, PreparationOptions::default()).unwrap();
    let hline = whole(&h);
    assert_eq!(glyphs(&hline)[0].0, 58);
    let p = s
        .prepare(&a, PreparationOptions::default().with_orientation(CW))
        .unwrap();
    let line = whole(&p);
    let old = placed(&p, vec![(line.clone(), 20.0, 100.0)]);
    let before = s.stats();
    let again = whole(&p);
    assert_eq!(s.stats(), before, "finalization cannot requery resources");
    assert_eq!(glyphs(&again), glyphs(&line));
    for setting in [r#""vrtr" 0"#, r#""vrtr" 1"#, r#""vrtr" 0"#] {
        let style = Style {
            font: Font::new().family("Ahem").feature(setting),
            ..own_style()
        };
        let current = capture(&mut s, "X", style.clone(), false, CW);
        let currentline = whole(&current);
        let mut fresh = system_with(bytes.clone());
        let freshp = capture(&mut fresh, "X", style, false, CW);
        assert_eq!(glyphs(&currentline), glyphs(&whole(&freshp)));
        assert_eq!(currentline.feedback().advance, 10.0);
    }
    let horizontal = s.prepare(&a, PreparationOptions::default()).unwrap();
    assert_eq!(glyphs(&whole(&horizontal)), glyphs(&hline));
    assert_eq!(glyphs(&line)[0].0, 274);
    s.refresh_fonts().unwrap();
    drop(s);
    assert_eq!(old.run_views().next().unwrap().face().bytes(), bytes);
    assert_eq!(old.lines()[0].bounds(), Rect::new(98.0, 20.0, 10.0, 10.0));
    assert_eq!(placed(&p, vec![(line, 20.0, 100.0)]).bounds(), old.bounds());
}

#[test]
fn real_in_flow_and_out_of_flow_atomics_prepare_with_retained_sideways_occurrences() {
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    for kind in [
        crate::InlineBoxKind::InFlow,
        crate::InlineBoxKind::OutOfFlow,
    ] {
        let mut src = source("AB");
        src.inline_box(crate::InlineBox::new(
            Id::from_u64(1515003),
            kind,
            1,
            Size::new(10.0, 10.0),
        ));
        let mut value = input(src, own_style(), false);
        value.anchor_order = vec![crate::ParagraphAnchor {
            item: crate::ParagraphAnchorItem::Atomic(0),
            affinity: Affinity::After,
        }];
        value.boundary_policies = [
            crate::ParagraphBoundaryDefinition::AtomicBefore(0),
            crate::ParagraphBoundaryDefinition::AtomicAfter(0),
        ]
        .into_iter()
        .map(|definition| crate::ParagraphBoundaryPolicy {
            definition,
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        })
        .collect();
        let a = analyze(value);
        let h = s.prepare(&a, PreparationOptions::default()).unwrap();
        assert_eq!(h.atomic_views().count(), 1);
        assert_eq!(
            h.atomic_views().next().unwrap().original_box().id(),
            Id::from_u64(1515003)
        );
        let sideways = s
            .prepare(&a, PreparationOptions::default().with_orientation(CW))
            .expect("actual native clockwise atomic occurrence must prepare");
        assert_eq!(sideways.orientation(), CW);
        assert_eq!(sideways.analyzed_source().identity(), a.identity());
        assert_eq!(sideways.atomic_views().count(), 1);
        let actual = sideways.atomic_views().next().unwrap();
        assert_eq!(actual.original_box().id(), Id::from_u64(1515003));
        assert_eq!(actual.original_box().kind(), kind);
        assert_eq!(actual.source_position().get(), 1);
        assert_eq!(h.analyzed_source().source().text().code_units(), [65, 66]);
    }
}

#[test]
fn real_post_phase_one_transform_and_removed_source_keep_original_custody() {
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let upper = Style {
        text_transform: crate::TextTransform::Set(
            crate::TextTransformSet::none().with_case(crate::CaseTransform::Uppercase),
        ),
        ..own_style()
    };
    let p = capture(&mut s, "x", upper, false, CW);
    assert_eq!(p.analyzed_source().source().text().code_units(), [120]);
    // Phase I remains original lowercase here. Casing has a separate actual
    // transformed backing before analysis/shaping (the owning contract keeps
    // ProcessedSource readbacks in the Phase-I coordinate domain).
    assert_eq!(
        p.analyzed_source()
            .processed_source()
            .processed_text()
            .code_units(),
        [120]
    );
    assert_eq!(
        p.analyzed_source()
            .inner
            .projection
            .text
            .encode_utf16()
            .collect::<Vec<_>>(),
        [88]
    );
    let line = whole(&p);
    assert_eq!(glyphs(&line), [(274, 0.0, 0.0, 0.0, 10.0)]);
    let layout = placed(&p, vec![(line, 20.0, 100.0)]);
    assert_eq!(layout.source().text().code_units(), [120]);
    assert_eq!(layout.glyph_runs()[0].glyphs()[0].range(), Range::new(0, 1));
    let collapsed = Style {
        white_space: WhiteSpace::Collapse,
        ..own_style()
    };
    let p = capture(&mut s, "A  B", collapsed, false, CW);
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [65, 32, 32, 66]
    );
    assert_eq!(
        p.analyzed_source()
            .processed_source()
            .processed_text()
            .code_units(),
        [65, 32, 66]
    );
    let line = whole(&p);
    assert_eq!(line.feedback().advance, 30.0);
    let layout = placed(&p, vec![(line, 20.0, 100.0)]);
    assert_eq!(layout.source().text().code_units(), [65, 32, 32, 66]);
    assert_frames(&layout, Rect::new(98.0, 20.0, 10.0, 30.0));
    let removed = Style {
        white_space: WhiteSpace::Discard,
        ..own_style()
    };
    let p = capture(&mut s, "  ", removed, false, CW);
    let line = whole(&p);
    assert_eq!(p.analyzed_source().source().text().code_units(), [32, 32]);
    assert!(
        p.analyzed_source()
            .processed_source()
            .processed_text()
            .is_empty()
    );
    assert_eq!(line.feedback().advance, 0.0);
    assert_eq!(line.feedback().line_height, 10.0);
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::NonFormatted(
            crate::NonFormattedLinePlacement {
                line: line.nonformatted().unwrap(),
                origin: Point::new(7.0, 11.0),
                orientation: CW,
                atomics: vec![],
            },
        )],
    })
    .unwrap();
    assert!(layout.lines().is_empty());
    assert_eq!(layout.positions().count(), 0);
    assert_eq!(layout.metrics().size(), Size::new(0.0, 0.0));
    assert_eq!(layout.hit(Point::new(7.0, 11.0)), Hit::None);
    assert_eq!(layout.source().text().code_units(), [32, 32]);
    assert_eq!(layout.nonformatted_lines().count(), 1);
}

#[test]
fn authentic_wbr_control_is_an_optional_source_free_cut_with_clockwise_receipts() {
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let mut value = input(source("AB"), own_style(), false);
    value.controls = vec![crate::ParagraphControl {
        at: 1,
        kind: crate::ParagraphControlKind::Wbr {
            style: Box::new(own_style()),
        },
    }];
    value.anchor_order = vec![crate::ParagraphAnchor {
        item: crate::ParagraphAnchorItem::Control(0),
        affinity: Affinity::After,
    }];
    let p = prepare(&mut s, value, CW);
    let offer = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Allowed)
        .unwrap()
        .handle();
    let control = p.control_views().next().unwrap();
    assert!(matches!(
        control.kind(),
        crate::ParagraphControlKind::Wbr { .. }
    ));
    let first = p
        .select(selected_request(&p, start(&p), offer.clone()))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let second = p
        .select(selected_request(&p, offer.view().boundary(), terminal(&p)))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (10.0, 10.0)
    );
    assert_eq!(first.selected_line().request().context.end, LineEnd::Soft);
    let layout = placed(&p, vec![(first, 20.0, 100.0), (second, 20.0, 80.0)]);
    assert_eq!(layout.source().text().code_units(), [65, 66]);
    assert_eq!(layout.lines().len(), 2);
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(98.0, 20.0, 10.0, 10.0)
    );
    assert_eq!(
        layout.lines()[1].bounds(),
        Rect::new(78.0, 20.0, 10.0, 10.0)
    );
}

#[test]
fn owned_vertical_scalar_run_is_total_without_authorizing_vertical_placement() {
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let p = capture(&mut s, "X", own_style(), false, H);
    let layout = placed(&p, vec![(whole(&p), 20.0, 100.0)]);
    let actual = layout.glyph_runs().remove(0);
    // A public scalar record has an axis interpretation, not a newly issued
    // placed-line receipt. Retain real glyph/font data, never forge a Layout.
    let record = crate::Run::new(
        0,
        actual.range(),
        actual.font().clone(),
        actual.style().clone(),
        actual.brush(),
        crate::RunMetrics::new(10.0, 100.0, 20.0, 10.0, LineOrientation::Vertical),
        crate::GlyphBasis::Unrotated,
        actual.glyphs().to_vec(),
    );
    assert_eq!(record.orientation(), LineOrientation::Vertical);
    assert_eq!(record.origin(), Point::new(100.0, 20.0));
    assert_eq!(record.glyphs()[0].id(), 58);
    let mut r = selected_request(&p, start(&p), terminal(&p));
    r.context.orientation = LineOrientation::Vertical;
    assert_eq!(
        reason(&p.select(r).unwrap_err()),
        &ProtocolError::UnsupportedContext {
            field: ProtocolContextField::Orientation
        }
    );
    assert_eq!(layout.run_views().next().unwrap().orientation(), H);
}

#[test]
fn explicit_decoration_offset_is_already_physical_after_clockwise_placement() {
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let style = Style {
        underline: crate::Decoration::solid()
            .with_offset(crate::DecorationOffset::try_absolute(-2.0).unwrap())
            .with_thickness(crate::DecorationThickness::try_absolute(1.0).unwrap()),
        ..own_style()
    };
    let p = capture(&mut s, "X", style, false, CW);
    let layout = placed(&p, vec![(whole(&p), 20.0, 100.0)]);
    // Canonical underline [0,2,10,1] relative to the baseline rotates left.
    let decorations = layout.decoration_views().collect::<Vec<_>>();
    assert_eq!(decorations.len(), 1);
    assert_eq!(decorations[0].rect(), Rect::new(97.0, 20.0, 1.0, 10.0));
}

#[cfg(feature = "text-accessibility")]
#[test]
fn accessibility_retains_physical_cw_bounds_width_and_actual_bidi_direction() {
    for (text, rtl, direction) in [
        ("X", false, accesskit::TextDirection::TopToBottom),
        ("\u{5d0}", true, accesskit::TextDirection::BottomToTop),
    ] {
        let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
        let p = capture(&mut s, text, own_style(), rtl, CW);
        let layout = placed(&p, vec![(whole(&p), 20.0, 100.0)]);
        let mut state = crate::Accessibility::default();
        let parent = accesskit::NodeId(1515004);
        let update = layout
            .accessibility_update(&mut state, parent, parent, Point::default())
            .unwrap();
        let nodes = update
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == accesskit::Role::TextRun)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 1);
        let node = &nodes[0].1;
        assert_eq!(node.text_direction(), Some(direction));
        assert_eq!(node.character_widths(), Some(&[10.0][..]));
        let bounds = node.bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
            (98.0, 20.0, 108.0, 30.0)
        );
        let cursor = layout
            .access_position(&state, Cursor::new(0, Affinity::After))
            .unwrap()
            .unwrap();
        assert_eq!(cursor.character_index, 0);
    }
}
