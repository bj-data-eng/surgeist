//! Authored Scroll Snap values, before writing-mode mapping and snap selection.

use crate::numeric::expression::{CapturedNumericComponent, length_components_equal};
use crate::numeric::{
    CssLength, CssLengthPercentage, CssNonNegative, CssNumeric, CssNumericRoot, CssRange,
};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;
use crate::values::common::CssAutoOr;
use crate::values::common::common_values::{four_sides_entry, side_element, side_pair_entry};

/// The selected scroll-snap axis spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollSnapAxis {
    /// `x`: the scroll container snaps in its horizontal axis only.
    X,
    /// `y`: the scroll container snaps in its vertical axis only.
    Y,
    /// `block`: the scroll container snaps in its block axis only.
    Block,
    /// `inline`: the scroll container snaps in its inline axis only.
    Inline,
    /// `both`: the scroll container snaps in both axes independently.
    Both,
}

impl CssScrollSnapAxis {
    const fn as_css(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Block => "block",
            Self::Inline => "inline",
            Self::Both => "both",
        }
    }
}

/// The optional scroll-snap strictness; omission means proximity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollSnapStrictness {
    /// `mandatory`: the scroll container is required to be snapped to a snap position when no scrolling operation is active and a valid snap position exists.
    Mandatory,
    /// `proximity`: the scroll container may snap to a snap position at the end of a scroll, at the user agent's discretion.
    Proximity,
}

/// The authored scroll-snap-type value.
///
/// # Specified serialization
///
/// Serializes canonical specified keywords without resolving the scroll axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollSnapType {
    /// `none`: the scroll container must not snap.
    None,
    /// An axis, optionally with an authored strictness.
    Axis {
        /// The snap axis.
        axis: CssScrollSnapAxis,
        /// The authored strictness, or `None` when it was omitted (which means `proximity`).
        strictness: Option<CssScrollSnapStrictness>,
    },
}

impl CssScrollSnapType {
    /// Constructs an axis value while retaining whether strictness was authored.
    #[must_use]
    pub const fn axis(
        axis: CssScrollSnapAxis,
        strictness: Option<CssScrollSnapStrictness>,
    ) -> Self {
        Self::Axis { axis, strictness }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssScrollSnapType {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        match self {
            Self::None => {
                context.charge_nodes(1)?;
                context.append(output, "none")?;
            }
            Self::Axis { axis, strictness } => {
                context.charge_nodes(1)?;
                context.append(output, axis.as_css())?;
                if let Some(strictness) = strictness {
                    context.charge_nodes(1)?;
                    if *strictness == CssScrollSnapStrictness::Mandatory {
                        context.append(output, " mandatory")?;
                    }
                }
            }
        }
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssScrollSnapType);

/// One alignment keyword, interpreted separately for block and inline axes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollSnapAlignment {
    /// `none`: the box does not define a snap position on this axis.
    None,
    /// `start`: start alignment of the box's snap area within the snapport is a snap position on this axis.
    Start,
    /// `end`: end alignment of the box's snap area within the snapport is a snap position on this axis.
    End,
    /// `center`: center alignment of the box's snap area within the snapport is a snap position on this axis.
    Center,
}

impl CssScrollSnapAlignment {
    const fn as_css(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Start => "start",
            Self::End => "end",
            Self::Center => "center",
        }
    }
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssScrollSnapAlignment
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(self.as_css())
    }
}

// `scroll-snap-align` takes `[none | start | end | center]{1,2}` and, per Scroll Snap 1 §5.2,
// "if only one value is specified, the second value defaults to the same value": an equal
// second keyword is omitted from the specified serialization, while the model keeps whether it
// was authored.
side_element!(CssScrollSnapAlignment, same = |a, b, _context| Ok(a == b));
side_pair_entry!(CssScrollSnapAlignment);

/// The authored scroll-snap-stop keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollSnapStop {
    /// `normal`: the scroll container may pass over a snap position defined by this element during a scrolling operation.
    Normal,
    /// `always`: the scroll container must not pass over a snap position defined by this element during a scrolling operation.
    Always,
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssScrollSnapStop {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Normal => "normal",
            Self::Always => "always",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssScrollSnapStop);

