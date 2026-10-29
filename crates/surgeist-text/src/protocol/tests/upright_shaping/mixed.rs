//! Functional Mixed qualification from actual Unicode, native packets and BASE.
#[path = "mixed_extended.rs"]
mod extended;
#[path = "mixed_flow.rs"]
mod flow;
#[path = "mixed_selections.rs"]
mod selections;
#[path = "mixed_spacing.rs"]
mod spacing;

use super::*;
use crate::{
    FontBaseline, FontBaselineAxis, FontBaselineOutcome, FontBaselineProvenance,
    FontBaselineScript, GlyphBasis,
};
use icu_properties::{CodePointMapData, props::VerticalOrientation};

fn premises(bytes: &[u8], horizontal: bool) {
    let face = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(face.maxp().unwrap().num_glyphs(), 281);
    assert_eq!(face.hhea().unwrap().number_of_h_metrics(), 281);
    let hhea = face.hhea().unwrap();
    assert_eq!(
        (
            hhea.ascender().to_i16(),
            hhea.descender().to_i16(),
            hhea.line_gap().to_i16()
        ),
        (800, -200, 0)
    );
    assert_eq!(face.os2().unwrap().fs_selection().bits() & 0x80, 0);
    for (cp, id) in mixed_fonts::mappings() {
        assert_eq!(face.charmap().map(cp), Some(GlyphId::new(id)));
    }
    assert_eq!(face.vhea().unwrap().number_of_long_ver_metrics(), 281);
    let metrics = face.glyph_metrics(Size::unscaled(), LocationRef::default());
    for (id, bounds) in [
        (58, [0.0, -200.0, 1000.0, 800.0]),
        (82, [0.0, -200.0, 1000.0, 0.0]),
        (274, [0.0, 400.0, 1000.0, 600.0]),
        (277, [200.0, -200.0, 400.0, 800.0]),
    ] {
        assert_eq!(
            face.hmtx().unwrap().h_metrics()[id as usize].advance.get(),
            1000
        );
        let vertical = &face.vmtx().unwrap().v_metrics()[id as usize];
        assert_eq!(
            (vertical.advance.get(), vertical.side_bearing.get()),
            (1200, 100)
        );
        let b = metrics.bounds(GlyphId::new(id)).unwrap();
        assert_eq!([b.x_min, b.y_min, b.x_max, b.y_max], bounds);
    }
    let base = face.base().unwrap();
    assert_eq!(base.horiz_axis().is_some(), horizontal);
    for (axis, expected) in [
        (base.horiz_axis(), [100, 700]),
        (base.vert_axis(), [0, 1200]),
    ] {
        let Some(axis) = axis else { continue };
        let axis = axis.unwrap();
        assert_eq!(
            axis.base_tag_list()
                .unwrap()
                .unwrap()
                .baseline_tags()
                .iter()
                .map(|tag| <[u8; 4]>::try_from(tag.be_bytes()).unwrap())
                .collect::<Vec<_>>(),
            [*b"ideo", *b"idtp"]
        );
        let scripts = axis.base_script_list().unwrap();
        assert_eq!(scripts.base_script_records().len(), 4);
        for (row, tag) in scripts
            .base_script_records()
            .iter()
            .zip(mixed_fonts::SCRIPTS)
        {
            assert_eq!(row.base_script_tag().to_be_bytes(), *tag);
            let values = row
                .base_script(scripts.offset_data())
                .unwrap()
                .base_values()
                .unwrap()
                .unwrap();
            assert_eq!(
                values
                    .base_coords()
                    .iter()
                    .map(|v| v.unwrap().coordinate())
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }
    let vo = CodePointMapData::<VerticalOrientation>::new();
    assert_eq!(vo.get('X'), VerticalOrientation::Rotated);
    assert_eq!(vo.get('一'), VerticalOrientation::Upright);
    assert_eq!(vo.get('\u{2329}'), VerticalOrientation::TransformedRotated);
    assert_eq!(vo.get('、'), VerticalOrientation::TransformedUpright);
}

fn native(
    bytes: &[u8],
    text: &str,
    upright: bool,
    script: surgeist_shaper::Script,
    authored: &[&str],
) -> Vec<(u32, u32, f64, f64, f64)> {
    let face = surgeist_shaper::FontRef::new(bytes).unwrap();
    let data = surgeist_shaper::ShaperData::new(&face);
    let shaper = data.shaper(&face).build();
    let direction = if upright {
        surgeist_shaper::Direction::TopToBottom
    } else {
        surgeist_shaper::Direction::LeftToRight
    };
    let mut buffer = surgeist_shaper::UnicodeBuffer::new();
    for (ordinal, scalar) in text.chars().enumerate() {
        buffer.add(scalar, u32::try_from(ordinal).unwrap());
    }
    buffer.set_direction(direction);
    buffer.set_script(script);
    let mut features = vec![surgeist_shaper::Feature::new(
        surgeist_shaper::Tag::new(if upright { b"vert" } else { b"vrtr" }),
        1,
        ..,
    )];
    features.extend(
        authored
            .iter()
            .map(|f| f.parse::<surgeist_shaper::Feature>().unwrap()),
    );
    if upright {
        let plan =
            surgeist_shaper::ShapePlan::new(&shaper, direction, Some(script), None, &features);
        let raw = skrifa::FontRef::new(bytes).unwrap();
        let metrics = raw.glyph_metrics(Size::unscaled(), LocationRef::default());
        let output = shaper
            .shape_vertical_with_metrics(
                &plan,
                buffer,
                &features,
                surgeist_shaper::VerticalOriginReference::try_font_design_x(600.0).unwrap(),
                |id| {
                    let glyph = GlyphId::new(id);
                    let row = &raw.vmtx().unwrap().v_metrics()[id as usize];
                    let top = metrics.bounds(glyph).unwrap().y_max as i32
                        + i32::from(row.side_bearing.get());
                    let width =
                        i32::from(raw.hmtx().unwrap().h_metrics()[id as usize].advance.get());
                    Ok::<_, std::convert::Infallible>(surgeist_shaper::VerticalNominalMetrics::new(
                        u32::from(row.advance.get()),
                        width,
                        top,
                    ))
                },
            )
            .unwrap();
        assert_eq!(
            output.shaping_status(),
            surgeist_shaper::ShapingStatus::Complete
        );
        output
            .glyph_infos()
            .iter()
            .zip(output.glyph_positions())
            .map(|(info, pos)| {
                (
                    info.glyph_id,
                    info.cluster,
                    -pos.y_advance / 100.0,
                    pos.x_offset / 100.0,
                    -pos.y_offset / 100.0,
                )
            })
            .collect()
    } else {
        let output = shaper.shape(buffer, &features);
        assert_eq!(
            output.shaping_status(),
            surgeist_shaper::ShapingStatus::Complete
        );
        output
            .glyph_infos()
            .iter()
            .zip(output.glyph_positions())
            .map(|(info, pos)| {
                (
                    info.glyph_id,
                    info.cluster,
                    f64::from(pos.x_advance) / 100.0,
                    f64::from(pos.x_offset) / 100.0,
                    -f64::from(pos.y_offset) / 100.0,
                )
            })
            .collect()
    }
}

fn packets(
    line: &FinalizedLine,
    bytes: &[u8],
) -> Vec<(Range, GlyphBasis, u32, f64, f64, f64, f64)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.font_size(), 10.0);
            assert_eq!(
                run.face().catalogue_identity(),
                line.prepared_text().font_catalogue_identity()
            );
            run.glyphs()
                .map(|g| {
                    (
                        g.range(),
                        run.glyph_basis(),
                        g.id(),
                        run.origin().x + g.pen(),
                        f64::from(g.x()),
                        f64::from(g.y()),
                        f64::from(g.advance()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn copy_source(layout: &crate::Layout, source: &Source) {
    assert_eq!(layout.source(), source);
    // Placement iteration is visual. Locate the actual logical source cuts
    // rather than treating visual extremes as the original text's endpoints.
    let start = layout
        .positions()
        .find(|position| position.source_cursor() == crate::Cursor::new(0, crate::Affinity::After))
        .unwrap();
    let end = layout
        .positions()
        .find(|position| {
            position.source_cursor()
                == crate::Cursor::new(source.text().len(), crate::Affinity::Before)
        })
        .unwrap();
    let selection = layout.position_selection(&start, &end).unwrap();
    assert_eq!(selection.source_text(), source.text());
    assert_eq!(selection.plaintext(), source.text());
}

#[test]
fn mixed_latin_and_han_use_actual_metrics_and_distinct_central_references() {
    for (horizontal, c, height, descent) in [(true, 4.0, 13.0, 7.0), (false, 3.0, 12.0, 6.0)] {
        let bytes = mixed_fonts::fixture(horizontal);
        premises(&bytes, horizontal);
        assert_eq!(
            native(&bytes, "X", false, surgeist_shaper::script::LATIN, &[]),
            [(274, 0, 10.0, 0.0, 0.0)]
        );
        assert_eq!(
            native(&bytes, "一", true, surgeist_shaper::script::HAN, &[]),
            [(82, 0, 12.0, -6.0, 1.0)]
        );
        let mut system = system_with(&bytes);
        let face = system.registered_font_faces().next().unwrap().to_owned();
        let instance = face.unsynthesized_instance(10.0, &[]).unwrap();
        for (axis, expected, provenance) in [
            (
                FontBaselineAxis::HorizontalY,
                c,
                if horizontal {
                    FontBaselineProvenance::BaseEdgesCssMidpoint
                } else {
                    FontBaselineProvenance::InstanceMetricsCssMidpoint
                },
            ),
            (
                FontBaselineAxis::VerticalX,
                6.0,
                FontBaselineProvenance::BaseEdgesCssMidpoint,
            ),
        ] {
            let measurement = instance
                .baseline(
                    system.font_catalogue_identity(),
                    FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
                    axis,
                    FontBaseline::Central,
                )
                .unwrap();
            let FontBaselineOutcome::Available(value) = measurement.outcome() else {
                panic!("qualified Central")
            };
            assert_eq!(
                (value.coordinate(), value.provenance()),
                (expected, provenance)
            );
        }
        let (prepared, line) = composed_line(
            &mut system,
            "X一",
            &[],
            AnalysisTypography::Mixed,
            LineOrientation::Vertical,
            ParagraphBaseDirection::LeftToRight,
        );
        assert_eq!(
            prepared.analyzed_source().typography(),
            AnalysisTypography::Mixed
        );
        assert_eq!(
            prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 2)]
        );
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().ascent,
                line.feedback().descent,
                line.feedback().line_height,
                line.feedback().baseline
            ),
            (22.0, 6.0, descent, height, 6.0)
        );
        assert_eq!(
            packets(&line, &bytes),
            [
                (
                    Range::new(0, 1),
                    GlyphBasis::Clockwise,
                    274,
                    0.0,
                    0.0,
                    0.0,
                    10.0
                ),
                (
                    Range::new(1, 2),
                    GlyphBasis::Unrotated,
                    82,
                    10.0,
                    -6.0,
                    1.0,
                    12.0
                ),
            ]
        );
        let source = prepared.analyzed_source().source().clone();
        let layout = place_upright(prepared, line);
        assert_eq!(
            layout.bounds(),
            Rect::new(100.0 - descent, 20.0, height, 22.0)
        );
        let borrowed = layout.run_views().collect::<Vec<_>>();
        let owned = layout.glyph_runs();
        assert_eq!(borrowed.len(), owned.len());
        let mut physical = Vec::new();
        for (run, owned) in borrowed.iter().zip(&owned) {
            assert_eq!(run.orientation(), LineOrientation::Vertical);
            assert_eq!(owned.orientation(), LineOrientation::Vertical);
            assert_eq!(run.glyph_basis(), owned.glyph_basis());
            assert_eq!(run.origin(), owned.origin());
            assert_eq!(run.face().bytes(), owned.font().bytes());
            let origin = run.origin();
            for (g, owned_g) in run.glyphs().zip(owned.glyphs()) {
                assert_eq!(
                    (g.id(), g.pen(), g.x(), g.y(), g.advance()),
                    (
                        owned_g.id(),
                        owned_g.pen(),
                        owned_g.x(),
                        owned_g.y(),
                        owned_g.advance()
                    )
                );
                physical.push(match run.glyph_basis() {
                    GlyphBasis::Clockwise => Point::new(
                        origin.x - f64::from(g.y()),
                        origin.y + g.pen() + f64::from(g.x()),
                    ),
                    GlyphBasis::Counterclockwise => Point::new(
                        origin.x + f64::from(g.y()),
                        origin.y + g.pen() - f64::from(g.x()),
                    ),
                    GlyphBasis::Unrotated => Point::new(
                        origin.x + f64::from(g.x()),
                        origin.y + g.pen() + f64::from(g.y()),
                    ),
                });
            }
        }
        assert_eq!(
            physical,
            [Point::new(100.0 - c, 20.0), Point::new(94.0, 31.0)]
        );
        copy_source(&layout, &source);
    }
}

#[test]
fn mixed_transformed_punctuation_and_explicit_features_do_not_choose_axes() {
    let bytes = mixed_fonts::fixture(true);
    premises(&bytes, true);
    for (authored, ids) in [
        (&[][..], [274, 82, 82]),
        (&[r#""vrt2" 1"#, r#""vrtr" 0"#][..], [277, 277, 277]),
        (&[r#""vrtr" 1"#, r#""vrtr" 0"#][..], [58, 82, 82]),
        (&[r#""vrtr" 0"#, r#""vrtr" 1"#][..], [274, 274, 274]),
        (&[r#""vert" 1"#, r#""vert" 0"#][..], [274, 58, 58]),
        (&[r#""vert" 0"#, r#""vert" 1"#][..], [274, 82, 82]),
    ] {
        assert_eq!(
            native(&bytes, "X", false, surgeist_shaper::script::LATIN, authored)[0].0,
            ids[0]
        );
        for scalar in ["\u{2329}", "、"] {
            assert_eq!(
                native(
                    &bytes,
                    scalar,
                    true,
                    surgeist_shaper::script::LATIN,
                    authored
                )[0]
                .0,
                ids[1]
            );
        }
        let (_, line) = composed_line(
            &mut system_with(&bytes),
            "X\u{2329}、",
            authored,
            AnalysisTypography::Mixed,
            LineOrientation::Vertical,
            ParagraphBaseDirection::LeftToRight,
        );
        let actual = packets(&line, &bytes);
        assert_eq!(
            actual.iter().map(|p| (p.1, p.2, p.6)).collect::<Vec<_>>(),
            [
                (GlyphBasis::Clockwise, ids[0], 10.0),
                (GlyphBasis::Unrotated, ids[1], 12.0),
                (GlyphBasis::Unrotated, ids[2], 12.0),
            ]
        );
        assert_eq!(line.feedback().advance, 34.0);
    }
    let mut mappings = mixed_fonts::mappings();
    for (cp, glyph) in &mut mappings {
        if [0x2329, 0x3008].contains(cp) {
            *glyph = 35;
        }
    }
    let absent = mixed_fonts::with_cmap(&bytes, &mappings);
    assert_eq!(
        native(
            &absent,
            "\u{2329}",
            true,
            surgeist_shaper::script::LATIN,
            &[]
        ),
        [(35, 0, 12.0, -6.0, 9.0)]
    );
    let (_, line) = composed_line(
        &mut system_with(&absent),
        "X\u{2329}",
        &[],
        AnalysisTypography::Mixed,
        LineOrientation::Vertical,
        ParagraphBaseDirection::LeftToRight,
    );
    assert_eq!(
        packets(&line, &absent)
            .iter()
            .map(|p| (p.1, p.2, p.6))
            .collect::<Vec<_>>(),
        [
            (GlyphBasis::Clockwise, 274, 10.0),
            (GlyphBasis::Unrotated, 35, 12.0)
        ]
    );
}
