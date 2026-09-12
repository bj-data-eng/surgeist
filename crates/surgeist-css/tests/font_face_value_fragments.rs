#![forbid(unsafe_code)]

//! Raw values use font-face grammar without inventing a descriptor-name occurrence.
//! Unicode-range expectations follow CSS Syntax 3 section 7.1.
use surgeist_css::{
    CssAuthoredFontFeatureSettings, CssErrorCode, CssFontDisplay,
    CssFontFaceDescriptorKind as Kind, CssFontFaceDescriptorValue as Value, CssFontFaceFamily,
    CssFontFaceSource, CssFontFaceSourceList, CssFontFaceStretch, CssFontFaceStyle,
    CssFontFaceWeight, CssFontLocalName, CssRecoveryAction, CssRule, CssUnicodeRange,
    CssUnicodeRangeList, ErrorKind, parse_font_face_descriptor_value, parse_sheet,
};

fn local_x() -> CssFontFaceSourceList {
    CssFontFaceSourceList::try_new(vec![CssFontFaceSource::Local(
        CssFontLocalName::try_new("X").unwrap(),
    )])
    .unwrap()
}

#[test]
fn root_delimiters_after_font_functions_keep_their_own_token_origin() {
    for (source, offset, token) in [
        ("local(\"X\");", 10, ";"),
        ("local(X)}", 8, "}"),
        ("local(X){}", 8, "{"),
        ("/*😀*/\r\nlocal(\"X\");", 20, ";"),
    ] {
        let report = parse_font_face_descriptor_value(source, Kind::Src);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one outer rejection")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            offset,
            "{source}"
        );
        let ErrorKind::InvalidDescriptorValue(detail) = diagnostic.error().kind() else {
            panic!("descriptor error")
        };
        assert_eq!(detail.encountered().unwrap().authored(), token, "{source}");
    }
}

#[test]
fn every_descriptor_kind_returns_its_source_neutral_typed_value() {
    let cases = [
        (
            Kind::FontFamily,
            "X",
            Value::FontFamily(CssFontFaceFamily::try_new("X").unwrap()),
        ),
        (Kind::Src, "local(X)", Value::Src(local_x())),
        (
            Kind::FontWeight,
            "400 700",
            Value::FontWeight(CssFontFaceWeight::try_range(400.0, 700.0).unwrap()),
        ),
        (
            Kind::FontStyle,
            "italic",
            Value::FontStyle(CssFontFaceStyle::Italic),
        ),
        (
            Kind::FontStretch,
            "75% 125%",
            Value::FontStretch(CssFontFaceStretch::try_range_percent(75.0, 125.0).unwrap()),
        ),
        (
            Kind::FontDisplay,
            "swap",
            Value::FontDisplay(CssFontDisplay::Swap),
        ),
        (
            Kind::UnicodeRange,
            "u+?",
            Value::UnicodeRange(
                CssUnicodeRangeList::try_new(vec![CssUnicodeRange::try_new(0, 15).unwrap()])
                    .unwrap(),
            ),
        ),
        (
            Kind::FontFeatureSettings,
            "normal",
            Value::FontFeatureSettings(CssAuthoredFontFeatureSettings::Normal),
        ),
    ];
    for (kind, source, expected) in cases {
        let report = parse_font_face_descriptor_value(source, kind);
        assert!(report.is_clean(), "{}: {report:?}", kind.css_name());
        assert_eq!(report.syntax(), &Some(expected));
        assert_eq!(report.syntax().as_ref().unwrap().kind(), kind);
        let empty = parse_font_face_descriptor_value("", kind);
        assert!(empty.syntax().is_none(), "{}: {empty:?}", kind.css_name());
        assert!(
            empty
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
    }
}

#[test]
fn incomplete_unicode_range_uses_actual_raw_eof_without_a_responsible_token() {
    for (source, line, column) in [
        ("", 0, 0),
        ("U+", 0, 2),
        ("u", 0, 1),
        ("/*😀*/\r\n/*é*/U+", 1, 7),
    ] {
        let report = parse_font_face_descriptor_value(source, Kind::UnicodeRange);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|d| d.action() == CssRecoveryAction::RejectInput)
            .unwrap();
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        assert_eq!(diagnostic.error().position().line().value(), line);
        assert_eq!(diagnostic.error().position().column().value(), column);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
        let ErrorKind::InvalidDescriptorValue(detail) = diagnostic.error().kind() else {
            panic!("typed descriptor error");
        };
        assert!(detail.encountered().is_none());
    }
}

