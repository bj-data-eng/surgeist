//! Real-provider diagnostic for surviving Normal DI before attached-mark relief.
//! Intended private child of combined_cell_widths; fixture premises execute
//! before Text admission. No provider result is assumed from a glyph class.

use super::*;
use surgeist_shaper::{
    Direction as ShapeDirection, Feature, FeatureSource, FontRef, GlyphBuffer, ShaperData,
    ShapingStatus, Tag, UnicodeBuffer, WidthFeature, WidthInputOutcome, WidthObservationCoverage,
    script,
};

fn words(values: &[u16]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}
fn short(value: usize) -> u16 {
    u16::try_from(value).unwrap()
}

fn cgj_cmap() -> Vec<u8> {
    cmap_for_mark('\u{034f}')
}

fn cmap_for_mark(mark: char) -> Vec<u8> {
    let mappings = [
        (0x20_u32, 274_u32),
        (0x41, 35),
        (u32::from(mark), 100),
        (0x25cc, 277),
    ];
    let mut table = words(&[0, 1, 3, 10]);
    table.extend(12_u32.to_be_bytes());
    table.extend(words(&[12, 0]));
    table.extend(
        u32::try_from(16 + mappings.len() * 12)
            .unwrap()
            .to_be_bytes(),
    );
    table.extend(0_u32.to_be_bytes());
    table.extend(u32::try_from(mappings.len()).unwrap().to_be_bytes());
    for (point, glyph) in mappings {
        table.extend(point.to_be_bytes());
        table.extend(point.to_be_bytes());
        table.extend(glyph.to_be_bytes());
    }
    table
}

fn cgj_gsub() -> Vec<u8> {
    // Real GSUB1 format2. Default ccmp promotes CGJ100 to retained normalR58.
    // Optional width operations cover only A35, never the original CGJ root.
    let entries = [
        (*b"ccmp", 100, 58),
        (*b"hwid", 35, 36),
        (*b"qwid", 35, 16),
        (*b"twid", 35, 58),
    ];
    let mut scripts = words(&[1]);
    scripts.extend(*b"DFLT");
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, 4, 0, 1, 2, 3]));
    let mut features = words(&[4]);
    for (index, (tag, _, _)) in entries.iter().enumerate() {
        features.extend(tag);
        features.extend(words(&[short(2 + 6 * entries.len() + 6 * index)]));
    }
    for index in 0..entries.len() {
        features.extend(words(&[0, 1, short(index)]));
    }
    let mut lookups = words(&[4]);
    lookups.resize(2 + entries.len() * 2, 0);
    for (index, (_, input, output)) in entries.into_iter().enumerate() {
        let offset = short(lookups.len());
        lookups[2 + index * 2..4 + index * 2].copy_from_slice(&offset.to_be_bytes());
        lookups.extend(words(&[1, 0, 1, 8]));
        lookups.extend(words(&[2, 8, 1, output, 1, 1, input]));
    }
    let mut result = words(&[
        1,
        0,
        10,
        short(10 + scripts.len()),
        short(10 + scripts.len() + features.len()),
    ]);
    result.extend(scripts);
    result.extend(features);
    result.extend(lookups);
    result
}

fn normal_cgj_font() -> Vec<u8> {
    let original = composition_font();
    let mut hmtx = sfnt::table(&original, *b"hmtx").to_vec();
    hmtx[35 * 4..35 * 4 + 2].copy_from_slice(&2000_u16.to_be_bytes());
    // Width targets remain B36=500/R58=400/period16=300. R58 has a real
    // retained outline and advance400; the promoted DI is not a blank proxy.
    let mut hhea = sfnt::table(&original, *b"hhea").to_vec();
    hhea[10..12].copy_from_slice(&2000_u16.to_be_bytes());
    hhea[12..14].copy_from_slice(&0_i16.to_be_bytes());
    hhea[14..16].copy_from_slice(&(-700_i16).to_be_bytes());
    hhea[16..18].copy_from_slice(&1000_i16.to_be_bytes());
    // Actual glyph classes determine native mark positioning/advance. They
    // confer NO Text coverage or DI exception. GSUB/DI receipts below prove
    // the semantic promotion independently of this real positioning input.
    let gdef = words(&[
        1, 0, 12, 0, 0, 0, 2, 4, 16, 16, 1, 35, 36, 1, 58, 58, 1, 100, 100, 3,
    ]);
    let result = sfnt::replace_tables(
        &original,
        vec![
            (*b"cmap", cgj_cmap()),
            (*b"GSUB", cgj_gsub()),
            (*b"GDEF", gdef),
            (*b"hhea", hhea),
            (*b"hmtx", hmtx),
        ],
    );
    for tag in [
        *b"name", *b"glyf", *b"loca", *b"maxp", *b"vhea", *b"vmtx", *b"BASE",
    ] {
        assert_eq!(sfnt::table(&result, tag), sfnt::table(&original, tag));
    }
    assert_eq!(sfnt::u16_at(sfnt::table(&result, *b"head"), 18), 1000);
    for (glyph, advance) in [(35, 2000), (36, 500), (58, 400), (16, 300)] {
        assert_eq!(
            sfnt::u16_at(sfnt::table(&result, *b"hmtx"), glyph * 4),
            advance
        );
    }
    result
}

