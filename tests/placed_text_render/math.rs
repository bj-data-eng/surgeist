//! Complete original node facts drive native math-auto shaping and consumers.

use super::*;

// Consumer-owned finite adaptation of Text's licensed math_ahem fixture.
// Only cmap/hmtx/checksums change; Ahem outlines remain untouched. At UPEM1000,
// original A/c/h/03F4 advance1000; targets 1D434/1D450/210E/1D6F3 advance
// 2000/1500/1800/1400 respectively. These are font facts, not math metrics.
fn math_ahem() -> Vec<u8> {
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
        (0x63, 69),
        (0x68, 74),
        (0x3f4, 68),
        (0x210e, 100),
        (0x1d434, 36),
        (0x1d450, 82),
        (0x1d6f3, 37),
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
    for (glyph, advance) in [
        (3, 500_u16),
        (36, 2000),
        (37, 1400),
        (82, 1500),
        (100, 1800),
    ] {
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
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    system
}

fn style(math: bool, white_space: text::WhiteSpace, word_break: text::WordBreak) -> text::Style {
    text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space,
        word_break,
        text_transform: if math {
            text::TextTransform::MathAuto
        } else {
            text::TextTransform::none()
        },
        ..text::Style::default()
    }
}

fn source(value: &str) -> text::Source {
    text::Source::identified(value, text::Id::from_u64(6305), 19)
}

fn origin(
    source: &text::Source,
    aggregate: text::Range,
    complete: &str,
    contribution: text::Range,
) -> text::ParagraphOrigin {
    text::ParagraphOrigin {
        source: source.clone(),
        local: aggregate,
        aggregate,
        kind: text::ParagraphOriginKind::Authored,
        text_node: Some(
            text::CompleteTextNodeContribution::new(text::Text::from(complete), contribution)
                .unwrap(),
        ),
    }
}

struct MathTree {
    inputs: Vec<layout::LayoutInput>,
    non_box: layout::NodeInput,
}

impl layout::Traverse for MathTree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::ops::Range<usize>;
    fn children(&self, node: usize) -> Self::Children<'_> {
        if node == 0 {
            1..self.inputs.len()
        } else {
            0..0
        }
    }
    fn child_count(&self, node: usize) -> usize {
        if node == 0 { self.inputs.len() - 1 } else { 0 }
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}
impl layout::LayoutTree for MathTree {
    type MeasureError = Infallible;
    fn node_input(&self, node: usize) -> &layout::NodeInput {
        self.inputs[node]
            .as_view()
            .as_box()
            .unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: usize) -> layout::LayoutInputView<'_> {
        self.inputs[node].as_view()
    }
}

fn capture(
    system: &mut text::System,
    source: &text::Source,
    origins: Vec<text::ParagraphOrigin>,
    style: text::Style,
    available: layout::Available,
) -> (
    layout::CompletedLayoutBatch<usize>,
    Vec<text::PreparedOrigin>,
) {
    let connections = origins
        .windows(2)
        .map(|pair| text::ParagraphConnection {
            at: pair[0].aggregate.end,
            kind: text::ParagraphConnectionKind::Continuous,
        })
        .collect();
    let ancestor_wrap = style.wrap;
    let boundary_policies = (0..origins.len().saturating_sub(1))
        .map(|index| text::ParagraphBoundaryPolicy {
            definition: text::ParagraphBoundaryDefinition::Connection(index),
            preceding_wrap: ancestor_wrap,
            following_wrap: ancestor_wrap,
            ancestor_wrap,
        })
        .collect();
    let analyzed = text::AnalyzedSource::from_paragraph(text::ParagraphInput {
        source: source.clone(),
        base_style: style,
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: text::ParagraphBaseDirection::LeftToRight,
        paragraph_strut: text::ParagraphStrut::DefaultFont,
        terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins,
        connections,
        boundary_policies,
        controls: vec![],
        replacements: vec![],
        anchor_order: vec![],
    })
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let origins = prepared
        .origin_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    let mut inputs = vec![layout::LayoutInput::box_input(layout::NodeInput {
        display: layout::Display::Block,
        size: layout::Size::new(
            match available {
                layout::Available::Definite(width) => {
                    layout::PreferredSizeOf::value(layout::LengthPercentageOf::px(width).unwrap())
                }
                _ => layout::PreferredSizeOf::AUTO,
            },
            layout::PreferredSizeOf::AUTO,
        ),
        ..layout::NodeInput::DEFAULT
    })];
    inputs.extend(origins.iter().map(|origin| {
        layout::LayoutInput::InlineText(layout::TextParticipation::new(origin.clone()))
    }));
    let tree = MathTree {
        inputs,
        non_box: layout::NodeInput::non_box(),
    };
    let capture = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::TextParagraphBinding {
            prepared,
            container: 0,
            nodes: (1..=origins.len()).collect(),
            origins: origins
                .iter()
                .enumerate()
                .map(|(index, origin)| layout::TextOriginBinding {
                    origin: origin.clone(),
                    node: index + 1,
                })
                .collect(),
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        surgeist_layout::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &capture,
        0,
        layout::LayoutRootRequest::viewport(layout::Size::new(
            available,
            layout::Available::MaxContent,
        ))
        .unwrap(),
    )
    .unwrap();
    (batch, origins)
}

