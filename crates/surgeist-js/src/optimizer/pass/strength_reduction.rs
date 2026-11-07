use crate::{
    JsResult,
    optimizer::{
        PassAction,
        control::{OptimizerAdmission, OptimizerWork, admit},
        retirement,
    },
};
use surgeist_js_ast::{
    Expression, Spanned,
    expression::operator::{
        Binary,
        binary::{ArithmeticOp, BinaryOp},
    },
};

#[derive(Debug, Default)]
pub(crate) struct StrengthReduction;

impl StrengthReduction {
    pub(crate) fn reduce_expression(
        expr: &mut Expression,
        admission: &mut OptimizerAdmission<'_>,
    ) -> JsResult<PassAction<Expression>> {
        admit(admission, OptimizerWork::ReductionAttempt)?;
        match expr {
            Expression::Binary(binary)
                if binary.op() == BinaryOp::Arithmetic(ArithmeticOp::Div) =>
            {
                Self::try_reduce_div(binary, admission)
            }
            _ => Ok(PassAction::Keep),
        }
    }

    fn as_literal_int(expr: &Expression) -> Option<i32> {
        if let Expression::Literal(lit) = expr
            && let surgeist_js_ast::expression::literal::LiteralKind::Int(v) = lit.kind()
        {
            return Some(*v);
        }
        None
    }

    fn try_reduce_div(
        binary: &mut Binary,
        admission: &mut OptimizerAdmission<'_>,
    ) -> JsResult<PassAction<Expression>> {
        if let Some(div_val) = Self::as_literal_int(binary.rhs())
            && div_val == 2
        {
            admit(admission, OptimizerWork::Rewrite)?;
            retirement::expression(binary.rhs(), admission)?;
            retirement::container(admission)?;
            retirement::container(admission)?;
            let span = binary.span();
            let lhs = std::mem::replace(
                binary.lhs_mut(),
                surgeist_js_ast::expression::literal::Literal::new(
                    surgeist_js_ast::expression::literal::LiteralKind::Undefined,
                    span,
                )
                .into(),
            );

            return Ok(PassAction::Replace(
                Binary::new(
                    BinaryOp::Arithmetic(ArithmeticOp::Mul),
                    lhs,
                    surgeist_js_ast::expression::literal::Literal::new(
                        surgeist_js_ast::expression::literal::LiteralKind::Num(0.5),
                        span,
                    )
                    .into(),
                )
                .into(),
            ));
        }

        Ok(PassAction::Keep)
    }
}
