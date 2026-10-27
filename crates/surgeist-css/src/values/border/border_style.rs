//! Authored border styles before writing-mode or painting resolution.

use crate::serialization::CssSerializationError;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;
use crate::values::border::CssLineStyle;
use crate::values::common::common_values::{four_sides_entry, side_element, side_pair_entry};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl CssLineStyle {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_nodes(1)?;
        context.append(
            output,
            match self {
                Self::None => "none",
                Self::Hidden => "hidden",
                Self::Dotted => "dotted",
                Self::Dashed => "dashed",
                Self::Solid => "solid",
                Self::Double => "double",
                Self::Groove => "groove",
                Self::Ridge => "ridge",
                Self::Inset => "inset",
                Self::Outset => "outset",
            },
        )
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssLineStyle {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(&mut writer.context, &mut writer.css)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssLineStyle);

side_element!(CssLineStyle, same = |a, b, _context| Ok(a == b));
four_sides_entry!(CssLineStyle);
side_pair_entry!(CssLineStyle);
