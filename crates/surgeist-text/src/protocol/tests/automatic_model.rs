//! New issued resource/cut/portion observations: independently mapped source facts.
use super::*;
use crate::{
    HyphenateCharacter, HyphenationCounts, HyphenationProvenance, HyphenationResource, Hyphens,
    ParagraphConnection, ParagraphConnectionKind, ParagraphOrigin, ParagraphOriginKind,
    ParagraphTerminalEdge, Range, Text, Wrap,
};

fn automatic_style(size: f32) -> Style {
    Style {
        size,
        hyphens: Hyphens::Auto,
        locale: Some("en-US".into()),
        hyphenate_character: HyphenateCharacter::String(Text::from("A")),
        white_space: WhiteSpace::Preserve,
        ..style()
    }
}
fn fixture() -> Vec<u8> {
    let mut cmap: Vec<_> = (b'a'..=b'z').map(|cp| (u32::from(cp), 35)).collect();
    cmap.extend([
        (0x41, 35),
        (0x20, 3),
        (0x301, 35),
        (0xe9, 35),
        (0x61c, 35),
        (0x200d, 35),
    ]);
    crate::shaping_test_fonts::range_ligature_ahem(&[98, 99], &cmap)
}
fn input(source: Source, cuts: &[usize]) -> ParagraphInput {
    ParagraphInput {
        source: source.clone(),
        base_style: automatic_style(10.0),
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: cuts
            .windows(2)
            .map(|p| ParagraphOrigin {
                text_node: None,
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
            .map(|i| crate::ParagraphBoundaryPolicy {
                definition: crate::ParagraphBoundaryDefinition::Connection(i),
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            })
            .collect(),
        replacements: vec![],
        anchor_order: vec![],
    }
}
fn prepare(input: ParagraphInput) -> PreparedText {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(fixture()).unwrap();
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    );
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}
fn cut_at(p: &PreparedText, at: usize) -> PreparedHyphenationCut {
    p.hyphenation_cut_views()
        .find(|cut| {
            cut.boundary().view().source_before().get() == at
                && matches!(
                    cut.provenance(),
                    HyphenationProvenance::Pattern
                        | HyphenationProvenance::SourceException { .. }
                        | HyphenationProvenance::TextCorrection { .. }
                )
        })
        .unwrap()
        .handle()
}
fn first(p: &PreparedText, cut: &PreparedHyphenationCut) -> FinalizedLine {
    let offer = cut.view().opportunity().unwrap();
    let mut value = request(p);
    value.end = offer.view().boundary();
    value.participants = p
        .participants()
        .take(value.end.view().participant_offset())
        .collect();
    value.termination = PreparedTermination::Ordinary(offer.clone());
    value.alternative = Some(offer.view().alternatives().next().unwrap().handle());
    value.context.end = LineEnd::Soft;
    value.atomics = value
        .participants
        .iter()
        .filter_map(|participant| {
            if let PreparedParticipant::Atomic(occurrence) = participant {
                Some(AtomicAllocation {
                    occurrence: occurrence.clone(),
                    allocation: AtomicLineAllocation::OutOfFlow,
                })
            } else {
                None
            }
        })
        .collect();
    p.select(value)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}
fn place_first(p: &PreparedText, line: FinalizedLine) -> crate::Layout {
    let value = line.selected_line();
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::Range {
            start: value.request().start.clone(),
            end: value.request().end.clone(),
        },
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 20.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap()
}
#[test]
fn automatic_marked_pattern_keeps_both_preceding_egc_origins_and_actual_style_owner() {
    // Qualified raw TeX max/odd scoring: exten+acute+sive bytes[2,7] maps
    // original UTF16[2,6]. n+acute is ONE EGC and contributes ONE count.
    let mut source = Source::new("exten\u{301}sive");
    source.span(Range::new(5, 10), automatic_style(20.0));
    let p = prepare(input(source, &[0, 5, 10]));
    let word = p.word_views().next().unwrap();
    assert_eq!(word.query(), Some("exten\u{301}sive"));
    assert_eq!(word.resource(), HyphenationResource::EnUs2005);
    assert_eq!(word.provenance(), Some(HyphenationProvenance::Pattern));
    let cut = cut_at(&p, 6);
    assert_eq!(
        cut.view().counts(),
        Some(HyphenationCounts {
            total: 9,
            before: 5,
            after: 4
        })
    );
    assert_eq!(cut.view().style().size, 20.0);
    let origins: Vec<_> = p.origin_views().map(|o| o.handle()).collect();
    assert_eq!(
        cut.view()
            .preceding_source_pieces()
            .map(|piece| (piece.origin(), piece.source(), piece.local()))
            .collect::<Vec<_>>(),
        [
            (origins[0].clone(), Range::new(4, 5), Range::new(4, 5)),
            (origins[1].clone(), Range::new(5, 6), Range::new(5, 6))
        ]
    );
    let line = first(&p, &cut);
    assert!(line.run_views().all(|run| run.face().bytes() == fixture()));
    assert!(
        line.run_views()
            .any(|run| run.effective_instance().unwrap().font_size() == 20.0
                && run.glyphs().any(|glyph| glyph.range() == Range::new(6, 6)
                    && glyph.id() == 35
                    && glyph.advance() == 20.0))
    );
    let layout = place_first(&p, line);
    for (origin, range) in [
        (&origins[0], Range::new(4, 5)),
        (&origins[1], Range::new(5, 6)),
    ] {
        let observed = layout.source_range_fragments(origin, range).unwrap();
        let automatic: Vec<_> = observed
            .units()
            .filter(|u| u.hyphenation_cuts().next().is_some())
            .collect();
        assert_eq!(automatic.len(), 1);
        assert_eq!(automatic[0].hyphenation_cuts().next(), Some(cut.clone()));
        assert!(automatic[0].defining_sources().next().is_none());
        assert_eq!(
            automatic[0]
                .selected_intersections()
                .map(|piece| piece.aggregate_range())
                .collect::<Vec<_>>(),
            [range]
        );
    }
    assert!(
        layout
            .source_range_fragments(&origins[0], Range::new(3, 4))
            .unwrap()
            .units()
            .all(|unit| unit.hyphenation_cuts().next().is_none())
    );
}
#[test]
fn formatting_egc_owns_insertion_even_with_zero_count_and_advance() {
    let p = prepare(input(Source::new("exten\u{61c}sive"), &[0, 10]));
    let cut = cut_at(&p, 6);
    assert_eq!(
        cut.view().counts(),
        Some(HyphenationCounts {
            total: 9,
            before: 5,
            after: 4
        })
    );
    assert_eq!(
        cut.view()
            .preceding_source_pieces()
            .map(|piece| piece.source())
            .collect::<Vec<_>>(),
        [Range::new(5, 6)]
    );
    let line = first(&p, &cut);
    let layout = place_first(&p, line);
    let origin = p.origin_views().next().unwrap().handle();
    assert_eq!(
        layout
            .source_range_fragments(&origin, Range::new(5, 6))
            .unwrap()
            .units()
            .filter(|u| u.hyphenation_cuts().next() == Some(cut.clone()))
            .count(),
        1
    );
    assert!(
        layout
            .source_range_fragments(&origin, Range::new(4, 5))
            .unwrap()
            .units()
            .all(|u| u.hyphenation_cuts().next().is_none())
    );
    assert_eq!(
        layout.source().text().code_units(),
        "exten\u{61c}sive".encode_utf16().collect::<Vec<_>>()
    );
}
#[test]
fn long_multibyte_spelling_maps_real_resource_bytes_without_scalar_or_utf16_guessing() {
    let source = format!("{}extensive", "é".repeat(24));
    let p = prepare(input(Source::new(source.clone()), &[0, 33]));
    let cuts: Vec<_> = p
        .hyphenation_cut_views()
        .filter_map(|cut| {
            cut.opportunity()
                .map(|_| cut.boundary().view().source_before().get())
        })
        .collect();
    assert_eq!(cuts, [26, 29]);
    assert_eq!(
        p.word_views().next().unwrap().query(),
        Some(source.as_str())
    );
    let cut = cut_at(&p, 29);
    assert_eq!(
        cut.view().counts(),
        Some(HyphenationCounts {
            total: 33,
            before: 29,
            after: 4
        })
    );
    assert_eq!(
        first(&p, &cut)
            .selected_line()
            .request()
            .end
            .view()
            .source_before()
            .get(),
        29
    );
}
#[test]
fn source_exception_zero_and_shy_collision_retain_distinct_authored_provenance() {
    let p = prepare(input(Source::new("pre\u{ad}sent"), &[0, 8]));
    let word = p.word_views().next().unwrap();
    assert_eq!(word.query(), Some("present"));
    assert_eq!(
        word.provenance(),
        Some(HyphenationProvenance::SourceException { entry: "present" })
    );
    assert!(
        p.hyphenation_cut_views()
            .all(|cut| cut.provenance() == HyphenationProvenance::Source)
    );
    assert_eq!(p.alternative_views().count(), 1);
    let p = prepare(input(Source::new("as\u{ad}sociate"), &[0, 10]));
    assert_eq!(
        p.hyphenation_cut_views()
            .map(|cut| (
                cut.boundary().view().source_before().get(),
                cut.provenance()
            ))
            .collect::<Vec<_>>(),
        [
            (3, HyphenationProvenance::Source),
            (
                5,
                HyphenationProvenance::SourceException { entry: "associate" }
            )
        ]
    );
}
#[test]
fn measurement_portions_keep_indication_and_refuse_placement_and_foreign_owners() {
    let p = prepare(input(Source::new("as\u{ad}sociate X"), &[0, 12]));
    let word = p.word_views().next().unwrap();
    let portions = word.portions().unwrap();
    assert_eq!(portions.len(), 2);
    let mut values = Vec::new();
    for part in portions {
        let mut value = request(&p);
        value.start = part.start();
        value.end = part.end();
        value.participants = p
            .participants()
            .skip(value.start.view().participant_offset())
            .take(value.end.view().participant_offset() - value.start.view().participant_offset())
            .collect();
        value.alternative = part.alternative();
        value.termination = PreparedTermination::Measurement(part.clone());
        value.context.start = if value.start.view().participant_offset() == 0 {
            LineStart::Paragraph
        } else {
            LineStart::Continuation
        };
        value.context.end = LineEnd::Soft;
        let line = p
            .select(value.clone())
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        values.push(line.feedback().advance);
        let error = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared: p.clone(),
            coverage: PlacementCoverage::Range {
                start: part.start(),
                end: part.end(),
            },
            entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 20.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })],
        })
        .unwrap_err();
        assert_eq!(reason(&error), &ProtocolError::MeasurementNotPlaceable);
        let foreign = prepare(input(Source::new("as\u{ad}sociate X"), &[0, 12]));
        value.termination = PreparedTermination::Measurement(
            foreign.word_views().next().unwrap().portions().unwrap()[0].clone(),
        );
        assert!(p.select(value).is_err());
    }
    assert_eq!(values, [30.0, 70.0]);
}

