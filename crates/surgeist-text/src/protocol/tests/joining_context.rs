use super::*;
use crate::{Range, WordBreak};

fn breaking_style() -> Style {
    Style {
        word_break: WordBreak::BreakAll,
        ..style()
    }
}
fn arabic_system() -> System {
    let mut value = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    value
        .register_fonts(crate::shaping_test_fonts::arabic_nonjoining_ahem())
        .unwrap();
    value
}
fn placed_forms(layout: &crate::Layout) -> Vec<(Range, u32)> {
    let fixture = crate::shaping_test_fonts::arabic_nonjoining_ahem();
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), fixture);
        assert_eq!(run.font_size(), 20.0);
        assert_eq!(
            run.face().catalogue_identity(),
            layout.key().font_catalogue()
        );
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(instance.font_size(), 20.0);
    }

    let mut result: Vec<_> = layout
        .glyph_runs()
        .iter()
        .flat_map(|run| run.glyphs().iter().map(|glyph| (glyph.range(), glyph.id())))
        .collect();
    result.sort_by_key(|(range, _)| (range.start, range.end));
    result
}
#[test]
fn standalone_metric_policies_keep_unforced_arabic_joining() {
    let mut actual = Vec::new();
    for quantize in [false, true] {
        let mut system = arabic_system();
        let source = Source::new("\u{628}\u{628}\u{628}");
        let control = system
            .layout(
                source.clone(),
                breaking_style(),
                crate::Options {
                    quantize,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        // Existing Arabic fixture's literal init/medi/fina forms, in source order.
        assert_eq!(
            placed_forms(&control),
            [
                (Range::new(0, 1), 100),
                (Range::new(1, 2), 277),
                (Range::new(2, 3), 82)
            ]
        );
        let split = system
            .layout(
                source.clone(),
                breaking_style(),
                crate::Options {
                    width: Some(20.0),
                    quantize,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        assert_eq!(split.source(), &source);
        actual.push((quantize, split.lines().len(), placed_forms(&split)));
    }
    assert_eq!(
        actual,
        [
            (
                false,
                3,
                vec![
                    (Range::new(0, 1), 100),
                    (Range::new(1, 2), 277),
                    (Range::new(2, 3), 82)
                ]
            ),
            (
                true,
                3,
                vec![
                    (Range::new(0, 1), 100),
                    (Range::new(1, 2), 277),
                    (Range::new(2, 3), 82)
                ]
            ),
        ]
    );
}

fn native_input(source: Source) -> ParagraphInput {
    let length = source.text().len();
    ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: breaking_style(),
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::RightToLeft,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, length),
            aggregate: Range::new(0, length),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}
fn native_prepare(system: &mut System, input: ParagraphInput) -> PreparedText {
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
fn ending(prepared: &PreparedText, kind: PreparedBreakKind, before: usize) -> PreparedOpportunity {
    prepared
        .opportunity_views()
        .find(|offer| {
            offer.kind() == kind && offer.boundary().view().source_before().get() == before
        })
        .unwrap()
        .handle()
}
fn selected_line(
    prepared: &PreparedText,
    start: PreparedBoundary,
    ending: PreparedOpportunity,
) -> crate::Result<FinalizedLine> {
    let mut value = request(prepared);
    value.start = start;
    value.end = ending.view().boundary();
    let from = value.start.view().participant_offset();
    let to = value.end.view().participant_offset();
    value.participants = prepared.participants().skip(from).take(to - from).collect();
    value.controls = value
        .participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Control(control) => Some(ControlAllocation {
                occurrence: control.clone(),
                strut: None,
            }),
            _ => None,
        })
        .collect();
    value.atomics = value
        .participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Atomic(atomic) => Some(AtomicAllocation {
                occurrence: atomic.clone(),
                allocation: AtomicLineAllocation::OutOfFlow,
            }),
            _ => None,
        })
        .collect();
    value.context.base_direction = ParagraphBaseDirection::RightToLeft;
    value.context.start = if from == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    value.context.end = match ending.view().kind() {
        PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary => LineEnd::Soft,
        PreparedBreakKind::Forced => LineEnd::Forced,
        PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
    };
    value.termination = crate::PreparedTermination::Ordinary(ending);
    prepared.select(value)?.finalize(SpacingAllocations::None)
}
fn source_glyphs(line: &FinalizedLine, fixture: &[u8]) -> Vec<(Range, u32, f32)> {
    let mut result = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), fixture);
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        let metrics = instance
            .metrics(line.prepared_text().font_catalogue_identity())
            .unwrap();
        assert_eq!(metrics.ascent(), f64::from(run.font_size()) * 0.8);
        assert_eq!(metrics.descent(), -f64::from(run.font_size()) * 0.2);
        result.extend(
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.id(), run.font_size())),
        );
    }
    result.sort_by_key(|(range, _, _)| (range.start, range.end));
    result
}
fn beh_forms(line: &FinalizedLine) -> Vec<(Range, u32, f32)> {
    let prepared = line.prepared_text();
    let source = prepared.analyzed_source();
    source_glyphs(line, &crate::shaping_test_fonts::arabic_nonjoining_ahem())
        .into_iter()
        .filter(|(range, _, _)| {
            source.source().text().code_units().get(range.start) == Some(&0x628)
        })
        .collect()
}
#[test]
fn joining_context_crosses_effective_instance_boundaries() {
    let mut source = Source::new("\u{628}\u{628}\u{628}");
    source.span(
        Range::new(1, 2),
        Style {
            size: 30.0,
            ..breaking_style()
        },
    );
    let mut system = arabic_system();
    let prepared = native_prepare(&mut system, native_input(source));
    drop(system);
    let first = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        ending(&prepared, PreparedBreakKind::Allowed, 1),
    )
    .unwrap();
    let continuation = selected_line(
        &prepared,
        first.selected_line().request().end.clone(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 3),
    )
    .unwrap();
    assert_eq!(beh_forms(&first), [(Range::new(0, 1), 100, 20.0)]);
    assert_eq!(
        beh_forms(&continuation),
        [(Range::new(1, 2), 277, 30.0), (Range::new(2, 3), 82, 20.0)]
    );
}
#[test]
fn actual_forced_and_disconnected_boundaries_stop_joining_context() {
    let mut system = arabic_system();
    let mut input = native_input(Source::new("\u{628}\u{628}"));
    input.controls.push(paragraph::ParagraphControl {
        at: 1,
        kind: paragraph::ParagraphControlKind::ForcedBreak,
    });
    input.anchor_order.push(paragraph::ParagraphAnchor {
        item: paragraph::ParagraphAnchorItem::Control(0),
        affinity: crate::Affinity::After,
    });
    let prepared = native_prepare(&mut system, input);
    let first = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        ending(&prepared, PreparedBreakKind::Forced, 1),
    )
    .unwrap();
    let second = selected_line(
        &prepared,
        first.selected_line().request().end.clone(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 2),
    )
    .unwrap();
    assert_eq!(beh_forms(&first), [(Range::new(0, 1), 35, 20.0)]);
    assert_eq!(beh_forms(&second), [(Range::new(1, 2), 35, 20.0)]);
    let mut input = native_input(Source::new("\u{628}\u{628}"));
    input.origins[0].local = Range::new(0, 1);
    input.origins[0].aggregate = Range::new(0, 1);
    input.origins.push(paragraph::ParagraphOrigin {
        text_node: None,
        source: input.source.clone(),
        local: Range::new(1, 2),
        aggregate: Range::new(1, 2),
        kind: paragraph::ParagraphOriginKind::Authored,
    });
    input.connections.push(paragraph::ParagraphConnection {
        at: 1,
        kind: paragraph::ParagraphConnectionKind::Disconnected,
    });
    let prepared = native_prepare(&mut system, input);
    let whole = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 2),
    )
    .unwrap();
    assert_eq!(
        beh_forms(&whole),
        [(Range::new(0, 1), 35, 20.0), (Range::new(1, 2), 35, 20.0)]
    );
    let prepared = native_prepare(&mut system, native_input(Source::new("\u{628}\n\u{628}")));
    let first = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        ending(&prepared, PreparedBreakKind::Forced, 2),
    )
    .unwrap();
    let second = selected_line(
        &prepared,
        first.selected_line().request().end.clone(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 3),
    )
    .unwrap();
    assert_eq!(beh_forms(&first), [(Range::new(0, 1), 35, 20.0)]);
    assert_eq!(beh_forms(&second), [(Range::new(2, 3), 35, 20.0)]);
}
#[test]
fn optional_ligature_cut_shapes_only_actual_selected_atoms() {
    let fixture = crate::shaping_test_fonts::ligature_ahem();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(fixture.clone()).unwrap();
    let prepared = native_prepare(&mut system, native_input(Source::new("AA")));
    let whole = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 2),
    )
    .unwrap();
    assert_eq!(
        source_glyphs(&whole, &fixture),
        [(Range::new(0, 2), 58, 20.0)]
    );
    assert_eq!(whole.feedback().advance, 20.0);
    let first = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        ending(&prepared, PreparedBreakKind::Allowed, 1),
    )
    .unwrap();
    let second = selected_line(
        &prepared,
        first.selected_line().request().end.clone(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 2),
    )
    .unwrap();
    assert_eq!(
        source_glyphs(&first, &fixture),
        [(Range::new(0, 1), 35, 20.0)]
    );
    assert_eq!(
        source_glyphs(&second, &fixture),
        [(Range::new(1, 2), 35, 20.0)]
    );
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (20.0, 20.0)
    );
}
#[test]
fn full_captured_context_reports_real_context_limit_without_mutating_prior_results() {
    let mut system = arabic_system();
    // Six genuine transparent Arabic Letter Marks exceed the provider's five
    // pre-context cells; supplying all context must preserve its actual refusal.
    let source = Source::new(format!("\u{628}{}\u{628}", "\u{61c}".repeat(6)));
    let mut input = native_input(source.clone());
    input.controls.push(paragraph::ParagraphControl {
        at: 1,
        kind: paragraph::ParagraphControlKind::OptionalBreak {
            wrap: crate::Wrap::Word,
        },
    });
    input.anchor_order.push(paragraph::ParagraphAnchor {
        item: paragraph::ParagraphAnchorItem::Control(0),
        affinity: crate::Affinity::After,
    });
    let prepared = native_prepare(&mut system, input);
    let before = system.stats();
    let catalogue = system.font_catalogue_identity().clone();
    let whole = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 8),
    )
    .unwrap();
    let original = beh_forms(&whole);
    assert_eq!(
        original,
        [(Range::new(0, 1), 100, 20.0), (Range::new(7, 8), 82, 20.0)]
    );
    let soft = ending(&prepared, PreparedBreakKind::Allowed, 7);
    let error = selected_line(
        &prepared,
        soft.view().boundary(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 8),
    )
    .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&crate::ErrorDetail::ShapingRefused {
            reason: crate::ShapingRefusalReason::ContextLimit
        })
    );
    let post_error = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        ending(&prepared, PreparedBreakKind::Allowed, 1),
    )
    .unwrap_err();
    assert_eq!(
        post_error.detail(),
        Some(&crate::ErrorDetail::ShapingRefused {
            reason: crate::ShapingRefusalReason::ContextLimit,
        })
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(beh_forms(&whole), original);
    assert_eq!(system.stats(), before);
    assert_eq!(system.font_catalogue_identity(), &catalogue);
    let retry = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 8),
    )
    .unwrap();
    assert_eq!(beh_forms(&retry), original);
}

