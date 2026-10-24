use super::*;
use crate::{OverflowWrap, Range, Wrap};

fn input(text: &str, overflow_wrap: OverflowWrap) -> ParagraphInput {
    let source = Source::new(text);
    let length = source.text().len();
    ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: Style {
            overflow_wrap,
            ..style()
        },
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
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

fn prepare(system: &mut System, input: ParagraphInput) -> PreparedText {
    let ancestor_wrap = input.base_style.wrap;
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    // These fixtures explicitly describe one flat containing box. The producer
    // binds that actual fact to Text's processed pairs, without deriving them.
    let policies = analyzed
        .wrapping_relations()
        .map(|relation| crate::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}

fn first_after(prepared: &PreparedText, start: &PreparedBoundary) -> PreparedOpportunity {
    prepared
        .opportunity_views()
        .find(|offer| {
            offer.boundary().view().participant_offset() > start.view().participant_offset()
        })
        .unwrap()
        .handle()
}

fn offers(
    prepared: &PreparedText,
    purpose: ConditionalPurpose,
) -> Vec<PreparedConditionalOpportunity> {
    let start = prepared.boundary_views().next().unwrap().handle();
    prepared
        .conditional_opportunities(&start, None, &first_after(prepared, &start), purpose, &[])
        .unwrap()
}

fn sources(offers: &[PreparedConditionalOpportunity]) -> Vec<usize> {
    offers
        .iter()
        .map(|offer| offer.boundary().view().source_before().get())
        .collect()
}

fn selection(
    prepared: &PreparedText,
    offer: &PreparedConditionalOpportunity,
) -> LineSelectionRequest {
    let mut value = request(prepared);
    value.start = offer.start();
    value.end = offer.boundary();
    let from = value.start.view().participant_offset();
    let to = value.end.view().participant_offset();
    value.participants = prepared.participants().skip(from).take(to - from).collect();
    value.controls = value
        .participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Control(occurrence) => Some(ControlAllocation {
                occurrence: occurrence.clone(),
                strut: None,
            }),
            _ => None,
        })
        .collect();
    value.atomics = value
        .participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Atomic(occurrence) => Some(AtomicAllocation {
                occurrence: occurrence.clone(),
                allocation: AtomicLineAllocation::OutOfFlow,
            }),
            _ => None,
        })
        .collect();
    value.termination = PreparedTermination::Conditional(offer.clone());
    value.context.start = if from == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    value.context.end = LineEnd::Soft;
    value.context.available_inline = match offer.purpose() {
        ConditionalPurpose::Overflow => Some(1.0),
        ConditionalPurpose::MinContent => None,
    };
    value
}

#[test]
fn native_conditional_permission_and_intrinsic_purpose_are_distinct() {
    let mut system = system();
    for (mode, finite, minimum) in [
        (OverflowWrap::Normal, vec![], vec![]),
        (OverflowWrap::BreakWord, vec![1, 2], vec![]),
        (OverflowWrap::Anywhere, vec![1, 2], vec![1, 2]),
    ] {
        let prepared = prepare(&mut system, input("AAA", mode));
        assert_eq!(
            prepared
                .opportunity_views()
                .map(|offer| offer.kind())
                .collect::<Vec<_>>(),
            [PreparedBreakKind::ParagraphEnd]
        );
        assert_eq!(
            sources(&offers(&prepared, ConditionalPurpose::Overflow)),
            finite
        );
        assert_eq!(
            sources(&offers(&prepared, ConditionalPurpose::MinContent)),
            minimum
        );
    }
    let mut nowrap = input("AAA", OverflowWrap::Anywhere);
    nowrap.base_style.wrap = Wrap::None;
    let prepared = prepare(&mut system, nowrap);
    assert!(offers(&prepared, ConditionalPurpose::Overflow).is_empty());
    assert!(offers(&prepared, ConditionalPurpose::MinContent).is_empty());
}

#[test]
fn conditional_cuts_keep_graphemes_and_original_surrogate_pairs_whole() {
    let mut system = system();
    for (text, expected) in [
        ("A\u{301}B", vec![2]),
        ("\u{10400}B", vec![2]),
        ("\u{10400}", vec![]),
    ] {
        let prepared = prepare(&mut system, input(text, OverflowWrap::Anywhere));
        assert_eq!(
            sources(&offers(&prepared, ConditionalPurpose::Overflow)),
            expected,
            "{text:?}"
        );
    }
}

#[test]
fn conditional_capabilities_preserve_query_identity_and_do_not_claim_fit() {
    let mut system = system();
    let prepared = prepare(&mut system, input("AAA", OverflowWrap::Anywhere));
    let first = offers(&prepared, ConditionalPurpose::Overflow);
    let again = offers(&prepared, ConditionalPurpose::Overflow);
    assert_eq!(first, again);
    let minimum = offers(&prepared, ConditionalPurpose::MinContent);
    assert_ne!(first[0], minimum[0]);
    let line = prepared
        .select(selection(&prepared, &first[0]))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    // Eligibility is not fit: this actual20px glyph was selected for a1px span.
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        line.run_views()
            .next()
            .unwrap()
            .glyphs()
            .next()
            .unwrap()
            .range(),
        Range::new(0, 1)
    );
    let mut replay = selection(&prepared, &first[1]);
    replay.start = first[0].boundary();
    replay.participants.remove(0);
    replay.context.start = LineStart::Continuation;
    assert!(matches!(
        reason(&prepared.select(replay).unwrap_err()),
        ProtocolError::InvalidConditionalSelection { .. }
    ));
    let mut repurpose = selection(&prepared, &first[0]);
    repurpose.context.available_inline = None;
    assert_eq!(
        reason(&prepared.select(repurpose).unwrap_err()),
        &ProtocolError::InvalidConditionalContext {
            purpose: ConditionalPurpose::Overflow
        }
    );
    let mut repurpose = selection(&prepared, &minimum[0]);
    repurpose.context.available_inline = Some(20.0);
    assert_eq!(
        reason(&prepared.select(repurpose).unwrap_err()),
        &ProtocolError::InvalidConditionalContext {
            purpose: ConditionalPurpose::MinContent
        }
    );
    let other = prepare(&mut system, input("AAA", OverflowWrap::Anywhere));
    let mut foreign = selection(&other, &offers(&other, ConditionalPurpose::Overflow)[0]);
    foreign.termination = PreparedTermination::Conditional(first[0].clone());
    assert!(matches!(
        reason(&other.select(foreign).unwrap_err()),
        ProtocolError::ForeignPreparation {
            kind: ProtocolHandleKind::ConditionalOpportunity,
            ..
        }
    ));
}

