//! Authored font controls independent of font matching or rendering.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef,
    CssNonNegative, CssNumber, CssNumeric, CssSerializationError, CssValueOrigin, CssValueTokenRef,
};

type SerializationResult<T> = Result<T, CssSerializationError>;

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
    Number(CssNumeric<CssNumber, CssNonNegative>),
}

impl PartialEq for CssFontSizeAdjust {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Number(left), Self::Number(right)) => left == right,
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

impl crate::specified_rule_serialization::SpecifiedProvider for CssFontKerning {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::Normal => "normal",
            Self::None => "none",
        })?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontKerning);

impl crate::specified_rule_serialization::SpecifiedProvider for CssFontSizeAdjust {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        match self {
            Self::None => writer.keyword("none")?,
            Self::Number(value) => {
                let captured = value.capture_specified(&mut writer.context)?;
                writer.append(&captured)?;
            }
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontSizeAdjust);

impl crate::specified_rule_serialization::SpecifiedProvider for CssFontLanguageString {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.node()?;
        writer.append_string(self.as_str())?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontLanguageString);

impl CssFontLanguageOverride {
    pub(crate) fn append_specified(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        match self {
            Self::Normal => writer.keyword("normal")?,
            Self::String(value) => {
                writer.node()?;
                writer.append_string(value.as_str())?;
            }
        }
        Ok(())
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontLanguageOverride {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontLanguageOverride);

impl crate::specified_rule_serialization::SpecifiedProvider for CssFontOpticalSizing {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::None => "none",
        })?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontOpticalSizing);