#[test]
fn deleted_source_text_does_not_reenter_processed_joining_context() {
    let mut input = native_input(Source::new("\u{628} \u{628}"));
    input.base_style.white_space = WhiteSpace::Discard;
    let mut system = arabic_system();
    let prepared = native_prepare(&mut system, input);
    let first = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        ending(&prepared, PreparedBreakKind::Allowed, 1),
    )
    .unwrap();
    let second = selected_line(
        &prepared,
        first.selected_line().request().end.clone(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 3),
    )
    .unwrap();
    assert_eq!(beh_forms(&first), [(Range::new(0, 1), 100, 20.0)]);
    assert_eq!(beh_forms(&second), [(Range::new(2, 3), 82, 20.0)]);
    assert_eq!(
        prepared
            .analyzed_source()
            .processed_source()
            .processed_text()
            .code_units(),
        &[0x628, 0x628]
    );
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        &[0x628, 0x20, 0x628]
    );
}
#[test]
fn child_domain_hole_is_excluded_from_parent_joining_context() {
    use crate::{
        Affinity, ContextualSourceInput, DocumentSegmentBreaks, Id, InlineBox, InlineBoxKind,
        ParagraphBoundaryDefinition, ParagraphBoundaryPolicy, ParagraphConnectionKind,
        ParagraphOrigin, ParagraphOriginKind, Size, SourceProcessingConnection,
        SourceProcessingEnvelope, SourceProcessingEvent, SourceProcessingParagraph,
        SourceProcessingScope, WhiteSpaceTrim, Wrap, process_contextual_source,
    };
    let mut source = Source::new("\u{628}I\u{628}");
    source.inline_box(InlineBox::new(
        Id::from_u64(1),
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
                text_node: None,
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
        replacements: vec![],
        boundary_policies: vec![ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(0),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        }],
        scopes: vec![
            SourceProcessingScope::Block {
                used_direction: crate::ParagraphBaseDirection::LeftToRight,
                trim: WhiteSpaceTrim::default(),
                base_style: breaking_style(),
            },
            SourceProcessingScope::OutOfFlowBlock {
                used_direction: crate::ParagraphBaseDirection::LeftToRight,
                trim: WhiteSpaceTrim::default(),
                base_style: style(),
                atomic: 0,
            },
        ],
        paragraphs: vec![
            SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::RightToLeft,
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
    let processed = std::sync::Arc::new(process_contextual_source(capture).unwrap());
    let parent =
        AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap()).unwrap();
    let mut system = arabic_system();
    let prepared = system
        .prepare(&parent, PreparationOptions::default())
        .unwrap();
    let soft = prepared
        .opportunity_views()
        .find(|offer| {
            offer.kind() == PreparedBreakKind::Allowed
                && offer.boundary().view().source_before().get() == 1
                && offer.boundary().view().source_after().get() == 2
        })
        .unwrap()
        .handle();
    let first = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        soft,
    )
    .unwrap();
    let second = selected_line(
        &prepared,
        first.selected_line().request().end.clone(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 3),
    )
    .unwrap();
    assert_eq!(beh_forms(&first), [(Range::new(0, 1), 100, 20.0)]);
    assert_eq!(beh_forms(&second), [(Range::new(2, 3), 82, 20.0)]);
    assert_eq!(
        source_glyphs(&first, &crate::shaping_test_fonts::arabic_nonjoining_ahem()).len(),
        1
    );
    assert_eq!(
        source_glyphs(
            &second,
            &crate::shaping_test_fonts::arabic_nonjoining_ahem()
        )
        .len(),
        1
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
}

fn discretionary_input(
    source: Source,
    definition: paragraph::ParagraphReplacementDefinition,
    text: &str,
) -> ParagraphInput {
    let mut input = native_input(source);
    input.base_style.word_break = WordBreak::Normal;
    let anchor = match definition {
        paragraph::ParagraphReplacementDefinition::Source { source, .. } => source.end,
        paragraph::ParagraphReplacementDefinition::Control { .. } => {
            input.controls.push(paragraph::ParagraphControl {
                at: 3,
                kind: paragraph::ParagraphControlKind::DiscretionaryBreak {
                    style: Box::new(input.base_style.clone()),
                },
            });
            input.anchor_order.push(paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::Control(0),
                affinity: crate::Affinity::After,
            });
            3
        }
    };
    input.replacements.push(
        paragraph::ParagraphReplacement::resolved_native_logical_before(
            definition,
            anchor,
            vec![crate::Text::from(text)],
        )
        .unwrap(),
    );
    input
}

