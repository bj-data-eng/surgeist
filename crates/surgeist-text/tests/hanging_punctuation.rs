#![forbid(unsafe_code)]

//! Independent public native punctuation expectations from pinned Text4 §9.2,
//! Appendix E and the tracked native attribution/union determination. These
//! tests preserve real glyph/source/caret custody.

use surgeist_text as t;
mod hanging_punctuation {
    pub mod font;
}

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");
const KERN: &[u8] = include_bytes!("fixtures/fonts/kern-ahem/KernAhem.ttf");
const LTR: t::ParagraphBaseDirection = t::ParagraphBaseDirection::LeftToRight;
const BOX_LTR: t::LineBoxDirection = t::LineBoxDirection::LeftToRight;
const BOX_RTL: t::LineBoxDirection = t::LineBoxDirection::RightToLeft;
type Measures = (f64, f64, f64, f64);

fn punctuation(
    first: bool,
    end: Option<t::HangingPunctuationEnd>,
    last: bool,
) -> t::HangingPunctuation {
    t::HangingPunctuation::new(first, end, last)
}
fn style(value: t::HangingPunctuation) -> t::Style {
    t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        white_space: t::WhiteSpace::Preserve,
        wrap: t::Wrap::Word,
        hanging_punctuation: value,
        ..t::Style::default()
    }
}
fn input(value: &str, base: t::Style) -> t::ParagraphInput {
    let source = t::Source::identified(value, t::Id::from_u64(71072), 4);
    let len = source.text().len();
    t::ParagraphInput {
        source: source.clone(),
        base_style: base,
        segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: LTR,
        paragraph_strut: t::ParagraphStrut::DefaultFont,
        terminal_edge: t::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![t::ParagraphOrigin {
            text_node: None,
            source,
            local: t::Range::new(0, len),
            aggregate: t::Range::new(0, len),
            kind: t::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}
fn prepare(value: t::ParagraphInput, bytes: &[u8]) -> t::PreparedText {
    let wrap = value.base_style.wrap;
    let source = value.source.clone();
    let analyzed =
        t::AnalyzedSource::from_paragraph(value, t::AnalysisTypography::HorizontalComposition)
            .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| t::WrappingPolicy {
            relation: r.handle(),
            ancestor_wrap: wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    let p = system
        .prepare(&analyzed, t::PreparationOptions::default())
        .unwrap();
    assert_eq!(p.analyzed_source().source(), &source);
    p
}
fn request(
    p: &t::PreparedText,
    width: f64,
    first: Option<bool>,
    last: Option<bool>,
) -> t::LineSelectionRequest {
    let end = p
        .opportunity_views()
        .find(|o| o.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let participants = p.participants().collect::<Vec<_>>();
    let controls = participants
        .iter()
        .filter_map(|part| match part {
            t::PreparedParticipant::Control(c) => Some(t::ControlAllocation {
                occurrence: c.clone(),
                strut: None,
                boundary: matches!(
                    c.view().kind(),
                    t::ParagraphControlKind::InlineBoundaryStart
                        | t::ParagraphControlKind::InlineBoundaryEnd
                )
                .then_some(t::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
            }),
            _ => None,
        })
        .collect();
    t::LineSelectionRequest {
        segment: p.source_segments().next().unwrap(),
        start: p.boundary_views().next().unwrap().handle(),
        end: end.view().boundary(),
        termination: t::PreparedTermination::Ordinary(end),
        participants,
        alternative: None,
        incoming: None,
        atomics: vec![],
        controls,
        boundary_context: vec![],
        line_box_struts: Vec::new(),
        line_strut: None,
        context: t::LineContext {
            alignment_insets: surgeist_text::LineAlignmentInsets::ZERO,
            base_direction: if p.analyzed_source().direction() == t::Direction::RightToLeft {
                t::ParagraphBaseDirection::RightToLeft
            } else {
                LTR
            },
            line_box_direction: BOX_LTR,
            formatted_line: t::FormattedLineContext::new(first, last),
            orientation: t::LineOrientation::Horizontal,
            start: t::LineStart::Paragraph,
            end: t::LineEnd::Paragraph,
            available_inline: Some(width),
            indent: 0.0,
            alignment: t::Alignment::Start,
            tabs: t::TabContext::NoTabs,
            spacing: t::SpacingPolicy::None,
        },
    }
}
fn finish(p: &t::PreparedText, r: t::LineSelectionRequest) -> t::Result<t::FinalizedLine> {
    p.select(r)?.finalize(surgeist_text::LineAllocations {
        spacing: t::SpacingAllocations::None,
        scopes: surgeist_text::ScopeAllocations::None,
    })
}
fn measures(line: &t::FinalizedLine) -> Measures {
    let m = line.measurements();
    (
        m.fit().advance(),
        m.alignment().advance(),
        m.min_content_contribution().advance(),
        m.max_content_contribution().advance(),
    )
}
fn usage(line: &t::FinalizedLine) -> (bool, bool) {
    let u = line.measurements().formatted_line_usage();
    (u.first(), u.last())
}
fn glyphs(line: &t::FinalizedLine, bytes: &[u8]) -> Vec<(t::Range, u32, f32, f32, f64, f64)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.font_size(), 10.0);
            let instance = run.effective_instance().unwrap();
            assert_eq!(instance.font_size(), 10.0);
            run.glyphs()
                .map(|g| {
                    (
                        g.range(),
                        g.id(),
                        g.advance(),
                        g.x(),
                        g.pen(),
                        run.origin().x + g.pen() + f64::from(g.x()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
fn placed(p: &t::PreparedText, line: t::FinalizedLine) -> t::Layout {
    let r = line.selected_line();
    let r = r.request();
    t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: p.clone(),
        coverage: t::PlacementCoverage::Range {
            start: r.start.clone(),
            end: r.end.clone(),
        },
        entries: vec![t::PlacementEntry::Formatted(t::LinePlacement {
            line,
            anchor: t::LineAnchor {
                inline_origin: 0.0,
                baseline: 8.0,
            },
            orientation: t::LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap()
}
fn carets(layout: &t::Layout, expected: &[f64]) {
    for (index, x) in expected.iter().copied().enumerate() {
        for affinity in [t::Affinity::Before, t::Affinity::After] {
            assert_eq!(
                layout
                    .cursor(t::Cursor::new(index, affinity))
                    .unwrap()
                    .rect()
                    .origin
                    .x,
                x
            );
        }
    }
}
fn demand(error: &t::Error, expected: t::PhaseTwoContext) {
    assert!(
        matches!(error.detail(), Some(t::ErrorDetail::InvalidTextProtocol { reason:
        t::ProtocolError::MissingPhaseTwoContext(actual) }) if *actual == expected),
        "{error:?}"
    );
}
fn unit_ranges(line: &t::FinalizedLine) -> Vec<(usize, usize)> {
    line.edge_units()
        .filter_map(|e| e.punctuation_unit().map(|r| (r.start(), r.end())))
        .collect()
}

#[test]
fn force_and_allow_measurements_retain_actual_glyphs_source_and_carets() {
    use t::HangingPunctuationEnd::{AllowEnd, ForceEnd};
    for (end, width, expected) in [
        (None, 25.0, (20.0, 20.0, 20.0, 20.0)),
        (None, 15.0, (20.0, 20.0, 20.0, 20.0)),
        (Some(ForceEnd), 25.0, (10.0, 10.0, 10.0, 10.0)),
        (Some(ForceEnd), 15.0, (10.0, 10.0, 10.0, 10.0)),
        (Some(AllowEnd), 25.0, (10.0, 20.0, 10.0, 20.0)),
        (Some(AllowEnd), 20.0, (10.0, 20.0, 10.0, 20.0)),
        (Some(AllowEnd), 15.0, (10.0, 15.0, 10.0, 20.0)),
        (Some(AllowEnd), 10.0, (10.0, 10.0, 10.0, 20.0)),
        (Some(AllowEnd), 5.0, (10.0, 10.0, 10.0, 20.0)),
    ] {
        let p = prepare(input("A.", style(punctuation(false, end, false))), AHEM);
        let line = finish(&p, request(&p, width, None, None)).unwrap();
        assert_eq!(line.feedback().advance, 20.0);
        assert_eq!(measures(&line), expected);
        assert_eq!(usage(&line), (false, false));
        assert_eq!(
            glyphs(&line, AHEM),
            [
                (t::Range::new(0, 1), 35, 10.0, 0.0, 0.0, 0.0),
                (t::Range::new(1, 2), 16, 10.0, 0.0, 10.0, 10.0)
            ]
        );
        if end.is_some() {
            assert_eq!(unit_ranges(&line), [(1, 2)]);
        }
        let layout = placed(&p, line);
        assert_eq!(layout.source().text().code_units(), [65, 46]);
        carets(&layout, &[0.0, 10.0, 20.0]);
        let a = layout
            .positions()
            .find(|p| p.source_cursor() == t::Cursor::new(0, t::Affinity::After))
            .unwrap();
        let b = layout
            .positions()
            .find(|p| p.source_cursor() == t::Cursor::new(2, t::Affinity::Before))
            .unwrap();
        let selected = layout.position_selection(&a, &b).unwrap();
        assert_eq!(selected.source_text(), &t::Text::from("A."));
        assert_eq!(selected.plaintext(), &t::Text::from("A."));
        #[cfg(feature = "text-accessibility")]
        {
            let mut state = t::Accessibility::new(200);
            let parent = accesskit::NodeId(100);
            let update = layout
                .accessibility_update(&mut state, parent, parent, t::Point::default())
                .unwrap();
            let nodes = update
                .nodes
                .iter()
                .filter(|(_, n)| n.role() == accesskit::Role::TextRun)
                .collect::<Vec<_>>();
            assert_eq!(
                nodes
                    .iter()
                    .filter_map(|(_, n)| n.value())
                    .collect::<String>(),
                "A."
            );
            assert_eq!(
                nodes
                    .iter()
                    .flat_map(|(_, n)| n.character_lengths().iter().copied())
                    .collect::<Vec<_>>(),
                [1, 1]
            );
            assert_eq!(
                nodes
                    .iter()
                    .flat_map(|(_, n)| n.character_widths().unwrap().iter().copied())
                    .collect::<Vec<_>>(),
                [10.0, 10.0]
            );
            assert_eq!(
                nodes
                    .iter()
                    .map(|(_, n)| n.bounds().unwrap().x0)
                    .collect::<Vec<_>>(),
                [0.0, 10.0]
            );
            let anchor = layout
                .access_position(&state, t::Cursor::new(0, t::Affinity::After))
                .unwrap()
                .unwrap();
            let focus = layout
                .access_position(&state, t::Cursor::new(2, t::Affinity::Before))
                .unwrap()
                .unwrap();
            let selection = layout
                .access_selection(
                    &state,
                    t::Selection::new(
                        t::Cursor::new(0, t::Affinity::After),
                        t::Cursor::new(2, t::Affinity::Before),
                    ),
                )
                .unwrap()
                .unwrap();
            assert_eq!(selection.anchor, anchor);
            assert_eq!(selection.focus, focus);
        }
    }
}

#[test]
fn first_last_lazy_unknown_false_true_and_zero_advance_are_semantic() {
    use skrifa::MetadataProvider;
    let zero_font = hanging_punctuation::font::zero_advance_parentheses();
    let face = skrifa::FontRef::new(&zero_font).unwrap();
    for (character, glyph) in [('(', 10), (')', 11), ('A', 35)] {
        assert_eq!(face.charmap().map(character).unwrap().to_u32(), glyph);
    }
    for tag in [*b"cmap", *b"glyf", *b"loca"] {
        assert_eq!(
            hanging_punctuation::font::table(&zero_font, tag),
            hanging_punctuation::font::table(AHEM, tag)
        );
    }
    let head = hanging_punctuation::font::table(&zero_font, *b"head");
    assert_eq!(u16::from_be_bytes(head[18..20].try_into().unwrap()), 1000);
    let hmtx = hanging_punctuation::font::table(&zero_font, *b"hmtx");
    let original_hmtx = hanging_punctuation::font::table(AHEM, *b"hmtx");
    for (glyph, advance) in [(10, 0), (11, 0), (35, 1000)] {
        assert_eq!(
            u16::from_be_bytes(hmtx[glyph * 4..glyph * 4 + 2].try_into().unwrap()),
            advance
        );
        assert_eq!(
            hmtx[glyph * 4 + 2..glyph * 4 + 4],
            original_hmtx[glyph * 4 + 2..glyph * 4 + 4]
        );
    }
    for (text, first, role) in [
        ("(A", true, t::PhaseTwoContext::FirstFormattedLine),
        ("A)", false, t::PhaseTwoContext::LastFormattedLine),
    ] {
        for zero in [false, true] {
            // Zero advance comes from the actual punctuation font metric,
            // independently of tracking at its adjacent character boundary.
            let bytes = if zero { zero_font.as_slice() } else { AHEM };
            let value = input(text, style(punctuation(first, None, !first)));
            let p = prepare(value, bytes);
            demand(
                &finish(&p, request(&p, 40.0, None, None)).unwrap_err(),
                role,
            );
            for yes in [false, true] {
                let line = finish(
                    &p,
                    request(&p, 40.0, first.then_some(yes), (!first).then_some(yes)),
                )
                .unwrap();
                let raw = if zero { 10.0 } else { 20.0 };
                let measured = if yes { 10.0 } else { raw };
                assert_eq!(line.feedback().advance, raw);
                assert_eq!(measures(&line), (measured, measured, measured, measured));
                assert_eq!(usage(&line), (first, !first));
                let punctuation_advance: f32 = if zero { 0.0 } else { 10.0 };
                let start = if first && yes {
                    -f64::from(punctuation_advance)
                } else {
                    0.0
                };
                assert_eq!(
                    glyphs(&line, bytes),
                    if first {
                        [
                            (
                                t::Range::new(0, 1),
                                10,
                                punctuation_advance,
                                0.0,
                                0.0,
                                start,
                            ),
                            (
                                t::Range::new(1, 2),
                                35,
                                10.0,
                                0.0,
                                f64::from(punctuation_advance),
                                start + f64::from(punctuation_advance),
                            ),
                        ]
                    } else {
                        [
                            (t::Range::new(0, 1), 35, 10.0, 0.0, 0.0, 0.0),
                            (
                                t::Range::new(1, 2),
                                11,
                                punctuation_advance,
                                0.0,
                                10.0,
                                10.0,
                            ),
                        ]
                    }
                );
                if zero {
                    for run in line.run_views() {
                        let instance = run.effective_instance().unwrap();
                        for glyph in run.glyphs().filter(|g| matches!(g.id(), 10 | 11)) {
                            let bounds = instance
                                .glyph_outline(glyph.id(), run.face().catalogue_identity())
                                .unwrap()
                                .bounds()
                                .expect("zero-advance punctuation retains real ink");
                            assert!(bounds.size.width > 0.0 && bounds.size.height > 0.0);
                        }
                    }
                }
                if first && yes && !zero {
                    assert_eq!(
                        glyphs(&line, bytes).iter().map(|g| g.5).collect::<Vec<_>>(),
                        [-10.0, 0.0]
                    );
                    carets(&placed(&p, line), &[-10.0, 0.0, 10.0]);
                } else if zero {
                    let layout = placed(&p, line);
                    assert_eq!(layout.source().text(), &t::Text::from(text));
                    carets(
                        &layout,
                        if first {
                            &[0.0, 0.0, 10.0]
                        } else {
                            &[0.0, 10.0, 10.0]
                        },
                    );
                }
            }
        }
    }
    let p = prepare(
        input(
            "AA",
            style(punctuation(
                true,
                Some(t::HangingPunctuationEnd::ForceEnd),
                true,
            )),
        ),
        AHEM,
    );
    let line = finish(&p, request(&p, 40.0, None, None)).unwrap();
    assert_eq!(measures(&line), (20.0, 20.0, 20.0, 20.0));
    assert_eq!(usage(&line), (false, false));
}

#[test]
fn last_is_demanded_when_only_min_content_exposes_the_quote() {
    let p = prepare(input("A\" ", style(punctuation(false, None, true))), AHEM);
    demand(
        &finish(&p, request(&p, 35.0, None, None)).unwrap_err(),
        t::PhaseTwoContext::LastFormattedLine,
    );
    for (last, min) in [(false, 20.0), (true, 10.0)] {
        let line = finish(&p, request(&p, 35.0, None, Some(last))).unwrap();
        assert_eq!(line.feedback().advance, 30.0);
        assert_eq!(measures(&line), (20.0, 30.0, min, 30.0));
        assert_eq!(usage(&line), (false, true));
    }
}

#[test]
fn same_authentic_quote_unions_once_with_start_priority_and_distinct_quotes_stay_distinct() {
    let p = prepare(input("\"", style(punctuation(true, None, true))), AHEM);
    for (direction, x) in [(BOX_LTR, -10.0), (BOX_RTL, 20.0)] {
        let mut r = request(&p, 20.0, Some(true), None);
        r.context.line_box_direction = direction;
        let line = finish(&p, r).unwrap();
        assert_eq!(line.feedback().advance, 10.0);
        assert_eq!(measures(&line), (0.0, 0.0, 0.0, 0.0));
        assert_eq!(usage(&line), (true, false));
        assert_eq!(unit_ranges(&line), [(0, 1)]);
        assert_eq!(glyphs(&line, AHEM).len(), 1);
        assert_eq!(glyphs(&line, AHEM)[0].5, x);
        carets(&placed(&p, line), &[x, x + 10.0]);
    }
    let p = prepare(input("\"\"", style(punctuation(true, None, true))), AHEM);
    let line = finish(&p, request(&p, 20.0, Some(true), Some(true))).unwrap();
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(measures(&line), (0.0, 0.0, 0.0, 0.0));
    assert_eq!(
        glyphs(&line, AHEM).iter().map(|g| g.5).collect::<Vec<_>>(),
        [-10.0, 0.0]
    );
    assert_eq!(unit_ranges(&line), [(0, 1), (1, 2)]);
    carets(&placed(&p, line), &[-10.0, 0.0, 10.0]);
}

#[test]
fn one_punctuation_tcu_per_edge_and_opposed_frame_use_actual_visual_edges() {
    let p = prepare(
        input(
            "A.,",
            style(punctuation(
                false,
                Some(t::HangingPunctuationEnd::ForceEnd),
                false,
            )),
        ),
        AHEM,
    );
    let line = finish(&p, request(&p, 40.0, None, None)).unwrap();
    assert_eq!(line.feedback().advance, 30.0);
    assert_eq!(measures(&line), (20.0, 20.0, 20.0, 20.0));
    assert_eq!(unit_ranges(&line), [(2, 3)]);
    let p = prepare(input("((A", style(punctuation(true, None, false))), AHEM);
    let line = finish(&p, request(&p, 40.0, Some(true), None)).unwrap();
    assert_eq!(measures(&line), (20.0, 20.0, 20.0, 20.0));
    assert_eq!(unit_ranges(&line), [(0, 1)]);
    assert_eq!(
        glyphs(&line, AHEM).iter().map(|g| g.5).collect::<Vec<_>>(),
        [-10.0, 0.0, 10.0]
    );
    let p = prepare(input("A(", style(punctuation(true, None, false))), AHEM);
    let mut r = request(&p, 20.0, Some(true), None);
    r.context.line_box_direction = BOX_RTL;
    let line = finish(&p, r).unwrap();
    assert_eq!(measures(&line), (10.0, 10.0, 10.0, 10.0));
    assert_eq!(unit_ranges(&line), [(1, 2)]);
    assert_eq!(
        glyphs(&line, AHEM).iter().map(|g| g.5).collect::<Vec<_>>(),
        [10.0, 20.0]
    );
    carets(&placed(&p, line), &[10.0, 20.0, 30.0]);
}

#[test]
fn compound_unconditional_period_and_preserved_space_have_per_measure_terminal_chains() {
    let base = style(punctuation(
        false,
        Some(t::HangingPunctuationEnd::ForceEnd),
        false,
    ));
    let p = prepare(input("A. A", base.clone()), AHEM);
    let end = p
        .opportunity_views()
        .find(|o| {
            o.kind() == t::PreparedBreakKind::Allowed
                && o.boundary().view().source_after().get() == 3
        })
        .unwrap()
        .handle();
    let mut r = request(&p, 40.0, None, None);
    r.end = end.view().boundary();
    r.participants.truncate(r.end.view().participant_offset());
    r.termination = t::PreparedTermination::Ordinary(end);
    r.context.end = t::LineEnd::Soft;
    let line = finish(&p, r).unwrap();
    assert_eq!(line.feedback().advance, 30.0);
    assert_eq!(measures(&line), (10.0, 10.0, 10.0, 10.0));
    assert_eq!(
        glyphs(&line, AHEM).iter().map(|g| g.5).collect::<Vec<_>>(),
        [0.0, 10.0, 20.0]
    );
    let p = prepare(input("A. ", base), AHEM);
    for (width, expected) in [
        (35.0, (20.0, 30.0, 10.0, 30.0)),
        (25.0, (10.0, 15.0, 10.0, 30.0)),
    ] {
        let line = finish(&p, request(&p, width, None, None)).unwrap();
        assert_eq!(line.feedback().advance, 30.0);
        assert_eq!(measures(&line), expected);
        carets(&placed(&p, line), &[0.0, 10.0, 20.0, 30.0]);
    }
}

#[test]
fn actual_selected_inline_bp_blocks_only_its_edge_and_is_counted_once() {
    for (start_bp, end_bp, first, last, expected, x, used) in [
        (0.0, 0.0, Some(true), None, 0.0, -10.0, (true, false)),
        (5.0, 0.0, None, Some(true), 5.0, 5.0, (false, true)),
        (0.0, 5.0, Some(true), None, 5.0, -10.0, (true, false)),
        (5.0, 5.0, None, None, 20.0, 5.0, (false, false)),
    ] {
        let mut value = input("\"", style(punctuation(true, None, true)));
        value.controls = vec![
            t::ParagraphControl {
                at: 0,
                kind: t::ParagraphControlKind::InlineBoundaryStart,
            },
            t::ParagraphControl {
                at: 1,
                kind: t::ParagraphControlKind::InlineBoundaryEnd,
            },
        ];
        value.anchor_order = (0..2)
            .map(|i| t::ParagraphAnchor {
                item: t::ParagraphAnchorItem::Control(i),
                affinity: t::Affinity::After,
            })
            .collect();
        let p = prepare(value, AHEM);
        let mut r = request(&p, 40.0, first, last);
        for allocation in &mut r.controls {
            let bp = match allocation.occurrence.view().kind() {
                t::ParagraphControlKind::InlineBoundaryStart => start_bp,
                t::ParagraphControlKind::InlineBoundaryEnd => end_bp,
                _ => unreachable!(),
            };
            allocation.boundary = Some(t::InlineBoundaryAllocation::contribution(bp).unwrap());
        }
        let line = finish(&p, r).unwrap();
        assert_eq!(line.feedback().advance, 10.0 + start_bp + end_bp);
        assert_eq!(measures(&line), (expected, expected, expected, expected));
        assert_eq!(usage(&line), used);
        assert_eq!(glyphs(&line, AHEM).len(), 1);
        assert_eq!(glyphs(&line, AHEM)[0].5, x);
        let slots = line.control_slots().collect::<Vec<_>>();
        assert_eq!(slots.len(), 2);
        assert_eq!(
            slots[0].strip_interval().1 - slots[0].strip_interval().0,
            start_bp
        );
        assert_eq!(
            slots[1].strip_interval().1 - slots[1].strip_interval().0,
            end_bp
        );
    }
}

#[test]
fn incoming_offselected_joining_context_is_not_used_punctuation_padding() {
    use skrifa::MetadataProvider;
    let bytes = hanging_punctuation::font::punctuation_ligature();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map(')').unwrap().to_u32(), 11);
    assert_eq!(font.charmap().map('一').unwrap().to_u32(), 36);
    let mut value = input(
        ")一",
        t::Style {
            word_break: t::WordBreak::BreakAll,
            ..style(punctuation(false, None, true))
        },
    );
    value.controls = vec![
        t::ParagraphControl {
            at: 1,
            kind: t::ParagraphControlKind::InlineBoundaryStart,
        },
        t::ParagraphControl {
            at: 2,
            kind: t::ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    value.anchor_order = (0..2)
        .map(|i| t::ParagraphAnchor {
            item: t::ParagraphAnchorItem::Control(i),
            affinity: t::Affinity::Before,
        })
        .collect();
    let p = prepare(value, &bytes);
    let end = p
        .opportunity_views()
        .find(|o| {
            o.kind() == t::PreparedBreakKind::Allowed
                && o.boundary().view().participant_offset() == 1
        })
        .unwrap_or_else(|| {
            panic!(
                "actual offers: {:?}",
                p.opportunity_views()
                    .map(|o| (
                        o.kind(),
                        o.boundary().view().source_before().get(),
                        o.boundary().view().source_after().get(),
                        o.boundary().view().participant_offset()
                    ))
                    .collect::<Vec<_>>()
            )
        })
        .handle();
    let mut r = request(&p, 40.0, None, None);
    r.end = end.view().boundary();
    r.termination = t::PreparedTermination::Ordinary(end);
    r.participants.truncate(1);
    r.controls.clear();
    r.context.end = t::LineEnd::Soft;
    let first = p.boundary_context_demand(&r).unwrap().unwrap();
    assert_eq!(
        *first.view().kind(),
        t::ParagraphControlKind::InlineBoundaryStart
    );
    assert_eq!(first.view().source_position().get(), 1);
    // The demanded Start is outside selected source0..1. Joining context
    // supplies no used decoration on this selected fragment's physical edge.
    assert!(p.select(r.clone()).is_err());
    r.boundary_context.push(t::InlineBoundaryContextAllocation {
        occurrence: first.clone(),
        boundary: t::InlineBoundaryAllocation::contribution(5.0).unwrap(),
    });
    assert!(p.boundary_context_demand(&r).unwrap().is_none());
    r.context.formatted_line = t::FormattedLineContext::new(None, Some(true));
    let positive = finish(&p, r.clone()).unwrap();
    assert_eq!(positive.feedback().advance, 10.0);
    assert_eq!(measures(&positive), (0.0, 0.0, 0.0, 0.0));
    assert_eq!(usage(&positive), (false, true));
    assert!(positive.control_slots().next().is_none());
    assert_eq!(glyphs(&positive, &bytes)[0].5, 0.0);
    r.context.formatted_line = t::FormattedLineContext::default();
    demand(
        &finish(&p, r.clone()).unwrap_err(),
        t::PhaseTwoContext::LastFormattedLine,
    );
    r.boundary_context[0].boundary = t::InlineBoundaryAllocation::CONTRIBUTION_ZERO;
    let farther = p.boundary_context_demand(&r).unwrap().unwrap();
    assert_ne!(farther, first);
    assert_eq!(
        *farther.view().kind(),
        t::ParagraphControlKind::InlineBoundaryEnd
    );
    r.boundary_context.push(t::InlineBoundaryContextAllocation {
        occurrence: farther,
        boundary: t::InlineBoundaryAllocation::CONTRIBUTION_ZERO,
    });
    assert!(p.boundary_context_demand(&r).unwrap().is_none());
    demand(
        &finish(&p, r.clone()).unwrap_err(),
        t::PhaseTwoContext::LastFormattedLine,
    );
    r.context.formatted_line = t::FormattedLineContext::new(None, Some(true));
    let open = finish(&p, r).unwrap();
    assert_eq!(open.feedback().advance, 10.0);
    assert_eq!(measures(&open), (0.0, 0.0, 0.0, 0.0));
    assert_eq!(usage(&open), (false, true));
    assert_eq!(unit_ranges(&open), [(0, 1)]);
    assert!(open.control_slots().next().is_none());
    assert_eq!(positive.feedback().advance, 10.0);
}

#[test]
fn authored_declarations_and_resolved_role_identity_are_retained() {
    use std::hash::{Hash, Hasher};
    fn hash(value: &impl Hash) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut h);
        h.finish()
    }
    let declarations = [
        t::HangingPunctuation::default(),
        punctuation(true, None, false),
        punctuation(false, Some(t::HangingPunctuationEnd::ForceEnd), false),
        punctuation(false, Some(t::HangingPunctuationEnd::AllowEnd), false),
        punctuation(false, None, true),
        punctuation(true, Some(t::HangingPunctuationEnd::AllowEnd), true),
    ];
    for (i, decl) in declarations.iter().copied().enumerate() {
        let p = prepare(input("A.", style(decl)), AHEM);
        assert_eq!(p.analyzed_source().style().hanging_punctuation, decl);
        assert_eq!(
            p.analyzed_source().source().id(),
            Some(t::Id::from_u64(71072))
        );
        for other in &declarations[..i] {
            assert_ne!(decl, *other);
            assert_ne!(hash(&decl), hash(other));
        }
    }
    let p = prepare(input("(A", style(punctuation(true, None, false))), AHEM);
    let yes = finish(&p, request(&p, 40.0, Some(true), None)).unwrap();
    let yes_again = finish(&p, request(&p, 40.0, Some(true), None)).unwrap();
    let no = finish(&p, request(&p, 40.0, Some(false), None)).unwrap();
    assert_eq!(
        yes.selected_line().request().context.formatted_line,
        t::FormattedLineContext::new(Some(true), None)
    );
    assert_eq!(measures(&yes), measures(&yes_again));
    assert_ne!(measures(&yes), measures(&no));
    assert_eq!(yes.feedback().advance, 20.0);
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let source = t::Source::identified("A.", t::Id::from_u64(71073), 9);
    let options = t::Options {
        width: Some(15.0),
        ..t::Options::default()
    };
    let before = system.stats();
    let old = system
        .layout(
            source.clone(),
            style(t::HangingPunctuation::default()),
            options,
        )
        .unwrap();
    assert_eq!(system.stats().layout_misses(), before.layout_misses() + 1);
    let key = old.key();
    let before = system.stats();
    let force = style(punctuation(
        false,
        Some(t::HangingPunctuationEnd::ForceEnd),
        false,
    ));
    let changed = system
        .layout(source.clone(), force.clone(), options)
        .unwrap();
    assert_ne!(changed.key(), key);
    assert_eq!(system.stats().layout_misses(), before.layout_misses() + 1);
    let before = system.stats();
    let hit = system.layout(source.clone(), force, options).unwrap();
    assert_eq!(hit.key(), changed.key());
    assert_eq!(system.stats().layout_hits(), before.layout_hits() + 1);
    assert_eq!(system.stats().layout_misses(), before.layout_misses());
    for layout in [&old, &changed, &hit] {
        assert_eq!(layout.source(), &source);
        assert_eq!(layout.lines().len(), 1);
        carets(layout, &[0.0, 10.0, 20.0]);
    }
    assert_eq!(old.key(), key);
}

#[test]
fn literal_kerning_feature_and_parent_carets_qualify_actual_hanging_contributions() {
    for (kern, advance, raw, offset) in [(true, 9.0, 18.0, -1.0), (false, 10.0, 20.0, 0.0)] {
        let font = t::Font::new().family("Surgeist Kern Ahem");
        let font = if kern {
            font
        } else {
            font.feature(r#""kern" off"#)
        };
        for (end, width, alignment, max) in [
            (None, 25.0, raw, raw),
            (
                Some(t::HangingPunctuationEnd::ForceEnd),
                25.0,
                advance,
                advance,
            ),
            (Some(t::HangingPunctuationEnd::AllowEnd), 25.0, raw, raw),
            (Some(t::HangingPunctuationEnd::AllowEnd), 15.0, 15.0, raw),
        ] {
            let base = t::Style {
                font: font.clone(),
                ..style(punctuation(false, end, false))
            };
            let p = prepare(input("A.", base), KERN);
            let line = finish(&p, request(&p, width, None, None)).unwrap();
            let fit = if end.is_some() { advance } else { raw };
            assert_eq!(measures(&line), (fit, alignment, fit, max));
            assert_eq!(line.feedback().advance, raw);
            assert_eq!(
                glyphs(&line, KERN),
                [
                    (t::Range::new(0, 1), 35, advance as f32, 0.0, 0.0, 0.0),
                    (
                        t::Range::new(1, 2),
                        16,
                        advance as f32,
                        offset,
                        advance,
                        advance + f64::from(offset)
                    )
                ]
            );
            let units = p
                .unit_views()
                .map(|u| (u.source_range(), u.advance()))
                .collect::<Vec<_>>();
            assert_eq!(
                units,
                [
                    (t::Range::new(0, 1), advance),
                    (t::Range::new(1, 2), advance)
                ]
            );
            carets(&placed(&p, line), &[0.0, advance, raw]);
        }
    }
}

#[test]
fn source_built_shared_glyph_is_qualified_before_only_period_tcu_hangs() {
    use skrifa::MetadataProvider;
    let bytes = hanging_punctuation::font::punctuation_ligature();
    let f = skrifa::FontRef::new(&bytes).unwrap();
    for cp in ['A', '.', '(', '\u{301}'] {
        assert_eq!(f.charmap().map(cp).unwrap().to_u32(), 35);
    }
    let head = hanging_punctuation::font::table(&bytes, *b"head");
    assert_eq!(u16::from_be_bytes(head[18..20].try_into().unwrap()), 1000);
    let hmtx = hanging_punctuation::font::table(&bytes, *b"hmtx");
    for id in [35, 58] {
        assert_eq!(
            u16::from_be_bytes(hmtx[id * 4..id * 4 + 2].try_into().unwrap()),
            1000
        );
    }
    let mut original_carets = None;
    for end in [None, Some(t::HangingPunctuationEnd::ForceEnd)] {
        let p = prepare(input("A.", style(punctuation(false, end, false))), &bytes);
        assert_eq!(
            p.analyzed_source().graphemes().collect::<Vec<_>>(),
            [t::Range::new(0, 1), t::Range::new(1, 2)]
        );
        // The original glyph table gives one 10px X draw; the existing actual
        // provider contribution contract divides it across two eligible atoms.
        assert_eq!(
            p.unit_views()
                .map(|u| (u.source_range(), u.advance()))
                .collect::<Vec<_>>(),
            [(t::Range::new(0, 1), 5.0), (t::Range::new(1, 2), 5.0)]
        );
        let line = finish(&p, request(&p, 20.0, None, None)).unwrap();
        assert_eq!(
            glyphs(&line, &bytes),
            [(t::Range::new(0, 2), 58, 10.0, 0.0, 0.0, 0.0)]
        );
        assert_eq!(line.feedback().advance, 10.0);
        let m = if end.is_some() { 5.0 } else { 10.0 };
        assert_eq!(measures(&line), (m, m, m, m));
        let rows = line.edge_units().collect::<Vec<_>>();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].source_pieces().collect::<Vec<_>>(),
            [t::Range::new(0, 1)]
        );
        assert_eq!(
            rows[1].source_pieces().collect::<Vec<_>>(),
            [t::Range::new(1, 2)]
        );
        if end.is_some() {
            assert_eq!(rows[0].excluded_contribution(), 0.0);
            assert_eq!(rows[1].excluded_contribution(), 5.0);
            assert_eq!(unit_ranges(&line), [(1, 2)]);
        }
        let layout = placed(&p, line);
        let cursors = (0..=2)
            .flat_map(|index| {
                [t::Affinity::Before, t::Affinity::After]
                    .map(move |affinity| t::Cursor::new(index, affinity))
            })
            .collect::<Vec<_>>();
        let actual = cursors
            .iter()
            .map(|c| layout.cursor(*c).unwrap().rect().origin.x)
            .collect::<Vec<_>>();
        assert_eq!(actual[0], 0.0);
        assert_eq!(actual[1], 0.0);
        assert_eq!(actual[4], 10.0);
        assert_eq!(actual[5], 10.0);
        // Provider-cluster caret admission stays authoritative; attribution of
        // two contributions does not manufacture a ligature interior caret.
        if let Some(original) = &original_carets {
            assert_eq!(&actual, original);
        } else {
            original_carets = Some(actual);
        }
    }
}

#[test]
fn complete_marked_tcu_uses_effective_properties_and_base_owner_without_glyph_guessing() {
    use icu_properties::{
        CodePointMapData,
        props::{GeneralCategory as G, Script},
        script::ScriptWithExtensions,
    };
    assert_eq!(CodePointMapData::<G>::new().get32(0x20dd), G::EnclosingMark);
    assert_eq!(
        CodePointMapData::<Script>::new().get32(0x20dd),
        Script::Inherited
    );
    assert!(!ScriptWithExtensions::new().has_script32(0x20dd, Script::Common));
    let bytes = hanging_punctuation::font::punctuation_ligature();
    for (text, decl) in [
        ("(\u{301}A", punctuation(true, None, false)),
        ("\"\u{20dd}A", punctuation(true, None, false)),
        (
            "B.\u{20dd}",
            punctuation(false, Some(t::HangingPunctuationEnd::ForceEnd), false),
        ),
    ] {
        let p = prepare(input(text, style(decl)), &bytes);
        let none = prepare(input(text, style(t::HangingPunctuation::default())), &bytes);
        let baseline = finish(&none, request(&none, 40.0, None, None)).unwrap();
        assert_eq!(baseline.feedback().advance, 20.0);
        assert_eq!(
            glyphs(&baseline, &bytes).len(),
            2,
            "real two-input ccmp TCU plus ordinary A"
        );
        let line = finish(&p, request(&p, 40.0, decl.first().then_some(true), None)).unwrap();
        assert_eq!(line.feedback().advance, 20.0);
        assert_eq!(measures(&line), (10.0, 10.0, 10.0, 10.0));
        let target = if decl.first() { (0, 2) } else { (1, 3) };
        assert!(unit_ranges(&line).iter().all(|r| *r == target));
        assert!(!unit_ranges(&line).is_empty());
        assert_eq!(glyphs(&line, &bytes).len(), 2);
    }
    for base_first in [false, true] {
        let mut value = input("(\u{301}A", style(punctuation(base_first, None, false)));
        value.source.span(
            t::Range::new(1, 2),
            t::Style {
                brush: t::Brush::color(0.2, 0.4, 0.6, 1.0),
                ..style(punctuation(!base_first, None, false))
            },
        );
        value.origins = [(0, 1), (1, 2), (2, 3)]
            .into_iter()
            .enumerate()
            .map(|(i, (a, b))| t::ParagraphOrigin {
                text_node: None,
                source: t::Source::identified(
                    t::Text::from_code_units(value.source.text().code_units()[a..b].to_vec()),
                    t::Id::from_u64(72000 + i as u64),
                    3,
                ),
                local: t::Range::new(0, b - a),
                aggregate: t::Range::new(a, b),
                kind: t::ParagraphOriginKind::Authored,
            })
            .collect();
        value.connections = [1, 2]
            .into_iter()
            .map(|at| t::ParagraphConnection {
                at,
                kind: t::ParagraphConnectionKind::Continuous,
            })
            .collect();
        value.boundary_policies = (0..2)
            .map(|i| t::ParagraphBoundaryPolicy {
                definition: t::ParagraphBoundaryDefinition::Connection(i),
                preceding_wrap: t::Wrap::Word,
                following_wrap: t::Wrap::Word,
                ancestor_wrap: t::Wrap::Word,
            })
            .collect();
        let p = prepare(value, &bytes);
        let line = finish(&p, request(&p, 40.0, base_first.then_some(true), None)).unwrap();
        assert_eq!(usage(&line), (base_first, false));
        if base_first {
            assert!(!unit_ranges(&line).is_empty());
            assert!(unit_ranges(&line).iter().all(|r| *r == (0, 2)));
        } else {
            assert!(unit_ranges(&line).is_empty());
        }
        let origins = p.origin_views().map(|o| o.handle()).collect::<Vec<_>>();
        assert_eq!(origins.len(), 3);
        let layout = placed(&p, line);
        for (i, origin) in origins.iter().enumerate() {
            assert_eq!(
                origin.view().source().id(),
                Some(t::Id::from_u64(72000 + i as u64))
            );
            let pieces = layout
                .source_range_fragments(origin, t::Range::new(0, 1))
                .unwrap();
            assert!(!pieces.units().collect::<Vec<_>>().is_empty());
        }
        assert_eq!(layout.source().text().code_units(), [0x28, 0x301, 0x41]);
    }
}

#[test]
fn effective_space_mark_unit_cannot_recover_raw_ideographic_space_permission() {
    use icu_properties::{CodePointMapData, props::GeneralCategory as G};
    use skrifa::MetadataProvider;
    assert_eq!(
        CodePointMapData::<G>::new().get32(0x3000),
        G::SpaceSeparator
    );
    assert_eq!(CodePointMapData::<G>::new().get32(0x301), G::NonspacingMark);
    let bytes = hanging_punctuation::font::punctuation_ligature();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map('\u{3000}').unwrap().to_u32(), 3);
    assert_eq!(font.charmap().map('\u{301}').unwrap().to_u32(), 35);
    let p = prepare(
        input("\u{3000}A", style(punctuation(true, None, false))),
        &bytes,
    );
    demand(
        &finish(&p, request(&p, 40.0, None, None)).unwrap_err(),
        t::PhaseTwoContext::FirstFormattedLine,
    );
    let line = finish(&p, request(&p, 40.0, Some(true), None)).unwrap();
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(measures(&line), (10.0, 10.0, 10.0, 10.0));
    assert_eq!(unit_ranges(&line), [(0, 1)]);
    assert_eq!(usage(&line), (true, false));
    let text = "\u{3000}\u{301}A";
    let p = prepare(input(text, style(punctuation(true, None, false))), &bytes);
    assert_eq!(
        p.analyzed_source().graphemes().collect::<Vec<_>>(),
        [t::Range::new(0, 2), t::Range::new(2, 3)]
    );
    let none = prepare(input(text, style(t::HangingPunctuation::default())), &bytes);
    let baseline = finish(&none, request(&none, 40.0, None, None)).unwrap();
    let line = finish(&p, request(&p, 40.0, None, None)).unwrap();
    assert_eq!(measures(&line), measures(&baseline));
    assert_eq!(line.feedback().advance, baseline.feedback().advance);
    assert!(unit_ranges(&line).is_empty());
    assert_eq!(usage(&line), (false, false));
    assert_eq!(glyphs(&line, &bytes), glyphs(&baseline, &bytes));
    assert_eq!(
        placed(&p, line).source().text().code_units(),
        [0x3000, 0x301, 0x41]
    );
}

#[test]
fn hanging_computed_indication_retains_alternative_and_original_source_custody() {
    for force in [false, true] {
        let base = t::Style {
            locale: Some("en-US".into()),
            hyphens: t::Hyphens::Manual,
            hyphenate_character: t::HyphenateCharacter::String(t::Text::from(".")),
            hyphenate_limit_chars: t::HyphenateLimitChars {
                total: t::HyphenationMinimum::Count(0),
                before: t::HyphenationMinimum::Count(0),
                after: t::HyphenationMinimum::Count(0),
            },
            ..style(punctuation(
                false,
                force.then_some(t::HangingPunctuationEnd::ForceEnd),
                false,
            ))
        };
        let p = prepare(input("AB\u{ad}CDE", base), AHEM);
        let alternative = p
            .alternative_views()
            .find(|a| a.source_range() == Some(t::Range::new(2, 3)))
            .unwrap()
            .handle();
        assert_eq!(alternative.view().insertion_anchor().get(), 3);
        assert_eq!(alternative.view().text(), &t::Text::from("."));
        assert!(
            matches!(alternative.view().definition(),t::PreparedAlternativeDefinition::Source {source,..} if source==t::Range::new(2,3))
        );
        let end = p
            .opportunity_views()
            .find(|o| o.alternatives().any(|a| a.handle() == alternative))
            .unwrap()
            .handle();
        let mut r = request(&p, 40.0, None, None);
        r.end = end.view().boundary();
        r.participants.truncate(r.end.view().participant_offset());
        r.termination = t::PreparedTermination::Ordinary(end);
        r.alternative = Some(alternative.clone());
        r.context.end = t::LineEnd::Soft;
        let line = finish(&p, r).unwrap();
        assert_eq!(line.feedback().advance, 30.0);
        let m = if force { 20.0 } else { 30.0 };
        assert_eq!(measures(&line), (m, m, m, m));
        let g = glyphs(&line, AHEM);
        assert_eq!(g.len(), 3);
        assert_eq!(g[2], (t::Range::new(3, 3), 16, 10.0, 0.0, 20.0, 20.0));
        let row = line
            .edge_units()
            .find(|row| {
                row.owners()
                    .any(|owner| owner == t::LineEdgeOwner::Alternative(alternative.clone()))
            })
            .unwrap();
        assert_eq!(row.advance(), 10.0);
        assert!(row.source_pieces().next().is_none());
        if force {
            assert_eq!(
                row.punctuation_unit().map(|r| (r.start(), r.end())),
                Some((2, 3))
            );
        }
        let mapping = line
            .mapping_views()
            .find(|m| m.alternative() == Some(alternative.clone()))
            .unwrap();
        assert_eq!(mapping.source_range(), t::Range::new(3, 3));
        assert_eq!(mapping.insertion_anchor().unwrap().get(), 3);
        let layout = placed(&p, line);
        assert_eq!(
            layout.source().text().code_units(),
            [65, 66, 0xad, 67, 68, 69]
        );
    }
}

#[test]
fn captured_enclosing_inline_continues_without_its_later_end_decoration() {
    use t::SourceProcessingEvent as E;
    let base = style(punctuation(
        false,
        Some(t::HangingPunctuationEnd::ForceEnd),
        false,
    ));
    let source = t::Source::identified("A. B", t::Id::from_u64(71074), 5);
    let processed = std::sync::Arc::new(
        t::process_contextual_source(t::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::SourceProcessingEnvelope::Complete,
            origins: vec![t::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: t::Range::new(0, 4),
                aggregate: t::Range::new(0, 4),
                kind: t::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            controls: vec![
                t::ParagraphControl {
                    at: 0,
                    kind: t::ParagraphControlKind::InlineBoundaryStart,
                },
                t::ParagraphControl {
                    at: 4,
                    kind: t::ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            scopes: vec![
                t::SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: t::SourceProcessingTopology::Complete,
                    trim: t::WhiteSpaceTrim::default(),
                    base_style: base.clone(),
                    used_direction: LTR,
                },
                t::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: t::WhiteSpaceTrim::default(),
                    style: Some(base),
                    boundaries: Some(t::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
            ],
            paragraphs: vec![t::SourceProcessingParagraph {
                base_direction: LTR,
                paragraph_strut: t::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                E::Control {
                    control: 0,
                    affinity: t::Affinity::After,
                },
                E::Origin {
                    origin: 0,
                    affinity: t::Affinity::After,
                },
                E::Control {
                    control: 1,
                    affinity: t::Affinity::After,
                },
                E::ExitScope(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let analyzed = t::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        t::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| t::WrappingPolicy {
            relation: r.handle(),
            ancestor_wrap: t::Wrap::Word,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let p = system
        .prepare(&analyzed, t::PreparationOptions::default())
        .unwrap();
    let scope = p
        .captured_owner()
        .unwrap()
        .members()
        .flat_map(|member| {
            member
                .scope_intersections()
                .map(|i| i.scope())
                .collect::<Vec<_>>()
        })
        .find(|s| s.role() == t::ScopeRole::Inline)
        .unwrap();
    let entry = p
        .control_views()
        .find(|c| c.handle().captured_control() == scope.entry_control())
        .unwrap()
        .handle();
    let exit = p
        .control_views()
        .find(|c| c.handle().captured_control() == scope.exit_control())
        .unwrap()
        .handle();
    assert_eq!(scope.entry_control(), entry.captured_control());
    assert_eq!(scope.exit_control(), exit.captured_control());
    assert_eq!(entry.view().source_position().get(), 0);
    assert_eq!(exit.view().source_position().get(), 4);
    let cut = p
        .opportunity_views()
        .find(|o| {
            o.kind() == t::PreparedBreakKind::Allowed
                && o.boundary().view().source_after().get() == 3
        })
        .unwrap()
        .handle();
    for bp in [0.0, 5.0] {
        let mut first = request(&p, 40.0, None, None);
        first.end = cut.view().boundary();
        first.termination = t::PreparedTermination::Ordinary(cut.clone());
        first
            .participants
            .truncate(first.end.view().participant_offset());
        first.controls.retain(|c| {
            first
                .participants
                .contains(&t::PreparedParticipant::Control(c.occurrence.clone()))
        });
        first.context.end = t::LineEnd::Soft;
        assert_eq!(first.controls.len(), 1);
        assert_eq!(first.controls[0].occurrence, entry);
        while let Some(occurrence) = p.boundary_context_demand(&first).unwrap() {
            assert!(occurrence == entry || occurrence == exit);
            let advance = if occurrence == exit { bp } else { 0.0 };
            first
                .boundary_context
                .push(t::InlineBoundaryContextAllocation {
                    occurrence,
                    boundary: t::InlineBoundaryAllocation::contribution(advance).unwrap(),
                });
            assert!(first.boundary_context.len() <= 2);
        }
        let line = finish(&p, first).unwrap();
        assert_eq!(line.feedback().advance, 30.0);
        assert_eq!(measures(&line), (10.0, 10.0, 10.0, 10.0));
        assert_eq!(
            line.control_slots()
                .map(|s| s.control())
                .collect::<Vec<_>>()
                .as_slice(),
            std::slice::from_ref(&entry)
        );
        assert_eq!(unit_ranges(&line), [(1, 2)]);
        let mut next = request(&p, 40.0, None, None);
        next.start = cut.view().boundary();
        let from = next.start.view().participant_offset();
        next.participants.drain(..from);
        next.controls.retain(|c| {
            next.participants
                .contains(&t::PreparedParticipant::Control(c.occurrence.clone()))
        });
        assert_eq!(next.controls.len(), 1);
        assert_eq!(next.controls[0].occurrence, exit);
        next.controls[0].boundary = Some(t::InlineBoundaryAllocation::contribution(bp).unwrap());
        next.context.start = t::LineStart::Continuation;
        while let Some(occurrence) = p.boundary_context_demand(&next).unwrap() {
            assert_eq!(occurrence, entry);
            next.boundary_context
                .push(t::InlineBoundaryContextAllocation {
                    occurrence,
                    boundary: t::InlineBoundaryAllocation::CONTRIBUTION_ZERO,
                });
            assert!(next.boundary_context.len() <= 1);
        }
        let receiving = finish(&p, next).unwrap();
        assert_eq!(receiving.feedback().advance, 10.0 + bp);
        assert_eq!(
            measures(&receiving),
            (10.0 + bp, 10.0 + bp, 10.0 + bp, 10.0 + bp)
        );
        assert_eq!(glyphs(&receiving, AHEM)[0].0, t::Range::new(3, 4));
    }
    assert_eq!(p.analyzed_source().source(), &source);
}

#[test]
fn complete_spacing_remaps_punctuation_advance_without_reclassifying_natural_overflow() {
    use t::HangingPunctuationEnd::{AllowEnd, ForceEnd};
    // BreakSpaces keeps SPACE ordinary in every measure, independently of the
    // eligible punctuation; it cannot enter a preserved-whitespace edge chain.
    // A SPACE period has three genuine 10px contributions. Natural AllowEnd
    // overflow is decided at raw30, before the actual SPACE gets its allocation.
    // At width35 an expansion cannot create eligibility; at width25 contraction
    // cannot erase it, although the remapped exclusion becomes zero.
    for (end, width, amount, expected) in [
        (Some(AllowEnd), 35.0, 10.0, (30.0, 40.0, 30.0, 40.0)),
        (Some(AllowEnd), 25.0, -10.0, (10.0, 20.0, 10.0, 20.0)),
        (Some(ForceEnd), 35.0, 10.0, (30.0, 30.0, 30.0, 30.0)),
        (Some(ForceEnd), 25.0, -10.0, (10.0, 10.0, 10.0, 10.0)),
        (None, 35.0, 10.0, (40.0, 40.0, 40.0, 40.0)),
        (None, 25.0, -10.0, (20.0, 20.0, 20.0, 20.0)),
    ] {
        let p = prepare(
            input(
                "A .",
                t::Style {
                    white_space: t::WhiteSpace::BreakSpaces,
                    ..style(punctuation(false, end, false))
                },
            ),
            AHEM,
        );
        assert_eq!(
            p.analyzed_source().style().white_space,
            t::WhiteSpace::BreakSpaces
        );
        assert_eq!(
            p.unit_views()
                .map(|u| (u.source_range(), u.advance()))
                .collect::<Vec<_>>(),
            [
                (t::Range::new(0, 1), 10.0),
                (t::Range::new(1, 2), 10.0),
                (t::Range::new(2, 3), 10.0)
            ]
        );
        let mut r = request(&p, width, None, None);
        r.context.spacing = t::SpacingPolicy::Spaces;
        let selected = p.select(r).unwrap();
        let slots = selected.spacing_slots().collect::<Vec<_>>();
        assert_eq!(slots.len(), 1);
        let space = p
            .unit_views()
            .find(|u| u.source_range() == t::Range::new(1, 2))
            .unwrap()
            .handle();
        assert_eq!(
            slots[0].participants().cloned().collect::<Vec<_>>(),
            [t::PreparedParticipant::Text(space)]
        );
        assert_eq!(
            slots[0]
                .pieces()
                .map(|piece| piece.owner())
                .collect::<Vec<_>>(),
            [t::SpacingSlotOwner::Source {
                origin: p.origin_views().next().unwrap().handle(),
                local: t::Range::new(1, 2),
                aggregate: t::Range::new(1, 2),
            }]
        );
        let natural = selected
            .finalize(surgeist_text::LineAllocations {
                spacing: t::SpacingAllocations::None,
                scopes: surgeist_text::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(natural.feedback().advance, 30.0);
        let natural_measure = match end {
            Some(ForceEnd) => (20.0, 20.0, 20.0, 20.0),
            Some(AllowEnd) => (20.0, width.min(30.0), 20.0, 30.0),
            None => (30.0, 30.0, 30.0, 30.0),
        };
        assert_eq!(measures(&natural), natural_measure);
        let classification = natural.edge_units().nth(2).unwrap().disposition();
        if end == Some(AllowEnd) {
            assert_eq!(
                classification,
                t::LineEdgeDisposition::Conditional {
                    side: t::LineEdgeSide::End,
                    eligible: width < 30.0
                }
            );
        }
        let line = selected
            .finalize(surgeist_text::LineAllocations {
                spacing: t::SpacingAllocations::Complete(vec![t::SpacingAllocation {
                    slot: slots[0].clone(),
                    amount,
                }]),
                scopes: surgeist_text::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(line.feedback().advance, 30.0 + amount);
        assert_eq!(measures(&line), expected);
        assert_eq!(line.selected_line(), selected);
        assert_eq!(usage(&line), (false, false));
        let feedback = line.spacing_slots().collect::<Vec<_>>();
        assert_eq!(feedback.len(), 1);
        assert_eq!(feedback[0].slot, slots[0]);
        assert_eq!(feedback[0].advance, 10.0 + amount);
        assert_eq!(
            feedback[0].disposition,
            t::SpacingSlotDisposition::Retained { trailing: false }
        );
        let period = line.edge_units().nth(2).unwrap();
        assert_eq!(period.advance(), 10.0);
        assert_eq!(period.disposition(), classification);
        assert_eq!(
            period.source_pieces().collect::<Vec<_>>(),
            [t::Range::new(2, 3)]
        );
        assert_eq!(period.excluded_contribution(), (30.0 + amount) - expected.1);
        if end.is_some() {
            assert_eq!(unit_ranges(&line), [(2, 3)]);
        } else {
            assert!(unit_ranges(&line).is_empty());
        }
        let g = glyphs(&line, AHEM);
        assert_eq!(
            g,
            [
                (t::Range::new(0, 1), 35, 10.0, 0.0, 0.0, 0.0),
                (t::Range::new(1, 2), 3, 10.0, 0.0, 10.0, 10.0),
                (
                    t::Range::new(2, 3),
                    16,
                    10.0,
                    0.0,
                    20.0 + amount,
                    20.0 + amount
                )
            ]
        );
        let layout = placed(&p, line);
        carets(&layout, &[0.0, 10.0, 20.0 + amount, 30.0 + amount]);
        assert_eq!(layout.source().text().code_units(), [65, 32, 46]);
        assert_eq!(natural.feedback().advance, 30.0);
        assert_eq!(measures(&natural), natural_measure);
    }
}
