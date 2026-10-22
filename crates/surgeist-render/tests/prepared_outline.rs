//! Public headless behavior. Geometry expectations come from the licensed Ahem
//! glyph/source tables, not from the preparation output under test.
use surgeist_render::{
    Color, Error, ErrorCode, FontData, NormalizedCoordinate, OutlineCapability, OutlineLimits,
    OutlineNumericPhase, OutlinePreparationError, OutlineResource, OutlineSynthesis, PathElement,
    PreparedOutline, Rect, Scene, TextGlyph, TextInstance, TextPaint, TextRunBoundsKind, Transform,
    VariationTable,
};

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");
const X: u32 = 58;

fn limits() -> OutlineLimits {
    OutlineLimits::new(16, 1024, 1 << 20, 1 << 20)
}
fn instance(size: f32, coordinates: Vec<NormalizedCoordinate>, skew: f32) -> TextInstance {
    TextInstance::try_new(
        size,
        coordinates,
        OutlineSynthesis::try_new(skew, 0.0).unwrap(),
    )
    .unwrap()
}
fn font(bytes: &[u8]) -> FontData {
    FontData::try_from_bytes(bytes.to_vec(), 0).unwrap()
}
fn glyph() -> TextGlyph {
    TextGlyph::try_new(X, -20.0, 30.0, -123.0).unwrap()
}
fn diagnostic(error: &Error) -> OutlinePreparationError {
    assert_eq!(error.code(), ErrorCode::OutlinePreparation);
    *error.outline_preparation_diagnostic().unwrap()
}
fn prepare(bytes: &[u8], instance: &TextInstance) -> PreparedOutline {
    font(bytes)
        .prepare_outline(instance, &[glyph()], limits())
        .unwrap()
}

#[test]
fn prepared_outline_retains_real_ahem_ink_separately_from_negative_advance() {
    // X is the rectangle (0,-200)..(1000,800), UPEM1000; the run flips y.
    let prepared = prepare(AHEM, &instance(1000.0, vec![], 0.0));
    assert_eq!(
        prepared.bounds().ink_rect(),
        Some(Rect::try_new(-20.0, -770.0, 1000.0, 1000.0).unwrap())
    );
    assert_eq!(prepared.glyphs(), &[glyph()]);
    let elements: Vec<_> = prepared.path_elements().collect();
    assert_eq!(elements.len(), 5);
    assert!(matches!(elements[0], PathElement::MoveTo(p) if p.x() == -20.0 && p.y() == -770.0));
    assert!(matches!(elements[1], PathElement::LineTo(p) if p.x() == 980.0 && p.y() == -770.0));
}

#[test]
fn prepared_outline_true_empty_and_empty_input_have_no_authored_bounds_escape() {
    let font = font(AHEM);
    let instance = instance(1000.0, vec![], 0.0);
    for glyphs in [
        vec![],
        vec![TextGlyph::try_new(3, 0.0, 0.0, 1000.0).unwrap()],
    ] {
        let prepared = font.prepare_outline(&instance, &glyphs, limits()).unwrap();
        assert_eq!(prepared.bounds().kind(), TextRunBoundsKind::Empty);
        assert_eq!(prepared.path_elements().count(), 0);
    }
}

#[test]
fn prepared_outline_actual_skew_precedes_reflection_and_local_placement() {
    let prepared = prepare(AHEM, &instance(1000.0, vec![], 0.5));
    // Font y800 shifts x+400, font y-200 shifts x-100, before local (-20,30).
    assert_eq!(
        prepared.bounds().ink_rect(),
        Some(Rect::try_new(-120.0, -770.0, 1500.0, 1000.0).unwrap())
    );
    assert!(
        matches!(prepared.path_elements().next(), Some(PathElement::MoveTo(p)) if p.x() == 380.0 && p.y() == -770.0)
    );
}