fn discretionary_line(
    prepared: &PreparedText,
    start: PreparedBoundary,
) -> crate::Result<FinalizedLine> {
    let offer = prepared
        .opportunity_views()
        .find(|offer| {
            offer.kind() == PreparedBreakKind::Discretionary
                && offer.boundary().view().participant_offset() > start.view().participant_offset()
        })
        .unwrap();
    let mut value = request(prepared);
    value.start = start;
    value.end = offer.boundary();
    let from = value.start.view().participant_offset();
    let to = value.end.view().participant_offset();
    value.participants = prepared.participants().skip(from).take(to - from).collect();
    value.controls = value
        .participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Control(control) => Some(ControlAllocation {
                occurrence: control.clone(),
                strut: None,
            }),
            _ => None,
        })
        .collect();
    value.termination = PreparedTermination::Ordinary(offer.handle());
    value.alternative = Some(offer.alternatives().next().unwrap().handle());
    value.context.base_direction = ParagraphBaseDirection::RightToLeft;
    value.context.start = if from == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    value.context.end = LineEnd::Soft;
    prepared.select(value)?.finalize(SpacingAllocations::None)
}

type OriginalBehForms = Vec<(Range, u32, f32)>;

fn original_beh_forms(line: &FinalizedLine) -> OriginalBehForms {
    // A replacement can share the following Beh's numeric insertion anchor;
    // only positive original ranges can qualify an original Beh's form.
    beh_forms(line)
        .into_iter()
        .filter(|(range, _, _)| !range.is_empty())
        .collect()
}