/// One `scroll-margin` side; the schema names the generic shapes through this private name.
pub(crate) type ScrollMarginSide = CssNumeric<CssLength>;

/// One `scroll-padding` side; the schema names the generic value through this private name.
pub(crate) type ScrollPaddingSide = CssAutoOr<CssNumeric<CssLengthPercentage, CssNonNegative>>;

// `scroll-padding` sides are `auto | <length-percentage [0,∞]>` and `scroll-margin` sides are
// `<length>`. A pair omits an equal second component, as the four-side shorthands omit an
// equal side (CSSOM "serialize a CSS value" step 2). Two sides are equal when their exact
// ordinary literals are equal or, for math, when their canonical projections are.
fn numeric_side_equal<T: CssNumericRoot, R: CssRange<T>>(
    left: &CssNumeric<T, R>,
    right: &CssNumeric<T, R>,
    context: &mut SpecifiedSerializationContext,
) -> Result<bool, CssSerializationError> {
    let capture =
        |value: &CssNumeric<T, R>, context: &mut SpecifiedSerializationContext| match value
            .calculation()
        {
            Some(calculation) => {
                CapturedNumericComponent::capture_calculation(&calculation.expression, context)
            }
            None => value
                .capture_specified(context)
                .map(CapturedNumericComponent::Plain),
        };
    let left_capture = capture(left, context)?;
    let right_capture = capture(right, context)?;
    Ok(length_components_equal(
        left.literal_component(),
        right.literal_component(),
        &left_capture,
        &right_capture,
    ))
}

side_element!(
    ScrollPaddingSide,
    same = |a, b, context| match (a, b) {
        (CssAutoOr::Auto, CssAutoOr::Auto) => Ok(true),
        (CssAutoOr::Value(a), CssAutoOr::Value(b)) => numeric_side_equal(a, b, context),
        _ => Ok(false),
    }
);
side_element!(
    ScrollMarginSide,
    same = |a, b, context| numeric_side_equal(a, b, context)
);
four_sides_entry!(ScrollPaddingSide, ScrollMarginSide);
side_pair_entry!(ScrollPaddingSide, ScrollMarginSide);

#[cfg(test)]
mod cumulative_bridge_contract {
    use super::*;
    use crate::component::{CssComponentValue, CssValueOrigin};
    use crate::numeric::{CssCalculation, CssLength};
    use crate::serialization::CssSerializationError;
    use crate::serialization::CssSerializationLimits as Limits;
    use crate::serialization::specified_rule_serialization::SpecifiedProvider as _;
    use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
    use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind as Kind;
    use crate::values::common::{CssBoxSideKind, CssFourSides, CssSidePair};

    type SerializationResult<T> = Result<T, CssSerializationError>;

