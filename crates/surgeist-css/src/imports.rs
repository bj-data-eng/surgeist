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

/// A checked borrowed import with a supplied current rule-owned media list.
///
/// The original rule supplies its checked target, layer and supports clauses and
/// genuine provenance. The supplied media replaces only the formatting input;
/// no authored rule, live identity, URL resolution or source position is created.
#[derive(Clone, Copy, Debug)]
pub struct CssImportRuleView<'a> {
    rule: &'a CssImportRule,
    media: &'a CssMediaQueryList,
}

impl<'a> CssImportRuleView<'a> {
    pub fn try_new(
        rule: &'a CssImportRule,
        media: &'a CssMediaQueryList,
    ) -> Result<Self, CssRuleCssomSerializationError> {
        Self::try_new_with_limits(rule, media, CssSpecifiedValueSerializationLimits::default())
    }

    /// Checks the complete current payload under one cumulative formatting budget.
    /// An empty supplied list removes authored media from output. Existing import
    /// wrapper qualifications and recovered-media CSSOM behavior remain intact.
    pub fn try_new_with_limits(
        rule: &'a CssImportRule,
        media: &'a CssMediaQueryList,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, CssRuleCssomSerializationError> {
        let view = Self { rule, media };
        view.serialize_cssom_with_limits(limits)?;
        Ok(view)
    }

