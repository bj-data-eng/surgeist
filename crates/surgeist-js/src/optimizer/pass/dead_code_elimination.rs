use crate::{
    JsError, JsResult,
    optimizer::{
        control::{OptimizerAdmission, OptimizerWork, admit},
        retirement,
    },
};
use core::ops::ControlFlow;
use surgeist_js_ast::{
    Expression, expression::literal::LiteralKind, statement::Statement, visitor::Visitor,
};

enum DeclarationSearchBreak {
    Found,
    Refused(JsError),
}

struct ContainsHoistedDeclarationsVisitor<'borrow, 'host> {
    admission: &'borrow mut OptimizerAdmission<'host>,
}

impl ContainsHoistedDeclarationsVisitor<'_, '_> {
    fn found(&mut self) -> ControlFlow<DeclarationSearchBreak> {
        self.admit_visit()?;
        ControlFlow::Break(DeclarationSearchBreak::Found)
    }
}

impl<'ast> Visitor<'ast> for ContainsHoistedDeclarationsVisitor<'_, '_> {
    type BreakTy = DeclarationSearchBreak;

    fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
        match admit(self.admission, OptimizerWork::DeclarationVisit) {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => ControlFlow::Break(DeclarationSearchBreak::Refused(error)),
        }
    }

    fn visit_var_declaration(
        &mut self,
        _: &'ast surgeist_js_ast::declaration::VarDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.found()
    }

    fn visit_function_declaration(
        &mut self,
        _: &'ast surgeist_js_ast::function::FunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.found()
    }

    fn visit_generator_declaration(
        &mut self,
        _: &'ast surgeist_js_ast::function::GeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.found()
    }

    fn visit_async_function_declaration(
        &mut self,
        _: &'ast surgeist_js_ast::function::AsyncFunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.found()
    }

    fn visit_async_generator_declaration(
        &mut self,
        _: &'ast surgeist_js_ast::function::AsyncGeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.found()
    }
}

#[derive(Debug, Default)]
pub(crate) struct DeadCodeElimination;

impl DeadCodeElimination {
    fn as_literal_bool(expr: &Expression) -> Option<bool> {
        if let Expression::Literal(lit) = expr
            && let LiteralKind::Bool(v) = lit.kind()
        {
            return Some(*v);
        }
        None
    }

    fn contains_hoisted_declarations(
        stmt: &Statement,
        admission: &mut OptimizerAdmission<'_>,
    ) -> JsResult<bool> {
        match (ContainsHoistedDeclarationsVisitor { admission }).visit_statement(stmt) {
            ControlFlow::Continue(()) => Ok(false),
            ControlFlow::Break(DeclarationSearchBreak::Found) => Ok(true),
            ControlFlow::Break(DeclarationSearchBreak::Refused(error)) => Err(error),
        }
    }

