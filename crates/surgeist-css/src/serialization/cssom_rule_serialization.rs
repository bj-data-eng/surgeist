//! Literal CSSOM rule composition and the authored ordered sheet aggregate.
use std::fmt;

use crate::declaration::CssDeclarationBlockError;
use crate::rule::{CssRule, CssSheet};
use crate::serialization::specified_rule_serialization::{
    SpecifiedRuleSerializationSource, SpecifiedRuleWriter,
};
use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind;
use crate::serialization::{CssSerializationError, CssSerializationLimits};

/// A represented rule whose literal format is undefined by the selected sources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRuleCssomKind {
    /// No selected literal CSSOM algorithm defines this adopted authored profile.
    When,
    /// No selected literal CSSOM algorithm defines this adopted authored profile.
    Else,
}

impl CssRuleCssomKind {
    const fn description(self) -> &'static str {
        match self {
            Self::When => "@when rules",
            Self::Else => "@else rules",
        }
    }
}

/// A modern rule with selected subformat providers but no adopted complete literal wrapper.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRuleCssomFormat {
    /// `@counter-style` rules.
    CounterStyle,
    /// `@font-feature-values` rules.
    FontFeatureValues,
    /// `@font-palette-values` rules.
    FontPaletteValues,
    /// `@color-profile` rules.
    ColorProfile,
    /// `@supports` rules in the named supports-condition form.
    SupportsCondition,
    /// `@custom-media` rules.
    CustomMedia,
    /// `@layer` statements.
    LayerStatement,
    /// `@layer` blocks.
    LayerBlock,
    /// `@container` rules.
    Container,
    /// `@scope` rules.
    Scope,
}

impl CssRuleCssomFormat {
    const fn description(self) -> &'static str {
        match self {
            Self::CounterStyle => "@counter-style rules",
            Self::FontFeatureValues => "@font-feature-values rules",
            Self::FontPaletteValues => "@font-palette-values rules",
            Self::ColorProfile => "@color-profile rules",
            Self::SupportsCondition => "@supports-condition rules",
            Self::CustomMedia => "@custom-media rules",
            Self::LayerStatement => "@layer statements",
            Self::LayerBlock => "@layer blocks",
            Self::Container => "@container rules",
            Self::Scope => "@scope rules",
        }
    }
}

/// Why the complete literal rule request failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CssRuleCssomSerializationErrorKind {
    Resource(CssSpecifiedValueSerializationErrorKind),
    Value(CssSpecifiedValueSerializationErrorKind),
    DeclarationBlock,
    SourceUndefined(CssRuleCssomKind),
    FormatUnavailable(CssRuleCssomFormat),
}

#[derive(Clone, Debug, PartialEq)]
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
///
/// The value or declaration-block failure behind it is the error's source and is
/// exposed by its accessor; [`Self::is_resource_limit`] groups the resource refusals.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRuleSerializationError {
    kind: CssRuleCssomSerializationErrorKind,
    rule_path: Vec<usize>,
    keyframe_block_index: Option<usize>,
    source: RuleCssomSource,
}
impl CssRuleSerializationError {
    /// Returns whether a resource limit, including a graph depth limit, refused the rule
    /// or one of its value or declaration providers.
    #[must_use]
    pub const fn is_resource_limit(&self) -> bool {
        matches!(self.kind, CssRuleCssomSerializationErrorKind::Resource(_))
    }
    /// Returns the value serialization failure behind a resource or value refusal,
    /// whether a provider or a declaration block reported it.
    #[must_use]
    pub fn serialization_error(&self) -> Option<&CssSerializationError> {
        match &self.source {
            RuleCssomSource::Provider(value) => Some(value),
            RuleCssomSource::DeclarationBlock(error) => error.serialization(),
            RuleCssomSource::SourceUndefined(_) | RuleCssomSource::FormatUnavailable(_) => None,
        }
    }
    /// Returns the declaration block's failure.
    #[must_use]
    pub const fn declaration_block_error(&self) -> Option<&CssDeclarationBlockError> {
        match &self.source {
            RuleCssomSource::DeclarationBlock(error) => Some(error),
            _ => None,
        }
    }
    /// Returns the represented rule whose literal format the selected sources leave
    /// undefined.
    #[must_use]
    pub const fn undefined_source(&self) -> Option<CssRuleCssomKind> {
        match self.kind {
            CssRuleCssomSerializationErrorKind::SourceUndefined(kind) => Some(kind),
            _ => None,
        }
    }
    /// Returns the rule format that has selected subformats but no complete literal
    /// wrapper.
    #[must_use]
    pub const fn unavailable_format(&self) -> Option<CssRuleCssomFormat> {
        match self.kind {
            CssRuleCssomSerializationErrorKind::FormatUnavailable(format) => Some(format),
            _ => None,
        }
    }
    /// Child indexes from the requested rule, or starting with the sheet's rule index.
    #[must_use]
    pub fn rule_path(&self) -> &[usize] {
        &self.rule_path
    }
    /// Returns the zero-based keyframe block index within the keyframes rule, when the failure
    /// concerns a keyframe block.
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
            RuleCssomSource::Provider(value) => Some(value.kind),
            RuleCssomSource::DeclarationBlock(error) => {
                error.serialization().map(|value| value.kind)
            }
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
                "selected sources define no literal CSSOM format for {}",
                kind.description()
            ),
            CssRuleCssomSerializationErrorKind::FormatUnavailable(kind) => write!(
                f,
                "no complete literal CSSOM format has been selected for {}",
                kind.description()
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
        let namespaces = crate::selector::CssNamespaceContext::from_sheet(self);
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
