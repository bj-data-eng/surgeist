use super::*;
use crate::{
    accounting::{RenderAccountingOptions, RenderLimits, RenderQuantity},
    tests::icc_png::{gray_profile, image, lut_profile, matrix_profile, v2_matrix_profile},
};
use ::png::chunk::{cHRM, gAMA, iCCP, sRGB};

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

fn assert_diagnostic(bytes: &[u8], expected: GlyphPreparationError) {
    assert_eq!(
        test_decode(bytes, 1 << 20)
            .err()
            .expect("the PNG must refuse")
            .glyph_preparation_diagnostic(),
        Some(&expected)
    );
}

#[test]
fn selected_valid_lut_profile_refuses_despite_lower_profiles() {
    let profile = lut_profile();
    let bytes = png_with_lower_profiles(&profile);
    assert_diagnostic(&bytes, GlyphPreparationError::UnsupportedPng);
    let mut bad_png = bytes;
    *bad_png.last_mut().unwrap() ^= 1;
    assert_diagnostic(&bad_png, GlyphPreparationError::InvalidFont);
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

    // ICC.1:2022 §8.10.2 skips an optional multi-process route when the CMM
    // cannot execute it. The admitted P3 matrix therefore gives the same
    // independent oracle as the profile without that optional route.
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
