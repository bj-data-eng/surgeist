//! Component-native scroll grammar with shared-root recursive query regions.
use super::boolean_condition::{
    BooleanModel, BooleanNode, BooleanNodeKind, ComponentOperands, ConditionFailure,
    parse_component_condition,
};
use super::query_components::{ComponentCursor as Cursor, authored_operand};
use crate::supports::{SupportsLexical, trivia};
use crate::*;

/// A scroll-state query, as regions of its lexical root.
pub(super) type ScrollNode = BooleanNode<ScrollLeaf>;

pub(super) enum ScrollLeaf {
    Feature(CssContainerScrollFeature),
    Opaque,
}

struct ScrollModel;

impl BooleanModel<ScrollLeaf> for ScrollModel {
    type Leaf = CssScrollStateLeaf;

    fn leaf(leaf: ScrollLeaf, lexical: &SupportsLexical) -> CssScrollStateLeaf {
        match leaf {
            ScrollLeaf::Feature(feature) => CssScrollStateLeaf::Feature(feature),
            ScrollLeaf::Opaque => CssScrollStateLeaf::GeneralEnclosed(
                CssContainerGeneralEnclosed::new(lexical.clone()),
            ),
        }
    }
}

impl ScrollNode {
    pub(super) fn into_query(self, parent: SupportsLexical) -> CssCondition<CssScrollStateLeaf> {
        self.build::<ScrollModel>(&parent)
    }
}

// A whole scroll-state feature is tried before the boolean algebra.
pub(super) fn parse(
    items: &[CssComponentValue],
) -> Result<ScrollNode, ConditionFailure<CssComponentValueError>> {
    if let Some(feature) = feature(items)? {
        return Ok(ScrollNode {
            range: 0..items.len(),
            kind: BooleanNodeKind::Leaf(ScrollLeaf::Feature(feature)),
        });
    }
    parse_component_condition(&mut ScrollOperands, items)
}

struct ScrollOperands;

impl<'a> ComponentOperands<'a> for ScrollOperands {
    type Leaf = ScrollLeaf;
    type Error = CssComponentValueError;

    fn operand(
        &mut self,
        cursor: &mut Cursor<'a>,
    ) -> Result<ScrollNode, ConditionFailure<Self::Error>> {
        cursor.skip();
        let start = cursor.index;
        let Some(component) = cursor.next() else {
            return Err(ConditionFailure::Mismatch(start));
        };
        let kind = match component.view() {
            CssComponentValueRef::Block(block) if block.kind() == CssBlockKind::Parenthesis => {
                match parse(block.values().items()) {
                    Ok(group) => BooleanNodeKind::Parenthesized(Box::new(group)),
                    Err(ConditionFailure::Mismatch(_)) => BooleanNodeKind::Leaf(ScrollLeaf::Opaque),
                    Err(failure @ ConditionFailure::Fatal(_)) => return Err(failure),
                }
            }
            CssComponentValueRef::Function(_) => BooleanNodeKind::Leaf(ScrollLeaf::Opaque),
            _ => return Err(ConditionFailure::Mismatch(start)),
        };
        Ok(ScrollNode {
            range: start..cursor.index,
            kind,
        })
    }
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
