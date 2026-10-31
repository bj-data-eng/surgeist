//! Independent source-only cell bidi and line-unit qualification. WM4 treats
//! composition as an upright TCU for outer bidi, while the real horizontal
//! payload resolves its own declared direction. Breaking keeps actual content.

use crate::*;
use std::sync::Arc;

fn root_input(
    text: &str,
    mode: TextCombineUpright,
    outer: ParagraphBaseDirection,
    inner: ParagraphBaseDirection,
) -> ContextualSourceInput {
    let mut input = super::input(text, WhiteSpace::Preserve);
    input.source = Source::identified(text, Id::from_u64(78477), 18);
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
        unreachable!("actual captured root Block")
    };
    *topology = SourceProcessingTopology::Complete;
    *writing_axis = Some(SourceWritingAxis::Vertical);
    *used_direction = outer;
    *horizontal_inner_direction = Some(inner);
    base_style.text_combine_upright = mode;
    base_style.text_transform = TextTransform::none();
    input.paragraphs[0].base_direction = outer;
    input
}

fn analyze(
    input: ContextualSourceInput,
    typography: AnalysisTypography,
) -> (Arc<ProcessedSource>, AnalyzedDomain) {
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), source.text());
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        source.text().code_units()
    );
    assert!(processed.source().boxes().is_empty());
    assert!(processed.captured_joint_flow(0).is_none());
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        typography,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let output = domain.outputs().next().unwrap();
    assert!(std::ptr::eq(output.processed_source(), processed.as_ref()));
    assert_eq!(output.source(), processed.source());
    assert_eq!(
        output
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    (processed, domain)
}

