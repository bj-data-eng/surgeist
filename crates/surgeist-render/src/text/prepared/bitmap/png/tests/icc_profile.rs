use super::*;
use crate::{
    accounting::{RenderAccountingOptions, RenderLimits, RenderQuantity},
    tests::icc_png::{
        LabStages, colorimetric_lut_profile, gray_profile, image, lut_a_to_b_transform,
        lut_profile, lut_profile_with_transform, lut8_transform, lut16_lab_transform,
        lut16_transform, matrix_and_lut_profile, matrix_profile, staged_lab_transform,
        v2_matrix_profile,
    },
};
use ::png::chunk::{cHRM, gAMA, iCCP, sRGB};

mod multi_process;

const SOURCE: [u8; 12] = [128, 128, 128, 0, 230, 128, 64, 128, 200, 160, 80, 255];
const POWER: [u8; 12] = [197, 135, 162, 0, 255, 130, 89, 128, 240, 165, 107, 255];
const SAMPLED: [u8; 12] = [197, 135, 136, 0, 255, 130, 89, 128, 240, 165, 99, 255];

fn png_with_lower_profiles(profile: &[u8]) -> Vec<u8> {
    // A deliberately different gamma and primaries expose lower-profile
    // combination. sRGB and iCCP coexistence is discouraged, with iCCP
    // precedence specified by PNG Third Edition §§4.3 and 11.3.2.
    let swapped: Vec<_> = [31270_u32, 32900, 15000, 6000, 30000, 60000, 64000, 33000]
        .into_iter()
        .flat_map(u32::to_be_bytes)
        .collect();
    image(
        &SOURCE,
        Some(profile),
        &[
            (gAMA, &100000_u32.to_be_bytes()),
            (cHRM, &swapped),
            (sRGB, &[0]),
        ],
        &[],
    )
}

#[test]
fn selected_icc_matrix_curves_override_lower_profiles_and_preserve_alpha() {
    // Independent oracle from Display-P3 -> XYZ D65 -> linear sRGB using CSS
    // Color 4's rational matrices, followed by standard sRGB encoding,
    // destination clipping and byte rounding. For example, RGB=(128,128,128)
    // with blue x^1.5 gives [196.977, 134.787, 161.920] before rounding.
    // Confirmed separately with the fixture's quantized PCS coefficients.
    for (sampled, expected) in [(false, POWER), (true, SAMPLED)] {
        let profile = matrix_profile(sampled);
        for bytes in [
            image(&SOURCE, Some(&profile), &[], &[]),
            png_with_lower_profiles(&profile),
        ] {
            let decoded = test_decode(&bytes, 1 << 20).unwrap();
            assert_eq!((decoded.width, decoded.height), (3, 1));
            assert_eq!(decoded.pixels, expected);
        }
    }
}

#[test]
fn selected_icc_v2_evaluates_identity_gamma_and_sampled_curves() {
    let source = [128, 140, 128, 0, 230, 142, 64, 128, 200, 139, 80, 255];
    let bytes = image(&source, Some(&v2_matrix_profile()), &[], &[]);
    // The calibrated source primaries match destination sRGB primaries.
    // Independent sRGB encoding of identity, square, and piecewise-linear
    // channel values gives the literal bytes below. For the first pixel,
    // the unrounded bytes are [187.845, 149.198, 137.699].
    assert_eq!(
        test_decode(&bytes, 1 << 20).unwrap().pixels,
        [188, 149, 138, 0, 244, 151, 99, 128, 229, 148, 110, 255]
    );
}

#[test]
fn gray_icc_profile_for_rgba_selects_untagged_srgb() {
    // PNG Third Edition §11.3.2.3 requires an RGB profile for color type 6.
    // This is a valid monochrome ICC profile embedded in a mismatched image.
    let bytes = image(
        &SOURCE,
        Some(&gray_profile()),
        &[(gAMA, &100000_u32.to_be_bytes())],
        &[],
    );
    assert_eq!(test_decode(&bytes, 1 << 20).unwrap().pixels, SOURCE);
}

