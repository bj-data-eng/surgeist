//! Source-cell outer levels are real analysis results even without inner
//! presentation atoms, and remain local to each completed output binding.

use crate::*;
use std::sync::Arc;

fn root_input(
    text: &str,
    whitespace: WhiteSpace,
    mode: TextCombineUpright,
    outer: ParagraphBaseDirection,
    inner: Option<ParagraphBaseDirection>,
) -> ContextualSourceInput {
    let mut input = super::input(text, whitespace);
    input.source = Source::identified(text, Id::from_u64(78480), 21);
    input.origins = vec![super::origin(
        &input.source,
        Range::new(0, text.encode_utf16().count()),
    )];
    let SourceProcessingScope::Block {
        topology,
        base_style,
        used_direction,
        writing_axis,
        horizontal_inner_direction,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!("actual source Block")
    };
    *topology = SourceProcessingTopology::Complete;
    *writing_axis = Some(SourceWritingAxis::Vertical);
    *used_direction = outer;
    *horizontal_inner_direction = inner;
    base_style.text_combine_upright = mode;
    base_style.text_transform = TextTransform::none();
    input.paragraphs[0].base_direction = outer;
    input
}

fn capture(input: ContextualSourceInput, processed_text: &str) -> Arc<ProcessedSource> {
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(processed_text));
    assert!(processed.source().boxes().is_empty());
    assert!(processed.captured_joint_flow(0).is_none());
    processed
}

#[test]
fn an_empty_used_whitespace_cell_keeps_its_upright_l_outer_level_without_fake_atoms() {
    let processed = capture(
        root_input(
            "  ",
            WhiteSpace::Collapse,
            TextCombineUpright::All,
            ParagraphBaseDirection::RightToLeft,
            Some(ParagraphBaseDirection::LeftToRight),
        ),
        " ",
    );
    let raw = processed.paragraphs().next().unwrap();
    assert_eq!(raw.source_pieces().collect::<Vec<_>>(), [Range::new(0, 2)]);
    let domain =
        AnalyzedDomain::from_processed_paragraph(raw, AnalysisTypography::HorizontalComposition)
            .unwrap();
    assert_eq!(domain.outputs().len(), 1);
    let output = domain.outputs().next().unwrap();
    assert!(std::ptr::eq(output.processed_source(), processed.as_ref()));
    assert_eq!(
        output.inner.paragraph.base_direction,
        Some(ParagraphBaseDirection::RightToLeft)
    );
    assert_eq!(output.inner.analysis.base_level, 1);
    assert!(output.inner.projection.atoms.is_empty());
    assert_eq!(output.inner.projection.text, "");
    assert!(output.inner.analysis.levels.is_empty());
    assert_eq!(output.combined_cells().len(), 1);
    let cell = output.combined_cells().next().unwrap();
    assert_eq!(cell.scope(), processed.captured_scope(0).unwrap());
    assert_eq!(cell.parent_scope(), None);
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.inner_direction(), ParagraphBaseDirection::LeftToRight);
    assert_eq!(cell.selection_unit_count(), 2);
    assert_eq!(cell.used_unit_count(), 0);
    assert!(cell.used_code_points().next().is_none());
    assert!(cell.presentation_code_points().next().is_none());
    assert!(cell.inner_bidi_levels().next().is_none());
    // A real nonempty selected upright-L cell in base RTL resolves to level2,
    // independently of its genuinely empty inner/presentation atom ranges.
    assert_eq!(cell.outer_bidi_level().get(), 2);
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 2);
    for (index, piece) in pieces.iter().enumerate() {
        assert_eq!(piece.source(), processed.source());
        assert_eq!(piece.local(), Range::new(index, index + 1));
        assert_eq!(piece.aggregate(), Range::new(index, index + 1));
        assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
        assert!(piece.complete_text_node().is_none());
        assert!(!piece.is_used());
        assert!(!piece.has_presentation_emission());
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(0).unwrap()
        );
        assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
    }
    assert_eq!(
        output
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    assert_eq!(output.source(), processed.source());
    assert_eq!(processed.source().text(), &Text::from("  "));
    assert_eq!(processed.processed_text(), &Text::from(" "));
}

