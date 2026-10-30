//! Functional qualification of completed captured-domain output. These APIs
//! are new; expectations come from source/joint authority, Unicode Final_Sigma,
//! and the indivisibility of the completed i + U+0307 grapheme.

use crate::*;
use std::sync::Arc;

fn lowercase() -> TextTransform {
    TextTransform::Set(TextTransformSet::none().with_case(CaseTransform::Lowercase))
}

fn facts(direction: ParagraphBaseDirection, strut: ParagraphStrut) -> SourceProcessingParagraph {
    SourceProcessingParagraph {
        base_direction: direction,
        paragraph_strut: strut,
    }
}

/// Parts are real original occurrences. Raw IDs deliberately differ from event
/// order. An empty part remains an actual Origin event and raw paragraph.
fn request(
    parts: &[&str],
    raw_ids: &[usize],
    used: &[SourceProcessingParagraph],
    transform: TextTransform,
    relationship: Option<ParagraphConnectionKind>,
    joint_facts_owner: Option<usize>,
) -> ContextualSourceInput {
    assert_eq!(parts.len(), raw_ids.len());
    assert_eq!(parts.len(), used.len());
    let mut input = super::input(parts.concat(), WhiteSpace::Preserve);
    input.source = Source::identified(parts.concat(), Id::from_u64(78470), 12);
    let SourceProcessingScope::Block { base_style, .. } = &mut input.scopes[0] else {
        unreachable!("real owning Block fixture")
    };
    base_style.text_transform = transform;
    input.origins.clear();
    input.paragraphs = vec![used[0]; parts.len()];
    input.connections.clear();
    input.boundary_policies.clear();
    input.events = vec![SourceProcessingEvent::EnterScope(0)];
    let mut cursor = 0;
    for (ordinal, part) in parts.iter().enumerate() {
        let end = cursor + part.encode_utf16().count();
        input
            .origins
            .push(super::origin(&input.source, Range::new(cursor, end)));
        input.paragraphs[raw_ids[ordinal]] = used[ordinal];
        input
            .events
            .push(SourceProcessingEvent::EnterParagraph(raw_ids[ordinal]));
        input.events.push(SourceProcessingEvent::Origin {
            origin: ordinal,
            affinity: Affinity::After,
        });
        if ordinal + 1 < parts.len()
            && let Some(kind) = relationship
        {
            let connection = input.connections.len();
            input.connections.push(SourceProcessingConnection {
                before_origin: ordinal,
                after_origin: ordinal + 1,
                kind,
            });
            input
                .events
                .push(SourceProcessingEvent::Connection(connection));
            if kind == ParagraphConnectionKind::Continuous {
                input.boundary_policies.push(super::policy(
                    ParagraphBoundaryDefinition::Connection(connection),
                ));
            }
        }
        input
            .events
            .push(SourceProcessingEvent::ExitParagraph(raw_ids[ordinal]));
        cursor = end;
    }
    input.events.push(SourceProcessingEvent::ExitScope(0));
    input.joint_flows = joint_facts_owner
        .map(|facts_owner| SourceProcessingJointFlow {
            scope: 0,
            paragraphs: raw_ids.to_vec(),
            facts_owner,
        })
        .into_iter()
        .collect();
    input
}

fn capture(input: ContextualSourceInput) -> Arc<ProcessedSource> {
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), source.text());
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        source.text().code_units()
    );
    processed
}

