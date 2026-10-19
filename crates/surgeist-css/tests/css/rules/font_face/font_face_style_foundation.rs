#![forbid(unsafe_code)]

//! Existing-public-API @font-face style grammar from CSS Fonts 4 WD
//! (2026-09-07) §4.4. Typed authored ranges follow with the model.

use surgeist_css::*;

type Kind = CssFontFaceDescriptorKind;

fn admitted(value: &str) {
    let raw = parse_font_face_descriptor_value(value, Kind::FontStyle);
    assert!(raw.is_clean(), "raw {value}: {:?}", raw.diagnostics());
    assert!(raw.syntax().is_some(), "raw descriptor: {value}");

    let source = format!("@font-face{{font-family:Demo;font-style:{value}}}");
    let report = parse_sheet(&source);
    assert!(
        report.is_clean(),
        "sheet {source}: {:?}",
        report.diagnostics()
    );
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("one font-face rule: {source}");
    };
    assert_eq!(rule.descriptors().occurrences().len(), 2);
    let record = rule.descriptors().effective(Kind::FontStyle).unwrap();
    assert_eq!(record.value().kind(), Kind::FontStyle);
    assert!(matches!(
        record.value(),
        CssAuthoredFontFaceDescriptorValue::Ordinary(_)
    ));
    assert_eq!(
        record.position().unwrap().byte_offset().value(),
        source.find("font-style:").unwrap()
    );
    assert!(validate_sheet(&source).is_ok());
}

#[test]
fn normal_italic_and_legacy_oblique_controls_remain_admitted() {
    for value in ["normal", "italic", "oblique", "oblique -10deg 20deg"] {
        admitted(value);
    }
}

#[test]
fn descriptor_accepts_auto_directions_bounded_single_and_authored_ranges() {
    for value in [
        "auto",
        "left",
        "right",
        "oblique 90deg",
        "oblique -90deg 90deg",
        "oblique 90deg -90deg",
        "oblique 100grad -.25turn",
        "oblique .25turn 1.5707963267948966rad",
        "oblique calc(100deg) 10deg",
    ] {
        admitted(value);
    }
}

#[test]
fn duplicate_style_occurrences_remain_ordered_with_last_effective() {
    let source = "@font-face{font-style:auto;font-style:oblique 90deg -90deg;font-style:left}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("one font-face rule");
    };
    assert_eq!(rule.descriptors().occurrences().len(), 3);
    assert_eq!(
        rule.descriptors()
            .effective(Kind::FontStyle)
            .unwrap()
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        source.rfind("font-style:").unwrap()
    );
    let absent = parse_sheet("@font-face{}");
    let [CssRule::FontFace(rule)] = absent.syntax().rules() else {
        panic!("empty font-face remains a rule");
    };
    assert!(rule.descriptors().effective(Kind::FontStyle).is_none());
}

#[test]
fn invalid_descriptor_angles_and_arity_drop_locally_with_original_position() {
    for invalid in [
        "oblique 90.00000000000000000001deg",
        "oblique 100.00000000000000000001grad",
        "oblique .25000000000000000001turn",
        "oblique 1.5707963267948968rad",
        "oblique 0",
        "oblique 1px",
        "oblique 10deg 20deg 30deg",
        "auto 10deg",
    ] {
        let raw = parse_font_face_descriptor_value(invalid, Kind::FontStyle);
        assert!(raw.syntax().is_none(), "raw accepted {invalid}: {raw:?}");
        assert!(!raw.is_clean());

        let source = format!(
            "@font-face{{font-family:Demo;font-style:normal;src:url(face);font-style:{invalid};font-display:swap}}"
        );
        let report = parse_sheet(&source);
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("font-face survives invalid descriptor: {source}");
        };
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid descriptor diagnostic: {source}: {report:?}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            source.find(&format!("font-style:{invalid}")).unwrap()
        );
        assert_eq!(rule.descriptors().occurrences().len(), 4);
        assert_eq!(
            rule.descriptors()
                .effective(Kind::FontStyle)
                .unwrap()
                .position()
                .unwrap()
                .byte_offset()
                .value(),
            source.find("font-style:normal").unwrap()
        );
        assert!(rule.descriptors().effective(Kind::FontDisplay).is_some());
        assert!(validate_sheet(&source).is_err());
    }
}

#[test]
fn valid_env_defers_whole_style_descriptor_without_relaxing_ordinary_grammar() {
    let source = "@font-face{font-style:normal;font-style:wide env(style)}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("one font-face rule");
    };
    assert_eq!(rule.descriptors().occurrences().len(), 2);
    assert!(matches!(
        rule.descriptors()
            .effective(Kind::FontStyle)
            .unwrap()
            .value(),
        CssAuthoredFontFaceDescriptorValue::Pending(_)
    ));
}
