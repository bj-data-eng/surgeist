//! Component-native scroll grammar with shared-root recursive query regions.
use super::query_components::{ComponentCursor as Cursor, authored_operand};
use crate::supports::{SupportsLexical, trivia};
use crate::*;
use std::ops::Range;

pub(super) struct ScrollNode {
    range: Range<usize>,
    kind: ScrollNodeKind,
}
enum ScrollNodeKind {
    Feature(CssContainerScrollFeature),
    Parenthesized(Box<ScrollNode>),
    Not(Box<ScrollNode>),
    And(Vec<ScrollNode>),
    Or(Vec<ScrollNode>),
    Opaque,
}
impl ScrollNode {
    pub(super) fn into_query(self, parent: SupportsLexical) -> CssContainerScrollQuery {
        let lexical = parent.select(self.range);
        let kind = match self.kind {
            ScrollNodeKind::Feature(feature) => CssContainerScrollQueryKind::Feature(feature),
            ScrollNodeKind::Parenthesized(child) => {
                let opener = lexical
                    .items()
                    .iter()
                    .position(|item| !trivia(item))
                    .expect("group");
                CssContainerScrollQueryKind::Parenthesized(Box::new(
                    child.into_query(lexical.children(opener)),
                ))
            }
            ScrollNodeKind::Not(child) => {
                CssContainerScrollQueryKind::Not(Box::new(child.into_query(lexical.clone())))
            }
            ScrollNodeKind::And(children) => {
                CssContainerScrollQueryKind::And(CssContainerScrollQueryList::new(
                    children
                        .into_iter()
                        .map(|child| child.into_query(lexical.clone()))
                        .collect(),
                ))
            }
            ScrollNodeKind::Or(children) => {
                CssContainerScrollQueryKind::Or(CssContainerScrollQueryList::new(
                    children
                        .into_iter()
                        .map(|child| child.into_query(lexical.clone()))
                        .collect(),
                ))
            }
            ScrollNodeKind::Opaque => CssContainerScrollQueryKind::GeneralEnclosed(
                CssContainerGeneralEnclosed::new(lexical.clone()),
            ),
        };
        CssContainerScrollQuery::new(kind, lexical)
    }
}

pub(super) fn parse(
    items: &[CssComponentValue],
) -> Result<Option<ScrollNode>, CssComponentValueError> {
    if let Some(feature) = feature(items)? {
        return Ok(Some(ScrollNode {
            range: 0..items.len(),
            kind: ScrollNodeKind::Feature(feature),
        }));
    }
    let mut cursor = Cursor::new(items);
    if cursor.ident("not") {
        let Some(child) = atom(&mut cursor)? else {
            return Ok(None);
        };
        return Ok(cursor.done().then_some(ScrollNode {
            range: 0..items.len(),
            kind: ScrollNodeKind::Not(Box::new(child)),
        }));
    }
    let Some(mut first) = atom(&mut cursor)? else {
        return Ok(None);
    };
    let conjunction = if cursor.ident("and") {
        true
    } else if cursor.ident("or") {
        false
    } else {
        if !cursor.done() {
            return Ok(None);
        }
        first.range = 0..items.len();
        return Ok(Some(first));
    };
    let mut children = vec![first];
    loop {
        let Some(child) = atom(&mut cursor)? else {
            return Ok(None);
        };
        children.push(child);
        if !cursor.ident(if conjunction { "and" } else { "or" }) {
            break;
        }
    }
    Ok(cursor.done().then_some(ScrollNode {
        range: 0..items.len(),
        kind: if conjunction {
            ScrollNodeKind::And(children)
        } else {
            ScrollNodeKind::Or(children)
        },
    }))
}
fn atom(cursor: &mut Cursor<'_>) -> Result<Option<ScrollNode>, CssComponentValueError> {
    cursor.skip();
    let start = cursor.index;
    let Some(component) = cursor.next() else {
        return Ok(None);
    };
    let kind = match component.view() {
        CssComponentValueRef::Block(block) if block.kind() == CssBlockKind::Parenthesis => {
            match parse(block.values().items())? {
                Some(child) => ScrollNodeKind::Parenthesized(Box::new(child)),
                None => ScrollNodeKind::Opaque,
            }
        }
        CssComponentValueRef::Function(_) => ScrollNodeKind::Opaque,
        _ => return Ok(None),
    };
    Ok(Some(ScrollNode {
        range: start..cursor.index,
        kind,
    }))
}
fn feature(
    items: &[CssComponentValue],
) -> Result<Option<CssContainerScrollFeature>, CssComponentValueError> {
    let mut cursor = Cursor::new(items);
    let Some(CssValueTokenRef::Ident(name)) = cursor.token() else {
        return Ok(None);
    };
    let Some(kind) = CssContainerScrollFeatureKind::parse(name) else {
        return Ok(None);
    };
    if cursor.done() {
        return Ok(Some(CssContainerScrollFeature::Boolean(kind)));
    }
    if !matches!(cursor.token(), Some(CssValueTokenRef::Colon)) {
        return Ok(None);
    }
    let Some((values, pending)) = authored_operand(&items[cursor.index..])? else {
        return Ok(None);
    };
    if pending {
        return Ok(Some(match kind {
            CssContainerScrollFeatureKind::Stuck => {
                CssContainerScrollFeature::Stuck(CssContainerStuckValue::pending(values))
            }
            CssContainerScrollFeatureKind::Snapped => {
                CssContainerScrollFeature::Snapped(CssContainerSnappedValue::pending(values))
            }
            CssContainerScrollFeatureKind::Scrollable => CssContainerScrollFeature::Scrollable(
                CssContainerScrollDirectionValue::pending(values),
            ),
            CssContainerScrollFeatureKind::Scrolled => CssContainerScrollFeature::Scrolled(
                CssContainerScrollDirectionValue::pending(values),
            ),
        }));
    }
    let mut significant = values.items().iter().filter(|item| !trivia(item));
    let Some(CssComponentValueRef::Token(CssValueTokenRef::Ident(keyword))) =
        significant.next().map(CssComponentValue::view)
    else {
        return Ok(None);
    };
    if significant.next().is_some() {
        return Ok(None);
    }
    let feature = match kind {
        CssContainerScrollFeatureKind::Stuck => {
            CssContainerStuckKeyword::parse(keyword).map(|keyword| {
                CssContainerScrollFeature::Stuck(CssContainerStuckValue::typed(keyword, values))
            })
        }
        CssContainerScrollFeatureKind::Snapped => {
            CssContainerSnappedKeyword::parse(keyword).map(|keyword| {
                CssContainerScrollFeature::Snapped(CssContainerSnappedValue::typed(keyword, values))
            })
        }
        CssContainerScrollFeatureKind::Scrollable => {
            CssContainerScrollDirectionKeyword::parse(keyword).map(|keyword| {
                CssContainerScrollFeature::Scrollable(CssContainerScrollDirectionValue::typed(
                    keyword, values,
                ))
            })
        }
        CssContainerScrollFeatureKind::Scrolled => {
            CssContainerScrollDirectionKeyword::parse(keyword).map(|keyword| {
                CssContainerScrollFeature::Scrolled(CssContainerScrollDirectionValue::typed(
                    keyword, values,
                ))
            })
        }
    };
    Ok(feature)
}
