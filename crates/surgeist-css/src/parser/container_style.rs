//! Component-native style grammar; probes select regions of one lexical root.
use super::boolean_condition::{
    BooleanModel, BooleanNode, BooleanNodeKind, ComponentOperands, ConditionFailure,
    parse_component_condition,
};
use super::query_components::ComponentCursor as Cursor;
use crate::component::{CssBlockKind, CssComponentValue, CssComponentValueRef, CssValueTokenRef};
use crate::declaration::CssCustomPropertyName;
use crate::property::CssPropertyGrammar;
use crate::query::container::container_style::StyleRangeKind;
use crate::query::container::{
    CssContainerGeneralEnclosed, CssContainerStyleFeature, CssContainerStyleFeatureName,
    CssContainerStyleRange, CssContainerStyleRangeOperand, CssContainerStyleValue,
    CssStyleQueryLeaf,
};
use crate::query::supports::construction::{SupportsLexical, trivia};
use crate::query::{CssCondition, CssQueryComparison};
use std::ops::Range;

/// A style query, as regions of its lexical root.
pub(super) type StyleNode = BooleanNode<StyleLeaf>;

pub(super) enum StyleLeaf {
    Boolean(CssContainerStyleFeatureName),
    Plain(CssContainerStyleFeatureName, Range<usize>),
    Range(Vec<Range<usize>>, Vec<CssQueryComparison>),
    Opaque,
}

struct StyleModel;

impl BooleanModel<StyleLeaf> for StyleModel {
    type Leaf = CssStyleQueryLeaf;

    fn leaf(leaf: StyleLeaf, lexical: &SupportsLexical) -> CssStyleQueryLeaf {
        let feature = |value| CssStyleQueryLeaf::Feature(value);
        match leaf {
            StyleLeaf::Boolean(name) => feature(CssContainerStyleFeature::Boolean(name)),
            StyleLeaf::Plain(name, range) => feature(CssContainerStyleFeature::Plain {
                name,
                value: CssContainerStyleValue::new(lexical.select(range)),
            }),
            StyleLeaf::Range(ranges, comparisons) => {
                let mut operands = ranges.into_iter().map(|range| {
                    let region = lexical.select(range);
                    let reference = custom_reference(region.items());
                    CssContainerStyleRangeOperand::new(
                        CssContainerStyleValue::new(region),
                        reference,
                    )
                });
                let left = operands.next().expect("left operand");
                let second = operands.next().expect("second operand");
                let kind = if let Some(right) = operands.next() {
                    let left_inclusive = inclusive(comparisons[0]);
                    let right_inclusive = inclusive(comparisons[1]);
                    if ascending(comparisons[0]) {
                        StyleRangeKind::Ascending {
                            left,
                            left_inclusive,
                            middle: second,
                            right_inclusive,
                            right,
                        }
                    } else {
                        StyleRangeKind::Descending {
                            left,
                            left_inclusive,
                            middle: second,
                            right_inclusive,
                            right,
                        }
                    }
                } else {
                    StyleRangeKind::Binary {
                        left,
                        comparison: comparisons[0],
                        right: second,
                    }
                };
                feature(CssContainerStyleFeature::Range(
                    CssContainerStyleRange::new(kind),
                ))
            }
            StyleLeaf::Opaque => CssStyleQueryLeaf::GeneralEnclosed(
                CssContainerGeneralEnclosed::new(lexical.clone()),
            ),
        }
    }
}

impl StyleNode {
    pub(super) fn into_query(self, parent: SupportsLexical) -> CssCondition<CssStyleQueryLeaf> {
        self.build::<StyleModel>(&parent)
    }
}

// A whole style feature is tried before the boolean algebra (Contain 3
// `<style-query>`); an operand that is neither group nor function fails.
pub(super) fn parse(
    items: &[CssComponentValue],
) -> Result<StyleNode, ConditionFailure<std::convert::Infallible>> {
    if let Some(leaf) = feature(items) {
        return Ok(StyleNode {
            range: 0..items.len(),
            kind: BooleanNodeKind::Leaf(leaf),
        });
    }
    parse_component_condition(&mut StyleOperands, items)
}

struct StyleOperands;

impl<'a> ComponentOperands<'a> for StyleOperands {
    type Leaf = StyleLeaf;
    type Error = std::convert::Infallible;

