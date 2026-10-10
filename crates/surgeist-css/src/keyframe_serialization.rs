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
        let selectors = (|| {
            writer.node()?;
            writer.keyframe_selectors(self.selectors)?;
            writer.append(" { ")
        })();
        selectors.map_err(CssKeyframeRuleViewError::Serialization)?;
        self.declarations
            .append_cssom(&mut writer)
            .map_err(CssKeyframeRuleViewError::Declaration)?;
        if !self.declarations.entries().is_empty() {
            writer
                .append(" ")
                .map_err(CssKeyframeRuleViewError::Serialization)?;
        }
        writer
            .append("}")
            .map_err(CssKeyframeRuleViewError::Serialization)?;
        Ok(writer.css)
    }
}
