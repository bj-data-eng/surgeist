//! Existing-callable conditional-case context witnesses. Pinned CSS Text 3
//! full Unicode case mapping uses Final_Sigma context; source continuity and
//! joint-output authority remain distinct under the adopted issue-784 contract.

use crate::*;
use std::sync::Arc;

fn request(
    transform: TextTransform,
    relationship: Option<ParagraphConnectionKind>,
) -> ContextualSourceInput {
    let mut input = super::input("ΟΣΑ", WhiteSpace::Preserve);
    input.source = Source::identified("ΟΣΑ", Id::from_u64(78450), 9);
    input.origins = [Range::new(0, 2), Range::new(2, 3)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    let SourceProcessingScope::Block { base_style, .. } = &mut input.scopes[0] else {
        unreachable!("actual owning Block fixture")
    };
    base_style.text_transform = transform;
    input.paragraphs = vec![
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        },
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::RightToLeft,
            paragraph_strut: ParagraphStrut::AllocatedContributions,
        },
    ];
    input.joint_flows.clear();
    input.connections.clear();
    input.boundary_policies.clear();
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
    ];
    if let Some(kind) = relationship {
        input.connections.push(SourceProcessingConnection {
            before_origin: 0,
            after_origin: 1,
            kind,
        });
        input.events.push(SourceProcessingEvent::Connection(0));
        if kind == ParagraphConnectionKind::Continuous {
            input
                .boundary_policies
                .push(super::policy(ParagraphBoundaryDefinition::Connection(0)));
        }
    }
    input.events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::EnterParagraph(1),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::ExitScope(0),
    ]);
    input
}

fn lowercase() -> TextTransform {
    TextTransform::Set(TextTransformSet::none().with_case(CaseTransform::Lowercase))
}

fn captured(input: ContextualSourceInput) -> Arc<ProcessedSource> {
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from("ΟΣΑ"));
    assert!(processed.captured_joint_flow(0).is_none());
    let raw = processed.paragraphs().collect::<Vec<_>>();
    assert_eq!(raw.len(), 2);
    assert_eq!(
        raw[0].source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 2)]
    );
    assert_eq!(
        raw[1].source_pieces().collect::<Vec<_>>(),
        [Range::new(2, 3)]
    );
    assert_eq!(
        super::text(raw[0].plaintext_items(Affinity::After)),
        [0x39f, 0x3a3]
    );
    assert_eq!(
        super::text(raw[1].plaintext_items(Affinity::After)),
        [0x391]
    );
    assert_eq!(
        processed.captured_paragraph(0).unwrap().base_direction(),
        ParagraphBaseDirection::LeftToRight
    );
    assert_eq!(
        processed.captured_paragraph(1).unwrap().base_direction(),
        ParagraphBaseDirection::RightToLeft
    );
    assert_eq!(
        processed.captured_paragraph(0).unwrap().paragraph_strut(),
        ParagraphStrut::DefaultFont
    );
    assert_eq!(
        processed.captured_paragraph(1).unwrap().paragraph_strut(),
        ParagraphStrut::AllocatedContributions
    );
    processed
}

