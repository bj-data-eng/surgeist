//! Matched blocks remain declaration-value data under whole-value substitution
//! admission, including the palette owner's independent var permission.
use surgeist_css::{
    CssFontPaletteDescriptorKind as Kind, CssFontPaletteDescriptorValue,
    CssFontPaletteDescriptorValueRef, CssRule, parse_component_values,
    parse_font_palette_descriptor_value, parse_sheet,
};

#[test]
fn root_matched_blocks_remain_pending_in_every_descriptor() {
    for kind in [Kind::FontFamily, Kind::BasePalette, Kind::OverrideColors] {
        for value in ["{} var(--x)", "env(x) {}", "{var(--x)}"] {
            let source = format!(
                "@font-palette-values --x {{ font-family: Demo; {}: {value}; base-palette: dark; }} .after {{ color: red; }}",
                kind.css_name()
            );
            let report = parse_sheet(&source);
            assert!(
                report.is_clean(),
                "{}: {value}: {report:?}",
                kind.css_name()
            );
            let [CssRule::FontPaletteValues(rule), CssRule::Style(_)] = report.syntax().rules()
            else {
                panic!("valid sibling descriptors and following rule must survive");
            };
            assert_eq!(rule.descriptors().len(), 3);
            let fragment = parse_font_palette_descriptor_value(value, kind);
            assert!(
                fragment.is_clean(),
                "{}: {value}: {fragment:?}",
                kind.css_name()
            );
            let components = parse_component_values(value).unwrap();
            let constructed =
                CssFontPaletteDescriptorValue::try_new(kind, components.clone()).unwrap();
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
                    admitted.components().serialize().unwrap().as_css().trim(),
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
                kind.css_name()
            );
            let report = parse_sheet(&source);
            assert!(report.is_clean(), "{:?}", report.diagnostics());
            let [CssRule::FontPaletteValues(rule)] = report.syntax().rules() else {
                panic!("expected palette");
            };
            let parsed = rule.descriptors()[1].value();
            let fragment = parse_font_palette_descriptor_value(value, kind);
            assert!(fragment.is_clean());
            let constructed =
                CssFontPaletteDescriptorValue::try_new(kind, parsed.components().clone()).unwrap();
            for checked in [parsed, fragment.syntax().as_ref().unwrap(), &constructed] {
                assert!(matches!(
                    checked.view(),
                    CssFontPaletteDescriptorValueRef::Pending(_)
                ));
                assert_eq!(checked.to_specified_css().unwrap().trim(), value);
            }
        }
    }
}
