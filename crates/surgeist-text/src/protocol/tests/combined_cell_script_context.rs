//! Existing-callable real-font witness for an opaque cell's script boundary.
//! Private child of combined_cells.

use super::*;
use skrifa::MetadataProvider;
use surgeist_shaper::{
    Direction, FontRef as ShaperFontRef, ShaperData, ShapingStatus, UnicodeBuffer, script,
};

fn words(values: &[u16]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

fn script_context_font() -> Vec<u8> {
    let original = native_metric_font(&crate::shaping_test_fonts::casing_ahem());
    // grek/latn default LangSys select distinct ccmp FEATURE INDICES. Duplicate
    // feature tags are intentional; this is the existing qualified selected-
    // ScriptList/FeatureList mechanism, not a new SFNT assembly implementation.
    let mut scripts = words(&[2]);
    scripts.extend(*b"grek");
    scripts.extend(words(&[14]));
    scripts.extend(*b"latn");
    scripts.extend(words(&[26]));
    scripts.extend(words(&[4, 0, 0, 0xffff, 1, 0]));
    scripts.extend(words(&[4, 0, 0, 0xffff, 1, 1]));
    assert_eq!(scripts.len(), 38);
    let mut features = words(&[2]);
    features.extend(*b"ccmp");
    features.extend(words(&[14]));
    features.extend(*b"ccmp");
    features.extend(words(&[20]));
    features.extend(words(&[0, 1, 0, 0, 1, 1]));
    assert_eq!(features.len(), 26);
    let mut lookups = words(&[2, 6, 28]);
    for target in [58, 36] {
        // GSUB1/SingleSubst2: the only covered nominal glyph is actual SPACE3.
        lookups.extend(words(&[1, 0, 1, 8]));
        lookups.extend(words(&[2, 8, 1, target, 1, 1, 3]));
    }
    let mut gsub = words(&[1, 0, 10, 48, 74]);
    gsub.extend(scripts);
    gsub.extend(features);
    gsub.extend(lookups);
    let mut hmtx = sfnt::table(&original, *b"hmtx").to_vec();
    for (glyph, advance) in [(3, 1000_u16), (35, 1000), (36, 600), (58, 900)] {
        hmtx[glyph * 4..glyph * 4 + 2].copy_from_slice(&advance.to_be_bytes());
    }
    let body = sfnt::replace_tables(&original, vec![(*b"GSUB", gsub), (*b"hmtx", hmtx)]);
    // Keep the real Greek/Latin/SPACE cmap, licensed outlines, identity, native
    // vertical metrics and named BASE reference from their existing owners.
    for tag in [
        *b"cmap", *b"glyf", *b"loca", *b"maxp", *b"name", *b"vhea", *b"vmtx", *b"BASE",
    ] {
        assert_eq!(sfnt::table(&body, tag), sfnt::table(&original, tag));
    }
    body
}

fn qualify_script_context_font(bytes: &[u8]) {
    let nominal = skrifa::FontRef::new(bytes).unwrap();
    let font = ShaperFontRef::new(bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    for (code_point, glyph) in [(' ', 3), ('A', 35), ('Α', 35)] {
        assert_eq!(
            nominal.charmap().map(code_point),
            Some(skrifa::GlyphId::new(glyph))
        );
    }
    // Authentic strong-script strings qualify both actual native script routes
    // BEFORE the Text assertion, on the exact registered final font bytes.
    for (text, script, first_glyph, first_advance) in [
        (" A", script::LATIN, 36, 600),
        (" Α", script::GREEK, 58, 900),
    ] {
        let mut input = UnicodeBuffer::new();
        for (ordinal, scalar) in text.chars().enumerate() {
            input.add(scalar, u32::try_from(ordinal).unwrap());
        }
        input.set_direction(Direction::LeftToRight);
        input.set_script(script);
        input.set_language("en".parse().unwrap());
        let output = shaper.shape(input, &[]);
        assert_eq!(output.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            output
                .glyph_infos()
                .iter()
                .map(|info| (info.glyph_id, info.cluster))
                .collect::<Vec<_>>(),
            [(first_glyph, 0), (35, 1)]
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
            [(first_advance, 0, 0, 0), (1000, 0, 0, 0)]
        );
    }
}

fn script_context_analysis() -> (Arc<ProcessedSource>, AnalyzedSource) {
    let ordinary = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        text_combine_upright: TextCombineUpright::None,
        text_transform: TextTransform::none(),
        wrap: Wrap::Word,
        ..Style::default()
    };
    let combined = Style {
        text_combine_upright: TextCombineUpright::All,
        ..ordinary.clone()
    };
    let mut source = Source::identified(" ΑA", Id::from_u64(784_906), 76);
    source.span(Range::new(0, 3), ordinary.clone());
    source.span(Range::new(1, 2), combined.clone());
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: (0..3)
                .map(|index| ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(index, index + 1),
                    aggregate: Range::new(index, index + 1),
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..2)
                .map(|index| SourceProcessingConnection {
                    before_origin: index,
                    after_origin: index + 1,
                    kind: ParagraphConnectionKind::Continuous,
                })
                .collect(),
            controls: vec![],
            boundary_policies: (0..2)
                .map(|index| ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(index),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            computed_line_boundaries: vec![],
            scope_alignment_limits: Some(ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            scopes: vec![
                SourceProcessingScope::Block {
                    topology: SourceProcessingTopology::Complete,
                    trim: WhiteSpaceTrim::default(),
                    base_style: ordinary,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(font_reference()),
                },
                SourceProcessingScope::Inline {
                    trim: WhiteSpaceTrim::default(),
                    style: Some(combined),
                    boundaries: None,
                    alignment_boundary: None,
                    font_reference: Some(font_reference()),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            joint_flows: vec![],
            events: vec![
                SourceProcessingEvent::EnterScope(0),
                SourceProcessingEvent::EnterParagraph(0),
                SourceProcessingEvent::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::Connection(0),
                SourceProcessingEvent::EnterScope(1),
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
            ],
        })
        .unwrap(),
    );
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(" ΑA"));
    let raw = processed.paragraphs().next().unwrap();
    assert_eq!(
        raw.source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
    );
    // Use the published exact-view constructor, not a new width-trial front.
    let analyzed =
        AnalyzedSource::from_processed_paragraph(raw, AnalysisTypography::Mixed).unwrap();
    assert_eq!(analyzed.combined_cells().len(), 1);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.selection_unit_count(), 1);
    assert_eq!(cell.used_unit_count(), 1);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x0391]);
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].source(), &source);
    assert_eq!(pieces[0].local(), Range::new(1, 2));
    assert_eq!(pieces[0].aggregate(), Range::new(1, 2));
    assert_eq!(
        pieces[0].origin_position(),
        processed.captured_origin(1).unwrap()
    );
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    (processed, analyzed)
}

#[test]
fn a_greek_cell_does_not_seed_the_outside_common_spaces_ordinary_latin_script() {
    let body = script_context_font();
    qualify_script_context_font(&body);
    let (processed, analyzed) = script_context_analysis();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(body.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 1);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    let line = finalize(&prepared);
    // WM4 treats the cell as one outer U+FFFC-like object for other text layout
    // purposes. Its inner Greek script cannot seed ordinary Common SPACE.
    // The only outside strong script is the actual trailing Latin A.
    for (range, expected) in [
        (Range::new(0, 1), 36),
        (Range::new(1, 2), 35),
        (Range::new(2, 3), 35),
    ] {
        let glyphs = line
            .run_views()
            .filter(|run| run.range() == range)
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), body.as_slice());
                run.glyphs().map(|glyph| glyph.id()).collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs, [expected], "actual source range {range:?}");
    }
    assert_eq!(processed.source().text(), &Text::from(" ΑA"));
}