#[test]
fn prepared_outline_exact_coordinate_lattice_rejects_loss_and_wrong_axis_inventory() {
    assert_eq!(
        NormalizedCoordinate::try_from_bits(8192).unwrap(),
        NormalizedCoordinate::try_from_scalar(0.5).unwrap()
    );
    assert!(matches!(
        diagnostic(&NormalizedCoordinate::try_from_scalar(0.1).unwrap_err()),
        OutlinePreparationError::CoordinateLoss { .. }
    ));
    assert!(NormalizedCoordinate::try_from_bits(16385).is_err());
    for scalar in [f32::NAN, f32::INFINITY, -1.01] {
        assert!(NormalizedCoordinate::try_from_scalar(scalar).is_err());
    }
    let extra = instance(
        1000.0,
        vec![NormalizedCoordinate::try_from_bits(0).unwrap()],
        0.0,
    );
    assert_eq!(
        diagnostic(
            &font(AHEM)
                .prepare_outline(&extra, &[glyph()], limits())
                .unwrap_err()
        ),
        OutlinePreparationError::CoordinateCount {
            supplied: 1,
            required: 0
        }
    );
}

#[test]
fn prepared_outline_real_nondefault_variation_changes_exact_geometry_and_retains_bits() {
    let bytes = variable_ahem();
    for (bits, expected_width) in [(0, 1000.0), (8192, 1050.0), (16384, 1100.0)] {
        let instance = instance(
            1000.0,
            vec![NormalizedCoordinate::try_from_bits(bits).unwrap()],
            0.0,
        );
        let prepared = prepare(&bytes, &instance);
        assert_eq!(prepared.instance(), &instance);
        assert_eq!(prepared.instance().coordinates()[0].bits(), bits);
        assert_eq!(
            prepared.bounds().ink_rect(),
            Some(Rect::try_new(-20.0, -770.0, expected_width, 1000.0).unwrap())
        );
        assert!(
            prepared
                .path_elements()
                .any(|e| matches!(e, PathElement::LineTo(p) if p.x() == expected_width - 20.0))
        );
    }
    let missing = instance(1000.0, vec![], 0.0);
    assert_eq!(
        diagnostic(
            &font(&bytes)
                .prepare_outline(&missing, &[glyph()], limits())
                .unwrap_err()
        ),
        OutlinePreparationError::CoordinateCount {
            supplied: 0,
            required: 1
        }
    );
}

#[test]
fn prepared_outline_present_malformed_variation_metadata_cannot_become_default_success() {
    let coordinate = vec![NormalizedCoordinate::try_from_bits(16384).unwrap()];
    for (tag, table) in [
        (*b"fvar", VariationTable::Fvar),
        (*b"gvar", VariationTable::Gvar),
    ] {
        let bytes = replace_tables(&variable_ahem(), vec![(tag, vec![0; 4])]);
        assert_eq!(
            diagnostic(
                &font(&bytes)
                    .prepare_outline(
                        &instance(1000.0, coordinate.clone(), 0.0),
                        &[glyph()],
                        limits()
                    )
                    .unwrap_err()
            ),
            OutlinePreparationError::InvalidVariation { table }
        );
        let mut outside = variable_ahem();
        let record = (0..usize::from(u16_at(&outside, 4)))
            .map(|i| 12 + i * 16)
            .find(|r| outside[*r..*r + 4] == tag)
            .unwrap();
        outside[record + 8..record + 12].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(
            diagnostic(
                &font(&outside)
                    .prepare_outline(
                        &instance(1000.0, coordinate.clone(), 0.0),
                        &[glyph()],
                        limits()
                    )
                    .unwrap_err()
            ),
            OutlinePreparationError::InvalidVariation { table }
        );
    }
    let bytes = variable_ahem();
    let mut gvar = table(&bytes, *b"gvar").to_vec();
    // Missing y-delta stream after a valid header and x-delta stream. The provider
    // otherwise intentionally ignores a failed gvar decode during glyf drawing.
    gvar.pop();
    let malformed = replace_tables(&bytes, vec![(*b"gvar", gvar)]);
    assert_eq!(
        diagnostic(
            &font(&malformed)
                .prepare_outline(&instance(1000.0, coordinate, 0.0), &[glyph()], limits())
                .unwrap_err()
        ),
        OutlinePreparationError::InvalidVariation {
            table: VariationTable::Gvar
        }
    );
    // A malformed higher-priority VARC source must not silently use the valid
    // glyf table in the same face.
    let malformed_varc = replace_tables(AHEM, vec![(*b"VARC", vec![0; 4])]);
    assert_eq!(
        diagnostic(
            &font(&malformed_varc)
                .prepare_outline(&instance(1000.0, vec![], 0.0), &[glyph()], limits())
                .unwrap_err()
        ),
        OutlinePreparationError::UnsupportedOutline {
            capability: OutlineCapability::VariableComposite
        }
    );
}

