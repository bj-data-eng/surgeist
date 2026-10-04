//! Canonical specified palette values.

use crate::syntax::color::serialization::{
    is_default_mix, mix_weight_texts, serialize_interpolation,
};
use crate::{
    CssFontPalette, CssFontPaletteBase, CssFontPaletteDescriptorValue,
    CssFontPaletteDescriptorValueRef, CssFontPaletteMix, CssFontPaletteMixComponent,
    CssFontPaletteValuesRule, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, specified_rule_serialization::SpecifiedRuleWriter,
};

enum PaletteWork<'a> {
    Palette(&'a CssFontPalette),
    Text(&'static str),
    Owned(String),
}

impl CssFontPalette {
    /// Canonical authored output; named palettes and calculations stay symbolic.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Emits the entire graph atomically under cumulative node and byte limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        let mut work = Vec::new();
        reserve_palette_work(&mut work, 1)?;
        work.push(PaletteWork::Palette(self));
        while let Some(next) = work.pop() {
            match next {
                PaletteWork::Text(text) => writer.append(text)?,
                PaletteWork::Owned(text) => writer.append(&text)?,
                PaletteWork::Palette(value) => {
                    writer.context.charge_input(1)?;
                    writer.context.charge_projection(1)?;
                    match value {
                        Self::Normal => writer.append("normal")?,
                        Self::Light => writer.append("light")?,
                        Self::Dark => writer.append("dark")?,
                        Self::Named(name) => writer.append_identifier(name.as_str())?,
                        Self::Mix(value) => schedule_palette_mix(value, &mut work, &mut writer)?,
                    }
                }
            }
        }
        Ok(writer.css)
    }
}

fn reserve_palette_work(
    work: &mut Vec<PaletteWork<'_>>,
    additional: usize,
) -> Result<(), CssSpecifiedValueSerializationError> {
    work.try_reserve(additional).map_err(|_| {
        CssSpecifiedValueSerializationError::new(
            crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
        )
    })
}

fn schedule_palette_mix<'a>(
    value: &'a CssFontPaletteMix,
    work: &mut Vec<PaletteWork<'a>>,
    writer: &mut SpecifiedRuleWriter,
) -> Result<(), CssSpecifiedValueSerializationError> {
    let slots = value
        .components()
        .len()
        .checked_mul(4)
        .and_then(|value| value.checked_add(5))
        .ok_or_else(|| {
            CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
    reserve_palette_work(work, slots)?;
    let weights = mix_weight_texts(
        value
            .components()
            .iter()
            .map(CssFontPaletteMixComponent::weight),
        &mut writer.context,
    )?;
    work.push(PaletteWork::Text(")"));
    for (index, (component, weight)) in value.components().iter().zip(weights).enumerate().rev() {
        if let Some(weight) = weight {
            work.push(PaletteWork::Owned(weight));
            work.push(PaletteWork::Text(" "));
        }
        work.push(PaletteWork::Palette(component.palette()));
        if index != 0 {
            work.push(PaletteWork::Text(", "));
        }
    }
    if let Some(interpolation) = value.interpolation() {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        if !is_default_mix(interpolation) {
            work.push(PaletteWork::Text(", "));
            work.push(PaletteWork::Owned(serialize_interpolation(
                interpolation,
                &writer.context,
            )?));
            work.push(PaletteWork::Text("in "));
        }
    }
    work.push(PaletteWork::Text("palette-mix("));
    Ok(())
}

impl SpecifiedRuleWriter {
    pub(crate) fn palette(
        &mut self,
        rule: &CssFontPaletteValuesRule,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
        self.append("@font-palette-values ")?;
        self.append_identifier(rule.name().as_str())?;
        self.append(" {")?;
        for descriptor in rule.descriptors() {
            self.context.charge_input(1)?;
            self.context.charge_projection(1)?;
            self.append(" ")?;
            self.append(descriptor.value().kind().css_name())?;
            self.append(": ")?;
            self.descriptor_value(descriptor.value())?;
            self.append(";")?;
        }
        self.append(" }")
    }

    fn descriptor_value(
        &mut self,
        value: &CssFontPaletteDescriptorValue,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        match value.view() {
            CssFontPaletteDescriptorValueRef::FontFamily(families) => {
                for (index, family) in families.iter().enumerate() {
                    if index != 0 {
                        self.append(", ")?;
                    }
                    family.append_specified(self)?;
                }
                Ok(())
            }
            CssFontPaletteDescriptorValueRef::BasePalette(base) => match base {
                CssFontPaletteBase::Light => self.append_keyword("light"),
                CssFontPaletteBase::Dark => self.append_keyword("dark"),
                CssFontPaletteBase::Index(index) => index
                    .value()
                    .append_specified(&mut self.context, &mut self.css),
            },
            CssFontPaletteDescriptorValueRef::OverrideColors(overrides) => {
                for (index, pair) in overrides.iter().enumerate() {
                    if index != 0 {
                        self.append(", ")?;
                    }
                    self.context.charge_input(1)?;
                    self.context.charge_projection(1)?;
                    pair.index()
                        .value()
                        .append_specified(&mut self.context, &mut self.css)?;
                    self.append(" ")?;
                    pair.color()
                        .append_specified(&mut self.context, &mut self.css)?;
                }
                Ok(())
            }
            CssFontPaletteDescriptorValueRef::Pending(_) => {
                crate::pending_serialization::append_pending_specified(
                    value.components(),
                    &mut self.context,
                    &mut self.css,
                )
            }
        }
    }

    fn append_keyword(&mut self, keyword: &str) -> Result<(), CssSpecifiedValueSerializationError> {
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
        self.append(keyword)
    }
}

impl CssFontPaletteDescriptorValue {
    /// Canonical specified descriptor text; pending values retain their token stream.
    pub fn to_specified_css(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.descriptor_value(self)?;
        Ok(writer.css)
    }
}

impl CssFontPaletteValuesRule {
    /// Canonical specified rule text. Descriptor duplicates retain authored order.
    pub fn to_specified_css(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.palette(self)?;
        Ok(writer.css)
    }
}
