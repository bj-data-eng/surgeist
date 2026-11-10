use crate::css::support::parse_report as common;

use common::CssParseReportTestExt;

use surgeist_css::CssParser;
use surgeist_css::declaration::CssImportance;
use surgeist_css::diagnostic::{CssDeclarationContextRef, CssErrorCode, CssTokenKind};
use surgeist_css::property::CssPropertyNameRef;
use surgeist_css::rule::CssRule;

fn style_declarations(source: &str) -> surgeist_css::declaration::CssDeclarationList {
    let sheet = CssParser::default()
        .parse_sheet(source)
        .expect("stylesheet must parse");
    let [CssRule::Style(rule)] = sheet.rules() else {
        panic!("expected one style rule");
    };
    rule.declarations().clone()
}

#[test]
fn declaration_importance_normal_and_terminal_important_are_modeled() {
    let normal = style_declarations(".x { width: 1px }");
    assert_eq!(normal.as_slice()[0].importance(), CssImportance::Normal);
    assert_eq!(CssImportance::default(), CssImportance::Normal);

    for source in [
        ".x { width: 1px!important }",
        ".x { width: 1px !IMPORTANT; }",
        ".x { width: 1px /**/ ! /**/ important /**/; }",
    ] {
        let declarations = style_declarations(source);
        assert_eq!(declarations.len(), 1, "{source}");
        assert!(!declarations.is_empty(), "{source}");
        assert_eq!(declarations.iter().count(), 1, "{source}");
        assert_eq!(
            declarations.as_slice()[0].importance(),
            CssImportance::Important,
            "{source}"
        );
    }
}

#[test]
fn declaration_importance_strips_only_terminal_annotation_boundary() {
    let declarations = style_declarations(
        ".x { --custom: A/**/ B  !important; width: var(--measure,  2px) /**/ !IMPORTANT }",
    );
    let custom = declarations.as_slice()[0]
        .custom()
        .expect("custom declaration");
    assert_eq!(
        custom.value().value().expect("authored value").as_css(),
        "A/**/ B"
    );
    let substitution = declarations.as_slice()[1]
        .known()
        .expect("known width")
        .substitution_dependent()
        .expect("expected substitution-dependent width");
    assert_eq!(substitution.as_css(), "var(--measure,  2px)");
}

// Syntax 3 (2026 CRD) §5.5.6 steps 6 and 7 read an annotation only at the end
// of a value. A `!` anywhere else is an ordinary delimiter in the value, which
// the property's grammar rejects at the `!`; a custom property keeps it.
#[test]
fn declaration_importance_a_bang_before_the_end_is_part_of_the_value() {
    for (source, bang) in [
        (".x { width: 1px !; }", 16),
        (".x { width: 1px !imporant; }", 16),
        (".x { width: 1px !important !important; }", 16),
        (".x { width: 1px !important extra; }", 16),
    ] {
        let error = CssParser::default()
            .parse_sheet(source)
            .expect_err("the value must be rejected");
        assert_eq!(error.code(), CssErrorCode::InvalidPropertyValue, "{source}");
        assert_eq!(error.position().byte_offset().value(), bang, "{source}");
        assert_eq!(error.encountered().unwrap().kind(), CssTokenKind::Delim);
        assert_eq!(error.encountered().unwrap().authored(), "!");
    }
    let sheet = CssParser::default()
        .parse_sheet(".x { --custom: value !oops; }")
        .expect("a custom property keeps the bang in its value");
    let [CssRule::Style(rule)] = sheet.rules() else {
        panic!("expected a style rule");
    };
    let custom = rule.declarations().as_slice()[0]
        .custom()
        .expect("custom declaration");
    assert_eq!(
        custom.value().value().expect("authored value").as_css(),
        "value !oops"
    );
}

#[test]
fn declaration_importance_strict_sheet_rejects_bad_declaration_between_valid_siblings() {
    let source = ".x { height: 1px; width: 2px !oops; opacity: 1; }";
    let error = CssParser::default()
        .parse_sheet(source)
        .expect_err("current strict parser must reject the sheet");
    assert_eq!(error.code(), CssErrorCode::InvalidPropertyValue);
    assert_eq!(
        error.position().byte_offset().value(),
        source.find('!').expect("bang")
    );
}

