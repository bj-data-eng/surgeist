//! One real face supplies different script references on a shared named line.
//! Raw metrics/outlines remain unchanged; Render consumes actual paint origins.
use super::*;

const AHEM: &[u8] = include_bytes!("../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const ALPHA: text::FontBaseline = text::FontBaseline::Alphabetic;

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let at = (0..usize::from(u16_at(bytes, 4)))
        .map(|index| 12 + 16 * index)
        .find(|at| bytes[*at..*at + 4] == tag)
        .unwrap();
    let start = usize::try_from(u32_at(bytes, at + 8)).unwrap();
    let length = usize::try_from(u32_at(bytes, at + 12)).unwrap();
    &bytes[start..start + length]
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

fn font() -> Vec<u8> {
    // BASE1.0, horizontal Axis, one romn tag and sorted grek/latn records.
    // The two complete BaseScripts contain format1 coordinates -100/+100.
    // This bounded root fixture adds one table to the licensed Ahem bytes;
    // it needs no cross-crate private test module or additional dependency.
    let mut base = vec![
        0, 1, 0, 0, 0, 8, 0, 0, // header
        0, 4, 0, 10, // Axis offsets
        0, 1, b'r', b'o', b'm', b'n', // BaseTagList
        0, 2, b'g', b'r', b'e', b'k', 0, 14, b'l', b'a', b't', b'n', 0, 30,
    ];
    for coordinate in [-100_i16, 100] {
        base.extend([0, 6, 0, 0, 0, 0, 0, 0, 0, 1, 0, 6, 0, 1]);
        base.extend(coordinate.to_be_bytes());
    }
    let mut tables = (0..usize::from(u16_at(AHEM, 4)))
        .map(|index| {
            let tag: [u8; 4] = AHEM[12 + 16 * index..16 + 16 * index].try_into().unwrap();
            assert_ne!(tag, *b"BASE");
            (tag, table(AHEM, tag).to_vec())
        })
        .collect::<Vec<_>>();
    tables.push((*b"BASE", base));
    tables.sort_by_key(|(tag, _)| *tag);
    let count = u16::try_from(tables.len()).unwrap();
    let power = 1_u16 << count.ilog2();
    let mut output = vec![0; 12 + 16 * tables.len()];
    output[..4].copy_from_slice(&AHEM[..4]);
    output[4..6].copy_from_slice(&count.to_be_bytes());
    output[6..8].copy_from_slice(&(power * 16).to_be_bytes());
    output[8..10].copy_from_slice(&u16::try_from(count.ilog2()).unwrap().to_be_bytes());
    output[10..12].copy_from_slice(&(count * 16 - power * 16).to_be_bytes());
    let mut head = 0;
    for (index, (tag, mut data)) in tables.into_iter().enumerate() {
        while !output.len().is_multiple_of(4) {
            output.push(0);
        }
        if tag == *b"head" {
            data[8..12].fill(0);
            head = output.len();
        }
        let at = 12 + 16 * index;
        output[at..at + 4].copy_from_slice(&tag);
        output[at + 4..at + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        let offset = u32::try_from(output.len()).unwrap();
        output[at + 8..at + 12].copy_from_slice(&offset.to_be_bytes());
        output[at + 12..at + 16].copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        output.extend(data);
    }
    while !output.len().is_multiple_of(4) {
        output.push(0);
    }
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&output));
    output[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    output
}

fn cmap_glyph(bytes: &[u8], scalar: u16) -> u16 {
    let cmap = table(bytes, *b"cmap");
    let record = (0..usize::from(u16_at(cmap, 2)))
        .map(|index| 4 + 8 * index)
        .find(|at| u16_at(cmap, *at) == 3 && u16_at(cmap, *at + 2) == 1)
        .unwrap();
    let sub = &cmap[usize::try_from(u32_at(cmap, record + 4)).unwrap()..];
    assert_eq!(u16_at(sub, 0), 4);
    let count = usize::from(u16_at(sub, 6)) / 2;
    let index = (0..count)
        .find(|index| scalar <= u16_at(sub, 14 + 2 * index))
        .unwrap();
    let start = u16_at(sub, 16 + 2 * count + 2 * index);
    assert!(scalar >= start);
    let delta = u16_at(sub, 16 + 4 * count + 2 * index);
    let range_at = 16 + 6 * count + 2 * index;
    let range = u16_at(sub, range_at);
    if range == 0 {
        scalar.wrapping_add(delta)
    } else {
        let glyph = u16_at(
            sub,
            range_at + usize::from(range) + 2 * usize::from(scalar - start),
        );
        if glyph == 0 {
            0
        } else {
            glyph.wrapping_add(delta)
        }
    }
}

fn qualify_font(bytes: &[u8]) {
    assert_eq!(checksum(bytes), 0xb1b0_afba);
    for tag in [
        *b"hhea", *b"OS/2", *b"hmtx", *b"cmap", *b"glyf", *b"loca", *b"maxp",
    ] {
        assert_eq!(table(bytes, tag), table(AHEM, tag));
    }
    assert_eq!(u16_at(table(bytes, *b"head"), 18), 1000);
    assert_eq!(u16_at(table(bytes, *b"head"), 50), 0);
    assert_eq!(u16_at(table(bytes, *b"OS/2"), 62) & 0x80, 0);
    assert_eq!(
        [4, 6, 8]
            .map(|at| i16::from_be_bytes(table(bytes, *b"hhea")[at..at + 2].try_into().unwrap())),
        [800, -200, 0]
    );
    assert_eq!((cmap_glyph(bytes, 65), cmap_glyph(bytes, 0x3a7)), (35, 274));
    for (glyph, bounds) in [(35, [0, -200, 1000, 800]), (274, [0, 400, 1000, 600])] {
        assert_eq!(u16_at(table(bytes, *b"hmtx"), glyph * 4), 1000);
        let start = usize::from(u16_at(table(bytes, *b"loca"), glyph * 2)) * 2;
        let outline = &table(bytes, *b"glyf")[start..];
        assert_eq!(
            [2, 4, 6, 8].map(|at| i16::from_be_bytes(outline[at..at + 2].try_into().unwrap())),
            bounds
        );
    }
    // Independently follow every relevant BASE owner-relative offset.
    let base = table(bytes, *b"BASE");
    assert_eq!(
        (u16_at(base, 0), u16_at(base, 2), u16_at(base, 6)),
        (1, 0, 0)
    );
    let axis = usize::from(u16_at(base, 4));
    let tags = axis + usize::from(u16_at(base, axis));
    assert_eq!(u16_at(base, tags), 1);
    assert_eq!(&base[tags + 2..tags + 6], b"romn");
    let scripts = axis + usize::from(u16_at(base, axis + 2));
    assert_eq!(u16_at(base, scripts), 2);
    for (index, tag, expected) in [(0, *b"grek", -100), (1, *b"latn", 100)] {
        let record = scripts + 2 + 6 * index;
        assert_eq!(&base[record..record + 4], &tag);
        let script = scripts + usize::from(u16_at(base, record + 4));
        assert_eq!((u16_at(base, script + 2), u16_at(base, script + 4)), (0, 0));
        let values = script + usize::from(u16_at(base, script));
        assert_eq!((u16_at(base, values), u16_at(base, values + 2)), (0, 1));
        let coordinate = values + usize::from(u16_at(base, values + 4));
        assert_eq!(u16_at(base, coordinate), 1);
        assert_eq!(
            i16::from_be_bytes(base[coordinate + 2..coordinate + 4].try_into().unwrap()),
            expected
        );
    }
}

fn prepare(
    system: &mut text::System,
    source: &text::Source,
    orientation: text::LineOrientation,
) -> text::PreparedText {
    let authored = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: text::WhiteSpace::Preserve,
        ..text::Style::default()
    };
    let ancestor_wrap = authored.wrap;
    let analyzed = text::AnalyzedSource::from_paragraph(
        text::ParagraphInput {
            source: source.clone(),
            base_style: authored,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
            terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: text::Range::new(0, 2),
                aggregate: text::Range::new(0, 2),
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| text::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let options = text::PreparationOptions::default()
        .with_orientation(orientation)
        .with_baseline_reference(ALPHA);
    let prepared = system.prepare(&analyzed, options).unwrap();
    assert_eq!(prepared.baseline_reference(), Some(ALPHA));
    assert_eq!(prepared.preparation_options(), options);
    assert_eq!(
        prepared
            .preparation_options()
            .requested_baseline_reference(),
        Some(ALPHA)
    );
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    prepared
}

fn composition(bytes: &[u8], mode: layout::WritingMode) {
    let horizontal = mode == layout::WritingMode::HorizontalTb;
    let orientation = if horizontal {
        text::LineOrientation::Horizontal
    } else {
        text::LineOrientation::SidewaysClockwise
    };
    let source = text::Source::identified("A\u{3a7}", text::Id::from_u64(1561008), 5);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    let prepared = prepare(&mut system, &source, orientation);
    let origin = prepared.origin_views().next().unwrap().handle();
    let px = |value| layout::PreferredSizeOf::value(layout::LengthPercentageOf::px(value).unwrap());
    let tree = Tree {
        inputs: [
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                writing_mode: mode,
                inline_baseline_context: layout::InlineBaselineContext::Named(
                    layout::BaselineName::Alphabetic,
                ),
                size: layout::Size::new(px(40.0), px(40.0)),
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::inline_text(layout::TextParticipation::new(origin.clone())),
        ],
        non_box: layout::NodeInput::non_box(),
    };
    let flow = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::TextParagraphBinding {
            inline_box_fragments: layout::InlineBoxFragmentationDemand::NotRequested,
            prepared: prepared.clone(),
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
        Vec::new(),
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
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    // Raw A8/D-2 with Latin b1 gives7/3; Greek b-1 gives9/1.
    // Their real component maxima are9/3/H12. One common alignment is
    // y9 for H, x(40-9)=31 for Vrl/Srl, and x(12-9)=3 for Vlr.
    let common = match mode {
        layout::WritingMode::HorizontalTb => 9.0,
        layout::WritingMode::VerticalLr => 3.0,
        layout::WritingMode::VerticalRl | layout::WritingMode::SidewaysRl => 31.0,
        _ => unreachable!("the test supplies only supported H/CW profiles"),
    };
    let axis = if horizontal {
        layout::PhysicalAxis::Vertical
    } else {
        layout::PhysicalAxis::Horizontal
    };
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let output = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        let pair = output.named_baselines.get(layout::BaselineName::Alphabetic);
        let fact = layout::BaselineFactOf::Available(
            layout::BaselineCoordinateOf::try_new(axis, common as f32).unwrap(),
        );
        assert_eq!((pair.first(), pair.last()), (fact, fact));
    }
    let line = batch.final_inline_fragments()[0]
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert_eq!(line.prepared_text(), prepared);
    assert_eq!(
        (
            line.feedback().baseline,
            line.feedback().line_height,
            line.feedback().advance
        ),
        (9.0, 12.0, 20.0)
    );
    assert_eq!(placed.orientation(), orientation);
    assert_eq!(placed.source(), &source);
    assert_eq!(placed.source().text().code_units(), [65, 0x3a7]);
    assert_eq!(placed.lines().len(), 1);
    assert_eq!(placed.lines()[0].baseline(), common);
    let bounds = if horizontal {
        text::Rect::new(0.0, 0.0, 40.0, 12.0)
    } else {
        text::Rect::new(common - 3.0, 0.0, 12.0, 40.0)
    };
    assert_eq!(placed.lines()[0].bounds(), bounds);
    assert_eq!(placed.bounds(), bounds);
    let expected_origins = if horizontal {
        [text::Point::new(0.0, 10.0), text::Point::new(10.0, 8.0)]
    } else {
        [
            text::Point::new(common - 1.0, 0.0),
            text::Point::new(common + 1.0, 10.0),
        ]
    };
    let runs = placed.run_views().collect::<Vec<_>>();
    assert_eq!(
        runs.len(),
        2,
        "different real BASE displacements cannot merge paint origins"
    );
    for (index, run) in runs.iter().enumerate() {
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.face().index(), 0);
        assert_eq!(
            run.face().catalogue_identity(),
            prepared.font_catalogue_identity()
        );
        assert_eq!(run.range(), text::Range::new(index, index + 1));
        assert_eq!(run.origin(), expected_origins[index]);
        assert_eq!(
            run.glyph_basis(),
            if horizontal {
                text::GlyphBasis::Unrotated
            } else {
                text::GlyphBasis::Clockwise
            }
        );
        let instance = run.effective_instance().unwrap();
        let raw = instance
            .metrics(prepared.font_catalogue_identity())
            .unwrap();
        assert_eq!(
            (raw.ascent(), raw.descent(), raw.leading()),
            (8.0, -2.0, 0.0)
        );
        let reference = run
            .baseline(
                prepared.font_catalogue_identity(),
                text::FontBaselineAxis::HorizontalY,
                ALPHA,
            )
            .unwrap();
        assert_eq!(
            reference.script().iso15924(),
            if index == 0 { *b"Latn" } else { *b"Grek" }
        );
        let text::FontBaselineOutcome::Available(value) = reference.outcome() else {
            panic!("the independently qualified script romn coordinate is present")
        };
        assert_eq!(value.coordinate(), if index == 0 { 1.0 } else { -1.0 });
        assert_eq!(
            value.provenance(),
            text::FontBaselineProvenance::BaseCoordinate
        );
        assert_eq!(
            value.base_script(),
            Some(if index == 0 { *b"latn" } else { *b"grek" })
        );
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
            [(if index == 0 { 35 } else { 274 }, 0.0, 0.0, 0.0, 10.0)]
        );
    }
    assert!(runs[0].face().is_same_face(&runs[1].face()));
    assert_eq!(
        placed
            .glyph_runs()
            .iter()
            .map(|run| run.origin())
            .collect::<Vec<_>>(),
        expected_origins
    );
    let metrics = placed.inline_metric_facts();
    assert_eq!(metrics.lines()[0].line().baseline(), common);
    assert_eq!(
        metrics
            .runs()
            .iter()
            .map(|run| run.baseline())
            .collect::<Vec<_>>(),
        if horizontal {
            vec![10.0, 8.0]
        } else {
            vec![common - 1.0, common + 1.0]
        }
    );
    for (range, expected) in [
        (
            text::Range::new(0, 1),
            if horizontal {
                text::Rect::new(0.0, 2.0, 10.0, 10.0)
            } else {
                text::Rect::new(common - 3.0, 0.0, 10.0, 10.0)
            },
        ),
        (
            text::Range::new(1, 2),
            if horizontal {
                text::Rect::new(10.0, 0.0, 10.0, 10.0)
            } else {
                text::Rect::new(common - 1.0, 10.0, 10.0, 10.0)
            },
        ),
    ] {
        assert_eq!(
            placed
                .source_range_fragments(&origin, range)
                .unwrap()
                .units()
                .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [expected]
        );
    }
    let first_caret = if horizontal {
        text::Rect::new(0.0, 0.0, 1.0, 12.0)
    } else {
        text::Rect::new(common - 3.0, 0.0, 12.0, 1.0)
    };
    assert_eq!(
        placed
            .cursor(text::Cursor::new(0, text::Affinity::After))
            .unwrap()
            .rect(),
        first_caret
    );
    assert_eq!(system.stats().layout_misses(), 0);
    // The exact immutable batch owns its fonts and geometry after the Text
    // system is gone. Render cannot query it or apply another BASE correction.
    drop(system);
    let limits = render::TextLayoutLimits::new(
        render::GlyphRunLimits::new(8, 1024, 8 << 20, 8 << 20),
        8,
        8,
        8,
    );
    for outer in [
        render::Transform::identity(),
        render::Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap(),
    ] {
        let paint = render::prepare_text_layout(placed, outer, limits).unwrap();
        assert!(std::ptr::eq(paint.layout(), placed));
        assert_eq!(paint.scene().len(), 2);
        assert_eq!(paint.layout().source(), &source);
        assert_eq!(
            paint
                .layout()
                .run_views()
                .map(|run| run.origin())
                .collect::<Vec<_>>(),
            expected_origins
        );
        assert_eq!(
            paint
                .layout()
                .cursor(text::Cursor::new(0, text::Affinity::After))
                .unwrap()
                .rect(),
            first_caret
        );
    }
}

