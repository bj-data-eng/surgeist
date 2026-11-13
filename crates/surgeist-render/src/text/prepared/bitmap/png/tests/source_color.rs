use super::*;
use crate::{
    RenderAccountingError, RenderAccountingOptions, RenderAttemptStatus, RenderClock, RenderLimits,
    RenderPhaseFilter, RenderProfile, RenderQuantity,
};
use surgeist_ledger::Dimension;

const SRGB: [u32; 8] = [31270, 32900, 64000, 33000, 30000, 60000, 15000, 6000];
const SWAPPED: [u32; 8] = [31270, 32900, 15000, 6000, 30000, 60000, 64000, 33000];
const AXES: [u32; 8] = [13890, 3425, 100000, 0, 0, 100000, 0, 0];
fn chromaticities(values: [u32; 8]) -> Vec<u8> {
    values.into_iter().flat_map(u32::to_be_bytes).collect()
}
fn image_for(pixels: [u8; 8], chunks: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
    let mut filtered = vec![0];
    filtered.extend(pixels);
    color_image(&filtered, chunks)
}
fn assert_pixels(bytes: &[u8], expected: [u8; 8]) {
    assert_eq!(test_decode(bytes, 1 << 20).unwrap().pixels, expected);
}
fn released(context: &RenderAccountingContext) {
    let report = context.last_attempt().unwrap();
    assert_eq!(report.cleanup_fault(), None);
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
}

#[test]
fn power_gamma_transforms_hidden_rgb_but_never_alpha() {
    for (a, b) in [(0, 1), (128, 255)] {
        assert_pixels(
            &image_for(
                [128, 128, 128, a, 128, 128, 128, b],
                &[(b"gAMA", &100000_u32.to_be_bytes())],
            ),
            [188, 188, 188, a, 188, 188, 188, b],
        );
    }
}

#[test]
fn gamma_zero_is_ignored_and_missing_gamma_uses_the_piecewise_srgb_curve() {
    let source = [12, 128, 240, 0, 255, 0, 0, 128];
    assert_pixels(
        &image_for(source, &[(b"gAMA", &0_u32.to_be_bytes())]),
        source,
    );
    for gamma in [None, Some(0_u32)] {
        let chrm = chromaticities(SWAPPED);
        let mut chunks = vec![(b"cHRM", chrm.as_slice())];
        let payload = gamma.map(u32::to_be_bytes);
        if let Some(value) = payload.as_ref() {
            chunks.push((b"gAMA", value));
        }
        assert_pixels(
            &image_for(source, &chunks),
            [240, 128, 12, 0, 0, 0, 255, 128],
        );
    }
    // sRGB encode(pow(sample/255,100000/45455)), independently rounded.
    // Dark values distinguish this actual power law from sRGB's linear toe.
    assert_pixels(
        &image_for(
            [8, 16, 32, 128, 64, 128, 192, 1],
            &[(b"gAMA", &45455_u32.to_be_bytes())],
        ),
        [2, 7, 26, 128, 62, 129, 193, 1],
    );
}

#[test]
fn calibrated_half_profiles_compose_in_either_legal_chunk_order() {
    let chrm = chromaticities(SWAPPED);
    let gamma = 100000_u32.to_be_bytes();
    for chunks in [
        [(b"cHRM", chrm.as_slice()), (b"gAMA", gamma.as_slice())],
        [(b"gAMA", gamma.as_slice()), (b"cHRM", chrm.as_slice())],
    ] {
        assert_pixels(
            &image_for([128, 64, 192, 128, 255, 0, 0, 64], &chunks),
            [225, 137, 188, 128, 0, 0, 255, 64],
        );
    }
}

#[test]
fn d50_adaptation_uses_actual_source_white_and_nonneutral_calibration() {
    let mut xy = SRGB;
    xy[..2].copy_from_slice(&[34570, 35850]);
    let xy = chromaticities(xy);
    let chunks = [
        (b"cHRM", xy.as_slice()),
        (b"gAMA", &100000_u32.to_be_bytes()),
    ];
    // Independent exact-rational primary calibration/Bradford plus60-digit
    // destination transfer yields encoded[.7496152454,.5370354598,.8735554754].
    assert_pixels(
        &image_for([128, 64, 192, 128, 255, 255, 255, 1], &chunks),
        [191, 137, 223, 128, 255, 255, 255, 1],
    );
    assert_pixels(
        &image_for([128, 128, 128, 0, 0, 0, 0, 255], &chunks),
        [188, 188, 188, 0, 0, 0, 0, 255],
    );
}

