//! Authored font controls independent of font matching or rendering.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef,
    CssSpecifiedNonNegativeNumber, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, CssValueTokenRef,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// The authored `font-kerning` choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontKerning {
    Auto,
    Normal,
    None,
}

/// The authored `font-size-adjust` choice.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssFontSizeAdjust {
    None,
    Number(CssSpecifiedNonNegativeNumber),
}

impl PartialEq for CssFontSizeAdjust {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Number(left), Self::Number(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}
impl Eq for CssFontSizeAdjust {}

/// A decoded string from the authored `font-language-override` value.
#[derive(Clone, Debug)]
pub struct CssFontLanguageString {
    component: Box<CssComponentValue>,
}

impl CssFontLanguageString {
    pub fn try_new(decoded: impl Into<String>) -> Result<Self, CssComponentValueError> {
        Self::try_from_component(CssComponentValue::try_string(decoded)?)
    }

    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssComponentValueError> {
        if !matches!(
            component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::String(_))
        ) {
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidString,
                component.origin().clone(),
            ));
        }
        Ok(Self {
            component: Box::new(component),
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        let CssComponentValueRef::Token(CssValueTokenRef::String(value)) = self.component.view()
        else {
            unreachable!("checked string component")
        };
        value
    }

    #[must_use]
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }

    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
}

impl PartialEq for CssFontLanguageString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}
impl Eq for CssFontLanguageString {}

/// The authored `font-language-override` value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontLanguageOverride {
    Normal,
    String(CssFontLanguageString),
}

/// The authored `font-optical-sizing` choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontOpticalSizing {
    Auto,
    None,
}

fn keyword(writer: &mut SpecifiedRuleWriter, value: &str) -> SerializationResult<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    writer.append(value)
}

impl CssFontKerning {
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        keyword(
            writer,
            match self {
                Self::Auto => "auto",
                Self::Normal => "normal",
                Self::None => "none",
            },
        )?;
        Ok(())
    }
}

impl CssFontSizeAdjust {
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        match self {
            Self::None => keyword(writer, "none")?,
            Self::Number(value) => {
                let captured = value.capture_specified(&mut writer.context)?;
                writer.append(&captured)?;
            }
        }
        Ok(())
    }
}

impl CssFontLanguageString {
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        writer.append_string(self.as_str())?;
        Ok(())
    }
}

impl CssFontLanguageOverride {
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
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

    pub(crate) fn append_specified(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        match self {
            Self::Normal => keyword(writer, "normal")?,
            Self::String(value) => {
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)?;
                writer.append_string(value.as_str())?;
            }
        }
        Ok(())
    }
}

impl CssFontOpticalSizing {
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        keyword(
            writer,
            match self {
                Self::Auto => "auto",
                Self::None => "none",
            },
        )?;
        Ok(())
    }
}