#[test]
fn invalid_selected_icc_uses_full_srgb_fallback() {
    let mut bad_signature = matrix_profile(false);
    bad_signature[36..40].copy_from_slice(b"acsp".map(|value| value ^ 1).as_slice());
    let mut bad_size = matrix_profile(false);
    let wrong_size = u32::try_from(bad_size.len() - 4).unwrap().to_be_bytes();
    bad_size[..4].copy_from_slice(&wrong_size);
    let mut duplicate = matrix_profile(false);
    duplicate[144..148].copy_from_slice(b"rXYZ");
    let mut overlapping = matrix_profile(false);
    let first_offset = u32::from_be_bytes(overlapping[136..140].try_into().unwrap());
    overlapping[148..152].copy_from_slice(&(first_offset + 4).to_be_bytes());
    for (case, profile) in [
        ("header signature", bad_signature),
        ("declared size", bad_size),
        ("duplicate tag", duplicate),
        ("partial overlap", overlapping),
    ] {
        let bytes = image(
            &SOURCE,
            Some(&profile),
            &[(gAMA, &100000_u32.to_be_bytes())],
            &[],
        );
        assert_eq!(
            test_decode(&bytes, 1 << 20).unwrap().pixels,
            SOURCE,
            "{case}"
        );
    }
}

#[test]
fn selected_icc_v2_negative_xyz_column_falls_back_to_srgb() {
    // ICC.1:2001-04 §§5.3.10 and 6.5.26 require nonnegative XYZ values.
    let mut profile = v2_matrix_profile();
    let red = tag_offset(&profile, b"rXYZ");
    profile[red + 16..red + 20].copy_from_slice(&(-6554_i32).to_be_bytes());
    let bytes = image_with_profile_and_gamma(&profile);
    assert_eq!(test_decode(&bytes, 1 << 20).unwrap().pixels, SOURCE);
}

#[test]
fn selected_icc_v4_monitor_with_non_d50_white_falls_back_to_srgb() {
    // ICC.1:2022 §9.2.36 requires a display media white point of PCS D50.
    let mut profile = matrix_profile(false);
    profile[12..16].copy_from_slice(b"mntr");
    let valid = image_with_profile_and_gamma(&profile);
    assert_eq!(test_decode(&valid, 1 << 20).unwrap().pixels, POWER);
    let white = tag_offset(&profile, b"wtpt");
    profile[white + 16..white + 20].copy_from_slice(&65536_u32.to_be_bytes());
    let bytes = image_with_profile_and_gamma(&profile);
    assert_eq!(test_decode(&bytes, 1 << 20).unwrap().pixels, SOURCE);
}

fn tag_offset(profile: &[u8], signature: &[u8; 4]) -> usize {
    let count = u32::from_be_bytes(profile[128..132].try_into().unwrap()) as usize;
    let entry = profile[132..132 + count * 12]
        .as_chunks::<12>()
        .0
        .iter()
        .find(|entry| &entry[..4] == signature)
        .unwrap();
    u32::from_be_bytes(entry[4..8].try_into().unwrap()) as usize
}

fn image_with_profile_and_gamma(profile: &[u8]) -> Vec<u8> {
    image(
        &SOURCE,
        Some(profile),
        &[(gAMA, &100000_u32.to_be_bytes())],
        &[],
    )
}

fn assert_diagnostic(bytes: &[u8], expected: GlyphPreparationError) {
    assert_eq!(
        test_decode(bytes, 1 << 20)
            .err()
            .expect("the PNG must refuse")
            .glyph_preparation_diagnostic(),
        Some(&expected)
    );
}

// LUT color oracles use ICC.1:2022 Tables 11–13 and §§10.10–10.12,
// followed by the CSS Color 4 Lab/XYZ and Bradford/sRGB equations.
// <https://www.color.org/specifications/ICC.1-2022-05.pdf>
// <https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#color-conversion-code>
const LUT16_XYZ: [u8; 12] = [188, 188, 188, 0, 254, 185, 129, 128, 234, 207, 144, 255];
const B_ONLY_XYZ: [u8; 12] = [255, 252, 255, 0, 255, 116, 204, 128, 255, 242, 219, 255];

