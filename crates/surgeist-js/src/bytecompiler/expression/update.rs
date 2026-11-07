use crate::JsResult;
use crate::bytecompiler::control::CompilerWork;
use crate::bytecompiler::{
    Access, BindingAccessOpcode, BindingKind, ByteCompiler, Register, ToJsString,
};
use surgeist_js_ast::{
    expression::{
        access::{PropertyAccess, PropertyAccessField},
        operator::{Update, update::UpdateOp},
    },
    scope::BindingLocatorError,
};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_update(
        &mut self,
        update: &Update,
        dst: &Register,
        discard: bool,
    ) -> JsResult<()> {
        let mut position = self.position_guard(update)?;
        let compiler = &mut *position;
        let increment = matches!(
            update.op(),
            UpdateOp::IncrementPost | UpdateOp::IncrementPre
        );
        let post = matches!(
            update.op(),
            UpdateOp::IncrementPost | UpdateOp::DecrementPost
        );

        compiler.control.admit(CompilerWork::Search)?;
        match Access::from_update_target(update.target()) {
            Access::Variable { name } => {
                let name = name.try_to_js_string(&*compiler.interner, &mut compiler.control)?;
                compiler.control.admit(CompilerWork::Copy {
                    bytes: std::mem::size_of::<crate::JsString>(),
                })?;
                let binding = compiler.lexical_identifier_reference(name.clone())?;
                let is_lexical = binding.is_lexical();
                let index = compiler.get_binding(&binding)?;

                // Fast path: for mutable local bindings with (post/pre)-increment/decrement,
                // use the local register directly to avoid unnecessary Move instructions.
                //
                // Pre-increment (++i):
                //   Inc(dst, local); Move(local, dst) → 2 ops
                //
                // Post-increment (i++):
                //   Move(dst, local); Inc(local, local) → 2 ops
                //
                // Inc(local, local) works because Inc writes new to dst AFTER old to src,
                // so when dst==src the new value wins.
                //
                // Skip for const bindings — they must fall through to emit ThrowMutateImmutable.
                if is_lexical
                    && {
                        compiler.control.admit(CompilerWork::Copy {
                            bytes: std::mem::size_of::<crate::JsString>(),
                        })?;
                        compiler.lexical_set_mutable_binding(name.clone())?.is_ok()
                    }
                    && let BindingKind::Local(Some(local_reg)) = &index
                {
                    let local_op = (*local_reg).into();

                    if discard {
                        // Result unused — just increment in-place.
                        if increment {
                            compiler.bytecode.emit_inc(
                                local_op,
                                local_op,
                                &mut compiler.control,
                            )?;
                        } else {
                            compiler.bytecode.emit_dec(
                                local_op,
                                local_op,
                                &mut compiler.control,
                            )?;
                        }
                        position.finish()?;
                        return Ok(());
                    }

                    if post {
                        // Save old value to dst (post-increment returns old value).
                        compiler.bytecode.emit_move(
                            dst.variable(),
                            local_op,
                            &mut compiler.control,
                        )?;
                        // Increment in-place.
                        if increment {
                            compiler.bytecode.emit_inc(
                                local_op,
                                local_op,
                                &mut compiler.control,
                            )?;
                        } else {
                            compiler.bytecode.emit_dec(
                                local_op,
                                local_op,
                                &mut compiler.control,
                            )?;
                        }
                    } else {
                        if increment {
                            compiler.bytecode.emit_inc(
                                dst.variable(),
                                local_op,
                                &mut compiler.control,
                            )?;
                        } else {
                            compiler.bytecode.emit_dec(
                                dst.variable(),
                                local_op,
                                &mut compiler.control,
                            )?;
                        }
                        // Write the new value back to the local register.
                        compiler.bytecode.emit_move(
                            local_op,
                            dst.variable(),
                            &mut compiler.control,
                        )?;
                    }
                    position.finish()?;
                    return Ok(());
                }

                if is_lexical {
                    compiler.emit_binding_access(BindingAccessOpcode::GetName, &index, dst)?;
                } else {
                    compiler.emit_binding_access(
                        BindingAccessOpcode::GetNameAndLocator,
                        &index,
                        dst,
                    )?;
                }

                let value = compiler.register_allocator.alloc(&mut compiler.control)?;
                if increment {
                    compiler.bytecode.emit_inc(
                        value.variable(),
                        dst.variable(),
                        &mut compiler.control,
                    )?;
                } else {
                    compiler.bytecode.emit_dec(
                        value.variable(),
                        dst.variable(),
                        &mut compiler.control,
                    )?;
                }

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
                                &value,
                            )?;
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = compiler.get_or_insert_string(name)?;
                            compiler
                                .bytecode
                                .emit_throw_mutate_immutable(index.into(), &mut compiler.control)?;
                        }
                        Err(BindingLocatorError::Silent) => {}
                    }
                } else {
                    compiler.emit_binding_access(
                        BindingAccessOpcode::SetNameByLocator,
                        &index,
                        &value,
                    )?;
                }
                if !post && !discard {
                    compiler.bytecode.emit_move(
                        dst.variable(),
                        value.variable(),
                        &mut compiler.control,
                    )?;
                }

                compiler
                    .register_allocator
                    .dealloc(value, &mut compiler.control)?;
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => {
                    let object = compiler.register_allocator.alloc(&mut compiler.control)?;
                    compiler.compile_expr(access.target(), &object)?;

                    match access.field() {
                        PropertyAccessField::Const(ident) => {
                            compiler.emit_get_property_by_name(dst, None, &object, ident.sym())?;
                            let value = compiler.register_allocator.alloc(&mut compiler.control)?;
                            if increment {
                                compiler.bytecode.emit_inc(
                                    value.variable(),
                                    dst.variable(),
                                    &mut compiler.control,
                                )?;
                            } else {
                                compiler.bytecode.emit_dec(
                                    value.variable(),
                                    dst.variable(),
                                    &mut compiler.control,
                                )?;
                            }

                            compiler.emit_set_property_by_name(
                                &value,
                                None,
                                &object,
                                ident.sym(),
                            )?;

                            if !post {
                                compiler.bytecode.emit_move(
                                    dst.variable(),
                                    value.variable(),
                                    &mut compiler.control,
                                )?;
                            }

                            compiler
                                .register_allocator
                                .dealloc(object, &mut compiler.control)?;
                            compiler
                                .register_allocator
                                .dealloc(value, &mut compiler.control)?;
                        }
                        PropertyAccessField::Expr(expr) => {
                            let key = compiler.register_allocator.alloc(&mut compiler.control)?;
                            compiler.compile_expr(expr, &key)?;

                            compiler.bytecode.emit_get_property_by_value_push(
                                dst.variable(),
                                key.variable(),
                                object.variable(),
                                object.variable(),
                                &mut compiler.control,
                            )?;

                            let value = compiler.register_allocator.alloc(&mut compiler.control)?;
                            if increment {
                                compiler.bytecode.emit_inc(
                                    value.variable(),
                                    dst.variable(),
                                    &mut compiler.control,
                                )?;
                            } else {
                                compiler.bytecode.emit_dec(
                                    value.variable(),
                                    dst.variable(),
                                    &mut compiler.control,
                                )?;
                            }

                            compiler.bytecode.emit_set_property_by_value(
                                value.variable(),
                                key.variable(),
                                object.variable(),
                                object.variable(),
                                &mut compiler.control,
                            )?;

                            if !post {
                                compiler.bytecode.emit_move(
                                    dst.variable(),
                                    value.variable(),
                                    &mut compiler.control,
                                )?;
                            }

                            compiler
                                .register_allocator
                                .dealloc(key, &mut compiler.control)?;
                            compiler
                                .register_allocator
                                .dealloc(object, &mut compiler.control)?;
                            compiler
                                .register_allocator
                                .dealloc(value, &mut compiler.control)?;
                        }
                    }
                }
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

                    let value = compiler.register_allocator.alloc(&mut compiler.control)?;
                    if increment {
                        compiler.bytecode.emit_inc(
                            value.variable(),
                            dst.variable(),
                            &mut compiler.control,
                        )?;
                    } else {
                        compiler.bytecode.emit_dec(
                            value.variable(),
                            dst.variable(),
                            &mut compiler.control,
                        )?;
                    }
                    compiler.bytecode.emit_set_private_field(
                        value.variable(),
                        object.variable(),
                        index.into(),
                        &mut compiler.control,
                    )?;

                    if !post {
                        compiler.bytecode.emit_move(
                            dst.variable(),
                            value.variable(),
                            &mut compiler.control,
                        )?;
                    }

                    compiler
                        .register_allocator
                        .dealloc(value, &mut compiler.control)?;
                    compiler
                        .register_allocator
                        .dealloc(object, &mut compiler.control)?;
                }
                PropertyAccess::Super(access) => match access.field() {
                    PropertyAccessField::Const(ident) => {
                        let object = compiler.register_allocator.alloc(&mut compiler.control)?;
                        let receiver = compiler.register_allocator.alloc(&mut compiler.control)?;
                        compiler.super_(&receiver, &object)?;

                        compiler.emit_get_property_by_name(
                            dst,
                            Some(&receiver),
                            &object,
                            ident.sym(),
                        )?;

                        let value = compiler.register_allocator.alloc(&mut compiler.control)?;
                        if increment {
                            compiler.bytecode.emit_inc(
                                value.variable(),
                                dst.variable(),
                                &mut compiler.control,
                            )?;
                        } else {
                            compiler.bytecode.emit_dec(
                                value.variable(),
                                dst.variable(),
                                &mut compiler.control,
                            )?;
                        }

                        compiler.emit_set_property_by_name(
                            &value,
                            Some(&receiver),
                            &object,
                            ident.sym(),
                        )?;
                        if !post {
                            compiler.bytecode.emit_move(
                                dst.variable(),
                                value.variable(),
                                &mut compiler.control,
                            )?;
                        }

                        compiler
                            .register_allocator
                            .dealloc(receiver, &mut compiler.control)?;
                        compiler
                            .register_allocator
                            .dealloc(object, &mut compiler.control)?;
                        compiler
                            .register_allocator
                            .dealloc(value, &mut compiler.control)?;
                    }
                    PropertyAccessField::Expr(expr) => {
                        let object = compiler.register_allocator.alloc(&mut compiler.control)?;
                        let receiver = compiler.register_allocator.alloc(&mut compiler.control)?;
                        compiler.super_(&receiver, &object)?;

                        let key = compiler.register_allocator.alloc(&mut compiler.control)?;
                        compiler.compile_expr(expr, &key)?;

                        compiler.bytecode.emit_get_property_by_value(
                            dst.variable(),
                            key.variable(),
                            receiver.variable(),
                            object.variable(),
                            &mut compiler.control,
                        )?;
                        if increment {
                            compiler.bytecode.emit_inc(
                                dst.variable(),
                                dst.variable(),
                                &mut compiler.control,
                            )?;
                        } else {
                            compiler.bytecode.emit_dec(
                                dst.variable(),
                                dst.variable(),
                                &mut compiler.control,
                            )?;
                        }
                        compiler.bytecode.emit_set_property_by_value(
                            dst.variable(),
                            key.variable(),
                            receiver.variable(),
                            object.variable(),
                            &mut compiler.control,
                        )?;

                        compiler
                            .register_allocator
                            .dealloc(receiver, &mut compiler.control)?;
                        compiler
                            .register_allocator
                            .dealloc(object, &mut compiler.control)?;
                        compiler
                            .register_allocator
                            .dealloc(key, &mut compiler.control)?;
                    }
                },
            },
            Access::This => unreachable!(),
        }
        position.finish()?;
        Ok(())
    }
}
