//! Bounded canonical serialization of authored quotation-mark systems.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{CssQuotePair, CssQuotePairList, CssQuotes, CssSerializationError};

type SerializationResult<T> = std::result::Result<T, CssSerializationError>;

fn append_pair_contents(
    writer: &mut SpecifiedRuleWriter,
    pair: &CssQuotePair,
) -> SerializationResult<()> {
    writer.string(pair.open().as_str())?;
    writer.append(" ")?;
    writer.string(pair.close().as_str())
}

fn append_pairs(
    writer: &mut SpecifiedRuleWriter,
    pairs: &CssQuotePairList,
) -> SerializationResult<()> {
    // One aggregate for the checked list; its strings are charged as leaves.
    writer.node()?;
    writer.space_separated(pairs.pairs(), append_pair_contents)?;
    Ok(())
}

impl crate::specified_rule_serialization::SpecifiedProvider for CssQuotePair {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.node()?;
        append_pair_contents(writer, self)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssQuotePair);

impl crate::specified_rule_serialization::SpecifiedProvider for CssQuotePairList {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        append_pairs(writer, self)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssQuotePairList);

impl crate::specified_rule_serialization::SpecifiedProvider for CssQuotes {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        match self {
            Self::Auto => writer.keyword("auto")?,
            Self::None => writer.keyword("none")?,
            Self::MatchParent => writer.keyword("match-parent")?,
            Self::Pairs(pairs) => append_pairs(writer, pairs)?,
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssQuotes);
