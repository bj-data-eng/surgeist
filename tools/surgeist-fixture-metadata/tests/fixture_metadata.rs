use surgeist_fixture_metadata::{
    FixtureMetadataError, FixtureMetadataRequest, INLINE_METRICS_SCHEMA_LIMITATION,
    generate_layout_ready_metadata,
};
use surgeist_test::fixtures::{
    FixtureExpectation, FixtureStatusKind, LayoutReadyFixtureMetadata, SchemaVersion,
};

fn request(css_source: &str) -> FixtureMetadataRequest {
    FixtureMetadataRequest {
        identity: "inline_card".to_string(),
        source_path: "css/inline-card.css".to_string(),
        generated_path: "layout-ready/inline-card.metadata".to_string(),
        css_source: css_source.to_string(),
        expectation: Some("matches-layout-ready-contract".to_string()),
    }
}

#[test]
fn generated_metadata_conforms_to_public_layout_ready_schema() {
    let metadata = generate_layout_ready_metadata(request(
        "fixture { display: block; width: 320px; font-size: 18px; line-height: 24px; }",
    ))
    .expect("valid CSS should produce metadata");

    assert_schema_conformance(&metadata);
    assert_eq!(metadata.identity().as_str(), "inline_card");
    assert_eq!(
        metadata.artifacts().source().as_str(),
        "css/inline-card.css"
    );
    assert_eq!(
        metadata.artifacts().generated().as_str(),
        "layout-ready/inline-card.metadata"
    );
    assert_eq!(
        metadata.expectation().map(FixtureExpectation::as_str),
        Some("matches-layout-ready-contract")
    );
    assert_eq!(
        metadata.provenance().generator(),
        "surgeist-fixture-metadata"
    );
    assert_eq!(
        metadata.provenance().adapter(),
        Some("root css-style/style-text/style-layout adapters")
    );
}

#[test]
fn invalid_css_property_value_is_rejected_before_metadata_output() {
    let error = generate_layout_ready_metadata(request("fixture { opacity: 2; width: 320px; }"))
        .expect_err("invalid style value should prevent metadata generation");

    assert!(matches!(error, FixtureMetadataError::Adapter(_)));
}

#[test]
fn current_schema_limitation_for_inline_metrics_is_documented() {
    assert!(INLINE_METRICS_SCHEMA_LIMITATION.contains("inline metric fields"));

    let metadata =
        generate_layout_ready_metadata(request("fixture { font-size: 18px; line-height: 24px; }"))
            .expect("schema-conforming metadata should still be generated");

    assert_schema_conformance(&metadata);
}

fn assert_schema_conformance(metadata: &LayoutReadyFixtureMetadata) {
    assert_eq!(metadata.schema_version(), SchemaVersion::current());
    assert_eq!(metadata.status().kind(), FixtureStatusKind::Ready);
    assert!(metadata.status().is_ready());
}