#[test]
fn selected_icc_lut16_xyz_overrides_lower_png_profiles_and_preserves_alpha() {
    let mut bad_png = png_with_lower_profiles(&lut_profile());
    *bad_png.last_mut().unwrap() ^= 1;
    assert_diagnostic(&bad_png, GlyphPreparationError::InvalidFont);

    // The eight corners are sums of the fixture's P3 primaries encoded as
    // u1Fixed15. For RGB=(230,128,64), their trilinear blend gives destination
    // bytes [254.060, 185.011, 128.826] before rounding.
    for version in [2, 4] {
        let profile = colorimetric_lut_profile(lut16_transform(), b"XYZ ", version);
        let bytes = png_with_lower_profiles(&profile);
        assert_eq!(
            test_decode(&bytes, 1 << 20).unwrap().pixels,
            LUT16_XYZ,
            "ICC v{version}"
        );
    }
}

#[test]
fn selected_icc_lut8_evaluates_neutral_lab_in_v2_and_v4() {
    // Trilinear interpolation gives L*=100(R+G+B)/(3*255), a*=b*=0.
    // The second pixel maps to 132.038 per channel before byte rounding.
    for version in [2, 4] {
        let profile = colorimetric_lut_profile(lut8_transform(), b"Lab ", version);
        assert_eq!(
            test_decode(&png_with_lower_profiles(&profile), 1 << 20)
                .unwrap()
                .pixels,
            [119, 119, 119, 0, 132, 132, 132, 128, 138, 138, 138, 255],
            "ICC v{version}"
        );
    }
}

#[test]
fn selected_icc_lut8_xyz_has_no_standard_pcs_encoding() {
    // ICC.1:2022 §10.11 leaves the 8-bit PCSXYZ encoding undefined.
    let profile = lut_profile_with_transform(lut8_transform(), b"XYZ ");
    assert_diagnostic(
        &png_with_lower_profiles(&profile),
        GlyphPreparationError::UnsupportedPng,
    );
}

#[test]
fn selected_icc_lut16_lab_applies_both_tables_and_uses_legacy_encoding() {
    let source = [
        130, 128, 128, 0, 230, 64, 40, 128, 202, 160, 80, 255, 255, 0, 0, 1, 0, 0, 0, 254,
    ];
    // In §10.10 the lightness and neutral codes are ff00 and 8000. The
    // fixture gives [100*R', 40*G', -32*B'] where R', G', B' are the input
    // table outputs. The last two pixels map to Lab white and black.
    for version in [2, 4] {
        let profile = colorimetric_lut_profile(lut16_lab_transform(), b"Lab ", version);
        let bytes = image(&source, Some(&profile), &[], &[]);
        assert_eq!(
            test_decode(&bytes, 1 << 20).unwrap().pixels,
            [
                83, 51, 88, 0, 229, 207, 223, 128, 206, 152, 187, 255, 255, 255, 255, 1, 0, 0, 0,
                254
            ],
            "ICC v{version}"
        );
    }
}

#[test]
fn selected_icc_a_to_b_evaluates_b_only_xyz_curves() {
    let profile = colorimetric_lut_profile(lut_a_to_b_transform(), b"XYZ ", 4);
    // Identity B curves give X, Y, Z = RGB/255 × (65535/32768).
    let decoded = test_decode(&png_with_lower_profiles(&profile), 1 << 20).unwrap();
    assert_eq!(decoded.pixels, B_ONLY_XYZ);
}

#[test]
fn selected_icc_a_to_b_evaluates_the_final_curve_before_outer_padding() {
    // The last B curve has a 16-bit gamma. Its two alignment bytes follow
    // the complete mAB payload and are excluded from the tag's declared size.
    let mut transform = lut_a_to_b_transform();
    transform[64..68].copy_from_slice(&1_u32.to_be_bytes());
    transform.extend_from_slice(&0x0200_u16.to_be_bytes());
    let profile = colorimetric_lut_profile(transform, b"XYZ ", 4);
    // Only the Z coordinate is squared before its u1Fixed15 scaling.
    assert_eq!(
        test_decode(&png_with_lower_profiles(&profile), 1 << 20)
            .unwrap()
            .pixels,
        [255, 250, 196, 0, 255, 112, 78, 128, 255, 241, 90, 255],
    );
}