fn assert_owner(
    analyzed: &AnalyzedSource,
    processed: &Arc<ProcessedSource>,
    raw_ids: &[usize],
    facts_owner: usize,
    joint: bool,
) {
    assert!(std::ptr::eq(
        analyzed.processed_source(),
        processed.as_ref()
    ));
    assert_eq!(analyzed.source(), processed.source());
    let owner = analyzed
        .captured_owner()
        .expect("actual captured output owner");
    assert_eq!(owner.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(owner.members().len(), raw_ids.len());
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        raw_ids
            .iter()
            .map(|&id| processed.captured_paragraph(id).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        owner.facts_owner(),
        processed.captured_paragraph(facts_owner).unwrap()
    );
    assert_eq!(
        owner.joint_flow(),
        if joint {
            processed.captured_joint_flow(0)
        } else {
            None
        }
    );
}

fn assert_originals(
    analyzed: &AnalyzedSource,
    processed: &Arc<ProcessedSource>,
    origins: &[(usize, Range)],
) {
    assert_eq!(analyzed.inner.paragraph.origins.len(), origins.len());
    for (local, &(input_origin, range)) in origins.iter().enumerate() {
        let original = &analyzed.inner.paragraph.origins[local];
        assert_eq!(original.aggregate, range);
        assert_eq!(original.local, range);
        assert_eq!(
            original.original(analyzed.processed_source()),
            processed.source()
        );
        let map = &analyzed.inner.paragraph.origin_map;
        assert_eq!(
            map.flow_origin(map.view(local).unwrap()).captured_event(),
            Some(processed.captured_origin(input_origin).unwrap().index)
        );
    }
    for row in 0..analyzed.inner.transform.rows.len() {
        for piece in analyzed.inner.transform.original_pieces(row) {
            assert!(
                origins.iter().any(|&(input_origin, range)| {
                    piece.origin.captured_event()
                        == Some(processed.captured_origin(input_origin).unwrap().index)
                        && range.start <= piece.aggregate.start
                        && piece.aggregate.end <= range.end
                        && piece.local == piece.aggregate
                }),
                "every real transform contribution retains its actual original owner"
            );
        }
    }
}

fn assert_projection(analyzed: &AnalyzedSource, text: &str, atom_sources: &[Range]) {
    assert_eq!(analyzed.inner.projection.text, text);
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        text.chars().map(u32::from).collect::<Vec<_>>()
    );
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.source)
            .collect::<Vec<_>>(),
        atom_sources
    );
}

