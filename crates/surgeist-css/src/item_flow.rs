//! Authored flow-oriented item placement values from the selected Grid 3 proposal.
//!
//! Contextual axes, `auto`, packing and tolerance evaluation remain downstream.
//! Wrap facets retain authored omission and `wrap-reverse` stays distinct from
//! its computed `wrap reverse` equivalent. The shorthand stores four complete
//! constituents, filling only omitted whole constituents with their initials.

use crate::{
    CssFlowTolerance, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

macro_rules! specified_methods {
    () => {
        /// Emits canonical authored text without evaluating layout context.
        pub fn serialize_specified(&self) -> SerializationResult<String> {
            self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
        }

        /// Emits atomically under cumulative input, projection and byte limits.
        /// Each keyword facet costs one node in each work budget. Composite
        /// carriers add no node; tolerance retains its existing owning costs.
        pub fn serialize_specified_with_limits(
            &self,
            limits: CssSpecifiedValueSerializationLimits,
        ) -> SerializationResult<String> {
            let mut writer = SpecifiedRuleWriter::new(limits);
            self.append_to_rule_writer(&mut writer)?;
            Ok(writer.css)
        }
    };
}

/// Symbolic flow-oriented placement direction, before choosing physical axes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssItemDirection {
    #[default]
    Auto,
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl CssItemDirection {
    specified_methods!();

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::Row => "row",
            Self::Column => "column",
            Self::RowReverse => "row-reverse",
            Self::ColumnReverse => "column-reverse",
        })
    }
}

/// One authored wrapping mode facet; `Auto` remains contextual.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssItemWrapMode {
    Auto,
    NoWrap,
    Wrap,
}

impl CssItemWrapMode {
    const fn keyword(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::NoWrap => "nowrap",
            Self::Wrap => "wrap",
        }
    }
}

/// One authored wrapping order facet, without physical-axis resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssItemWrapOrder {
    Normal,
    Reverse,
}

impl CssItemWrapOrder {
    const fn keyword(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Reverse => "reverse",
        }
    }
}

/// A nonempty authored wrap value preserving which facets were supplied.
///
/// The twelve valid states are three mode-only values, two order-only values,
/// six mode/order pairs and the distinct `wrap-reverse` spelling. No missing
/// internal facet is synthesized. Only an omitted whole shorthand constituent
/// receives this type's initial `Mode(Auto)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssItemWrap {
    Mode(CssItemWrapMode),
    Order(CssItemWrapOrder),
    Both(CssItemWrapMode, CssItemWrapOrder),
    WrapReverse,
}

impl Default for CssItemWrap {
    fn default() -> Self {
        Self::Mode(CssItemWrapMode::Auto)
    }
}

impl CssItemWrap {
    specified_methods!();

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        match self {
            Self::Mode(mode) => writer.keyword(mode.keyword()),
            Self::Order(order) => writer.keyword(order.keyword()),
            Self::Both(mode, order) => {
                writer.keyword(mode.keyword())?;
                writer.append(" ")?;
                writer.keyword(order.keyword())
            }
            Self::WrapReverse => writer.keyword("wrap-reverse"),
        }
    }
}

/// Authored packing strategy; `normal` cannot coexist with dense or balance.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssItemPack {
    #[default]
    Normal,
    Dense,
    Balance,
    DenseBalance,
}

impl CssItemPack {
    specified_methods!();

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        match self {
            Self::Normal => writer.keyword("normal"),
            Self::Dense => writer.keyword("dense"),
            Self::Balance => writer.keyword("balance"),
            Self::DenseBalance => {
                writer.keyword("dense")?;
                writer.append(" ")?;
                writer.keyword("balance")
            }
        }
    }
}

/// Four checked authored `item-flow` constituents in stable grammar order.
///
/// Construction composes already-valid values. It does not evaluate `auto`,
/// compute wrap-reverse, run packing, or resolve the signed tolerance. Canonical
/// output always includes all four constituents, preserving wrap facet presence
/// when the shorthand is parsed again.
#[derive(Clone, Debug, PartialEq)]
pub struct CssItemFlow {
    direction: CssItemDirection,
    wrap: CssItemWrap,
    pack: CssItemPack,
    tolerance: CssFlowTolerance,
}

impl CssItemFlow {
    /// Composes four intrinsically valid constituents without contextual work.
    #[must_use]
    pub const fn new(
        direction: CssItemDirection,
        wrap: CssItemWrap,
        pack: CssItemPack,
        tolerance: CssFlowTolerance,
    ) -> Self {
        Self {
            direction,
            wrap,
            pack,
            tolerance,
        }
    }

