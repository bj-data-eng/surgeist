#![forbid(unsafe_code)]
//! Exact authored property provenance from the selected dated Sizing snapshots.

use surgeist_css::{
    CssKnownProperty, CssSupportStatus, feature_metadata, property_support_metadata,
};

fn assert_provenance(
    name: &str,
    feature_id: &str,
    source_id: &str,
    url: &str,
    property: CssKnownProperty,
) {
    let support = property_support_metadata(name).expect("known property support");
    assert_eq!(support.property(), property);
    let feature = support.feature();
    assert_eq!(feature.id().as_str(), feature_id);
    assert!(std::ptr::eq(feature, feature_metadata(feature_id).unwrap()));
    assert_eq!(feature.source().id().as_str(), source_id, "{name}");
    assert_eq!(feature.source().url(), Some(url), "{name}");
    assert_eq!(feature.production(), format!("#propdef-{name}"), "{name}");
    assert_eq!(feature.status(), CssSupportStatus::Complete, "{name}");
    assert_eq!(feature.supported_subset(), None, "{name}");
    assert_eq!(feature.unsupported_remainder(), None, "{name}");
    assert_eq!(feature.recognized_unsupported_code(), None, "{name}");
}

#[test]
fn box_sizing_reports_the_pinned_sizing3_source_and_complete_authored_grammar() {
    assert_provenance(
        "box-sizing",
        "baseline.property.box-sizing",
        "I-SIZING3-20260904",
        "https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/",
        CssKnownProperty::BoxSizing,
    );
}

#[test]
fn aspect_ratio_reports_the_pinned_sizing4_source_and_complete_authored_grammar() {
    assert_provenance(
        "aspect-ratio",
        "baseline.property.aspect-ratio",
        "X-SIZING4-20260904",
        "https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/",
        CssKnownProperty::AspectRatio,
    );
}
