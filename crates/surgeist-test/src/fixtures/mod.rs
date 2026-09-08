//! Shared fixture schemas and harness support.

pub mod layout_ready;

pub use layout_ready::{
    ArtifactPaths, CURRENT_SCHEMA_VERSION, FixtureExpectation, FixtureIdentity, FixturePath,
    FixtureStatus, FixtureStatusKind, GeneratorProvenance, LayoutReadyFixtureMetadata, SchemaError,
    SchemaVersion,
};

#[cfg(test)]
mod tests {
    use super::{
        ArtifactPaths, FixtureExpectation, FixtureIdentity, FixturePath, FixtureStatus,
        GeneratorProvenance, LayoutReadyFixtureMetadata, SchemaVersion,
    };

    #[test]
    fn layout_ready_metadata_constructor_sets_schema_and_keeps_paths_relative() {
        let metadata = LayoutReadyFixtureMetadata::new(
            FixtureIdentity::new("block_basic").expect("fixture identity should be valid"),
            ArtifactPaths::new(
                FixturePath::new("html/block/basic.html").expect("source path should be valid"),
                FixturePath::new("xml/block/basic.xml").expect("generated path should be valid"),
            ),
            GeneratorProvenance::new("root-layout-fixture-generator")
                .expect("generator name should be valid")
                .with_adapter("root-composed-style-layout-adapter")
                .expect("adapter should be valid"),
        )
        .with_expectation(
            FixtureExpectation::new("matches-browser-border-box")
                .expect("expectation label should be valid"),
        )
        .with_status(FixtureStatus::ready());

        assert_eq!(metadata.schema_version(), SchemaVersion::current());
        assert_eq!(metadata.identity().as_str(), "block_basic");
        assert_eq!(
            metadata.artifacts().source().as_str(),
            "html/block/basic.html"
        );
        assert_eq!(
            metadata.artifacts().generated().as_str(),
            "xml/block/basic.xml"
        );
        assert_eq!(
            metadata.provenance().generator(),
            "root-layout-fixture-generator"
        );
        assert_eq!(
            metadata.provenance().adapter(),
            Some("root-composed-style-layout-adapter")
        );
        assert_eq!(
            metadata.expectation().map(FixtureExpectation::as_str),
            Some("matches-browser-border-box")
        );
        assert!(metadata.status().is_ready());
    }

    #[test]
    fn fixture_paths_are_fixture_relative() {
        assert_eq!(
            FixturePath::new("xml/block/basic.xml")
                .expect("relative path should be valid")
                .as_str(),
            "xml/block/basic.xml"
        );

        assert!(FixturePath::new("").is_err());
        assert!(FixturePath::new("/tmp/basic.xml").is_err());
        assert!(FixturePath::new("../layout/basic.xml").is_err());
    }

    #[test]
    fn status_reasons_must_explain_non_ready_fixtures() {
        assert!(FixtureStatus::ignored("depends on unsupported writing mode").is_ok());
        assert!(FixtureStatus::known_failure("browser rounds differently").is_ok());

        assert!(FixtureStatus::ignored("   ").is_err());
        assert!(FixtureStatus::known_failure("").is_err());
    }

    #[test]
    fn expectation_labels_must_be_non_empty() {
        assert!(FixtureExpectation::new("matches-browser-border-box").is_ok());
        assert!(FixtureExpectation::new(" ").is_err());
    }
}