fn qualify_root_cell(text: &str, inner: ParagraphBaseDirection, expected_inner: &[u8]) {
    // Neither typography is a writing-mode proof. Both cases carry the same
    // explicit vertical domain axis and the same actual outer RTL facts.
    for typography in [
        AnalysisTypography::HorizontalComposition,
        AnalysisTypography::Mixed,
    ] {
        let (processed, domain) = analyze(
            root_input(
                text,
                TextCombineUpright::All,
                ParagraphBaseDirection::RightToLeft,
                inner,
            ),
            typography,
        );
        let output = domain.outputs().next().unwrap();
        assert_eq!(
            output.inner.paragraph.base_direction,
            Some(ParagraphBaseDirection::RightToLeft)
        );
        assert_eq!(output.inner.analysis.base_level, 1);
        assert_eq!(output.inner.projection.text, text);
        let mut cursor = 0;
        let expected_atoms = text
            .chars()
            .map(|character| {
                let start = cursor;
                cursor += character.len_utf16();
                (u32::from(character), Range::new(start, cursor))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            output
                .inner
                .projection
                .atoms
                .iter()
                .map(|atom| (atom.code_point, atom.source))
                .collect::<Vec<_>>(),
            expected_atoms
        );
        // An upright L unit in the actual RTL paragraph receives level2.
        // These remain the actual authored atoms, not an inserted U+FFFC.
        assert_eq!(output.inner.analysis.levels, vec![2; expected_atoms.len()]);
        assert_eq!(output.combined_cells().len(), 1);
        let cell = output.combined_cells().next().unwrap();
        assert_eq!(cell.scope(), processed.captured_scope(0).unwrap());
        assert_eq!(cell.parent_scope(), None);
        assert_eq!(cell.mode(), TextCombineUpright::All);
        assert_eq!(cell.inner_direction(), inner);
        assert_eq!(
            cell.used_code_points().collect::<Vec<_>>(),
            expected_atoms
                .iter()
                .map(|&(point, _)| point)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            cell.inner_bidi_levels()
                .map(BidiLevel::get)
                .collect::<Vec<_>>(),
            expected_inner
        );
        let pieces = cell.source_pieces().collect::<Vec<_>>();
        assert_eq!(pieces.len(), expected_atoms.len());
        for (piece, &(_, range)) in pieces.iter().zip(&expected_atoms) {
            assert_eq!(piece.source(), processed.source());
            assert_eq!(piece.aggregate(), range);
            assert_eq!(piece.local(), range);
            assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
            assert!(piece.complete_text_node().is_none());
            assert!(piece.is_used());
            assert_eq!(
                piece.origin_position(),
                processed.captured_origin(0).unwrap()
            );
            assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
        }
        assert_eq!(processed.source().text(), &Text::from(text));
    }
}

#[test]
fn latin_cell_keeps_inner_ltr_level_zero_inside_the_outer_rtl_level_two_unit() {
    qualify_root_cell("A", ParagraphBaseDirection::LeftToRight, &[0]);
}

#[test]
fn latin_cell_resolves_its_declared_inner_rtl_independently_of_outer_tcu_bidi() {
    qualify_root_cell("A", ParagraphBaseDirection::RightToLeft, &[2]);
}

#[test]
fn hebrew_cell_keeps_inner_rtl_levels_while_its_actual_outer_atoms_are_upright_l() {
    qualify_root_cell("אב", ParagraphBaseDirection::RightToLeft, &[1, 1]);
}

#[test]
fn alphabetic_cell_exteriors_keep_real_unicode_breaking_while_its_interior_is_indivisible() {
    let mut input = root_input(
        "xABx",
        TextCombineUpright::None,
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::LeftToRight,
    );
    let mut body = super::style(WhiteSpace::Preserve, 10.0);
    body.text_combine_upright = TextCombineUpright::All;
    input.source.span(Range::new(1, 3), body.clone());
    input.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: Some(body),
        boundaries: None,
        alignment_boundary: None,
        font_reference: None,
        horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
    });
    input.origins = [Range::new(0, 1), Range::new(1, 3), Range::new(3, 4)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    input.connections = (0..2)
        .map(|origin| SourceProcessingConnection {
            before_origin: origin,
            after_origin: origin + 1,
            kind: ParagraphConnectionKind::Continuous,
        })
        .collect();
    input.boundary_policies = (0..2)
        .map(|edge| super::policy(ParagraphBoundaryDefinition::Connection(edge)))
        .collect();
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::Connection(1),
        SourceProcessingEvent::Origin {
            origin: 2,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    let (processed, domain) = analyze(input, AnalysisTypography::HorizontalComposition);
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.inner.projection.text, "xABx");
    assert_eq!(
        output
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| (atom.code_point, atom.source))
            .collect::<Vec<_>>(),
        [
            (0x78, Range::new(0, 1)),
            (0x41, Range::new(1, 2)),
            (0x42, Range::new(2, 3)),
            (0x78, Range::new(3, 4))
        ]
    );
    assert_eq!(output.combined_cells().len(), 1);
    let cell = output.combined_cells().next().unwrap();
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(
        cell.parent_scope(),
        Some(processed.captured_scope(0).unwrap())
    );
    assert_eq!(cell.selection_unit_count(), 2);
    assert_eq!(cell.used_unit_count(), 2);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41, 0x42]);
    assert_eq!(
        cell.source_pieces()
            .map(|piece| (
                piece.aggregate(),
                piece.origin_position(),
                piece.paragraph()
            ))
            .collect::<Vec<_>>(),
        [
            (
                Range::new(1, 2),
                processed.captured_origin(1).unwrap(),
                processed.captured_paragraph(0).unwrap()
            ),
            (
                Range::new(2, 3),
                processed.captured_origin(1).unwrap(),
                processed.captured_paragraph(0).unwrap()
            )
        ]
    );
    let analysis = &output.inner.analysis;
    // UAX14 AL×AL keeps the actual x/A and B/x joins unbroken. A replacement
    // object's contingent-break behavior must not create exterior candidates.
    assert!(
        analysis
            .line_candidates
            .iter()
            .all(|candidate| !matches!(candidate.atom, 1 | 3))
    );
    // A permitted unit edge still needs real line permission; it is not itself
    // an opportunity. The cell's INTERNAL edge is forbidden independently.
    assert!(analysis.line_units.permits_edge(1));
    assert!(!analysis.line_units.permits_edge(2));
    assert!(analysis.line_units.permits_edge(3));
    assert_eq!(
        analysis
            .line_units
            .ranges(&analysis.graphemes)
            .collect::<Vec<_>>(),
        [0..1, 1..3, 3..4]
    );
    assert_eq!(processed.source().text(), &Text::from("xABx"));
}

