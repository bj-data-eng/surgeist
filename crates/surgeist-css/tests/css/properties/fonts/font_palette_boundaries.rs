//! Matched blocks remain declaration-value data under whole-value substitution
//! admission, including the palette owner's independent var permission.
use surgeist_css::{
    CssComponentValues, CssFontPaletteDescriptorKind as Kind, CssFontPaletteDescriptorValue,
    CssFontPaletteDescriptorValueRef, CssParser, CssRule,
};

#[test]
fn root_matched_blocks_remain_pending_in_every_descriptor() {
    for kind in [Kind::FontFamily, Kind::BasePalette, Kind::OverrideColors] {
        for value in ["{} var(--x)", "env(x) {}", "{var(--x)}"] {
            let source = format!(
                "@font-palette-values --x {{ font-family: Demo; {}: {value}; base-palette: dark; }} .after {{ color: red; }}",
                kind.as_str()
            );
            let report = CssParser::default().parse_sheet(&source);
            assert!(report.is_clean(), "{}: {value}: {report:?}", kind.as_str());
            let [CssRule::FontPaletteValues(rule), CssRule::Style(_)] = report.syntax().rules()
            else {
                panic!("valid sibling descriptors and following rule must survive");
            };
            assert_eq!(rule.descriptors().len(), 3);
            let fragment = CssFontPaletteDescriptorValue::parse(&CssParser::default(), value, kind);
            assert!(
                fragment.is_clean(),
                "{}: {value}: {fragment:?}",
                kind.as_str()
            );
            let components = CssComponentValues::parse(&CssParser::default(), value).unwrap();
            let constructed = CssFontPaletteDescriptorValue::try_new(
                kind,
                components.clone(),
                Default::default(),
            )
            .unwrap();
            assert_eq!(constructed.components(), &components);
            for admitted in [
                rule.descriptors()[1].value(),
                fragment.syntax().as_ref().unwrap(),
                &constructed,
            ] {
                assert_eq!(admitted.kind(), kind);
                assert!(matches!(
                    admitted.view(),
                    CssFontPaletteDescriptorValueRef::Pending(_)
                ));
                assert_eq!(
                    admitted
                        .components()
                        .serialize_tokens(Default::default())
                        .unwrap()
                        .as_css()
                        .trim(),
                    value
                );
            }
        }
    }
}

#[test]
fn nested_fallback_blocks_remain_pending_across_descriptor_entry_points() {
    for kind in [Kind::FontFamily, Kind::BasePalette, Kind::OverrideColors] {
        for value in ["var(--x, {hello})", "env(x, {hello})"] {
            let source = format!(
                "@font-palette-values --x {{ font-family: Demo; {}: {value}; }}",
                kind.as_str()
            );
            let report = CssParser::default().parse_sheet(&source);
            assert!(report.is_clean(), "{:?}", report.diagnostics());
            let [CssRule::FontPaletteValues(rule)] = report.syntax().rules() else {
                panic!("expected palette");
            };
            let parsed = rule.descriptors()[1].value();
            let fragment = CssFontPaletteDescriptorValue::parse(&CssParser::default(), value, kind);
            assert!(fragment.is_clean());
            let constructed = CssFontPaletteDescriptorValue::try_new(
                kind,
                parsed.components().clone(),
                Default::default(),
            )
            .unwrap();
            for checked in [parsed, fragment.syntax().as_ref().unwrap(), &constructed] {
                assert!(matches!(
                    checked.view(),
                    CssFontPaletteDescriptorValueRef::Pending(_)
                ));
                assert_eq!(
                    checked
                        .serialize_specified(Default::default())
                        .unwrap()
                        .trim(),
                    value
                );
            }
        }
    }
}
