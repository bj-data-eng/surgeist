//! Canonical specified keywords from the selected Writing Modes publications.

use crate::specified_serialization::serialize_keyword_sequence;
use crate::{
    CssDirection, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    CssTextOrientation, CssUnicodeBidi, CssWritingMode,
};

impl CssDirection {
    /// Serializes the canonical specified direction keyword without
    /// applying inheritance or contextual text and layout behavior.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically, charging one input node, one projection node,
    /// and the exact output byte count. Authored input is never modified.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        };
        serialize_keyword_sequence(text, limits)
    }
}

impl CssUnicodeBidi {
    /// Serializes the canonical specified unicode-bidi keyword without
    /// applying inheritance or contextual text and layout behavior.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically, charging one input node, one projection node,
    /// and the exact output byte count. Authored input is never modified.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::Normal => "normal",
            Self::Embed => "embed",
            Self::Isolate => "isolate",
            Self::BidiOverride => "bidi-override",
            Self::IsolateOverride => "isolate-override",
            Self::Plaintext => "plaintext",
        };
        serialize_keyword_sequence(text, limits)
    }
}

impl CssWritingMode {
    /// Serializes the canonical specified writing-mode keyword without
    /// applying inheritance or contextual text and layout behavior.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically, charging one input node, one projection node,
    /// and the exact output byte count. Authored input is never modified.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::HorizontalTb => "horizontal-tb",
            Self::VerticalRl => "vertical-rl",
            Self::VerticalLr => "vertical-lr",
            Self::SidewaysRl => "sideways-rl",
            Self::SidewaysLr => "sideways-lr",
        };
        serialize_keyword_sequence(text, limits)
    }
}

impl CssTextOrientation {
    /// Serializes the canonical specified text-orientation keyword without
    /// applying inheritance or contextual text and layout behavior.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically, charging one input node, one projection node,
    /// and the exact output byte count. Authored input is never modified.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::Mixed => "mixed",
            Self::Upright => "upright",
            Self::Sideways => "sideways",
        };
        serialize_keyword_sequence(text, limits)
    }
}
