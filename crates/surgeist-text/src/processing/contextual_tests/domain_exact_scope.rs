//! Exact captured-view qualification using the new completed-cell observation.
//! Contained source cells and definite ineligibility require no domain widening;
//! #1588 demand locality excludes unrelated neighbor font/style validation.

use crate::*;
use std::sync::Arc;

fn root_input(
    text: &str,
    mode: TextCombineUpright,
    axis: SourceWritingAxis,
) -> ContextualSourceInput {
    let mut input = super::input(text, WhiteSpace::Preserve);
    input.source = Source::identified(text, Id::from_u64(78476), 17);
    input.origins = vec![super::origin(
        &input.source,
        Range::new(0, text.encode_utf16().count()),
    )];
    let SourceProcessingScope::Block {
        topology,
        base_style,
        writing_axis,
        horizontal_inner_direction,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!("actual root Block")
    };
    *topology = SourceProcessingTopology::Complete;
    *writing_axis = Some(axis);
    *horizontal_inner_direction = Some(ParagraphBaseDirection::LeftToRight);
    base_style.text_combine_upright = mode;
    base_style.text_transform = TextTransform::none();
    input
}

fn with_unused_neighbor(
    mode: TextCombineUpright,
    axis: SourceWritingAxis,
    inline_all: bool,
) -> ContextualSourceInput {
    let mut input = root_input("ABX", mode, axis);
    let mut unused = super::style(WhiteSpace::Preserve, 10.0);
    unused.size = -1.0;
    unused.text_combine_upright = mode;
    // Finite invalid size is admitted by source/whitespace capture. Its own
    // exact analysis will consume this field and must retain InvalidStyle.
    input.source.span(Range::new(2, 3), unused);
    if inline_all {
        let mut contained = super::style(WhiteSpace::Preserve, 10.0);
        contained.text_combine_upright = TextCombineUpright::All;
        input.source.span(Range::new(0, 2), contained.clone());
        input.scopes.push(SourceProcessingScope::Inline {
            trim: WhiteSpaceTrim::default(),
            style: Some(contained),
            boundaries: None,
            alignment_boundary: None,
            font_reference: None,
            horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
        });
    }
    // Source resources are the actual final authored Source, including both
    // genuine style winners; raw IDs intentionally run 1 then 0.
    input.origins = [Range::new(0, 2), Range::new(2, 3)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    input.paragraphs = vec![
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::RightToLeft,
            paragraph_strut: ParagraphStrut::AllocatedContributions,
        },
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        },
    ];
    input.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    input.boundary_policies = vec![super::policy(ParagraphBoundaryDefinition::Connection(0))];
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(1),
    ];
    if inline_all {
        input.events.push(SourceProcessingEvent::EnterScope(1));
    }
    input.events.push(SourceProcessingEvent::Origin {
        origin: 0,
        affinity: Affinity::After,
    });
    if inline_all {
        input.events.push(SourceProcessingEvent::ExitScope(1));
    }
    input.events.extend([
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    input
}

fn captured(input: ContextualSourceInput) -> Arc<ProcessedSource> {
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), source.text());
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        source.text().code_units()
    );
    assert!(processed.captured_joint_flow(0).is_none());
    processed
}

fn assert_source_and_ax(analyzed: &AnalyzedSource, expected: &str) {
    assert_eq!(analyzed.inner.host_projection.text, expected);
    #[cfg(feature = "text-accessibility")]
    {
        let id = accesskit::NodeId(78476);
        let mut state = Accessibility::new(78500);
        let update = analyzed.accessibility_update(&mut state, id, id).unwrap();
        assert_eq!(
            update
                .nodes
                .iter()
                .filter_map(|(_, node)| node.value())
                .collect::<String>(),
            expected
        );
    }
}

fn assert_exact(analyzed: &AnalyzedSource, processed: &Arc<ProcessedSource>, paragraph: usize) {
    assert!(std::ptr::eq(
        analyzed.processed_source(),
        processed.as_ref()
    ));
    assert_eq!(analyzed.source(), processed.source());
    assert_source_and_ax(analyzed, "AB");
    assert_eq!(
        analyzed.inner.paragraph.base_direction,
        Some(ParagraphBaseDirection::LeftToRight)
    );
    assert_eq!(analyzed.paragraph_strut(), ParagraphStrut::DefaultFont);
    let owner = analyzed.captured_owner().unwrap();
    assert_eq!(owner.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(paragraph).unwrap()]
    );
    assert_eq!(
        owner.facts_owner(),
        processed.captured_paragraph(paragraph).unwrap()
    );
    assert!(owner.joint_flow().is_none());
    assert_eq!(analyzed.inner.paragraph.origins.len(), 1);
    let origin = &analyzed.inner.paragraph.origins[0];
    assert_eq!(origin.aggregate, Range::new(0, 2));
    assert_eq!(origin.local, Range::new(0, 2));
    assert_eq!(
        origin.original(analyzed.processed_source()),
        processed.source()
    );
    let map = &analyzed.inner.paragraph.origin_map;
    assert_eq!(
        map.flow_origin(map.view(0).unwrap()).captured_event(),
        Some(processed.captured_origin(0).unwrap().index)
    );
}

