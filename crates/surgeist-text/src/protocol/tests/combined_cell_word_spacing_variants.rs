//! A covered width packet must qualify its own intrinsic separator advance,
//! rather than reuse normal's material word-spacing gate. Real GSUB replaces
//! material NBSP153 with retained blank glyph253/advance0 under hwid.

use super::*;
use skrifa::{
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};
use surgeist_shaper::FeatureSource;

fn covered_zero_font() -> Vec<u8> {
    let original = parent_font();
    let template = composition_font();
    let mut gsub = sfnt::table(&template, *b"GSUB").to_vec();
    let features = usize::from(sfnt::u16_at(&gsub, 6));
    let lookups = usize::from(sfnt::u16_at(&gsub, 8));
    assert_eq!(sfnt::u16_at(&gsub, features), 3);
    assert_eq!(sfnt::u16_at(&gsub, lookups), 3);
    for (tag, prior, replacement) in [
        (*b"hwid", 36, 253_u16),
        (*b"twid", 58, 36),
        (*b"qwid", 16, 58),
    ] {
        let row = (0..3)
            .map(|index| features + 2 + index * 6)
            .find(|&row| gsub[row..row + 4] == tag)
            .unwrap();
        let feature = features + usize::from(sfnt::u16_at(&gsub, row + 4));
        assert_eq!(sfnt::u16_at(&gsub, feature + 2), 1);
        let index = usize::from(sfnt::u16_at(&gsub, feature + 4));
        let lookup = lookups + usize::from(sfnt::u16_at(&gsub, lookups + 2 + index * 2));
        assert_eq!(sfnt::u16_at(&gsub, lookup), 1); // actual SingleSubst
        assert_eq!(sfnt::u16_at(&gsub, lookup + 4), 1);
        let sub = lookup + usize::from(sfnt::u16_at(&gsub, lookup + 6));
        assert_eq!(sfnt::u16_at(&gsub, sub), 2);
        assert_eq!(sfnt::u16_at(&gsub, sub + 4), 1);
        assert_eq!(sfnt::u16_at(&gsub, sub + 6), prior);
        let coverage = sub + usize::from(sfnt::u16_at(&gsub, sub + 2));
        assert_eq!(&gsub[coverage..coverage + 6], &[0, 1, 0, 1, 0, 35]);
        // Same genuine one-operand record/length/offsets, now for NBSP153.
        gsub[coverage + 4..coverage + 6].copy_from_slice(&153_u16.to_be_bytes());
        gsub[sub + 6..sub + 8].copy_from_slice(&replacement.to_be_bytes());
    }
    let bytes = sfnt::replace_tables(&original, vec![(*b"GSUB", gsub.clone())]);
    assert_eq!(sfnt::table(&bytes, *b"GSUB"), gsub);
    for tag in [
        *b"cmap", *b"hmtx", *b"glyf", *b"loca", *b"hhea", *b"vhea", *b"vmtx", *b"BASE", *b"name",
    ] {
        assert_eq!(sfnt::table(&bytes, tag), sfnt::table(&original, tag));
    }
    glyph_frame_font_premises(&bytes);
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.charmap().map(' '), Some(GlyphId::new(153)));
    let metrics = parsed.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for (glyph, advance) in [(153, 1000.0), (253, 0.0), (36, 1000.0), (58, 1000.0)] {
        assert_eq!(metrics.advance_width(GlyphId::new(glyph)), Some(advance));
    }
    assert_eq!(parsed.head().unwrap().index_to_loc_format(), 0);
    let loca = sfnt::table(&bytes, *b"loca");
    assert_eq!(sfnt::u16_at(loca, 253 * 2), sfnt::u16_at(loca, 254 * 2));
    let blank = metrics.bounds(GlyphId::new(253)).unwrap();
    assert_eq!(
        [blank.x_min, blank.y_min, blank.x_max, blank.y_max],
        [0.0; 4]
    );
    let font = ShaperFontRef::new(&bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    for (request, glyph, advance) in [
        (None, 153, 1000),
        (Some((*b"hwid", WidthFeature::HalfWidth)), 253, 0),
        (Some((*b"twid", WidthFeature::ThirdWidth)), 36, 1000),
        (Some((*b"qwid", WidthFeature::QuarterWidth)), 58, 1000),
    ] {
        let mut input = UnicodeBuffer::new();
        input.add(' ', 0);
        input.set_direction(Direction::LeftToRight);
        input.set_script(script::LATIN);
        input.set_language("en".parse().unwrap());
        let requested = request.map(|(tag, _)| Feature::new(Tag::new(&tag), 1, ..));
        if let Some((_, feature)) = request {
            input.observe_width_feature(feature);
        }
        let features = requested.into_iter().collect::<Vec<_>>();
        let output = shaper.shape(input, &features);
        assert_eq!(output.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            output
                .glyph_infos()
                .iter()
                .map(|info| (info.glyph_id, info.cluster))
                .collect::<Vec<_>>(),
            [(glyph, 0)]
        );
        assert_eq!(
            output
                .glyph_positions()
                .iter()
                .map(|position| (
                    position.x_advance,
                    position.y_advance,
                    position.x_offset,
                    position.y_offset
                ))
                .collect::<Vec<_>>(),
            [(advance, 0, 0, 0)]
        );
        if let Some((_, feature)) = request {
            let receipt = output.width_feature_receipt().unwrap().unwrap();
            assert_eq!(receipt.feature(), feature);
            assert_eq!(receipt.input_len(), 1);
            assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
            let input = receipt.input(0).unwrap();
            assert_eq!(input.outcome(), WidthInputOutcome::Covered);
            assert!(input.provenance().iter().any(|entry| entry.source()
                == FeatureSource::Request(features[0])
                && entry.tag() == features[0].tag));
            assert!(receipt.input(1).is_none());
        } else {
            assert!(output.width_feature_receipt().unwrap().is_none());
        }
    }
    bytes
}