fn ordinary_trial(
    p: &PreparedText,
    start: PreparedBoundary,
    offer: PreparedOpportunity,
    alternative: Option<PreparedAlternative>,
    width: Option<f64>,
) -> FinalizedLine {
    let mut value = request(p);
    value.start = start;
    value.end = offer.view().boundary();
    let from = value.start.view().participant_offset();
    let to = value.end.view().participant_offset();
    value.participants = p.participants().skip(from).take(to - from).collect();
    value.controls = value
        .participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Control(occurrence) => Some(ControlAllocation {
                occurrence: occurrence.clone(),
                strut: None,
                boundary: None,
            }),
            _ => None,
        })
        .collect();
    value.termination = PreparedTermination::Ordinary(offer);
    value.alternative = alternative;
    value.context.start = if from == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    value.context.end = LineEnd::Soft;
    value.context.available_inline = width;
    p.select(value)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}
#[test]
fn conditional_automatic_dispositions_preserve_variant_custody_and_semantic_identity() {
    let mut value = input(Source::new("associate"), &[0, 9]);
    value.base_style.overflow_wrap = crate::OverflowWrap::Anywhere;
    let p = prepare(value);
    let start = p.boundary_views().next().unwrap().handle();
    let ceiling = p.opportunity_views().last().unwrap().handle();
    let automatic: Vec<_> = p
        .opportunity_views()
        .filter(|offer| offer.kind() == PreparedBreakKind::Discretionary)
        .map(|offer| offer.handle())
        .collect();
    assert_eq!(automatic.len(), 2);
    let receipts = |width| {
        automatic
            .iter()
            .map(|offer| {
                ordinary_trial(
                    &p,
                    start.clone(),
                    offer.clone(),
                    Some(offer.view().alternatives().next().unwrap().handle()),
                    Some(width),
                )
            })
            .collect::<Vec<_>>()
    };
    let trials = receipts(10.0);
    assert_eq!(
        trials
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [30.0, 50.0]
    );
    let dispositions = trials
        .iter()
        .cloned()
        .map(crate::ConditionalDisposition::AutoFitExhausted)
        .collect::<Vec<_>>();
    let cuts = p
        .source_segments()
        .next()
        .unwrap()
        .conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &dispositions,
            None,
        )
        .and_then(|search| search.begin(&[]))
        .map(|batch| batch.opportunities().to_vec())
        .unwrap();
    assert_eq!(
        cuts.iter()
            .map(|cut| cut.boundary().view().source_before().get())
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6, 7, 8]
    );
    assert_eq!(
        cuts,
        p.source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &receipts(20.0)
                    .into_iter()
                    .map(crate::ConditionalDisposition::AutoFitExhausted)
                    .collect::<Vec<_>>(),
                None
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .unwrap()
    );
    for bad in [
        vec![],
        vec![dispositions[1].clone(), dispositions[0].clone()],
        vec![dispositions[0].clone(), dispositions[0].clone()],
    ] {
        assert!(
            p.source_segments()
                .next()
                .unwrap()
                .conditional_search(
                    &start,
                    None,
                    &ceiling,
                    ConditionalPurpose::Overflow,
                    &bad,
                    None
                )
                .and_then(|search| search.begin(&[]))
                .map(|batch| batch.opportunities().to_vec())
                .is_err()
        );
    }
    assert!(
        p.source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::MinContent,
                &dispositions,
                None
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .is_err()
    );
    let intrinsic = automatic
        .iter()
        .map(|offer| {
            crate::ConditionalDisposition::AutoIntrinsicRefinement(
                offer.view().alternatives().next().unwrap().handle(),
            )
        })
        .collect::<Vec<_>>();
    let minimum = p
        .source_segments()
        .next()
        .unwrap()
        .conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::MinContent,
            &intrinsic,
            None,
        )
        .and_then(|search| search.begin(&[]))
        .map(|batch| batch.opportunities().to_vec())
        .unwrap();
    assert_eq!(
        minimum
            .iter()
            .map(|cut| cut.boundary().view().source_before().get())
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6, 7, 8]
    );
    assert!(
        p.source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &intrinsic,
                None
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .is_err()
    );
    let foreign = prepare(input(Source::new("associate"), &[0, 9]));
    let wrong = ordinary_trial(
        &foreign,
        foreign.boundary_views().next().unwrap().handle(),
        foreign.opportunity_views().next().unwrap().handle(),
        Some(foreign.alternative_views().next().unwrap().handle()),
        Some(10.0),
    );
    let mut bad = dispositions.clone();
    bad[0] = crate::ConditionalDisposition::AutoFitExhausted(wrong);
    assert!(
        p.source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &bad,
                None
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .is_err()
    );
}
#[test]
fn automatic_manual_priority_requires_exact_admitted_portion_and_measurements_never_become_phantoms()
 {
    let mut value = input(Source::new("as\u{ad}sociate"), &[0, 10]);
    value.base_style.overflow_wrap = crate::OverflowWrap::Anywhere;
    let p = prepare(value);
    let portions = p.word_views().next().unwrap().portions().unwrap();
    let part = portions[1].clone();
    let mut value = request(&p);
    value.start = part.start();
    value.end = part.end();
    value.termination = PreparedTermination::Measurement(part.clone());
    value.alternative = part.alternative();
    value.participants = p
        .participants()
        .skip(value.start.view().participant_offset())
        .take(value.end.view().participant_offset() - value.start.view().participant_offset())
        .collect();
    value.context.start = LineStart::Continuation;
    value.context.end = LineEnd::Soft;
    value.context.available_inline = Some(80.0);
    let portion = p
        .select(value)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(portion.feedback().advance, 70.0);
    let automatic = p
        .alternative_views()
        .find(|alt| {
            matches!(
                alt.definition(),
                crate::PreparedAlternativeDefinition::Automatic { .. }
            )
        })
        .unwrap()
        .handle();
    let ceiling = p.opportunity_views().last().unwrap().handle();
    let dispositions = [crate::ConditionalDisposition::AutoManualPriority {
        alternative: automatic.clone(),
        portion: portion.clone(),
    }];
    assert!(
        !p.source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &part.start(),
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &dispositions,
                None
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .unwrap()
            .is_empty()
    );
    assert!(
        p.source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &part.start(),
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &[crate::ConditionalDisposition::Phantom(portion.clone())],
                None
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .is_err()
    );
    assert!(
        p.source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &part.start(),
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &[crate::ConditionalDisposition::AutoFitExhausted(portion)],
                None
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .is_err()
    );
    let manual = p
        .alternative_views()
        .find(|alt| {
            matches!(
                alt.definition(),
                crate::PreparedAlternativeDefinition::Source { .. }
            )
        })
        .unwrap()
        .handle();
    let impossible = [crate::ConditionalDisposition::AutoIntrinsicRefinement(
        manual,
    )];
    assert!(
        p.source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &p.boundary_views().next().unwrap().handle(),
                None,
                &ceiling,
                ConditionalPurpose::MinContent,
                &impossible,
                None
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .is_err()
    );
}

