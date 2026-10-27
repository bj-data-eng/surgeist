//! Authored font controls independent of font matching or rendering.

use crate::component::values::CssComponentValueErrorKind;
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueRef, CssValueOrigin,
    CssValueTokenRef,
};
use crate::numeric::{CssNonNegative, CssNumber, CssNumeric};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;

type SerializationResult<T> = Result<T, CssSerializationError>;

/// The authored `font-kerning` choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontKerning {
    /// `auto`: whether kerning is applied is left to the user agent.
    Auto,
    /// `normal`: kerning is applied.
    Normal,
    /// `none`: kerning is not applied.
    None,
}

/// The authored `font-size-adjust` choice.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssFontSizeAdjust {
    /// `none`: the font's x-height is not preserved.
    None,
    /// A non-negative `<number>` aspect value used to adjust the used font size; may be a deferred calculation.
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
    /// Creates a font-language string from decoded text.
    ///
    /// # Errors
    ///
    /// Returns an `InvalidString` error when `decoded` contains U+0000 or does not tokenize back to the same text.
    pub fn try_new(decoded: impl Into<String>) -> Result<Self, CssComponentValueError> {
        Self::try_from_component(CssComponentValue::try_string(decoded)?)
    }

    /// Wraps a component value that is a string token.
    ///
    /// # Errors
    ///
    /// Returns an `InvalidString` error carrying the component's origin when the component is not a string token.
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

    /// Returns the decoded string text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        let CssComponentValueRef::Token(CssValueTokenRef::String(value)) = self.component.view()
        else {
            unreachable!("checked string component")
        };
        value
    }

    /// Returns the underlying string component value.
    #[must_use]
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }

    /// Returns the origin of the underlying string component.
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
    /// `normal`: for OpenType fonts, the content language of the element is used to infer the OpenType language system.
    Normal,
    /// An authored `<string>`: a single case-sensitive four-character OpenType language system tag, used instead of the language system implied by the element's language.
    String(CssFontLanguageString),
}

/// The authored `font-optical-sizing` choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontOpticalSizing {
    /// `auto`: the user agent may modify the shape of glyphs based on the font size and the pixel density of the screen.
    Auto,
    /// `none`: the user agent must not modify the shape of glyphs for optical size.
    None,
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssFontKerning {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::Normal => "normal",
            Self::None => "none",
        })?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssFontKerning);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssFontSizeAdjust {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
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
crate::serialization::specified_rule_serialization::specified_serialization!(CssFontSizeAdjust);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssFontLanguageString
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.node()?;
        writer.append_string(self.as_str())?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssFontLanguageString);

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
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssFontLanguageOverride
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(
    CssFontLanguageOverride
);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssFontOpticalSizing
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::None => "none",
        })?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssFontOpticalSizing);