#[test]
fn short_discretionary_arabic_keeps_unbroken_forms_without_an_admitted_ending() {
    let source = Source::new("\u{628}\u{628}\u{ad}\u{628}");
    let input = discretionary_input(
        source.clone(),
        paragraph::ParagraphReplacementDefinition::Source {
            origin: 0,
            source: Range::new(2, 3),
        },
        "-",
    );
    let prepared = native_prepare(&mut arabic_system(), input);
    assert!(
        !prepared
            .opportunity_views()
            .any(|v| v.kind() == PreparedBreakKind::Discretionary)
    );
    let whole = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 4),
    )
    .unwrap();
    assert_eq!(
        original_beh_forms(&whole),
        [
            (Range::new(0, 1), 100, 20.0),
            (Range::new(1, 2), 277, 20.0),
            (Range::new(3, 4), 82, 20.0)
        ]
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
}

fn selected_source_shy_forms(text: &str) -> (OriginalBehForms, OriginalBehForms) {
    let source = Source::new("\u{621}\u{628}\u{628}\u{ad}\u{628}\u{621}");
    let input = discretionary_input(
        source.clone(),
        paragraph::ParagraphReplacementDefinition::Source {
            origin: 0,
            source: Range::new(3, 4),
        },
        text,
    );
    let mut system = arabic_system();
    let prepared = native_prepare(&mut system, input);
    let start = prepared.boundary_views().next().unwrap().handle();
    let whole = selected_line(
        &prepared,
        start.clone(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 6),
    )
    .unwrap();
    // Literal fixture GSUB isol/init/medi/fina=35/100/277/82. Text4's
    // word-break-shaping rule preserves these forms at hyphenation breaks.
    assert_eq!(
        original_beh_forms(&whole),
        [
            (Range::new(1, 2), 100, 20.0),
            (Range::new(2, 3), 277, 20.0),
            (Range::new(4, 5), 82, 20.0)
        ]
    );
    assert!(whole.selected_line().request().alternative.is_none());
    drop(system);
    let first = discretionary_line(&prepared, start).unwrap();
    let continuation = selected_line(
        &prepared,
        first.selected_line().request().end.clone(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 6),
    )
    .unwrap();
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert!(first.selected_line().request().alternative.is_some());
    (
        original_beh_forms(&first),
        original_beh_forms(&continuation),
    )
}

