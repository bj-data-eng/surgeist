#![forbid(unsafe_code)]
//! Inspect authored synthesis choices without selecting or altering a font.
use surgeist_css::CssParser;
use surgeist_css::property::{CssContributions, CssExpansion, CssKnownPropertyValueRef};
use surgeist_css::values::font::{CssFontSynthesis, CssFontSynthesisValues};

fn main() {
    let report =
        CssParser::default().parse_block_contents("font-synthesis:position weight!important");
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    let CssKnownPropertyValueRef::FontSynthesis(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary synthesis")
    };
    assert_eq!(
        value
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "weight position"
    );
    let checked = CssFontSynthesis::Values(
        CssFontSynthesisValues::try_new(true, false, false, true).unwrap(),
    );
    assert_eq!(value.value(), &checked);
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = source.expand().unwrap()
    else {
        panic!("four intrinsic longhand contributions")
    };
    assert_eq!(values.items().len(), 4);
    for value in values.items() {
        assert!(value.source().same_occurrence(source));
    }
    println!(
        "{}",
        checked.serialize_specified(Default::default()).unwrap()
    );
}