fn assert_contained_cell(
    analyzed: &AnalyzedSource,
    processed: &Arc<ProcessedSource>,
    paragraph: usize,
    scope: usize,
) {
    assert_eq!(analyzed.combined_cells().len(), 1);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.scope(), processed.captured_scope(scope).unwrap());
    assert_eq!(
        cell.parent_scope(),
        if scope == 0 {
            None
        } else {
            Some(processed.captured_scope(0).unwrap())
        }
    );
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.inner_direction(), ParagraphBaseDirection::LeftToRight);
    assert_eq!(cell.selection_unit_count(), 2);
    assert_eq!(cell.used_unit_count(), 2);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41, 0x42]);
    assert_eq!(
        cell.inner_bidi_levels()
            .map(BidiLevel::get)
            .collect::<Vec<_>>(),
        [0, 0]
    );
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 2);
    for (index, piece) in pieces.iter().enumerate() {
        assert_eq!(piece.source(), processed.source());
        assert_eq!(piece.aggregate(), Range::new(index, index + 1));
        assert_eq!(piece.local(), Range::new(index, index + 1));
        assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
        assert!(piece.complete_text_node().is_none());
        assert!(piece.is_used());
        assert!(piece.has_presentation_emission());
        assert_eq!(piece.emission_subposition(), Some(0));
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(0).unwrap()
        );
        assert_eq!(
            piece.paragraph(),
            processed.captured_paragraph(paragraph).unwrap()
        );
    }
}

fn assert_unused_neighbor_is_actually_invalid(
    processed: &Arc<ProcessedSource>,
    prior: &AnalyzedSource,
) {
    let identity = prior.identity();
    let raw = processed.paragraphs().nth(1).unwrap();
    assert_eq!(
        raw.captured_paragraph(),
        processed.captured_paragraph(0).unwrap()
    );
    assert_eq!(raw.source_pieces().collect::<Vec<_>>(), [Range::new(2, 3)]);
    let error =
        AnalyzedSource::from_processed_paragraph(raw, AnalysisTypography::HorizontalComposition)
            .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidStyle);
    assert_eq!(prior.identity(), identity);
    assert_eq!(processed.processed_text(), &Text::from("ABX"));
    assert_exact(prior, processed, 1);
}

#[test]
fn exact_complete_single_raw_root_all_publishes_its_contained_cell_and_source_ax() {
    let processed = captured(root_input(
        "AB",
        TextCombineUpright::All,
        SourceWritingAxis::Vertical,
    ));
    let raw = processed.paragraphs().next().unwrap();
    assert_eq!(raw.source_pieces().collect::<Vec<_>>(), [Range::new(0, 2)]);
    let analyzed =
        AnalyzedSource::from_processed_paragraph(raw, AnalysisTypography::HorizontalComposition)
            .unwrap();
    assert_exact(&analyzed, &processed, 0);
    assert_contained_cell(&analyzed, &processed, 0, 0);
    assert_eq!(processed.paragraphs().len(), 1);
}

#[test]
fn exact_ineligible_digits_do_not_validate_an_unrequested_neighbor_font_field() {
    let mode = TextCombineUpright::Digits(TextCombineUprightDigits::Two);
    let processed = captured(with_unused_neighbor(
        mode,
        SourceWritingAxis::Vertical,
        false,
    ));
    let raw = processed.paragraphs().next().unwrap();
    assert_eq!(
        raw.captured_paragraph(),
        processed.captured_paragraph(1).unwrap()
    );
    let analyzed =
        AnalyzedSource::from_processed_paragraph(raw, AnalysisTypography::HorizontalComposition)
            .unwrap();
    assert_exact(&analyzed, &processed, 1);
    assert_eq!(analyzed.combined_cells().len(), 0);
    assert_unused_neighbor_is_actually_invalid(&processed, &analyzed);
}

#[test]
fn exact_inline_all_cell_does_not_publish_or_validate_its_unrelated_neighbor() {
    let processed = captured(with_unused_neighbor(
        TextCombineUpright::None,
        SourceWritingAxis::Vertical,
        true,
    ));
    let analyzed = AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_exact(&analyzed, &processed, 1);
    assert_contained_cell(&analyzed, &processed, 1, 1);
    assert_unused_neighbor_is_actually_invalid(&processed, &analyzed);
    assert_contained_cell(&analyzed, &processed, 1, 1);
    assert_eq!(processed.paragraphs().len(), 2);
}

#[test]
fn exact_known_horizontal_all_keeps_ordinary_output_and_neighbor_validation_local() {
    let processed = captured(with_unused_neighbor(
        TextCombineUpright::All,
        SourceWritingAxis::Horizontal,
        false,
    ));
    let analyzed = AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_exact(&analyzed, &processed, 1);
    assert_eq!(analyzed.combined_cells().len(), 0);
    assert_unused_neighbor_is_actually_invalid(&processed, &analyzed);
}