#[test]
fn selected_source_shy_nonempty_replacement_keeps_original_joining() {
    assert_eq!(
        selected_source_shy_forms("-"),
        (
            vec![(Range::new(1, 2), 100, 20.0), (Range::new(2, 3), 277, 20.0)],
            vec![(Range::new(4, 5), 82, 20.0)]
        )
    );
}

#[test]
fn selected_source_shy_empty_replacement_keeps_original_joining() {
    assert_eq!(
        selected_source_shy_forms(""),
        (
            vec![(Range::new(1, 2), 100, 20.0), (Range::new(2, 3), 277, 20.0)],
            vec![(Range::new(4, 5), 82, 20.0)]
        )
    );
}

#[test]
fn selected_source_shy_on_continuation_keeps_both_original_context_sides() {
    let source = Source::new("\u{621}\u{628}\u{628}\u{ad}\u{628}\u{ad}\u{628}\u{621}");
    let mut input = discretionary_input(
        source,
        paragraph::ParagraphReplacementDefinition::Source {
            origin: 0,
            source: Range::new(3, 4),
        },
        "-",
    );
    input.replacements.push(
        paragraph::ParagraphReplacement::resolved_native_logical_before(
            paragraph::ParagraphReplacementDefinition::Source {
                origin: 0,
                source: Range::new(5, 6),
            },
            6,
            vec![crate::Text::from("-")],
        )
        .unwrap(),
    );
    let mut system = arabic_system();
    let prepared = native_prepare(&mut system, input);
    let start = prepared.boundary_views().next().unwrap().handle();
    let end = ending(&prepared, PreparedBreakKind::ParagraphEnd, 8);
    let whole = selected_line(&prepared, start.clone(), end.clone()).unwrap();
    assert_eq!(
        original_beh_forms(&whole),
        [
            (Range::new(1, 2), 100, 20.0),
            (Range::new(2, 3), 277, 20.0),
            (Range::new(4, 5), 277, 20.0),
            (Range::new(6, 7), 82, 20.0)
        ]
    );
    let first = discretionary_line(&prepared, start).unwrap();
    let middle =
        discretionary_line(&prepared, first.selected_line().request().end.clone()).unwrap();
    let last = selected_line(&prepared, middle.selected_line().request().end.clone(), end).unwrap();
    assert_eq!(
        (original_beh_forms(&middle), original_beh_forms(&last)),
        (
            vec![(Range::new(4, 5), 277, 20.0)],
            vec![(Range::new(6, 7), 82, 20.0)]
        )
    );
}

