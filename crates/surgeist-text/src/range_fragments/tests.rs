use super::*;
use crate::*;
const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
fn style(size: f32) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size,
        line_height: LineHeight::Absolute(40.0),
        ..Style::default()
    }
}
fn system(bytes: Vec<u8>) -> System {
    let mut s = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    s.register_fonts(bytes).unwrap();
    s
}
fn input(
    source: Source,
    style: Style,
    cuts: &[usize],
    direction: ParagraphBaseDirection,
) -> ParagraphInput {
    ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: style,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: cuts
            .windows(2)
            .map(|p| ParagraphOrigin {
                source: source.clone(),
                local: Range::new(p[0], p[1]),
                aggregate: Range::new(p[0], p[1]),
                kind: ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: cuts[1..cuts.len() - 1]
            .iter()
            .map(|&at| ParagraphConnection {
                at,
                kind: ParagraphConnectionKind::Continuous,
            })
            .collect(),
        controls: vec![],
        boundary_policies: (0..cuts.len() - 2)
            .map(|i| ParagraphBoundaryPolicy {
                definition: ParagraphBoundaryDefinition::Connection(i),
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            })
            .collect(),
        replacements: vec![],
        anchor_order: vec![],
    }
}
fn prepare(input: ParagraphInput, bytes: Vec<u8>) -> PreparedText {
    system(bytes)
        .prepare(
            &crate::test_wrapping::bind_missing(
                &AnalyzedSource::from_paragraph(input).unwrap(),
                crate::Wrap::Word,
            ),
            PreparationOptions::default(),
        )
        .unwrap()
}
fn select(
    prepared: &PreparedText,
    start: PreparedBoundary,
    termination: PreparedOpportunity,
    alternative: Option<PreparedAlternative>,
    width: Option<f64>,
) -> FinalizedLine {
    let end = termination.view().boundary();
    let from = start.view().participant_offset();
    let to = end.view().participant_offset();
    let parts: Vec<_> = prepared.participants().skip(from).take(to - from).collect();
    let controls = parts
        .iter()
        .filter_map(|p| {
            if let PreparedParticipant::Control(c) = p {
                Some(ControlAllocation {
                    occurrence: c.clone(),
                    strut: None,
                })
            } else {
                None
            }
        })
        .collect();
    prepared
        .select(LineSelectionRequest {
            incoming: None,
            start,
            end,
            termination: crate::PreparedTermination::Ordinary(termination.clone()),
            alternative,
            participants: parts.clone(),
            atomics: parts
                .iter()
                .filter_map(|p| match p {
                    PreparedParticipant::Atomic(occurrence) => Some(AtomicAllocation {
                        occurrence: occurrence.clone(),
                        allocation: AtomicLineAllocation::OutOfFlow,
                    }),
                    _ => None,
                })
                .collect(),
            controls,
            line_strut: (prepared.inner.analyzed.paragraph.paragraph_strut
                == ParagraphStrut::AllocatedContributions)
                .then_some(LineStrut {
                    baseline: 10.0,
                    line_extent: 20.0,
                }),
            context: LineContext {
                base_direction: if prepared.analyzed_source().direction() == Direction::RightToLeft
                {
                    ParagraphBaseDirection::RightToLeft
                } else {
                    ParagraphBaseDirection::LeftToRight
                },
                orientation: LineOrientation::Horizontal,
                start: if from == 0 {
                    LineStart::Paragraph
                } else {
                    LineStart::Continuation
                },
                end: match termination.view().kind() {
                    PreparedBreakKind::Forced => LineEnd::Forced,
                    PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                    _ => LineEnd::Soft,
                },
                available_inline: width,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,

                spacing: SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn place(prepared: PreparedText, x: f64, y: f64) -> Layout {
    let ends: Vec<_> = prepared
        .opportunity_views()
        .filter(|v| {
            matches!(
                v.kind(),
                PreparedBreakKind::Forced | PreparedBreakKind::ParagraphEnd
            )
        })
        .map(|v| v.handle())
        .collect();
    let mut start = prepared.boundary_views().next().unwrap().handle();
    let mut lines = Vec::new();
    for (index, end) in ends.into_iter().enumerate() {
        let next = end.view().boundary();
        let line = select(&prepared, start, end, None, None);
        let atomics = line
            .atomic_slots()
            .map(|slot| AtomicPlacement {
                occurrence: slot.occurrence,
                border: Rect::new(0.0, 0.0, 5.0, 10.0),
                content: Rect::new(0.0, 0.0, 5.0, 10.0),
                baseline: 8.0,
            })
            .collect();
        lines.push(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: x,
                baseline: y + index as f64 * 40.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics,
        });
        start = next;
    }
    Layout::from_finalized_lines(PlacementRequest {
        prepared,
        coverage: PlacementCoverage::WholeParagraph,
        lines,
    })
    .unwrap()
}
fn rectangles(result: &SourceRangeFragments<'_>) -> Vec<Rect> {
    result
        .units()
        .flat_map(|u| u.pieces().map(|p| p.rect()).collect::<Vec<_>>())
        .collect()
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 0.00001, "{a} != {b}");
}
#[test]
fn actual_zero_formatting_units_use_own_font_metrics_and_distinct_placement_identity() {
    let prepared = prepare(
        input(
            Source::new("\u{2069}\u{2069}A"),
            style(16.0),
            &[0, 3],
            ParagraphBaseDirection::LeftToRight,
        ),
        AHEM.to_vec(),
    );
    let origin = prepared.origin_views().next().unwrap().handle();
    let layout = place(prepared.clone(), 7.0, 30.0);
    let a = layout
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    let b = layout
        .source_range_fragments(&origin, Range::new(1, 2))
        .unwrap();
    let first = a.units().next().unwrap();
    let second = b.units().next().unwrap();
    assert!(!first.same_unit(&second));
    assert_eq!(first.pieces().count(), 1);
    let rect = first.pieces().next().unwrap().rect();
    near(rect.origin.x, 7.0);
    near(rect.origin.y, 17.2);
    near(rect.size.width, 0.0);
    near(rect.size.height, 16.0);
    assert_eq!(
        first
            .pieces()
            .next()
            .unwrap()
            .effective_instance()
            .face()
            .bytes(),
        AHEM
    );
    let clone = layout.clone();
    let same = clone
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    assert!(first.same_unit(&same.units().next().unwrap()));
    let other = place(prepared, 27.0, 50.0);
    let moved = other
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    assert!(!first.same_unit(&moved.units().next().unwrap()));
    near(rectangles(&moved)[0].origin.y, 37.2);
}
#[test]
fn optional_ligature_has_distinct_range_units_with_equal_shaped_shares() {
    let prepared = prepare(
        input(
            Source::new("AA"),
            style(12.5),
            &[0, 1, 2],
            ParagraphBaseDirection::LeftToRight,
        ),
        crate::shaping_test_fonts::ligature_ahem(),
    );
    let origins: Vec<_> = prepared.origin_views().map(|v| v.handle()).collect();
    let layout = place(prepared, 0.0, 30.0);
    let a = layout
        .source_range_fragments(&origins[0], Range::new(0, 1))
        .unwrap();
    let b = layout
        .source_range_fragments(&origins[1], Range::new(1, 2))
        .unwrap();
    let first = a.units().next().unwrap();
    let second = b.units().next().unwrap();
    assert!(!first.same_unit(&second));
    assert_eq!(
        first
            .selected_intersections()
            .map(|i| i.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1)]
    );
    assert_eq!(
        first
            .expanded_intersections()
            .map(|i| i.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1)]
    );
    near(first.pieces().map(|p| p.rect().size.width).sum(), 6.25);
    near(second.pieces().map(|p| p.rect().size.width).sum(), 6.25);
    near(second.pieces().next().unwrap().rect().origin.x, 6.25);
    assert_ne!(rectangles(&a), rectangles(&b));
    assert_eq!(first.fragments().count(), 1);
    let whole = prepare(
        input(
            Source::new("AA"),
            style(12.5),
            &[0, 2],
            ParagraphBaseDirection::LeftToRight,
        ),
        crate::shaping_test_fonts::ligature_ahem(),
    );
    let origin = whole.origin_views().next().unwrap().handle();
    let layout = place(whole, 0.0, 30.0);
    let both = layout
        .source_range_fragments(&origin, Range::new(0, 2))
        .unwrap();
    assert_eq!(both.units().count(), 2);
    near(rectangles(&both).iter().map(|r| r.size.width).sum(), 12.5);
}
#[test]
fn surrogate_halves_and_cross_font_grapheme_parts_expand_without_borrowing_neighbor_metrics() {
    let prepared = prepare(
        input(
            Source::new("\u{1f600}"),
            style(10.0),
            &[0, 2],
            ParagraphBaseDirection::LeftToRight,
        ),
        AHEM.to_vec(),
    );
    let o = prepared.origin_views().next().unwrap().handle();
    let layout = place(prepared, 0.0, 30.0);
    let a = layout.source_range_fragments(&o, Range::new(0, 1)).unwrap();
    let b = layout.source_range_fragments(&o, Range::new(1, 2)).unwrap();
    assert!(
        a.units()
            .next()
            .unwrap()
            .same_unit(&b.units().next().unwrap())
    );
    assert_eq!(
        a.units()
            .next()
            .unwrap()
            .expanded_intersections()
            .next()
            .unwrap()
            .local_range(),
        Range::new(0, 2)
    );
    let mut source = Source::new("A\u{301}");
    source.span(Range::new(1, 2), style(20.0));
    let prepared = prepare(
        input(
            source,
            style(10.0),
            &[0, 2],
            ParagraphBaseDirection::LeftToRight,
        ),
        crate::shaping_test_fonts::mark_ahem(),
    );
    let o = prepared.origin_views().next().unwrap().handle();
    let layout = place(prepared, 0.0, 30.0);
    let partial = layout.source_range_fragments(&o, Range::new(1, 2)).unwrap();
    let unit = partial.units().next().unwrap();
    assert_eq!(partial.units().count(), 1);
    assert_eq!(
        unit.pieces()
            .map(|p| p.effective_instance().font_size())
            .collect::<Vec<_>>(),
        [10.0, 20.0]
    );
    assert_eq!(
        unit.pieces()
            .map(|p| p.rect().size.height)
            .collect::<Vec<_>>(),
        [10.0, 20.0]
    );
}
#[test]
fn removed_duplicates_and_edges_have_custody_without_selecting_the_representative() {
    let mut s = style(10.0);
    s.white_space = WhiteSpace::Collapse;
    let p = prepare(
        input(
            Source::new("A   B  "),
            s,
            &[0, 7],
            ParagraphBaseDirection::LeftToRight,
        ),
        AHEM.to_vec(),
    );
    let o = p.origin_views().next().unwrap().handle();
    let l = place(p, 0.0, 30.0);
    assert!(
        l.source_range_fragments(&o, Range::new(2, 4))
            .unwrap()
            .units()
            .next()
            .is_none()
    );
    assert!(
        l.source_range_fragments(&o, Range::new(5, 7))
            .unwrap()
            .units()
            .next()
            .is_none()
    );
    let representative = l.source_range_fragments(&o, Range::new(1, 2)).unwrap();
    near(
        rectangles(&representative)
            .iter()
            .map(|r| r.size.width)
            .sum(),
        10.0,
    );
    assert_eq!(
        l.source_range_fragments(&o, Range::new(0, 7))
            .unwrap()
            .units()
            .count(),
        3
    );
}
#[test]
fn preserved_zero_advance_space_remains_eligible_independently_of_discarded_custody() {
    let mut s = style(10.0);
    s.word_spacing = -10.0;
    let p = prepare(
        input(
            Source::new("A A"),
            s,
            &[0, 3],
            ParagraphBaseDirection::LeftToRight,
        ),
        AHEM.to_vec(),
    );
    let origin = p.origin_views().next().unwrap().handle();
    let layout = place(p, 0.0, 30.0);
    let q = layout
        .source_range_fragments(&origin, Range::new(1, 2))
        .unwrap();
    assert_eq!(q.units().count(), 1);
    let piece = q.units().next().unwrap().pieces().next().unwrap();
    near(piece.rect().origin.x, 10.0);
    near(piece.rect().size.width, 0.0);
    near(piece.rect().size.height, 10.0);
}

#[test]
fn preserved_newlines_belong_to_the_terminating_line_and_mapped_crlf_halves_share_one_unit() {
    for text in ["\n", "\n\n", "A\n"] {
        let len = text.len();
        let p = prepare(
            input(
                Source::new(text),
                style(10.0),
                &[0, len],
                ParagraphBaseDirection::LeftToRight,
            ),
            AHEM.to_vec(),
        );
        let o = p.origin_views().next().unwrap().handle();
        let l = place(p, 0.0, 30.0);
        let q = l
            .source_range_fragments(&o, Range::new(len - 1, len))
            .unwrap();
        assert_eq!(q.units().count(), 1);
        let unit = q.units().next().unwrap();
        assert_eq!(
            unit.line_index(),
            text.chars().filter(|c| *c == '\n').count() - 1
        );
        near(unit.pieces().next().unwrap().rect().size.width, 0.0);
        near(unit.pieces().next().unwrap().rect().size.height, 10.0);
    }
    let mut capture = input(
        Source::new("\r\n"),
        style(10.0),
        &[0, 2],
        ParagraphBaseDirection::LeftToRight,
    );
    capture.segment_breaks = DocumentSegmentBreaks::Defined(vec![Range::new(0, 2)]);
    let p = prepare(capture, AHEM.to_vec());
    let o = p.origin_views().next().unwrap().handle();
    let l = place(p, 0.0, 30.0);
    let a = l.source_range_fragments(&o, Range::new(0, 1)).unwrap();
    let b = l.source_range_fragments(&o, Range::new(1, 2)).unwrap();
    assert!(
        a.units()
            .next()
            .unwrap()
            .same_unit(&b.units().next().unwrap())
    );
    assert_eq!(
        a.units()
            .next()
            .unwrap()
            .expanded_intersections()
            .next()
            .unwrap()
            .local_range(),
        Range::new(0, 2)
    );
}
#[test]
fn origin_admission_is_allocation_qualified_and_empty_requests_are_rejected() {
    let build = || {
        prepare(
            input(
                Source::new("AB"),
                style(10.0),
                &[0, 1, 2],
                ParagraphBaseDirection::LeftToRight,
            ),
            AHEM.to_vec(),
        )
    };
    let p = build();
    let o = p.origin_views().next().unwrap().handle();
    let foreign = build().origin_views().next().unwrap().handle();
    let l = place(p, 0.0, 30.0);
    assert!(matches!(
        l.source_range_fragments(&foreign, Range::new(0, 1))
            .unwrap_err()
            .detail(),
        Some(ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::ForeignPreparation { .. }
        })
    ));
    assert!(matches!(
        l.source_range_fragments(&o, Range::new(0, 0))
            .unwrap_err()
            .detail(),
        Some(ErrorDetail::SourceRangeFragments {
            reason: SourceRangeFragmentError::EmptyRange
        })
    ));
    assert!(matches!(
        l.source_range_fragments(&o, Range::new(0, 2))
            .unwrap_err()
            .detail(),
        Some(ErrorDetail::SourceRangeFragments {
            reason: SourceRangeFragmentError::OutsideOrigin { .. }
        })
    ));
}

