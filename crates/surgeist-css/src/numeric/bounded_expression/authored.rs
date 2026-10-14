//! Records from an already checked typed expression, without a token grammar.
//!
//! The CSS feeder supplies admitted direct children and cached checked types.
//! These constructors share the canonical evaluation records with source input;
//! they perform no numerical evaluation, parsing, traversal or allocation.

use super::*;

pub(crate) const fn node(kind: NodeKind, ty: CssNumericType, site: usize) -> Record {
    Record::Node(Node {
        kind,
        ty,
        site,
        closing: None,
    })
}
pub(crate) const fn term(
    previous: Option<usize>,
    ordinal: usize,
    operand: usize,
    operator: Option<CssCalculationSumOperator>,
    operator_site: Option<usize>,
) -> Record {
    Record::Term(Term {
        previous,
        ordinal,
        operand,
        operator,
        operator_site,
    })
}
pub(crate) const fn factor(
    previous: Option<usize>,
    ordinal: usize,
    operand: usize,
    operator: Option<CssCalculationProductOperator>,
    operator_site: Option<usize>,
) -> Record {
    Record::Factor(Factor {
        previous,
        ordinal,
        operand,
        operator,
        operator_site,
    })
}
pub(crate) const fn argument(
    previous: Option<usize>,
    ordinal: usize,
    node: Option<usize>,
) -> Record {
    Record::Argument(Argument {
        previous,
        ordinal,
        node,
    })
}
