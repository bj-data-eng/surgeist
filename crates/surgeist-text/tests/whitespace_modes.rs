#![forbid(unsafe_code)]

//! Public native whitespace-mode contracts with real registered Ahem SFNT data.
//! Independent values follow the adopted mode model and pinned CSS Text4;
//! Text issues opportunities/measurements, while the test explicitly chooses
//! lines. It does not impersonate Layout's fitter or a paragraph intrinsic API.

#[path = "whitespace_modes/font.rs"]
mod font;

use std::sync::Arc;
use surgeist_text as t;

const LTR: t::paragraph::ParagraphBaseDirection = t::paragraph::ParagraphBaseDirection::LeftToRight;
const RTL: t::paragraph::ParagraphBaseDirection = t::paragraph::ParagraphBaseDirection::RightToLeft;

fn style(mode: t::style::WhiteSpace, wrap: t::style::Wrap) -> t::style::Style {
    t::style::Style {
        font: t::style::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::style::LineHeight::Absolute(10.0),
        white_space: mode,
        wrap,
        ..t::style::Style::default()
    }
}

fn system() -> t::System {
    let mut system = t::System::default();
    system
        .register_fonts(font::bytes().to_vec(), crate::common::FONT_LIMITS)
        .unwrap();
    system
}

fn source(value: &str) -> t::source::Source {
    t::source::Source::identified(
        t::source::Text::from(value),
        t::source::Id::from_u64(64023),
        17,
    )
}

fn origin(source: &t::source::Source, range: t::source::Range) -> t::paragraph::ParagraphOrigin {
    t::paragraph::ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: range,
        aggregate: range,
        kind: t::paragraph::ParagraphOriginKind::Authored,
    }
}