#[test]
fn prepared_outline_limits_refuse_whole_runs_and_retry_preserves_original_inputs() {
    let font = font(AHEM);
    let instance = instance(1000.0, vec![], 0.0);
    let glyphs = [glyph(), glyph()];
    for (limits, resource) in [
        (
            OutlineLimits::new(1, 1024, 1 << 20, 1 << 20),
            OutlineResource::Glyphs,
        ),
        (
            OutlineLimits::new(16, 4, 1 << 20, 1 << 20),
            OutlineResource::PathElements,
        ),
        (
            OutlineLimits::new(16, 1024, 0, 1 << 20),
            OutlineResource::ScratchBytes,
        ),
        (
            OutlineLimits::new(16, 1024, 1 << 20, 0),
            OutlineResource::RetainedBytes,
        ),
    ] {
        assert!(
            matches!(diagnostic(&font.prepare_outline(&instance, &glyphs, limits).unwrap_err()), OutlinePreparationError::Limit { resource: actual, .. } if actual == resource)
        );
    }
    let prepared = font.prepare_outline(&instance, &glyphs, limits()).unwrap();
    assert_eq!(prepared.glyphs(), glyphs);
    assert_eq!(prepared.path_elements().count(), 10);
    // Blob boxes its input bytes: discarded input Vec spare capacity is not
    // retained font custody. The declared charge is for boxed extent/headers.
    let mut bytes = Vec::with_capacity(2 << 20);
    bytes.extend_from_slice(AHEM);
    let overcapacity = FontData::try_from_bytes(bytes, 0).unwrap();
    let required = |font: &FontData| match diagnostic(
        &font
            .prepare_outline(&instance, &[], OutlineLimits::new(0, 0, 0, 0))
            .unwrap_err(),
    ) {
        OutlinePreparationError::Limit {
            resource: OutlineResource::RetainedBytes,
            required,
            ..
        } => required,
        other => panic!("expected declared retained admission, got {other:?}"),
    };
    let charge = required(&font);
    assert_eq!(required(&overcapacity), charge);
    // Readable SFNT tables permit trailing bytes. The independently known
    // extra boxed extent must be charged, unlike discarded input capacity.
    const PADDING: usize = 257;
    let mut padded = AHEM.to_vec();
    padded.extend([0; PADDING]);
    let padded = FontData::try_from_bytes(padded, 0).unwrap();
    assert_eq!(required(&padded), charge + PADDING);
    assert!(
        matches!(diagnostic(&padded.prepare_outline(&instance, &[], OutlineLimits::new(0,0,0,charge)).unwrap_err()), OutlinePreparationError::Limit { resource: OutlineResource::RetainedBytes, required, .. } if required == charge + PADDING)
    );
    assert!(
        padded
            .prepare_outline(
                &instance,
                &[],
                OutlineLimits::new(0, 0, 0, charge + PADDING)
            )
            .is_ok()
    );
    assert!(
        overcapacity
            .prepare_outline(&instance, &[], OutlineLimits::new(0, 0, 0, charge))
            .is_ok()
    );
    assert!(
        matches!(diagnostic(&font.prepare_outline(&instance, &[], OutlineLimits::new(0, 0, 0, charge - 1)).unwrap_err()), OutlinePreparationError::Limit { resource: OutlineResource::RetainedBytes, required, .. } if required == charge)
    );
}

