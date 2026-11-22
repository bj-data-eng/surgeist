//! Public ICC matrix and tone-curve witnesses from ICC.1:2022 Annex F.

#![forbid(unsafe_code)]

use surgeist_render::{
    Result,
    accounting::{
        RenderAccountingContext, RenderAccountingError, RenderAccountingOptions,
        RenderAttemptStatus, RenderLimits, RenderPhase, RenderProfile, RenderResource, RenderWork,
    },
    icc::{IccPcsColor, IccPcsSpace, IccProfile, IccProfileError, IccRenderingIntent},
};

type Tag = (&'static [u8; 4], Vec<u8>);

const V2: [u8; 4] = [2, 0x40, 0, 0];
const V4: [u8; 4] = [4, 0x40, 0, 0];
const D50: [i32; 3] = [63190, 65536, 54061];
const RELATIVE: IccRenderingIntent = IccRenderingIntent::MediaRelativeColorimetric;
const ABSOLUTE: IccRenderingIntent = IccRenderingIntent::AbsoluteColorimetric;
const RGB_COLUMNS: [[i32; 3]; 3] = [
    [32768, 16384, 0],
    [16384, 32768, 8192],
    [14038, 16384, 45869],
];
const RGB_PCS: [f64; 3] = [0.32588768005371094, 0.453125, 0.13749408721923828];

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

fn xyz(coordinates: [i32; 3]) -> Vec<u8> {
    let mut data = b"XYZ \0\0\0\0".to_vec();
    for coordinate in coordinates {
        data.extend(coordinate.to_be_bytes());
    }
    data
}

fn profile(
    version: [u8; 4],
    class: &[u8; 4],
    device: &[u8; 4],
    pcs: &[u8; 4],
    white: [i32; 3],
    transforms: Vec<Tag>,
) -> Vec<u8> {
    let (description, copyright) = if version == V2 {
        let mut description = b"desc\0\0\0\0\0\0\0\x0bICC matrix\0".to_vec();
        description.extend([0; 78]);
        (description, b"text\0\0\0\0Surgeist fixture\0".to_vec())
    } else {
        (localized("ICC matrix"), localized("Surgeist fixture"))
    };
    let mut tags = vec![
        (b"desc", description),
        (b"cprt", copyright),
        (b"wtpt", xyz(white)),
    ];
    tags.extend(transforms);

    let mut bytes = vec![0; 132 + 12 * tags.len()];
    bytes[8..12].copy_from_slice(&version);
    bytes[12..16].copy_from_slice(class);
    bytes[16..20].copy_from_slice(device);
    bytes[20..24].copy_from_slice(pcs);
    bytes[24..36].copy_from_slice(&[0x07, 0xea, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0]);
    bytes[36..40].copy_from_slice(b"acsp");
    bytes[68..80].copy_from_slice(&xyz(D50)[8..]);
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

fn curve(samples: &[u16]) -> Vec<u8> {
    let mut data = b"curv\0\0\0\0".to_vec();
    data.extend(u32::try_from(samples.len()).unwrap().to_be_bytes());
    for sample in samples {
        data.extend(sample.to_be_bytes());
    }
    data
}

fn rgb_tags(columns: [[i32; 3]; 3], curves: [Vec<u8>; 3]) -> Vec<Tag> {
    let [r, g, b] = columns;
    let [r_curve, g_curve, b_curve] = curves;
    vec![
        (b"rXYZ", xyz(r)),
        (b"gXYZ", xyz(g)),
        (b"bXYZ", xyz(b)),
        (b"rTRC", r_curve),
        (b"gTRC", g_curve),
        (b"bTRC", b_curve),
    ]
}

fn calibrated_rgb_tags() -> Vec<Tag> {
    rgb_tags(RGB_COLUMNS, [curve(&[512]), curve(&[]), curve(&[512])])
}

fn gray_profile(version: [u8; 4], pcs: &[u8; 4], curve: Vec<u8>) -> Vec<u8> {
    profile(version, b"scnr", b"GRAY", pcs, D50, vec![(b"kTRC", curve)])
}

fn samples_match(actual: &[f64], expected: &[f64], tolerance: f64) {
    assert_eq!(actual.len(), expected.len());
    for (index, (&actual, &expected)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (actual - expected).abs() <= tolerance,
            "channel {index}: got {actual}, expected {expected}"
        );
    }
}

fn close(actual: &[f64], expected: &[f64]) {
    samples_match(actual, expected, 1.0e-10);
}

fn assert_icc_error<T>(result: Result<T>, expected: IccProfileError) {
    let Err(error) = result else {
        panic!("ICC operation accepted input that should produce {expected:?}");
    };
    assert_eq!(error.icc_profile_diagnostic(), Some(&expected), "{error}");
}

#[test]
fn gray_gamma_uses_the_achromatic_pcs_axis_for_each_representation() {
    let mut accounting = RenderAccountingContext::default();
    for (signature, space, white_scaled, off_axis) in [
        (
            b"XYZ ",
            IccPcsSpace::XyzD50,
            [0.24105, 0.25, 0.206225],
            [0.9, 0.25, 0.01],
        ),
        (
            b"Lab ",
            IccPcsSpace::Lab,
            [25.0, 0.0, 0.0],
            [25.0, 54.0, -17.0],
        ),
    ] {
        let bytes = gray_profile(V4, signature, curve(&[512]));
        let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
        assert_eq!(profile.device_space().signature(), *b"GRAY");
        let gray = profile.device_color(&[0.5]).unwrap();
        let pcs = profile.to_pcs(RELATIVE, &gray, &mut accounting).unwrap();
        assert_eq!(pcs.space(), space);
        // F.2 scales the nominal PCS white by (1/2)^2.
        samples_match(&pcs.coordinates(), &white_scaled, 1.0e-12);

        let pcs = IccPcsColor::try_new(space, off_axis, RELATIVE).unwrap();
        let gray = profile.from_pcs(pcs, &mut accounting).unwrap();
        close(gray.channels(), &[0.5]);
    }
}

#[test]
fn gray_v2_descending_curve_inverts_and_selects_the_nearest_endpoint() {
    let bytes = gray_profile(V2, b"XYZ ", curve(&[65535, 0]));
    let mut accounting = RenderAccountingContext::default();
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    let gray = profile.device_color(&[0.25]).unwrap();
    let xyz = profile.to_pcs(RELATIVE, &gray, &mut accounting).unwrap();
    // The descending curve y = 1 - x supplies three-quarters of PCS white.
    samples_match(&xyz.coordinates(), &[0.72315, 0.75, 0.618675], 1.0e-12);
    for (y, expected) in [(0.75, 0.25), (-0.125, 1.0), (1.25, 0.0)] {
        let xyz = IccPcsColor::try_new(IccPcsSpace::XyzD50, [0.0, y, 0.0], RELATIVE).unwrap();
        close(
            profile.from_pcs(xyz, &mut accounting).unwrap().channels(),
            &[expected],
        );
    }
}

#[test]
fn sampled_inverse_selects_opposite_ends_of_interior_and_terminal_plateaus() {
    let bytes = gray_profile(V4, b"XYZ ", curve(&[0, 16384, 16384, 49152, 49152]));
    let mut accounting = RenderAccountingContext::default();
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    // F.1 selects the right edge of [.25, .5], the left edge of [.75, 1],
    // and the midpoint of the sloping segment for the intermediate code.
    for (y, expected) in [
        (16384.0 / 65535.0, 0.5),
        (49152.0 / 65535.0, 0.75),
        (32768.0 / 65535.0, 0.625),
        (1.25, 0.75),
    ] {
        let xyz = IccPcsColor::try_new(IccPcsSpace::XyzD50, [0.0, y, 0.0], RELATIVE).unwrap();
        close(
            profile.from_pcs(xyz, &mut accounting).unwrap().channels(),
            &[expected],
        );
    }
}

#[test]
fn quantized_parametric_join_uses_the_nearest_attained_value() {
    // Type 3 applies c*x below d and (a*x + b)^g at and above d.
    // With d=1/2 and c=(1/2)-2^-16 the upper branch starts at 1/4,
    // leaving an upward gap when the encoded coefficients are used exactly.
    let mut parametric = b"para\0\0\0\0\0\x03\0\0".to_vec();
    for coefficient in [131072_i32, 65536, 0, 32767, 32768] {
        parametric.extend(coefficient.to_be_bytes());
    }
    let bytes = gray_profile(V4, b"XYZ ", parametric);
    let mut accounting = RenderAccountingContext::default();
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    let left_y = 32767.0 / 262144.0;
    for (x, expected_y) in [(0.25, left_y), (0.5, 0.25), (0.75, 0.5625)] {
        let gray = profile.device_color(&[x]).unwrap();
        let xyz = profile.to_pcs(RELATIVE, &gray, &mut accounting).unwrap();
        close(&[xyz.coordinates()[1]], &[expected_y]);
    }
    // 1/4 - 2^-19 is closer to the attained upper join than to any
    // value of the lower branch. F.1(b) selects the upper join at x=1/2.
    for (y, expected) in [(left_y, 0.25), (131071.0 / 524288.0, 0.5), (0.5625, 0.75)] {
        let xyz = IccPcsColor::try_new(IccPcsSpace::XyzD50, [0.0, y, 0.0], RELATIVE).unwrap();
        close(
            profile.from_pcs(xyz, &mut accounting).unwrap().channels(),
            &[expected],
        );
    }
}

#[test]
fn noninvertible_curves_and_a_singular_matrix_remain_usable_forward() {
    let mut accounting = RenderAccountingContext::default();
    for (samples, expected_y) in [
        (&[32768, 32768][..], 32768.0 / 65535.0),
        (&[0, 65535, 0][..], 0.5),
    ] {
        let bytes = gray_profile(V4, b"XYZ ", curve(samples));
        let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
        let gray = profile.device_color(&[0.25]).unwrap();
        let pcs = profile.to_pcs(RELATIVE, &gray, &mut accounting).unwrap();
        close(&[pcs.coordinates()[1]], &[expected_y]);
        assert_icc_error(
            profile.from_pcs(pcs, &mut accounting),
            IccProfileError::UnsupportedTransform,
        );
    }

    let first = [15797, 16384, 13515];
    let last = [31596, 32768, 27031];
    let tags = rgb_tags([first, first, last], [curve(&[]), curve(&[]), curve(&[])]);
    let bytes = profile(V4, b"mntr", b"RGB ", b"XYZ ", D50, tags);
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    let rgb = profile.device_color(&[0.25, 0.5, 0.0]).unwrap();
    let pcs = profile.to_pcs(RELATIVE, &rgb, &mut accounting).unwrap();
    close(
        &pcs.coordinates(),
        &[11847.75 / 65536.0, 0.1875, 10136.25 / 65536.0],
    );
    assert_icc_error(
        profile.from_pcs(pcs, &mut accounting),
        IccProfileError::UnsupportedTransform,
    );
}

#[test]
fn rgb_matrix_runs_after_channel_curves_and_before_inverse_channel_curves() {
    let bytes = profile(V2, b"mntr", b"RGB ", b"XYZ ", D50, calibrated_rgb_tags());
    let mut accounting = RenderAccountingContext::default();
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    let rgb = profile.device_color(&[0.5, 0.75, 0.25]).unwrap();
    let pcs = profile.to_pcs(RELATIVE, &rgb, &mut accounting).unwrap();
    // gamma2, identity, gamma2 give linear [1/4, 3/4, 1/16].
    close(&pcs.coordinates(), &RGB_PCS);

    // F.7–F.16 give linear [-.25, 1.25, .25] before clipping, then
    // [0, 1, .25] before the three inverse curves.
    let pcs = IccPcsColor::try_new(
        IccPcsSpace::XyzD50,
        [63190.0 / 262144.0, 0.625, 86829.0 / 262144.0],
        RELATIVE,
    )
    .unwrap();
    close(
        profile.from_pcs(pcs, &mut accounting).unwrap().channels(),
        &[0.0, 1.0, 0.5],
    );
}

#[test]
fn v2_registry_profile_id_is_opaque_and_reserved_tail_stays_zero() {
    let mut bytes = profile(V2, b"mntr", b"RGB ", b"XYZ ", D50, calibrated_rgb_tags());
    // The ICC Registry's sRGB2014 and PSOcoated_v3 v2 profiles populate
    // the same 16-byte profile identifier slot as v4 profiles.
    bytes[84..100].copy_from_slice(&[7; 16]);
    let mut accounting = RenderAccountingContext::default();
    assert!(IccProfile::inspect(&bytes, &mut accounting).is_ok());
    bytes[100] = 1;
    assert_icc_error(
        IccProfile::inspect(&bytes, &mut accounting),
        IccProfileError::InvalidProfile,
    );
}

#[test]
fn sampled_inverse_preparation_refusal_releases_scratch_and_retries() {
    let samples: Vec<_> = (0..=64).map(|index| 512_u16 * index).collect();
    let bytes = gray_profile(V4, b"XYZ ", curve(&samples));
    let options =
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
    let mut accounting = RenderAccountingContext::new(options);
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    let target =
        IccPcsColor::try_new(IccPcsSpace::XyzD50, [0.0, 8192.0 / 65535.0, 0.0], RELATIVE).unwrap();
    accounting.set_options(
        options.with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::CoordinateVisits, 24)
                .unwrap(),
        ),
    );

    let error = profile.from_pcs(target, &mut accounting).unwrap_err();
    let terminal = accounting.take_last_attempt().unwrap();
    let report = terminal.attempt().unwrap();
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(_))
    ));
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    assert_eq!(report.phase_count(RenderPhase::IccProfileConvert), 0);
    assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(terminal.cleanup_fault(), None);

    accounting.set_options(options);
    close(
        profile
            .from_pcs(target, &mut accounting)
            .unwrap()
            .channels(),
        &[0.25],
    );
}

