use crate::JsResult;
use crate::bytecompiler::control::CompilerWork;
use crate::{
    bytecompiler::{Access, BindingAccessOpcode, ByteCompiler, Label, Register, ToJsString},
    vm::opcode::BindingOpcode,
};
use surgeist_js_ast::{
    Expression,
    expression::{
        access::{PropertyAccess, PropertyAccessField},
        operator::{Assign, assign::AssignOp},
    },
    scope::BindingLocatorError,
};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_assign(&mut self, assign: &Assign, dst: &Register) -> JsResult<()> {
        let mut position = self.position_guard(assign)?;
        let compiler = &mut *position;

        compiler.control.admit(CompilerWork::Search)?;
        if assign.op() == AssignOp::Assign {
            match Access::from_assign_target(assign.lhs()) {
                Ok(access) => {
                    compiler.access_set(access, |compiler| {
                        compiler.compile_expr(assign.rhs(), dst)?;
                        Ok(dst)
                    })?;
                }
                Err(pattern) => {
                    compiler.compile_expr(assign.rhs(), dst)?;
                    compiler.compile_declaration_pattern(pattern, BindingOpcode::SetName, dst)?;
                }
            }
        } else {
            let access = Access::from_assign_target(assign.lhs())
                .expect("patterns should throw early errors on complex assignment operators");

            let short_circuit = matches!(
                assign.op(),
                AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce
            );

            let emit = |compiler: &mut Self,
                        dst: &Register,
                        expr: &Expression,
                        op: AssignOp|
             -> JsResult<Option<Label>> {
                if short_circuit {
                    let next = compiler.next_opcode_location()?;
                    match op {
                        AssignOp::BoolAnd => compiler.bytecode.emit_logical_and(
                            Self::DUMMY_ADDRESS,
                            dst.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::BoolOr => compiler.bytecode.emit_logical_or(
                            Self::DUMMY_ADDRESS,
                            dst.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::Coalesce => compiler.bytecode.emit_coalesce(
                            Self::DUMMY_ADDRESS,
                            dst.variable(),
                            &mut compiler.control,
                        )?,
                        _ => unreachable!(),
                    }
                    compiler.compile_expr(expr, dst)?;
                    Ok(Some(Label { index: next }))
                } else {
                    let rhs = compiler.register_allocator.alloc(&mut compiler.control)?;
                    compiler.compile_expr(expr, &rhs)?;
                    match op {
                        AssignOp::Add => compiler.bytecode.emit_add(
                            dst.variable(),
                            dst.variable(),
                            rhs.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::Sub => compiler.bytecode.emit_sub(
                            dst.variable(),
                            dst.variable(),
                            rhs.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::Mul => compiler.bytecode.emit_mul(
                            dst.variable(),
                            dst.variable(),
                            rhs.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::Div => compiler.bytecode.emit_div(
                            dst.variable(),
                            dst.variable(),
                            rhs.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::Mod => compiler.bytecode.emit_mod(
                            dst.variable(),
                            dst.variable(),
                            rhs.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::Exp => compiler.bytecode.emit_pow(
                            dst.variable(),
                            dst.variable(),
                            rhs.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::And => compiler.bytecode.emit_bit_and(
                            dst.variable(),
                            dst.variable(),
                            rhs.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::Or => compiler.bytecode.emit_bit_or(
                            dst.variable(),
                            dst.variable(),
                            rhs.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::Xor => compiler.bytecode.emit_bit_xor(
                            dst.variable(),
                            dst.variable(),
                            rhs.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::Shl => compiler.bytecode.emit_shift_left(
                            dst.variable(),
                            dst.variable(),
                            rhs.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::Shr => compiler.bytecode.emit_shift_right(
                            dst.variable(),
                            dst.variable(),
                            rhs.variable(),
                            &mut compiler.control,
                        )?,
                        AssignOp::Ushr => compiler.bytecode.emit_unsigned_shift_right(
                            dst.variable(),
                            dst.variable(),
                            rhs.variable(),
                            &mut compiler.control,
                        )?,
                        _ => unreachable!(),
                    }
                    compiler
                        .register_allocator
                        .dealloc(rhs, &mut compiler.control)?;
                    Ok(None)
                }
            };

            let early_exit;

            match access {
                Access::Variable { name } => {
                    let name = name.try_to_js_string(&*compiler.interner, &mut compiler.control)?;

                    compiler.control.admit(CompilerWork::Copy {
                        bytes: std::mem::size_of::<crate::JsString>(),
                    })?;
                    let binding = compiler.lexical_identifier_reference(name.clone())?;
                    let is_lexical = binding.is_lexical();
                    let index = compiler.get_binding(&binding)?;

                    if is_lexical {
                        compiler.emit_binding_access(BindingAccessOpcode::GetName, &index, dst)?;
                    } else {
                        compiler.emit_binding_access(
                            BindingAccessOpcode::GetNameAndLocator,
                            &index,
                            dst,
                        )?;
                    }

                    early_exit = emit(compiler, dst, assign.rhs(), assign.op())?;

                    if is_lexical {
                        compiler.control.admit(CompilerWork::Copy {
                            bytes: std::mem::size_of::<crate::JsString>(),
                        })?;
                        match compiler.lexical_set_mutable_binding(name.clone())? {
                            Ok(binding) => {
                                let index = compiler.insert_binding(binding)?;
                                compiler.emit_binding_access(
                                    BindingAccessOpcode::SetName,
                                    &index,
                                    dst,
                                )?;
                            }
                            Err(BindingLocatorError::MutateImmutable) => {
                                let index = compiler.get_or_insert_string(name)?;
                                compiler.bytecode.emit_throw_mutate_immutable(
                                    index.into(),
                                    &mut compiler.control,
                                )?;
                            }
                            Err(BindingLocatorError::Silent) => {}
                        }
                    } else {
                        compiler.emit_binding_access(
                            BindingAccessOpcode::SetNameByLocator,
                            &index,
                            dst,
                        )?;
                    }
                }
                Access::Property { access } => match access {
                    PropertyAccess::Simple(access) => match access.field() {
                        PropertyAccessField::Const(name) => {
                            let object =
                                compiler.register_allocator.alloc(&mut compiler.control)?;
                            compiler.compile_expr(access.target(), &object)?;

                            compiler.emit_get_property_by_name(dst, None, &object, name.sym())?;

                            early_exit = emit(compiler, dst, assign.rhs(), assign.op())?;

                            compiler.emit_set_property_by_name(dst, None, &object, name.sym())?;

                            compiler
                                .register_allocator
                                .dealloc(object, &mut compiler.control)?;
                        }
                        PropertyAccessField::Expr(expr) => {
                            let object =
                                compiler.register_allocator.alloc(&mut compiler.control)?;
                            compiler.compile_expr(access.target(), &object)?;

                            let key = compiler.register_allocator.alloc(&mut compiler.control)?;
                            compiler.compile_expr(expr, &key)?;

                            compiler.bytecode.emit_get_property_by_value_push(
                                dst.variable(),
                                key.variable(),
                                object.variable(),
                                object.variable(),
                                &mut compiler.control,
                            )?;

                            early_exit = emit(compiler, dst, assign.rhs(), assign.op())?;

                            compiler.bytecode.emit_set_property_by_value(
                                dst.variable(),
                                key.variable(),
                                object.variable(),
                                object.variable(),
                                &mut compiler.control,
                            )?;

                            compiler
                                .register_allocator
                                .dealloc(key, &mut compiler.control)?;
                            compiler
                                .register_allocator
                                .dealloc(object, &mut compiler.control)?;
                        }
                    },
                    PropertyAccess::Private(access) => {
                        let index = compiler.get_or_insert_private_name(access.field())?;

                        let object = compiler.register_allocator.alloc(&mut compiler.control)?;
                        compiler.compile_expr(access.target(), &object)?;

                        compiler.bytecode.emit_get_private_field(
                            dst.variable(),
                            object.variable(),
                            index.into(),
                            &mut compiler.control,
                        )?;

                        early_exit = emit(compiler, dst, assign.rhs(), assign.op())?;

                        compiler.bytecode.emit_set_private_field(
                            dst.variable(),
                            object.variable(),
                            index.into(),
                            &mut compiler.control,
                        )?;

                        compiler
                            .register_allocator
                            .dealloc(object, &mut compiler.control)?;
                    }
                    PropertyAccess::Super(access) => match access.field() {
                        PropertyAccessField::Const(name) => {
                            let object =
                                compiler.register_allocator.alloc(&mut compiler.control)?;
                            let receiver =
                                compiler.register_allocator.alloc(&mut compiler.control)?;
                            compiler.super_(&receiver, &object)?;

                            compiler.emit_get_property_by_name(
                                dst,
                                Some(&receiver),
                                &object,
                                name.sym(),
                            )?;

                            early_exit = emit(compiler, dst, assign.rhs(), assign.op())?;

                            compiler.emit_set_property_by_name(
                                dst,
                                Some(&receiver),
                                &object,
                                name.sym(),
                            )?;

                            compiler
                                .register_allocator
                                .dealloc(receiver, &mut compiler.control)?;
                            compiler
                                .register_allocator
                                .dealloc(object, &mut compiler.control)?;
                        }
                        PropertyAccessField::Expr(expr) => {
                            let object =
                                compiler.register_allocator.alloc(&mut compiler.control)?;
                            let receiver =
                                compiler.register_allocator.alloc(&mut compiler.control)?;
                            compiler.super_(&receiver, &object)?;

                            let key = compiler.register_allocator.alloc(&mut compiler.control)?;
                            compiler.compile_expr(expr, &key)?;

                            compiler.bytecode.emit_get_property_by_value_push(
                                dst.variable(),
                                key.variable(),
                                receiver.variable(),
                                object.variable(),
                                &mut compiler.control,
                            )?;

                            early_exit = emit(compiler, dst, assign.rhs(), assign.op())?;

                            compiler.bytecode.emit_set_property_by_value(
                                dst.variable(),
                                key.variable(),
                                receiver.variable(),
                                object.variable(),
                                &mut compiler.control,
                            )?;

                            compiler
                                .register_allocator
                                .dealloc(key, &mut compiler.control)?;
                            compiler
                                .register_allocator
                                .dealloc(receiver, &mut compiler.control)?;
                            compiler
                                .register_allocator
                                .dealloc(object, &mut compiler.control)?;
                        }
                    },
                },
                Access::This => unreachable!(),
            }

            if let Some(early_exit) = early_exit {
                let skip = compiler.jump()?;
                compiler.patch_jump(early_exit)?;
                compiler.patch_jump(skip)?;
            }
        }
        position.finish()?;
        Ok(())
    }
}
