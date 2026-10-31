//! Functional source-selection qualification for the new domain front.
//! WM4/native source authority determines selection and joins before fonts.
//! Selected cells deliberately stop at the public preparation capability guard.

use crate::*;
use std::sync::Arc;

fn split(
    mode: TextCombineUpright,
    axis: SourceWritingAxis,
    relationship: Option<ParagraphConnectionKind>,
    joint: bool,
) -> ContextualSourceInput {
    let mut input = super::input("AB", WhiteSpace::Preserve);
    input.source = Source::identified("AB", Id::from_u64(78471), 13);
    input.origins = [Range::new(0, 1), Range::new(1, 2)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    let SourceProcessingScope::Block {
        base_style,
        topology,
        writing_axis,
        horizontal_inner_direction,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!("actual owning Block")
    };
    base_style.text_combine_upright = mode;
    *topology = SourceProcessingTopology::Complete;
    *writing_axis = Some(axis);
    *horizontal_inner_direction = Some(ParagraphBaseDirection::LeftToRight);
    let used = SourceProcessingParagraph {
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
    };
    // Actual raw IDs run 1 then 0. Neither their numeric order nor the member
    // supplied to domain analysis is the source-order authority.
    input.paragraphs = vec![used; 2];
    input.connections.clear();
    input.boundary_policies.clear();
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(1),
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
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    input.joint_flows = if joint {
        vec![SourceProcessingJointFlow {
            scope: 0,
            paragraphs: vec![1, 0],
            facts_owner: 1,
        }]
    } else {
        vec![]
    };
    input
}

fn captured(input: ContextualSourceInput) -> Arc<ProcessedSource> {
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from("AB"));
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        [0x41, 0x42]
    );
    let raw = processed.paragraphs().collect::<Vec<_>>();
    assert_eq!(raw.len(), 2);
    for (i, id) in [1, 0].into_iter().enumerate() {
        assert_eq!(
            raw[i].captured_paragraph(),
            processed.captured_paragraph(id).unwrap()
        );
        assert_eq!(
            raw[i].source_pieces().collect::<Vec<_>>(),
            [Range::new(i, i + 1)]
        );
        assert_eq!(
            super::text(raw[i].plaintext_items(Affinity::After)),
            [if i == 0 { 0x41 } else { 0x42 }]
        );
    }
    processed
}

fn analyze(processed: &Arc<ProcessedSource>) -> Result<AnalyzedDomain> {
    // Keep typography IDENTICAL for horizontal/vertical controls. Only the
    // authentic captured domain axis may enable combination.
    AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().nth(1).unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
}

fn assert_custody(
    output: &AnalyzedSource,
    processed: &Arc<ProcessedSource>,
    ids: &[usize],
    joint: bool,
) {
    assert!(std::ptr::eq(output.processed_source(), processed.as_ref()));
    assert_eq!(output.source(), processed.source());
    let owner = output.captured_owner().unwrap();
    assert_eq!(owner.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        ids.iter()
            .map(|&id| processed.captured_paragraph(id).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        owner.facts_owner(),
        processed.captured_paragraph(ids[0]).unwrap()
    );
    assert_eq!(
        owner.joint_flow(),
        if joint {
            processed.captured_joint_flow(0)
        } else {
            None
        }
    );
    assert_eq!(output.inner.paragraph.origins.len(), ids.len());
    for (local, &id) in ids.iter().enumerate() {
        let input_origin = if id == 1 { 0 } else { 1 };
        let range = Range::new(input_origin, input_origin + 1);
        let origin = &output.inner.paragraph.origins[local];
        assert_eq!(origin.aggregate, range);
        assert_eq!(origin.local, range);
        assert_eq!(
            origin.original(output.processed_source()),
            processed.source()
        );
        let map = &output.inner.paragraph.origin_map;
        assert_eq!(
            map.flow_origin(map.view(local).unwrap()).captured_event(),
            Some(processed.captured_origin(input_origin).unwrap().index)
        );
    }
}

fn empty_catalogue() -> System {
    System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap()
}

fn assert_selected_cell_cannot_fall_through_to_ordinary_shaping(output: &AnalyzedSource) {
    let mut system = empty_catalogue();
    let stats = system.stats();
    let identity = output.identity();
    assert_eq!(system.registered_font_faces().count(), 0);
    let error = system
        .prepare(output, PreparationOptions::default())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::TextCombine {
            reason: TextCombineError::UnsupportedCellPreparation
        })
    ));
    assert_eq!(system.stats(), stats);
    assert_eq!(output.identity(), identity);
    // Stats are the public System counters, not a private font-query trace.
    // The precise capability error with this empty catalogue establishes that
    // the cell was refused before an ordinary missing-font result could escape.
}

