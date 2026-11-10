use crate::JsResult;
use std::rc::Rc;

use super::{
    BindingAccessOpcode, ByteCompiler, Literal, Register, ToJsString,
    control::{CompilerWork, payload_bytes, reserve, thin_push, thin_with_capacity},
    function::prepare_code_storage,
};
use crate::{
    js_string,
    vm::{CodeBlock, CodeBlockFlags, opcode::BindingOpcode},
};
use surgeist_js_ast::{
    Expression,
    expression::Identifier,
    function::{
        ClassDeclaration, ClassElement, ClassElementName, ClassExpression, FormalParameterList,
        FunctionExpression,
    },
    property::{MethodDefinitionKind, PropertyName},
    scope::ScopeHandle,
};
use surgeist_js_interner::Sym;

// Static class elements that are initialized at a later time in the class creation.
enum StaticElement {
    // A static class block with it's function code.
    StaticBlock(Rc<CodeBlock>),

    // A static class field with it's function code, an optional name index and the information if the function is an anonymous function.
    StaticField {
        code: Rc<CodeBlock>,
        name_index: StaticFieldName,
        is_anonymous_function: bool,
    },
}

enum StaticFieldName {
    PrivateName(u32),
    Index(u32),
    Register(Register),
}

/// Describes the complete specification of a class.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ClassSpec<'a> {
    name: Option<Identifier>,
    super_ref: Option<&'a Expression>,
    constructor: Option<&'a FunctionExpression>,
    elements: &'a [ClassElement],
    has_binding_identifier: bool,
    name_scope: Option<&'a ScopeHandle>,
}

impl<'a> From<&'a ClassDeclaration> for ClassSpec<'a> {
    fn from(class: &'a ClassDeclaration) -> Self {
        Self {
            name: Some(class.name()),
            super_ref: class.super_ref(),
            constructor: class.constructor(),
            elements: class.elements(),
            has_binding_identifier: true,
            name_scope: Some(class.name_scope()),
        }
    }
}

impl<'a> From<&'a ClassExpression> for ClassSpec<'a> {
    fn from(class: &'a ClassExpression) -> Self {
        Self {
            name: class.name(),
            super_ref: class.super_ref(),
            constructor: class.constructor(),
            elements: class.elements(),
            has_binding_identifier: class.name().is_some(),
            name_scope: class.name_scope(),
        }
    }
}

impl ByteCompiler<'_, '_> {
    /// This function compiles a class declaration or expression.
    ///
    /// The compilation of a class declaration and expression is mostly equal.
    /// A class declaration binds the resulting class object to it's identifier.
    /// A class expression leaves the resulting class object on the stack for following operations.
    pub(crate) fn compile_class(
        &mut self,
        class: ClassSpec<'_>,
        dst: Option<&Register>,
    ) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        // 11.2.2 Strict Mode Code - <https://tc39.es/ecma262/#sec-strict-mode-code>
        //  - All parts of a ClassDeclaration or a ClassExpression are strict mode code.
        let strict = self.strict();
        self.control.admit(CompilerWork::MetadataWrite)?;
        self.code_block_flags |= CodeBlockFlags::STRICT;

        let class_name_symbol = class.name.map_or(Sym::EMPTY_STRING, Identifier::sym);
        let class_name = class_name_symbol.try_to_js_string(self.interner, &mut self.control)?;

        let outer_scope = self.push_declarative_scope(class.name_scope)?;

        // The new span is not the same as the parent `ByteCompiler` have.
        let spanned_source_text = self.spanned_source_text.clone_only_source();
        let _constructor_depth = self.control.enter()?;
        let mut compiler = ByteCompiler::new(
            class_name.clone(),
            true,
            self.json_parse,
            self.variable_scope.clone(),
            self.lexical_scope.clone(),
            false,
            false,
            self.interner,
            self.in_with,
            spanned_source_text,
            self.source_path.clone(),
            &mut *self.scopes,
            self.control.reborrow(),
        )?;

        compiler.control.admit(CompilerWork::MetadataWrite)?;
        compiler.code_block_flags |= CodeBlockFlags::IS_CLASS_CONSTRUCTOR;

