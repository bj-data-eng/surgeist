//! The selected published Fonts4 grammar replaces the earlier source-list
//! production; immutable source identities must keep their original edition.
//! Complete support is still unjustified: selected font-face descriptors remain
//! missing, and legacy variation format strings lack their specified equivalent
//! format/technology projection (Fonts4 section 4.3.1's compatibility table).

use surgeist_css::{CssSupportStatus, feature_metadata, specification_source};

const SELECTED_ID: &str = "I-FONTS4-20260907";
const SELECTED_URL: &str = "https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/";

#[test]
fn selected_font_source_records_reference_the_pinned_published_edition() {
    for (id, production) in [
        ("baseline.rule.font-face", "#font-face-rule"),
        ("baseline.descriptor.src", "#font-face-src-parsing"),
        ("official.value.font-source", "#font-face-src-parsing"),
        (
            "ext.value.font-source-modern-hints",
            "#font-face-src-parsing",
        ),
    ] {
        let feature = feature_metadata(id).expect("existing conformance identity");
        assert_eq!(feature.source().id().as_str(), SELECTED_ID, "{id}");
        assert_eq!(feature.source().url(), Some(SELECTED_URL), "{id}");
        assert_eq!(feature.production(), production, "{id}");
        assert_eq!(feature.status(), CssSupportStatus::Partial, "{id}");
        assert!(feature.supported_subset().is_some(), "{id}");
        assert!(feature.unsupported_remainder().is_some(), "{id}");
    }
}

#[test]
fn font_source_editions_have_distinct_immutable_registry_identities() {
    let selected = specification_source(SELECTED_ID).expect("selected dated source");
    assert_eq!(selected.url(), Some(SELECTED_URL));
    assert_eq!(selected.module(), "CSS Fonts");
    assert_eq!(selected.level(), "4");
    let previous = specification_source("I-FONTS4").expect("preserved earlier source");
    assert_eq!(
        previous.url(),
        Some("https://www.w3.org/TR/2026/WD-css-fonts-4-20260422/")
    );
    assert_ne!(previous.id(), selected.id());
}

#[test]
fn the_superseded_fonts3_source_identity_remains_available() {
    assert_eq!(
        specification_source("O-FONTS3").unwrap().url(),
        Some("https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/")
    );
}