fn assert_ordinary_independent_outputs(domain: &AnalyzedDomain, processed: &Arc<ProcessedSource>) {
    assert!(Arc::ptr_eq(domain.processed_source(), processed));
    assert_eq!(domain.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(domain.outputs().len(), 2);
    for (i, output) in domain.outputs().enumerate() {
        let id = [1, 0][i];
        assert_custody(output, processed, &[id], false);
        assert_eq!(output.inner.projection.text, if i == 0 { "A" } else { "B" });
        assert_eq!(output.inner.projection.atoms.len(), 1);
        assert_eq!(
            output.inner.projection.atoms[0].code_point,
            if i == 0 { 0x41 } else { 0x42 }
        );
        assert_eq!(
            output.inner.projection.atoms[0].source,
            Range::new(i, i + 1)
        );
        // No selected cell exists here. The real ordinary preparation path
        // therefore reaches its own honest missing-font refusal in an empty
        // registered-only catalogue, rather than the cell capability guard.
        let mut system = empty_catalogue();
        let error = system
            .prepare(output, PreparationOptions::default())
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::NoUsableFontFace);
    }
}

#[test]
fn source_continuous_all_requires_joint_ownership_in_genuine_member_order() {
    let processed = captured(split(
        TextCombineUpright::All,
        SourceWritingAxis::Vertical,
        Some(ParagraphConnectionKind::Continuous),
        false,
    ));
    let error = analyze(&processed).unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(
        matches!(error.detail(), Some(ErrorDetail::SourceProcessing {
        reason: SourceProcessingError::MissingJointAnalysisAuthority { first_paragraph: 1, last_paragraph: 0, source },
    }) if *source == Range::new(0, 2))
    );
    assert!(processed.captured_joint_flow(0).is_none());
    assert_eq!(
        processed.captured_connection(0).unwrap().kind(),
        ParagraphConnectionKind::Continuous
    );
    assert_eq!(processed.processed_text(), &Text::from("AB"));
}

#[test]
fn materially_unknown_all_reports_missing_source_before_join_permission() {
    let processed = captured(split(
        TextCombineUpright::All,
        SourceWritingAxis::Vertical,
        None,
        false,
    ));
    let error = analyze(&processed).unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::MissingSourceContinuity {
                before_origin: 0,
                after_origin: 1,
                operation: SourceContinuityOperation::Combination
            },
        })
    ));
    assert!(processed.captured_connection(0).is_none());
    assert!(processed.captured_joint_flow(0).is_none());
    assert_eq!(processed.source().text(), &Text::from("AB"));
}

#[test]
fn ineligible_ascii_letters_do_not_demand_digit_combination_joins_or_unused_continuity() {
    for limit in [
        TextCombineUprightDigits::Two,
        TextCombineUprightDigits::Three,
        TextCombineUprightDigits::Four,
    ] {
        for relationship in [None, Some(ParagraphConnectionKind::Continuous)] {
            let mut input = split(
                TextCombineUpright::Digits(limit),
                SourceWritingAxis::Vertical,
                relationship,
                false,
            );
            input.paragraphs[0].base_direction = ParagraphBaseDirection::RightToLeft;
            input.paragraphs[0].paragraph_strut = ParagraphStrut::AllocatedContributions;
            let processed = captured(input);
            let domain = analyze(&processed).unwrap();
            assert_ordinary_independent_outputs(&domain, &processed);
            let second = domain.outputs().nth(1).unwrap();
            assert_eq!(
                second.inner.paragraph.base_direction,
                Some(ParagraphBaseDirection::RightToLeft)
            );
            assert_eq!(
                second.paragraph_strut(),
                ParagraphStrut::AllocatedContributions
            );
        }
    }
}