#[test]
fn general_bradford_adaptation_preserves_nonneutral_ink_and_terminal_clipping() {
    let mut xy = SRGB;
    xy[..2].copy_from_slice(&[33333, 33333]);
    let xy = chromaticities(xy);
    let chunks = [
        (b"cHRM", xy.as_slice()),
        (b"gAMA", &100000_u32.to_be_bytes()),
    ];
    // Independent Fraction Gaussian calibration and Bradford, then Decimal
    // sRGB encoding: red is1.05325476019 before the selected terminal clip.
    assert_pixels(
        &image_for([128, 64, 192, 128, 255, 0, 0, 255], &chunks),
        [192, 138, 224, 128, 255, 21, 11, 255],
    );
    assert_pixels(
        &image_for([0, 255, 0, 64, 0, 0, 255, 0], &chunks),
        [0, 254, 18, 64, 0, 0, 254, 0],
    );
}

#[test]
fn rec2020_outside_destination_gamut_is_channel_clipped_not_css_minde_mapped() {
    let xy = chromaticities([31270, 32900, 70800, 29200, 17000, 79700, 13100, 4600]);
    assert_pixels(
        &image_for(
            [255, 0, 0, 128, 255, 0, 0, 0],
            &[(b"cHRM", &xy), (b"gAMA", &100000_u32.to_be_bytes())],
        ),
        [255, 0, 0, 128, 255, 0, 0, 0],
    );
}

#[test]
fn exact_zero_y_boundary_and_near_collinear_profiles_are_not_epsilon_rejected() {
    // XYZ-axis primaries and white at green: red/blue have exactly zero
    // energy; the selected Bradford has a negative, nonzero S-cone response.
    let xy = chromaticities([0, 100000, 100000, 0, 0, 100000, 0, 0]);
    assert_pixels(
        &image_for(
            [255, 0, 0, 128, 0, 255, 0, 64],
            &[(b"cHRM", &xy), (b"gAMA", &100000_u32.to_be_bytes())],
        ),
        [0, 0, 0, 128, 255, 255, 255, 64],
    );
    // One-unit xy triangle, with white exactly at its red vertex. Its exact
    // nonzero determinant must survive despite large cancelling products.
    let xy = chromaticities([31270, 32900, 31270, 32900, 31271, 32900, 31270, 32901]);
    assert_pixels(
        &image_for(
            [255, 0, 0, 128, 0, 255, 0, 64],
            &[(b"cHRM", &xy), (b"gAMA", &100000_u32.to_be_bytes())],
        ),
        [255, 255, 255, 128, 0, 0, 0, 64],
    );
}

#[test]
fn extreme_legal_gamma_stays_finite_and_uses_its_actual_exponent() {
    assert_pixels(
        &image_for(
            [254, 128, 1, 0, 255, 255, 255, 128],
            &[(b"gAMA", &1_u32.to_be_bytes())],
        ),
        [0, 0, 0, 0, 255, 255, 255, 128],
    );
    assert_pixels(
        &image_for(
            [0, 1, 128, 1, 1, 254, 255, 255],
            &[(b"gAMA", &0x7fff_ffff_u32.to_be_bytes())],
        ),
        [0, 255, 255, 1, 255, 255, 255, 255],
    );
}

#[test]
fn invalid_identification_selects_full_srgb_fallback_without_a_lower_profile() {
    let source = [128, 64, 192, 0, 8, 16, 32, 128];
    let mut invalids = vec![
        [31270, 0, 64000, 33000, 30000, 60000, 15000, 6000],
        [90000, 90000, 64000, 33000, 30000, 60000, 15000, 6000],
        [10000, 10000, 64000, 33000, 30000, 60000, 15000, 6000],
        [31270, 32900, 31270, 32900, 31271, 32900, 31272, 32900],
    ];
    let mut coincident = SRGB;
    coincident[4..6].copy_from_slice(&[64000, 33000]);
    invalids.push(coincident);
    for values in invalids {
        assert_pixels(
            &image_for(
                source,
                &[
                    (b"gAMA", &100000_u32.to_be_bytes()),
                    (b"cHRM", &chromaticities(values)),
                ],
            ),
            source,
        );
    }
    for intent in [4, 255] {
        assert_pixels(
            &image_for(
                source,
                &[
                    (b"gAMA", &100000_u32.to_be_bytes()),
                    (b"cHRM", &chromaticities(SWAPPED)),
                    (b"sRGB", &[intent]),
                ],
            ),
            source,
        );
    }
}

