//! Completed source selection and used-payload qualification. Literal oracles
//! follow the adopted issue-784 SPACE/count/width policy, independently of fonts,
//! provider visibility or the implementation's intermediate plan representation.

use crate::processing::DeletionCause;
use crate::*;
use std::sync::Arc;

fn root_input(
    text: &str,
    whitespace: WhiteSpace,
    mode: TextCombineUpright,
    transform: TextTransform,
) -> ContextualSourceInput {
    let mut input = super::input(text, whitespace);
    input.source = Source::identified(text, Id::from_u64(78475), 16);
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
        unreachable!("genuine root Block")
    };
    *topology = SourceProcessingTopology::Complete;
    *writing_axis = Some(SourceWritingAxis::Vertical);
    *horizontal_inner_direction = Some(ParagraphBaseDirection::LeftToRight);
    base_style.text_combine_upright = mode;
    base_style.text_transform = transform;
    input
}

fn boxed_input(
    before: &str,
    inside: &str,
    after: &str,
    root_mode: TextCombineUpright,
    inner_whitespace: WhiteSpace,
    inner_transform: TextTransform,
) -> ContextualSourceInput {
    let text = format!("{before}{inside}{after}");
    let a = before.encode_utf16().count();
    let b = a + inside.encode_utf16().count();
    let end = b + after.encode_utf16().count();
    let mut input = root_input(
        &text,
        WhiteSpace::Collapse,
        root_mode,
        TextTransform::none(),
    );
    let mut inner_style = super::style(inner_whitespace, 10.0);
    inner_style.text_combine_upright = TextCombineUpright::All;
    inner_style.text_transform = inner_transform;
    if a != b {
        // The actual body winner carries its transform/whitespace; the captured
        // Inline Style supplies computed scope context, not automatic restyling.
        input.source.span(Range::new(a, b), inner_style.clone());
    }
    input.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: Some(inner_style),
        boundaries: None,
        alignment_boundary: None,
        font_reference: None,
        horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
    });
    let mut ranges = vec![Range::new(0, a), Range::new(a, b)];
    if !after.is_empty() {
        ranges.push(Range::new(b, end));
    }
    input.origins = ranges
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    input.connections = (0..input.origins.len() - 1)
        .map(|origin| SourceProcessingConnection {
            before_origin: origin,
            after_origin: origin + 1,
            kind: ParagraphConnectionKind::Continuous,
        })
        .collect();
    input.boundary_policies = (0..input.connections.len())
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
    ];
    if !after.is_empty() {
        input.events.extend([
            SourceProcessingEvent::Connection(1),
            SourceProcessingEvent::Origin {
                origin: 2,
                affinity: Affinity::After,
            },
        ]);
    }
    input.events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    input
}

fn analyzed(
    input: ContextualSourceInput,
    expected_processed: &str,
) -> (Arc<ProcessedSource>, AnalyzedDomain) {
    analyzed_with_plaintext(input, expected_processed, expected_processed)
}

fn analyzed_with_plaintext(
    input: ContextualSourceInput,
    expected_processed: &str,
    expected_plaintext: &str,
) -> (Arc<ProcessedSource>, AnalyzedDomain) {
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(expected_processed));
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        expected_plaintext.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(processed.paragraphs().len(), 1);
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(domain.outputs().len(), 1);
    let output = domain.outputs().next().unwrap();
    assert!(std::ptr::eq(output.processed_source(), processed.as_ref()));
    assert_eq!(output.source(), processed.source());
    let owner = output.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    assert!(owner.joint_flow().is_none());
    (processed, domain)
}

