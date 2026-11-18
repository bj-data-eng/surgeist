#![forbid(unsafe_code)]

//! Declared WordBreak::BreakWord has Normal+OverflowWrap::Anywhere semantics.
//! The independent authority is pinned CSS Text4 section6.1, retained at
//! references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md:2416.
//! Conditional inventories establish eligibility, never consumer fit. Tests
//! separately finalize real Ahem trials and discharge earlier ordinary variants.
//! This new-API target makes no preimplementation RED claim.

use std::sync::Arc;
use surgeist_text as t;

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");
const DIRECTIONS: [t::paragraph::ParagraphBaseDirection; 2] = [
    t::paragraph::ParagraphBaseDirection::LeftToRight,
    t::paragraph::ParagraphBaseDirection::RightToLeft,
];
const OVERFLOWS: [t::style::OverflowWrap; 3] = [
    t::style::OverflowWrap::Normal,
    t::style::OverflowWrap::Anywhere,
    t::style::OverflowWrap::BreakWord,
];

fn style(word: t::style::WordBreak, overflow: t::style::OverflowWrap) -> t::style::Style {
    t::style::Style {
        font: t::style::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::style::LineHeight::Absolute(10.0),
        white_space: t::style::WhiteSpace::Preserve,
        wrap: t::style::Wrap::Word,
        word_break: word,
        overflow_wrap: overflow,
        hyphens: t::hyphenation::Hyphens::None,
        locale: Some("en-US".into()),
        hyphenate_character: t::hyphenation::HyphenateCharacter::String(t::source::Text::from("X")),
        hyphenate_limit_chars: t::hyphenation::HyphenateLimitChars {
            total: t::hyphenation::HyphenationMinimum::Count(0),
            before: t::hyphenation::HyphenationMinimum::Count(0),
            after: t::hyphenation::HyphenationMinimum::Count(0),
        },
        ..t::style::Style::default()
    }
}

fn system() -> t::System {
    let mut system = t::System::default();
    system
        .register_fonts(AHEM.to_vec(), crate::common::FONT_LIMITS)
        .unwrap();
    system
}

