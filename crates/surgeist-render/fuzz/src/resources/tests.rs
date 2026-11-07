use super::*;
use surgeist_fuzz::OutcomeKind;
use surgeist_render::{
    FontData, GlyphSynthesis, PathElement, Rect, RenderAccountingContext, TextGlyph, TextInstance,
    TextRunBoundsKind,
};

#[test]
fn exact_ahem_x_has_literal_reflected_rectangle_and_preserves_negative_advance() {
    // Independent source facts: Ahem UPEM1000, X58=(0,-200)..(1000,800).
    let font = FontData::try_from_bytes(
        AHEM.to_vec(),
        0,
        surgeist_font::DecodeLimits::new(32768, 32768, 32768, 256, 16, 1 << 20, 1 << 20),
    )
    .unwrap();
    let instance = TextInstance::try_new(1000.0, vec![], GlyphSynthesis::none()).unwrap();
    let glyphs = [TextGlyph::try_new(58, -20.0, 30.0, -123.0).unwrap()];
    let run = font
        .prepare_glyph_run(
            &instance,
            &glyphs,
            surgeist_render::GlyphRunLimits::new(16, 1024, 1 << 20, 1 << 20),
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
    assert_eq!(AHEM.len(), 22572);
    assert_eq!(
        run.bounds().ink_rect(),
        Some(Rect::try_new(-20.0, -770.0, 1000.0, 1000.0).unwrap())
    );
    assert_eq!(run.glyphs()[0].advance(), -123.0);
    let path = run.path_elements().collect::<Vec<_>>();
    assert_eq!(path.len(), 5);
    assert!(matches!(path[0],PathElement::MoveTo(p) if p.x()==-20.0 && p.y()==-770.0));
    assert!(matches!(path[1],PathElement::LineTo(p) if p.x()==980.0 && p.y()==-770.0));
    assert!(matches!(path[2],PathElement::LineTo(p) if p.x()==980.0 && p.y()==230.0));
    assert!(matches!(path[3],PathElement::LineTo(p) if p.x()==-20.0 && p.y()==230.0));
    assert!(matches!(path[4], PathElement::Close));
}

#[test]
fn exact_ahem_space_is_empty_and_skew_precedes_y_reflection_and_placement() {
    let target = ResourceTarget::new();
    for (literal, expected) in [(5, None), (6, Some((-120.0, -770.0, 1500.0, 1000.0)))] {
        let case = target.literal(literal).unwrap();
        let Operation::Glyph(g) = case.operation else {
            panic!("glyph literal");
        };
        let font = FontData::try_from_bytes(
            g.font.bytes,
            0,
            surgeist_font::DecodeLimits::new(32768, 32768, 32768, 256, 16, 1 << 20, 1 << 20),
        )
        .unwrap();
        let instance = TextInstance::try_new(
            g.size,
            vec![],
            GlyphSynthesis::try_new(g.skew, g.embolden).unwrap(),
        )
        .unwrap();
        let glyphs: Vec<_> = g
            .glyphs
            .iter()
            .map(|v| TextGlyph::try_new(v.id, v.x, v.y, v.advance).unwrap())
            .collect();
        let run = font
            .prepare_glyph_run(
                &instance,
                &glyphs,
                surgeist_render::GlyphRunLimits::new(16, 1024, 1 << 20, 1 << 20),
                &mut RenderAccountingContext::default(),
            )
            .unwrap();
        if let Some((x, y, w, h)) = expected {
            assert_eq!(
                run.bounds().ink_rect(),
                Some(Rect::try_new(x, y, w, h).unwrap())
            );
            assert!(
                matches!(run.path_elements().next(),Some(PathElement::MoveTo(p)) if p.x()==380.0 && p.y()==-770.0)
            );
        } else {
            assert_eq!(run.bounds().kind(), TextRunBoundsKind::Empty);
            assert_eq!(run.path_elements().count(), 0);
        }
    }
}

#[test]
fn literals_report_only_exercised_properties_and_replay_the_same_outcome() {
    let target = ResourceTarget::new();
    for literal in 0..24 {
        let case = target.literal(literal).unwrap();
        let bytes = target.encode(&case).unwrap();
        let decoded = target.decode(&bytes).unwrap();
        assert_eq!(target.encode(&decoded).unwrap(), bytes);
        let actual = target.evaluate(&case).unwrap();
        assert_eq!(
            target.evaluate(&decoded).unwrap(),
            actual,
            "literal {literal}"
        );
        let success = matches!(literal, 0 | 4 | 5 | 6 | 11 | 14..=18 | 21);
        assert_eq!(
            actual.kind(),
            if success {
                OutcomeKind::Evaluated
            } else {
                OutcomeKind::ExpectedRejection
            },
            "literal {literal}: {actual:?}"
        );
        let expected: &[&str] = match literal {
            0..=3 => &[FONT],
            11..=13 | 22 => &[IMAGE],
            14..=18 => &[FONT, GLYPH, RETRY],
            _ => &[FONT, GLYPH],
        };
        assert_eq!(
            actual
                .checked_properties()
                .iter()
                .map(|p| p.as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }
    assert!(target.literal(24).is_err());
}

#[test]
fn rgba_wire_has_literal_little_endian_dimensions_and_exact_payload() {
    // Revision1 wire: magic, revision, Image tag, f64 width/height,
    // u32 byte count, exact owned RGBA bytes. No entropy or fixture name.
    let mut wire = b"SRFR\x01\x02".to_vec();
    wire.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0xf0, 0x3f]);
    wire.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0xf0, 0x3f]);
    wire.extend_from_slice(&[4, 0, 0, 0, 17, 23, 41, 255]);
    let target = ResourceTarget::new();
    let case = target.decode(&wire).unwrap();
    assert_eq!(target.encode(&case).unwrap(), wire);
    assert_eq!(
        target.evaluate(&case).unwrap().kind(),
        OutcomeKind::Evaluated
    );
    assert!(target.decode(b"SRFS\x01\x02").is_err());
}

