use crate::JsResult;
use crate::bytecompiler::control::{CompilerWork, reserve, thin_push, thin_with_capacity};
use crate::{
    bytecompiler::{Access, ByteCompiler, Literal, Register, ToJsString},
    vm::opcode::BindingOpcode,
};
use surgeist_js_ast::{
    pattern::{ArrayPatternElement, ObjectPatternElement, Pattern},
    property::PropertyName,
};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_declaration_pattern_impl(
        &mut self,
        pattern: &Pattern,
        def: BindingOpcode,
        object: &Register,
    ) -> JsResult<()> {
        match pattern {
            Pattern::Object(pattern) => {
                self.bytecode
                    .emit_value_not_null_or_undefined(object.variable(), &mut self.control)?;

                let mut excluded_keys_registers = Vec::new();
                self.control.admit(CompilerWork::Search)?;
                let rest_exits = pattern.has_rest();

                for binding in pattern.bindings() {
                    self.control.admit(CompilerWork::Visit)?;
                    use ObjectPatternElement::{
                        AssignmentPropertyAccess, AssignmentRestPropertyAccess, Pattern,
                        RestProperty, SingleName,
                    };

                    match binding {
                        //  SingleNameBinding : BindingIdentifier Initializer[opt]
                        SingleName {
                            ident,
                            name,
                            default_init,
                        } => {
                            let dst = self.register_allocator.alloc(&mut self.control)?;

                            self.control.admit(CompilerWork::Visit)?;
                            match name {
                                PropertyName::Literal(ident) => {
                                    self.emit_get_property_by_name(
                                        &dst,
                                        None,
                                        object,
                                        ident.sym(),
                                    )?;
                                    let key = self.register_allocator.alloc(&mut self.control)?;
                                    let string = ident
                                        .try_to_js_string(&*self.interner, &mut self.control)?;
                                    self.emit_store_literal(Literal::String(string), &key)?;
                                    self.retain_pattern_key(&mut excluded_keys_registers, key)?;
                                }
                                PropertyName::Computed(node) => {
                                    let key = self.register_allocator.alloc(&mut self.control)?;
                                    self.compile_expr(node, &key)?;
                                    if rest_exits {
                                        self.bytecode.emit_get_property_by_value_push(
                                            dst.variable(),
                                            key.variable(),
                                            object.variable(),
                                            object.variable(),
                                            &mut self.control,
                                        )?;
                                        self.retain_pattern_key(&mut excluded_keys_registers, key)?;
                                    } else {
                                        self.bytecode.emit_get_property_by_value(
                                            dst.variable(),
                                            key.variable(),
                                            object.variable(),
                                            object.variable(),
                                            &mut self.control,
                                        )?;
                                        self.register_allocator.dealloc(key, &mut self.control)?;
                                    }
                                }
                            }

                            self.control.admit(CompilerWork::Visit)?;
                            if let Some(init) = default_init {
                                let skip = self.jump_if_not_undefined(&dst)?;
                                self.compile_expr(init, &dst)?;
                                self.patch_jump(skip)?;
                            }

                            let binding_name =
                                ident.try_to_js_string(&*self.interner, &mut self.control)?;

                            self.emit_binding(def, binding_name, &dst)?;
                            self.register_allocator.dealloc(dst, &mut self.control)?;
                        }
                        //  BindingRestProperty : ... BindingIdentifier
                        RestProperty { ident } => {
                            let value = self.register_allocator.alloc(&mut self.control)?;
                            self.bytecode
                                .emit_store_empty_object(value.variable(), &mut self.control)?;
                            let mut excluded_keys = thin_with_capacity(
                                excluded_keys_registers.len(),
                                &mut self.control,
                            )?;
                            for r in &excluded_keys_registers {
                                self.control.admit(CompilerWork::MetadataVisit)?;
                                thin_push(&mut excluded_keys, r.variable(), &mut self.control)?;
                            }
                            self.bytecode.emit_copy_data_properties(
                                value.variable(),
                                object.variable(),
                                excluded_keys,
                                &mut self.control,
                            )?;
                            self.release_pattern_keys(&mut excluded_keys_registers)?;
                            let binding_name =
                                ident.try_to_js_string(&*self.interner, &mut self.control)?;
                            self.emit_binding(def, binding_name, &value)?;
                            self.register_allocator.dealloc(value, &mut self.control)?;
                        }
                        AssignmentRestPropertyAccess { access } => {
                            let value = self.register_allocator.alloc(&mut self.control)?;
                            self.bytecode
                                .emit_store_empty_object(value.variable(), &mut self.control)?;
                            let mut excluded_keys = thin_with_capacity(
                                excluded_keys_registers.len(),
                                &mut self.control,
                            )?;
                            for r in &excluded_keys_registers {
                                self.control.admit(CompilerWork::MetadataVisit)?;
                                thin_push(&mut excluded_keys, r.variable(), &mut self.control)?;
                            }
                            self.bytecode.emit_copy_data_properties(
                                value.variable(),
                                object.variable(),
                                excluded_keys,
                                &mut self.control,
                            )?;
                            self.release_pattern_keys(&mut excluded_keys_registers)?;
                            self.access_set(Access::Property { access }, |_| Ok(&value))?;
                            self.register_allocator.dealloc(value, &mut self.control)?;
                        }
                        AssignmentPropertyAccess {
                            name,
                            access,
                            default_init,
                        } => {
                            let key = self.register_allocator.alloc(&mut self.control)?;
                            self.control.admit(CompilerWork::Visit)?;
                            match &name {
                                PropertyName::Literal(ident) => {
                                    let key = self.register_allocator.alloc(&mut self.control)?;
                                    let string = ident
                                        .try_to_js_string(&*self.interner, &mut self.control)?;
                                    self.emit_store_literal(Literal::String(string), &key)?;
                                    self.retain_pattern_key(&mut excluded_keys_registers, key)?;
                                }
                                PropertyName::Computed(node) => {
                                    self.compile_expr(node, &key)?;
                                    self.bytecode.emit_to_property_key(
                                        key.variable(),
                                        key.variable(),
                                        &mut self.control,
                                    )?;
                                }
                            }

                            let dst = self.register_allocator.alloc(&mut self.control)?;
                            self.access_set(
                                Access::Property { access },
                                |compiler: &mut ByteCompiler<'_, '_>| {
                                    compiler.control.admit(CompilerWork::Visit)?;
                                    match name {
                                        PropertyName::Literal(ident) => {
                                            compiler.emit_get_property_by_name(
                                                &dst,
                                                None,
                                                object,
                                                ident.sym(),
                                            )?;
                                            compiler
                                                .register_allocator
                                                .dealloc(key, &mut compiler.control)?;
                                        }
                                        PropertyName::Computed(_) => {
                                            if rest_exits {
                                                compiler.bytecode.emit_get_property_by_value_push(
                                                    dst.variable(),
                                                    key.variable(),
                                                    object.variable(),
                                                    object.variable(),
                                                    &mut compiler.control,
                                                )?;
                                                compiler.retain_pattern_key(
                                                    &mut excluded_keys_registers,
                                                    key,
                                                )?;
                                            } else {
                                                compiler.bytecode.emit_get_property_by_value(
                                                    dst.variable(),
                                                    key.variable(),
                                                    object.variable(),
                                                    object.variable(),
                                                    &mut compiler.control,
                                                )?;
                                                compiler
                                                    .register_allocator
                                                    .dealloc(key, &mut compiler.control)?;
                                            }
                                        }
                                    }

                                    compiler.control.admit(CompilerWork::Visit)?;
                                    if let Some(init) = default_init {
                                        let skip = compiler.jump_if_not_undefined(&dst)?;
                                        compiler.compile_expr(init, &dst)?;
                                        compiler.patch_jump(skip)?;
                                    }

                                    Ok(&dst)
                                },
                            )?;
                            self.register_allocator.dealloc(dst, &mut self.control)?;
                        }
                        Pattern {
                            name,
                            pattern,
                            default_init,
                        } => {
                            let dst = self.register_allocator.alloc(&mut self.control)?;

                            self.control.admit(CompilerWork::Visit)?;
                            match name {
                                PropertyName::Literal(ident) => {
                                    self.emit_get_property_by_name(
                                        &dst,
                                        None,
                                        object,
                                        ident.sym(),
                                    )?;
                                }
                                PropertyName::Computed(node) => {
                                    let key = self.register_allocator.alloc(&mut self.control)?;
                                    self.compile_expr(node, &key)?;
                                    self.bytecode.emit_get_property_by_value(
                                        dst.variable(),
                                        key.variable(),
                                        object.variable(),
                                        object.variable(),
                                        &mut self.control,
                                    )?;
                                    self.register_allocator.dealloc(key, &mut self.control)?;
                                }
                            }

                            self.control.admit(CompilerWork::Visit)?;
                            if let Some(init) = default_init {
                                let skip = self.jump_if_not_undefined(&dst)?;
                                self.compile_expr(init, &dst)?;
                                self.patch_jump(skip)?;
                            }
                            self.compile_declaration_pattern(pattern, def, &dst)?;
                            self.register_allocator.dealloc(dst, &mut self.control)?;
                        }
                    }
                }

                self.release_pattern_keys(&mut excluded_keys_registers)?;
            }
            Pattern::Array(pattern) => {
                self.bytecode
                    .emit_value_not_null_or_undefined(object.variable(), &mut self.control)?;
                self.bytecode
                    .emit_get_iterator(object.variable(), &mut self.control)?;

                let handler_index = self.push_handler()?;
                for element in pattern.bindings() {
                    self.compile_array_pattern_element(element, def)?;
                }

                let no_exception_thrown = self.jump()?;
                self.patch_handler(handler_index)?;

                let has_exception = self.register_allocator.alloc(&mut self.control)?;
                let exception = self.register_allocator.alloc(&mut self.control)?;
                self.bytecode.emit_maybe_exception(
                    has_exception.variable(),
                    exception.variable(),
                    &mut self.control,
                )?;

                let iterator_close_handler = self.push_handler()?;
                self.iterator_close(false)?;
                self.patch_handler(iterator_close_handler)?;

                let jump = self.jump_if_false(&has_exception)?;
                self.register_allocator
                    .dealloc(has_exception, &mut self.control)?;

                self.bytecode
                    .emit_throw(exception.variable(), &mut self.control)?;
                self.register_allocator
                    .dealloc(exception, &mut self.control)?;

                self.patch_jump(jump)?;
                self.bytecode.emit_re_throw(&mut self.control)?;

                self.patch_jump(no_exception_thrown)?;

                self.iterator_close(false)?;
            }
        }
        Ok(())
    }

    fn retain_pattern_key(&mut self, keys: &mut Vec<Register>, key: Register) -> JsResult<()> {
        reserve(keys, 1, &mut self.control)?;
        self.control.admit(CompilerWork::MetadataWrite)?;
        self.control.admit(CompilerWork::Copy {
            bytes: std::mem::size_of::<Register>(),
        })?;
        keys.push(key);
        Ok(())
    }

    fn release_pattern_keys(&mut self, keys: &mut Vec<Register>) -> JsResult<()> {
        while !keys.is_empty() {
            self.control.admit(CompilerWork::MetadataVisit)?;
            self.control.admit(CompilerWork::MetadataWrite)?;
            let key = keys.pop().expect("a nonempty pattern-key list");
            self.register_allocator.dealloc(key, &mut self.control)?;
        }
        Ok(())
    }

    fn compile_array_pattern_element(
        &mut self,
        element: &ArrayPatternElement,
        def: BindingOpcode,
    ) -> JsResult<()> {
        self.control.admit(CompilerWork::Visit)?;
        use ArrayPatternElement::{
            Elision, Pattern, PatternRest, PropertyAccess, PropertyAccessRest, SingleName,
            SingleNameRest,
        };

        match element {
            // ArrayBindingPattern : [ Elision ]
            Elision => {
                self.bytecode.emit_iterator_next(&mut self.control)?;
            }
            // SingleNameBinding : BindingIdentifier Initializer[opt]
            SingleName {
                ident,
                default_init,
            } => {
                self.bytecode.emit_iterator_next(&mut self.control)?;
                let value = self.register_allocator.alloc(&mut self.control)?;
                self.bytecode
                    .emit_iterator_done(value.variable(), &mut self.control)?;
                self.if_else(
                    &value,
                    |compiler| {
                        compiler
                            .bytecode
                            .emit_store_undefined(value.variable(), &mut compiler.control)
                    },
                    |compiler| {
                        compiler
                            .bytecode
                            .emit_iterator_value(value.variable(), &mut compiler.control)
                    },
                )?;

                self.control.admit(CompilerWork::Visit)?;
                if let Some(init) = default_init {
                    let skip = self.jump_if_not_undefined(&value)?;
                    self.compile_expr(init, &value)?;
                    self.patch_jump(skip)?;
                }

                let binding_name = ident.try_to_js_string(&*self.interner, &mut self.control)?;

                self.emit_binding(def, binding_name, &value)?;
                self.register_allocator.dealloc(value, &mut self.control)?;
            }
            PropertyAccess {
                access,
                default_init,
            } => {
                let value = self.register_allocator.alloc(&mut self.control)?;
                self.access_set(Access::Property { access }, |compiler| {
                    compiler
                        .bytecode
                        .emit_iterator_next(&mut compiler.control)?;
                    compiler
                        .bytecode
                        .emit_iterator_done(value.variable(), &mut compiler.control)?;
                    compiler.if_else(
                        &value,
                        |compiler| {
                            compiler
                                .bytecode
                                .emit_store_undefined(value.variable(), &mut compiler.control)
                        },
                        |compiler| {
                            compiler
                                .bytecode
                                .emit_iterator_value(value.variable(), &mut compiler.control)
                        },
                    )?;

                    compiler.control.admit(CompilerWork::Visit)?;
                    if let Some(init) = default_init {
                        let skip = compiler.jump_if_not_undefined(&value)?;
                        compiler.compile_expr(init, &value)?;
                        compiler.patch_jump(skip)?;
                    }

                    Ok(&value)
                })?;
                self.register_allocator.dealloc(value, &mut self.control)?;
            }
            // BindingElement : BindingPattern Initializer[opt]
            Pattern {
                pattern,
                default_init,
            } => {
                self.bytecode.emit_iterator_next(&mut self.control)?;
                let value = self.register_allocator.alloc(&mut self.control)?;
                self.bytecode
                    .emit_iterator_done(value.variable(), &mut self.control)?;
                self.if_else(
                    &value,
                    |compiler| {
                        compiler
                            .bytecode
                            .emit_store_undefined(value.variable(), &mut compiler.control)
                    },
                    |compiler| {
                        compiler
                            .bytecode
                            .emit_iterator_value(value.variable(), &mut compiler.control)
                    },
                )?;

                self.control.admit(CompilerWork::Visit)?;
                if let Some(init) = default_init {
                    let skip = self.jump_if_not_undefined(&value)?;
                    self.compile_expr(init, &value)?;
                    self.patch_jump(skip)?;
                }
                self.compile_declaration_pattern(pattern, def, &value)?;
                self.register_allocator.dealloc(value, &mut self.control)?;
            }
            // BindingRestElement : ... BindingIdentifier
            SingleNameRest { ident } => {
                let value = self.register_allocator.alloc(&mut self.control)?;
                self.bytecode
                    .emit_iterator_to_array(value.variable(), &mut self.control)?;
                let binding_name = ident.try_to_js_string(&*self.interner, &mut self.control)?;
                self.emit_binding(def, binding_name, &value)?;
                self.register_allocator.dealloc(value, &mut self.control)?;
            }
            PropertyAccessRest { access } => {
                let value = self.register_allocator.alloc(&mut self.control)?;
                self.access_set(Access::Property { access }, |compiler| {
                    compiler
                        .bytecode
                        .emit_iterator_to_array(value.variable(), &mut compiler.control)?;
                    Ok(&value)
                })?;
                self.register_allocator.dealloc(value, &mut self.control)?;
            }
            // BindingRestElement : ... BindingPattern
            PatternRest { pattern } => {
                let value = self.register_allocator.alloc(&mut self.control)?;
                self.bytecode
                    .emit_iterator_to_array(value.variable(), &mut self.control)?;
                self.compile_declaration_pattern(pattern, def, &value)?;
                self.register_allocator.dealloc(value, &mut self.control)?;
            }
        }
        Ok(())
    }
}
