#![forbid(unsafe_code)]
//! Inspect typed descriptor payloads without matching or modifying a font.

use surgeist_css::{
    CssAuthoredFontFaceDescriptorValue as Authored, CssFontFaceDescriptorKind as Kind,
    CssFontFaceDescriptorValue as Value, CssFontMetricOverride, CssRule, parse_component_values,
    parse_sheet,
};

fn main() {
    let source = concat!(
        "@font-face{font-named-instance:'Grotesque';font-language-override:'TRK';",
        "ascent-override:125%;descent-override:normal;line-gap-override:env(gap,0%)}"
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("font-face payload")
    };
    assert_eq!(face.descriptors().occurrences().len(), 5);
    let Authored::Ordinary(Value::AscentOverride(ascent)) = face
        .descriptors()
        .effective(Kind::AscentOverride)
        .unwrap()
        .value()
    else {
        panic!("ordinary ascent percentage")
    };
    assert_eq!(ascent.serialize_specified().unwrap(), "125%");
    let Authored::Pending(gap) = face
        .descriptors()
        .effective(Kind::LineGapOverride)
        .unwrap()
        .value()
    else {
        panic!("whole-descriptor environment value")
    };
    let Value::LineGapOverride(CssFontMetricOverride::Percentage(gap)) = gap
        .reparse_after_substitution(parse_component_values("0%").unwrap())
        .unwrap()
    else {
        panic!("strict ordinary replacement")
    };
    println!("line-gap-override: {}", gap.serialize_specified().unwrap());
}