        let value = compiler.register_allocator.alloc(&mut compiler.control)?;
        if let Some(expr) = &class.constructor {
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
            let _ = compiler.push_scope(expr.scopes().function_scope())?;

            compiler.control.admit(CompilerWork::MetadataVisit)?;
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            compiler.length = expr.parameters().length();
            compiler.parameter_scope = expr.scopes().parameter_scope();

            compiler.function_declaration_instantiation(
                expr.body(),
                expr.parameters(),
                false,
                true,
                false,
                expr.scopes(),
            )?;

            {
                let mut compiler = compiler.position_guard(expr)?;
                compiler.compile_statement_list(expr.body().statement_list(), false, false)?;
                compiler.finish()?;
            }

            compiler
                .bytecode
                .emit_store_undefined(value.variable(), &mut compiler.control)?;
        } else if class.super_ref.is_some() {
            // We push an empty, unused function scope since the compiler expects a function scope.
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
            let default_scope = compiler.new_scope(compiler.lexical_scope.clone(), true)?;
            let _ = compiler.push_scope(&default_scope)?;
            compiler
                .bytecode
                .emit_super_call_derived(&mut compiler.control)?;
            compiler.pop_into_register(&value)?;
            compiler
                .bytecode
                .emit_bind_this_value(value.variable(), &mut compiler.control)?;
        } else {
            // We push an empty, unused function scope since the compiler expects a function scope.
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
            let default_scope = compiler.new_scope(compiler.lexical_scope.clone(), true)?;
            let _ = compiler.push_scope(&default_scope)?;
            compiler
                .bytecode
                .emit_store_undefined(value.variable(), &mut compiler.control)?;
        }
        compiler
            .bytecode
            .emit_set_accumulator(value.variable(), &mut compiler.control)?;
        compiler
            .register_allocator
            .dealloc(value, &mut compiler.control)?;

        // 17. If ClassHeritageopt is present, set F.[[ConstructorKind]] to derived.
        compiler.control.admit(CompilerWork::MetadataWrite)?;
        compiler.code_block_flags.set(
            CodeBlockFlags::IS_DERIVED_CONSTRUCTOR,
            class.super_ref.is_some(),
        );

        prepare_code_storage(&mut compiler.control)?;
        let code = Rc::new(
            compiler.finish(
                class
                    .constructor
                    .map(|constructor| constructor.parameters()),
            )?,
        );
        drop(_constructor_depth);
        let index = self.push_function_to_constants(code)?;

        let class_register = self.register_allocator.alloc(&mut self.control)?;
        self.emit_get_function(&class_register, index)?;

        let prototype_register = self.register_allocator.alloc(&mut self.control)?;

        if let Some(node) = class.super_ref {
            self.compile_expr(node, &prototype_register)?;
            self.bytecode.emit_store_class_prototype(
                prototype_register.variable(),
                class_register.variable(),
                prototype_register.variable(),
                &mut self.control,
            )?;
        } else {
            self.bytecode
                .emit_store_undefined(prototype_register.variable(), &mut self.control)?;
        }

        let proto_register = self.register_allocator.alloc(&mut self.control)?;

        self.bytecode.emit_set_class_prototype(
            proto_register.variable(),
            prototype_register.variable(),
            class_register.variable(),
            &mut self.control,
        )?;
        self.register_allocator
            .dealloc(prototype_register, &mut self.control)?;

        let mut name_indices = thin_with_capacity(0, &mut self.control)?;
        for element in class.elements {
            self.control.admit(CompilerWork::Visit)?;
            self.control.admit(CompilerWork::Read {
                bytes: payload_bytes::<ClassElement>(1)?,
            })?;
            match element {
                ClassElement::MethodDefinition(m) => {
                    if let ClassElementName::PrivateName(name) = m.name() {
                        let index = self.get_or_insert_private_name(*name)?;
                        thin_push(&mut name_indices, index, &mut self.control)?;
                    }
                }
                ClassElement::PrivateFieldDefinition(field)
                | ClassElement::PrivateStaticFieldDefinition(field) => {
                    let index = self.get_or_insert_private_name(*field.name())?;
                    thin_push(&mut name_indices, index, &mut self.control)?;
                }
                _ => {}
            }
        }
        self.bytecode.emit_push_private_environment(
            class_register.variable(),
            name_indices,
            &mut self.control,
        )?;

