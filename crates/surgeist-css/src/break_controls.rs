//! Bounded specified keyword rendering for fragmentation controls.

use crate::{CssBreakBetween, CssBreakInside};

impl crate::specified_rule_serialization::SpecifiedProvider for CssBreakBetween {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
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
crate::specified_rule_serialization::specified_serialization!(CssBreakBetween);

impl crate::specified_rule_serialization::SpecifiedProvider for CssBreakInside {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::Avoid => "avoid",
            Self::AvoidPage => "avoid-page",
            Self::AvoidColumn => "avoid-column",
            Self::AvoidRegion => "avoid-region",
        })
    }
}
crate::specified_rule_serialization::specified_serialization!(CssBreakInside);

impl crate::specified_rule_serialization::SpecifiedProvider for crate::CssBoxDecorationBreak {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.keyword(match self {
            Self::Slice => "slice",
            Self::Clone => "clone",
        })
    }
}
crate::specified_rule_serialization::specified_serialization!(crate::CssBoxDecorationBreak);
