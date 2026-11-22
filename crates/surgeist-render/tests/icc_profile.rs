//! Synthetic ICC.1:2022 profiles. Header and fallback tags follow §§7–8 and
//! 10.11. Float matrix coefficients and offsets use the wire order in §10.16.

#![forbid(unsafe_code)]

use surgeist_render::{
    Result,
    accounting::{
        RenderAccountingContext, RenderAccountingError, RenderAccountingOptions, RenderAttemptKind,
        RenderAttemptStatus, RenderLimits, RenderPhase, RenderProfile, RenderQuantity,
        RenderRefusalReason, RenderResource, RenderWork,
    },
    icc::{
        IccPcsColor, IccPcsSpace, IccProfile, IccProfileClass, IccProfileError, IccRenderingIntent,
    },
};

type Tag = (&'static [u8; 4], Vec<u8>);

const D50: [u8; 12] = [0, 0, 0xf6, 0xd6, 0, 1, 0, 0, 0, 0, 0xd3, 0x2d];
const CMYK_SAMPLE: [f64; 4] = [-0.25, 0.5, 1.5, 0.75];
const PERCEPTUAL: IccRenderingIntent = IccRenderingIntent::Perceptual;

// Table 54, one en-US UTF-16BE string.
fn localized(text: &str) -> Vec<u8> {
    let mut data = b"mluc\0\0\0\0\0\0\0\x01\0\0\0\x0cenUS".to_vec();
    let characters: Vec<_> = text.encode_utf16().collect();
    data.extend(u32::try_from(characters.len() * 2).unwrap().to_be_bytes());
    data.extend(28_u32.to_be_bytes());
    for character in characters {
        data.extend(character.to_be_bytes());
    }
    data
}

fn profile_bytes(
    class: &[u8; 4],
    device: &[u8; 4],
    pcs: &[u8; 4],
    media_white_point: &[u8; 12],
    transforms: Vec<Tag>,
) -> Vec<u8> {
    let mut white_point = b"XYZ \0\0\0\0".to_vec();
    white_point.extend(media_white_point);
    let mut tags = vec![
        (b"desc", localized("Synthetic ICC test profile")),
        (b"cprt", localized("Surgeist test fixture")),
        (b"wtpt", white_point),
    ];
    tags.extend(transforms);

    let mut bytes = vec![0; 132 + 12 * tags.len()];
    bytes[8..12].copy_from_slice(&[4, 0x40, 0, 0]);
    bytes[12..16].copy_from_slice(class);
    bytes[16..20].copy_from_slice(device);
    bytes[20..24].copy_from_slice(pcs);
    bytes[24..36].copy_from_slice(&[0x07, 0xea, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0]);
    bytes[36..40].copy_from_slice(b"acsp");
    bytes[68..80].copy_from_slice(&D50);
    bytes[128..132].copy_from_slice(&u32::try_from(tags.len()).unwrap().to_be_bytes());

    for (index, (signature, payload)) in tags.into_iter().enumerate() {
        let entry = 132 + 12 * index;
        let offset = u32::try_from(bytes.len()).unwrap();
        bytes[entry..entry + 4].copy_from_slice(signature);
        bytes[entry + 4..entry + 8].copy_from_slice(&offset.to_be_bytes());
        bytes[entry + 8..entry + 12]
            .copy_from_slice(&u32::try_from(payload.len()).unwrap().to_be_bytes());
        bytes.extend(payload);
        bytes.resize(bytes.len().next_multiple_of(4), 0);
    }
    let size = u32::try_from(bytes.len()).unwrap();
    bytes[..4].copy_from_slice(&size.to_be_bytes());
    bytes
}

// Table 44, two-point table with a constant CLUT and linear input/output tables.
// Its coordinates are deliberately distinct from the float fixtures' outputs.
fn legacy_lut(inputs: u8, outputs: u8, clut_value: u8) -> Vec<u8> {
    let mut data = b"mft1\0\0\0\0".to_vec();
    data.extend([inputs, outputs, 2, 0]);
    for value in [65536_i32, 0, 0, 0, 65536, 0, 0, 0, 65536] {
        data.extend(value.to_be_bytes());
    }
    for _ in 0..inputs {
        data.extend(0..=u8::MAX);
    }
    data.resize(
        data.len() + (1 << inputs) * usize::from(outputs),
        clut_value,
    );
    for _ in 0..outputs {
        data.extend(0..=u8::MAX);
    }
    data
}

fn cmyk_profile(mut tags: Vec<Tag>) -> Vec<u8> {
    for signature in [b"A2B0", b"A2B1", b"A2B2"] {
        tags.push((signature, legacy_lut(4, 3, 128)));
    }
    for signature in [b"B2A0", b"B2A1", b"B2A2"] {
        tags.push((signature, legacy_lut(3, 4, 128)));
    }
    tags.push((b"gamt", legacy_lut(3, 1, 0)));
    // Unlike the header illuminant, the media white is deliberately non-D50.
    // Per ICC.1 §9.1 this must not affect direct D2B3/B2D3 results.
    let media_white = [0, 0, 0xe0, 0, 0, 0, 0xe8, 0, 0, 0, 0xc8, 0];
    profile_bytes(b"prtr", b"CMYK", b"Lab ", &media_white, tags)
}

fn rgb_profile(class: &[u8; 4], mut tags: Vec<Tag>) -> Vec<u8> {
    tags.extend([
        (b"A2B0", legacy_lut(3, 3, 128)),
        (b"B2A0", legacy_lut(3, 3, 128)),
    ]);
    profile_bytes(class, b"RGB ", b"XYZ ", &D50, tags)
}

// Table 62 stores output rows in order, followed by one offset per output.
fn matrix<const IN: usize, const OUT: usize>(
    rows: [[f32; IN]; OUT],
    offsets: [f32; OUT],
) -> Vec<u8> {
    let mut stage = b"matf\0\0\0\0".to_vec();
    stage.extend(u16::try_from(IN).unwrap().to_be_bytes());
    stage.extend(u16::try_from(OUT).unwrap().to_be_bytes());
    for coefficient in rows.into_iter().flatten().chain(offsets) {
        stage.extend(coefficient.to_be_bytes());
    }
    stage
}

// Table 55 addresses each stage relative to the tag's first byte.
fn mpe(inputs: u16, outputs: u16, stages: &[Vec<u8>]) -> Vec<u8> {
    let mut tag = b"mpet\0\0\0\0".to_vec();
    tag.extend(inputs.to_be_bytes());
    tag.extend(outputs.to_be_bytes());
    tag.extend(u32::try_from(stages.len()).unwrap().to_be_bytes());
    let mut offset = u32::try_from(16 + 8 * stages.len()).unwrap();
    for stage in stages {
        let length = u32::try_from(stage.len()).unwrap();
        tag.extend(offset.to_be_bytes());
        tag.extend(length.to_be_bytes());
        offset += length.next_multiple_of(4);
    }
    for stage in stages {
        tag.extend(stage);
        tag.resize(tag.len().next_multiple_of(4), 0);
    }
    tag
}

fn cmyk_to_lab(cyan_weight: f32, offsets: [f32; 3]) -> Vec<u8> {
    let rows = [
        [cyan_weight, -2.0, 0.0, 8.0],
        [0.0, 6.0, 20.0, -2.0],
        [-8.0, 0.0, -4.0, 8.0],
    ];
    mpe(4, 3, &[matrix(rows, offsets)])
}

fn lab_to_cmyk(offsets: [f32; 4]) -> Vec<u8> {
    let rows = [
        [0.015625, 0.03125, 0.0],
        [0.0, 0.0625, -0.03125],
        [0.0625, -0.0625, 0.125],
        [0.0, -0.125, 0.125],
    ];
    mpe(3, 4, &[matrix(rows, offsets)])
}

fn perceptual_cmyk_profile() -> Vec<u8> {
    cmyk_profile(vec![
        (b"D2B0", cmyk_to_lab(4.0, [46.0, -44.0, -6.5])),
        (b"B2D0", lab_to_cmyk([-1.25, 2.5, -5.25, -1.75])),
    ])
}

fn assert_icc_error<T>(result: Result<T>, expected: IccProfileError) {
    let Err(error) = result else {
        panic!("ICC operation accepted input that should produce {expected:?}");
    };
    assert_eq!(error.icc_profile_diagnostic(), Some(&expected), "{error}");
}

#[test]
fn cmyk_lab_matrices_select_every_intent_in_both_directions() {
    let bytes = cmyk_profile(vec![
        (b"D2B0", cmyk_to_lab(4.0, [46.0, -44.0, -6.5])),
        (b"D2B1", cmyk_to_lab(8.0, [53.0, -36.0, -10.0])),
        (b"D2B2", cmyk_to_lab(12.0, [60.0, -27.0, -11.0])),
        (b"D2B3", cmyk_to_lab(16.0, [67.0, -22.0, 0.0])),
        (b"B2D0", lab_to_cmyk([-1.25, 2.5, -5.25, -1.75])),
        (b"B2D1", lab_to_cmyk([-1.0, 2.25, -5.0, -2.25])),
        (b"B2D2", lab_to_cmyk([-0.75, 2.75, -4.75, -2.75])),
        (b"B2D3", lab_to_cmyk([-0.5, 3.0, -4.5, -3.25])),
    ]);
    let mut accounting = RenderAccountingContext::default();
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    assert_eq!(profile.class(), IccProfileClass::Output);
    assert_eq!(profile.device_space().signature(), *b"CMYK");
    assert_eq!(profile.device_space().channel_count(), 4);
    assert_eq!(profile.pcs_space(), IccPcsSpace::Lab);

    let cmyk = profile.device_color(&CMYK_SAMPLE).unwrap();
    assert_eq!(cmyk.channels(), CMYK_SAMPLE);

    // Hand-calculated values from §10.16.2.3 equation (21). For D2B0,
    // L* = -1 - 1 + 6 + 46 = 50. For B2D0, C = 0.9375 - 0.375 - 1.25.
    // Separately constructed PCS input makes reverse checks independent.
    for (intent, expected_lab, expected_cmyk) in [
        (PERCEPTUAL, [50.0, -12.5, -4.5], [-0.6875, 1.5, 0.25, 0.75]),
        (
            IccRenderingIntent::MediaRelativeColorimetric,
            [56.0, -4.5, -8.0],
            [-0.4375, 1.25, 0.5, 0.25],
        ),
        (
            IccRenderingIntent::Saturation,
            [62.0, 4.5, -9.0],
            [-0.1875, 1.75, 0.75, -0.25],
        ),
        (
            IccRenderingIntent::AbsoluteColorimetric,
            [68.0, 9.5, 2.0],
            [0.0625, 2.0, 1.0, -0.75],
        ),
    ] {
        let lab = profile.to_pcs(intent, &cmyk, &mut accounting).unwrap();
        assert_eq!(lab.space(), IccPcsSpace::Lab);
        assert_eq!(lab.intent(), intent);
        assert_eq!(lab.coordinates(), expected_lab);

        let input = IccPcsColor::try_new(IccPcsSpace::Lab, [60.0, -12.0, 8.0], intent).unwrap();
        let output = profile.from_pcs(input, &mut accounting).unwrap();
        assert_eq!(output.channels(), expected_cmyk);
    }
}

#[test]
fn input_display_and_color_space_profiles_run_ordered_xyz_matrices() {
    let first = matrix([[2.0, 4.0, 0.0], [0.0, 0.0, 2.0]], [0.25, -0.25]);
    let second = matrix(
        [[1.0, -1.0], [2.0, 0.5], [0.25, 0.5]],
        [1.0, 0.125, -0.0625],
    );
    let forward = mpe(3, 3, &[first, second]);
    let reverse = mpe(
        3,
        3,
        &[matrix(
            [[0.0, 0.0, 2.0], [1.0, 1.0, 0.0], [-1.0, 0.0, 1.0]],
            [0.25, 0.125, -0.25],
        )],
    );
    let mut accounting = RenderAccountingContext::default();
    for (signature, class) in [
        (b"scnr", IccProfileClass::Input),
        (b"mntr", IccProfileClass::Display),
        (b"spac", IccProfileClass::ColorSpace),
    ] {
        let bytes = rgb_profile(
            signature,
            vec![(b"D2B0", forward.clone()), (b"B2D0", reverse.clone())],
        );
        let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
        assert_eq!(profile.class(), class);
        assert_eq!(profile.device_space().signature(), *b"RGB ");
        assert_eq!(profile.device_space().channel_count(), 3);
        assert_eq!(profile.pcs_space(), IccPcsSpace::XyzD50);

        // The two stages give [0.25, 2.75], then [-1.5, 2.0, 1.375].
        let rgb = profile.device_color(&[0.5, -0.25, 1.5]).unwrap();
        let xyz = profile.to_pcs(PERCEPTUAL, &rgb, &mut accounting).unwrap();
        assert_eq!(xyz.space(), IccPcsSpace::XyzD50);
        assert_eq!(xyz.coordinates(), [-1.5, 2.0, 1.375]);

        let input =
            IccPcsColor::try_new(IccPcsSpace::XyzD50, [0.25, -0.5, 1.25], PERCEPTUAL).unwrap();
        let output = profile.from_pcs(input, &mut accounting).unwrap();
        assert_eq!(output.channels(), [2.75, -0.125, 0.75]);
    }
}

#[test]
fn excluded_classes_and_other_versions_are_rejected_at_inspection() {
    let mut accounting = RenderAccountingContext::default();
    let mut bytes = perceptual_cmyk_profile();
    for class in [b"abst", b"link", b"nmcl"] {
        bytes[12..16].copy_from_slice(class);
        assert_icc_error(
            IccProfile::inspect(&bytes, &mut accounting),
            IccProfileError::ExcludedClass,
        );
    }

    bytes[12..16].copy_from_slice(b"prtr");
    bytes[8..12].copy_from_slice(&[5, 0, 0, 0]);
    assert_icc_error(
        IccProfile::inspect(&bytes, &mut accounting),
        IccProfileError::UnsupportedVersion,
    );
}

#[test]
fn missing_intent_does_not_substitute_another_or_a_legacy_route() {
    let mut accounting = RenderAccountingContext::default();
    let bytes = perceptual_cmyk_profile();
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    let cmyk = profile.device_color(&CMYK_SAMPLE).unwrap();
    let intent = IccRenderingIntent::Saturation;
    assert_icc_error(
        profile.to_pcs(intent, &cmyk, &mut accounting),
        IccProfileError::UnsupportedTransform,
    );

    let lab = IccPcsColor::try_new(IccPcsSpace::Lab, [60.0, -12.0, 8.0], intent).unwrap();
    assert_icc_error(
        profile.from_pcs(lab, &mut accounting),
        IccProfileError::UnsupportedTransform,
    );
}

#[test]
fn unsupported_stage_and_malformed_stage_size_or_shape_are_distinct() {
    let mut accounting = RenderAccountingContext::default();
    let unknown_stage = b"xmpl\0\0\0\0\0\x04\0\x03".to_vec();
    let mut short_matrix = matrix([[0.0; 4]; 3], [0.0; 3]);
    short_matrix.truncate(short_matrix.len() - 4);
    let wrong_inputs = matrix([[0.0; 3]; 3], [0.0; 3]);
    for (stage, expected) in [
        (unknown_stage, IccProfileError::UnsupportedTransform),
        (short_matrix, IccProfileError::InvalidProfile),
        (wrong_inputs, IccProfileError::InvalidProfile),
    ] {
        let bytes = cmyk_profile(vec![(b"D2B0", mpe(4, 3, &[stage]))]);
        let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
        let cmyk = profile.device_color(&CMYK_SAMPLE).unwrap();
        assert_icc_error(profile.to_pcs(PERCEPTUAL, &cmyk, &mut accounting), expected);
    }
}

#[test]
fn conversion_rejects_incompatible_values_and_out_of_range_coordinates() {
    let mut accounting = RenderAccountingContext::default();
    let bytes = perceptual_cmyk_profile();
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    assert_icc_error(
        profile.device_color(&[0.0; 3]),
        IccProfileError::IncompatibleColor,
    );

    for invalid in [f64::NAN, f64::NEG_INFINITY, f64::from(f32::MAX) * 2.0] {
        assert_icc_error(
            profile.device_color(&[0.0, invalid, 0.0, 0.0]),
            IccProfileError::InvalidCoordinate,
        );
        assert_icc_error(
            IccPcsColor::try_new(IccPcsSpace::Lab, [50.0, 0.0, invalid], PERCEPTUAL),
            IccProfileError::InvalidCoordinate,
        );
    }

    let xyz = IccPcsColor::try_new(IccPcsSpace::XyzD50, [0.25, 0.5, 0.75], PERCEPTUAL).unwrap();
    assert_icc_error(
        profile.from_pcs(xyz, &mut accounting),
        IccProfileError::IncompatibleColor,
    );

    let too_large_for_lab = profile
        .device_color(&[f64::from(f32::MAX), 0.0, 0.0, 0.0])
        .unwrap();
    assert_icc_error(
        profile.to_pcs(PERCEPTUAL, &too_large_for_lab, &mut accounting),
        IccProfileError::NumericOutput,
    );
}

#[test]
fn inspection_caps_report_a_typed_refusal_and_release_custody_for_retry() {
    let options =
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
    let tag = RenderWork::IccProfileTagVisits;
    let scratch = RenderResource::ScratchCpuBytes;
    let bytes = perceptual_cmyk_profile();

    for (limits, quantity) in [
        (
            RenderLimits::new().with_work_limit(tag, 0).unwrap(),
            RenderQuantity::Work(tag),
        ),
        (
            RenderLimits::new().with_scratch_bytes(0),
            RenderQuantity::Resource(scratch),
        ),
    ] {
        let mut accounting = RenderAccountingContext::new(options.with_limits(limits));
        let error = IccProfile::inspect(&bytes, &mut accounting).unwrap_err();
        let terminal = accounting.take_last_attempt().unwrap();
        let report = terminal.attempt().unwrap();
        let refusal = report.refusal().unwrap();

        assert!(
            matches!(error.accounting_diagnostic(), Some(RenderAccountingError::Refused(detail)) if detail == refusal)
        );
        assert_eq!(report.kind(), RenderAttemptKind::IccProfile);
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(report.phase_count(RenderPhase::IccProfilePrepare), 1);
        assert_eq!(refusal.quantity(), quantity);
        assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 0 });
        assert_eq!(refusal.current(), 0);
        assert!(refusal.amount() > 0);
        assert_eq!(report.usage().live(scratch), 0);
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(terminal.cleanup_fault(), None);

        accounting.set_options(options);
        let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
        let cmyk = profile.device_color(&CMYK_SAMPLE).unwrap();
        let pcs = profile.to_pcs(PERCEPTUAL, &cmyk, &mut accounting).unwrap();
        assert_eq!(pcs.coordinates(), [50.0, -12.5, -4.5]);
        let report = accounting.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.kind(), RenderAttemptKind::IccProfile);
        assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(report.phase_count(RenderPhase::IccProfileConvert), 1);
        assert_eq!(report.usage().live(scratch), 0);
    }
}
