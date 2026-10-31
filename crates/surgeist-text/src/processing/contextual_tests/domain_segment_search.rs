//! Segment-owned conditional-query admission. Empty source sections retain
//! identity at equal numerical cuts; the ordinary no-progress policy is kept.

use crate::*;
use std::sync::Arc;

fn parent_output(nonempty: bool) -> (Arc<ProcessedSource>, PreparedText) {
    let text = if nonempty { "AC" } else { "" };
    let mut input = super::input(text, WhiteSpace::Preserve);
    let SourceProcessingScope::Block {
        topology,
        base_style,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!("actual parent Block")
    };
    *topology = SourceProcessingTopology::Complete;
    base_style.text_combine_upright = TextCombineUpright::None;
    base_style.text_transform = TextTransform::none();
    input.scopes.push(SourceProcessingScope::Block {
        topology: SourceProcessingTopology::Complete,
        trim: WhiteSpaceTrim::default(),
        base_style: super::style(WhiteSpace::Preserve, 10.0),
        used_direction: ParagraphBaseDirection::LeftToRight,
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
    });
    input.paragraphs = vec![
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        };
        3
    ];
    input.joint_flows = vec![SourceProcessingJointFlow {
        scope: 0,
        paragraphs: vec![0, 2],
        facts_owner: 0,
    }];
    input.origins.clear();
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
    ];
    if nonempty {
        input.origins = [Range::new(0, 1), Range::new(1, 2)]
            .into_iter()
            .map(|range| super::origin(&input.source, range))
            .collect();
        input.events.push(SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        });
        input.connections = vec![SourceProcessingConnection {
            before_origin: 0,
            after_origin: 1,
            kind: ParagraphConnectionKind::Disconnected,
        }];
    }
    input.events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::EnterParagraph(1),
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::EnterParagraph(2),
    ]);
    if nonempty {
        input.events.extend([
            SourceProcessingEvent::Connection(0),
            SourceProcessingEvent::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
        ]);
    }
    input.events.extend([
        SourceProcessingEvent::ExitParagraph(2),
        SourceProcessingEvent::ExitScope(0),
    ]);
    assert!(input.controls.is_empty());
    assert!(input.source.boxes().is_empty());
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(text));
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        text.encode_utf16().collect::<Vec<_>>()
    );
    let raw = processed.paragraphs().collect::<Vec<_>>();
    assert_eq!(raw.len(), 3);
    for (index, member) in raw.iter().enumerate() {
        assert_eq!(
            member.captured_paragraph(),
            processed.captured_paragraph(index).unwrap()
        );
    }
    assert!(raw[1].source_pieces().next().is_none());
    if nonempty {
        assert_eq!(
            raw[0].source_pieces().collect::<Vec<_>>(),
            [Range::new(0, 1)]
        );
        assert_eq!(
            raw[2].source_pieces().collect::<Vec<_>>(),
            [Range::new(1, 2)]
        );
    } else {
        assert!(
            raw.iter()
                .all(|member| member.source_pieces().next().is_none())
        );
        assert!(processed.captured_origin(0).is_none());
        assert!(processed.captured_connection(0).is_none());
    }
    let domain = AnalyzedDomain::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = domain.outputs().next().unwrap();
    assert_eq!(analyzed.inner.projection.text, text);
    assert_eq!(analyzed.combined_cells().len(), 0);
    assert_eq!(analyzed.source(), &source);
    let mut system = super::system(); // Actual registered Ahem supplies the default-font receipt.
    let prepared = system
        .prepare(analyzed, PreparationOptions::default())
        .unwrap();
    let owner = prepared.captured_owner().unwrap();
    assert_eq!(owner.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [
            processed.captured_paragraph(0).unwrap(),
            processed.captured_paragraph(2).unwrap()
        ]
    );
    assert_eq!(
        owner.facts_owner(),
        processed.captured_paragraph(0).unwrap()
    );
    assert_eq!(owner.joint_flow(), processed.captured_joint_flow(0));
    assert_eq!(prepared.source_segments().len(), 2);
    (processed, prepared)
}