fn selected_control_forms(text: &str) -> Vec<(OriginalBehForms, OriginalBehForms)> {
    [None, Some(0)]
        .into_iter()
        .map(|origin| {
            let source = Source::new("\u{621}\u{628}\u{628}\u{628}\u{621}");
            let input = discretionary_input(
                source.clone(),
                paragraph::ParagraphReplacementDefinition::Control { control: 0, origin },
                text,
            );
            let mut system = arabic_system();
            let prepared = native_prepare(&mut system, input);
            let start = prepared.boundary_views().next().unwrap().handle();
            let whole = selected_line(
                &prepared,
                start.clone(),
                ending(&prepared, PreparedBreakKind::ParagraphEnd, 5),
            )
            .unwrap();
            assert_eq!(
                original_beh_forms(&whole),
                [
                    (Range::new(1, 2), 100, 20.0),
                    (Range::new(2, 3), 277, 20.0),
                    (Range::new(3, 4), 82, 20.0)
                ]
            );
            drop(system);
            let first = discretionary_line(&prepared, start).unwrap();
            let continuation = selected_line(
                &prepared,
                first.selected_line().request().end.clone(),
                ending(&prepared, PreparedBreakKind::ParagraphEnd, 5),
            )
            .unwrap();
            assert_eq!(prepared.analyzed_source().source(), &source);
            (
                original_beh_forms(&first),
                original_beh_forms(&continuation),
            )
        })
        .collect()
}

#[test]
fn selected_control_nonempty_replacement_keeps_original_joining_without_invented_origin() {
    let expected = (
        vec![(Range::new(1, 2), 100, 20.0), (Range::new(2, 3), 277, 20.0)],
        vec![(Range::new(3, 4), 82, 20.0)],
    );
    assert_eq!(selected_control_forms("-"), [expected.clone(), expected]);
}

#[test]
fn selected_control_empty_replacement_keeps_original_joining_without_invented_origin() {
    let expected = (
        vec![(Range::new(1, 2), 100, 20.0), (Range::new(2, 3), 277, 20.0)],
        vec![(Range::new(3, 4), 82, 20.0)],
    );
    assert_eq!(selected_control_forms(""), [expected.clone(), expected]);
}