    #[must_use]
    pub const fn rule(self) -> &'a CssImportRule {
        self.rule
    }

    #[must_use]
    pub const fn media(self) -> &'a CssMediaQueryList {
        self.media
    }

    pub fn serialize_cssom(self) -> Result<String, CssRuleCssomSerializationError> {
        self.serialize_cssom_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Uses the owning import writer; failure returns no partial text and permits
    /// unchanged retry without modifying either borrowed input.
    pub fn serialize_cssom_with_limits(
        self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssRuleCssomSerializationError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer
            .node()
            .map_err(crate::cssom_rule_serialization::RuleCssomSource::from)
            .and_then(|()| {
                self.rule
                    .append_cssom_with_media(&mut writer, Some(self.media))
                    .map_err(Into::into)
            })
            .map_err(|error| CssRuleCssomSerializationError::new(error, Vec::new(), None))?;
        Ok(writer.css)
    }
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
    /// Generic graph projection: the enclosing graph owns the rule node.
    /// Targets and clauses own their retained component trees; the media provider
    /// owns its list aggregate and member graph, charged before any text scratch.
    pub(crate) fn append_specified(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), crate::query_rule_serialization::QueryRuleSerializationError> {
        self.append_specified_with_media(context, output, self.media())
    }

    pub(crate) fn append_specified_with_media(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
        media: Option<&CssMediaQueryList>,
    ) -> Result<(), crate::query_rule_serialization::QueryRuleSerializationError> {
        for component in [&self.syntax.prelude.target]
            .into_iter()
            .chain(self.syntax.prelude.layer.iter())
            .chain(self.syntax.prelude.supports.iter())
        {
            crate::component_values::charge_specified_components(
                context,
                std::slice::from_ref(component),
            )?;
        }
        if let Some(media) = media {
            media.charge_cssom(context)?;
        }
        if context.output_suppressed() {
            return Ok(());
        }
        let deep = [&self.syntax.prelude.target]
            .into_iter()
            .chain(self.syntax.prelude.layer.iter())
            .chain(self.syntax.prelude.supports.iter())
            .any(|component| crate::media::component_depth(component) >= 64)
            || media.is_some_and(|media| media.queries().iter().any(crate::media::query_is_deep));
        let value = crate::media::with_media_stack(deep, || {
            // Interpretation probes use the same grammar and protections as the
            // direct owner, but scratch is bounded by remaining operation bytes.
            let protect = self.import_protection(
                media,
                context.remaining_bytes(),
                crate::media::emit_query_cssom,
            )?;
            let mut out = CssCanonicalBuilder::new(context.remaining_bytes());
            self.emit_import_into(&mut out, media, protect, crate::media::emit_query_cssom)?;
            Ok::<_, CssImportSerializationError>(out.finish()?)
        })?;
        context.append(output, value.as_css())?;
        Ok(())
    }

    /// CSSOM WD20210826 defines plain import locations as serialized URLs.
    /// Modern clauses and extended URL forms keep their retained-output contract
    /// until their complete literal wrapper has an independently selected source.
    pub(crate) fn append_cssom(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), crate::query_rule_serialization::QueryRuleSerializationError> {
        self.append_cssom_with_media(writer, self.media())
    }

    pub(crate) fn append_cssom_with_media(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
        media: Option<&CssMediaQueryList>,
    ) -> Result<(), crate::query_rule_serialization::QueryRuleSerializationError> {
        let location = match self.target() {
            CssImportTarget::String(value) => value.as_str(),
            CssImportTarget::Url(value)
                if value.url().function() == CssUrlFunction::Url
                    && value.url().modifiers().is_empty() =>
            {
                value.as_str()
            }
            _ => {
                return self.append_specified_with_media(
                    &mut writer.context,
                    &mut writer.css,
                    media,
                );
            }
        };
        if self.layer().is_some() || self.supports().is_some() {
            return self.append_specified_with_media(&mut writer.context, &mut writer.css, media);
        }
        // Charge retained syntax rather than a newly manufactured URL graph.
        crate::component_values::charge_specified_components(
            &mut writer.context,
            std::slice::from_ref(&self.syntax.prelude.target),
        )?;
        if let Some(media) = media {
            media.charge_cssom(&mut writer.context)?;
        }
        if writer.context.output_suppressed() {
            return Ok(());
        }
        let deep =
            media.is_some_and(|media| media.queries().iter().any(crate::media::query_is_deep));
        crate::media::with_media_stack(deep, || {
            // Reuse clause/media interpretation probes even when the URL target
            // itself has been normalized. Scratch stays within remaining bytes.
            let protect = self.import_protection(
                media,
                writer.context.remaining_bytes(),
                crate::media::emit_query_cssom,
            )?;
            writer.append("@import url(")?;
            writer.append_string(location)?;
            writer.append(")")?;
            let (tail, _) = self.import_tail(
                media,
                protect,
                writer.context.remaining_bytes(),
                crate::media::emit_query_cssom,
            )?;
            writer.append(tail.as_css())?;
            writer.append(";")?;
            Ok(())
        })
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
        // The direct byte-only contract retains its unlimited interpretation probes.
        let protect = self.import_protection(self.media(), usize::MAX, crate::media::emit_query)?;
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        self.emit_import_into(&mut out, self.media(), protect, crate::media::emit_query)?;
        Ok(out.finish()?)
    }
    fn import_protection(
        &self,
        media: Option<&CssMediaQueryList>,
        max_bytes: usize,
        emit_query: fn(
            &mut CssCanonicalBuilder,
            &CssMediaQuery,
        ) -> Result<(), CssMediaSerializationError>,
    ) -> Result<bool, CssImportSerializationError> {
        let (tail, expected) = self.import_tail(media, false, max_bytes, emit_query)?;
        let origin = self.syntax.at_keyword.origin();
        let protect = if crate::parser::import_boundaries_match(&tail, expected, origin)? {
            false
        } else {
            if !matches!(
                media.and_then(|media| media.queries().first()),
                Some(CssMediaQuery::Condition(_))
            ) {
                return Err(CssImportSerializationError::InterpretationChanged {
                    origin: origin.clone(),
                });
            }
            let (protected, boundaries) = self.import_tail(media, true, max_bytes, emit_query)?;
            if !crate::parser::import_boundaries_match(&protected, boundaries, origin)? {
                return Err(CssImportSerializationError::InterpretationChanged {
                    origin: origin.clone(),
                });
            }
            true
        };
        Ok(protect)
    }
    fn emit_import_into(
        &self,
        out: &mut CssCanonicalBuilder,
        media: Option<&CssMediaQueryList>,
        protect: bool,
        emit_query: fn(
            &mut CssCanonicalBuilder,
            &CssMediaQuery,
        ) -> Result<(), CssMediaSerializationError>,
    ) -> Result<(), CssImportSerializationError> {
        out.push_grammar(
            CssCanonicalToken::AtKeyword("import"),
            self.syntax.at_keyword.origin(),
        )?;
        out.push_grammar(
            CssCanonicalToken::Whitespace,
            self.syntax.prelude.target.origin(),
        )?;
        out.push_component(&self.syntax.prelude.target)?;
        self.emit_import_tail_with(out, media, protect, emit_query)?;
        out.push_grammar(CssCanonicalToken::Semicolon, &self.syntax.prelude.semicolon)?;
        Ok(())
    }
    fn import_tail(
        &self,
        media: Option<&CssMediaQueryList>,
        protect: bool,
        max_bytes: usize,
        emit_query: fn(
            &mut CssCanonicalBuilder,
            &CssMediaQuery,
        ) -> Result<(), CssMediaSerializationError>,
    ) -> Result<(CssSerializedValue, [Option<usize>; 2]), CssImportSerializationError> {
        // Scratch bytes are not final emission bytes. The caller selects the
        // direct byte-only or shared bounded projection contract.
        let mut out = CssCanonicalBuilder::new(max_bytes);
        let expected = self.emit_import_tail_with(&mut out, media, protect, emit_query)?;
        Ok((out.finish()?, expected))
    }
    fn emit_import_tail_with(
        &self,
        out: &mut CssCanonicalBuilder,
        media: Option<&CssMediaQueryList>,
        protect: bool,
        emit_query: fn(
            &mut CssCanonicalBuilder,
            &CssMediaQuery,
        ) -> Result<(), CssMediaSerializationError>,
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
        if let Some(media) = media {
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
                emit_query(out, query)?;
                if index == 0 && protect {
                    out.push_grammar(CssCanonicalToken::CloseParen, &CssValueOrigin::Programmatic)?;
                }
            }
        }
        Ok(boundaries)
    }
}
