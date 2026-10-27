//! Authored import serialization using checked clause and media admission.
use crate::component::values::serialize::{CssCanonicalBuilder, CssCanonicalToken};
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueLimits, CssComponentValues,
    CssSerializedValue, CssValueOrigin,
};
use crate::query::media::{CssMediaConstructionError, CssMediaQuery, CssMediaQueryList};
use crate::query::supports::CssSupportsConstructionError;
use crate::rule::{CssImportRule, CssImportTarget, CssRuleConstructionError};
use crate::selector::CssNamespaceContext;
use crate::serialization::{
    CssRuleSerializationError, CssSerializationError, CssSerializationLimits,
};
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
    pub(crate) const fn from_checked_parts(
        rule: &'a CssImportRule,
        media: &'a CssMediaQueryList,
    ) -> Self {
        Self { rule, media }
    }

    /// Checks the complete current payload under one cumulative formatting budget.
    /// An empty supplied list removes authored media from output. Existing import
    /// wrapper qualifications and recovered-media CSSOM behavior remain intact.
    pub fn try_new(
        rule: &'a CssImportRule,
        media: &'a CssMediaQueryList,
        limits: CssSerializationLimits,
    ) -> Result<Self, CssRuleSerializationError> {
        let view = Self { rule, media };
        view.serialize_cssom(limits)?;
        Ok(view)
    }

    /// Returns the borrowed import rule.
    #[must_use]
    pub const fn rule(self) -> &'a CssImportRule {
        self.rule
    }

    /// Returns the borrowed media query list that stands in for the rule's authored media.
    #[must_use]
    pub const fn media(self) -> &'a CssMediaQueryList {
        self.media
    }

    /// Uses the owning import writer; failure returns no partial text and permits
    /// unchanged retry without modifying either borrowed input.
    pub fn serialize_cssom(
        self,
        limits: CssSerializationLimits,
    ) -> Result<String, CssRuleSerializationError> {
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer
            .node()
            .map_err(crate::serialization::cssom_rule_serialization::RuleCssomSource::from)
            .and_then(|()| {
                self.rule
                    .append_cssom_with_media(&mut writer, Some(self.media))
                    .map_err(Into::into)
            })
            .map_err(|error| CssRuleSerializationError::new(error, Vec::new(), None))?;
        Ok(writer.css)
    }
}
/// A lexical import emission failure, before its public classification.
///
/// Rule-graph providers and the public token serializer map it to
/// [`CssSerializationError`] through [`import_error`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ImportSerializationFailure {
    Component(CssComponentValueError),
    Media(crate::query::media::construction::MediaSerializationFailure),
    InterpretationChanged { origin: CssValueOrigin },
}
impl From<CssComponentValueError> for ImportSerializationFailure {
    fn from(value: CssComponentValueError) -> Self {
        Self::Component(value)
    }
}
impl From<crate::query::media::construction::MediaSerializationFailure>
    for ImportSerializationFailure
{
    fn from(value: crate::query::media::construction::MediaSerializationFailure) -> Self {
        Self::Media(value)
    }
}
/// Classifies an import emission failure: a component failure by its kind, a
/// media failure as media serialization classifies it, and a changed
/// interpretation as an unrepresentable value.
pub(crate) fn import_error(error: ImportSerializationFailure) -> CssSerializationError {
    match error {
        ImportSerializationFailure::Component(error) => CssSerializationError::component(
            crate::component::values::specified_component_kind(&error),
            error,
        ),
        ImportSerializationFailure::Media(error) => {
            crate::query::media::construction::media_error(error)
        }
        ImportSerializationFailure::InterpretationChanged { origin } => {
            CssSerializationError::import_interpretation(&origin)
        }
    }
}
/// A complete import construction failure located in the original input, read
/// through [`CssRuleConstructionError`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ImportConstructionKind {
    InvalidRuleGrammar { origin: CssValueOrigin },
    RecoveredInput { origin: CssValueOrigin },
    Component(CssComponentValueError),
    Supports(CssSupportsConstructionError),
    Media(CssMediaConstructionError),
    Serialization(CssSerializationError),
    WorkerUnavailable { origin: CssValueOrigin },
}
impl ImportConstructionKind {
    pub(crate) fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::InvalidRuleGrammar { origin }
            | Self::RecoveredInput { origin }
            | Self::WorkerUnavailable { origin } => origin,
            Self::Component(error) => error.origin(),
            Self::Supports(error) => error.origin(),
            Self::Media(error) => error.origin(),
            Self::Serialization(error) => error.origin().unwrap_or(&CssValueOrigin::Programmatic),
        }
    }
    pub(crate) fn describe(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid authored import construction: ")?;
        f.write_str(match self {
            Self::InvalidRuleGrammar { .. } => "the components do not form an @import rule",
            Self::RecoveredInput { .. } => "an implicitly closed component is not admitted",
            Self::Component(_) => "a component is invalid",
            Self::Supports(_) => "the supports condition is invalid",
            Self::Media(_) => "the media query list is invalid",
            Self::Serialization(_) => "the rule cannot be serialized",
            Self::WorkerUnavailable { .. } => "the construction thread could not be started",
        })
    }
    pub(crate) fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Component(error) => Some(error),
            Self::Supports(error) => Some(error),
            Self::Media(error) => Some(error),
            Self::Serialization(error) => Some(error),
            _ => None,
        }
    }
}
impl From<CssComponentValueError> for CssRuleConstructionError {
    fn from(error: CssComponentValueError) -> Self {
        ImportConstructionKind::Component(error).into()
    }
}
impl From<CssSupportsConstructionError> for CssRuleConstructionError {
    fn from(error: CssSupportsConstructionError) -> Self {
        ImportConstructionKind::Supports(error).into()
    }
}
impl From<CssMediaConstructionError> for CssRuleConstructionError {
    fn from(error: CssMediaConstructionError) -> Self {
        ImportConstructionKind::Media(error).into()
    }
}
impl From<CssSerializationError> for CssRuleConstructionError {
    fn from(error: CssSerializationError) -> Self {
        ImportConstructionKind::Serialization(error).into()
    }
}

