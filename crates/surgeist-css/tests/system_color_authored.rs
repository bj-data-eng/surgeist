#![forbid(unsafe_code)]
//! Symbolic system colors from CSS Color 4 CRD 2026-09-08, section 6.2 and Appendix A.
//!
//! The explicit keyword table comes from the pinned specification, independently
//! of the parser and serializer. Host palette selection and deprecated alias
//! computed-value mappings belong downstream; authored aliases remain distinct.

use surgeist_css::{
    CssAbsoluteColorEligibility, CssAbsoluteColorExclusion, CssColor, CssKnownProperty,
    CssKnownPropertyValueRef, CssRecoveryAction, CssSystemColor, parse_style_attribute,
    validate_style_attribute,
};

const SYSTEM_COLORS: [(&str, CssSystemColor, &str); 19] = [
    ("AccentColor", CssSystemColor::AccentColor, "accentcolor"),
    (
        "AccentColorText",
        CssSystemColor::AccentColorText,
        "accentcolortext",
    ),
    ("ActiveText", CssSystemColor::ActiveText, "activetext"),
    ("ButtonBorder", CssSystemColor::ButtonBorder, "buttonborder"),
    ("ButtonFace", CssSystemColor::ButtonFace, "buttonface"),
    ("ButtonText", CssSystemColor::ButtonText, "buttontext"),
    ("Canvas", CssSystemColor::Canvas, "canvas"),
    ("CanvasText", CssSystemColor::CanvasText, "canvastext"),
    ("Field", CssSystemColor::Field, "field"),
    ("FieldText", CssSystemColor::FieldText, "fieldtext"),
    ("GrayText", CssSystemColor::GrayText, "graytext"),
    ("Highlight", CssSystemColor::Highlight, "highlight"),
    (
        "HighlightText",
        CssSystemColor::HighlightText,
        "highlighttext",
    ),
    ("LinkText", CssSystemColor::LinkText, "linktext"),
    ("Mark", CssSystemColor::Mark, "mark"),
    ("MarkText", CssSystemColor::MarkText, "marktext"),
    ("SelectedItem", CssSystemColor::SelectedItem, "selecteditem"),
    (
        "SelectedItemText",
        CssSystemColor::SelectedItemText,
        "selecteditemtext",
    ),
    ("VisitedText", CssSystemColor::VisitedText, "visitedtext"),
];

// Appendix A in https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#deprecated-system-colors
// requires support for these keywords. This table records the authored aliases,
// not their downstream mappings to non-deprecated system colors.
const DEPRECATED_SYSTEM_COLORS: [(&str, CssSystemColor, &str); 23] = [
    ("ActiveBorder", CssSystemColor::ActiveBorder, "activeborder"),
    (
        "ActiveCaption",
        CssSystemColor::ActiveCaption,
        "activecaption",
    ),
    ("AppWorkspace", CssSystemColor::AppWorkspace, "appworkspace"),
    ("Background", CssSystemColor::Background, "background"),
    (
        "ButtonHighlight",
        CssSystemColor::ButtonHighlight,
        "buttonhighlight",
    ),
    ("ButtonShadow", CssSystemColor::ButtonShadow, "buttonshadow"),
    ("CaptionText", CssSystemColor::CaptionText, "captiontext"),
    (
        "InactiveBorder",
        CssSystemColor::InactiveBorder,
        "inactiveborder",
    ),
    (
        "InactiveCaption",
        CssSystemColor::InactiveCaption,
        "inactivecaption",
    ),
    (
        "InactiveCaptionText",
        CssSystemColor::InactiveCaptionText,
        "inactivecaptiontext",
    ),
    (
        "InfoBackground",
        CssSystemColor::InfoBackground,
        "infobackground",
    ),
    ("InfoText", CssSystemColor::InfoText, "infotext"),
    ("Menu", CssSystemColor::Menu, "menu"),
    ("MenuText", CssSystemColor::MenuText, "menutext"),
    ("Scrollbar", CssSystemColor::Scrollbar, "scrollbar"),
    (
        "ThreeDDarkShadow",
        CssSystemColor::ThreeDDarkShadow,
        "threeddarkshadow",
    ),
    ("ThreeDFace", CssSystemColor::ThreeDFace, "threedface"),
    (
        "ThreeDHighlight",
        CssSystemColor::ThreeDHighlight,
        "threedhighlight",
    ),
    (
        "ThreeDLightShadow",
        CssSystemColor::ThreeDLightShadow,
        "threedlightshadow",
    ),
    ("ThreeDShadow", CssSystemColor::ThreeDShadow, "threedshadow"),
    ("Window", CssSystemColor::Window, "window"),
    ("WindowFrame", CssSystemColor::WindowFrame, "windowframe"),
    ("WindowText", CssSystemColor::WindowText, "windowtext"),
];

