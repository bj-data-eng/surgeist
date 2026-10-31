//! Public placement must qualify the actual rounded Cell flow endpoints.
//! Reassociating a fixed square's endpoint with its authored gap changes f64
//! meaning even when every genuine local and physical operand is admitted.

use super::*;

fn selected_at_indent(prepared: &PreparedText, indent: f64) -> SelectedLine {
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    prepared
        .select(LineSelectionRequest {
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants: prepared.participants().collect(),
            termination: PreparedTermination::Ordinary(segment.end_opportunity()),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: LineContext {
                alignment_insets: LineAlignmentInsets::ZERO,
                formatted_line: FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: LineBoxDirection::LeftToRight,
                orientation: LineOrientation::Vertical,
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                indent,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::None,
            },
        })
        .expect("the genuine Cell plus ordinary sibling has an admitted complete selection")
}

const C: f64 = 4_503_599_627_370_496.0;

fn finalized_endpoint_witness(
    size: f32,
    tracking: f64,
) -> (Fixture, PreparedText, FinalizedLine, f64, f64) {
    // The actual captured Inline owns AA at the requested real size/tracking.
    // Only following B belongs to RootInline at the same size/LS0. WS is0.
    let (mut system, fixture) = fixture("AA", tracking, false, true, size);
    assert_eq!(fixture.processed.source().text(), &Text::from("AAB"));
    assert_eq!(fixture.inner, Range::new(0, 2));
    let scope = fixture.processed.captured_scope(1).unwrap();
    assert_eq!(scope.style().unwrap().size, size);
    assert_eq!(scope.style().unwrap().letter_spacing, tracking);
    assert_eq!(scope.style().unwrap().word_spacing, 0.0);
    let prepared = system
        .prepare(
            &fixture.analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .expect("real native font/source preparation succeeds before the placement under test");
    assert_eq!(
        prepared.analyzed_source().identity(),
        fixture.analyzed.identity()
    );
    let prepared_cells = prepared.cell_views().collect::<Vec<_>>();
    let [prepared_cell] = prepared_cells.as_slice() else {
        panic!("one genuine source Cell");
    };
    assert_eq!(prepared_cell.extent(), f64::from(size));
    assert_eq!(
        (prepared_cell.before(), prepared_cell.after()),
        (f64::from(size) / 2.0, f64::from(size) / 2.0)
    );
    assert!(prepared_cell.has_provider_input());
    assert_eq!(prepared_cell.source_cell().scope(), scope);
    assert_eq!(prepared_cell.source_cell().selection_unit_count(), 2);
    assert_eq!(
        prepared_cell
            .source_cell()
            .used_code_points()
            .collect::<Vec<_>>(),
        [0x41, 0x41]
    );
    let metrics = prepared.scope_metrics(&scope).unwrap();
    assert_eq!(metrics.font_size(), size);
    assert!(
        metrics
            .effective_instance()
            .unwrap()
            .face()
            .is_same_face(&fixture.body.face())
    );
    let selected = selected_at_indent(&prepared, C);
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .expect("actual local square and complete flow endpoints are representable");
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual finalized Cell");
    };
    assert_eq!(cell.source_cell().scope(), scope);
    assert_eq!(cell.square().origin.x, C);
    assert_eq!(cell.square().size.width, 2.0);
    let actual_before = cell.square().origin.x + cell.square().size.width;
    assert_eq!(actual_before, C + 2.0);
    assert_eq!(cell.scope_displacement().get(), 0.0);
    let mut inner = Vec::new();
    let mut ordinary = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.effective_instance().unwrap().font_size(), size);
        if run.cell().is_some() {
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            assert!(run.face().is_same_face(&fixture.body.face()));
            for glyph in run.glyphs() {
                inner.push((glyph.range(), glyph.id(), glyph.advance()));
            }
        } else {
            assert!(run.face().is_same_face(&fixture.parent.face()));
            for glyph in run.glyphs() {
                assert_eq!(
                    (glyph.range(), glyph.id(), glyph.advance()),
                    (Range::new(2, 3), 36, size)
                );
                ordinary.push(run.origin().x + glyph.pen());
            }
        }
    }
    // AA normal2E is overwide; complete hwid isE (hmtx500/1000 per A).
    // The actual source packet remains real while outer flow ends at C+3.
    assert_eq!(
        inner,
        [
            (Range::new(0, 1), 36, size / 2.0),
            (Range::new(1, 2), 36, size / 2.0)
        ]
    );
    assert_eq!(ordinary, [C + 3.0]);
    let actual_after = ordinary[0];
    assert_ne!(actual_before, actual_after);
    drop(system);
    (fixture, prepared, line, actual_before, actual_after)
}

