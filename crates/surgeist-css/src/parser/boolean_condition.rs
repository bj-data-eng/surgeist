//! One `not`/`and`/`or` condition grammar with grammar-specific operands.
//!
//! Media Queries 4 §3 defines `<media-condition>` and its without-`or` form;
//! Conditional 3 `<supports-condition>`, Contain 3 `<container-condition>`,
//! `<style-query>` and `<scroll-state-query>`, and Conditional 5 §3
//! `<boolean-condition>` ("a boolean algebra a la Media Queries 4 § 3") share
//! its shape: one negated operand, one operand, or operands joined by a single
//! repeated operator. Each grammar supplies its operands, its operator
//! provenance and its result construction; this module owns only that shape.

use super::query_components::ComponentCursor;
use crate::supports::{SupportsLexical, trivia};
use crate::{CssComponentValue, CssCondition, CssConditionKind, CssConditionList};
use std::ops::Range;

/// A boolean condition grammar over its own input.
pub(super) trait BooleanConditionGrammar {
    /// The provenance a grammar keeps for each operator keyword.
    type Operator;
    type Node;
    type Error;
    /// Consumes `keyword` when it is next.
    fn keyword(&mut self, keyword: &'static str) -> Result<Option<Self::Operator>, Self::Error>;
    /// One parenthesized or function operand.
    fn operand(&mut self) -> Result<Self::Node, Self::Error>;
    /// The condition ends here: anything that follows is a grammar failure.
    fn finish(&mut self) -> Result<(), Self::Error>;
    fn negation(&mut self, operator: Self::Operator, operand: Self::Node) -> Self::Node;
    fn single(&mut self, operand: Self::Node) -> Self::Node;
    fn junction(
        &mut self,
        conjunction: bool,
        operators: Vec<Self::Operator>,
        operands: Vec<Self::Node>,
    ) -> Self::Node;
}

/// `not <operand> | <operand> [ [and <operand>]* | [or <operand>]* ]`; with
/// `allow_or` false, only the conjunction form (Media Queries 4
/// `<media-condition-without-or>`).
pub(super) fn parse_boolean_condition<G: BooleanConditionGrammar>(
    grammar: &mut G,
    allow_or: bool,
) -> Result<G::Node, G::Error> {
    if let Some(operator) = grammar.keyword("not")? {
        let operand = grammar.operand()?;
        grammar.finish()?;
        return Ok(grammar.negation(operator, operand));
    }
    let first = grammar.operand()?;
    let (keyword, operator) = if let Some(operator) = grammar.keyword("and")? {
        ("and", operator)
    } else if let Some(operator) = if allow_or {
        grammar.keyword("or")?
    } else {
        None
    } {
        ("or", operator)
    } else {
        grammar.finish()?;
        return Ok(grammar.single(first));
    };
    let mut operators = vec![operator];
    let mut operands = vec![first, grammar.operand()?];
    while let Some(operator) = grammar.keyword(keyword)? {
        operators.push(operator);
        operands.push(grammar.operand()?);
    }
    grammar.finish()?;
    Ok(grammar.junction(keyword == "and", operators, operands))
}

/// The single failure convention of component-list conditions: a grammar
/// mismatch at a component index, which permits another interpretation, or a
/// failure that does not.
pub(super) enum ConditionFailure<E> {
    Mismatch(usize),
    Fatal(E),
}

impl<E> From<E> for ConditionFailure<E> {
    fn from(error: E) -> Self {
        Self::Fatal(error)
    }
}

/// A component-list condition retaining each node's region of its list.
pub(super) struct BooleanNode<L> {
    pub(super) range: Range<usize>,
    pub(super) kind: BooleanNodeKind<L>,
}

pub(super) enum BooleanNodeKind<L> {
    Leaf(L),
    Parenthesized(Box<BooleanNode<L>>),
    Not(Box<BooleanNode<L>>),
    And(Vec<BooleanNode<L>>),
    Or(Vec<BooleanNode<L>>),
}

/// Operands of a condition over one component list.
pub(super) trait ComponentOperands<'a> {
    type Leaf;
    type Error;
    /// One operand at the cursor: a component region and its node.
    fn operand(
        &mut self,
        cursor: &mut ComponentCursor<'a>,
    ) -> Result<BooleanNode<Self::Leaf>, ConditionFailure<Self::Error>>;
}

