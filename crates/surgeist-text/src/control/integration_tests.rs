use crate::{
    AnalyzedSource, DocumentSegmentBreaks, Font, FontCatalogueMode, Options, Range, Source, Style,
    System, SystemOptions, WhiteSpace,
};

fn style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

fn system(font: Option<Vec<u8>>) -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    if let Some(font) = font {
        system.register_fonts(font).unwrap();
    }
    system
}

fn layout(system: &mut System, value: &str) -> crate::Layout {
    system
        .layout(
            Source::new(value),
            style(),
            Options {
                quantize: false,
                ..Options::default()
            },
        )
        .unwrap()
}

#[test]
fn every_eligible_control_has_actual_ink_and_retains_original_units() {
    let mut system = system(None);
    for cp in (0..=159).filter(|&cp| super::is_visible(cp)) {
        let value = char::from_u32(cp).unwrap().to_string();
        let output = layout(&mut system, &value);
        assert_eq!(output.source().text().code_units(), [cp as u16]);
        assert_eq!(
            output
                .analyzed_source()
                .processed_source()
                .processed_text()
                .code_units(),
            [cp as u16]
        );
        let runs: Vec<_> = output.run_views().collect();
        assert_eq!(runs.len(), 1);
        let instance = runs[0].effective_instance().unwrap();
        let glyphs: Vec<_> = runs[0].glyphs().collect();
        assert_eq!(glyphs.len(), 1);
        assert_ne!(glyphs[0].id(), 0);
        let bounds = instance
            .glyph_outline(glyphs[0].id(), instance.face().catalogue_identity())
            .unwrap()
            .bounds()
            .unwrap();
        assert!(
            bounds.size.width > 0.0 && bounds.size.height > 0.0,
            "U+{cp:04X}"
        );
        assert!(glyphs[0].advance() > 0.0);
        assert_eq!(instance.normalized_coords().len(), 0);
        let metrics = instance
            .metrics(instance.face().catalogue_identity())
            .unwrap();
        assert_eq!(metrics.instance().face().bytes(), runs[0].face().bytes());
        assert!(metrics.ascent() > 0.0 && metrics.descent() <= 0.0);
        assert!(metrics.strikeout().thickness() > 0.0);
        assert_eq!(output.lines().len(), 1);
    }
    // Intrinsic synthesis is not ordinary registered-font matching/fallback.
    assert_eq!(system.registered_font_faces().count(), 0);
    assert_eq!(
        system
            .layout(Source::new("X"), style(), Options::default())
            .unwrap_err()
            .code,
        crate::ErrorCode::NoUsableFontFace
    );
}

