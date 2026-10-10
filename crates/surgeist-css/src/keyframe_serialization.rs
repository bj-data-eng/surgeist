//! Detached keyText and checked keyframe assembly under one cumulative budget.

use std::fmt;

use crate::{
    CssDeclarationBlockError, CssKeyframePercent, CssKeyframeSelectorList,
    CssSpecifiedDeclarationBlock, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, specified_rule_serialization::SpecifiedRuleWriter,
};

impl CssKeyframeSelectorList {
    /// Iterates the ordered endpoint-normalized offsets without numeric rounding.
    ///
    /// `from` and `to` map to literal 0 and 100 percent. Duplicates remain;
    /// symbolic math is cloned without evaluation. Comparing complete iterator
    /// contents preserves count and order for consumer-owned matching.
    pub fn offsets(&self) -> impl ExactSizeIterator<Item = CssKeyframePercent> + '_ {
        self.selectors()
            .iter()
            .map(crate::CssKeyframeSelector::offset)
    }

    /// Formats keyText, normalizing endpoints and preserving ordered duplicates.
    ///
    /// Shared percentage formatting rounds only output; authored offsets remain
    /// unchanged. Symbolic percentages use the existing percentage writer.
    pub fn serialize_key_text(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_key_text_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Formats one complete keyText under explicit aggregate resource limits.
    ///
    /// The selector-list aggregate and each selector cost one input and one
    /// projection node. Symbolic percentage providers share this budget. Errors
    /// return no partial text and neither modify selectors nor affect retry.
    pub fn serialize_key_text_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.keyframe_selectors(self)?;
        Ok(writer.css)
    }
}

impl SpecifiedRuleWriter {
    pub(crate) fn keyframe_selectors(
        &mut self,
        selectors: &CssKeyframeSelectorList,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        self.node()?;
        for (index, selector) in selectors.selectors().iter().enumerate() {
            if index != 0 {
                self.append(", ")?;
            }
            self.keyframe_selector(selector)?;
        }
        Ok(())
    }
}

/// A checked borrowed detached rule in the specified formatting phase.
///
/// Selectors retain authored meaning; declarations have already crossed the
/// owning Keyframe-domain terminal constructor. Programmatic inputs need no
/// fabricated source positions. This view borrows immutable inputs and assigns
/// no live identity, parent links, revisions, search or mutation behavior.
#[derive(Clone, Copy, Debug)]
pub struct CssKeyframeRuleView<'a> {
    selectors: &'a CssKeyframeSelectorList,
    declarations: &'a CssSpecifiedDeclarationBlock,
}

/// Why a checked detached keyframe could not be assembled or formatted.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssKeyframeRuleViewError {
    /// Declarations from another domain carry no proof of keyframe admission.
    NonKeyframeDeclarations,
    /// Selector or punctuation serialization exhausted its resource budget.
    Serialization(CssSpecifiedValueSerializationError),
    /// The owning declaration formatter rejected the complete block.
    Declaration(CssDeclarationBlockError),
}

impl fmt::Display for CssKeyframeRuleViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonKeyframeDeclarations => formatter
                .write_str("keyframe assembly requires a checked keyframe declaration block"),
            Self::Serialization(error) => error.fmt(formatter),
            Self::Declaration(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for CssKeyframeRuleViewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NonKeyframeDeclarations => None,
            Self::Serialization(error) => Some(error),
            Self::Declaration(error) => Some(error),
        }
    }
}

impl<'a> CssKeyframeRuleView<'a> {
    /// Couples selectors with a checked Keyframe-domain specified block.
    ///
    /// A block from another domain is rejected even if its current entries happen
    /// to be admissible. Use the owning keyframe constructor to establish proof.
    pub fn try_new(
        selectors: &'a CssKeyframeSelectorList,
        declarations: &'a CssSpecifiedDeclarationBlock,
    ) -> Result<Self, CssKeyframeRuleViewError> {
        if !declarations.is_keyframe() {
            return Err(CssKeyframeRuleViewError::NonKeyframeDeclarations);
        }
        Ok(Self {
            selectors,
            declarations,
        })
    }

