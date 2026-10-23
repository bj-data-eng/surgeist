//! Bounded authored counter-change serialization with explicit property context.

use crate::{
    CssCounterChangesValue, CssCounterProperty, CssSerializationError, CssSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl CssCounterChangesValue {
    /// Serializes this value as canonical specified CSS for `property`.
    ///
    /// Unlike values with a context-free `serialize_specified`, counter changes
    /// need their longhand: omitted operands remain absent in this authored
    /// model, and serialization inserts `0` for reset/set or `1` for increment,
    /// without executing any counter operations or resolving symbolic integer
    /// calculations. `limits` bounds input nodes, projection nodes and output
    /// UTF-8 bytes cumulatively.
    ///
    /// A list charges one aggregate, each entry one aggregate, then its name and
    /// explicit numeric subtree. A synthesized integer charges one projection
    /// node and its output bytes, but no authored input node. Thus one omitted
    /// entry charges 3 input and 4 projection nodes. With `calc(2.5)`, it charges
    /// 5 input and 4 projection nodes: the existing numeric projector traverses
    /// authored function plus leaf, then projects the calc wrapper to its scalar.
    ///
    /// # Errors
    ///
    /// Returns [`CssSerializationError`] when a limit is exhausted. No partial
    /// text is returned and the value is unchanged.
    pub fn serialize_specified(
        &self,
        property: CssCounterProperty,
        limits: CssSerializationLimits,
    ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(property, &mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        property: CssCounterProperty,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let Some(changes) = self.changes() else {
            writer.node()?;
            writer.append("none")?;
            return Ok(());
        };

        writer.node()?; // Nonempty ordered list.
        for (index, change) in changes.iter().enumerate() {
            if index != 0 {
                writer.append(" ")?;
            }
            writer.node()?; // One coupled name/integer entry.
            writer.node()?; // Checked decoded name.
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
        Ok(())
    }
}
