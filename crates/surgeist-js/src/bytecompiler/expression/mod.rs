//! Lowering for ECMAScript expressions.
//!
//! The functions in this module implement the bytecompiler side of expression evaluation. They
//! preserve the source language evaluation order while translating AST nodes into opcode sequences
//! that the VM can execute later. See the ECMAScript expression grammar and runtime semantics in
//! the [specification][spec].
//!
//! [spec]: https://tc39.es/ecma262/#sec-ecmascript-language-expressions

use crate::JsResult;
mod assign;
mod binary;
mod object_literal;
mod unary;
mod update;

use std::ops::Deref;

use super::{
    Access, CallResultDest, Callable, NodeKind, Register, ToJsString,
    control::{
        CompilerWork, primitive_error, reserve, size_overflow, thin_push, thin_with_capacity,
    },
};
use crate::{
    bytecompiler::{ByteCompiler, Literal},
    vm::{CallFrame, GeneratorResumeKind},
};
use surgeist_js_ast::{
    Expression,
    expression::{
        ImportPhase,
        access::{PropertyAccess, PropertyAccessField},
        literal::{
            Literal as AstLiteral, LiteralKind as AstLiteralKind, TemplateElement, TemplateLiteral,
        },
        operator::Conditional,
    },
};

impl ByteCompiler<'_, '_> {
    fn compile_literal(&mut self, lit: &AstLiteral, dst: &Register) -> JsResult<()> {
        match lit.kind() {
            AstLiteralKind::String(v) => {
                let value = v.try_to_js_string(&*self.interner, &mut self.control)?;
                self.emit_store_literal(Literal::String(value), dst)?;
            }
            AstLiteralKind::Int(v) => self.emit_store_integer(*v, dst)?,
            AstLiteralKind::Num(v) => self.emit_store_rational(*v, dst)?,
            AstLiteralKind::BigInt(v) => {
                let value = crate::value::primitive::try_copy_raw_bigint(v, &mut |work| {
                    self.control.admit(CompilerWork::Primitive(work))
                })
                .map_err(primitive_error)?;
                self.emit_store_literal(Literal::BigInt(value), dst)?;
            }
            AstLiteralKind::Bool(true) => self
                .bytecode
                .emit_store_true(dst.variable(), &mut self.control)?,
            AstLiteralKind::Bool(false) => self
                .bytecode
                .emit_store_false(dst.variable(), &mut self.control)?,
            AstLiteralKind::Null => self
                .bytecode
                .emit_store_null(dst.variable(), &mut self.control)?,
            AstLiteralKind::Undefined => self
                .bytecode
                .emit_store_undefined(dst.variable(), &mut self.control)?,
        }
        Ok(())
    }

    fn compile_conditional(&mut self, op: &Conditional, dst: &Register) -> JsResult<()> {
        self.compile_if_else(
            op.condition(),
            |compiler| compiler.compile_expr(op.if_true(), dst),
            |compiler| compiler.compile_expr(op.if_false(), dst),
        )?;
        Ok(())
    }

    fn compile_template_literal(
        &mut self,
        template_literal: &TemplateLiteral,
        dst: &Register,
    ) -> JsResult<()> {
        let mut registers = Vec::new();
        reserve(
            &mut registers,
            template_literal.elements().len(),
            &mut self.control,
        )?;
        for element in template_literal.elements() {
            self.control.admit(CompilerWork::Visit)?;
            let value = self.register_allocator.alloc(&mut self.control)?;
            match element {
                TemplateElement::String(s) => {
                    let string = s.try_to_js_string(&*self.interner, &mut self.control)?;
                    self.emit_store_literal(Literal::String(string), &value)?;
                }
                TemplateElement::Expr(expr) => {
                    self.compile_expr(expr, &value)?;
                }
            }
            self.control.admit(CompilerWork::MetadataWrite)?;
            self.control.admit(CompilerWork::Copy {
                bytes: std::mem::size_of::<Register>(),
            })?;
            registers.push(value);
        }

        let mut values = thin_with_capacity(registers.len(), &mut self.control)?;
        for reg in &registers {
            self.control.admit(CompilerWork::MetadataVisit)?;
            thin_push(&mut values, reg.variable(), &mut self.control)?;
        }
        self.bytecode
            .emit_concat_to_string(dst.variable(), values, &mut self.control)?;
        for reg in registers {
            self.control.admit(CompilerWork::MetadataVisit)?;
            self.register_allocator.dealloc(reg, &mut self.control)?;
        }
        Ok(())
    }

