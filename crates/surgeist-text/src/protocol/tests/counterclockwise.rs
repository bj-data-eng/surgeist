//! Independent functional qualification of native counterclockwise composition.
//! Ahem10 supplies raw H8/2 and10px advances. Physical expectations use the
//! declared CCW map (X+v-B,Y-u), not output sampled from the implementation.
//! This new profile has no historical callable preimplementation RED.
mod allocations;
mod source_custody;

use super::*;
use crate::{Affinity, Cursor, GlyphBasis, Hit, Id, Movement, Point, Range, Rect, Selection, Size};
use skrifa::{GlyphId, MetadataProvider, raw::TableProvider};

const CCW: LineOrientation = LineOrientation::SidewaysCounterclockwise;
const CW: LineOrientation = LineOrientation::SidewaysClockwise;
const H: LineOrientation = LineOrientation::Horizontal;
const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn own_style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: crate::LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}
fn source(value: &str) -> Source {
    Source::identified(value, Id::from_u64(1_579_001), 19)
}
fn paragraph(source: Source, style: Style, rtl: bool) -> ParagraphInput {
    let len = source.text().len();
    ParagraphInput {
        source: source.clone(),
        base_style: style,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: if rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        },
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
fn system_with(bytes: Vec<u8>) -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    system
}
fn analyze(value: ParagraphInput) -> AnalyzedSource {
    crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    )
}
fn prepare(
    system: &mut System,
    value: ParagraphInput,
    orientation: LineOrientation,
) -> PreparedText {
    system
        .prepare(
            &analyze(value),
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap()
}
fn capture(
    system: &mut System,
    raw: &str,
    style: Style,
    rtl: bool,
    orientation: LineOrientation,
) -> PreparedText {
    prepare(system, paragraph(source(raw), style, rtl), orientation)
}
fn start(p: &PreparedText) -> PreparedBoundary {
    p.boundary_views().next().unwrap().handle()
}
fn terminal(p: &PreparedText) -> PreparedOpportunity {
    p.opportunity_views()
        .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle()
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
            PreparedParticipant::Control(control) => Some(ControlAllocation {
                occurrence: control.clone(),
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
    r.context.alignment = crate::Alignment::Start;
    r.termination = crate::PreparedTermination::Ordinary(offer);
    r
}
fn whole(p: &PreparedText) -> FinalizedLine {
    p.select(selected_request(p, start(p), terminal(p)))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}
fn entries(p: &PreparedText, rows: Vec<(FinalizedLine, f64, f64)>) -> Vec<crate::PlacementEntry> {
    rows.into_iter()
        .map(|(line, x, y)| {
            crate::PlacementEntry::Formatted(LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: y,
                    baseline: x,
                },
                orientation: p.orientation(),
                atomics: vec![],
            })
        })
        .collect()
}
fn placed(p: &PreparedText, rows: Vec<(FinalizedLine, f64, f64)>) -> crate::Layout {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: entries(p, rows),
    })
    .unwrap()
}
fn glyphs(line: &FinalizedLine) -> Vec<(u32, f64, f32, f32, f32)> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>()
        })
        .collect()
}
fn text_hit(layout: &crate::Layout, point: Point) -> usize {
    let Hit::Text(position) = layout.hit(point) else {
        panic!("expected real text hit at {point:?}")
    };
    position.source_cursor().index()
}
fn caret(layout: &crate::Layout, index: usize, affinity: Affinity) -> Rect {
    layout.cursor(Cursor::new(index, affinity)).unwrap().rect()
}

