//! Native ICC LUT vectors derived from ICC.1:2022 §§6.3.2, 6.3.4 and 10.10–10.13.

#![forbid(unsafe_code)]

use surgeist_render::{
    Result,
    accounting::{
        RenderAccountingContext, RenderAccountingError, RenderAccountingOptions,
        RenderAttemptStatus, RenderLimits, RenderPhase, RenderProfile, RenderQuantity,
        RenderRefusalReason, RenderResource, RenderWork,
    },
    icc::{IccPcsColor, IccPcsSpace, IccProfile, IccProfileError, IccRenderingIntent},
};

type Tag = (&'static [u8; 4], Vec<u8>);

const D50: [i32; 3] = [0xf6d6, 0x10000, 0xd32d];
const IDENTITY: [i32; 9] = [65536, 0, 0, 0, 65536, 0, 0, 0, 65536];
const PCS_CODE_MAX: f64 = 65535.0;
const RELATIVE: IccRenderingIntent = IccRenderingIntent::MediaRelativeColorimetric;
const PERCEPTUAL: IccRenderingIntent = IccRenderingIntent::Perceptual;
const SATURATION: IccRenderingIntent = IccRenderingIntent::Saturation;
const ABSOLUTE: IccRenderingIntent = IccRenderingIntent::AbsoluteColorimetric;

#[derive(Clone, Copy)]
enum Version {
    V2,
    V4,
}

fn localized(text: &str) -> Vec<u8> {
    let mut data = b"mluc\0\0\0\0\0\0\0\x01\0\0\0\x0cenUS".to_vec();
    let text: Vec<_> = text.encode_utf16().collect();
    data.extend(u32::try_from(text.len() * 2).unwrap().to_be_bytes());
    data.extend(28_u32.to_be_bytes());
    for character in text {
        data.extend(character.to_be_bytes());
    }
    data
}

fn profile_bytes(
    version: Version,
    class: &[u8; 4],
    device: &[u8; 4],
    pcs: &[u8; 4],
    white: [i32; 3],
    transforms: Vec<Tag>,
) -> Vec<u8> {
    let mut white_point = b"XYZ \0\0\0\0".to_vec();
    for coordinate in white {
        white_point.extend(coordinate.to_be_bytes());
    }
    let (version, description, copyright) = match version {
        Version::V2 => {
            let mut description = b"desc\0\0\0\0".to_vec();
            let title = b"Synthetic LUT profile\0";
            description.extend(u32::try_from(title.len()).unwrap().to_be_bytes());
            description.extend(title);
            // Empty optional Unicode and ScriptCode descriptions.
            description.extend([0; 78]);
            (
                [2, 0x40, 0, 0],
                description,
                b"text\0\0\0\0Surgeist test fixture\0".to_vec(),
            )
        }
        Version::V4 => (
            [4, 0x40, 0, 0],
            localized("Synthetic LUT profile"),
            localized("Surgeist test fixture"),
        ),
    };
    let mut tags = vec![
        (b"desc", description),
        (b"cprt", copyright),
        (b"wtpt", white_point),
    ];
    tags.extend(transforms);

    let mut bytes = vec![0; 132 + 12 * tags.len()];
    bytes[8..12].copy_from_slice(&version);
    bytes[12..16].copy_from_slice(class);
    bytes[16..20].copy_from_slice(device);
    bytes[20..24].copy_from_slice(pcs);
    bytes[24..36].copy_from_slice(&[0x07, 0xea, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0]);
    bytes[36..40].copy_from_slice(b"acsp");
    for (slot, coordinate) in bytes[68..80].as_chunks_mut::<4>().0.iter_mut().zip(D50) {
        slot.copy_from_slice(&coordinate.to_be_bytes());
    }
    bytes[128..132].copy_from_slice(&u32::try_from(tags.len()).unwrap().to_be_bytes());

    let mut written = Vec::<(Vec<u8>, u32)>::new();
    for (index, (signature, payload)) in tags.into_iter().enumerate() {
        let entry = 132 + 12 * index;
        let length = u32::try_from(payload.len()).unwrap();
        let offset = if let Some((_, offset)) = written.iter().find(|(data, _)| *data == payload) {
            *offset
        } else {
            let offset = u32::try_from(bytes.len()).unwrap();
            bytes.extend(&payload);
            bytes.resize(bytes.len().next_multiple_of(4), 0);
            written.push((payload, offset));
            offset
        };
        bytes[entry..entry + 4].copy_from_slice(signature);
        bytes[entry + 4..entry + 8].copy_from_slice(&offset.to_be_bytes());
        bytes[entry + 8..entry + 12].copy_from_slice(&length.to_be_bytes());
    }
    let size = u32::try_from(bytes.len()).unwrap();
    bytes[..4].copy_from_slice(&size.to_be_bytes());
    bytes
}

fn lut_header(signature: &[u8; 4], inputs: usize, outputs: usize, matrix: [i32; 9]) -> Vec<u8> {
    let mut data = signature.to_vec();
    data.extend([0; 4]);
    data.extend([
        u8::try_from(inputs).unwrap(),
        u8::try_from(outputs).unwrap(),
        2,
        0,
    ]);
    for coefficient in matrix {
        data.extend(coefficient.to_be_bytes());
    }
    data
}

fn lut16<const IN: usize, const OUT: usize>(
    matrix: [i32; 9],
    input_endpoints: [u16; IN],
    corner: impl Fn([u16; IN]) -> [u16; OUT],
) -> Vec<u8> {
    let mut data = lut_header(b"mft2", IN, OUT, matrix);
    data.extend([0, 2, 0, 2]);
    for endpoint in input_endpoints {
        data.extend([0, 0]);
        data.extend(endpoint.to_be_bytes());
    }
    // Table 40, first input varies least rapidly. Two points per axis.
    for address in 0..1 << IN {
        let axes = std::array::from_fn(|axis| u16::from(address & (1 << (IN - axis - 1)) != 0));
        for code in corner(axes) {
            data.extend(code.to_be_bytes());
        }
    }
    for _ in 0..OUT {
        data.extend([0, 0, 255, 255]);
    }
    data
}

fn lut8<const IN: usize, const OUT: usize>(corner: impl Fn([u8; IN]) -> [u8; OUT]) -> Vec<u8> {
    let mut data = lut_header(b"mft1", IN, OUT, IDENTITY);
    for _ in 0..IN {
        data.extend(0..=255);
    }
    for address in 0..1 << IN {
        let axes = std::array::from_fn(|axis| u8::from(address & (1 << (IN - axis - 1)) != 0));
        data.extend(corner(axes));
    }
    for _ in 0..OUT {
        data.extend(0..=255);
    }
    data
}

fn cmyk_profile(version: Version, forward: Vec<u8>, reverse: Vec<u8>) -> Vec<u8> {
    let tags = vec![
        (b"A2B0", forward.clone()),
        (b"A2B1", forward.clone()),
        (b"A2B2", forward),
        (b"B2A0", reverse.clone()),
        (b"B2A1", reverse.clone()),
        (b"B2A2", reverse),
        (b"gamt", lut8::<3, 1>(|_| [0])),
    ];
    profile_bytes(version, b"prtr", b"CMYK", b"Lab ", D50, tags)
}

fn rgb_profile(pcs: &[u8; 4], white: [i32; 3], tags: Vec<Tag>) -> Vec<u8> {
    profile_bytes(Version::V4, b"spac", b"RGB ", pcs, white, tags)
}

fn close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (index, (&actual, &expected)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (actual - expected).abs() <= 1.0e-10,
            "channel {index}: got {actual}, expected {expected}"
        );
    }
}

