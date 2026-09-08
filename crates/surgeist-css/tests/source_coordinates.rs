use std::hash::Hash;

mod common;

use common::CssParseReportTestExt;
use surgeist_css::{
    CssByteOffset, CssDefinedFalseMediaReason, CssErrorCode, CssKnownProperty, CssLineIndex,
    CssMediaConditionKind, CssMediaQuery, CssMediaType, CssRecoveryAction, CssRule,
    CssSourcePosition, CssSourceSpan, CssSupportsConditionKind, CssTokenKind, CssUtf16ColumnIndex,
    ErrorKind, parse_sheet, parse_style_attribute,
};

fn assert_copy_hash_ord<T: Copy + Eq + Ord + Hash>() {}

fn assert_position(position: CssSourcePosition, byte_offset: usize, line: u32, column: u32) {
    assert_eq!(position.byte_offset().value(), byte_offset);
    assert_eq!(position.line().value(), line);
    assert_eq!(position.column().value(), column);
}

#[test]
fn defined_false_media_nodes_preserve_exact_non_bmp_byte_and_utf16_positions() {
    let source = "@media /*😀*/ only F\\75ture, /*😀*/ (UnKnOwN: CAlc(1foo + 2px)) {}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Media(rule)] = report.syntax().rules() else {
        panic!("expected retained media rule")
    };
    let [
        CssMediaQuery::Typed(typed),
        CssMediaQuery::Condition(condition),
    ] = rule.query().queries()
    else {
        panic!("expected unknown type and defined-false feature")
    };

    assert_eq!(typed.media_type(), CssMediaType::Unknown);
    let unknown_type = typed.unknown_media_type().expect("unknown type details");
    let typed_offset = source.find("only").unwrap();
    let type_offset = source.find("F\\75ture").unwrap();
    assert_position(
        typed.position(),
        typed_offset,
        0,
        u32::try_from(source[..typed_offset].encode_utf16().count()).unwrap(),
    );
    assert_position(
        unknown_type.position(),
        type_offset,
        0,
        u32::try_from(source[..type_offset].encode_utf16().count()).unwrap(),
    );
    assert_eq!(unknown_type.as_css(), "F\\75ture");

    let condition_offset = source.find("(UnKnOwN").unwrap();
    let CssMediaConditionKind::DefinedFalse(defined_false) = condition.kind() else {
        panic!("expected defined-false details")
    };
    assert_position(
        condition.position(),
        condition_offset,
        0,
        u32::try_from(source[..condition_offset].encode_utf16().count()).unwrap(),
    );
    assert_eq!(defined_false.position(), condition.position());
    assert_eq!(defined_false.as_css(), "(UnKnOwN: CAlc(1foo + 2px))");
    assert_eq!(
        defined_false.reason(),
        CssDefinedFalseMediaReason::UnknownFeature
    );
}

#[test]
fn counter_style_descriptor_positions_and_errors_use_non_bmp_utf16_coordinates() {
    let source = concat!(
        "/*😀*/@counter-style kept { system: additive; range: infinite -1; ",
        "additive-symbols: 10 X, 10 I; additive-symbols: 10 X, 1 I; }",
    );
    let report = parse_sheet(source);
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("expected retained counter style")
    };
    let range_offset = source.find("range").unwrap();
    assert_position(
        rule.descriptors().range().unwrap().position(),
        range_offset,
        0,
        u32::try_from(source[..range_offset].encode_utf16().count()).unwrap(),
    );

    let [diagnostic] = report.diagnostics() else {
        panic!("expected invalid additive ordering recovery")
    };
    let responsible = source.match_indices("10 I").next().unwrap().0;
    let descriptor_start = source.find("additive-symbols").unwrap();
    let descriptor_end = descriptor_start + source[descriptor_start..].find(';').unwrap() + 1;
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(source[..responsible].encode_utf16().count()).unwrap(),
    );
    assert_position(
        diagnostic.span().start(),
        descriptor_start,
        0,
        u32::try_from(source[..descriptor_start].encode_utf16().count()).unwrap(),
    );
    assert_position(
        diagnostic.span().end(),
        descriptor_end,
        0,
        u32::try_from(source[..descriptor_end].encode_utf16().count()).unwrap(),
    );
}