impl CssImportRule {
    /// Checks one complete import, including its explicit semicolon. Surrounding
    /// trivia is allowed and counted against input limits; canonical output may omit it.
    ///
    /// Checks lexical input and canonical output under the supplied resource limits.
    /// Recovered components and malformed media are rejected before any rule escapes.
    pub fn try_from_components(
        values: CssComponentValues,
        context: &CssNamespaceContext,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssRuleConstructionError> {
        if values.nesting_depth() < 32 {
            return crate::parser::import::construct_import(values, context, limits);
        }
        let mut values = Some(values);
        std::thread::scope(|scope| {
            let worker = std::thread::Builder::new()
                .name("surgeist-css-import".into())
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(scope, || {
                    crate::parser::import::construct_import(
                        values.take().expect("single import worker"),
                        context,
                        limits,
                    )
                })
                .map_err(|_| {
                    CssRuleConstructionError::from(ImportConstructionKind::WorkerUnavailable {
                        origin: CssValueOrigin::Programmatic,
                    })
                })?;
            match worker.join() {
                Ok(result) => result,
                Err(panic) => std::panic::resume_unwind(panic),
            }
        })
    }

    /// Serializes authored clauses and symbolic media as tokens with their
    /// origins, without evaluating or loading them.
    ///
    /// Recovered media members prevent serialization of the entire rule.
    /// `limits.max_css_bytes()` caps canonical output, including inserted
    /// syntax and expanded symbolic media representations; it does not cap
    /// temporary allocation for interpretation probes. The other component
    /// limits do not apply. No partial output is returned.
    ///
    /// # Errors
    ///
    /// Returns [`CssSerializationError`] for a recovered media member, an
    /// exhausted byte limit, or output that would reparse differently.
    pub fn serialize_tokens(
        &self,
        limits: CssComponentValueLimits,
    ) -> Result<CssSerializedValue, CssSerializationError> {
        let max_css_bytes = limits.max_css_bytes();
        // Both original clauses and symbolic media can contain deep component trees.
        let deep = [&self.syntax.prelude.target]
            .into_iter()
            .chain(self.syntax.prelude.layer.iter())
            .chain(self.syntax.prelude.supports.iter())
            .any(|component| crate::query::media::construction::component_depth(component) >= 64)
            || self.media().is_some_and(|media| {
                media
                    .queries()
                    .iter()
                    .any(crate::query::media::construction::query_is_deep)
            });
        let origin = || self.syntax.prelude.target.origin().clone();
        crate::query::media::construction::with_media_stack(deep, origin, || {
            self.serialize_import(max_css_bytes)
        })
        .map_err(import_error)
    }
    /// Generic graph projection: the enclosing graph owns the rule node.
    /// Targets and clauses own their retained component trees; the media provider
    /// owns its list aggregate and member graph, charged before any text scratch.
    pub(crate) fn append_specified(
        &self,
        context: &mut crate::serialization::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<
        (),
        crate::serialization::specified_rule_serialization::SpecifiedRuleSerializationSource,
    > {
        self.append_specified_with_media(context, output, self.media())
    }

    pub(crate) fn append_specified_with_media(
        &self,
        context: &mut crate::serialization::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
        media: Option<&CssMediaQueryList>,
    ) -> Result<
        (),
        crate::serialization::specified_rule_serialization::SpecifiedRuleSerializationSource,
    > {
        for component in [&self.syntax.prelude.target]
            .into_iter()
            .chain(self.syntax.prelude.layer.iter())
            .chain(self.syntax.prelude.supports.iter())
        {
            crate::component::values::charge_specified_components(
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
            .any(|component| crate::query::media::construction::component_depth(component) >= 64)
            || media.is_some_and(|media| {
                media
                    .queries()
                    .iter()
                    .any(crate::query::media::construction::query_is_deep)
            });
        let origin = || self.syntax.prelude.target.origin().clone();
        let value = crate::query::media::construction::with_media_stack(deep, origin, || {
            // Interpretation probes use the same grammar and protections as the
            // direct owner, but scratch is bounded by remaining operation bytes.
            let protect = self.import_protection(
                media,
                context.remaining_bytes(),
                crate::query::media::construction::emit_query_projected,
            )?;
            let mut out = CssCanonicalBuilder::new(context.remaining_bytes());
            self.emit_import_into(
                &mut out,
                media,
                protect,
                crate::query::media::construction::emit_query_projected,
            )?;
            Ok::<_, ImportSerializationFailure>(out.finish()?)
        })?;
        context.append(output, value.as_css())?;
        Ok(())
    }

    /// The CSSOM editor capture (`references/cssom-1--editor-capture-20261009--d42e145ec395.md`,
    /// CSSImportRule) serializes every import as `@import`, one space, serialize a
    /// URL on the decoded location, the media list and `;` (#1327). Its steps have
    /// no `layer` or `supports()` clause; they follow the URL in `@import` grammar
    /// order, the pinned WebKit `CSSImportRule::cssTextInternal` witness order.
    pub(crate) fn append_cssom(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<
        (),
        crate::serialization::specified_rule_serialization::SpecifiedRuleSerializationSource,
    > {
        self.append_cssom_with_media(writer, self.media())
    }

    pub(crate) fn append_cssom_with_media(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
        media: Option<&CssMediaQueryList>,
    ) -> Result<
        (),
        crate::serialization::specified_rule_serialization::SpecifiedRuleSerializationSource,
    > {
        // Serialize a URL takes only the location: string, `url()` and `src()`
        // targets share it, and request modifiers have no CSSOM step.
        let location = match self.target() {
            CssImportTarget::String(value) => value.as_str(),
            CssImportTarget::Url(value) => value.as_str(),
        };
        // Charge retained syntax rather than a newly manufactured URL graph.
        for component in [&self.syntax.prelude.target]
            .into_iter()
            .chain(self.syntax.prelude.layer.iter())
            .chain(self.syntax.prelude.supports.iter())
        {
            crate::component::values::charge_specified_components(
                &mut writer.context,
                std::slice::from_ref(component),
            )?;
        }
        if let Some(media) = media {
            media.charge_cssom(&mut writer.context)?;
        }
        if writer.context.output_suppressed() {
            return Ok(());
        }
        let deep = [&self.syntax.prelude.layer, &self.syntax.prelude.supports]
            .into_iter()
            .flatten()
            .any(|component| crate::query::media::construction::component_depth(component) >= 64)
            || media.is_some_and(|media| {
                media
                    .queries()
                    .iter()
                    .any(crate::query::media::construction::query_is_deep)
            });
        let origin = || self.syntax.prelude.target.origin().clone();
        crate::query::media::construction::with_media_stack(deep, origin, || {
            // Reuse clause/media interpretation probes even when the URL target
            // itself has been normalized. Scratch stays within remaining bytes.
            let protect = self.import_protection(
                media,
                writer.context.remaining_bytes(),
                crate::query::media::construction::emit_query_cssom,
            )?;
            writer.append("@import url(")?;
            writer.append_string(location)?;
            writer.append(")")?;
            let (tail, _) = self.import_tail(
                media,
                protect,
                writer.context.remaining_bytes(),
                crate::query::media::construction::emit_query_cssom,
            )?;
            writer.append(tail.as_css())?;
            writer.append(";")?;
            Ok(())
        })
    }

    fn serialize_import(
        &self,
        max_css_bytes: usize,
    ) -> Result<CssSerializedValue, ImportSerializationFailure> {
        if let Some(media) = self.media()
            && let Some(CssMediaQuery::Never(value)) = media
                .queries()
                .iter()
                .find(|q| matches!(q, CssMediaQuery::Never(_)))
        {
            return Err(
                crate::query::media::construction::MediaSerializationFailure::RecoveredNever {
                    origin: value.origin().clone(),
                }
                .into(),
            );
        }
        // The direct byte-only contract retains its unlimited interpretation probes.
        let protect = self.import_protection(
            self.media(),
            usize::MAX,
            crate::query::media::construction::emit_query,
        )?;
        let mut out = CssCanonicalBuilder::new(max_css_bytes);
        self.emit_import_into(
            &mut out,
            self.media(),
            protect,
            crate::query::media::construction::emit_query,
        )?;
        Ok(out.finish()?)
    }
    fn import_protection(
        &self,
        media: Option<&CssMediaQueryList>,
        max_bytes: usize,
        emit_query: fn(
            &mut CssCanonicalBuilder,
            &CssMediaQuery,
        )
            -> Result<(), crate::query::media::construction::MediaSerializationFailure>,
    ) -> Result<bool, ImportSerializationFailure> {
        let (tail, expected) = self.import_tail(media, false, max_bytes, emit_query)?;
        let origin = self.syntax.at_keyword.origin();
        let protect = if crate::parser::import::import_boundaries_match(&tail, expected, origin)? {
            false
        } else {
            if !matches!(
                media.and_then(|media| media.queries().first()),
                Some(CssMediaQuery::Condition(_))
            ) {
                return Err(ImportSerializationFailure::InterpretationChanged {
                    origin: origin.clone(),
                });
            }
            let (protected, boundaries) = self.import_tail(media, true, max_bytes, emit_query)?;
            if !crate::parser::import::import_boundaries_match(&protected, boundaries, origin)? {
                return Err(ImportSerializationFailure::InterpretationChanged {
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
        )
            -> Result<(), crate::query::media::construction::MediaSerializationFailure>,
    ) -> Result<(), ImportSerializationFailure> {
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
        )
            -> Result<(), crate::query::media::construction::MediaSerializationFailure>,
    ) -> Result<(CssSerializedValue, [Option<usize>; 2]), ImportSerializationFailure> {
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
        )
            -> Result<(), crate::query::media::construction::MediaSerializationFailure>,
    ) -> Result<[Option<usize>; 2], ImportSerializationFailure> {
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
