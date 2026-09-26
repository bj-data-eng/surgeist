//! Canonical authored float and clear keywords, before writing-mode mapping.

use crate::specified_serialization::serialize_keyword_sequence;
use crate::{
    CssClear, CssFloat, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

impl CssFloat {
    /// Serializes the specified float keyword without resolving the containing block's writing mode.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical keyword under explicit resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::None => "none",
            Self::Left => "left",
            Self::Right => "right",
            Self::InlineStart => "inline-start",
            Self::InlineEnd => "inline-end",
        };
        serialize_keyword_sequence(text, limits)
    }
}

impl CssClear {
    /// Serializes the specified clear keyword without resolving the containing block's writing mode.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical keyword under explicit resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::None => "none",
            Self::Left => "left",
            Self::Right => "right",
            Self::Both => "both",
            Self::InlineStart => "inline-start",
            Self::InlineEnd => "inline-end",
        };
        serialize_keyword_sequence(text, limits)
    }
}
