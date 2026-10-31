//! Functional qualification of demanded width-observation refusal on a real font.
//! Intended as a private child of combined_cell_widths; registration is external.
//! The public diagnostic is new, so these cases claim no historical RED.

use super::*;
use surgeist_shaper::{
    Direction, Feature, FontRef as ShaperFontRef, GlyphBuffer, ShaperData, ShapingStatus, Tag,
    UnicodeBuffer, WidthFeature, WidthInputOutcome, WidthObservationCoverage, WidthObservationPath,
    script,
};

fn rejected_hwid_font() -> Vec<u8> {
    let original = composition_font();
    let original_gsub = sfnt::table(&original, *b"GSUB");
    let mut gsub = original_gsub.to_vec();
    let features = usize::from(sfnt::u16_at(&gsub, 6));
    let lookups = usize::from(sfnt::u16_at(&gsub, 8));
    assert_eq!(sfnt::u16_at(&gsub, features), 3);
    assert_eq!(sfnt::u16_at(&gsub, lookups), 3);
    let row = (0..3)
        .map(|index| features + 2 + index * 6)
        .find(|row| &gsub[*row..*row + 4] == b"hwid")
        .unwrap();
    let feature = features + usize::from(sfnt::u16_at(&gsub, row + 4));
    assert_eq!(sfnt::u16_at(&gsub, feature + 2), 1);
    let lookup_index = usize::from(sfnt::u16_at(&gsub, feature + 4));
    let lookup = lookups + usize::from(sfnt::u16_at(&gsub, lookups + 2 + lookup_index * 2));
    assert_eq!(sfnt::u16_at(&gsub, lookup), 1);
    assert_eq!(sfnt::u16_at(&gsub, lookup + 4), 1);
    let subtable = lookup + usize::from(sfnt::u16_at(&gsub, lookup + 6));
    assert_eq!(sfnt::u16_at(&gsub, subtable), 2);
    assert_eq!(sfnt::u16_at(&gsub, subtable + 4), 1);
    assert_eq!(sfnt::u16_at(&gsub, subtable + 6), 36);
    let coverage = subtable + usize::from(sfnt::u16_at(&gsub, subtable + 2));
    assert_eq!(&gsub[coverage..coverage + 6], &[0, 1, 0, 1, 0, 35]);
    // The maintained Shaper qualification uses this same unsupported Coverage
    // format99 to distinguish rejected admission from valid empty Coverage.
    // Keep the real SingleSubst, replacement, count and glyph operands intact.
    gsub[coverage..coverage + 2].copy_from_slice(&99_u16.to_be_bytes());
    assert_eq!(&gsub[..coverage], &original_gsub[..coverage]);
    assert_eq!(&gsub[coverage + 2..], &original_gsub[coverage + 2..]);
    let body = sfnt::replace_tables(&original, vec![(*b"GSUB", gsub.clone())]);
    assert_eq!(sfnt::table(&body, *b"GSUB"), gsub);
    assert_eq!(sfnt::u16_at(&body, 4), sfnt::u16_at(&original, 4));
    for index in 0..usize::from(sfnt::u16_at(&original, 4)) {
        let record = 12 + index * 16;
        let tag: [u8; 4] = original[record..record + 4].try_into().unwrap();
        if tag == *b"GSUB" {
            continue;
        }
        let before = sfnt::table(&original, tag);
        let after = sfnt::table(&body, tag);
        if tag == *b"head" {
            // The existing SFNT assembler must recompute checkSumAdjustment.
            assert_eq!(&after[..8], &before[..8]);
            assert_eq!(&after[12..], &before[12..]);
        } else {
            assert_eq!(after, before);
        }
    }
    body
}

fn input(count: usize, observation: Option<WidthFeature>) -> UnicodeBuffer {
    let mut input = UnicodeBuffer::new();
    for ordinal in 0..count {
        input.add('A', u32::try_from(ordinal).unwrap());
    }
    input.set_direction(Direction::LeftToRight);
    input.set_script(script::LATIN);
    input.set_language("en".parse().unwrap());
    if let Some(feature) = observation {
        input.observe_width_feature(feature);
    }
    input
}

fn packet(buffer: &GlyphBuffer, count: usize, glyph: u32, advance: i32) {
    assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
    assert_eq!(buffer.glyph_infos().len(), count);
    assert_eq!(buffer.glyph_positions().len(), count);
    for (ordinal, (info, position)) in buffer
        .glyph_infos()
        .iter()
        .zip(buffer.glyph_positions())
        .enumerate()
    {
        assert_eq!(info.glyph_id, glyph);
        assert_eq!(info.cluster, u32::try_from(ordinal).unwrap());
        assert_eq!(
            (
                position.x_advance,
                position.y_advance,
                position.x_offset,
                position.y_offset
            ),
            (advance, 0, 0, 0)
        );
    }
}

