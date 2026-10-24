//! Real selected Text resources reach explicit headless GPU pixels.

use super::*;
use crate::{Options, Parameters, RenderRoute, Renderer, Size};
use skrifa::MetadataProvider;

const FONT_SIZE: f32 = 28.0;

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, part| {
        let mut word = [0; 4];
        word[..part.len()].copy_from_slice(part);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

fn blank_control_ahem() -> Vec<u8> {
    // Only the cmap changes: real licensed glyph3 is empty SPACE, while
    // glyph58 is the unchanged positive-area X. Both Cc map to glyph3.
    let entries = [(0_u32, 3_u32), (1, 3), (32, 3), (88, 58)];
    let mut cmap = Vec::new();
    for word in [0_u16, 1, 3, 10] {
        cmap.extend(word.to_be_bytes());
    }
    cmap.extend(12_u32.to_be_bytes());
    cmap.extend(12_u16.to_be_bytes());
    cmap.extend(0_u16.to_be_bytes());
    cmap.extend((16 + entries.len() as u32 * 12).to_be_bytes());
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend((entries.len() as u32).to_be_bytes());
    for (cp, gid) in entries {
        for word in [cp, cp, gid] {
            cmap.extend(word.to_be_bytes());
        }
    }
    let mut bytes = replace_tables(vec![(*b"cmap", cmap)]);
    let records: Vec<_> = (0..usize::from(u16_at(&bytes, 4)))
        .map(|i| 12 + i * 16)
        .collect();
    let head_record = *records
        .iter()
        .find(|&&r| bytes[r..r + 4] == *b"head")
        .unwrap();
    let head = u32_at(&bytes, head_record + 8) as usize;
    bytes[head + 8..head + 12].fill(0);
    for r in records {
        let start = u32_at(&bytes, r + 8) as usize;
        let end = start + u32_at(&bytes, r + 12) as usize;
        let sum = checksum(&bytes[start..end]);
        bytes[r + 4..r + 8].copy_from_slice(&sum.to_be_bytes());
    }
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&bytes));
    bytes[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&bytes), 0xb1b0_afba);
    bytes
}