#[test]
fn malformed_tags_revision_truncation_trailing_bytes_and_counts_refuse() {
    let target = ResourceTarget::new();
    for wire in [
        b"SRFR\x02".as_slice(),
        b"SRFR\x01\xff",
        b"SRFR\x01\x03\xff",
        b"SRFR\x01\x00",
    ] {
        assert!(target.decode(wire).is_err());
    }
    let glyph = target.encode(&target.literal(4).unwrap()).unwrap();
    for cut in [0, 1, 4, 5, 6, 37, 100, glyph.len() - 1] {
        assert!(target.decode(&glyph[..cut]).is_err());
    }
    let mut trailing = glyph.clone();
    trailing.push(0);
    assert!(target.decode(&trailing).is_err());
    // Font count is after header/tag, index and seven u32 limits.
    let mut font_count = glyph.clone();
    font_count[38..42].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(target.decode(&font_count).is_err());
    let coordinate_offset = 42 + AHEM.len() + 12;
    let mut coordinate_count = glyph.clone();
    coordinate_count[coordinate_offset..coordinate_offset + 4].copy_from_slice(&9u32.to_le_bytes());
    assert!(target.decode(&coordinate_count).is_err());
    let glyph_offset = coordinate_offset + 4;
    let mut glyph_count = glyph;
    glyph_count[glyph_offset..glyph_offset + 4].copy_from_slice(&17u32.to_le_bytes());
    assert!(target.decode(&glyph_count).is_err());
    let mut image = b"SRFR\x01\x02".to_vec();
    image.extend_from_slice(&0f64.to_le_bytes());
    image.extend_from_slice(&0f64.to_le_bytes());
    image.extend_from_slice(&4097u32.to_le_bytes());
    assert!(target.decode(&image).is_err());
}

#[test]
fn exact_ahem_oracles_refuse_a_different_font_even_when_transport_is_valid() {
    let target = ResourceTarget::new();
    let mut wire = target.encode(&target.literal(4).unwrap()).unwrap();
    // Same length and SFNT header: nevertheless not the exact known fixture.
    wire[42 + 100] ^= 1;
    assert!(target.decode(&wire).is_err());
    let mut case = target.literal(4).unwrap();
    let Operation::Glyph(g) = &mut case.operation else {
        panic!("glyph");
    };
    g.font.bytes[100] ^= 1;
    assert!(target.encode(&case).is_err());
}

