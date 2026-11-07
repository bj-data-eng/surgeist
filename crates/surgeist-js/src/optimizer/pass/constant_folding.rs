use crate::{
    JsBigInt, JsError, JsResult, JsString, JsValue,
    error::{EngineError, EngineResourceError},
    optimizer::{
        PassAction,
        control::{OptimizerAdmission, OptimizerWork, admit},
        retirement,
    },
    string::{JsStr, JsStrVariant, StringError},
    value::{
        JsVariant,
        primitive::{PrimitiveBinaryOp, PrimitiveError, PrimitiveUnaryOp, PrimitiveWork},
    },
};
use surgeist_js_ast::{
    Expression, Spanned,
    expression::{
        literal::{Literal, LiteralKind},
        operator::{
            Binary, Unary,
            binary::{ArithmeticOp, BinaryOp, BitwiseOp, LogicalOp, RelationalOp},
            unary::UnaryOp,
        },
    },
};
use surgeist_js_interner::{Interner, JStrRef};

fn resource(error: EngineResourceError) -> JsError {
    EngineError::Resource(error).into()
}

fn string_error(error: StringError<JsError>) -> JsError {
    match error {
        StringError::Refused(error) => error,
        StringError::SizeOverflow => resource(EngineResourceError::SizeOverflow),
        StringError::Allocation(_) => resource(EngineResourceError::Allocation),
    }
}

fn literal_to_value(
    literal: &Literal,
    interner: &Interner,
    admission: &mut OptimizerAdmission<'_>,
) -> JsResult<JsValue> {
    Ok(match literal.kind() {
        LiteralKind::String(symbol) => {
            let units = interner.resolve_expect(*symbol).utf16();
            let string = if admission.is_some() {
                JsString::try_from_str(JsStr::utf16(units), &mut |work| {
                    admit(
                        admission,
                        OptimizerWork::Primitive(PrimitiveWork::String(work)),
                    )
                })
                .map_err(string_error)?
            } else {
                JsString::from(JsStr::utf16(units))
            };
            JsValue::new(string)
        }
        LiteralKind::BigInt(value) => {
            let words = value.bits().div_ceil(32);
            admit(admission, OptimizerWork::LiteralCopy { words })?;
            admit(
                admission,
                OptimizerWork::LiteralCapacity {
                    elements: words
                        .checked_add(1)
                        .ok_or_else(|| resource(EngineResourceError::SizeOverflow))?,
                },
            )?;
            let result = JsValue::new(JsBigInt::new(value.clone()));
            admit(
                admission,
                OptimizerWork::Primitive(PrimitiveWork::Checkpoint),
            )?;
            result
        }
        LiteralKind::Num(value) => JsValue::new(*value),
        LiteralKind::Int(value) => JsValue::new(*value),
        LiteralKind::Bool(value) => JsValue::new(*value),
        LiteralKind::Null => JsValue::null(),
        LiteralKind::Undefined => JsValue::undefined(),
    })
}

fn value_to_literal(
    value: &JsValue,
    interner: &mut Interner,
    admission: &mut OptimizerAdmission<'_>,
) -> JsResult<LiteralKind> {
    Ok(match value.variant() {
        JsVariant::Null => LiteralKind::Null,
        JsVariant::Undefined => LiteralKind::Undefined,
        JsVariant::Boolean(value) => LiteralKind::Bool(value),
        JsVariant::String(value) => {
            // The interner owns hashing, comparison, conversion and publication.
            // A UTF16 view enters directly; Latin1 widening is a separate String
            // producer operation rather than an unaccounted consumer buffer.
            let symbol = if let JsStrVariant::Utf16(units) = value.variant() {
                interner.try_get_or_intern(JStrRef::Utf16(units), |work| {
                    admit(admission, OptimizerWork::Interner(work))
                })?
            } else {
                let units = if admission.is_some() {
                    value
                        .as_str()
                        .try_to_vec(&mut |work| {
                            admit(
                                admission,
                                OptimizerWork::Primitive(PrimitiveWork::String(work)),
                            )
                        })
                        .map_err(string_error)?
                } else {
                    value.to_vec()
                };
                interner.try_get_or_intern(JStrRef::Utf16(&units), |work| {
                    admit(admission, OptimizerWork::Interner(work))
                })?
            };
            LiteralKind::String(symbol)
        }
        JsVariant::Float64(value) => LiteralKind::Num(value),
        JsVariant::Integer32(value) => LiteralKind::Int(value),
        JsVariant::BigInt(value) => {
            let words = value.as_inner().bits().div_ceil(32);
            admit(admission, OptimizerWork::LiteralCopy { words })?;
            admit(
                admission,
                OptimizerWork::LiteralCapacity {
                    elements: words
                        .checked_add(1)
                        .ok_or_else(|| resource(EngineResourceError::SizeOverflow))?,
                },
            )?;
            let result = LiteralKind::BigInt(Box::new(value.as_inner().clone()));
            admit(
                admission,
                OptimizerWork::Primitive(PrimitiveWork::Checkpoint),
            )?;
            result
        }
        JsVariant::Object(_) | JsVariant::Symbol(_) => {
            unreachable!("literal operations produce neither objects nor symbols")
        }
    })
}