fn glyphs(placed: &text::Layout, bytes: &[u8]) -> Vec<(u32, f32)> {
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

fn advances(batch: &layout::CompletedLayoutBatch<usize>) -> Vec<f64> {
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

fn intrinsic_width(batch: &layout::CompletedLayoutBatch<usize>) -> f32 {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output()
        .size
        .width
}

#[test]
fn math_scalar_mapping_retains_original_utf16_selection_and_distinct_real_ink() {
    let bytes = math_ahem();
    let mut system = system(&bytes);
    for (value, original, original_glyph, target, advance, presentation_end) in [
        ("c", 0x63, 69, 82, 15.0, 2),
        ("h", 0x68, 74, 100, 18.0, 1),
        ("A", 0x41, 35, 36, 20.0, 2),
        ("\u{3f4}", 0x3f4, 68, 37, 14.0, 2),
    ] {
        for math in [false, true] {
            let source = source(value);
            let (batch, origins) = capture(
                &mut system,
                &source,
                vec![origin(
                    &source,
                    text::Range::new(0, 1),
                    value,
                    text::Range::new(0, 1),
                )],
                style(math, text::WhiteSpace::Preserve, text::WordBreak::Normal),
                layout::Available::Definite(40.0),
            );
            let placed = batch.placed_text()[0].layout();
            let advance = if math { advance } else { 10.0 };
            assert_eq!(
                glyphs(placed, &bytes),
                [(if math { target } else { original_glyph }, advance as f32)]
            );
            assert_eq!(advances(&batch), [advance]);
            assert_eq!(placed.source(), &source);
            assert_eq!(source.text().code_units(), [original]);
            assert_eq!(source.id(), Some(text::Id::from_u64(6305)));
            assert_eq!(source.revision(), 19);
            let finalized = batch.final_inline_fragments()[0]
                .fragment()
                .finalized_fragment()
                .finalized_line();
            assert_eq!(
                finalized
                    .mapping_views()
                    .map(|mapping| mapping.processed_range().end().get())
                    .max(),
                Some(1)
            );
            assert_eq!(
                finalized
                    .mapping_views()
                    .map(|mapping| mapping.presentation_range().end())
                    .max(),
                Some(if math { presentation_end } else { 1 })
            );
            assert_eq!(
                placed
                    .glyph_runs()
                    .iter()
                    .flat_map(|run| run.glyphs().iter().map(|glyph| glyph.range()))
                    .collect::<Vec<_>>(),
                [text::Range::new(0, 1)]
            );
            let at = |x| {
                placed
                    .positions()
                    .find(|position| {
                        placed.position_geometry(position).unwrap().rect().origin
                            == text::Point::new(x, 0.0)
                    })
                    .unwrap()
            };
            let begin = at(0.0);
            let end = at(advance);
            for (a, b) in [(&begin, &end), (&end, &begin)] {
                let selected = placed.position_selection(a, b).unwrap();
                assert_eq!(selected.source_text().code_units(), [original]);
                assert_eq!(selected.plaintext().code_units(), [original]);
                assert_eq!(selected.source_pieces().len(), 1);
                assert_eq!(selected.source_pieces()[0].origin(), &origins[0]);
                assert_eq!(
                    selected.source_pieces()[0].local_range(),
                    text::Range::new(0, 1)
                );
                assert_eq!(
                    selected.geometry().rects()[0].rect(),
                    text::Rect::new(0.0, 0.0, advance, 10.0)
                );
            }
            let fragments = placed
                .source_range_fragments(&origins[0], text::Range::new(0, 1))
                .unwrap();
            assert_eq!(fragments.units().count(), 1);
            assert_eq!(
                fragments
                    .units()
                    .next()
                    .unwrap()
                    .pieces()
                    .next()
                    .unwrap()
                    .rect(),
                text::Rect::new(0.0, 0.0, advance, 10.0)
            );
            let actual = placed
                .run_views()
                .flat_map(|run| {
                    run.glyphs()
                        .map(|glyph| {
                            render::TextGlyph::try_new(
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
            let font = render::FontData::try_from_bytes(bytes.clone(), 0).unwrap();
            let instance =
                render::TextInstance::try_new(10.0, vec![], render::GlyphSynthesis::none())
                    .unwrap();
            let retained = font
                .prepare_glyph_run(
                    &instance,
                    &actual,
                    render::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
                )
                .unwrap();
            assert_eq!(retained.bounds().ink_rect().unwrap().width(), 10.0);
            let paint = render::prepare_text_layout(
                placed,
                render::Transform::identity(),
                render::TextLayoutLimits::new(
                    render::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
                    8,
                    32,
                    8,
                ),
            )
            .unwrap();
            assert!(std::ptr::eq(paint.layout(), placed));
            assert_eq!(paint.layout().source(), &source);
            assert!(!paint.scene().is_empty());
        }
    }
}

#[test]
fn complete_node_cardinality_survives_origin_splitting_and_changes_real_fit() {
    let bytes = math_ahem();
    let mut system = system(&bytes);
    let source = source("cc");
    for separate_nodes in [false, true, false] {
        let make_origins = || {
            if separate_nodes {
                vec![
                    origin(&source, text::Range::new(0, 1), "c", text::Range::new(0, 1)),
                    origin(&source, text::Range::new(1, 2), "c", text::Range::new(0, 1)),
                ]
            } else {
                vec![
                    origin(
                        &source,
                        text::Range::new(0, 1),
                        "cc",
                        text::Range::new(0, 1),
                    ),
                    origin(
                        &source,
                        text::Range::new(1, 2),
                        "cc",
                        text::Range::new(1, 2),
                    ),
                ]
            }
        };
        let (batch, origins) = capture(
            &mut system,
            &source,
            make_origins(),
            style(true, text::WhiteSpace::Preserve, text::WordBreak::BreakAll),
            layout::Available::Definite(25.0),
        );
        let placed = batch.placed_text()[0].layout();
        assert_eq!(
            glyphs(placed, &bytes),
            vec![
                (
                    if separate_nodes { 82 } else { 69 },
                    if separate_nodes { 15.0 } else { 10.0 }
                );
                2
            ]
        );
        assert_eq!(
            advances(&batch),
            if separate_nodes {
                vec![15.0, 15.0]
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
            if separate_nodes {
                vec![text::Range::new(0, 1), text::Range::new(1, 2)]
            } else {
                vec![text::Range::new(0, 2)]
            }
        );
        for (index, origin) in origins.iter().enumerate() {
            assert_eq!(origin.view().source(), &source);
            let range = placed
                .source_range_fragments(origin, text::Range::new(index, index + 1))
                .unwrap();
            assert_eq!(range.units().count(), 1);
            let rect = range
                .units()
                .next()
                .unwrap()
                .pieces()
                .next()
                .unwrap()
                .rect();
            assert_eq!(
                rect,
                text::Rect::new(
                    if separate_nodes {
                        0.0
                    } else {
                        [0.0, 10.0][index]
                    },
                    if separate_nodes {
                        [0.0, 10.0][index]
                    } else {
                        0.0
                    },
                    if separate_nodes { 15.0 } else { 10.0 },
                    10.0
                )
            );
        }
        for (purpose, expected) in [
            (
                layout::Available::MinContent,
                if separate_nodes { 15.0 } else { 10.0 },
            ),
            (
                layout::Available::MaxContent,
                if separate_nodes { 30.0 } else { 20.0 },
            ),
        ] {
            let (intrinsic, _) = capture(
                &mut system,
                &source,
                make_origins(),
                style(true, text::WhiteSpace::Preserve, text::WordBreak::BreakAll),
                purpose,
            );
            assert_eq!(intrinsic_width(&intrinsic), expected);
        }
    }
}

#[test]
fn discard_cannot_turn_complete_space_c_space_into_eligible_single_character_node() {
    let bytes = math_ahem();
    let mut system = system(&bytes);
    for (value, range, glyph, advance) in [
        (" c ", text::Range::new(1, 2), 69, 10.0),
        ("c", text::Range::new(0, 1), 82, 15.0),
    ] {
        let source = source(value);
        let whole = text::Range::new(0, source.text().len());
        let (batch, origins) = capture(
            &mut system,
            &source,
            vec![origin(&source, whole, value, whole)],
            style(true, text::WhiteSpace::Discard, text::WordBreak::Normal),
            layout::Available::Definite(40.0),
        );
        let placed = batch.placed_text()[0].layout();
        assert_eq!(
            placed
                .analyzed_source()
                .processed_source()
                .processed_text()
                .code_units(),
            [0x63]
        );
        assert_eq!(glyphs(placed, &bytes), [(glyph, advance as f32)]);
        assert_eq!(advances(&batch), [advance]);
        assert_eq!(placed.source(), &source);
        assert_eq!(
            placed
                .glyph_runs()
                .iter()
                .flat_map(|run| run.glyphs().iter().map(|glyph| glyph.range()))
                .collect::<Vec<_>>(),
            [range]
        );
        let fragments = placed.source_range_fragments(&origins[0], range).unwrap();
        assert_eq!(
            fragments
                .units()
                .next()
                .unwrap()
                .pieces()
                .next()
                .unwrap()
                .rect(),
            text::Rect::new(0.0, 0.0, advance, 10.0)
        );
    }
}

#[test]
fn normal_math_letter_word_stays_unbroken_while_break_all_uses_actual_mapped_advances() {
    let bytes = math_ahem();
    let mut system = system(&bytes);
    let source = source("Ac");
    let make_origins = || {
        vec![
            origin(&source, text::Range::new(0, 1), "A", text::Range::new(0, 1)),
            origin(&source, text::Range::new(1, 2), "c", text::Range::new(0, 1)),
        ]
    };
    for word_break in [text::WordBreak::Normal, text::WordBreak::BreakAll] {
        let (batch, _) = capture(
            &mut system,
            &source,
            make_origins(),
            style(true, text::WhiteSpace::Preserve, word_break),
            layout::Available::Definite(25.0),
        );
        let placed = batch.placed_text()[0].layout();
        assert_eq!(glyphs(placed, &bytes), [(36, 20.0), (82, 15.0)]);
        assert_eq!(
            advances(&batch),
            if word_break == text::WordBreak::Normal {
                vec![35.0]
            } else {
                vec![20.0, 15.0]
            }
        );
        for (purpose, expected) in [
            (
                layout::Available::MinContent,
                if word_break == text::WordBreak::Normal {
                    35.0
                } else {
                    20.0
                },
            ),
            (layout::Available::MaxContent, 35.0),
        ] {
            let (intrinsic, _) = capture(
                &mut system,
                &source,
                make_origins(),
                style(true, text::WhiteSpace::Preserve, word_break),
                purpose,
            );
            assert_eq!(intrinsic_width(&intrinsic), expected);
        }
    }
}

#[cfg(feature = "text-accessibility")]
#[test]
fn math_accessibility_keeps_original_c_with_fifteen_pixel_source_geometry() {
    let bytes = math_ahem();
    let mut system = system(&bytes);
    let source = source("c");
    let (batch, _) = capture(
        &mut system,
        &source,
        vec![origin(
            &source,
            text::Range::new(0, 1),
            "c",
            text::Range::new(0, 1),
        )],
        style(true, text::WhiteSpace::Preserve, text::WordBreak::Normal),
        layout::Available::Definite(40.0),
    );
    let placed = batch.placed_text()[0].layout();
    drop(system);
    let mut accessibility = text::Accessibility::new(20);
    let parent = 1000_u64.into();
    let update = placed
        .accessibility_update(
            &mut accessibility,
            parent,
            parent,
            text::Point::new(100.0, 200.0),
        )
        .unwrap();
    let position = placed
        .access_position(&accessibility, text::Cursor::new(0, text::Affinity::After))
        .unwrap()
        .unwrap();
    let node = &update
        .nodes
        .iter()
        .find(|(id, _)| *id == position.node)
        .unwrap()
        .1;
    assert_eq!(placed.source(), &source);
    assert_eq!(node.value(), Some("c"));
    assert_eq!(node.character_lengths(), [1]);
    let bounds = node.bounds().unwrap();
    assert_eq!(
        (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
        (100.0, 200.0, 115.0, 210.0)
    );
}