    /// Replaces only after every declaration search and cleanup reservation has
    /// succeeded. A refused admission leaves this statement unchanged.
    pub(crate) fn try_eliminate(
        node: &mut Statement,
        admission: &mut OptimizerAdmission<'_>,
    ) -> JsResult<bool> {
        match node {
            Statement::If(if_stmt) => {
                let Some(cond_value) = Self::as_literal_bool(if_stmt.cond()) else {
                    return Ok(false);
                };
                let discarded = if cond_value {
                    if_stmt.else_node()
                } else {
                    Some(if_stmt.body())
                };
                if let Some(discarded) = discarded
                    && Self::contains_hoisted_declarations(discarded, admission)?
                {
                    return Ok(false);
                }
                admit(admission, OptimizerWork::Rewrite)?;
                if admission.is_some() {
                    // Statement::If and its If payload disappear; only the
                    // selected branch survives the consuming decomposition.
                    retirement::container(admission)?;
                    retirement::container(admission)?;
                    retirement::expression(if_stmt.cond(), admission)?;
                    if let Some(discarded) = discarded {
                        retirement::statement(discarded, admission)?;
                    }
                }
                let Statement::If(if_stmt) = core::mem::replace(node, Statement::Empty) else {
                    unreachable!("the inspected statement is an if statement")
                };
                let (_, body, alternate) = if_stmt.into_parts();
                *node = if cond_value {
                    body
                } else {
                    alternate.unwrap_or(Statement::Empty)
                };
                Ok(true)
            }
            Statement::WhileLoop(while_loop) => {
                if Self::as_literal_bool(while_loop.condition()) != Some(false)
                    || Self::contains_hoisted_declarations(while_loop.body(), admission)?
                {
                    return Ok(false);
                }
                admit(admission, OptimizerWork::Rewrite)?;
                retirement::statement(node, admission)?;
                *node = Statement::Empty;
                Ok(true)
            }
            Statement::ForLoop(for_loop) => {
                if for_loop.condition().and_then(Self::as_literal_bool) != Some(false)
                    || Self::contains_hoisted_declarations(for_loop.body(), admission)?
                    // Initializers can have observable effects even when the
                    // condition is false. Retain the original conservative rule.
                    || for_loop.init().is_some()
                {
                    return Ok(false);
                }
                admit(admission, OptimizerWork::Rewrite)?;
                retirement::statement(node, admission)?;
                *node = Statement::Empty;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::JsNativeError;
    use surgeist_js_ast::{
        LinearPosition, LinearSpan, Position, Span, Spanned, StatementList, StatementListItem,
        declaration::{VarDeclaration, Variable, VariableList},
        expression::{Identifier, TaggedTemplate, literal::Literal},
        statement::{
            Block, If,
            iteration::{ForLoop, WhileLoop},
        },
    };
    use surgeist_js_interner::Sym;

    fn boolean(value: bool) -> Expression {
        Literal::new(value, Span::new((1, 1), (1, 1))).into()
    }

    fn block(statements: Vec<Statement>) -> Statement {
        Block::from(StatementList::new(
            statements
                .into_iter()
                .map(StatementListItem::from)
                .collect::<Vec<_>>(),
            LinearPosition::default(),
            false,
        ))
        .into()
    }

    fn variable() -> Statement {
        VarDeclaration(
            VariableList::new(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::ARGUMENTS, Span::new((1, 1), (1, 1))),
                    None,
                )]
                .into_boxed_slice(),
            )
            .unwrap(),
        )
        .into()
    }

    fn refusal() -> JsError {
        JsNativeError::range()
            .with_message("optimizer admission refused")
            .into()
    }

    #[test]
    fn refusal_at_each_dce_boundary_preserves_the_statement() {
        let candidates: [Statement; 5] = [
            If::new(
                boolean(true),
                Statement::Empty,
                Some(block(vec![Statement::Empty; 4])),
            )
            .into(),
            If::new(
                boolean(false),
                block(vec![Statement::Empty; 4]),
                Some(Statement::Empty),
            )
            .into(),
            If::new(boolean(false), Statement::Empty, None).into(),
            WhileLoop::new(boolean(false), block(vec![Statement::Empty; 4])).into(),
            ForLoop::new(
                None,
                Some(boolean(false)),
                Some(boolean(true)),
                block(vec![Statement::Empty; 4]),
            )
            .into(),
        ];
        for original in candidates {
            let mut successful = original.clone();
            let mut admitted = 0;
            let mut allow = |_| {
                admitted += 1;
                Ok(())
            };
            assert!(
                DeadCodeElimination::try_eliminate(&mut successful, &mut Some(&mut allow)).unwrap()
            );
            for refused_index in 0..admitted {
                let mut statement = original.clone();
                let mut calls = 0;
                let mut admission = |_| {
                    let current = calls;
                    calls += 1;
                    if current == refused_index {
                        Err(refusal())
                    } else {
                        Ok(())
                    }
                };
                let error =
                    DeadCodeElimination::try_eliminate(&mut statement, &mut Some(&mut admission))
                        .unwrap_err();
                assert_eq!(
                    error.as_native().unwrap().message(),
                    "optimizer admission refused"
                );
                assert_eq!(calls, refused_index + 1);
                assert_eq!(statement, original);
            }
        }
    }

