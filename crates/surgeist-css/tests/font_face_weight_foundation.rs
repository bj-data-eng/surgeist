#![forbid(unsafe_code)]

//! Existing-public-API @font-face weight grammar from CSS Fonts 4 WD
//! (2026-09-07) §4.4. Typed endpoint contracts follow with the model.

use surgeist_css::*;

type Kind = CssFontFaceDescriptorKind;

fn admitted(value: &str) {
    let raw = parse_font_face_descriptor_value(value, Kind::FontWeight);
    assert!(raw.is_clean(), "raw {value}: {:?}", raw.diagnostics());
    assert!(raw.syntax().is_some(), "raw descriptor: {value}");

    let source = format!("@font-face{{font-family:Demo;font-weight:{value}}}");
    let report = parse_sheet(&source);
    assert!(
        report.is_clean(),
        "sheet {source}: {:?}",
        report.diagnostics()
    );
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("one font-face rule: {source}");
    };
    let records: Vec<_> = rule.descriptors().occurrences().collect();
    assert_eq!(records.len(), 2, "family plus weight: {source}");
    let record = rule.descriptors().effective(Kind::FontWeight).unwrap();
    assert_eq!(record.value().kind(), Kind::FontWeight);
    assert!(matches!(
        record.value(),
        CssAuthoredFontFaceDescriptorValue::Ordinary(_)
    ));
    assert_eq!(
        record.position().unwrap().byte_offset().value(),
        source.find("font-weight:").unwrap()
    );
    assert!(validate_sheet(&source).is_ok());
}

#[test]
fn existing_keywords_integer_singles_and_numeric_pair_remain_admitted() {
    for value in ["normal", "bold", "1", "1000", "100 900"] {
        admitted(value);
    }
}

#[test]
fn auto_and_exact_single_absolute_weights_are_admitted() {
    for value in ["auto", "350.5", "1e0", "1e3", "calc(350 + 50)"] {
        admitted(value);
    }
    let absent = parse_sheet("@font-face{font-family:Demo}");
    let [CssRule::FontFace(rule)] = absent.syntax().rules() else {
        panic!("one font-face rule");
    };
    assert!(rule.descriptors().effective(Kind::FontWeight).is_none());
    admitted("auto");
}

#[test]
fn two_absolute_endpoints_keep_mixed_order_descents_and_duplicates() {
    for value in [
        "normal bold",
        "bold normal",
        "normal 700",
        "700 bold",
        "900 100",
        "400 400",
        "bold bold",
        "350.5 700.25",
        "calc(350 + 50) bold",
    ] {
        admitted(value);
    }
    let source = "@font-face{font-weight:auto;font-weight:normal bold;font-weight:900 100}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("one font-face rule");
    };
    assert_eq!(rule.descriptors().occurrences().len(), 3);
    assert_eq!(
        rule.descriptors()
            .effective(Kind::FontWeight)
            .unwrap()
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        source.rfind("font-weight:").unwrap()
    );
}

#[test]
fn invalid_relative_auto_arity_and_exact_out_of_range_values_drop_locally() {
    for invalid in [
        "bolder",
        "lighter",
        "auto 400",
        "400 auto",
        "normal bold 700",
        "0",
        "1001",
        "0.99999999999999999999",
        "1000.00000000000000000001",
        "1e-999999",
        "1e999999",
        "50%",
        "400px",
    ] {
        let raw = parse_font_face_descriptor_value(invalid, Kind::FontWeight);
        assert!(raw.syntax().is_none(), "raw accepted {invalid}: {raw:?}");
        assert!(!raw.is_clean());

        let source = format!(
            "@font-face{{font-family:Demo;font-weight:normal;src:url(face);font-weight:{invalid};font-display:swap}}"
        );
        let report = parse_sheet(&source);
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("font-face survives bad descriptor: {source}");
        };
        let [diagnostic] = report.diagnostics() else {
            panic!("one dropped descriptor: {source}: {report:?}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            source.find(&format!("font-weight:{invalid}")).unwrap()
        );
        assert_eq!(rule.descriptors().occurrences().len(), 4);
        assert_eq!(
            rule.descriptors()
                .effective(Kind::FontWeight)
                .unwrap()
                .position()
                .unwrap()
                .byte_offset()
                .value(),
            source.find("font-weight:normal").unwrap()
        );
        assert!(rule.descriptors().effective(Kind::FontDisplay).is_some());
        assert!(validate_sheet(&source).is_err());
    }
}

#[test]
fn valid_env_defers_the_whole_descriptor_without_changing_ordinary_grammar() {
    let source = "@font-face{font-weight:normal;font-weight:wide env(weight)}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("one font-face rule");
    };
    assert_eq!(rule.descriptors().occurrences().len(), 2);
    assert!(matches!(
        rule.descriptors()
            .effective(Kind::FontWeight)
            .unwrap()
            .value(),
        CssAuthoredFontFaceDescriptorValue::Pending(_)
    ));
}