#[test]
fn transparent_child_holes_keep_automatic_source_anchor_separate_from_mixed_cut_custody() {
    use crate::{
        Affinity, ContextualSourceInput, DocumentSegmentBreaks, Id, InlineBox, InlineBoxKind, Size,
        SourceProcessingConnection, SourceProcessingEnvelope, SourceProcessingEvent as Event,
        SourceProcessingParagraph, SourceProcessingScope, WhiteSpaceTrim,
        process_contextual_source,
    };
    use std::sync::Arc;
    // Parent asso+ciate is the same nine-letter word; child I contributes
    // neither spelling nor counts. A genuine parent empty occurrence follows
    // the child, at5; it cannot replace the preceding text's source authority.
    let mut source = Source::new("assoIciate");
    source.inline_box(InlineBox::new(
        Id::from_u64(67),
        InlineBoxKind::OutOfFlow,
        4,
        Size::new(5.0, 10.0),
    ));
    let capture = ContextualSourceInput {
        joint_flows: Vec::new(),
        scope_alignment_limits: None,
        computed_line_boundaries: vec![],
        source: source.clone(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        origins: [
            Range::new(0, 4),
            Range::new(4, 5),
            Range::new(5, 5),
            Range::new(5, 10),
        ]
        .into_iter()
        .map(|range| ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: range,
            aggregate: range,
            kind: ParagraphOriginKind::Authored,
        })
        .collect(),
        connections: vec![
            SourceProcessingConnection {
                before_origin: 0,
                after_origin: 2,
                kind: ParagraphConnectionKind::Continuous,
            },
            SourceProcessingConnection {
                before_origin: 2,
                after_origin: 3,
                kind: ParagraphConnectionKind::Continuous,
            },
        ],
        controls: vec![],
        replacements: vec![],
        boundary_policies: (0..2)
            .map(|i| crate::ParagraphBoundaryPolicy {
                definition: crate::ParagraphBoundaryDefinition::Connection(i),
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            })
            .collect(),
        scopes: vec![
            SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: crate::SourceProcessingTopology::Unknown,
                trim: WhiteSpaceTrim::default(),
                base_style: automatic_style(10.0),
                used_direction: ParagraphBaseDirection::LeftToRight,
            },
            SourceProcessingScope::OutOfFlowBlock {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: crate::SourceProcessingTopology::Unknown,
                trim: WhiteSpaceTrim::default(),
                base_style: automatic_style(10.0),
                used_direction: ParagraphBaseDirection::LeftToRight,
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
            Event::EnterScope(0),
            Event::EnterParagraph(0),
            Event::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            Event::Atomic {
                atomic: 0,
                affinity: Affinity::After,
            },
            Event::EnterScope(1),
            Event::EnterParagraph(1),
            Event::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            Event::ExitParagraph(1),
            Event::ExitScope(1),
            Event::Connection(0),
            Event::Origin {
                origin: 2,
                affinity: Affinity::After,
            },
            Event::Connection(1),
            Event::Origin {
                origin: 3,
                affinity: Affinity::After,
            },
            Event::ExitParagraph(0),
            Event::ExitScope(0),
        ],
    };
    let processed = Arc::new(process_contextual_source(capture).unwrap());
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        crate::Wrap::Word,
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(fixture()).unwrap();
    let p = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(p.word_views().next().unwrap().query(), Some("associate"));
    let cuts: Vec<_> = p
        .hyphenation_cut_views()
        .filter(|cut| {
            cut.counts()
                == Some(HyphenationCounts {
                    total: 9,
                    before: 4,
                    after: 5,
                })
        })
        .map(|cut| cut.handle())
        .collect();
    assert_eq!(cuts.len(), 3);
    assert_ne!(cuts[0].view().boundary(), cuts[1].view().boundary());
    assert_eq!(
        cuts.iter()
            .map(|cut| cut.view().boundary().view().source_before().get())
            .collect::<Vec<_>>(),
        [4, 4, 5]
    );
    for cut in cuts {
        assert_eq!(
            cut.view()
                .preceding_source_pieces()
                .map(|piece| piece.source())
                .collect::<Vec<_>>(),
            [Range::new(3, 4)]
        );
        let line = first(&p, &cut);
        let selected = line.selected_line();
        let alternative = selected.request().alternative.as_ref().unwrap();
        assert_eq!(alternative.view().insertion_anchor().get(), 4);
        assert!(
            line.fragment_views()
                .filter(|fragment| matches!(
                    fragment.owner(),
                    crate::FinalizedFragmentOwner::Alternative(_)
                ))
                .all(
                    |fragment| fragment.aggregate_range() == Some(Range::new(4, 4))
                        && fragment.local_range() == Some(Range::new(4, 4))
                )
        );
        assert_eq!(line.feedback().advance, 50.0);
    }
}