#[test]
fn conditional_interval_stops_at_first_ordinary_or_forced_endpoint() {
    let mut system = system();
    let prepared = prepare(&mut system, input("AA AA", OverflowWrap::Anywhere));
    let start = prepared.boundary_views().next().unwrap().handle();
    let first = first_after(&prepared, &start);
    let end = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    assert!(matches!(
        reason(
            &prepared
                .conditional_opportunities(&start, None, &end, ConditionalPurpose::Overflow, &[])
                .unwrap_err()
        ),
        ProtocolError::InvalidConditionalReceipt { .. }
    ));
    assert_eq!(
        sources(
            &prepared
                .conditional_opportunities(&start, None, &first, ConditionalPurpose::Overflow, &[])
                .unwrap()
        ),
        [1, 2]
    );
    let prepared = prepare(&mut system, input("AA\nAA", OverflowWrap::Anywhere));
    assert_eq!(
        sources(&offers(&prepared, ConditionalPurpose::Overflow)),
        [1]
    );
}

fn glyphs(line: &FinalizedLine) -> Vec<(Range, u32)> {
    let mut values = Vec::new();
    for run in line.run_views() {
        values.extend(run.glyphs().map(|glyph| (glyph.range(), glyph.id())));
    }
    values.sort_by_key(|(range, _)| (range.start, range.end));
    values
}

#[test]
fn conditional_cuts_reshape_optional_ligatures_with_the_retained_instance() {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    let fixture = crate::shaping_test_fonts::ligature_ahem();
    system.register_fonts(fixture.clone()).unwrap();
    let prepared = prepare(&mut system, input("AA", OverflowWrap::Anywhere));
    let whole = prepared
        .select(request(&prepared))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(glyphs(&whole), [(Range::new(0, 2), 58)]);
    assert_eq!(whole.feedback().advance, 20.0);
    let cuts = offers(&prepared, ConditionalPurpose::Overflow);
    assert_eq!(sources(&cuts), [1]);
    drop(system);
    let line = prepared
        .select(selection(&prepared, &cuts[0]))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(glyphs(&line), [(Range::new(0, 1), 35)]);
    assert_eq!(line.feedback().advance, 20.0);
    let run = line.run_views().next().unwrap();
    assert_eq!(run.face().bytes(), fixture);
    let instance = run.effective_instance().unwrap();
    assert!(instance.face().is_same_face(&run.face()));
    assert_eq!(instance.font_size(), 20.0);
}

#[test]
fn conditional_cut_and_continuation_keep_actual_arabic_base_context() {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    let fixture = crate::shaping_test_fonts::arabic_ahem();
    system.register_fonts(fixture.clone()).unwrap();
    let mut value = input("\u{628}\u{628}\u{628}", OverflowWrap::Anywhere);
    value.base_direction = ParagraphBaseDirection::RightToLeft;
    let prepared = prepare(&mut system, value);
    let mut control = request(&prepared);
    control.context.base_direction = ParagraphBaseDirection::RightToLeft;
    let whole = prepared
        .select(control)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(
        glyphs(&whole),
        [
            (Range::new(0, 1), 100),
            (Range::new(1, 2), 277),
            (Range::new(2, 3), 82)
        ]
    );
    let cuts = offers(&prepared, ConditionalPurpose::Overflow);
    assert_eq!(sources(&cuts), [1, 2]);
    let mut first_request = selection(&prepared, &cuts[0]);
    first_request.context.base_direction = ParagraphBaseDirection::RightToLeft;
    let first = prepared
        .select(first_request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(glyphs(&first), [(Range::new(0, 1), 100)]);
    let second_start = cuts[0].boundary();
    let second_cuts = prepared
        .conditional_opportunities(
            &second_start,
            None,
            &first_after(&prepared, &second_start),
            ConditionalPurpose::Overflow,
            &[],
        )
        .unwrap();
    assert_eq!(sources(&second_cuts), [2]);
    let mut second_request = selection(&prepared, &second_cuts[0]);
    second_request.context.base_direction = ParagraphBaseDirection::RightToLeft;
    let second = prepared
        .select(second_request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(glyphs(&second), [(Range::new(1, 2), 277)]);
    let mut last_request = request(&prepared);
    last_request.start = second_cuts[0].boundary();
    last_request.participants = prepared.participants().skip(2).collect();
    last_request.context.start = LineStart::Continuation;
    last_request.context.base_direction = ParagraphBaseDirection::RightToLeft;
    let last = prepared
        .select(last_request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(glyphs(&last), [(Range::new(2, 3), 82)]);
    for line in [first, second, last] {
        assert_eq!(line.prepared_text(), prepared);
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), fixture);
            assert_eq!(run.font_size(), 20.0);
            assert!(
                run.effective_instance()
                    .unwrap()
                    .face()
                    .is_same_face(&run.face())
            );
        }
    }
}

#[test]
fn conditional_termination_cannot_select_failed_ordinary_replacement() {
    use paragraph::{ParagraphReplacement, ParagraphReplacementDefinition};
    let mut system = system();
    let mut value = input("AX\u{ad}XXX", OverflowWrap::Anywhere);
    value.replacements.push(
        ParagraphReplacement::resolved_native_logical_before(
            ParagraphReplacementDefinition::Source {
                origin: 0,
                source: Range::new(2, 3),
            },
            3,
            vec![crate::Text::from("-")],
        )
        .unwrap(),
    );
    let prepared = prepare(&mut system, value);
    let first = prepared.opportunity_views().next().unwrap();
    assert_eq!(first.kind(), PreparedBreakKind::Discretionary);
    let alternative = first.alternatives().next().unwrap().handle();
    let cuts = offers(&prepared, ConditionalPurpose::Overflow);
    assert_eq!(sources(&cuts), [1, 2]);
    let original = prepared.analyzed_source().source().text().clone();
    let mut invalid = selection(&prepared, &cuts[0]);
    invalid.alternative = Some(alternative);
    assert!(matches!(
        reason(&prepared.select(invalid).unwrap_err()),
        ProtocolError::InvalidConditionalSelection { .. }
    ));
    let selected = prepared.select(selection(&prepared, &cuts[0])).unwrap();
    assert!(selected.request().alternative.is_none());
    let line = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(glyphs(&line), [(Range::new(0, 1), 35)]);
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(prepared.analyzed_source().source().text(), &original);
}

