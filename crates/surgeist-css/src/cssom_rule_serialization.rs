//! Literal CSSOM rule composition and the authored ordered sheet aggregate.
use std::fmt;

use crate::{
    CssDeclarationBlockError, CssDeclarationBlockErrorKind, CssRule, CssSerializationError,
    CssSerializationLimits, CssSheet, CssSpecifiedValueSerializationErrorKind,
    specified_rule_serialization::{SpecifiedRuleSerializationSource, SpecifiedRuleWriter},
};

/// A represented rule whose literal format is undefined by the selected sources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRuleCssomKind {
    /// No selected literal CSSOM algorithm defines this adopted authored profile.
    When,
    /// No selected literal CSSOM algorithm defines this adopted authored profile.
    Else,
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
    Provider(CssSerializationError),
    DeclarationBlock(CssDeclarationBlockError),
    SourceUndefined(CssRuleCssomKind),
    FormatUnavailable(CssRuleCssomFormat),
}
impl From<CssSerializationError> for RuleCssomSource {
    fn from(value: CssSerializationError) -> Self {
        Self::Provider(value)
    }
}
impl From<SpecifiedRuleSerializationSource> for RuleCssomSource {
    fn from(value: SpecifiedRuleSerializationSource) -> Self {
        Self::Provider(value.0)
    }
}
impl From<CssDeclarationBlockError> for RuleCssomSource {
    fn from(value: CssDeclarationBlockError) -> Self {
        Self::DeclarationBlock(value)
    }
}

/// Atomic literal serialization failure, retaining its actual graph and provider provenance.
#[derive(Clone, Debug)]
pub struct CssRuleSerializationError {
    kind: CssRuleCssomSerializationErrorKind,
    rule_path: Vec<usize>,
    keyframe_block_index: Option<usize>,
    source: RuleCssomSource,
}
impl CssRuleSerializationError {
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
            RuleCssomSource::Provider(value) => Some(value.kind()),
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
                | CssSpecifiedValueSerializationErrorKind::DepthLimit
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
impl fmt::Display for CssRuleSerializationError {
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
impl std::error::Error for CssRuleSerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.source {
            RuleCssomSource::Provider(value) => Some(value),
            RuleCssomSource::DeclarationBlock(value) => Some(value),
            RuleCssomSource::SourceUndefined(_) | RuleCssomSource::FormatUnavailable(_) => None,
        }
    }
}
impl CssRule {
    /// Serializes the selected literal CSSOM format with independently normalized declaration runs.
    ///
    /// Shares monotonic work and UTF-8 byte budgets across all providers and descendants.
    pub fn serialize_cssom(
        &self,
        limits: CssSerializationLimits,
    ) -> Result<String, CssRuleSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.append_cssom_rule_graph(self, Vec::new(), None)?;
        Ok(writer.css)
    }
}
impl CssSheet {
    /// Authored aggregate contract: rule strings in order, one LF separator and no final LF.
    /// This is an owned rule-text operation, independent of a live CSSOM stylesheet.
    ///
    /// Returns no partial text on any rule, source, projection, or byte failure.
    pub fn serialize_cssom(
        &self,
        limits: CssSerializationLimits,
    ) -> Result<String, CssRuleSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer
            .node()
            .map_err(|value| CssRuleSerializationError::new(value.into(), Vec::new(), None))?;
        let namespaces = crate::CssNamespaceContext::from_sheet(self);
        for (index, rule) in self.rules().iter().enumerate() {
            let mut path = Vec::new();
            path.try_reserve(1).map_err(|_| {
                CssRuleSerializationError::new(
                    CssSerializationError::new(
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
                return Err(CssRuleSerializationError::new(value.into(), path, None));
            }
            writer.append_cssom_rule_graph(rule, path, Some(&namespaces))?;
        }
        Ok(writer.css)
    }
}
