//! Specified Multicol column values and cumulative serialization.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssColumnCount, CssColumnFill, CssColumnSpan, CssColumns, CssPositiveIntegerValue,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

impl CssColumnFill {
    /// Serializes the authored fill keyword.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Bounds the input, projection, and emitted bytes of the keyword.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        context.charge_input(1)?;
        context.charge_projection(1)?;
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

impl CssColumnSpan {
    /// Serializes the authored span keyword.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Bounds the input, projection, and emitted bytes of the keyword.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        context.charge_input(1)?;
        context.charge_projection(1)?;
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

impl CssPositiveIntegerValue {
    /// Serializes a positive count without resolving deferred integer math.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Applies one input, projection, and byte budget to the whole value.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }

    pub(crate) fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        match self {
            Self::Literal(value) => value.integer().append_specified(context, output),
            Self::Calculation(value) => value.serialize_specified_into(context, output),
        }
    }
}

impl CssColumnCount {
    /// Serializes the specified count or `auto`.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes under a bounded input, projection, and byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }

    fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        match self {
            Self::Auto => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, "auto")
            }
            Self::Count(value) => value.serialize_specified_into(context, output),
        }
    }
}

impl CssColumns {
    pub(crate) fn append_cssom_inverse_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<()> {
        writer.node()?;
        let emit_width = !matches!(self.width(), crate::CssColumnWidth::Auto)
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

    /// Serializes both effective values in width-then-count order, including omitted `auto`.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Uses one cumulative budget; the pair itself charges one input and projection node.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        context.charge_input(1)?;
        context.charge_projection(1)?;
        self.width().serialize_specified_into(context, output)?;
        context.append(output, " ")?;
        self.count().serialize_specified_into(context, output)?;
        Ok(())
    }
}