#[test]
fn wrapping_binding_has_new_context_identity_and_preserves_the_executed_core() {
    let mut system = system();
    let raw = AnalyzedSource::from_paragraph(input("AAA", OverflowWrap::Anywhere)).unwrap();
    let old_identity = raw.identity();
    let old_graphemes: Vec<_> = raw.graphemes().collect();
    let policies: Vec<_> = raw
        .wrapping_relations()
        .map(|view| crate::WrappingPolicy {
            relation: view.handle(),
            ancestor_wrap: Wrap::Word,
        })
        .collect();
    let bound = raw.with_wrapping_policies(policies.clone()).unwrap();
    assert_ne!(bound.identity(), old_identity);
    assert!(std::sync::Arc::ptr_eq(&bound.inner.core, &raw.inner.core));
    assert_eq!(bound.source(), raw.source());
    assert_eq!(bound.graphemes().collect::<Vec<_>>(), old_graphemes);
    assert!(std::ptr::eq(
        bound.processed_source(),
        raw.processed_source()
    ));
    assert!(matches!(
        reason(&bound.with_wrapping_policies(policies.clone()).unwrap_err()),
        ProtocolError::ForeignWrappingRelation { .. }
    ));
    let prepared = system
        .prepare(&bound, PreparationOptions::default())
        .unwrap();
    let prior = offers(&prepared, ConditionalPurpose::Overflow);
    assert_eq!(sources(&prior), [1, 2]);
    let changed = bound
        .with_wrapping_policies(
            bound
                .wrapping_relations()
                .map(|view| crate::WrappingPolicy {
                    relation: view.handle(),
                    ancestor_wrap: Wrap::None,
                })
                .collect(),
        )
        .unwrap();
    assert_ne!(changed.identity(), bound.identity());
    let other = system
        .prepare(&changed, PreparationOptions::default())
        .unwrap();
    assert_ne!(other, prepared);
    assert!(offers(&other, ConditionalPurpose::Overflow).is_empty());
    assert_eq!(
        sources(&offers(&prepared, ConditionalPurpose::Overflow)),
        [1, 2]
    );
    assert_eq!(
        prepared
            .select(selection(&prepared, &prior[0]))
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        20.0
    );
    assert!(
        raw.wrapping_relations()
            .all(|view| view.ancestor_wrap().is_none())
    );
    assert!(
        bound
            .wrapping_relations()
            .all(|view| view.ancestor_wrap() == Some(Wrap::Word))
    );
    let old_line = prepared
        .select(request(&prepared))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let new_line = other
        .select(request(&other))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let old_layout = place(prepared.clone(), old_line, 0.0, 16.0).unwrap();
    let new_layout = place(other.clone(), new_line, 0.0, 16.0).unwrap();
    assert_ne!(old_layout.key(), new_layout.key());
    assert_eq!(old_layout.lines()[0].range(), new_layout.lines()[0].range());
    let duplicate = vec![policies[0].clone(), policies[0].clone()];
    assert!(matches!(
        reason(&raw.with_wrapping_policies(duplicate).unwrap_err()),
        ProtocolError::DuplicateWrappingPolicy { .. }
    ));
    let mut conflict = policies[0].clone();
    conflict.ancestor_wrap = Wrap::None;
    assert!(matches!(
        reason(
            &raw.with_wrapping_policies(vec![policies[0].clone(), conflict])
                .unwrap_err()
        ),
        ProtocolError::ConflictingWrappingPolicy { .. }
    ));
}

#[cfg(feature = "text-accessibility")]
#[test]
fn bound_wrapping_keeps_source_only_ax_content_and_separate_context_custody() {
    let raw = AnalyzedSource::from_paragraph(input("AAA", OverflowWrap::Anywhere)).unwrap();
    let policies = raw
        .wrapping_relations()
        .map(|view| crate::WrappingPolicy {
            relation: view.handle(),
            ancestor_wrap: Wrap::Word,
        })
        .collect();
    let bound = raw.with_wrapping_policies(policies).unwrap();
    let parent = accesskit::NodeId(100);
    let mut old_state = crate::Accessibility::new(200);
    let mut new_state = crate::Accessibility::new(200);
    let old_update = raw
        .accessibility_update(&mut old_state, parent, parent)
        .unwrap();
    let new_update = bound
        .accessibility_update(&mut new_state, parent, parent)
        .unwrap();
    assert_eq!(
        old_update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect::<String>(),
        "AAA"
    );
    assert_eq!(
        new_update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect::<String>(),
        "AAA"
    );
    let cursor = crate::Cursor::new(1, crate::Affinity::After);
    assert!(raw.access_position(&old_state, cursor).unwrap().is_some());
    assert_eq!(
        bound
            .access_position(&old_state, cursor)
            .unwrap_err()
            .detail(),
        Some(&crate::ErrorDetail::AccessibilityBindingMismatch)
    );
    assert!(bound.access_position(&new_state, cursor).unwrap().is_some());
    assert!(raw.access_position(&old_state, cursor).unwrap().is_some());
}

#[test]
fn missing_wrapping_fact_is_refused_only_on_the_path_that_consumes_it() {
    let mut system = system();
    let raw = AnalyzedSource::from_paragraph(input("AAA", OverflowWrap::Anywhere)).unwrap();
    let prepared = system.prepare(&raw, PreparationOptions::default()).unwrap();
    let start = prepared.boundary_views().next().unwrap().handle();
    assert!(matches!(
        reason(
            &prepared
                .conditional_opportunities(
                    &start,
                    None,
                    &first_after(&prepared, &start),
                    ConditionalPurpose::Overflow,
                    &[]
                )
                .unwrap_err()
        ),
        ProtocolError::MissingWrappingPolicy { .. }
    ));
    // Unrelated paragraph termination remains usable without conditional facts.
    assert_eq!(
        prepared
            .select(request(&prepared))
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        60.0
    );
    let raw = AnalyzedSource::from_paragraph(input("A\nA", OverflowWrap::Normal)).unwrap();
    let prepared = system.prepare(&raw, PreparationOptions::default()).unwrap();
    assert_eq!(
        prepared
            .opportunity_views()
            .map(|view| view.kind())
            .collect::<Vec<_>>(),
        [PreparedBreakKind::Forced, PreparedBreakKind::ParagraphEnd]
    );
    assert!(offers(&prepared, ConditionalPurpose::Overflow).is_empty());
    let raw = AnalyzedSource::from_paragraph(input("A A", OverflowWrap::Normal)).unwrap();
    assert!(matches!(
        reason(
            &system
                .prepare(&raw, PreparationOptions::default())
                .unwrap_err()
        ),
        ProtocolError::MissingWrappingPolicy { .. }
    ));
    let mut collapsed = input("A A", OverflowWrap::Normal);
    collapsed.base_style.white_space = WhiteSpace::Collapse;
    let raw = AnalyzedSource::from_paragraph(collapsed).unwrap();
    assert!(system.prepare(&raw, PreparationOptions::default()).is_ok());
}

fn ordinary_receipt(
    prepared: &PreparedText,
    start: PreparedBoundary,
    offer: PreparedOpportunity,
    alternative: Option<PreparedAlternative>,
) -> FinalizedLine {
    let mut value = request(prepared);
    value.start = start;
    value.end = offer.view().boundary();
    let from = value.start.view().participant_offset();
    let to = value.end.view().participant_offset();
    value.participants = prepared.participants().skip(from).take(to - from).collect();
    value.controls = value
        .participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Control(occurrence) => Some(ControlAllocation {
                occurrence: occurrence.clone(),
                strut: None,
            }),
            _ => None,
        })
        .collect();
    value.termination = PreparedTermination::Ordinary(offer.clone());
    value.alternative = alternative;
    value.context.start = if from == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    value.context.end = match offer.view().kind() {
        PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary => LineEnd::Soft,
        PreparedBreakKind::Forced => LineEnd::Forced,
        PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
    };
    value.context.available_inline = Some(20.0);
    prepared
        .select(value)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}