    #[must_use]
    pub const fn selectors(&self) -> &'a CssKeyframeSelectorList {
        self.selectors
    }

    #[must_use]
    pub const fn declarations(&self) -> &'a CssSpecifiedDeclarationBlock {
        self.declarations
    }

    /// Formats a complete detached CSSOM keyframe with the normal declaration owner.
    pub fn serialize_cssom(&self) -> Result<String, CssKeyframeRuleViewError> {
        self.serialize_cssom_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Formats selector, braces and declarations under one cumulative budget.
    ///
    /// The rule costs one node, followed by the keyText and declaration providers'
    /// published work. Errors return no partial output, preserve original source
    /// observations, and permit unchanged retry. No semantic reparse is performed.
    pub fn serialize_cssom_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssKeyframeRuleViewError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer
            .append_selected_keyframe(*self)
            .map_err(|error| match error {
                crate::cssom_rule_serialization::RuleCssomSource::DeclarationBlock(error) => {
                    CssKeyframeRuleViewError::Declaration(error)
                }
                crate::cssom_rule_serialization::RuleCssomSource::Provider(
                    crate::specified_rule_serialization::SpecifiedRuleSerializationSource::Value(
                        error,
                    ),
                ) => CssKeyframeRuleViewError::Serialization(error),
                _ => unreachable!("selected keyframe has only value and declaration providers"),
            })?;
        Ok(writer.css)
    }
}

/// Borrowed current whole-keyframes payload, without live identity or authored coordinates.
///
/// The checked name and every checked Keyframe-domain child are borrowed directly.
/// Children retain supplied order, duplicates and empty blocks. Appending, deleting,
/// matching and revisions belong to the consumer; no authored envelope is rebuilt.
#[derive(Clone, Copy, Debug)]
pub struct CssKeyframesRuleView<'a> {
    name: &'a crate::CssKeyframesName,
    rules: &'a [CssKeyframeRuleView<'a>],
}
impl<'a> CssKeyframesRuleView<'a> {
    pub fn try_new(
        name: &'a crate::CssKeyframesName,
        rules: &'a [CssKeyframeRuleView<'a>],
    ) -> Result<Self, crate::CssRuleCssomSerializationError> {
        Self::try_new_with_limits(name, rules, CssSpecifiedValueSerializationLimits::default())
    }
    /// Checks the complete current payload with one cumulative formatting budget.
    pub fn try_new_with_limits(
        name: &'a crate::CssKeyframesName,
        rules: &'a [CssKeyframeRuleView<'a>],
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Self, crate::CssRuleCssomSerializationError> {
        let view = Self { name, rules };
        view.serialize_cssom_with_limits(limits)?;
        Ok(view)
    }
    #[must_use]
    pub const fn name(self) -> &'a crate::CssKeyframesName {
        self.name
    }
    #[must_use]
    pub const fn rules(self) -> &'a [CssKeyframeRuleView<'a>] {
        self.rules
    }
    pub fn serialize_cssom(self) -> Result<String, crate::CssRuleCssomSerializationError> {
        self.serialize_cssom_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Failure publishes no partial text; borrowed payloads and retry are unchanged.
    pub fn serialize_cssom_with_limits(
        self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, crate::CssRuleCssomSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        let mut index = None;
        writer
            .node()
            .map_err(crate::cssom_rule_serialization::RuleCssomSource::from)
            .and_then(|()| writer.append_selected_keyframes(self, &mut index))
            .map_err(|error| {
                crate::CssRuleCssomSerializationError::new(error, Vec::new(), index)
            })?;
        Ok(writer.css)
    }
}
impl SpecifiedRuleWriter {
    pub(crate) fn append_selected_keyframe(
        &mut self,
        view: CssKeyframeRuleView<'_>,
    ) -> Result<(), crate::cssom_rule_serialization::RuleCssomSource> {
        self.keyframe_cssom_payload(
            view.selectors,
            !view.declarations.entries().is_empty(),
            |writer| view.declarations.append_cssom(writer).map_err(Into::into),
        )
    }
    /// The caller owns the logical rule charge; children use the existing leaf owner.
    pub(crate) fn append_selected_keyframes(
        &mut self,
        view: CssKeyframesRuleView<'_>,
        index: &mut Option<usize>,
    ) -> Result<(), crate::cssom_rule_serialization::RuleCssomSource> {
        self.keyframes_cssom_payload(view.name, view.rules.len(), index, |writer, index| {
            writer.append_selected_keyframe(view.rules[index])
        })
    }
}