#[test]
fn selected_source_shy_after_forced_boundary_keeps_only_same_domain_context() {
    let source = Source::new("\u{628}\u{621}\u{621}\u{628}\u{ad}\u{628}\u{621}");
    let mut input = discretionary_input(
        source,
        paragraph::ParagraphReplacementDefinition::Source {
            origin: 0,
            source: Range::new(4, 5),
        },
        "-",
    );
    input.controls.push(paragraph::ParagraphControl {
        at: 1,
        kind: paragraph::ParagraphControlKind::ForcedBreak,
    });
    input.anchor_order.push(paragraph::ParagraphAnchor {
        item: paragraph::ParagraphAnchorItem::Control(0),
        affinity: crate::Affinity::After,
    });
    let mut system = arabic_system();
    let prepared = native_prepare(&mut system, input);
    let forced = selected_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        ending(&prepared, PreparedBreakKind::Forced, 1),
    )
    .unwrap();
    let first =
        discretionary_line(&prepared, forced.selected_line().request().end.clone()).unwrap();
    let continuation = selected_line(
        &prepared,
        first.selected_line().request().end.clone(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 7),
    )
    .unwrap();
    assert_eq!(original_beh_forms(&forced), [(Range::new(0, 1), 35, 20.0)]);
    assert_eq!(
        (
            original_beh_forms(&first),
            original_beh_forms(&continuation)
        ),
        (
            vec![(Range::new(3, 4), 100, 20.0)],
            vec![(Range::new(5, 6), 82, 20.0)]
        )
    );
}

#[test]
fn selected_source_shy_context_stops_at_actual_disconnected_origin() {
    let source = Source::new("\u{621}\u{628}\u{628}\u{ad}\u{621}\u{621}\u{628}");
    let mut input = discretionary_input(
        source.clone(),
        paragraph::ParagraphReplacementDefinition::Source {
            origin: 0,
            source: Range::new(3, 4),
        },
        "-",
    );
    input.origins[0].local = Range::new(0, 6);
    input.origins[0].aggregate = Range::new(0, 6);
    input.origins.push(paragraph::ParagraphOrigin {
        text_node: None,
        source,
        local: Range::new(6, 7),
        aggregate: Range::new(6, 7),
        kind: paragraph::ParagraphOriginKind::Authored,
    });
    input.connections.push(paragraph::ParagraphConnection {
        at: 6,
        kind: paragraph::ParagraphConnectionKind::Disconnected,
    });
    let mut system = arabic_system();
    let prepared = native_prepare(&mut system, input);
    let first = discretionary_line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
    )
    .unwrap();
    let continuation = selected_line(
        &prepared,
        first.selected_line().request().end.clone(),
        ending(&prepared, PreparedBreakKind::ParagraphEnd, 7),
    )
    .unwrap();
    assert_eq!(
        original_beh_forms(&first),
        [(Range::new(1, 2), 100, 20.0), (Range::new(2, 3), 82, 20.0)]
    );
    assert_eq!(
        original_beh_forms(&continuation),
        [(Range::new(6, 7), 35, 20.0)]
    );
}

#[test]
fn selected_source_shy_complete_original_context_preserves_joining_and_prior_results() {
    let source = Source::new(format!(
        "\u{621}\u{628}\u{628}\u{ad}{}\u{628}\u{621}",
        "\u{61c}".repeat(6)
    ));
    let input = discretionary_input(
        source.clone(),
        paragraph::ParagraphReplacementDefinition::Source {
            origin: 0,
            source: Range::new(3, 4),
        },
        "",
    );
    let mut system = arabic_system();
    let prepared = native_prepare(&mut system, input);
    let stats = system.stats();
    let catalogue = system.font_catalogue_identity().clone();
    let start = prepared.boundary_views().next().unwrap().handle();
    let end = ending(&prepared, PreparedBreakKind::ParagraphEnd, 12);
    let whole = selected_line(&prepared, start.clone(), end.clone()).unwrap();
    let expected = vec![
        (Range::new(1, 2), 100, 20.0),
        (Range::new(2, 3), 277, 20.0),
        (Range::new(10, 11), 82, 20.0),
    ];
    assert_eq!(original_beh_forms(&whole), expected);
    let trial = discretionary_line(&prepared, start.clone());
    let retry = selected_line(&prepared, start, end).unwrap();
    assert_eq!(original_beh_forms(&whole), expected);
    assert_eq!(original_beh_forms(&retry), expected);
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(system.stats(), stats);
    assert_eq!(system.font_catalogue_identity(), &catalogue);
    // All original characters are available as the joining-analysis body.
    // The old five-cell external post-context limit does not bound that body.
    assert_eq!(
        original_beh_forms(&trial.unwrap()),
        [(Range::new(1, 2), 100, 20.0), (Range::new(2, 3), 277, 20.0)]
    );
}
