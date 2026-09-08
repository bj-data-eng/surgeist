use crate::{CustomPropertyName, Error, ErrorCode, Property};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StyleSourceId {
    value: u64,
}

impl StyleSourceId {
    pub fn try_new(value: u64) -> crate::Result<Self> {
        if value == 0 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "style source id must be non-zero",
            ));
        }

        Ok(Self { value })
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleDiagnostic {
    kind: StyleDiagnosticKind,
    subject: StyleDiagnosticSubject,
    source: Option<StyleSourceId>,
    reason: InvalidAtComputedValueReason,
}

impl StyleDiagnostic {
    #[must_use]
    pub fn invalid_at_computed_value(
        subject: StyleDiagnosticSubject,
        source: Option<StyleSourceId>,
        reason: InvalidAtComputedValueReason,
    ) -> Self {
        Self {
            kind: StyleDiagnosticKind::InvalidAtComputedValue,
            subject,
            source,
            reason,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> StyleDiagnosticKind {
        self.kind
    }

    #[must_use]
    pub const fn subject(&self) -> &StyleDiagnosticSubject {
        &self.subject
    }

    #[must_use]
    pub const fn source(&self) -> Option<StyleSourceId> {
        self.source
    }

    #[must_use]
    pub const fn reason(&self) -> &InvalidAtComputedValueReason {
        &self.reason
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StyleDiagnosticKind {
    InvalidAtComputedValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StyleDiagnosticSubject {
    Property(Property),
    CustomProperty(CustomPropertyName),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidAtComputedValueReason {
    MissingCustomProperty(CustomPropertyName),
    InvalidCustomProperty(CustomPropertyName),
    MissingTypedCustomPropertyValue(CustomPropertyName, Property),
    CyclicCustomProperty(CustomPropertyName),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CustomPropertyName, Property};

    #[test]
    fn style_source_ids_reject_zero_and_preserve_opaque_value() {
        assert!(StyleSourceId::try_new(0).is_err());
        let source = StyleSourceId::try_new(42).unwrap();
        assert_eq!(source.get(), 42);
    }

    #[test]
    fn invalid_at_computed_value_diagnostics_preserve_subject_and_source() {
        let source = StyleSourceId::try_new(7).unwrap();
        let diagnostic = StyleDiagnostic::invalid_at_computed_value(
            StyleDiagnosticSubject::Property(Property::Color),
            Some(source),
            InvalidAtComputedValueReason::MissingCustomProperty(
                CustomPropertyName::try_new("--brand").unwrap(),
            ),
        );

        assert_eq!(
            diagnostic.kind(),
            StyleDiagnosticKind::InvalidAtComputedValue
        );
        assert_eq!(diagnostic.source(), Some(source));
        assert_eq!(
            diagnostic.subject(),
            &StyleDiagnosticSubject::Property(Property::Color)
        );
    }
}
