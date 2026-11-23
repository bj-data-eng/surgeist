//! Actual width-mapped font metrics through native Layout, selection and paint.

use super::*;

// Minimal consumer-owned adaptation of test_fonts/shaping.rs::width_ahem.
// Only cmap/hmtx and required SFNT checksums change; licensed Ahem outlines stay
// untouched. UPEM1000: A35=1000, FF21→36=2000, S53=1000, FF33→58=1500,
// sharp-s134=1000, SPACE3=500, IDEOGRAPHIC SPACE1=2000 (both blank),
// combining3099→100=0, matching the qualified Text-owned width fixture.
pub(super) fn width_ahem() -> Vec<u8> {
    fn u16_at(bytes: &[u8], at: usize) -> u16 {
        u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
    }
    fn u32_at(bytes: &[u8], at: usize) -> u32 {
        u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
    }
    fn checksum(bytes: &[u8]) -> u32 {
        bytes.chunks(4).fold(0_u32, |sum, chunk| {
            let mut word = [0; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            sum.wrapping_add(u32::from_be_bytes(word))
        })
    }
    fn record(bytes: &[u8], tag: &[u8; 4]) -> usize {
        (0..usize::from(u16_at(bytes, 4)))
            .map(|index| 12 + index * 16)
            .find(|at| &bytes[*at..*at + 4] == tag)
            .unwrap()
    }
    let mut bytes =
        include_bytes!("../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec();
    let mappings = [
        (0x20_u32, 3_u32),
        (0x41, 35),
        (0x53, 53),
        (0xdf, 134),
        (0x3000, 1),
        (0x3099, 100),
        (0xff21, 36),
        (0xff33, 58),
    ];
    let mut cmap = Vec::from(&[0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12][..]);
    cmap.extend(12_u16.to_be_bytes());
    cmap.extend(0_u16.to_be_bytes());
    cmap.extend(
        u32::try_from(16 + mappings.len() * 12)
            .unwrap()
            .to_be_bytes(),
    );
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(u32::try_from(mappings.len()).unwrap().to_be_bytes());
    for (scalar, glyph) in mappings {
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(glyph.to_be_bytes());
    }
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    let cmap_at = u32::try_from(bytes.len()).unwrap();
    let entry = record(&bytes, b"cmap");
    bytes[entry + 4..entry + 8].copy_from_slice(&checksum(&cmap).to_be_bytes());
    bytes[entry + 8..entry + 12].copy_from_slice(&cmap_at.to_be_bytes());
    bytes[entry + 12..entry + 16]
        .copy_from_slice(&u32::try_from(cmap.len()).unwrap().to_be_bytes());
    bytes.extend(cmap);
    let entry = record(&bytes, b"hmtx");
    let at = usize::try_from(u32_at(&bytes, entry + 8)).unwrap();
    let length = usize::try_from(u32_at(&bytes, entry + 12)).unwrap();
    for (glyph, advance) in [(1, 2000_u16), (3, 500), (36, 2000), (58, 1500), (100, 0)] {
        bytes[at + glyph * 4..at + glyph * 4 + 2].copy_from_slice(&advance.to_be_bytes());
    }
    let sum = checksum(&bytes[at..at + length]);
    bytes[entry + 4..entry + 8].copy_from_slice(&sum.to_be_bytes());
    let entry = record(&bytes, b"head");
    let at = usize::try_from(u32_at(&bytes, entry + 8)).unwrap();
    let length = usize::try_from(u32_at(&bytes, entry + 12)).unwrap();
    assert_eq!(u16_at(&bytes, at + 18), 1000);
    bytes[at + 8..at + 12].fill(0);
    let sum = checksum(&bytes[at..at + length]);
    bytes[entry + 4..entry + 8].copy_from_slice(&sum.to_be_bytes());
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&bytes));
    bytes[at + 8..at + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&bytes), 0xb1b0_afba);
    bytes
}

