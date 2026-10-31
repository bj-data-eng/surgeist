//! New whole-joint structural-section custody qualification. A later
//! participant-free source deletion cannot migrate into an earlier segment.
//! The uncaptured removal case characterizes existing incident/copy semantics.

use crate::processing::DeletionCause;
use crate::*;
use std::sync::Arc;

fn captured_parent() -> Arc<ProcessedSource> {
    let mut input = super::input("AB ", WhiteSpace::Preserve);
    input.source = Source::identified("AB ", Id::from_u64(78478), 19);
    let mut discarded = super::style(WhiteSpace::Discard, 10.0);
    discarded.text_combine_upright = TextCombineUpright::None;
    input.source.span(Range::new(2, 3), discarded);
    let SourceProcessingScope::Block { topology, .. } = &mut input.scopes[0] else {
        unreachable!("actual parent Block")
    };
    *topology = SourceProcessingTopology::Complete;
    input.scopes.push(SourceProcessingScope::Block {
        topology: SourceProcessingTopology::Complete,
        trim: WhiteSpaceTrim::default(),
        base_style: super::style(WhiteSpace::Preserve, 10.0),
        used_direction: ParagraphBaseDirection::LeftToRight,
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
    });
    input.origins = [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    input.paragraphs = vec![
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        };
        3
    ];
    input.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 2,
        kind: ParagraphConnectionKind::Disconnected,
    }];
    input.boundary_policies.clear();
    input.joint_flows = vec![SourceProcessingJointFlow {
        scope: 0,
        paragraphs: vec![0, 2],
        facts_owner: 0,
    }];
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::EnterParagraph(1),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::EnterParagraph(2),
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::Origin {
            origin: 2,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(2),
        SourceProcessingEvent::ExitScope(0),
    ];
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.source().text(), &Text::from("AB "));
    assert_eq!(processed.processed_text(), &Text::from("AB"));
    let raw = processed.paragraphs().collect::<Vec<_>>();
    for (i, (extent, text)) in [
        (Range::new(0, 1), "A"),
        (Range::new(1, 2), "B"),
        (Range::new(2, 3), ""),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            raw[i].captured_paragraph(),
            processed.captured_paragraph(i).unwrap()
        );
        assert_eq!(raw[i].source_pieces().collect::<Vec<_>>(), [extent]);
        assert_eq!(
            super::text(raw[i].plaintext_items(Affinity::After)),
            text.encode_utf16().collect::<Vec<_>>()
        );
    }
    use crate::processing::lexical_inventory::{
        LexicalDisposition, LexicalRecordKind, LexicalState,
    };
    let LexicalState::Executed { records, .. } = &processed.lexical else {
        panic!("actual executed source capture")
    };
    assert!(records.iter().any(
        |record| matches!(record.kind, LexicalRecordKind::Character {
        source, mode: WhiteSpace::Discard,
        disposition: LexicalDisposition::Deleted { processed: 2, cause: DeletionCause::Discard }, ..
    } if source == Range::new(2, 3))
    ));
    let link = processed
        .lexical
        .deletion(Range::new(2, 3), DeletionCause::Discard);
    assert_eq!(link.occurrences(&processed.lexical).count(), 1);
    assert_eq!(link.original_range(&processed.lexical), Range::new(2, 3));
    processed
}