        self.control.admit(CompilerWork::MetadataWrite)?;
        let mut static_elements = Vec::new();

        if let Some(scope) = class.name_scope {
            let binding = self.scope_identifier_reference(scope, class_name.clone())?;
            let index = self.insert_binding(binding)?;
            self.emit_binding_access(
                BindingAccessOpcode::PutLexicalValue,
                &index,
                &class_register,
            )?;
        }

        for element in class.elements {
            self.control.admit(CompilerWork::Visit)?;
            self.control.admit(CompilerWork::Read {
                bytes: payload_bytes::<ClassElement>(1)?,
            })?;
            match element {
                ClassElement::MethodDefinition(m) => match m.name() {
                    ClassElementName::PropertyName(PropertyName::Literal(name)) => {
                        let index = self.get_or_insert_name(name.sym())?;
                        let method = self.method(m.into())?;

                        let object_register = if m.is_static() {
                            &class_register
                        } else {
                            &proto_register
                        };

                        match (m.is_static(), m.kind()) {
                            (true, MethodDefinitionKind::Get) => {
                                self.bytecode.emit_define_class_static_getter_by_name(
                                    method.variable(),
                                    object_register.variable(),
                                    index.into(),
                                    &mut self.control,
                                )?;
                            }
                            (true, MethodDefinitionKind::Set) => {
                                self.bytecode.emit_define_class_static_setter_by_name(
                                    method.variable(),
                                    object_register.variable(),
                                    index.into(),
                                    &mut self.control,
                                )?;
                            }
                            (true, _) => {
                                self.bytecode.emit_define_class_static_method_by_name(
                                    method.variable(),
                                    object_register.variable(),
                                    index.into(),
                                    &mut self.control,
                                )?;
                            }
                            (false, MethodDefinitionKind::Get) => {
                                self.bytecode.emit_define_class_getter_by_name(
                                    method.variable(),
                                    object_register.variable(),
                                    index.into(),
                                    &mut self.control,
                                )?;
                            }
                            (false, MethodDefinitionKind::Set) => {
                                self.bytecode.emit_define_class_setter_by_name(
                                    method.variable(),
                                    object_register.variable(),
                                    index.into(),
                                    &mut self.control,
                                )?;
                            }
                            (false, _) => {
                                self.bytecode.emit_define_class_method_by_name(
                                    method.variable(),
                                    object_register.variable(),
                                    index.into(),
                                    &mut self.control,
                                )?;
                            }
                        }

                        self.register_allocator.dealloc(method, &mut self.control)?;
                    }
                    ClassElementName::PropertyName(PropertyName::Computed(name)) => {
                        let key = self.register_allocator.alloc(&mut self.control)?;
                        self.compile_expr(name, &key)?;
                        self.bytecode.emit_to_property_key(
                            key.variable(),
                            key.variable(),
                            &mut self.control,
                        )?;
                        let method = self.method(m.into())?;
                        let object_register = if m.is_static() {
                            &class_register
                        } else {
                            &proto_register
                        };

                        match (m.is_static(), m.kind()) {
                            (true, MethodDefinitionKind::Get) => {
                                self.bytecode.emit_define_class_static_getter_by_value(
                                    method.variable(),
                                    key.variable(),
                                    object_register.variable(),
                                    &mut self.control,
                                )?;
                            }
                            (true, MethodDefinitionKind::Set) => {
                                self.bytecode.emit_define_class_static_setter_by_value(
                                    method.variable(),
                                    key.variable(),
                                    object_register.variable(),
                                    &mut self.control,
                                )?;
                            }
                            (true, _) => {
                                self.bytecode.emit_define_class_static_method_by_value(
                                    method.variable(),
                                    key.variable(),
                                    object_register.variable(),
                                    &mut self.control,
                                )?;
                            }
                            (false, MethodDefinitionKind::Get) => {
                                self.bytecode.emit_define_class_getter_by_value(
                                    method.variable(),
                                    key.variable(),
                                    object_register.variable(),
                                    &mut self.control,
                                )?;
                            }
                            (false, MethodDefinitionKind::Set) => {
                                self.bytecode.emit_define_class_setter_by_value(
                                    method.variable(),
                                    key.variable(),
                                    object_register.variable(),
                                    &mut self.control,
                                )?;
                            }
                            (false, _) => {
                                self.bytecode.emit_define_class_method_by_value(
                                    method.variable(),
                                    key.variable(),
                                    object_register.variable(),
                                    &mut self.control,
                                )?;
                            }
                        }

                        self.register_allocator.dealloc(method, &mut self.control)?;
                        self.register_allocator.dealloc(key, &mut self.control)?;
                    }
                    ClassElementName::PrivateName(name) => {
                        let index = self.get_or_insert_private_name(*name)?;
                        let method = self.method(m.into())?;
                        match (m.is_static(), m.kind()) {
                            (true, MethodDefinitionKind::Get) => {
                                self.bytecode.emit_set_private_getter(
                                    class_register.variable(),
                                    method.variable(),
                                    index.into(),
                                    &mut self.control,
                                )?;
                            }
                            (true, MethodDefinitionKind::Set) => {
                                self.bytecode.emit_set_private_setter(
                                    class_register.variable(),
                                    method.variable(),
                                    index.into(),
                                    &mut self.control,
                                )?;
                            }
                            (true, _) => {
                                self.bytecode.emit_set_private_method(
                                    class_register.variable(),
                                    method.variable(),
                                    index.into(),
                                    &mut self.control,
                                )?;
                            }
                            (false, MethodDefinitionKind::Get) => {
                                self.bytecode.emit_push_class_private_getter(
                                    class_register.variable(),
                                    method.variable(),
                                    index.into(),
                                    &mut self.control,
                                )?;
                            }
                            (false, MethodDefinitionKind::Set) => {
                                self.bytecode.emit_push_class_private_setter(
                                    class_register.variable(),
                                    method.variable(),
                                    index.into(),
                                    &mut self.control,
                                )?;
                            }
                            (false, _) => {
                                self.bytecode.emit_push_class_private_method(
                                    class_register.variable(),
                                    proto_register.variable(),
                                    method.variable(),
                                    index.into(),
                                    &mut self.control,
                                )?;
                            }
                        }
                        self.register_allocator.dealloc(method, &mut self.control)?;
                    }
                },
                ClassElement::FieldDefinition(field) => {
                    let name = self.register_allocator.alloc(&mut self.control)?;
                    match field.name() {
                        PropertyName::Literal(ident) => {
                            let literal =
                                ident.try_to_js_string(self.interner, &mut self.control)?;
                            self.emit_store_literal(Literal::String(literal), &name)?;
                        }
                        PropertyName::Computed(expr) => {
                            self.compile_expr(expr, &name)?;
                        }
                    }
                    let _field_depth = self.control.enter()?;
                    let mut field_compiler = ByteCompiler::new(
                        js_string!(),
                        true,
                        self.json_parse,
                        self.variable_scope.clone(),
                        self.lexical_scope.clone(),
                        false,
                        false,
                        self.interner,
                        self.in_with,
                        self.spanned_source_text.clone_only_source(),
                        self.source_path.clone(),
                        &mut *self.scopes,
                        self.control.reborrow(),
                    )?;

                    // Function environment
                    field_compiler.control.admit(CompilerWork::MetadataWrite)?;
                    field_compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
                    let _ = field_compiler.push_scope(field.scope())?;
                    let value = field_compiler
                        .register_allocator
                        .alloc(&mut field_compiler.control)?;
                    let is_anonymous_function = if let Some(node) = &field.initializer() {
                        field_compiler.compile_expr(node, &value)?;
                        field_compiler.is_anonymous_function_definition(node)?
                    } else {
                        field_compiler
                            .bytecode
                            .emit_store_undefined(value.variable(), &mut field_compiler.control)?;
                        false
                    };

                    field_compiler
                        .bytecode
                        .emit_set_accumulator(value.variable(), &mut field_compiler.control)?;
                    field_compiler
                        .register_allocator
                        .dealloc(value, &mut field_compiler.control)?;

                    field_compiler.control.admit(CompilerWork::MetadataWrite)?;
                    field_compiler.code_block_flags |= CodeBlockFlags::IN_CLASS_FIELD_INITIALIZER;

                    prepare_code_storage(&mut field_compiler.control)?;
                    let code = Rc::new(field_compiler.finish(None)?);
                    drop(_field_depth);
                    let index = self.push_function_to_constants(code)?;

                    let dst = self.register_allocator.alloc(&mut self.control)?;
                    self.emit_get_function(&dst, index)?;
                    self.bytecode.emit_push_class_field(
                        class_register.variable(),
                        name.variable(),
                        dst.variable(),
                        is_anonymous_function.into(),
                        &mut self.control,
                    )?;
                    self.register_allocator.dealloc(name, &mut self.control)?;
                    self.register_allocator.dealloc(dst, &mut self.control)?;
                }
                ClassElement::PrivateFieldDefinition(field) => {
                    let name_index = self.get_or_insert_private_name(*field.name())?;
                    let _field_depth = self.control.enter()?;
                    let mut field_compiler = ByteCompiler::new(
                        class_name.clone(),
                        true,
                        self.json_parse,
                        self.variable_scope.clone(),
                        self.lexical_scope.clone(),
                        false,
                        false,
                        self.interner,
                        self.in_with,
                        self.spanned_source_text.clone_only_source(),
                        self.source_path.clone(),
                        &mut *self.scopes,
                        self.control.reborrow(),
                    )?;
                    field_compiler.control.admit(CompilerWork::MetadataWrite)?;
                    field_compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
                    let _ = field_compiler.push_scope(field.scope())?;
                    let value = field_compiler
                        .register_allocator
                        .alloc(&mut field_compiler.control)?;
                    if let Some(node) = field.initializer() {
                        field_compiler.compile_expr(node, &value)?;
                    } else {
                        field_compiler
                            .bytecode
                            .emit_store_undefined(value.variable(), &mut field_compiler.control)?;
                    }
                    field_compiler
                        .bytecode
                        .emit_set_accumulator(value.variable(), &mut field_compiler.control)?;
                    field_compiler
                        .register_allocator
                        .dealloc(value, &mut field_compiler.control)?;

                    field_compiler.control.admit(CompilerWork::MetadataWrite)?;
                    field_compiler.code_block_flags |= CodeBlockFlags::IN_CLASS_FIELD_INITIALIZER;

                    prepare_code_storage(&mut field_compiler.control)?;
                    let code = Rc::new(field_compiler.finish(None)?);
                    drop(_field_depth);
                    let index = self.push_function_to_constants(code)?;
                    let dst = self.register_allocator.alloc(&mut self.control)?;
                    self.emit_get_function(&dst, index)?;
                    self.bytecode.emit_push_class_field_private(
                        class_register.variable(),
                        dst.variable(),
                        name_index.into(),
                        &mut self.control,
                    )?;
                    self.register_allocator.dealloc(dst, &mut self.control)?;
                }
                ClassElement::StaticFieldDefinition(field) => {
                    let name_index = match field.name() {
                        PropertyName::Literal(name) => {
                            StaticFieldName::Index(self.get_or_insert_name(name.sym())?)
                        }
                        PropertyName::Computed(name) => {
                            let name_register = self.register_allocator.alloc(&mut self.control)?;
                            self.compile_expr(name, &name_register)?;
                            StaticFieldName::Register(name_register)
                        }
                    };
                    let _field_depth = self.control.enter()?;
                    let mut field_compiler = ByteCompiler::new(
                        class_name.clone(),
                        true,
                        self.json_parse,
                        self.variable_scope.clone(),
                        self.lexical_scope.clone(),
                        false,
                        false,
                        self.interner,
                        self.in_with,
                        self.spanned_source_text.clone_only_source(),
                        self.source_path.clone(),
                        &mut *self.scopes,
                        self.control.reborrow(),
                    )?;
                    field_compiler.control.admit(CompilerWork::MetadataWrite)?;
                    field_compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
                    let _ = field_compiler.push_scope(field.scope())?;
                    let value = field_compiler
                        .register_allocator
                        .alloc(&mut field_compiler.control)?;
                    let is_anonymous_function = if let Some(node) = &field.initializer() {
                        field_compiler.compile_expr(node, &value)?;
                        field_compiler.is_anonymous_function_definition(node)?
                    } else {
                        field_compiler
                            .bytecode
                            .emit_store_undefined(value.variable(), &mut field_compiler.control)?;
                        false
                    };

                    field_compiler
                        .bytecode
                        .emit_set_accumulator(value.variable(), &mut field_compiler.control)?;
                    field_compiler
                        .register_allocator
                        .dealloc(value, &mut field_compiler.control)?;

                    field_compiler.control.admit(CompilerWork::MetadataWrite)?;
                    field_compiler.code_block_flags |= CodeBlockFlags::IN_CLASS_FIELD_INITIALIZER;

                    prepare_code_storage(&mut field_compiler.control)?;
                    let code = field_compiler.finish(None)?;
                    drop(_field_depth);
                    let code = Rc::new(code);

                    reserve(&mut static_elements, 1, &mut self.control)?;
                    self.control.admit(CompilerWork::Copy {
                        bytes: payload_bytes::<StaticElement>(1)?,
                    })?;
                    self.control.admit(CompilerWork::MetadataWrite)?;
                    static_elements.push(StaticElement::StaticField {
                        code,
                        name_index,
                        is_anonymous_function,
                    });
                }
                ClassElement::PrivateStaticFieldDefinition(field) => {
                    let name_index = self.get_or_insert_private_name(*field.name())?;
                    let _field_depth = self.control.enter()?;
                    let mut field_compiler = ByteCompiler::new(
                        class_name.clone(),
                        true,
                        self.json_parse,
                        self.variable_scope.clone(),
                        self.lexical_scope.clone(),
                        false,
                        false,
                        self.interner,
                        self.in_with,
                        self.spanned_source_text.clone_only_source(),
                        self.source_path.clone(),
                        &mut *self.scopes,
                        self.control.reborrow(),
                    )?;
                    field_compiler.control.admit(CompilerWork::MetadataWrite)?;
                    field_compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
                    let _ = field_compiler.push_scope(field.scope())?;
                    let value = field_compiler
                        .register_allocator
                        .alloc(&mut field_compiler.control)?;
                    let is_anonymous_function = if let Some(node) = &field.initializer() {
                        field_compiler.compile_expr(node, &value)?;
                        field_compiler.is_anonymous_function_definition(node)?
                    } else {
                        field_compiler
                            .bytecode
                            .emit_store_undefined(value.variable(), &mut field_compiler.control)?;
                        false
                    };

                    field_compiler
                        .bytecode
                        .emit_set_accumulator(value.variable(), &mut field_compiler.control)?;
                    field_compiler
                        .register_allocator
                        .dealloc(value, &mut field_compiler.control)?;

                    field_compiler.control.admit(CompilerWork::MetadataWrite)?;
                    field_compiler.code_block_flags |= CodeBlockFlags::IN_CLASS_FIELD_INITIALIZER;

                    prepare_code_storage(&mut field_compiler.control)?;
                    let code = field_compiler.finish(None)?;
                    drop(_field_depth);
                    let code = Rc::new(code);

                    reserve(&mut static_elements, 1, &mut self.control)?;
                    self.control.admit(CompilerWork::Copy {
                        bytes: payload_bytes::<StaticElement>(1)?,
                    })?;
                    self.control.admit(CompilerWork::MetadataWrite)?;
                    static_elements.push(StaticElement::StaticField {
                        code,
                        name_index: StaticFieldName::PrivateName(name_index),
                        is_anonymous_function,
                    });
                }
                ClassElement::StaticBlock(block) => {
                    let _static_depth = self.control.enter()?;
                    let mut compiler = ByteCompiler::new(
                        js_string!(),
                        true,
                        false,
                        self.variable_scope.clone(),
                        self.lexical_scope.clone(),
                        false,
                        false,
                        self.interner,
                        self.in_with,
                        self.spanned_source_text.clone_only_source(),
                        self.source_path.clone(),
                        &mut *self.scopes,
                        self.control.reborrow(),
                    )?;
                    compiler.control.admit(CompilerWork::MetadataWrite)?;
                    compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
                    let _ = compiler.push_scope(block.scopes().function_scope())?;

                    compiler.function_declaration_instantiation(
                        block.statements(),
                        &FormalParameterList::default(),
                        false,
                        true,
                        false,
                        block.scopes(),
                    )?;

                    {
                        let mut compiler = compiler.position_guard(block.statements())?;
                        compiler.compile_statement_list(
                            block.statements().statement_list(),
                            false,
                            false,
                        )?;
                        compiler.finish()?;
                    }

                    prepare_code_storage(&mut compiler.control)?;
                    let code = Rc::new(compiler.finish(None)?);
                    drop(_static_depth);
                    reserve(&mut static_elements, 1, &mut self.control)?;
                    self.control.admit(CompilerWork::Copy {
                        bytes: payload_bytes::<StaticElement>(1)?,
                    })?;
                    self.control.admit(CompilerWork::MetadataWrite)?;
                    static_elements.push(StaticElement::StaticBlock(code));
                }
            }
        }