#[test]
fn phantom_receipt_preserves_leading_original_custody_and_continuation() {
    let mut system = system();
    let mut value = input(" XXXX", OverflowWrap::Anywhere);
    value.base_style.white_space = WhiteSpace::Collapse;
    let prepared = prepare(&mut system, value);
    let start = prepared.boundary_views().next().unwrap().handle();
    let first = first_after(&prepared, &start);
    let ceiling = prepared.opportunity_views().last().unwrap().handle();
    let phantom = ordinary_receipt(&prepared, start.clone(), first, None);
    assert!(
        !phantom
            .prefix_at(&crate::TextSeam::AfterParticipant(
                phantom
                    .selected_line()
                    .request()
                    .participants
                    .last()
                    .unwrap()
                    .clone()
            ))
            .unwrap()
            .has_retained_flow_participation()
    );
    let cuts = prepared
        .conditional_opportunities(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[crate::ConditionalDisposition::Phantom(phantom.clone())],
        )
        .unwrap();
    assert_eq!(sources(&cuts), [2, 3, 4]);
    let first = prepared
        .select(selection(&prepared, &cuts[0]))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(first.selected_line().request().start, start);
    assert_eq!(first.selected_line().request().participants.len(), 2);
    assert_eq!(first.feedback().advance, 20.0);
    assert!(
        glyphs(&first)
            .iter()
            .any(|(range, _)| *range == Range::new(1, 2))
    );
    let continuation = ordinary_receipt(&prepared, cuts[0].boundary(), ceiling, None);
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        lines: vec![
            LinePlacement {
                line: first,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 16.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            },
            LinePlacement {
                line: continuation,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 36.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            },
        ],
    })
    .unwrap();
    assert_eq!(
        layout.source().text().code_units(),
        " XXXX".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 5)]
    );
}

#[test]
fn phantom_receipts_cover_exact_same_offset_ordinals_and_recheck_strut_presence() {
    use paragraph::{ParagraphAnchor, ParagraphAnchorItem, ParagraphControl, ParagraphControlKind};
    let mut system = system();
    let mut value = input(" XXXX", OverflowWrap::Anywhere);
    value.base_style.white_space = WhiteSpace::Collapse;
    value.controls = vec![
        ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word }
        };
        2
    ];
    value.anchor_order = (0..2)
        .map(|index| ParagraphAnchor {
            item: ParagraphAnchorItem::Control(index),
            affinity: crate::Affinity::Before,
        })
        .collect();
    let prepared = prepare(&mut system, value);
    let start = prepared.boundary_views().next().unwrap().handle();
    let inventory: Vec<_> = prepared
        .opportunity_views()
        .map(|view| view.handle())
        .collect();
    assert_eq!(inventory.len(), 4);
    let receipts: Vec<_> = inventory[..3]
        .iter()
        .map(|offer| ordinary_receipt(&prepared, start.clone(), offer.clone(), None))
        .collect();
    let ceiling = &inventory[3];
    let cuts = prepared
        .conditional_opportunities(
            &start,
            None,
            ceiling,
            ConditionalPurpose::Overflow,
            &phantom_dispositions(&receipts),
        )
        .unwrap();
    assert_eq!(sources(&cuts), [2, 3, 4]);
    assert_eq!(
        cuts,
        prepared
            .conditional_opportunities(
                &start,
                None,
                ceiling,
                ConditionalPurpose::Overflow,
                &phantom_dispositions(&receipts.clone())
            )
            .unwrap()
    );
    let other_trials: Vec<_> = receipts
        .iter()
        .map(|line| {
            let mut value = line.selected_line().request().clone();
            value.context.available_inline = Some(500.0);
            prepared
                .select(value)
                .unwrap()
                .finalize(SpacingAllocations::None)
                .unwrap()
        })
        .collect();
    assert_eq!(
        cuts,
        prepared
            .conditional_opportunities(
                &start,
                None,
                ceiling,
                ConditionalPurpose::Overflow,
                &phantom_dispositions(&other_trials)
            )
            .unwrap()
    );
    for bad in [
        vec![receipts[0].clone()],
        vec![
            receipts[1].clone(),
            receipts[0].clone(),
            receipts[2].clone(),
        ],
        vec![
            receipts[0].clone(),
            receipts[0].clone(),
            receipts[2].clone(),
        ],
    ] {
        assert!(matches!(
            reason(
                &prepared
                    .conditional_opportunities(
                        &start,
                        None,
                        ceiling,
                        ConditionalPurpose::Overflow,
                        &phantom_dispositions(&bad)
                    )
                    .unwrap_err()
            ),
            ProtocolError::InvalidConditionalReceipt { .. }
        ));
    }
    let mut selected = selection(&prepared, &cuts[0]);
    selected.controls[0].strut = Some(ControlStrut {
        baseline: 0.0,
        line_extent: 0.0,
        alignment: LineBoxAlignment::Baseline,
    });
    assert!(matches!(
        reason(&prepared.select(selected).unwrap_err()),
        ProtocolError::InvalidConditionalSelection { .. }
    ));
    assert!(matches!(
        reason(
            &prepared
                .conditional_opportunities(
                    &start,
                    None,
                    ceiling,
                    ConditionalPurpose::MinContent,
                    &phantom_dispositions(&receipts)
                )
                .unwrap_err()
        ),
        ProtocolError::InvalidConditionalContext {
            purpose: ConditionalPurpose::MinContent
        }
    ));
    let minimum_receipts: Vec<_> = receipts
        .iter()
        .map(|line| {
            let mut value = line.selected_line().request().clone();
            value.context.available_inline = None;
            prepared
                .select(value)
                .unwrap()
                .finalize(SpacingAllocations::None)
                .unwrap()
        })
        .collect();
    let minimum = prepared
        .conditional_opportunities(
            &start,
            None,
            ceiling,
            ConditionalPurpose::MinContent,
            &phantom_dispositions(&minimum_receipts),
        )
        .unwrap();
    assert_eq!(sources(&minimum), [2, 3, 4]);
    let minimum_line = prepared
        .select(selection(&prepared, &minimum[0]))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(minimum_line.feedback().advance, 20.0);
}

#[test]
fn retained_zero_or_signed_text_cannot_be_a_phantom_receipt() {
    let mut system = system();
    for (spacing, advance) in [(-20.0, 0.0), (-40.0, -20.0)] {
        let mut value = input("A AAA", OverflowWrap::Anywhere);
        value.base_style.white_space = WhiteSpace::Collapse;
        value.base_style.letter_spacing = spacing;
        let prepared = prepare(&mut system, value);
        let start = prepared.boundary_views().next().unwrap().handle();
        let first = first_after(&prepared, &start);
        let ceiling = prepared.opportunity_views().last().unwrap().handle();
        let receipt = ordinary_receipt(&prepared, start.clone(), first, None);
        assert_eq!(receipt.feedback().advance, advance);
        assert!(matches!(
            reason(
                &prepared
                    .conditional_opportunities(
                        &start,
                        None,
                        &ceiling,
                        ConditionalPurpose::Overflow,
                        &[crate::ConditionalDisposition::Phantom(receipt)]
                    )
                    .unwrap_err()
            ),
            ProtocolError::RetainedConditionalPrefix { .. }
        ));
    }
}

