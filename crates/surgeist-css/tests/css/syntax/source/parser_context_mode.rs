use crate::css::support::authored_syntax::{assert_diagnostics, assert_invalid_property_value};
use surgeist_css::declaration::CssDeclaration;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::CssKnownProperty;
use surgeist_css::{CssParser, CssParserContext, CssParserMode};

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
                let report = CssDeclaration::parse(
                    &CssParser::new(changed, Default::default()),
                    "glyph-orientation-vertical:90deg",
                );
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
    let report = CssDeclaration::parse(&CssParser::new(quirks, Default::default()), "width:7");
    assert!(report.is_clean());
    assert_eq!(report.syntax().as_ref().unwrap().parser_context(), quirks);
    for standards in [original, quirks.with_mode(CssParserMode::Standards)] {
        assert_eq!(standards, original);
        let report =
            CssDeclaration::parse(&CssParser::new(standards, Default::default()), "width:7");
        assert!(report.syntax().is_none());
        // Standards mode has no unitless-length quirk, so the number fails
        // the width grammar.
        let [diagnostic] = report.diagnostics() else {
            panic!("one width grammar failure: {report:?}")
        };
        assert_invalid_property_value(diagnostic.error(), CssKnownProperty::Width, Some("7"));
        assert_diagnostics(
            report.diagnostics(),
            &[(
                CssErrorCode::InvalidPropertyValue,
                CssRecoveryAction::RejectInput,
                "width:".len(),
            )],
        );
    }
    assert_eq!(CssParserContext::default().mode(), CssParserMode::Standards);
    assert_eq!(
        CssParserContext::new(CssParserMode::Quirks).mode(),
        CssParserMode::Quirks
    );
}
