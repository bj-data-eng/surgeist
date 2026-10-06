//! Literal CSSOM rule composition and the authored ordered sheet aggregate.
use std::fmt;

use crate::{
    CssDeclarationBlockError, CssDeclarationBlockErrorKind, CssMediaCssomSerializationError,
    CssRule, CssSheet, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits,
    specified_rule_serialization::{SpecifiedRuleSerializationSource, SpecifiedRuleWriter},
};

/// A represented rule whose literal format is undefined by the selected sources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRuleCssomKind {
    Page,
}

/// A modern rule with selected subformat providers but no adopted complete literal wrapper.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRuleCssomFormat {
    CounterStyle,
    FontFeatureValues,
    FontPaletteValues,
    ColorProfile,
    SupportsCondition,
    CustomMedia,
    LayerStatement,
    LayerBlock,
    Supports,
    Container,
    Scope,
}

/// Why the complete literal rule request failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRuleCssomSerializationErrorKind {
    Resource(CssSpecifiedValueSerializationErrorKind),
    Value(CssSpecifiedValueSerializationErrorKind),
    DeclarationBlock,
    SourceUndefined(CssRuleCssomKind),
    FormatUnavailable(CssRuleCssomFormat),
}

#[derive(Clone, Debug)]
pub(crate) enum RuleCssomSource {
    Provider(SpecifiedRuleSerializationSource),
    DeclarationBlock(CssDeclarationBlockError),
    SourceUndefined(CssRuleCssomKind),
    FormatUnavailable(CssRuleCssomFormat),
}
impl From<CssSpecifiedValueSerializationError> for RuleCssomSource {
    fn from(value: CssSpecifiedValueSerializationError) -> Self {
        Self::Provider(value.into())
    }
}
impl From<CssMediaCssomSerializationError> for RuleCssomSource {
    fn from(value: CssMediaCssomSerializationError) -> Self {
        Self::Provider(value.into())
    }
}
impl From<crate::query_rule_serialization::QueryRuleSerializationError> for RuleCssomSource {
    fn from(value: crate::query_rule_serialization::QueryRuleSerializationError) -> Self {
        Self::Provider(value.into())
    }
}
impl From<CssDeclarationBlockError> for RuleCssomSource {
    fn from(value: CssDeclarationBlockError) -> Self {
        Self::DeclarationBlock(value)
    }
}
impl From<SpecifiedRuleSerializationSource> for RuleCssomSource {
    fn from(value: SpecifiedRuleSerializationSource) -> Self {
        Self::Provider(value)
    }
}