    fn operand(
        &mut self,
        cursor: &mut Cursor<'a>,
    ) -> Result<StyleNode, ConditionFailure<Self::Error>> {
        cursor.skip();
        let start = cursor.index;
        let Some(component) = cursor.next() else {
            return Err(ConditionFailure::Mismatch(start));
        };
        let kind = match component.view() {
            CssComponentValueRef::Block(block) if block.kind() == CssBlockKind::Parenthesis => {
                // A style query has no resource failure; a mismatch is an enclosure.
                match parse(block.values().items()) {
                    Ok(group) => BooleanNodeKind::Parenthesized(Box::new(group)),
                    Err(_) => BooleanNodeKind::Leaf(StyleLeaf::Opaque),
                }
            }
            CssComponentValueRef::Function(_) => BooleanNodeKind::Leaf(StyleLeaf::Opaque),
            _ => return Err(ConditionFailure::Mismatch(start)),
        };
        Ok(StyleNode {
            range: start..cursor.index,
            kind,
        })
    }
}
fn decoded_ident(component: &CssComponentValue) -> Option<&str> {
    match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => Some(name),
        _ => None,
    }
}
fn custom_reference(items: &[CssComponentValue]) -> Option<CssCustomPropertyName> {
    let mut significant = items.iter().filter(|item| !trivia(item));
    let name = decoded_ident(significant.next()?)?;
    if significant.next().is_some() {
        return None;
    }
    super::variables::parse_custom_property_name(name)
}
fn feature_name(name: &str) -> Option<CssContainerStyleFeatureName> {
    if let Some(custom) = super::variables::parse_custom_property_name(name) {
        Some(CssContainerStyleFeatureName::Custom(custom))
    } else {
        CssPropertyGrammar::from_name(name).map(CssContainerStyleFeatureName::Property)
    }
}

fn feature(items: &[CssComponentValue]) -> Option<StyleLeaf> {
    let mut cursor = Cursor::new(items);
    if let Some(name) = cursor.next().and_then(decoded_ident).and_then(feature_name) {
        if cursor.done() {
            return Some(StyleLeaf::Boolean(name));
        }
        if matches!(
            cursor.next().map(CssComponentValue::view),
            Some(CssComponentValueRef::Token(CssValueTokenRef::Colon))
        ) {
            let range = cursor.index..items.len();
            if value(&items[range.clone()]) {
                return Some(StyleLeaf::Plain(name, range));
            }
        }
    }
    // Only sibling comparators split a range; nested comparisons fail operand admission.
    let separators = super::query_components::component_query_comparisons(items)?;
    let mut comparisons = Vec::new();
    let mut ranges = Vec::new();
    let mut start = 0;
    for (range, comparison) in separators {
        ranges.push(start..range.start);
        start = range.end;
        comparisons.push(comparison);
    }
    if comparisons.is_empty() {
        return None;
    }
    if comparisons.len() == 2
        && !(ascending(comparisons[0]) && ascending(comparisons[1])
            || descending(comparisons[0]) && descending(comparisons[1]))
    {
        return None;
    }
    ranges.push(start..items.len());
    if !ranges.iter().all(|range| value(&items[range.clone()])) {
        return None;
    }
    Some(StyleLeaf::Range(ranges, comparisons))
}
fn ascending(comparison: CssQueryComparison) -> bool {
    matches!(
        comparison,
        CssQueryComparison::LessThan | CssQueryComparison::LessThanOrEqual
    )
}
fn descending(comparison: CssQueryComparison) -> bool {
    matches!(
        comparison,
        CssQueryComparison::GreaterThan | CssQueryComparison::GreaterThanOrEqual
    )
}
fn inclusive(comparison: CssQueryComparison) -> bool {
    matches!(
        comparison,
        CssQueryComparison::LessThanOrEqual | CssQueryComparison::GreaterThanOrEqual
    )
}
fn value(items: &[CssComponentValue]) -> bool {
    // Comments remain lexical provenance, but are not tokens in declaration-value.
    if !items
        .iter()
        .any(|item| !matches!(item.view(), CssComponentValueRef::Comment(_)))
    {
        return false;
    }
    let mut pending = vec![(items, true)];
    while let Some((items, top)) = pending.pop() {
        for item in items {
            match item.view() {
                CssComponentValueRef::Token(CssValueTokenRef::Delim('<' | '>' | '=')) => {
                    return false;
                }
                CssComponentValueRef::Token(
                    CssValueTokenRef::Semicolon | CssValueTokenRef::Delim('!'),
                ) if top => return false,
                CssComponentValueRef::Function(function) => {
                    pending.push((function.values().items(), false))
                }
                CssComponentValueRef::Block(block) => pending.push((block.values().items(), false)),
                _ => {}
            }
        }
    }
    super::variables::checked_variable_components(
        items,
        super::variables::SubstitutionContext::PermissiveStyleOperand,
    )
    .is_some()
}
