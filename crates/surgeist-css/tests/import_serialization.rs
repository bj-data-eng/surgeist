//! Canonical imports preserve the authored clause/media interpretation.
use surgeist_css::{
    CssImportSerializationError, CssMediaSerializationError, CssRule, CssSerializedOrigin,
    CssValueOrigin, parse_sheet,
};

#[test]
fn canonical_imports_preserve_targets_clauses_and_symbolic_media() {
    for (source, expected) in [
        ("@IMPORT 'theme.css';", "@import 'theme.css';"),
        (
            "@import url(theme.css) layer(theme) supports(display:grid) SCREEN AND (WIDTH>=+001.5PX);",
            "@import url(theme.css) layer(theme) supports(display:grid) screen and (width >= +001.5PX);",
        ),
        (
            "@import \"x\" (ASPECT-RATIO:2), print;",
            "@import \"x\" (aspect-ratio: 2 / 1), print;",
        ),
        (
            "@import \"x\" future(1/**/e2);",
            "@import \"x\" future(1/**/e2);",
        ),
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::Import(import)] = report.syntax().rules() else {
            panic!("one import");
        };
        assert_eq!(import.serialize().unwrap().as_css(), expected);
    }
}

#[test]
fn clause_shaped_media_keeps_its_interpretation_after_serialization() {
    for tail in [
        "layer()",
        "supports(2px)",
        "layer(initial)",
        "layer(theme) and (color)",
        "supports(display:grid) and (color)",
    ] {
        let source = format!("@import \"x\" {tail};");
        let report = parse_sheet(&source);
        assert!(report.is_clean());
        let [CssRule::Import(import)] = report.syntax().rules() else {
            panic!("one import");
        };
        assert!(import.layer().is_none());
        assert!(import.supports().is_none());
        let output = import.serialize().unwrap();
        assert_eq!(output.as_css(), source);
        let reparsed = parse_sheet(output.as_css());
        assert!(reparsed.is_clean());
        let [CssRule::Import(import)] = reparsed.syntax().rules() else {
            panic!("one reparsed import");
        };
        assert!(import.layer().is_none());
        assert!(import.supports().is_none());
        assert_eq!(import.media().unwrap().queries().len(), 1);
    }
}

#[test]
fn recovered_media_prevents_partial_import_output() {
    let report = parse_sheet("@import \"x\" layer(theme) screen and, print;");
    assert!(!report.is_clean());
    let [CssRule::Import(import)] = report.syntax().rules() else {
        panic!("retained import");
    };
    assert!(matches!(
        import.serialize(),
        Err(CssImportSerializationError::Media(
            CssMediaSerializationError::RecoveredNever {
                origin: CssValueOrigin::Parsed(_)
            }
        ))
    ));
}

#[test]
fn emitted_target_and_terminator_keep_original_source_coordinates() {
    let source = "/* prefix */ @IMPORT 'x';";
    let report = parse_sheet(source);
    let [CssRule::Import(import)] = report.syntax().rules() else {
        panic!("one import");
    };
    let output = import.serialize().unwrap();
    assert_eq!(output.as_css(), "@import 'x';");
    for (offset, original) in [(0, 13), (8, 21), (11, 24)] {
        let Some(CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin))) =
            output.origin_at(offset)
        else {
            panic!("original token at {offset}");
        };
        assert_eq!(origin.span().start().byte_offset().value(), original);
    }
}

#[test]
fn eof_recovery_closes_output_without_inventing_source_coordinates() {
    let report = parse_sheet("@import \"x\" layer(");
    assert!(!report.is_clean());
    let [CssRule::Import(import)] = report.syntax().rules() else {
        panic!("retained import");
    };
    let output = import.serialize().unwrap();
    assert_eq!(output.as_css(), "@import \"x\" layer();");
    assert!(matches!(
        output.origin_at(output.as_css().len() - 1),
        Some(CssSerializedOrigin::Token(CssValueOrigin::Programmatic))
    ));
    assert!(parse_sheet(output.as_css()).is_clean());
}

#[test]
fn deeply_nested_import_media_serializes_without_losing_components() {
    let source = format!(
        "@import \"x\" {}1{};",
        "future(".repeat(256),
        ")".repeat(256)
    );
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Import(import)] = report.syntax().rules() else {
        panic!("one import");
    };
    assert_eq!(import.serialize().unwrap().as_css(), source);
}