#[test]
fn actual_used_alphabetic_script_origins_reach_render_from_named_layout() {
    let bytes = font();
    qualify_font(&bytes);
    // Confirm real asymmetric ink through Render's public headless owner before
    // composing Layout geometry; neither expected bound comes from Text output.
    let render_font = render::FontData::try_from_bytes(bytes.clone(), 0).unwrap();
    let instance =
        render::TextInstance::try_new(10.0, vec![], render::GlyphSynthesis::none()).unwrap();
    for (glyph, expected) in [
        (35, render::Rect::try_new(0.0, -8.0, 10.0, 10.0).unwrap()),
        (274, render::Rect::try_new(0.0, -6.0, 10.0, 2.0).unwrap()),
    ] {
        let admitted = render_font
            .prepare_glyph_run(
                &instance,
                &[render::TextGlyph::try_new(glyph, 0.0, 0.0, 10.0).unwrap()],
                render::GlyphRunLimits::new(1, 64, 1 << 20, 1 << 20),
            )
            .unwrap();
        assert_eq!(admitted.bounds().ink_rect(), Some(expected));
    }
    for mode in [
        layout::WritingMode::HorizontalTb,
        layout::WritingMode::SidewaysRl,
        layout::WritingMode::VerticalRl,
        layout::WritingMode::VerticalLr,
    ] {
        composition(&bytes, mode);
    }
}