fn assert_cell(
    cell: AnalyzedCombinedCell<'_>,
    processed: &Arc<ProcessedSource>,
    scope: usize,
    mode: TextCombineUpright,
    selection: usize,
    used: usize,
    code_points: &[u32],
) {
    assert_eq!(cell.scope(), processed.captured_scope(scope).unwrap());
    assert_eq!(
        cell.parent_scope(),
        if scope == 0 {
            None
        } else {
            Some(processed.captured_scope(0).unwrap())
        }
    );
    assert_eq!(cell.mode(), mode);
    assert_eq!(cell.inner_direction(), ParagraphBaseDirection::LeftToRight);
    assert_eq!(cell.selection_unit_count(), selection);
    assert_eq!(cell.used_unit_count(), used);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), code_points);
    assert_eq!(
        cell.inner_bidi_levels()
            .map(BidiLevel::get)
            .collect::<Vec<_>>(),
        vec![0; code_points.len()]
    );
}

fn assert_pieces(
    cell: AnalyzedCombinedCell<'_>,
    processed: &Arc<ProcessedSource>,
    expected: &[(usize, Range, bool)],
) {
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), expected.len());
    for (piece, &(origin, range, used)) in pieces.iter().zip(expected) {
        assert_eq!(piece.source(), processed.source());
        assert_eq!(piece.local(), range);
        assert_eq!(piece.aggregate(), range);
        assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
        assert!(piece.complete_text_node().is_none());
        assert_eq!(piece.is_used(), used);
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(origin).unwrap()
        );
        assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
    }
}

fn assert_deletion(processed: &ProcessedSource, range: Range, cause: DeletionCause) {
    let link = processed.lexical.deletion(range, cause);
    assert_eq!(link.occurrences(&processed.lexical).count(), 1);
    assert_eq!(link.original_range(&processed.lexical), range);
}

#[test]
fn interior_collapsed_space_remains_a_semantic_used_unit_without_provider_emission() {
    let (processed, domain) = analyzed(
        root_input(
            "A  B",
            WhiteSpace::Collapse,
            TextCombineUpright::All,
            TextTransform::none(),
        ),
        "A B",
    );
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.combined_cells().len(), 1);
    let cell = output.combined_cells().next().unwrap();
    assert_cell(
        cell,
        &processed,
        0,
        TextCombineUpright::All,
        4,
        4,
        &[0x41, 0x20, 0x20, 0x42],
    );
    assert_pieces(
        cell,
        &processed,
        &[
            (0, Range::new(0, 1), true),
            (0, Range::new(1, 2), true),
            (0, Range::new(2, 3), true),
            (0, Range::new(3, 4), true),
        ],
    );
    assert_deletion(&processed, Range::new(2, 3), DeletionCause::CollapsedSpace);
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces[2].emission_subposition(), None);
    assert!(!pieces[2].has_presentation_emission());
    assert!(pieces[0].has_presentation_emission());
    assert!(pieces[1].has_presentation_emission());
    assert!(pieces[3].has_presentation_emission());
    assert_eq!(
        output
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| (atom.code_point, atom.source))
            .collect::<Vec<_>>(),
        [
            (0x41, Range::new(0, 1)),
            (0x20, Range::new(1, 2)),
            (0x42, Range::new(3, 4))
        ]
    );
    assert_eq!(processed.processed_text(), &Text::from("A B"));
}

#[test]
fn root_all_zero_space_lookaround_rejects_both_partial_groups_without_resurrection() {
    let (processed, domain) = analyzed(
        boxed_input(
            "A ",
            " ",
            "",
            TextCombineUpright::All,
            WhiteSpace::Collapse,
            TextTransform::none(),
        ),
        "A ",
    );
    assert_deletion(&processed, Range::new(2, 3), DeletionCause::CollapsedSpace);
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.combined_cells().len(), 0);
    assert_eq!(output.inner.projection.text, "A ");
    assert_eq!(
        output
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| (atom.code_point, atom.source))
            .collect::<Vec<_>>(),
        [(0x41, Range::new(0, 1)), (0x20, Range::new(1, 2))]
    );
    assert_eq!(processed.source().text(), &Text::from("A  "));
    assert_eq!(
        processed.captured_scope(1).unwrap().parent(),
        Some(processed.captured_scope(0).unwrap())
    );
}