#[test]
fn domain_context_preserves_independent_outputs_and_unequal_used_facts() {
    for (relationship, first) in [
        (ParagraphConnectionKind::Continuous, "οσ"),
        (ParagraphConnectionKind::Disconnected, "ος"),
    ] {
        // Final_Sigma sees the actual following Alpha only across Continuous.
        // Source context is not authority to merge independently used facts.
        let processed = capture(request(
            &["ΟΣ", "Α"],
            &[1, 0],
            &[
                facts(
                    ParagraphBaseDirection::LeftToRight,
                    ParagraphStrut::DefaultFont,
                ),
                facts(
                    ParagraphBaseDirection::RightToLeft,
                    ParagraphStrut::AllocatedContributions,
                ),
            ],
            lowercase(),
            Some(relationship),
            None,
        ));
        let raw = processed.paragraphs().collect::<Vec<_>>();
        assert_eq!(
            raw[0].captured_paragraph(),
            processed.captured_paragraph(1).unwrap()
        );
        assert_eq!(
            raw[1].captured_paragraph(),
            processed.captured_paragraph(0).unwrap()
        );
        // Either authentic member selects the complete same domain, not just
        // that member's output. Output order follows actual source events.
        let domain = AnalyzedDomain::from_processed_paragraph(
            raw[1].clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert!(Arc::ptr_eq(domain.processed_source(), &processed));
        assert_eq!(domain.domain(), processed.captured_scope(0).unwrap());
        assert_eq!(domain.outputs().len(), 2);
        let outputs = domain.outputs().collect::<Vec<_>>();
        assert_projection(outputs[0], first, &[Range::new(0, 1), Range::new(1, 2)]);
        assert_projection(outputs[1], "α", &[Range::new(2, 3)]);
        for (i, id) in [1, 0].into_iter().enumerate() {
            assert_owner(outputs[i], &processed, &[id], id, false);
            assert_originals(
                outputs[i],
                &processed,
                &[(
                    i,
                    if i == 0 {
                        Range::new(0, 2)
                    } else {
                        Range::new(2, 3)
                    },
                )],
            );
            let exact = AnalyzedSource::from_processed_paragraph(
                raw[i].clone(),
                AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            assert_eq!(
                exact.inner.projection.text,
                outputs[i].inner.projection.text
            );
            assert_owner(&exact, &processed, &[id], id, false);
        }
        assert_eq!(
            outputs[0].inner.paragraph.base_direction,
            Some(ParagraphBaseDirection::LeftToRight)
        );
        assert_eq!(outputs[0].paragraph_strut(), ParagraphStrut::DefaultFont);
        assert_eq!(
            outputs[1].inner.paragraph.base_direction,
            Some(ParagraphBaseDirection::RightToLeft)
        );
        assert_eq!(
            outputs[1].paragraph_strut(),
            ParagraphStrut::AllocatedContributions
        );
        assert_ne!(outputs[0].identity(), outputs[1].identity());
    }
}

#[test]
fn a_whole_joint_output_uses_its_nonterminal_facts_owner_without_widening_exact_views() {
    let used = facts(
        ParagraphBaseDirection::RightToLeft,
        ParagraphStrut::AllocatedContributions,
    );
    let processed = capture(request(
        &["Ο", "Σ", "Α"],
        &[2, 0, 1],
        &[used; 3],
        lowercase(),
        Some(ParagraphConnectionKind::Continuous),
        Some(0),
    ));
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let domain = AnalyzedDomain::from_processed_paragraph(
        raw[2].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(domain.outputs().len(), 1);
    let whole = domain.outputs().next().unwrap();
    assert_projection(
        whole,
        "οσα",
        &[Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)],
    );
    assert_owner(whole, &processed, &[2, 0, 1], 0, true);
    assert_originals(
        whole,
        &processed,
        &[
            (0, Range::new(0, 1)),
            (1, Range::new(1, 2)),
            (2, Range::new(2, 3)),
        ],
    );
    assert_eq!(
        whole.inner.paragraph.base_direction,
        Some(ParagraphBaseDirection::RightToLeft)
    );
    assert_eq!(
        whole.paragraph_strut(),
        ParagraphStrut::AllocatedContributions
    );
    for (i, expected) in ["ο", "σ", "α"].into_iter().enumerate() {
        let exact = AnalyzedSource::from_processed_paragraph(
            raw[i].clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(exact.inner.projection.text, expected);
        let id = [2, 0, 1][i];
        assert_owner(&exact, &processed, &[id], id, false);
        assert_eq!(exact.inner.paragraph.origins.len(), 1);
    }
}

#[test]
fn a_covering_joint_owns_the_complete_mark_unit_but_does_not_expand_the_exact_constructor() {
    let used = facts(
        ParagraphBaseDirection::LeftToRight,
        ParagraphStrut::DefaultFont,
    );
    let processed = capture(request(
        &["i", "\u{307}"],
        &[1, 0],
        &[used; 2],
        TextTransform::none(),
        Some(ParagraphConnectionKind::Continuous),
        Some(1),
    ));
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let domain = AnalyzedDomain::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(domain.outputs().len(), 1);
    let whole = domain.outputs().next().unwrap();
    assert_projection(whole, "i\u{307}", &[Range::new(0, 1), Range::new(1, 2)]);
    assert_owner(whole, &processed, &[1, 0], 1, true);
    assert_originals(
        whole,
        &processed,
        &[(0, Range::new(0, 1)), (1, Range::new(1, 2))],
    );
    assert_eq!(whole.graphemes().collect::<Vec<_>>(), [Range::new(0, 2)]);
    for member in raw {
        let error = AnalyzedSource::from_processed_paragraph(
            member,
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::UnsupportedFeature);
        assert!(
            matches!(error.detail(), Some(ErrorDetail::TextTransform { reason: TextTransformError::IndivisibleCoverage { source } }) if *source == Range::new(0, 2))
        );
    }
}

#[test]
fn empty_original_members_and_non_bmp_resources_retain_their_actual_capture_owners() {
    use crate::protocol::paragraph::SemanticAnchorItem;
    let used = facts(
        ParagraphBaseDirection::LeftToRight,
        ParagraphStrut::AllocatedContributions,
    );
    let input = request(
        &["A", "", "𐐀"],
        &[2, 0, 1],
        &[used; 3],
        TextTransform::none(),
        Some(ParagraphConnectionKind::Continuous),
        None,
    );
    let processed = capture(input.clone());
    let other = capture(input);
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let domain = AnalyzedDomain::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 3);
    let outputs = domain.outputs().collect::<Vec<_>>();
    for (i, (id, extent, expected, atom_sources)) in [
        (2, Range::new(0, 1), "A", vec![Range::new(0, 1)]),
        (0, Range::new(1, 1), "", vec![]),
        (1, Range::new(1, 3), "𐐀", vec![Range::new(1, 3)]),
    ]
    .into_iter()
    .enumerate()
    {
        assert_projection(outputs[i], expected, &atom_sources);
        assert_owner(outputs[i], &processed, &[id], id, false);
        assert_originals(outputs[i], &processed, &[(i, extent)]);
        assert_eq!(raw[i].source_pieces().collect::<Vec<_>>(), [extent]);
        assert_eq!(
            super::text(raw[i].plaintext_items(Affinity::After)),
            expected.encode_utf16().collect::<Vec<_>>()
        );
    }
    assert!(outputs[1].inner.projection.atoms.is_empty());
    let anchors = outputs[1]
        .inner
        .paragraph
        .anchor_order
        .iter()
        .filter_map(|anchor| match anchor.item {
            SemanticAnchorItem::EmptyOrigin(origin) => Some(origin),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(anchors.len(), 1);
    assert_eq!(
        outputs[1].inner.paragraph.origins[anchors[0]].aggregate,
        Range::new(1, 1)
    );
    assert_eq!(
        processed.source().text().code_units(),
        [0x41, 0xd801, 0xdc00]
    );
    let other_domain = AnalyzedDomain::from_processed_paragraph(
        other.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert!(Arc::ptr_eq(other_domain.processed_source(), &other));
    assert_ne!(domain.domain(), other_domain.domain());
    assert_ne!(
        outputs[1].captured_owner().unwrap().facts_owner(),
        other_domain
            .outputs()
            .nth(1)
            .unwrap()
            .captured_owner()
            .unwrap()
            .facts_owner()
    );
    assert_eq!(processed.source(), other.source());
}

#[test]
fn missing_material_source_context_refuses_the_batch_but_not_a_definite_independent_output() {
    let used = facts(
        ParagraphBaseDirection::LeftToRight,
        ParagraphStrut::DefaultFont,
    );
    let processed = capture(request(
        &["ΟΣ", "Α"],
        &[1, 0],
        &[used; 2],
        lowercase(),
        None,
        None,
    ));
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let prior = AnalyzedSource::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let prior_identity = prior.identity();
    assert_projection(&prior, "α", &[Range::new(2, 3)]);
    let error = AnalyzedDomain::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::MissingSourceContinuity {
                before_origin: 0,
                after_origin: 1,
                operation: SourceContinuityOperation::Transformation
            }
        })
    ));
    assert_eq!(prior.identity(), prior_identity);
    assert_owner(&prior, &processed, &[0], 0, false);
    assert_eq!(processed.processed_text(), &Text::from("ΟΣΑ"));
    assert_eq!(
        raw[0].source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 2)]
    );
    // No conditional transform and no crossing EGC means this same Unknown
    // relationship is harmless, so it must not become a blanket batch gate.
    let neutral = capture(request(
        &["ΟΣ", "Α"],
        &[1, 0],
        &[used; 2],
        TextTransform::none(),
        None,
        None,
    ));
    let neutral_domain = AnalyzedDomain::from_processed_paragraph(
        neutral.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(
        neutral_domain
            .outputs()
            .map(|output| output.inner.projection.text.as_str())
            .collect::<Vec<_>>(),
        ["ΟΣ", "Α"]
    );
}