        for element in static_elements {
            self.control.admit(CompilerWork::MetadataVisit)?;
            self.control.admit(CompilerWork::Read {
                bytes: payload_bytes::<StaticElement>(1)?,
            })?;
            self.control.admit(CompilerWork::Copy {
                bytes: payload_bytes::<StaticElement>(1)?,
            })?;
            match element {
                StaticElement::StaticBlock(code) => {
                    let index = self.push_function_to_constants(code)?;
                    let function = self.register_allocator.alloc(&mut self.control)?;
                    self.emit_get_function(&function, index)?;
                    self.bytecode.emit_set_home_object(
                        function.variable(),
                        class_register.variable(),
                        &mut self.control,
                    )?;
                    self.push_from_register(&class_register)?;
                    self.push_from_register(&function)?;
                    self.register_allocator
                        .dealloc(function, &mut self.control)?;
                    self.bytecode.emit_call(0u32.into(), &mut self.control)?;
                    self.bytecode.emit_pop(&mut self.control)?;
                }
                StaticElement::StaticField {
                    code,
                    name_index,
                    is_anonymous_function,
                } => {
                    let index = self.push_function_to_constants(code)?;
                    let function = self.register_allocator.alloc(&mut self.control)?;
                    self.emit_get_function(&function, index)?;
                    self.bytecode.emit_set_home_object(
                        function.variable(),
                        class_register.variable(),
                        &mut self.control,
                    )?;
                    self.push_from_register(&class_register)?;
                    self.push_from_register(&function)?;
                    self.register_allocator
                        .dealloc(function, &mut self.control)?;
                    self.bytecode.emit_call(0u32.into(), &mut self.control)?;
                    let value = self.register_allocator.alloc(&mut self.control)?;
                    self.pop_into_register(&value)?;
                    match name_index {
                        StaticFieldName::PrivateName(name) => {
                            self.bytecode.emit_define_private_field(
                                class_register.variable(),
                                value.variable(),
                                name.into(),
                                &mut self.control,
                            )?;
                        }
                        StaticFieldName::Index(name) => {
                            self.bytecode.emit_define_own_property_by_name(
                                class_register.variable(),
                                value.variable(),
                                name.into(),
                                &mut self.control,
                            )?;
                        }
                        StaticFieldName::Register(key) => {
                            if is_anonymous_function {
                                self.bytecode.emit_to_property_key(
                                    key.variable(),
                                    key.variable(),
                                    &mut self.control,
                                )?;
                                self.bytecode.emit_set_function_name(
                                    value.variable(),
                                    key.variable(),
                                    0u32.into(),
                                    &mut self.control,
                                )?;
                            }
                            self.bytecode.emit_define_own_property_by_value(
                                value.variable(),
                                key.variable(),
                                class_register.variable(),
                                &mut self.control,
                            )?;
                            self.register_allocator.dealloc(key, &mut self.control)?;
                        }
                    }

                    self.register_allocator.dealloc(value, &mut self.control)?;
                }
            }
        }

