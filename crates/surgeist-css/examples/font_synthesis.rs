#![forbid(unsafe_code)]
//! Inspect authored synthesis choices without selecting or altering a font.
use surgeist_css::{
    CssContributions, CssExpansion, CssFontSynthesis, CssFontSynthesisValues,
    CssKnownPropertyValueRef, expand_declaration, parse_style_attribute,
};

fn main() {
    let report = parse_style_attribute("font-synthesis:position weight!important");
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    let CssKnownPropertyValueRef::FontSynthesis(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary synthesis")
    };
    assert_eq!(
        value.synthesis().serialize_specified().unwrap(),
        "weight position"
    );
    let checked = CssFontSynthesis::Values(
        CssFontSynthesisValues::try_new(true, false, false, true).unwrap(),
    );
    assert_eq!(value.synthesis(), &checked);
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("four intrinsic longhand contributions")
    };
    assert_eq!(values.items().len(), 4);
    for value in values.items() {
        assert!(value.source().same_occurrence(source));
    }
    println!("{}", checked.serialize_specified().unwrap());
}