#[test]
fn rgb_display_absolute_uses_v2_media_white_without_readapting_v4_colorants() {
    let mut accounting = RenderAccountingContext::default();
    let mut adaptation = b"sf32\0\0\0\0".to_vec();
    // V4 colorants already reflect this synthetic nonidentity adaptation.
    for coefficient in [131072_i32, 0, 0, 0, 65536, 0, 0, 0, 65536] {
        adaptation.extend(coefficient.to_be_bytes());
    }
    let mut v4_tags = calibrated_rgb_tags();
    v4_tags.push((b"chad", adaptation));

    for (version, white, tags, x_scale) in [
        (V2, [D50[0] / 2, D50[1], D50[2]], calibrated_rgb_tags(), 0.5),
        (V4, D50, v4_tags, 1.0),
    ] {
        let bytes = profile(version, b"mntr", b"RGB ", b"XYZ ", white, tags);
        let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
        let device = [0.5, 0.75, 0.25];
        let rgb = profile.device_color(&device).unwrap();
        let relative = profile.to_pcs(RELATIVE, &rgb, &mut accounting).unwrap();
        close(&relative.coordinates(), &RGB_PCS);
        let absolute = profile.to_pcs(ABSOLUTE, &rgb, &mut accounting).unwrap();
        assert_eq!(absolute.intent(), ABSOLUTE);
        let expected = [RGB_PCS[0] * x_scale, RGB_PCS[1], RGB_PCS[2]];
        samples_match(&absolute.coordinates(), &expected, 2.0e-6);
        let independent = IccPcsColor::try_new(IccPcsSpace::XyzD50, expected, ABSOLUTE).unwrap();
        let rgb = profile.from_pcs(independent, &mut accounting).unwrap();
        samples_match(rgb.channels(), &device, 4.0e-6);
    }
}