/// Atomic literal serialization failure, retaining its actual graph and provider provenance.
#[derive(Clone, Debug)]
pub struct CssRuleCssomSerializationError {
    kind: CssRuleCssomSerializationErrorKind,
    rule_path: Vec<usize>,
    keyframe_block_index: Option<usize>,
    source: RuleCssomSource,
}
impl CssRuleCssomSerializationError {
    #[must_use]
    pub const fn kind(&self) -> CssRuleCssomSerializationErrorKind {
        self.kind
    }
    /// Child indexes from the requested rule, or starting with the sheet's rule index.
    #[must_use]
    pub fn rule_path(&self) -> &[usize] {
        &self.rule_path
    }
    #[must_use]
    pub const fn keyframe_block_index(&self) -> Option<usize> {
        self.keyframe_block_index
    }
    pub(crate) fn new(
        source: RuleCssomSource,
        rule_path: Vec<usize>,
        keyframe_block_index: Option<usize>,
    ) -> Self {
        let value = match &source {
            RuleCssomSource::Provider(SpecifiedRuleSerializationSource::Value(value)) => {
                Some(value.kind())
            }
            RuleCssomSource::Provider(SpecifiedRuleSerializationSource::Media(value)) => {
                match value.as_ref() {
                    CssMediaCssomSerializationError::Resource { error, .. } => Some(error.kind()),
                    CssMediaCssomSerializationError::Media(_) => {
                        Some(CssSpecifiedValueSerializationErrorKind::UnserializableBoundary)
                    }
                }
            }
            RuleCssomSource::DeclarationBlock(error) => match error.kind() {
                CssDeclarationBlockErrorKind::Serialization(value) => Some(value.kind()),
                _ => None,
            },
            RuleCssomSource::SourceUndefined(_) | RuleCssomSource::FormatUnavailable(_) => None,
        };
        let kind = match value {
            Some(
                value @ (CssSpecifiedValueSerializationErrorKind::InputNodeLimit
                | CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
                | CssSpecifiedValueSerializationErrorKind::ByteLimit
                | CssSpecifiedValueSerializationErrorKind::CapacityOverflow),
            ) => CssRuleCssomSerializationErrorKind::Resource(value),
            Some(value) => CssRuleCssomSerializationErrorKind::Value(value),
            None => match &source {
                RuleCssomSource::SourceUndefined(kind) => {
                    CssRuleCssomSerializationErrorKind::SourceUndefined(*kind)
                }
                RuleCssomSource::FormatUnavailable(kind) => {
                    CssRuleCssomSerializationErrorKind::FormatUnavailable(*kind)
                }
                _ => CssRuleCssomSerializationErrorKind::DeclarationBlock,
            },
        };
        Self {
            kind,
            rule_path,
            keyframe_block_index,
            source,
        }
    }
}
impl fmt::Display for CssRuleCssomSerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            CssRuleCssomSerializationErrorKind::Resource(_) => {
                f.write_str("literal CSSOM rule serialization exceeded a resource limit")
            }
            CssRuleCssomSerializationErrorKind::Value(_) => {
                f.write_str("literal CSSOM rule serialization cannot represent a value")
            }
            CssRuleCssomSerializationErrorKind::DeclarationBlock => {
                f.write_str("literal CSSOM rule serialization cannot project a declaration block")
            }
            CssRuleCssomSerializationErrorKind::SourceUndefined(kind) => write!(
                f,
                "selected sources define no literal CSSOM format for {kind:?}"
            ),
            CssRuleCssomSerializationErrorKind::FormatUnavailable(kind) => write!(
                f,
                "no complete literal CSSOM format has been selected for {kind:?}"
            ),
        }
    }
}
impl std::error::Error for CssRuleCssomSerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.source {
            RuleCssomSource::Provider(SpecifiedRuleSerializationSource::Value(value)) => {
                Some(value)
            }
            RuleCssomSource::Provider(SpecifiedRuleSerializationSource::Media(value)) => {
                Some(value.as_ref())
            }
            RuleCssomSource::DeclarationBlock(value) => Some(value),
            RuleCssomSource::SourceUndefined(_) | RuleCssomSource::FormatUnavailable(_) => None,
        }
    }
}
impl CssRule {
    /// Serializes the selected literal CSSOM format with independently normalized declaration runs.
    pub fn serialize_cssom(&self) -> Result<String, CssRuleCssomSerializationError> {
        self.serialize_cssom_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Shares monotonic work and UTF-8 byte budgets across all providers and descendants.
    pub fn serialize_cssom_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssRuleCssomSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.append_cssom_rule_graph(self, Vec::new())?;
        Ok(writer.css)
    }
}
impl CssSheet {
    /// Authored aggregate contract: rule strings in order, one LF separator and no final LF.
    /// This is an owned rule-text operation, independent of a live CSSOM stylesheet.
    pub fn serialize_cssom(&self) -> Result<String, CssRuleCssomSerializationError> {
        self.serialize_cssom_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Returns no partial text on any rule, source, projection, or byte failure.
    pub fn serialize_cssom_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssRuleCssomSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer
            .node()
            .map_err(|value| CssRuleCssomSerializationError::new(value.into(), Vec::new(), None))?;
        for (index, rule) in self.rules().iter().enumerate() {
            let mut path = Vec::new();
            path.try_reserve(1).map_err(|_| {
                CssRuleCssomSerializationError::new(
                    CssSpecifiedValueSerializationError::new(
                        CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    )
                    .into(),
                    Vec::new(),
                    None,
                )
            })?;
            path.push(index);
            if index != 0
                && let Err(value) = writer.append("\n")
            {
                return Err(CssRuleCssomSerializationError::new(
                    value.into(),
                    path,
                    None,
                ));
            }
            writer.append_cssom_rule_graph(rule, path)?;
        }
        Ok(writer.css)
    }
}