#[test]
fn rtl_pdi_keeps_post_l1_l2_x_and_original_local_origin_offsets() {
    let source = Source::new("X\u{2067}XX\u{2069} X");
    let mut s = style(16.0);
    s.white_space = WhiteSpace::Collapse;
    let mut capture = input(
        source,
        s,
        &[0, 1, 5, 7],
        ParagraphBaseDirection::RightToLeft,
    );
    for (row, text) in capture
        .origins
        .iter_mut()
        .zip(["X", "\u{2067}XX\u{2069}", " X"])
    {
        row.source = Source::new(text);
        row.local = Range::new(0, text.encode_utf16().count());
    }
    let p = prepare(capture, AHEM.to_vec());
    let o = p.origin_views().nth(1).unwrap().handle();
    let soft = p
        .opportunity_views()
        .find(|v| {
            v.kind() == PreparedBreakKind::Allowed && v.boundary().view().source_before().get() == 6
        })
        .unwrap()
        .handle();
    let end = p
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let first = select(
        &p,
        p.boundary_views().next().unwrap().handle(),
        soft.clone(),
        None,
        Some(72.0),
    );
    let second = select(&p, soft.view().boundary(), end, None, Some(72.0));
    let l = Layout::from_finalized_lines(PlacementRequest {
        prepared: p,
        coverage: PlacementCoverage::WholeParagraph,
        lines: vec![
            LinePlacement {
                line: first,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 30.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            },
            LinePlacement {
                line: second,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 70.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            },
        ],
    })
    .unwrap();
    let q = l.source_range_fragments(&o, Range::new(3, 4)).unwrap();
    let u = q.units().next().unwrap();
    let r = u.pieces().next().unwrap().rect();
    assert_eq!((r.origin.x, r.size.width), (24.0, 0.0));
    near(r.origin.y, 17.2);
    near(r.size.height, 16.0);
    assert_eq!(
        u.selected_intersections().next().unwrap().aggregate_range(),
        Range::new(4, 5)
    );
    assert_eq!(
        u.selected_intersections().next().unwrap().local_range(),
        Range::new(3, 4)
    );
    let content = l.source_range_fragments(&o, Range::new(0, 4)).unwrap();
    assert_eq!(
        content
            .units()
            .flat_map(|u| u
                .selected_intersections()
                .map(|i| i.local_range())
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [
            Range::new(0, 1),
            Range::new(1, 2),
            Range::new(2, 3),
            Range::new(3, 4)
        ]
    );
}
#[test]
fn own_formatting_font_survives_catalogue_replacement_and_system_drop() {
    let mut source = Source::new("A\u{2069}B");
    source.span(Range::new(1, 2), style(20.0));
    let a = AnalyzedSource::from_paragraph(input(
        source,
        style(10.0),
        &[0, 3],
        ParagraphBaseDirection::LeftToRight,
    ))
    .unwrap();
    let mut fonts = system(AHEM.to_vec());
    let p = fonts.prepare(&a, PreparationOptions::default()).unwrap();
    let old = fonts.font_catalogue_identity().clone();
    fonts
        .register_fonts(crate::test_fonts::variable_ahem())
        .unwrap();
    assert_ne!(fonts.font_catalogue_identity(), &old);
    let o = p.origin_views().next().unwrap().handle();
    drop(fonts);
    let l = place(p, 0.0, 30.0);
    let q = l.source_range_fragments(&o, Range::new(1, 2)).unwrap();
    let piece = q.units().next().unwrap().pieces().next().unwrap();
    assert_eq!(piece.effective_instance().font_size(), 20.0);
    assert_eq!(piece.effective_instance().face().catalogue_identity(), &old);
    near(piece.rect().origin.y, 14.0);
    near(piece.rect().size.height, 20.0);
    near(piece.rect().origin.x, 10.0);
}
#[test]
fn wholly_removed_original_needs_no_fabricated_font_and_newline_modes_stay_distinct() {
    let mut s = style(10.0);
    s.font = Font::new().family("Unavailable");
    s.white_space = WhiteSpace::Discard;
    let mut capture = input(
        Source::new("  \n  "),
        s,
        &[0, 5],
        ParagraphBaseDirection::LeftToRight,
    );
    capture.paragraph_strut = ParagraphStrut::AllocatedContributions;
    let mut no_fonts = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    let p = no_fonts
        .prepare(
            &AnalyzedSource::from_paragraph(capture).unwrap(),
            PreparationOptions::default(),
        )
        .unwrap();
    let o = p.origin_views().next().unwrap().handle();
    let l = place(p, 0.0, 30.0);
    assert_eq!(
        l.source_range_fragments(&o, Range::new(0, 5))
            .unwrap()
            .units()
            .count(),
        0
    );
    let p = prepare(
        input(
            Source::new("\r\n"),
            style(10.0),
            &[0, 2],
            ParagraphBaseDirection::LeftToRight,
        ),
        AHEM.to_vec(),
    );
    let o = p.origin_views().next().unwrap().handle();
    let l = place(p, 0.0, 30.0);
    let cr = l.source_range_fragments(&o, Range::new(0, 1)).unwrap();
    let lf = l.source_range_fragments(&o, Range::new(1, 2)).unwrap();
    assert!(
        !cr.units()
            .next()
            .unwrap()
            .same_unit(&lf.units().next().unwrap())
    );
    near(rectangles(&cr)[0].size.width, 10.0);
    near(rectangles(&lf)[0].size.width, 0.0);
}
#[test]
fn late_real_font_numeric_failure_exposes_no_partial_result_and_leaves_layout_unchanged() {
    let mut source = Source::new("AB");
    let mut large = style(64.0);
    large.line_height = LineHeight::Absolute(1024.0);
    let mut small = style(1.0);
    small.line_height = large.line_height;
    source.span(Range::new(1, 2), small);
    let p = prepare(
        input(source, large, &[0, 2], ParagraphBaseDirection::LeftToRight),
        AHEM.to_vec(),
    );
    let o = p.origin_views().next().unwrap().handle();
    let l = place(p, 0.0, 1e17);
    let before = l.metrics();
    assert_eq!(
        l.source_range_fragments(&o, Range::new(0, 1))
            .unwrap()
            .units()
            .count(),
        1
    );
    let error = l.source_range_fragments(&o, Range::new(0, 2)).unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::InvalidTextNumeric {
            phase: TextNumericPhase::SourceRangeFragments,
            category: TextNumericError::NonZeroLost,
            ..
        })
    ));
    assert_eq!(l.metrics(), before);
    assert_eq!(
        l.source_range_fragments(&o, Range::new(0, 1))
            .unwrap()
            .units()
            .count(),
        1
    );
}
#[test]
fn partial_placement_rejects_outside_original_even_when_origin_covers_both_lines() {
    let p = prepare(
        input(
            Source::new("A\nB"),
            style(10.0),
            &[0, 3],
            ParagraphBaseDirection::LeftToRight,
        ),
        AHEM.to_vec(),
    );
    let o = p.origin_views().next().unwrap().handle();
    let forced = p
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let start = p.boundary_views().next().unwrap().handle();
    let line = select(&p, start.clone(), forced.clone(), None, None);
    let l = Layout::from_finalized_lines(PlacementRequest {
        prepared: p,
        coverage: PlacementCoverage::Range {
            start,
            end: forced.view().boundary(),
        },
        lines: vec![LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 30.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        }],
    })
    .unwrap();
    assert!(matches!(
        l.source_range_fragments(&o, Range::new(0, 3))
            .unwrap_err()
            .detail(),
        Some(ErrorDetail::SourceRangeFragments {
            reason: SourceRangeFragmentError::OutsidePlacement { .. }
        })
    ));
    assert_eq!(
        l.source_range_fragments(&o, Range::new(1, 2))
            .unwrap()
            .units()
            .count(),
        1
    );
}
#[test]
fn selected_source_and_control_alternatives_keep_authentic_source_provenance() {
    for (control, replacement) in [(false, "-"), (false, "A"), (true, "\u{301}"), (true, "A")] {
        let shared_egc = control && replacement == "\u{301}";
        let source = Source::new(if control { "XABBB" } else { "XA\u{ad}BBB" });
        let len = source.text().len();
        let mut capture = input(
            source,
            style(10.0),
            &[0, len],
            ParagraphBaseDirection::LeftToRight,
        );
        let definition = if control {
            capture.controls.push(ParagraphControl {
                at: 2,
                kind: ParagraphControlKind::DiscretionaryBreak {
                    style: Box::new(capture.base_style.clone()),
                },
            });
            capture.anchor_order.push(ParagraphAnchor {
                item: ParagraphAnchorItem::Control(0),
                affinity: Affinity::Before,
            });
            ParagraphReplacementDefinition::Control {
                control: 0,
                origin: None,
            }
        } else {
            ParagraphReplacementDefinition::Source {
                origin: 0,
                source: Range::new(2, 3),
            }
        };
        capture.replacements.push(
            ParagraphReplacement::resolved_native_logical_before(
                definition,
                if control { 2 } else { 3 },
                vec![Text::from(replacement)],
            )
            .unwrap(),
        );
        let p = prepare(
            capture,
            if replacement == "A" {
                crate::shaping_test_fonts::ligature_ahem()
            } else {
                crate::shaping_test_fonts::mark_ahem()
            },
        );
        let o = p.origin_views().next().unwrap().handle();
        let offer = p
            .opportunity_views()
            .find(|v| v.kind() == PreparedBreakKind::Discretionary)
            .unwrap()
            .handle();
        let alternative = offer.view().alternatives().next().unwrap().handle();
        let end = p
            .opportunity_views()
            .find(|v| v.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        let first = select(
            &p,
            p.boundary_views().next().unwrap().handle(),
            offer.clone(),
            Some(alternative.clone()),
            None,
        );
        let second = select(&p, offer.view().boundary(), end, None, None);
        let l = Layout::from_finalized_lines(PlacementRequest {
            prepared: p,
            coverage: PlacementCoverage::WholeParagraph,
            lines: vec![
                LinePlacement {
                    line: first,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: 30.0,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                },
                LinePlacement {
                    line: second,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: 70.0,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                },
            ],
        })
        .unwrap();
        let a = l.source_range_fragments(&o, Range::new(1, 2)).unwrap();
        let unit = a.units().next().unwrap();
        assert_eq!(
            unit.expanded_intersections()
                .map(|i| i.local_range())
                .collect::<Vec<_>>(),
            [Range::new(1, 2)]
        );
        assert_eq!(
            unit.fragments()
                .any(|f| f.view().owner()
                    == FinalizedFragmentOwner::Alternative(alternative.clone())),
            shared_egc
        );
        if !control {
            let q = l.source_range_fragments(&o, Range::new(2, 3)).unwrap();
            assert_eq!(q.units().count(), 1);
            let actual = q.units().next().unwrap();
            assert_eq!(actual.alternatives().next(), Some(alternative.clone()));
            assert_eq!(
                actual
                    .selected_intersections()
                    .next()
                    .unwrap()
                    .local_range(),
                Range::new(2, 3)
            );
            assert_eq!(
                actual.defining_sources().next().unwrap().local_range(),
                Range::new(2, 3)
            );
            assert_eq!(actual.expanded_intersections().count(), 0);
            near(
                actual.pieces().map(|p| p.rect().size.width).sum(),
                if replacement == "A" { 5.0 } else { 10.0 },
            );
            assert!(!actual.same_unit(&unit));
        }
    }
}

#[test]
fn contextual_holes_remain_real_pieces_of_shared_units_and_generated_origins_are_not_dom_claims() {
    let mut source = Source::new("AIA");
    source.inline_box(InlineBox::new(
        Id::from_u64(7),
        InlineBoxKind::OutOfFlow,
        1,
        Size::new(5.0, 10.0),
    ));
    let capture = ContextualSourceInput {
        source: source.clone(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        origins: (0..3)
            .map(|i| ParagraphOrigin {
                source: source.clone(),
                local: Range::new(i, i + 1),
                aggregate: Range::new(i, i + 1),
                kind: ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: vec![SourceProcessingConnection {
            before_origin: 0,
            after_origin: 2,
            kind: ParagraphConnectionKind::Continuous,
        }],
        controls: vec![],
        boundary_policies: vec![ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(0),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        }],
        replacements: vec![],
        scopes: vec![
            SourceProcessingScope::Block {
                used_direction: crate::ParagraphBaseDirection::LeftToRight,
                trim: WhiteSpaceTrim::default(),
                base_style: style(10.0),
            },
            SourceProcessingScope::OutOfFlowBlock {
                used_direction: crate::ParagraphBaseDirection::LeftToRight,
                trim: WhiteSpaceTrim::default(),
                base_style: style(20.0),
                atomic: 0,
            },
        ],
        paragraphs: vec![
            SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont
            };
            2
        ],
        events: vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::Atomic {
                atomic: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::EnterScope(1),
            SourceProcessingEvent::EnterParagraph(1),
            SourceProcessingEvent::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(1),
            SourceProcessingEvent::ExitScope(1),
            SourceProcessingEvent::Connection(0),
            SourceProcessingEvent::Origin {
                origin: 2,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::ExitScope(0),
        ],
    };
    let processed = Arc::new(process_contextual_source(capture).unwrap());
    let mut fonts = system(crate::shaping_test_fonts::ligature_ahem());
    let p = fonts
        .prepare(
            &AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap())
                .unwrap(),
            PreparationOptions::default(),
        )
        .unwrap();
    let origins: Vec<_> = p.origin_views().map(|v| v.handle()).collect();
    let l = place(p, 0.0, 30.0);
    let a = l
        .source_range_fragments(&origins[0], Range::new(0, 1))
        .unwrap();
    let b = l
        .source_range_fragments(&origins[1], Range::new(2, 3))
        .unwrap();
    let unit = a.units().next().unwrap();
    assert!(!unit.same_unit(&b.units().next().unwrap()));
    assert_eq!(
        unit.expanded_intersections()
            .map(|i| i.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1)]
    );
    near(unit.pieces().map(|p| p.rect().size.width).sum(), 5.0);
    assert!(
        l.source_range_fragments(&origins[0], Range::new(1, 2))
            .is_err()
    );
    let child = fonts
        .prepare(
            &AnalyzedSource::from_processed_paragraph(processed.paragraphs().nth(1).unwrap())
                .unwrap(),
            PreparationOptions::default(),
        )
        .unwrap();
    let child_origin = child.origin_views().next().unwrap().handle();
    assert!(
        l.source_range_fragments(&child_origin, Range::new(1, 2))
            .is_err()
    );
    let mut generated = input(
        Source::new("A"),
        style(10.0),
        &[0, 1],
        ParagraphBaseDirection::LeftToRight,
    );
    generated.origins[0].kind = ParagraphOriginKind::Generated;
    let p = prepare(generated, AHEM.to_vec());
    let o = p.origin_views().next().unwrap().handle();
    let l = place(p, 0.0, 30.0);
    let q = l.source_range_fragments(&o, Range::new(0, 1)).unwrap();
    assert_eq!(
        q.units()
            .next()
            .unwrap()
            .selected_intersections()
            .next()
            .unwrap()
            .origin()
            .view()
            .kind(),
        ParagraphOriginKind::Generated
    );
}

fn place_alternative_pair(
    p: PreparedText,
    offer: PreparedOpportunity,
    alternative: Option<PreparedAlternative>,
) -> Layout {
    let end = p
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let first = select(
        &p,
        p.boundary_views().next().unwrap().handle(),
        offer.clone(),
        alternative,
        None,
    );
    let second = select(&p, offer.view().boundary(), end, None, None);
    Layout::from_finalized_lines(PlacementRequest {
        prepared: p,
        coverage: PlacementCoverage::WholeParagraph,
        lines: vec![
            LinePlacement {
                line: first,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 30.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            },
            LinePlacement {
                line: second,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 70.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            },
        ],
    })
    .unwrap()
}
#[test]
fn source_definitions_select_only_the_exact_chosen_nonempty_replacement_and_keep_empty_and_unchosen_roles()
 {
    for replacement in ["-", "", "\u{2069}"] {
        let mut capture = input(
            Source::new("AX\u{ad}\u{ad}BBB"),
            style(10.0),
            &[0, 2, 3, 4, 7],
            ParagraphBaseDirection::LeftToRight,
        );
        for (origin, text) in capture
            .origins
            .iter_mut()
            .zip(["AX", "\u{ad}", "\u{ad}", "BBB"])
        {
            origin.source = Source::new(text);
            origin.local = Range::new(0, origin.source.text().len());
        }
        capture.replacements.push(
            ParagraphReplacement::resolved_native_logical_before(
                ParagraphReplacementDefinition::Source {
                    origin: 2,
                    source: Range::new(3, 4),
                },
                4,
                vec![Text::from(replacement)],
            )
            .unwrap(),
        );
        let p = prepare(capture, AHEM.to_vec());
        let origins: Vec<_> = p.origin_views().map(|v| v.handle()).collect();
        let offer = p
            .opportunity_views()
            .find(|v| {
                v.boundary().view().source_before().get() == 4 && v.alternatives().next().is_some()
            })
            .unwrap()
            .handle();
        let alt = offer.view().alternatives().next().unwrap().handle();
        let l = place_alternative_pair(p, offer, Some(alt.clone()));
        let first = l
            .source_range_fragments(&origins[1], Range::new(0, 1))
            .unwrap();
        assert!(first.units().all(|u| u.alternatives().next().is_none()));
        near(rectangles(&first).iter().map(|r| r.size.width).sum(), 0.0);
        let defining = l
            .source_range_fragments(&origins[2], Range::new(0, 1))
            .unwrap();
        assert_eq!(
            defining.units().count(),
            usize::from(!replacement.is_empty())
        );
        if let Some(unit) = defining.units().next() {
            assert_eq!(unit.alternatives().next(), Some(alt));
            assert_eq!(
                unit.defining_sources().next().unwrap().aggregate_range(),
                Range::new(3, 4)
            );
            assert_eq!(
                unit.selected_intersections().next().unwrap().local_range(),
                Range::new(0, 1)
            );
            near(
                unit.pieces().map(|p| p.rect().size.width).sum(),
                if replacement == "-" { 10.0 } else { 0.0 },
            );
            assert_eq!(unit.presentation_range().len(), 1);
            assert!(unit.pieces().all(|piece| piece.rect().size.height == 10.0));
            assert!(!unit.same_unit(&first.units().next().unwrap()));
        }
        assert_eq!(
            l.analyzed_source().source().text().code_units(),
            [65, 88, 0xad, 0xad, 66, 66, 66]
        );
        let text: Vec<_> = l
            .inner
            .prepared
            .analyzed
            .processed
            .plaintext_items(Affinity::Before)
            .flat_map(|item| match item {
                PlaintextItem::Text(t) => t.code_units().to_vec(),
                PlaintextItem::InlineBox(_) => vec![],
            })
            .collect();
        assert_eq!(text, [65, 88, 0xad, 0xad, 66, 66, 66]);
    }
    let mut capture = input(
        Source::new("A\u{ad}B"),
        style(10.0),
        &[0, 3],
        ParagraphBaseDirection::LeftToRight,
    );
    capture.replacements.push(
        ParagraphReplacement::resolved_native_logical_before(
            ParagraphReplacementDefinition::Source {
                origin: 0,
                source: Range::new(1, 2),
            },
            2,
            vec![Text::from("-")],
        )
        .unwrap(),
    );
    let p = prepare(capture, AHEM.to_vec());
    let o = p.origin_views().next().unwrap().handle();
    let l = place(p, 0.0, 30.0);
    let q = l.source_range_fragments(&o, Range::new(1, 2)).unwrap();
    assert!(q.units().all(|u| u.alternatives().next().is_none()));
    near(rectangles(&q).iter().map(|r| r.size.width).sum(), 0.0);
}
#[test]
fn source_less_breaks_and_control_alternatives_with_incidental_origins_do_not_acquire_text_geometry()
 {
    for origin in [None, Some(0)] {
        let mut capture = input(
            Source::new("AXBBB"),
            style(10.0),
            &[0, 5],
            ParagraphBaseDirection::LeftToRight,
        );
        capture.controls.push(ParagraphControl {
            at: 2,
            kind: ParagraphControlKind::DiscretionaryBreak {
                style: Box::new(capture.base_style.clone()),
            },
        });
        capture.anchor_order.push(ParagraphAnchor {
            item: ParagraphAnchorItem::Control(0),
            affinity: Affinity::Before,
        });
        capture.replacements.push(
            ParagraphReplacement::resolved_native_logical_before(
                ParagraphReplacementDefinition::Control { control: 0, origin },
                2,
                vec![Text::from("-")],
            )
            .unwrap(),
        );
        let p = prepare(capture, AHEM.to_vec());
        let o = p.origin_views().next().unwrap().handle();
        let offer = p
            .opportunity_views()
            .find(|v| v.alternatives().next().is_some())
            .unwrap()
            .handle();
        let alt = offer.view().alternatives().next().unwrap().handle();
        let l = place_alternative_pair(p, offer, Some(alt));
        let q = l.source_range_fragments(&o, Range::new(0, 5)).unwrap();
        assert_eq!(q.units().count(), 5);
        assert!(
            q.units()
                .all(|u| u.alternatives().next().is_none() && u.defining_sources().next().is_none())
        );
        near(rectangles(&q).iter().map(|r| r.size.width).sum(), 50.0);
        let endpoint = l.source_range_fragments(&o, Range::new(0, 1)).unwrap();
        assert_eq!(endpoint.units().count(), 1);
        near(rectangles(&endpoint)[0].size.width, 10.0);
    }
    let mut capture = input(
        Source::new("AB"),
        style(10.0),
        &[0, 2],
        ParagraphBaseDirection::LeftToRight,
    );
    capture.controls.push(ParagraphControl {
        at: 1,
        kind: ParagraphControlKind::ForcedBreak,
    });
    capture.anchor_order.push(ParagraphAnchor {
        item: ParagraphAnchorItem::Control(0),
        affinity: Affinity::Before,
    });
    let p = prepare(capture, AHEM.to_vec());
    let o = p.origin_views().next().unwrap().handle();
    let l = place(p, 0.0, 30.0);
    let q = l.source_range_fragments(&o, Range::new(0, 2)).unwrap();
    assert_eq!(q.units().count(), 2);
    assert_eq!(
        q.units().map(|u| u.line_index()).collect::<Vec<_>>(),
        [0, 1]
    );
    near(rectangles(&q).iter().map(|r| r.size.width).sum(), 20.0);
}

#[test]
fn range_egc_shares_keep_owned_spacing_and_rtl_physical_sides() {
    let fixtures = [
        (
            "A A",
            vec![35, 3],
            vec![(0x41, 35), (0x20, 3)],
            ParagraphBaseDirection::LeftToRight,
        ),
        (
            "\u{5d0}\u{5d0}",
            vec![35, 35],
            vec![(0x5d0, 35)],
            ParagraphBaseDirection::RightToLeft,
        ),
        (
            "A\u{301}A",
            vec![35, 100, 35],
            vec![(0x41, 35), (0x301, 100)],
            ParagraphBaseDirection::LeftToRight,
        ),
    ];
    for (text, components, mappings, direction) in fixtures {
        let mut own_style = style(12.5);
        let spaced = text == "A A";
        if spaced {
            own_style.letter_spacing = 1.0;
            own_style.word_spacing = 3.0;
        }
        let p = prepare(
            input(
                Source::new(text),
                own_style,
                &[0, text.encode_utf16().count()],
                direction,
            ),
            crate::shaping_test_fonts::range_ligature_ahem(&components, &mappings),
        );
        let o = p.origin_views().next().unwrap().handle();
        let l = place(p, 0.0, 30.0);
        let first_end = if text == "A\u{301}A" { 2 } else { 1 };
        let a = l.source_range_fragments(&o, Range::new(0, 1)).unwrap();
        let b = l
            .source_range_fragments(&o, Range::new(first_end, first_end + 1))
            .unwrap();
        let ua = a.units().next().unwrap();
        let ub = b.units().next().unwrap();
        assert!(!ua.same_unit(&ub));
        let ra = ua.pieces().next().unwrap().rect();
        let rb = ub.pieces().next().unwrap().rect();
        near(ra.size.width, if spaced { 7.25 } else { 6.25 });
        near(rb.size.width, if spaced { 10.25 } else { 6.25 });
        if direction == ParagraphBaseDirection::RightToLeft {
            assert!(ra.origin.x > rb.origin.x);
        } else {
            near(rb.origin.x, ra.origin.x + ra.size.width);
        }
        if first_end == 2 {
            let mark = l.source_range_fragments(&o, Range::new(1, 2)).unwrap();
            assert!(ua.same_unit(&mark.units().next().unwrap()));
            assert_eq!(
                ua.expanded_intersections()
                    .map(|i| i.aggregate_range())
                    .collect::<Vec<_>>(),
                [Range::new(0, 1), Range::new(1, 2)]
            );
        }
    }
}

#[test]
fn nonmonotone_provider_partition_is_a_local_capability_refusal() {
    let mut own_style = style(10.0);
    own_style.letter_spacing = -6.0;
    let p = prepare(
        input(
            Source::new("AAB"),
            own_style,
            &[0, 3],
            ParagraphBaseDirection::LeftToRight,
        ),
        crate::shaping_test_fonts::ligature_ahem(),
    );
    let o = p.origin_views().next().unwrap().handle();
    let l = place(p, 0.0, 30.0);
    let before = l.source_range_fragments(&o, Range::new(2, 3)).unwrap();
    assert_eq!(before.units().count(), 1);
    let error = l.source_range_fragments(&o, Range::new(0, 1)).unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::SourceRangeFragments {
            reason: SourceRangeFragmentError::UnsupportedPartition { line: 0, unit: 0 }
        })
    ));
    assert_eq!(
        l.source_range_fragments(&o, Range::new(2, 3))
            .unwrap()
            .units()
            .count(),
        1
    );
}

