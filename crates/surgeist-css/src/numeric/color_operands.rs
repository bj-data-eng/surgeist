//! Constant-time borrows of the original checked numerical tree.
//!
//! The caller admits the typed parent header before observing it, then the
//! selected direct child record before borrowing that child. A returned Node
//! only lends a pointer: its header and any literal component are separate
//! observations. None of these views classifies a unit name or visits text.

use crate::color::CssRelativeColorChannel;
use crate::component::{CssComponentValue, CssValueOrigin};
use crate::numeric::expression::{CssCalculationExpression, NodeKind};
use crate::numeric::{
    CssCalculationProductOperator, CssCalculationSumOperator, CssMathFunction, CssNumericConstant,
    CssNumericLiteralRef, CssNumericType, CssRoundingStrategy,
};
use std::mem::size_of;

#[derive(Clone, Copy)]
pub(crate) struct Node<'a>(&'a CssCalculationExpression);

// Formatting a witness must not recursively format the borrowed AST or text.
impl std::fmt::Debug for Node<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("Node")
            .field(&std::ptr::from_ref(self.0))
            .finish()
    }
}

pub(crate) fn node(expression: &CssCalculationExpression) -> Node<'_> {
    Node(expression)
}

#[derive(Clone, Copy)]
pub(crate) struct Header<'a> {
    pub(crate) kind: Kind<'a>,
    pub(crate) ty: CssNumericType,
    pub(crate) origin: &'a CssValueOrigin,
    pub(crate) closing: Option<&'a CssValueOrigin>,
    pub(crate) syntax_len: usize,
}

#[derive(Clone, Copy)]
pub(crate) enum Kind<'a> {
    Value(CssNumericLiteralRef<'a>),
    Constant(CssNumericConstant),
    Size,
    TreeCounting,
    Variable(CssRelativeColorChannel),
    ProfileChannel(&'a crate::color::CssColorProfileComponentName),
    Group,
    Sum {
        len: usize,
    },
    Product {
        len: usize,
    },
    Function {
        function: CssMathFunction,
        len: usize,
        strategy: Option<CssRoundingStrategy>,
    },
}

pub(crate) fn header(Node(expression): Node<'_>) -> Header<'_> {
    let kind = match &expression.kind {
        // Constructing this literal reference borrows the existing component
        // and copies cached type metadata. It does not inspect the component
        // token to distinguish Integer/Number or dispatch a dimension unit.
        NodeKind::Value(component) => Kind::Value(CssNumericLiteralRef {
            component,
            ty: expression.ty,
        }),
        NodeKind::Constant(value) => Kind::Constant(*value),
        NodeKind::Size => Kind::Size,
        NodeKind::TreeCounting(_) => Kind::TreeCounting,
        NodeKind::Variable(channel) => Kind::Variable(*channel),
        NodeKind::ProfileChannel(name) => Kind::ProfileChannel(name),
        NodeKind::Group(_) => Kind::Group,
        NodeKind::Sum(values) => Kind::Sum { len: values.len() },
        NodeKind::Product(values) => Kind::Product { len: values.len() },
        NodeKind::Function {
            function,
            args,
            strategy,
        } => Kind::Function {
            function: *function,
            len: args.len(),
            strategy: *strategy,
        },
    };
    Header {
        kind,
        ty: expression.ty,
        origin: &expression.origin,
        closing: expression.closing.as_ref(),
        syntax_len: expression.syntax.len(),
    }
}

pub(crate) const fn header_bytes() -> usize {
    size_of::<CssCalculationExpression>()
}

#[derive(Clone, Copy)]
pub(crate) struct Child<'a> {
    pub(crate) expression: Option<Node<'a>>,
    pub(crate) operator: Operator,
}

#[derive(Clone, Copy)]
pub(crate) enum Operator {
    None,
    Sum(CssCalculationSumOperator),
    Product(CssCalculationProductOperator),
}

/// The parent header and selected stored child record are admitted separately.
/// Outer None is an invalid ordinal; inner None is an actual missing argument.
pub(crate) fn child(Node(expression): Node<'_>, index: usize) -> Option<Child<'_>> {
    match &expression.kind {
        NodeKind::Group(value) => (index == 0).then(|| Child {
            expression: Some(node(value)),
            operator: Operator::None,
        }),
        NodeKind::Sum(values) => values.get(index).map(|(operator, expression)| Child {
            expression: Some(node(expression)),
            operator: operator.map_or(Operator::None, Operator::Sum),
        }),
        NodeKind::Product(values) => values.get(index).map(|(operator, expression)| Child {
            expression: Some(node(expression)),
            operator: operator.map_or(Operator::None, Operator::Product),
        }),
        NodeKind::Function { args, .. } => args.get(index).map(|expression| Child {
            expression: expression.as_ref().map(node),
            operator: Operator::None,
        }),
        _ => None,
    }
}

/// Dispatches only the already observed kind, without reading a tree node.
/// A Group stores a Box pointer; observing its pointed child header requires
/// the separate header_bytes admission after this pointer has been borrowed.
pub(crate) fn child_bytes(kind: Kind<'_>) -> usize {
    match kind {
        Kind::Group => size_of::<Box<CssCalculationExpression>>(),
        Kind::Sum { .. } => {
            size_of::<(Option<CssCalculationSumOperator>, CssCalculationExpression)>()
        }
        Kind::Product { .. } => size_of::<(
            Option<CssCalculationProductOperator>,
            CssCalculationExpression,
        )>(),
        Kind::Function { .. } => size_of::<Option<CssCalculationExpression>>(),
        _ => 0,
    }
}

pub(crate) fn syntax_item(Node(expression): Node<'_>, index: usize) -> Option<&CssComponentValue> {
    expression.syntax.get(index)
}

pub(crate) const fn syntax_item_bytes() -> usize {
    size_of::<CssComponentValue>()
}

pub(crate) fn same(left: Node<'_>, right: Node<'_>) -> bool {
    std::ptr::eq(left.0, right.0)
}
