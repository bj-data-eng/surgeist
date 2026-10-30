//! Real whole-upright Text output must retain unrotated asymmetric outlines.
//! The literal oracle is the unchanged Ahem chi stripe and nominal vertical
//! absence synthesis. An authored cmap maps real Wide/Hani source to that
//! retained outline; no Han design is claimed. Unlike isolated horizontal-script
//! units, these vertical units share a real provider run with two inline pens.
use super::*;
use kurbo::Shape as _;
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
    raw::TableProvider,
};

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let record = (0..usize::from(u16_at(bytes, 4)))
        .map(|index| 12 + index * 16)
        .find(|record| bytes[*record..*record + 4] == tag)
        .unwrap();
    let start = usize::try_from(u32_at(bytes, record + 8)).unwrap();
    let len = usize::try_from(u32_at(bytes, record + 12)).unwrap();
    &bytes[start..start + len]
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

fn chi_font() -> Vec<u8> {
    // A real format12 cmap with one group U+4E00 -> glyph274. All outline,
    // loca, glyph inventory and horizontal metrics bytes remain licensed Ahem.
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10];
    cmap.extend(12_u32.to_be_bytes());
    cmap.extend([0, 12, 0, 0]);
    cmap.extend(28_u32.to_be_bytes());
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(1_u32.to_be_bytes());
    cmap.extend(0x4e00_u32.to_be_bytes());
    cmap.extend(0x4e00_u32.to_be_bytes());
    cmap.extend(274_u32.to_be_bytes());
    let mut tables = (0..usize::from(u16_at(AHEM, 4)))
        .map(|index| {
            let at = 12 + index * 16;
            let tag: [u8; 4] = AHEM[at..at + 4].try_into().unwrap();
            (
                tag,
                if tag == *b"cmap" {
                    cmap.clone()
                } else {
                    table(AHEM, tag).to_vec()
                },
            )
        })
        .collect::<Vec<_>>();
    tables.sort_by_key(|(tag, _)| *tag);
    let mut output = vec![0; 12 + tables.len() * 16];
    output[..12].copy_from_slice(&AHEM[..12]);
    for (index, (tag, mut data)) in tables.into_iter().enumerate() {
        if tag == *b"head" {
            data[8..12].fill(0);
        }
        while !output.len().is_multiple_of(4) {
            output.push(0);
        }
        let record = 12 + index * 16;
        let start = u32::try_from(output.len()).unwrap();
        output[record..record + 4].copy_from_slice(&tag);
        output[record + 4..record + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        output[record + 8..record + 12].copy_from_slice(&start.to_be_bytes());
        output[record + 12..record + 16]
            .copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        output.extend(data);
    }
    while !output.len().is_multiple_of(4) {
        output.push(0);
    }
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&output));
    let head = (0..usize::from(u16_at(&output, 4)))
        .map(|index| 12 + index * 16)
        .find(|record| output[*record..*record + 4] == *b"head")
        .unwrap();
    let start = usize::try_from(u32_at(&output, head + 8)).unwrap();
    output[start + 8..start + 12].copy_from_slice(&adjustment.to_be_bytes());
    output
}