#[test]
fn supports_nodes_preserve_exact_non_bmp_byte_and_utf16_positions() {
    let source = "/*😀*/\n@supports /*🦊*/ (D\\69splay: grid) {}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("expected supports rule");
    };
    let CssSupportsConditionKind::Declaration(declaration) = rule.condition().kind() else {
        panic!("expected declaration condition");
    };

    let rule_offset = source.find("@supports").unwrap();
    let condition_offset = source.find("(D\\69splay").unwrap();
    let declaration_offset = source.find("D\\69splay").unwrap();
    assert_position(rule.position(), rule_offset, 1, 0);
    assert_position(
        rule.condition().position(),
        condition_offset,
        1,
        u32::try_from(
            source[source.find('\n').unwrap() + 1..condition_offset]
                .encode_utf16()
                .count(),
        )
        .unwrap(),
    );
    assert_position(
        declaration.position(),
        declaration_offset,
        1,
        u32::try_from(
            source[source.find('\n').unwrap() + 1..declaration_offset]
                .encode_utf16()
                .count(),
        )
        .unwrap(),
    );
    assert_eq!(declaration.property(), "D\\69splay");
}

#[test]
fn font_error_after_non_bmp_text_has_exact_utf16_coordinates_and_span() {
    let source = "--😀: 1; font: menu serif; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("mispositioned system font must recover once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_position(diagnostic.error().position(), 17, 0, 15);
    assert_position(diagnostic.span().start(), 11, 0, 9);
    assert_position(diagnostic.span().end(), 28, 0, 26);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected font property error");
    };
    assert_eq!(detail.property(), CssKnownProperty::Font);
    assert_eq!(detail.encountered().unwrap().kind(), CssTokenKind::Ident);
    assert_eq!(detail.encountered().unwrap().authored(), "menu");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects the recovered font shorthand");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn opentype_tag_error_has_exact_utf16_coordinates_and_full_declaration_span() {
    let source = "--😀: 1; font-feature-settings: \"😀abc\"; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("supplementary OpenType tag must recover once");
    };
    let responsible = source.find("\"😀abc\"").expect("feature tag");
    let declaration_start = source
        .find("font-feature-settings")
        .expect("feature declaration");
    let declaration_end = declaration_start
        + source[declaration_start..]
            .find(';')
            .expect("feature declaration terminator")
        + 1;
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(source[..responsible].encode_utf16().count()).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(source[..declaration_start].encode_utf16().count())
            .expect("UTF-16 declaration start"),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(source[..declaration_end].encode_utf16().count())
            .expect("UTF-16 declaration end"),
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected font feature property error");
    };
    assert_eq!(detail.property(), CssKnownProperty::FontFeatureSettings);
    let encountered = detail.encountered().expect("responsible tag token");
    assert_eq!(encountered.kind(), CssTokenKind::String);
    assert_eq!(encountered.authored(), "\"😀abc\"");
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color,
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects the recovered supplementary tag");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn font_face_feature_error_after_non_bmp_text_has_exact_descriptor_span() {
    let source = concat!(
        "/*😀*/@font-face{font-family:Demo;src:url(face);",
        "font-feature-settings:\"😀abc\"}.after{color:red}",
    );
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::FontFace(_), CssRule::Style(_)]
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid descriptor must recover once");
    };
    let responsible = source.find("\"😀abc\"").unwrap();
    let descriptor_start = source.find("font-feature-settings").unwrap();
    let descriptor_end = source.find("}.after").unwrap();
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidDescriptorValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(source[..responsible].encode_utf16().count()).unwrap(),
    );
    assert_position(
        diagnostic.span().start(),
        descriptor_start,
        0,
        u32::try_from(source[..descriptor_start].encode_utf16().count()).unwrap(),
    );
    assert_position(
        diagnostic.span().end(),
        descriptor_end,
        0,
        u32::try_from(source[..descriptor_end].encode_utf16().count()).unwrap(),
    );
}

