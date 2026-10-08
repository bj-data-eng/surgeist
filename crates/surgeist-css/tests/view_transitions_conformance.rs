#![forbid(unsafe_code)]

use surgeist_css::*;

// The selected VT1 CRD20240328 and CSS Snapshot 2026 stable-module
// classification are the independent provenance authority for these records.
#[test]
fn name_and_pseudo_catalog_records_bind_clean_admission_to_the_selected_stable_source() {
    let cases = [
        (
            "official.property.view-transition-name",
            CssFeatureKind::Property,
            "#propdef-view-transition-name",
            ".x{view-transition-name:Card}",
        ),
        (
            "official.pseudo-element.view-transition",
            CssFeatureKind::Selector,
            "#selectordef-view-transition,#pseudo-root",
            "::view-transition{color:red}",
        ),
        (
            "official.pseudo-element.view-transition-group",
            CssFeatureKind::Selector,
            "#selectordef-view-transition-group,#pseudo-root",
            "::view-transition-group(*):only-child{color:red}",
        ),
        (
            "official.pseudo-element.view-transition-image-pair",
            CssFeatureKind::Selector,
            "#selectordef-view-transition-image-pair,#pseudo-root",
            "::view-transition-image-pair(none):only-child{color:red}",
        ),
        (
            "official.pseudo-element.view-transition-old",
            CssFeatureKind::Selector,
            "#selectordef-view-transition-old,#pseudo-root",
            "::view-transition-old(auto):only-child{color:red}",
        ),
        (
            "official.pseudo-element.view-transition-new",
            CssFeatureKind::Selector,
            "#selectordef-view-transition-new,#pseudo-root",
            "::view-transition-new(Card):only-child{color:red}",
        ),
    ];
    for (id, kind, production, css) in cases {
        let metadata = feature_metadata(id).unwrap();
        assert_eq!(metadata.kind(), kind);
        assert_eq!(metadata.production(), production);
        assert_eq!(metadata.status(), CssSupportStatus::Complete);
        assert_eq!(metadata.source().id().as_str(), "S-VIEWTRANSITIONS1");
        assert_eq!(
            metadata.source().tier(),
            CssSpecificationTier::Snapshot2026Stable
        );
        assert_eq!(
            metadata.source().url(),
            Some("https://www.w3.org/TR/2024/CRD-css-view-transitions-1-20240328/")
        );
        let report = parse_sheet(css);
        assert!(report.is_clean(), "{id}: {report:?}");
        assert_eq!(report.syntax().rules().len(), 1);
    }
}