#[test]
fn automatic_resource_language_is_whole_word_but_cut_style_is_logical_preceding() {
    let mut source = Source::new("associate");
    let mut manual = automatic_style(20.0);
    manual.hyphens = Hyphens::Manual;
    source.span(Range::new(0, 3), manual);
    let p = prepare(input(source, &[0, 3, 9]));
    let observed: Vec<_> = p
        .hyphenation_cut_views()
        .map(|cut| {
            (
                cut.boundary().view().source_before().get(),
                cut.opportunity().is_some(),
                cut.style().size,
            )
        })
        .collect();
    assert_eq!(observed, [(2, false, 20.0), (4, true, 10.0)]);
    assert_eq!(p.word_views().count(), 1);
    let mut source = Source::new("as\u{ad}sociate");
    let mut other = automatic_style(20.0);
    other.locale = Some("en-GB".into());
    source.span(Range::new(3, 10), other);
    let p = prepare(input(source, &[0, 3, 10]));
    let word = p.word_views().next().unwrap();
    assert_eq!(word.query(), Some("associate"));
    assert_eq!(word.resource(), HyphenationResource::UnavailableLanguage);
    assert_eq!(word.provenance(), None);
    assert_eq!(p.alternative_views().count(), 1);
    assert!(
        matches!(p.alternative_views().next().unwrap().definition(),crate::PreparedAlternativeDefinition::Source {source,..} if source==Range::new(2,3))
    );
}

#[test]
fn computed_mark_shares_real_egc_without_duplicate_source_attribution() {
    let mut value = input(Source::new("exten\u{301}sive"), &[0, 10]);
    value.base_style.hyphenate_character = HyphenateCharacter::String(Text::from("\u{301}"));
    let p = prepare(value);
    let cut = cut_at(&p, 6);
    let layout = place_first(&p, first(&p, &cut));
    let origin = p.origin_views().next().unwrap().handle();
    let observed = layout
        .source_range_fragments(&origin, Range::new(4, 5))
        .unwrap();
    let units: Vec<_> = observed.units().collect();
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].hyphenation_cuts().next(), Some(cut));
    assert_eq!(
        units[0]
            .selected_intersections()
            .map(|piece| piece.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(4, 5)]
    );
    assert_eq!(
        units[0]
            .expanded_intersections()
            .map(|piece| piece.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(4, 5), Range::new(5, 6)]
    );
    assert!(units[0].defining_sources().next().is_none());
}