#[test]
fn an_authentic_horizontal_axis_disables_all_without_typography_or_fact_merging() {
    for relationship in [None, Some(ParagraphConnectionKind::Continuous)] {
        let processed = captured(split(
            TextCombineUpright::All,
            SourceWritingAxis::Horizontal,
            relationship,
            false,
        ));
        assert_eq!(
            processed.captured_scope(0).unwrap().writing_axis(),
            Some(SourceWritingAxis::Horizontal)
        );
        let domain = analyze(&processed).unwrap();
        assert_ordinary_independent_outputs(&domain, &processed);
    }
}

#[test]
fn covering_joint_all_publishes_one_real_output_and_refuses_unimplemented_cell_preparation() {
    let processed = captured(split(
        TextCombineUpright::All,
        SourceWritingAxis::Vertical,
        Some(ParagraphConnectionKind::Continuous),
        true,
    ));
    let domain = analyze(&processed).unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let output = domain.outputs().next().unwrap();
    assert_custody(output, &processed, &[1, 0], true);
    assert_selected_cell_cannot_fall_through_to_ordinary_shaping(output);
    assert_eq!(processed.processed_text(), &Text::from("AB"));
}

#[test]
fn a_real_disconnection_yields_owned_singleton_cells_without_cross_member_join_demand() {
    let processed = captured(split(
        TextCombineUpright::All,
        SourceWritingAxis::Vertical,
        Some(ParagraphConnectionKind::Disconnected),
        false,
    ));
    let domain = analyze(&processed).unwrap();
    assert_eq!(domain.outputs().len(), 2);
    for (i, output) in domain.outputs().enumerate() {
        assert_custody(output, &processed, &[[1, 0][i]], false);
        // All accepts a nonempty one-unit interval. Its real Disconnected
        // boundary prevents a crossing candidate, not its own local cell.
        assert_selected_cell_cannot_fall_through_to_ordinary_shaping(output);
    }
    assert_eq!(
        processed.captured_connection(0).unwrap().kind(),
        ParagraphConnectionKind::Disconnected
    );
}

#[test]
fn real_inline_box_lookaround_rejects_both_partial_root_all_sequences() {
    let mut input = split(
        TextCombineUpright::All,
        SourceWritingAxis::Vertical,
        Some(ParagraphConnectionKind::Continuous),
        false,
    );
    let mut inline_style = super::style(WhiteSpace::Preserve, 10.0);
    inline_style.text_combine_upright = TextCombineUpright::All;
    input.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: Some(inline_style),
        boundaries: None,
        alignment_boundary: None,
        font_reference: None,
        horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
    });
    // The B occurrence truly belongs inside a real ordinary inline box. Both
    // candidate sides are All under the same authentic root-All ancestor;
    // WM4 lookaround rejects partial sequences rather than crossing that box.
    let at = input
        .events
        .iter()
        .position(|event| matches!(event, SourceProcessingEvent::Origin { origin: 1, .. }))
        .unwrap();
    input
        .events
        .insert(at, SourceProcessingEvent::EnterScope(1));
    input
        .events
        .insert(at + 2, SourceProcessingEvent::ExitScope(1));
    let processed = captured(input);
    let inline = processed.captured_scope(1).unwrap();
    assert_eq!(inline.parent(), Some(processed.captured_scope(0).unwrap()));
    assert_eq!(inline.writing_axis(), Some(SourceWritingAxis::Vertical));
    let domain = analyze(&processed).unwrap();
    assert_ordinary_independent_outputs(&domain, &processed);
}
