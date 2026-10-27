//! Named-instance and metric descriptor payloads without font matching or execution.

use crate::component::values::CssComponentValueErrorKind;
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueRef, CssValueOrigin,
    CssValueTokenRef,
};
use crate::numeric::{CssNonNegative, CssNumeric, CssPercentage};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;

type Result<T> = std::result::Result<T, CssSerializationError>;

/// A decoded `font-named-instance` string, retaining its component and provenance.
/// Empty and unrecognized names are valid authored strings; matching belongs downstream.
///
/// # Specified serialization
///
/// Emits CSSOM double-quoted string text without matching a font instance.
#[derive(Clone, Debug)]
pub struct CssFontNamedInstanceString {
    component: Box<CssComponentValue>,
}

impl CssFontNamedInstanceString {
    /// Constructs decoded string text. NUL is rejected by the common component owner.
    pub fn try_new(
        decoded: impl Into<String>,
    ) -> std::result::Result<Self, CssComponentValueError> {
        Self::try_from_component(CssComponentValue::try_string(decoded)?)
    }

    /// Retains exactly one checked string component, including its original source.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> std::result::Result<Self, CssComponentValueError> {
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
            unreachable!("checked named-instance string component")
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

    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.append_string(self.as_str())
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssFontNamedInstanceString
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
    CssFontNamedInstanceString
);

impl PartialEq for CssFontNamedInstanceString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}
impl Eq for CssFontNamedInstanceString {}

/// The authored `font-named-instance` descriptor: `auto` or a literal string.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontNamedInstance {
    Auto,
    String(CssFontNamedInstanceString),
}

impl CssFontNamedInstance {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Auto => {
                writer.node()?;
                writer.append("auto")?;
            }
            Self::String(value) => value.append_specified(writer)?,
        }
        Ok(())
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssFontNamedInstance
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssFontNamedInstance);

/// Shared authored payload for ascent, descent and line-gap overrides.
/// Percentage calculations remain symbolic; contextual metric clamping belongs downstream.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontMetricOverride {
    Normal,
    Percentage(CssNumeric<CssPercentage, CssNonNegative>),
}

impl CssFontMetricOverride {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Normal => {
                writer.node()?;
                writer.append("normal")?;
            }
            Self::Percentage(value) => {
                let captured = value.capture_specified(&mut writer.context)?;
                writer.append(&captured)?;
            }
        }
        Ok(())
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssFontMetricOverride
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssFontMetricOverride);
