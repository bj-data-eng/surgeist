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
/// A complete import construction failure located in the original input.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssImportConstructionError {
    InvalidRuleGrammar { origin: CssValueOrigin },
    RecoveredInput { origin: CssValueOrigin },
    Component(CssComponentValueError),
    Supports(CssSupportsConstructionError),
    Media(CssMediaConstructionError),
    Serialization(CssImportSerializationError),
    WorkerUnavailable { origin: CssValueOrigin },
}
impl CssImportConstructionError {
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::InvalidRuleGrammar { origin }
            | Self::RecoveredInput { origin }
            | Self::WorkerUnavailable { origin } => origin,
            Self::Component(error) => error.origin(),
            Self::Supports(error) => error.origin(),
            Self::Media(error) => error.origin(),
            Self::Serialization(error) => error.origin(),
        }
    }
}
impl fmt::Display for CssImportConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid authored import construction: {self:?}")
    }
}
impl std::error::Error for CssImportConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Component(error) => Some(error),
            Self::Supports(error) => Some(error),
            Self::Media(error) => Some(error),
            Self::Serialization(error) => Some(error),
            _ => None,
        }
    }
}
impl From<CssComponentValueError> for CssImportConstructionError {
    fn from(error: CssComponentValueError) -> Self {
        Self::Component(error)
    }
}
impl From<CssSupportsConstructionError> for CssImportConstructionError {
    fn from(error: CssSupportsConstructionError) -> Self {
        Self::Supports(error)
    }
}
impl From<CssMediaConstructionError> for CssImportConstructionError {
    fn from(error: CssMediaConstructionError) -> Self {
        Self::Media(error)
    }
}
impl From<CssImportSerializationError> for CssImportConstructionError {
    fn from(error: CssImportSerializationError) -> Self {
        Self::Serialization(error)
    }
}

impl CssImportRule {
    /// Checks one complete import, including its explicit semicolon. Surrounding
    /// trivia is allowed and counted against input limits; canonical output may omit it.
    pub fn try_from_components(
        values: CssComponentValues,
        context: &CssNamespaceContext,
    ) -> Result<Self, CssImportConstructionError> {
        Self::try_from_components_with_limits(values, context, CssComponentValueLimits::default())
    }
    /// Checks lexical input and canonical output under the supplied resource limits.
    /// Recovered components and malformed media are rejected before any rule escapes.
    pub fn try_from_components_with_limits(
        values: CssComponentValues,
        context: &CssNamespaceContext,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssImportConstructionError> {
        if values.nesting_depth() < 32 {
            return crate::parser::construct_import(values, context, limits);
        }
        let mut values = Some(values);
        std::thread::scope(|scope| {
            let worker = std::thread::Builder::new()
                .name("surgeist-css-import".into())
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(scope, || {
                    crate::parser::construct_import(
                        values.take().expect("single import worker"),
                        context,
                        limits,
                    )
                })
                .map_err(|_| CssImportConstructionError::WorkerUnavailable {
                    origin: CssValueOrigin::Programmatic,
                })?;
            match worker.join() {
                Ok(result) => result,
                Err(panic) => std::panic::resume_unwind(panic),
            }
        })
    }

    /// Serializes authored clauses and symbolic media without evaluating or loading them.
    /// Recovered media members prevent serialization of the entire rule.
    pub fn serialize(&self) -> Result<CssSerializedValue, CssImportSerializationError> {
        self.serialize_with_limit(usize::MAX)
    }
    /// Applies a byte limit to canonical output, including inserted syntax and
    /// expanded symbolic media representations. No partial output is returned.
    /// This caps output bytes, not temporary allocation for interpretation probes.
    pub fn serialize_with_limit(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, CssImportSerializationError> {
        // Both original clauses and symbolic media can contain deep component trees.
        let deep = [&self.syntax.prelude.target]
            .into_iter()
            .chain(self.syntax.prelude.layer.iter())
            .chain(self.syntax.prelude.supports.iter())
            .any(|component| crate::media::component_depth(component) >= 64)
            || self
                .media()
                .is_some_and(|media| media.queries().iter().any(crate::media::query_is_deep));
        crate::media::with_media_stack(deep, || self.serialize_import(max_css_bytes))
    }
    fn serialize_import(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, CssImportSerializationError> {
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
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
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
        // Clause interpretation probes do not consume the complete-rule budget.
        // Only final ordered emission can identify its first overflowing token.
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
