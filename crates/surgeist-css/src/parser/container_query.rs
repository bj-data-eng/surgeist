use cssparser::{ParseError, Parser};

use super::boolean_condition::{
    BooleanModel, BooleanNode, BooleanNodeKind, ComponentOperands, ConditionFailure,
    first_significant, parse_component_condition,
};
use super::query_components::{
    ComponentCursor as ContainerCursor, RangePrefix, component_query_comparisons,
    media_literal_number, negative_literal, query_plain_range, query_range_chain,
};
use super::recovery::RecoveryState;
use crate::error::{Error, invalid_syntax};
use crate::media_features::MediaRangeState;
use crate::numeric::CalculationRoot;
use crate::supports::SupportsLexical;
use crate::syntax::*;
use crate::{
    CssComponentValue, CssComponentValueLimits, CssComponentValueRef, CssComponentValues,
    CssValueOrigin, CssValueTokenRef,
};

/// The parsed and checked entry points share this immutable component grammar.
pub(crate) fn container_condition_from_enclosed(
    enclosed: CssGeneralEnclosed,
) -> Result<CssContainerCondition, CssContainerConstructionError> {
    let values =
        CssComponentValues::try_new(vec![enclosed.component().clone()], Default::default())?;
    construct_container_condition(values, CssComponentValueLimits::default())
}

