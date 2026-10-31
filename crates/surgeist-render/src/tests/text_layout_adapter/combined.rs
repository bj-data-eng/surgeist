//! Real combined-cell glyph pens follow the horizontal payload inside a Vertical line.
//! The licensed Ahem derivative executes GSUB Multiple A35 ->35,36. Each real
//! horizontal advance is500/UPEM1000, so two normal glyphs fit the size10 cell.
use super::*;
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
    raw::TableProvider,
};
use std::sync::Arc;

fn words(values: &[u16]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

fn multiple_gsub() -> Vec<u8> {
    // GSUB1.0: ScriptList10, FeatureList30, LookupList44. DFLT's default
    // LangSys enables normal ccmp, whose sole type2 lookup owns the subtable.
    let mut bytes = words(&[1, 0, 10, 30, 44, 1]);
    bytes.extend(*b"DFLT");
    bytes.extend(words(&[8, 4, 0, 0, 0xffff, 1, 0, 1]));
    bytes.extend(*b"ccmp");
    bytes.extend(words(&[8, 0, 1, 0, 1, 4, 2, 0, 1, 8]));
    // MultipleSubst1: coverage at8 contains35; sequence at14 is[35,36].
    bytes.extend(words(&[1, 8, 1, 14, 1, 1, 35, 2, 35, 36]));
    assert_eq!(bytes.len(), 76);
    bytes
}

fn vertical_base() -> Vec<u8> {
    // BASE1.0 with no horizontal axis. Vertical Axis8 has tag list4 and
    // script list14; Latn BaseValues has real design coordinates0 and1200.
    let mut bytes = words(&[1, 0, 0, 8, 4, 14, 2]);
    bytes.extend(*b"ideo");
    bytes.extend(*b"idtp");
    bytes.extend(words(&[1]));
    bytes.extend(*b"latn");
    bytes.extend(words(&[8, 6, 0, 0, 0, 2, 8, 12, 1, 0, 1, 1200]));
    assert_eq!(bytes.len(), 52);
    bytes
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

fn font() -> Vec<u8> {
    font_with_advance(500)
}

fn font_with_advance(glyph_advance: u16) -> Vec<u8> {
    let original = skrifa::FontRef::new(AHEM).unwrap();
    let count = original.maxp().unwrap().num_glyphs();
    assert_eq!(count, 281);
    assert_eq!(original.hhea().unwrap().number_of_h_metrics(), count);
    let glyphs = original.glyph_metrics(Size::unscaled(), LocationRef::default());
    let mut hmtx = table(AHEM, *b"hmtx").to_vec();
    for glyph in [35, 36] {
        hmtx[glyph * 4..glyph * 4 + 2].copy_from_slice(&glyph_advance.to_be_bytes());
    }
    // Keep fontwide extrema consistent with the real retained outlines and
    // changed advances, including their intentional negative right bearings.
    let mut min_right = i16::MAX;
    let mut min_bottom = i16::MAX;
    let mut max_extent = i16::MIN;
    for glyph in 0..count {
        let bounds = glyphs.bounds(GlyphId::new(u32::from(glyph))).unwrap();
        let width = bounds.x_max - bounds.x_min;
        let height = bounds.y_max - bounds.y_min;
        assert_eq!(width.fract(), 0.0);
        assert_eq!(height.fract(), 0.0);
        let at = usize::from(glyph) * 4;
        let advance = i32::from(u16_at(&hmtx, at));
        let bearing = i32::from(i16::from_be_bytes(hmtx[at + 2..at + 4].try_into().unwrap()));
        min_right = min_right.min(i16::try_from(advance - bearing - width as i32).unwrap());
        min_bottom = min_bottom.min(i16::try_from(1100 - height as i32).unwrap());
        max_extent = max_extent.max(i16::try_from(100 + height as i32).unwrap());
    }
    let mut hhea = table(AHEM, *b"hhea").to_vec();
    hhea[14..16].copy_from_slice(&min_right.to_be_bytes());
    let mut vhea = 0x0001_0000_u32.to_be_bytes().to_vec();
    for value in [1000_i16, -1000, 0] {
        vhea.extend(value.to_be_bytes());
    }
    vhea.extend(1200_u16.to_be_bytes());
    for value in [100_i16, min_bottom, max_extent, 0, 1, 0, 0, 0, 0, 0, 0] {
        vhea.extend(value.to_be_bytes());
    }
    vhea.extend(count.to_be_bytes());
    assert_eq!(vhea.len(), 36);
    let mut vmtx = Vec::new();
    for _ in 0..count {
        vmtx.extend(words(&[1200, 100]));
    }
    // Reuse this test family's SFNT assembler; do not replace Source or glyph
    // receipts. Complete its checksums for the actual registered derivative.
    let mut bytes = replace_tables(vec![
        (*b"hhea", hhea),
        (*b"hmtx", hmtx),
        (*b"vhea", vhea),
        (*b"vmtx", vmtx),
        (*b"BASE", vertical_base()),
        (*b"GSUB", multiple_gsub()),
    ]);
    let head_record = (0..usize::from(u16_at(&bytes, 4)))
        .map(|index| 12 + index * 16)
        .find(|&record| bytes[record..record + 4] == *b"head")
        .unwrap();
    let head = usize::try_from(u32_at(&bytes, head_record + 8)).unwrap();
    bytes[head + 8..head + 12].fill(0);
    for index in 0..usize::from(u16_at(&bytes, 4)) {
        let record = 12 + index * 16;
        let start = usize::try_from(u32_at(&bytes, record + 8)).unwrap();
        let len = usize::try_from(u32_at(&bytes, record + 12)).unwrap();
        let sum = checksum(&bytes[start..start + len]);
        bytes[record + 4..record + 8].copy_from_slice(&sum.to_be_bytes());
    }
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&bytes));
    bytes[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    bytes
}

fn premises(bytes: &[u8]) {
    premises_with_advance(bytes, 500);
}

fn premises_with_advance(bytes: &[u8], glyph_advance: u16) {
    assert_eq!(checksum(bytes), 0xb1b0_afba);
    for tag in [*b"name", *b"cmap", *b"glyf", *b"loca", *b"maxp", *b"OS/2"] {
        assert_eq!(table(bytes, tag), table(AHEM, tag));
    }
    assert_eq!(table(bytes, *b"GSUB"), multiple_gsub());
    let face = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    let hhea = face.hhea().unwrap();
    assert_eq!(
        (hhea.ascender().to_i16(), hhea.descender().to_i16()),
        (800, -200)
    );
    assert_eq!(face.charmap().map('A'), Some(GlyphId::new(35)));
    assert_eq!(face.vhea().unwrap().number_of_long_ver_metrics(), 281);
    let metrics = face.glyph_metrics(Size::unscaled(), LocationRef::default());
    for glyph in [35, 36] {
        assert_eq!(
            metrics.advance_width(GlyphId::new(glyph)),
            Some(f32::from(glyph_advance))
        );
        let bounds = metrics.bounds(GlyphId::new(glyph)).unwrap();
        assert_eq!(
            [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
            [0.0, -200.0, 1000.0, 800.0]
        );
        let row = face.vmtx().unwrap().v_metrics()[glyph as usize];
        assert_eq!((row.advance.get(), row.side_bearing.get()), (1200, 100));
    }
    let base = face.base().unwrap();
    let axis = base.vert_axis().unwrap().unwrap();
    assert_eq!(
        axis.base_tag_list()
            .unwrap()
            .unwrap()
            .baseline_tags()
            .iter()
            .map(|tag| tag.get().to_be_bytes())
            .collect::<Vec<_>>(),
        [*b"ideo", *b"idtp"]
    );
    let scripts = axis.base_script_list().unwrap();
    assert_eq!(scripts.base_script_records().len(), 1);
    let script = &scripts.base_script_records()[0];
    assert_eq!(script.base_script_tag().to_be_bytes(), *b"latn");
    let values = script
        .base_script(scripts.offset_data())
        .unwrap()
        .base_values()
        .unwrap()
        .unwrap();
    assert_eq!(
        values
            .base_coords()
            .iter()
            .map(|value| value.unwrap().coordinate())
            .collect::<Vec<_>>(),
        [0, 1200]
    );
}

fn capture(
    input: &str,
    cell_range: text::Range,
    letter_spacing: f64,
) -> Arc<text::ProcessedSource> {
    let parent = text::Style {
        size: 10.0,
        line_height: text::LineHeight::Absolute(12.0),
        white_space: text::WhiteSpace::Preserve,
        text_combine_upright: text::TextCombineUpright::None,
        text_transform: text::TextTransform::none(),
        letter_spacing: 0.0,
        wrap: text::Wrap::Word,
        ..style()
    };
    let body = text::Style {
        line_height: text::LineHeight::Absolute(10.0),
        text_combine_upright: text::TextCombineUpright::All,
        letter_spacing,
        ..parent.clone()
    };
    let reference = text::ScopeFontReference {
        script: text::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: text::FontBaselineAxis::VerticalX,
        baseline: text::FontBaseline::Central,
    };
    let len = input.encode_utf16().count();
    assert!(cell_range.start < cell_range.end && cell_range.end <= len);
    let ranges = [
        text::Range::new(0, cell_range.start),
        cell_range,
        text::Range::new(cell_range.end, len),
    ]
    .into_iter()
    .filter(|range| range.start < range.end)
    .collect::<Vec<_>>();
    let mut source = text::Source::identified(input, text::Id::from_u64(784_915), 1);
    let mut events = vec![
        text::SourceProcessingEvent::EnterScope(0),
        text::SourceProcessingEvent::EnterParagraph(0),
    ];
    for (index, &range) in ranges.iter().enumerate() {
        let inside = range == cell_range;
        source.span(range, if inside { body.clone() } else { parent.clone() });
        if index != 0 {
            events.push(text::SourceProcessingEvent::Connection(index - 1));
        }
        if inside {
            events.push(text::SourceProcessingEvent::EnterScope(1));
        }
        events.push(text::SourceProcessingEvent::Origin {
            origin: index,
            affinity: text::Affinity::After,
        });
        if inside {
            events.push(text::SourceProcessingEvent::ExitScope(1));
        }
    }
    events.extend([
        text::SourceProcessingEvent::ExitParagraph(0),
        text::SourceProcessingEvent::ExitScope(0),
    ]);
    Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: ranges
                .iter()
                .map(|&range| text::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: range,
                    aggregate: range,
                    kind: text::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..ranges.len() - 1)
                .map(|index| text::SourceProcessingConnection {
                    before_origin: index,
                    after_origin: index + 1,
                    kind: text::ParagraphConnectionKind::Continuous,
                })
                .collect(),
            controls: vec![],
            boundary_policies: (0..ranges.len() - 1)
                .map(|index| text::ParagraphBoundaryPolicy {
                    definition: text::ParagraphBoundaryDefinition::Connection(index),
                    preceding_wrap: text::Wrap::Word,
                    following_wrap: text::Wrap::Word,
                    ancestor_wrap: text::Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows: vec![],
            scope_alignment_limits: Some(text::ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            scopes: vec![
                text::SourceProcessingScope::Block {
                    topology: text::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: parent,
                    used_direction: text::ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(text::SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(text::ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(reference),
                },
                text::SourceProcessingScope::Inline {
                    trim: Default::default(),
                    style: Some(body),
                    boundaries: None,
                    alignment_boundary: None,
                    font_reference: Some(reference),
                    horizontal_inner_direction: Some(text::ParagraphBaseDirection::LeftToRight),
                },
            ],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::AllocatedContributions,
            }],
            events,
        })
        .unwrap(),
    )
}

#[test]
fn two_glyph_combined_cell_uses_horizontal_pens_inside_vertical_line() {
    let bytes = font();
    premises(&bytes);
    let processed = capture("A", text::Range::new(0, 1), 0.0);
    assert_eq!(processed.source().text(), &text::Text::from("A"));
    let domain = text::AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::Mixed,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = domain.outputs().next().unwrap();
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(analyzed.combined_cells().len(), 1);
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(
        cell.parent_scope(),
        Some(processed.captured_scope(0).unwrap())
    );
    assert_eq!(cell.selection_unit_count(), 1);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41]);
    assert_eq!(cell.presentation_code_points().collect::<Vec<_>>(), [0x41]);
    let piece = cell.source_pieces().next().unwrap();
    assert_eq!(piece.source(), processed.source());
    assert_eq!(piece.aggregate(), text::Range::new(0, 1));
    assert_eq!(
        piece.origin_position(),
        processed.captured_origin(0).unwrap()
    );
    let mut system = system(bytes.clone());
    assert_eq!(system.registered_font_faces().count(), 1);
    let prepared = system
        .prepare(
            analyzed,
            text::PreparationOptions::default().with_orientation(text::LineOrientation::Vertical),
        )
        .unwrap();
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    let selected = prepared
        .select(text::LineSelectionRequest {
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants: prepared.participants().collect(),
            termination: text::PreparedTermination::Ordinary(segment.end_opportunity()),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: text::LineContext {
                alignment_insets: text::LineAlignmentInsets::ZERO,
                formatted_line: text::FormattedLineContext::default(),
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                line_box_direction: text::LineBoxDirection::LeftToRight,
                orientation: text::LineOrientation::Vertical,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: text::Alignment::Start,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::None,
            },
        })
        .unwrap();
    let line = selected
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: text::ScopeAllocations::None,
        })
        .unwrap();
    assert!(line.has_in_flow_content());
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(line.measurements().fit().advance(), 10.0);
    let layout = text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared: prepared.clone(),
        coverage: text::PlacementCoverage::WholeParagraph,
        entries: vec![text::PlacementEntry::Formatted(text::LinePlacement {
            line,
            anchor: text::LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: text::LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), processed.source());
    assert_eq!(layout.analyzed_source().identity(), analyzed.identity());
    assert_eq!(layout.lines().len(), 1);
    // BASE0..12 has Central6. The size10 square centers on common baseline100,
    // hence x95..105/y20..30. Horizontal hhea8/-2 puts its glyph origin at95,28.
    assert_eq!(
        layout.lines()[0].bounds(),
        text::Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    let runs = layout
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    assert_eq!(runs.len(), 1);
    let run = &runs[0];
    assert_eq!(run.orientation(), text::LineOrientation::Vertical);
    assert_eq!(run.glyph_basis(), text::GlyphBasis::Unrotated);
    assert_eq!(run.face().bytes(), bytes.as_slice());
    assert_eq!(run.range(), text::Range::new(0, 1));
    assert_eq!(run.glyph_frame().origin(), text::Point::new(95.0, 28.0));
    assert_eq!(
        run.glyph_frame().pen_direction(),
        text::Point::new(1.0, 0.0)
    );
    // This qualifies the real provider invocation before testing Render: one
    // source scalar expands through ccmp into two glyphs, with retained raw pens.
    assert_eq!(
        run.glyphs()
            .map(|glyph| (
                glyph.id(),
                glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance()
            ))
            .collect::<Vec<_>>(),
        [(35, 0.0, 0.0, 0.0, 5.0), (36, 5.0, 0.0, 0.0, 5.0)]
    );
    let before = system.stats();
    let rendered = prepare_text_layout(&layout, Transform::identity(), limits()).unwrap();
    assert_eq!(system.stats(), before);
    assert!(std::ptr::eq(rendered.layout(), &layout));
    assert_eq!(rendered.scene().commands.len(), 1);
    let (outline, transform) = outline(&rendered.scene().commands[0]);
    assert_eq!(transform, Transform::translation(95.0, 28.0).unwrap());
    assert_eq!(outline.font().bytes(), bytes.as_slice());
    assert_eq!(outline.instance().size(), 10.0);
    assert!(outline.instance().coordinates().is_empty());
    assert_eq!(outline.instance().synthesis(), GlyphSynthesis::none());
    assert_eq!(
        outline.glyphs(),
        [
            TextGlyph::try_new(35, 0.0, 0.0, 5.0).unwrap(),
            TextGlyph::try_new(36, 5.0, 0.0, 5.0).unwrap(),
        ]
    );
    // Unchanged glyph outlines span0..10 by-8..2. The second pen adds5 in X,
    // so occupied ink is15x10; the one-em advance remains10 despite overhang.
    assert_eq!(
        outline.bounds().ink_rect(),
        Some(Rect::try_new(0.0, -8.0, 15.0, 10.0).unwrap())
    );
}

#[test]
fn compressed_two_glyph_cell_keeps_raw_pens_and_executes_one_horizontal_scale_in_render() {
    use kurbo::Shape as _;

    let bytes = font_with_advance(800);
    premises_with_advance(&bytes, 800);
    // The real GSUB contains normal ccmp only, so no width variant is available.
    // Two actual advances800 at size10/UPEM1000 give W16. Native compression
    // retains those normal glyphs and uses E/W=10/16=5/8, with no Y scaling.
    let processed = capture("A", text::Range::new(0, 1), 0.0);
    assert_eq!(processed.source().text(), &text::Text::from("A"));
    let domain = text::AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::Mixed,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = domain.outputs().next().unwrap();
    let mut cells = analyzed.combined_cells();
    let cell = cells.next().unwrap();
    assert!(cells.next().is_none());
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(cell.selection_unit_count(), 1);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41]);
    assert_eq!(cell.presentation_code_points().collect::<Vec<_>>(), [0x41]);
    let mut pieces = cell.source_pieces();
    let piece = pieces.next().unwrap();
    assert!(pieces.next().is_none());
    assert_eq!(piece.source(), processed.source());
    assert_eq!(piece.local(), text::Range::new(0, 1));
    assert_eq!(piece.aggregate(), text::Range::new(0, 1));
    assert_eq!(
        piece.origin_position(),
        processed.captured_origin(0).unwrap()
    );
    let mut system = system(bytes.clone());
    assert_eq!(system.registered_font_faces().count(), 1);
    let resource = system.registered_font_faces().next().unwrap().to_owned();
    let prepared = system
        .prepare(
            analyzed,
            text::PreparationOptions::default().with_orientation(text::LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    let selected = prepared
        .select(text::LineSelectionRequest {
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants: prepared.participants().collect(),
            termination: text::PreparedTermination::Ordinary(segment.end_opportunity()),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: text::LineContext {
                alignment_insets: text::LineAlignmentInsets::ZERO,
                formatted_line: text::FormattedLineContext::default(),
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                line_box_direction: text::LineBoxDirection::LeftToRight,
                orientation: text::LineOrientation::Vertical,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: text::Alignment::Start,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::None,
            },
        })
        .unwrap();
    let line = selected
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: text::ScopeAllocations::None,
        })
        .unwrap();
    assert!(line.has_in_flow_content());
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(line.measurements().fit().advance(), 10.0);
    let layout = text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared: prepared.clone(),
        coverage: text::PlacementCoverage::WholeParagraph,
        entries: vec![text::PlacementEntry::Formatted(text::LinePlacement {
            line,
            anchor: text::LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: text::LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), processed.source());
    assert_eq!(layout.analyzed_source().identity(), analyzed.identity());
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(
        layout.lines()[0].bounds(),
        text::Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    let runs = layout
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    assert_eq!(runs.len(), 1);
    let run = &runs[0];
    assert!(run.face().is_same_face(&resource.face()));
    assert_eq!(run.range(), text::Range::new(0, 1));
    assert_eq!(run.orientation(), text::LineOrientation::Vertical);
    assert_eq!(run.glyph_basis(), text::GlyphBasis::Unrotated);
    let frame = run.glyph_frame();
    assert_eq!(frame.origin(), text::Point::new(95.0, 28.0));
    assert_eq!(frame.pen_direction(), text::Point::new(1.0, 0.0));
    assert_eq!(frame.outline_x(), text::Point::new(0.625, 0.0));
    assert_eq!(frame.outline_y(), text::Point::new(0.0, 1.0));
    assert_eq!(
        run.glyphs()
            .map(|glyph| (
                glyph.id(),
                glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance()
            ))
            .collect::<Vec<_>>(),
        [(35, 0.0, 0.0, 0.0, 8.0), (36, 8.0, 0.0, 0.0, 8.0)]
    );
    drop(resource);
    drop(system);
    // Caller translation is applied after the real cell frame exactly once.
    // These are headless Scene admissions; neither case creates a GPU renderer.
    for (placement, matrix, physical_ink) in [
        (
            Transform::identity(),
            [0.625, 0.0, 0.0, 1.0, 95.0, 28.0],
            kurbo::Rect::new(95.0, 20.0, 106.25, 30.0),
        ),
        (
            Transform::translation(7.0, -3.0).unwrap(),
            [0.625, 0.0, 0.0, 1.0, 102.0, 25.0],
            kurbo::Rect::new(102.0, 17.0, 113.25, 27.0),
        ),
    ] {
        let rendered = prepare_text_layout(&layout, placement, limits()).unwrap();
        assert!(std::ptr::eq(rendered.layout(), &layout));
        assert_eq!(rendered.scene().commands.len(), 1);
        let (outline, transform) = outline(&rendered.scene().commands[0]);
        assert_eq!(transform.as_array(), matrix);
        assert_eq!(outline.font().bytes(), bytes.as_slice());
        assert_eq!(outline.instance().size(), 10.0);
        assert!(outline.instance().coordinates().is_empty());
        assert_eq!(outline.instance().synthesis(), GlyphSynthesis::none());
        assert_eq!(
            outline.glyphs(),
            [
                TextGlyph::try_new(35, 0.0, 0.0, 8.0).unwrap(),
                TextGlyph::try_new(36, 8.0, 0.0, 8.0).unwrap(),
            ]
        );
        // Actual local outlines still span0..18 by-8..2. X compression makes
        // physical ink11.25x10; neither of those ink widths is outer advanceE10.
        assert_eq!(
            outline.bounds().ink_rect(),
            Some(Rect::try_new(0.0, -8.0, 18.0, 10.0).unwrap())
        );
        let mut path = outline.path().clone();
        path.apply_affine(kurbo::Affine::new(transform.as_array()));
        assert_eq!(path.bounding_box(), physical_ink);
    }
}

#[test]
fn tracked_compressed_cell_keeps_its_fixed_square_and_frame_at_a_nonzero_flow_offset() {
    use kurbo::Shape as _;

    let bytes = font_with_advance(800);
    premises_with_advance(&bytes, 800);
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.charmap().map('C'), Some(GlyphId::new(37)));
    assert_eq!(
        face.glyph_metrics(Size::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(37)),
        Some(1000.0)
    );
    // Only A expands through ccmp. C retains its original1000 design advance;
    // the two inner glyphs retain800 each, requiring10/16 horizontal compression.
    let processed = capture("CAC", text::Range::new(1, 2), 2.0);
    assert_eq!(processed.source().text(), &text::Text::from("CAC"));
    assert_eq!(processed.processed_text(), &text::Text::from("CAC"));
    assert_eq!(
        processed
            .source()
            .spans()
            .iter()
            .map(|span| (
                span.range(),
                span.style().text_combine_upright,
                span.style().letter_spacing,
            ))
            .collect::<Vec<_>>(),
        [
            (text::Range::new(0, 1), text::TextCombineUpright::None, 0.0),
            (text::Range::new(1, 2), text::TextCombineUpright::All, 2.0),
            (text::Range::new(2, 3), text::TextCombineUpright::None, 0.0),
        ]
    );
    let domain = text::AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::Mixed,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = domain.outputs().next().unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: text::Wrap::Word,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut source_cells = analyzed.combined_cells();
    let source_cell = source_cells.next().unwrap();
    assert!(source_cells.next().is_none());
    assert_eq!(source_cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(source_cell.selection_unit_count(), 1);
    assert_eq!(source_cell.used_code_points().collect::<Vec<_>>(), [0x41]);
    let mut pieces = source_cell.source_pieces();
    let piece = pieces.next().unwrap();
    assert!(pieces.next().is_none());
    assert_eq!(piece.source(), processed.source());
    assert_eq!(piece.local(), text::Range::new(1, 2));
    assert_eq!(piece.aggregate(), text::Range::new(1, 2));
    assert_eq!(
        piece.origin_position(),
        processed.captured_origin(1).unwrap()
    );

    let mut system = system(bytes.clone());
    let prepared = system
        .prepare(
            &analyzed,
            text::PreparationOptions::default().with_orientation(text::LineOrientation::Vertical),
        )
        .unwrap();
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    let selected = prepared
        .select(text::LineSelectionRequest {
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants: prepared.participants().collect(),
            termination: text::PreparedTermination::Ordinary(segment.end_opportunity()),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: text::LineContext {
                alignment_insets: text::LineAlignmentInsets::ZERO,
                formatted_line: text::FormattedLineContext::default(),
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                line_box_direction: text::LineBoxDirection::LeftToRight,
                orientation: text::LineOrientation::Vertical,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: text::Alignment::Start,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::None,
            },
        })
        .unwrap();
    let line = selected
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: text::ScopeAllocations::None,
        })
        .unwrap();
    // Text3 half tracking: C10 + gap1 + square10 + gap1 + C10 =32.
    // Tracking is outside the square; neither its width nor the inner W16 changes.
    assert_eq!(line.feedback().advance, 32.0);
    assert_eq!(line.measurements().fit().advance(), 32.0);
    assert_eq!(line.measurements().alignment().advance(), 32.0);
    {
        let mut cells = line.cell_views();
        let cell = cells.next().unwrap();
        assert!(cells.next().is_none());
        assert_eq!(
            cell.square(),
            text::Rect::new(11.0, cell.reference() - 5.0, 10.0, 10.0)
        );
        assert_eq!(cell.scope_displacement().get(), 0.0);
        assert_eq!(
            cell.source_cell().scope(),
            processed.captured_scope(1).unwrap()
        );
        let mut runs = line.run_views().filter(|run| run.cell().is_some());
        let run = runs.next().unwrap();
        assert!(runs.next().is_none());
        assert_eq!(run.range(), text::Range::new(1, 2));
    }
    let layout = text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared: prepared.clone(),
        coverage: text::PlacementCoverage::WholeParagraph,
        entries: vec![text::PlacementEntry::Formatted(text::LinePlacement {
            line,
            anchor: text::LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: text::LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), processed.source());
    assert_eq!(layout.analyzed_source().identity(), analyzed.identity());
    assert_eq!(layout.lines().len(), 1);
    // Physical square95..105 by31..41. The trailing caret is at flow42,
    // one unit beyond the square, while horizontal hhea8/-2 gives origin95,39.
    assert_eq!(
        layout
            .cursor(text::Cursor::new(1, text::Affinity::After))
            .unwrap()
            .rect(),
        text::Rect::new(95.0, 31.0, 10.0, 1.0)
    );
    assert_eq!(
        layout
            .cursor(text::Cursor::new(2, text::Affinity::Before))
            .unwrap()
            .rect(),
        text::Rect::new(95.0, 42.0, 10.0, 1.0)
    );
    let mut cell_runs = layout
        .run_views()
        .filter(|run| run.range() == text::Range::new(1, 2));
    let run = cell_runs.next().unwrap();
    assert!(cell_runs.next().is_none());
    assert_eq!(run.face().bytes(), bytes.as_slice());
    assert_eq!(run.glyph_basis(), text::GlyphBasis::Unrotated);
    assert_eq!(run.orientation(), text::LineOrientation::Vertical);
    let frame = run.glyph_frame();
    assert_eq!(frame.origin(), text::Point::new(95.0, 39.0));
    assert_eq!(frame.outline_x(), text::Point::new(0.625, 0.0));
    assert_eq!(frame.outline_y(), text::Point::new(0.0, 1.0));
    assert_eq!(frame.pen_direction(), text::Point::new(1.0, 0.0));
    assert_eq!(
        run.glyphs()
            .map(|glyph| (
                glyph.id(),
                glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance(),
            ))
            .collect::<Vec<_>>(),
        [(35, 0.0, 0.0, 0.0, 8.0), (36, 8.0, 0.0, 0.0, 8.0)]
    );
    let mut following = layout
        .run_views()
        .filter(|run| run.range() == text::Range::new(2, 3));
    let following = following.next().unwrap();
    let mut glyphs = following.glyphs();
    let glyph = glyphs.next().unwrap();
    assert!(glyphs.next().is_none());
    assert_eq!((glyph.id(), glyph.advance()), (37, 10.0));
    let frame = following.glyph_frame();
    let pen = frame.pen_direction();
    let x = f64::from(glyph.x()) + glyph.pen() * pen.x;
    let y = f64::from(glyph.y()) + glyph.pen() * pen.y;
    assert_eq!(
        frame.origin().y + frame.outline_x().y * x + frame.outline_y().y * y,
        42.0
    );

    let packet = [
        TextGlyph::try_new(35, 0.0, 0.0, 8.0).unwrap(),
        TextGlyph::try_new(36, 8.0, 0.0, 8.0).unwrap(),
    ];
    let before = system.stats();
    for (placement, matrix, physical_ink) in [
        (
            Transform::identity(),
            [0.625, 0.0, 0.0, 1.0, 95.0, 39.0],
            kurbo::Rect::new(95.0, 31.0, 106.25, 41.0),
        ),
        (
            Transform::translation(7.0, -3.0).unwrap(),
            [0.625, 0.0, 0.0, 1.0, 102.0, 36.0],
            kurbo::Rect::new(102.0, 28.0, 113.25, 38.0),
        ),
    ] {
        let rendered = prepare_text_layout(&layout, placement, limits()).unwrap();
        assert_eq!(system.stats(), before);
        assert!(std::ptr::eq(rendered.layout(), &layout));
        // Select by the actual two-glyph Cell packet, never scene command order.
        let mut commands = rendered.scene().commands.iter().filter(|command| {
            matches!(
                command, Command::PreparedGlyphRun { run, .. } if run.glyphs() == packet.as_slice()
            )
        });
        let command = commands.next().unwrap();
        assert!(commands.next().is_none());
        let (outline, transform) = outline(command);
        assert_eq!(transform.as_array(), matrix);
        assert_eq!(outline.font().bytes(), bytes.as_slice());
        assert_eq!(outline.instance().size(), 10.0);
        assert!(outline.instance().coordinates().is_empty());
        assert_eq!(outline.instance().synthesis(), GlyphSynthesis::none());
        assert_eq!(outline.glyphs(), packet);
        // Raw pens0/8 plus unchanged ten-unit outlines give local ink0..18.
        assert_eq!(
            outline.bounds().ink_rect(),
            Some(Rect::try_new(0.0, -8.0, 18.0, 10.0).unwrap())
        );
        let mut path = outline.path().clone();
        path.apply_affine(kurbo::Affine::new(transform.as_array()));
        assert_eq!(path.bounding_box(), physical_ink);
    }
}