#[test]
fn a_late_invalid_authored_style_cannot_replace_prior_published_analysis_or_source() {
    let used = facts(
        ParagraphBaseDirection::LeftToRight,
        ParagraphStrut::DefaultFont,
    );
    let mut input = request(
        &["A", "B", "C"],
        &[2, 0, 1],
        &[used; 3],
        TextTransform::none(),
        Some(ParagraphConnectionKind::Continuous),
        None,
    );
    input.source.span(
        Range::new(2, 3),
        Style {
            white_space: WhiteSpace::Preserve,
            size: -1.0,
            ..Style::default()
        },
    );
    // Refresh the actual original resources after adding the authored span;
    // all three are genuine ranges of that same final Source value.
    input.origins = [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    let processed = capture(input);
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let prior = AnalyzedSource::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let identity = prior.identity();
    assert_projection(&prior, "A", &[Range::new(0, 1)]);
    let error = AnalyzedDomain::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidStyle);
    assert_eq!(prior.identity(), identity);
    assert_owner(&prior, &processed, &[2], 2, false);
    assert_projection(&prior, "A", &[Range::new(0, 1)]);
    assert_eq!(processed.processed_text(), &Text::from("ABC"));
    assert_eq!(
        raw.iter()
            .map(ProcessedParagraph::source_range)
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
    );
    assert!(
        raw.iter()
            .all(|view| Arc::ptr_eq(view.processed_source(), &processed))
    );
}