#[test]
fn explicit_srgb_bypasses_semantically_impossible_lower_hints_without_calibration() {
    let source = [128, 64, 192, 0, 8, 16, 32, 128];
    for hint in [[90000; 8], AXES] {
        for intent in 0..=3 {
            let bytes = image_for(
                source,
                &[
                    (b"gAMA", &1_u32.to_be_bytes()),
                    (b"cHRM", &chromaticities(hint)),
                    (b"sRGB", &[intent]),
                ],
            );
            let mut context = RenderAccountingContext::default();
            assert_eq!(
                decode_in(&bytes, 1 << 20, &mut context).unwrap().pixels,
                source
            );
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .phase_usage(RenderPhase::PngColorPrepare)
                    .work(RenderWork::CoordinateVisits),
                1
            );
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .phase_usage(RenderPhase::PngColorConvert)
                    .work(RenderWork::CoordinateVisits),
                0
            );
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::ImageTexelVisits),
                0
            );
        }
    }
}

#[test]
fn color_chunk_lengths_cardinality_location_integer_range_and_crc_remain_malformed() {
    let source = [128; 8];
    let gamma = 100000_u32.to_be_bytes();
    let xy = chromaticities(SRGB);
    let cases = vec![
        image_for(source, &[(b"gAMA", &gamma), (b"gAMA", &gamma)]),
        image_for(source, &[(b"sRGB", &[0]), (b"sRGB", &[1])]),
        image_for(source, &[(b"cHRM", &xy), (b"cHRM", &xy)]),
        image_for(source, &[(b"gAMA", &[])]),
        image_for(source, &[(b"sRGB", &[0, 1])]),
        image_for(source, &[(b"cHRM", &xy[..31])]),
        image_for(
            source,
            &[(b"gAMA", &0x8000_0000_u32.to_be_bytes()), (b"sRGB", &[0])],
        ),
        image_for(
            source,
            &[
                (b"cHRM", &chromaticities([0x8000_0000; 8])),
                (b"sRGB", &[0]),
            ],
        ),
    ];
    for bytes in cases {
        assert_invalid(&bytes);
    }
    for (kind, payload) in [
        (b"gAMA", gamma.as_slice()),
        (b"sRGB", &[0]),
        (b"cHRM", xy.as_slice()),
    ] {
        let plain = image_for(source, &[]);
        let end = plain.len() - 12;
        let mut misplaced = plain[..end].to_vec();
        append(&mut misplaced, kind, payload);
        misplaced.extend_from_slice(&plain[end..]);
        assert_invalid(&misplaced);
    }
    let mut bad_crc = image_for(source, &[(b"sRGB", &[0]), (b"gAMA", &gamma)]);
    bad_crc[41] ^= 1;
    assert_invalid(&bad_crc);
}
fn assert_invalid(bytes: &[u8]) {
    assert_eq!(
        test_decode(bytes, 1 << 20)
            .err()
            .unwrap()
            .glyph_preparation_diagnostic(),
        Some(&GlyphPreparationError::InvalidFont)
    );
}

#[test]
fn unsupported_higher_profiles_never_silently_defer_to_supported_tags() {
    for (kind, data) in [
        (b"iCCP", [0; 4]),
        (b"cICP", [1, 13, 0, 1]),
        (b"mDCV", [0; 4]),
        (b"cLLI", [0; 4]),
    ] {
        let bytes = image_for(
            [128; 8],
            &[
                (b"sRGB", &[0]),
                (b"gAMA", &100000_u32.to_be_bytes()),
                (kind, &data),
            ],
        );
        assert_eq!(
            test_decode(&bytes, 1 << 20)
                .err()
                .unwrap()
                .glyph_preparation_diagnostic(),
            Some(&GlyphPreparationError::UnsupportedPng)
        );
    }
}

#[test]
fn exact_zero_bradford_cone_is_unsupported_after_full_png_validation() {
    let bytes = image_for(
        [128; 8],
        &[
            (b"cHRM", &chromaticities(AXES)),
            (b"gAMA", &100000_u32.to_be_bytes()),
        ],
    );
    let mut context = RenderAccountingContext::default();
    for _ in 0..2 {
        let error = decode_in(&bytes, 1 << 20, &mut context).err().unwrap();
        assert_eq!(
            error.glyph_preparation_diagnostic(),
            Some(&GlyphPreparationError::UnsupportedPng)
        );
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::ImageTexelVisits),
            0
        );
        released(&context);
    }
    let mut invalid = bytes.clone();
    *invalid.last_mut().unwrap() ^= 1;
    assert_invalid(&invalid);
}

