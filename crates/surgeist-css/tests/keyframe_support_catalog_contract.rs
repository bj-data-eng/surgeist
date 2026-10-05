#![forbid(unsafe_code)]
//! The selected Animations 1 keyframe production owns authored grammar and recovery.
//! Its support catalog describes that production independently of execution.
use surgeist_css::*;

#[test]
fn keyframe_catalog_covers_the_selected_authored_rule_production() {
    let css = "@keyframes k{from{color:red}25%{opacity:.5}to{animation-timing-function:linear}}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(validate_sheet(css).is_ok());
    let [CssRule::Keyframes(rule)] = report.syntax().rules() else {
        panic!("one keyframe rule")
    };
    assert_eq!(rule.blocks().len(), 3);
    let metadata = feature_metadata("baseline.rule.keyframes").unwrap();
    assert_eq!(metadata.kind(), CssFeatureKind::Rule);
    assert_eq!(metadata.source().id().as_str(), "I-ANIMATIONS1");
    assert_eq!(metadata.status(), CssSupportStatus::Complete);
    assert_eq!(metadata.supported_subset(), None);
    assert_eq!(metadata.unsupported_remainder(), None);
    assert_eq!(metadata.recognized_unsupported_code(), None);
}

#[test]
fn existing_font_face_catalog_remains_complete_for_its_authored_grammar() {
    let css = "@font-face { font-family: Inter; src: url(inter.woff2); }";
    assert!(parse_sheet(css).is_clean());
    assert!(validate_sheet(css).is_ok());
    let metadata = feature_metadata("baseline.rule.font-face").unwrap();
    assert_eq!(metadata.kind(), CssFeatureKind::Rule);
    assert_eq!(metadata.status(), CssSupportStatus::Complete);
    assert_eq!(metadata.supported_subset(), None);
    assert_eq!(metadata.unsupported_remainder(), None);
}
