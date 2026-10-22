use super::*;
use crate::{
    FontCatalogueMode, FontData, System, SystemOptions,
    test_fonts::{
        remove_tables, replace_tables, table, u16_at, variable_ahem, variable_ahem_with_tuple,
    },
};
const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");
const X: u32 = 58;
fn registered(bytes: Vec<u8>) -> (System, FontData) {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    let data = system.registered_font_faces().next().unwrap().to_owned();
    (system, data)
}
fn geometry(data: &FontData, glyph: u32, size: f32) -> Result<GlyphOutlineGeometry<'_>> {
    data.unsynthesized_instance(size, &[])?
        .glyph_outline(glyph, data.catalogue_identity())
}
fn reason(error: &Error) -> &FontOutlineError {
    let ErrorDetail::FontOutline { glyph, reason } = error.detail().unwrap() else {
        panic!("{error:?}")
    };
    assert_eq!(*glyph, X);
    reason
}
fn header(contours: i16, bounds: [i16; 4]) -> Vec<u8> {
    [contours, bounds[0], bounds[1], bounds[2], bounds[3]]
        .into_iter()
        .flat_map(i16::to_be_bytes)
        .collect()
}
// Actual licensed Ahem derivative: preserve every unrelated glyph/table, replacing
// only named glyph source and checked loca extents, as Render's owning fixtures do.
fn replace_glyphs(replacements: &[(u16, Vec<u8>)]) -> Vec<u8> {
    let loca = table(AHEM, *b"loca");
    let glyf = table(AHEM, *b"glyf");
    let count = usize::from(u16_at(table(AHEM, *b"maxp"), 4));
    let mut new_loca = Vec::new();
    let mut new_glyf = Vec::new();
    for id in 0..count {
        new_loca.extend(u16::try_from(new_glyf.len() / 2).unwrap().to_be_bytes());
        if let Some((_, data)) = replacements
            .iter()
            .find(|(glyph, _)| usize::from(*glyph) == id)
        {
            new_glyf.extend_from_slice(data);
        } else {
            let start = usize::from(u16_at(loca, id * 2)) * 2;
            let end = usize::from(u16_at(loca, (id + 1) * 2)) * 2;
            new_glyf.extend_from_slice(&glyf[start..end]);
        }
        if !new_glyf.len().is_multiple_of(2) {
            new_glyf.push(0);
        }
    }
    new_loca.extend(u16::try_from(new_glyf.len() / 2).unwrap().to_be_bytes());
    replace_tables(AHEM, vec![(*b"loca", new_loca), (*b"glyf", new_glyf)])
}
fn composite(child: u16, x: i16, y: i16) -> Vec<u8> {
    let mut data = header(-2, [0, -200, 1000, 800]);
    data.extend(3u16.to_be_bytes());
    data.extend(child.to_be_bytes());
    data.extend(x.to_be_bytes());
    data.extend(y.to_be_bytes());
    data
}

#[test]
fn actual_outline_retains_resource_and_catalogue_and_proves_empty_separately() {
    let (mut system, data) = registered(AHEM.to_vec());
    let basis = system.font_catalogue_identity().clone();
    let stats = system.stats();
    let retained = data
        .unsynthesized_instance(360.0, &[])
        .unwrap()
        .glyph_outline(X, &basis)
        .unwrap();
    assert_eq!(
        retained.bounds(),
        Some(Rect::new(0.0, -288.0, 360.0, 360.0))
    );
    assert_eq!(retained.glyph_id(), X);
    assert_eq!(retained.instance().font_size(), 360.0);
    assert!(retained.instance().face().is_same_face(&data.face()));
    assert_eq!(geometry(&data, 3, 360.0).unwrap().bounds(), None); // actual space loca
    assert_eq!(system.stats(), stats);
    system.register_fonts(AHEM.to_vec()).unwrap();
    assert_eq!(
        data.unsynthesized_instance(360.0, &[])
            .unwrap()
            .glyph_outline(X, system.font_catalogue_identity())
            .unwrap_err()
            .code,
        ErrorCode::FontCatalogueMismatch
    );
    drop(system);
    assert_eq!(
        retained.bounds(),
        Some(Rect::new(0.0, -288.0, 360.0, 360.0))
    );
    assert_eq!(retained.instance().face().catalogue_identity(), &basis);
    assert_eq!(
        geometry(&data, X, 360.0).unwrap().bounds(),
        retained.bounds()
    );
}