#[test]
fn color_work_and_numeric_workspace_refusal_release_unpublished_pixels_and_retry() {
    let mut xy = SRGB;
    xy[..2].copy_from_slice(&[33333, 33333]);
    let bytes = image_for(
        [128, 64, 192, 0, 255, 0, 0, 128],
        &[
            (b"cHRM", &chromaticities(xy)),
            (b"gAMA", &100000_u32.to_be_bytes()),
        ],
    );
    let expected = [192, 138, 224, 0, 255, 21, 11, 128];
    let original = bytes.clone();
    let detailed = RenderProfile::Detailed {
        timing: false,
        event_limit: 8192,
        filter: RenderPhaseFilter::All,
    };
    let mut context =
        RenderAccountingContext::new(RenderAccountingOptions::new().with_profile(detailed));
    assert_eq!(
        decode_in(&bytes, 1 << 20, &mut context).unwrap().pixels,
        expected
    );
    let report = context.last_attempt().unwrap().attempt().unwrap();
    let scratch = report.usage().peak(RenderResource::ScratchCpuBytes);
    // Fixed numeric workspace is acquired before inflate. CPU custody's actual
    // origin is Preparation, distinct from a central cap's refusing ancestor.
    let (origin, setup) = report
        .events()
        .find_map(|event| match event {
            crate::RenderEvent::Change {
                phase,
                quantity: RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
                amount,
                kind: crate::RenderChangeKind::Acquire,
                ..
            } => Some((phase, amount)),
            _ => None,
        })
        .unwrap();
    assert_eq!(origin, RenderPhase::Preparation);
    let coordinates = report.usage().work(RenderWork::CoordinateVisits);
    let convert_coordinates = report
        .phase_usage(RenderPhase::PngColorConvert)
        .work(RenderWork::CoordinateVisits);
    assert_eq!(report.usage().work(RenderWork::ImageTexelVisits), 2);
    assert!(setup > 0);
    let filtered = [0, 128, 64, 192, 0, 255, 0, 0, 128];
    let preview = setup
        + u64::try_from(
            size_of::<DecompressorOxide>()
                + size_of::<[u16; 16]>()
                + size_of::<[u32; 17]>()
                + zlib(&filtered).len()
                + filtered.len(),
        )
        .unwrap();
    for (case, limits, quantity, origin, texels, event_expected) in [
        (
            "numeric",
            RenderLimits::new().with_scratch_bytes(setup - 1),
            RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
            RenderPhase::Preparation,
            0,
            Some(true),
        ),
        (
            "preview",
            RenderLimits::new().with_scratch_bytes(preview - 1),
            RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
            RenderPhase::Preparation,
            0,
            Some(false),
        ),
        // Actual capacities may exceed requested lengths; this boundary may
        // therefore refuse either the preview or a later actual acquisition.
        (
            "capacity",
            RenderLimits::new().with_scratch_bytes(scratch - 1),
            RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
            RenderPhase::Preparation,
            0,
            None,
        ),
        (
            "texel-zero",
            RenderLimits::new()
                .with_work_limit(RenderWork::ImageTexelVisits, 0)
                .unwrap(),
            RenderQuantity::Work(RenderWork::ImageTexelVisits),
            RenderPhase::PngColorConvert,
            0,
            Some(true),
        ),
        (
            "texel-one",
            RenderLimits::new()
                .with_work_limit(RenderWork::ImageTexelVisits, 1)
                .unwrap(),
            RenderQuantity::Work(RenderWork::ImageTexelVisits),
            RenderPhase::PngColorConvert,
            1,
            Some(true),
        ),
        (
            "coordinate-last",
            RenderLimits::new()
                .with_work_limit(RenderWork::CoordinateVisits, coordinates - 1)
                .unwrap(),
            RenderQuantity::Work(RenderWork::CoordinateVisits),
            RenderPhase::PngColorConvert,
            2,
            Some(true),
        ),
    ] {
        context.set_options(
            RenderAccountingOptions::new()
                .with_limits(limits)
                .with_profile(detailed),
        );
        let error = decode_in(&bytes, 1 << 20, &mut context).err().unwrap();
        let attempt = context.last_attempt().unwrap().attempt().unwrap();
        let refusal = attempt.refusal().unwrap();
        assert_eq!(refusal.quantity(), quantity);
        assert_eq!(refusal.phase(), RenderPhase::Attempt);
        assert_eq!(attempt.dropped_events(), 0);
        let refusals: Vec<_> = attempt
            .events()
            .filter_map(|event| match event {
                crate::RenderEvent::Refused { phase, .. } => Some(phase),
                _ => None,
            })
            .collect();
        if let Some(expected) = event_expected {
            assert_eq!(refusals.len(), usize::from(expected), "{case}: {refusal:?}");
        }
        assert!(refusals.iter().all(|phase| *phase == origin), "{case}");
        if case == "numeric" {
            assert_eq!((refusal.current(), refusal.amount()), (0, setup));
            assert_eq!(attempt.usage().peak(RenderResource::ScratchCpuBytes), 0);
        }
        if case == "preview" {
            assert_eq!(
                (refusal.current(), refusal.amount()),
                (setup, preview - setup)
            );
        }
        assert_eq!(attempt.usage().work(RenderWork::ImageTexelVisits), texels);
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Refused(refusal))
        );
        if quantity == RenderQuantity::Work(RenderWork::ImageTexelVisits) {
            assert_eq!((refusal.current(), refusal.amount()), (texels, 1));
            if texels == 0 {
                assert_eq!(
                    attempt
                        .phase_usage(RenderPhase::PngColorConvert)
                        .work(RenderWork::CoordinateVisits),
                    0
                );
            }
        }
        if quantity == RenderQuantity::Work(RenderWork::CoordinateVisits) {
            assert_eq!((refusal.current(), refusal.amount()), (coordinates - 1, 1));
            assert_eq!(
                attempt.usage().work(RenderWork::CoordinateVisits),
                coordinates - 1
            );
            assert_eq!(
                attempt
                    .phase_usage(RenderPhase::PngColorConvert)
                    .work(RenderWork::CoordinateVisits),
                convert_coordinates - 1
            );
        }
        assert_eq!(
            context.last_attempt().unwrap().status(),
            RenderAttemptStatus::Refused
        );
        released(&context);
        assert_eq!(bytes, original);
        context.set_options(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_scratch_bytes(scratch)
                    .with_work_limit(RenderWork::CoordinateVisits, coordinates)
                    .unwrap()
                    .with_work_limit(RenderWork::ImageTexelVisits, 2)
                    .unwrap(),
            ),
        );
        assert_eq!(
            decode_in(&bytes, 1 << 20, &mut context).unwrap().pixels,
            expected
        );
        released(&context);
    }
}

