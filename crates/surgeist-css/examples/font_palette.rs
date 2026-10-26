#![forbid(unsafe_code)]
//! Inspect a symbolic palette mix without loading or mixing a font palette.
use surgeist_css::{
    CssContributions, CssExpansion, CssFontPalette, CssFontPaletteMix, CssFontPaletteMixComponent,
    CssKnownPropertyValueRef, CssParser,
};

fn main() {
    let report = CssParser::default()
        .parse_block_contents("font-palette:palette-mix(30% light, dark)!important");
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    let CssKnownPropertyValueRef::FontPalette(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary palette")
    };
    assert_eq!(
        value
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "palette-mix(light 30%, dark 70%)"
    );
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = source.expand().unwrap()
    else {
        panic!("intrinsic palette longhand")
    };
    assert_eq!(values.items().len(), 1);
    assert!(values.items()[0].source().same_occurrence(source));
    let checked = CssFontPalette::Mix(Box::new(
        CssFontPaletteMix::try_new(
            None,
            vec![
                CssFontPaletteMixComponent::new(CssFontPalette::Light, None),
                CssFontPaletteMixComponent::new(CssFontPalette::Dark, None),
            ],
        )
        .unwrap(),
    ));
    assert_eq!(
        checked.serialize_specified(Default::default()).unwrap(),
        "palette-mix(light, dark)"
    );
    println!(
        "{}",
        checked.serialize_specified(Default::default()).unwrap()
    );
}