fn reason(error: &Error) -> &ProtocolError {
    assert_eq!(error.code, ErrorCode::InvalidTextProtocol);
    let Some(ErrorDetail::InvalidTextProtocol { reason }) = error.detail() else {
        panic!("the actual section query must retain its precise protocol refusal")
    };
    reason
}

fn coordinates(boundary: &PreparedBoundary) -> (usize, usize, usize, usize) {
    let view = boundary.view();
    (
        view.participant_offset(),
        view.source_before().get(),
        view.source_after().get(),
        view.processed_position().get(),
    )
}

#[test]
fn empty_structural_sections_reject_cross_owned_ceilings_even_at_identical_numeric_cuts() {
    let (processed, prepared) = parent_output(false);
    let segments = prepared.source_segments().collect::<Vec<_>>();
    assert_ne!(segments[0], segments[1]);
    assert_ne!(segments[0].end_opportunity(), segments[1].end_opportunity());
    for segment in &segments {
        assert_eq!(segment.prepared_text(), prepared);
        for boundary in [segment.start(), segment.end()] {
            assert_eq!(coordinates(&boundary), (0, 0, 0, 0));
        }
        let entry = segment.entry_position().unwrap();
        let exit = segment.exit_position().unwrap();
        assert!(entry.same_capture(&processed.captured_scope(0).unwrap().enter_position()));
        assert!(entry < exit);
    }
    assert_ne!(segments[0].entry_position(), segments[1].entry_position());
    assert_ne!(segments[0].exit_position(), segments[1].exit_position());
    for index in 0..2 {
        let segment = &segments[index];
        let start = segment.start();
        let ceiling = segment.end_opportunity();
        // Conditional search requires genuine progress. Empty sections do not
        // manufacture a successful query merely to expose its identity.
        let error = segment
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::MinContent,
                &[],
                None,
            )
            .unwrap_err();
        assert_eq!(
            reason(&error),
            &ProtocolError::InvalidConditionalQuery {
                start: start.index,
                ceiling: ceiling.index
            }
        );
        let other = &segments[1 - index];
        let error = segment
            .conditional_search(
                &start,
                None,
                &other.end_opportunity(),
                ConditionalPurpose::MinContent,
                &[],
                None,
            )
            .unwrap_err();
        assert_eq!(
            reason(&error),
            &ProtocolError::MismatchedSourceSegment {
                expected: segment.index,
                actual: other.index
            }
        );
    }
    assert_eq!(processed.source().text(), &Text::from(""));
    assert_eq!(prepared.analyzed_source().source(), processed.source());
}

#[test]
fn successful_conditional_searches_retain_their_genuine_structural_segment_identity() {
    let (processed, prepared) = parent_output(true);
    let segments = prepared.source_segments().collect::<Vec<_>>();
    let mut searches = Vec::new();
    for (index, segment) in segments.iter().enumerate() {
        assert_eq!(segment.prepared_text(), prepared);
        let start = segment.start();
        let end = segment.end();
        assert_eq!(coordinates(&start), (index, index, index, index));
        assert_eq!(
            coordinates(&end),
            (index + 1, index + 1, index + 1, index + 1)
        );
        let ceiling = segment.end_opportunity();
        let query = segment
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::MinContent,
                &[],
                None,
            )
            .unwrap();
        assert_eq!(query.source_segment(), *segment);
        assert_eq!(
            query,
            segment
                .conditional_search(
                    &start,
                    None,
                    &ceiling,
                    ConditionalPurpose::MinContent,
                    &[],
                    None
                )
                .unwrap()
        );
        let other = &segments[1 - index];
        let error = segment
            .conditional_search(
                &start,
                None,
                &other.end_opportunity(),
                ConditionalPurpose::MinContent,
                &[],
                None,
            )
            .unwrap_err();
        assert_eq!(
            reason(&error),
            &ProtocolError::MismatchedSourceSegment {
                expected: segment.index,
                actual: other.index
            }
        );
        searches.push(query);
    }
    assert_ne!(searches[0], searches[1]);
    assert_eq!(searches[0].source_segment(), segments[0]);
    assert_eq!(searches[1].source_segment(), segments[1]);
    assert_eq!(prepared.analyzed_source().inner.projection.text, "AC");
    assert_eq!(processed.source().text(), &Text::from("AC"));
}
