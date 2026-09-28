//! Bounded authored counter-change serialization with explicit property context.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssCounterChangesValue, CssCounterProperty, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn charge(writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)
}

impl CssCounterChangesValue {
    /// Emits every counter name and effective integer for the selected longhand.
    ///
    /// Omitted operands remain absent in this authored model. Serialization
    /// inserts `0` for reset/set or `1` for increment, without executing any
    /// counter operations or resolving symbolic integer calculations.
    pub fn serialize_specified(&self, property: CssCounterProperty) -> Result<String> {
        self.serialize_specified_with_limits(
            property,
            CssSpecifiedValueSerializationLimits::default(),
        )
    }

    /// Applies one cumulative input-node, projection-node, and CSS-byte budget.
    ///
    /// A list charges one aggregate, each entry one aggregate, then its name and
    /// explicit numeric subtree. A synthesized integer charges one projection
    /// node and its output bytes, but no authored input node. Thus one omitted
    /// entry charges 3 input and 4 projection nodes. With `calc(2.5)`, it charges
    /// 5 input and 4 projection nodes: the existing numeric projector traverses
    /// authored function plus leaf, then projects the calc wrapper to its scalar.
    pub fn serialize_specified_with_limits(
        &self,
        property: CssCounterProperty,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        let Some(changes) = self.changes() else {
            charge(&mut writer)?;
            writer.append("none")?;
            return Ok(writer.css);
        };

        charge(&mut writer)?; // Nonempty ordered list.
        for (index, change) in changes.iter().enumerate() {
            if index != 0 {
                writer.append(" ")?;
            }
            charge(&mut writer)?; // One coupled name/integer entry.
            charge(&mut writer)?; // Checked decoded name.
            writer.append_identifier(change.name().as_str())?;
            writer.append(" ")?;
            if let Some(number) = change.value() {
                number.append_specified(&mut writer.context, &mut writer.css)?;
            } else {
                writer.context.charge_projection(1)?;
                writer.append(match property {
                    CssCounterProperty::Reset | CssCounterProperty::Set => "0",
                    CssCounterProperty::Increment => "1",
                })?;
            }
        }
        Ok(writer.css)
    }
}