#[test]
fn size_adjust_error_after_non_bmp_text_has_exact_utf16_coordinates_and_span() {
    let source = "--😀: 1; font-size-adjust: -1; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("negative size adjustment must recover once");
    };
    let responsible = source.find("-1").unwrap();
    let declaration_start = source.find("font-size-adjust").unwrap();
    let declaration_end = declaration_start + source[declaration_start..].find(';').unwrap() + 1;
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(source[..responsible].encode_utf16().count()).unwrap(),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(source[..declaration_start].encode_utf16().count()).unwrap(),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(source[..declaration_end].encode_utf16().count()).unwrap(),
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected font-size-adjust property error");
    };
    assert_eq!(detail.property(), CssKnownProperty::FontSizeAdjust);
    let encountered = detail.encountered().expect("responsible negative number");
    assert_eq!(encountered.kind(), CssTokenKind::Number);
    assert_eq!(encountered.authored(), "-1");
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color,
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered negative size adjustment");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn font_variant_conflict_after_non_bmp_text_has_exact_utf16_coordinates_and_span() {
    let source = "--😀: 1; font-variant: jis04 traditional; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("conflicting East Asian forms must recover once");
    };
    let responsible = source.find("traditional").unwrap();
    let declaration_start = source.find("font-variant").unwrap();
    let declaration_end = declaration_start + source[declaration_start..].find(';').unwrap() + 1;
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(source[..responsible].encode_utf16().count()).unwrap(),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(source[..declaration_start].encode_utf16().count()).unwrap(),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(source[..declaration_end].encode_utf16().count()).unwrap(),
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected font-variant property error");
    };
    assert_eq!(detail.property(), CssKnownProperty::FontVariant);
    let encountered = detail
        .encountered()
        .expect("responsible conflicting keyword");
    assert_eq!(encountered.kind(), CssTokenKind::Ident);
    assert_eq!(encountered.authored(), "traditional");
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color,
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects the recovered shorthand");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn font_source_error_after_non_bmp_text_has_exact_utf16_coordinates_and_span() {
    let source = concat!(
        ".😀{color:red}",
        "@font-face{font-family:Demo;src:url(face) format(woff3)}",
        ".after{color:blue}",
    );
    let report = parse_sheet(source);
    assert_eq!(report.syntax().rules().len(), 2);
    assert_eq!(report.diagnostics().len(), 2);
    let diagnostic = &report.diagnostics()[0];
    let responsible = source.find("woff3").unwrap();
    let descriptor_start = source.find("src:").unwrap();
    let descriptor_end = source[descriptor_start..].find('}').unwrap() + descriptor_start;
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidDescriptorValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(source[..responsible].encode_utf16().count()).unwrap(),
    );
    assert_position(
        diagnostic.span().start(),
        descriptor_start,
        0,
        u32::try_from(source[..descriptor_start].encode_utf16().count()).unwrap(),
    );
    assert_position(
        diagnostic.span().end(),
        descriptor_end,
        0,
        u32::try_from(source[..descriptor_end].encode_utf16().count()).unwrap(),
    );
    let ErrorKind::InvalidDescriptorValue(detail) = diagnostic.error().kind() else {
        panic!("expected font source descriptor error");
    };
    assert_eq!(detail.descriptor().as_str(), "src");
    assert_eq!(detail.encountered().unwrap().authored(), "woff3");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_sheet(source)
            .expect_err("strict validation rejects recovered font source");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn timing_type_error_after_non_bmp_text_has_exact_utf16_coordinates_and_span() {
    let source = "--😀: 1; transition-duration: calc(1px + 2px); color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid time calculation must recover once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_position(diagnostic.error().position(), 37, 0, 35);
    assert_position(diagnostic.span().start(), 11, 0, 9);
    assert_position(diagnostic.span().end(), 48, 0, 46);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected structured timing property error");
    };
    assert_eq!(detail.property(), CssKnownProperty::TransitionDuration);
    let encountered = detail.encountered().expect("responsible length token");
    assert_eq!(encountered.kind(), CssTokenKind::Dimension);
    assert_eq!(encountered.authored(), "1px");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered timing type error");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn opacity_domain_error_after_non_bmp_text_has_exact_coordinates_and_span() {
    let source = "--😀: 1; opacity: 1px; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid opacity dimension must recover once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_position(diagnostic.error().position(), 20, 0, 18);
    assert_position(diagnostic.span().start(), 11, 0, 9);
    assert_position(diagnostic.span().end(), 24, 0, 22);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected structured opacity property error");
    };
    assert_eq!(detail.property(), CssKnownProperty::Opacity);
    let encountered = detail.encountered().expect("responsible dimension token");
    assert_eq!(encountered.kind(), CssTokenKind::Dimension);
    assert_eq!(encountered.authored(), "1px");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered opacity dimension");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn grid_repeat_error_after_non_bmp_text_has_exact_coordinates_and_span() {
    let source = "--😀: 1; grid-template-columns: repeat(auto-fit, 1fr); color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid auto-repeat must recover once");
    };
    let responsible = source.find("1fr").unwrap();
    let declaration_start = source.find("grid-template-columns").unwrap();
    let declaration_end = declaration_start + source[declaration_start..].find(';').unwrap() + 1;
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(source[..responsible].encode_utf16().count()).unwrap(),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(source[..declaration_start].encode_utf16().count()).unwrap(),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(source[..declaration_end].encode_utf16().count()).unwrap(),
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected Grid property error");
    };
    assert_eq!(detail.property(), CssKnownProperty::GridTemplateColumns);
    assert_eq!(
        detail.encountered().unwrap().kind(),
        CssTokenKind::Dimension
    );
    assert_eq!(detail.encountered().unwrap().authored(), "1fr");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects flexible auto-repeat");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn color_component_error_after_non_bmp_text_has_exact_coordinates_and_span() {
    let source = "--😀: 1; color: rgb(1px 2 3); opacity: 0.5";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid RGB component must recover once");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidColorSyntax);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find("1px").expect("responsible RGB component");
    let declaration_start = source.find("color").expect("color declaration");
    let declaration_end = source.find(';').expect("custom property terminator")
        + 1
        + source[source.find(';').unwrap() + 1..]
            .find(';')
            .expect("color declaration terminator")
        + 1;
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(responsible - 2).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(declaration_start - 2).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(declaration_end - 2).expect("UTF-16 column"),
    );
    let ErrorKind::InvalidColorSyntax(detail) = diagnostic.error().kind() else {
        panic!("expected structured color error");
    };
    assert_eq!(
        detail.component().map(|value| value.as_str()),
        Some("component")
    );
    assert_eq!(
        detail.encountered().unwrap().kind(),
        CssTokenKind::Dimension
    );
    assert_eq!(detail.encountered().unwrap().authored(), "1px");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered color component error");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn perceptual_color_error_after_non_bmp_text_has_exact_coordinates_and_span() {
    let source = "--😀: 1; color: lab(50% 1px 30); opacity: 0.5";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid Lab component must recover once");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidColorSyntax);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find("1px").expect("responsible Lab component");
    let declaration_start = source.find("color").expect("color declaration");
    let declaration_end = declaration_start
        + source[declaration_start..]
            .find(';')
            .expect("color declaration terminator")
        + 1;
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(source[..responsible].encode_utf16().count()).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(source[..declaration_start].encode_utf16().count()).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(source[..declaration_end].encode_utf16().count()).expect("UTF-16 column"),
    );
    let ErrorKind::InvalidColorSyntax(detail) = diagnostic.error().kind() else {
        panic!("expected structured color error");
    };
    assert_eq!(
        detail.component().map(|value| value.as_str()),
        Some("component")
    );
    let encountered = detail.encountered().expect("responsible Lab component");
    assert_eq!(encountered.kind(), CssTokenKind::Dimension);
    assert_eq!(encountered.authored(), "1px");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered Lab component error");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn relative_color_error_after_non_bmp_text_has_exact_coordinates_span_and_strict_parity() {
    let source = "--😀: 1; color: hwb(from red h s b); opacity: 0.5";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("foreign relative channel must recover once");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidColorSyntax);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find(" s b").expect("foreign channel") + 1;
    let declaration_start = source.find("color").expect("color declaration");
    let declaration_end = declaration_start
        + source[declaration_start..]
            .find(';')
            .expect("color declaration terminator")
        + 1;
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(source[..responsible].encode_utf16().count()).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(source[..declaration_start].encode_utf16().count()).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(source[..declaration_end].encode_utf16().count()).expect("UTF-16 column"),
    );
    let ErrorKind::InvalidColorSyntax(detail) = diagnostic.error().kind() else {
        panic!("expected structured relative-color detail");
    };
    assert_eq!(
        detail.component().map(|component| component.as_str()),
        Some("relative channel")
    );
    let encountered = detail.encountered().expect("responsible foreign channel");
    assert_eq!(encountered.kind(), CssTokenKind::Ident);
    assert_eq!(encountered.authored(), "s");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects foreign relative channels");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn color_mix_error_after_non_bmp_text_has_exact_coordinates_span_and_strict_parity() {
    let source = "--😀: 1; color: color-mix(in srgb longer hue, red, blue); opacity: 0.5";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("rectangular-space hue method must recover once");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidColorSyntax);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find("longer").expect("responsible hue method");
    let declaration_start = source.find("color").expect("color declaration");
    let declaration_end = declaration_start
        + source[declaration_start..]
            .find(';')
            .expect("color declaration terminator")
        + 1;
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(source[..responsible].encode_utf16().count()).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(source[..declaration_start].encode_utf16().count()).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(source[..declaration_end].encode_utf16().count()).expect("UTF-16 column"),
    );
    let ErrorKind::InvalidColorSyntax(detail) = diagnostic.error().kind() else {
        panic!("expected structured color-mix detail");
    };
    assert_eq!(
        detail.component().map(|component| component.as_str()),
        Some("hue interpolation"),
    );
    let encountered = detail.encountered().expect("responsible hue method");
    assert_eq!(encountered.kind(), CssTokenKind::Ident);
    assert_eq!(encountered.authored(), "longer");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects rectangular-space hue methods");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn legacy_rgb_mixed_domain_reports_the_later_component_and_retains_its_sibling() {
    let source = "--😀: 1; color: rgb(1, 20%, 3); opacity: 0.5";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(
        report.syntax()[1]
            .known()
            .expect("retained opacity declaration")
            .property(),
        CssKnownProperty::Opacity,
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("mixed legacy RGB component domains must recover once");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidColorSyntax);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find("20%").expect("responsible RGB component");
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(source[..responsible].encode_utf16().count()).expect("UTF-16 column"),
    );
    let ErrorKind::InvalidColorSyntax(detail) = diagnostic.error().kind() else {
        panic!("expected structured color error");
    };
    assert_eq!(
        detail.component().map(|value| value.as_str()),
        Some("component")
    );
    let encountered = detail.encountered().expect("responsible percentage token");
    assert_eq!(encountered.kind(), CssTokenKind::Percentage);
    assert_eq!(encountered.authored(), "20%");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects mixed legacy RGB component domains");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn easing_count_error_after_non_bmp_text_has_exact_utf16_coordinates_and_span() {
    let source = "--😀: 1; transition-timing-function: steps(1, jump-none); color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid jump-none count must recover once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find("steps(1").expect("steps function") + "steps(".len();
    let declaration_start = source
        .find("transition-timing-function")
        .expect("timing declaration start");
    let declaration_end = declaration_start
        + source[declaration_start..]
            .find(';')
            .expect("timing declaration end")
        + 1;
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(responsible - 2).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(declaration_start - 2).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(declaration_end - 2).expect("UTF-16 column"),
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected structured easing property error");
    };
    assert_eq!(
        detail.property(),
        CssKnownProperty::TransitionTimingFunction
    );
    let encountered = detail.encountered().expect("responsible step count");
    assert_eq!(encountered.kind(), CssTokenKind::Number);
    assert_eq!(encountered.authored(), "1");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered easing count error");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn generic_position_error_after_non_bmp_text_has_exact_utf16_coordinates_and_span() {
    let source = "--😀: 1; mask-position: 50% left; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid generic position must recover once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_position(diagnostic.error().position(), 30, 0, 28);
    assert_position(diagnostic.span().start(), 11, 0, 9);
    assert_position(diagnostic.span().end(), 35, 0, 33);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected structured position property error");
    };
    assert_eq!(detail.property(), CssKnownProperty::MaskPosition);
    let encountered = detail
        .encountered()
        .expect("responsible horizontal keyword");
    assert_eq!(encountered.kind(), CssTokenKind::Ident);
    assert_eq!(encountered.authored(), "left");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered generic position input");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn object_position_error_after_non_bmp_text_has_exact_utf16_coordinates_and_span() {
    let source = "--😀: 1; object-position: left top 10px; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid object position must recover once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find("10px").expect("responsible third component");
    let declaration_start = source.find("object-position").expect("property start");
    let declaration_end = declaration_start
        + source[declaration_start..]
            .find(';')
            .expect("declaration end")
        + 1;
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(responsible - 2).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(declaration_start - 2).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(declaration_end - 2).expect("UTF-16 column"),
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected structured object-position property error");
    };
    assert_eq!(detail.property(), CssKnownProperty::ObjectPosition);
    let encountered = detail.encountered().expect("responsible third component");
    assert_eq!(encountered.kind(), CssTokenKind::Dimension);
    assert_eq!(encountered.authored(), "10px");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered object-position input");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn transform_domain_error_after_non_bmp_text_has_exact_utf16_coordinates_and_span() {
    let source = "--😀: 1; transform: translate3d(1px, 2px, 3%); color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid transform z percentage must recover once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find("3%").expect("responsible z percentage");
    let declaration_start = source.find("transform").expect("property start");
    let declaration_end = declaration_start
        + source[declaration_start..]
            .find(';')
            .expect("declaration end")
        + 1;
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(responsible - 2).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(declaration_start - 2).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(declaration_end - 2).expect("UTF-16 column"),
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected structured transform property error");
    };
    assert_eq!(detail.property(), CssKnownProperty::Transform);
    let encountered = detail.encountered().expect("responsible z percentage");
    assert_eq!(encountered.kind(), CssTokenKind::Percentage);
    assert_eq!(encountered.authored(), "3%");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered transform input");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn filter_domain_error_after_non_bmp_text_has_exact_utf16_coordinates_and_span() {
    let source = "--😀: 1; filter: brightness(-1); color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid filter amount must recover once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find("-1").expect("responsible filter amount");
    let declaration_start = source.find("filter").expect("property start");
    let declaration_end = declaration_start
        + source[declaration_start..]
            .find(';')
            .expect("declaration end")
        + 1;
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(responsible - 2).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(declaration_start - 2).expect("UTF-16 column"),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(declaration_end - 2).expect("UTF-16 column"),
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected structured filter property error");
    };
    assert_eq!(detail.property(), CssKnownProperty::Filter);
    let encountered = detail.encountered().expect("responsible filter number");
    assert_eq!(encountered.kind(), CssTokenKind::Number);
    assert_eq!(encountered.authored(), "-1");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered filter input");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn basic_shape_error_after_non_bmp_text_has_exact_utf16_coordinates_and_span() {
    let source = "--😀: 1; clip-path: polygon(round -1px, 0 0); color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid polygon rounding must recover once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find("-1px").unwrap();
    let declaration_start = source.find("clip-path").unwrap();
    let declaration_end = declaration_start + source[declaration_start..].find(';').unwrap() + 1;
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(responsible - 2).unwrap(),
    );
    assert_position(
        diagnostic.span().start(),
        declaration_start,
        0,
        u32::try_from(declaration_start - 2).unwrap(),
    );
    assert_position(
        diagnostic.span().end(),
        declaration_end,
        0,
        u32::try_from(declaration_end - 2).unwrap(),
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected structured clip-path property error");
    };
    assert_eq!(detail.property(), CssKnownProperty::ClipPath);
    let encountered = detail.encountered().expect("responsible polygon radius");
    assert_eq!(encountered.kind(), CssTokenKind::Dimension);
    assert_eq!(encountered.authored(), "-1px");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered basic shape");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