fn assert_icc_error<T>(result: Result<T>, expected: IccProfileError) {
    let Err(error) = result else {
        panic!("ICC operation accepted input that should produce {expected:?}");
    };
    assert_eq!(error.icc_profile_diagnostic(), Some(&expected), "{error}");
}

#[test]
fn lut16_cmyk_preserves_legacy_lab_codes_in_both_profile_versions() {
    let forward = lut16(IDENTITY, [65535; 4], |[c, m, y, k]| {
        [
            0x4000 + 0x4000 * c + 0x2000 * k,
            0x6000 + 0x4000 * m,
            0xff00 + 0x00ff * y,
        ]
    });
    let reverse = lut16(IDENTITY, [65535; 3], |[l, a, b]| {
        [65535 * l, 65535 * a, 65535 * b, 65535 * (1 - l)]
    });
    let mut accounting = RenderAccountingContext::default();
    for version in [Version::V2, Version::V4] {
        let bytes = cmyk_profile(version, forward.clone(), reverse.clone());
        let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
        let cmyk = profile.device_color(&[0.25, 0.5, 0.75, 1.0]).unwrap();
        let lab = profile.to_pcs(RELATIVE, &cmyk, &mut accounting).unwrap();
        assert_eq!(lab.space(), IccPcsSpace::Lab);
        assert_eq!(lab.intent(), RELATIVE);
        // Legacy L* full scale is 0xff00; neutral is 0x8000. Interpolation
        // yields a noninteger b* code past 0xff00, which remains valid.
        close(
            &lab.coordinates(),
            &[43.92156862745098, 0.0, 127.7470703125],
        );

        let lab = IccPcsColor::try_new(IccPcsSpace::Lab, [50.0, 0.0, 127.75], RELATIVE).unwrap();
        let cmyk = profile.from_pcs(lab, &mut accounting).unwrap();
        close(
            cmyk.channels(),
            &[
                32640.0 / PCS_CODE_MAX,
                32768.0 / PCS_CODE_MAX,
                65472.0 / PCS_CODE_MAX,
                32895.0 / PCS_CODE_MAX,
            ],
        );
    }
}

