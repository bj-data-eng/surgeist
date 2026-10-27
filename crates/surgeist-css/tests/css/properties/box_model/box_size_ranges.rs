//! Sizing 3 section 3.2 requires nonnegative literal box sizes; flex-basis
//! references the width grammar. Calculations remain symbolic for later resolution.
//! https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/#sizing-values
//! https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/#flex-basis-property
use crate::css::support::property_admission::{
    assert_checked_rejected, assert_invalid_value, assert_style_attribute_rejected,
};
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::{
    CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef, CssPropertyTarget,
};

const BOX_PROPERTIES: &[CssKnownProperty] = &[
    CssKnownProperty::Width,
    CssKnownProperty::Height,
    CssKnownProperty::MinWidth,
    CssKnownProperty::MinHeight,
    CssKnownProperty::MaxWidth,
    CssKnownProperty::MaxHeight,
    CssKnownProperty::FlexBasis,
];

#[test]
fn negative_literal_box_sizes_reject_in_parsing_and_checked_construction() {
    for &property in BOX_PROPERTIES {
        for value in ["-50%", "-1px", "-0.5em"] {
            let source = format!("{}:{value};color:red", property.canonical_name());
            let report = CssParser::default().parse_block_contents(&source);
            assert_eq!(report.syntax().len(), 1, "{source}: {report:?}");
            assert_eq!(
                report.syntax()[0].known().unwrap().property(),
                CssKnownProperty::Color
            );
            let [diagnostic] = report.diagnostics() else {
                panic!("one invalid size: {source}")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert_eq!(
                diagnostic.error().code(),
                CssErrorCode::InvalidPropertyValue
            );
            assert_invalid_value(diagnostic.error(), property, Some(value));
            assert_checked_rejected(
                CssDeclaration::try_from_components(
                    &CssParser::default(),
                    CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
                    CssComponentValues::parse(&CssParser::default(), value).unwrap(),
                    CssImportance::Normal,
                ),
                property,
                Some(value),
            );
        }
    }
    for (source, token) in [("flex:1 1 -1px", "-1px"), ("flex:1 1 -50%", "-50%")] {
        let report = assert_style_attribute_rejected(source, CssKnownProperty::Flex, Some(token));
        assert!(report.syntax().is_empty(), "{source}: {report:?}");
    }
}

#[test]
fn nonnegative_box_sizes_keep_zero_and_positive_boundaries() {
    for &property in BOX_PROPERTIES {
        for value in ["0", "-0px", "0%", "1px", "50%"] {
            let report = CssParser::default()
                .parse_block_contents(format!("{}:{value}", property.canonical_name()));
            assert!(
                report.is_clean(),
                "{}:{value}: {report:?}",
                property.canonical_name()
            );
            assert_eq!(report.syntax().len(), 1);
        }
    }
}

#[test]
fn signed_offsets_and_symbolic_box_calculations_remain_valid() {
    let report = CssParser::default()
        .parse_block_contents("margin-left:-1px;left:-50%;text-indent:-2em;width:calc(-1px)");
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.syntax().len(), 4);
    let CssKnownPropertyValueRef::Width(width) = report.syntax()[3]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("width")
    };
    assert_eq!(
        width
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "calc(-1px)"
    );
}
