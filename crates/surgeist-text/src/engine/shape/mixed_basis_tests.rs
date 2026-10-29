//! Retained Mixed bases use real per-role providers only during selection.
//! Registered by Root as a child of selected_basis_tests after producer freeze.
use super::*;
use crate::{GlyphBasis, mixed_test_fonts as mixed_fonts};

fn prepare_mixed(system: &mut System, source: &Source) -> PreparedText {
    let style = Style {
        line_height: crate::LineHeight::Absolute(12.0),
        ..own(0.0)
    };
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input(source, style, LTR), AnalysisTypography::Mixed)
            .unwrap(),
        Wrap::Word,
    );
    system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap()
}

fn roles(line: &FinalizedLine) -> Vec<(Range, GlyphBasis, u32, f32)> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.range(),
                        run.glyph_basis(),
                        glyph.id(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn mixed_selected_basis_retains_axes_resources_and_raw_packets_without_replay_provider_calls() {
    let bytes = mixed_fonts::fixture(true);
    let source = Source::new("X一");
    let mut system = system(&bytes);
    let prepared = prepare_mixed(&mut system, &source);
    reset_provider_calls();
    let selected = prepared
        .select(request(&prepared, LTR, SpacingPolicy::InterCharacter))
        .unwrap();
    let calls = provider_calls();
    assert!(calls > 0);
    assert_eq!(selected.spacing_slots().len(), 1);
    let basis = &selected.inner.basis;
    let old_catalogue = prepared.font_catalogue_identity().clone();
    system.refresh_fonts().unwrap();
    assert_ne!(system.font_catalogue_identity(), &old_catalogue);
    drop(system);
    assert_calls(calls);
    let natural = selected.finalize(SpacingAllocations::None).unwrap();
    assert_calls(calls);
    assert_eq!(natural.feedback().advance, 22.0);
    assert_eq!(
        roles(&natural),
        [
            (Range::new(0, 1), GlyphBasis::Clockwise, 274, 10.0),
            (Range::new(1, 2), GlyphBasis::Unrotated, 82, 12.0),
        ]
    );
    let raw = packets(&natural, &bytes);
    let faces = natural
        .run_views()
        .map(|run| run.face().to_owned())
        .collect::<Vec<_>>();
    let repeated = selected.finalize(SpacingAllocations::None).unwrap();
    assert_calls(calls);
    let expanded = natural.with_spacing(amounts(&selected, &[2.0])).unwrap();
    assert_calls(calls);
    assert_eq!(expanded.feedback().advance, 24.0);
    assert_eq!(inline_paints(&expanded), [0.0, 12.0]);
    let replaced = expanded.with_spacing(amounts(&selected, &[1.0])).unwrap();
    assert_calls(calls);
    assert_eq!(replaced.feedback().advance, 23.0);
    let restored = replaced.with_spacing(SpacingAllocations::None).unwrap();
    assert_calls(calls);
    assert_eq!(restored.feedback().advance, 22.0);
    let error = restored
        .with_spacing(amounts(&selected, &[f64::from_bits(1)]))
        .unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::InvalidTextNumeric);
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric {
            field: "character cell advance",
            phase: crate::TextNumericPhase::Finalization,
            category: crate::TextNumericError::NonZeroLost,
            ..
        })
    ));
    assert_calls(calls);
    let retry = restored.with_spacing(amounts(&selected, &[3.0])).unwrap();
    assert_calls(calls);
    assert_eq!(retry.feedback().advance, 25.0);
    for line in [&natural, &repeated, &expanded, &replaced, &restored, &retry] {
        assert_eq!(line.selected_line(), selected);
        assert!(std::ptr::eq(&line.inner.selected.basis, basis));
        assert_eq!(packets(line, &bytes), raw);
        assert_eq!(roles(line), roles(&natural));
        assert_eq!(
            line.prepared_text().font_catalogue_identity(),
            &old_catalogue
        );
        for (run, face) in line.run_views().zip(&faces) {
            assert!(run.face().is_same_face(&face.face()));
        }
        assert_eq!(line.prepared_text().analyzed_source().source(), &source);
    }
    assert_eq!(natural.feedback().advance, 22.0);
}