pub(crate) fn construct_container_condition(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<CssContainerCondition, CssContainerConstructionError> {
    values.validate_with_limits(limits)?;
    let node = container_condition(values.items())?.map_err(|index| {
        CssContainerConstructionError::InvalidConditionGrammar {
            origin: values
                .items()
                .get(index)
                .or_else(|| values.items().last())
                .map_or(CssValueOrigin::Programmatic, |value| value.origin().clone()),
        }
    })?;
    Ok(node.into_condition(SupportsLexical::root(values)))
}

// Probes own only semantic leaves and relative sibling regions. The selected
// tree shares one lexical root, including opaque leaves and redundant groups.
type ContainerNode = BooleanNode<ContainerLeaf>;

enum ContainerLeaf {
    Opaque,
    Feature(CssContainerFeatureQuery),
    Style(super::container_style::StyleNode),
    Scroll(super::container_scroll::ScrollNode),
}

struct ContainerModel;

impl BooleanModel<ContainerLeaf> for ContainerModel {
    type Condition = CssContainerCondition;
    type Kind = CssContainerConditionKind;

    fn leaf(leaf: ContainerLeaf, lexical: &SupportsLexical) -> Self::Kind {
        match leaf {
            ContainerLeaf::Opaque => CssContainerConditionKind::GeneralEnclosed(
                CssContainerGeneralEnclosed::new(lexical.clone()),
            ),
            ContainerLeaf::Feature(feature) => CssContainerConditionKind::Feature(feature),
            ContainerLeaf::Style(style) => CssContainerConditionKind::Style(
                style.into_query(lexical.children(first_significant(lexical))),
            ),
            ContainerLeaf::Scroll(scroll) => CssContainerConditionKind::ScrollState(
                scroll.into_query(lexical.children(first_significant(lexical))),
            ),
        }
    }
    fn parenthesized(group: Box<Self::Condition>) -> Self::Kind {
        CssContainerConditionKind::Parenthesized(group)
    }
    fn not(operand: Box<Self::Condition>) -> Self::Kind {
        CssContainerConditionKind::Not(operand)
    }
    fn and(operands: Vec<Self::Condition>) -> Self::Kind {
        CssContainerConditionKind::And(CssContainerConditionList::new(operands))
    }
    fn or(operands: Vec<Self::Condition>) -> Self::Kind {
        CssContainerConditionKind::Or(CssContainerConditionList::new(operands))
    }
    fn condition(kind: Self::Kind, lexical: SupportsLexical) -> Self::Condition {
        CssContainerCondition::new(kind, lexical)
    }
}

impl ContainerNode {
    fn into_condition(self, parent: SupportsLexical) -> CssContainerCondition {
        self.build::<ContainerModel>(&parent)
    }
}

fn container_condition(
    items: &[CssComponentValue],
) -> Result<Result<ContainerNode, usize>, crate::CssComponentValueError> {
    match parse_component_condition(&mut ContainerOperands, items) {
        Ok(node) => Ok(Ok(node)),
        Err(ConditionFailure::Mismatch(index)) => Ok(Err(index)),
        Err(ConditionFailure::Fatal(error)) => Err(error),
    }
}

// Contain 3 `<query-in-parens>`: a parenthesized condition or size feature,
// a `style()` or `scroll-state()` query, or any other enclosure.
struct ContainerOperands;

impl<'a> ComponentOperands<'a> for ContainerOperands {
    type Leaf = ContainerLeaf;
    type Error = crate::CssComponentValueError;

    fn operand(
        &mut self,
        cursor: &mut ContainerCursor<'a>,
    ) -> Result<ContainerNode, ConditionFailure<Self::Error>> {
        cursor.peek();
        let index = cursor.index;
        let Some(component) = cursor.next() else {
            return Err(ConditionFailure::Mismatch(index));
        };
        let enclosed = match component.view() {
            CssComponentValueRef::Function(_) => true,
            CssComponentValueRef::Block(block) => block.kind() == crate::CssBlockKind::Parenthesis,
            _ => false,
        };
        if !enclosed {
            return Err(ConditionFailure::Mismatch(index));
        }
        Ok(ContainerNode {
            kind: container_atom(component)?,
            range: index..index + 1,
        })
    }
}

fn container_atom(
    component: &CssComponentValue,
) -> Result<BooleanNodeKind<ContainerLeaf>, crate::CssComponentValueError> {
    match component.view() {
        CssComponentValueRef::Function(function)
            if function.name().eq_ignore_ascii_case("style") =>
        {
            if let Some(style) = super::container_style::parse(function.values().items()) {
                return Ok(BooleanNodeKind::Leaf(ContainerLeaf::Style(style)));
            }
        }
        CssComponentValueRef::Function(function)
            if function.name().eq_ignore_ascii_case("scroll-state") =>
        {
            if let Some(scroll) = super::container_scroll::parse(function.values().items())? {
                return Ok(BooleanNodeKind::Leaf(ContainerLeaf::Scroll(scroll)));
            }
        }
        CssComponentValueRef::Block(block) if block.kind() == crate::CssBlockKind::Parenthesis => {
            if let Ok(condition) = container_condition(block.values().items())? {
                return Ok(BooleanNodeKind::Parenthesized(Box::new(condition)));
            }
            if let Some(feature) = container_feature(block.values().items())? {
                return Ok(BooleanNodeKind::Leaf(ContainerLeaf::Feature(feature)));
            }
        }
        _ => {}
    }
    Ok(BooleanNodeKind::Leaf(ContainerLeaf::Opaque))
}
fn container_trivia(value: &CssComponentValue) -> bool {
    matches!(
        value.view(),
        CssComponentValueRef::Comment(_)
            | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
    )
}

// Intrinsic grammar failure tries general-enclosed; resource failures do not.
enum ContainerFeatureError {
    Grammar,
    Component(crate::CssComponentValueError),
}
impl From<crate::CssComponentValueError> for ContainerFeatureError {
    fn from(value: crate::CssComponentValueError) -> Self {
        Self::Component(value)
    }
}
type ContainerFeatureResult<T> = Result<T, ContainerFeatureError>;

fn container_feature(
    items: &[CssComponentValue],
) -> Result<Option<CssContainerFeatureQuery>, crate::CssComponentValueError> {
    match parse_container_size(items) {
        Ok(feature) => Ok(Some(feature)),
        Err(ContainerFeatureError::Grammar) => Ok(None),
        Err(ContainerFeatureError::Component(error)) => Err(error),
    }
}
fn significant(items: &[CssComponentValue]) -> &[CssComponentValue] {
    let start = items
        .iter()
        .position(|item| !container_trivia(item))
        .unwrap_or(items.len());
    let end = items
        .iter()
        .rposition(|item| !container_trivia(item))
        .map_or(start, |i| i + 1);
    &items[start..end]
}
fn container_name(items: &[CssComponentValue]) -> Option<ContainerFeatureName> {
    let [item] = significant(items) else {
        return None;
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) = item.view() else {
        return None;
    };
    ContainerFeatureName::parse(name)
}
// Neutral sibling token scanning shared by size and style ranges. Nested
// operators remain inside their operands; only adjacent '=' (comments allowed)
// forms an inclusive comparator. A range has at most two comparisons.
fn parse_container_size(
    items: &[CssComponentValue],
) -> ContainerFeatureResult<CssContainerFeatureQuery> {
    let items = significant(items);
    if let Some(name) = container_name(items) {
        if name.prefix().is_some() {
            return Err(ContainerFeatureError::Grammar);
        }
        return Ok(CssContainerFeatureQuery::Boolean(name.kind()));
    }
    let comparisons = component_query_comparisons(items).ok_or(ContainerFeatureError::Grammar)?;
    if comparisons.is_empty() {
        let mut cursor = ContainerCursor::new(items);
        let CssValueTokenRef::Ident(ident) =
            cursor.token().ok_or(ContainerFeatureError::Grammar)?
        else {
            return Err(ContainerFeatureError::Grammar);
        };
        let name = ContainerFeatureName::parse(ident).ok_or(ContainerFeatureError::Grammar)?;
        if !matches!(cursor.token(), Some(CssValueTokenRef::Colon)) {
            return Err(ContainerFeatureError::Grammar);
        }
        let operand = &items[cursor.index..];
        if name.kind() == CssContainerSizeFeatureKind::Orientation {
            return container_orientation(operand).map(CssContainerFeatureQuery::Orientation);
        }
        return container_range(name, MediaRangeState::Plain { value: operand });
    }
    let (name, shape) = match comparisons.as_slice() {
        [(operator, comparison)] => {
            let left = &items[..operator.start];
            let right = &items[operator.end..];
            if let Some(name) = container_name(left) {
                (
                    name,
                    MediaRangeState::FeatureFirst {
                        comparison: *comparison,
                        value: right,
                    },
                )
            } else {
                let name = container_name(right).ok_or(ContainerFeatureError::Grammar)?;
                (
                    name,
                    MediaRangeState::ValueFirst {
                        comparison: *comparison,
                        value: left,
                    },
                )
            }
        }
        [(first, first_comparison), (second, second_comparison)] => {
            let name = container_name(&items[first.end..second.start])
                .ok_or(ContainerFeatureError::Grammar)?;
            let shape = query_range_chain(
                &items[..first.start],
                *first_comparison,
                &items[second.end..],
                *second_comparison,
            )
            .ok_or(ContainerFeatureError::Grammar)?;
            (name, shape)
        }
        _ => return Err(ContainerFeatureError::Grammar),
    };
    if name.prefix().is_some() || name.kind() == CssContainerSizeFeatureKind::Orientation {
        return Err(ContainerFeatureError::Grammar);
    }
    container_range(name, shape)
}
fn container_range(
    name: ContainerFeatureName,
    shape: MediaRangeState<&[CssComponentValue]>,
) -> ContainerFeatureResult<CssContainerFeatureQuery> {
    fn map<T>(
        shape: MediaRangeState<&[CssComponentValue]>,
        prefix: Option<RangePrefix>,
        value: impl Fn(&[CssComponentValue]) -> ContainerFeatureResult<T>,
    ) -> ContainerFeatureResult<CssMediaRange<T>> {
        Ok(CssMediaRange::new(match shape {
            MediaRangeState::Plain { value: input } => query_plain_range(value(input)?, prefix),
            MediaRangeState::FeatureFirst {
                comparison,
                value: input,
            } => MediaRangeState::FeatureFirst {
                comparison,
                value: value(input)?,
            },
            MediaRangeState::ValueFirst {
                comparison,
                value: input,
            } => MediaRangeState::ValueFirst {
                comparison,
                value: value(input)?,
            },
            MediaRangeState::Ascending {
                left,
                left_inclusive,
                right,
                right_inclusive,
            } => MediaRangeState::Ascending {
                left: value(left)?,
                left_inclusive,
                right: value(right)?,
                right_inclusive,
            },
            MediaRangeState::Descending {
                left,
                left_inclusive,
                right,
                right_inclusive,
            } => MediaRangeState::Descending {
                left: value(left)?,
                left_inclusive,
                right: value(right)?,
                right_inclusive,
            },
            MediaRangeState::Min { .. } | MediaRangeState::Max { .. } => {
                unreachable!("prefix applied only after operand admission")
            }
        }))
    }
    Ok(match name.kind() {
        CssContainerSizeFeatureKind::Width => {
            CssContainerFeatureQuery::Width(map(shape, name.prefix(), container_length)?)
        }
        CssContainerSizeFeatureKind::Height => {
            CssContainerFeatureQuery::Height(map(shape, name.prefix(), container_length)?)
        }
        CssContainerSizeFeatureKind::InlineSize => {
            CssContainerFeatureQuery::InlineSize(map(shape, name.prefix(), container_length)?)
        }
        CssContainerSizeFeatureKind::BlockSize => {
            CssContainerFeatureQuery::BlockSize(map(shape, name.prefix(), container_length)?)
        }
        CssContainerSizeFeatureKind::AspectRatio => {
            CssContainerFeatureQuery::AspectRatio(map(shape, name.prefix(), container_ratio)?)
        }
        CssContainerSizeFeatureKind::Orientation => return Err(ContainerFeatureError::Grammar),
    })
}
fn container_operand(
    items: &[CssComponentValue],
) -> ContainerFeatureResult<(CssComponentValues, bool)> {
    super::query_components::authored_operand(items)?.ok_or(ContainerFeatureError::Grammar)
}
fn container_numeric(
    values: CssComponentValues,
    root: CalculationRoot,
) -> ContainerFeatureResult<CssCalculationExpression> {
    // The component boundary already checked construction. Only parser-owned
    // recovered closures can reach this helper, so preserve their provenance.
    crate::numeric::construct_container(values, root, CssComponentValueLimits::default()).map_err(
        |error| {
            error.component_error().cloned().map_or(
                ContainerFeatureError::Grammar,
                ContainerFeatureError::Component,
            )
        },
    )
}
fn container_length(items: &[CssComponentValue]) -> ContainerFeatureResult<CssContainerLength> {
    let (values, pending) = container_operand(items)?;
    if pending {
        return Ok(CssContainerLength::pending(CssContainerPendingValue::new(
            CssContainerValueDomain::Length,
            values,
        )));
    }
    let value = CssLengthCalculation::from_expression(container_numeric(
        values.clone(),
        CalculationRoot::Length,
    )?);
    Ok(CssContainerLength::typed(value, values))
}
fn container_ratio(items: &[CssComponentValue]) -> ContainerFeatureResult<CssContainerRatio> {
    let (values, pending) = container_operand(items)?;
    if pending {
        return Ok(CssContainerRatio::pending(CssContainerPendingValue::new(
            CssContainerValueDomain::Ratio,
            values,
        )));
    }
    let operand = |items: &[CssComponentValue]| -> ContainerFeatureResult<CssNumberCalculation> {
        let values = CssComponentValues::try_new(items.to_vec(), Default::default())?;
        if media_literal_number(&values).is_some_and(negative_literal) {
            return Err(ContainerFeatureError::Grammar);
        }
        Ok(CssNumberCalculation::from_expression(container_numeric(
            values,
            CalculationRoot::Number,
        )?))
    };
    let mut split = values.items().split(|item| {
        matches!(
            item.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Delim('/'))
        )
    });
    let numerator = operand(split.next().expect("one ratio region"))?;
    let denominator_items = split.next();
    if split.next().is_some() {
        return Err(ContainerFeatureError::Grammar);
    }
    let denominator = if let Some(items) = denominator_items {
        operand(items)?
    } else {
        CssNumberCalculation::try_from_components(
            CssComponentValues::try_new(
                vec![CssComponentValue::try_number("1")?],
                Default::default(),
            )?,
            Default::default(),
        )
        .expect("implicit ratio denominator is one")
    };
    Ok(CssContainerRatio::typed(
        CssMediaRatio::new(numerator, denominator, denominator_items.is_none()),
        values,
    ))
}
fn container_orientation(
    items: &[CssComponentValue],
) -> ContainerFeatureResult<CssContainerOrientation> {
    let (values, pending) = container_operand(items)?;
    if pending {
        return Ok(CssContainerOrientation::pending(
            CssContainerPendingValue::new(CssContainerValueDomain::Orientation, values),
        ));
    }
    let [value] = values.items() else {
        return Err(ContainerFeatureError::Grammar);
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Ident(value)) = value.view() else {
        return Err(ContainerFeatureError::Grammar);
    };
    let value = if value.eq_ignore_ascii_case("portrait") {
        CssOrientation::Portrait
    } else if value.eq_ignore_ascii_case("landscape") {
        CssOrientation::Landscape
    } else {
        return Err(ContainerFeatureError::Grammar);
    };
    Ok(CssContainerOrientation::typed(value, values))
}