#[test]
fn trimmed_fullwidth_single_unit_keeps_width_and_reactivates_its_own_exterior_space() {
    use crate::processing::lexical_inventory::{
        LexicalDisposition, LexicalRecordKind, LexicalState,
    };
    use crate::transform::relation::ComponentOwner;
    let fullwidth = TextTransform::Set(TextTransformSet::none().with_full_width(true));
    let (processed, domain) = analyzed(
        boxed_input(
            "a",
            " b ",
            " c",
            TextCombineUpright::None,
            WhiteSpace::Collapse,
            fullwidth,
        ),
        "a b c",
    );
    assert_deletion(&processed, Range::new(4, 5), DeletionCause::CollapsedSpace);
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.combined_cells().len(), 1);
    let cell = output.combined_cells().next().unwrap();
    // Selection owns inner original1..4. Edges remove1/3; the one remaining
    // used unit is FULLWIDTH SMALL LETTER B, so inverse-width must not run.
    assert_cell(
        cell,
        &processed,
        1,
        TextCombineUpright::All,
        3,
        1,
        &[0xff42],
    );
    assert_pieces(
        cell,
        &processed,
        &[
            (1, Range::new(1, 2), false),
            (1, Range::new(2, 3), true),
            (1, Range::new(3, 4), false),
        ],
    );
    assert_eq!(
        output
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| (atom.code_point, atom.source))
            .collect::<Vec<_>>(),
        [
            (0x61, Range::new(0, 1)),
            (0xff42, Range::new(2, 3)),
            (0x20, Range::new(4, 5)),
            (0x63, Range::new(5, 6))
        ]
    );
    let exterior_atom = output
        .inner
        .projection
        .atoms
        .iter()
        .position(|atom| atom.source == Range::new(4, 5))
        .unwrap();
    let ComponentOwner::Original(row) = output
        .inner
        .transform
        .component_for_atom(exterior_atom)
        .owner
    else {
        panic!("exterior SPACE retains an actual original, not a virtual glyph")
    };
    let LexicalState::Executed { records, .. } = &processed.lexical else {
        panic!("real executed capture")
    };
    let lexical = records.iter().position(|record| matches!(record.kind, LexicalRecordKind::Character {
        source, disposition: LexicalDisposition::Deleted { cause: DeletionCause::CollapsedSpace, .. }, ..
    } if source == Range::new(4, 5))).unwrap();
    let selection_part = output
        .inner
        .transform
        .emission_for_atom(exterior_atom)
        .selection_part
        .expect("reactivated exterior SPACE retains its selection occurrence");
    assert_eq!(
        output.inner.combination.as_ref().unwrap().plan.parts[selection_part].reference,
        crate::transform::combined::SelectionPart::CollapsedSpace(lexical)
    );
    let originals = output.inner.transform.original_pieces(row);
    assert_eq!(originals.len(), 1);
    assert_eq!(originals[0].aggregate, Range::new(4, 5));
    assert_eq!(originals[0].local, Range::new(4, 5));
    assert_eq!(
        originals[0].origin.captured_event(),
        Some(processed.captured_origin(2).unwrap().index)
    );
    assert_eq!(processed.processed_text(), &Text::from("a b c"));
}