fn upright(mut authored: text::Style) -> (text::Layout, text::System, Vec<u8>) {
    authored.size = 10.0;
    authored.line_height = text::LineHeight::Absolute(10.0);
    let bytes = chi_font();
    assert_eq!(checksum(&bytes), 0xb1b0_afba);
    for tag in [*b"glyf", *b"loca", *b"maxp", *b"hmtx", *b"hhea"] {
        assert_eq!(table(&bytes, tag), table(AHEM, tag));
    }
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.charmap().map(0x4e00_u32), Some(GlyphId::new(274)));
    let hhea = font.hhea().unwrap();
    assert_eq!(
        (
            hhea.ascender().to_i16(),
            hhea.descender().to_i16(),
            hhea.line_gap().to_i16()
        ),
        (800, -200, 0)
    );
    for tag in [
        b"BASE", b"vhea", b"vmtx", b"fvar", b"CFF ", b"CFF2", b"VARC",
    ] {
        assert!(
            font.table_data(skrifa::Tag::new(tag)).is_none(),
            "absent {tag:?}"
        );
    }
    assert!(font.glyf().is_ok());
    let metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());
    assert_eq!(metrics.advance_width(GlyphId::new(274)), Some(1000.0));
    let bounds = metrics.bounds(GlyphId::new(274)).unwrap();
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0, 400.0, 1000.0, 600.0]
    );
    let mut system = system(bytes.clone());
    let catalogue = system.font_catalogue_identity().clone();
    let face = system.registered_font_faces().next().unwrap().to_owned();
    let instance = face.unsynthesized_instance(10.0, &[]).unwrap();
    // With no nominal vertical tables, advance=A-D=1000, Ox=W/2=500,
    // Oy=A=800. With no BASE/vhea, the glyph-free cross em is centered at
    // the nominal pen: Central0 and used sides[-500,+500]. Scale once by.01.
    let nominal = instance.vertical_glyph_metrics(274, &catalogue).unwrap();
    assert_eq!(
        (nominal.advance(), nominal.origin_x(), nominal.origin_y()),
        (10.0, 5.0, 8.0)
    );
    let cross = instance
        .upright_metrics(
            &catalogue,
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
    let typography = text::AnalysisTypography::Upright {
        computed_direction: text::ParagraphBaseDirection::RightToLeft,
    };
    let analyzed = text::AnalyzedSource::new(
        text::Source::identified("\u{4e00}\u{4e00}", text::Id::from_u64(7811544), 7),
        authored,
        typography,
    )
    .unwrap();
    assert_eq!(analyzed.typography(), typography);
    assert_eq!(analyzed.direction(), text::Direction::LeftToRight);
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        [text::Range::new(0, 1), text::Range::new(1, 2)]
    );
    let prepared = system
        .prepare(
            &analyzed,
            text::PreparationOptions::default().with_orientation(text::LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.orientation(), text::LineOrientation::Vertical);
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    let end = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let line = prepared
        .select(text::LineSelectionRequest {
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
        .unwrap()
        .finalize(surgeist_text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: surgeist_text::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        (line.feedback().ascent, line.feedback().descent),
        (5.0, 5.0)
    );
    assert_eq!(
        line.run_views()
            .flat_map(|run| {
                assert_eq!(run.glyph_basis(), text::GlyphBasis::Unrotated);
                run.glyphs()
                    .map(|glyph| {
                        (
                            glyph.range(),
                            glyph.id(),
                            glyph.x(),
                            glyph.y(),
                            glyph.advance(),
                            glyph.pen(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        [
            (text::Range::new(0, 1), 274, -5.0, 8.0, 10.0, 0.0),
            (text::Range::new(1, 2), 274, -5.0, 8.0, 10.0, 10.0),
        ]
    );
    let layout = text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared,
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
    assert_eq!(layout.source().text().code_units(), [0x4e00, 0x4e00]);
    assert_eq!(layout.source().id(), Some(text::Id::from_u64(7811544)));
    assert_eq!(layout.source().revision(), 7);
    assert_eq!(layout.analyzed_source().identity(), analyzed.identity());
    assert_eq!(layout.bounds(), text::Rect::new(95.0, 20.0, 10.0, 20.0));
    let runs = layout.run_views().collect::<Vec<_>>();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].origin(), text::Point::new(100.0, 20.0));
    assert_eq!(runs[0].glyph_basis(), text::GlyphBasis::Unrotated);
    assert!(runs[0].face().is_same_face(&face.face()));
    assert_eq!(runs[0].face().catalogue_identity(), &catalogue);
    assert_eq!(runs[0].range(), text::Range::new(0, 2));
    (layout, system, bytes)
}

fn commanded_ink(run: &PreparedGlyphRun, transform: Transform) -> kurbo::Rect {
    let mut path = run.path().clone();
    path.apply_affine(kurbo::Affine::new(transform.as_array()));
    path.bounding_box()
}

#[test]
fn upright_asymmetric_outlines_keep_raw_axes_and_vertical_pens_through_outer_placement() {
    let (input, system, bytes) = upright(style());
    let stats = system.stats();
    let source_run = input.run_views().next().unwrap();
    let instance = source_run.effective_instance().unwrap();
    for (outer, coefficients, ink) in [
        (
            Transform::identity(),
            [1.0, 0.0, 0.0, 1.0, 100.0, 20.0],
            kurbo::Rect::new(95.0, 22.0, 105.0, 34.0),
        ),
        (
            Transform::translation(7.0, 9.0).unwrap(),
            [1.0, 0.0, 0.0, 1.0, 107.0, 29.0],
            kurbo::Rect::new(102.0, 31.0, 112.0, 43.0),
        ),
        (
            Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap(),
            [2.0, 0.0, 0.0, 3.0, 207.0, 69.0],
            kurbo::Rect::new(197.0, 75.0, 217.0, 111.0),
        ),
    ] {
        let scene = prepare_text_layout(&input, outer, limits())
            .expect("native upright Text outlines must reach the direct Render adapter");
        assert!(std::ptr::eq(scene.layout(), &input));
        assert_eq!(
            system.stats(),
            stats,
            "Render consumes the existing layout without reshaping"
        );
        assert_eq!(scene.scene().commands.len(), 1);
        let (run, transform) = outline(&scene.scene().commands[0]);
        assert_eq!(transform.as_array(), coefficients);
        assert_eq!(run.font().bytes(), bytes);
        assert_eq!(run.font().bytes(), instance.face().bytes());
        assert_eq!(run.font().index(), instance.face().index());
        assert_eq!(run.instance().size(), instance.font_size());
        assert!(run.instance().coordinates().is_empty());
        assert_eq!(instance.normalized_coords().collect::<Vec<_>>(), []);
        assert_eq!(run.instance().synthesis(), GlyphSynthesis::none());
        assert_eq!(
            run.glyphs(),
            [
                TextGlyph::try_new(274, -5.0, 8.0, 10.0).unwrap(),
                TextGlyph::try_new(274, -5.0, 18.0, 10.0).unwrap(),
            ]
        );
        // Original chi ink x0..10,y-6..-4 receives raw offset(-5,+8).
        // The second pen10 moves only physical y. An accidental cell rotation
        // or adding the pen to x changes this retained path's actual bounds.
        assert_eq!(
            run.bounds().ink_rect(),
            Some(Rect::try_new(-5.0, 2.0, 10.0, 12.0).unwrap())
        );
        assert_eq!(commanded_ink(run, transform), ink);
    }
    let error = refuse(
        &input,
        Transform::identity(),
        bounded(1, 1024, ROOM, 32, 32, 32),
    );
    assert_limit(&error, TextLayoutResource::Glyphs, 2, 1);
    assert_eq!(system.stats(), stats);
    assert_eq!(input.source().text().code_units(), [0x4e00, 0x4e00]);
    drop(system);
    let retry = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    let (run, transform) = outline(&retry.scene().commands[0]);
    assert_eq!(run.font().bytes(), bytes);
    assert_eq!(run.glyphs().len(), 2);
    assert_eq!(
        commanded_ink(run, transform),
        kurbo::Rect::new(95.0, 22.0, 105.0, 34.0)
    );
}

#[test]
fn upright_explicit_decoration_is_already_physical_and_receives_outer_placement_once() {
    let authored = text::Style {
        underline: text::Decoration::solid()
            .with_offset(text::DecorationOffset::try_absolute(-2.0).unwrap())
            .with_thickness(text::DecorationThickness::try_absolute(1.0).unwrap()),
        ..style()
    };
    let (input, system, _bytes) = upright(authored);
    let stats = system.stats();
    let decorations = input.decoration_views().collect::<Vec<_>>();
    assert_eq!(decorations.len(), 1);
    assert_eq!(
        decorations[0].rect(),
        text::Rect::new(97.0, 20.0, 1.0, 20.0)
    );
    let outer = Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap();
    let scene = prepare_text_layout(&input, outer, limits())
        .expect("accepted upright explicit decoration must accompany real unrotated glyphs");
    assert_eq!(system.stats(), stats);
    assert_eq!(scene.scene().commands.len(), 2);
    let (run, transform) = outline(&scene.scene().commands[0]);
    assert_eq!(transform.as_array(), [2.0, 0.0, 0.0, 3.0, 207.0, 69.0]);
    assert_eq!(
        commanded_ink(run, transform),
        kurbo::Rect::new(197.0, 75.0, 217.0, 111.0)
    );
    let Command::Layer { layer, children } = &scene.scene().commands[1] else {
        panic!("physical decoration must use the single outer placement layer")
    };
    assert_eq!(layer.transform(), outer);
    assert_eq!(children.len(), 1);
    let Command::Fill { shape, .. } = &children[0] else {
        panic!("actual underline fill")
    };
    assert_eq!(
        *shape,
        crate::Shape::rect(Rect::try_new(97.0, 20.0, 1.0, 20.0).unwrap())
    );
    let error = refuse(&input, outer, bounded(32, 1024, ROOM, 32, 32, 0));
    assert_limit(&error, TextLayoutResource::Decorations, 1, 0);
    let retry = prepare_text_layout(&input, outer, limits()).unwrap();
    assert!(std::ptr::eq(retry.layout(), &input));
    assert_eq!(retry.scene().commands.len(), 2);
    let (retry_run, retry_transform) = outline(&retry.scene().commands[0]);
    // Each invocation owns a fresh font copy. Its identity is independent;
    // retained bytes, effective instance, glyph facts and executed paint agree.
    assert_eq!(retry_run.font().bytes(), run.font().bytes());
    assert_eq!(retry_run.font().index(), run.font().index());
    assert_eq!(retry_run.instance(), run.instance());
    assert_eq!(retry_run.glyphs(), run.glyphs());
    assert_eq!(retry_run.path(), run.path());
    assert_eq!(retry_transform, transform);
    assert_eq!(
        commanded_ink(retry_run, retry_transform),
        kurbo::Rect::new(197.0, 75.0, 217.0, 111.0)
    );
    let Command::Layer {
        layer: retry_layer,
        children: retry_children,
    } = &retry.scene().commands[1]
    else {
        panic!("retry retains physical decoration's single outer layer")
    };
    assert_eq!(retry_layer.transform(), outer);
    assert_eq!(retry_children.len(), 1);
    let Command::Fill {
        shape: retry_shape,
        paint: retry_paint,
    } = &retry_children[0]
    else {
        panic!("retry retains the actual underline fill")
    };
    let Command::Fill { paint, .. } = &children[0] else {
        unreachable!()
    };
    assert_eq!(retry_shape, shape);
    assert_eq!(retry_paint, paint);
    assert_eq!(system.stats(), stats);
}

#[test]
fn upright_native_synthesis_uses_original_font_axes_once_before_outer_scaling() {
    for (font, italic, bold) in [
        (
            text::Font::new().family("Ahem").style(text::Slant::Italic),
            true,
            false,
        ),
        (
            text::Font::new().family("Ahem").weight(text::Weight::Bold),
            false,
            true,
        ),
        (
            text::Font::new()
                .family("Ahem")
                .weight(text::Weight::Bold)
                .style(text::Slant::Italic),
            true,
            true,
        ),
    ] {
        let (input, system, bytes) = upright(text::Style { font, ..style() });
        let source_run = input.run_views().next().unwrap();
        let instance = source_run.effective_instance().unwrap();
        assert_eq!(instance.x_shear() > 0.0, italic);
        assert_eq!(instance.embolden() > 0.0, bold);
        let skew = f64::from(instance.x_shear());
        let copy = f64::from(instance.embolden());
        // Original chi points are x0..10 at font-up y4..6. Existing synthesis
        // applies x+=skew*font_y, then an optional positive local-x contour
        // copy. Raw offset(-5,+8) is separate from that operation. Each final
        // local point narrows once to f32, before the outer affine transform.
        let left = f64::from((-5.0 + 4.0 * skew) as f32);
        let right = f64::from((5.0 + 6.0 * skew + copy) as f32);
        let outer = Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap();
        let stats = system.stats();
        let scene = prepare_text_layout(&input, outer, limits()).unwrap();
        assert_eq!(system.stats(), stats);
        assert!(std::ptr::eq(scene.layout(), &input));
        assert_eq!(scene.scene().commands.len(), 1);
        let (run, transform) = outline(&scene.scene().commands[0]);
        assert_eq!(run.font().bytes(), bytes);
        assert_eq!(run.font().index(), instance.face().index());
        assert_eq!(run.instance().size(), instance.font_size());
        assert!(run.instance().coordinates().is_empty());
        assert_eq!(
            run.instance().synthesis(),
            GlyphSynthesis::try_new(instance.x_shear(), instance.embolden()).unwrap()
        );
        assert_eq!(
            run.glyphs(),
            [
                TextGlyph::try_new(274, -5.0, 8.0, 10.0).unwrap(),
                TextGlyph::try_new(274, -5.0, 18.0, 10.0).unwrap(),
            ]
        );
        assert_eq!(transform.as_array(), [2.0, 0.0, 0.0, 3.0, 207.0, 69.0]);
        assert_eq!(
            run.bounds().ink_rect(),
            Some(Rect::try_new(left, 2.0, right - left, 12.0).unwrap())
        );
        assert_eq!(
            commanded_ink(run, transform),
            kurbo::Rect::new(207.0 + 2.0 * left, 75.0, 207.0 + 2.0 * right, 111.0)
        );
    }
}