fn native_shape(bytes: &[u8], width: Option<(WidthFeature, &[u8; 4])>) -> GlyphBuffer {
    native_shape_mark(bytes, '\u{034f}', width)
}

fn native_shape_mark(
    bytes: &[u8],
    mark: char,
    width: Option<(WidthFeature, &[u8; 4])>,
) -> GlyphBuffer {
    let font = FontRef::new(bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    let mut input = UnicodeBuffer::new();
    input.add('A', 0);
    input.add(mark, 1);
    input.set_direction(ShapeDirection::LeftToRight);
    input.set_script(script::LATIN);
    input.set_language("en".parse().unwrap());
    let features = if let Some((feature, tag)) = width {
        input.observe_width_feature(feature);
        vec![Feature::new(Tag::new(tag), 1, ..)]
    } else {
        vec![]
    };
    shaper.shape(input, &features)
}

fn assert_native_normal(buffer: &GlyphBuffer, first: u32, advance: i32) {
    assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
    assert_eq!(
        buffer
            .glyph_infos()
            .iter()
            .zip(buffer.glyph_positions())
            .map(|(info, position)| (
                info.glyph_id,
                position.x_advance,
                position.y_advance,
                position.x_offset,
                position.y_offset
            ))
            .collect::<Vec<_>>(),
        [(first, advance, 0, 0, 0), (58, 400, 0, 0, 0)]
    );
    let receipt = buffer.default_ignorable_receipt().unwrap();
    assert!(receipt.residual_input_positions().is_empty());
    assert!(receipt.deleted_input_positions().is_empty());
    assert!(receipt.residual_glyph_positions().is_empty());
}

fn provider_premises(bytes: &[u8]) {
    use skrifa::{GlyphId, MetadataProvider};
    let font = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(font.charmap().map('A'), Some(GlyphId::new(35)));
    assert_eq!(font.charmap().map('\u{034f}'), Some(GlyphId::new(100)));
    let normal = native_shape(bytes, None);
    assert_native_normal(&normal, 35, 2000);
    assert!(normal.width_feature_receipt().unwrap().is_none());
    for (feature, tag, glyph, advance) in [
        (WidthFeature::HalfWidth, *b"hwid", 36, 500),
        (WidthFeature::ThirdWidth, *b"twid", 58, 400),
        (WidthFeature::QuarterWidth, *b"qwid", 16, 300),
    ] {
        let result = native_shape(bytes, Some((feature, &tag)));
        assert_native_normal(&result, glyph, advance);
        let receipt = result.width_feature_receipt().unwrap().unwrap();
        assert_eq!(receipt.feature(), feature);
        assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
        assert_eq!(receipt.input_len(), 2);
        let base = receipt.input(0).unwrap();
        assert_eq!(base.outcome(), WidthInputOutcome::Covered);
        let requested = Feature::new(Tag::new(&tag), 1, ..);
        assert!(
            base.provenance()
                .iter()
                .any(|entry| entry.tag() == requested.tag
                    && entry.source() == FeatureSource::Request(requested))
        );
        let di = receipt.input(1).unwrap();
        assert_eq!(di.outcome(), WidthInputOutcome::NoAction);
        assert!(di.provenance().is_empty());
        assert!(receipt.input(2).is_none());
    }
}

fn capture_normal_cgj(body: Vec<u8>) -> (System, Arc<ProcessedSource>, AnalyzedSource) {
    capture_attached_mark(body, '\u{034f}')
}

fn capture_attached_mark(
    body: Vec<u8>,
    mark: char,
) -> (System, Arc<ProcessedSource>, AnalyzedSource) {
    assert!(matches!(mark, '\u{0307}' | '\u{034f}'));
    let text = format!("A{mark}");
    let parent_style = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        text_combine_upright: TextCombineUpright::None,
        text_transform: TextTransform::none(),
        wrap: Wrap::Word,
        ..Style::default()
    };
    let child_style = Style {
        font: Font::new().family(crate::shaping_test_fonts::COMBINED_WIDTH_FAMILY),
        line_height: LineHeight::Absolute(10.0),
        text_combine_upright: TextCombineUpright::All,
        ..parent_style.clone()
    };
    let mut source = Source::identified(text.as_str(), Id::from_u64(784_906), 76);
    source.span(Range::new(0, 2), child_style.clone());
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: vec![ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(0, 2),
                aggregate: Range::new(0, 2),
                kind: ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            scope_alignment_limits: Some(ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            scopes: vec![
                SourceProcessingScope::Block {
                    topology: SourceProcessingTopology::Complete,
                    trim: WhiteSpaceTrim::default(),
                    base_style: parent_style,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(font_reference()),
                },
                SourceProcessingScope::Inline {
                    trim: WhiteSpaceTrim::default(),
                    style: Some(child_style),
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
                SourceProcessingEvent::EnterScope(1),
                SourceProcessingEvent::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitScope(1),
                SourceProcessingEvent::ExitParagraph(0),
                SourceProcessingEvent::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(text.as_str()));
    let raw = processed.paragraphs().next().unwrap();
    assert_eq!(raw.source_pieces().collect::<Vec<_>>(), [Range::new(0, 2)]);
    let domain = AnalyzedDomain::from_processed_paragraph(raw, AnalysisTypography::Mixed).unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.combined_cells().len(), 1);
    let cell = output.combined_cells().next().unwrap();
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(cell.selection_unit_count(), 1); // Both actual mark fixtures form one genuine source EGC.
    assert_eq!(cell.used_unit_count(), 1);
    assert_eq!(
        cell.used_code_points().collect::<Vec<_>>(),
        [0x41, u32::from(mark)]
    );
    assert_eq!(
        cell.presentation_code_points().collect::<Vec<_>>(),
        [0x41, u32::from(mark)]
    );
    assert_eq!(
        cell.source_pieces()
            .map(|piece| {
                assert_eq!(piece.source(), &source);
                assert_eq!(
                    piece.origin_position(),
                    processed.captured_origin(0).unwrap()
                );
                assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
                assert!(piece.is_used());
                (piece.local(), piece.aggregate())
            })
            .collect::<Vec<_>>(),
        [
            (Range::new(0, 1), Range::new(0, 1)),
            (Range::new(1, 2), Range::new(1, 2))
        ]
    );
    let analyzed = crate::test_wrapping::bind_missing(output, Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(parent_font()).unwrap();
    system.register_fonts(body).unwrap();
    (system, processed, analyzed)
}

#[test]
fn surviving_normal_cgj_cannot_inherit_base_width_coverage_as_an_attached_mark() {
    let body = normal_cgj_font();
    // An Mn adaptation is not assumed from the older Cf fixture: these public
    // calls on EXACT final bytes must prove Complete + Normal DI + NoAction.
    provider_premises(&body);
    let (mut system, processed, analyzed) = capture_normal_cgj(body.clone());
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let line = finalize(&prepared);
    one_em(&line);
    // Normal input has occupied interval0..24 from20+4. Width results9/8/7
    // are each incomplete for surviving Normal CGJ: none may borrow ordinary
    // attached-mark relief. Preserve normal35/58 and compress by10/24=5/12.
    assert_eq!(
        line.run_views()
            .flat_map(|run| run
                .glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [(35, 20.0), (58, 4.0)]
    );
    let layout = placed_vertical_cell(&prepared, line);
    assert_eq!(layout.source(), processed.source());
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    let views = layout
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    assert_eq!(views.len(), 1);
    let view = &views[0];
    assert_eq!(view.face().bytes(), body.as_slice());
    assert_eq!(
        view.glyphs()
            .map(|glyph| (
                glyph.id(),
                glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance()
            ))
            .collect::<Vec<_>>(),
        [(35, 0.0, 0.0, 0.0, 20.0), (58, 20.0, 0.0, 0.0, 4.0)]
    );
    let frame = view.glyph_frame();
    close(frame.origin().x, 95.0);
    close(frame.origin().y, 28.0);
    close(frame.outline_x().x, 5.0 / 12.0);
    assert_eq!(frame.outline_x().y, 0.0);
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
    assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
    close(frame.origin().x + frame.outline_x().x * 24.0, 105.0);
    assert_eq!(processed.source().text(), &Text::from("A\u{034f}"));
}

#[derive(Clone, Copy)]
enum CgjAction {
    Delete,
    CoverNormal,
}

fn single_substitution(mappings: &[(u16, u16)]) -> Vec<u8> {
    let mut table = words(&[2, short(6 + mappings.len() * 2), short(mappings.len())]);
    for (_, output) in mappings {
        table.extend(words(&[*output]));
    }
    table.extend(words(&[1, short(mappings.len())]));
    for (input, _) in mappings {
        table.extend(words(&[*input]));
    }
    table
}

fn cgj_action_gsub(action: CgjAction) -> Vec<u8> {
    // Default deletion is a genuine GSUB2 empty Sequence, not a buffer flag.
    // The other variant preserves the confirmed default promotion100->58,
    // then independently gives that surviving root a successful width action.
    let default = match action {
        CgjAction::Delete => (2, words(&[1, 10, 1, 8, 0, 1, 1, 100])),
        CgjAction::CoverNormal => (1, single_substitution(&[(100, 58)])),
    };
    let mut entries = vec![(*b"ccmp", default.0, default.1)];
    for (tag, target) in [(*b"hwid", 36), (*b"qwid", 16), (*b"twid", 58)] {
        let mappings = match action {
            CgjAction::Delete => vec![(35, target)],
            // A successful identity substitution is a real action: the
            // receipt must establish CGJ's own coverage, not its glyph ID.
            CgjAction::CoverNormal => vec![(35, target), (58, 58)],
        };
        entries.push((tag, 1, single_substitution(&mappings)));
    }
    let mut scripts = words(&[1]);
    scripts.extend(*b"DFLT");
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, 4, 0, 1, 2, 3]));
    let mut features = words(&[4]);
    for (index, (tag, _, _)) in entries.iter().enumerate() {
        features.extend(tag);
        features.extend(words(&[short(2 + 6 * entries.len() + 6 * index)]));
    }
    for index in 0..entries.len() {
        features.extend(words(&[0, 1, short(index)]));
    }
    let mut lookups = words(&[4]);
    lookups.resize(2 + entries.len() * 2, 0);
    for (index, (_, kind, subtable)) in entries.into_iter().enumerate() {
        let offset = short(lookups.len());
        lookups[2 + index * 2..4 + index * 2].copy_from_slice(&offset.to_be_bytes());
        lookups.extend(words(&[kind, 0, 1, 8]));
        lookups.extend(subtable);
    }
    let mut result = words(&[
        1,
        0,
        10,
        short(10 + scripts.len()),
        short(10 + scripts.len() + features.len()),
    ]);
    result.extend(scripts);
    result.extend(features);
    result.extend(lookups);
    result
}

fn native_packet(buffer: &GlyphBuffer) -> Vec<(u32, i32)> {
    assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
    buffer
        .glyph_infos()
        .iter()
        .zip(buffer.glyph_positions())
        .map(|(info, position)| {
            assert_eq!(position.y_advance, 0);
            (info.glyph_id, position.x_advance)
        })
        .collect()
}

fn assert_width_roots(
    buffer: &GlyphBuffer,
    feature: WidthFeature,
    tag: &[u8; 4],
    second: WidthInputOutcome,
) {
    let receipt = buffer.width_feature_receipt().unwrap().unwrap();
    assert_eq!(receipt.feature(), feature);
    assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
    assert_eq!(receipt.input_len(), 2);
    let requested = Feature::new(Tag::new(tag), 1, ..);
    for (index, outcome) in [WidthInputOutcome::Covered, second].into_iter().enumerate() {
        let root = receipt.input(index).unwrap();
        assert_eq!(root.outcome(), outcome);
        if outcome == WidthInputOutcome::Covered {
            assert!(
                root.provenance()
                    .iter()
                    .any(|entry| entry.tag() == requested.tag
                        && entry.source() == FeatureSource::Request(requested))
            );
        } else {
            assert!(root.provenance().is_empty());
        }
    }
    assert!(receipt.input(2).is_none());
}

fn assert_half_width_payload(
    body: Vec<u8>,
    mark: char,
    expected: &[(u32, f32)],
    width: f64,
    inset: f64,
) {
    assert_half_width_payload_offsets(body, mark, expected, width, inset, true);
}

fn assert_half_width_payload_offsets(
    body: Vec<u8>,
    mark: char,
    expected: &[(u32, f32)],
    width: f64,
    inset: f64,
    check_mark_offsets: bool,
) {
    let (mut system, processed, analyzed) = capture_attached_mark(body.clone(), mark);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let line = finalize(&prepared);
    one_em(&line);
    assert_eq!(
        line.run_views()
            .flat_map(|run| run
                .glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        expected
    );
    let layout = placed_vertical_cell(&prepared, line);
    assert_eq!(layout.source(), processed.source());
    assert_eq!(
        processed.source().text(),
        &Text::from(format!("A{mark}").as_str())
    );
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    let views = layout
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    assert_eq!(views.len(), 1);
    let view = &views[0];
    assert_eq!(view.face().bytes(), body.as_slice());
    let mut pen = 0.0;
    for (index, (glyph, &(id, advance))) in view.glyphs().zip(expected).enumerate() {
        assert_eq!(glyph.id(), id);
        close(glyph.pen(), pen);
        if check_mark_offsets || index == 0 {
            assert_eq!((glyph.x(), glyph.y()), (0.0, 0.0));
        }
        assert_eq!(glyph.advance(), advance);
        pen += f64::from(advance);
    }
    close(pen, width);
    let frame = view.glyph_frame();
    close(frame.origin().x, 95.0 + inset);
    close(frame.origin().y, 28.0);
    assert_eq!(frame.outline_x(), Point::new(1.0, 0.0));
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
    assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
    close(inset * 2.0 + width, 10.0);
}

fn assert_native_ordinary_mark(buffer: &GlyphBuffer, first: u32, advance: i32) {
    // ot_shape's default Latin fallback invokes position_marks. For the actual
    // U0307 CCC230 mark, that owner writes zero advances after positioning;
    // this remains an ordinary nonDI source mark despite GSUB100->58.
    assert_eq!(native_packet(buffer), [(first, advance), (58, 0)]);
    let receipt = buffer.default_ignorable_receipt().unwrap();
    assert!(receipt.residual_input_positions().is_empty());
    assert!(receipt.deleted_input_positions().is_empty());
    assert!(receipt.residual_glyph_positions().is_empty());
}

#[test]
fn ordinary_non_di_attached_mark_can_inherit_its_covered_base_width_action() {
    // U0307 is an ordinary attached Mn, not Default_Ignorable. The cmap lacks
    // precomposed U0226, so actual shaping must also prove the two-root route.
    let body = sfnt::replace_tables(
        &normal_cgj_font(),
        vec![(*b"cmap", cmap_for_mark('\u{0307}'))],
    );
    let normal = native_shape_mark(&body, '\u{0307}', None);
    assert_native_ordinary_mark(&normal, 35, 2000);
    assert!(normal.width_feature_receipt().unwrap().is_none());
    for (feature, tag, glyph, advance) in [
        (WidthFeature::HalfWidth, *b"hwid", 36, 500),
        (WidthFeature::ThirdWidth, *b"twid", 58, 400),
        (WidthFeature::QuarterWidth, *b"qwid", 16, 300),
    ] {
        let result = native_shape_mark(&body, '\u{0307}', Some((feature, &tag)));
        assert_native_ordinary_mark(&result, glyph, advance);
        assert_width_roots(&result, feature, &tag, WidthInputOutcome::NoAction);
    }
    // Maintained Latin fallback positioning zeros this ordinary CCC230 mark,
    // independently of its target's hmtx400/GDEFBase class. Its native fallback
    // offsets are outside this coverage control's numerical oracle. The
    // half-width base contributes5 and the mark0; center(10-5)/2=2.5.
    assert_half_width_payload_offsets(body, '\u{0307}', &[(36, 5.0), (58, 0.0)], 5.0, 2.5, false);
}

#[test]
fn residual_and_actually_deleted_cgj_do_not_block_covered_base_width_or_paint() {
    for deleted in [false, true] {
        let gsub = if deleted {
            cgj_action_gsub(CgjAction::Delete)
        } else {
            // The original qualified font has ONLY its three width features.
            // Omitting ccmp leaves CGJ truly residual; a self-substitution
            // would be an action and would not establish this premise.
            sfnt::table(&composition_font(), *b"GSUB").to_vec()
        };
        let body = sfnt::replace_tables(&normal_cgj_font(), vec![(*b"GSUB", gsub)]);
        let normal = native_shape(&body, None);
        let expected = if deleted {
            vec![(35, 2000)]
        } else {
            vec![(35, 2000), (274, 0)]
        };
        assert_eq!(native_packet(&normal), expected);
        assert!(normal.width_feature_receipt().unwrap().is_none());
        for (feature, tag, glyph, advance) in [
            (WidthFeature::HalfWidth, *b"hwid", 36, 500),
            (WidthFeature::ThirdWidth, *b"twid", 58, 400),
            (WidthFeature::QuarterWidth, *b"qwid", 16, 300),
        ] {
            let result = native_shape(&body, Some((feature, &tag)));
            let expected = if deleted {
                vec![(glyph, advance)]
            } else {
                vec![(glyph, advance), (274, 0)]
            };
            assert_eq!(native_packet(&result), expected);
            assert_width_roots(
                &result,
                feature,
                &tag,
                WidthInputOutcome::NoSurvivingOperand,
            );
            let receipt = result.default_ignorable_receipt().unwrap();
            assert_eq!(
                receipt.deleted_input_positions(),
                if deleted { &[1][..] } else { &[][..] }
            );
            assert_eq!(
                receipt.residual_input_positions(),
                if deleted { &[][..] } else { &[1][..] }
            );
            assert_eq!(
                receipt.residual_glyph_positions(),
                if deleted { &[][..] } else { &[1][..] }
            );
        }
        let receipt = normal.default_ignorable_receipt().unwrap();
        assert_eq!(
            receipt.deleted_input_positions(),
            if deleted { &[1][..] } else { &[][..] }
        );
        assert_eq!(
            receipt.residual_input_positions(),
            if deleted { &[][..] } else { &[1][..] }
        );
        assert_eq!(
            receipt.residual_glyph_positions(),
            if deleted { &[][..] } else { &[1][..] }
        );
        // Retire only the actual Residual/Deleted native paint. Original CGJ
        // remains in the authentic semantic source pieces checked by capture.
        // Half-width base5 is widest fitting candidate; center(10-5)/2=2.5.
        assert_half_width_payload(body, '\u{034f}', &[(36, 5.0)], 5.0, 2.5);
    }
}

#[test]
fn surviving_normal_cgj_with_its_own_width_action_remains_eligible() {
    let body = sfnt::replace_tables(
        &normal_cgj_font(),
        vec![(*b"GSUB", cgj_action_gsub(CgjAction::CoverNormal))],
    );
    let normal = native_shape(&body, None);
    assert_native_normal(&normal, 35, 2000);
    assert!(normal.width_feature_receipt().unwrap().is_none());
    for (feature, tag, glyph, advance) in [
        (WidthFeature::HalfWidth, *b"hwid", 36, 500),
        (WidthFeature::ThirdWidth, *b"twid", 58, 400),
        (WidthFeature::QuarterWidth, *b"qwid", 16, 300),
    ] {
        let result = native_shape(&body, Some((feature, &tag)));
        assert_native_normal(&result, glyph, advance);
        assert_width_roots(&result, feature, &tag, WidthInputOutcome::Covered);
    }
    // Same NormalDI source/advance4 as the negative witness, but this time
    // its OWN explicit successful width action is proven by the receipt.
    assert_half_width_payload(body, '\u{034f}', &[(36, 5.0), (58, 4.0)], 9.0, 0.5);
}
