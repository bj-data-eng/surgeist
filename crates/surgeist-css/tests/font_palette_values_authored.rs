//! Fonts 4 §9.2 admits an authored palette definition as a global rule.
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-palette-values

use surgeist_css::{CssRule, parse_sheet};

#[test]
fn valid_palette_definition_precedes_following_style_without_recovery() {
    let report = parse_sheet(
        "@font-palette-values --theme { font-family: Demo; base-palette: light; } \
         .after { color: red; }",
    );

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 2);
    assert!(matches!(report.syntax().rules()[1], CssRule::Style(_)));
}