#[test]
fn original_host_boundaries_and_control_mark_presentation_grapheme_are_distinct() {
    let analyzed = AnalyzedSource::new(Source::new("\u{1}\u{301}"), style()).unwrap();
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    assert_eq!(analyzed.inner.analysis.graphemes.len(), 1);
    assert_eq!(analyzed.inner.analysis.graphemes[0], 0..2);
    assert_eq!(analyzed.source().text().code_units(), [1, 0x301]);
    assert_eq!(analyzed.inner.projection.text, "\u{1}\u{301}");
    let info = analyzed.inner.analysis.infos[0].0;
    assert_eq!(info.script, icu_properties::props::Script::Common);
    assert_eq!(
        info.bidi_class,
        icu_properties::props::BidiClass::OtherNeutral
    );
    assert_eq!(
        info.grapheme_cluster_break,
        icu_properties::props::GraphemeClusterBreak::Other
    );
    assert!(info.contributes_to_shaping());
    assert!(!info.is_control());

    let font = crate::shaping_test_fonts::mark_ahem();
    let mut system = system(Some(font.clone()));
    let prepared = system
        .prepare(&analyzed, crate::PreparationOptions::default())
        .unwrap();
    assert_eq!(
        prepared
            .unit_views()
            .map(|unit| unit.source_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    assert_eq!(
        prepared
            .boundary_views()
            .map(|boundary| boundary.source_before().get())
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(
        prepared
            .unit_views()
            .map(|unit| unit.font_instances().count())
            .sum::<usize>(),
        2
    );
    let output = layout(&mut system, "\u{1}\u{301}");
    let mut saw_control = false;
    let mut saw_mark = false;
    for run in output.run_views() {
        if run.face().bytes() == font {
            assert_eq!(run.glyphs().map(|g| g.id()).collect::<Vec<_>>(), [100]);
            saw_mark = true;
        } else {
            assert!(run.glyphs().all(|g| g.id() != 0));
            saw_control = true;
        }
    }
    assert!(saw_control && saw_mark);
    assert_eq!(output.source().text().code_units(), [1, 0x301]);
    let positions: Vec<_> = output.positions().collect();
    assert_eq!(
        positions
            .iter()
            .map(|position| position.source_cursor().index())
            .collect::<Vec<_>>(),
        [0, 2]
    );
    let selection = output
        .position_selection(positions.first().unwrap(), positions.last().unwrap())
        .unwrap();
    assert_eq!(selection.source_text().code_units(), [1, 0x301]);
    assert_eq!(selection.plaintext().code_units(), [1, 0x301]);
}

#[test]
fn control_symbols_interrupt_cursive_joining_without_replacing_source() {
    let font = crate::shaping_test_fonts::arabic_ahem();
    let mut system = system(Some(font.clone()));
    let connected = layout(&mut system, "\u{628}\u{628}");
    assert_eq!(
        connected
            .run_views()
            .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [82, 100]
    );
    let separated = layout(&mut system, "\u{628}\u{1}\u{628}");
    let ordinary: Vec<_> = separated
        .run_views()
        .filter(|r| r.face().bytes() == font)
        .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
        .collect();
    assert_eq!(ordinary, [35, 35]);
    assert_eq!(separated.source().text().code_units(), [0x628, 1, 0x628]);
    assert_eq!(
        separated.analyzed_source().direction(),
        crate::Direction::RightToLeft
    );
}

#[test]
fn intrinsic_font_survives_refresh_and_remains_publication_qualified() {
    let mut system = system(None);
    let initial = system.font_catalogue_identity().clone();
    let old = layout(&mut system, "\0");
    assert_eq!(system.font_catalogue_identity(), &initial);
    assert_eq!(layout(&mut system, "\0").key(), old.key());
    let old_face = old.run_views().next().unwrap().face().to_owned();
    system.refresh_fonts().unwrap();
    let new = layout(&mut system, "\0");
    let new_face = new.run_views().next().unwrap().face().to_owned();
    assert!(old_face.face().is_same_face(&new_face.face()));
    assert_ne!(old_face.catalogue_identity(), new_face.catalogue_identity());
    let old_run = old.run_views().next().unwrap();
    let old_instance = old_run.effective_instance().unwrap();
    assert!(
        old_instance
            .glyph_outline(
                old_run.glyphs().next().unwrap().id(),
                old_face.catalogue_identity()
            )
            .unwrap()
            .bounds()
            .is_some()
    );
    assert_eq!(
        old_instance
            .glyph_outline(
                old_run.glyphs().next().unwrap().id(),
                new_face.catalogue_identity()
            )
            .unwrap_err()
            .code,
        crate::ErrorCode::FontCatalogueMismatch
    );
    drop(system);
    assert!(
        old_instance
            .glyph_outline(
                old_run.glyphs().next().unwrap().id(),
                old_face.catalogue_identity()
            )
            .unwrap()
            .bounds()
            .is_some()
    );
}

#[test]
fn intrinsic_font_cannot_supply_a_requested_default_font_strut() {
    let analyzed = AnalyzedSource::new(Source::new("\u{1}"), style()).unwrap();
    let mut system = system(None);
    let identity = system.font_catalogue_identity().clone();
    assert_eq!(
        system
            .prepare(&analyzed, crate::PreparationOptions::default())
            .unwrap_err()
            .code,
        crate::ErrorCode::NoUsableFontFace
    );
    assert_eq!(system.font_catalogue_identity(), &identity);
    assert_eq!(system.registered_font_faces().count(), 0);
    assert!(layout(&mut system, "\u{1}").metrics().full_width() > 0.0);
}

#[test]
fn actual_declared_control_delimiter_is_processed_before_symbol_presentation() {
    let source = Source::new("A\u{b}B\u{b}");
    let processed = std::sync::Arc::new(
        crate::process_source_whitespace(
            source,
            style(),
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 2)]),
        )
        .unwrap(),
    );
    assert_eq!(processed.processed_text().code_units(), [65, 10, 66, 11]);
    let paragraph = crate::protocol::paragraph::ParagraphData::standalone(processed.source());
    let projection =
        crate::projection::SourceProjection::from_processed(&processed, &paragraph).unwrap();
    assert_eq!(projection.text, "A\nB\u{b}");
    assert_eq!(projection.segmentation, "A\nB\u{2300}");
    assert!(!super::is_visible(9));
    assert!(!super::is_visible(10));
    assert!(!super::is_visible(13));
    for cp in [0, 1, 11, 12, 0x85] {
        assert!(!crate::engine::analysis::is_hard_separator(cp));
    }
    assert!(crate::engine::analysis::is_hard_separator(10));
}