#[test]
fn a_cross_member_grapheme_requires_joint_authority_in_actual_source_member_order() {
    let used = facts(
        ParagraphBaseDirection::LeftToRight,
        ParagraphStrut::DefaultFont,
    );
    let processed = capture(request(
        &["i", "\u{307}"],
        &[1, 0],
        &[used; 2],
        TextTransform::none(),
        Some(ParagraphConnectionKind::Continuous),
        None,
    ));
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let error = AnalyzedDomain::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    // The completed EGC needs both real members. Required IDs follow their
    // genuine event order (1 then 0), not numeric order or the seed member.
    assert!(
        matches!(error.detail(), Some(ErrorDetail::SourceProcessing {
        reason: SourceProcessingError::MissingJointAnalysisAuthority {
            first_paragraph: 1, last_paragraph: 0, source,
        },
    }) if *source == Range::new(0, 2))
    );
    assert_eq!(
        raw[0].captured_paragraph(),
        processed.captured_paragraph(1).unwrap()
    );
    assert_eq!(
        raw[1].captured_paragraph(),
        processed.captured_paragraph(0).unwrap()
    );
    assert_eq!(
        raw[0].source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 1)]
    );
    assert_eq!(
        raw[1].source_pieces().collect::<Vec<_>>(),
        [Range::new(1, 2)]
    );
    assert_eq!(processed.processed_text(), &Text::from("i\u{307}"));
    assert!(processed.captured_joint_flow(0).is_none());
    // A genuine Disconnected relationship stops the unit. These are then two
    // independently owned outputs and acquire no joint permission requirement.
    let stopped = capture(request(
        &["i", "\u{307}"],
        &[1, 0],
        &[used; 2],
        TextTransform::none(),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ));
    let domain = AnalyzedDomain::from_processed_paragraph(
        stopped.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(domain.outputs().len(), 2);
    let outputs = domain.outputs().collect::<Vec<_>>();
    assert_projection(outputs[0], "i", &[Range::new(0, 1)]);
    assert_projection(outputs[1], "\u{307}", &[Range::new(1, 2)]);
    assert_owner(outputs[0], &stopped, &[1], 1, false);
    assert_owner(outputs[1], &stopped, &[0], 0, false);
}