#[test]
fn actual_rounded_cell_gap_survives_placement_without_reassociating_its_endpoints() {
    // C=2^52 has ULP1. E2.5 and the boundary mean(1+0)/2=.5 give
    // canonical complete flow advance3, before=C+2 and actual after=C+3.
    // Reconstructing (C+2)+.5 instead rounds back to the even endpoint C+2.
    const E: f64 = 2.5;
    const GAP: f64 = 0.5;
    assert_eq!(C + E, C + 2.0);
    assert_eq!((C + 2.0) + GAP, C + 2.0);
    assert_eq!(C + (E + GAP), C + 3.0);
    assert_ne!(C + 2.0, C + 3.0);
    let (fixture, prepared, line, actual_before, actual_after) =
        finalized_endpoint_witness(2.5, 1.0);
    // Inline anchor0 preserves these exact executed endpoints. Placement must
    // not replace actual_after with the different before+gap result.
    assert_eq!(0.0 + actual_before, C + 2.0);
    assert_eq!(0.0 + actual_after, C + 3.0);
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .expect(
        "anchor0 preserves actual Cell endpoints C+2/C+3; the .5 displacement must remain usable",
    );
    assert_eq!(layout.source(), fixture.processed.source());
    assert_eq!(
        layout
            .cursor(Cursor::new(2, Affinity::Before))
            .unwrap()
            .rect()
            .origin
            .y,
        C + 3.0
    );
}

#[test]
fn actual_cell_gap_lost_at_physical_anchor_is_refused_without_reassociating_its_endpoints() {
    // The same actual C+2/C+3 endpoints now come from E1.5 and mean(3+0)/2=1.5.
    // Adding physical anchor1.5 collapses both to C+4 (ties to even). The
    // reconstructed local (C+2)+1.5=C+4 would instead place at C+6, hiding loss.
    const E: f64 = 1.5;
    const GAP: f64 = 1.5;
    const ANCHOR: f64 = 1.5;
    assert_eq!(C + E, C + 2.0);
    assert_eq!(C + (E + GAP), C + 3.0);
    assert_eq!((C + 2.0) + GAP, C + 4.0);
    assert_eq!(ANCHOR + (C + 4.0), C + 6.0);
    let (_fixture, prepared, line, actual_before, actual_after) =
        finalized_endpoint_witness(1.5, 3.0);
    assert_eq!(ANCHOR + actual_before, C + 4.0);
    assert_eq!(ANCHOR + actual_after, C + 4.0);
    let error = Layout::from_finalized_lines(PlacementRequest {
        prepared,
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: ANCHOR,
                baseline: 100.0,
            },
            orientation: LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .expect_err("the actual nonzero1.5 spacing displacement disappears at anchor1.5");
    assert_eq!(error.code, ErrorCode::InvalidTextNumeric);
    assert!(
        matches!(error.detail(), Some(ErrorDetail::InvalidTextNumeric {
        field: "placed spacing displacement", phase: TextNumericPhase::Placement,
        category: TextNumericError::NonZeroLost, value,
    }) if *value == GAP),
        "unexpected refusal: {error:?}"
    );
}
