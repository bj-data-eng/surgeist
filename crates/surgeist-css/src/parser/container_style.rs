//! Component-native style grammar; probes select regions of one lexical root.
use super::query_components::ComponentCursor as Cursor;
use crate::container_style::StyleRangeKind;
use crate::supports::{SupportsLexical, trivia};
use crate::*;
use std::ops::Range;

pub(super) struct StyleNode {
    range: Range<usize>,
    kind: StyleNodeKind,
}
enum StyleNodeKind {
    Boolean(CssContainerStyleFeatureName),
    Plain(CssContainerStyleFeatureName, Range<usize>),
    Range(Vec<Range<usize>>, Vec<CssQueryComparison>),
    Parenthesized(Box<StyleNode>),
    Not(Box<StyleNode>),
    And(Vec<StyleNode>),
    Or(Vec<StyleNode>),
    Opaque,
}
impl StyleNode {
    pub(super) fn into_query(self, parent: SupportsLexical) -> CssContainerStyleQuery {
        let lexical = parent.select(self.range);
        let feature = |value| CssContainerStyleQueryKind::Feature(value);
        let kind = match self.kind {
            StyleNodeKind::Boolean(name) => feature(CssContainerStyleFeature::Boolean(name)),
            StyleNodeKind::Plain(name, range) => feature(CssContainerStyleFeature::Plain {
                name,
                value: CssContainerStyleValue::new(lexical.select(range)),
            }),
            StyleNodeKind::Range(ranges, comparisons) => {
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
            StyleNodeKind::Parenthesized(child) => {
                let index = lexical
                    .items()
                    .iter()
                    .position(|item| !trivia(item))
                    .expect("group");
                CssContainerStyleQueryKind::Parenthesized(Box::new(
                    child.into_query(lexical.children(index)),
                ))
            }
            StyleNodeKind::Not(child) => {
                CssContainerStyleQueryKind::Not(Box::new(child.into_query(lexical.clone())))
            }
            StyleNodeKind::And(children) => {
                CssContainerStyleQueryKind::And(CssContainerStyleQueryList::new(
                    children
                        .into_iter()
                        .map(|child| child.into_query(lexical.clone()))
                        .collect(),
                ))
            }
            StyleNodeKind::Or(children) => {
                CssContainerStyleQueryKind::Or(CssContainerStyleQueryList::new(
                    children
                        .into_iter()
                        .map(|child| child.into_query(lexical.clone()))
                        .collect(),
                ))
            }
            StyleNodeKind::Opaque => CssContainerStyleQueryKind::GeneralEnclosed(
                CssContainerGeneralEnclosed::new(lexical.clone()),
            ),
        };
        CssContainerStyleQuery::new(kind, lexical)
    }
}

pub(super) fn parse(items: &[CssComponentValue]) -> Option<StyleNode> {
    if let Some(kind) = feature(items) {
        return Some(StyleNode {
            range: 0..items.len(),
            kind,
        });
    }
    let mut cursor = Cursor::new(items);
    if cursor.ident("not") {
        let child = atom(&mut cursor)?;
        return cursor.done().then_some(StyleNode {
            range: 0..items.len(),
            kind: StyleNodeKind::Not(Box::new(child)),
        });
    }
    let mut first = atom(&mut cursor)?;
    let conjunction = if cursor.ident("and") {
        true
    } else if cursor.ident("or") {
        false
    } else {
        if !cursor.done() {
            return None;
        }
        first.range = 0..items.len();
        return Some(first);
    };
    let mut children = vec![first];
    loop {
        children.push(atom(&mut cursor)?);
        if !cursor.ident(if conjunction { "and" } else { "or" }) {
            break;
        }
    }
    cursor.done().then_some(StyleNode {
        range: 0..items.len(),
        kind: if conjunction {
            StyleNodeKind::And(children)
        } else {
            StyleNodeKind::Or(children)
        },
    })
}
fn atom(cursor: &mut Cursor<'_>) -> Option<StyleNode> {
    cursor.skip();
    let start = cursor.index;
    let component = cursor.next()?;
    let kind = match component.view() {
        CssComponentValueRef::Block(block) if block.kind() == CssBlockKind::Parenthesis => {
            match parse(block.values().items()) {
                Some(child) => StyleNodeKind::Parenthesized(Box::new(child)),
                None => StyleNodeKind::Opaque,
            }
        }
        CssComponentValueRef::Function(_) => StyleNodeKind::Opaque,
        _ => return None,
    };
    Some(StyleNode {
        range: start..cursor.index,
        kind,
    })
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

fn feature(items: &[CssComponentValue]) -> Option<StyleNodeKind> {
    let mut cursor = Cursor::new(items);
    if let Some(name) = cursor.next().and_then(decoded_ident).and_then(feature_name) {
        if cursor.done() {
            return Some(StyleNodeKind::Boolean(name));
        }
        if matches!(
            cursor.next().map(CssComponentValue::view),
            Some(CssComponentValueRef::Token(CssValueTokenRef::Colon))
        ) {
            let range = cursor.index..items.len();
            if value(&items[range.clone()]) {
                return Some(StyleNodeKind::Plain(name, range));
            }
        }
    }
    // Only sibling comparators split a range; nested comparisons fail operand admission.
    let separators = super::queries::component_query_comparisons(items)?;
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
    Some(StyleNodeKind::Range(ranges, comparisons))
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
