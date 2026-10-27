//! Authored names and selector arguments from CSS View Transitions 1.

use crate::serialization::CssSerializationError;
use crate::values::common::CssCustomIdent;

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

    /// Returns the actual owned decoded-name buffer capacity in bytes, in
    /// constant work without allocation. Includes spare capacity; excludes the
    /// inline value header, allocator bookkeeping and unrelated shared graphs.
    #[must_use]
    pub fn retained_buffer_bytes(&self) -> usize {
        self.0.retained_buffer_bytes()
    }
}

/// The specified `none | <custom-ident>` value, before participant discovery.
///
/// # Specified serialization
///
/// Emits canonical specified CSS using the shared identifier escaping rules.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssViewTransitionName {
    /// `none`: the element does not participate separately in a view transition.
    None,
    /// A `<custom-ident>` that names the element as a view-transition participant.
    Custom(CssViewTransitionIdent),
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssViewTransitionName
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        writer.node()?;
        match self {
            Self::None => writer.append("none"),
            Self::Custom(name) => writer.append_identifier(name.as_str()),
        }
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssViewTransitionName);

/// A named transition pseudo-element's wildcard or case-sensitive custom name.
/// Unlike the property name, selector names admit `none` and `auto`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssViewTransitionNameSelector {
    /// `*`: the wildcard that matches every participant name.
    Wildcard,
    /// A case-sensitive `<custom-ident>` participant name.
    Name(CssCustomIdent),
}