fn input(
    value: &str,
    base: t::style::Style,
    direction: t::paragraph::ParagraphBaseDirection,
) -> t::paragraph::ParagraphInput {
    let source = source(value);
    let end = source.text().len();
    t::paragraph::ParagraphInput {
        source: source.clone(),
        base_style: base,
        segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
        terminal_edge: t::paragraph::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![origin(&source, t::source::Range::new(0, end))],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn prepare_input(
    input: t::paragraph::ParagraphInput,
    options: t::prepared::PreparationOptions,
) -> t::prepared::PreparedText {
    let analyzed = t::analyzed::AnalyzedSource::from_paragraph(
        input,
        t::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    prepare_analyzed(analyzed, options)
}

fn prepare_analyzed(
    analyzed: t::analyzed::AnalyzedSource,
    options: t::prepared::PreparationOptions,
) -> t::prepared::PreparedText {
    // These fixtures explicitly supply one enclosing block's computed Wrap.
    // Bind that real NCA fact to every relation requesting it; omission is an
    // unavailable-context error, not an instruction to infer from widths.
    let wrap = analyzed.style().wrap;
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| t::analyzed::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    system().prepare(&analyzed, options).unwrap()
}

fn prepare(
    value: &str,
    mode: t::style::WhiteSpace,
    wrap: t::style::Wrap,
    direction: t::paragraph::ParagraphBaseDirection,
) -> t::prepared::PreparedText {
    prepare_input(
        input(value, style(mode, wrap), direction),
        t::prepared::PreparationOptions::default(),
    )
}

fn direction(prepared: &t::prepared::PreparedText) -> t::paragraph::ParagraphBaseDirection {
    if prepared.analyzed_source().direction() == t::style::Direction::RightToLeft {
        RTL
    } else {
        LTR
    }
}

fn cuts(prepared: &t::prepared::PreparedText, kind: t::prepared::PreparedBreakKind) -> Vec<usize> {
    prepared
        .opportunity_views()
        .filter(|offer| offer.kind() == kind)
        .map(|offer| offer.boundary().view().source_after().get())
        .collect()
}

fn offer(
    prepared: &t::prepared::PreparedText,
    kind: t::prepared::PreparedBreakKind,
    cut: usize,
) -> t::prepared::PreparedOpportunity {
    prepared
        .opportunity_views()
        .find(|offer| offer.kind() == kind && offer.boundary().view().source_after().get() == cut)
        .unwrap_or_else(|| {
            panic!(
                "missing {kind:?} at source{cut}; offers {:?}",
                prepared
                    .opportunity_views()
                    .map(|offer| (offer.kind(), offer.boundary().view().source_after().get()))
                    .collect::<Vec<_>>()
            )
        })
        .handle()
}

fn finish(
    prepared: &t::prepared::PreparedText,
    start: t::prepared::PreparedBoundary,
    end: t::prepared::PreparedOpportunity,
    width: Option<f64>,
    tabs: t::tabs::TabContext,
) -> t::finalized::FinalizedLine {
    let boundary = end.view().boundary();
    let from = start.view().participant_offset();
    let to = boundary.view().participant_offset();
    let participants = prepared
        .participants()
        .skip(from)
        .take(to - from)
        .collect::<Vec<_>>();
    let controls = participants
        .iter()
        .filter_map(|part| match part {
            t::prepared::PreparedParticipant::Control(occurrence) => Some({
                let occurrence = occurrence.clone();
                t::allocation::ControlAllocation {
                    boundary: matches!(
                        occurrence.view().kind(),
                        surgeist_text::paragraph::ParagraphControlKind::InlineBoundaryStart
                            | surgeist_text::paragraph::ParagraphControlKind::InlineBoundaryEnd
                    )
                    .then_some(
                        surgeist_text::allocation::InlineBoundaryAllocation::CONTRIBUTION_ZERO,
                    ),
                    occurrence,
                    strut: None,
                }
            }),
            _ => None,
        })
        .collect();
    prepared
        .select(t::selection::LineSelectionRequest {
            inline_boxes: None,
            segment: prepared.source_segments().next().unwrap(),
            boundary_context: Vec::new(),
            start,
            end: boundary,
            termination: t::selection::PreparedTermination::Ordinary(end.clone()),
            participants,
            incoming: None,
            alternative: None,
            atomics: vec![],
            controls,
            line_box_struts: Vec::new(),
            line_strut: None,
            context: t::selection::LineContext {
                alignment_insets: surgeist_text::allocation::LineAlignmentInsets::ZERO,
                formatted_line: surgeist_text::selection::FormattedLineContext::default(),
                base_direction: direction(prepared),
                line_box_direction: match direction(prepared) {
                    surgeist_text::paragraph::ParagraphBaseDirection::LeftToRight => {
                        surgeist_text::selection::LineBoxDirection::LeftToRight
                    }
                    surgeist_text::paragraph::ParagraphBaseDirection::RightToLeft => {
                        surgeist_text::selection::LineBoxDirection::RightToLeft
                    }
                },
                orientation: t::selection::LineOrientation::Horizontal,
                start: if from == 0 {
                    t::selection::LineStart::Paragraph
                } else {
                    t::selection::LineStart::Continuation
                },
                end: match end.view().kind() {
                    t::prepared::PreparedBreakKind::Forced => t::selection::LineEnd::Forced,
                    t::prepared::PreparedBreakKind::ParagraphEnd => {
                        t::selection::LineEnd::Paragraph
                    }
                    _ => t::selection::LineEnd::Soft,
                },
                available_inline: width,
                indent: 0.0,
                alignment: t::options::Alignment::Start,
                tabs,
                spacing: t::spacing::SpacingPolicy::Spaces,
            },
        })
        .unwrap()
        .finalize(surgeist_text::scope_alignment::LineAllocations {
            spacing: t::spacing::SpacingAllocations::None,
            scopes: surgeist_text::scope_alignment::ScopeAllocations::None,
        })
        .unwrap()
}

fn whole(prepared: &t::prepared::PreparedText, width: Option<f64>) -> t::finalized::FinalizedLine {
    let end = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == t::prepared::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    finish(
        prepared,
        prepared.boundary_views().next().unwrap().handle(),
        end,
        width,
        t::tabs::TabContext::NoTabs,
    )
}

fn forced_lines(prepared: &t::prepared::PreparedText) -> Vec<t::finalized::FinalizedLine> {
    let mut start = prepared.boundary_views().next().unwrap().handle();
    let mut lines = Vec::new();
    for end in prepared
        .opportunity_views()
        .filter(|offer| {
            matches!(
                offer.kind(),
                t::prepared::PreparedBreakKind::Forced
                    | t::prepared::PreparedBreakKind::ParagraphEnd
            )
        })
        .map(|offer| offer.handle())
    {
        let next = end.view().boundary();
        lines.push(finish(
            prepared,
            start,
            end,
            Some(100.0),
            t::tabs::TabContext::NoTabs,
        ));
        start = next;
    }
    lines
}

fn glyphs(line: &t::finalized::FinalizedLine) -> Vec<(u32, f32, f64)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), font::bytes());
            assert_eq!(run.face().index(), 0);
            let instance = run.effective_instance().unwrap();
            run.glyphs()
                .map(|glyph| {
                    instance
                        .glyph_outline(glyph.id(), run.face().catalogue_identity())
                        .unwrap();
                    (
                        glyph.id(),
                        glyph.advance(),
                        run.origin().x + glyph.pen() + f64::from(glyph.x()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn measures(line: &t::finalized::FinalizedLine) -> [f64; 5] {
    let m = line.measurements();
    [
        line.feedback().advance,
        m.fit().advance(),
        m.alignment().advance(),
        m.min_content_contribution().advance(),
        m.max_content_contribution().advance(),
    ]
}

fn no_hang(line: &t::finalized::FinalizedLine, advance: f64) {
    assert_eq!(measures(line), [advance; 5]);
    assert_eq!(
        (line.feedback().hanging_start, line.feedback().hanging_end),
        (0.0, 0.0)
    );
    assert!(
        line.edge_units()
            .all(|unit| unit.disposition() == t::finalized::LineEdgeDisposition::Ordinary)
    );
}

fn place_and_copy(
    prepared: t::prepared::PreparedText,
    lines: Vec<t::finalized::FinalizedLine>,
    range: t::source::Range,
    expected_plain: &str,
) {
    let original = prepared.analyzed_source().source().clone();
    let layout = t::layout::Layout::from_finalized_lines(t::placement::PlacementRequest {
        prepared,
        coverage: t::placement::PlacementCoverage::WholeParagraph,
        entries: lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                t::placement::PlacementEntry::Formatted(t::placement::LinePlacement {
                    line,
                    anchor: t::placement::LineAnchor {
                        inline_origin: 0.0,
                        baseline: 8.0 + index as f64 * 10.0,
                    },
                    orientation: t::selection::LineOrientation::Horizontal,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap();
    assert_eq!(layout.source(), &original);
    let before = layout
        .positions()
        .find(|position| {
            position.source_cursor()
                == t::layout::Cursor::new(range.start, t::layout::Affinity::After)
        })
        .unwrap();
    let after = layout
        .positions()
        .find(|position| {
            position.source_cursor()
                == t::layout::Cursor::new(range.end, t::layout::Affinity::Before)
        })
        .unwrap();
    let selected = layout.position_selection(&before, &after).unwrap();
    assert_eq!(
        selected.source_text().code_units(),
        &original.text().code_units()[range.start..range.end]
    );
    assert_eq!(selected.plaintext(), &t::source::Text::from(expected_plain));
}

fn contextual(
    value: &str,
    base: t::style::Style,
    trim: t::capture::WhiteSpaceTrim,
) -> Arc<t::processing::ProcessedSource> {
    let source = source(value);
    let end = source.text().len();
    use t::capture::SourceProcessingEvent as E;
    Arc::new(
        t::capture::process_contextual_source(t::capture::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::capture::SourceProcessingEnvelope::Complete,
            origins: vec![origin(&source, t::source::Range::new(0, end))],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            scopes: vec![
                t::capture::SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: t::capture::SourceProcessingTopology::Complete,
                    trim: t::capture::WhiteSpaceTrim::default(),
                    base_style: base,
                    used_direction: LTR,
                },
                t::capture::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim,
                    style: None,
                    boundaries: None,
                },
            ],
            paragraphs: vec![t::capture::SourceProcessingParagraph {
                base_direction: LTR,
                paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                E::Origin {
                    origin: 0,
                    affinity: t::layout::Affinity::After,
                },
                E::ExitScope(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    )
}

#[test]
fn registered_font_qualifies_real_cells_blank_separators_mark_and_transform_targets() {
    let p = prepare(
        "ABX",
        t::style::WhiteSpace::BreakSpaces,
        t::style::Wrap::Word,
        LTR,
    );
    let line = whole(&p, Some(100.0));
    assert_eq!(
        glyphs(&line),
        [(35, 10.0, 0.0), (36, 10.0, 10.0), (58, 10.0, 20.0)]
    );
    no_hang(&line, 30.0);
    assert!(matches!(
        line.outline_ink().unwrap(),
        t::finalized::LineOutlineInk::Bounds(_)
    ));
    for separator in [
        " ", "\u{1680}", "\u{2003}", "\u{2007}", "\u{202f}", "\u{a0}",
    ] {
        let p = prepare(
            separator,
            t::style::WhiteSpace::BreakSpaces,
            t::style::Wrap::Word,
            LTR,
        );
        let line = whole(&p, Some(100.0));
        assert_eq!(glyphs(&line), [(3, 10.0, 0.0)]);
        assert_eq!(
            line.outline_ink().unwrap(),
            t::finalized::LineOutlineInk::Empty
        );
    }
    let p = prepare(
        "X\u{301}",
        t::style::WhiteSpace::BreakSpaces,
        t::style::Wrap::Word,
        LTR,
    );
    assert_eq!(
        glyphs(&whole(&p, Some(100.0)))
            .iter()
            .map(|g| (g.0, g.1))
            .collect::<Vec<_>>(),
        [(58, 10.0), (100, 0.0)]
    );
    let p = prepare(
        "Ａ　Ｂ",
        t::style::WhiteSpace::BreakSpaces,
        t::style::Wrap::Word,
        LTR,
    );
    assert_eq!(
        glyphs(&whole(&p, Some(100.0)))
            .iter()
            .map(|g| (g.0, g.1))
            .collect::<Vec<_>>(),
        [(277, 10.0), (1, 10.0), (274, 10.0)]
    );
}

#[test]
fn preserve_breaks_keeps_each_real_forced_delimiter_and_deleted_source_affinity_even_nowrap() {
    const VALUE: &str = "A \t\n B\n\nC";
    for wrap in [t::style::Wrap::Word, t::style::Wrap::None] {
        for direction in [LTR, RTL] {
            let p = prepare(VALUE, t::style::WhiteSpace::PreserveBreaks, wrap, direction);
            let analyzed = p.analyzed_source();
            let processed = analyzed.processed_source();
            assert_eq!(
                processed.processed_text(),
                &t::source::Text::from("A\nB\n\nC")
            );
            assert_eq!(processed.source().text(), &t::source::Text::from(VALUE));
            for (at, before, after) in [(1, 1, 3), (2, 4, 5)] {
                assert_eq!(
                    processed
                        .processed_to_source(
                            processed.position(at).unwrap(),
                            t::layout::Affinity::Before
                        )
                        .unwrap()
                        .get(),
                    before
                );
                assert_eq!(
                    processed
                        .processed_to_source(
                            processed.position(at).unwrap(),
                            t::layout::Affinity::After
                        )
                        .unwrap()
                        .get(),
                    after
                );
            }
            for (at, expected) in [(1, 1), (2, 1), (4, 2)] {
                for affinity in [t::layout::Affinity::Before, t::layout::Affinity::After] {
                    assert_eq!(
                        processed
                            .source_to_processed(
                                t::source::SourcePosition::try_new(processed.source().text(), at)
                                    .unwrap(),
                                affinity
                            )
                            .unwrap()
                            .get(),
                        expected
                    );
                }
            }
            let forced: Vec<_> = p
                .opportunity_views()
                .filter(|offer| offer.kind() == t::prepared::PreparedBreakKind::Forced)
                .map(|offer| offer.boundary().view().processed_position().get())
                .collect();
            assert_eq!(forced, [2, 4, 5]);
            let lines = forced_lines(&p);
            assert_eq!(
                lines
                    .iter()
                    .map(|line| line.feedback().advance)
                    .collect::<Vec<_>>(),
                [10.0, 10.0, 0.0, 10.0]
            );
            place_and_copy(p, lines, t::source::Range::new(0, 9), "A\nB\n\nC");
        }
    }
}

#[test]
fn new_modes_distinguish_host_cr_delimiters_from_ordinary_cr_space() {
    for mode in [
        t::style::WhiteSpace::PreserveBreaks,
        t::style::WhiteSpace::BreakSpaces,
    ] {
        for (value, breaks, expected, count) in [
            (
                "A\rB",
                t::processing::DocumentSegmentBreaks::Defined(vec![]),
                "A B",
                0,
            ),
            (
                "A\rB",
                t::processing::DocumentSegmentBreaks::Defined(vec![t::source::Range::new(1, 2)]),
                "A\nB",
                1,
            ),
            (
                "A\r\nB\rC",
                t::processing::DocumentSegmentBreaks::Defined(vec![t::source::Range::new(1, 3)]),
                "A\nB C",
                1,
            ),
        ] {
            let mut request = input(value, style(mode, t::style::Wrap::None), LTR);
            request.segment_breaks = breaks;
            let p = prepare_input(request, t::prepared::PreparationOptions::default());
            assert_eq!(
                p.analyzed_source().processed_source().processed_text(),
                &t::source::Text::from(expected)
            );
            assert_eq!(
                cuts(&p, t::prepared::PreparedBreakKind::Forced).len(),
                count
            );
            let lines = forced_lines(&p);
            place_and_copy(
                p,
                lines,
                t::source::Range::new(0, value.encode_utf16().count()),
                expected,
            );
        }
    }
}

#[test]
fn preserve_breaks_collapses_across_inline_origins_and_transparent_bidi_controls() {
    let mut request = input(
        "A \t\n B\n\nC",
        style(t::style::WhiteSpace::PreserveBreaks, t::style::Wrap::Word),
        LTR,
    );
    let retained = request.source.clone();
    request.origins = [
        t::source::Range::new(0, 2),
        t::source::Range::new(2, 5),
        t::source::Range::new(5, 9),
    ]
    .into_iter()
    .map(|range| origin(&retained, range))
    .collect();
    request.connections = [2, 5]
        .into_iter()
        .map(|at| t::paragraph::ParagraphConnection {
            at,
            kind: t::paragraph::ParagraphConnectionKind::Continuous,
        })
        .collect();
    request.boundary_policies = (0..2)
        .map(|index| t::paragraph::ParagraphBoundaryPolicy {
            definition: t::paragraph::ParagraphBoundaryDefinition::Connection(index),
            preceding_wrap: t::style::Wrap::Word,
            following_wrap: t::style::Wrap::Word,
            ancestor_wrap: t::style::Wrap::Word,
        })
        .collect();
    request.controls = vec![
        t::paragraph::ParagraphControl {
            at: 2,
            kind: t::paragraph::ParagraphControlKind::RightToLeftEmbedding,
        },
        t::paragraph::ParagraphControl {
            at: 5,
            kind: t::paragraph::ParagraphControlKind::PopDirectionalFormat,
        },
    ];
    request.anchor_order = (0..2)
        .map(|index| t::paragraph::ParagraphAnchor {
            item: t::paragraph::ParagraphAnchorItem::Control(index),
            affinity: t::layout::Affinity::After,
        })
        .collect();
    let p = prepare_input(request, t::prepared::PreparationOptions::default());
    assert_eq!(
        p.analyzed_source().processed_source().processed_text(),
        &t::source::Text::from("A\nB\n\nC")
    );
    assert_eq!(cuts(&p, t::prepared::PreparedBreakKind::Forced).len(), 3);
    let lines = forced_lines(&p);
    assert_eq!(
        lines
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [10.0, 10.0, 0.0, 10.0]
    );
    place_and_copy(p, lines, t::source::Range::new(0, 9), "A\nB\n\nC");
}

#[test]
fn preserve_breaks_phase_two_removes_space_and_trailing_ogham_but_hangs_em_not_nbsp() {
    for wrap in [t::style::Wrap::Word, t::style::Wrap::None] {
        for direction in [LTR, RTL] {
            for (value, raw, fit, removed, hung) in [
                (" X ", 10.0, 10.0, 2, 0),
                ("\u{1680}X\u{1680}", 20.0, 20.0, 1, 0),
                ("X\u{2003}", 20.0, 10.0, 0, 1),
                ("X\u{a0}", 20.0, 20.0, 0, 0),
            ] {
                let p = prepare(value, t::style::WhiteSpace::PreserveBreaks, wrap, direction);
                let line = whole(&p, Some(15.0));
                assert_eq!(
                    line.feedback().advance,
                    raw,
                    "{value:?} {wrap:?} {direction:?}"
                );
                assert_eq!(line.measurements().fit().advance(), fit);
                assert_eq!(line.measurements().alignment().advance(), fit);
                assert_eq!(
                    line.edge_units()
                        .filter(
                            |unit| unit.disposition() == t::finalized::LineEdgeDisposition::Removed
                        )
                        .count(),
                    removed
                );
                assert_eq!(
                    line.edge_units()
                        .filter(|unit| matches!(
                            unit.disposition(),
                            t::finalized::LineEdgeDisposition::Unconditional { .. }
                        ))
                        .count(),
                    hung
                );
            }
        }
    }
}

#[test]
fn break_spaces_issues_each_real_cut_and_two_chosen_twenty_pixel_lines_without_soft_copy_breaks() {
    let p = prepare(
        "A  B",
        t::style::WhiteSpace::BreakSpaces,
        t::style::Wrap::Word,
        LTR,
    );
    assert_eq!(cuts(&p, t::prepared::PreparedBreakKind::Allowed), [2, 3]);
    let start = p.boundary_views().next().unwrap().handle();
    let first_offer = offer(&p, t::prepared::PreparedBreakKind::Allowed, 2);
    let continuation = first_offer.view().boundary();
    let first = finish(
        &p,
        start,
        first_offer,
        Some(20.0),
        t::tabs::TabContext::NoTabs,
    );
    let second = finish(
        &p,
        continuation,
        offer(&p, t::prepared::PreparedBreakKind::ParagraphEnd, 4),
        Some(20.0),
        t::tabs::TabContext::NoTabs,
    );
    for (line, expected_range) in [
        (&first, t::source::Range::new(0, 2)),
        (&second, t::source::Range::new(2, 4)),
    ] {
        no_hang(line, 20.0);
        let selected = line.selected_line();
        let request = selected.request();
        assert_eq!(
            (
                request.start.view().source_before().get(),
                request.end.view().source_after().get()
            ),
            (expected_range.start, expected_range.end)
        );
        assert_eq!(glyphs(line).iter().map(|glyph| glyph.1).sum::<f32>(), 20.0);
    }
    // A paragraph min/max result belongs to the consumer. These actual legal
    // Text selections independently establish the intrinsic partition witness.
    let full = whole(&p, None);
    assert_eq!(
        full.measurements().max_content_contribution().advance(),
        40.0
    );
    let mut start = p.boundary_views().next().unwrap().handle();
    let mut minimum_chunks = Vec::new();
    for end in [
        offer(&p, t::prepared::PreparedBreakKind::Allowed, 2),
        offer(&p, t::prepared::PreparedBreakKind::Allowed, 3),
        offer(&p, t::prepared::PreparedBreakKind::ParagraphEnd, 4),
    ] {
        let next = end.view().boundary();
        minimum_chunks.push(
            finish(&p, start, end, None, t::tabs::TabContext::NoTabs)
                .measurements()
                .min_content_contribution()
                .advance(),
        );
        start = next;
    }
    assert_eq!(minimum_chunks, [20.0, 10.0, 10.0]);
    place_and_copy(p, vec![first, second], t::source::Range::new(0, 4), "A  B");
    let preserved = prepare(
        "A  B",
        t::style::WhiteSpace::Preserve,
        t::style::Wrap::Word,
        LTR,
    );
    assert_eq!(
        cuts(&preserved, t::prepared::PreparedBreakKind::Allowed),
        [3]
    );
    let line = finish(
        &preserved,
        preserved.boundary_views().next().unwrap().handle(),
        offer(&preserved, t::prepared::PreparedBreakKind::Allowed, 3),
        Some(20.0),
        t::tabs::TabContext::NoTabs,
    );
    assert_eq!(
        (line.feedback().advance, line.measurements().fit().advance()),
        (30.0, 10.0)
    );
}

#[test]
fn break_spaces_trailing_separator_never_hangs_in_any_real_terminal_or_nowrap_context() {
    for separator in [' ', '\u{2003}', '\u{2007}', '\u{202f}', '\u{a0}'] {
        for width in [15.0, 40.0] {
            for wrap in [t::style::Wrap::Word, t::style::Wrap::None] {
                for direction in [LTR, RTL] {
                    for (forced, terminal) in [
                        (false, t::paragraph::ParagraphTerminalEdge::Unknown),
                        (false, t::paragraph::ParagraphTerminalEdge::BlockEnd),
                        (true, t::paragraph::ParagraphTerminalEdge::BlockEnd),
                    ] {
                        let value = if forced {
                            format!("X{separator}\nB")
                        } else {
                            format!("X{separator}")
                        };
                        let mut request = input(
                            &value,
                            style(t::style::WhiteSpace::BreakSpaces, wrap),
                            direction,
                        );
                        request.terminal_edge = terminal;
                        let p = prepare_input(request, t::prepared::PreparationOptions::default());
                        let end = if forced {
                            p.opportunity_views()
                                .find(|offer| {
                                    offer.kind() == t::prepared::PreparedBreakKind::Forced
                                })
                                .unwrap()
                                .handle()
                        } else {
                            offer(&p, t::prepared::PreparedBreakKind::ParagraphEnd, 2)
                        };
                        let line = finish(
                            &p,
                            p.boundary_views().next().unwrap().handle(),
                            end,
                            Some(width),
                            t::tabs::TabContext::NoTabs,
                        );
                        no_hang(&line, 20.0);
                        assert_eq!(glyphs(&line).iter().map(|glyph| glyph.1).sum::<f32>(), 20.0);
                    }
                    if wrap == t::style::Wrap::Word
                        && !matches!(separator, '\u{a0}' | '\u{2007}' | '\u{202f}')
                    {
                        let p = prepare(
                            &format!("X{separator}B"),
                            t::style::WhiteSpace::BreakSpaces,
                            wrap,
                            direction,
                        );
                        let line = finish(
                            &p,
                            p.boundary_views().next().unwrap().handle(),
                            offer(&p, t::prepared::PreparedBreakKind::Allowed, 2),
                            Some(width),
                            t::tabs::TabContext::NoTabs,
                        );
                        no_hang(&line, 20.0);
                    }
                }
            }
        }
    }
}

#[test]
fn break_spaces_cannot_split_a_space_combining_unit_or_create_nbsp_owned_cuts() {
    let p = prepare(
        "A \u{301}B",
        t::style::WhiteSpace::BreakSpaces,
        t::style::Wrap::Word,
        LTR,
    );
    assert_eq!(
        p.analyzed_source().graphemes().collect::<Vec<_>>(),
        [
            t::source::Range::new(0, 1),
            t::source::Range::new(1, 3),
            t::source::Range::new(3, 4)
        ]
    );
    assert!(
        cuts(&p, t::prepared::PreparedBreakKind::Allowed).is_empty(),
        "no interior or relocated cut for SPACE+combining mark"
    );
    let line = whole(&p, Some(100.0));
    no_hang(&line, 30.0);
    assert_eq!(
        glyphs(&line).iter().map(|g| (g.0, g.1)).collect::<Vec<_>>(),
        [(35, 10.0), (3, 10.0), (100, 0.0), (36, 10.0)]
    );
    for mode in [
        t::style::WhiteSpace::BreakSpaces,
        t::style::WhiteSpace::Preserve,
    ] {
        let p = prepare("A\u{a0}B", mode, t::style::Wrap::Word, LTR);
        assert!(cuts(&p, t::prepared::PreparedBreakKind::Allowed).is_empty());
    }
}

#[test]
fn break_spaces_preserves_real_css_tabs_and_issues_after_each_without_font_glyphs() {
    let mut base = style(t::style::WhiteSpace::BreakSpaces, t::style::Wrap::Word);
    base.tab_size = t::tabs::TabSize::spaces(2.0).unwrap();
    let mut request = input("A\t\tB", base.clone(), LTR);
    request.tab_reference = Some(t::tabs::ParagraphTabReference::new(base, LTR).unwrap());
    let p = prepare_input(
        request,
        t::prepared::PreparationOptions::try_new(1.0, t::tabs::TabPreparation::CssStops).unwrap(),
    );
    assert_eq!(
        p.analyzed_source().processed_source().processed_text(),
        &t::source::Text::from("A\t\tB")
    );
    assert_eq!(cuts(&p, t::prepared::PreparedBreakKind::Allowed), [2, 3]);
    let line = finish(
        &p,
        p.boundary_views().next().unwrap().handle(),
        offer(&p, t::prepared::PreparedBreakKind::ParagraphEnd, 4),
        Some(100.0),
        t::tabs::TabContext::CssStops { origin: Some(0.0) },
    );
    no_hang(&line, 50.0);
    assert_eq!(glyphs(&line), [(35, 10.0, 0.0), (36, 10.0, 40.0)]);
    place_and_copy(p, vec![line], t::source::Range::new(0, 4), "A\t\tB");
}

#[test]
fn captured_trim_runs_before_preserve_breaks_collapse_and_full_width_copy_stays_original() {
    let mut base = style(t::style::WhiteSpace::PreserveBreaks, t::style::Wrap::Word);
    base.text_transform = t::style::TextTransform::Set(
        t::style::TextTransformSet::none()
            .with_case(t::style::CaseTransform::Uppercase)
            .with_full_width(true),
    );
    let processed = contextual(
        "  a \t b  ",
        base,
        t::capture::WhiteSpaceTrim {
            discard_inner: true,
            ..t::capture::WhiteSpaceTrim::default()
        },
    );
    assert_eq!(processed.processed_text(), &t::source::Text::from("a b"));
    let analyzed = t::analyzed::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        t::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let p = prepare_analyzed(analyzed, t::prepared::PreparationOptions::default());
    let line = whole(&p, Some(100.0));
    assert_eq!(
        glyphs(&line)
            .iter()
            .map(|glyph| (glyph.0, glyph.1))
            .collect::<Vec<_>>(),
        // Collapsible original SPACE does not qualify for full-width. The
        // transformed letters do, and retain distinct actual glyphs.
        [(277, 10.0), (3, 10.0), (274, 10.0)]
    );
    place_and_copy(p, vec![line], t::source::Range::new(0, 9), "a b");
}

#[test]
fn preserve_spaces_and_discard_keep_closed_mode_semantics_and_plaintext_ordering() {
    for wrap in [t::style::Wrap::Word, t::style::Wrap::None] {
        let p = prepare("X\t\n", t::style::WhiteSpace::PreserveSpaces, wrap, LTR);
        assert_eq!(
            p.analyzed_source().processed_source().processed_text(),
            &t::source::Text::from("X  ")
        );
        assert!(cuts(&p, t::prepared::PreparedBreakKind::Forced).is_empty());
        let line = whole(&p, Some(15.0));
        assert_eq!(line.feedback().advance, 30.0);
        assert_eq!(
            line.measurements().fit().advance(),
            if wrap == t::style::Wrap::Word {
                10.0
            } else {
                30.0
            }
        );
        place_and_copy(p, vec![line], t::source::Range::new(0, 3), "X  ");
    }
    let untransformed = prepare(
        " a\t\nb ",
        t::style::WhiteSpace::Discard,
        t::style::Wrap::Word,
        LTR,
    );
    assert!(cuts(&untransformed, t::prepared::PreparedBreakKind::Allowed).is_empty());
    assert!(cuts(&untransformed, t::prepared::PreparedBreakKind::Forced).is_empty());
    let mut base = style(t::style::WhiteSpace::Discard, t::style::Wrap::Word);
    base.text_transform = t::style::TextTransform::Set(
        t::style::TextTransformSet::none()
            .with_case(t::style::CaseTransform::Uppercase)
            .with_full_width(true),
    );
    let p = prepare_input(
        input(" a\t\nb ", base, LTR),
        t::prepared::PreparationOptions::default(),
    );
    assert_eq!(
        p.analyzed_source().processed_source().processed_text(),
        &t::source::Text::from("ab")
    );
    // Surviving full-width letters retain their ordinary opportunities;
    // Discard must not manufacture a disappearing-whitespace opportunity.
    assert!(cuts(&p, t::prepared::PreparedBreakKind::Forced).is_empty());
    let line = whole(&p, Some(100.0));
    assert_eq!(
        glyphs(&line)
            .iter()
            .map(|glyph| glyph.0)
            .collect::<Vec<_>>(),
        [277, 274]
    );
    place_and_copy(p, vec![line], t::source::Range::new(0, 6), "ab");
}

#[test]
fn preserve_breaks_does_not_collapse_across_distinct_real_block_domains() {
    use t::capture::SourceProcessingEvent as E;
    let source = source("A \tB");
    let base = style(t::style::WhiteSpace::PreserveBreaks, t::style::Wrap::Word);
    let block = || t::capture::SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: t::capture::SourceProcessingTopology::Complete,
        trim: t::capture::WhiteSpaceTrim::default(),
        base_style: base.clone(),
        used_direction: LTR,
    };
    let paragraph = || t::capture::SourceProcessingParagraph {
        base_direction: LTR,
        paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
    };
    let processed = Arc::new(
        t::capture::process_contextual_source(t::capture::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::capture::SourceProcessingEnvelope::Complete,
            origins: vec![
                origin(&source, t::source::Range::new(0, 2)),
                origin(&source, t::source::Range::new(2, 4)),
            ],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            scopes: vec![block(), block()],
            paragraphs: vec![paragraph(), paragraph()],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: t::layout::Affinity::After,
                },
                E::ExitParagraph(0),
                E::ExitScope(0),
                E::EnterScope(1),
                E::EnterParagraph(1),
                E::Origin {
                    origin: 1,
                    affinity: t::layout::Affinity::After,
                },
                E::ExitParagraph(1),
                E::ExitScope(1),
            ],
        })
        .unwrap(),
    );
    assert_eq!(processed.processed_text(), &t::source::Text::from("A  B"));
    let paragraphs = processed.paragraphs().collect::<Vec<_>>();
    assert_eq!(paragraphs.len(), 2);
    for (paragraph, range, plain) in [
        (paragraphs[0].clone(), t::source::Range::new(0, 2), "A "),
        (paragraphs[1].clone(), t::source::Range::new(2, 4), "B"),
    ] {
        assert_eq!(paragraph.source_pieces().collect::<Vec<_>>(), [range]);
        let p = prepare_analyzed(
            t::analyzed::AnalyzedSource::from_processed_paragraph(
                paragraph,
                t::analyzed::AnalysisTypography::HorizontalComposition,
            )
            .unwrap(),
            t::prepared::PreparationOptions::default(),
        );
        let line = whole(&p, Some(100.0));
        assert_eq!(line.feedback().advance, 10.0);
        place_and_copy(p, vec![line], range, plain);
    }
}

#[test]
fn break_spaces_other_breakable_separators_have_each_real_offer_and_retain_selected_width() {
    let value = "A\u{2003}\u{2002}B";
    let p = prepare(
        value,
        t::style::WhiteSpace::BreakSpaces,
        t::style::Wrap::Word,
        LTR,
    );
    assert_eq!(cuts(&p, t::prepared::PreparedBreakKind::Allowed), [2, 3]);
    let first_end = offer(&p, t::prepared::PreparedBreakKind::Allowed, 2);
    let second_start = first_end.view().boundary();
    let first = finish(
        &p,
        p.boundary_views().next().unwrap().handle(),
        first_end,
        Some(20.0),
        t::tabs::TabContext::NoTabs,
    );
    let second = finish(
        &p,
        second_start,
        offer(&p, t::prepared::PreparedBreakKind::ParagraphEnd, 4),
        Some(20.0),
        t::tabs::TabContext::NoTabs,
    );
    no_hang(&first, 20.0);
    no_hang(&second, 20.0);
    assert_eq!(
        glyphs(&first)
            .iter()
            .map(|g| (g.0, g.1))
            .collect::<Vec<_>>(),
        [(35, 10.0), (3, 10.0)]
    );
    assert_eq!(
        glyphs(&second)
            .iter()
            .map(|g| (g.0, g.1))
            .collect::<Vec<_>>(),
        [(3, 10.0), (36, 10.0)]
    );
    place_and_copy(p, vec![first, second], t::source::Range::new(0, 4), value);
}

#[test]
fn raw_standalone_refuses_new_modes_while_native_source_paths_are_functional() {
    for (mode, feature) in [
        (
            t::style::WhiteSpace::PreserveBreaks,
            t::style::TextStyleFeature::WhiteSpacePreserveBreaks,
        ),
        (
            t::style::WhiteSpace::BreakSpaces,
            t::style::TextStyleFeature::WhiteSpaceBreakSpaces,
        ),
    ] {
        let error = system()
            .layout(
                source("A B"),
                style(mode, t::style::Wrap::Word),
                t::options::Options::default(),
            )
            .unwrap_err();
        assert_eq!(
            error.detail(),
            Some(&t::ErrorDetail::UnsupportedTextStyle {
                feature,
                reason: t::style::UnsupportedTextStyleReason::RequiresProcessedSourceLayout
            })
        );
        let processed = t::processing::process_source_whitespace(
            source("A B"),
            style(mode, t::style::Wrap::Word),
            t::processing::DocumentSegmentBreaks::LineFeedDefault,
        )
        .unwrap();
        assert_eq!(processed.processed_text(), &t::source::Text::from("A B"));
        assert_eq!(processed.source().revision(), 17);
    }
}

#[test]
fn break_spaces_exact_glue_and_joiner_cuts_follow_the_adopted_unicode17_priority() {
    // #64 determination6391670728 and its source reconciliation: own GL and
    // WJ/ZWJ/TCU protections remain; Unicode17 LB12a still exempts SP and BA.
    // An illegal after-SPACE cut is never moved to a combining unit's far end.
    for (value, expected) in [
        ("A\u{2007}B", vec![]),
        ("A\u{202f}B", vec![]),
        (" \u{a0}", vec![1]),
        ("\t\u{a0}", vec![1]),
        ("\u{2007}\u{a0}", vec![]),
        ("\u{2007}\u{2060}B", vec![]),
        (" \u{2060}B", vec![]),
        ("A\u{2007}\u{200b}\u{202f}B", vec![3]),
        (" \u{200d}\u{301}", vec![]),
        (" \u{200d}\u{301}B", vec![]),
        ("A B", vec![2]),
        ("A  B", vec![2, 3]),
        ("\t\tB", vec![1, 2]),
        ("\u{2007}\u{202f}B", vec![]),
        (" \u{2007}B", vec![1]),
    ] {
        let p = if value.contains('\t') {
            let base = style(t::style::WhiteSpace::BreakSpaces, t::style::Wrap::Word);
            let mut request = input(value, base.clone(), LTR);
            request.tab_reference = Some(t::tabs::ParagraphTabReference::new(base, LTR).unwrap());
            prepare_input(
                request,
                t::prepared::PreparationOptions::try_new(1.0, t::tabs::TabPreparation::CssStops)
                    .unwrap(),
            )
        } else {
            prepare(
                value,
                t::style::WhiteSpace::BreakSpaces,
                t::style::Wrap::Word,
                LTR,
            )
        };
        assert_eq!(
            cuts(&p, t::prepared::PreparedBreakKind::Allowed),
            expected,
            "actual source {value:?}"
        );
    }
}

#[test]
fn preserve_breaks_and_collapse_share_space_collapse_but_each_deleted_original_keeps_its_gate() {
    for (first_mode, second_mode) in [
        (
            t::style::WhiteSpace::PreserveBreaks,
            t::style::WhiteSpace::Collapse,
        ),
        (
            t::style::WhiteSpace::Collapse,
            t::style::WhiteSpace::PreserveBreaks,
        ),
    ] {
        for (first_wrap, second_wrap, expected_cut) in [
            (t::style::Wrap::None, t::style::Wrap::Word, 3),
            (t::style::Wrap::Word, t::style::Wrap::None, 2),
        ] {
            let mut request = input(
                "A  B",
                style(t::style::WhiteSpace::PreserveBreaks, t::style::Wrap::None),
                LTR,
            );
            request
                .source
                .span(t::source::Range::new(0, 2), style(first_mode, first_wrap));
            request
                .source
                .span(t::source::Range::new(2, 4), style(second_mode, second_wrap));
            request.origins = vec![origin(&request.source, t::source::Range::new(0, 4))];
            let p = prepare_input(request, t::prepared::PreparationOptions::default());
            assert_eq!(
                p.analyzed_source().processed_source().processed_text(),
                &t::source::Text::from("A B")
            );
            assert_eq!(
                cuts(&p, t::prepared::PreparedBreakKind::Allowed),
                [expected_cut]
            );
            assert!(cuts(&p, t::prepared::PreparedBreakKind::Forced).is_empty());
            let first_end = offer(&p, t::prepared::PreparedBreakKind::Allowed, expected_cut);
            let second_start = first_end.view().boundary();
            let first = finish(
                &p,
                p.boundary_views().next().unwrap().handle(),
                first_end,
                Some(10.0),
                t::tabs::TabContext::NoTabs,
            );
            let second = finish(
                &p,
                second_start,
                offer(&p, t::prepared::PreparedBreakKind::ParagraphEnd, 4),
                Some(10.0),
                t::tabs::TabContext::NoTabs,
            );
            assert_eq!(
                (first.feedback().advance, second.feedback().advance),
                (10.0, 10.0)
            );
            place_and_copy(p, vec![first, second], t::source::Range::new(0, 4), "A B");
        }
    }
}

fn prepare_style_source_seams(
    parts: &[&str],
    direction: t::paragraph::ParagraphBaseDirection,
) -> t::prepared::PreparedText {
    let value = parts.concat();
    let base = style(t::style::WhiteSpace::BreakSpaces, t::style::Wrap::Word);
    let mut request = input(&value, base.clone(), direction);
    request.origins.clear();
    let colors = [
        t::style::Brush::color(1.0, 0.0, 0.0, 1.0),
        t::style::Brush::color(0.0, 1.0, 0.0, 1.0),
        t::style::Brush::color(0.0, 0.0, 1.0, 1.0),
        t::style::Brush::color(1.0, 1.0, 0.0, 1.0),
    ];
    let mut sources = Vec::new();
    let mut at = 0;
    for (index, part) in parts.iter().enumerate() {
        let length = part.encode_utf16().count();
        let aggregate = t::source::Range::new(at, at + length);
        let local = t::source::Range::new(0, length);
        let mut child_style = base.clone();
        child_style.brush = colors[index % colors.len()];
        let mut original = t::source::Source::identified(
            t::source::Text::from(*part),
            t::source::Id::from_u64(640240 + u64::try_from(index).unwrap()),
            23,
        );
        original.span(local, child_style.clone());
        request.source.span(aggregate, child_style);
        request.origins.push(t::paragraph::ParagraphOrigin {
            text_node: Some(
                t::paragraph::CompleteTextNodeContribution::new(original.text().clone(), local)
                    .unwrap(),
            ),
            source: original.clone(),
            local,
            aggregate,
            kind: t::paragraph::ParagraphOriginKind::Authored,
        });
        if index != 0 {
            request.connections.push(t::paragraph::ParagraphConnection {
                at,
                kind: t::paragraph::ParagraphConnectionKind::Continuous,
            });
            request
                .boundary_policies
                .push(t::paragraph::ParagraphBoundaryPolicy {
                    definition: t::paragraph::ParagraphBoundaryDefinition::Connection(index - 1),
                    preceding_wrap: t::style::Wrap::Word,
                    following_wrap: t::style::Wrap::Word,
                    ancestor_wrap: t::style::Wrap::Word,
                });
        }
        sources.push((original, local, aggregate));
        at += length;
    }
    let prepared = prepare_input(request, t::prepared::PreparationOptions::default());
    let retained = prepared.origin_views().collect::<Vec<_>>();
    assert_eq!(retained.len(), sources.len());
    for (view, (original, local, aggregate)) in retained.iter().zip(&sources) {
        assert_eq!(
            view.source(),
            original,
            "retain each different original and computed style"
        );
        assert_eq!(view.local_range(), *local);
        assert_eq!(view.aggregate_range(), *aggregate);
        assert_eq!(view.kind(), t::paragraph::ParagraphOriginKind::Authored);
    }
    prepared
}

struct SeamCase<'a> {
    parts: &'a [&'a str],
    cuts: &'a [usize],
    graphemes: &'a [(usize, usize)],
    advance: f64,
    split_advances: Option<[f64; 2]>,
}

fn check_style_source_seam(case: &SeamCase<'_>, direction: t::paragraph::ParagraphBaseDirection) {
    let value = case.parts.concat();
    let end = value.encode_utf16().count();
    let p = prepare_style_source_seams(case.parts, direction);
    let analyzed = p.analyzed_source();
    let processed = analyzed.processed_source();
    assert_eq!(
        processed.source().text().code_units(),
        value.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        processed.processed_text(),
        &t::source::Text::from(value.as_str())
    );
    // BreakSpaces is identity here, including joiners and source-only ZW. Every
    // original/processed endpoint and both affinities retain exact custody.
    for at in 0..=end {
        for affinity in [t::layout::Affinity::Before, t::layout::Affinity::After] {
            let original =
                t::source::SourcePosition::try_new(processed.source().text(), at).unwrap();
            assert_eq!(
                processed
                    .source_to_processed(original, affinity)
                    .unwrap()
                    .get(),
                at
            );
            assert_eq!(
                processed
                    .processed_to_source(processed.position(at).unwrap(), affinity)
                    .unwrap()
                    .get(),
                at
            );
        }
    }
    let expected_graphemes = case
        .graphemes
        .iter()
        .map(|&(start, end)| t::source::Range::new(start, end))
        .collect::<Vec<_>>();
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        expected_graphemes,
        "authentic grapheme crosses source/style boundaries: {value:?}"
    );
    assert_eq!(
        cuts(&p, t::prepared::PreparedBreakKind::Allowed),
        case.cuts,
        "adopted exact cuts across {direction:?} seams: {:?}",
        case.parts
    );
    assert!(cuts(&p, t::prepared::PreparedBreakKind::Forced).is_empty());
    let full = whole(&p, Some(100.0));
    no_hang(&full, case.advance);
    let brushes = full.run_views().map(|run| run.brush()).collect::<Vec<_>>();
    let first_brush = t::style::Brush::color(1.0, 0.0, 0.0, 1.0);
    assert!(
        brushes.contains(&first_brush),
        "the first actual styled origin reaches finalized runs"
    );
    assert!(
        brushes.iter().any(|brush| *brush != first_brush),
        "a different neighboring style reaches finalized runs"
    );
    assert_eq!(
        glyphs(&full)
            .iter()
            .map(|glyph| f64::from(glyph.1))
            .sum::<f64>(),
        case.advance
    );
    if let Some(expected) = case.split_advances {
        assert_eq!(case.cuts.len(), 1);
        let first_end = offer(&p, t::prepared::PreparedBreakKind::Allowed, case.cuts[0]);
        let second_start = first_end.view().boundary();
        let first = finish(
            &p,
            p.boundary_views().next().unwrap().handle(),
            first_end,
            Some(100.0),
            t::tabs::TabContext::NoTabs,
        );
        let second = finish(
            &p,
            second_start,
            offer(&p, t::prepared::PreparedBreakKind::ParagraphEnd, end),
            Some(100.0),
            t::tabs::TabContext::NoTabs,
        );
        no_hang(&first, expected[0]);
        no_hang(&second, expected[1]);
        place_and_copy(
            p,
            vec![first, second],
            t::source::Range::new(0, end),
            &value,
        );
    } else {
        place_and_copy(p, vec![full], t::source::Range::new(0, end), &value);
    }
}

#[test]
fn break_spaces_protected_gl_wj_zwj_and_combining_units_survive_real_style_source_seams() {
    let cases = [
        SeamCase {
            parts: &["A", "\u{2007}", "B"],
            cuts: &[],
            graphemes: &[(0, 1), (1, 2), (2, 3)],
            advance: 30.0,
            split_advances: None,
        },
        SeamCase {
            parts: &["A", "\u{202f}", "B"],
            cuts: &[],
            graphemes: &[(0, 1), (1, 2), (2, 3)],
            advance: 30.0,
            split_advances: None,
        },
        SeamCase {
            parts: &["\u{2007}", "\u{2060}", "B"],
            cuts: &[],
            graphemes: &[(0, 1), (1, 2), (2, 3)],
            advance: 20.0,
            split_advances: None,
        },
        SeamCase {
            parts: &[" ", "\u{2060}B"],
            cuts: &[],
            graphemes: &[(0, 1), (1, 2), (2, 3)],
            advance: 20.0,
            split_advances: None,
        },
        SeamCase {
            parts: &[" ", "\u{200d}", "\u{301}", "B"],
            cuts: &[],
            graphemes: &[(0, 3), (3, 4)],
            advance: 20.0,
            split_advances: None,
        },
        SeamCase {
            parts: &[" \u{200d}", "\u{301}B"],
            cuts: &[],
            graphemes: &[(0, 3), (3, 4)],
            advance: 20.0,
            split_advances: None,
        },
        SeamCase {
            parts: &["A ", "\u{301}", "B"],
            cuts: &[],
            graphemes: &[(0, 1), (1, 3), (3, 4)],
            advance: 30.0,
            split_advances: None,
        },
        SeamCase {
            parts: &["\u{2007}", "\u{202f}B"],
            cuts: &[],
            graphemes: &[(0, 1), (1, 2), (2, 3)],
            advance: 30.0,
            split_advances: None,
        },
    ];
    for direction in [LTR, RTL] {
        for case in &cases {
            check_style_source_seam(case, direction);
        }
    }
}

#[test]
fn break_spaces_legal_space_and_higher_priority_zw_offers_cross_real_style_source_seams() {
    let cases = [
        SeamCase {
            parts: &[" ", "\u{2007}B"],
            cuts: &[1],
            graphemes: &[(0, 1), (1, 2), (2, 3)],
            advance: 30.0,
            split_advances: Some([10.0, 20.0]),
        },
        SeamCase {
            parts: &[" ", "\u{a0}"],
            cuts: &[1],
            graphemes: &[(0, 1), (1, 2)],
            advance: 20.0,
            split_advances: Some([10.0, 10.0]),
        },
        SeamCase {
            parts: &["A\u{2007}", "\u{200b}", "\u{202f}B"],
            cuts: &[3],
            graphemes: &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)],
            advance: 40.0,
            split_advances: Some([20.0, 20.0]),
        },
    ];
    for direction in [LTR, RTL] {
        for case in &cases {
            check_style_source_seam(case, direction);
        }
    }
}

mod common;