fn assert_exact_view(
    analyzed: &AnalyzedSource,
    processed: &Arc<ProcessedSource>,
    ordinal: usize,
    expected: &str,
    code_points: &[u32],
) {
    let (extent, direction, strut) = if ordinal == 0 {
        (
            Range::new(0, 2),
            ParagraphBaseDirection::LeftToRight,
            ParagraphStrut::DefaultFont,
        )
    } else {
        (
            Range::new(2, 3),
            ParagraphBaseDirection::RightToLeft,
            ParagraphStrut::AllocatedContributions,
        )
    };
    assert_eq!(analyzed.source(), processed.source());
    assert_eq!(
        analyzed.processed_source().processed_text(),
        &Text::from("ΟΣΑ")
    );
    assert_eq!(analyzed.inner.projection.text, expected);
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        code_points
    );
    assert_eq!(analyzed.inner.paragraph.base_direction, Some(direction));
    assert_eq!(analyzed.inner.paragraph.paragraph_strut, strut);
    assert_eq!(analyzed.inner.paragraph.origins.len(), 1);
    assert_eq!(analyzed.inner.paragraph.origins[0].aggregate, extent);
    assert_eq!(analyzed.inner.paragraph.origins[0].local, extent);
    assert_eq!(
        analyzed.inner.paragraph.origins[0].original(analyzed.processed_source()),
        processed.source()
    );
    let key = analyzed
        .inner
        .paragraph
        .origin_map
        .flow_origin(analyzed.inner.paragraph.origin_map.view(0).unwrap());
    assert_eq!(
        key.captured_event(),
        Some(processed.captured_origin(ordinal).unwrap().index)
    );
    assert_eq!(analyzed.inner.transform.rows.len(), code_points.len());
    for (index, atom) in analyzed.inner.projection.atoms.iter().enumerate() {
        let original = Range::new(extent.start + index, extent.start + index + 1);
        assert_eq!(atom.source, original);
        assert_eq!(atom.processed, original);
        assert_eq!(analyzed.inner.transform.original_pieces(index).len(), 1);
        let piece = analyzed.inner.transform.original_pieces(index)[0];
        assert_eq!(piece.origin, key);
        assert_eq!(piece.aggregate, original);
        assert_eq!(piece.local, original);
    }
    let raw = processed.paragraphs().nth(ordinal).unwrap();
    assert_eq!(raw.source_range(), extent);
    assert!(Arc::ptr_eq(raw.processed_source(), processed));
}

#[test]
fn continuous_source_context_changes_sigma_without_expanding_issued_view_coverage() {
    // Unicode Final_Sigma: Omicron before Sigma is cased, and the actual
    // connected following Alpha is also cased. Sigma therefore maps to 03C3,
    // although the following Alpha belongs to another independent raw view.
    let processed = captured(request(
        lowercase(),
        Some(ParagraphConnectionKind::Continuous),
    ));
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let first = AnalyzedSource::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_exact_view(&first, &processed, 0, "οσ", &[0x3bf, 0x3c3]);
    let second = AnalyzedSource::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_exact_view(&second, &processed, 1, "α", &[0x3b1]);
    assert_eq!(processed.paragraphs().count(), 2);
    assert!(
        processed
            .captured_paragraph(0)
            .unwrap()
            .joint_flow()
            .is_none()
    );
    assert!(
        processed
            .captured_paragraph(1)
            .unwrap()
            .joint_flow()
            .is_none()
    );
}

#[test]
fn a_real_disconnection_keeps_final_sigma_and_independent_following_alpha() {
    let processed = captured(request(
        lowercase(),
        Some(ParagraphConnectionKind::Disconnected),
    ));
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let first = AnalyzedSource::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_exact_view(&first, &processed, 0, "ος", &[0x3bf, 0x3c2]);
    let second = AnalyzedSource::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_exact_view(&second, &processed, 1, "α", &[0x3b1]);
}

#[test]
fn material_unknown_sigma_context_refuses_without_rebinding_published_source_views() {
    let processed = captured(request(lowercase(), None));
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let error = AnalyzedSource::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    // The unavailable context is the actual missing source relationship;
    // endpoints identify input origins, not view-local rows or source offsets.
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::MissingSourceContinuity {
                before_origin: 0,
                after_origin: 1,
                ..
            },
        })
    ));
    assert!(processed.captured_connection(0).is_none());
    assert_eq!(processed.processed_text(), &Text::from("ΟΣΑ"));
    assert_eq!(
        raw[0].source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 2)]
    );
    assert_eq!(
        raw[1].source_pieces().collect::<Vec<_>>(),
        [Range::new(2, 3)]
    );
    assert_eq!(
        super::text(raw[0].plaintext_items(Affinity::After)),
        [0x39f, 0x3a3]
    );
    assert!(Arc::ptr_eq(raw[0].processed_source(), &processed));
}

#[test]
fn unknown_source_context_does_not_demand_unrequested_case_mapping() {
    let processed = captured(request(TextTransform::none(), None));
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let first = AnalyzedSource::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_exact_view(&first, &processed, 0, "ΟΣ", &[0x39f, 0x3a3]);
    let second = AnalyzedSource::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_exact_view(&second, &processed, 1, "Α", &[0x391]);
    assert!(processed.captured_connection(0).is_none());
}
