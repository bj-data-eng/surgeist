//! Admitted inspection of the metadata determining an AST node's source start.

use super::{NameAdmission, NameWork};
use crate::{
    Expression, Position, Spanned,
    expression::{
        Call,
        access::{PropertyAccess, PropertyAccessField},
        operator::{Assign, Unary, Update, assign::AssignTarget},
    },
    function::{FunctionBody, FunctionExpression, PrivateName},
    pattern::Pattern,
};

mod private {
    pub trait Sealed {}
}

/// An AST node with an admitted source-start query.
///
/// The query preserves [`Spanned::span`]'s starting position while inspecting
/// only the nodes that determine that start. It does not inspect unrelated end
/// positions or validate a complete composed range. In particular, parentheses
/// retain their stored opening position rather than being flattened.
///
/// Implementations are explicit and sealed: an arbitrary [`Spanned`] value
/// cannot fall back to an unadmitted full-span traversal. The traversal uses an
/// iterative cursor with constant auxiliary storage and no scratch allocation.
pub trait SourceStart: Spanned + private::Sealed {
    /// Gets the source start, admitting each inspected typed node first.
    ///
    /// `None` selects the same start-only traversal without a caller policy.
    ///
    /// # Errors
    /// Returns the caller's original first refusal, without inspecting later nodes.
    fn try_start_position<E>(&self, admission: NameAdmission<'_, E>) -> Result<Position, E>;
}

// Borrowing does not introduce another AST node or an unmetered span fallback.
impl<T: SourceStart> private::Sealed for &T {}
impl<T: SourceStart> SourceStart for &T {
    fn try_start_position<E>(&self, admission: NameAdmission<'_, E>) -> Result<Position, E> {
        T::try_start_position(*self, admission)
    }
}

enum StartNode<'a> {
    Expression(&'a Expression),
    Assign(&'a Assign),
    AssignTarget(&'a AssignTarget),
    PropertyAccess(&'a PropertyAccess),
    PropertyAccessField(&'a PropertyAccessField),
    Pattern(&'a Pattern),
    Unary(&'a Unary),
    Update(&'a Update),
    FunctionExpression(&'a FunctionExpression),
    FunctionBody(&'a FunctionBody),
    PrivateName(&'a PrivateName),
    Call(&'a Call),
}

fn visit<E>(admission: &mut NameAdmission<'_, E>) -> Result<(), E> {
    if let Some(admit) = admission.as_deref_mut() {
        admit(NameWork::Visit)?;
    }
    Ok(())
}

fn start<E>(mut node: StartNode<'_>, mut admission: NameAdmission<'_, E>) -> Result<Position, E> {
    loop {
        visit(&mut admission)?;
        // This macro is used only for the explicitly listed stored-span leaves.
        // There is no generic Spanned branch or catch-all expression fallback.
        macro_rules! stored {
            ($leaf:expr) => {{
                visit(&mut admission)?;
                return Ok($leaf.span().start());
            }};
        }
        node = match node {
            StartNode::Expression(expression) => match expression {
                Expression::This(value) => stored!(value),
                Expression::Identifier(value) => stored!(value),
                Expression::Literal(value) => stored!(value),
                Expression::RegExpLiteral(value) => stored!(value),
                Expression::ArrayLiteral(value) => stored!(value),
                Expression::ObjectLiteral(value) => stored!(value),
                Expression::Spread(value) => stored!(value),
                Expression::FunctionExpression(value) => StartNode::FunctionExpression(value),
                Expression::ArrowFunction(value) => stored!(value),
                Expression::AsyncArrowFunction(value) => stored!(value),
                Expression::GeneratorExpression(value) => stored!(value),
                Expression::AsyncFunctionExpression(value) => stored!(value),
                Expression::AsyncGeneratorExpression(value) => stored!(value),
                Expression::ClassExpression(value) => stored!(value),
                Expression::TemplateLiteral(value) => stored!(value),
                Expression::PropertyAccess(value) => StartNode::PropertyAccess(value),
                Expression::New(value) => {
                    visit(&mut admission)?;
                    StartNode::Call(value.call())
                }
                Expression::Call(value) => StartNode::Call(value),
                Expression::SuperCall(value) => stored!(value),
                Expression::ImportCall(value) => stored!(value),
                Expression::Optional(value) => stored!(value),
                Expression::TaggedTemplate(value) => stored!(value),
                Expression::NewTarget(value) => stored!(value),
                Expression::ImportMeta(value) => stored!(value),
                Expression::Assign(value) => StartNode::Assign(value),
                Expression::Unary(value) => StartNode::Unary(value),
                Expression::Update(value) => StartNode::Update(value),
                Expression::Binary(value) => {
                    visit(&mut admission)?;
                    StartNode::Expression(value.lhs())
                }
                Expression::BinaryInPrivate(value) => {
                    visit(&mut admission)?;
                    StartNode::PrivateName(value.lhs())
                }
                Expression::Conditional(value) => {
                    visit(&mut admission)?;
                    StartNode::Expression(value.condition())
                }
                Expression::Await(value) => stored!(value),
                Expression::Yield(value) => stored!(value),
                Expression::Parenthesized(value) => stored!(value),
            },
            StartNode::Assign(assign) => StartNode::AssignTarget(assign.lhs()),
            StartNode::AssignTarget(target) => match target {
                AssignTarget::Identifier(identifier) => stored!(identifier),
                AssignTarget::Access(access) => StartNode::PropertyAccess(access),
                AssignTarget::Pattern(pattern) => StartNode::Pattern(pattern),
            },
            StartNode::PropertyAccess(access) => match access {
                PropertyAccess::Simple(access) => {
                    visit(&mut admission)?;
                    StartNode::Expression(access.target())
                }
                PropertyAccess::Private(access) => stored!(access),
                PropertyAccess::Super(access) => stored!(access),
            },
            StartNode::PropertyAccessField(field) => match field {
                PropertyAccessField::Const(identifier) => stored!(identifier),
                PropertyAccessField::Expr(expression) => StartNode::Expression(expression),
            },
            StartNode::Pattern(pattern) => match pattern {
                Pattern::Object(pattern) => stored!(pattern),
                Pattern::Array(pattern) => stored!(pattern),
            },
            // Each of these concrete implementations reads its own stored Span.
            StartNode::Unary(value) => return Ok(value.span().start()),
            StartNode::Update(value) => return Ok(value.span().start()),
            StartNode::FunctionExpression(value) => return Ok(value.span().start()),
            StartNode::FunctionBody(value) => return Ok(value.span().start()),
            StartNode::PrivateName(value) => return Ok(value.span().start()),
            StartNode::Call(value) => return Ok(value.span().start()),
        };
    }
}

macro_rules! source_start {
    ($ty:ty, $variant:ident) => {
        impl private::Sealed for $ty {}
        impl SourceStart for $ty {
            fn try_start_position<E>(
                &self,
                admission: NameAdmission<'_, E>,
            ) -> Result<Position, E> {
                start(StartNode::$variant(self), admission)
            }
        }
    };
}

source_start!(Expression, Expression);
source_start!(Assign, Assign);
source_start!(PropertyAccessField, PropertyAccessField);
source_start!(Unary, Unary);
source_start!(Update, Update);
source_start!(FunctionExpression, FunctionExpression);
source_start!(FunctionBody, FunctionBody);
source_start!(PrivateName, PrivateName);
source_start!(Call, Call);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        LinearPosition, LinearSpan, Span, StatementList,
        expression::{
            Await, Identifier, ImportCall, ImportMeta, ImportPhase, New, NewTarget, Optional,
            OptionalOperation, OptionalOperationKind, Parenthesized, RegExpLiteral, Spread,
            SuperCall, TaggedTemplate, This, Yield,
            access::{PrivatePropertyAccess, SimplePropertyAccess, SuperPropertyAccess},
            literal::{
                ArrayLiteral, Literal, LiteralKind, ObjectLiteral, TemplateElement, TemplateLiteral,
            },
            operator::{
                Binary, BinaryInPrivate, Conditional,
                assign::AssignOp,
                binary::{ArithmeticOp, BinaryOp},
                unary::UnaryOp,
                update::{UpdateOp, UpdateTarget},
            },
        },
        function::{
            ArrowFunction, AsyncArrowFunction, AsyncFunctionExpression, AsyncGeneratorExpression,
            ClassExpression, FormalParameterList, GeneratorExpression,
        },
        pattern::{ArrayPattern, ObjectPattern},
    };
    use surgeist_js_interner::Sym;

    fn position(column: u32) -> Position {
        Position::new(7, column)
    }
    fn span(column: u32) -> Span {
        Span::new(position(column), position(column + 1))
    }
    fn outer_span() -> Span {
        Span::new(position(2), position(99))
    }
    fn linear_span() -> LinearSpan {
        LinearSpan::new(LinearPosition::new(0), LinearPosition::new(100))
    }
    fn literal(column: u32) -> Expression {
        Literal::new(LiteralKind::Int(1), span(column)).into()
    }
    fn identifier(column: u32) -> Identifier {
        Identifier::new(Sym::EVAL, span(column))
    }
    fn body() -> FunctionBody {
        FunctionBody::new(
            StatementList::default(),
            Span::new(position(30), position(90)),
        )
    }
    fn function() -> FunctionExpression {
        FunctionExpression::new(
            None,
            FormalParameterList::default(),
            body(),
            Some(linear_span()),
            false,
            outer_span(),
        )
    }
    fn call() -> Call {
        Call::new(literal(20), Box::new([literal(40)]), outer_span())
    }
    fn binary(left: Expression, right: Expression) -> Expression {
        Binary::new(BinaryOp::Arithmetic(ArithmeticOp::Add), left, right).into()
    }

    /// Expected positions are authored above, not obtained from the query under test.
    /// Every request position is also a refusal boundary, including one short.
    fn check<N: SourceStart>(node: &N, expected: Position, visits: usize) {
        #[derive(Debug)]
        struct Refusal;
        let original = Refusal;
        assert_eq!(node.span().start(), expected); // Independent legacy correspondence check.
        assert_eq!(node.try_start_position::<()>(None), Ok(expected));
        for limit in 0..=visits {
            let mut requests = Vec::new();
            let mut stopped = false;
            let mut admission = |work| {
                assert!(!stopped, "source-start inspection continued after refusal");
                requests.push(work);
                assert_eq!(work, NameWork::Visit);
                if requests.len() > limit {
                    stopped = true;
                    Err(&original)
                } else {
                    Ok(())
                }
            };
            let result = node.try_start_position(Some(&mut admission));
            if limit == visits {
                assert_eq!(result.expect("exact visits"), expected);
                assert_eq!(requests.len(), visits);
            } else {
                assert!(std::ptr::eq(
                    result.expect_err("next typed node refused"),
                    &original
                ));
                assert_eq!(requests.len(), limit + 1);
            }
        }
        assert_eq!(node.try_start_position::<()>(None), Ok(expected));
    }

    #[test]
    fn every_stored_expression_variant_keeps_its_authored_start_without_reading_children() {
        let stored = outer_span();
        let cases: Vec<Expression> = vec![
            This::new(stored).into(),
            Identifier::new(Sym::EVAL, stored).into(),
            Literal::new(LiteralKind::Int(1), stored).into(),
            RegExpLiteral::new(Sym::EVAL, Sym::EMPTY_STRING, stored).into(),
            ArrayLiteral::new(vec![Some(literal(20))], false, stored).into(),
            ObjectLiteral::new(
                Vec::<crate::expression::literal::PropertyDefinition>::new(),
                stored,
            )
            .into(),
            Spread::new(literal(20), stored).into(),
            function().into(),
            ArrowFunction::new(
                None,
                FormalParameterList::default(),
                body(),
                linear_span(),
                stored,
            )
            .into(),
            AsyncArrowFunction::new(
                None,
                FormalParameterList::default(),
                body(),
                linear_span(),
                stored,
            )
            .into(),
            GeneratorExpression::new(
                None,
                FormalParameterList::default(),
                body(),
                linear_span(),
                false,
                stored,
            )
            .into(),
            AsyncFunctionExpression::new(
                None,
                FormalParameterList::default(),
                body(),
                linear_span(),
                false,
                stored,
            )
            .into(),
            AsyncGeneratorExpression::new(
                None,
                FormalParameterList::default(),
                body(),
                linear_span(),
                false,
                stored,
            )
            .into(),
            ClassExpression::new(None, Some(literal(20)), None, Box::new([]), false, stored).into(),
            TemplateLiteral::new(
                Box::new([
                    TemplateElement::String(Sym::EMPTY_STRING),
                    TemplateElement::Expr(literal(20)),
                    TemplateElement::String(Sym::EMPTY_STRING),
                ]),
                stored,
            )
            .into(),
            call().into(),
            SuperCall::new(vec![literal(20)], stored).into(),
            ImportCall::new(
                literal(20),
                Some(literal(40)),
                ImportPhase::Evaluation,
                stored,
            )
            .into(),
            Optional::new(
                literal(20),
                Box::new([OptionalOperation::new(
                    OptionalOperationKind::SimplePropertyAccess {
                        field: identifier(40).into(),
                    },
                    true,
                    span(40),
                )]),
                stored,
            )
            .into(),
            TaggedTemplate::new(
                literal(20),
                Box::new([Sym::EMPTY_STRING]),
                Box::new([Some(Sym::EMPTY_STRING)]),
                Box::new([]),
                41,
                stored,
            )
            .into(),
            NewTarget::new(stored).into(),
            ImportMeta::new(stored).into(),
            Unary::new(UnaryOp::Delete, literal(20), stored).into(),
            Update::new(
                UpdateOp::IncrementPre,
                UpdateTarget::Identifier(identifier(20)),
                stored,
            )
            .into(),
            Await::new(Box::new(literal(20)), stored).into(),
            Yield::new(Some(literal(20)), false, stored).into(),
            Parenthesized::new(literal(20), stored).into(),
        ];
        for expression in cases {
            check(&expression, position(2), 2);
        }
    }

    #[test]
    fn derived_expression_variants_follow_only_the_start_determining_operand() {
        let cases = [
            (binary(literal(4), literal(80)), position(4), 4),
            (
                Expression::from(Assign::new(
                    AssignOp::Assign,
                    identifier(4).into(),
                    literal(80),
                )),
                position(4),
                4,
            ),
            (
                Expression::from(BinaryInPrivate::new(
                    PrivateName::new(Sym::EVAL, span(4)),
                    literal(80),
                )),
                position(4),
                3,
            ),
            (
                Expression::from(Conditional::new(literal(4), literal(40), literal(80))),
                position(4),
                4,
            ),
            (Expression::from(New::from(call())), position(2), 3),
            (
                Expression::from(PropertyAccess::Simple(SimplePropertyAccess::new(
                    literal(4),
                    PropertyAccessField::Expr(Box::new(literal(80))),
                ))),
                position(4),
                5,
            ),
            (
                Expression::from(PropertyAccess::Private(PrivatePropertyAccess::new(
                    literal(20),
                    PrivateName::new(Sym::EVAL, span(40)),
                    outer_span(),
                ))),
                position(2),
                3,
            ),
            (
                Expression::from(PropertyAccess::Super(SuperPropertyAccess::new(
                    PropertyAccessField::Expr(Box::new(literal(40))),
                    outer_span(),
                ))),
                position(2),
                3,
            ),
        ];
        for (expression, expected, visits) in cases {
            check(&expression, expected, visits);
        }
    }

    #[test]
    fn all_direct_guard_input_types_and_transparent_borrows_keep_their_positions() {
        check(
            &Assign::new(AssignOp::Assign, identifier(4).into(), literal(80)),
            position(4),
            3,
        );
        check(
            &Unary::new(UnaryOp::Delete, literal(20), outer_span()),
            position(2),
            1,
        );
        check(
            &Update::new(
                UpdateOp::IncrementPre,
                UpdateTarget::Identifier(identifier(20)),
                outer_span(),
            ),
            position(2),
            1,
        );
        check(
            &Update::new(
                UpdateOp::IncrementPost,
                UpdateTarget::Identifier(identifier(4)),
                Span::new(position(4), position(8)),
            ),
            position(4),
            1,
        );
        let function = function();
        check(&&function, position(2), 1);
        check(&body(), position(30), 1);
        check(&PropertyAccessField::Const(identifier(4)), position(4), 2);
        check(
            &PropertyAccessField::Expr(Box::new(binary(literal(4), literal(80)))),
            position(4),
            5,
        );
        check(&PrivateName::new(Sym::EVAL, span(4)), position(4), 1);
        check(&call(), position(2), 1);
        check(&literal(4), position(4), 2);
    }

    #[test]
    fn assignment_targets_preserve_property_and_pattern_starts() {
        let targets = [
            (AssignTarget::Identifier(identifier(4)), 3),
            (
                AssignTarget::Access(PropertyAccess::Simple(SimplePropertyAccess::new(
                    literal(4),
                    identifier(30),
                ))),
                6,
            ),
            (
                AssignTarget::Access(PropertyAccess::Private(PrivatePropertyAccess::new(
                    literal(20),
                    PrivateName::new(Sym::EVAL, span(30)),
                    Span::new(position(4), position(40)),
                ))),
                4,
            ),
            (
                AssignTarget::Access(PropertyAccess::Super(SuperPropertyAccess::new(
                    identifier(30).into(),
                    Span::new(position(4), position(40)),
                ))),
                4,
            ),
            (
                AssignTarget::Pattern(Pattern::Object(ObjectPattern::new(Box::new([]), span(4)))),
                4,
            ),
            (
                AssignTarget::Pattern(Pattern::Array(ArrayPattern::new(Box::new([]), span(4)))),
                4,
            ),
        ];
        for (target, visits) in targets {
            check(
                &Assign::new(AssignOp::Assign, target, literal(80)),
                position(4),
                visits,
            );
        }
    }

    fn left_spine(length: usize) -> Expression {
        let mut expression = literal(4);
        for _ in 0..length {
            expression = binary(expression, literal(80));
        }
        expression
    }

    // The AST's ordinary recursive destructor is independent of the query's
    // stack usage. Dismantle this deliberately deep test fixture iteratively.
    fn drop_left_spine(mut expression: Expression) {
        while let Expression::Binary(binary) = &mut expression {
            expression = std::mem::replace(binary.lhs_mut(), literal(4));
        }
    }

    #[test]
    fn deep_start_spine_uses_an_iterative_query_and_rhs_or_parentheses_do_not_expand_it() {
        const DEPTH: usize = 16_384;
        let expression = left_spine(DEPTH);
        let mut visits = 0;
        let mut admission = |work| {
            assert_eq!(work, NameWork::Visit);
            visits += 1;
            Ok::<_, ()>(())
        };
        assert_eq!(
            expression.try_start_position(Some(&mut admission)),
            Ok(position(4))
        );
        assert_eq!(visits, 2 * DEPTH + 2);
        drop_left_spine(expression);

        let mut expression = binary(literal(4), left_spine(DEPTH));
        let mut visits = 0;
        let mut admission = |work| {
            assert_eq!(work, NameWork::Visit);
            visits += 1;
            Ok::<_, ()>(())
        };
        assert_eq!(
            expression.try_start_position(Some(&mut admission)),
            Ok(position(4))
        );
        assert_eq!(visits, 4);
        if let Expression::Binary(binary) = &mut expression {
            drop_left_spine(std::mem::replace(binary.rhs_mut(), literal(80)));
        }

        let mut expression =
            Expression::Parenthesized(Parenthesized::new(left_spine(DEPTH), outer_span()));
        let mut visits = 0;
        let mut admission = |work| {
            assert_eq!(work, NameWork::Visit);
            visits += 1;
            Ok::<_, ()>(())
        };
        assert_eq!(
            expression.try_start_position(Some(&mut admission)),
            Ok(position(2))
        );
        assert_eq!(visits, 2);
        if let Expression::Parenthesized(parenthesized) = &mut expression {
            let nested = std::mem::replace(&mut parenthesized.expression, Box::new(literal(20)));
            drop_left_spine(*nested);
        }
    }
}