#[test]
fn prepared_outline_numeric_and_unsupported_synthesis_refusals_do_not_publish_empty() {
    let font = font(AHEM);
    // Actual path underflow and overflow, rather than an arbitrary size cap.
    for (size, skew) in [(f32::from_bits(1), 0.0), (f32::MAX, 1.0)] {
        assert!(matches!(
            diagnostic(
                &font
                    .prepare_outline(
                        &instance(size, vec![], skew),
                        &[TextGlyph::try_new(X, 0.0, 0.0, 1.0).unwrap()],
                        limits(),
                    )
                    .unwrap_err()
            ),
            OutlinePreparationError::Numeric { .. }
        ));
    }
    let huge = TextGlyph::try_new(X, f32::MAX, 0.0, 1.0).unwrap();
    assert!(matches!(
        diagnostic(
            &font
                .prepare_outline(&instance(1000.0, vec![], 0.0), &[huge], limits())
                .unwrap_err()
        ),
        OutlinePreparationError::Numeric { .. }
    ));
    let bold = TextInstance::try_new(1000.0, vec![], OutlineSynthesis::try_new(0.0, 1.0).unwrap())
        .unwrap();
    assert_eq!(
        diagnostic(
            &font
                .prepare_outline(&bold, &[glyph()], limits())
                .unwrap_err()
        ),
        OutlinePreparationError::UnsupportedEmboldening
    );
    let invalid = TextGlyph::try_new(281, 0.0, 0.0, 0.0).unwrap();
    assert_eq!(
        diagnostic(
            &font
                .prepare_outline(&instance(1000.0, vec![], 0.0), &[invalid], limits())
                .unwrap_err()
        ),
        OutlinePreparationError::GlyphRange {
            glyph: 281,
            count: 281
        }
    );
}

#[test]
fn prepared_outline_unscaled_provider_preserves_large_and_non_lattice_sizes() {
    // Ahem's literal 1000-unit square at UPEM1000. The unscaled provider never
    // converts 1M ppem to the overflowing requested-size Fixed quotient.
    let large = prepare(AHEM, &instance(1_000_000.0, vec![], 0.0));
    assert_eq!(
        large.bounds().ink_rect(),
        Some(Rect::try_new(-20.0, -799_970.0, 1_000_000.0, 1_000_000.0).unwrap())
    );
    assert!(
        large
            .path_elements()
            .any(|p| matches!(p, PathElement::LineTo(p) if p.x() == 999_980.0))
    );
    let fractional = prepare(AHEM, &instance(10.1, vec![], 0.0));
    // Apply size10.1 to literal Ahem corners then round once at the canonical
    // path boundary. No F26Dot6 size quantization occurs.
    let top = f64::from(21.92_f32);
    let bottom = f64::from(32.02_f32);
    let right = f64::from(-9.9_f32);
    assert_eq!(
        fractional.bounds().ink_rect(),
        Some(Rect::try_new(-20.0, top, right + 20.0, bottom - top).unwrap())
    );
}

#[test]
fn prepared_outline_harfbuzz_first_offcurve_contour_has_literal_start_and_control() {
    // One OFF point followed by two ON points. HB starts at the second point;
    // the FT path instead starts at the last point. These are literal glyph facts.
    let mut data = vec![];
    for n in [1i16, 0, 0, 1000, 1000, 2, 0] {
        data.extend(n.to_be_bytes());
    }
    data.extend([0, 1, 1]);
    for n in [500i16, 500, -1000, 1000, -1000, 0] {
        data.extend(n.to_be_bytes());
    }
    let bytes = replace_x_glyph(data);
    let prepared = font(&bytes)
        .prepare_outline(
            &instance(1000.0, vec![], 0.0),
            &[TextGlyph::try_new(X, 0.0, 0.0, 0.0).unwrap()],
            limits(),
        )
        .unwrap();
    let elements: Vec<_> = prepared.path_elements().collect();
    assert!(matches!(elements[0], PathElement::MoveTo(p) if p.x() == 1000.0 && p.y() == 0.0));
    assert!(matches!(elements[1], PathElement::LineTo(p) if p.x() == 0.0 && p.y() == 0.0));
    assert!(
        matches!(elements[2], PathElement::QuadTo(c,p) if c.x() == 500.0 && c.y() == -1000.0 && p.x() == 1000.0 && p.y() == 0.0)
    );
    assert!(matches!(elements[3], PathElement::Close));
}

