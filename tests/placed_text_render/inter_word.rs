//! Native Layout allocation reaches actual Render glyph operations unchanged.
use super::*;

const AHEM: &[u8] = include_bytes!("../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn table<'a>(bytes: &'a [u8], tag: &[u8; 4]) -> &'a [u8] {
    let row = (0..usize::from(u16_at(bytes, 4)))
        .map(|index| 12 + 16 * index)
        .find(|at| &bytes[*at..*at + 4] == tag)
        .unwrap();
    let at = usize::try_from(u32_at(bytes, row + 8)).unwrap();
    let len = usize::try_from(u32_at(bytes, row + 12)).unwrap();
    &bytes[at..at + len]
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

fn visible_separator_font() -> Vec<u8> {
    // Consumer-owned adaptation of the existing root SFNT fixture builders.
    // The only semantic change is NBSP -> actual visible X58; licensed Ahem
    // outlines and metrics are untouched. See the owning Ahem PROVENANCE.md
    // and COPYING (public domain with CC0 fallback). No font query is an oracle.
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12];
    cmap.extend(12_u16.to_be_bytes());
    cmap.extend(0_u16.to_be_bytes());
    cmap.extend(40_u32.to_be_bytes());
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(2_u32.to_be_bytes());
    for (scalar, glyph) in [(0x41_u32, 35_u32), (0xa0, 58)] {
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(glyph.to_be_bytes());
    }
    let mut tables = (0..usize::from(u16_at(AHEM, 4)))
        .map(|index| {
            let tag: [u8; 4] = AHEM[12 + 16 * index..16 + 16 * index].try_into().unwrap();
            let data = if tag == *b"cmap" {
                cmap.clone()
            } else {
                table(AHEM, &tag).to_vec()
            };
            (tag, data)
        })
        .collect::<Vec<_>>();
    tables.sort_by_key(|(tag, _)| *tag);
    let count = u16::try_from(tables.len()).unwrap();
    let power = 1_u16 << count.ilog2();
    let mut bytes = vec![0; 12 + tables.len() * 16];
    bytes[..4].copy_from_slice(&AHEM[..4]);
    bytes[4..6].copy_from_slice(&count.to_be_bytes());
    bytes[6..8].copy_from_slice(&(power * 16).to_be_bytes());
    bytes[8..10].copy_from_slice(&u16::try_from(count.ilog2()).unwrap().to_be_bytes());
    bytes[10..12].copy_from_slice(&(count * 16 - power * 16).to_be_bytes());
    let mut head = 0;
    for (index, (tag, mut data)) in tables.into_iter().enumerate() {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        if tag == *b"head" {
            head = bytes.len();
            data[8..12].fill(0);
        }
        let row = 12 + 16 * index;
        let offset = u32::try_from(bytes.len()).unwrap();
        bytes[row..row + 4].copy_from_slice(&tag);
        bytes[row + 4..row + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        bytes[row + 8..row + 12].copy_from_slice(&offset.to_be_bytes());
        bytes[row + 12..row + 16]
            .copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        bytes.extend(data);
    }
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&bytes));
    bytes[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    bytes
}

fn qualify_font(bytes: &[u8]) {
    assert_eq!(checksum(bytes), 0xb1b0_afba);
    assert_eq!(u16_at(table(bytes, b"head"), 18), 1000);
    assert_eq!(u16_at(table(bytes, b"head"), 50), 0);
    assert!(u16_at(table(bytes, b"hhea"), 34) > 58);
    assert_eq!(u16_at(table(bytes, b"maxp"), 4), 281);
    assert_eq!(u16_at(table(bytes, b"OS/2"), 62) & 0x80, 0);
    let hhea = table(bytes, b"hhea");
    assert_eq!(
        [4, 6, 8].map(|at| i16::from_be_bytes(hhea[at..at + 2].try_into().unwrap())),
        [800, -200, 0]
    );
    let cmap = table(bytes, b"cmap");
    assert_eq!((u16_at(cmap, 4), u16_at(cmap, 6)), (3, 10));
    let sub = usize::try_from(u32_at(cmap, 8)).unwrap();
    assert_eq!((u16_at(cmap, sub), u32_at(cmap, sub + 12)), (12, 2));
    for (index, scalar, glyph) in [(0, 0x41, 35), (1, 0xa0, 58)] {
        let group = sub + 16 + index * 12;
        assert_eq!(
            [0, 4, 8].map(|at| u32_at(cmap, group + at)),
            [scalar, scalar, glyph]
        );
        let glyph = usize::try_from(glyph).unwrap();
        assert_eq!(u16_at(table(bytes, b"hmtx"), glyph * 4), 1000);
        let start = usize::from(u16_at(table(bytes, b"loca"), glyph * 2)) * 2;
        let end = usize::from(u16_at(table(bytes, b"loca"), (glyph + 1) * 2)) * 2;
        assert!(end > start);
        let outline = &table(bytes, b"glyf")[start..end];
        assert_eq!(
            [2, 4, 6, 8].map(|at| i16::from_be_bytes(outline[at..at + 2].try_into().unwrap())),
            [0, -200, 1000, 800]
        );
    }
    // No substitution, positioning, or named-baseline table can silently
    // change these literal glyph and metric premises.
    for tag in [b"GSUB", b"GPOS", b"BASE", b"vhea", b"vmtx", b"fvar"] {
        assert!(
            (0..usize::from(u16_at(bytes, 4)))
                .all(|index| &bytes[12 + index * 16..16 + index * 16] != tag)
        );
    }
    for tag in [b"hhea", b"hmtx", b"OS/2", b"glyf", b"loca", b"maxp"] {
        assert_eq!(table(bytes, tag), table(AHEM, tag));
    }
}

