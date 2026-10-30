//! Actual Mixed Text runs share a Vertical frame but retain distinct outline axes.
//! Licensed Ahem chi274 is a horizontal stripe; glyph277 is a vertical stripe.
//! Literal native vmtx1200/TSB100 distinguishes upright12 from horizontal10.
use super::*;
use kurbo::Shape as _;
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
    raw::TableProvider,
};

const MAPPINGS: [(u32, u32); 6] = [
    (0x3a7, 274),
    (0x5d0, 274),
    (0x5d1, 277),
    (0x3001, 274),
    (0x3002, 277),
    (0x4e00, 274),
];

fn font() -> Vec<u8> {
    // Author only these tables, using the existing parent SFNT rewriter.
    // Glyph outlines, horizontal metrics and family remain the licensed Ahem.
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10];
    cmap.extend(12_u32.to_be_bytes());
    cmap.extend([0, 12, 0, 0]);
    cmap.extend(
        u32::try_from(16 + MAPPINGS.len() * 12)
            .unwrap()
            .to_be_bytes(),
    );
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(u32::try_from(MAPPINGS.len()).unwrap().to_be_bytes());
    for (cp, glyph) in MAPPINGS {
        cmap.extend(cp.to_be_bytes());
        cmap.extend(cp.to_be_bytes());
        cmap.extend(glyph.to_be_bytes());
    }
    let face = skrifa::FontRef::new(AHEM).unwrap();
    let count = face.maxp().unwrap().num_glyphs();
    let metrics = face.glyph_metrics(Size::unscaled(), LocationRef::default());
    let max_height = (0..count)
        .map(|id| {
            let bounds = metrics.bounds(GlyphId::new(u32::from(id))).unwrap();
            assert_eq!((bounds.y_max - bounds.y_min).fract(), 0.0);
            (bounds.y_max - bounds.y_min) as i16
        })
        .max()
        .unwrap();
    let mut vhea = 0x0001_0000_u32.to_be_bytes().to_vec();
    for value in [1000_i16, -1000, 0] {
        vhea.extend(value.to_be_bytes());
    }
    vhea.extend(1200_u16.to_be_bytes());
    for value in [
        100_i16,
        1100 - max_height,
        100 + max_height,
        0,
        1,
        0,
        0,
        0,
        0,
        0,
        0,
    ] {
        vhea.extend(value.to_be_bytes());
    }
    vhea.extend(count.to_be_bytes());
    assert_eq!(vhea.len(), 36);
    let mut vmtx = Vec::new();
    for _ in 0..count {
        vmtx.extend(1200_u16.to_be_bytes());
        vmtx.extend(100_i16.to_be_bytes());
    }
    replace_tables(vec![(*b"cmap", cmap), (*b"vhea", vhea), (*b"vmtx", vmtx)])
}

fn premises(bytes: &[u8]) {
    for tag in [*b"glyf", *b"loca", *b"hhea", *b"hmtx", *b"maxp"] {
        assert_eq!(table(bytes, tag), table(AHEM, tag));
    }
    let face = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(u32_at(table(bytes, *b"vhea"), 0), 0x0001_0000);
    assert_eq!(face.vhea().unwrap().number_of_long_ver_metrics(), 281);
    let hhea = face.hhea().unwrap();
    assert_eq!(
        (hhea.ascender().to_i16(), hhea.descender().to_i16()),
        (800, -200)
    );
    for tag in [
        b"BASE", b"GSUB", b"GPOS", b"fvar", b"CFF ", b"CFF2", b"VARC",
    ] {
        assert!(face.table_data(skrifa::Tag::new(tag)).is_none());
    }
    for (cp, glyph) in MAPPINGS {
        assert_eq!(face.charmap().map(cp), Some(GlyphId::new(glyph)));
    }
    let metrics = face.glyph_metrics(Size::unscaled(), LocationRef::default());
    for (id, bounds) in [
        (274, [0.0, 400.0, 1000.0, 600.0]),
        (277, [200.0, -200.0, 400.0, 800.0]),
    ] {
        assert_eq!(metrics.advance_width(GlyphId::new(id)), Some(1000.0));
        let actual = metrics.bounds(GlyphId::new(id)).unwrap();
        assert_eq!(
            [actual.x_min, actual.y_min, actual.x_max, actual.y_max],
            bounds
        );
        let row = face.vmtx().unwrap().v_metrics()[id as usize];
        assert_eq!((row.advance.get(), row.side_bearing.get()), (1200, 100));
    }
}

