//! Independent qualification of Text3 §7.3 shaping and §1.4 unit custody.
//!
//! The retained fixtures author actual GSUB/GPOS operations. Expected glyphs,
//! advances and attachment offsets below follow those tables, not a comparison
//! with another preparation. These new-API cases have no historical RED claim.
use crate::*;
use skrifa::{GlyphId, MetadataProvider, raw::TableProvider};
use std::sync::Arc;

use super::capture::{captured_input, generous, reference, style};

fn input(
    raw: &str,
    owners: &[Option<usize>],
    boundaries: &[ScopeAlignmentBoundary],
) -> ContextualSourceInput {
    assert_eq!(raw.chars().count(), owners.len());
    assert!(raw.chars().all(|c| c.len_utf16() == 1));
    let mut value = captured_input(boundaries.len());
    value.scope_alignment_limits = Some(generous());
    value.source = Source::identified(raw, Id::from_u64(1_578_780), 31);
    value.origins = (0..owners.len())
        .map(|i| ParagraphOrigin {
            text_node: None,
            source: value.source.clone(),
            local: Range::new(i, i + 1),
            aggregate: Range::new(i, i + 1),
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    value.connections = (0..owners.len().saturating_sub(1))
        .map(|i| SourceProcessingConnection {
            before_origin: i,
            after_origin: i + 1,
            kind: ParagraphConnectionKind::Continuous,
        })
        .collect();
    value.boundary_policies = (0..value.connections.len())
        .map(|i| ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(i),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        })
        .collect();
    let SourceProcessingScope::Block {
        base_style,
        font_reference,
        ..
    } = &mut value.scopes[0]
    else {
        unreachable!()
    };
    *base_style = style(10.0, 10.0);
    *font_reference = Some(reference());
    for (i, boundary) in boundaries.iter().enumerate() {
        let SourceProcessingScope::Inline {
            style: used,
            alignment_boundary,
            font_reference,
            ..
        } = &mut value.scopes[i + 1]
        else {
            unreachable!()
        };
        *used = Some(style(10.0, 10.0));
        *alignment_boundary = Some(*boundary);
        *font_reference = Some(reference());
    }
    value.paragraphs[0].paragraph_strut = ParagraphStrut::AllocatedContributions;
    use SourceProcessingEvent as E;
    value.events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    let mut open = None;
    for (i, &owner) in owners.iter().enumerate() {
        if owner != open {
            if let Some(scope) = open {
                value.events.push(E::ExitScope(scope));
            }
            if let Some(scope) = owner {
                value.events.push(E::EnterScope(scope));
            }
            open = owner;
        }
        if i != 0 {
            value.events.push(E::Connection(i - 1));
        }
        value.events.push(E::Origin {
            origin: i,
            affinity: Affinity::After,
        });
    }
    if let Some(scope) = open {
        value.events.push(E::ExitScope(scope));
    }
    value.events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    value
}

fn analysis(value: ContextualSourceInput) -> AnalyzedSource {
    let capture = Arc::new(process_contextual_source(value).unwrap());
    let analyzed = AnalyzedSource::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    crate::test_wrapping::bind_missing(&analyzed, Wrap::Word)
}

fn system(bytes: &[u8]) -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    system
}

fn prepare(bytes: &[u8], value: ContextualSourceInput) -> PreparedText {
    system(bytes)
        .prepare(&analysis(value), PreparationOptions::default())
        .unwrap()
}

fn select(prepared: &PreparedText, rtl: bool) -> SelectedLine {
    let mut request = super::super::request(prepared);
    request.context.available_inline = None;
    request.context.spacing = SpacingPolicy::None;
    request.context.alignment = Alignment::Start;
    if rtl {
        request.context.base_direction = ParagraphBaseDirection::RightToLeft;
        request.context.line_box_direction = LineBoxDirection::RightToLeft;
    }
    prepared.select(request).unwrap()
}

fn allocations(selected: &SelectedLine, shifts: &[f64]) -> LineAllocations {
    let scopes: Vec<_> = selected.scopes().collect();
    assert_eq!(
        scopes.len(),
        shifts.len(),
        "actual selected target inventory"
    );
    LineAllocations {
        spacing: SpacingAllocations::None,
        scopes: ScopeAllocations::Complete(
            scopes
                .into_iter()
                .zip(shifts)
                .map(|(scope, &shift)| ScopeAllocation {
                    scope,
                    displacement: BaselineShift::try_new(shift).unwrap(),
                })
                .collect(),
        ),
    }
}