fn capture(
    bytes: &[u8],
    orientation: text::LineOrientation,
) -> (layout::CompletedLayoutBatch<usize>, text::PreparedOrigin) {
    qualify_font(bytes);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    if orientation == text::LineOrientation::Vertical {
        // Absence-qualified synthetic vertical metrics are an existing native
        // policy, not a claim that hmtx alone supplies TTB. The public owner
        // must positively resolve A-D=10 and nominal origin (W/2,A)=(5,8).
        let face = system.registered_font_faces().next().unwrap().to_owned();
        let instance = face.unsynthesized_instance(10.0, &[]).unwrap();
        for glyph in [35, 58] {
            let vertical = instance
                .vertical_glyph_metrics(glyph, system.font_catalogue_identity())
                .unwrap();
            assert_eq!(
                (vertical.advance(), vertical.origin_x(), vertical.origin_y()),
                (10.0, 5.0, 8.0)
            );
        }
        let cross = instance
            .upright_metrics(
                system.font_catalogue_identity(),
                text::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
            )
            .unwrap();
        assert_eq!((cross.used_over(), cross.used_under()), (5.0, -5.0));
    }
    let source = text::Source::identified("A\u{a0}A", text::Id::from_u64(1564001), 7);
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: text::WhiteSpace::Preserve,
        word_spacing: 4.0,
        ..text::Style::default()
    };
    let ancestor_wrap = style.wrap;
    let analyzed = text::AnalyzedSource::from_paragraph(
        text::ParagraphInput {
            source: source.clone(),
            base_style: style,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
            terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: text::Range::new(0, 3),
                aggregate: text::Range::new(0, 3),
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        if orientation == text::LineOrientation::Vertical {
            text::AnalysisTypography::Upright {
                computed_direction: text::ParagraphBaseDirection::LeftToRight,
            }
        } else {
            text::AnalysisTypography::HorizontalComposition
        },
    )
    .unwrap();
    let wrapping = analyzed
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| text::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(wrapping).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            text::PreparationOptions::default()
                .with_orientation(orientation)
                .with_baseline_reference(if orientation == text::LineOrientation::Vertical {
                    text::FontBaseline::Central
                } else {
                    text::FontBaseline::Alphabetic
                }),
        )
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let px = |value| layout::PreferredSizeOf::value(layout::LengthPercentageOf::px(value).unwrap());
    let tree = Tree {
        inputs: [
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                writing_mode: match orientation {
                    text::LineOrientation::Horizontal => layout::WritingMode::HorizontalTb,
                    text::LineOrientation::Sideways => layout::WritingMode::SidewaysRl,
                    text::LineOrientation::Vertical => layout::WritingMode::VerticalRl,
                },
                inline_baseline_context: layout::InlineBaselineContext::Named(
                    if orientation == text::LineOrientation::Vertical {
                        layout::BaselineName::Central
                    } else {
                        layout::BaselineName::Alphabetic
                    },
                ),
                text_align: layout::TextAlign::Justify,
                text_align_last: layout::TextAlignLast::Justify,
                text_justification: layout::TextJustification::InterWord,
                size: layout::Size::new(px(40.0), px(40.0)),
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(origin.clone())),
        ],
        non_box: layout::NodeInput::non_box(),
    };
    let flow = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::TextParagraphBinding {
            prepared,
            container: 0,
            nodes: vec![1],
            origins: vec![layout::TextOriginBinding {
                origin: origin.clone(),
                node: 1,
            }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        layout::WrappingInput::default(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &flow,
        0,
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::Definite(40.0),
            layout::Available::Definite(40.0),
        ))
        .unwrap(),
    )
    .unwrap();
    (batch, origin)
}

