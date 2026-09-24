//! Canonical specified palette values and fail-closed rule composition.

use std::fmt;

use crate::{
    CssFontFamilyName, CssFontPaletteBase, CssFontPaletteDescriptorValue,
    CssFontPaletteDescriptorValueRef, CssFontPaletteValuesRule, CssRule, CssSheet,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, specified_serialization::SpecifiedSerializationContext,
};

/// Why generic rule or sheet serialization could not return complete CSS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSpecifiedRuleSerializationErrorKind {
    UnsupportedRule,
    UnsupportedEncoding,
    Resource(CssSpecifiedValueSerializationErrorKind),
}

/// An unsupported rule is never omitted or silently serialized as raw syntax.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedRuleSerializationError {
    kind: CssSpecifiedRuleSerializationErrorKind,
    rule_index: Option<usize>,
    source: Option<CssSpecifiedValueSerializationError>,
}

impl CssSpecifiedRuleSerializationError {
    #[must_use]
    pub const fn kind(&self) -> CssSpecifiedRuleSerializationErrorKind {
        self.kind
    }

    /// Zero-based stylesheet rule index when a rule caused the failure.
    #[must_use]
    pub const fn rule_index(&self) -> Option<usize> {
        self.rule_index
    }

    fn unsupported_rule(rule_index: Option<usize>) -> Self {
        Self {
            kind: CssSpecifiedRuleSerializationErrorKind::UnsupportedRule,
            rule_index,
            source: None,
        }
    }

    fn unsupported_encoding() -> Self {
        Self {
            kind: CssSpecifiedRuleSerializationErrorKind::UnsupportedEncoding,
            rule_index: None,
            source: None,
        }
    }

    fn resource(error: CssSpecifiedValueSerializationError, rule_index: Option<usize>) -> Self {
        Self {
            kind: CssSpecifiedRuleSerializationErrorKind::Resource(error.kind()),
            rule_index,
            source: Some(error),
        }
    }
}

impl fmt::Display for CssSpecifiedRuleSerializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            CssSpecifiedRuleSerializationErrorKind::UnsupportedRule => {
                formatter.write_str("canonical serialization is not available for this CSS rule")
            }
            CssSpecifiedRuleSerializationErrorKind::UnsupportedEncoding => formatter
                .write_str("canonical serialization is not available for encoding metadata"),
            CssSpecifiedRuleSerializationErrorKind::Resource(_) => {
                formatter.write_str("canonical CSS rule serialization exceeded a resource limit")
            }
        }
    }
}

impl std::error::Error for CssSpecifiedRuleSerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}

struct SpecifiedRuleWriter {
    context: SpecifiedSerializationContext,
    css: String,
}

impl SpecifiedRuleWriter {
    fn new(limits: CssSpecifiedValueSerializationLimits) -> Self {
        Self {
            context: SpecifiedSerializationContext::new(limits),
            css: String::new(),
        }
    }

    fn append(&mut self, text: &str) -> Result<(), CssSpecifiedValueSerializationError> {
        self.context.append(&mut self.css, text)
    }

    fn palette(
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
                    self.context.charge_input(1)?;
                    self.context.charge_projection(1)?;
                    self.append_family(family.as_str())?;
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

    fn append_identifier(
        &mut self,
        value: &str,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        self.append_escaped(value, EscapedKind::Identifier)
    }

    fn append_string(&mut self, value: &str) -> Result<(), CssSpecifiedValueSerializationError> {
        self.append_escaped(value, EscapedKind::String)
    }

    fn append_escaped(
        &mut self,
        value: &str,
        kind: EscapedKind,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        let mut scratch = String::new();
        let mut bounded = BoundedEscaped {
            context: &self.context,
            css: &mut scratch,
            error: None,
        };
        let result = match kind {
            EscapedKind::Identifier => cssparser::serialize_identifier(value, &mut bounded),
            EscapedKind::String => cssparser::serialize_string(value, &mut bounded),
        };
        if result.is_err() {
            return Err(bounded.error.unwrap_or_else(|| {
                CssSpecifiedValueSerializationError::new(
                    CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            }));
        }
        self.append(&scratch)
    }

    fn append_family(&mut self, name: &str) -> Result<(), CssSpecifiedValueSerializationError> {
        if can_serialize_family_unquoted(name) {
            for (index, word) in name.split(' ').enumerate() {
                if index != 0 {
                    self.append(" ")?;
                }
                self.append_identifier(word)?;
            }
            Ok(())
        } else {
            self.append_string(name)
        }
    }
}

enum EscapedKind {
    Identifier,
    String,
}

struct BoundedEscaped<'a> {
    context: &'a SpecifiedSerializationContext,
    css: &'a mut String,
    error: Option<CssSpecifiedValueSerializationError>,
}

impl fmt::Write for BoundedEscaped<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.context
            .append_temporary(self.css, text)
            .map_err(|error| {
                self.error = Some(error);
                fmt::Error
            })
    }
}

fn can_serialize_family_unquoted(name: &str) -> bool {
    if name.is_empty() || name.starts_with(' ') || name.ends_with(' ') || name.contains("  ") {
        return false;
    }
    if CssFontFamilyName::try_ident_sequence(name.split(' ').map(str::to_owned).collect()).is_none()
    {
        return false;
    }
    name.split(' ').all(|word| {
        let mut chars = word.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !(first.is_ascii_alphabetic() || first == '_' || first == '-' || first as u32 >= 0x80) {
            return false;
        }
        if first == '-' && word.len() == 1 {
            return false;
        }
        if first == '-' && chars.clone().next().is_some_and(|c| c.is_ascii_digit()) {
            return false;
        }
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c as u32 >= 0x80)
    })
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

impl CssRule {
    /// Serializes only rule kinds with a complete canonical specified writer.
    pub fn to_specified_css(&self) -> Result<String, CssSpecifiedRuleSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedRuleSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        append_rule(&mut writer, self, None)?;
        Ok(writer.css)
    }
}

impl CssSheet {
    /// Serializes a complete supported stylesheet atomically in source order.
    /// Unsupported rules or legacy encoding metadata fail closed.
    pub fn to_specified_css(&self) -> Result<String, CssSpecifiedRuleSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedRuleSerializationError> {
        if self.encoding().is_some() {
            return Err(CssSpecifiedRuleSerializationError::unsupported_encoding());
        }
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer
            .context
            .charge_input(1)
            .map_err(|error| CssSpecifiedRuleSerializationError::resource(error, None))?;
        writer
            .context
            .charge_projection(1)
            .map_err(|error| CssSpecifiedRuleSerializationError::resource(error, None))?;
        for (index, rule) in self.rules().iter().enumerate() {
            if index != 0 {
                writer.append("\n").map_err(|error| {
                    CssSpecifiedRuleSerializationError::resource(error, Some(index))
                })?;
            }
            append_rule(&mut writer, rule, Some(index))?;
        }
        Ok(writer.css)
    }
}

fn append_rule(
    writer: &mut SpecifiedRuleWriter,
    rule: &CssRule,
    index: Option<usize>,
) -> Result<(), CssSpecifiedRuleSerializationError> {
    match rule {
        CssRule::FontPaletteValues(rule) => writer
            .palette(rule)
            .map_err(|error| CssSpecifiedRuleSerializationError::resource(error, index)),
        _ => Err(CssSpecifiedRuleSerializationError::unsupported_rule(index)),
    }
}