#[test]
fn color_phases_are_truthful_and_profiles_do_not_change_reached_work_or_custody() {
    fn no_clock() -> std::time::Duration {
        panic!("untimed color decode must not read a clock")
    }
    fn clock() -> std::time::Duration {
        std::time::Duration::ZERO
    }
    let bytes = image_for([128; 8], &[(b"gAMA", &100000_u32.to_be_bytes())]);
    let mut baseline = None;
    for (profile, timed) in [
        (RenderProfile::Off, false),
        (RenderProfile::Summary { timing: false }, false),
        (
            RenderProfile::Detailed {
                timing: false,
                event_limit: 0,
                filter: RenderPhaseFilter::All,
            },
            false,
        ),
        (RenderProfile::Summary { timing: true }, true),
    ] {
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new()
                .with_profile(profile)
                .with_clock(RenderClock::Host(if timed { clock } else { no_clock })),
        );
        assert_eq!(
            decode_in(&bytes, 1 << 20, &mut context).unwrap().pixels,
            [188, 188, 188, 128, 188, 188, 188, 128]
        );
        let report = context.last_attempt().unwrap().attempt().unwrap();
        let work: Vec<_> = RenderWork::ALL
            .iter()
            .map(|&unit| report.usage().work(unit))
            .collect();
        let scratch = report.usage().peak(RenderResource::ScratchCpuBytes);
        if let Some((prior, peak)) = &baseline {
            assert_eq!(&work, prior);
            assert_eq!(scratch, *peak);
        } else {
            baseline = Some((work, scratch));
        }
        assert!(
            report
                .phase_usage(RenderPhase::PngColorPrepare)
                .work(RenderWork::CoordinateVisits)
                > 0
        );
        assert_eq!(
            report
                .phase_usage(RenderPhase::PngColorConvert)
                .work(RenderWork::ImageTexelVisits),
            2
        );
        assert_eq!(
            report.phase_elapsed(RenderPhase::PngColorPrepare).is_some(),
            timed
        );
        assert_eq!(
            report.phase_elapsed(RenderPhase::PngColorConvert).is_some(),
            timed
        );
        released(&context);
    }
    let mut context = RenderAccountingContext::new(
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
    );
    decode_in(
        &image_for([128; 8], &[(b"sRGB", &[0])]),
        1 << 20,
        &mut context,
    )
    .unwrap();
    let report = context.last_attempt().unwrap().attempt().unwrap();
    assert_eq!(report.phase_count(RenderPhase::PngColorPrepare), 1);
    assert_eq!(report.phase_count(RenderPhase::PngColorConvert), 0);
}
