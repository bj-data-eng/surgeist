fn script_custody_admission_requires_candidate(
    request: BoxObservationRequest,
    source_path: &str,
    source: &str,
    source_sha256: &str,
    refusal: &str,
) {
    use surgeist_generator::Sha256Digest;

    assert_eq!(
        Sha256Digest::from_bytes(TEST_HELPER_SOURCE.as_bytes()).as_str(),
        "eb92590a7b4ae6bb9ef10f64148e7fa4e7ff0a6f6bccf1abf7eceea0fc44b0e4"
    );
    assert_eq!(
        Sha256Digest::from_bytes(source.as_bytes()).as_str(),
        source_sha256
    );
    let source_path = RelativePath::new(source_path).unwrap();
    let setup = |helper: &str| {
        helper_setup_expression(
            helper,
            include_bytes!("../../layout/browser_parity/fonts/Ahem.ttf"),
            source,
            TEST_BASE_STYLE_SOURCE,
            &source_path,
            request,
        )
    };
    let retained = include_str!(
        "../../layout/browser_parity/captured/original-subgrid-body-qualification/scripts/gentest/test_helper.js"
    );
    setup(retained).expect("the retained producer remains admissible");
    for other_profile_helper in [
        include_str!(
            "../../layout/browser_parity/captured/canonical-box-differences/scripts/gentest/test_helper.js"
        ),
        include_str!(
            "../../layout/browser_parity/captured/prepared-range-differences/scripts/gentest/test_helper.js"
        ),
    ] {
        assert!(
            matches!(setup(other_profile_helper), Err(AdapterError::Preparation(reason)) if reason == refusal)
        );
    }
    let modified = format!("{TEST_HELPER_SOURCE}\ndescribeElement = () => null;\n");
    assert!(matches!(
        setup(&modified),
        Err(AdapterError::Preparation(reason)) if reason == refusal
    ));
    setup(TEST_HELPER_SOURCE).expect("the script-custody helper must preserve fresh profile setup");
}

#[test]
fn script_custody_admission_original_subgrid_accepts_candidate_and_refuses_mutation() {
    script_custody_admission_requires_candidate(
        BoxObservationRequest::OriginalSubgridBody,
        original_subgrid_body_profile::SOURCE,
        include_str!(
            "../../layout/browser_parity/html/subgrid/subgrid_gap_percentage_border_bottom.html"
        ),
        original_subgrid_body_profile::SOURCE_SHA256,
        "original subgrid body capture helper changed",
    );
}

#[test]
fn script_custody_admission_atomic_percentage_accepts_candidate_and_refuses_mutation() {
    script_custody_admission_requires_candidate(
        BoxObservationRequest::PreparedAtomicPercentage,
        prepared_atomic_percentage_profile::SOURCE,
        include_str!(
            "../../layout/browser_parity/html/block/fri06_atomic_inline_percentage_block_size.html"
        ),
        prepared_atomic_percentage_profile::SOURCE_SHA256,
        "prepared atomic percentage capture helper changed",
    );
}

#[test]
fn script_custody_admission_horizontal_br_accepts_candidate_and_refuses_mutation() {
    script_custody_admission_requires_candidate(
        BoxObservationRequest::HorizontalBrFontBand,
        horizontal_br_font_band_profile::SOURCE,
        include_str!("../../layout/browser_parity/html/block/block_br_inline_block_metrics.html"),
        horizontal_br_font_band_profile::SOURCE_SHA256,
        "horizontal break font-band capture helper changed",
    );
}

#[test]
fn script_custody_admission_vertical_br_accepts_candidate_and_refuses_mutation() {
    let sources = [
        include_str!(
            "../../layout/browser_parity/html/block/block_br_vertical_lr_inline_block_metrics.html"
        ),
        include_str!(
            "../../layout/browser_parity/html/block/block_br_vertical_rl_inline_block_metrics.html"
        ),
        include_str!(
            "../../layout/browser_parity/html/block/block_br_vertical_rl_rtl_inline_block_metrics.html"
        ),
    ];
    assert_eq!(vertical_br_font_band_profile::SOURCES.len(), sources.len());
    for (source_path, source) in vertical_br_font_band_profile::SOURCES
        .iter()
        .copied()
        .zip(sources)
    {
        script_custody_admission_requires_candidate(
            BoxObservationRequest::VerticalBrFontBand,
            source_path,
            source,
            vertical_br_font_band_profile::source_digest(source_path).unwrap(),
            "vertical break font-band capture helper changed",
        );
    }
}

#[test]
fn script_custody_admission_empty_br_accepts_candidate_and_refuses_mutation() {
    let sources = [
        include_str!("../../layout/browser_parity/html/block/block_br_empty_lines_metrics.html"),
        include_str!(
            "../../layout/browser_parity/html/block/block_br_vertical_rl_empty_lines_metrics.html"
        ),
    ];
    assert_eq!(
        empty_line_br_font_band_profile::SOURCES.len(),
        sources.len()
    );
    for (source_path, source) in empty_line_br_font_band_profile::SOURCES
        .iter()
        .copied()
        .zip(sources)
    {
        script_custody_admission_requires_candidate(
            BoxObservationRequest::EmptyLineBrFontBand,
            source_path,
            source,
            empty_line_br_font_band_profile::source_digest(source_path).unwrap(),
            "empty-line break capture helper changed",
        );
    }
}