#[test]
fn declaration_importance_keyframes_use_distinct_unimportant_declarations() {
    let sheet = CssParser::default()
        .parse_sheet("@keyframes fade { from { opacity: 0 } to { opacity: 1; } }")
        .expect("keyframes must parse");
    let [CssRule::Keyframes(rule)] = sheet.rules() else {
        panic!("expected keyframes");
    };
    let declarations: &surgeist_css::rule::keyframes::CssKeyframeDeclarationList =
        rule.blocks()[0].declarations();
    assert_eq!(declarations.as_slice().len(), 1);
    assert_eq!(declarations.iter().count(), 1);
    assert!(!declarations.is_empty());
    assert!(matches!(
        declarations.as_slice()[0].property_name(),
        CssPropertyNameRef::Known(_)
    ));

    let source = "@keyframes fade { from { opacity: 0 !important; } }";
    let report = CssParser::default().parse_sheet(source);
    let [CssRule::Keyframes(keyframes)] = report.syntax().rules() else {
        panic!("invalid importance must leave the authored keyframes structure");
    };
    assert!(keyframes.blocks()[0].declarations().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid importance must produce one diagnostic");
    };
    let error = diagnostic.error();
    assert_eq!(error.code(), CssErrorCode::InvalidDeclarationAnnotation);
    assert_eq!(
        error.position().byte_offset().value(),
        source.find('!').expect("bang")
    );
    let detail = &error;
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidDeclarationAnnotation,
        "expected annotation detail"
    );
    assert!(matches!(
        detail.declaration_context().unwrap(),
        CssDeclarationContextRef::Keyframe(_)
    ));

    let custom_source = "@keyframes fade { from { --phase: ready !important; } }";
    let report = CssParser::default().parse_sheet(custom_source);
    let [CssRule::Keyframes(keyframes)] = report.syntax().rules() else {
        panic!("invalid custom importance must leave the authored keyframes structure");
    };
    assert!(keyframes.blocks()[0].declarations().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid custom importance must produce one diagnostic");
    };
    let error = diagnostic.error();
    let detail = &error;
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidDeclarationAnnotation,
        "expected custom keyframe annotation detail"
    );
    let CssDeclarationContextRef::KeyframeCustomProperty(name) =
        detail.declaration_context().unwrap()
    else {
        panic!("expected custom keyframe context");
    };
    assert_eq!(name.as_str(), "--phase");
    assert_eq!(
        error.position().byte_offset().value(),
        custom_source.find('!').expect("bang")
    );
}

#[test]
fn declaration_importance_font_face_occurrences_retain_positions_and_reject_annotations() {
    let source = "@font-face { font-family: Inter; src: url(inter.woff2); font-display: swap; }";
    let sheet = CssParser::default()
        .parse_sheet(source)
        .expect("font-face must parse");
    let [CssRule::FontFace(rule)] = sheet.rules() else {
        panic!("expected font-face");
    };
    let descriptors = rule.descriptors();
    assert_eq!(
        ordinary_face!(descriptors, FontFamily).unwrap().as_str(),
        "Inter"
    );
    assert_eq!(
        descriptors
            .effective(surgeist_css::rule::font_face::CssFontFaceDescriptorKind::FontFamily)
            .unwrap()
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        13
    );
    assert_eq!(
        descriptors
            .effective(surgeist_css::rule::font_face::CssFontFaceDescriptorKind::Src)
            .unwrap()
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        33
    );
    assert_eq!(
        descriptors
            .effective(surgeist_css::rule::font_face::CssFontFaceDescriptorKind::FontDisplay)
            .expect("font-display")
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        56
    );

    for descriptor in [
        "font-family: Inter !important",
        "src: url(inter.woff2) !important",
        "font-weight: 400 !important",
        "font-style: normal !important",
        "font-stretch: 100% !important",
        "font-stretch: env(width) !important",
        "font-display: swap !important",
        "unicode-range: U+0-7F !important",
    ] {
        let source = format!("@font-face {{ font-family: Inter; src: url(i); {descriptor}; }}");
        let error = CssParser::default()
            .parse_sheet(&source)
            .expect_err("descriptor annotation must fail");
        assert_eq!(
            error.code(),
            CssErrorCode::InvalidDeclarationAnnotation,
            "{source}"
        );
        let detail = &error;
        assert_eq!(
            detail.code(),
            CssErrorCode::InvalidDeclarationAnnotation,
            "expected annotation detail"
        );
        let CssDeclarationContextRef::Descriptor {
            at_rule,
            descriptor: actual,
        } = detail.declaration_context().unwrap()
        else {
            panic!("expected descriptor context");
        };
        assert_eq!(at_rule.as_str(), "font-face");
        assert_eq!(
            actual.as_str(),
            descriptor.split(':').next().expect("descriptor name")
        );
        assert_eq!(detail.encountered().unwrap().authored(), "!");
        assert_eq!(
            error.position().byte_offset().value(),
            source.find('!').expect("bang")
        );
    }

    for authored in [
        "FoNt-DiSpLaY: swap !IMPORTANT",
        "font-display: swap !oops",
        "font-display: swap !important extra",
        "font-display: swap !important !important",
    ] {
        let source = format!("@font-face {{ font-family: Inter; src: url(i); {authored}; }}");
        let error = CssParser::default()
            .parse_sheet(&source)
            .expect_err("descriptor annotation must fail");
        assert_eq!(error.code(), CssErrorCode::InvalidDeclarationAnnotation);
        let detail = &error;
        assert_eq!(
            detail.code(),
            CssErrorCode::InvalidDeclarationAnnotation,
            "expected descriptor annotation detail"
        );
        let CssDeclarationContextRef::Descriptor {
            at_rule,
            descriptor,
        } = detail.declaration_context().unwrap()
        else {
            panic!("expected descriptor context");
        };
        assert_eq!(at_rule.as_str(), "font-face");
        assert_eq!(descriptor.as_str(), "font-display");
        assert_eq!(
            error.position().byte_offset().value(),
            source.find('!').expect("bang")
        );
    }
}