/// Collect and validate the complete original prelude once, after the enclosing
/// production's shared depth check, before any semantic classification.
pub(super) fn collect_container_components<'i>(
    source: &str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<(CssComponentValues, Vec<usize>), ParseError<'i, Error>> {
    let implicit =
        recovery.check_specialized_components(source, input, "baseline.rule.container")?;
    let values = CssComponentValues::collect_from_parser(input, recovery.source_snapshot())
        .map_err(|error| {
            crate::error::invalid_component_value(input.current_source_location(), error)
        })?;
    Ok((values, implicit))
}
// Parser collection already enforces the stylesheet's complete component limits.
// Checked construction performs its selected limits before entering this same grammar.
pub(crate) fn construct_container_prelude(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<CssContainerPrelude, CssContainerConstructionError> {
    values.validate_with_limits(limits)?;
    admit_container_prelude(values).map_err(|error| match error {
        ContainerPreludeAdmissionError::Component(error) => error.into(),
        ContainerPreludeAdmissionError::Grammar { origin, .. } => {
            CssContainerConstructionError::InvalidPreludeGrammar { origin }
        }
    })
}

enum ContainerPreludeAdmissionError {
    Component(crate::CssComponentValueError),
    Grammar {
        index: usize,
        origin: CssValueOrigin,
    },
}

fn admit_container_prelude(
    values: CssComponentValues,
) -> Result<CssContainerPrelude, ContainerPreludeAdmissionError> {
    let lexical = SupportsLexical::root(values);
    let items = lexical.items();
    let mut entries = Vec::new();
    let mut start = 0;
    // Only sibling comma tokens delimit entries. Function/block contents and
    // escaped commas in decoded identifier tokens remain inside their component.
    for end in (0..items.len())
        .filter(|&index| {
            matches!(
                items[index].view(),
                CssComponentValueRef::Token(CssValueTokenRef::Comma)
            )
        })
        .chain(std::iter::once(items.len()))
    {
        let region = lexical.select(start..end);
        let mut cursor = ContainerCursor::new(region.items());
        let name = match cursor.peek().map(CssComponentValue::view) {
            Some(CssComponentValueRef::Token(CssValueTokenRef::Ident(name))) => {
                CssContainerName::from_decoded(name.to_owned())
            }
            _ => None,
        };
        if name.is_some() {
            cursor.next();
        }
        let has_query = cursor.peek().is_some();
        let invalid = |index: usize| ContainerPreludeAdmissionError::Grammar {
            index,
            origin: items
                .get(index)
                .or_else(|| items.last())
                .map_or(CssValueOrigin::Programmatic, |value| value.origin().clone()),
        };
        if !has_query {
            let name = name.ok_or_else(|| invalid(start + cursor.index))?;
            entries.push(CssContainerQueryEntry::name_only(name, region));
        } else {
            let query_start = cursor.index;
            let node = container_condition(&region.items()[query_start..])
                .map_err(ContainerPreludeAdmissionError::Component)?
                .map_err(|index| invalid(start + query_start + index))?;
            let query = node.into_condition(region.select(query_start..region.items().len()));
            entries.push(CssContainerQueryEntry::with_query(name, query, region));
        }
        start = end + 1;
    }
    Ok(CssContainerPrelude::new(entries, lexical))
}

pub(super) fn container_prelude_from_components<'i>(
    values: CssComponentValues,
    location: cssparser::SourceLocation,
) -> Result<CssContainerPrelude, ParseError<'i, Error>> {
    let end = values.items().len();
    admit_container_prelude(values).map_err(|error| match error {
        ContainerPreludeAdmissionError::Component(error) => {
            container_component_error(location, error)
        }
        ContainerPreludeAdmissionError::Grammar { index, origin } => {
            let failure = if index >= end {
                None
            } else {
                crate::media::parsed_position(&origin)
            };
            let location = failure.map_or(location, |position| cssparser::SourceLocation {
                line: position.line().value(),
                column: position.column().value() + 1,
            });
            invalid_syntax(location)
        }
    })
}