fn fold_result(result: Result<JsValue, PrimitiveError<JsError>>) -> JsResult<Option<JsValue>> {
    match result {
        Ok(value) => Ok(Some(value)),
        // A guest abrupt completion is evaluated at runtime. A host failure must
        // abort preparation and retain all accepted progress.
        Err(PrimitiveError::Abrupt(error)) if error.as_engine().is_none() => Ok(None),
        Err(PrimitiveError::Abrupt(error) | PrimitiveError::Refused(error)) => Err(error),
        Err(PrimitiveError::WorkOverflow) => Err(resource(EngineResourceError::SizeOverflow)),
        Err(PrimitiveError::Allocation(_)) => Err(resource(EngineResourceError::Allocation)),
    }
}

#[derive(Debug, Default)]
pub(crate) struct ConstantFolding;

impl ConstantFolding {
    pub(crate) fn fold_expression(
        expr: &mut Expression,
        interner: &mut Interner,
        admission: &mut OptimizerAdmission<'_>,
    ) -> JsResult<PassAction<Expression>> {
        admit(admission, OptimizerWork::FoldAttempt)?;
        match expr {
            Expression::Unary(unary) => Self::unary(unary, interner, admission),
            Expression::Binary(binary) => Self::binary(binary, interner, admission),
            _ => Ok(PassAction::Keep),
        }
    }

    fn unary(
        unary: &Unary,
        interner: &mut Interner,
        admission: &mut OptimizerAdmission<'_>,
    ) -> JsResult<PassAction<Expression>> {
        let Expression::Literal(literal) = unary.target() else {
            return Ok(PassAction::Keep);
        };
        let kind = match unary.op() {
            UnaryOp::Delete => LiteralKind::Bool(true),
            UnaryOp::Void => LiteralKind::Undefined,
            op => {
                let op = match op {
                    UnaryOp::Minus => PrimitiveUnaryOp::Minus,
                    UnaryOp::Plus => PrimitiveUnaryOp::Plus,
                    UnaryOp::Not => PrimitiveUnaryOp::Not,
                    UnaryOp::Tilde => PrimitiveUnaryOp::BitNot,
                    UnaryOp::TypeOf => PrimitiveUnaryOp::TypeOf,
                    UnaryOp::Delete | UnaryOp::Void => unreachable!("handled above"),
                };
                let input = literal_to_value(literal, interner, admission)?;
                let value = if admission.is_some() {
                    fold_result(input.primitive_unary(op, &mut |work| {
                        admit(admission, OptimizerWork::Primitive(work))
                    }))?
                } else {
                    match input.primitive_unary_unmetered(op) {
                        Ok(value) => Some(value),
                        Err(error) if error.as_engine().is_none() => None,
                        Err(error) => return Err(error),
                    }
                };
                let Some(value) = value else {
                    return Ok(PassAction::Keep);
                };
                value_to_literal(&value, interner, admission)?
            }
        };
        admit(admission, OptimizerWork::Rewrite)?;
        retirement::expression(unary.target(), admission)?;
        retirement::container(admission)?;
        retirement::container(admission)?;
        Ok(PassAction::Replace(Literal::new(kind, unary.span()).into()))
    }

