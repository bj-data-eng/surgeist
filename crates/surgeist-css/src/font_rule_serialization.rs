//! One cumulative Fonts specified-value/rule projection; authored occurrences remain intact.
use crate::{
    CssAuthoredFontFaceDescriptorValue, CssComponentValue, CssComponentValueRef, CssFontDisplay,
    CssFontFaceDescriptorKind as Kind, CssFontFaceDescriptorValue, CssFontFaceRule,
    CssFontFeatureDisplayValue, CssFontFeatureDisplayValueRef, CssFontFeatureValue,
    CssFontFeatureValueDefinition, CssFontFeatureValueRef, CssFontFeatureValuesItem,
    CssFontFeatureValuesRule, CssSpecifiedValueSerializationError as Error,
    CssSpecifiedValueSerializationErrorKind as ErrorKind,
    CssSpecifiedValueSerializationLimits as Limits, CssValueTokenRef,
    specified_rule_serialization::SpecifiedRuleWriter,
};
type Result<T> = std::result::Result<T, Error>;

/// Equal retained decimal magnitude with the same numeric domain/unit. No rounded text comparison.
pub(crate) fn equal_literals(
    a: Option<&CssComponentValue>,
    b: Option<&CssComponentValue>,
) -> Option<bool> {
    let (a, b) = (a?, b?);
    let (a, b) = match (a.view(), b.view()) {
        (
            CssComponentValueRef::Token(CssValueTokenRef::Number(a)),
            CssComponentValueRef::Token(CssValueTokenRef::Number(b)),
        ) => (a, b),
        (
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(a)),
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(b)),
        ) => (a, b),
        (
            CssComponentValueRef::Token(CssValueTokenRef::Dimension {
                number: a,
                unit: au,
            }),
            CssComponentValueRef::Token(CssValueTokenRef::Dimension {
                number: b,
                unit: bu,
            }),
        ) if au.eq_ignore_ascii_case(bu) => (a, b),
        _ => return Some(false),
    };
    Some(
        crate::exact_decimal::LexicalDecimal::new(a.representation()).value_eq(
            &crate::exact_decimal::LexicalDecimal::new(b.representation()),
        ),
    )
}

fn node(writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)
}
fn display(writer: &mut SpecifiedRuleWriter, value: CssFontDisplay) -> Result<()> {
    node(writer)?;
    writer.append(match value {
        CssFontDisplay::Auto => "auto",
        CssFontDisplay::Block => "block",
        CssFontDisplay::Swap => "swap",
        CssFontDisplay::Fallback => "fallback",
        CssFontDisplay::Optional => "optional",
    })
}
impl CssFontFaceDescriptorValue {
    /// Canonical specified descriptor value, without a synthetic rule or resource loading.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(Limits::default())
    }
    /// One cumulative input/projection/output budget; failures return no partial CSS.
    pub fn serialize_specified_with_limits(&self, limits: Limits) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::FontFamily(value) => value.append_specified(writer),
            Self::Src(value) => value.append_specified(writer),
            Self::FontWeight(value) => value.append_specified(&mut writer.context, &mut writer.css),
            Self::FontStyle(value) => value.append_specified(&mut writer.context, &mut writer.css),
            Self::FontWidth(value) => value.append_specified(&mut writer.context, &mut writer.css),
            Self::FontDisplay(value) => display(writer, *value),
            Self::UnicodeRange(value) => {
                node(writer)?;
                for (index, range) in value.ranges().iter().enumerate() {
                    node(writer)?;
                    if index != 0 {
                        writer.append(", ")?;
                    }
                    writer.append(&format!("U+{:X}", range.start()))?;
                    if range.end() != range.start() {
                        writer.append(&format!("-{:X}", range.end()))?;
                    }
                }
                Ok(())
            }
            Self::FontFeatureSettings(value) => value.append_specified(writer),
            Self::FontVariationSettings(value) => value.append_specified(writer),
            Self::FontNamedInstance(value) => value.append_specified(writer),
            Self::FontLanguageOverride(value) => value.append_specified(writer),
            Self::AscentOverride(value)
            | Self::DescentOverride(value)
            | Self::LineGapOverride(value) => value.append_specified(writer),
        }
    }
}
fn authored(
    writer: &mut SpecifiedRuleWriter,
    value: &CssAuthoredFontFaceDescriptorValue,
) -> Result<()> {
    match value {
        CssAuthoredFontFaceDescriptorValue::Ordinary(value) => value.append_specified(writer),
        CssAuthoredFontFaceDescriptorValue::Pending(value) => {
            crate::pending_serialization::append_pending_specified(
                value.components(),
                &mut writer.context,
                &mut writer.css,
            )
        }
    }
}

impl SpecifiedRuleWriter {
    pub(crate) fn font_face(&mut self, rule: &CssFontFaceRule) -> Result<()> {
        node(self)?;
        // CSSOM's older listed kinds retain relative order. Additional Fonts4
        // descriptors append in kind declaration order (selected project policy).
        const ORDER: [Kind; 14] = [
            Kind::FontFamily,
            Kind::Src,
            Kind::UnicodeRange,
            Kind::FontFeatureSettings,
            Kind::FontWidth,
            Kind::FontWeight,
            Kind::FontStyle,
            Kind::FontDisplay,
            Kind::FontVariationSettings,
            Kind::FontNamedInstance,
            Kind::FontLanguageOverride,
            Kind::AscentOverride,
            Kind::DescentOverride,
            Kind::LineGapOverride,
        ];
        let effective = ORDER.map(|kind| rule.descriptors().effective(kind));
        for record in rule.descriptors().occurrences() {
            node(self)?;
            if !effective
                .iter()
                .flatten()
                .any(|candidate| std::ptr::eq(record, *candidate))
            {
                self.without_output(|writer| authored(writer, record.value()))?;
            }
        }
        self.append("@font-face {")?;
        for (kind, record) in ORDER.into_iter().zip(effective) {
            if let Some(record) = record {
                self.append(" ")?;
                self.append(kind.css_name())?;
                self.append(": ")?;
                authored(self, record.value())?;
                self.append(";")?;
            }
        }
        self.append(" }")
    }