fn first_declaration_position(source: &str) -> CssSourcePosition {
    let sheet = parse_sheet(source).expect("valid stylesheet");
    let CssRule::Style(rule) = &sheet.rules()[0] else {
        panic!("expected style rule");
    };
    rule.declarations().as_slice()[0].position()
}

#[test]
fn source_types_are_copyable_comparable_and_hashable() {
    assert_copy_hash_ord::<CssByteOffset>();
    assert_copy_hash_ord::<CssLineIndex>();
    assert_copy_hash_ord::<CssUtf16ColumnIndex>();
    assert_copy_hash_ord::<CssSourcePosition>();
    assert_copy_hash_ord::<CssSourceSpan>();
}

#[test]
fn source_public_nodes_expose_zero_based_byte_line_and_utf16_coordinates() {
    assert_position(first_declaration_position(".a { width: 1px; }"), 5, 0, 5);
    assert_position(first_declaration_position(".a {\n  width: 1px; }"), 7, 1, 2);
    assert_position(
        first_declaration_position(".a {\r\n  width: 1px; }"),
        8,
        1,
        2,
    );
    assert_position(
        first_declaration_position(".\\61 bc { width: 1px; }"),
        10,
        0,
        10,
    );
    assert_position(first_declaration_position(".😀2 { width: 1px; }"), 9, 0, 7);

    let sheet = parse_sheet("/*a\nbc*/@import \"theme.css\";").expect("valid import");
    let CssRule::Import(rule) = &sheet.rules()[0] else {
        panic!("expected import rule");
    };
    assert_position(rule.position(), 8, 1, 4);

    let sheet = parse_sheet("/*😀*/@import \"x\" supports(display: grid);")
        .expect("valid conditional import");
    let CssRule::Import(rule) = &sheet.rules()[0] else {
        panic!("expected conditional import rule");
    };
    assert_position(rule.position(), 8, 0, 6);
    assert_position(
        rule.supports()
            .expect("import supports")
            .condition()
            .position(),
        29,
        0,
        27,
    );

    let sheet = parse_sheet("/*😀*/\n  @namespace s\\76 g \"urn:svg\";")
        .expect("valid namespace declaration");
    let CssRule::Namespace(rule) = &sheet.rules()[0] else {
        panic!("expected namespace rule");
    };
    assert_position(rule.position(), 11, 1, 2);
    assert_eq!(rule.prefix().expect("decoded prefix").as_str(), "svg");
}