#[test]
fn reverse_lut16_applies_the_xyz_matrix_before_the_channel_tables() {
    let forward = lut16(IDENTITY, [65535; 3], |[r, g, b]| {
        [65535 * r, 65535 * g, 65535 * b]
    });
    // X'=2Y, Y'=X, Z'=Z. Only the first input table attenuates.
    let matrix = [0, 131072, 0, 65536, 0, 0, 0, 0, 65536];
    let reverse = lut16(matrix, [32768, 65535, 65535], |[x, y, z]| {
        [65535 * x, 65535 * y, 65535 * z]
    });
    let bytes = rgb_profile(b"XYZ ", D50, vec![(b"A2B0", forward), (b"B2A0", reverse)]);
    let mut accounting = RenderAccountingContext::default();
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    let xyz = IccPcsColor::try_new(IccPcsSpace::XyzD50, [0.5, 0.75, 0.125], PERCEPTUAL).unwrap();
    let rgb = profile.from_pcs(xyz, &mut accounting).unwrap();
    // Actual XYZ units use 32768 codes per unit. Values entering the LUT
    // are divided by the u16 maximum, then the red table scales by 32768/65535.
    close(
        rgb.channels(),
        &[
            49152.0 * 32768.0 / (PCS_CODE_MAX * PCS_CODE_MAX),
            16384.0 / PCS_CODE_MAX,
            4096.0 / PCS_CODE_MAX,
        ],
    );
}

#[test]
fn lut8_uses_exact_intent_before_perceptual_fallback_in_both_directions() {
    let bytes = rgb_profile(
        b"Lab ",
        D50,
        vec![
            (b"A2B0", lut8::<3, 3>(|_| [204, 128, 100])),
            (b"A2B1", lut8::<3, 3>(|_| [51, 144, 153])),
            (b"B2A0", lut8(|[l, a, b]| [255 * l, 255 * a, 255 * b])),
            (b"B2A1", lut8::<3, 3>(|_| [51, 153, 204])),
        ],
    );
    let mut accounting = RenderAccountingContext::default();
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    let rgb = profile.device_color(&[0.25, 0.5, 0.75]).unwrap();
    for (intent, forward, reverse) in [
        (RELATIVE, [20.0, 16.0, 25.0], [0.2, 0.6, 0.8]),
        (
            SATURATION,
            [80.0, 0.0, -28.0],
            [0.625, 164.5 / 255.0, 118.25 / 255.0],
        ),
    ] {
        let lab = profile.to_pcs(intent, &rgb, &mut accounting).unwrap();
        close(&lab.coordinates(), &forward);
        assert_eq!(lab.intent(), intent);
        let lab = IccPcsColor::try_new(IccPcsSpace::Lab, [62.5, 36.5, -9.75], intent).unwrap();
        let rgb = profile.from_pcs(lab, &mut accounting).unwrap();
        close(rgb.channels(), &reverse);
    }
}