fn limits() -> render::TextLayoutLimits {
    render::TextLayoutLimits::new(
        render::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
        8,
        32,
        8,
    )
}

fn block_on<T>(future: impl std::future::Future<Output = T>) -> T {
    struct WakeThread(std::thread::Thread);
    impl std::task::Wake for WakeThread {
        fn wake(self: std::sync::Arc<Self>) {
            self.0.unpark();
        }
        fn wake_by_ref(self: &std::sync::Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = std::task::Waker::from(std::sync::Arc::new(WakeThread(std::thread::current())));
    let mut context = std::task::Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        if let std::task::Poll::Ready(value) =
            std::future::Future::poll(future.as_mut(), &mut context)
        {
            return value;
        }
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        assert!(
            !remaining.is_zero(),
            "native Render future exceeded 60 seconds"
        );
        std::thread::park_timeout(remaining);
    }
}

#[test]
fn native_inter_word_fill_keeps_equal_half_visible_paint_in_horizontal_clockwise_and_upright_lines()
{
    let bytes = visible_separator_font();
    // One small real GPU surface is reused across all three native bases.
    // There is no skip or CPU substitute when a compatible adapter is absent.
    let mut renderer = block_on(render::Renderer::new(render::Options::default())).unwrap();
    let mut surface =
        block_on(renderer.create_headless(render::Size::try_new(40.0, 40.0).unwrap(), 1.0))
            .unwrap();
    for orientation in [
        text::LineOrientation::Horizontal,
        text::LineOrientation::Sideways,
        text::LineOrientation::Vertical,
    ] {
        let horizontal = orientation == text::LineOrientation::Horizontal;
        let clockwise = orientation == text::LineOrientation::Sideways;
        let (batch, origin) = capture(&bytes, orientation);
        let placed = batch.placed_text()[0].layout();
        assert_eq!(placed.orientation(), orientation);
        assert_eq!(placed.lines().len(), 1);
        let line = batch.final_inline_fragments()[0]
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(line.feedback().advance, 40.0);
        assert_eq!(line.measurements().alignment().advance(), 40.0);
        assert_eq!(
            line.selected_line().request().context.spacing,
            text::SpacingPolicy::InterWord
        );
        let rows = line.spacing_slots().collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].authored_word_spacing, 4.0);
        assert_eq!(rows[0].allocation, 6.0);
        assert_eq!(rows[0].advance, 20.0);
        assert_eq!(
            rows[0].geometry,
            text::SpacingSlotGeometry::Independent {
                intrinsic_advance: 10.0
            }
        );
        let expected_cells = if !horizontal {
            [
                text::Rect::new(30.0, 0.0, 10.0, 10.0),
                text::Rect::new(30.0, 10.0, 10.0, 20.0),
                text::Rect::new(30.0, 30.0, 10.0, 10.0),
            ]
        } else {
            [
                text::Rect::new(0.0, 0.0, 10.0, 10.0),
                text::Rect::new(10.0, 0.0, 20.0, 10.0),
                text::Rect::new(30.0, 0.0, 10.0, 10.0),
            ]
        };
        for (index, expected) in expected_cells.into_iter().enumerate() {
            let fragments = placed
                .source_range_fragments(&origin, text::Range::new(index, index + 1))
                .unwrap();
            assert_eq!(
                fragments
                    .units()
                    .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
                [expected]
            );
        }
        // Qualify the producer's raw basis first. Pixel expectations below
        // come from the parsed square outlines and literal 0/15/30 pens.
        let mut index = 0;
        for run in placed.run_views() {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(
                run.glyph_basis(),
                if clockwise {
                    text::GlyphBasis::Clockwise
                } else {
                    text::GlyphBasis::Unrotated
                }
            );
            let inline_origin = if horizontal {
                run.origin().x
            } else {
                run.origin().y
            };
            if clockwise {
                assert_eq!(run.origin().x, 32.0);
            } else if horizontal {
                assert_eq!(run.origin().y, 8.0);
            } else {
                assert_eq!(run.origin().x, 35.0);
            }
            for glyph in run.glyphs() {
                let id = [35, 58, 35][index];
                let pen = [0.0, 15.0, 30.0][index];
                let (dx, dy) = if orientation == text::LineOrientation::Vertical {
                    (-5.0, 8.0)
                } else {
                    (0.0, 0.0)
                };
                assert_eq!(
                    (glyph.id(), glyph.x(), glyph.y(), glyph.advance()),
                    (id, dx, dy, 10.0)
                );
                assert_eq!(inline_origin + glyph.pen(), pen);
                index += 1;
            }
        }
        assert_eq!(index, 3);
        let paint =
            render::prepare_text_layout(placed, render::Transform::identity(), limits()).unwrap();
        block_on(renderer.render(&mut surface, paint.scene(), render::Parameters::default()))
            .unwrap();
        let pixels = block_on(renderer.read_headless(&surface)).unwrap();
        assert_eq!((pixels.size().width(), pixels.size().height()), (40, 40));
        // A/separator/A draw three actual 10px squares. Half of authored4 +
        // allocated6 appears before and after visible X58, leaving gaps10..15
        // and25..30. Outer bounds alone could not detect either wrong half.
        for y in 0..40_usize {
            for x in 0..40_usize {
                let (inline, inside_cross) = if horizontal {
                    (x, y < 10)
                } else {
                    (y, x >= 30)
                };
                let ink = inside_cross
                    && ((0..10).contains(&inline)
                        || (15..25).contains(&inline)
                        || (30..40).contains(&inline));
                let expected = if ink { 255_u8 } else { 0 };
                let actual = pixels.rgba()[(y * 40 + x) * 4 + 3];
                assert!(
                    actual.abs_diff(expected) <= 2,
                    "{orientation:?} pixel({x},{y}) alpha={actual}, expected {expected}"
                );
            }
        }
        assert_eq!(paint.layout().source().text().code_units(), [65, 0xa0, 65]);
        let at = |offset, affinity| {
            placed
                .positions()
                .find(|position| position.source_cursor() == text::Cursor::new(offset, affinity))
                .unwrap()
        };
        let selection = placed
            .position_selection(
                &at(0, text::Affinity::After),
                &at(3, text::Affinity::Before),
            )
            .unwrap();
        assert_eq!(selection.source_text().code_units(), [65, 0xa0, 65]);
        assert_eq!(selection.plaintext().code_units(), [65, 0xa0, 65]);
    }
}

