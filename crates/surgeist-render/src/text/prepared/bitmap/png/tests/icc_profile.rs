use super::*;
use crate::{
    accounting::{RenderAccountingOptions, RenderLimits, RenderQuantity},
    tests::icc_png::{image, lut_profile, matrix_profile},
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