#[test]
fn mixed_native_di_receipts_preserve_normal_clockwise_output_and_removed_source_roles() {
    let vo = icu_properties::CodePointMapData::<icu_properties::props::VerticalOrientation>::new();
    assert_eq!(
        vo.get('一'),
        icu_properties::props::VerticalOrientation::Upright
    );
    assert_eq!(
        vo.get('\u{2064}'),
        icu_properties::props::VerticalOrientation::Rotated
    );
    // Assigned U2064 is genuine R/Cf/BN; reserved U2065 has VO=U.
    assert!(
        icu_properties::CodePointSetData::new::<icu_properties::props::DefaultIgnorableCodePoint>()
            .contains('\u{2064}')
    );
    assert_eq!(
        icu_properties::CodePointMapData::<icu_properties::props::GeneralCategory>::new()
            .get('\u{2064}'),
        icu_properties::props::GeneralCategory::Format
    );
    assert_eq!(
        icu_properties::CodePointMapData::<icu_properties::props::GraphemeClusterBreak>::new()
            .get('\u{2064}'),
        icu_properties::props::GraphemeClusterBreak::Control
    );
    assert_eq!(
        icu_properties::CodePointMapData::<icu_properties::props::BidiClass>::new().get('\u{2064}'),
        icu_properties::props::BidiClass::BoundaryNeutral
    );
    for (action, exclusions, normal) in [
        (font::Action::None, vec![(1, false)], false),
        (font::Action::Delete, vec![(1, true)], false),
        (font::Action::PartialDelete, vec![], true),
    ] {
        let base = font::fixture(Some('\u{2064}'), action);
        resource_premises(&base);
        let bytes = mixed_fonts::with_cmap(
            &base,
            &[
                (0x20, 274),
                (0x41, 35),
                (0x42, 36),
                (0x43, 100),
                (0x51, 277),
                (0x52, 58),
                (0x2064, 100),
                (0x4e00, 35),
            ],
        );
        let bytes = mixed_fonts::with_metrics(&bytes, true);
        // The DI invocation inherits Han's script but is a separate horizontal
        // R body, with ordinary B as post-context. Substitution and the complete
        // DI receipt, not output width, prove Normal/residual/deleted custody.
        literal_control(&bytes, "B", &[(36, 2000)]);
        let face = surgeist_shaper::FontRef::new(&bytes).unwrap();
        let data = surgeist_shaper::ShaperData::new(&face);
        let shaper = data.shaper(&face).build();
        let mut input = surgeist_shaper::UnicodeBuffer::new();
        input.add('\u{2064}', 0);
        input.set_direction(surgeist_shaper::Direction::LeftToRight);
        input.set_script(surgeist_shaper::script::HAN);
        input.set_post_context("B");
        let output = shaper.shape(
            input,
            &[surgeist_shaper::Feature::new(
                surgeist_shaper::Tag::new(b"vrtr"),
                1,
                ..,
            )],
        );
        assert_eq!(
            output.shaping_status(),
            surgeist_shaper::ShapingStatus::Complete
        );
        let receipt = output.default_ignorable_receipt().unwrap();
        let residual = if matches!(action, font::Action::None) {
            vec![0]
        } else {
            vec![]
        };
        let deleted = if matches!(action, font::Action::Delete) {
            vec![0]
        } else {
            vec![]
        };
        assert_eq!(receipt.residual_input_positions(), residual);
        assert_eq!(receipt.deleted_input_positions(), deleted);
        let raw = output
            .glyph_infos()
            .iter()
            .zip(output.glyph_positions())
            .enumerate()
            .filter(|(ordinal, _)| {
                !receipt
                    .residual_glyph_positions()
                    .contains(&u32::try_from(*ordinal).unwrap())
            })
            .map(|(_, (info, pos))| (info.glyph_id, pos.x_advance))
            .collect::<Vec<_>>();
        assert_eq!(raw, if normal { vec![(58, 1500)] } else { vec![] });
        let source = Source::new("一\u{2064}B");
        let mut system = system(&bytes);
        let prepared = prepare_mixed(&mut system, &source);
        assert_eq!(
            prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
        );
        reset_provider_calls();
        let selected = prepared
            .select(request(&prepared, LTR, SpacingPolicy::None))
            .unwrap();
        let calls = provider_calls();
        assert!(calls > 0);
        assert_eq!(exclusion_rows(&selected.inner.basis.shaped), exclusions);
        let di_run = selected
            .inner
            .basis
            .shaped
            .runs
            .iter()
            .find(|run| run.atoms.contains(&1))
            .unwrap();
        assert_eq!(di_run.atoms, 1..2);
        assert_eq!(di_run.glyph_basis, GlyphBasis::Clockwise);
        let natural = selected.finalize(SpacingAllocations::None).unwrap();
        assert_calls(calls);
        let mut expected = vec![(Range::new(0, 1), GlyphBasis::Unrotated, 35, 12.0)];
        if normal {
            expected.push((Range::new(1, 2), GlyphBasis::Clockwise, 58, 15.0));
        }
        expected.push((Range::new(2, 3), GlyphBasis::Clockwise, 36, 20.0));
        assert_eq!(roles(&natural), expected);
        assert_eq!(natural.feedback().advance, if normal { 47.0 } else { 32.0 });
        let repeated = selected.finalize(SpacingAllocations::None).unwrap();
        assert_calls(calls);
        let restored = natural.with_spacing(SpacingAllocations::None).unwrap();
        assert_calls(calls);
        assert_eq!(roles(&repeated), expected);
        assert_eq!(roles(&restored), expected);
        assert_eq!(packets(&restored, &bytes), packets(&natural, &bytes));
        assert_eq!(prepared.analyzed_source().source(), &source);
        let placed = crate::Layout::from_finalized_lines(crate::PlacementRequest {
            prepared: prepared.clone(),
            coverage: crate::PlacementCoverage::WholeParagraph,
            entries: vec![crate::PlacementEntry::Formatted(crate::LinePlacement {
                line: restored,
                anchor: crate::LineAnchor {
                    inline_origin: 20.0,
                    baseline: 100.0,
                },
                orientation: LineOrientation::Vertical,
                atomics: vec![],
            })],
        })
        .unwrap();
        let positions = placed.positions().collect::<Vec<_>>();
        let selection = placed
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap();
        assert_eq!(selection.source_text(), source.text());
        assert_eq!(selection.plaintext(), source.text());
        assert_calls(calls);
    }
}
