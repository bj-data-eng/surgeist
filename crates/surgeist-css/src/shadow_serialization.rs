//! Bounded authored shadow serialization; no computed defaults or paint values.

use crate::{
    CssBoxShadow, CssBoxShadowList, CssDropShadow, CssShadow, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! specified_shadow_methods {
    () => {
        /// Serializes the authored shadow, retaining omitted components and symbolic values.
        pub fn serialize_specified(&self) -> Result<String> {
            self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
        }

        /// Uses one cumulative input, projection, and byte budget for all children.
        /// Failure returns no partial CSS and leaves the authored value unchanged.
        pub fn serialize_specified_with_limits(
            &self,
            limits: CssSpecifiedValueSerializationLimits,
        ) -> Result<String> {
            let mut writer = SpecifiedRuleWriter::new(limits);
            self.append_specified(&mut writer)?;
            Ok(writer.css)
        }
    };
}

impl CssShadow {
    specified_shadow_methods!();

    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge_aggregate(writer)?;
        if let Some(color) = self.color() {
            color.append_specified(&mut writer.context, &mut writer.css)?;
            writer.append(" ")?;
        }
        self.offset_x()
            .append_specified(&mut writer.context, &mut writer.css)?;
        writer.append(" ")?;
        self.offset_y()
            .append_specified(&mut writer.context, &mut writer.css)?;
        if let Some(blur) = self.blur_radius() {
            writer.append(" ")?;
            blur.append_specified(&mut writer.context, &mut writer.css)?;
        }
        if let Some(spread) = self.spread_radius() {
            writer.append(" ")?;
            spread.append_specified(&mut writer.context, &mut writer.css)?;
        }
        if self.inset() {
            charge_aggregate(writer)?;
            writer.append(" inset")?;
        }
        Ok(())
    }
}

impl CssBoxShadowList {
    specified_shadow_methods!();

    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge_aggregate(writer)?;
        for (index, shadow) in self.shadows().iter().enumerate() {
            if index != 0 {
                writer.append(", ")?;
            }
            shadow.append_specified(writer)?;
        }
        Ok(())
    }
}

impl CssBoxShadow {
    specified_shadow_methods!();

    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::None => {
                charge_aggregate(writer)?;
                writer.append("none")
            }
            Self::Shadows(shadows) => shadows.append_specified(writer),
        }
    }
}

impl CssDropShadow {
    specified_shadow_methods!();

    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge_aggregate(writer)?;
        writer.append("drop-shadow(")?;
        if let Some(color) = self.color() {
            color.append_specified(&mut writer.context, &mut writer.css)?;
            writer.append(" ")?;
        }
        self.offset_x()
            .append_specified(&mut writer.context, &mut writer.css)?;
        writer.append(" ")?;
        self.offset_y()
            .append_specified(&mut writer.context, &mut writer.css)?;
        if let Some(deviation) = self.standard_deviation() {
            writer.append(" ")?;
            deviation.append_specified(&mut writer.context, &mut writer.css)?;
        }
        writer.append(")")
    }
}

fn charge_aggregate(writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)
}