    fn length(text: &str) -> CssNumeric<CssLength> {
        CssNumeric::<CssLength>::try_from_component(
            CssComponentValue::try_dimension(text, "px").unwrap(),
        )
        .unwrap()
    }
    fn padding(text: &str) -> CssAutoOr<CssNumeric<CssLengthPercentage, CssNonNegative>> {
        CssAutoOr::Value(
            CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
                CssComponentValue::try_dimension(text, "px").unwrap(),
            )
            .unwrap(),
        )
    }
    fn composition(
        expected: &str,
        nodes: usize,
        append: impl Fn(&mut SpecifiedRuleWriter) -> SerializationResult<()>,
    ) {
        let mut writer = SpecifiedRuleWriter::new(Limits::new(
            2 * nodes + 1,
            2 * nodes + 1,
            2 * expected.len() + 2,
        ));
        writer.context.charge_input(1).unwrap();
        writer.context.charge_projection(1).unwrap();
        writer.append("!").unwrap();
        append(&mut writer).unwrap();
        writer.append(";").unwrap();
        append(&mut writer).unwrap();
        assert_eq!(writer.css, format!("!{expected};{expected}"));
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(writer.append("x").unwrap_err().kind(), Kind::ByteLimit);
        let mut suppressed = SpecifiedRuleWriter::new(Limits::new(nodes, nodes, 0));
        suppressed.without_output(|writer| append(writer)).unwrap();
        assert!(suppressed.css.is_empty());
        assert!(!suppressed.context.output_suppressed());
        assert_eq!(
            suppressed.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            suppressed.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
    }

    #[test]
    fn pairs_share_prefix_siblings_and_authored_child_costs() {
        let margin = CssSidePair::new(length("1"), Some(length("1.0")));
        assert_eq!(
            margin.serialize_specified(Default::default()).unwrap(),
            "1px"
        );
        composition("1px", 3, |writer| margin.append_to_rule_writer(writer));
        let padding = CssSidePair::new(CssAutoOr::Auto, None);
        assert_eq!(
            padding.serialize_specified(Default::default()).unwrap(),
            "auto"
        );
        composition("auto", 2, |writer| padding.append_to_rule_writer(writer));
    }

    #[test]
    fn shorthand_compression_preserves_authored_visits_and_logical_marker() {
        let margin = CssFourSides::try_new(
            CssBoxSideKind::Physical,
            vec![length("1"), length("2"), length("1"), length("2")],
        )
        .unwrap();
        assert_eq!(
            margin.serialize_specified(Default::default()).unwrap(),
            "1px 2px"
        );
        // A physical shorthand charges the aggregate node that a logical one charges for
        // `logical`: one aggregate plus four authored lengths of one node each.
        composition("1px 2px", 5, |writer| margin.append_to_rule_writer(writer));
        let padding = CssFourSides::try_new(
            CssBoxSideKind::Logical,
            vec![padding("1"), padding("1.0"), padding("1")],
        )
        .unwrap();
        assert_eq!(
            padding.serialize_specified(Default::default()).unwrap(),
            "logical 1px"
        );
        composition("logical 1px", 4, |writer| {
            padding.append_to_rule_writer(writer)
        });
    }

    #[test]
    fn unequal_exact_literals_with_equal_rounded_text_stay_separate() {
        let value = CssSidePair::new(length(".12345641"), Some(length(".12345642")));
        composition("0.123456px 0.123456px", 3, |writer| {
            value.append_to_rule_writer(writer)
        });
        assert!(matches!(value.first().literal_component().unwrap().view(),
            crate::component::CssComponentValueRef::Token(crate::component::CssValueTokenRef::Dimension { number, unit })
            if number.representation() == ".12345641" && unit == "px"));
    }

    #[test]
    fn suppressed_symbolic_math_and_large_literals_need_no_remaining_bytes() {
        let components =
            crate::component::values::parse_component_values("calc(10px + 1em)").unwrap();
        let math = CssNumeric::<CssLength>::try_from_calculation(
            CssCalculation::<CssLength>::try_from_components(
                components.clone(),
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap();
        let value = CssSidePair::new(math, Some(length("1e50")));
        let before = value.clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(100, 100, 1));
        writer.append("x").unwrap();
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert_eq!(writer.css, "x");
        assert!(!writer.context.output_suppressed());
        assert_eq!(value, before);
        assert_eq!(
            value.first().calculation().unwrap().components(),
            &components
        );
        let CssValueOrigin::Parsed(origin) = value.first().origin() else {
            panic!("parsed checked math origin");
        };
        assert_eq!(origin.source().as_str(), "calc(10px + 1em)");
        assert_eq!(origin.span().start().byte_offset().value(), 0);
        assert_eq!(origin.span().end().byte_offset().value(), "calc(".len());
    }

    #[test]
    fn nested_suppressed_work_error_restores_enclosing_mode() {
        let value =
            CssFourSides::try_new(CssBoxSideKind::Logical, vec![CssAutoOr::Auto; 4]).unwrap();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(2, 100, 1));
        let error = writer
            .without_output(|writer| {
                let error = writer
                    .without_output(|writer| value.append_to_rule_writer(writer))
                    .unwrap_err();
                assert!(writer.context.output_suppressed());
                Err::<(), _>(error)
            })
            .unwrap_err();
        assert_eq!(error.kind(), Kind::InputNodeLimit);
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
    }
}
