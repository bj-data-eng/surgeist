#![forbid(unsafe_code)]
// Snapshot 2026 section 4 selects conic gradients and gradient interpolation;
// the selected Images 4 stop-list grammar is their bounded shared dependency.
use surgeist_css::{CssSpecificationTier, feature_metadata};

fn selected_gradient_source(feature: &str) {
    let source = feature_metadata(feature)
        .expect("selected gradient metadata")
        .source();
    assert_eq!(
        source.tier(),
        CssSpecificationTier::Snapshot2026PreCrException
    );
    assert_eq!(
        source.url(),
        Some("https://www.w3.org/TR/2025/WD-css-images-4-20250930/")
    );
}

#[test]
fn selected_conic_gradient_has_snapshot_feature_provenance() {
    selected_gradient_source("ext.value.conic-gradient");
}

#[test]
fn selected_gradient_interpolation_has_snapshot_feature_provenance() {
    selected_gradient_source("ext.value.gradient-interpolation");
}

#[test]
fn selected_gradient_stop_list_has_snapshot_feature_provenance() {
    selected_gradient_source("ext.value.gradient-stop-list");
}

#[test]
fn ui4_image_imports_retain_their_distinct_extension_provenance() {
    for feature in ["ext.value.image-1d", "ext.value.stripes"] {
        let source = feature_metadata(feature).unwrap().source();
        assert_eq!(source.id().as_str(), "X-IMAGES4-20250930");
        assert_eq!(source.tier(), CssSpecificationTier::SurgeistExtension);
        assert_eq!(
            source.url(),
            Some("https://www.w3.org/TR/2025/WD-css-images-4-20250930/")
        );
    }
}
