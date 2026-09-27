//! Neutral component cursor and query range helpers.
use crate::media_features::MediaRangeState;
use std::ops::Range;

use crate::supports::trivia;
use crate::*;

pub(super) struct ComponentCursor<'a> {
    pub(super) items: &'a [CssComponentValue],
    pub(super) index: usize,
}
impl<'a> ComponentCursor<'a> {
    pub(super) fn new(items: &'a [CssComponentValue]) -> Self {
        Self { items, index: 0 }
    }
    pub(super) fn skip(&mut self) {
        while self.items.get(self.index).is_some_and(trivia) {
            self.index += 1;
        }
    }
    pub(super) fn peek(&mut self) -> Option<&'a CssComponentValue> {
        self.skip();
        self.items.get(self.index)
    }
    pub(super) fn next(&mut self) -> Option<&'a CssComponentValue> {
        let item = self.peek()?;
        self.index += 1;
        Some(item)
    }
    pub(super) fn ident(&mut self, keyword: &str) -> bool {
        if matches!(self.peek().map(CssComponentValue::view), Some(CssComponentValueRef::Token(CssValueTokenRef::Ident(name))) if name.eq_ignore_ascii_case(keyword))
        {
            self.index += 1;
            true
        } else {
            false
        }
    }
    pub(super) fn token(&mut self) -> Option<CssValueTokenRef<'a>> {
        match self.next()?.view() {
            CssComponentValueRef::Token(token) => Some(token),
            _ => None,
        }
    }
    pub(super) fn done(&mut self) -> bool {
        self.peek().is_none()
    }
}

/// Preserve the complete nontrivia-bounded operand, validating every var() call.
/// A true flag means post-substitution domain checking is required for the whole value.
/// Input component validity and whole-query limits have already been checked.
pub(super) fn authored_operand(
    items: &[CssComponentValue],
) -> Result<Option<(CssComponentValues, bool)>, CssComponentValueError> {
    let start = items
        .iter()
        .position(|item| !trivia(item))
        .unwrap_or(items.len());
    let end = items
        .iter()
        .rposition(|item| !trivia(item))
        .map_or(start, |index| index + 1);
    let items = &items[start..end];
    if items.is_empty()
        || items.iter().any(|item| {
            matches!(
                item.view(),
                CssComponentValueRef::Token(
                    CssValueTokenRef::Semicolon | CssValueTokenRef::Delim('!')
                )
            )
        })
    {
        return Ok(None);
    }
    let Some(pending) = super::variables::checked_variable_components(
        items,
        super::variables::SubstitutionContext::ExistingVarOnlyQuery,
    ) else {
        return Ok(None);
    };
    Ok(Some((
        CssComponentValues::try_new(items.to_vec())?,
        pending,
    )))
}

pub(super) fn component_query_comparisons(
    items: &[CssComponentValue],
) -> Option<Vec<(Range<usize>, CssQueryComparison)>> {
    let mut comparisons = Vec::new();
    let mut index = 0;
    while index < items.len() {
        if let CssComponentValueRef::Token(CssValueTokenRef::Delim(symbol @ ('<' | '>' | '='))) =
            items[index].view()
        {
            let start = index;
            index += 1;
            let mut end = index;
            let inclusive = if symbol != '=' {
                while matches!(
                    items.get(end).map(CssComponentValue::view),
                    Some(CssComponentValueRef::Comment(_))
                ) {
                    end += 1;
                }
                matches!(
                    items.get(end).map(CssComponentValue::view),
                    Some(CssComponentValueRef::Token(CssValueTokenRef::Delim('=')))
                )
            } else {
                false
            };
            if inclusive {
                index = end + 1;
            }
            comparisons.push((start..index, query_comparison(symbol, inclusive)));
            if comparisons.len() > 2 {
                return None;
            }
        } else {
            index += 1;
        }
    }
    Some(comparisons)
}

pub(super) fn query_comparison(symbol: char, inclusive: bool) -> CssQueryComparison {
    match (symbol, inclusive) {
        ('<', false) => CssQueryComparison::LessThan,
        ('<', true) => CssQueryComparison::LessThanOrEqual,
        ('>', false) => CssQueryComparison::GreaterThan,
        ('>', true) => CssQueryComparison::GreaterThanOrEqual,
        ('=', false) => CssQueryComparison::Equal,
        _ => unreachable!("checked query comparison"),
    }
}
pub(super) fn query_plain_range<T>(value: T, prefix: Option<RangePrefix>) -> MediaRangeState<T> {
    match prefix {
        None => MediaRangeState::Plain { value },
        Some(RangePrefix::Min) => MediaRangeState::Min { value },
        Some(RangePrefix::Max) => MediaRangeState::Max { value },
    }
}
pub(super) fn query_range_chain<T>(
    left: T,
    first: CssQueryComparison,
    right: T,
    second: CssQueryComparison,
) -> Option<MediaRangeState<T>> {
    use CssQueryComparison::{GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual};
    Some(match (first, second) {
        (LessThan | LessThanOrEqual, LessThan | LessThanOrEqual) => MediaRangeState::Ascending {
            left,
            left_inclusive: first == LessThanOrEqual,
            right,
            right_inclusive: second == LessThanOrEqual,
        },
        (GreaterThan | GreaterThanOrEqual, GreaterThan | GreaterThanOrEqual) => {
            MediaRangeState::Descending {
                left,
                left_inclusive: first == GreaterThanOrEqual,
                right,
                right_inclusive: second == GreaterThanOrEqual,
            }
        }
        _ => return None,
    })
}

pub(super) fn media_literal_number(
    value: &crate::CssComponentValues,
) -> Option<crate::CssNumericTokenRef<'_>> {
    value.items().iter().find_map(|c| match c.view() {
        crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Number(n)) => Some(n),
        _ => None,
    })
}
pub(super) fn negative_literal(number: crate::CssNumericTokenRef<'_>) -> bool {
    let text = number.representation();
    text.starts_with('-')
        && text
            .split(['e', 'E'])
            .next()
            .expect("numeric mantissa")
            .bytes()
            .any(|b| matches!(b, b'1'..=b'9'))
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum RangePrefix {
    Min,
    Max,
}
