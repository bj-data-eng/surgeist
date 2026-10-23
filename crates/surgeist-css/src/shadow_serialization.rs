//! Bounded authored shadow serialization; no computed defaults or paint values.

use crate::{
    CssBoxShadow, CssBoxShadowList, CssDropShadow, CssSerializationError, CssShadow,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSerializationError>;

macro_rules! specified_shadow_methods {
    ($ty:ty) => {
        impl crate::specified_rule_serialization::SpecifiedProvider for $ty {
            fn append_to_rule_writer(
                &self,
                writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
            ) -> std::result::Result<(), crate::CssSerializationError> {
                self.append_specified(writer)?;
                Ok(())
            }
        }
        crate::specified_rule_serialization::specified_serialization!($ty);
    };
}

impl CssShadow {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
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
            writer.node()?;
            writer.append(" inset")?;
        }
        Ok(())
    }
}
specified_shadow_methods!(CssShadow);

impl CssBoxShadowList {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.comma_separated(self.shadows(), |writer, shadow| {
            shadow.append_specified(writer)
        })?;
        Ok(())
    }
}
specified_shadow_methods!(CssBoxShadowList);

impl CssBoxShadow {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::None => {
                writer.node()?;
                writer.append("none")
            }
            Self::Shadows(shadows) => shadows.append_specified(writer),
        }
    }
}
specified_shadow_methods!(CssBoxShadow);

impl CssDropShadow {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
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
specified_shadow_methods!(CssDropShadow);