// One float MPE matrix with a three-channel header and identity coefficients.
fn float_matrix(offsets: [f32; 3]) -> Vec<u8> {
    let mut data = b"mpet\0\0\0\0\0\x03\0\x03\0\0\0\x01".to_vec();
    data.extend(24_u32.to_be_bytes());
    data.extend(60_u32.to_be_bytes());
    data.extend(b"matf\0\0\0\0\0\x03\0\x03");
    for value in [1.0_f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
        .into_iter()
        .chain(offsets)
    {
        data.extend(value.to_be_bytes());
    }
    data
}

fn unknown_mpe() -> Vec<u8> {
    b"mpet\0\0\0\0\0\x03\0\x03\0\0\0\x01\0\0\0\x18\0\0\0\x0cxmpl\0\0\0\0\0\x03\0\x03".to_vec()
}

#[test]
fn a_supported_mpe_wins_and_an_unsupported_mpe_uses_the_selected_legacy_route() {
    let zero = lut16(IDENTITY, [65535; 3], |_| [0; 3]);
    let legacy = lut16(IDENTITY, [65535; 3], |_| [8192, 16384, 24576]);
    let mut accounting = RenderAccountingContext::default();
    for (mpe, expected_pcs, expected_device) in [
        (
            float_matrix([0.125, 0.125, 0.125]),
            [0.25, 0.375, 0.625],
            [0.25, 0.375, 0.625],
        ),
        (
            unknown_mpe(),
            [0.25, 0.5, 0.75],
            [
                8192.0 / PCS_CODE_MAX,
                16384.0 / PCS_CODE_MAX,
                24576.0 / PCS_CODE_MAX,
            ],
        ),
    ] {
        let bytes = rgb_profile(
            b"XYZ ",
            D50,
            vec![
                (b"A2B0", zero.clone()),
                (b"B2A0", zero.clone()),
                (b"A2B1", legacy.clone()),
                (b"B2A1", legacy.clone()),
                (b"D2B1", mpe.clone()),
                (b"B2D1", mpe),
            ],
        );
        let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
        let rgb = profile.device_color(&[0.125, 0.25, 0.5]).unwrap();
        let xyz = profile.to_pcs(RELATIVE, &rgb, &mut accounting).unwrap();
        close(&xyz.coordinates(), &expected_pcs);
        let xyz = IccPcsColor::try_new(IccPcsSpace::XyzD50, [0.125, 0.25, 0.5], RELATIVE).unwrap();
        let rgb = profile.from_pcs(xyz, &mut accounting).unwrap();
        close(rgb.channels(), &expected_device);
    }
}

#[test]
fn malformed_selected_route_does_not_fall_through_to_a_valid_tag() {
    let good = lut16(IDENTITY, [65535; 3], |_| [8192, 16384, 24576]);
    let mut broken_lut = good.clone();
    broken_lut.truncate(broken_lut.len() - 2);
    let mut broken_mpe = float_matrix([0.0; 3]);
    broken_mpe[33] = 4; // Selected stage claims four input channels.
    let mut accounting = RenderAccountingContext::default();

    for (forward, reverse, extra) in [
        (broken_lut.clone(), broken_lut, vec![]),
        (
            good.clone(),
            good.clone(),
            vec![(b"D2B1", broken_mpe.clone()), (b"B2D1", broken_mpe)],
        ),
    ] {
        let mut tags = vec![
            (b"A2B0", good.clone()),
            (b"B2A0", good.clone()),
            (b"A2B1", forward),
            (b"B2A1", reverse),
        ];
        tags.extend(extra);
        let bytes = rgb_profile(b"XYZ ", D50, tags);
        let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
        let rgb = profile.device_color(&[0.125, 0.25, 0.5]).unwrap();
        assert_icc_error(
            profile.to_pcs(RELATIVE, &rgb, &mut accounting),
            IccProfileError::InvalidProfile,
        );
        let xyz = IccPcsColor::try_new(IccPcsSpace::XyzD50, [0.125, 0.25, 0.5], RELATIVE).unwrap();
        assert_icc_error(
            profile.from_pcs(xyz, &mut accounting),
            IccProfileError::InvalidProfile,
        );
    }
}

#[test]
fn indirect_absolute_uses_media_white_and_direct_absolute_bypasses_it() {
    let relative = lut16(IDENTITY, [65535; 3], |_| [0, 16384, 0]);
    let reverse = lut16(IDENTITY, [65535; 3], |[x, y, z]| {
        [65535 * x, 65535 * y, 65535 * z]
    });
    let mut accounting = RenderAccountingContext::default();

    for (direct_tags, absolute_y, device_y) in [
        (vec![], 0.25, 16384.0 / PCS_CODE_MAX),
        (
            vec![
                (b"D2B3", float_matrix([0.0, 0.125, 0.0])),
                (b"B2D3", float_matrix([0.0; 3])),
            ],
            0.375,
            0.25,
        ),
    ] {
        let mut tags = vec![
            (b"A2B0", relative.clone()),
            (b"A2B1", relative.clone()),
            (b"A2B2", relative.clone()),
            (b"B2A0", reverse.clone()),
            (b"B2A1", reverse.clone()),
            (b"B2A2", reverse.clone()),
            (b"gamt", lut16(IDENTITY, [65535; 3], |_| [0])),
        ];
        tags.extend(direct_tags);
        let bytes = profile_bytes(
            Version::V4,
            b"prtr",
            b"RGB ",
            b"XYZ ",
            [D50[0], 32768, D50[2]],
            tags,
        );
        let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
        let rgb = profile.device_color(&[0.0, 0.25, 0.0]).unwrap();
        let pcs = profile.to_pcs(ABSOLUTE, &rgb, &mut accounting).unwrap();
        close(&pcs.coordinates(), &[0.0, absolute_y, 0.0]);
        assert_eq!(pcs.intent(), ABSOLUTE);
        let input = IccPcsColor::try_new(IccPcsSpace::XyzD50, [0.0, 0.25, 0.0], ABSOLUTE).unwrap();
        close(
            profile.from_pcs(input, &mut accounting).unwrap().channels(),
            &[0.0, device_y, 0.0],
        );

        let rel = profile.to_pcs(RELATIVE, &rgb, &mut accounting).unwrap();
        close(&rel.coordinates(), &[0.0, 0.5, 0.0]);
    }
}

fn curves(square_axis: usize) -> Vec<u8> {
    let mut data = Vec::new();
    for axis in 0..3 {
        if axis == square_axis {
            data.extend(b"para\0\0\0\0\0\0\0\0\0\x02\0\0");
        } else {
            data.extend(b"curv\0\0\0\0\0\0\0\0");
        }
    }
    data
}

fn staged(signature: &[u8; 4]) -> Vec<u8> {
    let mut data = vec![0; 32];
    data[..4].copy_from_slice(signature);
    data[8..10].copy_from_slice(&[3, 3]);
    let b_curves = 32_u32.to_be_bytes();
    data[12..16].copy_from_slice(&b_curves);
    data[20..24].copy_from_slice(&b_curves); // The B and M curve sets alias.
    data.extend(curves(0));
    let matrix_offset = u32::try_from(data.len()).unwrap();
    data[16..20].copy_from_slice(&matrix_offset.to_be_bytes());
    // The matrix swaps the first two channels and adds 1/8 to the first.
    for coefficient in [0_i32, 65536, 0, 65536, 0, 0, 0, 0, 65536, 8192, 0, 0] {
        data.extend(coefficient.to_be_bytes());
    }
    let clut_offset = u32::try_from(data.len()).unwrap();
    data[24..28].copy_from_slice(&clut_offset.to_be_bytes());
    data.extend([2, 2, 2]);
    data.extend([0; 13]);
    data.extend([1, 0, 0, 0]);
    for corner in 0..8 {
        let [x, y, z] = [4, 2, 1].map(|bit| u8::from(corner & bit != 0));
        data.extend([255 * z, 255 * x, 255 * (1 - y)]);
    }
    let a_offset = u32::try_from(data.len()).unwrap();
    data[28..32].copy_from_slice(&a_offset.to_be_bytes());
    data.extend(curves(1));
    data
}

#[test]
fn staged_luts_execute_direction_specific_order_and_aliased_curves() {
    let bytes = rgb_profile(
        b"Lab ",
        D50,
        vec![(b"A2B0", staged(b"mAB ")), (b"B2A0", staged(b"mBA "))],
    );
    let mut accounting = RenderAccountingContext::default();
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    let rgb = profile.device_color(&[0.25, 0.5, 0.75]).unwrap();
    let lab = profile.to_pcs(PERCEPTUAL, &rgb, &mut accounting).unwrap();
    // A, CLUT, M, matrix, B yield normalized [0.140625, 0.5625, 0.75].
    // Staged Lab uses the current PCS encoding.
    close(&lab.coordinates(), &[14.0625, 15.4375, 63.25]);

    let lab = IccPcsColor::try_new(IccPcsSpace::Lab, [25.0, -0.5, 63.25], PERCEPTUAL).unwrap();
    let rgb = profile.from_pcs(lab, &mut accounting).unwrap();
    // B, matrix, M, CLUT, A give [0.75, (0.390625)^2, 0.9375].
    close(rgb.channels(), &[0.75, 0.152587890625, 0.9375]);
}

#[test]
fn legacy_conversion_budget_refusal_releases_scratch_and_allows_retry() {
    let bytes = rgb_profile(
        b"Lab ",
        D50,
        vec![(b"A2B0", staged(b"mAB ")), (b"B2A0", staged(b"mBA "))],
    );
    let options =
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
    let mut accounting = RenderAccountingContext::new(options);
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    let rgb = profile.device_color(&[0.25, 0.5, 0.75]).unwrap();
    let baseline = profile.to_pcs(PERCEPTUAL, &rgb, &mut accounting).unwrap();
    close(&baseline.coordinates(), &[14.0625, 15.4375, 63.25]);
    let coordinate = RenderWork::CoordinateVisits;
    let prep = RenderPhase::IccProfilePrepare;
    let baseline = accounting.take_last_attempt().unwrap();
    let limit = baseline
        .attempt()
        .unwrap()
        .phase_usage(prep)
        .work(coordinate);
    accounting.set_options(
        options.with_limits(
            RenderLimits::new()
                .with_work_limit(coordinate, limit)
                .unwrap(),
        ),
    );

    let error = profile
        .to_pcs(PERCEPTUAL, &rgb, &mut accounting)
        .unwrap_err();
    let terminal = accounting.take_last_attempt().unwrap();
    let report = terminal.attempt().unwrap();
    let refusal = report.refusal().unwrap();
    assert!(
        matches!(error.accounting_diagnostic(), Some(RenderAccountingError::Refused(detail)) if detail == refusal)
    );
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    assert_eq!(refusal.phase(), RenderPhase::Attempt);
    assert_eq!(refusal.quantity(), RenderQuantity::Work(coordinate));
    assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit });
    assert_eq!(report.phase_count(RenderPhase::IccProfileConvert), 1);
    assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
    assert_eq!(terminal.cleanup_fault(), None);

    accounting.set_options(options);
    let retry = profile.to_pcs(PERCEPTUAL, &rgb, &mut accounting).unwrap();
    close(&retry.coordinates(), &[14.0625, 15.4375, 63.25]);
}
