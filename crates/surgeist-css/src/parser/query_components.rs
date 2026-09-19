//! Neutral component cursor and declaration-value admission shared by container queries.
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
    let Some(pending) = super::variables::checked_variable_components(items) else {
        return Ok(None);
    };
    Ok(Some((
        CssComponentValues::try_new(items.to_vec())?,
        pending,
    )))
}
