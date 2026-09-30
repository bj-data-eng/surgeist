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
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        context.charge_input(1)?;
        context.charge_projection(1)?;
        context.append(
            &mut output,
            match self {
                Self::Auto => "auto",
                Self::Balance => "balance",
                Self::BalanceAll => "balance-all",
            },
        )?;
        Ok(output)
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
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        context.charge_input(1)?;
        context.charge_projection(1)?;
        context.append(
            &mut output,
            match self {
                Self::None => "none",
                Self::All => "all",
            },
        )?;
        Ok(output)
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
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.serialize_specified_into(&mut context, &mut output)?;
        Ok(output)
    }

    fn serialize_specified_into(
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
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.serialize_specified_into(&mut context, &mut output)?;
        Ok(output)
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
    /// Serializes both effective values in width-then-count order, including omitted `auto`.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Uses one cumulative budget; the pair itself charges one input and projection node.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        context.charge_input(1)?;
        context.charge_projection(1)?;
        self.width()
            .serialize_specified_into(&mut context, &mut output)?;
        context.append(&mut output, " ")?;
        self.count()
            .serialize_specified_into(&mut context, &mut output)?;
        Ok(output)
    }
}