    pub(crate) fn font_features(&mut self, rule: &CssFontFeatureValuesRule) -> Result<()> {
        node(self)?;
        self.append("@font-feature-values ")?;
        for (index, family) in rule.families().iter().enumerate() {
            if index != 0 {
                self.append(", ")?;
            }
            family.append_specified(self)?;
        }
        self.append(" {")?;
        let last_display = rule
            .items()
            .iter()
            .rposition(|item| matches!(item, CssFontFeatureValuesItem::FontDisplay(_)));
        // Bounded seven kinds; names are tracked by last occurrence for selected
        // source-example order, while block kinds use first-occurrence order.
        let mut kinds = Vec::new();
        for (index, item) in rule.items().iter().enumerate() {
            node(self)?;
            match item {
                CssFontFeatureValuesItem::FontDisplay(value) => {
                    if Some(index) != last_display {
                        self.without_output(|writer| value.value().append_to_rule_writer(writer))?;
                    }
                }
                CssFontFeatureValuesItem::Block(block) => {
                    if !kinds.contains(&block.kind()) {
                        kinds
                            .try_reserve(1)
                            .map_err(|_| Error::new(ErrorKind::CapacityOverflow))?;
                        kinds.push(block.kind());
                    }
                }
            }
        }
        if let Some(index) = last_display {
            let CssFontFeatureValuesItem::FontDisplay(value) = &rule.items()[index] else {
                unreachable!()
            };
            self.append(" font-display: ")?;
            value.value().append_to_rule_writer(self)?;
            self.append(";")?;
        }
        for kind in kinds {
            let mut definitions = Vec::new();
            let mut last = std::collections::HashMap::new();
            for item in rule.items() {
                if let CssFontFeatureValuesItem::Block(block) = item
                    && block.kind() == kind
                {
                    for definition in block.definitions() {
                        node(self)?;
                        node(self)?;
                        definitions
                            .try_reserve(1)
                            .map_err(|_| Error::new(ErrorKind::CapacityOverflow))?;
                        last.try_reserve(1)
                            .map_err(|_| Error::new(ErrorKind::CapacityOverflow))?;
                        last.insert(definition.name().as_str(), definitions.len());
                        definitions.push(definition);
                    }
                }
            }
            self.append(" @")?;
            self.append(kind.css_name())?;
            self.append(" {")?;
            for (index, definition) in definitions.iter().enumerate() {
                if last[definition.name().as_str()] == index {
                    self.append(" ")?;
                    self.font_feature_definition(definition)?;
                } else {
                    self.without_output(|writer| writer.font_feature_definition(definition))?;
                }
            }
            self.append(" }")?;
        }
        self.append(" }")
    }
    fn font_feature_definition(
        &mut self,
        definition: &CssFontFeatureValueDefinition,
    ) -> Result<()> {
        self.append_identifier(definition.name().as_str())?;
        self.append(": ")?;
        definition.value().append_to_rule_writer(self)?;
        self.append(";")
    }
}

/// Tests one retained number/percentage coefficient against an exact keyword magnitude.
pub(crate) fn literal_equals(component: Option<&CssComponentValue>, magnitude: &str) -> bool {
    let Some(component) = component else {
        return false;
    };
    let number = match component.view() {
        CssComponentValueRef::Token(
            CssValueTokenRef::Number(number) | CssValueTokenRef::Percentage(number),
        ) => number,
        _ => return false,
    };
    crate::exact_decimal::LexicalDecimal::new(number.representation())
        .value_eq(&crate::exact_decimal::LexicalDecimal::new(magnitude))
}

impl CssFontFeatureDisplayValue {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self.view() {
            CssFontFeatureDisplayValueRef::Ordinary(value) => display(writer, value),
            CssFontFeatureDisplayValueRef::Pending(_) => {
                crate::pending_serialization::append_pending_specified(
                    self.components(),
                    &mut writer.context,
                    &mut writer.css,
                )
            }
        }
    }
    /// Serializes the value without manufacturing a declaration occurrence.
    pub fn to_specified_css(&self) -> Result<String> {
        self.to_specified_css_with_limits(Limits::default())
    }
    pub fn to_specified_css_with_limits(&self, limits: Limits) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }
}
impl CssFontFeatureValue {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self.view() {
            CssFontFeatureValueRef::Indexes(values) => {
                writer.context.charge_input(values.len())?;
                writer.context.charge_projection(values.len())?;
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        writer.append(" ")?;
                    }
                    writer.append(value.as_decimal_str())?;
                }
                Ok(())
            }
            CssFontFeatureValueRef::Pending(_) => {
                crate::pending_serialization::append_pending_specified(
                    self.components(),
                    &mut writer.context,
                    &mut writer.css,
                )
            }
        }
    }
    /// Serializes completed exact indexes or the pending whole component stream.
    pub fn to_specified_css(&self) -> Result<String> {
        self.to_specified_css_with_limits(Limits::default())
    }
    pub fn to_specified_css_with_limits(&self, limits: Limits) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }
}