#[test]
fn actual_size_and_concrete_synthesis_change_outline_not_line_metrics() {
    let (_, data) = registered(AHEM.to_vec());
    let synthesis = super::super::FontSynthesis::resolve(360.0, Some(45.0), true).unwrap();
    assert_eq!(synthesis.x_shear(), 1.0);
    assert_eq!(synthesis.embolden(), 10.0);
    let instance =
        EffectiveFontInstance::qualify_synthesized(data.face(), 360.0, &[], synthesis).unwrap();
    let result = instance
        .glyph_outline(X, data.catalogue_identity())
        .unwrap();
    assert_eq!(
        result.bounds(),
        Some(Rect::new(-72.0, -288.0, 730.0, 360.0))
    );
    assert_eq!(result.instance().synthesis(), synthesis);
    assert_eq!(
        instance
            .metrics(data.catalogue_identity())
            .unwrap()
            .ascent(),
        288.0
    );
    assert_eq!(
        instance
            .glyph_outline(3, data.catalogue_identity())
            .unwrap()
            .bounds(),
        None
    );
}

#[test]
fn canonical_dense_variation_uses_actual_coordinates_including_default() {
    let (_, data) = registered(variable_ahem());
    for (bits, width) in [(0, 1000.0), (8192, 1050.0), (16384, 1100.0)] {
        let coords = [bits];
        let instance = data.unsynthesized_instance(1000.0, &coords).unwrap();
        let result = instance
            .glyph_outline(X, data.catalogue_identity())
            .unwrap();
        assert_eq!(result.bounds(), Some(Rect::new(0.0, -800.0, width, 1000.0)));
        assert_eq!(
            result.instance().normalized_coords().collect::<Vec<_>>(),
            coords
        );
    }
    assert_eq!(
        geometry(&data, X, 1000.0).unwrap().bounds(),
        Some(Rect::new(0.0, -800.0, 1000.0, 1000.0))
    );
}

#[test]
fn malformed_simple_source_and_unfaithful_valid_encoding_are_distinct() {
    let mut overrepeat = header(1, [0, 0, 0, 0]);
    overrepeat.extend(3u16.to_be_bytes());
    overrepeat.extend(0u16.to_be_bytes());
    overrepeat.extend([0x39, 255]);
    let mut missing_program = header(0, [0, 0, 0, 0]);
    missing_program.extend(1u16.to_be_bytes());
    for raw in [overrepeat, missing_program] {
        let (_, data) = registered(replace_glyphs(&[(X as u16, raw)]));
        assert_eq!(
            reason(&geometry(&data, X, 1000.0).unwrap_err()),
            &FontOutlineError::InvalidFont
        );
    }
    let mut valid = header(1, [0, 0, 2000, 4000]);
    valid.extend(3u16.to_be_bytes());
    valid.extend(0u16.to_be_bytes());
    valid.extend([0x39, 0, 0x39, 0, 0x01, 0x01]);
    for value in [1000i16, 1000, 3000, 1000] {
        valid.extend(value.to_be_bytes());
    }
    let (_, data) = registered(replace_glyphs(&[(X as u16, valid)]));
    assert_eq!(
        reason(&geometry(&data, X, 1000.0).unwrap_err()),
        &FontOutlineError::ProviderRefused { glyph: X }
    );
}

#[test]
fn static_composite_negative_header_and_cycle_attachment_are_real_source() {
    let (_, data) = registered(replace_glyphs(&[(X as u16, composite(35, 0, 0))]));
    // Ahem A is the same rectangle; negative -2 is a real composite header.
    assert_eq!(
        geometry(&data, X, 360.0).unwrap().bounds(),
        Some(Rect::new(0.0, -288.0, 360.0, 360.0))
    );
    let mut cycle = composite(X as u16, 0, 0);
    let (_, data) = registered(replace_glyphs(&[(X as u16, cycle.clone())]));
    assert_eq!(
        reason(&geometry(&data, X, 360.0).unwrap_err()),
        &FontOutlineError::InvalidFont
    );
    cycle[10..12].copy_from_slice(&1u16.to_be_bytes()); // actual point attachment
    let (_, data) = registered(replace_glyphs(&[(X as u16, cycle)]));
    assert_eq!(
        reason(&geometry(&data, X, 360.0).unwrap_err()),
        &FontOutlineError::Unsupported {
            capability: FontOutlineCapability::CompositeGlyph
        }
    );
}