#[test]
fn phantom_frontier_requires_every_real_discretionary_variant() {
    use paragraph::{
        ParagraphAnchor, ParagraphAnchorItem, ParagraphControl, ParagraphControlKind,
        ParagraphReplacement, ParagraphReplacementDefinition,
    };
    let mut system = system();
    let mut value = input(" XXXXX", OverflowWrap::Anywhere);
    value.base_style.white_space = WhiteSpace::Collapse;
    value.controls.push(ParagraphControl {
        at: 3,
        kind: ParagraphControlKind::DiscretionaryBreak {
            style: Box::new(value.base_style.clone()),
        },
    });
    value.anchor_order.push(ParagraphAnchor {
        item: ParagraphAnchorItem::Control(0),
        affinity: crate::Affinity::Before,
    });
    value.replacements = vec![
        ParagraphReplacement::resolved_native_logical_before(
            ParagraphReplacementDefinition::Control {
                control: 0,
                origin: None,
            },
            3,
            vec![crate::Text::from(""), crate::Text::from("A")],
        )
        .unwrap(),
    ];
    let mut invalid = value.clone();
    invalid.controls[0].kind = ParagraphControlKind::OptionalBreak { wrap: Wrap::Word };
    assert!(matches!(
        reason(&AnalyzedSource::from_paragraph(invalid).unwrap_err()),
        ProtocolError::InvalidReplacement { .. }
    ));
    let prepared = prepare(&mut system, value);
    let control_position = prepared
        .participants()
        .position(|p| matches!(p, PreparedParticipant::Control(_)))
        .unwrap();
    let start = prepared
        .boundary_views()
        .find(|b| b.participant_offset() == control_position)
        .unwrap()
        .handle();
    let inventory: Vec<_> = prepared
        .opportunity_views()
        .map(|view| view.handle())
        .collect();
    assert_eq!(
        inventory
            .iter()
            .map(|offer| offer.view().kind())
            .collect::<Vec<_>>(),
        [
            PreparedBreakKind::Allowed,
            PreparedBreakKind::Discretionary,
            PreparedBreakKind::ParagraphEnd
        ]
    );
    // The admitted word's prefix is already consumed. The control-only
    // continuation can be phantom, independently of that real prefix.
    let mut receipts = vec![];
    let alternatives: Vec<_> = inventory[1]
        .view()
        .alternatives()
        .map(|view| view.handle())
        .collect();
    receipts.push(ordinary_receipt(
        &prepared,
        start.clone(),
        inventory[1].clone(),
        Some(alternatives[0].clone()),
    ));
    assert!(matches!(
        reason(
            &prepared
                .conditional_opportunities(
                    &start,
                    None,
                    &inventory[2],
                    ConditionalPurpose::Overflow,
                    &phantom_dispositions(&receipts)
                )
                .unwrap_err()
        ),
        ProtocolError::InvalidConditionalReceipt { .. }
    ));
    receipts.push(ordinary_receipt(
        &prepared,
        start.clone(),
        inventory[1].clone(),
        Some(alternatives[1].clone()),
    ));
    assert!(matches!(
        reason(
            &prepared
                .conditional_opportunities(
                    &start,
                    None,
                    &inventory[2],
                    ConditionalPurpose::Overflow,
                    &phantom_dispositions(&receipts)
                )
                .unwrap_err()
        ),
        ProtocolError::RetainedConditionalPrefix { .. }
    ));
}

#[test]
fn true_outer_permission_does_not_vote_immediate_empty_chain_facts_or_collapse_custody() {
    use paragraph::{
        ParagraphAnchor, ParagraphAnchorItem, ParagraphBoundaryDefinition, ParagraphBoundaryPolicy,
        ParagraphConnection, ParagraphConnectionKind, ParagraphControl, ParagraphControlKind,
        ParagraphOrigin, ParagraphOriginKind,
    };
    let mut system = system();
    let mut value = input("AB", OverflowWrap::Anywhere);
    for occurrence in 0..2 {
        value.source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(100 + occurrence),
            crate::InlineBoxKind::OutOfFlow,
            1,
            crate::Size::new(10.0, 10.0),
        ));
    }
    value.origins = [("A", 0, 1), ("", 1, 1), ("", 1, 1), ("B", 1, 2)]
        .map(|(text, start, end)| ParagraphOrigin {
            text_node: None,
            source: Source::new(text),
            local: Range::new(0, end - start),
            aggregate: Range::new(start, end),
            kind: ParagraphOriginKind::Authored,
        })
        .to_vec();
    value.connections = vec![
        ParagraphConnection {
            at: 1,
            kind: ParagraphConnectionKind::Continuous
        };
        3
    ];
    value.boundary_policies = [Wrap::Word, Wrap::None, Wrap::Word]
        .into_iter()
        .enumerate()
        .map(|(index, ancestor_wrap)| ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(index),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap,
        })
        .collect();
    value.controls = [
        ParagraphControlKind::InlineBoundaryStart,
        ParagraphControlKind::PopDirectionalIsolate,
    ]
    .map(|kind| ParagraphControl { at: 1, kind })
    .to_vec();
    value.anchor_order = [
        ParagraphAnchorItem::EmptyOrigin(1),
        ParagraphAnchorItem::Atomic(0),
        ParagraphAnchorItem::Control(0),
        ParagraphAnchorItem::EmptyOrigin(2),
        ParagraphAnchorItem::Atomic(1),
        ParagraphAnchorItem::Control(1),
    ]
    .map(|item| ParagraphAnchor {
        item,
        affinity: crate::Affinity::Before,
    })
    .to_vec();
    let raw = AnalyzedSource::from_paragraph(value).unwrap();
    let relation = raw.wrapping_relations().next().unwrap();
    assert_eq!(raw.wrapping_relations().len(), 1);
    assert_eq!(
        relation.before(),
        crate::WrappingNeighbor::Text {
            origin: 0,
            local: Range::new(0, 1)
        }
    );
    assert_eq!(
        relation.after(),
        crate::WrappingNeighbor::Text {
            origin: 3,
            local: Range::new(0, 1)
        }
    );
    assert_eq!(relation.ancestor_wrap(), None);
    for (ancestor_wrap, count) in [(Wrap::Word, 7), (Wrap::None, 0)] {
        let bound = raw
            .with_wrapping_policies(vec![crate::WrappingPolicy {
                relation: relation.handle(),
                ancestor_wrap,
            }])
            .unwrap();
        let prepared = system
            .prepare(&bound, PreparationOptions::default())
            .unwrap();
        let cuts = offers(&prepared, ConditionalPurpose::Overflow);
        assert_eq!(cuts.len(), count);
        for pair in cuts.windows(2) {
            assert_ne!(pair[0], pair[1]);
        }
        for (index, cut) in cuts.iter().enumerate() {
            assert_eq!(cut.boundary().view().source_before().get(), 1);
            let selected = prepared.select(selection(&prepared, cut)).unwrap();
            assert_eq!(selected.request().participants.len(), index + 1);
            assert_eq!(
                selected
                    .finalize(SpacingAllocations::None)
                    .unwrap()
                    .feedback()
                    .advance,
                20.0
            );
        }
    }
}

