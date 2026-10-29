//! Native upright cells and unrotated glyph facts through the public phase front.
use crate::*;
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};

fn font() -> Vec<u8> {
    let horizontal = crate::vertical_test_fonts::horizontal(800, -200, 0, 1000);
    let count = crate::test_fonts::u16_at(crate::test_fonts::table(&horizontal, *b"maxp"), 4);
    let native = crate::vertical_test_fonts::with_native(&horizontal, 1200, 100, count);
    let bytes = crate::test_fonts::replace_tables(
        &native,
        vec![(
            *b"BASE",
            crate::baseline_test_fonts::base(
                0,
                None,
                Some(crate::baseline_test_fonts::single_axis(
                    &[*b"ideo", *b"idtp"],
                    &[0, 1200],
                )),
                None,
            ),
        )],
    );
    // Qualify the independent numeric premises before asking Text to consume them.
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(face.charmap().map('X').unwrap().to_u32(), 58);
    assert_eq!(face.hmtx().unwrap().h_metrics()[58].advance.get(), 1000);
    assert_eq!(face.vmtx().unwrap().v_metrics()[58].advance.get(), 1200);
    assert_eq!(face.vmtx().unwrap().v_metrics()[58].side_bearing.get(), 100);
    assert!(face.glyf().is_ok());
    let bounds = face
        .glyph_metrics(FontSize::unscaled(), LocationRef::default())
        .bounds(GlyphId::new(58))
        .unwrap();
    assert_eq!((bounds.y_min, bounds.y_max), (-200.0, 800.0));
    bytes
}

fn style(orientation: LineOrientation) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(if orientation == LineOrientation::Vertical {
            12.0
        } else {
            10.0
        }),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

fn prepare(value: &str, style: Style, orientation: LineOrientation) -> PreparedText {
    let source = Source::identified(value, Id::from_u64(1544007), 3);
    let len = source.text().len();
    let input = ParagraphInput {
        source: source.clone(),
        base_style: style,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: if len == 0 {
            vec![ParagraphAnchor {
                item: ParagraphAnchorItem::EmptyOrigin(0),
                affinity: Affinity::After,
            }]
        } else {
            vec![]
        },
    };
    let typography = if orientation == LineOrientation::Vertical {
        AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::RightToLeft,
        }
    } else {
        AnalysisTypography::HorizontalComposition
    };
    let analysis = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input, typography).unwrap(),
        Wrap::Word,
    );
    assert_eq!(analysis.typography(), typography);
    assert_eq!(analysis.direction(), Direction::LeftToRight);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font()).unwrap();
    system
        .prepare(
            &analysis,
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap()
}

fn start(prepared: &PreparedText) -> PreparedBoundary {
    prepared.boundary_views().next().unwrap().handle()
}

fn terminal(prepared: &PreparedText) -> PreparedOpportunity {
    prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle()
}