#[test]
fn absent_representation_and_out_of_range_do_not_become_empty() {
    let (_, data) = registered(AHEM.to_vec());
    let count = u32::from(u16_at(table(AHEM, *b"maxp"), 4));
    let error = geometry(&data, count, 360.0).unwrap_err();
    assert!(
        matches!(error.detail(),Some(ErrorDetail::FontOutline{glyph,reason:FontOutlineError::GlyphRange{count:n}}) if *glyph==count && *n==count)
    );
    let (_, data) = registered(remove_tables(AHEM, &[*b"glyf", *b"loca"]));
    assert_eq!(
        reason(&geometry(&data, X, 360.0).unwrap_err()),
        &FontOutlineError::UnavailableOutline
    );
}

#[test]
fn malformed_used_head_format_is_not_an_outline_success_or_empty() {
    let mut head = table(AHEM, *b"head").to_vec();
    head[50..52].copy_from_slice(&2i16.to_be_bytes());
    let (_, data) = registered(replace_tables(AHEM, vec![(*b"head", head)]));
    for glyph in [X, 3] {
        let error = geometry(&data, glyph, 360.0).unwrap_err();
        assert!(
            matches!(
                error.detail(),
                Some(ErrorDetail::FontOutline {
                    reason: FontOutlineError::InvalidFont,
                    ..
                })
            ),
            "{error:?}"
        );
    }
}

#[test]
fn actual_empty_child_loads_obey_provider_depth_and_cached_height() {
    for terminal in [35u16, 3] {
        for depth in [32u16, 33] {
            let changes: Vec<_> = (0..depth)
                .map(|i| {
                    (
                        (X as u16) + i,
                        composite(
                            if i + 1 == depth {
                                terminal
                            } else {
                                (X as u16) + i + 1
                            },
                            0,
                            0,
                        ),
                    )
                })
                .collect();
            let (_, data) = registered(replace_glyphs(&changes));
            let result = geometry(&data, X, 360.0);
            if depth == 32 {
                assert_eq!(
                    result.unwrap().bounds(),
                    if terminal == 3 {
                        None
                    } else {
                        Some(Rect::new(0.0, -288.0, 360.0, 360.0))
                    }
                );
            } else {
                assert_eq!(
                    reason(&result.unwrap_err()),
                    &FontOutlineError::Unsupported {
                        capability: FontOutlineCapability::CompositeGlyph
                    }
                );
            }
        }
    }
    // Cache a shallow empty leaf, then reach it after 32 intermediate calls.
    let mut root = composite(3, 0, 0);
    root[10..12].copy_from_slice(&35u16.to_be_bytes());
    root.extend([0, 3, 0, 59, 0, 0, 0, 0]);
    let mut changes = vec![(X as u16, root)];
    for i in 1..33u16 {
        changes.push((
            (X as u16) + i,
            composite(if i == 32 { 3 } else { (X as u16) + i + 1 }, 0, 0),
        ));
    }
    let (_, data) = registered(replace_glyphs(&changes));
    assert_eq!(
        reason(&geometry(&data, X, 360.0).unwrap_err()),
        &FontOutlineError::Unsupported {
            capability: FontOutlineCapability::CompositeGlyph
        }
    );
}

#[test]
fn sparse_and_malformed_variation_refuse_without_mutating_the_resource() {
    let tuple = [
        0, 1, 0, 10, 0, 20, 0xA0, 0, 0x40, 0, 8, 7, 0, 1, 1, 1, 1, 1, 1, 1, 7, 0, 100, 100, 0, 0,
        0, 0, 0, 0x87,
    ];
    let (_, data) = registered(variable_ahem_with_tuple(&tuple));
    let instance = data.unsynthesized_instance(1000.0, &[16384]).unwrap();
    assert_eq!(
        reason(
            &instance
                .glyph_outline(X, data.catalogue_identity())
                .unwrap_err()
        ),
        &FontOutlineError::Unsupported {
            capability: FontOutlineCapability::SparseVariation
        }
    );
    let bytes = variable_ahem();
    let mut gvar = table(&bytes, *b"gvar").to_vec();
    gvar[4..6].copy_from_slice(&2u16.to_be_bytes());
    let (_, data) = registered(replace_tables(&bytes, vec![(*b"gvar", gvar)]));
    assert_eq!(
        reason(&geometry(&data, X, 1000.0).unwrap_err()),
        &FontOutlineError::InvalidFont
    );
}

