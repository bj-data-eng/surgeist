//! Bounded canonical serialization of authored quotation-mark systems.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssQuotePair, CssQuotePairList, CssQuotes, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

type SerializationResult<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn append_string(writer: &mut SpecifiedRuleWriter, value: &str) -> SerializationResult<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    writer.append_string(value)
}

fn append_pair_contents(
    writer: &mut SpecifiedRuleWriter,
    pair: &CssQuotePair,
) -> SerializationResult<()> {
    append_string(writer, pair.open().as_str())?;
    writer.append(" ")?;
    append_string(writer, pair.close().as_str())
}

fn append_pairs(
    writer: &mut SpecifiedRuleWriter,
    pairs: &CssQuotePairList,
) -> SerializationResult<()> {
    // One aggregate for the checked list; its strings are charged as leaves.
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    for (index, pair) in pairs.pairs().iter().enumerate() {
        if index != 0 {
            writer.append(" ")?;
        }
        append_pair_contents(writer, pair)?;
    }
    Ok(())
}

fn append_keyword(writer: &mut SpecifiedRuleWriter, keyword: &str) -> SerializationResult<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    writer.append(keyword)
}

impl CssQuotePair {
    /// Serializes one checked opening/closing pair with canonical string escapes.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Applies one cumulative node and byte budget to the pair.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        append_pair_contents(writer, self)?;
        Ok(())
    }
}

impl CssQuotePairList {
    /// Serializes the nonempty list in authored pair order.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Applies one cumulative node and byte budget across all pairs.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        append_pairs(writer, self)?;
        Ok(())
    }
}

impl CssQuotes {
    /// Serializes the symbolic keyword or checked pair list without resolving language.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Applies one cumulative node and byte budget to the complete value.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        match self {
            Self::Auto => append_keyword(writer, "auto")?,
            Self::None => append_keyword(writer, "none")?,
            Self::MatchParent => append_keyword(writer, "match-parent")?,
            Self::Pairs(pairs) => append_pairs(writer, pairs)?,
        }
        Ok(())
    }
}
