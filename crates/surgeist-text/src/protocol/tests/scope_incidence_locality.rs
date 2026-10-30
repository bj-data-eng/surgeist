//! Original enclosing edges are not clipped to paragraph sentinels.

use super::*;
use crate::{CapturedScopeIncidence as R, SourceProcessingEvent as E};

#[test]
fn enclosing_edges_localize_to_their_paragraph_and_remote_same_domain_is_outside() {
    let source = Source::new("AB");
    let processed = crate::process_contextual_source(crate::ContextualSourceInput {
        scope_alignment_limits: None,
        source: source.clone(),
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        envelope: crate::SourceProcessingEnvelope::Complete,
        scopes: vec![
            crate::SourceProcessingScope::Block {
                font_reference: None,
                topology: crate::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style(),
                used_direction: ParagraphBaseDirection::LeftToRight,
            },
            crate::SourceProcessingScope::Inline {
                alignment_boundary: None,
                font_reference: None,
                trim: Default::default(),
                style: None,
                boundaries: None,
            },
        ],
        paragraphs: (0..2)
            .map(|_| crate::SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
            })
            .collect(),
        origins: (0..2)
            .map(|i| crate::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: crate::Range::new(i, i + 1),
                aggregate: crate::Range::new(i, i + 1),
                kind: crate::ParagraphOriginKind::Authored,
            })
            .collect(),
        controls: vec![],
        connections: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        computed_line_boundaries: vec![],
        events: vec![
            E::EnterScope(0),
            E::EnterScope(1),
            E::EnterParagraph(0),
            E::Origin {
                origin: 0,
                affinity: crate::Affinity::After,
            },
            E::ExitParagraph(0),
            E::ExitScope(1),
            E::EnterParagraph(1),
            E::Origin {
                origin: 1,
                affinity: crate::Affinity::After,
            },
            E::ExitParagraph(1),
            E::ExitScope(0),
        ],
    })
    .unwrap();
    let processed = std::sync::Arc::new(processed);
    let scope = processed.captured_scope(1).unwrap();
    let mut fonts = system();
    let paragraphs: Vec<_> = processed
        .paragraphs()
        .map(|p| {
            fonts
                .prepare(
                    &AnalyzedSource::from_processed_paragraph(
                        p,
                        crate::AnalysisTypography::HorizontalComposition,
                    )
                    .unwrap(),
                    PreparationOptions::default(),
                )
                .unwrap()
        })
        .collect();
    for (paragraph, first, expected) in [
        (&paragraphs[0], true, R::AtBefore),
        (&paragraphs[0], false, R::AtAfter),
        (&paragraphs[1], true, R::After),
    ] {
        let boundary = if first {
            paragraph.boundary_views().next().unwrap().handle()
        } else {
            paragraph.boundary_views().last().unwrap().handle()
        };
        let incidences = paragraph.boundary_incidences(&boundary).unwrap();
        assert_eq!(incidences.len(), 1);
        assert_eq!(
            paragraph
                .incidence_scope_relation(&incidences[0], &scope)
                .unwrap(),
            expected
        );
        assert_eq!(incidences[0].boundary(), boundary);
    }
}