#[test]
fn counterclockwise_ltr_and_odd_rtl_keep_raw_horizontal_metrics_and_directed_carets() {
    let bytes = crate::shaping_test_fonts::sideways_ahem();
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(
        (
            face.hhea().unwrap().ascender().to_i16(),
            face.hhea().unwrap().descender().to_i16()
        ),
        (800, -200)
    );
    for tag in [b"vhea", b"vmtx", b"VORG"] {
        assert!(face.table_data(skrifa::Tag::new(tag)).is_none());
    }
    assert_eq!(face.charmap().map('\u{5d0}'), Some(GlyphId::new(58)));
    let mut system = system_with(bytes.clone());
    for (raw, rtl, ids) in [("AB", false, [35, 36]), ("\u{5d0}\u{5d0}", true, [58, 58])] {
        let p = capture(&mut system, raw, own_style(), rtl, CCW);
        assert_eq!(p.orientation(), CCW);
        assert_eq!(p.preparation_options().orientation(), CCW);
        assert_eq!(
            p.unit_views().map(|v| v.level().get()).collect::<Vec<_>>(),
            vec![u8::from(rtl); 2]
        );
        let line = whole(&p);
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (20.0, 8.0, 10.0)
        );
        assert_eq!(
            glyphs(&line),
            [
                (ids[0], 0.0, 0.0, 0.0, 10.0),
                (ids[1], 10.0, 0.0, 0.0, 10.0)
            ]
        );
        let layout = placed(&p, vec![(line, -7.25, 13.5)]);
        assert_eq!(layout.orientation(), CCW);
        assert_eq!(layout.bounds(), Rect::new(-15.25, -6.5, 10.0, 20.0));
        assert_eq!(layout.lines()[0].bounds(), layout.bounds());
        assert_eq!(layout.lines()[0].baseline(), -7.25);
        assert_eq!(
            layout.source().text().code_units(),
            raw.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            (layout.source().id(), layout.source().revision()),
            (Some(Id::from_u64(1_579_001)), 19)
        );
        for run in layout.run_views() {
            assert_eq!(run.glyph_basis(), GlyphBasis::Counterclockwise);
            assert_eq!(run.orientation(), CCW);
            assert_eq!(run.origin(), Point::new(-7.25, 13.5));
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.face().index(), 0);
        }
        for run in layout.glyph_runs() {
            assert_eq!(run.glyph_basis(), GlyphBasis::Counterclockwise);
        }
        let (first, last) = if rtl { (-7.5, 12.5) } else { (12.5, -7.5) };
        assert_eq!(
            caret(&layout, 0, Affinity::After),
            Rect::new(-15.25, first, 10.0, 1.0)
        );
        assert_eq!(
            caret(&layout, 2, Affinity::Before),
            Rect::new(-15.25, last, 10.0, 1.0)
        );
        let selection = layout
            .selection(Selection::new(
                Cursor::new(0, Affinity::After),
                Cursor::new(2, Affinity::Before),
            ))
            .unwrap();
        assert_eq!(selection.rects()[0].rect(), layout.bounds());
        assert_eq!(
            text_hit(&layout, Point::new(-10.25, 11.5)),
            if rtl { 2 } else { 0 }
        );
        assert_eq!(
            text_hit(&layout, Point::new(-10.25, -4.5)),
            if rtl { 0 } else { 2 }
        );
    }
}