fn selected_placed(system: &mut text::System, value: &str) -> text::Layout {
    let source = text::Source::new(value);
    let analyzed = text::AnalyzedSource::from_paragraph(text::ParagraphInput {
        source: source.clone(),
        base_style: text::Style {
            size: FONT_SIZE,
            line_height: text::LineHeight::Absolute(f64::from(FONT_SIZE)),
            white_space: text::WhiteSpace::Preserve,
            ..style()
        },
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: text::ParagraphBaseDirection::LeftToRight,
        paragraph_strut: text::ParagraphStrut::DefaultFont,
        terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![text::ParagraphOrigin {
            text_node: None,
            source,
            local: text::Range::new(0, 1),
            aggregate: text::Range::new(0, 1),
            kind: text::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    })
    .unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let termination = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let line = prepared
        .select(text::LineSelectionRequest {
            incoming: None,
            start: prepared.boundary_views().next().unwrap().handle(),
            end: termination.view().boundary(),
            termination: text::PreparedTermination::Ordinary(termination),
            alternative: None,
            participants: prepared.participants().collect(),
            atomics: vec![],
            controls: vec![],
            line_strut: None,
            context: text::LineContext {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                orientation: text::LineOrientation::Horizontal,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: Some(32.0),
                indent: 0.0,
                alignment: text::Alignment::Start,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(text::SpacingAllocations::None)
        .unwrap();
    text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared,
        coverage: text::PlacementCoverage::WholeParagraph,
        lines: vec![text::LinePlacement {
            line,
            anchor: text::LineAnchor {
                inline_origin: 2.0,
                baseline: 26.0,
            },
            orientation: text::LineOrientation::Horizontal,
            atomics: vec![],
        }],
    })
    .unwrap()
}

#[test]
fn retained_native_control_fonts_paint_distinct_codepoints_after_system_drop() {
    let bytes = blank_control_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    for cp in [0_u32, 1, 32] {
        assert_eq!(font.charmap().map(cp).unwrap().to_u32(), 3);
    }
    assert_eq!(font.charmap().map('X').unwrap().to_u32(), 58);
    let mut system = system(bytes.clone());
    let registered = system.registered_font_faces().next().unwrap().to_owned();
    let blank = registered.unsynthesized_instance(FONT_SIZE, &[]).unwrap();
    assert!(
        blank
            .glyph_outline(3, registered.catalogue_identity())
            .unwrap()
            .bounds()
            .is_none()
    );
    assert!(
        blank
            .glyph_outline(58, registered.catalogue_identity())
            .unwrap()
            .bounds()
            .is_some()
    );

    let values = ["X", " ", "\0", "\u{1}"];
    let layouts: Vec<_> = values
        .iter()
        .map(|value| selected_placed(&mut system, value))
        .collect();
    let historical = layouts[2].run_views().next().unwrap().face().to_owned();
    let old_publication = historical.catalogue_identity().clone();
    system.refresh_fonts().unwrap();
    let refreshed = selected_placed(&mut system, "\0");
    let refreshed_run = refreshed.run_views().next().unwrap();
    let refreshed_face = refreshed_run.face();
    assert!(historical.face().is_same_face(&refreshed_face));
    assert_ne!(
        historical.catalogue_identity(),
        refreshed_face.catalogue_identity()
    );
    drop(refreshed);
    // The historical layouts alone retain the intrinsic font during paint.
    drop(historical);
    drop(system);

    let mut renderer = pollster::block_on(Renderer::new(Options::default()))
        .expect("control paint qualification requires a real GPU adapter");
    assert!(renderer.default_wgpu_device_queue().is_some());
    let mut surface = pollster::block_on(renderer.create_headless(Size::new(32.0, 32.0), 1.0))
        .expect("control paint qualification requires a real headless surface");
    let mut pixels = Vec::new();
    for (index, (value, layout)) in values.iter().zip(&layouts).enumerate() {
        let expected_units: Vec<_> = value.encode_utf16().collect();
        assert_eq!(layout.source().text().code_units(), expected_units);
        assert_eq!(
            layout
                .analyzed_source()
                .processed_source()
                .processed_text()
                .code_units(),
            expected_units
        );
        assert_eq!(layout.lines()[0].range(), text::Range::new(0, 1));
        let positions: Vec<_> = layout.positions().collect();
        let before = positions
            .iter()
            .find(|p| p.source_cursor() == text::Cursor::new(0, text::Affinity::After))
            .unwrap();
        let after = positions
            .iter()
            .find(|p| p.source_cursor() == text::Cursor::new(1, text::Affinity::Before))
            .unwrap();
        let selection = layout.position_selection(before, after).unwrap();
        assert_eq!(selection.source_text().code_units(), expected_units);
        assert_eq!(selection.plaintext().code_units(), expected_units);

        let source_run = layout.run_views().next().unwrap();
        let instance = source_run.effective_instance().unwrap();
        let metrics = instance
            .metrics(instance.face().catalogue_identity())
            .unwrap();
        assert_eq!(metrics.units_per_em(), 1000);
        assert_eq!(metrics.instance().font_size(), FONT_SIZE);
        assert_eq!(metrics.instance().face().bytes(), source_run.face().bytes());
        // Both real faces declare UPEM1000, ascent800 and descent-200.
        let scale = f64::from(FONT_SIZE) / 1000.0;
        assert!((metrics.ascent() - 800.0 * scale).abs() < 1e-6);
        assert!((metrics.descent() - -200.0 * scale).abs() < 1e-6);
        let prepared = prepare_text_layout(layout, Transform::identity(), limits()).unwrap();
        assert!(std::ptr::eq(prepared.layout(), layout));
        let (run, transform) = outline(&prepared.scene().commands[0]);
        assert_eq!(run.font().bytes(), source_run.face().bytes());
        assert_eq!(run.font().index(), source_run.face().index());
        assert_eq!(run.instance().size(), instance.font_size());
        assert_eq!(instance.normalized_coords().len(), 0);
        assert_eq!((instance.x_shear(), instance.embolden()), (0.0, 0.0));
        assert!(run.instance().coordinates().is_empty());
        assert_eq!(run.instance().synthesis(), GlyphSynthesis::none());
        assert_eq!(transform, Transform::translation(2.0, 26.0).unwrap());
        let actual_glyphs: Vec<_> = source_run.glyphs().collect();
        assert_eq!(actual_glyphs.len(), 1);
        assert_eq!(source_run.range(), text::Range::new(0, 1));
        assert_eq!(run.glyphs()[0].id(), actual_glyphs[0].id());
        assert_eq!(run.glyphs()[0].advance(), actual_glyphs[0].advance());
        assert_eq!(
            run.glyphs()[0].x(),
            actual_glyphs[0].pen() + f64::from(actual_glyphs[0].x())
        );
        assert_eq!(run.glyphs()[0].y(), f64::from(actual_glyphs[0].y()));
        if index < 2 {
            assert_eq!(run.font().bytes(), bytes);
            assert_eq!(run.glyphs()[0].id(), if index == 0 { 58 } else { 3 });
        } else {
            assert_ne!(run.font().bytes(), bytes);
            assert_eq!(source_run.face().catalogue_identity(), &old_publication);
            assert!(run.bounds().ink_rect().is_some());
            assert!(run.path_elements().count() > 0);
        }
        let stats = pollster::block_on(renderer.render(
            &mut surface,
            prepared.scene(),
            Parameters::default(),
        ))
        .expect("actual selected Text scene must paint through the production GPU path");
        assert_eq!(stats.route, Some(RenderRoute::DirectVello));
        let image = pollster::block_on(renderer.read_headless(&surface))
            .expect("the completed GPU frame must support explicit readback");
        assert_eq!((image.size().width(), image.size().height()), (32, 32));
        let painted = image
            .rgba()
            .chunks_exact(4)
            .filter(|rgba| rgba[3] > 0)
            .count();
        if index == 1 {
            assert_eq!(painted, 0, "qualified blank SPACE must not paint");
        } else {
            assert!(
                painted > 0,
                "{value:?} must produce actual nontransparent pixels"
            );
        }
        eprintln!("control paint {value:?}: {painted} nontransparent pixels");
        pixels.push(image.into_rgba());
    }
    assert_ne!(
        pixels[2], pixels[3],
        "different codepoint depictions must paint differently"
    );
    assert!(renderer.default_device_has_no_terminal_signal_for_test());
}