        self.register_allocator
            .dealloc(proto_register, &mut self.control)?;

        self.pop_declarative_scope(outer_scope)?;
        self.bytecode
            .emit_pop_private_environment(&mut self.control)?;

        if let Some(dst) = dst {
            self.bytecode.emit_move(
                dst.variable(),
                class_register.variable(),
                &mut self.control,
            )?;
        } else {
            self.emit_binding(BindingOpcode::InitVar, class_name, &class_register)?;
        }

        self.register_allocator
            .dealloc(class_register, &mut self.control)?;

        // NOTE: Reset strict mode to before class declaration/expression evaluation.
        self.control.admit(CompilerWork::MetadataWrite)?;
        self.code_block_flags.set(CodeBlockFlags::STRICT, strict);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Context, Source,
        error::{EngineError, RuntimeLimitError},
        execution::{ExecutionOptions, ProfileMode},
        js_string,
    };

    const SOURCE: &str = "
        let trace = [];
        function key(name) { trace.push(name); return name; }
        class Base {
            static [key('first')] = 1;
            static #secret = 40;
            static { trace.push('block'); }
            [key('field')] = 2;
            #value = Base.#secret;
            constructor(a, b = 1) { this.n = a + b; }
            *values() { yield* [this.#value + this.n]; }
            static async answer() { return this.#secret + 2; }
            static get total() { return this.#secret + this.first; }
        }
        class Derived extends Base {}
        let instance = new Derived(1);
        let iterator = instance.values();
        [iterator.next().value, iterator.return(7).value,
         Base.length, Derived.length, Base.total, trace.join(',')].join('|');
    ";

    #[test]
    fn generated_class_compilers_share_admission_and_preserve_element_order_on_retry() {
        let expected = js_string!("42|7|1|0|41|first,field,block");
        let mut baseline = Context::builder().build().unwrap();
        let complete = baseline
            .eval_with_budget(Source::from_bytes(SOURCE), ExecutionOptions::new(4096))
            .unwrap();
        assert_eq!(complete.result.unwrap(), expected.clone().into());
        let work = complete.report.compiler_work().unwrap();
        let peak = complete.report.compiler_depth_peak();
        assert!(work > 0);
        assert!(peak > 1);
        for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
            let mut exact = Context::builder().build().unwrap();
            let exact = exact
                .eval_with_budget(
                    Source::from_bytes(SOURCE),
                    ExecutionOptions::new(4096)
                        .with_profile(profile)
                        .with_compiler_work_limit(work)
                        .with_compiler_depth_limit(peak),
                )
                .unwrap();
            assert_eq!(exact.result.unwrap(), expected.clone().into());
            assert_eq!(exact.report.compiler_work().unwrap(), work);
            assert_eq!(exact.report.compiler_depth_peak(), peak);

            let limit = work - 1;
            let mut retry = Context::builder().build().unwrap();
            let stopped = retry
                .eval_with_budget(
                    Source::from_bytes(SOURCE),
                    ExecutionOptions::new(4096)
                        .with_profile(profile)
                        .with_compiler_work_limit(limit),
                )
                .unwrap();
            assert_eq!(
                stopped.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::CompilerWork { limit },
                ))
            );
            assert!(!stopped.report.effects_started());
            assert_eq!(
                retry.eval(Source::from_bytes(SOURCE)).unwrap(),
                expected.clone().into()
            );
        }
    }
}