#[test]
fn undeclared_namespace_prefix_at_eof_preserves_non_bmp_coordinates() {
    let source = "@namespace svg \"u\";/*😀*/missing|a {";
    let report = parse_sheet(source);
    assert!(matches!(report.syntax().rules(), [CssRule::Namespace(_)]));
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one undeclared-prefix diagnostic")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_position(diagnostic.error().position(), 35, 0, 33);
    assert_position(diagnostic.span().start(), 27, 0, 25);
    assert_position(diagnostic.span().end(), source.len(), 0, 36);
}

#[test]
fn selectors3_language_errors_preserve_non_bmp_and_eof_coordinates() {
    let source = "/*😀*/.before {} .bad:lang(\"en\") {} .after {}";
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::Style(_), CssRule::Style(_)]
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one invalid language-range diagnostic")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    let start = source.find(".bad").unwrap();
    let end = start + ".bad:lang(\"en\") {}".len();
    let responsible = source.find("\"en\"").unwrap();
    assert_position(
        diagnostic.error().position(),
        responsible,
        0,
        u32::try_from(responsible - 2).unwrap(),
    );
    assert_position(
        diagnostic.span().start(),
        start,
        0,
        u32::try_from(start - 2).unwrap(),
    );
    assert_position(
        diagnostic.span().end(),
        end,
        0,
        u32::try_from(end - 2).unwrap(),
    );

    let eof_source = "/*😀*/.before {} .bad:lang(";
    let eof = parse_sheet(eof_source);
    assert!(matches!(eof.syntax().rules(), [CssRule::Style(_)]));
    let [diagnostic] = eof.diagnostics() else {
        panic!("expected one EOF language-range diagnostic")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidQualifiedRule
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_position(
        diagnostic.error().position(),
        eof_source.len(),
        0,
        u32::try_from(eof_source.len() - 2).unwrap(),
    );
    let eof_start = eof_source.find(".bad").unwrap();
    assert_position(
        diagnostic.span().start(),
        eof_start,
        0,
        u32::try_from(eof_start - 2).unwrap(),
    );
    assert_position(
        diagnostic.span().end(),
        eof_source.len(),
        0,
        u32::try_from(eof_source.len() - 2).unwrap(),
    );
}