    #[test]
    fn first_and_last_hoisted_declarations_keep_the_discarded_branch() {
        for position in [0, 31] {
            let mut statements = vec![Statement::Empty; 32];
            statements[position] = variable();
            let discarded = block(statements);
            let candidates: [Statement; 4] = [
                If::new(boolean(true), Statement::Empty, Some(discarded.clone())).into(),
                If::new(boolean(false), discarded.clone(), None).into(),
                WhileLoop::new(boolean(false), discarded.clone()).into(),
                ForLoop::new(None, Some(boolean(false)), None, discarded).into(),
            ];
            for original in candidates {
                let mut statement = original.clone();
                let mut visits = 0;
                let mut allow = |work| {
                    assert!(matches!(work, OptimizerWork::DeclarationVisit));
                    visits += 1;
                    Ok(())
                };
                assert!(
                    !DeadCodeElimination::try_eliminate(&mut statement, &mut Some(&mut allow))
                        .unwrap()
                );
                assert_eq!(statement, original);
                assert!(visits > position);

                // Refusing even the final declaration's override is a host
                // failure, distinct from successfully finding that declaration.
                for refused_index in 0..visits {
                    let mut statement = original.clone();
                    let mut calls = 0;
                    let mut admission = |_| {
                        let current = calls;
                        calls += 1;
                        if current == refused_index {
                            Err(refusal())
                        } else {
                            Ok(())
                        }
                    };
                    assert!(
                        DeadCodeElimination::try_eliminate(
                            &mut statement,
                            &mut Some(&mut admission)
                        )
                        .is_err()
                    );
                    assert_eq!(calls, refused_index + 1);
                    assert_eq!(statement, original);
                }
            }
        }
    }

    #[test]
    fn retaining_a_branch_preserves_template_identity_and_source_metadata() {
        let span = Span::new(Position::new(3, 2), Position::new(3, 12));
        let linear_span = LinearSpan::new(LinearPosition::new(12), LinearPosition::new(17));
        let retained = Statement::Expression(
            TaggedTemplate::new(
                Literal::with_linear_span(17, span, linear_span).into(),
                vec![Sym::ARGUMENTS].into_boxed_slice(),
                vec![Some(Sym::ARGUMENTS)].into_boxed_slice(),
                Box::default(),
                871,
                span,
            )
            .into(),
        );
        for condition in [false, true] {
            let mut statement = if condition {
                If::new(boolean(true), retained.clone(), Some(Statement::Empty)).into()
            } else {
                If::new(boolean(false), Statement::Empty, Some(retained.clone())).into()
            };
            assert!(DeadCodeElimination::try_eliminate(&mut statement, &mut None).unwrap());
            let Statement::Expression(Expression::TaggedTemplate(template)) = statement else {
                panic!("the selected tagged template must survive")
            };
            assert_eq!(template.identifier(), 871);
            assert_eq!(template.span(), span);
            let Expression::Literal(tag) = template.tag() else {
                panic!("the tag literal must survive")
            };
            assert_eq!(tag.span(), span);
            assert_eq!(tag.linear_span(), linear_span);
        }
    }

    #[test]
    fn cleanup_quotes_skip_retained_descendants() {
        let mut small = If::new(boolean(true), Statement::Empty, None).into();
        let mut large = If::new(boolean(true), block(vec![Statement::Empty; 1024]), None).into();
        let mut small_calls = 0;
        let mut allow = |_| {
            small_calls += 1;
            Ok(())
        };
        assert!(DeadCodeElimination::try_eliminate(&mut small, &mut Some(&mut allow)).unwrap());
        let mut remaining = small_calls;
        let mut bounded = |_| {
            if remaining == 0 {
                return Err(refusal());
            }
            remaining -= 1;
            Ok(())
        };
        assert!(DeadCodeElimination::try_eliminate(&mut large, &mut Some(&mut bounded)).unwrap());
        assert_eq!(remaining, 0);
        let Statement::Block(large) = large else {
            panic!("the selected block must survive")
        };
        assert_eq!(large.statement_list().statements().len(), 1024);
    }

    #[test]
    fn false_loops_keep_initializers_and_non_boolean_conditions() {
        let candidates: [Statement; 4] = [
            ForLoop::new(
                Some(boolean(true).into()),
                Some(boolean(false)),
                None,
                Statement::Empty,
            )
            .into(),
            ForLoop::new(None, None, None, Statement::Empty).into(),
            WhileLoop::new(boolean(true), Statement::Empty).into(),
            WhileLoop::new(
                Literal::new(0, Span::new((1, 1), (1, 1))).into(),
                Statement::Empty,
            )
            .into(),
        ];
        for original in candidates {
            let mut statement = original.clone();
            assert!(!DeadCodeElimination::try_eliminate(&mut statement, &mut None).unwrap());
            assert_eq!(statement, original);
        }
    }
}
