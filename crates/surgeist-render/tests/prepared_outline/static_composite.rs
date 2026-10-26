//! Public static composite and shared simple-decoder qualification.
//! Source contour facts, not provider-produced paths, determine geometry.
use super::*;

const A: u16 = 35;
const INNER: u16 = 59;

// Extend only the existing glyf/loca fixture operation to several actual glyphs.
// All unrelated SFNT table copying stays in the existing parent helper.
fn replace_glyphs(replacements: &[(u16, Vec<u8>)]) -> Vec<u8> {
    assert_eq!(u16_at(table(AHEM, *b"head"), 50), 0);
    let loca = table(AHEM, *b"loca");
    let glyf = table(AHEM, *b"glyf");
    let count = usize::from(u16_at(table(AHEM, *b"maxp"), 4));
    let mut new_loca = Vec::new();
    let mut new_glyf = Vec::new();
    let mut head = table(AHEM, *b"head").to_vec();
    for id in 0..count {
        new_loca.extend(u16::try_from(new_glyf.len() / 2).unwrap().to_be_bytes());
        if let Some((_, data)) = replacements
            .iter()
            .find(|(glyph, _)| usize::from(*glyph) == id)
        {
            new_glyf.extend_from_slice(data);
            // Keep global font bounds consistent with actual derivative glyphs.
            if data.len() >= 10 {
                for (glyph_at, head_at, minimum) in
                    [(2, 36, true), (4, 38, true), (6, 40, false), (8, 42, false)]
                {
                    let value =
                        i16::from_be_bytes(data[glyph_at..glyph_at + 2].try_into().unwrap());
                    let old = i16::from_be_bytes(head[head_at..head_at + 2].try_into().unwrap());
                    head[head_at..head_at + 2].copy_from_slice(
                        &(if minimum {
                            old.min(value)
                        } else {
                            old.max(value)
                        })
                        .to_be_bytes(),
                    );
                }
            }
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
    replace_tables(
        AHEM,
        vec![(*b"head", head), (*b"loca", new_loca), (*b"glyf", new_glyf)],
    )
}
fn lsb(bytes: &[u8], glyph: u16, value: i16) -> Vec<u8> {
    let count = usize::from(u16_at(table(bytes, *b"hhea"), 34));
    let glyph = usize::from(glyph);
    let at = if glyph < count {
        glyph * 4 + 2
    } else {
        count * 4 + (glyph - count) * 2
    };
    let mut hmtx = table(bytes, *b"hmtx").to_vec();
    hmtx[at..at + 2].copy_from_slice(&value.to_be_bytes());
    replace_tables(bytes, vec![(*b"hmtx", hmtx)])
}
fn header(contours: i16, bbox: [i16; 4]) -> Vec<u8> {
    [contours, bbox[0], bbox[1], bbox[2], bbox[3]]
        .into_iter()
        .flat_map(i16::to_be_bytes)
        .collect()
}
fn composite(bbox: [i16; 4], components: &[(u16, u16, i16, i16)], tail: &[u8]) -> Vec<u8> {
    let mut data = header(-1, bbox);
    for &(flags, glyph, x, y) in components {
        data.extend(flags.to_be_bytes());
        data.extend(glyph.to_be_bytes());
        if flags & 1 != 0 {
            data.extend(x.to_be_bytes());
            data.extend(y.to_be_bytes());
        } else {
            data.push(i8::try_from(x).unwrap().to_be_bytes()[0]);
            data.push(i8::try_from(y).unwrap().to_be_bytes()[0]);
        }
    }
    data.extend_from_slice(tail);
    data
}
fn simple_square(flags: &[u8], residual: &[u8]) -> Vec<u8> {
    let mut data = header(1, [0, 0, 100, 100]);
    data.extend(3u16.to_be_bytes());
    data.extend(1u16.to_be_bytes());
    data.push(0x00); // complete opaque unhinted program
    data.extend_from_slice(flags);
    for delta in [0i16, 100, 0, -100, 0, 0, 100, 0] {
        data.extend(delta.to_be_bytes());
    }
    data.extend_from_slice(residual);
    data
}
fn malformed_repeat() -> Vec<u8> {
    let mut data = header(1, [0, 0, 0, 0]);
    data.extend(3u16.to_be_bytes());
    data.extend(0u16.to_be_bytes());
    data.extend([0x39, 0xff]); // claims 256 flags for four points, not residual
    data
}
fn missing_zero_contour_program() -> Vec<u8> {
    let mut data = header(0, [0, 0, 0, 0]);
    data.extend(1u16.to_be_bytes());
    data
}
fn one_point_repeat_zero() -> Vec<u8> {
    let mut data = header(1, [0, 0, 0, 0]);
    data.extend(0u16.to_be_bytes());
    data.extend(0u16.to_be_bytes());
    data.extend([0x39, 0]);
    data
}
fn underconsumed_flags() -> Vec<u8> {
    let mut data = header(1, [0, 0, 2000, 4000]);
    data.extend(3u16.to_be_bytes());
    data.extend(0u16.to_be_bytes());
    data.extend([0x39, 0, 0x39, 0, 0x01, 0x01]);
    for value in [1000i16, 1000, 3000, 1000] {
        data.extend(value.to_be_bytes());
    }
    data
}
fn at_origin() -> TextGlyph {
    TextGlyph::try_new(X, 0.0, 0.0, 0.0).unwrap()
}
fn all_done(failures: Vec<String>) {
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn expect_ink(
    label: &str,
    bytes: &[u8],
    glyphs: &[TextGlyph],
    expected: Rect,
    elements: usize,
    failures: &mut Vec<String>,
) {
    let font = font(bytes);
    let retained = font.clone();
    let instance = instance(1000.0, vec![], 0.0);
    let kept = instance.clone();
    let glyph_copy = glyphs.to_vec();
    let result = font.prepare_glyph_run(&instance, glyphs, limits());
    assert_eq!(font, retained);
    assert_eq!(instance, kept);
    assert_eq!(glyphs, glyph_copy);
    match result {
        Err(error) => {
            eprintln!("{label}: public refusal {:?}", diagnostic(&error));
            failures.push(format!("{label}: {:?}", diagnostic(&error)));
        }
        Ok(prepared) => {
            assert_eq!(prepared.bounds().ink_rect(), Some(expected), "{label}");
            assert_eq!(prepared.path_elements().count(), elements, "{label}");
            assert_eq!(prepared.font(), &retained);
            assert_eq!(prepared.instance(), &kept);
            assert_eq!(prepared.glyphs(), glyphs);
            let shared = prepared.clone();
            assert!(prepared.is_same_preparation(&shared));
        }
    }
}
fn expect_error(
    label: &str,
    bytes: &[u8],
    glyphs: &[TextGlyph],
    expected: GlyphPreparationError,
    failures: &mut Vec<String>,
) {
    let font = font(bytes);
    let retained = font.clone();
    let instance = instance(1000.0, vec![], 0.0);
    let kept = instance.clone();
    let glyph_copy = glyphs.to_vec();
    let result = font.prepare_glyph_run(&instance, glyphs, limits());
    assert_eq!(font, retained);
    assert_eq!(instance, kept);
    assert_eq!(glyphs, glyph_copy);
    match result {
        Err(error) => {
            let actual = diagnostic(&error);
            eprintln!("{label}: actual={actual:?}, expected={expected:?}");
            if actual != expected {
                failures.push(format!("{label}: actual={actual:?}, expected={expected:?}"));
            }
        }
        Ok(prepared) => {
            eprintln!(
                "{label}: actual SUCCESS bounds={:?}, path={:?}; expected={expected:?}",
                prepared.bounds(),
                prepared.path_elements().collect::<Vec<_>>()
            );
            failures.push(format!("{label}: success instead of {expected:?}"));
        }
    }
}

#[test]
fn translated_components_keep_signed_offsets_and_root_metric_normalization() {
    let mut failures = Vec::new();
    for (label, dx, dy, flags, bearing, x, y) in [
        ("zero word offset", 0, 0, 3, 0, -20.0, -770.0),
        ("signed word translation", 120, -80, 3, 120, 100.0, -690.0),
        (
            "root LSB changes visible translation",
            120,
            -80,
            3,
            0,
            -20.0,
            -690.0,
        ),
        ("signed byte translation", -120, 80, 2, -120, -140.0, -850.0),
    ] {
        let data = composite(
            [dx, -200 + dy, 1000 + dx, 800 + dy],
            &[(flags, A, dx, dy)],
            &[],
        );
        let bytes = lsb(&replace_glyphs(&[(X as u16, data)]), X as u16, bearing);
        expect_ink(
            label,
            &bytes,
            &[glyph()],
            Rect::try_new(x, y, 1000.0, 1000.0).unwrap(),
            5,
            &mut failures,
        );
    }
    // Explicit identity scale and complete opaque unhinted program are supported.
    for (label, flags, tail) in [
        ("explicit identity scale", 0x000b, vec![0x40, 0]),
        ("terminal instruction program", 0x0103, vec![0, 1, 0xff]),
    ] {
        let data = composite([0, -200, 1000, 800], &[(flags, A, 0, 0)], &tail);
        expect_ink(
            label,
            &replace_glyphs(&[(X as u16, data)]),
            &[glyph()],
            Rect::try_new(-20.0, -770.0, 1000.0, 1000.0).unwrap(),
            5,
            &mut failures,
        );
    }
    all_done(failures);
}

#[test]
fn nested_and_repeated_components_retain_each_actual_contour() {
    let mut failures = Vec::new();
    let inner = composite([120, -280, 1120, 720], &[(3, A, 120, -80)], &[]);
    let outer = composite([100, -250, 1100, 750], &[(3, INNER, -20, 30)], &[]);
    let bytes = lsb(
        &replace_glyphs(&[(INNER, inner.clone()), (X as u16, outer)]),
        X as u16,
        100,
    );
    expect_ink(
        "nested signed translations without child metric shifts",
        &bytes,
        &[glyph()],
        Rect::try_new(80.0, -720.0, 1000.0, 1000.0).unwrap(),
        5,
        &mut failures,
    );
    for offset in [0, 500] {
        let outer = composite(
            [120, -280, 1120 + offset, 720],
            &[(0x23, INNER, 0, 0), (3, INNER, offset, 0)],
            &[],
        );
        let bytes = lsb(
            &replace_glyphs(&[(INNER, inner.clone()), (X as u16, outer)]),
            X as u16,
            120,
        );
        expect_ink(
            &format!("shared descendant repeated at {offset}"),
            &bytes,
            &[glyph()],
            Rect::try_new(100.0, -690.0, 1000.0 + f64::from(offset), 1000.0).unwrap(),
            10,
            &mut failures,
        );
    }
    all_done(failures);
}

#[test]
fn direct_malformed_structure_never_becomes_empty_or_clamped_points() {
    let mut failures = Vec::new();
    let mut missing_flags = header(1, [0, 0, 0, 0]);
    missing_flags.extend(3u16.to_be_bytes());
    missing_flags.extend(0u16.to_be_bytes());
    let mut missing_repeat = missing_flags.clone();
    missing_repeat.extend([0x31, 0x39]);
    let mut missing_x = header(1, [0, 0, 100, 100]);
    missing_x.extend(0u16.to_be_bytes());
    missing_x.extend(0u16.to_be_bytes());
    missing_x.extend([1]);
    let mut missing_y = missing_x.clone();
    missing_y.extend(0i16.to_be_bytes());
    let mut missing_endpoints = header(1, [0, 0, 0, 0]);
    // Odd slices receive one loca padding byte; use even truncations so it cannot
    // accidentally supply the missing u16 extent.
    missing_endpoints.truncate(10);
    for (label, data) in [
        ("excess repeat is not clamped", malformed_repeat()),
        (
            "zero contours still need declared program",
            missing_zero_contour_program(),
        ),
        ("contour endpoint array missing", missing_endpoints),
        ("logical flags absent", missing_flags),
        ("repeat byte absent", missing_repeat),
        ("X coordinate bytes absent", missing_x),
        ("Y coordinate bytes absent", missing_y),
    ] {
        expect_error(
            label,
            &replace_glyphs(&[(X as u16, data)]),
            &[at_origin()],
            GlyphPreparationError::InvalidFont,
            &mut failures,
        );
    }
    all_done(failures);
}

// Provider observations are diagnostics only. The admitted-domain oracle is
// the complete original stream: F>N requires Provider before decode/draw.
fn observe_raw_provider(label: &str, bytes: &[u8]) {
    use skrifa::{
        GlyphId,
        raw::{
            TableProvider,
            tables::glyf::{Glyph, PointFlags},
            types::Point,
        },
    };
    let font = skrifa::FontRef::from_index(bytes, 0).unwrap();
    let head = font.head().unwrap();
    let loca = font.loca(Some(head.index_to_loc_format() == 1)).unwrap();
    let glyf = font.glyf().unwrap();
    let Some(Glyph::Simple(simple)) = loca.get_glyf(GlyphId::new(X), &glyf).unwrap() else {
        panic!("actual simple fixture")
    };
    let mut points = vec![Point { x: 0i32, y: 0i32 }; simple.num_points()];
    let mut flags = vec![PointFlags::default(); points.len()];
    let result = simple.read_points_fast(&mut points, &mut flags);
    eprintln!(
        "RAW {label}: result={result:?}, points={points:?}, on_curve={:?}",
        flags
            .iter()
            .map(|flag| flag.is_on_curve())
            .collect::<Vec<_>>()
    );
}

#[test]
fn valid_repeat_prefix_outside_provider_domain_never_publishes_unfaithful_geometry() {
    let mut failures = Vec::new();
    for (label, data) in [
        ("one point repeat0 F2>N1", one_point_repeat_zero()),
        ("four point prefix F6>N4", underconsumed_flags()),
    ] {
        let bytes = lsb(&replace_glyphs(&[(X as u16, data)]), X as u16, 0);
        observe_raw_provider(label, &bytes);
        expect_error(
            label,
            &bytes,
            &[at_origin()],
            GlyphPreparationError::Provider { glyph: X },
            &mut failures,
        );
    }
    all_done(failures);
}

#[test]
fn valid_repeat_program_and_nonzero_simple_residual_keep_literal_square() {
    let mut failures = Vec::new();
    for (label, flags, residual) in [
        ("repeat3 complete program", &[0x09, 3][..], &[][..]),
        ("repeat0 fits F4=N4", &[0x09, 0, 0x09, 2][..], &[][..]),
        (
            "unused residual is not another source extent",
            &[0x09, 3][..],
            &[0xff, 0xa5, 0x39, 0xff][..],
        ),
    ] {
        let bytes = lsb(
            &replace_glyphs(&[(X as u16, simple_square(flags, residual))]),
            X as u16,
            0,
        );
        expect_ink(
            label,
            &bytes,
            &[at_origin()],
            Rect::try_new(0.0, -100.0, 100.0, 100.0).unwrap(),
            5,
            &mut failures,
        );
    }
    all_done(failures);
}

#[test]
fn composite_children_share_raw_and_provider_qualification_after_a_valid_glyph() {
    let mut failures = Vec::new();
    let mut empty = header(0, [0, 0, 0, 0]);
    empty.extend(1u16.to_be_bytes());
    empty.push(0xff);
    for (label, data, expected) in [
        (
            "child excess repeat",
            malformed_repeat(),
            GlyphPreparationError::InvalidFont,
        ),
        (
            "child missing zero-contour program",
            missing_zero_contour_program(),
            GlyphPreparationError::InvalidFont,
        ),
        (
            "child one-point repeat0",
            one_point_repeat_zero(),
            GlyphPreparationError::Provider {
                glyph: u32::from(INNER),
            },
        ),
        (
            "child underconsumed flags",
            underconsumed_flags(),
            GlyphPreparationError::Provider {
                glyph: u32::from(INNER),
            },
        ),
    ] {
        let bbox = [
            i16::from_be_bytes(data[2..4].try_into().unwrap()),
            i16::from_be_bytes(data[4..6].try_into().unwrap()),
            i16::from_be_bytes(data[6..8].try_into().unwrap()),
            i16::from_be_bytes(data[8..10].try_into().unwrap()),
        ];
        let root = composite(bbox, &[(3, INNER, 0, 0)], &[]);
        let bytes = lsb(
            &replace_glyphs(&[(INNER, data), (X as u16, root)]),
            X as u16,
            0,
        );
        expect_error(
            label,
            &bytes,
            &[
                TextGlyph::try_new(u32::from(A), 0.0, 0.0, 0.0).unwrap(),
                at_origin(),
            ],
            expected,
            &mut failures,
        );
    }
    // Both direct and composite zero-contour success require a complete program.
    for (label, bytes) in [
        (
            "direct complete empty program",
            replace_glyphs(&[(X as u16, empty.clone())]),
        ),
        (
            "composite complete empty program",
            replace_glyphs(&[
                (INNER, empty),
                (X as u16, composite([0, 0, 0, 0], &[(3, INNER, 0, 0)], &[])),
            ]),
        ),
        (
            "loca-empty component",
            replace_glyphs(&[(X as u16, composite([0, 0, 0, 0], &[(3, 3, 0, 0)], &[]))]),
        ),
    ] {
        match font(&bytes).prepare_glyph_run(
            &instance(1000.0, vec![], 0.0),
            &[at_origin()],
            limits(),
        ) {
            Ok(prepared) => {
                assert_eq!(
                    prepared.bounds().kind(),
                    TextRunBoundsKind::Empty,
                    "{label}"
                );
                assert_eq!(prepared.path_elements().count(), 0);
            }
            Err(error) => {
                eprintln!("{label}: {:?}", diagnostic(&error));
                failures.push(format!("{label}: {:?}", diagnostic(&error)));
            }
        }
    }
    all_done(failures);
}

#[test]
fn unsupported_component_controls_and_whole_run_limit_retry_preserve_inputs() {
    let unsupported = GlyphPreparationError::UnsupportedOutline {
        capability: GlyphCapability::CompositeGlyph,
    };
    let mut failures = Vec::new();
    for (label, flags, tail) in [
        ("nonidentity scale", 0x000b, vec![0x20, 0]),
        ("point attachment", 0x0001, vec![]),
        ("metric override", 0x0203, vec![]),
    ] {
        let bytes = replace_glyphs(&[(
            X as u16,
            composite([0, -200, 1000, 800], &[(flags, A, 0, 0)], &tail),
        )]);
        expect_error(
            label,
            &bytes,
            &[
                TextGlyph::try_new(u32::from(A), 0.0, 0.0, 0.0).unwrap(),
                at_origin(),
            ],
            unsupported,
            &mut failures,
        );
    }
    let bytes = replace_glyphs(&[(
        X as u16,
        composite([0, -200, 1000, 800], &[(3, A, 0, 0)], &[]),
    )]);
    let font = font(&bytes);
    let kept_font = font.clone();
    let instance = instance(1000.0, vec![], 0.0);
    let kept_instance = instance.clone();
    let glyphs = [glyph()];
    let kept_glyphs = glyphs;
    for (resource, low) in [
        (
            GlyphResource::PathElements,
            GlyphRunLimits::new(16, 0, 1 << 20, 1 << 20),
        ),
        (
            GlyphResource::ScratchBytes,
            GlyphRunLimits::new(16, 1024, 0, 1 << 20),
        ),
        (
            GlyphResource::RetainedBytes,
            GlyphRunLimits::new(16, 1024, 1 << 20, 0),
        ),
    ] {
        match font.prepare_glyph_run(&instance, &glyphs, low) {
            Err(error) if matches!(diagnostic(&error),GlyphPreparationError::Limit{resource:actual,..} if actual==resource) =>
                {}
            Err(error) => failures.push(format!("limit {resource:?}: {:?}", diagnostic(&error))),
            Ok(_) => failures.push(format!("limit {resource:?}: unexpected preparation")),
        }
        assert_eq!(font, kept_font);
        assert_eq!(instance, kept_instance);
        assert_eq!(glyphs, kept_glyphs);
    }
    // Same objects, raised limits, actual original composite facts retained.
    match font.prepare_glyph_run(&instance, &glyphs, limits()) {
        Ok(prepared) => {
            assert_eq!(
                prepared.bounds().ink_rect(),
                Some(Rect::try_new(-20.0, -770.0, 1000.0, 1000.0).unwrap())
            );
            assert_eq!(prepared.path_elements().count(), 5);
            assert_eq!(prepared.font(), &kept_font);
            assert_eq!(prepared.instance(), &kept_instance);
            assert_eq!(prepared.glyphs(), kept_glyphs);
        }
        Err(error) => failures.push(format!(
            "same-input raised-limit retry: {:?}",
            diagnostic(&error)
        )),
    }
    all_done(failures);
}
