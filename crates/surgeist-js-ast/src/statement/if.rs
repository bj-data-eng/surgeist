//! If statement

use crate::{
    expression::Expression,
    statement::Statement,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use core::{fmt::Write as _, ops::ControlFlow};
use surgeist_js_interner::{Interner, ToIndentedString, ToInternedString};

/// The `if` statement executes a statement if a specified condition is [`truthy`][truthy]. If
/// the condition is [`falsy`][falsy], another statement can be executed.
///
/// Multiple `if...else` statements can be nested to create an else if clause.
///
/// Note that there is no elseif (in one word) keyword in JavaScript.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-IfStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else
/// [truthy]: https://developer.mozilla.org/en-US/docs/Glossary/truthy
/// [falsy]: https://developer.mozilla.org/en-US/docs/Glossary/falsy
/// [expression]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Expressions
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct If {
    condition: Expression,
    body: Box<Statement>,
    else_node: Option<Box<Statement>>,
}

impl If {
    /// Gets the condition of the if statement.
    #[inline]
    #[must_use]
    pub const fn cond(&self) -> &Expression {
        &self.condition
    }

    /// Gets the body to execute if the condition is true.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &Statement {
        &self.body
    }

    /// Gets the `else` node, if it has one.
    #[inline]
    pub fn else_node(&self) -> Option<&Statement> {
        self.else_node.as_ref().map(Box::as_ref)
    }

    /// Creates an `If` AST node.
    #[inline]
    #[must_use]
    pub fn new(condition: Expression, body: Statement, else_node: Option<Statement>) -> Self {
        Self {
            condition,
            body: body.into(),
            else_node: else_node.map(Box::new),
        }
    }

    /// Decomposes this statement into its condition and owned branches.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (Expression, Statement, Option<Statement>) {
        (self.condition, *self.body, self.else_node.map(|node| *node))
    }
}

impl ToIndentedString for If {
    fn to_indented_string(&self, interner: &Interner, indent: usize) -> String {
        let mut buf = format!("if ({}) ", self.cond().to_interned_string(interner));
        match self.else_node() {
            Some(else_e) => {
                let _ = write!(
                    buf,
                    "{} else {}",
                    self.body().to_indented_string(interner, indent),
                    else_e.to_indented_string(interner, indent)
                );
            }
            None => {
                buf.push_str(&self.body().to_indented_string(interner, indent));
            }
        }
        buf
    }
}

impl From<If> for Statement {
    fn from(if_stm: If) -> Self {
        Self::If(if_stm)
    }
}

impl VisitWith for If {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        visitor.visit_expression(&self.condition)?;
        visitor.visit_statement(&self.body)?;
        if let Some(stmt) = &self.else_node {
            visitor.visit_statement(stmt)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        visitor.visit_expression_mut(&mut self.condition)?;
        visitor.visit_statement_mut(&mut self.body)?;
        if let Some(stmt) = &mut self.else_node {
            visitor.visit_statement_mut(stmt)?;
        }
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LinearPosition, LinearSpan, Span, Spanned, expression::literal::Literal};

    #[test]
    fn consuming_an_if_preserves_its_parts_and_source_tracking() {
        let span = Span::new((2, 3), (2, 8));
        let linear_span = LinearSpan::new(LinearPosition::new(10), LinearPosition::new(15));
        let condition = Literal::with_linear_span(true, span, linear_span).into();
        let body = Statement::Expression(Literal::new(7, span).into());
        let alternate = Statement::Expression(Literal::new(11, span).into());
        let node = If::new(condition, body.clone(), Some(alternate.clone()));

        let (condition, actual_body, actual_alternate) = node.into_parts();

        let Expression::Literal(condition) = condition else {
            panic!("the original condition must survive")
        };
        assert_eq!(condition.span(), span);
        assert_eq!(condition.linear_span(), linear_span);
        assert_eq!(actual_body, body);
        assert_eq!(actual_alternate, Some(alternate));
    }
}