fn reason(error: &Error) -> ScopeAlignmentError {
    let Some(ErrorDetail::ScopeAlignment { reason }) = error.detail() else {
        panic!("scope refusal required: {error:?}")
    };
    *reason
}

fn glyphs(line: &FinalizedLine, font: &[u8]) -> Vec<(Range, u32, f32, f32, f32)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), font);
            assert_eq!(run.font_size(), 10.0);
            assert!(
                run.effective_instance()
                    .unwrap()
                    .face()
                    .is_same_face(&run.face())
            );
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        glyph.x(),
                        glyph.y(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn place(prepared: &PreparedText, line: FinalizedLine) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 3.25,
                baseline: 20.75,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap()
}

fn ahem_scale(font: &[u8]) {
    let face = skrifa::FontRef::new(font).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(
        (
            face.hhea().unwrap().ascender().to_i16(),
            face.hhea().unwrap().descender().to_i16()
        ),
        (800, -200)
    );
    // Inspect original font data independently of the provider's output. Every
    // base/ligature/Arabic form used here has a real 1000-unit horizontal metric.
    let horizontal = face.hmtx().unwrap();
    for glyph in [35, 58, 82, 100, 277] {
        assert_eq!(horizontal.h_metrics()[glyph].advance.get(), 1000);
    }
}

fn ahem_facts(font: &[u8]) {
    ahem_scale(font);
    let face = skrifa::FontRef::new(font).unwrap();
    assert_eq!(face.charmap().map('A'), Some(GlyphId::new(35)));
}