#[test]
fn selected_malformed_gray_curve_has_a_typed_error_in_both_directions() {
    let mut malformed = curve(&[0, 65535]);
    malformed.truncate(malformed.len() - 2);
    let bytes = gray_profile(V4, b"XYZ ", malformed);
    let mut accounting = RenderAccountingContext::default();
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    let gray = profile.device_color(&[0.5]).unwrap();
    assert_icc_error(
        profile.to_pcs(RELATIVE, &gray, &mut accounting),
        IccProfileError::InvalidProfile,
    );
    let xyz = IccPcsColor::try_new(IccPcsSpace::XyzD50, [0.1, 0.25, 0.2], RELATIVE).unwrap();
    assert_icc_error(
        profile.from_pcs(xyz, &mut accounting),
        IccProfileError::InvalidProfile,
    );
}

#[test]
fn model_fallback_respects_class_pcs_and_edition() {
    let mut accounting = RenderAccountingContext::default();
    for (version, class, device, space, eligible) in [
        (V2, b"scnr", b"GRAY", b"Lab ", false),
        (V4, b"prtr", b"GRAY", b"Lab ", true),
        (V4, b"mntr", b"RGB ", b"Lab ", false),
        (V2, b"scnr", b"3CLR", b"XYZ ", true),
        (V2, b"mntr", b"3CLR", b"XYZ ", false),
        (V4, b"mntr", b"3CLR", b"XYZ ", true),
        (V4, b"spac", b"RGB ", b"XYZ ", false),
    ] {
        let (tags, control, expected) = if device == b"GRAY" {
            (vec![(b"kTRC", curve(&[512]))], &[0.5][..], [25.0, 0.0, 0.0])
        } else {
            (calibrated_rgb_tags(), &[0.5, 0.75, 0.25][..], RGB_PCS)
        };
        let bytes = profile(version, class, device, space, D50, tags);
        let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
        let sample = profile.device_color(control).unwrap();
        let forward = profile.to_pcs(RELATIVE, &sample, &mut accounting);
        let pcs = IccPcsColor::try_new(profile.pcs_space(), expected, RELATIVE).unwrap();
        let reverse = profile.from_pcs(pcs, &mut accounting);
        if eligible {
            close(&forward.unwrap().coordinates(), &expected);
            close(reverse.unwrap().channels(), control);
        } else {
            assert_icc_error(forward, IccProfileError::UnsupportedTransform);
            assert_icc_error(reverse, IccProfileError::UnsupportedTransform);
        }
    }
}