fn finalized(prepared: &PreparedText, segment: &PreparedSourceSegment) -> FinalizedLine {
    let start = segment.start();
    let end = segment.end();
    let first = start.view().participant_offset();
    let last = end.view().participant_offset();
    let offer = segment.end_opportunity();
    assert_eq!(offer.view().boundary(), end);
    prepared
        .select(LineSelectionRequest {
            segment: segment.clone(),
            start,
            end,
            participants: prepared
                .participants()
                .skip(first)
                .take(last - first)
                .collect(),
            termination: PreparedTermination::Ordinary(offer),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: LineContext {
                alignment_insets: LineAlignmentInsets::ZERO,
                formatted_line: FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: LineBoxDirection::LeftToRight,
                orientation: LineOrientation::Horizontal,
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: Some(100.0),
                indent: 0.0,
                alignment: Alignment::Left,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap()
}

fn mappings(line: &FinalizedLine) -> Vec<(PresentationMappingKind, Range)> {
    line.mapping_views()
        .filter(|piece| !piece.source_range().is_empty())
        .map(|piece| (piece.kind(), piece.source_range()))
        .collect()
}

/// An intentional owning semantic observation: these are the actual base
/// relation events retained by this finalized request, not guessed event cuts.
fn phase_i_deletions(line: &FinalizedLine) -> Vec<(Range, usize)> {
    use crate::protocol::prepared::SelectedRelationEvent;
    use crate::transform::relation::RelationEvent;
    let selected = line.selected_line();
    let prepared = line.prepared_text();
    let map = &prepared.inner.analyzed.transform;
    selected
        .inner
        .presentation
        .selected_relation
        .events
        .iter()
        .filter_map(|entry| {
            let SelectedRelationEvent::Base { event } = *entry else {
                return None;
            };
            match map.events()[event] {
                RelationEvent::PhaseIDeleted {
                    source,
                    processed,
                    lexical,
                } => {
                    assert_eq!(
                        lexical.original_range(&prepared.inner.analyzed.processed.lexical),
                        source
                    );
                    Some((source, processed))
                }
                _ => None,
            }
        })
        .collect()
}

/// Copy projection over the actual public original-mapping pieces. This fixture
/// has neither alternatives nor repeated expansion pieces, so source/deletion
/// intervals concatenate once in receipt order. It does not manufacture a caret
/// for the nonformatted completion or claim a public placed-cursor copy of it.
fn mapping_copy(line: &FinalizedLine) -> Vec<u16> {
    let prepared = line.prepared_text();
    let analyzed = prepared.analyzed_source();
    let original = analyzed.source().text().code_units();
    mappings(line)
        .into_iter()
        .flat_map(|(kind, range)| {
            assert!(matches!(
                kind,
                PresentationMappingKind::Source | PresentationMappingKind::Deleted
            ));
            original[range.start..range.end].to_vec()
        })
        .collect()
}

fn formatted_entry(line: FinalizedLine) -> PlacementEntry {
    PlacementEntry::Formatted(LinePlacement {
        line,
        anchor: LineAnchor {
            inline_origin: 0.0,
            baseline: 8.0,
        },
        orientation: LineOrientation::Horizontal,
        atomics: vec![],
    })
}

#[test]
fn a_later_discarded_member_belongs_only_to_its_empty_structural_completion() {
    let processed = captured_parent();
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = domain.outputs().next().unwrap();
    assert_eq!(analyzed.inner.projection.text, "A");
    assert_eq!(
        analyzed
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        [
            processed.captured_paragraph(0).unwrap(),
            processed.captured_paragraph(2).unwrap()
        ]
    );
    assert_eq!(
        analyzed
            .inner
            .paragraph
            .origins
            .iter()
            .map(|row| row.aggregate)
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(2, 3)]
    );
    let analyzed = crate::test_wrapping::bind_missing(analyzed, Wrap::Word);
    let prepared = super::system()
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let segments = prepared.source_segments().collect::<Vec<_>>();
    assert_eq!(segments.len(), 2);
    assert_ne!(segments[0], segments[1]);
    assert_eq!(segments[0].end().view().participant_offset(), 1);
    assert_eq!(
        segments[0].end().view().participant_offset(),
        segments[1].start().view().participant_offset()
    );
    // Phase-I Discard is a real source-owned logical participant, even
    // though this structural completion contributes no painted/inflow atom.
    assert_eq!(
        (
            segments[1].start().view().participant_offset(),
            segments[1].end().view().participant_offset()
        ),
        (1, 2)
    );
    let deleted_participant = prepared.participants().nth(1).unwrap();
    let PreparedParticipant::Text(deleted_unit) = &deleted_participant else {
        panic!("the actual Phase-I deletion retains its original-owned text unit")
    };
    let deleted_view = deleted_unit.view();
    assert_eq!(deleted_view.source_range(), Range::new(2, 3));
    assert_eq!(
        (
            deleted_view.processed_range().start().get(),
            deleted_view.processed_range().end().get()
        ),
        (2, 2)
    );
    let first = finalized(&prepared, &segments[0]);
    let last = finalized(&prepared, &segments[1]);
    assert!(first.has_in_flow_content());
    assert!(!last.has_in_flow_content());
    assert_eq!(first.selected_line().request().segment, segments[0]);
    assert_eq!(last.selected_line().request().segment, segments[1]);
    assert_eq!(
        last.selected_line().request().participants.as_slice(),
        &[deleted_participant]
    );
    assert_eq!(
        mappings(&first),
        [(PresentationMappingKind::Source, Range::new(0, 1))]
    );
    assert_eq!(
        mappings(&last),
        [(PresentationMappingKind::Deleted, Range::new(2, 3))]
    );
    assert!(phase_i_deletions(&first).is_empty());
    assert_eq!(phase_i_deletions(&last), [(Range::new(2, 3), 2)]);
    assert_eq!(mapping_copy(&first), [0x41]);
    assert_eq!(mapping_copy(&last), [0x20]);
    // Public partial placement/copy must exclude the actual later source owner.
    let partial = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::Range {
            start: segments[0].start(),
            end: segments[0].end(),
        },
        entries: vec![formatted_entry(first.clone())],
    })
    .unwrap();
    let positions = partial.positions().collect::<Vec<_>>();
    let copy = partial
        .position_selection(positions.first().unwrap(), positions.last().unwrap())
        .unwrap();
    assert_eq!(copy.source_text().code_units(), [0x41]);
    assert_eq!(copy.source_pieces().len(), 1);
    assert_eq!(copy.source_pieces()[0].aggregate_range(), Range::new(0, 1));
    let origins = prepared
        .origin_views()
        .map(|origin| origin.handle())
        .collect::<Vec<_>>();
    assert_eq!(origins[0].view().aggregate_range(), Range::new(0, 1));
    assert_eq!(origins[1].view().aggregate_range(), Range::new(2, 3));
    let error = partial
        .source_range_fragments(&origins[1], Range::new(2, 3))
        .unwrap_err();
    assert!(
        matches!(error.detail(), Some(ErrorDetail::SourceRangeFragments {
        reason: SourceRangeFragmentError::OutsidePlacement { aggregate },
    }) if *aggregate == Range::new(2, 3))
    );
    let completion = last.nonformatted().unwrap();
    let whole = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![
            formatted_entry(first.clone()),
            PlacementEntry::NonFormatted(NonFormattedLinePlacement {
                line: completion,
                origin: Point::new(0.0, 10.0),
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            }),
        ],
    })
    .unwrap();
    assert_eq!(whole.source(), processed.source());
    assert_eq!(whole.source().text(), &Text::from("AB "));
    let retained = whole.nonformatted_lines().collect::<Vec<_>>();
    assert_eq!(retained.len(), 1);
    let actual_last = retained[0].line.finalized_line();
    assert_eq!(actual_last.selected_line().request().segment, segments[1]);
    assert_eq!(phase_i_deletions(actual_last), [(Range::new(2, 3), 2)]);
    let original_copy = mapping_copy(&first)
        .into_iter()
        .chain(mapping_copy(actual_last))
        .collect::<Vec<_>>();
    assert_eq!(
        original_copy,
        [0x41, 0x20],
        "actual receipt custody excludes child B and owns the removed SPACE once"
    );
    let removed = whole
        .source_range_fragments(&origins[1], Range::new(2, 3))
        .unwrap();
    assert_eq!(removed.origin(), &origins[1]);
    assert_eq!(removed.local_range(), Range::new(2, 3));
    assert_eq!(removed.units().len(), 0);
    assert_eq!(mapping_copy(&first), [0x41]);
    assert_eq!(processed.processed_text(), &Text::from("AB"));
}