fn finalize(
    prepared: &PreparedText,
    start: PreparedBoundary,
    end: PreparedOpportunity,
) -> FinalizedLine {
    let finish = end.view().boundary();
    let a = start.view().participant_offset();
    let b = finish.view().participant_offset();
    let participants = prepared
        .participants()
        .skip(a)
        .take(b - a)
        .collect::<Vec<_>>();
    let controls = participants
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
    let request = LineSelectionRequest {
        boundary_context: vec![],
        incoming: None,
        start,
        end: finish,
        termination: PreparedTermination::Ordinary(end.clone()),
        alternative: None,
        participants,
        atomics: vec![],
        controls,
        line_box_struts: vec![],
        line_strut: None,
        context: LineContext {
            formatted_line: FormattedLineContext::default(),
            base_direction: ParagraphBaseDirection::LeftToRight,
            line_box_direction: LineBoxDirection::LeftToRight,
            orientation: prepared.orientation(),
            start: if a == 0 {
                LineStart::Paragraph
            } else {
                LineStart::Continuation
            },
            end: match end.view().kind() {
                PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                PreparedBreakKind::Forced => LineEnd::Forced,
                _ => LineEnd::Soft,
            },
            available_inline: None,
            alignment_insets: LineAlignmentInsets::ZERO,
            indent: 0.0,
            alignment: Alignment::Start,
            tabs: TabContext::NoTabs,
            spacing: SpacingPolicy::None,
        },
    };
    prepared
        .select(request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}

fn place(prepared: &PreparedText, rows: Vec<(FinalizedLine, f64, f64)>) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: rows
            .into_iter()
            .map(|(line, inline_origin, baseline)| {
                PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin,
                        baseline,
                    },
                    orientation: prepared.orientation(),
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap()
}

#[test]
fn real_uniform_runs_report_outline_basis_and_physical_source_geometry() {
    for (orientation, basis, cell, run_origin, end, raw) in [
        (
            LineOrientation::Horizontal,
            GlyphBasis::Unrotated,
            Rect::new(20.0, 92.0, 10.0, 10.0),
            Point::new(20.0, 100.0),
            Rect::new(30.0, 92.0, 1.0, 10.0),
            (0.0, 0.0, 10.0),
        ),
        (
            LineOrientation::SidewaysClockwise,
            GlyphBasis::Clockwise,
            Rect::new(98.0, 20.0, 10.0, 10.0),
            Point::new(100.0, 20.0),
            Rect::new(98.0, 30.0, 10.0, 1.0),
            (0.0, 0.0, 10.0),
        ),
        (
            LineOrientation::Vertical,
            GlyphBasis::Unrotated,
            Rect::new(94.0, 20.0, 12.0, 12.0),
            Point::new(100.0, 20.0),
            Rect::new(94.0, 32.0, 12.0, 1.0),
            (-6.0, 9.0, 12.0),
        ),
    ] {
        let prepared = prepare("X", style(orientation), orientation);
        let origin = prepared.origin_views().next().unwrap().handle();
        let line = finalize(&prepared, start(&prepared), terminal(&prepared));
        assert_eq!(line.run_views().next().unwrap().glyph_basis(), basis);
        let layout = place(&prepared, vec![(line.clone(), 20.0, 100.0)]);
        assert_eq!(layout.bounds(), cell);
        assert_eq!(layout.lines()[0].bounds(), cell);
        let view = layout.run_views().next().unwrap();
        assert_eq!(view.origin(), run_origin);
        assert_eq!(view.glyph_basis(), basis);
        let glyph = view.glyphs().next().unwrap();
        assert_eq!((glyph.x(), glyph.y(), glyph.advance()), raw);
        assert_eq!(glyph.pen(), 0.0);
        let owned = layout.glyph_runs();
        assert_eq!(owned[0].glyph_basis(), basis);
        assert_eq!(owned[0].origin(), run_origin);
        assert_eq!(
            (
                owned[0].glyphs()[0].x(),
                owned[0].glyphs()[0].y(),
                owned[0].glyphs()[0].advance()
            ),
            raw
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(1, Affinity::Before))
                .unwrap()
                .rect(),
            end
        );
        assert_eq!(
            layout
                .selection(Selection::new(
                    Cursor::new(0, Affinity::After),
                    Cursor::new(1, Affinity::Before)
                ))
                .unwrap()
                .rects()[0]
                .rect(),
            cell
        );
        let range = layout
            .source_range_fragments(&origin, Range::new(0, 1))
            .unwrap();
        assert_eq!(range.units().len(), 1);
        let unit = range.units().next().unwrap();
        assert_eq!(unit.finalized_line(), line);
        assert_eq!(
            unit.selected_intersections()
                .map(|piece| piece.local_range())
                .collect::<Vec<_>>(),
            [Range::new(0, 1)]
        );
        let piece = unit.pieces().next().unwrap();
        assert_eq!(piece.rect(), cell);
        assert_eq!(piece.advance_edges(), (20.0, 20.0 + f64::from(raw.2)));
        assert_eq!(piece.baseline(), 100.0);
        assert_eq!(
            piece.effective_instance().face().bytes(),
            view.face().bytes()
        );
        assert_eq!(layout.source().id(), Some(Id::from_u64(1544007)));
        assert_eq!(layout.source().revision(), 3);
    }
}

#[test]
fn upright_line_movement_and_hits_follow_actual_columns_in_either_stack_direction() {
    let prepared = prepare(
        "X\nX",
        style(LineOrientation::Vertical),
        LineOrientation::Vertical,
    );
    let cut = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let first = finalize(&prepared, start(&prepared), cut.clone());
    let second = finalize(&prepared, cut.view().boundary(), terminal(&prepared));
    for next_cross in [60.0, 140.0] {
        let layout = place(
            &prepared,
            vec![
                (first.clone(), 20.0, 100.0),
                (second.clone(), 50.0, next_cross),
            ],
        );
        assert_eq!(
            layout.lines()[1].bounds(),
            Rect::new(next_cross - 6.0, 50.0, 12.0, 12.0)
        );
        let begin = Cursor::new(0, Affinity::After);
        let exact = layout
            .positions()
            .find(|position| position.source_cursor() == begin)
            .unwrap();
        let next = layout.move_position(&exact, Movement::NextLine).unwrap();
        assert_eq!(next.source_cursor(), Cursor::new(2, Affinity::After));
        assert_eq!(
            layout.position_geometry(&next).unwrap().rect(),
            Rect::new(next_cross - 6.0, 50.0, 12.0, 1.0)
        );
        assert_eq!(
            layout.move_cursor(begin, Movement::NextLine).unwrap(),
            next.source_cursor()
        );
        let Hit::Text(hit) = layout.hit(Point::new(next_cross, 51.0)) else {
            panic!("actual second-column text hit")
        };
        assert_eq!(hit.source_cursor(), Cursor::new(2, Affinity::After));
        assert_eq!(
            layout.position_geometry(&hit).unwrap().rect(),
            Rect::new(next_cross - 6.0, 50.0, 12.0, 1.0)
        );
        let Hit::Text(end) = layout.hit(Point::new(next_cross, 61.0)) else {
            panic!("actual second-column end hit")
        };
        assert_eq!(end.source_cursor(), Cursor::new(3, Affinity::Before));
    }
}