fn output(prepared: &PreparedText, line: &FinalizedLine, processed: &ProcessedSource, font: &[u8]) {
    one_em(line);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual covered-width Cell");
    };
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(
        cell.source_cell().scope(),
        processed.captured_scope(1).unwrap()
    );
    let runs = line
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    let [run] = runs.as_slice() else {
        panic!("retained zero-advance provider packet");
    };
    assert!(run.cell().is_some());
    assert_eq!(run.face().bytes(), font);
    assert_eq!(run.font_size(), 10.0);
    assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
    assert_eq!(
        run.glyphs()
            .map(|glyph| (
                glyph.range(),
                glyph.id(),
                run.origin().x + glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance()
            ))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), 253, 0.0, 0.0, 0.0, 0.0)]
    );
    let layout = placed_vertical_cell(prepared, line.clone());
    assert_eq!(layout.source(), processed.source());
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    let packets = layout
        .glyph_runs()
        .into_iter()
        .filter(|packet| !packet.glyphs().is_empty())
        .collect::<Vec<_>>();
    let [packet] = packets.as_slice() else {
        panic!("actual owned selected hwid glyph");
    };
    assert_eq!(packet.font().bytes(), font);
    assert_eq!(packet.font_size(), 10.0);
    let frame = packet.glyph_frame();
    assert_eq!(frame.outline_x(), Point::new(1.0, 0.0));
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
    assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
    // Intrinsic0 suppresses the packet's OWN WS4: interval0..0, offset5.
    close(frame.origin().x, 100.0 + run.origin().x);
    close(frame.origin().y, 28.0);
    let [glyph] = packet.glyphs() else {
        panic!("one genuine substituted blank glyph");
    };
    assert_eq!(
        (glyph.range(), glyph.id(), glyph.advance()),
        (Range::new(0, 1), 253, 0.0)
    );
    close(frame.origin().x + glyph.pen(), 100.0);
}

#[test]
fn covered_zero_intrinsic_width_packet_suppresses_its_own_word_term_and_retains_source() {
    let font = covered_zero_font();
    let (processed, analyzed) = captured(" ", WhiteSpace::Preserve, 4.0);
    source_custody(&processed, &analyzed, &[0], 4.0);
    assert_eq!(processed.source().text(), &Text::from(" "));
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 1);
    // Normal NBSP153 intrinsic10 has WS4 and occupied width14>E10.
    // Covered hwid253 intrinsic0 suppresses that term and is the first fit.
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one real nonempty-source Cell");
    };
    assert_eq!(cell.extent(), 10.0);
    assert!(cell.has_provider_input());
    assert_eq!(
        cell.source_cell().used_code_points().collect::<Vec<_>>(),
        [0xa0]
    );
    let metrics = prepared
        .scope_metrics(&processed.captured_scope(1).unwrap())
        .unwrap();
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!(
        metrics.effective_instance().unwrap().face().bytes(),
        font.as_slice()
    );
    let line = complete_line(&prepared);
    output(&prepared, &line, &processed, &font);
    drop(system);
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    source_custody(
        &processed,
        &replay.prepared_text().analyzed_source(),
        &[0],
        4.0,
    );
    output(&prepared, &replay, &processed, &font);
}