fn system(bytes: &[u8]) -> text::System {
    let mut system = text::System::default();
    system
        .register_fonts(bytes.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    system
}

fn capture(
    system: &mut text::System,
    value: &str,
    transform: text::style::TextTransform,
    white_space: text::style::WhiteSpace,
    word_break: text::style::WordBreak,
    authored_width: Option<f32>,
    request: layout::request::LayoutRootRequest,
) -> (
    layout::batch::CompletedLayoutBatch<usize>,
    text::prepared::PreparedOrigin,
    text::source::Source,
) {
    let source = text::source::Source::identified(value, text::source::Id::from_u64(6304), 17);
    let range = text::source::Range::new(0, source.text().len());
    let style = text::style::Style {
        font: text::style::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::style::LineHeight::Absolute(10.0),
        white_space,
        word_break,
        text_transform: transform,
        ..text::style::Style::default()
    };
    let ancestor_wrap = style.wrap;
    let analyzed = text::analyzed::AnalyzedSource::from_paragraph(
        text::paragraph::ParagraphInput {
            source: source.clone(),
            base_style: style,
            segment_breaks: text::processing::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::paragraph::ParagraphStrut::DefaultFont,
            terminal_edge: text::paragraph::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![text::paragraph::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: range,
                aggregate: range,
                kind: text::paragraph::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        text::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    // These witnesses have no phase-I deletion/conversion: equal processed
    // strings still differ in the Preserve versus Collapse fact.
    assert_eq!(
        analyzed.processed_source().processed_text().code_units(),
        source.text().code_units()
    );
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::analyzed::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(&analyzed, text::prepared::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let tree = Tree {
        inputs: [
            layout::node::LayoutInput::box_input(layout::node::NodeInput {
                display: layout::box_model::Display::Block,
                size: layout::geometry::Size::new(
                    match authored_width {
                        Some(width) => layout::sizing::PreferredSizeOf::value(
                            layout::length::LengthPercentageOf::px(width).unwrap(),
                        ),
                        None => layout::sizing::PreferredSizeOf::AUTO,
                    },
                    layout::sizing::PreferredSizeOf::AUTO,
                ),
                ..layout::node::NodeInput::DEFAULT
            }),
            layout::node::LayoutInput::InlineText(layout::text_layout::TextParticipation::new(
                origin.clone(),
            )),
        ],
        non_box: layout::node::NodeInput::non_box(),
    };
    let capture = layout::text_layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::text_layout::TextParagraphBinding {
            inline_box_fragments: layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
            prepared,
            container: 0,
            nodes: vec![1],
            origins: vec![layout::text_layout::TextOriginBinding {
                origin: origin.clone(),
                node: 1,
            }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        surgeist_layout::inline_wrapping::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(&capture, 0, request).unwrap();
    (batch, origin, source)
}

fn glyphs(placed: &text::layout::Layout, bytes: &[u8]) -> Vec<(u32, f32)> {
    placed
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn advances(batch: &layout::batch::CompletedLayoutBatch<usize>) -> Vec<f64> {
    batch
        .final_inline_fragments()
        .iter()
        .map(|entry| {
            let fragment = entry.fragment();
            (
                fragment.line_index(),
                fragment
                    .finalized_fragment()
                    .finalized_line()
                    .feedback()
                    .advance,
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>()
        .into_values()
        .collect()
}

fn intrinsic_width(batch: &layout::batch::CompletedLayoutBatch<usize>) -> f32 {
    batch
        .node_layout(0)
        .unwrap()
        .single_box()
        .expect("fixture node produces one box")
        .final_output()
        .size
        .width
}

#[test]
fn width_maps_real_glyph_advance_without_expanding_ink_or_original_selection() {
    let bytes = width_ahem();
    let mut system = system(&bytes);
    for enabled in [false, true, false] {
        let (batch, origin, source) = capture(
            &mut system,
            "A",
            text::style::TextTransform::Set(
                text::style::TextTransformSet::none().with_full_width(enabled),
            ),
            text::style::WhiteSpace::Preserve,
            text::style::WordBreak::Normal,
            Some(30.0),
            NativeViewport::new(
                layout::geometry::Size::new(Some(30.0), None),
                layout::geometry::FlowAxes::new(
                    layout::geometry::WritingMode::HorizontalTb,
                    layout::geometry::Direction::Ltr,
                ),
            )
            .request(layout::geometry::Size::new(
                layout::constraint::SizingOperation::Normal,
                layout::constraint::SizingOperation::Normal,
            )),
        );
        let placed = batch.placed_text()[0].layout();
        let advance = if enabled { 20.0 } else { 10.0 };
        assert_eq!(placed.source(), &source);
        assert_eq!(source.text().code_units(), [0x41]);
        assert_eq!(source.id(), Some(text::source::Id::from_u64(6304)));
        assert_eq!(source.revision(), 17);
        assert_eq!(
            glyphs(placed, &bytes),
            [(if enabled { 36 } else { 35 }, advance as f32)]
        );
        assert_eq!(advances(&batch), [advance]);
        let position_at = |x| {
            placed
                .positions()
                .find(|position| {
                    placed.position_geometry(position).unwrap().rect().origin
                        == text::geometry::Point::new(x, 0.0)
                })
                .unwrap()
        };
        let begin = position_at(0.0);
        let end = position_at(advance);
        for (a, b) in [(&begin, &end), (&end, &begin)] {
            let selection = placed.position_selection(a, b).unwrap();
            assert_eq!(selection.source_text().code_units(), [0x41]);
            assert_eq!(selection.plaintext().code_units(), [0x41]);
            assert_eq!(selection.source_pieces().len(), 1);
            assert_eq!(selection.source_pieces()[0].origin(), &origin);
            assert_eq!(
                selection.source_pieces()[0].local_range(),
                text::source::Range::new(0, 1)
            );
            assert_eq!(
                selection.geometry().rects()[0].rect(),
                text::geometry::Rect::new(0.0, 0.0, advance, 10.0)
            );
        }
        let fragments = placed
            .source_range_fragments(&origin, text::source::Range::new(0, 1))
            .unwrap();
        assert_eq!(
            fragments
                .units()
                .next()
                .unwrap()
                .pieces()
                .next()
                .unwrap()
                .rect(),
            text::geometry::Rect::new(0.0, 0.0, advance, 10.0)
        );
        let font = render::text::FontData::try_from_bytes(
            bytes.clone(),
            0,
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
        let instance =
            render::text::TextInstance::try_new(10.0, vec![], render::text::GlyphSynthesis::none())
                .unwrap();
        let actual = placed
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|glyph| {
                        render::text::TextGlyph::try_new(
                            glyph.id(),
                            glyph.pen() + f64::from(glyph.x()),
                            f64::from(glyph.y()),
                            glyph.advance(),
                        )
                        .unwrap()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let retained = font
            .prepare_glyph_run(
                &instance,
                &actual,
                render::text::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
                &mut surgeist_render::accounting::RenderAccountingContext::default(),
            )
            .unwrap();
        // The retained Ahem square is10px wide even when its hmtx advance is20.
        assert_eq!(retained.bounds().ink_rect().unwrap().width(), 10.0);
        let paint = render::text_layout::prepare_text_layout(
            placed,
            render::geometry::Transform::identity(),
            render::text_layout::TextLayoutLimits::new(
                render::text::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
                8,
                32,
                8,
            ),
            &mut surgeist_render::accounting::RenderAccountingContext::default(),
        )
        .unwrap();
        assert!(std::ptr::eq(paint.layout(), placed));
        assert_eq!(paint.layout().source(), &source);
        assert!(!paint.scene().is_empty());
    }
}

#[test]
fn case_then_width_expansion_drives_fit_intrinsics_and_receiving_original_custody() {
    let bytes = width_ahem();
    let mut system = system(&bytes);
    for (case, width, expected_glyphs, expected_advances, min, max) in [
        (
            text::style::CaseTransform::None,
            false,
            vec![(134, 10.0)],
            vec![10.0],
            10.0,
            10.0,
        ),
        (
            text::style::CaseTransform::Uppercase,
            false,
            vec![(53, 10.0); 2],
            vec![10.0; 2],
            10.0,
            20.0,
        ),
        (
            text::style::CaseTransform::Uppercase,
            true,
            vec![(58, 15.0); 2],
            vec![15.0; 2],
            15.0,
            30.0,
        ),
    ] {
        let transform = text::style::TextTransform::Set(
            text::style::TextTransformSet::none()
                .with_case(case)
                .with_full_width(width),
        );
        let (batch, origin, source) = capture(
            &mut system,
            "ß",
            transform,
            text::style::WhiteSpace::Preserve,
            text::style::WordBreak::BreakAll,
            Some(16.0),
            NativeViewport::new(
                layout::geometry::Size::new(Some(16.0), None),
                layout::geometry::FlowAxes::new(
                    layout::geometry::WritingMode::HorizontalTb,
                    layout::geometry::Direction::Ltr,
                ),
            )
            .request(layout::geometry::Size::new(
                layout::constraint::SizingOperation::Normal,
                layout::constraint::SizingOperation::Normal,
            )),
        );
        let placed = batch.placed_text()[0].layout();
        assert_eq!(glyphs(placed, &bytes), expected_glyphs);
        assert_eq!(advances(&batch), expected_advances);
        assert_eq!(source.text().code_units(), [0xdf]);
        assert!(
            placed
                .lines()
                .iter()
                .all(|line| line.range() == text::source::Range::new(0, 1))
        );
        if width {
            let at = |x, y| {
                placed
                    .positions()
                    .find(|position| {
                        placed.position_geometry(position).unwrap().rect().origin
                            == text::geometry::Point::new(x, y)
                    })
                    .unwrap()
            };
            let receiving = at(0.0, 10.0);
            let end = at(15.0, 10.0);
            for (a, b) in [(&receiving, &end), (&end, &receiving)] {
                let selected = placed.position_selection(a, b).unwrap();
                assert_eq!(selected.source_text().code_units(), [0xdf]);
                assert_eq!(selected.plaintext().code_units(), [0xdf]);
                assert_eq!(selected.source_pieces().len(), 1);
                assert_eq!(selected.source_pieces()[0].origin(), &origin);
                assert_eq!(
                    selected.source_pieces()[0].aggregate_range(),
                    text::source::Range::new(0, 1)
                );
                assert_eq!(
                    selected.geometry().rects()[0].rect(),
                    text::geometry::Rect::new(0.0, 10.0, 15.0, 10.0)
                );
            }
            let range = placed
                .source_range_fragments(&origin, text::source::Range::new(0, 1))
                .unwrap();
            let mut rectangles = range
                .units()
                .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
                .collect::<Vec<_>>();
            rectangles.sort_by(|a, b| a.origin.y.total_cmp(&b.origin.y));
            assert_eq!(
                rectangles,
                [
                    text::geometry::Rect::new(0.0, 0.0, 15.0, 10.0),
                    text::geometry::Rect::new(0.0, 10.0, 15.0, 10.0)
                ]
            );
        }
        for (purpose, expected) in [
            (layout::constraint::SizingOperation::MinContent, min),
            (layout::constraint::SizingOperation::MaxContent, max),
        ] {
            let (intrinsic, _, _) = capture(
                &mut system,
                "ß",
                transform,
                text::style::WhiteSpace::Preserve,
                text::style::WordBreak::BreakAll,
                None,
                NativeViewport::new(
                    layout::geometry::Size::new(None, None),
                    layout::geometry::FlowAxes::new(
                        layout::geometry::WritingMode::HorizontalTb,
                        layout::geometry::Direction::Ltr,
                    ),
                )
                .request(layout::geometry::Size::new(
                    purpose,
                    layout::constraint::SizingOperation::Normal,
                )),
            );
            assert_eq!(intrinsic_width(&intrinsic), expected);
        }
    }
}

#[test]
fn normal_wrap_uses_mapped_character_opportunities_and_real_twenty_pixel_advances() {
    let bytes = width_ahem();
    let mut system = system(&bytes);
    // Ordinary Latin AA is one20px word. Fullwidth letters have normal
    // ideographic opportunities: two20px glyphs cannot share a25px line.
    for enabled in [false, true] {
        let transform = text::style::TextTransform::Set(
            text::style::TextTransformSet::none().with_full_width(enabled),
        );
        let (batch, _, source) = capture(
            &mut system,
            "AA",
            transform,
            text::style::WhiteSpace::Preserve,
            text::style::WordBreak::Normal,
            Some(25.0),
            NativeViewport::new(
                layout::geometry::Size::new(Some(25.0), None),
                layout::geometry::FlowAxes::new(
                    layout::geometry::WritingMode::HorizontalTb,
                    layout::geometry::Direction::Ltr,
                ),
            )
            .request(layout::geometry::Size::new(
                layout::constraint::SizingOperation::Normal,
                layout::constraint::SizingOperation::Normal,
            )),
        );
        let placed = batch.placed_text()[0].layout();
        assert_eq!(
            advances(&batch),
            if enabled {
                vec![20.0, 20.0]
            } else {
                vec![20.0]
            }
        );
        assert_eq!(
            placed
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            if enabled {
                vec![
                    text::source::Range::new(0, 1),
                    text::source::Range::new(1, 2),
                ]
            } else {
                vec![text::source::Range::new(0, 2)]
            }
        );
        assert_eq!(placed.source(), &source);
        for (purpose, expected) in [
            (layout::constraint::SizingOperation::MinContent, 20.0),
            (
                layout::constraint::SizingOperation::MaxContent,
                if enabled { 40.0 } else { 20.0 },
            ),
        ] {
            let (intrinsic, _, _) = capture(
                &mut system,
                "AA",
                transform,
                text::style::WhiteSpace::Preserve,
                text::style::WordBreak::Normal,
                None,
                NativeViewport::new(
                    layout::geometry::Size::new(None, None),
                    layout::geometry::FlowAxes::new(
                        layout::geometry::WritingMode::HorizontalTb,
                        layout::geometry::Direction::Ltr,
                    ),
                )
                .request(layout::geometry::Size::new(
                    purpose,
                    layout::constraint::SizingOperation::Normal,
                )),
            );
            assert_eq!(intrinsic_width(&intrinsic), expected);
        }
    }
}

#[test]
fn preserved_space_maps_but_surviving_collapsed_space_stays_narrow() {
    let bytes = width_ahem();
    let mut system = system(&bytes);
    // Interior spaces avoid conditional trailing-space measurement effects.
    for (value, enabled, mode, expected, width) in [
        (
            "A A",
            false,
            text::style::WhiteSpace::Preserve,
            vec![(35, 10.0), (3, 5.0), (35, 10.0)],
            25.0,
        ),
        (
            "A\u{3000}A",
            false,
            text::style::WhiteSpace::Preserve,
            vec![(35, 10.0), (1, 20.0), (35, 10.0)],
            40.0,
        ),
        (
            "A A",
            true,
            text::style::WhiteSpace::Preserve,
            vec![(36, 20.0), (1, 20.0), (36, 20.0)],
            60.0,
        ),
        (
            "A A",
            true,
            text::style::WhiteSpace::Collapse,
            vec![(36, 20.0), (3, 5.0), (36, 20.0)],
            45.0,
        ),
    ] {
        let (batch, origin, source) = capture(
            &mut system,
            value,
            text::style::TextTransform::Set(
                text::style::TextTransformSet::none().with_full_width(enabled),
            ),
            mode,
            text::style::WordBreak::Normal,
            Some(100.0),
            NativeViewport::new(
                layout::geometry::Size::new(Some(100.0), None),
                layout::geometry::FlowAxes::new(
                    layout::geometry::WritingMode::HorizontalTb,
                    layout::geometry::Direction::Ltr,
                ),
            )
            .request(layout::geometry::Size::new(
                layout::constraint::SizingOperation::Normal,
                layout::constraint::SizingOperation::Normal,
            )),
        );
        let placed = batch.placed_text()[0].layout();
        assert_eq!(glyphs(placed, &bytes), expected);
        assert_eq!(advances(&batch), [width]);
        let range = placed
            .source_range_fragments(&origin, text::source::Range::new(1, 2))
            .unwrap();
        let rect = range
            .units()
            .next()
            .unwrap()
            .pieces()
            .next()
            .unwrap()
            .rect();
        assert_eq!(rect.origin.x, if enabled { 20.0 } else { 10.0 });
        assert_eq!(
            rect.size.width,
            if value.contains('\u{3000}') || (enabled && mode == text::style::WhiteSpace::Preserve)
            {
                20.0
            } else {
                5.0
            }
        );
        assert_eq!(placed.source(), &source);
        let blank = placed
            .run_views()
            .flat_map(|run| run.glyphs().collect::<Vec<_>>())
            .find(|glyph| matches!(glyph.id(), 1 | 3))
            .unwrap();
        let font = render::text::FontData::try_from_bytes(
            bytes.clone(),
            0,
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
        let instance =
            render::text::TextInstance::try_new(10.0, vec![], render::text::GlyphSynthesis::none())
                .unwrap();
        let blank = render::text::TextGlyph::try_new(
            blank.id(),
            blank.pen() + f64::from(blank.x()),
            f64::from(blank.y()),
            blank.advance(),
        )
        .unwrap();
        let retained = font
            .prepare_glyph_run(
                &instance,
                &[blank],
                render::text::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
                &mut surgeist_render::accounting::RenderAccountingContext::default(),
            )
            .unwrap();
        assert_eq!(
            retained.bounds().kind(),
            render::text::TextRunBoundsKind::Empty
        );
        let paint = render::text_layout::prepare_text_layout(
            placed,
            render::geometry::Transform::identity(),
            render::text_layout::TextLayoutLimits::new(
                render::text::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
                8,
                32,
                8,
            ),
            &mut surgeist_render::accounting::RenderAccountingContext::default(),
        )
        .unwrap();
        assert!(std::ptr::eq(paint.layout(), placed));
        assert!(!paint.scene().is_empty());
    }
}

#[cfg(feature = "text-accessibility")]
#[test]
fn width_accessibility_keeps_original_sharp_s_with_actual_thirty_pixel_geometry() {
    let bytes = width_ahem();
    let mut system = system(&bytes);
    let (batch, _, _) = capture(
        &mut system,
        "ß",
        text::style::TextTransform::Set(
            text::style::TextTransformSet::none()
                .with_case(text::style::CaseTransform::Uppercase)
                .with_full_width(true),
        ),
        text::style::WhiteSpace::Preserve,
        text::style::WordBreak::Normal,
        Some(40.0),
        NativeViewport::new(
            layout::geometry::Size::new(Some(40.0), None),
            layout::geometry::FlowAxes::new(
                layout::geometry::WritingMode::HorizontalTb,
                layout::geometry::Direction::Ltr,
            ),
        )
        .request(layout::geometry::Size::new(
            layout::constraint::SizingOperation::Normal,
            layout::constraint::SizingOperation::Normal,
        )),
    );
    let placed = batch.placed_text()[0].layout();
    drop(system);
    let mut accessibility = text::accessibility::Accessibility::new(20);
    let parent = 1000_u64.into();
    let update = placed
        .accessibility_update(
            &mut accessibility,
            parent,
            parent,
            text::geometry::Point::new(100.0, 200.0),
        )
        .unwrap();
    let position = placed
        .access_position(
            &accessibility,
            text::layout::Cursor::new(0, text::layout::Affinity::After),
        )
        .unwrap()
        .unwrap();
    let node = &update
        .nodes
        .iter()
        .find(|(id, _)| *id == position.node)
        .unwrap()
        .1;
    assert_eq!(node.value(), Some("ß"));
    assert_eq!(node.character_lengths(), [2]);
    let bounds = node.bounds().unwrap();
    assert_eq!(
        (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
        (100.0, 200.0, 130.0, 210.0)
    );
}
