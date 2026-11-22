use super::*;
use crate::{
    accounting::{RenderAccountingOptions, RenderLimits, RenderQuantity},
    tests::icc_png::{image, matrix_profile},
};

#[test]
fn png_phase_counts_profile_name_visits_with_the_other_png_bytes() {
    let mut iccp = b"Profile\0\0".to_vec();
    iccp.extend(zlib(&matrix_profile(false)));
    let png = image(
        &[128, 128, 128, 5],
        None,
        &[(::png::chunk::iCCP, &iccp)],
        &[],
    );
    let mut accounting = RenderAccountingContext::default();
    decode_in(&png, 1 << 20, &mut accounting).unwrap();
    let attempt = accounting.last_attempt().unwrap().attempt().unwrap();
    // Two CRC passes cover type and data of IHDR, iCCP, IDAT and IEND.
    // Only the first pass inspects the name and its NUL; reconstruction
    // visits each of the four pixel bytes once.
    let crc_bytes = 2 * (png.len() - 8 - 4 * 8) as u64;
    let expected = crc_bytes + 8 + 4;
    assert_eq!(attempt.usage().work(RenderWork::PngByteVisits), expected);
    assert_eq!(
        attempt
            .phase_usage(RenderPhase::Png)
            .work(RenderWork::PngByteVisits),
        expected
    );
}

#[test]
fn exact_inflate_budget_admits_profile_and_smaller_budget_releases_backing() {
    let profile = matrix_profile(false);
    let png = image(&[128, 128, 128, 5], Some(&profile), &[], &[]);
    let size = profile.len() as u64;
    let with_limit = |limit| {
        RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::IccProfileInflateBytes, limit)
                    .unwrap(),
            ),
        )
    };

    let mut refused = with_limit(size - 1);
    let error = decode_in(&png, 1 << 20, &mut refused).err().unwrap();
    assert!(error.accounting_diagnostic().is_some());
    let report = refused.last_attempt().unwrap();
    assert_eq!(report.cleanup_fault(), None);
    let attempt = report.attempt().unwrap();
    assert_eq!(
        attempt.refusal().unwrap().quantity(),
        RenderQuantity::Work(RenderWork::IccProfileInflateBytes)
    );
    let usage = attempt.usage();
    assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(usage.work(RenderWork::IccProfileTagVisits), 0);
    assert_eq!(usage.work(RenderWork::ImageTexelVisits), 0);
    assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);

    let mut admitted = with_limit(size);
    let decoded = decode_in(&png, 1 << 20, &mut admitted).unwrap();
    assert_eq!(decoded.pixels, [197, 135, 162, 5]);
    let report = admitted.last_attempt().unwrap().attempt().unwrap();
    let preparation = report.phase_usage(RenderPhase::PngColorPrepare);
    assert_eq!(preparation.work(RenderWork::IccProfileInflateBytes), size);
    assert!(preparation.work(RenderWork::IccProfileTagVisits) >= 10);
    assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
}