#[test]
fn numeric_transport_preserves_nan_payloads_infinities_subnormals_and_signed_zero() {
    let target = ResourceTarget::new();
    for single in [0, 0x8000_0000, 1, 0x7f80_0000, 0xff80_0000, 0x7fc0_1234] {
        for double in [
            0,
            0x8000_0000_0000_0000,
            1,
            0x7ff0_0000_0000_0000,
            0x7ff8_0000_0000_1234,
        ] {
            let mut case = target.literal(4).unwrap();
            let Operation::Glyph(g) = &mut case.operation else {
                panic!("glyph");
            };
            g.size = f32::from_bits(single);
            g.skew = f32::from_bits(single);
            g.embolden = f32::from_bits(single);
            g.glyphs[0].x = f64::from_bits(double);
            g.glyphs[0].y = f64::from_bits(double);
            g.glyphs[0].advance = f32::from_bits(single);
            let bytes = target.encode(&case).unwrap();
            let decoded = target.decode(&bytes).unwrap();
            let Operation::Glyph(g) = &decoded.operation else {
                panic!("glyph");
            };
            assert_eq!(g.size.to_bits(), single);
            assert_eq!(g.skew.to_bits(), single);
            assert_eq!(g.embolden.to_bits(), single);
            assert_eq!(g.glyphs[0].x.to_bits(), double);
            assert_eq!(g.glyphs[0].y.to_bits(), double);
            assert_eq!(g.glyphs[0].advance.to_bits(), single);
            assert_eq!(target.encode(&decoded).unwrap(), bytes);
        }
    }
}

#[test]
fn bounded_generation_materializes_exact_replay_without_invented_font_ink() {
    let target = ResourceTarget::new();
    for input in [
        vec![],
        vec![0],
        vec![0; 8192],
        vec![1, 6],
        vec![2, 16, 16, 23],
        vec![255; 4096],
    ] {
        let Generation::Generated(case) = target.generate(&input).unwrap() else {
            panic!("finite generation");
        };
        let bytes = target.encode(&case).unwrap();
        let decoded = target.decode(&bytes).unwrap();
        assert_eq!(
            target.evaluate(&case).unwrap(),
            target.evaluate(&decoded).unwrap()
        );
        assert_eq!(bytes, target.encode(&decoded).unwrap());
        if let Operation::Font(f) = &case.operation {
            assert!(f.bytes.len() <= MAX_RANDOM_FONT);
            assert_eq!(
                target
                    .evaluate(&case)
                    .unwrap()
                    .checked_properties()
                    .iter()
                    .map(|v| v.as_str())
                    .collect::<Vec<_>>(),
                [FONT]
            );
        }
    }
}

#[test]
fn harness_caps_reject_before_public_producer_allocation() {
    let target = ResourceTarget::new();
    let mut case = target.literal(0).unwrap();
    let Operation::Font(f) = &mut case.operation else {
        panic!("font");
    };
    f.limits[6] = MAX_PRODUCER_BYTES + 1;
    assert!(target.encode(&case).is_err());
    let too_wide = ResourceCase {
        operation: Operation::Image {
            width: 17.0,
            height: 1.0,
            rgba: vec![],
        },
    };
    assert!(target.encode(&too_wide).is_err());
    let mut case = target.literal(14).unwrap();
    let Operation::Retry { glyph, .. } = &mut case.operation else {
        panic!("retry");
    };
    glyph.limits[0] = 0;
    assert!(target.encode(&case).is_err());
}

#[test]
fn captured_refused_and_raised_limits_drive_replay_without_recalculation() {
    let target = ResourceTarget::new();
    let mut case = target.literal(15).unwrap();
    let Operation::Retry {
        glyph,
        refused_limits,
        refused_path_work,
        raised_path_work,
        ..
    } = &mut case.operation
    else {
        panic!("retry");
    };
    glyph.glyphs.push(glyph.glyphs[0]);
    // Two source-backed rectangles need ten elements. Capture six, rather than
    // rederiving nine from this new input or from a producer report.
    refused_limits[1] = 6;
    assert_eq!(*refused_path_work, u64::MAX);
    assert_eq!(*raised_path_work, u64::MAX);
    let bytes = target.encode(&case).unwrap();
    assert_eq!(&bytes[11..15], &6u32.to_le_bytes());
    let decoded = target.decode(&bytes).unwrap();
    assert_eq!(
        target.evaluate(&decoded).unwrap().kind(),
        OutcomeKind::Evaluated
    );
    let Operation::Retry { refused_limits, .. } = &mut case.operation else {
        panic!("retry");
    };
    refused_limits[1] = 10;
    assert!(target.encode(&case).is_err());
    let mut work_case = target.literal(18).unwrap();
    let Operation::Retry {
        refused_path_work,
        raised_path_work,
        ..
    } = &mut work_case.operation
    else {
        panic!("retry");
    };
    *refused_path_work = 2;
    *raised_path_work = 5;
    let decoded = target.decode(&target.encode(&work_case).unwrap()).unwrap();
    assert_eq!(
        target.evaluate(&decoded).unwrap().kind(),
        OutcomeKind::Evaluated
    );
}