#[test]
fn selected_icc_a_to_b_prefers_colorimetric_then_perceptual_then_matrix() {
    // ICC.1:2022 §8.10.2 ranks A2B1, A2B0 and matrix/TRC for this intent.
    for (profile, expected) in [
        (matrix_and_lut_profile(true), B_ONLY_XYZ),
        (matrix_and_lut_profile(false), LUT16_XYZ),
        (matrix_profile(true), SAMPLED),
    ] {
        let bytes = png_with_lower_profiles(&profile);
        assert_eq!(test_decode(&bytes, 1 << 20).unwrap().pixels, expected);
    }
}

#[test]
fn selected_icc_a_to_b_interpolates_asymmetric_clut_after_a_curves_and_before_b_curves() {
    let source = [
        96, 64, 222, 0, 192, 168, 68, 128, 250, 244, 194, 255, 0, 255, 0, 12, 255, 255, 255, 254,
    ];
    let profile = colorimetric_lut_profile(staged_lab_transform(LabStages::Clut), b"Lab ", 4);
    let bytes = image(&source, Some(&profile), &[], &[]);
    // With u=(R/255)^2, v=2G/255 and w=B/255, trilinear interpolation
    // gives the raw L coordinate (36 + 46uv + 46w + 24v)/255. The B curve
    // squares it before Lab decoding. A simplex blend changes the uv term.
    assert_eq!(
        test_decode(&bytes, 1 << 20).unwrap().pixels,
        [
            18, 38, 14, 0, 69, 40, 46, 128, 221, 121, 117, 255, 16, 32, 29, 12, 255, 154, 138, 254
        ],
    );
}

#[test]
fn selected_icc_a_to_b_clips_matrix_output_between_m_and_b_curves() {
    let source = [
        60, 230, 36, 0, 170, 84, 170, 128, 245, 230, 10, 255, 250, 30, 190, 1,
    ];
    let profile = colorimetric_lut_profile(staged_lab_transform(LabStages::Matrix), b"Lab ", 4);
    let bytes = image(&source, Some(&profile), &[], &[]);
    // M squares the first input. The matrix then produces 2(R/255)^2−1/4,
    // which falls below 0 in the first pixel and above 1 in the last two.
    // §10.12.5 clips each result before the B curve squares L again.
    assert_eq!(
        test_decode(&bytes, 1 << 20).unwrap().pixels,
        [
            0, 0, 79, 0, 75, 102, 96, 128, 255, 241, 255, 255, 212, 255, 223, 1
        ],
    );
}

#[test]
fn selected_icc_a_to_b_runs_all_five_stages_with_full_clut_precision() {
    let source = [
        96, 64, 220, 0, 192, 170, 68, 128, 250, 244, 194, 255, 0, 255, 0, 12,
    ];
    let profile = colorimetric_lut_profile(staged_lab_transform(LabStages::Full), b"Lab ", 4);
    let bytes = image(&source, Some(&profile), &[], &[]);
    // Add 199/65535 to each interpolated CLUT component, then apply M,
    // matrix, clipping and B. The second pixel's blue is 38.950 before
    // rounding. Discarding those low CLUT bits changes its rounded value.
    assert_eq!(
        test_decode(&bytes, 1 << 20).unwrap().pixels,
        [0, 3, 23, 0, 0, 8, 39, 128, 255, 250, 255, 255, 0, 3, 27, 12],
    );
}

#[test]
fn selected_icc_legacy_luts_with_inconsistent_tables_fall_back_to_srgb() {
    // ICC.1:2022 Tables 40–44 require complete tables for the encoded
    // channel, grid and entry counts, beyond just the fixed tag prefix.
    let mut truncated_8 = lut8_transform();
    truncated_8.pop().unwrap();
    let mut invalid_grid_8 = lut8_transform();
    invalid_grid_8[10] = 1;
    let mut truncated_16 = lut16_transform();
    truncated_16.truncate(truncated_16.len() - 2);
    let mut invalid_input_count_16 = lut16_transform();
    invalid_input_count_16[48..50].copy_from_slice(&1_u16.to_be_bytes());
    let mut invalid_channels_16 = lut16_transform();
    invalid_channels_16[8] = 2; // Header RGB requires three input channels.
    assert_invalid_lut_fallback([
        ("mft1 short output table", truncated_8, b"Lab "),
        ("mft1 one-point grid", invalid_grid_8, b"Lab "),
        ("mft2 short output table", truncated_16, b"XYZ "),
        ("mft2 invalid entry count", invalid_input_count_16, b"XYZ "),
        ("mft2 wrong input channels", invalid_channels_16, b"XYZ "),
    ]);
}

