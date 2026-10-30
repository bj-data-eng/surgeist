//! Native selected CCW Text reaches the direct Render owner. Authored GPOS
//! and retained asymmetric Ahem outlines supply independent numeric oracles.
use super::*;
use kurbo::Shape as _;
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
    raw::TableProvider,
};

const GX: f64 = -3.25;
const GY: f64 = 40.5;
const CCW: text::LineOrientation = text::LineOrientation::SidewaysCounterclockwise;

fn words(values: &[u16]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

#[derive(Clone, Copy)]
enum Representation {
    Outline,
    Color,
    Bitmap,
}

fn font(representation: Representation, positioned: bool) -> Vec<u8> {
    // Reuse the parent's retained SFNT rewriter. Only authored table payloads
    // are new; licensed glyf/loca/hmtx/hhea/name remain the actual resources.
    let entries = [(0x41_u32, 274_u32), (0x42, 277), (0x5d0, 274), (0x5d1, 277)];
    let mut cmap = words(&[0, 1, 3, 10]);
    cmap.extend(12_u32.to_be_bytes());
    cmap.extend(words(&[12, 0]));
    cmap.extend(64_u32.to_be_bytes()); // Format12 header16 plus four groups12.
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(4_u32.to_be_bytes());
    for (cp, glyph) in entries {
        cmap.extend(cp.to_be_bytes());
        cmap.extend(cp.to_be_bytes());
        cmap.extend(glyph.to_be_bytes());
    }
    let mut replacements = vec![(*b"cmap", cmap)];
    if positioned {
        // DFLT -> kern -> SinglePos1 on chi274. At size10/UPEM1000:
        // xPlacement125=+1.25, yPlacement-250=>yDown+2.5, xAdvance125=>+1.25.
        let mut gpos = words(&[1, 0, 10, 30, 44, 1]);
        gpos.extend(*b"DFLT");
        gpos.extend(words(&[8, 4, 0, 0, 0xffff, 1, 0, 1]));
        gpos.extend(*b"kern");
        gpos.extend(words(&[
            8,
            0,
            1,
            0,
            1,
            4,
            1,
            0,
            1,
            8,
            1,
            12,
            7,
            125,
            (-250_i16) as u16,
            125,
            1,
            1,
            274,
        ]));
        replacements.push((*b"GPOS", gpos));
    }
    match representation {
        Representation::Outline => {}
        Representation::Color => {
            // COLRv0 root chi274 paints real vertical-stripe glyph277 in red.
            let mut colr = words(&[0, 1]);
            colr.extend(14_u32.to_be_bytes());
            colr.extend(20_u32.to_be_bytes());
            colr.extend(words(&[1, 274, 0, 1, 277, 0]));
            let mut cpal = words(&[0, 1, 1, 1]);
            cpal.extend(14_u32.to_be_bytes());
            cpal.extend(words(&[0]));
            cpal.extend([0, 0, 255, 255]);
            replacements.extend([(*b"COLR", colr), (*b"CPAL", cpal)]);
        }
        Representation::Bitmap => {
            // Existing PNG dependency, no acquired asset or alternate encoder.
            let mut png = Vec::new();
            let mut encoder = png::Encoder::new(&mut png, 2, 3);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer
                .write_image_data(&[255, 0, 0, 255].repeat(6))
                .unwrap();
            drop(writer);
            let count = usize::from(u16_at(table(AHEM, *b"maxp"), 4));
            let start = 4 + (count + 1) * 4;
            let end = start + 8 + png.len();
            let mut sbix = words(&[1, 1]);
            sbix.extend(1_u32.to_be_bytes());
            sbix.extend(12_u32.to_be_bytes());
            sbix.extend(words(&[10, 72])); // Real size10 strike.
            for glyph in 0..=count {
                sbix.extend(
                    u32::try_from(if glyph <= 274 { start } else { end })
                        .unwrap()
                        .to_be_bytes(),
                );
            }
            sbix.extend(words(&[1, 2])); // Pixel bearings, BottomLeft origin.
            sbix.extend(*b"png ");
            sbix.extend(png);
            replacements.push((*b"sbix", sbix));
        }
    }
    replace_tables(replacements)
}

fn premises(bytes: &[u8]) {
    for tag in [*b"glyf", *b"loca", *b"hmtx", *b"hhea"] {
        assert_eq!(table(bytes, tag), table(AHEM, tag));
    }
    let face = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    for (cp, glyph) in [(0x41_u32, 274), (0x42, 277), (0x5d0, 274), (0x5d1, 277)] {
        assert_eq!(face.charmap().map(cp), Some(GlyphId::new(glyph)));
    }
    assert!(face.vhea().is_err());
    assert!(face.vmtx().is_err());
    let metrics = face.glyph_metrics(Size::unscaled(), LocationRef::default());
    for (glyph, expected) in [
        (274, [0.0, 400.0, 1000.0, 600.0]),
        (277, [200.0, -200.0, 400.0, 800.0]),
    ] {
        assert_eq!(metrics.advance_width(GlyphId::new(glyph)), Some(1000.0));
        let b = metrics.bounds(GlyphId::new(glyph)).unwrap();
        assert_eq!([b.x_min, b.y_min, b.x_max, b.y_max], expected);
    }
    assert_eq!(metrics.left_side_bearing(GlyphId::new(274)), Some(0.0));
}

fn authored() -> text::Style {
    text::Style {
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        ..style()
    }
}

fn selected(
    bytes: &[u8],
    raw: &str,
    authored: text::Style,
    orientation: text::LineOrientation,
    direction: text::ParagraphBaseDirection,
) -> (text::PreparedText, text::FinalizedLine, text::System) {
    let mut system = system(bytes.to_vec());
    let analyzed = text::AnalyzedSource::new(
        text::Source::identified(raw, text::Id::from_u64(1579001), 7),
        authored,
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            text::PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    assert_eq!(prepared.orientation(), orientation);
    let end = prepared
        .opportunity_views()
        .find(|o| o.kind() == text::PreparedBreakKind::ParagraphEnd)
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
                orientation,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: text::Alignment::Left,
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
    (prepared, line, system)
}

fn place(
    prepared: text::PreparedText,
    line: text::FinalizedLine,
    orientation: text::LineOrientation,
) -> text::Layout {
    text::Layout::from_finalized_lines(request(prepared, line, orientation)).unwrap()
}
fn request(
    prepared: text::PreparedText,
    line: text::FinalizedLine,
    orientation: text::LineOrientation,
) -> text::PlacementRequest {
    text::PlacementRequest {
        prepared,
        coverage: text::PlacementCoverage::WholeParagraph,
        entries: vec![text::PlacementEntry::Formatted(text::LinePlacement {
            line,
            anchor: if orientation == text::LineOrientation::Horizontal {
                text::LineAnchor {
                    inline_origin: GX,
                    baseline: GY,
                }
            } else {
                text::LineAnchor {
                    inline_origin: GY,
                    baseline: GX,
                }
            },
            orientation,
            atomics: vec![],
        })],
    }
}
fn native(
    bytes: &[u8],
    raw: &str,
    authored: text::Style,
    orientation: text::LineOrientation,
    direction: text::ParagraphBaseDirection,
) -> (text::Layout, text::System) {
    let (prepared, line, system) = selected(bytes, raw, authored, orientation, direction);
    (place(prepared, line, orientation), system)
}
fn ink(run: &PreparedGlyphRun, transform: Transform) -> kurbo::Rect {
    let mut path = run.path().clone();
    path.apply_affine(kurbo::Affine::new(transform.as_array()));
    path.bounding_box()
}

#[test]
fn native_asymmetric_ccw_basis_uses_raw_pens_offsets_and_one_outer_transform_with_h_cw_controls() {
    let bytes = font(Representation::Outline, true);
    premises(&bytes);
    for (orientation, basis, matrix, physical) in [
        (
            text::LineOrientation::Horizontal,
            text::GlyphBasis::Unrotated,
            [1.0, 0.0, 0.0, 1.0, GX, GY],
            kurbo::Rect::new(-2.0, 32.5, 12.0, 42.5),
        ),
        (
            text::LineOrientation::SidewaysClockwise,
            text::GlyphBasis::Clockwise,
            [0.0, 1.0, -1.0, 0.0, GX, GY],
            kurbo::Rect::new(-5.25, 41.75, 4.75, 55.75),
        ),
        (
            CCW,
            text::GlyphBasis::Counterclockwise,
            [0.0, -1.0, 1.0, 0.0, GX, GY],
            kurbo::Rect::new(-11.25, 25.25, -1.25, 39.25),
        ),
    ] {
        let (input, system) = native(
            &bytes,
            "AB",
            authored(),
            orientation,
            text::ParagraphBaseDirection::LeftToRight,
        );
        assert_eq!(input.source().id(), Some(text::Id::from_u64(1579001)));
        assert_eq!(input.source().revision(), 7);
        assert_eq!(input.source().text().code_units(), [65, 66]);
        let run = input.run_views().next().unwrap();
        assert_eq!(run.range(), text::Range::new(0, 2));
        assert_eq!(run.origin(), text::Point::new(GX, GY));
        assert_eq!(run.orientation(), orientation);
        assert_eq!(run.glyph_basis(), basis);
        assert_eq!(
            run.glyphs()
                .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>(),
            [(274, 0.0, 1.25, 2.5, 11.25), (277, 11.25, 0.0, 0.0, 10.0)]
        );
        let stats = system.stats();
        for outer in [
            Transform::identity(),
            Transform::translation(7.0, 9.0).unwrap(),
            Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap(),
        ] {
            let scene = prepare_text_layout(&input, outer, limits()).unwrap();
            assert!(std::ptr::eq(scene.layout(), &input));
            assert_eq!(system.stats(), stats);
            assert_eq!(scene.scene().commands.len(), 1);
            let (paint, actual) = outline(&scene.scene().commands[0]);
            assert_eq!(
                actual,
                Transform::try_new(matrix).unwrap().then(outer).unwrap()
            );
            assert_eq!(paint.font().bytes(), bytes);
            assert_eq!(paint.font().index(), 0);
            assert_eq!(paint.instance().size(), 10.0);
            assert!(paint.instance().coordinates().is_empty());
            assert_eq!(paint.instance().synthesis(), GlyphSynthesis::none());
            assert_eq!(
                paint.glyphs(),
                [
                    TextGlyph::try_new(274, 1.25, 2.5, 11.25).unwrap(),
                    TextGlyph::try_new(277, 11.25, 0.0, 10.0).unwrap()
                ]
            );
            assert_eq!(
                paint.bounds().ink_rect(),
                Some(Rect::try_new(1.25, -8.0, 14.0, 10.0).unwrap())
            );
            assert_eq!(
                ink(paint, actual),
                kurbo::Affine::new(outer.as_array()).transform_rect_bbox(physical)
            );
        }
        if orientation == CCW {
            // Font over8/under2 and total advance21.25 are metrics, not ink.
            assert_eq!(input.bounds(), text::Rect::new(-11.25, 19.25, 10.0, 21.25));
        }
    }
}

#[test]
fn rtl_dflt_only_gpos_is_unpositioned_in_both_horizontal_and_native_ccw_profiles() {
    let bytes = font(Representation::Outline, true);
    premises(&bytes);
    assert_eq!(&table(&bytes, *b"GPOS")[12..16], b"DFLT");
    // Hebrew's deliberate gpos_tag guard requires a hebr ScriptRecord. A DFLT
    // fallback disables GPOS independently of orientation or kern's default.
    for (orientation, basis, matrix, physical) in [
        (
            text::LineOrientation::Horizontal,
            text::GlyphBasis::Unrotated,
            [1.0, 0.0, 0.0, 1.0, GX, GY],
            kurbo::Rect::new(-1.25, 32.5, 16.75, 42.5),
        ),
        (
            CCW,
            text::GlyphBasis::Counterclockwise,
            [0.0, -1.0, 1.0, 0.0, GX, GY],
            kurbo::Rect::new(-11.25, 20.5, -1.25, 38.5),
        ),
    ] {
        let (input, system) = native(
            &bytes,
            "\u{5d0}\u{5d1}",
            authored(),
            orientation,
            text::ParagraphBaseDirection::RightToLeft,
        );
        assert_eq!(input.source().text().code_units(), [0x5d0, 0x5d1]);
        assert_eq!(input.direction(), text::Direction::RightToLeft);
        let raw = input.run_views().next().unwrap();
        assert_eq!(raw.glyph_basis(), basis);
        assert_eq!(
            raw.glyphs()
                .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>(),
            [(277, 0.0, 0.0, 0.0, 10.0), (274, 10.0, 0.0, 0.0, 10.0)]
        );
        let stats = system.stats();
        let scene = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
        assert_eq!(system.stats(), stats);
        let (run, transform) = outline(&scene.scene().commands[0]);
        assert_eq!(transform.as_array(), matrix);
        assert_eq!(
            run.glyphs(),
            [
                TextGlyph::try_new(277, 0.0, 0.0, 10.0).unwrap(),
                TextGlyph::try_new(274, 10.0, 0.0, 10.0).unwrap()
            ]
        );
        // Original stripe277 x2..4 and chi274 x10..20 supply this hull.
        assert_eq!(
            run.bounds().ink_rect(),
            Some(Rect::try_new(2.0, -8.0, 18.0, 10.0).unwrap())
        );
        assert_eq!(ink(run, transform), physical);
    }
}

#[test]
fn actual_rtl_horizontal_provider_order_keeps_positive_raw_pens_in_upward_ccw_flow() {
    let original = font(Representation::Outline, true);
    let before = table(&original, *b"GPOS");
    assert_eq!(&before[12..16], b"DFLT");
    let mut gpos = before.to_vec();
    // Only the ScriptRecord tag changes. Its offset, LangSys, kern feature,
    // lookup and authored numeric fields stay byte-identical. No feature is
    // forced: hebr makes this actual Hebrew script eligible for ordinary GPOS.
    gpos[12..16].copy_from_slice(b"hebr");
    let bytes = replace_tables(vec![
        (*b"cmap", table(&original, *b"cmap").to_vec()),
        (*b"GPOS", gpos),
    ]);
    let after = table(&bytes, *b"GPOS");
    assert_eq!(&before[..12], &after[..12]);
    assert_eq!(&after[12..16], b"hebr");
    assert_eq!(&before[16..], &after[16..]);
    premises(&bytes);
    for (orientation, basis, matrix, physical) in [
        (
            text::LineOrientation::Horizontal,
            text::GlyphBasis::Unrotated,
            [1.0, 0.0, 0.0, 1.0, GX, GY],
            kurbo::Rect::new(-1.25, 32.5, 18.0, 42.5),
        ),
        (
            CCW,
            text::GlyphBasis::Counterclockwise,
            [0.0, -1.0, 1.0, 0.0, GX, GY],
            kurbo::Rect::new(-11.25, 19.25, -1.25, 38.5),
        ),
    ] {
        let (input, system) = native(
            &bytes,
            "\u{5d0}\u{5d1}",
            authored(),
            orientation,
            text::ParagraphBaseDirection::RightToLeft,
        );
        assert_eq!(input.source().text().code_units(), [0x5d0, 0x5d1]);
        assert_eq!(input.direction(), text::Direction::RightToLeft);
        let raw = input.run_views().next().unwrap();
        assert_eq!(raw.glyph_basis(), basis);
        assert_eq!(
            raw.glyphs()
                .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>(),
            [(277, 0.0, 0.0, 0.0, 10.0), (274, 10.0, 1.25, 2.5, 11.25)]
        );
        let stats = system.stats();
        let scene = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
        assert_eq!(system.stats(), stats);
        let (run, transform) = outline(&scene.scene().commands[0]);
        assert_eq!(transform.as_array(), matrix);
        assert_eq!(
            run.glyphs(),
            [
                TextGlyph::try_new(277, 0.0, 0.0, 10.0).unwrap(),
                TextGlyph::try_new(274, 11.25, 2.5, 11.25).unwrap()
            ]
        );
        // Same authored GPOS as LTR: p10+dx1.25 and yDown2.5. Retained
        // stripe277 and positioned chi274 give x2..21.25, y-8..2.
        assert_eq!(
            run.bounds().ink_rect(),
            Some(Rect::try_new(2.0, -8.0, 19.25, 10.0).unwrap())
        );
        assert_eq!(ink(run, transform), physical);
    }
}

#[test]
fn ccw_physical_decoration_gets_only_caller_placement_after_native_line_mapping() {
    let bytes = font(Representation::Outline, true);
    let authored = text::Style {
        underline: text::Decoration::solid()
            .with_offset(text::DecorationOffset::try_absolute(-2.0).unwrap())
            .with_thickness(text::DecorationThickness::try_absolute(1.0).unwrap()),
        ..authored()
    };
    let (input, _) = native(
        &bytes,
        "AB",
        authored,
        CCW,
        text::ParagraphBaseDirection::LeftToRight,
    );
    // Local under2..3 maps RIGHT of baseline, and u0..21.25 maps upward.
    assert_eq!(
        input.decoration_views().next().unwrap().rect(),
        text::Rect::new(-1.25, 19.25, 1.0, 21.25)
    );
    for outer in [
        Transform::identity(),
        Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap(),
    ] {
        let scene = prepare_text_layout(&input, outer, limits()).unwrap();
        let commands = &scene.scene().commands;
        assert_eq!(commands.len(), 2);
        let decorations = if outer == Transform::identity() {
            &commands[1..]
        } else {
            let Command::Layer { layer, children } = &commands[1] else {
                panic!("one physical decoration placement layer")
            };
            assert_eq!(layer.transform(), outer);
            children.as_slice()
        };
        let [Command::Fill { shape, .. }] = decorations else {
            panic!("one real decoration fill")
        };
        assert_eq!(
            *shape,
            crate::Shape::rect(Rect::try_new(-1.25, 19.25, 1.0, 21.25).unwrap())
        );
    }
}

#[test]
fn ccw_synthesis_shears_actual_font_axes_before_rotation_and_nonuniform_scale() {
    let bytes = font(Representation::Outline, true);
    let authored = text::Style {
        font: text::Font::new().family("Ahem").style(text::Slant::Italic),
        ..authored()
    };
    let (input, _) = native(
        &bytes,
        "A",
        authored,
        CCW,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let raw = input.run_views().next().unwrap();
    let instance = raw.effective_instance().unwrap();
    assert!(instance.x_shear() > 0.0);
    assert_eq!(instance.embolden(), 0.0);
    let scene = prepare_text_layout(
        &input,
        Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap(),
        limits(),
    )
    .unwrap();
    let (run, transform) = outline(&scene.scene().commands[0]);
    assert_eq!(
        run.instance().synthesis(),
        GlyphSynthesis::try_new(instance.x_shear(), 0.0).unwrap()
    );
    assert_eq!(transform.as_array(), [0.0, -3.0, 2.0, 0.0, 0.5, 130.5]);
    let s = f64::from(instance.x_shear());
    let left = f64::from((1.25 + 4.0 * s) as f32);
    let right = f64::from((11.25 + 6.0 * s) as f32);
    assert_eq!(
        run.bounds().ink_rect(),
        Some(Rect::try_new(left, -3.5, right - left, 2.0).unwrap())
    );
    assert_eq!(
        ink(run, transform),
        kurbo::Rect::new(-6.5, 130.5 - 3.0 * right, -2.5, 130.5 - 3.0 * left)
    );
}

#[test]
fn ccw_admits_actual_colr_layers_and_sbix_pixels_with_their_own_ink_and_instance() {
    for representation in [Representation::Color, Representation::Bitmap] {
        let bytes = font(representation, false);
        premises(&bytes);
        let (input, system) = native(
            &bytes,
            "A",
            authored(),
            CCW,
            text::ParagraphBaseDirection::LeftToRight,
        );
        let stats = system.stats();
        let scene = prepare_text_layout(
            &input,
            Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap(),
            limits(),
        )
        .unwrap();
        assert_eq!(system.stats(), stats);
        let (run, transform) = outline(&scene.scene().commands[0]);
        assert_eq!(run.font().bytes(), bytes);
        assert_eq!(run.instance().synthesis(), GlyphSynthesis::none());
        assert_eq!(run.instance().size(), 10.0);
        assert!(run.instance().coordinates().is_empty());
        assert_eq!(
            run.glyphs(),
            [TextGlyph::try_new(274, 0.0, 0.0, 10.0).unwrap()]
        );
        assert_eq!(transform.as_array(), [0.0, -3.0, 2.0, 0.0, 0.5, 130.5]);
        match representation {
            Representation::Color => {
                assert_eq!(
                    run.bounds().ink_rect(),
                    Some(Rect::try_new(2.0, -8.0, 2.0, 10.0).unwrap())
                );
                let [crate::text::GlyphPaint::Fill { color, .. }] = run.paints() else {
                    panic!("one actual COLR layer")
                };
                assert_eq!(*color, Some(Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()));
                assert_eq!(
                    ink(run, transform),
                    kurbo::Rect::new(-15.5, 118.5, 4.5, 124.5)
                );
            }
            Representation::Bitmap => {
                // Real chi LSB0/yMin400, strike10, bearing(1,2), PNG2x3:
                // local x1..3, y(-4-2-3)..(-4-2) = -9..-6.
                assert_eq!(
                    run.bounds().ink_rect(),
                    Some(Rect::try_new(1.0, -9.0, 2.0, 3.0).unwrap())
                );
                let [
                    crate::text::GlyphPaint::Bitmap {
                        image,
                        rect,
                        brush_transform,
                    },
                ] = run.paints()
                else {
                    panic!("one retained decoded bitmap")
                };
                assert_eq!(image.size(), crate::Size::try_new(2.0, 3.0).unwrap());
                assert_eq!(*rect, Rect::try_new(1.0, -9.0, 2.0, 3.0).unwrap());
                assert_eq!(brush_transform.as_coeffs(), [1.0, 0.0, 0.0, 1.0, 1.0, -9.0]);
                assert_eq!(
                    kurbo::Affine::new(transform.as_array())
                        .transform_rect_bbox(kurbo::Rect::new(1.0, -9.0, 3.0, -6.0)),
                    kurbo::Rect::new(-17.5, 121.5, -11.5, 127.5)
                );
                assert_eq!(run.path_elements().count(), 0);
            }
            Representation::Outline => unreachable!(),
        }
    }
}

#[test]
fn ccw_numeric_and_resource_refusals_publish_no_prefix_and_keep_the_accepted_scene() {
    let bytes = font(Representation::Outline, true);
    let (input, system) = native(
        &bytes,
        "AB",
        authored(),
        CCW,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let stats = system.stats();
    let accepted = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    let retained = accepted.scene().clone();
    let error = refuse(
        &input,
        Transform::identity(),
        bounded(1, 1024, ROOM, 32, 32, 32),
    );
    assert_limit(&error, TextLayoutResource::Glyphs, 2, 1);
    let error = refuse(
        &input,
        Transform::translation(1e30, 1e30).unwrap(),
        limits(),
    );
    let cause = item_render(&error, TextLayoutLocation::Run { line: 0, run: 0 });
    assert!(matches!(
        cause.glyph_preparation_diagnostic(),
        Some(GlyphPreparationError::Numeric {
            phase: GlyphNumericPhase::Placement,
            ..
        })
    ));
    let (before, transform) = outline(&retained.commands[0]);
    let (after, actual) = outline(&accepted.scene().commands[0]);
    assert!(before.is_same_preparation(after));
    assert_eq!(transform, actual);
    assert_eq!(
        ink(after, actual),
        kurbo::Rect::new(-11.25, 25.25, -1.25, 39.25)
    );
    assert_eq!(system.stats(), stats);
    drop(system);
    assert!(prepare_text_layout(&input, Transform::identity(), limits()).is_ok());
    drop(accepted);
    drop(input);
    assert_eq!(
        ink(before, transform),
        kurbo::Rect::new(-11.25, 25.25, -1.25, 39.25)
    );
}

#[test]
fn ccw_selected_images_refuse_synthesis_precisely_and_keep_prior_publication() {
    for representation in [Representation::Color, Representation::Bitmap] {
        let bytes = font(representation, false);
        let (ordinary, _) = native(
            &bytes,
            "BA",
            authored(),
            CCW,
            text::ParagraphBaseDirection::LeftToRight,
        );
        let accepted = prepare_text_layout(&ordinary, Transform::identity(), limits()).unwrap();
        let previous = accepted.scene().clone();
        let italic = text::Style {
            font: text::Font::new().family("Ahem").style(text::Slant::Italic),
            ..authored()
        };
        let (input, system) = native(
            &bytes,
            "BA",
            italic,
            CCW,
            text::ParagraphBaseDirection::LeftToRight,
        );
        assert!(
            input
                .run_views()
                .next()
                .unwrap()
                .effective_instance()
                .unwrap()
                .x_shear()
                > 0.0
        );
        let stats = system.stats();
        // The actual ordinary B outline precedes selected color/bitmap A in
        // this run. A late unsupported instance still returns only an error.
        let error = refuse(&input, Transform::identity(), limits());
        let cause = item_render(&error, TextLayoutLocation::Run { line: 0, run: 0 });
        assert_eq!(
            cause.glyph_preparation_diagnostic(),
            Some(&GlyphPreparationError::UnsupportedImageInstance)
        );
        assert_eq!(system.stats(), stats);
        let (before, transform) = outline(&previous.commands[0]);
        let (after, actual) = outline(&accepted.scene().commands[0]);
        assert!(before.is_same_preparation(after));
        assert_eq!(actual, transform);
        assert_eq!(after.glyphs().len(), 2);
        assert!(prepare_text_layout(&ordinary, Transform::identity(), limits()).is_ok());
    }
}

#[test]
fn native_ccw_preserves_precise_nonidentity_composite_refusal_after_an_ordinary_glyph() {
    let bytes = composite_x_ahem(Some(8192));
    let (input, system) = native(
        &bytes,
        "AX",
        authored(),
        CCW,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let raw = input.run_views().next().unwrap();
    assert_eq!(raw.glyph_basis(), text::GlyphBasis::Counterclockwise);
    assert_eq!(raw.glyphs().map(|g| g.id()).collect::<Vec<_>>(), [35, 58]);
    let stats = system.stats();
    let error = refuse(&input, Transform::identity(), limits());
    let cause = item_render(&error, TextLayoutLocation::Run { line: 0, run: 0 });
    assert_eq!(
        cause.glyph_preparation_diagnostic(),
        Some(&GlyphPreparationError::UnsupportedOutline {
            capability: GlyphCapability::CompositeGlyph,
        })
    );
    assert_eq!(system.stats(), stats);
    assert_eq!(input.source().text().code_units(), [65, 88]);
}

#[test]
fn genuine_ccw_selected_line_refuses_cw_placement_without_losing_its_source_or_replay() {
    let bytes = font(Representation::Outline, true);
    let (prepared, line, system) = selected(
        &bytes,
        "AB",
        authored(),
        CCW,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let before = system.stats();
    let error = text::Layout::from_finalized_lines(request(
        prepared.clone(),
        line.clone(),
        text::LineOrientation::SidewaysClockwise,
    ))
    .expect_err("actual orientation mismatch");
    assert_eq!(
        error.detail(),
        Some(&text::ErrorDetail::InvalidTextProtocol {
            reason: text::ProtocolError::MismatchedContext {
                field: text::ProtocolContextField::Orientation
            },
        })
    );
    let input = place(prepared, line, CCW);
    assert_eq!(input.source().text().code_units(), [65, 66]);
    assert_eq!(input.source().id(), Some(text::Id::from_u64(1579001)));
    let scene = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    assert_eq!(system.stats(), before);
    let (run, transform) = outline(&scene.scene().commands[0]);
    assert_eq!(
        ink(run, transform),
        kurbo::Rect::new(-11.25, 25.25, -1.25, 39.25)
    );
}

#[test]
fn native_ccw_retains_actual_variable_instance_coordinates_and_qualified_ink() {
    let bytes = variable_ahem();
    for (setting, bits, width) in [(0.0, 0, 1000.0), (0.5, 8192, 1050.0), (1.0, 16384, 1100.0)] {
        // Existing owned fvar axis0..1 and dense gvar tuple add100 to the X
        // outline's right edge at peak1; phantom advances have zero deltas.
        let authored = text::Style {
            size: 1000.0,
            line_height: text::LineHeight::Absolute(1000.0),
            font: text::Font::new()
                .family("Ahem")
                .variation(format!("'wght' {setting}")),
            ..style()
        };
        let (input, system) = native(
            &bytes,
            "X",
            authored,
            CCW,
            text::ParagraphBaseDirection::LeftToRight,
        );
        let stats = system.stats();
        let scene = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
        assert_eq!(system.stats(), stats);
        let (run, transform) = outline(&scene.scene().commands[0]);
        assert_eq!(run.font().bytes(), bytes);
        assert_eq!(run.instance().coordinates().len(), 1);
        assert_eq!(run.instance().coordinates()[0].bits(), bits);
        assert_eq!(
            run.bounds().ink_rect(),
            Some(Rect::try_new(0.0, -800.0, width, 1000.0).unwrap())
        );
        assert_eq!(
            ink(run, transform),
            kurbo::Rect::new(-803.25, GY - width, 196.75, GY)
        );
        let error = refuse(
            &input,
            Transform::identity(),
            bounded(1, 1024, ROOM, 1, 0, 0),
        );
        assert_limit(&error, TextLayoutResource::CoordinateValues, 1, 0);
    }
}