#[test]
fn real_typed_public_errors_become_declared_stable_property_failures() {
    use surgeist_render::{Image, Point, RenderLimits, RenderWork, Size};
    // Exercise the classifier with actual caller-invalid public requests. These
    // are controls for error classification, not injected producer regressions.
    fn check<T, E: std::fmt::Display>(
        result: std::result::Result<T, E>,
        property: &str,
        discriminator: &str,
    ) {
        let Err(outcome) = crate::product_observation(result, property, discriminator).unwrap()
        else {
            panic!("real typed public rejection");
        };
        assert_eq!(outcome.kind(), OutcomeKind::Failure);
        let key = outcome.failure_key().unwrap();
        assert_eq!(key.class(), surgeist_fuzz::FailureClass::PropertyViolation);
        assert_eq!(key.property().as_str(), property);
        assert_eq!(key.discriminator(), Some(discriminator));
        assert!(
            ResourceTarget::new()
                .descriptor()
                .properties()
                .contains(key.property())
        );
    }
    let size = Size::try_new(1.0, 1.0).unwrap();
    check(
        Image::from_rgba(size, vec![0; 3]),
        IMAGE,
        "image-repeat-admission",
    );
    check(
        RenderLimits::new().with_work_limit(RenderWork::PreparedTransferredBytes, 4),
        RETRY,
        "retry-accounting-settings",
    );
    check(Point::try_new(f64::NAN, 0.0), GLYPH, "golden-point");
    check(Rect::try_new(0.0, 0.0, -1.0, 1.0), GLYPH, "golden-bounds");
    let image = crate::product_observation(
        Image::from_rgba(size, vec![17, 23, 41, 255]),
        IMAGE,
        "image-repeat-admission",
    )
    .unwrap()
    .unwrap();
    assert_eq!(image.size(), size);
    let limits = crate::product_observation(
        RenderLimits::new().with_work_limit(RenderWork::PathElementsEmitted, 4),
        RETRY,
        "retry-accounting-settings",
    )
    .unwrap()
    .unwrap();
    assert_eq!(limits.work_limit(RenderWork::PathElementsEmitted), 4);
    let point = crate::product_observation(Point::try_new(-20.0, -770.0), GLYPH, "golden-point")
        .unwrap()
        .unwrap();
    assert_eq!((point.x(), point.y()), (-20.0, -770.0));
    let rect = crate::product_observation(
        Rect::try_new(-20.0, -770.0, 1000.0, 1000.0),
        GLYPH,
        "golden-bounds",
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        (rect.x(), rect.y(), rect.width(), rect.height()),
        (-20.0, -770.0, 1000.0, 1000.0)
    );
}

#[cfg(feature = "property")]
proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 32,
        max_local_rejects: 64,
        max_global_rejects: 64,
        max_shrink_iters: 16,
        max_shrink_time: 0,
        failure_persistence: None,
        rng_seed: proptest::test_runner::RngSeed::Fixed(1155),
        fork: false,
        timeout: 0,
        ..proptest::test_runner::Config::default()
    })]
    #[test]
    fn generated_resource_cases_replay_the_same_canonical_observation(input in surgeist_fuzz::generation::property::bytes(512,ResourceTarget::new().descriptor().limits())) {
        let target=ResourceTarget::new();
        let Generation::Generated(case)=target.generate(&input).unwrap() else {unreachable!("bounded generator");};
        let bytes=target.encode(&case).unwrap();
        let decoded=target.decode(&bytes).unwrap();
        proptest::prop_assert_eq!(target.encode(&decoded).unwrap(),bytes);
        let actual=target.evaluate(&case).unwrap();
        proptest::prop_assert_ne!(actual.kind(),OutcomeKind::Failure,"{:?}",actual);
        proptest::prop_assert_eq!(target.evaluate(&decoded).unwrap(),actual);
    }
}