fn compose(
    source: &str,
    direction: text::ParagraphBaseDirection,
) -> (text::Layout, text::FinalizedLine, text::System, Vec<u8>) {
    let bytes = font();
    premises(&bytes);
    let mut system = system(bytes.clone());
    let face = system.registered_font_faces().next().unwrap().to_owned();
    let instance = face.unsynthesized_instance(10.0, &[]).unwrap();
    // No BASE: HorizontalY Central=(800-200)/2=300 design units;
    // vhea1.0 supplies nominal-pen upright synthesis, used sides +/-500.
    let central = instance
        .baseline(
            system.font_catalogue_identity(),
            text::FontBaselineScript::from_iso15924(*b"Grek").unwrap(),
            text::FontBaselineAxis::HorizontalY,
            text::FontBaseline::Central,
        )
        .unwrap();
    let text::FontBaselineOutcome::Available(central) = central.outcome() else {
        panic!("the genuine missing-BASE HorizontalY synthesis is available")
    };
    assert_eq!(central.coordinate(), 3.0);
    let cross = instance
        .upright_metrics(
            system.font_catalogue_identity(),
            text::FontBaselineScript::from_iso15924(*b"Hani").unwrap(),
        )
        .unwrap();
    assert_eq!(
        cross.reference(),
        text::UprightMetricReference::NominalVerticalPenX
    );
    assert_eq!(
        (
            cross.central_coordinate_or_distance(),
            cross.used_under(),
            cross.used_over()
        ),
        (0.0, -5.0, 5.0)
    );
    for (glyph, y) in [(274, 7.0), (277, 9.0)] {
        let nominal = instance
            .vertical_glyph_metrics(glyph, system.font_catalogue_identity())
            .unwrap();
        assert_eq!(
            (nominal.advance(), nominal.origin_x(), nominal.origin_y()),
            (12.0, 5.0, y)
        );
    }
    let source = text::Source::identified(source, text::Id::from_u64(1570060), 3);
    let analyzed = text::AnalyzedSource::new(
        source.clone(),
        text::Style {
            size: 10.0,
            line_height: text::LineHeight::Absolute(10.0),
            ..style()
        },
        text::AnalysisTypography::Mixed,
    )
    .unwrap();
    assert_eq!(
        analyzed.direction(),
        match direction {
            text::ParagraphBaseDirection::LeftToRight => text::Direction::LeftToRight,
            text::ParagraphBaseDirection::RightToLeft => text::Direction::RightToLeft,
        }
    );
    let prepared = system
        .prepare(
            &analyzed,
            text::PreparationOptions::default().with_orientation(text::LineOrientation::Vertical),
        )
        .unwrap();
    let end = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let line = prepared
        .select(text::LineSelectionRequest {
            segment: prepared.source_segments().next().unwrap(),
            start: prepared.boundary_views().next().unwrap().handle(),
            end: end.view().boundary(),
            termination: text::PreparedTermination::Ordinary(end),
            participants: prepared.participants().collect(),
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
                base_direction: direction,
                line_box_direction: match direction {
                    text::ParagraphBaseDirection::LeftToRight => {
                        text::LineBoxDirection::LeftToRight
                    }
                    text::ParagraphBaseDirection::RightToLeft => {
                        text::LineBoxDirection::RightToLeft
                    }
                },
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
        .unwrap()
        .finalize(surgeist_text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: surgeist_text::ScopeAllocations::None,
        })
        .unwrap();
    let layout = text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared,
        coverage: text::PlacementCoverage::WholeParagraph,
        entries: vec![text::PlacementEntry::Formatted(text::LinePlacement {
            line: line.clone(),
            anchor: text::LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: text::LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), &source);
    assert_eq!(layout.analyzed_source().identity(), analyzed.identity());
    (layout, line, system, bytes)
}

fn packets(line: &text::FinalizedLine) -> Vec<(text::Range, text::GlyphBasis, u32, f32, f32, f32)> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.range(),
                        run.glyph_basis(),
                        glyph.id(),
                        glyph.x(),
                        glyph.y(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn admitted(run: &PreparedGlyphRun, bytes: &[u8]) {
    assert_eq!(run.font().bytes(), bytes);
    assert_eq!(run.font().index(), 0);
    assert_eq!(run.instance().size(), 10.0);
    assert!(run.instance().coordinates().is_empty());
    assert_eq!(run.instance().synthesis(), GlyphSynthesis::none());
}

fn ink(run: &PreparedGlyphRun, transform: Transform) -> kurbo::Rect {
    let mut path = run.path().clone();
    path.apply_affine(kurbo::Affine::new(transform.as_array()));
    path.bounding_box()
}

#[test]
fn mixed_clockwise_and_upright_outlines_keep_distinct_metrics_pens_and_outer_transform_once() {
    let (layout, line, system, bytes) =
        compose("ΧΧ一一Χ", text::ParagraphBaseDirection::LeftToRight);
    let before = system.stats();
    let r = text::GlyphBasis::Clockwise;
    let u = text::GlyphBasis::Unrotated;
    assert_eq!(line.feedback().advance, 54.0);
    assert_eq!(
        packets(&line),
        [
            (text::Range::new(0, 1), r, 274, 0.0, 0.0, 10.0),
            (text::Range::new(1, 2), r, 274, 0.0, 0.0, 10.0),
            (text::Range::new(2, 3), u, 274, -5.0, 7.0, 12.0),
            (text::Range::new(3, 4), u, 274, -5.0, 7.0, 12.0),
            (text::Range::new(4, 5), r, 274, 0.0, 0.0, 10.0),
        ]
    );
    let runs = layout.run_views().collect::<Vec<_>>();
    assert_eq!(
        runs.iter()
            .map(|run| (run.range(), run.glyph_basis(), run.origin()))
            .collect::<Vec<_>>(),
        [
            (text::Range::new(0, 2), r, text::Point::new(97.0, 20.0)),
            (text::Range::new(2, 4), u, text::Point::new(100.0, 40.0)),
            (text::Range::new(4, 5), r, text::Point::new(97.0, 64.0)),
        ]
    );
    for run in &runs {
        assert_eq!(run.orientation(), text::LineOrientation::Vertical);
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.font_size(), 10.0);
    }
    for (outer, matrices, physical) in [
        (
            Transform::identity(),
            [
                [0.0, 1.0, -1.0, 0.0, 97.0, 20.0],
                [1.0, 0.0, 0.0, 1.0, 100.0, 40.0],
                [0.0, 1.0, -1.0, 0.0, 97.0, 64.0],
            ],
            [
                kurbo::Rect::new(101.0, 20.0, 103.0, 40.0),
                kurbo::Rect::new(95.0, 41.0, 105.0, 55.0),
                kurbo::Rect::new(101.0, 64.0, 103.0, 74.0),
            ],
        ),
        (
            Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap(),
            [
                [0.0, 3.0, -2.0, 0.0, 201.0, 69.0],
                [2.0, 0.0, 0.0, 3.0, 207.0, 129.0],
                [0.0, 3.0, -2.0, 0.0, 201.0, 201.0],
            ],
            [
                kurbo::Rect::new(209.0, 69.0, 213.0, 129.0),
                kurbo::Rect::new(197.0, 132.0, 217.0, 174.0),
                kurbo::Rect::new(209.0, 201.0, 213.0, 231.0),
            ],
        ),
    ] {
        let scene = prepare_text_layout(&layout, outer, limits()).unwrap();
        assert!(std::ptr::eq(scene.layout(), &layout));
        assert_eq!(system.stats(), before);
        assert_eq!(scene.scene().commands.len(), 3);
        let glyphs = [
            vec![
                TextGlyph::try_new(274, 0.0, 0.0, 10.0).unwrap(),
                TextGlyph::try_new(274, 10.0, 0.0, 10.0).unwrap(),
            ],
            vec![
                TextGlyph::try_new(274, -5.0, 7.0, 12.0).unwrap(),
                TextGlyph::try_new(274, -5.0, 19.0, 12.0).unwrap(),
            ],
            vec![TextGlyph::try_new(274, 0.0, 0.0, 10.0).unwrap()],
        ];
        let local = [
            Rect::new(0.0, -6.0, 20.0, 2.0),
            Rect::new(-5.0, 1.0, 10.0, 14.0),
            Rect::new(0.0, -6.0, 10.0, 2.0),
        ];
        let mut copied = None;
        for (index, command) in scene.scene().commands.iter().enumerate() {
            let (run, transform) = outline(command);
            admitted(run, &bytes);
            assert_eq!(run.glyphs(), glyphs[index]);
            assert_eq!(run.bounds().ink_rect(), Some(local[index]));
            assert_eq!(transform.as_array(), matrices[index]);
            assert_eq!(ink(run, transform), physical[index]);
            assert_ne!(
                run.font().bytes().as_ptr(),
                runs[index].face().bytes().as_ptr()
            );
            if let Some(first) = copied {
                assert_eq!(run.font().bytes().as_ptr(), first);
            } else {
                copied = Some(run.font().bytes().as_ptr());
            }
        }
    }
    let published = prepare_text_layout(&layout, Transform::identity(), limits()).unwrap();
    let retained = published.scene().clone();
    let error = refuse(
        &layout,
        Transform::identity(),
        bounded(4, 1024, ROOM, 32, 32, 32),
    );
    assert_limit(&error, TextLayoutResource::Glyphs, 5, 4);
    assert_eq!(
        ink(
            outline(&published.scene().commands[1]).0,
            outline(&published.scene().commands[1]).1
        ),
        kurbo::Rect::new(95.0, 41.0, 105.0, 55.0)
    );
    assert_eq!(system.stats(), before);
    drop(system);
    assert!(prepare_text_layout(&layout, Transform::identity(), limits()).is_ok());
    drop(published);
    drop(runs);
    drop(layout);
    drop(line);
    let (run, transform) = outline(&retained.commands[1]);
    admitted(run, &bytes);
    assert_eq!(
        ink(run, transform),
        kurbo::Rect::new(95.0, 41.0, 105.0, 55.0)
    );
}

#[test]
fn mixed_odd_upright_punctuation_and_hebrew_rotate_only_r_outlines_and_reverse_source_once() {
    // UTR50 gives the two ideographic punctuation marks Tu; Hebrew is R.
    // Both ON punctuation units resolve level1 between actual Hebrew R units.
    // BTT therefore emits source2 then1, and L2 places source3,2,1,0 once.
    let (layout, line, system, bytes) =
        compose("א、。ב", text::ParagraphBaseDirection::RightToLeft);
    let before = system.stats();
    assert_eq!(
        line.prepared_text()
            .unit_views()
            .map(|unit| unit.level().get())
            .collect::<Vec<_>>(),
        [1; 4]
    );
    let r = text::GlyphBasis::Clockwise;
    let u = text::GlyphBasis::Unrotated;
    assert_eq!(line.feedback().advance, 44.0);
    assert_eq!(
        packets(&line),
        [
            (text::Range::new(3, 4), r, 277, 0.0, 0.0, 10.0),
            (text::Range::new(2, 3), u, 277, -5.0, 9.0, 12.0),
            (text::Range::new(1, 2), u, 274, -5.0, 7.0, 12.0),
            (text::Range::new(0, 1), r, 274, 0.0, 0.0, 10.0),
        ]
    );
    let source_runs = layout.run_views().collect::<Vec<_>>();
    assert_eq!(
        source_runs
            .iter()
            .map(|run| (run.range(), run.glyph_basis(), run.origin()))
            .collect::<Vec<_>>(),
        [
            (text::Range::new(3, 4), r, text::Point::new(97.0, 20.0)),
            (text::Range::new(1, 3), u, text::Point::new(100.0, 30.0)),
            (text::Range::new(0, 1), r, text::Point::new(97.0, 54.0)),
        ]
    );
    let scene =
        prepare_text_layout(&layout, Transform::translation(7.0, 9.0).unwrap(), limits()).unwrap();
    assert!(std::ptr::eq(scene.layout(), &layout));
    assert_eq!(scene.scene().commands.len(), 3);
    let expected = [
        (
            vec![TextGlyph::try_new(277, 0.0, 0.0, 10.0).unwrap()],
            [0.0, 1.0, -1.0, 0.0, 104.0, 29.0],
            kurbo::Rect::new(102.0, 31.0, 112.0, 33.0),
        ),
        (
            vec![
                TextGlyph::try_new(277, -5.0, 9.0, 12.0).unwrap(),
                TextGlyph::try_new(274, -5.0, 19.0, 12.0).unwrap(),
            ],
            [1.0, 0.0, 0.0, 1.0, 107.0, 39.0],
            kurbo::Rect::new(102.0, 40.0, 112.0, 54.0),
        ),
        (
            vec![TextGlyph::try_new(274, 0.0, 0.0, 10.0).unwrap()],
            [0.0, 1.0, -1.0, 0.0, 104.0, 63.0],
            kurbo::Rect::new(108.0, 63.0, 110.0, 73.0),
        ),
    ];
    for (command, (glyphs, matrix, physical)) in scene.scene().commands.iter().zip(expected) {
        let (run, transform) = outline(command);
        admitted(run, &bytes);
        assert_eq!(run.glyphs(), glyphs);
        assert_eq!(transform.as_array(), matrix);
        assert_eq!(ink(run, transform), physical);
    }
    assert_eq!(system.stats(), before);
    assert_eq!(
        layout.source().text().code_units(),
        [0x5d0, 0x3001, 0x3002, 0x5d1]
    );
    assert_eq!(layout.source().id(), Some(text::Id::from_u64(1570060)));
    assert_eq!(layout.source().revision(), 3);
}