fn lut16_constant(inputs: u8, codes: &[u16]) -> Vec<u8> {
    let mut data = b"mft2\0\0\0\0".to_vec();
    data.extend([inputs, u8::try_from(codes.len()).unwrap(), 2, 0]);
    for coefficient in [65536_i32, 0, 0, 0, 65536, 0, 0, 0, 65536] {
        data.extend(coefficient.to_be_bytes());
    }
    data.extend([0, 2, 0, 2]);
    for _ in 0..inputs {
        data.extend([0, 0, 255, 255]);
    }
    for _ in 0..(1 << inputs) {
        for code in codes {
            data.extend(code.to_be_bytes());
        }
    }
    for _ in codes {
        data.extend([0, 0, 255, 255]);
    }
    data
}

#[test]
fn selected_luts_take_precedence_over_the_gray_curve_in_both_directions() {
    let tags = vec![
        (b"kTRC", curve(&[512])),
        (b"A2B1", lut16_constant(1, &[8192, 16384, 24576])),
        (b"B2A1", lut16_constant(3, &[16384])),
    ];
    let bytes = profile(V4, b"scnr", b"GRAY", b"XYZ ", D50, tags);
    let mut accounting = RenderAccountingContext::default();
    let profile = IccProfile::inspect(&bytes, &mut accounting).unwrap();
    let gray = profile.device_color(&[0.5]).unwrap();
    let pcs = profile.to_pcs(RELATIVE, &gray, &mut accounting).unwrap();
    close(&pcs.coordinates(), &[0.25, 0.5, 0.75]);
    let pcs = IccPcsColor::try_new(IccPcsSpace::XyzD50, [0.1, 0.25, 0.2], RELATIVE).unwrap();
    close(
        profile.from_pcs(pcs, &mut accounting).unwrap().channels(),
        &[16384.0 / 65535.0],
    );
}
