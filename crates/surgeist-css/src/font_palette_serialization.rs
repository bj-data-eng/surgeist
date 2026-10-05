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
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    /// Visits the complete authored graph with the enclosing operation's
    /// cumulative work, byte budget and emission mode.
    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
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
                        Self::Mix(value) => schedule_palette_mix(value, &mut work, writer)?,
                    }
                }
            }
        }
        Ok(())
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

#[cfg(test)]
mod composition_tests {
    use super::*;
    use crate::CssSpecifiedValueSerializationErrorKind as Kind;
    use crate::{CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef};

    fn palette(source: &str) -> CssFontPalette {
        let declaration = crate::parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::FontPalette),
            crate::parse_component_values(source).unwrap(),
            crate::CssImportance::Normal,
        )
        .unwrap_or_else(|error| panic!("{source}: {error:?}"));
        let CssKnownPropertyValueRef::FontPalette(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("checked font palette");
        };
        value.palette().clone()
    }

    #[test]
    fn bridge_composes_with_the_callers_buffer_and_byte_limit() {
        let value = palette("palette-mix(light 20%, dark 80%)");
        let expected = "xpalette-mix(light 20%, dark 80%)y";
        let mut writer = SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(
            100_000,
            100_000,
            expected.len(),
        ));
        writer.append("x").unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        writer.append("y").unwrap();
        assert_eq!(writer.css, expected);
        assert_eq!(writer.context.remaining_bytes(), 0);
        assert_eq!(writer.append("z").unwrap_err().kind(), Kind::ByteLimit);
    }

    #[test]
    fn suppressed_nested_mix_visits_named_and_calculated_children_with_no_bytes() {
        for source in [
            "normal",
            "light",
            "dark",
            "--theme",
            "palette-mix(in --Profile, light 20%, dark 80%)",
            "palette-mix(light calc(10%), dark)",
            "palette-mix(palette-mix(in srgb, light 30%, normal) 20%, dark)",
            "palette-mix(palette-mix(dark), palette-mix(light, --theme), normal)",
        ] {
            let value = palette(source);
            let original = value.clone();
            let mut writer = SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(
                100_000, 100_000, 1,
            ));
            writer.append("x").unwrap();
            writer
                .without_output(|writer| value.append_to_rule_writer(writer))
                .unwrap_or_else(|error| panic!("{source}: {error:?}"));
            assert_eq!(writer.css, "x");
            assert!(!writer.context.output_suppressed());
            assert_eq!(value, original);
        }
    }

    #[test]
    fn suppressed_palette_and_emitted_sibling_share_exact_semantic_budget() {
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(2, 2, 4));
        writer
            .without_output(|writer| CssFontPalette::Normal.append_to_rule_writer(writer))
            .unwrap();
        CssFontPalette::Dark
            .append_to_rule_writer(&mut writer)
            .unwrap();
        assert_eq!(writer.css, "dark");
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
    }

    #[test]
    fn deep_singleton_palette_bridge_keeps_iterative_suppressed_traversal() {
        let mut value = CssFontPalette::Normal;
        for _ in 0..256 {
            value = CssFontPalette::Mix(Box::new(
                CssFontPaletteMix::try_new(
                    None,
                    vec![CssFontPaletteMixComponent::new(value, None)],
                )
                .unwrap(),
            ));
        }
        assert_eq!(
            CssFontPaletteMix::try_new(
                None,
                vec![CssFontPaletteMixComponent::new(value.clone(), None)]
            )
            .unwrap_err(),
            crate::CssFontPaletteMixConstructionError::NestingLimit,
        );
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(257, 100_000, 0));
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
    }

    #[test]
    fn nested_palette_failure_restores_output_and_retains_consumed_work() {
        let value = palette("palette-mix(light, dark)");
        for (input, projection, expected) in [
            (1, 100, Kind::InputNodeLimit),
            (100, 1, Kind::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(
                input, projection, 2,
            ));
            writer.append("x").unwrap();
            let error = writer
                .without_output(|writer| {
                    let error = writer
                        .without_output(|writer| value.append_to_rule_writer(writer))
                        .unwrap_err();
                    assert!(writer.context.output_suppressed());
                    Err::<(), _>(error)
                })
                .unwrap_err();
            assert_eq!(error.kind(), expected);
            assert!(!writer.context.output_suppressed());
            writer.append("y").unwrap();
            assert_eq!(writer.css, "xy");
            let result = CssFontPalette::Normal.append_to_rule_writer(&mut writer);
            assert_eq!(result.unwrap_err().kind(), expected);
        }
    }
}