fn qualified_rejected_hwid_font() -> Vec<u8> {
    let body = rejected_hwid_font();
    let font = ShaperFontRef::new(&body).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    // Qualify the EXACT final native font before either Text outcome. No fake
    // provider or inferred error from table tokens substitutes for execution.
    for count in [1, 2] {
        let normal = shaper.shape(input(count, None), &[]);
        packet(&normal, count, 35, 1000);
        assert!(normal.width_feature_receipt().unwrap().is_none());
        let request = Feature::new(Tag::new(b"hwid"), 1, ..);
        let observed = shaper.shape(input(count, Some(WidthFeature::HalfWidth)), &[request]);
        packet(&observed, count, 35, 1000);
        let receipt = observed.width_feature_receipt().unwrap().unwrap();
        assert_eq!(receipt.feature(), WidthFeature::HalfWidth);
        assert_eq!(receipt.input_len(), count);
        assert_eq!(
            receipt.coverage(),
            WidthObservationCoverage::Unqualified(WidthObservationPath::LookupAdmission)
        );
        for ordinal in 0..count {
            let root = receipt.input(ordinal).unwrap();
            assert_eq!(root.outcome(), WidthInputOutcome::NoAction);
            assert!(root.provenance().is_empty());
        }
        assert!(receipt.input(count).is_none());
    }
    // These remain genuine fitting alternatives for AA. Refusing hwid in the
    // Text case therefore cannot be replaced by silent twid/qwid fallback.
    for (tag, feature, glyph, advance) in [
        (*b"twid", WidthFeature::ThirdWidth, 58, 400),
        (*b"qwid", WidthFeature::QuarterWidth, 16, 300),
    ] {
        let request = Feature::new(Tag::new(&tag), 1, ..);
        let observed = shaper.shape(input(2, Some(feature)), &[request]);
        packet(&observed, 2, glyph, advance);
        let receipt = observed.width_feature_receipt().unwrap().unwrap();
        assert_eq!(receipt.feature(), feature);
        assert_eq!(receipt.input_len(), 2);
        assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
        for ordinal in 0..2 {
            assert_eq!(
                receipt.input(ordinal).unwrap().outcome(),
                WidthInputOutcome::Covered
            );
        }
    }
    body
}

#[test]
fn fitting_normal_cell_succeeds_with_an_undemanded_unqualified_hwid_path() {
    let body = qualified_rejected_hwid_font();
    let (mut system, processed, analyzed, body) =
        source_with_font_features_and_font("A", &[], body);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    assert_eq!(prepared.analyzed_source().source(), processed.source());
    let line = finalize(&prepared);
    one_em(&line);
    assert_eq!(
        line.run_views()
            .flat_map(|run| {
                let glyphs = run
                    .glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>();
                if !glyphs.is_empty() {
                    assert_eq!(run.face().bytes(), body.as_slice());
                }
                glyphs
            })
            .collect::<Vec<_>>(),
        [(35, 10.0)]
    );
    let placed = placed_vertical_cell(&prepared, line);
    let run = placed
        .run_views()
        .find(|run| run.glyphs().next().is_some())
        .unwrap();
    assert_eq!(run.glyph_frame().outline_x(), Point::new(1.0, 0.0));
    assert_eq!(processed.source().text(), &Text::from("A"));
}

#[test]
fn overwide_cell_refuses_demanded_unqualified_hwid_instead_of_falling_back() {
    let body = qualified_rejected_hwid_font();
    let (mut system, processed, analyzed, _) = source_with_font_features_and_font("AA", &[], body);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41, 0x41]);
    assert_eq!(cell.used_unit_count(), 2);
    // Normal W20 exceeds the actual E10; twidW8/qwidW6 are available, but a
    // demanded unqualified hwid observation is a refusal, not candidate absence.
    let error = match system.prepare(
        &analyzed,
        PreparationOptions::default().with_orientation(LineOrientation::Vertical),
    ) {
        Ok(_) => panic!("demanded unqualified hwid must not publish prepared output"),
        Err(error) => error,
    };
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::TextCombine {
            reason: TextCombineError::UnqualifiedWidthObservation {
                scope: 1,
                source: Range::new(0, 2),
                feature: WidthFeature::HalfWidth,
                path: WidthObservationPath::LookupAdmission,
            },
        })
    );
    assert_eq!(processed.source().text(), &Text::from("AA"));
    assert_eq!(processed.processed_text(), &Text::from("AA"));
}
