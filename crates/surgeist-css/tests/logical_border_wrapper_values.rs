#![forbid(unsafe_code)]

use surgeist_css::*;

fn checked_color(value: &str) -> CssAuthoredColor {
    let report = parse_style_attribute(&format!("border-top-color:{value}"));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let Some(CssKnownPropertyValueRef::BorderTopColor(color)) =
        report.syntax()[0].known().unwrap().property_value()
    else {
        panic!("checked border color");
    };
    color.current().clone()
}

fn logical_current<'a>(name: &str, value: CssKnownPropertyValueRef<'a>) -> &'a CssBorderValue {
    match (name, value) {
        ("border-block-start", CssKnownPropertyValueRef::BorderBlockStart(value)) => {
            value.current()
        }
        ("border-block-end", CssKnownPropertyValueRef::BorderBlockEnd(value)) => value.current(),
        ("border-inline-start", CssKnownPropertyValueRef::BorderInlineStart(value)) => {
            value.current()
        }
        ("border-inline-end", CssKnownPropertyValueRef::BorderInlineEnd(value)) => value.current(),
        ("border-block", CssKnownPropertyValueRef::BorderBlock(value)) => value.current(),
        ("border-inline", CssKnownPropertyValueRef::BorderInline(value)) => value.current(),
        _ => panic!("logical border wrapper for {name}"),
    }
}

#[test]
fn six_logical_wrappers_expose_the_shared_checked_triple_and_preserve_omission() {
    let expected = CssBorderValue::try_new(
        Some(CssBorderWidth::Thin),
        Some(CssBorderStyle::Solid),
        Some(checked_color("red")),
    )
    .unwrap();
    let omitted = CssBorderValue::try_new(None, Some(CssBorderStyle::Solid), None).unwrap();
    for name in [
        "border-block-start",
        "border-block-end",
        "border-inline-start",
        "border-inline-end",
        "border-block",
        "border-inline",
    ] {
        for (authored, model, canonical) in [
            ("red solid thin", &expected, "thin solid red"),
            ("solid", &omitted, "solid"),
        ] {
            let report = parse_style_attribute(&format!("{name}:{authored}"));
            assert!(report.is_clean(), "{name}: {:?}", report.diagnostics());
            let value = report.syntax()[0]
                .known()
                .unwrap()
                .property_value()
                .unwrap();
            let current = logical_current(name, value);
            assert_eq!(current, model, "{name}:{authored}");
            assert_eq!(current.serialize_specified().unwrap(), canonical);
        }
        let report = parse_style_attribute(&format!("{name}:oklch(50% 0.2 30deg) solid 1e100px"));
        assert!(report.is_clean(), "{name}: {:?}", report.diagnostics());
        let value = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap();
        let current = logical_current(name, value);
        assert_eq!(
            current.serialize_specified().unwrap(),
            format!("1{}px solid oklch(0.5 0.2 30)", "0".repeat(100))
        );
    }
}