#[test]
fn unicode_range_values_need_no_surrounding_font_face_and_keep_original_token_positions() {
    let source = " /*x*/ U+41-5A, u+? ";
    let report = parse_font_face_descriptor_value(source, Kind::UnicodeRange);
    assert!(report.is_clean(), "{report:?}");
    let Some(Value::UnicodeRange(ranges)) = report.syntax() else {
        panic!("typed ranges");
    };
    assert_eq!(
        ranges.ranges(),
        [
            CssUnicodeRange::try_new(0x41, 0x5a).unwrap(),
            CssUnicodeRange::try_new(0, 15).unwrap()
        ]
    );
    let source = "/*😀*/\r\n/*é*/?";
    let report = parse_font_face_descriptor_value(source, Kind::UnicodeRange);
    assert!(report.syntax().is_none());
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RejectInput)
        .unwrap();
    assert_eq!(diagnostic.error().position().byte_offset().value(), 16);
    assert_eq!(diagnostic.error().position().line().value(), 1);
    assert_eq!(diagnostic.error().position().column().value(), 5);
}

#[test]
fn raw_descriptor_boundaries_reject_annotations_and_delimiters_before_src_recovery() {
    for source in [
        "local(X);",
        "local(X),bad();",
        "local(X),{}",
        "local(X),}",
        "local(X),)",
        "local(X),]",
        "local(X)!important",
        "local(X),bad()!important",
        "local(X);font-display:swap",
    ] {
        let report = parse_font_face_descriptor_value(source, Kind::Src);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
        assert!(!report.diagnostics().iter().any(|d| matches!(
            d.action(),
            CssRecoveryAction::DropFontSourceListItem
                | CssRecoveryAction::RetainWithImplicitClosure
        )));
    }
}

#[test]
fn src_member_recovery_and_implicit_closures_commit_only_retained_components() {
    for source in ["local(X),bad()", "local(X),bad("] {
        let report = parse_font_face_descriptor_value(source, Kind::Src);
        assert_eq!(
            report.syntax(),
            &Some(Value::Src(local_x())),
            "{source}: {report:?}"
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::DropFontSourceListItem)
        );
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
        assert!(report.into_validation_result().is_err());
    }
    for source in ["bad()", "bad("] {
        let report = parse_font_face_descriptor_value(source, Kind::Src);
        assert!(report.syntax().is_none());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
        assert!(!report.diagnostics().iter().any(|d| matches!(
            d.action(),
            CssRecoveryAction::DropFontSourceListItem
                | CssRecoveryAction::RetainWithImplicitClosure
        )));
    }
    let source = "local(X";
    let report = parse_font_face_descriptor_value(source, Kind::Src);
    assert_eq!(report.syntax(), &Some(Value::Src(local_x())));
    let closure = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        .unwrap();
    assert_eq!(
        closure.error().position().byte_offset().value(),
        source.len()
    );
    assert_eq!(closure.span().start().byte_offset().value(), source.len());
    assert_eq!(closure.span().end().byte_offset().value(), source.len());
}

#[test]
fn descriptor_resource_failure_retains_its_stop_action() {
    let source = format!("{}x{}", "f(".repeat(300), ")".repeat(300));
    let report = parse_font_face_descriptor_value(&source, Kind::Src);
    assert!(report.syntax().is_none());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::StopAtNestingLimit)
    );
    assert!(
        !report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RejectInput)
    );
}

#[test]
fn stylesheet_occurrences_keep_real_names_and_last_valid_values() {
    let source = "@font-face{FONT-DISPLAY:swap;font-display:block;unicode-range:u+?}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("font face");
    };
    assert_eq!(face.descriptors().occurrences().len(), 3);
    let display = face.descriptors().font_display().unwrap();
    assert_eq!(display.value(), &CssFontDisplay::Block);
    assert_eq!(
        display.position().byte_offset().value(),
        source.find("font-display:block").unwrap()
    );
    let range = face.descriptors().unicode_range().unwrap();
    assert_eq!(
        range.position().byte_offset().value(),
        source.find("unicode-range").unwrap()
    );
    assert_eq!(
        range.value().ranges(),
        [CssUnicodeRange::try_new(0, 15).unwrap()]
    );
    assert!(face.descriptors().font_family().is_none());
    assert!(face.descriptors().src().is_none());
}