#[test]
fn automatic_mixed_directional_occurrences_retain_actual_insertion_side() {
    let mut value = input(Source::new("associate"), &[0, 9]);
    value.controls = vec![
        crate::ParagraphControl {
            at: 4,
            kind: crate::ParagraphControlKind::RightToLeftOverride,
        },
        crate::ParagraphControl {
            at: 4,
            kind: crate::ParagraphControlKind::PopDirectionalFormat,
        },
    ];
    value.anchor_order = (0..2)
        .map(|i| crate::ParagraphAnchor {
            item: crate::ParagraphAnchorItem::Control(i),
            affinity: crate::Affinity::After,
        })
        .collect();
    let p = prepare(value);
    let cuts: Vec<_> = p
        .hyphenation_cut_views()
        .filter(|cut| cut.boundary().view().source_before().get() == 4)
        .map(|cut| cut.handle())
        .collect();
    assert_eq!(cuts.len(), 3);
    // X9 format controls inherit preceding original level0. The three cuts
    // still attach to distinct original Text/RLO/PDF occurrences, rather than
    // resolving each indication inside the original override stack.
    for (cut, controls) in cuts.into_iter().zip([0, 1, 2]) {
        let mut value = request(&p);
        let offer = cut.view().opportunity().unwrap();
        value.end = offer.view().boundary();
        value.participants = p
            .participants()
            .take(value.end.view().participant_offset())
            .collect();
        value.termination = PreparedTermination::Ordinary(offer.clone());
        value.alternative = Some(offer.view().alternatives().next().unwrap().handle());
        value.context.end = LineEnd::Soft;
        value.controls = value
            .participants
            .iter()
            .filter_map(|participant| {
                if let PreparedParticipant::Control(control) = participant {
                    Some({
                        let occurrence = control.clone();
                        ControlAllocation {
                            boundary: matches!(
                                occurrence.view().kind(),
                                crate::ParagraphControlKind::InlineBoundaryStart
                                    | crate::ParagraphControlKind::InlineBoundaryEnd
                            )
                            .then_some(crate::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                            occurrence,
                            strut: None,
                        }
                    })
                } else {
                    None
                }
            })
            .collect();
        let line = p
            .select(value)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        let attachment = line.inner.selected.presentation.domains[0].attachment;
        let super::super::prepared::presentation::Attachment::Before { participant, atom } =
            attachment
        else {
            panic!("Before domain")
        };
        if controls == 0 {
            assert!(atom.is_some());
            assert!(matches!(
                p.inner.tables.participants[participant],
                super::super::prepared::ParticipantData::Text(_)
            ));
        } else {
            assert!(atom.is_none());
            let super::super::prepared::ParticipantData::Control(i) =
                p.inner.tables.participants[participant]
            else {
                panic!("actual formatting control")
            };
            assert_eq!(p.inner.tables.controls[i].occurrence, controls - 1);
        }
        assert_eq!(line.feedback().advance, 50.0);
        let computed: Vec<_> = line
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .filter(|glyph| glyph.range() == Range::new(4, 4))
                    .map(|glyph| glyph.id())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(computed, [35]);
        assert_eq!(
            line.inner
                .data
                .clusters
                .iter()
                .filter(|cluster| cluster.source == Range::new(4, 4))
                .map(|cluster| (cluster.level, cluster.advance))
                .collect::<Vec<_>>(),
            [(0, 10.0)]
        );
        assert_eq!(line.selected_line().request().controls.len(), controls);
    }
}

#[test]
fn automatic_indication_uses_cut_primary_and_retains_resources_after_catalogue_refresh() {
    let primary = crate::shaping_test_fonts::range_ligature_ahem(
        &[98, 99],
        &[
            (0x61, 35),
            (0x73, 35),
            (0x6f, 35),
            (0x63, 35),
            (0x69, 35),
            (0x74, 35),
            (0x65, 35),
            (0x2d, 15),
        ],
    );
    let secondary = crate::shaping_test_fonts::range_ligature_ahem(
        &[98, 99],
        &[
            (0x61, 35),
            (0x73, 35),
            (0x6f, 35),
            (0x63, 35),
            (0x69, 35),
            (0x74, 35),
            (0x65, 35),
            (0x2010, 36),
            (0x2d, 15),
        ],
    );
    let face = skrifa::FontRef::new(&secondary).unwrap();
    let mut name = face
        .table_data(skrifa::Tag::new(b"name"))
        .unwrap()
        .as_bytes()
        .to_vec();
    for i in 0..name.len().saturating_sub(7) {
        if &name[i..i + 8] == b"\0A\0h\0e\0m" {
            name[i + 1] = b'B';
        }
    }
    let secondary = crate::test_fonts::replace_tables(&secondary, vec![(*b"name", name)]);
    let mut source = Source::new("associate");
    let own = Style {
        hyphenate_character: HyphenateCharacter::Auto,
        ..automatic_style(10.0)
    };
    let adjacent = Style {
        font: Font::new().family("Bhem"),
        ..own.clone()
    };
    source.span(Range::new(4, 9), adjacent);
    let mut value = input(source, &[0, 4, 9]);
    value.base_style = own;
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(primary.clone()).unwrap();
    system.register_fonts(secondary).unwrap();
    let p = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let cut = cut_at(&p, 4);
    let alternative = cut
        .view()
        .opportunity()
        .unwrap()
        .view()
        .alternatives()
        .next()
        .unwrap()
        .handle();
    assert_eq!(alternative.view().text().code_units(), [45]);
    let catalogue = p.font_catalogue_identity().clone();
    system.refresh_fonts().unwrap();
    drop(system);
    let line = first(&p, &cut);
    assert_eq!(line.feedback().advance, 50.0);
    assert!(
        line.run_views()
            .all(|run| run.face().catalogue_identity() == &catalogue)
    );
    assert!(line.run_views().any(|run| run.face().bytes() == primary
        && run.glyphs().any(|glyph| glyph.range() == Range::new(4, 4)
            && glyph.id() == 15
            && glyph.advance() == 10.0)));
}
#[test]
fn automatic_words_respect_actual_disconnections_at_connected_origin_boundaries() {
    let continuous = prepare(input(Source::new("associate"), &[0, 4, 9]));
    assert_eq!(
        continuous
            .word_views()
            .map(|word| word.query().unwrap().to_owned())
            .collect::<Vec<_>>(),
        ["associate"]
    );
    let mut value = input(Source::new("associate"), &[0, 4, 9]);
    value.connections[0].kind = ParagraphConnectionKind::Disconnected;
    let disconnected = prepare(value);
    assert_eq!(
        disconnected
            .word_views()
            .map(|word| word.query().unwrap().to_owned())
            .collect::<Vec<_>>(),
        ["asso", "ciate"]
    );
    assert!(
        disconnected
            .hyphenation_cut_views()
            .all(|cut| cut.boundary().view().source_before().get() != 4)
    );
    // A real isolated UTF16 unit stays distinct from a scalar replacement
    // character. Neither may be silently deleted to spell the exception.
    for text in [
        Text::from_code_units(vec![97, 115, 0xd800, 115, 111, 99, 105, 97, 116, 101]),
        Text::from("as\u{fffd}sociate"),
    ] {
        let source = Source::new(text);
        let len = source.text().len();
        let p = prepare(input(source.clone(), &[0, len]));
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            source.text().code_units()
        );
        assert!(p.word_views().all(|word| word.query() != Some("associate")));
        assert!(
            p.hyphenation_cut_views()
                .all(|cut| cut.boundary().view().source_before().get() != 4)
        );
    }
}

