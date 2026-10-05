//! Canonical specified values from the selected Writing Modes publications.

use crate::specified_serialization::serialize_keyword_sequence;
use crate::{
    CssDirection, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    CssTextCombineUpright, CssTextOrientation, CssUnicodeBidi, CssWritingMode,
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        };
        writer.keyword(text)
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::Normal => "normal",
            Self::Embed => "embed",
            Self::Isolate => "isolate",
            Self::BidiOverride => "bidi-override",
            Self::IsolateOverride => "isolate-override",
            Self::Plaintext => "plaintext",
        };
        writer.keyword(text)
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::HorizontalTb => "horizontal-tb",
            Self::VerticalRl => "vertical-rl",
            Self::VerticalLr => "vertical-lr",
            Self::SidewaysRl => "sideways-rl",
            Self::SidewaysLr => "sideways-lr",
        };
        writer.keyword(text)
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::Mixed => "mixed",
            Self::Upright => "upright",
            Self::Sideways => "sideways",
        };
        writer.keyword(text)
    }
}

impl CssTextCombineUpright {
    /// Serializes the specified value, preserving omitted counts and deferring
    /// computed integer rounding and range clamping. Finite calculated counts
    /// use the shared six-place specified number text policy.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges one outer input and projection node plus the explicit count's
    /// costs, and includes the prefix in the total output byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        use crate::CssSpecifiedValueSerializationErrorKind as Kind;
        let count = match self {
            Self::None => return serialize_keyword_sequence("none", limits),
            Self::All => return serialize_keyword_sequence("all", limits),
            Self::Digits(None) => return serialize_keyword_sequence("digits", limits),
            Self::Digits(Some(count)) => count,
        };
        let input_nodes = limits
            .max_input_nodes()
            .checked_sub(1)
            .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::InputNodeLimit))?;
        let projection_nodes = limits
            .max_projection_nodes()
            .checked_sub(1)
            .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::ProjectionNodeLimit))?;
        let bytes = limits
            .max_css_bytes()
            .checked_sub("digits ".len())
            .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::ByteLimit))?;
        let child_limits =
            CssSpecifiedValueSerializationLimits::new(input_nodes, projection_nodes, bytes);
        let child = if let Some(calculation) = count.calculation() {
            crate::numeric::project_specified(&calculation.expression, child_limits)?
        } else {
            let text = match count.literal() {
                Some(2) => "2",
                Some(3) => "3",
                Some(4) => "4",
                _ => unreachable!("checked literal count or calculation"),
            };
            serialize_keyword_sequence(text, child_limits)?
        };
        let capacity = "digits "
            .len()
            .checked_add(child.len())
            .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::CapacityOverflow))?;
        let mut result = String::with_capacity(capacity);
        result.push_str("digits ");
        result.push_str(&child);
        Ok(result)
    }
}