#[test]
fn deleted_bridge_relations_keep_real_original_endpoints_and_logical_before_style() {
    use paragraph::{
        ParagraphBoundaryDefinition, ParagraphBoundaryPolicy, ParagraphConnection,
        ParagraphConnectionKind, ParagraphOrigin, ParagraphOriginKind,
    };
    let mut system = system();
    let mut value = input("A B", OverflowWrap::Anywhere);
    value.base_style.white_space = WhiteSpace::Discard;
    value.origins = [("A", 0), (" ", 1), ("B", 2)]
        .map(|(text, start)| ParagraphOrigin {
            text_node: None,
            source: Source::new(text),
            local: Range::new(0, 1),
            aggregate: Range::new(start, start + 1),
            kind: ParagraphOriginKind::Authored,
        })
        .to_vec();
    value.connections = [1, 2]
        .map(|at| ParagraphConnection {
            at,
            kind: ParagraphConnectionKind::Continuous,
        })
        .to_vec();
    value.boundary_policies = [Wrap::None, Wrap::Word]
        .into_iter()
        .enumerate()
        .map(|(index, ancestor_wrap)| ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(index),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap,
        })
        .collect();
    let raw = AnalyzedSource::from_paragraph(value).unwrap();
    let view = raw.wrapping_relations().next().unwrap();
    assert_eq!(
        view.before(),
        crate::WrappingNeighbor::Text {
            origin: 0,
            local: Range::new(0, 1)
        }
    );
    assert_eq!(
        view.after(),
        crate::WrappingNeighbor::Text {
            origin: 2,
            local: Range::new(0, 1)
        }
    );
    assert!(view.ancestor_wrap().is_none());
    let bound = raw
        .with_wrapping_policies(vec![crate::WrappingPolicy {
            relation: view.handle(),
            ancestor_wrap: Wrap::Word,
        }])
        .unwrap();
    let prepared = system
        .prepare(&bound, PreparationOptions::default())
        .unwrap();
    assert_eq!(
        sources(&offers(&prepared, ConditionalPurpose::Overflow)),
        [1, 2]
    );
    let mut value = input("AAA", OverflowWrap::Anywhere);
    value.source.span(
        Range::new(1, 3),
        Style {
            overflow_wrap: OverflowWrap::Normal,
            ..style()
        },
    );
    let prepared = prepare(&mut system, value);
    assert_eq!(
        sources(&offers(&prepared, ConditionalPurpose::Overflow)),
        [1]
    );
}

#[test]
fn conditional_context_uses_base_capture_after_actual_hyphen_variant_trial() {
    use paragraph::{ParagraphReplacement, ParagraphReplacementDefinition};
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::arabic_nonjoining_ahem())
        .unwrap();
    let mut value = input(
        "\u{621}\u{628}\u{628}\u{ad}\u{628}\u{621}",
        OverflowWrap::Anywhere,
    );
    value.base_direction = ParagraphBaseDirection::RightToLeft;
    value.replacements.push(
        ParagraphReplacement::resolved_native_logical_before(
            ParagraphReplacementDefinition::Source {
                origin: 0,
                source: Range::new(3, 4),
            },
            4,
            vec![crate::Text::from("-")],
        )
        .unwrap(),
    );
    let prepared = prepare(&mut system, value);
    let mut whole_request = request(&prepared);
    whole_request.context.base_direction = ParagraphBaseDirection::RightToLeft;
    let whole = prepared
        .select(whole_request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert!(glyphs(&whole).contains(&(Range::new(1, 2), 100)));
    let first = prepared.opportunity_views().next().unwrap();
    let alternative = first.alternatives().next().unwrap().handle();
    let mut trial = request(&prepared);
    trial.end = first.boundary();
    trial.participants = prepared
        .participants()
        .take(trial.end.view().participant_offset())
        .collect();
    trial.termination = PreparedTermination::Ordinary(first.handle());
    trial.alternative = Some(alternative);
    trial.context.base_direction = ParagraphBaseDirection::RightToLeft;
    trial.context.end = LineEnd::Soft;
    trial.context.available_inline = Some(20.0);
    let trial = prepared
        .select(trial)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert!(trial.feedback().advance > 20.0);
    assert!(trial.selected_line().request().alternative.is_some());
    let cuts = offers(&prepared, ConditionalPurpose::Overflow);
    assert_eq!(sources(&cuts), [1, 2, 3]);
    let mut selected = selection(&prepared, &cuts[1]);
    selected.context.base_direction = ParagraphBaseDirection::RightToLeft;
    let line = prepared
        .select(selected)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(
        glyphs(&line),
        [(Range::new(0, 1), 36), (Range::new(1, 2), 100)]
    );
    assert!(line.selected_line().request().alternative.is_none());
}

#[test]
fn conditional_provider_and_numeric_refusals_leave_prior_actual_results_valid() {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::arabic_ahem())
        .unwrap();
    let mut value = input(
        "\u{628}\u{61c}\u{61c}\u{61c}\u{61c}\u{61c}\u{61c}\u{628}",
        OverflowWrap::Anywhere,
    );
    value.base_direction = ParagraphBaseDirection::RightToLeft;
    let source = value.source.clone();
    let prepared = prepare(&mut system, value);
    let mut whole_request = request(&prepared);
    whole_request.context.base_direction = ParagraphBaseDirection::RightToLeft;
    let whole = prepared
        .select(whole_request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let prior = glyphs(&whole);
    assert!(prior.contains(&(Range::new(0, 1), 100)));
    assert!(prior.contains(&(Range::new(7, 8), 82)));
    let cuts = offers(&prepared, ConditionalPurpose::Overflow);
    let cut = cuts
        .iter()
        .find(|cut| cut.boundary().view().source_before().get() == 1)
        .unwrap();
    let mut value = selection(&prepared, cut);
    value.context.base_direction = ParagraphBaseDirection::RightToLeft;
    let selected = prepared.select(value.clone()).unwrap();
    assert_eq!(
        selected
            .finalize(SpacingAllocations::None)
            .unwrap_err()
            .detail(),
        Some(&crate::ErrorDetail::ShapingRefused {
            reason: crate::ShapingRefusalReason::ContextLimit
        })
    );
    value.context.available_inline = Some(f64::NAN);
    assert!(matches!(
        prepared.select(value).unwrap_err().detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric { .. })
    ));
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(glyphs(&whole), prior);
}

