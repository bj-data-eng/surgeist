use super::*;
use crate::{
    accounting::RenderAttemptStatus,
    tests::icc_png::{
        mpe_aliased_matrix_transform, mpe_clut_transform, mpe_curve_transform, mpe_matrix_profile,
        mpe_overflowing_matrix_transform, mpe_profile, unsupported_mpe_transform,
    },
    text::GlyphNumericPhase,
};

#[test]
fn selected_icc_mpe_xyz_overrides_legacy_and_lower_png_profiles() {
    let profile = mpe_matrix_profile();
    let decoded = test_decode(&png_with_lower_profiles(&profile), 1 << 20).unwrap();
    // ICC.1:2022 §10.16.2.3 produces direct D50 XYZ. The second pixel is
    // approximately [0.65061, 0.58174, 0.21686], which gives sRGB bytes
    // [254.079, 185.008, 128.690] before rounding using CSS Color 4.
    assert_eq!(
        decoded.pixels,
        [188, 188, 188, 0, 254, 185, 129, 128, 234, 207, 144, 255],
    );
}

#[test]
fn selected_icc_mpe_runs_aliased_elements_in_directory_order() {
    let profile = mpe_profile(
        mpe_aliased_matrix_transform(),
        b"XYZ ",
        lut16_transform(),
        None,
    );
    let source = [0, 40, 90, 7, 128, 30, 200, 128, 255, 0, 0, 255];
    let bytes = image(&source, Some(&profile), &[], &[]);
    // Halve RGB, map red plus 1/4 to a D50 neutral, and halve again.
    // Luminance is 1/8 + R/1020. The sRGB transfer gives 99.086, 137.083,
    // and 164.750 before rounding. Both half stages must run.
    assert_eq!(
        test_decode(&bytes, 1 << 20).unwrap().pixels,
        [99, 99, 99, 7, 137, 137, 137, 128, 165, 165, 165, 255],
    );
}

#[test]
fn selected_icc_mpe_numeric_overflow_releases_custody_before_publication() {
    let profile = mpe_profile(
        mpe_overflowing_matrix_transform(),
        b"XYZ ",
        lut16_transform(),
        None,
    );
    let source = [0, 200, 20, 7, 255, 30, 200, 128];
    let bytes = image(&source, Some(&profile), &[], &[]);
    let mut context = RenderAccountingContext::default();
    let error = decode_in(&bytes, 1 << 20, &mut context).err().unwrap();
    // Both matrices have finite coefficients. On the second pixel the
    // first gives f32::MAX and the second doubles it, exceeding binary32.
    assert_eq!(
        error.glyph_preparation_diagnostic(),
        Some(&GlyphPreparationError::Numeric {
            phase: GlyphNumericPhase::BitmapColor,
            value: 2.0 * f64::from(f32::MAX),
        }),
    );
    let terminal = context.last_attempt().unwrap();
    assert_eq!(
        terminal.status(),
        RenderAttemptStatus::Failed(crate::ErrorCode::GlyphPreparation),
    );
    assert_eq!(terminal.cleanup_fault(), None);
    let usage = terminal.attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::ImageTexelVisits), 2);
    assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);

    let retry = image(&source[..4], Some(&profile), &[], &[]);
    assert_eq!(
        decode_in(&retry, 1 << 20, &mut context).unwrap().pixels,
        [0, 0, 0, 7],
    );
}

#[test]
fn selected_icc_mpe_clamps_clut_inputs_and_preserves_extended_outputs_in_order() {
    let profile = mpe_profile(mpe_clut_transform(), b"XYZ ", lut16_transform(), None);
    let source = [
        0, 255, 64, 0, 255, 0, 200, 128, 96, 137, 222, 255, 192, 170, 68, 12, 255, 255, 255, 254,
    ];
    let bytes = image(&source, Some(&profile), &[], &[]);
    // ICC.1:2022 §10.16.2 defines the 3→2→3 channel chain and pass-through
    // elements. Let u=clip(2R/255−1/2) and v=clip(2G/255−1/2), with w
    // piecewise-linear through (v,w)=(0,0), (1/2,1/4), (1,1). The final
    // matrix gives XYZ=[1/4+u/8+w/8+uw/4, 3/8+v/8+u/16, 5/16+u/8+v/16].
    // The first pixel has u=0, v=1 and outputs [3/8, 1/2, 3/8] in XYZ.
    // Clipping the CLUT outputs would, for example, push all Y values to 1/4.
    assert_eq!(
        test_decode(&bytes, 1 << 20).unwrap().pixels,
        [
            118, 204, 177, 0, 138, 185, 194, 128, 112, 197, 179, 255, 248, 172, 205, 12, 255, 162,
            208, 254
        ],
    );
}