#[test]
fn captured_rlo_inside_a_cell_sets_inner_bidi_without_becoming_a_payload_scalar() {
    use crate::engine::analysis::wrapping::Event;
    use crate::protocol::paragraph::SemanticAnchorItem;

    // UAX9 X4 gives RLO the next odd level1 in this LTR context and overrides
    // B to R; its I2 level remains1. A precedes RLO and retains level0. Each
    // mode has its OWN genuine capture; neither lends the other authority.
    for mode in [TextCombineUpright::None, TextCombineUpright::All] {
        let mut input = root_input(
            "AB",
            mode,
            ParagraphBaseDirection::LeftToRight,
            ParagraphBaseDirection::LeftToRight,
        );
        input.origins = [Range::new(0, 1), Range::new(1, 2)]
            .into_iter()
            .map(|range| super::origin(&input.source, range))
            .collect();
        input.connections = vec![SourceProcessingConnection {
            before_origin: 0,
            after_origin: 1,
            kind: ParagraphConnectionKind::Continuous,
        }];
        input.boundary_policies = vec![super::policy(ParagraphBoundaryDefinition::Connection(0))];
        input.controls = vec![
            ParagraphControl {
                at: 1,
                kind: ParagraphControlKind::RightToLeftOverride,
            },
            ParagraphControl {
                at: 2,
                kind: ParagraphControlKind::PopDirectionalFormat,
            },
        ];
        input.events = vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::Control {
                control: 0,
                affinity: Affinity::Before,
            },
            SourceProcessingEvent::Connection(0),
            SourceProcessingEvent::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::Control {
                control: 1,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::ExitScope(0),
        ];
        let (processed, domain) = analyze(input, AnalysisTypography::HorizontalComposition);
        let output = domain.outputs().next().unwrap();
        assert_eq!(output.inner.projection.text, "AB");
        assert_eq!(
            output
                .inner
                .projection
                .atoms
                .iter()
                .map(|atom| (atom.code_point, atom.source))
                .collect::<Vec<_>>(),
            [(0x41, Range::new(0, 1)), (0x42, Range::new(1, 2)),]
        );
        let rlo = processed.captured_control(0).unwrap();
        let pdf = processed.captured_control(1).unwrap();
        assert_eq!(rlo.kind(), &ParagraphControlKind::RightToLeftOverride);
        assert_eq!(pdf.kind(), &ParagraphControlKind::PopDirectionalFormat);
        assert_eq!(rlo.source_position().get(), 1);
        assert_eq!(pdf.source_position().get(), 2);
        assert!(processed.captured_origin(0).unwrap() < rlo.event_position());
        assert!(rlo.event_position() < processed.captured_origin(1).unwrap());
        assert!(processed.captured_origin(1).unwrap() < pdf.event_position());
        let paragraph = &output.inner.paragraph;
        let topology = paragraph.scope_topology.as_ref().unwrap();
        for captured in [rlo, pdf] {
            assert_eq!(captured.scope(), processed.captured_scope(0).unwrap());
            assert_eq!(
                captured.paragraph(),
                processed.captured_paragraph(0).unwrap()
            );
            let local = topology
                .control_orders
                .iter()
                .position(|&order| order == captured.event_position().index)
                .unwrap();
            assert_eq!(&paragraph.controls[local].kind, captured.kind());
            assert_eq!(
                paragraph.controls[local].at,
                captured.source_position().get()
            );
            assert!(output.inner.logical.events.iter().any(|event| matches!(event.kind,
                Event::Anchor(anchor) if matches!(anchor.item, SemanticAnchorItem::Control(control) if control == local)
            )));
        }
        if mode == TextCombineUpright::None {
            assert_eq!(output.combined_cells().len(), 0);
            assert_eq!(output.inner.analysis.levels, [0, 1]);
            continue;
        }
        // Establish the admitted completed cell and exact source custody before
        // observing its inner levels. Captured controls influence its isolate;
        // they do not become additional authored/used body code points.
        assert_eq!(output.combined_cells().len(), 1);
        let cell = output.combined_cells().next().unwrap();
        assert_eq!(cell.scope(), processed.captured_scope(0).unwrap());
        assert_eq!(cell.mode(), TextCombineUpright::All);
        assert_eq!(cell.inner_direction(), ParagraphBaseDirection::LeftToRight);
        assert_eq!(cell.selection_unit_count(), 2);
        assert_eq!(cell.used_unit_count(), 2);
        assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41, 0x42]);
        assert_eq!(
            cell.presentation_code_points().collect::<Vec<_>>(),
            [0x41, 0x42]
        );
        assert_eq!(
            cell.source_pieces()
                .map(|piece| {
                    assert_eq!(piece.source(), processed.source());
                    assert!(piece.is_used());
                    assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
                    assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
                    (piece.local(), piece.aggregate(), piece.origin_position())
                })
                .collect::<Vec<_>>(),
            [
                (
                    Range::new(0, 1),
                    Range::new(0, 1),
                    processed.captured_origin(0).unwrap()
                ),
                (
                    Range::new(1, 2),
                    Range::new(1, 2),
                    processed.captured_origin(1).unwrap()
                ),
            ]
        );
        assert_eq!(cell.outer_bidi_level().get(), 0);
        assert_eq!(output.inner.analysis.levels, [0, 0]);
        assert_eq!(
            cell.inner_bidi_levels()
                .map(BidiLevel::get)
                .collect::<Vec<_>>(),
            [0, 1]
        );
        assert_eq!(processed.source().text(), &Text::from("AB"));
    }
}
