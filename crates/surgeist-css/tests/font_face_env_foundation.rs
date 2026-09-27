#![forbid(unsafe_code)]

//! Existing-public-API admission evidence from CSS Env 1 WD (2025-09-23) §3.
//! A syntactically valid `env()` defers the entire known descriptor grammar.

use surgeist_css::{
    CssFontFaceDescriptorKind as Kind, CssRecoveryAction, CssRule,
    parse_font_face_descriptor_value, parse_sheet, validate_sheet,
};

fn one_font_face(source: &str) -> surgeist_css::CssParseReport<surgeist_css::CssSheet> {
    let report = parse_sheet(source);
    assert!(
        matches!(report.syntax().rules(), [CssRule::FontFace(_)]),
        "font-face rule retained: {source}: {report:?}"
    );
    report
}

#[test]
fn ordinary_descriptors_still_parse_and_plain_var_does_not_defer_them() {
    for (kind, ordinary) in [
        (Kind::FontFamily, "Demo"),
        (Kind::Src, "url(face)"),
        (Kind::FontWeight, "400"),
        (Kind::FontStyle, "italic"),
        (Kind::FontWidth, "75%"),
        (Kind::FontDisplay, "swap"),
        (Kind::UnicodeRange, "U+20"),
        (Kind::FontFeatureSettings, "normal"),
    ] {
        let report = parse_font_face_descriptor_value(ordinary, kind);
        assert!(report.is_clean(), "{}: {report:?}", kind.css_name());
        assert!(report.syntax().is_some());

        let variable = parse_font_face_descriptor_value("var(--value)", kind);
        assert!(
            variable.syntax().is_none(),
            "{}: {variable:?}",
            kind.css_name()
        );
        assert!(
            variable
                .diagnostics()
                .iter()
                .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::RejectInput })
        );
    }

    let unknown = one_font_face("@font-face{font-family:Demo;mystery:env(value);src:url(face)}");
    assert!(!unknown.is_clean());
    assert!(
        unknown
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::DropDescriptor })
    );
}

#[test]
fn valid_whole_env_values_are_clean_for_every_known_descriptor_and_both_width_names() {
    for (kind, name) in [
        (Kind::FontFamily, "font-family"),
        (Kind::Src, "src"),
        (Kind::FontWeight, "font-weight"),
        (Kind::FontStyle, "font-style"),
        (Kind::FontWidth, "font-width"),
        (Kind::FontWidth, "font-stretch"),
        (Kind::FontDisplay, "font-display"),
        (Kind::UnicodeRange, "unicode-range"),
        (Kind::FontFeatureSettings, "font-feature-settings"),
    ] {
        let raw = parse_font_face_descriptor_value("env(value)", kind);
        assert!(raw.is_clean(), "raw {name}: {raw:?}");
        assert!(raw.syntax().is_some(), "raw {name}");
        assert!(raw.into_validation_result().is_ok(), "raw {name}");

        let source = format!("@font-face{{{name}:env(value)}}");
        let sheet = one_font_face(&source);
        assert!(sheet.is_clean(), "sheet {name}: {sheet:?}");
        assert!(validate_sheet(&source).is_ok(), "sheet {name}");
    }
}

#[test]
fn one_valid_env_defers_otherwise_invalid_ordinary_tokens_without_src_recovery() {
    for (kind, name, value) in [
        (Kind::Src, "src", "bogus, env(source)"),
        (Kind::FontWidth, "font-width", "wide env(width)"),
        (Kind::FontStyle, "font-style", "no-such-style env(style)"),
        (
            Kind::FontDisplay,
            "font-display",
            "env(display) var(--later)",
        ),
        (Kind::FontWidth, "font-stretch", "125% env(width, 75%)"),
    ] {
        let raw = parse_font_face_descriptor_value(value, kind);
        assert!(raw.is_clean(), "raw {name}:{value}: {raw:?}");
        assert!(raw.syntax().is_some());
        assert!(!raw.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::DropFontSourceListItem
        }));

        let source = format!("@font-face{{{name}:{value}}}");
        let sheet = one_font_face(&source);
        assert!(sheet.is_clean(), "sheet {name}:{value}: {sheet:?}");
        assert!(validate_sheet(&source).is_ok());
        assert!(!sheet.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::DropFontSourceListItem
        }));
    }
}

#[test]
fn nested_valid_fallback_qualifies_for_deferral() {
    for value in [
        "env(width, env(fallback, 75%))",
        "calc(env(width, 75%) + 25%)",
    ] {
        let raw = parse_font_face_descriptor_value(value, Kind::FontWidth);
        assert!(raw.is_clean(), "{value}: {raw:?}");
        assert!(raw.syntax().is_some());
    }
}

#[test]
fn malformed_env_and_mixed_valid_invalid_env_family_do_not_qualify() {
    for value in ["env()", "env(123)", "env(valid) env()"] {
        let raw = parse_font_face_descriptor_value(value, Kind::FontWidth);
        assert!(raw.syntax().is_none(), "{value}: {raw:?}");
        assert!(!raw.is_clean());
        let source = format!("@font-face{{font-width:{value}}}");
        let sheet = one_font_face(&source);
        assert!(!sheet.is_clean(), "{value}: {sheet:?}");
        assert!(
            sheet
                .diagnostics()
                .iter()
                .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::DropDescriptor })
        );
    }
}

#[test]
fn importance_and_raw_root_boundaries_are_not_rescued_by_env() {
    for value in ["env(width)!important", "env(width);", "env(width){}"] {
        let raw = parse_font_face_descriptor_value(value, Kind::FontWidth);
        assert!(raw.syntax().is_none(), "{value}: {raw:?}");
        assert!(
            raw.diagnostics()
                .iter()
                .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::RejectInput })
        );
    }
    let source = "@font-face{font-width:env(width)!important;font-display:swap}";
    let sheet = one_font_face(source);
    assert!(!sheet.is_clean());
    assert!(
        sheet
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::DropDescriptor })
    );
}

#[test]
fn eof_implicit_env_closure_is_retained_with_diagnostics_and_not_clean_validation() {
    let raw = parse_font_face_descriptor_value("env(width", Kind::FontWidth);
    assert!(raw.syntax().is_some(), "raw: {raw:?}");
    assert!(!raw.is_clean());
    assert!(
        raw.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        })
    );
    assert!(raw.into_validation_result().is_err());

    let source = "@font-face{font-width:env(width";
    let sheet = one_font_face(source);
    assert!(!sheet.is_clean());
    assert!(
        sheet.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        })
    );
    assert!(validate_sheet(source).is_err());
}