#[test]
fn initial_continuity_keeps_one_real_ligature_with_shared_equal_frames() {
    let font = crate::shaping_test_fonts::ligature_ahem();
    ahem_facts(&font);
    // The authored ccmp LigatureSubst is [35,35] -> 58. At size10 its
    // single 1000-unit advance is10, rather than the20 of two input glyphs.
    let prepared = prepare(
        &font,
        input(
            "AA",
            &[Some(1), Some(2)],
            &[ScopeAlignmentBoundary::Initial; 2],
        ),
    );
    let selected = select(&prepared, false);
    let scopes: Vec<_> = selected.scopes().collect();
    assert_eq!(
        scopes
            .iter()
            .map(|s| s.captured_scope().capture_ordinal())
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert!(scopes[0].shares_operands_with(&scopes[1]).unwrap());
    let natural = selected
        .finalize(allocations(&selected, &[0.0, 0.0]))
        .unwrap();
    let expected = vec![(Range::new(0, 2), 58, 0.0, 0.0, 10.0)];
    assert_eq!(glyphs(&natural, &font), expected);
    assert_eq!(natural.feedback().advance, 10.0);
    let accepted = place(&prepared, natural.clone());
    assert_eq!(
        accepted.run_views().next().unwrap().origin(),
        Point::new(3.25, 20.75)
    );

    let shifted = selected
        .finalize(allocations(&selected, &[3.0, 3.0]))
        .unwrap();
    assert_eq!(glyphs(&shifted, &font), expected);
    assert_eq!(shifted.feedback().advance, 10.0);
    let moved = place(&prepared, shifted);
    assert_eq!(
        moved.run_views().next().unwrap().origin(),
        Point::new(3.25, 17.75)
    );
    assert_eq!(
        reason(
            &selected
                .finalize(allocations(&selected, &[3.0, 0.0]))
                .unwrap_err()
        ),
        ScopeAlignmentError::IncompatibleFrames {
            first: 1,
            second: 2
        }
    );
    // A failed replacement cannot change already accepted native output.
    assert_eq!(glyphs(&natural, &font), expected);
    assert_eq!(
        accepted.run_views().next().unwrap().origin(),
        Point::new(3.25, 20.75)
    );
}

#[test]
fn noninitial_is_a_mandatory_shaping_break_even_for_zero_displacement() {
    let font = crate::shaping_test_fonts::ligature_ahem();
    let prepared = prepare(
        &font,
        input(
            "AA",
            &[Some(1), Some(2)],
            &[ScopeAlignmentBoundary::NonInitial; 2],
        ),
    );
    let selected = select(&prepared, false);
    let scopes: Vec<_> = selected.scopes().collect();
    assert_eq!(scopes.len(), 2);
    assert!(!scopes[0].shares_operands_with(&scopes[1]).unwrap());
    let expected = vec![
        (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
        (Range::new(1, 2), 35, 0.0, 0.0, 10.0),
    ];
    for shifts in [[0.0, 0.0], [2.0, -3.0]] {
        let line = selected.finalize(allocations(&selected, &shifts)).unwrap();
        assert_eq!(glyphs(&line, &font), expected);
        assert_eq!(line.feedback().advance, 20.0);
        let origins: Vec<_> = place(&prepared, line)
            .run_views()
            .map(|run| run.origin())
            .collect();
        assert_eq!(
            origins,
            [
                Point::new(3.25, 20.75 - shifts[0]),
                Point::new(13.25, 20.75 - shifts[1])
            ]
        );
    }
}

#[test]
fn initial_decoration_change_does_not_become_a_font_shaping_barrier() {
    let font = crate::shaping_test_fonts::ligature_ahem();
    let mut value = input(
        "AA",
        &[Some(1), Some(2)],
        &[ScopeAlignmentBoundary::Initial; 2],
    );
    let SourceProcessingScope::Inline { style: used, .. } = &mut value.scopes[1] else {
        unreachable!()
    };
    used.as_mut().unwrap().underline = Decoration::solid()
        .with_offset(DecorationOffset::try_absolute(-2.0).unwrap())
        .with_thickness(DecorationThickness::try_absolute(1.0).unwrap());
    let prepared = prepare(&font, value);
    let selected = select(&prepared, false);
    assert!(
        selected
            .scopes()
            .next()
            .unwrap()
            .shares_operands_with(&selected.scopes().nth(1).unwrap())
            .unwrap()
    );
    let line = selected
        .finalize(allocations(&selected, &[0.0, 0.0]))
        .unwrap();
    assert_eq!(
        glyphs(&line, &font),
        [(Range::new(0, 2), 58, 0.0, 0.0, 10.0)]
    );
    assert_eq!(line.feedback().advance, 10.0);
}

#[test]
fn mandatory_breaks_preserve_actual_arabic_joining_context() {
    let font = crate::shaping_test_fonts::arabic_ahem();
    ahem_scale(&font);
    // The actual arab font maps Beh to35 and init/medi/fina to100/277/82.
    // Independent reshaping as isolated characters would incorrectly give35
    // three times. Both preceding and following real source context matter.
    let face = skrifa::FontRef::new(&font).unwrap();
    assert_eq!(face.charmap().map('\u{628}'), Some(GlyphId::new(35)));
    for boundary in [
        ScopeAlignmentBoundary::Initial,
        ScopeAlignmentBoundary::NonInitial,
    ] {
        let mut value = input(
            "\u{628}\u{628}\u{628}",
            &[Some(1), Some(2), Some(3)],
            &[boundary; 3],
        );
        let arabic_reference = ScopeFontReference {
            script: FontBaselineScript::from_iso15924(*b"Arab").unwrap(),
            ..reference()
        };
        let SourceProcessingScope::Block {
            used_direction,
            font_reference,
            ..
        } = &mut value.scopes[0]
        else {
            unreachable!()
        };
        *used_direction = ParagraphBaseDirection::RightToLeft;
        *font_reference = Some(arabic_reference);
        for scope in &mut value.scopes[1..] {
            let SourceProcessingScope::Inline { font_reference, .. } = scope else {
                unreachable!()
            };
            *font_reference = Some(arabic_reference);
        }
        value.paragraphs[0].base_direction = ParagraphBaseDirection::RightToLeft;
        let prepared = prepare(&font, value);
        let selected = select(&prepared, true);
        let line = selected
            .finalize(allocations(&selected, &[0.0; 3]))
            .unwrap();
        let mut actual = glyphs(&line, &font);
        actual.sort_by_key(|row| row.0.start);
        assert_eq!(
            actual,
            [
                (Range::new(0, 1), 100, 0.0, 0.0, 10.0),
                (Range::new(1, 2), 277, 0.0, 0.0, 10.0),
                (Range::new(2, 3), 82, 0.0, 0.0, 10.0),
            ],
            "actual formatting class {boundary:?}"
        );
        assert_eq!(line.feedback().advance, 30.0);
    }
}

#[test]
fn initial_mark_attachment_is_shared_and_cannot_receive_conflicting_frames() {
    let font = crate::shaping_test_fonts::mark_ahem();
    ahem_facts(&font);
    let face = skrifa::FontRef::new(&font).unwrap();
    assert_eq!(face.charmap().map('\u{300}'), Some(GlyphId::new(100)));
    assert_eq!(
        face.charmap().map('\u{c0}'),
        None,
        "no precomposed fallback may hide the actual MarkBase operation"
    );
    let prepared = prepare(
        &font,
        input(
            "A\u{300}",
            &[Some(1), Some(2)],
            &[ScopeAlignmentBoundary::Initial; 2],
        ),
    );
    let selected = select(&prepared, false);
    let scopes: Vec<_> = selected.scopes().collect();
    assert_eq!(scopes.len(), 2);
    assert!(scopes[0].shares_operands_with(&scopes[1]).unwrap());
    // GDEF marks glyph100; MarkBase base(400,600)-mark(100,200)
    // gives design offset(300,400). At size10 the attached physical point
    // is(3,-4). After the base's10-unit pen, raw mark offset is(-7,-4).
    // Monotone grapheme custody associates both glyphs with original0..2.
    let expected = vec![
        (Range::new(0, 2), 35, 0.0, 0.0, 10.0),
        (Range::new(0, 2), 100, -7.0, -4.0, 0.0),
    ];
    let accepted = selected
        .finalize(allocations(&selected, &[0.0, 0.0]))
        .unwrap();
    assert_eq!(glyphs(&accepted, &font), expected);
    assert_eq!(accepted.feedback().advance, 10.0);
    let shifted = selected
        .finalize(allocations(&selected, &[3.0, 3.0]))
        .unwrap();
    assert_eq!(glyphs(&shifted, &font), expected);
    let runs = place(&prepared, shifted);
    assert!(
        runs.run_views()
            .all(|run| run.origin() == Point::new(3.25, 17.75))
    );
    assert_eq!(
        reason(
            &selected
                .finalize(allocations(&selected, &[3.0, 0.0]))
                .unwrap_err()
        ),
        ScopeAlignmentError::IncompatibleFrames {
            first: 1,
            second: 2
        }
    );
    assert_eq!(glyphs(&accepted, &font), expected);
}

#[test]
fn demanded_noninitial_boundary_inside_one_extended_grapheme_refuses_precisely() {
    let font = crate::shaping_test_fonts::mark_ahem();
    let value = input(
        "A\u{300}",
        &[Some(1), Some(2)],
        &[
            ScopeAlignmentBoundary::Initial,
            ScopeAlignmentBoundary::NonInitial,
        ],
    );
    let result = (|| -> Result<PreparedText> {
        let capture = Arc::new(process_contextual_source(value)?);
        let analyzed = AnalyzedSource::from_processed_paragraph(
            capture.paragraphs().next().unwrap(),
            AnalysisTypography::HorizontalComposition,
        )?;
        let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
        system(&font).prepare(&analyzed, PreparationOptions::default())
    })();
    // Text3 §1.4 leaves split-unit rendering undefined. The adopted native
    // contract refuses this independently demanded split, before attempting
    // to give one indivisible A+grave unit two independent shaping frames.
    assert_eq!(
        reason(&result.unwrap_err()),
        ScopeAlignmentError::UnpartitionableUnit { scope: 2 }
    );
}

#[test]
fn shared_ligature_with_unscoped_source_cannot_move_the_unscoped_participant() {
    let font = crate::shaping_test_fonts::ligature_ahem();
    let prepared = prepare(
        &font,
        input("AA", &[Some(1), None], &[ScopeAlignmentBoundary::Initial]),
    );
    let selected = select(&prepared, false);
    let scopes: Vec<_> = selected.scopes().collect();
    assert_eq!(scopes.len(), 1);
    assert_eq!(scopes[0].captured_scope().capture_ordinal(), 1);
    let accepted = selected.finalize(allocations(&selected, &[0.0])).unwrap();
    let expected = vec![(Range::new(0, 2), 58, 0.0, 0.0, 10.0)];
    assert_eq!(glyphs(&accepted, &font), expected);
    let scene = place(&prepared, accepted.clone());
    // There is no fabricated SelectedScope for the domain's unscoped A.
    // Moving only the real target cannot move their jointly owned glyph.
    assert_eq!(
        reason(
            &selected
                .finalize(allocations(&selected, &[3.0]))
                .unwrap_err()
        ),
        ScopeAlignmentError::UnrepresentedOwnership { scope: 1 }
    );
    assert_eq!(glyphs(&accepted, &font), expected);
    assert_eq!(
        scene.run_views().next().unwrap().origin(),
        Point::new(3.25, 20.75)
    );
}