#[test]
fn automatic_allowed_endpoint_keeps_its_nonhyphenating_variant_as_a_real_barrier() {
    for mode in [crate::WordBreak::Normal, crate::WordBreak::KeepAll] {
        let mut value = input(Source::new("associate"), &[0, 9]);
        value.base_style.word_break = mode;
        value.base_style.overflow_wrap = crate::OverflowWrap::Anywhere;
        value.controls.push(paragraph::ParagraphControl {
            at: 2,
            kind: paragraph::ParagraphControlKind::Wbr {
                style: Box::new(value.base_style.clone()),
            },
        });
        value.anchor_order.push(paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Control(0),
            affinity: crate::Affinity::After,
        });
        let p = prepare(value);
        let word = p.word_views().next().unwrap();
        assert_eq!(word.query(), Some("associate"));
        assert_eq!(word.resource(), HyphenationResource::EnUs2005);
        assert_eq!(
            word.provenance(),
            Some(HyphenationProvenance::SourceException { entry: "associate" })
        );
        let wbr = p.participants().position(|part| matches!(part, PreparedParticipant::Control(control) if matches!(control.view().kind(), paragraph::ParagraphControlKind::Wbr { .. }))).unwrap();
        let after_wbr = p
            .boundary_views()
            .find(|boundary| boundary.participant_offset() == wbr + 1)
            .unwrap()
            .handle();
        let start = p.boundary_views().next().unwrap().handle();
        let cut = p
            .hyphenation_cut_views()
            .find(|cut| {
                cut.boundary() == after_wbr
                    && cut.provenance()
                        == HyphenationProvenance::SourceException { entry: "associate" }
            })
            .unwrap()
            .handle();
        let offer = cut.view().opportunity().unwrap();
        assert_eq!(offer.view().boundary(), after_wbr);
        assert_eq!(after_wbr.view().source_before().get(), 2);
        assert_eq!(offer.view().kind(), PreparedBreakKind::Allowed);
        let auto = offer.view().alternatives().next().unwrap().handle();
        assert!(
            matches!(auto.view().definition(), crate::PreparedAlternativeDefinition::Automatic { cut: actual } if actual == cut)
        );
        let ordinary = ordinary_trial(&p, start.clone(), offer.clone(), None, Some(10.0));
        assert_eq!(ordinary.feedback().advance, 20.0);
        assert!(ordinary.selected_line().request().alternative.is_none());
        let automatic = ordinary_trial(
            &p,
            start.clone(),
            offer.clone(),
            Some(auto.clone()),
            Some(10.0),
        );
        assert_eq!(automatic.feedback().advance, 30.0);
        // Preserve the original nonzero source1-to-cut2 automatic trial.
        let start = p
            .boundary_views()
            .find(|row| row.participant_offset() == 1)
            .unwrap()
            .handle();
        let ceiling = cut_at(&p, 4).view().opportunity().unwrap();
        assert_eq!(ceiling.view().boundary().view().source_before().get(), 4);
        let automatic = ordinary_trial(
            &p,
            start.clone(),
            offer.clone(),
            Some(auto.clone()),
            Some(10.0),
        );
        assert_eq!(automatic.feedback().advance, 20.0);
        // The actual pre-Wbr boundary excludes its separate earlier Auto offer.
        // Failure must identify this Wbr's None sibling, not that earlier barrier.
        let start = p
            .boundary_views()
            .find(|boundary| boundary.participant_offset() == wbr)
            .unwrap()
            .handle();
        let automatic = ordinary_trial(
            &p,
            start.clone(),
            offer.clone(),
            Some(auto.clone()),
            Some(10.0),
        );
        // An automatic disposition cannot discharge that endpoint's real None
        // variant, even when the exact automatic trial itself is supplied.
        let error = p
            .source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &[crate::ConditionalDisposition::AutoFitExhausted(automatic)],
                None,
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .unwrap_err();
        assert!(
            matches!(reason(&error), ProtocolError::InvalidConditionalReceipt { receipt: 0, opportunity } if *opportunity == offer.index)
        );
        let error = p
            .source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::MinContent,
                &[crate::ConditionalDisposition::AutoIntrinsicRefinement(
                    auto.clone(),
                )],
                None,
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .unwrap_err();
        assert!(
            matches!(reason(&error), ProtocolError::InvalidConditionalReceipt { receipt: 0, opportunity } if *opportunity == offer.index)
        );
        // Complete coverage still works: the genuine zero-flow Wbr None requires
        // its own phantom receipt before the separately authenticated Auto variant.
        for purpose in [ConditionalPurpose::Overflow, ConditionalPurpose::MinContent] {
            let width = (purpose == ConditionalPurpose::Overflow).then_some(10.0);
            let none = ordinary_trial(&p, start.clone(), offer.clone(), None, width);
            assert_eq!(none.feedback().advance, 0.0);
            let automatic =
                ordinary_trial(&p, start.clone(), offer.clone(), Some(auto.clone()), width);
            let auto_disposition = if purpose == ConditionalPurpose::Overflow {
                crate::ConditionalDisposition::AutoFitExhausted(automatic)
            } else {
                crate::ConditionalDisposition::AutoIntrinsicRefinement(auto.clone())
            };
            let cuts = p
                .source_segments()
                .next()
                .unwrap()
                .conditional_search(
                    &start,
                    None,
                    &ceiling,
                    purpose,
                    &[
                        crate::ConditionalDisposition::Phantom(none),
                        auto_disposition,
                    ],
                    None,
                )
                .and_then(|search| search.begin(&[]))
                .map(|batch| batch.opportunities().to_vec())
                .unwrap();
            assert!(
                cuts.iter()
                    .any(|cut| cut.boundary().view().source_before().get() == 3)
            );
        }
    }
}
#[test]
fn automatic_range_units_keep_optional_ligature_shares_distinct() {
    let mut cmap: Vec<_> = (b'a'..=b'z').map(|cp| (u32::from(cp), 35)).collect();
    cmap.push((0x41, 35));
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(&[35, 35], &cmap);
    let mut value = input(Source::new("extensive"), &[0, 9]);
    value.base_style.hyphenate_character = HyphenateCharacter::String(Text::from("AA"));
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let p = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let cut = cut_at(&p, 5);
    let line = first(&p, &cut);
    // Seven emitted A's form three actual optional AA ligatures plus A.
    assert_eq!(line.feedback().advance, 40.0);
    assert_eq!(
        line.run_views()
            .flat_map(|run| run.glyphs().map(|glyph| glyph.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [58, 58, 58, 35]
    );
    let origin = p.origin_views().next().unwrap().handle();
    let layout = place_first(&p, line);
    let fragments = layout
        .source_range_fragments(&origin, Range::new(4, 5))
        .unwrap();
    let auto: Vec<_> = fragments
        .units()
        .filter(|unit| unit.hyphenation_cuts().next() == Some(cut.clone()))
        .collect();
    assert_eq!(auto.len(), 2);
    assert_eq!(
        auto.iter()
            .map(|unit| unit
                .pieces()
                .map(|piece| piece.rect().size.width)
                .sum::<f64>())
            .collect::<Vec<_>>(),
        [5.0, 10.0]
    );
    assert!(!auto[0].same_unit(&auto[1]));
    let outside = layout
        .source_range_fragments(&origin, Range::new(5, 6))
        .unwrap_err();
    assert!(
        matches!(outside.detail(),Some(crate::ErrorDetail::SourceRangeFragments { reason:crate::SourceRangeFragmentError::OutsidePlacement {aggregate} }) if *aggregate==Range::new(5,6))
    );
}

#[test]
fn line_limit_dispositions_authenticate_exact_automatic_variants_and_purpose() {
    let mut value = input(Source::new("associate"), &[0, 9]);
    value.base_style.overflow_wrap = crate::OverflowWrap::Anywhere;
    let p = prepare(value.clone());
    let start = p.boundary_views().next().unwrap().handle();
    let ceiling = p.opportunity_views().last().unwrap().handle();
    let alternatives = p
        .opportunity_views()
        .flat_map(|view| {
            view.alternatives()
                .map(|alt| alt.handle())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(alternatives.len(), 2);
    let dispositions = |limit| {
        alternatives
            .iter()
            .cloned()
            .map(|alternative| crate::ConditionalDisposition::AutoLineLimit { alternative, limit })
            .collect::<Vec<_>>()
    };
    let zero = dispositions(0);
    let finite = dispositions(1);
    let query = |purpose, rows: &[_]| {
        p.source_segments()
            .next()
            .unwrap()
            .conditional_search(&start, None, &ceiling, purpose, rows, None)
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
    };
    let overflow = query(ConditionalPurpose::Overflow, &finite).unwrap();
    assert_eq!(
        overflow
            .iter()
            .map(|cut| cut.boundary().view().source_before().get())
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6, 7, 8]
    );
    // Suppression covers inventory, not a phantom prefix: even before first Auto2.
    assert_eq!(
        overflow,
        query(ConditionalPurpose::Overflow, &dispositions(1)).unwrap()
    );
    assert_ne!(
        overflow,
        query(ConditionalPurpose::Overflow, &zero).unwrap()
    );
    assert!(query(ConditionalPurpose::MinContent, &finite).is_err());
    let minimum = query(ConditionalPurpose::MinContent, &zero).unwrap();
    assert_ne!(overflow, minimum);
    assert!(query(ConditionalPurpose::Overflow, &[]).is_err());
    assert!(
        query(
            ConditionalPurpose::Overflow,
            &[finite[1].clone(), finite[0].clone()]
        )
        .is_err()
    );
    assert!(
        query(
            ConditionalPurpose::Overflow,
            &[finite[0].clone(), finite[0].clone()]
        )
        .is_err()
    );
    let foreign = prepare(value);
    let mut bad = finite.clone();
    bad[0] = crate::ConditionalDisposition::AutoLineLimit {
        alternative: foreign.alternative_views().next().unwrap().handle(),
        limit: 1,
    };
    assert!(query(ConditionalPurpose::Overflow, &bad).is_err());
    let mut request = request(&p);
    request.end = overflow[0].boundary();
    request.participants = p
        .participants()
        .take(request.end.view().participant_offset())
        .collect();
    request.termination = PreparedTermination::Conditional(overflow[0].clone());
    request.context.end = LineEnd::Soft;
    request.context.available_inline = Some(10.0);
    let line = p
        .select(request.clone())
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.feedback().advance, 10.0);
    request.context.available_inline = None;
    assert!(p.select(request).is_err());
    // Authored SHY has independent permission and cannot be excluded by line policy.
    let manual = prepare(input(Source::new("as\u{ad}sociate"), &[0, 10]));
    let first = manual.opportunity_views().next().unwrap();
    let manual_disposition = crate::ConditionalDisposition::AutoLineLimit {
        alternative: first.alternatives().next().unwrap().handle(),
        limit: 0,
    };
    assert!(
        manual
            .source_segments()
            .next()
            .unwrap()
            .conditional_search(
                &manual.boundary_views().next().unwrap().handle(),
                None,
                &manual.opportunity_views().last().unwrap().handle(),
                ConditionalPurpose::Overflow,
                &[manual_disposition],
                None
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .is_err()
    );
}

#[test]
fn word_ordinary_fallbacks_keep_genuine_custody_and_require_the_influencing_text_occurrence() {
    let p = prepare(input(Source::new("A associate"), &[0, 2, 11]));
    let start = p.boundary_views().next().unwrap().handle();
    let word = p
        .word_views()
        .find(|word| word.query() == Some("associate"))
        .unwrap();
    let offers = word.ordinary_fallbacks(&start).unwrap();
    assert_eq!(offers.len(), 1);
    assert_eq!(offers[0].view().kind(), PreparedBreakKind::Allowed);
    assert_eq!(offers[0].view().boundary().view().source_before().get(), 2);
    assert!(
        word.ordinary_fallbacks(&offers[0].view().boundary())
            .unwrap()
            .is_empty()
    );
    let inside = cut_at(&p, 4).view().boundary();
    assert!(word.ordinary_fallbacks(&inside).unwrap().is_empty());
    let foreign = prepare(input(Source::new("A associate"), &[0, 11]));
    let error = word
        .ordinary_fallbacks(&foreign.boundary_views().next().unwrap().handle())
        .unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextProtocol {
            reason: crate::ProtocolError::ForeignPreparation {
                kind: crate::ProtocolHandleKind::Boundary,
                ..
            }
        })
    ));
    assert!(
        p.word_views()
            .find(|word| word.query() == Some("A"))
            .unwrap()
            .ordinary_fallbacks(&start)
            .unwrap()
            .is_empty()
    );
    let p = prepare(input(Source::new("A\nA associate"), &[0, 13]));
    let word = p
        .word_views()
        .find(|word| word.query() == Some("associate"))
        .unwrap();
    assert!(
        word.ordinary_fallbacks(&p.boundary_views().next().unwrap().handle())
            .unwrap()
            .is_empty()
    );
    let after = p
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .boundary();
    assert_eq!(
        word.ordinary_fallbacks(&after)
            .unwrap()
            .iter()
            .map(|offer| offer.view().boundary().view().source_before().get())
            .collect::<Vec<_>>(),
        [4]
    );

    let mut source = Source::new("Aassociate");
    source.inline_box(crate::InlineBox::new(
        crate::Id::from_u64(444),
        crate::InlineBoxKind::InFlow,
        1,
        crate::Size::new(10.0, 10.0),
    ));
    let mut value = input(source, &[0, 1, 10]);
    value.anchor_order = vec![crate::ParagraphAnchor {
        item: crate::ParagraphAnchorItem::Atomic(0),
        affinity: crate::Affinity::Before,
    }];
    value.boundary_policies.extend([
        crate::ParagraphBoundaryPolicy {
            definition: crate::ParagraphBoundaryDefinition::AtomicBefore(0),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        },
        crate::ParagraphBoundaryPolicy {
            definition: crate::ParagraphBoundaryDefinition::AtomicAfter(0),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::None,
        },
    ]);
    let p = prepare(value);
    let word = p
        .word_views()
        .find(|word| word.query() == Some("associate"))
        .unwrap();
    let atomic = p.atomic_views().next().unwrap().handle();
    let before = p
        .opportunity_views()
        .find(|offer| {
            offer.kind() == PreparedBreakKind::Allowed
                && offer.boundary().view().participant_offset() == 1
        })
        .unwrap();
    assert_eq!(before.boundary().view().source_before().get(), 1);
    assert!(
        matches!(p.participants().nth(1),Some(PreparedParticipant::Atomic(ref actual)) if *actual==atomic)
    );
    // The earlier AtomicBefore offer shares the atom cursor with the word, but
    // its next influence is the atomic, never this word's first text occurrence.
    assert!(
        word.ordinary_fallbacks(&p.boundary_views().next().unwrap().handle())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn ordinary_word_fallbacks_preserve_transparent_mixed_cut_family() {
    let mut source = Source::new("A associate");
    source.inline_box(crate::InlineBox::new(
        crate::Id::from_u64(445),
        crate::InlineBoxKind::OutOfFlow,
        2,
        crate::Size::new(0.0, 0.0),
    ));
    let mut value = input(source, &[0, 2, 2, 11]);
    value.controls = vec![
        crate::ParagraphControl {
            at: 2,
            kind: crate::ParagraphControlKind::RightToLeftOverride,
        },
        crate::ParagraphControl {
            at: 2,
            kind: crate::ParagraphControlKind::PopDirectionalFormat,
        },
    ];
    value.anchor_order = [
        crate::ParagraphAnchorItem::EmptyOrigin(1),
        crate::ParagraphAnchorItem::Atomic(0),
        crate::ParagraphAnchorItem::Control(0),
        crate::ParagraphAnchorItem::Control(1),
    ]
    .into_iter()
    .map(|item| crate::ParagraphAnchor {
        item,
        affinity: crate::Affinity::After,
    })
    .collect();
    let p = prepare(value);
    let start = p.boundary_views().next().unwrap().handle();
    let word = p
        .word_views()
        .find(|word| word.query() == Some("associate"))
        .unwrap();
    let offers = word.ordinary_fallbacks(&start).unwrap();
    assert_eq!(offers.len(), 5);
    assert_eq!(
        offers
            .iter()
            .map(|offer| offer.view().boundary().view().source_before().get())
            .collect::<Vec<_>>(),
        [2; 5]
    );
    let positions = offers
        .iter()
        .map(|offer| offer.view().boundary().view().participant_offset())
        .collect::<Vec<_>>();
    assert!(positions.windows(2).all(|pair| pair[0] + 1 == pair[1]));
    for offer in offers {
        assert_eq!(offer.view().kind(), PreparedBreakKind::Allowed);
        let mut value = request(&p);
        value.end = offer.view().boundary();
        value.participants = p
            .participants()
            .take(value.end.view().participant_offset())
            .collect();
        value.termination = PreparedTermination::Ordinary(offer);
        value.alternative = None;
        value.context.end = LineEnd::Soft;
        value.atomics = value
            .participants
            .iter()
            .filter_map(|part| {
                if let PreparedParticipant::Atomic(occurrence) = part {
                    Some(AtomicAllocation {
                        occurrence: occurrence.clone(),
                        allocation: AtomicLineAllocation::OutOfFlow,
                    })
                } else {
                    None
                }
            })
            .collect();
        value.controls = value
            .participants
            .iter()
            .filter_map(|p| {
                if let PreparedParticipant::Control(control) = p {
                    Some({
                        let occurrence = control.clone();
                        ControlAllocation {
                            boundary: matches!(
                                occurrence.view().kind(),
                                crate::ParagraphControlKind::InlineBoundaryStart
                                    | crate::ParagraphControlKind::InlineBoundaryEnd
                            )
                            .then_some(crate::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                            occurrence,
                            strut: None,
                        }
                    })
                } else {
                    None
                }
            })
            .collect();
        let line = p
            .select(value)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(line.feedback().advance, 20.0);
        assert_eq!(
            line.selected_line().request().participants.len(),
            line.selected_line()
                .request()
                .end
                .view()
                .participant_offset()
        );
    }
}

#[test]
fn before_auto_uses_the_actual_lri_occurrence_in_an_rtl_paragraph() {
    let mut value = input(Source::new("associate"), &[0, 9]);
    value.base_direction = ParagraphBaseDirection::RightToLeft;
    value.base_style.hyphenate_character = HyphenateCharacter::String(Text::from("\u{5d0}A"));
    value.controls = vec![crate::ParagraphControl {
        at: 4,
        kind: crate::ParagraphControlKind::LeftToRightIsolate,
    }];
    value.anchor_order = vec![crate::ParagraphAnchor {
        item: crate::ParagraphAnchorItem::Control(0),
        affinity: crate::Affinity::After,
    }];
    let mut cmaps: Vec<_> = (b'a'..=b'z').map(|cp| (u32::from(cp), 35)).collect();
    cmaps.extend([(65, 35), (0x5d0, 36)]);
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(&[98, 99], &cmaps);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    );
    let p = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(p.inner.analyzed.projection.atoms[3].code_point, 111);
    assert_eq!(p.inner.analyzed.analysis.levels[3], 2);
    assert_eq!(p.inner.analyzed.analysis.control_levels, [1]);
    let cuts: Vec<_> = p
        .hyphenation_cut_views()
        .filter(|v| v.boundary().view().source_before().get() == 4)
        .map(|v| v.handle())
        .collect();
    assert_eq!(cuts.len(), 2);
    for (cut, after_control) in cuts.into_iter().zip([false, true]) {
        assert_eq!(
            cut.view().counts(),
            Some(crate::HyphenationCounts {
                total: 9,
                before: 4,
                after: 5
            })
        );
        let offer = cut.view().opportunity().unwrap();
        let mut r = request(&p);
        r.end = offer.view().boundary();
        r.termination = PreparedTermination::Ordinary(offer.clone());
        r.alternative = Some(offer.view().alternatives().next().unwrap().handle());
        r.participants = p
            .participants()
            .take(r.end.view().participant_offset())
            .collect();
        r.controls = r
            .participants
            .iter()
            .filter_map(|v| {
                if let PreparedParticipant::Control(c) = v {
                    Some({
                        let occurrence = c.clone();
                        ControlAllocation {
                            boundary: matches!(
                                occurrence.view().kind(),
                                crate::ParagraphControlKind::InlineBoundaryStart
                                    | crate::ParagraphControlKind::InlineBoundaryEnd
                            )
                            .then_some(crate::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                            occurrence,
                            strut: None,
                        }
                    })
                } else {
                    None
                }
            })
            .collect();
        r.context.base_direction = ParagraphBaseDirection::RightToLeft;
        r.context.line_box_direction = crate::LineBoxDirection::RightToLeft;
        r.context.end = LineEnd::Soft;
        let line = p
            .select(r)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(line.feedback().advance, 60.0);
        let presentation = &line.inner.selected.presentation;
        let domain = &presentation.domains[0];
        assert_eq!(
            (domain.outer, domain.base),
            if after_control { (1, 1) } else { (2, 0) }
        );
        assert_eq!(
            presentation.units[domain.units.clone()]
                .iter()
                .map(|u| u.level)
                .collect::<Vec<_>>(),
            if after_control {
                vec![1, 2]
            } else {
                vec![1, 0]
            }
        );
        let super::super::prepared::presentation::Attachment::Before { participant, atom } =
            domain.attachment
        else {
            panic!("Before")
        };
        assert_eq!(atom.is_none(), after_control);
        assert_eq!(
            matches!(
                p.inner.tables.participants[participant],
                super::super::prepared::ParticipantData::Control(_)
            ),
            after_control
        );
        // Local Hebrew+Latin has levels1,0 at base0 (Hebrew,Latin),
        // versus1,2 at base1 (Latin,Hebrew). Outer level is separate.
        let ids: Vec<_> = line
            .run_views()
            .flat_map(|r| {
                r.glyphs()
                    .filter(|g| g.range() == Range::new(4, 4))
                    .map(|g| g.id())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(
            ids,
            if after_control {
                vec![35, 36]
            } else {
                vec![36, 35]
            }
        );
        assert!(line.run_views().all(|r| r.face().bytes() == bytes));
        assert_eq!(
            p.inner.analyzed.processed.source().text().code_units(),
            Source::new("associate").text().code_units()
        );
    }
}