struct ComponentCondition<'a, 'g, G> {
    operands: &'g mut G,
    cursor: ComponentCursor<'a>,
}

impl<'a, G: ComponentOperands<'a>> BooleanConditionGrammar for ComponentCondition<'a, '_, G> {
    type Operator = ();
    type Node = BooleanNode<G::Leaf>;
    type Error = ConditionFailure<G::Error>;

    fn keyword(&mut self, keyword: &'static str) -> Result<Option<()>, Self::Error> {
        Ok(self.cursor.ident(keyword).then_some(()))
    }
    fn operand(&mut self) -> Result<Self::Node, Self::Error> {
        self.operands.operand(&mut self.cursor)
    }
    fn finish(&mut self) -> Result<(), Self::Error> {
        if self.cursor.done() {
            Ok(())
        } else {
            Err(ConditionFailure::Mismatch(self.cursor.index))
        }
    }
    fn negation(&mut self, (): (), operand: Self::Node) -> Self::Node {
        whole(self.cursor.items, BooleanNodeKind::Not(Box::new(operand)))
    }
    fn single(&mut self, mut operand: Self::Node) -> Self::Node {
        operand.range = 0..self.cursor.items.len();
        operand
    }
    fn junction(&mut self, conjunction: bool, _: Vec<()>, operands: Vec<Self::Node>) -> Self::Node {
        whole(
            self.cursor.items,
            if conjunction {
                BooleanNodeKind::And(operands)
            } else {
                BooleanNodeKind::Or(operands)
            },
        )
    }
}

// A combined condition spans its whole component list.
fn whole<L>(items: &[CssComponentValue], kind: BooleanNodeKind<L>) -> BooleanNode<L> {
    BooleanNode {
        range: 0..items.len(),
        kind,
    }
}

/// Parses `items` as one complete condition with `operands`.
pub(super) fn parse_component_condition<'a, G: ComponentOperands<'a>>(
    operands: &mut G,
    items: &'a [CssComponentValue],
) -> Result<BooleanNode<G::Leaf>, ConditionFailure<G::Error>> {
    parse_boolean_condition(
        &mut ComponentCondition {
            operands,
            cursor: ComponentCursor::new(items),
        },
        true,
    )
}

/// Builds one public condition model from a component-list node tree.
pub(super) trait BooleanModel<L> {
    /// The public leaf type of the built condition.
    type Leaf: crate::CssConditionLeaf<Backing = crate::condition::LexicalBacking>;
    fn leaf(leaf: L, lexical: &SupportsLexical) -> Self::Leaf;
}

impl<L> BooleanNode<L> {
    /// Every node selects its region of the shared lexical root; a group's
    /// contents are its enclosure's children.
    pub(super) fn build<M: BooleanModel<L>>(
        self,
        parent: &SupportsLexical,
    ) -> CssCondition<M::Leaf> {
        let lexical = parent.select(self.range);
        let kind = match self.kind {
            BooleanNodeKind::Leaf(leaf) => CssConditionKind::Leaf(M::leaf(leaf, &lexical)),
            BooleanNodeKind::Parenthesized(group) => {
                let opener = first_significant(&lexical);
                CssConditionKind::Parenthesized(Box::new(
                    group.build::<M>(&lexical.children(opener)),
                ))
            }
            BooleanNodeKind::Not(operand) => {
                CssConditionKind::Not(Box::new(operand.build::<M>(&lexical)))
            }
            BooleanNodeKind::And(operands) => CssConditionKind::And(CssConditionList::new(
                operands
                    .into_iter()
                    .map(|operand| operand.build::<M>(&lexical))
                    .collect(),
            )),
            BooleanNodeKind::Or(operands) => CssConditionKind::Or(CssConditionList::new(
                operands
                    .into_iter()
                    .map(|operand| operand.build::<M>(&lexical))
                    .collect(),
            )),
        };
        CssCondition::lexical(kind, lexical)
    }
}

/// The index of a region's enclosing operand.
pub(super) fn first_significant(lexical: &SupportsLexical) -> usize {
    lexical
        .items()
        .iter()
        .position(|item| !trivia(item))
        .expect("an operand region holds its enclosure")
}
