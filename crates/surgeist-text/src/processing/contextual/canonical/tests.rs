use super::*;
use crate::{SourceProcessingEvent as E, *};

fn processed(forced: bool) -> Arc<ProcessedSource> {
    let source = Source::new("A");
    Arc::new(
        process_contextual_source(ContextualSourceInput {
            scope_alignment_limits: None,
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            scopes: vec![
                SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: Style::default(),
                    used_direction: ParagraphBaseDirection::LeftToRight,
                },
                SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: None,
                    boundaries: None,
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
            }],
            origins: vec![ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, 1),
                aggregate: Range::new(0, 1),
                kind: ParagraphOriginKind::Authored,
            }],
            controls: vec![],
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: if forced {
                vec![ComputedLineBoundaryInput::Scope {
                    scope: 1,
                    side: SourceBoxSide::Before,
                }]
            } else {
                vec![]
            },
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                E::Origin {
                    origin: 0,
                    affinity: Affinity::After,
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
fn neutral_capture_stays_lazy_and_concurrent_queries_share_one_fact_allocation() {
    let source = processed(false);
    let capture = source.captured_scope(1).unwrap().capture;
    assert!(capture.canonical.0.lock().unwrap().is_none());
    let barrier = Arc::new(std::sync::Barrier::new(4));
    let values = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let capture = Arc::clone(&capture);
                let barrier = Arc::clone(&barrier);
                scope.spawn(move || {
                    barrier.wait();
                    capture.canonical_flow().unwrap()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    for value in &values {
        assert!(Arc::ptr_eq(value, &values[0]));
        // Actual canonical Enter=2, Origin=3, Exit=4. No computed or
        // connection bookkeeping changes these genuine original positions.
        assert_eq!(value.scope_ranks(1), Some((2, 5)));
        assert_eq!(value.edges(0), [2, 5]);
    }
}

#[test]
fn deep_capture_clone_starts_fresh_and_shared_facts_do_not_retain_capture() {
    let source = processed(false);
    let capture = source.captured_scope(1).unwrap().capture;
    let original = capture.canonical_flow().unwrap();
    let copy = capture.as_ref().clone();
    assert!(copy.canonical.0.lock().unwrap().is_none());
    let rebuilt = copy.canonical_flow().unwrap();
    assert!(!Arc::ptr_eq(&original, &rebuilt));
    assert_eq!(original.scope_ranks(1), rebuilt.scope_ranks(1));
    let weak = Arc::downgrade(&capture);
    drop(source);
    drop(capture);
    assert!(weak.upgrade().is_none());
    assert_eq!(original.scope_ranks(1), Some((2, 5)));
}

#[test]
fn interrupted_publication_recovers_without_replacing_complete_facts() {
    let source = processed(false);
    let capture = source.captured_scope(1).unwrap().capture;
    let interrupted = std::panic::catch_unwind(|| {
        let _guard = capture.canonical.0.lock().unwrap();
        panic!("interrupted before canonical publication");
    });
    assert!(interrupted.is_err());
    let value = capture.canonical_flow().unwrap();
    let interrupted = std::panic::catch_unwind(|| {
        let _guard = capture
            .canonical
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        panic!("interrupted after complete canonical publication");
    });
    assert!(interrupted.is_err());
    assert!(Arc::ptr_eq(&value, &capture.canonical_flow().unwrap()));
}

#[test]
fn census_counts_computed_admission_facts_before_local_index_and_only_once() {
    let source = processed(true);
    let captured = source.captured_scope(1).unwrap();
    assert!(captured.capture.canonical.0.lock().unwrap().is_some());
    let analyzed = AnalyzedSource::from_processed_paragraph(
        source.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert!(analyzed.inner.scope_cuts.get().is_none());
    let mut census = crate::engine::analysis::scope::IndexPayloadCensus::default();
    census.analyzed(&analyzed.inner.core);
    let bytes = census.bytes();
    assert!(bytes >= std::mem::size_of::<CanonicalFlow>());
    captured.capture.canonical_payload(&mut census);
    assert_eq!(census.bytes(), bytes);
}
