use crate::component::CssValueOrigin;

/// A checked decoded directionality identifier; unknown values remain authored syntax.
/// Equality retains component spelling and provenance, without evaluating direction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssDirectionality {
    pub(crate) component: Box<crate::component::CssComponentValue>,
}

impl CssDirectionality {
    /// Constructs a decoded identifier, escaping it as needed. Empty and NUL fail.
    pub fn try_new(
        value: impl Into<String>,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        Self::from_component(crate::component::CssComponentValue::try_ident(value)?)
    }

    pub(crate) fn from_component(
        component: crate::component::CssComponentValue,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        if matches!(
            component.view(),
            crate::component::CssComponentValueRef::Token(
                crate::component::CssValueTokenRef::Ident(_)
            )
        ) {
            Ok(Self {
                component: Box::new(component),
            })
        } else {
            Err(crate::component::CssComponentValueError::new(
                crate::component::values::CssComponentValueErrorKind::InvalidIdentifier,
                component.origin().clone(),
            ))
        }
    }

    /// Returns the decoded, case-preserving identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self.component.view() {
            crate::component::CssComponentValueRef::Token(
                crate::component::CssValueTokenRef::Ident(value),
            ) => value,
            _ => unreachable!("checked directionality identifier"),
        }
    }

    /// Returns the original token origin or explicit programmatic provenance.
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }

    /// Serializes the argument as a canonical identifier, without the pseudo-class.
    #[must_use]
    pub fn to_css_string(&self) -> String {
        surgeist_css_parser::Token::Ident(self.as_str()).to_string()
    }
}