#[test]
fn selected_icc_mpe_evaluates_segmented_curves_to_direct_lab() {
    let profile = mpe_profile(mpe_curve_transform(), b"Lab ", lut16_lab_transform(), None);
    let source = [
        0, 0, 0, 0, 51, 173, 72, 128, 153, 230, 128, 255, 204, 96, 212, 1, 255, 255, 255, 254,
    ];
    let bytes = image(&source, Some(&profile), &[], &[]);
    // ICC.1:2022 §10.16.2.2 gives L* via points (0,20), (1/2,50), (1,75)
    // at x=1.25R/255, with the zero endpoint supplied by the earlier
    // formula. It gives a*=40 log10(0.9(G/255)^2+0.1)+20 and
    // b*=24·2^(−B/255)−10. The fourth pixel has x=1, which belongs
    // to the sampled segment. The last pixel has x>1 and L*=90.
    assert_eq!(
        test_decode(&bytes, 1 << 20).unwrap().pixels,
        [
            17, 56, 26, 0, 99, 77, 67, 128, 183, 140, 140, 255, 176, 188, 178, 1, 255, 213, 223,
            254
        ],
    );
}

#[test]
fn unsupported_icc_mpe_falls_through_to_a2b1_then_a2b0() {
    // A structurally valid element with an unknown signature skips D2B1
    // per ICC.1:2022 §§8.10.2 and 10.16.1.
    for (colorimetric, expected) in [
        (Some(lut_a_to_b_transform()), B_ONLY_XYZ),
        (None, LUT16_XYZ),
    ] {
        let profile = mpe_profile(
            unsupported_mpe_transform(),
            b"XYZ ",
            lut16_transform(),
            colorimetric,
        );
        let bytes = png_with_lower_profiles(&profile);
        assert_eq!(test_decode(&bytes, 1 << 20).unwrap().pixels, expected);
    }
}

#[test]
fn malformed_selected_icc_mpe_falls_back_to_srgb() {
    // Each fixture leaves the outer ICC and iCCP valid, with usable legacy
    // routes and lower PNG metadata. The only damage is inside D2B1.
    let clut_transform = mpe_clut_transform();
    let matrix_offset = be_offset(&clut_transform, 16);
    let begin_offset = be_offset(&clut_transform, 24);
    let clut_offset = be_offset(&clut_transform, 32);
    let mut out_of_bounds = clut_transform.clone();
    out_of_bounds[20..24].copy_from_slice(&u32::MAX.to_be_bytes());
    let mut unaligned = clut_transform.clone();
    unaligned[16..20].copy_from_slice(&u32::try_from(matrix_offset + 1).unwrap().to_be_bytes());
    let mut channel_mismatch = clut_transform.clone();
    channel_mismatch[begin_offset + 9] = 3;
    let mut one_point_grid = clut_transform.clone();
    one_point_grid[clut_offset + 12] = 1;
    let mut nan_coefficient = clut_transform.clone();
    nan_coefficient[matrix_offset + 12..matrix_offset + 16]
        .copy_from_slice(&f32::NAN.to_be_bytes());
    let mut denormal_coefficient = clut_transform;
    denormal_coefficient[matrix_offset + 12..matrix_offset + 16]
        .copy_from_slice(&1_u32.to_be_bytes()); // Smallest positive binary32 subnormal.
    let mut descending_breakpoints = mpe_curve_transform();
    let curves_offset = be_offset(&descending_breakpoints, 24);
    let lightness_offset = curves_offset + be_offset(&descending_breakpoints, curves_offset + 12);
    descending_breakpoints[lightness_offset + 12..lightness_offset + 16]
        .copy_from_slice(&2.0_f32.to_be_bytes());

    let mut failures = Vec::new();
    for (case, transform, pcs) in [
        ("out-of-range element extent", out_of_bounds, b"XYZ "),
        ("unaligned element offset", unaligned, b"XYZ "),
        ("channel count mismatch", channel_mismatch, b"XYZ "),
        ("one-point CLUT dimension", one_point_grid, b"XYZ "),
        ("NaN matrix coefficient", nan_coefficient, b"XYZ "),
        (
            "subnormal matrix coefficient",
            denormal_coefficient,
            b"XYZ ",
        ),
        (
            "descending curve breakpoints",
            descending_breakpoints,
            b"Lab ",
        ),
    ] {
        let profile = mpe_profile(
            transform,
            pcs,
            lut_a_to_b_transform(),
            Some(lut_a_to_b_transform()),
        );
        let bytes = image_with_profile_and_gamma(&profile);
        match test_decode(&bytes, 1 << 20) {
            Ok(decoded) if decoded.pixels == SOURCE => {}
            Ok(decoded) => failures.push(format!("{case}: decoded {:?}", decoded.pixels)),
            Err(error) => failures.push(format!("{case}: {error:?}")),
        }
    }
    assert!(
        failures.is_empty(),
        "malformed D2B1 must use sRGB: {failures:?}"
    );
}

fn be_offset(bytes: &[u8], field: usize) -> usize {
    u32::from_be_bytes(bytes[field..field + 4].try_into().unwrap()) as usize
}
