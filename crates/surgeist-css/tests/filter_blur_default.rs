#![forbid(unsafe_code)]
//! Filter Effects 1 section 6.1 defines an omitted blur radius as 0px.
//! https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/#funcdef-filter-blur
use surgeist_css::{
    CssFilter, CssFilterFunction, CssFilterFunctionValue, CssFilterValue, CssKnownPropertyValueRef,
    CssLength, CssRecoveryAction, parse_style_attribute,
};

#[test]
fn omitted_blur_radius_retains_authored_input_and_specified_zero() {
    for source in [
        "filter:blur()",
        "filter:blur(/**/)",
        "backdrop-filter:blur()",
    ] {
        let report = parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let declaration = &report.syntax()[0];
        let value = declaration.known().unwrap().property_value().unwrap();
        let (current, has_legacy) = match value {
            CssKnownPropertyValueRef::Filter(value) => {
                (value.current(), value.i01_subset().is_some())
            }
            CssKnownPropertyValueRef::BackdropFilter(value) => {
                (value.current(), value.i01_subset().is_some())
            }
            _ => panic!("filter property"),
        };
        let CssFilterValue::Functions(functions) = current else {
            panic!("filter functions");
        };
        let [CssFilterFunctionValue::Blur(blur)] = functions.functions() else {
            panic!("one blur function");
        };
        assert_eq!(blur.length(), &CssLength::try_px(0.0).unwrap());
        assert!(!has_legacy);
        let span = declaration.parsed_value().unwrap().span();
        assert_eq!(
            &source[span.start().byte_offset().value()..span.end().byte_offset().value()],
            source.split_once(':').unwrap().1
        );
    }
}

#[test]
fn omitted_blur_radius_survives_implicit_function_closure() {
    let report = parse_style_attribute("filter:blur(");
    assert_eq!(report.syntax().len(), 1, "{report:?}");
    assert!(!report.is_clean());
    assert!(
        report.diagnostics().iter().all(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        })
    );
}

#[test]
fn explicit_blur_radii_keep_their_values_and_legacy_spelling() {
    for (argument, expected) in [
        ("0", CssLength::Zero),
        ("0px", CssLength::try_px(0.0).unwrap()),
        ("2px", CssLength::try_px(2.0).unwrap()),
    ] {
        let report = parse_style_attribute(&format!("filter:blur({argument})"));
        assert!(report.is_clean(), "{report:?}");
        let CssKnownPropertyValueRef::Filter(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("filter")
        };
        let CssFilterValue::Functions(functions) = value.current() else {
            panic!("functions")
        };
        let [CssFilterFunctionValue::Blur(blur)] = functions.functions() else {
            panic!("blur")
        };
        assert_eq!(blur.length(), &expected);
        let Some(CssFilter::Functions(legacy)) = value.i01_subset() else {
            panic!("legacy")
        };
        let [CssFilterFunction::Blur(arguments)] = legacy.functions() else {
            panic!("legacy blur")
        };
        assert_eq!(arguments.as_css(), argument);
    }
    for argument in ["-1px", "1%", ",", "1px 2px"] {
        let report = parse_style_attribute(&format!("filter:blur({argument})"));
        assert!(report.syntax().is_empty(), "{argument}: {report:?}");
        assert!(!report.is_clean());
    }
}