#[test]
fn prepared_outline_composite_and_sparse_variation_refuse_the_whole_run() {
    let mut composite = vec![];
    for n in [-1i16, 0, -200, 1000, 800, 11, 35, 0, 0, 8192] {
        composite.extend(n.to_be_bytes());
    }
    let composite = replace_x_glyph(composite);
    let valid_then_composite = [TextGlyph::try_new(35, 0.0, 0.0, 1000.0).unwrap(), glyph()];
    assert_eq!(
        diagnostic(
            &font(&composite)
                .prepare_outline(
                    &instance(1000.0, vec![], 0.0),
                    &valid_then_composite,
                    limits()
                )
                .unwrap_err()
        ),
        OutlinePreparationError::UnsupportedOutline {
            capability: OutlineCapability::CompositeGlyph
        }
    );
    // Explicit point indices are sparse, even when every point is listed. IUP
    // is intentionally outside this qualified entry rather than silently run.
    let tuple = [
        0, 1, 0, 10, 0, 20, 0xA0, 0, 0x40, 0, 8, 7, 0, 1, 1, 1, 1, 1, 1, 1, 7, 0, 100, 100, 0, 0,
        0, 0, 0, 0x87,
    ];
    let sparse = variable_ahem_with_tuple(&tuple);
    assert_eq!(
        diagnostic(
            &font(&sparse)
                .prepare_outline(
                    &instance(
                        1000.0,
                        vec![NormalizedCoordinate::try_from_bits(16384).unwrap()],
                        0.0
                    ),
                    &[glyph()],
                    limits()
                )
                .unwrap_err()
        ),
        OutlinePreparationError::UnsupportedOutline {
            capability: OutlineCapability::SparseVariation
        }
    );
}

#[test]
fn prepared_outline_active_partial_delta_fixed_overflow_is_not_default_success() {
    // An i32 delta32768 is exact in f32 at scalarONE, but the provider's partial
    // scalar route first shifts it into signed16.16 and would wrap at +0.5.
    let mut tuple = vec![0, 1, 0, 10, 0, 35, 0xA0, 0, 0x40, 0, 0, 0xC7];
    for delta in [0i32, 32768, 32768, 0, 0, 0, 0, 0] {
        tuple.extend(delta.to_be_bytes());
    }
    tuple.push(0x87);
    let bytes = variable_ahem_with_tuple(&tuple);
    let full = prepare(
        &bytes,
        &instance(
            1000.0,
            vec![NormalizedCoordinate::try_from_bits(16384).unwrap()],
            0.0,
        ),
    );
    assert_eq!(full.bounds().ink_rect().unwrap().width(), 33768.0);
    let error = font(&bytes)
        .prepare_outline(
            &instance(
                1000.0,
                vec![NormalizedCoordinate::try_from_bits(8192).unwrap()],
                0.0,
            ),
            &[glyph()],
            limits(),
        )
        .unwrap_err();
    assert!(matches!(
        diagnostic(&error),
        OutlinePreparationError::Numeric {
            phase: OutlineNumericPhase::ProviderPoint,
            ..
        }
    ));
}

#[test]
fn prepared_outline_shared_identity_and_scene_survive_original_input_drop() {
    let prepared = {
        let bytes = AHEM.to_vec();
        let font = font(&bytes);
        let glyphs = vec![glyph()];
        font.prepare_outline(&instance(1000.0, vec![], 0.0), &glyphs, limits())
            .unwrap()
    };
    let cloned = prepared.clone();
    let independent = prepare(AHEM, &instance(1000.0, vec![], 0.0));
    assert!(prepared.is_same_preparation(&cloned));
    assert!(!prepared.is_same_preparation(&independent));
    let mut scene = Scene::new();
    scene.prepared_outline(
        prepared,
        Transform::identity(),
        TextPaint::try_fill(Color::BLACK.into()).unwrap(),
    );
    drop(cloned);
    assert_eq!(scene.stats().glyphs, 1);
    let moved = scene.clone();
    drop(scene);
    assert_eq!(moved.stats().glyphs, 1);
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let record = (0..usize::from(u16_at(bytes, 4)))
        .map(|i| 12 + i * 16)
        .find(|r| bytes[*r..*r + 4] == tag)
        .unwrap();
    let offset = u32_at(bytes, record + 8) as usize;
    let len = u32_at(bytes, record + 12) as usize;
    &bytes[offset..offset + len]
}
fn replace_tables(bytes: &[u8], replacements: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    let mut tables: Vec<_> = (0..usize::from(u16_at(bytes, 4)))
        .map(|i| {
            let r = 12 + i * 16;
            let tag: [u8; 4] = bytes[r..r + 4].try_into().unwrap();
            (tag, table(bytes, tag).to_vec())
        })
        .collect();
    for (tag, data) in replacements {
        if let Some(t) = tables.iter_mut().find(|t| t.0 == tag) {
            t.1 = data;
        } else {
            tables.push((tag, data));
        }
    }
    tables.sort_by_key(|t| t.0);
    let mut output = vec![0; 12 + tables.len() * 16];
    output[..4].copy_from_slice(&bytes[..4]);
    output[4..6].copy_from_slice(&u16::try_from(tables.len()).unwrap().to_be_bytes());
    let power =
        tables.len().next_power_of_two() / if tables.len().is_power_of_two() { 1 } else { 2 };
    output[6..8].copy_from_slice(&u16::try_from(power * 16).unwrap().to_be_bytes());
    output[8..10].copy_from_slice(&u16::try_from(power.ilog2()).unwrap().to_be_bytes());
    output[10..12].copy_from_slice(
        &u16::try_from((tables.len() - power) * 16)
            .unwrap()
            .to_be_bytes(),
    );
    for (i, (tag, data)) in tables.into_iter().enumerate() {
        while !output.len().is_multiple_of(4) {
            output.push(0);
        }
        let r = 12 + i * 16;
        let offset = u32::try_from(output.len()).unwrap();
        output[r..r + 4].copy_from_slice(&tag);
        output[r + 8..r + 12].copy_from_slice(&offset.to_be_bytes());
        output[r + 12..r + 16].copy_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        output.extend(data);
    }
    output
}