#[test]
fn optional_ligature_keeps_selected_space_allocation_on_its_own_range_unit() {
    let mut own_style = style(12.5);
    own_style.letter_spacing = 1.0;
    own_style.word_spacing = 3.0;
    let p = prepare(
        input(
            Source::new("A A"),
            own_style,
            &[0, 3],
            ParagraphBaseDirection::LeftToRight,
        ),
        crate::shaping_test_fonts::range_ligature_ahem(&[35, 3], &[(0x41, 35), (0x20, 3)]),
    );
    let o = p.origin_views().next().unwrap().handle();
    let end = p
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let first = select(
        &p,
        p.boundary_views().next().unwrap().handle(),
        end,
        None,
        None,
    );
    let mut request = first.inner.selected.request.clone();
    request.context.spacing = SpacingPolicy::Spaces;
    let selected = p.select(request).unwrap();
    let allocations = selected
        .spacing_slots()
        .map(|slot| SpacingAllocation { slot, amount: 4.0 })
        .collect();
    let line = selected
        .finalize(SpacingAllocations::Complete(allocations))
        .unwrap();
    let l = Layout::from_finalized_lines(PlacementRequest {
        prepared: p,
        coverage: PlacementCoverage::WholeParagraph,
        lines: vec![LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 30.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        }],
    })
    .unwrap();
    let a = l.source_range_fragments(&o, Range::new(0, 1)).unwrap();
    let space = l.source_range_fragments(&o, Range::new(1, 2)).unwrap();
    near(rectangles(&a)[0].size.width, 7.25);
    near(rectangles(&space)[0].size.width, 14.25);
    near(rectangles(&space)[0].origin.x, 7.25);
}