    #[must_use]
    pub const fn direction(&self) -> CssItemDirection {
        self.direction
    }

    #[must_use]
    pub const fn wrap(&self) -> CssItemWrap {
        self.wrap
    }

    #[must_use]
    pub const fn pack(&self) -> CssItemPack {
        self.pack
    }

    #[must_use]
    pub const fn tolerance(&self) -> &CssFlowTolerance {
        &self.tolerance
    }

    specified_methods!();

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        self.direction.append_to_rule_writer(writer)?;
        writer.append(" ")?;
        self.wrap.append_to_rule_writer(writer)?;
        writer.append(" ")?;
        self.pack.append_to_rule_writer(writer)?;
        writer.append(" ")?;
        self.tolerance.append_to_rule_writer(writer)
    }
}

impl Default for CssItemFlow {
    fn default() -> Self {
        Self::new(
            CssItemDirection::Auto,
            CssItemWrap::default(),
            CssItemPack::Normal,
            CssFlowTolerance::normal(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CssLengthPercentageCalculation, CssSpecifiedLengthPercentage,
        CssSpecifiedValueSerializationErrorKind as Kind, parse_component_values,
    };
    use CssSpecifiedValueSerializationLimits as Limits;

    fn composed_budget(
        append: impl Fn(&mut SpecifiedRuleWriter) -> SerializationResult<()>,
        expected: &str,
        input: usize,
        projection: usize,
    ) {
        let text = format!("[{expected} {expected}]");
        let compose = |writer: &mut SpecifiedRuleWriter| {
            writer.append("[")?;
            append(writer)?;
            writer.append(" ")?;
            append(writer)?;
            writer.append("]")
        };
        let mut writer =
            SpecifiedRuleWriter::new(Limits::new(input * 2, projection * 2, text.len()));
        compose(&mut writer).unwrap();
        assert_eq!(writer.css, text);
        for (limits, kind) in [
            (
                Limits::new(input * 2 - 1, projection * 2, text.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(input * 2, projection * 2 - 1, text.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(input * 2, projection * 2, text.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(compose(&mut writer).unwrap_err().kind(), kind);
        }

        let mut writer = SpecifiedRuleWriter::new(Limits::new(input, projection, 2));
        writer.append("[").unwrap();
        writer
            .without_output(|writer| {
                writer.without_output(|writer| append(writer))?;
                assert!(writer.context.output_suppressed());
                Ok(())
            })
            .unwrap();
        assert!(!writer.context.output_suppressed());
        writer.append("]").unwrap();
        assert_eq!(writer.css, "[]");
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );

        for (limits, kind) in [
            (Limits::new(input - 1, projection, 1), Kind::InputNodeLimit),
            (
                Limits::new(input, projection - 1, 1),
                Kind::ProjectionNodeLimit,
            ),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            let error = writer
                .without_output(|writer| writer.without_output(|writer| append(writer)))
                .unwrap_err();
            assert_eq!(error.kind(), kind);
            assert!(!writer.context.output_suppressed());
            writer.append("x").unwrap();
            assert_eq!(writer.css, "x");
        }
    }

    #[test]
    fn item_keywords_and_fixed_products_share_cumulative_and_suppressed_work() {
        composed_budget(
            |writer| CssItemDirection::RowReverse.append_to_rule_writer(writer),
            "row-reverse",
            1,
            1,
        );
        let wrap = CssItemWrap::Both(CssItemWrapMode::Wrap, CssItemWrapOrder::Reverse);
        composed_budget(
            |writer| wrap.append_to_rule_writer(writer),
            "wrap reverse",
            2,
            2,
        );
        composed_budget(
            |writer| CssItemPack::DenseBalance.append_to_rule_writer(writer),
            "dense balance",
            2,
            2,
        );
        let flow = CssItemFlow::default();
        composed_budget(
            |writer| flow.append_to_rule_writer(writer),
            "auto auto normal normal",
            4,
            4,
        );
    }

    #[test]
    fn composed_flow_visits_final_numeric_children_during_suppression() {
        let tolerance = CssFlowTolerance::length_percentage(
            CssSpecifiedLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::try_from_components(
                    parse_component_values("calc(1px + 2em)").unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
        );
        let flow = CssItemFlow::new(
            CssItemDirection::Row,
            CssItemWrap::default(),
            CssItemPack::Normal,
            tolerance,
        );
        composed_budget(
            |writer| flow.append_to_rule_writer(writer),
            "row auto normal calc(2em + 1px)",
            7,
            8,
        );
    }
}
