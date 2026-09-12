//! Expansion-owned tests: include as an ordinary cfg(test) child module.
//! These exercise the shared transition used for omitted and reset-only values.
//! Fonts 4 leaves font-family initial selection to the UA; CSS Color 4 leaves
//! CanvasText symbolic. The transition must preserve those distinct states.
use super::*;
use crate::{CssInitialValueRef, CssKnownProperty, CssPropertyKindRef, CssUserAgentInitial};

fn initial(property: CssKnownProperty) -> crate::CssLonghandInitialValue {
    let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
        panic!("longhand metadata")
    };
    metadata.initial_value()
}

#[test]
fn omitted_initial_preserves_user_agent_requirement_and_fixed_value() {
    let family = initial(CssKnownProperty::FontFamily);
    assert!(matches!(
        family.view(),
        CssInitialValueRef::UserAgent(CssUserAgentInitial::FontFamily)
    ));
    let family = OwnedContributionValue::from_initial(family);
    assert_eq!(family.property(), CssKnownProperty::FontFamily);
    assert!(matches!(
        family.view(),
        CssContributionValueRef::UserAgentInitial(CssUserAgentInitial::FontFamily)
    ));

    let color = OwnedContributionValue::from_initial(initial(CssKnownProperty::Color));
    assert_eq!(color.property(), CssKnownProperty::Color);
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::Color(color)) = color.view() else {
        panic!("intrinsic symbolic color remains an ordinary specified color")
    };
    assert_eq!(
        color.system(),
        Some(crate::CssAuthoredSystemColor::CanvasText)
    );

    let width = OwnedContributionValue::from_initial(initial(CssKnownProperty::BorderTopWidth));
    assert_eq!(width.property(), CssKnownProperty::BorderTopWidth);
    assert!(matches!(
        width.view(),
        CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderTopWidth(CssLength::Medium))
    ));
}