#[cfg(feature = "text-accessibility")]
#[test]
fn native_inter_word_accessibility_keeps_nbsp_source_and_allocated_character_geometry() {
    let bytes = visible_separator_font();
    let (batch, _) = capture(&bytes, text::LineOrientation::Horizontal);
    let placed = batch.placed_text()[0].layout();
    let paint =
        render::prepare_text_layout(placed, render::Transform::identity(), limits()).unwrap();
    let mut accessibility = text::Accessibility::new(20);
    let parent = 1000_u64.into();
    let cursor = text::Cursor::new(1, text::Affinity::After);
    let analyzed = placed.analyzed_source();
    let source_update = analyzed
        .accessibility_update(&mut accessibility, parent, parent)
        .unwrap();
    let source_position = analyzed
        .access_position(&accessibility, cursor)
        .unwrap()
        .unwrap();
    let source_node = &source_update
        .nodes
        .iter()
        .find(|(id, _)| *id == source_position.node)
        .unwrap()
        .1;
    assert_eq!(source_node.value(), Some("\u{a0}"));
    assert_eq!(source_node.character_lengths(), [2]);
    assert!(source_node.bounds().is_none());
    let update = paint
        .layout()
        .accessibility_update(
            &mut accessibility,
            parent,
            parent,
            text::Point::new(100.0, 200.0),
        )
        .unwrap();
    let position = placed
        .access_position(&accessibility, cursor)
        .unwrap()
        .unwrap();
    let node = &update
        .nodes
        .iter()
        .find(|(id, _)| *id == position.node)
        .unwrap()
        .1;
    assert_eq!(node.value(), Some("\u{a0}"));
    assert_eq!(node.character_lengths(), [2]);
    assert_eq!(position.character_index, 0);
    let bounds = node.bounds().unwrap();
    assert_eq!(
        (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
        (110.0, 200.0, 130.0, 210.0)
    );
    assert_eq!(placed.source().text().code_units(), [65, 0xa0, 65]);
    assert_eq!(placed.source().id(), Some(text::Id::from_u64(1564001)));
    assert_eq!(placed.source().revision(), 7);
}
