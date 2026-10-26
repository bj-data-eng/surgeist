//! Values 4 Appendix C (WD 2024-03-12) makes `<quirky-length>` "a type of
//! `<length>`" in the listed properties, so a quirks-mode Number denotes the
//! px length with the same value. CSSOM "serialize a CSS value" step 2 omits
//! components when that does not change the meaning, so a quirky member and an
//! equal explicit px member select the same shortest shorthand form. Each
//! quirky member still serializes as its own `Npx` text.

use surgeist_css::*;

const QUIRKS: CssParserContext = CssParserContext::new(CssParserMode::Quirks);

fn cssom(context: CssParserContext, source: &str) -> String {
    let report = CssParser::new(context, Default::default()).parse_block_contents(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax(), Default::default())
        .unwrap()
        .serialize_cssom(Default::default())
        .unwrap()
}

#[test]
fn quirky_and_explicit_px_members_with_equal_values_shorten_together() {
    for (source, expected) in [
        (
            "margin-top:5px;margin-right:5px;margin-bottom:5px;margin-left:5px",
            "margin: 5px;",
        ),
        (
            "margin-top:5;margin-right:5;margin-bottom:5;margin-left:5",
            "margin: 5px;",
        ),
        (
            "margin-top:5;margin-right:5px;margin-bottom:5px;margin-left:5px",
            "margin: 5px;",
        ),
        ("margin:5 5px 5px 5px", "margin: 5px;"),
        ("margin:5 5px", "margin: 5px;"),
        ("margin:5px 5", "margin: 5px;"),
        ("margin:5 6px 5 6PX", "margin: 5px 6px;"),
        ("padding:5 5px", "padding: 5px;"),
        ("padding:5px 5 5PX", "padding: 5px;"),
        ("border-width:5 5px", "border-width: 5px;"),
        ("border-width:2px 2 2 2px", "border-width: 2px;"),
    ] {
        assert_eq!(cssom(QUIRKS, source), expected, "{source}");
    }
}

#[test]
fn quirky_members_stay_distinct_from_unequal_values_and_other_units() {
    for (source, expected) in [
        ("margin:5 6px", "margin: 5px 6px;"),
        ("margin:5 5em", "margin: 5px 5em;"),
        ("margin:5 5%", "margin: 5px 5%;"),
        ("padding:5 5pt", "padding: 5px 5pt;"),
        ("border-width:5 5em", "border-width: 5px 5em;"),
        ("border-width:5 6", "border-width: 5px 6px;"),
    ] {
        assert_eq!(cssom(QUIRKS, source), expected, "{source}");
    }
}

#[test]
fn standards_mode_px_shortening_is_unchanged() {
    let standards = CssParserContext::new(CssParserMode::Standards);
    for (source, expected) in [
        ("margin:5px 5px", "margin: 5px;"),
        ("margin:5px 5em", "margin: 5px 5em;"),
        ("padding:5px 5PX 5px", "padding: 5px;"),
        ("border-width:5px 5px", "border-width: 5px;"),
    ] {
        assert_eq!(cssom(standards, source), expected, "{source}");
    }
}