#[test]
fn selected_icc_a_to_b_with_malformed_curve_set_falls_back_to_srgb() {
    // ICC.1:2022 Table 45 and §10.12.6 require an aligned B-curve offset
    // and one complete embedded curve for every output channel.
    let mut missing = lut_a_to_b_transform();
    missing[12..16].fill(0);
    missing.truncate(32);
    let mut past_end = lut_a_to_b_transform();
    let outside_tag = u32::try_from(past_end.len() + 4).unwrap();
    past_end[12..16].copy_from_slice(&outside_tag.to_be_bytes());
    let mut unaligned = lut_a_to_b_transform();
    unaligned[12..16].copy_from_slice(&33_u32.to_be_bytes());
    let mut truncated = lut_a_to_b_transform();
    truncated.truncate(truncated.len() - 4);
    let mut invalid_type = lut_a_to_b_transform();
    invalid_type[56..60].copy_from_slice(b"XYZ ");
    assert_invalid_lut_fallback([
        ("absent B curves", missing, b"XYZ "),
        ("B offset outside tag", past_end, b"XYZ "),
        ("unaligned B offset", unaligned, b"XYZ "),
        ("short third B curve", truncated, b"XYZ "),
        ("wrong third B curve type", invalid_type, b"XYZ "),
    ]);
}

fn assert_invalid_lut_fallback<const N: usize>(cases: [(&str, Vec<u8>, &[u8; 4]); N]) {
    let mut failures = Vec::new();
    for (case, transform, pcs) in cases {
        // Rebuild the outer ICC tag table and PNG iCCP chunk so the only
        // invalid data is in the selected LUT payload.
        let profile = lut_profile_with_transform(transform, pcs);
        let bytes = image_with_profile_and_gamma(&profile);
        match test_decode(&bytes, 1 << 20) {
            Ok(decoded) if decoded.pixels == SOURCE => {}
            Ok(decoded) => failures.push(format!("{case}: decoded {:?}", decoded.pixels)),
            Err(error) => failures.push(format!("{case}: {error:?}")),
        }
    }
    assert!(
        failures.is_empty(),
        "malformed selected ICC LUT must use untagged sRGB: {failures:?}"
    );
}

#[test]
fn invalid_iccp_framing_is_a_png_error() {
    let mut framed = b"Profile\0\0".to_vec();
    framed.extend(zlib(&matrix_profile(false)));
    let mut image_with_bad_crc = image(&SOURCE, None, &[(iCCP, &framed)], &[]);
    // IHDR takes 25 bytes after the 8-byte signature. iCCP then has its
    // 8-byte prefix, the payload, and its four-byte CRC.
    image_with_bad_crc[8 + 25 + 8 + framed.len()] ^= 1;
    for bytes in [
        image_with_bad_crc,
        image(&SOURCE, None, &[(iCCP, &[0; 4])], &[]),
        image(&SOURCE, None, &[(iCCP, b"Bad  name\0\0")], &[]),
        image(&SOURCE, None, &[(iCCP, b"Bad\0\x01data")], &[]),
        image(&SOURCE, None, &[(iCCP, &framed), (iCCP, &framed)], &[]),
        image(&SOURCE, None, &[], &[(iCCP, &framed)]),
    ] {
        assert_diagnostic(&bytes, GlyphPreparationError::InvalidFont);
    }
}

