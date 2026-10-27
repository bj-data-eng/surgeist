//! Bounded specified keyword rendering for fragmentation controls.

use crate::values::box_model::{CssBreakBetween, CssBreakInside};

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssBreakBetween {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::Avoid => "avoid",
            Self::AvoidPage => "avoid-page",
            Self::Page => "page",
            Self::Left => "left",
            Self::Right => "right",
            Self::Recto => "recto",
            Self::Verso => "verso",
            Self::AvoidColumn => "avoid-column",
            Self::Column => "column",
            Self::AvoidRegion => "avoid-region",
            Self::Region => "region",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssBreakBetween);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssBreakInside {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::Avoid => "avoid",
            Self::AvoidPage => "avoid-page",
            Self::AvoidColumn => "avoid-column",
            Self::AvoidRegion => "avoid-region",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssBreakInside);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for crate::values::box_model::CssBoxDecorationBreak
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Slice => "slice",
            Self::Clone => "clone",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(
    crate::values::box_model::CssBoxDecorationBreak
);