#[test]
fn literal_vertical_feature_chain_is_not_automatically_enabled_for_counterclockwise() {
    let bytes = crate::shaping_test_fonts::sideways_ahem();
    let mut system = system_with(bytes.clone());
    for (settings, expected) in [
        (&[][..], 58),
        (&[r#""vrtr" 0"#][..], 58),
        (&[r#""vrtr" 1"#][..], 274),
        (&[r#""vert" 1"#][..], 82),
        (&[r#""vrt2" 1"#][..], 277),
        (&[r#""vrtr" 1"#, r#""vrtr" 0"#][..], 58),
        (&[r#""vrtr" 0"#, r#""vrtr" 1"#][..], 274),
        (&[r#""vert" 1"#, r#""vrt2" 1"#, r#""vrtr" 1"#][..], 274),
    ] {
        let mut style = own_style();
        for setting in settings {
            style.font = style.font.feature(*setting);
        }
        let p = capture(&mut system, "X", style, false, CCW);
        assert_eq!(glyphs(&whole(&p)), [(expected, 0.0, 0.0, 0.0, 10.0)]);
    }
    // Same analysis and same live catalogue exercise actual profile cache keys.
    let analyzed = analyze(paragraph(source("X"), own_style(), false));
    for (orientation, expected, basis) in [
        (CW, 274, GlyphBasis::Clockwise),
        (CCW, 58, GlyphBasis::Counterclockwise),
        (H, 58, GlyphBasis::Unrotated),
        (CCW, 58, GlyphBasis::Counterclockwise),
    ] {
        let p = system
            .prepare(
                &analyzed,
                PreparationOptions::default().with_orientation(orientation),
            )
            .unwrap();
        let line = whole(&p);
        assert_eq!(glyphs(&line)[0].0, expected);
        let layout = placed(&p, vec![(line, 10.0, 20.0)]);
        assert_eq!(layout.run_views().next().unwrap().glyph_basis(), basis);
    }
}

#[test]
fn asymmetrical_mark_offsets_and_real_ligature_carets_keep_the_horizontal_basis() {
    let bytes = crate::shaping_test_fonts::mark_ahem();
    let mut system = system_with(bytes.clone());
    let p = capture(&mut system, "A\u{301}B", own_style(), false, CCW);
    let line = whole(&p);
    // Actual GPOS anchors(400,600)-(100,200) give3/-4px at UPEM1000.
    // The provider mark pen10 plus xOffset-7 remains3, not a rewritten pen.
    assert_eq!(
        glyphs(&line),
        [
            (35, 0.0, 0.0, 0.0, 10.0),
            (100, 10.0, -7.0, -4.0, 0.0),
            (36, 10.0, 0.0, 0.0, 10.0)
        ]
    );
    let layout = placed(&p, vec![(line, -7.25, 13.5)]);
    let run = layout.run_views().next().unwrap();
    let mark = run.glyphs().nth(1).unwrap();
    assert_eq!(run.glyph_basis(), GlyphBasis::Counterclockwise);
    assert_eq!(
        (
            run.origin().x + f64::from(mark.y()),
            run.origin().y - mark.pen() - f64::from(mark.x())
        ),
        (-11.25, 10.5)
    );
    assert_eq!(layout.glyph_runs()[0].glyphs()[1].range(), Range::new(0, 2));
    assert_eq!(
        caret(&layout, 2, Affinity::After),
        Rect::new(-15.25, 2.5, 10.0, 1.0)
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
        Rect::new(-15.25, 3.5, 10.0, 10.0)
    );
    let mut system = system_with(crate::shaping_test_fonts::ligature_ahem());
    let p = capture(&mut system, "AA", own_style(), false, CCW);
    let line = whole(&p);
    assert_eq!(glyphs(&line), [(58, 0.0, 0.0, 0.0, 10.0)]);
    let layout = placed(&p, vec![(line, -7.25, 13.5)]);
    assert_eq!(layout.glyph_runs()[0].glyphs()[0].range(), Range::new(0, 2));
    // Ahem has no GDEF and this actual shared provider cluster lends only
    // its two authentic outer caret stops. Affinity projects source1 After
    // to logical end2 before bidi placement; metric shares are not carets.
    assert_eq!(layout.positions().count(), 2);
    assert_eq!(
        caret(&layout, 1, Affinity::After),
        Rect::new(-15.25, 2.5, 10.0, 1.0)
    );
    assert_eq!(
        caret(&layout, 1, Affinity::Before),
        Rect::new(-15.25, 12.5, 10.0, 1.0)
    );
    let origin = p.origin_views().next().unwrap().handle();
    let original = layout
        .source_range_fragments(&origin, Range::new(1, 2))
        .unwrap();
    assert_eq!(original.local_range(), Range::new(1, 2));
    let unit = original.units().next().unwrap();
    assert_eq!(
        unit.selected_intersections()
            .map(|piece| piece.local_range())
            .collect::<Vec<_>>(),
        [Range::new(1, 2)]
    );
    assert_eq!(layout.source().text().code_units(), [65, 65]);
    assert_eq!(
        layout
            .selection(Selection::new(
                Cursor::new(1, Affinity::After),
                Cursor::new(2, Affinity::Before)
            ))
            .unwrap()
            .rects()[0]
            .rect(),
        Rect::new(-15.25, 3.5, 10.0, 10.0)
    );
}

#[test]
fn source_range_piece_uses_decreasing_physical_edges_and_retained_instance() {
    let mut system = system_with(AHEM.to_vec());
    let p = capture(&mut system, "A", own_style(), false, CCW);
    let origin = p.origin_views().next().unwrap().handle();
    let line = whole(&p);
    let layout = placed(&p, vec![(line.clone(), -7.25, 13.5)]);
    let query = layout
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    let unit = query.units().next().unwrap();
    assert_eq!(unit.finalized_line(), line);
    assert_eq!(
        unit.selected_intersections()
            .map(|v| (v.origin(), v.local_range()))
            .collect::<Vec<_>>(),
        [(origin, Range::new(0, 1))]
    );
    let piece = unit.pieces().next().unwrap();
    assert_eq!(piece.rect(), Rect::new(-15.25, 3.5, 10.0, 10.0));
    assert_eq!(piece.advance_edges(), (13.5, 3.5));
    assert_eq!(piece.baseline(), -7.25);
    assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
    assert_eq!(piece.effective_instance().face().bytes(), AHEM);
}

#[test]
fn adjacent_movement_preserves_true_u_instead_of_inverse_caret_hull_top_left() {
    let mut system = system_with(AHEM.to_vec());
    let p = capture(
        &mut system,
        "A\nAB",
        Style {
            letter_spacing: -9.0,
            ..own_style()
        },
        false,
        CCW,
    );
    let forced = p
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let mut a = selected_request(&p, start(&p), forced.clone());
    a.context.spacing = SpacingPolicy::InterCharacter;
    let first = p
        .select(a)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let mut b = selected_request(&p, forced.view().boundary(), terminal(&p));
    b.context.spacing = SpacingPolicy::InterCharacter;
    let second = p
        .select(b)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (10.0, 11.0)
    );
    assert_eq!(
        glyphs(&second),
        [(35, 0.0, 0.0, 0.0, 10.0), (36, 1.0, 0.0, 0.0, 10.0)]
    );
    let layout = placed(&p, vec![(first, 20.25, 50.5), (second, -30.5, 80.25)]);
    let from = layout
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(0, Affinity::After))
        .unwrap();
    assert_eq!(
        layout.position_geometry(&from).unwrap().rect(),
        Rect::new(12.25, 49.5, 10.0, 1.0)
    );
    let next = layout.move_position(&from, Movement::NextLine).unwrap();
    assert_eq!(next.source_cursor(), Cursor::new(2, Affinity::After));
    assert_eq!(
        layout.position_geometry(&next).unwrap().rect(),
        Rect::new(-38.5, 79.25, 10.0, 1.0)
    );
    let cluster = layout.move_position(&next, Movement::NextCluster).unwrap();
    assert_eq!(cluster.source_cursor().index(), 3);
    assert_eq!(
        layout.position_geometry(&cluster).unwrap().rect(),
        Rect::new(-38.5, 78.25, 10.0, 1.0)
    );
    assert_eq!(
        layout
            .move_cursor(Cursor::new(0, Affinity::After), Movement::NextLine)
            .unwrap(),
        Cursor::new(2, Affinity::After)
    );
}

#[test]
fn signed_underline_offsets_are_rotated_once_into_counterclockwise_physical_bounds() {
    let mut system = system_with(AHEM.to_vec());
    for (offset, expected_x) in [(-2.0, 102.0), (2.0, 98.0)] {
        let p = capture(
            &mut system,
            "A",
            Style {
                underline: crate::Decoration::solid()
                    .with_offset(crate::DecorationOffset::try_absolute(offset).unwrap())
                    .with_thickness(crate::DecorationThickness::try_absolute(1.0).unwrap()),
                ..own_style()
            },
            false,
            CCW,
        );
        let layout = placed(&p, vec![(whole(&p), 100.0, 20.0)]);
        let values = layout.decoration_views().collect::<Vec<_>>();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].rect(), Rect::new(expected_x, 10.0, 1.0, 10.0));
    }
}

#[cfg(feature = "text-accessibility")]
#[test]
fn accesskit_uses_actual_ccw_direction_bounds_and_positive_inline_character_extent() {
    for (raw, rtl, direction) in [
        ("X", false, accesskit::TextDirection::BottomToTop),
        ("\u{5d0}", true, accesskit::TextDirection::TopToBottom),
    ] {
        let mut system = system_with(crate::shaping_test_fonts::sideways_ahem());
        let p = capture(&mut system, raw, own_style(), rtl, CCW);
        let layout = placed(&p, vec![(whole(&p), -7.25, 13.5)]);
        let mut accessibility = crate::Accessibility::default();
        let parent = accesskit::NodeId(1_579_002);
        let update = layout
            .accessibility_update(&mut accessibility, parent, parent, Point::new(3.0, 7.0))
            .unwrap();
        let nodes = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == accesskit::Role::TextRun)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].1.text_direction(), Some(direction));
        assert_eq!(nodes[0].1.character_widths(), Some(&[10.0][..]));
        let bounds = nodes[0].1.bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
            (-12.25, 10.5, -2.25, 20.5)
        );
        assert_eq!(
            layout
                .access_position(&accessibility, Cursor::new(0, Affinity::After))
                .unwrap()
                .unwrap()
                .character_index,
            0
        );
    }
}

#[test]
fn named_alphabetic_reference_is_horizontal_while_central_and_upright_profiles_refuse() {
    let mut system = system_with(AHEM.to_vec());
    let analysis = analyze(paragraph(source("A"), own_style(), false));
    let p = system
        .prepare(
            &analysis,
            PreparationOptions::default()
                .with_orientation(CCW)
                .with_baseline_reference(crate::FontBaseline::Alphabetic),
        )
        .unwrap();
    let layout = placed(&p, vec![(whole(&p), -7.25, 13.5)]);
    assert_eq!(
        layout.run_views().next().unwrap().origin(),
        Point::new(-7.25, 13.5)
    );
    assert_eq!(layout.bounds(), Rect::new(-15.25, 3.5, 10.0, 10.0));
    let error = system
        .prepare(
            &analysis,
            PreparationOptions::default()
                .with_orientation(CCW)
                .with_baseline_reference(crate::FontBaseline::Central),
        )
        .unwrap_err();
    assert_eq!(
        reason(&error),
        &ProtocolError::UnsupportedContext {
            field: ProtocolContextField::BaselineReference
        }
    );
    for typography in [
        crate::AnalysisTypography::Mixed,
        crate::AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::LeftToRight,
        },
    ] {
        let analysis =
            AnalyzedSource::from_paragraph(paragraph(source("A"), own_style(), false), typography)
                .unwrap();
        let error = system
            .prepare(
                &analysis,
                PreparationOptions::default().with_orientation(CCW),
            )
            .unwrap_err();
        assert_eq!(
            reason(&error),
            &ProtocolError::MismatchedContext {
                field: ProtocolContextField::Orientation
            }
        );
    }
}

#[test]
fn real_odd_level_ligature_keeps_source_carets_in_reverse_logical_order() {
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(&[35, 35], &[(0x5d0, 35)]);
    let mut system = system_with(bytes);
    let p = capture(&mut system, "\u{5d0}\u{5d0}", own_style(), true, CCW);
    assert!(p.unit_views().all(|u| u.level().get() == 1));
    assert_eq!(
        p.analyzed_source().graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    let line = whole(&p);
    assert_eq!(glyphs(&line), [(58, 0.0, 0.0, 0.0, 10.0)]);
    let layout = placed(&p, vec![(line, -7.25, 13.5)]);
    assert_eq!(layout.glyph_runs()[0].glyphs()[0].range(), Range::new(0, 2));
    assert_eq!(
        caret(&layout, 0, Affinity::After),
        Rect::new(-15.25, 2.5, 10.0, 1.0)
    );
    assert_eq!(
        caret(&layout, 1, Affinity::After),
        Rect::new(-15.25, 12.5, 10.0, 1.0)
    );
    assert_eq!(
        caret(&layout, 2, Affinity::Before),
        Rect::new(-15.25, 12.5, 10.0, 1.0)
    );
    // The same logical After->end policy is applied BEFORE odd-level bidi:
    // end2 lies at localu0, while Before->start0 lies at localu10.
    assert_eq!(layout.positions().count(), 2);
    assert_eq!(
        caret(&layout, 1, Affinity::Before),
        Rect::new(-15.25, 2.5, 10.0, 1.0)
    );
    let origin = p.origin_views().next().unwrap().handle();
    let original = layout
        .source_range_fragments(&origin, Range::new(1, 2))
        .unwrap();
    assert_eq!(original.local_range(), Range::new(1, 2));
    let unit = original.units().next().unwrap();
    assert_eq!(
        unit.selected_intersections()
            .map(|piece| piece.local_range())
            .collect::<Vec<_>>(),
        [Range::new(1, 2)]
    );
    assert_eq!(layout.source().text().code_units(), [0x5d0, 0x5d0]);
    assert_eq!(text_hit(&layout, Point::new(-10.25, 11.5)), 2);
    assert_eq!(text_hit(&layout, Point::new(-10.25, 5.5)), 0);
}

#[test]
fn genuine_odd_rtl_range_pieces_use_actual_visual_provider_intervals_in_h_and_ccw() {
    let bytes = crate::shaping_test_fonts::sideways_ahem();
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.charmap().map('\u{5d0}'), Some(GlyphId::new(58)));
    let mut system = system_with(bytes.clone());
    for orientation in [H, CCW] {
        let p = capture(
            &mut system,
            "\u{5d0}\u{5d0}",
            own_style(),
            true,
            orientation,
        );
        assert_eq!(
            p.analyzed_source().direction(),
            crate::Direction::RightToLeft
        );
        assert_eq!(
            p.unit_views().map(|u| u.level().get()).collect::<Vec<_>>(),
            [1, 1]
        );
        let origin = p.origin_views().next().unwrap().handle();
        let line = whole(&p);
        assert_eq!(
            glyphs(&line),
            [(58, 0.0, 0.0, 0.0, 10.0), (58, 10.0, 0.0, 0.0, 10.0)]
        );
        // H's anchor fields denote x/y; CCW's denote y/x. Both place their
        // actual raw-font baseline origin at physical(-7.25,13.5).
        let (cross, inline) = if orientation == H {
            (13.5, -7.25)
        } else {
            (-7.25, 13.5)
        };
        let layout = placed(&p, vec![(line, cross, inline)]);
        assert_eq!(layout.source().text().code_units(), [0x5d0, 0x5d0]);
        assert_eq!(
            layout
                .glyph_runs()
                .iter()
                .flat_map(|run| run.glyphs().iter().map(|g| g.range()))
                .collect::<Vec<_>>(),
            [Range::new(1, 2), Range::new(0, 1)]
        );
        // Real odd bidi puts logical source0 in visual localu10..20 and
        // source1 in0..10. Directed edges follow increasing local advance;
        // CCW maps those endpoints to decreasing physical y, without another
        // reflection for the paragraph direction.
        for (range, rect, edges, baseline) in if orientation == H {
            [
                (
                    Range::new(0, 1),
                    Rect::new(2.75, 5.5, 10.0, 10.0),
                    (2.75, 12.75),
                    13.5,
                ),
                (
                    Range::new(1, 2),
                    Rect::new(-7.25, 5.5, 10.0, 10.0),
                    (-7.25, 2.75),
                    13.5,
                ),
            ]
        } else {
            [
                (
                    Range::new(0, 1),
                    Rect::new(-15.25, -6.5, 10.0, 10.0),
                    (3.5, -6.5),
                    -7.25,
                ),
                (
                    Range::new(1, 2),
                    Rect::new(-15.25, 3.5, 10.0, 10.0),
                    (13.5, 3.5),
                    -7.25,
                ),
            ]
        } {
            let query = layout.source_range_fragments(&origin, range).unwrap();
            assert_eq!(query.local_range(), range);
            let unit = query.units().next().unwrap();
            assert_eq!(
                unit.selected_intersections()
                    .map(|piece| piece.local_range())
                    .collect::<Vec<_>>(),
                [range]
            );
            let piece = unit.pieces().next().unwrap();
            assert_eq!(piece.rect(), rect, "{orientation:?} source{range:?}");
            assert_eq!(
                piece.advance_edges(),
                edges,
                "{orientation:?} source{range:?}"
            );
            assert_eq!(piece.baseline(), baseline);
            assert_eq!(piece.effective_instance().face().bytes(), bytes);
        }
    }
}
