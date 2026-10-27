//! Owner-local diagnostic payload observation; source handles are measured separately.
use crate::diagnostic::error::{CssDeclarationContext, ErrorKind};
use crate::diagnostic::{CssError, CssTokenSummary};
use crate::rule::CssDescriptorName;
use crate::source::CssRetainedPayloadError;

pub(crate) struct DiagnosticBacking<'a, F> {
    pub(crate) admit: &'a mut F,
    pub(crate) bytes: usize,
}
impl<E, F: FnMut() -> Result<(), E>> DiagnosticBacking<'_, F> {
    fn step(&mut self) -> Result<(), CssRetainedPayloadError<E>> {
        (self.admit)().map_err(CssRetainedPayloadError::admission)
    }
    fn add(&mut self, bytes: usize) -> Result<(), CssRetainedPayloadError<E>> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or_else(CssRetainedPayloadError::capacity_overflow)?;
        Ok(())
    }
    fn string(&mut self, value: &String) -> Result<(), CssRetainedPayloadError<E>> {
        self.step()?;
        self.add(value.capacity())
    }
    fn token(&mut self, value: &Option<CssTokenSummary>) -> Result<(), CssRetainedPayloadError<E>> {
        self.step()?;
        if let Some(value) = value {
            self.string(&value.authored)?;
        }
        Ok(())
    }
    pub(crate) fn kind(&mut self, kind: &ErrorKind) -> Result<(), CssRetainedPayloadError<E>> {
        self.step()?;
        match kind {
            ErrorKind::EscapeParseError(_)
            | ErrorKind::UnexpectedEnd(_)
            | ErrorKind::NestingLimit(_)
            | ErrorKind::ParserUnavailable(_) => Ok(()),
            ErrorKind::UnexpectedToken(value) => self.string(&value.encountered.authored),
            ErrorKind::InvalidAtRulePlacement(value) => self.string(&value.name.0),
            ErrorKind::InvalidAtRulePrelude(value) | ErrorKind::InvalidAtRuleBody(value) => {
                self.string(&value.name.0)?;
                self.token(&value.encountered)
            }
            ErrorKind::NamespaceRedeclaration(value) => {
                self.step()?;
                if let Some(prefix) = &value.prefix {
                    self.step()?;
                    self.add(prefix.diagnostic_backing_bytes())?;
                }
                Ok(())
            }
            ErrorKind::LegacyNavigationTarget(value) => self.string(&value.target),
            ErrorKind::UnknownAtRule(value) => self.string(&value.name.0),
            ErrorKind::UnsupportedAtRule(value) => self.string(&value.name.0),
            ErrorKind::InvalidQualifiedRule(value) => self.token(&value.encountered),
            ErrorKind::InvalidSelector(value) => self.token(&value.encountered),
            ErrorKind::InvalidMediaQuery(value) => {
                self.step()?;
                if let Some(feature) = &value.feature {
                    self.string(&feature.0)?;
                }
                self.token(&value.encountered)
            }
            ErrorKind::UnknownProperty(value) => self.string(&value.name.0),
            ErrorKind::UnsupportedProperty(value) => self.string(&value.name.0),
            ErrorKind::InvalidPropertyValue(value) => self.token(&value.encountered),
            ErrorKind::InvalidComponentValue(_) => {
                // The pointee contains scalar kind and inline source handles only.
                self.step()?;
                self.add(std::mem::size_of::<crate::component::CssComponentValueError>())
            }
            ErrorKind::InvalidDeclarationAnnotation(value) => {
                self.step()?;
                match &value.context {
                    CssDeclarationContext::OrdinaryCustom(name)
                    | CssDeclarationContext::KeyframeCustom(name) => {
                        self.step()?;
                        self.add(name.diagnostic_backing_bytes())?;
                    }
                    CssDeclarationContext::Descriptor {
                        at_rule,
                        descriptor,
                    } => {
                        self.string(&at_rule.0)?;
                        self.string(&descriptor.0)?;
                    }
                    CssDeclarationContext::SvgGlyph
                    | CssDeclarationContext::KeyframeSvgGlyph
                    | CssDeclarationContext::OrdinaryKnown(_)
                    | CssDeclarationContext::Keyframe(_) => {}
                }
                self.string(&value.encountered.authored)
            }
            ErrorKind::UnknownDescriptor(value) => {
                self.string(&value.at_rule.0)?;
                self.string(&value.descriptor.0)
            }
            ErrorKind::UnsupportedDescriptor(value) => {
                self.string(&value.at_rule.0)?;
                self.string(&value.descriptor.0)
            }
            ErrorKind::InvalidDescriptorValue(value) => {
                self.string(&value.at_rule.0)?;
                self.step()?;
                self.add(std::mem::size_of::<CssDescriptorName>())?;
                self.string(&value.descriptor.0)?;
                self.token(&value.encountered)
            }
            ErrorKind::InvalidDescriptorCombination(value) => {
                self.string(&value.at_rule.0)?;
                self.string(&value.responsible.0)?;
                self.step()?;
                self.add(
                    value
                        .conflicting
                        .capacity()
                        .checked_mul(std::mem::size_of::<CssDescriptorName>())
                        .ok_or_else(CssRetainedPayloadError::capacity_overflow)?,
                )?;
                for name in &value.conflicting {
                    self.string(&name.0)?;
                }
                Ok(())
            }
            ErrorKind::InvalidColorSyntax(value) => {
                self.step()?;
                if let Some(component) = &value.component {
                    self.string(&component.0)?;
                }
                self.token(&value.encountered)
            }
        }
    }
}
impl CssError {
    pub(crate) fn retained_backing_bytes_with<E>(
        &self,
        admit: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<usize, CssRetainedPayloadError<E>> {
        let mut measure = DiagnosticBacking { admit, bytes: 0 };
        measure.kind(&self.kind)?;
        Ok(measure.bytes)
    }
}

impl ErrorKind {
    pub(crate) fn retained_backing_bytes_with<E>(
        &self,
        admit: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<usize, CssRetainedPayloadError<E>> {
        let mut measure = DiagnosticBacking { admit, bytes: 0 };
        measure.kind(self)?;
        Ok(measure.bytes)
    }
}

#[cfg(test)]
pub(crate) mod tests;
