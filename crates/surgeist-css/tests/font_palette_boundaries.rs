//! The accepted palette contract forbids root descriptor delimiters before
//! whole-value substitution admission, while nested fallback blocks remain data.
use surgeist_css::{
    CssFontPaletteDescriptorKind as Kind, CssFontPaletteDescriptorValue,
    CssFontPaletteDescriptorValueRef, CssRecoveryAction, CssRule, parse_component_values,
    parse_font_palette_descriptor_value, parse_sheet,
};

#[test]
fn root_blocks_are_rejected_before_pending_admission_in_every_descriptor() {
    for kind in [Kind::FontFamily, Kind::BasePalette, Kind::OverrideColors] {
        for value in ["{} var(--x)", "env(x) {}", "{var(--x)}"] {
            let source = format!(
                "@font-palette-values --x {{ font-family: Demo; {}: {value}; base-palette: dark; }} .after {{ color: red; }}",
                kind.css_name()
            );
            let report = parse_sheet(&source);
            assert!(!report.is_clean(), "{}: {value}", kind.css_name());
            let [CssRule::FontPaletteValues(rule), CssRule::Style(_)] = report.syntax().rules()
            else {
                panic!("valid sibling descriptors and following rule must survive");
            };
            assert_eq!(rule.descriptors().len(), 2);
            assert_eq!(report.diagnostics().len(), 1);
            assert_eq!(
                report.diagnostics()[0].action(),
                CssRecoveryAction::DropDescriptor
            );
            assert_eq!(
                report.diagnostics()[0]
                    .error()
                    .position()
                    .byte_offset()
                    .value(),
                source.find(value).unwrap() + value.find('{').unwrap()
            );
            assert!(
                parse_font_palette_descriptor_value(value, kind)
                    .syntax()
                    .is_none()
            );
            assert!(
                CssFontPaletteDescriptorValue::try_new(
                    kind,
                    parse_component_values(value).unwrap()
                )
                .is_err()
            );
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