// Authored derivative of licensed Ahem. One wght axis [0,1], default0. At +1,
// X's two right-edge points move +100 font units, while all phantom points and
// y deltas stay zero. Thus width is 1000 + coordinate*100 at UPEM1000.
fn variable_ahem() -> Vec<u8> {
    variable_ahem_with_tuple(&[
        0, 1, 0, 10, 0, 11, 0xA0, 0, 0x40, 0, 0, 7, 0, 100, 100, 0, 0, 0, 0, 0, 0x87,
    ])
}
fn variable_ahem_with_tuple(tuple: &[u8]) -> Vec<u8> {
    let mut fvar = vec![];
    fvar.extend(0x00010000u32.to_be_bytes());
    for value in [16u16, 2, 1, 20, 0, 8] {
        fvar.extend(value.to_be_bytes());
    }
    fvar.extend(*b"wght");
    for value in [0i32, 0, 65536] {
        fvar.extend(value.to_be_bytes());
    }
    fvar.extend(0u16.to_be_bytes());
    fvar.extend(256u16.to_be_bytes());
    let count = usize::from(u16_at(table(AHEM, *b"maxp"), 4));
    let data_offset = 20 + (count + 1) * 4;
    let mut gvar = vec![];
    gvar.extend(0x00010000u32.to_be_bytes());
    gvar.extend(1u16.to_be_bytes());
    gvar.extend(0u16.to_be_bytes());
    gvar.extend(u32::try_from(data_offset).unwrap().to_be_bytes());
    gvar.extend(u16::try_from(count).unwrap().to_be_bytes());
    gvar.extend(1u16.to_be_bytes());
    gvar.extend(u32::try_from(data_offset).unwrap().to_be_bytes());
    for i in 0..=count {
        gvar.extend(
            (if i <= X as usize {
                0u32
            } else {
                tuple.len() as u32
            })
            .to_be_bytes(),
        );
    }
    gvar.extend_from_slice(tuple);
    replace_tables(AHEM, vec![(*b"fvar", fvar), (*b"gvar", gvar)])
}

// Rebuild only X's literal glyf/loca data; original Ahem remains unchanged.
fn replace_x_glyph(data: Vec<u8>) -> Vec<u8> {
    let loca = table(AHEM, *b"loca");
    let glyf = table(AHEM, *b"glyf");
    let count = usize::from(u16_at(table(AHEM, *b"maxp"), 4));
    let mut new_loca = vec![];
    let mut new_glyf = vec![];
    for id in 0..count {
        new_loca.extend(u16::try_from(new_glyf.len() / 2).unwrap().to_be_bytes());
        if id == X as usize {
            new_glyf.extend_from_slice(&data);
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

#[path = "prepared_outline/static_composite.rs"]
mod static_composite;

#[path = "prepared_outline/synthesis.rs"]
mod synthesis;
