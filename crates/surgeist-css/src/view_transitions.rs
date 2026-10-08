//! Authored names and selector arguments from CSS View Transitions 1.

use crate::{
    CssCustomIdent, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

/// A checked participant name in the `view-transition-name` custom-ident arm.
/// The decoded name preserves case; `none` and `auto` are excluded in every ASCII case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssViewTransitionIdent(CssCustomIdent);

impl CssViewTransitionIdent {
    /// Applies the property-specific exclusions to a checked custom identifier.
    #[must_use]
    pub fn try_new(value: CssCustomIdent) -> Option<Self> {
        (!value.as_str().eq_ignore_ascii_case("none")
            && !value.as_str().eq_ignore_ascii_case("auto"))
        .then_some(Self(value))
    }

    /// Borrows the decoded, case-preserved name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// The specified `none | <custom-ident>` value, before participant discovery.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssViewTransitionName {
    None,
    Custom(CssViewTransitionIdent),
}

impl CssViewTransitionName {
    /// Emits canonical specified CSS using the shared identifier escaping rules.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges one input and one projection node, and every emitted UTF-8 byte.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        match self {
            Self::None => writer.append("none"),
            Self::Custom(name) => writer.append_identifier(name.as_str()),
        }
    }
}

/// A named transition pseudo-element's wildcard or case-sensitive custom name.
/// Unlike the property name, selector names admit `none` and `auto`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssViewTransitionNameSelector {
    Wildcard,
    Name(CssCustomIdent),
}
