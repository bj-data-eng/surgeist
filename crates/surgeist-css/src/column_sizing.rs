//! Specified Multicol column values and cumulative serialization.

use crate::specified_rule_serialization::SpecifiedProvider as _;
use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssColumnCount, CssColumnFill, CssColumnSpan, CssColumns, CssSerializationError};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl crate::specified_rule_serialization::SpecifiedProvider for CssColumnFill {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        context.charge_nodes(1)?;
        context.append(
            output,
            match self {
                Self::Auto => "auto",
                Self::Balance => "balance",
                Self::BalanceAll => "balance-all",
            },
        )?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssColumnFill);

impl crate::specified_rule_serialization::SpecifiedProvider for CssColumnSpan {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        context.charge_nodes(1)?;
        context.append(
            output,
            match self {
                Self::None => "none",
                Self::All => "all",
            },
        )?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssColumnSpan);

impl CssColumnCount {
    fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        match self {
            Self::Auto => {
                context.charge_nodes(1)?;
                context.append(output, "auto")
            }
            Self::Count(value) => value.append_specified(context, output),
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssColumnCount {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssColumnCount);

impl CssColumns {
    pub(crate) fn append_cssom_inverse_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<()> {
        writer.node()?;
        let emit_width = !matches!(self.width(), crate::CssSizeValue::Auto)
            || matches!(self.count(), CssColumnCount::Auto);
        writer.source_member(0, |writer| {
            if emit_width {
                self.width()
                    .serialize_specified_into(&mut writer.context, &mut writer.css)
            } else {
                writer.without_output(|writer| {
                    self.width()
                        .serialize_specified_into(&mut writer.context, &mut writer.css)
                })
            }
        })?;
        writer.source_member(1, |writer| {
            if matches!(self.count(), CssColumnCount::Auto) {
                writer.without_output(|writer| self.count().append_to_rule_writer(writer))
            } else {
                if emit_width {
                    writer.append(" ")?;
                }
                self.count().append_to_rule_writer(writer)
            }
        })
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssColumns {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        context.charge_nodes(1)?;
        self.width().serialize_specified_into(context, output)?;
        context.append(output, " ")?;
        self.count().serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssColumns);