#[test]
fn zero_only_all_activates_an_empty_used_cell_but_empty_and_discarded_inputs_do_not() {
    let (processed, domain) = analyzed(
        boxed_input(
            "a ",
            " ",
            "b",
            TextCombineUpright::None,
            WhiteSpace::Collapse,
            TextTransform::none(),
        ),
        "a b",
    );
    assert_deletion(&processed, Range::new(2, 3), DeletionCause::CollapsedSpace);
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.combined_cells().len(), 1);
    let cell = output.combined_cells().next().unwrap();
    assert_cell(cell, &processed, 1, TextCombineUpright::All, 1, 0, &[]);
    assert_pieces(cell, &processed, &[(1, Range::new(2, 3), false)]);
    let piece = cell.source_pieces().next().unwrap();
    assert_eq!(piece.emission_subposition(), None);
    assert!(!piece.has_presentation_emission());
    assert_eq!(processed.source().text(), &Text::from("a  b"));
    let (empty, empty_domain) = analyzed(
        boxed_input(
            "a ",
            "",
            "b",
            TextCombineUpright::None,
            WhiteSpace::Collapse,
            TextTransform::none(),
        ),
        "a b",
    );
    assert_eq!(
        empty_domain
            .outputs()
            .next()
            .unwrap()
            .combined_cells()
            .len(),
        0
    );
    assert_eq!(
        empty
            .paragraphs()
            .next()
            .unwrap()
            .source_pieces()
            .collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 2), Range::new(2, 3)]
    );
    let (discard, discard_domain) = analyzed(
        boxed_input(
            "a ",
            " ",
            "b",
            TextCombineUpright::None,
            WhiteSpace::Discard,
            TextTransform::none(),
        ),
        "a b",
    );
    assert_deletion(&discard, Range::new(2, 3), DeletionCause::Discard);
    assert_eq!(
        discard_domain
            .outputs()
            .next()
            .unwrap()
            .combined_cells()
            .len(),
        0
    );
    assert_eq!(discard.source().text(), &Text::from("a  b"));
}

#[test]
fn uppercase_expansion_retains_repeated_ancestry_and_uses_two_unit_inverse_width() {
    let transform = TextTransform::Set(
        TextTransformSet::none()
            .with_case(CaseTransform::Uppercase)
            .with_full_width(true),
    );
    let (processed, domain) = analyzed(
        root_input(
            "ß",
            WhiteSpace::Collapse,
            TextCombineUpright::All,
            transform,
        ),
        "ß",
    );
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.combined_cells().len(), 1);
    let cell = output.combined_cells().next().unwrap();
    // Canonical uppercase expands to SS, then fullwidth normal presentation.
    // Two actual used EGCs trigger normative inverse width back to ASCII SS.
    assert_cell(
        cell,
        &processed,
        0,
        TextCombineUpright::All,
        2,
        2,
        &[0x53, 0x53],
    );
    assert_pieces(
        cell,
        &processed,
        &[(0, Range::new(0, 1), true), (0, Range::new(0, 1), true)],
    );
    assert_eq!(
        cell.source_pieces()
            .map(|piece| piece.emission_subposition())
            .collect::<Vec<_>>(),
        [Some(0), Some(1)]
    );
    assert!(
        cell.source_pieces()
            .all(|piece| piece.has_presentation_emission())
    );
    assert_eq!(processed.source().text(), &Text::from("ß"));
}

#[test]
fn scalar_space_trim_keeps_its_attached_mark_and_resegments_the_surviving_payload() {
    let (processed, domain) = analyzed_with_plaintext(
        root_input(
            " \u{301}A",
            WhiteSpace::Collapse,
            TextCombineUpright::All,
            TextTransform::none(),
        ),
        " \u{301}A",
        "\u{301}A",
    );
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.combined_cells().len(), 1);
    let cell = output.combined_cells().next().unwrap();
    // Selection EGCs are SPACE+acute and A. Inner leading SPACE removal is a
    // scalar exclusion, not permission to erase the attached original mark.
    // The surviving leading acute and A are two freshly segmented used EGCs.
    assert_cell(
        cell,
        &processed,
        0,
        TextCombineUpright::All,
        2,
        2,
        &[0x301, 0x41],
    );
    assert_pieces(
        cell,
        &processed,
        &[
            (0, Range::new(0, 1), false),
            (0, Range::new(1, 2), true),
            (0, Range::new(2, 3), true),
        ],
    );
    assert_eq!(
        output
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| (atom.code_point, atom.source))
            .collect::<Vec<_>>(),
        [(0x301, Range::new(1, 2)), (0x41, Range::new(2, 3))]
    );
}