#[test]
fn independent_outputs_use_their_own_local_cell_binding_and_actual_paragraph_base() {
    let mut input = root_input(
        "AB",
        WhiteSpace::Preserve,
        TextCombineUpright::None,
        ParagraphBaseDirection::LeftToRight,
        None,
    );
    let mut cell_style = super::style(WhiteSpace::Preserve, 10.0);
    cell_style.text_combine_upright = TextCombineUpright::All;
    for range in [Range::new(0, 1), Range::new(1, 2)] {
        input.source.span(range, cell_style.clone());
        input.scopes.push(SourceProcessingScope::Inline {
            trim: WhiteSpaceTrim::default(),
            style: Some(cell_style.clone()),
            boundaries: None,
            alignment_boundary: None,
            font_reference: None,
            horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
        });
    }
    input.origins = [Range::new(0, 1), Range::new(1, 2)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    // Actual event order is raw1 LTR, then raw0 RTL. Equal domain/root style
    // supplies no authority to merge their independently used paragraph facts.
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
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::EnterScope(2),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitScope(2),
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    let processed = capture(input, "AB");
    let raw = processed.paragraphs().collect::<Vec<_>>();
    assert_eq!(
        raw[0].captured_paragraph(),
        processed.captured_paragraph(1).unwrap()
    );
    assert_eq!(
        raw[1].captured_paragraph(),
        processed.captured_paragraph(0).unwrap()
    );
    let domain = AnalyzedDomain::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 2);
    for (index, (raw_id, direction, base, outer, character, strut)) in [
        (
            1,
            ParagraphBaseDirection::LeftToRight,
            0,
            0,
            0x41,
            ParagraphStrut::DefaultFont,
        ),
        (
            0,
            ParagraphBaseDirection::RightToLeft,
            1,
            2,
            0x42,
            ParagraphStrut::AllocatedContributions,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let output = domain.outputs().nth(index).unwrap();
        assert!(std::ptr::eq(output.processed_source(), processed.as_ref()));
        assert_eq!(output.source(), processed.source());
        assert_eq!(output.inner.paragraph.base_direction, Some(direction));
        assert_eq!(output.paragraph_strut(), strut);
        assert_eq!(output.inner.analysis.base_level, base);
        assert_eq!(
            output
                .inner
                .projection
                .atoms
                .iter()
                .map(|atom| (atom.code_point, atom.source))
                .collect::<Vec<_>>(),
            [(character, Range::new(index, index + 1))]
        );
        assert_eq!(output.inner.analysis.levels, [outer]);
        let owner = output.captured_owner().unwrap();
        assert_eq!(
            owner.members().collect::<Vec<_>>(),
            [processed.captured_paragraph(raw_id).unwrap()]
        );
        assert_eq!(
            owner.facts_owner(),
            processed.captured_paragraph(raw_id).unwrap()
        );
        assert!(owner.joint_flow().is_none());
        assert_eq!(output.combined_cells().len(), 1);
        let cell = output.combined_cells().next().unwrap();
        assert_eq!(cell.scope(), processed.captured_scope(index + 1).unwrap());
        assert_eq!(
            cell.parent_scope(),
            Some(processed.captured_scope(0).unwrap())
        );
        assert_eq!(cell.mode(), TextCombineUpright::All);
        assert_eq!(cell.inner_direction(), ParagraphBaseDirection::LeftToRight);
        assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [character]);
        assert_eq!(
            cell.inner_bidi_levels()
                .map(BidiLevel::get)
                .collect::<Vec<_>>(),
            [0]
        );
        // Each output borrows its first LOCAL binding. The second output has
        // a different actual paragraph base and a later shared-plan group.
        assert_eq!(cell.outer_bidi_level().get(), outer);
        let pieces = cell.source_pieces().collect::<Vec<_>>();
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].source(), processed.source());
        assert_eq!(pieces[0].aggregate(), Range::new(index, index + 1));
        assert_eq!(pieces[0].local(), Range::new(index, index + 1));
        assert!(pieces[0].is_used());
        assert!(pieces[0].has_presentation_emission());
        assert_eq!(
            pieces[0].origin_position(),
            processed.captured_origin(index).unwrap()
        );
        assert_eq!(
            pieces[0].paragraph(),
            processed.captured_paragraph(raw_id).unwrap()
        );
    }
    assert_eq!(processed.source().text(), &Text::from("AB"));
    assert_eq!(
        processed.captured_connection(0).unwrap().kind(),
        ParagraphConnectionKind::Continuous
    );
}