#[test]
fn invalid_iccp_zlib_refuses_with_valid_chunk_crc_and_releases_custody() {
    let stream = zlib(&matrix_profile(false));
    let mut valid = b"Profile\0\0".to_vec();
    valid.extend_from_slice(&stream);
    let mut corrupt = valid.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    let mut trailing = valid.clone();
    trailing.push(0);
    let mut concatenated = valid.clone();
    concatenated.extend_from_slice(&stream);
    let truncated = &valid[..valid.len() - 1];
    let mut context = RenderAccountingContext::default();
    let retry = image(&SOURCE, None, &[(iCCP, &valid)], &[]);
    for (case, payload) in [
        ("wrong Adler", corrupt.as_slice()),
        ("missing trailer byte", truncated),
        ("trailing byte", trailing.as_slice()),
        ("second stream", concatenated.as_slice()),
    ] {
        // The PNG encoder writes a fresh CRC for each damaged zlib payload.
        let bytes = image(&SOURCE, None, &[(iCCP, payload)], &[]);
        let error = decode_in(&bytes, 1 << 20, &mut context).err().unwrap();
        assert_eq!(
            error.glyph_preparation_diagnostic(),
            Some(&GlyphPreparationError::InvalidFont),
            "{case}"
        );
        let report = context.last_attempt().unwrap();
        assert_eq!(report.cleanup_fault(), None, "{case}");
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0, "{case}");
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0, "{case}");
        assert_eq!(
            usage.work(RenderWork::PreparedTransferredBytes),
            0,
            "{case}"
        );
        assert_eq!(
            decode_in(&retry, 1 << 20, &mut context).unwrap().pixels,
            POWER
        );
    }
}

#[test]
fn selected_icc_charges_texel_work_and_releases_failed_preparation_before_retry() {
    let profile = matrix_profile(false);
    let bytes = image(&SOURCE, Some(&profile), &[], &[]);
    let mut context = RenderAccountingContext::new(
        RenderAccountingOptions::new().with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::ImageTexelVisits, 1)
                .unwrap(),
        ),
    );
    let error = decode_in(&bytes, 1 << 20, &mut context).err().unwrap();
    assert!(error.accounting_diagnostic().is_some(), "{error:?}");
    let report = context.last_attempt().unwrap();
    assert_eq!(report.cleanup_fault(), None);
    let attempt = report.attempt().unwrap();
    assert_eq!(
        attempt.refusal().unwrap().quantity(),
        RenderQuantity::Work(RenderWork::ImageTexelVisits)
    );
    assert_eq!(
        attempt
            .phase_usage(RenderPhase::PngColorConvert)
            .work(RenderWork::ImageTexelVisits),
        1
    );
    let usage = attempt.usage();
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);

    let small = image(&SOURCE[..4], Some(&profile), &[], &[]);
    assert_eq!(
        decode_in(&small, 1 << 20, &mut context).unwrap().pixels,
        POWER[..4]
    );
}

#[test]
fn selected_icc_piecewise_display_p3_executes_both_transfer_branches() {
    use crate::tests::icc_png::piecewise_p3_profile;

    let source = [
        180, 90, 130, 127, 32, 128, 200, 0, 255, 160, 0, 134, 10, 11, 0, 1,
    ];
    let png = image(&source, Some(&piecewise_p3_profile(false)), &[], &[]);
    let decoded = test_decode(&png, 1 << 20).unwrap();
    // Independent CSS Color 4 Display P3 to sRGB calculation uses the same
    // piecewise transfer on both sides of the sRGB matrix conversion. The
    // final source pixel straddles the low and high branches at 10 and 11.
    assert_eq!(
        decoded.pixels,
        [
            193, 83, 131, 127, 0, 130, 206, 0, 255, 154, 0, 134, 10, 11, 0, 1
        ]
    );
}

#[test]
fn icc_matrix_fallback_runs_when_the_optional_multi_process_route_is_unavailable() {
    use crate::tests::icc_png::piecewise_p3_profile;

    // ICC.1:2022 §§8.10.2 and 10.16.1 skip a multi-process route with an
    // unknown element signature. This profile falls back to the P3 matrix.
    let png = image(
        &[180, 90, 130, 127],
        Some(&piecewise_p3_profile(true)),
        &[],
        &[],
    );
    assert_eq!(
        test_decode(&png, 1 << 20).unwrap().pixels,
        [193, 83, 131, 127]
    );
}