#[test]
fn marked_digit_cells_own_the_whole_egc_and_exclude_a_leading_standalone_mark() {
    let mode = TextCombineUpright::Digits(TextCombineUprightDigits::Two);
    for text in ["9\u{fe0f}\u{20e3}", "9\u{20e3}", "9\u{20dd}"] {
        let (processed, domain) = analyzed(
            root_input(text, WhiteSpace::Preserve, mode, TextTransform::none()),
            text,
        );
        let output = domain.outputs().next().unwrap();
        assert_eq!(output.combined_cells().len(), 1);
        let cell = output.combined_cells().next().unwrap();
        let code_points = text.chars().map(u32::from).collect::<Vec<_>>();
        assert_cell(cell, &processed, 0, mode, 1, 1, &code_points);
        let pieces = (0..text.encode_utf16().count())
            .map(|at| (0, Range::new(at, at + 1), true))
            .collect::<Vec<_>>();
        assert_pieces(cell, &processed, &pieces);
    }
    for text in ["#\u{fe0f}\u{20e3}", "*\u{fe0f}\u{20e3}"] {
        let (_, domain) = analyzed(
            root_input(text, WhiteSpace::Preserve, mode, TextTransform::none()),
            text,
        );
        assert_eq!(domain.outputs().next().unwrap().combined_cells().len(), 0);
    }
    let (processed, domain) = analyzed(
        root_input(
            "\u{301}23",
            WhiteSpace::Preserve,
            mode,
            TextTransform::none(),
        ),
        "\u{301}23",
    );
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.combined_cells().len(), 1);
    let cell = output.combined_cells().next().unwrap();
    assert_cell(cell, &processed, 0, mode, 2, 2, &[0x32, 0x33]);
    assert_pieces(
        cell,
        &processed,
        &[(0, Range::new(1, 2), true), (0, Range::new(2, 3), true)],
    );
    assert_eq!(output.inner.projection.atoms[0].code_point, 0x301);
    assert_eq!(output.inner.projection.atoms[0].source, Range::new(0, 1));
}

#[test]
fn inner_rtl_levels_describe_the_actual_inverse_width_used_payload() {
    let mut input = root_input(
        "\u{3099}\u{5d0}",
        WhiteSpace::Preserve,
        TextCombineUpright::All,
        TextTransform::none(),
    );
    let SourceProcessingScope::Block {
        horizontal_inner_direction,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!("genuine complete vertical All root")
    };
    *horizontal_inner_direction = Some(ParagraphBaseDirection::RightToLeft);
    let (processed, domain) = analyzed(input, "\u{3099}\u{5d0}");
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.combined_cells().len(), 1);
    let cell = output.combined_cells().next().unwrap();
    assert_eq!(cell.scope(), processed.captured_scope(0).unwrap());
    assert_eq!(cell.parent_scope(), None);
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.inner_direction(), ParagraphBaseDirection::RightToLeft);
    // A leading standalone mark and Alef are two genuine EGCs. The native
    // multiunit inverse-width rule maps original U+3099 to U+FF9E, not Alef.
    assert_eq!(cell.selection_unit_count(), 2);
    assert_eq!(cell.used_unit_count(), 2);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0xff9e, 0x5d0]);
    assert_eq!(
        cell.presentation_code_points().collect::<Vec<_>>(),
        [0xff9e, 0x5d0]
    );
    // Unicode bidi classes of the ACTUAL used payload are L,R. In its RTL
    // isolate, UAX9 I2 resolves these to2,1. Pre-width NSM,R would instead be
    // 1,1 after W1, which is not the sequence promised by this readback.
    assert_eq!(
        cell.inner_bidi_levels()
            .map(BidiLevel::get)
            .collect::<Vec<_>>(),
        [2, 1]
    );
    assert_pieces(
        cell,
        &processed,
        &[(0, Range::new(0, 1), true), (0, Range::new(1, 2), true)],
    );
    assert_eq!(
        output
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| (atom.code_point, atom.source))
            .collect::<Vec<_>>(),
        [(0xff9e, Range::new(0, 1)), (0x5d0, Range::new(1, 2)),]
    );
    assert_eq!(processed.source().text(), &Text::from("\u{3099}\u{5d0}"));
    assert_eq!(processed.processed_text(), &Text::from("\u{3099}\u{5d0}"));
}
