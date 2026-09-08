mod common;

use common::CssParseReportTestExt;
use surgeist_css::{
    CssDeclarationContextRef, CssErrorCode, CssImportance, CssPropertyNameRef, CssRule,
    CssTokenKind, ErrorKind, parse_sheet,
};

fn style_declarations(source: &str) -> surgeist_css::CssDeclarationList {
    let sheet = parse_sheet(source).expect("stylesheet must parse");
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

#[test]
fn declaration_importance_malformed_annotations_report_first_bang() {
    for (source, bang) in [
        (".x { width: 1px !; }", 16),
        (".x { width: 1px !imporant; }", 16),
        (".x { width: 1px !important !important; }", 16),
        (".x { width: 1px !important extra; }", 16),
        (".x { --custom: value !oops; }", 21),
    ] {
        let error = parse_sheet(source).expect_err("annotation must be rejected");
        assert_eq!(
            error.code(),
            CssErrorCode::InvalidDeclarationAnnotation,
            "{source}"
        );
        assert_eq!(error.position().byte_offset().value(), bang, "{source}");
        let ErrorKind::InvalidDeclarationAnnotation(detail) = error.kind() else {
            panic!("expected annotation detail for {source}");
        };
        assert_eq!(detail.encountered().kind(), CssTokenKind::Delim);
        assert_eq!(detail.encountered().authored(), "!");
        match detail.context() {
            CssDeclarationContextRef::KnownProperty(_) if !source.contains("--custom") => {}
            CssDeclarationContextRef::CustomProperty(name) => {
                assert_eq!(name.as_str(), "--custom");
            }
            context => panic!("unexpected context {context:?}"),
        }
    }
}

#[test]
fn declaration_importance_strict_sheet_rejects_bad_declaration_between_valid_siblings() {
    let source = ".x { height: 1px; width: 2px !oops; opacity: 1; }";
    let error = parse_sheet(source).expect_err("current strict parser must reject the sheet");
    assert_eq!(error.code(), CssErrorCode::InvalidDeclarationAnnotation);
    assert_eq!(
        error.position().byte_offset().value(),
        source.find('!').expect("bang")
    );
}

#[test]
fn declaration_importance_keyframes_use_distinct_unimportant_declarations() {
    let sheet = parse_sheet("@keyframes fade { from { opacity: 0 } to { opacity: 1; } }")
        .expect("keyframes must parse");
    let [CssRule::Keyframes(rule)] = sheet.rules() else {
        panic!("expected keyframes");
    };
    let declarations: &surgeist_css::CssKeyframeDeclarationList = rule.blocks()[0].declarations();
    assert_eq!(declarations.as_slice().len(), 1);
    assert_eq!(declarations.iter().count(), 1);
    assert!(!declarations.is_empty());
    assert!(matches!(
        declarations.as_slice()[0].property_name(),
        CssPropertyNameRef::Known(_)
    ));

    let source = "@keyframes fade { from { opacity: 0 !important; } }";
    let report = parse_sheet(source);
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
    let ErrorKind::InvalidDeclarationAnnotation(detail) = error.kind() else {
        panic!("expected annotation detail");
    };
    assert!(matches!(
        detail.context(),
        CssDeclarationContextRef::Keyframe(_)
    ));

    let custom_source = "@keyframes fade { from { --phase: ready !important; } }";
    let report = parse_sheet(custom_source);
    let [CssRule::Keyframes(keyframes)] = report.syntax().rules() else {
        panic!("invalid custom importance must leave the authored keyframes structure");
    };
    assert!(keyframes.blocks()[0].declarations().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid custom importance must produce one diagnostic");
    };
    let error = diagnostic.error();
    let ErrorKind::InvalidDeclarationAnnotation(detail) = error.kind() else {
        panic!("expected custom keyframe annotation detail");
    };
    let CssDeclarationContextRef::KeyframeCustomProperty(name) = detail.context() else {
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
    let sheet = parse_sheet(source).expect("font-face must parse");
    let [CssRule::FontFace(rule)] = sheet.rules() else {
        panic!("expected font-face");
    };
    let descriptors = rule.descriptors();
    assert_eq!(descriptors.font_family().value().as_str(), "Inter");
    assert_eq!(
        descriptors.font_family().position().byte_offset().value(),
        13
    );
    assert_eq!(descriptors.src().position().byte_offset().value(), 33);
    assert_eq!(
        descriptors
            .font_display()
            .expect("font-display")
            .position()
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
        "font-display: swap !important",
        "unicode-range: U+0-7F !important",
    ] {
        let source = format!("@font-face {{ font-family: Inter; src: url(i); {descriptor}; }}");
        let error = parse_sheet(&source).expect_err("descriptor annotation must fail");
        assert_eq!(
            error.code(),
            CssErrorCode::InvalidDeclarationAnnotation,
            "{source}"
        );
        let ErrorKind::InvalidDeclarationAnnotation(detail) = error.kind() else {
            panic!("expected annotation detail");
        };
        let CssDeclarationContextRef::Descriptor {
            at_rule,
            descriptor: actual,
        } = detail.context()
        else {
            panic!("expected descriptor context");
        };
        assert_eq!(at_rule.as_str(), "font-face");
        assert_eq!(
            actual.as_str(),
            descriptor.split(':').next().expect("descriptor name")
        );
        assert_eq!(detail.encountered().authored(), "!");
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
        let error = parse_sheet(&source).expect_err("descriptor annotation must fail");
        assert_eq!(error.code(), CssErrorCode::InvalidDeclarationAnnotation);
        let ErrorKind::InvalidDeclarationAnnotation(detail) = error.kind() else {
            panic!("expected descriptor annotation detail");
        };
        let CssDeclarationContextRef::Descriptor {
            at_rule,
            descriptor,
        } = detail.context()
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