#[test]
fn raw_analysis_keeps_its_ordinary_model_without_additional_native_cuts() {
    let mut system = system();
    for overflow_wrap in [
        OverflowWrap::Normal,
        OverflowWrap::BreakWord,
        OverflowWrap::Anywhere,
    ] {
        let raw = AnalyzedSource::new(
            Source::new("AAA"),
            Style {
                overflow_wrap,
                ..style()
            },
        )
        .unwrap();
        let prepared = system.prepare(&raw, PreparationOptions::default()).unwrap();
        let expected = if overflow_wrap == OverflowWrap::Normal {
            vec![PreparedBreakKind::ParagraphEnd]
        } else {
            vec![
                PreparedBreakKind::Allowed,
                PreparedBreakKind::Allowed,
                PreparedBreakKind::ParagraphEnd,
            ]
        };
        assert_eq!(
            prepared
                .opportunity_views()
                .map(|view| view.kind())
                .collect::<Vec<_>>(),
            expected
        );
        for purpose in [ConditionalPurpose::Overflow, ConditionalPurpose::MinContent] {
            assert!(offers(&prepared, purpose).is_empty());
        }
        let other = system.prepare(&raw, PreparationOptions::default()).unwrap();
        let foreign_start = other.boundary_views().next().unwrap().handle();
        let start = prepared.boundary_views().next().unwrap().handle();
        assert!(matches!(
            reason(
                &prepared
                    .conditional_opportunities(
                        &foreign_start,
                        None,
                        &first_after(&prepared, &start),
                        ConditionalPurpose::MinContent,
                        &[]
                    )
                    .unwrap_err()
            ),
            ProtocolError::ForeignPreparation {
                kind: ProtocolHandleKind::Boundary,
                ..
            }
        ));
    }
}

#[test]
fn contextual_conditional_cuts_keep_child_holes_and_actual_incident_endpoints() {
    use crate::{
        Affinity, ContextualSourceInput, DocumentSegmentBreaks, Id, InlineBox, InlineBoxKind,
        ParagraphBoundaryDefinition, ParagraphBoundaryPolicy, ParagraphConnectionKind,
        ParagraphOrigin, ParagraphOriginKind, Size, SourceProcessingConnection,
        SourceProcessingEnvelope, SourceProcessingEvent, SourceProcessingParagraph,
        SourceProcessingScope, WhiteSpaceTrim, process_contextual_source,
    };
    let mut source = Source::new("AIB");
    source.inline_box(InlineBox::new(
        Id::from_u64(900),
        InlineBoxKind::OutOfFlow,
        1,
        Size::new(5.0, 10.0),
    ));
    let capture = ContextualSourceInput {
        source: source.clone(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        origins: (0..3)
            .map(|index| ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(index, index + 1),
                aggregate: Range::new(index, index + 1),
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
                topology: crate::SourceProcessingTopology::Unknown,
                used_direction: crate::ParagraphBaseDirection::LeftToRight,
                trim: WhiteSpaceTrim::default(),
                base_style: Style {
                    overflow_wrap: OverflowWrap::Anywhere,
                    ..style()
                },
            },
            SourceProcessingScope::OutOfFlowBlock {
                topology: crate::SourceProcessingTopology::Unknown,
                used_direction: crate::ParagraphBaseDirection::LeftToRight,
                trim: WhiteSpaceTrim::default(),
                base_style: style(),
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
    let processed = std::sync::Arc::new(process_contextual_source(capture).unwrap());
    let parent =
        AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap()).unwrap();
    let mut system = system();
    let prepared = system
        .prepare(&parent, PreparationOptions::default())
        .unwrap();
    let cuts = offers(&prepared, ConditionalPurpose::Overflow);
    assert_eq!(cuts.len(), 2);
    assert_ne!(cuts[0], cuts[1]);
    let sides: Vec<_> = cuts
        .iter()
        .map(|cut| {
            (
                cut.boundary().view().source_before().get(),
                cut.boundary().view().source_after().get(),
            )
        })
        .collect();
    assert_eq!(sides, [(1, 1), (1, 2)]);
    let first = prepared
        .select(selection(&prepared, &cuts[1]))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(first.selected_line().request().participants.len(), 2);
    assert_eq!(glyphs(&first), [(Range::new(0, 1), 35)]);
    let continuation = ordinary_receipt(
        &prepared,
        cuts[1].boundary(),
        prepared.opportunity_views().last().unwrap().handle(),
        None,
    );
    assert_eq!(glyphs(&continuation), [(Range::new(2, 3), 36)]);
    let continuation = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::Range {
            start: cuts[1].boundary(),
            end: prepared.boundary_views().last().unwrap().handle(),
        },
        lines: vec![LinePlacement {
            line: continuation,
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 16.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        }],
    })
    .unwrap();
    assert_eq!(continuation.lines()[0].range(), Range::new(2, 3));
    assert_eq!(continuation.source(), &source);
}

#[test]
fn conditional_eligibility_is_not_created_by_atomic_or_control_occurrences() {
    use crate::{
        Affinity, Id, InlineBox, InlineBoxKind, ParagraphAnchor, ParagraphAnchorItem,
        ParagraphBoundaryDefinition, ParagraphBoundaryPolicy, ParagraphControl,
        ParagraphControlKind, Size,
    };
    let mut system = system();
    for kind in [InlineBoxKind::InFlow, InlineBoxKind::OutOfFlow] {
        let mut value = input("", OverflowWrap::Anywhere);
        value.source.inline_box(InlineBox::new(
            Id::from_u64(50),
            kind,
            0,
            Size::new(10.0, 10.0),
        ));
        value.controls.push(ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        });
        value.anchor_order = [
            ParagraphAnchorItem::EmptyOrigin(0),
            ParagraphAnchorItem::Atomic(0),
            ParagraphAnchorItem::Control(0),
        ]
        .map(|item| ParagraphAnchor {
            item,
            affinity: Affinity::Before,
        })
        .to_vec();
        if kind == InlineBoxKind::InFlow {
            value.boundary_policies = [
                ParagraphBoundaryDefinition::AtomicBefore(0),
                ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .map(|definition| ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            })
            .to_vec();
        }
        let prepared = prepare(&mut system, value);
        assert!(offers(&prepared, ConditionalPurpose::Overflow).is_empty());
        assert!(offers(&prepared, ConditionalPurpose::MinContent).is_empty());
        assert_eq!(
            prepared
                .opportunity_views()
                .map(|view| view.kind())
                .collect::<Vec<_>>(),
            [PreparedBreakKind::ParagraphEnd]
        );
    }
    let mut value = input("AA", OverflowWrap::Anywhere);
    value.source.inline_box(InlineBox::new(
        Id::from_u64(51),
        InlineBoxKind::InFlow,
        1,
        Size::new(10.0, 10.0),
    ));
    value.anchor_order.push(ParagraphAnchor {
        item: ParagraphAnchorItem::Atomic(0),
        affinity: Affinity::Before,
    });
    value.boundary_policies = [
        ParagraphBoundaryDefinition::AtomicBefore(0),
        ParagraphBoundaryDefinition::AtomicAfter(0),
    ]
    .map(|definition| ParagraphBoundaryPolicy {
        definition,
        preceding_wrap: Wrap::Word,
        following_wrap: Wrap::Word,
        ancestor_wrap: Wrap::Word,
    })
    .to_vec();
    let prepared = prepare(&mut system, value);
    assert!(offers(&prepared, ConditionalPurpose::Overflow).is_empty());
    assert_eq!(
        prepared
            .opportunity_views()
            .map(|view| view.kind())
            .collect::<Vec<_>>(),
        [
            PreparedBreakKind::Allowed,
            PreparedBreakKind::Allowed,
            PreparedBreakKind::ParagraphEnd
        ]
    );
}