#[test]
fn uncaptured_lithuanian_trailing_removal_keeps_incident_endpoint_and_complete_original_copy() {
    let source = Source::identified("i\u{307}", Id::from_u64(78479), 20);
    let mut style = super::style(WhiteSpace::Preserve, 10.0);
    style.locale = Some("lt".to_owned());
    style.text_transform =
        TextTransform::Set(TextTransformSet::none().with_case(CaseTransform::Uppercase));
    let analyzed = AnalyzedSource::new(
        source.clone(),
        style,
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.inner.projection.text, "I");
    let prepared = super::system()
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert!(prepared.captured_owner().is_none());
    let segment = prepared.source_segments().next().unwrap();
    let end = segment.end();
    assert_eq!(
        (
            end.view().source_before().get(),
            end.view().source_after().get(),
            end.view().processed_position().get()
        ),
        (1, 1, 2)
    );
    let line = finalized(&prepared, &segment);
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![formatted_entry(line)],
    })
    .unwrap();
    let positions = layout.positions().collect::<Vec<_>>();
    let copy = layout
        .position_selection(positions.first().unwrap(), positions.last().unwrap())
        .unwrap();
    assert_eq!(copy.source_text().code_units(), [0x69, 0x307]);
    assert_eq!(copy.source_pieces().len(), 1);
    assert_eq!(copy.source_pieces()[0].aggregate_range(), Range::new(0, 2));
    assert_eq!(copy.source_pieces()[0].local_range(), Range::new(0, 2));
    assert_eq!(layout.source(), &source);
}
