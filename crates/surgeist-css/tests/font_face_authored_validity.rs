//! Fonts 4 §4.1 gives @font-face a declaration-list body. Missing font-family
//! or src excludes the face from font matching; it does not invalidate its
//! authored rule. Unknown or invalid descriptors use declaration recovery.
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-rule

use surgeist_css::{CssRecoveryAction, CssRule, parse_sheet};

#[test]
fn missing_matching_descriptors_preserves_clean_authored_font_face() {
    for (source, occurrences) in [
        ("@font-face {}", 0),
        ("@font-face { font-family: Demo; }", 1),
        ("@font-face { src: url(face.woff2); }", 1),
        ("@font-face { font-display: swap; }", 1),
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("expected the authored font-face for {source}");
        };
        assert_eq!(rule.descriptors().occurrences().len(), occurrences);
        assert_eq!(
            rule.descriptors().font_family().is_some(),
            source.contains("font-family")
        );
        assert_eq!(rule.descriptors().src().is_some(), source.contains("src:"));
        assert_eq!(rule.position().byte_offset().value(), 0);
    }
}

#[test]
fn invalid_descriptor_does_not_discard_accepted_font_face() {
    let source = "@font-face { font-family: Demo; src: nope; font-display: swap; }";
    let report = parse_sheet(source);
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("descriptor recovery must retain the accepted outer rule");
    };
    assert_eq!(rule.descriptors().occurrences().len(), 2);
    assert_eq!(rule.descriptors().font_family().unwrap().as_str(), "Demo");
    assert!(rule.descriptors().src().is_none());
    let [diagnostic] = report.diagnostics() else {
        panic!("expected exactly the invalid src descriptor diagnostic");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
}

#[test]
fn invalid_font_face_prelude_still_discards_the_outer_rule() {
    let source = "@font-face unexpected { src: url(face); } .after { color: red; }";
    let report = parse_sheet(source);
    assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
    let [diagnostic] = report.diagnostics() else {
        panic!("expected exactly the invalid outer rule diagnostic");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
}