fn phantom_dispositions(lines: &[FinalizedLine]) -> Vec<crate::ConditionalDisposition> {
    lines
        .iter()
        .cloned()
        .map(crate::ConditionalDisposition::Phantom)
        .collect()
}

#[test]
fn finite_consumer_policy_discharge_reaches_a_real_manual_later_ceiling() {
    let mut system = system();
    let mut value = input("as\u{ad}sociate", OverflowWrap::Anywhere);
    value.base_style.size = 10.0;
    value.base_style.hyphens = crate::Hyphens::Manual;
    value.base_style.hyphenate_character =
        crate::HyphenateCharacter::String(crate::Text::from("A"));
    let p = prepare(&mut system, value);
    let start = p.boundary_views().next().unwrap().handle();
    let manual = p
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::Discretionary)
        .unwrap();
    assert_eq!(manual.boundary().view().source_before().get(), 3);
    let mut trial = request(&p);
    trial.end = manual.boundary();
    trial.termination = PreparedTermination::Ordinary(manual.handle());
    trial.alternative = Some(manual.alternatives().next().unwrap().handle());
    trial.participants = p
        .participants()
        .take(trial.end.view().participant_offset())
        .collect();
    trial.context.end = LineEnd::Soft;
    trial.context.available_inline = Some(70.0);
    let first = p
        .select(trial.clone())
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(first.feedback().advance, 30.0);
    assert!(first.has_in_flow_content());
    let ceiling = p
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    assert!(
        p.conditional_opportunities(&start, None, &ceiling, ConditionalPurpose::Overflow, &[])
            .is_err()
    );
    let disposition = ConditionalDisposition::ConsumerHyphenationPolicy(first);
    let cuts = p
        .conditional_opportunities(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            std::slice::from_ref(&disposition),
        )
        .unwrap();
    assert!(
        sources(&cuts).contains(&1),
        "policy discharge cannot advance the phantom frontier"
    );
    let cut = cuts
        .iter()
        .find(|cut| cut.boundary().view().source_before().get() == 8)
        .unwrap();
    let mut selected = selection(&p, cut);
    selected.context.available_inline = Some(70.0);
    let line = p
        .select(selected)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 70.0);
    assert!(line.selected_line().request().alternative.is_none());
    let second = p
        .select(trial)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(
        cuts,
        p.conditional_opportunities(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[ConditionalDisposition::ConsumerHyphenationPolicy(second)]
        )
        .unwrap()
    );
    assert!(
        p.conditional_opportunities(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::MinContent,
            std::slice::from_ref(&disposition)
        )
        .is_err()
    );
    assert!(
        p.conditional_opportunities(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[disposition.clone(), disposition]
        )
        .is_err()
    );
}

#[test]
fn finite_consumer_policy_receipts_preserve_barriers_start_and_context_identity() {
    let mut system = system();
    let mut value = input("as\u{ad}sociate", OverflowWrap::Anywhere);
    value.base_style.size = 10.0;
    value.base_style.hyphens = crate::Hyphens::Manual;
    value.base_style.hyphenate_character =
        crate::HyphenateCharacter::String(crate::Text::from("A"));
    let p = prepare(&mut system, value.clone());
    let foreign = prepare(&mut system, value);
    let start = p.boundary_views().next().unwrap().handle();
    let ceiling = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let make = |p: &PreparedText, width: f64, start_at: usize| {
        let manual = p
            .opportunity_views()
            .find(|o| o.kind() == PreparedBreakKind::Discretionary)
            .unwrap();
        let mut r = request(p);
        r.start = p
            .boundary_views()
            .find(|b| b.source_after().get() == start_at)
            .unwrap()
            .handle();
        r.end = manual.boundary();
        r.termination = PreparedTermination::Ordinary(manual.handle());
        r.alternative = Some(manual.alternatives().next().unwrap().handle());
        r.participants = p
            .participants()
            .skip(r.start.view().participant_offset())
            .take(r.end.view().participant_offset() - r.start.view().participant_offset())
            .collect();
        r.context.start = if start_at == 0 {
            LineStart::Paragraph
        } else {
            LineStart::Continuation
        };
        r.context.end = LineEnd::Soft;
        r.context.available_inline = Some(width);
        p.select(r)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap()
    };
    let query = |line| {
        p.conditional_opportunities(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[ConditionalDisposition::ConsumerHyphenationPolicy(line)],
        )
    };
    assert!(query(make(&foreign, 70.0, 0)).is_err());
    assert!(query(make(&p, 70.0, 1)).is_err());
    let a = query(make(&p, 70.0, 0)).unwrap();
    let b = query(make(&p, 71.0, 0)).unwrap();
    assert_ne!(
        a, b,
        "a changed finite policy trial is a changed capability context"
    );
    let mut ordinary = request(&p);
    ordinary.context.available_inline = Some(70.0);
    let ordinary = p
        .select(ordinary)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert!(
        query(ordinary).is_err(),
        "Paragraph/None cannot be discharged by hyphenation policy"
    );
    let plain = prepare(&mut system, input("X Y", OverflowWrap::Anywhere));
    let begin = plain.boundary_views().next().unwrap().handle();
    let offer = plain
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Allowed)
        .unwrap();
    let mut r = request(&plain);
    r.end = offer.boundary();
    r.termination = PreparedTermination::Ordinary(offer.handle());
    r.participants = plain
        .participants()
        .take(r.end.view().participant_offset())
        .collect();
    r.context.end = LineEnd::Soft;
    r.context.available_inline = Some(70.0);
    let line = plain
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert!(line.has_in_flow_content());
    let end = plain
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    assert!(
        plain
            .conditional_opportunities(
                &begin,
                None,
                &end,
                ConditionalPurpose::Overflow,
                &[ConditionalDisposition::ConsumerHyphenationPolicy(line)]
            )
            .is_err(),
        "meaningful ordinary None remains a genuine barrier"
    );
}