fn parsed_color(value: &str) -> CssColor {
    let source = format!("color: {value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1, "{source}");
    let CssKnownPropertyValueRef::Color(color) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected a typed color: {source}");
    };
    let color = color.value().clone();
    let validated = validate_style_attribute(&source).unwrap();
    assert_eq!(&validated, report.syntax(), "{source}");
    color
}

fn assert_symbolic(color: &CssColor, expected: CssSystemColor, text: &str) {
    assert_eq!(color.system(), Some(expected), "{text}");
    assert!(color.named().is_none(), "{text}");
    assert!(color.rgb_value().is_none(), "{text}");
    assert_eq!(
        color.absolute_eligibility(),
        CssAbsoluteColorEligibility::Contextual(CssAbsoluteColorExclusion::SystemColor),
        "{text}",
    );
    assert_eq!(color.to_specified_css().unwrap(), text);
}

#[test]
fn every_current_system_keyword_remains_symbolic_in_each_ascii_case() {
    for (authored, expected, canonical) in SYSTEM_COLORS {
        for spelling in [
            authored.to_owned(),
            authored.to_ascii_uppercase(),
            canonical.to_owned(),
        ] {
            assert_symbolic(&parsed_color(&spelling), expected, canonical);
        }
    }
}

#[test]
fn constructed_system_colors_preserve_identity_without_a_host_palette() {
    for (_, expected, canonical) in SYSTEM_COLORS {
        // Observe the constructor directly before any serialization/reparse.
        assert_symbolic(&CssColor::from_system(expected), expected, canonical);
    }
}

#[test]
fn escaped_system_identifiers_keep_the_same_symbolic_identity() {
    for (authored, expected, canonical) in SYSTEM_COLORS {
        let escaped = format!("\\{:x} {}", authored.as_bytes()[0], &authored[1..]);
        assert_symbolic(&parsed_color(&escaped), expected, canonical);
    }
}

#[test]
fn every_deprecated_system_keyword_retains_its_authored_alias_in_each_ascii_case() {
    for (authored, expected, canonical) in DEPRECATED_SYSTEM_COLORS {
        for spelling in [
            authored.to_owned(),
            authored.to_ascii_uppercase(),
            canonical.to_owned(),
        ] {
            assert_symbolic(&parsed_color(&spelling), expected, canonical);
        }
    }
}

#[test]
fn constructed_deprecated_system_colors_retain_the_alias_without_a_host_palette() {
    for (_, expected, canonical) in DEPRECATED_SYSTEM_COLORS {
        // Observe the constructor directly before any serialization/reparse.
        assert_symbolic(&CssColor::from_system(expected), expected, canonical);
    }
}

#[test]
fn escaped_deprecated_system_identifiers_retain_the_authored_alias() {
    for (authored, expected, canonical) in DEPRECATED_SYSTEM_COLORS {
        let escaped = format!("\\{:x} {}", authored.as_bytes()[0], &authored[1..]);
        assert_symbolic(&parsed_color(&escaped), expected, canonical);
    }
}

#[test]
fn similar_unknown_keywords_are_recovered_without_losing_the_valid_sibling() {
    for invalid in [
        "CanvasText1",
        "Canvas-Text",
        "AccentColors",
        "SelectedItemTexts",
        "GrayTextual",
        "Canvas Text",
    ] {
        let source = format!("color: {invalid}; opacity: 0.5");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration,
            "{source}",
        );
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Opacity,
            "{source}",
        );
        assert!(validate_style_attribute(&source).is_err(), "{source}");
    }
}
