#![forbid(unsafe_code)]

//! Existing-public-API @font-face width contracts from CSS Fonts 4 WD
//! (2026-09-07) §§4.4 and 4.4.1. Exact typed ranges follow with the model.

use surgeist_css::*;

fn retained_count(source: &str) -> usize {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("one font-face rule: {source}")
    };
    assert_eq!(rule.position().byte_offset().value(), 0);
    assert!(
        validate_sheet(source).is_ok(),
        "strict validation: {source}"
    );
    rule.descriptors().occurrences().len()
}

#[test]
fn existing_stretch_descriptor_keyword_and_percentage_range_are_retained() {
    let source = concat!(
        "@font-face{font-family:Demo;src:url(face);",
        "font-stretch:condensed;font-stretch:75% 125%}"
    );
    assert_eq!(retained_count(source), 4);
}

#[test]
fn both_descriptor_names_accept_auto_single_and_two_width_values() {
    for name in ["font-width", "font-stretch", "FONT-WIDTH", "FONT-STRETCH"] {
        for value in [
            "auto",
            "normal",
            "condensed",
            "0%",
            "75%",
            "125.125%",
            "75% 125%",
            "condensed expanded",
            "condensed 125%",
            "125% condensed",
            "125% 75%",
            "expanded condensed",
        ] {
            let source = format!("@font-face{{font-family:Demo;src:url(face);{name}:{value}}}");
            assert_eq!(retained_count(&source), 3, "one width occurrence: {source}");
        }
    }
}

#[test]
fn descriptor_aliases_share_one_ordered_occurrence_family() {
    let source = concat!(
        "@font-face{font-family:Demo;src:url(face);",
        "font-stretch:condensed 125%;font-width:auto;",
        "FONT-STRETCH:125% 75%;font-width:expanded condensed}"
    );
    assert_eq!(
        retained_count(source),
        6,
        "all four width occurrences remain in order"
    );
}

#[test]
fn invalid_width_descriptor_drops_only_its_own_occurrence_at_its_source_position() {
    for (name, invalid) in [
        ("font-width", "auto 75%"),
        ("font-stretch", "75% auto"),
        ("font-width", "75% 100% 125%"),
        ("font-stretch", "-1%"),
        ("font-width", "0"),
    ] {
        let source = format!(
            "@font-face{{font-family:Demo;src:url(face);font-stretch:condensed;{name}:{invalid};font-width:75%}}"
        );
        let report = parse_sheet(&source);
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("font-face survives invalid descriptor: {source}")
        };
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid descriptor diagnostic: {source}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        let invalid_start = source.find(&format!("{name}:{invalid}")).unwrap();
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            invalid_start
        );
        assert_eq!(
            rule.descriptors().occurrences().len(),
            4,
            "valid neighbors survive"
        );
        assert!(validate_sheet(&source).is_err());
    }
}
