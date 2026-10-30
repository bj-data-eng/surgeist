//! Existing-callable material source continuity witness. Raw paragraph storage
//! markers provide neither continuity nor a disconnected relationship.

use crate::*;
use std::sync::Arc;

fn input(
    text: &str,
    mode: WhiteSpace,
    relationship: Option<ParagraphConnectionKind>,
) -> ContextualSourceInput {
    let mut input = super::input(text, mode);
    input.origins = [Range::new(0, 1), Range::new(1, 2)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    input.paragraphs = vec![
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        },
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
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

#[test]
fn unknown_source_relationship_refuses_material_cross_paragraph_space_collapse() {
    // Each raw paragraph owns one actual SPACE. Neither a source relationship
    // nor joint-analysis authority was supplied. Global Phase I must not guess
    // whether the second occurrence collapses against the first.
    let error = process_contextual_source(input("  ", WhiteSpace::Collapse, None)).unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    // Unavailable contextual source processing retains its typed source category.
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::SourceProcessing { .. })
    ));
}

#[test]
fn explicit_relationships_and_nonmaterial_unknown_context_keep_real_source_outputs() {
    for (text, mode, relationship, expected) in [
        (
            "  ",
            WhiteSpace::Collapse,
            Some(ParagraphConnectionKind::Continuous),
            " ",
        ),
        (
            "  ",
            WhiteSpace::Collapse,
            Some(ParagraphConnectionKind::Disconnected),
            "  ",
        ),
        ("AB", WhiteSpace::Collapse, None, "AB"),
        ("  ", WhiteSpace::Preserve, None, "  "),
        ("  ", WhiteSpace::Discard, None, ""),
    ] {
        let input = input(text, mode, relationship);
        let source = input.source.clone();
        let processed = Arc::new(process_contextual_source(input).unwrap());
        assert_eq!(processed.source(), &source);
        assert_eq!(processed.processed_text(), &Text::from(expected));
        assert!(processed.captured_joint_flow(0).is_none());
        let raw = processed.paragraphs().collect::<Vec<_>>();
        assert_eq!(raw.len(), 2);
        for (ordinal, range) in [Range::new(0, 1), Range::new(1, 2)].into_iter().enumerate() {
            assert_eq!(raw[ordinal].source_range(), range);
            assert_eq!(raw[ordinal].source_pieces().collect::<Vec<_>>(), [range]);
            assert!(Arc::ptr_eq(raw[ordinal].processed_source(), &processed));
            assert!(raw[ordinal].data().connections.is_empty());
            assert!(raw[ordinal].data().boundary_policies.is_empty());
            assert_eq!(
                processed.captured_paragraph(ordinal).unwrap().joint_flow(),
                None
            );
        }
    }
}