fn container_component_error<'i>(
    fallback: cssparser::SourceLocation,
    error: crate::CssComponentValueError,
) -> ParseError<'i, Error> {
    let location = crate::media::parsed_position(error.origin()).map_or(fallback, |position| {
        cssparser::SourceLocation {
            line: position.line().value(),
            column: position.column().value() + 1,
        }
    });
    crate::error::invalid_component_value(location, error)
}
#[derive(Clone, Copy)]
enum ContainerFeatureName {
    Width(Option<RangePrefix>),
    Height(Option<RangePrefix>),
    InlineSize(Option<RangePrefix>),
    BlockSize(Option<RangePrefix>),
    AspectRatio(Option<RangePrefix>),
    Orientation,
}

impl ContainerFeatureName {
    fn kind(self) -> CssContainerSizeFeatureKind {
        match self {
            Self::Width(_) => CssContainerSizeFeatureKind::Width,
            Self::Height(_) => CssContainerSizeFeatureKind::Height,
            Self::InlineSize(_) => CssContainerSizeFeatureKind::InlineSize,
            Self::BlockSize(_) => CssContainerSizeFeatureKind::BlockSize,
            Self::AspectRatio(_) => CssContainerSizeFeatureKind::AspectRatio,
            Self::Orientation => CssContainerSizeFeatureKind::Orientation,
        }
    }
    fn prefix(self) -> Option<RangePrefix> {
        match self {
            Self::Width(prefix)
            | Self::Height(prefix)
            | Self::InlineSize(prefix)
            | Self::BlockSize(prefix)
            | Self::AspectRatio(prefix) => prefix,
            Self::Orientation => None,
        }
    }
    fn parse(name: &str) -> Option<Self> {
        Some(match name.to_ascii_lowercase().as_str() {
            "width" => Self::Width(None),
            "min-width" => Self::Width(Some(RangePrefix::Min)),
            "max-width" => Self::Width(Some(RangePrefix::Max)),
            "height" => Self::Height(None),
            "min-height" => Self::Height(Some(RangePrefix::Min)),
            "max-height" => Self::Height(Some(RangePrefix::Max)),
            "inline-size" => Self::InlineSize(None),
            "min-inline-size" => Self::InlineSize(Some(RangePrefix::Min)),
            "max-inline-size" => Self::InlineSize(Some(RangePrefix::Max)),
            "block-size" => Self::BlockSize(None),
            "min-block-size" => Self::BlockSize(Some(RangePrefix::Min)),
            "max-block-size" => Self::BlockSize(Some(RangePrefix::Max)),
            "aspect-ratio" => Self::AspectRatio(None),
            "min-aspect-ratio" => Self::AspectRatio(Some(RangePrefix::Min)),
            "max-aspect-ratio" => Self::AspectRatio(Some(RangePrefix::Max)),
            "orientation" => Self::Orientation,
            _ => return None,
        })
    }
}