    pub(crate) fn compile_expr_impl(&mut self, expr: &Expression, dst: &Register) -> JsResult<()> {
        match expr {
            Expression::Literal(lit) => self.compile_literal(lit, dst)?,
            Expression::RegExpLiteral(regexp) => {
                let pattern_index = self.get_or_insert_name(regexp.pattern())?;
                let flags_index = self.get_or_insert_name(regexp.flags())?;
                self.bytecode.emit_store_regexp(
                    dst.variable(),
                    pattern_index.into(),
                    flags_index.into(),
                    &mut self.control,
                )?;
            }
            Expression::Unary(unary) => self.compile_unary(unary, dst)?,
            Expression::Update(update) => self.compile_update(update, dst, false)?,
            Expression::Binary(binary) => self.compile_binary(binary, dst)?,
            Expression::BinaryInPrivate(binary) => self.compile_binary_in_private(binary, dst)?,
            Expression::Assign(assign) => self.compile_assign(assign, dst)?,
            Expression::ObjectLiteral(object) => self.compile_object_literal(object, dst)?,
            Expression::Identifier(name) => {
                self.access_get(Access::Variable { name: *name }, dst)?
            }
            Expression::PropertyAccess(access) => {
                self.access_get(Access::Property { access }, dst)?
            }
            Expression::Conditional(op) => self.compile_conditional(op, dst)?,
            Expression::ArrayLiteral(literal) => {
                let dst_op = dst.variable();
                self.bytecode
                    .emit_store_new_array(dst_op, &mut self.control)?;

                for element in literal.as_ref() {
                    self.control.admit(CompilerWork::Visit)?;
                    match element {
                        Some(Expression::Spread(spread)) => {
                            let _depth = self.control.enter()?;
                            self.control.admit(CompilerWork::Visit)?;
                            let value = self.register_allocator.alloc(&mut self.control)?;
                            self.compile_expr(spread.target(), &value)?;
                            self.bytecode
                                .emit_get_iterator(value.variable(), &mut self.control)?;
                            self.bytecode
                                .emit_push_iterator_to_array(dst_op, &mut self.control)?;
                            self.register_allocator.dealloc(value, &mut self.control)?;
                        }
                        Some(elem) => {
                            self.compile_expr_operand(elem, |this, op| {
                                this.bytecode.emit_push_value_to_array(
                                    op,
                                    dst_op,
                                    &mut this.control,
                                )?;
                                Ok(())
                            })?;
                        }
                        None => {
                            self.bytecode
                                .emit_push_elision_to_array(dst_op, &mut self.control)?;
                        }
                    }
                }
            }
            Expression::This(_this) => self.access_get(Access::This, dst)?,
            Expression::Spread(spread) => self.compile_expr(spread.target(), dst)?,
            Expression::FunctionExpression(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, dst)?;
            }
            Expression::ArrowFunction(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, dst)?;
            }
            Expression::AsyncArrowFunction(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, dst)?;
            }
            Expression::GeneratorExpression(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, dst)?;
            }
            Expression::AsyncFunctionExpression(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, dst)?;
            }
            Expression::AsyncGeneratorExpression(function) => {
                self.function_with_binding(function.into(), NodeKind::Expression, dst)?;
            }
            Expression::Call(call) => {
                self.call(Callable::Call(call), CallResultDest::Register(dst))?;
            }
            Expression::New(new) => {
                self.call(Callable::New(new), CallResultDest::Register(dst))?;
            }
            Expression::TemplateLiteral(template_literal) => {
                self.compile_template_literal(template_literal, dst)?;
            }
            Expression::Await(expr) => {
                self.compile_expr(expr.target(), dst)?;
                self.bytecode
                    .emit_await(dst.variable(), &mut self.control)?;
                let resume_kind = self.register_allocator.alloc(&mut self.control)?;
                self.pop_into_register(&resume_kind)?;
                self.pop_into_register(dst)?;
                self.generator_next(dst, &resume_kind)?;
                self.register_allocator
                    .dealloc(resume_kind, &mut self.control)?;
            }
            Expression::Yield(r#yield) => {
                if let Some(expr) = r#yield.target() {
                    self.compile_expr(expr, dst)?;
                } else {
                    self.bytecode
                        .emit_store_undefined(dst.variable(), &mut self.control)?;
                }

                if !r#yield.delegate() {
                    self.r#yield(dst)?;
                    return Ok(());
                }

                // need to delegate to an inner iterator

                if self.is_async() {
                    self.bytecode
                        .emit_get_async_iterator(dst.variable(), &mut self.control)?;
                } else {
                    self.bytecode
                        .emit_get_iterator(dst.variable(), &mut self.control)?;
                }

                let resume_kind = self.register_allocator.alloc(&mut self.control)?;
                let is_return = self.register_allocator.alloc(&mut self.control)?;
                self.bytecode
                    .emit_store_undefined(dst.variable(), &mut self.control)?;
                self.emit_resume_kind(GeneratorResumeKind::Normal, &resume_kind)?;

                let start_address = self.next_opcode_location()?;

                let (return_method_undefined, throw_method_undefined) =
                    self.generator_delegate_next(dst, &resume_kind, &is_return)?;

                if self.is_async() {
                    self.bytecode
                        .emit_await(dst.variable(), &mut self.control)?;
                    self.pop_into_register(&resume_kind)?;
                    self.pop_into_register(dst)?;
                } else {
                    self.emit_resume_kind(GeneratorResumeKind::Normal, &resume_kind)?;
                }

                let (resume_return, resume_exit) =
                    self.generator_delegate_resume(dst, &resume_kind, &is_return)?;

                if self.is_async() {
                    self.bytecode
                        .emit_iterator_value(dst.variable(), &mut self.control)?;
                    self.async_generator_yield(dst, &resume_kind)?;
                } else {
                    self.bytecode
                        .emit_iterator_result(dst.variable(), &mut self.control)?;
                    self.bytecode
                        .emit_generator_yield(dst.variable(), &mut self.control)?;
                    self.pop_into_register(&resume_kind)?;
                    self.pop_into_register(dst)?;
                }
                self.bytecode.emit_jump(start_address, &mut self.control)?;

                self.register_allocator
                    .dealloc(resume_kind, &mut self.control)?;
                self.register_allocator
                    .dealloc(is_return, &mut self.control)?;

                self.patch_jump(return_method_undefined)?;
                self.patch_jump(resume_return)?;

                if self.is_async() {
                    self.bytecode
                        .emit_await(dst.variable(), &mut self.control)?;
                    self.bytecode.emit_pop(&mut self.control)?;
                } else {
                    self.push_from_register(dst)?;
                }
                self.close_active_iterators()?;

                self.r#return(true)?;

                self.patch_jump(throw_method_undefined)?;

                self.iterator_close(self.is_async())?;
                self.emit_type_error("iterator does not have a throw method")?;

                self.patch_jump(resume_exit)?;
            }
            Expression::TaggedTemplate(template) => {
                let this = self.register_allocator.alloc(&mut self.control)?;
                let function = self.register_allocator.alloc(&mut self.control)?;

                self.control.admit(CompilerWork::Search)?;
                match template.tag() {
                    Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                        let _depth = self.control.enter()?;
                        self.control.admit(CompilerWork::Visit)?;
                        self.compile_expr(access.target(), &this)?;
                        match access.field() {
                            PropertyAccessField::Const(ident) => {
                                self.emit_get_property_by_name(
                                    &function,
                                    None,
                                    &this,
                                    ident.sym(),
                                )?;
                            }
                            PropertyAccessField::Expr(field) => {
                                let key = self.register_allocator.alloc(&mut self.control)?;
                                self.compile_expr(field, &key)?;
                                self.bytecode.emit_get_property_by_value(
                                    function.variable(),
                                    key.variable(),
                                    this.variable(),
                                    this.variable(),
                                    &mut self.control,
                                )?;
                                self.register_allocator.dealloc(key, &mut self.control)?;
                            }
                        }
                    }
                    Expression::PropertyAccess(PropertyAccess::Private(access)) => {
                        let _depth = self.control.enter()?;
                        self.control.admit(CompilerWork::Visit)?;
                        let index = self.get_or_insert_private_name(access.field())?;
                        self.compile_expr(access.target(), &this)?;
                        self.bytecode.emit_get_private_field(
                            function.variable(),
                            this.variable(),
                            index.into(),
                            &mut self.control,
                        )?;
                    }
                    expr => {
                        self.bytecode
                            .emit_store_undefined(this.variable(), &mut self.control)?;
                        self.compile_expr(expr, &function)?;
                    }
                }

                self.push_from_register(&this)?;
                self.push_from_register(&function)?;

                self.register_allocator.dealloc(this, &mut self.control)?;
                self.register_allocator
                    .dealloc(function, &mut self.control)?;

                let site = template.identifier();
                let count = template.cookeds().len();
                u32::try_from(count).map_err(|_| size_overflow())?;
                let part_count = count.checked_mul(2).ok_or_else(size_overflow)?;
                let jump_label = self.template_lookup(dst, site)?;

                let mut part_registers = Vec::new();
                reserve(&mut part_registers, part_count, &mut self.control)?;

                for (cooked, raw) in template.cookeds().iter().zip(template.raws()) {
                    self.control.admit(CompilerWork::Visit)?;
                    let value = self.register_allocator.alloc(&mut self.control)?;
                    if let Some(cooked) = cooked {
                        let string = cooked.try_to_js_string(&*self.interner, &mut self.control)?;
                        self.emit_store_literal(Literal::String(string), &value)?;
                    } else {
                        self.bytecode
                            .emit_store_undefined(value.variable(), &mut self.control)?;
                    }
                    self.control.admit(CompilerWork::MetadataWrite)?;
                    self.control.admit(CompilerWork::Copy {
                        bytes: std::mem::size_of::<Register>(),
                    })?;
                    part_registers.push(value);
                    self.control.admit(CompilerWork::Visit)?;
                    let value = self.register_allocator.alloc(&mut self.control)?;
                    let string = raw.try_to_js_string(&*self.interner, &mut self.control)?;
                    self.emit_store_literal(Literal::String(string), &value)?;
                    self.control.admit(CompilerWork::MetadataWrite)?;
                    self.control.admit(CompilerWork::Copy {
                        bytes: std::mem::size_of::<Register>(),
                    })?;
                    part_registers.push(value);
                }

                let mut values = thin_with_capacity(part_count, &mut self.control)?;
                for r in &part_registers {
                    self.control.admit(CompilerWork::MetadataVisit)?;
                    thin_push(&mut values, r.index(), &mut self.control)?;
                }
                self.bytecode.emit_template_create(
                    site,
                    dst.variable(),
                    values,
                    &mut self.control,
                )?;
                for r in part_registers {
                    self.control.admit(CompilerWork::MetadataVisit)?;
                    self.register_allocator.dealloc(r, &mut self.control)?;
                }

                self.patch_jump(jump_label)?;
                self.push_from_register(dst)?;

                for expr in template.exprs() {
                    self.control.admit(CompilerWork::Visit)?;
                    self.compile_expr_to_stack(expr)?;
                }

                let argument_count = template
                    .exprs()
                    .len()
                    .checked_add(1)
                    .ok_or_else(size_overflow)?;
                let argument_count = u32::try_from(argument_count).map_err(|_| size_overflow())?;
                self.bytecode
                    .emit_call(argument_count.into(), &mut self.control)?;
                self.pop_into_register(dst)?;
            }
            Expression::ClassExpression(class) => {
                self.compile_class(class.deref().into(), Some(dst))?;
            }
            Expression::SuperCall(super_call) => {
                let value = self.register_allocator.alloc(&mut self.control)?;

                self.bytecode
                    .emit_get_function_object(value.variable(), &mut self.control)?;
                self.bytecode
                    .emit_get_prototype(value.variable(), &mut self.control)?;

                self.push_from_register(&CallFrame::undefined_register())?;
                self.push_from_register(&value)?;
                self.register_allocator.dealloc(value, &mut self.control)?;

                let mut contains_spread = false;
                for argument in super_call.arguments() {
                    self.control.admit(CompilerWork::Search)?;
                    if matches!(argument, Expression::Spread(_)) {
                        contains_spread = true;
                        break;
                    }
                }

                if contains_spread {
                    let array = self.register_allocator.alloc(&mut self.control)?;
                    let array_op = array.variable();

                    self.bytecode
                        .emit_store_new_array(array_op, &mut self.control)?;

                    for arg in super_call.arguments() {
                        self.control.admit(CompilerWork::Visit)?;
                        if let Expression::Spread(spread) = arg {
                            let _depth = self.control.enter()?;
                            self.control.admit(CompilerWork::Visit)?;
                            let value = self.register_allocator.alloc(&mut self.control)?;
                            self.compile_expr(spread.target(), &value)?;
                            self.bytecode
                                .emit_get_iterator(value.variable(), &mut self.control)?;
                            self.bytecode
                                .emit_push_iterator_to_array(array_op, &mut self.control)?;
                            self.register_allocator.dealloc(value, &mut self.control)?;
                        } else {
                            self.compile_expr_operand(arg, |this, op| {
                                this.bytecode.emit_push_value_to_array(
                                    op,
                                    array_op,
                                    &mut this.control,
                                )?;
                                Ok(())
                            })?;
                        }
                    }

                    self.push_from_register(&array)?;
                    self.register_allocator.dealloc(array, &mut self.control)?;
                } else {
                    for arg in super_call.arguments() {
                        self.control.admit(CompilerWork::Visit)?;
                        self.compile_expr_to_stack(arg)?;
                    }
                }

                if contains_spread {
                    self.bytecode.emit_super_call_spread(&mut self.control)?;
                } else {
                    let argument_count =
                        u32::try_from(super_call.arguments().len()).map_err(|_| size_overflow())?;
                    self.bytecode
                        .emit_super_call(argument_count.into(), &mut self.control)?;
                }
                self.pop_into_register(dst)?;
                self.bytecode
                    .emit_bind_this_value(dst.variable(), &mut self.control)?;
            }
            Expression::ImportCall(import) => {
                self.compile_expr(import.specifier(), dst)?;
                let options = self.register_allocator.alloc(&mut self.control)?;
                if let Some(opts) = import.options() {
                    self.compile_expr(opts, &options)?;
                } else {
                    self.bytecode
                        .emit_store_undefined(options.variable(), &mut self.control)?;
                }

                let phase: u32 = match import.phase() {
                    ImportPhase::Evaluation => 0,
                    ImportPhase::Defer => 1,
                    ImportPhase::Source => 2,
                };
                self.bytecode.emit_import_call(
                    dst.variable(),
                    options.variable(),
                    phase.into(),
                    &mut self.control,
                )?;
                self.register_allocator
                    .dealloc(options, &mut self.control)?;
            }
            Expression::NewTarget(_new_target) => {
                self.bytecode
                    .emit_new_target(dst.variable(), &mut self.control)?;
            }
            Expression::ImportMeta(_import_meta) => {
                self.bytecode
                    .emit_import_meta(dst.variable(), &mut self.control)?;
            }
            Expression::Optional(opt) => {
                let this = self.register_allocator.alloc(&mut self.control)?;
                self.compile_optional_preserve_this(opt, &this, dst)?;
                self.register_allocator.dealloc(this, &mut self.control)?;
            }
            Expression::Parenthesized(parenthesized) => {
                self.compile_expr(parenthesized.expression(), dst)?;
            }
        }
        Ok(())
    }
}
