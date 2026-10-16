#![forbid(unsafe_code)]
use surgeist_css::{CssParserContext, CssParserMode};

const QUIRKS_SVG: CssParserContext = CssParserContext::new(CssParserMode::Standards)
    .with_svg_glyph_orientation_vertical()
    .with_mode(CssParserMode::Quirks);

#[test]
fn selecting_each_mode_preserves_the_independent_glyph_definition() {
    assert_eq!(QUIRKS_SVG.mode(), CssParserMode::Quirks);
    for original_mode in [CssParserMode::Standards, CssParserMode::Quirks] {
        for svg in [false, true] {
            let original = CssParserContext::new(original_mode);
            let original = if svg {
                original.with_svg_glyph_orientation_vertical()
            } else {
                original
            };
            for mode in [CssParserMode::Standards, CssParserMode::Quirks] {
                let changed = original.with_mode(mode);
                assert_eq!(original.mode(), original_mode);
                assert_eq!(changed.mode(), mode);
                assert_eq!(changed.with_mode(mode), changed);
                assert_eq!(changed.with_mode(original_mode), original);
                let report = changed.parse_declaration("glyph-orientation-vertical:90deg");
                assert!(report.is_clean(), "{:?}", report.diagnostics());
                let declaration = report.syntax().as_ref().unwrap();
                assert_eq!(declaration.parser_context(), changed);
                assert_eq!(declaration.svg_glyph_orientation_vertical().is_some(), svg);
                assert_eq!(declaration.known().is_some(), !svg);
            }
        }
    }
}

#[test]
fn changed_mode_controls_quirky_grammar_without_resetting_the_original_context() {
    let original =
        CssParserContext::new(CssParserMode::Standards).with_svg_glyph_orientation_vertical();
    let quirks = original.with_mode(CssParserMode::Quirks);
    let report = quirks.parse_declaration("width:7");
    assert!(report.is_clean());
    assert_eq!(report.syntax().as_ref().unwrap().parser_context(), quirks);
    for standards in [original, quirks.with_mode(CssParserMode::Standards)] {
        assert_eq!(standards, original);
        let report = standards.parse_declaration("width:7");
        assert!(report.syntax().is_none());
        assert!(!report.is_clean());
    }
    assert_eq!(CssParserContext::default().mode(), CssParserMode::Standards);
    assert_eq!(
        CssParserContext::new(CssParserMode::Quirks).mode(),
        CssParserMode::Quirks
    );
}
