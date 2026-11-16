//! Bounded authored counter-change serialization with explicit property context.

use crate::serialization::{CssSerializationError, CssSerializationLimits};
use crate::values::content::{CssCounterChangesValue, CssCounterProperty};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl CssCounterChangesValue {
    /// Serializes this value as canonical specified CSS for `property`.
    ///
    /// Unlike values with a context-free `serialize_specified`, counter changes
    /// need their longhand, whose default integer is `0` for reset/set and `1`
    /// for increment (Lists 3 §4.1 and §4.2). An omitted operand stays absent,
    /// and so does a literal integer equal to the default, because CSSOM 1
    /// §6.7.2 leaves out a component whose omission does not change the value.
    /// A calculation is never compared with the default, so it stays as
    /// authored. No counter operation is executed and no symbolic integer
    /// calculation is resolved. `limits` bounds input nodes, projection nodes
    /// and output UTF-8 bytes cumulatively.
    ///
    /// A list charges one aggregate, each entry one aggregate, then its name and
    /// explicit numeric subtree; a literal default is visited and charged like
    /// any other integer without producing bytes. Thus one omitted entry
    /// charges 3 input and 3 projection nodes. With `calc(2.5)`, it charges
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
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(property, &mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        property: CssCounterProperty,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let Some(changes) = self.changes() else {
            writer.node()?;
            writer.append("none")?;
            return Ok(());
        };

        let default = match property {
            CssCounterProperty::Reset | CssCounterProperty::Set => 0,
            CssCounterProperty::Increment => 1,
        };
        writer.node()?; // Nonempty ordered list.
        for (index, change) in changes.iter().enumerate() {
            if index != 0 {
                writer.append(" ")?;
            }
            writer.node()?; // One coupled name/integer entry.
            writer.node()?; // Checked decoded name.
            writer.append_identifier(change.name().as_str())?;
            // Lists 3 §4.1 and §4.2 give an omitted integer its default, so a
            // literal integer equal to it is visited but not written. A
            // calculation, even one that evaluates to the default, stays.
            if let Some(number) = change.value() {
                if number.literal().and_then(|literal| literal.value()) == Some(default) {
                    writer.without_output(|writer| {
                        number.append_specified(&mut writer.context, &mut writer.css)
                    })?;
                } else {
                    writer.append(" ")?;
                    number.append_specified(&mut writer.context, &mut writer.css)?;
                }
            }
        }
        Ok(())
    }
}
