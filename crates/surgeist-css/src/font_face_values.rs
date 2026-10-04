//! Named-instance and metric descriptor payloads without font matching or execution.

use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef,
    CssSpecifiedNonNegativePercentage, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, CssValueTokenRef,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

/// A decoded `font-named-instance` string, retaining its component and provenance.
/// Empty and unrecognized names are valid authored strings; matching belongs downstream.
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

    /// Emits CSSOM double-quoted string text without matching a font instance.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges one input/projection node and actual escaped UTF-8 bytes atomically.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_specified(&mut writer)?;
        Ok(writer.css)
    }

    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        writer.append_string(self.as_str())
    }
}

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
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Enum dispatch adds no visit; the keyword or string charges one node of each kind.
    /// Failures return no partial CSS and leave the authored value unchanged.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        match self {
            Self::Auto => {
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)?;
                writer.append("auto")?;
            }
            Self::String(value) => value.append_specified(&mut writer)?,
        }
        Ok(writer.css)
    }
}

/// Shared authored payload for ascent, descent and line-gap overrides.
/// Percentage calculations remain symbolic; contextual metric clamping belongs downstream.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontMetricOverride {
    Normal,
    Percentage(CssSpecifiedNonNegativePercentage),
}

impl CssFontMetricOverride {
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// `normal` charges one input/projection node. Percentages reuse their complete
    /// existing projection costs, including calculation visits, without a dispatch node.
    /// One actual-output-byte budget applies; failure returns no partial CSS.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        match self {
            Self::Normal => {
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)?;
                writer.append("normal")?;
            }
            Self::Percentage(value) => {
                let captured = value.capture_specified(&mut writer.context)?;
                writer.append(&captured)?;
            }
        }
        Ok(writer.css)
    }
}
