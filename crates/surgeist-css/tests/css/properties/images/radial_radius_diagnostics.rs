#![forbid(unsafe_code)]

//! Rejected radial radii identify the responsible argument, preserving recovery.
//! The public diagnostic contract uses original UTF-8 offsets and UTF-16 columns.

use surgeist_css::{
    CssErrorCode, CssKnownProperty, CssRecoveryAction, CssRule, CssTokenKind, ErrorKind,
    parse_sheet, parse_style_attribute, validate_style_attribute,
};

fn assert_failure(source: &str, diagnostic: &surgeist_css::CssRecoveryDiagnostic) {
    let responsible = source.find("calc(").unwrap();
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible
    );
    assert_eq!(
        diagnostic.error().position().line().value() as usize,
        source[..responsible]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
    );
    assert_eq!(
        diagnostic.error().position().column().value() as usize,
        source[..responsible]
            .rsplit('\n')
            .next()
            .unwrap()
            .encode_utf16()
            .count()
    );
    let start = source.find("background-image").unwrap();
    let end = start + source[start..].find(';').unwrap() + 1;
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(diagnostic.span().end().byte_offset().value(), end);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected typed property-value rejection");
    };
    assert_eq!(detail.property(), CssKnownProperty::BackgroundImage);
    let encountered = detail.encountered().expect("responsible function opener");
    assert_eq!(encountered.kind(), CssTokenKind::Function);
    assert_eq!(encountered.authored(), "calc(");
}

#[test]
fn malformed_second_ellipse_radius_identifies_its_function_and_preserves_color() {
    for gradient in ["radial-gradient", "repeating-radial-gradient"] {
        let source = format!(
            "background-image:{gradient}(ellipse 1px calc(2px + bad), red, blue);color:red"
        );
        let report = parse_style_attribute(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejection");
        };
        assert_failure(&source, diagnostic);
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(
            validate_style_attribute(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn malformed_second_radius_uses_original_unicode_coordinates_in_a_stylesheet() {
    let source = ".subject {\n--😀: kept; background-image:radial-gradient(ellipse 1px calc(2px + bad), red, blue);color:red; }";
    let report = parse_sheet(source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejection");
    };
    assert_failure(source, diagnostic);
    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("containing rule retained");
    };
    assert_eq!(rule.declarations().len(), 2);
    assert_eq!(
        rule.declarations()[1].known().unwrap().property(),
        CssKnownProperty::Color
    );
}

#[test]
fn malformed_circle_radius_already_identifies_its_function() {
    let source = "background-image:radial-gradient(circle calc(2px + bad), red, blue);color:red";
    let report = parse_style_attribute(source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejection");
    };
    assert_failure(source, diagnostic);
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
}