fn input(
    value: &str,
    base: t::style::Style,
    direction: t::paragraph::ParagraphBaseDirection,
) -> t::paragraph::ParagraphInput {
    let source = t::source::Source::identified(value, t::source::Id::from_u64(65301), 11);
    let end = source.text().len();
    t::paragraph::ParagraphInput {
        source: source.clone(),
        base_style: base,
        segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
        terminal_edge: t::paragraph::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![t::paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: t::source::Range::new(0, end),
            aggregate: t::source::Range::new(0, end),
            kind: t::paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn prepare_analyzed(
    analyzed: t::analyzed::AnalyzedSource,
    wrap: t::style::Wrap,
) -> t::prepared::PreparedText {
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| t::analyzed::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    system()
        .prepare(&analyzed, t::prepared::PreparationOptions::default())
        .unwrap()
}

fn prepare(request: t::paragraph::ParagraphInput) -> t::prepared::PreparedText {
    let wrap = request.base_style.wrap;
    let source = request.source.clone();
    let p = prepare_analyzed(
        t::analyzed::AnalyzedSource::from_paragraph(
            request,
            t::analyzed::AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        wrap,
    );
    assert_eq!(p.analyzed_source().source(), &source);
    p
}

fn begin(p: &t::prepared::PreparedText) -> t::prepared::PreparedBoundary {
    p.boundary_views().next().unwrap().handle()
}

fn first_after(
    p: &t::prepared::PreparedText,
    start: &t::prepared::PreparedBoundary,
) -> t::prepared::PreparedOpportunity {
    p.opportunity_views()
        .find(|offer| {
            offer.boundary().view().participant_offset() > start.view().participant_offset()
        })
        .unwrap()
        .handle()
}

fn terminal(p: &t::prepared::PreparedText) -> t::prepared::PreparedOpportunity {
    p.opportunity_views()
        .find(|offer| offer.kind() == t::prepared::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle()
}

fn conditional(
    p: &t::prepared::PreparedText,
    purpose: t::conditional::ConditionalPurpose,
) -> Vec<t::conditional::PreparedConditionalOpportunity> {
    let start = begin(p);
    p.source_segments()
        .next()
        .unwrap()
        .conditional_search(
            &start,
            None,
            &first_after(p, &start),
            purpose,
            &[],
            surgeist_text::conditional::ConditionalLineContext::Unframed { inline_boxes: None },
        )
        .and_then(|search| search.begin(&[]))
        .map(|batch| batch.opportunities().to_vec())
        .unwrap()
}

fn conditional_cuts(
    p: &t::prepared::PreparedText,
    purpose: t::conditional::ConditionalPurpose,
) -> Vec<usize> {
    conditional(p, purpose)
        .iter()
        .map(|offer| offer.boundary().view().source_after().get())
        .collect()
}

fn ordinary_cuts(p: &t::prepared::PreparedText) -> Vec<usize> {
    p.opportunity_views()
        .filter(|offer| offer.kind() == t::prepared::PreparedBreakKind::Allowed)
        .map(|offer| offer.boundary().view().source_after().get())
        .collect()
}

fn finish(
    p: &t::prepared::PreparedText,
    start: t::prepared::PreparedBoundary,
    termination: t::selection::PreparedTermination,
    alternative: Option<t::prepared::PreparedAlternative>,
    available: Option<f64>,
) -> t::finalized::FinalizedLine {
    let (end, line_end) = match &termination {
        t::selection::PreparedTermination::Ordinary(offer) => (
            offer.view().boundary(),
            match offer.view().kind() {
                t::prepared::PreparedBreakKind::ParagraphEnd => t::selection::LineEnd::Paragraph,
                t::prepared::PreparedBreakKind::Forced => t::selection::LineEnd::Forced,
                _ => t::selection::LineEnd::Soft,
            },
        ),
        t::selection::PreparedTermination::Conditional(offer) => {
            (offer.boundary(), t::selection::LineEnd::Soft)
        }
        t::selection::PreparedTermination::Measurement(_)
        | t::selection::PreparedTermination::PhraseMeasurement(_) => {
            panic!("these public line trials use issued ordinary or conditional terminations")
        }
    };
    let from = start.view().participant_offset();
    let to = end.view().participant_offset();
    let participants = p
        .participants()
        .skip(from)
        .take(to - from)
        .collect::<Vec<_>>();
    let controls = participants
        .iter()
        .filter_map(|part| match part {
            t::prepared::PreparedParticipant::Control(occurrence) => {
                Some(t::allocation::ControlAllocation {
                    occurrence: occurrence.clone(),
                    strut: None,
                    boundary: None,
                })
            }
            _ => None,
        })
        .collect();
    p.select(t::selection::LineSelectionRequest {
        inline_boxes: None,
        segment: p.source_segments().next().unwrap(),
        start,
        end,
        participants,
        termination,
        alternative,
        incoming: None,
        atomics: vec![],
        controls,
        boundary_context: vec![],
        line_box_struts: Vec::new(),
        line_strut: None,
        context: t::selection::LineContext {
            alignment_insets: surgeist_text::allocation::LineAlignmentInsets::ZERO,
            formatted_line: surgeist_text::selection::FormattedLineContext::default(),
            base_direction: if p.analyzed_source().direction() == t::style::Direction::RightToLeft {
                DIRECTIONS[1]
            } else {
                DIRECTIONS[0]
            },
            line_box_direction: match if p.analyzed_source().direction()
                == t::style::Direction::RightToLeft
            {
                DIRECTIONS[1]
            } else {
                DIRECTIONS[0]
            } {
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
            end: line_end,
            available_inline: available,
            indent: 0.0,
            alignment: t::options::Alignment::Start,
            tabs: t::tabs::TabContext::NoTabs,
            spacing: t::spacing::SpacingPolicy::None,
        },
    })
    .unwrap()
    .finalize(surgeist_text::scope_alignment::LineAllocations {
        spacing: t::spacing::SpacingAllocations::None,
        scopes: surgeist_text::scope_alignment::ScopeAllocations::None,
    })
    .unwrap()
}

fn glyphs(line: &t::finalized::FinalizedLine) -> Vec<(t::source::Range, u32, f32)> {
    let mut result = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        for glyph in run.glyphs() {
            result.push((glyph.range(), glyph.id(), glyph.advance()));
        }
    }
    result.sort_by_key(|row| (row.0.start, row.0.end));
    result
}

#[test]
fn break_word_overrides_every_declared_overflow_for_both_conditional_purposes() {
    for direction in DIRECTIONS {
        for (word, overflow, finite, minimum) in [
            (
                t::style::WordBreak::Normal,
                t::style::OverflowWrap::Normal,
                false,
                false,
            ),
            (
                t::style::WordBreak::Normal,
                t::style::OverflowWrap::BreakWord,
                true,
                false,
            ),
            (
                t::style::WordBreak::Normal,
                t::style::OverflowWrap::Anywhere,
                true,
                true,
            ),
            (
                t::style::WordBreak::BreakWord,
                t::style::OverflowWrap::Normal,
                true,
                true,
            ),
            (
                t::style::WordBreak::BreakWord,
                t::style::OverflowWrap::BreakWord,
                true,
                true,
            ),
            (
                t::style::WordBreak::BreakWord,
                t::style::OverflowWrap::Anywhere,
                true,
                true,
            ),
        ] {
            let p = prepare(input("AAAA", style(word, overflow), direction));
            assert_eq!(p.analyzed_source().style().word_break, word);
            assert_eq!(p.analyzed_source().style().overflow_wrap, overflow);
            assert_eq!(
                p.opportunity_views()
                    .map(|offer| offer.kind())
                    .collect::<Vec<_>>(),
                [t::prepared::PreparedBreakKind::ParagraphEnd]
            );
            assert_eq!(
                conditional_cuts(&p, t::conditional::ConditionalPurpose::Overflow),
                if finite { vec![1, 2, 3] } else { vec![] }
            );
            assert_eq!(
                conditional_cuts(&p, t::conditional::ConditionalPurpose::MinContent),
                if minimum { vec![1, 2, 3] } else { vec![] }
            );
        }
    }
}

#[test]
fn native_real_trials_exhaust_the_only_ordinary_variant_before_two_fitting_pieces() {
    for direction in DIRECTIONS {
        for overflow in OVERFLOWS {
            let p = prepare(input(
                "AAAA",
                style(t::style::WordBreak::BreakWord, overflow),
                direction,
            ));
            let whole = finish(
                &p,
                begin(&p),
                t::selection::PreparedTermination::Ordinary(terminal(&p)),
                None,
                Some(20.0),
            );
            assert_eq!(whole.measurements().fit().advance(), 40.0);
            assert!(
                whole.measurements().fit().advance() > 20.0,
                "actual ordinary search has one meaningful None variant, which does not fit"
            );
            let offers = conditional(&p, t::conditional::ConditionalPurpose::Overflow);
            let cut = offers
                .iter()
                .find(|offer| offer.boundary().view().source_after().get() == 2)
                .unwrap();
            let first = finish(
                &p,
                begin(&p),
                t::selection::PreparedTermination::Conditional(cut.clone()),
                None,
                Some(20.0),
            );
            let second = finish(
                &p,
                cut.boundary(),
                t::selection::PreparedTermination::Ordinary(terminal(&p)),
                None,
                Some(20.0),
            );
            assert_eq!(first.feedback().advance, 20.0);
            assert_eq!(second.feedback().advance, 20.0);
            assert_eq!(
                glyphs(&first),
                [
                    (t::source::Range::new(0, 1), 35, 10.0),
                    (t::source::Range::new(1, 2), 35, 10.0)
                ]
            );
            assert_eq!(
                glyphs(&second),
                [
                    (t::source::Range::new(2, 3), 35, 10.0),
                    (t::source::Range::new(3, 4), 35, 10.0)
                ]
            );
            assert!(first.selected_line().request().alternative.is_none());
            assert!(first.selected_line().request().incoming.is_none());
            let layout = t::layout::Layout::from_finalized_lines(t::placement::PlacementRequest {
                prepared: p.clone(),
                coverage: t::placement::PlacementCoverage::WholeParagraph,
                entries: [first, second]
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
            assert_eq!(
                layout
                    .lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                [t::source::Range::new(0, 2), t::source::Range::new(2, 4)]
            );
            assert_eq!(layout.source().text(), &t::source::Text::from("AAAA"));
            // The same eligible cut does not prove fit in a smaller allocation.
            let too_small = finish(
                &p,
                begin(&p),
                t::selection::PreparedTermination::Conditional(cut.clone()),
                None,
                Some(1.0),
            );
            assert_eq!(too_small.feedback().advance, 20.0);
        }
    }
}

#[test]
fn intrinsic_conditional_pieces_measure_one_cell_while_maximum_keeps_four() {
    for direction in DIRECTIONS {
        for overflow in OVERFLOWS {
            let p = prepare(input(
                "AAAA",
                style(t::style::WordBreak::BreakWord, overflow),
                direction,
            ));
            let mut start = begin(&p);
            let mut advances = Vec::new();
            loop {
                let ceiling = first_after(&p, &start);
                let offers = p
                    .source_segments()
                    .next()
                    .unwrap()
                    .conditional_search(
                        &start,
                        None,
                        &ceiling,
                        t::conditional::ConditionalPurpose::MinContent,
                        &[],
                        surgeist_text::conditional::ConditionalLineContext::Unframed {
                            inline_boxes: None,
                        },
                    )
                    .and_then(|search| search.begin(&[]))
                    .map(|batch| batch.opportunities().to_vec())
                    .unwrap();
                let (termination, end) = offers.first().map_or_else(
                    || {
                        (
                            t::selection::PreparedTermination::Ordinary(ceiling.clone()),
                            ceiling.view().boundary(),
                        )
                    },
                    |offer| {
                        (
                            t::selection::PreparedTermination::Conditional(offer.clone()),
                            offer.boundary(),
                        )
                    },
                );
                let line = finish(&p, start, termination, None, None);
                advances.push(line.measurements().min_content_contribution().advance());
                if end == terminal(&p).view().boundary() {
                    break;
                }
                start = end;
            }
            assert_eq!(advances, [10.0, 10.0, 10.0, 10.0]);
            let maximum = finish(
                &p,
                begin(&p),
                t::selection::PreparedTermination::Ordinary(terminal(&p)),
                None,
                None,
            );
            assert_eq!(
                maximum.measurements().max_content_contribution().advance(),
                40.0
            );
            let distinct = prepare(input(
                "AAAA",
                style(
                    t::style::WordBreak::Normal,
                    t::style::OverflowWrap::BreakWord,
                ),
                direction,
            ));
            assert_eq!(
                conditional_cuts(&distinct, t::conditional::ConditionalPurpose::MinContent),
                []
            );
            let unbroken = finish(
                &distinct,
                begin(&distinct),
                t::selection::PreparedTermination::Ordinary(terminal(&distinct)),
                None,
                None,
            );
            assert_eq!(
                unbroken.measurements().min_content_contribution().advance(),
                40.0
            );
        }
    }
}

#[test]
fn complete_phantom_disposition_authenticates_a_later_conditional_interval() {
    for direction in DIRECTIONS {
        for purpose in [
            t::conditional::ConditionalPurpose::Overflow,
            t::conditional::ConditionalPurpose::MinContent,
        ] {
            let mut base = style(
                t::style::WordBreak::BreakWord,
                t::style::OverflowWrap::Normal,
            );
            base.white_space = t::style::WhiteSpace::Collapse;
            let p = prepare(input(" AAAA", base, direction));
            let start = begin(&p);
            let first = first_after(&p, &start);
            assert_eq!(first.view().kind(), t::prepared::PreparedBreakKind::Allowed);
            assert_eq!(first.view().boundary().view().source_after().get(), 1);
            let ceiling = terminal(&p);
            assert!(
                p.source_segments()
                    .next()
                    .unwrap()
                    .conditional_search(
                        &start,
                        None,
                        &ceiling,
                        purpose,
                        &[],
                        surgeist_text::conditional::ConditionalLineContext::Unframed {
                            inline_boxes: None
                        },
                    )
                    .and_then(|search| search.begin(&[]))
                    .map(|batch| batch.opportunities().to_vec())
                    .is_err(),
                "an earlier real None variant cannot be omitted from the proof"
            );
            let available =
                (purpose == t::conditional::ConditionalPurpose::Overflow).then_some(20.0);
            let phantom = finish(
                &p,
                start.clone(),
                t::selection::PreparedTermination::Ordinary(first),
                None,
                available,
            );
            assert!(!phantom.has_in_flow_content());
            let offers = p
                .source_segments()
                .next()
                .unwrap()
                .conditional_search(
                    &start,
                    None,
                    &ceiling,
                    purpose,
                    &[t::conditional::ConditionalDisposition::Phantom(phantom)],
                    surgeist_text::conditional::ConditionalLineContext::Unframed {
                        inline_boxes: None,
                    },
                )
                .and_then(|search| search.begin(&[]))
                .map(|batch| batch.opportunities().to_vec())
                .unwrap();
            assert_eq!(
                offers
                    .iter()
                    .map(|offer| offer.boundary().view().source_after().get())
                    .collect::<Vec<_>>(),
                [2, 3, 4]
            );
            let selected = finish(
                &p,
                start,
                t::selection::PreparedTermination::Conditional(offers[1].clone()),
                None,
                available,
            );
            assert_eq!(selected.feedback().advance, 20.0);
            assert_eq!(
                glyphs(&selected),
                [
                    (t::source::Range::new(1, 2), 35, 10.0),
                    (t::source::Range::new(2, 3), 35, 10.0)
                ]
            );
        }
    }
}

#[test]
fn declared_tuple_survives_validation_source_spans_and_contextual_capture() {
    for direction in DIRECTIONS {
        for overflow in OVERFLOWS {
            let base = style(t::style::WordBreak::BreakWord, overflow);
            let validated = t::style::ValidatedStyle::try_from(base.clone()).unwrap();
            assert_eq!(validated.authored(), &base);
            let mut request = input("AAAA", base.clone(), direction);
            request
                .source
                .span(t::source::Range::new(0, 4), base.clone());
            let p = prepare(request.clone());
            assert_eq!(p.analyzed_source().source().spans()[0].style(), &base);
            use t::capture::SourceProcessingEvent as E;
            let processed = Arc::new(
                t::capture::process_contextual_source(t::capture::ContextualSourceInput {
                    joint_flows: Vec::new(),
                    scope_alignment_limits: None,
                    computed_line_boundaries: vec![],
                    source: request.source,
                    segment_breaks: request.segment_breaks,
                    envelope: t::capture::SourceProcessingEnvelope::Complete,
                    origins: request.origins,
                    connections: vec![],
                    controls: vec![],
                    boundary_policies: vec![],
                    replacements: vec![],
                    scopes: vec![t::capture::SourceProcessingScope::Block {
                        writing_axis: None,
                        horizontal_inner_direction: None,
                        font_reference: None,
                        topology: t::capture::SourceProcessingTopology::Complete,
                        trim: t::capture::WhiteSpaceTrim::default(),
                        base_style: base.clone(),
                        used_direction: direction,
                    }],
                    paragraphs: vec![t::capture::SourceProcessingParagraph {
                        base_direction: direction,
                        paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
                    }],
                    events: vec![
                        E::EnterScope(0),
                        E::EnterParagraph(0),
                        E::Origin {
                            origin: 0,
                            affinity: t::layout::Affinity::After,
                        },
                        E::ExitParagraph(0),
                        E::ExitScope(0),
                    ],
                })
                .unwrap(),
            );
            let analyzed = t::analyzed::AnalyzedSource::from_processed_paragraph(
                processed.paragraphs().next().unwrap(),
                t::analyzed::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            let captured = prepare_analyzed(analyzed, t::style::Wrap::Word);
            assert_eq!(
                captured.captured_owner().unwrap().domain().style(),
                Some(&base)
            );
            assert_eq!(captured.analyzed_source().style(), &base);
            assert_eq!(
                conditional_cuts(&captured, t::conditional::ConditionalPurpose::MinContent),
                [1, 2, 3]
            );
        }
    }
}

#[test]
fn conditional_seam_uses_actual_logical_before_owner_in_both_directions() {
    for direction in DIRECTIONS {
        for before_alias in [false, true] {
            let normal = style(t::style::WordBreak::Normal, t::style::OverflowWrap::Normal);
            let alias = style(
                t::style::WordBreak::BreakWord,
                t::style::OverflowWrap::Normal,
            );
            let mut request = input("AAAA", normal.clone(), direction);
            request.source.span(
                t::source::Range::new(0, 2),
                if before_alias {
                    alias.clone()
                } else {
                    normal.clone()
                },
            );
            request.source.span(
                t::source::Range::new(2, 4),
                if before_alias { normal } else { alias },
            );
            request.origins = [t::source::Range::new(0, 2), t::source::Range::new(2, 4)]
                .into_iter()
                .map(|range| t::paragraph::ParagraphOrigin {
                    text_node: None,
                    source: request.source.clone(),
                    local: range,
                    aggregate: range,
                    kind: t::paragraph::ParagraphOriginKind::Authored,
                })
                .collect();
            request.connections.push(t::paragraph::ParagraphConnection {
                at: 2,
                kind: t::paragraph::ParagraphConnectionKind::Continuous,
            });
            request
                .boundary_policies
                .push(t::paragraph::ParagraphBoundaryPolicy {
                    definition: t::paragraph::ParagraphBoundaryDefinition::Connection(0),
                    preceding_wrap: t::style::Wrap::Word,
                    following_wrap: t::style::Wrap::Word,
                    ancestor_wrap: t::style::Wrap::Word,
                });
            let p = prepare(request);
            assert_eq!(ordinary_cuts(&p), []);
            for purpose in [
                t::conditional::ConditionalPurpose::Overflow,
                t::conditional::ConditionalPurpose::MinContent,
            ] {
                assert_eq!(
                    conditional_cuts(&p, purpose),
                    if before_alias { vec![1, 2] } else { vec![3] },
                    "logical-before ownership before_alias={before_alias} {direction:?}"
                );
            }
        }
    }
}

#[test]
fn conditional_typographic_and_wrapping_guards_are_preserved() {
    for direction in DIRECTIONS {
        for (value, expected) in [
            ("A\u{301}A", vec![2]),
            ("\u{1f600}A", vec![2]),
            ("\u{1f469}\u{200d}\u{1f4bb}A", vec![5]),
            ("\u{1f1fa}\u{1f1f8}\u{1f1e8}\u{1f1e6}", vec![4]),
        ] {
            let p = prepare(input(
                value,
                style(
                    t::style::WordBreak::BreakWord,
                    t::style::OverflowWrap::Normal,
                ),
                direction,
            ));
            let actual = conditional_cuts(&p, t::conditional::ConditionalPurpose::Overflow);
            let control = prepare(input(
                value,
                style(
                    t::style::WordBreak::Normal,
                    t::style::OverflowWrap::Anywhere,
                ),
                direction,
            ));
            assert_eq!(
                actual,
                conditional_cuts(&control, t::conditional::ConditionalPurpose::Overflow)
            );
            // Emoji/RI already have an ordinary seam. Conditional queries stop
            // before that authentic ceiling rather than duplicating its offer.
            let mut all = ordinary_cuts(&p);
            all.extend(actual);
            all.sort_unstable();
            all.dedup();
            assert_eq!(
                all, expected,
                "all actual soft cuts retain whole EGCs: {value:?}"
            );
        }
        let mut nowrap = style(
            t::style::WordBreak::BreakWord,
            t::style::OverflowWrap::Normal,
        );
        nowrap.wrap = t::style::Wrap::None;
        let mut request = input("AAAA", nowrap, direction);
        request.source.span(
            t::source::Range::new(0, 4),
            style(
                t::style::WordBreak::BreakWord,
                t::style::OverflowWrap::Normal,
            ),
        );
        let p = prepare(request);
        assert_eq!(
            conditional_cuts(&p, t::conditional::ConditionalPurpose::Overflow),
            []
        );
        assert_eq!(
            conditional_cuts(&p, t::conditional::ConditionalPurpose::MinContent),
            []
        );
        let mut disconnected = input(
            "AAAA",
            style(
                t::style::WordBreak::BreakWord,
                t::style::OverflowWrap::Normal,
            ),
            direction,
        );
        disconnected.origins = [t::source::Range::new(0, 2), t::source::Range::new(2, 4)]
            .into_iter()
            .map(|range| t::paragraph::ParagraphOrigin {
                text_node: None,
                source: disconnected.source.clone(),
                local: range,
                aggregate: range,
                kind: t::paragraph::ParagraphOriginKind::Authored,
            })
            .collect();
        disconnected
            .connections
            .push(t::paragraph::ParagraphConnection {
                at: 2,
                kind: t::paragraph::ParagraphConnectionKind::Disconnected,
            });
        disconnected
            .boundary_policies
            .push(t::paragraph::ParagraphBoundaryPolicy {
                definition: t::paragraph::ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: t::style::Wrap::Word,
                following_wrap: t::style::Wrap::Word,
                ancestor_wrap: t::style::Wrap::Word,
            });
        let p = prepare(disconnected);
        assert_eq!(
            conditional_cuts(&p, t::conditional::ConditionalPurpose::Overflow),
            [1, 3],
            "the absent text connection never gains a conditional cut"
        );
        for (value, delimiter) in [
            ("AA\nAA", None),
            ("AA\r\nAA", Some(t::source::Range::new(2, 4))),
        ] {
            let mut request = input(
                value,
                style(
                    t::style::WordBreak::BreakWord,
                    t::style::OverflowWrap::Normal,
                ),
                direction,
            );
            if let Some(range) = delimiter {
                request.segment_breaks = t::processing::DocumentSegmentBreaks::Defined(vec![range]);
            }
            let p = prepare(request);
            assert_eq!(
                first_after(&p, &begin(&p)).view().kind(),
                t::prepared::PreparedBreakKind::Forced
            );
            assert_eq!(
                conditional_cuts(&p, t::conditional::ConditionalPurpose::Overflow),
                [1]
            );
        }
        let mut request = input(
            "AAAA",
            style(
                t::style::WordBreak::BreakWord,
                t::style::OverflowWrap::Normal,
            ),
            direction,
        );
        request.controls.push(t::paragraph::ParagraphControl {
            at: 2,
            kind: t::paragraph::ParagraphControlKind::ForcedBreak,
        });
        request.anchor_order.push(t::paragraph::ParagraphAnchor {
            item: t::paragraph::ParagraphAnchorItem::Control(0),
            affinity: t::layout::Affinity::After,
        });
        let p = prepare(request);
        assert_eq!(
            conditional_cuts(&p, t::conditional::ConditionalPurpose::Overflow),
            [1]
        );
    }
}

#[test]
fn break_word_preserves_normal_hyphenation_without_indication_on_emergency_cuts() {
    for direction in DIRECTIONS {
        for overflow in OVERFLOWS {
            for indication in ["", "X"] {
                let mut base = style(t::style::WordBreak::BreakWord, overflow);
                base.hyphens = t::hyphenation::Hyphens::Manual;
                base.hyphenate_character =
                    t::hyphenation::HyphenateCharacter::String(t::source::Text::from(indication));
                let p = prepare(input("AA\u{ad}AA", base, direction));
                assert_eq!(
                    ordinary_cuts(&p),
                    [],
                    "effective Normal does not acquire BreakAll's ordinary after-SHY cause"
                );
                let alternatives = p.alternative_views().collect::<Vec<_>>();
                assert_eq!(alternatives.len(), 1);
                assert_eq!(
                    alternatives[0].source_range(),
                    Some(t::source::Range::new(2, 3))
                );
                assert_eq!(alternatives[0].text(), &t::source::Text::from(indication));
                let offers = conditional(&p, t::conditional::ConditionalPurpose::Overflow);
                let first = offers
                    .iter()
                    .find(|offer| offer.boundary().view().source_after().get() == 1)
                    .unwrap();
                let line = finish(
                    &p,
                    begin(&p),
                    t::selection::PreparedTermination::Conditional(first.clone()),
                    None,
                    Some(10.0),
                );
                assert_eq!(glyphs(&line), [(t::source::Range::new(0, 1), 35, 10.0)]);
            }
            let mut base = style(t::style::WordBreak::BreakWord, overflow);
            base.hyphens = t::hyphenation::Hyphens::Auto;
            let p = prepare(input("associate", base, direction));
            assert_eq!(
                p.alternative_views()
                    .filter(|alt| matches!(
                        alt.definition(),
                        t::prepared::PreparedAlternativeDefinition::Automatic { .. }
                    ))
                    .map(|alt| alt.insertion_anchor().get())
                    .collect::<Vec<_>>(),
                [2, 4],
                "retained en-US exception as-so-ciate supplies independent cuts2/4"
            );
        }
    }
}

#[test]
fn line_break_anywhere_remains_ordinary_while_break_word_retains_normal_cjk_rules() {
    for direction in DIRECTIONS {
        for (line_break, expected) in [
            (t::style::LineBreak::Strict, vec![]),
            (t::style::LineBreak::Normal, vec![]),
            (t::style::LineBreak::Loose, vec![1]),
        ] {
            let mut base = style(
                t::style::WordBreak::BreakWord,
                t::style::OverflowWrap::Normal,
            );
            base.line_break = line_break;
            base.locale = Some("ja".into());
            assert_eq!(
                ordinary_cuts(&prepare(input("漢ぁ", base, direction))),
                expected
            );
        }
        let mut base = style(
            t::style::WordBreak::BreakWord,
            t::style::OverflowWrap::Normal,
        );
        base.line_break = t::style::LineBreak::Anywhere;
        assert_eq!(
            ordinary_cuts(&prepare(input("AAAA", base, direction))),
            [1, 2, 3]
        );
    }
}

fn raw_facts(
    layout: &t::layout::Layout,
) -> (Vec<t::source::Range>, Vec<(t::source::Range, u32, f32)>) {
    let mut glyphs = Vec::new();
    for run in layout.glyph_runs() {
        assert_eq!(run.font().bytes(), AHEM);
        for glyph in run.glyphs() {
            glyphs.push((glyph.range(), glyph.id(), glyph.advance()));
        }
    }
    glyphs.sort_by_key(|row| (row.0.start, row.0.end));
    (
        layout.lines().iter().map(|line| line.range()).collect(),
        glyphs,
    )
}

#[test]
fn standalone_declared_tuple_matrix_has_real_widths_and_separate_cache_entries() {
    let mut system = system();
    let source = t::source::Source::identified("AAAA", t::source::Id::from_u64(65302), 17);
    let mut frozen = Vec::new();
    // Raw Standalone accepts Direction::Auto; explicit paragraph directions
    // are independently exercised through native ParagraphInput above.
    let direction = t::style::Direction::Auto;
    for (word, overflow, breaks) in [
        (
            t::style::WordBreak::Normal,
            t::style::OverflowWrap::Normal,
            false,
        ),
        (
            t::style::WordBreak::Normal,
            t::style::OverflowWrap::Anywhere,
            true,
        ),
        (
            t::style::WordBreak::Normal,
            t::style::OverflowWrap::BreakWord,
            true,
        ),
        (
            t::style::WordBreak::BreakWord,
            t::style::OverflowWrap::Normal,
            true,
        ),
        (
            t::style::WordBreak::BreakWord,
            t::style::OverflowWrap::Anywhere,
            true,
        ),
        (
            t::style::WordBreak::BreakWord,
            t::style::OverflowWrap::BreakWord,
            true,
        ),
    ] {
        let mut base = style(word, overflow);
        base.direction = direction;
        for width in [Some(20.0), Some(40.0), None] {
            let options = t::options::Options {
                width,
                quantize: false,
                ..t::options::Options::default()
            };
            let before = system.stats();
            let actual = system
                .layout(source.clone(), base.clone(), options)
                .unwrap();
            let expected = if breaks && width == Some(20.0) {
                vec![t::source::Range::new(0, 2), t::source::Range::new(2, 4)]
            } else {
                vec![t::source::Range::new(0, 4)]
            };
            let facts = raw_facts(&actual);
            assert_eq!(
                facts.0, expected,
                "declared {word:?}/{overflow:?} {direction:?} width={width:?}"
            );
            assert_eq!(
                facts.1,
                (0..4)
                    .map(|at| (t::source::Range::new(at, at + 1), 35, 10.0))
                    .collect::<Vec<_>>()
            );
            assert_eq!(actual.source(), &source);
            assert_eq!(system.stats().layout_misses(), before.layout_misses() + 1);
            let hit = system
                .layout(source.clone(), base.clone(), options)
                .unwrap();
            assert_eq!(hit.key(), actual.key());
            assert_eq!(raw_facts(&hit), facts);
            assert_eq!(system.stats().layout_hits(), before.layout_hits() + 1);
            assert!(
                frozen.iter().all(|(key, _, _)| *key != actual.key()),
                "declared tuples retain distinct cache identity even when effective behavior agrees"
            );
            frozen.push((actual.key(), actual, facts));
        }
    }
    for (key, layout, facts) in frozen {
        assert_eq!(layout.key(), key);
        assert_eq!(raw_facts(&layout), facts);
    }
}

#[test]
fn standalone_prefers_ordinary_space_and_retains_raw_following_owner_parity() {
    let mut system = system();
    for overflow in OVERFLOWS {
        let direction = t::style::Direction::Auto;
        let mut base = style(t::style::WordBreak::BreakWord, overflow);
        base.direction = direction;
        for (width, expected) in [
            (
                40.0,
                vec![t::source::Range::new(0, 3), t::source::Range::new(3, 5)],
            ),
            (50.0, vec![t::source::Range::new(0, 5)]),
        ] {
            let layout = system
                .layout(
                    t::source::Source::new("AA AA"),
                    base.clone(),
                    t::options::Options {
                        width: Some(width),
                        quantize: false,
                        ..t::options::Options::default()
                    },
                )
                .unwrap();
            assert_eq!(
                raw_facts(&layout).0,
                expected,
                "ordinary SPACE wins before conditional interior"
            );
        }
        for before_alias in [false, true] {
            let mut aliased = t::source::Source::new("AAAA");
            let mut explicit = t::source::Source::new("AAAA");
            let range = if before_alias {
                t::source::Range::new(0, 2)
            } else {
                t::source::Range::new(2, 4)
            };
            aliased.span(range, base.clone());
            let mut equivalent = base.clone();
            equivalent.word_break = t::style::WordBreak::Normal;
            equivalent.overflow_wrap = t::style::OverflowWrap::Anywhere;
            explicit.span(range, equivalent);
            let mut normal = style(t::style::WordBreak::Normal, t::style::OverflowWrap::Normal);
            normal.direction = direction;
            let options = t::options::Options {
                width: Some(20.0),
                quantize: false,
                ..t::options::Options::default()
            };
            let alias = system.layout(aliased, normal.clone(), options).unwrap();
            let control = system.layout(explicit, normal, options).unwrap();
            assert_eq!(
                raw_facts(&alias),
                raw_facts(&control),
                "characterized raw following-owner parity does not assert native before-owner behavior"
            );
        }
    }
}

mod common;