#[test]
fn upright_explicit_decorations_map_cross_thickness_without_horizontal_defaults() {
    let style = Style {
        underline: Decoration::solid()
            .with_offset(DecorationOffset::try_absolute(-2.0).unwrap())
            .with_thickness(DecorationThickness::try_absolute(1.0).unwrap()),
        ..style(LineOrientation::Vertical)
    };
    let prepared = prepare("X", style, LineOrientation::Vertical);
    let line = finalize(&prepared, start(&prepared), terminal(&prepared));
    let layout = place(&prepared, vec![(line, 20.0, 100.0)]);
    let decorations = layout.decoration_views().collect::<Vec<_>>();
    assert_eq!(decorations.len(), 1);
    assert_eq!(decorations[0].rect(), Rect::new(97.0, 20.0, 1.0, 12.0));
    assert_eq!(layout.decorations(), decorations);
}

#[test]
fn upright_nonformatted_receipt_retains_orientation_without_visible_geometry() {
    let prepared = prepare(
        "",
        style(LineOrientation::Vertical),
        LineOrientation::Vertical,
    );
    let line = finalize(&prepared, start(&prepared), terminal(&prepared));
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared,
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::NonFormatted(NonFormattedLinePlacement {
            line: line.nonformatted().unwrap(),
            origin: Point::new(100.0, 20.0),
            orientation: LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.orientation(), LineOrientation::Vertical);
    assert_eq!(layout.nonformatted_lines().count(), 1);
    assert_eq!(
        layout.nonformatted_lines().next().unwrap().orientation,
        LineOrientation::Vertical
    );
    assert!(layout.lines().is_empty());
    assert_eq!(layout.run_views().count(), 0);
    assert!(layout.glyph_runs().is_empty());
    assert_eq!(layout.positions().count(), 0);
    assert_eq!(layout.bounds(), Rect::default());
}

#[cfg(feature = "text-accessibility")]
#[test]
fn placed_upright_accessibility_uses_ttb_and_actual_height_span() {
    let prepared = prepare(
        "X",
        Style {
            line_height: LineHeight::Absolute(16.0),
            ..style(LineOrientation::Vertical)
        },
        LineOrientation::Vertical,
    );
    let line = finalize(&prepared, start(&prepared), terminal(&prepared));
    let layout = place(&prepared, vec![(line, 20.0, 100.0)]);
    let parent = accesskit::NodeId(1544007);
    let source_update = prepared
        .analyzed_source()
        .accessibility_update(&mut Accessibility::default(), parent, parent)
        .unwrap();
    assert!(source_update.nodes.iter().all(|(_, node)| {
        node.bounds().is_none()
            && node.font_family().is_none()
            && node.font_size().is_none()
            && node.character_widths().is_none()
    }));
    let source_text = source_update
        .nodes
        .iter()
        .find(|(_, node)| node.role() == accesskit::Role::TextRun)
        .unwrap();
    assert_eq!(source_text.1.value(), Some("X"));
    let update = layout
        .accessibility_update(
            &mut Accessibility::default(),
            parent,
            parent,
            Point::new(5.0, 7.0),
        )
        .unwrap();
    let nodes = update
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == accesskit::Role::TextRun)
        .collect::<Vec<_>>();
    assert_eq!(nodes.len(), 1);
    let node = &nodes[0].1;
    assert_eq!(
        node.text_direction(),
        Some(accesskit::TextDirection::TopToBottom)
    );
    assert_eq!(node.character_widths(), Some(&[12.0][..]));
    let bounds = node.bounds().unwrap();
    // Cross extent16 and inline extent12 distinguish physical height from width.
    assert_eq!(
        (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
        (97.0, 27.0, 113.0, 39.0)
    );
}
