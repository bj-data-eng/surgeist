//! Authored import serialization using checked clause and media admission.
use crate::component_values::{CssCanonicalBuilder, CssCanonicalToken};
use crate::*;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ImportPreludeSyntax {
    pub(crate) target: CssComponentValue,
    pub(crate) layer: Option<CssComponentValue>,
    pub(crate) supports: Option<CssComponentValue>,
    pub(crate) semicolon: CssValueOrigin,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ImportSyntax {
    pub(crate) at_keyword: CssComponentValue,
    pub(crate) prelude: ImportPreludeSyntax,
}
/// Failure to serialize a checked authored import atomically.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssImportSerializationError {
    Component(CssComponentValueError),
    Media(CssMediaSerializationError),
    InterpretationChanged { origin: CssValueOrigin },
}
impl CssImportSerializationError {
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::Component(value) => value.origin(),
            Self::Media(value) => value.origin(),
            Self::InterpretationChanged { origin } => origin,
        }
    }
}
impl fmt::Display for CssImportSerializationError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "import serialization failed: {self:?}")
    }
}
impl std::error::Error for CssImportSerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Component(error) => Some(error),
            Self::Media(error) => Some(error),
            Self::InterpretationChanged { .. } => None,
        }
    }
}
impl From<CssComponentValueError> for CssImportSerializationError {
    fn from(value: CssComponentValueError) -> Self {
        Self::Component(value)
    }
}
impl From<CssMediaSerializationError> for CssImportSerializationError {
    fn from(value: CssMediaSerializationError) -> Self {
        Self::Media(value)
    }
}
impl CssImportRule {
    /// Serializes authored clauses and symbolic media without evaluating or loading them.
    /// Recovered media members prevent serialization of the entire rule.
    pub fn serialize(&self) -> Result<CssSerializedValue, CssImportSerializationError> {
        // Both original clauses and symbolic media can contain deep component trees.
        let deep = [&self.syntax.prelude.target]
            .into_iter()
            .chain(self.syntax.prelude.layer.iter())
            .chain(self.syntax.prelude.supports.iter())
            .any(|component| crate::media::component_depth(component) >= 64)
            || self
                .media()
                .is_some_and(|media| media.queries().iter().any(crate::media::query_is_deep));
        crate::media::with_media_stack(deep, || self.serialize_import())
    }
    fn serialize_import(&self) -> Result<CssSerializedValue, CssImportSerializationError> {
        if let Some(media) = self.media()
            && let Some(CssMediaQuery::Never(value)) = media
                .queries()
                .iter()
                .find(|q| matches!(q, CssMediaQuery::Never(_)))
        {
            return Err(CssMediaSerializationError::RecoveredNever {
                origin: value.origin().clone(),
            }
            .into());
        }
        let (tail, expected) = self.import_tail(false)?;
        let origin = self.syntax.at_keyword.origin();
        let protect = if crate::parser::import_boundaries_match(&tail, expected, origin)? {
            false
        } else {
            if !matches!(
                self.media().and_then(|media| media.queries().first()),
                Some(CssMediaQuery::Condition(_))
            ) {
                return Err(CssImportSerializationError::InterpretationChanged {
                    origin: origin.clone(),
                });
            }
            let (protected, boundaries) = self.import_tail(true)?;
            if !crate::parser::import_boundaries_match(&protected, boundaries, origin)? {
                return Err(CssImportSerializationError::InterpretationChanged {
                    origin: origin.clone(),
                });
            }
            true
        };
        let mut out = CssCanonicalBuilder::new(usize::MAX);
        out.push_grammar(
            CssCanonicalToken::AtKeyword("import"),
            self.syntax.at_keyword.origin(),
        )?;
        out.push_grammar(
            CssCanonicalToken::Whitespace,
            self.syntax.prelude.target.origin(),
        )?;
        out.push_component(&self.syntax.prelude.target)?;
        self.emit_import_tail(&mut out, protect)?;
        out.push_grammar(CssCanonicalToken::Semicolon, &self.syntax.prelude.semicolon)?;
        Ok(out.finish()?)
    }
    fn import_tail(
        &self,
        protect: bool,
    ) -> Result<(CssSerializedValue, [Option<usize>; 2]), CssImportSerializationError> {
        let mut out = CssCanonicalBuilder::new(usize::MAX);
        let expected = self.emit_import_tail(&mut out, protect)?;
        Ok((out.finish()?, expected))
    }
    fn emit_import_tail(
        &self,
        out: &mut CssCanonicalBuilder,
        protect: bool,
    ) -> Result<[Option<usize>; 2], CssImportSerializationError> {
        let mut boundaries = [None, None];
        for (index, component) in [&self.syntax.prelude.layer, &self.syntax.prelude.supports]
            .into_iter()
            .enumerate()
        {
            if let Some(component) = component {
                out.push_grammar(CssCanonicalToken::Whitespace, component.origin())?;
                out.push_component(component)?;
                boundaries[index] = Some(out.byte_len());
            }
        }
        if let Some(media) = self.media() {
            for (index, query) in media.queries().iter().enumerate() {
                let origin = if index == 0 {
                    query.origin()
                } else {
                    media
                        .comma_origins
                        .get(index - 1)
                        .unwrap_or(&CssValueOrigin::Programmatic)
                };
                if index > 0 {
                    out.push_grammar(CssCanonicalToken::Comma, origin)?;
                }
                out.push_grammar(CssCanonicalToken::Whitespace, origin)?;
                if index == 0 && protect {
                    out.push_grammar(CssCanonicalToken::OpenParen, &CssValueOrigin::Programmatic)?;
                }
                crate::media::emit_query(out, query)?;
                if index == 0 && protect {
                    out.push_grammar(CssCanonicalToken::CloseParen, &CssValueOrigin::Programmatic)?;
                }
            }
        }
        Ok(boundaries)
    }
}