#[test]
fn numeric_refusals_preserve_same_instance_retry_and_true_empty() {
    let (_, data) = registered(AHEM.to_vec());
    let tiny = data.unsynthesized_instance(f32::from_bits(1), &[]).unwrap();
    let error = tiny
        .glyph_outline(X, data.catalogue_identity())
        .unwrap_err();
    assert!(
        matches!(
            error.detail(),
            Some(ErrorDetail::InvalidTextNumeric {
                phase: TextNumericPhase::GlyphOutline,
                ..
            })
        ),
        "{error:?}"
    );
    assert_eq!(
        tiny.glyph_outline(3, data.catalogue_identity())
            .unwrap()
            .bounds(),
        None
    );
    let synthesis = super::super::FontSynthesis::resolve(36.0, Some(89.99999), true).unwrap();
    let instance =
        EffectiveFontInstance::qualify_synthesized(data.face(), 36.0, &[], synthesis).unwrap();
    let error = instance
        .glyph_outline(X, data.catalogue_identity())
        .unwrap_err();
    assert!(
        matches!(
            error.detail(),
            Some(ErrorDetail::InvalidTextNumeric {
                phase: TextNumericPhase::GlyphOutline,
                ..
            })
        ),
        "{error:?}"
    );
    assert_eq!(
        geometry(&data, X, 360.0).unwrap().bounds(),
        Some(Rect::new(0.0, -288.0, 360.0, 360.0))
    );
    assert!(matches!(
        reserved::<u64>(usize::MAX),
        Err(Failure::Outline(FontOutlineError::CapacityOverflow))
    ));
}

#[test]
fn logical_outline_extent_preserves_f64_span_of_canonical_points() {
    let mut pen = BoundsPen::new(1.0, 0.0);
    pen.move_to(-f32::MAX, 0.0);
    pen.line_to(f32::MAX, 1.0);
    assert!(pen.failed.is_none());
    let Ok(bounds) = pen.finish() else {
        panic!("finite canonical points must not impose an f32 aggregate extent");
    };
    assert_eq!(bounds.origin.x, -f64::from(f32::MAX));
    assert_eq!(bounds.size.width, 2.0 * f64::from(f32::MAX));
    assert_eq!(bounds.size.height, 1.0);
}

#[test]
fn used_header_and_metric_extents_are_checked_even_for_actual_empty_glyphs() {
    let mut cases = Vec::new();
    for (offset, word) in [(0, 2i16), (12, 0), (50, -1), (52, 1)] {
        let mut head = table(AHEM, *b"head").to_vec();
        head[offset..offset + 2].copy_from_slice(&word.to_be_bytes());
        cases.push(replace_tables(AHEM, vec![(*b"head", head)]));
    }
    let mut maxp = table(AHEM, *b"maxp").to_vec();
    maxp[0..4].copy_from_slice(&0x00005000u32.to_be_bytes());
    cases.push(replace_tables(AHEM, vec![(*b"maxp", maxp)]));
    cases.push(replace_tables(
        AHEM,
        vec![(*b"maxp", table(AHEM, *b"maxp")[..6].to_vec())],
    ));
    let mut hhea = table(AHEM, *b"hhea").to_vec();
    hhea[32..34].copy_from_slice(&1i16.to_be_bytes());
    cases.push(replace_tables(AHEM, vec![(*b"hhea", hhea)]));
    cases.push(replace_tables(AHEM, vec![(*b"hmtx", vec![])]));
    cases.push(replace_tables(AHEM, vec![(*b"loca", vec![])]));
    for bytes in cases {
        let (_, data) = registered(bytes);
        for glyph in [X, 3] {
            let error = geometry(&data, glyph, 360.0).unwrap_err();
            assert!(
                matches!(
                    error.detail(),
                    Some(ErrorDetail::FontOutline {
                        reason: FontOutlineError::InvalidFont,
                        ..
                    })
                ),
                "{error:?}"
            );
        }
    }
}
