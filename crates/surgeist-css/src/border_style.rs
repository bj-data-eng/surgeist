//! Authored border styles before writing-mode or painting resolution.

use crate::common_values::{four_sides_entry, side_element, side_pair_entry};
use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssLineStyle, CssSerializationError};

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
impl crate::specified_rule_serialization::SpecifiedProvider for CssLineStyle {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(&mut writer.context, &mut writer.css)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssLineStyle);

side_element!(CssLineStyle, same = |a, b, _context| Ok(a == b));
four_sides_entry!(CssLineStyle);
side_pair_entry!(CssLineStyle);