    fn binary(
        binary: &mut Binary,
        interner: &mut Interner,
        admission: &mut OptimizerAdmission<'_>,
    ) -> JsResult<PassAction<Expression>> {
        let Expression::Literal(lhs_literal) = binary.lhs() else {
            return Ok(PassAction::Keep);
        };
        let span = binary.span();
        if binary.op() == BinaryOp::Comma {
            if !matches!(binary.rhs(), Expression::Literal(_)) {
                if lhs_literal.is_undefined() {
                    return Ok(PassAction::Keep);
                }
                admit(admission, OptimizerWork::Rewrite)?;
                retirement::expression(binary.lhs(), admission)?;
                *binary.lhs_mut() = Literal::new(LiteralKind::Undefined, span).into();
                return Ok(PassAction::Modified);
            }
            return Self::select(binary, false, admission);
        }
        if let BinaryOp::Logical(op) = binary.op() {
            // Truthiness is determined by scalar metadata: no materialization of
            // arbitrarily large strings/BigInts is required for branch selection.
            let truthy = match lhs_literal.kind() {
                LiteralKind::String(symbol) => !interner.resolve_expect(*symbol).utf16().is_empty(),
                LiteralKind::BigInt(value) => value.bits() != 0,
                LiteralKind::Num(value) => *value != 0.0 && !value.is_nan(),
                LiteralKind::Int(value) => *value != 0,
                LiteralKind::Bool(value) => *value,
                LiteralKind::Null | LiteralKind::Undefined => false,
            };
            let choose_left = match op {
                LogicalOp::And => !truthy,
                LogicalOp::Or => truthy,
                LogicalOp::Coalesce => !matches!(
                    lhs_literal.kind(),
                    LiteralKind::Null | LiteralKind::Undefined
                ),
            };
            return Self::select(binary, choose_left, admission);
        }
        let Expression::Literal(rhs_literal) = binary.rhs() else {
            return Ok(PassAction::Keep);
        };
        let op = match binary.op() {
            BinaryOp::Arithmetic(op) => match op {
                ArithmeticOp::Add => PrimitiveBinaryOp::Add,
                ArithmeticOp::Sub => PrimitiveBinaryOp::Sub,
                ArithmeticOp::Mul => PrimitiveBinaryOp::Mul,
                ArithmeticOp::Div => PrimitiveBinaryOp::Div,
                ArithmeticOp::Mod => PrimitiveBinaryOp::Rem,
                ArithmeticOp::Exp => PrimitiveBinaryOp::Pow,
            },
            BinaryOp::Bitwise(op) => match op {
                BitwiseOp::And => PrimitiveBinaryOp::BitAnd,
                BitwiseOp::Or => PrimitiveBinaryOp::BitOr,
                BitwiseOp::Xor => PrimitiveBinaryOp::BitXor,
                BitwiseOp::Shl => PrimitiveBinaryOp::Shl,
                BitwiseOp::Shr => PrimitiveBinaryOp::Shr,
                BitwiseOp::UShr => PrimitiveBinaryOp::Ushr,
            },
            BinaryOp::Relational(op) => match op {
                RelationalOp::In | RelationalOp::InstanceOf => return Ok(PassAction::Keep),
                RelationalOp::Equal => PrimitiveBinaryOp::Eq,
                RelationalOp::NotEqual => PrimitiveBinaryOp::NotEq,
                RelationalOp::StrictEqual => PrimitiveBinaryOp::StrictEq,
                RelationalOp::StrictNotEqual => PrimitiveBinaryOp::StrictNotEq,
                RelationalOp::LessThan => PrimitiveBinaryOp::Lt,
                RelationalOp::LessThanOrEqual => PrimitiveBinaryOp::Le,
                RelationalOp::GreaterThan => PrimitiveBinaryOp::Gt,
                RelationalOp::GreaterThanOrEqual => PrimitiveBinaryOp::Ge,
            },
            BinaryOp::Logical(_) | BinaryOp::Comma => {
                unreachable!("handled before materialization")
            }
        };
        let lhs = literal_to_value(lhs_literal, interner, admission)?;
        let rhs = literal_to_value(rhs_literal, interner, admission)?;
        let result = if admission.is_some() {
            fold_result(lhs.primitive_binary(op, &rhs, &mut |work| {
                admit(admission, OptimizerWork::Primitive(work))
            }))?
        } else {
            match lhs.primitive_binary_unmetered(op, &rhs) {
                Ok(value) => Some(value),
                Err(error) if error.as_engine().is_none() => None,
                Err(error) => return Err(error),
            }
        };
        let Some(value) = result else {
            return Ok(PassAction::Keep);
        };
        let kind = value_to_literal(&value, interner, admission)?;
        admit(admission, OptimizerWork::Rewrite)?;
        retirement::expression(binary.lhs(), admission)?;
        retirement::expression(binary.rhs(), admission)?;
        retirement::container(admission)?;
        retirement::container(admission)?;
        Ok(PassAction::Replace(Literal::new(kind, span).into()))
    }

    fn select(
        binary: &mut Binary,
        choose_left: bool,
        admission: &mut OptimizerAdmission<'_>,
    ) -> JsResult<PassAction<Expression>> {
        admit(admission, OptimizerWork::Rewrite)?;
        retirement::expression(
            if choose_left {
                binary.rhs()
            } else {
                binary.lhs()
            },
            admission,
        )?;
        retirement::container(admission)?;
        retirement::container(admission)?;
        let span = binary.span();
        let selected = if choose_left {
            binary.lhs_mut()
        } else {
            binary.rhs_mut()
        };
        Ok(PassAction::Replace(std::mem::replace(
            selected,
            Literal::new(LiteralKind::Undefined, span).into(),
        )))
    }
}
