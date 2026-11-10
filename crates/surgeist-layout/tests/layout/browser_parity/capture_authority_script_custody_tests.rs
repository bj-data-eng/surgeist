fn script_custody_flex_terminal_helper_controls(request: &str, helpers: &[(&str, bool)]) {
    let (report, xml) = match request {
        "prepared-ranges" => (
            include_str!("captured/prepared-range-differences/xml/generation-reports/all.json"),
            include_bytes!("captured/prepared-range-differences/xml/block/prepared_text_continuity_bidi__border_box_ltr.xml").as_slice(),
        ),
        "flex-composition" => (
            include_str!("captured/flex-composition-differences/xml/generation-reports/all.json"),
            include_bytes!("captured/flex-composition-differences/xml/flex/fri07_flex_composition__border_box_ltr.xml").as_slice(),
        ),
        _ => unreachable!(),
    };
    let report = unique_json(report).unwrap();
    for (helper, admitted) in helpers {
        let mut fixture = Fixture::new(&"a".repeat(64));
        fixture.metadata = serde_json::from_value(report["metadata"].clone()).unwrap();
        fixture.artifact = serde_json::from_value(report["generated"][0].clone()).unwrap();
        fixture.case.name.clone_from(&fixture.artifact.name);
        fixture.case.source.clone_from(&fixture.artifact.source);
        fixture.case.variant.clone_from(&fixture.artifact.variant);
        fixture
            .case
            .output
            .path
            .clone_from(&fixture.artifact.output);
        fixture
            .case
            .output
            .sha256
            .clone_from(&fixture.artifact.output_sha256);
        fixture.xml = sha256(xml);
        fixture.inputs = fixture.metadata.inputs.clone();
        fixture
            .inputs
            .insert(ROOT_HELPER_INPUT.to_owned(), (*helper).to_owned());
        fixture.metadata.inputs = fixture.inputs.clone();
        let request_sha = fixture.inputs[OBSERVATION_REQUEST].clone();
        let result = fixture.with_observation(request, Some(&request_sha));
        assert_eq!(
            result.is_ok(),
            *admitted,
            "{request}: {helper}: {:?}",
            result.as_ref().err()
        );
        if let Ok(verified) = result {
            assert_eq!(verified.projection().inputs[ROOT_HELPER_INPUT], *helper);
        }
    }
}

#[test]
fn script_custody_flex_terminal_retained_helpers_keep_their_profile_boundaries() {
    const RETAINED: &str = "afad9adf5a3915d88e94b0e3ad2d4768c242783422a0fb3bbc9a277492c10f9c";
    const RANGE: &str = "bd1b351e488f846b2e73947746ced8d8b290f49b108e220c228b2c4d94c069c0";
    const GEOMETRY: &str = "af0f458c290f0261106810fa3e42442afe7a5599872774130ac3244548cb74ee";
    script_custody_flex_terminal_helper_controls(
        "prepared-ranges",
        &[(RETAINED, true), (RANGE, true), (GEOMETRY, false)],
    );
    script_custody_flex_terminal_helper_controls(
        "flex-composition",
        &[(RETAINED, true), (RANGE, false), (GEOMETRY, false)],
    );
}

#[test]
fn script_custody_flex_terminal_admits_the_current_flex_producer() {
    script_custody_flex_terminal_helper_controls(
        "flex-composition",
        &[(
            "eef8d2108e8a1c50a510e283a97e0484a0b130cd7bcf69a790fd329e66b46686",
            true,
        )],
    );
}

#[test]
fn script_custody_flex_terminal_admits_the_current_prepared_range_producer() {
    script_custody_flex_terminal_helper_controls(
        "prepared-ranges",
        &[(
            "eef8d2108e8a1c50a510e283a97e0484a0b130cd7bcf69a790fd329e66b46686",
            true,
        )],
    );
}
